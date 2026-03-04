//! Price queries against the DuckDB `all_prices_today` and `all_prices` parquet views.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// PriceFilter
// ---------------------------------------------------------------------------

/// Optional filter parameters shared across price query methods.
#[derive(Debug, Clone, Default)]
pub struct PriceFilter {
    pub provider: Option<String>,
    pub finish: Option<String>,
    pub price_type: Option<String>,
}

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
    ///
    /// Optionally filtered by `provider`, `finish`, and `price_type` via [`PriceFilter`].
    pub fn today(&self, uuid: &str, filter: &PriceFilter) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["all_prices_today"])?;

        let mut parts = vec![
            "SELECT * FROM all_prices_today".to_string(),
            "WHERE uuid = ?".to_string(),
            "AND date = (SELECT MAX(date) FROM all_prices_today WHERE uuid = ?)".to_string(),
        ];
        let mut params = vec![uuid.to_string(), uuid.to_string()];

        append_filter(&mut parts, &mut params, filter);

        let sql = parts.join(" ");
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get price history for a card UUID, optionally filtered by date range and price filters.
    pub fn history(
        &self,
        uuid: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
        filter: &PriceFilter,
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

        if let Some(ref provider) = filter.provider {
            qb.where_eq("provider", provider);
        }
        if let Some(ref finish) = filter.finish {
            qb.where_eq("finish", finish);
        }
        if let Some(ref pt) = filter.price_type {
            qb.where_eq("price_type", pt);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get aggregated price trend statistics for a card UUID.
    ///
    /// Returns `min_price`, `max_price`, `avg_price`, `first_date`, `last_date`, `data_points`.
    pub fn price_trend(&self, uuid: &str, filter: &PriceFilter) -> Result<Value> {
        self.conn.ensure_views(&["all_prices"])?;

        let price_type = filter
            .price_type
            .as_deref()
            .unwrap_or("retail");

        let mut parts = vec![
            "SELECT".to_string(),
            "  MIN(price) AS min_price,".to_string(),
            "  MAX(price) AS max_price,".to_string(),
            "  AVG(price) AS avg_price,".to_string(),
            "  MIN(date) AS first_date,".to_string(),
            "  MAX(date) AS last_date,".to_string(),
            "  COUNT(*) AS data_points".to_string(),
            "FROM all_prices_today".to_string(),
            "WHERE uuid = ? AND price_type = ?".to_string(),
        ];
        let mut params = vec![uuid.to_string(), price_type.to_string()];

        if let Some(ref provider) = filter.provider {
            parts.push("AND provider = ?".to_string());
            params.push(provider.clone());
        }
        if let Some(ref finish) = filter.finish {
            parts.push("AND finish = ?".to_string());
            params.push(finish.clone());
        }

        let sql = parts.join(" ");
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
            .unwrap_or(Value::Null))
    }

    /// Find the cheapest printing of a card by name.
    ///
    /// Joins `cards` to `all_prices_today` and returns the printing with the lowest price.
    pub fn cheapest_printing(&self, name: &str, filter: &PriceFilter) -> Result<Option<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let provider = filter.provider.as_deref().unwrap_or("tcgplayer");
        let finish = filter.finish.as_deref().unwrap_or("normal");
        let price_type = filter.price_type.as_deref().unwrap_or("retail");

        let sql = r#"
            SELECT c.uuid, c.setCode, c.number, p.price, p.date
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE c.name = ? AND p.provider = ?
              AND p.finish = ? AND p.price_type = ?
              AND p.date = (SELECT MAX(p2.date) FROM all_prices_today p2
                WHERE p2.uuid = c.uuid AND p2.provider = ?
                AND p2.finish = ? AND p2.price_type = ?)
            ORDER BY p.price ASC
            LIMIT 1
        "#;

        let rows = self.conn.execute(
            sql,
            &[
                name.to_string(),
                provider.to_string(),
                finish.to_string(),
                price_type.to_string(),
                provider.to_string(),
                finish.to_string(),
                price_type.to_string(),
            ],
        )?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    /// Find the cheapest available printing of each card (global leaderboard).
    ///
    /// Groups by card name and uses `arg_min()` for efficient single-pass aggregation.
    /// Returns `name`, `cheapest_set`, `cheapest_number`, `cheapest_uuid`, `min_price`.
    pub fn cheapest_printings(
        &self,
        filter: &PriceFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let provider = filter.provider.as_deref().unwrap_or("tcgplayer");
        let finish = filter.finish.as_deref().unwrap_or("normal");
        let price_type = filter.price_type.as_deref().unwrap_or("retail");
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let sql = format!(
            r#"
            SELECT c.name,
              arg_min(c.setCode, p.price) AS cheapest_set,
              arg_min(c.number, p.price) AS cheapest_number,
              arg_min(c.uuid, p.price) AS cheapest_uuid,
              MIN(p.price) AS min_price
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE p.provider = ? AND p.finish = ? AND p.price_type = ?
              AND p.date = (SELECT MAX(date) FROM all_prices_today)
            GROUP BY c.name
            ORDER BY min_price ASC
            LIMIT {} OFFSET {}
            "#,
            limit, offset
        );

        let rows = self.conn.execute(
            &sql,
            &[
                provider.to_string(),
                finish.to_string(),
                price_type.to_string(),
            ],
        )?;
        Ok(rows_to_values(rows))
    }

    /// Find the most expensive printing of each card (global leaderboard).
    ///
    /// Groups by card name and uses `arg_max()` for efficient single-pass aggregation.
    /// Returns `name`, `priciest_set`, `priciest_number`, `priciest_uuid`, `max_price`.
    pub fn most_expensive_printings(
        &self,
        filter: &PriceFilter,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "all_prices_today"])?;

        let provider = filter.provider.as_deref().unwrap_or("tcgplayer");
        let finish = filter.finish.as_deref().unwrap_or("normal");
        let price_type = filter.price_type.as_deref().unwrap_or("retail");
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let sql = format!(
            r#"
            SELECT c.name,
              arg_max(c.setCode, p.price) AS priciest_set,
              arg_max(c.number, p.price) AS priciest_number,
              arg_max(c.uuid, p.price) AS priciest_uuid,
              MAX(p.price) AS max_price
            FROM cards c
            JOIN all_prices_today p ON c.uuid = p.uuid
            WHERE p.provider = ? AND p.finish = ? AND p.price_type = ?
              AND p.date = (SELECT MAX(date) FROM all_prices_today)
            GROUP BY c.name
            ORDER BY max_price DESC
            LIMIT {} OFFSET {}
            "#,
            limit, offset
        );

        let rows = self.conn.execute(
            &sql,
            &[
                provider.to_string(),
                finish.to_string(),
                price_type.to_string(),
            ],
        )?;
        Ok(rows_to_values(rows))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn append_filter(parts: &mut Vec<String>, params: &mut Vec<String>, filter: &PriceFilter) {
    if let Some(ref provider) = filter.provider {
        parts.push("AND provider = ?".to_string());
        params.push(provider.clone());
    }
    if let Some(ref finish) = filter.finish {
        parts.push("AND finish = ?".to_string());
        params.push(finish.clone());
    }
    if let Some(ref pt) = filter.price_type {
        parts.push("AND price_type = ?".to_string());
        params.push(pt.clone());
    }
}

fn rows_to_values(rows: Vec<HashMap<String, Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect()
}
