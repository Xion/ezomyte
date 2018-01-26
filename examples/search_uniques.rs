//! Example for how to search for unique items with given name.
//! Shows the items found along with their price.

extern crate env_logger;
extern crate ezomyte;
extern crate futures;
extern crate tokio_core;


use std::env;

use ezomyte::Rarity;
use futures::Stream;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:search_uniques";


fn main() {
    env_logger::init();

    let query = env::args().skip(1).next().map(|q| q.to_lowercase());

    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new(USER_AGENT, &core.handle());
    core.run(
        client.stashes().all().for_each(|stash| {
            let uniques = stash.items.iter().filter(|i| i.rarity == Rarity::Unique);
            for item in uniques {
                if let (Some(q), Some(n)) = (query.as_ref(), item.name.as_ref()) {
                    if !n.to_lowercase().contains(q) {
                        continue;
                    }
                }
                let price = item.exact_price()
                    .or_else(|| item.negotiable_price())
                    .or_else(|| stash.label.as_exact_price())
                    .or_else(|| stash.label.as_negotiable_price());
                let price_text = price.map(|p| format!("{}", p))
                    .unwrap_or_else(|| "<no price>".into());
                println!("{}; {} -- {}",
                    item.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>"),
                    item.base, price_text);
            }
            Ok(())
        })
    ).unwrap();
}
