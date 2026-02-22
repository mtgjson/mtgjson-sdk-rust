//! Connection integration tests: raw SQL execution, view registration, etc.

mod common;

use mtgjson_sdk::{CacheManager, Connection};
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

// ---------------------------------------------------------------------------
// execute
// ---------------------------------------------------------------------------

#[test]
fn execute_returns_correct_rows() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute("SELECT * FROM cards ORDER BY uuid", &[])
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["uuid"], "card-uuid-001");
    assert_eq!(rows[1]["uuid"], "card-uuid-002");
    assert_eq!(rows[2]["uuid"], "card-uuid-003");
}

#[test]
fn execute_with_params() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute(
            "SELECT * FROM cards WHERE setCode = ?",
            &["A25".to_string()],
        )
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn execute_returns_empty_for_no_matches() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute(
            "SELECT * FROM cards WHERE uuid = ?",
            &["nonexistent".to_string()],
        )
        .unwrap();
    assert!(rows.is_empty());
}

// ---------------------------------------------------------------------------
// execute_scalar
// ---------------------------------------------------------------------------

#[test]
fn execute_scalar_returns_single_value() {
    let (conn, _tmp) = common::setup_sample_db();

    let result = conn
        .execute_scalar("SELECT COUNT(*) FROM cards", &[])
        .unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_i64().unwrap(), 3);
}

#[test]
fn execute_scalar_returns_none_for_empty_result() {
    let (conn, _tmp) = common::setup_sample_db();

    let result = conn
        .execute_scalar(
            "SELECT uuid FROM cards WHERE uuid = ?",
            &["nonexistent".to_string()],
        )
        .unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// register_table_from_ndjson
// ---------------------------------------------------------------------------

#[test]
fn register_table_from_ndjson_creates_queryable_table() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache =
        CacheManager::new(Some(tmp_dir.path().to_path_buf()), true, Duration::from_secs(30))
            .unwrap();
    let conn = Connection::new(cache).unwrap();

    // Write a small NDJSON file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"{{"id": 1, "name": "Alpha"}}"#).unwrap();
    writeln!(file, r#"{{"id": 2, "name": "Beta"}}"#).unwrap();
    file.flush().unwrap();

    conn.register_table_from_ndjson("test_table", file.path().to_str().unwrap())
        .unwrap();

    // Verify the data is queryable
    let rows = conn.execute("SELECT * FROM test_table ORDER BY id", &[]).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["name"], "Alpha");
    assert_eq!(rows[1]["name"], "Beta");
}

#[test]
fn register_table_from_ndjson_marks_view_as_registered() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache =
        CacheManager::new(Some(tmp_dir.path().to_path_buf()), true, Duration::from_secs(30))
            .unwrap();
    let conn = Connection::new(cache).unwrap();

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"{{"x": 1}}"#).unwrap();
    file.flush().unwrap();

    assert!(!conn.has_view("my_table"));

    conn.register_table_from_ndjson("my_table", file.path().to_str().unwrap())
        .unwrap();

    assert!(conn.has_view("my_table"));
}

#[test]
fn register_table_replaces_existing_table() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache =
        CacheManager::new(Some(tmp_dir.path().to_path_buf()), true, Duration::from_secs(30))
            .unwrap();
    let conn = Connection::new(cache).unwrap();

    // First registration
    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, r#"{{"val": "old"}}"#).unwrap();
    file1.flush().unwrap();
    conn.register_table_from_ndjson("replaceable", file1.path().to_str().unwrap())
        .unwrap();

    // Second registration (replaces)
    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, r#"{{"val": "new"}}"#).unwrap();
    file2.flush().unwrap();
    conn.register_table_from_ndjson("replaceable", file2.path().to_str().unwrap())
        .unwrap();

    let rows = conn.execute("SELECT * FROM replaceable", &[]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["val"], "new");
}

// ---------------------------------------------------------------------------
// has_view / views
// ---------------------------------------------------------------------------

#[test]
fn has_view_returns_false_initially() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache =
        CacheManager::new(Some(tmp_dir.path().to_path_buf()), true, Duration::from_secs(30))
            .unwrap();
    let conn = Connection::new(cache).unwrap();

    assert!(!conn.has_view("cards"));
    assert!(!conn.has_view("sets"));
}

#[test]
fn views_returns_all_registered_view_names() {
    let (conn, _tmp) = common::setup_sample_db();

    let views = conn.views();
    assert!(views.contains(&"cards".to_string()));
    assert!(views.contains(&"sets".to_string()));
    assert!(views.contains(&"tokens".to_string()));
    assert!(views.contains(&"card_identifiers".to_string()));
    assert!(views.contains(&"card_legalities".to_string()));
    assert_eq!(views.len(), 5);
}

// ---------------------------------------------------------------------------
// reset_views
// ---------------------------------------------------------------------------

#[test]
fn reset_views_clears_registered_views() {
    let (conn, _tmp) = common::setup_sample_db();

    assert!(!conn.views().is_empty());

    conn.reset_views();

    assert!(conn.views().is_empty());
    assert!(!conn.has_view("cards"));
    assert!(!conn.has_view("sets"));
}

// ---------------------------------------------------------------------------
// raw
// ---------------------------------------------------------------------------

#[test]
fn raw_provides_access_to_underlying_duckdb_connection() {
    let (conn, _tmp) = common::setup_sample_db();

    // Use raw() to execute SQL directly
    let raw = conn.raw();
    raw.execute_batch("CREATE TABLE raw_test (id INTEGER, value TEXT)")
        .unwrap();
    raw.execute_batch("INSERT INTO raw_test VALUES (1, 'hello')")
        .unwrap();

    // Verify via the Connection's execute method
    let rows = conn.execute("SELECT * FROM raw_test", &[]).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["value"], "hello");
}

// ---------------------------------------------------------------------------
// execute_into
// ---------------------------------------------------------------------------

#[test]
fn execute_into_deserializes_rows() {
    let (conn, _tmp) = common::setup_sample_db();

    #[derive(serde::Deserialize, Debug)]
    struct SimpleCard {
        uuid: String,
        name: String,
    }

    let cards: Vec<SimpleCard> = conn
        .execute_into("SELECT uuid, name FROM cards ORDER BY uuid", &[])
        .unwrap();
    assert_eq!(cards.len(), 3);
    assert_eq!(cards[0].uuid, "card-uuid-001");
    assert_eq!(cards[0].name, "Lightning Bolt");
}

// ---------------------------------------------------------------------------
// Type conversions
// ---------------------------------------------------------------------------

#[test]
fn null_values_are_converted_to_json_null() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute(
            "SELECT power FROM cards WHERE uuid = ?",
            &["card-uuid-001".to_string()],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0]["power"].is_null());
}

#[test]
fn boolean_values_are_converted_correctly() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute(
            "SELECT isPromo FROM cards WHERE uuid = ?",
            &["card-uuid-001".to_string()],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["isPromo"], false);
}

#[test]
fn numeric_values_are_converted_correctly() {
    let (conn, _tmp) = common::setup_sample_db();

    let rows = conn
        .execute(
            "SELECT manaValue FROM cards WHERE uuid = ?",
            &["card-uuid-001".to_string()],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    // manaValue is 1.0
    let mv = rows[0]["manaValue"].as_f64().unwrap();
    assert!((mv - 1.0).abs() < f64::EPSILON);
}
