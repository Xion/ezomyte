//! League type.

use std::borrow::Cow;
use std::fmt;


/// League in Path of Exile.
///
/// For our purposes, we're only distinguishing permanent & temporary leagues,
/// without making note of a particular temporary league name (like "Harbinger" vs "Abyss").
#[derive(Clone)]  // TODO: impl Hash that's basically season.or_else(bools)
pub struct League {
    /// Season name, if available.
    pub(super) season: Option<String>,
    /// Whether it's a temporary (seasonal) league as opposed to permanent one.
    temporary: bool,
    /// Whether it's a hardcore (permadeath) league as opposed to a softcore one.
    hardcore: bool,
    /// Whether it's a solo self-found league.
    ssf: bool,  // btw
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
        League { season: None, temporary: false, hardcore: false, ssf: false }
    }

    /// Hardcore league (permanent non-SSF).
    #[inline]
    pub fn hardcore() -> Self {
        League { season: None, temporary: false, hardcore: true, ssf: false }
    }

    /// Temporary league (softcore non-SSF).
    #[inline]
    pub fn temporary() -> Self {
        League { season: None, temporary: true, hardcore: false, ssf: false }
    }

    /// Temporary hardcore league (non-SSF).
    #[inline]
    pub fn temporary_hardcore() -> Self {
        League { season: None, temporary: true, hardcore: true, ssf: false }
    }

    /// SSF league (permanent softcore).
    #[inline]
    pub fn ssf() -> Self {
        League { season: None, temporary: false, hardcore: false, ssf: true }
    }

    /// Hardcore SSF league (permanent).
    #[inline]
    pub fn hardcore_ssf() -> Self {
        League { season: None, temporary: false, hardcore: true, ssf: true }
    }

    /// Temporary SSF league (softcore).
    #[inline]
    pub fn temporary_ssf() -> Self {
        League { season: None, temporary: true, hardcore: false, ssf: true }
    }

    /// Temporary hardcore SSF league.
    #[inline]
    pub fn temporary_hardcore_ssf() -> Self {
        League { season: None, temporary: true, hardcore: true, ssf: true }
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

impl League {
    /// Name of the league's season, if known.
    ///
    /// A season is basically the unique identifying part of all temporary league names.
    /// The "Abyss" season, for example, consists of leagues called
    /// "Abyss" (softcore non-SSF), "Hardcore Abyss", etc.
    ///
    /// Permanent leagues are not part of any season.
    #[inline(always)]
    pub fn season(&self) -> Option<&str> {
        self.season.as_ref().map(|s| s.as_str())
    }

    /// Whether this is a temporary league.
    #[inline(always)]
    pub fn is_temporary(&self) -> bool { self.temporary }
    /// Whether this is a hardcore league.
    #[inline(always)]
    pub fn is_hardcore(&self) -> bool { self.hardcore }
    /// Whether this is a solo self-found league.
    #[inline(always)]
    pub fn is_ssf(&self) -> bool { self.ssf }
}

impl PartialEq for League {
    fn eq(&self, other: &League) -> bool {
        // Don't take season name into account when comparing for equality.
        self.temporary == other.temporary
        && self.hardcore == other.hardcore
        && self.ssf == other.ssf
    }
}

impl fmt::Debug for League {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let moniker = match (self.temporary, self.hardcore, self.ssf) {
            (false, false, false) => "standard",
            (false, true, false) => "hardcore",
            (true, false, false) => "temporary",
            (true, true, false) => "temporary_hardcore",
            (false, false, true) => "ssf",
            (false, true, true) => "hardcore_ssf",
            (true, false, true) => "temporary_ssf",
            (true, true, true) => "temporary_hardcore_ssf",
        };
        match self.season() {
            Some(s) => write!(fmt, "League{{season: \"{}\", ..League::{}()}}", s, moniker),
            None => write!(fmt, "League::{}()", moniker),
        }
    }
}

impl fmt::Display for League {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name: Cow<str> = match (self.temporary, self.hardcore, self.ssf) {
            (false, false, false) => "Standard".into(),
            (false, true, false) => "Hardcore".into(),
            (true, false, false) => self.season().unwrap_or("Temporary").into(),
            (true, true, false) => self.season()
                .map(|s| format!("Hardcore {}", s).into())
                .unwrap_or("Temporary Hardcore".into()),
            (false, false, true) => "SSF Standard".into(),
            (false, true, true) => "SSF Hardcore".into(),
            (true, false, true) =>  self.season()
                .map(|s| format!("SSF {}", s).into())
                .unwrap_or("Temporary SSF".into()),
            (true, true, true) => self.season()
                .map(|s| format!("SSF {} HC", s).into())
                .unwrap_or("Temporary Hardcore SSF".into()),
        };
        write!(fmt, "{}", name)
    }
}
