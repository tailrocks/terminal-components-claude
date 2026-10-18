# Complex-Component and Domain-Boundary Audit

Scope: `src/widgets/*` (grid, table, code, completion, diff, statusbar, hintbar, dialog, menu, picker, panel, splitter, viewport, tabs, textarea, input, field_common, select, choice, chips), all TablePro consumers, all Jackin `screens/*` plus `jackin_preview/app.rs`.

Convention: **[F]** = collected fact with citation. **[I]** = inference / recommendation. All paths are repo-relative; `file:line` refers to the file as it stands at the audited revision (d5e7075).

---

## 1. DataGrid dissection (`src/widgets/grid.rs`, 2192 lines)

### 1.1 Type-by-type classification

| Item | Location | Class | Evidence / note |
|---|---|---|---|
| `CellValue::{Null, Default, Text, Int, Num, Bool, Json}` | grid.rs:31-41 | **DATABASE-DOMAIN** | `Default` is documented as "Server default (inserted rows)" (grid.rs:35); `Null` renders as the literal `"NULL"` (grid.rs:46, 1971); `Json` is a SQL column type. A generic grid needs *displayable cells*, not a closed SQL value union. |
| `CellValue::text()` / `edit_text()` | grid.rs:44-60 | **DATABASE-DOMAIN** | `"NULL"`/`"DEFAULT"` literals; `Num` formatted `{n:.2}` — an engine-specific display rule baked into the library. |
| `CellKind::{Text,Id,Number,Bool,Timestamp,Json,Enum}` | grid.rs:64-73 | **MIXED** | Generic half: `default_width()` (grid.rs:76-86) and `right_aligned()` (grid.rs:87-89) are presentation. Domain half: `Id`⇒UUID, `Json`, `Enum`, `Timestamp` drive validation (grid.rs:291-319) and `cell_text` compaction (grid.rs:1974-2001). |
| `ColumnSpec.name` | grid.rs:94 | GENERIC | header text |
| `ColumnSpec.kind` | grid.rs:95 | MIXED | see `CellKind` |
| `ColumnSpec.primary` | grid.rs:96 | **DATABASE-DOMAIN** | drives `⚷` glyph + `▪ ` prefix (grid.rs:1610, 1619-1626), `Default` on insert (grid.rs:700-704), Tab-skip while editing (grid.rs:953) |
| `ColumnSpec.nullable` | grid.rs:97 | **DATABASE-DOMAIN** | `NOT NULL` errors (grid.rs:272, 1107), Delete⇒NULL (grid.rs:1099-1109) |
| `ColumnSpec.read_only` | grid.rs:98 | GENERIC | "this column is not editable" is a generic concept |
| `ColumnSpec.references: Option<String>` | grid.rs:99 | **DATABASE-DOMAIN** | FK target table; drives `→` glyph (grid.rs:1858-1868), trailing-arrow click (grid.rs:1307-1316), `Ctrl+]` (grid.rs:1159-1169) |
| `ColumnSpec.enum_values` | grid.rs:100 | **DATABASE-DOMAIN** | validation only (grid.rs:299-305) |
| `ColumnSpec.sortable` | grid.rs:101 | GENERIC | but default is derived from a SQL type: `kind != Json` (grid.rs:119) |
| `ColumnSpec.min_width/max_width` | grid.rs:102-103 | GENERIC | layout |
| `ColumnSpec.type_label` | grid.rs:105 | GENERIC-shaped, domain-sourced | "muted type label under the name"; only TablePro fills it (`tabs.rs:303`) |
| `RowTotal::{Exact,Estimated,Unknown}` | grid.rs:151-156 | **GENERIC** | any paged/remote source has this |
| `GridRows {rows, total, more}` | grid.rs:158-164 | GENERIC shape, domain payload | `rows: Vec<Vec<CellValue>>` |
| `UndoAction::{Cell,Delete,Insert}` | grid.rs:168-182 | **DATABASE-DOMAIN** | an undo log over the pending-mutation queue |
| `PendingChanges {cells, inserted, deleted}` + `dirty_rows/counts/total/is_dirty/value` | grid.rs:186-222 | **DATABASE-DOMAIN** | explicit: "Nothing reaches the server until the owner commits" (grid.rs:184) |
| `RowState::{Clean,Modified,Inserted,Deleted,Error}` | grid.rs:224-231 | **MIXED** | the *vocabulary* (row decoration) is generic; its *derivation* from `pending` (grid.rs:512-524) is domain |
| `EditState {row,col,buffer,error}` | grid.rs:233-239 | **GENERIC** | |
| `GridEvent::CellChanged` | grid.rs:243 | GENERIC | |
| `GridEvent::RowInserted/RowDeleted` | grid.rs:244-245 | MIXED | generic as "row add/remove requested"; here they report staged mutations |
| `GridEvent::SortRequested` | grid.rs:246 | **GENERIC** | |
| `GridEvent::FetchMore` | grid.rs:247 | **GENERIC** | |
| `GridEvent::Refresh` | grid.rs:248 | **GENERIC** | |
| `GridEvent::CommitRequested/DiscardRequested/PreviewSql` | grid.rs:249-251 | **DATABASE-DOMAIN** | `PreviewSql` names SQL in a reusable widget |
| `GridEvent::Copy(String)` | grid.rs:252 | **GENERIC** | |
| `GridEvent::FollowReference` | grid.rs:253 | **DATABASE-DOMAIN** | FK navigation |
| `GridEvent::OpenViewer` | grid.rs:254 | GENERIC | "this cell is too big to edit inline" is generic |
| `GridEvent::FilterOnCell/OpenFilters/ClearFilters` | grid.rs:255-257 | MIXED | generic as *filter requests*; `FilterOnCell{value: CellValue}` leaks the domain value type |
| `GridEvent::Activated(usize)` | grid.rs:258 | **GENERIC** | |
| `GridEvent::LeaveForward/LeaveBackward` | grid.rs:259-260 | **GENERIC** | focus handoff |
| `type Validator = fn(&ColumnSpec,&str)->Result<CellValue,String>` | grid.rs:265 | GENERIC hook, **too narrow** | bare fn pointer — no closures, no engine state; contradicts goal §19 |
| `default_validator` | grid.rs:267-322 | **DATABASE-DOMAIN** | `"{} is NOT NULL"`, UUID 36-char check, `YYYY-MM-DD`, JSON `{}`/`[]` sniffing, enum membership |
| `cmp_cells` | grid.rs:1950-1966 | **DATABASE-DOMAIN** | NULLs-last ordering is SQL semantics |
| `cell_text(v, kind, w)` | grid.rs:1969-2005 | MIXED | control-char sanitisation + `truncate_middle` for ids are generic; `"NULL"`/`"DEFAULT"`/`"''"`/`{…}` are domain |
| `impl Theme { fn change_glyph(RowState) }` | grid.rs:2007-2018 | **DOMAIN LEAK INTO THEME** | a widget module adds an inherent method to the library `Theme` keyed on a domain-derived enum |

### 1.2 `DataGrid` field-by-field

| Field | Location | Class |
|---|---|---|
| `id` | grid.rs:328 | GENERIC |
| `columns: Vec<ColumnSpec>` (pub) | grid.rs:329 | MIXED (see above) |
| `rows: Vec<Vec<CellValue>>` (private, owned) | grid.rs:330 | **DOMAIN** — grid owns a full copy of the result set |
| `order: Vec<usize>` display→source | grid.rs:331 | **GENERIC** (sort as permutation, so keys survive) |
| `total`, `more` | grid.rs:332-333 | GENERIC |
| `sort: Option<(usize,SortDir)>` | grid.rs:334 | GENERIC |
| `local_sort: bool` | grid.rs:336 | GENERIC |
| `filtered_cols: BTreeSet<usize>` | grid.rs:337 | GENERIC decoration (`∇` at grid.rs:1603) |
| `cursor: (usize,usize)` | grid.rs:338 | GENERIC |
| `anchor` (range selection) | grid.rs:340 | GENERIC |
| `selected_rows: BTreeSet<usize>` | grid.rs:341 | GENERIC |
| `pending: PendingChanges` (pub) | grid.rs:342 | **DOMAIN** |
| `undo: Vec<UndoAction>` | grid.rs:343 | **DOMAIN** |
| `scroll`, `hscroll` | grid.rs:344-345 | GENERIC |
| `edit: Option<EditState>` | grid.rs:346 | GENERIC |
| `editable: bool` | grid.rs:347 | GENERIC |
| `read_only_reason: Option<String>` | grid.rs:348 | GENERIC |
| `loading: bool` | grid.rs:349 | GENERIC |
| `cell_errors`, `row_errors` | grid.rs:350-351 | GENERIC storage, domain-filled (`apply_commit_result`, grid.rs:805-807) |
| `empty: EmptyState` | grid.rs:352 | GENERIC |
| `validator: Validator` | grid.rs:353 | MIXED / too narrow |
| `row_numbers: bool` | grid.rs:354 | GENERIC |
| `area: Rect` (pub), `body`, `widths`, `col_rects` | grid.rs:355-358 | GENERIC frame-local geometry — `area` is **publicly mutable layout state** (goal §10 forbids) |
| `show_bar: bool` | grid.rs:359 | **DOMAIN** |
| `bar: [Button; 3]` = `Preview SQL` / `Discard` / `Save` | grid.rs:360, 400-404 | **DOMAIN** — SQL-named buttons hard-coded in the reusable widget |

### 1.3 Method-by-method

**Generic:** `len/is_empty/rows/source_row` (415-427), `set_rows`/`append_rows`/`set_loading` (438-474), `content_rows` (476), `sample_widths` (480-500, p95 width sampling of first 200 rows), `apply_local_sort` (502-510), `is_editing`/`edit_error` (528-533), `cursor_src` (535), `cancel_edit` (623), `set_cursor` (824-838), `ensure_col_visible` (840-854), `in_range` (857-864), `on_more_row` (866-868), `copy_text` (870-920, TSV + optional header), navigation arm of `on_key` (989-1047), `request_sort` (1174-1185), id helpers (1189-1206), `owns`/`locate`/`locate_header`/`locate_rownum` (1208-1239), `on_drag` (1332-1353), `on_wheel` (1355-1362), `on_scrollbar` (1364-1374), `on_paste` (1376-1384), `position_label`/`rows_label`/`cols_label` (1389-1430), `layout_columns` (1452-1486), `fit_header_marks` (1489-1507).

**Domain:** `value()` (428-435, pending-shadowed read), `row_state` (512-524), `record_cell` (628-647, "reverting to stored clears the change"), `toggle_delete` (649-670), `remove_inserted` (672-690, index-shifting of the whole pending map), `insert_row` (692-722), `duplicate_row` (724-746), `undo` (748-775), `apply_commit_result` (778-809, folds pending into stored rows, drops deleted rows), `discard` (811-820), `pending_label` (1432-1448), `bar_ids`/`on_bar_key` (1926-1947), pending-bar rendering (1886-1923).

**Mixed / straddling:** `begin_edit` (539-588) — generic "start editing the cursor cell" plus three domain policies: `Bool` cycles `true→false→NULL` in place (553-564), `Json` refuses inline editing and emits `OpenViewer` (565-570), long `Text` also emits `OpenViewer` (571-578). `commit_edit` (590-621) — generic commit shape, but the empty-string policy ("Empty: use Delete for NULL", 600) is SQL semantics. `on_key` write arms (1080-1169) mix generic (Delete, `y`/`Y` copy) with domain (`p` preview SQL, `U` discard, `Ctrl+]` follow reference, `Ctrl+S` commit). `on_click` (1241-1330) mixes generic cell/header/rownum hit routing with the pending-bar buttons (1242-1250) and the FK trailing-arrow hot zone (1307-1316).

### 1.4 Render-time semantic mutation (invariant violation, goal §11)

**[F]** `DataGrid::render` commits an edit as a side effect of drawing when focus was lost:

```rust
// src/widgets/grid.rs:1518-1520
let focused = ctx.interaction.focused(self.id);
if !focused && self.edit.is_some() {
    self.commit_edit();
}
```

`commit_edit` runs `(self.validator)(...)`, calls `record_cell`, and mutates `PendingChanges` — i.e. **rendering stages a database mutation**. The same pattern exists at `table.rs:566-568`, `code.rs:611-614`, `input.rs:282-286` (which additionally runs `validate()` via `commit()`, input.rs:161-165), `textarea.rs:202-205`, and `select.rs:165-167` (rendering closes an open overlay).

Additionally `Dialog::render` mutates `actions.last().disabled` from the acknowledgement text (dialog.rs:465-470) — a semantic enable/disable computed inside draw.

### 1.5 Proposed split

**[I] Stays in the reusable grid** (`DataGrid`, no SQL vocabulary):

- Column model: `key: ColumnKey`, `title`, `align`, `min/max width`, `sortable`, `editable`, `sticky`, optional `subtitle` (replaces `type_label`), optional `badge`/`prefix glyph` (replaces `primary`'s `⚷`).
- Viewport: `scroll`/`hscroll`, `layout_columns`, `sample_widths`, `fit_header_marks`, `‹N`/`N›` overflow chips, `rows_label`/`cols_label`.
- Cursor + selection: cursor cell, `anchor` rectangular range, `selected_rows`, `Esc` clears, copy-as-TSV.
- Row/cell decoration supplied by the adapter (see below), not derived internally.
- Editing lifecycle: `begin_edit`/`commit_edit`/`cancel_edit` + explicit `focus_lost()` transition (**removed from `render`**).
- Requests as semantic actions: `Sort`, `Filter`, `ClearFilters`, `Refresh`, `FetchMore`, `Copy`, `Activate`, `LeaveForward/Backward`, `EditRequested`, `RowAddRequested`, `RowRemoveRequested`, `CellAction(ActionKey)`.
- An **action-surface slot**: a caller-supplied row of buttons + summary text, replacing the hard-coded `[Preview SQL, Discard, Save]` bar (grid.rs:400-404) and `pending_label` (grid.rs:1432-1448).
- Loading / empty / error presentation (`EmptyState`, spinner) — already generic (grid.rs:1648-1661).

**[I] Adapter interface the grid must expose** (small focused traits, borrowed data, no `'static`):

```rust
pub trait GridModel {
    type Key: Eq + Hash + Clone;            // stable row identity (replaces "source row index")
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> Self::Key;
    fn cell(&self, row: usize, col: usize) -> CellRef<'_>;   // borrowed text + tone + suffix glyph
    fn row_decor(&self, row: usize) -> RowDecor;             // marker glyph+tone, strikethrough, error msg
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor; // dirty / error(Option<&str>) / muted
    fn total(&self) -> RowTotal { RowTotal::Unknown }
    fn has_more(&self) -> bool { false }
}

pub trait GridEditor: GridModel {
    /// What activation on this cell means. Replaces the Bool/Json/long-Text
    /// special cases baked into begin_edit (grid.rs:552-579).
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent;
    //   Inline { initial: String } | Cycle | External | Refuse { reason: String }
    fn apply_cycle(&mut self, row: usize, col: usize);
    /// Validation + staging in one explicit call. Returns the message the grid
    /// shows in the editor on failure. Replaces `type Validator = fn(...)`.
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), String>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
    fn read_only_reason(&self) -> Option<&str>;
}

/// Optional; drives per-cell affordances such as the trailing `→`.
pub trait GridCellActions: GridModel {
    fn actions(&self, row: usize, col: usize) -> &[CellAction]; // glyph + key + ActionKey
}
```

Because `commit_cell` and `edit_intent` are `&mut self`/`&self` on an application type, closures, engine handles, catalog references and per-connection state are all available — the fn-pointer restriction (grid.rs:265) disappears.

**[I] Moves to TablePro** (`src/bin/tablepro/grid_model.rs`, new): `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, `default_validator`, `cmp_cells`, `apply_commit_result`, `insert_row`/`duplicate_row`/`toggle_delete`/`discard`/`undo`, `references` FK metadata, `primary`/`nullable`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar, `Theme::change_glyph`.

### 1.6 TablePro capabilities that depend on `DataGrid`, and how each survives

| # | Capability | Current dependency (cite) | Survives via |
|---|---|---|---|
| 1 | Typed cell rendering (NULL italic-muted, `''` empty, JSON `{…}`, UUID middle-truncated, numbers right-aligned) | grid.rs:1805-1856, 1969-2005; kinds mapped at tabs.rs:263-273 | Adapter returns `CellRef { text, tone, align, italic }`; grid only draws |
| 2 | Column metadata from the catalog (pk, nullable, generated⇒read-only, FK, enum values, type label) | tabs.rs:297-322 | Adapter keeps `ColumnSpec`; supplies grid with `title`, `subtitle`, `prefix_glyph`, `editable`, `sortable` |
| 3 | Read-only grids for views / no-PK tables with an explanatory reason | tabs.rs:394-403; result grids tabs.rs:2063-2067 | `GridEditor::is_editable` + `read_only_reason` |
| 4 | Server-side sort request vs local sort for fully loaded results | grid.rs:1174-1185; `local_sort=true` at tabs.rs:2068; handled at workbench.rs:789-804 | `GridAction::Sort` unchanged; local sort stays a grid option over the model's ordering |
| 5 | Filter chips + `f` filter-on-cell + `/` open filters + `F` clear | grid.rs:1140-1151; app.rs:392-399; workbench.rs:821-830 | `GridAction::FilterOnCell{row,col}` (indices only, no `CellValue`); TablePro reads the value from its own model |
| 6 | Pending edits with `• + − !` row markers and warning-toned dirty cells | grid.rs:1739-1751, 1820-1826; state from grid.rs:512-524 | `RowDecor`/`CellDecor` from the adapter's `PendingChanges` |
| 7 | Bool cycle `true→false→NULL` without an editor | grid.rs:553-564 | `EditIntent::Cycle` + `apply_cycle` |
| 8 | JSON / long-text open in a viewer dialog | grid.rs:565-578 → app.rs:457-476 | `EditIntent::External` ⇒ `GridAction::EditRequested` ⇒ app opens the viewer |
| 9 | Engine-aware validation with in-editor error and `!` marker | grid.rs:590-621, 1787-1799 | `GridEditor::commit_cell -> Result<(),String>` |
| 10 | Delete⇒NULL, NOT NULL refusal message | grid.rs:1080-1110 | Adapter handles `RowAction::ClearCell`; message returned as `CellDecor::error` |
| 11 | Insert / duplicate rows with server `DEFAULT` in pk & generated columns | grid.rs:692-746 | Adapter owns; grid emits `RowAddRequested{duplicate: bool}` |
| 12 | Undo (`u`) of the last staged change | grid.rs:748-775, 1153-1156 | Adapter owns the undo stack; `u` bound at app level |
| 13 | Discard all / commit all, with confirm dialogs | grid.rs:1127, 1157; workbench.rs:817-818; app.rs:400-417 | Action bar buttons + app keymap |
| 14 | SQL preview built from pending changes | grid.rs:1158; model.rs:12-117 reads `grid.pending` and `grid.rows()` | `preview_sql(table, columns, &model)` — model is a TablePro type, no library access needed |
| 15 | Commit result folding / per-row error | grid.rs:778-809; app.rs:1134 | Adapter method; `row_errors` surface through `RowDecor` |
| 16 | Foreign-key follow (`Ctrl+]`, trailing `→`, click on `→`) | grid.rs:1159-1169, 1307-1316, 1858-1868; workbench.rs:831-845 | `GridCellActions` supplies the `→` affordance + hot zone; grid emits `CellAction(FollowRef)` |
| 17 | Fetch-more virtual row (`↓ N loaded · Enter fetches more`) | grid.rs:1677-1698, 1049-1051 | Generic; kept |
| 18 | Range selection + `y`/`Y` copy as TSV with headers | grid.rs:870-920, 1119-1126, 2133-2147 | Generic; copy uses `CellRef::text` |
| 19 | Row selection with `✓`, bulk delete of selected rows | grid.rs:1060-1069, 1084-1091, 1725-1738 | Generic selection + `RowRemoveRequested(keys)` |
| 20 | `rows a–b of N` / `cols a–b of N` status line, sort/filter/read-only parts | grid.rs:1389-1430; composed at tabs.rs:795-838 | Generic labels; TablePro keeps its priority-drop composition |
| 21 | Pending bar `• N pending · 2 updates · 1 delete` + row-error detail + Save/Discard/Preview as focus stops | grid.rs:1886-1923, 1926-1947 | Generic **action surface** slot; TablePro supplies buttons, summary text and detail resolver |
| 22 | Pending count aggregated into the identity strip | workbench.rs:546-557 → `grid.pending.total()`; app.rs:2260-2265 | Adapter exposes `pending_total()` — no library involvement |

**[I] Acceptance condition for the boundary:** `rg -n 'sql|SQL|primary key|nullable|foreign|NULL|DEFAULT|commit' src/widgets/grid.rs` returns nothing but generic prose; TablePro's interaction tests (edit/commit/undo/discard/preview/FK-follow/filter-on-cell) pass unchanged.

---

## 2. Disposition per goal §20

Legend for disposition: **P** primitive · **H** headless behavior/state machine · **S** styled component · **C** composition of smaller components · **A** app-specific composition · **L** layered (low-level + convenience API).

### 2.1 Dialog — **L (headless overlay + composed content + convenience constructors)**

**[F] Closed body enum.** `DialogBody::{Text, Input, Facts{facts, code, ack}}` (dialog.rs:18-28) is exactly the "reusable dialog understands only a hard-coded list of body types" that goal §14 forbids. Consequences visible in application code:

- TablePro reuses `Dialog::facts` as a *generic scrollable text viewer* by passing lines as `code` and deleting the Cancel button: `d.actions.remove(0); d.cancel_index = Some(0); d.initial_focus = d.actions[0].id;` (app.rs:471-475 for the cell viewer, app.rs:1178-1181 for the SQL preview, app.rs:1207-1210 for the help dialog).
- The `code` block is truncated at 6 lines with a `… N more` suffix (dialog.rs:429-451) — a hard limit that TablePro's SQL preview must live with; no scrolling.
- `Facts` bodies are unscrollable: `props::render` clips at the action row (dialog.rs:421-427).
- Jackin bypassed `Dialog` entirely for anything richer: `ChoiceDialog` (modals.rs:569-783), `FormDialog` (modals.rs:916-1541), `InfoDialog` (modals.rs:1956-2280), `FileBrowser` (modals.rs:117-563), `HelpOverlay` (modals.rs:2284-2425), plus a `CustomModal` trait (screens/mod.rs:66-95) and `modal_frame` (modals.rs:36-96) — a re-implementation of dialog chrome outside the library.

**[F] Hard-coded bindings:** `y`/`n` quick answers only for `DialogBody::Text` (dialog.rs:297-311); `←`/`→`/`h`/`l` action traversal (dialog.rs:278-296); `Esc` maps to `cancel_index` (dialog.rs:263-269).

**[F] Domain assumption:** the Facts variant's `code: Vec<String>` is documented as "an optional preformatted block (SQL)" (dialog.rs:24).

**[F] Render-time semantic mutation:** dialog.rs:465-470 disables the confirming action from the ack text during `render`.

**[F] Focus plumbing pushed to the caller:** `on_key(&mut self, key, focus: &mut Focus, ring: &FocusRing)` (dialog.rs:207-212) and `on_click(id, pos, focus)` (dialog.rs:324-329) — the dialog mutates the application's focus.

**[I] Recommendation.** Three layers:
1. `Overlay` primitive (**P**): backdrop dim, `begin_modal` barrier, placement/flip/clamp, click-outside policy, Esc policy, focus trap + restore, cursor ownership, hint-layer contribution. Replaces the duplicated dim-and-frame code at dialog.rs:357-390, picker.rs:246-279, app.rs:2337-2376 (TablePro filter editor) and modals.rs:36-96.
2. `Dialog` (**C**): `title`, `body: impl DialogContent` (a small trait: `measure(width) -> u16`, `render(area, …)`, `on_key`, `focusables()`), `actions: Vec<Action>`, `cancel: Option<ActionKey>`. Body content is composed, not enumerated.
3. Convenience constructors on top of the same path (**L**): `Dialog::confirm/destructive/prompt/acknowledge/facts` — implemented as pre-built bodies, not a separate rendering branch.

Composed-content requirements this design must satisfy: scrollable bodies (Jackin's `InfoDialog.detail` + `FormDialog.scroll`, modals.rs:1963/1927), nested overlays (a `Select` popup inside a form, modals.rs:1514-1539), per-action enable predicates (the ack gate), and caller-owned focus order without the dialog touching `Focus`.

### 2.2 Menu — **S (styled) + H (list behavior) — closest to correct today**

**[F]** `ContextMenu` is data-driven (`Vec<MenuItem>`, menu.rs:19-53), does its own placement/flip/clamp (menu.rs:143-171), skips disabled rows (menu.rs:173-186), registers its own frame as a hit target so clicks don't fall through (menu.rs:260), and lets hover drive the cursor (menu.rs:243-248). `MenuBar` composes it (menu.rs:365-603) with hover-to-switch (menu.rs:517-529).

**[F] Gaps:** no submenus; `shortcut: Option<&'static str>` is a display string only (menu.rs:21) with no relation to the actual key handling; bindings `j/k/g/G` are hard-coded (menu.rs:190-205); both consumers dispatch **by label string** — `run_menu(&label, …)` (capsule.rs:368-471) and `run_host_menu(&label)` (app.rs:754-813), the latter *synthesising key events* to reach screen behaviour (app.rs:756, 779-806).

**[I]** Keep the component; give `MenuItem` a typed `ActionKey` payload and an optional `KeyChord` that both renders the hint and registers the binding, so label-string dispatch and key synthesis disappear. Add submenu support (`MenuItem::submenu(Vec<MenuItem>)`) — Jackin's brand menu (capsule.rs:290-301) and tab menu (capsule.rs:303-324) are already anchored sub-popovers built by hand.

### 2.3 Picker / command palette — **L (headless filtered-list machine + styled modal + convenience)**

**[F]** `Picker` owns `query: String` and does its own text input inline (picker.rs:196-218) rather than embedding a field; it renders a fake field with a manual cursor (picker.rs:306-341). Ranking is the owner's job (picker.rs:3). It carries `PickerStatus::{Ready, Loading, Error}` (picker.rs:30-41) — good, generic async states. `cursor_dirty` correctly separates wheel scrolling from cursor-driven scroll-into-view (picker.rs:62-64, 350-354) and is covered by tests (picker.rs:573-620).

**[F] Domain/opinion leakage:** `PickerEvent::Secondary` is documented "(e.g. close tab)" and is bound to `Delete` (picker.rs:76, 195); `NextScope` is bound to `Tab` (picker.rs:194) but the *scope model* lives entirely in the app (TablePro `self.scope` cycling 0..4, app.rs:1240-1248, 1490-1496). `hints: &str` is a raw string render parameter (picker.rs:244, 516-525) that both apps mostly pass empty (app.rs:2181 passes real text; modals.rs:1941 passes `""`).

**[F] Six distinct picker uses in two apps:** TablePro switcher/tab-list/safe-mode (app.rs:1215-1366); Jackin agent, provider, split, close-target, palette, workdir, 1Password chain (capsule.rs:518-562, 598-642, 922-939, 988-1005, 840-849; prelude.rs:206-228; modals.rs:1555-1943).

**[I]** Split into: `FilterList` (**H**: items, query, cursor, scroll, disabled skipping, status) + `PickerOverlay` (**S**) + `CommandPalette` (**L** convenience). Replace `hints: &str` with a `HintLayer` contribution (see 2.8). Make scope a first-class typed concept (`scopes: Vec<ScopeKey>`, `active_scope`) so both apps stop re-implementing it.

### 2.4 Code editor + completion — **C (composition) with a shared editing core**

**[F] `CodeEditor` is already language-agnostic:** `Highlighter = fn(&str) -> Vec<(Range,SyntaxTone)>` and `Segmenter = fn(&str) -> Vec<Range>` (code.rs:26-27) — TablePro supplies SQL implementations (tabs.rs:324-348). Diagnostics (code.rs:35-40), a running-block spinner (code.rs:78, 692-696), and an inline find bar (code.rs:42-48, 289-325) are generic.

**[F] Extension mechanism too narrow:** both hooks are bare `fn` pointers, so a highlighter cannot close over a dialect/catalog. Same class of problem as `grid::Validator`.

**[F] Hard-coded bindings:** navigation-mode keys `i`, `a`, `h/j/k/l`, `g/G`, `{`/`}` block jump, `/` find, `n`/`N` (code.rs:398-481) are vim-flavoured product choices living in the library. `tab_leaves` (code.rs:80) is a per-instance escape hatch for Tab semantics.

**[F] Coordination is manual in the app.** `QueryTab` wires editor↔completion by hand: trigger on `Changed`, re-trigger on `CursorMoved`, close on `Committed` (tabs.rs:1356-1376); accept splices text into the buffer and inserts a closing paren (tabs.rs:1260-1274); anchor is computed from `editor.cursor_cell()` minus the replace length (tabs.rs:1239-1242); wheel and click routing are hand-written (tabs.rs:1438-1446, 1513-1515). `Completion` itself is a clean non-modal anchored list (completion.rs:29-40, 147-241).

**[F] Cross-widget duplication:** `CodeEditor` (code.rs) and `TextArea` (textarea.rs) are two multi-line editors over the same `TextBuffer` with different key tables and different footer conventions.

**[I]** One `TextEditorCore` (**H**) shared by input/textarea/code (buffer, selection, cursor, h-scroll, paste, grapheme handling) + `CodeEditor` (**S**) over it with *trait-object or boxed-closure* `Highlighter`/`Segmenter` + a `Completion` controller (**L**) that owns the editor↔popup contract (`request(cursor, text) -> Option<Vec<Item>>`, accept-splice, dismiss-on-move), so the ~40 lines at tabs.rs:1326-1377 collapse to one call. Keybindings must move into a `KeyMap`/`Command` table so `i`/`a`/`{`/`}` are the *default map*, not the only map (goal §13).

### 2.5 Diff viewer — **C (composition over TextViewport) — already correct in shape**

**[F]** `DiffView` wraps `TextViewport` and only turns hunks into styled lines (diff.rs:191-288); scrolling/selection/copy come from the viewport (diff.rs:1-5). `unified_lines`/`review_lines` are free functions (diff.rs:310-494), and `DiffMode` is a two-variant toggle (diff.rs:167-189). Layout is width-cached and rebuilt only when dirty (diff.rs:241-258). Tests assert render does not undo a wheel (diff.rs:584-586).

**[F] Domain coupling is mild but real:** the `DiffFile`/`DiffHunk`/`DiffLine`/`DiffStatus` model lives in the library (diff.rs:18-165), and Jackin has a **second, parallel** model in `crate::sim::changes::{ChangeSet, ChangedFile, DiffStatus}` (inspect.rs:27) that must be converted.

**[I]** Keep `DiffView` as a composition. Move the *data model* behind a small `DiffSource` trait (or keep the types but make `DiffView` accept `&dyn DiffSource`) so Jackin's `ChangedFile` feeds it without duplication. `review_lines(f, width)` doing layout by width is fine but should become a `measure`/`layout` pass on the component, not a free function taking `u16`.

### 2.6 Panel — **P (primitive) + separate ScrollPanel that should be absorbed**

**[F]** `Panel` is a borrowed, non-stateful chrome primitive returning the inner rect (panel.rs:28-125) with `card`/`framed` kinds, focus flag, `meta`, `badge`, and a `bg_override` escape hatch (panel.rs:35). `Panel::bg(&Theme) -> Color` (panel.rs:69-77) is how every caller learns which background to pass down — **this is the mechanism behind the pervasive `bg: Color` render parameter** that goal §15 wants removed. Typical call: `let bg = panel.bg(t); let inner = panel.render(area, buf, t);` then `child.render(inner, buf, ctx, bg)` (workbench.rs:1247-1265, connections.rs:940-948, tabs.rs:852-863, manager/accounts/usage screens).

**[F]** `ScrollPanel` (panel.rs:177-304) is a second, stateful component in the same module whose `render` takes `style_line: fn(&Theme, &str) -> Style` (panel.rs:263) — another fn-pointer extension point; TablePro passes a DDL-keyword styler (tabs.rs:855-863). It duplicates ~80% of `TextViewport` (scroll, wheel, scrollbar, follow, wrap cache) without selection or spans.

**[I]** `Panel` stays a primitive but the surface must become **contextual**: `ctx.surface(kind)` pushed/popped so children inherit the background instead of receiving `bg: Color`. Delete `ScrollPanel`; express its uses as `TextViewport` with tone-carrying `Span`s (its callers — tabs.rs:855, tabs.rs:1763, tabs.rs:2132 — all just want styled read-only lines).

### 2.7 Status bar — **S (styled) — keep, promote**

**[F]** `StatusBar` is fully generic and well-factored: three groups, priority-based drop order (center → right → left, strongest left item never leaves), truncation of the survivor, inline meters, chip emphasis, click ids (statusbar.rs:112-305), with tests for layout and narrow behaviour (statusbar.rs:344-410).

**[F] Only one consumer.** Jackin's Capsule uses it (capsule.rs:1671-1804). TablePro instead uses `segments::render` for its identity strip (app.rs:2189-2282) and hand-rolls its grid status line with a bespoke priority-drop loop (tabs.rs:794-838). Jackin's host chrome also uses `segments` (app.rs:838-880, capsule.rs:1629-1666).

**[I]** `StatusBar` and `segments` are the same concept at two fidelities. Unify: one priority-ordered item strip with `Left/Center/Right` groups, and make the TablePro grid status line (tabs.rs:794-838) a third consumer. That deletes two hand-written priority-drop loops.

### 2.8 Hint bar — **P (layer resolver) — correct concept, under-integrated**

**[F]** `HintBar::resolve(&[Option<HintLayer>])` picks the topmost present layer (hintbar.rs:50-52); `HintLayer` carries hints, a badge, a status and centering (hintbar.rs:14-43); overflow drops from the right with a `…` marker (tested at hintbar.rs:86-113).

**[F] Neither application uses the layer model as designed.** TablePro builds a `Vec<Hint>` by hand per modal kind and per screen and calls `keyhint::render` directly (app.rs:2284-2335), never touching `HintBar`/`HintLayer`. Jackin imports `HintBar`/`HintLayer` (app.rs:13) but every hint set is produced by a `Screen::hints(focus, world)` method returning `Vec<Hint>` (screens/mod.rs:306) and by per-modal `hints()` methods (modals.rs:1918-1938, capsule.rs:2461-2520, prelude.rs:497-524, inspect.rs, etc.). The result is ~700 lines of hand-written hint tables across both apps, and hints are keyed off `focus: Option<WidgetId>` with long `match` ladders (e.g. tabs.rs:717-754, tabs.rs:1276-1324, workbench.rs:568-600, connections.rs:889-917).

**[I]** This is the single clearest win for goal §13's "contextual action or hint metadata". Components should *declare* their bindings (`fn hints(&self, state) -> HintLayer` derived from the same `KeyMap` that handles the keys), the shell composes layers (overlay ▸ mode ▸ focused component ▸ screen ▸ global), and screens contribute only genuinely product-level hints. Correctness follows for free: today `capsule.rs:2478-2492` lists prefix commands that must be kept in sync by hand with `handle_prefix_cmd` (capsule.rs:1252-1323).

### 2.9 Tab workspace — **split: `Tabs` = S primitive; the "workspace" = A**

**[F] `Tabs` widget** (tabs.rs:53-492) is generic: items with `dirty/busy/error/closable/prefix/suffix` (tabs.rs:19-29), strip scrolling with `‹N`/`N›`, `+` new-tab affordance, close hot zones, active underline (accent, or `border_strong` when `quiet`), one focus stop.

**[F] Identity is positional.** `tab_id(i) = id.child(i)` and `close_id(i) = id.child(i).sub("close")` (tabs.rs:106-111). `remove(i)` shifts `active` by index (tabs.rs:152-163). Both apps therefore **rebuild the whole `Tabs` every frame or on every change** to keep it in sync, discarding and restoring internal state by hand:

- TablePro: `self.strip = Tabs::with_items(TABSTRIP, items); self.strip.allow_new = true; self.strip.first = first; self.strip.set_active(active);` (workbench.rs:406-410); the same for result tabs (tabs.rs:1176-1193).
- Jackin Capsule: rebuilt **inside `render`** every frame — `let first = self.tabs.first; self.tabs = Tabs::with_items(STRIP, items); self.tabs.first = first; self.tabs.set_active(d.active);` (capsule.rs:1459-1462).

This is exactly the goal §23 Scenario E hazard: focus, close actions and pending edits are associated with a numeric position, and the "first visible tab" must be manually rescued across rebuilds.

**[F] Workspace-level behaviour lives entirely in the apps:** tab ordering, dirty-close confirmation, preview-tab reuse, duplicate-label disambiguation (workbench.rs:369-426, 428-468, 501-514, 959-966); result-tab pinning and reordering (tabs.rs:1204-1227).

**[I]** `Tabs` gains **stable keys** (`TabItem<K>` / `key: TabKey`) so ids, active tab, strip window and close actions follow the logical item; add `set_items` that reconciles instead of forcing reconstruction. The "tab workspace" (dirty-close policy, preview tabs, pinning, per-tab bodies) stays application-specific composition — correctly so.

### 2.10 Splitter / Split — **P (primitive) — correct, but drag geometry is caller-held**

**[F]** `Splitter` is a 63-line mouse affordance over `ui::layout::Split` (splitter.rs:16-63) with hover/pressed glyph weight so the affordance survives monochrome. Consumers must remember the container rect to drive `on_drag`: `self.seam.on_drag(&mut self.split, self.container, 1, pos)` (inspect.rs:230-233), with `seam_container: Rect` fields kept by hand in manager.rs:95 and accounts.rs:98.

**[F]** Keyboard resize is *not* in the component ("keyboard resize stays a chord on the owning pane", splitter.rs:2-3); Jackin implements `Alt+Shift+↑↓←→` itself over a pane tree (capsule.rs:765-823), and TablePro implements `Ctrl+↑/↓` over `Split::grow` (app.rs:759-766).

**[I]** Fold `Split` + `Splitter` into one `SplitPane` component that owns the container rect from its own render, exposes pointer-capture drag and an optional keyboard resize command, and enforces minimum pane sizes (today Jackin re-checks minimums by hand: capsule.rs:654-694, 804-821).

### 2.11 Viewport (`TextViewport`) — **P (primitive) — the strongest component in the set**

**[F]** Styled spans, wrap, bounded retention with selection/caret index fixup (viewport.rs:165-181), tail-follow, wheel/scrollbar, drag selection with edge auto-scroll (viewport.rs:490-511), word selection (viewport.rs:437-474), grapheme/width-correct cell model (viewport.rs:287-367), copy event, optional hardware caret (viewport.rs:653-670). Tests cover follow, drag-copy, wrap+retention (viewport.rs:698-743).

**[F] One design smell:** `set_area()` exists solely because Jackin renders a *clone* of the pane's viewport, so the original never learns its geometry — "owners that render a copy of the viewport call this before routing mouse or key events" (viewport.rs:236-252), used at capsule.rs:1408-1416 (`prime`) and paired with an `ctx.inert = true` window around the clone's render plus a manual cursor re-placement (capsule.rs:1567-1591).

**[I]** Keep as a primitive. The clone-and-prime workaround is a symptom of the state-ownership model, not of the viewport: with caller-owned view state (`&mut ViewportState` passed to render) Jackin can render directly from the daemon's pane and `set_area`, `prime` and the `inert` dance all disappear.

### 2.12 Table (`DataTable`) — **decompose; merge with the generic grid**

**[F]** `DataTable` (table.rs:96-119) duplicates most of `DataGrid` at lower fidelity: its own `Column` (table.rs:27-33) with `Constraint` widths, its own `Cell {text, error, tone}` (table.rs:63-67), its own `EditState` (table.rs:86-91) — a *third* copy of that struct after grid.rs:233 — its own sort with a numeric-column flag (table.rs:216-259), its own cell-nav/row-nav mode switch (table.rs:106), its own `validator: Option<fn(usize,&str)->Option<String>>` (table.rs:112), and its own `locate`/`owns` (table.rs:797-820).

**[F] Only one consumer**: TablePro's Structure tab, rebuilt wholesale per section (tabs.rs:536-655, `DataTable::new(...)` at tabs.rs:655), routed at workbench.rs:1049-1058 and workbench.rs:723-725.

**[F] Divergences that would surprise a user:** `DataGrid` sorts by *cell value* with NULLs last (grid.rs:1950-1966), `DataTable` sorts by *string* with an opt-in numeric parse (table.rs:216-234, 830-840); `DataGrid` renders row-hover underline only for editable cells, `DataTable` differs (table.rs:759-764); `DataGrid` emits `(Outcome, Option<GridEvent>)`, `DataTable` emits `(Outcome, Option<TableEvent>)` with a different vocabulary (`Committed/Cancelled/Activated/Leave*`, table.rs:122-132).

**[I]** Delete `DataTable`. The generic `DataGrid` after the §1.5 split covers it: a read-only-by-default grid with an owned string model, `cell_nav` becoming a `NavUnit::{Row, Cell}` option. TablePro's Structure tab becomes six `GridModel`s over catalog data.

### 2.13 Cross-cutting API-inconsistency findings (goal §7 matrix, complex components only)

| Inconsistency | Instances |
|---|---|
| Event return shape | `(Outcome, Option<Ev>)`: grid.rs:922, table.rs:338, tabs.rs:173, menu.rs:188, picker.rs:147, completion.rs:94, chips.rs:85, code.rs:289, select.rs:72, viewport.rs:537, input.rs:188, textarea.rs:91 · bare `Outcome`: list.rs:118, choice.rs:32/116/234, panel.rs:209 · `Outcome` + `&mut Focus`: dialog.rs:207 · `Option<Ev>` only: menu.rs:219, completion.rs:136, picker.rs:223 |
| Render signature | `(area, buf, ctx, bg: Color)`: grid, table, code, tabs, chips, select, choice, textarea, input, viewport, splitter, menu-bar · `(area, buf, ctx)` (own surface): dialog.rs:357, picker.rs:244 (+ `hints: &str`), menu.rs:235, completion.rs:147, statusbar.rs:264 · `(area, buf, &Theme, …)` no ctx: panel.rs:80, props.rs:51, hintbar.rs:56 |
| `owns`/`locate` pairs the app must chain | grid.rs:1208/1220/1232/1235, table.rs:797/811/818, tabs.rs:122/126, menu.rs:118/122, picker.rs:120/123, completion.rs:86/90, chips.rs:141, select.rs:65/68, props.rs:129/132, viewport.rs:532, diff.rs:260 |
| Caller-supplied fn-pointer extension | grid.rs:265 (`Validator`), table.rs:112, code.rs:26-27, panel.rs:263 (`style_line`), input.rs:36 |
| Public frame-local geometry | `pub area` on grid.rs:355, table.rs:114, tabs.rs:60 (`areas`), menu.rs:79, picker.rs:57, completion.rs:39, chips.rs:42, dialog.rs:52, select.rs:24, input.rs:33, viewport.rs:119, choice.rs:96 |
| Render performs a semantic transition | grid.rs:1518, table.rs:566, code.rs:611, input.rs:282, textarea.rs:202, select.rs:165, dialog.rs:465 |
| Hard-coded product bindings in library code | grid.rs:1111-1169 (`+`, `-`, `p`, `u`, `U`, `Ctrl+]`, `Ctrl+S`), code.rs:398-481 (`i`, `a`, `{`, `}`, `/`, `n`, `N`), picker.rs:194-195 (`Tab` scope, `Delete` secondary), viewport.rs:540-565 (`f` follow, `y` copy), menu.rs:190-205, tabs.rs:191-211 (digits, `x`, `n`), panel.rs:222 (`f` follow) |

---

## 3. Forms and text editing (goal §19)

### 3.1 Current transitions, per control

| Control | begin | commit | cancel | focus-loss | render-time side effect |
|---|---|---|---|---|---|
| `TextInput` | `begin_edit()` snapshots the value (input.rs:152-159); entered by Enter/F2 (input.rs:193-197) or click when already focused (input.rs:247-261) | `commit()` clears selection **and runs `validate()`** (input.rs:161-165); emits `Committed` / `CommittedTab{backward}` (input.rs:200-214) | `cancel()` restores the snapshot **and validates** (input.rs:167-172) | **implicit commit inside `render`** (input.rs:282-286) | yes — commit + validate |
| `TextArea` | `begin_edit()` (textarea.rs:80-84) | `commit()` — clears selection only, no validation (textarea.rs:86-89); Esc **commits** (textarea.rs:121-124) | none (Esc is commit) | implicit commit in `render` (textarea.rs:202-205) | yes |
| `CodeEditor` | `begin_edit()` (code.rs:212-216); nav-mode `i`/`a`/Enter (code.rs:400-408) | `commit()` = leave editing, clear selection (code.rs:223-226); Esc commits (code.rs:330-333) | none | implicit commit in `render` (code.rs:611-614) | yes |
| Grid cell | `begin_edit()` (grid.rs:539-588) | `commit_edit()` validates then `record_cell` (grid.rs:590-621) | `cancel_edit()` (grid.rs:623-625) | **commit inside `render`** (grid.rs:1518-1520) | yes — stages a DB mutation |
| Table cell | `begin_edit()` (table.rs:285-302) | `commit_edit()` writes into `self.rows`, re-sorts if the sorted column changed (table.rs:304-332) | `cancel_edit()` (table.rs:334-336) | commit inside `render` (table.rs:566-568) | yes |
| `Select` | `open = true` on Enter/Space/click (select.rs:104-108, 129-133) | Enter/Space closes and emits `Changed` (select.rs:86-94); arrows change the value **without opening** (select.rs:109-120) | Esc closes and restores `cursor = selected` (select.rs:95-99) | **`render` closes the popup** when unfocused (select.rs:165-167) and when disabled (select.rs:161-164) | yes — closes an overlay |
| `Checkbox`/`Toggle` | n/a | Space/Enter/click toggles immediately (choice.rs:32-50, 234-252) | n/a | n/a | no |
| `RadioGroup` | n/a | arrows **change the selection while moving** (choice.rs:121-130) — cursor and value are fused | n/a | n/a | no |
| `ChipBar` | n/a | Enter⇒`Activate`, Space⇒`Toggle`, Delete/Backspace/`x`⇒`Remove` (chips.rs:100-118) — all *requests*, the owner mutates | n/a | n/a | no |

### 3.2 Validator extension mechanisms (three incompatible shapes)

**[F]** `TextInput::validator: Option<fn(&str)->Option<String>>` (input.rs:36, 105-108) — returns an error message; `validate()` also enforces `required` when no validator is set (input.rs:174-181). `DataTable::validator: Option<fn(usize,&str)->Option<String>>` (table.rs:112, 172-175) — column-indexed. `DataGrid::validator: Validator = fn(&ColumnSpec,&str)->Result<CellValue,String>` (grid.rs:265, 353) — parses *and* validates. `TextArea` has **no** validator and only a caller-set `error: Option<String>` (textarea.rs:26, 67-70).

**[F]** Application-domain validation therefore lives outside the field: TablePro's `port_validator`/`name_validator` are free fns (connections.rs:89-105) and form-wide validation is a manual loop (connections.rs:246-250, 689-712). Jackin's Prelude validates in the screen and re-opens the dialog with an error injected into a fresh `TextInput` (`input.error = error;` prelude.rs:193, 240; validators at prelude.rs:319-348). Jackin's `FormDialog` has no validation at all — it emits `FormEvent::Changed(name)` and the owner re-renders `Note` rows (modals.rs:1031-1037, 906-914).

### 3.3 Masked / secret handling

**[F]** `TextInput.masked` + `reveal_tail: u8` (input.rs:41-45); `display_graphemes()` masks per grapheme and reveals the tail only when *not* editing (input.rs:127-146); `clear()` overwrites and drops the value with a comment that owners clear secrets this way (input.rs:118-123).

**[F] Exposure risks (goal §10, §29):**
- `TextInput` derives `Debug` (input.rs:18) with `pub buffer: TextBuffer` (input.rs:23) — `{:?}` prints the raw secret regardless of `masked`.
- `Dialog` derives `Debug` (dialog.rs:43) and may contain a `TextInput` body or an `AckInput` (dialog.rs:30-34) — same exposure through the dialog.
- `TextInput` derives `Clone`, so snapshots/copies duplicate secrets (input.rs:18).
- Jackin's `FormDialog` holds password/API-key fields as `FieldKindW::Input(TextInput)` (modals.rs:797) and `values()` returns raw `FieldValue::Text` (modals.rs:1039-1044), which flows through `ModalResult::Form(Some(values))` (app.rs:1472-1476) into screen code (accounts.rs handles `CredentialSource`).
- Jackin keeps a separate `unmasked: HashSet<(Option<RoleName>,String)>` reveal set and a domain `mask` fn (config.rs:202, 37) — a second masking implementation.

**[I]** Required: a `Secret` newtype whose `Debug`/`Display` redact, no `Clone` for the raw value (or a `expose()` method with an explicit name), manual `Debug` impls on `TextInput`/`Dialog`/`FormDialog`, and a conformance test asserting `format!("{:?}", field)` never contains the value.

### 3.4 Recommended field/editor model

**[I]**

1. **`Field<C>` wrapper owns the chrome.** Label (+ `*` required / `optional` suffix, today re-implemented per control at input.rs:289-327, textarea.rs:208-213, select.rs:168-173, choice.rs:160-165), help/error row (input.rs:427-444, textarea.rs:308-343, select.rs:200-207), gutter bar, focus ring registration, and `HEIGHT` (input.rs:184, select.rs:33, textarea.rs:76) become one component. Controls become bare editors. This alone removes the `plain_label` flag (input.rs:38, used at app.rs:1408-1412, workbench.rs:105-107, connections.rs:267-269, prelude.rs:189, modals.rs:148) and the `Select::HEIGHT`/`TextInput::HEIGHT` arithmetic scattered through connections.rs:1144-1179, app.rs:2388-2437 and modals.rs:871-880.

2. **Explicit edit lifecycle, never render-driven.** `begin_edit() → {commit(), cancel(), blur(policy)}` where `EditLifecycle::on_blur` is a typed policy (`CommitAndValidate` for TextInput, `Commit` for TextArea/CodeEditor, `Cancel`, `Keep`). The shell calls `blur()` when focus changes; `render` becomes pure. Test: render twice with focus absent and assert no value/pending change (currently would fail for grid, table, input, textarea, code, select).

3. **Controlled values.** `Field::value(&T)` + `on_change` action, with the component keeping only *transient* edit state (draft buffer, cursor, selection). This removes the "rebuild the widget to change its value" idiom: `form.port = TextInput::new(form.port.id, "Port").value(port).validator(port_validator)` (connections.rs:629-631), `f.op = Select::new(f.op.id, "Operator", &labels, 0)` (app.rs:1683-1684, 2063-2064), `FormDialog::set_text` reconstructing the input (modals.rs:1007-1011), `w.explorer_filter = TextInput::new(...)` (app.rs:802-804, 1560-1562).

4. **Validation as a hook, not a fn pointer.** `fn validate(&self, value: &str) -> Result<(), FieldError>` on a small trait (or `Box<dyn Fn>`), plus caller-set `error` for server-side/async results. One vocabulary across input, textarea, grid cell and select.

5. **`RadioGroup` must separate cursor from value** (choice.rs:121-130 currently selects while moving) to match `ListBox`/`Tabs`/`Picker`, all of which have a cursor distinct from the selection.

6. **Chips** stay request-emitting (correct today) but should share the collection vocabulary (cursor, activation, remove) rather than a bespoke `stops()` model (chips.rs:81-83).

---

## 4. Jackin complex surfaces: reusable vs Jackin-specific

### 4.1 Genuinely reusable primitives currently missing from the library

| # | Missing primitive | Evidence in Jackin | Also needed by | Justification |
|---|---|---|---|---|
| J1 | **Overlay/modal frame** (dim, barrier, rounded frame, title + right-aligned meta, hint row) | `modal_frame` (modals.rs:36-96), `hint_row` (modals.rs:98-106) | TablePro filter editor duplicates it (app.rs:2337-2376); `Dialog` (dialog.rs:357-390) and `Picker` (picker.rs:246-279) duplicate it again | Four copies of the same 40 lines; goal §14 requires one overlay model |
| J2 | **Form dialog / field group** (ordered fields, visibility toggling, scroll with focused-field reveal, action buttons, error row, nested popup) | `FormDialog` (modals.rs:916-1541); focused-field scroll-into-view at modals.rs:1356-1371; nested open-`Select` re-render at modals.rs:1514-1539 | TablePro's connection form (connections.rs:62-251, 1120-1256) and filter editor (app.rs:99-109, 2337-2468) are hand-built equivalents | Three independent form engines in one repo |
| J3 | **Choice dialog** (question + radio + buttons + per-option tone) | `ChoiceDialog` (modals.rs:569-783) | Would replace `DialogBody::Text` + `y/n` hacks | Composed dialog content, goal §14 |
| J4 | **Info/facts dialog with copyable rows** | `InfoDialog` (modals.rs:1956-2280) built on `PropsList` + a scrollable detail block | TablePro fakes this with `Dialog::facts` + button surgery (app.rs:467-476, 1167-1182) | Both apps need "read-only facts + copy + actions" |
| J5 | **Key-reference overlay** (multi-column, scrollable, scope label) | `HelpOverlay` (modals.rs:2284-2425) | TablePro renders help as a `Dialog::confirm` with a `\n`-joined string (app.rs:1196-1212) | Same product need, wildly different quality |
| J6 | **File/path browser** | `FileBrowser` (modals.rs:117-563) | — | Arguably app-specific (its `FsEntry` is simulated), but the *pattern* (path field + list + mode toggle + confirm) is a reusable "browse-and-choose" composition |
| J7 | **Multi-step / wizard controller** (step order, rewind, per-step state retention, stepper line) | `PreludeScreen` (prelude.rs:65-369); stepper text at prelude.rs:97-128; rewind at prelude.rs:298-317 | `StepRail` exists (steps.rs) but is a *display* rail only | The library has step *presentation* and no step *flow*; `ChoiceDialog::stepper(&str)` (modals.rs:617-620) is a string patch over the gap |
| J8 | **Async/staged picker chain** (loading state, error state with retry, breadcrumb scope, back-one-step) | `OpFlow` (modals.rs:1555-1943): `PickerStatus::Loading/Error` + `crumb()` + `back()` | TablePro's switcher has scopes but no async states | `PickerStatus` already exists (picker.rs:30-41); the *chain controller* does not |
| J9 | **Keyed list/tree with custom row rendering** | `ManagerScreen::Row {key: RowKey, depth, glyph, glyph_tone, label, meta, meta_tone, trailing, expandable}` (manager.rs:51-62) and `build_rows` (manager.rs:142-…); `AccountsScreen::Row` (accounts.rs:65-77); `ConfigTabs::Row {key, change, cells, problem, faint, header, folded, meta}` (config.rs:161-172); `UsageScreen::Row` (usage.rs:31-36); Editor's `AcctRow`/`RoleRow` (editor.rs:80-99) | TablePro's `TableTab` status parts and `HistoryTab` row annotation (tabs.rs:2429-2442, which pokes glyphs into the buffer *after* `ListBox::render`) | **Six** hand-rolled row models in Jackin alone, all with a stable key, a depth, a glyph, a meta column and an expand flag. `ListBox` (list.rs:13-17: label + meta + disabled) and `TreeView` (tree.rs:14-27) are too poor to serve them |
| J10 | **Change-slot row decoration** (`+ • − !` per row, "N changes" rollup, per-row undo) | `Change` enum + glyph (config.rs:128-145), rows carry `change` (config.rs:163) | `DataGrid::RowState` is the same idea (grid.rs:224-231, 1739-1751) | Two implementations of "pending change decoration"; should be one generic `RowDecor` |
| J11 | **Tab strip with stable keys** | Capsule rebuilds `Tabs` inside `render` (capsule.rs:1459-1462); Editor/Settings rebuild per tab-set | TablePro workbench.rs:406-410, tabs.rs:1176-1193 | See §2.9 |
| J12 | **Meter** — already in the library (`progress::Meter`) but the *tone mapping* is duplicated | `Self::meter_tone` (capsule.rs:2568-2577) and the same match inlined at capsule.rs:1756-1764, plus accounts.rs / usage.rs | — | Small: expose `MeterTone::from_ratio_and_freshness`-style helpers or keep tone mapping in the app consistently (currently both) |
| J13 | **Modal stack + result routing** | `Modal` enum (screens/mod.rs:98-108), `ModalTag` (screens/mod.rs:41-63), `ModalResult` (screens/mod.rs:111-131), `CustomModal` trait (screens/mod.rs:66-95), push/pop with focus save/restore (app.rs:1229-1266), `deliver` (app.rs:1268-1281) | TablePro re-implements a one-deep version: `Modal` enum (app.rs:112-116), `open_dialog`/`close_modal` with `saved_focus` (app.rs:1187-1193, 1435-1438), `dialog_closed(id, result, value)` (app.rs:1788-1854) | The **strongest evidence** that overlay stacking, focus trapping and focus restoration belong in the library (goal §14, §29) |

### 4.2 Genuinely Jackin-specific compositions (keep in the app)

- **Capsule pane tree + PTY simulation**: mode priority `Dialog › Drag › Select › Prefix › Normal` (capsule.rs:56-63, 158-170), `Ctrl+B` prefix with a 2 s timeout (capsule.rs:54, 505-507, 1252-1323), pane layout/seams from `sim::pty` (capsule.rs:194-208), split-refusal by minimum pane size (capsule.rs:654-694), zoom (capsule.rs:825-838), takeover screen (capsule.rs:2400-2428). Product behaviour; compose from `SplitPane` + `TextViewport` + `Tabs` + `MenuBar`.
- **Launch cockpit**: stage rail semantics, rain/atmosphere, credential-origin projection (cockpit.rs:40-138, 147-…). `StepRail` already carries the generic part.
- **Account/usage domain projections**: quota windows, freshness, validation outcomes (accounts.rs, usage.rs) — domain.
- **`Doc`/`ConfigTabs` original-vs-pending editing model** (config.rs:64-126, 194-212): the *diff-against-original* model is domain; the *row decoration* and *list* are J9/J10.
- **1Password flow semantics** (modals.rs:1555-1943): the chain controller is J8; the account/vault/item/field model is domain.
- **Rain / intro / outro** (`rain.rs`): explicitly allowed to stay app-specific by goal §22.3, provided it consumes semantic theme APIs.

### 4.3 Manual plumbing that must disappear (goal §2.9, §12)

**[F]** Representative counts of the mechanics goal §12 wants replaced by dispatch:

- Explicit `focus.focus(...)` inside click handlers: modals.rs:404, 409, 422, 428, 702, 708, 1240, 1245, 1254, 1262, 1271, 1281, 1290, 1297; capsule.rs:2009, 2013, 2030; accounts.rs/manager.rs/editor.rs throughout; TablePro connections.rs:746, 750, 798, 812, 824, 830; workbench.rs:977, 981, 1012, 1038, 1042, 1050, 1063, 1086, 1104, 1112; tabs.rs:1448, 1458, 1472, 1481.
- `owns`/`locate` ladders in screens: workbench.rs:980-1093 (nine consecutive `owns`/`locate` branches), tabs.rs:1437-1494, connections.rs:744-791, capsule.rs:1957-2024.
- Re-registering hits "on top of the surface" after drawing a modal: dialog.rs:478-487, picker.rs:279, modals.rs:551-557, 1491-1512, 2270-2278, app.rs:2462-2467 (TablePro filter editor).
- Manual `Tab`/`BackTab` handling inside modal components: dialog.rs:270-277, modals.rs:382-389, 675-682, 1084-1091, 2102-2109.
- Manual double-click detection in the shell (app.rs:1636-1640) and manual pressed/release drag bookkeeping (app.rs:1550-1571, 1602-1624).

---

## 5. TablePro surfaces

### 5.1 Connections screen (`connections.rs`)

**[F]** A 15-field form built from raw controls with a hand-written key router: `on_form_key` is a 115-line `if f == form.X.id` ladder with an `input!` macro to unify the Tab-commit dance (connections.rs:573-687); `on_form_click` is its click twin (connections.rs:793-861); `on_paste` a third traversal (connections.rs:863-887). Layout is manual arithmetic over `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` (connections.rs:1144-1196). Validation is `a && b` over two fields with manual focus-to-first-error (connections.rs:246-250, 704-711). Password field is a plain `TextInput` with a placeholder (connections.rs:155-157) — **not** `.masked()`.

**[I]** Becomes `FormDialog`-equivalent (J2) with a `Field` wrapper: fields declared as data, one router, automatic Tab order, automatic validation pass, `Secret` for the password. Expected reduction: ~300 lines of routing/layout.

**[F]** The connection list is a `TreeView` rebuilt on every mutation with cursor/filter rescued by hand (connections.rs:283-317). `ConnState` simulation (Idle/Connecting/Failed/Testing/Tested, connections.rs:26-42) and the detail card (connections.rs:975-1117) are app composition — correct.

### 5.2 Workbench (`workbench.rs`)

**[F]** Explorer is a `TreeView` with lazy loading (`TreeNode::lazy`, workbench.rs:143-146, 184) and simulated latency (`pending_loads: Vec<(Vec<usize>, u32)>`, workbench.rs:79, 332-357) — a good use of the library. Path→object resolution is positional and fragile: `object_at(path)` reconstructs the section order to map `path[2]` back to an `ObjectKind` (workbench.rs:305-322).

**[F]** Explorer visibility is a responsive drawer implemented in the screen (`narrow = area.width < 100`, workbench.rs:1223-1243) including a "still a focus stop when hidden" hack: `ctx.control(self.explorer.id, Rect::ZERO, false)` (workbench.rs:1268) and `ctx.control(pf, Rect::ZERO, false)` (workbench.rs:1275).

**[I]** `TreeNode` needs a **caller key** (`TreeNode::keyed(K)`) so `object_at`/`schema_at` disappear. Responsive collapse and "zero-rect focus stop" indicate the library needs a `Drawer`/`Collapsible` region primitive and a supported way to keep a control in the focus ring while hidden.

### 5.3 Workbench tabs (`tabs.rs`)

**[F] `ResultBody`** (tabs.rs:898-918) is a closed enum of five result presentations (`Rows(DataGrid)`, `Affected`, `Error`, `Plan{tree,raw,show_raw,…}`, `Cancelled`) with a 250-line render `match` (tabs.rs:1660-1931). The `Plan` arm **draws metric columns over the tree's own rows after `TreeView::render`**, reading `buf[(cols_x, y)].bg` to recover the row background and overwriting the tree's meta column (tabs.rs:1804-1852).

**[I]** That is the clearest demand in TablePro for a **custom row renderer** on collections (goal §18, §23-D): `TreeView` must accept a per-row cell/columns renderer so the app stops painting over it. Same for `HistoryTab`, which pokes a `!` glyph into `ListBox`'s buffer after render (tabs.rs:2434-2442).

**[F]** `QueryTab` is otherwise a legitimate app composition (editor + completion + result tabs + split + status line, tabs.rs:951-1937) — but see §2.4 for the editor/completion coordination that should be library-owned.

### 5.4 Results grid

Covered in §1. Note additionally: result grids are constructed with `local_sort = true` and a read-only reason when the source table is unknown (tabs.rs:2062-2068) — both survive the split unchanged.

### 5.5 Dialogs

**[F]** Every non-confirm dialog in TablePro is a `Dialog::facts` with post-construction surgery (app.rs:467-476, 1167-1182, 1206-1212). Safety dialogs are genuinely domain-rich and correctly app-owned: risk facts, SQL code block, typed acknowledgement token, danger vs primary confirm button (app.rs:882-984 for query execution, app.rs:987-1111 for commit). `Dialog::facts`'s ack mechanism (dialog.rs:110-136, 139-144) is the only library support and it is hard-wired to "last action is the confirm" (dialog.rs:465-470).

**[I]** With composed dialog content (§2.1), the safety dialog becomes `Dialog::new(title).body(Facts::new(props).code(sql_block).ack(token)).actions([...])` where `ack` produces an `enabled_when` predicate rather than mutating a button during render.

### 5.6 Menus

**[F]** TablePro has **no** menu bar; discoverability rests on the identity strip's clickable segments (`STRIP_SAFE`, `STRIP_SCOPE`, `STRIP_CONN`, `STRIP_HELP`, app.rs:37-40, 1971-1990) and a text help dialog. Jackin has a full `MenuBar` on every screen (app.rs:699-743, capsule.rs:228-287).

**[I]** Not a defect, but it means `MenuBar` currently has one consumer; the showcase must exercise it independently.

### 5.7 Pickers

**[F]** Three pickers with per-kind hint strings selected in `draw` (app.rs:2169-2181), scope cycling implemented as `self.scope = (self.scope + 1) % 4` plus a rebuild (app.rs:1490-1496, 1236-1279), and a parallel `switcher_targets: Vec<SwitchTarget>` vector aligned by index to the picker rows (app.rs:141, 1278, 1540) — a classic index-coupling hazard. The tab-list picker smuggles the tab index through `detail: i.to_string()` and parses it back (app.rs:1294, 1505-1508, 1608-1613).

**[I]** `PickerItem` needs an opaque caller key (`PickerItem<K>` or `key: ItemKey`) so parallel vectors and stringly-typed payloads disappear. This is the same fix as J9/§2.9 (stable keys across collections).

---

## 6. Collection concepts matrix

Facts as implemented today; the rightmost column is the recommendation.

| Concept | Data ownership | Key model | Cursor vs selection | Multi-select | Virtualization | Empty / loading / error | Custom rendering hook | Activation / child actions | Reorder | Focus reconciliation | **[I] Unify / preserve** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **ListBox** (list.rs) | Owned `Vec<ListItem>` (label, meta, disabled) — list.rs:13-17, 47 | Positional; `row_id(i)=id.child(i)` (list.rs:82) | Separate: `cursor` + `chosen`/`checked` (list.rs:49-52) | Yes (`SelectMode::Multi`, `checked`, shift-range via `anchor`, list.rs:90-105) | Yes (`ScrollState::visible_range`) | `empty_text` only (list.rs:55); no loading/error | **None** — consumers paint over the buffer (tabs.rs:2434-2442) | `activate(i)` (list.rs:107-116) | No | Cursor clamped on `set_items`-equivalent only by caller | Unify: keys, custom row renderer, `EmptyState` incl. loading/error |
| **TreeView** (tree.rs) | Owned `Vec<TreeNode>`; flattened to `Vec<FlatRow>` (tree.rs:107-118, 145+) | **Path** `Vec<usize>` (tree.rs:83) — positional but hierarchical | `cursor: usize` (row) + `selected: Option<Path>` (tree.rs:111-112) | No | Yes | No empty state; `busy` per node (tree.rs:25) | **None** (glyph + meta only) | `TreeEvent::{Expand, Activate}` (tree.rs:99-104) | No | `flatten()` recomputes rows; cursor clamped by caller (connections.rs:315) | Preserve hierarchy + lazy loading; add caller keys + row renderer |
| **DataTable** (table.rs) | Owned `Vec<Vec<Cell>>` (table.rs:99) | Positional + `order` permutation (table.rs:102) | `cursor_row`/`cursor_col` + `selected: Option<usize>` (table.rs:104-108) | No | Yes, both axes (`scroll`, `hscroll`) | `empty_text` only (table.rs:152) | **None** (`Cell.tone` only) | `TableEvent::Activated` (table.rs:126) | Sort only | `set_rows` clamps cursor (table.rs:213) | **Delete**; fold into grid |
| **DataGrid** (grid.rs) | Owned `Vec<Vec<CellValue>>` (grid.rs:330) + `order` (grid.rs:331) | Source row index as the key ("sorting never invalidates them", grid.rs:184-185) | Cursor cell + `selected_rows` + rectangular `anchor` (grid.rs:338-341) | Yes (rows) + **range** (rectangular) | Yes, both axes | `empty: EmptyState` + `loading` spinner (grid.rs:352, 1648-1660); per-cell/row errors | **None** — `CellKind` decides | `Activated`, `FetchMore`, FK-follow, viewer | Sort only | `set_rows` clamps cursor + `ensure_visible` (grid.rs:450-458) | Keep as the *one* tabular component; add model trait + renderers |
| **Tabs** (tabs.rs) | Owned `Vec<TabItem>` (tabs.rs:56) | **Positional** `id.child(i)` (tabs.rs:106-111) | `cursor` + `active` (tabs.rs:58-59) — correctly separate | No | Strip window (`first`, `fit`, tabs.rs:61, 67) | None | None (fixed prefix/suffix/dirty/busy/error slots) | `TabEvent::{Activated, Close, New}` (tabs.rs:71-75) | No (apps rebuild) | `remove(i)` shifts `active` (tabs.rs:152-163); apps rebuild wholesale | **Stable keys required** (Scenario E) |
| **PropsList** (props.rs) | Owned `Vec<Prop>` (props.rs:101) | Positional (props.rs:126) | `cursor` only | No | Yes | None | `Prop.tone`/`wrap`/`copyable` only (props.rs:17-24) | `PropsEvent::{Copy, Activate}` (props.rs:92-95) | No | `set_props` clamps cursor (props.rs:120-124) | Merge vocabulary with ListBox (it *is* a two-column list) |
| **StepRail** (steps.rs) | Owned `Vec<Step>` (steps.rs:69) | Positional | `cursor` when `selectable` (steps.rs:70-71) | No | Yes | `StepState` includes `Blocked`/`Failed` (steps.rs:18-26) — the closest thing to per-item error | None | None (display rail) | No | n/a | Preserve the difference (ordered lifecycle, not a selection list) |
| **Picker** (picker.rs) | Owned `Vec<PickerItem>` (picker.rs:50) | Positional; apps keep parallel key vectors (app.rs:1278, capsule.rs:549) | `cursor` only; choosing closes | No | Yes; `cursor_dirty` separates wheel from cursor (picker.rs:62-64) | **Best in class**: `Ready/Loading/Error{message,detail}` + `empty_text` (picker.rs:30-41, 355-403) | Fixed columns: glyph, label (with `matched` bold), detail, tag, group (picker.rs:19-28, 453-501) | `Chosen`, `ChosenAlt`, `Secondary`, `Back`, `NextScope` (picker.rs:68-80) | No | `set_items` resets cursor to first enabled (picker.rs:103-108) | Keys required; promote `PickerStatus` to the shared empty/loading/error vocabulary |
| **Completion** (completion.rs) | Owned `Vec<CompletionItem>` (completion.rs:32) | Positional (completion.rs:82) | `cursor` only | No | Yes | `is_open()` = non-empty (completion.rs:70-72) — no explicit loading state | Fixed: glyph, label with `matched` bold, right-aligned detail (completion.rs:19-27, 185-228) | `Accept(i)`, `Dismiss` (completion.rs:43-46) | No | `open()` resets cursor (completion.rs:62-68) | Same item shape as Picker — unify `Item {glyph,label,matched,detail,tag,group,disabled,key}` |
| **ChipBar** (chips.rs) | Owned `Vec<Chip>` (chips.rs:37) | Positional (`chip_id`, `close_id`, chips.rs:67-71) | `cursor` over chips **plus** the `+ Add` stop (chips.rs:81-83) | Per-chip `enabled` toggle | No (overflow shows `…`, chips.rs:192-195) | None | Fixed (label + `×`) | `Activate/Toggle/Remove/Add/Lead/ClearAll` (chips.rs:46-53) | No | `cursor` clamped in `on_key` (chips.rs:90) | Preserve (it is a horizontal action strip, not a list) — but share the item/activation vocabulary |

### 6.1 Shared vocabulary to unify

**[I]**
1. **Item key** — every collection takes a caller key type; ids, selection, cursor restoration, close actions and parallel side-vectors all key off it. Eliminates: `switcher_targets` (app.rs:141), `picker_agents`/`picker_accounts`/`palette_cmds` (capsule.rs:87-90), `leaves: Vec<(Vec<usize>, usize)>` (inspect.rs:74), `picker_targets` (manager.rs:91, editor.rs:123), index-in-`detail` smuggling (app.rs:1294).
2. **Cursor vs selection vs activation** — three distinct concepts; `RadioGroup` (choice.rs:121-130) is the only violator.
3. **Empty / loading / partial / error** — `PickerStatus` (picker.rs:30-41) is the best existing model; `EmptyState` (used by grid, list, tree-less screens) should absorb it so every collection answers the same way.
4. **Row/cell decoration** — one `RowDecor`/`CellDecor` covering grid `RowState` (grid.rs:224), config `Change` (config.rs:128), manager `Row.trailing`/`glyph_tone` (manager.rs:56-60), accounts `Row.health` (accounts.rs:71), list `!` overlays (tabs.rs:2434).
5. **Custom row/cell renderer** — required by TablePro's plan tree (tabs.rs:1804-1852), history list (tabs.rs:2434-2442) and by six Jackin row models (J9). Without it, "custom domain rows" (goal §23-D) is unmet.
6. **Scroll + scrollbar + wheel routing** — already shared via `ScrollState` and `scrollbar::render_vertical`, but every component re-implements `on_scrollbar` with the same track arithmetic (grid.rs:1364-1374, table.rs:516-526, panel.rs:243-254, viewport.rs:519-530, code.rs:566-576, props.rs, list.rs). Extract once.
7. **Focus reconciliation on data change** — currently ad hoc (`set_rows` clamps, `set_items` resets, `remove` shifts, apps rebuild). One documented rule: keep the key if present, else nearest surviving index, else parent.

### 6.2 Meaningful differences to preserve

- **Tree** hierarchy, lazy children, and path-addressed expansion (tree.rs:22-26, 99-104).
- **Grid** two-axis cursor, rectangular range selection, per-cell editing and column virtualization — not applicable to lists.
- **Tabs** single-active-plus-cursor semantics and a strip window rather than a scroll offset.
- **StepRail** ordered lifecycle with a frontier (steps.rs:110-112) — not a selection list.
- **ChipBar** horizontal flow with a trailing "add" affordance and per-item remove.
- **Completion** non-modal, owner-keeps-focus contract (completion.rs:1-3) versus **Picker**'s modal, owns-focus contract (picker.rs:246-260).
- **Grid/Table** sort-as-permutation (`order`) so edits and pending changes stay bound to the source row (grid.rs:184-185, table.rs:100-102) — this invariant must survive the refactor and deserves an explicit test.

---

## 7. Risks and executable acceptance conditions

**Risks**

1. **Behaviour drift in the grid split.** The three edit-intent special cases (Bool cycle, JSON viewer, long-text viewer, grid.rs:552-579) and the empty-string/NULL policy (grid.rs:594-604) are easy to lose. Mitigation: port `grid.rs:2020-2191`'s tests to the adapter *before* moving code.
2. **Removing render-time commit changes observable behaviour.** Clicking away from an editing cell currently commits during the next draw. The replacement must call `blur()` on focus change in the shell, or edits silently persist as drafts. TablePro tests that rely on the current timing may fail legitimately.
3. **Stable keys touch every consumer.** Tabs, picker, list, tree and grid key changes ripple into ~20 call sites per app.
4. **`Panel::bg(t)` removal is wide.** Every `render(..., bg)` signature changes; ~120 call sites across both apps.
5. **Secret redaction may break snapshot tests** that currently capture `Debug` output.
6. **`DataTable` deletion** must not regress the Structure tab's six sections (tabs.rs:536-655) or its header-click sort (workbench.rs:1051-1053).

**Acceptance conditions (executable)**

```bash
# 1. No database vocabulary in the reusable grid.
! rg -n -i '\b(sql|primary key|nullable|foreign|references|NOT NULL|DEFAULT VALUES|commit queue)\b' src/widgets/grid.rs

# 2. No generic library module knows either application.
! rg -n 'tablepro|jackin|Catalog|Workspace|Instance' src/widgets src/core src/ui src/theme.rs

# 3. Rendering performs no semantic transition (conformance test, not grep):
cargo test --workspace render_does_not_commit_or_cancel
#    - render twice with focus absent while editing => value, pending set and
#      overlay open-state are byte-identical to before the renders.

# 4. Stable identity under reorder (Scenario E):
cargo test --workspace dynamic_identity_survives_reorder

# 5. Secrets never leak through Debug:
cargo test --workspace secret_redaction     # asserts format!("{:?}") excludes the value

# 6. Fn-pointer extension points are gone:
! rg -n 'fn\(&\w+, &str\)|: fn\(' src/widgets

# 7. Application code no longer chains owns/locate:
rg -c '\.owns\(|\.locate\(' src/bin        # target: 0 outside justified, documented cases

# 8. Behaviour preserved:
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## 8. Summary of dispositions

| Component | Disposition |
|---|---|
| `grid::DataGrid` | **Decompose**: generic grid + TablePro `GridModel`/`GridEditor` adapter (§1.5) |
| `table::DataTable` | **Remove**; absorbed by the generic grid |
| `code::CodeEditor` | Refactor onto a shared `TextEditorCore`; closure/trait hooks; extractable keymap |
| `completion::Completion` | Keep; add a `Completion` **controller** owning the editor↔popup contract |
| `diff::DiffView` | Keep (composition); move the data model behind a source trait |
| `dialog::Dialog` | **Decompose** into `Overlay` primitive + composed `Dialog` + convenience constructors |
| `menu::{ContextMenu, MenuBar}` | Keep; typed action keys + chords; add submenus |
| `picker::Picker` | **Decompose** into `FilterList` (headless) + overlay + palette convenience; typed keys and scopes |
| `panel::Panel` | Keep as a primitive; replace `bg: Color` with contextual surfaces |
| `panel::ScrollPanel` | **Remove**; use `TextViewport` |
| `statusbar::StatusBar` | Keep; **merge** with `segments` into one priority strip |
| `hintbar::HintBar` | Keep; wire to component-declared bindings so screens stop hand-writing hint tables |
| `tabs::Tabs` | Keep; **stable keys** + reconciling `set_items` |
| `splitter::Splitter` + `ui::layout::Split` | **Merge** into one `SplitPane` with pointer capture and optional keyboard resize |
| `viewport::TextViewport` | Keep as-is (best-in-class); drop `set_area` once view state is caller-owned |
| `input`/`textarea`/`select`/`choice`/`chips`/`field_common` | Refactor behind a `Field` wrapper + shared `TextEditorCore` + explicit edit lifecycle + `Secret` |
| New library additions | `Overlay`, `FormDialog`/field group, `ChoiceDialog`, `InfoDialog`, `HelpOverlay`, wizard/step-flow controller, staged async picker chain, keyed list/tree with custom row renderers, modal stack with focus trap/restore (J1–J13) |
