//! Item price.

use std::fmt;
use std::cmp::Ordering;

use num::ToPrimitive;
use serde_json::{to_value as to_json, Value as Json};

use super::currency::Currency;


/// Price of an item in a particular `Currency`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Price(f64, Currency);
// TODO: consider using a decimal type for accuracy and the Hash/Eq traits

impl Price {
    /// Create a new `Price` object.
    #[inline]
    pub fn new<A: ToPrimitive>(amount: A, currency: Currency) -> Self {
        let amount = amount.to_f64().expect("price amount");
        Price(amount, currency)
    }

    /// Create a new `Price` of a single unit of given currency.
    #[inline]
    pub fn one(currency: Currency) -> Self {
        Price(1.0, currency)
    }
}

impl Price {
    /// Price amount.
    #[inline(always)]
    pub fn amount(&self) -> f64 { self.0 }
    /// Currency used in the price.
    #[inline(always)]
    pub fn currency(&self) -> Currency { self.1 }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Price) -> Option<Ordering> {
        if self.currency() == other.currency() {
            self.amount().partial_cmp(&other.amount())
        } else {
            None
        }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let currency = match to_json(self.currency()) {
            Ok(Json::String(s)) => s,
            Err(e) => {
                error!("Failed to Display-format currency `{:?}`: {}",
                    self.currency(), e);
                return Err(fmt::Error);
            }
            v => panic!("Unexpected serialization of Currency for Display: {:?}", v),
        };
        write!(fmt, "{} {}", self.amount(), currency)
    }
}
