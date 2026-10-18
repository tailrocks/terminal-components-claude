# F16 — Reach every showcase section at supported sizes

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Broad showcase Inputs/Buttons/TextAreas reachability overhaul. Targeted showcase proof for a changed shared component is still required now.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed inaccessible-control defect plus coverage gap.** At 72×20,
TextAreas' fixed upper-card reservation leaves the enabled error field without
usable geometry; three Tabs visit only Task description/Notes then navigation.
At 100×30, Inputs exposes only three of eight state references. Buttons' matrix
is vertically/horizontally truncated without a reveal route. Locations:
`pages/textareas.rs:64`, `inputs.rs:123`, `buttons.rs:72/150`, under
`src/bin/showcase`. Design captures/replay and ecosystem source cross-check agree.

Root: layout truncation has no interaction owner for omitted sections. Use existing
Tabs/Select for compact section switching, retaining stacked/wide composition
when it fits. Preserve drafts and restore focus per section on resize. A shared
app helper is justified after two pages exercise identical semantics. Existing
ScrollPanel/TextViewport display text; neither is an arbitrary-child scrolling
container. Risk: medium focus/draft/resize behavior. Acceptance: every enabled
control and documented reference state reachable by keyboard/mouse at all five
sizes without resizing; hidden sections keep drafts but no stale hits/cursors.
Test lower-field editing through shrink/grow and independent wheel ownership.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
