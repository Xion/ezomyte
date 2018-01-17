//! Deserializers for item data.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserilize, IntoDeserializer, Visitor, Unexpected};
use serde_json::Value as Json;

use super::super::{Item, Rarity};


const EXPECTING_MSG: &str = "map with item data";


impl<'de> Deserialize<'de> for Item {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_map(ItemVisitor)
    }
}

struct ItemVisitor;
impl<'de> Visitor<'de> for ItemVisitor {
    type Value = Item;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>
    {
        // TODO: preemptively check the size against minimum & maximum number of fields

        let mut id = None;
        let mut name = None;
        let mut base = None;
        let mut level = None;
        let mut category = None;
        let mut rarity = None;
        let mut quality = None;
        let mut properties = None;
        let mut details = None;
        let mut sockets = None;
        let mut extra = HashMap::new();
        let mut requirements = None;
        let mut corrupted = None;
        let mut influence = None;
        let mut duplicated = None;
        let mut flavour_text = None;

        while let Some(key) = map.next_key::<String>()? {
            let key = key.trim();
            match key {
                "id" => {
                    if id.is_some() {
                        return Err(de::Error::duplicate_field("id"));
                    }
                    id = Some(self.deserialize_nonempty_string(&mut map)?);
                }
                "name" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("name"));
                    }
                    name = Some(self.deserialize_name(&mut map)?);
                }
                // TODO: `base` will be a full name for magic items
                // so it has to be fixed using rarity later on
                "typeLine" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("typeLine"));
                    }
                    base = Some(self.deserialize_nonempty_string(&mut map)?);
                }
                "ilvl" => {
                    if level.is_some() {
                        return Err(de::Error::duplicate_field("ilvl"));
                    }
                    level = Some(map.next_value()?);
                }
                "category" => {
                    // TODO
                }
                "frameType" => {
                    if rarity.is_some() {
                        return Err(de::Error::duplicate_field("frameType"));
                    }
                    // TODO: this will swallow errors such as frameType being
                    // a number rather than simply being out of 0-3 range
                    // (which is fine); fix that, probably by deserializing
                    // in two stages: as a number and then as Rarity enum
                    let value: Rarity = map.next_value().unwrap_or_default();
                    rarity = Some(value);
                }
                "properties" => {
                    if properties.is_some() {
                        return Err(de::Error::duplicate_field("properties"));
                    }
                    let props = self.deserialize_item_properties(&mut map)?;

                    // Pluck out some of the properties that we are providing
                    // as separate fields on `Item`.
                    if let Some(q) = props.remove("Quality") {
                        quality = Some(deserialize(q)?);
                    }

                    properties = Some(props);
                }

                // TODO: many many others
                // (sockets, identified, corrupted, requirements, flavour_text,
                //  elder/shaped, etc.)
            }
        }

        let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
        let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
        let base = base.ok_or_else(|| de::Error::missing_field("typeLine"))?;
        let level = level.ok_or_else(|| de::Error::missing_field("ilvl"))?;
        let category = category.ok_or_else(|| de::Error::missing_field("category"))?;
        let rarity = rarity.ok_or_else(|| de::Error::missing_field("frameType"))?;
        let quality = quality.unwrap_or_default();
        let properties = properties.unwrap_or_default();
        let details = details.ok_or_else(|| de::Error::custom("insufficient item information"))?;
        let sockets = sockets.unwrap_or_default();
        let requirements = requirements.unwrap_or_default();
        let corrupted = corrupted.unwrap_or(false);
        let influence = influence.unwrap_or(None);
        let duplicated = duplicated.unwrap_or(false);
        let flavour_text = flavour_text.unwrap_or(None);

        // TODO: fix `base` using `rarity` when it's a magic item
        // (remove prefix & suffix)

        Item {
            id, name, base, level, catregory, rarity, quality, properties, details,
            sockets, requirements, corrupted, influence, duplicated, flavour_text,
        }
    }
}
impl ItemVisitor {
    /// Deserialize item name into an optional string
    /// by turning an empty one into None.
    /// (Items such as gems do not have a separate name).
    fn deserialize_name<V>(&self, map: &mut V) -> Result<Option<String>, V::Error>
        where V: de::MapAccess<'de>
    {
        let name: String = map.next_value()?;
        if name.is_empty() { None } else { Some(name) }
    }

    /// Deserialize the wonky structure of the "properties" key in the API
    /// into a more straightforward hashmap.
    /// Result will include keys such as "Quality" which should be later plucked
    /// into separate fields in `Item`.
    fn deserialize_item_properties<V: de::MapAccess<'de>>(
        &self, map: &mut V
    ) -> Result<HashMap<String, Option<String>>, V::Error> {
        let mut result = HashMap::new();

        // The "properties" array is polymorphic so we'll just deserialize it
        // to a typed JSON object. Seems hacky, but note that this is still
        // technically format-independent and allows to deserialize
        // items from something else than JSON. We're only using JSON DOM
        // as an intermediate representation.
        let array: Vec<HashMap<String, Json>> = map.next_value()?;
        for prop in array {
            // Example "properties" entry:
            // {
            //   "name": "Quality",
            //   "values": [["+17%", 1]],
            //   "displayMode": 0,
            //   "type": 6
            // }
            // Notice especially the nested array in "values".

            // TODO interpret values[0][1], which has to do with damage types
            // (physical, fire, etc.) or whether the value has been modified
            // by an affix on them item

            let name = prop.get("name").ok_or_else(|| de::Error::missing_field("name"))?;
            let values = prop.get("values").ok_or_else(|| de::Error::missing_field("values"))?;
            let value = values.as_array()
                .and_then(|v| v.get(0)).and_then(|v| v.as_array())
                .and_then(|v| v.get(0));
            result.insert(name, value);
        }

        Ok(result)
    }
}
impl ItemVisitor {
    fn deserialize_nonempty_string<V>(&self, map: &mut V) -> Result<String, V::Error>
        where V: de::MapAccess<'de>
    {
        map.next_value().and_then(|value: String| {
            if value.is_empty() {
                Err(de::Error::invalid_value(
                    Unexpected::Str(&value), &"non-empty string"))
            } else {
                Ok(value)
            }
        })
    }
}


// Utility functions

/// Deserialize a typed value out of an "intermediate" representation
// (usually a string or number) that has been deserialized previously.
///
/// This can be used to refine the final type of output after more information
/// is available in more complicated deserialization scenarios.
fn deserialize<'de, T, S, E>(from: S) -> Result<T, E>
    where T: Deserialize<'de, Error=E>,
          S: IntoDeserializer<'de, E>,
          E: de::Error
{
    let deserializer = IntoDeserializer::into_deserializer(from);
    T::deserialize(deserializer)
}
