import type { GraphView, ViewEdge, ViewNode } from "../model/types.js";
import { encodeEdge, encodeNode, LEGEND_ITEMS, PALETTE } from "../encode/style.js";

function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function dashArray(style: "solid" | "dashed" | "dotted"): string {
  if (style === "dashed") return "6 4";
  if (style === "dotted") return "2 3";
  return "none";
}

/** Deterministic SVG export — screenshot-diff friendly (no Cytoscape required). */
export function exportSvg(view: GraphView, opts?: { width?: number; height?: number }): string {
  const nodes = [...view.nodes].sort((a, b) => a.id.localeCompare(b.id));
  const edges = [...view.edges].sort((a, b) => a.id.localeCompare(b.id));

  const xs = nodes.map((n) => n.x);
  const ys = nodes.map((n) => n.y);
  const minX = Math.min(0, ...xs) - 40;
  const minY = Math.min(0, ...ys) - 40;
  const maxX = Math.max(200, ...xs) + 120;
  const maxY = Math.max(200, ...ys) + 80;
  const width = opts?.width ?? Math.ceil(maxX - minX);
  const height = opts?.height ?? Math.ceil(maxY - minY + 60);

  const byId = new Map(nodes.map((n) => [n.id, n]));

  const edgeEls = edges
    .map((e) => {
      const s = byId.get(e.src);
      const d = byId.get(e.dst);
      if (!s || !d) return "";
      const st = encodeEdge(e);
      return `<line data-id="${esc(e.id)}" x1="${s.x - minX}" y1="${s.y - minY}" x2="${d.x - minX}" y2="${d.y - minY}" stroke="${st.color}" stroke-width="${st.width}" stroke-dasharray="${dashArray(st.lineStyle)}" />`;
    })
    .filter(Boolean)
    .join("\n");

  const nodeEls = nodes
    .map((n) => {
      const st = encodeNode(n);
      const x = n.x - minX;
      const y = n.y - minY;
      const r = 14 + st.heatBoost * 6;
      const shape =
        st.shape === "round-rectangle"
          ? `<rect x="${x - r}" y="${y - r}" width="${r * 2}" height="${r * 2}" rx="4" fill="${st.fill}" stroke="${st.borderColor}" stroke-width="${st.borderWidth}" />`
          : st.shape === "diamond"
            ? `<polygon points="${x},${y - r} ${x + r},${y} ${x},${y + r} ${x - r},${y}" fill="${st.fill}" stroke="${st.borderColor}" stroke-width="${st.borderWidth}" />`
            : `<circle cx="${x}" cy="${y}" r="${r}" fill="${st.fill}" stroke="${st.borderColor}" stroke-width="${st.borderWidth}" />`;
      return `<g data-id="${esc(n.id)}" role="img" aria-label="${esc(st.ariaLabel)}">${shape}<text x="${x}" y="${y + r + 12}" text-anchor="middle" font-size="10" fill="${PALETTE.text}">${esc(n.label)}</text><text x="${x}" y="${y + r + 24}" text-anchor="middle" font-size="8" fill="${PALETTE.text}">${esc(st.badge)}</text></g>`;
    })
    .join("\n");

  const legend = LEGEND_ITEMS.map(
    (item, i) =>
      `<text x="8" y="${height - 48 + i * 12}" font-size="10" fill="${PALETTE.text}">${esc(item.label)}</text>`,
  ).join("\n");

  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" data-view-kind="${esc(view.view_kind)}" data-layout-seed="${esc(view.layout.seed)}" data-snapshot="${esc(view.snapshot_id)}">
<title>${esc(view.view_kind)} · ${esc(view.snapshot_id)}</title>
<rect width="100%" height="100%" fill="${PALETTE.background}"/>
${edgeEls}
${nodeEls}
${legend}
</svg>
`;
}

/** Stable fingerprint for screenshot-diff without binary PNGs. */
export function svgFingerprint(svg: string): string {
  // Normalize whitespace only — layout coords must stay.
  return svg.replace(/\r\n/g, "\n").trim();
}

export function exportMermaid(view: GraphView): string {
  const nodes = [...view.nodes].sort((a, b) => a.id.localeCompare(b.id));
  const edges = [...view.edges].sort((a, b) => a.id.localeCompare(b.id));
  const lines: string[] = ["flowchart LR"];
  for (const n of nodes) {
    const id = safeMermaidId(n.id);
    lines.push(`  ${id}["${escapeMermaid(n.label)} (${n.tier}/${n.confidence})"]`);
  }
  for (const e of edges) {
    const style = e.confidence === "precise" ? "-->" : e.confidence === "observed" ? "-.->" : "-.->";
    lines.push(`  ${safeMermaidId(e.src)} ${style} ${safeMermaidId(e.dst)}`);
  }
  if (view.drops?.length) {
    lines.push("  %% drops");
    for (const d of [...view.drops].sort((a, b) => a.id.localeCompare(b.id))) {
      lines.push(`  %% DROP ${d.id}: ${d.reason}`);
    }
  }
  return lines.join("\n") + "\n";
}

function safeMermaidId(id: string): string {
  return "n_" + id.replace(/[^a-zA-Z0-9_]/g, "_");
}

function escapeMermaid(s: string): string {
  return s.replace(/"/g, "'");
}

export type { ViewNode, ViewEdge };
