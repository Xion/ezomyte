//! Module implementing the various APIs exposed by the library.

mod stashes;


use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;

use futures::Stream as StdStream;

use super::error::Error;
pub use self::stashes::Stashes;


/// Stream type returned by various API methods.
pub type Stream<T, E = Error> = Box<StdStream<Item = T, Error = E>>;


/// Wrapper around entries that come from the API in batches.
///
/// Besides `Deref`ing to the entry type (`T`), the wrapper exposes batch tokens
/// (e.g. `change_id` from public stash API) for resuming an interrupted entry fetch
/// at some later time.
#[derive(Debug)]
pub struct Batched<T: Debug, C = String> {
    /// The entry itself.
    entry: T,
    /// Continuation token of the current batch
    /// (the one that the entry is coming from).
    curr_token: Option<C>,
    /// Continuation token of the next batch, if any.
    next_token: Option<C>,
}

impl<T: Debug, C> Deref for Batched<T, C> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl<T: Debug, C> Batched<T, C> {
    /// Returns the token for a batch that this entry is coming from.
    #[inline]
    pub fn current_batch_token<Q>(&self) -> Option<&Q>
        where Q: ?Sized, C: Borrow<Q>
    {
        self.curr_token.as_ref().map(|t| t.borrow())
    }

    /// Returns the token that can be used to request the next batch of entries.
    #[inline]
    pub fn next_batch_token<Q>(&self) -> Option<&Q>
        where Q: ?Sized, C: Borrow<Q>
    {
        self.next_token.as_ref().map(|t| t.borrow())
    }
}


#[cfg(test)]
mod tests {
    use super::Batched;

    /// Test to make sure we didn't screw up the Borrow trait constraint.
    #[test]
    fn batched_with_string_token() {
        let batched: Batched<(), String> = Batched{
            entry: (), curr_token: None, next_token: Some("foo".into())
        };
        let token: &str = batched.next_batch_token().unwrap();
        assert_eq!("foo", token);
    }
}
