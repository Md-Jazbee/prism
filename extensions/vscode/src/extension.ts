import * as vscode from "vscode";
import { PrismSession } from "./session";
import { EvidencePanelProvider, GraphPanelProvider } from "./panels/providers";
import { createHost, registerCommands, maybeFirstRun } from "./commands";

/**
 * Activation is intentionally thin: register views/commands only.
 * Daemon spawn and transport connect are deferred until first command
 * (see EXTENSION-ACTIVATION-BUDGET.md).
 */
export function activate(context: vscode.ExtensionContext): void {
  const started = Date.now();
  const session = new PrismSession();
  const evidence = new EvidencePanelProvider(context.extensionUri, session);
  const graph = new GraphPanelProvider(context.extensionUri, session);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(EvidencePanelProvider.viewType, evidence),
    vscode.window.registerWebviewViewProvider(GraphPanelProvider.viewType, graph),
  );

  const host = createHost(context, session, evidence, graph);
  context.subscriptions.push(host.status, host.output, host.decorations);
  registerCommands(context, host);

  // Defer first-run to idle — does not block activation promise.
  setTimeout(() => {
    void maybeFirstRun(host, context);
  }, 0);

  const elapsed = Date.now() - started;
  host.output.appendLine(`Prism activated in ${elapsed}ms (budget ≤150ms registration)`);
}

export function deactivate(): void {
  /* daemon left running for reuse; lockfile-owned */
}
