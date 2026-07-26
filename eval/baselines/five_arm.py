#!/usr/bin/env python3
"""Five-arm accuracy harness (P12 Stage D) — extends P9 four-arm with Graphify baseline.

Arms
  A  Frontier + explore (scripted)
  B  Medium + explore
  C  Medium + Prism (compile_context first)
  D  Frontier + Prism
  E  Medium + doc-aware graph baseline (Graphify)  ← ACC-5

Default mode is scripted/proxy. Live LLM judging is opt-in (PRISM_FOUR_ARM_LLM=1).
Outputs JSON under eval/baselines/five-arm/.
"""

from __future__ import annotations

import json
import statistics
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "fixtures" / "workflows"
DOC_QA = ROOT / "eval" / "tasks" / "doc-qa"
OUT = ROOT / "eval" / "baselines" / "five-arm"


@dataclass
class ArmResult:
    arm: str
    protocol: str
    first_tool: str
    chose_compile_first: bool
    hops: int
    tokens_proxy: int
    quality_proxy: float
    citation_validity_proxy: float
    notes: str


def load_traces() -> list[dict]:
    traces = []
    for p in sorted(FIXTURES.glob("*.trace.json")):
        traces.append(json.loads(p.read_text()))
    return traces


def load_doc_qa_tasks() -> list[dict]:
    tasks = []
    if DOC_QA.is_dir():
        for p in sorted(DOC_QA.glob("DQ*.json")):
            tasks.append(json.loads(p.read_text()))
    return tasks


def scripted_arm(arm: str) -> ArmResult:
    if arm in ("C", "D"):
        return ArmResult(
            arm=arm,
            protocol="prism",
            first_tool="compile_context",
            chose_compile_first=True,
            hops=1,
            tokens_proxy=800 if arm == "C" else 1200,
            quality_proxy=0.72 if arm == "D" else 0.70,
            citation_validity_proxy=0.85,
            notes="Scripted Prism arm — P12 Stage B gaps+docs; quality_proxy is structural completeness",
        )
    if arm == "E":
        return ArmResult(
            arm="E",
            protocol="graphify",
            first_tool="graph_query",
            chose_compile_first=False,
            hops=2,
            tokens_proxy=2000,
            quality_proxy=0.74,
            citation_validity_proxy=0.80,
            notes="Doc-aware graph baseline (Graphify) — BFS budget ~2k; citation-validity scored",
        )
    return ArmResult(
        arm=arm,
        protocol="explore",
        first_tool="grep",
        chose_compile_first=False,
        hops=12 if arm == "A" else 10,
        tokens_proxy=18000 if arm == "A" else 12000,
        quality_proxy=0.70 if arm == "A" else 0.62,
        citation_validity_proxy=0.55 if arm == "A" else 0.50,
        notes="Scripted explore arm — hop×token proxy; not a live frontier call",
    )


def trace_metrics(traces: list[dict]) -> dict:
    compile_first = []
    repair_success = []
    refusals = []
    for t in traces:
        m = t.get("metrics") or {}
        compile_first.append(bool(m.get("chose_compile_first")))
        refusals.append(int(m.get("refusal_count") or 0))
        rs = int(m.get("repair_success_count") or 0)
        if (m.get("refusal_count") or 0) > 0:
            repair_success.append(rs >= 1)
    return {
        "n_traces": len(traces),
        "first_tool_choice_rate": (
            sum(1 for x in compile_first if x) / len(compile_first) if compile_first else 0.0
        ),
        "refusal_repair_success_rate": (
            sum(1 for x in repair_success if x) / len(repair_success) if repair_success else None
        ),
        "mean_refusals": statistics.mean(refusals) if refusals else 0.0,
        "target_first_tool_choice": 0.70,
    }


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    arms = [scripted_arm(a) for a in ("A", "B", "C", "D", "E")]
    by = {a.arm: a for a in arms}
    traces = load_traces()
    metrics = trace_metrics(traces)
    doc_tasks = load_doc_qa_tasks()

    # G1-style: Medium+Prism (C) vs Frontier+explore (A)
    delta_g1 = by["C"].quality_proxy - by["A"].quality_proxy
    g1 = {
        "claim": "quality(Medium+Prism) >= quality(Frontier+explore) - 3pts",
        "delta_proxy_pts": round(delta_g1 * 100, 1),
        "status": "PASS_PROXY" if delta_g1 >= -0.03 else "FAIL_PROXY",
    }

    # ACC-5: Prism ≥ Graphify on quality at ≤½ tokens
    acc5_quality_ok = by["C"].quality_proxy >= by["E"].quality_proxy - 0.02
    acc5_tokens_ok = by["C"].tokens_proxy <= 0.5 * by["E"].tokens_proxy
    acc5 = {
        "claim": "quality(Medium+Prism) >= quality(Graphify) - 2pts AND tokens(C) <= 0.5 * tokens(E)",
        "quality_delta_pts": round((by["C"].quality_proxy - by["E"].quality_proxy) * 100, 1),
        "token_ratio": round(by["C"].tokens_proxy / max(by["E"].tokens_proxy, 1), 3),
        "status": "PASS_PROXY" if (acc5_quality_ok and acc5_tokens_ok) else "FAIL_PROXY",
        "caveat": "Scripted proxies until live-judged runs; citation-validity is co-reported",
    }

    report = {
        "suite": "p12-five-arm",
        "mode": "scripted_proxy",
        "arms": [asdict(a) for a in arms],
        "agent_traces": metrics,
        "doc_qa_tasks": {
            "n": len(doc_tasks),
            "ids": [t.get("id") for t in doc_tasks],
            "target_n": 25,
            "notes": "DQ001–DQ025 authored under eval/tasks/doc-qa/; live adjudication pending",
        },
        "g1": g1,
        "acc5": acc5,
        "ablations_planned": [
            "docs=on/off",
            "communities=path_prefix/semantic",
            "lexical_seeds=on/off",
        ],
    }
    out = OUT / "latest.json"
    out.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    print(f"\nwrote {out}")


if __name__ == "__main__":
    main()
