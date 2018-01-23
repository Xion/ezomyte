//!
//! ezomyte
//! Path of Exile API client
//!

                #[macro_use] extern crate enum_derive;
                             extern crate futures;
                             extern crate hyper;
                             extern crate hyper_tls;
                             extern crate itertools;
                #[macro_use] extern crate lazy_static;
                #[macro_use] extern crate log;
                #[macro_use] extern crate macro_attr;
                #[macro_use] extern crate newtype_derive;
                             extern crate regex;
                             extern crate separator;
                             extern crate serde;
                #[macro_use] extern crate serde_derive;
#[cfg_attr(test, macro_use)] extern crate serde_json;
                             extern crate tokio_core;


mod client;
mod error;
mod model;
mod stashes;
mod util;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
