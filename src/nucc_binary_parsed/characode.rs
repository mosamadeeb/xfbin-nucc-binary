use super::endian_from_bool;
use super::NuccBinaryParsed;
use super::NuccBinaryType;
use crate::utils::DekuFixedString;

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
    pub index: u32,

    #[deku(ctx = "8")]
    pub chara: DekuFixedString,
}

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "Endian::Little"
)]
pub struct CharaCode {
    #[serde(skip)]
    #[deku(update = "self.entries.len() as u32")]
    pub count: u32,

    #[deku(count = "count")]
    pub entries: Vec<Entry>,

    #[deku(skip, default = "endian == Endian::Big")]
    pub big_endian: bool,
}

impl NuccBinaryParsed for CharaCode {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::CharaCode(endian_from_bool(self.big_endian))
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
