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
pub struct Prop {
    #[serde(skip)]
    pub xfbin_path_ptr: u64,
    #[deku(skip)]
    pub xfbin_path: String,

    #[serde(skip)]
    pub clump_name_ptr: u64,
    #[deku(skip)]
    pub clump_name: String,

    #[serde(skip)]
    pub string2_ptr: u64,
    #[deku(skip)]
    pub string2: String,

    #[serde(skip)]
    pub string3_ptr: u64,
    #[deku(skip)]
    pub string3: String,

    pub unk0: u32,
    pub unk1_float: f32,
    pub unk2: u32,
    pub unk3: u32,

    pub unk4: u32,
    pub unk5: u32,
}

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

    #[serde(skip)]
    #[deku(update = "self.xfbin_paths.len() as u64")]
    pub xfbin_paths_count: u64,

    #[serde(skip)]
    pub xfbin_paths_ptr: u64,
    #[deku(skip)]
    pub xfbin_paths: Vec<String>,

    #[serde(skip)]
    #[deku(update = "self.props.len() as u64")]
    pub props_count: u64,

    #[serde(skip)]
    pub props_ptr: u64,
    #[deku(skip)]
    pub props: Vec<Prop>,

    #[deku(count = "0x18")]
    pub unk_bytes0: Vec<i8>,

    #[deku(count = "3")]
    pub unk_vec: Vec<f32>,

    pub unk0: u32,

    #[deku(count = "5")]
    pub unk_floats0: Vec<f32>,

    #[deku(count = "4")]
    pub unk_bytes1: Vec<i8>,

    pub unk1: u32,

    #[deku(count = "0x11")]
    pub unk_floats1: Vec<f32>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct StageInfo {
    pub unk0: u32,
    pub pointer_size: u32,
    pub entries: Vec<Entry>,

    big_endian: bool,
}

impl NuccBinaryParsed for StageInfo {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::StageInfo(endian_from_bool(self.big_endian))
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

impl From<(&[u8], Endian)> for StageInfo {
    fn from(converter: (&[u8], Endian)) -> Self {
        fn read_string_pointer(reader: &mut BinaryReader, pointer: u64) -> String {
            if pointer != 0 {
                reader.seek(pointer).unwrap();
                reader.read_string_null_terminated().unwrap()
            } else {
                String::from("")
            }
        }

        let (data, endian) = converter;
        let mut stream = SliceStream::new(data);
        let mut reader = BinaryReader::new(&mut stream, super::binary_stream_endian(endian));

        let unk0 = reader.read_u32().unwrap();
        let entry_count = reader.read_u32().unwrap();
        let pointer_size = reader.read_u32().unwrap();
        reader.seek_cur(4).unwrap(); // Padding

        let mut entries = Vec::new();
        entries.reserve_exact(entry_count as usize);

        for pos in (0..entry_count).map(|i| 0x10 + (0xB0 * i) as u64) {
            reader.seek(pos).unwrap();
            let input = reader.read_bytes(0xB0).unwrap();
            let (_, mut entry) = Entry::read(input.view_bits(), endian).unwrap();

            entry.entry_name = read_string_pointer(&mut reader, pos + entry.entry_name_ptr);

            if entry.xfbin_paths_ptr != 0 {
                reader.seek(pos + 0x10 + entry.xfbin_paths_ptr).unwrap();

                let mut pos: u64;
                for _ in 0..entry.xfbin_paths_count {
                    pos = reader.tell().unwrap();

                    let ptr = reader.read_u64().unwrap();
                    entry
                        .xfbin_paths
                        .push(read_string_pointer(&mut reader, pos + ptr));

                    reader.seek(pos + 8).unwrap();
                }
            }

            if entry.props_ptr != 0 {
                reader.seek(pos + 0x20 + entry.props_ptr).unwrap();

                let mut pos: u64;
                for _ in 0..entry.props_count {
                    pos = reader.tell().unwrap();

                    let input = reader.read_bytes(0x38).unwrap();
                    let (_, mut sub_entry) = Prop::read(input.view_bits(), endian).unwrap();

                    sub_entry.xfbin_path =
                        read_string_pointer(&mut reader, pos + sub_entry.xfbin_path_ptr);
                    sub_entry.clump_name =
                        read_string_pointer(&mut reader, pos + 0x08 + sub_entry.clump_name_ptr);
                    sub_entry.string2 =
                        read_string_pointer(&mut reader, pos + 0x10 + sub_entry.string2_ptr);
                    sub_entry.string3 =
                        read_string_pointer(&mut reader, pos + 0x18 + sub_entry.string3_ptr);

                    entry.props.push(sub_entry);

                    reader.seek(pos + 0x38).unwrap();
                }
            }

            entries.push(entry);
        }

        Self {
            unk0,
            pointer_size,
            entries,

            big_endian: endian == Endian::Big,
        }
    }
}

impl From<StageInfo> for Vec<u8> {
    fn from(mut parsed: StageInfo) -> Self {
        fn write_string(writer: &mut BinaryWriter, string: &str) -> u64 {
            let pos = writer.tell().unwrap();

            writer.write_string_null_terminated(string).unwrap();
            writer.align(8).unwrap();

            pos
        }

        fn write_pointer(writer: &mut BinaryWriter, pointer: u64, offset: u64) {
            writer.seek(offset).unwrap();
            writer.write_u64(pointer).unwrap();
        }

        let endian = endian_from_bool(parsed.big_endian);

        let mut stream = MemoryStream::new();
        let mut writer = BinaryWriter::new(&mut stream, super::binary_stream_endian(endian));

        let mut prop_pointers = vec![];
        let mut prop_stream = MemoryStream::new();
        let mut prop_writer =
            BinaryWriter::new(&mut prop_stream, super::binary_stream_endian(endian));

        let mut string_pointers = vec![];
        let mut string_stream = MemoryStream::new();
        let mut string_writer =
            BinaryWriter::new(&mut string_stream, super::binary_stream_endian(endian));

        writer.write_u32(parsed.unk0).unwrap();
        writer.write_u32(parsed.entries.len() as u32).unwrap();
        writer.write_u32(parsed.pointer_size).unwrap();
        writer.write_padding(4).unwrap();

        for entry in parsed.entries.iter_mut() {
            entry.update().unwrap();

            string_pointers.push(write_string(&mut string_writer, &entry.entry_name));

            prop_pointers.push(prop_writer.tell().unwrap());
            for xfbin_path in entry.xfbin_paths.iter() {
                prop_writer.write_u64(0).unwrap();
                string_pointers.push(write_string(&mut string_writer, xfbin_path));
            }

            let mut output = BitVec::new();
            for prop in entry.props.iter() {
                string_pointers.push(write_string(&mut string_writer, &prop.xfbin_path));
                string_pointers.push(write_string(&mut string_writer, &prop.clump_name));
                string_pointers.push(write_string(&mut string_writer, &prop.string2));
                string_pointers.push(write_string(&mut string_writer, &prop.string3));

                prop.write(&mut output, endian).unwrap();
            }

            prop_pointers.push(prop_writer.tell().unwrap());
            prop_writer.write_bytes(output.into_vec()).unwrap();

            let mut output = BitVec::new();
            entry.write(&mut output, endian).unwrap();
            writer.write_bytes(output.into_vec()).unwrap();
        }

        let prop_start = writer.len().unwrap();
        let string_start = prop_start + prop_writer.len().unwrap();

        let mut prop_pointers = prop_pointers.into_iter();
        let mut string_pointers = string_pointers.into_iter();
        for (offset, entry) in parsed
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (0x10 + (0xB0 * i) as u64, e))
        {
            write_pointer(
                &mut writer,
                (string_start - offset) + string_pointers.next().unwrap(),
                offset,
            );

            let xfbin_paths_ptr = prop_pointers.next().unwrap();
            let props_ptr = prop_pointers.next().unwrap();

            write_pointer(
                &mut writer,
                (prop_start - (offset + 0x10)) + xfbin_paths_ptr,
                offset + 0x10,
            );
            write_pointer(
                &mut writer,
                (prop_start - (offset + 0x20)) + props_ptr,
                offset + 0x20,
            );

            for offset in (0..entry.xfbin_paths.len()).map(|i| xfbin_paths_ptr + (8 * i) as u64) {
                write_pointer(
                    &mut prop_writer,
                    (string_start - offset) + string_pointers.next().unwrap(),
                    offset,
                );
            }

            for offset in (0..entry.props.len()).map(|i| props_ptr + (0x38 * i) as u64) {
                write_pointer(
                    &mut prop_writer,
                    (string_start - offset) + string_pointers.next().unwrap(),
                    offset,
                );
                write_pointer(
                    &mut prop_writer,
                    (string_start - (offset + 0x08)) + string_pointers.next().unwrap(),
                    offset + 0x08,
                );
                write_pointer(
                    &mut prop_writer,
                    (string_start - (offset + 0x10)) + string_pointers.next().unwrap(),
                    offset + 0x10,
                );
                write_pointer(
                    &mut prop_writer,
                    (string_start - (offset + 0x18)) + string_pointers.next().unwrap(),
                    offset + 0x18,
                );
            }
        }

        let mut result = Vec::<u8>::from(stream);
        result.append(&mut Vec::<u8>::from(prop_stream));
        result.append(&mut Vec::<u8>::from(string_stream));

        result
    }
}
