#!/usr/bin/env bash
# Run latest taskfmt validation/verification for one host-local subagent task.
set -euo pipefail

TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
REPO_ROOT="$(git rev-parse --show-toplevel)"
CATALOG_ROOT="${TC_CATALOG_ROOT:-$REPO_ROOT/refactoring-tasks/terminal-components/completion}"
WORKTREE="${TC_TASK_WORKTREE:-$REPO_ROOT}"
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

require_contexts() {
  local dir="$1"
  python3 - "$dir/verify.toml" "$RUN_DIR" <<'PY' || die "immutable verifier contexts are missing or unsafe"
import json
import re
import shlex
import sys
import tomllib
from pathlib import Path

verify_path = Path(sys.argv[1])
run_dir = Path(sys.argv[2])
with verify_path.open("rb") as stream:
    document = tomllib.load(stream)

references = set()
for check in document.get("checks", []):
    command = check.get("shell")
    if command is None:
        command = " ".join(shlex.quote(str(value)) for value in check.get("argv", []))
    references.update(re.findall(r"\$RUN_DIR/contexts/(CHK-[0-9]{3}\.json)", command))
if not references:
    raise SystemExit("task has no external proof contexts")

context_dir = run_dir / "contexts"
if not context_dir.is_dir() or context_dir.is_symlink():
    raise SystemExit("context directory is absent or symlinked")
actual = {path.name for path in context_dir.iterdir()}
if actual != references:
    raise SystemExit(f"context set mismatch: expected {sorted(references)}, got {sorted(actual)}")
for name in sorted(references):
    path = context_dir / name
    if path.is_symlink() or not path.is_file():
        raise SystemExit(f"unsafe context file: {path}")
    with path.open() as stream:
        value = json.load(stream)
    if not isinstance(value, dict):
        raise SystemExit(f"context is not a JSON object: {path}")
    if not isinstance(value.get("run_id"), str) or not value["run_id"]:
        raise SystemExit(f"context has no run_id: {path}")
run_ids = {json.loads((context_dir / name).read_text())["run_id"] for name in references}
if len(run_ids) != 1:
    raise SystemExit("contexts are not bound to one verifier run")
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
  require_scope_base
  require_external_run_dir
  require_clean_worktree
  require_native_proof
  require_contexts "$dir"
  [[ ! -e "$RUN_DIR/taskfmt-logs" ]] \
    || die "taskfmt log directory already exists; verifier run is not fresh"
  mkdir "$RUN_DIR/taskfmt-logs"
  export RUN_DIR TC_TASKFMT="$TASKFMT" TC_TASKFMT_SOURCE="$TASKFMT_SOURCE"
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
