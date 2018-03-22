//! Example of retrieving item mod values and operating on them as numbers.
//!
//! Requires two arguments:
//! * a search query for mod texts, e.g. "increased Life"
//! * lower bound for the item mod value (or the average of two values,
//!   following a simplifying assumption that it's always a damage range)
//!
//! E.g.:
//!
//!     cargo run --example mod_values --features=mods_db -- "% increased Life" 6
//!
//! Note that --features part is only necessary due to https://github.com/rust-lang/cargo/issues/4663,
//! and wouldn't be an issue in a real program with its own Cargo manifest.
//!

             extern crate env_logger;
             extern crate ezomyte;
             extern crate futures;
#[macro_use] extern crate log;
             extern crate tokio_core;


use std::env;
use std::process::exit;

use futures::Stream;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:mod_values";


fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 2 {
        error!("Expected two arguments, got {}", args.len());
        exit(2);
    }
    let query = &args[0];
    let threshold: u64 = args[1].parse().unwrap();

    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new(USER_AGENT, &core.handle());
    core.run(
        client.stashes().all().for_each(|stash| {
            for item in &stash.items {
                let mut matches = false;
                for mod_ in item.mods() {
                    if !mod_.as_str().contains(query) {
                        continue;
                    }
                    if let Some(ref vs) = mod_.values() {
                        assert!(vs.len() > 0);
                        let avg = vs.iter().sum::<f64>() / vs.len() as f64;
                        if avg >= threshold as f64 {
                            matches = true;
                            break;
                        }
                    }
                }
                if matches {
                    println!("{:?}", item);
                }
            }
            Ok(())
        })
    ).unwrap();
}
