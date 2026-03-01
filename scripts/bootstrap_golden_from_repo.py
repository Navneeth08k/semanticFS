#!/usr/bin/env python3
"""
Generate a bootstrap retrieval golden file for an arbitrary repository.

This is intentionally lightweight: it extracts likely symbol names from code
files and maps each query to an expected path that defines the symbol.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    tomllib = None


ALLOWED_EXTENSIONS = {
    ".py",
    ".rs",
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".dart",
    ".java",
    ".go",
    ".c",
    ".cc",
    ".cpp",
    ".h",
    ".hpp",
}

SKIP_DIR_NAMES = {
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".vscode",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".cache",
    ".dart_tool",
    ".pytest_cache",
    ".semanticfs",
    "node_modules",
    "coverage",
    "dist",
    "build",
    "out",
    "target",
    ".venv",
    "venv",
    "__pycache__",
}

MAX_FILE_BYTES = 512 * 1024
SKIP_SYMBOLS = {
    "main",
    "test",
    "tests",
    "init",
    "setup",
    "teardown",
    "run",
    "load",
    "save",
    "build",
    "export",
    "decorator",
}


LANG_PATTERNS: Sequence[Tuple[Sequence[str], Sequence[re.Pattern[str]]]] = [
    (
        [".py"],
        [
            re.compile(r"^\s*(?:async\s+)?def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("),
            re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
        ],
    ),
    (
        [".rs"],
        [
            re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(r"^\s*(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(r"^\s*(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(r"^\s*(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
        ],
    ),
    (
        [".ts", ".tsx", ".js", ".jsx"],
        [
            re.compile(
                r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
            ),
            re.compile(
                r"^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)\b"
            ),
            re.compile(
                r"^\s*(?:export\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:async\s*)?\("
            ),
        ],
    ),
    (
        [".dart"],
        [
            re.compile(r"^\s*(?:abstract\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(r"^\s*enum\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(r"^\s*mixin\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
            re.compile(
                r"^\s*(?:[A-Za-z_<>\[\]\?,\s]+\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{"
            ),
        ],
    ),
    (
        [".java"],
        [
            re.compile(
                r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:class|interface|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b"
            ),
            re.compile(
                r"^\s*(?:public|private|protected)\s+(?:static\s+)?[A-Za-z_<>\[\], ?]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("
            ),
        ],
    ),
    (
        [".go"],
        [
            re.compile(
                r"^\s*func\s+(?:\([^)]+\)\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\("
            )
        ],
    ),
    (
        [".c", ".cc", ".cpp", ".h", ".hpp"],
        [
            re.compile(
                r"^\s*[A-Za-z_][A-Za-z0-9_*\s]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{?\s*$"
            )
        ],
    ),
]


def parser_for_extension(ext: str) -> Sequence[re.Pattern[str]]:
    for exts, patterns in LANG_PATTERNS:
        if ext in exts:
            return patterns
    return ()


class PathFilter:
    def __init__(self, allow_patterns: Sequence[str], deny_patterns: Sequence[str]):
        self.allow_patterns = [
            compile_glob_regex(normalize_pattern(p)) for p in allow_patterns if p
        ]
        self.deny_patterns = [
            compile_glob_regex(normalize_pattern(p)) for p in deny_patterns if p
        ]

    @classmethod
    def from_config(cls, config_path: Path) -> "PathFilter":
        try:
            raw = config_path.read_text(encoding="utf-8")
        except OSError as exc:
            raise SystemExit(f"failed to read config file: {config_path}: {exc}") from exc
        allow_patterns, deny_patterns = load_filter_patterns(raw, config_path)
        return cls(allow_patterns, deny_patterns)

    def allows(self, relative_path: str) -> bool:
        rel = relative_path.strip("/")
        if not rel:
            return False
        if self.allow_patterns and not any(
            pattern.match(rel) for pattern in self.allow_patterns
        ):
            return False
        if any(pattern.match(rel) for pattern in self.deny_patterns):
            return False
        return True


def string_list(value: object) -> List[str]:
    if not isinstance(value, list):
        return []
    out: List[str] = []
    for item in value:
        if isinstance(item, str):
            trimmed = item.strip()
            if trimmed:
                out.append(trimmed)
    return out


def load_filter_patterns(raw: str, config_path: Path) -> Tuple[List[str], List[str]]:
    if tomllib is not None:
        try:
            parsed = tomllib.loads(raw)
        except tomllib.TOMLDecodeError as exc:
            raise SystemExit(f"failed to parse TOML config: {config_path}: {exc}") from exc
        filter_cfg = parsed.get("filter")
        if not isinstance(filter_cfg, dict):
            return [], []
        return (
            string_list(filter_cfg.get("allow_roots")),
            string_list(filter_cfg.get("deny_globs")),
        )
    return parse_filter_patterns_fallback(raw)


def parse_filter_patterns_fallback(raw: str) -> Tuple[List[str], List[str]]:
    match = re.search(r"(?ms)^\[filter\]\s*(.*?)(?=^\[|\Z)", raw)
    if not match:
        return [], []
    section = match.group(1)
    allow_patterns = parse_string_array(section, "allow_roots")
    deny_patterns = parse_string_array(section, "deny_globs")
    return allow_patterns, deny_patterns


def parse_string_array(section: str, key: str) -> List[str]:
    match = re.search(rf"(?ms)^{re.escape(key)}\s*=\s*\[(.*?)\]", section)
    if not match:
        return []
    body = match.group(1)
    out: List[str] = []
    for raw_value in re.findall(r'"((?:[^"\\]|\\.)*)"', body):
        out.append(json.loads(f'"{raw_value}"'))
    return out


def normalize_pattern(pattern: str) -> str:
    return pattern.strip().replace("\\", "/").lstrip("./")


def compile_glob_regex(pattern: str) -> re.Pattern[str]:
    out = ["^"]
    i = 0
    while i < len(pattern):
        char = pattern[i]
        if char == "*":
            if i + 1 < len(pattern) and pattern[i + 1] == "*":
                i += 2
                if i < len(pattern) and pattern[i] == "/":
                    out.append("(?:.*/)?")
                    i += 1
                else:
                    out.append(".*")
                continue
            out.append("[^/]*")
            i += 1
            continue
        if char == "?":
            out.append("[^/]")
            i += 1
            continue
        out.append(re.escape(char))
        i += 1
    out.append("$")
    return re.compile("".join(out))


def iter_source_files(root: Path, path_filter: PathFilter | None = None) -> Iterable[Path]:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIR_NAMES]
        for name in filenames:
            path = Path(dirpath) / name
            rel = path.relative_to(root).as_posix()
            ext = path.suffix.lower()
            if ext not in ALLOWED_EXTENSIONS:
                continue
            if path_filter and not path_filter.allows(rel):
                continue
            try:
                size = path.stat().st_size
            except OSError:
                continue
            if size > MAX_FILE_BYTES:
                continue
            yield path


def iter_git_tracked_source_files(
    root: Path, path_filter: PathFilter | None = None
) -> Iterable[Path]:
    try:
        proc = subprocess.run(
            ["git", "-C", str(root), "ls-files"],
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError as exc:
        stderr = (exc.stderr or "").strip()
        detail = f": {stderr}" if stderr else ""
        raise SystemExit(f"git ls-files failed for {root}{detail}") from exc

    for rel in proc.stdout.splitlines():
        rel = rel.strip()
        if not rel:
            continue
        norm_rel = rel.replace("\\", "/").strip("/")
        if not norm_rel:
            continue
        ext = Path(norm_rel).suffix.lower()
        if ext not in ALLOWED_EXTENSIONS:
            continue
        if path_filter and not path_filter.allows(norm_rel):
            continue
        path = root / norm_rel
        if not path.is_file():
            continue
        try:
            size = path.stat().st_size
        except OSError:
            continue
        if size > MAX_FILE_BYTES:
            continue
        yield path


def extract_symbols(path: Path, repo_root: Path, min_len: int) -> List[Dict[str, str]]:
    ext = path.suffix.lower()
    patterns = parser_for_extension(ext)
    if not patterns:
        return []
    try:
        raw = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return []

    rel = path.relative_to(repo_root).as_posix()
    out: List[Dict[str, str]] = []
    seen: set[str] = set()
    for line in raw.splitlines():
        for pat in patterns:
            m = pat.match(line)
            if not m:
                continue
            symbol = m.group(1).strip()
            if len(symbol) < min_len:
                continue
            if symbol.startswith("__") and symbol.endswith("__"):
                continue
            if symbol.lower() in SKIP_SYMBOLS:
                continue
            if symbol in seen:
                continue
            seen.add(symbol)
            out.append({"symbol": symbol, "path": rel})
            break
    return out


def choose_queries(
    candidates: Sequence[Dict[str, str]], max_queries: int
) -> List[Dict[str, str]]:
    if max_queries <= 0:
        return []

    selected: List[Dict[str, str]] = []
    used_symbols: set[str] = set()
    used_paths: set[str] = set()

    # First pass: maximize path coverage.
    for c in candidates:
        if len(selected) >= max_queries:
            break
        sym = c["symbol"]
        path = c["path"]
        if sym in used_symbols or path in used_paths:
            continue
        selected.append(c)
        used_symbols.add(sym)
        used_paths.add(path)

    # Second pass: fill remaining slots with unique symbols.
    for c in candidates:
        if len(selected) >= max_queries:
            break
        sym = c["symbol"]
        if sym in used_symbols:
            continue
        selected.append(c)
        used_symbols.add(sym)

    return selected


def build_fixture(
    dataset_name: str, selected: Sequence[Dict[str, str]]
) -> Dict[str, object]:
    queries = []
    for idx, item in enumerate(selected, start=1):
        queries.append(
            {
                "id": f"b{idx:02d}",
                "query": item["symbol"],
                "expected_paths": [item["path"]],
                "symbol_query": True,
            }
        )
    return {
        "schema_version": 1,
        "dataset_name": dataset_name,
        "queries": queries,
    }


def default_dataset_name(repo_root: Path) -> str:
    stem = re.sub(r"[^a-zA-Z0-9]+", "_", repo_root.name).strip("_").lower()
    if not stem:
        stem = "repo"
    return f"{stem}_bootstrap_v1"


def main() -> None:
    ap = argparse.ArgumentParser(description="Generate bootstrap golden queries from a repo")
    ap.add_argument("--repo-root", required=True, help="Path to repository root")
    ap.add_argument("--output", required=True, help="Output JSON file path")
    ap.add_argument("--dataset-name", default="", help="Dataset name override")
    ap.add_argument(
        "--config",
        default="",
        help="Optional TOML config path; if set, applies filter.allow_roots and filter.deny_globs",
    )
    ap.add_argument(
        "--git-tracked-only",
        action="store_true",
        help="Use `git ls-files` instead of walking the full tree (faster for large repos)",
    )
    ap.add_argument(
        "--max-queries",
        type=int,
        default=20,
        help="Max query count to emit (default: 20)",
    )
    ap.add_argument(
        "--min-symbol-len",
        type=int,
        default=4,
        help="Minimum symbol length to include (default: 4)",
    )
    args = ap.parse_args()

    repo_root = Path(args.repo_root).resolve()
    if not repo_root.exists() or not repo_root.is_dir():
        raise SystemExit(f"repo root not found or not a directory: {repo_root}")

    dataset_name = args.dataset_name.strip() or default_dataset_name(repo_root)
    path_filter: PathFilter | None = None
    if args.config.strip():
        config_path = Path(args.config).resolve()
        path_filter = PathFilter.from_config(config_path)

    all_candidates: List[Dict[str, str]] = []
    iterator = (
        iter_git_tracked_source_files(repo_root, path_filter)
        if args.git_tracked_only
        else iter_source_files(repo_root, path_filter)
    )
    for path in iterator:
        all_candidates.extend(extract_symbols(path, repo_root, args.min_symbol_len))

    # Stable ordering: file path then symbol name.
    all_candidates.sort(key=lambda c: (c["path"], c["symbol"].lower(), c["symbol"]))
    selected = choose_queries(all_candidates, args.max_queries)
    if not selected:
        raise SystemExit(
            "no symbols extracted for bootstrap fixture; try lowering --min-symbol-len"
        )

    fixture = build_fixture(dataset_name, selected)
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(fixture, indent=2), encoding="utf-8")

    print(f"repo_root={repo_root}")
    print(f"dataset_name={dataset_name}")
    if args.config.strip():
        print(f"config={Path(args.config).resolve()}")
    print(f"path_mode={'git_tracked_only' if args.git_tracked_only else 'walk'}")
    print(f"candidates={len(all_candidates)}")
    print(f"queries={len(selected)}")
    print(f"output={out_path.resolve()}")


if __name__ == "__main__":
    main()
