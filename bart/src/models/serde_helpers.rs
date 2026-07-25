//! Private serde helpers for deserializing BART API's stringly-typed fields.
//!
//! The BART API represents booleans as `"0"` / `"1"` strings and numeric values
//! (platform, car count, delay) as numeric strings. These helpers convert them
//! to proper Rust types via `#[serde(deserialize_with = "...")]`.

use serde::Deserialize;

pub fn flag_bool<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    match String::deserialize(d)?.as_str() {
        "1" => Ok(true),
        "0" => Ok(false),
        other => Err(serde::de::Error::custom(format!(
            "expected '0' or '1', got '{other}'"
        ))),
    }
}

pub fn numeric_str_u8<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    String::deserialize(d)?
        .parse::<u8>()
        .map_err(serde::de::Error::custom)
}

pub fn numeric_str_u32<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    String::deserialize(d)?
        .parse::<u32>()
        .map_err(serde::de::Error::custom)
}
