//! Build script.
//!
//! The script takes static data obtained from the API and generates some code
//! for inclusion in the library source, including e.g. the `Currency` enum.

             extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;


use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use itertools::Itertools;


fn main() {
    generate_currency_code().unwrap();
}


// Currency handling

const CURRENCY_JSON_FILES: &[&str] = &["data/currency.json", "data/extra/currency.json"];
const CURRENCY_ENUM_FILE: &str = "model/currency/enum.inc.rs";
const CURRENCY_DE_FILE: &str = "model/de/currency/visit_str.inc.rs";

lazy_static! {
    /// Mapping from currency IDs loaded from JSON file to their additional IDs
    /// used in public stash tab item pricing.
    static ref ADDITIONAL_CURRENCY_IDS: HashMap<String, &'static [&'static str]> = hashmap!{
        "exalted".into() => &["exa"] as &[&str],
        "fusing".into() => &["fuse"],
        "mir".into() => &["mirror"],
    };
}

/// Generate code for the `Currency` enum and its deserialization.
fn generate_currency_code() -> Result<(), Box<Error>> {
    let mut all_currencies = Vec::new();
    for &path in CURRENCY_JSON_FILES.iter() {
        let data_file = Path::new(".").join(path);
        let mut file = fs::OpenOptions::new().read(true).open(data_file)?;
        let currencies: Vec<CurrencyData> = serde_json::from_reader(&mut file)?;
        all_currencies.extend(currencies);
    }

    generate_currency_enum(&all_currencies)?;
    generate_currency_de(&all_currencies)?;
    Ok(())
}

/// Structure describing JSON objects in CURRENCY_JSON_FILE.
#[derive(Debug, Deserialize)]
struct CurrencyData {
    image: String,
    #[serde(rename = "text")]
    name: String,
    id: String,
}

fn generate_currency_enum(currencies: &[CurrencyData]) -> io::Result<()> {
    let mut out = create_out_file(CURRENCY_ENUM_FILE)?;

    // TODO: some templating engine could be useful for code generation
    // (but not the `quote` crate because we're creating unhygenic identifiers here)
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(out, "pub enum Currency {{")?;
    {
        // E.g. #[serde(rename="chaos")] ChaosOrb,
        for currency in currencies {
            writeln!(out, "    #[serde(rename=\"{}\")]", currency.id)?;
            writeln!(out, "    {},", upper_camel_case(&currency.name))?;
        }
    }
    writeln!(out, "}}")?;
    Ok(())
}

/// Generate the visit_str() method branches of Currency deserializer.
fn generate_currency_de(currencies: &[CurrencyData]) -> io::Result<()> {
    let mut out = create_out_file(CURRENCY_DE_FILE)?;

    writeln!(out, "match v {{")?;
    for currency in currencies {
        // Include possible alternative/community "names" for the currencies
        // so that they are deserialized correctly.
        let mut patterns: Vec<&str> = vec![&currency.id];
        if let Some(more) = ADDITIONAL_CURRENCY_IDS.get(&currency.id) {
            patterns.extend(more.iter());
        }
        writeln!(out, "    {} => Ok(Currency::{}),",
            patterns.into_iter().format_with(" | ", |x, f| f(&format_args!("\"{}\"", x))),
            upper_camel_case(&currency.name))?;
    }
    writeln!(out, "    v => Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG)),")?;
    writeln!(out, "}}")?;
    Ok(())
}


// Utility functions

/// Open a file inside the $OUT_DIR for writing.
fn create_out_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join(path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::OpenOptions::new()
        .create(true).truncate(true).write(true)
        .open(path)
}

fn upper_camel_case(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace().map(capitalize).join("")
}

fn capitalize(s: &str) -> String {
    let mut result = String::new();
    if s.is_empty() {
        return result;
    }
    result.push_str(s.chars().next().unwrap().to_uppercase().to_string().as_str());
    result.push_str(s.chars().skip(1).collect::<String>().to_lowercase().as_str());
    result
}
