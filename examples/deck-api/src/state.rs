use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;

/// Shared application state available to all route handlers via Axum's
/// `State` extractor.
pub struct AppState {
    /// The async MTGJSON SDK instance. Handles dispatching blocking SDK
    /// operations to a thread pool internally.
    pub sdk: mtgjson_sdk::AsyncMtgjsonSdk,

    /// Async HTTP client for fetching individual deck JSON files from the
    /// MTGJSON CDN. Separate from the SDK's own blocking `reqwest` client.
    pub http: reqwest::Client,

    /// In-memory cache of fetched deck contents, keyed by `fileName`.
    /// Avoids re-downloading the same deck file on repeated requests.
    pub deck_cache: Mutex<HashMap<String, Value>>,
}
