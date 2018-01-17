//! Item attributes in the broadest sense
//! (mods, quality, elder/shaper base, gem experience, etc.).

use std::fmt;
use std::ops::{Add, AddAssign};

use separator::Separatable;


macro_attr! {
    /// Rarity of an item.
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq,
             IterVariants!(Rarities))]
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
}

impl Default for Rarity {
    fn default() -> Self {
        Rarity::Normal
    }
}


macro_attr! {
    /// Influence of the Atlas on the item.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq,
             IterVariants!(Influences))]
    pub enum Influence {
        /// The item base has been influenced by the Elder.
        /// This is colloquially referred to as an "elder item".
        Elder,
        /// This item base has been influenced by the Shaper.
        /// This is colloquially referred to as a "shaped item".
        Shaper,
    }
}


macro_attr! {
    /// Item mod.
    /// For now this is just verbatim text of the mod.
    #[derive(Clone, Debug,
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

/// Experience gained by a gem.
#[derive(Clone, Copy)]
pub struct Experience {
    /// Experience for current level earned so far.
    /// This is always lower than total.
    current: u64,
    /// Total experience before the next level.
    total: u64,
}

impl Experience {
    /// Create an `Experience` object which represents a 0/total state.
    #[inline]
    pub fn zero_out_of(total: u64) -> Self {
        assert!(total > 0, "Total experience cannot be zero");
        Experience { current: 0, total }
    }

    /// Create an `Experience` object which represents
    /// a filled experience bar with given total.
    ///
    /// Such state correspond to a skill gem that is deliberately kept at a lower level.
    #[inline]
    pub fn full(total: u64) -> Self {
        assert!(total > 0, "Total experience cannot be zero");
        Experience { current: total, total }
    }
}

impl Experience {
    /// Experience for current level earned so far.
    /// This is always lower than total.
    #[inline]
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Total experience before the next level.
    #[inline]
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Whether the experience "bar" is completely empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current == 0
    }

    /// Whether the experience "bar" is fully filled.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.current == self.total
    }

    /// Experience as a 0..=1 fraction.
    pub fn fraction(&self) -> f64 {
        self.current as f64 / self.total as f64
    }

    /// Experience as a percentage value.
    pub fn percentage(&self) -> f64 {
        self.fraction() * 100.0
    }
}

impl Add<u64> for Experience {
    type Output = Experience;
    fn add(mut self, rhs: u64) -> Self {
        self += rhs; self
    }
}
impl AddAssign<u64> for Experience {
    fn add_assign(&mut self, rhs: u64) {
        let new = self.total.min(self.current + rhs);
        self.current = new;
    }
}

impl fmt::Display for Experience {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}/{}", self.current.separated_string(), self.total.separated_string())
    }
}

impl fmt::Debug for Experience {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Experience")
            .field(&self.current)
            .field(&self.total)
            .finish()
    }
}
