#!/usr/bin/env bash
# Run latest taskfmt validation/verification for one host-local subagent task.
set -euo pipefail

TASKFMT_REV="${TASKFMT_REV:-afd3b575dbcc7044620bec4b9493a74eca3e5ef2}"
TASKFMT_SHA256="${TASKFMT_SHA256:-f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de}"
TASKFMT_SOURCE="${TC_TASKFMT_SOURCE:-/Users/donbeave/Projects/taskfmt/task-format}"
TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
CATALOG_ROOT="${TC_CATALOG_ROOT:-$REPO_ROOT/refactoring-tasks/terminal-components/completion}"
WORKTREE="${TC_TASK_WORKTREE:-$REPO_ROOT}"
RUN_DIR="${TC_TASK_RUN_DIR:-}"
BASE="${TC_TASK_BASE:-$(git -C "$WORKTREE" rev-parse HEAD)}"

usage() {
  cat <<'EOF'
Usage: campaign-dispatch.sh <lint|verify|help>

Required:
  TASK=001                 Numbered task package.

Optional:
  TC_TASKFMT               Pinned standalone taskfmt executable.
  TASKFMT_SHA256           Exact pinned taskfmt executable SHA-256.
  TC_TASKFMT_SOURCE        Exact taskfmt source checkout.
  TC_CATALOG_ROOT          Catalog root (default: refactoring-tasks/terminal-components/completion).
  TC_TASK_WORKTREE         Isolated subagent worktree (default: current directory).
  TC_TASK_RUN_DIR          Required for verify; unique external run/log directory.
  TC_TASK_BASE             Immutable task scope base commit.

Only the standalone taskfmt lint and verify commands are allowed. This script
never starts containers, invokes taskfmt lifecycle binaries, or integrates refs.
EOF
}

die() {
  echo "campaign-dispatch: $*" >&2
  exit 1
}

require_absolute_paths() {
  [[ "$CATALOG_ROOT" = /* ]] || die "catalog root must be absolute: $CATALOG_ROOT"
  [[ "$WORKTREE" = /* ]] || die "subagent worktree must be absolute: $WORKTREE"
  [[ "$TASKFMT_SOURCE" = /* ]] || die "taskfmt source must be absolute: $TASKFMT_SOURCE"
}

task_dir() {
  local task="${TASK:-}"
  [[ "$task" =~ ^[0-9]{3}$ ]] || die "set TASK=NNN"
  echo "$CATALOG_ROOT/$task"
}

require_taskfmt() {
  require_absolute_paths
  [[ -x "$TASKFMT" ]] || die "taskfmt not executable: $TASKFMT"
  local actual_sha
  actual_sha="$(shasum -a 256 "$TASKFMT" | awk '{print $1}')"
  [[ "$actual_sha" == "$TASKFMT_SHA256" ]] \
    || die "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
  "$TASKFMT" --version | grep -Fq "git $TASKFMT_REV" \
    || die "taskfmt is not latest $TASKFMT_REV"
  [[ -d "$TASKFMT_SOURCE/.git" ]] \
    || die "taskfmt source is not a git checkout: $TASKFMT_SOURCE"
  [[ "$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)" == "$TASKFMT_REV" ]] \
    || die "taskfmt source is not latest $TASKFMT_REV"
}

cmd_lint() {
  require_taskfmt
  local dir
  dir="$(task_dir)"
  export TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
  "$TASKFMT" lint "$dir"
}

cmd_verify() {
  require_taskfmt
  local dir
  dir="$(task_dir)"
  [[ -d "$WORKTREE" ]] || die "subagent worktree missing: $WORKTREE"
  [[ -n "$RUN_DIR" && "$RUN_DIR" = /* ]] \
    || die "set TC_TASK_RUN_DIR to a unique absolute verifier run directory"
  mkdir -p "$RUN_DIR/taskfmt-logs"
  export TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
  "$TASKFMT" verify \
    --root "$WORKTREE" \
    --task-dir "$dir" \
    --base "$BASE" \
    --progress "" \
    --log-dir "$RUN_DIR/taskfmt-logs"
}

main() {
  case "${1:-help}" in
    lint) cmd_lint ;;
    verify) cmd_verify ;;
    help|-h|--help) usage ;;
    *) die "unknown command: ${1:-}" ;;
  esac
}

main "$@"
