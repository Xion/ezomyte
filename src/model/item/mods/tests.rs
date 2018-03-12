//! Tests for the item mods' module.

lazy_static! {
    /// Item mods prefixes that `database::Database` uses for sharding
    /// (in raw form, w/o them being converted to regular expressions).
    static ref MOD_TEXT_PREFIXES: Vec<&'static str> = include_str!("mod-text-prefixes.txt")
        .split("\n")
        .filter(|line| !line.is_empty() && !line.trim_left().starts_with("//"))
        .collect();
}


/// Verify that on the list of mod text prefixes,
/// those that are themselves a string-prefixes of others
/// are placed after those whose prefixes they are.
///
/// Since this description is confusing, here's an example:
///
///    abacus
///    abc
///    abnormal
///    abn
///    ab
///
/// This prefix file is well-formed, because "abn" is after "abnormal"
/// (`"abnormal".starts_with("abn")` is true)
/// and "ab" is after every other entry (because every other entry starts with "ab").
#[test]
fn prefixes_of_prefixes_are_correctly_ordered() {
    let count = MOD_TEXT_PREFIXES.len();
    for i in 0..count {
        for j in (i + 1)..count {
            let earlier = MOD_TEXT_PREFIXES[i];
            let later = MOD_TEXT_PREFIXES[j];
            assert!(!later.starts_with(earlier),
                "Prefix `{}` starts with `{}` which comes earlier in the list!",
                later, earlier);
        }
    }
}
// TODO: this test would be unnecessary if, rather than relying on the correct order,
// we would sort the list of prefixes topologically based on the starts_with relation
