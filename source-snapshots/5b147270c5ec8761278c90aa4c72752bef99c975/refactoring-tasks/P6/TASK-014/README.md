---
schema: task/v5
id: TASK-014
title: "Add prepared-cell TerminalView and terminal edge contracts"
kind: refactor
---

# TASK-014 — Add prepared-cell TerminalView and terminal edge contracts

## Goal

TerminalView presents caller-provided terminal cells with styles, continuation, cursor, and selection while emitting typed interaction requests, with bounded consumer adoption across reference applications.

## Context

Termrock may display prepared terminal cells but must not become an emulator, PTY runtime, shell, daemon, session manager, or escape parser. Keep terminal/session integration at the documented optional edge, and execute the P6 bounded consumer-adoption checkpoint for terminal-pane presentation in `showcase` and `jackin-preview`.

Dependencies: TASK-011, TASK-012, TASK-013. Canonical contracts: [components/README.md](../../../docs/components/README.md), [components/terminal-view.md](../../../docs/components/terminal-view.md), [architecture/overview.md](../../../docs/architecture/overview.md), [foundations/session.md](../../../docs/foundations/session.md), [verification/conformance.md](../../../docs/verification/conformance.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Prepared-cell TerminalView, cursor/selection/link/copy requests, continuation cells, capability handling, and session cleanup edge contracts.
- Bounded consumer-adoption checkpoint: integrate candidate TerminalView into terminal-pane presentation call sites in `showcase` and `jackin-preview`.

Out of scope:

- Terminal emulation, PTY, shell, daemon, agent-session manager, escape parsing, and provider services.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** Bounded consumer-adoption call sites in `showcase` and `jackin-preview` adopt candidate TerminalView while keeping application domain models, simulations, scenarios, snapshots, and observable visual/interaction outputs 1:1 invariant.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.

## Acceptance criteria

### AC-001 — Prepared cells draw with exact styles and continuation behavior while cursor/selection interactions produce typed requests.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Cursor visibility, selection, links, copy, resize, color capability, and terminal cleanup behavior follow the canonical edge contract.
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

- **D-001:** TerminalView consumes caller-provided cells and emits typed requests; all session and emulator concerns remain external; candidate components are integrated incrementally into consumer applications rather than deferred.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is enforced by narrow writable library/conformance paths and forbidden application/oracle paths.

## Checklist

<!-- checklist:start -->
- [x] **1** Prepare.
    - [x] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [x] **2** Implement.
    - [x] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [x] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [x] **2.3** Execute the bounded consumer-adoption checkpoint across terminal-pane applications, preserving frozen behavior and keeping product work out of scope. (`R-003`, `R-004`, `AC-003`, `CHK-003`)
- [x] **3** Verify.
    - [x] **3.1** Run the one completion gate. (`R-005`, `AC-004`, `CHK-004`)
<!-- checklist:end -->
