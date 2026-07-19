"""Phase 0 eval harness skeleton — list tasks and smoke-check cards."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TASKS = ROOT / "tasks"


def load_tasks() -> list[dict]:
    cards = sorted(TASKS.glob("T*.json"))
    out = []
    for path in cards:
        data = json.loads(path.read_text(encoding="utf-8"))
        data["_path"] = str(path.relative_to(ROOT))
        out.append(data)
    return out


def cmd_smoke(_: argparse.Namespace) -> int:
    tasks = load_tasks()
    if len(tasks) < 20:
        print(f"FAIL: expected ≥20 tasks, found {len(tasks)}", file=sys.stderr)
        return 1
    required = {"id", "type", "repo", "question", "commit_sha", "accepted_answer_criteria"}
    for t in tasks:
        missing = required - set(t)
        if missing:
            print(f"FAIL: {t.get('id')} missing {missing}", file=sys.stderr)
            return 1
        if t["commit_sha"] == "PIN_ME":
            # Allowed in P0 stubs — warn once at end
            pass
    pinned = sum(1 for t in tasks if t["commit_sha"] != "PIN_ME")
    print(f"OK: {len(tasks)} gold tasks · {pinned} pinned SHAs · {len(tasks) - pinned} stubs await fixture freeze")
    print("Procedure: see eval/README.md — How we know P1 saved tokens")
    return 0


def cmd_list(_: argparse.Namespace) -> int:
    for t in load_tasks():
        print(f"{t['id']}\t{t['type']}\t{t['repo']}\t{t['question']}")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(prog="prism-eval")
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("smoke", help="Validate gold task pack v0").set_defaults(func=cmd_smoke)
    sub.add_parser("list", help="List task cards").set_defaults(func=cmd_list)
    args = parser.parse_args()
    raise SystemExit(args.func(args))


if __name__ == "__main__":
    main()
