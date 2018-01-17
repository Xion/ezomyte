//! Deserializers for item attributes.

use serde::de::{self, Deserilize, Visitor, Unexpected};
use regex::Regex;

use super::super::{Quality, Rarity};


// Rarity

const EXPECTING_RARITY_MSG: &str = "rarity index (from 0=normal to 3=unique)";

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
        write!(fmt, "{}", EXPECTING_RARITY_MSG)
    }

    fn visit_u64<E: de::Error>(&self, v: u64) -> Result<Self::Value, E> {
        Rarity::iter_variants().skip(v as usize).next().ok_or_else(|| {
            de::Error::invalid_value(Unexpected::Unsigned(v), &EXPECTING_RARITY_MSG)
        })
    }
}


// Quality

const EXPECTING_QUALITY_MSG: &str = "item quality value (as `+{value}%`)";

impl<'de> Deserialize<'de> for Quality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(QualityVisitor)
    }
}

struct QualityVisitor;
impl<'de> Visitor<'de> for QualityVisitor {
    type Value = Quality;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_QUALITY_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, D::Error> {
        // TODO: consider providing FromStr implementation for Quality
        lazy_static! {
            static ref QUALITY_RE: Regex = Regex::new(r#"+(\d+)%"#).unwrap();
        }
        let caps = QUALITY_RE.captures(v).ok_or_else(|| de::Error::invalid_value(
            Unexpected::Str(v), &EXPECTING_QUALITY_MSG))?;
        let percentage: u8 = caps[0].parse();
        Ok(Quality::from(percentage))
    }
}
