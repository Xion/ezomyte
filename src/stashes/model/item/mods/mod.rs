//! Item mod (modifier).

                            mod id;
                            mod info;
#[cfg(feature = "mods_db")] mod database;
#[cfg(test)]                mod tests;
                            mod values;

pub use self::id::{ModId, ModType};
pub use self::info::ModInfo;
pub use self::values::{ModValue, ModValues};


use std::fmt;
use std::sync::Arc;


/// A single item mod occurrence.
#[derive(Clone)]
pub struct Mod {
    /// Type of the mod.
    type_: ModType,
    /// Original text of the mod.
    text: String,
    /// Parsed mod information & values.
    data: Option<(Arc<ModInfo>, ModValues)>,
}

impl Mod {
    /// Create `Mod` from given mod text that's found on an item.
    pub fn new<T: Into<String>>(type_: ModType, text: T) -> Self {
        let text = text.into();

        // Resolve mod text against item mod database if it's available.
        #[cfg(feature = "mods_db")]
        let data = database::ITEM_MODS.resolve(type_, &text);
        #[cfg(not(feature = "mods_db"))]
        let data = None;

        Mod{type_, text, data}
    }
}

impl Mod {
    /// Mod type (explicit, implicit, etc.).
    #[inline]
    pub fn mod_type(&self) -> ModType {
        self.type_
    }

    /// Information about the particular item mod.
    #[inline]
    #[cfg(feature = "mods_db")]
    pub fn info(&self) -> Option<&ModInfo> {
        self.data.as_ref().map(|&(ref mi, _)| &**mi)
    }

    /// Values associated with this particular mod's occurrence, if known.
    ///
    /// For example, if the mod is "+6% increased Attack Speed",
    /// its values include a single number 6.
    ///
    /// Note that not every number in the mod text is an actual mod value.
    /// A typical example is "Has 1 Abyssal Socket",
    /// where "1" is just a constant part of the mod text.
    #[inline]
    #[cfg(feature = "mods_db")]
    pub fn values(&self) -> Option<&ModValues> {
        self.data.as_ref().map(|&(_, ref mv)| mv)
    }

    /// Mod text as string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

impl fmt::Debug for Mod {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut ds = fmt.debug_struct("Mod");
        ds.field("type", &self.type_);
        match self.data {
            Some((ref mi, ref vs)) => {
                ds.field("text", &mi.text());
                ds.field("values", vs);
            }
            None => { ds.field("text", &self.text); }
        }
        ds.finish()
    }
}
