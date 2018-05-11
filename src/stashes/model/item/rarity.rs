//! Item rarity.


macro_attr! {
    /// Rarity of an item.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq,
             IterVariants!(Rarities))]
    pub enum Rarity {
        /// Normal ("white") item.
        ///
        /// This also includes items which aren't gear, like gems or divination cards.
        Normal,
        /// Magic ("blue") item.
        Magic,
        /// Rare ("yellow") item.
        Rare,
        /// Unique item.
        Unique,
    }
}

impl Default for Rarity {
    fn default() -> Self {
        Rarity::Normal
    }
}
