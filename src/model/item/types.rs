//! Item types & categories.


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
