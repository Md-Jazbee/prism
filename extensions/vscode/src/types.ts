/** Shared types for extension host ↔ daemon/CLI. */

export type TransportMode = "daemon" | "cli";

export interface HealthInfo {
  ok: boolean;
  service: string;
  api_version: string;
  snapshot_id: string;
  workspace: string;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
    hint?: string;
    snapshot_id?: string;
    suggested_anchors?: string[];
  };
}

export class PrismApiError extends Error {
  readonly code: string;
  readonly hint?: string;
  readonly snapshotId?: string;
  readonly suggestedAnchors?: string[];

  constructor(body: ApiErrorBody["error"]) {
    super(body.message);
    this.name = "PrismApiError";
    this.code = body.code;
    this.hint = body.hint;
    this.snapshotId = body.snapshot_id;
    this.suggestedAnchors = body.suggested_anchors;
  }
}

export interface CompileRequest {
  question: string;
  budget_tokens?: number;
  intent?: string;
  anchors?: string[];
  stack_frames?: string[];
  error_text?: string;
  changed_paths?: string[];
  require_precise?: boolean;
}

export interface EvidencePack {
  meta: {
    intent: string;
    budget_tokens: number;
    tokens_used: number;
    question: string;
    plan_id?: string;
    schema_version?: string;
  };
  hierarchy: Record<string, string[]>;
  fragments: EvidenceFragment[];
  citations?: EvidenceCitation[];
  gaps?: unknown[];
  explain?: ExplainPayload;
  drops?: unknown[];
}

export interface EvidenceFragment {
  id: string;
  kind: string;
  layer: string;
  text: string;
  token_estimate: number;
  confidence?: string;
  why_included?: string;
  must_include?: boolean;
  provenance?: {
    node_ids?: string[];
    analyzer?: string;
    tier?: string;
    spans?: Array<{ path: string; start_line?: number; end_line?: number }>;
  };
}

export interface EvidenceCitation {
  id: string;
  fragment_id: string;
  node_ids?: string[];
}

export interface ExplainPayload {
  budget_tokens?: number;
  tokens_used?: number;
  must_include_ok?: boolean;
  notes?: string[];
  drops?: Array<{ fragment_id?: string; reason?: string }>;
  fragments?: Array<{
    fragment_id: string;
    kept: boolean;
    must_include?: boolean;
    why_included?: string;
  }>;
}

export interface GraphViewPayload {
  schema_version: string;
  snapshot_id: string;
  view_kind: string;
  nodes: unknown[];
  edges: unknown[];
  [key: string]: unknown;
}

export interface SessionState {
  lastPack?: EvidencePack;
  lastView?: GraphViewPayload;
  showExplain: boolean;
  transportMode: TransportMode;
  degradationNote?: string;
}
