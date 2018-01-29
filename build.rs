//! Build script.

             extern crate itertools;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;


use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;

use itertools::Itertools;


fn main() {
    generate_currency_code().unwrap();
}


// Currency handling

const CURRENCY_JSON_FILE: &str = "data/currency.json";
const CURRENCY_ENUM_FILE: &str = "model/currency/enum.inc.rs";

/// Generate code for the `Currency` enum and its deserialization.
fn generate_currency_code() -> Result<(), Box<Error>> {
    let data_file = Path::new(".").join(CURRENCY_JSON_FILE);
    let mut file = fs::OpenOptions::new().read(true).open(data_file)?;
    let currencies: Vec<CurrencyData> = serde_json::from_reader(&mut file)?;

    let enum_file = Path::new(&env::var("OUT_DIR").unwrap()).join(CURRENCY_ENUM_FILE);
    fs::create_dir_all(enum_file.parent().unwrap())?;
    let mut out = fs::OpenOptions::new()
        .create(true).truncate(true).write(true)
        .open(enum_file)?;

    // TODO: some templating engine could be useful for code generation
    writeln!(out, "#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(out, "pub enum Currency {{")?;
    {
        // E.g. #[serde(rename="chaos")] ChaosOrb;
        for currency in currencies {
            // TODO: support both the originals & the manual overrides
            // (the overrides are based on data actually encountered in public stash tabs)
            // by also generating a custom Deserialize impl
            let id = match currency.id.as_str() {
                "exalted" => "exa",
                "fusing" => "fuse",
                "mirror" => "mir",
                id => id,
            };
            writeln!(out, "    #[serde(rename=\"{}\")]", id)?;
            writeln!(out, "    {},", upper_camel_case(&currency.name))?;
        }
    }
    writeln!(out, "}}")?;
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


// Utility functions

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
