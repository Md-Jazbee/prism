#!/usr/bin/env python3
"""ACC-1 pack answerability proxy over Doc-QA gold (DQ001–DQ025).

Runs `prism compile` per task and checks whether necessary_spans appear in the
pack (fragment text or provenance paths) and forbidden_sources are absent.

This is a scripted proxy — it does NOT close the live-judged ACC-1 gate.

Usage:
  python eval/baselines/doc_qa_pack_answerability.py [--prism PATH] [--budget N]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TASKS = ROOT / "eval" / "tasks" / "doc-qa"
OUT = ROOT / "eval" / "baselines" / "doc-qa-answerability"
REPORT_MD = ROOT / "docs" / "eval" / "P12-ACC1-PACK-ANSWERABILITY.md"


def load_tasks() -> list[dict]:
    return [json.loads(p.read_text()) for p in sorted(TASKS.glob("DQ*.json"))]


def parse_cli_json(raw: str) -> dict:
    i = raw.find("{")
    if i < 0:
        raise ValueError(f"no JSON object in CLI output: {raw[:200]!r}")
    obj, _ = json.JSONDecoder().raw_decode(raw[i:])
    return obj


def compile_pack(prism: Path, question: str, budget: int, intent: str = "architecture") -> dict:
    cmd = [
        str(prism),
        "compile",
        "--intent",
        intent,
        "--budget",
        str(budget),
        question,
        str(ROOT),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT))
    raw = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode != 0 and "{" not in raw:
        return {"status": "error", "returncode": proc.returncode, "raw": raw[-500:]}
    return parse_cli_json(raw)


def pack_blob(data: dict) -> tuple[str, list[str], str | None]:
    """Return (haystack text, provenance paths, refusal code)."""
    if data.get("status") not in (None, "ok"):
        # refusal shapes vary
        code = data.get("code") or (data.get("error") or {}).get("code")
        if code:
            return "", [], str(code)
        if data.get("status") and data.get("status") != "ok":
            return "", [], str(data.get("status"))
    body = data.get("data") or data
    frags = body.get("fragments") or []
    texts: list[str] = []
    paths: list[str] = []
    for f in frags:
        texts.append(f.get("text") or "")
        for nid in (f.get("provenance") or {}).get("node_ids") or []:
            paths.append(str(nid))
        # node ids often encode paths: doc:README.md, doc:docs/foo.md
        m = re.match(r"^(?:doc|section|file):(.+?)(?:#.*)?$", str(f.get("id") or ""))
        if m:
            paths.append(m.group(1))
    for nid in paths:
        if nid.startswith("doc:") or nid.startswith("section:") or nid.startswith("file:"):
            paths.append(nid.split(":", 1)[1].split("#", 1)[0])
    hay = "\n".join(texts).lower()
    return hay, paths, None


def span_hit(span: str, hay: str, paths: list[str]) -> bool:
    s = span.replace("\\", "/").lower().strip()
    if not s:
        return False
    if s in hay:
        return True
    base = Path(s).name.lower()
    for p in paths:
        pl = p.replace("\\", "/").lower()
        if s in pl or pl.endswith(s) or base == Path(pl).name:
            return True
    # Filename mention in prose headers "(README.md)" etc.
    if base and base in hay:
        return True
    return False


def forbidden_hit(forbidden: list[str], hay: str, paths: list[str]) -> list[str]:
    hits = []
    for f in forbidden:
        fl = f.replace("\\", "/").lower().rstrip("/")
        if fl and (fl in hay or any(fl in p.replace("\\", "/").lower() for p in paths)):
            hits.append(f)
    return hits


def score_task(task: dict, pack: dict) -> dict:
    hay, paths, refusal = pack_blob(pack)
    necessary = task.get("necessary_spans") or []
    forbidden = task.get("forbidden_sources") or []
    if refusal:
        return {
            "id": task["id"],
            "ok": False,
            "refusal": refusal,
            "span_hits": [],
            "span_misses": necessary,
            "forbidden_hits": [],
            "tokens_used": None,
        }
    hits = [s for s in necessary if span_hit(s, hay, paths)]
    misses = [s for s in necessary if s not in hits]
    bad = forbidden_hit(forbidden, hay, paths)
    meta = (pack.get("data") or {}).get("meta") or {}
    # Answerable proxy: ≥1 necessary span present and no forbidden sources.
    ok = (not necessary or len(hits) >= 1) and not bad
    # Stricter: all necessary spans (reported separately).
    return {
        "id": task["id"],
        "ok": ok,
        "all_spans": bool(necessary) and len(misses) == 0,
        "refusal": None,
        "span_hits": hits,
        "span_misses": misses,
        "forbidden_hits": bad,
        "tokens_used": meta.get("tokens_used"),
        "thesis_role": next(
            (
                (f.get("why_included") or "")
                for f in ((pack.get("data") or {}).get("fragments") or [])
                if "product_thesis" in (f.get("roles") or [f.get("why_included")])
            ),
            None,
        ),
    }


def write_report(report: dict) -> None:
    lines = [
        "# P12 ACC-1 — Pack answerability proxy",
        "",
        f"**Generated:** {report['generated_at']}",
        f"**Prism:** `{report['prism']}`",
        f"**Budget:** {report['budget_tokens']}",
        f"**n:** {report['n']} · **answerable (≥1 necessary span, no forbidden):** "
        f"{report['answerable_rate']:.1%} · **all-spans:** {report['all_spans_rate']:.1%}",
        "",
        "> Scripted proxy only — does **not** close live-judged ACC-1 (≥80%).",
        "",
        "| Task | OK | All spans | Tokens | Misses |",
        "|---|---|---|---|---|",
    ]
    for r in report["results"]:
        misses = ", ".join(r.get("span_misses") or []) or "—"
        lines.append(
            f"| {r['id']} | {'✅' if r['ok'] else '❌'} | "
            f"{'✅' if r.get('all_spans') else '❌'} | {r.get('tokens_used') or '—'} | {misses} |"
        )
    lines.append("")
    REPORT_MD.write_text("\n".join(lines) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--prism",
        default=str(ROOT / "target" / "release" / "prism"),
        help="Path to prism binary",
    )
    ap.add_argument("--budget", type=int, default=4000)
    args = ap.parse_args()
    prism = Path(args.prism)
    OUT.mkdir(parents=True, exist_ok=True)

    tasks = load_tasks()
    results = []
    t0 = time.time()
    for task in tasks:
        pack = compile_pack(prism, task["question"], args.budget)
        results.append(score_task(task, pack))

    n = len(results)
    answerable = sum(1 for r in results if r["ok"])
    all_spans = sum(1 for r in results if r.get("all_spans"))
    report = {
        "suite": "p12-acc1-pack-answerability-proxy",
        "status": "PROXY",
        "note": "Does not close live-judged ACC-1 gate",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "prism": str(prism),
        "budget_tokens": args.budget,
        "n": n,
        "answerable": answerable,
        "answerable_rate": round(answerable / n, 4) if n else 0.0,
        "all_spans": all_spans,
        "all_spans_rate": round(all_spans / n, 4) if n else 0.0,
        "elapsed_s": round(time.time() - t0, 2),
        "results": results,
    }
    (OUT / "latest.json").write_text(json.dumps(report, indent=2) + "\n")
    write_report(report)
    print(json.dumps({k: report[k] for k in report if k != "results"}, indent=2))
    print(f"wrote {OUT / 'latest.json'}")
    print(f"wrote {REPORT_MD}")


if __name__ == "__main__":
    main()
