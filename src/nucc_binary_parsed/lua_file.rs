use super::NuccBinaryParsed;
use super::NuccBinaryType;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct LuaFile {
    pub file: Vec<u8>,
}

impl NuccBinaryParsed for LuaFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::LUA
    }

    fn extension(&self, _: bool) -> String {
        String::from(".lua")
    }

    fn serialize(&self, _: bool) -> Vec<u8> {
        self.file.clone()
    }

    fn deserialize(data: &[u8], _: bool) -> Self
    where
        Self: Sized,
    {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<&[u8]> for LuaFile {
    fn from(data: &[u8]) -> Self {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<LuaFile> for Vec<u8> {
    fn from(parsed: LuaFile) -> Self {
        parsed.file
    }
}
