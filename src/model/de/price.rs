//! Deserializer for item prices.

use std::fmt;

use serde::de::{self, Deserialize, Visitor, Unexpected};

use ::common::util::Quasi;
use super::super::{Currency, Price};
use super::util::deserialize;
use util::parse_number;


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

        // TODO: we're also seeing prices as ".1 chaos (10:1c)"
        // which seem to indicate currency trade ratios; we should handle them
        let parts: Vec<_> = v.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG));
        }

        let amount: f64 = parse_number(parts[0]).map_err(|e| de::Error::custom(
            format!("cannot parse price amount `{}`: {}", parts[0], e)))?;
        let currency: Quasi<Currency> = deserialize(parts[1])?;
        Ok(Price::new(amount, currency))
    }
}
