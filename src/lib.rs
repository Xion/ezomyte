//!
//! ezomyte
//! Path of Exile API client
//!

             extern crate hyper;
             extern crate hyper_tls;
#[macro_use] extern crate macro_attr;
#[macro_use] extern crate newtype_derive;
             extern crate separator;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;
             extern crate tokio_core;


mod model;
mod util;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
