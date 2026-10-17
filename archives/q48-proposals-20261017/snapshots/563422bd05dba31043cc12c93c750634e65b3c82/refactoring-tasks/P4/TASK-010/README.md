---
schema: task/v5
id: TASK-010
title: "Consolidate pickers, command palette, chains, and completion"
kind: refactor
---

# TASK-010 — Consolidate pickers, command palette, chains, and completion

## Goal

Picker, CommandPalette, PickerChain, and Completion share one controlled picker mechanism with explicit ownership of filtering and selection, with bounded consumer adoption across reference applications.

## Context

Picker-like surfaces differ in recipe and data, but must not grow separate filtering, query, focus, and completion runtimes. Compose the collection, editing, and overlay foundations, and execute the P4 bounded consumer-adoption checkpoint across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Dependencies: TASK-006, TASK-009. Canonical contracts: [components/README.md](../../../docs/components/README.md), [components/picker.md](../../../docs/components/picker.md), [components/command-palette.md](../../../docs/components/command-palette.md), [components/picker-chain.md](../../../docs/components/picker-chain.md), [components/completion.md](../../../docs/components/completion.md), [foundations/collections.md](../../../docs/foundations/collections.md), [foundations/text.md](../../../docs/foundations/text.md), [foundations/layers.md](../../../docs/foundations/layers.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Shared picker mechanism, query/edit state, filtering, command palette, chained selection, completion ownership, and typed results.
- Bounded consumer-adoption checkpoint: integrate candidate picker, command palette, and completion surfaces into affected presentation call sites across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Out of scope:

- Shell completion services, provider integrations, product command registries, and duplicate picker runtimes.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** Bounded consumer-adoption call sites in `showcase`, `tablepro`, `jackin-preview`, and `holla` adopt candidate pickers and completion surfaces while keeping application domain models, simulations, scenarios, snapshots, and observable visual/interaction outputs 1:1 invariant.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — A picker filters and reconciles keyed options while preserving focus and emitting the documented typed result.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Empty/loading/partial results, chained selection, disabled options, Escape, keyboard-suppressed hover, and narrow overlays follow the contract.
```gherkin
Given an applicable boundary, interaction, or reconciliation state
When the future implementation is exercised
Then the documented state, identity, geometry, and ownership invariants hold
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-002`

### AC-003 — Frozen references, application invariants, and scope remain protected
```gherkin
Given the immutable visual-baseline applications and bounded consumer-adoption call sites
When the candidate tree is checked after implementation
Then only allowed component-adoption call sites change; application behavior, snapshots, baseline tests, and product integrations are unchanged
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

- **D-001:** One picker mechanism supports the four public surfaces; callers own query/data and receive typed selection or completion actions; candidate components are integrated incrementally into consumer applications rather than deferred.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is enforced by narrow writable library/conformance paths and forbidden application/oracle paths.

## Checklist

<!-- checklist:start -->
- [x] **1** Prepare.
    - [x] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [x] **2** Implement.
    - [x] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [x] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [x] **2.3** Execute the bounded consumer-adoption checkpoint across the four applications, preserving frozen behavior and keeping product work out of scope. (`R-003`, `R-004`, `AC-003`, `CHK-003`)
- [x] **3** Verify.
    - [x] **3.1** Run the one completion gate. (`R-005`, `AC-004`, `CHK-004`)
<!-- checklist:end -->
