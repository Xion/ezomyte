//! Identifier of an item mod in the mod database.
//!
//! Mod identifiers in their raw form are textual idents such as "crafted.stat_3299347043".
//! Types define here introduce some more compile-time safety to those values.

use std::error;
use std::fmt;
use std::str::FromStr;

use conv::errors::Unrepresentable;
use regex::Regex;
use serde::de::{self, Deserialize, Deserializer, Unexpected};


/// Mod identifier, including it's type (crafted, implicit, etc.) and a unique number.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ModId {
    type_: ModType,
    number: u64,
}

impl ModId {
    /// Create mod identifier of given type and number.
    #[inline]
    pub fn new(type_: ModType, number: u64) -> Self {
        ModId { type_, number }
    }
}

impl FromStr for ModId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref MOD_ID_RE: Regex = Regex::new(
                r#"(?P<type>\w+)\.stat_(?P<number>\d+)"#
            ).unwrap();
        }
        let caps = MOD_ID_RE.captures(s).ok_or_else(|| Error::Malformed(s.to_owned()))?;
        let type_ = ModType::from_str(caps.name("type").unwrap().as_str())
            .map_err(|Unrepresentable(t)| Error::UnknownModType(t))?;
        let number = {
            let n = caps.name("number").unwrap().as_str();
            n.parse().map_err(|_| Error::InvalidModNumber(n.to_owned()))?
        };
        Ok(ModId::new(type_, number))
    }
}
impl<'de> Deserialize<'de> for ModId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        ModId::from_str(&s).map_err(|e| match e {
            Error::Malformed(s) => de::Error::invalid_value(
                Unexpected::Str(s.as_str()), &"mod ID as $TYPE.stat_$NUMBER"),
            Error::UnknownModType(ref t) => de::Error::unknown_variant(t.as_str(), MOD_TYPES),
            Error::InvalidModNumber(ref n) => de::Error::invalid_value(
                Unexpected::Str(n.as_str()), &"mod number (as unsigned integer)"),
        })
    }
}

impl ModId {
    /// Type of the mod (crafted, explicit, etc.).
    #[inline]
    pub fn mod_type(&self) -> ModType {
        self.type_
    }

    /// Mod's identifying number (unique within mods of the same type).
    pub fn mod_number(&self) -> u64 {
        self.number
    }
}

impl fmt::Debug for ModId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "ModId::new({:?}, {:?})", self.type_, self.number)
    }
}


/// Error when converting from string to `ModId`.
#[derive(Debug)]
pub enum Error {
    /// Error for when the mod ID is completely malformed.
    Malformed(String),
    /// Error for when the mod type cannot be recognized.
    UnknownModType(String),
    /// Error for when the mod number cannot be parsed.
    InvalidModNumber(String),
}
impl error::Error for Error {
    fn description(&self) -> &str { "error parsing mod ID" }
    fn cause(&self) -> Option<&error::Error> { None }
}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Malformed(ref s) => write!(fmt, "malformed mod ID: {}", s),
            Error::UnknownModType(ref t) => write!(fmt, "unknown mod type `{}`", t),
            Error::InvalidModNumber(ref n) => write!(fmt, "invalid mod number `{}`", n),
        }
    }
}


macro_attr! {
    /// Type of an item mod.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash,
             IterVariants!(ModTypes))]
    pub enum ModType {
        /// Mods crafted by Forsaken Master benches.
        Crafted,
        /// Mods that can be enchanted on items in the Lord's Labirynth.
        Enchant,
        /// Explicit ("regular") item mods
        Explicit,
        /// Implicit item mods (inherent to the item base).
        Implicit,
    }
}

impl FromStr for ModType {
    type Err = Unrepresentable<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "crafted" => Ok(ModType::Crafted),
            "enchant" => Ok(ModType::Enchant),
            "explicit" => Ok(ModType::Explicit),
            "implicit" => Ok(ModType::Implicit),
            _ => Err(Unrepresentable(s.to_owned())),
        }
    }
}

const MOD_TYPES: &[&str] = &["crafted", "enchant", "explicit", "implicit"];


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::{ModType, MOD_TYPES};

    #[test]
    fn mod_type_strings() {
        for &type_ in MOD_TYPES.iter() {
            assert!(ModType::from_str(type_).is_ok());
        }
    }
}
