//! Deserializer for the `Requirement` enum.

use std::fmt;

use serde::de::{self, Deserialize, Visitor};

use super::super::Requirement;


const EXPECTING_MSG: &str = "requirement name";
const VARIANTS: &[&str] = &["Level", "Str", "Dex", "Int",
                            "Strength", "Dexterity", "Intelligence"];


impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(RequirementVisitor)
    }
}

struct RequirementVisitor;
impl<'de> Visitor<'de> for RequirementVisitor {
    type Value = Requirement;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match v.trim() {
            "Level" => Ok(Requirement::Level),
            "Str" | "Strength" => Ok(Requirement::Strength),
            "Dex" | "Dexterity" => Ok(Requirement::Dexterity),
            "Int" | "Intelligence" => Ok(Requirement::Intelligence),
            _ => Err(de::Error::unknown_variant(v, VARIANTS)),
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use model::Requirement;
    use super::VARIANTS;

    #[test]
    fn known_variants() {
        for variant in VARIANTS {
            from_value::<Requirement>(json!(variant)).unwrap();
        }
    }
}
