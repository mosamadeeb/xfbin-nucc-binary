use super::endian_from_bool;
use super::NuccBinaryParsed;
use super::NuccBinaryType;

use deku::bitvec::BitVec;
use deku::bitvec::BitView;
use deku::ctx::Endian;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::DekuFixedString;

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "Endian::Little"
)]
pub struct Entry {
    #[deku(ctx = "0x20")]
    pub folder_name: DekuFixedString,

    #[deku(ctx = "0x20")]
    pub file_name: DekuFixedString,

    pub file_type: u32,
    pub unk1: u32,
}

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "Endian::Little"
)]
pub struct PrmLoad {
    #[deku(update = "self.entries.len() as u32")]
    #[serde(skip)]
    pub entry_count: u32,

    #[deku(count = "entry_count")]
    pub entries: Vec<Entry>,

    #[deku(skip)]
    big_endian: bool,
}

impl NuccBinaryParsed for PrmLoad {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::PrmLoad(endian_from_bool(self.big_endian))
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

impl PrmLoad {
    pub fn read_parsed(data: &[u8], endian: Endian) -> Self {
        Self::read(data.view_bits(), endian).unwrap().1
    }

    pub fn write_parsed(&mut self) -> Vec<u8> {
        self.update().unwrap();

        let mut output = BitVec::new();
        self.write(&mut output, endian_from_bool(self.big_endian))
            .unwrap();
        output.into_vec()
    }
}
