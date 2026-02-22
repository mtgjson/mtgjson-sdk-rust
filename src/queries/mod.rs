//! Query modules for the MTGJSON SDK.
//!
//! Each module provides a query struct that borrows from a [`Connection`](crate::connection::Connection)
//! (or [`CacheManager`](crate::cache::CacheManager) for JSON-only queries) and exposes
//! methods returning `Result<T>` with `serde_json::Value` payloads.

pub mod cards;
pub mod decks;
pub mod enums;
pub mod identifiers;
pub mod legalities;
pub mod prices;
pub mod sealed;
pub mod sets;
pub mod skus;
pub mod tokens;

pub use cards::{CardQuery, SearchCardsParams};
pub use decks::DeckQuery;
pub use enums::EnumQuery;
pub use identifiers::IdentifierQuery;
pub use legalities::LegalityQuery;
pub use prices::PriceQuery;
pub use sealed::SealedQuery;
pub use sets::{SearchSetsParams, SetQuery};
pub use skus::SkuQuery;
pub use tokens::{SearchTokensParams, TokenQuery};
