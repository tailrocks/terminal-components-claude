# Picker query component audit — 2026-10-20

Reference behavior checked against frozen source commit
`4169783ae883788186d161af09ba9c481bd84e5c`, `src/widgets/picker.rs`:

- Escape clears a non-empty query; a subsequent Escape cancels the picker.
- Enter chooses the current eligible item; Alt+Enter chooses the alternate action.
- Ctrl+N/P navigate the results; Tab changes scope.
- Delete emits the secondary action when enabled; Backspace on an empty query
  emits Back. Other editing belongs to the query editor.
- Query editing, pointer selection, and cursor/horizontal-scroll ownership stay
  with the field.

The Termrock Picker now composes a real `TextInput` under its public picker ID
and gives results the stable child ID `picker-id.sub("results")`. The field
owns query text and editing state. `FilterList` contributes only the active
navigation and conditional owner actions through the runtime's same-owner
binding extension; it excludes overlapping editor commands first. The shared
field's default bindings remain unchanged unless a consumer explicitly opts
into `exclude_commands`.

The runtime integration regression exercises typing, Delete secondary,
Backspace editing, Back on an empty query, and Escape clear-then-dismiss. The
field regression checks that exclusions are opt-in and retain unrelated
commands. This is focused component evidence; paired CLI visual/functional
coverage remains a parent-level acceptance requirement.

Independent review found and the follow-up fixes cover these regressions:

- TextInput cancellation and commit end an edit session. When the still-focused
  owner receives a later character, TextInput now begins a fresh edit from the
  committed value before inserting. Picker integration checks typing after
  Escape-clearing and after an Enter with no eligible match.
- FilterList no longer claims physical Delete chords before the runtime resolves
  the typed action. Ctrl+K/P and Ctrl+J/N retain the frozen result-navigation
  behavior, and a real keymap regression removes secondary and remaps Down to
  Delete; the result cursor moves without dispatching Secondary.
- `FilterList::draw` retains its established full-area result viewport. Picker
  composes its query row explicitly through `draw_query`, so direct FilterList
  consumers do not lose a result row or shift result coordinates.
- TextInput has an opt-in auxiliary style target for GUTTER and PLACEHOLDER,
  preserving those part identities while allowing hosted controls to resolve
  through their host theme family. Picker opts in; ordinary TextInput defaults
  remain unchanged.

Validation:

- `cargo nextest run -p termrock-fields -p termrock-navigation -p termrock-overlays -E 'test(hosted_query_auxiliary_parts_can_resolve_through_host_family) | test(searchable_filter_list_draw_keeps_its_full_area_for_results) | test(picker_routes_query_editing_navigation_and_clear_through_text_input) | test(filter_navigation_remapping_uses_typed_action_not_delete_chord)' --no-fail-fast` — passed (4 tests).
- `cargo nextest run -p termrock-fields -p termrock-navigation -p termrock-overlays --lib --no-fail-fast` — 215 passed, 1 failed. The remaining failure is `termrock-fields::note::tests::note_wrap_and_clip`, an unrelated muted-color assertion (`Rgb(138, 138, 138)` vs `Rgb(128, 128, 128)`).
- `cargo nextest run -p termrock-fields -p termrock-navigation -p termrock-overlays --lib -E 'not test(note_wrap_and_clip)' --no-fail-fast` — 218 passed, 1 unrelated test skipped.
- Re-run including the known test: 218 passed, 1 failed (`note_wrap_and_clip`, muted-tone `Rgb(138,138,138)` vs expected `Rgb(128,128,128)`, at `crates/termrock-fields/src/note.rs:189`). The focused failure is reproducible without the changed code paths.
- `cargo clippy -p termrock-fields -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings` — passed.
- The runtime binding-composition prerequisite is commit `0cf26d54ecf53adec8a925050882d6a65d736f02`; its independent author and runtime review remains parent-owned.
