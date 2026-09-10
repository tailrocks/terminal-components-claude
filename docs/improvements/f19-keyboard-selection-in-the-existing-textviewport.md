# F19 — Keyboard selection in the existing TextViewport

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete shared TextViewport keyboard selection and Holla Activity/Plan/preview integration, preserving Diff delegation and existing mouse behavior.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No separate viewer or full terminal emulator.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed coverage gap.** `viewport.rs:563` scrolls/follows/copies/clears;
Shift+Right does nothing and Shift+Down scrolls without selection. Range creation
is mouse-only. Diff delegates to the same handler. Design, interaction, API and
research agents independently checked the absence and present consumers: Holla
output, Jackin panes/Inspect, Showcase Terminal/Diff.

Add an explicit keyboard browse/selection caret, distinct from producer-owned
terminal caret. Reuse logical CellPos, selection/copy events and visual mapping.
Specify entry/exit, anchor extension, word/line/document movement, follow pause,
offscreen autoscroll and hints before implementation. Follow F10–F14 ownership
work; do not introduce another viewer or duplicate Diff selection state.

Risk: medium/high interaction semantics. Acceptance: keyboard and mouse copy
identical partial/word/multiline/wrapped Unicode ranges; preserve source identity
under resize, append and retention. Escape clears selection before exiting the
owner; no browse keys leak into attached terminal input. Cover empty/single-line,
no-selection copy, monochrome reversal and hardware-cursor policy. Plain scrolling
keeps existing behavior outside explicit selection mode.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
