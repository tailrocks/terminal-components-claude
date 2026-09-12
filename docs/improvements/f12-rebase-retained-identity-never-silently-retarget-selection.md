# F12 — Rebase retained identity, never silently retarget selection

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete retained line identity reconciliation for selection, drag, viewport and producer caret. Integrate Holla output churn, resize and follow behavior.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed wrong-source copy; P2 reading-position defect.**
`TextViewport::push`, `viewport.rs:171`, saturates selected line indices after
eviction without resolving removed identities; it omits drag and visible anchors.
Cap 3, A/B/C: select A, append D → copied selection becomes B. Press B, append D,
drag → wrong newline range. With follow off, visible B jumps to C although B
survives. Text and interaction agents independently reproduce all three.

Apply one edit delta/stable-line reconciliation to selection head/anchor, active
drag anchor, producer caret and first visible logical position. Clear entirely
evicted selections; define clipping of partially surviving ranges. Preserve
retained reading position while follow is off, then map back into wrapped rows.
Dataset replacement needs an explicit reset/identity contract, not only index
clamping. Risk: medium/high selection semantics. Acceptance: all three repros,
partial/multiline eviction, active drag, wrapped rows, resize, follow-on, empty
and zero retention. Existing passing logical selection across resize must remain.

## Evidence

**Slice status:** current shared slice complete.

**Shared boundary:** one edit delta reconciles the selection head and anchor, the active drag anchor, the producer caret and the first visible logical line; fully evicted selections clear, partially surviving ranges clip, reading position is retained while follow is off. Tests: `viewport.rs: eviction_never_retargets_a_selection`, `active_drag_survives_eviction_with_the_right_source`, `reading_position_is_retained_while_follow_is_off`, `producer_caret_follows_eviction_and_replacement`, `replace_last_keeps_identity_and_clips_selection`, `keyboard_selection_pauses_follow_and_survives_append_and_resize`, `marks_paint_and_survive_eviction`.

**Holla integration:** activity output churn (append, tail replacement for prompt rows, retention eviction) and resize keep the reading position and find marks. Tests: `app_tests_proofs.rs: output_scrollbar_press_and_drag_redraw_and_find_index_resets_on_a_new_query`, `resize_below_minimum_then_normal_then_wide_then_minimum_keeps_every_state_consistent`; `app_tests_parity.rs: hp14_…` (retention while following), `hp15_…` (prompt row replaced in place).

**Captures inspected:** `shots/h_hp14_find` (marks after eviction), `shots/h_hp15_answered`.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
