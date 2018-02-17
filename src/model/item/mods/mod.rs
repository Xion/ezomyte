//! Item mod (modifier).

mod id;
mod database;

pub use self::id::{ModId, ModType};


use std::fmt;


macro_attr! {
    /// Item mod.
    /// For now this is just verbatim text of the mod.
    #[derive(Clone, Deserialize,
             NewtypeFrom!, NewtypeDeref!, NewtypeDerefMut!,
             NewtypeDisplay!)]
    pub struct Mod(String);
}

impl Mod {
    /// Mod text as string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for Mod {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Mod({:?})", self.0)
    }
}


/// Type of a mod parameter value
/// -- that is, the number that varies between occurrences of a mod on items.
pub type ModValue = f64;
