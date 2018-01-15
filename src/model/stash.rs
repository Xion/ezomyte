//! Stash tabs.

use std::cmp::Ordering;
use std::ops::Deref;

use super::currency::Currency;
use super::item::Item;


/// Stash tab returned by the API.
#[derive(Debug)]
pub struct Stash {
    /// Unique ID of the stash tab.
    pub id: String,
    /// League which the stash tab is in.
    pub league: League,
    /// Name of the stash tab.
    ///
    /// Note that some names like "~b/o 1 chaos" are special
    /// and indicate the price for every item in the tab.
    /// Those names are interpreted automatically.
    pub name: Label,
    /// Type of the stash tab.
    pub type_: StashType,
    /// Name of the player account with this stash tab,
    pub account: String,
    /// Name of the last character logged in on the stash's account, if known.
    pub last_character: Option<String>,
    /// Items stored in the stash tab.
    pub items: Vec<StashedItem>,

}


/// League of a stash tab.
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


/// Label (note) for an item or stash tab.
///
/// Those labels can have special meaning in PoE
/// and indicate e.g. prices applicable to all items in the tab.
#[derive(Debug)]
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


/// Type of the stash tab returned by the API.
#[derive(Debug, Deserialize)]
pub enum StashType {
    /// Regular stash tan.
    #[serde(rename = "NormalStash")]
    Normal,
    /// Premium stash tab (of regular size).
    #[serde(rename = "PremiumStash")]
    Premium,
    /// Premium Quad stash tab.
    #[serde(rename = "QuadStash")]
    Quad,
    /// Essence stash tab.
    #[serde(rename = "EssenceStash")]
    Essence,
    /// Currency stash tab.
    #[serde(rename = "CurrencyStash")]
    Currency,
    /// Divination cards stash tab.
    #[serde(rename = "DivinationCardStash")]
    Divination,
    /// Map stash tab.
    #[serde(rename = "MapStash")]  // TODO: verify
    Map,
}


/// Item placed in a stash tab.
#[derive(Debug)]
pub struct StashedItem {
    /// The item in question.
    item: Item,
    /// Item label (usually the price).
    label: Option<Label>,
    /// Horizontal position in the stash tab.
    x: u64,
    /// Vertical position in the stash tab.
    y: u64,
    /// Width of the item in stash tab tiles.
    width: u64,
    /// Height of the item in stash tab tiles.
    height: u64,
}

impl Deref for StashedItem {
    type Target = Item;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl StashedItem {
    /// The item in question.
    ///
    /// *Note*: `StashedItem` can automatically `Deref` to the `Item` type,
    /// so you can also use the `Item` interface directly.
    #[inline]
    pub fn item(&self) -> &Item { &self.item }

    /// Label attached to the item, if any.
    #[inline]
    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    /// Optional cosmetic note attached to the item.
    pub fn note(&self) -> Option<&str> {
        self.label().and_then(|l| match l {
            &Label::Cosmetic(ref s) => Some(s.as_str()),
            _ => None,
        })
    }

    /// Optional exact price for the item.
    pub fn exact_price(&self) -> Option<&Price> {
        self.label().and_then(|l| match l {
            &Label::ExactPrice(ref p) => Some(p),
            _ => None
        })
    }

    /// Optional negotiable price for the item.
    pub fn negotiable_price(&self) -> Option<&Price> {
        self.label().and_then(|l| match l {
            &Label::NegotiablePrice(ref p) => Some(p),
            _ => None
        })
    }

    /// Position of the item in the stash tab.
    #[inline]
    pub fn position(&self) -> (u64, u64) {
        (self.x, self.y)
    }
}
