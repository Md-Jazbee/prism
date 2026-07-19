//! Durable storage for Prism (W-STORE).
//!
//! All graph access goes through [`KgStore`] so SQLite → Kuzu is a later switch.

pub mod kg;
pub mod meta;

pub use kg::{KgStore, SqliteKgStore};
pub use meta::SqliteMetaStore;
