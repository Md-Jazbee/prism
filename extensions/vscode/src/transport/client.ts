import { execFile } from "node:child_process";
import { promisify } from "node:util";
import {
  CompileRequest,
  EvidencePack,
  GraphViewPayload,
  HealthInfo,
  PrismApiError,
  TransportMode,
  ApiErrorBody,
} from "../types";

const execFileAsync = promisify(execFile);

function majorOf(apiVersion: string): number {
  const m = /^(\d+)/.exec(apiVersion);
  return m ? Number(m[1]) : 0;
}

export interface TransportConfig {
  workspaceRoot: string;
  prismPath: string;
  baseUrl: string;
  token: string;
  preferDaemon: boolean;
  engineMajor: number;
}

export interface PrismTransport {
  mode: TransportMode;
  degradationNote?: string;
  health(): Promise<HealthInfo>;
  indexStatus(): Promise<unknown>;
  buildIndex(paths?: string[]): Promise<unknown>;
  compile(req: CompileRequest): Promise<EvidencePack>;
  impact(id: string, opts?: { depth?: number; require_precise?: boolean }): Promise<unknown>;
  slice(opts: {
    path: string;
    line?: number;
    symbol?: string;
    max_depth?: number;
    direction?: string;
  }): Promise<unknown>;
  repoMap(hubLimit?: number): Promise<unknown>;
  entrypoints(limit?: number): Promise<unknown>;
  view(body: Record<string, unknown>): Promise<GraphViewPayload>;
  assertVersionOk(health: HealthInfo): void;
}

async function daemonFetch(
  cfg: TransportConfig,
  method: string,
  path: string,
  body?: unknown,
  signal?: AbortSignal,
): Promise<unknown> {
  const url = `${cfg.baseUrl.replace(/\/$/, "")}${path}`;
  const res = await fetch(url, {
    method,
    headers: {
      Authorization: `Bearer ${cfg.token}`,
      "Content-Type": "application/json",
      "X-Prism-Token": cfg.token,
    },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  const text = await res.text();
  let json: unknown = undefined;
  try {
    json = text ? JSON.parse(text) : undefined;
  } catch {
    throw new Error(`Non-JSON response from ${path}: ${text.slice(0, 200)}`);
  }
  if (!res.ok) {
    const err = (json as ApiErrorBody)?.error;
    if (err?.code) throw new PrismApiError(err);
    throw new Error(`HTTP ${res.status} ${path}: ${text.slice(0, 200)}`);
  }
  return json;
}

async function cliJson(
  prismPath: string,
  args: string[],
  cwd: string,
): Promise<unknown> {
  const { stdout } = await execFileAsync(prismPath, args, {
    cwd,
    maxBuffer: 16 * 1024 * 1024,
    env: process.env,
  });
  const lines = stdout.trim().split(/\r?\n/);
  // CLI may print pretty JSON; take from first `{` or `[`
  const start = lines.findIndex((l) => l.startsWith("{") || l.startsWith("["));
  const blob = start >= 0 ? lines.slice(start).join("\n") : stdout;
  return JSON.parse(blob);
}

function createDaemonTransport(cfg: TransportConfig): PrismTransport {
  return {
    mode: "daemon",
    async health() {
      const url = `${cfg.baseUrl.replace(/\/$/, "")}/health`;
      const res = await fetch(url);
      return (await res.json()) as HealthInfo;
    },
    assertVersionOk(health: HealthInfo) {
      const maj = majorOf(health.api_version);
      if (maj !== cfg.engineMajor) {
        throw new PrismApiError({
          code: "VERSION_SKEW",
          message: `API major ${maj} ≠ extension engineMajor ${cfg.engineMajor}`,
          hint: "Upgrade the extension or the prism binary so majors match.",
        });
      }
    },
    indexStatus: () => daemonFetch(cfg, "GET", "/v1/index/status"),
    buildIndex: (paths = []) => daemonFetch(cfg, "POST", "/v1/index", { paths }),
    async compile(req) {
      const raw = (await daemonFetch(cfg, "POST", "/v1/context/compile", req)) as {
        pack?: EvidencePack;
      } & EvidencePack;
      return (raw.pack ?? raw) as EvidencePack;
    },
    impact: (id, opts) =>
      daemonFetch(cfg, "POST", "/v1/impact", {
        id,
        depth: opts?.depth ?? 2,
        limit: 100,
        require_precise: opts?.require_precise ?? false,
      }),
    slice: (opts) => daemonFetch(cfg, "POST", "/v1/slice", opts),
    repoMap: (hubLimit = 15) =>
      daemonFetch(cfg, "GET", `/v1/repo/map?hub_limit=${hubLimit}`),
    entrypoints: (limit = 40) =>
      daemonFetch(cfg, "GET", `/v1/intel/entrypoints?limit=${limit}`),
    async view(body) {
      const raw = (await daemonFetch(cfg, "POST", "/v1/view", body)) as {
        view?: GraphViewPayload;
      } & GraphViewPayload;
      return (raw.view ?? raw) as GraphViewPayload;
    },
  };
}

function createCliTransport(cfg: TransportConfig, note: string): PrismTransport {
  const root = cfg.workspaceRoot;
  const bin = cfg.prismPath;
  return {
    mode: "cli",
    degradationNote: note,
    async health() {
      return {
        ok: true,
        service: "prism-cli",
        api_version: `${cfg.engineMajor}.0.0`,
        snapshot_id: "cli",
        workspace: root,
      };
    },
    assertVersionOk() {
      /* CLI path: trust local binary */
    },
    indexStatus: () => cliJson(bin, ["index-status", root], root),
    buildIndex: () => cliJson(bin, ["index", root], root),
    async compile(req) {
      const args = ["compile", req.question, root];
      if (req.intent) args.push("--intent", req.intent);
      if (req.budget_tokens) args.push("--budget", String(req.budget_tokens));
      for (const a of req.anchors ?? []) args.push("--anchor", a);
      for (const s of req.stack_frames ?? []) args.push("--stack", s);
      if (req.error_text) args.push("--error", req.error_text);
      for (const p of req.changed_paths ?? []) args.push("--changed", p);
      const raw = await cliJson(bin, args, root);
      const wrapped = raw as { data?: EvidencePack; pack?: EvidencePack } & EvidencePack;
      return (wrapped.data ?? wrapped.pack ?? wrapped) as EvidencePack;
    },
    impact: (id, opts) => {
      const args = ["query", "impact", id, root, "--depth", String(opts?.depth ?? 2)];
      if (opts?.require_precise) args.push("--require-precise");
      return cliJson(bin, args, root);
    },
    slice: (opts) => {
      const args = ["semantic", "slice", root, "--file", opts.path];
      if (opts.line != null) args.push("--line", String(opts.line));
      if (opts.symbol) args.push("--symbol", opts.symbol);
      if (opts.max_depth != null) args.push("--depth", String(opts.max_depth));
      if (opts.direction) args.push("--direction", opts.direction);
      return cliJson(bin, args, root);
    },
    repoMap: (hubLimit = 15) =>
      cliJson(bin, ["query", "repo-map", root, "--hub-limit", String(hubLimit)], root),
    entrypoints: () => cliJson(bin, ["query", "entrypoints", root], root),
    async view(body) {
      const kind = String(body.view_kind ?? "architecture_map");
      const args = ["view", kind, root];
      if (body.seed_id) args.push("--seed", String(body.seed_id));
      if (body.question) args.push("--question", String(body.question));
      for (const a of (body.anchors as string[]) ?? []) args.push("--anchor", a);
      const raw = await cliJson(bin, args, root);
      const wrapped = raw as { view?: GraphViewPayload } & GraphViewPayload;
      return (wrapped.view ?? wrapped) as GraphViewPayload;
    },
  };
}

/**
 * Prefer daemon HTTP; on failure return CLI transport with degradation note.
 */
export async function connectTransport(cfg: TransportConfig): Promise<PrismTransport> {
  if (!cfg.preferDaemon) {
    return createCliTransport(cfg, "Daemon preference disabled — using CLI.");
  }
  try {
    const daemon = createDaemonTransport(cfg);
    const health = await daemon.health();
    daemon.assertVersionOk(health);
    return daemon;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return createCliTransport(
      cfg,
      `Daemon unavailable (${msg}) — using CLI fallback.`,
    );
  }
}

/** Pure helpers for unit tests. */
export const __test = { majorOf, createCliTransport, createDaemonTransport };
