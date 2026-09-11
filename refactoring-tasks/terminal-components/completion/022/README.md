---
schema: task/v5
id: TASK-022
title: "Restore generic Grid cell and table behavior"
kind: bugfix
---

# TASK-022 — Restore generic Grid cell and table behavior

## Goal

Generic keyed Grid restores table and current-cell completed-click behavior.

## Context

This package describes future implementation, not work authorized by the current planning goal. UI authority is immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Existing source/tests establish preservation obligations, not oracle parity. Required predecessors: `TASK-016`, `TASK-015`, `TASK-014`, `TASK-008`, `TASK-073`. Receipt acceptance and integrated source ancestry are both mandatory.

Read before editing:

- `/task/trusted/source-witnesses.md` in full; its finite source-state, actual-paint and real-mutant cases are normative R-001/R-002/R-003 proof obligations, not optional examples.
- `/task/trusted/obligations.md` in full, including every assigned historical clause.
- `/task/trusted/source-obligations.tsv` in full; its exact source-qualified clauses, dispositions and task/check mappings are normative, not optional background.
- `/work/docs/refactoring-plan/components.md`, `architecture.md`, `architecture-adjudication.md` and `proof-contract.md`.
- Every pinned oracle/main source and existing test named in the trusted family/historical rows.

## Preconditions

- **P-001:** The protected host has accepted TASK-001/070/071/072 products, sealed complete baseline TASK-006 and exact inventory/dispositions TASK-007/008; every listed predecessor is accepted and integrated into the recorded parent.
- **P-002:** Source pins, taskfmt `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, reviewed tui-snap revision, image/toolchain/lock and immutable catalog/context identities resolve from protected receipts.
- **P-003:** The host froze exact owned scenario membership, source-qualified non-applicability, numeric traces and new-test obligations; the executor cannot choose expected output or expand write scope.

## Scope

In scope:

- `crates/tui/src/components/grid.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/completion_022.rs` solely for the named task-specific regression and mutation witnesses.

Out of scope:

- Application domain repairs, unrelated component families, alternative runtime/editor/collection engines, baseline blessing, provider execution, branch promotion and publication.
- Changing accepted proof tools, protected adapters, historical archives, task packages, expected artifacts, fixture membership, stage dispositions or another task's source scope.

## Requirements

- **R-001 (MUST):** Use the explicit generic-default/DataGrid/DataTable-row/DataTable-cell policy in D-BRANCH-SOURCE and W-022-09; specifically DataGrid different-cell click moves; current unranged globally noneditable Grid activates, while globally editable Grid enters the source begin_edit branch, which refuses read_only columns or pending-deleted rows without activation, while DataTable row and cell modes retain their distinct source outcomes. Press alone and release outside do not activate. Preserve every twenty-two TablePro capability through generic model/editor interfaces: row/cell modes, sorting requests, reference action, row-number selection, validation, fetch-more, scrolling and editor placement. All source-qualified historical clauses assigned to this task and listed in trusted/obligations.md are binding with their recorded accepted/superseded/deferred dispositions. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-002 (MUST):** Only GridEditor/update_editable mutates caller model; three-method GridModel remains readonly. App adapter owns SQL/NULL/default/pending/undo/comparator/permutation. Ragged None cells never call decoration/action/editor hooks; state exposes owned semantics not derived windows. Preserve the architectural and CP-COMMON clauses in trusted/obligations.md, using the same production owner for pixels and interaction. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-003 (MUST):** Test initially current/different/ranged/locked/readonly cells, inline/cycle/external/refused editing, active editor caret, validation preventing navigation and model reorder between publication and action. Exact header/gutter/density/width/scrollbar/fade cells survive both themes/all capabilities and resize; custom cell sentinels reach paint. Execute every required historical/oracle test identity under the immutable stage map; compatible and previously closed cases stay green, all future-owned failures remain recorded and no case is silently omitted.
- **R-004 (MUST NOT):** Do not edit expected frames, oracle or comparison rules, accepted harness/adapter/test-disposition inputs, task protocols, required-set membership, capability/slot allowances or receipts. Do not duplicate generic widget engines, substitute test-only/inert painters, fabricate semantics, weaken checks, bless candidate output, or perform domain/provider operations.
- **R-005 (MUST):** The full host-controlled canonical gate passes on the frozen exact tree, with protected-input integrity, actual prerequisite ancestry and complete check evidence.

## Acceptance criteria

### AC-001 — Owned behavior matches exact oracle
```gherkin
Given the sealed grid and table oracle cases and the exact task history clauses
When the production behaviors and boundary states in R-001 are compared
Then every required cell cursor action and semantic observation matches its authoritative contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Architectural ownership holds
```gherkin
Given the live grid and table implementation and independent ownership witnesses
When the R-002 borrowed state routing styling and production-path boundaries are exercised
Then each boundary holds and every specified violating witness is rejected
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Complete regressions and negative cases hold
```gherkin
Given the frozen required test identities and the task-specific negative cases in R-003
When every required target and profile executes with complete outcome accounting
Then each positive and negative case has its required outcome and no closed case regresses
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Verification authority stays immutable
```gherkin
Given the host-owned catalog source pins expected artifacts and accepted prerequisite receipts
When exact identity ancestry scope and trust boundaries are checked
Then no forbidden write or candidate-controlled acceptance input is admitted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Direct production trajectories are complete
```gherkin
Given the frozen direct-lane grid and table actions and checkpoints
When the real production update and draw paths run on the frozen candidate
Then all required frames cursors semantics and ordered checkpoints are captured without substitution
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — Complete executable PTY diagnostic evidence
```gherkin
Given the frozen full-application PTY diagnostic corpus and its immutable correction-owner stage map
When exact input bytes resize events and clocks drive the production binaries
Then every required checkpoint and actual outcome is recorded without treating future-owned application mismatches as component parity
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-BRANCH-SOURCE:** Retain accepted generic Grid behavior by default and add one finite source-qualified gesture policy consumed by the same shared keyed update/editor path: modern default, DataGrid, DataTable row mode and DataTable cell mode. DataGrid different-cell completed click moves; current unranged globally noneditable Grid emits Activated, while globally editable Grid invokes begin_edit: column read_only or pending-deleted row returns Consumed with no action/edit/mutation, otherwise the source inline/cycle/external result applies. DataTable row mode completed click selects and activates the clicked source row, including a different row. DataTable cell mode different-cell click moves; a current editable column begins editing, but a current noneditable column produces no activation. Existing editor-click placement and failed commit behavior remain source-qualified. W-022-01 applies specifically to DataGrid; W-022-09 proves the four policies without application-side suppression or synthesized action. Freeze gesture-start keyed identity across Press/Click/DoubleClick, reorder and release-outside; do not let press-driven cursor movement manufacture a same-current-cell click. Preserve W-022-08 partial-column locate guards and W-022-07 accepted readonly mutation guards; this policy neither restores F04 unchecked writes nor invents app-level viewers.

- **D-GESTURE-AUTHORITY:** Preserve accepted modern-default Grid gestures and historical gesture metadata/keyed identity. Only the explicit DataGrid source policy supersedes the historical single-Moved/double-Activated product description: different-cell completed click moves; current unranged globally noneditable Grid activates; globally editable Grid calls begin_edit, which refuses a read_only column or pending-deleted row with Consumed/no action/no edit/no mutation. Eligible cells retain source inline/cycle/external outcomes. DataTable row/cell policies follow D-BRANCH-SOURCE/W-022-09 independently. Deferred F04 is neither a parity gate nor permission to restore unchecked mutation. Preserve accepted main mutation guards; application compatibility requires an actual source-qualified reachable oracle trajectory.
- **D-001:** Only GridEditor/update_editable mutates caller model; three-method GridModel remains readonly. App adapter owns SQL/NULL/default/pending/undo/comparator/permutation. Ragged None cells never call decoration/action/editor hooks; state exposes owned semantics not derived windows.
- **D-002:** Preserve accepted current APIs; historical proposed or rejected alternatives in the trusted appendix are not new implementation authority.
- **D-003:** TASK-073 establishes observation/registry infrastructure; TASK-031 proves complete repaired-family equality. Do not weaken inherited attribution or registry checks to accommodate an incomplete family.
- **D-004:** Missing or impossible required behavior is a failed obligation, not a skip. Any necessary change outside this narrow scope returns to its owning prerequisite and requires renewed acceptance.
- **D-005:** Future `/proof/bin/tc-proof` operations are installed only from accepted producer receipts: TASK-001 host/comparator core, TASK-070 preflight/required/oracle/capture/close, TASK-071 account-tests and TASK-072 architecture. No candidate-local substitute or recursive taskfmt gate is permitted.
- **D-006:** Oracle comparison CHK-004 covers this task's frozen component-owned direct fixtures and already closed required cases. Full production application PTY frames remain owned by their app correction/closure tasks; CHK-003 captures their complete diagnostic evidence and CHK-005 records every actual mismatch under the immutable stage map. Capture completeness is not parity. No app frame is cropped, masked, rewritten or treated as an empty passing set, and this task cannot depend on an unrepaired application's full-frame equality. Component-sensitive PTY lifecycle/behavior obligations use source-qualified applicability and their separately owned architecture/behavior probes; final app closure compares all complete PTY frames.

## Candidate observation ownership

R-001 and R-002 explicitly permit extraction-only observation seams in this task's already writable production source or completion-test files. They are untrusted candidate code, built with the frozen candidate and real production handlers/renderers; they are not accepted oracle adapters, judges, schema authors or receipt producers. TASK-002–006 own the protected observation schema and logical identity mapping; TASK-070 owns independently qualified source/binary/action binding and wrong-state, constant-state, omitted-field, wrong-source and test-only-substitution rejection. Candidate seams may serialize actual focus/edit/selection/target/overlay/domain state, but may not replace input dispatch/rendering, synthesize expected state, branch on a test-only product path or alter protected mappings. The existing prohibition on changing observation adapters means protected reference adapters and runner policy; it does not forbid these explicitly scoped untrusted extraction seams. A seam needing another task's source path remains outside scope and must be assigned before dispatch.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Bind exact authority and scope.
    - [ ] **1.1** Validate protected receipts, parent ancestry and task inputs. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Deliver the owned component outcome.
    - [ ] **2.1** Reproduce the explicit oracle boundaries and add completion_022 regression witnesses. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve borrowed ownership, attributed paint and the task-specific architecture contract. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Capture all frozen direct production trajectories. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.4** Capture complete full-application PTY diagnostics without claiming future-owned parity. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.5** Execute exact regression identities and every stated negative witness. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify the frozen candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
