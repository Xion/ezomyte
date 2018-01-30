//! Label on an item or stash.

use std::borrow::Cow;
use std::fmt;

use super::price::Price;


/// Label (note) for an item or stash tab.
///
/// Those labels can have special meaning in PoE
/// and indicate e.g. prices applicable to all items in the tab.
#[derive(Clone, Debug, PartialEq)]
pub enum Label {
    /// Empty label.
    Empty,
    /// Cosmetic name, without any other meaning.
    Cosmetic(String),
    /// Exact price ("~price $N $CURR").
    ExactPrice(Price),
    /// Negotiable price ("~b/o $N $CURR").
    NegotiablePrice(Price),
    /// Unrecognized combination of tilde-prefixed tag and value.
    Unknown(String, String),
}

impl Default for Label {
    fn default() -> Self {
        Label::Empty
    }
}

impl Label {
    /// Return whether the label is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match *self { Label::Empty => true, _ => false }
    }

    /// Return a possible price in the label.
    ///
    /// This doesn't distinguish between the exact price and negotiable/buyout price.
    pub fn price(&self) -> Option<Price> {
        match *self {
            Label::ExactPrice(p) => Some(p),
            Label::NegotiablePrice(p) => Some(p),
            _ => None,
        }
    }

    /// Return the exact `Price` specified in this `Label`, if any.
    #[inline]
    pub fn exact_price(&self) -> Option<Price> {
        match *self {
            Label::ExactPrice(p) => Some(p),
            _ => None,
        }
    }

    /// Return the negotiable (buyout) `Price` specified in this `Label`, if any.
    #[inline]
    pub fn negotiable_price(&self) -> Option<Price> {
        match *self {
            Label::NegotiablePrice(p) => Some(p),
            _ => None,
        }
    }

    /// Return the tilde-prefixed tag (like "b/o") from the original label
    /// (without the actual tilde prefix).
    pub fn tag(&self) -> Option<&str> {
        match *self {
            Label::ExactPrice(_) => Some("price"),
            Label::NegotiablePrice(_) => Some("b/o"),
            Label::Unknown(ref t, _) => Some(t),
            _ => None,
        }
    }

    /// Return a possible string value (usually the price)
    /// that's associated with the label's `tag`.
    pub fn value(&self) -> Option<Cow<str>> {
        match *self {
            Label::ExactPrice(ref p) |
            Label::NegotiablePrice(ref p) => Some(format!("{}", p).into()),
            Label::Unknown(_, ref v) => Some(v.as_str().into()),
            _ => None,
        }
    }

    /// Return a possible cosmetic note in this `Label`.
    pub fn note(&self) -> Option<&str> {
        match *self {
            Label::Empty => Some(""),
            Label::Cosmetic(ref s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Label::Empty => write!(fmt, ""),
            Label::Cosmetic(ref s) => write!(fmt, "{}", s),
            Label::ExactPrice(p) => write!(fmt, "~price {}", p),
            Label::NegotiablePrice(p) => write!(fmt, "~b/o {}", p),
            Label::Unknown(ref t, ref v) => write!(fmt, "~{} {}", t, v),
        }
    }
}
