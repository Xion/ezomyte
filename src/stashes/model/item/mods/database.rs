//! Module definining the database of known mods.
//!
//! Note that the actual data here is filled in using a build script.

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use regex::{self, RegexSet, RegexSetBuilder};

use super::ModValues;
use super::id::{ModId, ModType};
use super::info::{ModInfo, regex_from_mod_text_template};


lazy_static! {
    /// Database of known item mods.
    pub static ref ITEM_MODS: Database = Database::new().unwrap();
}

/// Initialize the item database at this point in the program.
///
/// If you don't perform na explicit initialization,
/// the database will be initialized upon the first item mod lookup.
pub fn initialize() {
    let _ = &*ITEM_MODS;
}


/// Structure holding information about all known item mods.
pub struct Database {
    /// Mapping of mod types -> mod IDs -> mod infos.
    by_type_and_id: HashMap<ModType, HashMap<ModId, Arc<ModInfo>>>,
    /// Map of `RegexMatcher`s by `ModType`.
    ///
    /// This is used during `Item` deserialization to lookup the mods by their
    /// UI texts (e.g. "+7% increased Maximum Life").
    matchers_by_type: HashMap<ModType, RegexMatcher<Arc<ModInfo>>>,
}

impl Database {
    /// Create the database and initialize it with known mods.
    fn new() -> Result<Self, Box<Error>> {
        lazy_static! {
            /// Mod text prefixes for the `RegexMatcher` shards.
            ///
            /// This is used to divide the space of mods into several buckets/shards
            /// to speed up the matching of actual mod texts against the known templates.
            static ref PREFIXES: Vec<String> = include_str!("mod-text-prefixes.txt")
                .split("\n")
                .filter(|line| !line.is_empty() && !line.trim_left().starts_with("//"))
                .map(regex_from_mod_text_template)
                .map(|p| p.trim_right_matches("$").to_owned())  // `^...$` -> `^...`
                .collect();
        }

        // This is created by the build script from the JSON files in `data/mods`.
        let by_type_and_id = include!(concat!(
            env!("OUT_DIR"), "/", "model/item/mods/database/by_type_and_id.inc.rs"));
        let matchers_by_type = by_type_and_id.iter()
            .map(|(&mod_type, id2info)| {
                let matcher = RegexMatcher::with_shards(
                    PREFIXES.iter().map(|p| p.as_str()),
                    id2info.values().map(|mi| (mi.regex.as_str(), mi.clone())))?;
                Ok((mod_type, matcher))
            })
            .collect::<Result<HashMap<_, _>, Box<Error>>>()?;
        Ok(Database{by_type_and_id, matchers_by_type})
    }
}

impl Database {
    /// Returns an iterator over all mods.
    #[inline]
    pub fn iter<'d>(&'d self) -> impl Iterator<Item=&'d ModInfo> + 'd {
        self.matchers_by_type.values().flat_map(|rm| rm.items().map(|mi| &**mi))
    }

    /// Total number of mods in the database.
    #[inline]
    pub fn len(&self) -> usize {
        self.matchers_by_type.values().map(|rm| rm.count()).sum()
    }

    /// Lookup a mod by its `ModId`.
    #[inline]
    pub(super) fn lookup(&self, id: ModId) -> Option<Arc<ModInfo>> {
        self.by_type_and_id.get(&id.mod_type())
            .and_then(|id2info| id2info.get(&id))
            .map(|mi| mi.clone())
    }

    /// Resolve a mod's actual text on an item.
    ///
    /// Returns the matched `ModInfo` and the values parsed from the text.
    pub(super) fn resolve(&self, mod_type: ModType, text: &str) -> Option<(Arc<ModInfo>, ModValues)> {
        let matcher =  self.matchers_by_type.get(&mod_type)?;

        let mut matching_mods = matcher.lookup_all(text);
        let match_count = matching_mods.len();
        let mod_ = match match_count {
            0 => { return None; }
            1 => {
                let mod_ = matching_mods.into_iter().next().unwrap();
                trace!("Mod text {:?} matched {:?}", text, mod_);
                mod_
            }
            n => {
                // In case of multiple matches, pick the longest one.
                // This should address picking the correct variants between cases like
                // "#% increased Attack Damage" vs. "#% increased Attack Damage per 450 Evasion Rating".
                warn!("Mod text `{}` ({:?}) matched {} mods: {:?}",
                    text, mod_type, n, matching_mods);
                matching_mods.sort_by_key(|mi| -(mi.regex.as_str().len() as isize));
                matching_mods.into_iter().next().unwrap()
            }
        };

        let values = mod_.parse_text(text)
            .expect(&format!("mod values for {:?} after parsing by {:?}", text, mod_));
        Some((mod_.clone(), values))
    }
}

impl fmt::Debug for Database {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Database(<{} mods>)", self.len())
    }
}


// Mod text matching

/// Matcher from regular expressions to some other arbitrary types.
///
/// This is used to match mods texts to `ModInfo`s.
#[derive(Debug)]
struct RegexMatcher<T> {
    /// Shards of the regex matcher.
    shards: Option<RegexMatcherShard<RegexMatcherShard<T>>>,
    /// Fallback shard for when the item doesn't match any of the shards.
    fallback: Option<RegexMatcherShard<T>>,
}

impl<T> RegexMatcher<T> {
    /// Create a `RegexMatcher` that doesn't actually match anything.
    pub fn empty() -> Self {
        RegexMatcher{shards: None, fallback: None}
    }

    /// Create a simple `RegexMatcher` that doesn't use sharding.
    pub fn new<'r, I>(mapping: I) -> Result<Self, Box<Error>>
        where I: IntoIterator<Item=(&'r str, T)>
    {
        Ok(RegexMatcher{
            shards: None,
            fallback: Some(RegexMatcherShard::new(mapping)?),
        })
    }

    /// Create a `RegexMatcher` that matches regular expressions to some other arbitrary types.
    ///
    /// The regular expressions are sharded into subsets based on their given prefixes
    /// in order to speed up matching over a large sequence of regexes.
    pub fn with_shards<'r, P, I>(prefixes: P, mapping: I) -> Result<Self, Box<Error>>
        where P: IntoIterator<Item=&'r str>,
              I: IntoIterator<Item=(&'r str, T)>
    {
        let mut prefixes: Vec<&str> = prefixes.into_iter().collect();
        if prefixes.is_empty() {
            return Self::new(mapping);
        }
        // Make sure the prefixes which are themselves prefixes of another prefixes (yo dawg)
        // are placed later in the prefix list by sorting on descending length.
        prefixes.sort_by_key(|p| -(p.len() as isize));

        // Create a temporary matcher shard to dispatch the regexes
        // into their final, prefix-based shards.
        //
        // This is a little tricky since we are basically using a regex matcher (for prefixes)
        // in order to match **regexes** (from `mapping`) so that we can split them into buckets/shards.
        // (That's why there is a `regex::escape` applied to each prefix).
        let prefix_regexes: Vec<_> = prefixes.iter()
            .map(|p| format!("^{}", regex::escape(p)))
            .collect();
        let prefix_matcher = RegexMatcherShard::new(
            prefix_regexes.iter().map(|p| p.as_str()).enumerate().map(|(i, p)| (p, i)))?;

        // Divide the (regex, item) pairs into those shards.
        let mut shard_splits: Vec<Vec<_>> = (0..prefixes.len()).map(|_| Vec::new()).collect();
        let mut catchall = Vec::new();
        for (re, item) in mapping {
            // Pick the first shard that matches this regex.
            let shard = match prefix_matcher.lookup_all(re).into_iter().next() {
                Some(&idx) => {
                    trace!("Mod regex `{}` goes in shard for prefix `{}`", re, prefixes[idx]);
                    &mut shard_splits[idx]
                },
                None => {
                    trace!("Mod regex `{}` goes in the fallback shard", re);
                    &mut catchall
                },
            };
            shard.push((re, item));
        }

        // Create the final matcher of matchers.
        //
        // This is using the same set of prefixes at the outer level,
        // but now they are actually being used normally, w/o escaping.
        let prefix_regexes: Vec<Cow<str>> = prefixes.iter()
            .map(|&p| if p.starts_with("^") { p.into() } else { format!("^{}", p).into() })
            .collect();
        let shards_mapping =
            prefix_regexes.iter().map(|p| &**p).zip(shard_splits.into_iter())
                // Eliminate empty shards.
                // Note that this may mean all shards are empty,
                // in which case we'll only use the `fallback` (if any).
                .filter(|&(_, ref shard)| !shard.is_empty())
                .map(|(p, shard)| {
                    let shard = RegexMatcherShard::new(shard)?;
                    Ok((p, shard))
                })
                .collect::<Result<Vec<_>, Box<Error>>>()?;
        let shards = if shards_mapping.is_empty() { None } else {
            Some(RegexMatcherShard::new(shards_mapping)?)
        };

        // As for the regular expressions that didn't fall into any prefix bucket,
        // put them into the fallback shard.
        let fallback = if catchall.is_empty() { None } else {
            Some(RegexMatcherShard::new(catchall)?)
        };

        Ok(RegexMatcher{shards, fallback})
    }
}
impl<T> Default for RegexMatcher<T> {
    fn default() -> Self {
        RegexMatcher::empty()
    }
}

impl<T> RegexMatcher<T> {
    /// Return the number of regular expressions being matched against.
    pub fn count(&self) -> usize {
        let regular_count = match self.shards {
            Some(ref shards) => shards.items().map(|shard| shard.count()).sum(),
            None => 0,
        };
        regular_count + self.fallback.iter().count()
    }

    /// Whether a fallback (catch-all) shard is used by this matcher.
    pub fn has_fallback(&self) -> bool {
        self.fallback.is_some()
    }

    /// Return an iterator over all possible items.
    #[inline]
    pub fn items<'r>(&'r self) -> impl Iterator<Item=&'r T> + 'r {
        Box::new(
            self.shards.iter()
                .flat_map(|shards| { shards.items().flat_map(|sh| sh.items()) })
                .chain(self.fallback.iter().flat_map(|fb| fb.items()))
        )
    }

    /// Try to match given text against all known patterns
    /// and return the first match.
    ///
    /// Sharded patterns are tried first, followed by the optional fallback.
    pub fn lookup<'m, 's>(&'m self, text: &'s str) -> Option<&'m T> {
        self.shards.as_ref().and_then(|shards| {
                shards.lookup(text).and_then(|sh| sh.lookup(text))
            }).or_else(|| {
                self.fallback.as_ref().and_then(|fb| fb.lookup(text))
            })
    }

    /// Try to match given text against known patterns
    /// and return all matches.
    ///
    /// Sharded patterns are tried first, and all shards are queried.
    /// Only if they haven't yielded ANY matches, the optional fallback is then tried.
    pub fn lookup_all<'m, 's>(&'m self, text: &'s str) -> Vec<&'m T> {
        let regular_matches = match self.shards {
            Some(ref shards) => {
                shards.lookup_all(text).into_iter()
                    .flat_map(|sh| sh.lookup_all(text))
                    .collect()
            }
            None => Vec::new(),
        };
        if !regular_matches.is_empty() {
            return regular_matches;
        }

        self.fallback.as_ref().map(|fb| fb.lookup_all(text))
            .unwrap_or_else(Vec::new)
    }

    /// Total number of shards used (incl. the possible fallback).
    pub fn shard_count(&self) -> usize {
        let regular_shards = self.shards.as_ref().map(|shards| shards.count()).unwrap_or(0);
        let fallback_shard = if self.has_fallback() { 1 } else { 0 };
        regular_shards + fallback_shard
    }
}


/// A single `RegexMatcher` shard.
#[derive(Debug)]
struct RegexMatcherShard<T> {
    /// Regex set for doing the actual matching.
    regex_set: RegexSet,
    /// List of items that the regexes map to.
    items: Vec<T>,
}

impl<T> RegexMatcherShard<T> {
    /// Create a new `RegexMatcherShard` given a mapping in the form of iterable of pairs.
    #[inline]
    pub fn new<'r, I>(mapping: I) -> Result<Self, Box<Error>>
        where I: IntoIterator<Item=(&'r str, T)>
    {
        let mut regexes = Vec::new();
        let mut items = Vec::new();
        for (regex, item) in mapping.into_iter() {
            regexes.push(regex);
            items.push(item);
        }

        let regex_set = RegexSetBuilder::new(regexes)
            .case_insensitive(true)
            .size_limit(SHARD_REGEXSET_SIZE_LIMIT_BYTES)
            // TODO: .dfa_size_limit() too?
            .build()?;
        Ok(RegexMatcherShard{regex_set, items})
    }
}

/// Size limit for a single compiled set of regular expressions
/// in the `RegexMatcherShard`.
///
/// *Note*: If this limits gets exceeded, it is preferrable to adjust the sharding
/// (in mods-text-prefixes.txt) rather than raising it.
const SHARD_REGEXSET_SIZE_LIMIT_BYTES: usize = 8 * 1024 * 1024;

impl<T> RegexMatcherShard<T> {
    /// Return the number of regular expressions being matched against by this shard.
    #[inline]
    pub fn count(&self) -> usize {
        self.regex_set.len()
    }

    /// Return an iterator over the possible items.
    #[inline]
    pub fn items<'r>(&'r self) -> impl Iterator<Item=&'r T> + 'r {
        Box::new(self.items.iter())
    }

    /// Try to match given text against all regular expressions in the shard,
    /// and return a reference to the corresponding item that matched (if any).
    pub fn lookup<'m, 's>(&'m self, text: &'s str) -> Option<&'m T> {
        let matched = self.lookup_all(text);
        if matched.len() > 1 {
            warn!("Ambiguous text in RegexMatcherShard::lookup(): {:?} (matched {} items)",
                text, matched.len());
            return None;
        }
        matched.into_iter().next()
    }

    /// Try to match given text against all regular expressions in the shard.
    /// Returns a reference to all items that matched.
    pub fn lookup_all<'m, 's>(&'m self, text: &'s str) -> Vec<&'m T> {
        self.regex_set.matches(text).iter().map(|idx| &self.items[idx]).collect()
    }
}


#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::super::id::ModType;
    use super::{initialize, ITEM_MODS};

    #[test]
    fn initialize_actually_initializes() {
        let first_init_duration = {
            let start = Instant::now();
            initialize();
            Instant::now().duration_since(start)
        };
        let second_init_time = {
            let start = Instant::now();
            initialize();
            Instant::now().duration_since(start)
        };
        // The factor here is super conservative.
        assert!(first_init_duration >= second_init_time * 100);
    }
    // TODO: benchmark for the initialization

    #[test]
    fn item_mods_db_is_valid() {
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
