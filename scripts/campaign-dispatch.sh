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
  TC_PROOF_TARGET_DIR      Existing external native-proof Cargo target directory.

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

resolve_target_dir() {
  local worktree_root="$1"
  local tc_target_dir="${TC_PROOF_TARGET_DIR:-}"
  local cargo_target="${CARGO_TARGET_DIR:-}"
  python3 - "$worktree_root" "$tc_target_dir" "$cargo_target" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
tc_raw, cargo_raw = sys.argv[2:]

def checked_path(raw: str, name: str) -> Path:
    path = Path(raw)
    if not path.is_absolute():
        raise SystemExit(f"{name} must be an absolute path: {raw or '<empty>'}")
    if path.is_symlink():
        raise SystemExit(f"{name} must not be a symlink: {path}")
    return path

if tc_raw and cargo_raw:
    tc_path = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
    cargo_path = checked_path(cargo_raw, "CARGO_TARGET_DIR")
    if tc_path.resolve() != cargo_path.resolve():
        raise SystemExit(
            "TC_PROOF_TARGET_DIR and CARGO_TARGET_DIR select ambiguous target roots"
        )
    selected = tc_path
elif tc_raw:
    selected = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
elif cargo_raw:
    selected = checked_path(cargo_raw, "CARGO_TARGET_DIR")
else:
    selected = root / "target"

if selected.is_symlink():
    raise SystemExit(f"proof target directory must not be a symlink: {selected}")
if not selected.exists():
    raise SystemExit(f"proof target directory is missing: {selected}")
if not selected.is_dir():
    raise SystemExit(f"proof target path is not a directory: {selected}")

resolved = selected.resolve()
if resolved == root or root in resolved.parents:
    raise SystemExit("proof target directory must be external to the worktree")
print(resolved)
PY
}

proof_binary() {
  local target_dir="$1"
  echo "$target_dir/debug/tc-proof"
}

require_native_proof() {
  local worktree_root="$1"
  local target_dir="$2"
  local binary="$3"
  local receipt commit tree
  [[ -d "$target_dir" && ! -L "$target_dir" ]] \
    || die "native proof target directory missing or linked: $target_dir"
  [[ -d "$target_dir/debug" && ! -L "$target_dir/debug" ]] \
    || die "native proof debug target directory missing or linked: $target_dir/debug"
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || die "native tc-proof comparator missing: $binary (run scripts/campaign-build-proof.sh with TC_PROOF_TARGET_DIR)"
  receipt="$target_dir/debug/tc-proof.build.json"
  [[ -f "$receipt" && ! -L "$receipt" ]] \
    || die "native tc-proof build receipt missing: $receipt (run scripts/campaign-build-proof.sh)"
  commit="$(git -C "$worktree_root" rev-parse HEAD)"
  tree="$(git -C "$worktree_root" rev-parse 'HEAD^{tree}')"
  python3 - "$receipt" "$worktree_root" "$target_dir" "$commit" "$tree" "$binary" <<'PY' \
    || die "native tc-proof build receipt does not match this worktree/target"
import hashlib
import json
import sys
from pathlib import Path

receipt, worktree, target, commit, tree, binary = sys.argv[1:]
try:
    with Path(receipt).open(encoding="utf-8") as stream:
        value = json.load(stream)
except (OSError, ValueError) as error:
    raise SystemExit(f"invalid proof build receipt: {error}") from error

if value.get("schema") != "tc-proof-native-build/v1":
    raise SystemExit("wrong build receipt schema")
if value.get("worktree") != str(Path(worktree).resolve()):
    raise SystemExit("wrong build worktree")
expected_target = str(Path(target).resolve())
if value.get("target_dir") != expected_target or value.get("cargo_target_dir") != expected_target:
    raise SystemExit("wrong build target")
if value.get("commit") != commit or value.get("tree") != tree:
    raise SystemExit("wrong build source binding")
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
  local binary="$2"
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
  local dir binary target_dir worktree_root
  dir="$(task_dir)"
  [[ -n "$WORKTREE" && "$WORKTREE" = /* ]] \
    || die "set TC_TASK_WORKTREE to an absolute isolated worktree"
  [[ -d "$WORKTREE" ]] || die "subagent worktree missing: $WORKTREE"
  [[ -n "$RUN_DIR" && "$RUN_DIR" = /* ]] \
    || die "set TC_TASK_RUN_DIR to a unique absolute verifier run directory"
  worktree_root="$(git -C "$WORKTREE" rev-parse --show-toplevel)" \
    || die "candidate worktree is not a Git worktree: $WORKTREE"
  require_scope_base
  require_external_run_dir
  require_clean_worktree
  target_dir="$(resolve_target_dir "$worktree_root")" \
    || die "native proof target directory is invalid"
  binary="$(proof_binary "$target_dir")"
  export TC_PROOF_TARGET_DIR="$target_dir" CARGO_TARGET_DIR="$target_dir"
  require_native_proof "$worktree_root" "$target_dir" "$binary"
  prepare_native_contexts "$dir" "$binary"
  export TC_PROOF_CONTEXT_INDEX="$RUN_DIR/context-index.json"
  local context_index_sha256
  context_index_sha256="$(shasum -a 256 "$TC_PROOF_CONTEXT_INDEX" | awk '{print $1}')"
  export TC_PROOF_CONTEXT_INDEX_SHA256="$context_index_sha256"
  export RUN_DIR TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
  export TC_PROOF_NATIVE_LAUNCH=1
  export TC_PROOF_NATIVE_BINARY="$binary"
  export TC_PROOF_NATIVE_LAUNCHER="$binary"
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
