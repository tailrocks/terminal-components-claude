# Lane C: Jackin status

## Status

Historical planning lane. Jackin code now exists in split crates; the broader crate migration remains incomplete and not ready. This page records candidate acceptance requirements, not an active lane assignment.

## Authority

Current direction: [architecture](../architecture.md), [design](../design.md), [status](../status.md), and [plan](../plan.md). Product behavior follows `design/interaction-polish`; reusable APIs and package boundaries follow the split architecture represented by `main`.

## Current evidence

Jackin Preview currently spans `crates/jackin-preview`, `crates/jackin-preview-app`, `crates/jackin-preview-domain`, `crates/jackin-preview-host-ui`, `crates/jackin-preview-presentation`, `crates/jackin-preview-sim`, and `crates/jackin-preview-capsule-ui`. `RunId` exists in `crates/jackin-preview-domain/src/instance.rs`. The old `apps/jackin-preview` and Slice 5/7 sequencing claims are historical.

The former review proposed pure status projection, runtime update causes and repaint deadlines, route-owned product time, semantic dimming, and total run identifiers. Current source contains runtime update causes, repaint scheduling, semantic dimming, and a `RunId` type. Presence alone does not prove the detailed interaction, color, timing, or acceptance contracts. See [historical review](../reviews/laneC-app-tick.md).

## Remaining work

- Compare Jackin behavior, UI, and colors against `design/interaction-polish`.
- Verify the proposed timing, status, identity, and dimming contracts against current code and tests.
- Verify current visual and functional parity before claiming completion.

## Verification

Inspect current crate manifests, public APIs, app routes, focused tests, and design-branch behavior. Old inventory counts and green runs are historical evidence only.
