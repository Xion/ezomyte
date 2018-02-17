//! Module definining the database of known mods.
//!
//! Note that the actual data here is filled in using a build script.

use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::str::FromStr;

use regex::{self, Regex};

use super::ModValue;
use super::id::ModId;
use util::parse_number;


/// Information about a single known item mod.
#[derive(Clone, Debug)]
pub struct ModInfo {
    /// Identifier of the mod.
    ///
    /// This is parsed from a string such as "implicit.stat_587431675".
    id: ModId,
    /// Regular expression matching the textual form of the mod
    /// as it appears on items.
    ///
    /// Within the regex, indexed (unnamed) capture groups are expected to be filled
    /// by specific mod values, e.g. "(\d+)% increased Global Critical Strike Chance".
    regex: Regex,
}

impl ModInfo {
    /// Create `ModInfo` from given mod ID and its text (as obtained from the PoE API).
    fn from_raw(id: &str, text: &str) -> Result<Self, Box<Error>> {
        const VALUE_RE: &str = r#"(\d+(?:\.\d+))?"#;  // Regex for integer or float values.
        lazy_static! {
            static ref HASH: String = regex::escape("#");
        }

        let id = ModId::from_str(id)?;
        let regex_str = regex::escape(text).replace(HASH.as_str(), VALUE_RE);
        Ok(ModInfo {
            id: id,
            regex: Regex::new(&regex_str)?,
        })
    }
}

impl ModInfo {
    /// ID of the mod.
    #[inline]
    pub fn id(&self) -> ModId {
        self.id
    }

    /// Returns how many parameters there are in the mod that are variable between items.
    ///
    /// Currently, all mods seem to have either zero parameters (e.g. "Has no sockets"),
    /// one parameter (e.g. "#% increased Attack Speed"), or two parameters
    /// (e.g. "Adds # to # Lightning Damage").
    #[inline]
    pub fn param_count(&self) -> usize {
        self.regex.captures_len() - 1  // -1 because the entire regex match is counted here
    }

    /// Parse mod text from an actual item and return the corresponding mod values.
    ///
    /// If the text doesn't match this mod, `None` is returned.
    pub fn parse_text(&self, text: &str) -> Option<Vec<ModValue>> {
        if self.param_count() == 0 {
            return Some(vec![]);
        }
        self.regex.captures(text).map(|caps| {
            caps.iter()
                .map(Option::unwrap)  // if there was a match, then every group caught a value
                .map(|m| parse_number(m.as_str()).unwrap())  // and they were all numbers
                .collect()
        })
    }
    // TODO: the opposite operation perhaps?
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &ModInfo) -> bool {
        self.id == other.id
    }
}
impl Eq for ModInfo {}


include!(concat!(env!("OUT_DIR"), "/", "model/item/mods/database.inc.rs"));


#[cfg(test)]
mod tests {
    use super::super::id::ModType;
    use super::ITEM_MODS;

    #[test]
    fn item_mods_db_is_valid() {
        // This will cause evaluation of the lazily initialized static.
        assert!(ITEM_MODS.len() > 0);
    }

    #[test]
    fn all_item_mod_types_in_db() {
        // Check that we have loaded mods of all types.
        for mt in ModType::iter_variants() {
            let mod_count = ITEM_MODS.keys().filter(|k| k.mod_type() == mt).count();
            assert!(mod_count > 0);
        }
    }
}
