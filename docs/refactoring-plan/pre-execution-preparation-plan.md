# Historical pre-execution preparation plan

> **Superseded.** This plan is retained for provenance only. Do not replay its
> commands or readiness claims. Use
> [`execution-readiness-report.md`](execution-readiness-report.md).

**Date:** 2026-09-15  
**Catalog tip:** `c3814f27` (`visual-baseline`)  
**Purpose:** Consolidate §22.17 witness rejections, subagent repair specs, and task-package gaps into one actionable plan so campaign execution is predictable. **Planning only — no TASK-001 dispatch.**

**Inputs:** [`review-readiness-final.md`](review-readiness-final.md), five partition witness subagents, six repair-analysis subagents (2026-09-15).

---

## Executive summary

| Dimension | Status |
| --- | --- |
| Mechanical validators (lint, validate-plan, bootstrap freeze) | **Green** |
| §22.17 witness completeness | **Done** @ `3213fce2` |
| §22 planning acceptance | **20/20** |
| **`READY FOR REFACTORING EXECUTION`** | **Not issuable** |
| Partition verdicts | **1 VERIFIED, 4 REJECTED** |
| Task catalog ready to arm `/goal` | **No** — **8 P1 catalog repairs** + **2 P2 hardening** before arming |

**Bottom line:** The plan is structurally sound (parity/traceability/orphans clean). Execution predictability is blocked by **trusted-package projection gaps**, **prose-only machine authority**, and **executor protocol contradictions**. Fix these on the planning branch **before** arming the campaign `/goal` or dispatching TASK-001.

---

## What was rejected and why

### Partition A — Parity / traceability / orphans — **VERIFIED**

No catalog repair required for arming. Documented deferrals:

- 363/363 scenarios `REQUIRED-UNCAPTURED` (expected until baseline producers run)
- 116 Jackin/TablePro scenarios use generic-only disposition prose (acceptable at planning boundary; tighten optionally)
- 752/7885 branch inventory paths without semantic review (not whole-branch acceptance)

### Partition B — DAG / receipts — **REJECTED**

| Issue | Root cause | Risk if unpatched |
| --- | --- | --- |
| **WITNESS-01** | Identical `account-tests` argv on TASK-002–069; preparation vs production mode only in runtime host JSON | Mis-bound host recreates DAG-FINAL-01 receipt cycle |
| DAG-FINAL-02 residual | 146 whole files writable in TASK-008; span-only intent is prose/CHK-006 only | Executor edits outside approved spans pass scope lint |

### Partition C — Architecture / ADJ-22 / coordinator — **REJECTED**

| Issue | Root cause | Risk if unpatched |
| --- | --- | --- |
| **READINESS-01** | Coordinator TSV claims branch witnesses integrated into TASK-031; `source-witnesses.md` has only W-031-01…11 | TASK-031 closes registry without ADJ-22 branch witness membership |
| **READINESS-02** | `W-042-JUMP-SUBMIT` in TASK-031 projection; TASK-042 not a predecessor | Premature “complete registry” before Files jump exists |
| **READINESS-03** | Disposition TSV lives in `docs/` only; not in TASK-008 trusted package | Main architectural tests keep conflicting assertions while components pass |

Producer task witnesses (021–028) are **sound**; the gap is **coordinator → trusted-package binding**.

### Partition D — Verification / bootstrap / executor — **REJECTED**

| Issue | Root cause | Risk if unpatched |
| --- | --- | --- |
| **READINESS-04 / INT-02** | Canonical AGENTS step 7 mandates forbidden `taskfmt verify --progress ""`; campaign override is README prose only | Executor claims local DONE; host rejects or wrong tree integrated |
| **IW-02** | 479 checks reference absent `/proof/bin/tc-proof`; lint cannot catch next argv ABI mismatch | Silent harness ABI regressions |
| **IW-03** | TASK-001 manual bootstrap exception is prose-only | False first harness receipt poisons ledger |
| **READINESS-08 / IW-04** | Runner index suite runs only on `--group 070` | TASK-071/072 pass without index isolation regression |

Bootstrap self-tests and 7/7 freeze groups are **green**. Absent `tc-proof-host` is **expected** (TASK-001 deliverable).

### Partition E — Integration / TASK-069 — **REJECTED**

| Issue | Root cause | Risk if unpatched |
| --- | --- | --- |
| **READINESS-05** | Human adversarial review bound to CHK-007 `close` | Passing TASK-069 falsely implies review complete |
| **READINESS-06** | Fidelity visual + merge-readiness only in obligations prose | Host could run smoke tier or skip drift detection |
| **INT-03** | `positive_isolation` vectors declared but not executed; index-only-freeze liar missing | Stale/unstaged bytes integrated at execution |

TASK-069 validation-only scope (writable paths, forbidden oracle) is **strong**.

---

## Master fix registry

### Wave 1 — P0 arming blockers (catalog bytes)

Execute in order; parallelize within wave where files don't overlap.

| ID | Wave | Owner | Work package | Files (primary) | Acceptance |
| --- | ---: | --- | --- | --- | --- |
| **READINESS-02** | 1a | Planning | Remove W-042 from TASK-031 projection | `branch-diff-components-b-coordinator-projection.tsv`, `reaudit-branch-coordinator-integration.md`, `architecture-adjudication.md` | No `W-042-JUMP-SUBMIT` under `031/` |
| **READINESS-01** | 1b | Planning | TASK-031 branch witness index | **Create** `031/trusted/branch-host-projection.tsv`; edit `031/trusted/source-witnesses.md`, `031/README.md` | 10 branch IDs + W-031-10/11 indexed; producer paths resolve |
| **READINESS-03** | 1b | Planning | TASK-008 disposition trusted mirror | **Create** `008/trusted/branch-test-disposition-bindings.tsv`, `008/trusted/external-test-source-scope.md`; edit `008/trusted/obligations.md`, `008/README.md` | W-021-07, W-025-07/08, W-026-05 in trusted bytes |
| **READINESS-04** | 1c | Planning | Campaign AGENTS overlay | **Create** `terminal-components/CAMPAIGN_AGENTS.md` ×73; edit 73 READMEs; extend `validate-plan.py` | Executor-visible supersession of AGENTS step 7 |
| **WITNESS-01** | 1d | Planning | Machine-bind accounting mode | **Create** `071/trusted/check-context-templates/` + per-package `NNN/trusted/check-context-templates/CHK-*.json`; extend `validate-plan.py` | TASK-002–008 templates = preparation; 009+ = production |
| **READINESS-05** | 1e | Planning | Decouple human review from CHK-007 | **Create** `069/trusted/coordinator-review-contract.md`; edit `069/trusted/obligations.md`, `source-obligations.tsv`, `069/README.md` | CHK-007 = machine join only; review → COORD-ADVERSARIAL |
| **READINESS-06** | 1e | Planning | Freeze visual + merge-readiness contexts | **Create** `069/trusted/host-context/*.template.json`, `host-context-spec.md`; edit obligations clause 3/7 | Fidelity argv + snapshots digest + main ancestry inspectable |
| **validate-plan** | 1f | Planning | Coordinator binding linter | `evidence/validate-plan.py` | Fails if projection ≠ trusted bytes |

**Wave 1 gate:** `validate-plan.py --summary` 0 errors; `taskfmt lint` 73/73; spot-check partitions C, D, E.

### Wave 2 — P1 hardening (predictability)

| ID | Work package | Files | Blocks arming? |
| --- | --- | --- | --- |
| **READINESS-08** | Wire runner index into 071/072 **or** document+lint transitive TASK-070 index receipt | `071/README.md`, `072/README.md`, optional `verify.toml` comment | Soft — document if not wired |
| **IW-02** | Catalog argv smoke in planning gates | `planning-verification.md`, optional `validate-plan.py` dry-run | Soft |
| **IW-03** | TASK-001 operator bootstrap checklist + evidence fields | `campaign-executor-protocol.md`, `001/README.md` | Soft — operator discipline |
| **INT-03** | Host isolation qualification corpus | `host-bootstrap-vectors.json`, **new** `index-only-freeze-host.py`, `host-bootstrap-driver.py`, refreeze bootstrap | Execution prerequisite; can parallel Wave 1 |
| **008 span manifest** | `008/trusted/span-patch-manifest.tsv` binding CHK-006 | TASK-008 trusted | Reduces whole-file writable risk |
| **007 trusted copy** | Mirror `inline-test-source-scope.md` into `007/trusted/` | TASK-007 | Hygiene |

### Wave 3 — P2 hygiene (non-blocking)

| Item | Action |
| --- | --- |
| **READINESS-07** | Sync scenario count 363 and edge count 3258 in `planning-acceptance.md`, `069/trusted/obligations.md` |
| **TASK-001** | Differentiate CHK-005/006/007 gate phases in driver |
| **TASK-070** | Wire `capture-atomicity-bootstrap` into verify.toml |
| **TASK-031** | Align obligations CHK-004 references with verify.toml |
| **TASK-073** | Add `precondition-receipts.tsv` |
| **072** | Consolidate duplicate bootstrap trusted trees or add checksum manifest |

### Wave 4 — Expected TASK-001+ deliverables (not planning defects)

Do **not** fix on `visual-baseline` planning branch as production work:

| Deliverable | Task | Notes |
| --- | --- | --- |
| `tools/refactor-proof/bin/tc-proof-host` | TASK-001 | First production work on `.worktrees/campaign` / `refactor/holla-parity` |
| `tools/refactor-proof/bin/tc-proof` | TASK-001 | Installed at `/proof/bin/tc-proof` after acceptance |
| Runner/accounting/architecture ops | TASK-070/071/072 | Qualified via bootstrap drivers today; production Rust later |
| Oracle captures for 363 scenarios | TASK-002–006 + apps | REQUIRED-UNCAPTURED until baseline chain |
| Linux host isolation qualification | Post-TASK-001 | Darwin-only accepted with worker constraint |

---

## Per-task improvement matrix

Tasks needing **catalog edits now** (planning branch):

| Task | Severity | Fix | Wave |
| --- | --- | --- | --- |
| **031** | P1 | `branch-host-projection.tsv`; remove W-042; D-008 README | 1b |
| **008** | P1 | disposition TSV + external-test scope + obligations; optional span manifest | 1b, 2 |
| **069** | P1 | coordinator-review-contract; host-context templates; obligation decoupling | 1e |
| **071** | P1 | canonical accounting context templates (bases for all packages) | 1d |
| **002–008** | P1 | per-check `trusted/check-context-templates/` (preparation mode) | 1d |
| **009–068, 073** | P1 | production accounting templates (CHK-005 typical) | 1d |
| **001** | P2 | operator bootstrap checklist; CHK phase differentiation | 2, 3 |
| **007** | P2 | trusted inline-scope copy; discovery witness TSV | 2, 3 |
| **070** | P2 | capture-atomicity verify wiring; index receipt doc | 2, 3 |
| **071/072** | P2 | index suite scope or transitive receipt doc | 2 |
| **040/042/051** | — | No P1 gaps; optional README cross-refs | 3 |
| **ALL ×73** | P1 | `CAMPAIGN_AGENTS.md` + README bullet | 1c |

Tasks **correct as-is** for planning boundary: **042** (trusted fixtures complete), **072** (richest verify graph, drivers present), **073** (by design no capture checks).

---

## What NOT to change before arming

| Do not | Reason |
| --- | --- |
| Edit canonical `AGENTS.md` step 7 | Breaks byte-frozen hash policy; use `CAMPAIGN_AGENTS.md` overlay |
| Move/retarget `visual-baseline` tag | Frozen oracle-era pin |
| Implement production Rust on `visual-baseline` | Production starts from architectural `main` |
| Bless snapshots / modify `snapshots/` | Oracle authority |
| Issue `READY FOR REFACTORING EXECUTION` before Wave 1 gate | Witness found material P1 gaps |
| Arm campaign `/goal` before Wave 1 + operator authorization | Predictable execution requires closed catalog gaps |

---

## Predictable execution model (after Wave 1)

```text
Phase 0 complete (Wave 1 green + spot-check)
    ↓
Operator issues READY FOR REFACTORING EXECUTION @ post-repair SHA
    ↓
Operator authorizes campaign /goal (body from campaign-execution-prompt.md)
    ↓
`.worktrees/campaign` on `refactor/holla-parity` (`scripts/campaign-init.sh`)
    ↓
TASK-001 → tc-proof-host receipt (manual bootstrap exception documented)
    ↓
TASK-070 → TASK-071 → TASK-072 bootstrap chain
    ↓
TASK-002–008 preparation (preparation-mode contexts machine-bound)
    ↓
Parallel component/app tasks per DAG (CAMPAIGN_AGENTS.md governs executors)
    ↓
App closures 039/050/057/064 (fidelity visual @ acceptance boundaries)
    ↓
TASK-069 (machine gates from frozen host-context templates)
    ↓
Coordinator adversarial review (COORD-ADVERSARIAL — separate from CHK-007)
    ↓
Merge-ready integration branch (separate authorization to merge main)
```

**Iteration tiers during execution:** edit loop = targeted `TUISNAP_FAST=1` filters (~17 s); acceptance = fidelity full matrix (~45–60 min). See [`campaign-iteration-guide.md`](campaign-iteration-guide.md).

---

## Verification commands — readiness vs blockers

| Command | Role | Blocks edit loop? | Blocks arming? |
| --- | --- | --- | --- |
| `validate-plan.py --summary` | Planning integrity | No | Yes if errors |
| `taskfmt project lint` | Package schema | No | Yes if errors |
| `store_integrity` | Snapshot inventory | No | No |
| `TUISNAP_FAST=1` + filter | Edit-loop visual | No | No |
| CI smoke visual | PR gate (~113 s) | No | No |
| Fidelity full visual | Acceptance only | No (by design) | No (at planning) |
| `tc-proof-host verify` | Task acceptance | N/A until TASK-001 | Yes for execution |

---

## Acceptance checklist before arming `/goal`

### Planning (Wave 1)

- [ ] READINESS-01–06 repairs landed and committed
- [ ] WITNESS-01 context templates + validate-plan enforcement
- [ ] READINESS-04 `CAMPAIGN_AGENTS.md` on all 73 packages
- [ ] `validate-plan.py --summary` → `error_count: 0`
- [ ] `taskfmt project lint` → 73/73
- [ ] `freeze-bootstrap-assets.py --check` (all 7 groups) → 211 assets unchanged (or refrozen if INT-03 landed)
- [ ] Spot-check witness partitions C, D, E on new catalog SHA
- [ ] Update `review-readiness-final.md` disposition rows

### Authorization

- [ ] Issue **`READY FOR REFACTORING EXECUTION`** on exact post-repair catalog SHA
- [ ] Operator explicit authorization to arm campaign `/goal`
- [ ] Record catalog SHA in campaign ledger (`git rev-parse HEAD`)

### Explicit non-starters until above complete

- Do **not** dispatch TASK-001 production edits on planning branch
- Do **not** treat green mechanical validators as execution authorization
- Do **not** arm from unamended coordinator paste (missing iteration tiers)

---

## Recommended implementer dispatch (Wave 1)

Split across isolated agents to avoid file conflicts:

| Agent | Scope | Est. files |
| --- | --- | --- |
| **A** | READINESS-02 + READINESS-01 (coordinator + TASK-031) | ~6 |
| **B** | READINESS-03 (TASK-008 trusted) | ~6 |
| **C** | READINESS-04 (CAMPAIGN_AGENTS + 73 README bullets) | ~75 |
| **D** | WITNESS-01 templates (071 base + mechanical gen for 002–073) | ~80 |
| **E** | READINESS-05–06 (TASK-069 trusted/host-context) | ~10 |
| **F** | validate-plan.py extensions (coordinator + accounting bindings) | 1 |
| **G** | Coordinator synthesis + doc sync + mechanical verification | ~8 docs |

After all agents: single coordinator runs full gate checklist and records new catalog SHA.

---

## Related documents

| Document | Role |
| --- | --- |
| [`review-readiness-final.md`](review-readiness-final.md) | §22.17 witness record @ `3213fce2` |
| [`campaign-arming-readiness.md`](campaign-arming-readiness.md) | Prompt analysis + command benchmarks |
| [`campaign-execution-prompt.md`](campaign-execution-prompt.md) | Canonical `/goal` body (arm from here) |
| [`campaign-iteration-guide.md`](campaign-iteration-guide.md) | Edit-loop vs acceptance visual tiers |
| [`execution-readiness-assessment.md`](execution-readiness-assessment.md) | Current readiness verdict |
| [`campaign-executor-protocol.md`](campaign-executor-protocol.md) | Host handoff + TASK-001 exception |

---

## Verdict

**PRE-EXECUTION PREPARATION INCOMPLETE**

Close **Wave 1** (8 P1 items) on the planning branch, re-spot-check witness partitions, then issue `READY FOR REFACTORING EXECUTION`. Only after that should TASK-001 bootstrap begin on an architectural-`main` worktree.
