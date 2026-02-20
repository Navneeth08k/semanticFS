#!/usr/bin/env python3
"""
Deterministically split a retrieval golden suite into tune and holdout sets.

This keeps evaluation honest: tune on one set, report only holdout metrics.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List


def load_suite(path: Path) -> Dict[str, Any]:
    raw = path.read_text(encoding="utf-8")
    suite = json.loads(raw)
    if not isinstance(suite, dict):
        raise SystemExit(f"invalid suite object: {path}")
    queries = suite.get("queries")
    if not isinstance(queries, list) or not queries:
        raise SystemExit(f"suite has no queries: {path}")
    return suite


def split_queries(queries: List[Dict[str, Any]], tune_count: int) -> tuple[list, list]:
    if tune_count <= 0 or tune_count >= len(queries):
        raise SystemExit(
            f"tune_count must be between 1 and {len(queries) - 1}, got {tune_count}"
        )

    # Interleave (0,2,4,...) then (1,3,5,...) to spread nearby queries.
    ordered_idx = list(range(0, len(queries), 2)) + list(range(1, len(queries), 2))
    tune_idx = set(ordered_idx[:tune_count])

    tune = [q for i, q in enumerate(queries) if i in tune_idx]
    holdout = [q for i, q in enumerate(queries) if i not in tune_idx]
    return tune, holdout


def with_dataset_name(suite: Dict[str, Any], name: str, queries: list) -> Dict[str, Any]:
    return {
        "schema_version": suite.get("schema_version", 1),
        "dataset_name": name,
        "queries": queries,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Split golden suite into tune/holdout")
    ap.add_argument("--input", required=True, help="Input suite JSON")
    ap.add_argument("--tune-output", required=True, help="Tune output JSON")
    ap.add_argument("--holdout-output", required=True, help="Holdout output JSON")
    ap.add_argument(
        "--tune-count",
        type=int,
        default=10,
        help="Number of queries in tune split (default: 10)",
    )
    args = ap.parse_args()

    in_path = Path(args.input)
    tune_out = Path(args.tune_output)
    holdout_out = Path(args.holdout_output)

    suite = load_suite(in_path)
    queries = suite["queries"]
    tune_queries, holdout_queries = split_queries(queries, args.tune_count)

    base_name = str(suite.get("dataset_name", in_path.stem)).strip() or in_path.stem
    tune_name = f"{base_name}_tune_v1"
    holdout_name = f"{base_name}_holdout_v1"

    tune_suite = with_dataset_name(suite, tune_name, tune_queries)
    holdout_suite = with_dataset_name(suite, holdout_name, holdout_queries)

    tune_out.parent.mkdir(parents=True, exist_ok=True)
    holdout_out.parent.mkdir(parents=True, exist_ok=True)
    tune_out.write_text(json.dumps(tune_suite, indent=2), encoding="utf-8")
    holdout_out.write_text(json.dumps(holdout_suite, indent=2), encoding="utf-8")

    print(f"input={in_path.resolve()}")
    print(f"queries_total={len(queries)}")
    print(f"tune_queries={len(tune_queries)} output={tune_out.resolve()}")
    print(f"holdout_queries={len(holdout_queries)} output={holdout_out.resolve()}")
    print(f"dataset_tune={tune_name}")
    print(f"dataset_holdout={holdout_name}")


if __name__ == "__main__":
    main()
