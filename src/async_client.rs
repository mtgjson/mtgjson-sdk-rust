//! Async wrapper around [`MtgjsonSdk`] for use in async runtimes (Tokio, etc.).
//!
//! Runs all SDK operations on a blocking thread pool via
//! [`tokio::task::spawn_blocking`], keeping the async event loop free.
//! DuckDB queries are CPU-bound but fast, making this approach efficient.
//!
//! # Example
//!
//! ```no_run
//! use mtgjson_sdk::AsyncMtgjsonSdk;
//!
//! #[tokio::main]
//! async fn main() {
//!     let sdk = AsyncMtgjsonSdk::builder().build().await.unwrap();
//!
//!     // Run any sync SDK method via closure
//!     let cards = sdk.run(|s| {
//!         s.cards().get_by_name("Lightning Bolt", None)
//!     }).await.unwrap();
//!
//!     // Convenience method for raw SQL
//!     let rows = sdk.sql("SELECT COUNT(*) FROM cards", &[]).await.unwrap();
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::{MtgjsonError, Result};
use crate::MtgjsonSdk;

// ---------------------------------------------------------------------------
// AsyncMtgjsonSdkBuilder
// ---------------------------------------------------------------------------

/// Builder for configuring and constructing an [`AsyncMtgjsonSdk`] instance.
pub struct AsyncMtgjsonSdkBuilder {
    cache_dir: Option<PathBuf>,
    offline: bool,
    timeout: Duration,
}

impl Default for AsyncMtgjsonSdkBuilder {
    fn default() -> Self {
        Self {
            cache_dir: None,
            offline: false,
            timeout: Duration::from_secs(120),
        }
    }
}

impl AsyncMtgjsonSdkBuilder {
    /// Set a custom cache directory.
    pub fn cache_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.cache_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable or disable offline mode.
    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Set the HTTP request timeout for CDN downloads.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the async SDK, initializing the cache and DuckDB connection.
    ///
    /// Initialization runs on the blocking thread pool so it won't block
    /// the async event loop.
    pub async fn build(self) -> Result<AsyncMtgjsonSdk> {
        tokio::task::spawn_blocking(move || {
            let mut builder = MtgjsonSdk::builder();
            if let Some(dir) = self.cache_dir {
                builder = builder.cache_dir(dir);
            }
            builder = builder.offline(self.offline).timeout(self.timeout);
            let sdk = builder.build()?;
            Ok(AsyncMtgjsonSdk {
                inner: Arc::new(Mutex::new(sdk)),
            })
        })
        .await
        .map_err(|e| MtgjsonError::InvalidArgument(format!("Task join error: {e}")))?
    }
}

// ---------------------------------------------------------------------------
// AsyncMtgjsonSdk
// ---------------------------------------------------------------------------

/// Async wrapper around [`MtgjsonSdk`].
///
/// All operations are dispatched to a blocking thread pool via
/// [`tokio::task::spawn_blocking`]. The underlying [`MtgjsonSdk`] is
/// protected by a [`Mutex`] since it uses `RefCell` internally.
///
/// # Usage
///
/// Use [`run()`](Self::run) to execute any sync SDK method:
///
/// ```no_run
/// # use mtgjson_sdk::AsyncMtgjsonSdk;
/// # async fn example() -> mtgjson_sdk::Result<()> {
/// let sdk = AsyncMtgjsonSdk::builder().build().await?;
/// let sets = sdk.run(|s| s.sets().list(None, None, None, None)).await?;
/// # Ok(())
/// # }
/// ```
pub struct AsyncMtgjsonSdk {
    inner: Arc<Mutex<MtgjsonSdk>>,
}

impl AsyncMtgjsonSdk {
    /// Create a new builder for configuring the async SDK.
    pub fn builder() -> AsyncMtgjsonSdkBuilder {
        AsyncMtgjsonSdkBuilder::default()
    }

    /// Run a sync SDK operation on the blocking thread pool.
    ///
    /// The closure receives an `&MtgjsonSdk` reference and should return
    /// a `Result<T>`. The operation runs on a dedicated blocking thread,
    /// keeping the async event loop free.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use mtgjson_sdk::AsyncMtgjsonSdk;
    /// # async fn example() -> mtgjson_sdk::Result<()> {
    /// # let sdk = AsyncMtgjsonSdk::builder().build().await?;
    /// let cards = sdk.run(|s| {
    ///     s.cards().get_by_name("Lightning Bolt", None)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&MtgjsonSdk) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let sdk = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let guard = sdk
                .lock()
                .map_err(|_| MtgjsonError::InvalidArgument("SDK lock poisoned".into()))?;
            f(&guard)
        })
        .await
        .map_err(|e| MtgjsonError::InvalidArgument(format!("Task join error: {e}")))?
    }

    /// Execute a raw SQL query asynchronously.
    ///
    /// Convenience wrapper around [`run()`](Self::run) for
    /// [`MtgjsonSdk::sql()`].
    pub async fn sql(
        &self,
        query: &str,
        params: &[String],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let query = query.to_string();
        let params = params.to_vec();
        self.run(move |s| s.sql(&query, &params)).await
    }

    /// Load and return the MTGJSON metadata asynchronously.
    pub async fn meta(&self) -> Result<serde_json::Value> {
        self.run(|s| s.meta()).await
    }

    /// Check for a newer MTGJSON version and reset views if stale.
    pub async fn refresh(&self) -> Result<bool> {
        self.run(|s| s.refresh()).await
    }

    /// Return the list of currently registered DuckDB view names.
    pub async fn views(&self) -> Result<Vec<String>> {
        self.run(|s| Ok(s.views())).await
    }

    /// Close the SDK, releasing all resources.
    ///
    /// After calling this, subsequent operations will fail with a
    /// poisoned lock error.
    pub async fn close(self) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let sdk = self
                .inner
                .lock()
                .map_err(|_| MtgjsonError::InvalidArgument("SDK lock poisoned".into()))?;
            // Dropping the MutexGuard drops the SDK
            drop(sdk);
            Ok(())
        })
        .await
        .map_err(|e| MtgjsonError::InvalidArgument(format!("Task join error: {e}")))?
    }
}
