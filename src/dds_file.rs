use super::NuccBinaryParsed;
use super::NuccBinaryType;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct DdsFile {
    pub file: Vec<u8>,
}

impl NuccBinaryParsed for DdsFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::DDS
    }

    fn extension(&self, _: bool) -> String {
        String::from(".dds")
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

impl From<&[u8]> for DdsFile {
    fn from(data: &[u8]) -> Self {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<DdsFile> for Vec<u8> {
    fn from(parsed: DdsFile) -> Self {
        parsed.file
    }
}
