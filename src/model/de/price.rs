//! Deserializer for item prices.

use std::fmt;
use std::num::{ParseIntError, ParseFloatError};

use serde::de::{self, Deserialize, Visitor, Unexpected};

use super::super::{Currency, Price};
use super::super::util::Quasi;
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

        // TODO: we're also seeing prices as ".1 chaos (10:1c)"
        // which seem to indicate currency trade ratios; we should handle them
        let parts: Vec<_> = v.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG));
        }

        let amount: f64 = parse_amount(parts[0]).map_err(|e| de::Error::custom(
            format!("cannot parse price amount `{}`: {}", parts[0], e)))?;
        let currency: Quasi<Currency> = deserialize(parts[1])?;
        Ok(Price::new(amount, currency))
    }
}

// Utility functions

fn parse_amount(s: &str) -> Result<f64, ParseAmountError> {
    // Assume amount is in the first of `$NUMBER` or `$NUMBER / $NUMBER`.
    let nums: Vec<_> = s.split('/').collect();
    let num_count = nums.len();
    match num_count {
        1 => {
            // Try parsing as integer to handle common cases like "1 chaos".
            // Otherwise fall back to floats for things like "1.5 exa".
            let amount = nums[0].trim();
            amount.parse::<u64>().map(|a| a as f64)
                .or_else(|_| amount.parse::<f64>())
                .map_err(Into::into)
        },
        2 => {
            let numerator: f64 = nums[0].trim().parse()?;
            let denominator: f64 = nums[1].trim().parse()?;
            Ok(numerator / denominator)
        }
        _ => Err(ParseAmountError::Syntax),
    }
}

#[derive(Debug, Error)]
enum ParseAmountError {
    /// Error parsing floating point amount.
    Float(ParseFloatError),
    /// Error parsing rational amount (X/Y fraction where X,Y are integers).
    Rational(ParseIntError),
    /// General syntax error while parsing the amount.
    Syntax,
}
