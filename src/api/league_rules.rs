//! Module for the PoE league rules API.

use futures::{Future as StdFuture, stream, Stream as StdStream};
use hyper::client::Connect;

use ::Client;
use super::Stream;


/// Interface for accessing league rules information.
#[derive(Debug)]
pub struct LeagueRules<C>
    where C: Clone + Connect
{
    client: Client<C>,
}

impl<C: Clone + Connect> LeagueRules<C> {
    #[inline]
    pub(crate) fn new(client: Client<C>) -> Self {
        LeagueRules { client }
    }
}

impl<C: Clone + Connect> LeagueRules<C> {
    /// Returns a stream of all known league rules.
    #[inline]
    pub fn all(&self) -> Stream<LeagueRule> {
        self.get_league_rules_stream()
    }

    fn get_league_rules_stream(&self) -> Stream<LeagueRule> {
        Box::new(
            self.client.get(LEAGUE_RULES_URL)
                .map(|resp: LeagueRulesResponse| stream::iter_ok(resp))
                .into_stream()
                .flatten()
        )
    }
}

const LEAGUE_RULES_URL: &str = "/league-rules";


/// Information about a single league rule, as obtained from the API.
#[derive(Debug, Deserialize)]
pub struct LeagueRule {
    /// Identifier of the league rule.
    pub id: u64,
    /// Name of the league rule.
    pub name: String,
    /// Description of the league rule.
    pub description: String,
}


/// Response from the league rules' API.
type LeagueRulesResponse = Vec<LeagueRule>;
