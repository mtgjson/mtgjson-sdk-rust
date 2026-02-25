//! Price queries against the DuckDB `all_prices_today` and `all_prices` parquet views.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// PriceQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG card prices backed by the `all_prices_today` DuckDB view.
pub struct PriceQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> PriceQuery<'a> {
    /// Create a new `PriceQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Get the full nested price structure for a card UUID.
    ///
    /// Returns a nested object keyed by `source -> provider -> price_type -> finish -> date -> price`.
    pub fn get(&self, uuid: &str) -> Result<Value> {
        self.conn.ensure_views(&["all_prices_today"])?;

        let (sql, params) = SqlBuilder::new("all_prices_today")
            .where_eq("uuid", uuid)
            .order_by(&["date DESC"])
            .build();

        let rows = self.conn.execute(&sql, &params)?;

        // Build a nested map: source -> provider -> currency -> price_type -> finish -> {date: price}
        let mut result: HashMap<String, HashMap<String, HashMap<String, HashMap<String, HashMap<String, HashMap<String, f64>>>>>> =
            HashMap::new();

        for row in &rows {
            let source = row.get("source").and_then(|v| v.as_str()).unwrap_or("");
            let provider = row.get("provider").and_then(|v| v.as_str()).unwrap_or("");
            let currency = row.get("currency").and_then(|v| v.as_str()).unwrap_or("");
            let price_type = row.get("price_type").and_then(|v| v.as_str()).unwrap_or("");
            let finish = row.get("finish").and_then(|v| v.as_str()).unwrap_or("");
            let date = row.get("date").and_then(|v| v.as_str()).unwrap_or("");
            let price = row
                .get("price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            result
                .entry(source.to_string())
                .or_default()
                .entry(provider.to_string())
                .or_default()
                .entry(currency.to_string())
                .or_default()
                .entry(price_type.to_string())
                .or_default()
                .entry(finish.to_string())
                .or_default()
                .insert(date.to_string(), price);
        }

        Ok(serde_json::to_value(result).unwrap_or(Value::Null))
    }

    /// Get the most recent price for each provider/price_type/finish group for a card UUID.
    pub fn today(&self, uuid: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["all_prices_today"])?;

        let sql = r#"
            SELECT *
            FROM all_prices_today
            WHERE uuid = ?
              AND date = (SELECT MAX(date) FROM all_prices_today WHERE uuid = ?)
        "#;

        let rows = self.conn.execute(sql, &[uuid.to_string(), uuid.to_string()])?;
        Ok(rows_to_values(rows))
    }

    /// Get price history for a card UUID, optionally filtered by date range.
    pub fn history(
        &self,
        uuid: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["all_prices"])?;

        let mut qb = SqlBuilder::new("all_prices");
        qb.where_eq("uuid", uuid);
        qb.order_by(&["date ASC"]);

        if let Some(df) = date_from {
            qb.where_gte("date", df);
        }

        if let Some(dt) = date_to {
            qb.where_lte("date", dt);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get aggregated price trend statistics for a card UUID.
    ///
    /// Returns `min_price`, `max_price`, `avg_price`, `first_date`, `last_date`, `data_points`.
    pub fn price_trend(&self, uuid: &str) -> Result<Value> {
        self.conn.ensure_views(&["all_prices"])?;

        let sql = r#"
            SELECT
                MIN(price) AS min_price,
                MAX(price) AS max_price,
                AVG(price) AS avg_price,
                MIN(date) AS first_date,
                MAX(date) AS last_date,
                COUNT(*) AS data_points
            FROM all_prices
            WHERE uuid = ?
        "#;

        let rows = self.conn.execute(sql, &[uuid.to_string()])?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
            .unwrap_or(Value::Null))
    }

    /// Find the cheapest printing of a card by name.
    ///
    /// Joins `cards` to `all_prices_today` and returns the printing with the lowest price.
    pub fn cheapest_printing(&self, name: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let sql = r#"
            SELECT c.*, p.price, p.source, p.provider, p.finish, p.date
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE c.name = ?
            ORDER BY p.price ASC
            LIMIT 1
        "#;

        let rows = self.conn.execute(sql, &[name.to_string()])?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    /// Find the N cheapest printings of a card by name, ordered by ascending price.
    pub fn cheapest_printings(&self, name: &str, limit: usize) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let sql = format!(
            r#"
            SELECT c.*, p.price, p.source, p.provider, p.finish, p.date
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE c.name = ?
            ORDER BY p.price ASC
            LIMIT {}
            "#,
            limit
        );

        let rows = self.conn.execute(&sql, &[name.to_string()])?;
        Ok(rows_to_values(rows))
    }

    /// Find the N most expensive printings of a card by name, ordered by descending price.
    pub fn most_expensive_printings(&self, name: &str, limit: usize) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let sql = format!(
            r#"
            SELECT c.*, p.price, p.source, p.provider, p.finish, p.date
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE c.name = ?
            ORDER BY p.price DESC
            LIMIT {}
            "#,
            limit
        );

        let rows = self.conn.execute(&sql, &[name.to_string()])?;
        Ok(rows_to_values(rows))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rows_to_values(rows: Vec<HashMap<String, Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect()
}
