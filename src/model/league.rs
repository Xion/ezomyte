//! League type.

use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use itertools::Itertools;


/// Main league in Path of Exile.
///
/// Those are the leagues you can select (or used to be able to select, for historical data)
/// in the character screen.
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


/// Error while converting a string league name to `League` type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseLeagueError {
    /// Error for when the league name is empty.
    Empty,
    /// Error for when the league name is malformed.
    Malformed,
}
impl Error for ParseLeagueError {
    fn description(&self) -> &str { "error parsing league name" }
    fn cause(&self) -> Option<&Error> { None }
}
impl fmt::Display for ParseLeagueError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", match *self {
            ParseLeagueError::Empty => "got an empty league name",
            ParseLeagueError::Malformed => "got a malformed league name",
        })
    }
}

impl FromStr for League {
    type Err = ParseLeagueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const COMMON_WORDS: &[&str] = &["Standard", "Hardcore", "HC", "SSF"];

        // Do some basic sanity checks on the league name to see if it's well-formed.
        if s.is_empty() {
            return Err(ParseLeagueError::Empty);
        }
        let has_valid_words = s.split_whitespace().all(|w| {
            COMMON_WORDS.contains(&w) || {
                // Other words are season names which must be capitalized.
                let first_upper = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                let rest_lower = w.chars().skip(1).all(|c| c.is_lowercase());
                w.len() > 1 && first_upper && rest_lower
            }
        });
        if !has_valid_words {
            return Err(ParseLeagueError::Malformed);
        }

        // Extract league's attributes (SC/HC, SSF?) and its season name.
        let mut league = match s {
            "Standard" => League::standard(),
            "Hardcore" => League::hardcore(),
            "SSF Standard" => League::ssf(),
            "SSF Hardcore" => League::hardcore_ssf(),
            s => {
                let hardcore = s.contains("Hardcore") || s.contains("HC");
                let ssf = s.contains("SSF");
                match (hardcore, ssf) {
                    (false, false) => League::temporary(),
                    (false, true) => League::temporary_ssf(),
                    (true, false) => League::temporary_hardcore(),
                    (true, true) => League::temporary_hardcore_ssf(),
                }
            }
        };
        league.season = {
            let season = s.split_whitespace().filter(|w| !COMMON_WORDS.contains(&w)).join("");
            if season.is_empty() { None } else { Some(season) }
        };
        Ok(league)
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use model::League;

    #[test]
    fn permanent_leagues() {
        assert_eq!(League::standard(), League::from_str("Standard").unwrap());
        assert_eq!(League::hardcore(), League::from_str("Hardcore").unwrap());
        assert_eq!(League::ssf(), League::from_str("SSF Standard").unwrap());
        assert_eq!(League::hardcore_ssf(), League::from_str("SSF Hardcore").unwrap());
    }

    #[test]
    fn abyss_leagues() {
        assert_eq!(League::temporary(), League::from_str("Abyss").unwrap());
        assert_eq!(League::temp_hc(), League::from_str("Hardcore Abyss").unwrap());
        assert_eq!(League::temp_ssf(), League::from_str("SSF Abyss").unwrap());
        assert_eq!(League::temp_hc_ssf(), League::from_str("SSF Abyss HC").unwrap());
    }

    #[test]
    fn harbinger_leagues() {
        assert_eq!(League::temporary(), League::from_str("Harbinger").unwrap());
        assert_eq!(League::temp_hc(), League::from_str("Hardcore Harbinger").unwrap());
        assert_eq!(League::temp_ssf(), League::from_str("SSF Harbinger").unwrap());
        assert_eq!(League::temp_hc_ssf(), League::from_str("SSF Harbinger HC").unwrap());
    }

    // TODO: all the other past leagues
}
