//! Deserializer for item prices.

use std::fmt;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::Price;
use super::util::deserialize;


const EXPECTING_MSG: &str = "item price (as `$N $CURRENCY`)";

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(PriceVisitor)
    }
}

struct PriceVisitor;
impl<'de> Visitor<'de> for PriceVisitor {
    type Value = Price;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing a FromStr implementation for Price

        if v.is_empty() {
            return Err(de::Error::invalid_length(0, &"non-empty string"));
        }

        let parts: Vec<_> = v.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG));
        }

        // XXX: there are amounts like "10/19 chaos" encountered in the wild,
        // so we need more complex parsing than just FromStr -_-
        let amount: f64 = parts[0].parse().map_err(|e| de::Error::custom(
            format!("cannot parse price amount `{}`: {}", parts[0], e)))?;
        let currency = deserialize(parts[1])?;
        Ok(Price::new(amount, currency))
    }
}
