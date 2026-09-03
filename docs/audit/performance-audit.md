# Performance and Ownership Audit — current code + measurement plan

Scope: `REFACTORING_GOAL.md` §25.6 and §28 ("Performance and ownership review"), judged against `docs/audit/architecture-research.md` §2–§3, §8 and `docs/audit/interaction-audit.md` Part B (B1–B3).

Every claim under **FACT** carries a `file:line` citation and is directly readable in the source. Every claim under **INFERENCE** is judgement, arithmetic on those facts, or a recommendation. Cell/row counts for representative screens are arithmetic on the layout code and are labelled as estimates.

---

## 0. Executive summary

**FACT** — five structural cost centres exist today:

| # | Cost centre | Where | Shape |
|---|---|---|---|
| 1 | `ui::text::fit` / `truncate` allocate 3 / 1 `String` per call, and are called once per visible cell | `src/ui/text.rs:11-30, 80-91` | O(visible cells) allocations per frame |
| 2 | `TextViewport::ensure_layout` rebuilds a `String` **per grapheme** for the whole buffer | `src/widgets/viewport.rs:287-333` | O(total content), not O(viewport) |
| 3 | `pane.term.clone()` — full deep clone of a viewport per pane per frame | `src/bin/jackin_preview/screens/capsule.rs:1567` | O(scrollback × panes) per frame |
| 4 | `CodeEditor::render` clones the document, the spans, the diagnostics and the find state every frame, then does a linear span scan per grapheme | `src/widgets/code.rs:627-628, 655-674, 783` | O(text) + O(graphemes × spans) per frame |
| 5 | Three-stage full-result-set copy on every TablePro grid load | `sql.rs:1050, 1081-1084` → `tabs.rs:482-486` → `grid.rs:480-500` | 3 owned copies of the same data |

**FACT** — style resolution costs **zero allocations** today: every `Theme` resolver returns `Style` by value (`src/theme.rs:229-528`) and `Style` is `Copy`. This is a baseline the proposed recipe system must not regress.

**FACT** — hit testing, focus ring lookup, and every `owns`/`locate` helper are linear scans (`src/core/hit.rs:65-97`, `src/core/focus.rs:31-59`, `src/widgets/grid.rs:1208-1239`).

**INFERENCE** — the two designs in `architecture-research.md` §2 and `interaction-audit.md` B3 fix the *lookup* problems (B3 removes every `locate` scan) and the *ownership* problems (borrowed `&'a [T]` rows remove #5), but introduce three new hot-path risks: a per-frame `ItemKey` hash pass over whole collections, an allocating `sorted_by_key` inside `Ui::style`, and a debug-only `Id → String` side table populated at registration. All three are avoidable; §6 gives concrete mitigations with thresholds.

---

## 1. Per-frame allocation inventory

### 1.1 Library primitives — the multiplier

**FACT**

| Helper | Allocations per call | Cite |
|---|---|---|
| `width(s)` | 0 | `src/ui/text.rs:6-8` |
| `truncate(s, max)` | 1 `String` (also on the fits-already path, `s.to_owned()`) | `src/ui/text.rs:11-30` |
| `truncate_middle(s, max)` | 4: `Vec<&str>` of all graphemes, `head`, `tail`, `format!` | `src/ui/text.rs:33-64` |
| `thousands(n)` | 2 | `src/ui/text.rs:67-77` |
| `fit(s, w)` | **3**: `truncate` + `" ".repeat(pad)` + `format!` | `src/ui/text.rs:80-84` |
| `fit_right(s, w)` | **3** | `src/ui/text.rs:87-91` |
| `wrap(s, w)` | ≥ 1 `String` per output line + the `Vec` | `src/ui/text.rs:94-139` |
| `fuzzy(label, word)` | 2 `to_lowercase` + 1 `Vec<usize>` | `src/ui/text.rs:145-172` |

**FACT** — there is **no width cache and no grapheme cache anywhere**. `truncate` calls `width(s)` over the whole string and then `width(g)` per grapheme (`src/ui/text.rs:12, 21`); `fit` then calls `width(&t)` a third time (`:82`).

**FACT** — `TextBuffer::pos_of` computes `matches('\n').count()` + `rfind` + `width(prefix)` on every call (`src/core/text.rs:406-412`); `offset_at` walks the document line by line and grapheme by grapheme (`:415-432`); `line_count` is `split('\n').count()` (`:397-399`). All three are called from render paths.

### 1.2 Library widgets — per-frame allocations

**FACT**

| Site | What allocates | Rate |
|---|---|---|
| `list.rs:284` | `fit(&item.label, lw)` | 3 allocs / visible row |
| `list.rs:221` | `truncate(&self.empty_text, …)` | 1 / frame (empty state) |
| `tree.rs:556` | `fit(&row.label, …)` | 3 / visible row |
| `table.rs:616-623` | `format!` + `truncate` + `fit`/`fit_right` for the header title | 5 / column / frame |
| `table.rs:765-768` | `fit`/`fit_right` per cell | 3 / visible cell |
| `table.rs:727` | `text.chars().skip(off).take(cw).collect::<String>()` (editing) | 1 / frame |
| `grid.rs:1601-1617` | `suffix` + `truncate` + `format!` + `fit`/`fit_right` per header | 6 / column / frame |
| `grid.rs:1805, 1852-1856` | `cell_text(...)` (1–2) + `fit`/`fit_right` (3) per cell | **5 / visible cell** |
| `grid.rs:1753` | `(src+1).to_string()` + `fit_right` per row number | 4 / visible row |
| `grid.rs:1583` and **`grid.rs:1764`** | `self.col_rects.clone()` — the second one is **inside the row loop** | 1 `Vec<Rect>` / visible row |
| `grid.rs:1891-1914` | `pending.counts()` → `dirty_rows()` builds a `BTreeSet`; `format!` ×3; `truncate` | ~6 / frame when the pending bar is shown |
| `tabs.rs:265` | `self.areas = vec![Rect::ZERO; items.len()]` | 1 `Vec` / frame |
| `tabs.rs:281` | `widths: Vec<u16>` over all items | 1 `Vec` / frame |
| `tabs.rs:323, 458` | `format!("‹{:<3}", …)` / `format!("{:>3}›", …)` | 2 / frame when overflowing |
| `picker.rs:408-426` | three full passes over **all** items calling `width()` to size the columns | 0 allocs but O(items) / frame |
| `viewport.rs:293-333` | see §1.4 | O(total content) |
| `code.rs:627-628, 655-658, 662` | `spans().to_vec()`, `text().to_owned()`, `lines: Vec<&str>`, `running.clone()`, `find.clone()`, `diagnostics.clone()`, `line_starts: Vec<usize>` | **7 collections / frame, sized by the document** |
| `scrollbar.rs:53-58` | `position_label` → `format!` | 1 per caller per frame |

**FACT** — `code.rs:590` recomputes `hash_text(self.buffer.text())` — an FNV pass over the **entire document** — on every frame, purely to decide whether the highlight cache is valid.

**FACT** — `code.rs:668-674` defines `tone_at(off)` as a linear `spans.iter().find(...)`, and calls it once per rendered grapheme (`:767`). `code.rs:783` does the same linear scan over `diags` per grapheme, and `:774` over `find.matches` per grapheme.

### 1.3 Representative screen estimates

**INFERENCE** (arithmetic on the layout code; ±20 % depending on terminal size).

**Showcase, Lists page, 120×40.** `compute_layout` gives body height 37 (`app.rs:745-770`); the page caps columns at 18 rows (`pages/lists.rs:75`); `Panel::card` insets `Margin::new(2,1)` plus a title row (`panel.rs:89, 96-100`), and the page reserves 2 more rows (`lists.rs:110`) → **13 visible rows** in the `Language` list, 12 items in `Files to include`, 0 in the empty list.

| Source | Allocations / frame |
|---|---|
| single list: 13 rows × 3 (`fit`) | 39 |
| multi list: 12 rows × 3 | 36 |
| page-level: `items[i].label.clone()` (`lists.rs:92`), `position_label` (`:94`), two `format!` (`:103, 118`), one `truncate` (`:127`) | 5 |
| sidebar: 22 `fit` calls (`app.rs:914`) | 66 |
| header: `format!` crumb + `format!` cap (`app.rs:837, 841`) | 2 |
| footer + hint bar (`app.rs:1018+`) | ~10 |
| **Total** | **≈ 160 allocations / frame**, all small (< 128 B) |

Hit registry ≈ **57 regions**; focus ring ≈ **4** stops (three lists + `NAV`).

**TablePro, table tab, grid 500 rows × 12 cols, 120×40.** Gutter is `3 + num_w + 1` = 7 (`grid.rs:1546-1551`); with `min_width` 6–10 and a 2-cell gap, **≈ 6 columns** fit in the remaining ~108 cells; body height ≈ **30 rows**.

| Source | Allocations / frame |
|---|---|
| cells: 30 × 6 × 5 | **900** |
| `col_rects.clone()` inside the row loop (`grid.rs:1764`) | 30 `Vec<Rect>` |
| row numbers: 30 × 4 | 120 |
| headers: 6 × 6 | 36 |
| pending bar (when dirty) | ~6 |
| status line (`tabs.rs:795-838`: parts `Vec`, `rows_label`, `cols_label`, `joined`, `truncate`) | ~15 |
| **Total** | **≈ 1 110 allocations / frame** |

Hit registry ≈ **222 regions** for the grid alone (30 rownum + 180 cells + 6 headers + more/left/right/scrollbar/control/scrollable), plus the connections tree (~60) and tab strips (~10) → **≈ 300 regions**.

**Jackin, workspace manager, 100+ rows, 120×40.**

**FACT** — `ManagerScreen::render` calls `self.build_rows(w)` unconditionally (`manager.rs:2345`), and `draw_detail` calls `build_detail(w)` **and** `rebuild_actions(w)` unconditionally (`manager.rs:1003-1004`).

**INFERENCE** — `build_rows` (`manager.rs:142-304`) allocates per row: one `label` `String` (`:203, 259-264`) and one `meta` `String` (`:177-190, 216-253`), plus the row `Vec`. 100 rows ⇒ **≈ 201 allocations per frame** before anything is drawn. `build_detail` adds ~20–40 `String`s (`manager.rs:700-760`), `rebuild_actions` allocates up to 5 `Button`s (`:762-802`). Drawing then adds ~35 visible tree rows × (`fit`/`truncate`) plus the roster's `format!` + `truncate` pairs (`manager.rs:1464-1504`, 3 per running instance). **Total ≈ 350–400 allocations / frame**, and the row rebuild is repeated on **every key** (`:1746`), **every click** (`:2052`), **every tick** (`:1687`) and **every message** (`:1741`) as well.

**Jackin, Capsule, 4 panes.**

**FACT** — `capsule.rs:1567`: `let mut term = pane.term.clone();` per pane per frame. The clone copies `lines: Vec<Vec<Span>>` (a `String` per span), `cells: Vec<Vec<Cell>>` (a `String` per **grapheme**, `viewport.rs:90-98`) and `visual: Vec<VisualRow>`.

**FACT** — `capsule.rs:1432-1462`: the whole `Tabs` widget is rebuilt every frame — `TabItem::new(&label)` copies the label (`tabs.rs:32-37`), `.prefix(...)`/`.suffix(...)` copy again (`tabs.rs:42-49`), and `Tabs::with_items` allocates the `areas` vector.

**FACT** — `capsule.rs:1484` `self.pane_rects.clone()`, `:1608` `self.seams.clone()`.

**FACT** — `draw_status` builds ~8 `StatusItem`s from `format!` per frame (`capsule.rs:1676-1802`); `draw_identity` builds 3–4 `Segment`s including `truncate_middle` (`capsule.rs:1629-1665`).

**INFERENCE** — with 4 panes × 2 000 scrollback lines × ~60 graphemes, the clone alone is **≈ 480 000 `String` allocations and ~15 MB per frame**. At the Capsule's 80 ms animation cadence this is ~6 M allocations/second. This is by a wide margin the single worst per-frame path in the repository.

### 1.4 Per-event allocations

**FACT**

- Showcase: `Input::Key` → `describe_key(&key)` builds a `String` on **every** keystroke (`app.rs:366`).
- Showcase mouse `Down`/`Up`/`Wheel`: `nav_index_at(id)` is a 22-iteration scan (`app.rs:696-698`), invoked twice on `Up` (`:658`) and once on wheel (`:688`).
- `TreeView::toggle` → `row.path.clone()` + `expanded.insert(path.clone())` + **full `flatten()`**, twice on the lazy path (`tree.rs:282-299`).
- `TreeView::on_key` Left does `self.rows.iter().position(|r| r.path == parent)` — O(all rows), comparing `Vec<usize>` (`tree.rs:368`).
- `DataGrid::on_key` `y`/`Y` build the whole copy buffer (`grid.rs:1119-1126` → `copy_text`, `:870-920`, one `String` per cell).
- Jackin manager: `on_key` calls `build_rows` **before** examining the key (`manager.rs:1746`) — every arrow key pays the full ~200-allocation rebuild.
- `Picker` swallows every unmatched `Char` into the query and re-asks the owner for items (`picker.rs:147-220`), and `set_items` replaces the whole `Vec<PickerItem>` (`picker.rs:103-108`).

---

## 2. Cloning of rows and strings

### 2.1 Collections own duplicated strings

**FACT**

- `ListItem { label: String, meta: Option<String> }`, copied in `ListItem::new` / `.meta()` (`list.rs:13-34`).
- `TreeNode { label: String, meta: Option<String>, children: Vec<TreeNode> }` (`tree.rs:14-27`); every constructor copies (`:30-79`).
- `FlatRow` duplicates the node's `label` and `meta` **again**, plus a `path: Vec<usize>` (`tree.rs:86-96`, built at `:171-181`).
- `TabItem { label: String, prefix: Option<String>, suffix: Option<String> }` (`tabs.rs:19-49`).
- `Cell { text: String, error: Option<String> }` (`table.rs:63-67`); `Column { title: String }` (`:27-33`).
- `CellValue::Text(String)` / `Json(String)`; `ColumnSpec` owns `name`, `references`, `enum_values`, `type_label` (`grid.rs:32-41, 93-106`).
- `Span { text: String }` and `Cell { g: String }` (`viewport.rs:23-31, 89-98`).
- `PickerItem { label: String, detail: String, matched: Vec<usize> }` (`picker.rs:19-28`).

### 2.2 Full-collection clones per load

**FACT** — TablePro's grid load is a **three-copy chain** for the same data:

1. `sql.rs:1050` — `crate::db::rows(table, 0, scan)` with `scan = min(row_count, 2_000)` generates fresh `Value`s, mostly `format!`-built `String`s (`db.rs:921-969+`).
2. `sql.rs:1081-1084` — the projection clones every cell again: `proj.iter().map(|&i| r[i].clone())`.
3. `tabs.rs:482-486` — `rs.rows.iter().map(|r| r.iter().map(to_cell).collect()).collect()`, and `to_cell` clones the `String` for `Text`/`Json` (`tabs.rs:275-284`).
4. `grid.rs:480-500` — `sample_widths` then calls `CellValue::text()` for the first 200 rows × every column, allocating a `String` per cell purely to measure its width (`grid.rs:490`), plus one `Vec<usize>` per column which it sorts.

**INFERENCE** — for a 12-column table with `scan = 2 000` and a 500-row cap: stage 1 ≈ 24 000 `Value`s / ~15 000 `String`s; stage 2 ≈ 24 000 more / ~15 000 `String`s; stage 3 ≈ 6 000 `CellValue`s / ~4 000 `String`s; stage 4 ≈ 2 400 throwaway `String`s + 12 `Vec`s. **≈ 36 000 heap allocations and several MB per grid load**, for 6 000 cells actually displayed.

**FACT** — TablePro rebuilds whole widgets on state changes rather than mutating them:

- `TableTab::rebuild_structure` (`tabs.rs:535-700`) constructs a brand-new `DataTable` (`:655`) and a brand-new `ScrollPanel` of DDL lines (`:657-699`) every time the structure sub-tab changes.
- `QueryTab::sync_result_tabs` (`tabs.rs:1176-1193`) rebuilds the entire `Tabs` widget with fresh `TabItem`s after every run, close or pin.
- `TableTab::rebuild_chips` (`tabs.rs:518-533`) rebuilds every chip label with `format!` (`tabs.rs:173-181`).

**FACT** — `TreeView::flatten` (`tree.rs:145-205`) clones `path` (`:172`), `label` (`:174`) and `meta` (`:175`) for **every** row, and is called from `new`, `set_children`, `set_busy`, `set_filter`, `reveal`, `toggle` (twice on the lazy path), `expand_all` and `collapse_all` (`tree.rs:137, 234, 241, 246, 255, 288/292, 298, 314, 319`). `expanded: HashSet<Vec<usize>>` (`tree.rs:110`) means every membership test hashes a heap `Vec`.

**FACT** — `DataGrid::apply_commit_result` drains and rebuilds `self.rows` into a fresh `Vec` (`grid.rs:789-796`); `remove_inserted` rebuilds `order` and re-keys the entire `pending.cells` map (`grid.rs:672-690`).

### 2.3 Per-frame clones

**FACT**

| Site | Clone |
|---|---|
| `capsule.rs:1567` | whole `TextViewport` per pane |
| `capsule.rs:1484, 1608` | `pane_rects`, `seams` |
| `grid.rs:1583, 1764` | `col_rects` (the second is per row) |
| `code.rs:627` | `spans().to_vec()` — all highlight spans |
| `code.rs:628` | `self.buffer.text().to_owned()` — the whole document |
| `code.rs:655, 657, 658` | `running`, `find` (needle + **all** matches), `diagnostics` |
| `manager.rs:2345, 1003-1004` | `build_rows`, `build_detail`, `rebuild_actions` |
| `capsule.rs:1460` | `Tabs::with_items(...)` |
| `showcase/pages/lists.rs:92` | `self.single.items[i].label.clone()` |

**INFERENCE** — `code.rs:627-628` are the clearest example of the ownership problem the goal names: the editor already owns the text and the spans, but the borrow checker forces a copy because `render` takes `&mut self` and then wants to read `self` while writing the buffer. The fix is structural (split the read-only view from the mutable state), not a micro-optimisation.

---

## 3. ID lookup complexity

### 3.1 Registry and ring

**FACT**

| Operation | Complexity | Cite |
|---|---|---|
| `HitRegistry::register` | O(1) amortised, drops empty rects | `hit.rs:30-50` |
| `HitRegistry::hit(pos)` | **O(R)** reverse linear scan, R = regions after the barrier | `hit.rs:65-71` |
| `HitRegistry::hit_scroll(pos)` | **O(R)** | `hit.rs:75-81` |
| `HitRegistry::area_of(id)` | **O(R_total)**, ignores the barrier | `hit.rs:91-97` |
| `FocusRing::contains(id)` | **O(F)** | `focus.rs:31-33` |
| `FocusRing::next/prev` | **O(F)** (`position` then index) | `focus.rs:39-59` |
| `Focus::ensure_valid` | **O(F)** | `focus.rs:94-98` |

**INFERENCE** — observed registry sizes (§1.3): showcase lists ≈ 57, TablePro workbench ≈ 300, Jackin manager ≈ 105, Jackin Capsule ≈ 30. Ring sizes are 4–15. At these magnitudes the linear scans are **not** a measurable cost: a 300-element reverse scan of `Rect::contains` is well under a microsecond. The design problem is the *architecture* (`§3.2`), not the constant.

**FACT** — `area_of` is called three times per frame from the showcase inspector (`app.rs:965, 966, 980`) and scans the whole registry each time.

**FACT** — a modal pushes a barrier but the background is **still fully registered**: `showcase::draw` renders the page and then the dialog (`app.rs:790-798`), so the full ~57 registrations are paid every frame while a dialog is open. `RenderCtx.inert` would suppress this (`ctx.rs:87-89, 98, 105, 120`) but is set only during Jackin's handoff animation (`jackin/app.rs:2330, 2343`).

### 3.2 `owns` / `locate` scans

**FACT**

| Component | `locate` | `owns` |
|---|---|---|
| `ListBox` | O(viewport) over `visible_range` (`list.rs:186-188`) | `locate` + 2 compares (`:191-193`) |
| `TreeView` | **O(2 × viewport)** — two id families per row (`tree.rs:432-441`) | `locate` + 2 (`:444-446`) |
| `DataTable` | **O(viewport × columns)** (`table.rs:797-809`) | `locate` + `locate_header` = O(viewport×cols + cols) (`:811-816`) |
| `DataGrid` | **O(viewport × visible_cols)** (`grid.rs:1220-1231`) | `locate` + `locate_header` + `locate_rownum` + bar scan (`:1208-1218`) |
| `Tabs` | O(items) (`tabs.rs:122-124`) | `locate` + O(items) for close ids (`:126-133`) |
| `Picker` | O(viewport) (`picker.rs:120-122`) | `:123-125` |

**FACT** — `WidgetId` is a 64-bit FNV hash with no reverse mapping (`id.rs:10-46`), which is *why* every reverse lookup must be a scan.

**INFERENCE** — worst case today: a click inside a TablePro grid runs `DataGrid::owns` → `locate` (30 × 6 = 180 id derivations, each a two-step FNV) → `locate_header` (12) → `locate_rownum` (30) → 3 bar compares, and the app tries this against **each** candidate component in turn (`tabs.rs:1457-1493`). ≈ 250 hash computations per click. Still sub-microsecond, but it is O(viewport × cols) per event and the goal explicitly forbids that shape (§25.6, §12).

**FACT** — `nav_index_at` in showcase re-derives 22 child ids per call (`app.rs:696-698`), and `PageId::index()` is a 22-element scan (`app.rs:174-176`) called several times per frame.

### 3.3 Non-id scans in hot paths

**FACT**

- `DataGrid::row_state(src)` does `self.pending.cells.keys().any(|(r,_)| *r == src)` — **O(pending)** per visible row per frame (`grid.rs:512-524`, called at `:1711`).
- `PendingChanges::dirty_rows()` allocates and fills a `BTreeSet` (`grid.rs:197-203`); `counts()` calls it (`:205-211`); `counts()` is called from `render` (`:1891`) and `pending_label` (`:1436`) — twice per frame.
- `Picker::render` computes `label_col`, `tag_col`, `group_col` with three full O(items) passes calling `width()` (`picker.rs:408-426`).
- `DataTable::render` scans every column for a hovered cell per row (`table.rs:673`); `DataGrid` does the same over the visible column window (`grid.rs:1703-1705`).
- `TextViewport::render` searches `self.visual` linearly to place the caret — **O(total visual rows)** (`viewport.rs:657-662`).
- `code.rs:668-674` `tone_at` is O(spans) per grapheme.

---

## 4. Overlay dispatch, style resolution, Unicode

### 4.1 Overlay dispatch

**FACT** — modality is a single barrier index in each registry (`hit.rs:26, 53-62`; `focus.rs:12, 20-29`), set from inside `render` via `RenderCtx::begin_modal` (`ctx.rs:126-131`). Dispatch after the barrier is O(regions-after-barrier), which is small. There is no per-layer structure and no pop.

**FACT** — `ui::popup::surface` registers every popup under one shared constant id `WidgetId::of("popup.surface")` (`src/ui/popup.rs:76`), so two popups in one frame collide.

**INFERENCE** — overlay dispatch is **not** a performance problem today. It is a correctness/architecture problem (nesting, ownership) already covered by `interaction-audit.md` A6/B7. The only measurable overlay cost is that the shadowed background is still fully rendered and registered every frame (§3.1).

### 4.2 Style resolution

**FACT** — every resolver on `Theme` returns a `Style` by value and allocates nothing: `row` (`theme.rs:329-359`), `lift` (`:362-372`), `gutter` (`:376-385`), `button` (`:387-451`), `field_style` (`:454-464`), `tone` (`:492-502`), `syntax` (`:506-519`), `backdrop` (`:297-321`), `scrollbar_*` (`:478-490`). `Theme` itself is `Copy` (`theme.rs:98`).

**INFERENCE** — resolver call counts for one frame of a large list (13 visible rows, `list.rs:233-298`):

| Call | Per row | Total (13 rows) |
|---|---|---|
| `t.row(s, bg)` | 1 (+1 nested `lift` when hovered) | 13–14 |
| `t.gutter(...)` | 1 | 13 |
| `st.fg(t.accent \| t.text_secondary)` for the marker | 1 | 13 |
| `st.fg(t.text_muted)` for meta | ≤ 1 | ≤ 13 |
| **Total** | **≈ 4** | **≈ 52 resolutions / frame, 0 allocations** |

For a 30 × 6 TablePro grid frame (`grid.rs:1669-1879`): `t.row` + `t.gutter` + the change-glyph match per row (90), plus 3–6 `Style` builder mutations per cell (~700) ⇒ **≈ 800 style operations per frame, 0 allocations**. `backdrop` is different: it walks **every cell of the screen** when a modal opens (`dialog.rs:359-372`, `picker.rs:253-259`) — 120 × 39 = 4 680 calls per frame, each a chain of up to 9 colour equality tests (`theme.rs:297-321`). Still allocation-free, but it is the largest style workload in the repo.

### 4.3 Unicode processing

**FACT** — every width and grapheme query goes straight to `unicode-width` / `unicode-segmentation` with no memoisation: `ui::text::width` (`ui/text.rs:6-8`), `truncate` (`:20-27`), `truncate_middle` (`:42-62`), `wrap`/`hard_wrap` (`:94-139`), `TextBuffer::prev_boundary`/`next_boundary` (`core/text.rs:171-185`), `pos_of` (`:406-412`), `offset_at` (`:415-432`), `viewport::ensure_layout` (`viewport.rs:299-302`), `code.rs:745-746` (`grapheme_indices` + `width` per rendered grapheme).

**INFERENCE** — width/grapheme calls per rendered cell:

| Path | Calls / cell |
|---|---|
| `ListBox` label | ~3 (`fit` → `truncate` scans graphemes, then `width` twice more) |
| `DataGrid` cell | ~4 (`cell_text` truncates once, `fit` truncates + widths again) |
| `CodeEditor` glyph | 1 `width(g)` per grapheme, plus `pos_of`/`offset_at` per cursor move |
| `TextViewport` | 1 `width(g)` per grapheme **of the whole buffer** on every relayout |

There is **no cache to be found** anywhere in `ui/text.rs` or `core/text.rs`. Adding one is not obviously right (the strings change every frame in most call sites); the correct fix is to stop producing intermediate `String`s at all — paint graphemes directly into the buffer with a single pass, which halves the width calls and removes the allocations.

---

## 5. Large-data behaviour today

### 5.1 `ListBox` with 100 000 rows

**FACT** — construction holds 100 000 `ListItem`s, each with an owned `label` (`list.rs:61-75`). Render is virtualised: `for (i, li) in self.scroll.visible_range().enumerate()` (`list.rs:233`). No sort, no flatten.

**INFERENCE** — verdict: **acceptable**. Frame cost is O(viewport). Two O(n) event paths remain: the `a` select-all/none (`list.rs:152-165`) and `checked: Vec<bool>` (`:52`), which is 100 KB — fine. Memory is the caller's problem, and Scenario D (`architecture-research.md:1145`) removes it.

### 5.2 `TreeView` with 100 000 nodes

**FACT** — `flatten()` walks the whole expanded tree and pushes a `FlatRow` with `path.clone()` + `label.clone()` + `meta.clone()` per row (`tree.rs:149-205`). It is called on every structural change including a single `toggle` (`tree.rs:288, 292, 298`). `expanded` is a `HashSet<Vec<usize>>` and is probed once per node per flatten (`:167`). Filtering calls `matches()`, which does `n.label.to_lowercase()` per node **and recurses over all children** (`tree.rs:146-148`) — O(n²) in the worst case for a deep tree. Render itself is virtualised (`tree.rs:487`).

**INFERENCE** — verdict: **not viable at 100 k**. One expand costs ≈ 300 000 allocations plus 100 000 `Vec` hashes. `set_filter` on 100 k nodes additionally allocates a lowercased `String` per node per level of nesting.

### 5.3 `DataTable` with 100 000 rows

**FACT** — `rows: Vec<Vec<Cell>>` fully owned (`table.rs:99`). Render virtualised by rows (`table.rs:660`) but iterates **all** columns per row (`:694`), registering each visible cell **twice** (`:779` and again at `:786`). `apply_sort` is a full `sort_by` over `order` (`table.rs:216-234`) whose comparator allocates: `ca.to_lowercase()` + `cb.to_lowercase()` per comparison (`:226`), or two `parse_num` calls each of which builds a filtered `String` (`:830-840`).

**INFERENCE** — sorting 100 000 rows ≈ 1.7 M comparisons × 2 allocations = **3.4 M allocations for one header click**. Verdict: **render fine, sort not viable**.

### 5.4 `DataGrid` with 100 000 rows

**FACT** — `set_rows` calls `sample_widths` (bounded to the first 200 rows, `grid.rs:489`) — good — but each sampled cell allocates a `String` (`:490`). `order = (0..len).collect()` (`:442`). `local_sort` is a full sort whose comparator falls back to `a.text().to_lowercase()` — **4 allocations per comparison** (`grid.rs:1963`). `content_rows`/`scroll` are O(1). Render is virtualised (`grid.rs:1669`), but `row_state` is O(pending) per row (`:519`) and `col_rects.clone()` is per row (`:1764`).

**INFERENCE** — verdict: **render acceptable but wasteful; `local_sort` not viable** (100 k rows ⇒ ~7 M allocations). Server-side sort is the default (`local_sort` is opt-in, `grid.rs:335`), which is the right architecture.

### 5.5 `TextViewport` — the worst case

**FACT** — `ensure_layout` rebuilds `cells: Vec<Vec<Cell>>` for **every line in the buffer**, allocating a `String` per grapheme (`viewport.rs:293-333`, `g.to_owned()` at `:321`, and four `" ".into()` per tab at `:306`). It runs whenever `dirty` is set, i.e. after every `push` (`:180`), `set_lines` (`:186`), `replace_last` (`:196`) or `clear` (`:204`), and whenever the width changes (`:288`).

**INFERENCE** — pushing one line into a 100 000-line viewport re-lays out all 100 000 lines. With ~60 graphemes/line that is **6 M `String` allocations per pushed line** — quadratic in the amount of output. Combined with `capsule.rs:1567`'s per-frame deep clone, this is the dominant cost in the whole repository. Verdict: **structurally broken for large data**; it must become an incremental, windowed layout storing `(byte range, width)` instead of owned graphemes.

---

## 6. Judging the proposed design

### 6.1 What the proposal fixes (INFERENCE, with cites)

| Current problem | Fixed by | How |
|---|---|---|
| §2.2 three-copy result set | `List::items(&'a [T])` + `.row(FnMut(&T, &mut RowUi))` (`architecture-research.md:369-405`) | domain rows are borrowed; only visible rows invoke the renderer (`:408`) |
| §3.2 `owns`/`locate` scans | B3 path-recorded routing (`interaction-audit.md:338-366`) | the registry records `{owner, part}`; hit-testing returns the resolved part — no hash inversion, no scan |
| §2.1 owned `ListItem`/`TabItem` strings | views borrow, state owns (`architecture-research.md:938`) | `XState` holds only interaction state |
| §5.4 index-keyed identity forcing rebuilds | `ItemKey` + `reconcile` (`architecture-research.md:361, 431`) | TablePro's `sync_result_tabs` and Jackin's `draw_strip` stop rebuilding widgets |
| render-time semantic mutation | `update` / `draw` split (`architecture-research.md:122-124`) | removes `code.rs:611-613`, `table.rs:566-568`, `grid.rs:1518-1520` commits-in-render |
| §1.1 dialog/picker full-screen `backdrop` walk | unchanged by the proposal | see §6.3 |

**INFERENCE** — no path in either document copies a whole data set per frame or per event. B3 explicitly rejects the runtime-owned component tree and the full-tree walk on the grounds of §9.4/§25.6 (`interaction-audit.md:336`), and §7's prior-art table rejects the retained `Box<dyn Widget>` tree for the same reason (`architecture-research.md:1037`). That judgement is correct and is the single most important performance decision in the design.

### 6.2 New hot-path risks introduced

**R1 — `ItemKey` hashing over whole collections, per frame.**
`Tabs::show` is specified to call `state.reconcile(items.iter().map(&key))` **before** drawing (`architecture-research.md:454`), and `ListState::reconcile(keys: impl Iterator<Item = ItemKey>)` has the same shape (`:362`). `ItemKey::text(s)` is FNV over the bytes (`:148`). For a 100 000-row list this is a 100 000-key hash pass on every frame — precisely the "avoidable full scan" §25.6 forbids, and it would fail acceptance condition `T-D` ("zero allocations… 100 000 rows", `:1083`) on time even if not on allocations. The same problem appears in reverse: `ListState` stores `cursor: Option<ItemKey>` (`:349`) but rendering needs a **row index** to drive `ScrollState`, so something must map key → index.

*Mitigation.* Cache `(cursor_key, cursor_index, gen)` inside `ListState`. On each frame compute a cheap generation stamp — `(items.len(), key(first), key(last))` — and skip reconciliation when it is unchanged; when it changed, first probe the cached index (`key(&items[i]) == cursor_key`) and only fall back to a scan on a miss. Provide `ListState::invalidate()` for callers that mutate in place.
*Acceptance threshold.* `list_100k_rows_render` must show `allocs/frame < 500` **and** `ns/frame` within 1.5× of `list_1k_rows_render`; `event_dispatch_is_not_o_n` must show a click into a 100 000-row list within 3× of the same click into a 100-row list.

**R2 — `Ui::style` allocates on the hot path.**
`architecture-research.md:711-723` resolves state rules with

```rust
for rule in r.parts[p].states.iter()
    .filter(|k| s.contains(k.when))
    .sorted_by_key(|k| k.when.bits().count_ones()) { … }
```

`sorted_by_key` on an iterator materialises a `Vec`. That is **one heap allocation per part per element per frame** where today the equivalent (`theme.rs:329-359`) allocates nothing. At the §4.2 counts that is ~50 allocations/frame for a list and ~800/frame for a grid — a straight regression against the current baseline.

*Mitigation.* The rules of a `PartRecipe` are static once the theme is built. Sort `states` by `when.count_ones()` at recipe-construction time and make resolution a plain `for rule in &part.states { if s.contains(rule.when) { acc = acc.merge(rule.patch) } }`. Document "state rules are stored in specificity order" as a recipe invariant.
*Acceptance threshold.* `style_resolve_10k_parts` must assert **exactly 0 allocations** and land within 2× of today's `Theme::row`+`Theme::gutter` baseline.

**R3 — hashing in the overlay lookup.**
§3.4 claims "O(1), no allocation, no hashing on the hot path" (`architecture-research.md:705`), but `Family`, `Variant` and `Part` all derive `Hash` (`:671-673`) and the resolution loop calls `ov.lookup(f, v, p, s)` for every entry of the style stack (`:718-720`), which reads like a map probe on a 4-tuple. Four hashes per part per element per frame is measurable at grid scale.

*Mitigation.* Represent `Overlay` as a small `&'static [(Family, Variant, Part, StateFlags, StylePatch)]` scanned linearly (overlays are declared `static`, per `:794-795`), and short-circuit the entire loop when the stack is empty — the overwhelmingly common case.
*Acceptance threshold.* `style_resolve_10k_parts` (empty stack) and `style_resolve_10k_parts_with_two_overlays` must both be allocation-free, and the two-overlay case must be within 2× of the empty case.

**R4 — the debug `Id → String` side table.**
B1 proposes "the runtime keeps a side table `Id -> String` **populated at registration**" (`interaction-audit.md:262`). Registration happens for every region on every frame (§3.1: 300 regions in TablePro). In a debug build that is 300 map inserts + 300 `String`s per frame, which will make `cargo run` visibly laggy and will corrupt any allocation-counting test run in debug mode.

*Mitigation.* Populate the name table **at `Id` construction** — `id!` expands to a `const` path (`architecture-research.md:150`), so the mapping can be a build-time static registry or a `once`-initialised table keyed by the literal. Never write to it from `register_*`. If a registration-time table is unavoidable, gate it behind an explicit `debug-ids` cargo feature rather than `debug_assertions`.
*Acceptance threshold.* `frame_tablepro_grid_500x12_120x40` must report identical allocation counts in debug and release.

**R5 — `RowUi` must not reintroduce `fit`.**
`RowUi::label(&str)` / `meta(&str)` (`architecture-research.md:383-390`) are the natural home for today's `fit` calls. If they call `ui::text::fit` internally, the 3-allocations-per-cell cost of §1.1 survives the refactor unchanged, and acceptance condition `T-D` ("`#[global_allocator]` counting shim asserts **zero** allocations during the render frame", `:1083`) becomes unsatisfiable.

*Mitigation.* Implement `RowUi::label` as a single grapheme walk that writes cells directly and pads in place — no intermediate `String`. Keep `ui::text::fit` only for the rare non-render caller, and delete `fit`/`fit_right` from every render path.
*Acceptance threshold.* `fit_10k_grapheme_line_to_80` records 3 allocations today; after the refactor the equivalent `RowUi` path must record **0**, and `frame_showcase_lists_120x40` must drop from ≈160 to **< 20 allocations/frame**.

**R6 — `Intents` drain cost.**
`ui.take_intents(id)` (`architecture-research.md:224`) is called once per component per frame. If it scans the whole queue each time, the frame cost is O(components × intents).

*Mitigation.* Build a small per-frame index (or sort the queue by `Id` once and binary-search); the queue is tiny (≤ ~4 intents/frame), so even a linear scan is fine — but assert it: `frame_*` allocation counts must not scale with the number of registered components.

**R7 — `KeySet` for multi-selection.**
`ListState.checked: KeySet` (`architecture-research.md:351`). Today "select all" over 100 000 rows flips a `Vec<bool>` (`list.rs:159-163`); with keys it would materialise 100 000 `ItemKey`s.
*Mitigation.* Give `KeySet` an inverted `AllExcept(set)` representation, or document that `ToggledAll` returns an action and leaves the set to the caller.

### 6.3 Problems the proposal does **not** address (INFERENCE)

1. **`TextViewport` / terminal panes.** Neither document sketches a viewport or a tree view. The two worst paths in the repo (§5.5, §5.2) therefore have no design. Recommendation: `Cell` must become `{ range: Range<u32>, w: u8, style_ix: u16 }` referencing the source `Span` text instead of `g: String`; layout must be incremental (append-only for `push`) and windowed (lay out only `visible_range ± 1 page`); the Capsule's per-frame `pane.term.clone()` must be replaced by a `&TextViewport` + a `ViewportOverride { caret_visible }` parameter so no clone is needed.
   *Acceptance thresholds.* `viewport_100k_lines_push` allocations must be **independent of `lines.len()`**; `frame_jackin_capsule_4_panes_120x40` must report **< 200 allocations/frame** (from an estimated ~480 000).
2. **`CodeEditor`.** The `update`/`draw` split removes the render-time commit but not the seven per-frame clones (`code.rs:627-628, 655-658, 662`) or the per-grapheme linear span scan (`:668-674, 774, 783`). Recommendation: keep the highlight cache keyed on a monotonically incremented edit counter instead of re-hashing the document every frame (`code.rs:590`); store spans sorted and advance a cursor across them as the render walks graphemes (O(graphemes + spans) instead of O(graphemes × spans)); same for diagnostics and find matches.
3. **`backdrop`.** The full-screen cell walk (§4.2) stays. Recommendation: resolve the dim as a `StylePatch` applied through the same `Resolved` path so a monochrome/no-colour theme gets it for free, and restrict the walk to the rect actually covered.
4. **`sample_widths`.** Measuring a column by allocating a `String` per cell (`grid.rs:490`) should become a `fn display_width(&self) -> usize` on `CellValue` that measures without materialising.

### 6.4 Verdict

**INFERENCE** — the proposed architecture is sound on the two criteria §25.6 names explicitly: **no path copies a whole data set** (borrowed `&'a [T]` + visible-only row renderers) and **no path does a full-tree scan per event** (B3 part tokens replace every `locate`). It is a net large improvement over the current code. The four regressions it *could* introduce (R1–R4) are all in the "small constant, huge multiplier" class and are all fixable with the mitigations above, none of which changes the public API. The remaining gap is that the two genuinely broken large-data components — `TextViewport` and `TreeView` — are not covered by either document and need their own design before Slice 4.

---

## 7. Measurement plan

**Constraints honoured.** No new dependencies. Uses only `std::time::Instant`, `std::alloc::{GlobalAlloc, System}`, `std::sync::atomic`, and the existing `ratatui::backend::TestBackend` / `ratatui::Terminal`.

### 7.1 Harness

**Location.** A single dedicated integration-test binary, `tests/perf.rs` (post-workspace: `crates/tui/tests/perf.rs`, plus `apps/*/tests/perf.rs` for the app-shell benchmarks). Shared helpers live in `crates/tui-testing/src/perf.rs`.

**Why a dedicated binary.** `#[global_allocator]` is per-binary. Declaring it in `tests/perf.rs` keeps the counting shim out of the library, out of the three application binaries, and out of every other test binary, so no other test's timing or behaviour changes.

```rust
// crates/tui-testing/src/perf.rs  (no #[global_allocator] here)
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub static ALLOCS: AtomicUsize = AtomicUsize::new(0);
pub static BYTES:  AtomicUsize = AtomicUsize::new(0);

pub struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(l.size(), Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { unsafe { System.dealloc(p, l) } }
    // realloc / alloc_zeroed forwarded, realloc counted as one alloc + delta bytes
}

pub struct Stats { pub ns: u128, pub allocs: usize, pub bytes: usize }

/// Warm up `warm` iterations, then run `iters` and return per-iteration medians
/// over 9 repetitions.
pub fn bench(name: &str, warm: usize, iters: usize, f: &mut dyn FnMut()) -> Stats { … }

/// One machine-readable line per benchmark.
pub fn report(name: &str, s: &Stats) {
    println!("PERF {name} ns={} allocs={} bytes={}", s.ns, s.allocs, s.bytes);
}
```

```rust
// tests/perf.rs
#[global_allocator]
static GLOBAL: tui_testing::perf::Counting = tui_testing::perf::Counting;
```

**Assertion policy.**
- **Allocation count and byte count are deterministic** and machine-independent → **hard assertions** against a checked-in baseline. Any increase fails.
- **Wall time is noisy** → reported always; asserted only when `PERF_STRICT=1` (set in one pinned CI job), and then only against `baseline × 1.2`. Otherwise the harness prints `REGRESSION? <name> ns=… baseline=…` and continues.
- Baseline lives in `tests/perf_baseline.txt`, one `name ns allocs bytes` line per test, regenerated only with `PERF_BLESS=1` and reviewed in the diff like a snapshot (goal §25.3: "Review snapshot changes rather than blindly accepting them").
- **The baseline must be recorded on the pre-refactor tree**, on a `perf/baseline` commit, so §25.6's "before and after" is literal.

Every screen benchmark also reports the two structural numbers the audit cares about: `hits=<registry len>` (`HitRegistry::len`, `hit.rs:83-85`) and `ring=<reachable len>` (`FocusRing::reachable`, `focus.rs:24-29`).

### 7.2 Benchmarks

#### A. Render frames — `tests/perf.rs::frames`

| Test name | Measures | Notes |
|---|---|---|
| `frame_showcase_lists_120x40` | full shell render of the Lists page: header, 22-entry sidebar, 3 lists, footer | 200 frames; expected baseline ≈ 160 allocs/frame, 57 hits, 4 ring |
| `frame_showcase_lists_80x24` | same at the minimum supported size | proves the responsive path costs no more |
| `frame_showcase_dialog_open` | Lists page with a modal open | quantifies §3.1: the shadowed background is still fully rendered and registered |
| `frame_tablepro_grid_500x12_120x40` | `TableTab` render with 500 rows × 12 columns loaded | 200 frames; expected ≈ 1 110 allocs/frame, ≈ 300 hits |
| `frame_jackin_manager_100rows_120x40` | manager render with 100+ instance rows expanded | 200 frames; captures `build_rows` + `build_detail` + `rebuild_actions` (`manager.rs:2345, 1003-1004`) |
| `frame_jackin_capsule_4panes_120x40` | 4 panes, 2 000 scrollback lines each | 200 frames; captures `pane.term.clone()` (`capsule.rs:1567`) — expected to dominate every other number in this suite |

Each builds the real app struct, drives one warm-up frame (so `ScrollState` viewports and `col_rects` are primed), then times `terminal.draw(|f| app.render(f))`.

#### B. Event handling — `tests/perf.rs::events`

| Test name | Measures |
|---|---|
| `key_showcase_down_lists` | 1 000 `Input::Key(Down)` after one render; includes `describe_key` (`app.rs:366`) |
| `key_tablepro_grid_cursor` | 1 000 arrow keys on the 500×12 grid |
| `key_tablepro_grid_sort_local` | 20 `s` presses with `local_sort = true`; isolates `cmp_cells`'s 4 allocations/comparison (`grid.rs:1963`) |
| `key_jackin_manager_move` | 1 000 arrow keys; includes the per-key `build_rows` (`manager.rs:1746`) |
| `key_tree_toggle_10k` | 100 expand/collapse toggles on a 10 000-node tree; isolates `flatten` (`tree.rs:145-205`) |
| `mouse_move_over_1000_regions` | synthetic `HitRegistry` of 1 000 non-overlapping regions; 10 000 `hit(pos)` calls covering hits, misses and the barrier case (`hit.rs:65-71`); **assert 0 allocations** |
| `mouse_move_showcase_frame` | replay a raster of positions over a real showcase frame's registry; reports `hits=` alongside ns/hit |
| `mouse_click_grid_cell` | 1 000 clicks routed through `DataGrid::owns`/`locate` (`grid.rs:1208-1231`) — the O(viewport×cols) path |
| `wheel_showcase_lists`, `wheel_tablepro_grid` | 1 000 wheel events; **assert 0 allocations** |
| `focus_tab_traversal_ring_200` | 10 000 `FocusRing::next` over a 200-entry ring (`focus.rs:39-48`); **assert 0 allocations** |

#### C. Style resolution — `tests/perf.rs::style`

| Test name | Measures |
|---|---|
| `style_resolve_10k_parts` | 10 000 resolutions in the mix a list frame uses (`row`, `gutter`, marker `fg`, meta `fg`). Today: `Theme::row`/`gutter` (`theme.rs:329-385`). After: `Ui::style(Family, Variant, Part, StateFlags)`. **Assert exactly 0 allocations** (R2). |
| `style_resolve_10k_parts_with_two_overlays` | same under a two-deep scoped overlay stack. **Assert 0 allocations**; ns within 2× of the previous test (R3). |
| `style_backdrop_full_screen_120x40` | 100 iterations of the modal dim walk (`dialog.rs:359-372`) — 4 680 `Theme::backdrop` calls per iteration. **Assert 0 allocations.** |
| `style_downgrade_theme_all_levels` | `Theme::for_level` × 4 levels × 1 000 (`theme.rs:183-225`); one-shot cost, guards against the proposed `map_colors` visitor becoming per-frame. |

#### D. Large data — `tests/perf.rs::large`

| Test name | Measures | Threshold |
|---|---|---|
| `list_100k_rows_construct` | one-shot build cost | report only |
| `list_100k_rows_render` | 100 frames, 40-row viewport | **allocs/frame < 500**; ns/frame within 1.5× of `list_1k_rows_render` (guards R1) |
| `list_1k_rows_render` | control for the above | — |
| `tree_100k_nodes_flatten` | one `toggle` at the root | today: report; after: `allocs < 10 × viewport` |
| `tree_100k_nodes_render` | 100 frames | allocs/frame independent of node count |
| `grid_500x12_render` | isolated `DataGrid::render`, no app shell, 200 frames | after refactor: **allocs/frame < 100** |
| `grid_500x12_load` | one `TableTab::load` | today ≈ 36 000 allocs; after: **< 8 000** (one owned copy, not three) |
| `grid_100k_local_sort` | one `apply_local_sort` | report; documents why `local_sort` must stay opt-in |
| `viewport_100k_lines_push` | push 1 000 lines into a 100 000-line viewport | **allocs must not scale with `lines.len()`** |
| `viewport_100k_lines_render` | 100 frames | allocs/frame independent of buffer size |
| `capsule_pane_clone_4x2000` | isolates `pane.term.clone()` | after refactor this test must be **deleted** (the clone is gone) |

#### E. Unicode — `tests/perf.rs::unicode`

Fixture: one 10 000-grapheme line mixing ASCII, CJK (width 2), combining marks, and an emoji ZWJ sequence.

| Test name | Measures | Assertion |
|---|---|---|
| `width_10k_grapheme_line` | `ui::text::width` (`ui/text.rs:6-8`), 1 000 iterations | **0 allocations**; report ns |
| `truncate_10k_grapheme_line_to_80` | `ui::text::truncate` (`:11-30`) | today **exactly 1** allocation |
| `fit_10k_grapheme_line_to_80` | `ui::text::fit` (`:80-84`) | today **exactly 3**; after refactor the `RowUi` equivalent must be **0** (R5) |
| `truncate_middle_10k_to_40` | `:33-64` | today **exactly 4** |
| `wrap_10k_graphemes_to_80` | `:94-127` | report allocs (≥ 1 per output line) |
| `textbuffer_pos_of_10k_line` | `core/text.rs:406-412` | **0 allocations**; report ns |
| `textbuffer_offset_at_10k_line` | `core/text.rs:415-432` | **0 allocations**; report ns |
| `viewport_layout_10k_grapheme_line` | `viewport.rs:287-333` for one line | today ≈ 10 000 allocations; after: **0** |

#### F. Invariant guards — `tests/perf.rs::invariants`

These are the executable form of §25.6's two prohibitions.

| Test name | Asserts |
|---|---|
| `render_twice_allocates_the_same` | render an identical frame twice; allocation counts must be equal. Catches lazily-built caches that never stabilise and (with `architecture-research.md:1081` `T-B`) that `draw` is idempotent. |
| `no_full_collection_clone_per_frame` | render a 100 000-row list frame and a 100 000-line viewport frame; **`bytes/frame < 64 KiB`** for each. This is the direct executable statement of "do not accept a design that requires copying entire application data sets". |
| `event_dispatch_is_not_o_n` | one click into a 100 000-row list: **0 allocations**, and ns within **3×** of the same click into a 100-row list. Catches a reintroduced `locate` scan (§3.2) or a per-event full `ItemKey` pass (R1). |
| `hit_registry_size_is_bounded` | for each representative screen, `hits` must stay within its recorded baseline ± 10 %. Catches accidental double registration (as at `table.rs:779` + `:786`). |
| `debug_and_release_alloc_counts_match` | run `frame_tablepro_grid_500x12_120x40` under `debug_assertions`; allocation count must equal the release number. Catches R4. |

### 7.3 CI wiring

**INFERENCE** — add one job, separate from the correctness gates so a noisy machine never blocks a merge:

```bash
# fast, deterministic, always-on: allocation counts only
cargo test --workspace --test perf --release

# pinned-runner job, timing enforced
PERF_STRICT=1 cargo test --workspace --test perf --release -- --test-threads=1
```

`--test-threads=1` is required: the global allocator counters are process-wide, so concurrent tests would interleave counts. The harness should `assert!(std::thread::available_parallelism().is_ok())` and document the single-thread requirement at the top of `tests/perf.rs`.

The `PERF` output lines are grepped into a build artefact so a reviewer can diff before/after in the PR description, satisfying goal §30 item 13 ("Performance findings").

---

## 8. Prioritised recommendations

**INFERENCE**, ordered by (measured impact × structural clarity):

1. **Delete the per-frame viewport clone.** `capsule.rs:1567` → pass `&TextViewport` plus an explicit caret-visibility override. Largest single win in the repository.
2. **Rewrite `TextViewport::ensure_layout`** to store `(byte range, width)` per cell instead of `String`, lay out only the visible window, and append incrementally on `push`. Fixes §5.5.
3. **Remove `fit`/`fit_right`/`truncate` from every render path** in favour of a direct grapheme-walk painter (`RowUi`/`CellUi`). Removes ~85 % of all per-frame allocations across all three apps at once, and is a precondition for acceptance condition `T-D`.
4. **Collapse the TablePro three-copy load** into one owned conversion, and give `CellValue` a `display_width()` that does not allocate.
5. **Make `TreeView::flatten` incremental and borrow-based** (or replace it with a keyed, windowed flat index). Fixes §5.2.
6. **Fix `CodeEditor`'s per-frame clones and per-grapheme scans**: an edit counter instead of `hash_text`, and a sorted-span cursor instead of `find`.
7. **Stop rebuilding rows in `render`/`on_key`** in the Jackin manager (`manager.rs:2345, 1746, 2052`): rebuild on world change only, gated by a world generation counter.
8. **Apply R1–R5 to the new design before implementation**, and land the benchmark harness *first* so the baseline is recorded on the current tree.

**Non-recommendations** (explicitly deferred): the linear hit-test and focus-ring scans are correct and fast at observed sizes (§3.1) — replacing them with spatial indexing would add complexity for no measurable gain; a Unicode width cache is not warranted because the strings change every frame — removing the intermediate `String`s is the real fix (§4.3).
