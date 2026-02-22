//! Shared test fixtures for the MTGJSON SDK integration tests.
//!
//! Provides `setup_sample_db()` which creates an in-memory DuckDB connection
//! populated with small sample tables (cards, sets, tokens, card_identifiers,
//! card_legalities) via NDJSON temp files.

use mtgjson_sdk::{CacheManager, Connection};
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Create a `Connection` backed by a temporary cache directory with sample data
/// loaded into DuckDB tables via NDJSON temp files.
///
/// Returns `(Connection, tempfile::TempDir)`. The caller must keep the `TempDir`
/// alive for the duration of the test so the cache directory is not deleted
/// prematurely.
pub fn setup_sample_db() -> (Connection, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache = CacheManager::new(Some(tmp_dir.path().to_path_buf()), true, Duration::from_secs(30)).unwrap();
    let conn = Connection::new(cache).unwrap();

    // -- cards table ----------------------------------------------------------
    register_cards(&conn);

    // -- sets table -----------------------------------------------------------
    register_sets(&conn);

    // -- tokens table ---------------------------------------------------------
    register_tokens(&conn);

    // -- card_identifiers table -----------------------------------------------
    register_card_identifiers(&conn);

    // -- card_legalities table (already in unpivoted format) -------------------
    register_card_legalities(&conn);

    (conn, tmp_dir)
}

fn register_cards(conn: &Connection) {
    let cards = vec![
        serde_json::json!({
            "uuid": "card-uuid-001",
            "name": "Lightning Bolt",
            "setCode": "A25",
            "colors": "R",
            "colorIdentity": "R",
            "manaValue": 1.0,
            "manaCost": "{R}",
            "type": "Instant",
            "text": "Lightning Bolt deals 3 damage to any target.",
            "rarity": "uncommon",
            "power": null,
            "toughness": null,
            "layout": "normal",
            "artist": "Christopher Moeller",
            "keywords": "",
            "availability": "paper, mtgo",
            "isPromo": false,
            "language": "English",
            "faceName": null,
            "side": null,
            "number": "141"
        }),
        serde_json::json!({
            "uuid": "card-uuid-002",
            "name": "Counterspell",
            "setCode": "A25",
            "colors": "U",
            "colorIdentity": "U",
            "manaValue": 2.0,
            "manaCost": "{U}{U}",
            "type": "Instant",
            "text": "Counter target spell.",
            "rarity": "uncommon",
            "power": null,
            "toughness": null,
            "layout": "normal",
            "artist": "Zack Stella",
            "keywords": "",
            "availability": "paper, mtgo",
            "isPromo": false,
            "language": "English",
            "faceName": null,
            "side": null,
            "number": "50"
        }),
        serde_json::json!({
            "uuid": "card-uuid-003",
            "name": "Fire // Ice",
            "setCode": "MH2",
            "colors": "R, U",
            "colorIdentity": "R, U",
            "manaValue": 2.0,
            "manaCost": "{1}{R}",
            "type": "Instant // Instant",
            "text": "Fire deals 2 damage divided as you choose among one or two targets.",
            "rarity": "uncommon",
            "power": null,
            "toughness": null,
            "layout": "split",
            "artist": "Franz Vohwinkel",
            "keywords": "",
            "availability": "paper",
            "isPromo": false,
            "language": "English",
            "faceName": "Fire",
            "side": "a",
            "number": "290"
        }),
    ];

    write_ndjson_and_register(conn, "cards", &cards);
}

fn register_sets(conn: &Connection) {
    let sets = vec![
        serde_json::json!({
            "code": "A25",
            "name": "Masters 25",
            "type": "masters",
            "releaseDate": "2018-03-16",
            "baseSetSize": 249,
            "totalSetSize": 249,
            "block": null
        }),
        serde_json::json!({
            "code": "MH2",
            "name": "Modern Horizons 2",
            "type": "draft_innovation",
            "releaseDate": "2021-06-18",
            "baseSetSize": 303,
            "totalSetSize": 303,
            "block": null
        }),
    ];

    write_ndjson_and_register(conn, "sets", &sets);
}

fn register_tokens(conn: &Connection) {
    let tokens = vec![
        serde_json::json!({
            "uuid": "token-uuid-001",
            "name": "Soldier",
            "setCode": "A25",
            "colors": "W",
            "type": "Token Creature \u{2014} Soldier",
            "artist": "Greg Staples",
            "number": "T1",
            "layout": "token",
            "availability": "paper"
        }),
        serde_json::json!({
            "uuid": "token-uuid-002",
            "name": "Goblin",
            "setCode": "MH2",
            "colors": "R",
            "type": "Token Creature \u{2014} Goblin",
            "artist": "Karl Kopinski",
            "number": "T2",
            "layout": "token",
            "availability": "paper"
        }),
    ];

    write_ndjson_and_register(conn, "tokens", &tokens);
}

fn register_card_identifiers(conn: &Connection) {
    let ids = vec![
        serde_json::json!({
            "uuid": "card-uuid-001",
            "scryfallId": "scryfall-001",
            "tcgplayerProductId": "12345",
            "mtgoId": "mtgo-001",
            "mtgArenaId": "arena-001",
            "multiverseId": "100001"
        }),
        serde_json::json!({
            "uuid": "card-uuid-002",
            "scryfallId": "scryfall-002",
            "tcgplayerProductId": "67890",
            "mtgoId": "mtgo-002",
            "mtgArenaId": "arena-002",
            "multiverseId": "100002"
        }),
    ];

    write_ndjson_and_register(conn, "card_identifiers", &ids);
}

fn register_card_legalities(conn: &Connection) {
    let legalities = vec![
        serde_json::json!({"uuid": "card-uuid-001", "format": "modern", "status": "Legal"}),
        serde_json::json!({"uuid": "card-uuid-001", "format": "vintage", "status": "Restricted"}),
        serde_json::json!({"uuid": "card-uuid-001", "format": "standard", "status": "Not Legal"}),
        serde_json::json!({"uuid": "card-uuid-002", "format": "modern", "status": "Legal"}),
        serde_json::json!({"uuid": "card-uuid-002", "format": "vintage", "status": "Legal"}),
        serde_json::json!({"uuid": "card-uuid-002", "format": "standard", "status": "Not Legal"}),
        serde_json::json!({"uuid": "card-uuid-003", "format": "modern", "status": "Legal"}),
        serde_json::json!({"uuid": "card-uuid-003", "format": "vintage", "status": "Legal"}),
    ];

    write_ndjson_and_register(conn, "card_legalities", &legalities);
}

/// Write a slice of JSON values as NDJSON to a temp file and register it
/// as a DuckDB table via `Connection::register_table_from_ndjson`.
fn write_ndjson_and_register(conn: &Connection, table_name: &str, rows: &[serde_json::Value]) {
    let mut file = NamedTempFile::new().unwrap();
    for row in rows {
        writeln!(file, "{}", serde_json::to_string(row).unwrap()).unwrap();
    }
    file.flush().unwrap();

    let path = file.path().to_str().unwrap();
    conn.register_table_from_ndjson(table_name, path).unwrap();
    // NamedTempFile is dropped here, but DuckDB has already read the data
    // into an in-memory table, so this is fine.
}
