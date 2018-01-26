//! Deserializers for item sockets.

use std::collections::HashMap;
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

        let mut abyss_count = 0;
        let mut regular_groups = HashMap::new();

        while let Some(socket) = seq.next_element::<HashMap<String, Json>>()? {
            let color = socket.get("sColour").and_then(|c| c.as_str())
                .ok_or_else(|| de::Error::custom("socket color missing"))?;
            if color == "A" {
                abyss_count += 1;
                continue;
            }
            let group = socket.get("group").and_then(|g| g.as_u64())
                .ok_or_else(|| de::Error::custom("socket group number missing"))?;
            let regular_color = deserialize(color)?;
            regular_groups.entry(group).or_insert_with(Vec::new)
                .push(regular_color);
        }

        // TODO: check if the regular group indices form a contiguous 0-based range
        let regular_groups = regular_groups.into_iter()
            .sorted_by_key(|&(i, _)| i).into_iter()
            .map(|(i, g)| SocketGroup{id: i as u8, colors: g})
            .collect();

        Ok(ItemSockets{abyss_count, regular_groups})
    }
}
