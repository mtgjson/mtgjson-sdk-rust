use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListSetsParams {
    pub set_type: Option<String>,
}

/// GET /api/sets?set_type=expansion
///
/// List all sets, optionally filtered by set type.
pub async fn list_sets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListSetsParams>,
) -> Result<Json<Value>, AppError> {
    let sets = state
        .sdk
        .run(move |s| s.sets().list(params.set_type.as_deref(), None, None, None))
        .await?;

    let count = sets.len();
    Ok(Json(json!({ "data": sets, "count": count })))
}

/// GET /api/sets/:code
///
/// Get details for a single set by its code.
pub async fn get_set(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<Value>, AppError> {
    let set = state.sdk.run(move |s| s.sets().get(&code)).await?;

    match set {
        Some(s) => Ok(Json(json!({ "data": s }))),
        None => Err(AppError::not_found("Set not found")),
    }
}
