//! Stash tabs.

use std::fmt;
use std::ops::Deref;

use super::item::Item;
use super::label::Label;
use super::league::League;
use super::price::Price;


/// Stash tab returned by the API.
pub struct Stash {
    /// Unique ID of the stash tab.
    pub id: String,
    /// League which the stash tab is in.
    pub league: League,
    /// Label of the stash tab.
    ///
    /// Note that some labels like "~b/o 1 chaos" are special
    /// and indicate the price for every item in the tab.
    /// Those labels are interpreted automatically.
    pub label: Label,
    /// Type of the stash tab.
    pub type_: StashType,
    /// Name of the player account with this stash tab,
    pub account: String,
    /// Name of the last character logged in on the stash's account, if known.
    pub last_character: Option<String>,
    /// Items stored in the stash tab.
    pub items: Vec<StashedItem>,
}

impl fmt::Debug for Stash {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Stash")
            .field("id", &self.id)
            .field("league", &self.league)
            .field("label", &self.label)
            .field("type", &self.type_)
            .field("account", &self.account)
            .field("last_character", &self.last_character)
            .field("items", &self.items)
            .finish()
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
    pub(super) item: Item,
    /// Item label.
    ///
    /// This is usually its price,
    /// especially if different from the stash-wide price.
    pub(super) label: Option<Label>,
    /// Horizontal position in the stash tab.
    pub(super) x: u64,
    /// Vertical position in the stash tab.
    pub(super) y: u64,
    /// Width of the item in stash tab tiles.
    pub(super) width: u64,
    /// Height of the item in stash tab tiles.
    pub(super) height: u64,
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
        self.label().and_then(|l| l.note())
    }

    /// Optional exact price for the item.
    pub fn exact_price(&self) -> Option<Price> {
        self.label().and_then(|l| l.exact_price())
    }

    /// Optional negotiable price for the item.
    pub fn negotiable_price(&self) -> Option<Price> {
        self.label().and_then(|l| l.negotiable_price())
    }

    /// Position of the item in the stash tab.
    #[inline]
    pub fn position(&self) -> (u64, u64) {
        (self.x, self.y)
    }

    /// Size of the item in stash tab tiles.
    #[inline]
    pub fn size(&self) -> (u64, u64) {
        (self.width, self.height)
    }
}
