# F23h — precise subsidiary regressions (V09)

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain actual Facts Input::Paste and input-modifier/source-mapping regressions for primitives changed by the Holla phase. Test exact owned payload and resulting state.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Standalone CodeEditor/TextArea scrollbar-drag, Enter matrices and find-reset gaps remain deferred unless an affected shared change requires that specific regression.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Current tests narrower than some names/report claims.

**Root fix / retained acceptance:** Add Ctrl/Alt/Shift Enter semantic combinations, actual Facts Input::Paste, editor scrollbar press/drag/redraw and nonzero-find-index reset to existing owning tests. Preserve known-good behavior; no framework needed.

## Evidence

**Slice status:** current Holla/shared slice complete · standalone CodeEditor/TextArea gaps stay Later.

**Regressions retained:** `app_tests_proofs.rs: facts_page_takes_an_actual_paste_into_the_editing_field_only` (a real `Input::Paste` lands in the editing field and nowhere else), `enter_with_modifiers_maps_to_exact_picker_and_finder_semantics` (Shift+Enter chooses like Enter, Alt+Enter is the alternate action, Ctrl+Enter is no chord), `output_scrollbar_press_and_drag_redraw_and_find_index_resets_on_a_new_query` (scrollbar press and drag redraw the viewport; a new find query resets a nonzero match index); `widgets/input.rs: required_error_clears_after_keyboard_and_paste_corrections`.

**Deferred remainder:** CodeEditor/TextArea scrollbar-drag and Enter matrices outside the changed primitives (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
