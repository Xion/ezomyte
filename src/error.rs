//! Module defining the error type used by the crate.

use std::error;
use std::fmt;

use hyper::{self, StatusCode};
use serde_json;


/// Error type emitted by the crate.
#[derive(Debug)]
pub enum Error {
    /// Error for when the API is currently undergoing maintenance.
    UnderMaintenance,
    /// Error for when the server returns an HTTP error code (not 2xx).
    Server(StatusCode),
    /// Error parsing the JSON response from the server.
    Json(serde_json::Error),
    /// General error from the underlying HTTP client.
    Http(hyper::Error),
}

impl From<hyper::Error> for Error {
    fn from(input: hyper::Error) -> Error {
        Error::Http(input)
    }
}
impl From<serde_json::Error> for Error {
    fn from(input: serde_json::Error) -> Error {
        Error::Json(input)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnderMaintenance => "API is under maintenance",
            Error::Server(_) => "server error",
            Error::Json(_) => "error parsing API response",
            Error::Http(_) => "general HTTP or network error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Json(ref e) => Some(e),
            Error::Http(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnderMaintenance => write!(fmt, "API is currently undergoing maintenance"),
            Error::Server(ref s) => write!(fmt, "HTTP error status: {}", s),
            Error::Json(ref e) => write!(fmt, "failed to parse API response: {}", e),
            Error::Http(ref e) => write!(fmt, "HTTP/networking error: {}", e),
        }
    }
}
