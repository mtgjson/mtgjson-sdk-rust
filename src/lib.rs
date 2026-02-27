//! MTGJSON SDK for Rust.
//!
//! Provides a high-level client for querying the complete MTGJSON dataset.
//! Data is downloaded from the MTGJSON CDN as parquet and JSON files, cached
//! locally, and queried in-process via DuckDB.
//!
//! # Quick start
//!
//! ```no_run
//! use mtgjson_sdk::MtgjsonSdk;
//!
//! let mut sdk = MtgjsonSdk::builder().build().unwrap();
//!
//! // Query cards
//! let cards = sdk.cards().get_by_name("Lightning Bolt", None).unwrap();
//!
//! // Open a draft booster
//! let pack = sdk.booster().open_pack("MH3", "draft").unwrap();
//! ```

#[cfg(feature = "async")]
pub mod async_client;
pub mod booster;
pub mod cache;
pub mod config;
pub mod connection;
pub mod error;
pub mod models;
pub mod queries;
pub mod sql_builder;

#[cfg(feature = "async")]
pub use async_client::AsyncMtgjsonSdk;
pub use cache::CacheManager;
pub use connection::Connection;
pub use error::{MtgjsonError, Result};
pub use sql_builder::SqlBuilder;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

// ---------------------------------------------------------------------------
// MtgjsonSdkBuilder
// ---------------------------------------------------------------------------

/// Builder for configuring and constructing an [`MtgjsonSdk`] instance.
///
/// Use [`MtgjsonSdk::builder()`] to obtain a builder, chain configuration
/// methods, and call [`build()`](MtgjsonSdkBuilder::build) to create the SDK.
pub struct MtgjsonSdkBuilder {
    cache_dir: Option<PathBuf>,
    offline: bool,
    timeout: Duration,
}

impl Default for MtgjsonSdkBuilder {
    fn default() -> Self {
        Self {
            cache_dir: None,
            offline: false,
            timeout: Duration::from_secs(120),
        }
    }
}

impl MtgjsonSdkBuilder {
    /// Set a custom cache directory.
    ///
    /// If not set, the platform-appropriate default cache directory is used
    /// (e.g. `~/.cache/mtgjson-sdk` on Linux, `~/Library/Caches/mtgjson-sdk`
    /// on macOS, `%LOCALAPPDATA%\mtgjson-sdk` on Windows).
    pub fn cache_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.cache_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable or disable offline mode.
    ///
    /// When offline, the SDK never downloads from the CDN and only uses
    /// previously cached data files. Defaults to `false`.
    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Set the HTTP request timeout for CDN downloads.
    ///
    /// Defaults to 120 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the SDK, initializing the cache and DuckDB connection.
    ///
    /// This may trigger a version check against the CDN (unless offline mode
    /// is enabled) but does **not** download any data files eagerly -- they
    /// are fetched lazily on first query.
    pub fn build(self) -> Result<MtgjsonSdk> {
        let cache = CacheManager::new(self.cache_dir, self.offline, self.timeout)?;
        let conn = Connection::new(cache)?;
        Ok(MtgjsonSdk { conn })
    }
}

// ---------------------------------------------------------------------------
// MtgjsonSdk
// ---------------------------------------------------------------------------

/// The main entry point for the MTGJSON SDK.
///
/// Wraps a [`Connection`] (which owns the [`CacheManager`] and DuckDB database)
/// and exposes domain-specific query interfaces as lightweight borrowing wrappers.
///
/// Created via [`MtgjsonSdk::builder()`].
pub struct MtgjsonSdk {
    conn: Connection,
}

impl MtgjsonSdk {
    /// Create a new builder for configuring the SDK.
    pub fn builder() -> MtgjsonSdkBuilder {
        MtgjsonSdkBuilder::default()
    }

    // -- Query accessors ---------------------------------------------------

    /// Access the card query interface.
    ///
    /// Returns a lightweight wrapper that borrows from the underlying
    /// connection and provides methods for querying card data.
    pub fn cards(&self) -> queries::cards::CardQuery<'_> {
        queries::cards::CardQuery::new(&self.conn)
    }

    /// Access the set query interface.
    pub fn sets(&self) -> queries::sets::SetQuery<'_> {
        queries::sets::SetQuery::new(&self.conn)
    }

    /// Access the token query interface.
    pub fn tokens(&self) -> queries::tokens::TokenQuery<'_> {
        queries::tokens::TokenQuery::new(&self.conn)
    }

    /// Access the price query interface.
    ///
    /// Requires the `prices_today` table to have been loaded into DuckDB.
    pub fn prices(&self) -> queries::prices::PriceQuery<'_> {
        queries::prices::PriceQuery::new(&self.conn)
    }

    /// Access the legality query interface.
    pub fn legalities(&self) -> queries::legalities::LegalityQuery<'_> {
        queries::legalities::LegalityQuery::new(&self.conn)
    }

    /// Access the identifier query interface.
    pub fn identifiers(&self) -> queries::identifiers::IdentifierQuery<'_> {
        queries::identifiers::IdentifierQuery::new(&self.conn)
    }

    /// Access the deck query interface.
    ///
    /// Deck data is loaded from `DeckList.json` via the cache manager.
    pub fn decks(&self) -> queries::decks::DeckQuery<'_> {
        queries::decks::DeckQuery::new(&self.conn)
    }

    /// Access the sealed product query interface.
    pub fn sealed(&self) -> queries::sealed::SealedQuery<'_> {
        queries::sealed::SealedQuery::new(&self.conn)
    }

    /// Access the TCGplayer SKU query interface.
    ///
    /// Requires the `tcgplayer_skus` table to have been loaded into DuckDB.
    pub fn skus(&self) -> queries::skus::SkuQuery<'_> {
        queries::skus::SkuQuery::new(&self.conn)
    }

    /// Access the enum/keyword query interface.
    ///
    /// Enum data is loaded from JSON files (`Keywords.json`, `CardTypes.json`,
    /// `EnumValues.json`) via the cache manager.
    pub fn enums(&self) -> queries::enums::EnumQuery<'_> {
        queries::enums::EnumQuery::new(&self.conn)
    }

    /// Access the booster pack simulator.
    ///
    /// The simulator reads from the set booster parquet tables to generate
    /// randomized booster packs matching real-world distribution rules.
    pub fn booster(&self) -> booster::BoosterSimulator<'_> {
        booster::BoosterSimulator::new(&self.conn)
    }

    // -- Metadata and utility methods --------------------------------------

    /// Load and return the MTGJSON metadata (version, date, etc.).
    ///
    /// Fetches `Meta.json` from the cache (downloading if necessary) and
    /// returns the parsed JSON object.
    pub fn meta(&self) -> Result<serde_json::Value> {
        self.conn.cache.borrow_mut().load_json("meta")
    }

    /// Return the list of currently registered DuckDB view names.
    ///
    /// Views are registered lazily on first query, so this list grows as
    /// different query interfaces are used.
    pub fn views(&self) -> Vec<String> {
        self.conn.views()
    }

    /// Execute a raw SQL query against the DuckDB database.
    ///
    /// Provides escape-hatch access for queries not covered by the
    /// domain-specific interfaces.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL string with `?` positional placeholders.
    /// * `params` - Parameter values corresponding to the placeholders.
    ///
    /// # Returns
    ///
    /// A vector of rows, each represented as a `HashMap<String, serde_json::Value>`.
    pub fn sql(
        &self,
        query: &str,
        params: &[String],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        self.conn.execute(query, params)
    }

    /// Check for a newer MTGJSON version and reset views if stale.
    ///
    /// Returns `true` if the data was stale and views were reset (meaning
    /// subsequent queries will re-download data), or `false` if already
    /// up to date.
    pub fn refresh(&self) -> Result<bool> {
        let stale = self.conn.cache.borrow_mut().is_stale()?;
        if stale {
            self.conn.cache.borrow().clear()?;
            self.conn.reset_views();
            eprintln!("MTGJSON data was stale; cache cleared and views reset");
        }
        Ok(stale)
    }

    /// Consume the SDK and release all resources.
    ///
    /// Closes the DuckDB connection and HTTP client. This is called
    /// automatically when the SDK is dropped, but can be invoked explicitly
    /// for deterministic cleanup.
    pub fn close(self) {
        // Connection and CacheManager are dropped automatically
        drop(self);
    }

    /// Return a reference to the underlying [`Connection`] for advanced usage.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Return a mutable reference to the underlying [`Connection`].
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for MtgjsonSdk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let views = self.conn.views();
        let cache = self.conn.cache.borrow();
        write!(
            f,
            "MtgjsonSdk(cache_dir={}, views=[{}], offline={})",
            cache.cache_dir.display(),
            views.join(", "),
            cache.offline
        )
    }
}
