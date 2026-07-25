//! Route data returned by the BART routes endpoint.

use crate::models::{
    common::{hex_to_rgb, Direction},
    serde_helpers,
};
use serde::{Deserialize, Serialize};

/// A single BART route (one direction of a line).
///
/// Each physical line (Blue, Yellow, etc.) has two routes: one northbound and one southbound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Full route name, e.g. `"Daly City to Dublin/Pleasanton"`.
    pub name: String,
    /// Origin–destination code, e.g. `"DALY-DUBL"`.
    pub abbr: String,
    /// Internal route identifier, e.g. `"ROUTE 12"`.
    #[serde(rename = "routeID")]
    pub route_id: String,
    /// Route number, e.g. `12`.
    #[serde(deserialize_with = "serde_helpers::numeric_str_u8")]
    pub number: u8,
    /// Hex color code for the line, e.g. `"#0099CC"`.
    pub hexcolor: String,
    /// Color name for the line, e.g. `"BLUE"`.
    pub color: String,
    /// Direction of travel.
    pub direction: Direction,
}

impl Route {
    /// Parses [`hexcolor`](Self::hexcolor) into `(r, g, b)` components.
    pub fn rgb(&self) -> Option<(u8, u8, u8)> {
        hex_to_rgb(&self.hexcolor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route_with_hex(hex: &str) -> Route {
        serde_json::from_value(serde_json::json!({
            "name": "Test Route",
            "abbr": "TEST",
            "routeID": "ROUTE 1",
            "number": "1",
            "hexcolor": hex,
            "color": "BLUE",
            "direction": "North"
        }))
        .unwrap()
    }

    #[test]
    fn rgb_parses_hexcolor() {
        let r = route_with_hex("#0099CC");
        assert_eq!(r.rgb(), Some((0x00, 0x99, 0xCC)));
    }

    #[test]
    fn rgb_returns_none_for_invalid_hex() {
        let r = route_with_hex("not-a-color");
        assert_eq!(r.rgb(), None);
    }
}
