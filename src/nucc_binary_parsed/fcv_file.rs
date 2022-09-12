use super::NuccBinaryParsed;
use super::NuccBinaryType;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct FcvFile {
    pub file: Vec<u8>,
}

impl NuccBinaryParsed for FcvFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::FCV
    }

    fn extension(&self, _: bool) -> String {
        String::from(".fcv")
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

impl From<&[u8]> for FcvFile {
    fn from(data: &[u8]) -> Self {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<FcvFile> for Vec<u8> {
    fn from(parsed: FcvFile) -> Self {
        parsed.file
    }
}
