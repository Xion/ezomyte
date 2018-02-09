//! Item properties.
//!
//! These properties are obtained from the "properties" key in the JSON response from API,
//! and for the most part are very much specific to a particular item class.

use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::iter::FromIterator;

use super::super::util::ExplicitDebug;


/// Type of a property key.
pub type Key = String;
// TODO: we could recognize common properties and represent them as an enum


/// Type of a property value.
pub type Value = String;
// TODO: introduce an enum here that corresponds to the valueType enum from the API
// (mostly to hold the types of elemental damage, and whether or not it was affected by an affix)


/// Container for miscellaneous item properties.
/// These are commonly obtained from the "properties" JSON array in the stash tabs API response.
///
/// Most properties have _values_ associated with them (e.g. weapon damage ranges),
/// though some serve as mere "markers" or "tags" (like gem classes: Spell, Support, Minion, etc.).
#[derive(Default)]
pub struct Properties {
    set: HashSet<Key>,
    map: HashMap<Key, Value>,
    // Marker to ensure the type doesn't get the `Sync` trait derived
    // because some mutation methods cannot be thread-safe w/o locks.
    _marker: UnsafeCell<()>,
}

impl Properties {
    /// Create a new property container.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    // TODO: with_hasher()
}

// Accessors.
impl Properties {
    /// Checks whether given property exists.
    #[inline]
    pub fn contains<K: ?Sized>(&self, k: &K) -> bool
        where Key: Borrow<K>, K: Hash + Eq
    {
        self.set.contains(k) || self.map.contains_key(k)
    }

    /// Retrieve the `Value` of given property, if it exists and has one.
    #[inline]
    pub fn get_value<'p, K: ?Sized>(&'p self, k: &K) -> Option<&'p Value>
        where Key: Borrow<K>, K: Hash + Eq
    {
        self.map.get(k)
    }

    /// Checks whether the properties container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.map.is_empty()
    }

    /// Return an iterator over property keys & optional values as pairs.
    #[inline]
    pub fn iter<'p>(&'p self) -> Box<Iterator<Item=(&'p Key, Option<&'p Value>)> + 'p> {
        IntoIterator::into_iter(self)
    }

    /// Return an iterator over property keys (names).
    #[inline]
    pub fn keys<'p>(&'p self) -> Box<Iterator<Item=&'p Key> + 'p> {
        Box::new(
            self.set.iter().chain(self.map.keys())
        )
    }
    // TODO: iterator methods for both value-full and value-less properties

    /// Returns the total number of properties (with or without values).
    #[inline]
    pub fn len(&self) -> usize {
        self.set.len() + self.map.len()
    }
}

// Mutators.
impl Properties {
    /// Clears the container, removing all properties.
    pub fn clear(&mut self) {
        self.set.clear();
        self.map.clear();
    }
    // TODO: drain()

    /// Insert a new property, possibly with a value, into the container.
    ///
    /// If `opt_value` is `None`, the property will be inserted
    /// without any value associated with it.
    ///
    /// If the property exists already, its value is updated and the previous one,
    /// if any, is returned. This means that in case the property existed before
    /// but didn't have a value, the function will return `Some(None)`.
    pub fn insert(&mut self, key: Key, opt_value: Option<Value>) -> Option<Option<Value>> {
        match opt_value {
            Some(value) => self.put_with_value(key, value),
            None => self.put(key),
        }
    }

    /// Returns an iterator of all over property keys & their optional values.
    ///
    /// The type of iterator element is `(&Key, Option<&mut Value>)`.
    /// Notice that this doesn't allow to add/remove values associated with properties,
    /// but only modify the value if they already have one.
    pub fn iter_mut<'p>(&'p mut self) -> Box<Iterator<Item=(&'p Key, Option<&'p mut Value>)> + 'p> {
        Box::new(
            self.set.iter().map(|k| (k, None))
                .chain(self.map.iter_mut().map(|(k, v)| (k, Some(v))))
        )
    }
    // TODO: an Entry-like API to insert properties and add/remove their values

    /// Insert a new, value-less property into the container.
    ///
    /// If the property exists already, its value is reset and the previous one,
    /// if any, is returned. This means that in case the property existed before
    /// but didn't have a value, the function will return `Some(None)`.
    ///
    /// If the property didn't exist already, `None` is returned.
    pub fn put(&mut self, key: Key) -> Option<Option<Value>> {
        let prev = self.map.remove(&key);
        // There is a brief window between the above and below calls where the container
        // is in transitional state that shouldn't be observed,
        // which is why it is made thread-unsafe (i.e. not `Sync`).
        if self.set.insert(key) { Some(prev) } else { None }
    }

    /// Insert a new property with given value into the container.
    ///
    /// If the property exists already, its value is updated and the previous one,
    /// if any, is returned. This means that in case the property existed before
    /// but didn't have a value, the function will return `Some(None)`.
    pub fn put_with_value(&mut self, key: Key, value: Value) -> Option<Option<Value>> {
        let was_in_set = self.set.remove(&key);
        //  The same note about brief transitional state applies here.
        match self.map.insert(key, value) {
            Some(prev) => Some(Some(prev)),
            None => if was_in_set { Some(None) } else { None },
        }
    }

    /// Remove a property from the container
    /// and return the value associated with it, if any.
    ///
    /// If the property didn't exist, this method will return `None`.
    /// Contrast this with the situation where the property existed
    /// but didn't have a value associated with it,
    /// in which case the return value will be `Some(None)`.
    pub fn remove<K: ?Sized>(&mut self, key: &K) -> Option<Option<Value>>
        where Key: Borrow<K>, K: Hash + Eq
    {
        if self.set.remove(key) {
            Some(None)
        } else {
            self.map.remove(key).map(Some)
        }
    }
}

impl FromIterator<Key> for Properties {
    fn from_iter<T: IntoIterator<Item=Key>>(iter: T) -> Self {
        Properties{set: iter.into_iter().collect(), ..Properties::default()}
    }
}
impl FromIterator<(Key, Value)> for Properties {
    fn from_iter<T: IntoIterator<Item=(Key, Value)>>(iter: T) -> Self {
        Properties{map: iter.into_iter().collect(), ..Properties::default()}
    }
}
impl FromIterator<(Key, Option<Value>)> for Properties {
    fn from_iter<T: IntoIterator<Item=(Key, Option<Value>)>>(iter: T) -> Self {
        let (map_items, set_items): (Vec<_>, Vec<_>) =
            iter.into_iter().partition(|&(_, ref v)| v.is_some());
        Properties {
            map: map_items.into_iter().map(|(k, v)| (k, v.unwrap())).collect(),
            set: set_items.into_iter().map(|(k, _)| k).collect(),
            ..Properties::default()
        }
    }
}

impl IntoIterator for Properties {
    type Item = (Key, Option<Value>);
    type IntoIter = Box<Iterator<Item=Self::Item>>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.set.into_iter().map(|p| (p, None))
                .chain(self.map.into_iter().map(|(k, v)| (k, Some(v))))
        )
    }
}
impl<'p> IntoIterator for &'p Properties {
    type Item = (&'p Key, Option<&'p Value>);
    type IntoIter = Box<Iterator<Item=Self::Item> + 'p>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.set.iter().map(|p| (p, None))
                .chain(self.map.iter().map(|(k, v)| (k, Some(v))))
        )
    }
}

impl fmt::Debug for Properties {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set()
            .entries(
                self.iter().map(|(k, v)| {
                    format!("\"{}\"{}", k, v.map(|v| format!(": \"{}\"", v))
                        .unwrap_or_else(String::new))
                })
                .map(ExplicitDebug::from))
            .finish()
    }
}
