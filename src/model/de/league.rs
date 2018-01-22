//! Deserializer for the league name.

use std::fmt;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::League;


const EXPECTING_MSG: &str = "league name";

impl<'de> Deserialize<'de> for League {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(LeagueVisitor)
    }
}

struct LeagueVisitor;
impl<'de> Visitor<'de> for LeagueVisitor {
    type Value = League;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing FromStr implementation for League

        // Since we have to accept basically anything as a temporary league name,
        // let's do very basic sanity checks to see if it's at least well-formed.
        if v.is_empty() {
            return Err(de::Error::invalid_length(0, &"non-empty string"));
        }
        let has_capitalized_words = v.split_whitespace().all(|w| {
            let first_upper = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            let rest_lower = w.chars().skip(1).all(|c| c.is_lowercase());
            w.len() > 1 && first_upper && rest_lower
        });
        if !has_capitalized_words {
            return Err(de::Error::invalid_value(
                Unexpected::Str(v), &"string of capitalized words"));
        }

        match v {
            "Standard" => Ok(League::standard()),
            "Hardcore" => Ok(League::hardcore()),
            v => Ok(
                if v.contains("Hardcore") {
                    League::temporary_hardcore()
                } else {
                    League::temporary()
                }
            )
            // SSF leagues are not found in the API responses for obvious reasons
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use model::League;

    #[test]
    fn permanent_leagues() {
        assert_eq!(League::standard(), from_value(json!("Standard")).unwrap());
        assert_eq!(League::hardcore(), from_value(json!("Hardcore")).unwrap());
    }

    #[test]
    fn abyss_leagues() {
        assert_eq!(League::temporary(), from_value(json!("Abyss")).unwrap());
        assert_eq!(League::temp_hc(), from_value(json!("Hardcore Abyss")).unwrap());
    }

    #[test]
    fn harbinger_leagues() {
        assert_eq!(League::temporary(), from_value(json!("Harbinger")).unwrap());
        assert_eq!(League::temp_hc(), from_value(json!("Hardcore Harbinger")).unwrap());
    }

    // TODO: all the other past leagues
}
