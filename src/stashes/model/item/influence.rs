//! Influence that War for the Atlas has over the item.


macro_attr! {
    /// War for the Atlas influence on the item.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq,
             IterVariants!(Influences))]
    pub enum Influence {
        /// The item base has been influenced by the Elder.
        /// This is colloquially referred to as an "elder item".
        Elder,
        /// This item base has been influenced by the Shaper.
        /// This is colloquially referred to as a "shaped item".
        Shaper,
    }
}
