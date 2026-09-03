I could not use grep/glob (only `Read` was available), so counts below are stated as lower bounds with an explicit coverage table. Here is the audit, ready to save verbatim to `docs/audit/app-audit.md`.

---

# Application Migration Audit — showcase · tablepro · jackin-preview

Baseline revision: `d5e7075` (clean tree). Package: single crate `junie-tui` (`Cargo.toml:1-25`), edition 2024, MSRV 1.88, deps `ratatui 0.30` + `unicode-width` + `unicode-segmentation`.

## 0. Method, coverage and confidence

**Tooling constraint (fact).** This audit was produced with a read-only agent that had **only** a file-read tool — no shell, no `grep`, no `glob`. Every citation below was read directly. Counts are therefore **lower bounds over the files listed as "full" below**, not repository-wide totals.

| Area | Read in full | Read partially | Not read |
|---|---|---|---|
| `src/bin/showcase/` | `main.rs`, `app.rs`, `app_tests.rs`, `pages/mod.rs`, and all 22 page modules | — | `data.rs` |
| `src/bin/tablepro/` | `main.rs`, `app.rs` (2511 lines), `workbench.rs`, `tabs.rs` (2526 lines), `connections.rs`, `app_tests.rs` | — | `db.rs`, `model.rs`, `sql.rs` |
| `src/bin/jackin_preview/` | `main.rs`, `app.rs` (2662 lines), `screens/mod.rs`, `arbiter.rs`, `clock.rs`, `scenario.rs`, `rain.rs`, `app_tests.rs`, `app_tests_chrome.rs` | `screens/modals.rs` (≈2400 lines, read 1–2400 in three passes), `screens/{manager,capsule,cockpit,inspect,config,editor,settings,accounts,usage,prelude}.rs` (headers + struct definitions only) | `domain/*`, `sim/*` |
| library | `lib.rs`, `runtime.rs`, `core/{event,focus,hit,id}.rs`, `ui/ctx.rs`, `widgets/{mod,brand}.rs` | `theme.rs` (1–200), `widgets/grid.rs` (1–550), `widgets/dialog.rs` (1–130), `ui/layout.rs` (1–90) | `core/{scroll,text}.rs`, `ui/{popup,text}.rs`, 27 of 31 widget modules |

Sections marked **[FACT]** are quoted or directly read. Sections marked **[INFERENCE]** are my judgement from the read evidence.

---

## 1. Per-application architecture

### 1.1 showcase

**[FACT] Shell.** `src/bin/showcase/main.rs:58-66` parses `--color`/`--page`, builds `Theme::for_level(level)`, constructs `App::new(theme)`, optionally `app.goto(page)`, then calls `junie_tui::runtime::run(&mut app)`. `main.rs:68-81` implements `junie_tui::runtime::Application` for `App` by forwarding `handle`, `render`, `should_quit`, `tick_interval`.

**[FACT] Event loop.** `src/runtime.rs:106-153`. `terminal.draw(|f| app.render(f))` when `dirty`; `event::poll(wait)` with `wait = tick_interval - elapsed`; inner drain loop reads all queued events (`runtime.rs:117-137`); `Input::Tick` delivered when `last_tick.elapsed() >= interval` (`runtime.rs:139-151`); a final frame is drawn if a tick-driven transition set `should_quit` (`runtime.rs:145-148`). `TerminalSession` (`runtime.rs:35-95`) owns raw mode, alt screen, mouse capture, bracketed paste, DECAWM, and installs a panic hook.

**[FACT] App struct.** `src/bin/showcase/app.rs:194-218`:
```
theme, pages: Vec<Box<dyn Page>>, page: PageId, nav_cursor: usize,
focus: Focus, ring: FocusRing, hits: HitRegistry,
hover: Option<WidgetId>, pressed: Option<WidgetId>, mouse: Option<Position>,
hover_suppressed: bool, dialog: Option<Dialog>, inspector: bool,
size, tick, last_key, status, flash: Option<(WidgetId, Instant)>, quit,
nav_areas: Vec<Rect>, layout: ShellLayout, saved_focus: Option<WidgetId>
```

**[FACT] Pages.** 22 pages enumerated by `PageId` (`app.rs:29-52`) and `NAV_ENTRIES` (`app.rs:60-171`), constructed eagerly as `Box<dyn Page>` in `App::new` (`app.rs:238-268`). The `Page` trait is `pages/mod.rs:102-117`: `title`, `blurb`, `render(area, buf, ctx)`, `handle(&PageEvent, &mut PageCtx) -> Outcome`, `hints(Option<WidgetId>) -> Vec<Hint>`, `editing()`, `animating()`.

**[FACT] Component construction/storage.** Each page owns concrete widget structs as plain fields, with ids derived from a module-level `const ID: WidgetId` — e.g. `buttons.rs:12` + `buttons.rs:23-33` (`Vec<Button>` with `ID.child(0..8)`), `forms.rs:43-56` + `59-82` (`TextInput`, `TextArea`, `RadioGroup`, `Checkbox` ×2, `Toggle` ×2, `Button` ×2 keyed by `ID.sub("name")` etc.), `settings.rs:51-69` (tabs + form + editable table + list + 5 buttons).

**[FACT] Render.** `app.rs:707-736`: builds a fresh `HitRegistry` and `FocusRing` every frame, constructs `RenderCtx::new(&theme, interaction, &mut hits, &mut ring)`, calls `self.draw(...)`, then swaps in the new registries and reconciles focus (`app.rs:721-732`), finally `frame.set_cursor_position(pos)` (`733-735`). `draw` (`781-800`) paints canvas, header, sidebar, main, optional inspector, footer, then the dialog last.

**[FACT] Event routing.** `App::handle` (`app.rs:348-372`) → `on_key` (`445-555`) / `on_mouse` (`576-694`) / `on_tick` (`374-397`). Hit testing happens in `on_mouse` (`583`, `591`, `604`, `628`, `669`, `685`); the resolved `WidgetId` is packaged into a `PageEvent` (`pages/mod.rs:38-70`) and dispatched via `App::dispatch` (`399-415`), which hands the page a `PageCtx { focus: &mut Focus, ring: &FocusRing, requests: Vec<Request> }` (`pages/mod.rs:79-83`) and then applies `Request::OpenDialog` / `Request::Status` (`app.rs:409-413`).

### 1.2 tablepro

**[FACT] Shell.** `src/bin/tablepro/main.rs:57-90`, same `Application` impl shape. `--connect NAME` resolves an index and calls `app.connect(i)` (`main.rs:61-73`).

**[FACT] App struct.** `src/bin/tablepro/app.rs:118-142`: `theme, screen: Screen, connections: ConnectionsScreen, workbench: Option<Workbench>, history: History, focus, ring, hits, hover, pressed, hover_suppressed, modal: Option<Modal>, size, tick, status, flash, quit, saved_focus, too_small, committing: Option<u32>, scope: usize, switcher_targets: Vec<SwitchTarget>`.

**[FACT] Screens.** Exactly two (`app.rs:86-90`): `Connections`, `Workbench`. Overlays are one-deep: `Modal::{Dialog, Picker(PickerKind, Picker, Option<SwitcherIndex>), Filter(FilterEditor)}` (`app.rs:111-116`).

**[FACT] Workbench.** `workbench.rs:67-87` owns `connection`, `catalog`, `schema`, `explorer: TreeView`, `explorer_filter: TextInput`, `explorer_visible`, `maximized`, `strip: Tabs`, `tabs: Vec<WorkTab>`, `active`, `query_counter`, `pending_loads`, `open_objects`, `current_object`, `pending_run`, `closed`. `WorkTab` is a closed enum of `Table(TableTab) | Query(QueryTab) | History(HistoryTab)` (`workbench.rs:36-41`).

**[FACT] Tab bodies.** `tabs.rs:367-385` `TableTab` (mode tabs, `DataGrid`, `ChipBar`, filters, sort, structure `Tabs` + `DataTable` + `ScrollPanel` DDL); `tabs.rs:951-966` `QueryTab` (`CodeEditor`, `Vec<ResultSet>`, result `Tabs`, `Running`, `Completion`, `Split`); `tabs.rs:2266-2278` `HistoryTab` (`ListBox`, `CodeEditor` detail, `TextInput` search, three `Button`s, `Split`).

**[FACT] Event routing.** `handle` (`app.rs:229-272`) → `on_key` (`491-582`) which first runs `workbench_chord` (`584-770`, ~20 global chords), then delegates to the screen with a `Cx { focus, ring, requests }` (`app.rs:65-84`), then applies `Request`s (`385-489`), then falls back to global keys (`557-581`). Mouse: `on_mouse` (`1858-2049`) hit-tests, focuses if `self.ring.contains(id)` (`1902-1904`), routes Up to modal / strip ids / screen (`1936-2018`).

**[FACT] Render.** `app.rs:2090-2116` mirrors showcase; `draw` (`2118-2187`) paints strip (`2189-2282`), screen body, footer (`2284-2335`), then the modal (`2165-2186`).

### 1.3 jackin-preview

**[FACT] Shell.** `src/bin/jackin_preview/main.rs:106-113`: builds `App::for_scenario(scenario, motion, frame, theme)`, drains stale input (`runtime::drain_pending_input`, `runtime.rs:158-165`), then `runtime::run`.

**[FACT] Routes.** `app.rs:52-65`: `Intro, Manager, Prelude, Editor, Settings, Accounts, Usage, Cockpit, Handoff, Capsule, Outro`. Per-route tick cadence `Route::tick_ms` (`app.rs:68-80`): 33 ms for Intro/Outro/Handoff/Cockpit, 80 ms for Capsule, else 80/200.

**[FACT] Screens.** `Screens` struct `app.rs:90-100` holds `manager`, `accounts`, `usage` eagerly, and `settings/editor/prelude/cockpit/capsule` as `Option<…>` created on navigation (`app.rs:1981-2078`). `Screens::get_mut/get` (`102-129`) maps a `Route` to `&mut dyn Screen`; `Intro/Outro/Handoff` have **no** screen (they are rendered by `rain.rs`).

**[FACT] Screen trait.** `screens/mod.rs:231-328` — the richest of the three apps: `on_key`, `on_click`, `on_double_click`, `on_drag`, `on_secondary`, `on_press`, `on_release`, `on_wheel`, `on_paste`, `on_tick`, `on_msg`, `on_modal`, `picker_items`, `form_changed`, `render`, `hints`, `crumb`, `strip_right`, `is_editing`, `animating`, `enter`, `primary_focus`, `on_esc_top`.

**[FACT] Modal stack.** `app.rs:83-88` `ModalEntry { modal, tag: ModalTag, owner: Route, saved_focus }`; `app.rs:142` `modals: Vec<ModalEntry>`. `push_modal` (`1229-1253`) computes the initial focus from a match over 9 modal variants and sets `capsule.dialog_open = true`; `pop_modal` (`1255-1266`) restores `saved_focus`. Results are delivered to the *owning route's* screen via `deliver` (`1268-1281`) with a `ModalTag` (`screens/mod.rs:40-63`).

**[FACT] Requests.** `screens/mod.rs:180-191`: `Status, Error, Open(Box<Modal>, ModalTag), Close, Go(Go), Copy, Help, WithForm(Box<dyn FnOnce(&mut FormDialog)>)`. `Go` (`134-178`) is a 12-variant navigation command applied by `App::go` (`app.rs:1981-2174`).

**[FACT] Render.** `app.rs:2250-2278` mirrors the other two, but focus reconciliation prefers `screen.primary_focus()` (`2268-2271`). `draw` (`2280-2371`) branches: Intro → `rain::render_intro`, Outro → `rain::render_outro`, Handoff → `draw_frame` of Cockpit/Capsule plus `rain::dim_buffer` under `ctx.inert = true` (`2328-2344`), else `draw_frame` + top modal + a second footer pass (`2345-2369`).

---

## 2. Findings per goal §7 search target

Counts are **lower bounds over files read in full** (see §0).

### 2.1 Direct `Focus` / `FocusRing` manipulation

| App | Sites (≥) | Notes |
|---|---|---|
| showcase | **68** (`app.rs` 21, pages 47) | `PageCtx` hands every page `&mut Focus` + `&FocusRing` (`pages/mod.rs:79-83`) |
| tablepro | **72** (`app.rs` 31, `workbench.rs` 20, `tabs.rs` 6, `connections.rs` 15) | `FilterEditor` key/click handlers take `&mut Focus, &FocusRing` directly |
| jackin | **≥ 46** in files read (`app.rs` 24, `modals.rs` ≥ 18, `inspect.rs` 4) | `CustomModal::on_key` signature *requires* `focus: &mut Focus, ring: &FocusRing` (`screens/mod.rs:67`) |

Worst 5 per app:

*showcase*
1. `app.rs:483-487` — `self.focus.set(None); self.focus.next(&self.ring); if self.focus.is(NAV) { self.focus.next(&self.ring); }` (hand-written "skip the sidebar" traversal).
2. `app.rs:721-732` — post-render focus reconciliation split into two branches by `dialog.is_none()`.
3. `forms.rs:100-104` — `cx.focus.focus(self.name.id)` / `cx.focus.focus(self.reviewer.id)` to move focus to the first invalid field.
4. `settings.rs:393,398,403,408,432,436,455` — seven `cx.focus.focus(id)` calls inside one `PageEvent::Click` arm.
5. `sidebars.rs:162-163` — an *application* control calls `ctx.ring.register(self.id)` itself.

*tablepro*
1. `app.rs:1648-1736` `filter_key(f, key, focus: &mut Focus, ring: &FocusRing)` — a screen-level function that owns Tab/BackTab traversal (`1665-1677`, `1725-1735`).
2. `app.rs:2051-2086` `filter_click(f, id, pos, focus: &mut Focus)` — same for the mouse.
3. `app.rs:1548-1602` — `SwitchTarget` handling calls `self.focus.focus(pf)` in six different arms.
4. `workbench.rs:1012-1027` — `cx.focus.focus(self.strip.id)` then `cx.focus.focus(pf)` on tab activation.
5. `connections.rs:465-467, 519, 668, 706-710, 733` — form submit/cancel/validate move focus by hand.

*jackin*
1. `app.rs:1229-1253 / 1255-1266` — `push_modal`/`pop_modal` implement save/restore focus for a 9-variant modal enum by hand.
2. `modals.rs:280-393` `FileBrowser::on_key` — `focus.focus(self.list.id)` (295), `focus.prev/next(ring)` (302-308), `focus.focus(self.path.id)` (327, 349) and a terminal `Tab`/`BackTab` fallback (382-389).
3. `modals.rs:1074-1233` `FormDialog::on_key` — `focus.next/prev(ring)` at 1085-1091 and 1106-1108, plus a hand-written Left/Right button ring at 1213-1229.
4. `modals.rs:646-697` `ChoiceDialog::on_key` — Tab/BackTab plus a modular Left/Right button ring (683-694).
5. `app.rs:2266-2274` — focus reconciliation with a `primary_focus()` preference and a `.filter(|p| ring.contains(*p))` guard.

### 2.2 Direct `HitRegistry` use

**[FACT]** The registry is a public app-owned field in all three apps: `showcase/app.rs:201`, `tablepro/app.rs:126`, `jackin/app.rs:145`.

*showcase* — 12 sites: `app.rs:583,591,604,628,669,685` (`hits.hit` / `hit_scroll`), `711,721`, `965-966,980` (`hits.area_of` for the inspector), `1014-1016` (`hits.len`), plus `app.rs:924` `ctx.hits.register_scroll(NAV, area)`.

*tablepro* — 10 sites in `app.rs` (`1864,1872,1889,1908,2036,2094,2104`) **plus 7 direct registrations from application render code**: `app.rs:2376` `ctx.hits.register(WidgetId::of("filter-editor"), area)` and `app.rs:2462-2467` — six explicit `ctx.hits.register(...)` calls re-registering the filter editor's own controls *above* the modal surface.

*jackin* — 9 sites in `app.rs` plus **≥ 5 blocks of manual re-registration inside modal renderers**, all of which exist only to re-assert hits after `ctx.begin_modal()`:
- `modals.rs:94` `ctx.hits.register(WidgetId::of("modal.surface"), area)`
- `modals.rs:551-557` (FileBrowser: path, 3 buttons, optional checkbox)
- `modals.rs:774-781` (ChoiceDialog: each radio option + each button)
- `modals.rs:1491-1512` and `1530-1534` (FormDialog: every field, every action, cancel, save, and an open select's options)
- `modals.rs:2270-2278` (InfoDialog: actions, close, every visible prop row)

**[INFERENCE]** This "render, then re-register hits by hand" pattern is a direct consequence of `RenderCtx::begin_modal()` (`ui/ctx.rs:126-131`) pushing a barrier *after* the children have already registered. Any new overlay model must make barrier ordering automatic, or these five blocks reappear.

### 2.3 `.owns(...)` / `.locate(...)` chains (incl. `scrollbar::id_for`)

| App | `locate*` | `owns` | `scrollbar::id_for` | `option_id`/`bar_ids` loops | total ≥ |
|---|---|---|---|---|---|
| showcase | 20 | 16 | 26 | 4 | **66** |
| tablepro | 12 | 18 | 18 | 3 | **51** |
| jackin (read files only) | ≥ 4 | ≥ 3 | ≥ 4 | ≥ 2 | **≥ 13** |

Worst 5 per app:

*showcase*
1. `scrolling.rs:131-167` — four separate arms (`Click`, `Drag`, `Wheel`) each looping over `[&mut self.prose, &mut self.log]` comparing `p.id == *id || scrollbar::id_for(p.id) == *id`.
2. `settings.rs:390-495` — 14 id comparisons in `Click`, 2 in `Drag`, 3 in `Wheel`.
3. `taskrunner.rs:412-463` — `tree.locate` returning `(row, toggle)`, then `on_click_toggle` vs `on_click_row`, plus four `scrollbar::id_for` comparisons.
4. `editable.rs:137-162` — `locate_header` → `on_click_header`; `locate` → `on_click_cell(row, col.unwrap_or(cursor_col), pos)`; then scrollbar; then `owns` for wheel.
5. `panels.rs:176-209` — three arms iterating `self.panels()` to match `scrollbar::id_for(p.id)`.

*tablepro*
1. `workbench.rs:1035-1094` — the `WorkTab::Table` click arm chains `mode_tabs.locate` → `structure_tabs.locate` → `structure.owns` + `locate_header` + `locate` → `scrollbar::id_for(ddl)` → `chips.owns` → `grid.owns` + `bar_ids().contains`.
2. `workbench.rs:759-774` — `for bid in t.grid.bar_ids() { if f == bid { … } }` then `if f == t.grid.id { … }`.
3. `tabs.rs:1437-1495` `QueryTab::on_click` — completion `owns`, editor id, `scrollbar::id_for(editor)`, `result_tabs.owns`, then a match on `ResultBody` with `g.owns(id)` / `tree.locate(id)` / `scrollbar::id_for(raw.id)`.
4. `workbench.rs:1165-1194` `on_wheel` — 8 ownership probes across explorer / grid / structure / ddl / query / history.
5. `connections.rs:823-834` — `for sel in [&mut form.engine, &mut form.group] { if sel.owns(id) … }` and `for rg in [&mut form.env, &mut form.safe] { for i in 0..rg.options.len() { if rg.option_id(i) == id … } }`.

*jackin (read files)*
1. `modals.rs:407-419` — `self.list.locate(id)` → manual "was this row already the cursor?" double-click emulation → `scrollbar::id_for(self.list.id)`.
2. `modals.rs:1235-1310` `FormDialog::on_click` — a `for f in self.fields` loop matching `Input.id == id`, `Select.owns(id)`, `Check.id == id`, `Radio.option_id(i) == id`, `Chooser.button.id == id`.
3. `modals.rs:2129-2172` `InfoDialog::on_click` — `props.locate(id)`, two `scrollbar::id_for` comparisons, action loop, close.
4. `app.rs:1644-1657` — `self.host_menu.owns(id)` then `host_menu.on_click(id)` then a second `host_menu.is_open()` close-on-outside branch.
5. `modals.rs:1530-1534` — reads back `ctx.hits.area_of(s.option_id(k))` and re-registers it, i.e. an ownership lookup used purely to fix z-order.

### 2.4 Manually derived child IDs

**[FACT]** `WidgetId` supports `of(&str)`, `child(usize)`, `sub(&str)` (`core/id.rs:25-40`). Applications derive children constantly.

*showcase* — ≥ 30 sites. Worst:
1. `app.rs:696-698` `fn nav_index_at(&self, id) -> Option<usize> { (0..NAV_ENTRIES.len()).find(|&i| NAV.child(i) == id) }` — a linear scan reversing an id back into an index, called from three mouse arms (`616`, `658`, `688`).
2. `buttons.rs:24-33` — nine buttons keyed `ID.child(0..8)`, with behaviour keyed by the same index (`buttons.rs:50` `if i == 8`).
3. `sidebars.rs:36-42` — `item_id(i) = self.id.child(i)` plus `locate(id)` doing the reverse scan.
4. `forms.rs:308-312` / `settings.rs:406-411` — `for i in 0..group.options.len() { if group.option_id(i) == id { … } }`.
5. `app_tests.rs:519` — the test suite itself reconstructs `WidgetId::of("scrolling").sub("list")` to find the list area.

*tablepro*
1. `app.rs:1841-1852` — `for i in 0..w.tabs.len() + 1 { if id == CLOSE_DIALOG.child(i) { … } }`: a dialog is matched back to a tab by *index*.
2. `workbench.rs:447-449` — `ID.sub("tab").child(self.tabs.len() + self.query_counter + 1000)` (an id derived from mutable counters + current length).
3. `workbench.rs:473` — `ID.sub("query").child(self.query_counter)`.
4. `tabs.rs:2062, 2122, 2132` — `WidgetId::of("result").child(n)`, `WidgetId::of("plan").child(n)`, `WidgetId::of("plan-raw").child(n)` where `n = result_counter + 1`.
5. `app.rs:1401-1422` — `let f = WidgetId::of("filter-editor"); … f.sub("col"), f.sub("op"), f.sub("value"), f.sub("value2"), f.sub("apply"), f.sub("cancel")`.

*jackin*
1. `app_tests.rs:309, 319, 322, 340, 381, 648` — tests address `crate::screens::accounts::FORM.sub("save")` / `.sub("provider")` / `.sub("op")`, i.e. the **test harness depends on the app's internal child-ID composition**.
2. `app_tests.rs:567-571, 649, 1182-1186` — `WidgetId::of("editor.cfg").sub("form").sub("save")` reconstructed in three tests.
3. `modals.rs:148-158` — `FileBrowser::new` derives `id.sub("path")`, `id.sub("list")`, `id.sub("ro")`, `id.sub("git")`, `id.sub("cancel")`, `id.sub("choose")`.
4. `modals.rs:591-595, 944-945, 1989` — same pattern in `ChoiceDialog`, `FormDialog`, `InfoDialog`.
5. `app.rs:46-50, 831, 1207, 1223` — bare `WidgetId::of("strip.help")`, `WidgetId::of("host.about")`, `WidgetId::of("help")`, `WidgetId::of("quit")` as free-floating global names.

### 2.5 Manual pressed / hover routing

**[FACT]** All three apps reimplement the same press/release state machine.

*showcase* `app.rs:576-694`:
- `MouseKind::Move` (579-589) — clears `hover_suppressed`, recomputes `hover`, returns `Changed` only if it differs *or* suppression was lifted.
- `MouseKind::Down` (603-625) — `self.pressed = hit; self.hover = hit;` then focus if `ring.contains(id)`.
- `MouseKind::Up` (626-667) — `let pressed = self.pressed.take(); if pressed != Some(id) { return Changed }` (the "valid completed click" rule), then `self.flash(id)` and dispatch.
- `Interaction::pressed` (`ui/ctx.rs:38-40`) — `(pressed == Some(id) && hover == Some(id)) || flash == Some(id)`.
- Keyboard press feedback is a separate `flash` timer: `app.rs:326-332`, `494-505`, `700-703`, `380-385`.

*tablepro* `app.rs:1858-2049` — identical structure (`1888-1906` Down, `1907-1934` Up with `pressed != Some(id)` at `1932`, `1935` flash) plus a drag arm that forwards `(pressed, pos)` to `workbench.on_drag` (`1871-1887`).

*jackin* `app.rs:1534-1753` — the same, **plus**:
- an explicit `on_press` / `on_release` screen hook pair (`1586-1599`, `1607-1624`) so screens can anchor text selection;
- a hand-rolled double-click detector: `last_click: Option<(WidgetId, i64)>` (`app.rs:159`), `1636-1640` `lid == id && now - at < 500`, then `on_double_click` with a fallback to `on_click` (`1685-1694`).

Worst offender across the three: **`showcase/pages/terminal.rs:337-349`** — the page reconstructs a press that the shell never delivered:
```rust
if *pressed == self.term.id {
    // the page sees no press event: anchor on the first drag
    let o = self.term.on_drag(*pos);
    if o == Outcome::Ignored { self.term.on_click(*pos); return self.term.on_drag(*pos); }
```

### 2.6 Raw Ratatui `Style` / `Color` decisions in application code

**[FACT] Structural cause.** Every widget `render` takes a caller-supplied background colour. Representative signatures used by apps: `buttons.rs:93` `self.buttons[i].render(r, buf, ctx, bg)`; `forms.rs:143` `self.name.render(rect, buf, ctx, bg)`; `lists.rs:106-116`; `workbench.rs:1252-1265`; `modals.rs:498-530`. The `bg` is obtained from `Panel::bg(t)` (`buttons.rs:76`, `forms.rs:130`, …) or hard-set to `t.canvas` (`showcase/app.rs:195` equivalent at `chrome.rs:195`, `settings.rs:161`, `workbench.rs:1214`, `capsule` chrome).

Direct `ratatui::style` construction in application code (worst 5 per app):

*showcase*
1. `pages/inputs.rs:65-106` `static_field` — a complete re-implementation of the `TextInput` renderer: `t.field_style(s)`, manual gutter, `Modifier::UNDERLINED` + `underline_color(t.accent)` (86-90), and a **fake cursor cell** `Style::new().bg(t.text_primary)` (96).
2. `pages/buttons.rs:143-176` — re-renders the button state matrix by calling `t.button(*kind, *s, bg)` and `t.gutter(*s, style.bg.unwrap_or(bg), on_accent)` directly and writing `"▎"` + `" Label "`.
3. `pages/overview.rs:110-114` — `fill(buf, Rect::new(x, y, 4, 1), ratatui::style::Style::new().bg(*color))` for token swatches; `overview.rs:145` `.add_modifier(Modifier::BOLD)`.
4. `pages/panels.rs:21-33` and `scrolling.rs:21-23` — `fn log_style(t: &Theme, line: &str) -> ratatui::style::Style` passed as a line-styling function pointer into `ScrollPanel::render`.
5. `pages/editable.rs:100-107` — `t.on(t.text_primary).fg(t.canvas)` to draw a legend swatch for the cell cursor.

*tablepro*
1. `app.rs:2337-2468` `draw_filter_editor` — a modal drawn entirely by hand: dim loop over `dim.positions()` writing `t.backdrop(c.style())` and clearing modifiers (2351-2357); `fill(buf, area, Style::new().bg(bg))` (2370); a raw `ratatui::widgets::Block` with `Borders::ALL` + `BorderType::Rounded` + `t.border(true).bg(bg)` (2371-2375).
2. `tabs.rs:1816-1851` — plan-tree metric overlay: reads the **already-rendered cell background** `let bgc = buf[(cols_x, y)].bg;` then re-writes text over the tree's own output.
3. `tabs.rs:2434-2442` — history list annotation: `let st = buf[(l.x + 1, y)].style();` then `st.fg(t.error).add_modifier(Modifier::BOLD)`.
4. `tabs.rs:855-862` — DDL line styler closure returning `t.primary().add_modifier(Modifier::BOLD)` / `t.secondary()` / `t.primary()`.
5. `connections.rs:1076-1091, 1244-1255` — error and test-result lines styled with `t.error_fg().bg(bg).add_modifier(Modifier::BOLD)`.

*jackin*
1. `rain.rs:59-86` — `ladder_color` + `style()` map an app-local `Tone` enum onto `t.text_ghost/faint/muted/secondary/primary` and `t.accent/accent_hover/accent_pressed`.
2. `rain.rs:102-172` `dim_buffer` — reads each cell's `fg`, finds its position in a hard-coded 5-token ladder, and also branches on `fg == t.accent || fg == t.success || fg == t.focus`, `fg == t.error || fg == t.warning`, `bg == t.canvas / t.accent / t.accent_bg / t.surface`.
3. `modals.rs:74-79` — `fill(buf, area, Style::new().bg(bg))` + raw `Block` with rounded borders, repeated for every modal family through `modal_frame`.
4. `modals.rs:741, 764, 1444, 2224` — `Style::new().fg(t.tone(*tone)).bg(bg)` for note/intro/option lines.
5. `manager.rs:28`, `inspect.rs:23`, `capsule.rs:32`, `accounts.rs:30`, `usage.rs:19`, `config.rs:26` — every large screen imports `ratatui::style::{Modifier, Style}` directly.

### 2.7 Hand-built dialogs, menus, tabs, forms, sidebars, status bars, scrollbars

*showcase*
| Control | Location | Duplicates |
|---|---|---|
| Navigation sidebar | `app.rs:868-926` (render) + `app.rs:461-492` (keys) + `696-698` (hit→index) | `widgets::list::ListBox` |
| `NavList` (page-level sidebar) | `sidebars.rs:26-165` | `ListBox` + sections |
| Header bar with clickable actions | `app.rs:827-866` | `widgets::segments` / `statusbar` |
| Footer hint row | `app.rs:1018-1077` | `widgets::hintbar` + `widgets::keyhint` (both exist and are used by the *other* two apps) |
| Static field renderer | `inputs.rs:65-106` | `widgets::input::TextInput` |
| Button state matrix | `buttons.rs:143-176` | `widgets::button::Button` |
| "Terminal too small" screen | `app.rs:802-825` | — (duplicated verbatim in the other two apps) |
| Inspector panel | `app.rs:948-1012` | `widgets::props` |

*tablepro*
| Control | Location | Duplicates |
|---|---|---|
| `FilterEditor` modal | struct `app.rs:99-109`; build `1368-1433`; keys `1648-1736`; clicks `2051-2086`; render `2337-2468` | a composed dialog/form |
| Identity strip | `app.rs:2189-2282` | `widgets::statusbar::StatusBar` (unused by tablepro) |
| Footer | `app.rs:2284-2335` (via `keyhint::render`) | `widgets::hintbar::HintBar` (unused by tablepro) |
| Grid status line with priority drop | `tabs.rs:794-838` — a bespoke `while … { remove lowest priority }` loop | `widgets::segments` priority logic |
| Plan-tree metric columns | `tabs.rs:1774-1852` | a table with a tree column |
| Too-small screen | `app.rs:2121-2145` | showcase `app.rs:802-825` |

*jackin*
| Control | Location | Duplicates |
|---|---|---|
| `modal_frame` (dim + barrier + frame + title + meta) | `modals.rs:36-96` | `widgets::dialog` chrome |
| `FileBrowser` | `modals.rs:117-563` | list + input + buttons + checkbox composition |
| `ChoiceDialog` | `modals.rs:569-783` | `Dialog` + `RadioGroup` |
| `FormDialog` | `modals.rs:916-1541` | a real form component (scroll, clip, focus-follow, open-select z-order) |
| `OpFlow` | `modals.rs:1555-1943` | a multi-step wizard over `Picker` |
| `InfoDialog` | `modals.rs:1956-2280` | `Dialog::facts` + `PropsList` + scrollable detail |
| `HelpOverlay` | `modals.rs:2284-2400+` | a scrollable multi-column key reference |
| Host menu bar + action dispatch | `app.rs:699-813` | `widgets::menu::MenuBar` (used) + a bespoke label→keypress dispatcher |
| Identity strip / footer | `app.rs:2433-2516` / `2518-2642` | `segments` + `hintbar` (both used, but the layering logic is app-local) |
| Too-small screen | `app.rs:2283-2310` | showcase/tablepro copies |

**[FACT] The `run_host_menu` anti-pattern.** `jackin/app.rs:754-813`: menu items are executed by **re-synthesising key presses** through `self.handle(Input::Key(Key { code, mods }))` — e.g. `"New workspace…" => return self.handle(key(KeyCode::Char('n'), NONE))`. The comment at `752-753` states this is deliberate ("so menus and keys can never disagree").

### 2.8 Custom key handling that duplicates component behaviour

*showcase* — 5 worst:
1. `app.rs:461-492` — sidebar keys `Up/k`, `Down/j`, `Home/g`, `End/G`, `Enter/Space/Right/l`.
2. `sidebars.rs:48-80` — `NavList::on_key` re-implements list movement **including skipping disabled items** (52-70).
3. `editor.rs:469-479` — `Ctrl+Space` (and `KeyCode::Null`) to force-open completion; `Ctrl+R` to run.
4. `chrome.rs:277-291` — `m` opens a context menu at a computed row rect; `F10` focuses the menubar.
5. `taskrunner.rs:374-381` — bare `r` runs the pipeline, guarded by three `!cx.focus.is(...)` checks.

*tablepro* — 5 worst:
1. `app.rs:584-770` `workbench_chord` — ~20 chords (`Ctrl+R/F5/Alt+R/Ctrl+X/Alt+X/Ctrl+T/Ctrl+W/Ctrl+O/Ctrl+P/Ctrl+G/Ctrl+Y/Ctrl+B/Ctrl+L/Ctrl+D/Ctrl+S/Ctrl+F/z/[/]/Ctrl+↑↓`) resolved before any component sees the key.
2. `app.rs:1665-1677` + `1725-1735` — Tab/BackTab implemented twice inside one modal handler.
3. `workbench.rs:891-926` — history list keys `/`, `c`, `s`, `Enter`, `r`, `y` intercepted before `ListBox::on_key`.
4. `tabs.rs:1379-1397` — result-tab keys `p`/`.`/`x` intercepted before `Tabs::on_key`.
5. `workbench.rs:634-668` — explorer `/` (focus the filter), `Enter` (open a table instead of folding), `F5`/`r` (refresh) all bypass `TreeView`.

*jackin* — 5 worst:
1. `modals.rs:316-332` — `FileBrowser` intercepts `Enter/Right/l`, `Left/h/Backspace`, `Space/s`, `g` before delegating to `ListBox::on_key` (333).
2. `modals.rs:1204-1231` — `FormDialog` re-implements Enter-activates-save and a Left/Right ring across `actions + cancel + save`.
3. `modals.rs:2093-2126` — `InfoDialog` handles `Esc/q/Enter/Tab/BackTab/y/Down/j/Up/k` itself.
4. `modals.rs:2315-2339` — `HelpOverlay` re-implements `j/k/PageUp/PageDown` scrolling.
5. `app.rs:593-628` — host-level `F10`, `?`, `u`, `c`, `s`, `Ctrl+Q` gated on `host && !editing`.

### 2.9 Reusable interaction logic embedded in screens

Ranked by how clearly it belongs in the library (**[INFERENCE]** on the disposition, **[FACT]** on the location):

1. `jackin/screens/modals.rs:916-1541` — `FormDialog`: scrollable field list with focus-follow scrolling (`1357-1371`), per-field clipping (`1379-1390`), open-select z-order fix (`1514-1539`), a change/dirty event stream (`FormEvent`, `905-914`). Nothing in it is Jackin-specific.
2. `jackin/screens/modals.rs:36-96` — `modal_frame`: backdrop dim + barrier + rounded frame + title/meta. Every overlay in Jackin funnels through it; TablePro re-implements it at `app.rs:2345-2377`.
3. `jackin/screens/modals.rs:1956-2280` — `InfoDialog`: props + copy-to-clipboard rows + scrollable detail + action row.
4. `showcase/pages/sidebars.rs:26-165` — `NavList`.
5. `tablepro/app.rs:99-109 + 1368-1433 + 1648-1736 + 2051-2086 + 2337-2468` — `FilterEditor` (a modal form; only the `FilterOp` vocabulary is domain).
6. `tablepro/tabs.rs:794-838` — priority-based status-segment dropping (already exists as `widgets::segments`).
7. `jackin/screens/modals.rs:117-563` — `FileBrowser` (the `World.fs` lookup is domain; list/path/choose is not).
8. `showcase/pages/mod.rs:120-168` — `layout::{caption, rows, columns}`: a `rows(area, &[heights])` splitter and a responsive 2-column helper, re-implemented ad hoc in the other two apps.
9. `showcase/pages/inputs.rs:65-106` — `static_field`.
10. Master-detail + draggable seam, repeated four times: `manager.rs:114-115`, `accounts.rs:96-98`, `inspect.rs:111-112`, `showcase/pages/terminal.rs:104-106`.

---

## 3. Reusable-looking controls hidden in applications

| # | Name | Location | What it does | Library counterpart | Proposed disposition |
|---|---|---|---|---|---|
| 1 | `NavList` + `NavItem` | `showcase/pages/sidebars.rs:16-165` | Sectioned nav list: current item, keyboard cursor, disabled skipping, badges, collapsed icon-only mode, own focus + hit registration | `widgets::list::ListBox` (no sections/collapse) | **move-to-library** as a `NavList`/`ListBox` section+collapse feature |
| 2 | Shell nav sidebar | `showcase/app.rs:868-926`, `461-492`, `696-698` | The same thing again, inside the shell | as above | **compose-from-primitives** (delete once #1 lands) |
| 3 | `static_field` | `showcase/pages/inputs.rs:65-106` | Renders a non-interactive `TextInput` in an arbitrary `VisualState` | `widgets::input::TextInput` | **compose-from-primitives**: needs a "render in state X without owning state" entry point |
| 4 | Button state matrix | `showcase/pages/buttons.rs:143-176` | Renders every `ButtonKind` × `VisualState` | `widgets::button::Button` | **compose-from-primitives** (same need as #3) |
| 5 | Showcase footer hint row | `showcase/app.rs:1018-1077` | Key hints + EDIT badge + right-aligned status, with width budgeting | `widgets::hintbar::HintBar` / `keyhint::render` | **compose-from-primitives** (use the existing widget) |
| 6 | `layout::{caption,rows,columns}` | `showcase/pages/mod.rs:120-168` | Fixed-height row stack; responsive 2-column split that stacks when narrow | none | **move-to-library** as layout primitives |
| 7 | State inspector panel | `showcase/app.rs:948-1012` | Key/value debug readout of focus/hover/pressed/ring/hits | `widgets::props` | **compose-from-primitives**; the *data source* is a library debug API |
| 8 | `FilterEditor` | `tablepro/app.rs:99-109, 1368-1433, 1648-1736, 2051-2086, 2337-2468` | Modal form: 2 selects, 2 inputs, live SQL preview, apply/cancel, own focus ring | none (no composed-dialog API) | **compose-from-primitives** once a composed dialog exists; keep `FilterOp` in TablePro |
| 9 | Status-segment priority dropper | `tablepro/tabs.rs:794-838` | Drops lowest-priority parts until the joined string fits | `widgets::segments` | **compose-from-primitives** (reuse, delete the copy) |
| 10 | Plan-tree metric columns | `tablepro/tabs.rs:1774-1852` | Right-aligned numeric columns overlaid on a `TreeView`, reading back cell bg | none | **move-to-library** as tree-row trailing columns; the metrics stay domain |
| 11 | TablePro identity strip | `tablepro/app.rs:2189-2282` | Left/right prioritised segments with clickable ids | `widgets::statusbar::StatusBar` | **compose-from-primitives** |
| 12 | `modal_frame` | `jackin/screens/modals.rs:36-96` | Dim + `begin_modal` + rounded elevated frame + title/meta + surface hit | `widgets::dialog` internals (private) | **move-to-library** as the public overlay surface primitive |
| 13 | `FileBrowser` | `jackin/screens/modals.rs:117-563` | Path input + directory list + read-only checkbox + Git-URL mode + resolve spinner | none | **compose-from-primitives**; the `World.fs`/`github` lookups stay domain |
| 14 | `ChoiceDialog` | `jackin/screens/modals.rs:569-783` | Lines + radio group + N buttons + cancel index + option tones + stepper | `Dialog` (closed body enum) | **move-to-library** as a composed-dialog convenience |
| 15 | `FormDialog` + `FormField` + `FieldKindW` | `jackin/screens/modals.rs:787-1541` | Heterogeneous scrollable form with visibility, dirty tracking, chooser fields, note rows, action buttons, open-select z-order | none | **move-to-library** — the single strongest candidate |
| 16 | `OpFlow` | `jackin/screens/modals.rs:1546-1943` | A 4-stage picker wizard with loading/error/retry status | `widgets::picker` + none | **compose-from-primitives**: needs a generic "staged picker" + `PickerStatus`; 1Password stays domain |
| 17 | `InfoDialog` | `jackin/screens/modals.rs:1947-2280` | Read-only facts, copyable rows, scrollable detail, extra actions, error title | `Dialog::facts` (`widgets/dialog.rs:110-130`) | **move-to-library**; supersedes `DialogBody::Facts` |
| 18 | `HelpOverlay` | `jackin/screens/modals.rs:2284-2400+` | Multi-column, round-robin, scrollable key reference | none | **move-to-library** (all three apps have a help surface: `showcase/app.rs:557-574`, `tablepro/app.rs:1195-1213`, `jackin/app.rs:882-1209`) |
| 19 | Host menu bar + `run_host_menu` | `jackin/app.rs:699-813` | Per-route menu definition + label→action dispatch by key synthesis | `widgets::menu::MenuBar` | **keep-domain-specific** for the item list; **move-to-library** the "menu item ↔ command" binding so key synthesis disappears |
| 20 | Master-detail + draggable seam | `manager.rs:114-115`, `accounts.rs:96-98`, `inspect.rs:111-112`, `showcase/pages/terminal.rs:104-106` | `Split` + `Splitter` + two scroll states + narrow-collapse | `ui::layout::Split` + `widgets::splitter` (primitives exist; the composition does not) | **move-to-library** as a `SplitPane` composition |
| 21 | "Terminal too small" screen | `showcase/app.rs:802-825`, `tablepro/app.rs:2121-2145`, `jackin/app.rs:2283-2310` | Centred 4-line minimum-size notice | none | **move-to-library** (three near-identical copies) |
| 22 | `PageCtx` / `Cx` request bus | `showcase/pages/mod.rs:74-98`, `tablepro/app.rs:51-84`, `jackin/screens/mod.rs:180-227` | Screens push `Status/OpenDialog/Go/...` for the shell to apply | none | **move-to-library** as the screen↔shell command channel |
| 23 | `InspectChanges` diff modal | `jackin/screens/inspect.rs:61-89` | Tree + `DiffView` + seam + compact/advanced modes + region focus | `widgets::diff::DiffView` (exists) | **keep-domain-specific**; the tree+diff+seam composition is #20 |

---

## 4. Domain leakage into the library

**[FACT]** Confirmed leakage, from files read:

### 4.1 `src/widgets/grid.rs` — database semantics in a generic grid

| Line | Leak |
|---|---|
| `grid.rs:4` | Module doc: "Sorting and filtering are *requests* to the owner (**server-side**)" |
| `grid.rs:46-47` | `CellValue::Null` renders the literal `"NULL"`; `CellValue::Default` renders `"DEFAULT"` ("Server default (inserted rows)", `grid.rs:35`) |
| `grid.rs:96-99` | `ColumnSpec { primary: bool, nullable: bool, read_only: bool, references: Option<String> }` — primary keys, nullability and foreign-key targets are library concepts |
| `grid.rs:125-140` | Builders `primary()`, `nullable()`, `references()` |
| `grid.rs:152-156` | `RowTotal::Estimated(usize)` — a DB row-count estimate |
| `grid.rs:186-191` | `PendingChanges { cells, inserted, deleted }` — an uncommitted-transaction model |
| `grid.rs:249-251` | `GridEvent::CommitRequested`, `DiscardRequested`, **`PreviewSql`** |
| `grid.rs:253-260` | `GridEvent::FollowReference`, `OpenViewer`, `FilterOnCell`, `OpenFilters`, `ClearFilters` |
| `grid.rs:272` | `default_validator` emits `"{} is NOT NULL"` |
| `grid.rs:307-311` | `CellKind::Id` validation requires a **36-char UUID**, error `"Must be a UUID"` |
| `grid.rs:313-319` | `CellKind::Timestamp` requires `YYYY-MM-DD` |
| `grid.rs:348` | `read_only_reason: Option<String>` — a DB "why can't I edit this" affordance |
| `grid.rs:400-404` | The built-in action bar hard-codes the button label **`"Preview SQL"`** |
| `grid.rs:265` | `pub type Validator = fn(&ColumnSpec, &str) -> Result<CellValue, String>` — a bare function pointer (goal §19 explicitly names this) |

### 4.2 `src/widgets/dialog.rs`

- `dialog.rs:18-28` — `DialogBody` is a **closed 3-variant enum** (`Text`, `Input`, `Facts`), exactly the shape goal §14 forbids.
- `dialog.rs:21-22` — the `Facts` doc comment reads "an optional preformatted block **(SQL)**" — TablePro vocabulary in the library.
- `dialog.rs:23-27` + `30-34` — `AckInput { input, token }`, a typed-acknowledgement concept driven by TablePro's Safe Mode (`tablepro/app.rs:959-970`).

### 4.3 `src/ui/layout.rs`

- `layout.rs:1` — module doc: "Split panes and other layout helpers that **the workbench** composes." `workbench` is TablePro's screen name (`tablepro/workbench.rs:24`).

### 4.4 `src/theme.rs`

- **No** TablePro/Jackin terms observed in `theme.rs:1-200`.
- **[FACT]** The Junie palette is baked into the only theme constructor: `theme.rs:56-95` (private `palette` module with 26 literal RGB constants) and `theme.rs:145-180` `Theme::junie()`. `Theme::for_level` (`183+`) is the only variation and only downgrades colour depth.
- **[FACT]** `theme.rs:167, 172, 177` — `accent`, `focus` and `success` are all `GREEN` in the default theme.

### 4.5 Not leakage

- `widgets/brand.rs:15-21` — `Lockup.text` is supplied by the application; Jackin's mark lives at `jackin/app.rs:45` (`BRAND_MARK`). Clean.
- `core/{event,focus,hit,id}.rs`, `ui/ctx.rs`, `runtime.rs` — no domain terms observed.

**[INFERENCE]** Unverified: 27 of 31 widget modules (`table.rs`, `code.rs`, `completion.rs`, `picker.rs`, `props.rs`, `steps.rs`, `viewport.rs`, `diff.rs`, `statusbar.rs`, `menu.rs`, …) were not read. Given that `grid.rs` and `dialog.rs` both leak, a full text sweep for `sql|SQL|table|schema|Capsule|Construct|jackin|workspace|instance|daemon` across `src/core`, `src/ui`, `src/widgets`, `src/theme.rs`, `src/runtime.rs` is required before the domain-boundary claim can be closed.

---

## 5. Application semantics that must be preserved (regression contract)

### 5.1 showcase — product behaviour

**[FACT]** 22 pages (`app.rs:60-171`), minimum viewport 72×20 (`app.rs:20-21`), sidebar 24 cols wide ≥110 else 19 (`app.rs:753-754`), inspector 30 cols when width ≥100 (`app.rs:755-759`). Global keys: `Tab`/`Shift+Tab`, `q`, `?`, `i`, `[`/`]`, `0`, `Esc` (`app.rs:508-554`); `Ctrl+C` always quits (`app.rs:357-360`). Tick cadence 80 ms animating / 400 ms idle (`app.rs:318-324`). Keyboard activation flashes the focused widget for 140 ms (`app.rs:328, 494-505`). Hover is suppressed after any key press until the pointer moves (`app.rs:367`, `579-588`).

**[FACT] Tests** (`src/bin/showcase/app_tests.rs`) — 20 tests:

| Test | Line | Proves |
|---|---|---|
| `launches_and_renders_shell` | 137 | Shell renders "Junie", "Overview", "Tokens" |
| `every_page_renders_at_representative_sizes_without_panic` | 147 | All 22 pages × 6 sizes (72×20 … 200×60), 12 Tab/Down/Right cycles + inspector toggle, no panic |
| `below_minimum_size_shows_reduced_state` | 171 | 60×15 shows "Terminal too small" / "Need 72×20, have 60×15" and hides page content |
| `resize_recovers_from_too_small` | 180 | `Input::Resize(120,40)` restores the page |
| `tab_traversal_is_deterministic_and_wraps` | 189 | Buttons page ring = nav + 7 enabled buttons (2 disabled skipped); Shift+Tab is the exact reverse |
| `disabled_buttons_are_skipped_and_cannot_activate` | 220 | Click on a disabled button does nothing; hovering it keeps `fg == Theme::junie().disabled` |
| `mouse_click_activates_and_keyboard_enter_activates` | 234 | Mouse and keyboard produce the same activation |
| `hover_and_focus_render_differently` | 245 | Hover ⇒ `bg == surface_overlay`, no BOLD; keyboard focus ⇒ BOLD + `▎` gutter in `focus` colour + hover lift cleared |
| `hit_testing_prefers_rows_over_their_container` | 266 | Hover resolves to a 1-row region, not the table |
| `table_sorts_both_directions_and_clears` | 278 | 3-state header sort, cursor-follows-row scrolling ("4–24 of 24"), numeric sort on Changes |
| `header_click_sorts` | 308 | Mouse header click sorts asc/desc |
| `editable_table_commit_cancel_and_validation` | 322 | EDIT badge, Enter commits, Esc reverts, invalid value keeps editing, Esc leaves edit |
| `input_editing_commit_and_revert` | 359 | Enter edits, Esc reverts, Tab commits **and** advances focus |
| `textarea_scrolls_with_wheel_and_keys` | 387 | Wheel scrolls under the pointer; `ln 28/28` position label |
| `list_scrolling_and_selection` | 404 | Single-select `Chosen:`; multi-select Space toggle; `a` selects all **enabled** (10 of 12) |
| `tree_expand_collapse_and_focus_bar_column_is_stable` | 431 | Left/Right fold; the focus bar column does **not** move with depth |
| `modal_traps_focus_and_restores_it` | 453 | Ring shrinks to the dialog's 2 actions; click-outside cancels; focus restored; `y` answers |
| `prompt_dialog_validates_and_returns_value` | 479 | Empty prompt blocked with "Name cannot be empty"; value returned |
| `form_validation_blocks_submit_and_focuses_first_error` | 499 | `Ctrl+S` blocks and moves focus to the invalid field |
| `scrollbar_click_and_drag_move_the_view` | 514 | Track click jumps; thumb drag scrolls back |
| `keyboard_navigation_between_pages` | 535 | `]`/`[`, sidebar Enter, sidebar click |
| `quit_keys` | 551 | `q` quits; `q` while editing types; `Ctrl+C` always quits |
| `settings_screen_remove_member_flow` | 565 | Tabs → table → destructive dialog focuses Cancel first; `y` removes; count updates |
| `task_runner_animates_and_can_be_cancelled` | 586 | Tick-driven progress; disabled Run skipped in the ring; cancel dialog |
| `color_downgrade_still_renders` | 607 | `ColorLevel::Ansi16` produces `Color::LightGreen` cells |
| `showcase_visual_baseline` | 624 | **Cell-exact digest** (symbol+fg+bg+modifier) of every page at 120×40 and 80×24, excluding the sidebar rect, against `tests/showcase_baseline.txt` |

### 5.2 tablepro — product behaviour

**[FACT]** Two screens; Safe Mode gate with 6 levels (`SafeMode::ALL`, used at `app.rs:1346-1362`); statement classification by "worst statement in the batch" (`app.rs:840-866`); typed-token acknowledgement for deliberate confirmations (`app.rs:959-963`); one-transaction commit with a 5-tick simulated latency (`app.rs:1821`, `339-346`); read-only refusal messages (`app.rs:879`, `1012-1016`); history recording including row-edit statements (`app.rs:1135-1148`); tab strip with `+` new-tab and close confirmation for dirty tabs (`workbench.rs:959-966`); explorer becomes a full-body drawer below 100 columns (`workbench.rs:1221-1243`); pinned result tabs survive a re-run (`tabs.rs:1060-1062`).

**[FACT] Tests** (`src/bin/tablepro/app_tests.rs`) — 21 tests:

| Test | Line | Proves |
|---|---|---|
| `connections_screen_lists_and_connects_with_keyboard` | 125 | Tree lists groups; Enter connects; 14 ticks reach the Workbench; strip shows env + safe token |
| `failed_connection_shows_error_and_retry` | 148 | `ConnectOutcome::Unreachable` ⇒ error text + "Reconnect" |
| `explorer_opens_table_and_grid_navigates` | 165 | Enter on `orders` opens a table tab; grid cursor + horizontal scroll + `cols ` label |
| `sort_and_filter_on_table_tab` | 193 | `s` toggles asc/desc on `created_at`; `f` opens the filter editor prefilled from the cell; applied filter shows `filtered (1)` and every visible status cell is `pending` |
| `structure_view_toggle` | 245 | `Ctrl+D` toggles Data/Structure; columns/indexes/types visible |
| `editor_completion_and_execution` | 266 | Completion opens after `FROM ord`; Enter accepts `orders`; `Ctrl+R` runs; result label `SELECT orders (25)`; history recorded |
| `execution_error_marks_editor_and_result` | 310 | `column "nope" does not exist` + editor diagnostic + `Error 1` tab |
| `cancel_running_query` | 325 | Esc cancels; a cancelled run is **not** recorded in history |
| `explain_opens_plan_tree` | 343 | `Alt+X` ⇒ EXPLAIN ANALYZE tree with Limit/Sort/Seq Scan; tree collapses; `r` shows raw `cost=` |
| `safety_gate_intercepts_dangerous_statement_on_production` | 367 | `DELETE FROM orders` ⇒ facts dialog with "DELETE without WHERE", "every row in orders", "Production", "Type orders to confirm"; a wrong token keeps the confirm button **out of the focus ring**; Esc ⇒ "Cancelled · nothing was executed" |
| `safety_gate_typed_token_executes` | 397 | Correct token arms Execute; the run starts; "rows affected" |
| `read_only_connection_refuses_writes` | 418 | `SafeMode::ReadOnly` refuses without a dialog |
| `silent_level_runs_scoped_writes_but_confirms_destructive` | 432 | `UPDATE` without WHERE runs silently; `TRUNCATE` always confirms |
| `quick_switcher_opens_table` | 458 | `Ctrl+O` fuzzy switcher; Esc clears the query then closes |
| `history_tab_reopens_query` | 477 | `Ctrl+Y`; `/` search; Enter reopens as a new query tab |
| `tab_strip_overflow_and_tab_list` | 494 | ≥12 tabs at 100 cols ⇒ `‹`/`›` overflow marks; `Ctrl+G` tab list |
| `pending_edits_preview_and_save` | 530 | Cell edit ⇒ `• 1 pending`; `p` previews `UPDATE public.orders SET currency = 'EUR'`; `Ctrl+S` ⇒ token ⇒ "Saved 1 change"; pending cleared |
| `safe_mode_picker_changes_level_and_strip` | 569 | `Ctrl+L` picker sets `SafeFull`; strip shows `safe+` |
| `mouse_opens_table_and_switches_tabs` | 580 | Single click ⇒ preview tab; second click promotes it; tab strip click; hover registers |
| `every_screen_renders_at_representative_sizes` | 602 | 5 sizes × a 20-step journey (open table, structure, new query, run, explain, history, switcher, tab list, safe mode, zoom, help); 60×15 ⇒ "Terminal too small" |
| `narrow_terminals_turn_the_explorer_into_a_drawer` | 653 | At 80×24 the explorer covers the body while focused; Tab leaves it and lands in the editor; `0` reopens it; opening a table closes it |
| `acceptance_flow_keyboard_only` | 680 | The full product journey, keyboard only |
| `acceptance_flow_mouse` | 782 | The same journey by mouse; header click sorts; wheel over the explorer scrolls it **without moving focus** |

### 5.3 jackin-preview — product behaviour and deterministic scenarios

**[FACT] Determinism contract.** No wall clock: `clock.rs:1-15` — virtual ms advanced only by ticks; fixed epoch `EPOCH_SECS = 1_788_401_640` (`clock.rs:7`); `Clock::advance` is a no-op when `!running` (`clock.rs:26-30`). `--motion paused` freezes ticks (`main.rs:97-103`, `app.rs:315-320`). Rain randomness is a single stateless mixer seeded by `MOTION_SEED` (`rain.rs:22-37`) plus a seeded xorshift starfield (`rain.rs:276-284`).

**[FACT] Scenarios** (`scenario.rs:4-53`) — 8, all reachable via `--scenario`:

| Scenario | Contract (`scenario.rs`) | Start route (`app.rs:247-280`) |
|---|---|---|
| `first-use` | Zero instances, no workspaces, no accounts | Intro → Manager |
| `returning` | An instance already running: no intro | Manager (`app_tests.rs:129-135` asserts "2 running") |
| `accounts-mixed` | Several providers/accounts, mixed health | Accounts (`app.rs:248`) |
| `launch-running` | Straight into an active cockpit, `LaunchPlan::Clean` | Cockpit |
| `launch-failure` | `LaunchPlan::FailNetwork` | Cockpit → failure |
| `capsule-multi` | Attached Capsule, several tabs and nested panes | Capsule (`jk-7f3a`) |
| `outro-last` | Attached to the last instance; exiting plays the outro | Capsule; with `--frame > 0` jumps straight to Outro (`app.rs:203-214`) |
| `hard-cases` | Long labels, missing daemon data, discovery failure, many rows (>100 roles, `app_tests.rs:1146`) | Manager |

**[FACT] Boundary arbiter contract** (`arbiter.rs:1-11`, tested `arbiter.rs:148-203`): a pending entry claim suppresses a duplicate intro; discovery failure is surfaced and the rich outro withheld (fail closed); exactly one exit token per Construct; every message is idempotent.

**[FACT] Motion contract** (`rain.rs`): intro phrase pacing `P1_LEN == 64` ticks (`rain.rs:488`, asserted `rain.rs:879`); `KNOCK_START`/`WARP_START`/`INTRO_END` derived constants (`491-494`); reduced motion = one static frame then `REDUCED_HOLD = 45` (`495`, `534-542`); skip semantics (phrases → warp → done, `560-570`); outro caption wording `"You were in the Construct for 2 hours 14 minutes"` (`748-755`, asserted `907`); `format_universe_duration` two-largest-units rule (`665-682`).

**[FACT] Tests** — `app_tests.rs` (17) + `app_tests_chrome.rs` (5) + unit tests in `rain.rs` (3), `arbiter.rs` (4), `clock.rs` (2), `scenario.rs` (1):

| Test | File:line | Proves |
|---|---|---|
| `first_use_plays_intro_then_manager_and_no_replay_when_returning` | `app_tests.rs:115` | Intro plays once; skip = phrases→warp→manager; `returning` joins without replay |
| `reduced_motion_and_paused_frames_are_deterministic` | 139 | Reduced shows "Enter Continue"; **two `--frame 282` runs render byte-identical text**; paused frames never advance |
| `manager_navigation_expand_and_detail_focus` | 158 | Tree expand reveals instances; Tab → detail; Esc returns to `manager::TREE`; row click updates the crumb |
| `launch_runs_all_stages_and_hands_off_to_the_capsule` | 179 | Cockpit → Handoff → Capsule; typing echoes in the pane |
| `launch_failure_returns_to_the_construct_when_another_instance_runs` | 203 | "Launch failed" + "Network"; Esc → Manager with "still running" |
| `detach_reconnect_and_final_exit_plays_one_outro` | 219 | `Ctrl+B d` detaches; Enter reconnects; `Ctrl+Q` → unsaved-work choice → Outro with the elapsed caption → quit |
| `still_inside_feedback_when_other_instances_remain` | 251 | Exit with 1 remaining ⇒ "Still inside the Construct", `running_count() == 1` |
| `too_small_state_and_resize_recover` | 264 | 60×18 too small; 80×24 recovers |
| `accounts_register_with_a_1password_reference_and_never_render_the_secret` | 273 | Full OpFlow chain; **`valid-ant01` never appears in the frame**; duplicate-source refusal; provider switch; save; refresh reports "still rate limited" |
| `accounts_plain_key_is_masked_everywhere_and_remove_asks_first` | 359 | Raw key never rendered while typing or after; only the 4-char tail; **`format!("{:?}", a.source)` must not contain the key**; remove asks first |
| `usage_overlay_is_read_only_and_hands_off_to_accounts` | 400 | "Usage · read-only"; `m` hands the selection to Accounts; Esc → Manager |
| `prelude_creates_a_pending_workspace_and_opens_the_editor` | 416 | 5-step chain; Esc rewinds to the previous step **with its state**; pending workspace fields |
| `prelude_refuses_a_duplicate_name_and_cancels_cleanly` | 463 | Duplicate name refused; full rewind ⇒ "Cancelled · nothing created" |
| `editor_edits_count_once_preview_then_saves_and_returns` | 497 | `• 1 change` counted once; leaving asks; save preview lists "1 modified"; async save returns to Manager and persists |
| `editor_env_plain_value_stays_masked_and_can_be_shown` | 538 | Plain env value masked; `m` reveals; new secret stored as `************1234` |
| `settings_trust_toggle_and_failed_save_keep_edits` | 581 | A failed save keeps `• 1 change`; the retry persists |
| `hard_cases_refresh_keeps_last_good_and_help_opens_everywhere` | 618 | Per-route help sections; "broker unreachable" |
| `complete_jackin_flow_keyboard_first` | 646 | **The 40-step product journey** (§34 of the original goal): 5 accounts via 3 credential paths, workspace creation, all 5 editor tabs, launch, build log, Capsule typing, second session, split/zoom/resize, scrollback + mouse selection + double-click word select + `y` copy, palette, capsule Usage, detach/reconnect, second instance, still-inside, final outro |
| `editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on` | 1028 | Per-workspace account enable/disable/prefer with effective-set resolution |
| `manager_launch_picker_hides_agents_without_an_account` | 1110 | Agents without a usable account are **omitted**, not shown disabled |
| `environments_stay_readable_with_a_hundred_roles` | 1137 | >100 roles: only configured role sections render; searchable role picker adds one |
| `cockpit_resolves_every_effective_account_for_the_container` | 1201 | The container receives the workspace's whole effective account set |
| `capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line` | `chrome.rs:22` | Row 0 = menu bar (not the identity strip); row 38 = status bar with `PR #482` + a `━` usage meter; last row = hint bar with `Ctrl+B` |
| `menu_bar_opens_switches_and_runs_an_action` | `chrome.rs:43` | F10 opens; ←→ switches; Esc closes; mouse click on `View` → `Usage` opens the dialog |
| `tab_context_menu_renames_and_closes_by_mouse_and_keyboard` | `chrome.rs:74` | Right-click and `Ctrl+B m` both open the tab menu; rename applies; Close asks |
| `hint_bar_stays_on_the_last_row_across_layers` | `chrome.rs:110` | Layer precedence; **the picker draws no hint row of its own** (`"Enter Choose"` appears exactly once) |
| `inspect_changes_opens_from_the_view_menu_in_both_modes` | `chrome.rs:133` | Diff opens; `m` toggles compact/advanced; `d` toggles unified/review; Esc×2 returns |
| `command_palette_scrolls_with_the_wheel_and_keeps_the_selection` | `chrome.rs:161` | Wheel scrolls rows; wheel-up restores exactly; the selection stays on the first item |

---

## 6. Test-harness facts

**[FACT] Common shape.** All three suites are `#[cfg(test)] mod app_tests;` inside the binary (`showcase/main.rs:4-5`, `tablepro/main.rs:5-6`, `jackin/main.rs:7-11`). Each defines a harness struct holding `{ app: App, term: Terminal<TestBackend> }`, drives the app with real `Input` values, and re-draws after every input.

| Helper | showcase (`app_tests.rs`) | tablepro (`app_tests.rs`) | jackin (`app_tests.rs`) |
|---|---|---|---|
| constructor | `Harness::new(w, h, PageId)` :20 | `H::new(w,h)` :21, `H::connected(w,h)` :28 | `H::new(Scenario, Motion, frame, w, h)` :20 |
| draw | :29 | :41 | :27 |
| key | `key(KeyCode)` :33, `key_mod` :42 | `key` :44, `ctrl` :52, `alt` :60 | `key` :30, `ctrl` :38 |
| typing | `type_str` :48 | `type_str` :68 | `type_str` :46 |
| ticks | (none — `app.handle(Input::Tick)` inline, :591) | `ticks(n)` :73 | `ticks(n)` :51 |
| mouse | `mouse(kind,x,y)` :54, `click` :63 | :79, :87 | :57, :65 |
| resize | via `term.backend_mut().resize` + `Input::Resize` :182-184 | — | `resize(w,h)` :69 |
| text dump | `text()` :68, `row(y)` :80 | `text()` :91 | `text()` :87 |
| search | `find_row` :88, `find` :94 (grapheme-accurate) | `find` :102 | `find` :98 |
| state probes | `focus_bar_x(y)` :112, `count` :120, `focus_area()` :124 | `focus()` :116, `wb()` :119, `wb_query()` :644 | `tab_to(WidgetId)` :75 |

**[FACT] Public surface the tests depend on** (this is the hard part of the migration contract):

- showcase: `App::new(Theme)`, `App::goto(PageId)`, `App::handle(Input) -> Outcome`, `App::render(&mut Frame)`, and public fields `quit`, `page`, `pages` (indexed by `PageId::index()`, `:351,363,383,603`), `focus`, `hits` (`hits.area_of`, `:128`), `ring` (`ring.reachable()`, `:460`), `dialog`, `hover`, plus `App::animating()` (`:589`) and `App::sidebar_area()` (`:633`, marked `#[allow(dead_code)]` at `app.rs:233`). `NAV_ENTRIES` is iterated by tests (`:156, 631`).
- tablepro: `App::new`, `App::connect(i)`, `app.screen`, `app.modal` (pattern-matched against `Modal::Dialog/Picker/Filter`, `:221,374,461,522,708`), `app.workbench` (`wb()`, `wb_query()`), `app.connections.connections`, `app.connections.start_connect(i)` (`:157`), `app.history.entries` (`:306,337`), `app.focus`, `app.quit`, and `WorkTab` pattern matching (`:174,466,480,...`).
- jackin: `App::for_scenario`, `app.route`, `app.world` (`accounts`, `workspaces`, `daemons`, `clipboard`, `global.trust`, `running_count()`, `workspace(id)`, `account_for(...)`, `instance(...)`), `app.screens.{editor,capsule,cockpit}`, `app.focus`, `app.quit`, and **three `WidgetId` constants reconstructed by tests**: `screens::manager::TREE`, `screens::accounts::FORM` (+ `.sub("save"/"provider"/"op")`), `WidgetId::of("editor.cfg").sub("form").sub("save")`.

**[FACT] Visual baseline.** Only the showcase has one: `app_tests.rs:623-668` hashes `(symbol, fg, bg, modifier)` of every cell **except cells inside `app.sidebar_area()`**, per page, at 120×40 and 80×24, into `tests/showcase_baseline.txt`; `UPDATE_BASELINE=1` regenerates. There is **no** cell-level baseline for tablepro or jackin — only text assertions.

**[FACT] Theme coupling in tests.** `showcase/app_tests.rs:112-118` (`focus_bar_x`) compares a cell's `fg` against `Theme::junie().focus`; `:230` against `Theme::junie().disabled`; `:252, 261-262` against `surface_overlay`, `focus`, `surface`.

**[INFERENCE] What an integration test must keep working after migration:**
1. `handle(Input) -> Outcome` / `render(&mut Frame)` on the app type, with `Outcome::Changed` still meaning "redraw needed" (the harnesses assert on it: `showcase:33-40`, `tablepro:44-51`).
2. A synchronous, deterministic draw after every input — no deferred/async layout.
3. A way to address a logical control by a stable, test-visible name. Today that is `WidgetId::of("…").sub("…")`; whatever replaces it must give tests the same reach (`jackin/app_tests.rs:75-86` `tab_to`, `:309`, `:568`; `showcase/app_tests.rs:519`).
4. A way to read back the resolved geometry of the focused/hovered control (`hits.area_of`, `showcase/app_tests.rs:124-129, 270`).
5. A way to read the reachable focus ring (`ring.reachable()`, `showcase/app_tests.rs:460-465`).
6. Tick injection at the app level with a *virtual* clock in Jackin (`clock.rs:26-30`).
7. The exact minimum-size copy strings (`"Terminal too small"`, `"Need 72×20, have 60×15"`).

---

## 7. Litmus-scenario gap analysis (goal §23)

### Scenario A — simple interactive screen (text field + button + list + dialog)
**[FACT] Evidence of the gap.** `showcase/pages/forms.rs:203-342` — 140 lines to route 9 controls, including:
- per-control `if f == self.x.id` chains (`233-276`);
- click routing that must pass "was it already focused" into the widget: `inputs.rs:229-236`, `forms.rs:293-307`, `settings.rs:396-405`, `tablepro/workbench.rs:975-979`, `jackin/modals.rs:402-406` — all of the form `let was = cx.focus.is(id); cx.focus.focus(id); w.on_click(pos, was)`;
- `for i in 0..self.mode.options.len() { if self.mode.option_id(i) == id }` (`forms.rs:308-312`);
- hover/pressed maintained by the shell (`showcase/app.rs:202-203, 576-694`);
- Tab traversal implemented in the shell (`app.rs:509-516`) and *again* inside every modal (`tablepro/app.rs:1665-1677`, `jackin/modals.rs:1084-1092`);
- modal trapping and focus restore hand-written (`showcase/app.rs:417-443`);
- cursor placement plumbed via `RenderCtx::cursor` → `frame.set_cursor_position` (`app.rs:714, 719, 733-735`);
- "which list row was clicked" via `l.locate(*id)` (`lists.rs:165-168`).

### Scenario B — custom theme
**[FACT]** `theme.rs:145-180` is the only theme constructor; the palette module is private (`theme.rs:56`); `for_level` (`183+`) only downgrades depth. No second theme exists anywhere in the repository as read. Components receive colour through a caller-supplied `bg: Color` argument on every `render` (§2.6), so a "custom theme" today means passing different backgrounds at every call site.

### Scenario C — local override
**[FACT]** No per-instance or per-part style hook was found on any component read. The nearest thing is `showcase/pages/buttons.rs:170-175`, which re-implements the button renderer by calling `t.button(kind, state, bg)` + `t.gutter(...)` + `buf.set_string` itself — i.e. an override requires copying the renderer. Line-level styling is passed as a bare `fn(&Theme, &str) -> Style` (`panels.rs:25-33`, used at `panels.rs:157`, `scrolling.rs:100`, `taskrunner.rs:351-361`, `tabs.rs:855-862`), which is the only extension point of its kind and is function-pointer/closure-shaped only for that one component.

### Scenario D — custom collection content (borrowed domain rows)
**[FACT]** Collections own converted data:
- `showcase/pages/tables.rs:39-65` — `task_rows()` builds `Vec<Vec<Cell>>` with owned `String`s from `crate::data::tasks()`.
- `showcase/pages/grid.rs:45-95` — `row(i)` builds `Vec<CellValue>` per row; `page(from)` materialises 40 at a time.
- `tablepro/tabs.rs:484-489` — every `TableTab::load` clones the entire result set: `rs.rows.iter().map(|r| r.iter().map(to_cell).collect()).collect()`.
- `tablepro/tabs.rs:2070-2074` — the same again for query results.
- `jackin/screens/usage.rs:39` — `type DetailLine = (String, Tone, Option<(u8, MeterTone)>)`, a per-frame owned tuple.
No borrowed-data or custom-cell-renderer path exists on `DataTable`, `DataGrid`, `ListBox` or `TreeView` as read.

### Scenario E — dynamic identity (insert / remove / reorder)
**[FACT] Evidence of the gap.**
- `tablepro/workbench.rs:447-449` — a tab's `WidgetId` is `ID.sub("tab").child(self.tabs.len() + self.query_counter + 1000)`, i.e. derived from mutable counters.
- `tablepro/workbench.rs:501-514` `close_tab` shifts `self.active` by index arithmetic.
- `tablepro/app.rs:1841-1852` — a close-confirmation dialog is matched back to a tab by scanning `CLOSE_DIALOG.child(i)` over `0..tabs.len()+1`.
- `tablepro/app.rs:1503-1510` and `1606-1614` — the tab-list picker smuggles the tab index through `PickerItem.detail` and parses it back: `it.detail.parse::<usize>()`.
- `widgets/grid.rs:186-191` — `PendingChanges` is keyed by *source row index*; the doc (`grid.rs:184-185`) claims safety under sorting ("a permutation") but says nothing about insert/delete of upstream rows.
- `showcase/app.rs:696-698` — nav index recovered from an id by linear scan.
**Counter-example [FACT]:** Jackin's manager already uses a value key: `RowKey::{CurrentDir, Workspace(WorkspaceId), Instance(String), NewWorkspace}` (`manager.rs:43-49`), with the doc note "Row identity is stable across background refreshes" (`manager.rs:2-3`). This is the model the library lacks.

### Scenario F — nested overlays
**[FACT]**
- showcase supports exactly **one** overlay: `dialog: Option<Dialog>` (`app.rs:206`). Pages that need a second overlay open it themselves and call `ctx.begin_modal()` from inside `Page::render` (`pickers.rs:361-372`; `editor.rs:432-435` and `chrome.rs:243-247` draw popups last by convention instead).
- tablepro supports one: `modal: Option<Modal>` (`app.rs:130`), and re-implements the backdrop + barrier + hit re-registration by hand for the filter editor (`app.rs:2345-2377`, `2462-2467`).
- jackin **does** have a stack (`app.rs:142`) with owner routes and per-modal focus save/restore, but: outside-click dismissal is a 9-arm match written per modal family (`app.rs:1755-1800`); click dispatch is another 9-arm match (`app.rs:1802-1947`); wheel routing a third (`app.rs:1731-1742`); and every modal renderer must re-register its own hits after the barrier (§2.2).

### Scenario G — custom component authoring
**[FACT]** `showcase/pages/sidebars.rs:90-164` is the only downstream-authored component in the repository. It needs: `ctx.theme`, `ctx.interaction.focused(id)`, `ctx.state(rid)`, `t.row(s, bg)`, `t.gutter(s, bg, on_accent)`, `st.fg(t.text_primary/text_secondary/accent/disabled/text_muted)`, `fill(buf, row, st)`, `ctx.clickable(rid, row)`, and `ctx.ring.register(self.id)` (162-163). It works — but the author reimplements the focus gutter, the row surface and the disabled treatment from tokens. **[INFERENCE]** There is no author-level "row" or "control surface" primitive; every custom component must re-derive the DESIGN.md state language by hand.

### Scenario H — TablePro adapter
**[FACT]** The adapter exists (`tablepro/tabs.rs:297-322` `column_specs`, `263-295` `cell_kind`/`to_cell`/`from_cell`), but the **target type is already DB-shaped**: `ColumnSpec.primary/nullable/references/enum_values` (`grid.rs:96-99`), `GridEvent::PreviewSql/FollowReference/FilterOnCell/CommitRequested/DiscardRequested` (`grid.rs:249-260`), `default_validator`'s `"is NOT NULL"` / `"Must be a UUID"` (`grid.rs:272, 309`), and the literal button label `"Preview SQL"` (`grid.rs:401`). Conversely, TablePro's SQL generation *is* correctly outside the widget (`crate::model::preview_sql`, called at `app.rs:1018, 1129, 1165`).

### Scenario I — visual preservation
**[FACT]** Only the showcase has a cell-exact baseline (`app_tests.rs:623-668` + `tests/showcase_baseline.txt`), and it deliberately excludes the sidebar rect (`app_tests.rs:632-640`). TablePro and Jackin regressions would be caught only by text-substring assertions. Jackin has one determinism assertion at frame granularity (`app_tests.rs:146-148`: two `--frame 282` runs must produce identical text) and one buffer-equality assertion inside the library-adjacent rain module (`rain.rs:920-928`).

---

## 8. Jackin visual effects and their theme/palette dependencies

**[FACT] Files.** `src/bin/jackin_preview/rain.rs` (952 lines) is the entire effect layer; `app.rs:2311-2344` is the only consumer.

**[FACT] Purity contract.** `rain.rs:1-8` — "Every frame is a pure function of `(tick, area, mode)`; there is no retained simulation grid and no wall-clock randomness, so a resize simply re-evaluates the same function at the new size." Randomness is `mix(a,b,c)` seeded by `MOTION_SEED = 0x4A41_434B_494E_5E5E` (`rain.rs:22-37`) plus a seeded xorshift for the starfield (`rain.rs:276-284`, seeded at `rain.rs:329`, outro salt `rain.rs:646`).

**[FACT] Effects.**
| Effect | Location | Timing constants |
|---|---|---|
| Typewriter phrases | `rain.rs:239-247`, `574-587` | `PHRASES` 3 × (text, ms/char, hold) `rain.rs:475-479`; `P1_LEN=64`, `P2_LEN`, `P3_LEN` `488-490` |
| Glitch reveal | `rain.rs:251-265` | `GLITCH_PASS_TICKS=2`, `GLITCH_PASSES=5` `270-271` |
| Hyperspace warp (intro + outro) | `Starfield` `rain.rs:306-445`, `sync_field` `449-469` | `WARP_TICKS=95` `273`; `WARP_START`, `INTRO_END` `493-494`; `OUT_WARP`, `OUT_CAPTION` `643-645` |
| Reduced-motion single frame | `rain.rs:596-604`, `708-715` | `REDUCED_HOLD=45` `495` |
| Cockpit atmosphere | `paint_atmosphere` `rain.rs:804-849` | column density 18 %, freeze on failure |
| Cockpit→Capsule handoff cross-fade | `handoff_stage` `rain.rs:855-870`, `dim_buffer` `102-172` | `HANDOFF_LEN=12` `851`; stages CockpitDim(1..4) → Canvas(2) → CapsuleDim(4..0) → Capsule |
| Brand pill during rituals | `draw_pill_bottom` `rain.rs:227-236` | uses `widgets::brand::Lockup` + `app::BRAND_MARK` |
| Hint chip during rituals | `draw_hint` `rain.rs:193-222` | clears its own cells first (`201-203`) |

**[FACT] Theme dependencies.**
- `rain::Tone` is a **local 6-value enum** (`rain.rs:52-57`): `Ladder(0..=4)` + `Accent`. It is *not* `theme::Tone`.
- `ladder_color` (`rain.rs:59-67`) maps `0→text_ghost, 1→text_faint, 2→text_muted, 3→text_secondary, 4→text_primary`.
- `style` (`rain.rs:70-86`) maps `Accent` dim 0/1/2 → `accent / accent_hover / accent_pressed`, and returns `None` beyond — i.e. **the number of dim steps is derived from the palette's ladder depth**.
- `fill_canvas` (`96-98`) and every `put` use `t.canvas` as background.
- `draw_hint` (`205-209`) uses `text_muted` (bold) + `text_faint` + `canvas`.
- `draw_pill_bottom` (`227-236`) delegates to `Lockup`, which uses `text_on_accent` on `accent` bold (`brand.rs:50-55`).

**[FACT] The fragile part — `dim_buffer` (`rain.rs:102-172`).** It reads each already-rendered cell's `fg`/`bg` and performs a **reverse lookup by colour equality**:
- `ladder.iter().position(|c| *c == fg)` against `[text_ghost, text_faint, text_muted, text_secondary, text_primary]` (`106-121`);
- `fg == t.accent || fg == t.success || fg == t.focus` → accent chain (`128-132`);
- `fg == t.error || fg == t.warning` → ladder position `4 - steps - 1` (`133-139`);
- unmatched → `text_faint`, or erased at `steps >= 3` (`140-146`);
- backgrounds compared against `t.canvas`, `t.accent`, then mapped to `accent_bg` / `canvas` / `surface` (`148-157`).

**[INFERENCE] Consequences for the refactor.**
1. `dim_buffer` is **not** semantic — it is colour-identity matching. In `Theme::junie()` `accent == success == focus == GREEN` (`theme.rs:167, 172, 177`), so the three-way test at `rain.rs:128` is currently degenerate; a theme that separates those roles changes the dim result silently.
2. Under `ColorLevel::Ansi16`/`Mono` the downgrade (`theme.rs:183-200+`) can collapse several ladder tokens onto one `Color`, making `position(|c| *c == fg)` ambiguous — the first match wins and the fade becomes non-monotonic. No test covers `dim_buffer` at a reduced colour level.
3. Goal §22.3 requires rain to "consume semantic theme APIs rather than assume Junie palette fields directly". Today it consumes 13 named palette fields plus colour-identity comparison. The migration needs either (a) a semantic "step this cell down N levels" API on the theme, or (b) a way for the underlying render pass to emit a *token* per cell rather than a `Color`.
4. `Starfield` holds `Vec<Option<WarpCell>>` sized to the terminal and is re-created on resize (`rain.rs:456-459`) — a resize during the warp restarts the field. This is deliberate (`rain.rs:1-8`) and must survive.
5. The three motion modes are a *product* contract asserted by tests (`app_tests.rs:139-155`), not an implementation detail: `Full`, `Reduced` (one static frame, `JACKIN_NO_MOTION=1`, `main.rs:97-101`), `Paused` (`--frame N`).

---

## 9. Summary of the highest-risk migration items

**[INFERENCE]**, ordered by risk:

1. **`DataGrid`'s DB vocabulary** (`grid.rs:96-99, 249-260, 272, 309, 401`) — splitting it is the largest single change and it is load-bearing for 3 TablePro surfaces plus the showcase grid page.
2. **Jackin's `modals.rs` family** (≈2400 lines, 7 modal types) — moving `FormDialog`/`InfoDialog`/`modal_frame` into the library while keeping `app_tests.rs`'s 24 assertions green, including the secret-masking assertions (`app_tests.rs:304-306, 344, 376-392, 566, 711, 737`).
3. **Test-visible `WidgetId` paths** — `jackin/app_tests.rs:309, 568, 649, 1183` and `showcase/app_tests.rs:519` all reconstruct internal child IDs. Any identity redesign must supply an equivalent test-addressing mechanism or those tests must be rewritten with the change documented.
4. **`rain::dim_buffer`'s colour-identity reverse lookup** (`rain.rs:102-172`) — silently theme-dependent, untested under colour downgrade.
5. **`showcase_visual_baseline`** (`app_tests.rs:623-668`) — a cell-exact digest across 44 page/size combinations; every intentional visual change must be classified before regeneration (goal §6.10, §26).
6. **Overlay barrier ordering** (`ui/ctx.rs:126-131`) — five separate manual hit re-registration blocks in Jackin and one in TablePro exist only to work around it.
7. **Focus reconciliation is duplicated three times** with three different rules (`showcase/app.rs:721-732`, `tablepro/app.rs:2106-2112`, `jackin/app.rs:2266-2274`); only Jackin consults `primary_focus()`.
