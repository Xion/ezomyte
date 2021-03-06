//! Deserializer for a single stashed item.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserialize, Visitor};

use ::common::util::Json;
use super::super::StashedItem;
use super::util::deserialize;


const EXPECTING_MSG: &str = "map with stashed item data";


impl<'de> Deserialize<'de> for StashedItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_map(StashedItemVisitor)
    }
}

struct StashedItemVisitor;
impl<'de> Visitor<'de> for StashedItemVisitor {
    type Value = StashedItem;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>
    {
        let mut label = None;
        let mut x = None;
        let mut y = None;
        let mut width = None;
        let mut height = None;

        let mut item = HashMap::new();
        while let Some(key) = map.next_key::<String>()? {
            let key = key.trim();
            match key {
                "note" => {
                    check_duplicate!("note" => label);
                    label = Some(Some(map.next_value()?));
                }
                "x" => {
                    check_duplicate!(x);
                    x = Some(map.next_value()?);
                }
                "y" => {
                    check_duplicate!(y);
                    y = Some(map.next_value()?);
                }
                "w" => {
                    check_duplicate!("w" => width);
                    width = Some(map.next_value()?);
                }
                "h" => {
                    check_duplicate!("h" => height);
                    height = Some(map.next_value()?);
                }
                "inventoryId" => { map.next_value::<String>()?; }  // ignore
                key => {
                    // Everything else we're passing through to the Item deserializer.
                    let value: Json = map.next_value()?;
                    item.insert(key.to_owned(), value);
                }
            }
        }

        let item = deserialize(item).map_err(|e| {
            de::Error::custom(format!("cannot deserialize stashed item: {}", e))
        })?;

        let label = label.unwrap_or_default();
        let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
        let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
        let width = width.ok_or_else(|| de::Error::missing_field("w"))?;
        let height = height.ok_or_else(|| de::Error::missing_field("h"))?;

        Ok(StashedItem{item, label, x, y, width, height})
    }
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use ::stashes::StashedItem;

    #[test]
    fn minimal() {
        let item_spec = json!({
            "id": "123abc",
            "name": "",
            "typeLine": "Example Amazing Item of Testing",
            "ilvl": 80,
            "category": "jewels",
            "frameType": 0,
            "x": 12,
            "y": 10,
            "w": 1,
            "h": 1,
        });
        let item: StashedItem = from_value(item_spec).unwrap();
        assert_eq!("123abc", item.id);
        assert_eq!((12, 10), item.position());
        assert_eq!((1, 1), item.size());
    }
}
