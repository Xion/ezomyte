//! Module implementing the various APIs exposed by the library.

mod stashes;


use futures::Stream as StdStream;

use super::error::Error;
pub use self::stashes::Stashes;


/// Stream type returned by various API methods.
pub type Stream<T, E = Error> = Box<StdStream<Item = T, Error = E>>;
