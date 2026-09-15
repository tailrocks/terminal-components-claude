# TASK-001 merge authorization checklist (PR #4 → `main`)

**Audience:** Operator (Alexey)  
**Purpose:** Mandatory pre-merge gate before merging [PR #4](https://github.com/donbeave/terminal-components-claude/pull/4) (`task-001-bootstrap` → `main`)  
**Planning branch:** `prep-wave1-verify` (catalog only — this checklist lives here; merge applies worktree branch)  
**Worktree branch:** `task-001-bootstrap`  
**Schema:** `tc-proof-merge-authorization-checklist/v1`  
**Machine authority:** [`proof-contract.md`](proof-contract.md) § Integration; [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) § Issuance statement

**This checklist authorizes merge to `main` only when every item below is checked at the recorded merge SHA.** It does **not** substitute for IW-03 receipt issuance, `/goal` arming, or integration-ref updates.

---

## Related documents

| Document | Role |
| --- | --- |
| [PR #4](https://github.com/donbeave/terminal-components-claude/pull/4) | Merge candidate — TASK-001 refactor-proof bootstrap |
| [`task-001-operator-evidence-draft.md`](task-001-operator-evidence-draft.md) | IW-03 evidence pre-fill (`tc-proof-operator-bootstrap-checklist/v1`) |
| [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) | Phase 4 operator procedure (OB-001–OB-008, SO-001–SO-007) |
| [`task-001-qualification-report.md`](task-001-qualification-report.md) | Advisory bootstrap qualification (CHK-004, host matrix, nextest) |
| [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) | Documented taskfmt verify exception / blocker notes (OB-006) |
| [`proof-contract.md`](proof-contract.md) | Integration ref policy — local CAS only; merge is separate authorization |

---

## Session variables (record at merge decision)

```sh
export TC_MERGE_SHA=                    # PR head commit at merge time (git rev-parse task-001-bootstrap)
export TC_MAIN_BASE=7b27732a8c3c131760ec3438f641cb3c11343a42
export TC_VISUAL_BASELINE_TAG=4a79c0a2d40fca46fc406b77157ce3b3f12ec16b  # peeled; must remain unmoved
export TC_EVIDENCE_DIR=/absolute/operator/retention/task-001-merge
```

---

## Pre-merge checklist

Complete every row at **`TC_MERGE_SHA`** before approving merge. Do not merge on stale advisory evidence from an earlier commit.

| # | Gate | Requirement | Verify | ☐ |
| ---: | --- | --- | --- | ---: |
| 1 | **IW-03 SO-001** | Planning authorization — EV-001 bound to issued catalog | [`task-001-operator-evidence-draft.md`](task-001-operator-evidence-draft.md) OB-001; coordinator sign-off | ☐ |
| 2 | **IW-03 SO-002** | Architectural worktree — EV-002 records source commit | Runbook §1; worktree from `main` @ `TC_MAIN_BASE` | ☐ |
| 3 | **IW-03 SO-003** | Bootstrap inputs frozen — EV-003–EV-007 before drivers | Runbook §4; pinned taskfmt @ `52d9f1eb…` | ☐ |
| 4 | **IW-03 SO-004** | Independent driver qualification — EV-008–EV-018 | Comparator + host drivers; EV-013 = EV-012; EV-018 = EV-017 | ☐ |
| 5 | **IW-03 SO-005** | Standalone **taskfmt verify** — exit **0** and last line **`DONE`**, **or** documented exception | Runbook §6; if blocked, cite [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) § Verdict summary and record exception ID + operator rationale in evidence retention | ☐ |
| 6 | **IW-03 SO-006** | Filesystem and trust inspection — EV-025–EV-026 | Runbook §7; map matches verify.toml + freeze evidence | ☐ |
| 7 | **IW-03 SO-007** | First receipt authorized — EV-027 populated | Runbook §8; coordinator sign-off before TASK-002+ handoff | ☐ |
| 8 | **CHK-004** | Comparator bootstrap **141/141** invocations pass | Re-run at merge SHA if HEAD moved since qualification report: `proof-comparator-bootstrap.py --runner …/bin/tc-proof` | ☐ |
| 9 | **Host matrix** | Host bootstrap **63/63** pass via synced Mach-O `bin/tc-proof-host` | `sync-binaries.sh` then `host-bootstrap-driver.py --host`; counts at **`TC_MERGE_SHA`** | ☐ |
| 10 | **taskfmt verify** | **`DONE`** on last stdout line **or** documented exception on file | Same as row 5; exception must cross-reference taskfmt-verify-notes, not paraphrase | ☐ |
| 11 | **`visual-baseline` tag** | Tag **unmoved** — no retarget, force-push, or release recreation | `git rev-parse refs/tags/visual-baseline^{commit}` → expect `TC_VISUAL_BASELINE_TAG` | ☐ |
| 12 | **`/goal` arming** | **Separate explicit authorization** — merge ≠ campaign arm | [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) § Issuance statement; runbook §11 | ☐ |
| 13 | **Integration ref policy** | Merge to `main` is **publication**; host `integrate` updates **local refs only** with expected-parent CAS | [`proof-contract.md`](proof-contract.md) — `integrate` never pushes/merges `main`; no integration ref advance without host verify at tested tree | ☐ |

**Aggregate IW-03:** SO-001 through SO-007 must all be signed (coordinator + operator roles per runbook §10) before merge authorization is valid.

---

## Hard stops

| Rule | Detail |
| --- | --- |
| **No merge without this checklist** | PR #4 states explicit operator authorization is required |
| **No `/goal` from merge** | Arming requires separate operator authorization per READY doc |
| **No `visual-baseline` mutation** | Frozen oracle tag and release remain @ peeled `4a79c0a2…` |
| **No integration-ref substitute** | Pushing `refs/heads/main` is not the same as `integrate --ref refs/heads/refactor/…` |
| **Re-qualify on SHA drift** | If PR head ≠ qualification report commit, re-run CHK-004 141/141 and host 63/63 at new SHA |

---

## Evidence capture at merge SHA

Record under `$TC_EVIDENCE_DIR`:

1. `git rev-parse HEAD` → **merge SHA**
2. CHK-004 driver JSON (141/141, failures `[]`)
3. Host matrix driver JSON (63/63)
4. `taskfmt verify` full transcript **or** exception document reference
5. Completed IW-03 evidence (from draft → signed template)
6. `git rev-parse refs/tags/visual-baseline^{commit}` confirmation
7. Operator merge authorization statement (name, UTC timestamp, SHA)

---

## Merge authorization sign-off

| Field | Value |
| --- | --- |
| Merge SHA (`TC_MERGE_SHA`) | |
| CHK-004 | /141 |
| Host matrix | /63 |
| taskfmt verify | `DONE` ☐ / exception documented ☐ |
| IW-03 SO-001–SO-007 | all signed ☐ |
| `visual-baseline` tag unchanged | ☐ |
| `/goal` arming | not authorized by this merge ☐ |
| Integration ref policy acknowledged | ☐ |

| Role | Name | Date (UTC) | Authorized merge PR #4 → `main` |
| --- | --- | --- | --- |
| Operator | | | ☐ |

---

## Planning validation

After updating this checklist on `prep-wave1-verify`:

```sh
cd /path/to/prep-wave1-verify
python3 docs/refactoring-plan/evidence/validate-plan.py --summary
# Expect error_count: 0
```
