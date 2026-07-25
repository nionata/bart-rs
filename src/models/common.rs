//! Shared enums used across multiple API endpoints.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Direction of travel on the BART network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::South => write!(f, "South"),
        }
    }
}

/// Parses a hex color string into `(r, g, b)` components.
///
/// Accepts strings with or without a leading `#`. Returns `None` if the input
/// is not a valid 6-digit hex color.
pub(crate) fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&h[0..2], 16).ok()?,
        u8::from_str_radix(&h[2..4], 16).ok()?,
        u8::from_str_radix(&h[4..6], 16).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_roundtrips() {
        let n: Direction = serde_json::from_str(r#""North""#).unwrap();
        assert_eq!(n, Direction::North);
        assert_eq!(n.to_string(), "North");

        let s: Direction = serde_json::from_str(r#""South""#).unwrap();
        assert_eq!(s, Direction::South);
    }

    #[test]
    fn hex_to_rgb_with_hash() {
        assert_eq!(hex_to_rgb("#FF9933"), Some((0xFF, 0x99, 0x33)));
    }

    #[test]
    fn hex_to_rgb_without_hash() {
        assert_eq!(hex_to_rgb("0099CC"), Some((0x00, 0x99, 0xCC)));
    }

    #[test]
    fn hex_to_rgb_rejects_short() {
        assert_eq!(hex_to_rgb("#FFF"), None);
    }

    #[test]
    fn hex_to_rgb_rejects_invalid_chars() {
        assert_eq!(hex_to_rgb("#ZZZZZZ"), None);
    }
}
