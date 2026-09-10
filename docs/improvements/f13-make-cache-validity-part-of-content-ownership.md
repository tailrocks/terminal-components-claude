# F13 — Make cache validity part of content ownership

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete enforced revisioned content ownership and migrate every affected caller. Prove Holla text, cursor and copy agree after every supported mutation; retain idle caching.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

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

**Slice status:** current shared slice complete.

**Shared boundary:** viewport content is mutated only through revisioned methods (`set_lines`, `push`, `replace_last`, `set_max_lines`); every mutation invalidates exactly the changed lines and the next frame reflects text, cursor and copy; unchanged frames reuse the cached layout. Tests: `viewport.rs: every_mutation_is_visible_next_frame`, `unchanged_overflow_redraw_reuses_layout`, `cached_layout_reflows_after_resize_wrap_and_content_changes`, `tail_update_reparses_only_the_changed_line`.

**Callers migrated:** Holla activity, plan and files screens, the showcase terminal page and Jackin panes use the revisioned methods; no direct field writes remain (the field is private).

**Holla proof:** `app_tests_proofs.rs: a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout` (incremental cells equal a fresh layout after a burst), `idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree`.

**Captures inspected:** `shots/h_hp14_burst`.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
