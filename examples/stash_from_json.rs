//! Example showing how to parse raw stash JSON data.
//!
//! Potential usage:
//!
//!   $ curl http://pathofexile.com/api/public-stash-tabs \|
//!        jq '.["stashes"][42]' \|
//!        cargo run --example stash_from_json
//!

             extern crate env_logger;
             extern crate ezomyte;
#[macro_use] extern crate log;
             extern crate serde_json;


use std::io;
use std::process::exit;

use ezomyte::Stash;


fn main() {
    env_logger::init();

    let stash: Stash = serde_json::from_reader(&mut io::stdin()).unwrap_or_else(|e| {
        error!("Error parsing stash JSON: {}", e);
        exit(1);
    });
    println!("{:?}", stash);
}
