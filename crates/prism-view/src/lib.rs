//! Graph View-Model projector (P6 Stage C).
//!
//! Views are budgeted projections — the visual analogue of Evidence Packs.
//! See `schemas/graph-view/v1` and `docs/architecture/GRAPH-VIEW-MODEL.md`.

mod budget;
mod kinds;
mod layout;
mod model;
mod project;

pub use budget::{ViewBudget, DEFAULT_MAX_EDGES, DEFAULT_MAX_NODES};
pub use kinds::ViewKind;
pub use model::{
    Citation, DropRecord, GraphView, LayoutInfo, ViewEdge, ViewGroup, ViewNode, ViewTooLarge,
    ViewOutcome, GRAPH_VIEW_SCHEMA_VERSION,
};
pub use project::{project_view, ViewParams};
