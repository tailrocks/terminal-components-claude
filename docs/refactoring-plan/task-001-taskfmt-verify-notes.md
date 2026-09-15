# TASK-001 standalone taskfmt verify notes (OB-006 / SO-005 prep)

> **Historical snapshot:** Qualification on retired `.worktrees/main` @ `task-001-bootstrap`. Operators use `.worktrees/campaign` / `refactor/holla-parity` — see [path-contract.md](path-contract.md) and [task-001-verify-container.md](task-001-verify-container.md).

**Date:** 2026-09-15  
**Branch (planning):** `prep-wave1-verify`  
**Branch (worktree):** `task-001-bootstrap` @ `3ed5157074d5a05f3a78149b17f9ae01d2324c25`  
**Worktree path:** `/Users/donbeave/Projects/terminal-components-claude/.worktrees/main`  
**Operator runbook:** [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6  
**CI layout:** [`task-001-ci-requirements.md`](task-001-ci-requirements.md)  
**Container simulation:** [`task-001-verify-container.md`](task-001-verify-container.md), [`scripts/task-001-verify-sandbox.sh`](../../scripts/task-001-verify-sandbox.sh)

This document records a **prep-wave1** attempt at the standalone `taskfmt verify` completion gate for TASK-001. It does **not** authorize SO-005 sign-off or receipt issuance.

---

## Verdict summary

| Gate criterion | Result |
| --- | --- |
| `taskfmt verify` exit code | **0** (process exit) |
| Last stdout line | **`RESULT FAIL`** (not `DONE`) |
| **OB-006 / SO-005 reached?** | **No** — gate requires exit 0 **and** last line `DONE` |

**Primary blockers**

1. **Container path layout** — `verify.toml` invokes checks at hardcoded `/task/`, `/work/`, `/proof/bootstrap/` paths. Without root bind mounts or an operator container exposing those paths, CHK-001/004/005/006/007 fail immediately (Python rc 2, file not found).
2. **Root mount blocked** — `scripts/task-001-verify-sandbox.sh mount` requires interactive `sudo`; non-interactive session cannot supply credentials (`sudo -n` also fails). Bind layout under `/tmp/tc-task001-verify` is valid (`docker-smoke` → `LAYOUT_OK`) but root paths remain absent.

**SO-005 achievable without sudo?** **No.** TASKFMT env overrides (`TASKFMT_ROOT`, `TASKFMT_TASK_DIR`, `TASKFMT_CONFIG`) maximize non-root pass count (`pass=5 fail=5`) but do **not** rewrite `verify.toml` subprocess argv. Full gate requires root `/task`, `/work`, `/proof/bootstrap` mounts on a Darwin host with `sandbox-exec`.

**Resolved in this session (not sufficient for SO-005):**

- Progress through leaf **3.1** with `state=DONE` — progress check **PASS**.
- Scope **PASS** after removing worktree `.qual/` scratch (taskfmt scope ignores `.gitignore`; untracked files under `.qual/` still fail `writable_paths`).

**Advisory qualification (substituted paths, not via taskfmt verify):** CHK-001, CHK-004, CHK-005 all exit **0** against worktree @ `cf2e79a0` after `sync-binaries.sh`. Implementation appears ready; the standalone gate path is blocked on environment layout and progress completion, not on driver failures.

---

## 1. Pinned taskfmt install

Binary already present; recreation skipped.

```sh
# Verify existing install (no recreate needed)
/tmp/taskfmt-install/bin/taskfmt --version
# taskfmt 0.2.0 (git 52d9f1eb7721f409bc47beb9fced7997b5c13ede)

/tmp/taskfmt-install/bin/taskfmt fingerprint
# 52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4

git -C /tmp/taskfmt-qualification rev-parse HEAD
# 52d9f1eb7721f409bc47beb9fced7997b5c13ede

shasum -a 256 /tmp/taskfmt-install/bin/taskfmt
# 55528a01d987489f9b8ae263eb913c85f0f7d6d540ae2d04efad0d8e363e5a68

shasum -a 256 /tmp/taskfmt-qualification/experiment.toml
# d236cca80572bca829b950896780dbe531956af4f0d23c43153ef8d512819c2a
```

**Exit codes:** all **0**. Pins match campaign values in [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md).

If missing, recreate:

```sh
export TASKFMT_REV=52d9f1eb7721f409bc47beb9fced7997b5c13ede
export TC_TASKFMT_SOURCE=/tmp/taskfmt-qualification
export TC_TASKFMT=/tmp/taskfmt-install/bin/taskfmt

rm -rf "$TC_TASKFMT_SOURCE" /tmp/taskfmt-install
git clone git@github.com:donbeave/task-format.git "$TC_TASKFMT_SOURCE"
git -C "$TC_TASKFMT_SOURCE" checkout "$TASKFMT_REV"
cargo install --locked --root /tmp/taskfmt-install \
  --path "$TC_TASKFMT_SOURCE/harness" --bin taskfmt
```

---

## 2. Session paths

```sh
export TC_PLANNING_REPO=/Users/donbeave/Projects/terminal-components-claude
export TC_WORKTREE=$TC_PLANNING_REPO/.worktrees/main
export TC_CATALOG_ROOT=$TC_PLANNING_REPO/refactoring-tasks/terminal-components
export TC_TASKFMT=/tmp/taskfmt-install/bin/taskfmt
export TC_TASKFMT_SOURCE=/tmp/taskfmt-qualification
export TC_CANDIDATE=$TC_WORKTREE
export TC_BASE=$(git -C "$TC_CANDIDATE" rev-parse HEAD)
# TC_BASE=3ed5157074d5a05f3a78149b17f9ae01d2324c25
```

---

## 3. Worktree build + binary sync

```sh
cd "$TC_WORKTREE"
cargo build -p refactor-proof
tools/refactor-proof/scripts/sync-binaries.sh
```

| Step | Exit |
| --- | ---: |
| `cargo build -p refactor-proof` | **0** |
| `sync-binaries.sh` | **0** |

Post-sync `host_sha256`: `97137da559217c39b4fce172556df7a6a1ad27e978e018725a9b05ee86092481`

---

## 4. Progress init and completion (runbook §6a)

Session run directory: `/tmp/tc-task001-verify/run-20260915T123715Z`

```sh
export TC_RUN=/tmp/tc-task001-verify/run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_RUN/logs"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_RUN/progress-init.md"
```

| Field | Value |
| --- | --- |
| Exit | **0** |
| Output line | `PROGRESS … task=TASK-001 current=1.1` |
| Initial state | `IN_PROGRESS` / leaf `1.1` |

Operator freeze handling: append checklist events through leaf **3.1** (leaves `1.1`, `2.1`, `2.2`, `2.3`, `2.4`, `3.1`). Minimal terminal event stream for this session:

```text
- 1 | STARTED | 1.1
- 2 | DONE | 1.1
- 3 | STARTED | 2.1
- 4 | DONE | 2.1
- 5 | STARTED | 2.2
- 6 | DONE | 2.2
- 7 | STARTED | 2.3
- 8 | DONE | 2.3
- 9 | STARTED | 2.4
- 10 | DONE | 2.4
- 11 | STARTED | 3.1
- 12 | DONE | 3.1
```

Header: `state: DONE`, `current: NONE`, `latest_event: 12`. Written to `$TC_RUN/progress.md` for verify.

---

## 5. Standalone taskfmt verify — bind-tree `/work` equivalent (no sudo mount)

When `mount` is blocked, run from the bind-tree work symlink with TASKFMT env overrides (equivalent to `/work` + `/proof/bootstrap` config):

```sh
export TC_BIND=/tmp/tc-task001-verify
export TC_RUN=/tmp/tc-task001-verify/run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_RUN/logs"

# Progress: reuse .qual sandbox DONE file or regenerate per task-001-progress-completion-guide.md
cp "$TC_WORKTREE/.qual/task-001-progress-sandbox/progress-done.md" "$TC_RUN/progress.md"

# Scope: remove .qual scratch before verify (taskfmt scope sees untracked files regardless of .gitignore)
rm -rf "$TC_WORKTREE/.qual"

scripts/task-001-verify-sandbox.sh prepare   # TC_BIND=/tmp/tc-task001-verify

cd "$TC_BIND/work"
export TASKFMT_ROOT="$TC_BIND/work"
export TASKFMT_TASK_DIR="$TC_BIND/task"
export TASKFMT_CONFIG="$TC_BIND/proof/bootstrap/experiment.toml"

"$TC_TASKFMT" verify \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs" \
  --verbose 2>&1 | tee "$TC_RUN/taskfmt-verify.log"
```

Direct-flag variant (same ceiling — CHK argv still hardcoded):

```sh
cd "$TC_CANDIDATE"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" verify \
  --root "$TC_CANDIDATE" \
  --task-dir "$TC_CATALOG_ROOT/completion/001" \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs" \
  --verbose 2>&1 | tee "$TC_RUN/taskfmt-verify.log"
```

### Outcome (latest session @ `20260915T124707Z`)

| Metric | Value |
| --- | --- |
| Process exit | **0** |
| Last stdout line | **`RESULT FAIL`** |
| SUMMARY | **`pass=5 fail=5`** (max without sudo mount) |

### Per-check results (latest session)

| Check | Result | Exit in log | Blocker |
| --- | --- | ---: | --- |
| config | PASS | — | — |
| task_lint | PASS | — | — |
| scope | **PASS** | — | `.qual/` removed before verify |
| forbidden_paths | PASS | — | — |
| forbidden_patterns | PASS | — | — |
| CHK-001 | **FAIL** | **2** | `/task/trusted/proof-bootstrap/host-bootstrap-driver.py` not found |
| CHK-004 | **FAIL** | **2** | `/task/trusted/proof-bootstrap/proof-comparator-bootstrap.py` not found |
| CHK-005 | **FAIL** | **2** | same as CHK-001 |
| CHK-006 | **FAIL** | **2** | same as CHK-001 |
| CHK-007 | **FAIL** | **2** | same as CHK-001 |
| progress | **PASS** | — | `state=DONE`, leaf **3.1** complete |

Prior session (`20260915T123715Z`, direct flags, `.qual/` present): `pass=4 fail=6` (scope FAIL).

Example CHK-001 log excerpt (`$TC_RUN/logs/CHK-001.log`):

```text
exit=2
matcher kind=exit expected="0" actual="2" pass=false
stderr:
… can't open file '/task/trusted/proof-bootstrap/host-bootstrap-driver.py': [Errno 2] No such file or directory
```

**Root cause:** `verify.toml` argv uses container mount paths (`/task/…`, `/work/…`, `/proof/bootstrap/…`). Direct `--task-dir` only affects taskfmt's own checks, not subprocess argv rewriting.

---

## 6. Container path simulation attempt

Via [`scripts/task-001-verify-sandbox.sh`](../../scripts/task-001-verify-sandbox.sh) with `TC_BIND=/tmp/tc-task001-verify`:

```sh
export TC_BIND=/tmp/tc-task001-verify
scripts/task-001-verify-sandbox.sh prepare
scripts/task-001-verify-sandbox.sh mount      # blocked: sudo password required
scripts/task-001-verify-sandbox.sh docker-smoke   # LAYOUT_OK (paths only)
```

Prepared bind layout:

```text
/tmp/tc-task001-verify/task              → $TC_CATALOG_ROOT/completion/001
/tmp/tc-task001-verify/work              → $TC_WORKTREE
/tmp/tc-task001-verify/proof/bootstrap/  → taskfmt bin, task-format checkout, experiment.toml
```

| Step | Exit | Notes |
| --- | ---: | --- |
| `prepare` | **0** | Symlinks valid under `$TC_BIND` |
| `mount` | **blocked** | `sudo: a password is required` (non-interactive; `sudo -n` exit 1) |
| `docker-smoke` | **0** | `LAYOUT_OK` — container volume paths resolve; does not run CHK drivers |
| `verify` (bind-tree + env) | **0** proc / **FAIL** gate | `pass=5 fail=5`; CHK blocked on missing `/task` root paths |

**Unblock options for operator:** run `scripts/task-001-verify-sandbox.sh mount` with interactive sudo (symlink or `TC_MOUNT_MODE=nullfs`); then `layout-smoke` and `verify` from `/work`. Linux Docker alone is insufficient — CHK host-matrix drivers require Darwin `sandbox-exec`.

---

## 7. Advisory driver qualification (substituted paths)

These runs **bypass** `taskfmt verify` and invoke drivers with real filesystem paths. They confirm worktree readiness but do **not** satisfy OB-006.

```sh
export TC_BOOTSTRAP=$TC_CATALOG_ROOT/completion/001/trusted/proof-bootstrap

# CHK-001 equivalent (self-test)
python3 "$TC_BOOTSTRAP/host-bootstrap-driver.py" \
  --taskfmt "$TC_TASKFMT" \
  --taskfmt-source "$TC_TASKFMT_SOURCE"
# exit 0 — schema tc-host-bootstrap-taskfmt/v1

# CHK-004 equivalent
python3 "$TC_BOOTSTRAP/proof-comparator-bootstrap.py" \
  --runner "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof"
# exit 0 — 141/141 invocations, failures=[]

# CHK-005 equivalent (Mach-O host, not wrapper)
python3 "$TC_BOOTSTRAP/host-bootstrap-driver.py" \
  --host "$TC_WORKTREE/target/debug/tc-proof-host" \
  --taskfmt "$TC_TASKFMT" \
  --taskfmt-source "$TC_TASKFMT_SOURCE"
# exit 0 — schema tc-host-bootstrap-qualification/v1, 32 cases
```

| Driver | Exit | Duration (approx) |
| --- | ---: | --- |
| CHK-001 self-test | **0** | ~2s |
| CHK-004 comparator | **0** | ~30s |
| CHK-005 host matrix | **0** | ~7m |

CHK-006 and CHK-007 use identical argv in `verify.toml`; advisory CHK-005 pass implies they would pass under container paths.

---

## 8. Remaining steps for SO-005 (operator)

1. Run `scripts/task-001-verify-sandbox.sh mount` (interactive sudo) so `/task`, `/work`, `/proof/bootstrap` resolve per `verify.toml`.
2. Remove worktree pollution outside `writable_paths` (e.g. `rm -rf .qual/task-001-progress-sandbox` in worktree) before verify.
3. Re-run §6b from [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) with frozen progress through **3.1**:

```sh
cd "$TC_CANDIDATE"
"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" verify \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs"
# Required: exit 0 AND last line DONE
```

4. Record EV-019–EV-024 in [`task-001-operator-evidence-template.md`](task-001-operator-evidence-template.md).

---

## 9. Artifact paths (latest session)

| Path | Purpose |
| --- | --- |
| `/tmp/tc-task001-verify/run-20260915T124707Z/progress.md` | frozen progress (`state=DONE`, through **3.1**) |
| `/tmp/tc-task001-verify/run-20260915T124707Z/logs/` | per-check logs (`pass=5 fail=5`) |
| `/tmp/tc-task001-verify/run-20260915T124707Z/taskfmt-verify.log` | full verify transcript (verbose, bind-tree + env) |
| `/tmp/tc-task001-verify/` | bind layout (`prepare` output; root mount not applied) |

Prior session artifacts: `run-20260915T123715Z/` (scope FAIL), `/tmp/tc-task-001-run-20260915T122252Z/`, `/private/tmp/tc-task-001-bind/`.

---

## 10. References

- [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine authority (container paths)
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — Phase 3 advisory matrix
- [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) — planning issuance (does not arm `/goal`)
