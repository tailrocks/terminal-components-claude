---
schema: task/v5
id: TASK-001
title: "Qualify the oracle, conformance package, and toolchain"
kind: refactor
---

# TASK-001 — Qualify the oracle, conformance package, and toolchain

## Goal

The in-place workspace has a pinned stable toolchain, a nonpublished conformance package, and an independent candidate-blind comparator for frozen output and observable interaction.

## Context

The annotated `visual-baseline` tag resolves to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Qualify its dimensions, cells, styles, cursor, focus, capture, actions, and interaction traces against an oracle that candidate code cannot rewrite.

Dependencies: none. Canonical contracts: [architecture/overview.md](../../../docs/architecture/overview.md), [implementation/quality-gates.md](../../../docs/implementation/quality-gates.md), [foundations/conformance.md](../../../docs/foundations/conformance.md), [verification/oracle-and-provenance.md](../../../docs/verification/oracle-and-provenance.md), [verification/visual-parity.md](../../../docs/verification/visual-parity.md), [verification/interaction-parity.md](../../../docs/verification/interaction-parity.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation` (created from the accepted Stage A commit).
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk`, Mise, and `cargo nextest`.
- **P-004:** Implementation starts from this repository's current root Cargo package; P0 establishes the in-repository workspace boundary without moving the applications or changing current package identity.

## Scope

In scope:

- Oracle qualification, exact comparator behavior, trusted reference inputs, and conformance-harness scaffolding.
- A repository-local `mise.toml` stable-toolchain pin and the nonpublished conformance package/workspace wiring, without moving the four applications or baseline files.
- A candidate-blind ExistingOracle fixture for TablePro pending-edit preview SQL, derived from the frozen implementation and covering the documented statement and key/default cases.

Out of scope:

- Component implementation, application changes, snapshot writes, baseline replacement, and any candidate-controlled expected output.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** The frozen applications, snapshots, and visual-baseline tests remain untouched; their exact output is the oracle.
- **R-004 (MUST NOT):** No product implementation, service integration, separate project/repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.
- **R-006 (MUST):** Pin a verified stable Rust toolchain through Mise for local and CI use; establish one public library package and one nonpublished conformance package in this repository workspace; keep conformance-only dependencies out of the production library graph.
- **R-007 (MUST):** Before candidate TablePro Grid integration exists, seal the exact ordered SQL preview statement vectors produced by the frozen baseline for update, insert, delete, and documented boundary cases. Record baseline provenance and keep the expected vectors outside candidate-controlled approval paths.

## Acceptance criteria

### AC-001 — A comparator rejects any dimension, symbol, wide-cell, style, cursor, focus, capture, or typed-action mismatch.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — A comparator detects a mismatch in candidate output without accepting candidate-generated expected data.
```gherkin
Given an applicable boundary, interaction, or reconciliation state
When the future implementation is exercised
Then the documented state, identity, geometry, and ownership invariants hold
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-002`

### AC-003 — Frozen references and scope remain protected
```gherkin
Given the immutable visual-baseline applications and protected paths
When the candidate tree is checked after implementation
Then application source, snapshots, baseline tests, and product integrations are unchanged
```

**Verification**

- **Type:** invariant
- **Covers:** `R-003, R-004`
- **Check:** `CHK-004`

### AC-004 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-006`

### AC-005 — The in-place package and toolchain boundaries hold
```gherkin
Given the existing repository and pinned toolchain contract
When the conformance workspace is inspected and exercised
Then Mise selects the declared stable toolchain and the nonpublished conformance package uses public library exports without adding conformance tools to production dependencies
```

**Verification**

- **Type:** invariant
- **Covers:** `R-006`
- **Check:** `CHK-005`

### AC-006 — TablePro preview SQL vectors are sealed from the frozen implementation
```gherkin
Given the TablePro pending-edit fixture states at the frozen baseline
When the oracle fixture is qualified before candidate Grid integration
Then the ordered UPDATE, INSERT, and DELETE statements and documented boundary outputs are fixed in the ExistingOracle lane with source provenance
```

**Verification**

- **Type:** invariant
- **Covers:** `R-007`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** Existing baseline outputs remain immutable `ExistingOracle` inputs; extracted evidence and future extensions stay separately labelled.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is enforced by narrow writable library/conformance paths and forbidden application/oracle paths.
- **D-004:** The conformance package and toolchain pin live in this repository; the discarded separate-repository source layout is not a destination.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [ ] **2** Implement.
    - [ ] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [ ] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [ ] **2.3** Preserve the frozen applications and keep product work out of scope. (`R-003`, `R-004`, `AC-003`, `CHK-004`)
    - [ ] **2.4** Establish the pinned toolchain and test-only workspace boundary. (`R-006`, `AC-005`, `CHK-005`)
    - [ ] **2.5** Seal the baseline TablePro preview SQL vector fixture before candidate integration. (`R-007`, `AC-006`, `CHK-003`)
- [ ] **3** Verify.
    - [ ] **3.1** Run the one completion gate. (`R-005`, `AC-004`, `CHK-006`)
<!-- checklist:end -->
