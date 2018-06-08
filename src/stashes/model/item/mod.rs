//! Items that can be traded.

mod category;
mod details;
mod experience;
mod influence;
mod mods;
mod properties;
mod quality;
mod rarity;
mod sockets;

pub use self::category::*;
pub use self::details::ItemDetails;
pub use self::experience::Experience;
pub use self::influence::Influence;
pub use self::mods::{Mod, ModType};
pub use self::properties::Properties;
pub use self::quality::Quality;
pub use self::rarity::Rarity;
pub use self::sockets::*;


use std::collections::HashMap;
use std::iter;

use serde_json::Value as Json;

use ::common::util::Quasi;



/// A tradeable item in Path of Exile.
///
/// This includes all classes of wearable and usable items
/// with the notable exception of `Currency` items.
#[derive(Debug)]
pub struct Item {
    /// Unique ID this game has associated with the item.
    pub id: String,
    /// Name of the item.
    ///
    /// It may be missing for white items, or generic items like gems.
    pub name: Option<String>,
    /// Item base (type).
    ///
    /// This is something like "Rustic Sash", "Crimson Jewel", or "Sunder".
    pub base: String,
    /// Item level.
    pub level: u64,
    /// Category of the item.
    ///
    /// This indicates what you can do with the item, e.g. the slot it is worn on.
    pub category: Quasi<ItemCategory>,
    /// Item rarity, such as magic or unique.
    ///
    /// For items other than gear, this will be just `Normal`.
    pub rarity: Rarity,
    /// Item quality.
    pub quality: Quality,
    /// Innate item properties.
    ///
    /// These properties are characteristics inherent to a particular item type,
    /// like armor/evasion/energy shield values and weapon damage range.
    /// Properties may have values (like damage ranges)
    /// or not (like gem "tags", e.g. "Spell", "Support", etc).
    pub properties: Properties,  // TODO: parse "X-Y" ranges
    /// Item details.
    ///
    /// These are specific to the particular kind of item.
    /// In most cases, details specify the mods of an item.
    ///
    /// If the item type doesn't define any details (e.g. it's a currency item),
    /// this will be `None`.
    pub details: Option<ItemDetails>,
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

    /// Whether this item has been identified
    /// (or didn't need identification in the first place).
    #[inline]
    pub fn is_identified(&self) -> bool {
        self.details.as_ref().map(|d| d.is_identified()).unwrap_or(true)
    }

    /// Iterate over the item mods, if any.
    #[inline]
    pub fn mods<'i>(&'i self) -> impl Iterator<Item=&'i Mod> {
        self.details.as_ref().map(|d| d.mods()).unwrap_or_else(|| Box::new(iter::empty()))
    }
}
// TODO: implement Display that corresponds to the way items are copied to clipboard
// when you hover over them in game and press Ctrl+C

/// A particular kind of requirement that a character must satisfy
/// in order to use an item.
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
