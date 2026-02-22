//! DuckDB connection wrapper with view registration and query execution.
//!
//! Uses schema introspection to adapt views dynamically:
//! - CSV VARCHAR columns are auto-detected and converted to arrays
//! - Wide-format legalities are auto-UNPIVOTed to (uuid, format, status) rows

use crate::cache::CacheManager;
use crate::error::Result;
use duckdb::{types::ValueRef, Connection as DuckDbConnection};
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Known list columns that don't follow the plural naming convention
/// (e.g. colorIdentity, availability, producedMana). Always converted
/// to arrays regardless of heuristic detection.
fn static_list_columns() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut map = HashMap::new();
    map.insert(
        "cards",
        HashSet::from([
            "artistIds",
            "attractionLights",
            "availability",
            "boosterTypes",
            "cardParts",
            "colorIdentity",
            "colorIndicator",
            "colors",
            "finishes",
            "frameEffects",
            "keywords",
            "originalPrintings",
            "otherFaceIds",
            "printings",
            "producedMana",
            "promoTypes",
            "rebalancedPrintings",
            "subsets",
            "subtypes",
            "supertypes",
            "types",
            "variations",
        ]),
    );
    map.insert(
        "tokens",
        HashSet::from([
            "artistIds",
            "availability",
            "boosterTypes",
            "colorIdentity",
            "colorIndicator",
            "colors",
            "finishes",
            "frameEffects",
            "keywords",
            "otherFaceIds",
            "producedMana",
            "promoTypes",
            "reverseRelated",
            "subtypes",
            "supertypes",
            "types",
        ]),
    );
    map
}

/// VARCHAR columns that are definitely NOT lists, even if they match the
/// plural-name heuristic. Prevents splitting text fields that contain commas,
/// JSON struct fields, and other scalar strings.
fn ignored_columns() -> HashSet<&'static str> {
    HashSet::from([
        "text",
        "originalText",
        "flavorText",
        "printedText",
        "identifiers",
        "legalities",
        "leadershipSkills",
        "purchaseUrls",
        "relatedCards",
        "rulings",
        "sourceProducts",
        "foreignData",
        "translations",
        "toughness",
        "status",
        "format",
        "uris",
        "scryfallUri",
    ])
}

/// VARCHAR columns containing JSON strings that should be cast to DuckDB's
/// JSON type. This enables SQL operators like ->>, json_extract(), etc.
fn json_cast_columns() -> HashSet<&'static str> {
    HashSet::from([
        "identifiers",
        "legalities",
        "leadershipSkills",
        "purchaseUrls",
        "relatedCards",
        "rulings",
        "sourceProducts",
        "foreignData",
        "translations",
    ])
}

/// Wraps a DuckDB connection and registers parquet files as views.
///
/// Uses schema introspection to adapt views dynamically:
/// - CSV VARCHAR columns are auto-detected and converted to arrays
/// - Wide-format legalities are auto-UNPIVOTed to (uuid, format, status) rows
pub struct Connection {
    conn: DuckDbConnection,
    /// The cache manager used to download/locate data files.
    pub cache: RefCell<CacheManager>,
    registered_views: RefCell<HashSet<String>>,
}

impl Connection {
    /// Create a connection backed by the given cache.
    ///
    /// Opens an in-memory DuckDB database.
    pub fn new(cache: CacheManager) -> Result<Self> {
        let conn = DuckDbConnection::open_in_memory()?;
        Ok(Self {
            conn,
            cache: RefCell::new(cache),
            registered_views: RefCell::new(HashSet::new()),
        })
    }

    /// Ensure one or more views are registered, downloading data if needed.
    pub fn ensure_views(&self, views: &[&str]) -> Result<()> {
        for name in views {
            if !self.registered_views.borrow().contains(*name) {
                self.ensure_view(name)?;
            }
        }
        Ok(())
    }

    /// Execute SQL and return results as a `Vec` of `HashMap`s.
    ///
    /// Each row is represented as a `HashMap<String, serde_json::Value>`.
    /// Automatically converts DuckDB types to `serde_json::Value`.
    pub fn execute(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let mut stmt = self.conn.prepare(sql)?;

        let param_values: Vec<&dyn duckdb::ToSql> = params
            .iter()
            .map(|p| p as &dyn duckdb::ToSql)
            .collect();

        let mut rows_result = stmt.query(param_values.as_slice())?;

        // Get column metadata AFTER query execution (calling before panics in duckdb-rs)
        let column_names: Vec<String> = rows_result
            .as_ref()
            .unwrap()
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let column_count = rows_result.as_ref().unwrap().column_count();

        let mut out: Vec<HashMap<String, serde_json::Value>> = Vec::new();

        while let Some(row) = rows_result.next()? {
            let mut map = HashMap::new();
            for i in 0..column_count {
                let col_name = &column_names[i];
                let value = convert_value_ref(row.get_ref(i)?);
                map.insert(col_name.clone(), value);
            }
            out.push(map);
        }

        Ok(out)
    }

    /// Execute SQL and deserialize each row into type `T`.
    ///
    /// First executes the query as `HashMap` rows, then deserializes each
    /// row using `serde_json`.
    pub fn execute_into<T: DeserializeOwned>(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<T>> {
        let rows = self.execute(sql, params)?;
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let value = serde_json::Value::Object(
                row.into_iter().collect::<serde_json::Map<String, serde_json::Value>>(),
            );
            let item: T = serde_json::from_value(value)?;
            results.push(item);
        }
        Ok(results)
    }

    /// Execute SQL and return the first column of the first row.
    ///
    /// Returns `None` if the result set is empty.
    pub fn execute_scalar(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Option<serde_json::Value>> {
        let mut stmt = self.conn.prepare(sql)?;
        let param_values: Vec<&dyn duckdb::ToSql> = params
            .iter()
            .map(|p| p as &dyn duckdb::ToSql)
            .collect();

        let mut rows = stmt.query(param_values.as_slice())?;

        if let Some(row) = rows.next()? {
            let value = convert_value_ref(row.get_ref(0)?);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Create a DuckDB table from a newline-delimited JSON file.
    ///
    /// More memory-efficient than loading data into a Rust structure first,
    /// since data is streamed from disk by DuckDB.
    pub fn register_table_from_ndjson(
        &self,
        table_name: &str,
        ndjson_path: &str,
    ) -> Result<()> {
        let path_fwd = ndjson_path.replace('\\', "/");
        self.conn.execute_batch(&format!(
            "DROP TABLE IF EXISTS {}; \
             CREATE TABLE {} AS SELECT * FROM read_json_auto('{}', format='newline_delimited')",
            table_name, table_name, path_fwd
        ))?;
        self.registered_views.borrow_mut().insert(table_name.to_string());
        Ok(())
    }

    /// Check whether a view has been registered.
    pub fn has_view(&self, name: &str) -> bool {
        self.registered_views.borrow().contains(name)
    }

    /// Return a list of all registered view names.
    pub fn views(&self) -> Vec<String> {
        self.registered_views.borrow().iter().cloned().collect()
    }

    /// Clear all registered views so they will be re-created on next access.
    pub fn reset_views(&self) {
        self.registered_views.borrow_mut().clear();
    }

    /// Access the underlying DuckDB connection for advanced usage.
    pub fn raw(&self) -> &DuckDbConnection {
        &self.conn
    }

    /// Lazily register a parquet file as a DuckDB view.
    ///
    /// Introspects the parquet schema on first registration and builds
    /// the view SQL dynamically, so the SDK adapts to upstream schema
    /// changes without code updates.
    fn ensure_view(&self, view_name: &str) -> Result<()> {
        if self.registered_views.borrow().contains(view_name) {
            return Ok(());
        }

        let path = self.cache.borrow_mut().ensure_parquet(view_name)?;
        // Use forward slashes for DuckDB compatibility
        let path_str = path.to_string_lossy().replace('\\', "/");

        if view_name == "card_legalities" {
            self.register_legalities_view(&path_str)?;
            return Ok(());
        }

        // Hybrid CSV->array detection: static baseline + dynamic heuristic
        let replace_clause = self.build_csv_replace(&path_str, view_name)?;

        self.conn.execute_batch(&format!(
            "CREATE OR REPLACE VIEW {} AS SELECT *{} FROM read_parquet('{}')",
            view_name, replace_clause, path_str
        ))?;
        self.registered_views.borrow_mut().insert(view_name.to_string());
        eprintln!("Registered view: {} -> {}", view_name, path_str);

        Ok(())
    }

    /// Build a REPLACE clause using a hybrid static + dynamic approach.
    ///
    /// Four layers:
    /// 1. Static baseline: known non-plural list columns
    /// 2. Dynamic heuristic: VARCHAR columns ending in 's' are likely lists
    /// 3. Safety blocklist: prevents splitting text fields and JSON structs
    /// 4. JSON casting: struct-like VARCHAR columns cast to DuckDB JSON type
    ///
    /// Only reads the parquet footer (DESCRIBE) -- no data scanning needed.
    fn build_csv_replace(&self, path_str: &str, view_name: &str) -> Result<String> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT column_name, column_type FROM \
             (DESCRIBE SELECT * FROM read_parquet('{}'))",
            path_str
        ))?;

        let mut rows = stmt.query([])?;
        let mut schema: Vec<(String, String)> = Vec::new();
        let mut schema_map: HashMap<String, String> = HashMap::new();

        while let Some(row) = rows.next()? {
            let col_name: String = row.get(0)?;
            let col_type: String = row.get(1)?;
            schema_map.insert(col_name.clone(), col_type.clone());
            schema.push((col_name, col_type));
        }

        let static_lists = static_list_columns();
        let ignored = ignored_columns();
        let json_cast = json_cast_columns();

        // Build candidate set from both layers
        let mut candidates: HashSet<String> = HashSet::new();

        // Layer 1: Static baseline (the "knowns")
        if let Some(static_cols) = static_lists.get(view_name) {
            for col in static_cols {
                candidates.insert(col.to_string());
            }
        }

        // Layer 2: Dynamic heuristic (the "unknowns")
        for (col, dtype) in &schema {
            if dtype != "VARCHAR" {
                continue;
            }
            if ignored.contains(col.as_str()) {
                continue;
            }
            if col.ends_with('s') {
                candidates.insert(col.clone());
            }
        }

        // Filter to columns that actually exist as VARCHAR in this file
        let mut final_cols: Vec<String> = candidates
            .into_iter()
            .filter(|col| schema_map.get(col).map(|t| t == "VARCHAR").unwrap_or(false))
            .collect();
        final_cols.sort();

        let mut exprs: Vec<String> = Vec::new();

        for col in &final_cols {
            exprs.push(format!(
                "CASE WHEN \"{}\" IS NULL OR TRIM(\"{}\") = '' \
                 THEN []::VARCHAR[] \
                 ELSE string_split(\"{}\", ', ') END AS \"{}\"",
                col, col, col, col
            ));
        }

        // Layer 4: JSON casting for struct-like VARCHAR columns
        let mut json_cols: Vec<&&str> = json_cast.iter().collect();
        json_cols.sort();
        for col in json_cols {
            if schema_map.get(*col).map(|t| t == "VARCHAR").unwrap_or(false) {
                exprs.push(format!("TRY_CAST(\"{}\" AS JSON) AS \"{}\"", col, col));
            }
        }

        if exprs.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(" REPLACE ({})", exprs.join(", ")))
        }
    }

    /// Register card_legalities by dynamically UNPIVOTing wide format.
    ///
    /// Introspects the parquet schema and UNPIVOTs all columns except 'uuid'
    /// into (uuid, format, status) rows. Automatically picks up new formats
    /// (e.g. 'timeless', 'oathbreaker') as they appear in the data.
    fn register_legalities_view(&self, path_str: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT column_name FROM \
             (DESCRIBE SELECT * FROM read_parquet('{}'))",
            path_str
        ))?;

        let mut rows = stmt.query([])?;
        let mut all_cols: Vec<String> = Vec::new();

        while let Some(row) = rows.next()? {
            let col_name: String = row.get(0)?;
            all_cols.push(col_name);
        }

        // Everything except 'uuid' is a format column
        let format_cols: Vec<&String> = all_cols.iter().filter(|c| c.as_str() != "uuid").collect();

        if format_cols.is_empty() {
            // Fallback: assume row format (test data or different schema)
            self.conn.execute_batch(&format!(
                "CREATE OR REPLACE VIEW card_legalities AS \
                 SELECT * FROM read_parquet('{}')",
                path_str
            ))?;
        } else {
            let cols_sql: String = format_cols
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", ");

            self.conn.execute_batch(&format!(
                "CREATE OR REPLACE VIEW card_legalities AS \
                 SELECT uuid, format, status FROM (\
                   UNPIVOT (SELECT * FROM read_parquet('{}'))\
                   ON {}\
                   INTO NAME format VALUE status\
                 ) WHERE status IS NOT NULL",
                path_str, cols_sql
            ))?;
        }

        self.registered_views.borrow_mut().insert("card_legalities".to_string());
        eprintln!(
            "Registered legalities view (UNPIVOT {} formats): {}",
            format_cols.len(),
            path_str
        );

        Ok(())
    }
}

/// Convert a DuckDB `ValueRef` to a `serde_json::Value`.
fn convert_value_ref(val: ValueRef<'_>) -> serde_json::Value {
    match val {
        ValueRef::Null => serde_json::Value::Null,
        ValueRef::Boolean(b) => serde_json::Value::Bool(b),
        ValueRef::TinyInt(n) => serde_json::Value::Number(n.into()),
        ValueRef::SmallInt(n) => serde_json::Value::Number(n.into()),
        ValueRef::Int(n) => serde_json::Value::Number(n.into()),
        ValueRef::BigInt(n) => serde_json::Value::Number(n.into()),
        ValueRef::HugeInt(n) => {
            // HugeInt may not fit in i64; try i64, fallback to string
            if let Ok(i) = i64::try_from(n) {
                serde_json::Value::Number(i.into())
            } else {
                serde_json::Value::String(n.to_string())
            }
        }
        ValueRef::Float(f) => serde_json::Number::from_f64(f as f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        ValueRef::Double(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        ValueRef::Text(bytes) => {
            let s = String::from_utf8_lossy(bytes).to_string();
            // Try to parse as JSON if it looks like a JSON structure
            serde_json::Value::String(s)
        }
        ValueRef::Blob(bytes) => {
            // Encode blob as base64 or hex string
            serde_json::Value::String(format!(
                "blob:{}",
                bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            ))
        }
        _ => {
            // For other types (Date, Time, Timestamp, Interval, List, etc.),
            // convert to string representation
            serde_json::Value::Null
        }
    }
}
