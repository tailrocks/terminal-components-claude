#!/usr/bin/env bash
# Host dispatch wrapper for campaign tasks (pre-arm: status/help only until receipt exists).
set -euo pipefail

INTEGRATION_REF="${INTEGRATION_REF:-refs/heads/refactor/holla-parity}"
CATALOG_ROOT="${TC_CATALOG_ROOT:-refactoring-tasks/terminal-components}"

usage() {
  cat <<'EOF'
Usage: campaign-dispatch.sh <command> [args]

Commands:
  status          Show ledger integration head and receipt summary
  prepare         tc-proof-host prepare (requires TASK-001 receipt for TASK-002+)
  freeze          tc-proof-host freeze --run RUN --candidate CANDIDATE
  verify          tc-proof-host verify --run RUN
  integrate       tc-proof-host integrate --run RUN --expected-parent PARENT
  help            This message

Environment:
  TC_CAMPAIGN_WORKTREE   Default .worktrees/campaign
  TC_CAMPAIGN_DIR        Host campaign root (default .campaign/host)
  TC_CATALOG_ROOT        Task catalog path
  RUN                    Run directory for freeze/verify/integrate
  PARENT                 Expected parent SHA for integrate
  TASK                   Task path e.g. terminal-components/completion/001

Pre-arm: run `status` only until campaign-pre-arm-checklist P2 is complete.
EOF
}

repo_root() {
  git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel
}

die() {
  echo "campaign-dispatch: $*" >&2
  exit 1
}

host_bin() {
  local root wt synced debug
  root="$(repo_root)"
  wt="${TC_CAMPAIGN_WORKTREE:-$root/.worktrees/campaign}"
  synced="$wt/tools/refactor-proof/bin/tc-proof-host"
  debug="$wt/target/debug/tc-proof-host"
  # verify.toml gate path: /work/tools/refactor-proof/bin/tc-proof-host (synced Mach-O).
  if [[ -x "$synced" ]] && file "$synced" 2>/dev/null | grep -q 'Mach-O'; then
    echo "$synced"
  elif [[ -x "$debug" ]]; then
    echo "$debug"
  elif [[ -x "$synced" ]]; then
    echo "$synced"
  else
    die "tc-proof-host not found (cargo build -p refactor-proof && sync-binaries.sh in worktree)"
  fi
}

ledger_head() {
  python3 - "$(repo_root)/.campaign/ledger.json" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    print(json.load(f)["integration_head"])
PY
}

cmd_status() {
  local root="$1"
  local ledger="$root/.campaign/ledger.json"
  [[ -f "$ledger" ]] || die "missing ledger — run campaign-init.sh"
  python3 - "$ledger" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    L = json.load(f)
print("integration_ref:", L["integration_ref"])
print("integration_head:", L["integration_head"])
print("armed:", L.get("armed", False))
print("catalog:", L["catalog"]["commit"][:12], "from", L["catalog"].get("branch", "?"))
print("receipts:", L.get("receipts") or {})
tasks = L.get("tasks") or []
if tasks:
    print("tasks recorded:", len(tasks))
else:
    print("tasks recorded: 0")
PY
}

require_not_armed_for_mutating() {
  local root="$1"
  python3 - "$root/.campaign/ledger.json" <<'PY' || die "cannot read ledger"
import json, sys
with open(sys.argv[1]) as f:
    armed = json.load(f).get("armed", False)
if armed:
    sys.exit(0)
print("pre-arm mode: host dispatch allowed for TASK-001 bootstrap only")
PY
}

main() {
  local root
  root="$(repo_root)"
  local cmd="${1:-help}"
  shift || true

  case "$cmd" in
    help|-h|--help)
      usage
      ;;
    status)
      cmd_status "$root"
      ;;
    prepare|freeze|verify|integrate)
      local host
      host="$(host_bin)"
      local run="${RUN:-}"
      [[ -n "$run" ]] || die "set RUN= for $cmd"
      case "$cmd" in
        prepare)
          local task="${TASK:-}"
          local parent="${PARENT:-$(ledger_head)}"
          [[ -n "$task" ]] || die "set TASK=terminal-components/completion/NNN"
          "$host" prepare --campaign "${TC_CAMPAIGN_DIR:-$root/.campaign/host}" \
            --task "$task" --parent "$parent" --run "$run"
          ;;
        freeze)
          local candidate="${CANDIDATE:-${TC_CAMPAIGN_WORKTREE:-$root/.worktrees/campaign}}"
          "$host" freeze --run "$run" --candidate "$candidate"
          ;;
        verify)
          "$host" verify --run "$run"
          ;;
        integrate)
          local parent="${PARENT:-$(ledger_head)}"
          "$host" integrate --run "$run" --ref "$INTEGRATION_REF" --expected-parent "$parent"
          ;;
      esac
      ;;
    *)
      die "unknown command: $cmd"
      ;;
  esac
}

main "$@"
