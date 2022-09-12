use super::calc_crc32;
use super::endian_from_bool;
use super::NuccBinaryParsed;
use super::NuccBinaryType;

use binary_stream::SeekStream;
use binary_stream::{BinaryReader, BinaryWriter, MemoryStream, SliceStream};
use deku::bitvec::BitVec;
use deku::bitvec::BitView;
use deku::ctx::Endian;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "Endian::Little"
)]
pub struct Entry {
    #[serde(skip)]
    pub entry_name_ptr: u64,
    #[deku(skip)]
    pub entry_name: String,

    #[deku(count = "4")]
    pub unk0: Vec<u32>,

    #[serde(skip)]
    pub char_name_ptr: u64,
    #[deku(skip)]
    pub char_name: String,

    pub unk1: u32,
    pub unk2: u32,

    pub unlock_status: u32,
    pub unk4: u32,

    pub shop_cost: u32,
    pub unk6: u32,

    #[serde(skip)]
    pub name_id_ptr: u64,
    #[deku(skip)]
    pub name_id: String,

    #[deku(skip)]
    #[serde(with = "hex::serde")]
    pub name_id_crc32_no_edit: Vec<u8>,

    #[serde(skip)]
    pub desc_id_ptr: u64,
    #[deku(skip)]
    pub desc_id: String,

    #[deku(skip)]
    #[serde(with = "hex::serde")]
    pub desc_id_crc32_no_edit: Vec<u8>,

    pub entry_number: u32,
    pub unk8: u32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SoundTestParam {
    pub unk0: u32,
    pub pointer_size: u32,
    pub entries: Vec<Entry>,

    big_endian: bool,
}

impl NuccBinaryParsed for SoundTestParam {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::SoundTestParam(endian_from_bool(self.big_endian))
    }

    fn extension(&self, _: bool) -> String {
        String::from(".json")
    }

    fn serialize(&self, _: bool) -> Vec<u8> {
        serde_json::to_string_pretty(self).unwrap().into()
    }

    fn deserialize(data: &[u8], _: bool) -> Self
    where
        Self: Sized,
    {
        serde_json::from_slice(data).unwrap()
    }
}

impl From<(&[u8], Endian)> for SoundTestParam {
    fn from(converter: (&[u8], Endian)) -> Self {
        fn read_string_pointer(reader: &mut BinaryReader, pointer: u64, offset: u64) -> String {
            if pointer != 0 {
                reader.seek(pointer - offset).unwrap();
                reader.read_string_null_terminated().unwrap()
            } else {
                String::from("")
            }
        }

        let (data, endian) = converter;
        let data = data.view_bits();

        let (data, unk0) = u32::read(data, endian).unwrap();
        let (data, entry_count) = u32::read(data, endian).unwrap();
        let (data, pointer_size) = u32::read(data, endian).unwrap();

        // Padding
        let (data, _) = u32::read(data, endian).unwrap();

        let mut entries = Vec::new();
        entries.reserve_exact(entry_count as usize);

        let mut data = data;
        for _ in 0..entry_count as usize {
            let (rest, entry) = Entry::read(data, endian).unwrap();
            entries.push(entry);
            data = rest;
        }

        let buffer = data.to_bitvec().into_vec();
        let mut stream = SliceStream::new(&buffer);
        let mut reader = BinaryReader::new(&mut stream, super::binary_stream_endian(endian));

        let entries_len = entries.len();
        for (start_offset, entry) in entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| ((0x50 * (entries_len - i)) as u64, e))
        {
            entry.entry_name = read_string_pointer(&mut reader, entry.entry_name_ptr, start_offset);
            entry.char_name =
                read_string_pointer(&mut reader, entry.char_name_ptr, start_offset - 0x18);
            entry.name_id =
                read_string_pointer(&mut reader, entry.name_id_ptr, start_offset - 0x38);
            entry.desc_id =
                read_string_pointer(&mut reader, entry.desc_id_ptr, start_offset - 0x40);

            entry.name_id_crc32_no_edit = calc_crc32(entry.name_id.as_bytes());
            entry.desc_id_crc32_no_edit = calc_crc32(entry.desc_id.as_bytes());
        }

        Self {
            unk0,
            pointer_size,
            entries,

            big_endian: endian == Endian::Big,
        }
    }
}

impl From<SoundTestParam> for Vec<u8> {
    fn from(mut parsed: SoundTestParam) -> Self {
        fn write_string_pointer(writer: &mut BinaryWriter, string: &str, offset: u64) -> u64 {
            let pos = writer.tell().unwrap();

            if !string.is_empty() {
                writer.write_string_null_terminated(string).unwrap();
                writer.align(8).unwrap();

                pos - offset
            } else {
                0
            }
        }

        let endian = endian_from_bool(parsed.big_endian);

        let mut stream = MemoryStream::new();
        let mut writer = BinaryWriter::new(&mut stream, super::binary_stream_endian(endian));

        writer.write_u32(parsed.unk0).unwrap();
        writer.write_u32(parsed.entries.len() as u32).unwrap();
        writer.write_u32(parsed.pointer_size).unwrap();

        writer
            .write_padding(4 + (0x50 * parsed.entries.len()) as u64)
            .unwrap();

        for (pointer_offset, entry) in parsed
            .entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| (((0x50 * i) + 0x10) as u64, e))
        {
            entry.entry_name_ptr =
                write_string_pointer(&mut writer, &entry.entry_name, pointer_offset);
            entry.char_name_ptr =
                write_string_pointer(&mut writer, &entry.char_name, pointer_offset + 0x18);
            entry.name_id_ptr =
                write_string_pointer(&mut writer, &entry.name_id, pointer_offset + 0x38);
            entry.desc_id_ptr =
                write_string_pointer(&mut writer, &entry.desc_id, pointer_offset + 0x40);

            let pos = writer.tell().unwrap();

            let mut output = BitVec::new();
            entry.write(&mut output, endian).unwrap();

            writer.seek(pointer_offset).unwrap();
            writer.write_bytes(output.into_vec()).unwrap();

            writer.seek(pos).unwrap();
        }

        stream.into()
    }
}
