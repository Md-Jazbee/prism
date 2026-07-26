import * as fs from "node:fs";
import * as path from "node:path";
import { execFileSync, spawn } from "node:child_process";
import type { ExtensionContext } from "vscode";

export interface BinaryResolution {
  prismPath: string;
  source: "setting" | "path" | "workspace" | "download";
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
  const names =
    process.platform === "win32"
      ? ["prism.exe", "prismd.exe"]
      : ["prism", "prismd"];
  const out: string[] = [];
  for (const profile of ["release", "debug"]) {
    for (const name of names) {
      if (name.startsWith("prismd")) continue;
      out.push(path.join(base, profile, name));
    }
  }
  return out;
}

/**
 * Resolve `prism` binary: setting → PATH → workspace target → download placeholder.
 */
export function resolvePrismBinary(
  workspaceRoot: string | undefined,
  binaryPathSetting: string,
): BinaryResolution | undefined {
  if (binaryPathSetting && existsExecutable(binaryPathSetting)) {
    return { prismPath: binaryPathSetting, source: "setting" };
  }
  const onPath = which("prism");
  if (onPath && existsExecutable(onPath)) {
    return { prismPath: onPath, source: "path" };
  }
  if (workspaceRoot) {
    for (const cand of workspaceCandidates(workspaceRoot)) {
      if (existsExecutable(cand)) {
        return { prismPath: cand, source: "workspace" };
      }
    }
  }
  return undefined;
}

export interface DownloadOffer {
  enabled: boolean;
  reason: string;
}

/** Download-on-demand is gated on a configured base URL + manifest entry. */
export function downloadOffer(
  downloadBaseUrl: string,
  manifestPath: string,
): DownloadOffer {
  if (!downloadBaseUrl) {
    return {
      enabled: false,
      reason: "prism.downloadBaseUrl is empty; use PATH or build the workspace binary.",
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
        reason: `No manifest entry for ${key}; populate binaries/manifest.json for release.`,
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

function readLockfile(workspaceRoot: string): { pid: number; bind: string } | undefined {
  const lock = path.join(workspaceRoot, ".prism", "daemon.lock");
  if (!fs.existsSync(lock)) return undefined;
  const text = fs.readFileSync(lock, "utf8").trim().split(/\r?\n/);
  const pid = Number(text[0]);
  const bind = text[1];
  if (!Number.isFinite(pid) || !bind) return undefined;
  return { pid, bind };
}

function readTokenFile(workspaceRoot: string): string | undefined {
  const p = path.join(workspaceRoot, ".prism", "daemon.token");
  if (fs.existsSync(p)) {
    return fs.readFileSync(p, "utf8").trim() || undefined;
  }
  return undefined;
}

/**
 * Attach to an existing daemon via lockfile, or spawn `prism daemon`.
 * Token is read from `.prism/daemon.token` when present, else generated and written.
 */
export async function ensureDaemon(
  prismPath: string,
  workspaceRoot: string,
  bindAddr: string,
  existingToken?: string,
): Promise<DaemonHandle> {
  const lock = readLockfile(workspaceRoot);
  const token =
    existingToken ||
    readTokenFile(workspaceRoot) ||
    `prism-local-ext-${Date.now().toString(16)}`;

  const tokenPath = path.join(workspaceRoot, ".prism", "daemon.token");
  fs.mkdirSync(path.dirname(tokenPath), { recursive: true });
  if (!fs.existsSync(tokenPath)) {
    fs.writeFileSync(tokenPath, token, { mode: 0o600 });
  }
  const finalToken = fs.readFileSync(tokenPath, "utf8").trim();

  if (lock) {
    try {
      process.kill(lock.pid, 0);
      return {
        pid: lock.pid,
        bindAddr: lock.bind,
        token: finalToken,
        mode: "attached",
      };
    } catch {
      /* stale lock — spawn */
    }
  }

  const child = spawn(
    prismPath,
    ["daemon", workspaceRoot, "--bind", bindAddr, "--token", finalToken],
    {
      detached: true,
      stdio: "ignore",
      env: { ...process.env, PRISM_TOKEN: finalToken },
    },
  );
  child.unref();

  // Wait briefly for lock / health
  for (let i = 0; i < 30; i++) {
    await sleep(100);
    const again = readLockfile(workspaceRoot);
    if (again) {
      return {
        pid: again.pid,
        bindAddr: again.bind,
        token: finalToken,
        mode: "spawned",
      };
    }
  }

  return {
    pid: child.pid,
    bindAddr,
    token: finalToken,
    mode: "spawned",
  };
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function extensionManifestPath(context: ExtensionContext): string {
  return path.join(context.extensionPath, "binaries", "manifest.json");
}
