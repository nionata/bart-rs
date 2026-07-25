//! Data models for the BART Legacy API.
//!
//! The top-level types are [`Station`], [`Route`], and [`StationEtd`] (with its
//! nested [`Etd`] and [`Estimate`]). All are returned directly by [`BartClient`](crate::BartClient)
//! methods and implement [`Clone`] and [`Debug`].

pub mod common;
pub mod etd;
pub mod route;
pub(crate) mod serde_helpers;
pub mod station;

pub use common::Direction;
pub use etd::{DirectionGroups, Estimate, Etd, Minutes, StationEtd};
pub use route::Route;
pub use station::Station;
