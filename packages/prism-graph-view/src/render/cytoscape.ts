import cytoscape, { type Core, type ElementDefinition } from "cytoscape";
import type { GraphView } from "../model/types.js";
import { assertBudgetOk } from "../model/types.js";
import { encodeEdge, encodeNode } from "../encode/style.js";

export interface MountOptions {
  container: HTMLElement;
  view: GraphView;
  onSelect?: (id: string, kind: "node" | "edge") => void;
}

/** Mount an interactive Cytoscape graph from a view-model only (no store access). */
export function mountCytoscape(opts: MountOptions): Core {
  assertBudgetOk(opts.view);
  const elements = toElements(opts.view);
  const cy = cytoscape({
    container: opts.container,
    elements,
    style: cytoscapeStyles(),
    layout: { name: "preset" },
    userZoomingEnabled: true,
    userPanningEnabled: true,
  });

  if (opts.onSelect) {
    cy.on("tap", "node", (evt) => opts.onSelect?.(evt.target.id(), "node"));
    cy.on("tap", "edge", (evt) => opts.onSelect?.(evt.target.id(), "edge"));
  }
  return cy;
}

export function toElements(view: GraphView): ElementDefinition[] {
  const nodes: ElementDefinition[] = [...view.nodes]
    .sort((a, b) => a.id.localeCompare(b.id))
    .map((n) => {
      const st = encodeNode(n);
      return {
        group: "nodes" as const,
        data: {
          id: n.id,
          label: n.label,
          badge: st.badge,
          aria: st.ariaLabel,
          citation: n.citation,
          tier: n.tier,
          confidence: n.confidence,
        },
        position: { x: n.x, y: n.y },
        style: {
          "background-color": st.fill,
          "border-width": st.borderWidth,
          "border-color": st.borderColor,
          shape: st.shape,
          label: `${n.label}\n${st.badge}`,
          "font-size": 10,
          "text-wrap": "wrap",
          color: "#111",
        },
      };
    });

  const edges: ElementDefinition[] = [...view.edges]
    .sort((a, b) => a.id.localeCompare(b.id))
    .map((e) => {
      const st = encodeEdge(e);
      return {
        group: "edges" as const,
        data: {
          id: e.id,
          source: e.src,
          target: e.dst,
          citation: e.citation,
          aria: st.ariaLabel,
        },
        style: {
          width: st.width,
          "line-color": st.color,
          "target-arrow-color": st.color,
          "target-arrow-shape": "triangle",
          "curve-style": "bezier",
          "line-style": st.lineStyle,
        },
      };
    });

  return [...nodes, ...edges];
}

function cytoscapeStyles(): cytoscape.StylesheetStyle[] {
  return [
    {
      selector: "node",
      style: {
        "text-valign": "bottom",
        "text-halign": "center",
      },
    },
    {
      selector: "edge",
      style: {
        "curve-style": "bezier",
      },
    },
    {
      selector: ":selected",
      style: {
        "border-width": 4,
        "line-color": "#0072B2",
      },
    },
  ];
}

/**
 * ELK refinement is optional. Coordinates in the IR are already deterministic;
 * when ELK is available in the host, memoize under `.prism/views/{seed}.json`.
 * This stub documents the contract without bundling elkjs (heavy) into unit tests.
 */
export function layoutMemoKey(view: GraphView): string {
  const nodeIds = [...view.nodes.map((n) => n.id)].sort().join(",");
  const edgeIds = [...view.edges.map((e) => e.id)].sort().join(",");
  return `${view.snapshot_id}|${view.view_kind}|${view.layout.seed}|${nodeIds}|${edgeIds}`;
}
