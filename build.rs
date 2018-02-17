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
    generate_item_mod_code().unwrap();
}


// Currency handling

const CURRENCY_JSON_FILES: &[&str] = &["data/currency.json", "data/extra/currency.json"];
const CURRENCY_ENUM_FILE: &str = "model/currency/enum.inc.rs";
const CURRENCY_DE_FILE: &str = "model/de/currency/visit_str.inc.rs";

lazy_static! {
    /// Mapping from currency IDs loaded from JSON files to their additional IDs
    /// used in public stash tab item pricing.
    static ref ADDITIONAL_CURRENCY_IDS: HashMap<&'static str, &'static [&'static str]> = hashmap!{
        // Regular currencies.
        "exalted" => &["exa"] as &[&str],
        "fusing" => &["fuse"],
        "mir" => &["mirror"],
        // Breach league splinters.
        "splinter-xoph" => &["splinter-of-xoph"],
        "splinter-tul" => &["splinter-of-tul"],
        "splinter-esh" => &["splinter-of-esh"],
        "splinter-uul-netol" => &["splinter-of-uul-netol"],
        "splinter-chayula" => &["splinter-of-chayula"],
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

/// Structure describing JSON objects in CURRENCY_JSON_FILES.
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
    writeln!(out, "/// Currency item used for trading.")?;
    writeln!(out, "#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(out, "pub enum Currency {{")?;
    {
        // E.g. #[serde(rename="chaos")] ChaosOrb,
        for currency in currencies {
            writeln!(out, "    /// {}.", currency.name)?;
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
        if let Some(more) = ADDITIONAL_CURRENCY_IDS.get(&currency.id.as_str()) {
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


// Item mod handling

const ITEM_MODS_DATA_DIR: &str = "data/mods";
const ITEM_MODS_TYPES: &[&str] = &["crafted", "enchant", "explicit", "implicit"];
const ITEM_MODS_DATABASE_FILE: &str = "model/item/mods/database/by_id.inc.rs";

/// Generate code for the database of `ModInfo`s.
fn generate_item_mod_code() -> Result<(), Box<Error>> {
    generate_item_mods_id_mapping()?;
    Ok(())
}

/// Generate the mapping of all `ModId`s to their `ModInfo`s.
fn generate_item_mods_id_mapping() -> io::Result<()> {
    let mut out = create_out_file(ITEM_MODS_DATABASE_FILE)?;

    // TODO: definitely split the hashmap between mod types, since there are more than 3k of them
    // and the vast majority is in the "explicit" category,
    // AND we are looking them by mod types, too
    writeln!(out, "{{")?;
    writeln!(out, "let mut hm = HashMap::new();")?;
    for &mod_type in ITEM_MODS_TYPES.iter() {
        let data_file = Path::new(".").join(ITEM_MODS_DATA_DIR).join(mod_type).with_extension("json");
        let file = fs::OpenOptions::new().read(true).open(data_file)?;
        let mods: Vec<ItemModData> = serde_json::from_reader(file)?;
        for imd in mods {
            writeln!(out, "hm.insert(")?;
            writeln!(out, r#"    ModId::from_str("{}").unwrap(),"#, imd.id)?;
            writeln!(out, r#"    Arc::new(ModInfo::from_raw("{}", "{}").unwrap()),"#,
                imd.id, imd.text)?;
            writeln!(out, ");")?;
        }
    }
    writeln!(out, "hm")?;
    writeln!(out, "}}")?;
    Ok(())
}

/// Structure describing the JSON objects in item mod files.
#[derive(Debug, Deserialize)]
struct ItemModData {
    #[serde(rename = "type")]
    type_: String,
    text: String,
    id: String,
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
