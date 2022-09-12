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
    pub string_pointer: u32,

    pub unk0: u32,
    pub costume_index: u32,

    pub red: u32,
    pub green: u32,
    pub blue: u32,

    #[deku(skip)]
    pub string: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PlayerColorParam {
    pub unk0: u32,
    pub unk1: u32,
    pub entries: Vec<Entry>,

    big_endian: bool,
}

impl NuccBinaryParsed for PlayerColorParam {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::PlayerColorParam(endian_from_bool(self.big_endian))
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

impl From<(&[u8], Endian)> for PlayerColorParam {
    fn from(converter: (&[u8], Endian)) -> Self {
        let (data, endian) = converter;
        let data = data.view_bits();

        let (data, unk0) = u32::read(data, endian).unwrap();
        let (data, entry_count) = u32::read(data, endian).unwrap();
        let (data, unk1) = u32::read(data, endian).unwrap();

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
        for (end_offset, entry) in entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| ((0x18 * (entries_len - i)) as u32, e))
        {
            if entry.string_pointer != 0 {
                reader
                    .seek((entry.string_pointer - end_offset) as u64)
                    .unwrap();
                entry.string = reader.read_string_null_terminated().unwrap();
            } else {
                entry.string = String::from("");
            }
        }

        Self {
            unk0,
            unk1,
            entries,

            big_endian: endian == Endian::Big,
        }
    }
}

impl From<PlayerColorParam> for Vec<u8> {
    fn from(mut parsed: PlayerColorParam) -> Self {
        let endian = if parsed.big_endian {
            Endian::Big
        } else {
            Endian::Little
        };

        let mut stream = MemoryStream::new();
        let mut writer = BinaryWriter::new(&mut stream, super::binary_stream_endian(endian));

        writer.write_u32(parsed.unk0).unwrap();
        writer.write_u32(parsed.entries.len() as u32).unwrap();
        writer.write_u32(parsed.unk1).unwrap();

        writer
            .write_padding(4 + (0x18 * parsed.entries.len()) as u64)
            .unwrap();

        for (pointer_offset, entry) in parsed
            .entries
            .iter_mut()
            .enumerate()
            .map(|(i, e)| (((0x18 * i) + 0x10) as u32, e))
        {
            let pos = writer.tell().unwrap() as u32;

            entry.string_pointer = 0;
            if !entry.string.is_empty() {
                writer
                    .write_string_null_terminated(entry.string.clone())
                    .unwrap();
                writer.align(4).unwrap();

                entry.string_pointer = pos - pointer_offset;
            }

            let pos = writer.tell().unwrap();

            let mut output = BitVec::new();
            entry.write(&mut output, endian).unwrap();

            writer.seek(pointer_offset as u64).unwrap();
            writer.write_bytes(output.into_vec()).unwrap();

            writer.seek(pos).unwrap();
        }

        stream.into()
    }
}
