/** Graph View-Model v1 types — mirror of schemas/graph-view/v1. */

export const GRAPH_VIEW_SCHEMA_VERSION = "graph-view/v1" as const;

export type Tier = "T1" | "T2" | "T3" | "T4";
export type Confidence = "extracted" | "heuristic" | "precise" | "observed";

export interface Span {
  start_line: number;
  end_line: number;
}

export interface Citation {
  node_ids: string[];
  file_path?: string;
  span?: Span;
}

export interface ViewNode {
  id: string;
  label: string;
  kind: string;
  tier: string;
  confidence: string;
  lod_rank: number;
  group?: string;
  citation: Citation;
  x: number;
  y: number;
  heat?: number;
}

export interface ViewEdge {
  id: string;
  src: string;
  dst: string;
  kind: string;
  tier: string;
  confidence: string;
  citation: Citation;
}

export interface ViewGroup {
  id: string;
  label: string;
  kind?: string;
}

export interface LayoutInfo {
  algorithm: string;
  seed: string;
  notes?: string[];
}

export interface BudgetUsed {
  max_nodes: number;
  max_edges: number;
  nodes_used: number;
  edges_used: number;
}

export interface DropRecord {
  id: string;
  reason: string;
}

export interface GraphView {
  schema_version: string;
  snapshot_id: string;
  view_kind: string;
  nodes: ViewNode[];
  edges: ViewEdge[];
  groups?: ViewGroup[];
  budget: BudgetUsed;
  layout: LayoutInfo;
  drops?: DropRecord[];
  notes?: string[];
}

export interface ViewTooLarge {
  code: "VIEW_TOO_LARGE" | string;
  message: string;
  view_kind: string;
  snapshot_id?: string;
  candidate_nodes: number;
  max_nodes: number;
  suggested_anchors: string[];
  hint: string;
}

export function isGraphView(v: unknown): v is GraphView {
  if (!v || typeof v !== "object") return false;
  const o = v as Record<string, unknown>;
  return (
    o.schema_version === GRAPH_VIEW_SCHEMA_VERSION &&
    typeof o.snapshot_id === "string" &&
    typeof o.view_kind === "string" &&
    Array.isArray(o.nodes) &&
    Array.isArray(o.edges) &&
    typeof o.budget === "object" &&
    typeof o.layout === "object"
  );
}

export function assertBudgetOk(view: GraphView): void {
  if (view.budget.nodes_used > view.budget.max_nodes) {
    throw new Error(
      `budget violated: nodes_used ${view.budget.nodes_used} > max_nodes ${view.budget.max_nodes}`,
    );
  }
  if (view.budget.edges_used > view.budget.max_edges) {
    throw new Error(
      `budget violated: edges_used ${view.budget.edges_used} > max_edges ${view.budget.max_edges}`,
    );
  }
  if (view.nodes.length > view.budget.max_nodes) {
    throw new Error(`node count ${view.nodes.length} exceeds max_nodes`);
  }
}
