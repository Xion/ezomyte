//! Item attributes in the broadest sense
//! (mods, quality, elder/shaper base, gem experience, etc.).

use std::fmt;


/// Rarity of an item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rarity {
    /// Normal ("white") item.
    ///
    /// This also includes items which aren't gear, like gems or divination cards.
    Normal,
    /// Magic ("blue") item.
    Magic,
    /// Rare ("yellow") item.
    Rare,
    /// Unique item.
    Unique,
}


/// Influence of the Atlas on the item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Influence {
    /// The item base has been influenced by the Elder.
    /// This is colloquially referred to as an "elder item".
    Elder,
    /// This item base has been influenced by the Shaper.
    /// This is colloquially referred to as a "shaped item".
    Shaper,
}


macro_attr! {
    /// Item mod.
    /// For now this is just verbatim text of the mod.
    #[derive(Clone, Debug,
             NewtypeFrom!, NewtypeDeref!, NewtypeDerefMut!,
             NewtypeDisplay!)]
    pub struct Mod(String);
}


macro_attr! {
    /// Quality of an item.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd,
             NewtypeFrom!, NewtypeDeref!, NewtypeDerefMut!,
             NewtypeAdd!, NewtypeAddAssign!)]
    pub struct Quality(u8);
}

impl fmt::Display for Quality {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "+{}%", self.0)
    }
}

/// Experience gained by a gem.
#[derive(Debug)]
pub struct Experience {
    /// Experience for current level earned so far.
    /// This is always lower than total but greater than 1.
    current: u64,
    /// Total experience before the next level.
    total: u64,
}

impl Experience {
    /// Create an `Experience` object which represents a 0/total state.
    #[inline]
    pub fn zero_out_of(total: u64) -> Self {
        Experience { current: 0, total }
    }

    /// Create an `Experience` object which represents
    /// a filled experience bar with given total.
    ///
    /// Such state correspond to a skill gem that is deliberately kept at a lower level.
    #[inline]
    pub fn full(total: u64) -> Self {
        Experience { current: total, total }
    }
}

impl Experience {
    /// Whether the experience "bar" is fully filled.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.current == self.total
    }
}
