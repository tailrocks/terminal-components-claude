# TASK-001 standalone taskfmt verify notes (OB-006 / SO-005 prep)

**Date:** 2026-09-15  
**Branch (planning):** `prep-wave1-verify`  
**Branch (worktree):** `task-001-bootstrap` @ `cf2e79a068518e229751f82b635832ecaba8ae4d`  
**Worktree path:** `/Users/donbeave/Projects/terminal-components-claude/.worktrees/main`  
**Operator runbook:** [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6  
**CI layout:** [`task-001-ci-requirements.md`](task-001-ci-requirements.md)

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
2. **Incomplete progress** — `progress-init` leaves `state=IN_PROGRESS`, `current=1.1`. Even if CHK commands were reachable, the progress check fails until operator completes checklist events through leaf **3.1** per campaign executor protocol.
3. **Root symlink simulation blocked** — creating `/task`, `/work`, `/proof/bootstrap` at filesystem root requires `sudo`; non-interactive session cannot supply credentials.

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
# TC_BASE=cf2e79a068518e229751f82b635832ecaba8ae4d
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

## 4. Progress init (runbook §6a)

```sh
export TC_RUN=/tmp/tc-task-001-run-20260915T122252Z
mkdir -p "$TC_RUN/logs"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_RUN/progress.md"
```

| Field | Value |
| --- | --- |
| Exit | **0** |
| Output line | `PROGRESS … task=TASK-001 current=1.1` |
| Initial state | `IN_PROGRESS` / leaf `1.1` |

---

## 5. Standalone taskfmt verify — direct flags (no container layout)

From worktree with explicit `--root`, `--task-dir`, `--base`, `--progress`, `--log-dir`:

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

### Outcome

| Metric | Value |
| --- | --- |
| Process exit | **0** |
| Last stdout line | **`RESULT FAIL`** |
| SUMMARY | `pass=4 fail=6` |

### Per-check results

| Check | Result | Exit in log | Blocker |
| --- | --- | ---: | --- |
| config | PASS | — | — |
| task_lint | PASS | — | — |
| scope | PASS | — | — |
| forbidden_paths | PASS | — | — |
| forbidden_patterns | PASS | — | — |
| CHK-001 | **FAIL** | **2** | `/task/trusted/proof-bootstrap/host-bootstrap-driver.py` not found |
| CHK-004 | **FAIL** | **2** | `/task/trusted/proof-bootstrap/proof-comparator-bootstrap.py` not found |
| CHK-005 | **FAIL** | **2** | same as CHK-001 |
| CHK-006 | **FAIL** | **2** | same as CHK-001 |
| CHK-007 | **FAIL** | **2** | same as CHK-001 |
| progress | **FAIL** | **1** | `state=IN_PROGRESS (want DONE)` |

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

Prepared bind layout under `/private/tmp/tc-task-001-bind`:

```sh
export TC_BIND=/private/tmp/tc-task-001-bind
mkdir -p "$TC_BIND/proof/bootstrap/bin"
ln -sf "$TC_CATALOG_ROOT/completion/001" "$TC_BIND/task"
ln -sf "$TC_WORKTREE" "$TC_BIND/work"
ln -sf "$TC_TASKFMT" "$TC_BIND/proof/bootstrap/bin/taskfmt"
ln -sf "$TC_TASKFMT_SOURCE" "$TC_BIND/proof/bootstrap/task-format"
cp "$TC_TASKFMT_SOURCE/experiment.toml" "$TC_BIND/proof/bootstrap/experiment.toml"
```

Attempted root symlinks (requires interactive sudo — **blocked**):

```sh
sudo ln -sfn "$TC_BIND/task" /task
sudo ln -sfn "$TC_BIND/work" /work
sudo ln -sfn "$TC_BIND/proof/bootstrap" /proof/bootstrap
```

| Step | Exit | Notes |
| --- | ---: | --- |
| Layout prep under `$TC_BIND` | **0** | Symlinks valid |
| Root symlink creation | **blocked** | `sudo` waits for password; non-interactive session |

**Unblock options for operator:** macOS bind/nullfs mounts at `/task`, `/work`, `/proof/bootstrap`; or campaign container with those paths per [`task-001-ci-requirements.md`](task-001-ci-requirements.md). Linux Docker alone is insufficient — CHK drivers require Darwin `sandbox-exec`.

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

1. Provision container/bind layout so `/task`, `/work`, `/proof/bootstrap` resolve per `verify.toml`.
2. Complete checklist events in `$TC_RUN/progress.md` through leaf **3.1** (operator freeze handling per [`campaign-executor-protocol.md`](campaign-executor-protocol.md)).
3. Re-run §6b from [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md):

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

## 9. Artifact paths (this session)

| Path | Purpose |
| --- | --- |
| `/tmp/tc-task-001-run-20260915T122252Z/progress.md` | progress-init output |
| `/tmp/tc-task-001-run-20260915T122252Z/logs/` | per-check logs from failed verify |
| `/tmp/tc-task-001-run-20260915T122252Z/taskfmt-verify.log` | full verify transcript |
| `/private/tmp/tc-task-001-bind/` | prepared (unmounted) simulation layout |

---

## 10. References

- [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine authority (container paths)
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — Phase 3 advisory matrix
- [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) — planning issuance (does not arm `/goal`)
