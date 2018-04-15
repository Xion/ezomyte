//! Module for the PvP matches PoE API.

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use futures::{Future as StdFuture, stream, Stream as StdStream};
use hyper::client::Connect;
use serde::de::{self, Deserialize, Deserializer};

use ::{Client, Stream};


/// Interface for accessing PvP matches information.
#[derive(Debug)]
pub struct PvpMatches<C>
    where C: Clone + Connect
{
    client: Client<C>,
}

impl<C: Clone + Connect> PvpMatches<C> {
    #[inline]
    pub(crate) fn new(client: Client<C>) -> Self {
        PvpMatches { client }
    }
}

impl<C: Clone + Connect> PvpMatches<C> {
    /// Returns a stream of all upcoming PvP matches.
    #[inline]
    pub fn all(&self) -> Stream<PvpMatch> {
        self.get_pvp_matches_stream(None)
    }

    /// Return a stream of PvP matches in a particular season.
    #[inline]
    pub fn in_season<S: Into<String>>(&self, season: S) -> Stream<PvpMatch> {
        self.get_pvp_matches_stream(Some(season.into()))
    }

    fn get_pvp_matches_stream(&self, season: Option<String>) -> Stream<PvpMatch> {
        let url: Cow<str> = match season {
            Some(s) => format!("{}?type=season&season={}", PVP_MATCHES_URL, s).into(),
            None => PVP_MATCHES_URL.into(),
        };
        Box::new(
            self.client.get(&*url)
                .map(|resp: PvpMatchesResponse| stream::iter_ok(resp))
                .into_stream()
                .flatten()
        )
    }
}

const PVP_MATCHES_URL: &str = "/pvp-matches";


/// Response from the PvP matches API.
type PvpMatchesResponse = Vec<PvpMatch>;

/// Information about a single PvP match, as obtained from the API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PvpMatch {
    /// Identifier of the match.
    pub id: String,
    /// When has this match started, or will start.
    #[serde(deserialize_with = "deserialize_datetime")]
    start_at: DateTime<Utc>,
    /// When has this match finished, or will finish.
    #[serde(deserialize_with = "deserialize_datetime")]
    end_at: DateTime<Utc>,
    /// URL to the forum thread.
    url: String,
    /// Description of the match.
    description: String,
    /// Match style, usually "Arena", "Blitz", or "Swiss".
    style: String,
    // TODO: consider an enum here?
    /// When does the match registration start.
    #[serde(deserialize_with = "deserialize_datetime")]
    register_at: DateTime<Utc>,
}

/// Deserialize a UTC DateTime stored as RFC 3339 string.
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let fixed_datetime: DateTime<_> = DateTime::parse_from_rfc3339(&s).map_err(|e| {
        de::Error::custom(format!("failed to parse string as RFC3339 datetime: {}", e))
    })?;
    if fixed_datetime.offset().local_minus_utc() != 0 {
        return Err(de::Error::custom(format!("expected UTC datetime, got one with offset {}",
            fixed_datetime.offset())));
    }
    Ok(fixed_datetime.with_timezone(&Utc))
}
