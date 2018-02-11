//! Wrapper over `serde_json::Value` with some additional trait implementations.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use serde_json::{self, Value};


macro_attr! {
    /// Wrapper over `serde_json::Value`
    /// that allows it to be used as the second parameter to the `Quasi` type.
    ///
    /// This basically implements all the deserialization and serialization traits
    /// as no-op passthroughs.
    #[derive(Clone, PartialEq,
             NewtypeDisplay!, NewtypeFrom!)]
    pub struct Json(Value);
}

impl Deref for Json {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Json {
    /// Construct `Json` object from `serde_json::Value`.
    #[inline]
    pub fn new(value: Value) -> Self {
        Json(value)
    }

    /// Construct `Json` object from string.
    #[inline]
    pub fn new_string(s: String) -> Self {
        Json(Value::String(s))
    }
    // TODO: other obvious constructors
}

impl From<String> for Json {
    fn from(s: String) -> Self {
        Json::new_string(s)
    }
}
impl<'s> From<&'s str> for Json {
    fn from(s: &'s str) -> Self {
        Json::new_string(s.to_owned())
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Value::deserialize(deserializer).map(Json::new)
    }
}
impl FromStr for Json {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Value::from_str(s).map(Json::new)
    }
}

impl<'de> Deserializer<'de> for Json {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        // The error conversion here is the sole reason why we implement `Deserialize`
        // manually (otherwise we wouldn't get a the `IntoDeserializer` impl
        // that works with the default `serde::de::value::Error`).
        self.0.deserialize_any(visitor)
            .map_err(|e| de::Error::custom(format!("JSON deserialization error: {}", e)))
    }

    // TODO: do the passthrough for all deserialize methods rather than
    // going through deserialize_any()
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> IntoDeserializer<'de> for Json {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}
// TODO: use the Json wrapper in place of NoopIntoDeserializer
// (because we only use the latter for serde_json::Value anyway)

impl Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.0.serialize(serializer)
    }
}

impl fmt::Debug for Json {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.0)
    }
}
