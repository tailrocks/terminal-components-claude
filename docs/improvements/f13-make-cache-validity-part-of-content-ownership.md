# F13 — Make cache validity part of content ownership

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete enforced revisioned content ownership and migrate every affected caller. Prove Holla text, cursor and copy agree after every supported mutation; retain idle caching.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice. Necessary compatibility migrations in other consumers are allowed; unrelated features are not.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed API/cache weakness, public-field mutation case.**
`TextViewport.lines` is writable but cache invalidation is private. Render OLD,
assign its span text NEW, render at same size → model NEW, buffer OLD
(`viewport.rs:111/245/305`). Text and interaction probes agree; this is distinct
from F11's supported setter failure.

Prefer read-only content access plus revisioned mutation methods/guards that
also own F11/F12. Migrate direct readers/writers from the inventory before
deliberate field-visibility changes. A mandatory caller revision token only
works if the API enforces or reliably detects its update; a private dirty flag
cannot be the caller contract. Risk: medium public compatibility. Acceptance:
every supported mutation updates text/copy/cursor next frame, including same-byte-
length edits; unchanged frames reuse cache. Do not rehash the whole history on
each frame as an unmeasured workaround or claim completion while silent direct
mutation remains possible.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
