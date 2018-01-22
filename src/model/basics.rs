//! Basic data type definitions.

use std::cmp::Ordering;

use super::currency::Currency;


/// Label (note) for an item or stash tab.
///
/// Those labels can have special meaning in PoE
/// and indicate e.g. prices applicable to all items in the tab.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Label {
    /// Cosmetic name, without any other meaning.
    Cosmetic(String),
    /// Exact price ("~price $N $CURR").
    ExactPrice(Price),
    /// Negotiable price ("~b/o $N $CURR").
    NegotiablePrice(Price),
}


/// Price of an item in a particular `Currency`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Price(usize, Currency);

impl Price {
    /// Create a new `Price` object.
    #[inline]
    pub fn new(amount: usize, currency: Currency) -> Self {
        Price(amount, currency)
    }

    /// Create a new `Price` of a single unit of given currency.
    #[inline]
    pub fn one(currency: Currency) -> Self {
        Price(1, currency)
    }
}

impl Price {
    /// Price amount.
    #[inline]
    pub fn amount(&self) -> usize { self.0 }
    /// Currency used in the price.
    #[inline]
    pub fn currency(&self) -> Currency { self.1 }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Price) -> Option<Ordering> {
        if self.currency() == other.currency() {
            Some(self.amount().cmp(&other.amount()))
        } else {
            None
        }
    }
}


/// League in Path of Exile.
///
/// For our purposes, we're only distinguishing permanent & temporary leagues,
/// without making note of a particular temporary league name (like "Harbinger" vs "Abyss").
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct League {
    /// Whether it's a temporary (seasonal) league as opposed to permanent one.
    pub temporary: bool,
    /// Whether it's a hardcore (permadeath) league as opposed to a softcore one.
    pub hardcore: bool,
    /// Whether it's a solo self-found league.
    pub ssf: bool,  // btw
    // TODO: races/void leagues
}

impl Default for League {
    fn default() -> Self {
        League::standard()
    }
}

// Constructors.
impl League {
    /// Standard league (permanent softcore non-SSF).
    #[inline]
    pub fn standard() -> Self {
        League { temporary: false, hardcore: false, ssf: false }
    }

    /// Hardcore league (permanent non-SSF).
    #[inline]
    pub fn hardcore() -> Self {
        League { temporary: false, hardcore: true, ssf: false }
    }

    /// Temporary league (softcore non-SSF).
    #[inline]
    pub fn temporary() -> Self {
        League { temporary: true, hardcore: false, ssf: false }
    }

    /// Temporary hardcore league (non-SSF).
    #[inline]
    pub fn temporary_hardcore() -> Self {
        League { temporary: true, hardcore: true, ssf: false }
    }

    /// SSF league (permanent softcore).
    #[inline]
    pub fn ssf() -> Self {
        League { temporary: false, hardcore: false, ssf: true }
    }

    /// Hardcore SSF league (permanent).
    #[inline]
    pub fn hardcore_ssf() -> Self {
        League { temporary: false, hardcore: true, ssf: true }
    }

    /// Temporary SSF league (softcore).
    #[inline]
    pub fn temporary_ssf() -> Self {
        League { temporary: true, hardcore: false, ssf: true }
    }

    /// Temporary hardcore SSF league.
    #[inline]
    pub fn temporary_hardcore_ssf() -> Self {
        League { temporary: true, hardcore: true, ssf: true }
    }
}
// Constructor aliases.
impl League {
    /// Alias for `standard`.
    #[inline]
    pub fn softcore() -> Self { Self::standard() }
    /// Alias for `standard`.
    #[inline]
    pub fn sc() -> Self { Self::standard() }
    /// Alias for `hardcore`.
    #[inline]
    pub fn hc() -> Self { Self::hardcore() }
    /// Alias for `temporary`.
    #[inline]
    pub fn temp() -> Self { Self::temporary() }
    /// Alias for `temporary`.
    #[inline]
    pub fn temp_sc() -> Self { Self::temporary() }
    /// Alias for `temporary_hardcore`.
    #[inline]
    pub fn temp_hc() -> Self { Self::temporary_hardcore() }
    /// Alias for `hardcore_ssf`.
    #[inline]
    pub fn hc_ssf() -> Self { Self::hardcore_ssf() }
    /// Alias for `temporary_ssf`.
    #[inline]
    pub fn temp_ssf() -> Self { Self::temporary_ssf() }
    /// Alias for `temporary_hardcore_ssf`.
    #[inline]
    pub fn temp_hc_ssf() -> Self { Self::temporary_hardcore_ssf() }
}
