//! Module defining the PoE API client object.

use hyper;
use hyper::client::{Connect, HttpConnector};
use tokio_core::reactor::Handle;


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
        where R: Into<String>, A: Into<String>
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
        where R: Into<String>, A: Into<String>
    {
        Client {
            http,
            api_root: api_root.into(),
            user_agent: user_agent.into(),
        }
    }
}
