---
schema: task/v5
id: TASK-023
title: "Restore concrete dialogs menus and submenu behavior"
kind: bugfix
---

# TASK-023 — Restore concrete dialogs menus and submenu behavior

## Goal

Dialogs and menus preserve measured facts, submenus and exact dismissal.

## Context

This package describes future implementation, not work authorized by the current planning goal. UI authority is immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Existing source/tests establish preservation obligations, not oracle parity. Required predecessors: `TASK-017`, `TASK-019`, `TASK-008`, `TASK-073`. Receipt acceptance and integrated source ancestry are both mandatory.

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

- `crates/tui/src/components/dialog.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/src/components/menu.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/completion_023.rs` solely for the named task-specific regression and mutation witnesses.

Out of scope:

- Application domain repairs, unrelated component families, alternative runtime/editor/collection engines, baseline blessing, provider execution, branch promotion and publication.
- Changing accepted proof tools, protected adapters, historical archives, task packages, expected artifacts, fixture membership, stage dispositions or another task's source scope.

## Requirements

- **R-001 (MUST):** Use content-measured open Dialog body for prompt/confirm/info/error/destructive acknowledgement and facts; current draft controls eligibility. Menu/ContextMenu/MenuBar retain title/dropdown/submenu bounds, enabled Move cursor changes, chord rendering, typed actions, dynamic bindings and bounded reverse wrap for three/five-item and disabled-menu boundaries in W-023-07. All source-qualified historical clauses assigned to this task and listed in trusted/obligations.md are binding with their recorded accepted/superseded/deferred dispositions. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-002 (MUST):** Compose established runtime layers and Form; no duplicate trap, backdrop, input dispatch or domain execution. Dialog body returns bare R and runs once at empty clip; callbacks cannot escape. Menu action and key route resolve the same effective owner/action metadata. Preserve the architectural and CP-COMMON clauses in trusted/obligations.md, using the same production owner for pixels and interaction. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-003 (MUST):** Probe nested Esc/outside/secondary/paste, popup over editor, disabled menu item and sibling submenu transitions; non-Move phases never masquerade as hover. Resize and close/reopen restore correct focus, not vanished opener. Actual facts/draft mutation must change next production frame and response. Execute every required historical/oracle test identity under the immutable stage map; compatible and previously closed cases stay green, all future-owned failures remain recorded and no case is silently omitted.
- **R-004 (MUST NOT):** Do not edit expected frames, oracle or comparison rules, accepted harness/adapter/test-disposition inputs, task protocols, required-set membership, capability/slot allowances or receipts. Do not duplicate generic widget engines, substitute test-only/inert painters, fabricate semantics, weaken checks, bless candidate output, or perform domain/provider operations.
- **R-005 (MUST):** The full host-controlled canonical gate passes on the frozen exact tree, with protected-input integrity, actual prerequisite ancestry and complete check evidence.

## Acceptance criteria

### AC-001 — Owned behavior matches exact oracle
```gherkin
Given the sealed dialog and menu-context-menubar oracle cases and the exact task history clauses
When the production behaviors and boundary states in R-001 are compared
Then every required cell cursor action and semantic observation matches its authoritative contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Architectural ownership holds
```gherkin
Given the live dialog and menu-context-menubar implementation and independent ownership witnesses
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
Given the frozen direct-lane dialog and menu-context-menubar actions and checkpoints
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

- **D-BRANCH-SOURCE:** Menu reverse navigation uses one shared bounded cyclic-index operation for context menus, open MenuBar switching and closed MenuBar previous navigation. Previous from index zero in an enabled three-item collection reaches index two; unsigned wrapping modulo is not equivalent to signed Euclidean cycling. W-023-07 freezes three/five-item boundaries and disabled scanning without changing accepted forward navigation, typed action remapping or runtime layer ownership. Empty/all-disabled paths terminate without selecting or activating an unavailable item; each mode's existing source-qualified close/open response remains distinct.

- **D-001:** Compose established runtime layers and Form; no duplicate trap, backdrop, input dispatch or domain execution. Dialog body returns bare R and runs once at empty clip; callbacks cannot escape. Menu action and key route resolve the same effective owner/action metadata.
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
    - [ ] **2.1** Reproduce the explicit oracle boundaries and add completion_023 regression witnesses. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve borrowed ownership, attributed paint and the task-specific architecture contract. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Capture all frozen direct production trajectories. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.4** Capture complete full-application PTY diagnostics without claiming future-owned parity. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.5** Execute exact regression identities and every stated negative witness. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify the frozen candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
