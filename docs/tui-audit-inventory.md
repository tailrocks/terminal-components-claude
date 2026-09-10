# Rich-TUI inventory and API audit

Current branch: `holla-fable`, inspected September 10, 2026. No other branch or history was inspected. The historical state/use map and findings below retain their original evidence. The final API appendix and final page map are authoritative for the completed worktree.

## Corrected scope and method

The library exports `core`, `runtime`, `theme`, `ui`, and `widgets`: 46 public modules across 47 source files, including 31 widget modules. The final showcase has 23 pages. The initial declaration scanner was incomplete: it stopped at the first `#[cfg(test)]` item in each file, counted implementation details under private modules, and misread function-pointer aliases. Its reported 849 entries were not an exported-API count. Those declaration lists have been removed and replaced with the compiler-derived appendix.

The final API inventory uses Rustdoc JSON for the production library, generated with:

```sh
RUSTC_BOOTSTRAP=1 rtk cargo rustdoc --lib -- --output-format json -Z unstable-options
```

`RUSTC_BOOTSTRAP` enables only Rustdoc's JSON output for this diagnostic command; source and Cargo configuration are unchanged. The export traversal begins at the crate root, follows public modules and public inherent members, and includes inherited-public trait members and enum payload fields. Private fields, private-module contents, test-only items, external blanket implementations and inferred auto traits are excluded. Derived trait implementations are listed separately as source-declared contracts; their compiler-generated methods are not counted as handwritten APIs.

The appendix lists every source-backed exported type, alias, function/method, field, variant, constant, module and re-export with its exact current file and line. Tuple payload fields use numeric names. The manually implemented `WidgetId: Debug` contract and its `fmt` method are included. Each JSON item identity is counted once; the `table::Tone` re-export is a separate exported path, not a second enum declaration.

Production `TextBuffer` has 37 methods. Its individually test-gated `cursor`, `has_selection` and `selected_text` helpers are absent from production exports, but methods following those helpers are included. `theme::palette` is private, so `rgb` and its 25 palette constants are not exported. `table`'s private numeric-sort wrapper and runtime restoration/event-loop helpers are also excluded.

| Export category | Count |
|---|---:|
| Public modules | 46 |
| Structs | 77 |
| Enums | 51 |
| Traits | 1 |
| Type aliases | 5 |
| Enum variants | 204 |
| Public struct and enum payload fields | 511 |
| Source functions and methods | 668 |
| Free constants | 6 |
| Associated constants | 2 |
| Re-exports | 1 |
| Manually implemented public trait contracts | 1 |
| Total source-backed exported entries | 1573 |
| Derived trait contracts (separate) | 572 |

The source hash covers sorted library filenames, each followed by NUL, file bytes and NUL, using SHA-256: `40e412fd6bb7ff7700b90db054b0a4e5a5d535863e4eea4e55594f6bfb26202e`. Every public-function span from Rustdoc was checked against its source declaration; zero mismatches. Counts describe available API, not behavioral test results.

Usage lists below are historical direct imports or qualified source references, not exhaustive transitive dependency claims. For example, Dialog consumes Button and TextInput internally. The state model remains `VisualState`: focused, hovered, pressed, selected, disabled, error, editing and busy. Final implementation and verification outcomes are in [the audit report](tui-audit.md).

## Historical state and usage snapshot

### `src/widgets/brand.rs`

Resting lockup; compact form; optional click target. Brand text is caller supplied.

- showcase: `src/bin/showcase/pages/chrome.rs`:13.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/rain.rs`:228; `src/bin/jackin_preview/screens/capsule.rs`:13; `src/bin/jackin_preview/app.rs`:38,2288,2436.
- holla: `src/bin/holla/app.rs`:14.

### `src/widgets/button.rs`

Primary, secondary, subtle, danger, toggle; focus, hover, pressed, selected/toggled, disabled, busy. Key activation and click.

- showcase: `src/bin/showcase/pages/taskrunner.rs`:10; `src/bin/showcase/pages/terminal.rs`:14; `src/bin/showcase/pages/grid.rs`:11; `src/bin/showcase/pages/pickers.rs`:12; `src/bin/showcase/pages/progress.rs`:8,89; `src/bin/showcase/pages/dialogs.rs`:8; `src/bin/showcase/pages/buttons.rs`:9; `src/bin/showcase/pages/settings.rs`:12; `src/bin/showcase/pages/sidebars.rs`:10; `src/bin/showcase/pages/forms.rs`:8.
- tablepro: `src/bin/tablepro/connections.rs`:12; `src/bin/tablepro/tabs.rs`:16,2516; `src/bin/tablepro/app.rs`:17,2454.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:14; `src/bin/jackin_preview/screens/accounts.rs`:15; `src/bin/jackin_preview/screens/cockpit.rs`:10; `src/bin/jackin_preview/screens/manager.rs`:14; `src/bin/jackin_preview/screens/editor.rs`:11; `src/bin/jackin_preview/screens/modals.rs`:14; `src/bin/jackin_preview/screens/prelude.rs`:10; `src/bin/jackin_preview/screens/settings.rs`:10.
- holla: `src/bin/holla/screens/activity.rs`:11; `src/bin/holla/screens/review.rs`:11; `src/bin/holla/screens/plan.rs`:12,870; `src/bin/holla/screens/snapshot.rs`:12; `src/bin/holla/app.rs`:15.

### `src/widgets/chips.rs`

Cursor, selected chip, close/add/lead actions, overflow. Key navigation, click ownership.

- showcase: `src/bin/showcase/pages/chips.rs`:12.
- tablepro: `src/bin/tablepro/tabs.rs`:17; `src/bin/tablepro/workbench.rs`:732,737,741,744,747,751,1066,1070,1073,1076.
- jackin_preview: no direct source reference.
- holla: `src/bin/holla/screens/activity.rs`:12.

### `src/widgets/choice.rs`

Checkbox checked/unchecked; radio cursor/selected; toggle on/off; focus/hover/pressed/disabled; key and click.

- showcase: `src/bin/showcase/pages/settings.rs`:13; `src/bin/showcase/pages/forms.rs`:9.
- tablepro: `src/bin/tablepro/connections.rs`:13.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:14; `src/bin/jackin_preview/screens/accounts.rs`:16; `src/bin/jackin_preview/screens/editor.rs`:12; `src/bin/jackin_preview/screens/modals.rs`:15; `src/bin/jackin_preview/screens/settings.rs`:11.
- holla: no direct source reference.

### `src/widgets/code.rs`

Navigation/editing/read-only, selection, find, diagnostics, running statement, completion request; key, click, drag, wheel, scrollbar, paste.

- showcase: `src/bin/showcase/pages/editor.rs`:17.
- tablepro: `src/bin/tablepro/tabs.rs`:18.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/completion.rs`

Closed/open, current/hover row, fuzzy match, scroll, choose/dismiss; key, click, wheel.

- showcase: `src/bin/showcase/pages/editor.rs`:18.
- tablepro: `src/bin/tablepro/tabs.rs`:19.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/dialog.rs`

Confirm/destructive/prompt/facts/armed acknowledgement, disabled confirm, edit/error, modal focus barrier, outside cancellation.

- showcase: `src/bin/showcase/app.rs`:18; `src/bin/showcase/pages/taskrunner.rs`:11; `src/bin/showcase/pages/grid.rs`:12; `src/bin/showcase/pages/mod.rs`:10; `src/bin/showcase/pages/dialogs.rs`:9; `src/bin/showcase/pages/settings.rs`:14.
- tablepro: `src/bin/tablepro/connections.rs`:534,554,557; `src/bin/tablepro/app.rs`:18.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:15; `src/bin/jackin_preview/screens/capsule.rs`:15; `src/bin/jackin_preview/screens/accounts.rs`:17; `src/bin/jackin_preview/screens/cockpit.rs`:11; `src/bin/jackin_preview/screens/manager.rs`:15; `src/bin/jackin_preview/screens/mod.rs`:20; `src/bin/jackin_preview/screens/editor.rs`:13; `src/bin/jackin_preview/screens/prelude.rs`:11; `src/bin/jackin_preview/screens/settings.rs`:12; `src/bin/jackin_preview/app.rs`:12.
- holla: `src/bin/holla/screens/plan.rs`:13; `src/bin/holla/screens/mod.rs`:19; `src/bin/holla/app.rs`:16.

### `src/widgets/diff.rs`

Unified/split modes; added/modified/deleted/renamed files; context/add/remove lines; viewport scrolling, selection, copy; mode and layout updates.

- showcase: no direct source reference.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/sim/changes.rs`:6.
- holla: no direct source reference.

### `src/widgets/empty.rs`

Empty/error presentation with optional hint; display only.

- showcase: `src/bin/showcase/pages/chips.rs`:13; `src/bin/showcase/pages/editor.rs`:19.
- tablepro: `src/bin/tablepro/connections.rs`:956,960,976,980; `src/bin/tablepro/tabs.rs`:20,1650,1918,2447; `src/bin/tablepro/workbench.rs`:10,1282.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:16; `src/bin/jackin_preview/screens/accounts.rs`:18; `src/bin/jackin_preview/screens/manager.rs`:16; `src/bin/jackin_preview/screens/usage.rs`:11.
- holla: `src/bin/holla/screens/activity.rs`:388,392; `src/bin/holla/screens/finder.rs`:12.

### `src/widgets/field_common.rs`

Shared edit action classification: apply, insert, commit, cancel, Tab direction, none.

- showcase: no direct source reference.
- tablepro: no direct source reference.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/grid.rs`

Row/cell navigation, selection/range, editing/error, loading/fetch more, pending edit/insert/delete, undo/save/discard, local/external sort, horizontal/vertical scroll.

- showcase: `src/bin/showcase/pages/grid.rs`:13.
- tablepro: `src/bin/tablepro/tabs.rs`:21; `src/bin/tablepro/app.rs`:19; `src/bin/tablepro/workbench.rs`:780,787; `src/bin/tablepro/model.rs`:15,17.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/hintbar.rs`

Base/focused/modal/menu/session hint layers, centered placement, edit badge, status. Layer resolution and dropping hints under width pressure.

- showcase: `src/bin/showcase/pages/chrome.rs`:14.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/app.rs`:13.
- holla: `src/bin/holla/app.rs`:17.

### `src/widgets/input.rs`

Navigation/editing, snapshot commit/cancel, validation/error/required, disabled, placeholder/help, masking/tail reveal; key, click, paste.

- showcase: `src/bin/showcase/pages/dialogs.rs`:10; `src/bin/showcase/pages/settings.rs`:15; `src/bin/showcase/pages/forms.rs`:10; `src/bin/showcase/pages/textareas.rs`:8; `src/bin/showcase/pages/inputs.rs`:8.
- tablepro: `src/bin/tablepro/connections.rs`:14; `src/bin/tablepro/tabs.rs`:24; `src/bin/tablepro/app.rs`:20,1654; `src/bin/tablepro/workbench.rs`:11.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:16; `src/bin/jackin_preview/screens/capsule.rs`:17; `src/bin/jackin_preview/screens/accounts.rs`:19; `src/bin/jackin_preview/screens/editor.rs`:14; `src/bin/jackin_preview/screens/modals.rs`:16; `src/bin/jackin_preview/screens/prelude.rs`:12.
- holla: `src/bin/holla/screens/review.rs`:12; `src/bin/holla/app.rs`:18.

### `src/widgets/keyhint.rs`

Key/action labels, dropping hints, edit badge, warning/error status glyphs. Display only.

- showcase: `src/bin/showcase/pages/chrome.rs`:15.
- tablepro: `src/bin/tablepro/connections.rs`:15; `src/bin/tablepro/tabs.rs`:25; `src/bin/tablepro/app.rs`:21; `src/bin/tablepro/workbench.rs`:12.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:17; `src/bin/jackin_preview/screens/capsule.rs`:18; `src/bin/jackin_preview/screens/accounts.rs`:20; `src/bin/jackin_preview/screens/cockpit.rs`:12; `src/bin/jackin_preview/screens/manager.rs`:17; `src/bin/jackin_preview/screens/usage.rs`:12; `src/bin/jackin_preview/screens/mod.rs`:21; `src/bin/jackin_preview/screens/editor.rs`:15; `src/bin/jackin_preview/screens/modals.rs`:17; `src/bin/jackin_preview/screens/prelude.rs`:13; `src/bin/jackin_preview/screens/settings.rs`:13; `src/bin/jackin_preview/app.rs`:14.
- holla: `src/bin/holla/screens/activity.rs`:13; `src/bin/holla/screens/review.rs`:13; `src/bin/holla/screens/finder.rs`:13; `src/bin/holla/screens/disk.rs`:14; `src/bin/holla/screens/plan.rs`:14; `src/bin/holla/screens/mod.rs`:20; `src/bin/holla/screens/snapshot.rs`:13; `src/bin/holla/app.rs`:19.

### `src/widgets/list.rs`

Single/multi selection, cursor/chosen/checked/range anchor, disabled rows, empty, overflow; key, click, wheel, scrollbar.

- showcase: `src/bin/showcase/pages/lists.rs`:8; `src/bin/showcase/pages/panels.rs`:9; `src/bin/showcase/pages/chrome.rs`:16; `src/bin/showcase/pages/scrolling.rs`:9; `src/bin/showcase/pages/settings.rs`:16.
- tablepro: `src/bin/tablepro/tabs.rs`:26.
- jackin_preview: `src/bin/jackin_preview/screens/modals.rs`:18.
- holla: no direct source reference.

### `src/widgets/menu.rs`

Closed/open bar and anchored menu, cursor/hover, disabled/separator/destructive items, choose/dismiss/outside, focus barrier.

- showcase: `src/bin/showcase/pages/chrome.rs`:17.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:19; `src/bin/jackin_preview/app.rs`:39.
- holla: `src/bin/holla/screens/mod.rs`:21; `src/bin/holla/app.rs`:20.

### `src/widgets/panel.rs`

Card/frame, focus/meta, content area; ScrollPanel wrapping, follow-tail, keyboard/wheel/scrollbar.

- showcase: `src/bin/showcase/app.rs`:950; `src/bin/showcase/pages/taskrunner.rs`:12; `src/bin/showcase/pages/lists.rs`:9; `src/bin/showcase/pages/terminal.rs`:15; `src/bin/showcase/pages/chips.rs`:14; `src/bin/showcase/pages/editable.rs`:8; `src/bin/showcase/pages/grid.rs`:16; `src/bin/showcase/pages/pickers.rs`:13; `src/bin/showcase/pages/panels.rs`:10; `src/bin/showcase/pages/trees.rs`:8; `src/bin/showcase/pages/progress.rs`:9; `src/bin/showcase/pages/dialogs.rs`:11; `src/bin/showcase/pages/editor.rs`:20; `src/bin/showcase/pages/chrome.rs`:20; `src/bin/showcase/pages/scrolling.rs`:10; `src/bin/showcase/pages/buttons.rs`:10; `src/bin/showcase/pages/tables.rs`:9; `src/bin/showcase/pages/settings.rs`:17; `src/bin/showcase/pages/sidebars.rs`:11; `src/bin/showcase/pages/overview.rs`:10; `src/bin/showcase/pages/forms.rs`:11; `src/bin/showcase/pages/textareas.rs`:9; `src/bin/showcase/pages/inputs.rs`:9.
- tablepro: `src/bin/tablepro/connections.rs`:16; `src/bin/tablepro/tabs.rs`:27; `src/bin/tablepro/workbench.rs`:13.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:18; `src/bin/jackin_preview/screens/accounts.rs`:21; `src/bin/jackin_preview/screens/manager.rs`:18; `src/bin/jackin_preview/screens/usage.rs`:13.
- holla: `src/bin/holla/screens/activity.rs`:14; `src/bin/holla/screens/review.rs`:14; `src/bin/holla/screens/finder.rs`:14; `src/bin/holla/screens/disk.rs`:15; `src/bin/holla/screens/plan.rs`:15; `src/bin/holla/screens/snapshot.rs`:14.

### `src/widgets/picker.rs`

Query/editing, filtering, cursor/current result, loading/empty/error, selection/scope events, key/click/wheel.

- showcase: `src/bin/showcase/pages/pickers.rs`:14.
- tablepro: `src/bin/tablepro/app.rs`:22.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:19; `src/bin/jackin_preview/screens/capsule.rs`:22; `src/bin/jackin_preview/screens/manager.rs`:19; `src/bin/jackin_preview/screens/mod.rs`:22; `src/bin/jackin_preview/screens/editor.rs`:16; `src/bin/jackin_preview/screens/modals.rs`:19; `src/bin/jackin_preview/screens/prelude.rs`:14; `src/bin/jackin_preview/screens/settings.rs`:14; `src/bin/jackin_preview/app.rs`:15.
- holla: `src/bin/holla/screens/mod.rs`:22; `src/bin/holla/app.rs`:23.

### `src/widgets/progress.rs`

Running/success/error/paused, spinner/indeterminate; meter low/medium/high, unknown/known, thin/block and semantic meter tones.

- showcase: `src/bin/showcase/pages/taskrunner.rs`:13; `src/bin/showcase/pages/progress.rs`:10,81.
- tablepro: `src/bin/tablepro/connections.rs`:1063; `src/bin/tablepro/tabs.rs`:1575,1642; `src/bin/tablepro/app.rs`:2251.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:23; `src/bin/jackin_preview/screens/accounts.rs`:22; `src/bin/jackin_preview/screens/cockpit.rs`:13; `src/bin/jackin_preview/screens/manager.rs`:20; `src/bin/jackin_preview/screens/usage.rs`:14; `src/bin/jackin_preview/screens/editor.rs`:17; `src/bin/jackin_preview/screens/modals.rs`:20; `src/bin/jackin_preview/screens/settings.rs`:15; `src/bin/jackin_preview/app.rs`:872,2465.
- holla: `src/bin/holla/screens/disk.rs`:16; `src/bin/holla/screens/plan.rs`:16; `src/bin/holla/screens/snapshot.rs`:15; `src/bin/holla/app.rs`:24.

### `src/widgets/props.rs`

Static/wrapped facts; PropsList cursor, copyable entries, empty/scroll, keyboard/click copy request.

- showcase: `src/bin/showcase/pages/chips.rs`:15; `src/bin/showcase/pages/grid.rs`:17; `src/bin/showcase/pages/pickers.rs`:15; `src/bin/showcase/pages/editor.rs`:21.
- tablepro: `src/bin/tablepro/connections.rs`:17; `src/bin/tablepro/tabs.rs`:28; `src/bin/tablepro/app.rs`:23.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:20; `src/bin/jackin_preview/screens/capsule.rs`:24; `src/bin/jackin_preview/screens/cockpit.rs`:14; `src/bin/jackin_preview/screens/manager.rs`:21; `src/bin/jackin_preview/screens/editor.rs`:18; `src/bin/jackin_preview/screens/modals.rs`:21; `src/bin/jackin_preview/screens/settings.rs`:16; `src/bin/jackin_preview/app.rs`:817,821,822,826.
- holla: `src/bin/holla/screens/review.rs`:15; `src/bin/holla/screens/finder.rs`:1338,1339; `src/bin/holla/screens/disk.rs`:19; `src/bin/holla/screens/plan.rs`:17; `src/bin/holla/screens/snapshot.rs`:16; `src/bin/holla/app.rs`:25.

### `src/widgets/scrollbar.rs`

Overflow visibility; track/thumb and focus/hover; click-to-offset geometry.

- showcase: `src/bin/showcase/pages/taskrunner.rs`:16; `src/bin/showcase/pages/lists.rs`:10; `src/bin/showcase/pages/terminal.rs`:16; `src/bin/showcase/pages/editable.rs`:9; `src/bin/showcase/pages/panels.rs`:11; `src/bin/showcase/pages/trees.rs`:9; `src/bin/showcase/pages/editor.rs`:22; `src/bin/showcase/pages/scrolling.rs`:11; `src/bin/showcase/pages/tables.rs`:10; `src/bin/showcase/pages/settings.rs`:18; `src/bin/showcase/pages/textareas.rs`:10.
- tablepro: `src/bin/tablepro/tabs.rs`:29; `src/bin/tablepro/workbench.rs`:14.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:21; `src/bin/jackin_preview/screens/capsule.rs`:25; `src/bin/jackin_preview/screens/accounts.rs`:23; `src/bin/jackin_preview/screens/cockpit.rs`:501,666,673,685; `src/bin/jackin_preview/screens/manager.rs`:22; `src/bin/jackin_preview/screens/usage.rs`:15; `src/bin/jackin_preview/screens/editor.rs`:19; `src/bin/jackin_preview/screens/modals.rs`:22; `src/bin/jackin_preview/app.rs`:1558.
- holla: `src/bin/holla/screens/activity.rs`:323,357; `src/bin/holla/screens/review.rs`:16; `src/bin/holla/screens/finder.rs`:15; `src/bin/holla/screens/disk.rs`:20; `src/bin/holla/screens/plan.rs`:18; `src/bin/holla/screens/snapshot.rs`:17; `src/bin/holla/screens/modals.rs`:11.

### `src/widgets/segments.rs`

Identity/context segments, optional click targets, bold/tone, priority-based omission.

- showcase: `src/bin/showcase/pages/chips.rs`:16.
- tablepro: `src/bin/tablepro/app.rs`:24.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:26; `src/bin/jackin_preview/screens/accounts.rs`:24; `src/bin/jackin_preview/screens/cockpit.rs`:15; `src/bin/jackin_preview/screens/manager.rs`:23; `src/bin/jackin_preview/screens/mod.rs`:23; `src/bin/jackin_preview/screens/editor.rs`:20; `src/bin/jackin_preview/screens/settings.rs`:17; `src/bin/jackin_preview/app.rs`:16.
- holla: `src/bin/holla/app.rs`:26.

### `src/widgets/select.rs`

Closed/open popup, selected/cursor, disabled, help; keyboard navigation/choose/dismiss and click.

- showcase: `src/bin/showcase/pages/chips.rs`:17.
- tablepro: `src/bin/tablepro/connections.rs`:18,623; `src/bin/tablepro/app.rs`:25,1680,2060.
- jackin_preview: `src/bin/jackin_preview/screens/config.rs`:22; `src/bin/jackin_preview/screens/accounts.rs`:25; `src/bin/jackin_preview/screens/editor.rs`:21; `src/bin/jackin_preview/screens/modals.rs`:23.
- holla: `src/bin/holla/screens/review.rs`:17.

### `src/widgets/splitter.rs`

Hover/drag; resize geometry from Split. Keyboard resizing belongs to owning composition.

- showcase: `src/bin/showcase/pages/terminal.rs`:17.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/screens/accounts.rs`:26; `src/bin/jackin_preview/screens/manager.rs`:24.
- holla: no direct source reference.

### `src/widgets/statusbar.rs`

Left/center/right groups, drop priority, normal/strong/chip emphasis, busy and meter, optional clickable items.

- showcase: `src/bin/showcase/pages/chrome.rs`:21.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:27.
- holla: `src/bin/holla/screens/activity.rs`:15; `src/bin/holla/screens/review.rs`:18; `src/bin/holla/screens/finder.rs`:16; `src/bin/holla/screens/disk.rs`:21; `src/bin/holla/screens/plan.rs`:19; `src/bin/holla/screens/mod.rs`:23; `src/bin/holla/screens/snapshot.rs`:18; `src/bin/holla/app.rs`:27.

### `src/widgets/steps.rs`

Queued/running/done/skipped/failed/blocked, frontier/counts, selectable cursor, metadata, scrolling.

- showcase: `src/bin/showcase/pages/terminal.rs`:18.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/screens/cockpit.rs`:16; `src/bin/jackin_preview/sim/launch.rs`:5.
- holla: no direct source reference.

### `src/widgets/table.rs`

Row/cell navigation, selection, header sort, numeric alignment/sort, editing/error, empty, horizontal/vertical scroll; keyboard/click/paste.

- showcase: `src/bin/showcase/pages/editable.rs`:10; `src/bin/showcase/pages/tables.rs`:11,105; `src/bin/showcase/pages/settings.rs`:19.
- tablepro: `src/bin/tablepro/tabs.rs`:30; `src/bin/tablepro/workbench.rs`:15.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/tabs.rs`

Active/cursor, closable/new, prefix/suffix, hidden window/overflow controls; keyboard and click.

- showcase: `src/bin/showcase/pages/settings.rs`:20.
- tablepro: `src/bin/tablepro/connections.rs`:19; `src/bin/tablepro/tabs.rs`:31; `src/bin/tablepro/workbench.rs`:16.
- jackin_preview: `src/bin/jackin_preview/screens/capsule.rs`:28; `src/bin/jackin_preview/screens/editor.rs`:22; `src/bin/jackin_preview/screens/settings.rs`:18.
- holla: `src/bin/holla/app.rs`:28.

### `src/widgets/textarea.rs`

Navigation/editing, placeholder/disabled/error/help, multiline selection and scrolling; key/click/wheel/paste. Esc commits by contract.

- showcase: `src/bin/showcase/pages/settings.rs`:21; `src/bin/showcase/pages/forms.rs`:12; `src/bin/showcase/pages/textareas.rs`:11.
- tablepro: `src/bin/tablepro/connections.rs`:81,198.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/tree.rs`

Expanded/collapsed, lazy/busy, leaf/note, filtered/revealed, selected/cursor, empty/overflow; keyboard, row/toggle click, wheel, scrollbar.

- showcase: `src/bin/showcase/data.rs`:4; `src/bin/showcase/pages/taskrunner.rs`:17,18; `src/bin/showcase/pages/pickers.rs`:16; `src/bin/showcase/pages/trees.rs`:10.
- tablepro: `src/bin/tablepro/connections.rs`:20; `src/bin/tablepro/tabs.rs`:32; `src/bin/tablepro/workbench.rs`:17.
- jackin_preview: no direct source reference.
- holla: no direct source reference.

### `src/widgets/viewport.rs`

Wrapped/unwrapped, follow/pause, bounded scrollback, selection/copy, word selection, vertical/horizontal scrolling; key/click/drag/wheel/scrollbar.

- showcase: `src/bin/showcase/pages/terminal.rs`:19.
- tablepro: no direct source reference.
- jackin_preview: `src/bin/jackin_preview/screens/cockpit.rs`:17; `src/bin/jackin_preview/sim/pty.rs`:10,1314; `src/bin/jackin_preview/domain/fixtures.rs`:1528,1534,1539,1544.
- holla: `src/bin/holla/screens/activity.rs`:16; `src/bin/holla/screens/plan.rs`:20,1056.

## Original showcase coverage (22 pages)

All 22 entries are registered in `src/bin/showcase/app.rs:29` and `NAV_ENTRIES`; modules implement `Page` from `src/bin/showcase/pages/mod.rs:102`. The shell supplies focus routing, contextual hints and modal requests. Page names below are source stems.

| Page | Direct component modules |
|---|---|
| buttons | button, panel |
| chips | chips, empty, panel, props, segments, select |
| chrome | brand, hintbar, keyhint, list, menu, panel, statusbar |
| dialogs | button, dialog, input, panel |
| editable | panel, scrollbar, table |
| editor | code, completion, empty, panel, props, scrollbar |
| forms | button, choice, input, panel, textarea |
| grid | button, dialog, grid, panel, props |
| inputs | input, panel |
| lists | list, panel, scrollbar |
| overview | panel |
| panels | list, panel, scrollbar |
| pickers | button, panel, picker, props, tree |
| progress | button, panel, progress |
| scrolling | list, panel, scrollbar |
| settings | button, choice, dialog, input, list, panel, scrollbar, table, tabs, textarea |
| sidebars | button, panel |
| tables | panel, scrollbar, table |
| taskrunner | button, dialog, panel, progress, scrollbar, tree |
| terminal | button, panel, scrollbar, splitter, steps, viewport |
| textareas | input, panel, scrollbar, textarea |
| trees | panel, scrollbar, tree |

Composition coverage: Forms combines inputs, text areas and choices; Settings combines field editing, members table and environment list; TaskRunner combines tree, progress and log panel; Terminal combines selectable viewport, step rail and splitter; Chrome combines menus, status and hints; Editor combines code, diagnostics and completion. Overview and Sidebars additionally draw app-level composition directly. Existing visual baseline covers pages at 120×40 and 80×24; it does not prove all interactive states or all four color levels.

## Original API findings and boundaries

| ID | Classification / priority | Exact evidence | Impact and proposed fix | Risk / verification |
|---|---|---|---|---|
| API-1 | Confirmed defect / P1 | `src/widgets/table.rs:206` replaces rows/order without clearing `edit`; `commit_edit` at L306 indexes `order[e.row]`. | Replacing rows during editing retains a transaction attached to the previous dataset. Empty replacement panics on commit; same-sized replacement overwrites an unrelated row. Reset row-bound edit and selection state at `set_rows`. | Low: document replacement semantics. Regress replacement with empty, shorter and same-sized data; preserve sort and valid cursor behavior. |
| API-2 | Architecture/API weakness / P2 | `src/widgets/list.rs:47` exposes separate `items` and `checked` vectors plus a private range anchor. `settings.rs:515-516` manually appends both; L579-592 manually removes items and resets checks/cursor. | Application must preserve library invariants without access to all dependent state. Add bounded list replacement/mutation APIs and use them at callers; preserve semantic selection only when explicitly supported. No broad collection framework is justified. | Medium if preserving selection identity; low for documented replacement/reset. Test growth/shrink after range selection, disabled rows and scroll clamping. |
| API-3 | Coverage gap / P2 | `src/widgets/diff.rs:1` implements DiffView; `src/bin/jackin_preview/screens/inspect.rs:14-15` uses it; none of 22 showcase pages imports diff. | Existing reusable capability lacks component-level discovery and visual baseline. Add a showcase composition using the existing viewer, mode controls and selection/copy hints. | Low library risk. Verify unified/split, narrow fallback, key/mouse behavior, no-color and captures. |
| API-4 | Design inconsistency / P2 | `DESIGN.md:1250` says no context menu or diff viewer. Menu catalogue exists at L966, `widgets/menu.rs:72` implements ContextMenu; DiffView exists. | Contract contradicts implementation, encouraging duplicate widgets or incorrect audits. Correct absence statement and document DiffView. README architecture list also omits later modules. | Low: reconcile catalogue against exports, pages and actual apps. |
| API-5 | Architecture/API weakness / P2 | Mouse/focus bookkeeping repeated in showcase `app.rs:576`, tablepro L1858, jackin-preview L1534, holla L1191; all maintain hover suppression and down/up dispatch. | Shared policy is maintained in four shells. Different app semantics make an immediate generic Widget trait insufficient. First extract only a proven common policy when a reproduced cross-app defect establishes required behavior. | Medium/high for broad routing change; retain as backlog until focused interaction evidence supports extraction. |

`README.md:346-351` proposes a Widget trait, Theme trait and configurable glyphs. These are hypotheses. Existing widgets deliberately return typed events (InputEvent, GridEvent, TreeEvent, etc.), while `Page` already provides a composition-level trait. Forcing them into one dynamic trait would require event erasure or application callbacks without demonstrated need. `Theme` already has public token fields and component style resolvers. A second visual theme may test assumptions but is not required to repair a verified defect. Fixed glyphs encode the intentional Junie grammar; no real caller currently proves that customization is missing.

Capabilities already available include code editing, diff viewing, bounded selectable scrollback, context menus, semantic meters, split resizing, typed acknowledgement, copyable facts and execution steps. No new component is justified by this inventory alone. New proposals must identify a missing reusable capability beyond these primitives.

## Final showcase coverage (23 pages)

Current registration: `src/bin/showcase/app.rs:31` (`PageId`) and `src/bin/showcase/app.rs:63` (`NAV_ENTRIES`). Page contract: `src/bin/showcase/pages/mod.rs:108`. Each page below exists as a source module; the table records its direct component imports.

| Page source | Direct component modules |
|---|---|
| `src/bin/showcase/pages/buttons.rs` | button, panel |
| `src/bin/showcase/pages/chips.rs` | chips, empty, panel, props, segments, select |
| `src/bin/showcase/pages/chrome.rs` | brand, hintbar, keyhint, list, menu, panel, statusbar |
| `src/bin/showcase/pages/dialogs.rs` | button, dialog, input, panel |
| `src/bin/showcase/pages/diff.rs` | button, diff, panel, scrollbar, viewport |
| `src/bin/showcase/pages/editable.rs` | panel, scrollbar, table |
| `src/bin/showcase/pages/editor.rs` | code, completion, empty, panel, props, scrollbar |
| `src/bin/showcase/pages/forms.rs` | button, choice, input, panel, textarea |
| `src/bin/showcase/pages/grid.rs` | button, dialog, grid, panel, props |
| `src/bin/showcase/pages/inputs.rs` | input, panel |
| `src/bin/showcase/pages/lists.rs` | list, panel, scrollbar |
| `src/bin/showcase/pages/overview.rs` | panel |
| `src/bin/showcase/pages/panels.rs` | list, panel, scrollbar |
| `src/bin/showcase/pages/pickers.rs` | button, panel, picker, props, tree |
| `src/bin/showcase/pages/progress.rs` | button, panel, progress |
| `src/bin/showcase/pages/scrolling.rs` | list, panel, scrollbar |
| `src/bin/showcase/pages/settings.rs` | button, choice, dialog, input, list, panel, scrollbar, table, tabs, textarea |
| `src/bin/showcase/pages/sidebars.rs` | button, panel |
| `src/bin/showcase/pages/tables.rs` | panel, scrollbar, table |
| `src/bin/showcase/pages/taskrunner.rs` | button, dialog, panel, progress, scrollbar, tree |
| `src/bin/showcase/pages/terminal.rs` | button, panel, scrollbar, splitter, steps, viewport |
| `src/bin/showcase/pages/textareas.rs` | input, panel, scrollbar, textarea |
| `src/bin/showcase/pages/trees.rs` | panel, scrollbar, tree |

Diff is the added page, using existing DiffView with unified/review/empty states, mode controls, narrow fallback, selection/copy and scrolling. The viewer is also consumed by `src/bin/jackin_preview/screens/inspect.rs`; its model remains in `sim/changes.rs`. Shared `PageEvent::Press` now carries the original pointer position to Diff and Terminal. No new widget module was added.

API-1 is fixed at dataset replacement and table edit identity boundaries. API-3 is covered by the Diff page. API-4 is reconciled in DESIGN and README. API-2 remains a documented collection-invariant weakness, and API-5 remains a broader shell-ownership opportunity after the narrow pointer-down fix. These statements distinguish original findings from final state; the original evidence above must not be read as claiming fixed behavior remains defective.

## Final exported API appendix

Entries are grouped by declaration file. Names omit only the common `junie_tui::` prefix. `struct_field` includes named public fields and enum payload slots. Module spans identify their module source file; associated methods identify their complete owning type. All declaration positions below come from the final Rustdoc export graph.

### `src/core/event.rs` (42 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::event` | module | `src/core/event.rs:1` |
| `core::event::Outcome` | enum | `src/core/event.rs:14` |
| `core::event::Outcome::Ignored` | variant | `src/core/event.rs:17` |
| `core::event::Outcome::Consumed` | variant | `src/core/event.rs:19` |
| `core::event::Outcome::Changed` | variant | `src/core/event.rs:21` |
| `core::event::Outcome::consumed` | function | `src/core/event.rs:25` |
| `core::event::Outcome::or` | function | `src/core/event.rs:30` |
| `core::event::Input` | enum | `src/core/event.rs:40` |
| `core::event::Input::Key` | variant | `src/core/event.rs:41` |
| `core::event::Input::Key::0` | struct_field | `src/core/event.rs:41` |
| `core::event::Input::Mouse` | variant | `src/core/event.rs:42` |
| `core::event::Input::Mouse::0` | struct_field | `src/core/event.rs:42` |
| `core::event::Input::Resize` | variant | `src/core/event.rs:43` |
| `core::event::Input::Resize::0` | struct_field | `src/core/event.rs:43` |
| `core::event::Input::Resize::1` | struct_field | `src/core/event.rs:43` |
| `core::event::Input::Paste` | variant | `src/core/event.rs:44` |
| `core::event::Input::Paste::0` | struct_field | `src/core/event.rs:44` |
| `core::event::Input::Tick` | variant | `src/core/event.rs:45` |
| `core::event::Key` | struct | `src/core/event.rs:49` |
| `core::event::Key::code` | struct_field | `src/core/event.rs:50` |
| `core::event::Key::mods` | struct_field | `src/core/event.rs:51` |
| `core::event::Key::ctrl` | function | `src/core/event.rs:55` |
| `core::event::Key::shift` | function | `src/core/event.rs:58` |
| `core::event::Key::alt` | function | `src/core/event.rs:61` |
| `core::event::Key::plain` | function | `src/core/event.rs:64` |
| `core::event::Key::is` | function | `src/core/event.rs:67` |
| `core::event::Key::is_char` | function | `src/core/event.rs:70` |
| `core::event::Key::ctrl_char` | function | `src/core/event.rs:73` |
| `core::event::MouseKind` | enum | `src/core/event.rs:79` |
| `core::event::MouseKind::Move` | variant | `src/core/event.rs:80` |
| `core::event::MouseKind::Down` | variant | `src/core/event.rs:81` |
| `core::event::MouseKind::Up` | variant | `src/core/event.rs:82` |
| `core::event::MouseKind::Drag` | variant | `src/core/event.rs:83` |
| `core::event::MouseKind::Secondary` | variant | `src/core/event.rs:85` |
| `core::event::MouseKind::WheelUp` | variant | `src/core/event.rs:86` |
| `core::event::MouseKind::WheelDown` | variant | `src/core/event.rs:87` |
| `core::event::MouseKind::WheelLeft` | variant | `src/core/event.rs:88` |
| `core::event::MouseKind::WheelRight` | variant | `src/core/event.rs:89` |
| `core::event::Mouse` | struct | `src/core/event.rs:93` |
| `core::event::Mouse::kind` | struct_field | `src/core/event.rs:94` |
| `core::event::Mouse::pos` | struct_field | `src/core/event.rs:95` |
| `core::event::Input::from_crossterm` | function | `src/core/event.rs:99` |

### `src/core/focus.rs` (17 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::focus` | module | `src/core/focus.rs:1` |
| `core::focus::FocusRing` | struct | `src/core/focus.rs:10` |
| `core::focus::FocusRing::register` | function | `src/core/focus.rs:16` |
| `core::focus::FocusRing::push_barrier` | function | `src/core/focus.rs:20` |
| `core::focus::FocusRing::reachable` | function | `src/core/focus.rs:24` |
| `core::focus::FocusRing::contains` | function | `src/core/focus.rs:31` |
| `core::focus::FocusRing::first` | function | `src/core/focus.rs:35` |
| `core::focus::FocusRing::next` | function | `src/core/focus.rs:39` |
| `core::focus::FocusRing::prev` | function | `src/core/focus.rs:50` |
| `core::focus::Focus` | struct | `src/core/focus.rs:64` |
| `core::focus::Focus::current` | function | `src/core/focus.rs:69` |
| `core::focus::Focus::is` | function | `src/core/focus.rs:73` |
| `core::focus::Focus::set` | function | `src/core/focus.rs:77` |
| `core::focus::Focus::focus` | function | `src/core/focus.rs:81` |
| `core::focus::Focus::next` | function | `src/core/focus.rs:85` |
| `core::focus::Focus::prev` | function | `src/core/focus.rs:89` |
| `core::focus::Focus::ensure_valid` | function | `src/core/focus.rs:94` |

### `src/core/hit.rs` (14 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::hit` | module | `src/core/hit.rs:1` |
| `core::hit::HitRegion` | struct | `src/core/hit.rs:13` |
| `core::hit::HitRegion::id` | struct_field | `src/core/hit.rs:14` |
| `core::hit::HitRegion::area` | struct_field | `src/core/hit.rs:15` |
| `core::hit::HitRegion::scroll_only` | struct_field | `src/core/hit.rs:18` |
| `core::hit::HitRegistry` | struct | `src/core/hit.rs:22` |
| `core::hit::HitRegistry::register` | function | `src/core/hit.rs:30` |
| `core::hit::HitRegistry::register_scroll` | function | `src/core/hit.rs:41` |
| `core::hit::HitRegistry::push_barrier` | function | `src/core/hit.rs:53` |
| `core::hit::HitRegistry::hit` | function | `src/core/hit.rs:65` |
| `core::hit::HitRegistry::hit_scroll` | function | `src/core/hit.rs:75` |
| `core::hit::HitRegistry::len` | function | `src/core/hit.rs:83` |
| `core::hit::HitRegistry::is_empty` | function | `src/core/hit.rs:87` |
| `core::hit::HitRegistry::area_of` | function | `src/core/hit.rs:91` |

### `src/core/id.rs` (7 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::id` | module | `src/core/id.rs:1` |
| `core::id::WidgetId` | struct | `src/core/id.rs:10` |
| `core::id::WidgetId::of` | function | `src/core/id.rs:27` |
| `core::id::WidgetId::child` | function | `src/core/id.rs:32` |
| `core::id::WidgetId::sub` | function | `src/core/id.rs:37` |
| `core::id::WidgetId as Debug` | trait_impl | `src/core/id.rs:42` |
| `core::id::WidgetId as Debug::fmt` | function | `src/core/id.rs:43` |

### `src/core/mod.rs` (1 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core` | module | `src/core/mod.rs:1` |

### `src/core/scroll.rs` (21 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::scroll` | module | `src/core/scroll.rs:1` |
| `core::scroll::ScrollState` | struct | `src/core/scroll.rs:8` |
| `core::scroll::ScrollState::offset` | struct_field | `src/core/scroll.rs:9` |
| `core::scroll::ScrollState::content_len` | struct_field | `src/core/scroll.rs:10` |
| `core::scroll::ScrollState::viewport_len` | struct_field | `src/core/scroll.rs:11` |
| `core::scroll::ScrollState::new` | function | `src/core/scroll.rs:15` |
| `core::scroll::ScrollState::max_offset` | function | `src/core/scroll.rs:23` |
| `core::scroll::ScrollState::overflows` | function | `src/core/scroll.rs:27` |
| `core::scroll::ScrollState::set_viewport` | function | `src/core/scroll.rs:31` |
| `core::scroll::ScrollState::set_content` | function | `src/core/scroll.rs:36` |
| `core::scroll::ScrollState::clamp` | function | `src/core/scroll.rs:41` |
| `core::scroll::ScrollState::scroll_by` | function | `src/core/scroll.rs:45` |
| `core::scroll::ScrollState::scroll_to` | function | `src/core/scroll.rs:52` |
| `core::scroll::ScrollState::page_up` | function | `src/core/scroll.rs:56` |
| `core::scroll::ScrollState::page_down` | function | `src/core/scroll.rs:60` |
| `core::scroll::ScrollState::jump_start` | function | `src/core/scroll.rs:64` |
| `core::scroll::ScrollState::jump_end` | function | `src/core/scroll.rs:68` |
| `core::scroll::ScrollState::ensure_visible` | function | `src/core/scroll.rs:73` |
| `core::scroll::ScrollState::visible_range` | function | `src/core/scroll.rs:86` |
| `core::scroll::ScrollState::thumb` | function | `src/core/scroll.rs:92` |
| `core::scroll::ScrollState::offset_for_track_pos` | function | `src/core/scroll.rs:107` |

### `src/core/text.rs` (42 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `core::text` | module | `src/core/text.rs:1` |
| `core::text::TextBuffer` | struct | `src/core/text.rs:33` |
| `core::text::CursorPos` | struct | `src/core/text.rs:43` |
| `core::text::CursorPos::line` | struct_field | `src/core/text.rs:44` |
| `core::text::CursorPos::col` | struct_field | `src/core/text.rs:46` |
| `core::text::TextBuffer::single` | function | `src/core/text.rs:50` |
| `core::text::TextBuffer::multi` | function | `src/core/text.rs:60` |
| `core::text::TextBuffer::text` | function | `src/core/text.rs:70` |
| `core::text::TextBuffer::is_empty` | function | `src/core/text.rs:74` |
| `core::text::TextBuffer::cursor_offset` | function | `src/core/text.rs:83` |
| `core::text::TextBuffer::select_range` | function | `src/core/text.rs:89` |
| `core::text::TextBuffer::selection_lines` | function | `src/core/text.rs:95` |
| `core::text::TextBuffer::has_selection_lines` | function | `src/core/text.rs:112` |
| `core::text::TextBuffer::insert_at` | function | `src/core/text.rs:119` |
| `core::text::TextBuffer::remove_range` | function | `src/core/text.rs:136` |
| `core::text::TextBuffer::set_text` | function | `src/core/text.rs:158` |
| `core::text::TextBuffer::selection` | function | `src/core/text.rs:164` |
| `core::text::TextBuffer::select_all` | function | `src/core/text.rs:177` |
| `core::text::TextBuffer::clear_selection` | function | `src/core/text.rs:182` |
| `core::text::TextBuffer::move_left` | function | `src/core/text.rs:301` |
| `core::text::TextBuffer::move_right` | function | `src/core/text.rs:311` |
| `core::text::TextBuffer::move_word_left` | function | `src/core/text.rs:321` |
| `core::text::TextBuffer::move_word_right` | function | `src/core/text.rs:326` |
| `core::text::TextBuffer::move_home` | function | `src/core/text.rs:331` |
| `core::text::TextBuffer::move_end` | function | `src/core/text.rs:336` |
| `core::text::TextBuffer::move_doc_start` | function | `src/core/text.rs:341` |
| `core::text::TextBuffer::move_doc_end` | function | `src/core/text.rs:346` |
| `core::text::TextBuffer::move_up` | function | `src/core/text.rs:352` |
| `core::text::TextBuffer::move_down` | function | `src/core/text.rs:365` |
| `core::text::TextBuffer::set_cursor_line_col` | function | `src/core/text.rs:378` |
| `core::text::TextBuffer::insert_char` | function | `src/core/text.rs:401` |
| `core::text::TextBuffer::insert_str` | function | `src/core/text.rs:409` |
| `core::text::TextBuffer::backspace` | function | `src/core/text.rs:424` |
| `core::text::TextBuffer::delete` | function | `src/core/text.rs:434` |
| `core::text::TextBuffer::delete_word_left` | function | `src/core/text.rs:443` |
| `core::text::TextBuffer::delete_to_line_end` | function | `src/core/text.rs:453` |
| `core::text::TextBuffer::delete_to_line_start` | function | `src/core/text.rs:462` |
| `core::text::TextBuffer::line_count` | function | `src/core/text.rs:474` |
| `core::text::TextBuffer::cursor_pos` | function | `src/core/text.rs:479` |
| `core::text::TextBuffer::pos_of` | function | `src/core/text.rs:485` |
| `core::text::TextBuffer::offset_at` | function | `src/core/text.rs:495` |
| `core::text::TextBuffer::width` | function | `src/core/text.rs:515` |

### `src/runtime.rs` (12 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `runtime` | module | `src/runtime.rs:1` |
| `runtime::Application` | trait | `src/runtime.rs:26` |
| `runtime::Application::handle` | function | `src/runtime.rs:27` |
| `runtime::Application::render` | function | `src/runtime.rs:28` |
| `runtime::Application::should_quit` | function | `src/runtime.rs:29` |
| `runtime::Application::tick_interval` | function | `src/runtime.rs:31` |
| `runtime::TerminalSession` | struct | `src/runtime.rs:39` |
| `runtime::TerminalSession::enter` | function | `src/runtime.rs:88` |
| `runtime::TerminalSession::terminal` | function | `src/runtime.rs:112` |
| `runtime::TerminalSession::leave` | function | `src/runtime.rs:117` |
| `runtime::run` | function | `src/runtime.rs:133` |
| `runtime::drain_pending_input` | function | `src/runtime.rs:232` |

### `src/theme.rs` (95 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `theme` | module | `src/theme.rs:1` |
| `theme::ColorLevel` | enum | `src/theme.rs:22` |
| `theme::ColorLevel::TrueColor` | variant | `src/theme.rs:23` |
| `theme::ColorLevel::Ansi256` | variant | `src/theme.rs:24` |
| `theme::ColorLevel::Ansi16` | variant | `src/theme.rs:25` |
| `theme::ColorLevel::Mono` | variant | `src/theme.rs:26` |
| `theme::ColorLevel::detect` | function | `src/theme.rs:30` |
| `theme::ColorLevel::label` | function | `src/theme.rs:45` |
| `theme::Theme` | struct | `src/theme.rs:99` |
| `theme::Theme::level` | struct_field | `src/theme.rs:100` |
| `theme::Theme::canvas` | struct_field | `src/theme.rs:102` |
| `theme::Theme::surface` | struct_field | `src/theme.rs:103` |
| `theme::Theme::surface_elevated` | struct_field | `src/theme.rs:104` |
| `theme::Theme::surface_overlay` | struct_field | `src/theme.rs:105` |
| `theme::Theme::field` | struct_field | `src/theme.rs:106` |
| `theme::Theme::field_hover` | struct_field | `src/theme.rs:107` |
| `theme::Theme::popover` | struct_field | `src/theme.rs:108` |
| `theme::Theme::highlight` | struct_field | `src/theme.rs:111` |
| `theme::Theme::highlight_danger` | struct_field | `src/theme.rs:113` |
| `theme::Theme::error_soft` | struct_field | `src/theme.rs:116` |
| `theme::Theme::border_subtle` | struct_field | `src/theme.rs:118` |
| `theme::Theme::border_strong` | struct_field | `src/theme.rs:119` |
| `theme::Theme::text_primary` | struct_field | `src/theme.rs:121` |
| `theme::Theme::text_secondary` | struct_field | `src/theme.rs:122` |
| `theme::Theme::text_muted` | struct_field | `src/theme.rs:123` |
| `theme::Theme::text_faint` | struct_field | `src/theme.rs:124` |
| `theme::Theme::text_ghost` | struct_field | `src/theme.rs:126` |
| `theme::Theme::text_on_accent` | struct_field | `src/theme.rs:127` |
| `theme::Theme::accent` | struct_field | `src/theme.rs:129` |
| `theme::Theme::accent_hover` | struct_field | `src/theme.rs:130` |
| `theme::Theme::accent_pressed` | struct_field | `src/theme.rs:131` |
| `theme::Theme::accent_bg` | struct_field | `src/theme.rs:132` |
| `theme::Theme::accent_bg_subtle` | struct_field | `src/theme.rs:133` |
| `theme::Theme::focus` | struct_field | `src/theme.rs:134` |
| `theme::Theme::disabled` | struct_field | `src/theme.rs:136` |
| `theme::Theme::error` | struct_field | `src/theme.rs:137` |
| `theme::Theme::error_bg` | struct_field | `src/theme.rs:138` |
| `theme::Theme::warning` | struct_field | `src/theme.rs:139` |
| `theme::Theme::success` | struct_field | `src/theme.rs:140` |
| `theme::Theme::info` | struct_field | `src/theme.rs:141` |
| `theme::Theme::junie` | function | `src/theme.rs:145` |
| `theme::Theme::for_level` | function | `src/theme.rs:183` |
| `theme::Theme::base` | function | `src/theme.rs:229` |
| `theme::Theme::on` | function | `src/theme.rs:233` |
| `theme::Theme::primary` | function | `src/theme.rs:237` |
| `theme::Theme::secondary` | function | `src/theme.rs:241` |
| `theme::Theme::muted` | function | `src/theme.rs:245` |
| `theme::Theme::faint` | function | `src/theme.rs:249` |
| `theme::Theme::accent_fg` | function | `src/theme.rs:253` |
| `theme::Theme::error_fg` | function | `src/theme.rs:257` |
| `theme::Theme::title` | function | `src/theme.rs:261` |
| `theme::Theme::label` | function | `src/theme.rs:267` |
| `theme::Theme::key_hint_key` | function | `src/theme.rs:275` |
| `theme::Theme::key_hint_action` | function | `src/theme.rs:281` |
| `theme::Theme::border` | function | `src/theme.rs:285` |
| `theme::Theme::backdrop` | function | `src/theme.rs:297` |
| `theme::Theme::disabled_style` | function | `src/theme.rs:330` |
| `theme::Theme::row` | function | `src/theme.rs:340` |
| `theme::Theme::lift` | function | `src/theme.rs:373` |
| `theme::Theme::gutter_symbol` | function | `src/theme.rs:391` |
| `theme::Theme::gutter` | function | `src/theme.rs:397` |
| `theme::Theme::button` | function | `src/theme.rs:408` |
| `theme::Theme::field_style` | function | `src/theme.rs:473` |
| `theme::Theme::placeholder` | function | `src/theme.rs:485` |
| `theme::Theme::selection` | function | `src/theme.rs:494` |
| `theme::Theme::scrollbar_track` | function | `src/theme.rs:503` |
| `theme::Theme::scrollbar_thumb` | function | `src/theme.rs:507` |
| `theme::Theme::tone` | function | `src/theme.rs:517` |
| `theme::Theme::syntax` | function | `src/theme.rs:531` |
| `theme::Theme::badge` | function | `src/theme.rs:546` |
| `theme::Tone` | enum | `src/theme.rs:559` |
| `theme::Tone::Normal` | variant | `src/theme.rs:561` |
| `theme::Tone::Secondary` | variant | `src/theme.rs:562` |
| `theme::Tone::Muted` | variant | `src/theme.rs:563` |
| `theme::Tone::Faint` | variant | `src/theme.rs:564` |
| `theme::Tone::Error` | variant | `src/theme.rs:565` |
| `theme::Tone::Warning` | variant | `src/theme.rs:566` |
| `theme::Tone::Success` | variant | `src/theme.rs:567` |
| `theme::SyntaxTone` | enum | `src/theme.rs:572` |
| `theme::SyntaxTone::Keyword` | variant | `src/theme.rs:573` |
| `theme::SyntaxTone::Ident` | variant | `src/theme.rs:574` |
| `theme::SyntaxTone::Number` | variant | `src/theme.rs:575` |
| `theme::SyntaxTone::Str` | variant | `src/theme.rs:576` |
| `theme::SyntaxTone::Operator` | variant | `src/theme.rs:577` |
| `theme::SyntaxTone::Punct` | variant | `src/theme.rs:578` |
| `theme::SyntaxTone::Comment` | variant | `src/theme.rs:579` |
| `theme::SyntaxTone::Plain` | variant | `src/theme.rs:580` |
| `theme::ButtonKind` | enum | `src/theme.rs:584` |
| `theme::ButtonKind::Primary` | variant | `src/theme.rs:585` |
| `theme::ButtonKind::Secondary` | variant | `src/theme.rs:586` |
| `theme::ButtonKind::Subtle` | variant | `src/theme.rs:587` |
| `theme::ButtonKind::Danger` | variant | `src/theme.rs:588` |
| `theme::ButtonKind::Toggle` | variant | `src/theme.rs:589` |
| `theme::BadgeKind` | enum | `src/theme.rs:593` |
| `theme::BadgeKind::Edit` | variant | `src/theme.rs:594` |

### `src/ui/ctx.rs` (36 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `ui::ctx` | module | `src/ui/ctx.rs:1` |
| `ui::ctx::Interaction` | struct | `src/ui/ctx.rs:17` |
| `ui::ctx::Interaction::focus` | struct_field | `src/ui/ctx.rs:18` |
| `ui::ctx::Interaction::hover` | struct_field | `src/ui/ctx.rs:19` |
| `ui::ctx::Interaction::pressed` | struct_field | `src/ui/ctx.rs:20` |
| `ui::ctx::Interaction::flash` | struct_field | `src/ui/ctx.rs:22` |
| `ui::ctx::Interaction::focus_hidden` | struct_field | `src/ui/ctx.rs:24` |
| `ui::ctx::Interaction::hover_suppressed` | struct_field | `src/ui/ctx.rs:27` |
| `ui::ctx::Interaction::tick` | struct_field | `src/ui/ctx.rs:28` |
| `ui::ctx::Interaction::focused` | function | `src/ui/ctx.rs:32` |
| `ui::ctx::Interaction::hovered` | function | `src/ui/ctx.rs:35` |
| `ui::ctx::Interaction::pressed` | function | `src/ui/ctx.rs:38` |
| `ui::ctx::VisualState` | struct | `src/ui/ctx.rs:45` |
| `ui::ctx::VisualState::focused` | struct_field | `src/ui/ctx.rs:46` |
| `ui::ctx::VisualState::hovered` | struct_field | `src/ui/ctx.rs:47` |
| `ui::ctx::VisualState::pressed` | struct_field | `src/ui/ctx.rs:48` |
| `ui::ctx::VisualState::selected` | struct_field | `src/ui/ctx.rs:49` |
| `ui::ctx::VisualState::disabled` | struct_field | `src/ui/ctx.rs:50` |
| `ui::ctx::VisualState::error` | struct_field | `src/ui/ctx.rs:51` |
| `ui::ctx::VisualState::editing` | struct_field | `src/ui/ctx.rs:52` |
| `ui::ctx::VisualState::busy` | struct_field | `src/ui/ctx.rs:53` |
| `ui::ctx::RenderCtx` | struct | `src/ui/ctx.rs:56` |
| `ui::ctx::RenderCtx::theme` | struct_field | `src/ui/ctx.rs:57` |
| `ui::ctx::RenderCtx::interaction` | struct_field | `src/ui/ctx.rs:58` |
| `ui::ctx::RenderCtx::hits` | struct_field | `src/ui/ctx.rs:59` |
| `ui::ctx::RenderCtx::ring` | struct_field | `src/ui/ctx.rs:60` |
| `ui::ctx::RenderCtx::cursor` | struct_field | `src/ui/ctx.rs:62` |
| `ui::ctx::RenderCtx::inert` | struct_field | `src/ui/ctx.rs:65` |
| `ui::ctx::RenderCtx::new` | function | `src/ui/ctx.rs:69` |
| `ui::ctx::RenderCtx::control` | function | `src/ui/ctx.rs:86` |
| `ui::ctx::RenderCtx::clickable` | function | `src/ui/ctx.rs:97` |
| `ui::ctx::RenderCtx::scrollable` | function | `src/ui/ctx.rs:104` |
| `ui::ctx::RenderCtx::state` | function | `src/ui/ctx.rs:110` |
| `ui::ctx::RenderCtx::set_cursor` | function | `src/ui/ctx.rs:119` |
| `ui::ctx::RenderCtx::begin_modal` | function | `src/ui/ctx.rs:126` |
| `ui::ctx::fill` | function | `src/ui/ctx.rs:135` |

### `src/ui/layout.rs` (22 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `ui::layout` | module | `src/ui/layout.rs:1` |
| `ui::layout::SplitDir` | enum | `src/ui/layout.rs:6` |
| `ui::layout::SplitDir::Horizontal` | variant | `src/ui/layout.rs:8` |
| `ui::layout::SplitDir::Vertical` | variant | `src/ui/layout.rs:10` |
| `ui::layout::Maximized` | enum | `src/ui/layout.rs:14` |
| `ui::layout::Maximized::None` | variant | `src/ui/layout.rs:15` |
| `ui::layout::Maximized::First` | variant | `src/ui/layout.rs:16` |
| `ui::layout::Maximized::Second` | variant | `src/ui/layout.rs:17` |
| `ui::layout::Split` | struct | `src/ui/layout.rs:23` |
| `ui::layout::Split::percent` | struct_field | `src/ui/layout.rs:24` |
| `ui::layout::Split::min_first` | struct_field | `src/ui/layout.rs:25` |
| `ui::layout::Split::min_second` | struct_field | `src/ui/layout.rs:26` |
| `ui::layout::Split::maximized` | struct_field | `src/ui/layout.rs:27` |
| `ui::layout::Split::new` | function | `src/ui/layout.rs:31` |
| `ui::layout::Split::toggle_max` | function | `src/ui/layout.rs:40` |
| `ui::layout::Split::grow` | function | `src/ui/layout.rs:48` |
| `ui::layout::Split::layout` | function | `src/ui/layout.rs:53` |
| `ui::layout::Split::handle` | function | `src/ui/layout.rs:62` |
| `ui::layout::Split::drag_to` | function | `src/ui/layout.rs:75` |
| `ui::layout::Split::nudge` | function | `src/ui/layout.rs:95` |
| `ui::layout::Split::vertical` | function | `src/ui/layout.rs:110` |
| `ui::layout::Split::horizontal` | function | `src/ui/layout.rs:131` |

### `src/ui/mod.rs` (1 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `ui` | module | `src/ui/mod.rs:1` |

### `src/ui/popup.rs` (6 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `ui::popup` | module | `src/ui/popup.rs:1` |
| `ui::popup::Placement` | enum | `src/ui/popup.rs:16` |
| `ui::popup::Placement::Below` | variant | `src/ui/popup.rs:18` |
| `ui::popup::Placement::Center` | variant | `src/ui/popup.rs:20` |
| `ui::popup::place` | function | `src/ui/popup.rs:25` |
| `ui::popup::surface` | function | `src/ui/popup.rs:61` |

### `src/ui/text.rs` (10 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `ui::text` | module | `src/ui/text.rs:1` |
| `ui::text::width` | function | `src/ui/text.rs:20` |
| `ui::text::truncate` | function | `src/ui/text.rs:25` |
| `ui::text::slice_cells` | function | `src/ui/text.rs:57` |
| `ui::text::truncate_middle` | function | `src/ui/text.rs:99` |
| `ui::text::thousands` | function | `src/ui/text.rs:133` |
| `ui::text::fit` | function | `src/ui/text.rs:146` |
| `ui::text::fit_right` | function | `src/ui/text.rs:153` |
| `ui::text::wrap` | function | `src/ui/text.rs:161` |
| `ui::text::fuzzy` | function | `src/ui/text.rs:288` |

### `src/widgets/brand.rs` (10 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::brand` | module | `src/widgets/brand.rs:1` |
| `widgets::brand::Lockup` | struct | `src/widgets/brand.rs:15` |
| `widgets::brand::Lockup::text` | struct_field | `src/widgets/brand.rs:17` |
| `widgets::brand::Lockup::compact` | struct_field | `src/widgets/brand.rs:20` |
| `widgets::brand::Lockup::new` | function | `src/widgets/brand.rs:24` |
| `widgets::brand::Lockup::compact` | function | `src/widgets/brand.rs:31` |
| `widgets::brand::Lockup::width` | function | `src/widgets/brand.rs:46` |
| `widgets::brand::Lockup::style` | function | `src/widgets/brand.rs:50` |
| `widgets::brand::Lockup::render` | function | `src/widgets/brand.rs:58` |
| `widgets::brand::Lockup::render_clickable` | function | `src/widgets/brand.rs:65` |

### `src/widgets/button.rs` (23 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::button` | module | `src/widgets/button.rs:1` |
| `widgets::button::Button` | struct | `src/widgets/button.rs:15` |
| `widgets::button::Button::id` | struct_field | `src/widgets/button.rs:16` |
| `widgets::button::Button::label` | struct_field | `src/widgets/button.rs:17` |
| `widgets::button::Button::kind` | struct_field | `src/widgets/button.rs:18` |
| `widgets::button::Button::disabled` | struct_field | `src/widgets/button.rs:19` |
| `widgets::button::Button::on` | struct_field | `src/widgets/button.rs:21` |
| `widgets::button::Button::busy` | struct_field | `src/widgets/button.rs:22` |
| `widgets::button::Button::area` | struct_field | `src/widgets/button.rs:23` |
| `widgets::button::Button::new` | function | `src/widgets/button.rs:27` |
| `widgets::button::Button::primary` | function | `src/widgets/button.rs:39` |
| `widgets::button::Button::secondary` | function | `src/widgets/button.rs:42` |
| `widgets::button::Button::subtle` | function | `src/widgets/button.rs:45` |
| `widgets::button::Button::danger` | function | `src/widgets/button.rs:48` |
| `widgets::button::Button::toggle` | function | `src/widgets/button.rs:51` |
| `widgets::button::Button::disabled` | function | `src/widgets/button.rs:56` |
| `widgets::button::Button::width` | function | `src/widgets/button.rs:62` |
| `widgets::button::Button::can_activate` | function | `src/widgets/button.rs:67` |
| `widgets::button::Button::on_key` | function | `src/widgets/button.rs:72` |
| `widgets::button::Button::on_click` | function | `src/widgets/button.rs:86` |
| `widgets::button::Button::render` | function | `src/widgets/button.rs:101` |
| `widgets::button::row_layout` | function | `src/widgets/button.rs:167` |
| `widgets::button::row_layout_right` | function | `src/widgets/button.rs:179` |

### `src/widgets/chips.rs` (33 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::chips` | module | `src/widgets/chips.rs:1` |
| `widgets::chips::Chip` | struct | `src/widgets/chips.rs:16` |
| `widgets::chips::Chip::label` | struct_field | `src/widgets/chips.rs:17` |
| `widgets::chips::Chip::enabled` | struct_field | `src/widgets/chips.rs:18` |
| `widgets::chips::Chip::removable` | struct_field | `src/widgets/chips.rs:19` |
| `widgets::chips::Chip::error` | struct_field | `src/widgets/chips.rs:20` |
| `widgets::chips::Chip::new` | function | `src/widgets/chips.rs:24` |
| `widgets::chips::ChipBar` | struct | `src/widgets/chips.rs:35` |
| `widgets::chips::ChipBar::id` | struct_field | `src/widgets/chips.rs:36` |
| `widgets::chips::ChipBar::chips` | struct_field | `src/widgets/chips.rs:37` |
| `widgets::chips::ChipBar::cursor` | struct_field | `src/widgets/chips.rs:38` |
| `widgets::chips::ChipBar::add_label` | struct_field | `src/widgets/chips.rs:39` |
| `widgets::chips::ChipBar::lead` | struct_field | `src/widgets/chips.rs:41` |
| `widgets::chips::ChipBar::area` | struct_field | `src/widgets/chips.rs:42` |
| `widgets::chips::ChipEvent` | enum | `src/widgets/chips.rs:46` |
| `widgets::chips::ChipEvent::Activate` | variant | `src/widgets/chips.rs:47` |
| `widgets::chips::ChipEvent::Activate::0` | struct_field | `src/widgets/chips.rs:47` |
| `widgets::chips::ChipEvent::Toggle` | variant | `src/widgets/chips.rs:48` |
| `widgets::chips::ChipEvent::Toggle::0` | struct_field | `src/widgets/chips.rs:48` |
| `widgets::chips::ChipEvent::Remove` | variant | `src/widgets/chips.rs:49` |
| `widgets::chips::ChipEvent::Remove::0` | struct_field | `src/widgets/chips.rs:49` |
| `widgets::chips::ChipEvent::Add` | variant | `src/widgets/chips.rs:50` |
| `widgets::chips::ChipEvent::Lead` | variant | `src/widgets/chips.rs:51` |
| `widgets::chips::ChipEvent::ClearAll` | variant | `src/widgets/chips.rs:52` |
| `widgets::chips::ChipBar::new` | function | `src/widgets/chips.rs:56` |
| `widgets::chips::ChipBar::chip_id` | function | `src/widgets/chips.rs:67` |
| `widgets::chips::ChipBar::close_id` | function | `src/widgets/chips.rs:70` |
| `widgets::chips::ChipBar::add_id` | function | `src/widgets/chips.rs:73` |
| `widgets::chips::ChipBar::lead_id` | function | `src/widgets/chips.rs:76` |
| `widgets::chips::ChipBar::on_key` | function | `src/widgets/chips.rs:85` |
| `widgets::chips::ChipBar::on_click` | function | `src/widgets/chips.rs:121` |
| `widgets::chips::ChipBar::owns` | function | `src/widgets/chips.rs:141` |
| `widgets::chips::ChipBar::render` | function | `src/widgets/chips.rs:148` |

### `src/widgets/choice.rs` (36 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::choice` | module | `src/widgets/choice.rs:1` |
| `widgets::choice::Checkbox` | struct | `src/widgets/choice.rs:31` |
| `widgets::choice::Checkbox::id` | struct_field | `src/widgets/choice.rs:32` |
| `widgets::choice::Checkbox::label` | struct_field | `src/widgets/choice.rs:33` |
| `widgets::choice::Checkbox::checked` | struct_field | `src/widgets/choice.rs:34` |
| `widgets::choice::Checkbox::disabled` | struct_field | `src/widgets/choice.rs:35` |
| `widgets::choice::Checkbox::area` | struct_field | `src/widgets/choice.rs:36` |
| `widgets::choice::Checkbox::new` | function | `src/widgets/choice.rs:40` |
| `widgets::choice::Checkbox::on_key` | function | `src/widgets/choice.rs:50` |
| `widgets::choice::Checkbox::on_click` | function | `src/widgets/choice.rs:62` |
| `widgets::choice::Checkbox::render` | function | `src/widgets/choice.rs:70` |
| `widgets::choice::RadioGroup` | struct | `src/widgets/choice.rs:118` |
| `widgets::choice::RadioGroup::id` | struct_field | `src/widgets/choice.rs:119` |
| `widgets::choice::RadioGroup::label` | struct_field | `src/widgets/choice.rs:120` |
| `widgets::choice::RadioGroup::options` | struct_field | `src/widgets/choice.rs:121` |
| `widgets::choice::RadioGroup::selected` | struct_field | `src/widgets/choice.rs:122` |
| `widgets::choice::RadioGroup::cursor` | struct_field | `src/widgets/choice.rs:123` |
| `widgets::choice::RadioGroup::disabled` | struct_field | `src/widgets/choice.rs:124` |
| `widgets::choice::RadioGroup::areas` | struct_field | `src/widgets/choice.rs:125` |
| `widgets::choice::RadioGroup::new` | function | `src/widgets/choice.rs:129` |
| `widgets::choice::RadioGroup::height` | function | `src/widgets/choice.rs:141` |
| `widgets::choice::RadioGroup::on_key` | function | `src/widgets/choice.rs:145` |
| `widgets::choice::RadioGroup::on_click` | function | `src/widgets/choice.rs:168` |
| `widgets::choice::RadioGroup::option_id` | function | `src/widgets/choice.rs:178` |
| `widgets::choice::RadioGroup::render` | function | `src/widgets/choice.rs:182` |
| `widgets::choice::Toggle` | struct | `src/widgets/choice.rs:257` |
| `widgets::choice::Toggle::id` | struct_field | `src/widgets/choice.rs:258` |
| `widgets::choice::Toggle::label` | struct_field | `src/widgets/choice.rs:259` |
| `widgets::choice::Toggle::on` | struct_field | `src/widgets/choice.rs:260` |
| `widgets::choice::Toggle::disabled` | struct_field | `src/widgets/choice.rs:261` |
| `widgets::choice::Toggle::area` | struct_field | `src/widgets/choice.rs:262` |
| `widgets::choice::Toggle::new` | function | `src/widgets/choice.rs:266` |
| `widgets::choice::Toggle::disabled` | function | `src/widgets/choice.rs:275` |
| `widgets::choice::Toggle::on_key` | function | `src/widgets/choice.rs:280` |
| `widgets::choice::Toggle::on_click` | function | `src/widgets/choice.rs:292` |
| `widgets::choice::Toggle::render` | function | `src/widgets/choice.rs:300` |

### `src/widgets/code.rs` (62 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::code` | module | `src/widgets/code.rs:1` |
| `widgets::code::Highlighter` | type_alias | `src/widgets/code.rs:26` |
| `widgets::code::Segmenter` | type_alias | `src/widgets/code.rs:27` |
| `widgets::code::Severity` | enum | `src/widgets/code.rs:30` |
| `widgets::code::Severity::Error` | variant | `src/widgets/code.rs:31` |
| `widgets::code::Severity::Warning` | variant | `src/widgets/code.rs:32` |
| `widgets::code::Diagnostic` | struct | `src/widgets/code.rs:36` |
| `widgets::code::Diagnostic::range` | struct_field | `src/widgets/code.rs:37` |
| `widgets::code::Diagnostic::severity` | struct_field | `src/widgets/code.rs:38` |
| `widgets::code::Diagnostic::message` | struct_field | `src/widgets/code.rs:39` |
| `widgets::code::FindState` | struct | `src/widgets/code.rs:43` |
| `widgets::code::FindState::needle` | struct_field | `src/widgets/code.rs:44` |
| `widgets::code::FindState::matches` | struct_field | `src/widgets/code.rs:47` |
| `widgets::code::FindState::current` | struct_field | `src/widgets/code.rs:48` |
| `widgets::code::FindState::editing` | struct_field | `src/widgets/code.rs:49` |
| `widgets::code::EditorEvent` | enum | `src/widgets/code.rs:53` |
| `widgets::code::EditorEvent::Changed` | variant | `src/widgets/code.rs:54` |
| `widgets::code::EditorEvent::CursorMoved` | variant | `src/widgets/code.rs:55` |
| `widgets::code::EditorEvent::Committed` | variant | `src/widgets/code.rs:57` |
| `widgets::code::EditorEvent::Leave` | variant | `src/widgets/code.rs:59` |
| `widgets::code::EditorEvent::Leave::backward` | struct_field | `src/widgets/code.rs:60` |
| `widgets::code::CodeEditor` | struct | `src/widgets/code.rs:65` |
| `widgets::code::CodeEditor::id` | struct_field | `src/widgets/code.rs:66` |
| `widgets::code::CodeEditor::buffer` | struct_field | `src/widgets/code.rs:67` |
| `widgets::code::CodeEditor::editing` | struct_field | `src/widgets/code.rs:68` |
| `widgets::code::CodeEditor::read_only` | struct_field | `src/widgets/code.rs:69` |
| `widgets::code::CodeEditor::scroll` | struct_field | `src/widgets/code.rs:70` |
| `widgets::code::CodeEditor::hscroll` | struct_field | `src/widgets/code.rs:71` |
| `widgets::code::CodeEditor::indent` | struct_field | `src/widgets/code.rs:72` |
| `widgets::code::CodeEditor::highlighter` | struct_field | `src/widgets/code.rs:73` |
| `widgets::code::CodeEditor::segmenter` | struct_field | `src/widgets/code.rs:74` |
| `widgets::code::CodeEditor::diagnostics` | struct_field | `src/widgets/code.rs:75` |
| `widgets::code::CodeEditor::running` | struct_field | `src/widgets/code.rs:77` |
| `widgets::code::CodeEditor::find` | struct_field | `src/widgets/code.rs:78` |
| `widgets::code::CodeEditor::placeholder` | struct_field | `src/widgets/code.rs:79` |
| `widgets::code::CodeEditor::tab_leaves` | struct_field | `src/widgets/code.rs:82` |
| `widgets::code::CodeEditor::area` | struct_field | `src/widgets/code.rs:83` |
| `widgets::code::CodeEditor::new` | function | `src/widgets/code.rs:104` |
| `widgets::code::CodeEditor::highlighter` | function | `src/widgets/code.rs:132` |
| `widgets::code::CodeEditor::segmenter` | function | `src/widgets/code.rs:136` |
| `widgets::code::CodeEditor::read_only` | function | `src/widgets/code.rs:140` |
| `widgets::code::CodeEditor::placeholder` | function | `src/widgets/code.rs:144` |
| `widgets::code::CodeEditor::text` | function | `src/widgets/code.rs:149` |
| `widgets::code::CodeEditor::set_text` | function | `src/widgets/code.rs:153` |
| `widgets::code::CodeEditor::is_empty` | function | `src/widgets/code.rs:162` |
| `widgets::code::CodeEditor::current_block` | function | `src/widgets/code.rs:167` |
| `widgets::code::CodeEditor::selection_or_block` | function | `src/widgets/code.rs:179` |
| `widgets::code::CodeEditor::blocks` | function | `src/widgets/code.rs:188` |
| `widgets::code::CodeEditor::jump_to` | function | `src/widgets/code.rs:194` |
| `widgets::code::CodeEditor::cursor_offset` | function | `src/widgets/code.rs:201` |
| `widgets::code::CodeEditor::cursor_cell` | function | `src/widgets/code.rs:206` |
| `widgets::code::CodeEditor::begin_edit` | function | `src/widgets/code.rs:220` |
| `widgets::code::CodeEditor::set_running` | function | `src/widgets/code.rs:227` |
| `widgets::code::CodeEditor::commit` | function | `src/widgets/code.rs:231` |
| `widgets::code::CodeEditor::open_find` | function | `src/widgets/code.rs:241` |
| `widgets::code::CodeEditor::on_key` | function | `src/widgets/code.rs:286` |
| `widgets::code::CodeEditor::on_click` | function | `src/widgets/code.rs:533` |
| `widgets::code::CodeEditor::on_drag` | function | `src/widgets/code.rs:551` |
| `widgets::code::CodeEditor::on_wheel` | function | `src/widgets/code.rs:560` |
| `widgets::code::CodeEditor::on_scrollbar` | function | `src/widgets/code.rs:569` |
| `widgets::code::CodeEditor::on_paste` | function | `src/widgets/code.rs:584` |
| `widgets::code::CodeEditor::render` | function | `src/widgets/code.rs:617` |

### `src/widgets/completion.rs` (32 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::completion` | module | `src/widgets/completion.rs:1` |
| `widgets::completion::CompletionItem` | struct | `src/widgets/completion.rs:19` |
| `widgets::completion::CompletionItem::label` | struct_field | `src/widgets/completion.rs:20` |
| `widgets::completion::CompletionItem::glyph` | struct_field | `src/widgets/completion.rs:22` |
| `widgets::completion::CompletionItem::detail` | struct_field | `src/widgets/completion.rs:23` |
| `widgets::completion::CompletionItem::insert` | struct_field | `src/widgets/completion.rs:24` |
| `widgets::completion::CompletionItem::matched` | struct_field | `src/widgets/completion.rs:26` |
| `widgets::completion::Completion` | struct | `src/widgets/completion.rs:30` |
| `widgets::completion::Completion::id` | struct_field | `src/widgets/completion.rs:31` |
| `widgets::completion::Completion::items` | struct_field | `src/widgets/completion.rs:32` |
| `widgets::completion::Completion::cursor` | struct_field | `src/widgets/completion.rs:33` |
| `widgets::completion::Completion::scroll` | struct_field | `src/widgets/completion.rs:34` |
| `widgets::completion::Completion::anchor` | struct_field | `src/widgets/completion.rs:35` |
| `widgets::completion::Completion::replace_len` | struct_field | `src/widgets/completion.rs:37` |
| `widgets::completion::Completion::max_rows` | struct_field | `src/widgets/completion.rs:38` |
| `widgets::completion::Completion::area` | struct_field | `src/widgets/completion.rs:39` |
| `widgets::completion::CompletionEvent` | enum | `src/widgets/completion.rs:43` |
| `widgets::completion::CompletionEvent::Accept` | variant | `src/widgets/completion.rs:44` |
| `widgets::completion::CompletionEvent::Accept::0` | struct_field | `src/widgets/completion.rs:44` |
| `widgets::completion::CompletionEvent::Dismiss` | variant | `src/widgets/completion.rs:45` |
| `widgets::completion::Completion::new` | function | `src/widgets/completion.rs:49` |
| `widgets::completion::Completion::open` | function | `src/widgets/completion.rs:62` |
| `widgets::completion::Completion::is_open` | function | `src/widgets/completion.rs:70` |
| `widgets::completion::Completion::close` | function | `src/widgets/completion.rs:74` |
| `widgets::completion::Completion::current` | function | `src/widgets/completion.rs:78` |
| `widgets::completion::Completion::row_id` | function | `src/widgets/completion.rs:82` |
| `widgets::completion::Completion::locate` | function | `src/widgets/completion.rs:86` |
| `widgets::completion::Completion::owns` | function | `src/widgets/completion.rs:90` |
| `widgets::completion::Completion::on_key` | function | `src/widgets/completion.rs:94` |
| `widgets::completion::Completion::on_click` | function | `src/widgets/completion.rs:136` |
| `widgets::completion::Completion::on_wheel` | function | `src/widgets/completion.rs:142` |
| `widgets::completion::Completion::render` | function | `src/widgets/completion.rs:147` |

### `src/widgets/dialog.rs` (40 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::dialog` | module | `src/widgets/dialog.rs:1` |
| `widgets::dialog::DialogBody` | enum | `src/widgets/dialog.rs:18` |
| `widgets::dialog::DialogBody::Text` | variant | `src/widgets/dialog.rs:19` |
| `widgets::dialog::DialogBody::Text::0` | struct_field | `src/widgets/dialog.rs:19` |
| `widgets::dialog::DialogBody::Input` | variant | `src/widgets/dialog.rs:20` |
| `widgets::dialog::DialogBody::Input::0` | struct_field | `src/widgets/dialog.rs:20` |
| `widgets::dialog::DialogBody::Facts` | variant | `src/widgets/dialog.rs:23` |
| `widgets::dialog::DialogBody::Facts::facts` | struct_field | `src/widgets/dialog.rs:24` |
| `widgets::dialog::DialogBody::Facts::code` | struct_field | `src/widgets/dialog.rs:25` |
| `widgets::dialog::DialogBody::Facts::ack` | struct_field | `src/widgets/dialog.rs:26` |
| `widgets::dialog::AckInput` | struct | `src/widgets/dialog.rs:31` |
| `widgets::dialog::AckInput::input` | struct_field | `src/widgets/dialog.rs:32` |
| `widgets::dialog::AckInput::token` | struct_field | `src/widgets/dialog.rs:33` |
| `widgets::dialog::DialogResult` | enum | `src/widgets/dialog.rs:37` |
| `widgets::dialog::DialogResult::Action` | variant | `src/widgets/dialog.rs:39` |
| `widgets::dialog::DialogResult::Action::0` | struct_field | `src/widgets/dialog.rs:39` |
| `widgets::dialog::DialogResult::Cancelled` | variant | `src/widgets/dialog.rs:40` |
| `widgets::dialog::Dialog` | struct | `src/widgets/dialog.rs:44` |
| `widgets::dialog::Dialog::id` | struct_field | `src/widgets/dialog.rs:45` |
| `widgets::dialog::Dialog::title` | struct_field | `src/widgets/dialog.rs:46` |
| `widgets::dialog::Dialog::body` | struct_field | `src/widgets/dialog.rs:47` |
| `widgets::dialog::Dialog::actions` | struct_field | `src/widgets/dialog.rs:48` |
| `widgets::dialog::Dialog::cancel_index` | struct_field | `src/widgets/dialog.rs:50` |
| `widgets::dialog::Dialog::width` | struct_field | `src/widgets/dialog.rs:51` |
| `widgets::dialog::Dialog::area` | struct_field | `src/widgets/dialog.rs:52` |
| `widgets::dialog::Dialog::result` | struct_field | `src/widgets/dialog.rs:53` |
| `widgets::dialog::Dialog::initial_focus` | struct_field | `src/widgets/dialog.rs:54` |
| `widgets::dialog::Dialog::confirm` | function | `src/widgets/dialog.rs:58` |
| `widgets::dialog::Dialog::destructive` | function | `src/widgets/dialog.rs:75` |
| `widgets::dialog::Dialog::prompt` | function | `src/widgets/dialog.rs:92` |
| `widgets::dialog::Dialog::facts` | function | `src/widgets/dialog.rs:110` |
| `widgets::dialog::Dialog::armed` | function | `src/widgets/dialog.rs:139` |
| `widgets::dialog::Dialog::with_actions` | function | `src/widgets/dialog.rs:146` |
| `widgets::dialog::Dialog::is_editing` | function | `src/widgets/dialog.rs:152` |
| `widgets::dialog::Dialog::height` | function | `src/widgets/dialog.rs:168` |
| `widgets::dialog::Dialog::on_key` | function | `src/widgets/dialog.rs:207` |
| `widgets::dialog::Dialog::on_paste` | function | `src/widgets/dialog.rs:316` |
| `widgets::dialog::Dialog::on_click` | function | `src/widgets/dialog.rs:324` |
| `widgets::dialog::Dialog::on_click_outside` | function | `src/widgets/dialog.rs:350` |
| `widgets::dialog::Dialog::render` | function | `src/widgets/dialog.rs:357` |

### `src/widgets/diff.rs` (60 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::diff` | module | `src/widgets/diff.rs:1` |
| `widgets::diff::DiffLineKind` | enum | `src/widgets/diff.rs:20` |
| `widgets::diff::DiffLineKind::Context` | variant | `src/widgets/diff.rs:21` |
| `widgets::diff::DiffLineKind::Add` | variant | `src/widgets/diff.rs:22` |
| `widgets::diff::DiffLineKind::Remove` | variant | `src/widgets/diff.rs:23` |
| `widgets::diff::DiffLine` | struct | `src/widgets/diff.rs:27` |
| `widgets::diff::DiffLine::kind` | struct_field | `src/widgets/diff.rs:28` |
| `widgets::diff::DiffLine::text` | struct_field | `src/widgets/diff.rs:29` |
| `widgets::diff::DiffLine::context` | function | `src/widgets/diff.rs:33` |
| `widgets::diff::DiffLine::add` | function | `src/widgets/diff.rs:39` |
| `widgets::diff::DiffLine::remove` | function | `src/widgets/diff.rs:45` |
| `widgets::diff::DiffHunk` | struct | `src/widgets/diff.rs:54` |
| `widgets::diff::DiffHunk::old_start` | struct_field | `src/widgets/diff.rs:55` |
| `widgets::diff::DiffHunk::new_start` | struct_field | `src/widgets/diff.rs:56` |
| `widgets::diff::DiffHunk::lines` | struct_field | `src/widgets/diff.rs:57` |
| `widgets::diff::DiffHunk::old_len` | function | `src/widgets/diff.rs:62` |
| `widgets::diff::DiffHunk::new_len` | function | `src/widgets/diff.rs:69` |
| `widgets::diff::DiffHunk::header` | function | `src/widgets/diff.rs:75` |
| `widgets::diff::DiffStatus` | enum | `src/widgets/diff.rs:87` |
| `widgets::diff::DiffStatus::Added` | variant | `src/widgets/diff.rs:88` |
| `widgets::diff::DiffStatus::Modified` | variant | `src/widgets/diff.rs:89` |
| `widgets::diff::DiffStatus::Deleted` | variant | `src/widgets/diff.rs:90` |
| `widgets::diff::DiffStatus::Renamed` | variant | `src/widgets/diff.rs:91` |
| `widgets::diff::DiffStatus::Renamed::from` | struct_field | `src/widgets/diff.rs:91` |
| `widgets::diff::DiffStatus::marker` | function | `src/widgets/diff.rs:96` |
| `widgets::diff::DiffStatus::label` | function | `src/widgets/diff.rs:104` |
| `widgets::diff::DiffStatus::tone` | function | `src/widgets/diff.rs:114` |
| `widgets::diff::DiffFile` | struct | `src/widgets/diff.rs:125` |
| `widgets::diff::DiffFile::path` | struct_field | `src/widgets/diff.rs:126` |
| `widgets::diff::DiffFile::status` | struct_field | `src/widgets/diff.rs:127` |
| `widgets::diff::DiffFile::hunks` | struct_field | `src/widgets/diff.rs:128` |
| `widgets::diff::DiffFile::additions` | function | `src/widgets/diff.rs:132` |
| `widgets::diff::DiffFile::deletions` | function | `src/widgets/diff.rs:139` |
| `widgets::diff::DiffFile::summary` | function | `src/widgets/diff.rs:147` |
| `widgets::diff::DiffFile::header` | function | `src/widgets/diff.rs:158` |
| `widgets::diff::DiffMode` | enum | `src/widgets/diff.rs:169` |
| `widgets::diff::DiffMode::Unified` | variant | `src/widgets/diff.rs:172` |
| `widgets::diff::DiffMode::Review` | variant | `src/widgets/diff.rs:174` |
| `widgets::diff::DiffMode::label` | function | `src/widgets/diff.rs:178` |
| `widgets::diff::DiffMode::toggled` | function | `src/widgets/diff.rs:184` |
| `widgets::diff::DiffView` | struct | `src/widgets/diff.rs:193` |
| `widgets::diff::DiffView::term` | struct_field | `src/widgets/diff.rs:194` |
| `widgets::diff::DiffView::mode` | struct_field | `src/widgets/diff.rs:195` |
| `widgets::diff::DiffView::new` | function | `src/widgets/diff.rs:204` |
| `widgets::diff::DiffView::id` | function | `src/widgets/diff.rs:216` |
| `widgets::diff::DiffView::file` | function | `src/widgets/diff.rs:220` |
| `widgets::diff::DiffView::set_file` | function | `src/widgets/diff.rs:224` |
| `widgets::diff::DiffView::set_mode` | function | `src/widgets/diff.rs:233` |
| `widgets::diff::DiffView::toggle_mode` | function | `src/widgets/diff.rs:240` |
| `widgets::diff::DiffView::layout_mode` | function | `src/widgets/diff.rs:250` |
| `widgets::diff::DiffView::layout` | function | `src/widgets/diff.rs:260` |
| `widgets::diff::DiffView::owns` | function | `src/widgets/diff.rs:282` |
| `widgets::diff::DiffView::on_key` | function | `src/widgets/diff.rs:286` |
| `widgets::diff::DiffView::on_wheel` | function | `src/widgets/diff.rs:290` |
| `widgets::diff::DiffView::on_click` | function | `src/widgets/diff.rs:294` |
| `widgets::diff::DiffView::on_drag` | function | `src/widgets/diff.rs:298` |
| `widgets::diff::DiffView::on_scrollbar` | function | `src/widgets/diff.rs:302` |
| `widgets::diff::DiffView::render` | function | `src/widgets/diff.rs:306` |
| `widgets::diff::unified_lines` | function | `src/widgets/diff.rs:346` |
| `widgets::diff::review_lines` | function | `src/widgets/diff.rs:442` |

### `src/widgets/empty.rs` (12 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::empty` | module | `src/widgets/empty.rs:1` |
| `widgets::empty::EmptyKind` | enum | `src/widgets/empty.rs:10` |
| `widgets::empty::EmptyKind::Empty` | variant | `src/widgets/empty.rs:12` |
| `widgets::empty::EmptyKind::Error` | variant | `src/widgets/empty.rs:14` |
| `widgets::empty::EmptyState` | struct | `src/widgets/empty.rs:18` |
| `widgets::empty::EmptyState::title` | struct_field | `src/widgets/empty.rs:19` |
| `widgets::empty::EmptyState::hint` | struct_field | `src/widgets/empty.rs:20` |
| `widgets::empty::EmptyState::kind` | struct_field | `src/widgets/empty.rs:21` |
| `widgets::empty::EmptyState::new` | function | `src/widgets/empty.rs:25` |
| `widgets::empty::EmptyState::error` | function | `src/widgets/empty.rs:32` |
| `widgets::empty::EmptyState::hint` | function | `src/widgets/empty.rs:39` |
| `widgets::empty::render` | function | `src/widgets/empty.rs:46` |

### `src/widgets/field_common.rs` (12 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::field_common` | module | `src/widgets/field_common.rs:1` |
| `widgets::field_common::EditAction` | enum | `src/widgets/field_common.rs:8` |
| `widgets::field_common::EditAction::Commit` | variant | `src/widgets/field_common.rs:9` |
| `widgets::field_common::EditAction::Cancel` | variant | `src/widgets/field_common.rs:10` |
| `widgets::field_common::EditAction::Tab` | variant | `src/widgets/field_common.rs:11` |
| `widgets::field_common::EditAction::Tab::backward` | struct_field | `src/widgets/field_common.rs:11` |
| `widgets::field_common::EditAction::Apply` | variant | `src/widgets/field_common.rs:12` |
| `widgets::field_common::EditAction::Apply::0` | struct_field | `src/widgets/field_common.rs:12` |
| `widgets::field_common::EditAction::Insert` | variant | `src/widgets/field_common.rs:13` |
| `widgets::field_common::EditAction::Insert::0` | struct_field | `src/widgets/field_common.rs:13` |
| `widgets::field_common::EditAction::None` | variant | `src/widgets/field_common.rs:14` |
| `widgets::field_common::edit_key` | function | `src/widgets/field_common.rs:20` |

### `src/widgets/grid.rs` (189 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::grid` | module | `src/widgets/grid.rs:1` |
| `widgets::grid::CellValue` | enum | `src/widgets/grid.rs:32` |
| `widgets::grid::CellValue::Null` | variant | `src/widgets/grid.rs:33` |
| `widgets::grid::CellValue::Default` | variant | `src/widgets/grid.rs:35` |
| `widgets::grid::CellValue::Text` | variant | `src/widgets/grid.rs:36` |
| `widgets::grid::CellValue::Text::0` | struct_field | `src/widgets/grid.rs:36` |
| `widgets::grid::CellValue::Int` | variant | `src/widgets/grid.rs:37` |
| `widgets::grid::CellValue::Int::0` | struct_field | `src/widgets/grid.rs:37` |
| `widgets::grid::CellValue::Num` | variant | `src/widgets/grid.rs:38` |
| `widgets::grid::CellValue::Num::0` | struct_field | `src/widgets/grid.rs:38` |
| `widgets::grid::CellValue::Bool` | variant | `src/widgets/grid.rs:39` |
| `widgets::grid::CellValue::Bool::0` | struct_field | `src/widgets/grid.rs:39` |
| `widgets::grid::CellValue::Json` | variant | `src/widgets/grid.rs:40` |
| `widgets::grid::CellValue::Json::0` | struct_field | `src/widgets/grid.rs:40` |
| `widgets::grid::CellValue::text` | function | `src/widgets/grid.rs:44` |
| `widgets::grid::CellValue::edit_text` | function | `src/widgets/grid.rs:55` |
| `widgets::grid::CellKind` | enum | `src/widgets/grid.rs:65` |
| `widgets::grid::CellKind::Text` | variant | `src/widgets/grid.rs:66` |
| `widgets::grid::CellKind::Id` | variant | `src/widgets/grid.rs:67` |
| `widgets::grid::CellKind::Number` | variant | `src/widgets/grid.rs:68` |
| `widgets::grid::CellKind::Bool` | variant | `src/widgets/grid.rs:69` |
| `widgets::grid::CellKind::Timestamp` | variant | `src/widgets/grid.rs:70` |
| `widgets::grid::CellKind::Json` | variant | `src/widgets/grid.rs:71` |
| `widgets::grid::CellKind::Enum` | variant | `src/widgets/grid.rs:72` |
| `widgets::grid::ColumnSpec` | struct | `src/widgets/grid.rs:93` |
| `widgets::grid::ColumnSpec::name` | struct_field | `src/widgets/grid.rs:94` |
| `widgets::grid::ColumnSpec::kind` | struct_field | `src/widgets/grid.rs:95` |
| `widgets::grid::ColumnSpec::primary` | struct_field | `src/widgets/grid.rs:96` |
| `widgets::grid::ColumnSpec::nullable` | struct_field | `src/widgets/grid.rs:97` |
| `widgets::grid::ColumnSpec::read_only` | struct_field | `src/widgets/grid.rs:98` |
| `widgets::grid::ColumnSpec::references` | struct_field | `src/widgets/grid.rs:99` |
| `widgets::grid::ColumnSpec::enum_values` | struct_field | `src/widgets/grid.rs:100` |
| `widgets::grid::ColumnSpec::sortable` | struct_field | `src/widgets/grid.rs:101` |
| `widgets::grid::ColumnSpec::min_width` | struct_field | `src/widgets/grid.rs:102` |
| `widgets::grid::ColumnSpec::max_width` | struct_field | `src/widgets/grid.rs:103` |
| `widgets::grid::ColumnSpec::type_label` | struct_field | `src/widgets/grid.rs:105` |
| `widgets::grid::ColumnSpec::new` | function | `src/widgets/grid.rs:109` |
| `widgets::grid::ColumnSpec::primary` | function | `src/widgets/grid.rs:125` |
| `widgets::grid::ColumnSpec::nullable` | function | `src/widgets/grid.rs:129` |
| `widgets::grid::ColumnSpec::read_only` | function | `src/widgets/grid.rs:133` |
| `widgets::grid::ColumnSpec::references` | function | `src/widgets/grid.rs:137` |
| `widgets::grid::ColumnSpec::type_label` | function | `src/widgets/grid.rs:141` |
| `widgets::grid::ColumnSpec::enum_values` | function | `src/widgets/grid.rs:145` |
| `widgets::grid::RowTotal` | enum | `src/widgets/grid.rs:152` |
| `widgets::grid::RowTotal::Exact` | variant | `src/widgets/grid.rs:153` |
| `widgets::grid::RowTotal::Exact::0` | struct_field | `src/widgets/grid.rs:153` |
| `widgets::grid::RowTotal::Estimated` | variant | `src/widgets/grid.rs:154` |
| `widgets::grid::RowTotal::Estimated::0` | struct_field | `src/widgets/grid.rs:154` |
| `widgets::grid::RowTotal::Unknown` | variant | `src/widgets/grid.rs:155` |
| `widgets::grid::GridRows` | struct | `src/widgets/grid.rs:159` |
| `widgets::grid::GridRows::rows` | struct_field | `src/widgets/grid.rs:160` |
| `widgets::grid::GridRows::total` | struct_field | `src/widgets/grid.rs:161` |
| `widgets::grid::GridRows::more` | struct_field | `src/widgets/grid.rs:163` |
| `widgets::grid::UndoAction` | enum | `src/widgets/grid.rs:169` |
| `widgets::grid::UndoAction::Cell` | variant | `src/widgets/grid.rs:170` |
| `widgets::grid::UndoAction::Cell::row` | struct_field | `src/widgets/grid.rs:171` |
| `widgets::grid::UndoAction::Cell::col` | struct_field | `src/widgets/grid.rs:172` |
| `widgets::grid::UndoAction::Cell::before` | struct_field | `src/widgets/grid.rs:173` |
| `widgets::grid::UndoAction::Delete` | variant | `src/widgets/grid.rs:175` |
| `widgets::grid::UndoAction::Delete::row` | struct_field | `src/widgets/grid.rs:176` |
| `widgets::grid::UndoAction::Delete::was_deleted` | struct_field | `src/widgets/grid.rs:177` |
| `widgets::grid::UndoAction::Insert` | variant | `src/widgets/grid.rs:179` |
| `widgets::grid::UndoAction::Insert::row` | struct_field | `src/widgets/grid.rs:180` |
| `widgets::grid::PendingChanges` | struct | `src/widgets/grid.rs:187` |
| `widgets::grid::PendingChanges::cells` | struct_field | `src/widgets/grid.rs:188` |
| `widgets::grid::PendingChanges::inserted` | struct_field | `src/widgets/grid.rs:189` |
| `widgets::grid::PendingChanges::deleted` | struct_field | `src/widgets/grid.rs:190` |
| `widgets::grid::PendingChanges::is_empty` | function | `src/widgets/grid.rs:194` |
| `widgets::grid::PendingChanges::dirty_rows` | function | `src/widgets/grid.rs:197` |
| `widgets::grid::PendingChanges::counts` | function | `src/widgets/grid.rs:205` |
| `widgets::grid::PendingChanges::total` | function | `src/widgets/grid.rs:212` |
| `widgets::grid::PendingChanges::is_dirty` | function | `src/widgets/grid.rs:216` |
| `widgets::grid::PendingChanges::value` | function | `src/widgets/grid.rs:219` |
| `widgets::grid::RowState` | enum | `src/widgets/grid.rs:225` |
| `widgets::grid::RowState::Clean` | variant | `src/widgets/grid.rs:226` |
| `widgets::grid::RowState::Modified` | variant | `src/widgets/grid.rs:227` |
| `widgets::grid::RowState::Inserted` | variant | `src/widgets/grid.rs:228` |
| `widgets::grid::RowState::Deleted` | variant | `src/widgets/grid.rs:229` |
| `widgets::grid::RowState::Error` | variant | `src/widgets/grid.rs:230` |
| `widgets::grid::EditState` | struct | `src/widgets/grid.rs:234` |
| `widgets::grid::EditState::row` | struct_field | `src/widgets/grid.rs:235` |
| `widgets::grid::EditState::col` | struct_field | `src/widgets/grid.rs:236` |
| `widgets::grid::EditState::buffer` | struct_field | `src/widgets/grid.rs:237` |
| `widgets::grid::EditState::error` | struct_field | `src/widgets/grid.rs:238` |
| `widgets::grid::GridEvent` | enum | `src/widgets/grid.rs:242` |
| `widgets::grid::GridEvent::CellChanged` | variant | `src/widgets/grid.rs:243` |
| `widgets::grid::GridEvent::CellChanged::col` | struct_field | `src/widgets/grid.rs:243` |
| `widgets::grid::GridEvent::CellChanged::row` | struct_field | `src/widgets/grid.rs:243` |
| `widgets::grid::GridEvent::RowInserted` | variant | `src/widgets/grid.rs:244` |
| `widgets::grid::GridEvent::RowInserted::0` | struct_field | `src/widgets/grid.rs:244` |
| `widgets::grid::GridEvent::RowDeleted` | variant | `src/widgets/grid.rs:245` |
| `widgets::grid::GridEvent::RowDeleted::0` | struct_field | `src/widgets/grid.rs:245` |
| `widgets::grid::GridEvent::SortRequested` | variant | `src/widgets/grid.rs:246` |
| `widgets::grid::GridEvent::SortRequested::0` | struct_field | `src/widgets/grid.rs:246` |
| `widgets::grid::GridEvent::FetchMore` | variant | `src/widgets/grid.rs:247` |
| `widgets::grid::GridEvent::Refresh` | variant | `src/widgets/grid.rs:248` |
| `widgets::grid::GridEvent::CommitRequested` | variant | `src/widgets/grid.rs:249` |
| `widgets::grid::GridEvent::DiscardRequested` | variant | `src/widgets/grid.rs:250` |
| `widgets::grid::GridEvent::PreviewSql` | variant | `src/widgets/grid.rs:251` |
| `widgets::grid::GridEvent::Copy` | variant | `src/widgets/grid.rs:252` |
| `widgets::grid::GridEvent::Copy::0` | struct_field | `src/widgets/grid.rs:252` |
| `widgets::grid::GridEvent::FollowReference` | variant | `src/widgets/grid.rs:253` |
| `widgets::grid::GridEvent::FollowReference::col` | struct_field | `src/widgets/grid.rs:253` |
| `widgets::grid::GridEvent::FollowReference::row` | struct_field | `src/widgets/grid.rs:253` |
| `widgets::grid::GridEvent::OpenViewer` | variant | `src/widgets/grid.rs:254` |
| `widgets::grid::GridEvent::OpenViewer::col` | struct_field | `src/widgets/grid.rs:254` |
| `widgets::grid::GridEvent::OpenViewer::row` | struct_field | `src/widgets/grid.rs:254` |
| `widgets::grid::GridEvent::FilterOnCell` | variant | `src/widgets/grid.rs:255` |
| `widgets::grid::GridEvent::FilterOnCell::col` | struct_field | `src/widgets/grid.rs:255` |
| `widgets::grid::GridEvent::FilterOnCell::value` | struct_field | `src/widgets/grid.rs:255` |
| `widgets::grid::GridEvent::OpenFilters` | variant | `src/widgets/grid.rs:256` |
| `widgets::grid::GridEvent::ClearFilters` | variant | `src/widgets/grid.rs:257` |
| `widgets::grid::GridEvent::Activated` | variant | `src/widgets/grid.rs:258` |
| `widgets::grid::GridEvent::Activated::0` | struct_field | `src/widgets/grid.rs:258` |
| `widgets::grid::GridEvent::LeaveForward` | variant | `src/widgets/grid.rs:259` |
| `widgets::grid::GridEvent::LeaveBackward` | variant | `src/widgets/grid.rs:260` |
| `widgets::grid::Validator` | type_alias | `src/widgets/grid.rs:265` |
| `widgets::grid::default_validator` | function | `src/widgets/grid.rs:267` |
| `widgets::grid::DataGrid` | struct | `src/widgets/grid.rs:327` |
| `widgets::grid::DataGrid::id` | struct_field | `src/widgets/grid.rs:328` |
| `widgets::grid::DataGrid::columns` | struct_field | `src/widgets/grid.rs:329` |
| `widgets::grid::DataGrid::total` | struct_field | `src/widgets/grid.rs:332` |
| `widgets::grid::DataGrid::more` | struct_field | `src/widgets/grid.rs:333` |
| `widgets::grid::DataGrid::sort` | struct_field | `src/widgets/grid.rs:334` |
| `widgets::grid::DataGrid::local_sort` | struct_field | `src/widgets/grid.rs:336` |
| `widgets::grid::DataGrid::filtered_cols` | struct_field | `src/widgets/grid.rs:337` |
| `widgets::grid::DataGrid::cursor` | struct_field | `src/widgets/grid.rs:338` |
| `widgets::grid::DataGrid::selected_rows` | struct_field | `src/widgets/grid.rs:341` |
| `widgets::grid::DataGrid::pending` | struct_field | `src/widgets/grid.rs:342` |
| `widgets::grid::DataGrid::scroll` | struct_field | `src/widgets/grid.rs:344` |
| `widgets::grid::DataGrid::hscroll` | struct_field | `src/widgets/grid.rs:345` |
| `widgets::grid::DataGrid::edit` | struct_field | `src/widgets/grid.rs:346` |
| `widgets::grid::DataGrid::editable` | struct_field | `src/widgets/grid.rs:347` |
| `widgets::grid::DataGrid::read_only_reason` | struct_field | `src/widgets/grid.rs:348` |
| `widgets::grid::DataGrid::loading` | struct_field | `src/widgets/grid.rs:349` |
| `widgets::grid::DataGrid::cell_errors` | struct_field | `src/widgets/grid.rs:350` |
| `widgets::grid::DataGrid::row_errors` | struct_field | `src/widgets/grid.rs:351` |
| `widgets::grid::DataGrid::empty` | struct_field | `src/widgets/grid.rs:352` |
| `widgets::grid::DataGrid::validator` | struct_field | `src/widgets/grid.rs:353` |
| `widgets::grid::DataGrid::row_numbers` | struct_field | `src/widgets/grid.rs:354` |
| `widgets::grid::DataGrid::area` | struct_field | `src/widgets/grid.rs:355` |
| `widgets::grid::DataGrid::new` | function | `src/widgets/grid.rs:367` |
| `widgets::grid::DataGrid::editable` | function | `src/widgets/grid.rs:408` |
| `widgets::grid::DataGrid::len` | function | `src/widgets/grid.rs:415` |
| `widgets::grid::DataGrid::is_empty` | function | `src/widgets/grid.rs:418` |
| `widgets::grid::DataGrid::rows` | function | `src/widgets/grid.rs:421` |
| `widgets::grid::DataGrid::source_row` | function | `src/widgets/grid.rs:424` |
| `widgets::grid::DataGrid::value` | function | `src/widgets/grid.rs:428` |
| `widgets::grid::DataGrid::set_rows` | function | `src/widgets/grid.rs:438` |
| `widgets::grid::DataGrid::append_rows` | function | `src/widgets/grid.rs:462` |
| `widgets::grid::DataGrid::set_loading` | function | `src/widgets/grid.rs:472` |
| `widgets::grid::DataGrid::row_state` | function | `src/widgets/grid.rs:512` |
| `widgets::grid::DataGrid::is_editing` | function | `src/widgets/grid.rs:528` |
| `widgets::grid::DataGrid::edit_error` | function | `src/widgets/grid.rs:531` |
| `widgets::grid::DataGrid::begin_edit` | function | `src/widgets/grid.rs:539` |
| `widgets::grid::DataGrid::commit_edit` | function | `src/widgets/grid.rs:590` |
| `widgets::grid::DataGrid::cancel_edit` | function | `src/widgets/grid.rs:623` |
| `widgets::grid::DataGrid::record_cell` | function | `src/widgets/grid.rs:628` |
| `widgets::grid::DataGrid::toggle_delete` | function | `src/widgets/grid.rs:649` |
| `widgets::grid::DataGrid::insert_row` | function | `src/widgets/grid.rs:692` |
| `widgets::grid::DataGrid::duplicate_row` | function | `src/widgets/grid.rs:726` |
| `widgets::grid::DataGrid::undo` | function | `src/widgets/grid.rs:750` |
| `widgets::grid::DataGrid::apply_commit_result` | function | `src/widgets/grid.rs:780` |
| `widgets::grid::DataGrid::discard` | function | `src/widgets/grid.rs:813` |
| `widgets::grid::DataGrid::on_key` | function | `src/widgets/grid.rs:924` |
| `widgets::grid::DataGrid::header_id` | function | `src/widgets/grid.rs:1191` |
| `widgets::grid::DataGrid::cell_id` | function | `src/widgets/grid.rs:1194` |
| `widgets::grid::DataGrid::rownum_id` | function | `src/widgets/grid.rs:1197` |
| `widgets::grid::DataGrid::more_id` | function | `src/widgets/grid.rs:1200` |
| `widgets::grid::DataGrid::left_id` | function | `src/widgets/grid.rs:1203` |
| `widgets::grid::DataGrid::right_id` | function | `src/widgets/grid.rs:1206` |
| `widgets::grid::DataGrid::owns` | function | `src/widgets/grid.rs:1210` |
| `widgets::grid::DataGrid::locate` | function | `src/widgets/grid.rs:1222` |
| `widgets::grid::DataGrid::locate_header` | function | `src/widgets/grid.rs:1234` |
| `widgets::grid::DataGrid::locate_rownum` | function | `src/widgets/grid.rs:1237` |
| `widgets::grid::DataGrid::on_click` | function | `src/widgets/grid.rs:1243` |
| `widgets::grid::DataGrid::on_drag` | function | `src/widgets/grid.rs:1346` |
| `widgets::grid::DataGrid::on_wheel` | function | `src/widgets/grid.rs:1369` |
| `widgets::grid::DataGrid::on_scrollbar` | function | `src/widgets/grid.rs:1378` |
| `widgets::grid::DataGrid::on_paste` | function | `src/widgets/grid.rs:1390` |
| `widgets::grid::DataGrid::position_label` | function | `src/widgets/grid.rs:1403` |
| `widgets::grid::DataGrid::rows_label` | function | `src/widgets/grid.rs:1411` |
| `widgets::grid::DataGrid::cols_label` | function | `src/widgets/grid.rs:1437` |
| `widgets::grid::DataGrid::pending_label` | function | `src/widgets/grid.rs:1446` |
| `widgets::grid::DataGrid::render` | function | `src/widgets/grid.rs:1523` |
| `widgets::grid::DataGrid::bar_ids` | function | `src/widgets/grid.rs:1944` |
| `widgets::grid::DataGrid::on_bar_key` | function | `src/widgets/grid.rs:1949` |
| `widgets::grid::cell_text` | function | `src/widgets/grid.rs:1987` |
| `theme::Theme::change_glyph` | function | `src/widgets/grid.rs:2027` |

### `src/widgets/hintbar.rs` (13 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::hintbar` | module | `src/widgets/hintbar.rs:1` |
| `widgets::hintbar::HintLayer` | struct | `src/widgets/hintbar.rs:14` |
| `widgets::hintbar::HintLayer::hints` | struct_field | `src/widgets/hintbar.rs:15` |
| `widgets::hintbar::HintLayer::badge` | struct_field | `src/widgets/hintbar.rs:16` |
| `widgets::hintbar::HintLayer::status` | struct_field | `src/widgets/hintbar.rs:17` |
| `widgets::hintbar::HintLayer::centered` | struct_field | `src/widgets/hintbar.rs:19` |
| `widgets::hintbar::HintLayer::new` | function | `src/widgets/hintbar.rs:23` |
| `widgets::hintbar::HintLayer::centered` | function | `src/widgets/hintbar.rs:31` |
| `widgets::hintbar::HintLayer::badge` | function | `src/widgets/hintbar.rs:35` |
| `widgets::hintbar::HintLayer::status` | function | `src/widgets/hintbar.rs:39` |
| `widgets::hintbar::HintBar` | struct | `src/widgets/hintbar.rs:45` |
| `widgets::hintbar::HintBar::resolve` | function | `src/widgets/hintbar.rs:50` |
| `widgets::hintbar::HintBar::render` | function | `src/widgets/hintbar.rs:56` |

### `src/widgets/input.rs` (43 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::input` | module | `src/widgets/input.rs:1` |
| `widgets::input::TextInput` | struct | `src/widgets/input.rs:19` |
| `widgets::input::TextInput::id` | struct_field | `src/widgets/input.rs:20` |
| `widgets::input::TextInput::label` | struct_field | `src/widgets/input.rs:21` |
| `widgets::input::TextInput::placeholder` | struct_field | `src/widgets/input.rs:22` |
| `widgets::input::TextInput::buffer` | struct_field | `src/widgets/input.rs:23` |
| `widgets::input::TextInput::disabled` | struct_field | `src/widgets/input.rs:24` |
| `widgets::input::TextInput::required` | struct_field | `src/widgets/input.rs:25` |
| `widgets::input::TextInput::help` | struct_field | `src/widgets/input.rs:26` |
| `widgets::input::TextInput::error` | struct_field | `src/widgets/input.rs:27` |
| `widgets::input::TextInput::editing` | struct_field | `src/widgets/input.rs:28` |
| `widgets::input::TextInput::area` | struct_field | `src/widgets/input.rs:33` |
| `widgets::input::TextInput::validator` | struct_field | `src/widgets/input.rs:36` |
| `widgets::input::TextInput::plain_label` | struct_field | `src/widgets/input.rs:38` |
| `widgets::input::TextInput::masked` | struct_field | `src/widgets/input.rs:41` |
| `widgets::input::TextInput::reveal_tail` | struct_field | `src/widgets/input.rs:44` |
| `widgets::input::InputEvent` | enum | `src/widgets/input.rs:48` |
| `widgets::input::InputEvent::Committed` | variant | `src/widgets/input.rs:49` |
| `widgets::input::InputEvent::Cancelled` | variant | `src/widgets/input.rs:50` |
| `widgets::input::InputEvent::CommittedTab` | variant | `src/widgets/input.rs:52` |
| `widgets::input::InputEvent::CommittedTab::backward` | struct_field | `src/widgets/input.rs:53` |
| `widgets::input::InputEvent::Changed` | variant | `src/widgets/input.rs:55` |
| `widgets::input::TextInput::new` | function | `src/widgets/input.rs:59` |
| `widgets::input::TextInput::placeholder` | function | `src/widgets/input.rs:81` |
| `widgets::input::TextInput::value` | function | `src/widgets/input.rs:85` |
| `widgets::input::TextInput::disabled` | function | `src/widgets/input.rs:89` |
| `widgets::input::TextInput::required` | function | `src/widgets/input.rs:93` |
| `widgets::input::TextInput::help` | function | `src/widgets/input.rs:97` |
| `widgets::input::TextInput::plain_label` | function | `src/widgets/input.rs:101` |
| `widgets::input::TextInput::validator` | function | `src/widgets/input.rs:105` |
| `widgets::input::TextInput::masked` | function | `src/widgets/input.rs:109` |
| `widgets::input::TextInput::reveal_tail` | function | `src/widgets/input.rs:113` |
| `widgets::input::TextInput::clear` | function | `src/widgets/input.rs:119` |
| `widgets::input::TextInput::text` | function | `src/widgets/input.rs:148` |
| `widgets::input::TextInput::begin_edit` | function | `src/widgets/input.rs:152` |
| `widgets::input::TextInput::commit` | function | `src/widgets/input.rs:161` |
| `widgets::input::TextInput::cancel` | function | `src/widgets/input.rs:167` |
| `widgets::input::TextInput::validate` | function | `src/widgets/input.rs:176` |
| `widgets::input::TextInput::HEIGHT` | assoc_const | `src/widgets/input.rs:188` |
| `widgets::input::TextInput::on_key` | function | `src/widgets/input.rs:192` |
| `widgets::input::TextInput::on_paste` | function | `src/widgets/input.rs:234` |
| `widgets::input::TextInput::on_click` | function | `src/widgets/input.rs:251` |
| `widgets::input::TextInput::render` | function | `src/widgets/input.rs:279` |

### `src/widgets/keyhint.rs` (8 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::keyhint` | module | `src/widgets/keyhint.rs:1` |
| `widgets::keyhint::Hint` | struct | `src/widgets/keyhint.rs:10` |
| `widgets::keyhint::Hint::key` | struct_field | `src/widgets/keyhint.rs:11` |
| `widgets::keyhint::Hint::action` | struct_field | `src/widgets/keyhint.rs:12` |
| `widgets::keyhint::hint` | function | `src/widgets/keyhint.rs:15` |
| `widgets::keyhint::render` | function | `src/widgets/keyhint.rs:22` |
| `widgets::keyhint::render_toned` | function | `src/widgets/keyhint.rs:44` |
| `widgets::keyhint::render_aligned` | function | `src/widgets/keyhint.rs:57` |

### `src/widgets/list.rs` (33 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::list` | module | `src/widgets/list.rs:1` |
| `widgets::list::ListItem` | struct | `src/widgets/list.rs:13` |
| `widgets::list::ListItem::label` | struct_field | `src/widgets/list.rs:14` |
| `widgets::list::ListItem::meta` | struct_field | `src/widgets/list.rs:15` |
| `widgets::list::ListItem::disabled` | struct_field | `src/widgets/list.rs:16` |
| `widgets::list::ListItem::new` | function | `src/widgets/list.rs:20` |
| `widgets::list::ListItem::meta` | function | `src/widgets/list.rs:27` |
| `widgets::list::ListItem::disabled` | function | `src/widgets/list.rs:31` |
| `widgets::list::SelectMode` | enum | `src/widgets/list.rs:38` |
| `widgets::list::SelectMode::Single` | variant | `src/widgets/list.rs:39` |
| `widgets::list::SelectMode::Multi` | variant | `src/widgets/list.rs:40` |
| `widgets::list::ListBox` | struct | `src/widgets/list.rs:46` |
| `widgets::list::ListBox::id` | struct_field | `src/widgets/list.rs:47` |
| `widgets::list::ListBox::items` | struct_field | `src/widgets/list.rs:48` |
| `widgets::list::ListBox::cursor` | struct_field | `src/widgets/list.rs:49` |
| `widgets::list::ListBox::mode` | struct_field | `src/widgets/list.rs:50` |
| `widgets::list::ListBox::chosen` | struct_field | `src/widgets/list.rs:51` |
| `widgets::list::ListBox::checked` | struct_field | `src/widgets/list.rs:52` |
| `widgets::list::ListBox::scroll` | struct_field | `src/widgets/list.rs:53` |
| `widgets::list::ListBox::area` | struct_field | `src/widgets/list.rs:54` |
| `widgets::list::ListBox::empty_text` | struct_field | `src/widgets/list.rs:55` |
| `widgets::list::ListBox::new` | function | `src/widgets/list.rs:61` |
| `widgets::list::ListBox::empty_text` | function | `src/widgets/list.rs:77` |
| `widgets::list::ListBox::row_id` | function | `src/widgets/list.rs:82` |
| `widgets::list::ListBox::checked_count` | function | `src/widgets/list.rs:86` |
| `widgets::list::ListBox::activate` | function | `src/widgets/list.rs:107` |
| `widgets::list::ListBox::on_key` | function | `src/widgets/list.rs:118` |
| `widgets::list::ListBox::on_click` | function | `src/widgets/list.rs:170` |
| `widgets::list::ListBox::on_wheel` | function | `src/widgets/list.rs:180` |
| `widgets::list::ListBox::locate` | function | `src/widgets/list.rs:186` |
| `widgets::list::ListBox::owns` | function | `src/widgets/list.rs:191` |
| `widgets::list::ListBox::on_scrollbar` | function | `src/widgets/list.rs:195` |
| `widgets::list::ListBox::render` | function | `src/widgets/list.rs:207` |

### `src/widgets/menu.rs` (71 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::menu` | module | `src/widgets/menu.rs:1` |
| `widgets::menu::MenuItem` | struct | `src/widgets/menu.rs:19` |
| `widgets::menu::MenuItem::label` | struct_field | `src/widgets/menu.rs:20` |
| `widgets::menu::MenuItem::shortcut` | struct_field | `src/widgets/menu.rs:21` |
| `widgets::menu::MenuItem::disabled` | struct_field | `src/widgets/menu.rs:22` |
| `widgets::menu::MenuItem::danger` | struct_field | `src/widgets/menu.rs:23` |
| `widgets::menu::MenuItem::separator_after` | struct_field | `src/widgets/menu.rs:24` |
| `widgets::menu::MenuItem::new` | function | `src/widgets/menu.rs:28` |
| `widgets::menu::MenuItem::shortcut` | function | `src/widgets/menu.rs:37` |
| `widgets::menu::MenuItem::disabled` | function | `src/widgets/menu.rs:41` |
| `widgets::menu::MenuItem::danger` | function | `src/widgets/menu.rs:45` |
| `widgets::menu::MenuItem::separator` | function | `src/widgets/menu.rs:49` |
| `widgets::menu::Placement` | enum | `src/widgets/menu.rs:56` |
| `widgets::menu::Placement::Below` | variant | `src/widgets/menu.rs:58` |
| `widgets::menu::Placement::Above` | variant | `src/widgets/menu.rs:60` |
| `widgets::menu::Placement::Right` | variant | `src/widgets/menu.rs:62` |
| `widgets::menu::MenuEvent` | enum | `src/widgets/menu.rs:66` |
| `widgets::menu::MenuEvent::Chosen` | variant | `src/widgets/menu.rs:67` |
| `widgets::menu::MenuEvent::Chosen::0` | struct_field | `src/widgets/menu.rs:67` |
| `widgets::menu::MenuEvent::Dismissed` | variant | `src/widgets/menu.rs:68` |
| `widgets::menu::ContextMenu` | struct | `src/widgets/menu.rs:72` |
| `widgets::menu::ContextMenu::id` | struct_field | `src/widgets/menu.rs:73` |
| `widgets::menu::ContextMenu::items` | struct_field | `src/widgets/menu.rs:74` |
| `widgets::menu::ContextMenu::cursor` | struct_field | `src/widgets/menu.rs:75` |
| `widgets::menu::ContextMenu::anchor` | struct_field | `src/widgets/menu.rs:76` |
| `widgets::menu::ContextMenu::placement` | struct_field | `src/widgets/menu.rs:77` |
| `widgets::menu::ContextMenu::area` | struct_field | `src/widgets/menu.rs:79` |
| `widgets::menu::ContextMenu::title` | struct_field | `src/widgets/menu.rs:81` |
| `widgets::menu::ContextMenu::new` | function | `src/widgets/menu.rs:85` |
| `widgets::menu::ContextMenu::anchor` | function | `src/widgets/menu.rs:98` |
| `widgets::menu::ContextMenu::at` | function | `src/widgets/menu.rs:105` |
| `widgets::menu::ContextMenu::title` | function | `src/widgets/menu.rs:109` |
| `widgets::menu::ContextMenu::row_id` | function | `src/widgets/menu.rs:114` |
| `widgets::menu::ContextMenu::locate` | function | `src/widgets/menu.rs:118` |
| `widgets::menu::ContextMenu::owns` | function | `src/widgets/menu.rs:122` |
| `widgets::menu::ContextMenu::size` | function | `src/widgets/menu.rs:127` |
| `widgets::menu::ContextMenu::on_key` | function | `src/widgets/menu.rs:188` |
| `widgets::menu::ContextMenu::on_click` | function | `src/widgets/menu.rs:219` |
| `widgets::menu::ContextMenu::on_click_outside` | function | `src/widgets/menu.rs:231` |
| `widgets::menu::ContextMenu::render` | function | `src/widgets/menu.rs:235` |
| `widgets::menu::MenuBarEvent` | enum | `src/widgets/menu.rs:354` |
| `widgets::menu::MenuBarEvent::Opened` | variant | `src/widgets/menu.rs:356` |
| `widgets::menu::MenuBarEvent::Opened::0` | struct_field | `src/widgets/menu.rs:356` |
| `widgets::menu::MenuBarEvent::Chosen` | variant | `src/widgets/menu.rs:358` |
| `widgets::menu::MenuBarEvent::Chosen::0` | struct_field | `src/widgets/menu.rs:358` |
| `widgets::menu::MenuBarEvent::Chosen::1` | struct_field | `src/widgets/menu.rs:358` |
| `widgets::menu::MenuBarEvent::Closed` | variant | `src/widgets/menu.rs:359` |
| `widgets::menu::MenuBarEvent::Brand` | variant | `src/widgets/menu.rs:361` |
| `widgets::menu::MenuBar` | struct | `src/widgets/menu.rs:365` |
| `widgets::menu::MenuBar::id` | struct_field | `src/widgets/menu.rs:366` |
| `widgets::menu::MenuBar::labels` | struct_field | `src/widgets/menu.rs:367` |
| `widgets::menu::MenuBar::menus` | struct_field | `src/widgets/menu.rs:368` |
| `widgets::menu::MenuBar::brand` | struct_field | `src/widgets/menu.rs:369` |
| `widgets::menu::MenuBar::cursor` | struct_field | `src/widgets/menu.rs:371` |
| `widgets::menu::MenuBar::open` | struct_field | `src/widgets/menu.rs:372` |
| `widgets::menu::MenuBar::areas` | struct_field | `src/widgets/menu.rs:373` |
| `widgets::menu::MenuBar::brand_area` | struct_field | `src/widgets/menu.rs:374` |
| `widgets::menu::MenuBar::new` | function | `src/widgets/menu.rs:378` |
| `widgets::menu::MenuBar::brand` | function | `src/widgets/menu.rs:391` |
| `widgets::menu::MenuBar::label_id` | function | `src/widgets/menu.rs:396` |
| `widgets::menu::MenuBar::brand_id` | function | `src/widgets/menu.rs:400` |
| `widgets::menu::MenuBar::is_open` | function | `src/widgets/menu.rs:404` |
| `widgets::menu::MenuBar::open_index` | function | `src/widgets/menu.rs:408` |
| `widgets::menu::MenuBar::owns` | function | `src/widgets/menu.rs:414` |
| `widgets::menu::MenuBar::open_menu` | function | `src/widgets/menu.rs:421` |
| `widgets::menu::MenuBar::close` | function | `src/widgets/menu.rs:432` |
| `widgets::menu::MenuBar::on_key` | function | `src/widgets/menu.rs:436` |
| `widgets::menu::MenuBar::on_click` | function | `src/widgets/menu.rs:486` |
| `widgets::menu::MenuBar::on_hover` | function | `src/widgets/menu.rs:517` |
| `widgets::menu::MenuBar::render` | function | `src/widgets/menu.rs:533` |
| `widgets::menu::MenuBar::render_open` | function | `src/widgets/menu.rs:598` |

### `src/widgets/mod.rs` (1 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets` | module | `src/widgets/mod.rs:1` |

### `src/widgets/panel.rs` (31 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::panel` | module | `src/widgets/panel.rs:1` |
| `widgets::panel::PanelKind` | enum | `src/widgets/panel.rs:23` |
| `widgets::panel::PanelKind::Card` | variant | `src/widgets/panel.rs:24` |
| `widgets::panel::PanelKind::Framed` | variant | `src/widgets/panel.rs:25` |
| `widgets::panel::Panel` | struct | `src/widgets/panel.rs:28` |
| `widgets::panel::Panel::title` | struct_field | `src/widgets/panel.rs:29` |
| `widgets::panel::Panel::kind` | struct_field | `src/widgets/panel.rs:30` |
| `widgets::panel::Panel::focused` | struct_field | `src/widgets/panel.rs:31` |
| `widgets::panel::Panel::meta` | struct_field | `src/widgets/panel.rs:33` |
| `widgets::panel::Panel::badge` | struct_field | `src/widgets/panel.rs:34` |
| `widgets::panel::Panel::bg_override` | struct_field | `src/widgets/panel.rs:35` |
| `widgets::panel::Panel::card` | function | `src/widgets/panel.rs:39` |
| `widgets::panel::Panel::framed` | function | `src/widgets/panel.rs:49` |
| `widgets::panel::Panel::focused` | function | `src/widgets/panel.rs:59` |
| `widgets::panel::Panel::meta` | function | `src/widgets/panel.rs:63` |
| `widgets::panel::Panel::bg` | function | `src/widgets/panel.rs:69` |
| `widgets::panel::Panel::render` | function | `src/widgets/panel.rs:80` |
| `widgets::panel::ScrollPanel` | struct | `src/widgets/panel.rs:177` |
| `widgets::panel::ScrollPanel::id` | struct_field | `src/widgets/panel.rs:178` |
| `widgets::panel::ScrollPanel::lines` | struct_field | `src/widgets/panel.rs:179` |
| `widgets::panel::ScrollPanel::scroll` | struct_field | `src/widgets/panel.rs:180` |
| `widgets::panel::ScrollPanel::follow` | struct_field | `src/widgets/panel.rs:181` |
| `widgets::panel::ScrollPanel::wrap` | struct_field | `src/widgets/panel.rs:182` |
| `widgets::panel::ScrollPanel::area` | struct_field | `src/widgets/panel.rs:183` |
| `widgets::panel::ScrollPanel::new` | function | `src/widgets/panel.rs:188` |
| `widgets::panel::ScrollPanel::wrap` | function | `src/widgets/panel.rs:199` |
| `widgets::panel::ScrollPanel::push` | function | `src/widgets/panel.rs:204` |
| `widgets::panel::ScrollPanel::on_key` | function | `src/widgets/panel.rs:209` |
| `widgets::panel::ScrollPanel::on_wheel` | function | `src/widgets/panel.rs:237` |
| `widgets::panel::ScrollPanel::on_scrollbar` | function | `src/widgets/panel.rs:243` |
| `widgets::panel::ScrollPanel::render` | function | `src/widgets/panel.rs:257` |

### `src/widgets/picker.rs` (52 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::picker` | module | `src/widgets/picker.rs:1` |
| `widgets::picker::PickerItem` | struct | `src/widgets/picker.rs:19` |
| `widgets::picker::PickerItem::label` | struct_field | `src/widgets/picker.rs:20` |
| `widgets::picker::PickerItem::detail` | struct_field | `src/widgets/picker.rs:21` |
| `widgets::picker::PickerItem::glyph` | struct_field | `src/widgets/picker.rs:22` |
| `widgets::picker::PickerItem::group` | struct_field | `src/widgets/picker.rs:23` |
| `widgets::picker::PickerItem::tag` | struct_field | `src/widgets/picker.rs:25` |
| `widgets::picker::PickerItem::matched` | struct_field | `src/widgets/picker.rs:26` |
| `widgets::picker::PickerItem::disabled` | struct_field | `src/widgets/picker.rs:27` |
| `widgets::picker::PickerStatus` | enum | `src/widgets/picker.rs:31` |
| `widgets::picker::PickerStatus::Ready` | variant | `src/widgets/picker.rs:33` |
| `widgets::picker::PickerStatus::Loading` | variant | `src/widgets/picker.rs:35` |
| `widgets::picker::PickerStatus::Loading::0` | struct_field | `src/widgets/picker.rs:35` |
| `widgets::picker::PickerStatus::Error` | variant | `src/widgets/picker.rs:37` |
| `widgets::picker::PickerStatus::Error::message` | struct_field | `src/widgets/picker.rs:38` |
| `widgets::picker::PickerStatus::Error::detail` | struct_field | `src/widgets/picker.rs:39` |
| `widgets::picker::Picker` | struct | `src/widgets/picker.rs:44` |
| `widgets::picker::Picker::id` | struct_field | `src/widgets/picker.rs:45` |
| `widgets::picker::Picker::status` | struct_field | `src/widgets/picker.rs:46` |
| `widgets::picker::Picker::title` | struct_field | `src/widgets/picker.rs:47` |
| `widgets::picker::Picker::placeholder` | struct_field | `src/widgets/picker.rs:48` |
| `widgets::picker::Picker::query` | struct_field | `src/widgets/picker.rs:49` |
| `widgets::picker::Picker::items` | struct_field | `src/widgets/picker.rs:50` |
| `widgets::picker::Picker::cursor` | struct_field | `src/widgets/picker.rs:51` |
| `widgets::picker::Picker::scroll` | struct_field | `src/widgets/picker.rs:52` |
| `widgets::picker::Picker::width` | struct_field | `src/widgets/picker.rs:53` |
| `widgets::picker::Picker::max_rows` | struct_field | `src/widgets/picker.rs:54` |
| `widgets::picker::Picker::scope` | struct_field | `src/widgets/picker.rs:56` |
| `widgets::picker::Picker::empty_text` | struct_field | `src/widgets/picker.rs:57` |
| `widgets::picker::Picker::area` | struct_field | `src/widgets/picker.rs:58` |
| `widgets::picker::Picker::searchable` | struct_field | `src/widgets/picker.rs:60` |
| `widgets::picker::PickerEvent` | enum | `src/widgets/picker.rs:68` |
| `widgets::picker::PickerEvent::QueryChanged` | variant | `src/widgets/picker.rs:70` |
| `widgets::picker::PickerEvent::Chosen` | variant | `src/widgets/picker.rs:71` |
| `widgets::picker::PickerEvent::Chosen::0` | struct_field | `src/widgets/picker.rs:71` |
| `widgets::picker::PickerEvent::ChosenAlt` | variant | `src/widgets/picker.rs:73` |
| `widgets::picker::PickerEvent::ChosenAlt::0` | struct_field | `src/widgets/picker.rs:73` |
| `widgets::picker::PickerEvent::Secondary` | variant | `src/widgets/picker.rs:75` |
| `widgets::picker::PickerEvent::Secondary::0` | struct_field | `src/widgets/picker.rs:75` |
| `widgets::picker::PickerEvent::NextScope` | variant | `src/widgets/picker.rs:76` |
| `widgets::picker::PickerEvent::Cancelled` | variant | `src/widgets/picker.rs:77` |
| `widgets::picker::PickerEvent::Back` | variant | `src/widgets/picker.rs:79` |
| `widgets::picker::Picker::new` | function | `src/widgets/picker.rs:83` |
| `widgets::picker::Picker::set_items` | function | `src/widgets/picker.rs:103` |
| `widgets::picker::Picker::set_cursor` | function | `src/widgets/picker.rs:111` |
| `widgets::picker::Picker::row_id` | function | `src/widgets/picker.rs:117` |
| `widgets::picker::Picker::locate` | function | `src/widgets/picker.rs:120` |
| `widgets::picker::Picker::owns` | function | `src/widgets/picker.rs:123` |
| `widgets::picker::Picker::on_key` | function | `src/widgets/picker.rs:147` |
| `widgets::picker::Picker::on_click` | function | `src/widgets/picker.rs:223` |
| `widgets::picker::Picker::on_wheel` | function | `src/widgets/picker.rs:234` |
| `widgets::picker::Picker::render` | function | `src/widgets/picker.rs:244` |

### `src/widgets/progress.rs` (43 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::progress` | module | `src/widgets/progress.rs:1` |
| `widgets::progress::SPINNER` | constant | `src/widgets/progress.rs:8` |
| `widgets::progress::spinner_frame` | function | `src/widgets/progress.rs:10` |
| `widgets::progress::ProgressStatus` | enum | `src/widgets/progress.rs:15` |
| `widgets::progress::ProgressStatus::Active` | variant | `src/widgets/progress.rs:16` |
| `widgets::progress::ProgressStatus::Done` | variant | `src/widgets/progress.rs:17` |
| `widgets::progress::ProgressStatus::Error` | variant | `src/widgets/progress.rs:18` |
| `widgets::progress::ProgressStatus::Paused` | variant | `src/widgets/progress.rs:19` |
| `widgets::progress::render_bar` | function | `src/widgets/progress.rs:23` |
| `widgets::progress::METER_LOW_MAX` | constant | `src/widgets/progress.rs:86` |
| `widgets::progress::METER_MEDIUM_MAX` | constant | `src/widgets/progress.rs:87` |
| `widgets::progress::MeterLevel` | enum | `src/widgets/progress.rs:90` |
| `widgets::progress::MeterLevel::Low` | variant | `src/widgets/progress.rs:91` |
| `widgets::progress::MeterLevel::Medium` | variant | `src/widgets/progress.rs:92` |
| `widgets::progress::MeterLevel::High` | variant | `src/widgets/progress.rs:93` |
| `widgets::progress::MeterLevel::of` | function | `src/widgets/progress.rs:97` |
| `widgets::progress::MeterVisual` | enum | `src/widgets/progress.rs:110` |
| `widgets::progress::MeterVisual::Line` | variant | `src/widgets/progress.rs:113` |
| `widgets::progress::MeterVisual::Block` | variant | `src/widgets/progress.rs:116` |
| `widgets::progress::MeterTone` | enum | `src/widgets/progress.rs:123` |
| `widgets::progress::MeterTone::Normal` | variant | `src/widgets/progress.rs:126` |
| `widgets::progress::MeterTone::Level` | variant | `src/widgets/progress.rs:128` |
| `widgets::progress::MeterTone::Level::0` | struct_field | `src/widgets/progress.rs:128` |
| `widgets::progress::MeterTone::Warning` | variant | `src/widgets/progress.rs:130` |
| `widgets::progress::MeterTone::Exhausted` | variant | `src/widgets/progress.rs:132` |
| `widgets::progress::MeterTone::Stale` | variant | `src/widgets/progress.rs:134` |
| `widgets::progress::MeterTone::Refreshing` | variant | `src/widgets/progress.rs:136` |
| `widgets::progress::MeterTone::Error` | variant | `src/widgets/progress.rs:138` |
| `widgets::progress::MeterTone::Unknown` | variant | `src/widgets/progress.rs:140` |
| `widgets::progress::MeterTone::level` | function | `src/widgets/progress.rs:145` |
| `widgets::progress::Meter` | struct | `src/widgets/progress.rs:164` |
| `widgets::progress::Meter::used_pct` | struct_field | `src/widgets/progress.rs:165` |
| `widgets::progress::Meter::value` | struct_field | `src/widgets/progress.rs:166` |
| `widgets::progress::Meter::tone` | struct_field | `src/widgets/progress.rs:167` |
| `widgets::progress::Meter::visual` | struct_field | `src/widgets/progress.rs:168` |
| `widgets::progress::Meter::new` | function | `src/widgets/progress.rs:172` |
| `widgets::progress::Meter::value` | function | `src/widgets/progress.rs:180` |
| `widgets::progress::Meter::tone` | function | `src/widgets/progress.rs:184` |
| `widgets::progress::Meter::visual` | function | `src/widgets/progress.rs:188` |
| `widgets::progress::Meter::render` | function | `src/widgets/progress.rs:220` |
| `widgets::progress::render_meter` | function | `src/widgets/progress.rs:326` |
| `widgets::progress::render_indeterminate` | function | `src/widgets/progress.rs:342` |
| `widgets::progress::render_spinner` | function | `src/widgets/progress.rs:377` |

### `src/widgets/props.rs` (33 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::props` | module | `src/widgets/props.rs:1` |
| `widgets::props::Prop` | struct | `src/widgets/props.rs:17` |
| `widgets::props::Prop::label` | struct_field | `src/widgets/props.rs:18` |
| `widgets::props::Prop::value` | struct_field | `src/widgets/props.rs:19` |
| `widgets::props::Prop::tone` | struct_field | `src/widgets/props.rs:20` |
| `widgets::props::Prop::wrap` | struct_field | `src/widgets/props.rs:21` |
| `widgets::props::Prop::copyable` | struct_field | `src/widgets/props.rs:23` |
| `widgets::props::Prop::new` | function | `src/widgets/props.rs:27` |
| `widgets::props::Prop::copyable` | function | `src/widgets/props.rs:36` |
| `widgets::props::Prop::tone` | function | `src/widgets/props.rs:40` |
| `widgets::props::Prop::wrap` | function | `src/widgets/props.rs:44` |
| `widgets::props::render` | function | `src/widgets/props.rs:51` |
| `widgets::props::PropsEvent` | enum | `src/widgets/props.rs:92` |
| `widgets::props::PropsEvent::Copy` | variant | `src/widgets/props.rs:93` |
| `widgets::props::PropsEvent::Copy::0` | struct_field | `src/widgets/props.rs:93` |
| `widgets::props::PropsEvent::Activate` | variant | `src/widgets/props.rs:94` |
| `widgets::props::PropsEvent::Activate::0` | struct_field | `src/widgets/props.rs:94` |
| `widgets::props::PropsList` | struct | `src/widgets/props.rs:100` |
| `widgets::props::PropsList::id` | struct_field | `src/widgets/props.rs:101` |
| `widgets::props::PropsList::props` | struct_field | `src/widgets/props.rs:102` |
| `widgets::props::PropsList::cursor` | struct_field | `src/widgets/props.rs:103` |
| `widgets::props::PropsList::scroll` | struct_field | `src/widgets/props.rs:104` |
| `widgets::props::PropsList::area` | struct_field | `src/widgets/props.rs:105` |
| `widgets::props::PropsList::new` | function | `src/widgets/props.rs:109` |
| `widgets::props::PropsList::set_props` | function | `src/widgets/props.rs:120` |
| `widgets::props::PropsList::row_id` | function | `src/widgets/props.rs:126` |
| `widgets::props::PropsList::locate` | function | `src/widgets/props.rs:129` |
| `widgets::props::PropsList::owns` | function | `src/widgets/props.rs:132` |
| `widgets::props::PropsList::on_key` | function | `src/widgets/props.rs:141` |
| `widgets::props::PropsList::on_click` | function | `src/widgets/props.rs:168` |
| `widgets::props::PropsList::on_wheel` | function | `src/widgets/props.rs:176` |
| `widgets::props::PropsList::on_scrollbar` | function | `src/widgets/props.rs:181` |
| `widgets::props::PropsList::render` | function | `src/widgets/props.rs:193` |

### `src/widgets/scrollbar.rs` (7 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::scrollbar` | module | `src/widgets/scrollbar.rs:1` |
| `widgets::scrollbar::TRACK` | constant | `src/widgets/scrollbar.rs:8` |
| `widgets::scrollbar::THUMB` | constant | `src/widgets/scrollbar.rs:9` |
| `widgets::scrollbar::id_for` | function | `src/widgets/scrollbar.rs:12` |
| `widgets::scrollbar::render_vertical` | function | `src/widgets/scrollbar.rs:18` |
| `widgets::scrollbar::offset_for_click` | function | `src/widgets/scrollbar.rs:47` |
| `widgets::scrollbar::position_label` | function | `src/widgets/scrollbar.rs:53` |

### `src/widgets/segments.rs` (12 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::segments` | module | `src/widgets/segments.rs:1` |
| `widgets::segments::Segment` | struct | `src/widgets/segments.rs:15` |
| `widgets::segments::Segment::text` | struct_field | `src/widgets/segments.rs:16` |
| `widgets::segments::Segment::tone` | struct_field | `src/widgets/segments.rs:17` |
| `widgets::segments::Segment::bold` | struct_field | `src/widgets/segments.rs:18` |
| `widgets::segments::Segment::id` | struct_field | `src/widgets/segments.rs:19` |
| `widgets::segments::Segment::priority` | struct_field | `src/widgets/segments.rs:21` |
| `widgets::segments::Segment::new` | function | `src/widgets/segments.rs:25` |
| `widgets::segments::Segment::bold` | function | `src/widgets/segments.rs:34` |
| `widgets::segments::Segment::clickable` | function | `src/widgets/segments.rs:38` |
| `widgets::segments::Segment::priority` | function | `src/widgets/segments.rs:42` |
| `widgets::segments::render` | function | `src/widgets/segments.rs:50` |

### `src/widgets/select.rs` (26 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::select` | module | `src/widgets/select.rs:1` |
| `widgets::select::Select` | struct | `src/widgets/select.rs:15` |
| `widgets::select::Select::id` | struct_field | `src/widgets/select.rs:16` |
| `widgets::select::Select::label` | struct_field | `src/widgets/select.rs:17` |
| `widgets::select::Select::options` | struct_field | `src/widgets/select.rs:18` |
| `widgets::select::Select::selected` | struct_field | `src/widgets/select.rs:19` |
| `widgets::select::Select::cursor` | struct_field | `src/widgets/select.rs:20` |
| `widgets::select::Select::open` | struct_field | `src/widgets/select.rs:21` |
| `widgets::select::Select::disabled` | struct_field | `src/widgets/select.rs:22` |
| `widgets::select::Select::help` | struct_field | `src/widgets/select.rs:23` |
| `widgets::select::Select::area` | struct_field | `src/widgets/select.rs:24` |
| `widgets::select::SelectEvent` | enum | `src/widgets/select.rs:28` |
| `widgets::select::SelectEvent::Changed` | variant | `src/widgets/select.rs:29` |
| `widgets::select::SelectEvent::Changed::0` | struct_field | `src/widgets/select.rs:29` |
| `widgets::select::Select::HEIGHT` | assoc_const | `src/widgets/select.rs:33` |
| `widgets::select::Select::new` | function | `src/widgets/select.rs:35` |
| `widgets::select::Select::help` | function | `src/widgets/select.rs:48` |
| `widgets::select::Select::disabled` | function | `src/widgets/select.rs:52` |
| `widgets::select::Select::value` | function | `src/widgets/select.rs:56` |
| `widgets::select::Select::option_id` | function | `src/widgets/select.rs:62` |
| `widgets::select::Select::locate` | function | `src/widgets/select.rs:65` |
| `widgets::select::Select::owns` | function | `src/widgets/select.rs:68` |
| `widgets::select::Select::on_key` | function | `src/widgets/select.rs:72` |
| `widgets::select::Select::on_click` | function | `src/widgets/select.rs:125` |
| `widgets::select::Select::dismiss` | function | `src/widgets/select.rs:144` |
| `widgets::select::Select::render` | function | `src/widgets/select.rs:153` |

### `src/widgets/splitter.rs` (8 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::splitter` | module | `src/widgets/splitter.rs:1` |
| `widgets::splitter::Splitter` | struct | `src/widgets/splitter.rs:16` |
| `widgets::splitter::Splitter::id` | struct_field | `src/widgets/splitter.rs:17` |
| `widgets::splitter::Splitter::dir` | struct_field | `src/widgets/splitter.rs:18` |
| `widgets::splitter::Splitter::area` | struct_field | `src/widgets/splitter.rs:19` |
| `widgets::splitter::Splitter::new` | function | `src/widgets/splitter.rs:23` |
| `widgets::splitter::Splitter::render` | function | `src/widgets/splitter.rs:33` |
| `widgets::splitter::Splitter::on_drag` | function | `src/widgets/splitter.rs:56` |

### `src/widgets/statusbar.rs` (39 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::statusbar` | module | `src/widgets/statusbar.rs:1` |
| `widgets::statusbar::Emphasis` | enum | `src/widgets/statusbar.rs:19` |
| `widgets::statusbar::Emphasis::Plain` | variant | `src/widgets/statusbar.rs:21` |
| `widgets::statusbar::Emphasis::Strong` | variant | `src/widgets/statusbar.rs:23` |
| `widgets::statusbar::Emphasis::Chip` | variant | `src/widgets/statusbar.rs:26` |
| `widgets::statusbar::StatusItem` | struct | `src/widgets/statusbar.rs:30` |
| `widgets::statusbar::StatusItem::text` | struct_field | `src/widgets/statusbar.rs:31` |
| `widgets::statusbar::StatusItem::tone` | struct_field | `src/widgets/statusbar.rs:32` |
| `widgets::statusbar::StatusItem::priority` | struct_field | `src/widgets/statusbar.rs:34` |
| `widgets::statusbar::StatusItem::id` | struct_field | `src/widgets/statusbar.rs:35` |
| `widgets::statusbar::StatusItem::emphasis` | struct_field | `src/widgets/statusbar.rs:36` |
| `widgets::statusbar::StatusItem::meter` | struct_field | `src/widgets/statusbar.rs:38` |
| `widgets::statusbar::StatusItem::busy` | struct_field | `src/widgets/statusbar.rs:41` |
| `widgets::statusbar::STATUS_METER_TRACK` | constant | `src/widgets/statusbar.rs:45` |
| `widgets::statusbar::StatusItem::new` | function | `src/widgets/statusbar.rs:48` |
| `widgets::statusbar::StatusItem::busy` | function | `src/widgets/statusbar.rs:60` |
| `widgets::statusbar::StatusItem::meter` | function | `src/widgets/statusbar.rs:65` |
| `widgets::statusbar::StatusItem::priority` | function | `src/widgets/statusbar.rs:69` |
| `widgets::statusbar::StatusItem::clickable` | function | `src/widgets/statusbar.rs:73` |
| `widgets::statusbar::StatusItem::strong` | function | `src/widgets/statusbar.rs:77` |
| `widgets::statusbar::StatusItem::chip` | function | `src/widgets/statusbar.rs:81` |
| `widgets::statusbar::StatusItem::width` | function | `src/widgets/statusbar.rs:87` |
| `widgets::statusbar::Group` | enum | `src/widgets/statusbar.rs:103` |
| `widgets::statusbar::Group::Left` | variant | `src/widgets/statusbar.rs:104` |
| `widgets::statusbar::Group::Center` | variant | `src/widgets/statusbar.rs:105` |
| `widgets::statusbar::Group::Right` | variant | `src/widgets/statusbar.rs:106` |
| `widgets::statusbar::Placed` | struct | `src/widgets/statusbar.rs:111` |
| `widgets::statusbar::Placed::group` | struct_field | `src/widgets/statusbar.rs:112` |
| `widgets::statusbar::Placed::index` | struct_field | `src/widgets/statusbar.rs:113` |
| `widgets::statusbar::Placed::x` | struct_field | `src/widgets/statusbar.rs:114` |
| `widgets::statusbar::Placed::width` | struct_field | `src/widgets/statusbar.rs:115` |
| `widgets::statusbar::Placed::text` | struct_field | `src/widgets/statusbar.rs:117` |
| `widgets::statusbar::StatusBar` | struct | `src/widgets/statusbar.rs:121` |
| `widgets::statusbar::StatusBar::left` | struct_field | `src/widgets/statusbar.rs:122` |
| `widgets::statusbar::StatusBar::center` | struct_field | `src/widgets/statusbar.rs:123` |
| `widgets::statusbar::StatusBar::right` | struct_field | `src/widgets/statusbar.rs:124` |
| `widgets::statusbar::StatusBar::new` | function | `src/widgets/statusbar.rs:132` |
| `widgets::statusbar::StatusBar::layout` | function | `src/widgets/statusbar.rs:145` |
| `widgets::statusbar::StatusBar::render` | function | `src/widgets/statusbar.rs:274` |

### `src/widgets/steps.rs` (38 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::steps` | module | `src/widgets/steps.rs:1` |
| `widgets::steps::StepState` | enum | `src/widgets/steps.rs:18` |
| `widgets::steps::StepState::Queued` | variant | `src/widgets/steps.rs:20` |
| `widgets::steps::StepState::Running` | variant | `src/widgets/steps.rs:21` |
| `widgets::steps::StepState::Done` | variant | `src/widgets/steps.rs:22` |
| `widgets::steps::StepState::Skipped` | variant | `src/widgets/steps.rs:23` |
| `widgets::steps::StepState::Failed` | variant | `src/widgets/steps.rs:24` |
| `widgets::steps::StepState::Blocked` | variant | `src/widgets/steps.rs:25` |
| `widgets::steps::StepState::label` | function | `src/widgets/steps.rs:29` |
| `widgets::steps::StepState::terminal` | function | `src/widgets/steps.rs:40` |
| `widgets::steps::Step` | struct | `src/widgets/steps.rs:49` |
| `widgets::steps::Step::label` | struct_field | `src/widgets/steps.rs:50` |
| `widgets::steps::Step::state` | struct_field | `src/widgets/steps.rs:51` |
| `widgets::steps::Step::meta` | struct_field | `src/widgets/steps.rs:53` |
| `widgets::steps::Step::new` | function | `src/widgets/steps.rs:57` |
| `widgets::steps::StepRail` | struct | `src/widgets/steps.rs:67` |
| `widgets::steps::StepRail::id` | struct_field | `src/widgets/steps.rs:68` |
| `widgets::steps::StepRail::steps` | struct_field | `src/widgets/steps.rs:69` |
| `widgets::steps::StepRail::selectable` | struct_field | `src/widgets/steps.rs:70` |
| `widgets::steps::StepRail::cursor` | struct_field | `src/widgets/steps.rs:71` |
| `widgets::steps::StepRail::scroll` | struct_field | `src/widgets/steps.rs:72` |
| `widgets::steps::StepRail::area` | struct_field | `src/widgets/steps.rs:73` |
| `widgets::steps::StepRail::numbered` | struct_field | `src/widgets/steps.rs:75` |
| `widgets::steps::StepRail::new` | function | `src/widgets/steps.rs:79` |
| `widgets::steps::StepRail::selectable` | function | `src/widgets/steps.rs:92` |
| `widgets::steps::StepRail::set_state` | function | `src/widgets/steps.rs:97` |
| `widgets::steps::StepRail::set_meta` | function | `src/widgets/steps.rs:103` |
| `widgets::steps::StepRail::frontier` | function | `src/widgets/steps.rs:110` |
| `widgets::steps::StepRail::counts` | function | `src/widgets/steps.rs:115` |
| `widgets::steps::StepRail::failed` | function | `src/widgets/steps.rs:124` |
| `widgets::steps::StepRail::row_id` | function | `src/widgets/steps.rs:128` |
| `widgets::steps::StepRail::locate` | function | `src/widgets/steps.rs:132` |
| `widgets::steps::StepRail::owns` | function | `src/widgets/steps.rs:136` |
| `widgets::steps::StepRail::on_key` | function | `src/widgets/steps.rs:145` |
| `widgets::steps::StepRail::on_click` | function | `src/widgets/steps.rs:161` |
| `widgets::steps::StepRail::on_wheel` | function | `src/widgets/steps.rs:169` |
| `widgets::steps::StepRail::on_scrollbar` | function | `src/widgets/steps.rs:174` |
| `widgets::steps::StepRail::render` | function | `src/widgets/steps.rs:186` |

### `src/widgets/table.rs` (83 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::table` | module | `src/widgets/table.rs:1` |
| `widgets::table::SortDir` | enum | `src/widgets/table.rs:15` |
| `widgets::table::SortDir::Asc` | variant | `src/widgets/table.rs:16` |
| `widgets::table::SortDir::Desc` | variant | `src/widgets/table.rs:17` |
| `widgets::table::Align` | enum | `src/widgets/table.rs:21` |
| `widgets::table::Align::Left` | variant | `src/widgets/table.rs:22` |
| `widgets::table::Align::Right` | variant | `src/widgets/table.rs:23` |
| `widgets::table::Column` | struct | `src/widgets/table.rs:27` |
| `widgets::table::Column::title` | struct_field | `src/widgets/table.rs:28` |
| `widgets::table::Column::width` | struct_field | `src/widgets/table.rs:29` |
| `widgets::table::Column::align` | struct_field | `src/widgets/table.rs:30` |
| `widgets::table::Column::editable` | struct_field | `src/widgets/table.rs:31` |
| `widgets::table::Column::sortable` | struct_field | `src/widgets/table.rs:32` |
| `widgets::table::Column::new` | function | `src/widgets/table.rs:36` |
| `widgets::table::Column::right` | function | `src/widgets/table.rs:45` |
| `widgets::table::Column::editable` | function | `src/widgets/table.rs:49` |
| `widgets::table::Column::min_width` | function | `src/widgets/table.rs:53` |
| `widgets::table::Cell` | struct | `src/widgets/table.rs:63` |
| `widgets::table::Cell::text` | struct_field | `src/widgets/table.rs:64` |
| `widgets::table::Cell::error` | struct_field | `src/widgets/table.rs:65` |
| `widgets::table::Cell::tone` | struct_field | `src/widgets/table.rs:66` |
| `widgets::table::Tone` | use | `src/widgets/table.rs:69` |
| `widgets::table::Cell::new` | function | `src/widgets/table.rs:72` |
| `widgets::table::Cell::tone` | function | `src/widgets/table.rs:79` |
| `widgets::table::EditState` | struct | `src/widgets/table.rs:86` |
| `widgets::table::EditState::row` | struct_field | `src/widgets/table.rs:88` |
| `widgets::table::EditState::col` | struct_field | `src/widgets/table.rs:89` |
| `widgets::table::EditState::buffer` | struct_field | `src/widgets/table.rs:90` |
| `widgets::table::EditState::error` | struct_field | `src/widgets/table.rs:91` |
| `widgets::table::DataTable` | struct | `src/widgets/table.rs:97` |
| `widgets::table::DataTable::id` | struct_field | `src/widgets/table.rs:98` |
| `widgets::table::DataTable::columns` | struct_field | `src/widgets/table.rs:99` |
| `widgets::table::DataTable::rows` | struct_field | `src/widgets/table.rs:100` |
| `widgets::table::DataTable::sort` | struct_field | `src/widgets/table.rs:104` |
| `widgets::table::DataTable::cursor_row` | struct_field | `src/widgets/table.rs:105` |
| `widgets::table::DataTable::cursor_col` | struct_field | `src/widgets/table.rs:106` |
| `widgets::table::DataTable::cell_nav` | struct_field | `src/widgets/table.rs:108` |
| `widgets::table::DataTable::selected` | struct_field | `src/widgets/table.rs:109` |
| `widgets::table::DataTable::scroll` | struct_field | `src/widgets/table.rs:110` |
| `widgets::table::DataTable::hscroll` | struct_field | `src/widgets/table.rs:111` |
| `widgets::table::DataTable::edit` | struct_field | `src/widgets/table.rs:112` |
| `widgets::table::DataTable::validator` | struct_field | `src/widgets/table.rs:113` |
| `widgets::table::DataTable::empty_text` | struct_field | `src/widgets/table.rs:114` |
| `widgets::table::DataTable::area` | struct_field | `src/widgets/table.rs:115` |
| `widgets::table::DataTable::numeric` | struct_field | `src/widgets/table.rs:119` |
| `widgets::table::TableEvent` | enum | `src/widgets/table.rs:123` |
| `widgets::table::TableEvent::Committed` | variant | `src/widgets/table.rs:124` |
| `widgets::table::TableEvent::Committed::row` | struct_field | `src/widgets/table.rs:125` |
| `widgets::table::TableEvent::Committed::col` | struct_field | `src/widgets/table.rs:126` |
| `widgets::table::TableEvent::Cancelled` | variant | `src/widgets/table.rs:128` |
| `widgets::table::TableEvent::Activated` | variant | `src/widgets/table.rs:129` |
| `widgets::table::TableEvent::Activated::0` | struct_field | `src/widgets/table.rs:129` |
| `widgets::table::TableEvent::LeaveForward` | variant | `src/widgets/table.rs:131` |
| `widgets::table::TableEvent::LeaveBackward` | variant | `src/widgets/table.rs:132` |
| `widgets::table::DataTable::new` | function | `src/widgets/table.rs:136` |
| `widgets::table::DataTable::cell_nav` | function | `src/widgets/table.rs:161` |
| `widgets::table::DataTable::numeric` | function | `src/widgets/table.rs:165` |
| `widgets::table::DataTable::validator` | function | `src/widgets/table.rs:173` |
| `widgets::table::DataTable::empty_text` | function | `src/widgets/table.rs:177` |
| `widgets::table::DataTable::len` | function | `src/widgets/table.rs:182` |
| `widgets::table::DataTable::is_empty` | function | `src/widgets/table.rs:185` |
| `widgets::table::DataTable::is_editing` | function | `src/widgets/table.rs:188` |
| `widgets::table::DataTable::source_row` | function | `src/widgets/table.rs:193` |
| `widgets::table::DataTable::header_id` | function | `src/widgets/table.rs:197` |
| `widgets::table::DataTable::row_id` | function | `src/widgets/table.rs:200` |
| `widgets::table::DataTable::cell_id` | function | `src/widgets/table.rs:203` |
| `widgets::table::DataTable::set_rows` | function | `src/widgets/table.rs:209` |
| `widgets::table::DataTable::sort_by` | function | `src/widgets/table.rs:242` |
| `widgets::table::DataTable::begin_edit` | function | `src/widgets/table.rs:290` |
| `widgets::table::DataTable::commit_edit` | function | `src/widgets/table.rs:309` |
| `widgets::table::DataTable::cancel_edit` | function | `src/widgets/table.rs:339` |
| `widgets::table::DataTable::on_key` | function | `src/widgets/table.rs:343` |
| `widgets::table::DataTable::on_paste` | function | `src/widgets/table.rs:450` |
| `widgets::table::DataTable::on_click_header` | function | `src/widgets/table.rs:460` |
| `widgets::table::DataTable::on_click_cell` | function | `src/widgets/table.rs:469` |
| `widgets::table::DataTable::on_wheel_h` | function | `src/widgets/table.rs:515` |
| `widgets::table::DataTable::on_wheel` | function | `src/widgets/table.rs:525` |
| `widgets::table::DataTable::on_scrollbar` | function | `src/widgets/table.rs:530` |
| `widgets::table::DataTable::render` | function | `src/widgets/table.rs:571` |
| `widgets::table::DataTable::locate` | function | `src/widgets/table.rs:810` |
| `widgets::table::DataTable::owns` | function | `src/widgets/table.rs:824` |
| `widgets::table::DataTable::locate_header` | function | `src/widgets/table.rs:831` |
| `widgets::table::DataTable::edit_error` | function | `src/widgets/table.rs:835` |

### `src/widgets/tabs.rs` (43 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::tabs` | module | `src/widgets/tabs.rs:1` |
| `widgets::tabs::TabItem` | struct | `src/widgets/tabs.rs:19` |
| `widgets::tabs::TabItem::label` | struct_field | `src/widgets/tabs.rs:20` |
| `widgets::tabs::TabItem::dirty` | struct_field | `src/widgets/tabs.rs:21` |
| `widgets::tabs::TabItem::busy` | struct_field | `src/widgets/tabs.rs:22` |
| `widgets::tabs::TabItem::error` | struct_field | `src/widgets/tabs.rs:23` |
| `widgets::tabs::TabItem::closable` | struct_field | `src/widgets/tabs.rs:24` |
| `widgets::tabs::TabItem::prefix` | struct_field | `src/widgets/tabs.rs:26` |
| `widgets::tabs::TabItem::suffix` | struct_field | `src/widgets/tabs.rs:28` |
| `widgets::tabs::TabItem::new` | function | `src/widgets/tabs.rs:32` |
| `widgets::tabs::TabItem::closable` | function | `src/widgets/tabs.rs:38` |
| `widgets::tabs::TabItem::prefix` | function | `src/widgets/tabs.rs:42` |
| `widgets::tabs::TabItem::suffix` | function | `src/widgets/tabs.rs:46` |
| `widgets::tabs::Tabs` | struct | `src/widgets/tabs.rs:53` |
| `widgets::tabs::Tabs::id` | struct_field | `src/widgets/tabs.rs:54` |
| `widgets::tabs::Tabs::items` | struct_field | `src/widgets/tabs.rs:55` |
| `widgets::tabs::Tabs::active` | struct_field | `src/widgets/tabs.rs:56` |
| `widgets::tabs::Tabs::cursor` | struct_field | `src/widgets/tabs.rs:57` |
| `widgets::tabs::Tabs::first` | struct_field | `src/widgets/tabs.rs:59` |
| `widgets::tabs::Tabs::areas` | struct_field | `src/widgets/tabs.rs:60` |
| `widgets::tabs::Tabs::allow_new` | struct_field | `src/widgets/tabs.rs:62` |
| `widgets::tabs::Tabs::quiet` | struct_field | `src/widgets/tabs.rs:65` |
| `widgets::tabs::TabEvent` | enum | `src/widgets/tabs.rs:71` |
| `widgets::tabs::TabEvent::Activated` | variant | `src/widgets/tabs.rs:72` |
| `widgets::tabs::TabEvent::Activated::0` | struct_field | `src/widgets/tabs.rs:72` |
| `widgets::tabs::TabEvent::Close` | variant | `src/widgets/tabs.rs:73` |
| `widgets::tabs::TabEvent::Close::0` | struct_field | `src/widgets/tabs.rs:73` |
| `widgets::tabs::TabEvent::New` | variant | `src/widgets/tabs.rs:74` |
| `widgets::tabs::Tabs::new` | function | `src/widgets/tabs.rs:78` |
| `widgets::tabs::Tabs::with_items` | function | `src/widgets/tabs.rs:92` |
| `widgets::tabs::Tabs::tab_id` | function | `src/widgets/tabs.rs:106` |
| `widgets::tabs::Tabs::close_id` | function | `src/widgets/tabs.rs:109` |
| `widgets::tabs::Tabs::new_id` | function | `src/widgets/tabs.rs:112` |
| `widgets::tabs::Tabs::left_id` | function | `src/widgets/tabs.rs:115` |
| `widgets::tabs::Tabs::right_id` | function | `src/widgets/tabs.rs:118` |
| `widgets::tabs::Tabs::locate` | function | `src/widgets/tabs.rs:122` |
| `widgets::tabs::Tabs::owns` | function | `src/widgets/tabs.rs:126` |
| `widgets::tabs::Tabs::hidden` | function | `src/widgets/tabs.rs:136` |
| `widgets::tabs::Tabs::set_active` | function | `src/widgets/tabs.rs:141` |
| `widgets::tabs::Tabs::remove` | function | `src/widgets/tabs.rs:152` |
| `widgets::tabs::Tabs::on_key` | function | `src/widgets/tabs.rs:173` |
| `widgets::tabs::Tabs::on_click` | function | `src/widgets/tabs.rs:216` |
| `widgets::tabs::Tabs::render` | function | `src/widgets/tabs.rs:258` |

### `src/widgets/textarea.rs` (27 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::textarea` | module | `src/widgets/textarea.rs:1` |
| `widgets::textarea::TextArea` | struct | `src/widgets/textarea.rs:22` |
| `widgets::textarea::TextArea::id` | struct_field | `src/widgets/textarea.rs:23` |
| `widgets::textarea::TextArea::label` | struct_field | `src/widgets/textarea.rs:24` |
| `widgets::textarea::TextArea::placeholder` | struct_field | `src/widgets/textarea.rs:25` |
| `widgets::textarea::TextArea::buffer` | struct_field | `src/widgets/textarea.rs:26` |
| `widgets::textarea::TextArea::disabled` | struct_field | `src/widgets/textarea.rs:27` |
| `widgets::textarea::TextArea::error` | struct_field | `src/widgets/textarea.rs:28` |
| `widgets::textarea::TextArea::help` | struct_field | `src/widgets/textarea.rs:29` |
| `widgets::textarea::TextArea::editing` | struct_field | `src/widgets/textarea.rs:30` |
| `widgets::textarea::TextArea::scroll` | struct_field | `src/widgets/textarea.rs:31` |
| `widgets::textarea::TextArea::area` | struct_field | `src/widgets/textarea.rs:32` |
| `widgets::textarea::TextArea::rows` | struct_field | `src/widgets/textarea.rs:38` |
| `widgets::textarea::TextArea::new` | function | `src/widgets/textarea.rs:42` |
| `widgets::textarea::TextArea::value` | function | `src/widgets/textarea.rs:62` |
| `widgets::textarea::TextArea::placeholder` | function | `src/widgets/textarea.rs:67` |
| `widgets::textarea::TextArea::disabled` | function | `src/widgets/textarea.rs:71` |
| `widgets::textarea::TextArea::error` | function | `src/widgets/textarea.rs:75` |
| `widgets::textarea::TextArea::help` | function | `src/widgets/textarea.rs:79` |
| `widgets::textarea::TextArea::height` | function | `src/widgets/textarea.rs:84` |
| `widgets::textarea::TextArea::begin_edit` | function | `src/widgets/textarea.rs:88` |
| `widgets::textarea::TextArea::commit` | function | `src/widgets/textarea.rs:95` |
| `widgets::textarea::TextArea::on_key` | function | `src/widgets/textarea.rs:100` |
| `widgets::textarea::TextArea::on_paste` | function | `src/widgets/textarea.rs:168` |
| `widgets::textarea::TextArea::on_click` | function | `src/widgets/textarea.rs:177` |
| `widgets::textarea::TextArea::on_wheel` | function | `src/widgets/textarea.rs:196` |
| `widgets::textarea::TextArea::render` | function | `src/widgets/textarea.rs:201` |

### `src/widgets/tree.rs` (63 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::tree` | module | `src/widgets/tree.rs:1` |
| `widgets::tree::TreeNode` | struct | `src/widgets/tree.rs:14` |
| `widgets::tree::TreeNode::label` | struct_field | `src/widgets/tree.rs:15` |
| `widgets::tree::TreeNode::children` | struct_field | `src/widgets/tree.rs:16` |
| `widgets::tree::TreeNode::meta` | struct_field | `src/widgets/tree.rs:17` |
| `widgets::tree::TreeNode::glyph` | struct_field | `src/widgets/tree.rs:19` |
| `widgets::tree::TreeNode::lazy` | struct_field | `src/widgets/tree.rs:22` |
| `widgets::tree::TreeNode::busy` | struct_field | `src/widgets/tree.rs:24` |
| `widgets::tree::TreeNode::note` | struct_field | `src/widgets/tree.rs:26` |
| `widgets::tree::TreeNode::leaf` | function | `src/widgets/tree.rs:30` |
| `widgets::tree::TreeNode::leaf_meta` | function | `src/widgets/tree.rs:42` |
| `widgets::tree::TreeNode::dir` | function | `src/widgets/tree.rs:49` |
| `widgets::tree::TreeNode::lazy` | function | `src/widgets/tree.rs:57` |
| `widgets::tree::TreeNode::glyph` | function | `src/widgets/tree.rs:64` |
| `widgets::tree::TreeNode::note` | function | `src/widgets/tree.rs:69` |
| `widgets::tree::TreeNode::meta` | function | `src/widgets/tree.rs:76` |
| `widgets::tree::Path` | type_alias | `src/widgets/tree.rs:83` |
| `widgets::tree::FlatRow` | struct | `src/widgets/tree.rs:86` |
| `widgets::tree::FlatRow::path` | struct_field | `src/widgets/tree.rs:87` |
| `widgets::tree::FlatRow::depth` | struct_field | `src/widgets/tree.rs:88` |
| `widgets::tree::FlatRow::label` | struct_field | `src/widgets/tree.rs:89` |
| `widgets::tree::FlatRow::meta` | struct_field | `src/widgets/tree.rs:90` |
| `widgets::tree::FlatRow::has_children` | struct_field | `src/widgets/tree.rs:91` |
| `widgets::tree::FlatRow::expanded` | struct_field | `src/widgets/tree.rs:92` |
| `widgets::tree::FlatRow::glyph` | struct_field | `src/widgets/tree.rs:93` |
| `widgets::tree::FlatRow::busy` | struct_field | `src/widgets/tree.rs:94` |
| `widgets::tree::FlatRow::note` | struct_field | `src/widgets/tree.rs:95` |
| `widgets::tree::TreeEvent` | enum | `src/widgets/tree.rs:99` |
| `widgets::tree::TreeEvent::Expand` | variant | `src/widgets/tree.rs:101` |
| `widgets::tree::TreeEvent::Expand::0` | struct_field | `src/widgets/tree.rs:101` |
| `widgets::tree::TreeEvent::Activate` | variant | `src/widgets/tree.rs:103` |
| `widgets::tree::TreeEvent::Activate::0` | struct_field | `src/widgets/tree.rs:103` |
| `widgets::tree::TreeView` | struct | `src/widgets/tree.rs:107` |
| `widgets::tree::TreeView::id` | struct_field | `src/widgets/tree.rs:108` |
| `widgets::tree::TreeView::nodes` | struct_field | `src/widgets/tree.rs:109` |
| `widgets::tree::TreeView::expanded` | struct_field | `src/widgets/tree.rs:110` |
| `widgets::tree::TreeView::cursor` | struct_field | `src/widgets/tree.rs:111` |
| `widgets::tree::TreeView::selected` | struct_field | `src/widgets/tree.rs:112` |
| `widgets::tree::TreeView::scroll` | struct_field | `src/widgets/tree.rs:113` |
| `widgets::tree::TreeView::area` | struct_field | `src/widgets/tree.rs:114` |
| `widgets::tree::TreeView::filter` | struct_field | `src/widgets/tree.rs:117` |
| `widgets::tree::TreeView::new` | function | `src/widgets/tree.rs:121` |
| `widgets::tree::TreeView::rows` | function | `src/widgets/tree.rs:141` |
| `widgets::tree::TreeView::flatten` | function | `src/widgets/tree.rs:145` |
| `widgets::tree::TreeView::node` | function | `src/widgets/tree.rs:207` |
| `widgets::tree::TreeView::node_mut` | function | `src/widgets/tree.rs:217` |
| `widgets::tree::TreeView::set_children` | function | `src/widgets/tree.rs:227` |
| `widgets::tree::TreeView::set_busy` | function | `src/widgets/tree.rs:237` |
| `widgets::tree::TreeView::set_filter` | function | `src/widgets/tree.rs:244` |
| `widgets::tree::TreeView::reveal` | function | `src/widgets/tree.rs:252` |
| `widgets::tree::TreeView::row_id` | function | `src/widgets/tree.rs:262` |
| `widgets::tree::TreeView::toggle_id` | function | `src/widgets/tree.rs:266` |
| `widgets::tree::TreeView::toggle` | function | `src/widgets/tree.rs:275` |
| `widgets::tree::TreeView::expand_all` | function | `src/widgets/tree.rs:302` |
| `widgets::tree::TreeView::collapse_all` | function | `src/widgets/tree.rs:317` |
| `widgets::tree::TreeView::on_key` | function | `src/widgets/tree.rs:322` |
| `widgets::tree::TreeView::on_click_row` | function | `src/widgets/tree.rs:403` |
| `widgets::tree::TreeView::on_click_toggle` | function | `src/widgets/tree.rs:421` |
| `widgets::tree::TreeView::on_wheel` | function | `src/widgets/tree.rs:426` |
| `widgets::tree::TreeView::locate` | function | `src/widgets/tree.rs:432` |
| `widgets::tree::TreeView::owns` | function | `src/widgets/tree.rs:444` |
| `widgets::tree::TreeView::on_scrollbar` | function | `src/widgets/tree.rs:448` |
| `widgets::tree::TreeView::render` | function | `src/widgets/tree.rs:460` |

### `src/widgets/viewport.rs` (64 exported entries)

| Exported item | Kind | Exact source |
|---|---|---|
| `widgets::viewport` | module | `src/widgets/viewport.rs:1` |
| `widgets::viewport::Span` | struct | `src/widgets/viewport.rs:23` |
| `widgets::viewport::Span::text` | struct_field | `src/widgets/viewport.rs:24` |
| `widgets::viewport::Span::tone` | struct_field | `src/widgets/viewport.rs:25` |
| `widgets::viewport::Span::bold` | struct_field | `src/widgets/viewport.rs:26` |
| `widgets::viewport::Span::italic` | struct_field | `src/widgets/viewport.rs:27` |
| `widgets::viewport::Span::underline` | struct_field | `src/widgets/viewport.rs:28` |
| `widgets::viewport::Span::reversed` | struct_field | `src/widgets/viewport.rs:30` |
| `widgets::viewport::Span::new` | function | `src/widgets/viewport.rs:34` |
| `widgets::viewport::Span::plain` | function | `src/widgets/viewport.rs:44` |
| `widgets::viewport::Span::muted` | function | `src/widgets/viewport.rs:47` |
| `widgets::viewport::Span::bold` | function | `src/widgets/viewport.rs:50` |
| `widgets::viewport::Span::italic` | function | `src/widgets/viewport.rs:54` |
| `widgets::viewport::Span::underline` | function | `src/widgets/viewport.rs:58` |
| `widgets::viewport::Span::reversed` | function | `src/widgets/viewport.rs:62` |
| `widgets::viewport::Line` | type_alias | `src/widgets/viewport.rs:68` |
| `widgets::viewport::line_text` | function | `src/widgets/viewport.rs:70` |
| `widgets::viewport::CellPos` | struct | `src/widgets/viewport.rs:76` |
| `widgets::viewport::CellPos::line` | struct_field | `src/widgets/viewport.rs:77` |
| `widgets::viewport::CellPos::col` | struct_field | `src/widgets/viewport.rs:78` |
| `widgets::viewport::ViewportEvent` | enum | `src/widgets/viewport.rs:82` |
| `widgets::viewport::ViewportEvent::Copy` | variant | `src/widgets/viewport.rs:84` |
| `widgets::viewport::ViewportEvent::Copy::0` | struct_field | `src/widgets/viewport.rs:84` |
| `widgets::viewport::ViewportEvent::SelectionChanged` | variant | `src/widgets/viewport.rs:85` |
| `widgets::viewport::ViewportEvent::FollowChanged` | variant | `src/widgets/viewport.rs:86` |
| `widgets::viewport::ViewportEvent::FollowChanged::0` | struct_field | `src/widgets/viewport.rs:86` |
| `widgets::viewport::TextViewport` | struct | `src/widgets/viewport.rs:109` |
| `widgets::viewport::TextViewport::id` | struct_field | `src/widgets/viewport.rs:110` |
| `widgets::viewport::TextViewport::lines` | struct_field | `src/widgets/viewport.rs:111` |
| `widgets::viewport::TextViewport::max_lines` | struct_field | `src/widgets/viewport.rs:112` |
| `widgets::viewport::TextViewport::scroll` | struct_field | `src/widgets/viewport.rs:113` |
| `widgets::viewport::TextViewport::follow` | struct_field | `src/widgets/viewport.rs:114` |
| `widgets::viewport::TextViewport::wrap` | struct_field | `src/widgets/viewport.rs:115` |
| `widgets::viewport::TextViewport::caret` | struct_field | `src/widgets/viewport.rs:117` |
| `widgets::viewport::TextViewport::caret_visible` | struct_field | `src/widgets/viewport.rs:118` |
| `widgets::viewport::TextViewport::area` | struct_field | `src/widgets/viewport.rs:119` |
| `widgets::viewport::TextViewport::new` | function | `src/widgets/viewport.rs:132` |
| `widgets::viewport::TextViewport::with_lines` | function | `src/widgets/viewport.rs:155` |
| `widgets::viewport::TextViewport::wrap` | function | `src/widgets/viewport.rs:161` |
| `widgets::viewport::TextViewport::max_lines` | function | `src/widgets/viewport.rs:166` |
| `widgets::viewport::TextViewport::push` | function | `src/widgets/viewport.rs:171` |
| `widgets::viewport::TextViewport::set_lines` | function | `src/widgets/viewport.rs:189` |
| `widgets::viewport::TextViewport::replace_last` | function | `src/widgets/viewport.rs:196` |
| `widgets::viewport::TextViewport::clear` | function | `src/widgets/viewport.rs:205` |
| `widgets::viewport::TextViewport::len` | function | `src/widgets/viewport.rs:213` |
| `widgets::viewport::TextViewport::is_empty` | function | `src/widgets/viewport.rs:217` |
| `widgets::viewport::TextViewport::selection` | function | `src/widgets/viewport.rs:234` |
| `widgets::viewport::TextViewport::set_area` | function | `src/widgets/viewport.rs:245` |
| `widgets::viewport::TextViewport::has_anchor` | function | `src/widgets/viewport.rs:273` |
| `widgets::viewport::TextViewport::has_selection` | function | `src/widgets/viewport.rs:277` |
| `widgets::viewport::TextViewport::clear_selection` | function | `src/widgets/viewport.rs:281` |
| `widgets::viewport::TextViewport::is_at_tail` | function | `src/widgets/viewport.rs:289` |
| `widgets::viewport::TextViewport::scrollback_depth` | function | `src/widgets/viewport.rs:294` |
| `widgets::viewport::TextViewport::set_follow` | function | `src/widgets/viewport.rs:298` |
| `widgets::viewport::TextViewport::pos_at` | function | `src/widgets/viewport.rs:419` |
| `widgets::viewport::TextViewport::selected_text` | function | `src/widgets/viewport.rs:435` |
| `widgets::viewport::TextViewport::select_word_at` | function | `src/widgets/viewport.rs:463` |
| `widgets::viewport::TextViewport::on_click` | function | `src/widgets/viewport.rs:503` |
| `widgets::viewport::TextViewport::on_drag` | function | `src/widgets/viewport.rs:516` |
| `widgets::viewport::TextViewport::on_wheel` | function | `src/widgets/viewport.rs:539` |
| `widgets::viewport::TextViewport::on_scrollbar` | function | `src/widgets/viewport.rs:545` |
| `widgets::viewport::TextViewport::owns` | function | `src/widgets/viewport.rs:558` |
| `widgets::viewport::TextViewport::on_key` | function | `src/widgets/viewport.rs:563` |
| `widgets::viewport::TextViewport::render` | function | `src/widgets/viewport.rs:606` |

## Derived trait contracts

These 572 `(type, trait)` contracts are deliberately separate from the exported-entry total. They originate in `derive` attributes on exported types. Trait-provided methods, inferred auto traits and external blanket implementations are not expanded into extra source entries.

| Public type | Derived traits and exact source |
|---|---|
| `core::event::Outcome` | `Clone` (`src/core/event.rs:13`), `Copy` (`src/core/event.rs:13`), `Debug` (`src/core/event.rs:13`), `Default` (`src/core/event.rs:13`), `Eq` (`src/core/event.rs:13`), `PartialEq` (`src/core/event.rs:13`), `StructuralPartialEq` (`src/core/event.rs:13`) |
| `core::event::Input` | `Clone` (`src/core/event.rs:39`), `Debug` (`src/core/event.rs:39`), `Eq` (`src/core/event.rs:39`), `PartialEq` (`src/core/event.rs:39`), `StructuralPartialEq` (`src/core/event.rs:39`) |
| `core::event::Key` | `Clone` (`src/core/event.rs:48`), `Copy` (`src/core/event.rs:48`), `Debug` (`src/core/event.rs:48`), `Eq` (`src/core/event.rs:48`), `PartialEq` (`src/core/event.rs:48`), `StructuralPartialEq` (`src/core/event.rs:48`) |
| `core::event::MouseKind` | `Clone` (`src/core/event.rs:78`), `Copy` (`src/core/event.rs:78`), `Debug` (`src/core/event.rs:78`), `Eq` (`src/core/event.rs:78`), `PartialEq` (`src/core/event.rs:78`), `StructuralPartialEq` (`src/core/event.rs:78`) |
| `core::event::Mouse` | `Clone` (`src/core/event.rs:92`), `Copy` (`src/core/event.rs:92`), `Debug` (`src/core/event.rs:92`), `Eq` (`src/core/event.rs:92`), `PartialEq` (`src/core/event.rs:92`), `StructuralPartialEq` (`src/core/event.rs:92`) |
| `core::focus::FocusRing` | `Clone` (`src/core/focus.rs:9`), `Debug` (`src/core/focus.rs:9`), `Default` (`src/core/focus.rs:9`) |
| `core::focus::Focus` | `Clone` (`src/core/focus.rs:63`), `Debug` (`src/core/focus.rs:63`), `Default` (`src/core/focus.rs:63`) |
| `core::hit::HitRegion` | `Clone` (`src/core/hit.rs:12`), `Copy` (`src/core/hit.rs:12`), `Debug` (`src/core/hit.rs:12`), `Eq` (`src/core/hit.rs:12`), `PartialEq` (`src/core/hit.rs:12`), `StructuralPartialEq` (`src/core/hit.rs:12`) |
| `core::hit::HitRegistry` | `Clone` (`src/core/hit.rs:21`), `Debug` (`src/core/hit.rs:21`), `Default` (`src/core/hit.rs:21`) |
| `core::id::WidgetId` | `Clone` (`src/core/id.rs:9`), `Copy` (`src/core/id.rs:9`), `Eq` (`src/core/id.rs:9`), `Hash` (`src/core/id.rs:9`), `Ord` (`src/core/id.rs:9`), `PartialEq` (`src/core/id.rs:9`), `PartialOrd` (`src/core/id.rs:9`), `StructuralPartialEq` (`src/core/id.rs:9`) |
| `core::scroll::ScrollState` | `Clone` (`src/core/scroll.rs:7`), `Copy` (`src/core/scroll.rs:7`), `Debug` (`src/core/scroll.rs:7`), `Default` (`src/core/scroll.rs:7`), `Eq` (`src/core/scroll.rs:7`), `PartialEq` (`src/core/scroll.rs:7`), `StructuralPartialEq` (`src/core/scroll.rs:7`) |
| `core::text::TextBuffer` | `Clone` (`src/core/text.rs:32`), `Debug` (`src/core/text.rs:32`), `Default` (`src/core/text.rs:32`), `Eq` (`src/core/text.rs:32`), `PartialEq` (`src/core/text.rs:32`), `StructuralPartialEq` (`src/core/text.rs:32`) |
| `core::text::CursorPos` | `Clone` (`src/core/text.rs:42`), `Copy` (`src/core/text.rs:42`), `Debug` (`src/core/text.rs:42`), `Eq` (`src/core/text.rs:42`), `PartialEq` (`src/core/text.rs:42`), `StructuralPartialEq` (`src/core/text.rs:42`) |
| `theme::ColorLevel` | `Clone` (`src/theme.rs:21`), `Copy` (`src/theme.rs:21`), `Debug` (`src/theme.rs:21`), `Eq` (`src/theme.rs:21`), `PartialEq` (`src/theme.rs:21`), `StructuralPartialEq` (`src/theme.rs:21`) |
| `theme::Theme` | `Clone` (`src/theme.rs:98`), `Copy` (`src/theme.rs:98`), `Debug` (`src/theme.rs:98`), `Eq` (`src/theme.rs:98`), `PartialEq` (`src/theme.rs:98`), `StructuralPartialEq` (`src/theme.rs:98`) |
| `theme::Tone` | `Clone` (`src/theme.rs:558`), `Copy` (`src/theme.rs:558`), `Debug` (`src/theme.rs:558`), `Default` (`src/theme.rs:558`), `Eq` (`src/theme.rs:558`), `PartialEq` (`src/theme.rs:558`), `StructuralPartialEq` (`src/theme.rs:558`) |
| `theme::SyntaxTone` | `Clone` (`src/theme.rs:571`), `Copy` (`src/theme.rs:571`), `Debug` (`src/theme.rs:571`), `Eq` (`src/theme.rs:571`), `PartialEq` (`src/theme.rs:571`), `StructuralPartialEq` (`src/theme.rs:571`) |
| `theme::ButtonKind` | `Clone` (`src/theme.rs:583`), `Copy` (`src/theme.rs:583`), `Debug` (`src/theme.rs:583`), `Eq` (`src/theme.rs:583`), `PartialEq` (`src/theme.rs:583`), `StructuralPartialEq` (`src/theme.rs:583`) |
| `theme::BadgeKind` | `Clone` (`src/theme.rs:592`), `Copy` (`src/theme.rs:592`), `Debug` (`src/theme.rs:592`), `Eq` (`src/theme.rs:592`), `PartialEq` (`src/theme.rs:592`), `StructuralPartialEq` (`src/theme.rs:592`) |
| `ui::ctx::Interaction` | `Clone` (`src/ui/ctx.rs:16`), `Copy` (`src/ui/ctx.rs:16`), `Debug` (`src/ui/ctx.rs:16`), `Default` (`src/ui/ctx.rs:16`) |
| `ui::ctx::VisualState` | `Clone` (`src/ui/ctx.rs:44`), `Copy` (`src/ui/ctx.rs:44`), `Debug` (`src/ui/ctx.rs:44`), `Default` (`src/ui/ctx.rs:44`), `Eq` (`src/ui/ctx.rs:44`), `PartialEq` (`src/ui/ctx.rs:44`), `StructuralPartialEq` (`src/ui/ctx.rs:44`) |
| `ui::layout::SplitDir` | `Clone` (`src/ui/layout.rs:5`), `Copy` (`src/ui/layout.rs:5`), `Debug` (`src/ui/layout.rs:5`), `Eq` (`src/ui/layout.rs:5`), `PartialEq` (`src/ui/layout.rs:5`), `StructuralPartialEq` (`src/ui/layout.rs:5`) |
| `ui::layout::Maximized` | `Clone` (`src/ui/layout.rs:13`), `Copy` (`src/ui/layout.rs:13`), `Debug` (`src/ui/layout.rs:13`), `Eq` (`src/ui/layout.rs:13`), `PartialEq` (`src/ui/layout.rs:13`), `StructuralPartialEq` (`src/ui/layout.rs:13`) |
| `ui::layout::Split` | `Clone` (`src/ui/layout.rs:22`), `Copy` (`src/ui/layout.rs:22`), `Debug` (`src/ui/layout.rs:22`), `Eq` (`src/ui/layout.rs:22`), `PartialEq` (`src/ui/layout.rs:22`), `StructuralPartialEq` (`src/ui/layout.rs:22`) |
| `ui::popup::Placement` | `Clone` (`src/ui/popup.rs:15`), `Copy` (`src/ui/popup.rs:15`), `Debug` (`src/ui/popup.rs:15`), `Eq` (`src/ui/popup.rs:15`), `PartialEq` (`src/ui/popup.rs:15`), `StructuralPartialEq` (`src/ui/popup.rs:15`) |
| `widgets::brand::Lockup` | `Clone` (`src/widgets/brand.rs:14`), `Debug` (`src/widgets/brand.rs:14`), `Eq` (`src/widgets/brand.rs:14`), `PartialEq` (`src/widgets/brand.rs:14`), `StructuralPartialEq` (`src/widgets/brand.rs:14`) |
| `widgets::button::Button` | `Clone` (`src/widgets/button.rs:14`), `Debug` (`src/widgets/button.rs:14`) |
| `widgets::chips::Chip` | `Clone` (`src/widgets/chips.rs:15`), `Debug` (`src/widgets/chips.rs:15`), `Eq` (`src/widgets/chips.rs:15`), `PartialEq` (`src/widgets/chips.rs:15`), `StructuralPartialEq` (`src/widgets/chips.rs:15`) |
| `widgets::chips::ChipBar` | `Clone` (`src/widgets/chips.rs:34`), `Debug` (`src/widgets/chips.rs:34`) |
| `widgets::chips::ChipEvent` | `Clone` (`src/widgets/chips.rs:45`), `Copy` (`src/widgets/chips.rs:45`), `Debug` (`src/widgets/chips.rs:45`), `Eq` (`src/widgets/chips.rs:45`), `PartialEq` (`src/widgets/chips.rs:45`), `StructuralPartialEq` (`src/widgets/chips.rs:45`) |
| `widgets::choice::Checkbox` | `Clone` (`src/widgets/choice.rs:30`), `Debug` (`src/widgets/choice.rs:30`) |
| `widgets::choice::RadioGroup` | `Clone` (`src/widgets/choice.rs:117`), `Debug` (`src/widgets/choice.rs:117`) |
| `widgets::choice::Toggle` | `Clone` (`src/widgets/choice.rs:256`), `Debug` (`src/widgets/choice.rs:256`) |
| `widgets::code::Severity` | `Clone` (`src/widgets/code.rs:29`), `Copy` (`src/widgets/code.rs:29`), `Debug` (`src/widgets/code.rs:29`), `Eq` (`src/widgets/code.rs:29`), `PartialEq` (`src/widgets/code.rs:29`), `StructuralPartialEq` (`src/widgets/code.rs:29`) |
| `widgets::code::Diagnostic` | `Clone` (`src/widgets/code.rs:35`), `Debug` (`src/widgets/code.rs:35`), `Eq` (`src/widgets/code.rs:35`), `PartialEq` (`src/widgets/code.rs:35`), `StructuralPartialEq` (`src/widgets/code.rs:35`) |
| `widgets::code::FindState` | `Clone` (`src/widgets/code.rs:42`), `Debug` (`src/widgets/code.rs:42`), `Eq` (`src/widgets/code.rs:42`), `PartialEq` (`src/widgets/code.rs:42`), `StructuralPartialEq` (`src/widgets/code.rs:42`) |
| `widgets::code::EditorEvent` | `Clone` (`src/widgets/code.rs:52`), `Copy` (`src/widgets/code.rs:52`), `Debug` (`src/widgets/code.rs:52`), `Eq` (`src/widgets/code.rs:52`), `PartialEq` (`src/widgets/code.rs:52`), `StructuralPartialEq` (`src/widgets/code.rs:52`) |
| `widgets::code::CodeEditor` | `Clone` (`src/widgets/code.rs:64`), `Debug` (`src/widgets/code.rs:64`) |
| `widgets::completion::CompletionItem` | `Clone` (`src/widgets/completion.rs:18`), `Debug` (`src/widgets/completion.rs:18`), `Eq` (`src/widgets/completion.rs:18`), `PartialEq` (`src/widgets/completion.rs:18`), `StructuralPartialEq` (`src/widgets/completion.rs:18`) |
| `widgets::completion::Completion` | `Clone` (`src/widgets/completion.rs:29`), `Debug` (`src/widgets/completion.rs:29`) |
| `widgets::completion::CompletionEvent` | `Clone` (`src/widgets/completion.rs:42`), `Copy` (`src/widgets/completion.rs:42`), `Debug` (`src/widgets/completion.rs:42`), `Eq` (`src/widgets/completion.rs:42`), `PartialEq` (`src/widgets/completion.rs:42`), `StructuralPartialEq` (`src/widgets/completion.rs:42`) |
| `widgets::dialog::DialogBody` | `Clone` (`src/widgets/dialog.rs:17`), `Debug` (`src/widgets/dialog.rs:17`) |
| `widgets::dialog::AckInput` | `Clone` (`src/widgets/dialog.rs:30`), `Debug` (`src/widgets/dialog.rs:30`) |
| `widgets::dialog::DialogResult` | `Clone` (`src/widgets/dialog.rs:36`), `Copy` (`src/widgets/dialog.rs:36`), `Debug` (`src/widgets/dialog.rs:36`), `Eq` (`src/widgets/dialog.rs:36`), `PartialEq` (`src/widgets/dialog.rs:36`), `StructuralPartialEq` (`src/widgets/dialog.rs:36`) |
| `widgets::dialog::Dialog` | `Clone` (`src/widgets/dialog.rs:43`), `Debug` (`src/widgets/dialog.rs:43`) |
| `widgets::diff::DiffLineKind` | `Clone` (`src/widgets/diff.rs:19`), `Copy` (`src/widgets/diff.rs:19`), `Debug` (`src/widgets/diff.rs:19`), `Eq` (`src/widgets/diff.rs:19`), `PartialEq` (`src/widgets/diff.rs:19`), `StructuralPartialEq` (`src/widgets/diff.rs:19`) |
| `widgets::diff::DiffLine` | `Clone` (`src/widgets/diff.rs:26`), `Debug` (`src/widgets/diff.rs:26`), `Eq` (`src/widgets/diff.rs:26`), `PartialEq` (`src/widgets/diff.rs:26`), `StructuralPartialEq` (`src/widgets/diff.rs:26`) |
| `widgets::diff::DiffHunk` | `Clone` (`src/widgets/diff.rs:53`), `Debug` (`src/widgets/diff.rs:53`), `Eq` (`src/widgets/diff.rs:53`), `PartialEq` (`src/widgets/diff.rs:53`), `StructuralPartialEq` (`src/widgets/diff.rs:53`) |
| `widgets::diff::DiffStatus` | `Clone` (`src/widgets/diff.rs:86`), `Debug` (`src/widgets/diff.rs:86`), `Eq` (`src/widgets/diff.rs:86`), `PartialEq` (`src/widgets/diff.rs:86`), `StructuralPartialEq` (`src/widgets/diff.rs:86`) |
| `widgets::diff::DiffFile` | `Clone` (`src/widgets/diff.rs:124`), `Debug` (`src/widgets/diff.rs:124`), `Eq` (`src/widgets/diff.rs:124`), `PartialEq` (`src/widgets/diff.rs:124`), `StructuralPartialEq` (`src/widgets/diff.rs:124`) |
| `widgets::diff::DiffMode` | `Clone` (`src/widgets/diff.rs:168`), `Copy` (`src/widgets/diff.rs:168`), `Debug` (`src/widgets/diff.rs:168`), `Default` (`src/widgets/diff.rs:168`), `Eq` (`src/widgets/diff.rs:168`), `PartialEq` (`src/widgets/diff.rs:168`), `StructuralPartialEq` (`src/widgets/diff.rs:168`) |
| `widgets::empty::EmptyKind` | `Clone` (`src/widgets/empty.rs:9`), `Copy` (`src/widgets/empty.rs:9`), `Debug` (`src/widgets/empty.rs:9`), `Default` (`src/widgets/empty.rs:9`), `Eq` (`src/widgets/empty.rs:9`), `PartialEq` (`src/widgets/empty.rs:9`), `StructuralPartialEq` (`src/widgets/empty.rs:9`) |
| `widgets::empty::EmptyState` | `Clone` (`src/widgets/empty.rs:17`), `Debug` (`src/widgets/empty.rs:17`), `Default` (`src/widgets/empty.rs:17`), `Eq` (`src/widgets/empty.rs:17`), `PartialEq` (`src/widgets/empty.rs:17`), `StructuralPartialEq` (`src/widgets/empty.rs:17`) |
| `widgets::grid::CellValue` | `Clone` (`src/widgets/grid.rs:31`), `Debug` (`src/widgets/grid.rs:31`), `PartialEq` (`src/widgets/grid.rs:31`), `StructuralPartialEq` (`src/widgets/grid.rs:31`) |
| `widgets::grid::CellKind` | `Clone` (`src/widgets/grid.rs:64`), `Copy` (`src/widgets/grid.rs:64`), `Debug` (`src/widgets/grid.rs:64`), `Eq` (`src/widgets/grid.rs:64`), `PartialEq` (`src/widgets/grid.rs:64`), `StructuralPartialEq` (`src/widgets/grid.rs:64`) |
| `widgets::grid::ColumnSpec` | `Clone` (`src/widgets/grid.rs:92`), `Debug` (`src/widgets/grid.rs:92`), `Eq` (`src/widgets/grid.rs:92`), `PartialEq` (`src/widgets/grid.rs:92`), `StructuralPartialEq` (`src/widgets/grid.rs:92`) |
| `widgets::grid::RowTotal` | `Clone` (`src/widgets/grid.rs:151`), `Copy` (`src/widgets/grid.rs:151`), `Debug` (`src/widgets/grid.rs:151`), `Eq` (`src/widgets/grid.rs:151`), `PartialEq` (`src/widgets/grid.rs:151`), `StructuralPartialEq` (`src/widgets/grid.rs:151`) |
| `widgets::grid::GridRows` | `Clone` (`src/widgets/grid.rs:158`), `Debug` (`src/widgets/grid.rs:158`), `PartialEq` (`src/widgets/grid.rs:158`), `StructuralPartialEq` (`src/widgets/grid.rs:158`) |
| `widgets::grid::UndoAction` | `Clone` (`src/widgets/grid.rs:168`), `Debug` (`src/widgets/grid.rs:168`), `PartialEq` (`src/widgets/grid.rs:168`), `StructuralPartialEq` (`src/widgets/grid.rs:168`) |
| `widgets::grid::PendingChanges` | `Clone` (`src/widgets/grid.rs:186`), `Debug` (`src/widgets/grid.rs:186`), `Default` (`src/widgets/grid.rs:186`), `PartialEq` (`src/widgets/grid.rs:186`), `StructuralPartialEq` (`src/widgets/grid.rs:186`) |
| `widgets::grid::RowState` | `Clone` (`src/widgets/grid.rs:224`), `Copy` (`src/widgets/grid.rs:224`), `Debug` (`src/widgets/grid.rs:224`), `Eq` (`src/widgets/grid.rs:224`), `PartialEq` (`src/widgets/grid.rs:224`), `StructuralPartialEq` (`src/widgets/grid.rs:224`) |
| `widgets::grid::EditState` | `Clone` (`src/widgets/grid.rs:233`), `Debug` (`src/widgets/grid.rs:233`) |
| `widgets::grid::GridEvent` | `Clone` (`src/widgets/grid.rs:241`), `Debug` (`src/widgets/grid.rs:241`), `PartialEq` (`src/widgets/grid.rs:241`), `StructuralPartialEq` (`src/widgets/grid.rs:241`) |
| `widgets::grid::DataGrid` | `Clone` (`src/widgets/grid.rs:326`), `Debug` (`src/widgets/grid.rs:326`) |
| `widgets::hintbar::HintLayer` | `Clone` (`src/widgets/hintbar.rs:13`), `Debug` (`src/widgets/hintbar.rs:13`), `Default` (`src/widgets/hintbar.rs:13`), `Eq` (`src/widgets/hintbar.rs:13`), `PartialEq` (`src/widgets/hintbar.rs:13`), `StructuralPartialEq` (`src/widgets/hintbar.rs:13`) |
| `widgets::input::TextInput` | `Clone` (`src/widgets/input.rs:18`), `Debug` (`src/widgets/input.rs:18`) |
| `widgets::input::InputEvent` | `Clone` (`src/widgets/input.rs:47`), `Copy` (`src/widgets/input.rs:47`), `Debug` (`src/widgets/input.rs:47`), `Eq` (`src/widgets/input.rs:47`), `PartialEq` (`src/widgets/input.rs:47`), `StructuralPartialEq` (`src/widgets/input.rs:47`) |
| `widgets::keyhint::Hint` | `Clone` (`src/widgets/keyhint.rs:9`), `Copy` (`src/widgets/keyhint.rs:9`), `Debug` (`src/widgets/keyhint.rs:9`), `Eq` (`src/widgets/keyhint.rs:9`), `PartialEq` (`src/widgets/keyhint.rs:9`), `StructuralPartialEq` (`src/widgets/keyhint.rs:9`) |
| `widgets::list::ListItem` | `Clone` (`src/widgets/list.rs:12`), `Debug` (`src/widgets/list.rs:12`) |
| `widgets::list::SelectMode` | `Clone` (`src/widgets/list.rs:37`), `Copy` (`src/widgets/list.rs:37`), `Debug` (`src/widgets/list.rs:37`), `Eq` (`src/widgets/list.rs:37`), `PartialEq` (`src/widgets/list.rs:37`), `StructuralPartialEq` (`src/widgets/list.rs:37`) |
| `widgets::list::ListBox` | `Clone` (`src/widgets/list.rs:45`), `Debug` (`src/widgets/list.rs:45`) |
| `widgets::menu::MenuItem` | `Clone` (`src/widgets/menu.rs:18`), `Debug` (`src/widgets/menu.rs:18`), `Eq` (`src/widgets/menu.rs:18`), `PartialEq` (`src/widgets/menu.rs:18`), `StructuralPartialEq` (`src/widgets/menu.rs:18`) |
| `widgets::menu::Placement` | `Clone` (`src/widgets/menu.rs:55`), `Copy` (`src/widgets/menu.rs:55`), `Debug` (`src/widgets/menu.rs:55`), `Eq` (`src/widgets/menu.rs:55`), `PartialEq` (`src/widgets/menu.rs:55`), `StructuralPartialEq` (`src/widgets/menu.rs:55`) |
| `widgets::menu::MenuEvent` | `Clone` (`src/widgets/menu.rs:65`), `Copy` (`src/widgets/menu.rs:65`), `Debug` (`src/widgets/menu.rs:65`), `Eq` (`src/widgets/menu.rs:65`), `PartialEq` (`src/widgets/menu.rs:65`), `StructuralPartialEq` (`src/widgets/menu.rs:65`) |
| `widgets::menu::ContextMenu` | `Clone` (`src/widgets/menu.rs:71`), `Debug` (`src/widgets/menu.rs:71`) |
| `widgets::menu::MenuBarEvent` | `Clone` (`src/widgets/menu.rs:353`), `Copy` (`src/widgets/menu.rs:353`), `Debug` (`src/widgets/menu.rs:353`), `Eq` (`src/widgets/menu.rs:353`), `PartialEq` (`src/widgets/menu.rs:353`), `StructuralPartialEq` (`src/widgets/menu.rs:353`) |
| `widgets::menu::MenuBar` | `Clone` (`src/widgets/menu.rs:364`), `Debug` (`src/widgets/menu.rs:364`) |
| `widgets::panel::PanelKind` | `Clone` (`src/widgets/panel.rs:22`), `Copy` (`src/widgets/panel.rs:22`), `Debug` (`src/widgets/panel.rs:22`), `Eq` (`src/widgets/panel.rs:22`), `PartialEq` (`src/widgets/panel.rs:22`), `StructuralPartialEq` (`src/widgets/panel.rs:22`) |
| `widgets::panel::ScrollPanel` | `Clone` (`src/widgets/panel.rs:176`), `Debug` (`src/widgets/panel.rs:176`) |
| `widgets::picker::PickerItem` | `Clone` (`src/widgets/picker.rs:18`), `Debug` (`src/widgets/picker.rs:18`), `Eq` (`src/widgets/picker.rs:18`), `PartialEq` (`src/widgets/picker.rs:18`), `StructuralPartialEq` (`src/widgets/picker.rs:18`) |
| `widgets::picker::PickerStatus` | `Clone` (`src/widgets/picker.rs:30`), `Debug` (`src/widgets/picker.rs:30`), `Default` (`src/widgets/picker.rs:30`), `Eq` (`src/widgets/picker.rs:30`), `PartialEq` (`src/widgets/picker.rs:30`), `StructuralPartialEq` (`src/widgets/picker.rs:30`) |
| `widgets::picker::Picker` | `Clone` (`src/widgets/picker.rs:43`), `Debug` (`src/widgets/picker.rs:43`) |
| `widgets::picker::PickerEvent` | `Clone` (`src/widgets/picker.rs:67`), `Debug` (`src/widgets/picker.rs:67`), `Eq` (`src/widgets/picker.rs:67`), `PartialEq` (`src/widgets/picker.rs:67`), `StructuralPartialEq` (`src/widgets/picker.rs:67`) |
| `widgets::progress::ProgressStatus` | `Clone` (`src/widgets/progress.rs:14`), `Copy` (`src/widgets/progress.rs:14`), `Debug` (`src/widgets/progress.rs:14`), `Eq` (`src/widgets/progress.rs:14`), `PartialEq` (`src/widgets/progress.rs:14`), `StructuralPartialEq` (`src/widgets/progress.rs:14`) |
| `widgets::progress::MeterLevel` | `Clone` (`src/widgets/progress.rs:89`), `Copy` (`src/widgets/progress.rs:89`), `Debug` (`src/widgets/progress.rs:89`), `Eq` (`src/widgets/progress.rs:89`), `PartialEq` (`src/widgets/progress.rs:89`), `StructuralPartialEq` (`src/widgets/progress.rs:89`) |
| `widgets::progress::MeterVisual` | `Clone` (`src/widgets/progress.rs:109`), `Copy` (`src/widgets/progress.rs:109`), `Debug` (`src/widgets/progress.rs:109`), `Default` (`src/widgets/progress.rs:109`), `Eq` (`src/widgets/progress.rs:109`), `PartialEq` (`src/widgets/progress.rs:109`), `StructuralPartialEq` (`src/widgets/progress.rs:109`) |
| `widgets::progress::MeterTone` | `Clone` (`src/widgets/progress.rs:122`), `Copy` (`src/widgets/progress.rs:122`), `Debug` (`src/widgets/progress.rs:122`), `Default` (`src/widgets/progress.rs:122`), `Eq` (`src/widgets/progress.rs:122`), `PartialEq` (`src/widgets/progress.rs:122`), `StructuralPartialEq` (`src/widgets/progress.rs:122`) |
| `widgets::progress::Meter` | `Clone` (`src/widgets/progress.rs:163`), `Debug` (`src/widgets/progress.rs:163`) |
| `widgets::props::Prop` | `Clone` (`src/widgets/props.rs:16`), `Debug` (`src/widgets/props.rs:16`), `Eq` (`src/widgets/props.rs:16`), `PartialEq` (`src/widgets/props.rs:16`), `StructuralPartialEq` (`src/widgets/props.rs:16`) |
| `widgets::props::PropsEvent` | `Clone` (`src/widgets/props.rs:91`), `Debug` (`src/widgets/props.rs:91`), `Eq` (`src/widgets/props.rs:91`), `PartialEq` (`src/widgets/props.rs:91`), `StructuralPartialEq` (`src/widgets/props.rs:91`) |
| `widgets::props::PropsList` | `Clone` (`src/widgets/props.rs:99`), `Debug` (`src/widgets/props.rs:99`) |
| `widgets::segments::Segment` | `Clone` (`src/widgets/segments.rs:14`), `Debug` (`src/widgets/segments.rs:14`), `Eq` (`src/widgets/segments.rs:14`), `PartialEq` (`src/widgets/segments.rs:14`), `StructuralPartialEq` (`src/widgets/segments.rs:14`) |
| `widgets::select::Select` | `Clone` (`src/widgets/select.rs:14`), `Debug` (`src/widgets/select.rs:14`) |
| `widgets::select::SelectEvent` | `Clone` (`src/widgets/select.rs:27`), `Copy` (`src/widgets/select.rs:27`), `Debug` (`src/widgets/select.rs:27`), `Eq` (`src/widgets/select.rs:27`), `PartialEq` (`src/widgets/select.rs:27`), `StructuralPartialEq` (`src/widgets/select.rs:27`) |
| `widgets::splitter::Splitter` | `Clone` (`src/widgets/splitter.rs:15`), `Copy` (`src/widgets/splitter.rs:15`), `Debug` (`src/widgets/splitter.rs:15`), `Eq` (`src/widgets/splitter.rs:15`), `PartialEq` (`src/widgets/splitter.rs:15`), `StructuralPartialEq` (`src/widgets/splitter.rs:15`) |
| `widgets::statusbar::Emphasis` | `Clone` (`src/widgets/statusbar.rs:18`), `Copy` (`src/widgets/statusbar.rs:18`), `Debug` (`src/widgets/statusbar.rs:18`), `Default` (`src/widgets/statusbar.rs:18`), `Eq` (`src/widgets/statusbar.rs:18`), `PartialEq` (`src/widgets/statusbar.rs:18`), `StructuralPartialEq` (`src/widgets/statusbar.rs:18`) |
| `widgets::statusbar::StatusItem` | `Clone` (`src/widgets/statusbar.rs:29`), `Debug` (`src/widgets/statusbar.rs:29`), `Eq` (`src/widgets/statusbar.rs:29`), `PartialEq` (`src/widgets/statusbar.rs:29`), `StructuralPartialEq` (`src/widgets/statusbar.rs:29`) |
| `widgets::statusbar::Group` | `Clone` (`src/widgets/statusbar.rs:102`), `Copy` (`src/widgets/statusbar.rs:102`), `Debug` (`src/widgets/statusbar.rs:102`), `Eq` (`src/widgets/statusbar.rs:102`), `PartialEq` (`src/widgets/statusbar.rs:102`), `StructuralPartialEq` (`src/widgets/statusbar.rs:102`) |
| `widgets::statusbar::Placed` | `Clone` (`src/widgets/statusbar.rs:110`), `Debug` (`src/widgets/statusbar.rs:110`), `Eq` (`src/widgets/statusbar.rs:110`), `PartialEq` (`src/widgets/statusbar.rs:110`), `StructuralPartialEq` (`src/widgets/statusbar.rs:110`) |
| `widgets::statusbar::StatusBar` | `Clone` (`src/widgets/statusbar.rs:120`), `Debug` (`src/widgets/statusbar.rs:120`), `Default` (`src/widgets/statusbar.rs:120`) |
| `widgets::steps::StepState` | `Clone` (`src/widgets/steps.rs:17`), `Copy` (`src/widgets/steps.rs:17`), `Debug` (`src/widgets/steps.rs:17`), `Default` (`src/widgets/steps.rs:17`), `Eq` (`src/widgets/steps.rs:17`), `Hash` (`src/widgets/steps.rs:17`), `PartialEq` (`src/widgets/steps.rs:17`), `StructuralPartialEq` (`src/widgets/steps.rs:17`) |
| `widgets::steps::Step` | `Clone` (`src/widgets/steps.rs:48`), `Debug` (`src/widgets/steps.rs:48`), `Eq` (`src/widgets/steps.rs:48`), `PartialEq` (`src/widgets/steps.rs:48`), `StructuralPartialEq` (`src/widgets/steps.rs:48`) |
| `widgets::steps::StepRail` | `Clone` (`src/widgets/steps.rs:66`), `Debug` (`src/widgets/steps.rs:66`) |
| `widgets::table::SortDir` | `Clone` (`src/widgets/table.rs:14`), `Copy` (`src/widgets/table.rs:14`), `Debug` (`src/widgets/table.rs:14`), `Eq` (`src/widgets/table.rs:14`), `PartialEq` (`src/widgets/table.rs:14`), `StructuralPartialEq` (`src/widgets/table.rs:14`) |
| `widgets::table::Align` | `Clone` (`src/widgets/table.rs:20`), `Copy` (`src/widgets/table.rs:20`), `Debug` (`src/widgets/table.rs:20`), `Eq` (`src/widgets/table.rs:20`), `PartialEq` (`src/widgets/table.rs:20`), `StructuralPartialEq` (`src/widgets/table.rs:20`) |
| `widgets::table::Column` | `Clone` (`src/widgets/table.rs:26`), `Debug` (`src/widgets/table.rs:26`) |
| `widgets::table::Cell` | `Clone` (`src/widgets/table.rs:62`), `Debug` (`src/widgets/table.rs:62`), `Eq` (`src/widgets/table.rs:62`), `PartialEq` (`src/widgets/table.rs:62`), `StructuralPartialEq` (`src/widgets/table.rs:62`) |
| `widgets::table::EditState` | `Clone` (`src/widgets/table.rs:85`), `Debug` (`src/widgets/table.rs:85`) |
| `widgets::table::DataTable` | `Clone` (`src/widgets/table.rs:96`), `Debug` (`src/widgets/table.rs:96`) |
| `widgets::table::TableEvent` | `Clone` (`src/widgets/table.rs:122`), `Copy` (`src/widgets/table.rs:122`), `Debug` (`src/widgets/table.rs:122`), `Eq` (`src/widgets/table.rs:122`), `PartialEq` (`src/widgets/table.rs:122`), `StructuralPartialEq` (`src/widgets/table.rs:122`) |
| `widgets::tabs::TabItem` | `Clone` (`src/widgets/tabs.rs:18`), `Debug` (`src/widgets/tabs.rs:18`), `Default` (`src/widgets/tabs.rs:18`), `Eq` (`src/widgets/tabs.rs:18`), `PartialEq` (`src/widgets/tabs.rs:18`), `StructuralPartialEq` (`src/widgets/tabs.rs:18`) |
| `widgets::tabs::Tabs` | `Clone` (`src/widgets/tabs.rs:52`), `Debug` (`src/widgets/tabs.rs:52`) |
| `widgets::tabs::TabEvent` | `Clone` (`src/widgets/tabs.rs:70`), `Copy` (`src/widgets/tabs.rs:70`), `Debug` (`src/widgets/tabs.rs:70`), `Eq` (`src/widgets/tabs.rs:70`), `PartialEq` (`src/widgets/tabs.rs:70`), `StructuralPartialEq` (`src/widgets/tabs.rs:70`) |
| `widgets::textarea::TextArea` | `Clone` (`src/widgets/textarea.rs:21`), `Debug` (`src/widgets/textarea.rs:21`) |
| `widgets::tree::TreeNode` | `Clone` (`src/widgets/tree.rs:13`), `Debug` (`src/widgets/tree.rs:13`) |
| `widgets::tree::FlatRow` | `Clone` (`src/widgets/tree.rs:85`), `Debug` (`src/widgets/tree.rs:85`) |
| `widgets::tree::TreeEvent` | `Clone` (`src/widgets/tree.rs:98`), `Debug` (`src/widgets/tree.rs:98`), `Eq` (`src/widgets/tree.rs:98`), `PartialEq` (`src/widgets/tree.rs:98`), `StructuralPartialEq` (`src/widgets/tree.rs:98`) |
| `widgets::tree::TreeView` | `Clone` (`src/widgets/tree.rs:106`), `Debug` (`src/widgets/tree.rs:106`) |
| `widgets::viewport::Span` | `Clone` (`src/widgets/viewport.rs:22`), `Debug` (`src/widgets/viewport.rs:22`), `Eq` (`src/widgets/viewport.rs:22`), `PartialEq` (`src/widgets/viewport.rs:22`), `StructuralPartialEq` (`src/widgets/viewport.rs:22`) |
| `widgets::viewport::CellPos` | `Clone` (`src/widgets/viewport.rs:75`), `Copy` (`src/widgets/viewport.rs:75`), `Debug` (`src/widgets/viewport.rs:75`), `Eq` (`src/widgets/viewport.rs:75`), `Ord` (`src/widgets/viewport.rs:75`), `PartialEq` (`src/widgets/viewport.rs:75`), `PartialOrd` (`src/widgets/viewport.rs:75`), `StructuralPartialEq` (`src/widgets/viewport.rs:75`) |
| `widgets::viewport::ViewportEvent` | `Clone` (`src/widgets/viewport.rs:81`), `Debug` (`src/widgets/viewport.rs:81`), `Eq` (`src/widgets/viewport.rs:81`), `PartialEq` (`src/widgets/viewport.rs:81`), `StructuralPartialEq` (`src/widgets/viewport.rs:81`) |
| `widgets::viewport::TextViewport` | `Clone` (`src/widgets/viewport.rs:108`), `Debug` (`src/widgets/viewport.rs:108`) |
