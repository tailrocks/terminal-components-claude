# Slice 7 — Jackin Preview migration plan

## 0. Scope, baseline, ownership

**[F] Files owned by this slice** (`COMPONENT_ARCHITECTURE.md:4077-4080`): `apps/jackin-preview/**` in full. Current tree: `src/bin/jackin_preview/{main.rs, app.rs, arbiter.rs, clock.rs, scenario.rs, rain.rs, perf_tests.rs, app_tests.rs, app_tests_chrome.rs, visual_tests.rs}`, `screens/{mod,manager,capsule,cockpit,inspect,config,editor,settings,accounts,usage,prelude,modals}.rs`, `domain/**`, `sim/**`, plus `tests/baselines/jackin.txt` (moves to `apps/jackin-preview/tests/baselines/jackin.txt`).

**[F] Sizes**: `app.rs` 2662 lines; `screens/modals.rs` 2426 lines; `rain.rs` 952; `screens/mod.rs` 337 (the `Screen` trait is `screens/mod.rs:231-328`, **23 methods** — the app-audit's "20" is wrong; recount below).

**[F] Baseline numbers** (`tests/perf_baseline.txt:6,7,17,3`):
`frame_jackin_capsule_4panes_120x40 = 1 080 602 allocs/frame` (the doc's §16.6 "≈480 000" is stale), `frame_jackin_manager_100rows_120x40 = 499 allocs`, `key_jackin_manager_move = 132 allocs/key`, `capsule_pane_clone_4x2000 = 571 572 allocs`. Slice-7 targets are unchanged: `< 200`, `< 60`, `0`, and deletion of the clone benchmark (`COMPONENT_ARCHITECTURE.md:2088,2087,2092,2114`).

**[F] Slice 7 runs in parallel with Slice 6** (`:4104`); a needed library change pauses the slice for fresh Opus adjudication.

---

## 1. Screen-by-screen migration table

Every "plumbing deleted" entry is code that must be **gone**, not wrapped. `«tui»` = `crates/tui/src/`, `«jk»` = `apps/jackin-preview/src/`.

### 1.1 Host chrome

| # | Current implementation (`file:line`) | Target composition | Plumbing deleted | Tests that must stay green |
|---|---|---|---|---|
| H1 | Row-0 host menu bar + right-aligned strip: `app.rs:838-880` `draw_host_menu` (reads `host_menu.areas.iter().map(r.right()).max()` to find the leftover rect, `:843-851`) | `MenuBar` (library, §14.1) drawn by the shell + `StatusBar` (merged `segments`, §20.10-9) for the right side | `self.host_menu.on_hover(ctx.interaction.hover)` (`:841`); the `areas`/`brand_area` geometry read-back (`:843-850`); `segments::render(rest, buf, ctx, &[], &right, t.canvas)` (`:879`) | `chrome.rs:22` (row 0 has `jackin❯ File … Help`, not the identity line), `app_tests.rs:129-135`, visual `start-*` |
| H2 | Non-host identity strip: `app.rs:2433-2516` `draw_strip` (Lockup + `left`/`right` `Segment` vectors, 4 clickable ids `STRIP_USAGE/ACCOUNTS/SETTINGS/HELP`, `app.rs:46-50`) | one `StatusBar` with `StatusItem` ids; the four clickable segments become `PartRef`s of `STRIP` | manual `Segment::clickable(WidgetId)` + the four `if id == STRIP_*` arms in `on_mouse` (`app.rs:1662-1677`) | visual `start-launch-running`, `cockpit-running`, `start-hard-cases` |
| H3 | Footer / hint bar: `app.rs:2518-2642` `draw_footer` — a 9-arm `match self.modals.last()` hand-built hint table (`:2521-2590`), an editing probe over screen **and** modal (`:2598-2604`), the `EDIT` badge (`:2605-2609`), status truncation (`:2622-2623`), still-inside override (`:2612-2621`), then `HintBar::resolve` over 4 layers (`:2625-2637`) | derived `HintBar` (§13.1): top layer ▸ mode ▸ focused component's visible bindings ▸ screen extras (`Screen::hints`) ▸ global fallback. `badge` from `StateFlags::EDITING` on the focused owner; `status` from `Jx` | the whole 9-arm modal hint table; the `is_editing()` fan-out; the second footer pass at `app.rs:2366-2367` | `chrome.rs:110` **incl. `t.matches("Enter Choose").count() == 1`** (`chrome.rs:124`), `chrome.rs:52,83,128` |
| H4 | Too-small screen: `app.rs:2283-2310` (4 centred lines, `"Terminal too small"`, `"Need {MIN_WIDTH}×{MIN_HEIGHT}, have {w}×{h}"`, `q Quit`, brand lockup) | library `TooSmall<'a>` (§18.3 #21) with `min_size` from `App::min_size()` (`§17.0 A1`) | `self.too_small` field (`app.rs:153`) and its key gate (`app.rs:506-511`) | `app_tests.rs:264-270`, visual `too-small` (60×18) |
| H5 | Construct-state label: `app.rs:2417-2431` `construct_state` | Jackin domain; feeds one `StatusItem` | — | `chrome.rs:26` (`!top.contains("inside the Construct")` on Capsule) |

### 1.2 Menus

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| M1 | Host `MenuBar` construction per route: `app.rs:699-743` `build_host_menu` + `sync_host_menu` (`:745-750`) | `MenuItem` gains typed `ActionKey` + `Chord` (§14.1, §20.10-6); one const table per route | `MenuItem::shortcut(&'static str)` display strings | `app_tests.rs:618-641` |
| M2 | **`run_host_menu`: dispatch by re-synthesising key presses** — `app.rs:754-813`, e.g. `"New workspace…" => return self.handle(key(KeyCode::Char('n'), NONE))` (`:779`), 12 such arms | `ActionKey` → one `match` on typed keys; the same `Chord` is registered as the binding, so menu and keymap cannot disagree | every `self.handle(key(...))` re-entry; the `editing_screen` special case (`:755, :759-768`) | none today directly — add `jackin::menu_items_dispatch_without_key_synthesis` |
| M3 | Capsule `MenuBar`: `capsule.rs:228-287` `build_menubar` (5 menus) + label dispatch `run_menu` (`capsule.rs:368-472`, 24 label arms) | same as M2 | label-string dispatch | `chrome.rs:43-71` (**View menu item order is load-bearing**: `chrome.rs:135-139` reaches *Inspect changes* by `Right,Right,End`), `chrome.rs:65-70` (`View → Usage`) |
| M4 | Tab context menu + brand menu: `capsule.rs:290-324` (`ContextMenu … .anchor(area, Placement::Below)`), stored as `tab_menu: Option<(usize, ContextMenu)>` with `usize::MAX` sentinel for the brand menu (`capsule.rs:83, :298`) | `ContextMenu` as `Popover` layer content with `Anchor::Rect{side: Below}`; the `usize::MAX` sentinel becomes two distinct layer ids | `Placement` enum (merged into `Anchor`, §18.1); `tabs.areas.get(tab)` geometry read-back (`capsule.rs:304`) | `chrome.rs:74-107` (right-click **and** `Ctrl+B m`), visual `capsule-tab-context-menu` |
| M5 | **[F] Menu/keymap drift, live today**: `capsule.rs:258` advertises `MenuItem::new("Container info").shortcut("Ctrl+B i")`, but `handle_prefix_cmd` (`capsule.rs:1266-1321`) has **no `'i'` arm** — `Ctrl+B i` falls to `capsule.rs:1319` `cx.status("Not a prefix command: i")` | M3 fixes it structurally | — | new: `conformance::menu_bar::bindings_match_handled_keys`, `conformance::conflicting_visible_bindings_are_reported` |

### 1.3 Tabs

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| T1 | Capsule tab strip **rebuilt inside `render`**: `capsule.rs:1459-1462` `let first = self.tabs.first; self.tabs = Tabs::with_items(STRIP, items); self.tabs.first = first; self.tabs.set_active(d.active);` | `Tabs<'a, T, K, R>` with `ItemKey` keys + reconciling `set` (§12.4, §20.10-13); items borrowed from `Daemon.tabs` per phase | the rebuild-and-rescue idiom; `TabItem` owned `String`s (`capsule.rs:1436-1457`) | `chrome.rs:31,93` (`row 2` shows `Shell` / `ops`), `app_tests.rs:879` (`tabs.len() == 2`), visual `capsule-*` |
| T2 | Editor tabs: `editor.rs:106` `tabs: Tabs`, `TAB_NAMES` (`editor.rs:51`), 5 `EdTab` variants | `Tabs` over a `const [EdTab; 5]` with `ItemKey::num` | — | `app_tests.rs:497-535, 1028-1107`, visual `editor-general…editor-accounts` (5 digests) |
| T3 | Settings tabs: `settings.rs:106` `Tabs::new(TABS, &TAB_NAMES)` (`settings.rs:40`) | as T2 | — | `app_tests.rs:581-615`, visual `settings` |

### 1.4 Forms

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| F1 | `FormDialog` + `FormField` + `FieldKindW` + `FieldValue`/`FormValues`: `modals.rs:787-1541` | library `Form` + `FieldSpec`/`FieldKind`/`FormData`/`FormState`/`FormAction` (§15.1); Jackin supplies three `FormData` impls: `«jk»/screens/accounts.rs::AccountDraft`, `«jk»/screens/config.rs::{MountDraft, EnvDraft}` | `values() -> FormValues` (`modals.rs:1039-1044`) — the secret-cloning channel; `set_text` rebuilding a `TextInput` (`:1004-1016`); `any_open_select` guard (`:1068-1072`); the manual Tab/BackTab (`:1084-1091`); the Left/Right action ring (`:1213-1230`); the 22-line hit re-registration block (`:1491-1512`); the open-select deferred re-render (`:1514-1539`); `content_height`/`f.height()` arithmetic (`:871-880, :1328-1334`); focus-follow scroll inside `render` (`:1356-1371`) | `app_tests.rs:273-356, 359-397, 538-578, 646-1025, 1137-1198`; visual `accounts-form`, `accounts-1password-step-1` |
| F2 | Accounts add/edit form assembly + `FormCtx` (`accounts.rs:79-85`), `FORM` id (`accounts.rs:48`), field names reconstructed by tests as `FORM.sub("save"/"provider"/"op")` | `Form` over `AccountDraft`; test addressing becomes `accounts::FORM.part(Part::custom("save"))` via `Harness::area_of` (§16.4 mapping 3) | `form_changed` fan-out through `Screen::form_changed` (`screens/mod.rs:304`) and `App::form_changed`'s take/replace dance (`app.rs:1309-1331`) | `app_tests.rs:309, 319, 322, 340, 381, 648` |
| F3 | Editor/Settings env & mount child forms: `config.rs:28-31` imports `FormDialog, FormField, FieldKindW, FieldValue, FormValues`; `Editing` enum routes results back to a row (`config.rs:183-192`) | `Form` inside a `Dialog` body slot; `Editing` stays domain | `Request::WithForm(Box<dyn FnOnce(&mut FormDialog)>)` (`screens/mod.rs:190`, applied at `app.rs:1969-1975`) — the whole escape hatch disappears | `app_tests.rs:552-578` (`editor.cfg.form.save`), `app_tests.rs:1164-1191` |

### 1.5 Trees / keyed collections (J9)

**[F] Six hand-rolled row models in Jackin**: `manager.rs:51-62` `Row{key: RowKey, depth, glyph, glyph_tone, label, meta, meta_tone, trailing, expandable}` + `DetailRow` (`manager.rs:65-71`); `accounts.rs:65-77` `Row{sel: Sel, depth, star, label, health, meta, meta_tone, faint, expandable, expanded}`; `config.rs:161-172` `Row{key: RowKey, change: Change, cells, problem, faint, header, folded, meta}`; `usage.rs:31-39` `Row` + `DetailLine = (String, Tone, Option<(u8, MeterTone)>)`; `editor.rs:80-99` `AcctRow`/`RoleRow`.

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| K1 | `ManagerScreen` rows + `build_rows` (`manager.rs:142+`), `RowKey` (`manager.rs:43-49` — **already a value key, the model the library lacks**) | `Tree<'a, Row, K, R>` with `ItemKey::text(RowKey)` + `.row(fn)` painting glyph/label/meta/trailing through `RowUi` | per-frame `build_rows` from key handlers; `scroll`/`detail_scroll` `ScrollState` fields (`manager.rs:77-78`) → `ScrollRegion` | `app_tests.rs:158-176`, `:1110-1134`, visual `manager-expanded-detail`, `manager` @ 80/100/160 |
| K2 | Accounts tree (`accounts.rs:91-93`) with fold set `folded: HashSet<UsageSurface>` | `Tree` keyed on `Sel` | — | `app_tests.rs:273-397, 618-641`, visual `start-accounts-mixed` |
| K3 | `ConfigTabs` mounts/envs `ListState` ×2 (`config.rs:174-180, 199-200`), `Change` glyph slot (`config.rs:128-145`) | `List` + `RowDecor{marker, tone}` (J10) — `Change::{Added,Modified,Removed}` maps to `GlyphRole::{Inserted, Dirty, Deleted}` | `Change::glyph()`; the second change-decoration implementation | `app_tests.rs:497-535, 1137-1198`, visual `editor-mounts`, `editor-environments` |
| K4 | Usage list + detail (`usage.rs:42-52`) with per-frame `DetailLine` tuples | `List` + `Meter` with `MeterTone::from_ratio` (J12) | `usage.rs:39` owned tuple per frame; the duplicated tone match at `capsule.rs:2568-2577` and inline at `capsule.rs:1756-1764` | `app_tests.rs:400-413, 637-641`, visual `usage` |
| K5 | Editor role rows / account rows (`editor.rs:87-99, 113-118`) | `List` with keys | `role_targets`/`picker_targets` parallel vectors (`editor.rs:123`, `manager.rs:91`) | `app_tests.rs:1028-1107, 1137-1198` |
| K6 | `InspectChanges` tree + `leaves: Vec<(Vec<usize>, usize)>` path→index side table (`inspect.rs:73-74`) | `Tree` with `TreeNode::keyed(ItemKey)`; `leaves` deleted | the path→index side table | `chrome.rs:133-158`, visual `capsule-inspect-changes` |

### 1.6 Account and usage surfaces

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| A1 | `AccountsScreen` master/detail: `Split::new(34,30,40)` + `Splitter` + `seam_container: Rect` (`accounts.rs:96-98`) | `SplitPane` (§18.3 #20) owning its container rect from its own draw | `seam_container` field; `narrow`/`drawer_open` hand-rolled responsive collapse (`accounts.rs:101-102`) → `SplitPane` collapse mode | `app_tests.rs:273-397` |
| A2 | Masking: `domain::account::{masked, tail_of, fingerprint}` (`accounts.rs:36`) and a **second** implementation `domain::workspace::mask` + `unmasked: HashSet<(Option<RoleName>, String)>` (`config.rs:37, :202`) | `Secret` + `SecretPolicy` for the **in-flight draft only**; `domain::workspace::mask` stays for the **stored/persisted display value** — see §6 note on `"************1234"` | `TextInput.reveal_tail` re-specified to a synthetic tail (§18.2 `input`) | `app_tests.rs:373-392, 538-578` |
| A3 | Usage read-only projection + `m` hand-off (`usage.rs`, `app.rs:604-615`) | unchanged product behaviour; `u`/`c`/`s` chords move to the app `KeyMap` (§13.1) | the `host && !editing` chord ladder `app.rs:598-629` | `app_tests.rs:400-413` |

### 1.7 Status bars

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| S1 | Capsule status bar: `capsule.rs:27` imports `StatusBar, StatusItem`; drawn `capsule.rs:1671-1804` with an inline meter and the `PR #482` chip | the merged `StatusBar` (§20.10-9) — Jackin is already its only consumer, so this is the reference consumer | inline `MeterTone` match (`capsule.rs:1756-1764`) | `chrome.rs:33-38` (`row 38` contains `PR #482`, `%` and `━`) |
| S2 | Host `segments::render` (`app.rs:879, :2515`) | absorbed by `StatusBar`; priority values (`priority(1..9)`) map to `StatusItem` priorities | `segments` module entirely | visual `start-*`, `manager` @ 80×24 (drop order visible) |

### 1.8 Hint bars — see H3. Additional obligation: `Screen::hints(focus, w) -> Vec<Hint>` (23 per-screen implementations, e.g. `capsule.rs:2461-2520`, `prelude.rs:497-524`) collapse to product-level extras only (§13.1, §20.10-10).

### 1.9 Pickers

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| P1 | Launch/agent picker + `picker_targets: Vec<(Agent, Option<String>)>` (`manager.rs:91`) | `Picker` with `ItemKey`; the parallel vector dies | `Screen::picker_items(&tag, &query, w)` (`screens/mod.rs:295-302`) and `App::refresh_picker` (`app.rs:1283-1307`) — filtering becomes ordinary `update` work over `FilterList` | `app_tests.rs:1110-1134` (**agents without an account are omitted, not disabled**) |
| P2 | Capsule spawn/account/split/close pickers + `picker_accounts`/`picker_agents`/`palette_cmds` (`capsule.rs:87-90`) | `Picker`/`CommandPalette` keyed | three parallel vectors | `app_tests.rs:872-888`, `chrome.rs:161-186` |
| P3 | Command palette (`PALETTE: [&str; 20]`, `capsule.rs:102-121`) | `CommandPalette` over `ActionKey` | label-string dispatch into `run_menu` | `chrome.rs:161-186` (**wheel down then wheel up restores the frame byte-for-byte**; selection stays on item 0) |
| P4 | Role picker over >100 roles (`config.rs:210 role_targets`) | `Picker` keyed on `RoleName` | `role_targets` | `app_tests.rs:1164-1191` |
| P5 | `OpFlow` 4-stage 1Password chain: `modals.rs:1546-1943` | library `PickerChain` (J8) + `«jk»/screens/op_flow.rs` for the domain (see §3) | `App`'s `Modal::Op` arms (`app.rs:419, 1485-1492, 1736, 1779-1782, 1915-1922`) | `app_tests.rs:273-356, 682-720`, visual `accounts-1password-step-1` |

### 1.10 Dialogs — see §3.

### 1.11 Launch progress

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| L1 | `CockpitScreen` 11-stage rail: `cockpit.rs:49 rail: StepRail`, `Stage::ALL` (`cockpit.rs:92`) | `Steps` (display rail with frontier — the meaningful difference preserved, §12.4) | — | `app_tests.rs:179-200, 203-216, 1201-1238`, visual `cockpit-running`, `cockpit-failure` |
| L2 | Build log: `cockpit.rs:51 log: TextViewport` with `wrap(true).max_lines(5_000)`, `follow = true` (`cockpit.rs:93-94`), `log_area: Rect` field (`cockpit.rs:61`) | `TextViewport` with caller-owned `ViewportState`; `log_area` deleted (`cx.area(LOG)`) | `log_area` | `app_tests.rs:853-857` (`Docker build`, PageUp, End, Esc) |
| L3 | Cockpit atmosphere: `rain::paint_atmosphere` (`rain.rs:804-849`) called with `exclude: &[Rect]` | stays app-specific; consumes `Role` (see §4) | — | visual `cockpit-running` |
| L4 | Credential-origin projection (`cockpit.rs:147-194`) | domain, unchanged | — | `app_tests.rs:1201-1238` (the container receives the whole effective account set) |

### 1.12 Diff and inspection

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| D1 | `InspectChanges` implements `CustomModal` (`inspect.rs:61-89`); `DiffView` fed from a **parallel** model `sim::changes::{ChangeSet, ChangedFile, DiffStatus}` (`inspect.rs:27`) | a screen-owned component drawn into `ui.layer(...)`; `DiffView` accepts `&dyn DiffSource` so `ChangedFile` feeds it directly (§14.1, §18.2 `diff`) | the `CustomModal` impl; the `ChangedFile → DiffFile` conversion; `tree_area`/`diff_area`/`container` cached rects (`inspect.rs:80-82`) | `chrome.rs:133-158` (`m` compact/advanced, `d` unified/review, Esc×2 returns), visual `capsule-inspect-changes` |

### 1.13 Capsule components

| # | Current | Target | Deleted | Tests |
|---|---|---|---|---|
| C1 | **Per-frame viewport clone**: `capsule.rs:1567` `let mut term = pane.term.clone();` then `ctx.inert = true; term.render(...); ctx.inert = false;` (`:1575-1577`) and a **manual cursor re-placement** (`:1578-1590`), paired with `prime()` writing last-frame geometry back into the world (`capsule.rs:1408-1416`) | render directly from the daemon's pane with caller-owned `ViewportState` (§20.9-10); `set_area`/`prime`/the `inert` dance all disappear | `prime`, `TextViewport::set_area`, the cursor re-placement block, `saved_focus` dead binding (`:1573, :1591`) | `app_tests.rs:868-933` (typing, scrollback, drag-select, double-click word select, `y` copy), perf `frame_jackin_capsule_4panes_120x40 < 200` |
| C2 | Pane frames drawn with raw `ratatui::widgets::Block` (`capsule.rs:1496-1500`) and seam glyphs picked by hand (`capsule.rs:1613-1618`) | `Ui::frame` + theme `BorderSet`; seams become `SplitPane`'s `Part::SEAM` with hover/pressed glyph weight | raw `Block`/`Borders`/`BorderType` | visual `capsule-*` |
| C3 | Mode priority `Dialog › Drag › Select › Prefix › Normal` (`capsule.rs:56-63`), `Ctrl+B` prefix with 2 s timeout (`capsule.rs:54, 505-507, 1252-1323`) | **stays domain** (§14.2); prefix commands declared as a `Binding` table so the help/hint text is derived, not hand-written | the hand-written prefix list `capsule.rs:2478-2492` | `app_tests.rs:222-223, 872-900, 943-949`, `chrome.rs:95-106, 126-129` |
| C4 | `UsageDialog` — the second `CustomModal` implementor, constructed at `capsule.rs:1146-1152` | screen-owned component in a layer | `CustomModal` trait (`screens/mod.rs:66-95`) deleted with its last user | `chrome.rs:68-70` (View → Usage), `app_tests.rs:943-946` |
| C5 | Takeover screen (`capsule.rs:2400-2428`), zoom (`:825-838`), split minima re-checked by hand (`:654-694, :804-821`) | `SplitPane` enforces minima once; takeover stays domain | the duplicated minimum checks | `app_tests.rs:881-900` |

### 1.14 Scrollable and split surfaces

**[F] Four copies of master-detail + draggable seam**: `manager.rs:114-115` (`Split::new(32,28,40)` + `Splitter` + `seam_container: Rect` at `:95`), `accounts.rs:96-98`, `inspect.rs:111-112`, plus the Capsule pane tree. All become one `SplitPane` (§18.3 #20). **[F]** Every screen also carries raw `ScrollState` fields (`manager.rs:77-78`, `accounts.rs:92-93`, `usage.rs:45-46`, `editor.rs:90,116`, `config.rs:178`, `modals.rs:927,1963,1987,2288`) plus per-site `scrollbar::render_vertical` + `scrollbar::id_for` routing (`modals.rs:417, 1455, 2138, 2141, 2252, 2406`; `app.rs:1558`) — all replaced by one `ScrollRegion` component (§12.2, `Part::TRACK`/`Part::THUMB`).

---

## 2. The `Screen` trait reduction — 23 methods → 6

**[F] Current trait, `screens/mod.rs:231-328`, exactly 23 methods**: `on_key`(232), `on_click`(233), `on_double_click`(236), `on_drag`(246), `on_secondary`(250), `on_press`(261), `on_release`(264), `on_wheel`(273), `on_paste`(276), `on_tick`(279), `on_msg`(282), `on_modal`(285), `picker_items`(295), `form_changed`(304), `render`(305), `hints`(306), `crumb`(308), `strip_right`(310), `is_editing`(313), `animating`(316), `enter`(320), `primary_focus`(322), `on_esc_top`(324). Eleven of them take `&mut Cx` carrying `focus: &mut Focus, ring: &FocusRing` (`screens/mod.rs:193-197`).

**Target (exact, per `COMPONENT_ARCHITECTURE.md:272-287`):**

```rust
// «jk»/screens/mod.rs
pub trait Screen {
    /// The only input entry point. Intents are already resolved to (owner, part).
    fn update(&mut self, cx: &mut Cx<'_>, jx: &mut Jx<'_>, w: &mut World) -> Response<()>;
    /// Pure paint. `&self` makes a semantic mutation a compile error.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, w: &World);
    /// Product-level hints only; component bindings are contributed automatically.
    fn hints(&self, _w: &World) -> HintLayer { HintLayer::empty() }
    fn crumb(&self, w: &World) -> String;
    fn primary_focus(&self) -> Option<Id>;
    fn on_esc_top(&mut self, _cx: &mut Cx<'_>, jx: &mut Jx<'_>, _w: &mut World) -> Response<()> {
        jx.go(Go::Manager);
        Response::consumed().repaint()
    }
}
```

`Jx<'_>` is Jackin's own product channel (today's `Cx` minus `Focus`/`FocusRing`): `go(Go)`, `status`, `error`, `open_layer`-shaped helpers, `close`, `copy`, `help`. `Request::{Open, Close, WithForm}` and `Modal`/`ModalTag`/`ModalResult` are deleted (§9, J13).

**Per-method subsumption:**

| Removed method | Subsumed by |
|---|---|
| `on_key` (232) | `Intent::Key` drained in `update` via `cx.intents(id)`; component bindings resolve first (§3.3 steps 3, 7) |
| `on_click` (233) | `Intent::Pointer{phase: Phase::Click, part}` — already resolved to `(owner, part)`; no `owns`/`locate` |
| `on_double_click` (236) | `Phase::DoubleClick`, runtime-owned 500 ms window (§8.6). Deletes `App`'s `last_click: Option<(WidgetId, i64)>` (`app.rs:157`) and the fallback at `app.rs:1636-1694` |
| `on_drag` (246) | `Phase::Drag` under a live `Capture` (§8.2); deletes `App::on_mouse`'s drag arm (`app.rs:1550-1571`) |
| `on_secondary` (250) | `Phase::Secondary` |
| `on_press` / `on_release` (261, 264) | `Phase::Press` / `Phase::Release`; the "valid completed click" rule moves to the runtime |
| `on_wheel` (273) | `Intent::Wheel{axis, delta, part}` routed to the innermost scrollable on the top layer (§8.3); deletes `hit_scroll` plumbing at `app.rs:1743` |
| `on_paste` (276) | `Intent::Paste(&'f str)` delivered only to the focused owner that declared `EDITING` (§3.3 step 3) |
| `on_tick` (279) | `App::update` on a tick pass, then screen work in `update`. **See Q3** — the accepted `App` trait has no tick hook |
| `on_msg` (282) | Domain `Msg`s drained by `App::update` before screen `update` (§3.4). **The fan-out must survive**: `app.rs:457-462` offers an unconsumed msg to `manager` and `accounts` even when they are not the active route → becomes two inherent calls `ManagerScreen::apply_msg` / `AccountsScreen::apply_msg` from `App::update`, not trait methods |
| `on_modal` (285) | `cx.layer_event(id) -> Option<LayerEvent::{Opened, Dismissed(reason), Closed(ActionKey)}>` plus the component's own `Response<XAction>` |
| `picker_items` (295) | ordinary `update` work over the screen's own `FilterList`/`PickerChain` state; deletes `App::refresh_picker` (`app.rs:1283-1307`) |
| `form_changed` (304) | `FormAction::{Changed, Committed}(Id)` handled in the owning screen's `update`; deletes `App::form_changed`'s `mem::replace` with a throwaway `FormDialog` (`app.rs:1319-1330`) |
| `render` (305) | `draw(&self, ui, area, w)` — `&self` makes render-time mutation a compile error |
| `strip_right` (310) | **Open — see Q1.** Recommended: `jx.strip_item(StatusItem)` calls issued from `update`, with the shell caching the last set so a pure repaint is not stale |
| `is_editing` (313) | `StateFlags::EDITING` on the focused owner + `swallows_typing` on the focus entry; drives the `EDIT` badge and the Capture-phase bare-`Char` skip (§11.4, §13.1) |
| `animating` (316) | `cx.request_repaint_after(Duration)` (§8.5); deletes `App::animating` (`app.rs:299-313`) and `Route::tick_ms` as a *repaint* heuristic (see §5) |
| `enter` (320) | inherent per-screen `enter` called from `App::go`, or a `LayerEvent::Opened`-shaped route-change signal. Not a trait method |

`hints`, `crumb`, `primary_focus`, `on_esc_top` are retained with the signatures above.

**Also deleted from `screens/mod.rs`:** `CustomModal` (66-95), `Modal` (98-108), `ModalResult` (111-131), `ModalTag` (41-63), `Request` (180-191), `Cx` (193-227). `Go` (134-178) and `plural` (330-336) stay.

---

## 3. `modals.rs` decomposition (2426 lines, seven modal types)

| Item | Lines | Disposition | Target file | Notes |
|---|---|---|---|---|
| `modal_frame` (dim + `begin_modal` + rounded frame + title + right-aligned meta + `hits.register("modal.surface")`) | `36-96` | **delete** → library | `«tui»/layer.rs` + `«tui»/components/panel.rs` (J1) | Four copies of these 40 lines exist repo-wide. `ctx.begin_modal()` (`:60`) and the barrier-after-children ordering disappear (§9.1); the shared `WidgetId::of("modal.surface")` (`:94`) is deleted; the dim loop (`:53-59`) becomes the single backdrop implementation that excludes the footer row (§20.10-8) |
| `hint_row` | `98-106` | **delete** | — | derived `HintBar` |
| `FileBrowser` | `110-563` | **app composition** (J6) | `«jk»/screens/file_browser.rs` | Library: `Dialog` body slot + `Form` (`FieldKind::Text` path, `FieldKind::Check` read-only) + `List<FsEntry>` + `Action` row. Domain (stays): `World.fs` filtering + sort (`:167-204`), `expand`/`tilde`, `w.github` URL resolution (`:238-241`), `url_mode` toggle (`:266-278`), `resolving` spinner (`:457-467`). Deleted: 6 derived child ids (`:148-155`), the manual `Tab`/`BackTab` fallback (`:382-389`), the double-click emulation `let was = self.list.cursor == i` (`:407-415`), `scrollbar::id_for` (`:417`), the 7-line hit re-registration block (`:551-557`), `pub area: Rect` (`:133`) |
| `ChoiceDialog` | `569-783` | **move to library** | `«tui»/components/dialog.rs` as `Dialog::choice(...)` (J3) | Library: question lines + `RadioGroup` + N actions + `cancel_index` + per-option tone. Deleted: the modular Left/Right button ring (`:683-694`), the `for i in 0..radio.options.len()` option-id scan (`:700-705`), the option re-tone overpaint that writes over `RadioGroup`'s own output (`:754-767` — replaced by `RowDecor::tone`), the 8-line hit re-registration (`:774-781`), `stepper(&str)` (`:617-620`) → `Wizard` (J7) for the Prelude |
| `FormDialog` / `FormField` / `FieldKindW` / `FieldValue` / `FormValues` / `FormEvent` | `787-1541` | **move to library** | `«tui»/components/form.rs` (J2, §15.1) | The strongest candidate. See §1.4 F1 for the deletion list. **`FormValues` is deleted outright, not ported** (§15.1 F5). Jackin keeps three `FormData` impls |
| `OpFlow` + `OpStep` | `1546-1943` | **split** | library `«tui»/components/picker_chain.rs` (J8) + `«jk»/screens/op_flow.rs` | Library: stage list, `EmptyState::Loading{label}` / `EmptyState::Error{message, detail}` with retry, breadcrumb scope, back-one-step, `begin_load` latency hook. Domain (stays in `«jk»`): `OpStep` account→vault→item→field, `SimOnePassword` calls (`:1686-1782`), `set_error`'s `OpError` → message/detail mapping (`:1654-1677`), `concealed_only` field filter (`:1774`), `crumb()` composition (`:1596-1608`), the per-step hint table (`:1918-1938`) → derived |
| `InfoDialog` + `InfoResult` | `1947-2280` | **move to library** | `«tui»/components/dialog.rs` as `Dialog::facts(...)` (J4) | Library: `Props` rows + copyable rows + scrollable detail slot + extra actions + error title. **Copy must not carry a value out of the component**: `InfoResult::Copy(String)` (`:1950`) becomes `PropsAction::Copy(ItemKey)`; the owning screen resolves the value from its own model and writes `world.clipboard`. Deleted: `copy_values: Vec<Option<String>>` (`:1970`), the `y` "copy the first copyable value" scan (`:2110-2116`), two `scrollbar::id_for` arms (`:2138, :2141`) with hand-computed tracks (`:2142-2153`), the `area_of`-then-re-register loop (`:2274-2278`) |
| `HelpOverlay` | `2284-2425` | **move to library** | `«tui»/components/help.rs` (J5) | Library: multi-column round-robin distribution (`:2363-2379`), scroll, scope label, position label. Jackin keeps **only** product extras: the 200-line per-route `sections` table at `app.rs:892-1206` collapses to the bindings the components already declare (§13.1). Deleted: `hint_row` (`:2423`), the hand-written `j/k/PageUp/PageDown` (`:2315-2339`) |
| `CustomModal` trait | `screens/mod.rs:66-95` | **delete** | — | Implementors: `InspectChanges` (`inspect.rs:61`) and `UsageDialog` (`capsule.rs:1146`). Both become screen-owned components drawn into `ui.layer(...)`, each with its own `update`/`draw` |

**Resulting file set** for the decomposition: `«jk»/screens/{file_browser.rs, op_flow.rs}` (new), and `modals.rs` **deleted**.

---

## 4. `rain.rs` and the theme boundary

### 4.1 [F] What rain consumes today

**Local `Tone` enum**, *not* `theme::Tone`: `rain.rs:52-57` `Ladder(u8) | Accent`, where `Ladder(4)` is primary and `Ladder(0)` is ghost.

**13 named palette fields**, in three places:
- `ladder_color` (`rain.rs:59-67`): `text_ghost, text_faint, text_muted, text_secondary, text_primary` (5).
- `style` (`rain.rs:70-86`): `accent, accent_hover, accent_pressed`, background `canvas` (4). **The number of accent dim steps is derived from the palette's ladder depth** — `dim > 2` returns `None`.
- `dim_buffer` (`rain.rs:102-172`) additionally: `success, focus, error, warning, accent_bg, surface` (6).

Plus `draw_hint` (`rain.rs:205-209`: `text_muted` bold, `text_faint`, `canvas`) and `draw_pill_bottom` → `Lockup` (`text_on_accent` on `accent`).

### 4.2 [F] `dim_buffer` is colour-identity matching, and it is already wrong

`rain.rs:119` `ladder.iter().position(|c| *c == fg)` against `[text_ghost, text_faint, text_muted, text_secondary, text_primary]`, then `:128` `fg == t.accent || fg == t.success || fg == t.focus`, `:133` `fg == t.error || fg == t.warning`, `:140-146` unmatched fallback, `:148-157` background comparisons against `canvas`/`accent`.

**[F] Three collisions exist in `Theme::junie()` at truecolor today:**
- `accent == focus == success == GREEN` (`theme.rs:167, 172, 177`).
- `border_subtle == text_ghost == WHITE_15` (`theme.rs:159, 165`) — so **every border cell already reverse-maps to ladder step 0** and is erased at `steps >= 1`.
- `disabled == text_faint == WHITE_30` (`theme.rs:164, 173`) — every disabled cell reverse-maps to ladder step 1.

**[I]** The last two are silent defects in today's handoff cross-fade; they are visible only in the `HandoffStage::CockpitDim/CapsuleDim` frames, which no test pins. They must be classified before the baseline is re-blessed.

### 4.3 Migration

1. **Keep `rain::Tone`** (goal §22.3 allows the effect layer to stay app-specific) but map it through `Role`, never through palette fields:
   ```rust
   // «jk»/rain.rs
   fn role(tone: Tone, dim: u8) -> Option<Role> {
       match tone {
           // rain's ladder index is REVERSED relative to FgStep's ordinal:
           // Ladder(4)=Primary … Ladder(0)=Ghost
           Tone::Ladder(i) if dim <= i => Some(Role::Fg(FG[(i - dim) as usize])),
           Tone::Ladder(_) => None,
           Tone::Accent => match dim {
               0 => Some(Role::Accent),
               1 => Some(Role::AccentHover),
               2 => Some(Role::AccentPressed),
               _ => None,
           },
       }
   }
   const FG: [FgStep; 5] = [FgStep::Ghost, FgStep::Faint, FgStep::Muted,
                            FgStep::Secondary, FgStep::Primary];
   ```
   Backgrounds become `Role::Surface(Surface::Canvas)` / `Role::AccentTint` (for today's `accent_bg`) / `Role::Surface(Surface::Surface)`.
2. **Replace `dim_buffer` entirely with `Ui::dim_layer(area, steps)`** (`COMPONENT_ARCHITECTURE.md:2313`, §11.6). `dim_layer` walks `FrameOut::roles` — the `Role` recorded per painted cell by `Ui`'s painting methods — and steps it down the ladder *semantically*. `rain::dim_buffer` (`rain.rs:102-172`) is deleted; `app.rs:2334, 2339` become `ui.dim_layer(area, n)`.
3. `fill_canvas` (`rain.rs:96-98`) → `ui.with_surface(Surface::Canvas, |ui| ui.fill(area, ui.surface_style()))`.
4. `put` (`rain.rs:88-94`) → `ui.paint_cell(pos, symbol, style)` (R3), so the per-cell role is recorded and `dim_layer` can read it. **This is the load-bearing change**: `dim_layer` only works on cells painted through `Ui`.
5. `draw_hint`'s self-clearing loop (`rain.rs:201-203`) stays — it is a deliberate "the hint owns its cells" rule.
6. `Starfield` (`rain.rs:306-469`) is unchanged: pure geometry + `xorshift`, resize re-creates the field (`rain.rs:456-459`) — deliberate, documented, must survive.
7. `rain.rs` is the **single documented exception** to `architecture::no_generic_component_copies_in_applications` (`COMPONENT_ARCHITECTURE.md:2037`) — the allow-list entry must be exactly `apps/jackin-preview/src/rain.rs`.

### 4.4 What breaks if a theme separates `accent`, `success` and `focus`

**[F]** Today the three-way test at `rain.rs:128` is degenerate because all three are `GREEN`. **[I]** With `Theme::paper()` (accent `#3b5bdb` indigo, success `#1f7a3d`, `focus = accent` by builder derivation, `COMPONENT_ARCHITECTURE.md:937, 1067`) the current code would:
- send `success`-coloured cells down the **accent** chain (`accent_hover`, `accent_pressed`), turning a green success glyph indigo mid-fade;
- with a theme where `focus != accent`, focus-ring cells likewise jump hue during the fade;
- under `ColorLevel::Ansi16`/`Mono` several ladder tokens collapse onto one `Color`, so `position(|c| *c == fg)` returns the **first** match and the fade becomes non-monotonic (a cell can get *brighter* as `steps` increases). **[F]** No test covers `dim_buffer` at a reduced colour level.

Under `Ui::dim_layer` none of this is possible: the recorded value is a `Role`, the step is index arithmetic on the `FgStep` ladder, and `Role::Success` is simply *not on the ladder* — the component decides its own degradation. **[I]** Decide and document: a non-ladder role (`Success`, `Warning`, `Danger`, `Info`) under `dim_layer` steps to `Role::Fg(FgStep)` at `4 - steps - 1` (the current `rain.rs:133-139` rule for `error`/`warning`) and erases at `steps >= 4`. That rule must live in `Ui::dim_layer`, not in Jackin.

**Acceptance:** `render::overlay::*` handoff digests under both `Theme::junie()` and `Theme::paper()` × `{truecolor, mono}`; a new `jackin::handoff_fade_is_monotonic_under_every_colour_level`.

---

## 5. The determinism contract

### 5.1 What must be preserved exactly

| Contract | Evidence | Obligation |
|---|---|---|
| **No wall clock** | `clock.rs:1-15`; `EPOCH_SECS = 1_788_401_640` (`clock.rs:7`); `Clock::advance` is a no-op when `!running` (`clock.rs:26-30`) | `Clock` moves unchanged. `world.clock` is the *only* time source; the runtime's own repaint deadline must never feed it |
| **`--motion paused` freezes ticks** | `main.rs:101-107`, `app.rs:315-320`, `world.clock.running = motion != Motion::Paused` (`app.rs:167`) | preserved verbatim |
| **Eight scenarios** | `scenario.rs:26-35` `Scenario::ALL`; start routes `app.rs:247-280` | all eight reachable; `Scenario::from_name` round-trip test (`scenario.rs:92-99`) stays |
| **Three motion modes** | `scenario.rs:55-85`; `JACKIN_NO_MOTION` resolution (`main.rs:101`, `scenario.rs:78-84`) | preserved; a *product* contract asserted by `app_tests.rs:139-155` |
| **Rain timing constants** | `TICK_MS=33` (`rain.rs:18`); `MOTION_SEED=0x4A41_434B_494E_5E5E` (`rain.rs:22`); `PHRASES` (`rain.rs:475-479`); `P1_LEN=64` asserted (`rain.rs:879`); `P2_LEN`,`P3_LEN`,`KNOCK_START`,`KNOCK_LEN`,`WARP_START`,`INTRO_END` (`rain.rs:488-494`); `REDUCED_HOLD=45` (`:495`); `GLITCH_PASS_TICKS=2`,`GLITCH_PASSES=5` (`:270-271`); `WARP_TICKS=95` (`:273`); `OUT_WARP`,`OUT_CAPTION`,`OUTRO_SALT` (`:643-646`); `HANDOFF_LEN=12` (`:851`); `handoff_stage` boundaries `0..=3 / 4..=5 / 6..=10 / _` (`:855-862`) | **none may change.** `COMPONENT_ARCHITECTURE.md:3945` lists "any change to the eight jackin scenario contracts, the rain timing constants, or the `format_universe_duration` wording" as a regression, not a §20.10 item |
| **Skip semantics** | `IntroState::skip` phrases→warp→done (`rain.rs:560-570`); `OutroState::skip` (`:741-746`) | preserved |
| **Outro caption wording** | `"You were in the Construct for 2 hours 14 minutes"` (`rain.rs:748-755`, asserted `rain.rs:907`); `format_universe_duration` two-largest-units (`rain.rs:665-682`, asserted `:909-911`) | byte-identical |
| **Arbiter contract** | `arbiter.rs:1-11`; tests `arbiter.rs:148-203` | unchanged; `arbiter.rs` is pure domain |
| **Frame pinning** | `App::for_scenario(scenario, motion, frame, theme)` (`app.rs:165`); `Scenario::OutroLast if frame > 0` jumps straight to Outro (`app.rs:203-214`); `frame >= rain::INTRO_END` skips the intro (`app.rs:219`); `CockpitScreen::seek(frame, …)` (`app.rs:262-273`, `cockpit.rs:141-145`) | the `frame` parameter and all three seek paths survive in `App::for_scenario` |
| **Two `--frame 282` runs byte-identical** | `app_tests.rs:146-148` | must stay green |
| **Paused frames never advance** | `app_tests.rs:149-154` | must stay green |
| **Starfield determinism** | `rain.rs:914-950` (buffer equality + repaint idempotence) | must stay green |

### 5.2 The virtual-clock advance — the one genuinely dangerous change

**[F] Today the clock advances by the *route's nominal interval*, not by real time**: `App::on_tick` (`app.rs:356-358`) does `let interval = self.route.tick_ms(true) as i64; let msgs = self.world.tick(interval);`, where `Route::tick_ms` (`app.rs:68-80`) is 33 ms for Intro/Outro/Handoff/Cockpit, 80 ms for Capsule, 80/200 elsewhere. `App::tick_interval` (`app.rs:315-320`) then reports the same number to the runtime as the *poll* interval.

**[I] Invariant for the migration:** the runtime's repaint deadline (`cx.request_repaint_after`, §8.5) replaces `tick_interval` as the *scheduling* mechanism only. `Route::tick_ms` must **survive as a Jackin-owned constant** used to advance `world.clock`, exactly as today:

```rust
// «jk»/app.rs, inside App::update on a tick pass
let interval = self.route.tick_ms(true) as i64;   // unchanged table
let msgs = self.world.tick(interval);             // unchanged
cx.request_repaint_after(Duration::from_millis(interval as u64));
```

If the clock is instead advanced by `design.motion.tick_ms` (80) or by wall-clock delta, **every one of the following breaks**: `visual_tests.rs:39` `FAILURE_TICKS = 77`, `:41` `RUNNING_FRAME = 20`, `:43` `OUTRO_FRAME = 150`, every `h.ticks(n)` count in `app_tests.rs` (≈40 sites), every fixture timestamp derived from `Clock::now_secs`, and the outro elapsed caption. This is the single highest-risk item in Slice 7.

**[I] Second invariant:** the tick cadence must remain *route-dependent*, because `rain::TICK_MS = 33` is the intro/outro/handoff/cockpit frame rate and `HANDOFF_LEN = 12` is counted in those ticks. A uniform `design.motion.tick_ms = 80` would make the intro run at 2.4× the wrong speed.

### 5.3 What the new runtime replaces

| Removed | Replacement |
|---|---|
| `App::tick_interval()` (`app.rs:315-320`) and the `Application::tick_interval` impl (`main.rs:129-131`) | `cx.request_repaint_after(Duration)` folded into a repaint deadline (§8.5) |
| `App::animating()` (`app.rs:299-313`) — a 6-clause disjunction over route, flash, screen, modal kind, jobs and daemon panes | each producer requests its own deadline: rain (`Route::tick_ms`), the flash timer (`design.motion.press_flash_ms = 140`), the status timer (5 000 ms, `app.rs:296`), `world.jobs`, `OpFlow`'s `loading_until` |
| `Screen::animating(&World)` (`screens/mod.rs:316`) | as above |
| `flash: Option<(WidgetId, i64)>` (`app.rs:149`) + the 140 ms window (`app.rs:647, 1635`) + `Interaction::flash` (`app.rs:323-326`) | runtime-owned press flash, cadence from `design.motion.press_flash_ms` (§11.2 A4) |
| `hover_suppressed` (`app.rs:148, 349, 1541-1542`) | runtime-owned (§8.6) |
| `intro_guard: u8` (`app.rs:158, 382-384, 514-516`) — swallows the first 3 ticks' worth of keys during the intro | **[I] must be preserved as a product rule** (it is why `runtime::drain_pending_input` at `main.rs:115` is not sufficient). Express it as a Jackin `KeyMap` Capture binding active for `Route::Intro` while `intro_guard > 0` |

---

## 6. The regression contract

### 6.1 `app_tests.rs` — 22 tests **[F]**

| # | Test | Line | Proves | Rewrite risk |
|---|---|---|---|---|
| 1 | `first_use_plays_intro_then_manager_and_no_replay_when_returning` | 115 | intro plays once; skip = phrases→warp→manager; `returning` joins without replay (`2 running`) | none |
| 2 | `reduced_motion_and_paused_frames_are_deterministic` | 139 | `Enter Continue`; **two `--frame 282` runs byte-identical**; paused frames never advance | none |
| 3 | `manager_navigation_expand_and_detail_focus` | 158 | tree expand; Tab→detail; Esc returns focus to `manager::TREE`; row click updates the crumb | **id addressing** (`:171`) |
| 4 | `launch_runs_all_stages_and_hands_off_to_the_capsule` | 179 | Cockpit→Handoff→Capsule; typing echoes | none |
| 5 | `launch_failure_returns_to_the_construct_when_another_instance_runs` | 203 | `Launch failed` + `Network`; Esc→Manager with `still running` | none |
| 6 | `detach_reconnect_and_final_exit_plays_one_outro` | 219 | `Ctrl+B d`; Enter reconnects; `Ctrl+Q`→unsaved-work choice→Outro caption→quit | **radio cursor≠value** (`:234-236`) |
| 7 | `still_inside_feedback_when_other_instances_remain` | 251 | `Still inside the Construct`; `running_count() == 1` | **radio** (`:255-257`) |
| 8 | `too_small_state_and_resize_recover` | 264 | 60×18 too small; 80×24 recovers | none |
| 9 | `accounts_register_with_a_1password_reference_and_never_render_the_secret` | 273 | full OpFlow chain; duplicate-source refusal; provider switch; save; refresh reports "still rate limited" | **id addressing** (`:309, 319, 322, 340`) |
| 10 | `accounts_plain_key_is_masked_everywhere_and_remove_asks_first` | 359 | raw key never rendered; 4-char tail; `Debug` of the source excludes the key; remove asks first | **radio** (`:367-369`), **id addressing** (`:381`) |
| 11 | `usage_overlay_is_read_only_and_hands_off_to_accounts` | 400 | `Usage · read-only`; `m` hands off; Esc→Manager | none |
| 12 | `prelude_creates_a_pending_workspace_and_opens_the_editor` | 416 | 5-step chain; Esc rewinds **with state**; pending fields | none |
| 13 | `prelude_refuses_a_duplicate_name_and_cancels_cleanly` | 463 | duplicate refused; full rewind ⇒ `Cancelled · nothing created` | none |
| 14 | `editor_edits_count_once_preview_then_saves_and_returns` | 497 | `• 1 change` counted once; leaving asks; preview lists `1 modified`; async save persists | none |
| 15 | `editor_env_plain_value_stays_masked` | 538 | plain env stays masked; `m` keeps it masked; new secret staged as `************1234` | **id addressing** (`:567-571`) |
| 16 | `settings_trust_toggle_and_failed_save_keep_edits` | 581 | a failed save keeps `• 1 change`; retry persists | none |
| 17 | `hard_cases_refresh_keeps_last_good_and_help_opens_everywhere` | 618 | per-route help sections; `broker unreachable` | **help sections are derived now** (`:630, :640`) |
| 18 | `complete_jackin_flow_keyboard_first` | 646 | the 40-step product journey | **radio** (`:668, 690-692, 727-732`), **id addressing** (`:648-649, 674, 708, 738, 805`) |
| 19 | `editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on` | 1028 | per-workspace enable/disable/prefer with effective-set resolution | none |
| 20 | `manager_launch_picker_hides_agents_without_an_account` | 1110 | unusable agents **omitted**, not disabled | none |
| 21 | `environments_stay_readable_with_a_hundred_roles` | 1137 | >100 roles; only configured sections render; searchable role picker adds one | **id addressing** (`:1182-1186`) |
| 22 | `cockpit_resolves_every_effective_account_for_the_container` | 1201 | the container receives the whole effective account set | none |

### 6.2 `app_tests_chrome.rs` — **6** tests **[F]** (the brief says 5; the file has six)

| Test | Line | Proves | Rewrite risk |
|---|---|---|---|
| `capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line` | 22 | row 0 = menu bar, not the identity line; row 2 = tab strip; row 38 = status bar with `PR #482` + `%` + `━`; last row has `Ctrl+B` | **row indices are absolute** — any chrome height change breaks it |
| `menu_bar_opens_switches_and_runs_an_action` | 43 | F10 opens; ←→ switches; Esc closes; mouse `View → Usage` | **View menu item order** |
| `tab_context_menu_renames_and_closes_by_mouse_and_keyboard` | 74 | right-click and `Ctrl+B m` both open; rename applies; Close asks | **tab-menu item order** (`End` reaches the last row, `:98`) |
| `hint_bar_stays_on_the_last_row_across_layers` | 110 | layer precedence; **the picker draws no hint row of its own** (`t.matches("Enter Choose").count() == 1`, `:124`); prefix row 0 shows `prefix…` | **highest risk under derived hints** — a component-declared "Enter Choose" binding rendered inside the picker would make the count 2 |
| `inspect_changes_opens_from_the_view_menu_in_both_modes` | 133 | diff opens; `m` compact/advanced; `d` unified/review; Esc×2 returns | **View menu item order** (`Right,Right,End`) |
| `command_palette_scrolls_with_the_wheel_and_keeps_the_selection` | 161 | wheel scrolls rows; **wheel-up restores the frame exactly** (`assert_eq!(h.text(), before)`); selection stays on item 0 | none, but wheel routing must be byte-exact |

### 6.3 Tests needing rewritten keystroke sequences — the exhaustive list

**(a) `RadioGroup` cursor ≠ value (§20.10-3).** **[F]** Today `RadioGroup::on_key` selects while the arrows move (`choice.rs:121-130`), and `FormDialog` emits `FormEvent::Changed` on that move (`modals.rs:1148-1157`), which drives `Screen::form_changed` to reveal dependent fields. Tests that assert a *reveal* after bare arrows and therefore **must gain a `Space`/`Enter`**:
- `app_tests.rs:367-369` — `Down, Down` then `assert!(contains("API key"))`.
- `app_tests.rs:666-669` — `Down` then `assert!(contains("Local agent folder"))`.
- `app_tests.rs:690-693` — `for _ in 0..provider_steps { Down }` then Tab into the 1Password chooser.
- `app_tests.rs:727-733` — `for _ in 0..3 { Down }` … `Down, Down`.
- `app_tests.rs:320-321` — `Down` then `assert!(contains("Codex · OpenAI"))`.
Sequences that press `Enter` immediately after the arrows (`:234-236`, `:255-257`, `:973-976`, `:1002-1005`) survive unchanged, because Enter commits the cursor.

**(b) Test-visible id addressing (§16.4 mapping 3).** `FORM.sub("save")` → `FORM.part(Part::custom("save"))` with `Harness::area_of`; a component's own sub-region uses `area_of_part`. Sites: `app_tests.rs:309, 319, 322, 340, 381, 648, 649, 567-571, 1182-1186`, and `crate::screens::manager::TREE` at `:171, :771`. All app `const Id`s become `pub` in the `jackin_app` **library** target (`COMPONENT_ARCHITECTURE.md:2007`).

**(c) `Harness::tab_to` bound.** **[F]** `app_tests.rs:75-86` loops at most 24 Tabs before panicking. §20.10-15 says `Field` chrome, `NavList`, `scroll_region` parts and disabled-but-registered entries change ring size and order. Either the bound rises or the ring shrinks; either way an entry in `docs/visual-changes.md` (item 15) recording old/new counts is required **before** any expected value is edited.

**(d) Dialog `y`/`n` (§20.10-5).** **[F]** Jackin's footer advertises `hint("y / n", "Quick answer")` (`app.rs:2552`). No Jackin test presses `y` on a dialog (`app_tests.rs:926`'s `y` is the viewport copy), so no rewrite — but Jackin must **opt in** to the `y`/`n` binding set through its `KeyMap`, or the hint silently disappears from every dialog capture.

**(e) Menu item order.** `chrome.rs:135-139` and `:98-99` navigate by `Right`/`End` — the View and tab menus must keep their item counts and order, or the sequences reach a different action.

### 6.4 `visual_tests.rs` — 36 digests **[F]**

`visual_tests.rs:74-251` `SURFACES`, digest = FNV-1a over `(symbol, fg, bg, modifier)` of **every** cell, no rect excluded (`:16-34`), compared against `tests/baselines/jackin.txt` (37 lines incl. trailing newline; 36 entries). Coverage: 8 scenario start routes, `intro-phrase`, `manager-expanded-detail`, `prelude-step-1`, 5 editor tabs, `settings`, 2 accounts surfaces, `usage`, 2 cockpit states, 6 capsule surfaces, `outro-caption`, `too-small`, and 6 responsive (manager/capsule × 80×24, 100×30, 160×50). The test asserts **each surface builds identically twice** before hashing (`:258-263`).

**[I] Obligations:**
- The file moves to `apps/jackin-preview/tests/visual.rs`, the baseline to `apps/jackin-preview/tests/baselines/jackin.txt` (`COMPONENT_ARCHITECTURE.md:1908, 4192-4193`).
- Every one of the 36 digests will change. Expected causes, each a §20.10 item that must appear in `docs/visual-changes.md` before `xtask bless-guard` will accept the bless: 2 (layer compositing), 5 (dialog hint row), 6 (menu shortcut column), 7b/7e/7f (geometry fixes), 8 (backdrop footer), 9 (StatusBar merge), 10 (derived hints), 11 (surface ladder — note the `border_subtle == text_ghost` collision of §4.2), 13 (tab strip window), 15 (ring composition), 16 (display width).
- Order is fixed: **change → capture → classify → bless** (`COMPONENT_ARCHITECTURE.md:1910`).
- The matrix should gain `× {junie, paper} × {truecolor, mono}` only if the coordinator extends §20.10-12's showcase rule to Jackin; **[I]** recommend yes for `manager`, `capsule`, `capsule-choice-dialog` and `outro-caption`, since those are the surfaces §4.4 puts at risk.

### 6.5 Secret-masking assertions that must not weaken — exhaustive **[F]**

| Assertion | Line |
|---|---|
| `!h.text().contains("valid-ant01")` — "secret leaked into the frame" | `app_tests.rs:304-307` |
| `!h.text().contains("throttled-thr01")` | `:344` |
| `!h.text().contains("sk-ant-valid")` **while typing** | `:373-376` |
| `!t.contains("sk-ant-valid")` after Tab **and** `t.contains("1234")` (tail only) | `:378-380` |
| `!h.text().contains("abcdef")` after save | `:384` |
| `matches!(a.source, PlainApiKey { tail, .. } if tail == "1234")` | `:386-388` |
| `!format!("{:?}", a.source).contains("abcdef")` — "fingerprint must not embed the key" | `:389-392` |
| `!t.contains("pw-fixture-only")` — plain env value | `:546` |
| `!h.text().contains("pw-fixture-only")` after re-masking | `:551` |
| `!h.text().contains("abcdefghijklmnop")` while typing | `:566` |
| `t.contains("************1234")` **and** `!t.contains("abcdefghijklmnop")` | `:575-576` |
| `!h.text().contains("valid-")` — "secret leaked" in the 40-step journey | `:711` |
| `!h.text().contains("abcdefghijklmn")` | `:737` |

**[I] Added obligations, none of which may replace the above:**
- `Secret` is `!Clone`, `!PartialEq`, `!Serialize`; `Debug`/`Display` redact (§15). `FormValues` (`modals.rs:794`) is **deleted**, not redacted — `format!("{:?}")` of the form is no longer the only defence.
- The enclosing owner calls `FormState::zeroize()` when cancelling or dismissing; `Form` does not own layer lifecycle events (§15.1).
- New: `jackin::form_dialog_secret_never_reaches_the_screen_as_a_string`, `conformance::form::secret_never_appears_in_debug`.
- **`"************1234"` is a *stored value* rendering, not a field mask.** It comes from `domain::workspace::mask` (`config.rs:37`), applied to the persisted `EnvValue`. Keep that domain function; use `Secret`/`SecretPolicy` only for the in-flight `TextInput` draft. If `SecretPolicy`'s default `GlyphRole::SecretMask` (Junie `•`) were applied to the stored value, `app_tests.rs:575` would fail on `*` vs `•`.

---

## 7. The known panic — `screens/capsule.rs:1183`

### 7.1 [F] Exact defect

```rust
// screens/capsule.rs:1181-1185
Prop::new(
    "Container ID",
    format!("3f9c{}e21a", &i.run_id.replace('-', "")[..8]),
)
.copyable(),
```

Producers of `Instance.run_id` (`domain/instance.rs:109`, a bare `String`):
- **`domain/fixtures.rs:469`** — `run_id: format!("run-{}", &id[3..])` with `id = "jk-7f3a"` ⇒ `"run-7f3a"`. `.replace('-', "")` ⇒ `"run7f3a"` = **7 bytes**. `[..8]` panics: *byte index 8 is out of bounds of `run7f3a`, which contains 7 bytes*. **Every fixture instance has this shape.**
- **`screens/cockpit.rs:84-90`** — `format!("run-{stamp12}-{suffix}")` ⇒ 21 chars; safe by accident.

Reachability: `open_container_info` is called from `run_menu("Container info")` (`capsule.rs:450`), i.e. **F10 → View → Container info** on every Capsule surface, and from the command palette. `handle_prefix_cmd` has no `'i'` arm (`capsule.rs:1266-1321`), so the menu's advertised `Ctrl+B i` (`capsule.rs:258`) is separately dead (§1.2 M5). **No test opens it** — `visual_tests.rs:184-189` opens the menu but selects nothing; `chrome.rs:65-70` selects *Usage*. The panic is live and untested.

### 7.2 The structural fix

The enabling condition is *not* a missing bounds check. It is that a **free-form `String` field has two producers that agree on nothing, and a display site invents a derived token by byte-slicing it**. Remove that condition:

1. **Introduce a `RunId` newtype in `domain/instance.rs`** with a single constructor that guarantees the shape, and only **total** accessors:

```rust
/// A launch run identifier. Constructed one way; every derived form is total.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RunId(String);

impl RunId {
    /// The only constructor. `stamp` and `suffix` are normalised to
    /// `[0-9a-z]`; the stored form is always `run-{stamp}-{suffix}`.
    pub fn new(stamp: &str, suffix: &str) -> Self { /* … */ }

    pub fn as_str(&self) -> &str { &self.0 }

    /// A fixed-width, ASCII, char-boundary-safe short token.
    /// A pure function of the whole id (FNV-1a, hex): total for every input,
    /// including a one-character id. Never a slice.
    pub fn token(&self) -> RunToken;   // pub struct RunToken([u8; 8]);
}
impl fmt::Display for RunId { /* as_str */ }
```

2. **Move the display formatting into the domain**, so the render site does no arithmetic at all:

```rust
impl Instance {
    /// The container's public short id, as the Capsule "Debug info" shows it.
    pub fn container_uid(&self) -> String {
        format!("3f9c{}e21a", self.run_id.token())
    }
}
```
`capsule.rs:1181-1185` becomes `Prop::new("Container ID", i.container_uid()).copyable()`.

3. **Route both producers through `RunId::new`**: `fixtures.rs:469` and `cockpit.rs:84-90`. The 12-character-stamp assumption at `cockpit.rs:88` (`[..12]` on a `replace`d timestamp) is the *same* defect one line over and is removed by the same change.

4. **Structural enforcement, not review.** The new workspace already denies the construct: `clippy::indexing_slicing = "deny"` and `clippy::panic = "deny"` (`COMPONENT_ARCHITECTURE.md:4240, 4239`). With `RunId` in place there is no way to obtain a short token except through a total function, so the lint has nothing to fight.

5. **Catching tests**: `instance::container_uid_is_total_for_every_fixture_run_id` (iterate `Scenario::ALL` × `world.instances`, plus one `CockpitScreen`-generated id, plus adversarial `RunId::new("", "")`); and an integration test `jackin::container_info_opens_in_every_scenario` that drives F10 → View → Container info on `capsule-multi` and `outro-last`.

6. **Deferred root cause, named**: `capsule.rs:1191` `format!("{}-4f11", i.run_id)` and `capsule.rs:1194` embed the raw id; those are safe formats and stay, but they now read `i.run_id` through `Display`, so the newtype does not leak.

---

## 8. Ordering, ownership, risks, open questions

### 8.1 Ordering inside Slice 7 (single owner, serial)

| Step | Work | Gate before continuing |
|---|---|---|
| 0 | Move the crate to `apps/jackin-preview/` with `[lib] jackin_app` + thin `[[bin]] jackin-preview` (`COMPONENT_ARCHITECTURE.md:4190-4193`, §21 item 23). Move `app_tests.rs`/`app_tests_chrome.rs`/`visual_tests.rs`/`perf_tests.rs` to `tests/`, `tests/baselines/jackin.txt` with them. Rewrite `use junie_tui::…` once. **No behaviour change.** | all 22 + 6 + 1 visual + 3 rain + 4 arbiter + 2 clock + 1 scenario tests green, byte-identical digests |
| 1 | **`domain`/`sim` first**: `RunId` newtype + `Instance::container_uid` (§7). Pure domain, no library dependency. | new `container_uid` tests green; digests unchanged |
| 2 | **Shell skeleton**: `impl App for App`, `Runtime<App>`, `Jx`, the six-method `Screen` trait *with the old bodies wrapped*. Delete `Focus`/`FocusRing`/`HitRegistry` fields (`app.rs:143-145`), `hover`/`pressed`/`hover_suppressed`/`flash`/`last_click` (`:146-149, 157`), the press/release/double-click machine (`app.rs:1534-1753`), focus reconciliation (`app.rs:2266-2274`). | `no_diagnostics_are_emitted_during_the_journey` on `complete_jackin_flow_keyboard_first` |
| 3 | **Layers**: `Modal`/`ModalEntry`/`ModalTag`/`ModalResult`/`push_modal`/`pop_modal`/`deliver`/`modal_key`/`modal_click`/`modal_outside_click` all deleted (`app.rs:83-88, 142, 1229-1281, 1333-1530, 1755-1947`). Every overlay becomes `cx.open_layer` + `ui.layer`. | `chrome.rs:110` (hint layering), `layer::nested_layers_each_trap` |
| 4 | **`modals.rs` decomposition** (§3), largest single commit. `file_browser.rs`, `op_flow.rs` created; `modals.rs` deleted. | tests 9, 10, 12, 13, 15, 18, 21; visual `accounts-*`, `prelude-step-1` |
| 5 | **Per-screen migration** in dependency order: `config.rs` (shared by editor+settings) → `editor.rs` → `settings.rs` → `accounts.rs` → `usage.rs` → `manager.rs` → `prelude.rs` → `inspect.rs` → `cockpit.rs` → `capsule.rs` (last, largest, and the perf target) | per-screen: its named tests + its visual digests |
| 6 | **`rain.rs` onto `Role` + `Ui::dim_layer`** (§4). Last, because `dim_layer` needs every screen painting through `Ui` | `rain.rs` unit tests; handoff digests under both themes |
| 7 | **Menu chords + derived hints** (M1–M3, H3). Deliberately after the screens, so component bindings exist to derive from | `chrome.rs:43, 74, 110, 133`; `bindings_match_handled_keys` |
| 8 | **Classify → capture → bless** the 36 digests; write `docs/visual-changes.md` entries | `xtask bless-guard` |
| 9 | Full Slice-7 gate (`COMPONENT_ARCHITECTURE.md:4080`) | see below |

**Gate (verbatim from `:4080`, expanded):**
```bash
cargo fmt --all --check
cargo clippy -p junie-tui -p jackin-preview --all-targets --all-features -- -D warnings
cargo test  -p junie-tui -p jackin-preview --all-targets --all-features
cargo test  -p jackin-preview --doc
RUSTDOCFLAGS="-D warnings" cargo doc -p jackin-preview --no-deps
cargo run -p xtask -- doc-check
cargo test  -p jackin-preview --test perf --release -- --test-threads=1
# thresholds: frame_jackin_capsule_4panes_120x40 < 200 allocs
#             frame_jackin_manager_100rows_120x40 < 60 allocs
#             key_jackin_manager_move == 0 allocs
#             capsule_pane_clone_4x2000 absent from perf_baseline.txt
for s in first-use returning accounts-mixed launch-running launch-failure \
         capsule-multi outro-last hard-cases; do
  cargo run -p jackin-preview -- --scenario "$s" --motion paused --frame 0
done
tools/capture.sh   # host, settings, account/usage, launch, Capsule, menu,
                   # modal, tab, status-bar, responsive; every diff classified
```

### 8.2 File ownership inside the slice

Single owner for the whole tree (`:4077`). Contended shared files that Slice 7 **must not** write: `crates/tui/**` (frozen), `crates/tui/tests/conformance.rs`'s suite list, `docs/visual-changes.md` (append-only, coordinator resolves ordering). Slice 6 owns `apps/tablepro/**` concurrently; the two trees are disjoint (`:4104`).

### 8.3 Risks, mitigations, catching tests

| # | Risk | Mitigation | Catching test |
|---|---|---|---|
| R1 | **The virtual clock is re-based on the runtime's repaint cadence**, invalidating every tick count and fixture timestamp (§5.2) | `Route::tick_ms` stays a Jackin constant and is the *only* argument to `world.tick`; the repaint deadline is derived from it, never the reverse | `app_tests.rs:139-155`; `visual_tests.rs` `cockpit-failure` (`FAILURE_TICKS = 77`), `cockpit-running` (frame 20), `outro-caption` (frame 150) |
| R2 | **Derived hints emit a second `Enter Choose`** inside the picker layer | the picker's own bindings are `visible: false` where the shell's hint bar already shows them; one hint surface only | `chrome.rs:124` (`count() == 1`) |
| R3 | **Capsule chrome height changes** and the absolute row assertions break | keep row 0 = menu bar, row 2 = tab strip, row 38 = status bar, last row = hint bar at 120×40 | `chrome.rs:25, 31, 33, 39` |
| R4 | **`RadioGroup` cursor≠value silently changes product meaning** in the accounts credential-source field (the reveal of `API key` / `Local agent folder`) | insert an explicit commit in the five sequences of §6.3(a) and confirm the *product intent* is unchanged, not just the assertion | tests 10, 18 |
| R5 | **`Ui::dim_layer` sees no roles** because a screen still paints through `ui.raw()` | `architecture::no_generic_component_copies_in_applications` allow-list is exactly `rain.rs`; rain itself paints via `ui.paint_cell` | `ui::dim_layer_uses_the_role_of_the_painted_cell`; handoff digests |
| R6 | **Capsule perf target unreachable** — the baseline is 1 080 602 allocs/frame, 5.4× the doc's stated "≈480 000" | the fix is structural (delete `pane.term.clone()` at `capsule.rs:1567` and `prime`), not incremental; if `< 200` is unreachable after that, pause for adjudication rather than relax the threshold | `frame_jackin_capsule_4panes_120x40` |
| R7 | **`tab_to` exceeds 24 Tabs** after ring composition changes | measure `ring().reachable()` per screen before and after; record counts under §20.10-15 | tests 9, 10, 15, 18, 21 |
| R8 | **`View` menu reordering** silently breaks `Right,Right,End` navigation | freeze the item lists at `capsule.rs:252-262` and `:311-317` during the slice; change them only in a separate reviewed commit | `chrome.rs:133, 74` |
| R9 | **Secret regression via a convenience API** re-introduced in an app-side `FormData` | `FieldMut::Secret(&mut Secret)` is the only channel; `Secret: !Clone` makes a copy a compile error | the 13 assertions of §6.4 + `form::form_action_variants_carry_no_value` |
| R10 | **`world.clipboard` writes move into a component** when `InfoDialog` becomes `Dialog::facts` | `PropsAction::Copy(ItemKey)`; the screen resolves the value and writes the clipboard | `app_tests.rs:909-932` (three clipboard assertions) |
| R11 | **`intro_guard` lost**, so a stale keypress skips the ritual | express it as a route-scoped Capture binding; `runtime::drain_pending_input` alone is insufficient | `app_tests.rs:118-126` |
| R12 | **`dim_buffer`'s existing collisions** (`border_subtle == text_ghost`, `disabled == text_faint`) change handoff output when semantics replace equality | classify as an intentional §20.10-7 sub-item (a defect fix), with before/after captures of `HandoffStage::CockpitDim(1..4)` | new handoff digest |

### 8.4 Open questions requiring fresh adjudication

| # | Question | Why it blocks | Recommended resolution **[I]** |
|---|---|---|---|
| **Q1** | **`Screen::strip_right`** (`screens/mod.rs:310`) is listed as removed in §3.4 with no named replacement. It returns `Vec<Segment>` with priorities and tones (dirty count, "refreshing", daemon-stale) — `HintLayer.status: Option<Cow<str>>` cannot carry it. | Six screens use it; the identity strip and Capsule status bar depend on it | Route it through `Jx` (`jx.strip_item(StatusItem)`) from `update`, with the shell caching the last set so a pure repaint is not stale. Keeps the trait at six methods and keeps the product half in `Jx`, per §3.4. The alternative — widening `Screen::hints`'s return type — changes an accepted signature |
| **Q2** | **`App::update` cannot tell a tick pass from an input pass.** §3.3 step 1 names "app.tick", but the accepted `App` trait (`§17.0 A1`, `:2159-2170`) has `update`, `draw`, `should_quit`, `keymap`, `min_size`, `on_esc` — no tick hook and no `Cx` accessor for elapsed virtual time. | Jackin's entire clock, the rain state machines, the launch stage machine, `world.jobs` and the status timeout are tick-driven | Add `FrameRead::tick_elapsed(&self) -> Option<Duration>` (a read, not a new phase) or `App::tick(&mut self, cx, dt) -> Response<()>`. Either is a public-API change and needs adjudication before step 2 |
| **Q3** | **Cross-route message fan-out.** `App::dispatch_msg` (`app.rs:446-465`) offers an unconsumed `Msg` to `manager` and `accounts` even when they are not the active route. §3.4 says messages are "drained by the app before this runs" but does not say how a non-active screen receives them. | `app_tests.rs:347-355` (a refresh started on Accounts completes after navigating away), `:601-614` | Inherent `apply_msg` on the two screens, called explicitly from `App::update`. Not a `Screen` trait method — record it so it is not "corrected" into one |
| **Q4** | **`Ui::dim_layer`'s treatment of non-ladder roles** (`Success`, `Warning`, `Danger`, `Info`) is undefined in §11.6. `rain.rs:133-139` has a rule (`ladder[4 - steps - 1]`, erase at `steps >= 4`). | The handoff cross-fade over the Capsule status bar, which is full of tone-carrying cells | Adopt `rain.rs:133-139`'s rule as `dim_layer`'s documented semantics and put it in the library, not in Jackin |
| **Q5** | **`Ui::scroll_region`'s exact signature is still open** (`COMPONENT_ARCHITECTURE.md:1172`, §25.7). | Every Jackin screen has ≥1 scroll region; nine `scrollbar::id_for` routing sites depend on it | Must be closed before step 5. It is a Slice-3 item, flagged as blocking 4E; confirm it landed before Slice 7 begins |
| **Q6** | **`app_tests_chrome.rs` has six tests, not five.** The plan brief and `COMPONENT_ARCHITECTURE.md:1930` ("22 jackin (17 + 5 chrome)") both undercount; the real inventory is **22 + 6 = 28** plus 10 in-module unit tests (rain 3, arbiter 4, clock 2, scenario 1) = 38. | `architecture::every_named_test_exists` compares the documented inventory against the compiled list | Correct §16.4's count to `22 + 6` and re-issue; a stale count fails the gate |

---

### Appendix — deletion inventory (Jackin-side, for the "no legacy path remains" check)

`app.rs`: `Focus`/`FocusRing`/`HitRegistry` fields (`143-145`), `hover`/`pressed`/`hover_suppressed`/`flash`/`last_click` (`146-149, 157`), `ModalEntry` (`83-88`), `modals: Vec<ModalEntry>` (`142`), `interaction()` (`322-336`), `animating()` (`299-313`), `tick_interval()` (`315-320`), `push_modal`/`pop_modal`/`deliver`/`refresh_picker`/`form_changed` (`1229-1331`), `modal_key` (`1333-1530`), `on_mouse` (`1534-1753`), `modal_outside_click` (`1755-1800`), `modal_click` (`1802-1947`), `apply_requests` (`1951-1979`), `draw_strip` (`2433-2516`), `draw_footer` (`2518-2642`), `modal_hints` (`2373-2377`), `top_tag`/`top_owner`/`form_values` (`2645-2661`), `run_host_menu` key synthesis (`754-813`), the 200-line help section table (`892-1206`), the too-small block (`2283-2310`).
`screens/mod.rs`: `CustomModal` (`66-95`), `Modal` (`98-108`), `ModalResult` (`111-131`), `ModalTag` (`41-63`), `Request` (`180-191`), `Cx` (`193-227`), 17 of 23 `Screen` methods.
`screens/modals.rs`: the whole file (2426 lines).
`rain.rs`: `dim_buffer` (`102-172`), `ladder_color` (`59-67`), the palette-field bodies of `style` (`70-86`).
`capsule.rs`: `prime` (`1408-1416`), the clone/inert/cursor-replace block (`1567-1591`), the tab-strip rebuild (`1459-1462`), `meter_tone` (`2568-2577`), the prefix-command hint list (`2478-2492`), `run_menu` label dispatch (`368-472`).
Per screen: every `seam_container: Rect`, every `*_area: Rect` cache, every raw `ScrollState`, every `scrollbar::id_for` comparison, every `focus.focus(...)` inside a click handler (the 14 sites named in `docs/audit/domain-boundary-audit.md:420`).
