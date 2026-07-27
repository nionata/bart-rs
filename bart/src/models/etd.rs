//! Real-time departure estimate data returned by the BART ETD endpoint.

use crate::models::{
    common::{hex_to_rgb, Direction},
    serde_helpers,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Departure time for a single train.
///
/// The API represents time as a numeric string (minutes until departure) or the
/// literal string `"Leaving"` when the train is currently at the platform.
///
/// # Examples
///
/// ```
/// use bart::Minutes;
///
/// let leaving: Minutes = serde_json::from_str(r#""Leaving""#).unwrap();
/// assert_eq!(leaving.to_string(), "Leaving");
/// assert_eq!(leaving.as_mins(), None);
///
/// let soon: Minutes = serde_json::from_str(r#""8""#).unwrap();
/// assert_eq!(soon.to_string(), "8 min");
/// assert_eq!(soon.as_mins(), Some(8));
/// ```
#[derive(Debug, Clone)]
pub enum Minutes {
    /// Train is currently at the platform.
    Leaving,
    /// Minutes until departure.
    Mins(u32),
}

impl Minutes {
    /// Returns `Some(n)` for [`Mins`](Minutes::Mins), or `None` for [`Leaving`](Minutes::Leaving).
    pub fn as_mins(&self) -> Option<u32> {
        match self {
            Minutes::Leaving => None,
            Minutes::Mins(n) => Some(*n),
        }
    }
}

impl Serialize for Minutes {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Minutes::Leaving => s.serialize_str("Leaving"),
            Minutes::Mins(n) => s.serialize_u32(*n),
        }
    }
}

impl<'de> Deserialize<'de> for Minutes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "Leaving" {
            Ok(Minutes::Leaving)
        } else {
            s.parse::<u32>()
                .map(Minutes::Mins)
                .map_err(|_| serde::de::Error::custom(format!("unexpected minutes value: {s}")))
        }
    }
}

impl fmt::Display for Minutes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Minutes::Leaving => write!(f, "Leaving"),
            Minutes::Mins(m) => write!(f, "{m} min"),
        }
    }
}

/// A single upcoming departure for a destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    /// Minutes until departure; see [`Minutes`].
    pub minutes: Minutes,
    /// Platform number.
    #[serde(deserialize_with = "serde_helpers::numeric_str_u8")]
    pub platform: u8,
    /// Direction of travel.
    pub direction: Direction,
    /// Number of cars in the train.
    #[serde(deserialize_with = "serde_helpers::numeric_str_u8")]
    pub length: u8,
    /// Color name for the line, e.g. `"YELLOW"`.
    pub color: String,
    /// Hex color code, e.g. `"#ffff33"`.
    pub hexcolor: String,
    /// Whether bikes are permitted on this train.
    #[serde(deserialize_with = "serde_helpers::flag_bool")]
    pub bikeflag: bool,
    /// Delay in seconds (`0` if on time).
    #[serde(deserialize_with = "serde_helpers::numeric_str_u32")]
    pub delay: u32,
    /// Whether this departure has been cancelled.
    #[serde(deserialize_with = "serde_helpers::flag_bool")]
    pub cancelflag: bool,
    /// Whether this departure is dynamically scheduled.
    #[serde(deserialize_with = "serde_helpers::flag_bool")]
    pub dynamicflag: bool,
}

impl Estimate {
    /// Returns `true` if the train is running behind schedule.
    pub fn is_delayed(&self) -> bool {
        self.delay > 0
    }

    /// Returns `true` if this departure has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelflag
    }

    /// Returns `true` if bikes are permitted on this train.
    pub fn bikes_allowed(&self) -> bool {
        self.bikeflag
    }

    /// Parses [`hexcolor`](Self::hexcolor) into `(r, g, b)` components.
    pub fn rgb(&self) -> Option<(u8, u8, u8)> {
        hex_to_rgb(&self.hexcolor)
    }
}

/// Upcoming departures from one station toward a single destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Etd {
    /// Destination station name, e.g. `"Antioch"`.
    pub destination: String,
    /// Destination station abbreviation, e.g. `"ANTC"`.
    pub abbreviation: String,
    /// Whether this is limited service (not all stops served).
    #[serde(deserialize_with = "serde_helpers::flag_bool")]
    pub limited: bool,
    /// Next departures toward this destination (typically three).
    pub estimate: Vec<Estimate>,
}

impl Etd {
    /// Minutes until the next departure, or `None` if the train is already leaving.
    pub fn next_mins(&self) -> Option<u32> {
        self.estimate.first().and_then(|e| e.minutes.as_mins())
    }
}

/// Departure estimates for a station, grouped by destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEtd {
    /// Station name, e.g. `"Glen Park"`.
    pub name: String,
    /// Station abbreviation, e.g. `"GLEN"`.
    pub abbr: String,
    /// Departures grouped by destination. Empty when the API reports no service.
    #[serde(default)]
    pub etd: Vec<Etd>,
}

/// Departures from a station split into northbound and southbound groups.
///
/// Returned by [`StationEtd::by_direction`]. Each group is sorted by next
/// departure time, with trains already at the platform appearing first.
#[derive(Debug)]
pub struct DirectionGroups<'a> {
    /// Northbound departures, sorted by next departure time.
    pub north: Vec<&'a Etd>,
    /// Southbound departures, sorted by next departure time.
    pub south: Vec<&'a Etd>,
}

impl StationEtd {
    /// Splits and sorts departures into northbound and southbound groups.
    ///
    /// Within each group, destinations are ordered by their next departure time,
    /// with trains already at the platform (`Leaving`) sorted first.
    pub fn by_direction(&self) -> DirectionGroups<'_> {
        let (mut north, mut south): (Vec<&Etd>, Vec<&Etd>) = self.etd.iter().partition(|etd| {
            etd.estimate
                .first()
                .is_some_and(|e| e.direction == Direction::North)
        });
        north.sort_by_key(|e| e.next_mins().unwrap_or(0));
        south.sort_by_key(|e| e.next_mins().unwrap_or(0));
        DirectionGroups { north, south }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de<'de, T: Deserialize<'de>>(s: &'de str) -> T {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn minutes_leaving() {
        let m: Minutes = de(r#""Leaving""#);
        assert!(matches!(m, Minutes::Leaving));
        assert_eq!(m.to_string(), "Leaving");
        assert_eq!(m.as_mins(), None);
    }

    #[test]
    fn minutes_numeric() {
        let m: Minutes = de(r#""8""#);
        assert!(matches!(m, Minutes::Mins(8)));
        assert_eq!(m.to_string(), "8 min");
        assert_eq!(m.as_mins(), Some(8));
    }

    #[test]
    fn minutes_zero() {
        let m: Minutes = de(r#""0""#);
        assert!(matches!(m, Minutes::Mins(0)));
    }

    #[test]
    fn minutes_invalid_rejects() {
        let r: Result<Minutes, _> = serde_json::from_str(r#""soon""#);
        assert!(r.is_err());
    }

    fn estimate_with_hex(hex: &str) -> Estimate {
        serde_json::from_value(serde_json::json!({
            "minutes": "5",
            "platform": "1",
            "direction": "North",
            "length": "6",
            "color": "YELLOW",
            "hexcolor": hex,
            "bikeflag": "1",
            "delay": "0",
            "cancelflag": "0",
            "dynamicflag": "0"
        }))
        .unwrap()
    }

    #[test]
    fn estimate_rgb_parses_hexcolor() {
        let e = estimate_with_hex("#FFFF33");
        assert_eq!(e.rgb(), Some((0xFF, 0xFF, 0x33)));
    }

    #[test]
    fn estimate_rgb_returns_none_for_invalid_hex() {
        let e = estimate_with_hex("bad");
        assert_eq!(e.rgb(), None);
    }
}
