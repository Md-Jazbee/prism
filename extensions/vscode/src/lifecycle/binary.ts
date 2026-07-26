import * as fs from "node:fs";
import * as path from "node:path";
import { execFile, execFileSync, spawn } from "node:child_process";
import { promisify } from "node:util";
import type { ExtensionContext } from "vscode";

const execFileAsync = promisify(execFile);

export interface BinaryResolution {
  prismPath: string;
  source: "setting" | "path" | "workspace" | "download" | "built";
}

function existsExecutable(p: string): boolean {
  try {
    fs.accessSync(p, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function which(cmd: string): string | undefined {
  try {
    const out = execFileSync(process.platform === "win32" ? "where" : "which", [cmd], {
      encoding: "utf8",
    })
      .trim()
      .split(/\r?\n/)[0];
    return out || undefined;
  } catch {
    return undefined;
  }
}

function workspaceCandidates(workspaceRoot: string): string[] {
  const base = path.join(workspaceRoot, "target");
  const names = process.platform === "win32" ? ["prism.exe"] : ["prism"];
  const out: string[] = [];
  for (const profile of ["release", "debug"]) {
    for (const name of names) {
      out.push(path.join(base, profile, name));
    }
  }
  return out;
}

function isPrismWorkspace(workspaceRoot: string): boolean {
  return (
    fs.existsSync(path.join(workspaceRoot, "Cargo.toml")) &&
    fs.existsSync(path.join(workspaceRoot, "crates", "prism-cli", "Cargo.toml"))
  );
}

/**
 * Resolve `prism` binary.
 * Prefer workspace target when developing this repo (avoids stale PATH binary).
 * Order: setting → workspace target → PATH → (caller may build).
 */
export function resolvePrismBinary(
  workspaceRoot: string | undefined,
  binaryPathSetting: string,
): BinaryResolution | undefined {
  if (binaryPathSetting && existsExecutable(binaryPathSetting)) {
    return { prismPath: binaryPathSetting, source: "setting" };
  }
  if (workspaceRoot) {
    for (const cand of workspaceCandidates(workspaceRoot)) {
      if (existsExecutable(cand)) {
        return { prismPath: cand, source: "workspace" };
      }
    }
  }
  const onPath = which("prism");
  if (onPath && existsExecutable(onPath)) {
    return { prismPath: onPath, source: "path" };
  }
  return undefined;
}

/** Try `cargo build -p prism-cli` when this is the Prism source tree. */
export async function tryBuildWorkspaceBinary(
  workspaceRoot: string,
): Promise<BinaryResolution | undefined> {
  if (!isPrismWorkspace(workspaceRoot)) return undefined;
  const cargo = which("cargo");
  if (!cargo) return undefined;
  await execFileAsync(cargo, ["build", "-p", "prism-cli"], {
    cwd: workspaceRoot,
    maxBuffer: 32 * 1024 * 1024,
    env: process.env,
  });
  for (const cand of workspaceCandidates(workspaceRoot)) {
    if (existsExecutable(cand)) {
      return { prismPath: cand, source: "built" };
    }
  }
  return undefined;
}

export interface DownloadOffer {
  enabled: boolean;
  reason: string;
}

export function downloadOffer(
  downloadBaseUrl: string,
  manifestPath: string,
): DownloadOffer {
  if (!downloadBaseUrl) {
    return {
      enabled: false,
      reason:
        "No download URL configured. Use PATH prism, set prism.binaryPath, or open the Prism source tree (cargo build -p prism-cli).",
    };
  }
  if (!fs.existsSync(manifestPath)) {
    return { enabled: false, reason: "Binary manifest missing." };
  }
  try {
    const raw = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as {
      binaries?: Record<string, unknown>;
    };
    const key = `${process.platform}-${process.arch}`;
    if (!raw.binaries || !raw.binaries[key]) {
      return {
        enabled: false,
        reason: `No manifest entry for ${key}.`,
      };
    }
    return { enabled: true, reason: `Download available for ${key}` };
  } catch (e) {
    return { enabled: false, reason: `Manifest unreadable: ${e}` };
  }
}

export interface DaemonHandle {
  pid?: number;
  bindAddr: string;
  token: string;
  mode: "attached" | "spawned";
}

export function readLockfile(
  workspaceRoot: string,
): { pid: number; bind: string } | undefined {
  const lock = path.join(workspaceRoot, ".prism", "daemon.lock");
  if (!fs.existsSync(lock)) return undefined;
  const text = fs.readFileSync(lock, "utf8").trim().split(/\r?\n/);
  const pid = Number(text[0]);
  const bind = text[1];
  if (!Number.isFinite(pid) || !bind) return undefined;
  return { pid, bind };
}

export function readTokenFile(workspaceRoot: string): string | undefined {
  const p = path.join(workspaceRoot, ".prism", "daemon.token");
  if (fs.existsSync(p)) {
    return fs.readFileSync(p, "utf8").trim() || undefined;
  }
  return undefined;
}

function writeTokenFile(workspaceRoot: string, token: string): void {
  const tokenPath = path.join(workspaceRoot, ".prism", "daemon.token");
  fs.mkdirSync(path.dirname(tokenPath), { recursive: true });
  fs.writeFileSync(tokenPath, token, { mode: 0o600 });
}

async function waitForHealth(
  bindAddr: string,
  timeoutMs = 15000,
): Promise<boolean> {
  const url = `http://${bindAddr.replace(/^https?:\/\//, "")}/health`;
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return true;
    } catch {
      /* retry */
    }
    await sleep(200);
  }
  return false;
}

async function probeAuthorized(bindAddr: string, token: string): Promise<boolean> {
  const base = `http://${bindAddr.replace(/^https?:\/\//, "")}`;
  try {
    const res = await fetch(`${base}/v1/index/status`, {
      headers: {
        Authorization: `Bearer ${token}`,
        "X-Prism-Token": token,
      },
    });
    return res.ok || res.status === 503; // 503 = auth ok, index missing
  } catch {
    return false;
  }
}

/**
 * Attach to an existing daemon (token from `.prism/daemon.token` only) or spawn one.
 * Never invents a token against a live foreign daemon — that caused sticky 401s.
 */
export async function ensureDaemon(
  prismPath: string,
  workspaceRoot: string,
  bindAddr: string,
  existingToken?: string,
): Promise<DaemonHandle> {
  const lock = readLockfile(workspaceRoot);
  const diskToken = readTokenFile(workspaceRoot);
  const envToken = process.env.PRISM_TOKEN;

  if (lock) {
    try {
      process.kill(lock.pid, 0);
      const healthy = await waitForHealth(lock.bind, 3000);
      if (!healthy) {
        throw new Error("lock present but /health not responding");
      }
      const candidates = [existingToken, diskToken, envToken].filter(
        (t): t is string => !!t && t.length > 0,
      );
      for (const tok of candidates) {
        if (await probeAuthorized(lock.bind, tok)) {
          writeTokenFile(workspaceRoot, tok);
          return {
            pid: lock.pid,
            bindAddr: lock.bind,
            token: tok,
            mode: "attached",
          };
        }
      }
      // Live daemon we cannot auth — do not invent a token; let caller fall back to CLI.
      throw new Error(
        "Daemon running but token mismatch (check .prism/daemon.token or restart with prism setup).",
      );
    } catch (e) {
      if (String(e).includes("token mismatch")) throw e;
      /* stale lock — spawn below */
    }
  }

  const token =
    existingToken ||
    diskToken ||
    envToken ||
    `prism-local-ext-${Date.now().toString(16)}`;
  writeTokenFile(workspaceRoot, token);

  const child = spawn(
    prismPath,
    ["daemon", workspaceRoot, "--bind", bindAddr, "--token", token],
    {
      detached: true,
      stdio: "ignore",
      env: { ...process.env, PRISM_TOKEN: token },
    },
  );
  child.unref();

  const ok = await waitForHealth(bindAddr, 20000);
  if (!ok) {
    throw new Error(`Daemon spawned but /health not ready within timeout (${bindAddr})`);
  }
  if (!(await probeAuthorized(bindAddr, token))) {
    throw new Error("Daemon healthy but authorized probe failed");
  }

  const again = readLockfile(workspaceRoot);
  return {
    pid: again?.pid ?? child.pid,
    bindAddr: again?.bind ?? bindAddr,
    token,
    mode: "spawned",
  };
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function extensionManifestPath(context: ExtensionContext): string {
  return path.join(context.extensionPath, "binaries", "manifest.json");
}
