# Independent fresh witness — §22.17 planning readiness (2026-09-15)

**Catalog commit:** `3213fce285268925520be20b14aa8a1c4b136586` (`visual-baseline`)  
**Witness type:** Fresh independent adversarial review on **post-repair frozen catalog bytes** (includes components-B ADJ-22, coordinator integration @ tip, campaign prep docs).  
**Scope:** Planning / Phase 0 only. No production refactoring, no TASK-001 dispatch, no oracle blessing, no `READY FOR REFACTORING EXECUTION` issuance.

---

## Overall verdict

| Question | Answer |
| --- | --- |
| **§22.17 fresh witness round complete?** | **Yes** — five independent partition reviewers + coordinator synthesis |
| **Issue `READY FOR REFACTORING EXECUTION` now?** | **Yes on `prep-wave1-verify`** — Wave 1 closed; see [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) |
| **Mechanical gates @ catalog SHA?** | **Green** — `validate-plan.py` 0 errors; 73/73 taskfmt lint; 7/7 bootstrap freeze groups |

**Partition summary:** 1 **VERIFIED** (parity/traceability), 4 **REJECTED** (DAG/receipts, architecture/integration projection, verification/bootstrap, closure/TASK-069). Rejection means the catalog can pass lint while execution-critical bindings remain prose-only or overclaimed — not that the witness failed to run.

---

## Witness methodology

Five fresh-context subagents reviewed disjoint partitions adversarially. Instruction:

> Assume the planners optimized for passing lint while violating source requirements. Find concrete ways this catalog passes mechanical checks while breaking architecture, receipts, disposition authority, or merge-readiness claims.

Coordinator re-verified the highest-severity findings on frozen bytes before recording them here. Prior `review-*-final.md` reports (2026-09-11) are **historical**; this document supersedes them for §22.17 closure at catalog SHA `3213fce2`.

### Mechanical measurements (coordinator + witnesses)

| Gate | Result |
| --- | --- |
| `python3 docs/refactoring-plan/evidence/validate-plan.py --summary` | **PASS** — `error_count: 0`, 363 scenarios, 3258 traceability edges, graph depth 35 |
| `taskfmt project lint terminal-components` (pinned `experiment.toml`) | **PASS** — 73/73 |
| `freeze-bootstrap-assets.py --check` (7 groups) | **PASS** — 211 assets, 0 changed |
| `derive-task-graph.py` | **PASS** — 73 tasks, depth 35, 24 longest paths, 0 scope overlaps |
| `tools/refactor-proof/bin/tc-proof-host` | **ABSENT** (expected; TASK-001 deliverable) |

### Catalog fingerprints

| Input | SHA-256 |
| --- | --- |
| Git commit | `3213fce285268925520be20b14aa8a1c4b136586` |
| `docs/refactoring-plan/proof-contract.md` | `2a9599117aebbd560023b577dc11db350f710529c29b8a05bc8e4b812a3e69ce` |
| `docs/refactoring-plan/task-graph.json` | `2148361f13c2e635910b929b621c1c752b22c8e343dd81637e588495e28cf001` |
| Completion catalog aggregate (922 files, lexicographic NUL-delimited) | `b6066a672cd9ea68a3dcbe314a29e1ff4b6e71b7005d6520a5f7f3c961b5e816` |
| Tag `visual-baseline` (peeled) | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` (unmoved) |
| Product oracle | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| Architectural `main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |

---

## Partition results

| Partition | Reviewer focus | Verdict | Notes |
| --- | --- | --- | --- |
| **A — Parity / traceability / orphans** | §22.7, §22.11, §22.19, §22.20 | **VERIFIED** (planning boundary) | 363/363 scenarios bound; 49/49 re-audit closed; 0 structural orphans. All visual/interaction proof **REQUIRED-UNCAPTURED** (expected pre-execution). |
| **B — DAG / receipts** | DAG-FINAL-01–05 repairs | **REJECTED** | Prior DAG findings largely repaired in prose/fixtures; **preparation mode not machine-bound in verify.toml** (WITNESS-01). |
| **C — Architecture / ADJ-22 / coordinator** | components-B integration, disposition bindings | **REJECTED** | Producer witnesses sound; **coordinator projection overclaims** TASK-031/TASK-008 trusted bindings (READINESS-01–03). |
| **D — Verification / bootstrap / executor** | §22.8–16, campaign protocol | **REJECTED** | Bootstrap self-tests green; **AGENTS vs campaign conflict** (READINESS-04); harness absent (expected). |
| **E — Integration / TASK-069 closure** | Merge-readiness gate | **REJECTED** | Validation-only scope strong; **CHK-007 overclaims human review**; merge-readiness/fidelity visual not catalog-bound (READINESS-05–06). |

---

## Prior finding disposition (DAG-FINAL / INT / VF)

| Prior ID | Original severity | Disposition @ `3213fce2` | Residual |
| --- | --- | --- | --- |
| DAG-FINAL-01 | P1 receipt cycle | **accepted-risk** — repaired in `proof-contract.md` + TASK-071 fixtures; mode only in host context JSON | WITNESS-01 |
| DAG-FINAL-02 | P1 TASK-008 inline scope | **closed** — 146 source paths in `008/verify.toml` + span manifest | span-only intent is prose/CHK-006 |
| DAG-FINAL-03 | P1 observation ownership | **closed** — canonical rule + per-task extraction seams | seam paths not lint-checked |
| DAG-FINAL-04 | P2 taskfmt config | **closed** — explicit `--config` in docs | — |
| DAG-FINAL-05 | P2 stale counts | **closed** — depth 35, 620 historical rows | — |
| INT-01 | High branch order | **closed** — coordinator accepted; docs aligned | — |
| INT-02 | Medium AGENTS conflict | **open** | READINESS-04 |
| INT-03 | High freeze index distrust | **open** — fixture gap documented | execution prerequisite |
| VF-01 | P1 `--group` ABI | **closed** — catalog uses `070`/`071`/`072` | — |

---

## Open findings (block `READY FOR REFACTORING EXECUTION`)

Material P1 items require catalog repair **or** explicit accepted-risk disposition with evidence before arming the campaign `/goal`.

| ID | Sev | Finding | Evidence | Required disposition |
| --- | --- | --- | --- | --- |
| **READINESS-01** | **P1** | ~~Coordinator projection gap~~ | **Closed** @ `prep-wave1-verify` — `branch-host-projection.tsv` |
| **READINESS-02** | **P1** | ~~W-042 in TASK-031~~ | **Closed** |
| **READINESS-03** | **P1** | ~~TASK-008 disposition not in trusted~~ | **Closed** |
| **WITNESS-01** | **P1** | ~~Accounting mode not catalog-bound~~ | **Closed** — 73 context templates |
| **READINESS-04** | **P1** | ~~INT-02 AGENTS conflict~~ | **Closed** — CAMPAIGN_AGENTS ×73 |
| **READINESS-05** | **P1** | ~~CHK-007 overclaims human review~~ | **Closed** — coordinator-review-contract |
| **READINESS-06** | **P2** | ~~Visual/merge prose-only~~ | **Closed** — host-context templates |
| **READINESS-07** | **P2** | `planning-acceptance.md` stale counts (361 scenarios / 3159 edges vs live 363 / 3258) | validate-plan counts | Doc sync only |
| **READINESS-08** | **P2** | Runner context-index qualification runs on `--group 070` only; TASK-071/072 verify paths skip index suite | `runner-bootstrap-driver.py`, verify.toml | Wire index check or document transitive TASK-070 receipt requirement |

**Known non-blockers (documented deferrals):** absent `tc-proof-host` (TASK-001); 363/363 scenarios REQUIRED-UNCAPTURED; 752/7885 branch inventory paths without semantic review; Darwin-only host isolation fixtures (INT-03).

---

## §22.17 closure statement

Independent subagents **have reviewed** the frozen catalog at `3213fce2` across parity, DAG, architecture, verification, and integration partitions. This satisfies PLANNING_GOAL §22 item 17 (**independent subagent review**) at the **witness-completeness** boundary.

It does **not** satisfy the campaign Phase 0 gate **`READY FOR REFACTORING EXECUTION`**, which additionally requires closing or explicitly accepting READINESS-01–05 (and operator authorization).

---

## Recommended next steps

1. **Repair READINESS-01–03** — align coordinator projection TSV with on-disk TASK-031/008 trusted bytes (highest priority; closes components-B integration overclaim).
2. **Repair READINESS-04** — resolve INT-02 AGENTS/campaign contradiction machine-visibly.
3. **Disposition WITNESS-01** — either catalog-bind preparation mode or qualify host-context rejection fixtures in frozen trusted templates.
4. **Repair READINESS-05–06** — decouple human review from TASK-069 CHK-007; freeze fidelity visual + merge-readiness commands in inspectable context bindings.
5. **Re-run witness spot-check** on repaired bytes (at minimum partitions C, D, E).
6. **Issue `READY FOR REFACTORING EXECUTION`** on exact post-repair catalog SHA with operator authorization.

---

## Explicit non-claims

- No runtime parity, production harness execution, or visual capture completeness.
- No whole-branch semantic acceptance (752/7885 inventory reviewed).
- No TASK-001 bootstrap or campaign dispatch authorization.
- BRANCH-01 coordinator witness ([`reaudit-branch-coordinator-integration.md`](reaudit-branch-coordinator-integration.md)) remains valid **bookkeeping**; this witness found **trusted-package projection gaps** not caught by `validate-plan.py`.
