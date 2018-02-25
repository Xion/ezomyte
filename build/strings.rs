//! Utility string functions.

use itertools::Itertools;


/// Convert a string from "Regular Words and Title Case" to "RegularWordsAndTitleCase".
pub fn upper_camel_case(s: &str) -> String {
    // TODO: return Cow
    s.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace().map(capitalize).join("")
}


/// Capitalize a string (make the first character uppercase and rest lowercase).
pub fn capitalize(s: &str) -> String {
    // TODO: return Cow
    let mut result = String::new();
    if s.is_empty() {
        return result;
    }
    result.push_str(s.chars().next().unwrap().to_uppercase().to_string().as_str());
    result.push_str(s.chars().skip(1).collect::<String>().to_lowercase().as_str());
    result
}
