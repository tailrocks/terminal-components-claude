---
schema: task/v5
id: TASK-018
title: "Prove performance bounds and complete independent review"
kind: refactor
---

# TASK-018 — Prove performance bounds and complete independent review

## Goal

Termrock refactoring is ready for independent review with measured performance bounds, clean scope, and no unresolved canonical-contract gap.

## Context

The final closure package combines performance evidence with a fresh review of architecture, API, component coverage, visual parity, task scope, and protected applications. It is a release gate for the future implementation.

Dependencies: TASK-017. Canonical contracts: [implementation/plan.md](../../../docs/implementation/plan.md), [implementation/quality-gates.md](../../../docs/implementation/quality-gates.md), [verification/conformance.md](../../../docs/verification/conformance.md), [verification/visual-parity.md](../../../docs/verification/visual-parity.md), [verification/interaction-parity.md](../../../docs/verification/interaction-parity.md), [architecture/overview.md](../../../docs/architecture/overview.md), [api/public-api.md](../../../docs/api/public-api.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Measured library/conformance performance, evidence for the declared two-minute review-lane target, full documentation/task traceability review, and all four independent review packets required by `quality-gates.md`.

Out of scope:

- Product optimization, application redesign, broad benchmark claims unrelated to Termrock, and any source/output mutation.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** The frozen applications, snapshots, and visual-baseline tests remain untouched; their exact output is the oracle.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.
- **R-006 (MUST):** Record cold/warm compile times, test-shard durations, full gate duration, and frame measurements on a declared runner; compare the review lane with the proposed two-minute target without dropping required cases or relaxing parity gates.

## Acceptance criteria

### AC-001 — The independent review finds no missing canonical contract, scope leak, stale destination, or unresolved parity failure.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Worst-case applicable state and composition workloads stay within documented bounds while preserving exact output and interaction behavior.
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
- **Check:** `CHK-005`

### AC-005 — Performance evidence is complete and coverage stays intact
```gherkin
Given the declared runner and complete conformance case set
When cold and warm builds, test shards, the full gate, and frame workloads are measured
Then the report compares the lane to the two-minute target without removing cases or weakening exact parity
```

**Verification**

- **Type:** invariant
- **Covers:** `R-006`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** Performance evidence covers shared library mechanisms and conformance workloads; release readiness requires all prior parity and negative gates.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is enforced by narrow writable library/conformance paths and forbidden application/oracle paths.

## Checklist

<!-- checklist:start -->
- [x] **1** Prepare.
    - [x] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [x] **2** Implement.
    - [x] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [x] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [x] **2.3** Preserve the frozen applications and keep product work out of scope. (`R-003`, `R-004`, `AC-003`, `CHK-004`)
    - [x] **2.4** Record cold/warm, shard, gate, and frame measurements. (`R-006`, `AC-005`, `CHK-003`)
- [x] **3** Verify.
    - [x] **3.1** Run the one completion gate. (`R-005`, `AC-004`, `CHK-005`)
<!-- checklist:end -->
