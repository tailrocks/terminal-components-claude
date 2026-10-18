---
schema: task/v5
id: TASK-017
title: "Close exact parity and negative mutation gates"
kind: refactor
---

# TASK-017 — Close exact parity and negative mutation gates

## Goal

All applicable visual and interaction states pass exact comparison, and deliberate mutations are rejected by independent verification.

## Context

Use the verification contracts to exercise only applicable state axes across showcase, tablepro, jackin-preview, and holla: focus, hover, capture, press timing, editing, disabled/read-only, scroll, overlays, resize, capabilities, motion, and degenerate geometry.

Dependencies: TASK-016. Canonical contracts: [verification/README.md](../../../docs/verification/README.md), [verification/visual-parity.md](../../../docs/verification/visual-parity.md), [verification/interaction-parity.md](../../../docs/verification/interaction-parity.md), [verification/conformance.md](../../../docs/verification/conformance.md), [design/visual-contract.md](../../../docs/design/visual-contract.md), [design/interaction-contract.md](../../../docs/design/interaction-contract.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Complete snapshot/trace parity, exact state coverage, negative mutation gates, oracle separation, and protected-path checks.

Out of scope:

- Accepting changed snapshots, changing application behavior, weakening comparisons, or inventing meaningless component states.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** The frozen applications, snapshots, and visual-baseline tests remain untouched; their exact output is the oracle.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — Every applicable required state compares exactly on dimensions, symbols, cells, styles, cursor, focus, capture, keys, drafts, commits, and actions.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — A deliberate symbol, color, timing, focus, selected-key, or typed-action mutation causes verification failure.
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
- **Check:** `CHK-003`

### AC-004 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-004`

## Fixed decisions

- **D-001:** Only `ExistingOracle` and approved extracted evidence can define expected output; candidate output can never bless itself.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is enforced by narrow writable library/conformance paths and forbidden application/oracle paths.

## Checklist

<!-- checklist:start -->
- [x] **1** Prepare.
    - [x] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [x] **2** Implement.
    - [x] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [x] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [x] **2.3** Preserve the frozen applications and keep product work out of scope. (`R-003`, `R-004`, `AC-003`, `CHK-003`)
- [x] **3** Verify.
    - [x] **3.1** Run the one completion gate. (`R-005`, `AC-004`, `CHK-004`)
<!-- checklist:end -->
