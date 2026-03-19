#!/usr/bin/env python3
"""
Multi-repo head-to-head: Naive Claude Code vs Claude Code + SemanticFS.

Runs across 4 repos of different sizes:
  - prizePicksAI   (tiny  — 5 real source files,  ~4 total with no deps)
  - KalshiTradingAlgo (small  — 17 real Python files, ~26k total with .venv)
  - syntaxless     (medium — 95+ TS files,         ~33k total with node_modules)
  - buckit         (large  — 70+ JS files,         ~50k total with node_modules)

For each repo: builds a SemanticFS index, then runs repo-specific tasks in
both NAIVE (Bash only) and SEMANTICFS (Bash + MCP search_codebase) modes.

Usage:
  python scripts/multi_repo_h2h.py [--repos prizePicksAI KalshiTradingAlgo ...]
                                    [--skip-index] [--naive-only] [--sfs-only]
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

# ── Paths ──────────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).parent.parent  # semanticFS project root
SEMANTICFS_EXE = REPO_ROOT / "target" / "release" / "semanticfs.exe"
CLAUDE_EXE = Path(
    "C:/Users/navneeth/AppData/Local/Packages"
    "/Claude_pzs8sxrjxfjjc/LocalCache/Roaming/Claude/claude-code/2.1.76/claude.exe"
)
BENCH_DIR = REPO_ROOT / ".semanticfs" / "bench"
DB_BASE   = Path("C:/Users/navneeth/AppData/Local/Temp/semanticfs_bench")
TASK_TIMEOUT_SECS = 240

# ── Repo + Task Definitions ────────────────────────────────────────────────────

REPOS = [
    {
        "id": "prizePicksAI",
        "name": "PrizePicksAI",
        "size_label": "Tiny (5 real source files, no deps)",
        "path": Path("C:/Users/navneeth/Desktop/NavneethThings/Projects/prizePicksAI"),
        "deny_globs": ["**/__pycache__/**", "**/*.pyc", "**/.git/**"],
        "system_preamble": (
            "You are a developer exploring the prizePicksAI codebase — "
            "a Python sports betting data scraper that fetches prop odds from "
            "DraftKings, PrizePicks, and ESPN. It has only 5 source files. "
            "Answer the question by finding the relevant code. "
            "Be concise: state the file, the function, and the key logic."
        ),
        "tasks": [
            {
                "id": "T1",
                "description": "Find DraftKings scraper function",
                "prompt": (
                    "In prizePicksAI, where is the code that scrapes DraftKings "
                    "sportsbook tables? Show me the function name and file."
                ),
                "correct_files": ["dkScraper.py"],
            },
            {
                "id": "T2",
                "description": "Find prop-odds NBA game API call",
                "prompt": (
                    "In prizePicksAI, which function calls the prop-odds API "
                    "to retrieve NBA games for a specific date? What file is it in?"
                ),
                "correct_files": ["prophits.py"],
            },
            {
                "id": "T3",
                "description": "Find PrizePicks bot-bypass scraper",
                "prompt": (
                    "The prizePicksAI project scrapes PrizePicks. "
                    "Which file handles this and what library is used to bypass "
                    "bot detection?"
                ),
                "correct_files": ["prizepicksScraper.py"],
            },
            {
                "id": "T4",
                "description": "Find ESPN box score parsing",
                "prompt": (
                    "In prizePicksAI, find the code that parses ESPN box score pages "
                    "to get player stats. What file, function, and HTML parsing library?"
                ),
                "correct_files": ["nbaTrueStats.py"],
            },
        ],
    },
    {
        "id": "KalshiTradingAlgo",
        "name": "KalshiTradingAlgo",
        "size_label": "Small-Medium (17 real Python files, ~26k total with .venv)",
        "path": Path("C:/Users/navneeth/Desktop/NavneethThings/Projects/KalshiTradingAlgo"),
        "deny_globs": [
            "**/.venv/**", "**/venv/**", "**/__pycache__/**",
            "**/*.pyc", "**/.git/**",
        ],
        "system_preamble": (
            "You are a developer exploring the KalshiTradingAlgo codebase — "
            "a Python automated prediction market trading bot that uses Gemini AI "
            "for sentiment analysis and trades on the Kalshi platform. "
            "It has about 17 core source files plus a .venv directory with thousands "
            "of third-party packages — avoid exploring .venv. "
            "Answer the question concisely: file, function, key logic."
        ),
        "tasks": [
            {
                "id": "T1",
                "description": "Find main trading pipeline orchestrator",
                "prompt": (
                    "In KalshiTradingAlgo, where is the function that orchestrates "
                    "the full trading pipeline from sentiment analysis through "
                    "trade execution? What file and function?"
                ),
                "correct_files": ["trading_algorithm.py"],
            },
            {
                "id": "T2",
                "description": "Find Gemini sentiment analysis call",
                "prompt": (
                    "KalshiTradingAlgo uses Gemini AI for sentiment analysis. "
                    "Which file and function calls the Gemini API to get "
                    "trending entities and sentiment scores?"
                ),
                "correct_files": ["sentiment_engine.py"],
            },
            {
                "id": "T3",
                "description": "Find API key validation logic",
                "prompt": (
                    "In KalshiTradingAlgo, which class or function validates "
                    "the required API keys (Gemini, Kalshi)? What file is it in?"
                ),
                "correct_files": ["config.py"],
            },
            {
                "id": "T4",
                "description": "Find trading signal generation",
                "prompt": (
                    "Where in KalshiTradingAlgo are buy/sell trading signals generated? "
                    "What file, class, and method produces the signal, and what inputs "
                    "does it use (sentiment score, price, etc.)?"
                ),
                "correct_files": ["signal_generator.py"],
            },
        ],
    },
    {
        "id": "syntaxless",
        "name": "syntaxless",
        "size_label": "Medium (95+ TS files, ~33k total with node_modules)",
        "path": Path("C:/Users/navneeth/Desktop/NavneethThings/Projects/syntaxless"),
        "deny_globs": [
            "**/node_modules/**", "**/.next/**", "**/dist/**",
            "**/build/**", "**/.git/**", "**/*.lock",
        ],
        "system_preamble": (
            "You are a developer exploring the syntaxless codebase — "
            "a Next.js web IDE that translates natural language to code using "
            "Gemini AI, with Supabase for auth and storage. "
            "The repo has a large node_modules directory — avoid it. "
            "Focus on app/, lib/, components/ source files. "
            "Answer concisely: file path, function/component, key logic."
        ),
        "tasks": [
            {
                "id": "T1",
                "description": "Find natural language to code API route",
                "prompt": (
                    "In the syntaxless project, find the API endpoint that "
                    "translates natural language instructions into code. "
                    "What file is it in and how does it call Gemini?"
                ),
                "correct_files": ["translate/route.ts", "translate/route.js"],
            },
            {
                "id": "T2",
                "description": "Find Supabase client initialization",
                "prompt": (
                    "In syntaxless, where is the Supabase client initialized "
                    "for use in browser components? What file and how does it "
                    "handle sessions?"
                ),
                "correct_files": ["supabase/client.ts", "supabase/client.js", "supabase/server.ts"],
            },
            {
                "id": "T3",
                "description": "Find main IDE editor page component",
                "prompt": (
                    "Find the main IDE editor page in syntaxless. "
                    "What file is it in and what code editor library does it use "
                    "for syntax highlighting?"
                ),
                "correct_files": ["ide", "projectId"],
            },
            {
                "id": "T4",
                "description": "Find route protection middleware",
                "prompt": (
                    "In syntaxless, how are authenticated routes protected? "
                    "Find the middleware file and explain what it checks before "
                    "allowing access."
                ),
                "correct_files": ["middleware.ts", "middleware.js"],
            },
        ],
    },
    {
        "id": "buckit",
        "name": "buckit",
        "size_label": "Large (70+ JS files, ~50k total with node_modules)",
        "path": Path("C:/Users/navneeth/Desktop/NavneethThings/Projects/buckit"),
        "deny_globs": [
            "**/node_modules/**", "**/.next/**", "**/dist/**",
            "**/build/**", "**/.git/**", "**/*.lock",
        ],
        "system_preamble": (
            "You are a developer exploring the buckit codebase — "
            "a React Native (Expo) mobile app for finding nearby basketball games "
            "and players, with Supabase backend, tournaments, push notifications, "
            "and social features. "
            "The repo has a large node_modules directory — avoid it. "
            "Focus on screens/, services/, contexts/, navigation/, supabase/ source files. "
            "Answer concisely: file path, function/component, key logic."
        ),
        "tasks": [
            {
                "id": "T1",
                "description": "Find nearby players location query",
                "prompt": (
                    "In the buckit app, which function finds basketball players "
                    "near the user's location? What file, function, and distance "
                    "calculation method does it use?"
                ),
                "correct_files": ["userService.js", "userService.ts"],
            },
            {
                "id": "T2",
                "description": "Find push notification Edge Function",
                "prompt": (
                    "Where in buckit is the push notification server-side function? "
                    "What runtime does it use and what notification service does it call?"
                ),
                "correct_files": ["send-push-notification"],
            },
            {
                "id": "T3",
                "description": "Find app entry point and navigation",
                "prompt": (
                    "Find the main entry point of the buckit app. "
                    "What file is it, how is navigation structured, "
                    "and what are the top-level navigator types used?"
                ),
                "correct_files": ["App.js", "App.ts", "App.tsx"],
            },
            {
                "id": "T4",
                "description": "Find user authentication context",
                "prompt": (
                    "In buckit, where is the user authentication state managed? "
                    "Find the context file that tracks the logged-in user and "
                    "explain how it loads user data."
                ),
                "correct_files": ["UserContext.js", "UserContext.ts", "AuthContext"],
            },
        ],
    },
]


# ── Data model ─────────────────────────────────────────────────────────────────

@dataclass
class RunResult:
    repo_id: str
    task_id: str
    mode: str
    input_tokens_fresh: int
    input_tokens_cache_write: int
    input_tokens_cache_read: int
    output_tokens: int
    total_input_consumed: int
    total_tokens: int
    turns: int
    duration_ms: int
    cost_usd: float
    found_correct: bool
    error: Optional[str]
    raw_result: str


# ── Index builder ──────────────────────────────────────────────────────────────

def build_index(repo: dict, db_path: Path, sfs_exe: Path, toml_path: Path) -> bool:
    """Write TOML config and build SemanticFS index for a repo. Returns True on success."""

    repo_path = repo["path"]
    deny_globs_toml = "\n".join(f'  "{g}",' for g in repo["deny_globs"])

    # Use a minimal single-repo config (values validated against config.rs defaults)
    toml_content = f"""[workspace]
repo_root = "{repo_path.as_posix()}"
mount_point = "/mnt/ai"

[filter]
mode = "repo_first"
allow_roots = ["**"]
deny_globs = [
{deny_globs_toml}
  "**/.git/**",
  "**/*.mp4",
]
max_file_mb = 5

[index]
debounce_ms = 300
publish_mode = "two_phase"
chunk_max_lines = 60
chunk_overlap_lines = 6
bulk_event_threshold = 80
hotset_max_paths = 32
pending_path_report_limit = 20

[embedding]
model = "bge-small-en-v1.5"
runtime = "hash"
quantization = "int8"
dimension = 384
batch_size = 64

[retrieval]
rrf_mode = "plain"
rrf_k = 60
topn_symbol = 5
topn_bm25 = 20
topn_vector = 20
topn_final = 10
symbol_exact_boost = 2.0
symbol_prefix_boost = 1.2
allow_stale = false
code_path_boost = 1.15
docs_path_penalty = 0.85
test_path_penalty = 0.95
asset_path_penalty = 0.45
recency_half_life_hours = 24.0
recency_min_boost = 0.85
recency_max_boost = 1.20

[fuse_cache]
max_virtual_inodes = 8192
max_cached_mb = 64
entry_ttl_ms = 300
attr_ttl_ms = 300

[fuse_session]
mode = "pinned"
max_entries = 512

[map]
base_summary_mode = "deterministic_precompute"
llm_enrichment = "disabled"
cache_ttl_sec = 300

[policy]
read_only = true
deny_secret_paths = true
search_result_redaction = false
trust_labels = ["trusted", "untrusted"]

[observability]
metrics_bind = "127.0.0.1:9470"
health_bind = "127.0.0.1:9471"
log_level = "warn"

[mcp]
enabled = true
mode = "minimal"
"""

    toml_path.parent.mkdir(parents=True, exist_ok=True)
    toml_path.write_text(toml_content, encoding="utf-8")
    db_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"  Building index for {repo['name']} ...", end="", flush=True)
    t0 = time.time()
    env = {**os.environ, "SEMANTICFS_DB_PATH": str(db_path)}

    try:
        result = subprocess.run(
            [str(sfs_exe), "--config", str(toml_path), "index", "build"],
            env=env, capture_output=True, text=True, timeout=300,
        )
        elapsed = time.time() - t0
        if result.returncode != 0:
            print(f" FAILED ({elapsed:.0f}s)")
            print(f"    stderr: {result.stderr[:200]}")
            return False
        print(f" done ({elapsed:.0f}s)")
        return True
    except Exception as e:
        print(f" ERROR: {e}")
        return False


# ── MCP config writer ──────────────────────────────────────────────────────────

def write_mcp_config(sfs_exe: Path, toml_path: Path, db_path: Path, out_path: Path):
    """Write an MCP config JSON using native stdio transport."""
    config = {
        "mcpServers": {
            "semanticfs": {
                "command": str(sfs_exe).replace("\\", "/"),
                "args": [
                    "--config", str(toml_path).replace("\\", "/"),
                    "serve", "mcp-stdio",
                ],
                "env": {
                    "SEMANTICFS_DB_PATH": str(db_path).replace("\\", "/"),
                },
            }
        }
    }
    out_path.write_text(json.dumps(config, indent=2), encoding="utf-8")


# ── Claude runner ──────────────────────────────────────────────────────────────

def run_claude(
    repo: dict,
    task: dict,
    mode: str,
    mcp_config_path: Optional[Path],
    claude_exe: Path,
) -> RunResult:
    repo_id = repo["id"]
    task_id = task["id"]
    preamble = repo["system_preamble"]
    prompt = f"{preamble}\n\nQUESTION: {task['prompt']}"

    cmd = [
        str(claude_exe),
        "--print",
        "--output-format", "json",
        "--dangerously-skip-permissions",
        "--no-session-persistence",
        "--tools", "Bash",
    ]
    if mode == "semanticfs" and mcp_config_path:
        cmd += ["--mcp-config", str(mcp_config_path)]

    print(f"    [{task_id}] {mode.upper():10s} ...", end="", flush=True)
    t0 = time.time()

    try:
        proc = subprocess.run(
            cmd,
            input=prompt,
            cwd=str(repo["path"]),
            capture_output=True,
            text=True,
            timeout=TASK_TIMEOUT_SECS,
            encoding="utf-8",
        )
        elapsed_ms = int((time.time() - t0) * 1000)
        stdout = proc.stdout.strip()
        stderr = proc.stderr.strip()

        if not stdout:
            err = (stderr[:200] if stderr else "empty stdout")
            print(f" ERROR: {err[:70]}")
            return RunResult(
                repo_id=repo_id, task_id=task_id, mode=mode,
                input_tokens_fresh=0, input_tokens_cache_write=0,
                input_tokens_cache_read=0, output_tokens=0,
                total_input_consumed=0, total_tokens=0,
                turns=0, duration_ms=elapsed_ms,
                cost_usd=0.0, found_correct=False, error=err, raw_result="",
            )

        data = json.loads(stdout)
        usage = data.get("usage", {})
        fresh      = usage.get("input_tokens", 0)
        cache_w    = usage.get("cache_creation_input_tokens", 0)
        cache_r    = usage.get("cache_read_input_tokens", 0)
        out_toks   = usage.get("output_tokens", 0)
        consumed   = fresh + cache_w
        total      = consumed + out_toks
        turns      = data.get("num_turns", 0)
        cost       = data.get("total_cost_usd", 0.0)
        result_txt = data.get("result", "")

        found = any(
            cf.lower() in result_txt.lower()
            for cf in task["correct_files"]
        )
        sym = "[Y]" if found else "[N]"
        print(f" {sym}  {consumed:>6,} ctx  {turns} turns  {elapsed_ms/1000:.1f}s")

        return RunResult(
            repo_id=repo_id, task_id=task_id, mode=mode,
            input_tokens_fresh=fresh,
            input_tokens_cache_write=cache_w,
            input_tokens_cache_read=cache_r,
            output_tokens=out_toks,
            total_input_consumed=consumed,
            total_tokens=total,
            turns=turns, duration_ms=elapsed_ms,
            cost_usd=cost, found_correct=found,
            error=None, raw_result=result_txt[:600],
        )

    except subprocess.TimeoutExpired:
        elapsed_ms = int((time.time() - t0) * 1000)
        print(f" TIMEOUT ({TASK_TIMEOUT_SECS}s)")
        return RunResult(
            repo_id=repo_id, task_id=task_id, mode=mode,
            input_tokens_fresh=0, input_tokens_cache_write=0,
            input_tokens_cache_read=0, output_tokens=0,
            total_input_consumed=0, total_tokens=0,
            turns=0, duration_ms=elapsed_ms,
            cost_usd=0.0, found_correct=False,
            error=f"timeout {TASK_TIMEOUT_SECS}s", raw_result="",
        )
    except Exception as e:
        elapsed_ms = int((time.time() - t0) * 1000)
        print(f" EXCEPTION: {e}")
        return RunResult(
            repo_id=repo_id, task_id=task_id, mode=mode,
            input_tokens_fresh=0, input_tokens_cache_write=0,
            input_tokens_cache_read=0, output_tokens=0,
            total_input_consumed=0, total_tokens=0,
            turns=0, duration_ms=elapsed_ms,
            cost_usd=0.0, found_correct=False,
            error=str(e), raw_result="",
        )


# ── Report ─────────────────────────────────────────────────────────────────────

def print_report(all_results: list, repos: list):
    W = 100
    print("\n" + "=" * W)
    print("  MULTI-REPO HEAD-TO-HEAD: Naive Claude Code  vs  Claude Code + SemanticFS")
    print("=" * W)

    grand = {"naive": {"ctx":0,"out":0,"cost":0.0,"found":0,"turns":0,"n":0},
             "semanticfs": {"ctx":0,"out":0,"cost":0.0,"found":0,"turns":0,"n":0}}

    for repo in repos:
        rid = repo["id"]
        results = [r for r in all_results if r.repo_id == rid]
        if not results:
            continue

        print(f"\n  REPO: {repo['name']}  ({repo['size_label']})")
        print("  " + "-" * (W - 2))
        hdr = f"  {'Task':<38} {'Mode':<12} {'Ctx Tokens':>10} {'Turns':>5} {'Cost $':>8} {'OK':>4}"
        print(hdr)
        print("  " + "-" * (W - 2))

        naive_map = {r.task_id: r for r in results if r.mode == "naive"}
        sfs_map   = {r.task_id: r for r in results if r.mode == "semanticfs"}

        repo_totals = {"naive": {"ctx":0,"cost":0.0,"found":0,"turns":0,"n":0},
                       "semanticfs": {"ctx":0,"cost":0.0,"found":0,"turns":0,"n":0}}

        for task in repo["tasks"]:
            tid = task["id"]
            desc = task["description"]
            for r, label in [(naive_map.get(tid), "naive"), (sfs_map.get(tid), "semanticfs")]:
                if r is None:
                    continue
                sym = "[Y]" if r.found_correct else "[N]"
                err_note = f"  ERR: {r.error[:25]}" if r.error else ""
                prefix = f"  {desc:<38}" if label == "naive" else f"  {'':38}"
                row = (f"{prefix} {label:<12} {r.total_input_consumed:>10,} "
                       f"{r.turns:>5} {r.cost_usd:>8.4f} {sym:>4}{err_note}")
                print(row)
                t = repo_totals[label]
                t["ctx"] += r.total_input_consumed
                t["cost"] += r.cost_usd
                t["found"] += int(r.found_correct)
                t["turns"] += r.turns
                t["n"] += 1
            print()

        # Per-repo summary
        for label in ["naive", "semanticfs"]:
            t = repo_totals[label]
            n = t["n"] or 1
            acc = f"{t['found']}/{n}"
            print(f"  {'TOTAL ' + label:<50} {t['ctx']:>10,} {t['turns']:>5} {t['cost']:>8.4f}  {acc}")
            # accumulate grand totals
            g = grand[label]
            g["ctx"] += t["ctx"]; g["cost"] += t["cost"]
            g["found"] += t["found"]; g["turns"] += t["turns"]; g["n"] += t["n"]

        ctx_n = repo_totals["naive"]["ctx"]
        ctx_s = repo_totals["semanticfs"]["ctx"]
        if ctx_n > 0:
            pct = 100 * (ctx_n - ctx_s) / ctx_n
            print(f"  Context token reduction for {repo['name']}: {pct:.1f}%")

    # Grand total
    print("\n" + "=" * W)
    print("  GRAND TOTAL (all repos combined)")
    print("  " + "-" * (W - 2))
    for label in ["naive", "semanticfs"]:
        g = grand[label]
        n = g["n"] or 1
        acc = f"{g['found']}/{n} ({100*g['found']/n:.0f}%)"
        print(f"  {'TOTAL ' + label:<50} {g['ctx']:>10,} {g['turns']:>5} {g['cost']:>8.4f}  {acc}")

    ctx_naive = grand["naive"]["ctx"]
    ctx_sfs   = grand["semanticfs"]["ctx"]
    cost_naive = grand["naive"]["cost"]
    cost_sfs   = grand["semanticfs"]["cost"]

    if ctx_naive > 0:
        ctx_pct  = 100 * (ctx_naive - ctx_sfs) / ctx_naive
        cost_pct = 100 * (cost_naive - cost_sfs) / max(cost_naive, 0.0001)
        n_n = grand["naive"]["n"] or 1
        n_s = grand["semanticfs"]["n"] or 1
        print("\n")
        print(f"  Context tokens:  naive={ctx_naive:,}  sfs={ctx_sfs:,}  saved={ctx_naive-ctx_sfs:,} ({ctx_pct:.1f}%)")
        print(f"  Cost:            naive=${cost_naive:.4f}  sfs=${cost_sfs:.4f}  saved=${cost_naive-cost_sfs:.4f} ({cost_pct:.1f}%)")
        print(f"  Avg turns:       naive={grand['naive']['turns']/n_n:.1f}  sfs={grand['semanticfs']['turns']/n_s:.1f}")
        print(f"  Accuracy:        naive={grand['naive']['found']}/{n_n} ({100*grand['naive']['found']/n_n:.0f}%)  "
              f"sfs={grand['semanticfs']['found']}/{n_s} ({100*grand['semanticfs']['found']/n_s:.0f}%)")
    print("=" * W + "\n")


# ── Main ───────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Multi-repo SemanticFS head-to-head")
    parser.add_argument("--repos", nargs="+",
                        help="Repo IDs to run (default: all 4)")
    parser.add_argument("--skip-index", action="store_true",
                        help="Skip index build (use existing)")
    parser.add_argument("--naive-only", action="store_true")
    parser.add_argument("--sfs-only", action="store_true")
    parser.add_argument("--claude", default=str(CLAUDE_EXE))
    parser.add_argument("--semanticfs", default=str(SEMANTICFS_EXE))
    args = parser.parse_args()

    claude_exe = Path(args.claude)
    sfs_exe    = Path(args.semanticfs)

    if not claude_exe.exists():
        print(f"ERROR: claude.exe not found at {claude_exe}")
        sys.exit(1)
    if not sfs_exe.exists():
        print(f"ERROR: semanticfs.exe not found at {sfs_exe}")
        print(f"  Run: cargo build --release -p semanticfs-cli")
        sys.exit(1)

    repos = REPOS
    if args.repos:
        repos = [r for r in REPOS if r["id"] in args.repos]
        if not repos:
            print(f"ERROR: no matching repos for {args.repos}")
            sys.exit(1)

    modes = []
    if not args.sfs_only:   modes.append("naive")
    if not args.naive_only: modes.append("semanticfs")

    total_tasks = sum(len(r["tasks"]) for r in repos) * len(modes)
    print(f"\nMulti-repo head-to-head: {len(repos)} repos × {total_tasks // len(modes)} tasks × {len(modes)} modes = {total_tasks} Claude API calls")
    print(f"Repos: {[r['id'] for r in repos]}")
    print(f"SemanticFS: {sfs_exe}")
    print(f"Claude:     {claude_exe}\n")

    all_results = []

    for repo in repos:
        print(f"\n{'='*60}")
        print(f"REPO: {repo['name']}  ({repo['size_label']})")
        print(f"Path: {repo['path']}")
        print(f"{'='*60}")

        if not repo["path"].exists():
            print(f"  SKIP: repo not found at {repo['path']}")
            continue

        # Per-repo isolated DB and config paths
        db_path   = DB_BASE / repo["id"] / "index.db"
        toml_path = DB_BASE / repo["id"] / "semanticfs.toml"
        mcp_path  = DB_BASE / repo["id"] / "mcp_config.json"

        # Build index
        if not args.skip_index:
            ok = build_index(repo, db_path, sfs_exe, toml_path)
            if not ok and "semanticfs" in modes:
                print(f"  Index build failed — SemanticFS mode will be skipped for {repo['name']}")
                modes_for_repo = ["naive"]
            else:
                modes_for_repo = modes
        else:
            modes_for_repo = modes
            print(f"  Skipping index build (--skip-index)")

        # Write MCP config
        if "semanticfs" in modes_for_repo:
            write_mcp_config(sfs_exe, toml_path, db_path, mcp_path)

        # Run tasks
        for task in repo["tasks"]:
            print(f"\n  Task {task['id']}: {task['description']}")
            for mode in modes_for_repo:
                r = run_claude(
                    repo=repo,
                    task=task,
                    mode=mode,
                    mcp_config_path=mcp_path if mode == "semanticfs" else None,
                    claude_exe=claude_exe,
                )
                all_results.append(r)
                time.sleep(2)  # avoid rate limiting

    # Report
    print_report(all_results, repos)

    # Save JSON artifact
    BENCH_DIR.mkdir(parents=True, exist_ok=True)
    out_path = BENCH_DIR / "multi_repo_h2h_latest.json"
    artifact = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "repos": [r["id"] for r in repos],
        "modes": modes,
        "results": [asdict(r) for r in all_results],
        "summary_by_repo": {},
    }
    for repo in repos:
        rid = repo["id"]
        rr = [r for r in all_results if r.repo_id == rid]
        artifact["summary_by_repo"][rid] = {
            "naive_ctx": sum(r.total_input_consumed for r in rr if r.mode == "naive"),
            "sfs_ctx":   sum(r.total_input_consumed for r in rr if r.mode == "semanticfs"),
            "naive_cost": sum(r.cost_usd for r in rr if r.mode == "naive"),
            "sfs_cost":   sum(r.cost_usd for r in rr if r.mode == "semanticfs"),
            "naive_accuracy": f"{sum(r.found_correct for r in rr if r.mode=='naive')}/{len([r for r in rr if r.mode=='naive'])}",
            "sfs_accuracy":   f"{sum(r.found_correct for r in rr if r.mode=='semanticfs')}/{len([r for r in rr if r.mode=='semanticfs'])}",
        }

    out_path.write_text(json.dumps(artifact, indent=2), encoding="utf-8")
    print(f"Results saved to: {out_path}")


if __name__ == "__main__":
    main()
