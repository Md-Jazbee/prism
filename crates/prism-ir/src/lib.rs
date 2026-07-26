//! Shared schemas and identity types for Prism (Phase 0–1).
//!
//! Schema documents live under `schemas/`; this crate holds the Rust mirrors
//! and version constants. Breaking fact/pack/plan schema ⇒ bump major here.

pub mod confidence;
pub mod facts;
pub mod identity;
pub mod versions;

pub use confidence::Confidence;
pub use facts::{
    doc_node_id, edge_id, file_node_id, section_node_id, slugify, symbol_node_id,
    unresolved_node_id, EdgeKind, FactBundle, FactEdge, FactNode, NodeKind, Span, Tier,
};
pub use identity::{FileId, RepositoryId, SnapshotId};
pub use versions::{
    EVENTS_SCHEMA_VERSION, FACT_SCHEMA_VERSION, META_SCHEMA_VERSION, PACK_SCHEMA_VERSION,
    PLAN_SCHEMA_VERSION, PRECISE_INDEX_SCHEMA_VERSION,
};
