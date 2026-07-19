//! Observability baseline for Phase 0 (W-OBS).
//!
//! Events can be emitted to tracing logs now; OTel exporters land later.

pub mod events;

pub use events::{emit_index_event, IndexEvent, IndexStats};
