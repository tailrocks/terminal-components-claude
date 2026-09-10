# Prior-fix verification ledger

Scope: current `holla-fable` worktree, September 10, 2026. This pass inspected
current source, retained tests, and existing artifacts only. No other branch or
history was consulted. No product code or baseline was changed. The source
inventory and terminal-protocol research have separate owners; this document
verifies the prior implementation claims, not their research rationale.

## Fresh evidence and its limits

| Check run in this planning pass | Result |
| --- | --- |
| `rtk cargo fmt --check` | Pass |
| `rtk cargo clippy --all-targets -- -D warnings` | Pass |
| `rtk cargo test` | 328 passed across eight suites; 9.30 s |
| `rtk cargo doc --no-deps` | Pass; 4.30 s |
| `rtk git diff --check` | Pass, including this report |
| `rtk proxy cargo test --release --lib unchanged_overflow_redraw_reuses_layout -- --nocapture` | Pass; 4,000 retained lines, ten unchanged redraws: **517.542 µs**, zero additional layout rebuilds |
| Current baseline rows and distribution | **460** rows; each of 20 size/palette combinations has 23 page rows |
| Existing capture completeness | **301** matching sets of PNG, ANSI, text, HTML, and cursor metadata: 250 under `shots/audit`, 51 under `shots/audit-flows` |
| PNG decoding | Pillow `Image.open(path).verify()` passed for all 301 existing PNGs |

The optimized build again warned that `rust-objcopy` could not load
`libLLVM.dylib` while stripping debug information. Compilation and execution
succeeded. The timing above is a new, single bounded sample, including buffer
render/equality work. It is not a throughput, latency, allocation, or whole-app
benchmark. No historical implementation was rebuilt to obtain a fresh speedup.

The full suite executes the retained regressions named below, including the
actual Crossterm `NO_COLOR` subprocess. The release command repeats one of those
tests with its timing visible. Tests are not multiplied by the 460 baseline
frames: those frames are assertions within one test.

`showcase_visual_baseline` in `src/bin/showcase/app_tests.rs` covers 23 pages ×
five terminal sizes (72×20, 80×24, 100×30, 120×40, 160×50) × four palettes
(TrueColor, Ansi256, Ansi16, Mono), with the first control focused. Its digest
includes symbols, foreground, background, and modifiers, but deliberately
excludes the sidebar. It does not hash hardware cursor metadata or underline
color, exercise the Crossterm backend, or enumerate all interaction states.

Capture counts were checked by extension and matching filename stems, not just
by a directory total. PNGs were decoded, not freshly regenerated or all visually
re-reviewed in this planning pass. Existing files therefore establish artifact
availability and structural completeness, not provenance of the binary that
produced each pixel. The recorded prior regeneration remains historical evidence.

Useful count/coverage commands, none of which blesses a baseline:

```sh
rtk proxy wc -l tests/showcase_baseline.txt
rtk proxy awk '{count[$1 " " $2]++} END {for (k in count) print k, count[k]}' tests/showcase_baseline.txt
rtk proxy find shots/audit shots/audit-flows -type f -name '*.png'
rtk cargo test --bin showcase showcase_visual_baseline
```

## Status meaning

- **Fixed**: the named bounded defect is addressed at its responsible source
  boundary and a retained regression passed now. This is not universal proof of
  every input, terminal, or externally mutable state.
- **Partially proven**: the bounded fix is present, but an associated broader
  claim lacks matching retained/fresh executable evidence. The missing proof
  is named; this label does not assert that the original defect still occurs.
- **Open**: a remaining limitation or unfulfilled acceptance condition, not a
  claim that a previously fixed regression failed.

Test names below are stable identifiers within the stated file/module. All
listed retained tests ran in the fresh full suite. Historical failing-first
process claims cannot be re-established from current files alone.

## Main report: all 17 finding rows

This section follows the table order in [the consolidated report](tui-audit.md).

| ID / prior claim | Current responsible symbols | Retained regression evidence | Status and exact boundary |
| --- | --- | --- | --- |
| F01 — TextBuffer selection panic, arbitrary offsets, combining-cluster edits, newline normalization | `src/core/text.rs`: `selection_lines`, `select_range`, `insert_at`, `remove_range`, `normalize_positions`, `normalized_text`, selection replacement paths | `selection_lines_handles_unicode_and_exclusive_newline`; `public_offsets_clamp_to_graphemes`; `word_motion_and_deletion_preserve_combining_clusters`; `edits_repair_positions_when_graphemes_join`; `selection_replacement_does_not_move_past_joining_neighbors`; `every_text_entry_preserves_line_mode` | **Fixed** for the asserted examples and bounded offset sweep. The reported independent 100,000-operation probe is historical and not retained as a repository test/script; its result is not fresh proof. No universal Unicode-fuzz claim follows from this suite. |
| F02 — CodeEditor modified Enter and expanding-lowercase search panic | `src/widgets/code.rs`: `on_key`, `refind`; `src/widgets/field_common.rs`: `edit_key`; `src/ui/text.rs`: `SearchText`, `find_ranges`, `fuzzy` | `modified_enter_commits_without_changing_document`; `find_uses_original_grapheme_ranges_after_case_expansion`; `find_preserves_smart_case_and_whole_source_graphemes`; `fuzzy_subsequence_matches_scalars_not_utf8_bytes` | **Partially proven** wording: Ctrl+Enter and named Unicode examples pass; retained editor test covers Control only, not “every Enter modifier.” The shared translator handles non-plain Enter as Commit. `Key::plain` permits Shift, so Shift+Enter is a newline, not a commit. Terminal delivery of modifiers is a separate protocol question. |
| F03 — replacement dataset retained stale DataTable edit/selection | `src/widgets/table.rs`: `DataTable::set_rows` cancels edit/selection before replacement, rebuilds order, retains sort, clamps cursor | `replacing_rows_cancels_edits_without_touching_the_new_dataset` (0, 1, 3 replacement rows); `replacing_rows_clears_selection_and_preserves_sort_and_valid_cursor` | **Fixed** for supported replacement datasets. This does not validate arbitrary ragged row shape or direct external mutation of public collections. |
| F04 — sorted invalid edit wrote into wrong source row; invalid clicks discarded ownership | `src/widgets/table.rs`: `EditState.row`, `begin_edit`, `commit_edit`, `sort_by`, `on_click_cell`; `src/widgets/grid.rs`: `on_click` validation gates | Table: `rejected_edit_stays_on_its_source_row_after_header_sort`; `direct_sort_during_edit_keeps_source_identity_in_every_order`; `clicking_another_cell_preserves_invalid_edit_and_cursor`. Grid: `clicking_another_cell_keeps_invalid_editor_and_cursor`; `invalid_edit_blocks_header_sort_request_and_preserves_draft` | **Fixed** for header sorting, all three direct sort states, correction, cancellation, and invalid cell transitions. Public source-index meaning is explicit; arbitrary external dataset mutation is not covered by these tests. |
| F05 — raw mode leaked on failed terminal startup | `src/runtime.rs`: `TerminalSession::enter`, `initialize_with_restoration`, `Restoration::leave`/`Drop`, `restore_terminal` | `failed_terminal_setup_restores_state_and_preserves_error`; `panicking_terminal_setup_restores_state`; `successful_setup_transfers_restoration_and_leaves_once` | **Partially proven** lifecycle claim: restoration ownership before fallible setup is fixed and injected failure/panic/idempotence pass. Full termios restoration after real quit/broken-output startup was checked in prior PTYs, not freshly repeated here. These tests do not cover live draw/read errors, suspend/resume, fatal signals, hook ownership, or successful writes to an unavailable output device. |
| F06 — minimum form clipping and unreachable sidebar entries | `src/bin/showcase/pages/forms.rs`: `FormsPage::render`; `src/bin/showcase/app.rs`: `draw_sidebar`, navigation `ScrollState`, wheel/scrollbar routing | `form_keeps_all_controls_inside_small_normal_and_wide_page`; `minimum_sidebar_reveals_last_page_and_wheel_does_not_move_focus` | **Fixed** in tested composition: form sentinel/hit/focus checks cover five page rectangles in TrueColor/Mono; sidebar test uses 72×20, last-page reachability, wheel persistence and track click. This is not every form validation/busy state at every size. |
| F07 — TextInput required error, masked clicks, narrow overflow | `src/widgets/input.rs`: `validate`, `live_validate`, `display_graphemes`, `on_click`, `render` | `required_error_clears_after_keyboard_and_paste_corrections`; `masked_clicks_follow_display_graphemes`; `scrolled_masked_clicks_and_cursor_share_geometry`; `narrow_fields_stay_within_their_allocated_rectangle`; `wide_graphemes_keep_cursor_and_clicks_aligned_after_scroll` | **Fixed** for tested display geometry and validation recovery. Narrow test covers widths 0–11, heights 0–3, nonzero origin and error/help cases. Masked examples include CJK, ZWJ emoji and combining clusters. |
| F08 — CodeEditor active-find paste and small-pane/find overflow | `src/widgets/code.rs`: `on_paste`, find routing in `on_key`, bounded body/gutter/find-footer rendering | `paste_and_backspace_edit_find_query_in_each_document_mode`; `narrow_editor_and_long_find_stay_inside_nonzero_area` | **Fixed** for navigation/editing/read-only find ownership, grapheme Backspace, widths 1/3/7/16/40 with nonzero-origin Mono sentinels and hardware cursor checks. This is not a complete width×height×palette×editor-state matrix. |
| F09 — TextArea invisible long-line editing; editor wheel/scrollbar immediately undone | `src/widgets/textarea.rs`: `on_click`, `on_wheel`, horizontal offset and follow predicates in `render`; `src/widgets/code.rs`: follow predicates, `on_wheel`, `on_scrollbar`, cursor visibility | TextArea: `long_unicode_lines_follow_cursor_and_click_the_visible_position`; `manual_scroll_stays_until_cursor_movement_or_editing`; `resizing_keeps_long_line_cursor_visible`; `narrow_textareas_do_not_write_outside_their_allocation`. CodeEditor: `manual_scroll_survives_render_then_typing_reveals_cursor`; `horizontal_scroll_hides_offscreen_cursor`; `smaller_viewport_reveals_cursor_after_manual_scroll` | **Partially proven** breadth: long-line geometry, wheel/manual offset and resize recovery are fixed/tested. These named editor regressions do not drive editor scrollbar pointer events; the Showcase test `scrollbar_click_and_drag_move_the_view` exercises ListBox. Add editor-specific pointer replay before claiming equivalent scrollbar regression coverage. |
| F10 — invisible-by-color gutters became visible under NO_COLOR; selection/read-only focus indistinct | `src/theme.rs`: `gutter_symbol`, `selection`, `disabled_style`; component render call sites; CodeEditor read-only focus styling | `tests/focus_gutter.rs`: `button_focus_remains_unambiguous_without_colour`; `list_focus_and_selection_have_distinct_symbols_without_colour`; `text_selection_uses_reverse_video_only_in_monochrome`; `disabled_fields_and_menu_commands_stay_distinct_and_inert`. CodeEditor: `focused_read_only_editor_keeps_monochrome_navigation_gutter` | **Fixed** for covered controls/states in all four palettes and read-only editor Mono. Actual backend preservation is separately proven by A03, not by palette-only buffer tests. DIM visibility remains terminal-dependent. |
| F11 — choice labels/markers escaped allocation; hidden radio retained hits | `src/widgets/choice.rs`: bounded Checkbox/RadioGroup/Toggle rendering, RadioGroup geometry reset, saturating height/label budgets | `tests/choice_containment.rs`: `form_choices_keep_cells_and_hit_regions_inside_their_area`; `radio_group_clears_old_option_areas_when_hidden`; `radio_height_saturates_terminal_coordinates`; `toggle_handles_labels_larger_than_terminal_coordinates`; `compact_choice_marks_keep_checked_state_without_colour`; `disabled_choices_reject_keyboard_and_mouse_changes` | **Fixed** for TrueColor/Mono widths 0–12, disabled state, oversized labels, compact marks and stale hidden geometry. No claim that clipping keeps every label legible in arbitrarily small space. |
| F12 — long footer status disappeared; Showcase duplicated faulty budget logic | `src/widgets/keyhint.rs`: `render_toned`; `src/bin/showcase/app.rs`: `draw_footer` delegates to shared renderer | `long_status_keeps_its_severity_and_yields_hints_without_overlapping_edit_badge`; `narrow_status_and_badge_stay_inside_assigned_area`; `warning_and_error_statuses_carry_glyphs_so_mono_keeps_the_weight` | **Fixed** for Unicode/error priority, EDIT badge and narrow sentinel cases. Long status deliberately displaces hints; this is bounded presentation, not a guarantee that every command remains shown simultaneously. |
| F13 — first field click edited; drag lost original press anchor | `src/bin/showcase/app.rs`: `on_mouse`; `PageEvent::Press`; Terminal/Diff page press handling | `first_completed_click_focuses_fields_second_click_edits`; `shell_drag_preserves_original_press_through_release_in_both_viewports` | **Fixed** for Inputs/Textareas/Forms/Editor first/second clicks and App/TestBackend Terminal/Diff drags whose first motion differs from press origin. Exact diff selection/copy is additionally covered by the component/page tests in F17/A05. This is not all overlay, resize-during-drag or pointer-cancellation paths. |
| F14 — Facts acknowledgement mouse dispatch missed nested input | `src/widgets/dialog.rs`: `input_mut`, `on_click`, `on_paste`; TablePro safety gate routing | `src/bin/tablepro/app_tests.rs`: `safety_gate_acknowledgement_supports_mouse_focus_edit_and_confirmation` | **Partially proven** evidence wording: two-click editing, wrong-token rejection, typed correction, arming and explicit confirmation pass. `H::type_str` emits individual key events; this regression does **not** paste. Paste reaches the same nested input by inspected source, but a Facts acknowledgement paste integration assertion remains absent. |
| F15 — TablePro Ctrl+D shadowed row duplication | `src/widgets/grid.rs`: duplicate chord dispatch; TablePro workbench Data/Structure owner and contextual hints | `duplicate_chords_share_defaults_undo_and_read_only_guards`; `src/bin/tablepro/app_tests.rs`: `duplicate_row_chord_preserves_structure_editing_and_modal_routes` | **Fixed** for standalone Control/Alt duplication, primary-key default, copied values, undo/read-only guard, TablePro Ctrl+D mode switch, and editing/modal suppression. “Portable Alt+D” means an available alias at the application API; physical terminal/OS Alt encoding still requires compatibility evidence. |
| F16 — unchanged overflowing TextViewport rebuilt full layout repeatedly | `src/widgets/viewport.rs`: `set_area`, `ensure_layout`, `layout_size`, mutation dirty flags | `unchanged_overflow_redraw_reuses_layout`; `cached_layout_reflows_after_resize_wrap_and_content_changes` | **Fixed** for unchanged 4,000-line redraws; complete buffers remain equal and rebuild count is unchanged. Fresh timing appears above. Resize/wrap/set_lines/push/replace_last/clear compare against fresh layout. Appending still reflows retained history; public direct line mutation is not proven safe by this fix. |
| F17 — existing DiffView absent from showcase; direct public mode skipped cache invalidation | `src/widgets/diff.rs`: `layout`, mode cache, `layout_mode`, `effective_layout_mode`; `src/bin/showcase/pages/diff.rs`: existing widget composition; DESIGN catalogue | `direct_public_mode_change_invalidates_layout`; `review_falls_back_when_narrow_and_restores_when_wide`; `default_review_narrow_and_empty_render_with_focus_stops_in_monochrome`; `keyboard_and_mouse_controls_scroll_and_copy_selection`; A05 tests | **Fixed** for unified/review/empty composition, TrueColor/Mono, narrow fallback, wide recovery, public mode changes, wheel and selection/copy. No new widget/dependency is required by these changes. The public name is `DiffView`, not `DiffViewer`. |

## Main report: all six additional fixed groups

| ID / group | Current source and retained regression | Status and exact boundary |
| --- | --- | --- |
| A01 — queued page-change events used stale render-time focus | `src/runtime.rs`: `drain_ready_inputs`, `event_loop`; `queued_activation_waits_for_new_controls_to_render` uses real Focus/FocusRing, batches ignored/consumed events, stops before queued activation, renders, then activates the new control | **Fixed** for this ordering boundary. Historical paused-child `] Tab Enter Ctrl+L` versus delayed terminal replay matched text/ANSI/cursor/PNG after the fix. That replay was not rerun here. The unchanged-page `Tab Enter Ctrl+L` sequence did not reproduce the original defect. No sustained-flood fairness or every popup/mouse transition guarantee follows. |
| A02 — duplicate contextual Tab hints | `src/bin/showcase/app.rs`: `draw_footer`; `page_context_and_shell_do_not_duplicate_tab_hints` | **Fixed** for every current page after moving focus from navigation at 160×50. It checks no more than one Tab label, not that all commands have hints or all modifier routes are unique. |
| A03 — actual NO_COLOR empty SGR erased modifiers; disabled Mono lacked distinction | `src/runtime.rs`: `render_frame`, `reset_frame_colors`, Crossterm's memoized color policy; `no_color_backend_preserves_selection_attributes`; `Theme::disabled_style` and the F10/F11 tests | **Fixed** for emitted selection reverse/bold/underline with actual `NO_COLOR=1`: isolated child uses real Crossterm backend and rejects resets before selected text. Disabled controls preserve DIM in tested buffers. Historical terminal captures supplement this. Runtime callers bypassing the shared draw boundary are outside this guarantee; actual terminal DIM support and force-color/environment combinations need their own compatibility matrix. |
| A04 — table/grid edit windows mixed scalar and cell coordinates | `src/ui/text.rs`: `slice_cells`; DataTable/DataGrid render and editing click offsets; `horizontal_windows_preserve_display_positions_and_whole_graphemes`; both widgets' `unicode_edit_window_keeps_cells_bounded_and_clicks_at_display_position`; public slicing doctest | **Fixed** for the tested offset sweep and CJK/ZWJ/combining editing window with preserved adjacent cells in TrueColor/Mono. Terminal font/width agreement is distinct from buffer geometry correctness. |
| A05 — Diff emphasis split graphemes, tab widths disagreed, scrollbar covered final review cell | `src/widgets/diff.rs`: `changed_range`, `emphasised`, review/tab measurement and scrollbar budget; `review_emphasis_keeps_complete_graphemes_and_column_widths`; `tab_indented_review_aligns_context_changes_and_copied_text`; `review_budget_reserves_scrollbar_and_recovers_on_resize` | **Fixed** for complete changed ZWJ/combining spans, shared four-space tabs, aligned review separators/copied text and 43/44-column scrollbar thresholds. These tests supply geometry evidence that the PNG tool cannot. |
| A06 — hard wrap emitted empty line plus oversized grapheme | `src/ui/text.rs`: `wrap`, `hard_wrap`; `wrapping_never_splits_or_overflows_a_wide_grapheme`; `wraps_words_and_hard_wraps_long_tokens` | **Fixed** for stated CJK/emoji/combining examples and widths 0–7 under the documented **`w.max(1)`** contract. Width zero intentionally permits a one-cell output; the test does not prove a zero-width allocation receives no glyph. Callers still own zero-area clipping. |

## Subsidiary editor report: all nine rows

This explicit crosswalk prevents the less prominent claims in
[the editor report](tui-audit-editor.md) from disappearing into F02/F08/F09.

| ID / subsidiary claim | Current symbol / retained test | Status |
| --- | --- | --- |
| E01 — Ctrl+Enter commit no longer panics or changes text | `CodeEditor::on_key`; `modified_enter_commits_without_changing_document` | **Fixed**, exactly Control+Enter. F02 records the broader-modifier limitation. |
| E02 — transformed search offsets map back to source graphemes, including Greek/expansion/empty/dedup | `SearchText::new`, `original_range`, `find_ranges`, `CodeEditor::refind`; `find_uses_original_grapheme_ranges_after_case_expansion`; `find_preserves_smart_case_and_whole_source_graphemes`; `fuzzy_subsequence_matches_scalars_not_utf8_bytes` | **Fixed** for the retained examples. Greek contextual lowercase is explicitly asserted through fuzzy matching and shares SearchText; there is not a separate editor Greek-navigation assertion. Rust lowercase semantics are not NFC normalization, locale collation or full Unicode case folding. |
| E03 — fuzzy compared UTF-8 bytes and returned transformed offsets | `fuzzy` scalar iteration and source-grapheme mapping; `fuzzy_returns_original_grapheme_offsets`; `fuzzy_subsequence_matches_scalars_not_utf8_bytes`; `fuzzy_preserves_ascii_ranking` | **Fixed** for false byte subsequence `Ã©`/`é`, expansion, CJK, emoji, combining, Greek and four ASCII ranking categories. “ASCII ranking unchanged” is bounded preservation evidence, not an exhaustive equivalence proof. |
| E04 — find paste ownership in navigation/editing/read-only, grapheme Backspace | `CodeEditor::on_paste`, find branch in `on_key`; `paste_and_backspace_edit_find_query_in_each_document_mode` | **Fixed** for all three listed document modes and whole-cluster query deletion. |
| E05 — set_text refreshes find; empty query resets match index | `CodeEditor::set_text`, `refind` clears matches/current before empty return; `replacing_document_refreshes_existing_find_matches`; E04 empties the query | **Partially proven** breadth: document replacement and query deletion are asserted; resetting a previously nonzero `find.current` to zero has source evidence but no dedicated retained assertion. |
| E06 — wheel persists, offscreen cursors hidden, typing/resize restores follow | `CodeEditor::render`, `cursor_cell`, follow state; `manual_scroll_survives_render_then_typing_reveals_cursor`; `horizontal_scroll_hides_offscreen_cursor`; `smaller_viewport_reveals_cursor_after_manual_scroll` | **Fixed** for wheel, horizontal hiding, typing and shrinking viewport. Pointer-scrollbar equivalence remains F09's missing test. |
| E07 — width-one nonzero-origin and long-find containment | CodeEditor clipped rendering and find suffix budget; `narrow_editor_and_long_find_stay_inside_nonzero_area` | **Fixed** for tested widths, Mono, sentinel cells and cursor containment. Not exhaustive zero-height/find-state coverage. |
| E08 — read-only editor remains visibly focusable in Mono | CodeEditor read-only field/focus distinction; `focused_read_only_editor_keeps_monochrome_navigation_gutter` | **Fixed** for focused read-only gutter. Find/navigation acceptance is source-supported; read-only does not mean disabled. |
| E09 — toggling public read_only during editing prevents indentation and insertion | Reconciliation in `CodeEditor::on_key` and `render`; `switching_to_read_only_disables_all_document_edit_actions` | **Fixed** for Tab, BackTab and character insertion, preserving text and leaving edit mode. Test name says “all” but enumerates these three previously relevant actions, not every key or direct public buffer mutation. |

## Historical evidence and misleading language

The reports are useful implementation records, but the following phrases must
not be promoted into stronger current acceptance claims:

1. **“Every Enter modifier”** in the main report is stronger than the retained
   Control-only regression. Shift is explicitly considered plain by `Key::plain`
   and inserts a newline in multiline editors. Enumerate semantic combinations
   and terminal-deliverable combinations separately.
2. **Facts acknowledgement “paste” regression** is source-supported routing,
   not what the named integration test executes. Its helper types character
   keys. Add an actual `Input::Paste` case if paste is a required acceptance item.
3. **Editor “wheel and scrollbar regressions”** should distinguish wheel/manual
   offset tests from a real editor scrollbar press/drag/render test. Existing
   Showcase scrollbar regression targets a list, not either editor.
4. **“Restored on every exit path”** in runtime module/run documentation is too
   broad. Restoration is best effort; output and `disable_raw_mode` errors are
   discarded. SIGKILL cannot unwind. Suspend/continue, repeated panic-hook
   installation and a lost output device are not covered by existing tests.
   The narrowly claimed post-raw-mode startup guard fix remains valid.
5. The historical **100,000-operation Unicode probe**, **93.290084 ms / 40
   original reflows**, **386.667 µs fixed sample**, real PTY termios checks and
   paused burst/determinism replays were not reproduced in this planning pass.
   The new fixed-only timing is 517.542 µs; it does not revalidate the historical
   before/after ratio. Some historical evidence lives only at host `/tmp` paths.
6. Subsidiary counts such as editor **10 / UI 7**, library **130**, and runtime
   **three/four/five** describe intermediate milestones. Current UI text module
   has nine tests; current full-suite result is the gate table above. The
   “failing first” workflow claim is historical, not inferable from current code.
7. The interaction report's “proposed fix” and “CodeEditor remediation is owned
   by its separate audit” wording is stale status, not evidence that those
   defects remain open. Public widget names there should be `TextViewport` and
   `DiffView`, not `Viewport` and `DiffViewer`.
8. “Most interaction/color states” missing from the baseline should not imply
   supported palettes were omitted: all four are represented. Missing coverage
   is dynamic interactions, sidebar, hardware cursor/underline-color metadata,
   non-Showcase application hashes, and actual backend/environment behavior.
9. The main/verification reports assign different priorities to queued input
   (P1/P2) and actual NO_COLOR (P2/P1). This is classification inconsistency,
   not an unresolved implementation disagreement. Planning should use one
   explicit impact taxonomy.
10. “Widths 0–7” in wrap tests means the documented `w.max(1)` contract. It
    does not establish literal zero-column containment. Likewise “both palettes”
    in several editor/widget tests means TrueColor and Mono, not all four.
11. **301 captures** proves neither 301 independent scenarios nor automatic
    current-source provenance. They are 250 fixture/size/mode combinations and
    51 interaction states with complete files. The default `audit_shots.sh`
    selects seven fixtures (175 sets); reproducing 250 requires the explicit
    ten-case selection, not the default command alone.

## Remaining verification work for the improvement plan

These are bounded acceptance gaps. Unless labeled confirmed, they are not new
failure reproductions. Prior fixes need not be reverted or reimplemented to
address them.

| ID / state | Evidence now | Smallest useful next acceptance proof |
| --- | --- | --- |
| V01 — Open: complete terminal lifecycle | Startup guard tests use injected restoration counters; historical PTY checks cover startup error and normal quit. `restore_terminal` ignores I/O errors; panic hooks chain globally and are not restored by the owner. | Isolated controlling-PTY harness: normal quit, post-initialization draw/read error, app panic, repeated enter/leave, and partial setup at successive output boundaries. Compare complete termios, captured mode-reset sequences and exactly owned hook lifetime. Document unrecoverable-output/fatal-signal limits rather than promising impossible recovery. |
| V02 — Open: suspend/resume and signal contract | Current runtime has no SIGTSTP/SIGCONT handler or suspension protocol. No current retained test covers shell job control. | Establish the supported lifecycle contract first; then drive only an owned child with stop/continue and resize while stopped. Assert shell usability during suspension, mode re-entry/full redraw on continuation, and clean quit. Signal-protocol research belongs to the terminal research owner. Absence of tests alone is not a reproduced suspend defect. |
| V03 — Partially proven: resize and input ordering | Widget resize/fresh-layout tests and Showcase too-small recovery exist. A01 uses a real focus ring with a minimal event pump, not a complete terminal event-loop harness. | Retained real-app replay of queued page/modal changes, resize, activation, paste and mouse press/drag/release. Compare batched versus separated input results and focus/hit/cursor ownership. Include below-minimum→normal→wide→minimum and collapse/expand while editing/dragging. Keep timings out of correctness assertions. |
| V04 — Open: input-flood fairness | `drain_ready_inputs` yields for Changed, but continuously ignored/consumed events drain until the source is empty. `next_ready_input` similarly drains unsupported raw events. Tick checks occur after draining returns. | A finite injected queue can count how much work occurs before returning; a bounded PTY flood can measure maximum animation/quit latency. Specify an input/time budget only after asserting the intended fairness contract. Source structure supports a starvation risk, not a measured sustained-flood failure here. |
| V05 — Partially proven: large-history performance | F16 proves zero idle reflows at 4,000 lines. Dirty `ensure_layout` maps all retained lines; no whole-app large-list/table/log benchmark is retained. | Parameterized retained sizes, wrapping widths and mutation workloads: idle redraw, append, replace-last, retention eviction, resize, select/copy and scroll. Record reflow counts/allocations plus median/tail time separately. Compare visible output with fresh layout. Audit supported mutation boundaries before calling public cache invalidation safe. |
| V06 — Open: renderer Unicode fidelity, confirmed tool limitation | Fresh Python invocation of `ansi2png.wcwidth` sums `e\u0301` to **2** and `👩‍💻` to **5**. `render` iterates code points, draws one background/glyph per code point, and uses a fixed selected font without per-glyph fallback. | Keep Unicode fixtures unchanged. Use Ratatui buffers/cursor/copied-text assertions as library geometry evidence now. If PNGs become normative, test grapheme shaping, combining/ZWJ widths, style boundaries and font coverage against declared terminal/font settings; otherwise label PNGs approximate. CJK/emoji tofu in a PNG is not by itself a widget failure. |
| V07 — Partially proven: terminal color/attribute compatibility | F10 covers four palette buffers; A03 exercises one real memoized NO_COLOR policy in an isolated subprocess. | Subprocess matrix for explicit palette, NO_COLOR empty/nonempty, force-color policy and representative terminal capabilities. Verify modifiers and absence/presence of color SGR independently; review DIM/reverse/underline in real terminals. Avoid mutating memoized environment policy in parallel in-process tests. |
| V08 — Open: reproducible evidence provenance | Baseline hashes are executable/current; PNG artifact decoding is current; historical standalone probes and PTY timing live partly in prose or `/tmp`. | Retain small deterministic harnesses for the accepted Unicode, PTY and burst cases. Capture a manifest with current source digest, binary digest, scenario/frame/size/color environment, capture-tool version and font. Do not equate a manifest or hash with visual review. |
| V09 — Partially proven: understated subsidiary assertions | F02 modifier sweep, F09 editor scrollbar pointer routing, F14 Facts paste, E05 nonzero find-index reset have source fixes but narrower retained assertions. | Add only these missing assertions to their existing owning tests. Do not introduce a new editing/modal abstraction merely to close an evidence gap. |

The renderer diagnosis is reproducible without writing Python bytecode:

```sh
rtk proxy /tmp/holla-venv/bin/python -B -c 'import sys; sys.path.insert(0,"tools"); import ansi2png; print([(repr(s),sum(map(ansi2png.wcwidth,s))) for s in ["e\u0301","👩\u200d💻","🇯🇵","界"]])'
```

To deliberately reproduce the existing 250-set matrix into a separate output
directory after a fresh binary build, preserving current captures and baselines:

```sh
rtk cargo build --bins
CASES='showcase-buttons showcase-forms showcase-inputs showcase-textareas showcase-diff tablepro-production jackin-capsule jackin-accounts holla-rust holla-upgrade' \
  PY=/tmp/holla-venv/bin/python SHOT_DIR=/tmp/holla-plan-captures \
  tools/audit_shots.sh
```

The path above requires an existing Pillow interpreter on this audit host;
choose another installed interpreter elsewhere. This command is a reproduction
recipe, not a claim that this planning pass ran it. A new artifact directory
should be chosen if that exact temporary path already contains wanted data.

## Independent cross-check: remaining table/grid defects

At the plan owner's request, this verification pass independently checked the
API audit owner's new sorting/shape findings. A freshly built current library
was linked into a temporary standalone executable using only public APIs. No
repository test or product file was added or changed. The probe is
`/tmp/holla-plan-verify.XmPWte/grid_table_probe.rs` on this host; the minimal
operations and assertions are recorded below so its conclusions do not depend
only on that temporary path.

```sh
rtk cargo build --lib
rtk proxy rustc --edition=2024 /tmp/holla-plan-verify.XmPWte/grid_table_probe.rs \
  --extern junie_tui=target/debug/libjunie_tui.rlib -L dependency=target/debug/deps \
  -o /tmp/holla-plan-verify.XmPWte/grid_table_probe
rtk proxy /tmp/holla-plan-verify.XmPWte/grid_table_probe
```

All probes use one sortable text column. Grid setup is `DataGrid::new`,
`local_sort = true`, then `set_rows(GridRows { rows, total: RowTotal::Exact(n),
more: false })`. Header activation is `on_click(grid.header_id(0),
Default::default())`. No external mutation of private row/order storage is used.

| Cross-check | Minimal operations / observed result | Source cause and classification |
| --- | --- | --- |
| C01 — local sort changes navigation identity | Rows `[beta, alpha]`; cursor starts at `(0, 0)`, source row 0. Activate ascending header. Cursor remains `(0, 0)` but `source_row(cursor.0)` becomes **1**. | **Confirmed behavior / open consistency defect**: `DataGrid::request_sort` permutes `order` without remapping cursor/anchor. DataTable explicitly preserves cursor source identity. Existing grid test proves pending-change identity, not navigation identity. This does not contradict F04's stable edit source index. |
| C02 — local sort ignores effective pending values | Rows `[beta, alpha]`; `record_cell(0, 0, Text("aardvark"))`; activate ascending header. Iterate display rows through `value(source_row(i), 0)`: **`[alpha, aardvark]`**. | **Confirmed defect, open**: `apply_local_sort` compares stored `rows[a][col]`; rendering uses effective `value`, including pending edits. Ascending indicator therefore disagrees with displayed values. Define and test effective-value sorting and its selection/cursor semantics together. |
| C03 — ragged DataTable sort panic | `DataTable::new` accepts rows `[[Cell::new("valid")], []]` for one column. Calling `sort_by(0)` panics at `src/widgets/table.rs:225`, index 0 into an empty row. | **Confirmed defect, open** at the accepted public data boundary. `apply_sort` directly indexes row cells, while construction does not reject/normalize missing cells. F03 fixed stale replacement editing, not row-shape validation. |
| C04 — ragged DataGrid local sort panic | Grid `set_rows` accepts `[[Text("valid")], []]`; ascending header panics at `src/widgets/grid.rs:505`, index 0 into an empty row. | **Confirmed defect, open**: `value` safely defaults missing cells to Null, and width sampling tolerates missing cells, but local sorting uses direct indexing. Fix the common ingestion/access contract rather than treating each panic as an isolated display patch. |

The standalone executable catches each expected panic, asserts the two observed
sort mismatches, prints the outcomes, and exits successfully. “Probe passed”
means the **defect was reproduced**, not that sorting is correct. The next
implementation should convert these into retained regressions with corrected
expectations. Grid cursor preservation is a consistency requirement supported
by the existing DataTable contract; deciding the exact anchor/range behavior
still needs an explicit selection contract.

## Conclusion

No retained prior-fix regression failed in this pass. All 17 main rows, all six
additional fixed groups, and all nine subsidiary editor rows have a current
source/test crosswalk. Several blanket claims exceed their retained proof;
those are explicitly bounded above. Separate newly reproduced table/grid
sorting and accepted-input shape defects remain open. The improvement plan
should preserve the working fixes, address those confirmed defects, correct
evidence wording, and close the named lifecycle, interaction, performance and
renderer gaps with reproducible tests.
