# Historical Jackin contract review

## Status

Historical candidate contract. Its former proposed decisions and lane blockers do not govern current work. Re-evaluate requirements against current Jackin crates and `design/interaction-polish`.

## Authority

Current architecture and product authorities are [architecture](../architecture.md), [design](../design.md), and [status](../status.md). Original review at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/reviews/laneC-app-tick.md).

## Current evidence

Jackin implementation is split across `crates/jackin-preview*`. The runtime currently exposes `UpdateCause`, bootstrap updates, repaint deadlines, and semantic `dim_layer`; `RunId` exists in `crates/jackin-preview-domain`. These source facts do not establish the detailed product behavior below or parity with the design branch.

## Candidate behavior contracts

- Jackin status is projected from current screen/world state during draw. Preserve every current status fact, priority, semantic tone, and stable action key without stale cached snapshots.
- Runtime updates distinguish Bootstrap, Event, Tick, and Settle. Bootstrap runs once before first draw or handled event, including headless execution. A delivered tick is visible only on its first update pass.
- Repaint requests retain the earliest deadline across unrelated input. Runtime schedules work; Jackin's route determines product time and advances domain state once per tick.
- `dim_layer(area, 0)` is identity. Semantic non-ladder roles dim monotonically and erase at the specified step. Compare all supported color levels with the design branch.
- Run identifiers remain total for generated and fixture values. Capsule/container and Manager presentation must not use unchecked string slicing.
- Preserve deterministic timestamps, paused motion, rain and handoff behavior, outro content, and all current visual surfaces.

These are candidates from a prior review, not current implementation findings. The former review's exact test names and inventory counts need fresh source mapping before use.

## Remaining work

Compare current routes, runtime integration, colors, and captures with `design/interaction-polish`. Map each candidate contract to current implementation and tests; add unresolved accepted requirements to the current plan. No Slice 5 sequencing blocker remains active by virtue of this document.

## Verification

Use current app/runtime source, focused tests, and visual comparison. Historical green runs and digest counts do not prove current parity.
