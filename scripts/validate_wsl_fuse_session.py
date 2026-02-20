#!/usr/bin/env python3
import argparse
import json
import pathlib
import subprocess
import sys
import time


def read_json(path: pathlib.Path, label: str) -> dict:
    st = path.stat()
    raw = path.read_text(encoding="utf-8").strip()
    tail = raw[-8:] if raw else ""
    print(f"{label} (len={len(raw)} st_size={st.st_size} tail={tail!r}): {raw!r}")
    return json.loads(raw)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate SemanticFS Linux FUSE session pin + refresh behavior."
    )
    parser.add_argument(
        "--repo",
        default="/mnt/c/Users/navneeth/Desktop/NavneethThings/Projects/semanticFS",
        help="Repo root inside WSL",
    )
    parser.add_argument(
        "--config",
        default="config/wsl-local.toml",
        help="Config path relative to repo root",
    )
    parser.add_argument(
        "--mountpoint",
        default="/tmp/semanticfs-mnt",
        help="Mounted semanticfs path",
    )
    parser.add_argument(
        "--wait-seconds",
        type=int,
        default=90,
        help="Max seconds to wait for mount readiness",
    )
    args = parser.parse_args()

    repo = pathlib.Path(args.repo)
    wk = pathlib.Path(args.mountpoint) / ".well-known"
    session_json = wk / "session.json"
    session_refresh = wk / "session.refresh"

    for _ in range(args.wait_seconds):
        if session_json.exists():
            break
        time.sleep(1)
    else:
        print(f"ERROR: mount not ready at {session_json}")
        return 2

    s1 = read_json(session_json, "session.json (before rebuild)")
    # Trigger active index version change while this process keeps the same pid.
    run = subprocess.run(
        [
            "cargo",
            "run",
            "--release",
            "-p",
            "semanticfs-cli",
            "--",
            "--config",
            args.config,
            "index",
            "build",
        ],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    print(f"index build exit={run.returncode}")
    if run.stdout:
        print(run.stdout.strip())
    if run.stderr:
        print(run.stderr.strip())
    if run.returncode != 0:
        return run.returncode

    s2 = read_json(session_json, "session.json (after rebuild, before refresh)")
    sr = read_json(session_refresh, "session.refresh")
    s3 = read_json(session_json, "session.json (after refresh)")

    if s1.get("mode") != "pinned":
        print("ERROR: mode is not pinned")
        return 3
    if sr.get("refreshed") is not True:
        print("ERROR: session.refresh did not set refreshed=true")
        return 4
    if s2.get("active_version") != s2.get("snapshot_version"):
        if s2.get("stale") is not True:
            print("ERROR: expected stale=true before refresh when active changed")
            return 5
    if s3.get("snapshot_version") != s3.get("active_version"):
        print("ERROR: snapshot_version != active_version after refresh")
        return 6
    if s3.get("stale") is not False:
        print("ERROR: stale should be false after refresh")
        return 7

    print("VALIDATION_OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
