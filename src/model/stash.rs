//! Stash tabs.

use super::currency::Currency;
use super::item::Item;


/// Stash tab returned by the API.
#[derive(Debug)]
pub struct Stash {
    /// Unique ID of the stash tab.
    pub id: String,
    /// Name of the stash tab.
    /// Note that some names like "~b/o 1 chaos" are special
    /// and indicate the price for every item in the tab.
    pub name: StashName,
    /// Type of the stash tab.
    pub type_: StashType,
    /// Name of the player account with this stash tab,
    pub account: String,
    /// Name of the last character logged in on the stash's account, if known.
    pub last_character: Option<String>,
    /// Items stored in the stash tab.
    pub items: Vec<Item>,

}

/// Stash tab name.
///
/// Note that those names can have special meaning in PoE
/// and indicate e.g. prices applicable to all items in the tab.
#[derive(Debug)]
pub enum StashName {
    /// Cosmetic name, without any other meaning.
    Cosmetic(String),
    /// Exact price ("~price $N $CURR").
    ExactPrice(usize, Currency),
    /// Negotiable price ("~b/o $N $CURR").
    NegotiablePrice(usize, Currency),
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
