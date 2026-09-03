# COMPONENT_ARCHITECTURE.md

**Status:** Accepted. This document is the single source of truth for the refactor. Builders implement it as written; a change to any *Decision*, *invariant*, exact type, or precedence rule requires a fresh `opus-analyst` adjudication recorded here and in `REFACTORING_STATE.md` (goal §0).

**Authority:** `REFACTORING_GOAL.md` › `DESIGN.md` › existing rendered output/tests › current source. Where the Slice‑1 audits conflict, the adjudications in §3–§15 below are final; the rejected alternative and the reason are stated with each.

**Inputs adjudicated:** `docs/audit/api-audit.md` (API), `docs/audit/app-audit.md` (APP), `docs/audit/domain-boundary-audit.md` (DOM), `docs/audit/interaction-audit.md` (INT), `docs/audit/architecture-research.md` (RES). `docs/audit/performance-audit.md` (PERF) landed after §1–§15 were written; §20.9 folds its obligations in and amends earlier decisions where needed.

Every claim tagged **[F]** is a collected fact carried from an audit with its citation. Everything else is a decision or its rationale.

---

## 1. Current-state diagnosis

### 1.1 What is good and must survive

* **[F]** Frame-rebuilt focus ring in render order, so Tab order is reading order (`src/core/focus.rs:1-29`); last-registered-wins hit registry so overlays shadow (`src/core/hit.rs:1-81`); the `Interaction` snapshot (`src/ui/ctx.rs:17-41`); `ScrollState` as a pure, rendering-free model (`src/core/scroll.rs:8-115`); grapheme/display-width-correct text utilities (`src/core/text.rs`, `src/ui/text.rs`).
* **[F]** No widget spells an RGB literal; all literals live in the private `mod palette` (`src/theme.rs:56-95`) (API §6.1).
* **[F]** `TextViewport` is best-in-class: spans, wrap, bounded retention with selection fixup, drag-select with edge auto-scroll, word select, optional caret (DOM §2.11).
* **[F]** `StatusBar`'s priority-drop layout, `HintBar::resolve`'s layer precedence, `Picker`'s `cursor_dirty` wheel/cursor separation, `DiffView`'s composition over the viewport, and sort-as-permutation in grid/table are all correct designs (DOM §2.7, §2.8, §2.3, §2.5, §6.2).
* **[F]** The Junie token values and the state grammar in `DESIGN.md` are approved and are preserved verbatim as `Theme::junie()`.

### 1.2 The nine structural defects

1. **Identity is a hash with no separator and positional children.** **[F]** `Id::of(a).sub(b) == Id::of(ab)` exactly, not probabilistically (`src/core/id.rs:15-37`; INT A1); `child(index)` is keyed on the *display* index in every collection (`list.rs:83`, `tabs.rs:106-111`, `tree.rs:262-268`, `picker.rs:118`). `Debug` prints only a hex hash (`id.rs:42-46`). Consequences: Scenario E is unreachable; TablePro smuggles a tab index through a display string and parses it back (`tablepro/app.rs:1506`); a close dialog is matched to a tab by scanning `CLOSE_DIALOG.child(i)` (`tablepro/app.rs:1841-1852`).
2. **Nine incompatible reply shapes.** **[F]** `Outcome`, `(Outcome,bool)`, `bool`, `(Outcome,Option<E>)`, `Option<E>`, a polled `Dialog.result` field, a caller-supplied row index, app-local outcome enums, and `result.take()` (INT A2). Redraw and consumption are fused into one enum, so "consumed at a scroll boundary without repaint" (`DESIGN.md:507`) is inexpressible and is honoured by exactly one component (`picker.rs:236-241`) and violated by two (`completion.rs:142-145`, `list.rs:180-183`).
3. **No dispatch.** **[F]** 12 hand-written `owns()`/`locate()` pairs with four different `locate` return types (API §3.5), driving ≥ 66 chained call sites in showcase, ≥ 51 in tablepro, ≥ 13 in the read subset of jackin (APP §2.3). Goal §12 names this chain and requires it removed.
4. **The applications run the interaction machine.** **[F]** Hover, hover suppression, press/release/activation, the 140 ms flash, focus save/restore, focus reconciliation, wheel target lookup, click-outside, double-click and cursor plumbing are implemented three times, divergently (APP §2.5, §2.1; INT A3–A6). Only jackin nests overlays; only jackin consults `primary_focus()` on reconciliation.
5. **Render performs semantic transitions.** **[F]** Nine confirmed violations of goal §11: commit+validate on focus loss (`input.rs:282-286`, `textarea.rs:202-205`, `code.rs:611-614`, `table.rs:566-568`, `grid.rs:1518-1520` — the last one *stages a database mutation from `draw`*), overlay close (`select.rs:161-167`), action arming (`dialog.rs:465-470`), cursor move (`menu.rs:243-248`), durable layout write (`grid.rs:1510`) (API §4). The structural cause is that `render(&mut self, …)` is the only place a widget learns it is focused.
6. **Theme is a flat 30-field `Copy` struct with equality-dispatch resolvers.** **[F]** `lift()` and `backdrop()` branch on colour *equality* against Junie tokens (`theme.rs:362-372`, `:297-321`); `for_level` can only downgrade Junie (`theme.rs:184`) and re-lists all 30 fields in a macro, so a new token is silently skipped; mono collapses accent (mean 126) and error (mean 122) onto the same grey (RES §1.2). Goal §15 scenarios 3–9 are entirely unimplemented; every spacing value and glyph is a literal in ~15 files (RES §1.4).
7. **Surface is threaded by hand.** **[F]** 24 render signatures end in `bg: Color`, obtained via `Panel::bg(t)` and passed down (API §3.6); three components instead hard-code their own plane; `Panel` has a public `bg_override` escape hatch.
8. **Collections own duplicated data and have no renderer hook.** **[F]** Every collection owns `String`s rebuilt per frame (`list.rs:13-17`, `tabs.rs:19-29`, `tablepro/tabs.rs:484-489` clones an entire result set per load); no component accepts a row or cell renderer, so applications paint over the widget's own output afterwards (`tablepro/tabs.rs:1804-1852`, `:2434-2442`) (API §3.3, DOM §5.3).
9. **Domain leakage and closed extension points.** **[F]** `DataGrid` carries SQL nulls/defaults, primary keys, nullability, FK references, a pending-change queue, an undo log, `PreviewSql`, a UUID validator, and buttons labelled "Preview SQL" (DOM §1.1); `DialogBody` is a closed 3-variant enum (`dialog.rs:18-28`); six extension points are bare `fn` pointers that cannot capture (API §3.12); `TextInput`/`Dialog`/`EditState`/`TextBuffer` all derive `Debug`+`Clone` over raw secrets (API §5).

### 1.3 Latent defects folded into the refactor

**[F]** `usize` underflow at `input.rs:433/:440/:418/:412` and `textarea.rs:299` on 1-cell-wide rects; ragged-row indexing panics in `grid.rs:1458/505` and `table.rs:220/294/317/700`; ~20 early returns leave stale geometry that the next click uses (API §7.1–7.2); `dialog.rs:389` returns after `begin_modal()` with zero focus stops and zero hit regions; all popups share `WidgetId::of("popup.surface")` (`ui/popup.rs:76`); `Split::vertical` and `Split::horizontal` collapse to opposite panes (`layout.rs:117` vs `:137`); `Interaction.focus_hidden` is dead (all three apps pass `false`); `fuzzy` returns byte offsets into a lowercased string used to index the original (`ui/text.rs:145-171`); `hit_scroll` returns non-scrollable regions, contradicting its own doc.

---

## 2. Design goals and non-goals

### 2.1 Goals (each maps to an acceptance test in §16)

G1 One component model, one identity model, one reply type, one theme model, one overlay model.
G2 `draw` cannot change semantics — enforced by the type system, not by review.
G3 Application code expresses product intent: no hit regions, no hover/press bookkeeping, no `owns`/`locate`, no manual Tab, no manual modal barriers, no manual cursor placement, no manual focus save/restore.
G4 Borrowed domain data everywhere; zero per-frame allocation on the collection hot path.
G5 Identity survives insert/remove/reorder (Scenario E) and is readable in a debugger and a test failure.
G6 A complete non-Junie theme, partial overrides, family/variant/scope/instance/part/state overrides — all without editing component source, all deterministic, all tested.
G7 Junie output is byte-identical to the reviewed baseline except for the intentional changes listed in §20.10.
G8 The library contains no TablePro or Jackin vocabulary; TablePro keeps every capability through adapters.
G9 A downstream author can build a component with public API only, participating in theme, focus, hover, press, dispatch, hit testing, cursor, scrolling, overlays, capture and tests.
G10 Every current widget module and every app-side reusable control has a recorded disposition (§18).

### 2.2 Non-goals

No virtual DOM, no retained widget tree owned by the runtime, no CSS/class strings, no macro DSL, no plugin ABI, no registry/installer CLI, no general-purpose layout/constraint solver, no async runtime, no spatial focus navigation, no `unsafe`, no compatibility facade, no `'static` requirement anywhere in a component's public surface.

---

## 3. The component model — **Adjudication A**

### 3.1 Decision

**Retained caller-owned state + per-frame borrowed props + an explicit two-phase frame with pre-resolved intents.**

Three layers, fixed for every component:

1. **Durable interaction state** — a concrete, caller-owned `XState` struct (`ListState`, `TextInputState`, `TabsState`, `GridState`, `ViewportState`, …). No lifetimes, no borrowed data, no rendering knowledge; `Debug` (redacting where secret-bearing), `Clone`, `Default`, unit-testable with no buffer and no terminal. Stateless components (`Button`, `Panel`, `Props`, `StatusBar`, `Progress`, `Empty`, `KeyHint`, `Brand`, `Splitter`) have no state struct.
2. **Per-frame props/view** — a short-lived struct borrowing application data (`Button<'a>`, `List<'a, T, K, R>`), built with consuming builders. **The props struct never borrows the state struct**; state is passed to whichever phase needs it. This is the Ratatui `StatefulWidget` ownership shape, and it is what makes (3) possible without two constructors per component.
3. **Two phases with pre-resolved intents.**
   * `fn update(&self, cx: &mut Cx<'_>, st: &mut XState) -> Response<XAction>` — no `Buffer` in scope, callable headless. The **only** place semantics change.
   * `fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &XState) -> Rect` — `&self` on the props, `&XState` shared. Paints, registers regions/ring entries, reports layout facts to the runtime, returns the rect it occupied.

Every component's `draw` takes `&self` and a shared `&XState`, and every renderer closure is `Fn`, never `FnMut`. **Goal §11's render-purity rule therefore becomes a compile error rather than a review item** — all nine violations in §1.2(5) are structurally impossible.

`update` and `draw` run in different runtime passes (§3.3), never fused into a `show()`. There is no `show`.

### 3.2 Rationale, and the rejected alternatives

**Rejected — RES §2.2's immediate-mode `show(ui, area)` = update+draw.** Four disqualifying consequences:

* *It re-admits the defect it is meant to remove.* `show` needs `&mut` state while drawing, so `draw`-time mutation stays legal and §11 remains a review rule. Under the accepted model it is a type error. RES §8.1(1) concedes exactly this ("a reviewer may read `show` as 'render mutates state'") and mitigates with a doc sentence and a test; a compile error is strictly stronger and free.
* *It breaks the three deterministic harnesses.* **[F]** All three do `handle(input)` → `draw()` → assert on the frame *and* on the `Outcome` returned by `handle` (`showcase/app_tests.rs:33-40`, `tablepro/app_tests.rs:44-51`, `jackin/app_tests.rs:30-45`). With `show`, `handle` can only enqueue an intent; the semantic action is not known until the next `draw`, so `handle` cannot return a truthful consumed/changed answer and ~60 assertions across the three suites become unsound. With the accepted model, `update` runs *inside* `handle`, so the return value stays truthful and every existing assertion shape survives.
* *It fights 55K lines of app code.* **[F]** All three apps are already handle/render split; jackin's `Screen` trait has 11 input methods plus `render` (`screens/mod.rs:231-328`), tablepro routes through `Cx`+`Request` (`app.rs:65-84`), showcase through `PageEvent`+`PageCtx`. `update`/`draw` maps 1:1 onto that structure; `show` requires inverting every screen.
* *It makes headless state-machine testing incidental rather than structural.* Goal §25.1 wants edit begin/commit/cancel/focus-loss, traversal, reconciliation and overlay stacking tested without a buffer. `update(cx, st)` is that entry point by construction.

**Rejected — INT Part B's retained app-owned components with runtime-resolved dispatch delivered to `on(Event)`.** Adopted in substance (app-owned values, pre-resolved `Hit{owner,part}`, no `locate`) but rejected in form on two points: (a) a single `on(Event)` method per component recreates the "one giant method" shape and cannot express a *typed* per-phase signature (`update` needs `&mut XState`, `draw` needs `&XState`); (b) `Response{flow,invalidate,action}` with a `handle`/`render` split leaves render's `&mut self` in place, so §1.2(5) survives. The accepted model keeps INT's dispatch, its `Response` fields (§6), and its overlay/focus/capture services, and replaces `on(Event)` with `update`, and `render(&mut self)` with `draw(&self)`.

**Rejected — a runtime-owned component tree (tuirealm/cursive shape).** Forces `'static` + `Box<dyn>` + interior mutability, which kills borrowed domain rows (Scenario D) and makes two sibling `&mut` widgets impossible.

**Rejected — a universal `Widget` trait.** Goal §5 forbids it; components differ in whether they take state, a model, or child slots. Uniformity is achieved by *naming and signature conventions* (§13), enforced by an architecture check, not by a trait.

**Rejected — generic theme parameters / trait-object components as the primary model.** Goal §10; they spread generics into application signatures or box every node per frame.

### 3.3 The exact runtime frame sequence

`Input` never touches a component directly. `Runtime<A>` owns all interaction state; the app owns only domain state and `XState`s.

```
Runtime::handle(&mut self, input: Input) -> Response<()>
 ── INPUT PHASE (no buffer in scope) ─────────────────────────────────────────
  1. normalize        Input from crossterm; drop key releases and unmapped buttons.
                      Resize -> record size, invalidate = Layout, no intents.
                      Tick   -> advance flash/motion clocks, then step 7 (app.tick).
  2. capture keymap   app KeyMap "Capture" bindings are matched FIRST, but a chord
                      that is a bare Char is skipped while `focus_owner_swallows_typing`
                      (§11.4). A capture hit produces an app action and skips 3-8.
  3. resolve          against LAST FRAME's Registry and FocusRing (never a fresh scan
                      of the app tree):
                        pointer -> Registry::hit(pos, top_layer) -> Hit{owner,part,layer,local}
                                   (a live Capture short-circuits this: the capture owner
                                    receives Drag/Release with `local` against the captured area)
                        wheel   -> Registry::hit_scroll(pos, axis) -> innermost scrollable
                                   handling that axis, returned even at zero headroom
                        key     -> FocusState::current()  (None -> app bubble phase only)
                        paste   -> the focused owner iff it declared EDITING
  4. interaction      hover / hover_suppressed / press bookkeeping / 140 ms flash /
                      double-click window / capture claim+release. All of §1.2(4).
  5. focus policy     Tab, Shift+Tab, Esc-to-dismiss-top-layer and press-focuses-owner
                      are executed by the runtime against the last ring, honouring
                      focus scopes and traps (§8). Focus changes here are staged, not
                      applied mid-queue.
  6. enqueue          Intents into `Intents`, keyed by owner Id. A staged focus change
                      enqueues Intent::FocusOut{to} to the old owner and
                      Intent::FocusIn{via} to the new one, in that order.
  7. app.update       app.update(&mut cx) -> Response<()>; screens call component
                      `update`s, which drain their own intent bucket by Id.
                      If the pass changed focus (cx.focus(id), a dismissed layer, a
                      reconciliation), re-run 6-7. Bounded at 4 passes; the 5th is a
                      debug_assert and a dropped transition (test: focus_transition_settles).
  8. bubble keymap    keys still unconsumed are offered to the app KeyMap "Bubble"
                      bindings, then to the screen's Esc ladder.
  9. finish           Intents that no component drained are dropped and counted as a
                      debug diagnostic (`UndeliveredIntent`); returns Response<()> whose
                      `flow` and `invalidate` are the fold of everything above.

Runtime::draw(&mut self, frame: &mut Frame)
 ── DRAW PHASE (no &mut app state in scope) ──────────────────────────────────
 10. new frame state  Registry::new(gen+1), FocusRing::new(), FrameOut::new(),
                      layer buffer pool reset, StyleStack reset to the theme.
 11. app.draw         app.draw(&mut ui) with layer 0 painting straight into the frame
                      buffer. Registrations carry {owner, part, area, layer, kind, gen}.
 12. layers           ui.layer(id, spec, |ui, area| …) executes immediately but paints
                      into a pooled layer buffer with a written-cell bitset and pushes a
                      focus scope + a hit layer; the runtime composites layers
                      bottom-to-top after app.draw returns, so z-order is the layer
                      order, NOT the call order.
 13. registry swap    last = new. Captures whose owner or area vanished are released.
                      Stale intents from an older generation are dropped.
 14. focus reconcile  if FocusState::current is absent from the new ring:
                        (a) nearest surviving entry in the same scope by previous index,
                        (b) else that scope's first enabled entry,
                        (c) else the innermost active scope's first enabled entry,
                        (d) else None.
                      `focus_visible` is true iff the last input was a key.
 15. cursor           the single retained cursor write is kept iff
                      `layer == top_layer && FocusState::current() == owner`; otherwise
                      dropped (debug diagnostic). Then frame.set_cursor_position or hide.
```

Latency: pointer intents resolve against the registry the user actually saw, which is the current behaviour (**[F]** all three apps rebuild hits/ring after drawing: `showcase/app.rs:711-721`, `tablepro/app.rs:2094-2105`, `jackin/app.rs:2254-2265`). A component drawn for the first time this frame is not clickable until it has been drawn once — identical to today, and documented.

`§25.6` compliance: pointer/wheel cost is one reverse scan of the top layer's regions; key cost is one hash lookup; per-component cost is one `Intents::take(id)` hash probe. No tree walk, no per-event data scan.

### 3.4 A jackin `Screen` trait method after migration

Before — **[F]** 20 methods, 11 of them input (`screens/mod.rs:231-328`), plus `Cx{focus,ring,requests}` handing every screen `&mut Focus`.

After — 6 methods; `Cx` no longer carries `Focus` or `FocusRing`:

```rust
pub trait Screen {
    /// The only input entry point. Intents are already resolved to (owner, part).
    fn update(&mut self, cx: &mut Cx<'_>, w: &mut World) -> Response<()>;
    /// Pure paint. `&self` makes a semantic mutation a compile error.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, w: &World);
    /// Product-level hints only; component bindings are contributed automatically (§11.4).
    fn hints(&self, _cx: &HintCtx<'_>, _w: &World) -> HintLayer { HintLayer::empty() }
    fn crumb(&self, w: &World) -> String;
    fn primary_focus(&self) -> Option<Id>;
    fn on_esc_top(&mut self, cx: &mut Cx<'_>, _w: &mut World) -> Response<()> {
        cx.go(Go::Manager); Response::consumed().repaint()
    }
}
```

`on_click`, `on_double_click`, `on_drag`, `on_secondary`, `on_press`, `on_release`, `on_wheel`, `on_paste`, `on_tick`, `on_msg`, `on_modal`, `picker_items`, `form_changed`, `strip_right`, `is_editing`, `animating`, `enter` are removed or subsumed: pointer/wheel/paste/focus arrive as intents inside `update`; `on_modal` becomes `LayerEvent` intents (§9.5); `is_editing`/`animating` become `cx.request_repaint_after(…)` and `StateFlags::EDITING` on the focused owner; `enter` becomes a `LayerEvent::Opened`/route-change intent; `picker_items`/`form_changed` become ordinary `update` work on the screen's own state.

A concrete migrated body (jackin Manager, condensed):

```rust
fn update(&mut self, cx: &mut Cx<'_>, w: &mut World) -> Response<()> {
    let mut r = Response::ignored();

    r |= Self::tree(&self.rows)                          // props built from data fields only
        .update(cx, &mut self.tree_state)
        .map_action(|a| match a {
            TreeAction::Activated(k) => self.open(k, w, cx),
            TreeAction::Expanded(k)  => self.expand(k, w),
            _ => {}
        });

    r |= Self::detail_button().update(cx).on_activated(|| cx.go(Go::Editor(self.current)));

    if let Some(ev) = cx.layer_event(LAUNCH_PICKER) {    // nested overlay lifecycle
        if let LayerEvent::Closed(PickerAction::Chosen(k)) = ev { self.launch(k, w, cx); }
    }
    r
}
```

There is no `if f == self.x.id` ladder, no `locate`, no `focus.focus(...)`, no `was_focused` flag, no scrollbar id wiring, and no `ctx.hits.register` — all of which appear in the current body (**[F]** APP §2.1–2.5).

---

## 4. State ownership rules

The seven concerns of goal §11 have exactly one home each.

| # | Concern | Owner | Type / access |
|---|---|---|---|
| 1 | Application-domain data and effects | the application | app structs; reaches components as `&'a [T]` + accessor closures, never copied |
| 2 | Component configuration / props | frame-local, caller-built | `X<'a>` consuming builders; dropped at the end of the phase |
| 3 | Durable component interaction state | the caller | `XState` stored in the app struct; `&mut` in `update`, `&` in `draw` |
| 4 | Controlled values and selections | the caller | `&'a mut String` / `&'a mut T` passed to `update`; `&'a str` / `&'a T` to `draw` |
| 5 | Frame-local layout and geometry | the **runtime** | `Registry` regions + `FrameOut::layout`; never a public field on a component |
| 6 | Theme and resolved visual state | the runtime `Ui`/`Cx` | `Theme` + `StyleStack` + `Surface`; `Resolved` is produced per query |
| 7 | Semantic actions to the application | the return channel | `Response<XAction>` (§6) |

**Invariants (each a test in §16):**

* **S1** No component type has a public field. Geometry is never public (kills the 21 `pub area` + 3 `pub areas` sites, **[F]** API §3.8) and never survives a frame in app state.
* **S2** `XState` contains no `Rect`, no `Color`, no `Style`, no borrowed data, and is `Debug + Clone + Default + PartialEq`. `PartialEq` is what makes "draw twice leaves state equal" checkable.
* **S3** A component reads geometry only from `cx.area(id)` / `cx.layout(id)`, which return **last frame's** facts, or `Option::None` on the first frame. First-frame absence is a documented no-op path, never a panic.
* **S4** Controlled is the default: the value lives in the caller; the in-flight draft lives in `XState` and is written back only on `commit`. Uncontrolled (`XState` owns the `String`) exists only for throwaway fields and is documented per component.
* **S5** External data change is reconciled explicitly by the component's `update`, before any action is emitted: `st.reconcile(keys)` (§12.4). No component silently retargets an edit after a reorder.
* **S6** Draw-time facts flow *up* to the runtime, never sideways into app state: `ui.report_layout(id, LayoutFacts { viewport_len, content_len, rows, cols })`. This is how "update non-semantic viewport metadata required by the current frame" (goal §11) is satisfied while `draw` stays `&self`.

---

## 5. Rendering rules

**R1** `draw` signature is fixed. Leaf: `fn draw(&self, ui: &mut Ui<'_>, area: Rect[, st: &XState]) -> Rect` (returns the rect actually occupied, for decoration and chaining). Container: `fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> R`. No other shapes.

**R2** `draw` may: write cells through `Ui` painting methods; register hit/scroll/focus regions; report layout facts; report a cursor request; push style scopes, surfaces, focus scopes and layers. It may not mutate the app, the state, or the theme — enforced by `&self`/`&XState`/`&Theme`.

**R3** All painting goes through `Ui` (`ui.paint_cell`, `ui.paint_str`, `ui.fill`, `ui.rule`, `ui.frame`, `ui.glyph`), so a layer's written-cell bitset is always correct. `ui.raw() -> (&mut Buffer, Rect)` is the documented escape hatch; it marks the whole rect written and is the only way to reach the buffer.

**R4** Clipping is automatic: `Ui` carries a clip rect, intersected on every `with_area`/container entry. A component cannot write outside its area. (**[F]** kills `grid.rs:1637`, `table.rs:639`, `panel.rs:117-122`, `chips.rs:193`.)

**R5** Degenerate rects are safe by contract: every component must render correctly into `0×0 … 3×3`. All width arithmetic uses `saturating_sub`. A component that cannot draw returns early **after** registering nothing; because the registry is rebuilt per frame there is no stale geometry (this also removes the ~20 stale-geometry sites, **[F]** API §7.2). A modal layer that cannot draw still traps focus, because the trap belongs to the layer, not to the draw (fixes `dialog.rs:389`).

**R6** Draw is idempotent: drawing the same props+state twice produces a byte-identical buffer, an identical registry, and an equal state. Test `conformance::draw_twice_is_byte_identical` + `…_leaves_state_equal`.

**R7** Layer painting is deferred-composited (§3.3 step 12). A popup no longer has to be "drawn last" by the caller (**[F]** `DESIGN.md:749`); this is an intentional behaviour improvement recorded in §20.10.

---

## 6. Event and semantic-action model — **Adjudication C**

### 6.1 Decision — one exact type set

```rust
// ---------- raw input (unchanged in spirit; `Input` is the runtime boundary) ----------
pub enum Input { Key(Key), Mouse(Mouse), Resize(u16, u16), Paste(String), Tick }
pub struct Key { pub code: KeyCode, pub mods: KeyModifiers }
pub struct Chord { pub code: KeyCode, pub mods: KeyModifiers }   // Eq+Hash, for KeyMap
pub struct Mouse { pub kind: MouseKind, pub pos: Position, pub mods: KeyModifiers } // +mods (was missing)
pub enum MouseKind { Move, Down, Up, Drag, Secondary, SecondaryUp, Wheel(Axis, i16) }
pub enum Axis { V, H }

// ---------- what a component actually receives ----------
pub struct PartRef { pub part: Part, pub item: Option<ItemKey> }

pub enum Intent<'f> {
    Key(Key),
    Paste(&'f str),
    Pointer { phase: Phase, part: PartRef, pos: Position, local: Position, mods: KeyModifiers },
    Wheel   { axis: Axis, delta: i16, part: PartRef, pos: Position },
    FocusIn { via: FocusVia },
    FocusOut { to: Option<Id> },
    Layer(LayerEvent),
    Cancel,                       // Esc reached this owner after layer dismissal
}
pub enum Phase { Press, Release, Click, DoubleClick, Secondary, DragStart, Drag, DragEnd }
pub enum FocusVia { Keyboard, Pointer, Programmatic, Restore }

// ---------- the one reply type ----------
pub enum Flow { Ignored, Consumed }
pub enum Invalidate { None, Paint, Layout }        // ordered: None < Paint < Layout

#[must_use]
pub struct Response<A = ()> {
    id: Id,
    flow: Flow,
    invalidate: Invalidate,
    state: StateFlags,
    action: Option<A>,
}

impl<A> Response<A> {
    // construction
    pub fn ignored() -> Self;
    pub fn consumed() -> Self;                       // flow=Consumed, invalidate=None
    pub fn changed() -> Self;                        // flow=Consumed, invalidate=Paint
    pub fn action(a: A) -> Self;                     // flow=Consumed, invalidate=Paint, action=Some
    pub fn for_id(self, id: Id) -> Self;
    pub fn with_state(self, s: StateFlags) -> Self;
    pub fn repaint(self) -> Self;                    // raise to >= Paint
    pub fn relayout(self) -> Self;                   // raise to Layout
    pub fn no_repaint(self) -> Self;                 // lower to None (the boundary-wheel rule)
    // reading
    pub fn id(&self) -> Id;
    pub fn flow(&self) -> Flow;
    pub fn consumed(&self) -> bool;
    pub fn invalidate(&self) -> Invalidate;
    pub fn changed(&self) -> bool;                   // invalidate >= Paint
    pub fn state(&self) -> StateFlags;
    pub fn focused(&self) -> bool; pub fn hovered(&self) -> bool; pub fn pressed(&self) -> bool;
    pub fn action_ref(&self) -> Option<&A>;
    pub fn take_action(&mut self) -> Option<A>;
    pub fn into_action(self) -> Option<A>;
    // composition
    pub fn map_action<B>(self, f: impl FnOnce(A) -> B) -> Response<B>;
    pub fn on_action(self, f: impl FnOnce(A)) -> Response<()>;
    pub fn erase(self) -> Response<()>;              // drops the action, keeps flow+invalidate
}
impl Response<Activated> { pub fn activated(&self) -> bool; pub fn on_activated(self, f: impl FnOnce()) -> Response<()>; }
impl<A> std::ops::BitOr for Response<A>   { /* flow: Consumed wins; invalidate: max; action: first Some */ }
impl<A> std::ops::BitOrAssign for Response<A> {}
```

`StateFlags` (16 bits, `bitflags`): `FOCUSED, FOCUS_VISIBLE, HOVERED, PRESSED, SELECTED, ACTIVE, CHECKED, DISABLED, READ_ONLY, ERROR, WARNING, BUSY, EDITING, DIRTY, EXPANDED, LOADING`.

Per-component actions are small, `Copy` where possible, and **carry `ItemKey`, never `usize`**:

```rust
pub struct Activated;                                         // Button, MenuItem, Chip activation
pub enum TextAction   { Changed, Committed, Cancelled, MoveNext, MovePrev }
pub enum ListAction   { Moved, Chose(ItemKey), Toggled(ItemKey), Activated(ItemKey), ToggledAll }
pub enum TreeAction   { Moved, Expanded(ItemKey), Collapsed(ItemKey), Chose(ItemKey), Activated(ItemKey) }
pub enum TabsAction   { Activated(ItemKey), Close(ItemKey), New }
pub enum GridAction   { Moved, Activated(ItemKey), Sort(ColumnKey, SortDir), ClearSort,
                        FilterOnCell(ItemKey, ColumnKey), OpenFilters, ClearFilters, Refresh,
                        FetchMore, Copy(String), EditRequested(ItemKey, ColumnKey),
                        RowAddRequested { after: Option<ItemKey>, duplicate: bool },
                        RowRemoveRequested(Vec<ItemKey>), CellAction(ItemKey, ColumnKey, ActionKey),
                        LeaveForward, LeaveBackward }
pub enum PickerAction { Chosen(ItemKey), ChosenAlt(ItemKey), Secondary(ItemKey), Back, Scope(ScopeKey), QueryChanged }
pub enum DialogAction { Action(ActionKey), Dismissed(DismissReason) }
pub enum ViewportAction { Copy(String), SelectionChanged, FollowChanged(bool) }
```

Application-domain actions never appear in a component; the screen translates at the composition boundary with `map_action` (the home of today's `FilterOutcome`, `PageEvent → Request`).

### 6.2 Rationale and rejections

* `flow` and `invalidate` are orthogonal because **[F]** `DESIGN.md:507` requires "a wheel at a boundary is consumed without a repaint", which `Outcome` cannot express and which two components get wrong today.
* **Rejected — a single flat enum** (`Reply { Ignored, Consumed, Changed, Action(A) }`): cannot say consumed + action + repaint simultaneously, which is precisely why `picker.rs:147` returns a tuple.
* **Rejected — `Box<dyn Any>` message bus:** unmatched, untyped, forces `'static` (goal §10).
* **Rejected — polled result fields** (`Dialog.result`): the reason all three apps re-check `if d.result.is_some()` after every key and click (**[F]** `showcase:455`, `tablepro:1447`, `jackin:1342`). Dialogs return `Response<DialogAction>`; the overlay stack converts a terminal action into close + focus restore.
* **Rejected — RES's `Response{id,state,area,action,changed}`:** `area` is meaningless in a phase with no layout, and a single `changed: bool` cannot carry the boundary rule. `area` is available as `cx.area(id)` or as `draw`'s return value; `changed` is derived from `invalidate`.

---

## 7. Component identity and stable keys — **Adjudication B**

### 7.1 Decision — one unified spec (INT B1 ∧ RES `id!`)

```rust
#[derive(Clone, Copy)]
pub struct Id {
    hash: u64,
    #[cfg(debug_assertions)] label: DebugLabel,     // 0 bytes in release
}
impl PartialEq for Id { fn eq(&self, o: &Self) -> bool { self.hash == o.hash } }
impl Eq for Id {}
impl Hash for Id { /* hashes `hash` only */ }
impl Ord for Id  { /* orders by `hash` only */ }

#[cfg(debug_assertions)]
#[derive(Clone, Copy)]
struct DebugLabel { root: &'static str, tail: Tail }
#[cfg(debug_assertions)]
enum Tail { Root, Sub(&'static str), Part(Part), Item(ItemKey), Index(usize) }

impl Id {
    pub const fn root(path: &'static str) -> Id;          // Kind::Name
    pub const fn sub(self, name: &'static str) -> Id;     // Kind::Name
    pub const fn part(self, p: Part) -> Id;               // Kind::Part
    pub const fn index(self, i: usize) -> Id;             // Kind::Index — UNSTABLE under reorder
    pub fn item(self, k: ItemKey) -> Id;                  // Kind::Item
    pub fn hash(self) -> u64;                             // for registries and tests
}
#[macro_export]
macro_rules! id { ($p:literal) => { $crate::Id::root(concat!(module_path!(), "::", $p)) } }
```

**Hashing (exact).** FNV‑1a over `u64`. Every segment is mixed as: `0xFF` separator byte, then the one-byte kind discriminant (`Name=1, Part=2, Item=3, Index=4`), then the payload bytes (`name.as_bytes()`, `part.0.to_le_bytes()`, `key.to_le_bytes()` with a per-variant tag byte, `i.to_le_bytes()`). This makes `Id::root("a").sub("b") != Id::root("ab")` an identity, not a probability, and keeps every derivation `const fn` except `item` (which takes a runtime key).

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ItemKey { Index(usize), Num(u64), Text(u64) }
impl ItemKey {
    pub const fn index(i: usize) -> Self;   // documented: UNSTABLE under insert/remove/reorder
    pub const fn num(n: u64) -> Self;
    pub fn text(s: &str) -> Self;           // FNV-1a of the bytes, kind-tagged
    pub fn pair(a: u64, b: u64) -> Self;    // composite keys (schema+table, provider+account)
}
```

**Typed parts.**

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct Part(u16);
impl Part {
    // 0..=255 reserved for the library; `custom` maps into 0x8000..=0xFFFF by FNV of the name.
    pub const CONTAINER: Part; pub const BORDER: Part; pub const BACKDROP: Part;
    pub const GUTTER: Part;    pub const MARKER: Part; pub const ICON: Part;
    pub const LABEL: Part;     pub const META: Part;   pub const HELP: Part;
    pub const TITLE: Part;     pub const BODY: Part;   pub const ACTIONS: Part;
    pub const FIELD: Part;     pub const TEXT: Part;   pub const PLACEHOLDER: Part;
    pub const ROW: Part;       pub const CELL: Part;   pub const HEADER: Part;
    pub const TRACK: Part;     pub const THUMB: Part;  pub const RULE: Part;
    pub const TAB: Part;       pub const CLOSE: Part;  pub const PREFIX: Part;
    pub const BADGE: Part;     pub const OVERFLOW: Part; pub const NEW: Part;
    pub const EMPTY: Part;     pub const QUERY: Part;  pub const SEAM: Part;
    pub const SUMMARY: Part;   pub const DETAIL: Part; pub const KEY: Part; pub const ACTION: Part;
    pub const fn custom(name: &'static str) -> Part;
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct PartRef { pub part: Part, pub item: Option<ItemKey> }
```

`PartRef` is stored **directly** in each registry region — 24 bytes, `Copy`. INT B3's `{part_kind:u16, slot:u16}` token plus a per-frame key side table is **rejected as unnecessary complexity**; there is no packing, no round-trip risk, and no per-frame table.

**Debuggability.** `Debug for Id` prints `orders.list ▸ Row(#0x1f3a)` in debug builds and `Id(3f9a…)` in release. The runtime also maintains `Registry::names: HashMap<u64, DebugLabel>` populated at registration, so a bare id from a diagnostic resolves. `PartialEq`/`Hash`/`Ord` ignore the label, so debug and release compare identically (test `id_equality_ignores_debug_label`).

**Collision safety.** `Registry::register` records a duplicate as a `Diagnostic::DuplicateId { id, first, second }`, surfaced through `Runtime::diagnostics()` in debug builds and asserted by tests. Never a panic in release (goal §10).

**Where ids come from.** Components: one `Id` per component instance, supplied by the caller as a `const` (`const SAVE: Id = id!("save");`) or derived (`self.id.item(key)`). Children are **never** addressed by a derived id in application code: they are addressed by `PartRef`, which the runtime resolves. `Id::index` exists for genuinely positional cases and is rejected by a debug assertion when a keyed collection's length changes while `Index` keys are live.

**Test addressing** (contract from **[F]** APP §6): `Runtime::area_of(Id) -> Option<Rect>`, `Runtime::area_of_part(Id, PartRef) -> Option<Rect>`, `Runtime::ring() -> &FocusRing`, `Runtime::focus() -> Option<Id>`. App-level `const Id`s stay public within the app crate, so `screens::accounts::FORM.part(Part::custom("save"))` replaces `FORM.sub("save")` with the same reach.

### 7.2 Rejections

Interned path ids (allocation + a global on the render path; ids stop being `const`, breaking `const NAV`). Generational handles (unusable in `const` and in tests that address a control before it is first drawn). Source-location ids, egui-style (unstable under reorder, invisible in tests). Keeping raw indices as the only child key (fails Scenario E; already forces the index-through-a-display-string hack).

---

## 8. Focus and interaction model — **Adjudication E (B4, B5, B6, B8, B10)**

### 8.1 Focus scopes and traps (confirms INT B4)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct ScopeId(Id);
#[derive(Clone, Copy, PartialEq, Eq)]       pub enum ScopeMode { Normal, Trap }

pub struct FocusEntry { pub id: Id, pub scope: ScopeId, pub disabled: bool,
                        pub area: Rect, pub layer: LayerId, pub swallows_typing: bool }
pub struct FocusRing  { entries: Vec<FocusEntry>, scopes: Vec<ScopeRecord> }
impl FocusRing {
    pub fn entries(&self) -> &[FocusEntry];
    pub fn reachable(&self) -> impl Iterator<Item = &FocusEntry> + '_;   // enabled ∧ innermost trap
    pub fn contains(&self, id: Id) -> bool;
    pub fn next(&self, from: Option<Id>) -> Option<Id>;
    pub fn prev(&self, from: Option<Id>) -> Option<Id>;
}
pub struct FocusState { current: Option<Id>, visible: bool, restore: HashMap<ScopeId, Id> }
impl FocusState {
    pub fn current(&self) -> Option<Id>;
    pub fn visible(&self) -> bool;                 // focus-visible: true iff last input was a key
    pub fn vis(&self, id: Id) -> FocusVis;         // None | Focused | FocusedVisible
}
// registration, from draw:
impl Ui<'_> {
    pub fn focus_scope<R>(&mut self, id: Id, mode: ScopeMode, f: impl FnOnce(&mut Ui) -> R) -> R;
    pub fn register_control(&mut self, id: Id, area: Rect, f: Focusability);
}
pub enum Focusability { Focusable, FocusableReadOnly, Disabled, ClickOnly }
```

**Invariants.** Disabled controls are **recorded** in the ring with `disabled: true` (so a test can assert "registered but skipped") and are excluded from traversal, preserving `DESIGN.md:656`. `Trap` confines Tab/Shift+Tab to the scope and its descendants and wraps inside it. Scopes nest; a modal layer opens its trap **when the layer is pushed**, not when it draws, so a modal that fails to draw still traps (fixes `dialog.rs:389`). Restoration is runtime-owned via `restore: ScopeId → Id`, deleting `saved_focus` from all three apps. Reconciliation follows the (a)(b)(c)(d) rule in §3.3 step 14 — replacing three divergent app fixups. Read-only controls stay in the ring; disabled controls do not.

**Rejected:** spatial/directional focus (unrequested; `DESIGN.md:601` specifies reading order); trap-on-render; app-owned restoration.

### 8.2 Pointer capture (confirms INT B5)

```rust
pub struct Capture { pub owner: Id, pub part: PartRef, pub origin: Position,
                     pub area: Rect, pub gen: u32 }
impl Cx<'_> {
    pub fn capture(&mut self, part: PartRef) -> bool;   // claim; false if another capture is live
    pub fn release_capture(&mut self);
    pub fn capture_origin(&self) -> Option<Position>;
    pub fn capture_area(&self) -> Option<Rect>;
}
```

While a capture is live: every `Drag`/`Release` goes to the capturing owner with `local` computed against the **captured** `area`; hit-testing for other widgets is suppressed; `StateFlags::PRESSED` stays set regardless of hover; release activates iff the pointer is inside the captured area. This deletes the cached-rect reconstruction in `list.rs:195-205`, `tree.rs:448-458`, `viewport.rs:519-530` and the caller-supplied container in `splitter.rs:56`, and removes the reason `scrollbar.rs:32` and `splitter.rs:41` bypass `Interaction::pressed`. Captures are released on resize, on owner disappearance, and on generation mismatch. Nested captures are rejected, never stacked.

### 8.3 Wheel routing (confirms INT B6)

```rust
pub struct Headroom { pub up: u16, pub down: u16, pub left: u16, pub right: u16 }
pub enum Axes { V, H, Both }
impl Ui<'_> { pub fn register_scroll(&mut self, id: Id, area: Rect, axes: Axes, head: Headroom); }
impl Registry { pub fn hit_scroll(&self, pos: Position, axis: Axis) -> Option<Hit>; }
```

Innermost scrollable that covers the point **and** handles the axis wins (innermost = registered last, i.e. draw order), **returned even at zero headroom** so the event is consumed without repaint instead of chaining outward (`DESIGN.md:507`). Wheel step is `design.motion.wheel_rows` (3), resolved by the runtime, not a literal in three apps. Horizontal wheel is uniformly supported. Wheel never moves focus or the cursor; `ScrollState::ensure_visible_on_next_layout` is set only by cursor motion, generalising `Picker`'s private `cursor_dirty` and fixing `completion.rs:172`.

**Rejected:** outward chaining at a boundary; focus-follows-wheel.

### 8.4 Cursor ownership (confirms INT B8)

`ui.set_cursor(owner: Id, pos: Position)` records `(layer, owner, pos)`. The runtime keeps the write iff `layer == top_layer && FocusState::current() == owner`; otherwise it drops it and records `Diagnostic::CursorRejected`. A background `TextInput` still flagged `EDITING` can never place the cursor under a dialog (today only draw order prevents it).

### 8.5 Invalidation (confirms INT B10)

`Invalidate` on `Response` is the return channel. Out-of-band sources use `cx.request_repaint()` and `cx.request_repaint_after(Duration)`, which the runtime folds into a repaint deadline — deleting the per-app `animating()`/`tick_interval()` heuristics (**[F]** `showcase:308-324`, `tablepro:192-197`, `jackin:299-320`). Cadence values (`tick_ms` 80, `idle_tick_ms` 400, `press_flash_ms` 140, `status_ms` 4000/5000) are design tokens. `Invalidate::Layout` ships from day one but currently behaves as `Paint`; it is reserved for layout caching and is asserted only for ordering.

### 8.6 Hover, press, activation, double-click, click-outside — runtime-owned

Hover never changes focus. Hover is suppressed after any key press until the pointer moves (`DESIGN.md:648`). Press records the target; release activates only on the same target (or inside a capture area). Keyboard and mouse activation produce the identical action (conformance test). Double-click is a 500 ms same-target window, owned by the runtime, delivered as `Phase::DoubleClick`; the `was_focused` argument threaded into `TextInput::on_click` disappears (**[F]** `input.rs:247-261` and five app call sites). "Click outside" is `hit.layer < top_layer || hit.is_none()` — a real outside test rather than "the hit returned None".

---

## 9. Overlay and modal model — **Adjudication E (B7)**

### 9.1 Decision — a runtime-owned layer stack

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] pub struct LayerId(u16);   // 0 = page

pub enum LayerKind { Modal, Popover, Tooltip }        // Modal: focus+pointer trap; Popover: pointer only
pub enum Anchor {
    Screen(Align),                                     // Align::{Center, UpperThird, Bottom, …}
    Rect { rect: Rect, side: Side, align: CrossAlign }, // Side::{Below, Above, Left, Right}
    Point(Position),
}
pub struct Dismiss { pub esc: bool, pub outside_click: bool, pub focus_out: bool }
pub struct LayerSpec {
    pub kind: LayerKind,
    pub owner: Id,                 // anchor owner + focus-restore target
    pub anchor: Anchor,
    pub dismiss: Dismiss,
    pub restore_focus: bool,
    pub initial_focus: Option<Id>,
    pub min_size: Size,
    pub backdrop: Backdrop,        // None | Dim { exclude_footer: bool }
    pub inert_below: bool,         // Modal: true
}
pub enum LayerEvent { Opened, Dismissed(DismissReason), Closed(ActionKey) }
pub enum DismissReason { Esc, OutsideClick, FocusOut, Programmatic }

impl Cx<'_> {                                   // opened / closed from `update`
    pub fn open_layer(&mut self, id: Id, spec: LayerSpec);
    pub fn close_layer(&mut self, id: Id, with: Option<ActionKey>);
    pub fn layer_event(&mut self, id: Id) -> Option<LayerEvent>;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
}
impl Ui<'_> {                                   // content is drawn from `draw`
    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>;
}
```

The stack is runtime state; the **content** is drawn by the app inside `ui.layer(...)`, so layer content borrows app data freely and nothing is boxed or `'static`. Placement, flip, clamp and clip are one resolver (`Anchor` + `Side` + `CrossAlign`), replacing the two independent `Placement` enums and algorithms (`ui/popup.rs:25-56`, `menu.rs:143-171`) and `Rect::centered` in `dialog.rs:376`. The backdrop dim is one implementation (replacing three byte-identical loops) and it excludes the footer row (`DESIGN.md:537`).

| Concern | Now | With the stack |
|---|---|---|
| z-order | app draws the overlay last, three ways | layer buffers composited bottom→top (§3.3 step 12) |
| nesting | jackin only | native; Scenario F |
| barriers | one index in each registry, set from `render` | regions carry `layer`; `hit()` filters `layer == top` |
| inert background | unused flag | `inert_below`: no ring entries, no hit regions, no cursor writes |
| Esc | 6 widget impls + 3 app ladders | `Dismiss.esc` on the top layer, then the screen's ladder |
| click-outside | "hit returned None" | "hit's layer < top layer, or None" |
| focus restore | 3 `saved_focus` fields | `restore_focus` + scope memory (§8.1) |
| cursor | last writer wins | top-layer + focused owner only (§8.4) |
| hints | jackin only | the top layer contributes its hint layer automatically (§11.4) |
| lifecycle | polled `result` fields | `LayerEvent::{Opened, Dismissed, Closed}` |
| small terminal | 3 ad-hoc screens | `min_size`, then clamp, then documented degradation |

`Dialog`, `Picker`, `ContextMenu`, `MenuBar` dropdowns, `Select`'s popup, `Completion`, and jackin's `FileBrowser`/`ChoiceDialog`/`FormDialog`/`InfoDialog`/`HelpOverlay` all become **content rendered into a layer**. `begin_modal` and the shared `"popup.surface"` id are deleted, and with them the six blocks of manual hit re-registration (**[F]** APP §2.2).

**Rejected:** sorted-`z` widgets (solves paint only); modality as a render side effect; per-app stacks.

### 9.2 Dialog content is open

`DialogBody` is deleted. `Dialog::show`-equivalent takes a body slot; `confirm`, `destructive`, `prompt`, `acknowledge`, `facts`, `choice`, `info` are convenience constructors over the same primitive and the same rendering path (goal §14). Action arming is a predicate evaluated in `update`, never a `disabled` flag flipped during draw.

---

## 10. Layout, measurement and surface inheritance

```rust
pub struct Size { pub min: (u16, u16), pub preferred: (u16, u16) }
pub struct Constraints { pub max: (u16, u16), pub tight_w: bool, pub tight_h: bool }
pub struct Insets { pub l: u16, pub t: u16, pub r: u16, pub b: u16 }

pub trait Measure { fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size; }  // small, optional

pub mod layout {
    pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect>;      // Track::{Fixed(u16), Flex(u16), Auto}
    pub fn columns(area: Rect, widths: &[Track], gap: u16) -> Vec<Rect>;
    pub fn responsive_columns(area: Rect, spec: &[Track], gap: u16, stack_below: u16) -> Vec<Rect>;
    pub fn action_row(area: Rect, widths: &[u16], gap: u16, align: RowAlign) -> Vec<Rect>;
    pub fn inset(area: Rect, i: Insets) -> Rect;
    pub fn split_v(area: Rect, at: u16) -> (Rect, Rect);
    pub fn split_h(area: Rect, at: u16) -> (Rect, Rect);
}
pub struct SplitModel { pub percent: u8, pub min_first: u16, pub min_second: u16,
                        pub maximized: Maximized, pub axis: SplitAxis }
```

**Decisions.** One `Split` implementation parameterised by axis, so the vertical/horizontal collapse asymmetry disappears; when both minima cannot fit, **the first pane wins on both axes** (documented, tested). `button::row_layout`/`row_layout_right` move into `layout::action_row` (they are the generic action-row primitive already used by dialog and grid). `showcase/pages/mod.rs:120-168`'s `rows`/`columns`/`caption` become library primitives. No general constraint solver.

**Surface inheritance replaces every `bg: Color` parameter.**

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface { Canvas, Surface, Elevated, Overlay, Popover, Field, FieldHover }
impl Surface { pub fn level(self) -> Option<usize>; pub fn from_level(i: usize) -> Surface; }

impl Theme {
    pub fn bg(&self, s: Surface) -> Color;
    pub fn raise(&self, s: Surface) -> Surface;   // Field -> FieldHover; ladder -> min(level+1, LAST)
}
impl Ui<'_> {
    pub fn surface(&self) -> Surface;
    pub fn bg(&self) -> Color;
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui) -> R) -> R;
}
```

Containers push the surface they fill; children inherit. `raise` is **index arithmetic on an ordered ladder**, not colour equality — this is what makes a light theme, a high-contrast theme, or a theme with duplicated plane colours behave correctly, and it removes the single biggest obstacle to Scenario B. The only remaining public raw colour is `Role::Custom(Color)` inside a `StylePatch` (§11), which still passes through capability downgrade. `Panel::bg_override` is deleted.

**Resize** is a first-class `Invalidate::Layout` with capture release and layer re-clamping, not a stored field.

---

## 11. Theme and customization model — **Adjudication D**

### 11.1 Decision

Adopt RES §3 in full: concrete `Theme` **data** (`ColorTokens` arrays + `DesignTokens`) + typed `Recipes` (family/variant/part/state) + `StylePatch` with `Slot<Inherit|Set|Clear>` + a scoped `Overlay` stack + the six-level precedence chain. No theme trait, no generic theme parameter, no trait object, no dynamic dispatch on the hot path. The README's "one `Theme` trait" suggestion is rejected: a trait would force every custom-theme author to reimplement resolution, and resolution is exactly the part that must stay uniform.

**Amendments to RES §3 (all binding):**

* **A1** The surface ladder is named to match `DESIGN.md` exactly: `Canvas, Surface, Elevated, Overlay, Popover` (RES's `Raised` is renamed `Surface`), plus the two non-ladder surfaces `Field`, `FieldHover`.
* **A2** `StylePatch` stores `Role`, never `Color`; roles bind to colours at the end of resolution against the live theme, the current `Surface`, and the colour capability. This is what makes a user patch written once work under every theme and every colour level, and it is non-negotiable.
* **A3** `Recipe` resolution is memoised per `(Family, Variant, Part, StateFlags, Surface, overlay-stack-hash)` in a small per-frame array cache (≤ 256 entries, cleared each frame), because rows resolve the same tuple repeatedly.
* **A4** Meter thresholds, spinner frames, wheel rows, press-flash and status durations join `DesignTokens` (they are currently consts in `progress.rs` and three `app.rs` files).
* **A5** Mono legibility is an intentional visual change, recorded in §20.10.

### 11.2 Exact token types

```rust
pub const SURFACE_LEVELS: usize = 5;      // Canvas, Surface, Elevated, Overlay, Popover
pub const FG_STEPS: usize = 5;            // Primary, Secondary, Muted, Faint, Ghost

#[derive(Clone, Copy, PartialEq, Eq)] pub enum FgStep { Primary, Secondary, Muted, Faint, Ghost }

pub struct ColorTokens {
    pub surfaces: [Color; SURFACE_LEVELS],
    pub field: Color, pub field_hover: Color,
    pub fg: [Color; FG_STEPS],
    pub on_accent: Color, pub on_danger: Color, pub on_surface_inverse: Color,
    pub border_subtle: Color, pub border_strong: Color,
    pub accent: Color, pub accent_hover: Color, pub accent_pressed: Color, pub accent_tint: Color,
    pub focus: Color, pub focus_ring: Color,
    pub selection_bg: Color, pub selection_fg: Color,
    pub highlight_bg: Color, pub highlight_fg: Color,
    pub highlight_danger_bg: Color, pub highlight_danger_fg: Color,
    pub backdrop_fg: Color, pub backdrop_bg: Color,
    pub danger: Color, pub danger_soft: Color, pub danger_tint: Color,
    pub warning: Color, pub warning_tint: Color, pub success: Color, pub info: Color,
    pub disabled_fg: Color, pub disabled_bg: Color, pub read_only_fg: Color,
    pub syntax: SyntaxTokens,
    pub meter:  MeterTokens,
}
pub struct SyntaxTokens { pub keyword, ident, string, number, operator, punct, comment, plain,
                          type_name, function, constant, invalid, deprecated,
                          match_bg, match_current_bg, bracket_match,
                          diagnostic_error, diagnostic_warning, diagnostic_info: Color }
pub struct MeterTokens  { pub low, medium, high, track, fill_rest, stale, unknown: Color,
                          pub series: [Color; 6] }
```

`Role` — the complete set a `StylePatch` may name:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    CurrentSurface, RaisedSurface,            // resolved against Ui::surface()
    Surface(Surface), Fg(FgStep),
    OnAccent, OnDanger, OnSurfaceInverse,
    BorderSubtle, BorderStrong,
    Accent, AccentHover, AccentPressed, AccentTint, Focus, FocusRing,
    SelectionBg, SelectionFg, HighlightBg, HighlightFg, HighlightDangerBg, HighlightDangerFg,
    BackdropFg, BackdropBg,
    Danger, DangerSoft, DangerTint, Warning, WarningTint, Success, Info,
    DisabledFg, DisabledBg, ReadOnlyFg,
    Syntax(SyntaxRole), Meter(MeterRole),
    Custom(Color),                            // the one documented raw-colour escape hatch
}
pub enum SyntaxRole { Keyword, Ident, Str, Number, Operator, Punct, Comment, Plain, TypeName,
                      Function, Constant, Invalid, Deprecated, MatchBg, MatchCurrentBg,
                      BracketMatch, DiagError, DiagWarning, DiagInfo }
pub enum MeterRole  { Low, Medium, High, Track, FillRest, Stale, Unknown, Series(u8) }
```

`GlyphRole` — every glyph in `DESIGN.md`'s table, one role each (roles are distinct even where Junie maps two to the same glyph, so a theme can separate them):

```rust
pub enum GlyphRole {
    FocusBar, Chosen, Checked, CheckboxOn, CheckboxOff, RadioOn, RadioOff, SwitchKnob,
    Dirty, Inserted, Deleted, Error, WarningMark,
    Collapsed, Expanded, SortAsc, SortDesc, Filtered, PrimaryMark, Bullet,
    FollowRef, MoreRows, OverflowLeft, OverflowRight, Ellipsis, Close,
    PathSep, EnvProduction, EnvStaging,
    RuleQuiet, RuleActive, ScrollTrack, ScrollThumb,
    ProgressDone, ProgressPaused, NewTab,
}
pub struct GlyphSet { /* one &'static str per GlyphRole */ }
pub struct BorderSet { pub tl, tr, bl, br, h, v: &'static str }   // rounded | square | ascii

pub struct DesignTokens {
    pub space: SpaceTokens,   // gutter 1, inline 1, gap 2, column_gap 2, form_gap 4,
                              // card_inset 2, frame_inset 3, dialog_inset 3, tree_indent 2
    pub size:  SizeTokens,    // field_height 3, tabs_height 2, dialog_width 54,
                              // dialog_width_wide 66, popup_max_rows 10, popup_min_width 12,
                              // popup_max_width 48, min_width 72, min_height 20,
                              // scrollbar_width 1, meter_track 10, code_preview_lines 6
    pub glyphs: GlyphSet,
    pub borders: BorderSet,
    pub motion: MotionTokens, // spinner_frames, tick_ms 80, idle_tick_ms 400,
                              // press_flash_ms 140, status_ms 4000, wheel_rows 3
    pub meter: MeterThresholds, // low_max 59, medium_max 84
    pub density: Density,     // Comfortable | Compact
}

pub struct Theme {
    pub color: ColorTokens,
    pub design: DesignTokens,
    pub recipes: Recipes,
    pub capability: Capability,   // { color: ColorLevel, unicode: UnicodeLevel }
}
impl Theme {
    pub fn junie() -> Theme;                      // the approved default, values unchanged
    pub fn paper() -> Theme;                      // the distinct non-Junie theme (§11.7)
    pub fn from_tokens(c: ColorTokens) -> Theme;  // derives design + recipes defaults
    pub fn builder(self) -> ThemeBuilder;         // partial override + safe derivation
    pub fn override_family(self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme;
    pub fn override_variant(self, f: Family, v: Variant, edit: impl FnOnce(&mut RecipeEdit)) -> Theme;
    pub fn define_variant(self, f: Family, v: Variant, edit: impl FnOnce(&mut RecipeEdit)) -> Theme;
    pub fn define_family(self, f: Family, edit: impl FnOnce(&mut RecipeEdit)) -> Theme;
    pub fn downgrade(&self, level: ColorLevel) -> Theme;
}
```

`Family` and `Variant` mirror `Part`: `u16` newtypes, library constants in the low range, `custom(&'static str)` const fn into the high range. Families: `BUTTON, CHOICE, CHIP, FIELD, INPUT, TEXTAREA, CODE, SELECT, LIST, TREE, GRID, PROPS, STEPS, TABS, PANEL, SPLIT, SCROLLBAR, VIEWPORT, DIFF, DIALOG, OVERLAY, MENU, PICKER, COMPLETION, FORM, HELP, WIZARD, STATUSBAR, HINTBAR, PROGRESS, METER, EMPTY, BRAND, KEYHINT`. Variants: `DEFAULT, PRIMARY, SECONDARY, SUBTLE, DANGER, TOGGLE, QUIET, GHOST` + custom.

### 11.3 Recipes, patches, precedence

```rust
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Slot<T> { #[default] Inherit, Set(T), Clear }
impl<T: Copy> Slot<T> { fn over(self, base: Slot<T>) -> Slot<T> { match self { Slot::Inherit => base, o => o } } }

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct StylePatch {
    pub fg: Slot<Role>, pub bg: Slot<Role>, pub underline: Slot<Role>,
    pub add: Modifier, pub remove: Modifier,
    pub glyph: Slot<GlyphRole>, pub size: Slot<u16>, pub align: Slot<Align>,
}
impl StylePatch {
    pub const fn new() -> Self;
    pub const fn set_fg(self, r: Role) -> Self;  pub const fn clear_fg(self) -> Self;
    pub const fn set_bg(self, r: Role) -> Self;  pub const fn clear_bg(self) -> Self;
    pub const fn set_underline(self, r: Role) -> Self;
    pub const fn add(self, m: Modifier) -> Self; pub const fn remove(self, m: Modifier) -> Self;
    pub const fn set_glyph(self, g: GlyphRole) -> Self;
    pub const fn set_size(self, n: u16) -> Self;
    pub fn merge(self, over: StylePatch) -> StylePatch;   // `over` wins where it speaks
}
pub struct StateRule { pub when: StateFlags, pub patch: StylePatch }
pub struct PartRecipe { pub base: StylePatch, pub states: SmallVec<[StateRule; 8]>,
                        pub glyph: Slot<GlyphRole>, pub size: Slot<u16> }
pub struct Recipe  { pub default_variant: Variant, pub parts: PartMap<PartRecipe>,
                     pub variants: SmallVec<[(Variant, PartMap<PartRecipe>); 6]> }
pub struct Recipes { by_family: Box<[Recipe]> }
pub struct Overlay { /* borrowed scope override; const-constructible */ }
pub struct Resolved { pub style: Style, pub glyph: Option<GlyphRole>, pub size: Option<u16> }
```

**Merge laws** (unit tests, §16): identity (`merge(x, default) == x`), absorption, clear (`Clear` resolves to "no colour" = the inherited surface fg / terminal default, never a panic), associativity, modifier symmetry (a later `remove(BOLD)` beats an earlier `add(BOLD)` and vice versa).

**Precedence, lowest → highest (documented, tested, exhaustive):**

1. family recipe base part style
2. variant delta (instance variant, else `recipe.default_variant`)
3. state rules, ordered by specificity (`when.count_ones()`), ties by declaration order, matched when `when ⊆ live`
4. theme-level global family/variant override
5. scope overlay stack, outermost → innermost
6. per-instance `.patch(...)` / `.patch_part(...)`

Then, and only then, roles bind to colours against `(theme.color, ui.surface(), theme.capability)`.

**Invariant:** overlays are borrowed and never mutate the `Theme`. `conformance::local_override_does_not_mutate_the_theme` asserts the theme is byte-identical before and after a scoped render.

### 11.4 Capability downgrade and the mono rule

```rust
impl ColorTokens {
    /// Exhaustive destructure: adding a field is a compile error here.
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> ColorTokens;
}
pub fn downgrade_color(c: Color, level: ColorLevel) -> Color;   // nearest_256 / nearest_16 / mono
impl Theme {
    pub fn downgrade(&self, level: ColorLevel) -> Theme {
        let mut out = self.clone();
        out.capability.color = level;
        out.color = self.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == ColorLevel::Mono { out.recipes.apply_mono_fallbacks(&mut out); }
        out
    }
}
```

This replaces the hand-written 30-field macro and the Junie-only `for_level`, so **any** theme downgrades (goal §15 scenario 11).

**The mono fallback rule (exact).** At `ColorLevel::Mono`, `apply_mono_fallbacks` appends one `StateRule` per family for each state below, so state survives without hue:

| Live flag | Mono guarantee |
|---|---|
| `FOCUSED` | `Part::GUTTER` glyph = `GlyphRole::FocusBar`, `Part::LABEL` adds `BOLD` |
| `SELECTED` / `CHECKED` | `Part::MARKER` glyph = `Chosen` / `Checked`, never colour-only |
| `PRESSED` | `Part::CONTAINER` `bg = Role::Fg(Primary)`, `fg = Role::Surface(Canvas)` (explicit reverse, never the terminal REVERSE attribute) |
| `DISABLED` | no gutter glyph, no marker, `fg = Role::Fg(Faint)`, all modifiers removed |
| `ERROR` | trailing `GlyphRole::Error` in `Part::MARKER` + `UNDERLINED` on `Part::FIELD` |
| `WARNING` / `DIRTY` | `GlyphRole::Dirty` in `Part::MARKER` |
| `EDITING` | `UNDERLINED` on `Part::TEXT` + the hardware cursor |
| `BUSY` / `LOADING` | spinner glyph in `Part::ICON` |
| `ACTIVE` (tabs) | `Part::RULE` glyph = `RuleActive` + `BOLD` label |

Test `conformance::mono_states_are_distinguishable` compares `(symbol, modifier)` pairs only, colour excluded, for every component × every state.

### 11.5 Where each concern lives (binding)

Colour roles → `ColorTokens`. Spacing → `design.space`. Dimensions → `design.size`, overridable per recipe via `PartRecipe.size`. Glyphs → `design.glyphs`. Border sets → `design.borders`. Focus indicator: the *glyph* is `GlyphSet::FocusBar`, *which parts wear it* is the recipe's `Part::GUTTER`. Selection indicator: glyph in `GlyphSet`, placement in `Part::MARKER`. Scrollbar symbols → `GlyphSet`, tone → the `SCROLLBAR` recipe's `TRACK`/`THUMB`. Animation cadence → `design.motion`. Density → `design.density` + per-instance `.density(...)`. Variant defaults → `Recipe.default_variant`. Meter thresholds → `design.meter`.

### 11.6 Junie-specific structural assumptions, resolved

**[F]** API §6.2 lists five that are *not* literal colours. Their resolutions: `lift` equality-dispatch → ladder index arithmetic (§10); `backdrop` equality-dispatch → a `backdrop` recipe keyed on `Role`, applied per resolved role rather than per `Color`; accent-budget rules ("only the lockup fills with accent", "one accent underline per screen", "a quota is never green") → recipe *defaults* in `Theme::junie()`, not component code; the reserved menu hue (`highlight`, `highlight_danger`, `danger_soft`) → first-class tokens every theme must supply; glyph/spacing literals → design tokens.

`rain::dim_buffer`'s colour-identity reverse lookup (**[F]** APP §8) is replaced by `Ui::dim_layer(area, steps)`, a runtime service that walks the **role** recorded per painted cell (`FrameOut::roles`, a parallel `Vec<Option<Role>>` filled by `Ui` painting methods) and steps it down the ladder semantically. Jackin's rain keeps its own `Tone` enum but maps it through `Role`, satisfying goal §22.3.

### 11.7 The distinct non-Junie theme

`Theme::paper()` — a **light** theme: `surfaces = [#fbfaf8, #f2f0ec, #e8e5df, #ded9d0, #cfc8bb]`, fg ladder from `#1b1a17` down to `#c6c0b6`, accent `#3b5bdb` (indigo), danger `#b02525`, warning `#a86400`, success `#1f7a3d`, `border_subtle #ded9d0` / `border_strong #9c948a`, square `BorderSet`, `Density::Compact`, and `default_variant` for `BUTTON` set to `SECONDARY`. It inverts the plane direction (hover *darkens*), changes hue family, changes glyph border set and density — the four axes that expose hidden Junie assumptions.

---

## 12. Composition, parts, and collections — **Adjudication G**

### 12.1 Composition mechanisms

| Capability | Mechanism |
|---|---|
| arbitrary content in panels/dialogs/overlays | slot closures `impl FnOnce(&mut Ui, Rect) -> R` — generic, no `Box`, no `'static` |
| header/body/footer/actions | a slot object with one method per slot (`DialogUi::{title, description, body, actions}`) |
| custom rows/cells | `RowUi` / `CellUi` painters + `Fn` renderers |
| wrap/decorate | nest `draw` calls; `draw` returns the rect it used |
| restyle a logical part | `.patch_part(&[(Part, StylePatch)])` |
| **replace** a logical part | `.slot(Part, &'a dyn Fn(&mut Ui, Rect))` — the component keeps layout, hit registration, focus and state |
| higher-level components | a plain struct with `update`/`draw` composed of primitives |
| reuse behaviour, new presentation | `XState` machines are public and buffer-free |
| borrowed content | `&'a [T]` + accessor closures; only visible items are visited |

`&'a dyn Fn` (never `Box`) keeps slots allocation-free and non-`'static`; this is the only `dyn` in a component's public surface and it is opt-in.

### 12.2 The one collection vocabulary

Applied to `List`, `Tree`, `Grid`, `Tabs`, `Picker`, `Completion`, `Props`, `Steps`, `Chips`:

```rust
// identity: every collection takes a key accessor; every action carries an ItemKey.
pub trait KeyFn<T>: Fn(&T) -> ItemKey {}

// per-row painting, parts pre-styled for the row's resolved state
pub struct RowUi<'u> { /* … */ }
impl RowUi<'_> {
    pub fn flags(&self) -> StateFlags;
    pub fn key(&self) -> ItemKey;
    pub fn marker(&mut self, g: GlyphRole);
    pub fn label(&mut self, s: &str);
    pub fn label_patched(&mut self, s: &str, p: &StylePatch);
    pub fn label_spans(&mut self, spans: &[Span<'_>]);
    pub fn meta(&mut self, s: &str);                  // dropped all-or-none (DESIGN.md:478)
    pub fn trailing(&mut self, s: &str, p: &StylePatch);
    pub fn columns(&mut self, widths: &[Track]) -> ColumnsUi<'_>;
    pub fn indent(&mut self, depth: u16);
    pub fn part(&mut self, p: Part, area_hint: u16) -> CellUi<'_>;
    pub fn raw(&mut self) -> (&mut Buffer, Rect);     // escape hatch, marks the rect written
}
pub struct CellUi<'u> { /* text, tone, align, italic, suffix glyph, patch */ }

// state of the data, not of the widget
pub enum EmptyState<'a> {
    Empty  { title: &'a str, hint: Option<&'a str> },
    Loading{ label: &'a str },
    Partial{ loaded: usize, total: RowTotal, hint: &'a str },
    Error  { message: &'a str, detail: Option<&'a str> },
}
pub enum RowTotal { Exact(usize), Estimated(usize), Unknown }

// decoration supplied by the owner, never derived inside the component
pub struct RowDecor  { pub marker: Option<GlyphRole>, pub tone: Option<Role>,
                       pub strike: bool, pub faint: bool, pub flags: StateFlags,
                       pub message: Option<&'static str> }
pub struct CellDecor { pub tone: Option<Role>, pub italic: bool, pub error: Option<&'static str>,
                       pub dirty: bool, pub suffix: Option<GlyphRole> }

// reconciliation: one rule for every collection
pub enum Reconciliation { Unchanged, CursorMoved(ItemKey), CursorLost, SelectionDropped(usize) }
impl XState { pub fn reconcile(&mut self, keys: impl Iterator<Item = ItemKey>) -> Reconciliation; }
```

**The reconcile rule (identical for every collection, tested once, table-driven):** keep the cursor/active/selected key if it is still present; else take the nearest surviving key by the previous index (forward first, then backward); else the first enabled key; else `None`. Checked sets drop vanished keys and report the count. Scroll offset is clamped to the new length. Every collection's `update` calls `reconcile` **before** emitting any action.

`SelectMode { Single, Multi, Range, None }`; cursor, selection and activation are three distinct concepts in every collection (`RadioGroup` is fixed to separate cursor from value).

**Scrolling is shared:** one `scroll_region(part)` helper on `Ui`/`Cx` provides scrollbar registration, track arithmetic, thumb drag through pointer capture, and `ensure_visible_on_next_layout` — deleting seven copies of `on_scrollbar` (**[F]** DOM §6.1(6)). The scrollbar is `Part::TRACK`/`Part::THUMB` of its container, not a separate id space; `scrollbar::id_for` is deleted.

### 12.3 Grid split (confirms DOM §1.5)

`DataTable` is deleted; `Grid` is the one tabular component. The reusable `Grid` keeps: columns (`key: ColumnKey`, `title`, `subtitle`, `align`, `min/max width`, `sortable`, `editable`, `sticky`, `prefix_glyph`, `badge`), two-axis viewport and column width sampling, `‹N`/`N›` overflow, cursor cell, rectangular range selection, row selection, copy-as-TSV, fetch-more row, `EmptyState`, an explicit edit lifecycle, an **action-surface slot**, `rows_label`/`cols_label`, and `NavUnit::{Row, Cell}`.

```rust
pub trait GridModel {
    type Row;
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> ItemKey;
    fn cell(&self, row: usize, col: usize) -> CellRef<'_>;         // borrowed text + tone + align
    fn row_decor(&self, row: usize) -> RowDecor { RowDecor::default() }
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor { CellDecor::default() }
    fn total(&self) -> RowTotal { RowTotal::Unknown }
    fn has_more(&self) -> bool { false }
}
pub trait GridEditor: GridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent;   // Inline{initial}|Cycle|External|Refuse{reason}
    fn apply_cycle(&mut self, row: usize, col: usize);
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
    fn read_only_reason(&self) -> Option<&str> { None }
}
pub trait GridCellActions: GridModel {
    fn actions(&self, row: usize, col: usize) -> &[CellAction];    // glyph + chord + ActionKey
}
```

`GridModel` is `&self` (used by both phases); `GridEditor` is `&mut self` and is reachable **only from `update`** — the phase split makes "rendering stages a database mutation" (`grid.rs:1518`) unrepresentable. Everything database-shaped moves to `apps/tablepro/src/grid_model.rs`: `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, `default_validator`, `cmp_cells`, `apply_commit_result`, insert/duplicate/toggle-delete/discard/undo, `primary`/`nullable`/`references`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar, and `Theme::change_glyph`. All 22 TablePro capabilities survive by the mapping in DOM §1.6, which is adopted verbatim as the migration checklist for Slice 6.

### 12.4 Other collections

`Tree` keeps hierarchy, lazy children and path-addressed expansion, gains caller keys (`TreeNode::keyed(k)`) and a row renderer — deleting `object_at`/`schema_at` path reconstruction and the plan-tree "paint over the widget after render" hack. `Tabs` gains stable keys and a reconciling `set`; `TabsAction` carries `ItemKey`; the strip window follows the logical first tab. `Picker` decomposes into `FilterList` (headless) + a picker overlay + a palette convenience, gains typed keys and first-class `scopes: &[ScopeKey]`, and its `hints: &str` parameter becomes a `HintLayer` contribution. `Completion` and `Picker` share one `Item { glyph, label, matched, detail, tag, group, disabled, key }`. `Props` becomes a two-column list variant. `Steps` keeps its ordered-lifecycle frontier (deliberately not a selection list). `Chips` keeps horizontal flow with the trailing add affordance, but shares cursor/activation/remove vocabulary.

Preserved meaningful differences: tree hierarchy + lazy loading; grid's two-axis cursor, rectangular range and per-cell editing; tabs' single-active-plus-cursor and strip window; steps' frontier; chips' horizontal flow; completion's non-modal owner-keeps-focus contract vs picker's modal contract; sort-as-permutation so edits stay bound to the source row (explicit test).

---

## 13. Public API conventions

| Question | Convention | Form |
|---|---|---|
| construction | Id-first constructor | `X::new(id, required…)`; alternates only for semantically different modes |
| configuration | consuming builder, no prefix | `fn variant(self, v: Variant) -> Self` — never `with_`, never `set_` on props |
| borrowed vs owned | props borrow, state owns | `X<'a>` holds `&'a` data; `XState` holds only interaction state |
| caller state | `&mut XState` per phase | `update(cx, &mut st)` / `draw(ui, area, &st)` |
| library state | runtime-managed | focus, hover, press, flash, cursor, regions, layers, captures, style stack |
| controlled value | `&mut T` in update, `&T` in draw | draft in state, value written on commit |
| uncontrolled | `XState::value()` | documented per component as the exception |
| variants & sizes | `.variant(Variant)` / `.density(Density)` | typed newtypes, `custom()` escape hatch |
| disabled vs read-only | `.disabled(bool)` / `.read_only(bool)` | read-only stays in the ring; disabled does not |
| loading/busy/error/editing | `.status(Status)` + `StateFlags` | `EDITING` is owned by state, never a prop |
| events | `Response<XAction>` | `.action()`, `.activated()`, `.changed()`, `.consumed()` |
| focus | automatic | `ui.register_control(id, area, Focusability)`; `.autofocus()`; `ui.focus_scope(…)` |
| render | `update` + `draw` | never fused; `draw` is `&self` |
| measure | `measure(&self, ui, Constraints) -> Size` | `Size { min, preferred }` |
| parts | `X::PARTS: &'static [Part]` | documented per component; used by theming, overrides, tests, hints |
| local override | `.patch(&StylePatch)` / `.patch_part(&[(Part, StylePatch)])` / `.slot(Part, &dyn Fn)` | |
| item renderer | `.row(Fn(&T, &mut RowUi))` / `.cell(…)` | closures, never `fn` pointers |
| identity | `.key(Fn(&T) -> ItemKey)` | actions carry `ItemKey` |
| errors | `FieldError`, `LayoutError` | typed, `Display + Error`; no panic on any interaction path |
| testing | `Harness` | `Harness::new(theme, size)`, `.key()`, `.click(id)`, `.actions()`, `.snapshot()` |
| API layers | `junie_tui::*` vs `junie_tui::author::*` | both `pub`, separately documented |

Additional binding rules: no boolean parameter soup (typed enums for semantically different modes); no gratuitous generics in application-visible signatures (collection generics are inferred and die at the call site); no public mutable rect or cache; no `'static` bound in any component's public surface; complete rustdoc on every public item (`#![deny(missing_docs)]`).

### 13.1 Keymaps, bindings and hints (confirms INT B9)

```rust
pub struct Binding<A: 'static> { pub chord: Chord, pub action: A,
                                 pub label: &'static str, pub priority: u8, pub visible: bool }
pub trait Bindings { type Action; fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Action>]; }
pub struct KeyMap { /* add / remove / remap, per Phase */ }
pub enum Phase2 { Capture, Bubble }
pub struct HintLayer { pub hints: Vec<Hint>, pub badge: Option<&'static str>,
                       pub status: Option<String>, pub centered: bool }
```

Components declare their bindings from small `const` tables selected by state (no per-frame allocation). The runtime resolves a key against the focused component's table after applying the app's `KeyMap` override layer, so no application chord is baked into a generic component: `Dialog`'s `y`/`n` becomes an opt-in binding set; `Picker`'s `Delete`-secondary gains a mouse equivalent; the grid's `p`/`u`/`U`/`Ctrl+]`/`Ctrl+S` move to TablePro's `KeyMap`. "App chords always win" is replaced by two explicit phases, with `Capture` skipped for bare-`Char` chords while the focused control `swallows_typing` — generalising the six ad-hoc `!editing` guards, and making the known-dead grid `Ctrl+D` *detectable* (`conformance::conflicting_visible_bindings_are_reported`) rather than merely documented.

Hints are **derived**: `HintBar` composes top layer ▸ temporary mode ▸ focused component's visible bindings by priority ▸ screen extras ▸ global fallback. The ~700 lines of hand-written hint tables across the apps collapse to product-level extras only. `EditAction::Apply(fn(&mut TextBuffer))` and `TextInput.validator: Option<fn…>` are deleted in favour of a `Binding` table over `EditAction` and the `Validate` trait.

---

## 14. Generic vs domain-specific boundaries — **Adjudication I**

### 14.1 Dispositions (confirming DOM §8, with amendments)

| Component | Disposition |
|---|---|
| `Dialog` | **Decompose** — `Overlay` layer primitive (§9) + composed `Dialog` with a body slot + convenience constructors `confirm/destructive/prompt/acknowledge/facts/choice/info` on the same path. `DialogBody` deleted; `&mut Focus` parameters deleted; ack arming moves to `update`. |
| `ContextMenu` / `MenuBar` | **Keep + extend** — typed `ActionKey` payload + optional `Chord` that both renders the hint and registers the binding (deletes label-string dispatch and jackin's key-synthesis `run_host_menu`); add `MenuItem::submenu`. Both become layer content. |
| `Picker` | **Decompose** — `FilterList` (headless) + picker overlay + `CommandPalette` convenience; typed keys; first-class scopes; `PickerStatus` promoted into `EmptyState`. |
| `Completion` | **Keep + controller** — a `Completion` controller owns the editor↔popup contract (`request(cursor, text)`, accept-splice, dismiss-on-move), collapsing the ~40 lines of hand-wiring in `tablepro/tabs.rs:1326-1377`. Non-modal `Popover` layer; owner keeps focus. |
| `StatusBar` + `segments` | **Merge** into one priority-ordered item strip with Left/Center/Right groups; TablePro's identity strip and grid status line become consumers, deleting two hand-written priority-drop loops. |
| `HintBar` | **Keep + wire** — fed by component bindings (§13.1). |
| `Tabs` | **Keep + stable keys** + reconciling `set`; no more per-frame rebuild inside `render`. |
| `Split` + `Splitter` | **Merge** into `SplitPane` owning its container rect from its own draw, with pointer capture and an optional keyboard-resize command; minima enforced once. |
| `TextViewport` | **Keep** (best-in-class); `set_area`/`prime`/`inert` clone dance disappears once view state is caller-owned. |
| `Panel` | **Keep** as a primitive; `bg: Color` and `bg_override` replaced by contextual `Surface`. |
| `ScrollPanel` | **Remove** — express its uses as `TextViewport` with tone-carrying spans. |
| `DataTable` | **Remove** — absorbed by `Grid` (`NavUnit::{Row,Cell}`). |
| `DataGrid` | **Decompose** — generic `Grid` + TablePro adapter (§12.3). |
| `CodeEditor` | **Refactor** onto `TextEditorCore`; `Highlighter`/`Segmenter` become `&dyn Fn` / trait objects that can capture a dialect and catalog; the vim-flavoured key table becomes the *default* `KeyMap`, not the only one. |
| `DiffView` | **Keep** as a composition; the data model moves behind a `DiffSource` trait so jackin's `ChangedFile` feeds it without a parallel model; `review_lines(f, width)` becomes a measure/layout pass. |
| `input`/`textarea`/`select`/`choice`/`chips`/`field_common` | **Refactor** behind `Field<C>` + `TextEditorCore` + explicit edit lifecycle + `Secret` (§15). |

### 14.2 Library additions J1–J13 (adjudicated)

| # | Item | Decision |
|---|---|---|
| J1 | Overlay/modal frame | **Library primitive** — `Overlay` layer + `Surface`/`Frame` chrome (§9). Four current copies deleted. |
| J2 | Form dialog / field group | **Library component** — `Form` (ordered fields, visibility, scroll-to-focused-field, action row, error row, nested popup) + `Field<C>`. Three independent form engines collapse to one. |
| J3 | Choice dialog | **Library convenience** — `Dialog::choice(...)` over the composed body. |
| J4 | Info/facts dialog with copyable rows | **Library convenience** — `Dialog::facts(...)` over `Props` + a scrollable detail slot; supersedes `DialogBody::Facts` and TablePro's button surgery. |
| J5 | Key-reference overlay | **Library component** — `HelpOverlay`, multi-column, scrollable, scope label, fed by the same binding metadata as `HintBar`. |
| J6 | File/path browser | **App composition** — the *pattern* (path field + list + mode toggle + confirm) is expressible with `Form` + `List`; `FsEntry` and the simulated filesystem stay in jackin. |
| J7 | Multi-step / wizard controller | **Library component** — `Wizard` (step order, rewind with per-step state retention, stepper line) alongside the display-only `Steps`. `ChoiceDialog::stepper(&str)` deleted. |
| J8 | Async/staged picker chain | **Library component** — `PickerChain` (stage list, `EmptyState::Loading/Error` with retry, breadcrumb scope, back-one-step). The 1Password account/vault/item model stays in jackin. |
| J9 | Keyed list/tree with custom row rendering | **Library** — the §12.2 vocabulary. Six hand-rolled jackin row models and two TablePro paint-over hacks are deleted. |
| J10 | Change-slot row decoration | **Library** — `RowDecor`; grid `RowState` and config `Change` both map onto it. |
| J11 | Tab strip with stable keys | **Library** — §12.4. |
| J12 | Meter tone mapping | **Library** — `MeterTokens` + `design.meter` thresholds + `MeterTone::from_ratio` helper; app-side duplicate matches deleted. |
| J13 | Modal stack + result routing | **Library** — §9. |

**Stays in the applications (justified):** TablePro's SQL generation, safety classification, Safe Mode, pending-change model, connection simulation, catalog; jackin's Capsule pane tree and PTY simulation, launch cockpit semantics, account/usage domain projections, `Doc`/`ConfigTabs` diff-against-original model, 1Password chain semantics, and `rain.rs` (goal §22.3, now consuming `Role` + `Ui::dim_layer`). Per goal §12, the only manual dispatch left in application code is: matching a screen's own `const Id`s, `map_action` at composition boundaries, the product Esc ladder beyond layer dismissal, product chords declared through `KeyMap`, and screen-level effects.

---

## 15. Forms and text editing — **Adjudication H**

```rust
// One editing core, shared by input / textarea / code / editable cells / picker query.
pub struct TextEditorCore { /* buffer, cursor, selection, h-scroll, multiline flag */ }
impl TextEditorCore {
    pub fn text(&self) -> &str;
    pub fn apply(&mut self, a: EditAction) -> EditOutcome;   // the only mutation entry point
    pub fn selection(&self) -> Option<Range<usize>>;
    pub fn cursor_pos(&self) -> CursorPos;
    pub fn zeroize(&mut self);                                // overwrites bytes before drop
}

// Explicit lifecycle. `render` is not in this list, and cannot be.
pub enum EditPhase { Idle, Editing }
pub enum BlurPolicy { CommitAndValidate, Commit, Cancel, Keep }
pub struct EditLifecycle { pub blur: BlurPolicy }
impl TextInputState {
    pub fn is_editing(&self) -> bool;
    pub fn begin(&mut self, current: &str);
    pub fn commit(&mut self, value: &mut String, v: &impl Validate) -> Result<(), FieldError>;
    pub fn cancel(&mut self);
    pub fn blur(&mut self, value: &mut String, v: &impl Validate, p: BlurPolicy) -> Result<(), FieldError>;
    pub fn set_error(&mut self, e: Option<FieldError>);       // external / async validation
}

// Validation: a trait with a blanket closure impl. No fn pointers anywhere.
pub trait Validate { fn check(&self, s: &str) -> Result<(), FieldError>; }
impl<F: Fn(&str) -> Result<(), FieldError>> Validate for F {}
pub struct NoValidate;
pub struct FieldError { pub message: Cow<'static, str>, pub code: Option<&'static str> }

// Secrets.
pub struct Secret(String);
impl Secret {
    pub fn new(s: String) -> Self;
    pub fn expose(&self) -> &str;                             // deliberately verbose
    pub fn fingerprint(&self) -> [u8; 8];
    pub fn masked_tail(&self, n: usize) -> String;            // SYNTHETIC, never the real tail
    pub fn zeroize(&mut self);
}
impl fmt::Debug   for Secret { /* "Secret([redacted])" */ }
impl fmt::Display for Secret { /* "[redacted]" */ }
// Secret is NOT Clone, NOT PartialEq, NOT Serialize.

pub struct SecretPolicy { pub mask: GlyphRole, pub synthetic_tail: usize }

// The field wrapper owns all chrome, once.
pub struct Field<'a, C> { id: Id, label: Option<&'a str>, required: bool,
                          optional_suffix: bool, help: Option<&'a str>,
                          error: Option<&'a str>, plain: bool, control: C }
impl<'a, C> Field<'a, C> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL,
                                         Part::MARKER, Part::FIELD, Part::HELP];
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;   // design.size.field_height
}
```

**Decisions.** `Field<C>` owns label (`*` required, `optional` suffix), help/error row, gutter, focus registration and height — deleting the per-control re-implementations and the `plain_label` flag, and deleting `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic from three apps. Values are **controlled** (`&mut String`), so the "rebuild the widget to change its value" idiom (five sites) disappears. Blur is an explicit intent-driven transition with a per-control policy (`CommitAndValidate` for `TextInput`, `Commit` for `TextArea`/`CodeEditor`, `Cancel` where a dialog demands it) — removing all five render-time commits. `RadioGroup` separates cursor from value. Masked fields render a synthetic tail, never the real characters (the safety property moves from jackin into the library, closing **[F]** API §5 item 13). Manual `Debug` impls redact on `TextInput`, `TextInputState`, `Field`, `Dialog`, `Form`, `EditState`, `TextEditorCore`; `conformance::secret_never_appears_in_debug` asserts it. `TextEditorCore::zeroize` overwrites before drop.

---

<!-- PART 2 (§16–§20, Appendices A–B) appended by a second synthesis pass -->
