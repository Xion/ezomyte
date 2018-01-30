//! Qualitty of an item.

use std::fmt;


macro_attr! {
    /// Quality of an item.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd,
             NewtypeFrom!, NewtypeDeref!, NewtypeDerefMut!,
             NewtypeAdd!, NewtypeAddAssign!)]
    pub struct Quality(u8);
}

impl Default for Quality {
    fn default() -> Self {
        Quality(0)
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "+{}%", self.0)
    }
}
