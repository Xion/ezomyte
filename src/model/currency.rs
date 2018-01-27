//! Currency types.


/// Currency item used for trading.
///
/// This doesn't include discontinued currencies (like Eternal Orbs),
/// league-specific currencies (like Harbinger Orbs),
/// or extremely common/rare consumable items (like scrolls or mirrors)
/// because they cannot be used when pricing items in stashes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Currency {
    // TODO: review the exact position where the SilverCoin should be
    // (should be the same as it appears in the in-game UI)
    #[serde(rename = "alt")]
    OrbOfAlteration,
    #[serde(rename = "fuse")]
    OrbOfFusing,
    #[serde(rename = "alch")]
    OrbOfAlchemy,
    #[serde(rename = "silver")]
    SilverCoin,
    #[serde(rename = "gcp")]
    GemcuttersPrism,
    #[serde(rename = "exa")]
    ExaltedOrb,
    #[serde(rename = "chrom")]
    ChromaticOrb,
    #[serde(rename = "jew")]
    JewellersOrb,
    #[serde(rename = "chance")]
    OrbOfChance,
    #[serde(rename = "chaos")]
    ChaosOrb,
    #[serde(rename = "chisel")]
    CartographersChisel,
    #[serde(rename = "scour")]
    OrbOfScouring,
    #[serde(rename = "blessed")]
    BlessedOrb,
    #[serde(rename = "regret")]
    OrbOfRegret,
    #[serde(rename = "regal")]
    RegalOrb,
    #[serde(rename = "divine")]
    DivineOrb,
    #[serde(rename = "vaal")]
    VaalOrb,

    // TODO: we sometimes actually encounter things like "ancient-orb" or "mirror"
    // in the stash dataset; it's unclear what to with them, because it'd be nice
    // to have them as enum variants, too, but then the number of variants would grow
    // without bound, so probably we'll just introduce a catchall Currency::Other(String)
}


// Commonly used nicknames for the currencies

pub const ALT: Currency = Currency::OrbOfAlteration;
pub const FUSE: Currency = Currency::OrbOfFusing;
pub const FUSING: Currency = Currency::OrbOfFusing;
pub const ALCH: Currency = Currency::OrbOfAlchemy;
pub const SILVER: Currency = Currency::SilverCoin;
pub const GCP: Currency = Currency::GemcuttersPrism;
pub const EXALT: Currency = Currency::ExaltedOrb;
pub const CHROME: Currency = Currency::ChromaticOrb;
pub const JEW: Currency = Currency::JewellersOrb;
pub const CHANCE: Currency = Currency::OrbOfChance;
pub const CHAOS: Currency = Currency::ChaosOrb;
pub const CHISEL: Currency = Currency::CartographersChisel;
pub const SCOUR: Currency = Currency::OrbOfScouring;
pub const BLESS: Currency = Currency::BlessedOrb;
pub const REGAL: Currency = Currency::RegalOrb;
pub const REGRET: Currency = Currency::OrbOfRegret;
pub const DIVINE: Currency = Currency::DivineOrb;
pub const VAAL: Currency = Currency::VaalOrb;
