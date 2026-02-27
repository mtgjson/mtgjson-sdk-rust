use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

/// GET /api/meta
///
/// Returns the MTGJSON dataset version and date.
pub async fn get_meta(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let meta = state.sdk.meta().await?;

    // Meta.json has { "data": { "version": "...", "date": "..." } }
    let data = meta.get("data").unwrap_or(&meta);
    let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
    let date = data.get("date").and_then(|v| v.as_str()).unwrap_or("unknown");

    Ok(Json(json!({
        "version": version,
        "date": date
    })))
}
