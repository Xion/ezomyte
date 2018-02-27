//! Example showing how to use the client to query & display
//! just the names of publicly visible items.

extern crate env_logger;
extern crate ezomyte;
extern crate futures;
extern crate tokio_core;


use futures::Stream;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:item_names";


fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new(USER_AGENT, &core.handle());
    core.run(
        client.stashes().all().for_each(|stash| {
            for item in &stash.items {
                println!("{}", item.name.as_ref().unwrap_or(&item.base));
            }
            Ok(())
        })
    ).unwrap();
}
