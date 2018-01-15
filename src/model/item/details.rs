//! Item details which are specific to a particular kind of an item.

use super::attributes::{Experience, Mod};


/// Details of the particular items, if known.
#[derive(Debug)]
pub enum ItemDetails {
    /// An unidentified item. No details available.
    Unidentified,
    /// Skill gem.
    Gem {
        /// Current level of the gem.
        ///
        /// Gems start at level 1.
        /// Standalone gems (not in gear) currently shouldn't have levels above 21
        /// (at least outside of glitches/bugs).
        level: u32,
        /// The amount of experience a gem has and requires for the next level.
        experience: Experience,
    },
    /// Flask item.
    Flask {
        /// Utility mods of the flask.
        ///
        /// These are typically the on-use effect (like bleed/freeze/etc. removal),
        /// mods affecting charges, etc..
        mods: Vec<Mod>,
    },
    /// Item with mods.
    ///
    /// This includes most items, such as those in gear slots,
    /// jewels, maps, and so on.
    Mods {
        /// Implicit mods an item has
        /// (those displayed in navy color above a horizontal line in the UI).
        ///
        /// Currently, PoE only supports a single implicit mod,
        /// but this may change in the future.
        implicit: Vec<Mod>,

        /// Enchantments an item has
        /// (i.e. mods displayed in light blue color above a horizontal line in the UI).
        ///
        /// Currently, PoE only supports a single enchantment,
        /// but this may change in the future.
        enchants: Vec<Mod>,

        /// Explicit mods of an item
        /// (those displayed in navy color in the main item pane).
        ///
        /// Note that these the mods which are visible in the UI,
        /// as opposed to *affixes* (prefixes & suffixes) which cannot be reliably
        /// extrapolated from mods.
        /// The practical consequence is that there may be more than 6 mods
        /// which would seemingly contradict the "3 prefixes + 3 suffixes" rule
        /// (due to the so-called hybrid affixes that result in multiple mods).
        explicit: Vec<Mod>,

        /// Crafted mods on an item
        /// (those displayed in light blue color in the main item pane, below explicit mods).
        crafted: Vec<Mod>,
    },
}
