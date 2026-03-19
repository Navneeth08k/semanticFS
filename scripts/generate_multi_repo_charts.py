#!/usr/bin/env python3
"""
Generate SVG charts from multi-repo head-to-head benchmark results.
Pure Python, zero dependencies.  Outputs to docs/charts/.
"""

import json
from pathlib import Path

ROOT       = Path(__file__).parent.parent
ARTIFACT   = ROOT / ".semanticfs" / "bench" / "multi_repo_h2h_latest.json"
OUT_DIR    = ROOT / "docs" / "charts"

# ── Palette ────────────────────────────────────────────────────────────────────
NAIVE_COLOR  = "#E05C5C"
SFS_COLOR    = "#4A90D9"
NAIVE_LIGHT  = "#F0A0A0"
SFS_LIGHT    = "#A0C4F0"
BG           = "#FAFAFA"
GRID         = "#E8E8E8"
TEXT         = "#2D2D2D"
SUBTEXT      = "#777777"
GREEN        = "#27AE60"
ORANGE       = "#E67E22"

OUT_DIR.mkdir(parents=True, exist_ok=True)

# ── Load data ──────────────────────────────────────────────────────────────────

data = json.loads(ARTIFACT.read_text())
results = data["results"]

REPOS = [
    {"id": "prizePicksAI",      "label": "prizePicksAI",    "size": "5 files\n(Tiny)"},
    {"id": "KalshiTradingAlgo", "label": "KalshiTrading",   "size": "17 files\n(Small)"},
    {"id": "syntaxless",        "label": "syntaxless",       "size": "95+ files\n(Medium)"},
    {"id": "buckit",            "label": "buckit",           "size": "70+ files\n(Large)"},
]

def get_runs(repo_id, mode):
    return [r for r in results if r["repo_id"] == repo_id and r["mode"] == mode]

def total_ctx(runs):
    return sum(r["total_input_consumed"] for r in runs)

def total_cost(runs):
    return sum(r["cost_usd"] for r in runs)

def total_cache_read(runs):
    return sum(r["input_tokens_cache_read"] for r in runs)

def full_ctx(runs):
    """Total tokens Claude actually processed (fresh + cache_write + cache_read)."""
    return sum(
        r["input_tokens_fresh"] + r["input_tokens_cache_write"] + r["input_tokens_cache_read"]
        for r in runs
    )

# ── Chart 1: Cost comparison per repo ────────────────────────────────────────

def chart_cost_comparison():
    W, H = 820, 380
    margin_l, margin_r, margin_t, margin_b = 80, 30, 50, 80
    plot_w = W - margin_l - margin_r
    plot_h = H - margin_t - margin_b

    repo_data = []
    for repo in REPOS:
        n_runs = get_runs(repo["id"], "naive")
        s_runs = get_runs(repo["id"], "semanticfs")
        if not n_runs or not s_runs:
            continue
        nc = total_cost(n_runs) * 100   # cents
        sc = total_cost(s_runs) * 100
        repo_data.append({
            "label": repo["label"],
            "size": repo["size"],
            "naive": nc,
            "sfs": sc,
            "pct": (nc - sc) / nc * 100 if nc > 0 else 0,
        })

    max_cost = max(max(d["naive"], d["sfs"]) for d in repo_data) * 1.2
    n = len(repo_data)
    group_w = plot_w / n
    bar_w = group_w * 0.32
    gap = group_w * 0.06

    def y(val):
        return margin_t + plot_h - (val / max_cost) * plot_h

    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">']
    lines.append(f'<rect width="{W}" height="{H}" fill="{BG}"/>')

    # Title
    lines.append(f'<text x="{W//2}" y="28" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="15" font-weight="600" fill="{TEXT}">API Cost per Repo (4 tasks each) — Naive vs SemanticFS</text>')

    # Y-axis grid lines
    y_ticks = 5
    for i in range(y_ticks + 1):
        val = max_cost * i / y_ticks
        yy = y(val)
        lines.append(f'<line x1="{margin_l}" y1="{yy:.1f}" x2="{margin_l + plot_w}" y2="{yy:.1f}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{margin_l - 8}" y="{yy + 4:.1f}" text-anchor="end" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{SUBTEXT}">{val:.1f}¢</text>')

    # Bars
    for i, d in enumerate(repo_data):
        gx = margin_l + i * group_w + group_w / 2
        x_naive = gx - bar_w - gap / 2
        x_sfs   = gx + gap / 2

        yn = y(d["naive"]); ys = y(d["sfs"])
        bh_n = margin_t + plot_h - yn; bh_s = margin_t + plot_h - ys

        lines.append(f'<rect x="{x_naive:.1f}" y="{yn:.1f}" width="{bar_w:.1f}" height="{bh_n:.1f}" '
                     f'fill="{NAIVE_COLOR}" rx="3"/>')
        lines.append(f'<rect x="{x_sfs:.1f}" y="{ys:.1f}" width="{bar_w:.1f}" height="{bh_s:.1f}" '
                     f'fill="{SFS_COLOR}" rx="3"/>')

        # value labels on bars
        lines.append(f'<text x="{x_naive + bar_w/2:.1f}" y="{yn - 5:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="10" fill="{TEXT}">{d["naive"]:.1f}¢</text>')
        lines.append(f'<text x="{x_sfs + bar_w/2:.1f}" y="{ys - 5:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="10" fill="{TEXT}">{d["sfs"]:.1f}¢</text>')

        # pct label
        pct = d["pct"]
        color = GREEN if pct > 0 else ORANGE
        sign = "▼" if pct > 0 else "▲"
        label = f'{sign}{abs(pct):.1f}%'
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 20:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" font-weight="600" fill="{color}">{label}</text>')

        # x-axis label
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 38:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="12" font-weight="600" fill="{TEXT}">{d["label"]}</text>')
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 54:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="10" fill="{SUBTEXT}">{d["size"].replace(chr(10)," ")}</text>')

    # Y-axis label
    lines.append(f'<text x="14" y="{H//2}" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{SUBTEXT}" transform="rotate(-90,14,{H//2})">Cost (¢)</text>')

    # Legend
    lx, ly = margin_l, margin_t - 8
    for color, label in [(NAIVE_COLOR, "Naive (Bash only)"), (SFS_COLOR, "Claude + SemanticFS")]:
        lines.append(f'<rect x="{lx}" y="{ly - 10}" width="14" height="14" fill="{color}" rx="2"/>')
        lines.append(f'<text x="{lx + 18}" y="{ly + 2}" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="12" fill="{TEXT}">{label}</text>')
        lx += 180

    lines.append('</svg>')
    return "\n".join(lines)


# ── Chart 2: Total tokens (full context window used) ──────────────────────────

def chart_full_context():
    W, H = 820, 380
    margin_l, margin_r, margin_t, margin_b = 90, 30, 50, 80
    plot_w = W - margin_l - margin_r
    plot_h = H - margin_t - margin_b

    repo_data = []
    for repo in REPOS:
        n_runs = get_runs(repo["id"], "naive")
        s_runs = get_runs(repo["id"], "semanticfs")
        if not n_runs or not s_runs:
            continue
        nv = full_ctx(n_runs)
        sv = full_ctx(s_runs)
        repo_data.append({
            "label": repo["label"],
            "naive": nv,
            "sfs": sv,
            "pct": (nv - sv) / nv * 100 if nv > 0 else 0,
        })

    max_v = max(max(d["naive"], d["sfs"]) for d in repo_data) * 1.2
    n = len(repo_data)
    group_w = plot_w / n
    bar_w = group_w * 0.32
    gap = group_w * 0.06

    def y(val):
        return margin_t + plot_h - (val / max_v) * plot_h

    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">']
    lines.append(f'<rect width="{W}" height="{H}" fill="{BG}"/>')

    lines.append(f'<text x="{W//2}" y="28" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="15" font-weight="600" fill="{TEXT}">Total Context Processed per Repo (fresh + cache_write + cache_read)</text>')

    y_ticks = 5
    for i in range(y_ticks + 1):
        val = max_v * i / y_ticks
        yy = y(val)
        lines.append(f'<line x1="{margin_l}" y1="{yy:.1f}" x2="{margin_l + plot_w}" y2="{yy:.1f}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        label = f'{val/1000:.0f}k' if val >= 1000 else f'{val:.0f}'
        lines.append(f'<text x="{margin_l - 8}" y="{yy + 4:.1f}" text-anchor="end" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{SUBTEXT}">{label}</text>')

    for i, d in enumerate(repo_data):
        gx = margin_l + i * group_w + group_w / 2
        x_naive = gx - bar_w - gap / 2
        x_sfs   = gx + gap / 2

        yn = y(d["naive"]); ys = y(d["sfs"])
        bh_n = margin_t + plot_h - yn; bh_s = margin_t + plot_h - ys

        lines.append(f'<rect x="{x_naive:.1f}" y="{yn:.1f}" width="{bar_w:.1f}" height="{bh_n:.1f}" '
                     f'fill="{NAIVE_COLOR}" rx="3"/>')
        lines.append(f'<rect x="{x_sfs:.1f}" y="{ys:.1f}" width="{bar_w:.1f}" height="{bh_s:.1f}" '
                     f'fill="{SFS_COLOR}" rx="3"/>')

        lines.append(f'<text x="{x_naive + bar_w/2:.1f}" y="{yn - 5:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="9.5" fill="{TEXT}">{d["naive"]//1000:.0f}k</text>')
        lines.append(f'<text x="{x_sfs + bar_w/2:.1f}" y="{ys - 5:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="9.5" fill="{TEXT}">{d["sfs"]//1000:.0f}k</text>')

        pct = d["pct"]
        color = GREEN if pct > 2 else (ORANGE if pct < -2 else SUBTEXT)
        sign = "▼" if pct > 0 else "▲"
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 20:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" font-weight="600" fill="{color}">'
                     f'{sign}{abs(pct):.1f}%</text>')

        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 38:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="12" font-weight="600" fill="{TEXT}">{d["label"]}</text>')

    lines.append(f'<text x="14" y="{H//2}" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{SUBTEXT}" transform="rotate(-90,14,{H//2})">Tokens</text>')

    lx, ly = margin_l, margin_t - 8
    for color, label in [(NAIVE_COLOR, "Naive (Bash only)"), (SFS_COLOR, "Claude + SemanticFS")]:
        lines.append(f'<rect x="{lx}" y="{ly - 10}" width="14" height="14" fill="{color}" rx="2"/>')
        lines.append(f'<text x="{lx + 18}" y="{ly + 2}" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="12" fill="{TEXT}">{label}</text>')
        lx += 180

    lines.append('</svg>')
    return "\n".join(lines)


# ── Chart 3: Accuracy (always 100%) + Turns ───────────────────────────────────

def chart_turns():
    W, H = 820, 300
    margin_l, margin_r, margin_t, margin_b = 60, 30, 50, 80
    plot_w = W - margin_l - margin_r
    plot_h = H - margin_t - margin_b

    repo_data = []
    for repo in REPOS:
        n_runs = get_runs(repo["id"], "naive")
        s_runs = get_runs(repo["id"], "semanticfs")
        if not n_runs or not s_runs:
            continue
        repo_data.append({
            "label": repo["label"],
            "naive_turns": sum(r["turns"] for r in n_runs),
            "sfs_turns": sum(r["turns"] for r in s_runs),
            "naive_acc": sum(1 for r in n_runs if r["found_correct"]),
            "sfs_acc": sum(1 for r in s_runs if r["found_correct"]),
            "total": len(n_runs),
        })

    max_turns = max(max(d["naive_turns"], d["sfs_turns"]) for d in repo_data) * 1.25
    n = len(repo_data)
    group_w = plot_w / n
    bar_w = group_w * 0.32
    gap = group_w * 0.06

    def y(val):
        return margin_t + plot_h - (val / max_turns) * plot_h

    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">']
    lines.append(f'<rect width="{W}" height="{H}" fill="{BG}"/>')

    lines.append(f'<text x="{W//2}" y="28" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="15" font-weight="600" fill="{TEXT}">Total Turns &amp; Accuracy (4 tasks per repo)</text>')

    y_ticks = 4
    for i in range(y_ticks + 1):
        val = max_turns * i / y_ticks
        yy = y(val)
        lines.append(f'<line x1="{margin_l}" y1="{yy:.1f}" x2="{margin_l + plot_w}" y2="{yy:.1f}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{margin_l - 8}" y="{yy + 4:.1f}" text-anchor="end" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{SUBTEXT}">{val:.0f}</text>')

    for i, d in enumerate(repo_data):
        gx = margin_l + i * group_w + group_w / 2
        x_naive = gx - bar_w - gap / 2
        x_sfs   = gx + gap / 2

        yn = y(d["naive_turns"]); ys = y(d["sfs_turns"])
        bh_n = margin_t + plot_h - yn; bh_s = margin_t + plot_h - ys

        lines.append(f'<rect x="{x_naive:.1f}" y="{yn:.1f}" width="{bar_w:.1f}" height="{bh_n:.1f}" '
                     f'fill="{NAIVE_COLOR}" rx="3"/>')
        lines.append(f'<rect x="{x_sfs:.1f}" y="{ys:.1f}" width="{bar_w:.1f}" height="{bh_s:.1f}" '
                     f'fill="{SFS_COLOR}" rx="3"/>')

        lines.append(f'<text x="{x_naive + bar_w/2:.1f}" y="{yn - 4:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{TEXT}">{d["naive_turns"]}</text>')
        lines.append(f'<text x="{x_sfs + bar_w/2:.1f}" y="{ys - 4:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{TEXT}">{d["sfs_turns"]}</text>')

        # Accuracy badge (always 4/4 = ✓ 100%)
        acc_n = f'{d["naive_acc"]}/{d["total"]}'
        acc_s = f'{d["sfs_acc"]}/{d["total"]}'
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 20:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{GREEN}" font-weight="600">'
                     f'✓ {acc_n} / {acc_s}</text>')

        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 38:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="12" font-weight="600" fill="{TEXT}">{d["label"]}</text>')
        lines.append(f'<text x="{gx:.1f}" y="{margin_t + plot_h + 54:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="10" fill="{SUBTEXT}">naive / sfs accuracy</text>')

    lines.append(f'<text x="14" y="{H//2}" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{SUBTEXT}" transform="rotate(-90,14,{H//2})">Turns</text>')

    lx, ly = margin_l, margin_t - 8
    for color, label in [(NAIVE_COLOR, "Naive (Bash only)"), (SFS_COLOR, "Claude + SemanticFS")]:
        lines.append(f'<rect x="{lx}" y="{ly - 10}" width="14" height="14" fill="{color}" rx="2"/>')
        lines.append(f'<text x="{lx + 18}" y="{ly + 2}" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="12" fill="{TEXT}">{label}</text>')
        lx += 180

    lines.append('</svg>')
    return "\n".join(lines)


# ── Chart 4: Context savings by repo size (scatter / line) ────────────────────

def chart_savings_by_size():
    """Show cost savings % vs approximate source-file count."""
    W, H = 700, 360
    ml, mr, mt, mb = 80, 40, 60, 70
    pw = W - ml - mr
    ph = H - mt - mb

    # Source file counts (excluding node_modules/.venv/etc.)
    points = [
        {"label": "prizePicksAI\n5 src files", "files": 5,    "pct": None},
        {"label": "KalshiTrading\n17 src files","files": 17,   "pct": None},
        {"label": "syntaxless\n95+ src files",  "files": 95,   "pct": None},
        {"label": "buckit\n70+ src files",      "files": 70,   "pct": None},
    ]

    for repo, pt in zip(REPOS, points):
        n_runs = get_runs(repo["id"], "naive")
        s_runs = get_runs(repo["id"], "semanticfs")
        if not n_runs or not s_runs:
            continue
        nc = total_cost(n_runs)
        sc = total_cost(s_runs)
        pt["pct"] = (nc - sc) / nc * 100 if nc > 0 else 0

    max_files = 110
    min_pct, max_pct = -50, 50

    def px(files):
        return ml + (files / max_files) * pw

    def py(pct):
        return mt + (1 - (pct - min_pct) / (max_pct - min_pct)) * ph

    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">']
    lines.append(f'<rect width="{W}" height="{H}" fill="{BG}"/>')

    lines.append(f'<text x="{W//2}" y="30" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="15" font-weight="600" fill="{TEXT}">Cost Savings vs Codebase Size</text>')
    lines.append(f'<text x="{W//2}" y="46" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{SUBTEXT}">SemanticFS helps most on large repos with many source files</text>')

    # Zero line
    y0 = py(0)
    lines.append(f'<line x1="{ml}" y1="{y0:.1f}" x2="{ml+pw}" y2="{y0:.1f}" '
                 f'stroke="#AAAAAA" stroke-width="1.5" stroke-dasharray="6,4"/>')
    lines.append(f'<text x="{ml + pw + 6}" y="{y0 + 4:.1f}" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="10" fill="{SUBTEXT}">0%</text>')

    # Y grid
    for pct_tick in [-40, -20, 0, 20, 40]:
        yy = py(pct_tick)
        lines.append(f'<line x1="{ml}" y1="{yy:.1f}" x2="{ml+pw}" y2="{yy:.1f}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{ml - 8}" y="{yy + 4:.1f}" text-anchor="end" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{SUBTEXT}">{pct_tick:+d}%</text>')

    # X grid
    for file_tick in [0, 20, 40, 60, 80, 100]:
        xx = px(file_tick)
        lines.append(f'<line x1="{xx:.1f}" y1="{mt}" x2="{xx:.1f}" y2="{mt+ph}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        lines.append(f'<text x="{xx:.1f}" y="{mt + ph + 18:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="11" fill="{SUBTEXT}">{file_tick}</text>')

    # Shaded regions
    lines.append(f'<rect x="{ml}" y="{mt}" width="{pw}" height="{y0 - mt:.1f}" '
                 f'fill="{GREEN}" opacity="0.07"/>')
    lines.append(f'<rect x="{ml}" y="{y0:.1f}" width="{pw}" height="{mt + ph - y0:.1f}" '
                 f'fill="{ORANGE}" opacity="0.07"/>')

    # SFS advantage / disadvantage labels
    lines.append(f'<text x="{ml + 8}" y="{mt + 16:.1f}" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{GREEN}" opacity="0.8">SemanticFS saves cost ▲</text>')
    lines.append(f'<text x="{ml + 8}" y="{mt + ph - 6:.1f}" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{ORANGE}" opacity="0.8">Overhead exceeds savings ▼</text>')

    # Draw trend line (linear fit)
    valid = [(p["files"], p["pct"]) for p in points if p["pct"] is not None]
    if len(valid) >= 2:
        xs = [v[0] for v in valid]
        ys = [v[1] for v in valid]
        n = len(xs)
        sx = sum(xs); sy = sum(ys); sxx = sum(x**2 for x in xs); sxy = sum(x*y for x,y in zip(xs,ys))
        slope = (n*sxy - sx*sy) / (n*sxx - sx**2) if (n*sxx - sx**2) != 0 else 0
        intercept = (sy - slope * sx) / n
        x1, x2 = min(xs), max(xs)
        y1, y2 = slope*x1 + intercept, slope*x2 + intercept
        lines.append(f'<line x1="{px(x1):.1f}" y1="{py(y1):.1f}" x2="{px(x2):.1f}" y2="{py(y2):.1f}" '
                     f'stroke="#888888" stroke-width="1.5" stroke-dasharray="4,3"/>')

    # Points
    for p in points:
        if p["pct"] is None:
            continue
        x = px(p["files"]); yy = py(p["pct"])
        color = GREEN if p["pct"] > 0 else ORANGE
        lines.append(f'<circle cx="{x:.1f}" cy="{yy:.1f}" r="8" fill="{color}" stroke="white" stroke-width="2"/>')
        pct_label = f'{p["pct"]:+.1f}%'
        lines.append(f'<text x="{x:.1f}" y="{yy + 4:.1f}" text-anchor="middle" '
                     f'font-family="Inter,system-ui,sans-serif" font-size="9" font-weight="700" fill="white">{pct_label}</text>')
        # repo label
        label_y = yy - 16 if p["pct"] > 0 else yy + 26
        for j, ln in enumerate(p["label"].split("\n")):
            lines.append(f'<text x="{x:.1f}" y="{label_y + j*13:.1f}" text-anchor="middle" '
                         f'font-family="Inter,system-ui,sans-serif" font-size="10" fill="{TEXT}">{ln}</text>')

    # Axis labels
    lines.append(f'<text x="{ml + pw//2}" y="{mt + ph + 48:.1f}" text-anchor="middle" '
                 f'font-family="Inter,system-ui,sans-serif" font-size="12" fill="{SUBTEXT}">Source File Count (excluding node_modules, .venv, etc.)</text>')
    lines.append(f'<text x="14" y="{H//2}" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="11" fill="{SUBTEXT}" transform="rotate(-90,14,{H//2})">Cost Savings (%)</text>')

    lines.append('</svg>')
    return "\n".join(lines)


# ── Chart 5: Summary scorecard ────────────────────────────────────────────────

def chart_scorecard():
    """A clean summary tile showing grand totals."""
    W, H = 820, 180

    # Compute grand totals
    all_naive = [r for r in results if r["mode"] == "naive"]
    all_sfs   = [r for r in results if r["mode"] == "semanticfs"]

    total_naive_cost  = sum(r["cost_usd"] for r in all_naive)
    total_sfs_cost    = sum(r["cost_usd"] for r in all_sfs)
    cost_pct = (total_naive_cost - total_sfs_cost) / total_naive_cost * 100

    naive_full = sum(r["input_tokens_fresh"] + r["input_tokens_cache_write"] + r["input_tokens_cache_read"] for r in all_naive)
    sfs_full   = sum(r["input_tokens_fresh"] + r["input_tokens_cache_write"] + r["input_tokens_cache_read"] for r in all_sfs)
    ctx_pct = (naive_full - sfs_full) / naive_full * 100

    naive_acc = sum(1 for r in all_naive if r["found_correct"])
    sfs_acc   = sum(1 for r in all_sfs if r["found_correct"])
    n = len(all_naive)

    naive_turns = sum(r["turns"] for r in all_naive)
    sfs_turns   = sum(r["turns"] for r in all_sfs)

    lines = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">']
    lines.append(f'<rect width="{W}" height="{H}" rx="8" fill="{BG}" stroke="{GRID}" stroke-width="1.5"/>')

    lines.append(f'<text x="{W//2}" y="30" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                 f'font-size="14" font-weight="700" fill="{TEXT}">Multi-Repo Benchmark Summary — 4 repos × 4 tasks × 2 modes = 32 API calls</text>')

    tiles = [
        {"label": "Accuracy",       "naive": f"{naive_acc}/{n}",          "sfs": f"{sfs_acc}/{n}",            "pct": None, "pct_color": GREEN},
        {"label": "Total Cost",     "naive": f"${total_naive_cost:.3f}",   "sfs": f"${total_sfs_cost:.3f}",    "pct": cost_pct, "pct_color": GREEN if cost_pct > 0 else ORANGE},
        {"label": "Total Context",  "naive": f"{naive_full//1000:.0f}k",   "sfs": f"{sfs_full//1000:.0f}k",    "pct": ctx_pct,  "pct_color": GREEN if ctx_pct > 0 else ORANGE},
        {"label": "Total Turns",    "naive": str(naive_turns),             "sfs": str(sfs_turns),              "pct": None, "pct_color": GREEN},
    ]

    tile_w = W / len(tiles)
    for i, t in enumerate(tiles):
        tx = i * tile_w
        cx = tx + tile_w / 2
        # separator
        if i > 0:
            lines.append(f'<line x1="{tx}" y1="45" x2="{tx}" y2="{H-10}" stroke="{GRID}" stroke-width="1"/>')

        lines.append(f'<text x="{cx:.1f}" y="60" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="12" fill="{SUBTEXT}">{t["label"]}</text>')

        # Naive row
        lines.append(f'<text x="{cx:.1f}" y="82" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="11" fill="{SUBTEXT}">Naive:</text>')
        lines.append(f'<text x="{cx:.1f}" y="100" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="20" font-weight="700" fill="{NAIVE_COLOR}">{t["naive"]}</text>')

        # SFS row
        lines.append(f'<text x="{cx:.1f}" y="120" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="11" fill="{SUBTEXT}">SemanticFS:</text>')
        lines.append(f'<text x="{cx:.1f}" y="140" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                     f'font-size="20" font-weight="700" fill="{SFS_COLOR}">{t["sfs"]}</text>')

        # pct
        if t["pct"] is not None:
            sign = "▼" if t["pct"] > 0 else "▲"
            label = f'{sign} {abs(t["pct"]):.1f}%'
            lines.append(f'<text x="{cx:.1f}" y="162" text-anchor="middle" font-family="Inter,system-ui,sans-serif" '
                         f'font-size="13" font-weight="700" fill="{t["pct_color"]}">{label}</text>')

    lines.append('</svg>')
    return "\n".join(lines)


# ── Write all charts ──────────────────────────────────────────────────────────

if __name__ == "__main__":
    charts = {
        "multi_repo_cost.svg":     chart_cost_comparison(),
        "multi_repo_context.svg":  chart_full_context(),
        "multi_repo_turns.svg":    chart_turns(),
        "multi_repo_savings.svg":  chart_savings_by_size(),
        "multi_repo_scorecard.svg": chart_scorecard(),
    }

    for name, svg in charts.items():
        path = OUT_DIR / name
        path.write_text(svg, encoding="utf-8")
        print(f"  wrote {path}")

    print(f"\nAll charts written to {OUT_DIR}")
