//! Build script.
//!
//! The script takes static data obtained from the API and generates some code
//! for inclusion in the library source, including e.g. the `Currency` enum.

#[macro_use] extern crate ezomyte_build;  // auxiliary crate in `build/`
             extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
             extern crate serde;
#[macro_use] extern crate serde_derive;
             extern crate serde_json;


use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use ezomyte_build::codegen;
use ezomyte_build::files::create_out_file;
use ezomyte_build::strings::upper_camel_case;
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
    let out = create_out_file(CURRENCY_ENUM_FILE)?;
    let mut ctx = codegen::Context::new(out);

    ctx.emit("/// Currency item used for trading.")?;
    ctx.emit("#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]")?;
    ctx.begin("pub enum Currency {")?;
    for currency in currencies {
        emit!(ctx, "/// {}.", currency.name)?;
        emit!(ctx, "#[serde(rename=\"{}\")]", currency.id)?;
        emit!(ctx, "{},", upper_camel_case(&currency.name))?;
    }
    ctx.end("}")?;
    Ok(())
}

/// Generate the visit_str() method branches of Currency deserializer.
fn generate_currency_de(currencies: &[CurrencyData]) -> io::Result<()> {
    let out = create_out_file(CURRENCY_DE_FILE)?;
    let mut ctx = codegen::Context::new(out);

    ctx.begin("match v {")?;
    for currency in currencies {
        // Include possible alternative/community "names" for the currencies
        // so that they are deserialized correctly.
        let mut patterns: Vec<&str> = vec![&currency.id];
        if let Some(more) = ADDITIONAL_CURRENCY_IDS.get(&currency.id.as_str()) {
            patterns.extend(more.iter());
        }
        emit!(ctx, "{} => Ok(Currency::{}),",
            patterns.into_iter().format_with(" | ", |x, f| f(&format_args!("\"{}\"", x))),
            upper_camel_case(&currency.name))?;
    }
    ctx.emit("v => Err(de::Error::invalid_value(Unexpected::Str(v), &EXPECTING_MSG)),")?;
    ctx.end("}")?;
    Ok(())
}


// Item mod handling

const ITEM_MODS_DATA_DIR: &str = "data/mods";
const ITEM_MODS_TYPES: &[&str] = &["crafted", "enchant", "explicit", "implicit"];
const ITEM_MODS_DATABASE_FILE: &str = "model/item/mods/database/by_type_and_id.inc.rs";

/// Generate code for the database of `ModInfo`s.
fn generate_item_mod_code() -> Result<(), Box<Error>> {
    generate_item_mods_mapping()?;
    Ok(())
}

/// Generate the mapping of all `ModType`s to mappings of `ModId`s to their `ModInfo`s
/// (i.e. a `HashMap<ModType, HashMap<ModId, Arc<ModInfo>>>`).
fn generate_item_mods_mapping() -> io::Result<()> {
    let out = create_out_file(ITEM_MODS_DATABASE_FILE)?;
    let mut ctx = codegen::Context::new(out);

    ctx.begin("{")?;
    ctx.emit("let mut hm = HashMap::new();")?;
    for &mod_type in ITEM_MODS_TYPES.iter() {
        ctx.begin("hm.insert(")?;
        emit!(ctx, r#"ModType::from_str("{}").unwrap(),"#, mod_type)?;
        generate_item_mods_mapping_for_type(&mut ctx, mod_type)?;
        ctx.end(");")?;
    }
    ctx.emit("hm")?;
    ctx.end("}")?;
    Ok(())
}

/// Generate mapping of `ModId`s of given `ModType` to their `ModInfo`s.
fn generate_item_mods_mapping_for_type<W: io::Write>(
    ctx: &mut codegen::Context<W>, mod_type: &str
) -> io::Result<()> {
    let data_file = Path::new(".").join(ITEM_MODS_DATA_DIR).join(mod_type).with_extension("json");
    let file = fs::OpenOptions::new().read(true).open(data_file)?;
    let mods: Vec<ItemModData> = serde_json::from_reader(file)?;

    ctx.begin("{")?;
    ctx.emit("let mut hm = HashMap::new();")?;
    for imd in mods {
        // Some mod texts contain newlines because they're used in the UI.
        let text = imd.text.replace('\n', " ");
        ctx.begin("hm.insert(")?;
        emit!(ctx, r#"ModId::from_str("{}").unwrap(),"#, imd.id)?;
        emit!(ctx, r#"Arc::new(ModInfo::from_raw("{}", "{}").unwrap()),"#,
            imd.id, text)?;
        ctx.end(");")?;
    }
    ctx.emit("hm")?;
    ctx.end("}")?;
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
