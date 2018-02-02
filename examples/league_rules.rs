//! Example showing how to query and display league rules.

extern crate env_logger;
extern crate ezomyte;
extern crate futures;
extern crate tokio_core;


use futures::Stream;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:league_rules";


fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new(USER_AGENT, &core.handle());
    core.run(
        client.league_rules().all().for_each(|rule| {
            println!("{:?}", rule);
            Ok(())
        })
    ).unwrap();
}
