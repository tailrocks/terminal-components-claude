---
schema: task/v5
id: TASK-026
title: "Restore Diff review fallback projection and selection"
kind: bugfix
---

# TASK-026 — Restore Diff review fallback projection and selection

## Goal

Diff preserves requested review mode through narrow fallback and width recovery.

## Context

This package describes future implementation, not work authorized by the current planning goal. UI authority is immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Existing source/tests establish preservation obligations, not oracle parity. Required predecessors: `TASK-021`, `TASK-012`, `TASK-008`, `TASK-073`. Receipt acceptance and integrated source ancestry are both mandatory.

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

- `crates/tui/src/components/diff.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/completion_026.rs` solely for the named task-specific regression and mutation witnesses.

Out of scope:

- Application domain repairs, unrelated component families, alternative runtime/editor/collection engines, baseline blessing, provider execution, branch promotion and publication.
- Changing accepted proof tools, protected adapters, historical archives, task packages, expected artifacts, fixture membership, stage dispositions or another task's source scope.

## Requirements

- **R-001 (MUST):** Choose effective unified fallback when review is too narrow without overwriting requested Review. Reserve real scrollbar budget and recover exact two-column projection on growth. Preserve tab-indented context/change alignment, complete grapheme emphasis and source-preserving selection/copy. All source-qualified historical clauses assigned to this task and listed in trusted/obligations.md are binding with their recorded accepted/superseded/deferred dispositions. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-002 (MUST):** Use borrowed DiffSource projected through shared TextViewport with revision/mode/width cache keys; rebuild only in update. No parallel owned diff model or per-app fallback painter. Preserve the architectural and CP-COMMON clauses in trusted/obligations.md, using the same production owner for pixels and interaction. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-003 (MUST):** Replay oracle review_falls_back_when_narrow_and_restores_when_wide, review_budget_reserves_scrollbar_and_recovers_on_resize, review_emphasis_keeps_complete_graphemes_and_column_widths and tab_indented_review_aligns_context_changes_and_copied_text. Test fits/scroll/fade, hidden bar, narrow threshold−1/equal/+1 and repeated mode/resize cycles. Execute every required historical/oracle test identity under the immutable stage map; compatible and previously closed cases stay green, all future-owned failures remain recorded and no case is silently omitted.
- **R-004 (MUST NOT):** Do not edit expected frames, oracle or comparison rules, accepted harness/adapter/test-disposition inputs, task protocols, required-set membership, capability/slot allowances or receipts. Do not duplicate generic widget engines, substitute test-only/inert painters, fabricate semantics, weaken checks, bless candidate output, or perform domain/provider operations.
- **R-005 (MUST):** The full host-controlled canonical gate passes on the frozen exact tree, with protected-input integrity, actual prerequisite ancestry and complete check evidence.

## Acceptance criteria

### AC-001 — Owned behavior matches exact oracle
```gherkin
Given the sealed diff-view oracle cases and the exact task history clauses
When the production behaviors and boundary states in R-001 are compared
Then every required cell cursor action and semantic observation matches its authoritative contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Architectural ownership holds
```gherkin
Given the live diff-view implementation and independent ownership witnesses
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
Given the frozen direct-lane diff-view actions and checkpoints
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

- **D-BRANCH-SOURCE:** W-026-05 freezes asymmetric remove/add run boundaries and independently projected copy. Pair lookup must stay inside its own remove/add run; a missing side is blank, never text borrowed from the next Add, Context or hunk. Preserve the one shared diff projection and viewport painter, mode threshold and raw-viewport versus diff-copy distinction. TASK-008 source-disposes any conflicting starting assertion without discarding compatible ownership/cache assertions; candidates cannot rewrite the protected map or bless output.

- **D-COPY-SOURCE:** Diff selection copies the oracle's projected diff text supplied to TextViewport, not raw original hunk bytes. The oracle `src/widgets/diff.rs:736–776` expands the tab-indented selected hunk to exactly `"    let x=1"`. An ordinary TextViewport copies its own supplied source, including literal tabs and controls. Keep paired witnesses: raw viewport tab copy remains raw; Diff's named tab-indented review selection returns the four spaces. Do not apply the raw-hunk interpretation to Diff while claiming source preservation.

- **D-001:** Use borrowed DiffSource projected through shared TextViewport with revision/mode/width cache keys; rebuild only in update. No parallel owned diff model or per-app fallback painter.
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
    - [ ] **2.1** Reproduce the explicit oracle boundaries and add completion_026 regression witnesses. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve borrowed ownership, attributed paint and the task-specific architecture contract. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Capture all frozen direct production trajectories. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.4** Capture complete full-application PTY diagnostics without claiming future-owned parity. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.5** Execute exact regression identities and every stated negative witness. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify the frozen candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
