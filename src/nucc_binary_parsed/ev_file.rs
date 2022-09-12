use super::NuccBinaryParsed;
use super::NuccBinaryType;

use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::{Endian, Limit};
use deku::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumMessage;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Clone, Copy, EnumIter, Display, EnumString, EnumMessage, Serialize, Deserialize)]
pub enum Version {
    /// JoJo
    Encrypted,
    /// Storm
    Unencrypted,
}

impl Default for Version {
    fn default() -> Self {
        Version::Encrypted
    }
}

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, version: Version",
    ctx_default = "Endian::Little, Version::default()"
)]
pub struct Entry {
    #[deku(
        reader = "Entry::decrypt(deku::rest, version)",
        writer = "Entry::encrypt(deku::output, version, &self.sound_name)"
    )]
    pub sound_name: String,

    pub unk0: i16,
    pub volume: f32,

    #[deku(count = "3")]
    pub unk2: Vec<i16>,

    pub timing: i16,

    pub unk3: f32,
    pub unk4: f32,

    #[deku(
        reader = "Entry::decrypt(deku::rest, version)",
        writer = "Entry::encrypt(deku::output, version, &self.xfbin_path)"
    )]
    pub xfbin_path: String,
    #[deku(
        reader = "Entry::decrypt(deku::rest, version)",
        writer = "Entry::encrypt(deku::output, version, &self.anm_name)"
    )]
    pub anm_name: String,
    #[deku(
        reader = "Entry::decrypt(deku::rest, version)",
        writer = "Entry::encrypt(deku::output, version, &self.target_bone)"
    )]
    pub target_bone: String,

    pub x_position: i32,
    pub y_position: i32,
    pub z_position: i32,

    pub unk5: i32,

    pub unk_int16: i16,
    pub loop_int16: i16,

    #[deku(
        reader = "Entry::decrypt(deku::rest, version)",
        writer = "Entry::encrypt(deku::output, version, &self.anm_command)"
    )]
    pub anm_command: String,
}

impl Entry {
    fn xor(data: &[u8], decrypt: bool) -> Vec<u8> {
        let mut key = b"\x8C\x91\x9B\x9A\x89\xD1\x87\x99\x9D\x96\x91"
            .iter()
            .cycle();

        let mut block = vec![];
        let mut result = vec![];

        if decrypt {
            for byte in data {
                block.push(byte ^ key.next().unwrap());

                if block.len() == 4 {
                    result.extend(block.into_iter().rev());
                    block = vec![];
                }
            }
        } else {
            for byte in data {
                block.push(*byte);

                if block.len() == 4 {
                    result.extend(block.into_iter().rev().map(|b| b ^ key.next().unwrap()));
                    block = vec![];
                }
            }
        }

        result
    }

    fn decrypt(
        input: &BitSlice<Msb0, u8>,
        version: Version,
    ) -> Result<(&BitSlice<Msb0, u8>, String), DekuError> {
        let (rest, data) = Vec::<u8>::read(input, Limit::from(0x20)).unwrap();

        let decrypted = match version {
            Version::Encrypted => Entry::xor(&data, true),
            Version::Unencrypted => data,
        };
        let string =
            String::from_utf8(decrypted.into_iter().take_while(|b| *b != 0).collect()).unwrap();

        Ok((rest, string))
    }

    fn encrypt(
        output: &mut BitVec<Msb0, u8>,
        version: Version,
        string: &str,
    ) -> Result<(), DekuError> {
        let string = string.to_string() + &String::from("\0").repeat(0x20 - string.len());

        let encrypted = match version {
            Version::Encrypted => Entry::xor(string.as_bytes(), false),
            Version::Unencrypted => string.as_bytes().to_vec(),
        };
        encrypted.write(output, ())
    }
}

#[derive(Default, Serialize, Deserialize)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, version: Version",
    ctx_default = "Endian::Little, Version::default()"
)]
pub struct EvFile {
    #[serde(skip)]
    #[deku(update = "self.entries.len() as u16")]
    pub count: u16,

    #[deku(count = "count", ctx = "version")]
    pub entries: Vec<Entry>,

    #[deku(skip, default = "endian == Endian::Big")]
    pub big_endian: bool,

    #[deku(skip, default = "version")]
    pub stored_version: Version,
}

impl NuccBinaryParsed for EvFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::Ev(if self.big_endian {
            Endian::Big
        } else {
            Endian::Little
        })
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
