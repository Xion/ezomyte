//! Stash tabs.

use std::ops::Deref;

use super::basics::{Label, League, Price};
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
