# Shared components plan

Status: plan, 2026-10-16, for `design/interaction-polish`. Owner: main session; every step is done by subagents (AGENTS.md, "Way of working").

Inputs: [`duplication-shell.md`](../audits/design-review-2026-10/duplication-shell.md), [`duplication-widgets.md`](../audits/design-review-2026-10/duplication-widgets.md) and [`duplication-screens.md`](../audits/design-review-2026-10/duplication-screens.md); [`mouse.md`](../audits/design-review-2026-10/mouse.md) and [`keyboard.md`](../audits/design-review-2026-10/keyboard.md); [`standards-interaction.md`](../audits/design-review-2026-10/standards-interaction.md) and [`standards-visual.md`](../audits/design-review-2026-10/standards-visual.md); the status sections of the other audits; and [`docs/visual-testing/plan.md`](../visual-testing/plan.md), which requires the port to `main` to match this branch 1:1. Finding references: **M#** is a mouse audit finding, **K#** a keyboard audit finding, **SI#** a standards-interaction verdict, **SV#** a standards-visual verdict, and **SIg#** / **SVg#** a "gaps not yet raised" item in those two reviews.

**The problem.** The four apps hold about 11k lines of shell code and about 11.5k lines of hand-built screen UI. Inside `src/widgets` there are about 145 duplicated functions (about 1,750 lines). The copies have drifted: the same gesture behaves differently in different apps and widgets. This plan moves that behaviour into one shared layer, decides each divergence once, and names everything so that porting to `main` (termrock) moves code instead of rewriting it.

## 1. Principles and target architecture

### 1.1 Principles (from AGENTS.md)

| # | Principle | What it means for this plan |
|---|---|---|
| P1 | Behaviour comes only from shared library code | Every row in the three duplication reviews is a defect. Each step deletes the copy **in all apps in the same step**. |
| P2 | Interfaces are shared and composable | Every widget implements the same core traits. Containers and the shell compose them generically: no `owns`, `locate` or per-widget routing in apps. |
| P3 | Apps are thin consumers | An app provides routes, pages, domain state, commands and configuration. Routing, focus, layers, hints, help, quit and the clock belong to `shell`. |
| P4 | Standards-based defaults with layered overrides | Defaults follow macOS HIG, then WAI-ARIA APG, then WCAG 2.2 AA. Overrides are layered: app profile (`Desktop`, `Vim`), then a per-widget field and builder, then the user's key map (§3). |
| P5 | Floating content goes through the overlay layer | Every popup or modal is a `Layer` (§1.7). Draw order inside a page never matters. |
| P6 | Proof | Each behaviour change ships a regression test, driven through the real widget or app, that fails before the fix. The visual gate stays green, or a visual change is named and re-approved. |

### 1.2 Layers

| Layer | Module (this branch) | Contents | Crate on `main` |
|---|---|---|---|
| Core | `src/core` | `Intent`, `Phase`, `Response<A>`, `Part`/`PartRef`, `StateFlags`, keys and key maps, `Cursor`, `RowSelection`, `ScrollPane`, `Clock` | `termrock-core`, `termrock-collections` |
| UI | `src/ui` | `RenderCtx`, hits, focus ring, `Layer`/`LayerSpec`, `RowUi`, reveal, text helpers, leave guard | `termrock-runtime` (`ui`, `layer`, `hit`, `focus`) |
| Blocks | `src/ui`, `src/widgets/blocks` | `ListCore`, `EditField`, `OptionList`, `ContentState` | `termrock-collections`, `termrock-fields` |
| Widgets | `src/widgets` | Every widget implements `Component` plus its capabilities | The per-family crates |
| Shell | `src/shell` (new) | `Shell`, `App`, `Page`, `Cx`, `Request`, routers, `Footer`, `StatusLine`, `Router`, `Chrome` | `termrock-shell` (new crate; §1.9) |
| Apps | `src/bin/*` | Routes, pages, domain state, commands, configuration | `apps/*` |

### 1.3 Event model

| Item | Shape | Notes |
|---|---|---|
| `Intent<'a>` | `Key(Key)` · `Binding(ActionKey)` · `Paste(&str)` · `Pointer { phase: Phase, part: PartRef, pos, count: u8, mods }` · `Wheel { axis: Axis, delta: i16, part, pos, mods }` · `FocusIn(FocusVia)` · `FocusOut(Option<WidgetId>)` · `Layer(LayerEvent)` · `Cancel` | Built once by the runtime dispatcher from `Input`, `PointerState` and `HitRegistry::resolve`. Widgets never see raw ids or `MouseKind`. |
| `Phase` | `Move, Press, Release, Click, Secondary, SecondaryRelease, Middle, DragStart, Drag, DragEnd` | `count` carries 1, 2 or 3 clicks (500 ms, 1-cell slop). There is no separate `DoubleClick` phase. |
| `Part` / `PartRef` | `Part(u16)` constants (`ROW`, `CELL`, `HEADER`, `ROW_NUMBER`, `TRACK`, `THUMB`, `CLOSE`, `TAB`, `QUERY`, `SEAM`, `BODY`, `CONTAINER`, ...) and `PartRef { part, item: Option<ItemKey> }` | `HitRegistry::register_part(owner, PartRef, area)` and `resolve(pos) -> Option<Hit { owner, part }>`. This replaces `owns` (18 copies), `locate` (15) and `row_id` (8). |
| `Response<A>` | `{ flow: Flow, invalidate: Invalidate, action: Option<A> }`, with `ignored()`, `consumed()`, `changed()` and `emit(a)` | `#[must_use]`. `Outcome` stays as `Response::outcome()` until step S15 removes it. This replaces the six return shapes (`Outcome`, `(Outcome, bool)`, `bool`, `Option<E>`, ...). |
| `StateFlags` | Bitflags: `FOCUSED, HOVERED, PRESSED, SELECTED, ACTIVE, CHECKED, DISABLED, READ_ONLY, ERROR, EDITING, DIRTY, EXPANDED, LOADING` | The single row and control state input to `Theme` and `RowUi`. |
| `Cx<'a>` | One event context for widgets and pages: `focus`, `ring`, `pointer`, `clock`, `context: ContextConfig`, `requests`, `focus_next()`, `capture(owner, part)` | The page-level helpers `status`, `error`, `open`, `go` and `copy` push `Request<R>`. |

### 1.4 Core traits

| Trait | Methods (defaults in brackets) | Implemented by |
|---|---|---|
| `Component` | `type Action`; `id()`; `update(&mut self, &Intent, &mut Cx) -> Response<Self::Action>` | Every widget |
| `Draw` | `draw(&mut self, area, &mut RenderCtx)`. The surface colour comes from `RenderCtx::surface()`, never from a parameter | Every widget. This replaces the five render signatures. |
| `Measure` | `measure(&self, avail: Size, &RenderCtx) -> Size` | Every widget that can size itself |
| `Scrollable` | `pane(&mut self) -> &mut ScrollPane`; [`after_scroll()`] | 17 scrolling widgets |
| `Navigable` | `cursor(&mut self) -> &mut Cursor`; `rows(&self) -> &dyn RowSource` | List, Tree, Table, Grid, Props, Steps, OptionList, Tabs, Chips, Radio |
| `Selectable` | `selection(&mut self) -> &mut RowSelection`; [`click_policy() -> ClickPolicy`] | List, Tree, Table, Grid, OptionEditor |
| `Editable` | `field(&mut self) -> &mut EditField`; [`wants_text_input()`] | TextInput, TextArea, CodeEditor, the cell editors, the Picker and Combo queries |
| `HasHints` | [`hints(&self, &HintCx) -> Vec<Hint>`, derived from `Bindings` and `NavKeys`] | Every focusable widget and every `Layer` |
| `Bindings` | `keymap()`, `actions() -> &[(Cmd, &str)]` | Every widget with a `KeyMap<XCmd>` |
| `ContextSource` (exists) | `context_items()`, [`context_select(target)` through `Cursor` and the selection rule] | Every row widget and every text widget |
| `Overlay` | `dismiss() -> Dismiss`; `on_dismiss(DismissReason) -> Response<A>`; `spec() -> LayerSpec` | Select, Picker, Completion, ContextMenu, MenuBar menus, Dialog, FormDialog, ChoiceDialog, InfoDialog, HelpOverlay, FileBrowser |
| `Focusable` | [`focusability() -> Focusability { Stop, PassThrough, Skip }`]; [`skip_when_empty`] | Containers, empty widgets, the busy Button |
| `MouseTarget` | Object-safe `pointer(&mut self, &Intent, &mut Cx) -> Response<()>` | Blanket impl for every `Component`. Hand-written adapters exist **only** for widgets not yet migrated, and are removed in S15. |

### 1.5 The shared input chain (one order, run by the dispatcher)

`chord guard (unassigned Ctrl/Alt)` → `Overlay` (Esc, outside click per `Dismiss`) → `Scrollable` (wheel, `Part::TRACK`/`THUMB`, scroll chaining to the nearest owner that can still move) → `Editable` → `Navigable` (`NavKeys`, type-ahead, `ensure_visible`) → `Selectable` (`SelectGesture` from mods or keys, `ClickPolicy`) → `ContextSource` (secondary click, Shift+F10, Menu key) → `Component::update` (domain only: sort, expand, close tab).

Pointer intents that a widget returns `Ignored` for bubble up the parent chain recorded by `RenderCtx::scope(id, |cx| ..)`. Hints are the focused leaf's hints, then each ancestor's, de-duplicated by key.

### 1.6 Shared building blocks

| Block | Shape | Replaces |
|---|---|---|
| `ScrollPane` | `{ v, h: ScrollState, h_cfg: HScroll, track_click: TrackClick, follow: Option<Follow>, held: Option<Axis> }`; `on_wheel(axis, delta)`, `on_bar_press`, `on_bar_drag`, `draw_bars(..) -> content Rect` | `on_wheel` ×15, `on_wheel_h` ×7, `on_scrollbar*` ×13, `on_bar_drag` ×5, `track()` ×9, two follow modes, the faded-edge plus scrollbar copies in screens (×26) |
| `Cursor` + `RowSource` | `Cursor { index, wrap }`, `apply(Nav, &dyn RowSource, page) -> Moved`, `type_ahead(c, &mut TypeAhead, src)`; `RowSource { len, enabled(i), label(i), key(i) -> ItemKey }` | 22 `nav.action` match blocks, 4 `step()` functions, 7 type-ahead copies, 10 chord guards, 69 screen-level list-key arms |
| `RowSelection` + `SelectGesture` + `ClickPolicy` | `SelectGesture { Only, Toggle, Extend { from }, All }`, `from_key` and `from_mods`; `RowSelection::apply(g, to, order, enabled)`; `ClickPolicy { activate: Activate::{Single, Double, ClickAgain}, edit_on, check: CheckClick }` | List `checked`, Grid `selected_rows`, OptionEditor `checked`, four click-to-activate rules |
| `RowUi` (`RowPainter` in the review) | `RowUi::paint(ctx, buf, rect, flags: StateFlags, look: RowLook::{Bar, Fill(CursorStyle)}) -> RowPaint { style, content }`; `mark(MarkKind)`; `label(text, &mut Reveal)` | 11 row-painting skeletons and about 1,300 lines of hand-drawn rows in screens |
| `EditField` + `Validate` | `EditField { buffer: TextBuffer, sel: PointerSelect, undo, validator, error }`; `on_key(&Key, multiline) -> EditResult { response, exit: Option<EditExit> }`; `Validate { type Value; validate(&str) -> Result<Value, FieldError> }`; `ValidateWhen { Commit, Live, AfterError }` | The Picker and Combo queries, the table and grid cell editors, three validator types |
| `OptionList` | `{ cursor: Cursor, pane: ScrollPane, query: Option<EditField>, look: CursorStyle, wheel: WheelMode }`, rows are `Item` (`key, label, glyph, detail, tag, group, disabled, reason`) | The bodies of Select, OptionEditor, Picker, Completion and ContextMenu, plus `PickerItem` and `CompletionItem` |
| `ListCore` | `{ cursor, pane, selection, typeahead, reveal, content: ContentState }` | The model under List, Tree, Props, Steps, Table and Grid |
| `ContentState` | `Ready \| Empty(EmptyState) \| Loading(Cow<str>) \| Error(String)` | Two empty APIs on List, grid-only loading, blank empty trees |

### 1.7 Containers and layers

| Concern | Rule |
|---|---|
| Focus | The ring is the order of `RenderCtx::control` registrations. `Focusable::focusability` lets a container be one stop or pass focus through, and `remember_child` defaults on. `FocusFallback::Nearest` is the default (K8, SI22). |
| Layers | `Layer` (object-safe) = `Overlay` + `Component` + `HasHints` + [`on_tick`] + [`hides_page_focus()`, true by default; false for ContextMenu] + `take_result() -> Option<LayerResult>`. `ShellLayers = ModalStack<LayerEntry { layer: Box<dyn Layer>, tag: LayerTag, owner: PageKey }>`. The stack restores the owner's focus on close (K9). |
| `Dismiss` | Keep this branch's `{ escape, outside_click, pass_through }` and add `focus_out` and `refuse_feedback: Flash`. A dismissal resolves to the cancel result through `DismissReason { Esc, OutsideClick, FocusOut, Programmatic }`. |
| `LayerSpec` | Mirrors `main`: `LayerKind { Modal, Popover, Tooltip }`, `Anchor`, `Backdrop`, `Dismiss`, `initial_focus`, `restore_focus`, `inert_below`. It is built from the existing `popup::Placement`/`Place`/`Frame`. |

### 1.8 The shell (`junie_tui::shell`)

| Component | Responsibility | Replaces (per app: S showcase, T tablepro, H holla, J jackin) |
|---|---|---|
| `Shell<A: App>` with a builder | Implements `runtime::Application`: render prologue and epilogue, `ensure_focus`, blur fan-out, tick, input drain. Builder: `name`, `brand`, `min_size`, `routes`, `keys`, `pointer`, `context`, `clock`, `motion_env`, `about`, `run(CommonArgs)` | `impl Application`, `draw_frame`, `handle` (×4) |
| `App` | `type State, Route, Cmd`; `tick(&mut S, &dyn Clock) -> Vec<Msg>`; `on_command(Cmd, &mut S, &mut Cx)`; `stakes(&S) -> Stakes`; `chrome() -> Chrome<Cmd>` | App-level glue |
| `Page<S>` | `draw`, `update(&Intent, &mut Cx) -> Response<()>`, `on_paste`, `on_blur`, `on_tick`, `on_msg`, `on_layer_result(tag, LayerResult)`, `on_interrupt()` (Ctrl+C first refusal), `hints`, `keymap`, `crumb`, `primary_focus`, `editing`, `layer_open`, `animating`, `tick_ms`, `on_esc_top`, `stakes`, `context_menu(ContextRequest)` | S `Page`/`PageEvent`/`PageCtx`, T's concrete screens, the H and J `Screen` traits (about 30 methods each) |
| `Cx` / `Request<R>` | `Status(text, Tone)`, `Error`, `Open(Box<dyn Layer>, LayerTag)`, `Go(Nav<R>)`, `Copy(String)`, `Close`, `Help`, `FocusNext`, `Quit`. Background pages may emit only `Status` and `Error` | Three `Request` enums and about 40 `with_top`/`apply_requests` blocks |
| `PointerRouter` | Owns `PointerState`. Handles press, release, click count, drag (with the drag-end hook), wheel on both axes (`wheel_lines`), secondary, middle, pass-through re-click, dead space, surface press, flash and `FocusOn::Press`. The chain is top layer → chrome → page | `on_mouse`/`route_mouse`/`on_release`/`modal_click`/`modal_outside_click`, about 1,340 lines |
| `KeyRouter` + `ShellKeys` | Fixed order: too-small, layer, open menu bar, capture (`editing() \|\| layer_open()`), `page.on_interrupt` for Ctrl+C, `ShellKeys`, page, fallback (Tab/BackTab; Esc goes to `on_esc_top` and then `router.back()`). `ShellKeys` is a `KeyMap<ShellAction>` holding `QuitKeys`, `help`, `menu_bar`, `context`, `next_page`/`prev_page` and focus navigation. Applies the activation flash | 4 `on_key` functions (about 590 lines), `esc_ladder`, `workbench_chord` |
| `Footer` | `compose(top layer hints, menu, page hints + reveal advertise + auto "Tab <next label>" (de-duplicated, from `ring.next()`), fallback hints)` → `HintBar`, with clickable hints (M1) | 4 `draw_footer` functions (296 lines) and the hand-written modal hints |
| `StatusLine` | `set(msg, Tone)`. Priority error > hint > info. Info expires after 5 s on the shell `Clock`; an error stays until the next action (SV11). Truncation is the library's | 4 `set_status` functions with 4 TTLs |
| `HelpOverlay::from_keymaps(&[HelpSection])` | Sections built from `ShellKeys`, `page.keymap()`, the focused widget's `Bindings`, and optional mouse rows. Opened by F1 everywhere, and by `?` when no field is editing | 4 help renderers (about 500 lines, hand-typed and already wrong) |
| `Router<R: Route>` | `routes!{}`, `Nav { Push, Replace, Back, Tab(TabOp), Quit }`, optional per-tab stacks, `enter`/`leave` lifecycle, the `LeaveGuard` check on every route change | S `goto`, T `Screen` enum, H `go` (159 lines), J `go` (197 lines) plus `ROUTES` |
| `Chrome` | `menu_bar(Vec<Menu<Cmd>>)` with typed commands, `tabs(TabPolicy)`, `status_segments`, `breadcrumb(clickable)` (M5), `about(Props)` | H `build_menu`/`run_menu`/`draw_menu`, J `*_host_menu` (about 270 lines) |
| `QuitGuard` | Built from `App::stakes` / `Page::stakes` and the existing `ui::leave::{LeaveGuard, QuitKeys, Stakes}`; the shell owns the dialog and the drain-then-quit loop. No new trait | T `request_quit`, H and J `open_quit_confirm` and the arbiter |
| `Clipboard` | Installs the OSC 52 writer (with its size limit) as `edit::clipboard::set_hook`; `Request::Copy` reports to `StatusLine` | H `copy_osc52`, three inline copies in J |
| `Clock` / `Motion` | `Clock { now() -> Moment }` with `WallClock` and `SimClock`; `Motion { Full, Reduced, Paused }` with `Motion::resolve(cli, env)`. Tick intervals come from `ShellConfig` | 4 `Motion` enums, 2 `clock.rs` |
| `CommonArgs` | `#[derive(clap::Args)]`: `--color`, `--motion`, `--frame`, plus the `--no-motion` env | About 90 identical lines in each app |
| `TooSmall` | `TooSmall { brand, min }` | 4 copies, about 25 lines each |

### 1.9 Naming: one set, mapped to `main`

| This plan | Names in the reviews | `main` (crate) | Decision |
|---|---|---|---|
| `Intent`, `Phase` | `UiEvent`, `PointerEvent`, `PointerKind` | `Intent`, `Phase` (core) | Adopt `main`. Add `count` (triple click); `main`'s `DoubleClick` phase cannot express three clicks, so add `count` there when porting. |
| `Part`, `PartRef`, `ItemKey` | `enum Part { Row(usize), .. }` | `Part(u16)`, `PartRef`, `ItemKey` (core) | Adopt `main` exactly. |
| `Response<A>`, `Flow`, `Invalidate` | `Response<E> { outcome, event }` | Same (core) | Adopt `main`. `Outcome` is a temporary adapter. |
| `StateFlags` | `RowVisual`, `VisualState` | `StateFlags` (core) | Adopt `main`. |
| `XAction` (widget output), `XCmd` (key-map command) | `ListEvent`, `TreeEvent`, `GridEvent`; key-map enums named `ListAction`, `GridAction`, ... | `ListAction`, `ListCmd` (navigation) | Adopt `main`: in one rename (S0), the current `XAction` key-map enums become `XCmd` and the `XEvent` enums become `XAction`. |
| `Component::update`, `Draw::draw`, `Measure::measure` | `handle`, `render`, `on_key`/`on_click`/... | Inherent `update(cx)`, `draw(ui, area)`, `measure(ui, c)` | Adopt `main`'s verbs. Add the traits that `main` lacks. Without them `main` needs glue per container (worse); port the traits. |
| `RenderCtx` | `RenderCx` | `Ui` | Keep `RenderCtx` here (renaming touches every widget for no behaviour); it becomes `Ui` in the port. |
| `Cx` | `EventCx`, `LayerCx`, page `Cx` | `Cx` (runtime) | One `Cx` for widgets, layers and pages, as in `main`. |
| `Layer`, `LayerSpec`, `LayerKind`, `DismissReason`, `LayerEvent` | `Layer`, `ModalKind`, `Overlay`, `Dismissal` | `LayerSpec`, `LayerKind`, `DismissReason`, `LayerEvent` (core, runtime) | `Overlay` is the widget capability; `Layer` is the shell stack entry; `ModalKind` is dropped. `Dismissal` is renamed `DismissReason`. |
| `Dismiss { escape, outside_click, pass_through, focus_out, refuse_feedback }` | (same) | `Dismiss { esc, outside_click, focus_out }` | **Keep ours.** `main` cannot express pass-through or refusal feedback (visual-testing plan L5); extend `main`'s when porting. |
| `ScrollState`, `ScrollPane` | `ScrollPane` | `ScrollState`, `ScrollRegion` | Keep `ScrollPane` (state plus bars). It maps onto `main`'s `ScrollState` plus `ScrollRegion`. |
| `ListCore`, `Cursor`, `RowSource` | (same) | Separate `ListState`/`TreeState`/... each with `cursor: Option<ItemKey>` | Keep `ListCore`. `main` repeats the cursor in every state struct (worse); each `XState` embeds `ListCore` in the port. `RowSource::key` gives the `ItemKey` bridge. |
| `RowUi` | `RowPainter` | `RowUi`, `RowFn` (runtime) | Adopt `main`'s name. |
| `EditField`, `Validate`, `FieldError` | `EditField`, `Validator`, `Validate` | `field_control`, `Validate`, `FieldError` (core) | Adopt `main`'s `Validate` and `FieldError`. `Validator<V>` is the closure implementation. |
| `Item` | `PickerItem`, `CompletionItem`, `MenuItem` rows | `Item` (navigation) | Adopt `main`'s `Item` for `OptionList` rows. |
| `KeyMap<XCmd>`, `Bindings`, `ShellKeys` | `ShellKeys`, `Actions` | `KeyMap` (ActionKey, `KeyPhase`), `Bindings` | Keep typed `KeyMap<C>` for widgets (type safety). `Bindings` adopts `main`'s name. `ShellKeys` maps onto `main`'s phased `KeyMap` (`bind_before_typing`). |
| `Focusable`, `Focusability` | `Focusable`, `FocusPolicy` | `Focusability`, `ScopeId` | Adopt `main`'s enum. |
| `Shell`, `App`, `Page`, `Router`, `Request`, `Footer`, `StatusLine`, `Chrome` | `AppShell`, `Shell`, `Screen`, `Page` | `App` (runtime) only | Keep. A new crate `termrock-shell` builds on `main`'s `App`/`Runtime`. `Screen` is rejected: it collides with tablepro's `Screen` enum. |
| `Clock`, `Moment`, `Motion` | `Clock { now_ms }` | `Moment`, `FeedbackClock` (runtime) | Adopt `Moment` as the return type. |
| `HelpOverlay::from_keymaps(&[HelpSection])` | `HelpSpec::from` | `HelpOverlay`, `HelpSection` (overlays) | Adopt `main`'s `HelpSection`. |
| `TooSmall`, `NavList`, `FilterList` | (same) | `too_small`, `nav_list`, `filter_list` | Same names. Compare APIs on port. |
| `StatusLine` | `StatusLine` | `StatusItem` (feedback) is a bar segment, not a message | Different concept; keep both. |

## 2. New library components (from the screens review)

| Component | API sketch | Adopters (sites deleted) | Step |
|---|---|---|---|
| `FormPanel` | `FormPanel::new(id, fields).tabs(..).columns(2).actions(..)`; `Component<Action = FormEvent>`; `FormField::pair(a, b)`; disabled-when predicates | T connections (629-1027, 1264-1487) and filter editor (through `FormDialog`); S forms, settings, selectable | S18 |
| `MasterDetail` | `MasterDetail::new(list, detail).min_split(n).detail_pct(p).collapse_below(w)`; `draw(area, cx, \|main\|, \|detail\|)`; `toggle_drawer()`; `menu_item()`; Esc/Tab handling in `update` | H cleanup, disk, files, finder, plan; J accounts, manager, usage; T connections, workbench, tabs; S app, taskrunner, selectable | S19 |
| `FilterTree` / `FilterList` | `{ input: TextInput, tree \| list }`; `/` edits, ↓ enters the list, Tab moves on; `set_filter`; `Component<Action = TreeAction \| ListAction>` | T connections, workbench; S sidebar; J's 3 filter prompts | S21 |
| `EditorShell` | `EditorShell::new(tabs, cancel, save)`; tabs with dirty and error flags, `[`/`]`; `ShellAction { Tab(i), Body, Save, Leave }`; built-in `LeaveGuard` prompt; "• N changes" | J settings, editor; later T commit flow | S20 |
| `PageHeader`, `Heading` | `PageHeader::new(title).status(Vec<Segment>).right(..).subtitle(s, Tone).draw(..) -> u16`; `Heading::new(t).meta(m)` | 11 H headers plus H `mod.rs:463 heading`; about 17 J headings; S app | S17 |
| `NavList` | Moved from `showcase/pages/sidebars.rs`: `NavList::new(id, Vec<NavItem { label, icon, section, badge, disabled }>).current(i).collapsed(b)`; `ClickPolicy` Activate (a navigation role, SI14) | S sidebar; later any app sidebar | S17 |
| `DetailCard` | `DetailCard::new(title).meta(m).props(v).actions(..).empty(EmptyState)` → body `Rect` | T connections, history, affected rows, plan cards; S selectable | S17 |
| `ActionRow` | `ActionRow::new(&mut [&mut Button]).message(text, Tone).align(Right)`; one button-row navigation (no wrap, skips disabled: K32) | H snapshot, activity, review; T connections, tabs; S forms, settings, selectable; `Dialog`/`FormDialog` buttons | S17 |
| `CommandMenu<T, C>` + `ContextAnchor::row(list, i, area, &scroll)` | Moved from `jackin_preview/screens/rowmenu.rs`: `open(target, anchor, title, Vec<(MenuItem, C)>)`; `Component<Action = (T, C)>` | J accounts, manager, config; H `mod.rs:376-436` | S22 |
| `FileBrowser` | Moved from J `modals.rs:30-526`; file access behind `trait FileSource`; implements `Layer`; takes `&[Step]` | J prelude, config | S22 |
| `StepLine` + `Wizard<S>` | `StepLine::new(&[Step])` (inline `StepRail`); `Wizard` handles steps, Back/Next and back-navigation; `ChoiceDialog::stepper(&[Step])` | J prelude; S taskrunner | S22 |
| `TextViewport::find()` + `FindState` | `FindState { query: EditField, matches, at }` with `update`, `meta()`, `apply(&mut TextViewport)`; `TextViewport::line_numbers(bool)` | H activity, files; J cockpit log | S22 |
| Small helpers | `Tabs::layout(area) -> (strip, body)`, `Tabs::passive()`; `MeterList`; `Notice::error(msg).detail(d).max_lines(n)`; `TextInput::readout(..)`, `render_state(StateFlags)`; `Item` builder with a `value: usize` payload; `progress::spinner` frame from `ctx.interaction.tick`; `ui::text::{plural, truncate_sep, duration_ms, bytes, crumb_fit, center_x}`; `Column::marker()`, `RowMark { dirty, pointer }`, `ListItem::header()`, `ListItem::status(Tone)`, `TreeNode::{meta_tone, trailing, label_tone}`, `TreeView::columns(Vec<Column>)` | All apps (sites in duplication-screens.md) | S2, S16, S17, S22 |

## 3. Drift decisions (one default, one override)

Default policy: macOS, then APG, then WCAG (standards-interaction.md). Each override sits at the layer shown: **P** app profile or `ShellConfig`, **W** a per-widget field and builder, **K** the user's key map.

| Divergence (where observed) | Library default | Override |
|---|---|---|
| Focus on click: S Release, others Press | `FocusOn::Press` | P `PointerConfig::focus_on` |
| Double click: count vs `on_double_click` fallback | `count` in `Intent::Pointer`; 500 ms, 1-cell slop | P `double_click_ms` |
| Drag-end hook (J only) | Every target gets `Phase::DragEnd` | none needed |
| Click flash (page only vs every click) | Every activated target, modals included, never on dead space | P `flash_ms` (0 = off) |
| Horizontal wheel (S ±1; table and grid 1 cell) | `wheel_lines` on both axes, through `HScroll.step`; native buttons 66/67 accepted (SI9) | W `HScroll::step`, P `wheel_lines` |
| Middle click on a tab (H only) | Closes the tab inside `Tabs` | W `Tabs::middle_click_closes` |
| Secondary click with nothing hit | Ignored. The surface target opens its menu if it has one | W `ContextSource` on the surface |
| Dead-space release with the menu bar open (H only) | Closes the menu (light dismiss) | W `MenuBar::dismiss` |
| Outside click on a modal (T no result, J Esc result, H table) | Resolves to the cancel result. A refused click flashes (M24) | W `Dismiss` + `refuse_feedback` |
| Modal background tick (H and J only) | `Layer::on_tick` for every layer | — |
| Background-page requests (J all, H Status/Error, S none) | Status and Error only | none (a correctness rule) |
| Blur fan-out (H dialogs only, J also Browser and Form) | Every layer, then every mounted page | — |
| Focus fallback (`First` vs `primary_focus`) | `FocusFallback::Nearest`, then `primary_focus` (K8, SI22) | P `ShellConfig::focus_fallback` |
| Key order (page first in S; chords first in H, J, T) | `KeyRouter` order (§1.8); printable keys go to an editing field first (WCAG 2.1.4) | K `ShellKeys` |
| Ctrl+C (S quit, T cancel then guard, H/J bypass) | Copy if there is a selection, else `Page::on_interrupt`, else quit through `LeaveGuard` (SI17) | K `QuitKeys` |
| Too-small keys (`q` vs `q`/Ctrl+C) | `QuitKeys` as everywhere else | K |
| F10 vs context key gating (H config, J hard-coded) | `ContextConfig::keys`; F10 plus Alt+mnemonic opens a menu; a second, documented menu-bar binding (SI10) | K `ShellKeys::menu_bar` |
| Activation flash gate (`!was_editing`, `!typing`, `!editing`) | `!page.editing() && Changed` | — |
| Help (four renderers; `?` typed into search) | One `HelpOverlay::from_keymaps`; F1 always; `?` when no field is editing; F1, `?` and Esc close it | K `ShellKeys::help` |
| Footer fallback (F1 vs ? vs none) and alignment | `F1 Help` fallback; auto "Tab <next>", de-duplicated; left-aligned (SVg10, **visual change**) | P `Theme::hintbar.align` |
| Status TTL (4 s wall / 5 s wall / 5 s sim) | Info 5 s on the shell `Clock`; errors persist | P `ShellConfig::status_ttl` |
| Motion (S has no Reduced) | `Motion::{Full, Reduced, Paused}` everywhere, with `<APP>_NO_MOTION` | P `motion_env` |
| Wheel in popup lists (Select and OptionEditor move the cursor; Picker, Completion and Menu scroll) | Scroll the viewport; the cursor follows only on a keyboard move (`FollowCursor::OnMove`) | W `OptionList::wheel: WheelMode` |
| Popup overflow (`more_above` glyphs vs a real scrollbar) | A real `ScrollPane` bar | W `scrollbar(ScrollbarVisibility)` |
| PageDown, disabled-row skipping, Left/Right return value | `Cursor::apply`: pages by the viewport, skips disabled rows, `Ignored` for unhandled ←/→ | W `wrap_cursor`, K `NavKeys` |
| Type-ahead miss (Ignored vs Consumed) and predicate | Miss = Consumed; always `RowSource::enabled` | W `type_ahead` |
| Vim letters on, type-ahead off | `NavKeys::NO_VIM`, type-ahead on in lists, selects, menus and trees (SI12, K33, K34) | P `Vim` profile, W `nav`, `type_ahead` |
| Activate on click (list double, tree and props single, table and grid click-again) | Data views: `Activate::Double`; navigation lists, sidebars and menus: `Single` (SI14, M15) | W `ClickPolicy` |
| Edit on click (table and grid: second click) | `edit_on: DoubleClick`; the focusing click never edits (M9). Grid cells start in `Explicit` (APG grid, SI6) | W `ClickPolicy::edit_on` |
| Selection shrink, Shift-click, Ctrl-click, gutter toggle | `SelectGesture` everywhere; a right-click outside the selection selects only that row (M10) | W `toggle_mods`, P `ClientPlatform` |
| Checkbox click | `CheckClick::BoxOnly` (M15, M34) | W |
| Chosen mark colour (list accent on hover; table and tree `t.chosen`) | `RowUi::mark` always uses `t.chosen(focused)` | Theme |
| Reveal (list and props keys; others hover only) | `RowUi::label` wires reveal everywhere. The full value shows in the peek line on focus, plus a peek key; never → (SI8). Delay 300 ms (M25) | W `reveal: OnHoverAndKey`, `Reveal::delay_ms` |
| Horizontal step and dead bars | `ScrollPane` for all | W `HScroll` |
| Track click | `TrackClick::Page` with hold-repeat (SI9, M35) | P `RenderCtx::track_click`, W |
| Validation (three types; revalidate on Insert vs clear) | `Validate`, `ValidateWhen::Commit`, then `AfterError` while an error is shown (SIg9) | W `validate_when` |
| Esc in multi-line (TextArea commits; old plan: revert) | Esc = Done, keeping the text; revert through undo (SI19, already the default) | W `esc_reverts` |
| Select All (text: Ctrl+L only in the Emacs preset, Ctrl+A in Desktop; tables Ctrl/Cmd+A hard-coded; lists only `a` toggle-all; viewports none) | Cmd+A wherever the terminal delivers Cmd (kitty keyboard protocol, pushed on enter where supported). Text fields follow the client platform's `FieldKeys` (macOS Emacs: Ctrl+A line start, Ctrl+L select all; elsewhere Desktop: Ctrl+A). Multi-selection rows and read-only text: Cmd+A and Ctrl+A (`keymap::SELECT_ALL`); single selection ignores it. Hints name ⌘A only once Cmd delivery is confirmed | P `Theme::field_keys`, `JUNIE_TUI_KEYBOARD_PROTOCOL`; W `field_keys`, `text_keymap`; K `TextCmd`, `ListAction`/`TableAction`/`GridAction`/`ViewportAction::SelectAll` |
| Text-field entry | `EditEntry::Immediate`; keyboard focus selects all, a pointer click places the caret (SI6, K1) | P `Theme::edit_entry`, W |
| Select closed arrows (`Step`) and Tab in an open Select | `SelectArrows::Open`; Tab chooses and moves on (SI13, K13) | W `arrows`, `tab_in_open` |
| Menu unknown key (swallowed) | Accelerators and type-ahead; any other key closes the menu and passes through (K14) | W `unknown_key` |
| Menu open on release | `open_on: Press`; press-drag-release chooses; right-drag-release (M12, SI4) | W `MenuBar::open_on`, `press_drag` |
| Destructive dialog default | Cancel is the default and focused button; danger is never primary (SI11, SV9) | W `default_button` |
| Splitter (mouse only, 1-cell hit) | Hit area ±1 cell; grab offset; Alt+Shift+arrows on the owning pane through `SplitKeys`, hinted (K18, M20) | W, K |
| Tab switching keys | ←/→ and digits only while the strip has focus; Ctrl+PgUp/PgDn globally (SI23) | K |
| Prefix timeout (J 2 s) | Sticky; Esc cancels (K26) | W `prefix_timeout_ms` |
| `wrap(bool)` meaning cursor wrap or line wrap | `wrap_cursor`, `wrap_lines`, `scroll(ScrollConfig)` | — (naming) |

## 4. Execution plan

**Already done on this branch; not re-planned:**

| Findings | Commit or API |
|---|---|
| M4 | `ScrollState::follow_cursor` |
| M11 | Tabs scroll window and wheel |
| M29 | Tree deep rows, inert note rows, Alt-click |
| K2, K22 | `Dialog::ack_enter`, mnemonics |
| K3 | holla Ctrl+C reaches the page first |
| K4, K21 | Library half: `ui::leave::{LeaveGuard, QuitKeys}` |
| K15, K29 | `request_focus_next`, undo, `FieldKeys` |
| SI2 | `guard_dirty_prompt` |
| SI19, SI20 | `esc_reverts` defaults to false; `FieldKeys::Emacs` |
| PO Select All | `core::keyboard` (kitty protocol, `super_delivered`), `Key::cmd`/`chord`, `FieldKeys::for_platform`, `TextCmd::SelectAll` through `edit_key_in` and `EditAction::SelectAll`, `keymap::SELECT_ALL` in the List, Table, Grid and Viewport key maps (keyboard.md, "Select All") |
| SVg6 | `Glyphs::ASCII` |

**Groundwork only, finished by the step shown:**
- `EditEntry` exists, but the theme default is still `Explicit`. Finished in S11.
- `TrackClick::Page` exists, but the default is still `Jump`. Finished in S3.
- `FocusFallback::Nearest` exists, but the runtime uses `First`. Finished in S7.
- `ensure_focus`, `ModalStack` and the `Modal` trait exist, but no app or widget adopts them uniformly. Finished in S5 and S7.

**The gate (G), on every step:**
1. `cargo nextest run --all-targets`, which includes the showcase digest (`tests/showcase_baseline.txt`), `focus_gutter`, `overlay_sites` and the journeys.
2. `cargo clippy --all-targets` and `cargo fmt --check`.
3. tuiscotti pilot: `cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(/^showcase\/pages/)'`.
4. tuiscotti smoke: `cargo nextest run --profile smoke --run-ignored only`.
5. The step's targeted regression tests, which must fail on the parent commit.

The rules that go with the gate:
- **No behaviour change** except the intended drift fixes the step lists. Each drift fix is its own commit with its test.
- A visual change is named in the commit message and re-approved with `cargo xtask visual accept <filter>` in a separate commit. `visual-baseline` is never touched.
- Pattern: **expand** (add the library API plus an adapter over the old API), **migrate** (move every caller in all apps), **contract** (delete the old API and the app copies).
- `rg` checks named per step must return 0 hits after the contract.

**Parallel groups** (agents work on disjoint files in isolated worktrees):

| Group | Steps |
|---|---|
| A | S0 alone |
| B | S1 ‖ S2 |
| C | S3 ‖ S4 ‖ S5 |
| D | S6 |
| E | S7 |
| F | S8a ‖ S9 ‖ S10 |
| G | S8b ‖ S11 ‖ S12 |
| H | S13 ‖ S14 |
| I | S15 |
| J | S16 (per app in parallel) ‖ S17 |
| K | S18 ‖ S19 ‖ S20 ‖ S21 ‖ S22 |
| T | Theme track, parallel to anything from group B on |

**Grouping rationale.** Adapters come first: `Response`, `Intent` and `PartRef` exist before any caller moves. The scroll, cursor and layer layers come next, so the router has something to route to. The `Page` trait lands before the key and hint layers it needs. Screens move last, because they depend on all of the above.

| Step | Adds (expand) | Migrates; deletes in all apps (contract) | Findings placed here | Acceptance beyond G | Deps |
|---|---|---|---|---|---|
| **S0** Conventions | `core::{intent, response}`: `Intent`, `Phase`, `Response<A>`, `Flow`, `Invalidate`, `StateFlags`; `Part`/`PartRef`/`ItemKey`; `From<Outcome>`. Rename the key-map enums `XAction` → `XCmd` and the output enums `XEvent` → `XAction` | Mechanical rename in `src/widgets/*` and `src/bin/*`; no logic change | — | Digest unchanged; `rg 'enum \w+Event\b' src/widgets` = 0 | — |
| **S1** Part-tagged hits | `HitRegistry::{register_part, resolve}`, `Hit { owner, part }`; widgets register parts alongside their ids | Widgets read `PartRef` internally; `owns`/`locate`/`row_id` become adapters (deleted in S6) | Jackin `manager.rs:1625` y-keyed row ids become `ItemKey` | Hit unit tests: every widget part resolves | S0 |
| **S2** Shell leaf pieces | `cli::CommonArgs`, `motion::Motion`, `clock::{Clock, WallClock, SimClock}`, `shell::StatusLine`, `widgets::too_small::TooSmall`, `ui::text::{plural, truncate_sep, duration_ms, bytes, crumb_fit, center_x}`, `Theme::centred_hints` | The four `main.rs` CLI and `Application` blocks (about 90 lines ×4), the `Motion` enums ×4, H/J `clock.rs`, `plural` ×2, `centred_hints` ×2, too-small ×4, `drain_pending_input` for all apps | Showcase `--motion reduced` (drift); status TTL and error persistence (SV11); SVg8 reduced motion | `--help` and an unknown flag exit non-zero (visual-testing F2); a status-expiry test | S0 |
| **S3** ScrollPane | `core::scroll::ScrollPane`, `Scrollable`, `scrollbar::render_with_fade` | 17 widgets embed `ScrollPane`; delete `on_wheel*`, `on_scrollbar*`, `on_bar_drag`, `track()`, both follow modes; the 26 screen fade-plus-bar copies call `render_with_fade` | M17 (Picker, Completion, TextArea, Help, Info, Form bars), M33, M35/SI9 (`TrackClick::Page` default), M27 autoscroll tick, SV13 track glyph (**visual**), SVg9 focus kept visible with 1 row of context; Help/Form/Info always returning `Changed` | Drag and press tests per bar; an h-step test on table and grid | S1 |
| **S4** Cursor, RowSource, type-ahead | `core::list::{Cursor, RowSource}`, `Cursor::type_ahead`, `KeyCx::unassigned_chord` (dispatcher) | 22 `nav.action` blocks, 4 `step()` functions, 7 type-ahead copies, 10 chord guards. OptionEditor stops matching raw `KeyCode` | K33, K34, SI12 (`NO_VIM` and type-ahead defaults, P `Vim` profile), SIg10 (type-ahead reset 500–1000 ms); `ContextSource::context_select` through `Cursor` (part of M10) | One table-driven nav test for every `Navigable` | S0 |
| **S5** Overlay and Layer | `Overlay`, `Layer`, `LayerSpec`, `LayerKind`, `DismissReason`, `ShellLayers`, `Dismiss::{focus_out, refuse_feedback}`, `Frame::close_button` | Implement for every library popup and modal; the T `Filter` and J `Op`/`Custom` modals become `Layer` impls in their apps; delete the `Modal` enums, `ModalTag` ×2, `ModalEntry`, `modal_key`, `deliver`, `push_modal`/`pop_modal` | M13 (× close, HelpOverlay 80 %), M24, K9 (popups restore the owner's focus), K13, K14, K31, K32, SI11/SV9 (Cancel default), SIg1 (ignore a click within 300 ms of FocusIn, `?1004`), SIg8 (reposition on resize); blur drift (H Form/Browser) | `overlay_sites` stays green; dismissal tests per layer | S1 |
| **S6** PointerRouter | `core::pointer::PointerRouter`, `MouseTarget` (blanket impl plus legacy adapters), `MouseKind::{SecondaryUp, SecondaryDrag}`, clickable `HintBar` | Delete `on_mouse`, `route_mouse`, `on_release`, `modal_click`, `modal_outside_click`, `secondary_at`; 86 `.owns(`, 43 `.on_scrollbar(`, 78 `.on_wheel(`, 32 `.locate(` call sites in `src/bin`; contract `owns`/`locate` | M1, M14, M17 (routing), M18, M12 (SecondaryUp, press-drag), M38, M37 (pointer and caret shapes, opt-in); all mouse drift rows in §3 | `pointer` journeys, the M14 middle-click harness, the M18 wheel-under-Select test; `rg '\.owns\(\|fn route_mouse' src` = 0 | S3, S5 |
| **S7** Page, Cx, Request | `shell::{Page, Cx, Request, LayerTag, PageKey}`; `Shell` handles focus upkeep and blur fan-out; a showcase `PageEvent` adapter; tablepro split into Connections and Workbench pages | Delete H and J `screens/mod.rs` framework copies (about 600 lines), T `Cx`, S `PageCtx`, 4 `forward_blur`, about 40 `with_top`/`apply_requests` blocks, 4 render prologues | K8 (Nearest fallback), K27 (covered content is inert), K30 (visual order); the background-request policy | Blur fan-out tests per app; digest unchanged | S5, S6 |
| **S8a** KeyRouter and ShellKeys | `shell::{KeyRouter, ShellKeys, ShellAction}`, `Page::on_interrupt` | 4 `on_key` functions, `esc_ladder`, `workbench_chord` | K11 (focused MenuBar handles its own keys), K20 (`wants_text_input` includes find), K21 (remaining apps), K26, K28, K35, K37 (`skip_when_empty`), K38, SI10, SI23 (Ctrl+PgUp/PgDn), SI16 (coalesce only wheel, motion, resize and repeated navigation) | One cross-app key-order table test | S7 |
| **S8b** Hints, footer and help | `HasHints`, `Bindings`, `HintCx`, `shell::Footer`, `HelpOverlay::from_keymaps`, `HelpSection`; the **component docs generator** (§5) | 4 `draw_footer` functions, modal hints in T/H/J, 4 help renderers (about 500 lines), 16 `"Enter","Edit"` literals | K10, K23, K39, K25 (`GridEvent::ReadOnly`, hints drop Edit), SI7 (hide undeliverable chords), SIg4 (kitty-detected keys feed the hints), SVg10 (left alignment, **visual**); the "no keyboard-only state" coverage test (mouse model) | Hint snapshot per widget; the coverage test walks every page's `KeyMap` | S8a, S4 |
| **S9** Selection and clicks | `SelectGesture`, `RowSelection::apply`, `ClickPolicy`, `Activate`, `CheckClick` | List `checked`/`anchor`, Grid `selected_rows`, OptionEditor `checked`; T `connections.rs:839` empty double-click branch, H `disk.rs:1338` toggle-on-click | M9, M10, M15, M16, M32, SI14, SI5 (checkbox cell toggles with no modifier), SI24 (commit on selection change, ask only on invalid), SIg2 (Shift-click fallback) | The M15 harness (holla Disk) flips; a selection model table test | S4, S6 |
| **S10** RowUi | `ui::row::RowUi`, `RowPaint::{mark, label}`, `RowLook` | 11 painting skeletons in widgets | K12 (keyboard reveal in Grid, Table, Menu, Picker, Select, StatusBar), SI8 (peek line plus peek key), M25 (300 ms delay), SV14/SVg (non-colour selection cue: **visual**), SV1 (bold for neutral active: **visual**), SVg3 (`highlight_on_selection`) | Digest unchanged before the visual commits; a contrast test for each new token | S4 |
| **S11** EditField and Validate | `widgets::edit::EditField`, `Validate`, `Validator`, `ValidateWhen`, `FieldError`; `PointerField` implemented once on `EditField` | Picker query (`picker.rs:224-606`), Combo query, table and grid `edit_key` and click-to-caret copies; contract `TextValidator`, `table::CellValidator`, `grid::Validator` | M2, M21, M31 (ScrollPanel selection through `TextViewport`, which brings Select All), M39, Select All for the picker, combo and find queries (inherited from `EditAction::SelectAll` once they sit on `TextBuffer`), K36 (finder query on `TextInput`), K1 (flip `Theme::DEFAULT_EDIT_ENTRY` to `Immediate`), K5, K7, K17 (Esc then Tab plus F6, hinted), SI6 (`FocusSelect::SelectAll` on keyboard focus), SIg5 (bracketed paste never fires shortcuts), SIg9 | The K1 walk: a caret at every text stop; the M2 CodeEditor drag test; an intended re-approval of field captures (**visual**: caret shown) | S4, S6 |
| **S12** OptionList and Item | `widgets::option_list::OptionList`, `Item` (main's shape, `reason`, `value`) | Bodies of Select, OptionEditor, Picker, Completion and ContextMenu; `PickerItem`/`CompletionItem` literals in T (`app.rs:1415-1523, 1653-1686`) | M19, M38, SI13 (`SelectArrows::Open`), SI21/SV15 (disabled reason in the status row, dimmed shortcut kept); T tab-list glyph mismatch (`app.rs:1655` vs `1682`) | Popup journeys; wheel tests in all five | S3, S4, S5 |
| **S13** ListCore, ContentState, ContextSource coverage | `ListCore`, `ContentState`, the standard Edit `ContextSource` (Cut, Copy, Paste, Select All, OSC 52; Select All runs `TextCmd::SelectAll` and its shortcut label is `label_for(text_keymap, SelectAll)`, so ⌘A shows only where Cmd is delivered) | List, Tree, Props and Steps on `ListCore`; the two List empty APIs; `ContextSource` for Props, Steps, Viewport, CodeEditor, TextInput/TextArea, Picker, Chips | M3, M22 (`context_items` from the key map), M30 (`+N` popup), M6 (`EmptyState::actions`) | A right-click test on every text widget | S9, S10, S11 |
| **S14** Router, Chrome, Clipboard, quit | `shell::{Router, Nav, Chrome, Breadcrumb, Clipboard}`; `App::stakes`; menus carry typed `Cmd` | S `goto`, T `Screen` enum, H `go` with its tab stacks, J `go`/`Screens`/`ROUTES`; H and J menu-bar code; label-string re-dispatch (H:1569, J:1667); `copy_osc52`; `request_quit`/`open_quit_confirm` ×3 | M5, M7 (toolbar), M8 (collapsed rail), M23, M40, K4 and K6 (dirty-form guard on every route), SI17, SIg3 | Quit and leave journeys per app | S7, S8a |
| **S15** Component and Container | `Component`, `Draw`, `Measure`, `Container::visit`, `AnyComponent`; builder renames (`wrap_cursor`, `wrap_lines`, `scroll(ScrollConfig)`, `set_*`) | Delete the `MouseTarget` legacy adapters, the `Outcome` adapter, the free render functions (`empty::render`, `props::render`, `keyhint::render*`) and the per-widget `on_*` methods | M28 (header seams and header menu), K19 (column cursor for sort) | `rg 'fn on_click\b\|fn on_wheel\b' src/widgets` = 0 | S6–S13 |
| **S16** Row and tree extensions | `Column::marker`, `RowMark`, `ListItem::{header, status}`, `TreeNode::{meta_tone, trailing, label_tone}`, `TreeView::columns` | S16a: library; S16b: H rows (cleanup, disk, files, finder, plan); S16c: J rows (settings, editor, usage, inspect, config, manager) and the manager/accounts trees (the double toggle hit); S16d: T EXPLAIN tree-table | 69 screen list-key arms disappear (K: every screen gets the widget keys) | Per-app captures unchanged (pixel-equal); the J tree hit test | S10, S13 |
| **S17** Small composites | `PageHeader`, `Heading`, `ActionRow`, `DetailCard`, `NavList` (moved), `Notice`, `MeterList`, `Tabs::{layout, passive}`, `TextInput::readout`, `Item` builder | The 11 H headers, about 17 J headings, S headers; button rows in H, T and S; S `sidebars.rs`; the tab-strip `y+3` offsets; S `inputs.rs` fake input | M34 (`close_target_w`), SVg (glyph ASCII fallback is already `Glyphs::ASCII`: verify only) | Captures unchanged | S8b, S10 |
| **S18** FormPanel | `FormPanel`, `FormField::pair` | T connections form engine; T filter editor through `FormDialog`; S forms, settings, selectable | K6 (Esc runs Cancel with a dirty confirm) | Form journeys | S11, S17 |
| **S19** MasterDetail | `MasterDetail` | 14 sites across all 4 apps; `SPLIT_MIN` imports | M20 (splitter handle in T, hit ±1, grab offset), K18 (`SplitKeys`: Alt+Shift+arrows, hinted) | Narrow-tier captures | S17 |
| **S20** EditorShell | `EditorShell` | J settings and editor shells | K4 (leave guard through `EditorShell`) | Editor journeys | S14, S18 |
| **S21** FilterTree / FilterList | `FilterTree`, `FilterList` | T connections and workbench filters; S sidebar filter; J's 3 filter prompts | — | Filter journeys | S11, S16 |
| **S22** Moved app widgets | `CommandMenu`, `ContextAnchor::row`, `FileBrowser` + `FileSource`, `StepLine`/`Wizard`, `FindState` + `line_numbers` | J `rowmenu.rs`; H `mod.rs:376-436`; J `modals.rs:30-526`; J prelude stepper; H activity and files find code; 12 wall-clock spinner sites; J dead code (`wrapped_arrow`, `saved_focus`, empty `modal_hints`) | — | J and H journeys | S5, S11, S13 |
| **T** Theme track | Plane-relative `text_faint` (SV3), mono hover and press (SV2), the panel title marker (SV4), `border_strong` field outlines (SVg1), one pressed direction (SVg2), success vs accent (SV14), `COLORFGBG` (SVg7) | Theme only; never app code | All **visual**, each re-approved on its own | Contrast test on every plane | S0 |

## 5. Per-widget documentation (`docs/components/<widget>.md`)

| Item | Decision |
|---|---|
| Source of truth | Each widget gets a `const DOC: WidgetDoc { name, role, standards: &[(&str, url)], knobs: &[Knob { name, ty, default, layer: P/W/K }] }`, plus its `Bindings` (keyboard rows, from the live default `KeyMap` and `NavKeys`), `HasHints` (hints by state) and a `MouseTable` built from the `ClickPolicy`, `ScrollPane` and `ContextSource` defaults (the mouse rows). |
| Generator | `cargo xtask docs components` writes `docs/components/<widget>.md` and `docs/components/README.md` (an index of roles and standards). The default test `docs_components_fresh` fails when a file differs from the generator. |
| Page layout | 1. Purpose and role. 2. **Keyboard** table: key, action, condition, remappable `XCmd`. 3. **Mouse** table: target `Part`, press, click ×1/×2/×3, secondary, drag, wheel. 4. Hints by state. 5. Knobs: name, default, layer, standard. 6. "Keyboard-only path" and "Mouse-first path" sections, each listing every action and how to reach it (the no-keyboard-only-state rule). 7. Standards references (HIG, APG pattern, WCAG SC). |
| Produced by | S8b adds the generator and the pages for the widgets that already have `Bindings`. S3, S4, S9, S11 and S12 add knob and mouse rows to their widgets' `DOC`. S15 makes `WidgetDoc` a `Component` supertrait requirement, so a widget without docs does not compile. Each step's PR commits the regenerated pages. |
| Audiences | Keyboard-only and mouse-first readers each get a table that is complete on its own. A row missing from either path fails the coverage test introduced in S8b. |

## 6. Risks, mitigations and expected deletions

| Risk | Mitigation |
|---|---|
| Every app's event path changes (S6, S7, S8a) | Each lands behind adapters; the old paths are deleted only in the contract commit. G plus the per-app journeys run on every commit; the pointer and keyboard audit harnesses are re-run as regression tests. |
| A visual change is slipped in | The digest (`tests/showcase_baseline.txt`) and the smoke tier gate every commit. Visual changes are only the **visual**-marked items, each in its own re-approval commit. |
| Parallel agents conflict in the same widget files | Each step owns disjoint files (§4 groups). S0 and S15 are cross-cutting and run alone. App screen migrations (S16) are split per app. |
| `main` port drift | §1.9 fixes the names now. Shapes follow `main` except where §1.9 records a better design (`Dismiss`, `count`, `ListCore`, typed key maps, traits). A port checklist row is added to visual-testing plan §F3 for each. |
| The `Page` trait becomes the new god object | `Page` has defaults for every method except `draw` and `update`. Domain-specific pieces stay in the app's `App::on_command`. |
| Retained widgets (`&mut self` render) vs `main`'s immediate mode | Out of scope here. `Draw` takes `&mut self` now; the port splits `XState` from the props struct. Nothing in this plan blocks that. |
| Effort | 24 steps plus a theme track, each one subagent-sized. Groups C, F, G, J and K run 3–5 agents in parallel. |

**Expected deletions from `src/bin` (estimates, from the three reviews):**

| App | Shell code | Screen code | Moved into the library | Net app lines removed |
|---|---|---|---|---|
| holla | ~1,000–1,300 | ~2,650 | — | ~3,700–3,950 |
| jackin_preview | ~1,500 | ~3,000 | ~600 (FileBrowser, CommandMenu) | ~4,500 |
| tablepro | ~800–1,000 | ~1,240 | ~300 (filter editor shape) | ~2,050–2,250 |
| showcase | ~300–500 | ~1,100 | ~410 (NavList, sidebar) | ~1,400–1,600 |
| **Total** | ~3,600–4,300 | ~8,000 | ~1,300 | **~11,500–12,300** |

Inside the library, about 1,750 duplicated lines in `src/widgets` collapse into the blocks in §1.6; the expected net is about −1,000 after the new blocks are added.
