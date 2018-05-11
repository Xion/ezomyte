//! Deserializers for item data.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use itertools::Itertools;
use regex::Regex;
use serde::de::{self, Deserialize, Visitor, Unexpected};
use serde_json::Value as Json;

use ::common::util::Quasi;
use super::super::{
    Influence, Item, ItemCategory, ItemDetails, Mod, ModType, Properties, Quality, Rarity,
};
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
        let (mut map_tier,
             mut item_quantity,
             mut item_rarity,
             mut monster_pack_size) = (None, None, None, None);
        let (mut gem_level, mut gem_xp) = (None, None);
        let (mut duration,
             mut charges_per_use,
             mut flask_mods) = (None, None, None);
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
                "frameType" => {
                    check_duplicate!("frameType" => rarity);

                    // This field is wonky, as it describes either an item rarity,
                    // or a less common item category (like a sealed prophecy item or a divination card).
                    const MAX_RARITY: u64 = 3;  // corresponds to Rarity::Unique
                    let value: u64 = map.next_value()?;
                    if value > MAX_RARITY {
                        // If the frameType doesn't describe rarity but rather item type,
                        // default to normal rarity.
                        rarity = Some(Rarity::Normal);
                        // Detect the uncommon item types.
                        match value {
                            8 => { category = Some(Quasi::from(ItemCategory::Prophecy)); }
                            _ => {}
                        }
                    } else {
                        rarity = Some(deserialize(value)?);
                    }
                }
                "category" => {
                    if category.as_ref().and_then(|c| c.as_ref()) == Some(&ItemCategory::Prophecy) {
                        // Sealed prophecies are detected using the "frameType" attribute,
                        // because their JSON "category" is actually `{"currency": []}` (!).
                        continue;
                    }
                    check_duplicate!(category);
                    category = Some(map.next_value::<Quasi<ItemCategory>>()?);
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
                    // Flavour text is usually given as an array of lines with a trailing '\r',
                    // although some items (like Belly) miss the carriage return.
                    let lines: Vec<String> = map.next_value()?;
                    let text = lines.iter().map(|l| l.trim()).join(" ");
                    flavour_text = Some(if text.is_empty() { None } else { Some(text) });
                }
                "properties" => {
                    check_duplicate!(properties);
                    let mut props: Properties = map.next_value()?;

                    // Pluck out some of the properties that we are providing
                    // as separate fields on `Item`.

                    // Common properties.
                    if let Some(q) = props.remove("Quality") {
                        let q = q.expect("item quality percentage");
                        if quality.is_some() {
                            return Err(de::Error::duplicate_field("Quality"));
                        }
                        quality = Some(deserialize(q)?);
                    }

                    // Map properties.
                    // TODO: introduce a Percentage data type and use it for the map bonuses
                    // and item quality so that we don't have to do the hack where we
                    // parse the map values as Quality
                    if let Some(tier) = props.remove("Map Tier") {
                        let tier = tier.expect("map tier");
                        if map_tier.is_some() {
                            return Err(de::Error::duplicate_field("Map Tier"));
                        }
                        map_tier = Some(tier.parse().map_err(|_| {
                            de::Error::invalid_value(Unexpected::Str(&tier), &"number as string")
                        })?);
                    }
                    if let Some(quantity) = props.remove("Item Quantity") {
                        let quantity = quantity.expect("item quantity bonus value");
                        if item_quantity.is_some() {
                            return Err(de::Error::duplicate_field("Item Quantity"));
                        }
                        item_quantity = Some(deserialize(quantity).map(|Quality(q)| q as i32)?);
                    }
                    if let Some(rarity) = props.remove("Item Rarity") {
                        let rarity = rarity.expect("item rarity bonus value");
                        if item_rarity.is_some() {
                            return Err(de::Error::duplicate_field("Item Rarity"));
                        }
                        item_rarity = Some(deserialize(rarity).map(|Quality(q)| q as i32)?);
                    }
                    if let Some(pack_size) = props.remove("Monster Pack Size") {
                        let pack_size = pack_size.expect("monster pack size bonus value");
                        if monster_pack_size.is_some() {
                            return Err(de::Error::duplicate_field("Monster Pack Size"));
                        }
                        monster_pack_size = Some(deserialize(pack_size).map(|Quality(q)| q as i32)?);
                    }

                    // Gem properties.
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

                    // Flask properties.
                    if let Some(secs) = props.remove("Lasts %0 Seconds") {
                        let secs = secs.expect("flask duration");
                        if duration.is_some() {
                            return Err(de::Error::duplicate_field("Lasts %0 Seconds"));
                        }
                        let secs: f64 = secs.parse().map_err(|_| {
                            de::Error::invalid_value(Unexpected::Str(&secs), &"flask duration in seconds")
                        })?;
                        duration = Some(Duration::from_millis((secs * 1000.0) as u64));
                    }
                    if let Some(charges) = props.remove("Consumes %0 of %1 Charges on use") {
                        // TODO: support the other value, i.e. max_charges
                        let charges = charges.expect("number of flask charges per use");
                        if charges_per_use.is_some() {
                            return Err(de::Error::duplicate_field("Consumes %0 of %1 Charges on use"));
                        }
                        charges_per_use = Some(charges.parse().map_err(|_| {
                            de::Error::invalid_value(Unexpected::Str(&charges), &"number as string")
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
                "nextLevelRequirements" => { map.next_value::<Json>()?; },  // ignore for now
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
                    item_quantity: item_quantity.unwrap_or(0),
                    item_rarity: item_rarity.unwrap_or(0),
                    monster_pack_size: monster_pack_size.unwrap_or(0),
                    mods: explicit_mods.unwrap_or_default(),
                })
            } else if let (Some(level), Some(xp)) = (gem_level, gem_xp) {
                Some(ItemDetails::Gem{
                    level,
                    experience: xp,
                })
            } else if let (Some(duration),
                           Some(charges_per_use),
                           Some(mods)) = (duration, charges_per_use, flask_mods) {
                Some(ItemDetails::Flask{duration, charges_per_use, mods})
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
