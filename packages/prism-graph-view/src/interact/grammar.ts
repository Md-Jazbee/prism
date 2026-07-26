import type { GraphView } from "../model/types.js";
import { weakestConfidence } from "../encode/style.js";

export type Gesture =
  | { type: "focus"; nodeId: string }
  | { type: "expand"; nodeId: string }
  | { type: "collapse"; groupId: string }
  | { type: "filter_tier"; minTier: "T1" | "T2" | "T3" | "T4" }
  | { type: "filter_confidence"; allowed: string[] }
  | { type: "path_between"; src: string; dst: string }
  | { type: "why_here"; elementId: string }
  | { type: "breadcrumb_back" };

export interface BoundedRequest {
  gesture: Gesture;
  /** Local-only vs needs daemon re-project */
  scope: "local" | "server";
  /** Suggested POST /v1/view body when scope=server */
  viewRequest?: Record<string, unknown>;
  /** Max nodes this gesture may introduce locally */
  localBudget: number;
  refusal: string;
}

const TIER_ORD: Record<string, number> = { T1: 1, T2: 2, T3: 3, T4: 4 };

/** Map a UI gesture to a budgeted request (never unbounded). */
export function gestureToRequest(g: Gesture, current?: GraphView): BoundedRequest {
  switch (g.type) {
    case "focus":
      return {
        gesture: g,
        scope: "local",
        localBudget: 40,
        refusal: "Focus stays within the current view; re-project to widen.",
      };
    case "expand":
      return {
        gesture: g,
        scope: "server",
        viewRequest: {
          view_kind: current?.view_kind ?? "architecture_map",
          seed_id: g.nodeId,
          max_nodes: current?.budget.max_nodes ?? 80,
        },
        localBudget: 0,
        refusal: "Expand requires a budgeted /v1/view; daemon may return VIEW_TOO_LARGE.",
      };
    case "collapse":
      return {
        gesture: g,
        scope: "local",
        localBudget: 0,
        refusal: "Collapse is local aggregation only.",
      };
    case "filter_tier":
    case "filter_confidence":
      return {
        gesture: g,
        scope: "local",
        localBudget: 0,
        refusal: "Filters only hide; they do not invent nodes.",
      };
    case "path_between":
      return {
        gesture: g,
        scope: "server",
        viewRequest: {
          view_kind: "slice_path",
          seed_id: g.src,
          anchors: [g.src, g.dst],
          max_nodes: 80,
        },
        localBudget: 0,
        refusal: "Path-between is a budgeted slice_path projection.",
      };
    case "why_here":
      return {
        gesture: g,
        scope: "local",
        localBudget: 0,
        refusal: "Citation/EXPLAIN must already be on the element.",
      };
    case "breadcrumb_back":
      return {
        gesture: g,
        scope: "local",
        localBudget: 0,
        refusal: "Client history only — no new query.",
      };
  }
}

export function filterByMinTier(view: GraphView, minTier: string): GraphView {
  const min = TIER_ORD[minTier] ?? 1;
  const nodes = view.nodes.filter((n) => (TIER_ORD[n.tier] ?? 1) >= min);
  const ids = new Set(nodes.map((n) => n.id));
  const edges = view.edges.filter((e) => ids.has(e.src) && ids.has(e.dst));
  return {
    ...view,
    nodes,
    edges,
    budget: {
      ...view.budget,
      nodes_used: nodes.length,
      edges_used: edges.length,
    },
  };
}

export function filterByConfidence(view: GraphView, allowed: string[]): GraphView {
  const allow = new Set(allowed);
  const edges = view.edges.filter((e) => allow.has(e.confidence));
  const nodeIds = new Set(edges.flatMap((e) => [e.src, e.dst]));
  // Keep all nodes that match OR are endpoints of kept edges OR lod_rank 0
  const nodes = view.nodes.filter(
    (n) => allow.has(n.confidence) || nodeIds.has(n.id) || n.lod_rank === 0,
  );
  const ids = new Set(nodes.map((n) => n.id));
  const edges2 = edges.filter((e) => ids.has(e.src) && ids.has(e.dst));
  return {
    ...view,
    nodes,
    edges: edges2,
    budget: {
      ...view.budget,
      nodes_used: nodes.length,
      edges_used: edges2.length,
    },
  };
}

export function focusNeighborhood(view: GraphView, nodeId: string, hops = 2): GraphView {
  const keep = new Set<string>([nodeId]);
  for (let h = 0; h < hops; h++) {
    for (const e of view.edges) {
      if (keep.has(e.src)) keep.add(e.dst);
      if (keep.has(e.dst)) keep.add(e.src);
    }
  }
  const nodes = view.nodes.filter((n) => keep.has(n.id));
  const ids = new Set(nodes.map((n) => n.id));
  const edges = view.edges.filter((e) => ids.has(e.src) && ids.has(e.dst));
  return {
    ...view,
    nodes,
    edges,
    budget: {
      ...view.budget,
      nodes_used: nodes.length,
      edges_used: edges.length,
    },
  };
}

/** Collapse a group into one super-node; aggregated edges inherit weakest confidence. */
export function collapseGroup(view: GraphView, groupId: string): GraphView {
  const members = view.nodes.filter((n) => n.group === groupId);
  if (members.length < 2) return view;
  const memberIds = new Set(members.map((m) => m.id));
  const conf = members.reduce((a, m) => weakestConfidence(a, m.confidence), members[0].confidence);
  const tier = members.reduce((a, m) => (a > m.tier ? a : m.tier), members[0].tier);
  const superNode = {
    ...members[0],
    id: `agg:${groupId}`,
    label: `${members[0].label} (+${members.length - 1})`,
    kind: "Aggregated",
    confidence: conf,
    tier,
    citation: {
      node_ids: members.flatMap((m) => m.citation.node_ids).slice(0, 32),
      file_path: members[0].citation.file_path,
    },
  };
  const nodes = [
    ...view.nodes.filter((n) => !memberIds.has(n.id)),
    superNode,
  ].sort((a, b) => a.id.localeCompare(b.id));

  const edgeMap = new Map<string, (typeof view.edges)[0] & { memberCount?: number }>();
  for (const e of view.edges) {
    const src = memberIds.has(e.src) ? superNode.id : e.src;
    const dst = memberIds.has(e.dst) ? superNode.id : e.dst;
    if (src === dst) continue;
    const key = `${src}->${dst}:${e.kind}`;
    const prev = edgeMap.get(key);
    if (!prev) {
      edgeMap.set(key, {
        ...e,
        id: `agg-e:${key}`,
        src,
        dst,
        kind: e.kind,
        confidence: e.confidence,
        memberCount: 1,
      });
    } else {
      prev.memberCount = (prev.memberCount ?? 1) + 1;
      prev.confidence = weakestConfidence(prev.confidence, e.confidence);
      prev.kind = prev.kind === e.kind ? prev.kind : "AGGREGATED";
    }
  }
  const edges = [...edgeMap.values()].sort((a, b) => a.id.localeCompare(b.id));
  return {
    ...view,
    nodes,
    edges,
    budget: {
      ...view.budget,
      nodes_used: nodes.length,
      edges_used: edges.length,
    },
    notes: [...(view.notes ?? []), `collapsed group ${groupId}`],
  };
}

export function whyHere(
  view: GraphView,
  elementId: string,
): { citation?: GraphView["nodes"][0]["citation"]; drop?: { id: string; reason: string } } {
  const node = view.nodes.find((n) => n.id === elementId);
  if (node) return { citation: node.citation };
  const edge = view.edges.find((e) => e.id === elementId);
  if (edge) return { citation: edge.citation };
  const drop = view.drops?.find((d) => d.id === elementId);
  if (drop) return { drop };
  return {};
}
