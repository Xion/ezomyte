//! Deserializer for item categories.

use std::fmt;

use itertools::Itertools;
use serde::de::{self, Deserialize, Visitor};

use super::super::{AccessoryType, ArmourType, ItemCategory, JewelType, WeaponType};


const EXPECTING_MSG: &str = "item category as string or 1-element map";

/// Category names as found in the API.
///
/// Note that (almost?) all of these can be either a standalone string or a map key.
/// In case of "jewels", for example, string means a regular jewel while a map
/// should be {"jewels": ["abyss"]} to describe abyss jewels.
///
/// However, it also seems that even categories that should only be represented
/// as string (like "gems") would sometimes have an empty array attached to them
/// (i.e. {"gems": []}), so we gotta be prepared for it.
const CATEGORIES: &[&str] = &[
    "accessories", "armour", "cards", "currency", "flasks", "gems", "jewels",
    "maps", "weapons",
];


impl<'de> Deserialize<'de> for ItemCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_any(ItemCategoryVisitor)
    }
}

struct ItemCategoryVisitor;
impl<'de> Visitor<'de> for ItemCategoryVisitor {
    type Value = ItemCategory;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>
    {
        if let Some(size) = map.size_hint() {
            if size != 1 {
                return Err(de::Error::invalid_length(size, &"exactly one key"));
            }
        }

        if let Some(key) = map.next_key::<String>()? {
            let subcats: Vec<String> = map.next_value()?;
            match key.trim().to_lowercase().as_str() {
                "accessories" => match subcats.get(0).map(|sc| sc.as_str()) {
                    Some("amulet") => Ok(ItemCategory::Accessory(AccessoryType::Amulet)),
                    Some("belt") => Ok(ItemCategory::Accessory(AccessoryType::Belt)),
                    Some("ring") => Ok(ItemCategory::Accessory(AccessoryType::Ring)),
                    sc => Err(de::Error::custom(format!("unexpected accessory type: {:?}", sc))),
                },
                "armour" => match subcats.get(0).map(|sc| sc.as_str()) {
                    Some("helmet") => Ok(ItemCategory::Armour(ArmourType::Helmet)),
                    Some("gloves") => Ok(ItemCategory::Armour(ArmourType::Gloves)),
                    Some("chest") => Ok(ItemCategory::Armour(ArmourType::Chest)),
                    Some("boots") => Ok(ItemCategory::Armour(ArmourType::Boots)),
                    Some("shield") => Ok(ItemCategory::Armour(ArmourType::Shield)),
                    Some("quiver") => Ok(ItemCategory::Armour(ArmourType::Quiver)),
                    sc => Err(de::Error::custom(format!("unexpected armour type: {:?}", sc))),
                },
                "weapons" => match subcats.get(0).map(|sc| sc.as_str()) {
                    Some("bow") => Ok(ItemCategory::Weapon(WeaponType::Bow)),
                    Some("claw") => Ok(ItemCategory::Weapon(WeaponType::Claw)),
                    Some("dagger") => Ok(ItemCategory::Weapon(WeaponType::Dagger)),
                    Some("oneaxe") => Ok(ItemCategory::Weapon(WeaponType::OneHandedAxe)),
                    Some("onemace") => Ok(ItemCategory::Weapon(WeaponType::OneHandedMace)),
                    Some("onesword") => Ok(ItemCategory::Weapon(WeaponType::OneHandedSword)),
                    Some("sceptre") => Ok(ItemCategory::Weapon(WeaponType::Sceptre)),
                    Some("staff") => Ok(ItemCategory::Weapon(WeaponType::Staff)),
                    Some("twoaxe") => Ok(ItemCategory::Weapon(WeaponType::TwoHandedAxe)),
                    Some("twomace") => Ok(ItemCategory::Weapon(WeaponType::TwoHandedMace)),
                    Some("twosword") => Ok(ItemCategory::Weapon(WeaponType::TwoHandedSword)),
                    Some("wand") => Ok(ItemCategory::Weapon(WeaponType::Wand)),
                    sc => Err(de::Error::custom(format!("unexpected weapon type: {:?}", sc))),
                },
                "jewels" => match subcats.get(0).map(|sc| sc.as_str()) {
                    Some("abyss") => Ok(ItemCategory::Jewel(JewelType::Abyss)),
                    None => Ok(ItemCategory::Jewel(JewelType::Regular)),  // "jewels": []
                    sc => Err(de::Error::custom(format!("unexpected jewel type: {:?}", sc))),
                },
                // TODO: consider verifying that these keys map to empty arrays
                // (because they should, right?)
                "cards" => Ok(ItemCategory::DivinationCard),
                "currency" => Ok(ItemCategory::Currency),
                "flasks" => Ok(ItemCategory::Flask),
                "gems" => Ok(ItemCategory::Gem),
                "maps" => Ok(ItemCategory::Map),
                // TODO: consider storing the key as ItemCategory::Other instead,
                // to support potentially complex new item categories introduced in future leagues
                // (but how do we keep the data? add another field to ::Other?)
                _ => Err(de::Error::unknown_field(&key, CATEGORIES)),
            }
        } else {
            Err(de::Error::custom(format!(
                "empty category map, expected one of {}", CATEGORIES.iter().format("/"))))
        }
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match v.trim().to_lowercase().as_str() {
            "jewels" => Ok(ItemCategory::Jewel(JewelType::Regular)),
            "flasks" => Ok(ItemCategory::Flask),
            "maps" => Ok(ItemCategory::Map),
            "gems" => Ok(ItemCategory::Gem),
            "cards" => Ok(ItemCategory::DivinationCard),
            "currency" => Ok(ItemCategory::Currency),
            c => {
                warn!("Unrecognized item category string `{}`, expected one of: {}",
                    c, CATEGORIES.iter().format(", "));
                Ok(ItemCategory::Other(c.to_owned()))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use model::{AccessoryType, ArmourType, ItemCategory, JewelType, WeaponType};

    #[test]
    fn accessories() {
        assert_eq!(
            ItemCategory::Accessory(AccessoryType::Amulet),
            from_value(json!({"accessories": ["amulet"]})).unwrap());
        assert_eq!(
            ItemCategory::Accessory(AccessoryType::Belt),
            from_value(json!({"accessories": ["belt"]})).unwrap());
        assert_eq!(
            ItemCategory::Accessory(AccessoryType::Ring),
            from_value(json!({"accessories": ["ring"]})).unwrap());
    }

    #[test]
    fn armour() {
        assert_eq!(
            ItemCategory::Armour(ArmourType::Helmet),
            from_value(json!({"armour": ["helmet"]})).unwrap());
        assert_eq!(
            ItemCategory::Armour(ArmourType::Gloves),
            from_value(json!({"armour": ["gloves"]})).unwrap());
        assert_eq!(
            ItemCategory::Armour(ArmourType::Chest),
            from_value(json!({"armour": ["chest"]})).unwrap());
        assert_eq!(
            ItemCategory::Armour(ArmourType::Boots),
            from_value(json!({"armour": ["boots"]})).unwrap());
        assert_eq!(
            ItemCategory::Armour(ArmourType::Shield),
            from_value(json!({"armour": ["shield"]})).unwrap());
        assert_eq!(
            ItemCategory::Armour(ArmourType::Quiver),
            from_value(json!({"armour": ["quiver"]})).unwrap());
    }

    #[test]
    fn weapons() {
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Bow),
            from_value(json!({"weapons": ["bow"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Claw),
            from_value(json!({"weapons": ["claw"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Dagger),
            from_value(json!({"weapons": ["dagger"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::OneHandedAxe),
            from_value(json!({"weapons": ["oneaxe"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::OneHandedMace),
            from_value(json!({"weapons": ["onemace"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::OneHandedSword),
            from_value(json!({"weapons": ["onesword"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Sceptre),
            from_value(json!({"weapons": ["sceptre"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Staff),
            from_value(json!({"weapons": ["staff"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::TwoHandedAxe),
            from_value(json!({"weapons": ["twoaxe"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::TwoHandedMace),
            from_value(json!({"weapons": ["twomace"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::TwoHandedSword),
            from_value(json!({"weapons": ["twosword"]})).unwrap());
        assert_eq!(
            ItemCategory::Weapon(WeaponType::Wand),
            from_value(json!({"weapons": ["wand"]})).unwrap());
    }

    #[test]
    fn jewels() {
        assert_eq!(
            ItemCategory::Jewel(JewelType::Regular),
            from_value(json!("jewels")).unwrap());
        assert_eq!(
            ItemCategory::Jewel(JewelType::Abyss),
            from_value(json!({"jewels": ["abyss"]})).unwrap());
    }

    #[test]
    fn superfluous_empty_arrays() {
        // These sometimes appear in actual API response samples.
        assert_eq!(ItemCategory::DivinationCard, from_value(json!({"cards": []})).unwrap());
        assert_eq!(ItemCategory::Currency, from_value(json!({"currency": []})).unwrap());
        assert_eq!(ItemCategory::Gem, from_value(json!({"gems": []})).unwrap());
        assert_eq!(ItemCategory::Map, from_value(json!({"maps": []})).unwrap());
    }
}
