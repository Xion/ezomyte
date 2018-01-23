//! Module for the `Stashes` accessor object.

use std::borrow::Cow;

use futures::{future, Future as StdFuture, stream, Stream as StdStream};
use hyper::client::Connect;

use super::client::Client;
use super::error::Error;
use super::model::Stash;


/// Stream type returned by methods of the `Stashes` type.
pub type Stream<T> = Box<StdStream<Item = T, Error = Error>>;


/// Interface for accessing the public stashes.
#[derive(Clone, Debug)]
pub struct Stashes<C>
    where C: Clone + Connect
{
    client: Client<C>,
}

impl<C: Clone + Connect> Stashes<C> {
    #[inline]
    pub(crate) fn new(client: Client<C>) -> Self {
        Stashes { client }
    }
}

impl<C: Clone + Connect> Stashes<C> {
    /// Returns a stream of all `Stash` objects from the beginning of time.
    #[inline]
    pub fn all(&self) -> Stream<Stash> {
        self.get_stashes_stream(None)
    }

    /// Returns a stream of `Stash` objects beginning at given `change_id`.
    #[inline]
    pub fn since<Cid>(&self, change_id: Cid) -> Stream<Stash>
        where Cid: Into<String>
    {
        self.get_stashes_stream(Some(change_id.into()))
    }

    fn get_stashes_stream(&self, change_id: Option<String>) -> Stream<Stash> {
        /// Enum for managing the state machine of the resulting Stream.
        enum State {
            Start{change_id: Option<String>},
            Next{change_id: String},
            End,
        }
        fn next_url(change_id: &str) -> String {
            format!("{}?id={}", STASHES_URL, change_id)
        }

        // Repeatedly query the public stash tabs endpoint
        // and yield `Stash` items as they come using Stream::unfold.
        let this = self.clone();
        Box::new(
            stream::unfold(State::Start{change_id}, move |state| {
                let url: Cow<str> = match state {
                    State::Start{change_id} => match change_id {
                        Some(ref cid) => next_url(cid).into(),
                        None => STASHES_URL.into(),
                    },
                    State::Next{change_id} => next_url(&change_id).into(),
                    // We handle stream termination via State::End
                    // so that the last page of results is correctly included.
                    State::End => { return None; },
                };
                Some(this.client.get(url).and_then(|resp: PublicStashTabsResponse| {
                    let next_state = match resp.next_change_id {
                        Some(next_cid) => State::Next{change_id: next_cid},
                        None => State::End,
                    };
                    future::ok((stream::iter_ok(resp.stashes), next_state))
                }))
            })
            .flatten()
        )
    }
}

const STASHES_URL: &str = "/public-stash-tabs";


/// Response from the /public-stash-tabs API endpoint.
#[derive(Debug, Deserialize)]
struct PublicStashTabsResponse {
    next_change_id: Option<String>,
    stashes: Vec<Stash>,
}
