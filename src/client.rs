//! HTTP client for the BART Legacy API.

use serde_json::Value;

use crate::{
    error::Result,
    models::{etd::StationEtd, route::Route, station::Station},
};

const BASE_URL: &str = "https://api.bart.gov";
// BART's public demo key — see https://www.bart.gov/schedules/developers/api
const PUBLIC_KEY: &str = "MW9S-E7SL-26DU-VV8V";

/// Async client for the BART Legacy API.
///
/// Construct with [`BartClient::new`] to use the public demo key, or
/// [`BartClient::with_key`] if you have a registered key.
///
/// # Example
///
/// ```no_run
/// use bart::BartClient;
///
/// #[tokio::main]
/// async fn main() -> bart::Result<()> {
///     let client = BartClient::new();
///     let stations = client.stations().await?;
///     assert!(!stations.is_empty());
///     Ok(())
/// }
/// ```
pub struct BartClient {
    client: reqwest::Client,
    api_key: String,
}

impl BartClient {
    /// Creates a client using the public BART demo API key.
    pub fn new() -> Self {
        Self::with_key(PUBLIC_KEY)
    }

    /// Creates a client with a specific API key.
    pub fn with_key(key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: key.into(),
        }
    }

    /// Returns all BART stations, sorted alphabetically by name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bart::BartClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> bart::Result<()> {
    ///     let stations = BartClient::new().stations().await?;
    ///     let embr = stations.iter().find(|s| s.abbr == "EMBR").unwrap();
    ///     println!("{} is at {}, {}", embr.name, embr.city, embr.state);
    ///     Ok(())
    /// }
    /// ```
    pub async fn stations(&self) -> Result<Vec<Station>> {
        let json = self.get("/api/stn.aspx", &[("cmd", "stns")]).await?;
        crate::parse::stations(&json)
    }

    /// Returns all active BART routes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bart::BartClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> bart::Result<()> {
    ///     let routes = BartClient::new().routes().await?;
    ///     for r in &routes {
    ///         println!("[{}] {} ({})", r.color, r.name, r.direction);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn routes(&self) -> Result<Vec<Route>> {
        let json = self.get("/api/route.aspx", &[("cmd", "routes")]).await?;
        crate::parse::routes(&json)
    }

    /// Returns real-time departure estimates for a station, grouped by destination.
    ///
    /// `orig` is a station abbreviation such as `"GLEN"`, `"EMBR"`, or `"12TH"`.
    /// Use [`stations`](Self::stations) to look up abbreviations.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bart::BartClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> bart::Result<()> {
    ///     let etds = BartClient::new().estimates("GLEN").await?;
    ///     for stn in &etds {
    ///         for etd in &stn.etd {
    ///             let next = &etd.estimate[0];
    ///             println!("→ {}: {} on platform {}", etd.destination, next.minutes, next.platform);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn estimates(&self, orig: &str) -> Result<Vec<StationEtd>> {
        let json = self
            .get("/api/etd.aspx", &[("cmd", "etd"), ("orig", orig)])
            .await?;
        crate::parse::etd(&json)
    }

    /// Returns real-time departure estimates filtered by direction.
    ///
    /// `orig` is a station abbreviation. `dir` is `"n"` (north) or `"s"` (south).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bart::BartClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> bart::Result<()> {
    ///     let etds = BartClient::new().estimates_filtered("EMBR", "n").await?;
    ///     for stn in &etds {
    ///         for etd in &stn.etd {
    ///             println!("→ {}: {}", etd.destination, etd.estimate[0].minutes);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn estimates_filtered(&self, orig: &str, dir: &str) -> Result<Vec<StationEtd>> {
        let json = self
            .get(
                "/api/etd.aspx",
                &[("cmd", "etd"), ("orig", orig), ("dir", dir)],
            )
            .await?;
        crate::parse::etd(&json)
    }

    /// Makes an authenticated GET request and returns the raw JSON body.
    async fn get(&self, path: &str, extra: &[(&str, &str)]) -> Result<Value> {
        let mut params = vec![("key", self.api_key.as_str()), ("json", "y")];
        params.extend_from_slice(extra);
        Ok(self
            .client
            .get(format!("{BASE_URL}{path}"))
            .query(&params)
            .send()
            .await?
            .json()
            .await?)
    }
}

impl Default for BartClient {
    fn default() -> Self {
        Self::new()
    }
}
