//! JSON parsing functions for BART API responses.
//!
//! Each function accepts a raw [`serde_json::Value`] — exactly what the BART
//! Legacy API returns — and deserializes it into typed model structs.
//!
//! [`BartClient`](crate::BartClient) calls these after every HTTP request.
//! They are also `pub` so callers can parse cached or fixture responses
//! without going through the client.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    error::{Error, Result},
    models::{etd::StationEtd, route::Route, station::Station},
};

/// Parses a BART stations response into a list of [`Station`]s.
///
/// Expects the JSON envelope returned by `/api/stn.aspx?cmd=stns`.
pub fn stations(json: &Value) -> Result<Vec<Station>> {
    check_api_error(json)?;
    extract(json, "/root/stations/station")
}

/// Parses a BART routes response into a list of [`Route`]s.
///
/// Expects the JSON envelope returned by `/api/route.aspx?cmd=routes`.
pub fn routes(json: &Value) -> Result<Vec<Route>> {
    check_api_error(json)?;
    extract(json, "/root/routes/route")
}

/// Parses a BART ETD response into a list of [`StationEtd`]s.
///
/// Expects the JSON envelope returned by `/api/etd.aspx?cmd=etd`.
pub fn etd(json: &Value) -> Result<Vec<StationEtd>> {
    check_api_error(json)?;
    extract(json, "/root/station")
}

/// Surfaces an application-level error from `root.message.error` if present.
///
/// The BART API returns HTTP 200 even for errors; the error lives in the body.
fn check_api_error(json: &Value) -> Result<()> {
    if let Some(err) = json.pointer("/root/message/error") {
        let text = err["text"].as_str().unwrap_or("unknown error");
        let details = err["details"].as_str().unwrap_or("");
        let msg = if details.is_empty() {
            text.to_string()
        } else {
            format!("{text}: {details}")
        };
        return Err(Error::Api(msg));
    }
    Ok(())
}

/// Navigates `json` with a JSON Pointer and deserializes the node.
fn extract<T: DeserializeOwned>(json: &Value, pointer: &str) -> Result<T> {
    let node = json
        .pointer(pointer)
        .ok_or_else(|| Error::Api(format!("missing field at '{pointer}'")))?
        .clone();
    Ok(serde_json::from_value(node)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Direction;

    fn fixture(name: &str) -> Value {
        let s = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();
        serde_json::from_str(&s).unwrap()
    }

    fn error_json(text: &str, details: &str) -> Value {
        serde_json::json!({
            "root": {
                "message": {
                    "error": { "text": text, "details": details }
                }
            }
        })
    }

    // ── check_api_error ───────────────────────────────────────────────────────

    #[test]
    fn api_error_ok_on_normal_response() {
        let json = serde_json::json!({"root": {"stations": {}}});
        assert!(check_api_error(&json).is_ok());
    }

    #[test]
    fn api_error_surfaces_text() {
        let json = error_json("Invalid station", "");
        let Err(Error::Api(msg)) = check_api_error(&json) else {
            panic!("expected Err")
        };
        assert_eq!(msg, "Invalid station");
    }

    #[test]
    fn api_error_appends_details() {
        let json = error_json("Bad request", "check your API key");
        let Err(Error::Api(msg)) = check_api_error(&json) else {
            panic!("expected Err")
        };
        assert_eq!(msg, "Bad request: check your API key");
    }

    #[test]
    fn api_error_omits_separator_when_details_empty() {
        let json = error_json("Something went wrong", "");
        let Err(Error::Api(msg)) = check_api_error(&json) else {
            panic!("expected Err")
        };
        assert!(!msg.contains(':'));
    }

    // ── extract ───────────────────────────────────────────────────────────────

    #[test]
    fn extract_missing_pointer_is_api_error() {
        let json = serde_json::json!({"root": {}});
        let r: Result<Vec<Station>> = extract(&json, "/root/stations/station");
        assert!(matches!(r, Err(Error::Api(_))));
    }

    #[test]
    fn extract_type_mismatch_is_json_error() {
        let json = serde_json::json!({"root": {"stations": {"station": "not an array"}}});
        let r: Result<Vec<Station>> = extract(&json, "/root/stations/station");
        assert!(matches!(r, Err(Error::Json(_))));
    }

    // ── stations ──────────────────────────────────────────────────────────────

    #[test]
    fn stations_parses_fixture() {
        assert!(!stations(&fixture("stations.json")).unwrap().is_empty());
    }

    #[test]
    fn stations_all_have_name_and_abbr() {
        for s in stations(&fixture("stations.json")).unwrap() {
            assert!(!s.name.is_empty());
            assert!(!s.abbr.is_empty());
        }
    }

    #[test]
    fn stations_abbr_is_uppercase() {
        for s in stations(&fixture("stations.json")).unwrap() {
            assert_eq!(s.abbr, s.abbr.to_uppercase());
        }
    }

    #[test]
    fn stations_coordinates_parse_as_f64() {
        for s in stations(&fixture("stations.json")).unwrap() {
            s.gtfs_latitude.parse::<f64>().unwrap_or_else(|_| {
                panic!("{} latitude '{}' is not a float", s.abbr, s.gtfs_latitude)
            });
            s.gtfs_longitude.parse::<f64>().unwrap_or_else(|_| {
                panic!("{} longitude '{}' is not a float", s.abbr, s.gtfs_longitude)
            });
        }
    }

    #[test]
    fn stations_all_in_california() {
        for s in stations(&fixture("stations.json")).unwrap() {
            assert_eq!(s.state, "CA");
        }
    }

    #[test]
    fn stations_known_fields() {
        let stns = stations(&fixture("stations.json")).unwrap();
        let glen = stns.iter().find(|s| s.abbr == "GLEN").unwrap();
        assert_eq!(glen.name, "Glen Park");
        assert_eq!(glen.city, "San Francisco");
        let twelfth = stns.iter().find(|s| s.abbr == "12TH").unwrap();
        assert_eq!(twelfth.city, "Oakland");
    }

    #[test]
    fn stations_error_response() {
        assert!(matches!(
            stations(&error_json("Invalid", "")),
            Err(Error::Api(_))
        ));
    }

    // ── routes ────────────────────────────────────────────────────────────────

    #[test]
    fn routes_parses_fixture() {
        assert!(!routes(&fixture("routes.json")).unwrap().is_empty());
    }

    #[test]
    fn routes_all_have_name() {
        for r in routes(&fixture("routes.json")).unwrap() {
            assert!(!r.name.is_empty());
        }
    }

    #[test]
    fn routes_number_is_nonzero() {
        for r in routes(&fixture("routes.json")).unwrap() {
            assert!(r.number > 0);
        }
    }

    #[test]
    fn routes_hexcolor_starts_with_hash() {
        for r in routes(&fixture("routes.json")).unwrap() {
            assert!(
                r.hexcolor.starts_with('#'),
                "hexcolor '{}' should start with #",
                r.hexcolor
            );
        }
    }

    #[test]
    fn routes_come_in_north_south_pairs() {
        let rs = routes(&fixture("routes.json")).unwrap();
        let north = rs
            .iter()
            .filter(|r| r.direction == Direction::North)
            .count();
        let south = rs
            .iter()
            .filter(|r| r.direction == Direction::South)
            .count();
        assert_eq!(north, south);
    }

    #[test]
    fn routes_multiple_distinct_colors() {
        let rs = routes(&fixture("routes.json")).unwrap();
        let colors: std::collections::HashSet<&str> = rs.iter().map(|r| r.color.as_str()).collect();
        assert!(colors.len() > 1);
    }

    #[test]
    fn routes_error_response() {
        assert!(matches!(
            routes(&error_json("Invalid", "")),
            Err(Error::Api(_))
        ));
    }

    // ── etd ───────────────────────────────────────────────────────────────────

    #[test]
    fn etd_parses_fixture() {
        let etds = etd(&fixture("etd.json")).unwrap();
        assert!(!etds.is_empty());
    }

    #[test]
    fn etd_fixture_is_glen_park() {
        let etds = etd(&fixture("etd.json")).unwrap();
        assert_eq!(etds[0].abbr, "GLEN");
        assert_eq!(etds[0].name, "Glen Park");
    }

    #[test]
    fn etd_has_multiple_destinations() {
        let etds = etd(&fixture("etd.json")).unwrap();
        assert!(etds[0].etd.len() > 1);
    }

    #[test]
    fn etd_each_destination_has_estimates() {
        for stn in etd(&fixture("etd.json")).unwrap() {
            for dest in &stn.etd {
                assert!(
                    !dest.estimate.is_empty(),
                    "{} has no estimates",
                    dest.destination
                );
            }
        }
    }

    #[test]
    fn etd_known_destinations_present() {
        let etds = etd(&fixture("etd.json")).unwrap();
        let dests: Vec<&str> = etds[0].etd.iter().map(|e| e.destination.as_str()).collect();
        assert!(dests.contains(&"Antioch"));
        assert!(dests.contains(&"Richmond"));
    }

    #[test]
    fn etd_delayed_trains_detected() {
        let etds = etd(&fixture("etd.json")).unwrap();
        let any_delayed = etds
            .iter()
            .flat_map(|s| &s.etd)
            .flat_map(|e| &e.estimate)
            .any(|est| est.is_delayed());
        assert!(
            any_delayed,
            "fixture should contain at least one delayed train"
        );
    }

    #[test]
    fn etd_fields_are_typed() {
        for stn in etd(&fixture("etd.json")).unwrap() {
            for dest in &stn.etd {
                for est in &dest.estimate {
                    let _: u32 = est.delay;
                    let _: u8 = est.platform;
                    let _: u8 = est.length;
                    let _: bool = est.bikeflag;
                    assert!(est.length > 0);
                }
            }
        }
    }

    #[test]
    fn etd_error_response() {
        assert!(matches!(
            etd(&error_json("Invalid station", "NOPE is not valid")),
            Err(Error::Api(_))
        ));
    }

    #[test]
    fn etd_invalid_station_fixture_is_api_error() {
        let Err(Error::Api(msg)) = etd(&fixture("etd_invalid_station.json")) else {
            panic!("expected Err(Api)")
        };
        assert!(msg.contains("Invalid orig"));
        assert!(msg.contains("GLENA"));
    }

    #[test]
    fn etd_no_service_parses_without_error() {
        let etds = etd(&fixture("etd_no_service.json")).unwrap();
        assert_eq!(etds.len(), 1);
        assert_eq!(etds[0].abbr, "FRMT");
        assert_eq!(etds[0].name, "Fremont");
        assert!(etds[0].etd.is_empty());
    }
}
