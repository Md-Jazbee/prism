//! Local agent traces — tool sequences only, never repository content.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const TRACE_SCHEMA_VERSION: &str = "agent-trace/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceEvent {
    pub ts: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_count: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TraceMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_tool: Option<String>,
    pub chose_compile_first: bool,
    pub refusal_count: u32,
    pub repair_success_count: u32,
    pub hops: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_in_estimate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_out_estimate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentTrace {
    pub schema_version: String,
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    pub outcome: String,
    pub events: Vec<TraceEvent>,
    #[serde(default)]
    pub metrics: TraceMetrics,
}

impl AgentTrace {
    pub fn new(workflow_id: Option<String>) -> Self {
        Self {
            schema_version: TRACE_SCHEMA_VERSION.into(),
            trace_id: Uuid::new_v4().to_string(),
            session_id: None,
            workflow_id,
            started_at: now_rfc3339(),
            ended_at: None,
            outcome: "unknown".into(),
            events: Vec::new(),
            metrics: TraceMetrics::default(),
        }
    }

    pub fn finish(&mut self, outcome: &str) {
        self.ended_at = Some(now_rfc3339());
        self.outcome = outcome.into();
        self.metrics = metrics_from_events(&self.events);
    }
}

pub fn metrics_from_events(events: &[TraceEvent]) -> TraceMetrics {
    let first_tool = events
        .iter()
        .find(|e| e.kind == "tool_call")
        .and_then(|e| e.tool.clone());
    let chose_compile_first = first_tool
        .as_deref()
        .map(|t| t == "compile_context")
        .unwrap_or(false);
    let refusal_count = events
        .iter()
        .filter(|e| e.kind == "refusal" || e.error_code.is_some())
        .count() as u32;
    let repair_success_count = events
        .iter()
        .filter(|e| e.kind == "repair" && e.ok == Some(true))
        .count() as u32;
    let hops = events.iter().filter(|e| e.kind == "tool_call").count() as u32;
    TraceMetrics {
        first_tool,
        chose_compile_first,
        refusal_count,
        repair_success_count,
        hops,
        tokens_in_estimate: None,
        tokens_out_estimate: None,
    }
}

pub fn open_trace_log(workspace: &Path) -> Result<PathBuf> {
    let dir = workspace.join(".prism/logs");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("agent-traces.jsonl"))
}

pub fn append_trace_event(workspace: &Path, trace: &AgentTrace) -> Result<()> {
    let path = open_trace_log(workspace)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    serde_json::to_writer(&mut f, trace)?;
    f.write_all(b"\n")?;
    Ok(())
}

fn now_rfc3339() -> String {
    // Avoid chrono dep: unix-ish stamp is enough for local traces.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[allow(dead_code)]
fn _file_type_check(_: File) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_first_metric() {
        let events = vec![
            TraceEvent {
                ts: "1".into(),
                kind: "tool_call".into(),
                tool: Some("compile_context".into()),
                ok: None,
                error_code: None,
                repair_action: None,
                latency_ms: Some(10),
                hit_count: None,
            },
            TraceEvent {
                ts: "2".into(),
                kind: "tool_result".into(),
                tool: Some("compile_context".into()),
                ok: Some(true),
                error_code: None,
                repair_action: None,
                latency_ms: None,
                hit_count: Some(3),
            },
        ];
        let m = metrics_from_events(&events);
        assert!(m.chose_compile_first);
        assert_eq!(m.hops, 1);
    }
}
