//! Debug utilities.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use conv::errors::NoError;


/// A dummy type that has specific `Debug` impl given by the `str`.
///
/// This is useful when implementing `Debug` for other types using the standard
/// helpers like `debug_struct`.
pub struct ExplicitDebug<'d>(Cow<'d, str>);

impl<'d> ExplicitDebug<'d> {
    #[inline]
    pub fn new(s: Cow<'d, str>) -> Self {
        ExplicitDebug(s)
    }
}

// Conversions from string (borrow/owned).
impl<'d> From<Cow<'d, str>> for ExplicitDebug<'d> {
    fn from(s: Cow<'d, str>) -> Self {
        ExplicitDebug(s)
    }
}
impl<'d> From<&'d str> for ExplicitDebug<'d> {
    fn from(s: &'d str) -> Self {
        ExplicitDebug(s.into())
    }
}
impl<'d> From<String> for ExplicitDebug<'d> {
    fn from(s: String) -> Self {
        ExplicitDebug(s.into())
    }
}
impl<'d> FromStr for ExplicitDebug<'d> {
    type Err = NoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExplicitDebug(s.to_owned().into()))
    }
}

impl<'d> fmt::Debug for ExplicitDebug<'d> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}
