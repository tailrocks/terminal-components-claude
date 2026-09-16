#!/usr/bin/env bash
# Hybrid TASK-071/072 container path simulation for advisory taskfmt verify.
# Extends task-001 bootstrap layout with /proof/bin/tc-proof and /run/tc-proof/contexts/CHK-001.json.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: hybrid-verify-sandbox.sh <command> [options]

Commands:
  prepare-hybrid  Create $TC_BIND layout for hybrid tasks (no sudo)
  mount           Verify container paths including /run/tc-proof (no sudo)
  layout-smoke    Assert hybrid container paths resolve expected files
  preflight-smoke Run CHK-001 preflight at /run/tc-proof/contexts/CHK-001.json
  verify          Run taskfmt verify from /work (requires mount + progress)
  docker-smoke    Linux Docker volume check for /run/tc-proof (paths only, no sudo)
  help            Show this message

prepare-hybrid options:
  --task NNN      Task number 071 or 072 (default: 071)

verify options:
  --progress PATH progress.md (required unless TC_RUN/progress.md exists)
  --log-dir PATH  Per-check logs (default: $TC_RUN/logs)
  --verbose       Pass --verbose to taskfmt verify

Environment:
  TC_PLANNING_REPO   Git repo root (auto-detected)
  TC_WORKTREE        Candidate worktree (default: $TC_PLANNING_REPO/.worktrees/campaign)
  TC_CATALOG_ROOT    Task catalog (default: $TC_PLANNING_REPO/refactoring-tasks/terminal-components)
  TC_BIND            Staging dir (default: /private/tmp/tc-task-001-bind)
  TC_TASKFMT         taskfmt binary (default: /tmp/taskfmt-install/bin/taskfmt)
  TC_TASKFMT_SOURCE  task-format checkout (default: /tmp/taskfmt-qualification)
  TC_BASE            Git base for verify (default: HEAD of worktree)
  TC_RUN             Run directory for logs and advisory tc-proof results
  TC_HYBRID_TASK     Default for --task (default: 071)

Notes:
  - Hybrid verify.toml CHK-001 invokes /proof/bin/tc-proof preflight against
    /run/tc-proof/contexts/CHK-001.json; remaining checks use bootstrap drivers.
  - This script never invokes sudo, osascript, or password prompts.
  - Root firmlinks require a one-time operator step (see mount error text).
  - /proof/bin/tc-proof is a sandbox wrapper that binds TC_PROOF_* from the frozen context.
  - Docker smoke confirms /run bind only; macOS sandbox-exec checks still need host mounts.
EOF
}

die() {
  echo "hybrid-verify-sandbox: $*" >&2
  exit 1
}

repo_root() {
  if [[ -n "${TC_PLANNING_REPO:-}" ]]; then
    echo "$TC_PLANNING_REPO"
    return
  fi
  local root
  root="$(git -C "${BASH_SOURCE[0]%/*}/.." rev-parse --show-toplevel 2>/dev/null)" || true
  [[ -n "$root" ]] || die "set TC_PLANNING_REPO or run from git checkout"
  echo "$root"
}

init_paths() {
  TC_PLANNING_REPO="$(repo_root)"
  TC_WORKTREE="${TC_WORKTREE:-$TC_PLANNING_REPO/.worktrees/campaign}"
  TC_CATALOG_ROOT="${TC_CATALOG_ROOT:-$TC_PLANNING_REPO/refactoring-tasks/terminal-components}"
  TC_BIND="${TC_BIND:-/private/tmp/tc-task-001-bind}"
  TC_TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-install/bin/taskfmt}"
  TC_TASKFMT_SOURCE="${TC_TASKFMT_SOURCE:-/tmp/taskfmt-qualification}"
  TC_HYBRID_TASK="${TC_HYBRID_TASK:-071}"
  if [[ -z "${TC_BASE:-}" ]]; then
    TC_BASE="$(git -C "$TC_WORKTREE" rev-parse HEAD 2>/dev/null || true)"
  fi
  TC_BASE="${TC_BASE:-UNKNOWN}"
}

synthetic_target() {
  local abs
  abs="$(cd "$(dirname "$1")" 2>/dev/null && pwd)/$(basename "$1")"
  abs="${abs#/}"
  echo "$abs"
}

synthetic_marker() {
  echo "# tc-task-001-bind $(synthetic_target "$TC_BIND")"
}

synthetic_fragment_path() {
  echo "$TC_BIND/synthetic.conf.fragment"
}

write_synthetic_fragment() {
  local frag task_rel work_rel proof_rel run_rel
  frag="$(synthetic_fragment_path)"
  task_rel="$(synthetic_target "$TC_BIND/task")"
  work_rel="$(synthetic_target "$TC_BIND/work")"
  proof_rel="$(synthetic_target "$TC_BIND/proof")"
  run_rel="$(synthetic_target "$TC_BIND/run")"
  {
    synthetic_marker
    printf 'task\t%s\n' "$task_rel"
    printf 'work\t%s\n' "$work_rel"
    printf 'proof\t%s\n' "$proof_rel"
    printf 'run\t%s\n' "$run_rel"
  } >"$frag"
  echo "Wrote synthetic fragment: $frag"
}

require_taskfmt() {
  [[ -x "$TC_TASKFMT" ]] || die "taskfmt not found at $TC_TASKFMT"
  [[ -d "$TC_TASKFMT_SOURCE" ]] || die "taskfmt source not found at $TC_TASKFMT_SOURCE"
  [[ -f "$TC_TASKFMT_SOURCE/experiment.toml" ]] || die "missing $TC_TASKFMT_SOURCE/experiment.toml"
}

require_worktree() {
  [[ -d "$TC_WORKTREE" ]] || die "worktree missing: $TC_WORKTREE"
}

require_tc_proof() {
  local proof_bin="$TC_WORKTREE/tools/refactor-proof/bin/tc-proof"
  [[ -f "$proof_bin" ]] || die "tc-proof missing: $proof_bin (run sync-binaries.sh in worktree if needed)"
}

normalize_hybrid_task() {
  local task="${1#TASK-}"
  task="${task##*/}"
  case "$task" in
    071|072) echo "$task" ;;
    *) die "unsupported hybrid task: $task (expected 071 or 072)" ;;
  esac
}

write_tc_proof_wrapper() {
  local wrapper real
  wrapper="$TC_BIND/proof/bin/tc-proof"
  real="$TC_BIND/proof/bin/tc-proof.real"
  mkdir -p "$TC_BIND/proof/bin"
  ln -sfn "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof" "$real"
  rm -f "$wrapper"
  cat >"$wrapper" <<EOF
#!/usr/bin/env bash
# Hybrid sandbox wrapper: bind frozen context env for taskfmt CHK-001 preflight.
set -euo pipefail
REAL="\${TC_PROOF_REAL:-$real}"
context=""
args=()
while [[ \$# -gt 0 ]]; do
  case "\$1" in
    --context)
      context="\$2"
      args+=("\$1" "\$2")
      shift 2
      ;;
    *)
      args+=("\$1")
      shift
      ;;
  esac
done
[[ -n "\$context" && -f "\$context" ]] || exec "\$REAL" "\${args[@]}"
eval "\$(python3 - "\$context" <<'PY'
import hashlib, json, os, sys
from pathlib import Path
path = Path(sys.argv[1])
raw = path.read_bytes()
ctx = json.loads(raw)
print(f"export TC_PROOF_CONTEXT_SHA256={hashlib.sha256(raw).hexdigest()!r}")
print(f"export TC_PROOF_RUN_ID={ctx['run_id']!r}")
print(f"export TC_PROOF_RESULT={os.environ.get('TC_PROOF_RESULT', f'/tmp/tc-proof-{os.getpid()}.json')!r}")
PY
)"
exec "\$REAL" "\${args[@]}"
EOF
  chmod +x "$wrapper"
}

materialize_chk001_context() {
  local dest="$TC_BIND/run/tc-proof/contexts/CHK-001.json"
  local proof_bin="$TC_BIND/proof/bin/tc-proof.real"
  local experiment="$TC_BIND/proof/bootstrap/experiment.toml"
  mkdir -p "$(dirname "$dest")"
  python3 - "$dest" "$proof_bin" "$experiment" <<'PY'
import hashlib
import json
import secrets
import sys
from pathlib import Path

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()

def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

dest, proof_bin, experiment = sys.argv[1:4]
tool = Path(proof_bin)
exp = Path(experiment)
tool_sha = sha256_bytes(tool.read_bytes())
bundle_sha = sha256_bytes(exp.read_bytes()) if exp.is_file() else "0" * 64
run_id = secrets.token_hex(24)
context = {
    "schema": "tc-proof-runner-context/v1",
    "run_id": run_id,
    "operation": "preflight",
    "tree": "0" * 40,
    "oracle_commit": "0" * 40,
    "oracle_tree": "0" * 40,
    "bundle": "/proof/bootstrap/experiment.toml",
    "bundle_sha256": bundle_sha,
    "tool": {"path": "/proof/bin/tc-proof.real", "sha256": tool_sha},
    "dependencies": [{"accepted": True, "integrated": True}],
    "adapter": {"changes": []},
    "lane": "direct",
    "axes": {"lanes": ["direct"], "widths": [8], "palettes": ["blue"]},
    "members": ["tiny/direct/8/blue"],
    "inventory": {
        "package": "tiny",
        "profile": "primary",
        "target": "unit",
        "source_commit": "0" * 40,
        "required": [],
        "closed": [],
        "future": {},
        "relocations": {},
    },
    "evidence": [],
    "configuration": {},
}
Path(dest).write_bytes(canonical(context))
print(dest)
PY
}

cmd_prepare_hybrid() {
  init_paths
  require_taskfmt
  require_worktree
  require_tc_proof

  local task_num="$TC_HYBRID_TASK"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --task) task_num="$2"; shift 2 ;;
      *) die "unknown prepare-hybrid option: $1" ;;
    esac
  done
  task_num="$(normalize_hybrid_task "$task_num")"

  local task_pkg bootstrap
  task_pkg="$TC_CATALOG_ROOT/completion/$task_num"
  bootstrap="$TC_BIND/proof/bootstrap"
  [[ -d "$task_pkg" ]] || die "task package missing: $task_pkg"

  mkdir -p "$bootstrap/bin"
  ln -sfn "$task_pkg" "$TC_BIND/task"
  ln -sfn "$TC_WORKTREE" "$TC_BIND/work"
  ln -sfn "$TC_TASKFMT" "$bootstrap/bin/taskfmt"
  ln -sfn "$TC_TASKFMT_SOURCE" "$bootstrap/task-format"
  cp -f "$TC_TASKFMT_SOURCE/experiment.toml" "$bootstrap/experiment.toml"
  write_tc_proof_wrapper

  local ctx_path
  ctx_path="$(materialize_chk001_context)"

  write_synthetic_fragment

  echo "Prepared hybrid bind layout at $TC_BIND (TASK-$task_num)"
  echo "  task              -> $task_pkg"
  echo "  work              -> $TC_WORKTREE"
  echo "  proof/bootstrap   -> taskfmt pins"
  echo "  proof/bin/tc-proof -> wrapper -> tc-proof.real"
  echo "  run/tc-proof/contexts/CHK-001.json -> $ctx_path"
}

path_mounted() {
  [[ -e "$1" ]]
}

operator_apfs_one_liner() {
  local frag run_rel
  frag="$(synthetic_fragment_path)"
  run_rel="$(synthetic_target "$TC_BIND/run")"
  cat <<EOF
# When task/work/proof firmlinks already exist, add /run only:
sudo sh -c 'grep -q "^run[[:space:]]" /etc/synthetic.conf || printf "run\\t${run_rel}\\n" >> /etc/synthetic.conf; /System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util -t'
# Fresh host (no tc-task-001-bind entries yet) — merge full fragment:
sudo sh -c 'grep -q tc-task-001-bind /etc/synthetic.conf 2>/dev/null || cat "$frag" >> /etc/synthetic.conf; /System/Library/Filesystems/apfs.fs/Contents/Resources/apfs.util -t'
EOF
}

cmd_mount() {
  init_paths
  [[ -d "$TC_BIND/task" ]] || die "run 'prepare-hybrid' first (missing $TC_BIND/task)"

  if path_mounted /task \
    && path_mounted /work \
    && path_mounted /proof/bootstrap \
    && path_mounted /proof/bin/tc-proof \
    && path_mounted /run/tc-proof/contexts/CHK-001.json; then
    echo "Hybrid container paths available at /task, /work, /proof/{bootstrap,bin}, /run/tc-proof"
    return 0
  fi

  cat >&2 <<EOF
hybrid-verify-sandbox: container paths missing (need /run/tc-proof firmlink).

Autonomous agents do not use sudo. Bind layout is at:
  $TC_BIND

One-time host provisioning (operator, outside agent sessions):
  1. ./scripts/hybrid-verify-sandbox.sh prepare-hybrid --task 071   # or 072
  2. Operator one-liner (merge fragment + refresh firmlinks):
$(operator_apfs_one_liner | sed 's/^/     /')
  3. ./scripts/hybrid-verify-sandbox.sh mount   # verify-only

See docs/refactoring-plan/task-production-verify-container.md and path-contract.md (hybrid 071/072)
EOF
  die "container paths not available"
}

cmd_layout_smoke() {
  init_paths
  local ok=1
  check() {
    if [[ -e "$1" ]]; then
      echo "OK  $1"
    else
      echo "MISS $1" >&2
      ok=0
    fi
  }

  check /task/verify.toml
  check /work/tools/refactor-proof/bin/tc-proof
  check /work/tools/refactor-proof/bin/tc-proof-host
  check /proof/bootstrap/bin/taskfmt
  check /proof/bootstrap/experiment.toml
  check /proof/bin/tc-proof
  check /run/tc-proof/contexts/CHK-001.json

  if [[ "$ok" -eq 1 ]]; then
    echo "LAYOUT_OK"
    return 0
  fi
  die "layout smoke failed — run 'prepare-hybrid' and operator firmlink refresh"
}

cmd_preflight_smoke() {
  init_paths
  path_mounted /proof/bin/tc-proof \
    || die "run 'mount' first (/proof/bin/tc-proof not present)"
  path_mounted /run/tc-proof/contexts/CHK-001.json \
    || die "run firmlink missing: /run/tc-proof/contexts/CHK-001.json (see mount error for operator one-liner)"

  local result
  result="${TC_RUN:-/tmp}/tc-preflight-smoke-$$.json"
  mkdir -p "$(dirname "$result")"
  export TC_PROOF_RESULT="$result"
  echo "Running CHK-001 preflight at /run/tc-proof/contexts/CHK-001.json"
  /proof/bin/tc-proof preflight --context /run/tc-proof/contexts/CHK-001.json
  echo "PREFLIGHT_OK exit=0 result=$result"
}

cmd_verify() {
  init_paths
  require_taskfmt

  local progress="" log_dir="" verbose=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --progress) progress="$2"; shift 2 ;;
      --log-dir) log_dir="$2"; shift 2 ;;
      --verbose) verbose=1; shift ;;
      *) die "unknown verify option: $1" ;;
    esac
  done

  path_mounted /task || die "run 'mount' first (/task not present)"
  path_mounted /work || die "run 'mount' first (/work not present)"
  path_mounted /proof/bootstrap || die "run 'mount' first (/proof/bootstrap not present)"
  path_mounted /run/tc-proof/contexts/CHK-001.json \
    || die "run 'mount' first (/run/tc-proof/contexts/CHK-001.json not present)"

  if [[ -z "$progress" ]]; then
    if [[ -n "${TC_RUN:-}" && -f "$TC_RUN/progress.md" ]]; then
      progress="$TC_RUN/progress.md"
    else
      die "pass --progress PATH or set TC_RUN with progress.md"
    fi
  fi
  [[ -f "$progress" ]] || die "progress file not found: $progress"

  if [[ -z "$log_dir" ]]; then
    TC_RUN="${TC_RUN:-$(dirname "$progress")}"
    log_dir="$TC_RUN/logs"
  fi
  mkdir -p "$log_dir"

  if git -C "$TC_WORKTREE" status --porcelain | grep -q '^??'; then
    echo "WARNING: worktree has untracked files; taskfmt scope may FAIL outside writable_paths." >&2
  fi

  local verbose_flag=()
  [[ "$verbose" -eq 1 ]] && verbose_flag=(--verbose)

  echo "Running hybrid taskfmt verify from /work (base=$TC_BASE)"
  (
    cd /work
    "$TC_TASKFMT" --config /proof/bootstrap/experiment.toml verify \
      --base "$TC_BASE" \
      --progress "$progress" \
      --log-dir "$log_dir" \
      "${verbose_flag[@]}"
  )
}

cmd_docker_smoke() {
  init_paths
  [[ -d "$TC_BIND/run/tc-proof" ]] || cmd_prepare_hybrid

  command -v docker >/dev/null 2>&1 || die "docker not available"

  echo "Docker hybrid layout smoke (paths only — no macOS sandbox-exec)"
  docker run --rm \
    -v "$TC_BIND/task:/task:ro" \
    -v "$TC_WORKTREE:/work:ro" \
    -v "$TC_BIND/proof:/proof:ro" \
    -v "$TC_BIND/run/tc-proof:/run/tc-proof:ro" \
    alpine:3.20 sh -c '
      set -e
      test -f /task/verify.toml
      test -f /work/tools/refactor-proof/bin/tc-proof-host
      test -f /proof/bootstrap/experiment.toml
      test -x /proof/bin/tc-proof
      test -f /run/tc-proof/contexts/CHK-001.json
      echo LAYOUT_OK
    '
}

main() {
  local cmd="${1:-help}"
  shift || true
  case "$cmd" in
    prepare-hybrid) cmd_prepare_hybrid "$@" ;;
    mount) cmd_mount "$@" ;;
    layout-smoke) cmd_layout_smoke "$@" ;;
    preflight-smoke) cmd_preflight_smoke "$@" ;;
    verify) cmd_verify "$@" ;;
    docker-smoke) cmd_docker_smoke "$@" ;;
    help|-h|--help) usage ;;
    *) die "unknown command: $cmd (try help)" ;;
  esac
}

main "$@"
