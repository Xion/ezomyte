//! Utility module.

use std::num::{ParseIntError, ParseFloatError};

use hyper;
use hyper::client::HttpConnector;
use hyper_tls;
use tokio_core::reactor::Handle;


const HTTPS_DNS_THREADS: usize = 4;

/// Type of a TLS-compatible Hyper Connector.
pub type HttpsConnector = hyper_tls::HttpsConnector<HttpConnector>;

/// Type of a TLS-capable asynchronous Hyper client.
pub type HttpsClient = hyper::Client<HttpsConnector>;

/// Create an asynchronous, TLS-capable HTTP Hyper client
/// working with given Tokio Handle.
pub fn https_client(handle: &Handle) -> HttpsClient {
    let connector =
        hyper_tls::HttpsConnector::new(HTTPS_DNS_THREADS, handle).unwrap();
    hyper::client::Config::default()
        .connector(connector)
        .build(handle)
}


/// Parse a number as 64-bit float.
///
/// The input number can be an integer, a float, or a numerator/denominator rational.
pub fn parse_number(s: &str) -> Result<f64, ParseNumberError> {
    // Assume number is in the from of `$NUMBER` or `$NUMBER / $NUMBER`.
    let nums: Vec<_> = s.split('/').collect();
    let num_count = nums.len();
    match num_count {
        1 => {
            // Try parsing as integer to handle common cases like "1 chaos".
            // Otherwise fall back to floats for things like "1.5 exa".
            let amount = nums[0].trim();
            amount.parse::<u64>().map(|a| a as f64)
                .or_else(|_| amount.parse::<f64>())
                .map_err(Into::into)
        },
        2 => {
            let numerator: f64 = nums[0].trim().parse()?;
            let denominator: f64 = nums[1].trim().parse()?;
            Ok(numerator / denominator)
        }
        _ => Err(ParseNumberError::Syntax),
    }
}

#[derive(Debug, Error)]
pub enum ParseNumberError {
    /// Error parsing floating point amount.
    Float(ParseFloatError),
    /// Error parsing rational amount (X/Y fraction where X,Y are integers).
    Rational(ParseIntError),
    /// General syntax error while parsing the amount.
    Syntax,
}
