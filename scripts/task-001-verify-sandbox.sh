#!/usr/bin/env bash
# TASK-001 container path simulation for taskfmt verify (prep-wave1-verify).
# Provisions /task, /work, /proof/bootstrap per 001/verify.toml.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: task-001-verify-sandbox.sh <command> [options]

Commands:
  prepare       Create $TC_BIND symlink tree (no sudo)
  mount         Expose /task, /work, /proof/bootstrap at filesystem root (sudo)
  unmount       Remove root symlinks or nullfs mounts (sudo)
  layout-smoke  Verify container paths resolve expected files
  verify        Run taskfmt verify from /work (requires mount + progress)
  docker-smoke  Optional Linux Docker volume layout check (paths only)
  help          Show this message

Environment:
  TC_PLANNING_REPO   Git repo root (auto-detected)
  TC_WORKTREE        Candidate worktree (default: $TC_PLANNING_REPO/.worktrees/main)
  TC_CATALOG_ROOT    Task catalog (default: $TC_PLANNING_REPO/refactoring-tasks/terminal-components)
  TC_BIND            Staging dir (default: /private/tmp/tc-task-001-bind)
  TC_TASKFMT         taskfmt binary (default: /tmp/taskfmt-install/bin/taskfmt)
  TC_TASKFMT_SOURCE  task-format checkout (default: /tmp/taskfmt-qualification)
  TC_MOUNT_MODE      symlink (default) or nullfs
  TC_BASE            Git base for verify (default: HEAD of worktree)
  TC_RUN             Run directory for logs (verify creates if unset)

verify options:
  --progress PATH    progress.md (required unless TC_RUN/progress.md exists)
  --log-dir PATH     Per-check logs (default: $TC_RUN/logs)
  --verbose          Pass --verbose to taskfmt verify

Notes:
  - SO-005 requires macOS with sandbox-exec; Linux Docker cannot pass CHK-005/006/007.
  - mount/unmount require interactive sudo on macOS.
  - Do not merge prep-wave1-verify to main; never move visual-baseline tag.
EOF
}

die() {
  echo "task-001-verify-sandbox: $*" >&2
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
  TC_WORKTREE="${TC_WORKTREE:-$TC_PLANNING_REPO/.worktrees/main}"
  TC_CATALOG_ROOT="${TC_CATALOG_ROOT:-$TC_PLANNING_REPO/refactoring-tasks/terminal-components}"
  TC_BIND="${TC_BIND:-/private/tmp/tc-task-001-bind}"
  TC_TASKFMT="${TC_TASKFMT:-/tmp/taskfmt-install/bin/taskfmt}"
  TC_TASKFMT_SOURCE="${TC_TASKFMT_SOURCE:-/tmp/taskfmt-qualification}"
  TC_MOUNT_MODE="${TC_MOUNT_MODE:-symlink}"
  TC_BASE="${TC_BASE:-$(git -C "$TC_WORKTREE" rev-parse HEAD)}"
}

require_taskfmt() {
  [[ -x "$TC_TASKFMT" ]] || die "taskfmt not found at $TC_TASKFMT (see task-001-taskfmt-verify-notes.md §1)"
  [[ -d "$TC_TASKFMT_SOURCE" ]] || die "taskfmt source not found at $TC_TASKFMT_SOURCE"
  [[ -f "$TC_TASKFMT_SOURCE/experiment.toml" ]] || die "missing $TC_TASKFMT_SOURCE/experiment.toml"
}

require_worktree() {
  [[ -d "$TC_WORKTREE" ]] || die "worktree missing: $TC_WORKTREE"
  [[ -f "$TC_CATALOG_ROOT/completion/001/verify.toml" ]] || die "TASK-001 package missing under $TC_CATALOG_ROOT"
}

cmd_prepare() {
  init_paths
  require_taskfmt
  require_worktree

  local task_pkg bootstrap
  task_pkg="$TC_CATALOG_ROOT/completion/001"
  bootstrap="$TC_BIND/proof/bootstrap"

  mkdir -p "$bootstrap/bin"
  ln -sfn "$task_pkg" "$TC_BIND/task"
  ln -sfn "$TC_WORKTREE" "$TC_BIND/work"
  ln -sfn "$TC_TASKFMT" "$bootstrap/bin/taskfmt"
  ln -sfn "$TC_TASKFMT_SOURCE" "$bootstrap/task-format"
  cp -f "$TC_TASKFMT_SOURCE/experiment.toml" "$bootstrap/experiment.toml"

  echo "Prepared bind layout at $TC_BIND"
  echo "  task   -> $task_pkg"
  echo "  work   -> $TC_WORKTREE"
  echo "  proof  -> $bootstrap"
}

path_mounted() {
  local p="$1"
  [[ -e "$p" ]] || return 1
  if [[ "$TC_MOUNT_MODE" == "nullfs" ]]; then
    mount | grep -q " on $p "
  else
    [[ -L "$p" || -d "$p" ]]
  fi
}

cmd_mount() {
  init_paths
  [[ -d "$TC_BIND/task" ]] || die "run 'prepare' first (missing $TC_BIND/task)"

  if path_mounted /task && path_mounted /work && path_mounted /proof/bootstrap; then
    echo "Container paths already mounted"
    return 0
  fi

  if [[ "$TC_MOUNT_MODE" == "nullfs" ]]; then
    echo "Mounting nullfs (sudo required)..."
    sudo mkdir -p /task /work /proof/bootstrap
    sudo mount -t nullfs "$TC_BIND/task" /task
    sudo mount -t nullfs "$TC_BIND/work" /work
    sudo mount -t nullfs "$TC_BIND/proof/bootstrap" /proof/bootstrap
  else
    echo "Creating root symlinks (sudo required)..."
    sudo mkdir -p /proof
    sudo ln -sfn "$TC_BIND/task" /task
    sudo ln -sfn "$TC_BIND/work" /work
    sudo ln -sfn "$TC_BIND/proof/bootstrap" /proof/bootstrap
  fi
  echo "Mounted /task, /work, /proof/bootstrap"
}

cmd_unmount() {
  init_paths
  if [[ "$TC_MOUNT_MODE" == "nullfs" ]]; then
    echo "Unmounting nullfs (sudo required)..."
    sudo umount /task 2>/dev/null || true
    sudo umount /work 2>/dev/null || true
    sudo umount /proof/bootstrap 2>/dev/null || true
  else
    echo "Removing root symlinks (sudo required)..."
    sudo rm -f /task /work
    sudo rm -f /proof/bootstrap
    sudo rmdir /proof 2>/dev/null || true
  fi
  echo "Unmounted container paths"
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

  check /task/trusted/proof-bootstrap/host-bootstrap-driver.py
  check /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py
  check /work/tools/refactor-proof/bin/tc-proof
  check /work/tools/refactor-proof/bin/tc-proof-host
  check /proof/bootstrap/bin/taskfmt
  check /proof/bootstrap/task-format/experiment.toml
  check /proof/bootstrap/experiment.toml

  if [[ "$ok" -eq 1 ]]; then
    echo "LAYOUT_OK"
    return 0
  fi
  die "layout smoke failed — run 'prepare' and 'mount' first"
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

  local verbose_flag=()
  [[ "$verbose" -eq 1 ]] && verbose_flag=(--verbose)

  echo "Running taskfmt verify from /work (base=$TC_BASE)"
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
  [[ -d "$TC_BIND/task" ]] || cmd_prepare

  command -v docker >/dev/null 2>&1 || die "docker not available"

  echo "Docker layout smoke (Linux — does not satisfy SO-005 host matrix)"
  docker run --rm \
    -v "$TC_BIND/task:/task:ro" \
    -v "$TC_WORKTREE:/work:ro" \
    -v "$TC_BIND/proof/bootstrap:/proof/bootstrap:ro" \
    alpine:3.20 sh -c '
      set -e
      test -f /task/trusted/proof-bootstrap/host-bootstrap-driver.py
      test -f /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py
      test -f /work/tools/refactor-proof/bin/tc-proof
      test -f /work/tools/refactor-proof/bin/tc-proof-host
      test -f /proof/bootstrap/experiment.toml
      echo LAYOUT_OK
    '
}

main() {
  local cmd="${1:-help}"
  shift || true
  case "$cmd" in
    prepare) cmd_prepare "$@" ;;
    mount) cmd_mount "$@" ;;
    unmount) cmd_unmount "$@" ;;
    layout-smoke) cmd_layout_smoke "$@" ;;
    verify) cmd_verify "$@" ;;
    docker-smoke) cmd_docker_smoke "$@" ;;
    help|-h|--help) usage ;;
    *) die "unknown command: $cmd (try help)" ;;
  esac
}

main "$@"
