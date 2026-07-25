//! Deterministic intent classification (no LLM).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Agent / natural-language intent set (ADD §17.2 / F7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    RepoQa,
    Debug,
    Impact,
    Refactor,
    Generate,
    Review,
    Architecture,
}

impl Intent {
    pub const ALL: &'static [Intent] = &[
        Intent::RepoQa,
        Intent::Debug,
        Intent::Impact,
        Intent::Refactor,
        Intent::Generate,
        Intent::Review,
        Intent::Architecture,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Intent::RepoQa => "repo_qa",
            Intent::Debug => "debug",
            Intent::Impact => "impact",
            Intent::Refactor => "refactor",
            Intent::Generate => "generate",
            Intent::Review => "review",
            Intent::Architecture => "architecture",
        }
    }
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Intent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "repo_qa" | "repo-qa" | "repoqa" | "qa" => Ok(Intent::RepoQa),
            "debug" => Ok(Intent::Debug),
            "impact" => Ok(Intent::Impact),
            "refactor" => Ok(Intent::Refactor),
            "generate" => Ok(Intent::Generate),
            "review" => Ok(Intent::Review),
            "architecture" | "arch" => Ok(Intent::Architecture),
            other => Err(format!("unknown intent: {other}")),
        }
    }
}

/// Optional planner hints from the agent / CLI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanHints {
    /// Force intent (skip keyword classifier).
    pub intent_override: Option<Intent>,
    /// Explicit symbol / path anchors.
    pub anchors: Vec<String>,
    pub stack_frames: Vec<String>,
    pub error_text: Option<String>,
    pub changed_paths: Vec<String>,
    pub budget_tokens: Option<u32>,
}

/// Classify intent from question text + hints using keyword heuristics.
pub fn classify_intent(question: &str, hints: &PlanHints) -> Intent {
    let q = question.to_ascii_lowercase();

    if !hints.stack_frames.is_empty() || hints.error_text.is_some() {
        return Intent::Debug;
    }
    if !hints.changed_paths.is_empty()
        && (q.contains("review") || q.contains("pr ") || q.contains("pull request"))
    {
        return Intent::Review;
    }
    if !hints.changed_paths.is_empty()
        && (q.contains("impact") || q.contains("affect") || q.contains("break"))
    {
        return Intent::Impact;
    }

    if contains_any(
        &q,
        &[
            "traceback",
            "stack trace",
            "stack frame",
            "exception",
            "panic",
            "segfault",
            "crash",
            "debug why",
            "why does",
            "bug in",
            "error:",
        ],
    ) {
        return Intent::Debug;
    }
    if contains_any(
        &q,
        &[
            "impact of",
            "who calls",
            "callers of",
            "what breaks",
            "affected by",
            "blast radius",
            "depend on",
        ],
    ) {
        return Intent::Impact;
    }
    if contains_any(
        &q,
        &[
            "refactor",
            "rename",
            "safe move",
            "all references",
            "find references",
        ],
    ) {
        return Intent::Refactor;
    }
    if contains_any(
        &q,
        &[
            "generate",
            "implement",
            "scaffold",
            "write a function",
            "add a method",
            "create a class",
        ],
    ) {
        return Intent::Generate;
    }
    if contains_any(
        &q,
        &[
            "architecture",
            "subsystem",
            "repo map",
            "module map",
            "communities",
            "hubs",
            "how is the repo organized",
            "high-level overview",
        ],
    ) {
        return Intent::Architecture;
    }
    if contains_any(
        &q,
        &[
            "review this",
            "code review",
            "pull request",
            "pr diff",
            "review the change",
        ],
    ) {
        return Intent::Review;
    }

    Intent::RepoQa
}

/// Pull likely anchors from free text (backticked ids, paths, CapWords, `file:line`).
pub fn extract_anchors(question: &str) -> Vec<String> {
    let mut out = Vec::new();

    // `backticked` tokens
    let mut rest = question;
    while let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find('`') {
            let tok = after[..end].trim();
            if is_anchor_token(tok) {
                push_unique(&mut out, tok.to_string());
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }

    // path-like tokens with extension or slash
    for tok in question.split_whitespace() {
        let cleaned = tok.trim_matches(|c: char| {
            matches!(c, ',' | '.' | ';' | ':' | ')' | '(' | '"' | '\'' | '?' | '!')
        });
        if cleaned.contains('/')
            && (cleaned.contains('.') || cleaned.ends_with(".py") || cleaned.ends_with(".rs"))
        {
            push_unique(&mut out, cleaned.to_string());
        } else if looks_like_symbol(cleaned) {
            push_unique(&mut out, cleaned.to_string());
        }
    }

    out
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

fn push_unique(out: &mut Vec<String>, s: String) {
    if !out.iter().any(|x| x == &s) {
        out.push(s);
    }
}

fn is_anchor_token(tok: &str) -> bool {
    !tok.is_empty()
        && tok.len() < 200
        && (looks_like_symbol(tok) || tok.contains('/') || tok.contains('.'))
}

fn looks_like_symbol(tok: &str) -> bool {
    if tok.len() < 2 || tok.len() > 120 {
        return false;
    }
    // CapWords / snake_case identifiers; reject pure lowercase stopwords
    let stop = [
        "the", "and", "for", "what", "where", "when", "how", "does", "this", "that", "with",
        "from", "into", "about", "please", "show", "tell", "explain",
    ];
    if stop.contains(&tok.to_ascii_lowercase().as_str()) {
        return false;
    }
    let chars: Vec<char> = tok.chars().collect();
    let alnum_underscore = chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':');
    if !alnum_underscore {
        return false;
    }
    // Prefer CamelCase, SCREAMING, or snake with underscore / qualified ::
    tok.contains('_')
        || tok.contains("::")
        || tok.chars().any(|c| c.is_ascii_uppercase())
}
