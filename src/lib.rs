mod dds_file;
mod ev_file;
mod lua_file;
mod message_info;
mod player_color_param;
mod png_file;
mod xml_file;

use binary_stream::Endian as BinaryEndian;
use deku::{
    bitvec::{BitVec, BitView},
    ctx::Endian,
    DekuRead, DekuUpdate, DekuWrite,
};
use downcast_rs::{impl_downcast, Downcast};
use regex::Regex;
use strum_macros::{Display, EnumIter, EnumString};

use dds_file::DdsFile;
use ev_file::EvFile;
use lua_file::LuaFile;
use message_info::MessageInfo;
use player_color_param::PlayerColorParam;
use png_file::PngFile;
use xml_file::XmlFile;

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

#[derive(EnumIter, Display, EnumString)]
pub enum NuccBinaryType {
    DDS,
    Ev(Endian),
    LUA,
    MessageInfo(Endian),
    PlayerColorParam(Endian),
    PNG,
    XML,
}

impl NuccBinaryType {
    pub fn patterns(&self) -> Vec<(Regex, Endian)> {
        match self {
            NuccBinaryType::DDS => {
                vec![(Regex::new(r"(\.dds)$").unwrap(), Endian::Little)]
            }
            NuccBinaryType::Ev(_) => {
                vec![(Regex::new(r"(_ev\.bin)$").unwrap(), Endian::Little)]
            }
            NuccBinaryType::LUA => {
                vec![(Regex::new(r"(\.lua)$").unwrap(), Endian::Little)]
            }
            NuccBinaryType::MessageInfo(_) => {
                vec![
                    (
                        Regex::new(r"((WIN(32|64)|PS4).*?/message.*?\.bin)$").unwrap(),
                        Endian::Little,
                    ),
                    // (
                    //     Regex::new(r"(PS3.*?/message.*?\.bin)$").unwrap(),
                    //     Endian::Big,
                    // ),
                ]
            }
            NuccBinaryType::PlayerColorParam(_) => {
                vec![(
                    Regex::new(r"(PlayerColorParam\.bin)$").unwrap(),
                    Endian::Little,
                )]
            }
            NuccBinaryType::PNG => {
                vec![(Regex::new(r"(\.png)$").unwrap(), Endian::Little)]
            }
            NuccBinaryType::XML => {
                vec![(Regex::new(r"(\.xml)$").unwrap(), Endian::Little)]
            }
        }
    }

    pub fn examples(&self) -> Vec<String> {
        match self {
            NuccBinaryType::DDS => {
                vec![String::from("Z:/STORM4_UI_DATA/charsel/charsel_I3.dds")]
            }
            NuccBinaryType::Ev(_) => {
                vec![String::from("player/1dio01_ev/1dio01_ev.bin")]
            }
            NuccBinaryType::LUA => {
                vec![String::from("d01/d01_010.lua")]
            }
            NuccBinaryType::MessageInfo(_) => {
                vec![
                    String::from("WIN64/eng/message_DLC110.bin"),
                    // String::from("PS3//eng//messageInfo.bin"),
                ]
            }
            NuccBinaryType::PlayerColorParam(_) => {
                vec![String::from("PlayerColorParam.bin")]
            }
            NuccBinaryType::PNG => {
                vec![String::from("Z:/char/x/duel_item/tex/c_bat_067.png")]
            }
            NuccBinaryType::XML => {
                vec![String::from("D:/JARP/trunk/param/spm/spm/0bao01_SPM.xml")]
            }
        }
    }

    pub fn convert(&self, data: &[u8], endian: Endian) -> Box<dyn NuccBinaryParsed> {
        match self {
            NuccBinaryType::DDS => Box::new(DdsFile::from(data)),
            NuccBinaryType::Ev(_) => Box::new(EvFile::read(data.view_bits(), endian).unwrap().1),
            NuccBinaryType::LUA => Box::new(LuaFile::from(data)),
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::from((data, endian))),
            NuccBinaryType::PlayerColorParam(_) => Box::new(PlayerColorParam::from((data, endian))),
            NuccBinaryType::PNG => Box::new(PngFile::from(data)),
            NuccBinaryType::XML => Box::new(XmlFile::from(data)),
        }
    }
}

fn binary_stream_endian(endian: Endian) -> BinaryEndian {
    match endian {
        Endian::Little => BinaryEndian::Little,
        Endian::Big => BinaryEndian::Big,
    }
}
