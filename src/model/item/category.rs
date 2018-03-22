//! Item categories.


/// Category of an item.
///
/// This roughly describes the item's purpose, such as the slot it can be worn on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemCategory {
    /// Accessory item (also referred to as jewelry).
    Accessory(AccessoryType),
    /// Armour item.
    Armour(ArmourType),
    /// Weapon item.
    Weapon(WeaponType),
    /// Jewel item.
    ///
    /// This includes jewels socketed into the passive tree,
    /// as well as abyss jewels that can additionally go into abyssal sockets.
    Jewel(JewelType),
    /// A flask.
    Flask,
    /// A map item.
    Map,
    /// Skill gem.
    Gem,
    /// Divination card.
    DivinationCard,
    /// Sealed prophecy item.
    Prophecy,
    /// Relic item.
    Relic,
    /// Currency item.
    Currency,  // NOTE: Keep it as last variant.
}

impl ItemCategory {
    /// Whether items of this category are normally used & consumed by players
    /// rather than being worn.
    ///
    /// Vendor recipes do not count as "use" in this meaning.
    pub fn is_consumable(&self) -> bool {
        match *self {
            ItemCategory::Map |
            ItemCategory::DivinationCard |
            ItemCategory::Currency => true, // TODO: prophecy
            _ => false,
        }
    }

    /// Whether items of this category can be worn on the character.
    ///
    /// Note that this also includes flasks, jewels, and gems,
    /// as they are put into equipment slots either directly or indirectly.
    pub fn is_equippable(&self) -> bool {
        match *self {
            ItemCategory::Accessory(..) |
            ItemCategory::Armour(..) |
            ItemCategory::Weapon(..) |
            ItemCategory::Jewel(..) |
            ItemCategory::Flask |
            ItemCategory::Gem => true,
            _ => false,
        }
    }

    /// Whether items of this category can be modified using currency items.
    pub fn is_modifiable(&self) -> bool {
        match *self {
            ItemCategory::Accessory(..) |
            ItemCategory::Armour(..) |
            ItemCategory::Weapon(..) |
            ItemCategory::Jewel(..) |
            ItemCategory::Flask |
            ItemCategory::Map |
            ItemCategory::Gem => true,
            _ => false,
        }
    }
}


/// Type of an accessory item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessoryType {
    /// Amulet (necklace).
    ///
    /// This also includes talismans from the Talisman league.
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

/// Type of a jewel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JewelType {
    /// Regular jewel.
    ///
    /// Those jewels  can only be placed in a passive tree slot.
    Regular,
    /// Abyss jewel.
    ///
    /// These can be placed both in passive tree slots and abyss sockets in gear.
    Abyss
}
