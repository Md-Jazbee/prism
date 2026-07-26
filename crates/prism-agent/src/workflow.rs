//! Execute a named workflow against MCP tool surface (local).

use crate::catalog::{load_embedded_catalog, WorkflowDef};
use crate::repair::repair_for;
use crate::trace::{append_trace_event, AgentTrace, TraceEvent};
use anyhow::{bail, Result};
use prism_mcp::{call_tool, ToolContext, ToolOutcome};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunResult {
    pub workflow_id: String,
    pub ok: bool,
    pub steps: Vec<StepResult>,
    pub trace: AgentTrace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub tool: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repair: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Merge runtime overrides (anchors, question, changed_paths, …) into step args.
pub fn run_workflow(
    workspace: &std::path::Path,
    workflow_id: &str,
    overrides: &Value,
    persist_trace: bool,
) -> Result<WorkflowRunResult> {
    let catalog = load_embedded_catalog()?;
    let wf = catalog
        .get(workflow_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("unknown workflow '{workflow_id}'"))?;
    run_workflow_def(workspace, &wf, overrides, persist_trace)
}

fn run_workflow_def(
    workspace: &std::path::Path,
    wf: &WorkflowDef,
    overrides: &Value,
    persist_trace: bool,
) -> Result<WorkflowRunResult> {
    let ctx = match ToolContext::open(workspace) {
        Ok(c) => c,
        Err(e) => bail!("{e}"),
    };
    let mut trace = AgentTrace::new(Some(wf.id.clone()));
    let mut steps = Vec::new();
    let mut last_pack = None;
    let mut all_ok = true;

    for step in &wf.steps {
        let mut args = step.args.clone();
        if let Some(obj) = args.as_object_mut() {
            if let Some(o) = overrides.as_object() {
                for (k, v) in o {
                    obj.insert(k.clone(), v.clone());
                }
            }
        } else {
            args = overrides.clone();
        }

        trace.events.push(TraceEvent {
            ts: format!("{}", steps.len()),
            kind: "tool_call".into(),
            tool: Some(step.tool.clone()),
            ok: None,
            error_code: None,
            repair_action: None,
            latency_ms: None,
            hit_count: None,
        });

        let outcome = call_tool(&ctx, &step.tool, args);
        match outcome {
            ToolOutcome::Ok(success) => {
                let result = success.result;
                if step.tool == "compile_context" {
                    last_pack = Some(result.clone());
                }
                trace.events.push(TraceEvent {
                    ts: format!("{}", steps.len()),
                    kind: "tool_result".into(),
                    tool: Some(step.tool.clone()),
                    ok: Some(true),
                    error_code: None,
                    repair_action: None,
                    latency_ms: Some(success.latency_ms),
                    hit_count: None,
                });
                steps.push(StepResult {
                    step_id: step.id.clone(),
                    tool: step.tool.clone(),
                    ok: true,
                    error_code: None,
                    repair: None,
                    result: Some(result),
                });
            }
            ToolOutcome::Err { error } => {
                let code = serde_json::to_value(&error.code)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_else(|| format!("{:?}", error.code).to_uppercase());
                let repair = error.repair.clone().unwrap_or_else(|| {
                    serde_json::to_value(repair_for(
                        &code,
                        &error.message,
                        error.hint.clone(),
                    ))
                    .unwrap_or(json!({}))
                });
                let repair_action = repair
                    .get("action")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                trace.events.push(TraceEvent {
                    ts: format!("{}", steps.len()),
                    kind: "refusal".into(),
                    tool: Some(step.tool.clone()),
                    ok: Some(false),
                    error_code: Some(code.clone()),
                    repair_action,
                    latency_ms: None,
                    hit_count: None,
                });
                if step.optional {
                    steps.push(StepResult {
                        step_id: step.id.clone(),
                        tool: step.tool.clone(),
                        ok: false,
                        error_code: Some(code),
                        repair: Some(repair),
                        result: None,
                    });
                    continue;
                }
                all_ok = false;
                steps.push(StepResult {
                    step_id: step.id.clone(),
                    tool: step.tool.clone(),
                    ok: false,
                    error_code: Some(code),
                    repair: Some(repair),
                    result: None,
                });
                break;
            }
        }
    }

    let outcome = if all_ok { "ok" } else { "refused" };
    trace.finish(outcome);
    if persist_trace {
        let _ = append_trace_event(workspace, &trace);
    }

    Ok(WorkflowRunResult {
        workflow_id: wf.id.clone(),
        ok: all_ok,
        steps,
        trace,
        pack: last_pack,
    })
}

/// Expected tool sequence for fixtures (no execution).
pub fn expected_trace_tools(workflow_id: &str) -> Result<Vec<String>> {
    let catalog = load_embedded_catalog()?;
    let wf = catalog
        .get(workflow_id)
        .ok_or_else(|| anyhow::anyhow!("unknown workflow"))?;
    Ok(wf.steps.iter().map(|s| s.tool.clone()).collect())
}

pub fn list_workflows() -> Result<Value> {
    let catalog = load_embedded_catalog()?;
    Ok(json!({
        "schema_version": catalog.schema_version,
        "workflows": catalog.workflows.iter().map(|w| json!({
            "id": w.id,
            "title": w.title,
            "trigger": w.trigger,
            "steps": w.steps.iter().map(|s| s.tool.clone()).collect::<Vec<_>>(),
            "gold_task_ids": w.gold_task_ids,
        })).collect::<Vec<_>>()
    }))
}

#[allow(dead_code)]
fn _bail_unused() -> Result<()> {
    bail!("unused")
}
