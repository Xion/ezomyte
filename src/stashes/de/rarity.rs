//! Deserializer for item rarity.

use std::fmt;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::Rarity;


const EXPECTING_MSG: &str = "rarity index (from 0=normal to 3=unique)";


impl<'de> Deserialize<'de> for Rarity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        // Rarity is encoded in the API as the "frameType" field
        // which is a number.
        deserializer.deserialize_u64(RarityVisitor)
    }
}

struct RarityVisitor;
impl<'de> Visitor<'de> for RarityVisitor {
    type Value = Rarity;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Rarity::iter_variants().skip(v as usize).next().ok_or_else(|| {
            de::Error::invalid_value(Unexpected::Unsigned(v), &EXPECTING_MSG)
        })
    }
}
