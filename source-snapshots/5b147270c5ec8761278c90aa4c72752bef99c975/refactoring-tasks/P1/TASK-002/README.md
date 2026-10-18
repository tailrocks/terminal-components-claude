---
schema: task/v5
id: TASK-002
title: "Establish identity, events, responses, and public API"
kind: refactor
---

# TASK-002 — Establish identity, events, responses, and public API

## Goal

The library exposes stable semantic identities and typed update responses through the constrained Termrock API contract.

## Context

Termrock needs caller-owned durable state, borrowed props, typed actions, stable IDs and keys, and explicit `update`, `draw`, and `measure` phases. Preserve these ownership boundaries while replacing legacy public concepts.

Dependencies: TASK-001. Canonical contracts: [architecture/overview.md](../../../docs/architecture/overview.md), [api/public-api.md](../../../docs/api/public-api.md), [foundations/identity.md](../../../docs/foundations/identity.md), [foundations/input-actions.md](../../../docs/foundations/input-actions.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Identity, events, responses, public API traits/functions, props/state separation, action typing, and stable-key rules.

Out of scope:

- Universal boxed `Widget`, `show()` semantics, legacy API compatibility promises, application behavior, and visual redesign.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** The frozen applications, snapshots, and visual-baseline tests remain untouched; their exact output is the oracle.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — A component can update caller-owned state and emit a typed action without painting or owning durable state.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Reconciliation preserves a semantic ID and selected key across reordered input while borrowed props remain short-lived.
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

- **D-001:** The target API is Termrock's typed update/draw/measure model; legacy `WidgetId`, `Outcome`, and `RenderCtx` are migration internals only.
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
