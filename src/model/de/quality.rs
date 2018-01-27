//! Deserializer for item quality.

use std::fmt;

use regex::Regex;
use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::Quality;


const EXPECTING_MSG: &str = "item quality value (as `+{value}%`)";


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
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing FromStr implementation for Quality
        lazy_static! {
            static ref QUALITY_RE: Regex = Regex::new(r#"[+](\d+)%"#).unwrap();
        }
        let caps = QUALITY_RE.captures(v).ok_or_else(|| de::Error::invalid_value(
            Unexpected::Str(v), &EXPECTING_MSG))?;
        let percentage: u8 = caps[1].parse()
            .map_err(|e| de::Error::custom(format!("invalid percentage number: {}", e)))?;
        Ok(Quality::from(percentage))
    }
}
