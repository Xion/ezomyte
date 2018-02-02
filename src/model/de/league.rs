//! Deserializer for the league name.

use std::fmt;
use std::str::FromStr;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::{League, ParseLeagueError};


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
        League::from_str(v).map_err(|e| match e {
            ParseLeagueError::Empty => de::Error::invalid_length(0, &"non-empty string"),
            ParseLeagueError::Malformed => de::Error::invalid_value(
                Unexpected::Str(v), &"string of capitalized words or acronyms"),
        })
    }
}
