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
  resolveSymbol(name: string, limit?: number): Promise<Array<{ id: string; name?: string; path?: string }>>;
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
  setup(opts?: { skipIndex?: boolean }): Promise<unknown>;
  doctorReady(): Promise<unknown>;
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
  try {
    const { stdout } = await execFileAsync(prismPath, args, {
      cwd,
      maxBuffer: 16 * 1024 * 1024,
      env: process.env,
    });
    const lines = stdout.trim().split(/\r?\n/);
    const start = lines.findIndex((l) => l.startsWith("{") || l.startsWith("["));
    const blob = start >= 0 ? lines.slice(start).join("\n") : stdout;
    return JSON.parse(blob);
  } catch (e: unknown) {
    const err = e as { stdout?: string; stderr?: string; message?: string };
    const text = err.stdout || err.stderr || err.message || String(e);
    try {
      const start = text.indexOf("{");
      if (start >= 0) {
        return JSON.parse(text.slice(start));
      }
    } catch {
      /* fall through */
    }
    throw new Error(text.slice(0, 400));
  }
}

function unwrapCompile(raw: unknown): EvidencePack {
  const r = raw as {
    status?: string;
    data?: EvidencePack & { code?: string; message?: string; hint?: string };
    pack?: EvidencePack;
    meta?: EvidencePack["meta"];
  };
  if (r.status && r.status !== "ok") {
    const d = r.data as { code?: string; message?: string; hint?: string } | undefined;
    throw new PrismApiError({
      code: d?.code ?? r.status.toUpperCase(),
      message: d?.message ?? `compile ${r.status}`,
      hint: d?.hint,
    });
  }
  if (r.data && (r.data as EvidencePack).meta) {
    return r.data as EvidencePack;
  }
  if (r.pack) return r.pack;
  if (r.meta) return r as unknown as EvidencePack;
  throw new Error("Unexpected compile response shape");
}

function unwrapView(raw: unknown): GraphViewPayload {
  const r = raw as {
    view?: GraphViewPayload;
    code?: string;
    message?: string;
    hint?: string;
    nodes?: unknown[];
  };
  if (r.code === "VIEW_TOO_LARGE") {
    throw new PrismApiError({
      code: "VIEW_TOO_LARGE",
      message: r.message ?? "view too large",
      hint: r.hint,
    });
  }
  if (r.view) return r.view;
  if (Array.isArray(r.nodes)) return r as GraphViewPayload;
  throw new Error("Unexpected view response shape");
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
      const raw = await daemonFetch(cfg, "POST", "/v1/context/compile", req);
      const wrapped = raw as { pack?: EvidencePack } & EvidencePack;
      return (wrapped.pack ?? wrapped) as EvidencePack;
    },
    async resolveSymbol(name, limit = 20) {
      const raw = (await daemonFetch(
        cfg,
        "GET",
        `/v1/symbols?name=${encodeURIComponent(name)}&limit=${limit}`,
      )) as { symbols?: Array<{ id: string; name?: string; path?: string }> };
      return raw.symbols ?? (raw as unknown as Array<{ id: string }>);
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
      const raw = await daemonFetch(cfg, "POST", "/v1/view", body);
      return unwrapView(raw);
    },
    async setup() {
      // Daemon has no setup route — use CLI for orchestration.
      return createCliTransport(cfg, "").setup();
    },
    async doctorReady() {
      return createCliTransport(cfg, "").doctorReady();
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
    indexStatus: () => cliJson(bin, ["index-status", root, "--json"], root),
    buildIndex: () => cliJson(bin, ["index", root, "--json"], root),
    async compile(req) {
      const args = ["compile", req.question, root];
      if (req.intent) args.push("--intent", req.intent);
      if (req.budget_tokens) args.push("--budget", String(req.budget_tokens));
      for (const a of req.anchors ?? []) args.push("--anchor", a);
      for (const s of req.stack_frames ?? []) args.push("--stack", s);
      if (req.error_text) args.push("--error", req.error_text);
      for (const p of req.changed_paths ?? []) args.push("--changed", p);
      const raw = await cliJson(bin, args, root);
      return unwrapCompile(raw);
    },
    async resolveSymbol(name, limit = 20) {
      const raw = await cliJson(
        bin,
        ["query", "resolve", name, root, "--limit", String(limit)],
        root,
      );
      return Array.isArray(raw) ? raw : [];
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
      return unwrapView(raw);
    },
    setup: (opts) => {
      const args = ["setup", root, "--json"];
      if (opts?.skipIndex) args.push("--skip-index");
      return cliJson(bin, args, root);
    },
    doctorReady: () => cliJson(bin, ["doctor", root, "--ready", "--json"], root),
  };
}

/**
 * Prefer daemon HTTP after an *authorized* probe; otherwise CLI.
 */
export async function connectTransport(cfg: TransportConfig): Promise<PrismTransport> {
  if (!cfg.preferDaemon || !cfg.token) {
    return createCliTransport(
      cfg,
      cfg.preferDaemon
        ? "No daemon token — using CLI."
        : "Daemon preference disabled — using CLI.",
    );
  }
  try {
    const daemon = createDaemonTransport(cfg);
    const health = await daemon.health();
    daemon.assertVersionOk(health);
    // Authorized probe — /health alone is insufficient (caused sticky 401s).
    await daemonFetch(cfg, "GET", "/v1/index/status");
    return daemon;
  } catch (e) {
    if (e instanceof PrismApiError && e.code === "UNAUTHORIZED") {
      return createCliTransport(
        cfg,
        "Daemon token rejected (UNAUTHORIZED) — using CLI fallback.",
      );
    }
    const msg = e instanceof Error ? e.message : String(e);
    return createCliTransport(
      cfg,
      `Daemon unavailable (${msg}) — using CLI fallback.`,
    );
  }
}

export const __test = {
  majorOf,
  createCliTransport,
  createDaemonTransport,
  unwrapCompile,
  unwrapView,
};
