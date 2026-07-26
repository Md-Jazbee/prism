import * as vscode from "vscode";
import { PrismSession } from "../session";
import { EvidencePack, GraphViewPayload } from "../types";

function nonce(): string {
  return Math.random().toString(36).slice(2);
}

function csp(webview: vscode.Webview): string {
  return [
    `default-src 'none'`,
    `img-src ${webview.cspSource} data:`,
    `style-src ${webview.cspSource} 'unsafe-inline'`,
    `script-src ${webview.cspSource} 'nonce-${nonce()}'`,
  ].join("; ");
}

/** Shared HTML shell for evidence + graph panels. */
export function webviewHtml(
  webview: vscode.Webview,
  extensionUri: vscode.Uri,
  panel: "evidence" | "graph",
): string {
  const scriptUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, "dist", "webview", "main.js"),
  );
  const n = nonce();
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource} data:; style-src ${webview.cspSource} 'unsafe-inline'; script-src ${webview.cspSource} 'nonce-${n}';" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Prism ${panel}</title>
  <style>
    :root {
      color-scheme: light dark;
      --bg: var(--vscode-editor-background);
      --fg: var(--vscode-editor-foreground);
      --muted: var(--vscode-descriptionForeground);
      --accent: var(--vscode-textLink-foreground);
      --border: var(--vscode-panel-border, #444);
    }
    html, body, #root { height: 100%; margin: 0; background: var(--bg); color: var(--fg); font-family: var(--vscode-font-family); }
  </style>
</head>
<body>
  <div id="root" data-panel="${panel}"></div>
  <script nonce="${n}" src="${scriptUri}"></script>
</body>
</html>`;
}

export class EvidencePanelProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = "prism.evidence";
  private view?: vscode.WebviewView;

  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly session: PrismSession,
  ) {}

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [vscode.Uri.joinPath(this.extensionUri, "dist")],
    };
    webviewView.webview.html = webviewHtml(
      webviewView.webview,
      this.extensionUri,
      "evidence",
    );
    webviewView.webview.onDidReceiveMessage((msg) => {
      if (msg?.type === "ready") this.push();
      if (msg?.type === "peek" && msg.citationId) {
        void vscode.commands.executeCommand("prism.evidencePeek", msg.citationId);
      }
      if (msg?.type === "copyForLlm") {
        void vscode.commands.executeCommand("prism.copyForLlm");
      }
      if (msg?.type === "toggleExplain") {
        void vscode.commands.executeCommand("prism.explain");
      }
    });
    this.push();
  }

  push(pack?: EvidencePack): void {
    if (pack) this.session.setPack(pack);
    const s = this.session.get();
    this.view?.webview.postMessage({
      type: "evidence",
      pack: s.lastPack,
      showExplain: s.showExplain,
      transportMode: s.transportMode,
      degradationNote: s.degradationNote,
    });
  }
}

export class GraphPanelProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = "prism.graph";
  private view?: vscode.WebviewView;

  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly session: PrismSession,
  ) {}

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.extensionUri, "dist"),
        vscode.Uri.joinPath(this.extensionUri, "..", "..", "packages", "prism-graph-view", "dist"),
      ],
    };
    webviewView.webview.html = webviewHtml(
      webviewView.webview,
      this.extensionUri,
      "graph",
    );
    webviewView.webview.onDidReceiveMessage((msg) => {
      if (msg?.type === "ready") this.push();
      if (msg?.type === "selectNode" && msg.id) {
        void vscode.commands.executeCommand("prism.focusNode", msg.id);
      }
    });
    this.push();
  }

  push(view?: GraphViewPayload): void {
    if (view) this.session.setView(view);
    const s = this.session.get();
    this.view?.webview.postMessage({
      type: "graph",
      view: s.lastView,
      transportMode: s.transportMode,
      degradationNote: s.degradationNote,
    });
  }
}

// silence unused csp helper warning in some bundlers
void csp;
