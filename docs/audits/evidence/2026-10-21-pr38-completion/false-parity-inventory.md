# PR #38 — False-parity inventory in production application code

Read-only audit of `/Users/donbeave/Projects/termrock-main-parity`
(branch `pr-38`, HEAD `7649ee7ed`). Nothing checked out, nothing modified.

Pinned references used to name what each bypass reproduces:

| Reference | Commit |
|---|---|
| PR #26 `design/interaction-polish` | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` |
| baseline source | `4169783ae883788186d161af09ba9c481bd84e5c` (tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`) |
| tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |

Scope: `crates/showcase*`, `crates/tablepro*`, `crates/jackin-preview*`,
`crates/holla*`, plus `termrock-e2e` / `termrock-conformance` checked for
captured-frame loading reaching production paths.

Definition used: a **false-parity mechanism** is production code that makes a
screen match a reference frame by selecting on the *shape of the fixture*
(exact page size, TrueColor + accent hex, scenario name, clock value, tick,
status string, selection index) or by repainting/recoloring cells after a
shared painter, instead of producing that state from the component's own
state machine.

---

## 1. Showcase (`showcase`, `showcase-demos`, `showcase-ui`)

Showcase carries the largest fixed-cell surface of any app: **12 fixed cell
tables** plus **6 exact-page-rect dispatches**.

### 1.1 `historical_*` functions / palettes / dispatch

| # | Path:line | Mechanism | Forces | Replacement |
|---|---|---|---|---|
| S1 | `crates/showcase-demos/src/pages/taskrunner.rs:161` `paint_historical` | Hand-built screen painter, called from `draw` at `:490` | Baseline task-runner body (tree rows, progress %, status column) | No shared replacement; `Steps`, `Tree`, `Progress` widgets exist and are used on the live path |
| S2 | `taskrunner.rs:2387` `historical_palette` | Returns six `PaintStyle`s derived from `Surface`/`Family`/`Variant` to reproduce baseline inks | Baseline panel/title/detail/meta/primary inks | Theme ladder (`ui.style`) already exists |
| S3 | `terminal.rs:393` `historical_palette`, used at `:124` | Same palette bypass for the narrow rail | Baseline terminal rail inks | Theme ladder |
| S4 | `crates/showcase-demos/src/pages/progress.rs:24-47` | 20 hardcoded hex colours (`CANVAS`, `SURFACE`, `ACCENT`, `BUTTON_BG`, …) + `ink()`/`settled()`/`face()` helpers at `:106-139` | Baseline progress page at every colour level | `Theme`/`Role` tokens |

### 1.2 Fixed frame / cell tables reproducing screens

| # | Path:line | Table | Entries | Reference screen |
|---|---|---|---|---|
| S5 | `taskrunner.rs:607` | `COMPACT` | 128 | idle page rect 51×16 inside 72×20 shell |
| S6 | `taskrunner.rs:838` | `NARROW` | 175 | idle page rect 59×20 inside 80×24 shell |
| S7 | `taskrunner.rs:1177` | `SETTLED` | 115 | idle 120×40 page, origin-relative |
| S8 | `taskrunner.rs:1563` | `WIDE` | 219 | idle page rect 134×46 inside 160×50 shell |
| S9 | `terminal.rs:568` | `SETTLED` | 426 | finished launch, 94×N settled page |
| S10 | `terminal.rs:1405` | `NARROW_SETTLED` | 223 | 59×20 settled terminal |
| S11 | `terminal.rs:1710` | `COMPACT_SETTLED` | 174 | 51×16 settled terminal |
| S12 | `terminal.rs:1951` | `WIDE_SETTLED` | 453 | 134×46 settled terminal |
| S13 | `chrome.rs:495` | `NARROW_80X24` | 91 | 59×20 page inside 80×24 shell |
| S14 | `chrome.rs:731` | `NARROW_72X20` | 72 | 51×16 page inside 72×20 shell |
| S15 | `chrome.rs:913` | `WIDE_160X50` | 183 | 134×46 page inside 160×50 shell |
| S16 | `chrome.rs:1693` | `MEDIUM_100X30` | 125 | 79×26 page inside 100×30 shell |

All twelve are `(row, col, fg_hex, bg_hex, bold, text)` runs painted by
`paint_narrow` / `paint_compact` / `paint_wide` / `paint_settled`
(`taskrunner.rs:545,565,585,521`; `terminal.rs:1382,1687,1922,546`;
`chrome.rs:410,430,450,473`). Sources are captioned “Cells from the approved
snapshot”.

### 1.3 Predicates selecting a path by exact width/height

| # | Path:line | Predicate | Path selected |
|---|---|---|---|
| S17 | `progress.rs:14-19`, `:143-153`, `:1356-1368` | `NARROW_PAGE_W/H` 59×20, `COMPACT_PAGE_W/H` 51×16, `WIDE_PAGE_W/H` 134×46 | `paint_narrow` strip for three exact frames |
| S18 | `progress.rs:1427` | `wide_page(area)` | `paint_wide_meters` (`:714`) |
| S19 | `terminal.rs:271-285` | `SETTLED_WIDTH/HEIGHT` 94×…, `NARROW_*` 59×20, `COMPACT_*` 51×16, `WIDE_*` 134×46 (consts `:524-531`) | four settled cell tables |
| S20 | `taskrunner.rs:447-455` | `area.width == 59 && area.height == 20`, `51×16`, `134×46` | `paint_narrow` / `paint_compact` / `paint_wide` |
| S21 | `chrome.rs:2081-2101` | `59×20`, `51×16`, `134×46`, `79×26` | four chrome cell tables |
| S22 | `selectable.rs:257-258`, `:526`, `:1446-1448` | `WIDE_PAGE_W/H` 134×46 | `paint_wide_playground` (`:1454`) |
| S23 | `lists.rs:53-54`, `:931` | `WIDE_PAGE_W/H` 134×46 | wide readout composition |
| S24 | `showcase-ui/src/app.rs:1173` | unconditional `x = x.saturating_add(1)` with comment “baseline crumb starts one cell later” | baseline breadcrumb alignment |

### 1.4 Clock / tick forcing

| # | Path:line | Mechanism | Forces |
|---|---|---|---|
| S25 | `terminal.rs:259-265` | `became_settled = !self.settled && cx.update_cause() == UpdateCause::Tick` — first tick switches to the settled frame | baseline “finished launch” frame, reached only because the harness ticks |

### 1.5 Repaints / recolours after a shared painter (button and table repairs)

| # | Path:line | Mechanism | Forces |
|---|---|---|---|
| S26 | `progress.rs:1126-1152` `paint_narrow_button` | Calls `Button::draw`, then overwrites fill, leading cell and label with `settled(WHITE, BUTTON_BG)` | baseline narrow button cells |
| S27 | `progress.rs:1154-1184` `paint_narrow_scroll` | Hand-paints `┃/┆/━/┄` thumb and track with fixed `THUMB`/`TRACK` | baseline scrollbar cells |
| S28 | `editable.rs:305-355` `legacy_table` + `:228 legacy_header` + `:266 legacy_row` | Draws the real editable table, then repaints header and every row via `paint_str`; called at `:520-523` only while `!editing && edits == 0` | baseline table at 70/90/130-column breakpoints |
| S29 | `forms.rs:82-210` `legacy_gutter`, `legacy_marker`, `legacy_placeholder`, `legacy_input_placeholder`, `legacy_choice_label` | Overpaint `▎`, `(●)/( )`, `○──`, placeholder and choice labels after `RadioGroup`/`Checkbox`/`Toggle`/`TextInput::draw`; ~15 call sites `:560-770` | baseline form/choice cells |
| S30 | `trees.rs:957-975` and onward | `project_tree().draw(...)` then “card paint below replaces its pixels with the baseline geometry” and per-row `PaintedRow` repaint | baseline project tree rows |
| S31 | `showcase-ui/src/app.rs:1389-1440` `baseline_nav_scroll` | After `nav_tree().draw`, repaints the track with `0x47/0x11` hex and re-applies `scroll_edges` | baseline sidebar scrollbar |

Also relevant: `buttons.rs:75-90/287-352` uses `ui.reference(...)` for the
state matrix — that is the intended reference-rendering API, **not** a bypass.

---

## 2. TablePro (`tablepro`, `tablepro-demo`, `tablepro-domain`, `tablepro-ui`)

Lead item in the removal order. TablePro uses real `Picker`, `Tabs`, `Tree`
and `Grid`, then constrains or overpaints them.

### 2.1 Pickers

| # | Path:line | Mechanism | Forces | Replacement |
|---|---|---|---|---|
| T1 | `tablepro-ui/src/tab_list.rs:53-66` | `LayerSize::Fixed(64, rows.saturating_add(7))` with comment “Baseline card: borders + title + rows + gap + hint row” | approved Ctrl+G tab-list card geometry for every tab count | `Picker` exists and is used; the fixed size is the bypass |
| T2 | `tablepro-ui/src/safe_mode.rs:47-59` | `LayerSize::Fixed(112, rows.saturating_add(5))`, “Baseline non-searchable card” | approved Safe Mode card at 120×40 | `Picker` |
| T3 | `tablepro-ui/src/quick_switcher.rs:41-55` | `LayerSize::Fixed(88, rows.saturating_add(7))`, “Baseline card: … up to 12 rows” | approved Ctrl+O card | `Picker` |
| T4 | `tablepro-ui/src/app.rs:9270-9291` `paint_picker_move_footer` | Hand-painted `↑↓ Move · Enter Choose · Esc Cancel` footer | approved Safe Mode footer row | `Picker` hint/footer painting |
| T5 | `tablepro-ui/src/app.rs:9292-9317` `paint_picker_layer_footer` | Hand-painted `Type Filter · ↑↓ Move · Enter Choose · Esc Cancel` | approved quick-switcher / tab-list footer row | `Picker` hint/footer painting |
| T6 | `tablepro-ui/src/app.rs:9320-9340` `draw_footer` | Selects T4/T5 by `Surface::QuickSwitcher / TabListPicker / SafeModePicker` | picks the frozen footer per picker | footer ownership belongs to `Picker` |

`crates/tablepro-ui/src/app/picker_contract.rs` is `#[cfg(test)]`
(`app.rs:31-32`) — contract tests, not a bypass.

### 2.2 Repainting shared widgets

| # | Path:line | Mechanism | Forces |
|---|---|---|---|
| T7 | `app.rs:5268-5360` `paint_legacy_tabs` (+ `legacy_tab_width` `:5258`), invoked at `:10222` immediately after `tab_strip().draw(...)` `:10214` | Repaints prefix ink, active fill, active rule, close cells over the real `Tabs` | approved tab strip cells |
| T8 | `app.rs:5060-5066` `legacy_tree_body` + `:2860/2999` | Shifts the explorer body rect one cell left/wider before `Tree::draw` | approved explorer geometry |
| T9 | `app.rs:6864-6890` `paint_legacy_filter`, calls `:3028`, `:8740` | Hand-paints the “Filter objects” field and its caret | approved filter field cells |
| T10 | `app.rs:6916`, `6932`, `6952`, `6973`, `6995`, `7156`, `7207` (`paint_explorer_title_space`, `_title_gap`, `_meta_band`, `_chevron_gap`, `_indent`, `_outside`, `_collapse`) | Seven micro-repaints after the shared tree/list painters (calls `:3011-3052`) | approved explorer title/meta/chevron/indent/collapse cells |
| T11 | `app.rs:6237-6260` `paint_grid_header`, `:5650-5700` `paint_grid_scrollbar`, `:5618` `paint_grid_focus_bar` | Header, scrollbar and focus bar repainted after `Grid::draw` | approved results-grid chrome |
| T12 | `app.rs:5693-5720`, `:6471-6496` | `truecolor = capability.color == TrueColor` branches choose fixed greys (`0x8c8c8c`, `0xcccccc`, `0x3f3f3f`, `0x5c5c5c`) or `DIM` | baseline scrollbar hover and row-fade ramps; reduced levels get a different ramp |

### 2.3 Exact-size and clock predicates

| # | Path:line | Mechanism | Forces |
|---|---|---|---|
| T13 | `app.rs:10151-10163` | `focus_new_query` steals focus only when `view.width == 120 && view.height == 40` (“Only 120×40 has the historical focused editor”) | baseline focused editor frame |
| T14 | `app.rs:10166-10172` | `cx.focus(FILTER_VALUE)` only when `120×40` (“The approved bar sits on the 120×40 value placeholder”) | baseline filter bar |
| T15 | `app.rs:10358-10360` + `:10366-10375` `feedback_clock` | Paused frames always run on `Simulation { initial: 0 }` so press flash stays lit (“The tag held flash time at zero”) | frozen press-flash frame |

### 2.4 Manual cursor registration

| # | Path:line | Mechanism |
|---|---|---|
| T16 | `app.rs:8781` | `ui.set_bar_cursor(CONNECTION_FILTER, …)` after hand-painting the connection filter caret (the field itself is painted at `:8730-8780`) |
| T17 | `app.rs:9153-9156` | `ui.set_bar_cursor(FILTER_VALUE, …)` on the hand-painted filter-editor value row |

### 2.5 Internal plumbing that exists only for the repaint layer

| # | Path:line | Mechanism |
|---|---|---|
| T18 | `app.rs:5104-5112` | Eight `thread_local!` `RefCell` channels (`COL_BUDGETS`, `GRID_CHROME`, `GRID_SCROLL`, `GRID_SEEN_ROW`, `GRID_SORT`, `GRID_FILTER`, `GRID_EDIT`) written at `:5202`, `:6019` and read by the chrome painters at `:5673`, `:5842`, `:6244`, `:6444`, `:6608` — draw-order coupling replacing widget state |
| T19 | `app.rs:4565` | Comment “Integer centering lands one cell left of the historical hint”, encoding a one-cell parity nudge in layout code |

---

## 3. Jackin (`jackin-preview`, `jackin-preview-app`, `jackin-preview-sim`, `jackin-preview-presentation`)

Jackin has the densest predicate layer and the clearest duplication of shared
components.

### 3.1 `historical_*` predicates and `draw_historical_*` dispatch

Every predicate below is a full frame-identity check: **exact width×height +
`ColorLevel::TrueColor` + `theme.color.accent == Rgb(0x48,0xe0,0x54)` +
scenario + `Motion::Paused` + 10-25 state fields**. Several also pin a status
string or a list cursor index.

| # | Predicate | Painter | Reference frame forced | Replacement |
|---|---|---|---|---|
| J1 | `app.rs:7260-7290` `accounts_add_form_scene` → `:7291` `historical_accounts_add_form` | `:7299` → `accounts_add_form_view::paint` | `a` on paused 120×40 accounts list; add form open | shared `termrock::fields` + `Form` used, but the frame is a cell table |
| J2 | `app.rs:7322-7353` `historical_accounts_filter` | `:7355` → `accounts_filter_wide::paint` | `/`, `work`, Enter on paused 120×40 list; status `Accounts · filt…` | `termrock::feedback::accounts_filter` exists |
| J3 | `app.rs:7362-7394` `historical_accounts_filter_80` | `:7396` → `accounts_filter_view::paint` | same filter at 80×24 | same |
| J4 | `app.rs:7405-7437` `historical_accounts_detail` (pins `selected_id == "acct-claude-work"`, `cursor == ItemKey::index(4)`, status `Accounts › Claude › Work`) | `:7439` → `accounts_drawer_view::paint_detail` | four Downs then 120×40 detail row | `termrock::feedback::paint_accounts_detail` |
| J5 | `app.rs:7447-7479` `historical_accounts_detail_80` | `:7481` | same at 80×24 | same |
| J6 | `app.rs:7518-7550` `historical_accounts_drawer` (`drawer_open`, same pinned selection/status) | `:7552` → `accounts_drawer_view::paint` | Enter on the detail row, 120×40 | `paint_accounts_drawer` |
| J7 | `app.rs:7560-7594` `usage_overview_narrow_host` — **inverted** predicate: excludes `120×40`, keeps every other size ≥72×20 | `:7601` `draw_usage_overview_narrow` | non-120×40 usage overview | `paint_usage_overview` |
| J8 | `app.rs:8013-8052` `historical_usage_overview` | `:8054` → `usage_overview_view::paint` | `u` on paused 120×40 accounts list | `paint_usage_overview` |
| J9 | `app.rs:8062-8102` `historical_usage_detail` (`UsageTab::Registration`, detail open) | `:8104` → `usage_detail_view::paint` | `u`, Down, Enter | `paint_usage_overview` detail |
| J10 | `app.rs:8131-8157` `historical_audit_accounts_80` | `:8159` → `accounts_detail_view::paint_audit` | paused 80×24 accounts overview | `paint_accounts_detail` |
| J11 | `app.rs:8200-8220` `accounts_boot_needle_rest` | `:8191` `draw_accounts_boot_needle_rest` paints literal `"jackin❯"` at (0,0) | brand glyph the harness waits for on 72/100/160 lists | real `SessionChrome` brand |
| J12 | `app.rs:8222-8240` `accounts_boot_needle` | `:8112` same literal | 120×40 accounts brand | same |
| J13 | `app.rs:8242-8268` `accounts_boot_needle_80` | `:8121` same literal | 80×24 accounts brand | same |
| J14 | `app.rs:8270-8280` `capsule_agent_picker_frame` | `capsule_split_view::paint` | Ctrl+B `"`/`c` agent picker | no shared picker in use |
| J15 | `app.rs:8281-8321` `capsule_zoom_frame` (pins `selected_pane == 1`, status `zoom · focused pane`) | `:8322` → `capsule_zoom_view::paint`; then `set_undrawn_top_bar_cursor` at (3,34) | Ctrl+B `z` zoomed pane | `feedback::paint_zoomed_pane` |
| J16 | `app.rs:8340-8368` `capsule_menu_80` (pins `capsule_tab_menu_pos == Position::new(1,2)`) | `:8403` → `capsule_menu_view::paint` + cursor at (3,19) | Ctrl+B `m` tab menu at 80×24 | `feedback::paint_capsule_menu` |
| J17 | `app.rs:8369-8385` `capsule_menu_120_frame` | pane shrink logic `:8624-8647` | Ctrl+B `m` at 120×40; every pane shortened one row | shared viewport |
| J18 | `app.rs:8386-8402` `capsule_palette_120_frame` | same shrink logic | Ctrl+B space palette at 120×40 | shared overlay layer |
| J19 | `app.rs:8425-8457` `historical_capsule_frame` — no size check; scenario/motion/state only | focus steal `:4929`, reserve-row logic `:8637-8650` | resting capsule pane layout | shared `Viewport` |
| J20 | `app.rs:7137-7150` `launch_failure_frame` | `launch_cockpit::paint_failure` | 120×40 launch failure, TrueColor | no shared replacement |
| J21 | `app.rs:7151-7167` `launch_cockpit_scene` — **`self.frame() == 40`** | `launch_cockpit::paint` | paused launch cockpit at tick 40 | same |
| J22 | `app.rs:10016-10047` `hard_launch_picker_host` | agent picker under manager | Enter on paused 120×40 hard-cases manager | shared picker not used |
| J23 | `app.rs:10048-10073` `historical_hard_launch_locked` (pins `ManagerRowKey::Workspace(12)`) | `:10075` hand-built `SessionMenu`/`SessionHint` + `paint_session_chrome` | five Downs, Space, Down, Enter on hard cases | `paint_session_chrome` exists but data is hardcoded |
| J24 | `app.rs:10141-10177` `historical_hard_cases_80` | `:10179` hand-built `ManagerLeaf[10]` + `paint_manager_frame` | paused 80×24 hard-cases manager | `paint_manager_frame` exists; data hardcoded |
| J25 | `app.rs:10298-10324` `historical_help_overlay` (pins status `Keyboard shortcuts`) | `:10326` `paint_pinned_split` + repaint of title at (3,2) with `0xb3b3b3` + `manager_help::draw` + selection cell recolor + `manager_help::dim_underlay` + custom `HintBar` with white/black patch | `?` on paused 120×40 Returning manager | `HelpOverlayState` / shared help card |

Dispatch is concentrated in `app.rs:11045-11221`:
`historical_manager` block `:11045-11108` (pinned manager split, detail
drawer, expanded split, FirstUse empty split), `prelude_view::paint` `:11110-11125`,
`editor_view::paint` `:11127-11148`, `settings_view::paint` `:11195-11210`,
then the plate calls `:11190-11221` (`draw_accounts_boot_needle*`,
`draw_historical_audit_accounts_80`, `draw_historical_accounts_add_form`,
`_filter`, `_filter_80`, `_detail`, `_detail_80`, `_drawer`,
`draw_usage_overview_narrow`, `draw_usage_detail_live`,
`draw_historical_usage_overview`, `draw_historical_usage_detail`,
`draw_historical_hard_launch_locked`, `draw_historical_hard_cases_80`,
`draw_historical_help_overlay`). Each call self-guards, so draw order plus
predicate is the screen router.

### 3.2 Fixed cell tables / frame constants

| # | Path:line | Table | Size | Reference |
|---|---|---|---|---|
| J26 | `app/accounts_add_form_view.rs:7-1413` | `ENTRIES: &[(u16,u16,&str,u32,u32,bool,bool)]` | 1 400+ runs | 120×40 accounts add form, cell by cell |
| J27 | `app/usage_overview_view.rs:7-…` | `ENTRIES` same shape | whole page | 120×40 Usage › Overview |
| J28 | `app/usage_detail_view.rs:7-…` | `ENTRIES` same shape | whole page | 120×40 usage detail |
| J29 | `app/settings_view.rs:21-…` | `TABS: &[(&str,u16,u16)]` fixed tab x/width table | 7 tabs | approved settings tab strip |
| J30 | `app/prelude_view.rs:25-…` | `ROWS: &[SourceRow]` fixed rows | source list | prelude step 1 |
| J31 | `app/launch_cockpit.rs:9-23` | `BRAND`, `ROLE`, `WORKSPACE`, `AGENT_LINE`, `CREDENTIALS`(`"credentials  5 accounts · Claude · Work (1Password) · …"`), `CREDENTIAL_VALUE`, `SPINNER` | fixed strings | baseline cockpit header/credentials |

Each `*_view.rs` header states “Replaces historical_*” — they are the same
bypass renamed, with the frame transcribed into a table.

### 3.3 Raw-buffer painting and blits

| # | Path:line | Mechanism | Reference |
|---|---|---|---|
| J32 | `app/launch_cockpit.rs:44,68,422` | `Buffer::empty(screen)` + `put(...)`/`paint_*` into a raw buffer, then `blit(ui, &buf, …)` | whole cockpit, failure hero, open footer; the app never renders the cockpit through widgets |
| J33 | `app.rs:6060-6075`, `:6381`, `:7028` | `blit_buffer` into the UI (intro, outro, rain background) | baseline intro/outro/rain |
| J34 | `app/launch_cockpit.rs:558`, `:1015-1035` | `NO_COLOR` + `ColorLevel` branches choose alternate inks; `legacy_mono(fg,bg,y==0)` | mono baseline |
| J35 | `app/launch_dialog_plate.rs:11-66` | `paint_info`/`paint_cancel` call the shared cards, then `ui.recolor_rgb` / `recolor_pair` / `add_dim` over them | 256/16/mono info and cancel dialogs |
| J36 | `app/launch_dialog_plate.rs:122-144` | `ansi256_ink`: hardcoded truecolor→Indexed map (`0x48e054→78`, `0x27272a→235`, …) | baseline `apply_256` index table |
| J37 | `app.rs:7196-7245` `paint_cockpit_trace` | Fixed `Indexed(111)`/`Indexed(16)` for 256, `legacy_mono`, plus `recolor_fg` remaps `233/78→232`, `235/16→238` | baseline trace label + ghost ramp |

### 3.4 Invisible controls registered only to force cursor output

| # | Path:line | Call |
|---|---|---|
| J38 | `app.rs:7319` | `ui.set_bar_cursor(crate::screens::accounts::START, Position::new(29, 7))` on the add-form plate |
| J39 | `app.rs:8325` | `ui.set_undrawn_top_bar_cursor(CAPSULE_INPUT, ui.full(), Position::new(3, 34))` after the zoom plate |
| J40 | `app.rs:8336` | same, `Position::new(3, 20)` — comment: “the input row is not drawn, so the caret has to be recorded here” |
| J41 | `app.rs:8410` | `ui.set_undrawn_top_bar_cursor(CAPSULE_TAB_MENU, …, Position::new(3, 19))` — “The bar sits on the capsule line, outside the popup clip” |
| J42 | `app.rs:10978` | `set_undrawn_top_bar_cursor(...)` on the manager path |

### 3.5 Repairs after shared painters

| # | Path:line | Function | Forces |
|---|---|---|---|
| J43 | `app/capsule_split_view.rs:76-103` | `repair_underlay` | agent-picker underlay cells |
| J44 | `app/capsule_split_view.rs:105-109` | `repair_card_pad` | card padding cells |
| J45 | `app/capsule_split_view.rs:192-226` | `repair_fitted_footer` | fitted-size footer row |
| J46 | `app/capsule_split_view.rs:334-354` | `repair_fitted_underlay` | fitted-size underlay |
| J47 | `app.rs:10333-10385` | inside `draw_historical_help_overlay` | title ink, focus gutter blanked to `0x173f1c`, hint bar patched white/black |
| J48 | `app.rs:6655-6675` | `draw_historical_detail_drawer` — `legacy` = TrueColor && accent hex; non-legacy paints the live drawer | two different drawers by colour level |

### 3.6 Other clock/status couplings

| # | Path:line | Mechanism |
|---|---|---|
| J49 | `app.rs:685-692` | `route` chosen by `Scenario` + `frame >= INTRO_END` / `frame > 0` |
| J50 | `app.rs:804` | `outro: scenario == OutroLast && frame > 0` |
| J51 | `app.rs:1163` | `scenario == Returning && motion == Paused` selects a fixed status path |
| J52 | `app/settings_view.rs:129`, `:329` | `sized = (80×24) || (120×40)` — only those two frames get the sized layout |

---

## 4. Holla (`holla`, `holla-ui`, `holla-sim`, `holla-domain`, `holla-plan`)

Holla's selector layer is **scenario + clock**, not size alone: every tag
plate is `scenario == X && now_ms() == 80` (`--frame 40` seeks 80 ms).

### 4.1 `tag_frame` predicates (scenario + clock value)

`crates/holla-sim/src/catalogue.rs`, all requiring `w.now_ms() == 80`:

| # | Line | Predicate | Reference |
|---|---|---|---|
| H1 | `:842` | `first_use_tag_frame` | concept home `first-use` tag frame |
| H2 | `:852` | `monorepo_root_tag_frame` | monorepo root |
| H3 | `:857` | `monorepo_child_tag_frame` | monorepo child |
| H4 | `:862` | `activities_multi_tag_frame` | activities multi |
| H5 | `:870` | `docker_cleanup_tag_frame` (plus `DOCKER_CLEANUP_QUERY = "docker clean"` `:868`) | docker cleanup |
| H6 | `:875` | `disk_cleanup_tag_frame` | disk cleanup |
| H7 | `:890` | `remote_host_tag_frame` | remote host |
| H8 | `:895` | `launch_failure_tag_frame` | launch failure |
| H9 | `:918` | `hard_cases_tag_frame` | hard cases |
| H10 | `:1086` | `parity_discovery_tag_frame` | parity-discovery catalogue |
| H11 | `:1139` | `parity_history_tag_frame` | parity-history bands |
| H12 | `:1334` | `parity_files_tag_frame` | parity-files bands |
| H13 | `:1476` | `parity_browser_tag_frame` | parity-browser bands |
| H14 | `:1568` | `parity_git_current_tag_frame` | git current |
| H15 | `:1794` | `parity_git_batch_tag_frame` | git batch |
| H16 | `:2026` | `parity_task_sources_tag_frame` | task sources |
| H17 | `:2147` | `parity_cargo_tag_frame` | parity-cargo bands |

Producer gates also use clock: `parity_discovery` `:922-928` returns `None`
until `now_ms() >= 80`; `history_enabled()` `:847` reads
`HOLLA_NO_HISTORY == "0"`.

### 4.2 Plate predicates (size bands)

`crates/holla-ui/src/app.rs`:

| # | Line | Predicate | Band |
|---|---|---|---|
| H18 | `:377` | `parity_history_tag_plate` | `80..160` |
| H19 | `:382` | `parity_history_clipped_plate` | `80..120` |
| H20 | `:387` | `parity_files_clipped_plate` | `80..120` |
| H21 | `:392` | `parity_files_tag_plate` | `80..160` |
| H22 | `:397` | `parity_browser_clipped_plate` | `80..120` |
| H23 | `:403` | `parity_browser_tag_plate` | `80..160` |
| H24 | `:408` | `parity_git_current_clipped_plate` | `80..120` |
| H25 | `:412` | `parity_git_current_tag_plate` | `80..160` |
| H26 | `:416` | `parity_cargo_clipped_plate` | `80..120` |
| H27 | `:420` | `parity_cargo_tag_plate` | `80..160` |
| H28 | `:424` | `parity_git_batch_clipped_plate` | `80..120` |
| H29 | `:428` | `parity_git_batch_tag_plate` | `80..160` |
| H30 | `:432-436` | `parity_task_sources_tag_plate` | `width == 80` only |

`crates/holla-sim/src/catalogue.rs` exact-size plates: `:881`
`disk_cleanup_100x30_plate`, `:901` `launch_failure_80x24_plate`, `:910`
`launch_failure_100x30_plate`.

`crates/holla-ui/src/screens/home.rs`: `:778,792,805,817,937,976,1022,1041,1054,1175`
`(80..120)` gates; `:1634-1636` (`wide`=120×40, `mid`=100×30, `narrow`=80×24)
in `paint_disk_cleanup_plate` (`:1632`); `:1877`, `:1884` same triple in
`paint_launch_failure_plate` (`:1875`).

### 4.3 Presentation selectors (status / header / bands)

| # | Path:line | Mechanism | Reference |
|---|---|---|---|
| H31 | `holla-ui/src/app.rs:1690-1706` | 17-branch `else if` chain in the status bar, one branch per tag frame / plate | baseline status row per scenario |
| H32 | `holla-ui/src/app.rs:439,456,475,500,517,534,551,576,606,624,675,692,830,847,891,986` | 16 `paint_*_status` functions with hardcoded strings: `~/scratch`, `91% ▲`, `/srv/payments`, `detached @ v2.14.3`, `↓14 ↑3`, `• 7`, `2 conflicts`, `! docker, github unavailable`, … | baseline status bars |
| H33 | `holla-ui/src/app.rs:1613-1660` | `paint_disk_menu_titles`, project string `"acme › frontend · Node project · pnpm"`, host strings `"mbp · local"`, `"devbox · development"`, `"◆ prod-eu-1 · production"` chosen by tag predicates | baseline header project/host |
| H34 | `holla-ui/src/app.rs:879-889` `launch_failure_plate_open` + `:884-885`, `:906`, `:951` | size-gated launch-failure plates | baseline launch-failure page |
| H35 | `holla-ui/src/app.rs:1677-1689` | `activities_multi_view(...)` collapses the “here” band and paints `paint_activities_multi_80x24_band` (`:2406`) | baseline activities band |
| H36 | `holla-ui/src/app.rs:1685-1688` | `activities_multi_chips` (`:2470`) painted only on the tag frame | baseline activity chips |
| H37 | `holla-ui/src/screens/home.rs:350-353` | `.or_else(|| catalogue::{first_use,monorepo_root,…,activities_multi}_plate(world, self.history))` — plate chosen before live rows | baseline concept home lists |
| H38 | `holla-ui/src/screens/home.rs:62-66`, `:68-74` | `seed_query` / `release_tag_query`: seeded `docker clean` released when the clock leaves tick 40 | baseline pre-typed finder query |
| H39 | `holla-ui/src/screens/home.rs:74`, `:234` + `:264-346` | `docker_cleanup_tag_frame` gates the filter and `set_bar_cursor(QUERY, (3,4))` repeated 7× | baseline finder caret |
| H40 | `holla-ui/src/app.rs:634`, `:773`, `:906`, `:951`, `:1724`, `:1778`, `:1797` | `width == 80 && height == 24` branches inside otherwise shared painters | 80×24 baseline chrome |

`holla-sim` `*_view(width)` builders (14) pre-compose `ParityDiscovery`
lines per width: `catalogue.rs:1094,1146,1462,1483,1780,2012,…`. They are the
data half of the same mechanism.

---

## 5. Cross-app findings

| # | Path:line | Finding |
|---|---|---|
| X1 | `crates/termrock-feedback/src/accounts_detail.rs:324` | `ui.recolor_rgb(...)` applied per row after row paint, to fake the row fade (`fade_keep`) — shared component recolouring its own output because the fade role is not modelled |
| X2 | `crates/termrock-feedback/src/manager_frame.rs:330` | `ui.recolor_rgb(...)` after painting, same fade pattern |
| X3 | `crates/termrock-feedback/src/inspector_body.rs:425` | `ui.recolor_fg(Rect, …)` per line after paint |
| X4 | `crates/jackin-preview-app/src/app.rs:6060-6075`, `:6381`, `:7028` + `launch_cockpit.rs:44/68/422` | Two raw-buffer pipelines (`Buffer` + `put` + `blit`) instead of UI/widgets. Only Jackin does this. |
| X5 | `crates/showcase-ui/src/app.rs:1953-1956`, `tablepro-ui/src/cli.rs:88`, `holla-ui/cli.rs` | `ColorLevel` parsing from CLI — **legitimate** capability handling, keep |
| X6 | `crates/termrock-e2e/src/lib.rs:18-20,53`, `termrock-conformance/tests/*` | `include_str!` of registry/schema JSON and captured-ANSI corpora — **test tooling only**; no app crate depends on `termrock-e2e` (verified: no `termrock-e2e` in any app `Cargo.toml`) |
| X7 | `baseline/before/*.ansi` (≈16 k files), `baselines/`, `snapshots/` | ANSI replay corpus. No production app reads `.ansi`; only tooling and visual tests |
| X8 | `crates/showcase-demos/src/pages/buttons.rs:75-90`, `:287-352` | `ui.reference(...)` state matrix — intended API, keep |
| X9 | `crates/tablepro-ui/src/app/picker_contract.rs` | `#[cfg(test)] mod` at `app.rs:31-32` — test-only, keep |
| X10 | `crates/jackin-preview-sim/src/fixtures/*`, `pinned.rs` | Fixture data (accounts, workspaces, PTY transcripts). Data, not drawing — keep, but it is what makes the tag predicates look "productive" |

Shared duplicate implementations to note: `jackin-preview-app/src/app/*_view.rs`
duplicate `termrock-feedback` painters (`accounts_detail`, `accounts_drawer`,
`accounts_filter`, `capsule_menu`, `usage_overview`, `settings_page`,
`zoom_pane`, `session_chrome`, `manager_frame`) with the layout frozen to one
frame, and `accounts_add_form_view.rs` duplicates form chrome that
`termrock::fields` already provides.

---

## 6. Legitimate items — do not remove as bypasses

- **Demo data**: `showcase-demos` `SPECS`/`GROUPS`/`MATRIX_STATES`
  (`buttons.rs:30-73`), `JOBS` (`progress.rs:64-104`), `FILES`, `TASKS`,
  `MODES`, `PROPS`, `CANDIDATES`; `jackin-preview-sim/src/fixtures/*`
  (`pinned.rs` included); `holla-sim` catalogue/action/world data;
  `tablepro-demo` catalog and grid fixtures.
- **Layout breakpoints** that only choose a composition and render all
  content from model state: `lists.rs:232/438/773`, `grid.rs:393/433/467`,
  `pickers.rs:404/480`, `panels.rs:449/646`, `textareas.rs`,
  `inputs.rs`, `scrolling.rs:296/446/…`, `settings.rs:572/618/679`,
  `overview.rs:362/372/417`, `grid.rs:379/389/433` (`SECTION_WIDE`),
  `showcase-ui/src/app.rs` `MIN_*` shell geometry.
- **Capability / colour handling**: `ColorLevel` parsing (`cli.rs`), theme
  `downgrade_color`, `NO_COLOR` detection, `Theme::capability` ladders, and
  mono glyph substitution in shared widgets. Also `Terminal`/`Output`
  content and `MediaQuery`-style `ColorLevel` display labels.
- **Terminal-content viewers**: showcase `terminal.rs` live `Output` widget,
  jackin pane viewports fed by PTY fixtures, holla preview panes.
- **Test-only fixtures and harnesses**: `showcase-ui/tests/fixtures/*.tsv`,
  `termrock-conformance/tests/fixtures`, `termrock-e2e` bundles/registry,
  `picker_contract.rs`, `#[cfg(test)]` modules, `include_str!` in tests.
- **`ui.reference(...)` state matrices** (`buttons.rs`, `forms.rs`,
  `inputs.rs`, `pickers.rs`, `textareas.rs`): reference rendering is the
  product feature, not a bypass.
- **Minimum/preferred window size** (`App::min_size`, `MIN_WIDTH/HEIGHT`
  early return with a “too small” panel): window policy, keep.
- **Scenario/motion/frame CLI plumbing in preview binaries**
  (`holla/src/main.rs`, `jackin-preview/src/main.rs`, `tablepro-ui/cli.rs`):
  the harness interface itself is legitimate; only the *draw/state* gates
  fed by it are findings.

---

## 7. Removal order

Order is by (a) how much a single change removes, (b) narrowness of the
shared-component contract it is blocking, (c) regression risk.

1. **TablePro pickers** — `tab_list.rs:53-66`, `safe_mode.rs:47-59`,
   `quick_switcher.rs:41-55`, `app.rs:9270-9340`.
   Delete the three `LayerSize::Fixed(...)` baseline formulas and the two
   manual footers; let `Picker` size and paint its own card and hints. Lowest
   risk: the real `Picker` already owns the state, only the geometry and
   footer ink are frozen. Gate: picker contract tests keep the approved cells
   through component defaults, not app arithmetic.

2. **Showcase button repairs** — `progress.rs:1126-1152` (`paint_narrow_button`),
   `forms.rs:82-210` (+ call sites `:560-770`), and the button-specific cells
   inside `taskrunner.rs` `SETTLED` / `terminal.rs` `SETTLED` that are the
   `Button`/`Toggle` states. Also `progress.rs:1154-1184` narrow scrollbar.
   Replace with `Button`/`Checkbox`/`Toggle`/`RadioGroup` plus theme roles;
   delete `legacy_*` overpaint. These are small, local, and have direct
   component equivalents, so they can go before the large cell tables.

3. **Jackin cell tables** — `accounts_add_form_view.rs:9`, `usage_overview_view.rs:9`,
   `usage_detail_view.rs:9`, plus `prelude_view.rs:25`, `settings_view.rs:21`,
   `launch_cockpit.rs:9-23`. Replace each `ENTRIES` table with the matching
   `termrock::feedback` painter fed from world state, then delete the
   `historical_*` predicates (J1-J13, J20-J21) that exist only to select the
   table. Largest single deletion in the repo, but the shared painters already
   exist and are already called on the live path, so the work is data mapping,
   not new rendering.

4. **Holla presentation selectors** — `holla-sim/src/catalogue.rs`
   `*_tag_frame` (17) + `*_plate` (6) + `*_view(width)` (14), and
   `holla-ui/src/app.rs:377-436` plate predicates, `:1690-1706` status chain,
   `:439-996` status painters, `:1613-1660` header overrides,
   `screens/home.rs:62-74/350-353` tag-query seeding. Last because the plates
   are also the fixture story for the preview binary, so the replacement must
   keep scenario replay honest (drive from `World` state, not clock equality).

After those four, the residual list is: Jackin raw-buffer cockpit + plate
recolors (J32-J37), Jackin invisible cursor controls (J39-J42), Jackin
`repair_*` (J43-J46), TablePro `legacy_tabs`/explorer/grid chrome
(T7-T12, T18), Showcase remaining cell tables (S5-S16) and the
`terminal.rs:259` tick gate, and the Showcase size dispatches (S17-S23).

---

## 8. Counting summary

| App | `historical_*` / `draw_historical_*` | `*_plate` / plate selectors | Fixed cell/frame tables | Exact size / colour / scenario / clock predicates | Repaint-after-painter / recolor | Raw buffer | Forced cursor |
|---|---|---|---|---|---|---|---|
| Showcase | 5 (`paint_historical`, 2× `historical_palette`, 2 call paths) | 0 | 12 (2 198 runs total) | 8 files, 24 sites | 6 (`paint_narrow_button`, `legacy_table`, 5 `legacy_*`, `baseline_nav_scroll`, trees rows) | 0 | 0 |
| TablePro | 0 | 0 | 0 tables (chrome via `thread_local`) | 3 (2×120×40, 1 frozen clock) | 11 (legacy tabs, 7 explorer repairs, legacy filter, grid header/scrollbar/focus) | 0 | 2 |
| Jackin | 14 `historical_*` + 12 frame predicates | 2 (`launch_dialog_plate`, `capsule_*_view`) | 6 (3 `ENTRIES` + tabs + prelude rows + cockpit constants) | 81 width/height comparisons; ~22 full frame-identity predicates | 4 `repair_*` + 2 legacy branches + help overpaint | 4 `Buffer` paints + 3 blits | 5 |
| Holla | 0 | 19 plate predicates + 14 `*_view` | 0 (hand-painted plates) | 17 tag frames (clock 80 ms), 37 size comparisons, 1 env var | 16 status painters + 7 chrome branches | 0 | 7 |

Highest-risk files by bypass density:

1. `crates/jackin-preview-app/src/app.rs` — 22 frame-identity predicates and a
   22-call dispatch tail (`:11045-11221`).
2. `crates/showcase-demos/src/pages/taskrunner.rs` — 4 cell tables (637 runs)
   and the `historical_palette` bypass.
3. `crates/showcase-demos/src/pages/terminal.rs` — 4 cell tables (1 276 runs)
   plus the `UpdateCause::Tick` settle gate.
4. `crates/holla-ui/src/app.rs` — 19 plate selectors and the 17-branch status
   chain.
5. `crates/holla-sim/src/catalogue.rs` — 17 clock-gated tag frames driving
   every plate above.
6. `crates/tablepro-ui/src/app.rs` — shared-widget repaint layer plus eight
   `thread_local` draw-order channels.
