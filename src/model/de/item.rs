//! Deserializers for item data.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use itertools::Itertools;
use regex::Regex;
use serde::de::{self, Deserialize, Visitor, Unexpected};
use serde_json::Value as Json;

use super::super::{Influence, Item, ItemCategory, ItemDetails, Mod, ModType, Properties, Rarity};
use super::super::util::Quasi;
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

                    // "requirements" use the same format of value encoding as "properties".
                    let req_kvps = map.next_value::<Properties>()?.into_iter()
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
                    category = Some(map.next_value::<Quasi<ItemCategory>>()?);
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
                    flask_mods = Some(Self::deserialize_mods(&mut map, ModType::Explicit)?);
                }
                "implicitMods" => {
                    check_duplicate!("implicitMods" => implicit_mods);
                    implicit_mods = Some(Self::deserialize_mods(&mut map, ModType::Implicit)?);
                }
                "enchantMods" => {
                    check_duplicate!("enchantMods" => enchant_mods);
                    enchant_mods = Some(Self::deserialize_mods(&mut map, ModType::Enchant)?);
                }
                "explicitMods" => {
                    check_duplicate!("explicitMods" => explicit_mods);
                    explicit_mods = Some(Self::deserialize_mods(&mut map, ModType::Explicit)?);
                }
                "craftedMods" => {
                    check_duplicate!("craftedMods" => crafted_mods);
                    crafted_mods = Some(Self::deserialize_mods(&mut map, ModType::Crafted)?);
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
                    // TODO: some items (like Belly) don't have the trailing \r or \n
                    // at each string of flavour_text, so we should add them (or spaces)
                    // if we're merging the text into a single string
                    flavour_text = Some(Some(text.join("").replace('\r', "")));
                }
                "properties" => {
                    check_duplicate!(properties);
                    let mut props: Properties = map.next_value()?;

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
                    let mut props: Properties = map.next_value()?;

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
        if rarity == Rarity::Magic && name.is_none() && !base.is_empty() {
            name = Some(base.to_owned());
            // TODO: Figure out a way of determining what the prefix and suffix is,
            // and eliminating them from `name` to get the base name.
            // Right now, unfortunately, the API doesn't return the affixes, just mods,
            // which means we cannot tell anything with certainty due to hybrid affixes.
            base = "".into();
        }

        // Round up most of the item information into an ItemDetails data type.
        let details = {
            let identified = identified.unwrap_or(true);
            let has_gear_mods =
                implicit_mods.is_some() || enchant_mods.is_some()
                || explicit_mods.is_some() || crafted_mods.is_some();

            // TODO: verify that we didn't get an invalid combination of fields
            // (like flask_mods + crafted_mods)
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
            } else if has_gear_mods
                      // Exclude currencies here because the API returns their on-use "mods"
                      // (like "Reforges a rare item with new random properties" for Chaos Orb)
                      // as `explicitMods`, and they obviously aren't `Gear`.
                      && category.as_ref() != Some(&ItemCategory::Currency) {
                Some(ItemDetails::Gear{
                    implicit: implicit_mods.unwrap_or_default(),
                    enchants: enchant_mods.unwrap_or_default(),
                    explicit: explicit_mods.unwrap_or_default(),
                    crafted: crafted_mods.unwrap_or_default(),
                })
            } else {
                // Some items -- like currencies or Sacrifice at Noon/Dusk/etc. fragments
                // -- don't have any identifiable details.
                None
            }
        };

        Ok(Item {
            id,
            name: name.map(|n| n.to_string()),
            base: base.to_string(),
            level, category, rarity, quality, properties, details,
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

    /// Deserialize a collection of item mods of given type.
    fn deserialize_mods<'de, V>(map: &mut V, mod_type: ModType) -> Result<Vec<Mod>, V::Error>
        where V: de::MapAccess<'de>
    {
        map.next_value().map(|mods: Vec<String>| {
            mods.into_iter().map(|m| Mod::new(mod_type, m)).collect()
        })
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
