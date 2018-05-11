//! Quasi-equivalent type wrapper.

use std::cmp::Ordering;
use std::fmt;
use std::iter;
use std::str::FromStr;

use conv::errors::NoError;
use serde::de::{Deserialize, Deserializer, IntoDeserializer};
use serde::ser::{Serialize, Serializer};

use super::json::Json;


/// Quasi-equivalent wrapper over a deserialized type.
///
/// Basically, `Quasi<T>` is simply wrapping `T` in the vast majority of cases.
/// The only exception is when `T` couldn't be deserialized successfully,
/// in which case the original source/subsititute representation (`S`)
/// is made available.
///
/// Because `Quasi<T>` is essentially `T`, you'd normally just call the `get` method
/// and work with the resulting `&T` reference directly.
/// Alternatively, the wrapper itself offers an `Option`-like interface for
/// safely accessing `T`.
/// There is intentionally no equivalent interface to access the subsititute value
/// outside of pattern matching.
#[must_use]
pub enum Quasi<T, S = Json> {
    /// The usual, "true" value.
    True(T),
    /// The substitute, "fake", undeserialized value.
    Substitute(S),
}

// Conversions from external types.
impl<T, S> From<T> for Quasi<T, S> {
    fn from(v: T) -> Self {
        Quasi::True(v)
    }
}
impl<T: Default, S> Default for Quasi<T, S> {
    fn default() -> Self {
        Quasi::True(Default::default())
    }
}

// (De)serialization / parsing.
impl<'de, T, S> Deserialize<'de> for Quasi<T, S>
    where T: Deserialize<'de>,
          S: Clone + Deserialize<'de> + IntoDeserializer<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        // Get the source / possible subsititute first,
        // then try to deserialize the actual value, falling back its source representation
        // if it fails.
        let source: S = Deserialize::deserialize(deserializer)?;
        let source_de = IntoDeserializer::into_deserializer(source.clone());
        Ok(match T::deserialize(source_de) {
            Ok(v) => Quasi::True(v),
            // TODO: do something with the error?
            Err(_) => Quasi::Substitute(source),
        })
    }
}
impl<T: FromStr> FromStr for Quasi<T, String> {
    type Err = NoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match T::from_str(s) {
            Ok(v) => Quasi::True(v),
            // TODO: do something with the error?
            Err(_) => Quasi::Substitute(s.to_owned()),
        })
    }
}
impl<T, S> Serialize for Quasi<T, S>
    where T: Serialize,
          S: Serialize
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
        where Ser: Serializer
    {
        match *self {
            Quasi::True(ref v) => Serialize::serialize(v, serializer),
            Quasi::Substitute(ref v) => Serialize::serialize(v, serializer),
        }
    }
}

// Unconditional (panicking) accessors.
impl<T, S> Quasi<T, S> {
    /// Unwraps the container, yielding the underlying value.
    ///
    /// # Panics
    /// Panics if it's a `Substitute` with a custom panic message provided by `msg`.
    pub fn expect(self, msg: &str) -> T {
        match self {
            Quasi::True(v) => v,
            Quasi::Substitute(_) => panic!("{}", msg),
        }
    }

    /// Get an immutable reference to the underlying type.
    ///
    /// # Panics
    /// Panics if it's a `Substitute`.
    pub fn get(&self) -> &T {
        match *self {
            Quasi::True(ref v) => v,
            Quasi::Substitute(_) => panic!("Quasi::get on a substitute"),
        }
    }

    /// Get a mutable reference to the underlying type.
    ///
    /// # Panics
    /// Panics if it's a `Substitute`.
    pub fn get_mut(&mut self) -> &mut T {
        match *self {
            Quasi::True(ref mut v) => v,
            Quasi::Substitute(_) => panic!("Quasi::get_mut on a substitute"),
        }
    }

    /// Moves the underyling value out of the `Quasi` container.
    ///
    /// # Panics
    /// Panics if it's a `Substitute`.
    pub fn unwrap(self) -> T {
        match self {
            Quasi::True(v) => v,
            Quasi::Substitute(_) => panic!("Quasi::unwrap on a substitute"),
        }
    }
}

// Safe accessors.
impl<T, S> Quasi<T, S> {
    /// Returns an optional reference to the underlying value.
    pub fn as_ref(&self) -> Option<&T> {
        match *self {
            Quasi::True(ref v) => Some(v),
            Quasi::Substitute(_) => None,
        }
    }

    /// Returns an optional mutable reference to the underlying value.
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match *self {
            Quasi::True(ref mut v) => Some(v),
            Quasi::Substitute(_) => None,
        }
    }

    /// Returns an iterator over the one underlying value.
    pub fn iter<'q>(&'q self) -> Box<Iterator<Item=&'q T> + 'q> {
        match *self {
            Quasi::True(ref v) => Box::new(iter::once(v)),
            Quasi::Substitute(_) => Box::new(iter::empty()),
        }
    }

    /// Returns a mutable iterator over the one underlying value.
    pub fn iter_mut<'q>(&'q mut self) -> Box<Iterator<Item=&'q mut T> + 'q> {
        match *self {
            Quasi::True(ref mut v) => Box::new(iter::once(v)),
            Quasi::Substitute(_) => Box::new(iter::empty()),
        }
    }

    /// Returns the underlying value or a default.
    pub fn unwrap_or(self, def: T) -> T {
        match self {
            Quasi::True(v) => v,
            Quasi::Substitute(_) => def,
        }
    }

    /// Returns the underlying value or computes one from a closure.
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Quasi::True(v) => v,
            Quasi::Substitute(_) => f(),
        }
    }
}
impl<T: Default, S> Quasi<T, S> {
    /// Returns the underlying value or a default.
    pub fn unwrap_or_default(self) -> T {
        match self {
            Quasi::True(v) => v,
            Quasi::Substitute(_) => Default::default(),
        }
    }
}

// Combinators.
impl<T, S> Quasi<T, S> {
    /// Maps a `Quasi<T>` to `Quasi<U>` by applying a function to the underlying value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Quasi<U, S> {
        match self {
            Quasi::True(v) => Quasi::True(f(v)),
            Quasi::Substitute(v) => Quasi::Substitute(v),
        }
    }
}

// Conditional, pass-through trait implementations.
impl<T, S> Clone for Quasi<T, S>
    where T: Clone, S: Clone
{
    fn clone(&self) -> Self {
        match *self {
            Quasi::True(ref v) => Quasi::True(v.clone()),
            Quasi::Substitute(ref v) => Quasi::Substitute(v.clone()),
        }
    }
}
impl<T, S> PartialEq for Quasi<T, S>
    where T: PartialEq, S: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Quasi::True(ref v1), &Quasi::True(ref v2)) => v1 == v2,
            (&Quasi::Substitute(ref v1), &Quasi::Substitute(ref v2)) => v1 == v2,
            _ => false,
        }
    }
}
impl<T, S> PartialOrd for Quasi<T, S>
    where T: PartialOrd, S: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&Quasi::True(ref v1), &Quasi::True(ref v2)) => v1.partial_cmp(v2),
            (&Quasi::Substitute(ref v1), &Quasi::Substitute(ref v2)) => v1.partial_cmp(v2),
            _ => None,
        }
    }
}
// TODO: Hash, Clone, etc.

// Conversions into external types.
impl<T, S> Into<Option<T>> for Quasi<T, S> {
    fn into(self) -> Option<T> {
        match self {
            Quasi::True(v) => Some(v),
            Quasi::Substitute(_) => None,
        }
    }
}
impl<T, S> Into<Result<T, S>> for Quasi<T, S> {
    fn into(self) -> Result<T, S> {
        match self {
            Quasi::True(v) => Ok(v),
            Quasi::Substitute(v) => Err(v),
        }
    }
}

impl<T, S> fmt::Debug for Quasi<T, S>
    where T: fmt::Debug, S: fmt::Debug
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Quasi::True(ref v) => write!(fmt, "Quasi::True({:?})", v),
            Quasi::Substitute(ref v) => write!(fmt, "Quasi::Substitute({:?})", v),
        }
    }
}

impl<T, S> fmt::Display for Quasi<T, S>
    where T: fmt::Display, S: fmt::Display
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Since the wrapper is supposed to be transparent, this is a passthrough.
        write!(fmt, "{}", match *self {
            Quasi::True(ref v) => v as &fmt::Display,
            Quasi::Substitute(ref v) => v as &fmt::Display,
        })
    }
}
