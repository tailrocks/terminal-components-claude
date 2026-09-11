---
schema: task/v5
id: TASK-032
title: "Restore live Showcase shell and all 23 routes"
kind: refactor
---

# TASK-032 — Restore live Showcase shell and all 23 routes

## Goal

Restore the live 23-entry shell route table including Diff; cursor movement versus choose versus enter-content, navigation return without quit, keyboard-before-global editing, persisted page state, header Help/Inspector, status and focus restoration. Sidebar changes at terminal heights 31/32, widths 109/110; inspector at widths 99/100; minimum is 72×20. Flash clears at ≥140ms and status only after 4000ms. Only shell routing and geometry/state contribution proof is in scope, plus the explicitly approved minimal real public DiffView route; no permission to repair other initial page compositions is granted.

## Context

Live public shell/chrome/navigation serves 23 pages with exact sidebar thresholds, help/inspector/focus/persistence and no compatibility paint-over. The application report shows that a migrated file, a green self-baseline or a generic component call can coexist with wrong live behavior. This task proves actual public-component state transitions and oracle output, not nominal API use.

Read before editing:

- `/task/trusted/source-obligations.tsv`: exact historical clauses and their binding requirement, acceptance and check mappings.
- `/task/trusted/obligations.md`: exact scenario membership, source clauses and frozen-proof rules.
- The host-provided immutable `showcase.md` or `holla.md` report for this application's full expansion grammar, and `proof-contract.md` for proof isolation and stage accounting.
- The cited oracle files at their pinned source revision and the accepted parent versions of every writable production file.

## Preconditions

- **P-001:** Accepted receipts for `terminal-components/completion/020`, `terminal-components/completion/027`, `terminal-components/completion/028`, `terminal-components/completion/023`, `terminal-components/completion/030`, `terminal-components/completion/031` exist, their accepted source commits are ancestors of the integrated parent, and the whole sealed baseline/test-disposition products are present.
- **P-002:** The host has frozen this canonical package, exact task scope base and independent proof tools, materialized every owned scenario expansion and installed `/proof/bin/tc-proof`; missing products stop execution.
- **P-003:** Start from the host's integration worktree, never from the oracle checkout or directly on main. The host controls `/run/tc-proof/context-index.json` and its immutable per-check contexts and all trust inputs.

## Scope

In scope:

- Restore the live 23-entry shell route table including Diff; cursor movement versus choose versus enter-content, navigation return without quit, keyboard-before-global editing, persisted page state, header Help/Inspector, status and focus restoration. Sidebar changes at terminal heights 31/32, widths 109/110; inspector at widths 99/100; minimum is 72×20. Flash clears at ≥140ms and status only after 4000ms. Only shell-owned composition and the minimal public Diff route wiring are in scope; page bodies, initial page frames and page-specific actions retain TASK-033–038 owners.
- Required parent transcripts (not claims of whole-scenario ownership): `APP:SC-SHELL-NAV`, `APP:SC-SHELL-HITS`, `APP:SC-SHELL-HELP`, `APP:SC-SHELL-INSPECTOR`, `APP:SC-SHELL-MINIMUM`, `APP:SC-SHELL-STATUS`, `APP:SC-SHELL-PERSIST`.
- Additional parent transcripts: `APP:SC-SHELL-CLI-GRAMMAR` and `APP:SC-SHELL-CLI-COLOR-ENV`. Restore their source-compatible parser and explicit launch policy through the accepted TASK-009 ADJ-17 seam. Own only C08/C09 semantic contributions and the F02/F03 complete pre-frame process subsets; successful full-page/process output remains unmasked TASK-039-owned diagnostic evidence until page repairs close.
- Changes only to the declared application source files and the new task-specific regression test; referenced shared components are accepted prerequisites.

Out of scope:

- Library/runtime repairs, unrelated application flows, dependency upgrades, provider integrations and changes to existing protected test authority.
- Oracle or baseline regeneration, comparator/adapter/required-set changes, blessing or weakening assertions, publication, merge to main and modification of task instructions.

## Requirements

The protected source-obligations.tsv is normative. Every mapped historical clause, disposition and proof requirement applies to the exact R/AC/check identity recorded in that file; the task's app-specific clauses supplement, rather than replace, those obligations.

- **R-001 (MUST):** Restore the live 23-entry shell route table including Diff; cursor movement versus choose versus enter-content, navigation return without quit, keyboard-before-global editing, persisted page state, header Help/Inspector, status and focus restoration. Sidebar changes at terminal heights 31/32, widths 109/110; inspector at widths 99/100; minimum is 72×20. Flash clears at ≥140ms and status only after 4000ms. Only shell-owned composition and the minimal public Diff route wiring are in scope; page bodies, initial page frames and page-specific actions retain TASK-033–038 owners. This task closes only the exact shell contributions in `trusted/contributions.tsv`. The parent scenario transcripts still generate complete, unmasked frame/cursor/state artifacts and every frame mismatch remains an explicitly reported future-owned failure under the sealed stage map. A passing contribution never closes its parent scenario or claims any cell region equals the oracle. Complete parent-scenario equality is owned by TASK-039 after the page repair slices. The independently qualified architecture operation (CHK-006) proves the exact source-qualified shell assertions, and architecture requires every exact contribution observation/assertion to pass. No executor-selected fields, checkpoints, masks, selectors or assertion subsets are allowed.
- **R-002 (MUST):** Remove shell compatibility paint-over; Brand, NavList, StatusBar, HintBar and runtime layers must jointly own their painted geometry and real hit/focus state. Never introduce a second sidebar or footer renderer. Preserve Moment/advance_to; Tick does not advance time.
- **R-003 (MUST):** All owned shell contributions, complete owned process contracts, prerequisite and previously closed scenario/contribution identities must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.
- **R-004 (MUST NOT):** Alter the oracle, expected outputs, fixture/action membership, observation adapters, trust roots, test disposition/relocation authority, tool pins, host context or any forbidden path; run real provider effects; bless candidate output; hide failures; or special-case verifier fixtures.
- **R-005 (MUST):** The completion gate succeeds on the exact frozen candidate tree and accepted dependency ancestry.

## Acceptance criteria

### AC-001 — Exact shell contributions satisfy source-qualified invariants

```gherkin
Given the immutable shell contribution table and independently qualified source observations
When the architecture operation exercises every required shell contribution on the frozen candidate
Then every named source-qualified shell assertion passes without closing any parent frame scenario
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-006`

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

### AC-008 — Owned complete-frame and process subsets equal the oracle

```gherkin
Given the exact nonempty complete-frame checkpoint subset in the trusted frame contribution table
When the independent comparator compares every owned frame cursor and complete process record
Then each owned artifact equals the oracle and every broader parent mismatch remains an unresolved diagnostic
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

## Fixed decisions

- **D-008:** R-001/AC-001/CHK-006 requires exact CLI grammar/default/explicit-color contributions C08/C09; R-001/AC-008/CHK-004 compares only the whole pre-frame process records explicitly listed in F02/F03 plus the existing full-frame subset. Preserve thin main/public App and the single runtime driver. Existing generic run APIs retain Detect; the CLI alone passes its source-resolved color through accepted ADJ-17 Requested policy. Never mutate host environment, widen an authored poorer theme, duplicate a runtime or reinterpret an existing library caller's theme as an override. All23 successful launch frames and full terminal output still execute and retain actual future-owned mismatches; semantic route proof cannot certify initial page equality. TASK-039 owns full parent closure. Missing accepted TASK-009 seam or TASK-008 assertion disposition means NEEDS_REPLAN.

- **D-001:** Remove shell compatibility paint-over; Brand, NavList, StatusBar, HintBar and runtime layers must jointly own their painted geometry and real hit/focus state. Never introduce a second sidebar or footer renderer. Preserve Moment/advance_to; Tick does not advance time.
- **D-002:** This task closes only the exact shell contributions in `trusted/contributions.tsv`. The parent scenario transcripts still generate complete, unmasked frame/cursor/state artifacts and every frame mismatch remains an explicitly reported future-owned failure under the sealed stage map. A passing contribution never closes its parent scenario or claims any cell region equals the oracle. Complete parent-scenario equality is owned by TASK-039 after the page repair slices. The independent architecture check proves the exact source-qualified shell assertions, and architecture requires every exact contribution observation/assertion to pass. No executor-selected fields, checkpoints, masks, selectors or assertion subsets are allowed. The source-level parent scenario rows and nonempty whole-frame checkpoint subsets in trusted/frame-contributions.tsv are fixed below; their complete finite numeric expansions come only from accepted baseline receipts. Direct-only time/private-state probes retain explicit lane applicability; missing PTY evidence requires a sealed source-proven non-applicability record, not an executor waiver.
- **D-003:** All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.
- **D-004:** Existing oracle-conflicting assertions can change only through the already accepted TASK-008 disposition/replacement overlay. Preserve compatible assertions and historical archive bytes. A missing disposition or inadequate writable scope means NEEDS_REPLAN, not a local exception.
- **D-005:** The host freezes, independently verifies and integrates the exact tested tree through a compare-and-swap local ref update. Never use taskfmt run/promote or merge main. Full terminal-cell equality supplements semantic effect/cursor/geometry proof; no candidate coordinate lookup or masking.
- **D-007:** Wire the missing Diff route with a minimal real public DiffView page in pages/diff.rs, using domain fixture data and live public update/draw; no inert placeholder, old-widget copy or reference-only facade. TASK-036 owns the complete Diff mode/control/selection journeys. This exception grants no other initial-page repair.
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
    - [ ] **2.3** Require exact independent frame, cursor, state and provenance equality. (`R-001`, `AC-008`, `CHK-004`)
    - [ ] **2.4** Prove live public ownership and absence of the named architectural bypasses. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.5** Execute and account for the full inventory without weakening closed behavior. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify.
    - [ ] **3.1** Pass the independent completion gate on the exact frozen tree. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
