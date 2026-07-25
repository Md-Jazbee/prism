//! Durable storage for Prism (W-STORE).
//!
//! All graph access goes through [`KgStore`] so SQLite → Kuzu is a later switch.

pub mod communities;
pub mod kg;
pub mod meta;
pub mod query;

pub use communities::{Community, Hub, RepoMap};
pub use kg::{KgStore, SqliteKgStore};
pub use meta::SqliteMetaStore;
pub use query::{
    parse_edge_kinds, EdgeDirection, GraphEdgeView, GraphNodeView, ImpactHit, IndexSizeStats,
    NeighborHit,
};
