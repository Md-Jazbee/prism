//! Phase 0 foundations: workspace identity, fingerprinting, incremental path.

pub mod fingerprint;
pub mod ignore_policy;
pub mod incremental;
pub mod workspace;

pub use fingerprint::{file_content_hash, hash_bytes, merkle_combine};
pub use ignore_policy::{is_secret_sensitive, IgnorePolicy};
pub use incremental::{IncrementalIndexer, IndexOptions, IndexResult};
pub use workspace::{WorkspaceIdentity, WorkspaceManager};
