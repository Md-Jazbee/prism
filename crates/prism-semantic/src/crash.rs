//! Soft-failure type for semantic path (never crashes agent).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("SEMANTIC_PARTIAL: {message}")]
pub struct SemanticPartial {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl SemanticPartial {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: "SEMANTIC_PARTIAL".into(),
            message: message.into(),
            notes: vec![],
        }
    }
}
