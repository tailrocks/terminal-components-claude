---
schema: task/v5
id: TASK-001
title: "Implement independently qualified comparator and verifier core"
kind: feature
---

# TASK-001 — Implement independently qualified comparator and verifier core

> Current disposition: RETIRED / NOT QUALIFIED. The former host verifier
> lifecycle was removed. This package remains for contract traceability and
> comparator maintenance; its fail-closed `verify.toml` cannot produce an
> accepted `qualified-harness` receipt.

## Goal

The historical goal was a separately installed tc-proof comparator and verifier
core passing independent comparator, subagent isolation, exact-tree
verification and integration qualification. Only the comparator remains
supported; no current qualification is claimed.

## Context

Visual regression gate: committed `snapshots/` grouped store (`../../../../docs/baseline/snapshots-v2.md` (task catalog: `refactoring-tasks/visual-validation.md`)). After product edits, `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` must match. Never write `snapshots/` or run `tuisnap accept`.

This is a later execution task retained as an immutable historical package. The
UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts
from `7b27732a8c3c131760ec3438f641cb3c11343a42`. No accepted producer product
is presently bound to this package. Dependency status alone never proves
integrated source ancestry or trusted product acceptance. Follow the
subagent-only host-local workflow in proof-contract.md; use standalone taskfmt
only for this package's lint evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [docs/refactoring-plan/proof-contract.md](../../../../docs/refactoring-plan/proof-contract.md).
- [docs/refactoring-plan/evidence/proof-comparator-protocol.md](../../../../docs/refactoring-plan/evidence/proof-comparator-protocol.md).
- [docs/refactoring-plan/campaign-executor-protocol.md](../../../../docs/refactoring-plan/campaign-executor-protocol.md).
- `trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `trusted/obligations.md` and `../../../../docs/refactoring-plan/proof-contract.md`.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.

## Scope

In scope:

- `tools/refactor-proof` for this task's stated product and directly necessary tests.
- Independently inspectable qualified-harness evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/source-obligations.tsv` together with the bounded outcome below. Implement only tc-proof compare and the thin verifier interfaces in proof-contract.md under tools/refactor-proof/. The comparator must pass all 72 independently authored vectors and 69 fresh positive recoveries. The verifier must pass every independently authored install/prepare/record/verify/seal-rejection/integrate case, including hostile workers and actual standalone taskfmt. A missing or unsupported isolation capability is a failing task, not an accepted limitation.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Keep project orchestration outside junie-tui and tui-snap. Use existing tuisnap schema-3 capture and exact comparison capabilities, pinned standalone taskfmt, protected verification authority, immutable file receipts and local integration compare-and-swap. Do not build a scheduler, monitor, receipt service or taskfmt schema extension. Candidate build/test code runs in separate workers without expected artifacts, credentials, network or trust-root access.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Qualify nonzero exit with printed DONE, omitted logs/checks, forged receipt, wrong producer, unintegrated prerequisite, mutable source/index/symlink/hard-link attacks, changed parent, refs/heads/main, missing expected files, incomplete test execution and temporary trust writes restored afterward. Every applicable rejection preserves independently inspected refs and trusted bytes and is followed by a fresh passing case.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never qualify either executable solely through its own tests or output claims. Never create a real terminal-components integration commit, change production code, seal a real oracle bundle, invoke task orchestration or promotion, alter the independent driver/vectors, or claim synthetic fixtures are product UX evidence.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by verifier and reviewer subagents.

These requirements and acceptance criteria remain traceability records, not a
completion claim. The retired gate in `verify.toml` deliberately blocks
qualification until a future task defines and independently accepts a
replacement verifier.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the qualified-harness producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed qualified-harness implementation
When its production call paths and exact ownership invariants in R-002 are checked
Then every declared boundary holds without a substituted renderer or self-approved verdict
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Reject omissions and regressions
```gherkin
Given the required source-qualified regression inventory and negative qualification inputs
When the complete R-003 execution runs with all outcomes recorded
Then every required case passes its specified outcome and no execution or result is silently omitted
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Keep authority immutable
```gherkin
Given the campaign-pinned task contract and protected source and verification inputs
When source identity ownership scope and receipt authority are validated
Then every R-004 prohibition holds and no candidate-selected artifact defines success
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

## Fixed decisions

- **D-001:** The UI oracle and architecture source have separate authority; never substitute main output for an expected frame.
- **D-002:** This retired package produces no accepted `qualified-harness` receipt. Only verifier and reviewer subagents can approve future evidence after independent checks; the coordinator cannot edit receipts or task metadata.
- **D-003:** Preserve the comparator contract and standalone taskfmt lint. Source/scenario runner operations are owned by TASK-070, accounting by TASK-071 and architecture verification by TASK-072. Those are distinct independently qualified prerequisites; do not claim this package establishes them.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Independent comparator checks may inspect the submitted executable; they cannot turn this retired package into an accepted product.
- **D-006:** No manual bootstrap exception exists for this retired package. A future replacement task must define its own fail-closed, subagent-reviewed evidence contract before any receipt can enter the protected ledger.

## Subagent execution

The implementer, verifier, and reviewer subagents own any future replacement
work. The coordinator assigns isolated worktrees and reviews evidence; this
package is not eligible for integration or acceptance.

All inspection is host-local. Use `$TASK_DIR` for this package and run only the
latest standalone taskfmt lint:

```text
taskfmt lint "$TASK_DIR"
```

Taskfmt is validation only. No containers, images, mounts, lifecycle commands,
or qualification runs are allowed for this package. The comparator check is a
maintenance smoke test; the remaining `verify.toml` checks fail closed by
design. Do not modify task metadata or protected oracle inputs.


## Checklist

<!-- checklist:start -->
- [x] **1** Retire the host lifecycle path and preserve standalone comparator functionality. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Define and independently accept a future replacement verifier task. (`R-001`, `AC-001`, `CHK-004`)
- [ ] **3** Remove the fail-closed gate only when replacement evidence is reviewed by verifier and reviewer subagents. (`R-002`, `AC-002`, `CHK-006`; `R-003`, `AC-003`, `CHK-005`; `R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
