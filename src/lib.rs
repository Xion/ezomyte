//!
//! Path of Exile API client
//!
//! ## Usage
//!
//! See [the crate README](https://github.com/Xion/ezomyte) for usage information.
//!

#![allow(dead_code)]  // TODO: prune unused parts
#![allow(missing_docs)]

                             extern crate chrono;
                             extern crate conv;
                #[macro_use] extern crate derive_error;
                #[macro_use] extern crate derive_more;
                #[macro_use] extern crate enum_derive;
                             extern crate futures;
                             extern crate hyper;
                             extern crate hyper_tls;
                             extern crate itertools;
                #[macro_use] extern crate lazy_static;
                #[macro_use] extern crate log;
                #[macro_use] extern crate macro_attr;
                #[macro_use] extern crate newtype_derive;
                             extern crate num;
                             extern crate regex;
                             extern crate separator;
                #[macro_use] extern crate serde;
                #[macro_use] extern crate serde_derive;
#[cfg_attr(test, macro_use)] extern crate serde_json;
                             extern crate smallvec;
                             extern crate tokio_core;


mod league_rules;
mod leagues;
mod pvp_matches;
mod stashes;

mod client;
mod common;
mod error;
mod util;


pub use self::league_rules::*;
pub use self::leagues::*;
pub use self::pvp_matches::*;
pub use self::stashes::*;

pub use self::client::*;
pub use self::error::*;
