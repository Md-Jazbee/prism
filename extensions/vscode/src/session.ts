import { SessionState, EvidencePack, GraphViewPayload, TransportMode, EvidenceCitation } from "./types";

export class PrismSession {
  private state: SessionState = {
    showExplain: false,
    transportMode: "cli",
  };

  get(): SessionState {
    return this.state;
  }

  setTransport(mode: TransportMode, note?: string): void {
    this.state = {
      ...this.state,
      transportMode: mode,
      degradationNote: note,
    };
  }

  setPack(pack: EvidencePack): void {
    this.state = { ...this.state, lastPack: pack };
  }

  setView(view: GraphViewPayload): void {
    this.state = { ...this.state, lastView: view };
  }

  toggleExplain(): boolean {
    this.state = { ...this.state, showExplain: !this.state.showExplain };
    return this.state.showExplain;
  }

  citationById(cid: string): { fragmentId?: string; nodeIds?: string[] } | undefined {
    const pack = this.state.lastPack;
    if (!pack) return undefined;
    const c = pack.citations?.find(
      (x: EvidenceCitation) =>
        x.id === cid || x.id === `C${cid}` || x.id === cid.replace(/^C/i, ""),
    );
    if (c) {
      return { fragmentId: c.fragment_id, nodeIds: c.node_ids };
    }
    // Fallback: C1 → first citation / fragment
    const idx = Number(String(cid).replace(/^C/i, "")) - 1;
    if (Number.isFinite(idx) && pack.citations?.[idx]) {
      const hit = pack.citations[idx];
      return { fragmentId: hit.fragment_id, nodeIds: hit.node_ids };
    }
    if (Number.isFinite(idx) && pack.fragments[idx]) {
      const frag = pack.fragments[idx];
      return { fragmentId: frag.id, nodeIds: frag.provenance?.node_ids };
    }
    return undefined;
  }
}
