---
schema: task/v5
id: TASK-034
title: "Restore Showcase customer grid and editable tables"
kind: refactor
---

# TASK-034 — Restore Showcase customer grid and editable tables

## Goal

Replace the five-metric/four-column impostor with the actual customer Grid model: 40→80→96 rows; eight typed columns; keyed selection, range/plain/TSV copy, widths and sorting; insert/duplicate/delete/undo/discard/refresh; pending edits and SQL dialog; four-tick commit, seats>500 error and successful retry. Restore Tasks/Checks tables and all editable-cell commit/cancel/validation/click behavior. Do not invent a live reference-column fixture absent from the oracle.

## Context

Customer Grid model/queue/SQL/row effects, Tables and Editable Tables use visible data as actual interaction model. The application report shows that a migrated file, a green self-baseline or a generic component call can coexist with wrong live behavior. This task proves actual public-component state transitions and oracle output, not nominal API use.

Read before editing:

- `/task/trusted/source-obligations.tsv`: exact historical clauses and their binding requirement, acceptance and check mappings.
- `/task/trusted/obligations.md`: exact scenario membership, source clauses and frozen-proof rules.
- The host-provided immutable `showcase.md` or `holla.md` report for this application's full expansion grammar, and `proof-contract.md` for proof isolation and stage accounting.
- The cited oracle files at their pinned source revision and the accepted parent versions of every writable production file.

## Preconditions

- **P-001:** Accepted receipts for `terminal-components/completion/033`, `terminal-components/completion/022` exist, their accepted source commits are ancestors of the integrated parent, and the whole sealed baseline/test-disposition products are present.
- **P-002:** The host has frozen this canonical package, exact task scope base and independent proof tools, materialized every owned scenario expansion and installed `/proof/bin/tc-proof`; missing products stop execution.
- **P-003:** Start from the host's integration worktree, never from the oracle checkout or directly on main. The host controls `/run/tc-proof/context-index.json` and its immutable per-check contexts and all trust inputs.

## Scope

In scope:

- Replace the five-metric/four-column impostor with the actual customer Grid model: 40→80→96 rows; eight typed columns; keyed selection, range/plain/TSV copy, widths and sorting; insert/duplicate/delete/undo/discard/refresh; pending edits and SQL dialog; four-tick commit, seats>500 error and successful retry. Restore Tasks/Checks tables and all editable-cell commit/cancel/validation/click behavior. Do not invent a live reference-column fixture absent from the oracle.
- Exact owned IDs: `APP:SC-BASE-tables`, `APP:SC-BASE-editable`, `APP:SC-BASE-grid`, `APP:SC-TABLE-SORT`, `APP:SC-EDITABLE-CELL`, `APP:SC-EDITABLE-MOUSE`, `APP:SC-GRID-NAV`, `APP:SC-GRID-EDIT`, `APP:SC-GRID-QUEUE`, `APP:SC-GRID-ACTIONS`.
- Changes only to the declared application source files and the new task-specific regression test; referenced shared components are accepted prerequisites.

Out of scope:

- Library/runtime repairs, unrelated application flows, dependency upgrades, provider integrations and changes to existing protected test authority.
- Oracle or baseline regeneration, comparator/adapter/required-set changes, blessing or weakening assertions, publication, merge to main and modification of task instructions.

## Requirements

The protected source-obligations.tsv is normative. Every mapped historical clause, disposition and proof requirement applies to the exact R/AC/check identity recorded in that file; the task's app-specific clauses supplement, rather than replace, those obligations.

- **R-001 (MUST):** Replace the five-metric/four-column impostor with the actual customer Grid model: 40→80→96 rows; eight typed columns; keyed selection, range/plain/TSV copy, widths and sorting; insert/duplicate/delete/undo/discard/refresh; pending edits and SQL dialog; four-tick commit, seats>500 error and successful retry. Restore Tasks/Checks tables and all editable-cell commit/cancel/validation/click behavior. Do not invent a live reference-column fixture absent from the oracle. Every exact row and checkpoint obligation in `trusted/obligations.md` must compare equal against the sealed oracle in its declared direct and PTY lanes; no text-only, initial-frame-only or retargeted-coordinate substitute is acceptable.
- **R-002 (MUST):** One generic Grid implementation supplies row and cell modes; the live model must be the visible customer data and actual dispatch model. Delete customer paint-over and invisible/live metric scaffolding; no application-local table editor or scrollbar clone.
- **R-003 (MUST):** All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.
- **R-004 (MUST NOT):** Alter the oracle, expected outputs, fixture/action membership, observation adapters, trust roots, test disposition/relocation authority, tool pins, host context or any forbidden path; run real provider effects; bless candidate output; hide failures; or special-case verifier fixtures.
- **R-005 (MUST):** The completion gate succeeds on the exact frozen candidate tree and accepted dependency ancestry.

## Acceptance criteria

### AC-001 — Owned observable journeys equal the oracle

```gherkin
Given the sealed oracle bundle and every exact scenario in the trusted obligations
When the frozen candidate executes the recorded direct and PTY transcripts without retargeting
Then every required checkpoint has equal complete cells cursor semantic state and effects
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Live public components retain accepted ownership

```gherkin
Given the actual production routes and component ownership probes for this task
When the trusted architecture checks inspect and exercise the frozen application
Then the task-specific public composition invariants hold without duplicate or inert controls
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Required execution and closed behavior remain preserved

```gherkin
Given the immutable required test identities stage map and accepted closed scenarios
When the trusted accounting check executes the complete required inventory
Then all owned and closed cases pass and every remaining result obeys the exact stage policy
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Proof authority and forbidden paths stay unchanged

```gherkin
Given the host-owned catalog baseline tools scope and prerequisite receipts
When the trusted preflight validates their identities and access boundaries
Then every required identity is authentic and no candidate-controlled proof authority is accepted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Complete direct capture is produced

```gherkin
Given the frozen candidate and accepted direct-lane action expansion
When the isolated direct runner replays every owned direct checkpoint
Then complete validated frame and semantic artifacts exist for the exact required identities
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — Complete executable capture is produced

```gherkin
Given the frozen candidate and accepted PTY-lane transcripts and applicability records
When the isolated executable runner replays every owned PTY journey
Then complete terminal frames process outcomes and required restoration evidence exist

```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** One generic Grid implementation supplies row and cell modes; the live model must be the visible customer data and actual dispatch model. Delete customer paint-over and invisible/live metric scaffolding; no application-local table editor or scrollbar clone.
- **D-002:** The source-level scenario rows are fixed below; their complete finite numeric expansions come only from accepted baseline receipts. Direct-only time/private-state probes retain explicit lane applicability; missing PTY evidence requires a sealed source-proven non-applicability record, not an executor waiver.
- **D-003:** All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.
- **D-004:** Existing oracle-conflicting assertions can change only through the already accepted TASK-008 disposition/replacement overlay. Preserve compatible assertions and historical archive bytes. A missing disposition or inadequate writable scope means NEEDS_REPLAN, not a local exception.
- **D-005:** The host freezes, independently verifies and integrates the exact tested tree through a compare-and-swap local ref update. Never use taskfmt run/promote or merge main. Full terminal-cell equality supplements semantic effect/cursor/geometry proof; no candidate coordinate lookup or masking.
- **D-006:** Application tasks are serialized within their app because shell/domain composition files overlap. Do not use a sibling's unaccepted working tree as a dependency.

## Candidate observation ownership

R-001 and R-002 explicitly permit extraction-only observation seams in this task's already writable production source or completion-test files. They are untrusted candidate code, built with the frozen candidate and real production handlers/renderers; they are not accepted oracle adapters, judges, schema authors or receipt producers. TASK-002–006 own the protected observation schema and logical identity mapping; TASK-070 owns independently qualified source/binary/action binding and wrong-state, constant-state, omitted-field, wrong-source and test-only-substitution rejection. Candidate seams may serialize actual focus/edit/selection/target/overlay/domain state, but may not replace input dispatch/rendering, synthesize expected state, branch on a test-only product path or alter protected mappings. The existing prohibition on changing observation adapters means protected reference adapters and runner policy; it does not forbid these explicitly scoped untrusted extraction seams. A seam needing another task's source path remains outside scope and must be assigned before dispatch.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Verify host receipts, fixed scope, immutable products and exact owned scenarios. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Restore the bounded application slice.
    - [ ] **2.1** Restore and capture every owned direct checkpoint and semantic effect. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.2** Replay every owned executable PTY journey and terminal outcome. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.3** Require exact independent frame, cursor, state and provenance equality. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.4** Prove live public ownership and absence of the named architectural bypasses. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.5** Execute and account for the full inventory without weakening closed behavior. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify.
    - [ ] **3.1** Pass the independent completion gate on the exact frozen tree. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
