//! Deserializers for item sockets.

use std::collections::{HashMap, HashSet};
use std::fmt;

use itertools::Itertools;
use serde::de::{self, Deserialize, Visitor};
use serde_json::Value as Json;

use super::super::{ItemSockets, SocketGroup};
use super::util::deserialize;


const EXPECTING_MSG: &str = "map with sockets data";


impl<'de> Deserialize<'de> for ItemSockets {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_seq(ItemSocketsVisitor)
    }
}

struct ItemSocketsVisitor;
impl<'de> Visitor<'de> for ItemSocketsVisitor {
    type Value = ItemSockets;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where V: de::SeqAccess<'de>
    {
        if let Some(0) = seq.size_hint() {
            return Ok(ItemSockets::default());
        }

        let mut abyssal_count = 0;
        let mut regular_groups = HashMap::new();
        let mut indices = HashSet::new();

        while let Some(socket) = seq.next_element::<HashMap<String, Json>>()? {
            let color = socket.get("sColour").and_then(|c| c.as_str())
                .ok_or_else(|| de::Error::missing_field("sColour"))?;
            if color == "A" {
                abyssal_count += 1;
                continue;
            }
            let group = socket.get("group").and_then(|g| g.as_u64())
                .ok_or_else(|| de::Error::missing_field("group"))?;
            let regular_color = deserialize(color)?;

            regular_groups.entry(group).or_insert_with(Vec::new)
                .push(regular_color);
            indices.insert(group);
        }

        // Check if the group indices form a contiguous 0-based range.
        let expected_indices = (0u64..indices.len() as u64).collect_vec();
        let actual_indices = indices.into_iter().sorted();
        if actual_indices != expected_indices {
            return Err(de::Error::custom(format!(
                "socket group indices not in a contiguous range: expected {:?}, got {:?}",
                expected_indices, actual_indices)));
        }

        let regular_groups = regular_groups.into_iter()
            .sorted_by_key(|&(i, _)| i).into_iter()
            .map(|(i, g)| SocketGroup{id: i as u8, colors: g})
            .collect();

        Ok(ItemSockets{abyssal_count, regular_groups})
    }
}
