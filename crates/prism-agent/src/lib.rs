//! Agent experience: workflows, refusal repair, progressive packs, traces (P9).

mod assets;
mod catalog;
mod progressive;
mod repair;
mod trace;
mod workflow;

pub use assets::{generate_agents_md, generate_cursor_rule, generate_skill_markdown};
pub use catalog::{load_embedded_catalog, WorkflowCatalog, WorkflowDef, WorkflowStep};
pub use progressive::{negotiate_budget, progressive_layers, ProgressiveLayer, ProgressivePack};
pub use repair::{repair_for, RepairAction, RepairKind};
pub use trace::{
    append_trace_event, metrics_from_events, open_trace_log, AgentTrace, TraceEvent, TraceMetrics,
    TRACE_SCHEMA_VERSION,
};
pub use workflow::{
    expected_trace_tools, list_workflows, run_workflow, StepResult, WorkflowRunResult,
};
