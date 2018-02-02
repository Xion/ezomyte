//! Module for the PoE league API.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use conv::TryFrom;
use futures::{future, Future, stream, Stream as StdStream};
use hyper::client::Connect;
use serde::de::{self, Deserializer, Visitor};

use ::{Client, League, ParseLeagueError};
use super::{Batched, Stream};


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
    fn get_leagues_stream(&self, type_: Type) -> Stream<Batched<LeagueInfo, usize>> {
        // XXX: the State enum, like in Stashes, to detect the end correctly
        let this = self.clone();
        Box::new(
            stream::unfold(None, move |offset: Option<usize>| {
                // TODO: consider using the `url` crate
                let base_url = {
                    let mut season = None;
                    let type_param = match type_ {
                        Type::All => "all",
                        Type::Main => "main",
                        Type::Event => "event",
                        Type::Season(ref s) => { season = Some(s.clone()); "season" }
                    };
                    // TODO: support non-compact data format, too
                    format!("{}?type={}&compact=1{}", LEAGUES_URL, type_param,
                        season.map(|s| format!("&season={}", s)).unwrap_or_else(String::new)).into()
                };
                let url = match offset {
                    Some(off) => format!("{}&offset={}", base_url, off),
                    None => base_url,
                };
                Some(this.client.get(url).and_then(move |resp: LeaguesResponse| {
                    let next_offset = offset.unwrap_or(0) + resp.leagues.len();
                    let leagues = resp.leagues.into_iter()
                        // API output contains some testing/alpha/broken/abandonded/etc. leagues
                        // which don't have the "start_at" field filled in, so we filter them out.
                        .filter(|l| l.start_at.is_some())
                        // TODO: report errors from this conversion rather than skipping the leagues
                        .filter_map(|l| LeagueInfo::try_from(l).ok())
                        .map(move |entry| Batched{
                            entry, curr_token: offset, next_token: Some(next_offset),
                        });
                    future::ok((stream::iter_ok(leagues), Some(next_offset)))
                }))
            })
            .flatten()
        )
    }
}


/// Information about a single league, as retrieved from the PoE API.
#[derive(Debug)]
pub struct LeagueInfo {
    id: String,
    league: League,
    start_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
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
        let league = League::from_str(&input.id)?;  // XXX: we probably need to remove "([^)]+)" from this
        Ok(LeagueInfo {
            id: input.id,
            league: league,
            start_at: input.start_at.expect("RawLeagueInfo::start_at"),
            end_at: input.end_at,
        })
    }
}


// Utility code

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

const LEAGUES_URL: &str = "/leagues";

/// Response from the leagues' API endpoint.
#[derive(Debug, Deserialize)]
struct LeaguesResponse {
    leagues: Vec<RawLeagueInfo>,
}

/// A single league info, as obtained from the API.
#[derive(Debug, Deserialize)]
struct RawLeagueInfo {
    id: String,
    url: Option<String>,
    #[serde(deserialize_with = "deserialize_opt_datetime")]
    start_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_opt_datetime")]
    end_at: Option<DateTime<Utc>>,
}

/// Deserialize an optional DateTime stored as RFC 3339 string, or as null.
fn deserialize_opt_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where D: Deserializer<'de>
{
    deserializer.deserialize_option(OptionDateTimeVisitor)
}
struct OptionDateTimeVisitor;
impl<'de> Visitor<'de> for OptionDateTimeVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", "UTC datetime in RFC 3339 format, or null")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let fixed_datetime: DateTime<_> = DateTime::parse_from_rfc3339(v).map_err(|e| {
            de::Error::custom(format!("failed to parse string as RFC3339 datetime: {}", e))
        })?;
        if fixed_datetime.offset().local_minus_utc() != 0 {
            return Err(de::Error::custom(format!("expected UTC datetime, got one with offset {}",
                fixed_datetime.offset())));
        }
        Ok(Some(fixed_datetime.with_timezone(&Utc)))
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}
