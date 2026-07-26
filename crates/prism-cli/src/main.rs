//! `prism` CLI — Phase 4 (`index` + query + plan + compile + precise + semantic).

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use prism_compile::CompileOutcome;
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use prism_obs::{emit_index_event, IndexEvent};
use prism_plan::{plan_query, Intent, PlanHints, PlanOutcome};
use prism_precise::{
    ambiguity_index, import_precise_index, load_precise_index, precision_required, read_manifest,
    rename_dry_run, score_call_resolution, CallEdge, PrecisionGate,
};
use prism_semantic::{
    build_file_artifact, build_workspace_python, ensure_shard, interproc_slice, local_slice,
    read_manifest as read_semantic_manifest, SliceCriterion, SliceDirection, SliceParams,
};
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
    long_about = "Pre-LLM repository understanding: index → knowledge graph → context compilation.\nPhase 3: precise tier overlay (`prism precise import`)."
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
    /// Structural KG queries + plan-only.
    Query {
        #[command(subcommand)]
        query: QueryCmd,
    },
    /// Compile an Evidence Pack under token budget (P2 Stage B).
    Compile {
        /// Natural-language or agent question.
        question: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Force intent: repo_qa|debug|impact|refactor|generate|review|architecture
        #[arg(long)]
        intent: Option<String>,
        /// Token budget (default 4000).
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
        /// Offline synthetic pack (no KG) — for fixtures / budget demos.
        #[arg(long)]
        synthetic: bool,
    },
    /// Serve MCP structural tools over stdio (Stage C).
    Mcp {
        /// Workspace root that already has `.prism/` index.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Precise tier (T2) — import SCIP/PreciseIndex overlays (P3 Stage A).
    Precise {
        #[command(subcommand)]
        cmd: PreciseCmd,
    },
    /// Semantic tier (T3) — intra-proc CFG/DFG + local slice (P4 Stage A).
    Semantic {
        #[command(subcommand)]
        cmd: SemanticCmd,
    },
}

#[derive(Subcommand, Debug)]
enum PreciseCmd {
    /// Import a PreciseIndex JSON and refine heuristic CALLS/REFERENCES.
    Import {
        /// Path to PreciseIndex JSON (`schemas/precise-index/v0`).
        index: PathBuf,
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
    },
    /// Show attached precise overlay manifest (or PRECISION_REQUIRED).
    Status {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Score T1 vs T2 call resolution against an oracle fixture.
    Score {
        #[arg(long)]
        t1: PathBuf,
        #[arg(long)]
        oracle: PathBuf,
        #[arg(long)]
        t2: PathBuf,
    },
    /// Ambiguity index over CALLS (feeds optional UpgradePrecision).
    Ambiguity {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Safe rename dry-run (read-only; T2 required unless --allow-heuristic).
    #[command(name = "rename-dry-run")]
    RenameDryRun {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        new_name: String,
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
        /// Bypass PRECISION_REQUIRED; results stay labeled heuristic.
        #[arg(long)]
        allow_heuristic: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SemanticCmd {
    /// Build T3 CFG/DFG artifacts for Python files under the workspace.
    Build {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Optional single repo-relative file.
        #[arg(long)]
        file: Option<String>,
    },
    /// Local or inter-procedural slice for a line or symbol criterion.
    Slice {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Repo-relative source path.
        #[arg(long)]
        file: String,
        /// 0-based line criterion (mutually exclusive with --symbol).
        #[arg(long)]
        line: Option<u32>,
        /// Function name criterion.
        #[arg(long)]
        symbol: Option<String>,
        /// Inter-procedural CALLS depth (0 = local only via interproc with depth 0).
        #[arg(long, default_value_t = 2)]
        depth: u32,
        /// Slice direction.
        #[arg(long, default_value = "backward")]
        direction: String,
        /// Force local-only (Stage A) slice.
        #[arg(long)]
        local: bool,
    },
    /// Build a call-graph shard around a seed function.
    ShardBuild {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        file: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value_t = 2)]
        depth: u32,
    },
    /// Show semantic manifest (or note if missing).
    Status {
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
        /// Require T2 overlay for accuracy claims.
        #[arg(long)]
        require_precise: bool,
    },
    /// Files that should be rechecked when a path changes (reverse-dep dirty list).
    Dirty {
        /// Changed repo-relative path.
        changed: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Path-prefix communities + hubs (Stage D / P5).
    #[command(name = "repo-map")]
    RepoMap {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 15)]
        hub_limit: usize,
        /// Include entrypoints, layering, hotspots, contracts.
        #[arg(long)]
        full: bool,
    },
    /// Heuristic entrypoints (main/cli/handlers).
    Entrypoints {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 40)]
        limit: usize,
    },
    /// Dirty reverse-deps + change hotspots.
    #[command(name = "detect-changes")]
    DetectChanges {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Changed repo-relative paths (repeatable).
        #[arg(long = "changed")]
        changed: Vec<String>,
    },
    /// Full repo intelligence report (P5 Stage A).
    Intel {
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
                "schema: meta={} fact={} events={} plan={} pack={} precise={}",
                prism_ir::META_SCHEMA_VERSION,
                prism_ir::FACT_SCHEMA_VERSION,
                prism_ir::EVENTS_SCHEMA_VERSION,
                prism_ir::PLAN_SCHEMA_VERSION,
                prism_ir::PACK_SCHEMA_VERSION,
                prism_ir::PRECISE_INDEX_SCHEMA_VERSION
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
                require_precise,
            } => {
                let (wm, kg) = open_kg(&path)?;
                if require_precise {
                    if let Err(e) = prism_precise::require_precise_claim(wm.root(), &kg, Some(&id))
                    {
                        println!("{}", serde_json::to_string_pretty(&e)?);
                        eprintln!("# impact status=PRECISION_REQUIRED");
                        return Ok(());
                    }
                }
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
                    "# impact hits={} depth={depth} latency_ms={ms} require_precise={require_precise} ({})",
                    hits.len(),
                    if require_precise {
                        "T2 gated"
                    } else {
                        "heuristic T1"
                    }
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
            QueryCmd::RepoMap {
                path,
                hub_limit,
                full,
            } => {
                let (wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                if full {
                    let report = kg.repo_intel(Some(wm.root()), hub_limit)?;
                    let ms = started.elapsed().as_millis() as u64;
                    emit_index_event(&IndexEvent::QueryFinished {
                        op: "repo_intel".into(),
                        latency_ms: ms,
                        hit_count: (report.entrypoints.len() + report.repo_map.hubs.len()) as u64,
                    });
                    println!("{}", serde_json::to_string_pretty(&report)?);
                    eprintln!(
                        "# intel entrypoints={} hubs={} layering={} require_t2={} latency_ms={ms}",
                        report.entrypoints.len(),
                        report.repo_map.hubs.len(),
                        report.layering_violations.len(),
                        report.require_t2_hint
                    );
                } else {
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
            }
            QueryCmd::Entrypoints { path, limit } => {
                let (_wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let eps = kg.detect_entrypoints(limit)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "entrypoints".into(),
                    latency_ms: ms,
                    hit_count: eps.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&eps)?);
                eprintln!("# entrypoints count={} latency_ms={ms}", eps.len());
            }
            QueryCmd::DetectChanges { path, changed } => {
                let (wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let report = kg.detect_changes(Some(wm.root()), &changed)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "detect_changes".into(),
                    latency_ms: ms,
                    hit_count: (report.dirty_files.len() + report.hotspots.len()) as u64,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                eprintln!(
                    "# detect_changes dirty={} hotspots={} latency_ms={ms}",
                    report.dirty_files.len(),
                    report.hotspots.len()
                );
            }
            QueryCmd::Intel { path, hub_limit } => {
                let (wm, kg) = open_kg(&path)?;
                let started = Instant::now();
                let report = kg.repo_intel(Some(wm.root()), hub_limit)?;
                let ms = started.elapsed().as_millis() as u64;
                emit_index_event(&IndexEvent::QueryFinished {
                    op: "repo_intel".into(),
                    latency_ms: ms,
                    hit_count: report.entrypoints.len() as u64,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                eprintln!(
                    "# intel algo={} entrypoints={} require_t2={} latency_ms={ms}",
                    report.algo_version,
                    report.entrypoints.len(),
                    report.require_t2_hint
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
        Commands::Compile {
            question,
            path,
            intent,
            budget,
            anchors,
            stack_frames,
            error,
            changed_paths,
            synthetic,
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
            let outcome = if synthetic {
                match plan_query(&question, &hints)? {
                    PlanOutcome::ScopeUnresolved(u) => CompileOutcome::ScopeUnresolved(u),
                    PlanOutcome::Ok(plan) => prism_compile::compile_synthetic(&plan),
                }
            } else {
                let wm = WorkspaceManager::open(&path)
                    .with_context(|| format!("open workspace {}", path.display()))?;
                prism_compile::compile_context(wm.root(), &question, &hints)?
            };
            let ms = started.elapsed().as_millis() as u64;
            let hit_count = match &outcome {
                CompileOutcome::Ok(p) => p.fragments.len() as u64,
                _ => 0,
            };
            emit_index_event(&IndexEvent::QueryFinished {
                op: "compile".into(),
                latency_ms: ms,
                hit_count,
            });
            println!("{}", serde_json::to_string_pretty(&outcome)?);
            match &outcome {
                CompileOutcome::Ok(p) => eprintln!(
                    "# compile status=ok intent={} fragments={} tokens={}/{} latency_ms={ms}",
                    p.meta.intent,
                    p.fragments.len(),
                    p.meta.tokens_used,
                    p.meta.budget_tokens
                ),
                CompileOutcome::ScopeUnresolved(e) => eprintln!(
                    "# compile status=scope_unresolved code={} latency_ms={ms}",
                    e.code
                ),
                CompileOutcome::BudgetExceeded(e) => eprintln!(
                    "# compile status=budget_exceeded must={} budget={} latency_ms={ms}",
                    e.must_include_tokens, e.budget_tokens
                ),
            }
        }
        Commands::Mcp { path } => {
            let wm = WorkspaceManager::open(&path)
                .with_context(|| format!("open workspace {}", path.display()))?;
            prism_mcp::serve_stdio(wm.root().to_path_buf())?;
        }
        Commands::Precise { cmd } => match cmd {
            PreciseCmd::Import { index, workspace } => {
                let wm = WorkspaceManager::open(&workspace)
                    .with_context(|| format!("open workspace {}", workspace.display()))?;
                let id = wm.identity()?;
                let (manifest, stats) = import_precise_index(
                    wm.root(),
                    &index,
                    id.snapshot.git_commit.clone(),
                    Some(id.snapshot.tree_fingerprint.clone()),
                )?;
                println!("{}", serde_json::to_string_pretty(&manifest)?);
                eprintln!(
                    "# precise import language={} symbols={} edges={} refined={} inserted={}",
                    manifest.language,
                    manifest.symbols,
                    manifest.edges,
                    stats.refined,
                    stats.inserted
                );
            }
            PreciseCmd::Status { path } => {
                let wm = WorkspaceManager::open(&path)?;
                if let Some(m) = read_manifest(wm.root())? {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                    eprintln!("# precise status=ok analyzer={}", m.analyzer);
                } else {
                    let err = precision_required(
                        PrecisionGate::OverlayPresent,
                        false,
                        false,
                        "no precise (T2) overlay attached",
                    )
                    .unwrap_err();
                    println!("{}", serde_json::to_string_pretty(&err)?);
                    eprintln!("# precise status=PRECISION_REQUIRED");
                }
            }
            PreciseCmd::Score { t1, oracle, t2 } => {
                let t1_edges: Vec<CallEdge> = serde_json::from_str(
                    &std::fs::read_to_string(&t1)
                        .with_context(|| format!("read {}", t1.display()))?,
                )?;
                let oracle_edges: Vec<CallEdge> = serde_json::from_str(
                    &std::fs::read_to_string(&oracle)
                        .with_context(|| format!("read {}", oracle.display()))?,
                )?;
                let index = load_precise_index(&t2)?;
                let t2_edges: Vec<CallEdge> = index
                    .edges
                    .iter()
                    .filter(|e| e.kind == "CALLS")
                    .map(|e| CallEdge {
                        src: e.src.clone(),
                        dst: e.dst.clone(),
                        file_path: e.file_path.clone(),
                        start_byte: e.span.as_ref().map(|s| s.start_byte),
                    })
                    .collect();
                let t1_score = score_call_resolution(&t1_edges, &oracle_edges);
                let t2_score = score_call_resolution(&t2_edges, &oracle_edges);
                let report = serde_json::json!({
                    "t1": t1_score,
                    "t2": t2_score,
                    "precision_delta": t2_score.precision - t1_score.precision,
                    "recall_delta": t2_score.recall - t1_score.recall,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                eprintln!(
                    "# precise score t1_p={:.2} t2_p={:.2} delta={:.2}",
                    t1_score.precision,
                    t2_score.precision,
                    t2_score.precision - t1_score.precision
                );
            }
            PreciseCmd::Ambiguity { path } => {
                let (_wm, kg) = open_kg(&path)?;
                let idx = ambiguity_index(&kg)?;
                println!("{}", serde_json::to_string_pretty(&idx)?);
                eprintln!(
                    "# ambiguity require_t2={} unresolved_rate={:.2} heuristic_rate={:.2}",
                    idx.require_t2, idx.unresolved_rate, idx.heuristic_rate
                );
            }
            PreciseCmd::RenameDryRun {
                symbol,
                new_name,
                workspace,
                allow_heuristic,
            } => {
                let (wm, kg) = open_kg(&workspace)?;
                match rename_dry_run(wm.root(), &kg, &symbol, &new_name, allow_heuristic)? {
                    Ok(report) => {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                        eprintln!(
                            "# rename-dry-run writes=false sites={} tier={}",
                            report.site_count, report.tier
                        );
                    }
                    Err(e) => {
                        println!("{}", serde_json::to_string_pretty(&e)?);
                        eprintln!("# rename-dry-run status=PRECISION_REQUIRED");
                    }
                }
            }
        },
        Commands::Semantic { cmd } => match cmd {
            SemanticCmd::Build { path, file } => {
                let wm = WorkspaceManager::open(&path)?;
                if let Some(rel) = file {
                    let art = build_file_artifact(wm.root(), &rel)?;
                    println!("{}", serde_json::to_string_pretty(&art)?);
                    eprintln!(
                        "# semantic build file={} functions={} notes={}",
                        art.path,
                        art.functions.len(),
                        art.notes.len()
                    );
                } else {
                    let manifest = build_workspace_python(wm.root())?;
                    println!("{}", serde_json::to_string_pretty(&manifest)?);
                    eprintln!(
                        "# semantic build files={} functions={}",
                        manifest.files, manifest.functions
                    );
                }
            }
            SemanticCmd::Slice {
                path,
                file,
                line,
                symbol,
                depth,
                direction,
                local,
            } => {
                let wm = WorkspaceManager::open(&path)?;
                if local {
                    let art = build_file_artifact(wm.root(), &file)?;
                    let criterion = match (line, symbol) {
                        (Some(l), None) => SliceCriterion::Line {
                            path: file.clone(),
                            line: l,
                        },
                        (None, Some(s)) => SliceCriterion::Symbol {
                            path: file.clone(),
                            symbol: s,
                        },
                        _ => bail!("provide exactly one of --line or --symbol"),
                    };
                    match local_slice(&art, &criterion) {
                        Ok(report) => {
                            println!("{}", serde_json::to_string_pretty(&report)?);
                            eprintln!(
                                "# semantic slice(local) function={} spans={}",
                                report.function,
                                report.spans.len()
                            );
                        }
                        Err(e) => {
                            println!("{}", serde_json::to_string_pretty(&e)?);
                            eprintln!("# semantic slice status=SEMANTIC_PARTIAL");
                        }
                    }
                } else {
                    let dir = match direction.as_str() {
                        "forward" => SliceDirection::Forward,
                        _ => SliceDirection::Backward,
                    };
                    let params = SliceParams {
                        direction: dir,
                        max_depth: depth,
                        max_functions: 16,
                        max_spans: 40,
                        residual_expand: true,
                        path: file,
                        line,
                        symbol,
                        snapshot_id: "adhoc".into(),
                    };
                    match interproc_slice(wm.root(), &params) {
                        Ok(report) => {
                            println!("{}", serde_json::to_string_pretty(&report)?);
                            eprintln!(
                                "# semantic slice depth={} spans={} memo={} truncated={}",
                                report.depth_reached,
                                report.spans.len(),
                                report.provenance.memo_hit,
                                report.truncated
                            );
                        }
                        Err(e) => {
                            println!("{}", serde_json::to_string_pretty(&e)?);
                            eprintln!("# semantic slice status=SEMANTIC_PARTIAL");
                        }
                    }
                }
            }
            SemanticCmd::ShardBuild {
                path,
                file,
                symbol,
                depth,
            } => {
                let wm = WorkspaceManager::open(&path)?;
                let _ = build_file_artifact(wm.root(), &file)?;
                let (shard, truncated) = ensure_shard(wm.root(), &file, &symbol, depth, 16)?;
                println!("{}", serde_json::to_string_pretty(&shard)?);
                eprintln!(
                    "# semantic shard-build id={} funcs={} truncated={}",
                    shard.shard_id,
                    shard.functions.len(),
                    truncated
                );
            }
            SemanticCmd::Status { path } => {
                let wm = WorkspaceManager::open(&path)?;
                if let Some(m) = read_semantic_manifest(wm.root())? {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                    eprintln!("# semantic status=ok algo={}", m.algo_version);
                } else {
                    println!(
                        "{}",
                        serde_json::json!({
                            "code": "SEMANTIC_PARTIAL",
                            "message": "no semantic artifacts — run `prism semantic build`",
                        })
                    );
                    eprintln!("# semantic status=missing");
                }
            }
        },
    }
    Ok(())
}
