# Slice 6 — TablePro migration plan

> Produced read-only by . Two corrections to the inputs are recorded in the "Deliverable note" below and must be applied to  §16.4.

## Deliverable note

I am read-only, so I have **not** written `docs/plans/slice6-tablepro.md`. Below is the complete plan, ready for `fable-builder` to commit verbatim to that path. Everything marked **[F]** was read from the tree at this HEAD (the `src/bin/tablepro/` tree is unchanged since `d5e7075`); everything else is inference or a design decision for the coordinator to record.

Two corrections to the inputs, up front, because they change the contract:

* **[F]** `src/bin/tablepro/app_tests.rs` contains **23** `#[test]` functions, not 21. The audit's prose says "21 tests" (`docs/audit/app-audit.md:443`) but its own table lists 23 rows (`:447-469`). The two the count omits are `acceptance_flow_keyboard_only` (`app_tests.rs:680`) and `acceptance_flow_mouse` (`app_tests.rs:782`). §16.4's "21 tablepro" inventory is likewise wrong and `architecture::every_named_test_exists` will be counted against the real 23.
* **[F]** The 42 digests are 21 surfaces × 2 sizes (`visual_tests.rs:82-236`, `:242-252`), and the file lives *inside the binary* (`main.rs:14-15`), hashing with a per-cell `format!` (`visual_tests.rs:21-32`) — it must move to `apps/tablepro/tests/visual.rs` on `Harness::snapshot()`/`Scene`/`Baseline` (§16.3).

---

---


**Status:** executable. Preconditions: Slice 5 closed (the `tui-next` → `junie-tui` rename, root `src/` and its three `[[bin]]`s removed, `apps/` layout live), Slice 4 packages 4A–4I closed, `crates/tui/tests/fixtures/grid_model.rs` green. Slice 6 and Slice 7 run in parallel over disjoint app trees (Appendix A "Dependency summary").

**Authority:** `REFACTORING_GOAL.md` §18/§22.2/§23-H/§29 › `DESIGN.md` › the pre-refactor captures and `tests/baselines/tablepro.txt` › `COMPONENT_ARCHITECTURE.md` §12.3, §14, §17, §18, §21–§29. DOM §1.6's 22-capability table is adopted verbatim as the checklist (§12.3, final sentence).

---

## 0. Preconditions and the one-commit package move

**[F]** Today: single package, `src/bin/tablepro/{main,app,workbench,tabs,connections,db,model,sql}.rs` + `#[cfg(test)] app_tests.rs, perf_tests.rs, visual_tests.rs` (`main.rs:4-16`).

Target (Appendix B.2):

```
apps/tablepro/Cargo.toml          # [lib] tablepro_app, [[bin]] tablepro, publish = false
apps/tablepro/src/lib.rs          # pub mod list + pub run() + the app's pub const Ids
apps/tablepro/src/main.rs         # fn main() { tablepro_app::run() }
apps/tablepro/src/{app,workbench,tabs/…,connections,grid_model,filter_editor,db,model,sql}.rs
apps/tablepro/tests/{app_tests.rs, visual.rs, perf.rs, baselines/tablepro.txt}
```

* **[F]** `main.rs:28-59` parses `--color`/`--connect`; `main.rs:81-94` implements the old `runtime::Application`. Appendix B.5 requires `--theme {junie|paper}` pass-through for the capture matrix — add it here.
* **[F]** `Theme::for_level(opts.level)` (`main.rs:63`) becomes `Theme::junie().downgrade(level)` / `Theme::paper().downgrade(level)` (§11.2).
* `impl Application for App` becomes `impl junie_tui::App for App` with `update`/`draw`/`should_quit`/`keymap`/`min_size`/`on_esc` (§17.0 A1). `tick_interval` is **deleted** (§8.5, `cx.request_repaint_after`).
* **[F]** `db.rs`, `sql.rs` are pure domain and move **unchanged**. `model.rs` moves with one signature change (§2.7 below).
* `tests/baselines/tablepro.txt` (today at the repo root, `visual_tests.rs:253`) moves to `apps/tablepro/tests/baselines/tablepro.txt` unchanged, so it stays a genuine before-image (§20.10-14).

---

## 1. Screen-by-screen migration table

`«tui»` = `crates/tui/src/`, `«tp»` = `apps/tablepro/src/`. "Deletes" lists the manual plumbing that must be *gone*, not merely moved.

| # | Surface | Today (`file:line`) | Target composition | Plumbing deleted | Tests that must keep passing |
|---|---|---|---|---|---|
| 1 | Connections list (tree + filter) | `connections.rs:266-267`, build `:283-317`, keys `:470-484`, click `:749-769`, render `:938-966` | `Tree<'a, Connection, K, R>` keyed by `ItemKey::text(&c.name)` + `Field::new("", TextInput)`; `Panel::framed("Connections")` | `tree.locate(id)` + `on_click_toggle`/`on_click_row` (`:749-755`); `cx.focus.focus(self.tree.id)` (`:750`); `connection_at_path` path→index (`:319-325`); the rebuild-and-rescue-cursor idiom (`:309-315`); the `/`-to-filter interception (`:471-475`) → a `Binding` | 1, 2, 20, 23 |
| 2 | Connection detail card + action row | `connections.rs:975-1117` | `Panel::card` + `Props` + `layout::action_row` + 4 `Button`s; spinner via `Status::Busy` on the Connect button | `row_layout` (`:1113`) → `layout::action_row`; the 5-button `for (b, action)` ladders (`:489-501`, `:773-785`); the render-time `connect_btn.busy`/`.label` writes (`:1101-1106`) move to props built once (§13) | 1, 2, 23 |
| 3 | **Connection form (15 fields)** | struct `:62-87`, ctor `:107-211`, keys `:573-687`, submit `:689-736`, click `:793-861`, paste `:863-887`, render `:1120-1256` | **`Form<'a>` + `FormData` exactly as §17 example 13** (`crates/tui/examples/13_connection_form.rs` is the executable template); `ConnDraft` is a 15-owned-value struct; `Field<C>` owns all chrome | `on_form_key`'s 115-line ladder and the `input!` macro (`:583-620`); `on_form_click` (`:793-861`); `on_paste` (`:863-887`); `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic (`:1144-1186`); `validate()`'s `a && b` (`:246-250`) and the focus-to-first-error block (`:704-711`); the engine→port widget rebuild (`:629-631`); the render-time `disabled` writes (`:1205-1206`) and tab-error write (`:1134`) | 1, 20, 23 + new `tablepro::connection_form_keyboard_and_mouse_reach_every_field`, `…_focuses_the_first_invalid_field`, `…_password_is_masked_and_absent_from_the_frame` (§16.4) |
| 3a | Password field | **[F]** `connections.rs:155-157` — a *plain* `TextInput` with a placeholder, **not** masked | `FieldKind::Text(TextInput::new(PW).secret(SecretPolicy::default()))`, value `FieldMut::Secret(&mut Secret)` | the plaintext `String` in `ConnForm`; `to_connection`'s clone path (`:213-244`) becomes by-reference (`Secret: !Clone`, §15.1 risk 5) | new `tablepro::connection_password_is_masked_and_absent_from_the_frame` |
| 4 | Delete-connection dialog | `:534-544`, `on_dialog_closed :551-571` | `Dialog::destructive(...)` opened with `cx.open_layer(DELETE, dlg.layer(cx))`; result via `Response<DialogAction>`, **not** a polled `result` field | `DialogResult::Action(1)` index matching (`:557`) → `ActionKey`; `on_dialog_closed` returning `bool` (`:551`) | 20 |
| 5 | Workbench chrome (explorer pane / drawer / body panel) | `workbench.rs:1210-1319`; narrow drawer `:1221-1243` | `SplitPane` (explorer ∥ body) with a narrow-collapse mode; `Panel::framed` for each pane; surface pushed with `ui.with_surface` | every `bg` argument (`:1214, 1252-1265, 1314-1316`) and `panel.bg(t)` (`:1250, 1310`); the zero-rect focus-stop hack `ctx.control(id, Rect::ZERO, false)` (`:1268`, `:1275`) — **see Q3** | 21, 20, 23 |
| 6 | Explorer tree + filter | `workbench.rs:104-107`, build `:133-159`, `schema_children :161-198`, `table_children :200-302`, `object_at :305-322`, `schema_at :324-330`, `tick_explorer :332-357`, keys `:608-674`, click `:975-1010`, wheel `:1166-1168` | `Tree<'a, ExplorerNode, K, R>` with `TreeNode::keyed(ExplorerKey)`; lazy children unchanged; `Field`-less `TextInput` for the filter | **`object_at`/`schema_at` positional path reconstruction (`:305-330`) is deleted** — the key *is* `ExplorerKey::{Db, Schema(String), Section(String, ObjectKind), Object(String, String), …}`; `explorer.locate(id)` (`:980`); `scrollbar::id_for(explorer.id)` (`:1008`, `:1141`); the filter widget rebuild (`app.rs:802-804`, `:1560-1562`) | 1, 3, 20, 21, 22, 23 |
| 7 | Tab strip | `workbench.rs:110/120`, `sync_strip :369-426`, `open_table :428-468`, `new_query :470-478`, `close_tab :501-514`, keys `:675-689`, click `:1011-1031` | `Tabs<'a, WorkTab, K, R>` keyed by `TabKey(u64)` (a monotonic counter on `Workbench`), `.row(...)` painting prefix/dirty/busy/error | **the whole-widget rebuild + `first`/`active` rescue (`:406-410`)**; `ID.sub("tab").child(len + query_counter + 1000)` (`:447-449`); `close_tab`'s index shifting (`:507-511`); `strip.owns(id)` (`:1011`); the `CLOSE_DIALOG.child(i)` scan (`app.rs:1841-1852`) → `ActionKey` + the tab's `ItemKey` | 16, 19, 20, 22, 23 |
| 8 | Table tab mode tabs (Data / Structure) | `tabs.rs:407-409, 764-770`; keys `workbench.rs:712-715`; click `:1037-1040` | `Tabs` with two `ItemKey`s; `TableMode` derived from `TabsState::active_key()` | `mode_tabs.locate(id)` + `cx.focus.focus(t.mode_tabs.id)` (`:1037-1039`); `mode_tabs.active == 0` index reads (`tabs.rs:766, 1297`, `app.rs:677`) | 5, 20, 22, 23 |
| 9 | Filter chips | `tabs.rs:405-406`, `rebuild_chips :518-533`, `:780-784`; keys `workbench.rs:729-758`; click `:1062-1083` | `ChipBar<'a, Filter, K, R>` keyed by `ItemKey::num(filter_id)`; the `match all ▾` lead becomes a `Part::PREFIX` slot or a sibling `Button` | `chips.owns(id)` (`:1062`); the six-arm `ChipEvent` match duplicated in keys and clicks (`:731-756` / `:1065-1081`); `chips.chips` rebuilt per load (`tabs.rs:518-527`) — chips now borrow `&[Filter]` per phase | 4, 20, 22 |
| 10 | **Data grid (table tab)** | `tabs.rs:394-404`, `load :478-516`, render `:790` | `Grid<'a>` + `TableGridModel` (`«tp»/grid_model.rs`) via `Grid::update_editable` / `Grid::draw` (§12.3, §23 K2) | see §2 in full | 3, 4, 17, 20, 22, 23 |
| 11 | Grid status line (bespoke priority drop) | `tabs.rs:794-838` — a hand-written `while … remove lowest priority` loop | `StatusBar` with `Left/Center/Right` groups and per-item priority (§14.1, §20.10-9) | the loop (`:825-832`) and its `min_by_key` + `expect("non-empty")` (`:827-830`) | 4, 5, 20, 22 |
| 12 | Structure view (6 sections) | `structure_tabs tabs.rs:410-422`, `rebuild_structure :535-700`, render `:865-889`; `DataTable` `:436, 655`; routing `workbench.rs:716-725, 1041-1058, 1149-1151` | **six read-only `GridModel`s** over `&Table` (§18.2 `table` row, DOM §2.12) driven by `Grid::update` (shared bound, `&M`) + a `Tabs` section strip | `DataTable` entirely; `structure.owns/locate_header/locate` (`workbench.rs:1049-1055`); `structure_tabs.locate` (`:1041`); the six `Vec<Vec<Cell>>` rebuilds (`tabs.rs:536-647`) become borrowed rows | 5, 20, 22, 23 |
| 13 | DDL pane | `tabs.rs:437, 699, 849-863`; wheel `workbench.rs:1177-1179` | `TextViewport` with tone-carrying `Span`s (§14.1 `ScrollPanel` → **Remove**) | `ScrollPanel`; the `fn(&Theme,&str)->Style` line styler (`tabs.rs:855-862`); `scrollbar::id_for(ddl.id)` (`workbench.rs:1059`, `:1177`) | 5, 20 |
| 14 | Query editor | `tabs.rs:970-973`; keys `:1330-1378`; click `:1447-1455`; drag `:1497-1503`; wheel `:1516-1518`; render `:1558` | `CodeEditor` on `TextEditorCore`, `Highlighter`/`Segmenter` as `&'a dyn Fn` closing over the catalog | `scrollbar::id_for(editor.id)` (3 sites: `:1454, 1501, 1516`); `let was = cx.focus.is(id); cx.focus.focus(id); editor.on_click(pos, was)` (`:1448-1450`); the `Ctrl+Space`/`KeyCode::Null` hack (`:1346-1355`) → a `Binding` | 6, 7, 8, 9, 13, 20, 21, 22, 23 |
| 15 | Completion | `tabs.rs:984`, `trigger_completion :1230-1258`, `accept_completion :1260-1274`, `:1331-1345, 1438-1446, 1513-1515, 1933-1936` | `CompletionController` owning the editor↔popup contract (§14.1); popup is a `Popover` layer, owner keeps focus | **the ~45 lines of hand-wiring** (`:1331-1345` + `:1357-1367` + `:1438-1446`); the anchor arithmetic `cursor_cell().x - replace` (`:1239-1242`); the closing-paren splice (`:1268-1272`) stays domain but moves into the controller's `accept` callback; "draw the popup last" (`:1933-1936`) → layer compositing | 6, 20, 22 |
| 16 | Result tabs | `tabs.rs:974-975`, `sync_result_tabs :1176-1193`, `:1195-1227, 1379-1398, 1457-1466, 1566-1570` | `Tabs` keyed by `ResultKey(u64)` (`result_counter`); pin/close/reorder stay app composition (§12.4) | the whole-widget rebuild (`:1191-1192`); `result_tabs.owns(id)` (`:1457`); `p`/`.`/`x` intercepted before `Tabs::on_key` (`:1380-1390`) → `Binding`s | 9, 20, 22 |
| 17 | Result status line | `tabs.rs:1572-1647` | `StatusBar`, same items | the manual spinner overpaint (`:1638-1645`) → `Status::Busy` on the strip item | 6, 7, 8, 9, 20 |
| 18 | **Results grid** | `tabs.rs:2062-2081`, render `:1662`, keys `:1404-1414`, click `:1471-1478`, wheel `:1523` | `Grid` + `ResultGridModel` (`«tp»/grid_model.rs`), same adapter type as #10 with `is_editable() == false` and `read_only_reason() == Some(...)` (§23 K2 evaluation bullet 2) | `grid.editable = rs.editable` (`:2063`); `grid.local_sort = true` (`:2068`) — **see Q1**; `g.owns(id)` (`:1471, 1523`) | 6, 9, 20, 22, 23 + `tablepro::result_grid_sorts_locally_and_refuses_edits` |
| 19 | Affected / Error result bodies | `tabs.rs:1663-1683`, `:1684-1748` | `Panel::card` + `Props`; the error card becomes `Props` + a wrapped detail slot | direct `buf.set_string` + `t.error_fg().bg(cbg)` (`:1696-1746`) | 7, 20 |
| 20 | **Plan tree + metric columns** | `tabs.rs:1749-1852`, `plan_to_tree :1949-1984` | `Tree` with `.row(|node, u| { u.label(..); let mut c = u.columns(&[Fixed(13),Fixed(8),Fixed(10),Fixed(4)]); … })` (§12.2 `RowUi::columns`) | **the paint-over-the-widget hack**: the metric overlay loop (`:1804-1852`) including `let bgc = buf[(cols_x, y)].bg;` (`:1844`) and the numeric index smuggled through `TreeNode::meta` (`:1809`, `:1983`) → an `ItemKey` into `nodes` | 9, 20, 22 |
| 21 | Plan detail card | `tabs.rs:1853-1914` | `Panel::card` + `Props`, driven off `TreeState::cursor_key()` | `row.meta.as_deref().and_then(parse::<usize>)` (`:1858-1859`) | 9, 20 |
| 22 | Plan raw pane | `tabs.rs:2132`, `:1756-1763, 1425-1427, 1488-1490, 1527-1528` | `TextViewport` (`ScrollPanel` removed) | `ScrollPanel`; `scrollbar::id_for(raw.id)` (`:1488, 1527`); the `r` toggle (`:1421-1424`) → a `Binding` | 9, 20 |
| 23 | Cancelled body | `tabs.rs:1917-1927` | `EmptyState::Empty { title, hint }` inside the grid slot | the free `empty::render(..., bg)` | 8, 20 |
| 24 | **History tab** | `tabs.rs:2266-2524`; keys `workbench.rs:861-957`; click `:1097-1135`; drag `:1156-1159`; wheel `:1183-1190` | `SplitPane` + `List<'a, HistoryEntry, K, R>` (keyed by `ItemKey::num(e.id)`) with a `.row()` renderer + `TextViewport` detail + `Field<TextInput>` + `layout::action_row` | **the paint-over-`ListBox` `!` glyph (`:2434-2442`, incl. `buf[(l.x+1,y)].style()`)** → `RowDecor { marker: Some(GlyphRole::Error), tone: Some(Role::Danger) }`; the whole-`ListBox` rebuild + cursor rescue (`:2329-2341`); `list.locate(id)` (`workbench.rs:1103`); three `scrollbar::id_for` (`:1109, 1112, 1156, 1187`); the `/ c s Enter r y` interception before `ListBox::on_key` (`:891-926`) → `Binding`s; `sync_detail_public` (`workbench.rs:1322-1332`) | 15, 20, 22 |
| 25 | **Filter editor modal** | struct `app.rs:99-109`, open `:1368-1433`, keys `:1648-1736`, apply `:1738-1786`, click `:2051-2086`, render `:2337-2468` | `«tp»/filter_editor.rs`: `Dialog` + `Form` (4 fields: Column `Select`, Operator `Select`, Value, Value2) + a live `WHERE` preview `Note` row; `FilterOp` stays domain | **the entire hand-built modal**: the dim loop (`:2345-2357`), the raw `ratatui::widgets::Block` (`:2371-2375`), `ctx.begin_modal()` (`:2358`), the six manual `ctx.hits.register` re-registrations (`:2462-2467`), the surface hit (`:2376`), `Select::HEIGHT`/`TextInput::HEIGHT` arithmetic (`:2389-2437`), `filter_key`'s twice-written Tab/BackTab (`:1665-1677`, `:1725-1735`), `filter_click` (`:2051-2086`), the `Select` rebuild on column change (`:1683-1684`, `:2063-2064`) | 4, 20, 22 + the `filter-editor` digest |
| 26 | Safety dialogs (query gate) | `app.rs:818-985` | `Dialog::facts(...).body(Facts::new(props).code(sql).ack(token))` with the confirm action's `Action::enabled(pred)` evaluated in `update` (§9.2) | `d.initial_focus = d.actions[N].id` index poking (`:971-978`); the render-time ack arming in `dialog.rs:465-470`; `d.width = 74` → `Dialog::width` + `measured_height` (§26 N1, §28 P4) | 10, 11, 12, 13, 20, 22 + the `safety-dialog-typed-ack` digest |
| 27 | Commit dialog | `app.rs:987-1111`; `finish_commit :1113-1153` | same as #26, token from `t.name` | `dlg.initial_focus` index poking (`:1102-1108`) | 17, 20 |
| 28 | SQL preview dialog | `app.rs:1155-1183` | `Dialog::info(...)` with a **scrollable** code slot | **the button surgery**: `d.actions.remove(0); d.cancel_index = Some(0); d.initial_focus = d.actions[0].id;` (`:1178-1180`); the 6-line `code` truncation in `dialog.rs:429-451` disappears with the slot | 17, 22 |
| 29 | Cell viewer dialog | `app.rs:457-476` | `Dialog::info(...)` + `TextViewport` body; reached via `EditIntent::External` ⇒ `GridAction::EditRequested` (§12.3, §21 item 30 A8) | the same button surgery (`:471-473`) | 20 (smoke) |
| 30 | Help dialog | `app.rs:1195-1213` | **`HelpOverlay`** (J5, §14.2) fed by the same `Binding` metadata as `HintBar` | the `\n`-joined string (`:1196-1205`); `d.actions[0].kind = ButtonKind::Secondary` (`:1210`) | 20 + the `help-dialog` digest |
| 31 | Quit / Discard / CloseTab dialogs | `app.rs:274-315`, `:400-416`, `:477-485` | `Dialog::destructive` convenience ctors; results as `DialogAction::Action(ActionKey)` | `DialogResult::Action(1)` (`:1798, 1808, 1820, 1829, 1844`); the `CLOSE_DIALOG.child(i)` scan (`:1841-1852`) | 20, 22 |
| 32 | Pickers ×3 | switcher `app.rs:1215-1279`, tab list `:1281-1336`, safe mode `:1338-1366`, `picker_chosen :1537-1646`, hints `:2169-2181` | `Picker<'a, T, K, R>` per kind, keyed; scopes first-class (`Picker::scopes(&[ScopeKey])`) | **`switcher_targets: Vec<SwitchTarget>` parallel vector** (`:141, 1278, 1540`); the tab index smuggled through `PickerItem.detail` and parsed back (`:1294, 1504-1508, 1607-1613`); `self.scope = (self.scope+1)%4` + rebuild (`:1229, 1241, 1492-1493`); the `hints: &str` parameter (`:2169-2181`) → a `HintLayer` | 14, 16, 18, 20, 22 + 3 digests |
| 33 | Identity strip | `app.rs:2189-2282` | `StatusBar` with clickable `StatusItem`s; `STRIP_SAFE/SCOPE/CONN/HELP` become `PartRef`s of one id (§18.3 #11) | `segments::render(..., t.canvas)` (`:2281`); the four free-floating `WidgetId::of("strip.*")` consts (`:37-40`) and their click arms (`:1971-1990`) | 1, 18, 20, 23 + every digest |
| 34 | Footer hints | `app.rs:2284-2335` | `HintBar` composing top layer ▸ mode ▸ focused component's `Binding`s ▸ screen extras (§13.1) | the per-modal-kind `Vec<Hint>` construction (`:2290-2323`); every `hints(focus)` `match` ladder: `workbench.rs:568-600`, `tabs.rs:717-754`, `:1276-1324`, `:2360-2378`, `connections.rs:889-917`; the manual `EDIT` badge (`:2328-2332`) → `StateFlags::EDITING` | 20 + every digest |
| 35 | Too-small screen | `app.rs:2121-2145` | `TooSmall<'a>` (§18.3 #21), copy strings preserved verbatim | the 4-line centred loop (`:2136-2143`) | 20 (`"Terminal too small"` at 60×15) |
| 36 | Modal shell + routing | `app.rs:112-116, 1187-1193, 1435-1438, 1440-1535, 1788-1854, 1858-2049` | the runtime layer stack (§9); `Modal` enum deleted; results via `Response<XAction>` + `LayerEvent` | `saved_focus` (`:136, 1188, 1437`); `open_dialog`/`close_modal`; `Interaction{focus,hover,pressed,flash,…}` (`:211-225`); the whole `on_mouse` press/hover/`pressed != Some(id)` machine (`:1858-2049`); `hits.hit`/`hit_scroll` (`:1864, 1872, 1889, 1908, 2036`); `focus.ensure_valid`/`ring.first()` reconciliation (`:2106-2112`); `frame.set_cursor_position` (`:2113-2115`) | all 23 |
| 37 | Global chords | `workbench_chord app.rs:584-770` (~20 chords), `esc_ladder :779-812` | a single `KeyMap` returned by `App::keymap()`, split across `KeyPhase::{Capture, Bubble}` (§13.1) — **see Q6, Q7** | the 190-line `match key.code` with `!editing` guards; `Ctrl+C` special-case (`:256-266`) | 5, 6, 8, 9, 13, 14, 15, 16, 17, 18, 20, 21, 22 |

---

## 2. The grid adapter, concretely

### 2.1 What `«tp»/grid_model.rs` owns

Everything in this list is deleted from `crates/tui/src/components/grid.rs` and re-homed here (§12.3 final paragraph, §18.2 `grid` row):

**[F]** moved verbatim (with the edits below): `CellValue` + `text()`/`edit_text()` (`src/widgets/grid.rs:31-61`), `CellKind` + `default_width`/`right_aligned` (`:64-90`), `ColumnSpec` and its builders (`:92-149`), `PendingChanges` (`:186-222`), `UndoAction` (`:168-182`), `RowState` + its derivation `row_state` (`:224-231`, `:512-524`), `default_validator` (`:267-322`), `cmp_cells` (`:1950-1966`), `cell_text` (`:1969-2005`), `record_cell` (`:628-647`), `toggle_delete` (`:649-670`), `remove_inserted` (`:672-690`), `insert_row` (`:692-722`), `duplicate_row` (`:724-746`), `undo` (`:748-775`), `apply_commit_result` (`:778-809`), `discard` (`:811-820`), `pending_label` (`:1432-1448`), the `[Preview SQL, Discard, Save]` bar (`:360, 400-404, 1886-1947`), `Theme::change_glyph` (`:2007-2018`).

### 2.2 Row identity — the one substantive change

**[F]** Today `PendingChanges` is keyed by *source row index* and the doc claims safety only under sorting (`grid.rs:184-185`); `remove_inserted` re-keys the whole map by hand (`:672-690`), and APP §7 Scenario E names this as the gap.

**[I] Decision:** introduce a monotonic row id. This deletes `remove_inserted`'s shift block outright and satisfies §12.2's reconcile rule and conformance case 12.

```rust
// apps/tablepro/src/grid_model.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct RowId(u64);
impl RowId { pub fn key(self) -> ItemKey { ItemKey::num(self.0) } }

/// One owned conversion at load (§20.9-11): one `String` per cell, produced once.
pub struct RowStore {
    ids:    Vec<RowId>,                 // parallel to `rows`
    rows:   Vec<Vec<CellValue>>,        // domain values, for SQL + validation only
    text:   Vec<String>,                // rows.len() * cols, pre-formatted by `cell_text`
    cols:   usize,
    next:   u64,
    order:  Vec<usize>,                 // display -> storage; see Q1
    total:  RowTotal,
    more:   bool,
}
```

`PendingChanges` is re-keyed by `RowId` and each staged cell carries its rendered text so `cell()` never formats on the draw path:

```rust
pub struct PendingCell { pub value: CellValue, pub text: String }
#[derive(Default)]
pub struct PendingChanges {
    cells:    HashMap<(RowId, usize), PendingCell>,
    inserted: Vec<RowId>,                 // ordered, so preview_sql is stable
    deleted:  BTreeSet<RowId>,
}
impl PendingChanges {
    pub fn is_empty(&self) -> bool;
    pub fn dirty_rows(&self) -> impl Iterator<Item = RowId> + '_;   // excludes inserted
    pub fn counts(&self) -> (usize, usize, usize);                  // (updates, inserts, deletes)
    pub fn total(&self) -> usize;
    pub fn is_dirty(&self, r: RowId, c: usize) -> bool;
    pub fn value(&self, r: RowId, c: usize) -> Option<&CellValue>;
    pub fn label(&self) -> Option<String>;                          // was `DataGrid::pending_label`
}
```

### 2.3 Column metadata

```rust
pub struct ColumnMeta {                  // was library `ColumnSpec` (grid.rs:92-149)
    pub name: String, pub kind: CellKind,
    pub primary: bool, pub nullable: bool, pub read_only: bool,
    pub references: Option<String>, pub enum_values: Vec<String>,
    pub type_label: String,
}
impl ColumnMeta {
    /// The library column this maps onto. Called once per load, never per frame.
    pub fn to_column<'a>(&'a self, key: ColumnKey) -> Column<'a>;
    //  title = name · subtitle = type_label · align = kind.right_aligned() ⇒ Right
    //  min/max width = kind.default_width() · sortable = kind != Json
    //  editable = !read_only · prefix_glyph = primary.then(GlyphRole::PrimaryMark)
}
```

### 2.4 The three model impls

```rust
/// The editable table-tab adapter. Also serves views / no-PK tables:
/// `is_editable` returns false and `read_only_reason` returns Some — a *runtime*
/// property of an editor-capable model (§23 K2, evaluation bullet 2).
pub struct TableGridModel {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnMeta>,
    store: RowStore,
    pending: PendingChanges,
    undo: Vec<UndoAction>,
    row_errors: HashMap<RowId, String>,
    cell_errors: HashMap<(RowId, usize), String>,
    read_only_reason: Option<String>,
    follow: [CellAction; 1],            // the `→` affordance, see 2.6
    empty: EmptyReason,
}

impl GridModel for TableGridModel {
    type Row = ();
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> ItemKey;                     // RowId::key
    fn cell(&self, row: usize, col: usize) -> CellRef<'_>;        // borrowed &str + tone + align
    fn row_decor(&self, row: usize) -> RowDecor<'_>;              // marker/tone/strike from RowState
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_>;// dirty / error(&str) / italic-NULL
    fn total(&self) -> RowTotal;
    fn has_more(&self) -> bool;
    fn read_only_reason(&self) -> Option<&str>;
    fn actions(&self, row: usize, col: usize) -> &[CellAction];
}

impl GridEditor for TableGridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent;
    fn apply_cycle(&mut self, row: usize, col: usize);
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
}

/// Six read-only models over `&'a Table`. `GridModel` only — four required
/// methods — driven by `Grid::update` (§23 K2, evaluation bullet 3).
pub struct StructureModel<'a> { table: &'a Table, section: StructureSection }
pub enum StructureSection { Columns, Indexes, ForeignKeys, Constraints, Triggers }
// (DDL is not a grid; it is a TextViewport — §1 row 13.)

/// Result grids. Same type as TableGridModel with a different constructor:
/// `TableGridModel::from_result(rs, specs, editable, reason)`.
```

Domain-only methods that never appear in a library trait:

```rust
impl TableGridModel {
    pub fn pending(&self) -> &PendingChanges;
    pub fn pending_total(&self) -> usize;                          // capability 22
    pub fn stored(&self, r: RowId, c: usize) -> &CellValue;        // SQL generation
    pub fn effective(&self, r: RowId, c: usize) -> &CellValue;     // was DataGrid::value
    pub fn row_ids(&self) -> &[RowId];
    pub fn clear_cell(&mut self, row: usize, col: usize) -> Result<(), FieldError>; // Delete ⇒ NULL
    pub fn insert_row(&mut self) -> RowId;
    pub fn duplicate_row(&mut self, row: usize) -> RowId;
    pub fn toggle_delete(&mut self, keys: &[ItemKey]);
    pub fn undo(&mut self) -> bool;
    pub fn discard(&mut self);
    pub fn apply_commit_result(&mut self, r: Result<(), (RowId, String)>);
    pub fn sort_locally(&mut self, col: usize, dir: SortDir);      // cmp_cells; see Q1
    pub fn reset_order(&mut self);
}
```

### 2.5 The three `EditIntent` policies (the easiest thing to lose)

**[F]** `begin_edit` bakes three special cases into the library (`grid.rs:552-579`) and `commit_edit` bakes the empty-string policy (`:594-604`). They become, in `edit_intent`/`commit_cell`:

```rust
fn edit_intent(&self, row: usize, col: usize) -> EditIntent {
    let m = &self.columns[col];
    if m.read_only { return EditIntent::Refuse { reason: format!("{} is generated", m.name) }; }
    if self.pending.deleted.contains(&self.id_at(row)) {
        return EditIntent::Refuse { reason: "Row is queued for deletion".into() };
    }
    match m.kind {
        CellKind::Bool => EditIntent::Cycle,                                   // true→false→NULL
        CellKind::Json => EditIntent::External,                                // ⇒ EditRequested
        CellKind::Text if width(self.effective(..).text()) > 2 * self.width(col)
                       => EditIntent::External,
        _ => EditIntent::Inline { initial: self.effective(..).edit_text() },
    }
}
```

`commit_cell` reproduces `grid.rs:594-621` exactly: empty text ⇒ `Text("")` for Text/Json/Enum, `Null` if nullable, else `Err("Empty: use Delete for NULL")`; otherwise `validate_cell(meta, text)` (the moved `default_validator`, `grid.rs:267-322`, error strings preserved character-for-character); on success `record_cell`, on failure a `FieldError` the grid renders in the editor.

**[I] Port the library's grid tests first.** `src/widgets/grid.rs:2020-2191` is the only executable specification of these policies; move them to `grid_model.rs`'s `#[cfg(test)] mod tests` **before** moving the code (DOM §7 risk 1).

### 2.6 Cell actions and the FK follow

```rust
const FOLLOW: ActionKey = ActionKey::custom("follow-ref");

fn actions(&self, _row: usize, col: usize) -> &[CellAction] {
    if self.columns[col].references.is_some() { &self.follow } else { &[] }
}
// self.follow = [CellAction::new(FOLLOW, GlyphRole::FollowRef)
//                  .chord(Chord::with(KeyCode::Char(']'), KeyModifiers::CONTROL))]
```

The grid paints the `→`, registers its hot zone, and emits `GridAction::CellAction(item, col, FOLLOW)`; the screen resolves `references` and pushes `Request::OpenTableFiltered` exactly as `workbench.rs:831-845` does today.

### 2.7 The action surface and the SQL preview

```rust
// «tp»/tabs/table.rs — the three buttons are child components with their own ids
const BAR:     Id = TABLE_GRID.part(Part::ACTIONS);
const PREVIEW: Id = TABLE_GRID.part(Part::custom("preview"));
const DISCARD: Id = TABLE_GRID.part(Part::custom("discard"));
const SAVE:    Id = TABLE_GRID.part(Part::custom("save"));
```

Their `update` runs beside `Grid::update_editable` in the screen's `update`; their `draw` runs inside `Grid::actions_slot(&|ui, row| { … })`. The summary text is `model.pending().label()` plus the cursor row's `row_errors` entry, reproducing `grid.rs:1891-1913`.

**`preview_sql` signature change** — **[F]** today `model.rs:12-16` takes `grid: &junie_tui::widgets::grid::DataGrid` and reads `grid.pending` + `grid.rows()`:

```rust
// apps/tablepro/src/model.rs — after
pub fn preview_sql(table: &Table, columns: &[ColumnMeta], model: &TableGridModel) -> Vec<String>;
```

No library type appears in it any more. Callers: `app.rs:1018, 1129, 1165`.

### 2.8 DOM §1.6 — how each of the 22 capabilities survives

| # | Capability | Survives as | Obvious? |
|---|---|---|---|
| 1 | Typed cell rendering (NULL italic-muted, `''`, `{…}`, UUID middle-truncated, numbers right-aligned) | `cell_text` moves to `grid_model.rs`; `RowStore.text` is pre-formatted at load; `CellRef { text: &str, tone, align }` + `CellDecor.italic` | yes |
| 2 | Column metadata from the catalog | `ColumnMeta::to_column` (§2.3); `column_specs` (`tabs.rs:297-322`) becomes `column_metas` | yes |
| 3 | Read-only grids with an explanatory reason | `GridModel::read_only_reason` (moved down from `GridEditor` by §23 K2 G3) + `GridEditor::is_editable == false` | yes |
| 4 | Server-side sort vs local sort | `GridAction::Sort(ColumnKey, SortDir)` for **both**; table tabs re-run `t.load(cat)`, result grids call `model.sort_locally()` using `cmp_cells` | **⚠ see Q1** — depends on whether the library `Grid` owns the permutation |
| 5 | Filter chips + `f` / `/` / `F` | `GridAction::FilterOnCell(ItemKey, ColumnKey)` — **indices only**; `Request::FilterOnCell(usize, CellValue)` (`app.rs:55`) becomes `FilterOnCell(ItemKey, ColumnKey)` and `open_filter_editor` reads the value from the model | yes |
| 6 | `• + − !` row markers + warning-toned dirty cells | `RowDecor { marker, tone, strike }` from `RowState`; `CellDecor { dirty: true }`; `change_glyph` becomes `fn row_marker(RowState) -> (GlyphRole, Role)` | yes |
| 7 | Bool cycle `true→false→NULL` | `EditIntent::Cycle` + `apply_cycle` | yes |
| 8 | JSON / long text opens the viewer | `EditIntent::External` ⇒ `GridAction::EditRequested(item, col)` ⇒ `Request::OpenViewer` | yes |
| 9 | Engine-aware validation, in-editor error, `!` marker | `commit_cell -> Result<(), FieldError>`; the grid keeps the editor open and renders the message | yes |
| 10 | Delete ⇒ NULL, NOT NULL refusal | **not a library action.** `Delete`/`Backspace` is a TablePro `Binding` on the grid's `BindingState`; the screen reads `GridState::cursor()` and calls `model.clear_cell(..)`; the refusal comes back as `CellDecor::error` | **⚠ requires `GridState::cursor()` — Q2** |
| 11 | Insert / duplicate with server `DEFAULT` in pk & generated columns | `GridAction::RowAddRequested { after, duplicate }` ⇒ `model.insert_row()` / `duplicate_row()` | yes |
| 12 | Undo (`u`) of the last staged change | app `Binding` ⇒ `model.undo()`; the undo stack is adapter state | yes |
| 13 | Discard all / commit all with confirm dialogs | action-bar buttons + `Ctrl+S`/`U` `Binding`s ⇒ `Request::{CommitPending, ConfirmDiscard}` | yes |
| 14 | SQL preview from pending changes | `preview_sql(table, &columns, &model)` (§2.7) — no library type in the signature | yes |
| 15 | Commit-result folding / per-row error | `model.apply_commit_result(..)`; `row_errors` surface through `RowDecor.message` | yes |
| 16 | FK follow (`Ctrl+]`, trailing `→`, click) | `GridModel::actions` + `GridAction::CellAction(item, col, FOLLOW)` (§2.6) | yes (after §23 K2 absorbed `GridCellActions`) |
| 17 | Fetch-more virtual row | generic; `has_more()` + `GridAction::FetchMore` | yes |
| 18 | Range selection + `y`/`Y` TSV copy | generic; copy reads `CellRef::text`. **Note:** `Y` (with header) must stay distinct from `y` | yes |
| 19 | Row selection `✓` + bulk delete | generic `SelectMode::Multi` + `GridAction::RowRemoveRequested(Vec<ItemKey>)` ⇒ `model.toggle_delete(&keys)` | yes |
| 20 | `rows a–b of N` / `cols a–b of N` + sort/filter/read-only parts | `Grid` supplies `rows_label`/`cols_label`; TablePro composes them in a `StatusBar` (§1 row 11) | yes |
| 21 | Pending bar (`• N pending · 2 updates · 1 delete`, row-error detail, 3 focus stops) | `Grid::actions_slot` + three child `Button`s (§2.7) | **⚠ slot contract — Q5** |
| 22 | Pending count in the identity strip | `w.pending_total()` (`workbench.rs:546-557`) → sums `model.pending_total()`; no library involvement | yes |

**Flagged as not obvious: 4, 10, 21.** Each maps to an open question in §6.

### 2.9 The boundary acceptance condition (DOM §1.6, adopted verbatim)

```bash
! rg -n -i '\b(sql|primary key|nullable|foreign|references|NOT NULL|DEFAULT VALUES|commit queue)\b' \
     crates/tui/src/components/grid.rs
! rg -n 'CellValue|PendingChanges|UndoAction|RowState|default_validator|cmp_cells|Preview SQL' crates/tui/src
cargo test -p junie-tui --test architecture no_domain_vocabulary_in_the_library
cargo test -p tablepro tablepro::grid_adapter_keeps_every_pending_change_capability
```

---

## 3. Ordering and file ownership

Appendix A gives Slice 6 one owner. **[I]** The tree splits cleanly into eight packages with disjoint files; run them as three waves.

| WP | Owner scope | Files owned (exclusive) | Depends on |
|---|---|---|---|
| **6‑0** | Package move + shell skeleton | `apps/tablepro/Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/db.rs`, `src/sql.rs`, root `Cargo.toml` members line | Slice 5 |
| **6A** | **Grid adapter** | `src/grid_model.rs`, `src/model.rs` | 6‑0, lib 4I |
| **6B** | Connections screen + form | `src/connections.rs` | 6‑0, lib 4B + 4F |
| **6C** | Table tab | `src/tabs/table.rs`, `src/tabs/structure.rs` | 6A, lib 4I + 4C + 4D + 4E |
| **6D** | Query tab | `src/tabs/query.rs`, `src/tabs/plan.rs` | 6A, lib 4H + 4F + 4D + 4E |
| **6E** | History tab | `src/tabs/history.rs` | 6‑0, lib 4C + 4E |
| **6F** | Workbench | `src/workbench.rs`, `src/tabs/mod.rs` | 6B–6E |
| **6G** | Shell | `src/app.rs`, `src/filter_editor.rs` | 6F |
| **6H** | Tests + baselines | `tests/{app_tests.rs, visual.rs, perf.rs, baselines/tablepro.txt}` | 6G |

**Contended, append-only in alphabetical position** (the §Slice‑4 protocol): `apps/tablepro/src/lib.rs` (module list + the `pub const Id`s the tests address), `apps/tablepro/Cargo.toml`.

**Dependency edges**

```
Slice 5 ──▶ 6‑0 ──┬──▶ 6A ──┬──▶ 6C ──┐
                  │         └──▶ 6D ──┤
                  ├──▶ 6B ────────────┼──▶ 6F ──▶ 6G ──▶ 6H
                  └──▶ 6E ────────────┘
```

Wave 1 = {6A, 6B, 6E}; wave 2 = {6C, 6D}; wave 3 = 6F → 6G → 6H (serial).

**Library packages Slice 6 consumes** (none may change during the slice; a needed change pauses the slice for a fresh adjudication per Appendix A's dependency summary): 4A `Button`/`KeyHint`/`TooSmall`, 4B `Field`/`TextInput`/`TextArea`/`Select`/`Checkbox`/`Toggle`/`RadioGroup`/`ChipBar`/`Secret`, 4C `List`/`Tree`/`Props`/`NavList`, 4D `Tabs`, 4E `Panel`/`SplitPane`/`ScrollRegion`/`TextViewport`, 4F `Dialog`/`Picker`/`Completion`/`Form`/`HelpOverlay`, 4G `StatusBar`/`HintBar`/`Progress`/`Meter`, 4H `CodeEditor`, 4I `Grid`.

**Per-package gate** (Appendix A Slice 6, expanded to name both render targets per §28 P2):

```bash
cargo fmt --all --check
cargo clippy -p junie-tui -p tablepro --all-targets --all-features -- -D warnings
cargo test -p tablepro --all-targets
cargo test -p junie-tui --test architecture
cargo run -p xtask -- boundary
cargo run -p xtask -- doc-check
cargo test -p tablepro --test perf --release -- --test-threads=1
```

**Slice close gate** additionally: all 23 `app_tests` green; `grid_500x12_load < 8 000 allocs` and `frame_tablepro_grid_500x12_120x40 < 100 allocs/frame` (§16.6); the two added benchmarks `frame_tablepro_connection_form_120x40 < 40` and `frame_tablepro_query_editor_2k_lines < 40`; `apps/tablepro/tests/baselines/tablepro.txt` regenerated **once**, in the order **change → capture → classify → bless** (§16.3), every one of the 42 lines classified in `docs/visual-changes.md`; captures of the connection, editor, grid, tabs, dialog, picker and results surfaces reviewed by a fresh read-only `opus-analyst`.

---

## 4. The regression contract

### 4.1 The wall-clock dependency (must be closed first)

**[F]** TablePro reads the wall clock in five places: `use std::time::{Duration, Instant}` (`app.rs:4`); `status: Option<(String, Instant)>` (`:133`); `flash: Option<(WidgetId, Instant)>` (`:134`); `set_status` (`:207-209`); `interaction()`'s `at.elapsed() < 140ms` (`:212-215`); `on_tick`'s 140 ms flash expiry and 5 s status expiry (`:323-334`); and the two `self.flash = Some((id, Instant::now()))` writes (`:552`, `:1935`). `REFACTORING_STATE.md:30` records this as pre-existing finding (6).

**Replacement:**

* **Press flash is deleted from TablePro entirely.** The runtime owns it (§3.3 step 4, `design.motion.press_flash_ms = 140`) and surfaces it as `StateFlags::PRESSED`. `Interaction` and both `flash` writes go.
* **Status becomes a tick counter, not a deadline.** `status: Option<(String, u32)>` seeded with `design.motion.status_ms / design.motion.tick_ms`, decremented in the `Input::Tick` arm, plus `cx.request_repaint_after(Duration::from_millis(design.motion.status_ms))` so an idle app still wakes. This is deterministic under `Harness::ticks(n)` (§16.4 item 6) and preserves observable behaviour: **[F]** no existing test or digest ever runs long enough for the status to expire (`ticks(14)` is the longest, `app_tests.rs:137`).
* `App::animating()` / `App::tick_interval()` (`app.rs:192-205`) are deleted (§8.5).

**Consequence for the digests, recorded deliberately:** today `grid-cell-editing` and `pending-change-bar` (`visual_tests.rs:119-136`) end on `Enter`, so the 140 ms wall-clock flash may or may not be live when the digest is taken; the `a == b` self-check (`visual_tests.rs:246-249`) passes only because both builds run equally fast. After migration the flash is driven by the runtime's virtual clock and is *deterministically* present or absent. Both digests move; classify under §20.10 (new item — "press flash becomes deterministic under the virtual clock").

### 4.2 `app_tests.rs` — all 23

Two mechanical changes apply to **every** test and are not listed per row: (a) `Outcome::Changed` ⇒ `assert!(r.is_changed())`, `Outcome::Consumed` ⇒ `is_consumed() && !is_changed()`, `Outcome::Ignored` ⇒ `!is_consumed()` (§16.4 item 1); (b) the local `H` harness (`app_tests.rs:15-122`) is replaced by `junie_tui_testing::Harness<App>` — every helper it defines already exists there (`new/key/ctrl/alt/type_str/ticks/mouse/click/text/find/focus`), plus `tab_to`, `click_id`, `area_of`, `ring`, `state_of`, `snapshot`.

| # | Test (`app_tests.rs:`) | What it proves | Verdict |
|---|---|---|---|
| 1 | `connections_screen_lists_and_connects_with_keyboard` :125 | grouped tree; Enter connects; 14 ticks reach the Workbench; strip shows env + safe token | **unmodified** |
| 2 | `failed_connection_shows_error_and_retry` :148 | `ConnectOutcome::Unreachable` ⇒ error text + "Reconnect" | **unmodified** (needs `pub` on `App::connections`, `ConnectionsScreen::start_connect`) |
| 3 | `explorer_opens_table_and_grid_navigates` :165 | Enter on `orders` opens a tab; cursor + horizontal scroll + `cols ` label | **assertions rewritten**: `g.cursor.0/.1` (`:186-187`) → `st.cursor()`; `g.hscroll.offset > 0` (`:188`) → the `cols ` label or `st.col_window()`. **Keystrokes unchanged.** Blocked on **Q2** |
| 4 | `sort_and_filter_on_table_tab` :193 | `s` toggles asc/desc; `f` prefills the filter editor; `filtered (1)`; every visible status cell is `pending` | **keystrokes rewritten**: `BackTab BackTab Enter` (`:224-226`) is the filter editor's Tab order, which `Form` declaration order changes. `t.grid.rows()[..].text()` (`:241`) → `model.effective(..)`. Assertions otherwise unchanged |
| 5 | `structure_view_toggle` :245 | `Ctrl+D` toggles Data/Structure; catalog content; `rows 1–` on return | **assertion rewritten**: `t.mode_tabs.active == 1` (`:260`) → `t.mode == TableMode::Structure` |
| 6 | `editor_completion_and_execution` :266 | completion after `FROM ord`; Enter accepts; `Ctrl+R` runs; `SELECT orders (25)`; history recorded | **keystrokes rewritten**: `h.key(Tab)` (`:268`) → `h.tab_to(QUERY_EDITOR)` — ring composition changes (§20.10-15). State reads (`completion.is_open`, `editor.text`, `editor.editing`) → state accessors |
| 7 | `execution_error_marks_editor_and_result` :310 | engine error text + editor diagnostic + `Error 1` tab | **verify the Esc count** (`:315-316`): the §21 item 3 ladder offers Esc to the focused editor before the layer. Assertion `editor.diagnostics` → `CodeEditorState`/props |
| 8 | `cancel_running_query` :325 | Esc cancels; a cancelled run is **not** recorded in history | **rewritten only if Q7 lands the other way.** Today `workbench_chord` intercepts Esc first (`app.rs:587-599`); under §3.3 step 8 the focused editor sees Esc first. Requires `Esc → Cancel` as a `KeyPhase::Capture` binding gated on `running` |
| 9 | `explain_opens_plan_tree` :343 | `Alt+X` ⇒ plan tree; collapse; `r` shows raw `cost=` | **keystrokes rewritten**: `Tab Tab` (`:356-357`) → `tab_to(PLAN_TREE)`. `r` becomes a `Binding` scoped to the plan body |
| 10 | `safety_gate_intercepts_dangerous_statement_on_production` :367 | facts dialog; **a wrong token keeps Execute out of the focus ring**; Esc ⇒ "Cancelled · nothing was executed" | **unmodified if** the disabled confirm registers `Focusability::Disabled` (present in `entries()`, absent from `reachable()`, §8.1). Blocked on **Q8** |
| 11 | `safety_gate_typed_token_executes` :397 | correct token arms Execute; Enter-commit advances focus to the buttons; run starts | **blocked on Q8** — the `Enter` at `:408` relies on commit-then-advance |
| 12 | `read_only_connection_refuses_writes` :418 | `SafeMode::ReadOnly` refuses without a dialog | **unmodified** |
| 13 | `silent_level_runs_scoped_writes_but_confirms_destructive` :432 | `UPDATE` w/o WHERE runs silently; `TRUNCATE` always confirms | **unmodified iff `Ctrl+L` (clear line, `:446`) stays an editor binding — Q6** |
| 14 | `quick_switcher_opens_table` :458 | fuzzy switcher; Esc clears the query then closes | **assertion rewritten**: `matches!(h.app.modal, Some(Modal::Picker(..)))` → `h.is_open(SWITCHER)` |
| 15 | `history_tab_reopens_query` :477 | `Ctrl+Y`; `/` search; Enter reopens as a new query tab | **unmodified** |
| 16 | `tab_strip_overflow_and_tab_list` :494 | ≥12 tabs at 100 cols ⇒ `‹`/`›`; `Ctrl+G` tab list | **unmodified** (drives `wb.open_table` directly) |
| 17 | `pending_edits_preview_and_save` :530 | edit ⇒ `• 1 pending`; `p` previews the exact `UPDATE`; `Ctrl+S` ⇒ token ⇒ "Saved 1 change"; pending cleared | **assertion rewritten**: `t.grid.pending.is_empty()` (`:565`) → `t.model.pending().is_empty()`. Keystrokes unchanged if Q6/Q8 hold |
| 18 | `safe_mode_picker_changes_level_and_strip` :569 | `Ctrl+L` picker sets `SafeFull`; strip shows `safe+` | **unmodified iff Q6 lands** (`Ctrl+L` must be a `Bubble` binding so the editor keeps clear-line) |
| 19 | `mouse_opens_table_and_switches_tabs` :580 | click ⇒ preview tab; second click promotes; strip click; hover registers | **assertion rewritten**: `h.app.hover.is_some()` (`:598`) → `h.hover().is_some()` |
| 20 | `every_screen_renders_at_representative_sizes` :602 | 5 sizes × a 20-step journey; 60×15 ⇒ "Terminal too small" | **unmodified** (deliberately Tab-count-free) |
| 21 | `narrow_terminals_turn_the_explorer_into_a_drawer` :653 | drawer covers the body while focused; **Tab leaves it and lands in the editor**; `0` reopens; opening a table closes it | **blocked on Q3.** The behaviour rests on `ctx.control(id, Rect::ZERO, false)` (`workbench.rs:1268, 1275`), which R5 ("a component that cannot draw registers nothing") removes |
| 22 | `acceptance_flow_keyboard_only` :680 | the full product journey, keyboard only | **keystrokes rewritten** in two places: the filter-editor Tab order (`:709-714`) and every `Tab`-to-reach step (`:741, 743`) → `tab_to` |
| 23 | `acceptance_flow_mouse` :782 | the same journey by mouse; header click sorts; wheel over the explorer scrolls **without moving focus** | **assertion rewritten**: `h.wb().explorer.scroll.offset` (`:812, 814`) → `TreeState` accessor. Keystrokes/clicks unchanged |

Summary: **8 unmodified** (1, 2, 12, 15, 16, 20 + conditionally 13, 18), **7 assertion-only rewrites** (3, 5, 14, 17, 19, 23 + 7), **5 keystroke rewrites** (4, 6, 9, 22 + conditionally 8), **3 blocked on an open question** (10/11 on Q8, 21 on Q3).

New tests owed by §16.4/§23: `tablepro::grid_adapter_keeps_every_pending_change_capability`, `tablepro::view_grid_is_read_only_with_a_reason`, `tablepro::result_grid_sorts_locally_and_refuses_edits`, `tablepro::connection_form_keyboard_and_mouse_reach_every_field`, `tablepro::connection_form_focuses_the_first_invalid_field`, `tablepro::connection_password_is_masked_and_absent_from_the_frame`, `tablepro::mouse_flow_full_journey` + `tablepro::keyboard_flow_full_journey` (renames of `acceptance_flow_*`), `tablepro::resize_across_every_supported_size`, `tablepro::focus_is_restored_after_every_overlay_closes`, `tablepro::no_diagnostics_are_emitted_during_the_journey`.

### 4.3 The 42 digests

**[F]** 21 surfaces (`visual_tests.rs:82-236`) × `{120×40, 80×24}` (`:242`). The three builders `table_tab :49`, `query_typed :59`, `query_nav :69` read `editor.editing` and must be rewritten against state accessors; `query_nav`'s "the first Esc only closes an open completion popup" (`:71-75`) is exactly the §21 item 3 ladder change and must be re-verified.

**[I] Every one of the 42 lines is expected to move.** The reasons, per surface, are the §20.10 entry each must be classified against:

| Surface | Expected §20.10 causes |
|---|---|
| `connections`, `connections-failed` | 9 (identity strip merges into `StatusBar`), 10 (derived hints), 11 (surface inheritance), 15 (ring composition), 16 (width) |
| `workbench-default`, `explorer-focused` | 9, 10, 11, 15, + drawer/`SplitPane` geometry |
| `table-grid` | 9 (the bespoke status-line drop loop is replaced), 10, 11, + `Column` width/align derived from `ColumnMeta` |
| `grid-cell-editing`, `pending-change-bar` | as `table-grid` + the deterministic press flash (§4.1) + the action-surface slot |
| `structure-view` | `DataTable` → `Grid` (row-hover and sort semantics differ, DOM §2.12), 9, 10, 11 |
| `query-editing`, `completion-popup` | 2 (layer compositing — the popup is no longer "drawn last"), 10, 11 |
| `results-grid`, `error-result` | 9, 10, 11 |
| `explain-plan` | **the metric overlay becomes real `RowUi::columns`** — column x-positions and the meta column change materially; 2, 10, 11 |
| `history-tab` | the `!` glyph becomes `RowDecor` (no longer painted over the row's own style), 10, 11 |
| `quick-switcher`, `tab-list-picker`, `safe-mode-picker` | 2, 4 (picker secondary gains a visible trailing affordance), 8 (uniform backdrop), 10 |
| `filter-editor` | **largest change**: hand-drawn modal → `Dialog` + `Form`; 2, 8 (its dim currently differs from `Dialog`'s), 10, 15 |
| `safety-dialog-typed-ack` | 5 (`y`/`n` become opt-in bindings, visible in the footer), 8, 10, + `measured_height` sizing |
| `help-dialog` | replaced by `HelpOverlay` — a different component |
| `maximised-tab` | 9, 10, 11 |

The regression contract is therefore **not** "the digests are unchanged"; it is: *the pre-refactor `tests/baselines/tablepro.txt` is preserved as the before-image; the 42 lines are blessed exactly once, at the end of the slice, after every line is classified in `docs/visual-changes.md` against a numbered §20.10 item*, and `xtask bless-guard` fails the commit otherwise (§16.3). Anything that cannot be attributed to a §20.10 item is a regression by definition (§20.10 "Not on this list").

---

## 5. Risks

| # | Risk | Mitigation | Test that catches it |
|---|---|---|---|
| R1 | **The three edit-intent special cases and the empty-string/NULL policy are lost.** `grid.rs:552-579` + `:594-604` are subtle and undocumented outside the code. | Port `src/widgets/grid.rs:2020-2191` into `grid_model.rs` **before** moving any code (DOM §7 risk 1). | `grid_model::edit_intent_inline_cycle_external_refuse`, `grid_model::empty_text_is_null_only_when_nullable` |
| R2 | **Removing render-time commit changes observable behaviour.** **[F]** `grid.rs:1518-1520` commits an edit during `draw` when focus was lost — clicking away currently stages a DB mutation. | The shell delivers `Intent::FocusOut` and the grid commits there with `BlurPolicy::CommitAndValidate`; audit every test that clicks away from an editing cell. | `conformance::grid::draw_does_not_commit_or_cancel`; `pending_edits_preview_and_save` (17) |
| R3 | **`Ctrl+L` collides** between the app's Safe-Mode picker (`app.rs:668`, resolved today by `!editing`) and the editor's clear-line, used by tests 13 and 17. A `Capture`-phase app binding wins even while editing, because `Ctrl+L` is not a bare `Char` (§13.1). | Declare the app's `Ctrl+L` in `KeyPhase::Bubble`. Same treatment for `Ctrl+D` (Data/Structure vs the grid's known-dead duplicate-row, `grid.rs:1118`). | `conformance::conflicting_visible_bindings_are_reported`; tests 13, 17, 18 |
| R4 | **Esc while a query is running.** **[F]** `app.rs:587-599` intercepts Esc before anything; §3.3 step 8 gives the focused component first refusal. | `Esc → Cancel` as a `Capture` binding gated on `running().is_some()`. | `cancel_running_query` (8) |
| R5 | **The explorer drawer's focus stop disappears.** R5 forbids registering from a component that cannot draw. | Q3: a zero-area `FocusEntry` (focusable, not hit-testable). Do **not** work around it by elevating the explorer into a `Popover` — that changes the drawer's visual contract. | `narrow_terminals_turn_the_explorer_into_a_drawer` (21) |
| R6 | **`DataTable` deletion regresses the Structure tab.** Six sections + header-click sort (`workbench.rs:1051-1053`); `DataTable` sorts by string with an opt-in numeric parse, `Grid` sorts by value (DOM §2.12). | Six `GridModel`s with `NavUnit::Row`; accept and record the sort-semantics change as a §20.10 item. | `structure_view_toggle` (5); the `structure-view` digests |
| R7 | **Ring composition changes break Tab-count assertions.** `Field` chrome, `scroll_region` parts, disabled-but-registered entries (§20.10-15). | Replace every "Tab N times to reach X" with `Harness::tab_to(id)`; record old/new `reachable()` listings per test in `docs/visual-changes.md` **before** editing an expected value. | tests 6, 9, 21, 22 |
| R8 | **The secret draft makes `ConnDraft: !Clone`.** **[F]** `ConnForm::to_connection(base)` clones today (`connections.rs:213-244`); `Secret` is not `Clone` (§15). | Rebuild `to_connection` as a by-reference build; `base.map(|b| b.last_used.clone())` stays. | `tablepro::connection_password_is_masked_and_absent_from_the_frame`; `conformance::form::secret_never_appears_in_debug` |
| R9 | **Grid allocation budget.** `frame_tablepro_grid_500x12_120x40 < 100 allocs/frame` and `grid_500x12_load < 8 000` (§16.6) require `RowStore.text` pre-formatting (§20.9-11) and `CellRef` borrowing. A naive adapter that formats in `cell()` blows both. | Pre-format at load; `cell()` returns `&str`; `sample_widths` measures `&str` and never formats (§20.9-11 explicitly deletes the `CellValue::display_width` clause). | `grid_500x12_load`, `frame_tablepro_grid_500x12_120x40` |
| R10 | **Digest bless race / partial matrix.** **[F]** `REFACTORING_STATE.md:78` records a bless race (since fixed) and §28 P2 records that `--test render` alone runs half the matrix. | Name both targets in every gate command; bless once, at slice close, on the committed tree. | `xtask bless-guard`; `rg -n -- '--test render\b' | rg -v render_components` empty |
| R11 | **`preview_sql` string output drifts.** Test 17 asserts the exact `UPDATE public.orders SET currency = 'EUR'`; `sql_literal` formats `Num` as `{n:.2}` (`model.rs:123`). | `preview_sql` is moved, not rewritten; `sql_literal` and `from_cell` unchanged. `pk_where`'s no-PK branch (`model.rs:24-45`) must iterate `RowId` instead of `src`. | `pending_edits_preview_and_save` (17) |
| R12 | **Filter-editor Tab order is a product behaviour, not an accident.** Test 4 and 22 both walk it backwards. | `Form` declaration order = Column, Operator, Value, Value2, then the action row last (F2, F7); record the new sequence in `docs/visual-changes.md` item 15 before editing the tests. | tests 4, 22; the `filter-editor` digests |

---

## 6. Open questions needing adjudication **before** the slice starts

Each blocks a named work package. Q1, Q2 and Q3 also block **Slice 4I / 4C**, so they must be answered before wave 2 of Slice 4, not merely before Slice 6.

| Q | Question | Why it blocks | Recommended answer |
|---|---|---|---|
| **Q1** | **Does the library `Grid` own a sort permutation, and on what comparison?** §12.3's kept-list does not mention `order` or `local_sort`; DOM §1.6 row 4 says "local sort stays a grid option over the model's ordering"; but §16.1 names the *library* unit test `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row`. The grid sees only `CellRef { text, tone, align }`, so it cannot reproduce `cmp_cells`' NULLs-last + numeric ordering (`grid.rs:1950-1966`) without a domain comparison. | 4I, 6A, 6C, 6D; capability 4 | **The adapter presents rows already ordered.** `Grid` emits `GridAction::Sort(ColumnKey, SortDir)` for *both* cases; table tabs re-run the query, result grids call `model.sort_locally()`. Delete `local_sort` from the library. Re-read the library test name as "the grid addresses rows by `ItemKey`, so a model reorder leaves cursor/selection/pending bound to the same logical row" — satisfiable with a fixture model that reorders itself. |
| **Q2** | **What does `GridState` expose?** TablePro's chords (`p`, `u`, `U`, `Ctrl+S`, `f`, `F`, `/`, `+`, `-`, `Ctrl+D`, `Delete`) move to its `KeyMap` (§13.1) and every one needs the grid's cursor. Three migrated tests read `g.cursor`, `g.hscroll.offset`, `g.pending`. | 4I, 6A, 6C, 6H; capabilities 10, 18, 19 | Add `GridState::{cursor() -> Option<(ItemKey, ColumnKey)>, selected_rows() -> &KeySet, is_editing() -> bool, edit_error() -> Option<&str>, row_window() -> Range<usize>, col_window() -> Range<usize>}`. This also removes the need for `CellAction.glyph: Option<GlyphRole>` (Delete⇒NULL becomes an app binding, not a cell action). |
| **Q3** | **May a control register a zero-area focus entry?** R5 says a component that cannot draw registers nothing; `hit::empty_rects_are_rejected` rejects empty *regions*. The explorer drawer needs a focus stop with no geometry (`workbench.rs:1268, 1275`). | 6F; test 21 | Permit `ui.register_control(id, Rect::ZERO, Focusability::Focusable)` — a `FocusEntry` with `area: Rect::ZERO` and **no** hit region. Document it as the "hidden but reachable" case; `Harness::click_id` on it returns `ignored()` + `Diagnostic::UnaddressableId`, which is already the specified behaviour (§21 item 17 F7). |
| **Q4** | **`Grid::actions_slot` contract.** Does it reserve a row unconditionally, or only when the caller supplies a slot? May children drawn inside the `&'a dyn Fn` closure register `Control` regions (the three bar buttons are focus stops today, `grid.rs:1926-1947`)? | 4I, 6C; capability 21 | Reserve one row iff a slot is set; children **may** register controls (the closure receives `&mut Ui`, so `register_control` is reachable and z-order is the draw order). Their `update` runs in the screen beside `Grid::update_editable`. |
| **Q5** | **`Dialog::acknowledge` — does Enter commit the token *and* advance focus to the action row?** **[F]** Tests 10, 11 and 17 all press `Enter → type → Enter → Right → Enter`. Today this is `InputEvent::CommittedTab`. | 4F, 6G; tests 10, 11, 17 | Yes: the ack field's `BlurPolicy` is `CommitAndValidate` and Enter emits `TextAction::MoveNext`, which the dialog turns into `cx.focus(next)`. Record it as a `Dialog` invariant so the three tests stay keystroke-identical. |
| **Q6** | **`KeyPhase` for the app chords that collide with editor bindings** — `Ctrl+L` (Safe Mode vs clear-line), `Ctrl+D` (Data/Structure vs duplicate-row), `Ctrl+F` (filter vs find), `Ctrl+S` (commit vs nothing), `r`/`s`/`f`/`p`/`u` (bare Chars). | 6G; tests 5, 13, 17, 18 | Bare-`Char` chords go in `Capture` (they are already skipped while the focused control `swallows_typing`, §13.1); every `Ctrl+…` chord that a text control also binds goes in `Bubble`. `conformance::conflicting_visible_bindings_are_reported` must be clean at slice close. |
| **Q7** | **Esc precedence while a query is running.** | 6G; test 8 | A `Capture` binding declared only when `running().is_some()`. |
| **Q8** | **Clock access.** §17.0 declares no clock accessor; `Cx::request_repaint_after` schedules a wake but does not report time. The status timeout needs one or the other. | 6G; §4.1 | App-owned tick counter (zero new library surface, deterministic under `Harness::ticks`). Also resolve the token value: §11.2 says `status_ms 4000`, §8.5 says "4000/5000", TablePro uses 5 000 (`app.rs:330`). Pick 5 000 for TablePro via a per-app override, or accept 4 000 and record it as a visual/behaviour change. |
| **Q9** | **`§29` is referenced but absent.** **[F]** `COMPONENT_ARCHITECTURE.md:9, 45, 1734, 1815, 1869, 3943, 6211, 6222` all cite "§29 / Adjudication Q", and `REFACTORING_STATE.md:115` lists "Record as §29" as an open task, but the document ends at §28.8 (line 6285). Slice 6 quotes §29's `Slot<GlyphRole>` contract for `RowDecor`/`CellDecor` markers. | 6A, 6C, 6E | Append §29 before Slice 6 opens; otherwise `xtask doc-check`'s §21–§26 range and `every_named_test_exists` will disagree with the inline markers. |
| **Q10** | **`ScrollPanel` → `TextViewport` with tone-carrying spans** — does `TextViewport` accept our role-carrying `Span` (`«tui»/text/span.rs`, §24 M1) and does the DDL keyword styler (`tabs.rs:855-862`) become span roles built at load? | 4E, 6C, 6D | Yes; build the spans once in `rebuild_structure` / `plan_text`, not per frame. Confirm `TextViewport` exposes a scrollbar as `Part::TRACK/THUMB` of itself so `scrollbar::id_for` disappears. |
| **Q11** | **The test-count discrepancy** (23 vs 21) and the `acceptance_flow_*` renames in §16.4. | 6H | Correct §16.4's inventory to 23 and record both old and new names so `architecture::every_named_test_exists` is satisfiable. |

---

### Collected facts vs inference — summary

Everything in §1's "Today" column, §2.1's move list, §4.1's five wall-clock sites, §4.2's per-test line numbers and §4.3's surface list is **[F]**, read from `src/bin/tablepro/**` and `src/widgets/grid.rs` at this HEAD. The target compositions, the `RowId` decision (§2.2), the work-package split (§3), the per-digest cause attribution (§4.3), all risks and all eleven open questions are **[I]** — decisions for the coordinator to record in `COMPONENT_ARCHITECTURE.md` and `REFACTORING_STATE.md` before Slice 6 opens.

**Files most relevant to the next agent:**
`/Users/donbeave/Projects/terminal-components-claude/src/bin/tablepro/{app.rs,workbench.rs,tabs.rs,connections.rs,model.rs,db.rs,app_tests.rs,visual_tests.rs}`,
`/Users/donbeave/Projects/terminal-components-claude/src/widgets/grid.rs`,
`/Users/donbeave/Projects/terminal-components-claude/COMPONENT_ARCHITECTURE.md` (§12.3 at :1174-1223, §23.2 at :4947-5017, §18 at :3703-3794, Appendix A Slice 6 at :4071-4075),
`/Users/donbeave/Projects/terminal-components-claude/tests/baselines/tablepro.txt`.
