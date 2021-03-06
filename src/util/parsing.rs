//! Module with parsing utilities.

use std::num::{ParseIntError, ParseFloatError};


/// Parse a number as 64-bit float.
///
/// The input number can be an integer, a float, or a numerator/denominator rational.
pub fn parse_number(s: &str) -> Result<f64, ParseNumberError> {
    // Assume number is in the from of `$NUMBER` or `$NUMBER / $NUMBER`.
    let nums: Vec<_> = s.trim().split('/').collect();
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

/// Error while parsing a number.
#[derive(Debug, Error)]
pub enum ParseNumberError {
    /// Error parsing a floating point number.
    Float(ParseFloatError),
    /// Error parsing a rational number (X/Y fraction where X,Y are integers).
    Rational(ParseIntError),
    /// General syntax error while parsing the number.
    Syntax,
}
