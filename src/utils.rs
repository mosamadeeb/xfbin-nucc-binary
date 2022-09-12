use deku::ctx::{Endian, Limit};
use deku::prelude::*;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Default)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(ctx = "_: Endian, size: usize")]
pub struct DekuFixedString {
    #[deku(
        reader = "Vec::<u8>::read(deku::rest, Limit::from(size)).map(|(r, s)| (r, String::from_utf8(s).unwrap().trim_end_matches(\'\0\').to_string()))",
        writer = "(string.clone() + &String::from(\"\0\").repeat(size - string.len())).as_bytes().write(deku::output, ())"
    )]
    pub string: String,
}

impl Serialize for DekuFixedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.string)
    }
}

impl<'de> Deserialize<'de> for DekuFixedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(DekuFixedStringVisitor)
    }
}

struct DekuFixedStringVisitor;

impl<'de> Visitor<'de> for DekuFixedStringVisitor {
    type Value = DekuFixedString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<DekuFixedString, E>
    where
        E: de::Error,
    {
        Ok(DekuFixedString {
            string: v.to_string(),
        })
    }

    fn visit_string<E>(self, v: String) -> Result<DekuFixedString, E>
    where
        E: de::Error,
    {
        Ok(DekuFixedString { string: v })
    }
}
