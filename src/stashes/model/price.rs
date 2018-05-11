//! Item price.

use std::fmt;
use std::cmp::Ordering;

use num::ToPrimitive;
use serde_json::{to_value as to_json, Value as Json};

use ::common::util::Quasi;
use super::currency::Currency;


/// Price of an item in a particular `Currency`.
#[derive(Clone, PartialEq)]
pub struct Price(f64, Quasi<Currency>);
// TODO: consider using a decimal type for accuracy and the Hash/Eq traits

impl Price {
    /// Create a new `Price` object.
    #[inline]
    pub fn new<A, C>(amount: A, currency: C) -> Self
        where A: ToPrimitive,
              C: Into<Quasi<Currency>>
    {
        let amount = amount.to_f64().expect("price amount");
        Price(amount, currency.into())
    }

    /// Create a new `Price` of a single unit of given currency.
    #[inline]
    pub fn one<C>(currency: C) -> Self
        where C: Into<Quasi<Currency>>
    {
        Price(1.0, currency.into())
    }
}

impl Price {
    /// Price amount.
    #[inline(always)]
    pub fn amount(&self) -> f64 { self.0 }
    /// Currency used in the price.
    #[inline(always)]
    pub fn currency(&self) -> &Quasi<Currency> { &self.1 }
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

impl fmt::Debug for Price {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Hide the `Quasi` wrapper in debug representation.
        write!(fmt, "Price({:?}, {})", self.amount(),
            self.currency().as_ref()
                .map(|c| format!("{:?}", c))  // Currency::Foo
                .unwrap_or_else(|| format!("{:?}", self.currency())))  // "unknown-currency"
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


#[cfg(test)]
mod tests {
    use common::util::Quasi;
    use ::stashes::currency::Currency;
    use super::Price;

    #[test]
    fn display_for_known_currency() {
        let price = Price::new(1, Currency::ChaosOrb);
        assert_eq!("1 chaos", format!("{}", price));
    }

    #[test]
    fn display_for_unknown_currency() {
        let price = Price(42.0f64, Quasi::Substitute("orb-of-foo".into()));
        assert_eq!("42 orb-of-foo", format!("{}", price));
    }
}
