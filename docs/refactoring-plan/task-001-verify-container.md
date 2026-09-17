# Historical TASK-001 container path simulation for `taskfmt verify`

> **Superseded.** Root-firmlink simulation was retired. Retained for
> provenance only; do not run its commands.

**Canonical reference:** [`path-contract.md`](path-contract.md) (all task bands). Production tasks (002+): [`task-production-verify-container.md`](task-production-verify-container.md).

**Date:** 2026-09-15  
**Branch:** `prep-wave1-verify` (planning only — do not merge to `main`)  
**Authority:** [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml)

This document describes how to provision the `/task`, `/work`, and `/proof/bootstrap` paths required by TASK-001 `verify.toml` so standalone `taskfmt verify` can resolve CHK-001/004/005/006/007 argv. It complements [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) (blockers) and [`task-001-ci-requirements.md`](task-001-ci-requirements.md) (full CI layout).

---

## Why container paths are required

`verify.toml` hardcodes subprocess argv at container mount points:

| Path | Role |
| --- | --- |
| `/task/` | TASK-001 package (`trusted/proof-bootstrap/` drivers) |
| `/work/` | Candidate worktree (`writable_paths = ["tools/refactor-proof"]`) |
| `/proof/bootstrap/` | Pinned taskfmt, task-format source, `experiment.toml` |

Passing `--task-dir` or `--root` to `taskfmt verify` does **not** rewrite check argv. Without root-level `/task`, `/work`, and `/proof/bootstrap`, CHK checks fail immediately (`python3` rc 2, file not found). See [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) §5.

**Additional gate:** SO-005 requires `taskfmt verify` exit **0** and last stdout line **`DONE`**. That needs completed checklist progress through leaf **3.1** in addition to reachable CHK commands ([`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6).

---

## Quick start (macOS operator host)

### 1. Prerequisites

| Prerequisite | Check |
| --- | --- |
| Pinned taskfmt @ `52d9f1eb…` | `/tmp/taskfmt-install/bin/taskfmt --version` |
| Worktree built + synced | `cargo build -p refactor-proof && tools/refactor-proof/scripts/sync-binaries.sh` in worktree |
| Darwin `sandbox-exec` | Required for CHK-005/006/007 ([`task-001-ci-requirements.md`](task-001-ci-requirements.md)) |

Set session paths (adjust worktree if needed):

```sh
export TC_PLANNING_REPO=/path/to/terminal-components-claude
export TC_WORKTREE=$TC_PLANNING_REPO/.worktrees/campaign   # branch refactor/holla-parity
export TC_CATALOG_ROOT=$TC_PLANNING_REPO/refactoring-tasks/terminal-components
export TC_TASKFMT=/tmp/taskfmt-install/bin/taskfmt
export TC_TASKFMT_SOURCE=/tmp/taskfmt-qualification
```

Install pinned taskfmt if missing — see [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) §1 or [`task-001-ci-requirements.md`](task-001-ci-requirements.md) § pinned taskfmt install.

### 2. Prepare bind layout + root mounts

```sh
cd "$TC_PLANNING_REPO"
scripts/task-001-verify-sandbox.sh prepare
scripts/task-001-verify-sandbox.sh mount    # requires interactive sudo once
scripts/task-001-verify-sandbox.sh layout-smoke
```

`prepare` creates `$TC_BIND` (default `/private/tmp/tc-task-001-bind`) with symlinks:

```text
$TC_BIND/task              → $TC_CATALOG_ROOT/completion/001
$TC_BIND/work              → $TC_WORKTREE
$TC_BIND/proof/bootstrap/  → taskfmt bin, task-format checkout, experiment.toml
```

`mount` creates `/task`, `/work`, `/proof/bootstrap` at filesystem root (symlinks or nullfs — see script `--help`).

### 3. Initialize progress and run verify

```sh
export TC_RUN=/tmp/tc-task-001-run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_RUN/logs"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_RUN/progress.md"

# Complete checklist events through leaf 3.1 per campaign executor protocol, then:
export TC_BASE=$(git -C "$TC_WORKTREE" rev-parse HEAD)

cd /work
"$TC_TASKFMT" --config /proof/bootstrap/experiment.toml verify \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs" \
  --verbose 2>&1 | tee "$TC_RUN/taskfmt-verify.log"

tail -1 "$TC_RUN/taskfmt-verify.log"   # must be DONE for SO-005
```

Or use the wrapper (after `mount`):

```sh
export TC_RUN=/tmp/tc-task-001-run-$(date -u +%Y%m%dT%H%M%SZ)
scripts/task-001-verify-sandbox.sh verify --progress "$TC_RUN/progress.md" --log-dir "$TC_RUN/logs"
```

### 4. Cleanup

```sh
scripts/task-001-verify-sandbox.sh unmount
```

---

## Script reference

[`scripts/task-001-verify-sandbox.sh`](../../scripts/task-001-verify-sandbox.sh)

| Command | Purpose |
| --- | --- |
| `prepare` | Create `$TC_BIND` symlink tree (no sudo) |
| `mount` | Expose `/task`, `/work`, `/proof/bootstrap` at root (sudo) |
| `unmount` | Remove root symlinks / nullfs mounts |
| `layout-smoke` | Assert expected files exist at container paths |
| `verify` | Run `taskfmt verify` from `/work` using container paths |
| `docker-smoke` | Optional Linux Docker layout check (paths only) |

Environment overrides: `TC_BIND`, `TC_PLANNING_REPO`, `TC_WORKTREE`, `TC_CATALOG_ROOT`, `TC_TASKFMT`, `TC_TASKFMT_SOURCE`, `TC_RUN`.

---

## Optional Docker layout smoke (not SO-005)

Linux Docker **cannot** satisfy SO-005: CHK host-matrix drivers require macOS `sandbox-exec` ([`task-001-ci-requirements.md`](task-001-ci-requirements.md) § Darwin sandbox-exec requirement).

When Docker is available, `docker-smoke` verifies volume mounts resolve the same paths inside a container:

```sh
scripts/task-001-verify-sandbox.sh prepare
scripts/task-001-verify-sandbox.sh docker-smoke
```

This confirms `/task/trusted/proof-bootstrap/*.py` and synced `/work/tools/refactor-proof/bin/*` are visible. It does **not** run full `taskfmt verify` or host qualification.

Manual equivalent:

```sh
docker run --rm \
  -v "$TC_BIND/task:/task:ro" \
  -v "$TC_WORKTREE:/work:ro" \
  -v "$TC_BIND/proof/bootstrap:/proof/bootstrap:ro" \
  alpine:3.20 sh -c '
    test -f /task/trusted/proof-bootstrap/host-bootstrap-driver.py &&
    test -f /work/tools/refactor-proof/bin/tc-proof-host &&
    test -x /proof/bootstrap/bin/taskfmt &&
    echo LAYOUT_OK
  '
```

---

## macOS mount alternatives

If root symlinks are undesirable, use nullfs (macOS):

```sh
export TC_BIND=/private/tmp/tc-task-001-bind
sudo mkdir -p /task /work /proof/bootstrap
sudo mount -t nullfs "$TC_BIND/task" /task
sudo mount -t nullfs "$TC_BIND/work" /work
sudo mount -t nullfs "$TC_BIND/proof/bootstrap" /proof/bootstrap
```

Unmount:

```sh
sudo umount /task /work /proof/bootstrap
```

The sandbox script `mount` subcommand tries symlinks first (`ln -sfn`); set `TC_MOUNT_MODE=nullfs` to prefer nullfs.

---

## Known blockers (unchanged)

| Blocker | Mitigation |
| --- | --- |
| Root paths need sudo on macOS | `scripts/task-001-verify-sandbox.sh mount` (interactive) |
| Progress must reach `DONE` | Operator checklist events through leaf 3.1 |
| Host matrix needs Darwin | Run verify on macOS host, not Linux Docker |
| `visual-baseline` tag | Never move, retarget, or recreate |

---

## Related documents

- [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) — prep-wave1 verify attempt and blockers
- [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) — OB-006 / SO-005 procedure
- [`task-001-ci-requirements.md`](task-001-ci-requirements.md) — full container and bootstrap spec
- [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine check authority
