---
schema: task/v5
id: TASK-007
title: "Unify text editing, fields, and forms"
kind: refactor
---

# TASK-007 — Unify text editing, fields, and forms

## Goal

TextInput, TextArea, Field, and Form consume one shared editing model while preserving their documented editing and Escape/paste differences, with bounded consumer adoption across reference applications.

## Context

Editing semantics must be shared without flattening visual or interaction differences. Use the text foundation and component contracts to separate durable draft/commit state from borrowed props, and execute the P3 bounded consumer-adoption checkpoint across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Dependencies: TASK-004, TASK-005. Canonical contracts: [components/README.md](../../../docs/components/README.md), [components/text-input.md](../../../docs/components/text-input.md), [components/text-area.md](../../../docs/components/text-area.md), [components/field.md](../../../docs/components/field.md), [components/form.md](../../../docs/components/form.md), [foundations/text.md](../../../docs/foundations/text.md), [api/public-api.md](../../../docs/api/public-api.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Shared text editing core, grapheme/cursor movement, drafts, validation hooks, Field, TextInput, TextArea, and Form.
- Bounded consumer-adoption checkpoint: integrate candidate text editing, field, and form components into affected presentation call sites across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Out of scope:

- CodeEditor and DiffView rich output, application form redesign, and one generic Escape policy for every editor.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** Bounded consumer-adoption call sites in `showcase`, `tablepro`, `jackin-preview`, and `holla` adopt candidate editing and form components while keeping application domain models, simulations, scenarios, snapshots, and observable visual/interaction outputs 1:1 invariant.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — Editing actions update the caller-owned draft and emit typed commit/cancel/validation responses exactly as documented.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Unicode graphemes, paste, Escape, read-only, disabled, invalid, and narrow-width behavior remain distinct where the contract requires it.
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

- **D-001:** TextInput and TextArea retain intentional baseline behavior differences; Form composes fields without owning their editing engines; candidate components are integrated incrementally into consumer applications rather than deferred.
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
