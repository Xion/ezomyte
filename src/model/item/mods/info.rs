//! Module definining the item mod info type.

use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::str::FromStr;

use regex::{self, Regex};

use util::parse_number;
use super::ModValues;
use super::id::ModId;


/// Information about a single known item mod.
#[derive(Clone, Debug)]
pub struct ModInfo {
    /// Identifier of the mod.
    ///
    /// This is parsed from a string such as "implicit.stat_587431675".
    id: ModId,
    /// Template for the mod text.
    ///
    /// This contains the placeholders for mod values, e.g. "+# to maximum Life".
    text_template: String,
    /// Regular expression matching the textual form of the mod
    /// as it appears on items.
    ///
    /// Within the regex, indexed (unnamed) capture groups are expected to be filled
    /// by specific mod values, e.g. "(\d+)% increased Global Critical Strike Chance".
    pub(super) regex: Regex,
}

impl ModInfo {
    /// Create `ModInfo` from given mod ID and its text (as obtained from the PoE API).
    pub(super) fn from_raw<T: Into<String>>(id: &str, text: T) -> Result<Self, Box<Error>> {
        let id = ModId::from_str(id)?;
        let text = text.into();
        let regex_str = regex_from_mod_text_template(&text);
        Ok(ModInfo {
            id: id,
            text_template: text,
            regex: Regex::new(&regex_str)?,
        })
    }
}

/// Convert a mod text template from PoE API (like "#% increased Life")
/// to a regular expression that can match the actual mod texts
/// (something like "^(\d+)% increased Life$").
pub(super) fn regex_from_mod_text_template(text: &str) -> String {
    /// Regex for integer or float values that occur in mod texts.
    const VALUE_RE: &str = r"\+?(\d+(?:\.\d+)?)";
    lazy_static! {
        /// Placeholder for mod value in the text template.
        /// It is replaced with `VALUE_RE` when converting mod text template to a regex.
        static ref VALUE_PH: String = regex::escape("#");
    }
     format!("^{}$", regex::escape(text.trim()).replace(VALUE_PH.as_str(), VALUE_RE))
}

impl ModInfo {
    /// ID of the mod.
    #[inline]
    pub fn id(&self) -> ModId {
        self.id
    }

    /// Expected format of the mod's text on an item.
    ///
    /// This is a string that in most cases contains hashes (`#`) as placeholders
    /// for mod's numerical, e.g. "+#% increased Attack Speed".
    #[inline]
    pub fn text(&self) -> &str {
        self.text_template.as_str()
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
