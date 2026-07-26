//! Workflow catalog (embedded from schemas/agent-workflow/v1/catalog.json).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CATALOG_JSON: &str = include_str!("../../../schemas/agent-workflow/v1/catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowCatalog {
    pub schema_version: String,
    pub workflows: Vec<WorkflowDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowDef {
    pub id: String,
    pub title: String,
    pub trigger: String,
    pub steps: Vec<WorkflowStep>,
    pub expected_pack_shape: String,
    #[serde(default)]
    pub refusal_points: Vec<String>,
    #[serde(default)]
    pub gold_task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStep {
    pub id: String,
    pub tool: String,
    #[serde(default)]
    pub args: Value,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub on_refusal: Option<String>,
}

pub fn load_embedded_catalog() -> Result<WorkflowCatalog> {
    serde_json::from_str(CATALOG_JSON).context("parse embedded agent-workflow catalog")
}

impl WorkflowCatalog {
    pub fn get(&self, id: &str) -> Option<&WorkflowDef> {
        self.workflows.iter().find(|w| w.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_four_workflows() {
        let c = load_embedded_catalog().unwrap();
        assert_eq!(c.schema_version, "agent-workflow/v1");
        assert_eq!(c.workflows.len(), 4);
        for id in ["onboarding", "review", "debug", "refactor_prep"] {
            assert!(c.get(id).is_some(), "missing {id}");
        }
    }
}
