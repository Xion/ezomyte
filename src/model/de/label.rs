//! Deserializer for the item & stash labels.

use std::fmt;

use serde::de::{self, Deserialize, Visitor};

use super::super::Label;
use super::util::deserialize;


const EXPECTING_MSG: &str = "item/stash label string";

impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        deserializer.deserialize_str(LabelVisitor)
    }
}

struct LabelVisitor;
impl<'de> Visitor<'de> for LabelVisitor {
    type Value = Label;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", EXPECTING_MSG)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing a FromStr implementation for Label
        const EXACT_PRICE_PREFIX: &str = "~price";
        const NEGOTIABLE_PRICE_PREFIX: &str = "~b/o";

        if v.is_empty() {
            return Err(de::Error::invalid_length(0, &"non-empty string"));
        }

        if v.starts_with(EXACT_PRICE_PREFIX) {
            let price = v.trim_left_matches(EXACT_PRICE_PREFIX).trim_left();
            deserialize(price).map(Label::ExactPrice)
        } else if v.starts_with(NEGOTIABLE_PRICE_PREFIX) {
            let price = v.trim_left_matches(NEGOTIABLE_PRICE_PREFIX).trim_left();
            deserialize(price).map(Label::NegotiablePrice)
        } else {
            // TODO: maybe we should store "~$UNKOWN $STUFF" as another Label variant?
            if v.starts_with("~") {
                let tag = v.trim_left_matches("~").split_whitespace().next().unwrap_or("");
                Err(de::Error::custom(format!("unknown label tag: ~{}", tag)))
            } else {
                Ok(Label::Cosmetic(v.to_owned()))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use model::{Currency, Label, Price};

    #[test]
    fn cosmetic() {
        assert_eq!(Label::Cosmetic("foo".into()), from_value(json!("foo")).unwrap());
        // Needs the tilde to be interpreted as price.
        assert_eq!(
            Label::Cosmetic("price 1 chaos".into()),
            from_value(json!("price 1 chaos")).unwrap());
    }

    #[test]
    fn exact_price() {
        assert_eq!(
            Label::ExactPrice(Price::one(Currency::OrbOfAlchemy)),
            from_value(json!("~price 1 alch")).unwrap());
        assert_eq!(
            Label::ExactPrice(Price::new(25, Currency::ChaosOrb)),
            from_value(json!("~price 25 chaos")).unwrap());
    }

    #[test]
    fn negotiable_price() {
        assert_eq!(
            Label::NegotiablePrice(Price::new(50, Currency::ChaosOrb)),
            from_value(json!("~b/o 50 chaos")).unwrap());
        assert_eq!(
            Label::NegotiablePrice(Price::new(10, Currency::ExaltedOrb)),
            from_value(json!("~b/o 10 exa")).unwrap());
    }
}
