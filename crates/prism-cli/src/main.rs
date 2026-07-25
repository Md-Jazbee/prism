//! `prism` CLI — Phase 2 Stage A (`index` + structural `query` + `query plan`).

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use prism_obs::{emit_index_event, IndexEvent};
use prism_plan::{plan_query, Intent, PlanHints, PlanOutcome};
use prism_store::{parse_edge_kinds, EdgeDirection, SqliteKgStore, SqliteMetaStore};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "prism",
    version,
    about = "Prism — Repository Intelligence Platform",
    long_about = "Pre-LLM repository understanding: index → knowledge graph → context compilation.\nPhase 2 Stage A: deterministic query plans (`prism query plan`)."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index a workspace (discover → hash → T1 extract → txn → invalidate).
    Index {
        /// Workspace root (default: cwd).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Discover and hash without writing meta/graph updates.
        #[arg(long)]
        dry_run: bool,
    },
    /// Print workspace identity (git SHA, dirty, tree fingerprint).
    Doctor {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show index freshness and graph cardinality.
    #[command(name = "index-status")]
    IndexStatus {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Structural KG queries (Stage B).
    Query {
        #[command(subcommand)]
        query: QueryCmd,
    },
    /// Serve MCP structural tools over stdio (Stage C).
    Mcp {
        /// Workspace root that already has `.prism/` index.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum QueryCmd {
    /// Lookup symbols by exact name.
    Resolve {
        name: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Optional path substring filter.
        #[arg(long)]
        file: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// 1-hop neighbors of a node id.
    Neighbors {
        id: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Comma-separated edge kinds (e.g. CALLS,IMPORTS).
        #[arg(long)]
        kind: Option<String>,
        #[arg(long, value_enum, default_value_t = DirArg::Outgoing)]
        dir: DirArg,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Depth-limited heuristic impact candidates from a seed node id.
    Impact {
        id: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 2)]
        depth: u32,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Files that should be rechecked when a path changes (reverse-dep dirty list).
    Dirty {
        /// Changed repo-relative path.
        changed: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Path-prefix communities + hubs (Stage D).
    #[command(name = "repo-map")]
    RepoMap {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 15)]
        hub_limit: usize,
    },
    /// Deterministic query plan only (P2 Stage A) — no Evidence Pack yet.
    Plan {
        /// Natural-language or agent question.
        question: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Force intent: repo_qa|debug|impact|refactor|generate|review|architecture
        #[arg(long)]
        intent: Option<String>,
        /// Token budget hint for later BudgetPack (default 4000).
        #[arg(long, default_value_t = 4000)]
        budget: u32,
        /// Explicit anchors (repeatable).
        #[arg(long = "anchor")]
        anchors: Vec<String>,
        /// Stack frame strings (debug).
        #[arg(long = "stack")]
        stack_frames: Vec<String>,
        /// Error / exception text (debug).
        #[arg(long)]
        error: Option<String>,
        /// Changed paths for review/impact (repeatable).
        #[arg(long = "changed")]
        changed_paths: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DirArg {
    Outgoing,
    Incoming,
    Both,
}

impl From<DirArg> for EdgeDirection {
    fn from(d: DirArg) -> Self {
        match d {
            DirArg::Outgoing => EdgeDirection::Outgoing,
            DirArg::Incoming => EdgeDirection::Incoming,
            DirArg::Both => EdgeDirection::Both,
        }
    }
}

fn open_kg(workspace: &PathBuf) -> Result<(WorkspaceManager, SqliteKgStore)> {
    let wm = WorkspaceManager::open(workspace)
        .with_context(|| format!("open workspace {}", workspace.display()))?;
    let kg = SqliteKgStore::open(wm.root().join(".prism/graph.sqlite"))?;
    Ok((wm, kg))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // MCP reserves stdout for JSON-RPC — log to stderr always for mcp; others ok on stderr too.
    let is_mcp = matches!(cli.command, Commands::Mcp { .. });
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_writer(std::io::stderr);
    if is_mcp {
        subscriber.with_ansi(false).init();
    } else {
        subscriber.init();
    }

    match cli.command {
        Commands::Index { path, dry_run } => {
            let wm = WorkspaceManager::open(&path)
                .with_context(|| format!("open workspace {}", path.display()))?;
            let prism_dir = wm.root().join(".prism");
            let mut indexer = IncrementalIndexer::open(wm, &prism_dir)?;
            let result = indexer.run(&IndexOptions { dry_run })?;
            println!(
                "indexed: discovered={} hashed={} extracted={} extract_skipped={} skipped_unchanged={} secret_skipped={} nodes={} edges={} unresolved_calls={} wall_ms={} fingerprint={}{}",
                result.stats.files_discovered,
                result.stats.files_hashed,
                result.stats.files_extracted,
                result.stats.files_extract_skipped,
                result.stats.files_skipped_unchanged,
                result.stats.files_secret_skipped,
                result.stats.nodes_written,
                result.stats.edges_written,
                result.stats.unresolved_calls,
                result.stats.wall_time_ms,
                &result.tree_fingerprint[..result.tree_fingerprint.len().min(16)],
                if dry_run { " (dry-run)" } else { "" }
            );
        }
        Commands::Doctor { path } => {
            let wm = WorkspaceManager::open(&path)?;
            let id = wm.identity()?;
            println!("root: {}", id.repository.root.display());
            println!(
                "git_commit: {}",
                id.snapshot.git_commit.as_deref().unwrap_or("(none)")
            );
            println!("dirty: {}", id.snapshot.dirty);
            println!("tree_fingerprint: {}", id.snapshot.tree_fingerprint);
            println!(
                "schema: meta={} fact={} events={} plan={}",
                prism_ir::META_SCHEMA_VERSION,
                prism_ir::FACT_SCHEMA_VERSION,
                prism_ir::EVENTS_SCHEMA_VERSION,
                prism_ir::PLAN_SCHEMA_VERSION
            );
        }
        Commands::IndexStatus { path } => {
            let wm = WorkspaceManager::open(&path)?;
            let prism = wm.root().join(".prism");
            if !prism.join("graph.sqlite").exists() {
                bail!("no index at {} — run `prism index` first", prism.display());
            }
            let meta = SqliteMetaStore::open(prism.join("meta.sqlite"))?;
            let kg = SqliteKgStore::open(prism.join("graph.sqlite"))?;
            let stats = kg.index_stats()?;
            let files = meta.list_file_paths()?.len();
            let id = wm.identity()?;
            let graph_bytes = std::fs::metadata(prism.join("graph.sqlite"))
                .map(|m| m.len())
                .unwrap_or(0);
            let meta_bytes = std::fs::metadata(prism.join("meta.sqlite"))
                .map(|m| m.len())
                .unwrap_or(0);
            println!("root: {}", wm.root().display());
            println!(
                "git_commit: {}",
                id.snapshot.git_commit.as_deref().unwrap_or("(none)")
            );
            println!("dirty_worktree: {}", id.snapshot.dirty);
            println!("tree_fingerprint: {}", id.snapshot.tree_fingerprint);
            println!("files_hashed: {files}");
            println!(
                "graph: nodes={} edges={} files_indexed={} graph_sqlite_bytes={} meta_sqlite_bytes={}",
                stats.nodes, stats.edges, stats.files_indexed, graph_bytes, meta_bytes
            );
            println!(
                "nfr_note: design targets local query P95 <50ms; index size ~3–10% of source (see docs/architecture/INDEX-SIZE-BUDGET.md)"
            );
        }
        Commands::Query { query } => match query {
            QueryCmd::Resolve {
                name,
                path,
                file,
                limit,
            } => {
                let (_wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let hits = kg.resolve_symbol(&name, file.as_deref(), limit)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "resolve".into(),
                    latency_ms: ms,
                    hit_count: hits.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&hits)?);
                eprintln!("# resolve hits={} latency_ms={ms}", hits.len());
            }
            QueryCmd::Neighbors {
                id,
                path,
                kind,
                dir,
                limit,
            } => {
                let (_wm, kg) = open_kg(&path)?;
                let kinds = parse_edge_kinds(kind.as_deref());
                let started = Instant::now();
                let hits = kg.neighbors(&id, kinds.as_deref(), dir.into(), limit)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "neighbors".into(),
                    latency_ms: ms,
                    hit_count: hits.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&hits)?);
                eprintln!("# neighbors hits={} latency_ms={ms}", hits.len());
            }
            QueryCmd::Impact {
                id,
                path,
                depth,
                limit,
            } => {
                let (_wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let hits = kg.impact(&id, depth, limit)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "impact".into(),
                    latency_ms: ms,
                    hit_count: hits.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&hits)?);
                eprintln!(
                    "# impact hits={} depth={depth} latency_ms={ms} (heuristic T1)",
                    hits.len()
                );
            }
            QueryCmd::Dirty { changed, path } => {
                let (_wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let dirty = kg.reverse_dep_files(&changed)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "dirty".into(),
                    latency_ms: ms,
                    hit_count: dirty.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&dirty)?);
                eprintln!("# dirty files={} latency_ms={ms}", dirty.len());
            }
            QueryCmd::RepoMap { path, hub_limit } => {
                let (_wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let map = kg.repo_map(hub_limit)?;
                let ms = started.elapsed().as_millis() as u64;
                let hits = (map.communities.len() + map.hubs.len()) as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "repo_map".into(),
                    latency_ms: ms,
                    hit_count: hits,
                });
                println!("{}", serde_json::to_string_pretty(&map)?);
                eprintln!(
                    "# repo_map communities={} hubs={} latency_ms={ms}",
                    map.communities.len(),
                    map.hubs.len()
                );
            }
            QueryCmd::Plan {
                question,
                path: _path,
                intent,
                budget,
                anchors,
                stack_frames,
                error,
                changed_paths,
            } => {
                let mut hints = PlanHints {
                    anchors,
                    stack_frames,
                    error_text: error,
                    changed_paths,
                    budget_tokens: Some(budget),
                    ..Default::default()
                };
                if let Some(raw) = intent {
                    hints.intent_override = Some(
                        Intent::from_str(&raw)
                            .map_err(|e| anyhow::anyhow!(e))
                            .with_context(|| format!("invalid --intent {raw}"))?,
                    );
                }
                let started = Instant::now();
                let outcome = plan_query(&question, &hints)?;
                let ms = started.elapsed().as_millis() as u64;
                let hit_count = match &outcome {
                    PlanOutcome::Ok(p) => p.steps.len() as u64,
                    PlanOutcome::ScopeUnresolved(_) => 0,
                };
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "plan".into(),
                    latency_ms: ms,
                    hit_count,
                });
                println!("{}", serde_json::to_string_pretty(&outcome)?);
                match &outcome {
                    PlanOutcome::Ok(p) => eprintln!(
                        "# plan status=ok intent={} steps={} budget={} latency_ms={ms}",
                        p.intent,
                        p.steps.len(),
                        p.budget_tokens
                    ),
                    PlanOutcome::ScopeUnresolved(e) => eprintln!(
                        "# plan status=scope_unresolved code={} latency_ms={ms}",
                        e.code
                    ),
                }
            }
        },
        Commands::Mcp { path } => {
            let wm = WorkspaceManager::open(&path)
                .with_context(|| format!("open workspace {}", path.display()))?;
            prism_mcp::serve_stdio(wm.root().to_path_buf())?;
        }
    }
    Ok(())
}
