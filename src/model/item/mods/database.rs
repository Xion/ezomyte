//! Module definining the database of known mods.
//!
//! Note that the actual data here is filled in using a build script.

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::iter;
use std::str::FromStr;
use std::sync::Arc;

use regex::{self, RegexSet, RegexSetBuilder};

use super::ModValues;
use super::id::{ModId, ModType};
use super::info::ModInfo;


// TODO: the item database takes quite a bit of space in memory,
// so the support for it should be gated behind a flag
lazy_static! {
    /// Database of known item mods.
    pub static ref ITEM_MODS: Database = Database::new().unwrap();
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
        let by_type_and_id = include!(concat!(
            env!("OUT_DIR"), "/", "model/item/mods/database/by_type_and_id.inc.rs"));
        let matchers_by_type = by_type_and_id.iter()
            .map(|(&mod_type, id2info)| {
                // TODO: use RegexMatcher with prefix shards
                let matcher = RegexMatcher::new(
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
    pub fn iter<'d>(&'d self) -> Box<Iterator<Item=&'d ModInfo> + 'd> {
        Box::new(
            self.matchers_by_type.values().flat_map(|rm| rm.items().map(|mi| &**mi))
        )
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
        self.matchers_by_type.get(&mod_type).and_then(|rm| rm.lookup(text)).map(|mod_| {
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


/// Matcher from regular expressions to some other arbitrary types.
///
/// This is used to match mods texts to `ModInfo`s.
#[derive(Debug)]
struct RegexMatcher<T> {
    shards: RegexMatcherShard<RegexMatcherShard<T>>,
}

impl<T> RegexMatcher<T> {
    /// Create a simple `RegexMatcher` that doesn't use sharding.
    pub fn new<'r, I>(mapping: I) -> Result<Self, Box<Error>>
        where I: IntoIterator<Item=(&'r str, T)>
    {
        const CATCH_ALL_RE: &str = ".*";  // matches everything

        let inner_shard = RegexMatcherShard::new(mapping)?;
        let shards = RegexMatcherShard::new(iter::once((CATCH_ALL_RE, inner_shard)))
            .expect("outer shard in RegexMatcher::new");
        Ok(RegexMatcher{shards})
    }

    /// Create a `RegexMatcher` that matches regular to some other arbitrary types.
    ///
    /// The regular expressions are sharded into subsets based on their given prefixes
    /// to speed up matching over a large sequence of regexes.
    pub fn with_shards<'r, P, I>(prefixes: P, mapping: I) -> Result<Self, Box<Error>>
        where P: IntoIterator<Item=&'r str>,
              I: IntoIterator<Item=(&'r str, T)>
    {
        let prefixes: Vec<&str> = prefixes.into_iter().collect();

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
        for (re, item) in mapping {
            let shard_idx = match prefix_matcher.lookup(re) {
                Some(&idx) => idx,
                None => {
                    // TODO: maybe this should be an error?
                    warn!("No matching prefix shard for regex: {}", re);
                    continue;
                }
            };
            shard_splits[shard_idx].push((re, item));
        }

        // Create the final matcher of matchers.
        //
        // This is using the same set of prefixes at the outer level,
        // but now they are actually being used normally, w/o escaping.
        let prefix_regexes: Vec<Cow<str>> = prefixes.iter()
            .map(|&p| {
                if p.starts_with("^") { p.into() } else { format!("^{}", p).into() }
            })
            .collect();
        let shards = RegexMatcherShard::new(
            prefix_regexes.iter().map(|p| &**p).zip(shard_splits.into_iter())
                .map(|(p, shard)| {
                    let shard = RegexMatcherShard::new(shard.into_iter())?;
                    Ok((p, shard))
                })
                .collect::<Result<Vec<_>, Box<Error>>>()?
        )?;

        Ok(RegexMatcher{shards})
    }
}

impl<T> RegexMatcher<T> {
    /// Return the number of regular expressions being matched against.
    pub fn count(&self) -> usize {
        self.shards.items().map(|shard| shard.count()).sum()
    }

    /// Return an iterator over all possible items.
    #[inline]
    pub fn items<'r>(&'r self) -> Box<Iterator<Item=&'r T> + 'r> {
        Box::new(self.shards.items().flat_map(|shard| shard.items()))
    }

    pub fn lookup<'m, 's>(&'m self, text: &'s str) -> Option<&'m T> {
        self.shards.lookup(text).and_then(|shard| shard.lookup(text))
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
            .size_limit(MOD_REGEXES_SIZE_LIMIT_BYTES)
            // TODO: .dfa_size_limit() too?
            .build()?;
        Ok(RegexMatcherShard{regex_set, items})
    }
}

impl<T> RegexMatcherShard<T> {
    /// Return the number of regular expressions being matched against by this shard.
    #[inline]
    pub fn count(&self) -> usize {
        self.regex_set.len()
    }

    /// Return an iterator over the possible items.
    #[inline]
    pub fn items<'r>(&'r self) -> Box<Iterator<Item=&'r T> + 'r> {
        Box::new(self.items.iter())
    }

    /// Try to match given text against all regular expressions in the shard,
    /// and return a reference to the corresponding item that matched (if any).
    pub fn lookup<'m, 's>(&'m self, text: &'s str) -> Option<&'m T> {
        let mut matched = Vec::new();
        for idx in self.regex_set.matches(text).iter() {
            matched.push(&self.items[idx]);
        }
        if matched.len() > 1 {
            warn!("Ambiguous text for regular expression matching: {:?}", text);
            return None;
        }
        matched.into_iter().next()
    }
}

/// Size limit for the compiled set of regular expression for all mod texts'.
///
/// We need to override it explicitly because the default (which seem to be 10MB)
/// is not enough to hold the `RegexSet` of all item mod texts.
const MOD_REGEXES_SIZE_LIMIT_BYTES: usize = 48 * 1024 * 1024;


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
