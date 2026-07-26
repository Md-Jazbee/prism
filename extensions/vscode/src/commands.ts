import * as vscode from "vscode";
import { PrismSession } from "./session";
import { PrismTransport, connectTransport, TransportConfig } from "./transport/client";
import {
  resolvePrismBinary,
  ensureDaemon,
  downloadOffer,
  extensionManifestPath,
  tryBuildWorkspaceBinary,
} from "./lifecycle/binary";
import { runWorkspaceSetup } from "./lifecycle/setup";
import { EvidencePanelProvider, GraphPanelProvider } from "./panels/providers";
import { createDecorations } from "./decorations/gutter";
import {
  registerMcp,
  generateAgentsMd,
  handleRefusal,
  redactPackForLlm,
} from "./agent/integration";
import { PrismApiError } from "./types";

export interface PrismHost {
  session: PrismSession;
  getTransport(): Promise<PrismTransport>;
  resetTransport(): void;
  evidence: EvidencePanelProvider;
  graph: GraphPanelProvider;
  decorations: ReturnType<typeof createDecorations>;
  status: vscode.StatusBarItem;
  output: vscode.OutputChannel;
  workspaceRoot(): string | undefined;
  recordUsage(command: string, refusal?: string): void;
  context: vscode.ExtensionContext;
}

function cfg(): vscode.WorkspaceConfiguration {
  return vscode.workspace.getConfiguration("prism");
}

export function createHost(
  context: vscode.ExtensionContext,
  session: PrismSession,
  evidence: EvidencePanelProvider,
  graph: GraphPanelProvider,
): PrismHost {
  const output = vscode.window.createOutputChannel("Prism");
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
  status.text = "$(graph) Prism";
  status.tooltip = "Prism — idle";
  status.command = "prism.setupWorkspace";
  status.show();

  const decorations = createDecorations(
    () => cfg().get<boolean>("decorations.enabled") === true,
  );

  let transportPromise: Promise<PrismTransport> | undefined;

  const workspaceRoot = () =>
    vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

  const resetTransport = () => {
    transportPromise = undefined;
  };

  const recordUsage = (command: string, refusal?: string) => {
    if (!cfg().get<boolean>("usageCounters")) return;
    const key = "prism.usage";
    const prev = context.globalState.get<Record<string, number>>(key) ?? {};
    const k = refusal ? `refusal:${refusal}` : `cmd:${command}`;
    prev[k] = (prev[k] ?? 0) + 1;
    void context.globalState.update(key, prev);
  };

  const getTransport = async (): Promise<PrismTransport> => {
    if (transportPromise) return transportPromise;
    transportPromise = (async () => {
      const root = workspaceRoot();
      if (!root) throw new Error("Open a workspace folder to use Prism.");
      let binary = resolvePrismBinary(root, cfg().get<string>("binaryPath") ?? "");
      if (!binary) {
        binary = await tryBuildWorkspaceBinary(root);
      }
      if (!binary) {
        const offer = downloadOffer(
          cfg().get<string>("downloadBaseUrl") ?? "",
          extensionManifestPath(context),
        );
        throw new Error(
          `prism binary not found. ${offer.reason} Or run Prism: Setup Workspace.`,
        );
      }
      const bind = cfg().get<string>("daemonBind") ?? "127.0.0.1:7420";
      let token = "";
      let baseUrl = `http://${bind}`;
      let preferDaemon = cfg().get<boolean>("preferDaemon") !== false;
      if (preferDaemon) {
        try {
          const handle = await ensureDaemon(binary.prismPath, root, bind);
          token = handle.token;
          baseUrl = `http://${handle.bindAddr}`;
        } catch (e) {
          output.appendLine(`daemon ensure failed: ${e}`);
          preferDaemon = false;
        }
      }
      const tcfg: TransportConfig = {
        workspaceRoot: root,
        prismPath: binary.prismPath,
        baseUrl,
        token,
        preferDaemon,
        engineMajor: cfg().get<number>("engineMajor") ?? 0,
      };
      const t = await connectTransport(tcfg);
      session.setTransport(t.mode, t.degradationNote);
      status.text =
        t.mode === "daemon" ? "$(graph) Prism · daemon" : "$(graph) Prism · CLI";
      status.tooltip = t.degradationNote ?? `Transport: ${t.mode} · click to Setup`;
      if (t.degradationNote) output.appendLine(t.degradationNote);
      return t;
    })();
    try {
      return await transportPromise;
    } catch (e) {
      transportPromise = undefined;
      throw e;
    }
  };

  return {
    session,
    getTransport,
    resetTransport,
    evidence,
    graph,
    decorations,
    status,
    output,
    workspaceRoot,
    recordUsage,
    context,
  };
}

async function resolveImpactId(
  host: PrismHost,
  sym: string,
): Promise<string | undefined> {
  const t = await host.getTransport();
  try {
    const hits = await t.resolveSymbol(sym, 20);
    if (!hits.length) return undefined;
    if (hits.length === 1) return hits[0].id;
    const pick = await vscode.window.showQuickPick(
      hits.map((h) => ({
        label: h.name ?? h.id,
        description: h.path ?? h.id,
        id: h.id,
      })),
      { placeHolder: `Multiple symbols named ${sym}` },
    );
    return pick?.id;
  } catch {
    return undefined;
  }
}

export function registerCommands(context: vscode.ExtensionContext, host: PrismHost): void {
  const wrap = (id: string, fn: (...args: unknown[]) => Promise<void>) =>
    vscode.commands.registerCommand(id, async (...args: unknown[]) => {
      try {
        host.recordUsage(id);
        await fn(...args);
      } catch (e) {
        if (e instanceof PrismApiError) {
          host.recordUsage(id, e.code);
          if (e.code === "UNAUTHORIZED") {
            host.resetTransport();
            host.output.appendLine("UNAUTHORIZED — resetting transport to CLI");
          }
          if (e.code === "INDEX_UNAVAILABLE") {
            const pick = await vscode.window.showWarningMessage(
              "Index unavailable",
              "Setup Workspace",
              "Dismiss",
            );
            if (pick === "Setup Workspace") {
              await vscode.commands.executeCommand("prism.setupWorkspace");
            }
            return;
          }
          await handleRefusal(e);
          return;
        }
        const msg = e instanceof Error ? e.message : String(e);
        host.output.appendLine(msg);
        void vscode.window.showErrorMessage(msg);
      }
    });

  context.subscriptions.push(
    wrap("prism.setupWorkspace", async () => {
      const root = host.workspaceRoot();
      if (!root) return;
      host.status.text = "$(sync~spin) Prism · setup";
      try {
        const result = await vscode.window.withProgress(
          {
            location: vscode.ProgressLocation.Notification,
            title: "Prism: setting up workspace…",
            cancellable: false,
          },
          async () => runWorkspaceSetup(host.context, root, host.output),
        );
        host.resetTransport();
        host.output.appendLine(result.detail.join("\n"));
        void vscode.window.showInformationMessage(
          `Prism ready · ${result.prismPath}`,
        );
        // Orient: open graph with architecture map
        try {
          const t = await host.getTransport();
          const view = await t.view({
            view_kind: "architecture_map",
            max_nodes: 80,
          });
          host.graph.push(view);
          await vscode.commands.executeCommand("prism.graph.focus");
        } catch (e) {
          host.output.appendLine(`orient view: ${e}`);
        }
        host.status.text = "$(graph) Prism · ready";
        await host.context.workspaceState.update("prism.setupDone", true);
      } catch (e) {
        host.status.text = "$(graph) Prism · setup failed";
        throw e;
      }
    }),

    wrap("prism.compileContext", async () => {
      const ed = vscode.window.activeTextEditor;
      const word = ed?.document.getText(ed.selection) || undefined;
      const question =
        (await vscode.window.showInputBox({
          prompt: "Question for compile_context",
          value: word ? `What is ${word}?` : "Orient me in this repository",
        })) ?? undefined;
      if (!question) return;
      const anchors: string[] = [];
      if (word) anchors.push(word);
      if (ed) {
        anchors.push(
          `${vscode.workspace.asRelativePath(ed.document.uri)}:${ed.selection.active.line + 1}`,
        );
      }
      const t = await host.getTransport();
      const pack = await t.compile({
        question,
        anchors,
        intent: "repo_qa",
        budget_tokens: 4000,
      });
      host.session.setPack(pack);
      host.evidence.push(pack);
      await vscode.commands.executeCommand("prism.evidence.focus");
      void vscode.window.showInformationMessage(
        `Pack ready · ${pack.meta.tokens_used}/${pack.meta.budget_tokens} tokens`,
      );
    }),

    wrap("prism.evidencePeek", async (citationId?: unknown) => {
      let cid = typeof citationId === "string" ? citationId : undefined;
      if (!cid) {
        cid = await vscode.window.showInputBox({
          prompt: "Citation id (e.g. C1)",
          placeHolder: "C1",
        });
      }
      if (!cid) return;
      const hit = host.session.citationById(cid);
      if (!hit) {
        void vscode.window.showWarningMessage(`No citation ${cid} in last pack`);
        return;
      }
      const frag = host.session
        .get()
        .lastPack?.fragments.find((f: { id: string }) => f.id === hit.fragmentId);
      const span = frag?.provenance?.spans?.[0];
      if (span?.path) {
        const root = host.workspaceRoot();
        const uri = root
          ? vscode.Uri.file(
              span.path.startsWith("/") ? span.path : `${root}/${span.path}`,
            )
          : vscode.Uri.file(span.path);
        const doc = await vscode.workspace.openTextDocument(uri);
        const line = Math.max(0, (span.start_line ?? 1) - 1);
        const editor = await vscode.window.showTextDocument(doc);
        const range = new vscode.Range(
          line,
          0,
          span.end_line ? span.end_line - 1 : line,
          0,
        );
        editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
        editor.selection = new vscode.Selection(range.start, range.end);
      }
      if (hit.nodeIds?.[0]) {
        const t = await host.getTransport();
        const view = await t.view({
          view_kind: "impact_cone",
          seed_id: hit.nodeIds[0],
          max_nodes: 80,
        });
        host.graph.push(view);
      }
    }),

    wrap("prism.impact", async () => {
      const ed = vscode.window.activeTextEditor;
      if (!ed) return;
      const sym =
        ed.document.getText(ed.selection) ||
        ed.document.getText(ed.document.getWordRangeAtPosition(ed.selection.active));
      if (!sym) {
        void vscode.window.showWarningMessage("Select a symbol for impact");
        return;
      }
      const id = (await resolveImpactId(host, sym)) ?? sym;
      const t = await host.getTransport();
      const result = await t.impact(id);
      host.output.appendLine(JSON.stringify(result, null, 2));
      try {
        const view = await t.view({
          view_kind: "impact_cone",
          seed_id: id,
          max_nodes: 80,
        });
        host.graph.push(view);
      } catch {
        const pack = await t.compile({
          question: `Impact of ${sym}`,
          anchors: [sym],
          intent: "impact",
        });
        host.evidence.push(pack);
      }
    }),

    wrap("prism.slice", async () => {
      const ed = vscode.window.activeTextEditor;
      if (!ed) return;
      const rel = vscode.workspace.asRelativePath(ed.document.uri);
      // CLI / API use 0-based lines
      const line = ed.selection.active.line;
      const t = await host.getTransport();
      const result = (await t.slice({ path: rel, line, max_depth: 2 })) as {
        slice?: {
          spans?: Array<{ path?: string; start_line?: number; end_line?: number }>;
          nodes?: Array<{ path?: string; line?: number; id?: string }>;
        };
      };
      host.output.appendLine(JSON.stringify(result, null, 2));
      const ranges: vscode.Range[] = [];
      for (const s of result.slice?.spans ?? []) {
        if (s.path === rel && s.start_line != null) {
          ranges.push(
            new vscode.Range(
              s.start_line,
              0,
              s.end_line ?? s.start_line,
              0,
            ),
          );
        }
      }
      for (const n of result.slice?.nodes ?? []) {
        if (n.path === rel && n.line != null) {
          ranges.push(new vscode.Range(n.line, 0, n.line, 0));
        }
      }
      host.decorations.setSliceRanges(ranges);
      const seed =
        result.slice?.nodes?.find((n) => n.id)?.id ||
        `${rel}:${line}`;
      try {
        const view = await t.view({
          view_kind: "slice_path",
          seed_id: seed,
          anchors: [rel],
          path: rel,
          max_nodes: 80,
        });
        host.graph.push(view);
      } catch (e) {
        host.output.appendLine(`slice view: ${e}`);
      }
    }),

    wrap("prism.explain", async () => {
      const on = host.session.toggleExplain();
      host.evidence.push();
      void vscode.window.showInformationMessage(on ? "EXPLAIN on" : "EXPLAIN off");
    }),

    wrap("prism.repoMap", async () => {
      const t = await host.getTransport();
      const map = await t.repoMap();
      host.output.appendLine(JSON.stringify(map, null, 2));
      const view = await t.view({
        view_kind: "architecture_map",
        max_nodes: 80,
      });
      host.graph.push(view);
      await vscode.commands.executeCommand("prism.graph.focus");
    }),

    wrap("prism.entrypoints", async () => {
      const t = await host.getTransport();
      const eps = await t.entrypoints();
      host.output.appendLine(JSON.stringify(eps, null, 2));
      void vscode.window.showInformationMessage(
        "Entrypoints written to Prism output channel",
      );
    }),

    wrap("prism.buildIndex", async () => {
      const t = await host.getTransport();
      host.status.text = "$(sync~spin) Prism · indexing";
      const result = await t.buildIndex([]);
      host.output.appendLine(JSON.stringify(result, null, 2));
      host.status.text =
        t.mode === "daemon" ? "$(graph) Prism · daemon" : "$(graph) Prism · CLI";
      void vscode.window.showInformationMessage("Index build complete");
    }),

    wrap("prism.showEvidence", async () => {
      await vscode.commands.executeCommand("prism.evidence.focus");
    }),

    wrap("prism.showGraph", async () => {
      await vscode.commands.executeCommand("prism.graph.focus");
    }),

    wrap("prism.copyForLlm", async () => {
      const pack = host.session.get().lastPack;
      if (!pack) {
        void vscode.window.showWarningMessage("No pack to copy");
        return;
      }
      const text = redactPackForLlm(pack);
      await vscode.env.clipboard.writeText(text);
      host.output.appendLine("# audit pack_bound_for_llm (local, redacted)");
      host.recordUsage("pack_bound_for_llm");
      void vscode.window.showInformationMessage("Redacted pack copied to clipboard");
    }),

    wrap("prism.agent.enableMcp", async () => {
      const root = host.workspaceRoot();
      if (!root) return;
      const binary = resolvePrismBinary(root, cfg().get<string>("binaryPath") ?? "");
      if (!binary) throw new Error("prism binary not found — run Setup Workspace");
      const msg = await registerMcp(root, binary.prismPath, true);
      void vscode.window.showInformationMessage(msg);
    }),

    wrap("prism.agent.disableMcp", async () => {
      const root = host.workspaceRoot();
      if (!root) return;
      const binary = resolvePrismBinary(root, cfg().get<string>("binaryPath") ?? "") ?? {
        prismPath: "prism",
      };
      const msg = await registerMcp(root, binary.prismPath, false);
      void vscode.window.showInformationMessage(msg);
    }),

    wrap("prism.agent.generateAgentsMd", async () => {
      const root = host.workspaceRoot();
      if (!root) return;
      const binary = resolvePrismBinary(root, cfg().get<string>("binaryPath") ?? "");
      const out = await generateAgentsMd(root, binary?.prismPath);
      void vscode.window.showInformationMessage(`Wrote ${out}`);
    }),

    wrap("prism.pickAnchor", async () => {
      const ed = vscode.window.activeTextEditor;
      const word =
        ed?.document.getText(ed.selection) ||
        ed?.document.getText(
          ed.document.getWordRangeAtPosition(ed.selection.active),
        );
      const anchor = await vscode.window.showInputBox({
        prompt: "Anchor (symbol, path, or path:line)",
        value: word,
      });
      if (!anchor) return;
      const t = await host.getTransport();
      const pack = await t.compile({
        question: `Explain ${anchor}`,
        anchors: [anchor],
        intent: "repo_qa",
      });
      host.evidence.push(pack);
    }),

    wrap("prism.focusNode", async (nodeId?: unknown) => {
      if (typeof nodeId !== "string") return;
      const t = await host.getTransport();
      const view = await t.view({
        view_kind: "impact_cone",
        seed_id: nodeId,
        max_nodes: 60,
      });
      host.graph.push(view);
    }),
  );
}

export async function maybeFirstRun(
  host: PrismHost,
  context: vscode.ExtensionContext,
): Promise<void> {
  const root = host.workspaceRoot();
  if (!root) return;
  if (context.workspaceState.get("prism.setupDone")) return;
  if (cfg().get<boolean>("autoSetupOnOpen") === false) return;

  const binary = resolvePrismBinary(root, cfg().get<string>("binaryPath") ?? "");
  const pick = await vscode.window.showInformationMessage(
    binary
      ? "Prism: set up this workspace? (index + AGENTS.md + MCP + daemon)"
      : "Prism: binary not found. Set up workspace? (will try cargo build if this is the Prism repo)",
    "Setup Workspace",
    "Later",
  );
  if (pick !== "Setup Workspace") return;
  await vscode.commands.executeCommand("prism.setupWorkspace");
}
