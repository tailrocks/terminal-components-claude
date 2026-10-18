# Architecture

## Current checkout

This branch is a single Cargo package, `junie-tui`, with a library and four binaries. The library is organized under `src/core`, `src/ui`, `src/widgets`, `src/shell`, and `src/theme.rs`; application code and fixtures live under `src/bin/{showcase,tablepro,jackin_preview,holla}`. `xtask` and the visual-baseline harness are workspace members/tools. The concrete source tree is authoritative for what exists today.

The library currently combines public component APIs, durable widget state, drawing, and event handling. The applications contain their own routing and composition. This is implementation evidence, not the intended final boundary.

## Structural target

The accepted structural direction is a virtual workspace with a curated `junie-tui` component library, a separate testing support crate, four application packages, and `xtask`. Applications consume public facades; internal runtime and component machinery remains private. The design branch preserves the product behavior and pixels that the workspace refactor must reproduce. The main branch's workspace work is evidence for structural progress, not evidence of product parity or completion.

The detailed component target, including state ownership, update/draw separation, identity, response types, focus and layer routing, theme resolution, collection contracts, and application boundaries, remains in [shared-components-plan.md](architecture/shared-components-plan.md). Its requirements are planned contracts until verified in code and tests.

## Acceptance

- Keep product appearance and interaction aligned with [design.md](design.md) and the approved visual evidence.
- Keep durable state separate from frame-local props and geometry.
- Make event ownership, focus, overlays, scrolling, and action delivery explicit and testable.
- Keep app-specific domain behavior in application packages and reusable presentation in the library.
- Demonstrate each structural boundary with executable checks; source layout alone does not prove the contract.

## Verification

Use the repository commands in [development.md](development.md), the architecture/task gates in [plan.md](plan.md), and the visual qualification procedure in [baseline/tuisnap-coverage.md](baseline/tuisnap-coverage.md). No current pass is implied by this target description.
