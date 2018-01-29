//! Module for the `Stashes` accessor object.

use std::borrow::Cow;

use futures::{future, Future as StdFuture, stream, Stream as StdStream};
use hyper::client::Connect;

use ::{Client, Stash};
use super::Stream;


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
    // TODO: find a way to relay the final next-change-id to callers
    // so they can implement polling for new items if they wish
    // TODO: also, a way to obtain the "current" next-change-id would be nice
    // (or even a method like newest() to automatically start from there)

    fn get_stashes_stream(&self, change_id: Option<String>) -> Stream<Stash> {
        /// Enum for managing the state machine of the resulting Stream.
        enum State {
            Start{change_id: Option<String>},
            Next{change_id: String},
            End,
        }

        // Repeatedly query the public stash tabs endpoint
        // and yield `Stash` items as they come using Stream::unfold.
        let this = self.clone();
        Box::new(
            stream::unfold(State::Start{change_id}, move |state| {
                let change_id = match state {
                    State::Start{change_id} => change_id,
                    State::Next{change_id} => Some(change_id),
                    // We handle stream termination via State::End
                    // so that the last page of results is correctly included.
                    State::End => return None,
                };
                let url: Cow<str> = match change_id.as_ref() {
                    Some(cid) => format!("{}?id={}", STASHES_URL, cid).into(),
                    None => STASHES_URL.into(),
                };
                // TODO: what happens when an invalid change_id is passed?
                // we should handle that as a separate error type here
                // (which may wrap the crate-level Error)
                Some(this.client.get(url).and_then(move |resp: PublicStashTabsResponse| {
                    let next_state = match resp.next_change_id {
                        Some(next_cid) => {
                            // If we got the same change_id, we've reached the end.
                            let same_cid = change_id.as_ref().map(|cid| cid == &next_cid)
                                .unwrap_or(false);
                            if same_cid { State::End } else {
                                State::Next{change_id: next_cid}
                            }
                        }
                        None => {
                            // According to API docs, this is not supposed to happen.
                            warn!("No next_change_id found in stash tabs' response");
                            State::End
                        }
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
