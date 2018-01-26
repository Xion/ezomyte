//! Example of tailing the latest items from public stashes API.
//! Uses poe.ninja database to get the most recent change_id.

extern crate env_logger;
extern crate ezomyte;
extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;


use std::error::Error;

use futures::{Future, Stream};
use hyper::Method;
use hyper::client::{Connect, Request};
use hyper::header::UserAgent;
use serde_json::Value as Json;
use tokio_core::reactor::Core;


const USER_AGENT: &str = "ezomyte example:latest";

const POE_NINJA_STATS_URL: &str = "http://poe.ninja/api/Data/GetStats";


fn main() {
    env_logger::init();

    let mut core = Core::new().unwrap();
    let http = hyper::Client::new(&core.handle());
    let client = ezomyte::Client::with_http(
        http.clone(), ezomyte::DEFAULT_API_ROOT, USER_AGENT);
    core.run(
        get_latest_change_id(&http).and_then(move |change_id| {
            client.stashes().since(change_id).for_each(|stash| {
                println!("{:#?}", stash);
                Ok(())
            })
        })
    ).unwrap();
}

fn get_latest_change_id<C: Connect>(
    http: &hyper::Client<C>
) -> Box<Future<Item=String, Error=Box<Error>>> {
    let mut request = Request::new(Method::Get, POE_NINJA_STATS_URL.parse().unwrap());
    request.headers_mut().set(UserAgent::new(USER_AGENT));
    Box::new(http.request(request).from_err().and_then(move |resp| {
        let status = resp.status();
        resp.body().concat2().from_err().and_then(move |body| {
            if status.is_success() {
                serde_json::from_slice::<Json>(&body)?.as_object()
                    .and_then(|obj| obj.get("next_change_id"))
                    .and_then(|cid| cid.as_str()).map(ToString::to_string)
                    .ok_or_else(|| "No next_change_id found in poe.ninja response".into())
            } else {
                Err(format!("Error talking to poe.ninja: {:?}", status).into())
            }
        })
    }))
}
