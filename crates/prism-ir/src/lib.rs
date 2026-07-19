//! Shared schemas and identity types for Prism (Phase 0).
//!
//! Schema documents live under `schemas/`; this crate holds the Rust mirrors
//! and version constants. Breaking fact/pack/plan schema ⇒ bump major here.

pub mod confidence;
pub mod identity;
pub mod versions;

pub use confidence::Confidence;
pub use identity::{FileId, RepositoryId, SnapshotId};
pub use versions::{EVENTS_SCHEMA_VERSION, FACT_SCHEMA_VERSION, META_SCHEMA_VERSION};
