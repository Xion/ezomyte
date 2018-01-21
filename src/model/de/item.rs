//! Deserializers for item data.

use std::collections::HashMap;
use std::fmt;

use itertools::Itertools;
use serde::de::{self, Deserialize, IntoDeserializer, Visitor, Unexpected};
use serde_json::Value as Json;

use super::super::{Experience, Influence, Item, ItemDetails, Rarity};


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
        let mut identified = None;
        let (mut gem_level, mut gem_xp) = (None, None);
        let mut flask_mods = None;
        let (mut implicit_mods,
             mut enchant_mods,
             mut explicit_mods,
             mut crafted_mods) = (None, None, None, None);
        let mut sockets = None;
        let mut requirements = None;
        let mut corrupted = None;
        let mut influence = None;
        let mut duplicated = None;
        let mut flavour_text = None;
        let mut extra = HashMap::new();

        while let Some(key) = map.next_key::<String>()? {
            let key = key.trim();
            match key {
                // Basic item attributes.
                "id" => {
                    if id.is_some() {
                        return Err(de::Error::duplicate_field("id"));
                    }
                    id = Some(Self::deserialize_nonempty_string(&mut map)?);
                }
                "name" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("name"));
                    }
                    name = Some(Self::deserialize_name(&mut map)?);
                }
                "typeLine" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("typeLine"));
                    }
                    // Note that `base` will be a full name for magic items
                    // (with prefix and suffix), so it has to be fixed using rarity later on.
                    base = Some(Self::deserialize_nonempty_string(&mut map)?);
                }
                "ilvl" => {
                    if level.is_some() {
                        return Err(de::Error::duplicate_field("ilvl"));
                    }
                    level = Some(map.next_value()?);
                }
                "requirements" => {
                    // TODO
                }

                // Item category / type.
                "category" => {
                    if category.is_some() {
                        return Err(de::Error::duplicate_field("category"));
                    }
                    category = Some(map.next_value()?);
                }
                "frameType" => {
                    if rarity.is_some() {
                        return Err(de::Error::duplicate_field("frameType"));
                    }
                    let value: u64 = map.next_value()?;
                    rarity = Some(deserialize(value)?);
                }

                // Item mods.
                "identified" => {
                    if identified.is_some() {
                        return Err(de::Error::duplicate_field("identified"));
                    }
                    identified = Some(map.next_value()?);
                }
                "utilityMods" => {
                    if flask_mods.is_some() {
                        return Err(de::Error::duplicate_field("utilityMods"));
                    }
                    flask_mods = Some(map.next_value()?);
                }
                "implicitMods" => {
                    if implicit_mods.is_some() {
                        return Err(de::Error::duplicate_field("implicitMods"));
                    }
                    implicit_mods = Some(map.next_value()?);
                }
                "enchantMods" => {
                    if enchant_mods.is_some() {
                        return Err(de::Error::duplicate_field("enchantMods"));
                    }
                    enchant_mods = Some(map.next_value()?);
                }
                "explicitMods" => {
                    if explicit_mods.is_some() {
                        return Err(de::Error::duplicate_field("explicitMods"));
                    }
                    explicit_mods = Some(map.next_value()?);
                }
                "craftedMods" => {
                    if crafted_mods.is_some() {
                        return Err(de::Error::duplicate_field("craftedMods"));
                    }
                    crafted_mods = Some(map.next_value()?);
                }

                // Sockets.
                "sockets" => {
                    // TODO
                }
                "socketedItems" => {
                    // These are unsupported and ignored for now.
                    let socketed: Vec<Json> = map.next_value()?;
                    if !socketed.is_empty() {
                        warn!("Ignoring non-empty `socketedItems` in {}",
                            name.as_ref().and_then(|n| n.as_ref().map(|n| n.as_str()))
                            .unwrap_or("<unknown item>"));
                    }
                }

                // Overall item modifiers.
                "corrupted" => {
                    if corrupted.is_some() {
                        return Err(de::Error::duplicate_field("corrupted"));
                    }
                    corrupted = Some(map.next_value()?);
                }
                "duplicated" => {
                    if duplicated.is_some() {
                        return Err(de::Error::duplicate_field("duplicated"));
                    }
                    duplicated = Some(map.next_value()?);
                }
                "elder" => {
                    if influence.is_some() {
                        return Err(de::Error::duplicate_field("elder/shaper"));
                    }
                    let is_elder = map.next_value()?;
                    if is_elder {
                        influence = Some(Some(Influence::Elder));
                    }
                }
                "shaper" => {
                    if influence.is_some() {
                        return Err(de::Error::duplicate_field("elder/shaper"));
                    }
                    let is_shaped = map.next_value()?;
                    if is_shaped {
                        influence = Some(Some(Influence::Shaper));
                    }
                }

                // Various other properties.
                "flavour_text" => {
                    if flavour_text.is_some() {
                        return Err(de::Error::duplicate_field("flavour_text"));
                    }
                    flavour_text = Some(map.next_value()?);
                }
                "properties" => {
                    if properties.is_some() {
                        return Err(de::Error::duplicate_field("properties"));
                    }
                    let mut props = Self::deserialize_value_map(&mut map)?;

                    // Pluck out some of the properties that we are providing
                    // as separate fields on `Item`.
                    if let Some(q) = props.remove("Quality") {
                        let q = q.expect("item quality percentage");
                        if quality.is_some() {
                            return Err(de::Error::duplicate_field("Quality"));
                        }
                        quality = Some(deserialize(q)?);
                    }
                    if let Some(lvl) = props.remove("Level") {
                        let lvl = lvl.expect("gem level");
                        if gem_level.is_some() {
                            return Err(de::Error::duplicate_field("Level"));
                        }
                        gem_level = Some(deserialize(lvl)?);
                    }

                    properties = Some(props);
                }
                "additionalProperties" => {
                    let mut props = Self::deserialize_value_map(&mut map)?;

                    // Currently, only the gem experience
                    // seems to be stored as additionalProperties.
                    if let Some(exp) = props.remove("Experience") {
                        let exp = exp.expect("gem experience");
                        if gem_xp.is_some() {
                            return Err(de::Error::duplicate_field("Experience"));
                        }
                        gem_xp = Some(deserialize(exp)?);
                    }

                    if !props.is_empty() {
                        warn!("Unexpected additionalProperties: {}", props.keys().format(", "));
                    }
                }
                key => {
                    trace!("Unrecognized item attribute `{}`, adding to `extra` map", key);
                    extra.insert(key.to_owned(), map.next_value()?);
                }
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
        let sockets = sockets.unwrap_or_default();
        let requirements = requirements.unwrap_or_default();
        let corrupted = corrupted.unwrap_or(false);
        let influence = influence.unwrap_or(None);
        let duplicated = duplicated.unwrap_or(false);
        let flavour_text = flavour_text.unwrap_or(None);

        // TODO: fix `base` using `rarity` when it's a magic item
        // (remove prefix & suffix).
        // TODO: remove junk like <<set:S>>, e.g.:
        // "<<set:MS>><<set:M>><<set:S>>Cautious Divine Life Flask of Warding"
        // from both `name` and `base`

        // Round up item mods' information into an ItemDetails data type.
        let details = {
            // TODO: verify that we didn't get an invalid combination of fields
            // (like flask_mods + crafted_mods)
            let identified = identified.unwrap_or(true);
            if !identified {
                ItemDetails::Unidentified
            } else if let (Some(level), Some(xp)) = (gem_level, gem_xp) {
                ItemDetails::Gem{
                    level,
                    experience: xp,
                }
            } else if let Some(fm) = flask_mods {
                ItemDetails::Flask{mods: fm}
            } else if let (Some(imp), Some(enc),
                           Some(ex), Some(c)) = (implicit_mods, enchant_mods,
                                                 explicit_mods, crafted_mods) {
                ItemDetails::Mods{
                    implicit: imp,
                    enchants: enc,
                    explicit: ex,
                    crafted: c,
                }
            } else {
                // A total lack of mods would indicate a white/common item.
                // TODO: verify this is indeed the case; perhaps the API would return
                // empty mod lists instead so white junk would be captured by the branch above
                // and this branch should be an error instead
                ItemDetails::default()
            }
        };

        // Retain the properties that have values.
        // TODO: figure out what the others are (w/o values), and possibly put them
        // in a separate field on `Item`
        let properties = properties.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect();

        Ok(Item {
            id, name, base, level, category, rarity, quality, properties, details,
            sockets, requirements, corrupted, influence, duplicated, flavour_text,
            extra,
        })
    }
}
impl ItemVisitor {
    /// Deserialize item name into an optional string
    /// by turning an empty one into None.
    /// (Items such as gems do not have a separate name).
    fn deserialize_name<'de, V>(map: &mut V) -> Result<Option<String>, V::Error>
        where V: de::MapAccess<'de>
    {
        map.next_value().map(|name: String| {
            if name.is_empty() { None } else { Some(name) }
        })
    }

    /// Deserialize the wonky "value map" structure that some JSON keys use
    /// into a more straightforward hashmap.
    /// The format is used at least by "properties", "additionalProperties",
    /// and "requirements" keys in the item JSON.
    ///
    /// Result will include keys such as "Quality" which should be later plucked
    /// into separate fields in `Item`.
    fn deserialize_value_map<'de ,V: de::MapAccess<'de>>(
        map: &mut V
    ) -> Result<HashMap<String, Option<String>>, V::Error> {
        let mut result = HashMap::new();

        // The value map array is polymorphic so we'll just deserialize it
        // to a typed JSON object. Seems hacky, but note that this is still
        // technically format-independent and allows to deserialize
        // items from something else than JSON. We're only using JSON DOM
        // as an intermediate representation.
        let array: Vec<HashMap<String, Json>> = map.next_value()?;
        for prop in array {
            // Example value map (from "properties"):
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

            let name = prop.get("name")
                .and_then(|n| n.as_str().map(|s| s.to_owned()))
                .ok_or_else(|| de::Error::missing_field("name"))?;
            let values = prop.get("values").ok_or_else(|| de::Error::missing_field("values"))?;
            let value = values.as_array()
                .and_then(|v| v.get(0)).and_then(|v| v.as_array())
                .and_then(|v| v.get(0)).and_then(|v| v.as_str().map(|s| s.to_owned()));
            result.insert(name, value);
        }

        Ok(result)
    }
}
impl ItemVisitor {
    fn deserialize_nonempty_string<'de, V>(map: &mut V) -> Result<String, V::Error>
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
/// (usually a string or number) that has been deserialized previously.
///
/// This can be used to refine the final type of output after more information
/// is available in more complicated deserialization scenarios.
fn deserialize<'de, T, S, E>(from: S) -> Result<T, E>
    where T: Deserialize<'de>,
          S: IntoDeserializer<'de, E>,
          E: de::Error
{
    let deserializer = IntoDeserializer::into_deserializer(from);
    T::deserialize(deserializer)
}
