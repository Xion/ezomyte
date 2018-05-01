//! Module for the PoE leagues' API.

use std::ops::Deref;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use conv::TryFrom;
use futures::{future, Future, stream, Stream as StdStream};
use hyper::client::Connect;
use regex::Regex;
use serde::de::{self, Deserialize, Deserializer};

use ::common::{League, ParseLeagueError};
use ::{Client, Stream};


/// Interface for accessing league information.
#[derive(Clone, Debug)]
pub struct Leagues<C>
    where C: Clone + Connect
{
    client: Client<C>,
}

impl<C: Clone + Connect> Leagues<C> {
    #[inline]
    pub(crate) fn new(client: Client<C>) -> Self {
        Leagues { client }
    }
}

impl<C: Clone + Connect> Leagues<C> {
    /// Return a stream of all league infos.
    #[inline]
    pub fn all(&self) -> Stream<LeagueInfo> {
        self.get_leagues_stream(Type::All)
    }

    /// Return a stream of the "main" leagues.
    ///
    /// These are the ones that appear in the character selection screen in game.
    #[inline]
    pub fn main(&self) -> Stream<LeagueInfo> {
        self.get_leagues_stream(Type::Main)
    }

    /// Return a stream of special/event leagues (races, etc.).
    #[inline]
    pub fn event(&self) -> Stream<LeagueInfo> {
        self.get_leagues_stream(Type::Event)
    }

    /// Return a stream of leagues in a particular season.
    ///
    /// Season name is something like "Abyss", "Harbinger", "Breach", etc.
    #[inline]
    pub fn in_season<S: Into<String>>(&self, season: S) -> Stream<LeagueInfo> {
        self.get_leagues_stream(Type::Season(season.into()))
    }

    fn get_leagues_stream(&self, type_: Type) -> Stream<LeagueInfo> {
        /// Enum for managing the state machine of the resulting Stream.
        enum State {
            Start(Type),
            Next{type_: Type, offset: usize},
            End,
        }

        let this = self.clone();
        Box::new(
            stream::unfold(State::Start(type_), move |state| {
                let (type_, offset) = match state {
                    State::Start(type_) => (type_, None),
                    State::Next{type_, offset} => (type_, Some(offset)),
                    State::End => return None,
                };
                let base_url = {
                    let mut season = None;
                    let type_param = match type_ {
                        Type::All => "all",
                        Type::Main => "main",
                        Type::Event => "event",
                        Type::Season(ref s) => { season = Some(s.clone()); "season" }
                    };
                    // TODO: consider using the `url` crate
                    // TODO: support non-compact data format, too
                    format!("{}?type={}&compact=1{}", LEAGUES_URL, type_param,
                        season.map(|s| format!("&season={}", s)).unwrap_or_else(String::new)).into()
                };
                let url = match offset {
                    Some(off) => format!("{}&offset={}", base_url, off),
                    None => base_url,
                };
                Some(this.client.get(url).and_then(move |resp: LeaguesResponse| {
                    let count = resp.len();
                    let next_offset = offset.unwrap_or(0) + count;
                    let leagues = resp.into_iter()
                        // API output contains some testing/alpha/broken/abandonded/etc. leagues
                        // which don't have the "start_at" field filled in, so we filter them out.
                        .filter(|l| l.start_at.is_some())
                        .filter_map(|l| {
                            let repr = format!("{:?}", l);
                            LeagueInfo::try_from(l).map_err(|e| {
                                warn!("Failed to parse league {}: {}", repr, e); e
                            }).ok()
                        });
                    let next_state = if count == MAX_COMPACT_ITEMS {
                        State::Next{type_, offset: next_offset}
                    } else {
                        State::End
                    };
                    future::ok((stream::iter_ok(leagues), next_state))
                }))
            })
            .flatten()
        )
    }
}

const LEAGUES_URL: &str = "/leagues";
const MAX_COMPACT_ITEMS: usize = 230;  // as per API docs


/// Information about a single league, as retrieved from the PoE API.
///
/// *Note*: This type `Deref`s to the `League` type which contains information
/// about the particular kind of the league: hardcode, SSF, etc.
#[derive(Debug)]
pub struct LeagueInfo {
    /// ID of the league as returned from the API.
    /// It typically includes the season name.
    pub id: String,
    league: League,
    /// When has this league started, or will start.
    pub start_at: DateTime<Utc>,
    /// When has this league finished, or will finish, if known.
    pub end_at: Option<DateTime<Utc>>,
}

impl Deref for LeagueInfo {
    type Target = League;
    fn deref(&self) -> &Self::Target {
        &self.league
    }
}

impl TryFrom<RawLeagueInfo> for LeagueInfo {
    type Err = ParseLeagueError;

    fn try_from(input: RawLeagueInfo) -> Result<Self, Self::Err> {
        lazy_static! {
            /// Regex to scrub league names like "Medallion (MDS008)"
            /// which are returned from the API.
            static ref JUNK_RE: Regex = Regex::new(r#"\([^)]+\)"#).unwrap();
        }
        let league = League::from_str(JUNK_RE.replace(&input.id, "").trim())?;
        Ok(LeagueInfo {
            id: input.id,
            league: league,
            start_at: input.start_at.expect("RawLeagueInfo::start_at"),
            end_at: input.end_at,
        })
    }
}


/// Type of the league stream to return.
/// For documentation, see https://www.pathofexile.com/developer/docs/api-resource-leagues.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Type {
    /// All leagues ever.
    All,
    /// Main leagues (currently selectable in the character screen).
    Main,
    /// Special event leagues.
    Event,
    /// Leagues within a particular season (like "Harbinger" or "Abyss").
    Season(String),
}


/// Response from the leagues' API endpoint.
type LeaguesResponse = Vec<RawLeagueInfo>;

/// A single league info, as obtained from the API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLeagueInfo {
    id: String,
    url: Option<String>,
    #[serde(deserialize_with = "deserialize_opt_datetime")]
    start_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_opt_datetime")]
    end_at: Option<DateTime<Utc>>,
}

/// Deserialize an optional UTC DateTime stored as RFC 3339 string, or as null.
fn deserialize_opt_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where D: Deserializer<'de>
{
    let opt_s: Option<String> = Deserialize::deserialize(deserializer)?;
    match opt_s {
        Some(ref s) => {
            let fixed_datetime: DateTime<_> = DateTime::parse_from_rfc3339(s).map_err(|e| {
                de::Error::custom(format!("failed to parse string as RFC3339 datetime: {}", e))
            })?;
            if fixed_datetime.offset().local_minus_utc() != 0 {
                return Err(de::Error::custom(format!("expected UTC datetime, got one with offset {}",
                    fixed_datetime.offset())));
            }
            Ok(Some(fixed_datetime.with_timezone(&Utc)))
        }
        None => Ok(None),
    }
}
