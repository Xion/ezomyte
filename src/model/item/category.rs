//! Item categories.


/// Category of an item.
/// This roughly describes the item's purpose, such as the slot it can be worn on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ItemCategory {
    Accessory(AccessoryType),
    Armour(ArmourType),
    Weapon(WeaponType),
    Jewel(JewelType),
    Flask,
    Map,
    Gem,
    DivinationCard,
    Currency,
    // TODO: try to eliminate this catch-all variant
    // (it may be good to have it though, for forward-compatibility with future leagues
    //  so that clients have at least limited support for possible new item categories)
    Other(String),
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

/// Type of a jewel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JewelType {
    /// Regular jewel
    /// Those jewels  can only be placed in a passive tree slot.
    Regular,
    /// Abyss jewel.
    /// These can be placed both in passive tree slots and abyss sockets in gear.
    Abyss
}
