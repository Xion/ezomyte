//! Module implementing the types used for mod values.

use std::fmt;
use std::iter::FromIterator;

use smallvec::SmallVec;


/// Type of a mod parameter value
/// -- that is, the number that varies between occurrences of a mod on items.
pub type ModValue = f64;


/// Type for parameter values of a single mod.
///
/// Currently, no mod seems to have more than two values associated with it,
/// so this container holds zero, one, or two items.
#[derive(Clone, Default, Index, IndexMut)]
pub struct ModValues(SmallVec<[ModValue; 4]>);

impl ModValues {
    /// Create an empty set of mod values.
    #[inline]
    pub fn none() -> Self {
        ModValues(SmallVec::new())
    }

    /// Create a set containing a single mod value.
    pub fn one<V: Into<ModValue>>(value: V) -> Self {
        let mut values = SmallVec::new();
        values.push(value.into());
        ModValues(values)
    }

    /// Create a set containg two mod values.
    pub fn two<A, B>(v1: A, v2: B) -> Self
        where A: Into<ModValue>, B: Into<ModValue>
    {
        let mut values = SmallVec::new();
        values.push(v1.into());
        values.push(v2.into());
        ModValues(values)
    }
}
impl FromIterator<ModValue> for ModValues {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=ModValue>
    {
        ModValues(SmallVec::from_iter(iter))
    }
}

impl ModValues {
    /// Returns the number of mod values stored in this set.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over mod values in this set.
    #[inline]
    pub fn iter<'v>(&'v self) -> Box<Iterator<Item=ModValue> + 'v> {
        Box::new(self.0.iter().map(|v| *v))
    }
}

impl IntoIterator for ModValues {
    type Item = ModValue;
    type IntoIter = Box<Iterator<Item=ModValue>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter())
    }
}
impl<'v> IntoIterator for &'v ModValues {
    type Item = ModValue;
    type IntoIter = Box<Iterator<Item=ModValue> + 'v>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Debug for ModValues {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.0.iter()).finish()
    }
}
