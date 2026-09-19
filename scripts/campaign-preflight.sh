#!/usr/bin/env bash
# Pre-arm readiness checks. Does NOT arm /goal or dispatch tasks.
#
# `proof-preparation` is deliberately separate: it qualifies verifier-owned
# native inputs while the campaign is NO-GO, but it never reads or changes the
# campaign ledger and never authorizes a task.
set -euo pipefail

INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.}"
TAG_PEELED_EXPECT="${TAG_PEELED_EXPECT:-4a79c0a2d40fca46fc406b77157ce3b3f12ec16b}"
TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_VERSION="0.2.0"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
TASKFMT_BIN="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/campaign-preflight.sh [preflight|proof-preparation]

preflight             Authorizing readiness gate. Fails while the report is
                      NO-GO or any ledger/evidence binding is stale.
proof-preparation     Non-authorizing native verifier-input qualification.
                      Requires TC_PROOF_RUN_DIR and optionally
                      TC_PROOF_PREPARATION_RECEIPT; never reads or mutates
                      .campaign/ledger.json.
EOF
}

repo_root() {
  git -C "$SCRIPT_DIR/.." rev-parse --show-toplevel
}

fail() {
  echo "campaign-preflight: FAIL: $*" >&2
  exit 1
}

pass() {
  echo "campaign-preflight: OK: $*"
}

campaign_worktree() {
  local root
  root="$(repo_root)"
  if [[ "$WORKTREE_PATH" = /* ]]; then
    echo "$WORKTREE_PATH"
  else
    echo "$root/$WORKTREE_PATH"
  fi
}

check_tag() {
  local peeled
  peeled="$(git rev-parse "refs/tags/visual-baseline^{commit}" 2>/dev/null || echo MISSING)"
  if [[ "$peeled" != "$TAG_PEELED_EXPECT" ]]; then
    fail "visual-baseline tag peeled=$peeled (expected $TAG_PEELED_EXPECT)"
  else
    pass "visual-baseline tag unmoved @ ${peeled:0:12}"
  fi
}

check_branch() {
  git show-ref --verify --quiet "refs/heads/$INTEGRATION_BRANCH" \
    || fail "missing branch $INTEGRATION_BRANCH (run campaign-init.sh)"
  pass "branch $INTEGRATION_BRANCH @ $(git rev-parse "$INTEGRATION_BRANCH" | cut -c1-12)"
}

check_worktree() {
  local wt wt_branch branch_head wt_head
  wt="$(campaign_worktree)"
  [[ -d "$wt" ]] || fail "missing worktree $wt"
  wt_branch="$(git -C "$wt" branch --show-current 2>/dev/null || echo detached)"
  if [[ "$wt_branch" != "$INTEGRATION_BRANCH" ]]; then
    fail "worktree on '$wt_branch' (expected $INTEGRATION_BRANCH)"
  fi
  branch_head="$(git rev-parse "$INTEGRATION_BRANCH")"
  wt_head="$(git -C "$wt" rev-parse HEAD)"
  [[ "$wt_head" == "$branch_head" ]] \
    || fail "worktree HEAD $wt_head is not current branch HEAD $branch_head"
  pass "worktree $wt on $INTEGRATION_BRANCH @ ${wt_head:0:12}"
}

check_readiness_gate() {
  local report
  report="$(repo_root)/docs/refactoring-plan/execution-readiness-report.md"
  [[ -f "$report" ]] || fail "readiness report missing: $report"
  if grep -Eq '^\*\*NO-GO\.\*\*$' "$report"; then
    fail "readiness report is NO-GO; preflight cannot authorize campaign work"
  fi
  grep -Eq '^\*\*GO\.\*\*$' "$report" \
    || fail "readiness report has no exact GO verdict"
  pass "readiness report is GO"
}

check_ledger() {
  local root ledger graph wt current_head
  root="$(repo_root)"
  ledger="$root/.campaign/ledger.json"
  graph="$root/docs/refactoring-plan/task-graph.json"
  wt="$(campaign_worktree)"
  current_head="$(git -C "$wt" rev-parse HEAD)"
  [[ -f "$ledger" ]] || fail "missing $ledger (run campaign-init.sh)"
  [[ -f "$graph" ]] || fail "missing dependency graph: $graph"
  PYTHONPATH="$SCRIPT_DIR" python3 - "$ledger" "$INTEGRATION_BRANCH" \
    "$current_head" "$graph" "$root" "$TASKFMT_REV" "$TASKFMT_VERSION" \
    "$TASKFMT_SHA256" "$TASKFMT_SOURCE" "$TASKFMT_BIN" <<'PY' \
    || fail "ledger invalid or stale"
import json
import sys
from pathlib import Path

from campaign_ledger import validate_preflight_ledger

ledger_path, branch, current_head, graph_path, root, revision, version, digest, source, binary = sys.argv[1:]
with Path(ledger_path).open(encoding="utf-8") as stream:
    ledger = json.load(stream)
with Path(graph_path).open(encoding="utf-8") as stream:
    graph = json.load(stream)
expected_taskfmt = {
    "taskfmt_revision": revision,
    "taskfmt_version": version,
    "taskfmt_sha256": digest,
    "taskfmt_source": source,
    "taskfmt_path": str(Path(binary).resolve()),
}
validate_preflight_ledger(
    ledger,
    branch,
    current_head=current_head,
    expected_taskfmt=expected_taskfmt,
    dependency_graph=graph,
    repository_root=root,
)
print("ledger schema/receipt/head/taskfmt/dependency/result/reviewer bindings: OK")
PY
  pass "ledger valid, disarmed, and bound to current HEAD"
}

check_taskfmt_identity() {
  [[ "$TASKFMT_BIN" = /* ]] || fail "taskfmt path must be absolute: $TASKFMT_BIN"
  [[ -d "$TASKFMT_SOURCE/.git" ]] \
    || fail "taskfmt source is not a git checkout: $TASKFMT_SOURCE"
  [[ -z "$(git -C "$TASKFMT_SOURCE" status --porcelain)" ]] \
    || fail "taskfmt source is dirty: $TASKFMT_SOURCE"
  [[ "$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)" == "$TASKFMT_REV" ]] \
    || fail "taskfmt source is not latest $TASKFMT_REV"
  [[ -f "$TASKFMT_BIN" && ! -L "$TASKFMT_BIN" && -x "$TASKFMT_BIN" ]] \
    || fail "taskfmt is not a regular executable: $TASKFMT_BIN"
  local actual_sha
  actual_sha="$(shasum -a 256 "$TASKFMT_BIN" | awk '{print $1}')"
  [[ "$actual_sha" == "$TASKFMT_SHA256" ]] \
    || fail "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
  "$TASKFMT_BIN" --version | grep -Fq "git $TASKFMT_REV" \
    || fail "taskfmt is not latest $TASKFMT_REV"
  pass "current taskfmt identity @ $TASKFMT_BIN"
}

check_taskfmt() {
  check_taskfmt_identity
  local catalog task
  catalog="$(repo_root)/refactoring-tasks/terminal-components/completion"
  for task in "$catalog"/[0-9][0-9][0-9]; do
    "$TASKFMT_BIN" lint "$task" >/dev/null \
      || fail "latest taskfmt lint failed: $task"
  done
  pass "latest taskfmt lint passed for every numbered package"
}

check_host_local_task_paths() {
  local root
  root="$(repo_root)"
  if rg -n '(^|[" ])/(task|work|proof|run)(/|[" ])' \
    "$root/refactoring-tasks/terminal-components/completion" \
    --glob 'verify.toml' >/dev/null; then
    fail "task verify.toml still contains legacy container paths"
  fi
  pass "task verify.toml paths are host-local"
}

check_harness() {
  local wt
  wt="$(campaign_worktree)"
  if [[ -f "$wt/Cargo.toml" ]] && grep -q refactor-proof "$wt/Cargo.toml" 2>/dev/null; then
    if (cd "$wt" && NEXTEST_USER_CONFIG_FILE=none cargo nextest list -p refactor-proof >/dev/null 2>/dev/null); then
      pass "refactor-proof nextest discovery"
    else
      fail "refactor-proof nextest discovery failed in worktree"
    fi
  else
    fail "worktree missing refactor-proof"
  fi
}

check_native_proof() {
  local wt target_dir binary receipt commit tree tc_target cargo_target
  wt="$(campaign_worktree)"
  tc_target="${TC_PROOF_TARGET_DIR:-}"
  cargo_target="${CARGO_TARGET_DIR:-}"
  if ! target_dir="$(
    python3 - "$wt" "$tc_target" "$cargo_target" <<'PY'
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
  )"; then
    fail "native proof target is invalid"
  fi
binary="$target_dir/debug/tc-proof"
receipt="$target_dir/debug/tc-proof.build.json"
[[ -d "$target_dir/debug" && ! -L "$target_dir/debug" ]] \
    || fail "native proof debug target directory missing or linked: $target_dir/debug"
[[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "native tc-proof comparator missing or linked: $binary (run campaign-build-proof.sh with TC_PROOF_TARGET_DIR)"
[[ -f "$receipt" && ! -L "$receipt" ]] \
    || fail "native tc-proof build receipt missing or linked: $receipt (run campaign-build-proof.sh with TC_PROOF_TARGET_DIR)"
  commit="$(git -C "$wt" rev-parse HEAD)"
  tree="$(git -C "$wt" rev-parse 'HEAD^{tree}')"
  PYTHONPATH="$SCRIPT_DIR" python3 - "$receipt" "$wt" "$target_dir" "$commit" "$tree" "$binary" <<'PY' \
    || fail "native tc-proof build receipt does not match this worktree"
import hashlib
import json
import sys
from pathlib import Path

receipt, worktree, target, commit, tree, binary = sys.argv[1:]
with Path(receipt).open(encoding="utf-8") as stream:
    value = json.load(stream)
if value.get("schema") != "tc-proof-native-build/v1":
    raise SystemExit("wrong build receipt schema")
if value.get("worktree") != str(Path(worktree).resolve()):
    raise SystemExit("wrong build worktree")
if value.get("target_dir") != str(Path(target).resolve()):
    raise SystemExit("wrong build target")
if value.get("cargo_target_dir") != str(Path(target).resolve()):
    raise SystemExit("wrong Cargo target")
if value.get("commit") != commit:
    raise SystemExit("wrong build commit")
if value.get("tree") != tree:
    raise SystemExit("wrong build tree")
if value.get("binary") != str(Path(binary).resolve()):
    raise SystemExit("wrong build binary")
binary_path = Path(binary)
if binary_path.is_symlink() or not binary_path.is_file():
    raise SystemExit("native comparator is not a regular file")
receipt_path = Path(receipt)
if receipt_path.is_symlink() or not receipt_path.is_file():
    raise SystemExit("native build receipt is not a regular file")
actual = hashlib.sha256(binary_path.read_bytes()).hexdigest()
if value.get("binary_sha256") != actual:
    raise SystemExit("native comparator hash mismatch")
PY
  pass "native tc-proof comparator and build receipt match external target, worktree HEAD, and tree"
}

check_validate_plan() {
  local root script
  root="$(repo_root)"
  script="$root/docs/refactoring-plan/evidence/validate-plan.py"
  if [[ -f "$script" ]]; then
    if python3 "$script" --summary 2>/dev/null | grep -q '"error_count": 0'; then
      pass "validate-plan error_count 0"
    else
      fail "validate-plan not green or script failed"
    fi
  else
    fail "validate-plan.py not found"
  fi
}

check_proof_preparation() {
  local root wt run receipt current_head
  root="$(repo_root)"
  wt="$(campaign_worktree)"
  run="${TC_PROOF_RUN_DIR:-}"
  receipt="${TC_PROOF_PREPARATION_RECEIPT:-}"
  [[ -n "$run" && "$run" = /* ]] \
    || fail "set TC_PROOF_RUN_DIR to an external absolute preparation directory"
  if [[ -z "$receipt" ]]; then
    receipt="$run/proof-preparation.json"
  fi
  [[ "$receipt" = /* ]] || fail "proof preparation receipt must be absolute: $receipt"
  [[ -f "$receipt" && ! -L "$receipt" ]] \
    || fail "proof preparation receipt missing or symlinked: $receipt"
  [[ -z "$(git -C "$wt" status --porcelain)" ]] \
    || fail "proof preparation worktree is dirty"
  current_head="$(git -C "$wt" rev-parse HEAD)"
  check_taskfmt_identity
  check_native_proof
  PYTHONPATH="$SCRIPT_DIR" python3 - "$receipt" "$wt" "$current_head" "$run" \
    "$TASKFMT_REV" "$TASKFMT_VERSION" "$TASKFMT_SHA256" "$TASKFMT_SOURCE" \
    "$TASKFMT_BIN" <<'PY' \
    || fail "proof preparation is not bound, complete, and current"
import json
import sys
from pathlib import Path

from campaign_ledger import validate_proof_preparation

receipt_path, worktree, current_head, run_dir, revision, version, digest, source, binary = sys.argv[1:]
with Path(receipt_path).open(encoding="utf-8") as stream:
    preparation = json.load(stream)
expected_taskfmt = {
    "taskfmt_revision": revision,
    "taskfmt_version": version,
    "taskfmt_sha256": digest,
    "taskfmt_source": source,
    "taskfmt_path": str(Path(binary).resolve()),
}
validate_proof_preparation(
    preparation,
    worktree=worktree,
    current_head=current_head,
    run_dir=run_dir,
    expected_taskfmt=expected_taskfmt,
)
print("proof preparation context/index/result/observer bindings: OK")
PY
  pass "proof preparation qualified; non-authorizing (ledger and tasks unchanged)"
}

main() {
  local mode="${1:-preflight}"
  case "$mode" in
    proof-preparation|proof-prep|qualify-proof-preparation)
      root="$(repo_root)"
      cd "$root"
      echo "==> Native proof-preparation qualification (non-authorizing)"
      check_branch
      check_worktree
      check_proof_preparation
      ;;
    preflight|--preflight)
      [[ "$#" -le 1 ]] || { usage >&2; fail "unknown preflight arguments"; }
      root="$(repo_root)"
      cd "$root"
      echo "==> Pre-arm preflight (does not arm /goal)"
      check_tag
      check_branch
      check_worktree
      check_readiness_gate
      check_ledger
      check_taskfmt
      check_host_local_task_paths
      check_harness
      check_native_proof
      check_validate_plan
      echo ""
      echo "Preflight complete. No task dispatch or integration is performed."
      ;;
    help|-h|--help)
      usage
      ;;
    *)
      usage >&2
      fail "unknown mode: $mode"
      ;;
  esac
}

main "$@"
