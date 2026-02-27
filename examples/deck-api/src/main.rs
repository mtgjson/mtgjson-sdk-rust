mod error;
mod routes;
mod state;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;

use state::AppState;

#[tokio::main]
async fn main() {
    eprintln!("Initializing MTGJSON SDK...");
    let sdk = mtgjson_sdk::AsyncMtgjsonSdk::builder()
        .build()
        .await
        .expect("Failed to initialize MTGJSON SDK");
    eprintln!("SDK ready.");

    let state = Arc::new(AppState {
        sdk,
        http: reqwest::Client::new(),
        deck_cache: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/api/meta", get(routes::meta::get_meta))
        .route("/api/sets", get(routes::sets::list_sets))
        .route("/api/sets/{code}", get(routes::sets::get_set))
        .route("/api/decks", get(routes::decks::list_decks))
        .route("/api/decks/search", get(routes::decks::search_decks))
        .route("/api/decks/{file_name}", get(routes::decks::get_deck))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    eprintln!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
