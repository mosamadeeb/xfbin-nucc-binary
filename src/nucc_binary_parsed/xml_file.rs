use super::NuccBinaryParsed;
use super::NuccBinaryType;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct XmlFile {
    pub file: Vec<u8>,
}

impl NuccBinaryParsed for XmlFile {
    fn binary_type(&self) -> NuccBinaryType {
        NuccBinaryType::XML
    }

    fn extension(&self, _: bool) -> String {
        String::from(".xml")
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

impl From<&[u8]> for XmlFile {
    fn from(data: &[u8]) -> Self {
        Self {
            file: data.to_vec(),
        }
    }
}

impl From<XmlFile> for Vec<u8> {
    fn from(parsed: XmlFile) -> Self {
        parsed.file
    }
}
