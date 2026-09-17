#!/usr/bin/env bash
# Pre-arm readiness checks. Does NOT arm /goal or dispatch tasks.
set -euo pipefail

INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.}"
TAG_PEELED_EXPECT="${TAG_PEELED_EXPECT:-4a79c0a2d40fca46fc406b77157ce3b3f12ec16b}"
TASKFMT_REV="${TASKFMT_REV:-afd3b575dbcc7044620bec4b9493a74eca3e5ef2}"

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
  local root
  root="$(repo_root)"
  [[ -d "$root/$WORKTREE_PATH" ]] || fail "missing worktree $root/$WORKTREE_PATH"
  local wt_branch
  wt_branch="$(git -C "$root/$WORKTREE_PATH" branch --show-current 2>/dev/null || echo detached)"
  if [[ "$wt_branch" != "$INTEGRATION_BRANCH" ]]; then
    fail "worktree on '$wt_branch' (expected $INTEGRATION_BRANCH)"
  else
    pass "worktree $root/$WORKTREE_PATH on $INTEGRATION_BRANCH"
  fi
}

check_ledger() {
  local root ledger
  root="$(repo_root)"
  ledger="$root/.campaign/ledger.json"
  [[ -f "$ledger" ]] || fail "missing $ledger (run campaign-init.sh)"
  python3 - "$ledger" <<'PY' || fail "ledger invalid"
import json, sys
p = sys.argv[1]
with open(p) as f:
    d = json.load(f)
assert d.get("schema") == "campaign-ledger/v1"
assert d.get("integration_ref") == "refs/heads/refactor/holla-parity"
assert d.get("armed") is False, "ledger shows armed=true — do not arm /goal via preflight"
assert d["catalog"]["commit"] != "REPLACE_AT_INIT", "catalog commit not recorded"
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
  [[ -x "$bin" ]] || fail "taskfmt not at $bin (run campaign-install-taskfmt.sh)"
  pass "taskfmt @ $bin"
  "$bin" --version | grep -Fq "git $TASKFMT_REV" \
    || fail "taskfmt is not latest $TASKFMT_REV"
  local catalog
  catalog="$(repo_root)/refactoring-tasks/terminal-components/completion"
  "$bin" lint "$catalog"/[0-9][0-9][0-9] >/dev/null \
    || fail "latest taskfmt lint failed"
  pass "latest taskfmt lint passed for all numbered packages"
}

check_harness() {
  local root wt
  root="$(repo_root)"
  wt="$root/$WORKTREE_PATH"
  if [[ -f "$wt/Cargo.toml" ]] && grep -q refactor-proof "$wt/Cargo.toml" 2>/dev/null; then
    if (cd "$wt" && cargo nextest list -p refactor-proof >/dev/null 2>/dev/null); then
      pass "refactor-proof nextest discovery"
    else
      fail "refactor-proof nextest discovery failed in worktree"
    fi
  else
    fail "worktree missing refactor-proof"
  fi
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
  check_harness
  check_validate_plan
  echo ""
  echo "Preflight complete. See docs/refactoring-plan/execution-readiness-report.md for remaining blockers."
  if [[ ! -f .campaign/ledger.json ]] || ! grep -q task-001 .campaign/ledger.json 2>/dev/null; then
    echo "TASK-001 receipt not recorded — host qualification remains blocked."
  fi
}

main "$@"
