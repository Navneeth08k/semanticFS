#!/usr/bin/env python3
"""
Head-to-head comparison: Naive Claude Code vs Claude Code + SemanticFS MCP.

Runs the same 6 realistic codebase-exploration tasks in two modes:
  - NAIVE:      Claude Code with only Bash tools (ls, find, grep, cat, etc.)
  - SEMANTICFS: Claude Code with Bash + SemanticFS search_codebase MCP tool

Measures per-task:
  - Input tokens consumed (fresh + cache-written)
  - Output tokens
  - Tool call turns
  - Wall-clock duration
  - Cost (USD)
  - Whether the correct file was found

Writes results to .semanticfs/bench/head_to_head_claude_latest.json
and prints a formatted comparison table.

Usage:
  python scripts/head_to_head_claude.py [--mcp-config PATH] [--claude PATH]
"""

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional


# ── Configuration ─────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).parent.parent  # semanticFS project root
AI_TESTGEN_ROOT = Path("C:/Users/navneeth/Desktop/NavneethThings/Projects/ai-testgen")

CLAUDE_EXE = Path(
    "C:/Users/navneeth/AppData/Local/Packages"
    "/Claude_pzs8sxrjxfjjc/LocalCache/Roaming/Claude/claude-code/2.1.76/claude.exe"
)

MCP_CONFIG_PATH = REPO_ROOT / "config" / "semanticfs_for_claude_mcp.json"
OUTPUT_PATH = REPO_ROOT / ".semanticfs" / "bench" / "head_to_head_claude_latest.json"

# Timeout per task run (seconds) — generous because indexing + LLM can be slow
TASK_TIMEOUT_SECS = 180

# ── Task definitions ──────────────────────────────────────────────────────────
#
# Each task has:
#   prompt       : what is sent to Claude
#   correct_files: list of filename substrings that count as "found answer"
#   description  : human-readable label for the report
#
# Ground truth established by manual code review.

TASKS = [
    {
        "id": "T1",
        "description": "Find Python signature extractor",
        "prompt": (
            "I'm onboarding to the ai-testgen codebase. "
            "Where is the code that extracts Python function signatures from a git diff? "
            "Show me the function name, the file it's in, and the key logic."
        ),
        "correct_files": ["diff_parser.py", "python_adapter.py"],
    },
    {
        "id": "T2",
        "description": "Find Gemini API initialization",
        "prompt": (
            "In the ai-testgen project, how does the system initialize the Gemini API client? "
            "Which file handles this, what model does it use, and how is the API key loaded?"
        ),
        "correct_files": ["prompt_builder.py"],
    },
    {
        "id": "T3",
        "description": "Find CLI entry point and argument parser",
        "prompt": (
            "I need to add a new command-line flag to ai-testgen. "
            "Find the main CLI entry point — what file is it in, "
            "what function defines the argument parser, and what arguments already exist?"
        ),
        "correct_files": ["cli.py"],
    },
    {
        "id": "T4",
        "description": "Find test file save logic",
        "prompt": (
            "In ai-testgen, when a test is generated, where does the code that saves "
            "the generated test output to disk live? Show me the function and file."
        ),
        "correct_files": ["cli.py"],
    },
    {
        "id": "T5",
        "description": "Find Java AST parsing library",
        "prompt": (
            "The ai-testgen tool parses Java source code. "
            "What library does it use to parse Java ASTs, "
            "which file contains this logic, and what is the function name?"
        ),
        "correct_files": ["diff_parser.py", "java_adapter.py"],
    },
    {
        "id": "T6",
        "description": "Find CI workflow trigger",
        "prompt": (
            "Does ai-testgen have a CI workflow? "
            "Find the workflow file, what it's called, and what events trigger it."
        ),
        "correct_files": ["ci.yml"],
    },
]

# System context shared by both modes
SYSTEM_PREAMBLE = (
    "You are a developer exploring the ai-testgen codebase — a Python tool that "
    "automatically generates tests using the Gemini AI API. "
    "Answer the question by finding the relevant code. "
    "Be concise: state the file, the function/class, and the key logic. "
    "The repo has a large .venv directory with thousands of third-party packages — "
    "avoid exploring it. Focus on the project's own source code."
)


# ── Data model ────────────────────────────────────────────────────────────────

@dataclass
class RunResult:
    task_id: str
    mode: str                     # "naive" | "semanticfs"
    input_tokens_fresh: int
    input_tokens_cache_write: int
    input_tokens_cache_read: int
    output_tokens: int
    total_input_consumed: int     # fresh + cache_write (tokens model actually processed new)
    total_tokens: int             # consumed + output
    turns: int
    duration_ms: int
    cost_usd: float
    found_correct: bool
    error: Optional[str]
    raw_result: str


# ── Claude runner ─────────────────────────────────────────────────────────────

def run_claude(
    task: dict,
    mode: str,
    mcp_config_path: Optional[Path],
    claude_exe: Path,
    cwd: Path,
) -> RunResult:
    """Run one Claude task in the given mode and return structured metrics."""

    task_id = task["id"]
    prompt = f"{SYSTEM_PREAMBLE}\n\nQUESTION: {task['prompt']}"

    # Claude CLI requires prompt via stdin in --print mode
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

    print(f"\n  [{task_id}] {mode.upper():10s} -> running...", end="", flush=True)
    t0 = time.time()

    try:
        proc = subprocess.run(
            cmd,
            input=prompt,          # send prompt via stdin
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=TASK_TIMEOUT_SECS,
            encoding="utf-8",
        )
        elapsed_ms = int((time.time() - t0) * 1000)

        stdout = proc.stdout.strip()
        stderr = proc.stderr.strip()

        if not stdout:
            err_msg = stderr[:300] if stderr else "empty stdout"
            print(f" ERROR: {err_msg[:80]}")
            return RunResult(
                task_id=task_id, mode=mode,
                input_tokens_fresh=0, input_tokens_cache_write=0,
                input_tokens_cache_read=0, output_tokens=0,
                total_input_consumed=0, total_tokens=0,
                turns=0, duration_ms=elapsed_ms,
                cost_usd=0.0, found_correct=False,
                error=err_msg, raw_result="",
            )

        # Parse JSON output from claude
        data = json.loads(stdout)
        usage = data.get("usage", {})

        input_fresh = usage.get("input_tokens", 0)
        cache_write = usage.get("cache_creation_input_tokens", 0)
        cache_read = usage.get("cache_read_input_tokens", 0)
        output_toks = usage.get("output_tokens", 0)
        consumed = input_fresh + cache_write
        total = consumed + output_toks
        turns = data.get("num_turns", 0)
        cost = data.get("total_cost_usd", 0.0)
        result_text = data.get("result", "")

        # Score: did the result mention the expected file?
        found = any(
            cf.lower() in result_text.lower()
            for cf in task["correct_files"]
        )

        symbol = "[Y]" if found else "[N]"
        print(f" done  {symbol}  {consumed:>6,} ctx tokens  {turns} turns  {elapsed_ms/1000:.1f}s")

        return RunResult(
            task_id=task_id, mode=mode,
            input_tokens_fresh=input_fresh,
            input_tokens_cache_write=cache_write,
            input_tokens_cache_read=cache_read,
            output_tokens=output_toks,
            total_input_consumed=consumed,
            total_tokens=total,
            turns=turns,
            duration_ms=elapsed_ms,
            cost_usd=cost,
            found_correct=found,
            error=None,
            raw_result=result_text[:500],
        )

    except subprocess.TimeoutExpired:
        elapsed_ms = int((time.time() - t0) * 1000)
        print(f" TIMEOUT after {TASK_TIMEOUT_SECS}s")
        return RunResult(
            task_id=task_id, mode=mode,
            input_tokens_fresh=0, input_tokens_cache_write=0,
            input_tokens_cache_read=0, output_tokens=0,
            total_input_consumed=0, total_tokens=0,
            turns=0, duration_ms=elapsed_ms,
            cost_usd=0.0, found_correct=False,
            error=f"timeout after {TASK_TIMEOUT_SECS}s", raw_result="",
        )
    except Exception as e:
        elapsed_ms = int((time.time() - t0) * 1000)
        print(f" EXCEPTION: {e}")
        return RunResult(
            task_id=task_id, mode=mode,
            input_tokens_fresh=0, input_tokens_cache_write=0,
            input_tokens_cache_read=0, output_tokens=0,
            total_input_consumed=0, total_tokens=0,
            turns=0, duration_ms=elapsed_ms,
            cost_usd=0.0, found_correct=False,
            error=str(e), raw_result="",
        )


# ── Report printer ────────────────────────────────────────────────────────────

def print_report(results, tasks):
    """Print a formatted comparison table."""

    naive_map = {r.task_id: r for r in results if r.mode == "naive"}
    sfs_map = {r.task_id: r for r in results if r.mode == "semanticfs"}

    print("\n")
    print("=" * 90)
    print("  HEAD-TO-HEAD: Naive Claude Code  vs  Claude Code + SemanticFS MCP")
    print("  Repo: ai-testgen (4,638 files total, 24 source files, large .venv contamination)")
    print("=" * 90)

    # Per-task rows
    header = f"  {'Task':<35} {'Mode':<12} {'Ctx Tokens':>10} {'Out Tokens':>10} {'Turns':>5} {'Cost $':>8} {'Found':>6}"
    print(f"\n{header}")
    print("  " + "-" * 87)

    totals = {"naive": {"ctx": 0, "out": 0, "cost": 0.0, "found": 0, "turns": 0, "n": 0},
              "semanticfs": {"ctx": 0, "out": 0, "cost": 0.0, "found": 0, "turns": 0, "n": 0}}

    for task in tasks:
        tid = task["id"]
        desc = task["description"]
        n = naive_map.get(tid)
        s = sfs_map.get(tid)

        for r, label in [(n, "naive"), (s, "semanticfs")]:
            if r is None:
                continue
            found_sym = "[Y]" if r.found_correct else "[N]"
            err_note = f" [ERR: {r.error[:30]}]" if r.error else ""
            prefix = f"  {desc:<35}" if label == "naive" else f"  {'':35}"
            row = (f"{prefix} {label:<12} {r.total_input_consumed:>10,} "
                   f"{r.output_tokens:>10,} {r.turns:>5} {r.cost_usd:>8.4f} {found_sym:>6}{err_note}")
            print(row)
            t = totals[label]
            t["ctx"] += r.total_input_consumed
            t["out"] += r.output_tokens
            t["cost"] += r.cost_usd
            t["found"] += int(r.found_correct)
            t["turns"] += r.turns
            t["n"] += 1

        print()

    # Totals + deltas
    print("  " + "=" * 87)
    for label in ["naive", "semanticfs"]:
        t = totals[label]
        n = t["n"] or 1
        accuracy = f"{t['found']}/{n} ({100*t['found']/n:.0f}%)"
        print(f"  {'TOTAL  ' + label:<47} {t['ctx']:>10,} {t['out']:>10,} "
              f"{t['turns']:>5} {t['cost']:>8.4f}  {accuracy}")

    n_tasks = totals["naive"]["n"] or 1
    s_tasks = totals["semanticfs"]["n"] or 1
    ctx_saved = totals["naive"]["ctx"] - totals["semanticfs"]["ctx"]
    cost_saved = totals["naive"]["cost"] - totals["semanticfs"]["cost"]
    ctx_pct = 100 * ctx_saved / max(totals["naive"]["ctx"], 1)
    cost_pct = 100 * cost_saved / max(totals["naive"]["cost"], 0.0001)

    print("\n")
    print(f"  Context tokens saved : {ctx_saved:,} ({ctx_pct:.1f}% reduction)")
    print(f"  Cost saved           : ${cost_saved:.4f} ({cost_pct:.1f}% reduction)")
    print(f"  Avg turns  naive     : {totals['naive']['turns']/n_tasks:.1f}")
    print(f"  Avg turns  semanticfs: {totals['semanticfs']['turns']/s_tasks:.1f}")
    print("=" * 90)
    print()


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Claude Code head-to-head comparison")
    parser.add_argument(
        "--mcp-config",
        default=str(MCP_CONFIG_PATH),
        help="Path to MCP config JSON for SemanticFS mode"
    )
    parser.add_argument(
        "--claude",
        default=str(CLAUDE_EXE),
        help="Path to claude.exe"
    )
    parser.add_argument(
        "--tasks",
        nargs="+",
        help="Run only specific task IDs (e.g. T1 T3)"
    )
    parser.add_argument(
        "--naive-only",
        action="store_true",
        help="Run only naive mode"
    )
    parser.add_argument(
        "--sfs-only",
        action="store_true",
        help="Run only SemanticFS mode"
    )
    args = parser.parse_args()

    claude_exe = Path(args.claude)
    mcp_config = Path(args.mcp_config)
    cwd = AI_TESTGEN_ROOT

    if not claude_exe.exists():
        print(f"ERROR: claude.exe not found at {claude_exe}")
        sys.exit(1)

    if not cwd.exists():
        print(f"ERROR: ai-testgen repo not found at {cwd}")
        sys.exit(1)

    tasks = TASKS
    if args.tasks:
        tasks = [t for t in TASKS if t["id"] in args.tasks]
        print(f"Running {len(tasks)} selected tasks: {[t['id'] for t in tasks]}")

    modes = []
    if not args.sfs_only:
        modes.append("naive")
    if not args.naive_only:
        modes.append("semanticfs")

    print(f"\nHead-to-head comparison: {len(tasks)} tasks × {len(modes)} modes")
    print(f"Repo: {cwd}")
    print(f"Claude: {claude_exe}")
    if "semanticfs" in modes:
        print(f"MCP config: {mcp_config}")
        if not mcp_config.exists():
            print(f"  WARNING: MCP config not found — SemanticFS mode will fail")

    results = []

    for task in tasks:
        print(f"\nTask {task['id']}: {task['description']}")
        for mode in modes:
            r = run_claude(
                task=task,
                mode=mode,
                mcp_config_path=mcp_config if mode == "semanticfs" else None,
                claude_exe=claude_exe,
                cwd=cwd,
            )
            results.append(r)
            # Small pause between runs to avoid rate limiting
            time.sleep(2)

    # Print report
    print_report(results, tasks)

    # Save JSON artifact
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    artifact = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "repo": str(cwd),
        "claude_exe": str(claude_exe),
        "mcp_config": str(mcp_config),
        "tasks": tasks,
        "results": [asdict(r) for r in results],
        "summary": {
            "naive_ctx_tokens_total": sum(r.total_input_consumed for r in results if r.mode == "naive"),
            "semanticfs_ctx_tokens_total": sum(r.total_input_consumed for r in results if r.mode == "semanticfs"),
            "naive_cost_usd_total": sum(r.cost_usd for r in results if r.mode == "naive"),
            "semanticfs_cost_usd_total": sum(r.cost_usd for r in results if r.mode == "semanticfs"),
            "naive_accuracy": f"{sum(r.found_correct for r in results if r.mode == 'naive')}/{len([r for r in results if r.mode == 'naive'])}",
            "semanticfs_accuracy": f"{sum(r.found_correct for r in results if r.mode == 'semanticfs')}/{len([r for r in results if r.mode == 'semanticfs'])}",
        }
    }
    with open(OUTPUT_PATH, "w", encoding="utf-8") as f:
        json.dump(artifact, f, indent=2)
    print(f"Results saved to: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
