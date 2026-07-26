import * as fs from "node:fs";
import * as path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const MCP_SERVER_KEY = "prism";

/**
 * Cursor / VS Code MCP auto-registration (file-based).
 */
export async function registerMcp(
  workspaceRoot: string,
  prismPath: string,
  enable: boolean,
): Promise<string> {
  const cursorDir = path.join(workspaceRoot, ".cursor");
  const cursorMcp = path.join(cursorDir, "mcp.json");
  const vscodeMcp = path.join(workspaceRoot, ".vscode", "mcp.json");
  const target = fs.existsSync(cursorDir) ? cursorMcp : vscodeMcp;

  if (!enable) {
    if (fs.existsSync(target)) {
      const raw = JSON.parse(fs.readFileSync(target, "utf8")) as {
        mcpServers?: Record<string, unknown>;
      };
      if (raw.mcpServers?.[MCP_SERVER_KEY]) {
        delete raw.mcpServers[MCP_SERVER_KEY];
        fs.writeFileSync(target, JSON.stringify(raw, null, 2) + "\n");
      }
    }
    return `Disabled MCP registration (${target})`;
  }

  fs.mkdirSync(path.dirname(target), { recursive: true });
  let raw: { mcpServers?: Record<string, unknown> } = { mcpServers: {} };
  if (fs.existsSync(target)) {
    try {
      raw = JSON.parse(fs.readFileSync(target, "utf8"));
    } catch {
      raw = { mcpServers: {} };
    }
  }
  raw.mcpServers = raw.mcpServers ?? {};
  raw.mcpServers[MCP_SERVER_KEY] = {
    command: prismPath,
    args: ["mcp", workspaceRoot],
  };
  fs.writeFileSync(target, JSON.stringify(raw, null, 2) + "\n");
  return `Registered Prism MCP at ${target}`;
}

/** Prefer CLI catalog generator; fall back to minimal template. */
export async function generateAgentsMd(
  workspaceRoot: string,
  prismPath?: string,
): Promise<string> {
  if (prismPath) {
    try {
      await execFileAsync(prismPath, ["agent", "generate-assets", workspaceRoot], {
        cwd: workspaceRoot,
        maxBuffer: 4 * 1024 * 1024,
      });
      return path.join(workspaceRoot, "AGENTS.md");
    } catch {
      /* fall through to template */
    }
  }
  const out = path.join(workspaceRoot, "AGENTS.md");
  const body = `# AGENTS.md — Prism guidance

> Prefer \`compile_context\` (or \`prism workflow run\`) before explore loops.
> Regenerate with: \`prism agent generate-assets\`

## Primary path

1. Call **compile_context** or **Prism: Compile Context**.
2. Answer from Evidence Pack citations.
3. Use micro-tools only for targeted follow-ups.

## Refusals

| Code | Action |
|---|---|
| SCOPE_UNRESOLVED | Pick Anchor |
| INDEX_UNAVAILABLE | Prism: Setup Workspace |
| PRECISION_REQUIRED | Import PreciseIndex / SCIP |
`;
  fs.writeFileSync(
    out,
    `<!-- prism:generated fallback — prefer \`prism agent generate-assets\` -->\n${body}`,
  );
  return out;
}

export function redactPackForLlm(pack: unknown): string {
  const p = pack as {
    meta?: unknown;
    citations?: unknown;
    fragments?: Array<{
      id: string;
      kind: string;
      layer: string;
      text: string;
      why_included?: string;
    }>;
    gaps?: unknown;
  };
  const frags = (p.fragments ?? []).map((f) => ({
    id: f.id,
    kind: f.kind,
    layer: f.layer,
    why_included: f.why_included,
    text:
      f.text.length > 800
        ? f.text.slice(0, 800) + "\n…[redacted for length]"
        : f.text,
  }));
  return JSON.stringify(
    { meta: p.meta, citations: p.citations, fragments: frags, gaps: p.gaps },
    null,
    2,
  );
}
