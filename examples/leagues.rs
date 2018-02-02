//! Example showing how to query and display leagues.

             extern crate env_logger;
             extern crate ezomyte;
             extern crate futures;
#[macro_use] extern crate log;
             extern crate tokio_core;


use std::env;
use std::process::exit;

use futures::Stream;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:leagues";


fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().skip(1).collect();
    let type_ = match args.iter().next() {
        Some(t) if ["--all", "--main", "--event", "--season"].contains(&t.as_str()) => &t[2..],
        Some(t) => { error!("unrecognized argument `{}`", t); exit(2); }
        None => "all",
    };
    let season = if type_ == "season" {
        if args.len() < 2 {
            error!("--season flag requires an argument");
            exit(2);
        }
        Some(args[1].trim())
    } else {
        None
    };

    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new(USER_AGENT, &core.handle());
    let leagues = match type_ {
        "all" => client.leagues().all(),
        "main" => client.leagues().main(),
        "event" => client.leagues().event(),
        "season" => client.leagues().in_season(season.unwrap()),
        _ => unreachable!(),
    };
    core.run(
        leagues.for_each(|league| {
            println!("{:?}", league);
            Ok(())
        })
    ).unwrap();
}
