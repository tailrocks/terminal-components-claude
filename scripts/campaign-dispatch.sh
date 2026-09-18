#!/usr/bin/env bash
# Run latest taskfmt validation/verification for one host-local subagent task.
set -euo pipefail

TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
ORACLE_TAG="refs/tags/visual-baseline"
ORACLE_COMMIT="4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
REPO_ROOT="$(git rev-parse --show-toplevel)"
CATALOG_ROOT="${TC_CATALOG_ROOT:-$REPO_ROOT/refactoring-tasks/terminal-components/completion}"
WORKTREE="${TC_TASK_WORKTREE:-}"
RUN_DIR="${TC_TASK_RUN_DIR:-}"
BASE="${TC_TASK_BASE:-}"

usage() {
  cat <<'EOF'
Usage: campaign-dispatch.sh <lint|verify|help>

Required:
  TASK=001                 Numbered task package.

Optional:
  TC_TASKFMT               Path to the exact pinned standalone taskfmt executable.
  TC_CATALOG_ROOT          Catalog root (default: refactoring-tasks/terminal-components/completion).
  TC_TASK_WORKTREE         Required isolated subagent worktree for verify.
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
  [[ "$TASKFMT_SOURCE" = /* ]] || die "taskfmt source must be absolute: $TASKFMT_SOURCE"
}

task_dir() {
  local task="${TASK:-}"
  [[ "$task" =~ ^[0-9]{3}$ ]] || die "set TASK=NNN"
  echo "$CATALOG_ROOT/$task"
}

proof_binary() {
  echo "$WORKTREE/target/debug/tc-proof"
}

require_native_proof() {
  local binary receipt commit
  binary="$(proof_binary)"
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || die "native tc-proof comparator missing: $binary (run scripts/campaign-build-proof.sh in this worktree)"
  receipt="$binary.build.json"
  [[ -f "$receipt" && ! -L "$receipt" ]] \
    || die "native tc-proof build receipt missing: $receipt (run scripts/campaign-build-proof.sh)"
  commit="$(git -C "$WORKTREE" rev-parse HEAD)"
  python3 - "$receipt" "$WORKTREE" "$commit" "$binary" <<'PY' \
    || die "native tc-proof build receipt does not match this worktree"
import hashlib
import json
import sys
from pathlib import Path

receipt, worktree, commit, binary = sys.argv[1:]
with Path(receipt).open() as stream:
    value = json.load(stream)
if value.get("schema") != "tc-proof-native-build/v1":
    raise SystemExit("wrong build receipt schema")
if value.get("worktree") != str(Path(worktree).resolve()):
    raise SystemExit("wrong build worktree")
if value.get("commit") != commit:
    raise SystemExit("wrong build commit")
if value.get("binary") != str(Path(binary).resolve()):
    raise SystemExit("wrong build binary")
actual = hashlib.sha256(Path(binary).read_bytes()).hexdigest()
if value.get("binary_sha256") != actual:
    raise SystemExit("native comparator hash mismatch")
PY
}

require_scope_base() {
  [[ "$BASE" =~ ^[0-9a-f]{40}$ ]] \
    || die "set TC_TASK_BASE to the explicit full scope-base commit"
  local resolved
  resolved="$(git -C "$WORKTREE" rev-parse --verify "${BASE}^{commit}" 2>/dev/null)" \
    || die "scope base is not a commit in the verifier worktree: $BASE"
  [[ "$resolved" == "$BASE" ]] \
    || die "scope base must be an explicit full commit, not an abbreviation: $BASE"
}

require_clean_worktree() {
  [[ -z "$(git -C "$WORKTREE" status --porcelain)" ]] \
    || die "subagent worktree has uncommitted changes; verify only a committed task tree"
}

require_external_run_dir() {
  python3 - "$RUN_DIR" "$WORKTREE" <<'PY' || die "verifier run directory is missing, reused, or inside the worktree"
import sys
from pathlib import Path

run = Path(sys.argv[1])
worktree = Path(sys.argv[2]).resolve()
if not run.is_absolute() or run.is_symlink() or not run.is_dir():
    raise SystemExit("run directory must be an existing regular directory")
if run.resolve() == worktree or worktree in run.resolve().parents:
    raise SystemExit("run directory must be external to the candidate worktree")
if not run.parent.is_dir() or run.parent.is_symlink():
    raise SystemExit("run directory parent must be a real directory")
PY
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
  [[ -z "$(git -C "$TASKFMT_SOURCE" status --porcelain)" ]] \
    || die "taskfmt source is dirty: $TASKFMT_SOURCE"
  [[ "$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)" == "$TASKFMT_REV" ]] \
    || die "taskfmt source is not latest $TASKFMT_REV"
}

prepare_native_contexts() {
  local dir="$1"
  local binary
  binary="$(proof_binary)"
  local -a command=(
    "$binary" prepare
    --task-dir "$dir"
    --run-dir "$RUN_DIR"
    --worktree "$WORKTREE"
    --scope-base "$BASE"
    --oracle-tag "$ORACLE_TAG"
    --oracle-commit "$ORACLE_COMMIT"
    --tool "$WORKTREE/tools/refactor-proof/bin/tc-proof"
    --comparator "$binary"
    --taskfmt "$TASKFMT"
  )
  if [[ -n "${TC_TASK_DEPENDENCY_RECEIPTS:-}" ]]; then
    local receipt
    local -a receipts
    IFS=: read -r -a receipts <<< "$TC_TASK_DEPENDENCY_RECEIPTS"
    for receipt in "${receipts[@]}"; do
      [[ -n "$receipt" ]] || die "dependency receipt list contains an empty path"
      command+=(--dependency-receipt "$receipt")
    done
  fi
  "${command[@]}" >/dev/null || die "native verifier preparation failed"
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
  local dir binary
  dir="$(task_dir)"
  binary="$(proof_binary)"
  [[ -n "$WORKTREE" && "$WORKTREE" = /* ]] \
    || die "set TC_TASK_WORKTREE to an absolute isolated worktree"
  [[ -d "$WORKTREE" ]] || die "subagent worktree missing: $WORKTREE"
  [[ -n "$RUN_DIR" && "$RUN_DIR" = /* ]] \
    || die "set TC_TASK_RUN_DIR to a unique absolute verifier run directory"
  require_scope_base
  require_external_run_dir
  require_clean_worktree
  require_native_proof
  prepare_native_contexts "$dir"
  export TC_PROOF_CONTEXT_INDEX="$RUN_DIR/context-index.json"
  export TC_PROOF_CONTEXT_INDEX_SHA256="$(shasum -a 256 "$TC_PROOF_CONTEXT_INDEX" | awk '{print $1}')"
  export RUN_DIR TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
  export TC_PROOF_NATIVE_LAUNCH=1
  export TC_PROOF_NATIVE_LAUNCHER="$binary"
  export TC_PROOF_NATIVE_CHILD=0
  export TC_PROOF_NATIVE_TIMEOUT_MS="${TC_PROOF_NATIVE_TIMEOUT_MS:-600000}"

  local taskfmt_status=0
  "$TASKFMT" verify \
    --root "$WORKTREE" \
    --task-dir "$dir" \
    --base "$BASE" \
    --progress "" \
    --log-dir "$RUN_DIR/taskfmt-logs" \
    || taskfmt_status=$?

  local validate_status=0
  "$binary" validate --run-dir "$RUN_DIR" || validate_status=$?
  if (( taskfmt_status != 0 )); then
    return "$taskfmt_status"
  fi
  (( validate_status == 0 )) || die "native proof validation failed after taskfmt"
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
