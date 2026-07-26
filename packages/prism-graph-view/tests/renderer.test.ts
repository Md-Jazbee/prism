import { readFileSync, writeFileSync, mkdirSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  assertBudgetOk,
  collapseGroup,
  encodeEdge,
  exportMermaid,
  exportSvg,
  filterByConfidence,
  gestureToRequest,
  isGraphView,
  layoutMemoKey,
  svgFingerprint,
  toElements,
  visualExplain,
  whyHere,
  type GraphView,
} from "../src/index.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const goldenDir = join(root, "fixtures", "views", "golden");
const shotDir = join(root, "fixtures", "views", "screenshots");

function loadGolden(name: string): GraphView {
  const raw = JSON.parse(readFileSync(join(goldenDir, name), "utf8"));
  expect(isGraphView(raw)).toBe(true);
  return raw as GraphView;
}

describe("graph-view fixtures", () => {
  const files = readdirSync(goldenDir).filter((f) => f.endsWith(".json"));

  it("loads all overlay goldens with valid budgets", () => {
    expect(files.length).toBeGreaterThanOrEqual(7);
    for (const f of files) {
      const view = loadGolden(f);
      assertBudgetOk(view);
      expect(view.nodes.every((n) => n.tier && n.confidence && n.citation.node_ids.length)).toBe(
        true,
      );
      expect(view.edges.every((e) => e.tier && e.confidence && e.citation.node_ids.length)).toBe(
        true,
      );
    }
  });
});

describe("encoding", () => {
  it("uses dashed strokes for heuristic edges", () => {
    expect(encodeEdge({ kind: "CALLS", tier: "T1", confidence: "heuristic" }).lineStyle).toBe(
      "dashed",
    );
    expect(encodeEdge({ kind: "CALLS", tier: "T2", confidence: "precise" }).lineStyle).toBe(
      "solid",
    );
    expect(encodeEdge({ kind: "CALLS", tier: "T1", confidence: "observed" }).lineStyle).toBe(
      "dotted",
    );
  });
});

describe("interaction grammar", () => {
  it("maps expand to a bounded server request", () => {
    const req = gestureToRequest({ type: "expand", nodeId: "comm:src/a/" });
    expect(req.scope).toBe("server");
    expect(req.viewRequest?.seed_id).toBe("comm:src/a/");
    expect(req.refusal).toContain("VIEW_TOO_LARGE");
  });

  it("collapses groups with weakest confidence", () => {
    const view = loadGolden("architecture_map.json");
    // Give both nodes same group for collapse demo
    const tweaked: GraphView = {
      ...view,
      nodes: view.nodes.map((n, i) =>
        i < 2 ? { ...n, group: "g1" } : n,
      ),
    };
    const collapsed = collapseGroup(tweaked, "g1");
    expect(collapsed.nodes.some((n) => n.id.startsWith("agg:"))).toBe(true);
    const agg = collapsed.nodes.find((n) => n.id.startsWith("agg:"))!;
    expect(agg.confidence).toBe("heuristic");
  });

  it("filters confidence without inventing nodes", () => {
    const view = loadGolden("architecture_map.json");
    const filtered = filterByConfidence(view, ["precise"]);
    expect(filtered.nodes.length).toBeLessThanOrEqual(view.nodes.length);
    expect(filtered.edges.every((e) => e.confidence === "precise")).toBe(true);
  });
});

describe("visual EXPLAIN + overlays", () => {
  it("surfaces pack drops", () => {
    const view = loadGolden("pack_map.json");
    const items = visualExplain(view);
    expect(items.map((i) => i.reason).sort()).toEqual(["BUDGET_SOFT_DROP", "DEDUPE"]);
    expect(whyHere(view, "frag:noise").drop?.reason).toBe("BUDGET_SOFT_DROP");
  });
});

describe("export + screenshot-diff", () => {
  it("SVG is deterministic for architecture_map", () => {
    const view = loadGolden("architecture_map.json");
    const a = svgFingerprint(exportSvg(view));
    const b = svgFingerprint(exportSvg(view));
    expect(a).toBe(b);
    expect(a).toContain('data-layout-seed="p7archseed0001"');
    expect(a).toContain("Heuristic — dashed");
  });

  it("matches committed SVG baselines (or writes when UPDATE_SHOTS=1)", () => {
    mkdirSync(shotDir, { recursive: true });
    const update = process.env.UPDATE_SHOTS === "1";
    for (const f of readdirSync(goldenDir).filter((x) => x.endsWith(".json"))) {
      const view = loadGolden(f);
      const svg = svgFingerprint(exportSvg(view));
      const out = join(shotDir, f.replace(/\.json$/, ".svg"));
      if (update) {
        writeFileSync(out, svg + "\n");
      } else {
        const baseline = svgFingerprint(readFileSync(out, "utf8"));
        expect(svg).toBe(baseline);
      }
    }
  });

  it("mermaid export includes drop comments for pack_map", () => {
    const m = exportMermaid(loadGolden("pack_map.json"));
    expect(m).toContain("%% DROP frag:noise");
    expect(m).toContain("flowchart LR");
  });

  it("cytoscape elements preserve citation payload", () => {
    const els = toElements(loadGolden("slice_path.json"));
    const node = els.find((e) => e.group === "nodes" && e.data?.id === "src");
    expect(node?.data?.citation).toBeTruthy();
    expect(layoutMemoKey(loadGolden("slice_path.json"))).toContain("p7sliceseed0001");
  });
});
