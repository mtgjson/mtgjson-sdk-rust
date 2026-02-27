use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

const CDN_BASE: &str = "https://mtgjson.com/api/v5";

#[derive(Deserialize)]
pub struct ListDecksParams {
    pub set_code: Option<String>,
    pub deck_type: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchDecksParams {
    pub name: Option<String>,
    pub set_code: Option<String>,
}

/// GET /api/decks?set_code=MH3&deck_type=Commander+Deck
///
/// List deck metadata, optionally filtered by set code and/or deck type.
pub async fn list_decks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDecksParams>,
) -> Result<Json<Value>, AppError> {
    let decks = state
        .sdk
        .run(move |s| {
            s.decks()
                .list(params.set_code.as_deref(), params.deck_type.as_deref())
        })
        .await?;

    let count = decks.len();
    Ok(Json(json!({ "data": decks, "count": count })))
}

/// GET /api/decks/search?name=energy&set_code=MH3
///
/// Search decks by name substring, optionally scoped to a set.
pub async fn search_decks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchDecksParams>,
) -> Result<Json<Value>, AppError> {
    let name = params
        .name
        .ok_or_else(|| AppError::bad_request("Missing required query parameter: name"))?;

    let set_code = params.set_code;
    let decks = state
        .sdk
        .run(move |s| s.decks().search(&name, set_code.as_deref()))
        .await?;

    let count = decks.len();
    Ok(Json(json!({ "data": decks, "count": count })))
}

/// GET /api/decks/:file_name
///
/// Get full deck contents (mainBoard, sideBoard, commander, etc.) by
/// the deck's `fileName` from DeckList. Fetches the individual deck JSON
/// from the MTGJSON CDN on first access, then caches in memory.
pub async fn get_deck(
    State(state): State<Arc<AppState>>,
    Path(file_name): Path<String>,
) -> Result<Json<Value>, AppError> {
    // 1. Check the in-memory cache.
    {
        let cache = state
            .deck_cache
            .lock()
            .map_err(|_| AppError::internal("Cache lock poisoned"))?;
        if let Some(cached) = cache.get(&file_name) {
            return Ok(Json(json!({ "data": cached })));
        }
    }

    // 2. Validate that this fileName exists in the deck list.
    let file_name_check = file_name.clone();
    let exists = state
        .sdk
        .run(move |s| {
            let all_decks = s.decks().list(None, None)?;
            Ok(all_decks.iter().any(|d| {
                d.get("fileName")
                    .and_then(|v| v.as_str())
                    .map(|f| f == file_name_check)
                    .unwrap_or(false)
            }))
        })
        .await?;

    if !exists {
        return Err(AppError::not_found(format!(
            "No deck with fileName '{file_name}' found in DeckList"
        )));
    }

    // 3. Fetch from CDN.
    let url = format!("{CDN_BASE}/decks/{file_name}.json");
    let resp = state.http.get(&url).send().await.map_err(|e| {
        AppError::bad_gateway(format!("Failed to fetch deck from CDN: {e}"))
    })?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::not_found(format!(
            "Deck file '{file_name}.json' not found on CDN"
        )));
    }
    if !resp.status().is_success() {
        return Err(AppError::bad_gateway(format!(
            "CDN returned status {}",
            resp.status()
        )));
    }

    let deck_json: Value = resp.json().await.map_err(|e| {
        AppError::bad_gateway(format!("Invalid JSON from CDN: {e}"))
    })?;

    // 4. Unwrap the {"data": {...}} envelope if present.
    let deck_data = match deck_json.get("data") {
        Some(inner) => inner.clone(),
        None => deck_json,
    };

    // 5. Cache and return.
    {
        let mut cache = state
            .deck_cache
            .lock()
            .map_err(|_| AppError::internal("Cache lock poisoned"))?;
        cache.insert(file_name, deck_data.clone());
    }

    Ok(Json(json!({ "data": deck_data })))
}
