//! League type.

use std::fmt;


/// League in Path of Exile.
///
/// For our purposes, we're only distinguishing permanent & temporary leagues,
/// without making note of a particular temporary league name (like "Harbinger" vs "Abyss").
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct League {
    /// Whether it's a temporary (seasonal) league as opposed to permanent one.
    pub temporary: bool,
    /// Whether it's a hardcore (permadeath) league as opposed to a softcore one.
    pub hardcore: bool,
    /// Whether it's a solo self-found league.
    pub ssf: bool,  // btw
    // TODO: races/void leagues
}

impl Default for League {
    fn default() -> Self {
        League::standard()
    }
}

// Constructors.
impl League {
    /// Standard league (permanent softcore non-SSF).
    #[inline]
    pub fn standard() -> Self {
        League { temporary: false, hardcore: false, ssf: false }
    }

    /// Hardcore league (permanent non-SSF).
    #[inline]
    pub fn hardcore() -> Self {
        League { temporary: false, hardcore: true, ssf: false }
    }

    /// Temporary league (softcore non-SSF).
    #[inline]
    pub fn temporary() -> Self {
        League { temporary: true, hardcore: false, ssf: false }
    }

    /// Temporary hardcore league (non-SSF).
    #[inline]
    pub fn temporary_hardcore() -> Self {
        League { temporary: true, hardcore: true, ssf: false }
    }

    /// SSF league (permanent softcore).
    #[inline]
    pub fn ssf() -> Self {
        League { temporary: false, hardcore: false, ssf: true }
    }

    /// Hardcore SSF league (permanent).
    #[inline]
    pub fn hardcore_ssf() -> Self {
        League { temporary: false, hardcore: true, ssf: true }
    }

    /// Temporary SSF league (softcore).
    #[inline]
    pub fn temporary_ssf() -> Self {
        League { temporary: true, hardcore: false, ssf: true }
    }

    /// Temporary hardcore SSF league.
    #[inline]
    pub fn temporary_hardcore_ssf() -> Self {
        League { temporary: true, hardcore: true, ssf: true }
    }
}
// Constructor aliases.
impl League {
    /// Alias for `standard`.
    #[inline]
    pub fn softcore() -> Self { Self::standard() }
    /// Alias for `standard`.
    #[inline]
    pub fn sc() -> Self { Self::standard() }
    /// Alias for `hardcore`.
    #[inline]
    pub fn hc() -> Self { Self::hardcore() }
    /// Alias for `temporary`.
    #[inline]
    pub fn temp() -> Self { Self::temporary() }
    /// Alias for `temporary`.
    #[inline]
    pub fn temp_sc() -> Self { Self::temporary() }
    /// Alias for `temporary_hardcore`.
    #[inline]
    pub fn temp_hc() -> Self { Self::temporary_hardcore() }
    /// Alias for `hardcore_ssf`.
    #[inline]
    pub fn hc_ssf() -> Self { Self::hardcore_ssf() }
    /// Alias for `temporary_ssf`.
    #[inline]
    pub fn temp_ssf() -> Self { Self::temporary_ssf() }
    /// Alias for `temporary_hardcore_ssf`.
    #[inline]
    pub fn temp_hc_ssf() -> Self { Self::temporary_hardcore_ssf() }
}

impl fmt::Debug for League {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let repr = match (self.temporary, self.hardcore, self.ssf) {
            (false, false, false) => "standard",
            (false, true, false) => "hardcore",
            (true, false, false) => "temporary",
            (true, true, false) => "temporary_hardcore",
            (false, false, true) => "ssf",
            (false, true, true) => "hardcore_ssf",
            (true, false, true) => "temporary_ssf",
            (true, true, true) => "temporary_hardcore_ssf",
        };
        write!(fmt, "League::{}()", repr)
    }
}

impl fmt::Display for League {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match (self.temporary, self.hardcore, self.ssf) {
            (false, false, false) => "Standard",
            (false, true, false) => "Hardcore",
            (true, false, false) => "Temporary",
            (true, true, false) => "Temporary Hardcore",
            (false, false, true) => "SSF",
            (false, true, true) => "Hardcore SSF",
            (true, false, true) => "Temporary SSF",
            (true, true, true) => "Temporary Hardcore SSF",
        };
        write!(fmt, "{}", name)
    }
}
