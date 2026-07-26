import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { mountCytoscape, type GraphView } from "@prism/graph-view";

declare global {
  interface Window {
    acquireVsCodeApi?: () => {
      postMessage: (msg: unknown) => void;
      getState: () => unknown;
      setState: (s: unknown) => void;
    };
  }
}

const vscode = window.acquireVsCodeApi?.();

type EvidenceMsg = {
  type: "evidence";
  pack?: {
    meta: {
      intent: string;
      tokens_used: number;
      budget_tokens: number;
      question: string;
    };
    fragments: Array<{
      id: string;
      layer: string;
      text: string;
      confidence?: string;
      why_included?: string;
    }>;
    citations?: Array<{ id: string; fragment_id: string }>;
    gaps?: unknown[];
    explain?: { notes?: string[]; drops?: unknown[] };
  };
  showExplain?: boolean;
  transportMode?: string;
  degradationNote?: string;
};

type GraphMsg = {
  type: "graph";
  view?: GraphView;
  transportMode?: string;
  degradationNote?: string;
};

function EvidenceApp() {
  const [msg, setMsg] = useState<EvidenceMsg | null>(null);
  useEffect(() => {
    const onMessage = (e: MessageEvent) => {
      if (e.data?.type === "evidence") setMsg(e.data as EvidenceMsg);
    };
    window.addEventListener("message", onMessage);
    vscode?.postMessage({ type: "ready" });
    return () => window.removeEventListener("message", onMessage);
  }, []);

  const pack = msg?.pack;
  return (
    <div style={{ padding: 12, display: "flex", flexDirection: "column", gap: 10, height: "100%", boxSizing: "border-box" }}>
      <header>
        <div style={{ fontSize: 12, opacity: 0.75 }}>
          {msg?.transportMode ?? "…"}
          {msg?.degradationNote ? ` · ${msg.degradationNote}` : ""}
        </div>
        <h2 style={{ margin: "4px 0", fontSize: 16 }}>Evidence Pack</h2>
        {pack ? (
          <div style={{ fontSize: 13, opacity: 0.85 }}>
            {pack.meta.intent} · {pack.meta.tokens_used}/{pack.meta.budget_tokens} tokens
            <div style={{ marginTop: 4 }}>{pack.meta.question}</div>
          </div>
        ) : (
          <p style={{ opacity: 0.7 }}>Run Prism: Compile Context to populate.</p>
        )}
      </header>

      {pack?.citations && pack.citations.length > 0 && (
        <section>
          <h3 style={{ fontSize: 13, margin: "0 0 6px" }}>Citations</h3>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {pack.citations.map((c) => (
              <button
                key={c.id}
                type="button"
                onClick={() => vscode?.postMessage({ type: "peek", citationId: c.id })}
                style={{
                  cursor: "pointer",
                  border: "1px solid var(--border, #555)",
                  background: "transparent",
                  color: "inherit",
                  padding: "2px 8px",
                }}
              >
                {c.id}
              </button>
            ))}
          </div>
        </section>
      )}

      {pack && (
        <section style={{ overflow: "auto", flex: 1 }}>
          <h3 style={{ fontSize: 13, margin: "0 0 6px" }}>Layers / fragments</h3>
          {pack.fragments.map((f) => (
            <article
              key={f.id}
              style={{
                borderTop: "1px solid var(--border, #444)",
                padding: "8px 0",
                fontSize: 12,
              }}
            >
              <div style={{ opacity: 0.7 }}>
                {f.layer} · {f.confidence ?? "?"} · {f.why_included ?? f.id}
              </div>
              <pre style={{ whiteSpace: "pre-wrap", margin: "4px 0 0", fontSize: 11 }}>
                {f.text.slice(0, 1200)}
              </pre>
            </article>
          ))}
        </section>
      )}

      {msg?.showExplain && pack?.explain && (
        <section style={{ fontSize: 12, opacity: 0.85 }}>
          <h3 style={{ fontSize: 13 }}>EXPLAIN</h3>
          <pre style={{ whiteSpace: "pre-wrap" }}>
            {JSON.stringify(pack.explain, null, 2)}
          </pre>
        </section>
      )}

      <footer style={{ display: "flex", gap: 8 }}>
        <button type="button" onClick={() => vscode?.postMessage({ type: "toggleExplain" })}>
          EXPLAIN
        </button>
        <button type="button" onClick={() => vscode?.postMessage({ type: "copyForLlm" })}>
          Copy for LLM
        </button>
      </footer>
    </div>
  );
}

function GraphApp() {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<{ destroy: () => void } | null>(null);
  const [note, setNote] = useState<string>("");

  useEffect(() => {
    const onMessage = (e: MessageEvent) => {
      const data = e.data as GraphMsg;
      if (data?.type !== "graph") return;
      setNote(
        [data.transportMode, data.degradationNote].filter(Boolean).join(" · "),
      );
      if (!containerRef.current || !data.view) return;
      cyRef.current?.destroy();
      cyRef.current = mountCytoscape({
        container: containerRef.current,
        view: data.view,
        onSelect: (id, kind) => {
          if (kind === "node") vscode?.postMessage({ type: "selectNode", id });
        },
      });
    };
    window.addEventListener("message", onMessage);
    vscode?.postMessage({ type: "ready" });
    return () => {
      window.removeEventListener("message", onMessage);
      cyRef.current?.destroy();
    };
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ padding: "8px 12px", fontSize: 12, opacity: 0.75 }}>
        Graph · {note || "awaiting view-model"}
      </div>
      <div ref={containerRef} style={{ flex: 1, minHeight: 200 }} />
    </div>
  );
}

const rootEl = document.getElementById("root");
const panel = rootEl?.dataset.panel ?? "evidence";
if (rootEl) {
  createRoot(rootEl).render(panel === "graph" ? <GraphApp /> : <EvidenceApp />);
}
