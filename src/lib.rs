use deku::ctx::Endian;
use downcast_rs::{impl_downcast, Downcast};
use regex::Regex;
use strum_macros::{Display, EnumIter, EnumString};

use binary_stream::Endian as BinaryEndian;
use message_info::MessageInfo;

pub mod message_info;

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
            NuccBinaryType::MessageInfo(_) => {
                (*boxed.downcast::<MessageInfo>().ok().unwrap()).into()
            }
        }
    }
}

pub struct NuccBinaryParsedConverter(pub NuccBinaryType, pub bool, pub Vec<u8>);

impl From<NuccBinaryParsedConverter> for Box<dyn NuccBinaryParsed> {
    fn from(converter: NuccBinaryParsedConverter) -> Self {
        let NuccBinaryParsedConverter(binary_type, use_json, data) = converter;

        match binary_type {
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::deserialize(&data, use_json)),
        }
    }
}

#[derive(EnumIter, Display, EnumString)]
pub enum NuccBinaryType {
    MessageInfo(Endian),
}

impl NuccBinaryType {
    pub fn patterns(&self) -> Vec<(Regex, Endian)> {
        match self {
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
        }
    }

    pub fn examples(&self) -> Vec<String> {
        match self {
            NuccBinaryType::MessageInfo(_) => {
                vec![
                    String::from("WIN64/eng/message_DLC110.bin"),
                    // String::from("PS3//eng//messageInfo.bin"),
                ]
            }
        }
    }

    pub fn convert(&self, data: &[u8], endian: Endian) -> Box<dyn NuccBinaryParsed> {
        match self {
            NuccBinaryType::MessageInfo(_) => Box::new(MessageInfo::from((data, endian))),
        }
    }
}

fn binary_stream_endian(endian: Endian) -> BinaryEndian {
    match endian {
        Endian::Little => BinaryEndian::Little,
        Endian::Big => BinaryEndian::Big,
    }
}
