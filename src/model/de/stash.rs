//! Deserializer for an entire item stash.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::de::{self, Deserialize, Visitor};
use serde_json::Value as Json;

use super::super::Stash;
use super::util::{deserialize, NoopIntoDeserializer};


const EXPECTING_MSG: &str = "map with item stash data";


impl<'de> Deserialize<'de> for Stash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_map(StashVisitor)
    }
}

struct StashVisitor;
impl<'de> Visitor<'de> for StashVisitor {
    type Value = Stash;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>
    {
        // (See the botoom of the function for the rationale behind nested Options).
        let mut id = None;
        let mut league = None;
        let mut label: Option<Option<_>> = None;
        let mut type_ = None;
        let mut account: Option<Option<_>> = None;
        let mut last_character = None;
        let mut items = None;

        while let Some(key) = map.next_key::<String>()? {
            let key = key.trim();
            match key {
                "id" => {
                    check_duplicate!(id);
                    id = Some(map.next_value()?);
                }
                "stash" => {
                    check_duplicate!("stash" => label);
                    label = Some(map.next_value()?);
                }
                "stashType" => {
                    check_duplicate!("stashType" => type_);
                    type_ = Some(map.next_value()?);
                }
                "accountName" => {
                    check_duplicate!("accountName" => account);
                    account = Some(map.next_value()?);
                }
                "lastCharacterName" => {
                    check_duplicate!("lastCharacterName" => last_character);
                    last_character = Some(map.next_value()?);
                }
                "items" => {
                    let items_json: Vec<HashMap<String, Json>> = map.next_value()?;

                    league = Some({
                        // The API puts "league" as a key on the items, not the stash,
                        // so we have to pluck it from there.
                        // Also check that all those league values are actually identical.
                        let leagues: HashSet<_> = items_json.iter()
                            .filter_map(|i| i.get("league").and_then(|l| l.as_str()))
                            .collect();
                        if leagues.len() > 1 {
                            return Err(de::Error::custom(format!(
                                "items from multiple ({}) leagues in a single stash tab?!", leagues.len())));
                        }
                        // If the stash is empty, we're gonna say it's from standard.
                        // Such stash isn't very interesting anyway,
                        // and making Stash::league into an Option just for this is awkward.
                        deserialize(leagues.into_iter().next().unwrap_or("Standard"))?
                    });

                    // Deserialize the actual items from the JSON structure
                    // (after applying a newtype workaround to serde_json::Value
                    //  so that we can call deserialize() on the whole vector of hashmaps).
                    let items_de: Vec<HashMap<_, _>> = items_json.into_iter()
                        .map(|i| i.into_iter()
                            .map(|(k, v)| (k, NoopIntoDeserializer::new(v)))
                            .collect()
                        ).collect();
                    items = Some(deserialize(items_de).map_err(|e| {
                        de::Error::custom(format!("cannot deserialize stashed items: {}", e))
                    })?);
                }

                // Ignored / unrecognized fields.
                "public" => {
                    // Ignoring. Can it even be false if the stash is visible in the API?
                    map.next_value::<bool>()?;
                },
                key => {
                    warn!("Unrecognized key in stash JSON: `{}`", key);
                    map.next_value::<Json>()?;  // ignore
                }
            }
        }

        let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
        let label = label.ok_or_else(|| de::Error::missing_field("stash"))?;
        let league = league.unwrap_or_default();
        let type_ = type_.ok_or_else(|| de::Error::missing_field("stashType"))?;
        let account = account.ok_or_else(|| de::Error::missing_field("accountName"))?;
        let last_character = last_character.unwrap_or_default();
        let items = items.unwrap_or_default();

        Ok(Stash{
            id, league, type_, last_character, items,
            // Historical records seem to include some broken stashes
            // where "stash" and "accountName" keys are null.
            // We'll just convert them to empty strings.
            label: label.unwrap_or_default(),
            account: account.unwrap_or_default(),
        })
    }
}
