//! Deserializer for item categories.

use std::fmt;

use itertools::Itertools;
use serde::de::{self, Deserialize, IntoDeserializer, Visitor, Unexpected};

use super::super::{AccessoryType, ArmourType, ItemCategory, JewelType, WeaponType};


const EXPECTING_MSG: &str = "map with item data";

// Note that "jewels" can be either a standalone string or a map key.
// The former denotes a regular jewel while the latter should be "jewels": ["abyss"]
// and describe abyss jewels.
const FIELDS: &[&str] = &["accessories", "armour", "jewels", "weapons"];
const OTHER_CATEGORIES: &str = "jewels/flasks/maps/gems";


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
        if let Some(key) = map.next_key::<String>()? {
            let subcats: Vec<String> = map.next_value()?;
            match key.trim().to_lowercase().as_str() {
                "accessories" => match subcats.get(0).map(|sc| sc.as_str()) {
                    Some("amulet") => Ok(ItemCategory::Accessory(AccessoryType::Amulet)),
                    Some("belt") => Ok(ItemCategory::Accessory(AccessoryType::Belt)),
                    Some("ring") => Ok(ItemCategory::Accessory(AccessoryType::Ring)),
                    sc => Err(de::Error::custom(format!("unexpected accessory type: {:?}", sc))),
                },
                // TODO: rest
                _ => Err(de::Error::unknown_field(&key, FIELDS)),
            }
        } else {
            Err(de::Error::custom(format!(
                "empty category map, expected one of {}", FIELDS.iter().format("/"))))
        }
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match v.trim().to_lowercase().as_str() {
            "jewels" => Ok(ItemCategory::Jewel(JewelType::Regular)),
            "flasks" => Ok(ItemCategory::Flask),
            "maps" => Ok(ItemCategory::Map),
            "gems" => Ok(ItemCategory::Gem),
            // TODO: divination cards
            _ => Err(de::Error::invalid_value(Unexpected::Str(v), &OTHER_CATEGORIES)),
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use model::ItemCategory;
    use super::OTHER_CATEGORIES;

    #[test]
    fn string_categories_covered() {
        for other_cat in OTHER_CATEGORIES.split("/") {
            serde_json::from_str::<ItemCategory>(&format!(r#""{}""#, other_cat)).unwrap();
        }
    }
}
