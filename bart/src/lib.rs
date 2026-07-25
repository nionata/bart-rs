//! BART API client library.
//!
//! Typed access to the [BART Legacy API](https://api.bart.gov/docs/overview/index.aspx),
//! covering stations, routes, and real-time departure estimates. The library uses
//! the public demo key by default, which is sufficient for personal and development use.
//!
//! # Why the Legacy API and not GTFS?
//!
//! The BART GTFS static feed is scheduled data only — it cannot tell you when the
//! next train will actually leave. The GTFS Realtime extension exists, but BART's
//! documentation for it is sparse and it requires decoding a protobuf binary format
//! (an extra dependency and protocol to learn). The Legacy API's `/etd` endpoint is
//! the authoritative source of **real-time departure estimates**, is JSON, and works
//! immediately with the public demo key. For a real-time departure board, the Legacy
//! API is the right choice.
//!
//! # Example
//!
//! ```no_run
//! use bart::BartClient;
//!
//! #[tokio::main]
//! async fn main() -> bart::Result<()> {
//!     let client = BartClient::new();
//!
//!     for station in client.stations().await? {
//!         println!("{} ({})", station.name, station.abbr);
//!     }
//!
//!     // Northbound trains from Glen Park
//!     for stn in client.estimates_filtered("GLEN", "n").await? {
//!         for etd in &stn.etd {
//!             println!("→ {}: {}", etd.destination, etd.estimate[0].minutes);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;
pub(crate) mod parse;

pub use client::BartClient;
pub use error::{Error, Result};
pub use models::{Direction, DirectionGroups, Estimate, Etd, Minutes, Route, Station, StationEtd};
