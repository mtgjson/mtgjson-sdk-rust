//! Booster pack simulator.
//!
//! Simulates opening MTG booster packs by querying the booster configuration
//! tables and performing weighted random card selection, mirroring the
//! distribution rules defined by MTGJSON.

use crate::connection::Connection;
use crate::error::{MtgjsonError, Result};
use rand::prelude::*;
use std::collections::HashMap;

/// Simulates opening MTG booster packs using the MTGJSON booster configuration data.
///
/// The simulator reads from the set booster parquet tables (content weights,
/// contents, sheet cards, and sheet metadata) to faithfully reproduce the
/// distribution of cards in sealed product.
pub struct BoosterSimulator<'a> {
    conn: &'a Connection,
}

impl<'a> BoosterSimulator<'a> {
    /// Create a new `BoosterSimulator` bound to the given connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Return the available booster type names for a set (e.g. `["draft", "collector"]`).
    ///
    /// Returns an empty vector if the set has no booster configuration data.
    pub fn available_types(&self, set_code: &str) -> Result<Vec<String>> {
        self.conn.ensure_views(&["set_booster_content_weights"])?;

        let upper = set_code.to_uppercase();
        let sql = r#"
            SELECT DISTINCT "boosterName"
            FROM set_booster_content_weights
            WHERE "setCode" = ?
            ORDER BY "boosterName"
        "#;

        let rows = self.conn.execute(sql, &[upper.clone()])?;

        let types: Vec<String> = rows
            .into_iter()
            .filter_map(|r| {
                r.get("boosterName")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(types)
    }

    /// Open a single booster pack and return the card data for every card in the pack.
    ///
    /// Performs weighted random selection of a pack template and then weighted
    /// random selection of cards from each sheet in the template. Returns a
    /// vector of card JSON objects fetched from the `cards` view.
    pub fn open_pack(
        &self,
        set_code: &str,
        booster_type: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let upper = set_code.to_uppercase();

        // 1. Get all pack templates with their weights
        let pack_templates = self.get_pack_templates(&upper, booster_type)?;
        if pack_templates.is_empty() {
            return Err(MtgjsonError::NotFound(format!(
                "No booster configuration found for set '{}' type '{}'",
                set_code, booster_type
            )));
        }

        // 2. Pick a random pack template (weighted by pack weight)
        let template = pick_pack(&pack_templates);

        // 3. For each sheet in the template, pick cards
        let mut all_uuids: Vec<String> = Vec::new();

        if let Some(sheets) = template.get("sheets") {
            if let Some(sheets_map) = sheets.as_object() {
                for (sheet_name, pick_count_val) in sheets_map {
                    let pick_count = pick_count_val.as_u64().unwrap_or(0) as usize;
                    if pick_count == 0 {
                        continue;
                    }

                    // Get sheet data (card weights and sheet properties)
                    let sheet = self.get_sheet_data(&upper, booster_type, sheet_name)?;
                    if let Some(ref sheet_data) = sheet {
                        let uuids = pick_from_sheet(sheet_data, pick_count);
                        all_uuids.extend(uuids);
                    }
                }
            }
        }

        // 4. Fetch card data by UUIDs
        if all_uuids.is_empty() {
            return Ok(Vec::new());
        }

        self.fetch_cards_by_uuids(&all_uuids)
    }

    /// Open a box containing `packs` booster packs.
    ///
    /// Returns a vector of packs, where each pack is a vector of card JSON objects.
    pub fn open_box(
        &self,
        set_code: &str,
        booster_type: &str,
        packs: usize,
    ) -> Result<Vec<Vec<serde_json::Value>>> {
        let mut box_contents = Vec::with_capacity(packs);
        for _ in 0..packs {
            let pack = self.open_pack(set_code, booster_type)?;
            box_contents.push(pack);
        }
        Ok(box_contents)
    }

    /// Get the contents of a specific sheet as a `{uuid: weight}` map.
    ///
    /// Returns `None` if the sheet does not exist for the given set/booster type.
    pub fn sheet_contents(
        &self,
        set_code: &str,
        booster_type: &str,
        sheet_name: &str,
    ) -> Result<Option<HashMap<String, i64>>> {
        self.conn.ensure_views(&["set_booster_sheet_cards"])?;

        let upper = set_code.to_uppercase();
        let sql = r#"
            SELECT "cardUuid", "cardWeight"
            FROM set_booster_sheet_cards
            WHERE "setCode" = ?
              AND "boosterName" = ?
              AND "sheetName" = ?
        "#;

        let rows = self.conn.execute(sql, &[upper.clone(), booster_type.to_string(), sheet_name.to_string()])?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut contents: HashMap<String, i64> = HashMap::new();
        for row in rows {
            let uuid = row
                .get("cardUuid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let weight = row
                .get("cardWeight")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);

            if !uuid.is_empty() {
                contents.insert(uuid, weight);
            }
        }

        Ok(Some(contents))
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Get all pack templates for a set/booster type, each with its weight and sheet layout.
    fn get_pack_templates(
        &self,
        set_code: &str,
        booster_type: &str,
    ) -> Result<Vec<serde_json::Value>> {
        self.conn.ensure_views(&[
            "set_booster_content_weights",
            "set_booster_contents",
        ])?;

        // Get pack indices and weights
        let weight_sql = r#"
            SELECT "boosterIndex", "boosterWeight"
            FROM set_booster_content_weights
            WHERE "setCode" = ?
              AND "boosterName" = ?
            ORDER BY "boosterIndex"
        "#;

        let weight_rows =
            self.conn.execute(weight_sql, &[set_code.to_string(), booster_type.to_string()])?;

        if weight_rows.is_empty() {
            return Ok(Vec::new());
        }

        // Get sheet picks for each pack template
        let contents_sql = r#"
            SELECT "boosterIndex", "sheetName", "sheetPicks"
            FROM set_booster_contents
            WHERE "setCode" = ?
              AND "boosterName" = ?
            ORDER BY "boosterIndex", "sheetName"
        "#;

        let contents_rows =
            self.conn.execute(contents_sql, &[set_code.to_string(), booster_type.to_string()])?;

        // Group contents by booster index
        let mut contents_map: HashMap<i64, serde_json::Map<String, serde_json::Value>> =
            HashMap::new();
        for row in &contents_rows {
            let idx = row
                .get("boosterIndex")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let sheet_name = row
                .get("sheetName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let picks = row
                .get("sheetPicks")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);

            contents_map
                .entry(idx)
                .or_default()
                .insert(sheet_name, serde_json::Value::Number(picks.into()));
        }

        // Build template objects
        let mut templates: Vec<serde_json::Value> = Vec::new();
        for row in &weight_rows {
            let idx = row
                .get("boosterIndex")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let weight = row
                .get("boosterWeight")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);

            let sheets = contents_map
                .get(&idx)
                .cloned()
                .unwrap_or_default();

            templates.push(serde_json::json!({
                "weight": weight,
                "sheets": sheets,
            }));
        }

        Ok(templates)
    }

    /// Get the full sheet data (card UUIDs with weights and sheet properties)
    /// for a specific sheet.
    fn get_sheet_data(
        &self,
        set_code: &str,
        booster_type: &str,
        sheet_name: &str,
    ) -> Result<Option<serde_json::Value>> {
        self.conn.ensure_views(&[
            "set_booster_sheet_cards",
            "set_booster_sheets",
        ])?;

        // Get sheet properties
        let props_sql = r#"
            SELECT "sheetHasBalanceColors", "sheetIsFoil", "sheetIsFixed",
                   "sheetAllowDuplicates", "totalWeight"
            FROM set_booster_sheets
            WHERE "setCode" = ?
              AND "boosterName" = ?
              AND "sheetName" = ?
            LIMIT 1
        "#;

        let props_rows =
            self.conn.execute(props_sql, &[set_code.to_string(), booster_type.to_string(), sheet_name.to_string()])?;

        let allow_duplicates = props_rows
            .first()
            .and_then(|r| r.get("sheetAllowDuplicates"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let total_weight = props_rows
            .first()
            .and_then(|r| r.get("totalWeight"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Get card UUIDs and weights
        let cards_sql = r#"
            SELECT "cardUuid", "cardWeight"
            FROM set_booster_sheet_cards
            WHERE "setCode" = ?
              AND "boosterName" = ?
              AND "sheetName" = ?
        "#;

        let card_rows =
            self.conn.execute(cards_sql, &[set_code.to_string(), booster_type.to_string(), sheet_name.to_string()])?;

        if card_rows.is_empty() {
            return Ok(None);
        }

        let mut cards = serde_json::Map::new();
        for row in &card_rows {
            let uuid = row
                .get("cardUuid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let weight = row
                .get("cardWeight")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);

            if !uuid.is_empty() {
                cards.insert(uuid, serde_json::Value::Number(weight.into()));
            }
        }

        Ok(Some(serde_json::json!({
            "allowDuplicates": allow_duplicates,
            "totalWeight": total_weight,
            "cards": cards,
        })))
    }

    /// Fetch full card data for a list of UUIDs from the `cards` view.
    fn fetch_cards_by_uuids(&self, uuids: &[String]) -> Result<Vec<serde_json::Value>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }

        self.conn.ensure_views(&["cards"])?;

        // Build IN clause with positional params
        let placeholders: Vec<&str> = uuids.iter().map(|_| "?").collect();

        let sql = format!(
            "SELECT * FROM cards WHERE uuid IN ({})",
            placeholders.join(", ")
        );

        let rows = self.conn.execute(&sql, uuids)?;

        // Build a lookup map for ordering
        let mut card_map: HashMap<String, serde_json::Value> = HashMap::new();
        for row in rows {
            if let Some(uuid) = row.get("uuid").and_then(|v| v.as_str()) {
                let val = serde_json::to_value(&row).unwrap_or(serde_json::Value::Null);
                card_map.insert(uuid.to_string(), val);
            }
        }

        // Return cards in the same order as the UUIDs (preserving duplicates)
        let mut result = Vec::with_capacity(uuids.len());
        for uuid in uuids {
            if let Some(card) = card_map.get(uuid) {
                result.push(card.clone());
            }
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Free-standing helpers
// ---------------------------------------------------------------------------

/// Weighted random pick of a pack template.
///
/// Each template is expected to have a `"weight"` integer field. Returns a
/// reference to the chosen template.
fn pick_pack(boosters: &[serde_json::Value]) -> &serde_json::Value {
    let mut rng = thread_rng();

    let total_weight: i64 = boosters
        .iter()
        .map(|b| b.get("weight").and_then(|w| w.as_i64()).unwrap_or(1))
        .sum();

    if total_weight <= 0 {
        return &boosters[rng.gen_range(0..boosters.len())];
    }

    let mut roll = rng.gen_range(0..total_weight);

    for booster in boosters {
        let w = booster
            .get("weight")
            .and_then(|w| w.as_i64())
            .unwrap_or(1);
        roll -= w;
        if roll < 0 {
            return booster;
        }
    }

    // Fallback (should not happen with valid weights)
    boosters.last().unwrap()
}

/// Pick `count` card UUIDs from a sheet using weighted random selection.
///
/// If the sheet's `allowDuplicates` field is `true`, cards are sampled with
/// replacement. Otherwise, cards are sampled without replacement (each card
/// can appear at most once).
fn pick_from_sheet(sheet: &serde_json::Value, count: usize) -> Vec<String> {
    let mut rng = thread_rng();

    let allow_duplicates = sheet
        .get("allowDuplicates")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let cards = match sheet.get("cards").and_then(|c| c.as_object()) {
        Some(c) => c,
        None => return Vec::new(),
    };

    if cards.is_empty() {
        return Vec::new();
    }

    // Build parallel vectors of UUIDs and weights
    let mut uuids: Vec<String> = Vec::with_capacity(cards.len());
    let mut weights: Vec<i64> = Vec::with_capacity(cards.len());

    for (uuid, weight_val) in cards {
        uuids.push(uuid.clone());
        weights.push(weight_val.as_i64().unwrap_or(1));
    }

    if allow_duplicates {
        // Sample with replacement
        weighted_choices_with_replacement(&uuids, &weights, count, &mut rng)
    } else {
        // Sample without replacement
        weighted_choices_without_replacement(&uuids, &weights, count, &mut rng)
    }
}

/// Weighted random sampling with replacement.
fn weighted_choices_with_replacement(
    uuids: &[String],
    weights: &[i64],
    count: usize,
    rng: &mut ThreadRng,
) -> Vec<String> {
    let total_weight: i64 = weights.iter().sum();
    if total_weight <= 0 {
        return Vec::new();
    }

    let mut results = Vec::with_capacity(count);
    for _ in 0..count {
        let mut roll = rng.gen_range(0..total_weight);
        for (i, &w) in weights.iter().enumerate() {
            roll -= w;
            if roll < 0 {
                results.push(uuids[i].clone());
                break;
            }
        }
    }
    results
}

/// Weighted random sampling without replacement.
fn weighted_choices_without_replacement(
    uuids: &[String],
    weights: &[i64],
    count: usize,
    rng: &mut ThreadRng,
) -> Vec<String> {
    let actual_count = count.min(uuids.len());
    let mut remaining_uuids: Vec<String> = uuids.to_vec();
    let mut remaining_weights: Vec<i64> = weights.to_vec();
    let mut results = Vec::with_capacity(actual_count);

    for _ in 0..actual_count {
        if remaining_uuids.is_empty() {
            break;
        }

        let total_weight: i64 = remaining_weights.iter().sum();
        if total_weight <= 0 {
            break;
        }

        let mut roll = rng.gen_range(0..total_weight);
        let mut picked_idx = remaining_uuids.len() - 1;

        for (i, &w) in remaining_weights.iter().enumerate() {
            roll -= w;
            if roll < 0 {
                picked_idx = i;
                break;
            }
        }

        results.push(remaining_uuids.remove(picked_idx));
        remaining_weights.remove(picked_idx);
    }

    results
}
