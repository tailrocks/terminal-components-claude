# F05 — Preserve TreeView target across lazy insertion

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Implement the shared TreeView identity contract and integrate it into HP04/HP19 where trees are used. Prove delayed insertion, collapse, filtering, fallback and reveal using primitive tests and Holla fixtures.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No separate TablePro feature work. Minimal caller migrations and existing regression checks required by a shared API change remain part of this slice.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed defect.** `TreeView::set_children` calls `flatten`, which keeps
only a clamped numeric cursor (`src/widgets/tree.rs:145/203/227`). Inserting
children above a focused sibling moves the cursor to a new child although the
old node survives. TablePro's `Workbench::tick_explorer` is a present delayed-load
consumer. API and independent design probes agree.

Root: flattened display position is treated as node identity. Capture the
focused path before rebuild, resolve it afterward, and define fallback for a
removed/hidden target (visible ancestor, then surviving neighbor). Keep cursor
and selected path distinct; do not steal focus when delivery occurs elsewhere.
Risk: medium collapse/filter behavior risk. Acceptance: delayed nested loads,
focus on later siblings, collapse, filters, empty children and keyboard/mouse
toggle paths preserve or deliberately relocate the target and reveal it.
Positional paths do not solve arbitrary sibling reordering; that remains a
separate owner-identity contract, not justification for a global key framework.

## Evidence

**Slice status:** current shared slice complete; TreeView identity is used by the Holla disk tree.

**Shared widget:** `src/widgets/tree.rs` captures the focused path before a rebuild, resolves it afterwards and relocates deliberately (visible ancestor, then surviving neighbour) when the node is gone; cursor and selection stay distinct. Tests: `tree.rs: delayed_children_above_the_cursor_keep_the_focused_node`, `collapse_relocates_to_the_visible_ancestor_and_removal_to_a_neighbour`, `filters_reveal_and_reset_deliberately`, `wheel_moves_the_viewport_and_keeps_the_cursor`.

**Holla integration:** `screens/disk.rs` rebuilds the scan tree from filesystem identity (expansion set and focus are re-resolved by path across every streamed rebuild; selection follows live paths and a parent dominates its descendants). Tests: `app_tests_parity.rs: hp18_disk_scan_streams_measures_exactly_and_keeps_cached_hints_as_hints`, `hp19_tree_navigation_sorting_folding_selection_and_top_files`; `app_tests_proofs.rs: idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree` (rebuild counter). HP04's browser is a flat listing, not a TreeView; it needs no lazy identity.

**Captures inspected:** `shots/h_hp18_scanning`, `shots/h_hp18_cancelled`, `shots/h_hp19_tree`, `shots/h_hp19_unfolded`.

**Deferred remainder:** none for this slice; positional paths do not solve arbitrary sibling reordering (out of scope by contract).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
