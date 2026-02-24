#!/usr/bin/env python3
"""
Build acceptance-grade tune/holdout suites from a larger bootstrap suite.

Output guarantees:
1. tune and holdout each have `split_size` queries.
2. each split contains mixed symbol and non-symbol queries.
3. deterministic selection and IDs for stable reruns.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any, Dict, Iterable, List, Set, Tuple


DEFAULT_GENERIC_SYMBOLS = {
    "run",
    "main",
    "init",
    "setup",
    "test",
    "tests",
    "build",
    "create",
    "update",
    "delete",
    "save",
    "load",
    "reset",
    "step",
    "lookup",
    "render",
    "handler",
}


def load_suite(path: Path) -> Dict[str, Any]:
    raw = path.read_text(encoding="utf-8")
    suite = json.loads(raw)
    if not isinstance(suite, dict):
        raise SystemExit(f"invalid suite object: {path}")
    queries = suite.get("queries")
    if not isinstance(queries, list) or not queries:
        raise SystemExit(f"suite has no queries: {path}")
    return suite


def normalize_query_text(query: str) -> str:
    return re.sub(r"\s+", " ", query.strip()).lower()


def symbol_to_phrase(symbol: str) -> str:
    s = symbol.strip()
    if not s:
        return s
    s = s.lstrip("_")
    # camelCase / PascalCase -> space separated
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", s)
    # snake_case / kebab-case -> space separated
    s = s.replace("_", " ").replace("-", " ")
    s = re.sub(r"\s+", " ", s).strip().lower()
    return s


def query_terms(query: str) -> List[str]:
    return [t for t in re.split(r"[^a-zA-Z0-9]+", query.lower()) if t]


def is_easy_symbol_query(query: str, min_symbol_chars: int, generic_terms: Set[str]) -> bool:
    q = normalize_query_text(query).lstrip("_")
    if len(q) < min_symbol_chars:
        return True
    if q in generic_terms:
        return True
    if q.isdigit():
        return True
    return False


def filter_symbols_for_quality(
    queries: List[Dict[str, Any]],
    min_symbol_chars: int,
    generic_terms: Set[str],
    allow_ambiguous_symbols: bool,
) -> List[Dict[str, Any]]:
    norm_to_paths: Dict[str, Set[str]] = {}
    for q in queries:
        norm = normalize_query_text(q["query"])
        path = q["expected_paths"][0]
        norm_to_paths.setdefault(norm, set()).add(path)

    out: List[Dict[str, Any]] = []
    for q in queries:
        norm = normalize_query_text(q["query"])
        if not allow_ambiguous_symbols and len(norm_to_paths.get(norm, set())) > 1:
            continue
        if is_easy_symbol_query(q["query"], min_symbol_chars, generic_terms):
            continue
        out.append(q)
    return out


def dedupe_queries(queries: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    out: List[Dict[str, Any]] = []
    seen: set[Tuple[str, str]] = set()
    for q in queries:
        query = str(q.get("query", "")).strip()
        expected = q.get("expected_paths") or []
        if not query or not isinstance(expected, list) or not expected:
            continue
        key = (normalize_query_text(query), str(expected[0]))
        if key in seen:
            continue
        seen.add(key)
        out.append(
            {
                "query": query,
                "expected_paths": [str(p) for p in expected],
                "symbol_query": bool(q.get("symbol_query", False)),
            }
        )
    return out


def interleave_indexes(n: int) -> List[int]:
    return list(range(0, n, 2)) + list(range(1, n, 2))


def build_split(
    symbols: List[Dict[str, Any]],
    split_size: int,
    non_symbol_count: int,
    split_tag: str,
    min_non_symbol_chars: int,
    min_non_symbol_tokens: int,
    generic_terms: Set[str],
) -> List[Dict[str, Any]]:
    symbol_count = split_size - non_symbol_count
    if symbol_count <= 0:
        raise SystemExit("split_size must be greater than non_symbol_count")
    if len(symbols) < symbol_count:
        raise SystemExit(
            f"not enough symbols for {split_tag}: need {symbol_count}, have {len(symbols)}"
        )

    selected_symbols = symbols[:symbol_count]
    queries: List[Dict[str, Any]] = []
    used_query_text: set[str] = set()

    # Add symbol queries first.
    for idx, item in enumerate(selected_symbols, start=1):
        query = item["query"]
        norm = normalize_query_text(query)
        if norm in used_query_text:
            continue
        used_query_text.add(norm)
        queries.append(
            {
                "id": f"{split_tag}s{idx:03d}",
                "query": query,
                "expected_paths": item["expected_paths"],
                "symbol_query": True,
            }
        )

    # Add non-symbol phrase variants.
    ns_added = 0
    ns_idx = 1
    for item in selected_symbols:
        if ns_added >= non_symbol_count:
            break
        phrase = symbol_to_phrase(item["query"])
        if len(phrase) < min_non_symbol_chars:
            continue
        terms = query_terms(phrase)
        if len(terms) < min_non_symbol_tokens:
            continue
        if any(t in generic_terms for t in terms):
            continue
        norm = normalize_query_text(phrase)
        if norm in used_query_text:
            continue
        # Require an actual transformation from the original symbol.
        if norm == normalize_query_text(item["query"]):
            continue
        used_query_text.add(norm)
        queries.append(
            {
                "id": f"{split_tag}n{ns_idx:03d}",
                "query": phrase,
                "expected_paths": item["expected_paths"],
                "symbol_query": False,
            }
        )
        ns_added += 1
        ns_idx += 1

    if len(queries) < split_size:
        raise SystemExit(
            f"unable to build {split_tag} split to size {split_size}; built {len(queries)}"
        )

    # Stable truncation in case symbol dedupe behavior changed.
    return queries[:split_size]


def apply_expected_path_overrides(
    queries: List[Dict[str, Any]], overrides: Dict[str, List[str]]
) -> None:
    if not overrides:
        return
    for q in queries:
        key = normalize_query_text(q["query"])
        if key in overrides:
            q["expected_paths"] = overrides[key]


def parse_overrides(items: List[str]) -> Dict[str, List[str]]:
    out: Dict[str, List[str]] = {}
    for item in items:
        if "=" not in item:
            raise SystemExit(f"invalid override (expected query=path1;path2): {item}")
        query_part, paths_part = item.split("=", 1)
        query = normalize_query_text(query_part)
        paths = [p.strip() for p in paths_part.split(";") if p.strip()]
        if not query or not paths:
            raise SystemExit(f"invalid override (empty query/paths): {item}")
        out[query] = paths
    return out


def write_suite(path: Path, dataset_name: str, queries: List[Dict[str, Any]]) -> None:
    payload = {
        "schema_version": 1,
        "dataset_name": dataset_name,
        "queries": queries,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Build curated mixed tune/holdout suites")
    ap.add_argument("--input", required=True, help="Input bootstrap suite JSON")
    ap.add_argument("--tune-output", required=True, help="Tune output JSON")
    ap.add_argument("--holdout-output", required=True, help="Holdout output JSON")
    ap.add_argument(
        "--split-size",
        type=int,
        default=40,
        help="Queries per split (default: 40)",
    )
    ap.add_argument(
        "--non-symbol-per-split",
        type=int,
        default=10,
        help="Non-symbol query count per split (default: 10)",
    )
    ap.add_argument(
        "--dataset-prefix",
        default="",
        help="Dataset prefix override (default derived from input dataset_name)",
    )
    ap.add_argument(
        "--override",
        action="append",
        default=[],
        help="Expected path override in form query=path1;path2",
    )
    ap.add_argument(
        "--min-symbol-chars",
        type=int,
        default=5,
        help="Minimum symbol query text length after normalization (default: 5)",
    )
    ap.add_argument(
        "--min-non-symbol-chars",
        type=int,
        default=8,
        help="Minimum generated non-symbol phrase length (default: 8)",
    )
    ap.add_argument(
        "--min-non-symbol-tokens",
        type=int,
        default=2,
        help="Minimum generated non-symbol phrase token count (default: 2)",
    )
    ap.add_argument(
        "--allow-ambiguous-symbols",
        action="store_true",
        help="Allow same normalized symbol query to map to multiple expected paths",
    )
    ap.add_argument(
        "--generic-symbols",
        default=",".join(sorted(DEFAULT_GENERIC_SYMBOLS)),
        help="Comma-separated generic/easy terms to exclude (default built-in list)",
    )
    args = ap.parse_args()

    in_path = Path(args.input)
    tune_out = Path(args.tune_output)
    holdout_out = Path(args.holdout_output)

    suite = load_suite(in_path)
    deduped = dedupe_queries(suite["queries"])
    if not deduped:
        raise SystemExit("no valid queries after dedupe")

    generic_terms = {
        t.strip().lower() for t in args.generic_symbols.split(",") if t.strip()
    }
    filtered = filter_symbols_for_quality(
        deduped,
        min_symbol_chars=max(1, args.min_symbol_chars),
        generic_terms=generic_terms,
        allow_ambiguous_symbols=args.allow_ambiguous_symbols,
    )
    if not filtered:
        raise SystemExit("no queries left after quality filtering")

    split_size = args.split_size
    non_symbol = args.non_symbol_per_split
    symbol_per_split = split_size - non_symbol
    if symbol_per_split <= 0:
        raise SystemExit("split-size must be greater than non-symbol-per-split")

    needed_symbols = symbol_per_split * 2
    if len(filtered) < needed_symbols:
        raise SystemExit(
            f"insufficient source queries after quality filtering: need at least {needed_symbols}, found {len(filtered)}"
        )

    # Deterministic spread by interleaving sorted candidates.
    filtered.sort(
        key=lambda q: (
            q["expected_paths"][0].lower(),
            q["query"].lower(),
            q["query"],
        )
    )
    order = interleave_indexes(len(filtered))
    ordered = [filtered[i] for i in order]
    selected = ordered[:needed_symbols]
    tune_symbols = selected[:symbol_per_split]
    holdout_symbols = selected[symbol_per_split:needed_symbols]

    tune_queries = build_split(
        tune_symbols,
        split_size,
        non_symbol,
        "t",
        min_non_symbol_chars=max(1, args.min_non_symbol_chars),
        min_non_symbol_tokens=max(1, args.min_non_symbol_tokens),
        generic_terms=generic_terms,
    )
    holdout_queries = build_split(
        holdout_symbols,
        split_size,
        non_symbol,
        "h",
        min_non_symbol_chars=max(1, args.min_non_symbol_chars),
        min_non_symbol_tokens=max(1, args.min_non_symbol_tokens),
        generic_terms=generic_terms,
    )

    overrides = parse_overrides(args.override)
    apply_expected_path_overrides(tune_queries, overrides)
    apply_expected_path_overrides(holdout_queries, overrides)

    base = args.dataset_prefix.strip()
    if not base:
        raw_name = str(suite.get("dataset_name", in_path.stem)).strip() or in_path.stem
        base = re.sub(r"_bootstrap.*$", "", raw_name)
        base = re.sub(r"[^a-zA-Z0-9_]+", "_", base).strip("_").lower()
        if not base:
            base = "repo"

    tune_name = f"{base}_curated_tune_v1"
    holdout_name = f"{base}_curated_holdout_v1"
    write_suite(tune_out, tune_name, tune_queries)
    write_suite(holdout_out, holdout_name, holdout_queries)

    print(f"input={in_path.resolve()}")
    print(f"source_queries={len(deduped)}")
    print(f"filtered_queries={len(filtered)}")
    print(
        "split_size={} symbol_per_split={} non_symbol_per_split={}".format(
            split_size, symbol_per_split, non_symbol
        )
    )
    print(f"tune_queries={len(tune_queries)} output={tune_out.resolve()}")
    print(f"holdout_queries={len(holdout_queries)} output={holdout_out.resolve()}")
    print(f"dataset_tune={tune_name}")
    print(f"dataset_holdout={holdout_name}")


if __name__ == "__main__":
    main()
