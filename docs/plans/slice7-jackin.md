# Jackin Preview acceptance plan

## Status

Jackin code is split across `crates/jackin-preview`, `crates/jackin-preview-app`, `crates/jackin-preview-domain`, `crates/jackin-preview-host-ui`, `crates/jackin-preview-presentation`, `crates/jackin-preview-sim`, and `crates/jackin-preview-capsule-ui`. The broader refactor remains incomplete and not ready. This plan tracks behavior/API acceptance in current crates; it does not schedule another app-package move.

## Authority

Current direction: [architecture](../architecture.md), [design](../design.md), [status](../status.md), and [plan](../plan.md). `design/interaction-polish` controls UI, colors, and behavior. `main` controls reusable crate boundaries and architecture.

## Current evidence

Current manifests prove package boundaries only. Runtime update-cause, deadline, dimming, and `RunId` APIs exist in current source, but their presence does not prove Jackin product parity or acceptance. Old `apps/jackin-preview`, Slice 5/7 sequencing, work ownership, and “blocker” statements are retired.

The detailed historical acceptance source is preserved at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/plans/slice7-jackin.md). It retains the original screen inventory, UI journeys, secret handling, timing, regression scenarios, performance thresholds, and open questions. These are candidates for current verification, not current source paths or completion claims.

## Remaining work

Verify these behavior and API contracts against current `crates/jackin-preview*` and the design reference:

- Preserve host chrome, route menus, tabs, status items, hints, dialogs, forms, Capsule panes, Manager rows, account/usage/settings routes, and all scenario flows.
- Keep status projection pure and current; preserve priority, semantic tone, stable action identity, and transient state across repaint.
- Distinguish Bootstrap, Event, Tick, and Settle updates. Bootstrap precedes the first draw or handled event, including headless use. A timer delivery advances Jackin once.
- Repaint requests preserve the earliest deadline across unrelated input. Runtime schedules ticks; the active route owns product time and supplies the domain delta.
- Dimming is identity at zero steps, monotonic for semantic roles, and erases at the specified step. Match supported color levels and handoff behavior to the design branch.
- Run identifiers and derived container/image tokens are total for all fixtures and generated values. No unchecked string slicing.
- Preserve transient secret masking and save/key-validation behavior; preserve persisted-value display exactly as the design branch specifies.
- Preserve paused-motion determinism, timestamps, rain/handoff transitions, failure and running frames, outro content, and all accepted visual surfaces.
- Map historical tests, digests, and performance subjects to current evidence. Classify design differences before changing a baseline.
- Keep shared controls and runtime behavior in reusable crates; keep Jackin routes and domain vocabulary in app crates.

Record unresolved findings in the canonical plan with current source references. Historical Slice 5/7 gates, counts, and sequencing do not create current blockers.

## Verification

Use current source, focused tests, deterministic runtime evidence, performance measurements, and visual comparisons against the accepted design branch. A package manifest or successful build alone does not establish completion.
