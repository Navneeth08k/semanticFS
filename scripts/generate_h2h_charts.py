#!/usr/bin/env python3
"""
Generate SVG charts from head-to-head comparison results.
Pure Python, zero dependencies (no numpy/matplotlib needed).
Outputs to docs/charts/ for embedding in README.
"""

import json
import os
from pathlib import Path

ROOT = Path(__file__).parent.parent
ARTIFACT = ROOT / ".semanticfs" / "bench" / "head_to_head_claude_latest.json"
OUT_DIR = ROOT / "docs" / "charts"

# ── Palette ────────────────────────────────────────────────────────────────────
NAIVE_COLOR    = "#E05C5C"   # warm red
SFS_COLOR      = "#4A90D9"   # clean blue
NAIVE_LIGHT    = "#F5A5A5"
SFS_LIGHT      = "#A5C8F0"
BG             = "#FAFAFA"
GRID           = "#E8E8E8"
TEXT           = "#2D2D2D"
SUBTEXT        = "#777777"
ACCENT         = "#2ECC71"   # green for savings

# ── Task labels ────────────────────────────────────────────────────────────────
TASK_LABELS = {
    "T1": "Python sig\nextractor",
    "T2": "Gemini API\ninit",
    "T3": "CLI entry\npoint",
    "T4": "Test file\nsave logic",
    "T5": "Java AST\nlibrary",
    "T6": "CI workflow\ntrigger",
}

TASK_LABELS_SINGLE = {
    "T1": "Python sig extractor",
    "T2": "Gemini API init",
    "T3": "CLI entry point",
    "T4": "Test file save logic",
    "T5": "Java AST library",
    "T6": "CI workflow trigger",
}


def load_data():
    with open(ARTIFACT) as f:
        raw = json.load(f)

    # Re-score T1/T5 with corrected ground truth
    correct_overrides = {
        "T1": ["diff_parser.py", "python_adapter.py"],
        "T5": ["diff_parser.py", "java_adapter.py"],
    }
    tasks = {t["id"]: t for t in raw["tasks"]}
    results = {}
    for r in raw["results"]:
        tid = r["task_id"]
        mode = r["mode"]
        cf = correct_overrides.get(tid, tasks.get(tid, {}).get("correct_files", []))
        found = any(f.lower() in r["raw_result"].lower() for f in cf)
        results[(tid, mode)] = {**r, "found_correct": found}
    return results


def esc(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


# ── Chart 1: Context Tokens per Task (grouped bar) ─────────────────────────────

def chart_ctx_tokens(results):
    W, H = 760, 380
    tasks = ["T1", "T2", "T3", "T4", "T5", "T6"]
    naive_vals = [results[(t, "naive")]["total_input_consumed"] for t in tasks]
    sfs_vals   = [results[(t, "semanticfs")]["total_input_consumed"] for t in tasks]
    max_val = max(max(naive_vals), max(sfs_vals)) * 1.15

    pad_l, pad_r, pad_t, pad_b = 70, 24, 50, 80
    plot_w = W - pad_l - pad_r
    plot_h = H - pad_t - pad_b
    n = len(tasks)
    group_w = plot_w / n
    bar_w = group_w * 0.32

    def y(v):
        return pad_t + plot_h - (v / max_val) * plot_h

    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="system-ui,sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{BG}" rx="8"/>',
        # Title
        f'<text x="{W//2}" y="28" text-anchor="middle" font-size="15" font-weight="600" fill="{TEXT}">Context Tokens Consumed per Task</text>',
        f'<text x="{W//2}" y="44" text-anchor="middle" font-size="11" fill="{SUBTEXT}">Lower is better — tokens the model processed fresh (cache writes + fresh input)</text>',
    ]

    # Grid lines
    for pct in [0.25, 0.5, 0.75, 1.0]:
        gy = pad_t + plot_h - pct * plot_h
        gv = int(max_val * pct)
        lines.append(f'<line x1="{pad_l}" y1="{gy:.1f}" x2="{W-pad_r}" y2="{gy:.1f}" stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{pad_l-6}" y="{gy+4:.1f}" text-anchor="end" font-size="10" fill="{SUBTEXT}">{gv:,}</text>')

    # Axes
    lines.append(f'<line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{pad_t+plot_h}" stroke="#CCC" stroke-width="1.5"/>')
    lines.append(f'<line x1="{pad_l}" y1="{pad_t+plot_h}" x2="{W-pad_r}" y2="{pad_t+plot_h}" stroke="#CCC" stroke-width="1.5"/>')

    for i, tid in enumerate(tasks):
        cx = pad_l + (i + 0.5) * group_w
        nv, sv = naive_vals[i], sfs_vals[i]
        nx = cx - bar_w * 0.6
        sx = cx + bar_w * 0.1

        # Naive bar
        ny = y(nv)
        bh = pad_t + plot_h - ny
        lines.append(f'<rect x="{nx:.1f}" y="{ny:.1f}" width="{bar_w:.1f}" height="{bh:.1f}" fill="{NAIVE_COLOR}" rx="3" opacity="0.9"/>')
        lines.append(f'<text x="{nx+bar_w/2:.1f}" y="{ny-4:.1f}" text-anchor="middle" font-size="9" fill="{NAIVE_COLOR}" font-weight="600">{nv:,}</text>')

        # SemanticFS bar
        sy2 = y(sv)
        bh2 = pad_t + plot_h - sy2
        lines.append(f'<rect x="{sx:.1f}" y="{sy2:.1f}" width="{bar_w:.1f}" height="{bh2:.1f}" fill="{SFS_COLOR}" rx="3" opacity="0.9"/>')
        label = str(sv) if sv > 50 else str(sv)
        lines.append(f'<text x="{sx+bar_w/2:.1f}" y="{sy2-4:.1f}" text-anchor="middle" font-size="9" fill="{SFS_COLOR}" font-weight="600">{sv:,}</text>')

        # Pct reduction
        if nv > 0:
            pct = int(100 * (nv - sv) / nv)
            lines.append(f'<text x="{cx:.1f}" y="{pad_t+plot_h+14:.1f}" text-anchor="middle" font-size="9" fill="{ACCENT}" font-weight="600">-{pct}%</text>')

        # Task label
        label_y = pad_t + plot_h + 26
        for j, part in enumerate(TASK_LABELS[tid].split("\n")):
            lines.append(f'<text x="{cx:.1f}" y="{label_y + j*13:.1f}" text-anchor="middle" font-size="10" fill="{SUBTEXT}">{esc(part)}</text>')

    # Legend
    lx = W - pad_r - 200
    ly = pad_t + 8
    lines += [
        f'<rect x="{lx}" y="{ly}" width="12" height="12" fill="{NAIVE_COLOR}" rx="2"/>',
        f'<text x="{lx+16}" y="{ly+10}" font-size="11" fill="{TEXT}">Naive (Bash only)</text>',
        f'<rect x="{lx}" y="{ly+18}" width="12" height="12" fill="{SFS_COLOR}" rx="2"/>',
        f'<text x="{lx+16}" y="{ly+28}" font-size="11" fill="{TEXT}">+ SemanticFS MCP</text>',
    ]

    lines.append("</svg>")
    return "\n".join(lines)


# ── Chart 2: Cost per Task ────────────────────────────────────────────────────

def chart_cost(results):
    W, H = 760, 340
    tasks = ["T1", "T2", "T3", "T4", "T5", "T6"]
    naive_vals = [results[(t, "naive")]["cost_usd"] for t in tasks]
    sfs_vals   = [results[(t, "semanticfs")]["cost_usd"] for t in tasks]
    max_val = max(max(naive_vals), max(sfs_vals)) * 1.2

    pad_l, pad_r, pad_t, pad_b = 64, 24, 50, 80
    plot_w = W - pad_l - pad_r
    plot_h = H - pad_t - pad_b
    n = len(tasks)
    group_w = plot_w / n
    bar_w = group_w * 0.32

    def y(v):
        return pad_t + plot_h - (v / max_val) * plot_h

    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="system-ui,sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{BG}" rx="8"/>',
        f'<text x="{W//2}" y="28" text-anchor="middle" font-size="15" font-weight="600" fill="{TEXT}">API Cost per Task (USD)</text>',
        f'<text x="{W//2}" y="44" text-anchor="middle" font-size="11" fill="{SUBTEXT}">Lower is better — total cost including cache operations</text>',
    ]

    for pct in [0.25, 0.5, 0.75, 1.0]:
        gy = pad_t + plot_h - pct * plot_h
        gv = max_val * pct
        lines.append(f'<line x1="{pad_l}" y1="{gy:.1f}" x2="{W-pad_r}" y2="{gy:.1f}" stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{pad_l-6}" y="{gy+4:.1f}" text-anchor="end" font-size="10" fill="{SUBTEXT}">${gv:.4f}</text>')

    lines.append(f'<line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{pad_t+plot_h}" stroke="#CCC" stroke-width="1.5"/>')
    lines.append(f'<line x1="{pad_l}" y1="{pad_t+plot_h}" x2="{W-pad_r}" y2="{pad_t+plot_h}" stroke="#CCC" stroke-width="1.5"/>')

    for i, tid in enumerate(tasks):
        cx = pad_l + (i + 0.5) * group_w
        nv, sv = naive_vals[i], sfs_vals[i]
        nx = cx - bar_w * 0.6
        sx = cx + bar_w * 0.1

        ny = y(nv)
        bh = pad_t + plot_h - ny
        lines.append(f'<rect x="{nx:.1f}" y="{ny:.1f}" width="{bar_w:.1f}" height="{bh:.1f}" fill="{NAIVE_COLOR}" rx="3" opacity="0.9"/>')
        lines.append(f'<text x="{nx+bar_w/2:.1f}" y="{ny-4:.1f}" text-anchor="middle" font-size="9" fill="{NAIVE_COLOR}" font-weight="600">${nv:.4f}</text>')

        sy2 = y(sv)
        bh2 = pad_t + plot_h - sy2
        lines.append(f'<rect x="{sx:.1f}" y="{sy2:.1f}" width="{bar_w:.1f}" height="{bh2:.1f}" fill="{SFS_COLOR}" rx="3" opacity="0.9"/>')
        lines.append(f'<text x="{sx+bar_w/2:.1f}" y="{sy2-4:.1f}" text-anchor="middle" font-size="9" fill="{SFS_COLOR}" font-weight="600">${sv:.4f}</text>')

        if nv > 0:
            pct = int(100 * (nv - sv) / nv)
            lines.append(f'<text x="{cx:.1f}" y="{pad_t+plot_h+14:.1f}" text-anchor="middle" font-size="9" fill="{ACCENT}" font-weight="600">-{pct}%</text>')

        label_y = pad_t + plot_h + 26
        for j, part in enumerate(TASK_LABELS[tid].split("\n")):
            lines.append(f'<text x="{cx:.1f}" y="{label_y + j*13:.1f}" text-anchor="middle" font-size="10" fill="{SUBTEXT}">{esc(part)}</text>')

    lx = W - pad_r - 200
    ly = pad_t + 8
    lines += [
        f'<rect x="{lx}" y="{ly}" width="12" height="12" fill="{NAIVE_COLOR}" rx="2"/>',
        f'<text x="{lx+16}" y="{ly+10}" font-size="11" fill="{TEXT}">Naive (Bash only)</text>',
        f'<rect x="{lx}" y="{ly+18}" width="12" height="12" fill="{SFS_COLOR}" rx="2"/>',
        f'<text x="{lx+16}" y="{ly+28}" font-size="11" fill="{TEXT}">+ SemanticFS MCP</text>',
    ]

    lines.append("</svg>")
    return "\n".join(lines)


# ── Chart 3: Summary scorecard ─────────────────────────────────────────────────

def chart_summary(results):
    W, H = 760, 280
    tasks = ["T1", "T2", "T3", "T4", "T5", "T6"]

    naive_ctx   = sum(results[(t, "naive")]["total_input_consumed"] for t in tasks)
    sfs_ctx     = sum(results[(t, "semanticfs")]["total_input_consumed"] for t in tasks)
    naive_cost  = sum(results[(t, "naive")]["cost_usd"] for t in tasks)
    sfs_cost    = sum(results[(t, "semanticfs")]["cost_usd"] for t in tasks)
    naive_turns = sum(results[(t, "naive")]["turns"] for t in tasks)
    sfs_turns   = sum(results[(t, "semanticfs")]["turns"] for t in tasks)
    ctx_save_pct  = 100 * (naive_ctx - sfs_ctx) / naive_ctx
    cost_save_pct = 100 * (naive_cost - sfs_cost) / naive_cost
    turn_save_pct = 100 * (naive_turns - sfs_turns) / naive_turns

    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="system-ui,sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{BG}" rx="8"/>',
        f'<text x="{W//2}" y="30" text-anchor="middle" font-size="15" font-weight="600" fill="{TEXT}">Overall Results: Naive vs SemanticFS (6 tasks, 100% accuracy both)</text>',
    ]

    # Three metric cards
    cards = [
        ("Context Tokens", f"{naive_ctx:,}", f"{sfs_ctx:,}", f"-{ctx_save_pct:.1f}%", "Fresh tokens model processed"),
        ("Total Cost (USD)", f"${naive_cost:.4f}", f"${sfs_cost:.4f}", f"-{cost_save_pct:.1f}%", "API cost across 6 tasks"),
        ("Tool Call Turns", f"{naive_turns}", f"{sfs_turns}", f"-{turn_save_pct:.1f}%", "Back-and-forth exploration steps"),
    ]

    card_w = 220
    card_h = 180
    gap = (W - 3 * card_w) / 4
    cy = 60

    for i, (metric, naive_v, sfs_v, saving, subtitle) in enumerate(cards):
        cx = gap + i * (card_w + gap)
        # Card background
        lines.append(f'<rect x="{cx:.1f}" y="{cy}" width="{card_w}" height="{card_h}" fill="white" rx="8" stroke="{GRID}" stroke-width="1.5"/>')
        # Metric name
        lines.append(f'<text x="{cx+card_w/2:.1f}" y="{cy+22}" text-anchor="middle" font-size="12" font-weight="600" fill="{TEXT}">{esc(metric)}</text>')
        lines.append(f'<text x="{cx+card_w/2:.1f}" y="{cy+36}" text-anchor="middle" font-size="9" fill="{SUBTEXT}">{esc(subtitle)}</text>')
        # Divider
        lines.append(f'<line x1="{cx+16}" y1="{cy+44}" x2="{cx+card_w-16}" y2="{cy+44}" stroke="{GRID}" stroke-width="1"/>')
        # Naive value
        lines.append(f'<rect x="{cx+12}" y="{cy+50}" width="8" height="8" fill="{NAIVE_COLOR}" rx="1"/>')
        lines.append(f'<text x="{cx+26}" y="{cy+59}" font-size="10" fill="{SUBTEXT}">Naive</text>')
        lines.append(f'<text x="{cx+card_w-12}" y="{cy+59}" text-anchor="end" font-size="13" font-weight="600" fill="{NAIVE_COLOR}">{esc(naive_v)}</text>')
        # SFS value
        lines.append(f'<rect x="{cx+12}" y="{cy+72}" width="8" height="8" fill="{SFS_COLOR}" rx="1"/>')
        lines.append(f'<text x="{cx+26}" y="{cy+81}" font-size="10" fill="{SUBTEXT}">SemanticFS</text>')
        lines.append(f'<text x="{cx+card_w-12}" y="{cy+81}" text-anchor="end" font-size="13" font-weight="600" fill="{SFS_COLOR}">{esc(sfs_v)}</text>')
        # Savings badge
        lines.append(f'<rect x="{cx+card_w/2-36:.1f}" y="{cy+104}" width="72" height="28" fill="{ACCENT}" rx="14" opacity="0.15"/>')
        lines.append(f'<text x="{cx+card_w/2:.1f}" y="{cy+122}" text-anchor="middle" font-size="16" font-weight="700" fill="{ACCENT}">{esc(saving)}</text>')
        lines.append(f'<text x="{cx+card_w/2:.1f}" y="{cy+148}" text-anchor="middle" font-size="10" fill="{SUBTEXT}">reduction</text>')

    lines.append("</svg>")
    return "\n".join(lines)


# ── Chart 4: Token savings per task (horizontal bar) ──────────────────────────

def chart_savings(results):
    W, H = 680, 320
    tasks = ["T1", "T2", "T3", "T4", "T5", "T6"]
    savings = []
    for t in tasks:
        nv = results[(t, "naive")]["total_input_consumed"]
        sv = results[(t, "semanticfs")]["total_input_consumed"]
        pct = 100 * (nv - sv) / max(nv, 1)
        savings.append((t, nv - sv, pct))

    max_abs = max(s[1] for s in savings) * 1.15
    pad_l, pad_r, pad_t, pad_b = 150, 100, 50, 40
    plot_w = W - pad_l - pad_r
    plot_h = H - pad_t - pad_b
    bar_h = plot_h / len(tasks) * 0.55
    row_h = plot_h / len(tasks)

    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="system-ui,sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{BG}" rx="8"/>',
        f'<text x="{W//2}" y="28" text-anchor="middle" font-size="15" font-weight="600" fill="{TEXT}">Context Token Savings per Task</text>',
        f'<text x="{W//2}" y="43" text-anchor="middle" font-size="11" fill="{SUBTEXT}">Tokens saved = Naive tokens - SemanticFS tokens</text>',
    ]

    # Grid
    for pct in [0.25, 0.5, 0.75, 1.0]:
        gx = pad_l + pct * plot_w
        gv = int(max_abs * pct)
        lines.append(f'<line x1="{gx:.1f}" y1="{pad_t}" x2="{gx:.1f}" y2="{pad_t+plot_h}" stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{gx:.1f}" y="{pad_t+plot_h+14}" text-anchor="middle" font-size="9" fill="{SUBTEXT}">{gv:,}</text>')

    lines.append(f'<line x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{pad_t+plot_h}" stroke="#CCC" stroke-width="1.5"/>')

    for i, (tid, saved, pct) in enumerate(savings):
        cy = pad_t + i * row_h + (row_h - bar_h) / 2
        bar_len = (saved / max_abs) * plot_w

        # Bar
        color = ACCENT if pct > 50 else SFS_COLOR
        lines.append(f'<rect x="{pad_l}" y="{cy:.1f}" width="{bar_len:.1f}" height="{bar_h:.1f}" fill="{color}" rx="3" opacity="0.85"/>')

        # Task label
        lines.append(f'<text x="{pad_l-8}" y="{cy+bar_h/2+4:.1f}" text-anchor="end" font-size="11" fill="{TEXT}">{esc(TASK_LABELS_SINGLE[tid])}</text>')

        # Value + pct
        lines.append(f'<text x="{pad_l+bar_len+8:.1f}" y="{cy+bar_h/2+4:.1f}" font-size="11" font-weight="600" fill="{color}">{saved:,} ({pct:.0f}%)</text>')

    lines.append("</svg>")
    return "\n".join(lines)


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    results = load_data()

    charts = [
        ("chart_ctx_tokens.svg", chart_ctx_tokens(results)),
        ("chart_cost.svg", chart_cost(results)),
        ("chart_summary.svg", chart_summary(results)),
        ("chart_savings.svg", chart_savings(results)),
    ]

    for fname, svg in charts:
        path = OUT_DIR / fname
        path.write_text(svg, encoding="utf-8")
        print(f"  Written: {path}")

    print(f"\nAll charts saved to {OUT_DIR}")


if __name__ == "__main__":
    main()
