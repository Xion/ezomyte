//! Module implementing some API helpers.

use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;


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
    /// Create a new batched entry.
    #[inline]
    pub(crate) fn new(curr_token: Option<C>, entry: T, next_token: Option<C>) -> Self {
        Batched { entry, curr_token, next_token }
    }

    /// Create a batched first entry in the API response stream.
    #[inline]
    pub(crate) fn first(entry: T, next_token: Option<C>) -> Self {
        Batched::new(None, entry, next_token)
    }

    /// Create a batched last entry from the API response stream.
    #[inline]
    pub(crate) fn last(curr_token: Option<C>, entry: T) -> Self {
        Batched::new(curr_token, entry, None)
    }

    /// Create the only batched from the API response stream.
    #[inline]
    pub(crate) fn only(entry: T) -> Self {
        Batched::new(None, entry, None)
    }
}
impl<T: Debug, C> From<T> for Batched<T, C> {
    fn from(input: T) -> Self {
        Batched::only(input)
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
