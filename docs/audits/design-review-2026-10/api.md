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
