# Interaction, Overlay and Identity Architecture Audit

Scope: `src/core/{event,focus,hit,id,scroll,text}.rs`, `src/runtime.rs`, `src/ui/{ctx,layout,popup,text}.rs`, `src/widgets/{dialog,menu,picker,select,completion,scrollbar,viewport,splitter,tabs,list,tree,button,input}.rs`, and the three applications.
Authorities: `REFACTORING_GOAL.md` §§11–14, 17; `DESIGN.md` (Scrolling and clipping, Focus model, Interaction grammar, Elevation & Depth, Component catalogue, Agent implementation guardrails).

Everything in **Part A** is a collected fact with a `file:line` citation. Everything marked **Inference** is judgement.

---

## PART A — Current state

### A1. Identity model

**How ids are made.** `WidgetId` is a newtype over `u64` holding an FNV‑1a hash (`src/core/id.rs:10`, `:12-23`). Three constructors:

- `WidgetId::of(path)` — hashes a static path string from the FNV offset basis (`src/core/id.rs:27`).
- `WidgetId::child(index)` — continues the parent hash over `index.to_le_bytes()` (`src/core/id.rs:32`).
- `WidgetId::sub(name)` — continues the parent hash over the name's bytes (`src/core/id.rs:37`).

**Collision risk — structural, not probabilistic.** `fnv1a` is a streaming hash resumed from the parent value with **no separator byte** (`src/core/id.rs:15-23`). Therefore, for all `a`, `b`:

```
WidgetId::of(a).sub(b) == WidgetId::of(&format!("{a}{b}"))
```

This is an exact identity, not a birthday collision. Live example: `WidgetId::of("connections").sub("tree")` (`src/bin/tablepro/app.rs:148`) is the same id as `WidgetId::of("connectionstree")`. `child(i)` is safer only because `usize::to_le_bytes` contributes 8 bytes including NULs. There is no duplicate-registration check anywhere: `HitRegistry::register` (`src/core/hit.rs:30`) and `FocusRing::register` (`src/core/focus.rs:16`) both push unconditionally.

**Stability across frames.** Ids derived from `of()`/`sub()` are stable. Ids derived from `child(index)` are **positional**: `Tabs::tab_id`/`close_id` (`src/widgets/tabs.rs:106-111`), `ListBox::row_id` (`src/widgets/list.rs:83`), `TreeView::row_id`/`toggle_id` (`src/widgets/tree.rs:262-268`, keyed on the *flattened* row index), `Picker::row_id` (`src/widgets/picker.rs:118`), `ContextMenu::row_id` (`src/widgets/menu.rs:115`), `Select::option_id` (`src/widgets/select.rs:63`), `MenuBar::label_id` (`src/widgets/menu.rs:397`), and the showcase nav (`NAV.child(i)`, `src/bin/showcase/app.rs:697`). `Tabs::remove` shifts every later tab's identity (`src/widgets/tabs.rs:152-163`); TablePro's close-confirmation dialog id is itself index-derived (`CLOSE_DIALOG.child(i)`, `src/bin/tablepro/app.rs:479`) and is matched back by scanning indices (`:1842-1851`).

**Debuggability.** `Debug` prints only the hex hash (`src/core/id.rs:42-46`). The originating path is not retained, so a focus or hit id in a log or a test failure cannot be read back to a control. Applications work around this by printing *rectangles* instead of ids in the state inspector (`src/bin/showcase/app.rs:961-980`).

**Reverse mapping.** Because a hash cannot be inverted, every component that owns children implements a linear scan: `locate` on `ListBox` (`src/widgets/list.rs:186`), `TreeView` (`:432`, returning `(usize, bool)`), `Picker` (`:120`), `Completion` (`:86`), `Select` (`:65`), `Tabs` (`:122`), `ContextMenu` (`:118`), plus `owns()` predicates on each (`list.rs:191`, `tree.rs:444`, `picker.rs:123`, `completion.rs:90`, `viewport.rs:532`, `tabs.rs:126`, `menu.rs:122` and `:414`). `TreeView::locate` scans the visible range for **two** id families per row (`src/widgets/tree.rs:433-441`).

### A2. Event model

**Input.** `Input` = `Key | Mouse | Resize(u16,u16) | Paste(String) | Tick` (`src/core/event.rs:40-46`). `Mouse` carries `MouseKind` = `Move | Down | Up | Drag | Secondary | WheelUp | WheelDown | WheelLeft | WheelRight` plus a `Position` (`:79-96`). Normalisation from crossterm drops key *release* events and non-left/right buttons (`:99-134`).

**Result shapes — nine different ones.** There is no unified reply type:

| Shape | Example |
|---|---|
| `Outcome` | `ListBox::on_key` (`list.rs:118`), `ListBox::on_wheel` (`:180`), `TreeView::on_wheel` (`tree.rs:426`) |
| `(Outcome, bool)` | `Button::on_key` (`button.rs:72`) |
| `bool` | `Button::on_click` (`button.rs:86`) |
| `(Outcome, Option<E>)` | `TextInput` (`input.rs:188`), `Select` (`select.rs:72`), `Tabs` (`tabs.rs:173`), `TreeView` (`tree.rs:322`), `Picker` (`picker.rs:147`), `Completion` (`completion.rs:94`), `TextViewport` (`viewport.rs:537`), `MenuBar` (`menu.rs:436`) |
| `Option<MenuEvent>` | `ContextMenu::on_click` (`menu.rs:219`), `on_click_outside` (`:231`) |
| mutate-a-public-field, poll later | `Dialog.result: Option<DialogResult>` (`dialog.rs:52`), polled at `src/bin/showcase/app.rs:454`, `src/bin/tablepro/app.rs:1447`, `src/bin/jackin_preview/app.rs:1342` |
| `usize` row index taken from the caller | `ListBox::on_click(row)` (`list.rs:170`) — the *caller* must have already run `locate` |
| app-local outcome enum | `FilterOutcome` (`src/bin/tablepro/app.rs:1653` ff.) |
| result field + `take()` | `FileBrowser.result`, `ChoiceDialog.result`, `InfoDialog.result` (`src/bin/jackin_preview/screens/modals.rs:132`, `:577`) |

**Consumed / ignored / redraw.** `Outcome::{Ignored, Consumed, Changed}` with `consumed()` and `or()` (`src/core/event.rs:14-37`). Redraw is fused into the same enum: the runtime redraws only when `handle` returns `Changed` (`src/runtime.rs:127-130`, `:141-142`). There is no way to say "consumed, and something else must repaint" or "ignored, but repaint".

**How semantic actions reach apps.** They do not, generically. Each application re-implements the routing:

- showcase: `PageEvent` (`src/bin/showcase/pages/mod.rs:38-70`) + `Request` (`:74-77`) + `PageCtx` (`:79`).
- tablepro: `Request` (`src/bin/tablepro/app.rs:51-63`) + `Cx` (`:65`).
- jackin: `Request`/`Go`/`ModalResult`/`ModalTag` + `Cx` + a 20‑method `Screen` trait (`src/bin/jackin_preview/screens/mod.rs:110-260`).

Component events carry **indices**, not identities: `TabEvent::Close(usize)` (`tabs.rs:73`), `PickerEvent::Chosen(usize)` (`picker.rs:71`), `MenuEvent::Chosen(usize)` (`menu.rs:67`), `MenuBarEvent::Chosen(usize, usize)` (`:357`). TablePro smuggles a stable index through a display field to survive picker filtering: `it.detail.parse::<usize>()` (`src/bin/tablepro/app.rs:1506`, `:1611`).

**Keyboard/mouse equivalence.** Not enforced, and divergent in practice. `Picker` has a keyboard-only secondary action (`Delete` → `PickerEvent::Secondary`, `picker.rs:195`) with no mouse equivalent, and mouse clicks always produce `Chosen` even when `Alt` semantics exist for the keyboard (`:223-231` vs `:160-165`). `ContextMenu::on_click` produces `Dismissed` for any unknown id (`menu.rs:227`) — a state the keyboard reaches only via `Esc`. `Button` returns `bool` from both paths but through different signatures (`button.rs:72`, `:86`).

**Propagation order.** Hand-written per application, top of `on_key`:

- showcase (`src/bin/showcase/app.rs:445-555`): too-small → dialog → sidebar-when-focused → page dispatch → global keys.
- tablepro (`:491-582`): too-small → modal → `Ctrl+N` special case → `workbench_chord` (a 180-line chord block, `:584-770`) → screen → global keys → `esc_ladder`.
- jackin (`:505-683`): route-specific intro/outro/handoff → `Ctrl+C` → modal → open menu bar → `F10` → host chords → screen → Tab/`q`/Esc.

There is no capture/bubble phase, no shared precedence rule, and no library-side dispatcher. `DESIGN.md:630-631` documents the consequence as a design rule: "Application chords are handled before the focused widget sees a key, so a widget key that collides with a chord … is unreachable in that application."

### A3. Focus

**Registration.** `FocusRing` is `Vec<WidgetId>` plus one `Option<usize>` barrier (`src/core/focus.rs:10-13`), rebuilt every frame in render order (`:1-5`), which makes Tab order reading order (matches `DESIGN.md:653-655`). Registration normally goes through `RenderCtx::control` (`src/ui/ctx.rs:86-94`), which registers a hit region always and a ring entry only when not disabled — matching `DESIGN.md:656` ("Disabled controls register a hit region but no ring entry"). Two call sites bypass it and touch `ctx.ring` directly: `Tabs::render` (`src/widgets/tabs.rs:488-490`) and the showcase sidebar (`src/bin/showcase/app.rs:922-925`).

**Tab order determinism.** Deterministic *given* a stable render order, but the ring holds bare ids with no area, label, scope or disabled flag, so nothing can be validated, reordered, or logged.

**Roving cursors.** Every composite widget invents its own: `ListBox.cursor` (`list.rs:49`), `TreeView.cursor` (`tree.rs:112`), `Tabs.cursor` + `Tabs.active` (`tabs.rs:57-58`), `Picker.cursor` (`picker.rs:51`), `ContextMenu.cursor` (`menu.rs:75`), `MenuBar.cursor` (`:370`), `Select.cursor` + `selected` (`select.rs:19-20`), `App.nav_cursor` (`src/bin/showcase/app.rs:198`). No shared contract for cursor-vs-selection, wrapping, or disabled skipping — `ContextMenu::step` wraps and skips disabled (`menu.rs:173-186`), `Picker::step` clamps and gives up at the ends (`picker.rs:127-145`), `ListBox::move_cursor` neither wraps nor skips disabled (`list.rs:90-105`).

**Nested scopes.** None exist. One flat ring, one barrier index.

**Modal trapping.** `FocusRing::push_barrier` sets `barrier = order.len()` (`src/core/focus.rs:20`); `reachable()` slices from there (`:24-29`). It is reached only via `RenderCtx::begin_modal` (`src/ui/ctx.rs:126-131`), which is called **from inside render**: `Dialog::render` (`dialog.rs:373`), `Picker::render` (`picker.rs:260`), jackin's `modal_frame` (`screens/modals.rs:60`). A second `begin_modal` overwrites the index rather than pushing a layer; there is no pop. Anchored popups push only the *hit* barrier and leave the ring alone (`src/ui/popup.rs:74`).

**Restoration.** Not in the library. Three independent implementations: `App.saved_focus` (`src/bin/showcase/app.rs:217`, set `:418`, restored `:429`); `App.saved_focus` (`src/bin/tablepro/app.rs:136`, `:1188`, `:1437`); `ModalEntry.saved_focus` per stack entry (`src/bin/jackin_preview/app.rs:87`, `:1245`, `:1258`).

**Reconciliation when controls disappear.** Three divergent post-render fixups, all after the frame is drawn:

- showcase: if no modal and focus not in ring → `ring.first()`; if modal → `ensure_valid` (`src/bin/showcase/app.rs:724-732`).
- tablepro: identical shape (`:2106-2112`).
- jackin: prefers the screen's `primary_focus()` before falling back to `ring.first()` (`:2266-2274`).

`Focus::ensure_valid` itself snaps to `ring.first()` (`src/core/focus.rs:94-98`), matching `DESIGN.md:661-662`.

**Focus-visible.** `Interaction.focus_hidden` exists and gates `focused()` (`src/ui/ctx.rs:24`, `:32-34`) but **is never set true**: all three apps hard-code `focus_hidden: false` (`showcase:336`, `tablepro:221`, `jackin:332`). Dead field.

**Stale hover suppression.** The library provides only the flag `Interaction.hover_suppressed` (`src/ui/ctx.rs:26-27`) consulted by `hovered()` (`:35-37`). Each app sets it on every key and clears it on the next `Move`: `showcase:367` / `:579-588`, `tablepro:267` / `:1860-1869`, `jackin:349` / `:1539-1548`. Behaviour matches `DESIGN.md:648-650`; the implementation is triplicated.

### A4. Hit testing and pointer

**Registry.** `HitRegistry` is `Vec<HitRegion>` (`id`, `area`, `scroll_only`) plus one barrier index (`src/core/hit.rs:12-27`). Empty rects are dropped (`:31-33`, `:42-44`). `hit()` scans in reverse and skips `scroll_only` (`:65-71`); `hit_scroll()` scans in reverse and accepts anything (`:75-81`); `area_of()` returns the last-registered rect for an id (`:91-97`).

**Z-order = draw order.** Documented at `src/core/hit.rs:1-6` and relied on: list containers register before rows so rows win (`list.rs:217-219`), tree registers the disclosure toggle after the row (`tree.rs:570-574`), tabs registers the close `×` after the tab (`tabs.rs:440-444`), the dialog re-registers its own controls after its surface (`dialog.rs:478-487`), and jackin's browser does the same (`screens/modals.rs:551-557`). There is no explicit layer or z value.

**Hover.** `App.hover = hits.hit(pos)` on `Move`, with a change/suppression check (`showcase:579-588`; identical in `tablepro:1860-1869`, `jackin:1539-1548`). Hover never touches focus in any of the three — correct per `DESIGN.md:1186-1188`. Exception by design: `ContextMenu::render` moves its *cursor* to the hovered row (`menu.rs:243-248`), and `MenuBar::on_hover` switches the open menu (`menu.rs:517-529`).

**Press / release / activation.** Implemented in each application, not in the library. Pattern (showcase `:603-667`): `Down` stores `pressed = hit`, focuses if `ring.contains(id)`; `Up` re-hits, `take()`s pressed, and returns early unless `pressed == Some(id)` (`:640-642`) — so activation requires a completed click on the same target, per `DESIGN.md:649-650`. Duplicated at `tablepro:1888-1934` and `jackin:1572-1643`. Visual press is `Interaction::pressed()` = `pressed == id && hover == id`, or a `flash` (`src/ui/ctx.rs:38-40`); the 140 ms flash timer is also per-app (`showcase:327-330`, `tablepro:212-215`, `jackin:324-326`).

**Pointer capture / drag.** No capture mechanism. Drag is routed by the app using the stored `pressed` id: `PageEvent::Drag{pressed,pos}` (`showcase:590-602`), `Workbench::on_drag(pressed,pos)` (`tablepro:1871-1887`), `Screen::on_drag` (`jackin:1550-1571`). Two consequences visible in code:

1. `Interaction::pressed()` drops the press visual as soon as the pointer leaves the region (`ctx.rs:39`), so `scrollbar::render_vertical` and `Splitter::render` bypass the helper and read the raw field instead (`scrollbar.rs:32`, `splitter.rs:41`).
2. Widgets rebuild the drag geometry from a cached frame rect: `ListBox::on_scrollbar` reconstructs the track from `self.area` (`list.rs:195-205`), likewise `TreeView` (`tree.rs:448-458`) and `TextViewport` (`viewport.rs:519-530`); `Splitter::on_drag` requires the caller to re-supply the container rect and gap (`splitter.rs:56-62`).

**Wheel routing.** App-side: `hits.hit_scroll(pos)` then hand the id to the screen (`showcase:685-691`, `tablepro:2036-2046`, `jackin:1743-1750`). Delta is a literal `±3` in all three (`showcase:681`, `tablepro:2026-2029`, `jackin:1727-1730`) rather than a token — `DESIGN.md:497` specifies three rows. `hit_scroll` returns the topmost region of *any* kind, including non-scrollable ones registered later, contradicting its own doc comment (`src/core/hit.rs:73-81`); screens compensate with `owns()`. Horizontal wheel is ignored by showcase (`:679`) and forwarded with a `horizontal` flag by tablepro (`:2025`, `:2043`).

**Scrollbar click and drag.** `scrollbar::id_for(container) = container.sub("scrollbar")` (`scrollbar.rs:12-14`); the bar registers itself clickable (`:43`) and `offset_for_click` maps a track position (`:47-50`, backed by `ScrollState::offset_for_track_pos`, `scroll.rs:107`). Every consumer wires the id by hand (`src/bin/showcase/pages/lists.rs:170`, `:178`; `screens/modals.rs:417`).

**Click-outside.** Defined as "the hit test returned `None`": showcase `:629-638`, tablepro `:1910-1931`, jackin `:1625-1630` → `modal_outside_click` (`:1755-1800`). Because a modal pushes a hit barrier, everything below is unreachable, so "outside" and "over nothing" are the same event. `Dialog::on_click_outside` cancels if cancelable (`dialog.rs:350-355`); jackin adds a policy exception for typed-acknowledgement dialogs (`:1762-1764`) and a `CustomModal::cancel_on_outside_click` hook (`screens/mod.rs:92-94`).

**Double click.** Only jackin, in the shell, with a 500 ms window on the same id (`:1636-1640`). `TextInput`'s "second click edits" uses a different mechanism entirely — the caller passes `was_focused` (`input.rs:247-261`).

### A5. Cursor output

`RenderCtx.cursor: Option<Position>` (`src/ui/ctx.rs:62`), written by `set_cursor` which is a plain last-writer-wins assignment guarded only by `inert` (`:119-123`). Writers in the library: `TextInput::render` (`input.rs:411-414`), `Picker::render` query row (`picker.rs:336-339`), `TextViewport::render` caret (`viewport.rs:653-670`). Each app copies `ctx.cursor` out after `draw` and calls `frame.set_cursor_position` (`showcase:719`/`:733-735`, `tablepro:2102`/`:2113-2115`, `jackin:2262`/`:2275-2277`).

Ownership under overlays is **draw order only**. `begin_modal` explicitly sets `inert = false` (`src/ui/ctx.rs:129`), and `inert` is never set true except during jackin's handoff animation (`jackin:2330`, `:2343`). Nothing prevents a background control that is still `editing` from writing the cursor before the dialog draws; the dialog simply overwrites it because it renders last (`showcase:797-799`).

### A6. Overlays

**There is no overlay service.** Three application-owned stacks with three different capabilities:

| App | Representation | Nesting |
|---|---|---|
| showcase | `dialog: Option<Dialog>` (`app.rs:206`) | none |
| tablepro | `modal: Option<Modal>` where `Modal = Dialog \| Picker(kind, …) \| Filter(FilterEditor)` (`app.rs:112-116`), annotated "one overlay at a time" (`:111`) | none |
| jackin | `modals: Vec<ModalEntry{modal, tag, owner, saved_focus}>` (`app.rs:83-88`, `:142`) | real stack |

**Stacking / z-order.** Determined by where the app draws the overlay in `draw`: showcase draws the dialog last (`:797-799`); tablepro takes the modal out, draws it, and puts it back (`:2165-2186`); jackin pops the top modal, draws it, pushes it back, then draws a modal footer (`:2348-2368`). Only the topmost jackin modal is ever drawn — lower entries in the stack are neither drawn nor dimmed.

**Modality and barriers.** Modality is a render-time side effect of `begin_modal` (`ctx.rs:126`). Both registries hold a single barrier index (`focus.rs:12`, `hit.rs:26`), so "layers" are emulated by monotonic forward moves within one frame; there is no per-layer query and no pop.

**Inert backgrounds.** `RenderCtx.inert` (`ctx.rs:65`) suppresses hit/ring/cursor registration (`:87-89`, `:98`, `:105`, `:120`) but is used only by jackin's handoff (`jackin:2330`). Modal inertness is achieved solely by the barrier, i.e. the background *is still registered* and merely shadowed.

**Backdrop.** Re-implemented three times as a cell-walk applying `Theme::backdrop`, always excluding the footer row: `Dialog::render` (`dialog.rs:359-372`), `Picker::render` (`picker.rs:246-259`), jackin `modal_frame` (`screens/modals.rs:46-59`). Matches `DESIGN.md:534-538`.

**Esc.** Interpreted independently by `Dialog` (`dialog.rs:262-269`), `Picker` (clears query, then cancels — `picker.rs:149-155`), `Select` (`select.rs:95-99`), `Completion` (`completion.rs:128-131`), `ContextMenu` (`menu.rs:213`), `TextViewport` (clears selection — `viewport.rs:566-571`), plus app-level ladders (`showcase:544-552`, `tablepro::esc_ladder:779-812`, `jackin` `Screen::on_esc_top` `:668-680`). The ladder in `DESIGN.md:606-614` exists only as prose plus these hand-written chains.

**Anchoring, flip, clamp, clip.** Two independent implementations:

- `ui::popup::place` — `Placement::{Below, Center}`; `Below` flips above then clamps to the bottom, clamps x to the right edge; `Center` sits in the upper third (`src/ui/popup.rs:25-56`).
- `ContextMenu::placed` — `Placement::{Below, Above, Right}`, its own flip/clamp arithmetic (`src/widgets/menu.rs:143-171`).

`ui::popup::surface` registers the frame under a **shared constant id** `WidgetId::of("popup.surface")` (`src/ui/popup.rs:76`) — any two popups drawn in one frame register the same id. `Select` draws its popup inline, at the point in the frame where the field is drawn (`select.rs:208-243`), which is why `DESIGN.md:749` instructs callers to "Render the open select last so its popup sits above later siblings".

**Small terminal.** Each app has its own 72×20 too-small screen (`showcase:802-825`, `tablepro:2122-2145`, `jackin:2284-2310`). Dialog sizing clamps width to `screen.width-4` with a floor of 20 and height to `screen.height-2` (`dialog.rs:374-376`); the body loop then guards against overrunning the action row (`:404-406`). `Picker` clamps similarly (`picker.rs:261-269`).

**Contextual hints.** A reusable `HintBar`/`HintLayer` exists with precedence resolution (`src/widgets/hintbar.rs:50-52`, `:56-66`) — but only jackin uses it (`jackin:2625-2641`). Showcase hand-builds a hint vector in the shell with a modal branch (`showcase:1018-1077`); tablepro builds `Hint` values directly. Hints originate from the *screen*, keyed on the focused id — `Page::hints(focus)` (`src/bin/showcase/pages/mod.rs:108`, e.g. `pages/lists.rs:196-207`) and `Screen::hints(focus, world)` — never from the component that owns the bindings.

**Lifecycle.** No open/close/dismiss events. Completion is polled by "did `result` become `Some`" after every key and click (`showcase:455`, `tablepro:1447`, `jackin:1342`, `:1810`). Opening does the focus save inline (`showcase::open_dialog:417-423`, `tablepro::open_dialog:1187-1193`, `jackin::push_modal:1229-1253`), and each clears hover and pressed by hand.

### A7. Scrolling

`ScrollState { offset, content_len, viewport_len }` is a pure model with clamping, paging, `ensure_visible`, `visible_range`, `thumb`, and `offset_for_track_pos` (`src/core/scroll.rs:8-115`). No rendering knowledge — good separation.

**Ownership** is per widget, as a public field: `ListBox.scroll` (`list.rs:53`), `TreeView.scroll` (`tree.rs:113`), `Picker.scroll` (`picker.rs:52`), `Completion.scroll` (`completion.rs:34`), `TextViewport.scroll` (`viewport.rs:113`). Each registers itself with `ctx.scrollable` (`list.rs:219`, `tree.rs:470`, `viewport.rs:602`) except `Picker`/`Completion`, which rely on the modal/popup barrier plus app-side matching.

**Nested scrolling** is not modelled: `hit_scroll` returns exactly one id (`hit.rs:75`); there is no chaining when the innermost region is at a boundary, and no notion of axis.

**Viewport metadata mutation during render** happens in every scrollable: `list.rs:215-216`, `tree.rs:468`, `picker.rs:348-349`, `completion.rs:170-171`, `viewport.rs:591-597`. This is permitted by `REFACTORING_GOAL.md:537` ("update non-semantic viewport metadata required by the current frame").

**The wheel-vs-cursor rule** (`DESIGN.md:505-509`: the wheel moves the viewport and never pulls the cursor back; the next key move re-ensures visibility) is implemented once, privately, in `Picker` via a `cursor_dirty` flag (`picker.rs:64`, `:107`, `:113`, `:143`, `:351-354`). `ListBox` and `TreeView` satisfy it incidentally because they only call `ensure_visible` from key handlers. **`Completion::render` calls `ensure_visible(cursor)` unconditionally** (`completion.rs:172`), so `Completion::on_wheel` (`:142-145`) is undone by the very next frame — a divergence from the documented rule. `Completion::on_wheel` also always returns `Changed`, unlike the boundary rule tested in `picker.rs:613-620`.

**Follow-tail** is a `TextViewport` concern: `render` calls `scroll.jump_end()` whenever `follow` is set (`viewport.rs:598-600`), and `set_area` does the same outside render (`:249-251`).

### A8. Layout

**Primitives available.** `Split` only: percent + `min_first`/`min_second` + `Maximized`, with `layout`, `handle`, `drag_to`, `nudge`, `vertical`, `horizontal` (`src/ui/layout.rs:23-148`). Text helpers: `width`, `truncate`, `truncate_middle`, `thousands`, `fit`, `fit_right`, `wrap`, `fuzzy` — all grapheme- and display-width aware (`src/ui/text.rs:6-172`). Button rows: `row_layout` and `row_layout_right` (`src/widgets/button.rs:167-187`).

**Measurement.** There is no measurement trait. Sizes are per-widget constants and methods: `TextInput::HEIGHT = 3` (`input.rs:184`), `Select::HEIGHT = 3` (`select.rs:33`), `Button::width()` (`button.rs:62`), `Tabs::tab_width` (private, `tabs.rs:240`), `ContextMenu::size()` (`menu.rs:127`), `Dialog::height(width)` (`dialog.rs:168`). Nothing reports a minimum, and containers cannot ask a child what it needs. The showcase reinvents row/column splitting locally (`src/bin/showcase/pages/mod.rs:120-167`).

**Padding / clipping / truncation.** Insets are open-coded per widget as `+1`, `+2`, `+3`, `Margin::new(3,2)` (`dialog.rs:387`), `Margin::new(2,1)` (`picker.rs:280`), `Margin::new(1,1)` (`popup.rs:77`), `Margin::new(3,1)` (`modals.rs:95`). Clipping is `area.intersection(*buf.area())` at the top of most render methods (`button.rs:108`, `input.rs:270`, `select.rs:154`, `list.rs:208`, `tree.rs:461`, `tabs.rs:259`, `viewport.rs:581`, `splitter.rs:34`, `menu.rs:237`, `popup.rs:62`) — but not in `Dialog::render` (which clamps by construction instead) and not in `Picker::render`.

**Resize.** `Input::Resize` is stored and nothing else: `showcase:350-353`, `tablepro:231-234`, `jackin:342-345`. All real relayout happens because render recomputes from `frame.area()` each frame.

**Tiny-rect safety — one concrete underflow.** `TextInput::render` computes the help/error row width as `area.width as usize - 2` (`src/widgets/input.rs:433` and `:440`). `area` is only guaranteed non-empty (`:270-273`), so a field rendered into a 1‑cell-wide rect with `area.height >= 3` underflows `usize` — panic in debug, an enormous width in release. All other `- 1` arithmetic I traced is either guarded (`input.rs:408`, `list.rs:299-301`, `tree.rs:576-578`) or bounded by a non-zero inset (`picker.rs:508`, `completion.rs:233`).

**Inconsistent collapse direction.** When both minima cannot fit, `Split::vertical` gives everything to the **first** pane (`layout.rs:116-119`) while `Split::horizontal` gives everything to the **second** (`:136-138`).

**Stale hit regions.** Both registries are rebuilt from scratch each frame (`showcase:711-721`, `tablepro:2094-2105`, `jackin:2254-2265`), so stale regions cannot persist across frames. Within a frame, an early `return` after registration can leave a region for content that was not drawn (e.g. `Select::render` registers the control at `select.rs:199` before the popup branch; `TreeView::render` `continue`s past a too-narrow row at `tree.rs:504-506` *after* filling and registering nothing for it — that path skips registration, which is correct).

### A9. Keybindings

**Hard-coded inside generic components:**

| Component | Bindings | Cite |
|---|---|---|
| `ListBox` | ↑↓ `j k J K`, PgUp/PgDn, Home/End `g G`, Enter/Space, `a` (all/none, multi) | `list.rs:123-166` |
| `TreeView` | the above plus `h l` ←→, `*` expand all, `-` collapse all | `tree.rs:326-398` |
| `Tabs` | ←→ `h l`, `1`–`9`, Enter/Space, `x`/Delete close, `n` new | `tabs.rs:182-213` |
| `Picker` | Esc, Enter (+Alt), ↑↓, Ctrl+N/P/J/K, PgUp/PgDn, Tab scope, Delete secondary, Backspace back, Ctrl+U clear, and **every other `Char` is swallowed into the query** | `picker.rs:147-220` |
| `TextViewport` | ↑↓ `j k`, PgUp/PgDn, Home/End `g G`, `f` follow, `y` copy, Esc clear | `viewport.rs:537-577` |
| `ContextMenu` | ↑↓ `j k`, Home/End `g G`, Enter/Space, Esc | `menu.rs:189-215` |
| `MenuBar` | ←→ `h l`, Enter/↓/Space, plus F10 owned by the app | `menu.rs:436-482`, `jackin:593-597` |
| `Completion` | ↓/Ctrl+N, ↑/Ctrl+P, PgUp/PgDn, Tab/Enter accept, Esc | `completion.rs:98-133` |
| `Select` | closed ↑↓←→ change, Enter/Space open; open ↑↓ `j k`, Enter/Space commit, Esc revert | `select.rs:72-123` |
| `Button` | Enter, Space | `button.rs:73` |
| `Dialog` | Esc, Tab/BackTab, ←→ `h l`, **`y`/`n` quick answers on text bodies** | `dialog.rs:262-313` |

`Dialog`'s `y`/`n` (`dialog.rs:297-311`) is an application convention baked into a generic component; the showcase then re-describes it in its footer and suppresses it for the help dialog (`showcase:1031-1033`).

**Application chords leaking / colliding.** TablePro's `workbench_chord` claims `Ctrl+R`, `F5`, `Alt+R`, `Ctrl+X`, `Alt+X`, `Ctrl+T`, `Ctrl+W`, `Ctrl+O`, `Ctrl+P`, `Ctrl+G`, `Ctrl+Y`, `Ctrl+B`, `Ctrl+L`, `Ctrl+D`, `Ctrl+S`, `Ctrl+F`, `z`, `[`, `]`, `Ctrl+↑/↓` **before** the focused widget (`src/bin/tablepro/app.rs:584-770`, called at `:512-516`). `DESIGN.md:629-631` records the resulting dead binding (the grid's `Ctrl+D` duplicate). In the other direction, generic widgets claim bare letters (`a`, `n`, `x`, `f`, `y`, `g`, `G`, `j`, `k`, `h`, `l`, `*`, `-`) that no application can reserve. jackin's host screens claim bare `u`, `c`, `s`, `q` (`jackin:598-628`).

**The one shared keymap** is `field_common::edit_key` (`src/widgets/field_common.rs:20-110`), covering `Ctrl+A/E/U/K/W/L`, `Alt+B/F`, word motion, Shift selection, Home/End, `Ctrl+Home/End` — matching `DESIGN.md:633-640`. Its extension point is `EditAction::Apply(fn(&mut TextBuffer))` — a bare function pointer (`field_common.rs:12`), the same narrowness as `TextInput.validator: Option<fn(&str) -> Option<String>>` (`input.rs:36`).

**HintBar sourcing today.** `HintLayer` holds `Vec<Hint>` where `Hint = (&'static str, &'static str)` (`src/widgets/keyhint.rs`, used at `hintbar.rs:14-20`). Layers are assembled by the shell from hand-written literals: jackin `draw_footer` builds a per-modal-variant match (`jackin:2521-2590`) plus `Screen::hints` (`:2592-2597`); showcase builds its own list including a per-dialog branch (`showcase:1022-1047`); TablePro composes `keyhint::hint` calls. `Page::hints(focus)` matches on the focused id and returns literals (`pages/lists.rs:196-207`). No component publishes its own bindings.

### A10. Render-time semantic mutation (goal §11 violations, collected)

| Site | What render mutates | Goal §11 clause |
|---|---|---|
| `input.rs:282-286` | Commits the edit and runs `validate()` when focus was lost | "commit an edit", "run validation because focus changed" |
| `select.rs:161-167` | Sets `self.open = false` when disabled or unfocused | "close an overlay" |
| `dialog.rs:465-470` | Flips `actions.last_mut().disabled` from the ack field | activation gating changed while drawing |
| `menu.rs:243-248` | Moves `self.cursor` to the hovered row | changes a selected value |
| `tabs.rs:291-313` | Mutates `self.first` and `self.fit` (strip scroll) | viewport metadata (permitted) but conflated with cursor state |
| `viewport.rs:598-600` | `jump_end()` while following | viewport metadata (permitted) |
| `completion.rs:172` | `ensure_visible(cursor)` every frame | overrides the wheel; contradicts `DESIGN.md:505-509` |

Additionally, frame geometry is public and mutable on nearly every widget, against `REFACTORING_GOAL.md:492` ("no public mutable layout rectangles or caches"): `Button.area` (`button.rs:23`), `TextInput.area` (`input.rs:33`), `Select.area` (`select.rs:24`), `ListBox.area` (`list.rs:54`), `TreeView.area` (`tree.rs:114`), `Tabs.areas: Vec<Rect>` (`tabs.rs:60`), `Dialog.area` (`dialog.rs:51`), `ContextMenu.area` (`menu.rs:79`), `MenuBar.areas`/`brand_area` (`menu.rs:373-374`), `Picker.area` (`picker.rs:57`), `Completion.area` (`completion.rs:39`), `TextViewport.area` (`viewport.rs:119`), `Splitter.area` (`splitter.rs:19`).

---

## PART B — Design research and recommendations

Referenced conceptually, for the problem each solves and whether that problem exists here:

- **Radix / shadcn `FocusScope` + `DismissableLayer` + `Portal`** — solves *nested modality*: a stack of layers each declaring trap/dismiss/outside-pointer policy, with focus restoration owned by the layer, not the app. That problem exists here and is currently solved three times in application code (A6).
- **Bubble Tea `Msg`/`Cmd`** — solves *one uniform event/result channel* so a component never needs a bespoke reply shape. That problem exists here (A2, nine shapes). Its `Cmd` (async effect) does not: effects are synchronous and app-owned.
- **egui `Id` stack + `Response` + `interact()`** — solves *implicit identity for immediate-mode widgets* and *one interaction result struct*. Identity is the exact problem here (A1); egui's per-frame id stack maps well onto our already-immediate render pass.
- **iced `Element::map`** — solves *action translation across composition boundaries* without the child knowing the parent's message type. That problem exists here whenever a screen wraps a component (`tablepro::FilterOutcome`).
- **tuirealm** — solves *named components in a registry with subscriptions*. It buys a runtime-owned component table; the cost is boxing everything as `dyn` and losing borrowed domain data. Partly relevant, mostly to reject.
- **ratatui `StatefulWidget`** — solves *render-time state living outside the widget value*. We already do the stronger thing (widgets are retained values owned by the app), so this is a naming reference only.

Not proposed anywhere below: virtual DOM, CSS/class system, macro DSL.

---

### B1. Identity — paths vs scoped ids vs typed keys vs registrations

**Options**

1. **Status quo**: unseparated FNV chain, index children.
2. **Interned path**: `Id(u32)` into a per-process interner holding `Vec<Segment>`; full path recoverable.
3. **Scoped hash with a frame-local id stack** (egui): `ctx.scope("row", i)` pushes; ids are `hash(parent, tag, discriminant)`; debug path retained under `debug_assertions`.
4. **Typed keys per component**: `Id` addresses only *components*; children are addressed by a component-local `Part` enum plus a `Key` (`&str` or `u64`), never by a hash.
5. **Explicit registrations**: the runtime hands out generational handles at register time.

**Evaluation.** (2) costs a global interner and a lock or a `&mut Ctx` on every id construction; ids also stop being `const`, which today lets modules declare `const NAV: WidgetId = WidgetId::of("app.nav")` (`showcase:23`). (5) makes ids unusable in `const` contexts and in tests that address a control before it has ever been drawn. (4) alone cannot express "which control is focused" across heterogeneous components without a second key space. (3) preserves `const` construction, is cheap, and fixes the concatenation family with one byte.

**Recommendation — (3) + (4) combined.**

- `Id(u64)` built by FNV‑1a with a **`0xFF` separator injected before each segment**, and a per-segment kind discriminant (`Name = 1`, `Index = 2`, `Key = 3`) mixed in first. This kills `of(a).sub(b) == of(ab)` by construction and keeps `const fn`.
- Under `debug_assertions`, an `Id` additionally carries a `&'static str`-and-index breadcrumb (or the runtime keeps a side table `Id -> String` populated at registration) so `Debug` prints `forms.name[3].close`.
- Collection children are addressed by an explicit **`ItemKey`** supplied by the caller — `Key::Str(&str)` or `Key::U64(u64)` — with `Key::Index(usize)` available but *documented as unstable under reorder*. Components hash the key, so identity survives insert/remove/reorder (goal §11 "dynamic item identity must not depend only on a shifting numeric index"; Scenario E).
- Hit and focus registration record the **part token** alongside the id (see B3), so `locate()` disappears from every component and every application.
- The frame registry detects duplicate registration of the same `Id` and, in debug builds, records a `DuplicateId` diagnostic (test-visible; not a panic in release, per goal §10 "no panics during normal interaction").

**Rejected.** Interned paths (allocation + a global on the render hot path, ids no longer `const`); generational handles (breaks `const` ids and test addressing); keeping raw indices as the only child key (fails Scenario E, and today already forces TablePro to smuggle an index through a display string, `tablepro:1506`).

**Risks.** Every existing `WidgetId::of(...)` literal changes value → all recorded baselines that embed ids must be regenerated (none appear to; ids are not rendered). Caller-supplied keys add ceremony to collections that genuinely are positional — mitigate with `Key::Index` and a `from_index` helper.

**Acceptance tests**

- `id_separator_prevents_concatenation_collision`: `assert_ne!(Id::of("a").sub("b"), Id::of("ab"))`, and the same for `child`/`sub` mixes.
- `id_kinds_are_distinct`: `Id::of("x").child(1) != Id::of("x").sub("\u{1}")`.
- `id_debug_prints_path`: `format!("{:?}", Id::of("forms").sub("name").item(Key::Str("email")))` contains `forms.name`, `email`.
- `duplicate_registration_is_reported`: register the same id twice in one frame → registry exposes exactly one duplicate diagnostic naming the id's path.
- `item_identity_survives_reorder` (unit): build a list keyed by `Key::Str`, focus item `"c"`, move it from index 4 to index 0, re-render → focus id unchanged and still resolves to `"c"`.
- TestBackend: `tabs_close_targets_the_logical_tab_after_reorder` — click the `×` of the tab labelled `B` after inserting a tab before it; the emitted action names `B`.

---

### B2. Event representation — one coherent model

**Options**

1. Status quo (`Outcome` + tuples + polled result fields).
2. One flat enum `Reply<A> { Ignored, Consumed, Changed, Action(A) }`.
3. A struct: `Response<A> { flow: Flow, invalidate: Invalidate, action: Option<A> }`.
4. A message bus: components push `Box<dyn Any>` into a queue drained by the app.

**Evaluation.** (2) cannot express "consumed **and** produced an action **and** needs repaint" without nesting, which is exactly the case today (`picker.rs:147` returns a tuple for this reason). (4) loses type safety and the compiler's exhaustiveness check on actions, and defeats goal §10 ("typed variants for semantically different modes"). (3) is small, `Copy` where `A: Copy`, and composes.

**Recommendation — (3), with three orthogonal fields and combinators.**

```
Flow      = Ignored | Consumed                       // did this handler claim the event
Invalidate = None | Paint | Layout                    // what must be redone (bitset-ordered)
Response<A> { flow, invalidate, action: Option<A> }
  ::ignored() / ::consumed() / ::changed() / ::action(a)
  fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B>   // iced's Element::map, minus the DOM
  fn or(self, other: Response<A>) -> Response<A>                  // replaces Outcome::or
```

- **Consumed vs ignored** is `flow` alone; **redraw** is `invalidate` alone. This directly fixes the "wheel at a boundary is consumed without a repaint" rule (`DESIGN.md:507`), which today is expressible only by returning `Consumed` and thereby also claiming the event *and* is inconsistently honoured (`picker.rs:236-241` honours it, `completion.rs:142-145` and `list.rs:180-183` do not).
- **Semantic component actions** are the component's own associated type (`ButtonAction::Activated`, `ListAction::Activated(Key)`, `TabsAction::Close(Key)`); they carry **keys, not indices**.
- **Application-domain actions** never appear in a component; the screen calls `.map_action(...)` at the composition boundary, which is where `FilterOutcome` (`tablepro:1653`) and `PageEvent`→`Request` (`pages/mod.rs:74`) live today.
- **Child-part identity** rides in the action (`ListAction::Activated(key)`) and in the delivered event (B3), not in a follow-up `locate` call.
- **Keyboard/mouse equivalence** becomes a conformance test rather than a convention (B10).
- The polled `Dialog.result` field disappears: dialogs return `Response<DialogAction>` like everything else, and the overlay stack (B7) converts a terminal action into a close + restore.

**Rejected.** A single enum (cannot carry flow + invalidate + action); `Box<dyn Any>` bus (untyped, unmatched, and forces `'static`); keeping the polled-result idiom (it is the reason all three apps repeat `if d.result.is_some()` after every key and click).

**Risks.** `Response<A>` with a non-`Copy` action makes `or()` awkward; keep actions small and `Clone`. Mechanical churn across every widget and screen — but the goal forbids compatibility shims (§2.5), so it is a single sweep.

**Acceptance tests**

- `flow_and_invalidate_are_independent`: `Response::consumed().no_repaint()` → `flow == Consumed && invalidate == None`.
- `or_prefers_consumed_and_max_invalidate`.
- `map_action_preserves_flow_and_invalidate`.
- `wheel_at_boundary_consumes_without_repaint` for every scrollable (table-driven over List, Tree, Picker, Completion, Viewport) — today only Picker passes (`picker.rs:613-620`).
- `disabled_control_consumes_without_action`: `Button::disabled` + Enter → `flow == Consumed`, `action == None` (matches `button.rs:75-79`).

---

### B3. Dispatch — routing to a child without `owns`/`locate` chains

The concrete shape of the problem: `src/bin/showcase/pages/lists.rs:164-191` does `locate` → `focus.focus` → `on_click(row)` → separately handle the scrollbar id → separately handle wheel via `owns`. `src/bin/tablepro/app.rs:2057-2085` does the same for five controls. `src/bin/jackin_preview/screens/modals.rs:402-443` does it for seven. Goal §12 names this chain explicitly and requires it removed.

**Options**

1. Status quo.
2. **Library-owned retained tree**: the runtime holds `Vec<Box<dyn Node>>` and routes internally.
3. **Path-recorded routing**: registration records `(owner_id, part)` and the runtime hands a component a *pre-resolved part*; the app still owns the component values.
4. **Trait-object walk from the app root** on every event.

**Evaluation.** (2) is the tuirealm shape. In Rust it forces the runtime to own the components, which forces `'static` and boxing, which kills borrowed domain data in collections (goal §9.4, Scenario D) and makes it impossible for a screen to hold `&mut` to two sibling widgets. Reject. (4) re-walks the whole tree per event — goal §25.6 forbids "avoidable full-tree scans on every event". (3) keeps ownership exactly where it is (app-owned component values, immediate render) and moves only the *addressing* into the runtime.

**Recommendation — (3) path-recorded routing with typed parts.**

During render, a component registers itself and its parts through the context:

```
cx.control(self.id, area, Interactivity::Focusable{disabled});
cx.part(self.id, ListPart::Row(key), row_area);           // part token recorded, not just an Id
cx.part(self.id, ListPart::Scrollbar, track_area);
```

The registry stores, per region, `{ owner: Id, part: PartToken, area, layer, kind }` where `PartToken` is a small `u32` the component encodes/decodes with a generated-free `impl` (a plain `enum` + `as u32`). Hit-testing returns `Hit { owner, part, layer, local: Position }`.

The runtime then delivers:

```
component.on(Event::Pointer { phase: Press|Release|Drag|Wheel(delta), part, local }, cx)
component.on(Event::Key(key), cx)                         // only to the focused owner
```

Consequences, each one removing a named item from goal §12:

- `owns(id)` is gone — the registry already knows the owner.
- `locate(id)` is gone — the part token *is* the resolved child; no hash inversion, no linear scan.
- "manually focus the parent" is gone — `cx.control` recorded focusability; the runtime focuses `Hit.owner` on press when it is focusable.
- "separately apply the emitted event" is gone — the component returns `Response<A>` from the same call.
- The scrollbar becomes `ListPart::Scrollbar` rather than a separate `scrollbar::id_for(id)` id space (`scrollbar.rs:12`) that every caller wires by hand.

Screens keep exactly one dispatch decision: *which component owns this id*, and that is a match on their own `const Id`s — which is genuinely domain-specific and justified per goal §12.

**Rejected.** Runtime-owned component tree (boxing + `'static` + borrow conflicts, loses Scenario D). Full-tree walk per event (performance, §25.6). Keeping `owns`/`locate` behind a helper macro (goal §5 forbids hiding behavior behind macros; and it would not remove the scan).

**Risks.** Part tokens must round-trip; a component that encodes a key into a `u32` token needs a side table for string keys — keep the token as `{ part_kind: u16, slot: u16 }` where `slot` indexes a per-frame key table the component also owns. Components that register parts conditionally (e.g. only visible rows) must handle "part token from last frame, item now gone" — the runtime should drop hits whose frame generation is stale.

**Acceptance tests**

- `dispatch_resolves_row_without_locate`: render a list, hit a row's cell → `Hit.part == ListPart::Row(key)`; the component's `on` receives that part and never scans.
- `press_on_focusable_part_focuses_the_owner_not_the_part`.
- `no_owns_or_locate_in_public_api`: architecture check (goal §25.5) — the library exposes no `fn owns` / `fn locate`, and the three app crates contain no call sites.
- `stale_part_from_previous_frame_is_dropped`: register row key `k`, remove it, replay the old hit → `Response::ignored()`, no panic.
- TestBackend: `showcase_lists_click_selects_the_clicked_row` at 120×40 and 80×24.

---

### B4. Focus scopes and traps

**Options**

1. Status quo: flat ring + one barrier index + app-owned `saved_focus`.
2. Scope **stack** recorded during render, resolved within the innermost trapping scope.
3. Persistent scope objects keyed by `ScopeId` across frames, storing last-focused per scope.
4. Full focus *tree* with directional (spatial) navigation.

**Evaluation.** (1) cannot nest and cannot restore (A3). (4) needs geometry and a policy for ties; nothing in `DESIGN.md` asks for spatial focus — reading-order Tab is the stated model (`DESIGN.md:601-603`, `:653-655`). Reject (4) as scope creep. (2) alone loses restoration across frames because the ring is rebuilt; (3) alone cannot express "trap". Combine.

**Recommendation — (2) + (3).**

- Ring entries become `FocusEntry { id, scope: ScopeId, disabled: bool, area: Rect }` — disabled entries are *recorded* (so hit registration and ring stay consistent, and a test can assert "registered but skipped") but excluded from traversal, preserving `DESIGN.md:656`.
- `cx.scope(scope_id, ScopeKind::{Normal, Trap})` opens a scope during render and closes on drop; scopes nest. `Trap` confines Tab/Shift+Tab to that scope and its descendants — the mechanism behind Radix `FocusScope`, and the missing piece for goal §23 Scenario F.
- A persistent `FocusState` (runtime-owned, not app-owned) stores `current: Id` plus `restore: HashMap<ScopeId, Id>`. Opening a trapping scope records the outer focus; closing it restores. This deletes `saved_focus` from all three applications (`showcase:217`, `tablepro:136`, `jackin:87`).
- **Reconciliation** becomes a documented, testable rule executed by the runtime at end-of-frame, replacing three divergent app fixups (A3): if `current` is absent from the new ring, prefer (a) the nearest surviving entry in the same scope by previous index, then (b) the scope's first entry, then (c) the innermost active scope's first entry. Today two apps do only (c) and jackin does a fourth thing (`jackin:2268-2270`).
- **Focus-visible**: keep a real `focus_visible: bool` in the interaction snapshot driven by "last input was a key", instead of the currently-dead `focus_hidden` (`ctx.rs:24`). Rendering asks `cx.focus_state(id) -> FocusVis::{None, Focused, FocusedVisible}`.
- **Scopes must be declarable outside render** as well, because trapping is currently a render side effect (`dialog.rs:373`): the overlay stack (B7) opens the scope when the layer is pushed, so a modal traps focus even in a frame where it draws nothing.

**Rejected.** Spatial/directional focus (unrequested, ambiguous ties). Keeping trap-on-render (a modal that fails to draw silently un-traps). App-owned restoration (three copies, already divergent).

**Risks.** Scope ids must be stable across frames or restoration leaks; derive them from the overlay/panel `Id`. Recording disabled entries grows the ring — negligible.

**Acceptance tests**

- `tab_order_is_render_order`.
- `disabled_is_registered_but_skipped`: ring contains the entry with `disabled: true`; `next()` never returns it; the hit registry *does* contain its region.
- `trap_scope_confines_tab_and_wraps_inside`.
- `nested_traps_confine_to_the_innermost`.
- `closing_a_trap_restores_the_outer_focus`, and `…restores_even_if_the_outer_control_moved`.
- `focus_reconciles_to_the_nearest_surviving_sibling`: focus entry 3 of 5, delete 3 → focus lands on the new entry 3 (old 4), not on entry 0.
- `focus_visible_is_false_after_a_mouse_click_and_true_after_tab`.
- TestBackend: `dialog_opens_traps_and_restores` — snapshot the gutter bar `▎` position before, during, and after (`DESIGN.md:302-304`).

---

### B5. Pointer capture

**Options**

1. Status quo: app remembers `pressed`, screens re-match ids, widgets rebuild track rects from cached `self.area`.
2. Implicit capture: the runtime routes all Drag/Up to the press target automatically.
3. Explicit capture claimed by the widget on press (`cx.capture(id, part)`), released on Up or Esc.

**Evaluation.** (2) is right for scrollbars and splitters but wrong for click-outside detection, which needs the *real* hit under the pointer at release (used today at `showcase:629`). (3) makes the distinction explicit and testable, and matches how a scrollbar and a splitter differ from a button: a button wants "press visual tracks hover" (`ctx.rs:39`), a thumb wants "keep dragging after the pointer leaves".

**Recommendation — (3), with the capture record owning the drag frame.**

`Capture { owner: Id, part: PartToken, origin: Position, area: Rect }` stored in runtime interaction state. While a capture is active:

- every `Drag` and `Up` goes to the capturing owner with `local` computed against the **captured `area`**, not a re-hit — deleting `ListBox::on_scrollbar`/`TreeView::on_scrollbar`/`TextViewport::on_scrollbar`'s rect reconstruction (`list.rs:195-205`, `tree.rs:448-458`, `viewport.rs:519-530`) and `Splitter::on_drag`'s caller-supplied container (`splitter.rs:56`);
- hover/hit-testing for *other* widgets is suppressed;
- `Interaction::pressed(id)` stays true regardless of hover, which removes the reason `scrollbar.rs:32` and `splitter.rs:41` bypass the helper;
- release resolves activation as "pointer is still inside the captured area" — the completed-click rule of `DESIGN.md:649-650`, now enforced once instead of three times.

Text/range selection (`TextViewport::on_click`/`on_drag`, `viewport.rs:477-511`, and grid range selection) becomes a capture with edge auto-scroll, which `viewport.rs:494-499` already open-codes.

**Rejected.** Implicit capture-on-any-press (breaks click-outside and makes hover-driven affordances jitter). Keeping app-side `pressed` routing (triplicated, and the cached-rect reconstruction is a latent bug whenever the widget was not drawn last frame).

**Risks.** A capture must be dropped when the capturing component disappears or the terminal resizes; add a frame-generation guard. Nested captures must be rejected, not stacked.

**Acceptance tests**

- `capture_routes_drag_after_pointer_leaves_the_area`.
- `capture_keeps_pressed_visual_when_hover_leaves`.
- `release_outside_captured_area_does_not_activate` (button) but `…does_commit` (scrollbar thumb).
- `capture_is_released_on_resize_and_on_owner_disappearance`.
- `scrollbar_drag_maps_track_position_to_offset` — reuse `scroll.rs:162-168` round-trip assertions through the capture path.
- TestBackend: `splitter_drag_moves_the_seam_and_repaints_both_panes`.

---

### B6. Wheel routing

**Options**

1. Status quo: `hit_scroll` returns the topmost region of any kind; the screen maps it via `owns`; delta hard-coded ±3 in each app.
2. Innermost-scrollable, no chaining.
3. Innermost-scrollable with outward chaining at boundaries.

**Evaluation.** `DESIGN.md:499-514` is explicit: "the wheel goes to the scrollable region under the pointer, found by the hit registry; focus is not required… nested regions win by draw order (the innermost registers last)… A wheel at a boundary is consumed without a repaint." That is (2) with the boundary rule, and it forbids (3).

**Recommendation — (2), made precise.**

- Scroll regions register with axes and current headroom: `cx.scrollable(id, area, Axes::{V, H, Both}, Headroom{up, down, left, right})`.
- `hit_scroll(pos, axis, direction)` returns the innermost region that (a) covers the point, (b) handles that axis, and (c) — for correctness of the boundary rule — is returned even at zero headroom, so the event is consumed without repaint rather than falling through to an outer container.
- Wheel step is a **design token** (three rows, `DESIGN.md:497`) resolved by the runtime, not a literal in three apps (`showcase:681`, `tablepro:2027`, `jackin:1728`). Horizontal wheel becomes uniformly supported instead of `Outcome::Ignored` in showcase (`:679`).
- The wheel-vs-cursor rule becomes a shared contract: `ScrollState` gains an explicit `ensure_visible_on_next_layout` flag set only by cursor motion — generalising Picker's private `cursor_dirty` (`picker.rs:64`) and fixing `Completion::render`'s unconditional `ensure_visible` (`completion.rs:172`).

**Rejected.** Chaining to an outer scrollable at a boundary (contradicts `DESIGN.md:507`; also causes the classic "page scrolls when the list ends" annoyance). Focus-follows-wheel (contradicts `DESIGN.md:501`).

**Risks.** Headroom must be computed from the *previous* frame's layout; on the first frame after a resize a region may report stale headroom. Acceptable: worst case one consumed-without-effect wheel event.

**Acceptance tests**

- `wheel_goes_to_the_innermost_scrollable_under_the_pointer` (nested viewport in a scroll panel).
- `wheel_at_boundary_is_consumed_and_does_not_chain_outward`.
- `wheel_does_not_move_focus_or_selection`, then `next_down_arrow_moves_the_cursor_one_row_and_pulls_it_into_view` — the exact sequence in `DESIGN.md:506-508`; already asserted for Picker (`picker.rs:594-610`), must become a shared conformance test.
- `horizontal_wheel_scrolls_columns_in_a_table`.
- `wheel_over_a_modal_does_not_scroll_the_page_behind_it` (`DESIGN.md:502-503`).

---

### B7. Overlay stack as a runtime service

**Options**

1. Status quo: three app-owned stacks, modality as a render side effect, one barrier index.
2. A library `OverlayStack` owned by the runtime, layers pushed/popped by explicit calls, rendered by the runtime after the page.
3. Overlays as ordinary widgets with a `z` field sorted before drawing.

**Evaluation.** (3) does not solve focus trapping, restoration, Esc precedence, or dismissal policy — it solves only paint order, which draw order already solves. (2) is the Radix `DismissableLayer` stack translated: each layer declares its own policy, and the runtime enforces precedence once.

**Recommendation — (2).**

```
Layer {
  id: Id,
  kind: Modal | Popover,                    // Modal traps focus + pointer; Popover traps pointer only
  owner: Id,                                // who opened it: anchor + focus restore target
  anchor: Anchor::{ Screen(Align), Rect(Rect, Side), Point(Position) },
  dismiss: Dismiss { esc: bool, outside_click: bool, focus_out: bool },
  restore_focus: bool,
}
```

Runtime obligations, each replacing something currently duplicated:

| Concern | Now | With the stack |
|---|---|---|
| Stacking / z-order | app draws overlay last (`showcase:797`, `tablepro:2165`, `jackin:2348`) | runtime draws page, then layers bottom-to-top |
| Nesting | only jackin (`app.rs:142`) | native; Scenario F |
| Modality / barriers | one index, `begin_modal` inside render (`ctx.rs:126`) | regions carry `layer: u16`; `hit()` filters `layer == top`; no index juggling |
| Inert background | barrier only; `inert` unused | background layers marked inert: no ring entries, no cursor writes, no hit regions |
| Esc | six widget implementations + three app ladders | `Dismiss.esc` on the top layer, then the screen's ladder |
| Click-outside | "hit returned None" (`showcase:629`) | "hit's layer is below the top layer, or None" — a real outside test |
| Anchoring/flip/clamp | two implementations (`popup.rs:25`, `menu.rs:143`) | one `Anchor` resolver with `Side` + flip + clamp + clip-to-screen |
| Focus restore | three `saved_focus` fields | `restore_focus` + B4 scope memory |
| Cursor | last-writer-wins | top-layer-only (B8) |
| Hints | jackin only (`hintbar.rs` at `jackin:2637`) | the stack contributes the top layer's hint layer automatically (B9) |
| Lifecycle | polled `result` fields | `LayerEvent::{Opened, Dismissed(Reason), Closed(action)}` |

`Dialog`, `Picker`, `ContextMenu`, `Select`'s popup, `Completion`, and jackin's `FileBrowser`/`ChoiceDialog`/`FormDialog`/`InfoDialog`/`HelpOverlay` all become *content* rendered into a layer, not things that push barriers themselves. That removes `begin_modal` from `dialog.rs:373`, `picker.rs:260`, `modals.rs:60`, and removes the shared `"popup.surface"` id (`popup.rs:76`).

Dialog content must be open (goal §14: "Do not preserve a closed dialog architecture in which the reusable dialog understands only a hard-coded list of body types") — today `DialogBody` is a three-variant closed enum (`dialog.rs:18-28`). The layer takes arbitrary composed content; `Dialog::confirm/destructive/prompt/facts` (`dialog.rs:58`, `:75`, `:92`, `:110`) become convenience constructors over the same primitive, as §14 requires.

Small-terminal behaviour becomes a layer property (min size, then clamp, then a documented degradation), replacing three ad-hoc too-small screens' interaction with modal sizing.

**Rejected.** Sorted-`z` widgets (solves paint only). Keeping modality as a render side effect (a modal that does not draw stops trapping — a real hazard at tiny sizes where `Dialog::render` returns early at `dialog.rs:389`). Per-app stacks (already three divergent implementations; jackin's is the only one that nests, and it lives in the wrong crate).

**Risks.** Layer content must be able to borrow app data; keep the render call `FnOnce(&mut Frame-ish)` supplied by the app rather than a boxed `dyn Widget` owned by the runtime. Dismissal policy interacting with capture (B5): a capture must survive an outside-click test.

**Acceptance tests**

- `modal_layer_blocks_hits_on_lower_layers`; `popover_layer_blocks_pointer_but_not_keyboard`.
- `nested_overlay_only_the_top_layer_is_hit_testable_and_focusable` (Scenario F).
- `esc_dismisses_only_the_top_layer`.
- `outside_click_dismisses_a_cancelable_layer_and_not_a_typed_ack_layer` (preserving `jackin:1762-1764`).
- `closing_the_top_layer_restores_focus_to_its_owner`.
- `anchor_flips_above_when_no_room_below_then_clamps_to_the_screen` — port `popup.rs:85-101` and `menu.rs:649-659` onto one resolver.
- `layer_lifecycle_emits_opened_then_closed_with_the_action`.
- TestBackend: `dialog_over_picker_over_page_80x24` snapshot; `backdrop_dims_everything_except_the_footer_row` (`DESIGN.md:537-538`); `two_popups_in_one_frame_have_distinct_ids`.

---

### B8. Cursor ownership

**Options**

1. Status quo: last writer wins, guarded only by `inert`.
2. Priority write: `set_cursor(id, pos)` accepted only from the focused widget of the topmost layer.
3. Pull model: the runtime asks the focused widget for a cursor after layout.

**Evaluation.** (3) needs a second traversal and a widget that can answer outside render. (2) is a one-line rule with a test.

**Recommendation — (2).** `cx.set_cursor(owner_id, pos)` records `(layer, owner, pos)`; the runtime keeps the write only if `layer == top_layer && focus.current() == owner`; otherwise it is dropped and, in debug builds, recorded as a diagnostic. Consequences: a background `TextInput` still flagged `editing` cannot place the cursor under a dialog (today only draw order prevents it, `showcase:797`); and the Picker's query cursor (`picker.rs:336`) is legitimate because the picker's query *is* the focused control of the top layer.

**Rejected.** Last-writer-wins (correct only by accident of draw order). Pull model (extra traversal, and the widget would need geometry outside render).

**Acceptance tests**

- `cursor_write_from_a_lower_layer_is_dropped`.
- `cursor_write_from_an_unfocused_widget_is_dropped`.
- `editing_field_under_a_dialog_does_not_place_the_cursor`.
- TestBackend: `prompt_dialog_places_the_cursor_in_its_field` — assert `Frame::cursor_position` equals the field's text column.

---

### B9. Keymap, action descriptors, and hint metadata

**Options**

1. Status quo: bindings hard-coded in `match key.code`, hints hand-written per screen.
2. A `KeyMap` (chord → action id) consulted by components, overridable per app.
3. Components publish `&[Binding { chord, action, label, when }]` and the runtime both resolves keys and feeds the hint bar.
4. Full command palette / command registry.

**Evaluation.** (4) is a product feature (TablePro and jackin already build their own pickers) and does not belong in the interaction layer. (2) without (3) still leaves hints hand-written — and goal §13 explicitly asks for "contextual action or hint metadata … so the shared HintBar can describe the active component without every screen manually recreating the same key descriptions". (3) subsumes (2).

**Recommendation — (3), plus an explicit key phase.**

- Each component exposes `fn bindings(&self, ctx: BindingCtx) -> &[Binding]` where `Binding { chord: Chord, action: Self::Action, label: &'static str, priority: u8, visible: bool }`. Default binding tables stay terminal-native (arrows + vim keys per `DESIGN.md:601-604`) and remain the current defaults so the approved behaviour is preserved.
- The runtime resolves a key against the focused component's table, after applying an app-supplied `KeyMap` **override layer** (add, remove, remap) so no application's chords are baked into a generic component (goal §13: "application-domain chords must remain outside generic component behavior"). `Dialog`'s `y`/`n` (`dialog.rs:297-311`) becomes an opt-in binding set, not a built-in.
- **Phases.** Replace "app chords always win" (`DESIGN.md:630`, `tablepro:512-516`) with two explicit phases: `Capture` (app claims a chord before the focused component) and `Bubble` (app sees only what the component ignored). Editing controls declare `swallows_typing`, which the runtime uses to skip `Capture` chords that are plain characters — generalising the current ad-hoc `!editing` guards at `tablepro:625`, `:668`, `:672`, `:691`, `:695`, `:708` and `jackin:598`. The known-dead grid `Ctrl+D` (`DESIGN.md:631`) becomes detectable rather than documented.
- **Hints** are derived, not written: `HintBar` takes the top layer's bindings, then the focused component's `visible` bindings sorted by `priority`, then the screen's extra layer, then the global fallback — the existing precedence in `HintBar::resolve` (`hintbar.rs:50-52`), now fed automatically. `Hint` widens from `(&'static str, &'static str)` to reference a `Binding`, so a remapped chord relabels itself.
- **Conflict detection** as an architecture check: a debug-only pass that reports two visible bindings for the same chord in one context.

**Rejected.** Stringly-typed command ids (goal §10 "no stringly typed component internals"); a command registry/palette in the interaction layer (product concern); leaving `EditAction::Apply(fn(&mut TextBuffer))` as a function pointer (goal §19 "Do not restrict extensibility to bare function pointers") — the edit keymap becomes a `Binding` table over an `EditAction` enum, and `TextInput.validator: Option<fn…>` (`input.rs:36`) becomes a boxed closure or an external-validation event.

**Risks.** Binding tables per state (a list in multi-select mode has `a`; single-select does not — `list.rs:152`) means `bindings()` must take state; keep it borrowing `&self` and returning a slice from a small const table chosen by state, so there is no per-frame allocation.

**Acceptance tests**

- `component_bindings_match_handled_keys`: for each component, every key the handler consumes appears in `bindings()` and vice versa (table-driven, catches drift).
- `app_keymap_override_removes_a_component_binding`.
- `capture_phase_chord_is_skipped_while_a_control_is_editing`.
- `conflicting_visible_bindings_are_reported`.
- `hintbar_lists_the_focused_component_bindings_by_priority` and `…drops_from_the_right_with_an_ellipsis` (extend `hintbar.rs:86-113`).
- `keyboard_and_mouse_activation_emit_the_same_action` — the shared conformance test goal §25.2 asks for; today violated by `Picker` (keyboard-only `Delete` secondary, `picker.rs:195`).

---

### B10. Invalidation

**Options**

1. Status quo: `Outcome::Changed` returned all the way up.
2. `Invalidate` as a separate field on `Response` (B2).
3. A frame-dirty flag set through `cx.request_repaint()`.

**Recommendation — (2) as the return channel, plus (3) for out-of-band sources** (timers, animations, background messages) which today force the whole `animating()` machinery to be re-derived in each app (`showcase:308-316`, `tablepro:192-197`, `jackin:299-313`). The runtime keeps a repaint deadline (`cx.request_repaint_after(Duration)`), so the two-speed tick (80 ms animating / 400 ms idle) stops being an app-level `tick_interval()` heuristic (`runtime.rs:27`, `showcase:318-324`).

`Invalidate::Layout` vs `Paint` is worth the extra variant only if layout caching lands; if not, ship `None | Paint` and reserve `Layout`.

**Risks.** A handler that forgets to invalidate produces a silently stale frame — mitigate with a debug assertion that a state-mutating action implies `Invalidate >= Paint`, and with the idempotence test below.

**Acceptance tests**

- `consumed_without_state_change_does_not_repaint`.
- `request_repaint_after_wakes_the_loop_once`.
- `render_twice_with_unchanged_inputs_is_byte_identical` (TestBackend) **and** `…leaves_component_state_equal` — goal §25.2; today this would fail for `Completion` (`completion.rs:172` re-pulls the viewport) and is only accidentally true elsewhere.

---

### B11. Bugs to fold into the refactor (found while auditing)

Each is a defect in current code, independent of the architecture choice; goal §"Bugs" wants the enabling condition removed, not just the symptom.

| # | Defect | Enabling condition | Structural fix |
|---|---|---|---|
| 1 | `Id::of(a).sub(b) == Id::of(ab)` (`id.rs:15-37`) | streaming hash with no separator | B1 separator + kind discriminant |
| 2 | `usize` underflow at `input.rs:433`/`:440` (`area.width as usize - 2`) | no minimum-size contract; render assumes a usable rect | B: measurement floor + a table-driven "every component survives 0×0…3×3" test |
| 3 | All popups share `WidgetId::of("popup.surface")` (`popup.rs:76`) | popup surface is a free function with no owner id | B7: the layer owns its id |
| 4 | Render commits an edit on focus loss (`input.rs:282-286`) | focus change is only observable during render | B4/B7: explicit focus-out event; goal §11 forbids this |
| 5 | Render closes an open select (`select.rs:161-167`) | overlay state lives in the widget with no lifecycle | B7: layer dismissal on `focus_out` policy |
| 6 | Render flips a button's `disabled` (`dialog.rs:465-470`) | ack validation has no event to run on | run on the ack field's `Changed` action |
| 7 | `Completion` wheel undone next frame (`completion.rs:172`) | the wheel/cursor rule is private to `Picker` | B6: shared `ensure_visible_on_next_layout` |
| 8 | `hit_scroll` returns non-scrollable topmost regions (`hit.rs:73-81`, contradicting its doc) | one flat region list, no kind filter | B6: axis + kind on the region |
| 9 | `Split::vertical` and `Split::horizontal` collapse to opposite panes (`layout.rs:116-119` vs `:136-138`) | duplicated arithmetic | one implementation parameterised by axis |
| 10 | `Interaction.focus_hidden` is dead (`ctx.rs:24`; all three apps pass `false`) | no producer | B4: replace with real `focus_visible` |

---

### B12. What Part B leaves in application code (deliberately)

Per goal §12 ("Any remaining manual dispatch in application code must be genuinely domain-specific and explicitly justified"), after B1–B10 the applications should still own, and only own:

- which component owns a given `const Id` when a screen composes several (a match on its own constants);
- domain action mapping at the composition boundary (`.map_action`);
- the Esc *ladder* beyond layer dismissal (`DESIGN.md:606-614` — cancel query → un-maximise → tab strip → explorer is product semantics);
- product chords declared through the `KeyMap` override layer, not embedded in components;
- screen-level effects (`Request`/`Go`/`Msg`).

Everything else currently in `showcase/app.rs`, `tablepro/app.rs` and `jackin_preview/app.rs` under "input" — hover tracking, hover suppression, press/release/activation, flash timing, focus save/restore, focus reconciliation, wheel target lookup, click-outside, double-click, cursor plumbing, modal stacking — is generic and belongs to the runtime, which is what goal §2.9 requires.
