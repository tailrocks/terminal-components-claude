Audit of commit `b9ff31567`. That commit landed while I was auditing, so some app-side line numbers may have shifted. Nothing in the repo was edited.

**Group A: modal and popup infrastructure.** This logic is duplicated across apps, and it is the clearest breach of the AGENTS.md rule.

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 1 | P0 | src/bin/jackin_preview/app.rs:1772, src/bin/holla/app.rs:1405, src/bin/tablepro/app.rs:1877, src/bin/showcase/app.rs:657 | app modal stacks (`enum Modal` in jackin screens/mod.rs:98, holla screens/mod.rs:56, tablepro app.rs:112) | Each app re-implements the modal stack, `modal_outside_click`, the `DialogResult` to action mapping, and "Picker/Menu outside click = Cancelled". The app decides dismissal, which AGENTS.md forbids. | `ui::modal::{Modal trait {on_key,on_click,on_click_outside,on_paste,render,result()->Option<R>}, ModalStack<R>}` with `push`, `top_mut`, `route_key/click/outside_click`, `render(screen,buf,ctx)` (calls `modal_backdrop` per layer). `Dialog`, `Picker` and `ContextMenu` implement it. |
| 2 | P0 | src/widgets/picker.rs:272; src/widgets/menu.rs:218, :236; src/widgets/completion.rs:174 | Picker, ContextMenu, Completion | No `Dismiss` field: Esc and outside click always close. Completion uses `surface()` without an owner, so it registers no backdrop and the app has to handle clicks elsewhere. | `pub dismiss: Dismiss` plus `fn dismiss(self, Dismiss)` on all three, default `Dismiss::LIGHT`. Use `surface_for(.., Some((id, dismiss)))` and handle `backdrop_id(id)` in `on_click`, as `Select` does. |
| 3 | P0 | src/bin/jackin_preview/screens/modals.rs:36; src/bin/holla/screens/modals.rs:194; src/widgets/dialog.rs:392-409; src/widgets/picker.rs:387-400 | modal frame | The backdrop, centered rounded elevated frame, title/meta row, width clamp (`min(w-4).max(20)`) and inner margin (3,2)/(3,1) are copied 4 times. | `ui::modal::Frame { width: u16, height: u16, title, meta: Option<String>, placement: popup::Placement, margin: Margin }` and `Frame::render(screen,buf,ctx)->Rect`, used by Dialog and Picker, then exported. |
| 4 | P1 | src/bin/holla/app.rs:1520, src/bin/tablepro/app.rs:1986, src/bin/jackin_preview/app.rs:1642 | Interaction bookkeeping | Every app hand-rolls `hover`, `pressed`, `hover_suppressed` and `flash` updates in `on_mouse`. | `core::pointer::PointerState { hover, pressed, hover_suppressed, flash }` with `on_mouse(&HitRegistry, Mouse) -> PointerOutcome` and `interaction(focus, tick) -> Interaction`. |
| 5 | P1 | src/bin/jackin_preview/screens/modals.rs:2290 (`HelpOverlay`), :555 `ChoiceDialog`, :902 `FormDialog`, :1956 `InfoDialog`; src/bin/tablepro/app.rs:99 `FilterEditor` | app-local widgets | These are generic components living in one app, so the other apps cannot reuse them. | Move them to `widgets::{help::HelpOverlay, dialog::Dialog::choice/::form/::info}` with the same `Dismiss` policy. |
| 6 | P1 | src/ui/popup.rs:80 vs src/widgets/menu.rs:57; picker.rs:399; completion.rs:172 | Placement | There are two `Placement` enums: popup has `Below`/`Center`, menu has `Below`/`Above`/`Right` with its own flip code. Picker is fixed to `Center` and Completion to `Below`. | One `popup::Placement { Below, Above, Right, Center }` handled by `popup::place`, which flips for all variants. Delete `menu::Placement`. Add `Picker::placement` (default `Center`) and `Completion::placement` (default `Below`). |
| 7 | P2 | src/ui/popup.rs:107 | place(Center) | The centered popup's vertical position is fixed at the upper third (`/3`, `max(1)`). | `Placement::Center` stays as today. Add `Placement::CenterAt { y_ratio: u8 }`, or `CenterExact` for the jackin `center:bool` case. |
| 8 | P2 | src/ui/ctx.rs:186 | `RenderCtx::modal_backdrop` | It always leaves the bottom row undimmed (footer). | `modal_backdrop_with(buf, screen, Backdrop { keep_bottom: u16 })`. Keep `modal_backdrop` as the `keep_bottom: 1` shorthand. |

**Group B: render order dependence.** These widgets need a render between events, or need the app to draw them last.

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 9 | P0 | src/widgets/dialog.rs:492, :231 | Dialog (facts + ack) | The confirm button is armed only inside `render`, and `finish()` never checks `armed()`. A key or click between edit and render can confirm while the typed token no longer matches. It also assumes the confirm button is the *last* action. | Check `armed()` in `finish` for the confirm index. Add `pub confirm_index: usize`, set by the `facts` constructor; default stays the last action, to keep today's behaviour. |
| 10 | P1 | src/widgets/select.rs:282 | Select | `render` closes the list when the field loses focus, so `on_key` sees stale `open` until the next frame. | `Select::on_focus_lost()`, or `close()` called from the focus layer. Add `pub close_on_blur: bool`, default `true`. |
| 11 | P1 | src/widgets/menu.rs:594 | MenuBar | The hover-switch is applied inside `render` (`self.on_hover`), so state changes on draw. | Keep `on_hover` and route it from the pointer layer (#4). Remove the render-time call once routing exists. |
| 12 | P1 | src/widgets/completion.rs:174; src/widgets/menu.rs:240 (`ContextMenu::render`) | Completion, ContextMenu | Both paint in place instead of via `ctx.overlay`, so they must be drawn last (Select and MenuBar do not have this problem). | Queue both through `ctx.overlay`, as `Select::queue_list` does. |
| 13 | P2 | src/widgets/picker.rs:387 | Picker | `render(.., hints: &str)` is a per-frame argument rather than state, which is inconsistent with other widgets. | `pub hints: Vec<Hint>` and `fn hints(self, ..)`. Drop the render parameter. |

**Group C: key bindings.** Add one `core::keymap` type.

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 14 | P0 | list.rs:131, tree.rs:372, table.rs:405, select.rs:140, menu.rs:195/480, choice.rs:150, steps.rs:156, textarea.rs:110, panel.rs:279, viewport.rs:1368, code.rs:415, grid.rs:995, dialog.rs:312 | all navigable widgets | vim aliases `j/k/h/l/g/G` are baked in. They are inconsistent too: List and Grid also accept `J/K`, Tree does not; some widgets guard on `key.plain()`, others do not. | `core::keymap::NavKeys { vim: bool, page: bool, home_end: bool }` with `NavKeys::DEFAULT` = today's behaviour (vim on). Add `pub nav: NavKeys` and `fn nav(self, NavKeys)` on every navigable widget, plus one shared `NavKeys::action(&Key)->Option<Nav>` that replaces the per-widget matches. |
| 15 | P1 | src/widgets/dialog.rs:331 | Dialog | `y`/`n` accelerators are fixed, and apply only to Text bodies. | `pub accelerators: Option<(char, char)>` and `fn accelerators(self, Option<(char,char)>)`, default `Some(('y','n'))`. |
| 16 | P1 | src/widgets/tabs.rs:195, :208, :180/213 | Tabs | Digits 1-9 jump to a tab; `x`/Delete close; `n`/Enter add a new tab. | `pub digit_jump: bool` (default `true`), `pub close_keys: &'static [KeyCode]` (default `[Char('x'), Delete]`), `pub new_key: Option<char>` (default `Some('n')`). |
| 17 | P2 | src/widgets/chips.rs:114/119/120, tree.rs:435/439, list.rs:159, table.rs:445, viewport.rs:1379/1386, panel.rs:285, grid.rs:986-1157 | various | Fixed action letters: chips `x`/`X`/`+`; tree `*`/`-`; list `a`; table `s`; viewport `f`/`y`; ScrollPanel `f`; Grid ~15 letters. | Per-widget action enum plus a `KeyMap<A> = Vec<(KeyBinding, A)>` field, `fn keymap(self, KeyMap<A>)`, defaulting to `A::default_keymap()` (today's bindings). Grid needs this most. |
| 18 | P2 | src/widgets/select.rs:186 | Select (closed) | Up/Left/Down/Right change the value without opening the list. macOS opens the popup instead. | `pub arrows: SelectArrows { Step, Open }`, default `Step`. |

**Group D: navigation, selection and layout modes.**

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 19 | P1 | menu.rs:181 (wraps) vs select.rs:145, tabs.rs:192, list/tree (clamp) | cursor wrap | ContextMenu and MenuBar wrap; Select, Tabs, List and Tree clamp. Neither is configurable. | `pub wrap: bool` and `fn wrap(self, bool)` on each list-like widget, defaulting to each widget's current behaviour. |
| 20 | P1 | src/widgets/tree.rs:421 | TreeView | Enter or Space on a parent always toggles; a parent can never be activated. | `pub branch_activate: BranchAction { Toggle, Activate, Both }`, default `Toggle`. |
| 21 | P2 | src/widgets/tree.rs:546/575 | TreeView | Indentation is 2 columns per level. | `pub indent: u16` and `fn indent(self, u16)`, default `2`. |
| 22 | P2 | src/widgets/menu.rs:612, :137 | MenuBar, ContextMenu | Labels that overflow the bar are silently dropped (`Rect::ZERO`). Menu min width (8+6) is fixed. Long menus do not scroll. | `MenuBar::overflow: Overflow { Hide, Chevron }` (default `Hide`). `ContextMenu::max_rows: u16` (default `u16::MAX`, i.e. no limit) and `ContextMenu::min_width: u16` (default `8`). |
| 23 | P2 | src/widgets/dialog.rs:77/95/113/143, :206/458 | Dialog | Widths 54/66 have no builder. The code preview is capped at 6 lines. | `fn width(self, u16)` (defaults unchanged) and `pub code_max_rows: u16`, default `6`. |
| 24 | P2 | src/widgets/picker.rs:134-135; src/widgets/completion.rs:169 | Picker, Completion | Picker width 64 and max_rows 12 are pub fields with no builders. Completion width is clamped to (24,48). | `Picker::width(u16)`, `Picker::max_rows(u16)`, `Completion::max_rows(u16)`, and `Completion::popup_width(min,max)` (default `(24,48)`), matching `Select::popup_width`. |

**Group E: glyphs.** Add `theme::Glyphs` (`Theme.glyphs`, with `Glyphs::UNICODE` as today's default and `Glyphs::ASCII` for Mono).

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 25 | P1 | theme.rs:391 has `gutter_symbol`, yet `"▎"` is re-hardcoded at menu.rs:634, picker.rs:447, panel.rs:117; `"›"` at select.rs:389, list.rs:292, table.rs:731, code.rs:754; `"▴"/"▾"` at select.rs:309/406/414; `"▾"/"▸"` at tree.rs:583 | many | Focus bar, current-row marker, disclosure and scroll arrows cannot be themed, and are duplicated. | `Glyphs { focus_bar, current, chevron_open, chevron_closed, disclosure_open, disclosure_closed, more_above, more_below, ellipsis, separator }`. Route `gutter_symbol` through `focus_bar`. |
| 26 | P2 | choice.rs:95/227/322, chips.rs:217, tabs.rs:433/330/465, scrollbar.rs:8, progress.rs:8, steps.rs:253, button.rs:151 | Checkbox, Radio, Toggle, Chips, Tabs, Scrollbar, Spinner, Steps | `[✓]`/`[ ]`, `●`/`○`, `──●`, `×`, `‹`/`›`, `│`/`┃`, braille spinner, `✓` are all consts. | Extend `Glyphs` with `check_on`, `check_off`, `radio_on`, `radio_off`, `toggle_on`, `toggle_off`, `close`, `overflow_left`, `overflow_right`, `scroll_track`, `scroll_thumb`, `spinner: &'static [&'static str]`, `done`. |

**Group F: English copy in library code.** Add `theme::Strings`, or a `Labels` struct per widget.

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 27 | P1 | dialog.rs:68/86/104/130, :133, :462 | Dialog | `"Cancel"`, `"Type {tok} to confirm"` and `"… {n} more"` are fixed. | `Strings { cancel, type_to_confirm: fn(&str)->String, more_lines: fn(usize)->String, required, no_matches, search_placeholder, empty_list, no_rows, loading }` on `Theme`, used by constructors. Also add `Dialog::cancel_label(&str)`. |
| 28 | P1 | grid.rs:404-406, :283-317, :1695, :1738; input.rs:181; picker.rs:129/137; list.rs:75; table.rs:155; diff.rs:271/464 | Grid, TextInput, Picker, ListBox, DataTable, Diff | `"Preview SQL"`, `"Discard"`, `"Save"`, validator messages, `"Loading rows…"`, `"↓ {} loaded · Enter fetches more"`, `"Required"`, `"Type to search…"`, `"No matches"`, `"Nothing here yet"`, `"No rows"`, `"Old"`/`"New"`/`"No file selected"`. Several also lack builders: `Picker` has none for placeholder or empty_text. | Defaults come from `Strings`. Add `Picker::placeholder()`, `Picker::empty_text()`, `DataGrid::bar_labels([&str;3])` and `TextInput::required_message(&str)`. |

**Group G: API consistency.**

| # | prio | file:line | widget | hardcoded behaviour | proposed config API |
|---|---|---|---|---|---|
| 29 | P1 | input.rs:37, table.rs:115, grid.rs:395 | TextInput, DataTable, DataGrid | Validators are `fn` pointers, so they cannot capture state such as an enum list or a schema. | `Option<Rc<dyn Fn(&str)->Option<String>>>`, keeping `fn validator(self, impl Fn + 'static)`. |
| 30 | P2 | menu.rs:22, completion.rs:22, tree.rs:19 | MenuItem, CompletionItem, TreeNode | `shortcut` and `glyph` are `&'static str`, so they cannot be built at runtime. Builder naming is also inconsistent: `Dialog::with_actions`; `MenuItem::danger()`/`separator()` take no argument while `disabled(bool)` does; `empty_text: String` (List, Table, Picker) vs `empty: EmptyState` (Grid). Tabs, TreeView and ListBox have no struct-level builders at all (only pub fields: `allow_new`, `quiet`, `mode`). | Use `Cow<'static, str>`. Rename to `Dialog::actions(vec, cancel)`, keeping `with_actions` as deprecated. Add `danger(bool)`, `separator(bool)`, `empty(EmptyState)` everywhere, `Tabs::allow_new(bool)`/`quiet(bool)` and `ListBox::mode(SelectMode)`. |

Suggested order: Group A (#1 to #3) first, then #9, then the `NavKeys` (#14), `Glyphs` (#25/#26) and `Strings` (#27/#28) theme types, which close about half of the remaining findings at once.
## Status (interaction and API pass)

Each finding is fixed (with its commit) or deferred with a reason.
"Partial" means part of the proposed API landed and the rest is deferred.

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 1 | Partial | 7bf62651d, f5f60a12a, 224173846 | `ui::modal::{Modal, ModalStack}`: push/pop with focus restore, `outside_click()`/`escape()` from the top modal's policy; all three multi-modal apps use it. Deferred: `route_key/click/paste/render` on the stack (the apps' modals take app-specific contexts: World, Cx, requests). |
| 2 | Fixed | 7bcc23dd2, 7c803a133, 7bf62651d | ContextMenu and Completion landed earlier; `Picker::dismiss` here. |
| 3 | Partial | 7bf62651d, 224173846 | `ui::modal::Frame` (backdrop, body clamp, placement, title/meta, margin); jackin's `modal_frame` is now a call to it. Deferred: Dialog, Picker and holla's text viewer still draw their own frame (moving them changes the visual baseline). |
| 4 | Fixed | 4efbcf819 | `core::pointer::PointerState`. |
| 5 | Partial | 755237d5d | `widgets::help::HelpOverlay`. Deferred: ChoiceDialog, FormDialog, InfoDialog and tablepro's FilterEditor depend on app domain types (World, FsEntry, OpReference, ColType). |
| 6 | Deferred | | One `popup::Placement` for menus: the menu flip logic and the layout pass's `popup::Place` margins are being reworked by the layout agent. |
| 7 | Deferred | | Same reason as #6. |
| 8 | Deferred | | `modal_backdrop_with`: no consumer needs another footer height yet. |
| 9 | Fixed | 5bfea9062 | Landed before this pass. |
| 10 | Deferred | | `Select::close_on_blur` with a focus-layer hook: same hook as interaction #15. |
| 11 | Deferred | | MenuBar hover from the pointer layer: `live_hover` already makes it keyboard-safe; moving the call out of render is open. |
| 12 | Fixed | 7c803a133, 7bcc23dd2 | Landed before this pass. |
| 13 | Deferred | | `Picker::hints` as state: changes every caller's render call. |
| 14 | Deferred | | `NavKeys` on every navigable widget: 13 widgets; the per-widget knobs (`type_ahead`, `skip_disabled`, `digit_jump`) cover the conflicts found so far. |
| 15 | Fixed | 71477f758 | `Dialog::accelerators`. |
| 16 | Fixed | c4f8a7b5b | `Tabs::{digit_jump, close_keys, new_key, allow_new}`. |
| 17 | Deferred | | Generic `KeyMap<A>` per widget (Grid's ~15 letters first). |
| 18 | Fixed | 326998660 | `Select::arrows(SelectArrows)`, default Step. |
| 19 | Deferred | | `wrap` on list-like widgets. |
| 20 | Fixed | 40b1d089a | `TreeView::branch_activate(BranchAction)`. |
| 21 | Fixed | 40b1d089a | `TreeView::indent`. |
| 22 | Partial | 71477f758 | `ContextMenu::min_width`. Deferred: `MenuBar::overflow`, `ContextMenu::max_rows` (needs menu scrolling). |
| 23 | Fixed | 71477f758, d2d6896b9 | `Dialog::width`; code previews scroll (layout pass) instead of a row cap. |
| 24 | Fixed | 71477f758 | `Picker::{width, max_rows}`, `Completion::{max_rows, popup_width}`. |
| 25 | Deferred | | `theme::Glyphs` touches every widget's drawing and the visual baseline; the state-model pass owns theme.rs now. |
| 26 | Deferred | | Same as #25. |
| 27 | Partial | 934985f8d | `Dialog::cancel_label` landed earlier. Deferred: `theme::Strings`. |
| 28 | Partial | 71477f758 | `Picker::{placeholder, empty_text}` builders. Deferred: `theme::Strings`, `DataGrid::bar_labels`, `TextInput::required_message`. |
| 29 | Partial | 71477f758 | TextInput validators are closures (`TextValidator`). Deferred: DataTable and DataGrid validators. |
| 30 | Partial | cec8f1474, c4f8a7b5b | `ListBox::mode`, `Tabs::allow_new`. Deferred: `Cow` labels, `Dialog::actions` rename, `danger(bool)`/`separator(bool)`, `empty(EmptyState)`. |

## Status update (fields and focus pass)

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 10 | Fixed | e184db2b0, e5917f9e1 | `Select::on_blur()` with `close_on_blur: bool` (default `true`), called from the focus-change hook every app forwards (interaction #15); `render` no longer closes the list, so `on_key` never sees a stale `open`. |

## Status update (popup placement and API pass)

Glyphs (#25, #26), strings (#27, #28) and keymaps (#14, #17) are scheduled
in a separate pass and are not covered here.

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 1 | Partial | | Unchanged from the first pass. Deferred: `route_key/click/paste/render` on `ModalStack`. Each app's modal enum dispatches to modals whose handlers take app contexts (jackin's `World`/`Cx`/`SimOnePassword`, tablepro's requests), so a shared router needs a `ModalLayer<C>` trait with an associated context and a rewrite of every app's modal dispatch. Those dispatch blocks are also being edited by the focus and context-menu passes. |
| 3 | Partial | 77ac73279, f03e91bb4, 60e507a07 | holla's text viewer now draws in `ui::modal::Frame` (a test pins the same rectangle). Dialog and Picker share the placement (`popup::Placement`, `place_with`) and the backdrop (`Backdrop`) with Frame. Deferred: they keep their own chrome. Both draw the title inside the frame where Frame puts it on the border, Dialog centres with ratatui's rounding (the odd cell goes after), and Picker clamps to `screen - 4/2`. Converging any of these would change the visual baseline. |
| 5 | Partial | 46d2ba57d | `widgets::choice_dialog::ChoiceDialog`, `widgets::form_dialog::{FormDialog, FormField, FormFieldKind, FieldValue, FormEvent, FormValues}` and `widgets::info_dialog::{InfoDialog, InfoResult}` moved from jackin unchanged: none of them held an app type. jackin re-exports them under the old paths. Deferred: tablepro's `FilterEditor`. Its shape is generic (column select, operator select, 0 to 2 values), but the operator list and its arity come from `FilterOp::ordered_for(ColType)`. Moving it needs a schema trait (operators per column kind, arity per operator) plus the ~300 lines of key, click and draw functions in tablepro/app.rs, which the context-menu and focus passes are editing now. |
| 6 | Fixed | 8c78c62a7, f03e91bb4 | One `popup::Placement { Below, Above, Right, Left, Center }`. `place_with` flips every anchored variant before it overflows. `Place` gains `vertical_margin: Option<u16>` (default `None`, which follows `margin`) and `overflow: Overflow { Shrink, Slide }` (default `Shrink`). `Place::MENU` is the old menu fit. `menu::Placement` is gone; the path is now a re-export of `popup::Placement`. `ContextMenu::fit` (default `Place::MENU`) replaces the `edge_margin` field; the `edge_margin(u16)` builder stays. `Picker::{placement, anchor, fit}` default to `Center`. `Completion::{placement, fit}` default to `Below`. `Dialog::{placement, anchor}` default to `Center`. `modal::Frame::{placement, anchor, fit}` default to `Center`/`Place::DEFAULT`. `FramePlacement` is deprecated (both variants always produced the same centred rectangle) and converts with `Into`. Fixes: a `Right` submenu at the screen edge used to cover its anchor, and an `Above` menu near the top used to do the same. |
| 7 | Fixed | 8c78c62a7 | `Placement::Center` is unchanged. `Place::v_align(VAlign::{Center, Third})` (from the layout pass) covers the upper-third and exact-centre cases, and modal frames take it through `Frame::fit`. |
| 8 | Fixed | 60e507a07 | `RenderCtx::modal_backdrop_with(buf, screen, Backdrop { keep_bottom, keep_top })`. `modal_backdrop` is the `Backdrop::DEFAULT` (`keep_bottom: 1`) shorthand, and `Backdrop::FULL` dims everything. `Dialog::backdrop`, `Picker::backdrop` and `Frame::dim` take it; all default to `Backdrop::DEFAULT`. |
| 11 | Deferred | | MenuBar hover from the pointer layer. Each app has to call `MenuBar::on_hover(pointer.hover)` from its mouse-move handler before the render-time call can go. Those handlers are being edited by the focus and context-menu passes. `live_hover` already keeps the render-time switch keyboard-safe. |
| 13 | Fixed | f03e91bb4 | `Picker::hints: String` plus the `hints(&str)` builder. `Picker::draw(screen, buf, ctx)` renders with it. `Picker::render(.., hints)` is deprecated and every caller has moved. |
| 19 | Fixed | b3d00bdb7 | `core::nav::step_index(cursor, len, delta, wrap)` is the single stepping rule. `ContextMenu::wrap` and `MenuBar::wrap` default to `true`. `ListBox::wrap`, `TreeView::wrap`, `Tabs::wrap` and `Select::wrap` (open list and closed arrows) default to `false`. Each default is the widget's existing behaviour. |
| 22 | Partial | a80b5e66a | `ContextMenu::max_rows: u16` (default `u16::MAX`). A menu taller than its room now scrolls instead of cutting off its last items: keys scroll the cursor into view, `ContextMenu::on_wheel` scrolls without moving it, and ▴/▾ on the frame mark more rows. `ContextMenu::offset` is the first row shown. `ContextMenu::min_width` landed in the first pass. Deferred: `MenuBar::overflow: Overflow { Hide, Chevron }`. A chevron needs a menu of the hidden titles plus its glyph, which belongs to the glyph pass. |
| 27 | Partial | | Unchanged. Only `theme::Strings` remains, and the strings pass owns it. |
| 28 | Partial | | Unchanged. `theme::Strings`, `DataGrid::bar_labels` and `TextInput::required_message` belong to the strings pass. |
| 29 | Fixed | 70e9a16e7 | `DataTable::validator(impl Fn(usize, &str) -> Option<String>)` stores a `table::CellValidator`. `DataGrid::validator(impl Fn(&ColumnSpec, &str) -> Result<CellValue, String>)` is a new builder. `grid::Validator` is an `Rc`'d closure (`Validator::new`, `Default` = `default_validator`). Breaking: code that assigned a `fn` to `DataGrid::validator` now wraps it in `Validator::new`. |
| 30 | Fixed | 4a07c4c9e | `MenuItem::shortcut`, `CompletionItem::glyph` and `TreeNode::glyph` are `Cow<'static, str>`, and the builders take `impl Into<Cow>`. `Dialog::actions(vec, cancel)` replaces the deprecated `with_actions` alias. `MenuItem::danger_if(bool)` and `separator_if(bool)` sit beside the no-argument `danger()`/`separator()`, which stay because 34 call sites use them. `ListBox`, `DataTable` and `Picker` gain `empty(EmptyState)` (an `Option` that overrides `empty_text`), and `DataGrid` gains the `empty(EmptyState)` builder. Breaking: `CompletionItem` struct literals now write `glyph: "T".into()`. |

## Status update (keymap pass)

Supersedes the Deferred rows for #14 and #17 above.

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 14 | Fixed | c1e48420c, 6b16b6258, 627a7500d, 02421eb86, e9bc6e0a3 | `core::keymap::{Nav, NavKeys}`. `NavKeys { arrows, vim, page, home_end }` with `NavKeys::DEFAULT` (all on: today's keys) and `NavKeys::NO_VIM`. One `NavKeys::action(&Key) -> Option<Nav>` table: arrows, PageUp/PageDown and Home/End return whatever modifiers are held (the widget still decides Shift+↓ extends, Ctrl+Home goes to the first row), while `h j k l`, `H J K L` and `g`/`G` never match with Ctrl or Alt held. Every navigable widget has `pub nav: NavKeys` and `.nav(NavKeys)`: ListBox, TreeView, DataTable, DataGrid, Select (open list; the closed field steps with the arrows only), ContextMenu, MenuBar (its open menu inherits the bar's keys), RadioGroup, StepRail, TextArea (reading), ScrollPanel, TextViewport, CodeEditor (reading), Dialog (body scroll and moves between actions), Tabs, ChipBar, Picker (a searchable picker keeps its letters, ←/→ and Home/End for the query), Completion (default `Completion::NAV`: arrows and paging only, since letters and Home/End belong to the editor), PropsList and HelpOverlay. Inconsistencies fixed: TreeView takes `J`/`K` like ListBox and DataGrid; every widget refuses Ctrl/Alt letters (ListBox and DataGrid used to move on Ctrl+K); an open Select takes `g`/`G` and the paging keys; ContextMenu pages by its visible rows; RadioGroup, ChipBar, Tabs and HelpOverlay gain Home/End and `g`/`G`; StepRail gains paging; Dialog's `j`/`k`/`g`/`G` scroll a tall body like the arrows. Deferred: ChoiceDialog, InfoDialog and FormDialog keep their own `h`/`l` and `j`/`k` matches, because the dialogs pass moved them into the library in this window and is still editing them. |
| 17 | Fixed | c1e48420c, 6b16b6258, 627a7500d, 02421eb86, 242201c5d | `core::keymap::{KeyBinding, KeyMap<A> = Vec<(KeyBinding, A)>, Actions}` plus `lookup`, `binding_for`, `label_for` and `hint_key`. A `KeyBinding::char` ignores Shift, because Shift is part of the character, and every other modifier must match exactly. `A::default_keymap()` reproduces the old keys, and each widget has `pub keymap` and `.keymap(KeyMap<A>)`. The action enums: `ListAction::ToggleAll` (`a`); `TreeAction::{ExpandAll, CollapseAll}` (`*`, `-`); `TableAction::SortByColumn` (`s`); `GridAction` with 17 actions (Del/Backspace, `-`, `+`, Ctrl+D/Alt+D, `y`, `Y`, Ctrl+S, `s`, `S`, `f`, `/`, `F`, `r`/F5, `u`, `U`, `p`, Ctrl+]); `PanelAction::Follow` (`f`); `ViewportAction::{ToggleFollow, Copy}` (`f`, `y`); `CodeAction` (Enter/`i`, `a`, `{`, `}`, `/`, `n`, `N`); `TabAction::{Close, New, Jump(i)}` (`x`/Delete, `n`, `1`-`9`); `ChipAction::{Remove, Add, ClearAll}` (Del/Backspace/`x`, `+`, `X`); `DialogAction::{Confirm, Cancel}` (`y`, `n`). Folded in: `Tabs::{digit_jump, close_keys, new_key}`, `Dialog::accelerators` and `DataGrid::duplicate_keys` are no longer fields. Their builders stay as shortcuts that rewrite the matching bindings. Hints: `hint_key(map, action)` interns the bound chord's label as a `&'static str` (`Hint::key` is static), or returns `None` when the action is unbound. tablepro's grid, chip bar, result-tab and tab-strip footers use it, so a remapped key is advertised and an unbound action loses its hint. Default labels equal the old literals, so the baseline is unchanged. Breaking: the removed fields above, and `Tabs::close_keys` now takes `&[KeyCode]`. Grid action letters no longer fire with Ctrl or Alt (Ctrl+R used to refresh, Alt+S used to sort), and named-key bindings need their exact modifiers (Shift+Delete no longer closes a tab or removes a chip; Shift+Enter no longer starts a CodeEditor edit). Deferred: Enter, Space, Esc and Tab stay fixed in every widget, because they are the activation and focus vocabulary. Picker's and Completion's Ctrl+N/P (and Picker's Ctrl+J/K) stay fixed as text-field keys. PropsList `y` and HelpOverlay `?`/`q` are not in a KeyMap yet. holla and jackin footers only name app-level keys, so they have no widget keymap to read. Dialog's `←→ Choose` and `↑↓ Scroll` hints stay literal. |

## Status update (context menus and text fields, round 2)

New public API from interaction #13 and #16 round 2; no finding above
changes status.

| Area | Commit(s) | API |
|---|---|---|
| Context menus | 08a846237, 788ebad9f | `keyhint::CONTEXT_MENU: Hint` (`Shift+F10 Menu`). `TextViewport::context_point() -> Position`: where a keyboard menu about a viewport opens (selection end, caret, followed tail, top line). |
| Pointer | 8c9a6c523, e1540a2a4 | `PointerState::press_count(id, pos, now_ms) -> u8`, `PointerState::surface_press(hits, id, focus) -> bool`; `HitRegistry::register_surface(id, area)` and `is_surface(id)`; `HitRegion::surface`. |
| Text fields | 8c9a6c523, 0a3fbc311 | `PointerSelect::drag_by_word: bool` (default `true`) with `drag`/`word`/`line`/`drag_by_word` builders; `TextInput::on_press_count`, `TextArea::on_press_count`; `field_common::{PointerField, route_press, route_drag, route_click}`; `TextBuffer::selected_text` is public. |
| Dialogs | 76ee283ba | `Dialog::{on_press, on_click_count}` (and `on_drag` reaches the input); `FormDialog::{on_press, on_drag, on_click_count}`. |
| Tables | 84cb0f669 | `DataTable::TOGGLE_CTRL_CMD` (the default `toggle_mods`) and `DataTable::TOGGLE_WITH_ALT`. |
| Platform defaults | (this change) | `core::event::ClientPlatform { MacOs, Other }`: `ClientPlatform::from_env(get)` (override `JUNIE_TUI_CLIENT_PLATFORM`, then `LC_TERMINAL=iTerm2`, then a macOS-only `TERM_PROGRAM`, else `ClientPlatform::build_target()`) and `ClientPlatform::current()` (that on the process environment, read once); `InputConfig::for_platform(ClientPlatform) -> InputConfig` (`Default` is `for_platform(ClientPlatform::current())`); `DataTable::TOGGLE_OPTION_CMD` and `DataTable::toggle_mods_for(ClientPlatform) -> KeyModifiers`, now the default `toggle_mods` (Option or Cmd on macOS, Ctrl or Cmd elsewhere); `keyhint::modifier_click(KeyModifiers, ClientPlatform) -> Option<&'static str>` (`Option+click` / `Ctrl+click`). |

## Status update (glyphs, strings and keymap leftovers)

Supersedes the Deferred and Partial rows for #22, #25, #26, #27 and #28
above, and the deferred notes of #14 and #17 in the keymap pass.

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 22 | Fixed | 3d5415e56 | `menu::Overflow { Hide, Chevron }` on `MenuBar::overflow` (builder `overflow`, default `Hide`: today's output). With `Chevron`, the titles that do not fit, and every title after the first hidden one, collapse into one chevron title drawn with `Glyphs::more_titles` (`»`). Its menu lists the hidden titles; choosing one opens that title's menu under the chevron (`MenuBarEvent::Opened(title)`). The chevron is clickable, hover-switches like a title and sits on the ←/→ ring; its own menu reports `Opened(labels.len())`. New: `MenuBar::{more_id, more_area, hidden}`. |
| 25 | Fixed | 18f15fd5f, b85109a1e | `theme::Glyphs` on `Theme::glyphs` (`Theme::with_glyphs`). `Glyphs::UNICODE` (default) is today's set; `Glyphs::ASCII` draws 7-bit only with the same cell widths. Fields: `focus_bar`, `current`, `chevron_open/closed`, `disclosure_open/closed`, `more_above/below`, `sort_asc/desc`, `ellipsis`, `separator`, `more_titles`, `arrows`, `shift`, `border` (the popup, dialog, menu and framed-panel `border::Set`). `Theme::gutter_symbol` and `Theme::interactive` draw `focus_bar`; the modal backdrop strips the theme's bar (an ASCII `|` only when it carries the focus ink). Hint keys go through `Glyphs::key_label` (arrows, `⇧`, the range dash) when the hint bar draws. Per-level hook: `LevelOverrides::ASCII_MONO` (hook `theme::ascii`) switches Mono to `Glyphs::ASCII` and `Strings::ASCII`; `theme::ascii_glyphs` switches the glyphs only. Every glyph a widget draws in src/widgets and src/ui now reads the theme; key-name constants (`keyhint::UP_DOWN`, reveal keys) stay Unicode and are mapped when the hint bar draws them. |
| 26 | Fixed | 18f15fd5f | Same type: `check_on/off/mixed` (`[✓] [ ] [–]`), `tick`, `check_off_narrow`, `dash`, `radio_on/off`, `dot_on/off`, `toggle_on/off`, `close`, `overflow_left/right`, `scroll_track/thumb`, `scroll_h_track/h_thumb`, `scroll_held` (splitters use the scrollbar glyphs), `spinner` (`Glyphs::spinner_frame`), `done`, `arrow` (cut cell, rename), `bar_fill/empty` (progress, meters, tab rules, menu separators, diff rules), `dirty`, `removed`, `warning`, `paused`, `none`, `mask`, `bullet`, `filter`, `key`, `newline`, `tab`, `control`. Pure functions gained `_with` variants that take the glyphs (`grid::cell_text_with`, `diff::{unified_lines_with, review_lines_with}`, `DiffFile::{summary_with, header_with}`, `TriState::glyph_in`); the old names keep the Unicode set. `scrollbar::{TRACK, THUMB, ...}`, `progress::SPINNER`, `button::MORE_LABEL` and `PickerItem::DEFAULT_GLYPH` stay as the Unicode defaults. Deferred: `ui::text::{truncate, truncate_middle, fit, slice_cells, wrap}` still cut with `…`. They are pure functions with about 300 call sites (most in the apps) and no theme; threading the glyph through needs a `_with` variant per helper and a pass over every caller. A widget's own cut marks (`…` drawn at an edge) already use `Glyphs::ellipsis`. |
| 27 | Fixed | acfbe62db, b85109a1e | `theme::Strings` on `Theme::strings` (`Theme::with_strings`). `Strings::DEFAULT` is today's copy, `Strings::ASCII` the same words with 7-bit punctuation. Templates fill `{}` in order with `Strings::fill`. A label still reading the strings it was built from follows a theme change (`Strings::follow`), so a theme set after construction applies; a builder sets text that the theme no longer changes. Dialog: `cancel`, `type_to_confirm`, `type_to_confirm_hint`, `more_cut`, `more_count` and the hints `hint_choose`, `hint_scroll`, `hint_scroll_code`, `hint_cancel`, `hint_close`, `hint_next`, `hint_apply`, `hint_edit` (read from the strings of the last render). FormDialog `cancel`/`save`, ChoiceDialog `cancel`/`next`, InfoDialog `close`. `Dialog::cancel_label` still overrides. |
| 28 | Fixed | acfbe62db | `required` (+ `TextInput::required_message(&str)`, an `Option` that wins over the theme), `optional` (the field suffix), `no_matches`, `search_placeholder` (`Picker::{placeholder, empty_text}`), `empty_list` (`ListBox::empty_text`), `no_rows` (`DataTable::empty_text`, the grid's `EmptyState`), `loading_rows`, `fetching`, `fetch_more`, the grid bar's `preview_sql`/`discard`/`save_changes` (+ `DataGrid::bar_labels([&str; 3])`), diff `diff_old`/`diff_new`/`diff_no_file`/`diff_no_changes`/`diff_review`, reveal `reveal_open`/`reveal_close`, HelpOverlay `help_title`/`help_footer`, and `not_in_preview`, the convention for a menu item's `reason` (holla and jackin still spell it; they are being edited by the app passes). Deferred: the grid and scrollbar position labels (`rows 1–20 of 1,203`, `{n} update(s)`, `12–24 of 120`), CodeEditor's `ln/col` and the Meter's `refreshing`. They pluralise and order numbers, which a `{}` template cannot express; they need a formatter hook (`fn(&Counts) -> String`) on `Strings`. |
| 14 | Fixed | 0b2379818 | Leftovers: `ChoiceDialog::nav` (default `NavKeys::DEFAULT`: ←/→ and `h`/`l` between buttons, as before), `FormDialog::nav` (default `NavKeys::NO_VIM`: ←/→ only, as before, so letters stay with the fields) and `InfoDialog::nav` (detail scroll with ↑/↓ and `j`/`k`) replace their own matches. Type-ahead, off by default like ListBox and Select: `ContextMenu::type_ahead`, `MenuBar::type_ahead` (hands it to its menus; letters then never switch titles), `DataTable::type_ahead` and `DataGrid::type_ahead` with `type_ahead_column: Option<usize>` (default the cursor's column). TreeView's type-ahead landed in 8da9efa7f. |
| 17 | Fixed | 0b2379818 | Leftovers: `PropsAction::Copy` (`y`) on `PropsList::keymap`, `HelpAction::Close` (`?`, `q`) on `HelpOverlay::keymap`, `InfoAction::{Close, Copy}` (`q`, `y`) on `InfoDialog::keymap`. Esc and Enter stay the fixed vocabulary. Behaviour change: these letters no longer fire with Ctrl or Alt held. |

Proof: `tests/ascii_glyphs.rs` renders every widget family (choice
controls, buttons, chips, tabs, scrollbars, splitters, progress, steps,
lists, trees, menus, selects, tables, grids, inputs, panels, dialogs,
diffs, hint bars) with `Glyphs::ASCII` + `Strings::ASCII` and asserts no
non-ASCII cell, after checking the same scene draws Unicode by default.
`tests/strings.rs` overrides the strings per widget and checks that
builders win and default labels follow a theme change back.
`tests/keymap_leftovers.rs` covers the new NavKeys, KeyMaps and
type-ahead. `menu.rs` tests cover the chevron. `tests/showcase_baseline.txt` is
unchanged. The showcase/pages and showcase/flows/selectable captures
passed after the glyph commit, except showcase/pages/chrome, which
still showed the pre-Shift+F10 hint until 8d1046880 re-approved it. After the later
commits they differ only where the showcase's tree sidebar (e48c7f75c)
changed the frame (breadcrumb and sidebar), pending that pass's
re-approval.

Breaking changes: `Theme` gains `glyphs` and `strings` (struct literals
of `Theme` must add them). `FormDialog`, `ChoiceDialog`, `InfoDialog`
and `HelpOverlay` gain private fields, so they can no longer be built
as struct literals outside the crate. `TriState::glyph`,
`grid::cell_text`, `diff::{unified_lines, review_lines}` and
`DiffFile::{summary, header}` keep the Unicode set; themed callers use
the `_with` variants.
