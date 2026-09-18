#!/usr/bin/env bash
# Pre-arm readiness checks. Does NOT arm /goal or dispatch tasks.
set -euo pipefail

INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.}"
TAG_PEELED_EXPECT="${TAG_PEELED_EXPECT:-4a79c0a2d40fca46fc406b77157ce3b3f12ec16b}"
TASKFMT_REV="afd3b575dbcc7044620bec4b9493a74eca3e5ef2"
TASKFMT_SHA256="f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de"
TASKFMT_SOURCE="/Users/donbeave/Projects/taskfmt/task-format"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

repo_root() {
  git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
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
  local wt
  wt="$(campaign_worktree)"
  [[ -d "$wt" ]] || fail "missing worktree $wt"
  local wt_branch
  wt_branch="$(git -C "$wt" branch --show-current 2>/dev/null || echo detached)"
  if [[ "$wt_branch" != "$INTEGRATION_BRANCH" ]]; then
    fail "worktree on '$wt_branch' (expected $INTEGRATION_BRANCH)"
  else
    pass "worktree $wt on $INTEGRATION_BRANCH"
  fi
}

check_ledger() {
  local root ledger
  root="$(repo_root)"
  ledger="$root/.campaign/ledger.json"
  [[ -f "$ledger" ]] || fail "missing $ledger (run campaign-init.sh)"
  PYTHONPATH="$SCRIPT_DIR" python3 - "$ledger" "$INTEGRATION_BRANCH" <<'PY' || fail "ledger invalid"
import json, sys
from campaign_ledger import validate_preflight_ledger

with open(sys.argv[1], encoding="utf-8") as f:
    ledger = json.load(f)
validate_preflight_ledger(ledger, sys.argv[2])
print("ledger schema OK; armed=false; catalog recorded")
PY
  local head
  head="$(git rev-parse HEAD)"
  python3 - "$ledger" "$head" <<'PY' || fail "ledger integration_head is not an ancestor of HEAD"
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
import subprocess
result = subprocess.run(["git", "merge-base", "--is-ancestor", d["integration_head"], sys.argv[2]])
assert result.returncode == 0, "integration_head is not an ancestor of HEAD"
PY
  pass "ledger.json valid (armed=false; integration head is an ancestor)"
}

check_taskfmt() {
  local bin="${TC_TASKFMT:-/tmp/taskfmt-latest-install/bin/taskfmt}"
  [[ -d "$TASKFMT_SOURCE/.git" ]] || fail "taskfmt source is not a git checkout: $TASKFMT_SOURCE"
  [[ -z "$(git -C "$TASKFMT_SOURCE" status --porcelain)" ]] \
    || fail "taskfmt source is dirty: $TASKFMT_SOURCE"
  [[ "$(git -C "$TASKFMT_SOURCE" rev-parse HEAD)" == "$TASKFMT_REV" ]] \
    || fail "taskfmt source is not latest $TASKFMT_REV"
  [[ -x "$bin" ]] || fail "taskfmt not at $bin (run campaign-install-taskfmt.sh)"
  pass "taskfmt @ $bin"
  local actual_sha
  actual_sha="$(shasum -a 256 "$bin" | awk '{print $1}')"
  [[ "$actual_sha" == "$TASKFMT_SHA256" ]] \
    || fail "taskfmt SHA-256 is $actual_sha; expected $TASKFMT_SHA256"
  "$bin" --version | grep -Fq "git $TASKFMT_REV" \
    || fail "taskfmt is not latest $TASKFMT_REV"
  local catalog
  catalog="$(repo_root)/refactoring-tasks/terminal-components/completion"
  local task
  for task in "$catalog"/[0-9][0-9][0-9]; do
    "$bin" lint "$task" >/dev/null \
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
  local wt binary receipt commit
  wt="$(campaign_worktree)"
  binary="$wt/target/debug/tc-proof"
  [[ -f "$binary" && ! -L "$binary" && -x "$binary" ]] \
    || fail "native tc-proof comparator missing: $binary (run scripts/campaign-build-proof.sh in this worktree)"
  receipt="$binary.build.json"
  [[ -f "$receipt" && ! -L "$receipt" ]] \
    || fail "native tc-proof build receipt missing: $receipt (run scripts/campaign-build-proof.sh)"
  commit="$(git -C "$wt" rev-parse HEAD)"
  python3 - "$receipt" "$wt" "$commit" "$binary" <<'PY' || fail "native tc-proof build receipt does not match this worktree"
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
  pass "native tc-proof comparator and build receipt match worktree HEAD"
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

main() {
  local root
  root="$(repo_root)"
  cd "$root"
  echo "==> Pre-arm preflight (does not arm /goal)"
  check_tag
  check_branch
  check_worktree
  check_ledger
  check_taskfmt
  check_host_local_task_paths
  check_harness
  check_native_proof
  check_validate_plan
  echo ""
  echo "Preflight complete. See docs/refactoring-plan/execution-readiness-report.md for remaining blockers."
}

main "$@"
