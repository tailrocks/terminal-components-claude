---
schema: task/v5
id: TASK-062
title: "Restore TablePro mounted safety and dirty-state overlays"
kind: bugfix
---

# TASK-062 — Restore TablePro mounted safety and dirty-state overlays

## Goal

Restore mounted safety/save/discard/dirty-state overlays, six safety modes and exact facts/token gates, pending SQL acknowledgement, nested select capture, paste/key no-leak behavior and current-target execution ownership.

## Context

The immutable UX oracle is holla-fable-2026-09-10, commit 02f5294bfdbf38004cc49130d0aff1d01f31434c; accepted architecture starts at main 7b27732a8c3c131760ec3438f641cb3c11343a42. This package owns T-SAFETY, not an application rewrite. The architectural enabling condition is that copied painting, surface labels and real input/model ownership can diverge. Restore the production owner instead of adding another visual facade.

Read before editing: /task/trusted/app-flow-contribution-contract.md, /task/trusted/app-flow-contributions.tsv, /task/trusted/app-flow-frame-contributions.tsv, /task/trusted/app-flow-stage-audit.tsv, /task/trusted/obligations.md, /task/trusted/source-obligations.tsv, /task/verify.toml, docs/refactoring-plan/tablepro.md, docs/refactoring-plan/tablepro-scenarios.tsv, docs/refactoring-plan/architecture.md, docs/refactoring-plan/architecture-adjudication.md, docs/refactoring-plan/proof-contract.md, and the source-qualified files/tests listed in the trusted obligations. The host-provided immutable catalog and sealed inputs govern acceptance; mutable repository copies are reference material only.

## Preconditions

- **P-001:** TASK-061, TASK-023 have accepted host receipts and their actual commits are ancestors of the integrated parent; the host has reconstructed a fresh exact scope base containing their outputs.
- **P-002:** TASK-005 oracle captures, TASK-006 complete sealed baseline, exact test identities/dispositions and checkpoint stage map are accepted. /proof/bin/tc-proof and /run/tc-proof/context-index.json and its immutable per-check contexts are protected host inputs, not executables or success files supplied by this task.
- **P-003:** The host has frozen this contract, trusted obligations, profiles, adapters, required membership and tool pins. All precondition checks pass before source editing; missing or contradictory scope/dependencies require NEEDS_REPLAN rather than a waiver.

- **P-004:** All exact flow contributions and any source-derived direct seed have been independently expanded, replay-qualified and sealed by TASK-005. A seed is not a partial parent replay or PTY reachability proof.

## Scope

In scope:

- Restore mounted safety/save/discard/dirty-state overlays, six safety modes and exact facts/token gates, pending SQL acknowledgement, nested select capture, paste/key no-leak behavior and current-target execution ownership.
- Change only `apps/tablepro/src` and `apps/tablepro/tests` for these observable outcomes and their non-conflicting direct tests.
- Compare all complete primary scenarios TP-058, TP-059, TP-060, TP-061, TP-062, TP-064; preserve every previously closed checkpoint and scenario.

Out of scope:

- Shared library/runtime repairs, other application flows and unrelated source cleanup.
- Real provider, credential, database, daemon, clipboard or export effects; all domain operations remain the oracle's simulations.
- Oracle capture/sealing, comparator/loader or fixture-policy changes, historical baseline rewriting, commits to main, publication and branch promotion.

## Requirements

- **R-001 (MUST):** Pass every exact whole-frame contribution assigned in trusted/app-flow-frame-contributions.tsv in addition to complete primary scenarios. Satisfy every accepted source-qualified clause assigned to this task in trusted/source-obligations.tsv and reproduce all complete primary scenarios TP-058, TP-059, TP-060, TP-061, TP-062, TP-064, with exact full cells, cursor, dimensions, ordered actions and semantic state/effects in each declared lane. Restore mounted safety/save/discard/dirty-state overlays, six safety modes and exact facts/token gates, pending SQL acknowledgement, nested select capture, paste/key no-leak behavior and current-target execution ownership.
- **R-002 (MUST):** Pass every exact semantic contribution in trusted/app-flow-contributions.tsv, independently of terminal-cell equality. Use shared runtime Layer/Dialog/Form/Select/Button and retain TablePro stable target identities, generations, per-tab drafts and captured destructive scope. Keep risk classifier, safety policy and SQL execution application-owned.
- **R-003 (MUST):** Preserve compatible existing tests and every prerequisite/previously closed checkpoint. Execute the complete required inventory without fail-fast omissions; accept only immutable, explicitly future-owned unresolved identities, never missing execution or reopened passes.
- **R-004 (MUST NOT):** Modify oracle/trust assets, frozen baseline or required-set membership, test dispositions, tool/profile/font pins or comparison rules; bless output, weaken tests, introduce candidate-selected coordinates/normalization, or retain rejected legacy rendering/input paths for the owned reusable controls.
- **R-005 (MUST):** The completion gate succeeds for the exact frozen candidate tree and protected context with all seven check results bound to that tree.

## Acceptance criteria

### AC-001 — Owned oracle behavior matches

```gherkin
Given the sealed oracle traces complete primary scenarios and exact flow-frame contribution IDs
When both declared capture lanes are compared for every required checkpoint
Then every owned cell cursor and semantic observation equals its corresponding oracle and no required result is absent
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Live reusable ownership is preserved

```gherkin
Given the candidate production routes and accepted public component boundaries
When ownership reachability and architecture probes exercise the owned controls
Then the same reusable owner paints and handles each control and every exact flow-state assertion and domain boundary holds
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Complete staged regression accounting holds

```gherkin
Given the frozen required test identities stage map and previously closed ledger
When every required target and scenario executes with no-fail-fast accounting
Then every owned and closed result passes and only explicitly unfinished future-owner failures remain
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Oracle and trust chain remain immutable

```gherkin
Given the protected task context oracle artifacts and prerequisite receipts
When source scope ancestry environment and protected-input integrity are checked
Then no oracle trust baseline membership or forbidden source path is changed or substituted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — direct capture is complete

```gherkin
Given the immutable direct lane membership and exact frozen event trace
When the frozen candidate executes production-handler events controlled time and allowed semantic observations
Then every declared direct checkpoint has valid tree-bound evidence and modeled-only states are not mislabeled as PTY reachability
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — PTY capture is complete

```gherkin
Given the immutable PTY lane membership and exact frozen event trace
When the frozen candidate executes actual executable terminal input output resize and teardown
Then every declared PTY checkpoint has valid tree-bound evidence and modeled-only states are not mislabeled as PTY reachability
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** Immutable oracle controls user-visible confirmation paths. Retain main identity/generation checks internally without adding new prompts or changing oracle close/delete semantics.
- **D-002:** Execute only a still-valid captured target once; invalidation follows the existing oracle cancellation/status behavior. Add internal stale-target proofs separately from UI traces.
- **D-003:** An overlay enum or Surface label without a draw/update/dismissal path is not implemented. Exact acknowledgement, disabled Execute, backdrop capture and nested Escape/focus restoration must be real.
- **D-004:** Use only flat numeric oracle traces. Selector resolution and expected artifacts were fixed by TASK-005; the candidate cannot rediscover focus, pointer coordinates or timing from its own layout. Direct and PTY compare against their respective oracle lanes; every declared variant and intermediate checkpoint is required.
- **D-005:** Trusted flow tables freeze separate nonempty whole-frame and semantic contributions for earlier slices. Their exact membership is mandatory new-behavior proof, not an empty preservation intersection. Only the last-producer primary closes an intact parent. Seeded fixtures are direct-only independent tests; the parent still executes every original action/assertion/frame in its declared lanes. No seed, crop, omitted prefix or semantic-only pass closes parent parity.
- **D-006:** Preserve the host's protected compatible/oracle-conflicting test dispositions. Additive in-scope tests may supplement but never replace or weaken protected acceptance tests. Validation failure is not permission to edit trusted fixtures.

## Candidate observation ownership

R-001 and R-002 explicitly permit extraction-only observation seams in this task's already writable production source or completion-test files. They are untrusted candidate code, built with the frozen candidate and real production handlers/renderers; they are not accepted oracle adapters, judges, schema authors or receipt producers. TASK-002–006 own the protected observation schema and logical identity mapping; TASK-070 owns independently qualified source/binary/action binding and wrong-state, constant-state, omitted-field, wrong-source and test-only-substitution rejection. Candidate seams may serialize actual focus/edit/selection/target/overlay/domain state, but may not replace input dispatch/rendering, synthesize expected state, branch on a test-only product path or alter protected mappings. The existing prohibition on changing observation adapters means protected reference adapters and runner policy; it does not forbid these explicitly scoped untrusted extraction seams. A seam needing another task's source path remains outside scope and must be assigned before dispatch.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Validate exact scope base, ancestry and immutable trust context. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Restore owned behavior.
    - [ ] **2.1** Deliver the bounded observable outcomes and all source-qualified clauses. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Prove live ownership, exact flow-state assertions and preserved domain boundaries. (`R-002`, `AC-002`, `CHK-006`)
- [ ] **3** Verify.
    - [ ] **3.1** Capture the complete declared direct lane. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **3.2** Capture the complete declared PTY lane. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **3.3** Compare every owned checkpoint exactly. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **3.4** Execute and reconcile the full required inventory and monotonic closed ledger. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **3.5** Run the completion gate on this exact candidate tree. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
