# Shared components plan

Status: plan, 2026-10-16, for `design/interaction-polish`. Owner: main session; every step is done by subagents (AGENTS.md, "Way of working").

Inputs: [`duplication-shell.md`](../audits/design-review-2026-10/duplication-shell.md), [`duplication-widgets.md`](../audits/design-review-2026-10/duplication-widgets.md) and [`duplication-screens.md`](../audits/design-review-2026-10/duplication-screens.md); [`mouse.md`](../audits/design-review-2026-10/mouse.md) and [`keyboard.md`](../audits/design-review-2026-10/keyboard.md); [`standards-interaction.md`](../audits/design-review-2026-10/standards-interaction.md) and [`standards-visual.md`](../audits/design-review-2026-10/standards-visual.md); the status sections of the other audits; and [`docs/visual-testing/plan.md`](../visual-testing/plan.md), which requires the port to `main` to match this branch 1:1. The showcase coverage track (§7) also takes [`showcase-coverage-library.md`](../audits/design-review-2026-10/showcase-coverage-library.md) and [`showcase-coverage-apps.md`](../audits/design-review-2026-10/showcase-coverage-apps.md). Finding references: **M#** is a mouse audit finding, **K#** a keyboard audit finding, **SI#** a standards-interaction verdict, **SV#** a standards-visual verdict, and **SIg#** / **SVg#** a "gaps not yet raised" item in those two reviews.

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
| `FormPanel` | `FormPanel::new(id, fields).tabs(..).columns(2).actions(..)`; `Component<Action = FormAction>`; `FormField::pair(a, b)`; disabled-when predicates | T connections (629-1027, 1264-1487) and filter editor (through `FormDialog`); S forms, settings, selectable | S18 |
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
| Select All (text: Ctrl+L only in the Emacs preset, Ctrl+A in Desktop; tables Ctrl/Cmd+A hard-coded; lists only `a` toggle-all; viewports none) | Cmd+A wherever the terminal delivers Cmd (kitty keyboard protocol, pushed on enter where supported). Text fields follow the client platform's `FieldKeys` (macOS Emacs: Ctrl+A line start, Ctrl+L select all; elsewhere Desktop: Ctrl+A). Multi-selection rows and read-only text: Cmd+A and Ctrl+A (`keymap::SELECT_ALL`); single selection ignores it. Hints name ⌘A only once Cmd delivery is confirmed | P `Theme::field_keys`, `JUNIE_TUI_KEYBOARD_PROTOCOL`; W `field_keys`, `text_keymap`; K `TextCmd`, `ListCmd`/`TableCmd`/`GridCmd`/`ViewportCmd::SelectAll` |
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
| **S8b** Hints, footer and help | `HasHints`, `Bindings`, `HintCx`, `shell::Footer`, `HelpOverlay::from_keymaps`, `HelpSection`; the **component docs generator** (§5) | 4 `draw_footer` functions, modal hints in T/H/J, 4 help renderers (about 500 lines), 16 `"Enter","Edit"` literals | K10, K23, K39, K25 (`GridAction::ReadOnly`, hints drop Edit), SI7 (hide undeliverable chords), SIg4 (kitty-detected keys feed the hints), SVg10 (left alignment, **visual**); the "no keyboard-only state" coverage test (mouse model) | Hint snapshot per widget; the coverage test walks every page's `KeyMap` | S8a, S4 |
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

### Step status

Each step's agent updates its row when the step lands: state, commits, and every deviation from the row in the table above.

| Step | State | Commits | Deviations and notes |
|---|---|---|---|
| S0 | Done, 2026-10-16 | `a3e367e8f` (expand: core types), `9e9d5b248` (`XAction` → `XCmd`), `a6343ba8e` (`XEvent` → `XAction`) | See "S0 outcome" below. |
| S1–S22, T | Not started | — | — |

**S0 outcome.** Conventions now in the library:
- `core::intent`: `Intent<'a>` (`Key`, `Binding(ActionKey)`, `Paste`, `Pointer { phase, part, pos, count, mods }`, `Wheel { axis, delta: i16, part, pos, mods }`, `FocusIn(FocusVia)`, `FocusOut(Option<WidgetId>)`, `Layer(LayerEvent)`, `Cancel`), `Phase` (no `DoubleClick`; `count` carries 1–3), `FocusVia`, `ActionKey`, `DismissReason`, `LayerEvent`. `Axis` is the existing `core::hit::Axis`.
- `core::response`: `Response<A>` (`ignored`, `consumed`, `changed`, `emit`, `repaint`, `relayout`, `no_repaint`, `is_consumed`, `is_changed`, `action`, `take_action`, `into_action`, `with_action`, `map_action`, `on_action`, `erase`; `|` and `|=` fold `Response<()>`), `Flow`, `Invalidate`, `StateFlags`. Adapters until S15: `From<Outcome> for Response<A>`, `Response::outcome()`, `Response::into_parts() -> (Outcome, Option<A>)`.
- `core::id`: `Part` (constants and numbers identical to `main`), `PartRef { part, item }`, `ItemKey { Index, Num, Text }` with `index`/`num`/`text`/`pair`, and `custom_hash16` (shared by `Part::custom` and `ActionKey::custom`).
- Names: an enum implementing `Actions` (key-map commands) is `XCmd` (`ListCmd`, `TableCmd`, `GridCmd`, `TreeCmd`, `TabCmd`, `ChipCmd`, `CodeCmd`, `DialogCmd`, `PropsCmd`, `HelpCmd`, `PanelCmd`, `InfoCmd`, `ViewportCmd`; `TextCmd` already was). A widget's output enum is `XAction` (`TableAction`, `GridAction`, `TreeAction`, `TabAction`, `ChipAction`, `PickerAction`, `SelectAction`, `MenuAction`, `MenuBarAction`, `InputAction`, `EditorAction`, `FormAction`, `RailAction`, `CompletionAction`, `PropsAction`, `ViewportAction`). `tests/naming_conventions.rs` fails if an `XEvent` enum returns to `src/widgets` or an `Actions` impl is not named `XCmd`. Other enums ending in `Action` (`ClickAction`, `BranchAction`, `EditAction`, `BlurAction`, `HeaderAction`, `UndoAction`) are neither key maps nor widget outputs and keep their names.
- No behaviour change and no drift fix (the step lists none). The showcase digest is unchanged; the pilot, smoke and `check-touched` tiers pass on each commit.

**S0 deviations from the row above:**
1. `Part`, `PartRef` and `ItemKey` live in `core::id`, as on `main`, not in `core::{intent, response}`. `StateFlags` lives in `core::response`, as on `main`.
2. `Intent` needs `ActionKey`, `LayerEvent`, `DismissReason` and `FocusVia`, so S0 adds minimal versions in `core::intent`. S5 extends these (it does not add a second `DismissReason`), and S8b builds `Bindings` on `ActionKey`. The existing `ui::modal::Dismissal` (`Keep`/`Close`/`None`) is a different concept and is left to S5.
3. `StateFlags` is hand-written over `u16` instead of using the `bitflags` crate, so `Cargo.toml` and the generated CI stay untouched. The bit layout is `main`'s, including the three flags the plan's list omits (`FOCUS_VISIBLE`, `WARNING`, `BUSY`).
4. `Response` has no `id` or `state` fields (the plan's shape). `main`'s `Response::action(a)` constructor is `emit(a)` here, and `action()` is the borrow (`main`: `action_ref()`). Map both when porting.
5. holla's private alternatives-menu command enum `MenuAction` became `AltCmd` to free the name for `widgets::menu::MenuAction`.
6. App-local enums (tablepro `ConnEvent`, jackin `LaunchEvent`, showcase `PageEvent`) are not widget output and keep their names. `PageEvent` goes in S7 and `ConnEvent` goes with the tablepro page split (S7).

**Notes for S1 and S2 (they run in parallel; their files are disjoint):**
- S1 builds `HitRegistry::register_part(owner, PartRef, area)` and `resolve(pos) -> Option<Hit { owner, part }>` on `core::id::{Part, PartRef, ItemKey}`. Do not add part types elsewhere. Use `ItemKey::index(i)` for positional rows today; jackin `manager.rs:1625` gets a real `ItemKey` (`ItemKey::text` or `ItemKey::pair`). `Intent::pointer(phase, part, pos)` is the one-click constructor for hit tests. S1 owns `src/core/hit.rs` and the widgets' `render` registration lines.
- S2 owns the CLI and `Application` blocks in each `src/bin/*/main.rs`, the new `cli`, `motion`, `clock` and `shell::StatusLine` modules, `widgets::too_small`, and `ui::text` helpers. It should not touch `src/core/hit.rs` or widget render code, so it does not collide with S1. `StatusLine::set(msg, Tone)` stays independent of `Response`.
- Both: return `Response<A>` from any new handler (not `Outcome`), and use `Response::outcome()` or `From<Outcome>` at the boundary with code not yet migrated. Name any new key-map enum `XCmd` and any new widget output enum `XAction`; `tests/naming_conventions.rs` enforces both.
- Origin moves every few minutes. A large mechanical rename conflicts on rebase, so S0 regenerated its rename commits from a script on the fresh origin tip for each push attempt, instead of rebasing.

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

## 7. Showcase coverage track (SC)

Status: plan, 2026-10-16. Owner: main session; every step is done by subagents. Inputs: [`showcase-coverage-library.md`](../audits/design-review-2026-10/showcase-coverage-library.md) (244 rows: 82 ✅, 69 ⚠️, 93 ❌) and [`showcase-coverage-apps.md`](../audits/design-review-2026-10/showcase-coverage-apps.md) (92 app patterns: 10 ✅, 82 ⚠️ or ❌; many still app-local).

**The problem.** The showcase shows every widget family, but only about a third of the library's capabilities. Ten whole widgets are never shown (`Breadcrumb`, `SidebarRail`, `ButtonRow`, `ChoiceDialog`, `FormDialog`, `InfoDialog`, `HelpOverlay`, `KeyReference`, `PropsList`, stacked modals through `ModalStack`). Most builder methods and policy variants are never called, and no option can be flipped while the showcase runs. The apps meanwhile use 82 patterns that the showcase does not show, and many of those live only in app code. This track makes the showcase the complete catalogue of the library and adds a gate so that a capability can never again ship without being shown.

### 7.1 Principles

| # | Principle | What it means |
|---|---|---|
| SCP1 | The showcase is the complete catalogue | Every public widget, component and block, every variant, every state and every configuration option (builder method, policy type, enum variant, theme token, shell setting) is demonstrated in the showcase. An item that cannot be shown (runtime plumbing, for example) is listed in the allowlist with a reason. |
| SCP2 | Each widget page is Playground + Options + States | **Playground**: the live widget with realistic demo data. **Options**: one live control per option of the widget (`Toggle` for a bool, `Select` or a segmented control for an enum, a preset `Select` for a number), declared once in the page's option table. **States**: a static grid, rows = states (rest, hover, pressed, focused, focused + hover, selected, disabled, busy, error, empty, loading, overflow, narrow), columns = variants, drawn through forced `StateFlags`/`VisualState`, so it is exact. The three sections are a `Tabs` strip at the top of the page. |
| SCP3 | The Inspector's Options pane drives the same knobs | `i` opens "State \| Options". The Options tab lists the focused page's option table plus the shell-wide options, and flipping one rebuilds the widget through **the same builder call** an app would make. `--opt key=value` sets any option at launch, so captures and tests reach every option without key presses. There is one option declaration per knob; the page section, the Inspector tab and `--opt` all read it. |
| SCP4 | Screens demos compose the library | A "Screens" page recreates an app screen (query workbench, connection manager, ...) from library components only. A screen that needs a piece the library lacks waits for that piece's extraction step; it never draws it locally. |
| SCP5 | The showcase is a thin consumer too | No app-local code: `src/bin/showcase` adds page wiring, demo data and option tables only. Any drawing, interaction, layout primitive or state logic a page needs lives in `src/core`, `src/ui` or `src/widgets`. If a page needs it and the library lacks it, the step stops and adds it to the library (with its own test) or records an extraction step. |
| SCP6 | Apps are thin consumers of what the showcase shows | An app-specific UI pattern is extracted to the library and demonstrated in the showcase before (or in the same change as) the apps adopt it. The extraction step deletes the app copies (P1). |
| SCP7 | Tree IA rules | The sidebar stays at most 2 levels deep (Overview at the top, then groups of leaf pages). No group holds a leaf with the group's own name. Every page has one stable `--page` slug (its label, letters and digits only, lower case, as `PageId::from_name` normalises it). A renamed or moved page keeps its old slug as an entry in `PAGE_ALIASES`. A new label's slug must not equal an existing alias (for example, `choices` already resolves to Cell editors, so the new checkbox and radio page is "Checks & radios", not "Choices"). |
| SCP8 | The gate decides | The coverage gate (SC0) fails when a public capability is neither referenced by a showcase option, state-grid row or demo, nor listed in the allowlist with a reason. Pending gaps are allowlisted against the SC step that closes them; each step deletes its entries, and an entry for an item that is now covered fails the gate as stale. |

### 7.2 Information architecture

The groups stay as they are. New pages are added as leaves; slugs are fixed now, so parallel steps do not collide. SC2 registers every planned page up front (its `PageId`, label, group and slug) as a stub that shows an `EmptyState` naming the SC step that fills it. Each page step then owns only its page file.

| Group | Existing leaves | New leaves (`--page` slug; extra aliases) | Filled by |
|---|---|---|---|
| (top) | Overview | — | SC3 |
| App shell | Chrome | Status & hints (`statushints`; `hints`) · Breadcrumbs & rails (`breadcrumbsrails`; `breadcrumbs`, `rails`) · Help & keys (`helpkeys`; `help`) · Quit & leave (`quitleave`; `quit`, `leave`) · Focus & navigation (`focusnavigation`; `focus`) | SC4, SC5, SC6, SC28 |
| Inputs & forms | Buttons, Inputs, Text areas, Chips & selects, Forms | Checks & radios (`checksradios`; `checks`, `radios`) · Toolbars (`toolbars`) | SC9, SC28 |
| Tables | Basic table, Editable, Data grid, Selection, Cell editors | Columns (`columns`) | SC11 |
| Navigation | Lists, Trees, Sidebars | Tabs (`tabs`) · Menus (`menus`) | SC14, SC16 |
| Overlays | Dialogs, Pickers | Dialog kinds (`dialogkinds`) · Popups & tooltips (`popupstooltips`; `popups`, `tooltips`) · File browser (`filebrowser`) | SC18, SC19, SC41 |
| Feedback | Progress | Empty & error (`emptyerror`; `empty`) · Meters (`meters`) · Notices (`notices`) | SC21, SC36 |
| Layout | Panels, Scrolling | Splits & stacks (`splitsstacks`; `splits`) · Master-detail (`masterdetail`) · Pane tree (`panetree`) | SC23, SC38, SC45 |
| Text & code | Code editor, Diff, Terminal | Text utilities (`textutilities`; `text`) | SC26 |
| Screens | Settings, Task runner | Query workbench · Connections · Finder · Disk cleanup · Activity · Workspaces · Accounts & usage · Launch cockpit · Read-only report (slugs = labels) | SC46–SC56 |

### 7.3 Extraction steps added by this track

The apps inventory names patterns that no S-step extracts yet. They become S-steps in §4 with the same rules (expand, migrate, contract; every app copy deleted; the gate G), and each carries its showcase demo (the SC step shown) in the same change.

| Step | Adds (expand) | Migrates; deletes in all apps (contract) | Deps | Demo |
|---|---|---|---|---|
| **S23** Small app patterns | `Tabs` preview flag and pinned (non-closable, `▪` prefix) tabs; `FooterNote` (priority-dropped parts) or `segments` reuse; `ActivityLine` (spinner on the shell `Clock`, elapsed, "Esc cancels", outcome) | T workbench preview tabs, result-tab pins, grid footer note, running status line | S2, S17 | SC42 |
| **S24** Row-model extensions | `TreeView` tri-state check column and trailing action row ("+ New …"); `DataTable` group heading rows, match ranges (bold matched characters), never-dropped columns, row marks `+ • −`, group fold rows; a `StepRail` table variant (plan outline) | H cleanup checklist, H finder grouped table, J config change tables, H plan outline, J manager/accounts action rows | S16 | SC43 |
| **S25** `PickerChain` | Chained `Picker` with a crumb scope, Backspace back, `PickerStatus` per level, live suggestions | J OpFlow (`modals.rs:551-948`), H go-to-path | S12, S22 | SC44 |
| **S26** `PaneTree`, zoom, cross-fade | n-ary split tree with focus and hover borders, zoom and resize chords; H and J zoom moved onto `Split::toggle_max`/`Maximized`; `ui::fade::dim_buffer` cross-fade under `Motion` | J capsule panes, H plan zoom, J handoff cross-fade | S2, S19 | SC45 |

### 7.4 Acceptance for every SC step

Beyond the gate G (§4):
1. **Harness tests.** Showcase app tests through the `Harness` (`src/bin/showcase/app_tests*.rs`) for every option and state the step adds: setting the option through the Inspector and through `--opt key=value` changes the rendered cells, hit regions or focus exactly as the builder documents; each States-grid row renders its state. A behaviour shown for the first time that turns out wrong is a library bug: fix it in the library with a failing-first regression test (AGENTS.md, "Proof").
2. **Intended digest regeneration.** `UPDATE_BASELINE=1 cargo nextest run --bin showcase showcase_visual_baseline` rewrites `tests/showcase_baseline.txt`. The commit message lists every page whose hash changed and why; any page outside the step's list must be unchanged. New pages and sections add rows.
3. **New pending visual scenarios.** `tests/visual_baseline/scenarios/showcase.rs` gains `.pending()` rows: one `SURFACES` root per new page and section, an `ELEMENTS` row per new state, an `OVERLAYS` row per new overlay with both dismissals, and `overlay_sites` entries for new overlay openers. Approval happens later, on purpose (`cargo xtask visual accept`); `visual-baseline` is never touched.
4. **Coverage gate.** The step deletes its own allowlist entries; the gate passes with the remaining ones.
5. **No app-local code** (SCP5). Review checks that the `src/bin/showcase` diff is wiring, demo data and option tables only.
6. **Docs.** The step's row in §7.8 is updated; a capability left out records its reason in the allowlist and here.

### 7.5 Steps

Files named are the files the step owns; steps in the same parallel group own disjoint files. "After Sx" means the capability is extracted or changed in Sx first. From now on (AGENTS.md, "Showcase") an S-step's own change must include its demo, so a follow-on step (SC29–SC45) either lands inside its S-step or is run by a paired agent in the same group, before the S-step is marked done.

**Foundation**

| Step | Delivers | Files owned | Deps | Acceptance beyond §7.4 |
|---|---|---|---|---|
| **SC0** Coverage gate | (1) A capability registry the library exposes: `junie_tui::catalog::{Capability, CapKind, CAPABILITIES}`, one entry per public widget, builder method, policy type and enum variant, state and theme token, with a stable id (`widgets::tabs::Tabs::overflow_menu`, `widgets::tabs::TabRule::Accent`). It is the seed of §5's `WidgetDoc` knobs; S8b builds on it rather than adding a second list. (2) A source scan in the test that lists `pub struct`, `pub enum` and their variants, and `pub fn`/`pub const fn` builders taking `self` and returning `Self`, under `src/{core,ui,widgets,theme,runtime}`, and fails when the registry misses one, so the registry cannot drift. (3) The showcase declarations: `src/bin/showcase/coverage.rs` lists, per capability id, how it is shown (`Option(key)`, `StateRow(name)` or `Demo(page, label)`); the option tables (SC1) and States grids (SC2) generate their entries from the same declarations, so a declaration cannot exist without the thing it claims. (4) The allowlist `tests/showcase_coverage_allow.txt`: one line per id, `id \| reason \| SCn` for a pending gap or `id \| reason \| permanent` for an intentional exclusion. (5) The gate test `src/bin/showcase/app_tests_catalog.rs` (next to `app_tests_coverage.rs`, so it can drive the `Harness`): fails on a capability neither declared nor allowlisted, on a stale allowlist entry (now covered, or an unknown id), on a pending entry without an SC step, and on a declared `Option` whose `--opt` value does not change the page's rendering. It prints the remaining gaps per SC step. | `src/catalog.rs` (new), `src/lib.rs` (one `pub mod`), `src/bin/showcase/coverage.rs` (new), `src/bin/showcase/app_tests_catalog.rs` (new), `tests/showcase_coverage_allow.txt` (new) | — (S0 done) | Lands green: every current gap is allowlisted against the SC step that §7.7 maps it to, and only the runtime plumbing rows are `permanent` (`TerminalSession`, job control, `SizeWatch`, `InputPump`, drains), plus jackin's intro and outro animations (product owner decision 2026-10-16: they stay in jackin). A deliberately removed declaration makes the test fail (shown in the commit). Digest unchanged. |
| **SC1** Inspector "State \| Options" | `i` opens a right pane built from library widgets (`Tabs`, `PropsList`, `Toggle`, `Select`, `RadioGroup`). **State tab**: today's diagnostics plus an `Intent`/`Response` event log (last 50), the active `KeyboardProtocol`, `PointerShape`, `ShapeSupport`, and size and input counters. **Options tab**: the page's option table, then the shell-wide section: colour level (TrueColor/256/16/Mono, live `Theme::for_level`), `LevelOverrides`, `Glyphs` Unicode/ASCII, `Strings` default/override, `FocusVisible`, `TrackClick`, `HScroll`, `Reveal` mode and delay, `Tooltips`, every `PointerConfig` field (focus_on, double_click_ms/slop, wheel_lines, hover_refresh_after_scroll, flash_ms, secondary_outside, end_edit_on_dead_space, autoscroll, track_repeat, pointer_shapes, caret_shape, click_skips_animation), `NavKeys` (vim, page, home/end), `FieldKeys`, `ClientPlatform`, `EditEntry`, `FocusSelect`, `BlurAction`, `QuitKeys`, `InputConfig`, `FocusFallback`, `Coalesce`, and motion. Below 100 columns the pane is an overlay (`Frame` placement Right through `RenderCtx::overlay`, light dismiss, Esc closes) instead of disappearing. `--opt key=value` (repeatable; `shell.<key>` or `<page>.<key>`; an unknown key or value exits non-zero and lists the valid ones). The `--opt` flag belongs to the showcase's own arguments, not to S2's `CommonArgs`. **Motion**: `--motion reduced` currently behaves as full (`main.rs:124`); S2 owns that fix. Until S2 lands, the motion option offers only Full and Paused (hints never advertise what does not work), and SC30 adds Reduced. | `src/bin/showcase/app.rs` (Inspector only), `src/bin/showcase/options.rs` (new), `src/bin/showcase/main.rs` (one `--opt` line) | SC0; rebase over S2 if it lands first (S2 owns the CLI block in `main.rs`) | Harness: every shell option flips through the pane and through `--opt`; the pane works at 80×24 as an overlay; a colour-level flip re-renders at that level. Digest rows for the pane open and closed. |
| **SC2** Page template, States grid, IA | Library `ui::specimen::{Specimen, StateGrid}`: `Specimen::specimen(&mut self, area, &mut RenderCtx, StateFlags)` draws a widget in a forced state; `StateGrid` lays out rows = states and columns = variants with labels, a 24-column narrow slot, and "Compare levels" (the grid at all 4 colour levels side by side). Implemented for `Button` and `TextInput`, porting the hand-made matrices at `buttons.rs:102` and `inputs.rs:145` onto it. The page template (Playground \| Options \| States as a `Tabs` strip) in `pages/mod.rs`. Every §7.2 page registered as a stub (`PageId`, label, group, slug, aliases) showing an `EmptyState` that names its SC step. An IA test: at most 2 levels, no group with a same-named leaf, every slug unique and never equal to another page's alias, and a golden list `tests/showcase_slugs.txt` of slugs and aliases that may only grow. | `src/ui/specimen.rs` (new), `src/bin/showcase/pages/mod.rs`, `pages/buttons.rs` and `pages/inputs.rs` (States sections only), `src/bin/showcase/app.rs` (`PageId`, `NAV_ENTRIES`, `PAGE_ALIASES` only), the stub page files, `tests/showcase_slugs.txt` (new) | SC0 | `VisualState` row closed for Button and TextInput; the two ported matrices are pixel-equal to before (digest unchanged for those sections); stub pages add digest rows. |

**Pages (parallel; group SC-C)**

| Step | Delivers (inventory rows closed) | Files owned | Deps |
|---|---|---|---|
| **SC3** Overview: theme | Token tables for every `Theme` token (field_hover, highlight*, selection_bg, success, ...); `Plane` × `Kind` ladder explorer (`Ladder`, `with_ladder`); `contrast_ratio`/`legible` table per colour level; `StateModifiers`; `CursorStyle` Fill/Bar, `Interactive`, `CursorRow`; `BadgeKind` all kinds; `ButtonKind` Toggle row; `Spacing`/`Insets`; `Glyphs` Unicode vs ASCII and `Strings` default vs override side by side; `LevelOverrides`/`for_level_with`; `Tone`/`SyntaxTone`. Each theme-track (T) commit updates these tables. | `pages/overview.rs` | SC1, SC2 |
| **SC4** Chrome + Status & hints | `MenuBar` OpenOn, Overflow Hide/Chevron (narrow), hover_switch, press_drag, hide_owner_focus, nav, `MenuBarAction::Closed`; per-route menus; brand item opens About (`InfoDialog`); `Lockup` compact; identity strip of `Segment`s (clickable, env tones `◆ production`/`◇ staging`, spinner, `• N pending`, priority drop); app menu anchored at a segment (`ContextMenu::at().title()`); `StatusItem` busy, meter, tooltip, `Emphasis::Plain`, centre group. New page: `HintBar` layer precedence, `HintAlign`/`HintBarStyle` (centre vs left), `keyhint::render_toned/aligned/styled`, `HintPriority`, `toggle_hint`, EDIT badge, reveal-key advert. | `pages/chrome.rs`, `pages/status_hints.rs` | SC1, SC2 |
| **SC5** Breadcrumbs & rails | `SidebarRail` now: framed panel, `‹` collapse in the title, 2-column rail with `›` and a vertical label, click / Ctrl+B / `0` to expand, `RailAction`, narrow drawer; on the Sidebars page beside the hand-made `NavList` (which S17 moves into the library). `Breadcrumb` static and clickable, middle collapse to `…`, last item middle-truncated; the showcase header crumb uses `Breadcrumb` instead of hand drawing (`app.rs` ~1480). | `pages/breadcrumbs_rails.rs`, `pages/sidebars.rs` (rail section only), `app.rs` (header crumb only) | SC1, SC2; not concurrently with SC6 (both touch `app.rs`) |
| **SC6** Help & keys + Quit & leave | `HelpOverlay` (sections, multi-column, scroll, refuses over another modal) and `KeyReference` (sections, `key_max_share`, hanging wrap); the showcase `?` opens `HelpOverlay` instead of `Dialog::confirm` (`app.rs:986`), with hand-listed sections until S8b generates them. `QuitKeys` (ctrl_c, ctrl_q, letter, esc_quits), `LeaveGuard` with `Stakes` Clean/Unsaved/Running, `unsaved_dialog`/`discard_dialog`/`running_dialog`, `KeyContext`/`LeaveKey` (Ctrl+C copies when there is a selection), letters never leave a field. | `pages/help_keys.rs`, `pages/quit_leave.rs`, `app.rs` (`?` handler only) | SC1, SC2; after SC5 |
| **SC7** Buttons | `Button` shortcut, focusable(false), focus_visible, mnemonic; busy and Toggle rows in the States grid; toolbar row (not focusable, shortcut labels, disabled while running, Cancel while busy); `ButtonRow` align End, gap, overflow More/Wrap; `row_layout_right`; `RowLayout.more/hidden`; "Next" row of subtle buttons with disabled ones. | `pages/buttons.rs` | SC2 |
| **SC8** Inputs + Text areas | `TextInput` required_message, wrap_label, blur_action, focus_select, field_keys, text_keymap, pointer_select, label_hit, reveal_tail; `EditEntry` Explicit; `FocusSelect` CaretEnd/CaretStart; `FieldKeys` Emacs/Desktop; `TextCmd` custom key map and a bound-command list; `UndoStack` coalescing explainer; clipboard fallback status; `PointerSelect` flags; `TextArea` focus_select, esc_reverts, field_keys, text_keymap, nav, pointer_select, blur_action, label_hit, horizontal. | `pages/inputs.rs`, `pages/textareas.rs` | SC2 |
| **SC9** Checks & radios | `Checkbox` checked/disabled/mixed, `CheckHit` Label vs Row; `RadioGroup` disabled, `nav`; `Toggle` states; States grids for all three. | `pages/checks_radios.rs` | SC2 |
| **SC10** Chips & selects | `ChipBar` lead ("match all ▾"), enabled toggle, `Chip` error, removable(false) stream chips with 1-9 toggles, overflow_marker, overflow_menu, close_target_w, `ChipCmd`, `nav`/`keymap`; `chips::overflow_run` `+N` row; `Segment` clickable; `Select` arrows, wheel mode, max_rows, placement, popup_width, popup_fit, cursor_style, close_on_blur, hover_moves_cursor, press_drag, label_hit, track_click, wrap, type_ahead, nav. | `pages/chips.rs` | SC2 |
| **SC11** Columns + Basic table | `ui::columns::Col` (content/fixed/flex/max/droppable) and `solve` with droppable columns; `ColumnSizing` (resizable, seam_w, min_width) and seams on `DataTable`; the library `header_menu`/`HeaderAction` (Autofit, ResetWidths)/`HeaderLabels` replacing the hand-built menu at `tables.rs:313`; `render_seam`; `SelectionMode::None`. | `pages/columns.rs`, `pages/tables.rs` | SC2 |
| **SC12** Data grid | Every grid knob (editor_config, type_ahead_column, overflow_badges, bar_labels, duplicate_keys, width_sample_rows, resizable_columns, seam_hit_w, track_click, nav, keymap, row_numbers, header_labels, filtered_cols, blur_action); `RowTotal::Unknown`; FetchMore row cap; error row and cell errors; whole-grid read-only with `read_only_reason` variants (view, no PK, query result) and the `ReadOnly` status; FK references followed; filter on cell opening a filter chip; grid `header_menu`; `GridCmd` key reference; Save review `Dialog::facts` with a typed table-name token; error empty. | `pages/grid.rs` | SC2 |
| **SC13** Editable, Selection, Cell editors | `DataTable` esc_clears, overflow_marks, type_ahead_column, editor_config, resizable_columns, seam_hit_w, track_click, nav, keymap, `empty(EmptyState)`, header_labels, blur_action; `TableCmd` override; `EditorConfig` (all 10 fields); `Affordance` Always/Never; `ChoiceEditor`/`ChoiceResult` standalone. | `pages/editable.rs`, `pages/selectable.rs`, `pages/celleditors.rs` | SC2 |
| **SC14** Tabs | `TabItem` prefix icons, suffix, closable `×`, dirty, busy, error; all four `TabRule`s (accent vs neutral under a layer); allow_new `+`, new_key, close_keys, middle_click_closes, overflow_menu `+N`, `‹ ›` scroll, wheel, close_target_w, wrap, nav, keymap, digit jump; `TabAction` Close/New; `TabCmd`; nested strips (main › segmented › sub-tabs with 1-6 jump); close-dirty-tab confirm. | `pages/tabs.rs` | SC2 |
| **SC15** Lists & trees | `TreeNode` lazy (delayed `set_children`), glyph, note, busy, meta; `TreeAction::Expand`; `BranchAction`, row_click_toggles, indent, divider, alt_click_recursive, reserve_right, show_meta, track_click, nav, wrap, `TreeCmd`; `ListBox` `ClickAction`, skip_disabled, wrap, type_ahead (and the `TypeAhead` timeout), reveal, `empty()`, scrollbar_gap, track_click, nav, keymap, `ListCmd`. | `pages/lists.rs`, `pages/trees.rs` | SC2 |
| **SC16** Menus | `ContextMenu` placement, anchor/anchor_control, fit, min_width, max_rows, edge_margin, release_selects, hide_owner_focus, nav, wrap, type_ahead; `MenuItem` reason, danger, separators, danger_if/separator_if; press-drag-release; right-click on a backdrop reopening the menu beneath; `ContextConfig::moves_cursor(false)`; `ContextKeys` extra key. | `pages/menus.rs`, `pages/row_menu.rs` | SC2 |
| **SC17** Dialogs | `Dialog::facts` variants: read-only viewer (Close only, code/JSON), confirm with facts and code, typed token (`AckInput`, `AckEnter` Submit/NextField; initial focus on Cancel), multi-action destructive (Retry / Alternative / Cancel with `cancel_label`), gate "1 of 2 / 2 of 2" titles; long body that scrolls; accelerators, mnemonics, guard_dirty_prompt, confirm_index, button_row, body_scroll, max_height, outer_margin, backdrop, placement, anchor under a control, nav, keymap, edit_entry, width. | `pages/dialogs.rs` | SC2 |
| **SC18** Dialog kinds | `FormDialog` with every `FormField` kind (input, select, radio, chooser, note, check, hidden), masked with `reveal_tail(4)`, progressive disclosure, Validate action, keep_open_on_save, a live preview slot; `InfoDialog` copyable rows (`y`), `.error()`, "read-only" meta, detail block, actions; `ChoiceDialog` radio options, option_tones, line, stepper, buttons. | `pages/dialog_kinds.rs` | SC2 |
| **SC19** Popups & tooltips | `popup::Placement` ×5, `Overflow` Shrink/Slide, `VAlign`, `Place`/`place_with`; `Dismiss` (escape off, outside_click, pass_through, close_button), `RefuseFeedback` Flash/None; `ctx::Backdrop` keep_bottom/keep_top; `modal::Frame` placement, anchor, fit, margin, min_size, dim; `refuse_outside_click`, close button `×`, `border_style`; `Tip` with shortcut; `Tooltips` enabled, delay_ms, max_width; `Reveal` every knob per widget and `RevealMode` Tooltip vs InlineWrap. | `pages/popups_tooltips.rs` | SC2 |
| **SC20** Pickers | `PickerStatus` Loading/Error (`r` retry)/Ready; live suggestions as you type; disabled rows with a reason (command palette); max_rows, fit, placement, anchor, backdrop, cursor_style, delete_is_secondary, nav, `empty()`; Delete as the secondary action on a tab list; live-refreshing switcher with an "N live" scope. | `pages/pickers.rs` | SC2 |
| **SC21** Progress + Empty & error | `BarLayout`/`render_bar_with`; `MeterTone::Level`, Refreshing and Stale tones; aggregate header ("k of n done · failed", only the failed clause toned); `EmptyState::error()`, `actions()`/`EmptyAction`/`render_ctx`; a "Read-only" demo (grid reason, menu-item reasons, "· read-only" panel meta, status on an attempted edit). | `pages/progress.rs`, `pages/empty_error.rs` | SC2 |
| **SC22** Panels + Scrolling | `PanelFocus` Title/Bar, focus_mark, meta_short, badge, bg_override, title_min; `ScrollPanel` hscroll, horizontal, track_click, nav, keymap, `PanelCmd`, line styler with `position_label` meta; `ScrollbarVisibility` Always/Never; `FollowCursor::Always`; `TrackClick` Jump vs Page per widget; `HScroll` off, no bar, step; `scrollbar::render_vertical` with `fade::scroll_edges` on a hand list, track repeat, drag autoscroll. | `pages/panels.rs`, `pages/scrolling.rs` | SC2 |
| **SC23** Splits & stacks | `Split` Horizontal/Vertical, percent, min sizes, `Maximized` First/Second and `toggle_max` (zoom `z`, Esc restores, chrome hides); `Splitter` drag, `SeamDoubleClick` Reset/Ignore, hit_pad, clamped default until dragged, three panes; `Stack` `Overflow` Scroll/Clip, min_child_rows, scrollbar_gap, track_click. | `pages/splits_stacks.rs` | SC2 |
| **SC24** Code editor | `CodeEditor` read_only, tab_leaves, esc_reverts, edit_entry, field_keys, text_keymap, nav, keymap, scrollbar_gap, track_click, horizontal, indent; a discoverable find demo; `CodeCmd` override; running-statement highlight (`set_running`); `()` insertion on function completion; `Completion` max_rows, popup_width, fit, placement, cursor_style, nav. | `pages/editor.rs` | SC2 |
| **SC25** Diff + Terminal | `DiffStatus` Added/Deleted/Renamed; no-file and narrow states; `unified_lines`/`review_lines` (`_with`); multi-file changes tree with the diff following the cursor, stacked when narrow. `TextViewport` caret, wrap, hscroll, scrollbar_gap, track_click, nav, keymap, with_lines, `ViewportCmd`, `SelectionChanged`, `WorkCounters`, `Mark`/`CellPos`, retention-drop meta, service prefix column, ANSI spans, horizontal wheel; `Span` italic/underline/reversed; `StepRail` selectable, numbered, track_click, nav and an "N/11" counter segment. | `pages/diff.rs`, `pages/terminal.rs` | SC2 |
| **SC26** Text utilities | `ui::text` truncate_middle, thousands, fit_right, slice_cells, wider_than, width_capped, find_ranges, `WrapOpts`/`wrap_with` (keep_indent, hang_at_gap); `props::Sheet`, `measure(_with)`, `render_with`, copyable `Prop`; `PropsList` with `PropsAction`/`PropsCmd`. | `pages/text_utilities.rs` | SC2 |
| **SC27** Settings + Task runner | Task runner `leave_stakes` Running (quit while running); Settings tabs with dirty markers. | `pages/settings.rs`, `pages/taskrunner.rs` | SC2, SC14 |
| **SC28** Focus & navigation + Toolbars | Strategy B nav policies (`FocusPolicy`, `AdjacentMove`, `AdjacentKeys`, `Scope`, `Bounds`) applied to Tabs, Tree, List, Select, Picker, Menu, Dialog and Toolbar; the `Toolbar` widget; `FocusFallback` Nearest/First; `request_focus_next`/`cancel_focus_request`; `InputConfig` (shift_wheel_horizontal, ctrl_click_secondary, legacy_ctrl_punctuation); `KeyboardProtocol` Auto/On/Off with the kitty flags and `super_delivered`; per-widget `nav()` (vim, page, home/end off); `.keymap()` override examples (`KeyMap`, `label_for`, `hint_key`, `select_all_bindings`). | `pages/focus_navigation.rs`, `pages/toolbars.rs` | SC2 |

**Follow-ons gated on S-steps (group SC-F; each lands with its S-step)**

| Step | Delivers | After | Files owned |
|---|---|---|---|
| **SC29** | `ScrollPane` bars on popups (M17), `scrollbar::render_with_fade` on Scrolling, `TrackClick::Page` as the default; `Cursor`/`RowSource` defaults (`NavKeys::NO_VIM`, type-ahead on) shown as Options | S3, S4 | `pages/scrolling.rs`, `options.rs` |
| **SC30** | `Motion` Full/Reduced/Paused for real (the motion option gains Reduced; a test shows Reduced differs from Full); `TooSmall` demo on Status & hints; `StatusLine` tone, 5 s TTL and frozen under Paused; `CommonArgs`; new `ui::text` helpers (plural, truncate_sep, duration_ms, bytes, crumb_fit, center_x) on Text utilities | S2 | `pages/status_hints.rs`, `pages/text_utilities.rs`, `options.rs` |
| **SC31** | Stacked modals through `ShellLayers`/`ModalStack` on Dialog kinds; `Dismiss` focus_out and refuse_feedback; `Frame::close_button`; `DismissReason` in the State log | S5, S7 | `pages/dialog_kinds.rs`, `pages/popups_tooltips.rs` |
| **SC32** | `EditField`, `Validate`, `ValidateWhen`, `FieldError` as Options on Inputs and Dialog kinds; `OptionList` `WheelMode`, `SelectArrows`, `Item` reason on Chips & selects, Pickers, Menus and Code editor (Completion) | S11, S12 | the pages named |
| **SC33** | `HelpOverlay::from_keymaps` replaces the hand-listed help (H's `TextModal` merged); `Footer` layer precedence; `HasHints` per widget in the State tab; generated `KeyReference` (including `GridCmd`); `ClickPolicy`/`Activate`/`CheckClick`/`SelectGesture` Options on Lists, Tables, Trees, Checks; `RowUi`/`RowLook`/`CursorStyle::Bar` and the reveal peek line. May split per S-step | S8b, S9, S10 | `pages/help_keys.rs`, `app.rs` (`?`), the row pages |
| **SC34** | `ContentState` Loading/Error/Empty in the States grid of every list-like widget; the standard Edit context menu on text widgets; the showcase shell on `Shell`/`Router`/`Chrome` with a clickable crumb and a page stack (Esc pops); truthful clipboard results; Chrome toolbar | S7, S13, S14 | `pages/breadcrumbs_rails.rs`, `pages/chrome.rs`, `app.rs` |
| **SC35** | `ListItem::{header, status}`, `TreeNode::{meta_tone, trailing, label_tone}`, `TreeView::columns` tree-table (EXPLAIN), `Column::marker`, `RowMark`; history list with status tones; files list as a `DataTable` | S16 | `pages/lists.rs`, `pages/trees.rs`, `pages/tables.rs` |
| **SC36** | `PageHeader`/`Heading`, `ActionRow` (with a message), `DetailCard`, library `NavList` (Sidebars switches to it), `Notice::error`, `MeterList` (new Meters page), `Tabs::{layout, passive}`, `TextInput::readout`, `Item` builder; Notices page | S17 | `pages/meters.rs`, `pages/notices.rs`, `pages/sidebars.rs`, `pages/buttons.rs`, `pages/inputs.rs` |
| **SC37** | `FormPanel` on Forms (two columns scrolling by field, tabs with an error marker, Host/Port 70/30, dependent disable, masked password, validation jump, Ctrl+S, inline test result); filter editor through `FormDialog` with a live preview | S18 | `pages/forms.rs`, `pages/dialog_kinds.rs` |
| **SC38** | Master-detail page (wide split, narrow summary, drawer, hidden pane as a focus stop); `SplitKeys` on Splits & stacks | S19 | `pages/master_detail.rs`, `pages/splits_stacks.rs` |
| **SC39** | `EditorShell` (dirty and error tabs, `[`/`]`, Save disabled when clean, busy save, "• N changes") | S20 | `pages/settings.rs` |
| **SC40** | `FilterTree`/`FilterList` on Trees and Lists, and the Sidebars filter | S21 | `pages/trees.rs`, `pages/lists.rs`, `pages/sidebars.rs` |
| **SC41** | `CommandMenu` with key replay on Menus; File browser page; `StepLine`/`Wizard` on Terminal; `TextViewport::find` ("find “q” · i of n") on Terminal and Code editor | S22 | `pages/menus.rs`, `pages/file_browser.rs`, `pages/terminal.rs`, `pages/editor.rs` |
| **SC42** | Preview and pinned tabs; `FooterNote` under the grid; `ActivityLine` on Progress | S23 | `pages/tabs.rs`, `pages/grid.rs`, `pages/progress.rs` |
| **SC43** | Tri-state checklist tree, trailing action row; `DataTable` group rows, match ranges, row marks, change table; plan outline table | S24 | `pages/trees.rs`, `pages/tables.rs`, `pages/taskrunner.rs` |
| **SC44** | `PickerChain` (crumb scope, Backspace back, per-level status) | S25 | `pages/pickers.rs` |
| **SC45** | Pane tree page (nested splits, borders, zoom, resize chords); cross-fade under `Motion` | S26 | `pages/pane_tree.rs`, `pages/overview.rs` |

**Screens (group SC-S; composed from the library only, parallel among themselves)**

| Step | Screen | Deps |
|---|---|---|
| **SC46** | Query workbench: explorer sidebar (rail, filter, lazy tree), main tabs with icons, `×`, `+`, overflow and preview; query tab (toolbar, editor/results split, zoom, result sub-tabs, `ActivityLine`, EXPLAIN tree-table, error card); table tab (Data/Structure strip, sub-tabs, DDL card, filter chips, footer note, pending bar, Save review with typed token); tab list, Open Quickly, Safe Mode picker and env-toned gate dialog; Esc ladder | S8a, S16, S17, S19, S21, S23; SC12, SC14, SC24 |
| **SC47** | Connections: grouped tree with env glyphs and filter, empty state with "New connection", detail card with a Warning row, Connect/Edit/Duplicate/Delete, connect spinner → failure block → Reconnect, the `FormPanel` form | S17, S18, S19, S21 |
| **SC48** | Finder: typing-hot query with a scope readout, grouped ranked table, preview card or one-line narrow summary, alternatives menu, "Why" explanation | S11, S17, S19, S24 |
| **SC49** | Disk cleanup: usage tree with sizes and scanning nodes, filesystem meters, scan progress, tri-state checklist with a mode/danger summary, Gate 1 review → Gate 2 typed phrase | S16, S17, S24 |
| **SC50** | Plan runner (extends Task runner): outline review → run with a live state column, aggregate progress, step output with zoom, outcome line and Next row | S23, S24, S26 |
| **SC51** | Activity: header with state clauses, stream chips, live log with find, attach/detach and stdin input mode (behaviour app-level, shown with library pieces), status-bar contributions | S2, S14, S17, S22 |
| **SC52** | Workspaces: tree + seam + detail + roster at ≥150 columns, narrow drawer, per-row spinners, purge typed confirm | S16, S19, S26 |
| **SC53** | Accounts & usage: account tree with provider folds, inspector with meters, `FormDialog` add/edit, picker chain for secret references, usage dialog with per-account tabs | S17, S19, S25 |
| **SC54** | Editor shell (extends Settings): change tables, Save preview with diff lines | S20, S24 |
| **SC55** | Launch cockpit: `StepRail` stages, activity chip opening the build log overlay, Cancel launch, cross-fade handoff | S22, S26 |
| **SC56** | Read-only report: `live · ago` meta, meters, facts, toned body, `r` refresh, Next buttons | S17 |

### 7.6 Parallel groups

| Group | Steps | Notes |
|---|---|---|
| SC-A | SC0 | Alone: it writes the registry and the allowlist every later step edits. |
| SC-B | SC1 ‖ SC2 | Disjoint: SC1 owns the Inspector, `options.rs` and `--opt`; SC2 owns `specimen.rs`, the template, page registration and stubs. Both may run beside S-group B or later. |
| SC-C | SC3 ‖ SC4 ‖ SC7 ‖ SC8 ‖ SC9 ‖ SC10 ‖ SC11 ‖ SC12 ‖ SC13 ‖ SC14 ‖ SC15 ‖ SC16 ‖ SC17 ‖ SC18 ‖ SC19 ‖ SC20 ‖ SC21 ‖ SC22 ‖ SC23 ‖ SC24 ‖ SC25 ‖ SC26 ‖ SC28; then SC5 → SC6 (both touch `app.rs`); SC27 after SC14 | One agent per step, disjoint page files; each rebases only its allowlist lines. Run 4–6 at a time. A page step must not run while an S-step migrates the same page file (check §4's owners). |
| SC-F | SC29–SC45 | Each in the same change as its S-step, or by a paired agent in that S-step's group. |
| SC-S | SC46–SC56 | When their deps have landed; parallel among themselves. |

### 7.7 Inventory mapping

Every ⚠️ and ❌ row of both inventories maps to at least one SC step; the ✅ rows of the apps inventory whose action column still asks for a demo are mapped too. When SC0 lands, these rows become its allowlist entries.

**Library inventory: 162 rows (69 ⚠️, 93 ❌)**

| module | item (inventory row) | shown | SC step |
|---|---|---|---|
| core | `edit::EditEntry` Immediate/TypeToEdit/Explicit | ⚠️ | SC8; SC1 (app-wide `Theme::with_edit_entry`); SC24 (CodeEditor); SC17 (Dialog) |
| core | `edit::FocusSelect` SelectAll/CaretEnd/CaretStart | ❌ | SC8; SC1 (app-wide) |
| core | `edit::FieldKeys` Emacs/Desktop, `for_platform` | ❌ | SC8; SC1 (app-wide `Theme::with_field_keys`) |
| core | `edit::TextCmd`, `CMD_TEXT_KEYS`, `text_keymap` | ❌ | SC8 (custom `text_keymap`, bound-command list) |
| core | `edit::UndoStack`/`EditKind` | ⚠️ | SC8 (undo coalescing explainer) |
| core | `edit::clipboard` | ⚠️ | SC8 (fallback and status); SC34 (truthful copy result, after S14) |
| core | `event::InputConfig` (shift_wheel_horizontal, ctrl_click_secondary, legacy_ctrl_punctuation) | ❌ | SC28; SC1 (app-wide `InputConfig`) |
| core | `event::ClientPlatform` MacOs/Other | ⚠️ | SC1 (live platform switch) |
| core | `focus::FocusFallback` Nearest/First | ❌ | SC28; SC1 |
| core | `focus::BlurAction` Commit/Cancel/Keep | ❌ | SC8 (Input, TextArea); SC12 (Grid); SC13 (Table, `EditorConfig.blur`); SC1 (app-wide) |
| core | `focus::request_focus_next/cancel_focus_request` | ❌ | SC28 |
| core | `intent::Intent/Phase/FocusVia/ActionKey/DismissReason/LayerEvent` | ❌ | SC1 (State tab: intent log) |
| core | `keyboard::KeyboardProtocol` Auto/On/Off, kitty flags, `super_delivered` | ❌ | SC28; SC1 (State tab: active protocol) |
| core | `keymap::Nav`/`NavKeys` (arrows/vim/page/home_end) | ❌ | SC1 (app-wide `NavKeys`); SC28 (per-widget `nav()`) |
| core | `keymap::KeyBinding/KeyMap/Actions/lookup/label_for/hint_key/select_all_bindings` | ⚠️ | SC28 (`.keymap()` override demo) |
| core | `PointerConfig.focus_on` Press/Release | ⚠️ | SC1 |
| core | `PointerConfig.double_click_ms/slop` | ❌ | SC1 |
| core | `PointerConfig.wheel_lines` | ❌ | SC1 |
| core | `PointerConfig.hover_refresh_after_scroll` | ❌ | SC1 |
| core | `PointerConfig.flash_ms` | ❌ | SC1 |
| core | `PointerConfig.secondary_outside` Reopen/Close/Ignore | ❌ | SC1 |
| core | `PointerConfig.end_edit_on_dead_space` | ❌ | SC1 |
| core | `PointerConfig.autoscroll/_ms` | ⚠️ | SC1 |
| core | `PointerConfig.track_repeat/_delay_ms/_ms` | ⚠️ | SC1 |
| core | `PointerConfig.pointer_shapes/caret_shape` | ⚠️ | SC1 |
| core | `PointerConfig.click_skips_animation` | ❌ | SC1 |
| core | `response::Response/Flow/Invalidate/StateFlags` | ❌ | SC1 (State tab: `Response` log) |
| core | `scroll::FollowCursor` OnMove/Always | ❌ | SC22 |
| core | `scroll::ScrollbarVisibility` Auto/Always/Never | ❌ | SC22 |
| core | `scroll::TrackClick` Jump/Page | ❌ | SC22 (per widget); SC1 (app-wide) |
| core | `scroll::HScroll` (enabled/bar/step/track_click, `off`) | ⚠️ | SC22 (per widget); SC1 (app-wide) |
| core | `selection::SelectionMode` None/Single/Multi | ⚠️ | SC11 (`None`) |
| core | `shape::PointerShape` (5 shapes) | ⚠️ | SC1 (State tab: shape indicator) |
| core | `shape::CaretShape` (4), `ShapeSupport`, `ShapeSync` | ❌ | SC1 (`caret_shape`, `ShapeSupport` readout) |
| core | `typeahead::TypeAhead` (timeout) | ⚠️ | SC15 (timeout knob) |
| core | Strategy B — `nav.rs` modules (`FocusPolicy`, `AdjacentMove`, `AdjacentKeys`, `Scope`, `Bounds`) | ❌ | SC28 |
| core | Strategy B — nav policy fields on Tabs/Tree/List/Select/Picker/Menu/Dialog/Toolbar | ❌ | SC28 |
| core | Strategy B — Toolbar widget | ❌ | SC28 (Toolbars page) |
| ui | `columns::Col` (content/fixed/flex/max/droppable) + `solve` | ❌ | SC11 |
| ui | `columns::ColumnSizing` (resizable/seam_w/min_width) | ⚠️ | SC11 |
| ui | `columns::header_menu`/`HeaderAction` (5)/`HeaderLabels` | ❌ | SC11 (DataTable); SC12 (grid header menu) |
| ui | `columns::render_seam` | ⚠️ | SC11 |
| ui | `context::ContextConfig` (keys, moves_cursor) | ⚠️ | SC16 |
| ui | `context::ContextKeys` (shift_f10/menu_key/extra) | ❌ | SC16 |
| ui | `ctx::FocusVisible` Always/KeyboardOnly | ⚠️ | SC1 (app-wide); SC7 (`Button::focus_visible`) |
| ui | `ctx::VisualState` (8 flags) | ⚠️ | SC2 (States grid on every widget page) |
| ui | `RenderCtx.reveal/tooltips/hscroll/track_click` (app-wide) | ❌ | SC1 |
| ui | `ctx::Backdrop` (keep_bottom/keep_top) | ❌ | SC19 |
| ui | `layout::Split` (percent/min_*/maximized) | ⚠️ | SC23 |
| ui | `layout::SplitDir` | ⚠️ | SC23 |
| ui | `layout::Overflow` Scroll/Clip, `min_child_rows`/`scrollbar_gap`/`track_click` | ❌ | SC23 |
| ui | `leave::Stakes` Clean/Unsaved/Running | ⚠️ | SC6; SC27 (Task runner `Running`) |
| ui | `leave::QuitKeys` (ctrl_c/ctrl_q/letter/esc_quits) | ⚠️ | SC6 |
| ui | `leave::KeyContext/LeaveKey` Copy/Quit | ⚠️ | SC6 |
| ui | `leave::unsaved_dialog/discard_dialog/running_dialog` | ⚠️ | SC6 |
| ui | `modal::Modal/ModalStack/Dismissal/ensure_focus` | ❌ | SC31 (after S5, S7) |
| ui | `modal::Frame` (placement/anchor/fit/margin/min_size/backdrop/dim) | ⚠️ | SC19 |
| ui | `modal::refuse_outside_click/close_button/border_style` | ⚠️ | SC19; SC31 (`Frame::close_button`, after S5) |
| ui | `popup::Dismiss` (escape/outside_click/pass_through/close_button/refuse_feedback) | ⚠️ | SC19; SC31 (`focus_out`, `refuse_feedback`, after S5) |
| ui | `popup::RefuseFeedback` Flash/None | ⚠️ | SC19 |
| ui | `popup::Placement` (5) | ⚠️ | SC19 |
| ui | `popup::Overflow` Shrink/Slide, `VAlign`, `Place`/`place_with` | ⚠️ | SC19 |
| ui | `reveal::Reveal` (hover/keyboard/max_rows/scroll/wheel_lines/delay_ms/wheel_passes_at_ends) | ⚠️ | SC19 (per widget); SC1 (app-wide) |
| ui | `reveal::RevealMode` Tooltip/InlineWrap | ⚠️ | SC19 |
| ui | `text::truncate_middle/thousands/fit_right/slice_cells/wider_than/width_capped/find_ranges` | ❌ | SC26 |
| ui | `text::WrapOpts`/`wrap_with` | ❌ | SC26 |
| ui | `tooltip::Tip` + `show` | ⚠️ | SC19 |
| ui | `tooltip::Tooltips` (enabled/delay_ms/max_width) | ❌ | SC19; SC1 (app-wide) |
| widgets | `Lockup` (text, compact) | ⚠️ | SC4 |
| widgets | `Breadcrumb` (segments, clickable) | ❌ | SC5; SC34 (clickable shell crumb through `Router`, after S14) |
| widgets | Button tooltip/shortcut | ⚠️ | SC7 |
| widgets | Button focusable/focus_visible/mnemonic | ❌ | SC7 |
| widgets | `ButtonRow` (align, gap, overflow More/Wrap), `row_layout_right`, `RowLayout.more/hidden` | ❌ | SC7; SC36 (`ActionRow`, after S17) |
| widgets | `ChipBar`/`Chip` | ⚠️ | SC10 |
| widgets | ChipBar overflow_marker/overflow_menu/close_target_w | ⚠️ | SC10 |
| widgets | `ChipCmd`, ChipBar `nav`/`keymap` | ❌ | SC10 |
| widgets | `Checkbox` (checked/disabled/mixed/hit) | ⚠️ | SC9; SC33 (`CheckClick`, after S9) |
| widgets | `RadioGroup` | ⚠️ | SC9 |
| widgets | `ChoiceDialog` (radio/stepper/option_tones/line/buttons) | ❌ | SC18 |
| widgets | CodeEditor read_only/tab_leaves/esc_reverts/edit_entry/field_keys/text_keymap/nav/keymap/scrollbar_gap/track_click/horizontal/indent | ❌ | SC24 |
| widgets | CodeEditor find (`FindState`) | ⚠️ | SC24 (discoverable find); SC41 (`FindState` shared, after S22) |
| widgets | `CodeCmd` (7) | ⚠️ | SC24 |
| widgets | Completion max_rows/popup_width/fit/placement/cursor_style/nav | ❌ | SC24; SC32 (`OptionList` body, after S12) |
| widgets | `Dialog::facts` | ⚠️ | SC17 |
| widgets | Dialog accelerators/mnemonics/guard_dirty_prompt/confirm_index/button_row/body_scroll/max_height/outer_margin/backdrop/placement/anchor/nav/keymap/edit_entry/width | ❌ | SC17 |
| widgets | Dialog body overflow | ❌ | SC17 |
| widgets | `DiffStatus` | ⚠️ | SC25 |
| widgets | DiffView empty/no-file/narrow | ⚠️ | SC25 |
| widgets | `unified_lines`/`review_lines` (`_with`), `DiffLineKind` | ⚠️ | SC25 |
| widgets | EmptyState `error()` | ❌ | SC21 |
| widgets | EmptyState `actions()`/`EmptyAction`/`render_ctx` | ❌ | SC21 |
| widgets | `PointerSelect` (drag/word/line/drag_by_word) | ❌ | SC8 |
| widgets | `FormDialog` (+ FormField kinds incl. chooser/note/hidden, FieldValue, FormEvent, keep_open_on_save) | ❌ | SC18; SC37 (`FormPanel` fields, after S18) |
| widgets | Grid paging, `RowTotal` | ⚠️ | SC12 |
| widgets | Grid `RowState`, cell/row errors | ⚠️ | SC12 |
| widgets | Grid read-only (`GridEvent::ReadOnly`) | ⚠️ | SC12 |
| widgets | Grid editor_config/type_ahead_column/overflow_badges/bar_labels/duplicate_keys/width_sample_rows/resizable_columns/seam_hit_w/track_click/nav/keymap/row_numbers/header_labels/filtered_cols/blur_action | ❌ | SC12 |
| widgets | `GridCmd` (18) | ⚠️ | SC12; SC33 (generated key reference, after S8b) |
| widgets | `HelpOverlay` | ❌ | SC6; SC33 (`from_keymaps`, after S8b) |
| widgets | `HintLayer.align`/`HintAlign` | ❌ | SC4 |
| widgets | `InfoDialog` | ❌ | SC18 |
| widgets | TextInput required_message/wrap_label/blur_action/focus_select/field_keys/text_keymap/pointer_select/label_hit | ❌ | SC8 |
| widgets | `keyhint` render/hint/Hint/constants | ⚠️ | SC4 |
| widgets | `KeyReference` | ❌ | SC6 |
| widgets | ListBox ClickAction/skip_disabled/wrap/type_ahead/reveal/empty()/scrollbar_gap/track_click/nav/keymap, `ListCmd` | ❌ | SC15; SC33 (`ClickPolicy`, after S9) |
| widgets | MenuItem danger_if/separator_if | ❌ | SC16 |
| widgets | ContextMenu placement/anchor(_control)/fit/min_width/max_rows/edge_margin/release_selects/hide_owner_focus/nav/wrap/type_ahead | ❌ | SC16 |
| widgets | MenuBar OpenOn, Overflow Hide/Chevron, hover_switch, press_drag, hide_owner_focus, nav | ❌ | SC4 |
| widgets | `MenuBarEvent::Closed` | ❌ | SC4 |
| widgets | `EditorConfig` (10 fields) | ❌ | SC13 |
| widgets | `Affordance` Focused/Always/Never | ❌ | SC13 |
| widgets | `ChoiceEditor`/`ChoiceResult` standalone | ⚠️ | SC13 |
| widgets | Panel PanelFocus Title/Bar, focus_mark, meta_short, bg_override, mark_clipped | ⚠️ | SC22 |
| widgets | ScrollPanel hscroll/horizontal/track_click/nav/keymap, `PanelCmd` | ❌ | SC22 |
| widgets | `PickerStatus` Ready/Loading/Error | ❌ | SC20 |
| widgets | Picker max_rows/fit/placement/anchor/backdrop/cursor_style/delete_is_secondary/nav/empty() | ❌ | SC20 |
| widgets | PickerItem disabled/key | ❌ | SC20 |
| widgets | `BarLayout`/`render_bar_with` | ❌ | SC21 |
| widgets | `props::Sheet`, `measure(_with)`, `render_with` | ❌ | SC26 |
| widgets | `PropsList` (+ PropsEvent/PropsCmd) | ❌ | SC26 |
| widgets | `SidebarRail` (collapsed_rail, RailEvent) | ❌ | SC5 |
| widgets | Segment clickable | ❌ | SC4 |
| widgets | Select SelectArrows/WheelMode/max_rows/placement/popup_width/popup_fit/cursor_style/close_on_blur/hover_moves_cursor/press_drag/label_hit/track_click/wrap/type_ahead/nav | ❌ | SC10; SC32 (`SelectArrows`, `WheelMode` on `OptionList`, after S12) |
| widgets | `SeamDoubleClick` Reset/Ignore, hit_pad | ⚠️ | SC23 |
| widgets | StatusItem busy/meter/tooltip/Emphasis::Plain; centre group | ❌ | SC4 |
| widgets | StepRail selectable/numbered/track_click/nav | ❌ | SC25 (Terminal page step rail) |
| widgets | DataTable esc_clears/overflow_marks/type_ahead_column/editor_config/resizable_columns/seam_hit_w/track_click/nav/keymap/empty()/header_labels/blur_action | ❌ | SC13; SC11 (seams, header menu) |
| widgets | `Tabs` | ⚠️ | SC14 |
| widgets | `TabItem` dirty/busy/error/prefix/suffix/closable | ❌ | SC14 |
| widgets | `TabRule` Auto/Accent/Neutral/FocusOnly | ⚠️ | SC14 |
| widgets | Tabs allow_new/new_key/close_keys/middle_click_closes/overflow_menu/close_target_w/wrap/nav/keymap; TabEvent Close/New; TabCmd | ❌ | SC14 |
| widgets | TextArea focus_select/esc_reverts/field_keys/text_keymap/nav/pointer_select/blur_action/label_hit/horizontal | ❌ | SC8 |
| widgets | TreeNode lazy/glyph/note/busy | ❌ | SC15 |
| widgets | Tree BranchAction/row_click_toggles/indent/divider/alt_click_recursive/reserve_right/show_meta/track_click/nav/wrap, `TreeCmd` | ❌ | SC15 |
| widgets | `TreeEvent::Expand` | ❌ | SC15 |
| widgets | Span italic/underline/reversed | ❌ | SC25 |
| widgets | TextViewport caret/wrap/hscroll/scrollbar_gap/track_click/nav/keymap/with_lines, ViewportCmd, `SelectionChanged`, WorkCounters | ❌ | SC25 |
| theme | `ColorLevel` ×4, detect, label | ⚠️ | SC1 (live switch); SC2 ("Compare levels"); SC3 |
| theme | `LevelOverrides`, `for_level_with` | ❌ | SC1; SC3 |
| theme | `Plane`/`Kind`/`Ladder(s)`/`with_ladder` | ❌ | SC3 |
| theme | `StateModifiers` | ❌ | SC3 |
| theme | `CursorStyle` Fill/Bar, `Interactive`, `CursorRow` | ❌ | SC3; SC33 (`RowLook`, after S10) |
| theme | `BadgeKind` | ⚠️ | SC3; SC4 |
| theme | `HintAlign`/`HintBarStyle` | ❌ | SC4 |
| theme | `Glyphs` + ascii_glyphs/ascii/with_glyphs | ❌ | SC1; SC3 |
| theme | `Strings` + with_strings/fill/follow | ❌ | SC1; SC3 |
| theme | `Spacing`/`Insets` | ❌ | SC3 |
| theme | Theme edit_entry/field_keys defaults | ❌ | SC1 |
| theme | `contrast_ratio`/`legible` | ❌ | SC3 |
| runtime | tick_interval / motion | ⚠️ | SC1; SC30 (`Motion::Reduced` fix, after S2) |
| runtime | `Coalesce` (6 fields, OFF) | ❌ | SC1 |
| runtime | `TerminalSession`, job_control | ⚠️ | SC0 (allowlist: runtime plumbing, exercised by every run; no visible surface) |
| runtime | `SizeWatch`/`InputPump`/drain* | ⚠️ | SC0 (allowlist: runtime plumbing); SC1 (State tab: size and input counters) |
| Cross-cutting visual states and shell | selected | ⚠️ | SC2 (static selected row in every States grid) |
| Cross-cutting visual states and shell | busy | ⚠️ | SC2; SC4 (StatusItem); SC14 (TabItem); SC15 (TreeNode) |
| Cross-cutting visual states and shell | error | ⚠️ | SC2; SC10 (Chip); SC21 (EmptyState); SC20 (Picker); SC14 (Tab) |
| Cross-cutting visual states and shell | empty | ⚠️ | SC2; SC34 (`ContentState` on every list-like widget, after S13) |
| Cross-cutting visual states and shell | loading | ⚠️ | SC2; SC20 (Picker Loading); SC15 (lazy tree) |
| Cross-cutting visual states and shell | overflow | ⚠️ | SC2; SC7 (ButtonRow); SC14 (Tabs); SC4 (MenuBar); SC17 (Dialog body) |
| Cross-cutting visual states and shell | narrow / too small | ⚠️ | SC2 (24-column narrow slot); SC30 (`TooSmall`, after S2) |
| Cross-cutting visual states and shell | colour levels | ⚠️ | SC1; SC2 |
| Cross-cutting visual states and shell | Inspector `i` | ⚠️ | SC1 |

The ✅ library rows with gaps noted in their "missing" column (Theme tokens, Button busy row, disabled on radio, checkbox, menu-bar item, tab and picker row, hover/pressed matrices for other widgets) are covered by SC2, SC3, SC7, SC9, SC4, SC14 and SC20 as part of their States grids.

**Apps inventory: 92 patterns (82 ⚠️ or ❌; 4 ✅ rows that still ask for a demo; 6 ✅ rows covered)**

| # | pattern | shown | extract first | SC step |
|---|---|---|---|---|
| 1 | Collapsible sidebar → `SidebarRail` | ❌ | — (library has it) | SC5 |
| 2 | Tab strip with icons, `×`, `+`, dirty/busy/error flags | ⚠️ | — | SC14 |
| 3 | Tab overflow arrows, `+N` menu, wheel, middle-click, digit jump | ❌ | — | SC14 |
| 4 | Nested tab strips (main › segmented › sub-tabs) | ❌ | `Tabs::{layout, passive}` (S17) for the composed case | SC14 (nesting); SC46 (in context) |
| 5 | Preview tab (single click replaces, Enter promotes) | ❌ | S23 (`Tabs` preview flag) | SC42 |
| 6 | Tab list picker with Delete and live refresh | ⚠️ | — | SC20 |
| 7 | Panel title meta, `meta_short`, badge, focus mark | ⚠️ | — | SC22 |
| 8 | Pane zoom / maximise (`z`, Esc restores) | ❌ | S26 (H and J onto `Split::toggle_max`) | SC23 (library `Maximized` now); SC45 |
| 9 | Splitter drag, double-click reset, nudge, clamp, 3 panes | ⚠️ | S19 (`SplitKeys` nudge) | SC23; SC38 |
| 10 | Master-detail with responsive drawer | ❌ | S19 `MasterDetail` | SC38 |
| 11 | Empty state with action buttons | ⚠️ | — | SC21 |
| 12 | Typed-confirmation facts dialog | ⚠️ | — | SC17 |
| 13 | Read-only facts or code viewer dialog | ⚠️ | — (`Dialog::viewer` preset, if added, lands in SC17 with its demo) | SC17 |
| 14 | `InfoDialog` copyable rows, error variant | ❌ | — | SC18 |
| 15 | `ChoiceDialog` radio, stepper, option tones | ❌ | — | SC18 |
| 16 | `FormDialog` all field kinds, disclosure, Validate | ❌ | — | SC18 |
| 17 | Help / key reference overlay | ❌ | S8b (`HelpOverlay::from_keymaps`, H migrated) | SC6 (library widgets now); SC33 |
| 18 | Multi-action destructive dialog | ⚠️ | — | SC17 |
| 19 | Dialog anchored to a control / placement | ❌ | — | SC17 |
| 20 | Breadcrumb trail (clickable, middle collapse) | ❌ | S14 (J crumb strings onto `Breadcrumb`) | SC5; SC34 |
| 21 | Page stack inside one tab (Esc pops) | ❌ | S14 (`Router` per-tab stacks) | SC34 |
| 22 | Menu bar per route, brand opens About | ✅ | — | SC4 (per-route menus, About) |
| 23 | Identity strip of segments (env tone, clickable, spinner) | ⚠️ | — | SC4 |
| 24 | Status bar busy / meter / strong / clickable / reveal | ⚠️ | — | SC4 |
| 25 | Hint bar layers, EDIT badge, toggle hints, status TTL | ⚠️ | S2 (status TTL), S8b (`Footer` layers) | SC4; SC30; SC33 |
| 26 | Toast / status with tone and expiry | ⚠️ | S2 `StatusLine` | SC30 |
| 27 | "Terminal too small" screen | ⚠️ | S2 `TooSmall` | SC30 |
| 28 | App shell layout, page dimmed under a layer | ⚠️ | S7, S14 (`Shell`, `Chrome`) | SC34 (the showcase shell itself runs on `Shell`) |
| 29 | Context menus: anchors, reasons, danger, key replay | ⚠️ | S22 `CommandMenu` (key replay) | SC16; SC41 |
| 30 | App/brand menu anchored at a segment | ❌ | — | SC4 |
| 31 | Column header menu on the data grid | ⚠️ | — | SC12 |
| 32 | Open Quickly picker | ✅ | — | — (covered) |
| 33 | Non-searchable level picker | ✅ | — | — (covered) |
| 34 | Picker Loading/Error/Ready, live suggestions, chain | ❌ | S25 `PickerChain` (chain only) | SC20 (status, live suggestions); SC44 |
| 35 | Command palette with disabled reasons | ⚠️ | — | SC20 |
| 36 | File browser modal | ❌ | S22 `FileBrowser` | SC41 |
| 37 | Hand-built filter editor modal | ❌ | S18 (onto `FormDialog` + live preview slot) | SC37 |
| 38 | Text modal (About / Why / Help) | ❌ | S8b (merge into `HelpOverlay` / `InfoDialog`) | SC33 |
| 39 | Data grid typed columns, FK, filters, FetchMore, duplicate | ⚠️ | — | SC12 |
| 40 | Read-only explanation (grid reason, menu reasons, titles) | ❌ | — | SC12; SC16; SC21 ("Read-only" demo) |
| 41 | Table footer status note with priority parts | ❌ | S23 `FooterNote` | SC42 |
| 42 | Pending-change queue → Save review with typed token | ⚠️ | — | SC12 |
| 43 | Filter chip bar (lead, enable) + filter on cell + follow FK | ⚠️ | — | SC10; SC12 |
| 44 | Stream toggle chips (non-removable, 1-9) | ⚠️ | — | SC10 |
| 45 | Chip/cell row with `+N` overflow (`overflow_run`) | ❌ | — | SC10 |
| 46 | Structure tables | ✅ | — | — (covered) |
| 47 | DDL / raw-text card (styler, position meta) | ⚠️ | — | SC22 |
| 48 | SQL editor: running statement, read-only | ✅ | — | SC24 (running highlight, read-only) |
| 49 | Toolbar buttons (not focusable, shortcut, Cancel while busy) | ⚠️ | — | SC7 |
| 50 | Result sub-tabs (pinned, error flag) | ❌ | S23 (pinned tab) | SC14 (error flag); SC42 (pinned) |
| 51 | Running status line ("running 1.2 s · Esc cancels") | ❌ | S23 `ActivityLine` | SC42 |
| 52 | Error card / inline failure block | ❌ | S17 `Notice::error` | SC36 |
| 53 | Fact cards | ✅ | — | — (covered) |
| 54 | Key/value detail by hand (scroll, fit) | ❌ | S17 `DetailCard` (+ `props` scroll) | SC36 |
| 55 | EXPLAIN plan tree-table | ❌ | S16 `TreeView::columns` | SC35 |
| 56 | Tree glyphs, meta, lazy + busy, note rows, selection glyphs | ⚠️ | — | SC15 |
| 57 | Hand-drawn trees with action row and per-row spinner | ❌ | S16c (J trees onto `TreeView`), S24 (trailing action row) | SC43 |
| 58 | Filter field over a tree/list | ❌ | S21 `FilterTree` / `FilterList` | SC40 |
| 59 | Tri-state checkbox tree-list | ❌ | S24 (tri-state check column) | SC43 |
| 60 | Grouped result table, `columns::solve`, match ranges | ❌ | S24 (group rows, match ranges) | SC11 (`solve` now); SC43 |
| 61 | Change/diff tables (`+ • −`, group folds, blockers) | ❌ | S24 (row marks, group rows) | SC43 |
| 62 | Plan outline table (review/run state column) | ⚠️ | S24 (`StepRail` table variant) | SC43; SC50 |
| 63 | Step rail / launch stages | ✅ | — | — (covered; counter segment in SC25) |
| 64 | Wizard / stepper string with Back | ❌ | S22 `StepLine` + `Wizard` | SC41 |
| 65 | Progress incl. aggregate header | ✅ | — | SC21 (aggregate header) |
| 66 | Meters list, Refreshing/Stale tones | ⚠️ | S17 `MeterList` | SC21 (tones now); SC36 |
| 67 | Live log viewport: find, marks, retention, prefixes | ⚠️ | S22 `TextViewport::find` | SC25 (marks, retention, prefixes); SC41 (find) |
| 68 | Attach/detach + stdin input mode | ❌ | — (behaviour stays app-level; shown with library pieces) | SC51 |
| 69 | PTY pane tree with prefix commands and zoom | ❌ | S26 `PaneTree` | SC45 |
| 70 | Diff viewer with file tree, narrow stack | ⚠️ | — | SC25 |
| 71 | Forms: two columns, tabs with error, dependent disable, test result | ⚠️ | S18 `FormPanel` | SC37 |
| 72 | Leave guard 3-way dialog | ✅ | — | SC6 (explicit demo) |
| 73 | Editor shell (dirty tabs, save preview, busy save) | ⚠️ | S20 `EditorShell` | SC39 |
| 74 | Button rows: right aligned, wrap, Next row, message | ⚠️ | S17 `ActionRow` (message) | SC7 (rows now); SC36 |
| 75 | Connect/launch lifecycle (spinner, failure, Reconnect) | ⚠️ | — | SC47 |
| 76 | Page header and section headings | ❌ | S17 `PageHeader` / `Heading` | SC36 |
| 77 | Finder query row (typing-hot, scope readout) | ❌ | S11 (K36: finder query on `TextInput`), S17 (`TextInput::readout`) | SC36; SC48 |
| 78 | Search field with right readout | ❌ | S17 `TextInput::readout` | SC36 |
| 79 | History list + detail (status-toned items) | ❌ | S16 (`ListItem::status`) | SC35 |
| 80 | Files browser list as a table | ❌ | S16b (H rows onto `DataTable`) | SC35 |
| 81 | Snapshot read-only report | ❌ | S17 (`PageHeader`, `MeterList`) | SC56 |
| 82 | Usage screen / dialog (tabs per account, quota meters) | ❌ | S17 `MeterList` | SC53 |
| 83 | Intro / outro ritual, cross-fade handoff | ❌ | S26 (`dim_buffer` cross-fade under `Motion`, the generic part only) | SC45 (cross-fade). The intro and outro animations themselves (typewriter, glitch caption, starfield, rain) stay in jackin: product owner decision 2026-10-16, a `permanent` app-local exclusion in the SC0 allowlist (`app::jackin_preview::intro`, `app::jackin_preview::outro`); no step extracts or demos them |
| 84 | Motion policy full/reduced/paused | ⚠️ | S2 `Motion` | SC1 (switch); SC30 |
| 85 | Press flash, focus on press, keyboard-only ring | ✅ | — | — (covered) |
| 86 | Scrollbars + edge fades on hand-drawn lists | ⚠️ | S3 `render_with_fade` | SC22 (`render_vertical` + `scroll_edges` now); SC29 |
| 87 | Reveal truncated cell/row (reveal key advert) | ❌ | — | SC19; SC4 (advert) |
| 88 | Tooltip on a status/strip item | ⚠️ | — | SC4 |
| 89 | Clipboard copy with truthful result | ⚠️ | S14 `Clipboard` | SC34 |
| 90 | Safe Mode / environment semantics | ❌ | — (domain; visual parts only) | SC4 (env-toned segment); SC46 (gate) |
| 91 | Copyable facts / `y` copy rows | ❌ | — | SC18 (`InfoDialog` copyable) |
| 92 | Global keyboard grammar (Esc ladder, chords) | ⚠️ | S8a (`KeyRouter`, `ShellKeys`) | SC46 (Esc ladder in the workbench) |

### 7.8 Step status

Each step's agent updates its row when the step lands: state, commits, deviations, and the number of allowlist entries it removed.

| Step | State | Commits | Deviations and notes |
|---|---|---|---|
| SC0 | Done, 2026-10-16 | feat(catalog): library capability registry; test(showcase): coverage gate with allowlist; feat(catalog): register the new tone and title-status builders | See "SC0 outcome" below. |
| SC2 | Done, 2026-10-16 | feat(ui): StateGrid and Specimen for forced-state samples; feat(ui): StateGrid row bands and single-level panes; feat(showcase): page template; Buttons and Inputs States on StateGrid; feat(showcase): planned page stubs; test(showcase): tree rules; test(visual): showcase sidebar re-approval for the planned pages | See "SC2 outcome" below. 13 of the 14 SC2 allowlist lines deleted. |
| SC1, SC3–SC56, S23–S26 | Not started | — | — |

**SC0 outcome.**
- `junie_tui::catalog` (`src/catalog.rs`): `CapKind` (`Type`, `Variant`, `Builder`, `Token`, `State`), `Capability { id, kind }`, `CAPABILITIES` (1350 entries: 301 types, 527 variants, 460 builders, 49 `Theme` tokens, 13 cross-cutting states) and `find(id)`.
- Source scan (`src/bin/showcase/catalog_scan.rs`, test-only): a token scanner (no `syn`) over `core`, `ui`, `widgets`, `theme` and `runtime`, following out-of-line `pub mod` children. It lists `pub struct`, `pub enum` and every variant, builders (`pub fn`/`pub const fn` in an inherent `impl`, taking `self` or `mut self`, returning `Self` or the impl's own type) and the public fields of `theme::Theme`. It skips `pub(crate)` items, private inline modules, `#[cfg(test)]` items, trait impls, and function and macro bodies. A builder in an `impl` outside its type's module is filed under the type's module.
- Showcase declarations (`src/bin/showcase/coverage.rs`): `Shown::{Demo(page, label), Option(page, key), StateRow(page, row)}`. It starts with 122 `Demo` declarations: widget types, the variants and builders the pages visibly use, and the Overview token table.
- Allowlist (`tests/showcase_coverage_allow.txt`): 1228 capability lines, each mapped by module and type to its step in §7.7, plus two `app::` lines. `permanent`: `runtime::{TerminalSession, SizeWatch, InputPump}` (job control and the drains are free functions, not catalog entries) and jackin's intro and outro (`app::jackin_preview::{intro, outro}`, product owner decision 2026-10-16). Lines per step: SC1 169, SC2 14, SC3 94, SC4 55, SC5 7, SC6 34, SC7 23, SC8 83, SC9 7, SC10 44, SC11 21, SC12 98, SC13 60, SC14 34, SC15 49, SC16 37, SC17 36, SC18 48, SC19 64, SC20 33, SC21 16, SC22 35, SC23 25, SC24 44, SC25 43, SC26 19, SC28 28, SC31 5, permanent 5.
- Gate (`src/bin/showcase/app_tests_catalog.rs`):
  - `registry_matches_library_source`: a missing entry prints a paste-ready `Capability::…("…"),` line; an entry the source no longer has fails.
  - `every_capability_is_shown_or_allowlisted`: fails on a capability neither declared nor allowlisted, on a stale line (now declared, an unknown id, or an `app::` line whose app is gone), on a pending line without `SCn`, and on duplicates.
  - `showcase_declarations_hold`: each `Demo` page renders its label at 220×80, with whitespace collapsed.
  - `remaining_gaps_per_step`: prints the per-step counts above with `--no-capture`.
  - Proof tests use fixture input, never the real source. A new widget with a new builder fails both the registry check and the coverage check, and passes once allowlisted. A stale, unknown, step-less or bad `app::` allowlist line fails. Removing a declaration fails. A declaration the showcase does not render fails.
- Deviations: `Option` and `StateRow` declarations always fail until SC1 (option tables, `--opt`) and SC2 (States grids) add what they point at. SC1 then makes the gate check that an `--opt` value changes the page's rendering. Allowlist reasons are per step (the page that will show the item), not per item. The showcase digest is unchanged.

**SC2 outcome.**
- Library `ui::specimen` (`src/ui/specimen.rs`): `SpecimenState` (14 states with stable slugs: rest, hover, pressed, focused, focused_hover, selected, disabled, busy, error, editing, empty, loading, overflow, narrow), the `Specimen` trait (`Button` and `TextInput` implement it) and `StateGrid` (builders `row`, `rows`, `column`, `label_width`, `column_width`, `narrow_width`, `focus_mark`; `render`, `render_rows` for row bands, `render_levels` and `render_level` for "Compare levels", `size`, `row_heights`, `levels_height`, `rendered_rows`). Samples draw through the widget's own `render` on an inert `RenderCtx::detached` context (no hit, focus stop, cursor, overlay or tooltip; the frame's reveal and tooltip bookkeeping untouched).
- Focus depiction: a focused sample would draw the live `▎`, a second focus indicator. `StateGrid` replaces every focus-bar cell a sample draws with `focus_mark` (default `▏`, `:` under ASCII glyphs) in the muted ink (a filled control keeps its own ink), so the shape differs from the live bar at every colour level, mono included. Recorded in `docs/audits/design-review-2026-10/focus.md` (decision log; closes finding #17). An editing sample draws its caret as a static cell.
- Template (`src/bin/showcase/pages/template.rs`, `Page::template()`): `Tabs` strip Playground | Options | States as the page's first focus stop (←/→, 1–3, click); Options is the SC1 extension point (`Template::render_options`, an `EmptyState` naming SC1 until option tables land); States holds the page's `StateGrid` with a "Compare levels" toggle, scrolling row by row (one `Stack` child per grid row) or pane by pane when comparing. At 76+ page columns the strip carries the section caption and States the focus legend. Buttons and Inputs use it; their hand-drawn matrices are gone.
- Stubs: the 29 pages of §7.2 are registered (`PageId`, label, group, slug, aliases), one file each with a `page()` constructor the step replaces, sharing `pages/stub.rs`.
- Gate: `StateRow(page, row)` holds only when the page's States section draws the row whole under its label (`Template::state_rows`); `Demo` renders every section of a templated page. The tree-rules test and the golden `tests/showcase_slugs.txt` (78 names) live in `src/bin/showcase/app_tests_template.rs`.
- Deviations: `specimen` takes `SpecimenState` (which maps to `VisualState`), not `StateFlags`: widgets and the theme resolve `VisualState`. The template lives in `pages/template.rs`, not inline in `pages/mod.rs`. The ported matrices are not pixel-equal to the old ones, on purpose: real widgets instead of hand drawing, the focus depiction, pressed as a pointer press (not pressed + focused), Inputs rows two or three lines tall, the "error + focus" row dropped, a Toggle column and the selected, busy, overflow, editing, empty and narrow rows added. `state::loading` and `SpecimenState::Loading` stay allowlisted for SC20 (no SC2 specimen loads); `StateGrid::label_width` and `focus_mark` for SC7.

**How to satisfy the coverage gate.** For new public API, add its `Capability::…` line to `src/catalog.rs` (the failing test prints it). Then either show it, with a declaration in `src/bin/showcase/coverage.rs` that the page really renders, or, only while its SC step is pending, add `id | reason | SCn` to the allowlist. A step that shows an item deletes its allowlist line in the same change. Removing public API removes its catalog entry and its declaration or allowlist line.
