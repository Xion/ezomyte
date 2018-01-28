//! Deserializers for item data.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use itertools::Itertools;
use regex::Regex;
use serde::de::{self, Deserialize, Visitor, Unexpected};
use serde_json::Value as Json;

use super::super::{Influence, Item, ItemDetails, Rarity};
use super::util::deserialize;


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

        // Data shared by all (or almost all) items.
        let mut id = None;
        let mut name = None;
        let mut base = None;
        let mut level = None;
        let mut category = None;
        let mut rarity = None;
        let mut quality = None;
        let mut properties = None;
        let mut identified = None;
        let mut sockets = None;
        let mut requirements = None;
        let mut corrupted = None;
        let mut influence = None;
        let mut duplicated = None;
        let mut flavour_text = None;
        let mut extra = HashMap::new();
        // Data specific to a particular category of items.
        let mut map_tier = None;
        let (mut gem_level, mut gem_xp) = (None, None);
        let mut flask_mods = None;
        let (mut implicit_mods,
             mut enchant_mods,
             mut explicit_mods,
             mut crafted_mods) = (None, None, None, None);

        while let Some(key) = map.next_key::<String>()? {
            let key = key.trim();
            match key {
                // Basic item attributes.
                "id" => {
                    check_duplicate!(id);
                    id = Some(Self::deserialize_nonempty_string(&mut map)?);
                }
                "name" => {
                    check_duplicate!(name);
                    let opt_name = Self::deserialize_name(&mut map)?;
                    name = Some(opt_name.map(|n| remove_angle_bracket_tags(&n).into_owned()));
                }
                "typeLine" => {
                    check_duplicate!("typeLine" => base);
                    let base_ = Self::deserialize_nonempty_string(&mut map)?;
                    base = Some(remove_angle_bracket_tags(&base_).into_owned());
                    // Note that `base` will be a full name for magic items
                    // (with prefix and suffix), so it has to be fixed using rarity later on.
                }
                "ilvl" => {
                    check_duplicate!("ilvl" => level);
                    level = Some(map.next_value()?);
                }
                "requirements" => {
                    check_duplicate!(requirements);
                    let req_kvps = Self::deserialize_value_map(&mut map)?.into_iter()
                        .filter_map(|(k, v)| v.map(|v| (k, v)));

                    let mut reqs = HashMap::new();
                    for (k, v) in req_kvps {
                        let key = deserialize(k)?;
                        let value = v.parse().map_err(|e| de::Error::custom(format!(
                            "failed to parse requirement value `{}`: {}", v, e)))?;
                        if reqs.contains_key(&key) {
                            return Err(de::Error::custom(
                                format!("duplicate requirement: {:?}", key)));
                        }
                        reqs.insert(key, value);
                    }
                    requirements = Some(reqs);
                }

                // Item category / type.
                "category" => {
                    check_duplicate!(category);
                    category = Some(map.next_value()?);
                }
                "frameType" => {
                    check_duplicate!("frameType" => rarity);
                    const MAX_RARITY: u64 = 3;  // corresponds to Rarity::Unique
                    let value: u64 = map.next_value()?;
                    rarity = Some(
                        // If the frameType doesn't describe rarity but rather item type,
                        // default to normal rarity.
                        if value > MAX_RARITY { Rarity::Normal } else { deserialize(value)? }
                    );
                }

                // Item mods.
                "identified" => {
                    check_duplicate!(identified);
                    identified = Some(map.next_value()?);
                }
                "utilityMods" => {
                    check_duplicate!("utilityMods" => flask_mods);
                    flask_mods = Some(map.next_value()?);
                }
                "implicitMods" => {
                    check_duplicate!("implicitMods" => implicit_mods);
                    implicit_mods = Some(map.next_value()?);
                }
                "enchantMods" => {
                    check_duplicate!("enchantMods" => enchant_mods);
                    enchant_mods = Some(map.next_value()?);
                }
                "explicitMods" => {
                    check_duplicate!("explicitMods" => explicit_mods);
                    explicit_mods = Some(map.next_value()?);
                }
                "craftedMods" => {
                    check_duplicate!("craftedMods" => crafted_mods);
                    crafted_mods = Some(map.next_value()?);
                }

                // Sockets.
                "sockets" => {
                    check_duplicate!(sockets);
                    sockets = Some(map.next_value()?);
                }
                "socketedItems" => {
                    // These are unsupported and ignored for now.
                    let socketed: Vec<Json> = map.next_value()?;
                    if !socketed.is_empty() {
                        warn!("Ignoring non-empty `socketedItems` in {}", name.as_ref()
                            .and_then(|n| n.as_ref().map(|n| n.as_str()))
                            .unwrap_or("<unknown item>"));
                    }
                }

                // Overall item modifiers.
                "corrupted" => {
                    check_duplicate!(corrupted);
                    corrupted = Some(map.next_value()?);
                }
                "duplicated" => {
                    check_duplicate!(duplicated);  // yo dawg
                    duplicated = Some(map.next_value()?);
                }
                "elder" => {
                    check_duplicate!("elder/shaper" => influence);
                    let is_elder = map.next_value()?;
                    if is_elder {
                        influence = Some(Some(Influence::Elder));
                    }
                }
                "shaper" => {
                    check_duplicate!("elder/shaper" => influence);
                    let is_shaped = map.next_value()?;
                    if is_shaped {
                        influence = Some(Some(Influence::Shaper));
                    }
                }

                // Various other properties.
                "flavourText" => {
                    check_duplicate!("flavourText" => flavour_text);
                    let text: Vec<String> = map.next_value()?;
                    flavour_text = Some(Some(text.join("").replace('\r', "")));
                }
                "properties" => {
                    check_duplicate!(properties);
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
                    if let Some(tier) = props.remove("Map Tier") {
                        let tier = tier.expect("map tier");
                        if map_tier.is_some() {
                            return Err(de::Error::duplicate_field("Map Tier"));
                        }
                        map_tier = Some(tier.parse().map_err(|_| {
                            de::Error::invalid_value(Unexpected::Str(&tier), &"number as string")
                        })?);
                    }
                    if let Some(lvl) = props.remove("Level") {
                        let mut lvl = lvl.expect("gem level");
                        if gem_level.is_some() {
                            return Err(de::Error::duplicate_field("Level"));
                        }
                        // For max-level gems, level is given as something like "20 (Max)".
                        // TODO: add is_max_level flag to ItemDetails::Gem,
                        // or make a dedicated data type for Level that includes this bit
                        lvl = lvl.replace("(Max)", "").trim().to_owned();
                        gem_level = Some(lvl.parse().map_err(|_| {
                            de::Error::invalid_value(Unexpected::Str(&lvl), &"number as string")
                        })?);
                    }

                    properties = Some(props);
                }
                "additionalProperties" => {
                    // TODO: signal duplicate "additionalProperties"
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

                // Ignored / unrecognized fields.
                "verified" => { map.next_value::<bool>()?; }  // ignore
                "support" => { map.next_value::<bool>()?; }   // ignore
                "league" => { map.next_value::<String>()?; }  // ignore, handled by `Stash`
                "lockedToCharacter" => { map.next_value::<bool>()?; }  // ignore
                key => {
                    trace!("Unrecognized item attribute `{}`, adding to `extra` map", key);
                    extra.insert(key.to_owned(), map.next_value()?);
                }
            }
        }

        let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
        let mut name = name.ok_or_else(|| de::Error::missing_field("name"))?;
        let mut base = base.ok_or_else(|| de::Error::missing_field("typeLine"))?;
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

        // Also, if a magic item has no name by itself,
        // then its "typeLine" (i.e. `base`) is what actually contains its full name.
        // We need to treat it as such, and also manually strip the prefix and/or suffix.
        if rarity == Rarity::Magic && name.is_none() && !base.is_empty() {
            name = Some(base.to_owned());

            // Use our knowledge of explicit mods to determine
            // if the magic item has a suffix, a prefix, or both.
            lazy_static! {
                // This matches a suffix text such as "of Warding" or "of the Leopard".
                static ref SUFFIX_RE: Regex = Regex::new(r#"of\s+(the\s+)?\w+"#).unwrap();
            }
            let mods_count = explicit_mods.as_ref().map(|m: &Vec<_>| m.len()).unwrap_or(0);
            let has_suffix = SUFFIX_RE.is_match(&*base);
            let has_prefix = match mods_count {
                // XXX: below is basically broken due to hybrid affixes,
                // meaning e.g. that 2 mods may mean a hybrid affix rather than suffix+prefix
                // and we have no way of discerning which case it is, short of knowing
                // the names of affixes :/
                0 => false,
                1 => !has_suffix,  // the sole affix is a suffix
                2...4 => true,  // >2 mods means at least one of the affixes is hybrid
                _ => return Err(de::Error::custom(format!(
                    "magic items can only have up to 4 explicit mods, but found {}", mods_count))),
            };

            // Strip them all from the item name to obtain the base name.
            if has_suffix {
                let offset = SUFFIX_RE.find_iter(&*base).last().unwrap().start();
                base = base[..offset].trim_right().to_owned().into();
            }
            if has_prefix {
                base = base.split_whitespace().into_iter().skip(1).collect();
            }
        }

        // Round up item mods' information into an ItemDetails data type.
        let details = {
            // TODO: verify that we didn't get an invalid combination of fields
            // (like flask_mods + crafted_mods)
            let identified = identified.unwrap_or(true);
            if !identified {
                Some(ItemDetails::Unidentified)
            } else if let Some(tier) = map_tier {
                Some(ItemDetails::Map{
                    tier: tier,
                    mods: explicit_mods.unwrap_or_default(),
                })
            } else if let (Some(level), Some(xp)) = (gem_level, gem_xp) {
                Some(ItemDetails::Gem{
                    level,
                    experience: xp,
                })
            } else if let Some(fm) = flask_mods {
                Some(ItemDetails::Flask{mods: fm})
            } else if let (Some(imp), Some(enc),
                           Some(ex), Some(c)) = (implicit_mods, enchant_mods,
                                                 explicit_mods, crafted_mods) {
                Some(ItemDetails::Gear{
                    implicit: imp,
                    enchants: enc,
                    explicit: ex,
                    crafted: c,
                })
            } else {
                // Some items -- like currencies or Sacrifice at Noon/Dusk/etc. fragments
                // -- don't have any identifiable details.
                None
            }
        };

        // Retain the `properties` that have values, turning the rest into Item `tags`.
        let (props_with_values, props_sans_values): (Vec<_>, Vec<_>) =
            properties.into_iter().partition(|&(_, ref v)| v.is_some());
        let properties = props_with_values.into_iter()
            .map(|(k, v)| (k, v.unwrap())).collect();
        let tags = props_sans_values.into_iter().map(|(k, _)| k).collect();

        Ok(Item {
            id,
            name: name.map(|n| n.to_string()),
            base: base.to_string(),
            level, category, rarity, quality, properties, tags, details,
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
    fn deserialize_value_map<'de, V: de::MapAccess<'de>>(
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

            let name = prop.get("name")
                .and_then(|n| n.as_str().map(|s| s.to_owned()))
                .ok_or_else(|| de::Error::missing_field("name"))?;
            let values = prop.get("values").ok_or_else(|| de::Error::missing_field("values"))?;
            let value = values.as_array()
                // TODO: support multiple values
                // (which are probably used for multiple kinds of elemental damage)
                .and_then(|v| v.get(0)).and_then(|v| v.as_array())
                // TODO interpret values[i][1], which has to do with damage types
                // (physical, fire, etc.) or whether the value has been modified
                // by an affix on them item
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

/// Remove the "tags" like <<set:MS>> that can sometimes be found
/// in the item "name" or "typeLine".
fn remove_angle_bracket_tags(s: &str) -> Cow<str> {
    lazy_static! {
        static ref ANGLE_TAG_RE: Regex = Regex::new(r#"<<\w+:\w+>>"#).unwrap();
    }
    ANGLE_TAG_RE.replace_all(s, "")
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use model::Item;

    #[test]
    fn minimal() {
        let item_spec = json!({
            "id": "123abc",
            "name": "",
            "typeLine": "Example Amazing Item of Testing",
            "ilvl": 80,
            "category": "jewels",
            "frameType": 0,
        });
        from_value::<Item>(item_spec).unwrap();
    }

    #[test]
    fn with_quality() {
        let item_spec = json!({
            "id": "123abc",
            "name": "",
            "typeLine": "Example Amazing Item of Testing",
            "ilvl": 80,
            "category": "jewels",
            "frameType": 0,
            "properties": [
                {
                    "name": "Quality",
                    "values": [["+13%", 1]],
                },
            ],
        });
        let item = from_value::<Item>(item_spec).unwrap();
        let quality: u8 = item.quality.into();
        assert_eq!(13, quality);
    }
}
