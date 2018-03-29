# ezomyte

[![crates.io](http://meritbadge.herokuapp.com/ezomyte)](https://crates.io/crates/ezomyte)
[![Build Status](https://travis-ci.org/Xion/ezomyte.svg?branch=master)](https://travis-ci.org/Xion/ezomyte)
[![License](https://img.shields.io/github/license/Xion/ezomyte.svg)]()

Client library for Path of Exile API

[Documentation](https://docs.rs/ezomyte)

_Warning_: The crate is in early stages and the interface (esp. the data model for items)
is likely to evolve over time.

----

## Installation

Add _ezomyte_ to your project's `[dependencies]` in _Cargo.toml_:

```toml
[dependencies]
ezomyte = "0.0.2"
```

## Usage

`ezomyte::Client` provides access to various part of Path of Exile API:
public stashes (`Client::stashes`), current & past leagues (`Client::leagues`), and so on.

All endpoints return asynchronous `Stream`s of structures
that has been deserialized from PoE API.
Here's a simple example of accessing public stash tabs
and looking for items with the unique rarity:

```rust
extern crate ezomyte;
extern crate futures;
extern crate tokio_core;

use ezomyte::Rarity;
use futures::Stream;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let client = ezomyte::Client::new("ezomyte example", &core.handle());
    core.run(
        client.stashes().all().for_each(|stash| {
            let uniques = stash.items.iter().filter(|i| i.rarity == Rarity::Unique);
            for item in uniques {
                // Prints something like "Belly of the Beast -- Full Wyrmscale".
                println!("{} -- {}",
                    item.name.as_ref().map(|n| n.as_str()).unwrap_or("<unnamed>"),
                    item.base);
            }
            Ok(())
        })
    ).unwrap();
}

```

See [the _examples_ directory](examples/) for more examples.

----

## Development

Besides the current version of Rust compiler and Cargo, you would want:

* the _just_ task runner (`cargo install just`)
* _jq_, the command line JSON manipulator (`apt-get install jq` or similar)

Running `just` will execute all the tests and compile examples.
