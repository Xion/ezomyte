//! Item mod (modifier).

mod id;
mod database;

pub use self::database::ModInfo;
pub use self::id::{ModId, ModType};


use std::fmt;
use std::sync::Arc;

use smallvec::SmallVec;


/// A single item mod occurrence.
#[derive(Clone)]
pub struct Mod {
    /// Original text of the mod.
    text: String,
    /// Parsed mod information & values.
    data: Option<(Arc<ModInfo>, ModValues)>,
}

impl Mod {
    /// Create `Mod` from given mod text that's found on an item.
    pub fn new<T: Into<String>>(type_: ModType, text: T) -> Self {
        let text = text.into();
        let data = database::ITEM_MODS.lookup(type_, &text);
        Mod{text, data}
    }
}

impl Mod {
    /// Mod text as string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

impl fmt::Debug for Mod {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // TODO: better Debug, maybe a standard derived one even
        write!(fmt, "Mod::new({:?})", self.text)
    }
}


/// Type of a mod parameter value
/// -- that is, the number that varies between occurrences of a mod on items.
pub type ModValue = f64;

/// Type for parameter values of a single mod.
///
/// Currently, no mod seems to have more than two values associated with it,
/// so this container holds zero, one, or two items.
pub type ModValues = SmallVec<[ModValue; 4]>;
