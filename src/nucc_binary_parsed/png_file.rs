use super::NuccBinaryParsed;
use super::NuccBinaryType;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct PngFile {
    pub file: Vec<u8>,
}

impl NuccBinaryParsed for PngFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::PNG
    }

    fn extension(&self, _: bool) -> String {
        String::from(".png")
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

impl From<&[u8]> for PngFile {
    fn from(data: &[u8]) -> Self {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<PngFile> for Vec<u8> {
    fn from(parsed: PngFile) -> Self {
        parsed.file
    }
}
