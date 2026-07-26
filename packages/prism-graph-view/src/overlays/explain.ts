import type { DropRecord, GraphView } from "../model/types.js";

export interface VisualExplainItem {
  id: string;
  reason: string;
  /** Stable sort key for screenshot diffs */
  sortKey: string;
}

/** Visual EXPLAIN — drops + reason codes as first-class annotations. */
export function visualExplain(view: GraphView): VisualExplainItem[] {
  const drops = view.drops ?? [];
  return [...drops]
    .map((d: DropRecord) => ({
      id: d.id,
      reason: d.reason,
      sortKey: `${d.reason}::${d.id}`,
    }))
    .sort((a, b) => a.sortKey.localeCompare(b.sortKey));
}

export function explainSvgAnnotations(view: GraphView): string {
  const items = visualExplain(view);
  if (!items.length) return "<!-- no drops -->";
  return items
    .map(
      (it, i) =>
        `<text data-explain="${escapeXml(it.id)}" x="8" y="${16 + i * 14}" font-size="11" fill="#D55E00">DROP ${escapeXml(it.id)}: ${escapeXml(it.reason)}</text>`,
    )
    .join("\n");
}

function escapeXml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** Overlay helpers: mark nodes that participate in pack/slice/impact semantics. */
export function overlayFlags(view: GraphView): {
  isPackMap: boolean;
  isImpact: boolean;
  isSlice: boolean;
  isHeat: boolean;
  hasExplain: boolean;
} {
  return {
    isPackMap: view.view_kind === "pack_map",
    isImpact: view.view_kind === "impact_cone",
    isSlice: view.view_kind === "slice_path",
    isHeat:
      view.view_kind === "hotspot_heat" ||
      view.view_kind === "ambiguity_heat" ||
      view.view_kind === "layering_violations",
    hasExplain: (view.drops?.length ?? 0) > 0,
  };
}
