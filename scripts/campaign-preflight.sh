#!/usr/bin/env bash
# Pre-arm readiness checks. Does NOT arm /goal or dispatch tasks.
set -euo pipefail

INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.worktrees/campaign}"
TAG_PEELED_EXPECT="${TAG_PEELED_EXPECT:-4a79c0a2d40fca46fc406b77157ce3b3f12ec16b}"

repo_root() {
  git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
}

warn() {
  echo "campaign-preflight: WARNING: $*" >&2
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
    warn "visual-baseline tag peeled=$peeled (expected $TAG_PEELED_EXPECT)"
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
  [[ -d "$WORKTREE_PATH" ]] || fail "missing worktree $WORKTREE_PATH"
  local wt_branch
  wt_branch="$(git -C "$WORKTREE_PATH" branch --show-current 2>/dev/null || echo detached)"
  if [[ "$wt_branch" != "$INTEGRATION_BRANCH" ]]; then
    warn "worktree on '$wt_branch' (expected $INTEGRATION_BRANCH)"
  else
    pass "worktree $WORKTREE_PATH on $INTEGRATION_BRANCH"
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
  pass "ledger.json valid (armed=false)"
}

check_taskfmt() {
  local bin="${TC_TASKFMT:-/tmp/taskfmt-install/bin/taskfmt}"
  if [[ ! -x "$bin" ]]; then
    warn "taskfmt not at $bin (run campaign-install-taskfmt.sh)"
    return
  fi
  pass "taskfmt @ $bin"
  "$bin" fingerprint 2>/dev/null || warn "taskfmt fingerprint failed"
}

check_harness() {
  local root wt
  root="$(repo_root)"
  wt="$root/$WORKTREE_PATH"
  if [[ -f "$wt/Cargo.toml" ]] && grep -q refactor-proof "$wt/Cargo.toml" 2>/dev/null; then
    if (cd "$wt" && cargo check -p refactor-proof -q 2>/dev/null); then
      pass "refactor-proof cargo check"
    else
      warn "refactor-proof cargo check failed in worktree"
    fi
  else
    warn "worktree missing refactor-proof (TASK-001 not integrated yet?)"
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
      warn "validate-plan not green or script failed"
    fi
  else
    warn "validate-plan.py not found (untracked docs?)"
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
  echo "Preflight complete. See docs/refactoring-plan/campaign-pre-arm-checklist.md for remaining items."
  if [[ ! -f .campaign/ledger.json ]] || ! grep -q task-001 .campaign/ledger.json 2>/dev/null; then
    echo "TASK-001 receipt not recorded — complete P2 before arming."
  fi
}

main "$@"
