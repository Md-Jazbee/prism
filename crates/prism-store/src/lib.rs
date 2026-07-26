//! Durable storage for Prism (W-STORE).
//!
//! All graph access goes through [`KgStore`] so SQLite → Kuzu is a later switch.

pub mod communities;
pub mod intel;
pub mod kg;
pub mod lexical;
pub mod meta;
pub mod query;

pub use communities::{Community, Hub, RepoMap};
pub use intel::{DetectChangesReport, Entrypoint, Hotspot, RepoIntelReport, INTEL_ALGO_VERSION};
pub use kg::{KgStore, SqliteKgStore};
pub use lexical::{tokenize_seed_terms, SeedCandidate, MIN_CANDIDATE_SCORE, MIN_GROUND_SCORE};
pub use meta::{SqliteMetaStore, ANALYZER_PIPELINE_VERSION};
pub use query::{
    parse_edge_kinds, AmbiguousSymbolGroup, EdgeDirection, GraphEdgeView, GraphNodeView, ImpactHit,
    IndexSizeStats, NeighborHit,
};
