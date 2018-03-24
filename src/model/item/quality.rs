//! Qualitty of an item.

use std::fmt;


// TODO: this could be also used for other percentage values/bonuses,
// such as item quality/rarity & monster pack bonuses on maps


macro_attr! {
    /// Quality of an item as a numeric percentage value.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd,
             NewtypeFrom!, NewtypeDeref!, NewtypeDerefMut!,
             NewtypeAdd!, NewtypeAddAssign!)]
    pub struct Quality(pub u8);
    // TODO: we don't really want the inner u8 to be `pub`, but `pub(crate)`
    // causes syntax issues with macro_attr/newtype_derive;
    // switch to proc_macro-based newtype derivation, or just impl all the above stuff manually
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
