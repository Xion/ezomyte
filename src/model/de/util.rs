//! Deserialization utilities.

use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer, IntoDeserializer};


/// Macro for checking duplicate fields when deserializing from a map or struct.
macro_rules! check_duplicate {
    // E.g.: check_duplicate!("frameType" => rarity)
    ($key:expr => $var:ident) => ({
        if $var.is_some() {
            return Err(::serde::de::Error::duplicate_field($key));
        }
    });
    // E.g.: check_duplicate!(identified)
    ($var:ident) => (
        check_duplicate!(stringify!($var) => $var);
    );
}


/// Deserialize a typed value out of an "intermediate" representation
/// (usually a string or number) that has been deserialized previously.
///
/// This can be used to refine the final type of output after more information
/// is available in more complicated deserialization scenarios.
pub fn deserialize<'de, T, S, E>(from: S) -> Result<T, E>
    where T: Deserialize<'de>,
          S: IntoDeserializer<'de, E>,
          E: de::Error
{
    let deserializer = IntoDeserializer::into_deserializer(from);
    T::deserialize(deserializer)
}


/// Newtype wrapper for types implementing `Deserializer` trait
/// which automatically provides them with a no-op `IntoDeserializer` impl.
///
/// This is needed for the serde_json::Value type which implements `Deserializer`
/// but NOT `IntoDeserializer`. As a consequence, things like HashMap<String, Value>
/// cannot be passed to deserialize() function above without this wrapper.
pub struct NoopIntoDeserializer<'de, D: Deserializer<'de>> {
    deserializer: D,
    _marker: PhantomData<&'de ()>,
}
// TODO: file an issue against serde_json to have this noop IntoDeserializer impl
// available by default w/o a newtype wrapper

impl<'de, D: Deserializer<'de>> NoopIntoDeserializer<'de, D> {
    #[inline(always)]
    pub fn new(deserializer: D) -> Self {
        NoopIntoDeserializer {
            deserializer,
            _marker: PhantomData,
        }
    }
}

impl<'de, D> IntoDeserializer<'de, D::Error> for NoopIntoDeserializer<'de, D>
    where D: Deserializer<'de>
{
    type Deserializer = D;
    fn into_deserializer(self) -> Self::Deserializer {
        self.deserializer
    }
}
