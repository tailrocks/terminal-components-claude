# Historical campaign preparation report

> **Superseded.** This report is retained for provenance only. Its branch,
> receipt, and taskfmt claims are not current inputs. Use
> [`execution-readiness-report.md`](execution-readiness-report.md).

**Date:** 2026-09-15 (re-verified after subagent audit + command benchmarks)  
**Branch:** `visual-baseline` @ `84742bf6` (verify at arm time with `git rev-parse HEAD`)  
**Frozen tag:** `visual-baseline` → `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` (unmoved; branch ahead of tag)  
**Purpose:** Evidence-based readiness assessment for arming the campaign `/goal`. **Preparation only — not execution.**

---

## 1. Stored artifacts

| Artifact | Path | Status |
| --- | --- | --- |
| Campaign execution prompt | [`campaign-execution-prompt.md`](campaign-execution-prompt.md) | **Stored** @ `84742bf6` — canonical body with iteration amendments (not unamended paste) |
| Arming readiness analysis | [`campaign-arming-readiness.md`](campaign-arming-readiness.md) | **New** — prompt analysis, benchmarks, pre-arm checklist |
| Campaign iteration guide | [`campaign-iteration-guide.md`](campaign-iteration-guide.md) | **Stored** — tiered gates, filters, measured timings |
| Preparation report | This file | Updated 2026-09-15 |
| Executor protocol | [`campaign-executor-protocol.md`](campaign-executor-protocol.md) | Tracked |
| Visual validation tiers | [`refactoring-tasks/visual-validation.md`](../../refactoring-tasks/visual-validation.md) | Updated locally |

Copy the `/goal` body from `campaign-execution-prompt.md` verbatim when arming. Read `campaign-iteration-guide.md` before any production edit loop.

---

## 2. Git identities verified

Re-verified 2026-09-15 (git rev-parse + remote ls-remote).

| Authority | Expected SHA | Verified | Notes |
| --- | --- | --- | --- |
| **Architectural `main`** | `7b27732a8c3c131760ec3438f641cb3c11343a42` | **Match** | Local + `origin/main` |
| **Tag `visual-baseline` (peeled)** | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` | **Match** | Annotated tag object `1ee5ebdc…`; **not retargeted** |
| **Branch `visual-baseline` tip** | (ahead of tag) | `84742bf6` | Catalog bytes to freeze at arm time — **`git rev-parse` wins** |
| **Product oracle** | `02f5294bfdbf38004cc49130d0aff1d01f31434c` | **Match** | Distinct from branch tip — correct |
| **task-format authority** | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` | **Match** | Remote + installed `taskfmt 0.2.0` |
| **tui-snap campaign pin** | `0a2e490802b7b048cd96349c6af860f8a3a05c3d` | **Match** | `Cargo.toml` git rev |

**Stale prose:** [`PROGRESS.md`](PROGRESS.md) may cite older tag/branch SHAs. **`git rev-parse` wins.**

---

## 3. Phase 0 status: NOT READY

**Verdict:** `NOT READY FOR REFACTORING EXECUTION`

Last audited: 2026-09-15 (subagent round + local verification).

### Mechanical gates (green)

| Validator | Result |
| --- | --- |
| `validate-plan.py --summary` | **PASS** — 0 errors |
| `assemble-plan.py` | **PASS** — idempotent |
| `derive-task-graph.py` | **PASS** — 73 tasks, depth 35 |
| `taskfmt project lint terminal-components` | **PASS** — 73/73 |
| `freeze-bootstrap-assets.py --check` (7 groups) | **PASS** — 211 assets |
| `proof-comparator-bootstrap.py --self-test` | **PASS** — prep only; `"qualified_production_harness": false` |
| `cargo nextest run -E 'test(store_integrity)'` | **PASS** |

### Process gates (open)

| Gate | Status |
| --- | --- |
| Re-audit register | **0 open / 49 closed** (planning boundary; 2026-09-15) |
| Independent `READY FOR REFACTORING EXECUTION` | **Not issued** |
| `planning-acceptance.md` §22 | **20/20 proven** — witness [`review-readiness-final.md`](review-readiness-final.md) @ `3213fce2`; READINESS-01–05 open |
| Production harness (`tc-proof-host`, `tools/refactor-proof/`) | **Not built** — TASK-001 deliverable |
| BRANCH-01 whole-branch diff | **Closed (planning boundary)** — [coordinator witness](reaudit-branch-coordinator-integration.md); 752/7885 inventory reviewed |
| Campaign doc freeze | Pending commit of current round |

### Top remaining blockers

1. ~~**§22.17 fresh independent witness round**~~ — **Done** → [`review-readiness-final.md`](review-readiness-final.md). Repair **READINESS-01–05** before arming.
2. **`READY FOR REFACTORING EXECUTION` non-issuance** — explicit gate; do not infer from green mechanical validators
3. **TASK-001 bootstrap** — canonical `taskfmt verify` + `tc-proof-host` non-runnable until harness exists
4. **Catalog commit/freeze** — arm only from committed SHA; record exact tree hash at readiness

**Recently closed (no longer blockers):** ROOT-08/09 (F03/F06/F08c/F17 adjudicated); ROOT-11/12 validator repairs; tui-snap pin.

---

## 4. Verification command audit — iteration is NOT blocked

Measured 2026-09-15. Re-confirmed CI smoke locally: **7550 listed / ~302 PTY captures executed, all PASS, ~87 s**.

### Iteration tier (use during every edit loop)

| Command | Duration | Blocker? |
| --- | --- | --- |
| `cargo nextest run` | **~1–3 min** | No |
| `cargo nextest run -E 'test(store_integrity)'` | **~2.7 s** | No |
| `TUISNAP_FAST=1 … & test(<filter>)` (25 combos) | **~17 s** (e.g. `showcase_pages_overview`) | No |
| Single combo filter | **~1–20 s** (flow-dependent) | No |
| `cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline)'` | **~113 s** | No — PR smoke green |

### Boundary tier (mandatory at acceptance only — not every edit)

| Command | Duration | Blocker? |
| --- | --- | --- |
| Fidelity full matrix (no `TUISNAP_FAST`) | **~45–60 min est.** | Duration at boundaries only |
| `TUISNAP_FAST=1` full matrix | **~30 min measured** | **Not acceptance** — 7350/7550 pass; 200 expected FAST-vs-fidelity drift on timing-sensitive flows |

### Campaign canonical gates (blocked until TASK-001)

| Command | Status |
| --- | --- |
| `taskfmt verify …` | **FAIL** on bare checkout — expects container paths + `tc-proof-host` |
| `tc-proof-host verify --run …` | **Binary absent** |

**Conclusion:** Rust/visual iteration commands are fast and green. The prompt's mandatory **fidelity** full matrix (~45–60 min) runs only at task acceptance, app closures, and TASK-069 — **not** during edit loops. Edit loops use targeted `TUISNAP_FAST=1` filters per [`campaign-iteration-guide.md`](campaign-iteration-guide.md) §1.

---

## 5. Visual baseline performance (committed on branch)

Infra committed in `32c238e1`, `f395f284`, `c5384043`. Pin: `tui-snap` @ `0a2e490`.

| Scenario | Wall time | Notes |
| --- | --- | --- |
| `showcase_pages_overview` (25 combos, FAST) | **~17 s** | Edit-loop representative @ `84742bf6` |
| CI smoke (7550 slots, ~302 PTY captures) | **~113 s** | All green @ prep benchmark parent |
| Full FAST matrix | **~30 min** | Optional nightly; not acceptance oracle |
| Full fidelity matrix | **~45–60 min est.** | Mandatory acceptance gate |

---

## 6. Prompt preparation applied

The stored prompt matches the coordinator specification. Preparation added (without weakening acceptance):

| Amendment | Location | Status |
| --- | --- | --- |
| Iteration vs acceptance split | `campaign-execution-prompt.md` visual section + link to iteration guide | **Applied** |
| Edit-loop targeted `TUISNAP_FAST=1` commands | execution prompt + iteration guide §1 | **Applied** |
| Fidelity-only mandatory full gate | execution prompt, iteration guide §3–§4, `visual-validation.md` | **Applied** |
| FAST full matrix ≠ acceptance | iteration guide §3 (7350/7550 measured) | **Applied** |
| IMPLEMENTER may use targeted visual for feedback | execution prompt § Mandatory per-task agent protocol | **Applied** |

---

## 7. Pre-campaign checklist

### A. Phase 0 (required before arming `/goal`)

- [x] Close all **49** rows in [`reaudit-findings.tsv`](reaudit-findings.tsv) at planning boundary
- [x] Complete **fresh** independent witness round for §22.17 → [`review-readiness-final.md`](review-readiness-final.md) @ `3213fce2`
- [x] Finish **BRANCH-01** coordinator integration ([witness](reaudit-branch-coordinator-integration.md))
- [x] Re-prove **§22.17** in [`planning-acceptance.md`](planning-acceptance.md) (20/20)
- [ ] Repair **READINESS-01–05** per witness doc; spot-check partitions C/D/E
- [ ] Obtain fresh **`READY FOR REFACTORING EXECUTION`** on frozen catalog bytes
- [ ] Commit + hash-freeze campaign docs (`campaign-execution-prompt.md`, iteration guide, this report)

### B. Execution prerequisites (after Phase 0, before TASK-001 production)

- [ ] `.worktrees/campaign` on `refactor/holla-parity` (`scripts/campaign-init.sh`)
- [ ] Implement + qualify TASK-001 bootstrap (`tc-proof-host`, comparator fixtures)
- [ ] Coordinator re-runs Phase 0 gate checklist, then dispatches TASK-001 only

### C. Explicit non-starters

- **Do not** arm campaign `/goal` yet
- **Do not** dispatch TASK-001 on `visual-baseline` for production edits
- **Do not** treat green mechanical validators as execution authorization

---

## 8. Recommended next step

1. **Commit campaign prep docs** (prompt, iteration guide, prep report, `visual-validation.md`) — removes ambiguity before catalog freeze.
2. Execute **Wave 1** from [`pre-execution-preparation-plan.md`](pre-execution-preparation-plan.md) — consolidated repair spec for all rejected partitions.
3. **When Phase 0 green:** arm `/goal` from committed `campaign-execution-prompt.md`; first production work is TASK-001 bootstrap on `.worktrees/campaign` / `refactor/holla-parity`.

**Do not start the campaign goal in this preparation session.**

---

## Appendix: readiness summary

| Dimension | Ready? |
| --- | --- |
| Prompt stored + iteration guidance | **Yes** |
| Verification commands block iteration? | **No** — edit loop ~0.3 s–17 s targeted; smoke ~113 s; fidelity full matrix only at boundaries |
| Phase 0 mechanical validators | **Yes** |
| Phase 0 process / witness / findings | **Partial** — §22.17 witness done; READINESS-01–05 open (49/49 re-audit closed) |
| TASK-001 harness | **No** |
| Arm campaign `/goal` now? | **No** |
