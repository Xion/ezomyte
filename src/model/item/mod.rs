//! Items that can be traded.

mod attributes;
mod details;
mod sockets;
mod types;


use std::collections::HashMap;

use serde_json::Value as Json;

use self::attributes::*;
use self::details::ItemDetails;
use self::sockets::ItemSockets;
use self::types::ItemCategory;



/// A tradeable item in Path of Exile.
///
/// This includes all classes of wearable and usable items
/// with the notable exception of `Currency` items.
#[derive(Debug)]
pub struct Item {
    /// Unique ID this game has associated with the item.
    pub id: Option<String>,
    /// Name of the item.
    ///
    /// It may be missing for white items, or generic items like gems.
    pub name: Option<String>,
    /// Item base (type).
    /// This is something like "Rustic Sash", "Crimson Jewel", or "Sunder".
    pub base: String,
    /// Item level.
    pub level: u64,
    /// Category of the item.
    /// This indicates what you can do with the item, e.g. the slot it is worn on.
    pub category: ItemCategory,
    /// Item rarity, such as magic or unique.
    ///
    /// For items other than gear, this will be just `Normal`.
    pub rarity: Rarity,
    /// Item quality.
    pub quality: Quality,
    /// Item details.
    ///
    /// These are specific to the particular kind of item.
    /// In most cases, the details specify the mods of an item.
    pub details: ItemDetails,
    /// Sockets an item has, if any.
    pub sockets: ItemSockets,  // TODO: socketedItems
    /// Extra item attributes that do not fit into any other part of the schema.
    ///
    /// This may include specific attributes of certain discontinued item types,
    /// like talismans. In any case, the attributes are preserved verbatim
    /// from the JSON.
    pub extra: HashMap<String, Json>,
    /// Requirements for wearing or using the item.
    pub requirements: HashMap<Requirement, u32>,
    /// Whether the item is corrupted.
    pub corrupted: bool,
    /// Whether the item has been influenced by the War of the Atlas.
    pub influence: Option<Influence>,
    /// Whether the item has been duplicated (mirrored).
    pub duplicated: bool,
    /// Flavor text associated with the item.
    pub flavour_text: Option<String>,
}

impl Item {
    /// Whether this item has an Elder-influenced base.
    #[inline]
    pub fn is_elder(&self) -> bool {
        self.influence == Some(Influence::Elder)
    }

    /// Whether this item has a Shaper-influenced base.
    #[inline]
    pub fn is_shaped(&self) -> bool {
        self.influence == Some(Influence::Shaper)
    }

    /// Whether this is a unique item.
    #[inline]
    pub fn is_unique(&self) -> bool {
        self.rarity == Rarity::Unique
    }
}

/// A particular kind of requirement that a character must satisfy to use an item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Requirement {
    /// Level requirement.
    Level,
    /// Strength stat requirement.
    Strength,
    /// Dexterity stat requirement.
    Dexterity,
    /// Intelligence stat requirement.
    Intelligence,
}
