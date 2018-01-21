//! Deserialization utilities.

use serde::de::{self, Deserialize, IntoDeserializer};


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
