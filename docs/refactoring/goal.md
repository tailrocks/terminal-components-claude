# Refactoring goal

## Status

The target architecture is planned; the design branch remains the product reference. The workspace refactor and application migration are incomplete until every required structural contract and product behavior is verified.

## Authority

For user-visible behavior, [`docs/design/DESIGN.md`](../design/DESIGN.md), the design/interaction-polish source, and approved immutable captures are authoritative. For package direction and API requirements, use [architecture.md](../architecture.md), [shared-components-plan.md](../architecture/shared-components-plan.md), and the accepted task contracts. Current source establishes implementation facts only; historical plans and status snapshots do not override these authorities.

## Current evidence

This checkout is a root library package with four demo applications. The intended structural direction is a virtual workspace with a reusable component library, testing support, `xtask`, and four app packages. Main has made structural progress, but that does not prove the design target has been reproduced.

## Remaining work

Complete the active obligations in [remaining-work.md](../remaining-work.md) and the sealed task packages. Preserve the required public behavior and APIs through an intentional migration: keep durable state caller-owned, separate update from draw, centralize input/focus/overlay/scroll policy, use stable identity and typed actions, keep app domain logic outside the component library, and verify boundaries with executable checks.

## Verification

Completion requires the task-specific build and architecture gates plus exact visual and interaction qualification across the required applications, scenarios, sizes, and terminal color modes. Follow [plan.md](../plan.md) and [baseline/tuisnap-coverage.md](../baseline/tuisnap-coverage.md). No completion result is asserted here.
