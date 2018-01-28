//! Deserializer for the item & stash labels.

use std::fmt;

use regex::Regex;
use serde::de::{self, Deserialize, Visitor};

use super::super::Label;
use super::util::deserialize;


const EXPECTING_MSG: &str = "item/stash label string";

// TODO: add tags in other PoE-supported languages
const EXACT_PRICE_TAGS: &[&str] = &["price", "ราคา" /* Thai */];
const NEGOTIABLE_PRICE_TAGS: &[&str] = &["b/o"];


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

    fn visit_str<E: de::Error>(self, mut v: &str) -> Result<Self::Value, E> {
        // TODO: consider providing a FromStr implementation for Label

        if v.is_empty() {
            return Ok(Label::Empty);
        }
        if !v.starts_with("~") {
            return Ok(Label::Cosmetic(v.to_owned()));
        }

        // TODO: it would seem you can actually have a price AND a cosmetic text;
        // here, we're stripping the latter to parse the price correctly,
        // but we could retain it if we changed the format of Label
        lazy_static! {
            static ref SEP_RE: Regex = Regex::new(r#"\s+[/|]"#).unwrap();
        }
        if let Some(m) = SEP_RE.find(v) {
            v = v[..m.start()].trim_right();
        }

        // Find the label type tag that appears after the tilde before the first whitespace.
        let (tag, rest) = match v.find(|c: char| c.is_whitespace()) {
            Some(idx) if idx <= v.len() => (&v[1..idx], v[idx + 1..].trim_left()),
            _ => return Err(de::Error::custom(format!("malformed label: {}", v))),
        };

        // XXX: some asshats think it's funny to name their stashes something like
        // "~b/o offer", which of course breaks the price parsing below;
        // we probably need to introduce something like Label::Malformed to accommodate that
        if EXACT_PRICE_TAGS.contains(&tag) {
            deserialize(rest).map(Label::ExactPrice)
        } else if NEGOTIABLE_PRICE_TAGS.contains(&tag) {
            deserialize(rest).map(Label::NegotiablePrice)
        } else {
            Ok(Label::Unknown(tag.into(), rest.into()))
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

    #[test]
    fn unknown() {
        assert_eq!(
            Label::Unknown("key".into(), "value".into()),
            from_value(json!("~key value")).unwrap());
    }
}
