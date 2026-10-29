# TablePro shared picker scope and selection contract

Status: implemented in a WIP branch; parent integration and full product acceptance remain pending.

## Reference and ownership

The fixed baseline source is `4169783ae883788186d161af09ba9c481bd84e5c`; the fixed design source is `f4867e9b5f8a08a3bb61ce80d644198a29f51984`. The baseline Picker advances scopes on Tab and moves backward on BackTab. TablePro refreshes a Picker after query and scope changes and starts at the first enabled result. The current shared FilterList already owns query editing, matching, cursor, and scrolling, so this change extends that contract instead of restoring the old task branch's separate query editor.

`SelectionPolicy::Preserve` remains the default. `FirstEnabledOnFilterChange` resets the cursor on query and scope changes while ordinary stable-key reconciliation still preserves selection across data reordering. TablePro quick switcher and tab picker opt into that policy. Showcase's quick picker demonstrates the same shared option. Application code continues to project domain data and handle typed `PickerAction` values.

## Changes and tests

- Added a BackTab binding and typed `PrevScope` action in `termrock-navigation`; `Picker` wraps to the previous scope and emits the existing typed `Scope` action.
- Added the shared selection policy to `FilterList` and `Picker`, with a pending reset that survives the owner's post-query or post-scope data projection.
- Added runtime tests for BackTab wraparound, non-empty query preservation across scope changes, first-enabled result selection after query/scope refresh, stable-key reorder preservation, and a real TablePro `Harness` regression where a selected result survives the new query set.
- Negative checks reproduced failures when the BackTab binding was removed and when Picker stopped re-arming selection reset after `QueryChanged`.

## Verification

Focused nextest runs passed: navigation binding (1/1), Picker projection (2/2), and real TablePro Picker test (1/1). The final package nextest inventory ran 557 tests: 545 passed, 12 failed, 0 skipped. A same-source pre-change run ran 554 tests: 542 passed, 12 failed, 0 skipped. The 12 failures were identical in both runs:

- `tablepro-ui app::query_result_keys::explain_orders_shows_a_plan_cell_the_error_frame_hides`
- `tablepro-ui::form_radio form_radio_enter_submits`
- `tablepro-ui::form_radio form_radio_narrow_frames_clip_the_environment_field`
- `tablepro-ui::form_radio form_radio_focus_bolds_the_painted_row`
- `tablepro-ui::form_radio form_radio_right_card_pins_miss_the_stock_form`
- `tablepro-ui::form_radio form_radio_off_marker_uses_the_hollow_circle`
- `tablepro-ui::footer_unicode connection_form_names_preserve_footer_grapheme_boundaries`
- `tablepro-ui::perf frame_tablepro_connection_form_120x40`
- `tablepro-ui::perf frame_tablepro_grid_500x14_120x40`
- `tablepro-ui::row_actions_contract inserting_after_five_hundred_rows_reveals_the_new_owned_record`
- `tablepro-ui::replacement_contract save_connect_from_workbench_form_requires_guard_and_cancel_keeps_work`
- `tablepro-ui::visual tablepro_visual_baseline`

`cargo fmt --all --check` and `git diff --check` passed. Strict Clippy for `termrock-navigation`, `termrock-overlays`, and the `termrock` facade passed. A broader strict Clippy run including TablePro and Showcase demos remains blocked by existing warnings in TablePro (23) and Showcase demos (3). The test-local panic warning found during that run was removed by returning a test error instead. The final package nextest log SHA-256 is `f2fbf03fcff46fd204d090cf0e9d9f7bd48ccfdea1779933008731747148445d`; the same-source baseline run log SHA-256 is `946f18db819d31aedac0a45d3d04c321b003b485825147aee98aa032d12ac8da`; the shared Clippy log SHA-256 is `5953419ba874013f2afa300b09fba0f10e6f9d1a0a2cecca7c784b6dbc38e953`.

After the package run, a small storage optimization was made in `FilterList::reconcile`: it retains the last query only for the opt-in `FirstEnabledOnFilterChange` policy; default `Preserve` consumers do not clone/store it. The focused navigation, projection, and TablePro tests plus strict shared Clippy passed again (`final-focused-and-clippy.log`, SHA-256 `041370485140ad57b2b8b7a0d594bf788eb0d6c130cde16ecaffba87f3979dba`). The broad package inventory was not rerun after this optimization; the unchanged 12-failure inventory above is the before/after evidence for the feature implementation, not a pass claim for the post-optimization tip.

The paired baseline/candidate check remains unrun for this increment. No visual-parity acceptance is claimed. The published reference artifacts and test expectations were not changed.

## Independent review corrections (2026-10-21)

An independent review of implementation `d911e577a1342729719a746a1b314bea7880be46` found two API defects: BackTab carrying the terminal's redundant SHIFT modifier did not match the BackTab binding, and an explicit `reset_selection_on_next_reconcile` request was ignored under the default `Preserve` policy. Both probes failed before the fixes: the shifted real Picker event remained in scope 0 rather than wrapping to scope 3; explicit reset retained key 3 rather than selecting enabled key 2.

The fix is shared. `Chord::effective_eq` now treats SHIFT as intrinsic only for printable characters and BackTab, while preserving other modifiers and the distinction for Tab and arrows. This equality is already used by runtime dispatch, remapping, removal, and conflict checks. `FilterList::reconcile` now honors an explicit reset independently of automatic query-change reset policy; automatic query changes still reset only under `FirstEnabledOnFilterChange`, and stable-key reorder reconciliation remains unchanged.

Regression proof is committed in `termrock-core` chord tests, a default-policy `FilterList` test (including a disabled first item and empty projection), and an actual `Picker` Harness test sending `BackTab + SHIFT` after query input. The exact tests failed before implementation and passed after. Focused nextest passed 4/4; the affected core/runtime/controls/navigation/overlays packages passed 923/923 with 3 skipped. Strict all-target Clippy for those five packages, `cargo fmt --all --check`, and `git diff --check` passed. These checks cover the shared routing contracts but are not the pinned real-CLI baseline comparison; application parity remains incomplete.

### Global keymap mutation consistency (2026-10-21)

A subsequent independent review found that `KeyMap::remove`, `KeyMap::remap`, and `remove_before_typing` used structural `Chord` equality while dispatch and conflict detection use `Chord::effective_eq`. The new public test for global BackTab mutation failed before this fix: removing an entry by BackTab+SHIFT left the BackTab binding active. The remap and typing-capture cases share the same comparison inconsistency.

All three mutation APIs now use `effective_eq`, leaving structural `Eq`/`Hash` intact. Tests verify that BackTab+SHIFT can remove/remap BackTab, before-typing removal follows the same equivalence, and ALT/SHIFT distinctions remain intact for other keys and actual ALT on BackTab. Keymap conflict tests now cover redundant SHIFT on BackTab and distinct ALT. Focused nextest passed 5/5; the full affected core/runtime/controls/navigation/overlays package set passed 924/924 with 3 skipped. Strict all-target Clippy, formatting, and diff checks passed. This closes the shared key-identity API defect; it does not supply the missing pinned real-CLI application comparisons.

The pinned real-CLI paired baseline/candidate comparison has not been run for this increment. This is not visual-parity clearance. Picker geometry, the TablePro tab-strip repaint, other fixture selectors, and the complete application acceptance inventory remain separate work.
