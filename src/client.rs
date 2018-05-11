//! Module defining the PoE API client object.

use futures::{Future as StdFuture, Stream as StdStream};
use hyper::{self, Method, StatusCode};
use hyper::client::{Connect, HttpConnector, Request};
use hyper::header::UserAgent;
use log::Level::*;
use regex::{Regex, RegexBuilder};
use serde::de::DeserializeOwned;
use serde_json;
use tokio_core::reactor::Handle;

use super::league_rules::LeagueRules;
use super::leagues::Leagues;
use super::pvp_matches::PvpMatches;
use super::stashes::Stashes;
pub use super::error::{Error, Stream};


/// Type of futures produced by the client.
pub type Future<T> = Box<StdFuture<Item = T, Error = Error>>;


/// Default root URL for the Path of Exile API.
pub const DEFAULT_API_ROOT: &str = "http://www.pathofexile.com/api";


/// Client interface for interacting with Path of Exile API.
#[derive(Clone, Debug)]
pub struct Client<C>
    where C: Clone + Connect
{
    /// The underlying HTTP client.
    http: hyper::Client<C>,
    /// Root URL of the API.
    api_root: String,
    /// User-Agent header value to use when making requests.
    user_agent: String,
}

// TODO: optional TLS support via a feature flag (like hubcaps, but disabled by default)
impl Client<HttpConnector> {
    /// Create a new `Client` which points to the default API URL.
    pub fn new<A>(user_agent: A, handle: &Handle) -> Self
        where A: Into<String>
    {
        Self::with_api_root(DEFAULT_API_ROOT, user_agent, handle)
    }

    /// Create a `Client` which points to given API URL.
    pub fn with_api_root<R, A>(api_root: R, user_agent: A, handle: &Handle) -> Self
        where R: AsRef<str>, A: Into<String>
    {
        let http = hyper::Client::configure()
            .keep_alive(true)
            .build(handle);
        Self::with_http(http, api_root, user_agent)
    }
}
impl<C: Clone + Connect> Client<C> {
    /// Create a `Client` which directly wraps a `hyper::Client`.
    pub fn with_http<R, A>(http: hyper::Client<C>, api_root: R, user_agent: A) -> Self
        where R: AsRef<str>, A: Into<String>
    {
        Client {
            http,
            api_root: api_root.as_ref().trim_right_matches("/").to_owned(),
            user_agent: user_agent.into(),
        }
    }
}

impl<C: Clone + Connect> Client<C> {
    /// Access interface for public stash tabs.
    #[inline]
    pub fn stashes(&self) -> Stashes<C> {
        Stashes::new(self.clone())
    }

    /// Access interface for PoE league information.
    #[inline]
    pub fn leagues(&self) -> Leagues<C> {
        Leagues::new(self.clone())
    }

    /// Access the interface PoE league rules' information.
    #[inline]
    pub fn league_rules(&self) -> LeagueRules<C> {
        LeagueRules::new(self.clone())
    }

    /// Access the interface for PoE PvP matches information.
    #[inline]
    pub fn pvp_matches(&self) -> PvpMatches<C> {
        PvpMatches::new(self.clone())
    }
}

impl<C: Clone + Connect> Client<C> {
    /// Make a GET request to given URL and return deserialized response.
    pub(crate) fn get<U, Out>(&self, url: U) -> Future<Out>
        where U: AsRef<str>,
              Out: DeserializeOwned + 'static
    {
        self.request(Method::Get, url)
    }

    /// Make a request to given URL and return deserialized response.
    fn request<U, Out>(&self, method: Method, url: U) -> Future<Out>
        where U: AsRef<str>,
              Out: DeserializeOwned + 'static
    {
        let url = format!("{}/{}",
            self.api_root, url.as_ref().trim_left_matches("/"));

        let mut request = Request::new(method.clone(), url.parse().unwrap());
        request.headers_mut().set(UserAgent::new(self.user_agent.clone()));

        trace!("{} {}", method, url);
        let this = self.clone();
        Box::new(
            this.http.request(request).from_err().and_then(move |resp| {
                let status = resp.status();
                debug!("HTTP {}{} for {} {}",
                    status.as_u16(),
                    status.canonical_reason()
                        .map(|r| format!(" ({})", r)).unwrap_or_else(String::new),
                    method, url);
                resp.body().concat2().from_err().and_then(move |body| {
                    // Log the beginning of the response, but not the entire one
                    // since it's likely megabytes.
                    if log_enabled!(Debug) {
                        const MAX_LEN: usize = 2048;
                        let body_text = String::from_utf8_lossy(&body);
                        let omitted = body_text.len() - MAX_LEN;
                        if omitted > 0 {
                            debug!("Response payload: {}... (and {} more bytes)",
                                &body_text[..MAX_LEN], omitted);
                        } else {
                            debug!("Response payload: {}", body_text);
                        }
                    }

                    if status.is_success() {
                        return serde_json::from_slice::<Out>(&body).map_err(Error::Json);
                    }

                    let body_text = String::from_utf8_lossy(&body);

                    // Check if we got an HTML page signaling maintenance.
                    // It's not perfect but should do well enough in practice.
                    lazy_static! {
                        static ref ANGLE_HTML_RE: Regex = RegexBuilder::new("<html")
                            .case_insensitive(true)
                            .build().unwrap();
                    }
                    let is_maintenance = status == StatusCode::ServiceUnavailable  // HTTP 503
                        && ANGLE_HTML_RE.is_match(&*body_text)
                        && body_text.contains("maintenance");
                    if is_maintenance {
                        Err(Error::UnderMaintenance)
                    } else {
                        Err(Error::Server(status))
                    }
                })
            })
        )
    }
}
