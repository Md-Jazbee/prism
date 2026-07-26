#!/usr/bin/env python3
"""ACC-3 seed-grounding precision sample.

Scores `eval/tasks/seed-grounding/AG*.json` against a live `.prism/graph.sqlite`
using the same thresholds as `prism-store` lexical grounding (exact name = 100).

Usage:
  python eval/baselines/seed_grounding_score.py [--db PATH]
"""

from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TASKS = ROOT / "eval" / "tasks" / "seed-grounding"
OUT = ROOT / "eval" / "baselines" / "seed-grounding"
MIN_GROUND = 70


def score_anchor(conn: sqlite3.Connection, anchor: str) -> int:
    a = anchor.strip()
    if not a:
        return 0
    row = conn.execute(
        "SELECT 1 FROM nodes WHERE name = ? LIMIT 1", (a,)
    ).fetchone()
    if row:
        return 100
    if "/" in a or "." in a:
        row = conn.execute(
            "SELECT 1 FROM nodes WHERE kind = 'File' AND (file_path = ? OR id = ?) LIMIT 1",
            (a, f"file:{a}"),
        ).fetchone()
        if row:
            return 90
    row = conn.execute(
        "SELECT 1 FROM nodes WHERE name LIKE ? COLLATE NOCASE "
        "AND id NOT LIKE 'unresolved:%' LIMIT 1",
        (f"{a}%",),
    ).fetchone()
    if row:
        return 75
    if len(a) >= 3:
        row = conn.execute(
            "SELECT 1 FROM nodes WHERE name LIKE ? COLLATE NOCASE "
            "AND id NOT LIKE 'unresolved:%' LIMIT 1",
            (f"%{a}%",),
        ).fetchone()
        if row:
            return 50
    return 0


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--db",
        default=str(ROOT / ".prism" / "graph.sqlite"),
        help="Path to graph.sqlite",
    )
    args = ap.parse_args()
    db = Path(args.db)
    OUT.mkdir(parents=True, exist_ok=True)

    if not db.exists():
        report = {
            "suite": "p12-acc3-seed-grounding",
            "status": "SKIP",
            "reason": f"no index at {db} — run prism index .",
            "n": 0,
            "precision": None,
        }
        (OUT / "latest.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
        return

    conn = sqlite3.connect(db)
    tasks = []
    for p in sorted(TASKS.glob("AG*.json")):
        tasks.append(json.loads(p.read_text()))

    rows = []
    tp = fp = tn = fn = 0
    for t in tasks:
        score = score_anchor(conn, t["anchor"])
        grounded = score >= MIN_GROUND
        must = bool(t.get("must_ground"))
        ok = grounded == must if not must else grounded
        # For must_ground=true: need grounded. For false: either refuse (not grounded) OR weak ok.
        if must:
            if grounded:
                tp += 1
                ok = True
            else:
                fn += 1
                ok = False
        else:
            if grounded:
                # Unexpected strong ground for negative / soft cases — count as FP only for TotallyFake*
                if t["anchor"].startswith("TotallyFake"):
                    fp += 1
                    ok = False
                else:
                    tn += 1
                    ok = True
            else:
                tn += 1
                ok = True
        rows.append(
            {
                "id": t["id"],
                "anchor": t["anchor"],
                "score": score,
                "grounded": grounded,
                "must_ground": must,
                "ok": ok,
            }
        )

    judged = [r for r in rows if r["must_ground"] or r["anchor"].startswith("TotallyFake")]
    correct = sum(1 for r in judged if r["ok"])
    precision = correct / len(judged) if judged else 0.0
    status = "PASS" if precision >= 0.90 and judged else ("FAIL" if judged else "SKIP")

    report = {
        "suite": "p12-acc3-seed-grounding",
        "db": str(db),
        "n_tasks": len(tasks),
        "n_judged": len(judged),
        "tp": tp,
        "fn": fn,
        "fp": fp,
        "tn": tn,
        "precision": round(precision, 4),
        "target": 0.90,
        "status": status,
        "rows": rows,
    }
    (OUT / "latest.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in report if k != "rows"}, indent=2))
    print(f"\nwrote {OUT / 'latest.json'} status={status} precision={precision:.2%}")


if __name__ == "__main__":
    main()
