//!
//! ezomyte
//! Path of Exile API client
//!

             extern crate hyper;
             extern crate hyper_tls;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;
             extern crate tokio_core;


mod util;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
