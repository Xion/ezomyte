//! Deserializer for item properties.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserialize, Visitor};
use serde_json::Value as Json;

use super::super::Properties;


const EXPECTING_MSG: &str = "item properties array";


impl<'de> Deserialize<'de> for Properties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_seq(PropertiesVisitor)
    }
}

struct PropertiesVisitor;
impl<'de> Visitor<'de> for PropertiesVisitor {
    type Value = Properties;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where A: de::SeqAccess<'de>
    {
        let mut result = Properties::new();
        if let Some(0) = seq.size_hint() {
            return Ok(result);
        }

        // The properties' array is polymorphic so we'll just deserialize it
        // to a typed JSON object. Seems hacky, but note that this is still
        // technically format-independent and allows to deserialize
        // items from something else than JSON. We're only using JSON DOM
        // as an intermediate representation.
        while let Some(prop) = seq.next_element::<HashMap<String, Json>>()? {
            // Example item in the sequence:
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

            // If there is no value, we split the key by commas
            // and potentially insert multiple properties.
            // This handles the common gem "tags", like "Support", "Projectile", etc.,
            // which for some reason are lumped together in a single comma-separated property.
            match value {
                Some(val) => { result.put_with_value(name, val); },
                None => {
                    for key in name.split(',') {
                        result.put(key.trim().to_owned());
                    }
                }
            }
        }

        Ok(result)
    }
}
