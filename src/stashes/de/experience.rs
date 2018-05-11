//! Deserializer for gem experience.

use std::fmt;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::Experience;


const EXPECTING_MSG: &str = "gem experience (as `{current}/{total}`)";


impl<'de> Deserialize<'de> for Experience {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(ExperienceVisitor)
    }
}

struct ExperienceVisitor;
impl<'de> Visitor<'de> for ExperienceVisitor {
    type Value = Experience;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing FromStr implementation for Quality
        let parts: Vec<_> = v.split("/").collect();
        if parts.len() != 2 {
            return Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG));
        }
        let current = parts[0].parse()
            .map_err(|e| de::Error::custom(format!("invalid current experience value: {}", e)))?;
        let total = parts[1].parse()
            .map_err(|e| de::Error::custom(format!("invalid total experience value: {}", e)))?;
        // TODO: check total > 0 and current <= total
        Ok(Experience::new(current, total))
    }
}
