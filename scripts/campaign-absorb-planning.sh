#!/usr/bin/env bash
# Copy planning catalog and prep docs onto refactor/holla-parity (no full branch merge).
set -euo pipefail

CATALOG_BRANCH="${CATALOG_BRANCH:-prep-wave1-verify}"
WORKTREE_PATH="${TC_CAMPAIGN_WORKTREE:-.worktrees/campaign}"

repo_root() {
  git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
}

die() {
  echo "campaign-absorb-planning: $*" >&2
  exit 1
}

main() {
  local root wt
  root="$(repo_root)"
  wt="$root/$WORKTREE_PATH"
  [[ -d "$wt/.git" || -f "$wt/.git" ]] || die "run campaign-init.sh first"

  cd "$wt"
  git fetch origin "$CATALOG_BRANCH" 2>/dev/null || true

  echo "==> Checking out planning paths from origin/$CATALOG_BRANCH"
  git checkout "origin/$CATALOG_BRANCH" -- \
    docs/refactoring-plan \
    docs/refactoring \
    refactoring-tasks \
    AGENTS.md \
    CLAUDE.md \
    2>/dev/null || die "could not checkout planning paths from $CATALOG_BRANCH"

  # Prep scripts from repo root (may be untracked on main worktree)
  mkdir -p "$wt/scripts"
  for f in campaign-init.sh campaign-preflight.sh campaign-dispatch.sh \
    campaign-install-taskfmt.sh campaign-absorb-planning.sh task-001-verify-sandbox.sh; do
    if [[ -f "$root/scripts/$f" ]]; then
      cp "$root/scripts/$f" "$wt/scripts/$f"
      chmod +x "$wt/scripts/$f"
    fi
  done

  mkdir -p "$wt/.cursor/rules" "$wt/.campaign"
  if [[ -f "$root/.cursor/rules/campaign-execution.mdc" ]]; then
    cp "$root/.cursor/rules/campaign-execution.mdc" "$wt/.cursor/rules/"
  fi
  if [[ -f "$root/.campaign/ledger.template.json" ]]; then
    cp "$root/.campaign/ledger.template.json" "$wt/.campaign/"
    cp "$root/.campaign/README.md" "$wt/.campaign/" 2>/dev/null || true
  fi

  # Ensure CLAUDE.md -> AGENTS.md
  if [[ -f AGENTS.md ]] && [[ ! -L CLAUDE.md ]]; then
    ln -sf AGENTS.md CLAUDE.md
  fi

  echo "==> Staging planning absorb (review before commit)"
  git add docs/refactoring-plan docs/refactoring refactoring-tasks \
    AGENTS.md CLAUDE.md scripts .cursor/rules .campaign/ledger.template.json .campaign/README.md 2>/dev/null || true

  cat <<EOF

Planning artifacts absorbed onto $(git branch --show-current).

Review and commit on refactor/holla-parity:
  cd $wt
  git status
  git commit -m "prep: planning catalog and campaign pre-arm tooling"

Do NOT arm /goal until campaign-pre-arm-checklist.md P2 complete.

EOF
}

main "$@"
