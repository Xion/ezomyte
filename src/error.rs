//! Module defining the error type used by the crate.

use std::error::Error as StdError;


/// Error type emitted by the crate.
pub type Error = Box<StdError>;
// TODO: actual useful error type lol
