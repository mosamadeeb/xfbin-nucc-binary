mod characode;
mod dds_file;
mod ev_file;
mod lua_file;
mod message_info;
mod player_color_param;
mod png_file;
mod sound_test_param;
mod xml_file;

use binary_stream::Endian as BinaryEndian;
use crc::{Crc, CRC_32_BZIP2};
use deku::{
    bitvec::{BitVec, BitView},
    ctx::Endian,
    DekuRead, DekuUpdate, DekuWrite,
};
use downcast_rs::{impl_downcast, Downcast};
use strum::IntoEnumIterator;

use super::NuccBinaryType;

pub use characode::CharaCode;
pub use dds_file::DdsFile;
pub use ev_file::{EvFile, Version as EvVersion};
pub use lua_file::LuaFile;
pub use message_info::MessageInfo;
pub use player_color_param::PlayerColorParam;
pub use png_file::PngFile;
pub use sound_test_param::SoundTestParam;
pub use xml_file::XmlFile;

pub trait NuccBinaryParsed: Downcast {
    fn binary_type(&self) -> NuccBinaryType;
    fn extension(&self, use_json: bool) -> String;
    fn serialize(&self, use_json: bool) -> Vec<u8>;
    fn deserialize(data: &[u8], use_json: bool) -> Self
    where
        Self: Sized;
}

impl_downcast!(NuccBinaryParsed);

const UNEXPECTED_ENUM: &str = "Version index out of bounds";

pub struct NuccBinaryParsedReader<'a>(pub NuccBinaryType, pub &'a [u8], pub Endian, pub usize);

impl From<NuccBinaryParsedReader<'_>> for Box<dyn NuccBinaryParsed> {
    fn from(reader: NuccBinaryParsedReader<'_>) -> Self {
        let NuccBinaryParsedReader(binary_type, data, endian, version) = reader;

        match binary_type {
            NuccBinaryType::CharaCode(_) => {
                Box::new(CharaCode::read(data.view_bits(), endian).unwrap().1)
            }
            NuccBinaryType::DDS => Box::new(DdsFile::from(data)),
            NuccBinaryType::Ev(_) => Box::new(
                EvFile::read(
                    data.view_bits(),
                    (
                        endian,
                        EvVersion::iter().nth(version).expect(UNEXPECTED_ENUM),
                    ),
                )
                .unwrap()
                .1,
            ),
            NuccBinaryType::LUA => Box::new(LuaFile::from(data)),
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::from((data, endian))),
            NuccBinaryType::PlayerColorParam(_) => Box::new(PlayerColorParam::from((data, endian))),
            NuccBinaryType::PNG => Box::new(PngFile::from(data)),
            NuccBinaryType::SoundTestParam(_) => Box::new(SoundTestParam::from((data, endian))),
            NuccBinaryType::XML => Box::new(XmlFile::from(data)),
        }
    }
}

pub struct NuccBinaryParsedWriter(pub Box<dyn NuccBinaryParsed>, pub usize);

impl From<NuccBinaryParsedWriter> for Vec<u8> {
    fn from(writer: NuccBinaryParsedWriter) -> Self {
        let NuccBinaryParsedWriter(boxed, _version) = writer;

        match boxed.binary_type() {
            NuccBinaryType::CharaCode(_) => {
                let mut chara = *boxed.downcast::<CharaCode>().ok().unwrap();
                chara.update().unwrap();

                let mut output = BitVec::new();
                chara
                    .write(&mut output, endian_from_bool(chara.big_endian))
                    .unwrap();
                output.into_vec()
            }
            NuccBinaryType::DDS => (*boxed.downcast::<DdsFile>().ok().unwrap()).into(),
            NuccBinaryType::Ev(_) => {
                let mut ev = *boxed.downcast::<EvFile>().ok().unwrap();
                ev.update().unwrap();

                let mut output = BitVec::new();
                ev.write(
                    &mut output,
                    (endian_from_bool(ev.big_endian), ev.stored_version),
                )
                .unwrap();
                output.into_vec()
            }
            NuccBinaryType::LUA => (*boxed.downcast::<LuaFile>().ok().unwrap()).into(),
            NuccBinaryType::MessageInfo(_) => {
                (*boxed.downcast::<MessageInfo>().ok().unwrap()).into()
            }
            NuccBinaryType::PlayerColorParam(_) => {
                (*boxed.downcast::<PlayerColorParam>().ok().unwrap()).into()
            }
            NuccBinaryType::PNG => (*boxed.downcast::<PngFile>().ok().unwrap()).into(),
            NuccBinaryType::SoundTestParam(_) => {
                (*boxed.downcast::<SoundTestParam>().ok().unwrap()).into()
            }
            NuccBinaryType::XML => (*boxed.downcast::<XmlFile>().ok().unwrap()).into(),
        }
    }
}

pub struct NuccBinaryParsedDeserializer(pub NuccBinaryType, pub bool, pub Vec<u8>);

impl From<NuccBinaryParsedDeserializer> for Box<dyn NuccBinaryParsed> {
    fn from(deserializer: NuccBinaryParsedDeserializer) -> Self {
        let NuccBinaryParsedDeserializer(binary_type, use_json, data) = deserializer;

        match binary_type {
            NuccBinaryType::CharaCode(_) => Box::new(CharaCode::deserialize(&data, use_json)),
            NuccBinaryType::DDS => Box::new(DdsFile::deserialize(&data, use_json)),
            NuccBinaryType::Ev(_) => Box::new(EvFile::deserialize(&data, use_json)),
            NuccBinaryType::LUA => Box::new(LuaFile::deserialize(&data, use_json)),
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::deserialize(&data, use_json)),
            NuccBinaryType::PlayerColorParam(_) => {
                Box::new(PlayerColorParam::deserialize(&data, use_json))
            }
            NuccBinaryType::PNG => Box::new(PngFile::deserialize(&data, use_json)),
            NuccBinaryType::SoundTestParam(_) => {
                Box::new(SoundTestParam::deserialize(&data, use_json))
            }
            NuccBinaryType::XML => Box::new(XmlFile::deserialize(&data, use_json)),
        }
    }
}

pub struct NuccBinaryParsedSerializer(pub Box<dyn NuccBinaryParsed>, pub bool);

impl From<NuccBinaryParsedSerializer> for Vec<u8> {
    fn from(serializer: NuccBinaryParsedSerializer) -> Self {
        let NuccBinaryParsedSerializer(boxed, use_json) = serializer;
        boxed.serialize(use_json)
    }
}

const MSG_ID_HASH: Crc<u32> = Crc::<u32>::new(&CRC_32_BZIP2);

fn calc_crc32(data: &[u8]) -> Vec<u8> {
    let mut output = BitVec::new();
    u32::write(&MSG_ID_HASH.checksum(data), &mut output, Endian::Little).unwrap();
    output.into()
}

fn binary_stream_endian(endian: Endian) -> BinaryEndian {
    match endian {
        Endian::Little => BinaryEndian::Little,
        Endian::Big => BinaryEndian::Big,
    }
}

fn endian_from_bool(is_big_endian: bool) -> Endian {
    if is_big_endian {
        Endian::Big
    } else {
        Endian::Little
    }
}
