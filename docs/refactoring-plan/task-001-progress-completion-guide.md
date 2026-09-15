# TASK-001 progress completion guide (OB-006 prep)

**Date:** 2026-09-15  
**Branch (planning):** `prep-wave1-verify`  
**Worktree:** `.worktrees/campaign` — branch `refactor/holla-parity` (see [`path-contract.md`](path-contract.md) § Worktree)  
**Sandbox:** `.worktrees/campaign/.qual/task-001-progress-sandbox/` (gitignored scratch — never commit)  
**Related:** [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md), [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6

This guide documents how to reach **`progress/v1` `state: DONE`** for TASK-001 so the standalone `taskfmt verify` **progress check** passes. It does **not** satisfy OB-006 / SO-005 by itself — CHK-001/004/005/006/007 still require container mount paths (`/task`, `/work`, `/proof/bootstrap`) and real qualification evidence.

---

## 1. Checklist leaves (from `001/README.md`)

TASK-001 has **six** checklist leaves (parent sections `1`, `2`, `3` are not leaves):

| Leaf | Description | Requirements / checks |
| --- | --- | --- |
| **1.1** | Validate source pins, prerequisites and forbidden scope | R-004, AC-004, CHK-001 |
| **2.1** | Strict comparison contexts, manifest validation, exact frame/semantic/provenance comparison | R-001, AC-001, CHK-004 |
| **2.2** | Isolated workers, protected receipt resolution, complete source freeze, external standalone taskfmt verification | R-002, AC-002, CHK-006 |
| **2.3** | Accepted binary installation and local compare-and-swap integration with DCO/coauthor commits in synthetic fixtures | R-003, AC-003, CHK-005 |
| **2.4** | Independent comparator and host qualification, including recovery positives and hostile write/read attempts | R-001, AC-001, CHK-004 |
| **3.1** | Complete host-controlled gate and preserve evidence | R-005, AC-005, CHK-007 |

Leaf order is fixed: `1.1 → 2.1 → 2.2 → 2.3 → 2.4 → 3.1`.

---

## 2. Progress grammar (from `001/AGENTS.md`)

- `progress.md` uses schema **`progress/v1`**, generated only by **`taskfmt progress-init`**.
- Append **versioned events** under `## Events`; do not duplicate the README checklist.
- Allowed event statuses: `STARTED`, `DONE`, `FAILED`, `REOPENED`, `BLOCKED`, `NEEDS_REPLAN`.
- Header fields (`state`, `current`, `latest_event`) must **match the event reduction** — taskfmt rejects disagreement.
- **`state: DONE`** requires every leaf above marked `DONE` in the event stream; derived `current` is `NONE`.
- Free-form notes belong under `## Handoff` only.
- There is **no** `taskfmt progress-append` subcommand; executors append events and update derived headers, then re-validate with `taskfmt verify`.

### Valid transition rules (taskfmt `progress/v1` reducer)

| Rule | Detail |
| --- | --- |
| Contiguous sequence | Event numbers start at `1` with no gaps |
| `STARTED` | No active leaf; leaf must not already be completed |
| `DONE` | Requires the same leaf to be currently active (`STARTED` without intervening terminal) |
| Between leaves | **`IN_PROGRESS` always requires an active leaf** — a file with `DONE \| 1.1` but no following `STARTED \| 2.1` is **invalid** (`progress: IN_PROGRESS requires an active leaf`) |
| Terminal | `BLOCKED` / `NEEDS_REPLAN` on the active leaf stops further events |
| Completion | After `DONE \| 3.1` with all six leaves completed → `state: DONE`, `current: NONE` |

**Practical implication:** when moving between leaves, append **`DONE \| <leaf>`** and **`STARTED \| <next-leaf>`** in the same edit (or before any verify/resume). Mid-leaf pause with only `STARTED` (no `DONE` yet) is valid.

---

## 3. Sandbox session paths

```sh
export TC_PLANNING_REPO=/Users/donbeave/Projects/terminal-components-claude
export TC_WORKTREE=$TC_PLANNING_REPO/.worktrees/campaign
export TC_CATALOG_ROOT=$TC_PLANNING_REPO/refactoring-tasks/terminal-components
export TC_CANDIDATE=$TC_WORKTREE
export TC_BASE=$(git -C "$TC_CANDIDATE" rev-parse HEAD)
# cf2e79a068518e229751f82b635832ecaba8ae4d

export TC_TASKFMT=/tmp/taskfmt-install/bin/taskfmt
export TC_TASKFMT_SOURCE=/tmp/taskfmt-qualification

export TC_SANDBOX=$TC_WORKTREE/.qual/task-001-progress-sandbox
mkdir -p "$TC_SANDBOX/logs"
```

Keep all sandbox artifacts under `.qual/` — untracked scratch, not operator evidence.

---

## 4. Step A — `progress-init`

```sh
"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_SANDBOX/progress.md"
```

**Observed output (2026-09-15 sandbox):**

```text
PROGRESS …/progress.md task=TASK-001 current=1.1
```

Initial header:

```yaml
state: IN_PROGRESS
current: 1.1
latest_event: 1
```

Initial events:

```text
- 1 | STARTED | 1.1
```

---

## 5. Step B — append events through leaf 3.1

### Complete event sequence (12 events)

After `progress-init`, append events **2–12**:

```text
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

### Final derived header (must match events)

```yaml
state: DONE
current: NONE
latest_event: 12
```

### One-shot write (sandbox qualification only)

For **sandbox grammar proof** (not operator evidence), write the completed file:

```sh
cat > "$TC_SANDBOX/progress-done.md" <<'EOF'
---
schema: progress/v1
task: TASK-001
state: DONE
current: NONE
latest_event: 12
---

## Events
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

## Handoff
CURRENT_FAILURE: none
DECISIONS: none
EOF
```

### Real execution (operator / executor)

During actual TASK-001 execution:

1. Work leaf **1.1** → append `DONE | 1.1` then immediately `STARTED | 2.1`.
2. Repeat for **2.1 → 2.2 → 2.3 → 2.4**, running the verifier checks named in AGENTS.md step 6 at each leaf where applicable.
3. At **3.1**, run `taskfmt verify` from `/work` with **nonempty** `$TC_RUN/progress.md` until exit 0 and last line `DONE` (after container mounts — see §8), **then** append `DONE | 3.1`.
4. Run full `taskfmt verify` with the completed progress file as completion evidence.

Progress events record **coordination state**, not gate evidence — checks must pass independently.

---

## 6. Step C — validate progress check

```sh
cd "$TC_CANDIDATE"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" verify \
  --root "$TC_CANDIDATE" \
  --task-dir "$TC_CATALOG_ROOT/completion/001" \
  --base "$TC_BASE" \
  --progress "$TC_SANDBOX/progress-done.md" \
  --log-dir "$TC_SANDBOX/logs" \
  --verbose 2>&1 | tee "$TC_SANDBOX/taskfmt-verify-done-progress.log"
```

### Sandbox results (2026-09-15)

| Check | Result | Notes |
| --- | --- | --- |
| **progress** | **PASS** | `state=DONE (want DONE)` |
| CHK-001/004/005/006/007 | FAIL (rc 2) | `/task/…` paths missing — container layout required |
| scope | FAIL | `.qual/` sandbox files outside `writable_paths` |
| **Overall** | `RESULT FAIL` | Expected without container mounts |

**Progress-only verdict:** **`state: DONE` achieved in sandbox** — `CHECK progress PASS`.

Full OB-006 still blocked on container path layout ([`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) §5–6).

---

## 7. Incremental append validation (sandbox)

Incremental appends were exercised under `$TC_SANDBOX/incremental/`. Intermediate files fail the progress check when a leaf is `DONE` but the next leaf is not yet `STARTED` (invalid `IN_PROGRESS` with no active leaf). After event 12 (`DONE | 3.1`):

```text
CHECK progress PASS
state: DONE
current: NONE
latest_event: 12
```

---

## 8. Operator run directory (production path)

For SO-005 evidence, use an operator run directory outside the worktree (not `.qual/`):

```sh
export TC_RUN=/tmp/tc-task-001-run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_RUN/logs"

"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_RUN/progress.md"
```

Complete events through **3.1** per §5, then provision container paths (**mandatory** — `--task-dir` does not rewrite CHK argv):

```sh
cd "$TC_PLANNING_REPO"
export TC_WORKTREE="$TC_CANDIDATE"
./scripts/task-001-verify-sandbox.sh prepare
./scripts/task-001-verify-sandbox.sh mount
./scripts/task-001-verify-sandbox.sh layout-smoke
```

Run verify from **`/work`** with **`/proof/bootstrap/experiment.toml`**:

```sh
cd /work

"$TC_TASKFMT" --config /proof/bootstrap/experiment.toml verify \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs"
# Required: exit 0 AND last stdout line DONE
```

See [`path-contract.md`](path-contract.md) and [`task-001-verify-container.md`](task-001-verify-container.md).

See [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6 for EV-019–EV-024 recording.

---

## 9. References

- [`refactoring-tasks/terminal-components/completion/001/README.md`](../../refactoring-tasks/terminal-components/completion/001/README.md) — checklist leaves
- [`refactoring-tasks/terminal-components/completion/001/AGENTS.md`](../../refactoring-tasks/terminal-components/completion/001/AGENTS.md) — progress grammar and verify protocol
- [`refactoring-tasks/terminal-components/completion/001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine authority (container paths)
- taskfmt `progress/v1` reducer — `task-format` harness @ `52d9f1eb7721f409bc47beb9fced7997b5c13ede`
