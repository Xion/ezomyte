//! Module definining the database of known mods.
//!
//! Note that the actual data here is filled in using a build script.

use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use regex::{self, Regex, RegexSet, RegexSetBuilder};

use super::ModValues;
use super::id::{ModId, ModType};
use util::parse_number;


// TODO: the item database takes quite a bit of space in memory,
// so the support for it should be gated behind a flag
lazy_static! {
    /// Database of known item mods.
    pub static ref ITEM_MODS: Database = Database::new().unwrap();
}

/// Structure holding information about all known item mods.
pub struct Database {
    /// Mapping of mod IDs to their infos.
    by_id: HashMap<ModId, Arc<ModInfo>>,
    /// Regex set for quickly matching actual occurrences of mods on items.
    regexes: RegexSet,
    /// All mods in the order corresponding to the `regexes` order of regular expressions.
    all_mods: Vec<Arc<ModInfo>>,
}

impl Database {
    /// Create the database and initialize it with known mods.
    fn new() -> Result<Self, Box<Error>> {
        let by_id = include!(concat!(
            env!("OUT_DIR"), "/", "model/item/mods/database/by_id.inc.rs"));
        let all_mods: Vec<_> = by_id.values().cloned().collect();
        let regexes = RegexSetBuilder::new(all_mods.iter().map(|mi| mi.regex.as_str()))
            .case_insensitive(true)
            .size_limit(MOD_REGEXES_SIZE_LIMIT_BYTES)
            // TODO: .dfa_size_limit() too?
            .build()?;
        Ok(Database{by_id, all_mods, regexes})
    }
}

impl Database {
    /// Returns an iterator over all mods.
    #[inline]
    pub fn iter<'d>(&'d self) -> Box<Iterator<Item=&'d ModInfo> + 'd> {
        Box::new(self.all_mods.iter().map(|mi| &**mi))
    }

    /// Total number of mods in the database.
    #[inline]
    pub fn len(&self) -> usize {
        self.all_mods.len()
    }

    /// Lookup a mod by its actual text on an item & mod type.
    pub(super) fn lookup(&self, mod_type: ModType, text: &str) -> Option<(Arc<ModInfo>, ModValues)> {
        let mut matched_mods = Vec::new();
        for idx in self.regexes.matches(text.trim()).iter() {
            let mod_ = &self.all_mods[idx];
            if mod_.id().mod_type() == mod_type {
                matched_mods.push(mod_);
            }
        }
        if matched_mods.len() > 1 {
            warn!("Mod text {:?} matched {} (>1) known mods!", text, matched_mods.len());
            return None;
        }
        matched_mods.into_iter().next().map(|mod_| {
            trace!("Mod text {:?} matched {:?}", text, mod_);
            let values = mod_.parse_text(text)
                .expect(&format!("mod values for {:?} after parsing by {:?}", text, mod_));
            (mod_.clone(), values)
        })
    }
}

impl fmt::Debug for Database {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Database(<{} mods>)", self.len())
    }
}

/// Size limit for the compiled set of regular expression for all mod texts'.
///
/// We need to override it explicitly because the default (which seem to be 10MB)
/// is not enough to hold the `RegexSet` of all item mod texts.
const MOD_REGEXES_SIZE_LIMIT_BYTES: usize = 64 * 1024 * 1024;


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
        const VALUE_RE: &str = r"(\d+(?:\.\d+)?)";  // Regex for integer or float values.
        lazy_static! {
            static ref HASH: String = regex::escape("#");
        }

        let id = ModId::from_str(id)?;
        let regex_str = format!("^{}$",
            regex::escape(text.trim()).replace(HASH.as_str(), VALUE_RE));
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
    pub fn parse_text(&self, text: &str) -> Option<ModValues> {
        if self.param_count() == 0 {
            return Some(ModValues::new());
        }
        self.regex.captures(text.trim()).map(|caps| {
            caps.iter().skip(1)
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
            let mod_count = ITEM_MODS.iter().filter(|mi| mi.id().mod_type() == mt).count();
            assert!(mod_count > 0);
        }
    }
}
