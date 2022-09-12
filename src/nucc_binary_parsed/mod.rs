mod dds_file;
mod ev_file;
mod lua_file;
mod message_info;
mod player_color_param;
mod png_file;
mod xml_file;

use binary_stream::Endian as BinaryEndian;
use deku::{bitvec::BitVec, ctx::Endian, DekuUpdate, DekuWrite};
use downcast_rs::{impl_downcast, Downcast};

use super::NuccBinaryType;

pub use dds_file::DdsFile;
pub use ev_file::EvFile;
pub use lua_file::LuaFile;
pub use message_info::MessageInfo;
pub use player_color_param::PlayerColorParam;
pub use png_file::PngFile;
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

impl From<Box<dyn NuccBinaryParsed>> for Vec<u8> {
    fn from(boxed: Box<dyn NuccBinaryParsed>) -> Self {
        match boxed.binary_type() {
            NuccBinaryType::DDS => (*boxed.downcast::<DdsFile>().ok().unwrap()).into(),
            NuccBinaryType::Ev(_) => {
                let mut ev = *boxed.downcast::<EvFile>().ok().unwrap();
                ev.update().unwrap();

                let mut output = BitVec::new();
                ev.write(
                    &mut output,
                    if ev.big_endian {
                        Endian::Big
                    } else {
                        Endian::Little
                    },
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
            NuccBinaryType::XML => (*boxed.downcast::<XmlFile>().ok().unwrap()).into(),
        }
    }
}

pub struct NuccBinaryParsedConverter(pub NuccBinaryType, pub bool, pub Vec<u8>);

impl From<NuccBinaryParsedConverter> for Box<dyn NuccBinaryParsed> {
    fn from(converter: NuccBinaryParsedConverter) -> Self {
        let NuccBinaryParsedConverter(binary_type, use_json, data) = converter;

        match binary_type {
            NuccBinaryType::DDS => Box::new(DdsFile::deserialize(&data, use_json)),
            NuccBinaryType::Ev(_) => Box::new(EvFile::deserialize(&data, use_json)),
            NuccBinaryType::LUA => Box::new(LuaFile::deserialize(&data, use_json)),
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::deserialize(&data, use_json)),
            NuccBinaryType::PlayerColorParam(_) => {
                Box::new(PlayerColorParam::deserialize(&data, use_json))
            }
            NuccBinaryType::PNG => Box::new(PngFile::deserialize(&data, use_json)),
            NuccBinaryType::XML => Box::new(XmlFile::deserialize(&data, use_json)),
        }
    }
}

fn binary_stream_endian(endian: Endian) -> BinaryEndian {
    match endian {
        Endian::Little => BinaryEndian::Little,
        Endian::Big => BinaryEndian::Big,
    }
}
