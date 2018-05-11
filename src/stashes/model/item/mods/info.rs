//! Module definining the item mod info type.

use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use regex::{self, Regex, RegexBuilder};

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
            regex: RegexBuilder::new(&regex_str).case_insensitive(true).build()?,
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
            return Some(ModValues::none());
        }
        self.regex.captures(text.trim()).map(|caps| {
            caps.iter().skip(1)
                .map(Option::unwrap)  // if there was a match, then every group caught a value
                .map(|m| parse_number(m.as_str()).unwrap())  // and they were all numbers
                .collect()
        })
    }

    /// Format mod text using provided mod values.
    ///
    /// # Panics
    /// This function panics if given the incorrect number of mod values
    /// (different than `param_count`).
    pub fn format_text(&self, values: &ModValues) -> String {
        let expected = self.param_count();
        let actual = values.len();
        if expected != actual {
            panic!("Invalid number of mod values for `{}`: expected {}, got {}",
                self.text_template, expected, actual);
        }

        let mut text = self.text_template.clone();
        for value in values {
            text = text.replacen("#", &value.to_string(), 1);
        }
        text
    }
}

impl PartialEq for ModInfo {
    fn eq(&self, other: &ModInfo) -> bool {
        self.id == other.id
    }
}
impl Eq for ModInfo {}

impl fmt::Display for ModInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.text())
    }
}


#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::super::{ModType, ModValue, ModValues};
    use super::ModInfo;

    const MOD_ID: &str = "explicit.stat_42";

    const MOD_TEXT_TEMPLATE: &str = "+#% increased Awesomeness";
    const MOD_TEXT: &str = "+69% increased Awesomeness";
    const MOD_VALUE: ModValue = 69.0;

    const NO_VALUES_MOD_TEXT: &str = "Has ALL the sockets";

    #[test]
    fn constructor() {
        let mod_info = ModInfo::from_raw(MOD_ID, MOD_TEXT_TEMPLATE).unwrap();

        assert_eq!(ModType::Explicit, mod_info.id().mod_type());
        assert_eq!(42, mod_info.id().mod_number());
        assert_eq!(MOD_TEXT_TEMPLATE, mod_info.text());
    }

    #[test]
    fn parse_text__no_values() {
        let mod_info = ModInfo::from_raw(MOD_ID, NO_VALUES_MOD_TEXT).unwrap();
        let values = mod_info.parse_text(NO_VALUES_MOD_TEXT).unwrap();
        assert_eq!(0, values.len());
    }

    #[test]
    fn parse_text__with_values() {
        let mod_info = ModInfo::from_raw(MOD_ID, MOD_TEXT_TEMPLATE).unwrap();
        let values = mod_info.parse_text(MOD_TEXT).unwrap();

        assert_eq!(1, values.len());
        assert_eq!(MOD_VALUE, values[0]);
    }

    #[test]
    #[should_panic(expected = "Invalid number of mod values")]
    fn format_text__no_values__but_some_given() {
        let mod_info = ModInfo::from_raw(MOD_ID, NO_VALUES_MOD_TEXT).unwrap();
        mod_info.format_text(&ModValues::two(1, 2));
    }

    #[test]
    fn format_text__no_values() {
        let mod_info = ModInfo::from_raw(MOD_ID, NO_VALUES_MOD_TEXT).unwrap();
        let text = mod_info.format_text(&ModValues::none());
        assert_eq!(NO_VALUES_MOD_TEXT, text);
    }

    #[test]
    #[should_panic(expected = "Invalid number of mod values")]
    fn format_text__with_values__but_none_given() {
        let mod_info = ModInfo::from_raw(MOD_ID, MOD_TEXT_TEMPLATE).unwrap();
        mod_info.format_text(&ModValues::none());
    }

    #[test]
    fn format_text__with_values() {
        let mod_info = ModInfo::from_raw(MOD_ID, MOD_TEXT_TEMPLATE).unwrap();
        let text = mod_info.format_text(&ModValues::one(MOD_VALUE));
        assert_eq!(MOD_TEXT, text);
    }
}
