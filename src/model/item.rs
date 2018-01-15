//! Items that can be traded.

use std::collections::HashMap;
use std::fmt;

use serde_json::Value as Json;


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

}
impl Experience {
    /// Whether the experience "bar" is fully filled.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.current == self.total
    }
}



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

/// Details of the particular items, if known.
#[derive(Debug)]
pub enum ItemDetails {
    /// An unidentified item. No details available.
    Unidentified,
    /// Skill gem.
    Gem {
        /// Current level of the gem.
        ///
        /// Gems start at level 1.
        /// Standalone gems (not in gear) currently shouldn't have levels above 21
        /// (at least outside of glitches/bugs).
        level: u32,
        /// The amount of experience a gem has and requires for the next level.
        experience: Experience,
    },
    /// Flask item.
    Flask {
        /// Utility mods of the flask.
        ///
        /// These are typically the on-use effect (like bleed/freeze/etc. removal),
        /// mods affecting charges, etc..
        mods: Vec<Mod>,
    },
    /// Item with mods.
    ///
    /// This includes most items, such as those in gear slots,
    /// jewels, maps, and so on.
    Mods {
        /// Implicit mods an item has (those above a horizontal line in the UI).
        ///
        /// Currently, PoE only supports a single implicit mod,
        /// but this may change in the future.
        implicit: Vec<Mod>,
        // TODO: there are enchantMods in the API, should we merge them here?

        /// Explicit mods of an item.
        ///
        /// Note that these the mods which are visible in the UI,
        /// as opposed to *affixes* (prefixes & suffixes) which cannot be reliably
        /// extrapolated from mods.
        /// The practical consequence is that there may be more than 6 mods
        /// which would seemingly contradict the "3 prefixes + 3 suffixes" rule
        /// (due to the so-called hybrid affixes that result in multiple mods).
        explicit: Vec<Mod>,
        // TODO: there are craftedMods in the API, should we merge them here?
    },
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

/// Category of an item.
/// This roughly describes the item's purpose, such as the slot it can be worn on.
#[derive(Debug)]
pub enum ItemCategory {
    Accessory(AccessoryType),
    Armour(ArmourType),
    Weapon(WeaponType),
    Jewel,
}

/// Type of an accessory item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessoryType {
    /// Amulet (necklace).
    Amulet,
    /// Belt.
    Belt,
    /// Ring.
    Ring,
}

/// Type of an armor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArmourType {
    /// Head slot item.
    Helmet,
    /// Hand slot item.
    Gloves,
    /// Body armor slot item.
    Chest,
    /// Feet slot item.
    Boots,
    /// A shield.
    Shield,
    /// A quiver.
    Quiver,
}
// TODO: consider introducing a Slot enum (to lump together quivers and shields)

/// Type of a weapon.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WeaponType {
    /// A bow.
    Bow,
    /// A claw.
    Claw,
    /// A dagger.
    Dagger,
    /// One-handed axe.
    OneHandedAxe,
    /// One-handed mace,
    OneHandedMace,
    /// One-handed sword.
    OneHandedSword,
    /// Scepter (one-handed intelligence-based mace).
    Sceptre,
    /// Staff,
    Staff,
    /// Two-handed axe.
    TwoHandedAxe,
    /// Two-handed mace.
    TwoHandedMace,
    /// Two-handed sword.
    TwoHandedSword,
    /// Wand.
    Wand,
}
// TODO: consider introducing weapon "kind" (mace/staff/sword/axe) that lumps 1H & 2H together;
// bear in mind that "handedness" is ambiguous for bows, though


/// Sockets an item has, if any.
#[derive(Debug)]
pub struct ItemSockets {
    /// Number of abyss sockets the item has.
    abyss_count: u64,
    /// Groups of regular sockets that are linked together.
    regular_groups: Vec<SocketGroup>,
}

impl ItemSockets {
    /// Number of abyss sockets this item has.
    #[inline]
    pub fn abyss_count(&self) -> u64 {
        self.abyss_count
    }

    /// Maximum number of linked sockets on the item.
    ///
    /// If an item is said to be N-linked (e.g. 5-linked), this will be N.
    #[inline]
    pub fn max_links(&self) -> u64 {
        unimplemented!()
    }
}

/// A group of linked sockets on an item.
#[derive(Debug)]
pub struct SocketGroup {
    /// ID of the socket group, assigned by the API.
    ///
    /// This is a small integer index starting from 0.
    id: u8,
    /// Colors of linked sockets.
    colors: Vec<Color>,
}

/// A color of an item or socket.
///
/// In PoE, this is associated with a particular main stat.
///
/// *Note*: Although it does appear as such in the API,
/// "abyss" is not a color so it's not included here.
/// See `ItemSockets::abyss_count` for the number of abyss sockets an item has.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    /// Red gem or socket, associated with Strength.
    Red,
    /// Green gem or socket, associated with Dexterity.
    Green,
    /// Blue gem or socket, associated with Intelligence.
    Blue,
    /// White gem or socket (not associated with any stat)
    White,
}
