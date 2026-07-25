//! Prism MCP structural tool surface (Phase 1 Stages C–D).

pub mod errors;
pub mod server;
pub mod tools;

pub use errors::{ToolError, ToolErrorCode};
pub use server::serve_stdio;
pub use tools::{
    call_tool, dispatch_json, list_tools_schema, ToolContext, ToolOutcome, ALLOWED_TOOLS,
};
