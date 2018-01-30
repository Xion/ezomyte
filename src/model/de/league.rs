//! Deserializer for the league name.

use std::fmt;

use itertools::Itertools;
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

        const COMMON_WORDS: &[&str] = &["Standard", "Hardcore", "HC", "SSF"];

        // Do some basic sanity checks on the league name to see if it's well-formed.
        if v.is_empty() {
            return Err(de::Error::invalid_length(0, &"non-empty string"));
        }
        let has_valid_words = v.split_whitespace().all(|w| {
            COMMON_WORDS.contains(&w) || {
                // Other words are season names which must be capitalized.
                let first_upper = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                let rest_lower = w.chars().skip(1).all(|c| c.is_lowercase());
                w.len() > 1 && first_upper && rest_lower
            }
        });
        if !has_valid_words {
            return Err(de::Error::invalid_value(
                Unexpected::Str(v), &"string of capitalized words or acronyms"));
        }

        // Extract league's attributes (SC/HC, SSF?) and its season name.
        let mut league = match v {
            "Standard" => League::standard(),
            "Hardcore" => League::hardcore(),
            "SSF Standard" => League::ssf(),
            "SSF Hardcore" => League::hardcore_ssf(),
            v => {
                let hardcore = v.contains("Hardcore") || v.contains("HC");
                let ssf = v.contains("SSF");
                match (hardcore, ssf) {
                    (false, false) => League::temporary(),
                    (false, true) => League::temporary_ssf(),
                    (true, false) => League::temporary_hardcore(),
                    (true, true) => League::temporary_hardcore_ssf(),
                }
            }
        };
        league.season = {
            let season = v.split_whitespace().filter(|w| !COMMON_WORDS.contains(&w)).join("");
            if season.is_empty() { None } else { Some(season) }
        };
        Ok(league)
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
        assert_eq!(League::ssf(), from_value(json!("SSF Standard")).unwrap());
        assert_eq!(League::hardcore_ssf(), from_value(json!("SSF Hardcore")).unwrap());
    }

    #[test]
    fn abyss_leagues() {
        assert_eq!(League::temporary(), from_value(json!("Abyss")).unwrap());
        assert_eq!(League::temp_hc(), from_value(json!("Hardcore Abyss")).unwrap());
        assert_eq!(League::temp_ssf(), from_value(json!("SSF Abyss")).unwrap());
        assert_eq!(League::temp_hc_ssf(), from_value(json!("SSF Abyss HC")).unwrap());
    }

    #[test]
    fn harbinger_leagues() {
        assert_eq!(League::temporary(), from_value(json!("Harbinger")).unwrap());
        assert_eq!(League::temp_hc(), from_value(json!("Hardcore Harbinger")).unwrap());
        assert_eq!(League::temp_ssf(), from_value(json!("SSF Harbinger")).unwrap());
        assert_eq!(League::temp_hc_ssf(), from_value(json!("SSF Harbinger HC")).unwrap());
    }

    // TODO: all the other past leagues
}
