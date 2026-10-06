# Consumer recipes and public API examples

This file is the canonical owner of consumer examples.
It shows ordinary use of reusable Termrock components.
Read it with the [public API contract](public-api.md), the
[component-only contract](../architecture/component-composition.md), and the
[component contracts](../components/README.md).

## Read the status before using an example

This file describes the desired consumer experience. It does not change source code.
The source-checked examples use declarations at commit `4d117b48092c5210c80070ac9bbeb6560973b2b0`.
They were read at that commit but were not compiled or executed for this task.
The other positive examples are proposed API contracts, not existing capabilities.
No example in this file has `current_compiled` status yet.

Keep useful current APIs where they satisfy the requirement.
Do not rename methods merely to match illustrative spelling below.
Before implementation, reconcile one signature per component in its canonical contract.
Do not retain incompatible current and target contracts as equally authoritative.

| Status | Meaning |
| --- | --- |
| `current_source_checked` | The named declarations were read. Compilation and behavior are not certified. |
| `current_compiled` | The exact snippet has a successful recorded external-consumer compile result. No example has this status yet. |
| `proposed_target` | The ownership and usage are the target. Named API gaps must be resolved before compilation. |
| `forbidden_example` | This is a pattern the reviewer must reject. It is not a recommended API. |

All examples are fragments unless stated otherwise.
Types named `model`, `view`, `ids`, `areas`, or `layers` represent app-owned values.
`cx` and `ui` come from the runtime. A preview must not manufacture their registries.
An example may omit unrelated fields. It must not hide the behavior under review.
The companion [example catalog](example-catalog.json) names the gaps and required checks.

## Shared helper rules

`rows` and `columns` denote a proposed bounded layout API with a fixed number of outputs.
Their input is an area, tracks, and a gap. Their output is allocated rectangles.
They do not paint. Their arithmetic must saturate when space is exhausted.
A root area or layout choice is permitted app configuration.
A table of exported coordinates for known snapshots is prohibited.

A helper ending in `apply_*` or `request_*` below means a typed app reducer or effect request.
Its contract excludes low-level drawing and generic widget input handling.
An implementation must show its body or link its audited source.
Do not use an undefined helper to hide a second UI implementation.

Every composed screen needs update, draw, and applicable measure evidence.
Draw-only fragments illustrate composition. They do not establish complete integration.
Each recipe's acceptance requires the complete path and the listed negative cases.


## EX-01 — A button with one owner

**Status:** `current_source_checked`.

**Source declarations at `4d117b48`:** `crates/termrock-controls/src/button.rs:173` (`new`),
`:194` (`variant`), `:201` (`disabled`), `:280` (`update`), `:362` (`draw`);
`crates/termrock/src/lib.rs:106,132,157` (facade re-exports).

**Contract owners:** `termrock-controls`.

This fragment uses the inspected Button constructor and phase methods.
The caller owns the save operation. The Button owns activation and painting.

```rust
use termrock::{Activated, Button, Cx, Id, Rect, Response, Ui, Variant};

fn update_save(id: Id, cx: &mut Cx<'_>, can_save: bool)
    -> Response<Activated>
{
    Button::new(id, "Save")
        .variant(Variant::PRIMARY)
        .disabled(!can_save)
        .update(cx)
}

fn draw_save(id: Id, ui: &mut Ui<'_>, area: Rect, can_save: bool) {
    Button::new(id, "Save")
        .variant(Variant::PRIMARY)
        .disabled(!can_save)
        .draw(ui, area);
}
```

The app handles the typed response in its update pass.
It does not draw a second label or a second focus marker.
Both passes use the same ID and eligibility value.
A passing constructor example does not certify baseline rendering.

**Required checks:** keyboard activation; completed pointer activation; disabled activation rejected; press then release outside.

## EX-02 — Current controlled text input

**Status:** `current_source_checked`.

**Source declarations at `4d117b48`:** `crates/termrock-fields/src/input.rs:103-108`
(`TextAction`), `:980` (`new`), `:1000` (`value`), `:1056` (`read_only`),
`:1137-1150` (`update`), `:1318-1325` (`draw`);
`crates/termrock/src/lib.rs:165-166` (facade re-exports).

**Contract owners:** `termrock-fields`.

This fragment uses the current `TextInput::new(id)` API.
It does not use the older proposed three-argument constructor.

```rust
use termrock::{Cx, Id, Rect, Response, TextAction, TextInput,
               TextInputState, Ui};

fn update_name(
    id: Id,
    cx: &mut Cx<'_>,
    state: &mut TextInputState,
    value: &mut String,
    locked: bool,
) -> Response<TextAction> {
    TextInput::new(id)
        .read_only(locked)
        .update(cx, state, value)
}

fn draw_name(
    id: Id,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &TextInputState,
    value: &str,
    locked: bool,
) {
    TextInput::new(id)
        .read_only(locked)
        .value(value)
        .draw(ui, area, state);
}
```

The inspected action is `TextAction::Committed`, not a `Commit { value }` payload.
The app must not manually paint the draft, caret, mask, or validation glyph.
The future signature decision must retain one coherent ownership model.
Check the original focus and completed-click policies before declaring behavior accepted.

**Required checks:** commit; cancel; read-only input; focus transfer; grapheme-safe pointer placement.

## EX-03 — Controlled choices

**Status:** `proposed_target`.

**Contract owners:** `termrock-controls`, `termrock-navigation`, `termrock-overlays`.

The following target uses typed requested values.
The app retains the committed value and applies the request.

```rust
let response = Checkbox::new(ids.read_only, "Mount read-only")
    .checked(model.read_only)
    .disabled(model.busy)
    .update(cx);

if let Some(ValueChange { value, .. }) = response.action() {
    model.read_only = *value;
}

// Draw pass, after update has ended.
Checkbox::new(ids.read_only, "Mount read-only")
    .checked(model.read_only)
    .disabled(model.busy)
    .draw(ui, areas.read_only);
```

`ValueChange` and the action accessor in this fragment are target contracts.
Map them to one accepted existing equivalent where it meets the contract.
Use the same pattern for Toggle, RadioGroup, Select, and ChipBar.
A RadioGroup cursor is not its committed choice.
The app must not turn a painted `[ ]` string into a substitute Checkbox.

**API gaps to resolve:** `ValueChange<T>`; `Response::action typed borrowed accessor`; `controlled-choice builder/update shape`.

**Required checks:** unchecked/checked; disabled; pointer cancellation; choice key retained after reorder.

## EX-04 — A form with validation and protected input

**Status:** `proposed_target`.

**Contract owners:** `termrock-forms`, `termrock-fields`.

A form describes fields and validates data. It does not draw field chrome.
This example uses a borrowed schema. The schema contains no raw paint callbacks.

```rust
let schema = FormSchema::new([
    FieldSpec::text(field_keys.name, "Name").required(),
    FieldSpec::secret(field_keys.token, "Token")
        .policy(SecretPolicy::masked_no_copy()),
]);

let response = Form::new(ids.connection)
    .schema(&schema)
    .update(cx, &mut view.form, &mut model.draft);

match response.action() {
    Some(FormAction::Submitted) => effects.request_preview(&model.draft),
    Some(FormAction::Cancelled) => model.cancel_edit(),
    _ => {}
}

// Draw pass. The same schema describes the same form.
Form::new(ids.connection)
    .schema(&schema)
    .draw(ui, areas.form, &view.form, &model.draft);
```

`effects.request_preview` emits a typed local preview action.
It does not contact a real provider in these preview apps.
`model.cancel_edit` restores the app's draft policy; it has no UI code.
The form owns field layout, error association, and traversal through shared components.
Validators return safe messages. They do not include secret input.
Document single-line and multiline commit/cancel differences separately.

**API gaps to resolve:** `FormSchema borrowed field schema`; `FieldSpec typed data binding`; `Form schema/update/draw contract`; `SecretPolicy::masked_no_copy`.

**Required checks:** invalid submit; first invalid field focus; cancel restores draft; masked input/copy rejection; nested form paste.

## EX-05 — List content without a custom row painter

**Status:** `proposed_target`.

**Contract owners:** `termrock-navigation`, `termrock-collections`, `termrock-controls`.

Describe row content as data. Let List apply selection, focus, hover, and clipping.

```rust
fn describe_workspace(row: &WorkspaceRow) -> RowContent<'_> {
    RowContent::new(row.label.as_str())
        .leading(row.icon)
        .detail(row.summary.as_str())
        .trailing(BadgeContent::new(row.status.label(), row.status.tone()))
}

let list = List::new(ids.workspaces).row_content(describe_workspace);
let response = list.update(cx, &mut view.workspaces, &model.rows);
if let Some(ListAction::Activated(key)) = response.action() {
    model.open_workspace(*key);
}

// Draw pass. `open_workspace` never paints.
List::new(ids.workspaces)
    .row_content(describe_workspace)
    .draw(ui, areas.workspaces, &view.workspaces, &model.rows);
```

`WorkspaceRow` and `open_workspace` are app data and reducer concepts.
`RowContent` and `BadgeContent` are proposed borrowed content descriptions.
They contain text, semantic roles, and content. They contain no screen coordinates.
List remains the only owner of row geometry and interaction.
Give each row a stable key. Do not derive identity from a label or row index.

**API gaps to resolve:** `RowContent`; `BadgeContent`; `List::row_content`; `keyed collection adapter`.

**Required checks:** long label; changed status; duplicate labels with distinct keys; scroll; remove pressed row.

## EX-06 — Tree and detail layout

**Status:** `proposed_target`.

**Contract owners:** `termrock-controls`, `termrock-navigation`.

The split contains two component compositions. It contains no fixed screenshot columns.

```rust
let response = SplitPane::new(ids.browser_split)
    .minimums(24, 20)
    .update(cx, &mut view.split);
view.apply_split_request(response);

// Draw pass. The layout helper saturates under small constraints.
let [tree_area, detail_area] = SplitPane::new(ids.browser_split)
    .minimums(24, 20)
    .areas(area, &view.split);

Tree::new(ids.files)
    .draw(ui, tree_area, &view.files, &model.file_tree);

Panel::new(ids.details)
    .title("Details")
    .draw(ui, detail_area, |ui, body| {
        PropsList::new(ids.properties)
            .draw(ui, body, &view.properties, &model.selected_properties());
    });
```

Tree update and typed selection run in the update pass too.
`selected_properties` returns borrowed property data; it cannot draw.
`apply_split_request` applies a typed preference, not pointer bookkeeping.
If the API updates split preference directly, remove that redundant helper.
Specify one accepted policy before implementation.
Test collapse, filtering, and removal with retained selection and focus.

**API gaps to resolve:** `SplitPane minimums/areas target`; `Panel composition callback shape`; `PropsList borrowed model draw contract`.

**Required checks:** split drag; maximize/restore; tree collapse; detail selection; resize below minima.

## EX-07 — Tabs and screen selection

**Status:** `proposed_target`.

**Contract owners:** `termrock-navigation`, `termrock-layout`.

A tab action changes app selection. It does not select a different rendering engine.

```rust
let response = Tabs::new(ids.editor_tabs)
    .selected(model.active_tab)
    .update(cx, &mut view.tabs, &model.tabs);

match response.action() {
    Some(TabsAction::Selected(key)) => model.active_tab = *key,
    Some(TabsAction::CloseRequested(key)) => model.request_close(*key),
    _ => {}
}

// Draw pass.
let [tab_area, body_area] = rows(area, [Track::Fixed(1), Track::Fill(1)], 0);
Tabs::new(ids.editor_tabs)
    .selected(model.active_tab)
    .draw(ui, tab_area, &view.tabs, &model.tabs);

match model.active_tab {
    TabKey::General => general_view.draw(ui, body_area, &model.general),
    TabKey::Mounts => mounts_view.draw(ui, body_area, &model.mounts),
}
```

Both child views compose Termrock components under this same contract.
A view helper may not contain low-level painting or scenario-specific screen data.
`rows` is a proposed fixed-array layout helper, not a new UI language.
It computes bounded rectangles from constraints and documented spacing.

**API gaps to resolve:** `controlled Tabs selected/update/draw shape`; `rows<const N>`; `Track::Fixed / Track::Fill`.

**Required checks:** tab overflow; close dirty tab; reorder tabs; narrow/wide restore.

## EX-08 — A dialog that keeps normal input behavior

**Status:** `proposed_target`.

**Contract owners:** `termrock-overlays`, `termrock-runtime`, `termrock-controls`.

Opening a dialog must open the runtime layer in every fixture and motion mode.
The layer owns dismissal, input priority, and focus restoration.

```rust
if requests.take_open_delete() {
    view.delete_target = model.selected_key();
    cx.open_layer(layers.delete, delete_dialog.layer_spec());
}

let response = delete_dialog.update(cx, &mut view.delete);
match response.action() {
    Some(DialogAction::Accepted) => model.request_delete(view.delete_target),
    Some(DialogAction::Cancelled) => view.delete_target = None,
    _ => {}
}

// Draw pass. The runtime does not run this body for a closed layer.
ui.layer(layers.delete, |ui, body| {
    delete_dialog.draw(ui, body, &view.delete, |ui, content| {
        Label::new("Delete the selected item?")
            .role(TextRole::Primary)
            .draw(ui, content);
    });
});
```

`Label` is a proposed primitive for content that has no existing suitable component.
Do not add it if an accepted existing text component satisfies the same contract.
The dialog config declares standard buttons and typed results.
The captured target is a stable key. It does not silently change after list reorder.
No paused-mode guard may bypass update, layer opening, or dismissal.

**API gaps to resolve:** `Dialog composition and typed result shape`; `Label / TextRole`; `LayerSpec accepted opening contract`.

**Required checks:** outside press/release; Escape; nested dialog; removed target; paused mode input.

## EX-09 — Menus, context menus, and a command palette

**Status:** `proposed_target`.

**Contract owners:** `termrock-overlays`.

The app supplies command data and handles results.
The menu and picker families share their library mechanisms.

```rust
let actions = model.available_commands();
let response = CommandPalette::new(ids.commands)
    .placeholder("Find a command")
    .update(cx, &mut view.commands, &actions);

if let Some(PaletteAction::Chosen(key)) = response.action() {
    model.dispatch(*key);
}

// Draw pass in the runtime-owned layer.
ui.layer(layers.commands, |ui, body| {
    CommandPalette::new(ids.commands)
        .placeholder("Find a command")
        .draw(ui, body, &view.commands, &actions);
});
```

`available_commands` returns borrowed labels, stable keys, shortcuts, and eligibility.
`dispatch` applies app actions. It cannot recreate menu navigation.
ContextMenu adds an anchor and a captured target to the shared Menu contract.
The app must not draw a menu from padded lines or maintain a second menu cursor.

**API gaps to resolve:** `PaletteAction / command-model contract`; `CommandPalette phase signatures`.

**Required checks:** filter query; disabled command; scroll; nested submenu; outside dismissal.

## EX-10 — One Grid for tables and editable cells

**Status:** `proposed_target`.

**Contract owners:** `termrock-grid`.

The model interface separates read access from edit authority.
Do not put SQL strings or database calls in Grid.

```rust
let grid = Grid::new(ids.results).mode(GridMode::Cells);
let response = grid.update_editable(cx, &mut view.grid, &mut model.rows);

match response.action() {
    Some(GridAction::SortRequested(column, order)) => model.sort(*column, *order),
    Some(GridAction::EditApplied { row, column }) => model.note_local_edit(*row, *column),
    _ => {}
}

// Draw pass.
Grid::new(ids.results)
    .mode(GridMode::Cells)
    .draw(ui, areas.results, &view.grid, &model.rows);
```

This target selects the editor-capability policy: GridEditor applies a validated local edit.
`EditApplied` reports that operation; it does not request the same edit again.
`note_local_edit` records app-level dirty metadata. It must not apply the cell edit twice.
Read-only usage accepts only the read model and cannot mutate it.
SQL preview belongs in the app domain adapter, outside Grid.
Row mode and cell mode retain different visual recipes on the same Grid mechanism.

**API gaps to resolve:** `GridModel / GridEditor accepted capability shape`; `GridMode`; `GridAction local-edit contract`.

**Required checks:** invalid edit; sort with draft; range selection; server rejection simulation; pending SQL preview.

## EX-11 — A log viewport, not a repainted text block

**Status:** `proposed_target`.

**Contract owners:** `termrock-viewport`, `termrock-controls`.

The app appends data. TextViewport owns reading position and selection policy.

```rust
for line in simulation.take_output() {
    model.log.append(line); // App-owned keyed source, with a changed revision.
}

let response = TextViewport::new(ids.output)
    .follow(FollowPolicy::WhenAtTail)
    .update(cx, &mut view.output, &model.log);
model.handle_output_request(response);

// Draw pass.
Panel::new(ids.output_panel)
    .title("Output")
    .draw(ui, areas.output, |ui, body| {
        TextViewport::new(ids.output)
            .follow(FollowPolicy::WhenAtTail)
            .draw(ui, body, &view.output, &model.log);
    });
```

`handle_output_request` handles typed copy/link requests only.
Use a prose policy where automatic tail following is wrong.
Do not reset scroll offset on each tick or replace state to match a snapshot.
Copy uses source text, not a reconstruction from painted terminal cells.

**API gaps to resolve:** `FollowPolicy`; `revisioned keyed text source contract`; `viewport request contract`.

**Required checks:** append at tail; manual scroll; selection and eviction; horizontal scroll; protected-row fade.

## EX-12 — Code and diff views

**Status:** `proposed_target`.

**Contract owners:** `termrock-editors`, `termrock-layout`.

The app supplies text, diagnostics, and diff records.
Editors own caret geometry, selection, scrolling, and edit policy.

```rust
let response = CodeEditor::new(ids.query)
    .read_only(model.read_only)
    .diagnostics(&model.diagnostics)
    .update(cx, &mut view.query, &mut model.query);
model.apply_editor_request(response);

// Draw pass.
let [query_area, diff_area] = columns(area, [Track::Fill(1), Track::Fill(1)], 1);
CodeEditor::new(ids.query)
    .read_only(model.read_only)
    .diagnostics(&model.diagnostics)
    .draw(ui, query_area, &view.query, &model.query);

DiffView::new(ids.changes)
    .mode(DiffMode::AdaptiveReview)
    .draw(ui, diff_area, &view.changes, &model.diff);
```

`apply_editor_request` changes app data or requests an app operation.
It does not draw diagnostics, line numbers, or a caret.
Diff source retains raw content and source keys.
Narrow fallback is a documented component policy, not a captured-width answer table.

**API gaps to resolve:** `columns<const N>`; `editor controlled-source contract`; `DiffMode::AdaptiveReview`.

**Required checks:** code completion; read-only; diagnostics; wide grapheme caret; narrow diff recovery.

## EX-13 — Motion uses time, not another screen

**Status:** `proposed_target`.

**Contract owners:** `termrock-feedback`, `termrock-navigation`.

A simulated job owns progress data. A component samples the supplied time.
Motion policy changes animation, not the component tree or input routing.

```rust
let now = runtime_frame.moment();

// Draw pass.
let [steps_area, progress_area, usage_area] = rows(
    area,
    [Track::Fill(1), Track::Fixed(1), Track::Fixed(1)],
    1,
);
Steps::new(ids.steps).draw(ui, steps_area, &view.steps, &model.stages);
ProgressBar::new(ids.progress)
    .value(model.completed_fraction())
    .status(model.run_status())
    .sample(now, model.animation_epoch, motion)
    .draw(ui, progress_area);
Meter::new(ids.usage)
    .value(model.quota_used())
    .freshness(model.quota_freshness())
    .draw(ui, usage_area);
```

The method names are target notation. Use verified existing equivalents when suitable.
Steps update runs normally. A cancel Button remains usable while motion is paused.
Progress completion and quota pressure use different semantic roles.
Capture each finite animation phase and timing boundary through the same component path.

**API gaps to resolve:** `time/motion sampling builder shape`; `progress status projection`; `meter freshness projection`.

**Required checks:** spinner phase wrap; feedback 139/140/141ms; paused cancellation; progress rounding; quota thresholds.

## EX-14 — Theme and part customization

**Status:** `proposed_target`.

**Contract owners:** `termrock-theme`, `termrock-controls`, `termrock-navigation`.

The stock theme remains the exact visual baseline recipe.
Custom theme tests use a separate extension contract.

```rust
let label_patch = StylePatch::new().text_role(TextRole::Emphasis);

Button::new(ids.run, "Run")
    .variant(Variant::PRIMARY)
    .patch_part(Part::LABEL, &label_patch)
    .disabled(model.busy)
    .draw(ui, areas.run);

// A content slot returns content, not arbitrary screen pixels.
List::new(ids.jobs)
    .row_content(|job| {
        RowContent::new(job.name.as_str())
            .trailing(BadgeContent::new(job.state.label(), job.state.tone()))
    })
    .draw(ui, areas.jobs, &view.jobs, &model.jobs);
```

`StylePatch::text_role` and declarative row content are proposed contracts.
Theme construction may define palette values at the theme boundary.
Screen code must not select raw RGB values to recreate an exported frame.
A patch cannot change hit geometry, disable a barrier, or hide a required focus marker.
Advanced authors may use a constrained part painter inside an approved reusable component.
That permission does not extend to arbitrary preview screen painting.

**API gaps to resolve:** `StylePatch semantic text-role setter`; `RowContent`; `BadgeContent`.

**Required checks:** default unchanged; custom label applied; disabled barrier retained; slot clipped; sibling unchanged.

## EX-15 — Terminal cells are content inside a component

**Status:** `proposed_target`.

**Contract owners:** `termrock-terminal`, `termrock-controls`.

The source contains only the simulated child terminal content.
It excludes the app menu, dialog, tabs, borders, and status bar.

```rust
let source = model.active_pane_cells();
let response = TerminalView::new(ids.pane)
    .update(cx, &mut view.pane, &source);

match response.action() {
    Some(TerminalAction::Input(request)) => simulation.send_input(request),
    Some(TerminalAction::Copy(range)) => model.request_copy(range),
    _ => {}
}

// Draw pass. The enclosing UI is still component composition.
Panel::new(ids.pane_panel)
    .title(model.active_pane_title())
    .draw(ui, areas.pane, |ui, body| {
        TerminalView::new(ids.pane)
            .draw(ui, body, &view.pane, &source);
    });
```

`send_input` drives the existing simulation, not a newly added real service.
Prepared-cell input is not a back door for a whole-screen screenshot.
The test changes pane data and checks that enclosing components remain independent.
The runtime still routes modal input before terminal input in paused mode.

**API gaps to resolve:** `TerminalSource adapter`; `TerminalAction`; `TerminalView phase contract`.

**Required checks:** continuation cells; cursor; selection/copy; paused modal input; pane resize.

## EX-16 — Jackin workspace prelude composition

**Status:** `proposed_target`.

**Contract owners:** `jackin-preview-app`, `jackin-preview-host-ui`, `termrock-forms`, `termrock-overlays`.

The original five-step flow remains app state.
The app does not become a reusable library component named after Jackin.

```rust
enum PreludeStep { Source, Destination, Edit, WorkingDirectory, Name }

struct PreludeViewState {
    wizard: WizardState,
    source: PickerState,
    destination: TextInputState,
    name: TextInputState,
}

// Update pass: a typed step result changes the preview model.
match wizard_response.action() {
    Some(WizardAction::Advance) if model.current_step_valid() => model.advance(),
    Some(WizardAction::Back) => model.back_preserving_drafts(),
    Some(WizardAction::Cancel) => model.cancel_pending_workspace(),
    _ => {}
}

// Draw pass inside the normal modal layer.
ui.layer(layers.prelude, |ui, area| {
    Wizard::new(ids.prelude)
        .step(model.step_key())
        .title(model.step_title())
        .draw(ui, area, &view.wizard, |ui, body| {
            match model.step {
                PreludeStep::Source => {
                    Picker::new(ids.source)
                        .draw(ui, body, &view.source, &model.source_choices);
                }
                PreludeStep::Name => {
                    TextInput::new(ids.name)
                        .value(&model.name)
                        .draw(ui, body, &view.name);
                }
                _ => model_step_view.draw(ui, body),
            }
        });
});
```

This is a partial composition, not a complete implementation.
`wizard_response` must come from `Wizard::update` and active child updates in the same work item.
`model_step_view` denotes the other documented component-only step compositions.
It is not a callback that may paint rows or chrome.
List the Source, Destination, Edit, WorkingDirectory, and Name compositions explicitly in the final app document.
Retain selected source, path drafts, read-only choice, backtracking, and cancellation.
Do not add a special `draw_prelude_120_40` path.

**API gaps to resolve:** `Wizard step composition and typed transition contract`; `app step-data adapter`.

**Required checks:** all five steps; backtracking; invalid name; changed source label; read-only choice; paused input.

## EX-17 — TablePro workbench composition

**Status:** `proposed_target`.

**Contract owners:** `tablepro-ui`, `tablepro-sql`, `termrock-grid`, `termrock-editors`.

TablePro owns mock query results, pending edits, and safety decisions.
Its UI uses the same Grid, CodeEditor, Tabs, and Dialog as other consumers.

```rust
// Update pass.
let edit_response = results_grid.update_editable(cx, &mut view.grid, &mut model.rows);
model.apply_grid_request(edit_response);
let query_response = query_editor.update(cx, &mut view.query, &mut model.query);
model.apply_query_request(query_response);

// Draw pass. No row text is reconstructed from a stored screenshot.
let [tabs_area, body] = rows(area, [Track::Fixed(1), Track::Fill(1)], 0);
tabs.draw(ui, tabs_area, &view.tabs, &model.tabs);
let [query_area, results_area] = split.areas(body, &view.split);
query_editor.draw(ui, query_area, &view.query, &model.query);
results_grid.draw(ui, results_area, &view.grid, &model.rows);

ui.layer(layers.confirm_changes, |ui, dialog_area| {
    confirmation.draw(ui, dialog_area, &view.confirmation, |ui, body| {
        CodeEditor::new(ids.sql_preview)
            .read_only(true)
            .draw(ui, body, &view.sql_preview, &model.sql_preview);
    });
});
```

`apply_grid_request` updates typed app-owned pending edits.
The SQL adapter then creates SQL from those edits, with the original safety vectors.
It does not accept component pixel state as SQL input.
Do not implement a historical connection/table/query renderer beside these components.

**API gaps to resolve:** `accepted Grid and editor phase contracts`; `typed app reducer adapters`.

**Required checks:** pending edit preview; invalid cell; sort/filter; safety acknowledgement; query cancellation.

## EX-18 — Holla finder and activity composition

**Status:** `proposed_target`.

**Contract owners:** `holla-ui`, `termrock-fields`, `termrock-navigation`, `termrock-viewport`.

Holla supplies fixture resources, matching rules, plans, and activity data.
Termrock owns search input, row presentation, overlays, and log interaction.

```rust
// Update pass.
let query_response = search.update(cx, &mut view.search, &mut model.query);
model.apply_search_request(query_response);
let row_response = results.update(cx, &mut view.results, &model.matches);
model.apply_result_request(row_response);

// Draw pass.
let [search_area, rest] = rows(area, [Track::Fixed(1), Track::Fill(1)], 1);
search.value(&model.query).draw(ui, search_area, &view.search);
let [results_area, preview_area] = split.areas(rest, &view.split);
results.draw(ui, results_area, &view.results, &model.matches);
TextViewport::new(ids.preview)
    .follow(FollowPolicy::Never)
    .draw(ui, preview_area, &view.preview, &model.preview_text);
```

Plan and activity screens use Steps, ProgressBar, TextViewport, and standard action controls.
App-specific ranking and eligibility remain data policies.
App code must not reimplement generic picker navigation, fades, or selection.
A narrow preview drawer is a normal component layout, not another fixture-specific screen.

**API gaps to resolve:** `borrowed match/preview adapters`; `follow policy`; `component-only layout helper`.

**Required checks:** search; no matches; narrow drawer; activity output; cancel; trust dialog.

## EX-19 — Showcase proves ordinary consumer use

**Status:** `proposed_target`.

**Contract owners:** `showcase-ui`, `showcase-demos`.

Showcase must use the same public components as the other previews.
Its pages may provide different data and part patches, not replacement widget implementations.

```rust
// Each registered demo owns typed fixture data and component state.
let response = navigation.update(cx, &mut view.navigation, &catalog.entries);
if let Some(NavAction::Activated(key)) = response.action() {
    model.selected_demo = *key;
}

// Draw pass.
let [navigation_area, demo_area] = split.areas(area, &view.split);
navigation.draw(ui, navigation_area, &view.navigation, &catalog.entries);
catalog.demo(model.selected_demo)
    .draw_components(ui, demo_area, &model.demo_data, &view.demo_state);
```

`draw_components` is a small app page interface, not a universal Widget replacement.
Its contract permits component construction and composition only.
Its update method must use those same component instances, IDs, and data rules.
Each demo documents stock use, one customization, and relevant error/empty states.
A blank page with imports does not count as a consumer example.

**API gaps to resolve:** `app demo-page interface`; `typed demo fixture/state mapping`.

**Required checks:** every page; changed demo data; keyboard navigation; mouse navigation; customization.

## EX-20 — How to handle a missing reusable primitive

**Status:** `proposed_target`.

**Contract owners:** `termrock-controls`, `termrock-feedback`.

Do not implement a new screen painter because one component cannot express a required detail.
Write a small component contract first.

```rust
// Proposed reusable content primitive, not a whole-screen painter.
pub struct Label<'a> {
    text: &'a str,
    role: TextRole,
    overflow: TextOverflow,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self;
    pub fn role(self, role: TextRole) -> Self;
    pub fn overflow(self, policy: TextOverflow) -> Self;
    pub fn measure(&self, cx: &MeasureCx<'_>, limits: Constraints) -> Size;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect;
}
```

These are signature declarations, not compiling function bodies.
Use an existing public primitive instead if it already meets this need.
A Label may implement low-level text painting in its owning library crate.
It may not read a scenario ID or contain a Jackin screen table.
Its contract covers clipping, Unicode, semantic styles, overflow, and noninteractive behavior.
A generic animation primitive may likewise own time-sampled art from parameters and seed.
Do not turn it into an interface for replaying complete reference screens.

**API gaps to resolve:** `Label`; `TextRole`; `TextOverflow`; `MeasureCx accepted measurement contract`.

**Required checks:** Unicode clipping; empty label; semantic theme; noninteractive label; two unrelated data sets.

## EX-21 — Prohibited: snapshot-shaped application rendering

**Status:** `forbidden_example`.

The following structure must be rejected even when its PNG matches.

```rust
if size == (120, 40) && scenario == Scenario::Returning && motion.is_paused() {
    draw_historical_manager(ui);
    return;
}

// Also prohibited in a preview screen:
ui.paint_str(Rect::new(20, 15, 1, 1), "▎", historical_style);
ui.paint_str(Rect::new(21, 15, 70, 1), padded_workspace_row, historical_style);
```

It selects a different implementation for a known test fixture.
It duplicates component geometry, styles, or state representation.
It can leave input behavior unrelated to the visible screen.
Use ordinary component composition for that data and geometry instead.

**Required checks:** historical-dispatch rejection; neighboring width; changed workspace label; paused mouse input.

## EX-22 — Prohibited: moving the shortcut into a helper

**Status:** `forbidden_example`.

A Termrock namespace does not make this reusable:

```rust
// Prohibited, even inside crates/termrock-*/.
TermrockScreen::from_rows(EXPORTED_JACKIN_SCREEN_120_40).draw(ui, area);

// Prohibited: the source includes app menus, fields, and dialogs.
TerminalView::new(ids.whole_app).source(saved_full_app_frame).draw(ui, area);

// Prohibited: a control is called, but its result is painted over.
grid.draw(ui, area, state, rows);
historical_table_overlay(ui, area);
```

Reusable prepared terminal content is limited to the child terminal surface.
It does not include the enclosing preview interface.
Content descriptions may contain real fixture text. They may not encode final screen cells.
The reviewer must trace helpers and generated data to their source.

**Required checks:** stored-answer dependency check; library-painter mutation reaches real app; action mutation fails actual journey.

## Acceptance of the recipe set

Map the recipes to every required component and all four preview applications.
The 22 recipe groups do not replace the 45 detailed component contracts.
For each missing component, add a specific example to its own reference.

A future implementation task must promote each positive target example with real evidence.
A compiling example is not proof of correct visual behavior.
A screenshot is not proof that the documented component created it.
Require compilation, state/action checks, component ownership, default parity, and supported customization.
Keep unimplemented API gaps pending. Do not use `ignore` fences to declare completion.
