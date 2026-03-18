#!/usr/bin/env bash
# Apply a SemanticFS config profile (Linux / macOS equivalent of apply_config_profile.ps1).
#
# Usage:
#   bash scripts/apply_config_profile.sh [OPTIONS]
#
# Options:
#   --profile     single-repo | multi-root-dev-box | home-projects  (default: single-repo)
#   --output      path for the generated TOML config                 (default: local.toml)
#   --repo-root   absolute path to the project repo                  (default: cwd)
#   --home-root   absolute home directory path                       (default: $HOME)
#   --projects-root  projects directory path                         (default: $HOME/projects)
#   --mount-point    FUSE mount point (Linux only)                   (default: /mnt/ai)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILES_DIR="${SCRIPT_DIR}/../config/profiles"

# ── Defaults ─────────────────────────────────────────────────────────────────
PROFILE="single-repo"
OUTPUT="local.toml"
REPO_ROOT="$(pwd)"
HOME_ROOT="${HOME}"
PROJECTS_ROOT="${HOME}/projects"
MOUNT_POINT="/mnt/ai"

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)       PROFILE="$2";        shift 2 ;;
    --output)        OUTPUT="$2";         shift 2 ;;
    --repo-root)     REPO_ROOT="$2";      shift 2 ;;
    --home-root)     HOME_ROOT="$2";      shift 2 ;;
    --projects-root) PROJECTS_ROOT="$2";  shift 2 ;;
    --mount-point)   MOUNT_POINT="$2";    shift 2 ;;
    -h|--help)
      sed -n '2,20p' "$0" | sed 's/^# //'
      exit 0
      ;;
    *)
      echo "Unknown flag: $1" >&2
      exit 1
      ;;
  esac
done

# ── Resolve template ──────────────────────────────────────────────────────────
case "${PROFILE}" in
  single-repo)        TEMPLATE="${PROFILES_DIR}/single-repo.sample.toml" ;;
  multi-root-dev-box) TEMPLATE="${PROFILES_DIR}/multi-root-dev-box.sample.toml" ;;
  home-projects)      TEMPLATE="${PROFILES_DIR}/home-projects.sample.toml" ;;
  *)
    echo "Unknown profile: ${PROFILE}. Choose: single-repo, multi-root-dev-box, home-projects" >&2
    exit 1
    ;;
esac

if [[ ! -f "${TEMPLATE}" ]]; then
  echo "Profile template not found: ${TEMPLATE}" >&2
  exit 1
fi

# ── Substitute placeholders ───────────────────────────────────────────────────
OUTPUT_DIR="$(dirname "${OUTPUT}")"
if [[ -n "${OUTPUT_DIR}" && "${OUTPUT_DIR}" != "." ]]; then
  mkdir -p "${OUTPUT_DIR}"
fi

sed \
  -e "s|__REPO_ROOT__|${REPO_ROOT}|g" \
  -e "s|__HOME_ROOT__|${HOME_ROOT}|g" \
  -e "s|__PROJECTS_ROOT__|${PROJECTS_ROOT}|g" \
  -e "s|__MOUNT_POINT__|${MOUNT_POINT}|g" \
  "${TEMPLATE}" > "${OUTPUT}"

# ── Summary ───────────────────────────────────────────────────────────────────
echo "Profile:       ${PROFILE}"
echo "Output:        $(realpath "${OUTPUT}" 2>/dev/null || echo "${OUTPUT}")"
echo "Repo root:     ${REPO_ROOT}"
echo "Home root:     ${HOME_ROOT}"
echo "Projects root: ${PROJECTS_ROOT}"
echo "Mount point:   ${MOUNT_POINT}"
