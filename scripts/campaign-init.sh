#!/usr/bin/env bash
# Initialize single-branch campaign workspace (pre-arm). Does NOT arm /goal.
set -euo pipefail

ARCH_MAIN="${ARCH_MAIN:-7b27732a8c3c131760ec3438f641cb3c11343a42}"
INTEGRATION_BRANCH="${INTEGRATION_BRANCH:-refactor/holla-parity}"
CATALOG_BRANCH="${CATALOG_BRANCH:-prep-wave1-verify}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.worktrees/campaign}"
SEED_BRANCH="${SEED_BRANCH:-task-001-bootstrap}"

repo_root() {
  git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
}

die() {
  echo "campaign-init: $*" >&2
  exit 1
}

main() {
  local root
  root="$(repo_root)"
  cd "$root"

  echo "==> Fetching remotes"
  git fetch origin --tags 2>/dev/null || true

  local tag_peeled
  tag_peeled="$(git rev-parse "refs/tags/visual-baseline^{commit}" 2>/dev/null || echo UNKNOWN)"
  echo "==> visual-baseline tag (peeled): $tag_peeled"

  local seed_sha="$ARCH_MAIN"
  if git rev-parse "origin/$SEED_BRANCH" >/dev/null 2>&1; then
    seed_sha="$(git rev-parse "origin/$SEED_BRANCH")"
    echo "==> Seeding $INTEGRATION_BRANCH from origin/$SEED_BRANCH @ ${seed_sha:0:12}"
  else
    echo "==> Seeding $INTEGRATION_BRANCH from architectural main @ ${seed_sha:0:12}"
  fi

  if git show-ref --verify --quiet "refs/heads/$INTEGRATION_BRANCH"; then
    echo "==> Branch $INTEGRATION_BRANCH already exists @ $(git rev-parse "$INTEGRATION_BRANCH" | cut -c1-12)"
  else
    git branch "$INTEGRATION_BRANCH" "$seed_sha"
    echo "==> Created branch $INTEGRATION_BRANCH @ ${seed_sha:0:12}"
  fi

  if [[ -d "$WORKTREE_PATH" ]] || [[ -f "$WORKTREE_PATH/.git" ]]; then
    echo "==> Worktree exists: $WORKTREE_PATH"
    git -C "$WORKTREE_PATH" checkout "$INTEGRATION_BRANCH" 2>/dev/null || true
  else
    mkdir -p "$(dirname "$WORKTREE_PATH")"
    git worktree add -B "$INTEGRATION_BRANCH" "$WORKTREE_PATH" "$INTEGRATION_BRANCH"
    echo "==> Created worktree $WORKTREE_PATH"
  fi

  mkdir -p .campaign/runs .campaign/evidence
  if [[ ! -f .campaign/ledger.json ]]; then
    cp .campaign/ledger.template.json .campaign/ledger.json
  fi

  local catalog_sha catalog_time head_sha
  catalog_sha="$(git rev-parse "origin/$CATALOG_BRANCH" 2>/dev/null || git rev-parse "$CATALOG_BRANCH" 2>/dev/null || echo REPLACE_AT_INIT)"
  catalog_time="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  head_sha="$(git rev-parse "$INTEGRATION_BRANCH")"

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$root" "$catalog_sha" "$catalog_time" "$head_sha" "$CATALOG_BRANCH" <<'PY'
import json, sys
root, catalog_sha, catalog_time, head_sha, catalog_branch = sys.argv[1:6]
path = f"{root}/.campaign/ledger.json"
with open(path) as f:
    ledger = json.load(f)
ledger["integration_head"] = head_sha
ledger["catalog"] = {
    "commit": catalog_sha,
    "branch": catalog_branch,
    "recorded_at": catalog_time,
}
with open(path, "w") as f:
    json.dump(ledger, f, indent=2)
    f.write("\n")
print(f"Updated {path}")
PY
  else
    echo "==> WARNING: python3 missing; edit .campaign/ledger.json manually"
  fi

  cat <<EOF

Campaign workspace initialized (pre-arm).

  Integration branch:  $INTEGRATION_BRANCH @ $(git rev-parse "$INTEGRATION_BRANCH" | cut -c1-12)
  Worktree:            $WORKTREE_PATH
  Ledger:              .campaign/ledger.json
  Catalog branch:      $CATALOG_BRANCH @ ${catalog_sha:0:12}

Next steps (do NOT arm /goal yet):
  1. ./scripts/campaign-install-taskfmt.sh
  2. Complete docs/refactoring-plan/campaign-pre-arm-checklist.md (P2 TASK-001)
  3. ./scripts/campaign-preflight.sh

Push when ready:
  git push -u origin $INTEGRATION_BRANCH

EOF
}

main "$@"
