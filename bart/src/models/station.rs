//! Station data returned by the BART stations endpoint.

use serde::{Deserialize, Serialize};

/// A single BART station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    /// Full station name, e.g. `"Glen Park"`.
    pub name: String,
    /// Station abbreviation used as the station ID in API calls, e.g. `"GLEN"`.
    pub abbr: String,
    /// Latitude per the GTFS feed (parse to `f64` as needed).
    pub gtfs_latitude: String,
    /// Longitude per the GTFS feed (parse to `f64` as needed).
    pub gtfs_longitude: String,
    /// Street address.
    pub address: String,
    /// City name.
    pub city: String,
    /// County name (inconsistently cased across stations in the API response).
    pub county: String,
    /// State abbreviation (`"CA"` for all current stations).
    pub state: String,
    /// ZIP code.
    pub zipcode: String,
}
