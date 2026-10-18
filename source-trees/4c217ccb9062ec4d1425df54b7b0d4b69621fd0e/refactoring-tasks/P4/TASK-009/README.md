---
schema: task/v5
id: TASK-009
title: "Consolidate overlays and the menu engine"
kind: refactor
---

# TASK-009 — Consolidate overlays and the menu engine

## Goal

Dialog, Menu, ContextMenu, MenuBar, HelpOverlay, and Wizard share one layer-aware menu/overlay engine while retaining their visual recipes, with bounded consumer adoption across reference applications.

## Context

Overlay placement, focus trapping/restoration, backdrop policy, submenu behavior, and activation timing belong to the shared runtime plus one menu mechanism. Distinct components remain distinct public surfaces. Execute the P4 bounded consumer-adoption checkpoint across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Dependencies: TASK-003, TASK-005. Canonical contracts: [components/README.md](../../../docs/components/README.md), [components/dialog.md](../../../docs/components/dialog.md), [components/menu.md](../../../docs/components/menu.md), [components/context-menu.md](../../../docs/components/context-menu.md), [components/menu-bar.md](../../../docs/components/menu-bar.md), [components/help-overlay.md](../../../docs/components/help-overlay.md), [components/wizard.md](../../../docs/components/wizard.md), [architecture/overview.md](../../../docs/architecture/overview.md), [foundations/layers.md](../../../docs/foundations/layers.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Shared menu engine, modal/context/menu-bar variants, help overlay, wizard transitions, focus traps, and restoration.
- Bounded consumer-adoption checkpoint: integrate candidate overlay and menu components into affected presentation call sites across `showcase`, `tablepro`, `jackin-preview`, and `holla`.

Out of scope:

- Separate menu painters, product dialogs, service workflows, and visual changes to existing application overlays.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** Bounded consumer-adoption call sites in `showcase`, `tablepro`, `jackin-preview`, and `holla` adopt candidate overlays and menus while keeping application domain models, simulations, scenarios, snapshots, and observable visual/interaction outputs 1:1 invariant.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — Opening, navigating, activating, and closing each overlay variant produces the documented focus/layer/action result.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Nested menus, outside clicks, disabled entries, modal backdrops, narrow geometry, and restoration follow the shared contract.
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

- **D-001:** One menu engine supports Menu, ContextMenu, and MenuBar; overlays use runtime-owned layers and typed actions; candidate components are integrated incrementally into consumer applications rather than deferred.
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
