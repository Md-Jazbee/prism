import * as vscode from "vscode";
import {
  resolvePrismBinary,
  tryBuildWorkspaceBinary,
  downloadOffer,
  extensionManifestPath,
  ensureDaemon,
} from "./binary";
import { registerMcp } from "../agent/assets";

export interface SetupResult {
  ok: boolean;
  prismPath: string;
  detail: string[];
}

/**
 * Graphify-like one-shot: binary → prism setup (index+assets+MCP) → optional daemon.
 */
export async function runWorkspaceSetup(
  context: vscode.ExtensionContext,
  workspaceRoot: string,
  output: vscode.OutputChannel,
  opts: { forceRebuild?: boolean } = {},
): Promise<SetupResult> {
  const detail: string[] = [];
  const cfg = vscode.workspace.getConfiguration("prism");

  let binary = resolvePrismBinary(workspaceRoot, cfg.get<string>("binaryPath") ?? "");
  if (!binary || opts.forceRebuild) {
    output.appendLine("Prism binary missing — attempting cargo build -p prism-cli…");
    const built = await tryBuildWorkspaceBinary(workspaceRoot);
    if (built) {
      binary = built;
      detail.push(`built ${built.prismPath}`);
    }
  }
  if (!binary) {
    const offer = downloadOffer(
      cfg.get<string>("downloadBaseUrl") ?? "",
      extensionManifestPath(context),
    );
    throw new Error(
      `prism binary not found. ${offer.reason}`,
    );
  }
  detail.push(`binary=${binary.prismPath} (${binary.source})`);

  // Full setup via CLI (single source of truth with `prism setup`).
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const execFileAsync = promisify(execFile);
  try {
    const { stdout } = await execFileAsync(
      binary.prismPath,
      ["setup", workspaceRoot, "--json"],
      { cwd: workspaceRoot, maxBuffer: 32 * 1024 * 1024, env: process.env },
    );
    detail.push(`setup: ${stdout.trim().slice(0, 500)}`);
    output.appendLine(stdout);
  } catch (e) {
    // Fallback: register MCP + generate assets via extension helpers if setup fails mid-way
    output.appendLine(`prism setup failed: ${e}`);
    const msg = await registerMcp(workspaceRoot, binary.prismPath, true);
    detail.push(msg);
    throw e;
  }

  // Start / attach daemon with shared token file.
  if (cfg.get<boolean>("preferDaemon") !== false) {
    try {
      const bind = cfg.get<string>("daemonBind") ?? "127.0.0.1:7420";
      const handle = await ensureDaemon(binary.prismPath, workspaceRoot, bind);
      detail.push(`daemon ${handle.mode} @ ${handle.bindAddr}`);
    } catch (e) {
      detail.push(`daemon skipped: ${e}`);
      output.appendLine(`daemon ensure: ${e}`);
    }
  }

  return { ok: true, prismPath: binary.prismPath, detail };
}
