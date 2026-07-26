/** Visual encoding — confidence/tier → shape, stroke, dash. Color alone is never sufficient. */

export interface EncodedNodeStyle {
  shape: "ellipse" | "round-rectangle" | "diamond" | "hexagon";
  fill: string;
  borderWidth: number;
  borderColor: string;
  badge: string;
  /** Heat-driven opacity boost 0–1 */
  heatBoost: number;
  ariaLabel: string;
}

export interface EncodedEdgeStyle {
  lineStyle: "solid" | "dashed" | "dotted";
  width: number;
  color: string;
  ariaLabel: string;
}

/** Colorblind-safe categorical fills (Okabe–Ito inspired). */
export const PALETTE = {
  t1: "#0072B2",
  t2: "#009E73",
  t3: "#E69F00",
  t4: "#CC79A7",
  heuristic: "#D55E00",
  precise: "#000000",
  observed: "#56B4E9",
  aggregated: "#999999",
  background: "#F7F7F7",
  text: "#111111",
} as const;

const CONFIDENCE_RANK: Record<string, number> = {
  heuristic: 0,
  extracted: 1,
  precise: 2,
  observed: 3,
};

export function confidenceRank(c: string): number {
  return CONFIDENCE_RANK[c] ?? 0;
}

/** Weakest confidence wins (aggregation honesty). */
export function weakestConfidence(a: string, b: string): string {
  return confidenceRank(a) <= confidenceRank(b) ? a : b;
}

export function encodeNode(n: {
  id: string;
  label: string;
  kind: string;
  tier: string;
  confidence: string;
  heat?: number;
}): EncodedNodeStyle {
  const tier = n.tier.toUpperCase();
  const fill =
    tier === "T4"
      ? PALETTE.t4
      : tier === "T3"
        ? PALETTE.t3
        : tier === "T2"
          ? PALETTE.t2
          : PALETTE.t1;

  let shape: EncodedNodeStyle["shape"] = "ellipse";
  if (n.kind === "Community" || n.kind.includes("Group")) shape = "round-rectangle";
  else if (n.kind.includes("Ambigu")) shape = "diamond";
  else if (n.kind.includes("Hotspot") || n.kind.includes("Layer")) shape = "hexagon";

  const borderWidth = tier === "T1" ? 1 : tier === "T2" ? 2 : tier === "T3" ? 3 : 4;
  const heatBoost = Math.min(1, Math.max(0, (n.heat ?? 0) / 10));

  return {
    shape,
    fill,
    borderWidth,
    borderColor: PALETTE.text,
    badge: `${tier}·${n.confidence}`,
    heatBoost,
    ariaLabel: `${n.label}, ${n.kind}, tier ${tier}, confidence ${n.confidence}`,
  };
}

export function encodeEdge(e: {
  kind: string;
  tier: string;
  confidence: string;
  memberCount?: number;
}): EncodedEdgeStyle {
  const lineStyle: EncodedEdgeStyle["lineStyle"] =
    e.confidence === "precise"
      ? "solid"
      : e.confidence === "observed"
        ? "dotted"
        : "dashed";

  const width = Math.min(6, 1 + Math.log2(1 + (e.memberCount ?? 1)));
  const color =
    e.kind === "AGGREGATED"
      ? PALETTE.aggregated
      : e.confidence === "heuristic"
        ? PALETTE.heuristic
        : PALETTE.precise;

  return {
    lineStyle,
    width,
    color,
    ariaLabel: `edge ${e.kind}, ${e.confidence}, tier ${e.tier}`,
  };
}

export const LEGEND_ITEMS = [
  { key: "precise", label: "Precise — solid stroke", lineStyle: "solid" as const },
  { key: "heuristic", label: "Heuristic — dashed stroke", lineStyle: "dashed" as const },
  { key: "observed", label: "Observed — dotted stroke", lineStyle: "dotted" as const },
  { key: "tier", label: "Tier — badge + border weight (not color alone)" },
] as const;
