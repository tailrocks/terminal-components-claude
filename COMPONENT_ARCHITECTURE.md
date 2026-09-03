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

---

## 16. Testing strategy

Every test named below is a real, runnable name. Builders create them with exactly these names; §16.5's `architecture::every_named_test_exists` asserts the inventory in this section matches the compiled test list, so a renamed or missing test is a build failure rather than a silent gap.

**Where tests live.**

| Level | Location | Runner |
|---|---|---|
| unit | `#[cfg(test)] mod tests` inside each library module | `cargo test -p junie-tui --lib` |
| conformance | `crates/tui/tests/conformance.rs` + `crates/tui-testing/src/conformance/` | `cargo test -p junie-tui --test conformance` |
| rendering / digest | `crates/tui/tests/render.rs`, `apps/*/tests/visual.rs` | `cargo test --workspace --test render --test visual` |
| application integration | `apps/*/tests/app_tests*.rs` (moved out of the binaries, see §18) | `cargo test -p showcase -p tablepro -p jackin-preview` |
| architecture | `crates/tui/tests/architecture.rs` + `xtask` | `cargo test --workspace --test architecture` |
| performance | `crates/tui/tests/perf.rs`, `apps/*/tests/perf.rs` | `cargo test --workspace --test perf --release -- --test-threads=1` |

`crates/tui-testing` is a **dev-only** crate (`publish = false`, depended on with `[dev-dependencies]` only) so the counting allocator, the `Harness` and the conformance driver never reach a shipped binary.

---

### 16.1 Unit tests (goal §25.1)

One `#[cfg(test)] mod tests` per module. Names are given verbatim; the module path is the test path.

**`id.rs`** — identity, goal §25.1 "stable identity"
`root_sub_part_index_item_are_all_distinct`, `separator_prevents_concatenation_collision` (asserts `Id::root("a").sub("b") != Id::root("ab")` **and** `Id::root("ab").sub("") != Id::root("a").sub("b")`), `kind_tag_separates_name_from_item_with_equal_bytes`, `id_equality_ignores_debug_label`, `id_is_const_constructible`, `item_key_text_is_stable_across_runs`, `item_key_pair_is_order_sensitive`, `part_custom_lands_in_the_high_range`, `part_constants_are_unique`, `debug_prints_path_in_debug_builds`, `debug_prints_hash_in_release_builds`.

**`response.rs`** — event consumption and invalidation
`ignored_consumed_changed_action_constructors`, `bitor_takes_consumed_over_ignored`, `bitor_takes_max_invalidate`, `bitor_keeps_the_first_action`, `repaint_raises_relayout_raises_further`, `no_repaint_lowers_to_none`, `map_action_preserves_flow_and_invalidate`, `erase_drops_the_action_only`, `must_use_is_enforced` (compile-fail via `trybuild`), `state_flags_round_trip`.

**`intent.rs` / `event.rs`**
`key_release_is_dropped`, `unmapped_mouse_button_is_dropped`, `mouse_carries_modifiers`, `chord_hashes_by_code_and_mods`, `secondary_up_is_modelled`, `wheel_carries_axis_and_delta`, `paste_reaches_only_an_editing_owner`.

**`focus.rs`** — traversal, scopes, restoration, disabled/read-only
`tab_cycles_forward_and_backward`, `shift_tab_is_the_exact_reverse`, `disabled_entries_are_registered_but_skipped`, `read_only_entries_stay_in_the_ring`, `click_only_entries_are_never_reachable`, `trap_confines_traversal_to_the_scope`, `trap_wraps_inside_the_scope`, `nested_scopes_resolve_innermost_first`, `scope_restore_returns_focus_to_the_opener`, `reconcile_prefers_nearest_surviving_entry_by_previous_index`, `reconcile_falls_back_to_scope_first_enabled`, `reconcile_falls_back_to_innermost_active_scope`, `reconcile_yields_none_when_nothing_is_reachable`, `focus_visible_is_true_only_after_a_key`, `trap_is_armed_when_the_layer_is_pushed_not_when_it_draws`.

**`hit.rs`** — hit ordering, layers, scroll routing
`last_registration_wins`, `higher_layer_shadows_lower`, `inert_below_registers_nothing`, `hit_returns_part_ref_not_a_derived_id`, `hit_scroll_returns_the_innermost_handler_of_the_axis`, `hit_scroll_returns_a_region_at_zero_headroom`, `hit_scroll_skips_regions_that_do_not_handle_the_axis`, `duplicate_id_is_reported_as_a_diagnostic_not_a_panic`, `empty_rects_are_rejected`, `generation_bump_invalidates_stale_regions`.

**`capture.rs`** — drag capture
`capture_claims_and_rejects_a_second_claim`, `drag_and_release_go_to_the_capture_owner`, `local_is_computed_against_the_captured_area`, `pressed_stays_set_while_the_pointer_leaves`, `release_outside_the_captured_area_does_not_activate`, `capture_is_released_on_resize`, `capture_is_released_when_the_owner_disappears`, `capture_is_released_on_generation_mismatch`.

**`scroll.rs`** — nested scrolling, boundary rule
`clamps_offset_to_content`, `ensure_visible_moves_minimally`, `thumb_covers_track_proportionally`, `track_position_round_trips`, `wheel_at_the_boundary_is_consumed_without_repaint`, `ensure_visible_on_next_layout_is_set_only_by_cursor_motion`, `fields_are_private_and_every_mutator_clamps`.

**`layer.rs`** — overlay stacking
`push_and_pop_maintain_layer_order`, `modal_pushes_a_trap_and_a_pointer_barrier`, `popover_pushes_a_pointer_barrier_only`, `esc_dismisses_only_the_top_layer`, `outside_click_is_layer_less_than_top_or_none`, `nested_layers_each_trap` (Scenario F), `anchor_rect_flips_then_clamps`, `anchor_screen_center_sits_in_the_upper_third`, `min_size_then_clamp_then_documented_degradation`, `closed_with_action_key_emits_layer_event_closed`, `dismissed_emits_the_reason`, `backdrop_excludes_the_footer_row`.

**`cursor.rs`**
`cursor_write_is_kept_for_the_focused_owner_on_the_top_layer`, `cursor_write_from_a_lower_layer_is_rejected`, `cursor_write_from_an_unfocused_owner_is_rejected`, `rejection_records_a_diagnostic`.

**`layout.rs` / `measure.rs`**
`rows_distributes_flex_after_fixed`, `columns_respects_gap_and_rounds_deterministically`, `responsive_columns_stack_below_the_threshold`, `action_row_right_aligns_and_left_aligns`, `inset_saturates_on_tiny_rects`, `split_first_pane_wins_on_both_axes_when_minima_do_not_fit`, `split_percent_is_clamped_to_5_95`, `measure_reports_min_and_preferred`.

**`text/` (buffer, editor, measure, fuzzy)**
`insert_and_move_by_grapheme`, `selection_replaces_on_insert`, `word_motion_and_deletion`, `word_chars_are_consistent_between_buffer_and_viewport`, `multiline_vertical_motion_keeps_column`, `single_line_rejects_newline`, `wide_characters_count_as_two_columns`, `combining_marks_are_one_grapheme`, `zwj_emoji_is_one_grapheme`, `pos_of_and_offset_at_round_trip`, `fuzzy_returns_grapheme_indices_into_the_original_label`, `fuzzy_ranks_prefix_before_boundary_before_substring_before_subsequence`, `editor_apply_is_the_only_mutation_entry_point`, `zeroize_overwrites_before_drop`.

**`theme/` (tokens, patch, recipe, resolve, downgrade)**
`slot_over_prefers_the_speaking_side`, `patch_merge_identity`, `patch_merge_absorption`, `patch_merge_is_associative`, `patch_clear_resolves_to_inherited_surface_fg`, `modifier_add_then_remove_is_symmetric`, `state_rules_are_stored_in_specificity_order` (R2 invariant), `state_rules_tie_break_by_declaration_order`, `state_rule_matches_only_when_when_is_a_subset`, `precedence_family_then_variant_then_state_then_global_then_scope_then_instance`, `roles_bind_after_the_whole_chain`, `raise_is_ladder_index_arithmetic_not_colour_equality`, `raise_saturates_at_the_last_level`, `field_raises_to_field_hover`, `downgrade_maps_every_token_exhaustively`, `downgrade_works_for_a_user_supplied_theme`, `mono_appends_one_state_rule_per_family`, `paper_theme_inverts_the_plane_direction`, `custom_family_and_variant_round_trip`, `theme_is_byte_identical_after_a_scoped_render`.

**`collection/` (key, reconcile, rowui, decor, empty)**
`reconcile_keeps_a_surviving_key`, `reconcile_takes_the_nearest_forward_then_backward`, `reconcile_falls_back_to_the_first_enabled_key`, `reconcile_yields_cursor_lost_when_empty`, `reconcile_drops_vanished_checked_keys_and_reports_the_count`, `reconcile_clamps_the_scroll_offset`, `reconcile_runs_before_any_action_is_emitted`, `generation_stamp_skips_a_no_op_reconcile` (R1), `cached_index_probe_hits_before_a_scan` (R1), `row_ui_label_writes_cells_without_an_intermediate_string` (R5), `row_ui_meta_is_dropped_all_or_none`, `row_ui_columns_clip_to_the_row`, `empty_state_covers_empty_loading_partial_error`.

**Component state machines** (`components/*.rs`, buffer-free, no terminal) — goal §25.1 "edit begin, commit, cancel, focus loss":
`input::begin_snapshots_the_value`, `input::commit_writes_the_controlled_value`, `input::commit_runs_validation_once`, `input::cancel_restores_the_snapshot`, `input::blur_commit_and_validate_policy`, `input::blur_cancel_policy`, `input::blur_keep_policy_leaves_the_draft`, `input::external_error_survives_a_redraw`, `input::masked_tail_is_synthetic`, `textarea::blur_commits_without_validation`, `select::escape_closes_and_restores_the_cursor`, `select::arrows_move_the_cursor_not_the_value_while_closed`, `choice::radio_group_separates_cursor_from_value`, `list::select_all_selects_only_enabled_items`, `list::range_selection_uses_the_anchor`, `tree::expand_collapse_is_keyed_not_positional`, `tree::lazy_children_do_not_reflatten_the_world`, `tabs::close_targets_the_logical_tab_after_a_reorder`, `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row`, `grid::edit_intent_inline_cycle_external_refuse`, `grid::range_copy_is_tsv`, `dialog::action_arming_is_evaluated_in_update`, `picker::query_change_emits_query_changed`, `wizard::rewind_retains_per_step_state`, `viewport::retention_fixes_up_selection_and_caret`, `code::edit_counter_invalidates_the_highlight_cache`, `secret::debug_and_display_redact`, `secret::is_not_clone_not_eq` (compile-fail via `trybuild`).

---

### 16.2 Shared conformance suite (goal §25.2)

**Mechanism.** One trait implemented once per public component; one macro generates the whole matrix. There is no per-component test writing.

```rust
// crates/tui-testing/src/conformance/mod.rs
bitflags! {
    pub struct Caps: u32 {
        const ACTIVATES   = 1 << 0;  // has a keyboard and a mouse activation path
        const DISABLEABLE = 1 << 1;
        const FOCUSABLE   = 1 << 2;
        const COLLECTION  = 1 << 3;  // takes items + a key fn
        const EDITS       = 1 << 4;  // has an edit lifecycle
        const SCROLLS     = 1 << 5;
        const OVERLAY     = 1 << 6;  // opens a layer
        const CAPTURES    = 1 << 7;  // claims pointer capture
        const CURSOR      = 1 << 8;  // writes the hardware cursor
        const SECRET      = 1 << 9;  // may hold secret bytes
    }
}

/// One registration per public component. `State = ()` for stateless components.
pub trait Conformance: 'static {
    const NAME: &'static str;                 // "button", "list", …
    const FAMILY: Family;
    const PARTS: &'static [Part];
    type State: Default + Clone + PartialEq + core::fmt::Debug;
    type Action;

    fn caps() -> Caps;
    fn id() -> Id;
    /// Fixture knobs the driver varies: disabled, read_only, item count, theme, colour level.
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action>;
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture);

    // capability-gated hooks; the default panics only if the matching cap is set
    fn activation_chords() -> &'static [Chord] { &[] }             // ACTIVATES
    fn activation_part() -> PartRef { PartRef::of(Part::CONTAINER) }
    fn bindings(s: BindingState) -> &'static [Binding<Self::Action>] { &[] }
    fn item_keys(f: &Fixture) -> Vec<ItemKey> { Vec::new() }        // COLLECTION
    fn reorder(f: &mut Fixture, perm: &[usize]) {}                  // COLLECTION
    fn action_key_of(a: &Self::Action) -> Option<ItemKey> { None }  // COLLECTION
    fn secret_bytes() -> &'static str { "" }                        // SECRET
}

pub struct Fixture {
    pub disabled: bool, pub read_only: bool, pub items: usize,
    pub theme: Theme, pub color: ColorLevel, pub area: Rect,
}

#[macro_export] macro_rules! conformance_suite { ($($case:ty),+ $(,)?) => { … } }
```

`crates/tui/tests/conformance.rs` ends with one invocation listing **every** public component; `architecture::conformance_covers_every_public_component` (§16.5) cross-checks that list against the `pub` component inventory, so adding a component without registering it fails CI.

```rust
conformance_suite!(
    ButtonCase, ChipCase, CheckboxCase, RadioGroupCase, ToggleCase, BrandCase, KeyHintCase, EmptyCase,
    FieldCase, TextInputCase, SecretInputCase, TextAreaCase, SelectCase,
    ListCase, NavListCase, TreeCase, PropsCase, StepsCase, GridCase, TabsCase, ChipBarCase,
    PanelCase, SplitPaneCase, ScrollRegionCase, TextViewportCase, DiffViewCase, CodeEditorCase,
    DialogCase, MenuBarCase, ContextMenuCase, PickerCase, FilterListCase, CompletionCase,
    FormCase, WizardCase, PickerChainCase, HelpOverlayCase,
    ProgressBarCase, SpinnerCase, MeterCase, StatusBarCase, HintBarCase, ScrollbarCase, TooSmallCase,
);
```

**Generated tests.** The macro emits one module per component (`mod button { … }`), so the fully-qualified names are `conformance::<component>::<case>`. Cases marked *(cap)* are emitted only when the capability is declared; the driver asserts a component never silently skips a case it should run.

| # | Generated test name | §25.2 contract | What it asserts |
|---|---|---|---|
| 1 | `disabled_cannot_activate` *(DISABLEABLE)* | disabled controls cannot activate | With `disabled: true`: every `activation_chords()` key and a full press→release over `activation_part()` return `Response::ignored()` with `action_ref().is_none()`; state is `PartialEq`-equal to the pre-input state; the entry is present in the ring with `disabled: true` and absent from `reachable()` |
| 2 | `keyboard_and_mouse_activation_are_equivalent` *(ACTIVATES)* | keyboard/mouse equivalence | The action produced by each chord equals the action produced by press→release on the same part, compared structurally (`ItemKey`s included); `flow` and `invalidate` also equal |
| 3 | `traversal_order_is_registration_order` *(FOCUSABLE)* | traversal order | Draw into a scene with two sentinels before and after; `ring().reachable()` ids appear in draw order; `next`/`prev` are exact inverses over the whole ring |
| 4 | `hover_does_not_steal_focus` | hover never focuses | A `MouseKind::Move` raster over every cell of the component's area leaves `focus()` unchanged; `HOVERED` is set; a key press sets `hover_suppressed` and clears the hover style until the pointer moves |
| 5 | `draw_twice_is_byte_identical` | rendering twice is stable | Two `draw` calls with the same props+state produce byte-identical `Buffer`s **and** identical `Registry` region lists (owner, part, area, layer, kind) |
| 6 | `draw_twice_leaves_state_equal` | rendering twice is stable | `st_before == st_after` by `PartialEq` after two draws, in the default, focused, hovered, editing and disabled fixtures |
| 7 | `draw_does_not_commit_or_cancel` *(EDITS)* | rendering does not commit or cancel edits | Begin an edit, remove focus, draw 3×: draft, committed value, error, pending set and overlay open-state are all unchanged; the controlled value is untouched |
| 8 | `draw_stays_inside_its_area` | component areas remain clipped | The component draws into a rect inset inside a sentinel-filled buffer; every cell outside the rect still holds the sentinel; every registered region satisfies `area ⊆ clip` |
| 9 | `mono_states_are_distinguishable` | no-colour output retains state indicators | Under `ColorLevel::Mono`, the `(symbol, modifier)` multiset differs pairwise between default / focused / selected / pressed / disabled / error / warning / editing / busy / active, colour excluded |
| 10 | `local_override_does_not_mutate_the_theme` | local overrides do not mutate the theme | Hash the `Theme` before, render inside `ui.with_overlay(&OV, …)` and with `.patch_part(…)`, hash after: equal; the overridden part's `Resolved` differs while the un-overridden sibling's does not |
| 11 | `id_separator_collision_free` | (added) | Every id and `PartRef` this component registers is unique within a frame; no two differ only by concatenation (`Diagnostic::DuplicateId` count is 0) |
| 12 | `item_identity_survives_reorder` *(COLLECTION)* | (added, Scenario E) | Set cursor/selection/checked on keys `k₁,k₂`; apply a reverse permutation and an insert+remove; after `reconcile`, cursor and checked set still name `k₁,k₂`; a click on the row now showing `k₁` emits an action carrying `k₁` |
| 13 | `focus_reconcile_follows_the_rule` *(FOCUSABLE)* | (added) | Remove the focused entry: focus lands on the nearest surviving entry by previous index; if the scope empties, on the scope's first enabled; then on the innermost active scope's first; then `None` — all four branches exercised |
| 14 | `focus_trap_and_restore` *(OVERLAY)* | (added) | Opening the layer shrinks `reachable()` to the layer's own stops; Tab wraps inside; closing restores focus to the opener; a layer that cannot draw (0×0) still traps |
| 15 | `pointer_capture_delivers_drag_and_release` *(CAPTURES)* | (added) | After a press claims capture, drags outside the component still reach it with `local` relative to the captured area; a second claim is refused; release outside the captured area does not activate |
| 16 | `wheel_at_boundary_is_consumed_without_repaint` *(SCROLLS)* | (added) | At offset 0 a wheel-up returns `Flow::Consumed` with `Invalidate::None`; mid-range returns `Consumed` + `Paint`; the event never chains to an outer scrollable and never moves focus or the cursor |
| 17 | `cursor_write_is_rejected_off_top_layer` *(CURSOR)* | (added) | Drawn under an open modal, the component's `ui.set_cursor` is dropped and one `Diagnostic::CursorRejected` is recorded; on the top layer with focus, it is kept |
| 18 | `secret_never_appears_in_debug` *(SECRET)* | (added) | `format!("{:?}")` of props, state, and any owning container (`Field`, `Dialog`, `Form`) contains neither `secret_bytes()` nor its snapshot; the rendered buffer contains neither; the `TestBackend` digest is unchanged when only the secret changes |
| 19 | `survives_tiny_rects_0x0_to_3x3` | (added) | For every `w,h ∈ 0..=3`: `draw` does not panic in a debug build, writes no cell outside the rect, registers no region outside it, and leaves no stale geometry (a click after a 0×0 frame resolves to `None`, never to last frame's rect) |
| 20 | `bindings_match_handled_keys` | (added) | Every chord in `bindings(state)` is consumed by `update` in that state, and every chord `update` consumes in that state appears in `bindings(state)` — the table and the handler cannot drift |

Suite-level tests (emitted once, not per component), in `conformance.rs`:

* `conformance::registry::every_public_component_is_registered`
* `conformance::registry::declared_parts_are_the_parts_actually_styled` — the parts a component resolves at draw time equal `Self::PARTS`
* `conformance::conflicting_visible_bindings_are_reported` — two visible bindings on the same chord in the same phase produce a `Diagnostic::BindingConflict` (this is what makes the historically dead grid `Ctrl+D` detectable)
* `conformance::focus_transition_settles` — the §3.3 step 7 re-run loop converges within 4 passes for every registered component
* `conformance::draw_registers_nothing_when_it_cannot_draw` — the 0×0 case across the whole registry

---

### 16.3 Rendering and snapshot tests (goal §25.3)

**Mechanism.** `TestBackend` + an FNV‑1a digest of `(symbol, fg, bg, modifier)` per cell — the existing `showcase_visual_baseline` mechanism (**[F]** APP §6, `app_tests.rs:623-668`) generalised. Digests, not golden images: a digest fails fast and the reviewer regenerates the *image* with `tools/capture.sh` to look at the difference.

```rust
// crates/tui-testing/src/digest.rs
pub struct Scene { /* theme, color level, size, name */ }
impl Scene {
    pub fn new(name: &'static str, theme: Theme, color: ColorLevel, w: u16, h: u16) -> Self;
    pub fn draw(&mut self, f: impl FnOnce(&mut Ui<'_>, Rect));
    pub fn digest(&self) -> u64;
    pub fn text(&self) -> String;
    pub fn assert_against(&self, baseline: &Baseline);   // writes on BLESS=1
}
pub struct Baseline { path: &'static str }   // one `name w h theme color hash` line, sorted
```

**Baseline files** (checked in, reviewed like source):

| File | Owner | Content |
|---|---|---|
| `crates/tui/tests/baselines/components.txt` | library | one line per component × state × theme × colour × size |
| `apps/showcase/tests/baselines/showcase.txt` | showcase | the migrated `showcase_visual_baseline`, now **including** the sidebar (the exclusion existed only because the shell sidebar was hand-drawn; it becomes a `NavList` and is covered) |
| `apps/tablepro/tests/baselines/tablepro.txt` | tablepro | **new** — closes **[F]** APP §9 risk 5 (TablePro had no cell-level baseline) |
| `apps/jackin-preview/tests/baselines/jackin.txt` | jackin | **new**, at `--motion paused --frame N` for determinism |

**Regeneration and review policy.** `BLESS=1 cargo test --workspace --test render --test visual` rewrites baselines. The rule from goal §6.10 is enforced mechanically: `xtask bless-guard` fails a commit that changes a baseline file without a matching entry in `docs/visual-changes.md` referencing a §20.10 item and a capture path under `shots/`. No baseline is regenerated because a test failed; the classification comes first.

**The matrix** — `crates/tui/tests/render.rs`:

* Test `render::components::<component>::<state>` for every registered `Conformance` case × states `{default, focused, focus_visible, hovered, focus_plus_hover, pressed, selected, disabled, read_only, busy, loading, error, warning, editing, empty, overflow}` where meaningful (the driver derives which are meaningful from `Caps`).
* Themes: `Theme::junie()` and `Theme::paper()` — test names get a `_paper` suffix.
* Colour levels: `truecolor`, `ansi256`, `ansi16`, `mono` — suffix `_256` / `_16` / `_mono`.
* Sizes: `80x24`, `100x30`, `120x40`, `160x50`, plus `40x10` for the narrow/overflow states.
* Overrides: `render::overrides::global_family_override_changes_every_button`, `render::overrides::scoped_overlay_changes_only_the_subtree`, `render::overrides::instance_patch_changes_only_one_instance`, `render::overrides::part_slot_replaces_the_part_and_keeps_hit_regions`.
* Composition: `render::overlay::modal_over_page`, `render::overlay::nested_picker_over_dialog`, `render::overlay::popover_anchored_below_then_flipped`, `render::overlay::backdrop_excludes_the_footer`, `render::overlay::layer_composites_bottom_to_top_regardless_of_call_order`.

**Showcase digest contract** (from **[F]** APP §6): `apps/showcase/tests/visual.rs::showcase_visual_baseline` keeps its exact shape — for each page in `NAV_ENTRIES` × `{120×40, 80×24}`, focus the first control, hash every cell, write `"{w}x{h} {label} {hash:016x}"`, regenerate under an env var. Two changes, both recorded in §20.10: the sidebar rect is no longer excluded, and the matrix gains `× {junie, paper} × {truecolor, mono}` (four times the lines). `UPDATE_BASELINE=1` is preserved as an alias of `BLESS=1` so the documented workflow (`README.md:325-328`) still works.

---

### 16.4 Application integration tests (goal §25.4)

**All current tests are retained**: 26 showcase, 21 tablepro, 22 jackin (17 + 5 chrome) plus the in-module `rain`/`arbiter`/`clock`/`scenario` unit tests — the exact inventories in **[F]** APP §5.1–§5.3. They move from `#[cfg(test)] mod app_tests` inside each binary to `apps/<app>/tests/app_tests.rs`, which forces each app to expose a small, deliberate test surface instead of reaching into private fields (goal §21).

**The `Harness` contract.** Every assertion shape in the three existing suites survives because `Harness` provides exactly these operations. This table is the migration contract: an operation missing here means an existing test cannot be expressed.

```rust
// crates/tui-testing/src/harness.rs
pub struct Harness<A: App> { /* Runtime<A> + Terminal<TestBackend> */ }

impl<A: App> Harness<A> {
    // construction
    pub fn new(app: A, theme: Theme, w: u16, h: u16) -> Self;
    pub fn with_color(self, level: ColorLevel) -> Self;

    // input → Response, then an automatic synchronous draw (replaces `handle` + `draw`)
    pub fn handle(&mut self, input: Input) -> Response<()>;
    pub fn key(&mut self, code: KeyCode) -> Response<()>;
    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) -> Response<()>;
    pub fn ctrl(&mut self, c: char) -> Response<()>;
    pub fn alt(&mut self, c: char) -> Response<()>;
    pub fn type_str(&mut self, s: &str) -> Response<()>;
    pub fn paste(&mut self, s: &str) -> Response<()>;
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Response<()>;
    pub fn click(&mut self, x: u16, y: u16) -> Response<()>;
    pub fn click_id(&mut self, id: Id) -> Response<()>;                 // clicks the centre of area_of(id)
    pub fn click_part(&mut self, id: Id, p: PartRef) -> Response<()>;
    pub fn double_click(&mut self, x: u16, y: u16) -> Response<()>;
    pub fn secondary(&mut self, x: u16, y: u16) -> Response<()>;
    pub fn drag(&mut self, from: (u16,u16), to: (u16,u16)) -> Response<()>;
    pub fn wheel(&mut self, axis: Axis, delta: i16, x: u16, y: u16) -> Response<()>;
    pub fn resize(&mut self, w: u16, h: u16) -> Response<()>;

    // explicit draw, for tests that assert on redraw suppression
    pub fn draw(&mut self);
    pub fn ticks(&mut self, n: usize);                                   // virtual clock, no wall clock
    pub fn tick(&mut self) -> Response<()>;

    // addressing (replaces hits.area_of / ring.reachable / focus.current)
    pub fn area_of(&self, id: Id) -> Option<Rect>;
    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect>;
    pub fn ring(&self) -> &FocusRing;                                    // .reachable(), .entries()
    pub fn focus(&self) -> Option<Id>;
    pub fn focus_visible(&self) -> bool;
    pub fn hover(&self) -> Option<Id>;
    pub fn state_of(&self, id: Id) -> StateFlags;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
    pub fn tab_to(&mut self, id: Id) -> bool;                            // jackin's helper, promoted
    pub fn diagnostics(&self) -> &[Diagnostic];

    // reading the frame
    pub fn text(&self) -> String;
    pub fn row(&self, y: u16) -> String;
    pub fn find(&self, needle: &str) -> Option<(u16, u16)>;              // grapheme-accurate
    pub fn find_row(&self, needle: &str) -> Option<u16>;
    pub fn count(&self, needle: &str) -> usize;
    pub fn cell(&self, x: u16, y: u16) -> &Cell;
    pub fn cursor(&self) -> Option<Position>;
    pub fn snapshot(&self) -> Scene;                                     // digest / assert_against
    pub fn actions(&mut self) -> Vec<AppActionRecord>;                   // drained semantic actions
}
```

Mapping of the seven "must keep working" facts from **[F]** APP §6:

1. `handle(Input) -> Response<()>` with `changed()` meaning "redraw needed" — `Response::changed()` replaces `Outcome::Changed`; the ~60 assertions of the form `assert_eq!(h.key(…), Outcome::Changed)` become `assert!(h.key(…).changed())`. `update` runs inside `handle`, so the answer stays truthful (this is the §3.2 argument against `show`, discharged here).
2. Synchronous deterministic draw after every input — `Harness::handle` draws before returning.
3. Stable test-visible addressing — app `const Id`s stay `pub` within the app crate; `FORM.sub("save")` becomes `FORM.part(Part::custom("save"))` or `area_of_part(FORM, PartRef::of(Part::custom("save")))`; `WidgetId::of("editor.cfg").sub("form").sub("save")` becomes `screens::editor::CFG_FORM.part(SAVE)`.
4. Resolved geometry read-back — `area_of` / `area_of_part`.
5. Reachable focus ring — `ring().reachable()`.
6. Virtual-clock tick injection — `ticks(n)`; jackin's `Clock` keeps its no-wall-clock contract.
7. Exact minimum-size copy strings — the shared `TooSmall` component keeps `"Terminal too small"` and `"Need {w}×{h}, have {w}×{h}"` verbatim; `showcase::below_minimum_size_shows_reduced_state` and its two siblings are unchanged.

**Theme coupling in tests** (`focus_bar_x` compares against `Theme::junie().focus`, **[F]** APP §6) becomes `h.resolved(id, Part::GUTTER).style.fg`, so the assertion survives a theme change and also runs under `Theme::paper()`.

**New application coverage** required by goal §25.4:

`showcase::complete_navigation_visits_every_page_and_every_state`, `showcase::custom_theme_injection_repaints_every_page` (`--theme paper`), `showcase::local_override_page_shows_three_distinct_buttons`, `showcase::author_component_page_participates_in_focus_and_hover`, `tablepro::mouse_flow_full_journey` and `tablepro::keyboard_flow_full_journey` (retained, renamed from `acceptance_flow_*`), `tablepro::grid_adapter_keeps_every_pending_change_capability`, `jackin::complete_flow_keyboard_first` (retained), `jackin::nested_overlay_picker_inside_dialog`, `*::resize_across_every_supported_size`, `*::focus_is_restored_after_every_overlay_closes`, `*::no_diagnostics_are_emitted_during_the_journey` (asserts zero `DuplicateId`, `CursorRejected`, `UndeliveredIntent`, `BindingConflict` in a full run — a strong, cheap regression net).

---

### 16.5 Architecture checks (goal §25.5)

`crates/tui/tests/architecture.rs` plus an `xtask` binary for the checks that need to read the workspace. Preference order per goal §25.5: **Cargo/visibility first, `cargo tree` second, text allow-lists last.**

| Test name | Mechanism | Asserts |
|---|---|---|
| `architecture::library_has_no_application_dependency` | `cargo metadata` from `xtask`: the dependency closure of `junie-tui` | `showcase`, `tablepro`, `jackin-preview` are absent; the only deps are `ratatui`, `unicode-width`, `unicode-segmentation`, `bitflags`, `smallvec` |
| `architecture::applications_depend_only_on_the_library_facade` | `cargo tree -p <app> -e normal --depth 1` + a source scan for `junie_tui::` paths | Every path resolves under `junie_tui::` or `junie_tui::author::`; there is no `#[path]`, no `include!`, and no `pub(crate)` reachable from an app (guaranteed structurally — a separate crate cannot name a `pub(crate)` item, so this test is a belt-and-braces report, not the enforcement) |
| `architecture::examples_are_external_consumers` | `cargo build -p junie-tui --examples` in CI plus a check that no example uses `#[path]` | The 12 §17 examples compile against the public API only |
| `architecture::all_examples_compile` | `cargo test --workspace --doc` + `--examples` gate | goal §25.5 "all examples compile" |
| `architecture::public_items_are_documented` | `#![deny(missing_docs)]` in `crates/tui/src/lib.rs` + `RUSTDOCFLAGS="-D warnings" cargo doc` | Every public item has rustdoc |
| `architecture::no_unsafe` | `#![forbid(unsafe_code)]` in the library; `crates/tui-testing` carries the single documented `unsafe impl GlobalAlloc` with a safety comment | goal §26 |
| `architecture::no_domain_vocabulary_in_the_library` | grep allow-list over `crates/tui/src/**` for `(?i)\b(sql|schema|primary key|nullable|foreign|references|not null|tablepro|jackin|workspace|instance|daemon|capsule|construct|catalog)\b`, with an allow-list file `crates/tui/tests/allow/domain.txt` (currently empty) | DOM §7 acceptance conditions 1 and 2 |
| `architecture::palette_literals_are_confined_to_theme_builtins` | grep for `Color::Rgb(` / `#[0-9a-f]{6}` over `crates/tui/src/**`, allow-listed to `theme/builtin/junie.rs`, `theme/builtin/paper.rs`, and `tests/fixtures/*.rs` | goal §25.5 |
| `architecture::no_raw_background_parameter` | grep for `bg: Color` / `bg: ratatui::style::Color` in any `pub fn` signature under `crates/tui/src` | The 24 `bg: Color` sites (**[F]** API §3.6) are gone; `Role::Custom(Color)` inside a `StylePatch` is the one allowed raw colour and is allow-listed by name |
| `architecture::no_owns_or_locate_in_applications` | grep for `\.owns\(`, `\.locate`, `scrollbar::id_for`, `\.child\(` over `apps/**/src` | Target 0; the allow-list file `apps/allow/dispatch.txt` must be empty and any entry requires a justification comment that the test prints |
| `architecture::no_generic_component_copies_in_applications` | grep for `fn render(` + `Style::new()` + `Block::default()` + `buf.set_string` over `apps/**/src`, allow-listed to `apps/jackin-preview/src/rain.rs` | goal §25.5 "application directories do not contain generic component copies"; rain is the single documented exception (goal §22.3) |
| `architecture::no_public_geometry_or_cache` | grep for `pub area`, `pub areas`, `pub anchor`, `pub .*_rects`, `pub scroll` under `crates/tui/src` | Invariant S1; kills the 21 `pub area` + 3 `pub areas` sites (**[F]** API §3.8) |
| `architecture::no_fn_pointer_extension_points` | grep for `: fn\(`, `Option<fn(`, `type \w+ = fn\(` under `crates/tui/src` | The 6 sites in **[F]** API §3.12 are gone (DOM §7 condition 6) |
| `architecture::draw_takes_shared_self` | `syn`-based scan in `xtask`: every `fn draw` in `crates/tui/src/components/**` has `&self` and, if it takes a state parameter, `&XState` | G2 — the structural form of "render cannot change semantics" |
| `architecture::no_static_bound_in_component_surface` | `syn` scan for `'static` bounds on public component types and their builder parameters, allow-listed to `Binding<A: 'static>` and `Conformance: 'static` | goal §2.2 |
| `architecture::conformance_covers_every_public_component` | `syn` scan of `pub struct`s in `components/**` vs the `conformance_suite!` list | G10 / §16.2 |
| `architecture::every_named_test_exists` | parses this section's names out of `COMPONENT_ARCHITECTURE.md` and compares against `cargo test -- --list` | Documentation and the suite cannot drift |
| `architecture::binary_names_are_preserved` | `cargo metadata` target names | `showcase`, `tablepro`, `jackin-preview` (goal §21) |
| `architecture::msrv_and_edition_are_unchanged` | `cargo metadata` | edition 2024, `rust-version = "1.88"` on every package |

---

### 16.6 Performance (goal §25.6)

**The measurement plan of `docs/audit/performance-audit.md` §7 is adopted verbatim** — harness, assertion policy, baseline file format, CI wiring, and every test name. Nothing in it is renamed. Restated obligations:

* Harness in `crates/tui-testing/src/perf.rs` (`Counting` global allocator shim, `ALLOCS`/`BYTES`, `bench`, `Stats`, `report`); `#[global_allocator]` declared **only** in `crates/tui/tests/perf.rs` and `apps/*/tests/perf.rs`.
* Allocation and byte counts are deterministic → **hard assertions**. Wall time is reported always, asserted only under `PERF_STRICT=1` against `baseline × 1.2`.
* Baseline `crates/tui/tests/perf_baseline.txt`, one `name ns allocs bytes` line, regenerated only with `PERF_BLESS=1`, reviewed in the diff.
* **The baseline is recorded on the pre-refactor tree**, on a `perf/baseline` commit, before Slice 3 begins (Appendix A, WP‑0). This is a hard sequencing constraint: without it "before and after" is not literal.
* `--test-threads=1` is mandatory; the counters are process-wide.
* Every screen benchmark also reports `hits=<registry len>` and `ring=<reachable len>`.

**Before → after thresholds.** "Before" is the recorded pre-refactor baseline; "after" is the assertion that must hold at the end of Slice 8.

| Test (perf §7) | Before | After (asserted) |
|---|---|---|
| `frame_showcase_lists_120x40` | ≈ 160 allocs/frame, 57 hits, 4 ring | **< 20 allocs/frame**; hits within ±10 %; ring ≥ 4 |
| `frame_showcase_lists_80x24` | report | ≤ `frame_showcase_lists_120x40` |
| `frame_showcase_dialog_open` | full background still registered | hits **< 25 %** of `frame_showcase_lists_120x40` (`inert_below`) |
| `frame_tablepro_grid_500x12_120x40` | ≈ 1 110 allocs/frame, ≈ 300 hits | **< 100 allocs/frame**; hits ≤ 320 |
| `frame_jackin_manager_100rows_120x40` | ≈ 350–400 allocs/frame | **< 60 allocs/frame** (rows rebuilt on world generation change only) |
| `frame_jackin_capsule_4panes_120x40` | ≈ 480 000 allocs/frame | **< 200 allocs/frame** |
| `key_showcase_down_lists` | includes `describe_key` `String` | **0 allocs/event** |
| `key_tablepro_grid_cursor` | report | **0 allocs/event** |
| `key_tablepro_grid_sort_local` | 4 allocs/comparison | **≤ 1 alloc/comparison** (`display_width`, no `to_lowercase` per compare) |
| `key_jackin_manager_move` | ≈ 200 allocs/key | **0 allocs/key** |
| `key_tree_toggle_10k` | full reflatten | allocs **< 10 × viewport** per toggle |
| `mouse_move_over_1000_regions` | 0 allocs | **0 allocs**, ns within 1.2× |
| `mouse_move_showcase_frame` | report | **0 allocs** |
| `mouse_click_grid_cell` | ≈ 250 hash computations | **0 allocs**, ns **< 0.2×** before (no `locate`) |
| `wheel_showcase_lists`, `wheel_tablepro_grid` | 0 allocs | **0 allocs** |
| `focus_tab_traversal_ring_200` | 0 allocs | **0 allocs** |
| `style_resolve_10k_parts` | `Theme::row`+`gutter`, 0 allocs | **exactly 0 allocs**, ns ≤ 2× before (R2) |
| `style_resolve_10k_parts_with_two_overlays` | n/a | **0 allocs**, ns ≤ 2× the empty-stack case (R3) |
| `style_backdrop_full_screen_120x40` | 4 680 equality chains, 0 allocs | **0 allocs**, ns ≤ 1× (walk restricted to the covered rect) |
| `style_downgrade_theme_all_levels` | one-shot | one-shot only; asserted **not** to appear in any `frame_*` profile |
| `list_100k_rows_construct` | report | report |
| `list_100k_rows_render` | — | **< 500 allocs/frame**, ns ≤ 1.5× `list_1k_rows_render` (R1) |
| `list_1k_rows_render` | control | control |
| `tree_100k_nodes_flatten` | ≈ 300 000 allocs per toggle | allocs **< 10 × viewport** |
| `tree_100k_nodes_render` | — | allocs/frame independent of node count |
| `grid_500x12_render` | — | **< 100 allocs/frame** |
| `grid_500x12_load` | ≈ 36 000 allocs | **< 8 000 allocs** (one owned conversion) |
| `grid_100k_local_sort` | ≈ 7 M allocs | report; documents why `local_sort` stays opt-in |
| `viewport_100k_lines_push` | ≈ 6 M allocs/line | allocs **independent of `lines.len()`** |
| `viewport_100k_lines_render` | — | allocs/frame independent of buffer size |
| `capsule_pane_clone_4x2000` | dominant cost | **the test is deleted**; its deletion is asserted by `architecture::every_named_test_exists` reading the §20.9 note |
| `width_10k_grapheme_line` | 0 allocs | **0 allocs** |
| `truncate_10k_grapheme_line_to_80` | exactly 1 | 1 (non-render callers only) |
| `fit_10k_grapheme_line_to_80` | exactly 3 | the `RowUi` equivalent records **0** (R5) |
| `truncate_middle_10k_to_40` | exactly 4 | ≤ 1 |
| `wrap_10k_graphemes_to_80` | ≥ 1/line | report |
| `textbuffer_pos_of_10k_line`, `textbuffer_offset_at_10k_line` | 0 allocs | **0 allocs**, ns ≤ 1× |
| `viewport_layout_10k_grapheme_line` | ≈ 10 000 allocs | **0 allocs** |
| `render_twice_allocates_the_same` | — | equal counts |
| `no_full_collection_clone_per_frame` | — | **bytes/frame < 64 KiB** for a 100 k list frame and a 100 k viewport frame |
| `event_dispatch_is_not_o_n` | — | **0 allocs**, ns within **3×** of the 100-row case |
| `hit_registry_size_is_bounded` | recorded | within baseline ± 10 % |
| `debug_and_release_alloc_counts_match` | — | equal (R4) |

**Three additions to perf §7**, needed because §7 has no coverage for two obligations §20.9 makes binding. They are additions, not renames, and are marked as such in `perf_baseline.txt`:

| Added test | Location | Threshold |
|---|---|---|
| `frame_tablepro_query_editor_2k_lines` | `apps/tablepro/tests/perf.rs::frames` | before ≈ 7 collections + O(graphemes × spans) per frame; after **< 40 allocs/frame** and ns scaling with *viewport*, not document length (§20.9 amendment 9) |
| `list_100k_select_all` | `crates/tui/tests/perf.rs::large` | `ToggledAll` must **not** materialise 100 000 `ItemKey`s: **< 100 allocs** (R7) |
| `intents_drain_scales_with_intents_not_components` | `crates/tui/tests/perf.rs::invariants` | a frame with 500 registered components and 2 intents costs the same as one with 20 components and 2 intents, ±10 % (R6) |

**CI wiring** (perf §7.3, adopted verbatim): one always-on job `cargo test --workspace --test perf --release` (allocation counts only) and one pinned-runner job `PERF_STRICT=1 cargo test --workspace --test perf --release -- --test-threads=1`. `PERF` lines are collected into a build artefact for the final report (goal §30 item 13).

---

## 17. Representative usage examples

Twelve examples, one file each under `crates/tui/examples/`, built by `cargo build -p junie-tui --examples` and gated by `architecture::all_examples_compile`. They use only the public facade, so they are literal proof of the "external consumer" claim. Examples 1–10 are also condensed into rustdoc doctests on the corresponding types (`cargo test --workspace --doc`).

### 17.0 API additions

Everything §17 needs that §1–§15 did not spell out. These are additions to the accepted architecture; each is consistent with an existing rule and is listed here so no example invents a name.

```rust
// ---- A1. Application entry point (§3.3 named `Runtime<A>` and `app.update`/`app.draw`) ----
pub trait App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;
    fn draw(&self, ui: &mut Ui<'_>);
    fn should_quit(&self) -> bool { false }
    fn keymap(&self) -> &KeyMap { KeyMap::empty() }
    fn min_size(&self) -> Size { Size { min: (72, 20), preferred: (120, 40) } }
}
pub struct Runtime<A: App> { /* … */ }
impl<A: App> Runtime<A> {
    pub fn new(app: A, theme: Theme) -> Self;
    pub fn handle(&mut self, input: Input) -> Response<()>;
    pub fn draw(&mut self, frame: &mut Frame<'_>);
    pub fn app(&self) -> &A;  pub fn app_mut(&mut self) -> &mut A;
    pub fn area_of(&self, id: Id) -> Option<Rect>;
    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect>;
    pub fn ring(&self) -> &FocusRing;
    pub fn focus(&self) -> Option<Id>;
    pub fn diagnostics(&self) -> &[Diagnostic];
    pub fn set_theme(&mut self, t: Theme);
}
/// Owns the terminal session (raw mode, alt screen, mouse, bracketed paste, panic hook).
pub fn run<A: App>(app: A, theme: Theme) -> std::io::Result<()>;

// ---- A2. Cx / Ui members used below (§4 S3, §8.2, §8.5, §9.1 already name most) ----
impl Cx<'_> {
    pub fn intents(&mut self, id: Id) -> IntentIter<'_>;   // drains this owner's bucket; O(1) probe
    pub fn state(&self, id: Id) -> StateFlags;             // runtime-resolved focus/hover/press
    pub fn area(&self, id: Id) -> Option<Rect>;            // LAST frame's geometry, None on frame 1
    pub fn layout(&self, id: Id) -> Option<LayoutFacts>;
    pub fn theme(&self) -> &Theme;
    pub fn quit(&mut self);
}
impl Ui<'_> {
    pub fn state(&self, id: Id) -> StateFlags;
    pub fn theme(&self) -> &Theme;
    pub fn design(&self) -> &DesignTokens;
    pub fn style(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved;
    pub fn with_area<R>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn with_overlay<R>(&mut self, ov: &Overlay, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn report_layout(&mut self, id: Id, l: LayoutFacts);
    pub fn dim_layer(&mut self, area: Rect, steps: u8);
}
pub struct LayoutFacts { pub viewport_len: usize, pub content_len: usize, pub rows: u16, pub cols: u16 }

// ---- A3. Controlled-value phase signature (§4 rule 4, made explicit) ----
// Components with a controlled value take it as a third `update` parameter and a
// borrowed field on the props for `draw`:
//   fn update(&self, cx: &mut Cx<'_>, st: &mut TextInputState, value: &mut String) -> Response<TextAction>
//   fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect   // value borrowed in props

// ---- A4. Action identity for dialogs, menus and cell actions (referenced by §6.1, §9.1, §12.3) ----
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ActionKey(u16);
impl ActionKey {
    pub const CONFIRM: ActionKey; pub const CANCEL: ActionKey; pub const CLOSE: ActionKey;
    pub const SAVE: ActionKey;    pub const DISCARD: ActionKey; pub const RETRY: ActionKey;
    pub const fn custom(name: &'static str) -> ActionKey;   // FNV into the high range
}
pub struct Action<'a> { /* key, label, variant, chord, enabled, danger */ }
impl<'a> Action<'a> {
    pub const fn new(key: ActionKey, label: &'a str) -> Self;
    pub const fn danger(key: ActionKey, label: &'a str) -> Self;
    pub const fn quiet(key: ActionKey, label: &'a str) -> Self;
    pub const fn chord(self, c: Chord) -> Self;
    pub const fn enabled(self, yes: bool) -> Self;          // the §9.2 arming predicate, evaluated in update
}

// ---- A5. Theme builder and recipe editor (§11.2 declared the entry points only) ----
pub struct ThemeBuilder { /* … */ }
impl ThemeBuilder {
    pub fn accent(self, c: Color) -> Self;      pub fn danger(self, c: Color) -> Self;
    pub fn warning(self, c: Color) -> Self;     pub fn success(self, c: Color) -> Self;
    pub fn info(self, c: Color) -> Self;
    pub fn surfaces(self, s: [Color; SURFACE_LEVELS]) -> Self;
    pub fn fg(self, f: [Color; FG_STEPS]) -> Self;
    pub fn borders(self, subtle: Color, strong: Color) -> Self;
    pub fn borders_set(self, b: BorderSet) -> Self;
    pub fn glyph(self, r: GlyphRole, s: &'static str) -> Self;
    pub fn space(self, s: SpaceTokens) -> Self; pub fn size(self, s: SizeTokens) -> Self;
    pub fn density(self, d: Density) -> Self;
    pub fn motion(self, m: MotionTokens) -> Self;
    /// Fills every token the caller did not set by deriving from the ones they did,
    /// preserving DESIGN.md's contrast relationships. Deterministic and tested.
    pub fn build(self) -> Theme;
}
pub struct RecipeEdit { /* … */ }
impl RecipeEdit {
    pub fn default_variant(&mut self, v: Variant) -> &mut Self;
    pub fn part(&mut self, p: Part) -> &mut PartEdit;
}
pub struct PartEdit { /* … */ }
impl PartEdit {
    pub fn base(&mut self, p: StylePatch) -> &mut Self;
    pub fn when(&mut self, s: StateFlags, p: StylePatch) -> &mut Self;   // stored pre-sorted (§20.9-1)
    pub fn glyph(&mut self, g: GlyphRole) -> &mut Self;
    pub fn size(&mut self, n: u16) -> &mut Self;
}
impl Overlay {
    /// Scope override, const-constructible, borrowed, never mutates the theme.
    pub const fn new(rules: &'static [(Family, Variant, Part, StateFlags, StylePatch)]) -> Overlay;
    pub const EMPTY: Overlay;
}

// ---- A6. Layer construction conveniences (§9.1 gave the struct) ----
impl LayerSpec {
    pub const fn modal(owner: Id) -> LayerSpec;          // Modal, Screen(Center), esc+outside, dim, inert
    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec;  // Popover, pointer barrier only
    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec;
}

// ---- A7. Component constructors and builders used below (§13 fixes the conventions) ----
impl<'a> Button<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL, Part::ICON];
    pub fn new(id: Id, label: &'a str) -> Self;
    pub fn variant(self, v: Variant) -> Self;      pub fn disabled(self, yes: bool) -> Self;
    pub fn icon(self, g: GlyphRole) -> Self;       pub fn autofocus(self) -> Self;
    pub fn patch(self, p: &'a StylePatch) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn slot(self, p: Part, f: &'a dyn Fn(&mut Ui<'_>, Rect)) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>) -> Response<Activated>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl<'a> TextInput<'a> {
    pub const PARTS: &'static [Part] = &[Part::FIELD, Part::TEXT, Part::PLACEHOLDER, Part::MARKER];
    pub fn new(id: Id) -> Self;
    pub fn value(self, v: &'a str) -> Self;                 // for draw
    pub fn placeholder(self, s: &'a str) -> Self;
    pub fn validate(self, v: &'a dyn Validate) -> Self;
    pub fn blur(self, p: BlurPolicy) -> Self;
    pub fn secret(self, policy: SecretPolicy) -> Self;
    pub fn read_only(self, yes: bool) -> Self;   pub fn disabled(self, yes: bool) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TextInputState, value: &mut String)
        -> Response<TextAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect;
}
impl<'a, C> Field<'a, C> {
    pub fn new(id: Id, label: &'a str, control: C) -> Self;
    pub fn required(self, yes: bool) -> Self;   pub fn help(self, s: &'a str) -> Self;
    pub fn error(self, s: Option<&'a str>) -> Self;
    // update/draw forward to the control and own the chrome
}
impl<'a, T, K, R> List<'a, T, K, R>
where K: Fn(&T) -> ItemKey, R: Fn(&T, &mut RowUi<'_>) {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER,
                                         Part::LABEL, Part::META, Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn new(id: Id, items: &'a [T]) -> List<'a, T, (), ()>;
    pub fn key(self, k: K) -> List<'a, T, K, R>;
    pub fn row(self, r: R) -> List<'a, T, K, R>;
    pub fn select_mode(self, m: SelectMode) -> Self;
    pub fn empty(self, e: EmptyState<'a>) -> Self;
    pub fn disabled_item(self, f: &'a dyn Fn(&T) -> bool) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState) -> Response<ListAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ListState) -> Rect;
}
impl<'a, T, K, R> Tabs<'a, T, K, R>
where K: Fn(&T) -> ItemKey, R: Fn(&T, &mut RowUi<'_>) {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::TAB, Part::CLOSE, Part::NEW,
                                         Part::RULE, Part::OVERFLOW, Part::BADGE];
    pub fn new(id: Id, items: &'a [T]) -> Tabs<'a, T, (), ()>;
    pub fn key(self, k: K) -> Self;  pub fn row(self, r: R) -> Self;   // Part::TAB pre-styled
    pub fn allow_new(self, yes: bool) -> Self;  pub fn closable(self, yes: bool) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState) -> Response<TabsAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TabsState) -> Rect;
}
impl<'a> Dialog<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::BORDER, Part::TITLE,
                                         Part::DETAIL, Part::BODY, Part::ACTIONS, Part::BACKDROP];
    pub fn new(id: Id) -> Self;
    pub fn title(self, s: &'a str) -> Self;
    pub fn description(self, s: &'a str) -> Self;
    pub fn actions(self, a: &'a [Action<'a>]) -> Self;
    pub fn cancel(self, k: ActionKey) -> Self;
    pub fn width(self, w: u16) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut DialogState) -> Response<DialogAction>;
    pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, st: &DialogState,
                   body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>;
    // convenience constructors over the same path (§9.2)
    pub fn confirm(id: Id, title: &'a str, question: &'a str) -> Self;
    pub fn destructive(id: Id, title: &'a str, question: &'a str) -> Self;
    pub fn prompt(id: Id, title: &'a str, label: &'a str) -> Self;
    pub fn acknowledge(id: Id, title: &'a str, token: &'a str) -> Self;
    pub fn facts(id: Id, title: &'a str, props: &'a [(&'a str, &'a str)]) -> Self;
    pub fn choice(id: Id, title: &'a str, options: &'a [&'a str]) -> Self;
    pub fn info(id: Id, title: &'a str) -> Self;
}
impl<'a, T, K, R> Picker<'a, T, K, R> { /* new(id) .items(&'a [T]) .key(K) .row(R)
                                           .query(&'a str) .scopes(&'a [ScopeKey])
                                           .status(EmptyState) .update(cx,&mut PickerState)
                                           .draw(ui,area,&PickerState) */ }
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct ScopeKey(u16);
pub struct DialogState  { /* action cursor, prompt draft */ }
pub struct PickerState  { /* query editor core, cursor, scroll, active scope */ }
pub struct ListState    { /* cursor key, checked KeySet, scroll, gen stamp */ }
pub struct TabsState    { /* active key, cursor key, strip window, gen stamp */ }
pub struct TextInputState { /* draft, editor core, phase, error */ }
```

---

**1 — Default button** (`examples/01_button.rs`)

```rust
use junie_tui::{id, run, App, Button, Cx, Id, Response, Ui, Theme, layout};

const SAVE: Id = id!("save");

#[derive(Default)]
struct Demo { saves: u32 }

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save").update(cx).on_activated(|| self.saves += 1)
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = layout::inset(ui.full(), Insets { l: 2, t: 1, r: 2, b: 1 });
        Button::new(SAVE, "Save").draw(ui, area);
    }
}

fn main() -> std::io::Result<()> { run(Demo::default(), Theme::junie()) }
```

Nothing registers a hit region, computes hover, tracks press, or places focus: `Button::draw` calls `ui.register_control(SAVE, area, Focusability::Focusable)` and the runtime does the rest (G3).

**2 — A complete custom theme** (`examples/02_custom_theme.rs`)

```rust
use junie_tui::theme::{BorderSet, ColorTokens, Density, FgStep, MeterTokens, SyntaxTokens, Theme};
use ratatui::style::Color::Rgb as rgb;

fn slate() -> Theme {
    Theme::from_tokens(ColorTokens {
        surfaces: [rgb(10,12,16), rgb(16,19,26), rgb(23,27,36), rgb(31,36,48), rgb(40,46,61)],
        field: rgb(16,19,26), field_hover: rgb(23,27,36),
        fg: [rgb(232,236,244), rgb(186,193,208), rgb(140,148,166), rgb(97,105,124), rgb(64,71,88)],
        on_accent: rgb(8,10,14), on_danger: rgb(255,245,245), on_surface_inverse: rgb(10,12,16),
        border_subtle: rgb(31,36,48), border_strong: rgb(72,80,98),
        accent: rgb(122,162,247), accent_hover: rgb(147,180,250), accent_pressed: rgb(96,138,232),
        accent_tint: rgb(22,32,54),
        focus: rgb(122,162,247), focus_ring: rgb(96,138,232),
        selection_bg: rgb(31,44,72), selection_fg: rgb(232,236,244),
        highlight_bg: rgb(38,52,84), highlight_fg: rgb(232,236,244),
        highlight_danger_bg: rgb(84,32,38), highlight_danger_fg: rgb(255,235,235),
        backdrop_fg: rgb(64,71,88), backdrop_bg: rgb(8,10,14),
        danger: rgb(240,110,120), danger_soft: rgb(96,42,50), danger_tint: rgb(48,22,28),
        warning: rgb(224,168,80), warning_tint: rgb(56,42,20),
        success: rgb(126,200,140), info: rgb(120,180,220),
        disabled_fg: rgb(74,82,100), disabled_bg: rgb(16,19,26), read_only_fg: rgb(140,148,166),
        syntax: SyntaxTokens::derive(rgb(122,162,247), rgb(126,200,140), rgb(224,168,80)),
        meter:  MeterTokens::derive(rgb(126,200,140), rgb(224,168,80), rgb(240,110,120)),
    })
    .builder()
    .borders_set(BorderSet::SQUARE)
    .density(Density::Compact)
    .build()
}
// `Theme::from_tokens` fills design tokens and recipe defaults; `downgrade` works for it
// exactly as for `junie()`, because `map_colors` is an exhaustive destructure (§11.4).
```

**3 — Partial theme override** (`examples/03_partial_theme.rs`)

```rust
use junie_tui::Theme;
use ratatui::style::Color::Rgb as rgb;

// Change three roles; everything else is inherited from Junie, unchanged, byte-for-byte.
let t = Theme::junie()
    .builder()
    .accent(rgb(0xC6, 0x7A, 0x2E))   // amber instead of green
    .focus(rgb(0xC6, 0x7A, 0x2E))
    .danger(rgb(0xB0, 0x25, 0x25))
    .build();

assert_eq!(t.color.surfaces, Theme::junie().color.surfaces);   // untouched roles inherit
```

**4 — Global family recipe override** (`examples/04_family_recipe.rs`)

```rust
use junie_tui::{Family, GlyphRole, Modifier, Part, Role, StateFlags, StylePatch, Theme, Variant};

// Every Button in the application: square gutter marker, bold label when focused,
// tinted container when hovered. No component source is edited.
let t = Theme::junie().override_family(Family::BUTTON, |r| {
    r.default_variant(Variant::SECONDARY);
    r.part(Part::GUTTER).glyph(GlyphRole::FocusBar);
    r.part(Part::LABEL)
        .base(StylePatch::new().set_fg(Role::Fg(FgStep::Primary)))
        .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD))
        .when(StateFlags::DISABLED, StylePatch::new().set_fg(Role::DisabledFg).remove(Modifier::BOLD));
    r.part(Part::CONTAINER)
        .when(StateFlags::HOVERED, StylePatch::new().set_bg(Role::AccentTint))
        .when(StateFlags::HOVERED | StateFlags::PRESSED,
              StylePatch::new().set_bg(Role::AccentPressed).set_fg(Role::OnAccent));
});
// The two-flag rule wins over the one-flag rule by `when.count_ones()` (§11.3 step 3),
// and the rules are stored pre-sorted so resolution never allocates (§20.9-1).
```

**5 — Per-instance part override** (`examples/05_instance_patch.rs`) — Scenario C

```rust
use junie_tui::{id, Button, Id, Part, Role, StylePatch, Ui, Variant, layout, Rect, Track};

const OK: Id = id!("ok");
const RESET: Id = id!("reset");

// One patch, declared `const`, so it costs nothing per frame.
const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL,  StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

fn draw_actions(ui: &mut Ui<'_>, area: Rect) {
    let cols = layout::action_row(area, &[10, 12], ui.design().space.gap, RowAlign::Right);
    Button::new(OK, "OK").variant(Variant::PRIMARY).draw(ui, cols[0]);
    Button::new(RESET, "Reset").patch_part(&RESET_LABEL).draw(ui, cols[1]);
}
// Both buttons use the same global theme and the same renderer; only one is patched,
// and `conformance::button::local_override_does_not_mutate_the_theme` proves the
// theme is byte-identical afterwards.
```

**6 — Text field with external validation** (`examples/06_validated_field.rs`)

```rust
use junie_tui::{id, BlurPolicy, Cx, Field, FieldError, Id, Response, TextAction,
                TextInput, TextInputState, Ui, Rect};

const EMAIL: Id = id!("email");

struct Form {
    email: String,                 // the controlled value — the caller owns it
    email_st: TextInputState,      // durable interaction state only
    server_error: Option<String>,  // async result from the application
}

fn valid_email(s: &str) -> Result<(), FieldError> {
    if s.contains('@') { Ok(()) }
    else { Err(FieldError { message: "Enter a valid address".into(), code: Some("email") }) }
}

impl Form {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = TextInput::new(EMAIL)
            .validate(&valid_email)                    // closure, via the blanket `Validate` impl
            .blur(BlurPolicy::CommitAndValidate)
            .update(cx, &mut self.email_st, &mut self.email);

        if let Some(TextAction::Committed) = r.action_ref() {
            self.server_error = check_uniqueness(&self.email);          // application effect
            self.email_st.set_error(self.server_error.as_deref()
                .map(|m| FieldError { message: m.to_owned().into(), code: Some("dup") }));
        }
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        Field::new(EMAIL, "Email", TextInput::new(EMAIL).value(&self.email))
            .required(true)
            .help("We only use this for sign-in.")
            .error(self.server_error.as_deref())
            .draw(ui, area, &self.email_st);
    }
}
// `draw` is `&self` and takes `&TextInputState`: committing or validating from draw is a
// compile error, which is what removes the five render-time commits of §1.2(5).
```

**7 — List with borrowed domain rows and a custom renderer** (`examples/07_borrowed_rows.rs`) — Scenario D

```rust
use junie_tui::{id, Cx, GlyphRole, Id, ItemKey, List, ListAction, ListState, Response,
                Role, RowUi, SelectMode, Ui, Rect, EmptyState};

pub struct Order { pub id: u64, pub customer: String, pub total_cents: i64, pub flagged: bool }

const ORDERS: Id = id!("orders");

struct Screen { orders: Vec<Order>, list: ListState, chosen: Option<u64> }

fn orders_view<'a>(rows: &'a [Order]) -> impl 'a + Fn(&Order, &mut RowUi<'_>) {
    |o: &Order, r: &mut RowUi<'_>| {
        if o.flagged { r.marker(GlyphRole::WarningMark); }
        r.label(&o.customer);                        // borrowed &str, one grapheme walk, 0 allocs
        let mut c = r.part(Part::META, 12);
        c.money(o.total_cents).align(Align::Right)   // formats into the cell, no String
         .tone(if o.total_cents < 0 { Role::Danger } else { Role::Fg(FgStep::Muted) });
    }
}

impl Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        List::new(ORDERS, &self.orders)
            .key(|o: &Order| ItemKey::num(o.id))
            .row(orders_view(&self.orders))
            .select_mode(SelectMode::Single)
            .update(cx, &mut self.list)
            .on_action(|a| if let ListAction::Chose(k) = a {
                self.chosen = self.orders.iter().find(|o| ItemKey::num(o.id) == k).map(|o| o.id);
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        List::new(ORDERS, &self.orders)
            .key(|o: &Order| ItemKey::num(o.id))
            .row(orders_view(&self.orders))
            .empty(EmptyState::Empty { title: "No orders", hint: Some("Adjust the filter") })
            .draw(ui, area, &self.list);
    }
}
// Nothing is converted to owned strings, only visible rows invoke the renderer, and the
// action carries `ItemKey`, never a display index.
```

**8 — Dynamic tabs with stable keys** (`examples/08_dynamic_tabs.rs`) — Scenario E

```rust
use junie_tui::{id, Cx, Id, ItemKey, Response, RowUi, Tabs, TabsAction, TabsState, Ui, Rect, GlyphRole};

pub struct Doc { pub key: u64, pub title: String, pub dirty: bool }
const STRIP: Id = id!("strip");

struct Workspace { docs: Vec<Doc>, strip: TabsState, next_key: u64 }

fn tab_view(d: &Doc, r: &mut RowUi<'_>) {
    r.label(&d.title);
    if d.dirty { r.marker(GlyphRole::Dirty); }
}

impl Workspace {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Tabs::new(STRIP, &self.docs)
            .key(|d: &Doc| ItemKey::num(d.key))
            .row(tab_view)
            .allow_new(true)
            .closable(true)
            .update(cx, &mut self.strip)          // reconcile() runs first, every frame
            .on_action(|a| match a {
                TabsAction::Activated(k) => { /* the active key, not an index */ }
                TabsAction::Close(k)     => self.docs.retain(|d| ItemKey::num(d.key) != k),
                TabsAction::New          => {
                    self.next_key += 1;
                    self.docs.insert(0, Doc { key: self.next_key, title: "Untitled".into(), dirty: false });
                }
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        Tabs::new(STRIP, &self.docs).key(|d: &Doc| ItemKey::num(d.key)).row(tab_view)
            .draw(ui, area, &self.strip);
    }
}
// Insert at position 0: the active tab, the strip window and any pending close still name
// the same `ItemKey`. Nothing is rebuilt; `TabsState` is never reconstructed.
```

**9 — Composed dialog with an arbitrary body** (`examples/09_composed_dialog.rs`)

```rust
use junie_tui::{id, Action, ActionKey, Cx, Dialog, DialogAction, DialogState, DismissReason,
                Id, LayerSpec, Props, Response, Ui, layout};

const CONFIRM: Id = id!("confirm.delete");
const K_CANCEL: ActionKey = ActionKey::CANCEL;
const K_DELETE: ActionKey = ActionKey::custom("delete");

struct Screen { dlg: DialogState, token: String, target: String, deleted: bool }

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) { cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM)); }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let armed = self.token.trim() == self.target;      // arming is an `update` predicate
        let actions = [
            Action::new(K_CANCEL, "Cancel"),
            Action::danger(K_DELETE, "Delete").enabled(armed),
        ];
        let r = Dialog::new(CONFIRM)
            .title("Delete table")
            .description("This cannot be undone.")
            .actions(&actions)
            .cancel(K_CANCEL)
            .update(cx, &mut self.dlg);

        match r.action_ref() {
            Some(DialogAction::Action(k)) if *k == K_DELETE => { self.deleted = true; cx.close_layer(CONFIRM, Some(K_DELETE)); }
            Some(DialogAction::Action(_)) | Some(DialogAction::Dismissed(DismissReason::Esc)) =>
                cx.close_layer(CONFIRM, Some(K_CANCEL)),
            _ => {}
        }
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(CONFIRM, |ui, area| {
            Dialog::new(CONFIRM).title("Delete table").width(60)
                .draw(ui, area, &self.dlg, |ui, body| {           // ARBITRARY body content
                    let rows = layout::rows(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)]);
                    Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")]).draw(ui, rows[0]);
                    ui.rule(rows[1]);
                    TextInput::new(CONFIRM.part(Part::FIELD)).value(&self.token)
                        .placeholder("Type the table name to confirm").draw(ui, rows[2], &self.token_st);
                });
        });
    }
}
// `DialogBody` does not exist. The body is a closure that borrows application data.
// Focus trapping, backdrop, Esc, click-outside, focus restore and the hint layer come
// from the layer, not from the dialog.
```

**10 — Nested picker inside a dialog** (`examples/10_nested_overlay.rs`) — Scenario F

```rust
use junie_tui::{id, Anchor, Cx, Id, ItemKey, LayerEvent, LayerSpec, Picker, PickerAction,
                PickerState, Response, RowUi, Ui, Side, CrossAlign};

const DLG: Id = id!("dlg");
const OWNER_PICK: Id = id!("dlg.owner_picker");

struct Screen { dlg: DialogState, pick: PickerState, people: Vec<Person>, owner: Option<u64> }

impl Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        // the picker opens ON TOP of the dialog; the dialog is now pointer- and key-inert
        r |= Button::new(DLG.part(Part::custom("owner")), "Choose owner…").update(cx)
                .on_activated(|| cx.open_layer(OWNER_PICK, LayerSpec {
                    anchor: Anchor::Rect { rect: cx.area(DLG.part(Part::custom("owner"))).unwrap_or_default(),
                                           side: Side::Below, align: CrossAlign::Start },
                    ..LayerSpec::modal(OWNER_PICK)
                }));

        if cx.is_open(OWNER_PICK) {
            r |= Picker::new(OWNER_PICK).items(&self.people)
                    .key(|p: &Person| ItemKey::num(p.id))
                    .row(|p: &Person, u: &mut RowUi<'_>| { u.label(&p.name); u.meta(&p.team); })
                    .update(cx, &mut self.pick)
                    .on_action(|a| if let PickerAction::Chosen(k) = a {
                        self.owner = self.people.iter().find(|p| ItemKey::num(p.id) == k).map(|p| p.id);
                        cx.close_layer(OWNER_PICK, Some(ActionKey::CONFIRM));
                    });
        }

        if let Some(LayerEvent::Dismissed(_)) = cx.layer_event(OWNER_PICK) { /* nothing to undo */ }
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DLG, |ui, a| { /* dialog body, incl. the "Choose owner…" button */ });
        ui.layer(OWNER_PICK, |ui, a| { Picker::new(OWNER_PICK)./*…*/draw(ui, a, &self.pick); });
    }
}
// Esc closes only the picker; the dialog stays open and regains focus at the button.
// No barrier is pushed by hand, no hit region is re-registered, and the picker draws no
// hint row of its own — the top layer contributes to the shared HintBar (§11.4 of §13.1).
```

**11 — A small complete application on shared focus and dispatch** (`examples/11_small_app.rs`) — Scenario A

```rust
use junie_tui::{id, layout, run, Action, ActionKey, App, Button, Cx, Dialog, DialogAction,
                DialogState, Field, Id, Insets, ItemKey, LayerSpec, List, ListAction, ListState,
                Response, RowUi, Rect, TextInput, TextInputState, Theme, Track, Ui, Variant};

const NAME:   Id = id!("name");
const ADD:    Id = id!("add");
const PEOPLE: Id = id!("people");
const CONFIRM:Id = id!("confirm");
const K_YES: ActionKey = ActionKey::CONFIRM;
const K_NO:  ActionKey = ActionKey::CANCEL;

#[derive(Default)]
struct Roster {
    name: String, name_st: TextInputState,
    people: Vec<String>, list: ListState,
    dlg: DialogState, pending_remove: Option<ItemKey>,
    quit: bool,
}

impl App for Roster {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        r |= TextInput::new(NAME).update(cx, &mut self.name_st, &mut self.name).erase();

        r |= Button::new(ADD, "Add").variant(Variant::PRIMARY)
                .disabled(self.name.trim().is_empty())
                .update(cx)
                .on_activated(|| { self.people.push(std::mem::take(&mut self.name)); });

        r |= List::new(PEOPLE, &self.people)
                .key(|s: &String| ItemKey::text(s))
                .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
                .update(cx, &mut self.list)
                .on_action(|a| if let ListAction::Activated(k) = a {
                    self.pending_remove = Some(k);
                    cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM));
                });

        if cx.is_open(CONFIRM) {
            let actions = [Action::new(K_NO, "Cancel"), Action::danger(K_YES, "Remove")];
            r |= Dialog::destructive(CONFIRM, "Remove person", "Remove this person from the roster?")
                    .actions(&actions).cancel(K_NO)
                    .update(cx, &mut self.dlg)
                    .on_action(|a| {
                        if let DialogAction::Action(K_YES) = a {
                            if let Some(k) = self.pending_remove.take() {
                                self.people.retain(|s| ItemKey::text(s) != k);
                            }
                        }
                        cx.close_layer(CONFIRM, None);
                    });
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let body = layout::inset(ui.full(), Insets { l: 2, t: 1, r: 2, b: 1 });
        let rows = layout::rows(&body, &[Track::Fixed(3), Track::Fixed(1), Track::Flex(1)]);
        let top  = layout::columns(rows[0], &[Track::Flex(1), Track::Fixed(10)], ui.design().space.gap);

        Field::new(NAME, "Name", TextInput::new(NAME).value(&self.name)).draw(ui, top[0], &self.name_st);
        Button::new(ADD, "Add").variant(Variant::PRIMARY)
            .disabled(self.name.trim().is_empty()).draw(ui, top[1]);
        List::new(PEOPLE, &self.people)
            .key(|s: &String| ItemKey::text(s))
            .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
            .draw(ui, rows[2], &self.list);

        ui.layer(CONFIRM, |ui, a| {
            Dialog::destructive(CONFIRM, "Remove person", "Remove this person from the roster?")
                .draw(ui, a, &self.dlg, |_, _| {});
        });
    }

    fn should_quit(&self) -> bool { self.quit }
}

fn main() -> std::io::Result<()> { run(Roster::default(), Theme::junie()) }
```

The Scenario A checklist is satisfied by omission: there is no hit region, no mouse coordinate, no hover or pressed field, no derived child id, no Tab implementation, no modal barrier, no focus save/restore, no `set_cursor_position`, and no "which row was clicked" arithmetic.

**12 — A downstream component using only `junie_tui::author`** (`examples/12_author_component.rs`) — Scenario G

```rust
use junie_tui::author::{Cx, Family, Focusability, GlyphRole, Id, Intent, ItemKey, Part, PartRef,
                        Phase, Rect, Response, StateFlags, Ui, Variant, Chord, Binding, BindingState};

/// A segmented control: N labelled segments, one selected, roving cursor.
pub struct Segmented<'a> { id: Id, labels: &'a [&'a str], variant: Variant }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SegmentedState { pub cursor: usize, pub selected: usize }

#[derive(Clone, Copy, Debug)] pub enum SegmentedAction { Moved, Selected(ItemKey) }

const SEGMENT: Part = Part::custom("segment");
const F_SEGMENTED: Family = Family::custom("segmented");

const BINDINGS: &[Binding<SegmentedAction>] = &[
    Binding { chord: Chord::key(KeyCode::Left),  action: SegmentedAction::Moved, label: "Prev", priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Right), action: SegmentedAction::Moved, label: "Next", priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Enter), action: SegmentedAction::Moved, label: "Select", priority: 80, visible: true },
];

impl<'a> Segmented<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, SEGMENT, Part::LABEL];
    pub fn new(id: Id, labels: &'a [&'a str]) -> Self { Self { id, labels, variant: Variant::DEFAULT } }
    pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }

    pub fn bindings(_s: BindingState) -> &'static [Binding<SegmentedAction>] { BINDINGS }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SegmentedState) -> Response<SegmentedAction> {
        let mut r = Response::ignored().for_id(self.id);
        let n = self.labels.len();
        if n == 0 { return r; }
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) if k.is(KeyCode::Left)  => { st.cursor = (st.cursor + n - 1) % n; r = Response::action(SegmentedAction::Moved); }
                Intent::Key(k) if k.is(KeyCode::Right) => { st.cursor = (st.cursor + 1) % n;     r = Response::action(SegmentedAction::Moved); }
                Intent::Key(k) if k.is(KeyCode::Enter) || k.is(KeyCode::Char(' ')) => {
                    st.selected = st.cursor;
                    r = Response::action(SegmentedAction::Selected(ItemKey::index(st.selected)));
                }
                Intent::Pointer { phase: Phase::Click, part: PartRef { part, item: Some(k) }, .. }
                    if part == SEGMENT => {
                        if let ItemKey::Index(i) = k { st.cursor = i; st.selected = i; }
                        r = Response::action(SegmentedAction::Selected(k));
                    }
                _ => {}
            }
        }
        r.for_id(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SegmentedState) -> Rect {
        if area.is_empty() { return area; }                       // registers nothing (R5)
        ui.register_control(self.id, area, Focusability::Focusable);
        let live = ui.state(self.id);
        let w = area.width / self.labels.len().max(1) as u16;
        for (i, label) in self.labels.iter().enumerate() {
            let cell = Rect { x: area.x + w * i as u16, y: area.y, width: w, height: area.height };
            let mut s = live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE | StateFlags::HOVERED);
            if i == st.selected { s |= StateFlags::SELECTED; }
            if i == st.cursor && live.contains(StateFlags::FOCUSED) { s |= StateFlags::ACTIVE; }
            let r = ui.style(F_SEGMENTED, self.variant, SEGMENT, s);
            ui.fill(cell, r.style);
            if s.contains(StateFlags::SELECTED) { ui.glyph(cell, GlyphRole::Chosen, r.style); }
            ui.paint_str(cell, label, ui.style(F_SEGMENTED, self.variant, Part::LABEL, s).style);
            ui.register_part(self.id, PartRef { part: SEGMENT, item: Some(ItemKey::index(i)) }, cell);
        }
        area
    }
}
// Theme resolution, focus, hover, press, dispatch, hit testing, capture, cursor, layers,
// digest testing and the conformance suite are all reachable from `author::` with no
// private access. Register it once and the whole §16.2 matrix runs against it:
//   impl Conformance for SegmentedCase { … }   conformance_suite!(…, SegmentedCase);
```

---

## 18. Migration map

Every current module, every foundation file, and every app-side reusable control from **[F]** APP §3, with its disposition. `crates/tui/src/` is abbreviated `«tui»`; `apps/<app>/src/` is abbreviated `«showcase»`, `«tablepro»`, `«jackin»`.

### 18.1 Foundation modules

| Current item | Disposition | Target type(s) | Target file | Notes — what is deleted, which invariant it now satisfies |
|---|---|---|---|---|
| `src/core/event.rs` | **decompose** | `Input`, `Key`, `Chord`, `Mouse`, `MouseKind`, `Axis` | `«tui»/event.rs` | `Outcome` **deleted** (replaced by `Response`, §6.1). `Mouse` gains `mods`; `Secondary`/`SecondaryUp`/`Wheel(Axis,i16)` added; right/middle drag no longer silently dropped. Satisfies §6.1, INT A2. |
| — (new, split out of `event.rs`) | **compose** | `Response<A>`, `Flow`, `Invalidate`, `StateFlags` | `«tui»/response.rs` | The nine reply shapes of §1.2(2) collapse to one. `#[must_use]`. Satisfies G1, `DESIGN.md:507` boundary-wheel rule. |
| — (new) | **compose** | `Intent`, `PartRef`, `Phase`, `FocusVia`, `IntentIter` | `«tui»/intent.rs` | Pre-resolved input. Removes `owns`/`locate` from the model (§3.3 step 3). |
| — (new) | **compose** | `Binding`, `Bindings`, `KeyMap`, `Phase2`, `HintLayer`, `Hint` | `«tui»/keymap.rs` | Hard-coded product chords in 18 modules (**[F]** API §3.9) become `const` tables + an app override layer (§13.1). |
| `src/core/focus.rs` | **refactor** | `FocusRing`, `FocusEntry`, `FocusState`, `ScopeId`, `ScopeMode`, `Focusability`, `FocusVis` | `«tui»/focus.rs` | Single `barrier: Option<usize>` **deleted** → scopes + traps. `Focus`/`FocusRing` no longer public `&mut` to components. Adds restore map, the (a)(b)(c)(d) reconcile rule, disabled-but-registered entries. Satisfies §8.1, Scenario F. |
| `src/core/hit.rs` | **refactor** | `Registry`, `Region`, `Hit`, `Headroom`, `Axes` | `«tui»/hit.rs` | Barrier **deleted** → `layer: LayerId` per region. Regions carry `PartRef` (24 B, `Copy`), so `area_of`-by-render-order and all 12 `locate` helpers die. `hit_scroll` returns a region at zero headroom. Satisfies §8.3, §7.1. |
| — (new, split out of `hit.rs`) | **compose** | `Capture` | `«tui»/capture.rs` | Deletes cached-rect reconstruction in `list`/`tree`/`viewport`/`splitter`. Satisfies §8.2. |
| `src/core/id.rs` | **refactor** | `Id`, `ItemKey`, `Part`, `PartRef`, `DebugLabel`, `id!` | `«tui»/id.rs` | `WidgetId` **deleted**; `child(usize)` **deleted** as the default child mechanism (`Id::index` survives, debug-asserted). Separator + kind byte make `root("a").sub("b") != root("ab")` an identity. `Debug` prints a path. Satisfies §7.1, G5, Scenario E. |
| `src/core/scroll.rs` | **retain + tighten** | `ScrollState` | `«tui»/scroll.rs` | Public fields **removed** (kills the 8 bypass sites in `grid`/`table`); adds `ensure_visible_on_next_layout` (generalises `Picker::cursor_dirty`); column-axis misuse replaced by the grid's own column window. Satisfies §8.3. |
| `src/core/text.rs` | **decompose** | `TextEditorCore`, `EditAction`, `EditOutcome`, `CursorPos` | `«tui»/text/{buffer.rs, editor.rs}` | `TextBuffer`'s derived `Debug`/`Clone` over raw bytes **removed** (manual redacting `Debug`); `zeroize` added; word-char set unified with the viewport; `line_count`/`pos_of` memoised behind the edit counter. Satisfies §15, API §5. |
| — (new, from `ui/text.rs`) | **move** | `width`, `wrap`, `fuzzy`, `thousands`, `truncate`, `truncate_middle` | `«tui»/text/{measure.rs, fuzzy.rs}` | `fit`/`fit_right` **removed from every render path** (R5); `fuzzy` returns grapheme indices into the *original* label, fixing the latent mis-highlight. Satisfies §20.9-7. |
| `src/runtime.rs` | **refactor** | `Runtime<A>`, `App`, `TerminalSession`, `run`, `drain_pending_input`, `Diagnostic` | `«tui»/runtime.rs`, `«tui»/diagnostics.rs` | `Application::render(&mut self)` **deleted** — the sanction for render-time mutation at the top of the stack. Adds the two-phase frame (§3.3), the layer compositor, `request_repaint_after`, and `diagnostics()`. Satisfies G2. |
| `src/theme.rs` | **decompose** | `Theme`, `ColorTokens`, `DesignTokens`, `Role`, `GlyphRole`, `GlyphSet`, `BorderSet`, `Recipe`, `Recipes`, `PartRecipe`, `StateRule`, `StylePatch`, `Slot`, `Overlay`, `Resolved`, `Family`, `Variant`, `Capability`, `ColorLevel` | `«tui»/theme/{mod,tokens,role,glyph,recipe,patch,resolve,downgrade}.rs`, `«tui»/theme/builtin/{junie,paper}.rs` | Flat 30-field `Copy` struct **deleted**; `lift`/`backdrop` colour-equality dispatch **deleted** (ladder index arithmetic, §10); the 30-field `for_level` macro **deleted** (`map_colors` exhaustive destructure); `Theme::change_glyph` (added from `grid.rs`) **deleted**. Junie token values preserved verbatim. Satisfies G6, §11. |
| `src/ui/ctx.rs` | **decompose** | `Ui`, `Cx`, `Surface`, `StyleStack`, `LayoutFacts` | `«tui»/ui/{mod,cx,paint,surface,layer_buf}.rs` | `RenderCtx`, `Interaction`, `begin_modal`, `focus_hidden` (dead), public `hits`/`ring` **all deleted**. Adds clip rect, surface stack, style stack, written-cell bitset, `raw()` escape hatch. Satisfies R2–R4, §10. |
| `src/ui/layout.rs` | **refactor + merge** | `layout::{rows, columns, responsive_columns, action_row, inset, split_v, split_h}`, `Track`, `Insets`, `RowAlign`, `SplitModel`, `Constraints`, `Size`, `Measure` | `«tui»/layout.rs`, `«tui»/measure.rs` | `Split`'s vertical/horizontal minima asymmetry **fixed** (first pane wins on both axes). Absorbs `button::row_layout*` and `showcase/pages/mod.rs`'s `rows`/`columns`/`caption`. Module doc's "the workbench" (domain leak) removed. |
| `src/ui/popup.rs` | **remove → compose** | `LayerId`, `LayerKind`, `LayerSpec`, `Anchor`, `Side`, `CrossAlign`, `Dismiss`, `Backdrop`, `LayerEvent`, `DismissReason` | `«tui»/layer.rs` | Both `Placement` enums and both placement algorithms **deleted**; the shared `WidgetId::of("popup.surface")` **deleted**; `Rect::centered` in `dialog.rs` **deleted**. One resolver: flip, clamp, clip, min-size. Satisfies §9. |
| `src/ui/text.rs` | **move** | see `text/measure.rs` above | `«tui»/text/` | Module removed; no `ui::text` namespace remains. |

### 18.2 The 31 widget modules

| Current module | Disposition | Target type(s) | Target file | Notes |
|---|---|---|---|---|
| `brand` | **retain + restyle** | `Lockup` | `«tui»/components/brand.rs` | "the only control that fills with the accent" moves from a doc rule in code to a `Theme::junie()` recipe default (§11.6). `pub area` removed (S1). |
| `button` | **refactor** | `Button<'a>` | `«tui»/components/button.rs` | `(Outcome,bool)`/`bool` → `Response<Activated>`; `bg: Color` → `Surface`; `pub area` → `cx.area(id)`; `row_layout*` moved to `layout::action_row`. Reference implementation of §13. |
| `chips` | **decompose** | `Chip<'a>`, `ChipBar<'a,T,K,R>`, `ChipBarState` | `«tui»/components/chip.rs` | Raw `ctx.ring.register` **deleted**; drop-out-of-ring-on-overflow bug fixed by `register_control` on the strip; `"+ Add filter"` / `"match all ▾"` TablePro defaults removed; keys via `ItemKey`. |
| `choice` | **decompose** | `Checkbox<'a>`, `Toggle<'a>`, `RadioGroup<'a,T,K,R>`, `RadioGroupState` | `«tui»/components/choice.rs` | Three `pub area`/`areas` removed; `RadioGroup::height()` deleted (`Field` measures); **cursor and value separated** (§20.10-3). |
| `code` | **refactor** | `CodeEditor<'a>`, `CodeEditorState`, `Highlighter`, `Segmenter`, `Diagnostic'` | `«tui»/components/code.rs` | Render-time commit (`code.rs:611`) **impossible** (`&self`); `fn`-pointer `Highlighter`/`Segmenter` → `&'a dyn Fn`; per-frame `hash_text` → edit counter; per-grapheme linear span scan → sorted-span cursor; vim key table → default `KeyMap`. Satisfies §20.9-9. |
| `completion` | **compose + controller** | `Completion<'a,T,K,R>`, `CompletionState`, `CompletionController` | `«tui»/components/completion.rs` | Becomes `Popover` layer content; missing `on_scrollbar` fixed by `scroll_region`; boundary-wheel violation (`completion.rs:142-145`) fixed; the ~40 lines of editor↔popup hand-wiring in `tablepro/tabs.rs:1326-1377` collapse into the controller. Shares `Item` with `Picker`. |
| `dialog` | **decompose** | `Dialog<'a>`, `DialogState`, `Action<'a>`, `ActionKey`, `DialogAction` | `«tui»/components/dialog.rs` | `DialogBody` **deleted**; polled `result` field **deleted**; `&mut Focus` parameters **deleted**; render-time ack arming (`dialog.rs:465`) **deleted** (an `update` predicate); backdrop loop **deleted** (one layer implementation); `dialog.rs:389`'s trap-less modal **fixed** (the trap belongs to the layer). Satisfies §9.2, goal §14. |
| `diff` | **retain (composition)** | `DiffView<'a>`, `DiffViewState`, `DiffSource`, `DiffMode` | `«tui»/components/diff.rs` | Data model moves behind `DiffSource` so jackin's `sim::changes::ChangedFile` feeds it without conversion; `review_lines(f,width)` becomes `measure`; the render-time `set_follow(false)`/`scroll_to` in the layout cache moves to `update`. |
| `empty` | **retain** | `EmptyState<'a>` rendering inside each collection | `«tui»/collection/empty.rs` | The free `render(…, bg)` **deleted**; empty/loading/partial/error become one vocabulary (absorbs `PickerStatus`). |
| `field_common` | **remove → refactor** | `EditAction` table on `TextEditorCore` + `Binding` set | `«tui»/text/editor.rs`, `«tui»/keymap.rs` | `EditAction::Apply(fn(&mut TextBuffer))` **deleted** (fn pointer, API §3.12). The shared keymap becomes a `const [Binding<EditAction>]`. |
| `grid` | **decompose** | `Grid<'a,M>`, `GridState`, `GridModel`, `GridEditor`, `GridCellActions`, `ColumnKey`, `ColumnSpec'`, `CellRef`, `NavUnit`, `EditIntent`, `CellAction` | `«tui»/components/grid.rs` | Everything DB-shaped moves out (see 18.4 TablePro row). `CellValue`, `PendingChanges`, `UndoAction`, `default_validator`, `cmp_cells`, `Validator` fn pointer, `"Preview SQL"`, `primary`/`nullable`/`references`/`enum_values`, `Theme::change_glyph` **all deleted from the library**. `col_rects.clone()` per row **deleted**. `GridEditor` is `&mut self` and unreachable from `draw`. Satisfies §12.3, Scenario H, DOM §7 condition 1. |
| `hintbar` | **retain + wire** | `HintBar`, `HintLayer`, `Hint` | `«tui»/components/hintbar.rs` | Layers are now *derived*: top layer ▸ mode ▸ focused component's visible bindings ▸ screen extras ▸ global. The ~700 lines of hand-written hint tables across the apps are deleted. |
| `input` | **decompose** | `TextInput<'a>`, `TextInputState`, `Secret`, `SecretPolicy`, `BlurPolicy` | `«tui»/components/input.rs`, `«tui»/components/secret.rs` | Render-time commit+validate (`input.rs:282`) **impossible**; `validator: Option<fn>` → `&dyn Validate`; `plain_label` **deleted** (`Field` owns chrome); `HEIGHT` **deleted** (`measure`); `reveal_tail` **re-specified** to a synthetic tail; 5 tiny-rect underflows fixed with `saturating_sub`. Satisfies §15, API §5. |
| `keyhint` | **retain** | `KeyHint` | `«tui»/components/keyhint.rs` | Rendered by `HintBar`; the free-function entry point stays for one-off chips. |
| `list` | **refactor** | `List<'a,T,K,R>`, `ListState`, `SelectMode`, `KeySet` | `«tui»/components/list.rs` | Owned `ListItem` **deleted** (borrowed `&'a [T]`); `row_id`/`locate`/`owns` **deleted**; boundary-wheel violation (`list.rs:180`) fixed; `SelectMode` gains `Range`/`None`; `KeySet` gets `AllExcept` (R7). Satisfies Scenario D. |
| `menu` | **retain + extend** | `MenuBar<'a>`, `ContextMenu<'a>`, `MenuItem<'a>`, `MenuState` | `«tui»/components/menu.rs` | Render-time cursor move on hover (`menu.rs:243`) → an explicit `Intent::Pointer{Move}`; `shortcut: &'static str` → a real `Chord` that both renders and binds (kills jackin's `run_host_menu` key synthesis); label-string dispatch → `ActionKey`; `MenuItem::submenu` added; own `Placement` merged into `Anchor`; becomes layer content. |
| `panel` | **split** | `Panel<'a>` retained; `ScrollPanel` **removed** | `«tui»/components/panel.rs` | `bg: Color`, `Panel::bg(t)` and `pub bg_override` **all deleted** (contextual `Surface`). Framed inner rect escaping the panel for `width ≤ 4` **fixed**. `ScrollPanel` callers migrate to `TextViewport` with tone-carrying spans. |
| `picker` | **decompose** | `FilterList<'a,T,K,R>` (headless), `Picker<'a,…>` overlay, `CommandPalette`, `PickerState`, `ScopeKey` | `«tui»/components/{filter_list,picker}.rs` | `hints: &str` **deleted** (a `HintLayer` contribution); positional `row_id` → `ItemKey`; `scopes` first-class; `PickerStatus` folded into `EmptyState`; `Delete`-secondary gains a mouse equivalent (§20.10-4); duplicated backdrop dim **deleted**. |
| `progress` | **decompose** | `Spinner`, `ProgressBar`, `Meter`, `MeterTone`, `MeterVisual` | `«tui»/components/{progress,meter}.rs` | Five `bg: Color` parameters **deleted**; `METER_LOW_MAX`/`MEDIUM_MAX` and `SPINNER` move to `DesignTokens` (A4); `MeterTone::{Warning,Exhausted,Stale,Refreshing}` (jackin quota lifecycle) move to jackin; `MeterTone::from_ratio` helper kills the app-side duplicate matches (J12). |
| `props` | **refactor** | `Props<'a>` (static) + `PropsList<'a,T,K,R>` as a two-column `List` variant | `«tui»/components/props.rs` | The **two independent render paths** (free fn vs `PropsList::render`) collapse to one; `locate`/`owns`/`row_id` deleted; used by `Dialog::facts`. |
| `scrollbar` | **retain as a part** | `Part::TRACK` / `Part::THUMB` of a `scroll_region` | `«tui»/components/scroll_region.rs` | `scrollbar::id_for` **deleted** (26 showcase + 18 tablepro + ≥4 jackin call sites). One `on_scrollbar` implementation replaces seven copies; thumb drag uses pointer capture. |
| `segments` | **merge** | absorbed by `StatusBar` | `«tui»/components/status.rs` | Two priority-drop loops become one `Left/Center/Right` item strip; `bg: Color` deleted. |
| `select` | **rebuild** | `Select<'a,T,K,R>`, `SelectState` | `«tui»/components/select.rs` | Render-time overlay close (`select.rs:161-167`) **impossible**; the popup becomes a `Popover` layer (so the focus barrier bug in `ui/popup.rs` disappears); 10-row clip → a real scroll region; `HEIGHT` deleted. |
| `splitter` | **merge** | `SplitPane<'a>`, `SplitPaneState` (with `SplitModel`) | `«tui»/components/split.rs` | `Splitter` + `ui::layout::Split` become one component that owns its container rect from its own draw; caller-held `seam_container: Rect` fields in three jackin screens **deleted**; drag through capture; optional keyboard resize as a binding. |
| `statusbar` | **retain + promote** | `StatusBar<'a>`, `StatusItem<'a>`, `Emphasis` | `«tui»/components/status.rs` | Absorbs `segments`; gains TablePro's identity strip and grid status line as consumers, deleting two hand-written priority-drop loops; `STATUS_METER_TRACK` moves to `design.size.meter_track`. |
| `steps` | **refactor** | `Steps<'a,T,K,R>`, `StepsState`, `StepState` | `«tui»/components/steps.rs` | Stays a *display* rail with a frontier (the meaningful difference, DOM §6.2); gains keys and a row renderer; the step *flow* becomes the separate `Wizard` (J7). |
| `table` | **remove** | absorbed by `Grid` with `NavUnit::{Row, Cell}` | — (`«tui»/components/grid.rs`) | `DataTable` **deleted**: its `Column`, `Cell`, third `EditState`, string sort, `validator: fn`, `locate`/`locate_header`, double cell registration, and 4 ragged-row panics all go. TablePro's Structure tab becomes six `GridModel`s. |
| `tabs` | **refactor** | `Tabs<'a,T,K,R>`, `TabsState`, `TabsAction` | `«tui»/components/tabs.rs` | Positional `tab_id(i)`/`close_id(i)` **deleted** → `ItemKey`; per-frame `areas`/`widths` `Vec`s **deleted**; the "rebuild the whole widget and rescue `first`/`active`" idiom in both apps **deleted**; strip window follows the logical first tab (§20.10-13). Satisfies Scenario E. |
| `textarea` | **refactor** | `TextArea<'a>`, `TextAreaState` | `«tui»/components/textarea.rs` | Render-time commit (`textarea.rs:202`) **impossible**; shares `TextEditorCore` with `input`/`code`; missing `owns`/`on_scrollbar` supplied by `scroll_region`; 1-cell-width underflow fixed. |
| `tree` | **refactor** | `Tree<'a,T,K,R>`, `TreeState`, `TreeNode<'a>`, `TreeAction` | `«tui»/components/tree.rs` | `path: Vec<usize>` identity → `ItemKey` (`TreeNode::keyed`); `expanded: HashSet<Vec<usize>>` → `HashSet<ItemKey>`; `flatten()` becomes **incremental and borrow-based** (§20.9-8); `FlatRow`'s duplicate `label`/`meta` **deleted**; row renderer added (kills TablePro's paint-over-the-tree hack); `object_at`/`schema_at` path reconstruction **deleted**. |
| `viewport` | **retain (rewritten storage)** | `TextViewport<'a>`, `ViewportState`, `Span<'a>`, `ViewportAction` | `«tui»/components/viewport.rs` | Best-in-class behaviour preserved verbatim. `Cell { g: String }` → `{ range: Range<u32>, w: u8, style_ix: u16 }`; layout becomes **incremental + windowed**; `set_area`/`prime`/the `inert` clone dance **deleted** once view state is caller-owned. Satisfies §20.9-7. |

### 18.3 The 23 app-side reusable controls (**[F]** APP §3)

| # | Current control | Disposition | Target type(s) | Target file | Notes |
|---|---|---|---|---|---|
| 1 | `NavList` + `NavItem` (`showcase/pages/sidebars.rs:16-165`) | **move** | `NavList<'a,T,K,R>`, `NavListState` | `«tui»/components/nav_list.rs` | Sections, collapsed icon-only mode, badges and disabled skipping become `List` features; the control's own `ctx.ring.register` and reverse `locate` scan are deleted. |
| 2 | Shell nav sidebar (`showcase/app.rs:868-926, 461-492, 696-698`) | **compose** | uses #1 | `«showcase»/app.rs` | `nav_index_at`'s 22-id reverse scan and the hand-written sidebar key table are deleted; the digest baseline now covers the sidebar (§20.10). |
| 3 | `static_field` (`showcase/pages/inputs.rs:65-106`) | **compose** | `TextInput` + `Field` + `.state_override(StateFlags)` | `«showcase»/pages/inputs.rs` | Needs the documented "render in state X without owning state" path: the showcase supplies a `TextInputState` fixture per cell. The fake cursor cell and manual underline are deleted. |
| 4 | Button state matrix (`showcase/pages/buttons.rs:143-176`) | **compose** | `Button` × `Variant` × `StateFlags` fixtures | `«showcase»/pages/buttons.rs` | Same mechanism as #3; the re-implemented renderer (`t.button` + `t.gutter` + `set_string`) is deleted. |
| 5 | Showcase footer hint row (`showcase/app.rs:1018-1077`) | **compose** | `HintBar` + `HintLayer` | `«showcase»/app.rs` | Uses the widget the other two apps already use; width budgeting comes from `HintBar`. |
| 6 | `layout::{caption,rows,columns}` (`showcase/pages/mod.rs:120-168`) | **move** | `layout::rows`, `layout::responsive_columns`, `Part::HELP` caption style | `«tui»/layout.rs` | The ad-hoc re-implementations in the other two apps are deleted. |
| 7 | State inspector panel (`showcase/app.rs:948-1012`) | **compose** | `Props` + `Runtime::diagnostics()`/`ring()`/`focus()` | `«showcase»/app.rs` | The *data source* becomes a supported debug API; three per-frame `area_of` full scans are deleted. |
| 8 | `FilterEditor` (`tablepro/app.rs:99-109, 1368-1433, 1648-1736, 2051-2086, 2337-2468`) | **compose** | `Dialog` + `Form` + `Field<Select>`/`Field<TextInput>` | `«tablepro»/filter_editor.rs` | The hand-drawn modal (dim loop, raw `Block`, six manual `hits.register` calls, twice-written Tab/BackTab) is deleted; `FilterOp` stays domain. |
| 9 | Status-segment priority dropper (`tablepro/tabs.rs:794-838`) | **compose** | `StatusBar` | `«tablepro»/tabs.rs` | The bespoke `while … remove lowest priority` loop is deleted. |
| 10 | Plan-tree metric columns (`tablepro/tabs.rs:1774-1852`) | **move** | `Tree` `.row(…)` with `RowUi::columns` | `«tui»/components/tree.rs` + `«tablepro»/tabs.rs` | The read-back-the-cell-background paint-over hack is deleted; metrics stay domain. |
| 11 | TablePro identity strip (`tablepro/app.rs:2189-2282`) | **compose** | `StatusBar` with clickable `StatusItem` ids | `«tablepro»/app.rs` | `STRIP_SAFE`/`STRIP_SCOPE`/`STRIP_CONN`/`STRIP_HELP` become `PartRef`s of one id. |
| 12 | `modal_frame` (`jackin/screens/modals.rs:36-96`) | **move** | `LayerSpec` + `Panel`/`Frame` chrome | `«tui»/layer.rs`, `«tui»/components/panel.rs` | Four copies of the same 40 lines deleted (J1); `hint_row` becomes the derived `HintBar` layer. |
| 13 | `FileBrowser` (`jackin/screens/modals.rs:117-563`) | **compose** | `Form` + `List` + `Field<Checkbox>` + `Dialog` | `«jackin»/screens/file_browser.rs` | `World.fs`/`github` lookups stay domain; the 6 derived child ids, the manual Tab fallback and the double-click emulation are deleted. |
| 14 | `ChoiceDialog` (`jackin/screens/modals.rs:569-783`) | **move** | `Dialog::choice(...)` | `«tui»/components/dialog.rs` | Composed body + `RadioGroup` + actions; the modular Left/Right button ring and `stepper(&str)` string patch are deleted (the stepper becomes `Wizard`). |
| 15 | `FormDialog` + `FormField` + `FieldKindW` (`jackin/screens/modals.rs:787-1541`) | **move** | `Form<'a>`, `FormState`, `Field<C>`, `FormAction` | `«tui»/components/form.rs` | J2, the strongest candidate. Ordered fields, visibility, focused-field scroll-into-view, per-field clipping, action row, error row and the open-select z-order fix all become library behaviour; the 22 lines of manual hit re-registration and the hand-written button ring are deleted. Three form engines (jackin `FormDialog`, tablepro `connections.rs`, tablepro `FilterEditor`) collapse to one. |
| 16 | `OpFlow` (`jackin/screens/modals.rs:1546-1943`) | **compose** | `PickerChain` (J8) | `«tui»/components/picker_chain.rs` + `«jackin»/screens/op_flow.rs` | Stage list, `EmptyState::Loading/Error` with retry, breadcrumb scope and back-one-step become library; the 1Password account/vault/item/field model stays domain. |
| 17 | `InfoDialog` (`jackin/screens/modals.rs:1947-2280`) | **move** | `Dialog::facts(...)` + `Props` + a scrollable detail slot | `«tui»/components/dialog.rs` | Supersedes `DialogBody::Facts` **and** TablePro's post-construction button surgery (J4). |
| 18 | `HelpOverlay` (`jackin/screens/modals.rs:2284-2425`) | **move** | `HelpOverlay<'a>`, `HelpOverlayState` | `«tui»/components/help.rs` | Multi-column, round-robin, scrollable, scope label — fed by the same `Binding` metadata as `HintBar`. Replaces showcase's `?` dialog and TablePro's `\n`-joined `Dialog::confirm` help (J5). |
| 19 | Host menu bar + `run_host_menu` (`jackin/app.rs:699-813`) | **split** | item list stays domain; `MenuItem.action: ActionKey` + `.chord(Chord)` move to the library | `«tui»/components/menu.rs` + `«jackin»/app.rs` | **Key synthesis is deleted**: menu items dispatch an `ActionKey` and the same `Chord` is registered as a binding, so "menus and keys can never disagree" becomes structural rather than a comment (§20.10-6). |
| 20 | Master-detail + draggable seam (`manager.rs:114`, `accounts.rs:96`, `inspect.rs:111`, `showcase/pages/terminal.rs:104`) | **move** | `SplitPane` | `«tui»/components/split.rs` | Four copies deleted; `seam_container: Rect` app fields deleted; narrow-collapse becomes a `SplitPane` mode. |
| 21 | "Terminal too small" screen (`showcase/app.rs:802`, `tablepro/app.rs:2121`, `jackin/app.rs:2283`) | **move** | `TooSmall<'a>` | `«tui»/components/too_small.rs` | Three near-identical copies deleted; the exact copy strings are preserved so the three existing tests pass unchanged. |
| 22 | `PageCtx` / `Cx` request bus (`showcase/pages/mod.rs:74-98`, `tablepro/app.rs:51-84`, `jackin/screens/mod.rs:180-227`) | **move (partially)** | `Cx` (focus, layers, capture, repaint, intents, area) in the library; the `Request`/`Go`/`Status` payload stays per-app | `«tui»/ui/cx.rs` + `«showcase»/pages/mod.rs`, `«tablepro»/app.rs`, `«jackin»/screens/mod.rs` | The generic half (`&mut Focus`, `&FocusRing`, hit access) is deleted from all three; the product half (navigation commands, status messages) is correctly app-specific and stays. |
| 23 | `InspectChanges` diff modal (`jackin/screens/inspect.rs:61-89`) | **keep domain** | composes `Tree` + `DiffView` + `SplitPane` | `«jackin»/screens/inspect.rs` | Only the composition changes (it now uses #20 and `DiffSource`); compact/advanced and region focus stay domain. |

**Also deleted at the application level, subsumed by the library** (not separate controls but the mechanics goal §2.9 names): the three `Focus`/`FocusRing` field sets and 186+ direct manipulation sites; the three `HitRegistry` fields and the six manual re-registration blocks; the three press/hover/flash/double-click state machines; the three focus-reconciliation implementations; the three `saved_focus` fields; the three `animating()`/`tick_interval()` heuristics; `showcase/pages/terminal.rs:337-349`'s press reconstruction; `jackin`'s 9-arm outside-click, 9-arm click-dispatch and 9-arm wheel-routing matches.

---

## 19. Alternatives considered and rejected

| Alternative | Where proposed | Why rejected | Adopted instead |
|---|---|---|---|
| Immediate-mode `show(ui, area)` fusing update+draw | RES §2.2 | Re-admits draw-time mutation as legal (§11 stays a review rule, not a compile error); breaks all three harnesses because `handle` can no longer return a truthful consumed/changed answer (~60 assertions unsound); requires inverting 55 K lines of already handle/render-split app code; makes headless state-machine testing incidental | Two phases: `update(&self, cx, &mut st) -> Response<A>` and `draw(&self, ui, area, &st) -> Rect` (§3.1) |
| Retained app-owned components with a single `on(Event)` method | INT Part B | One giant method per component cannot express a typed per-phase signature (`update` needs `&mut XState`, `draw` needs `&XState`); leaves `render(&mut self)` in place so §1.2(5) survives | INT's dispatch, `Response` fields, overlay/focus/capture services — with `on(Event)` replaced by `update` and `render(&mut self)` by `draw(&self)` (§3.2) |
| Runtime-owned component tree (tuirealm / cursive shape) | RES §7 prior art | Forces `'static` + `Box<dyn>` + interior mutability; kills borrowed domain rows (Scenario D); makes two sibling `&mut` widgets impossible; requires a full-tree walk per event (§25.6) | App-owned values + a per-frame `Registry`/`FocusRing` (§3.1, §3.3) |
| A universal `Widget` trait | `README.md`, goal §5 | Components differ in whether they take state, a model, or child slots; a trait large enough to cover all of them is the "giant universal trait" goal §5 forbids | Naming and signature conventions (§13) enforced by `architecture::draw_takes_shared_self` |
| Generic theme parameters / trait-object components as the primary model | goal §9.1/§9.2 option list | Spreads generics into every application signature, or boxes every node per frame | Concrete `Theme` data + typed recipes, no generics in app-visible signatures (§11.1) |
| A `Theme` **trait** | `README.md` suggestion | Every custom-theme author would have to reimplement resolution — and resolution is exactly the part that must stay uniform for precedence to be deterministic | Concrete `Theme` data + `ThemeBuilder` + `Overlay` (§11.1) |
| A flat enum reply `Reply { Ignored, Consumed, Changed, Action(A) }` | INT A2 discussion | Cannot express consumed + action + repaint at once — precisely why `picker.rs:147` returns a tuple today | `Response<A>` with orthogonal `flow` / `invalidate` / `action` (§6.1) |
| `Box<dyn Any>` message bus | goal §9.3 option list | Untyped, unmatched, forces `'static` | Per-component action enums + `map_action` at composition boundaries (§6.1) |
| Polled result fields (`Dialog.result`) | current code | Causes all three apps to re-check `if d.result.is_some()` after every key and click | `Response<DialogAction>` + `LayerEvent::{Opened, Dismissed, Closed}` (§6.2, §9.1) |
| RES's `Response{id,state,area,action,changed}` | RES §2 | `area` is meaningless in a phase with no layout; a single `changed: bool` cannot carry the boundary-wheel rule | `Response` without `area`; geometry via `cx.area(id)` / `draw`'s return value (§6.2) |
| INT B3's `{part_kind:u16, slot:u16}` token + a per-frame key side table | INT B3 | Unnecessary complexity: packing, round-trip risk, and a per-frame table for a 24-byte `Copy` value | `PartRef { part: Part, item: Option<ItemKey> }` stored directly in each region (§7.1) |
| Interned path ids | §7.2 | Allocation plus a global on the render path; ids stop being `const`, breaking `const NAV` | FNV over kind-tagged, separator-delimited segments, `const fn` except `item` (§7.1) |
| Generational handles for identity | §7.2 | Unusable in `const` and in tests that address a control before it is first drawn | as above |
| Source-location ids (egui style) | §7.2 | Unstable under reorder, invisible in tests | `ItemKey`-derived ids + `id!` with `module_path!` |
| Keeping raw indices as the only child key | §7.2 | Fails Scenario E; already forces the index-through-a-display-string hack in TablePro | `ItemKey`; `Id::index` retained for genuinely positional cases with a debug assertion |
| Spatial / directional focus navigation | §8.1 | Unrequested; `DESIGN.md:601` specifies reading order | Registration-order ring + scopes + traps (§8.1) |
| Trap armed on render | §8.1 | A modal that fails to draw loses its trap (`dialog.rs:389`) | Trap armed when the layer is pushed (§8.1, §9.1) |
| App-owned focus restoration | §8.1 | Three divergent `saved_focus` implementations | Runtime-owned `restore: ScopeId → Id` (§8.1) |
| Wheel chaining outward at a boundary | §8.3 | Contradicts `DESIGN.md:507` and surprises the user at nested scroll edges | Consume at the boundary without repaint (§8.3) |
| Focus-follows-wheel | §8.3 | Violates "hover/scroll never steal focus" | Wheel never moves focus or the cursor (§8.3) |
| Sorted-`z` widgets for overlays | §9.1 | Solves painting only — not nesting, barriers, focus restore, Esc, or lifecycle | Runtime-owned layer stack (§9.1) |
| Modality as a render side effect (`begin_modal`) | §9.1 | Barrier ordering after children have registered; two calls in one frame clobber each other; five manual re-registration blocks exist only to work around it | `LayerSpec` pushed from `update`, composited from `draw` (§9.1) |
| Per-app overlay stacks | §9.1 | Three implementations, one of which supports nesting | One runtime stack (§9.1) |
| A general-purpose layout / constraint solver | goal §17, §10 | The three apps do not demonstrate the need; it would add a large engine for row/column/split arithmetic | A small set of composable primitives + optional `Measure` (§10) |
| `Panel::bg(t)` + `bg: Color` threading (status quo) | current code | 24 signatures, plus three components that hard-code their own plane and one public `bg_override` escape hatch | Contextual `Surface` inheritance with ladder-index `raise` (§10) |
| Colour-equality plane arithmetic (`lift`, `backdrop`) | current `theme.rs` | Breaks for any theme where two roles share a value, for light themes, and under colour downgrade where tokens collapse | Ordered surface ladder with index arithmetic (§10, §11.6) |
| Sorting state rules inside `Ui::style` (`sorted_by_key`) | RES §3 draft | One heap allocation per part per element per frame — a straight regression against today's allocation-free resolvers | Rules stored pre-sorted at recipe-construction time (R2, §20.9-1) |
| Hash-map overlay lookup on `(Family,Variant,Part,StateFlags)` | RES §3.4 | Four hashes per part per element per frame; measurable at grid scale | Linear scan over a `&'static` slice, short-circuited when the stack is empty (R3, §20.9-4) |
| `Id → String` debug table populated at **registration** | INT B1 | 300 map inserts + 300 `String`s per frame in TablePro; visibly laggy and corrupts allocation-counting tests | Populate at `Id` construction, or gate behind a `debug-ids` feature (R4, §20.9-5) |
| `RowUi::label` implemented over `ui::text::fit` | RES §3 draft | Keeps the 3-allocations-per-cell cost and makes the zero-allocation frame goal unsatisfiable | A single grapheme walk that writes cells and pads in place (R5, §20.9-6) |
| A Unicode width / grapheme memo cache | perf §8 non-recommendations | The strings change every frame at most call sites; a cache would add complexity for no gain | Stop producing intermediate `String`s at all (R5) |
| Spatial indexing for hit testing and the focus ring | perf §8 non-recommendations | Observed sizes are 30–300 regions and 4–15 ring entries; a reverse linear scan is sub-microsecond | Keep the linear scans; fix the *architecture* (`PartRef` in the region) instead (§3.3) |
| A closed `DialogBody` enum | current code, goal §14 | Named explicitly by goal §14 as unacceptable; forces "button surgery" in TablePro and a parallel dialog family in jackin | A body slot closure + convenience constructors on the same path (§9.2) |
| `fn`-pointer extension points (validator, highlighter, segmenter, `style_line`, `Apply`) | current code | Cannot capture a dialect, a catalog, a connection or a locale | `&'a dyn Fn` slots and small traits with blanket closure impls (§12.1, §15) |
| Keeping `DataTable` alongside `DataGrid` | DOM §2.12 | A third `EditState`, a second sort semantics, a second event vocabulary, one consumer | Delete `DataTable`; `Grid` with `NavUnit::{Row, Cell}` (§12.3) |
| Keeping SQL vocabulary in the generic grid | current code | Blocks Scenario H and goal §18's DataGrid boundary | Generic `GridModel`/`GridEditor` + a TablePro adapter (§12.3) |
| Keeping `ScrollPanel` | DOM §2.6 | A strict subset of `TextViewport` with a second wrap cache and an `fn`-pointer styler | Remove; migrate callers to `TextViewport` with tone-carrying spans |
| A macro DSL / CSS-like class strings / a plugin ABI / a registry CLI | goal §5 | Explicitly out of scope; would hide behaviour from engineers and coding agents | Plain Rust builders, `const` patches, and open component source (§13) |
| A compatibility facade over the old API | goal §2 | Explicitly forbidden; would preserve the nine defects under new names | Hard cut with a complete disposition map (§18) |
| Renaming the library crate away from `junie-tui` | Adjudication F | §13 (accepted) already fixes the public paths `junie_tui::*` / `junie_tui::author::*`; the rename touches `tools/capture.sh`, `README.md`, every test import and the baseline fixture path, and changes no invariant | Keep `junie-tui` / `junie_tui`; neutrality is enforced by `architecture::no_domain_vocabulary_in_the_library` and by shipping `Theme::paper()` as a peer of `Theme::junie()` (Appendix B) |

---

## 20. Known trade-offs

**20.1 A component drawn for the first time this frame is not clickable until the next frame.** Pointer intents resolve against last frame's registry (§3.3). This is exactly today's behaviour in all three apps, it keeps the resolution the user actually saw, and it avoids a speculative layout pass. Documented on `Cx::area` and asserted by `focus_reconcile_follows_the_rule`. The cost is one frame of latency on a control that appears under an already-moving pointer.

**20.2 Two phases means two constructions of the props struct per frame.** `update` and `draw` each build `Button::new(SAVE, "Save")`. Props are stack-allocated borrowed views, so the cost is register moves, not allocation (`frame_showcase_lists_120x40 < 20 allocs/frame`). The benefit is that `draw` can take `&self` and `&XState`, which is what makes G2 a compile error. Callers who dislike the repetition can factor a `fn button(&self) -> Button<'_>` helper — the migrated jackin `Screen` in §3.4 does exactly that.

**20.3 Collection generics (`List<'a, T, K, R>`) are three type parameters.** They are always inferred at the call site and never appear in an application signature (§13). The alternative — boxing the key and row closures — costs two allocations per collection per frame and a `'static` bound. `architecture::no_static_bound_in_component_surface` guards the boundary.

**20.4 `Id` is a 64-bit hash with no reverse mapping in release builds.** Collision safety is *detection* (`Diagnostic::DuplicateId`), not prevention. With kind-tagged, separator-delimited segments the accidental-collision class of §1.2(1) is eliminated; a genuine FNV collision remains theoretically possible and is reported, never panicked (goal §10). Debug builds carry a zero-cost-in-release `DebugLabel`.

**20.5 `Invalidate::Layout` ships but behaves as `Paint`.** It is reserved for future layout caching; only its ordering is asserted (§8.5). Shipping it now avoids a breaking change later; the cost is one variant that does nothing distinguishable today.

**20.6 `&'a dyn Fn` slots are the only `dyn` in a component's public surface.** They are opt-in and allocation-free, but they are dynamic dispatch on a per-part path. Measured under `style_resolve_10k_parts`; the alternative (a generic slot parameter per part) would add one type parameter per replaceable part to every component signature, which fails §13's "no gratuitous generic parameters".

**20.7 Controlled values require the caller to hold a `String` per field.** For a 15-field form that is 15 owned strings the caller must place somewhere. This is deliberate: it deletes the "rebuild the widget to change its value" idiom (five sites) and makes external synchronisation trivial. Uncontrolled mode (`XState` owns the draft) remains for throwaway fields and is documented per component (S4).

**20.8 The conformance suite is a hard cost per new component.** Registering a `Conformance` case is ~40 lines. It is mandatory (`architecture::conformance_covers_every_public_component`), which is the point: 20 contracts become free for every component, forever, and the untested-module gap of **[F]** API §8.1 (21 of 31 modules with no tests) cannot recur.

### 20.9 Performance obligations (binding)

These are **amendments to §3–§15**, not advice. Each folds a `docs/audit/performance-audit.md` finding into the accepted architecture and carries the acceptance test that proves it. A builder who implements §3–§15 without these has not implemented the architecture.

| # | Amendment | Amends | Acceptance test (§16.6) |
|---|---|---|---|
| 1 | **State rules are stored in specificity order at recipe-build time.** `PartEdit::when` inserts into `PartRecipe.states` sorted by `when.count_ones()` ascending, ties by declaration order. §11.3 step 3's "ordered by specificity" is therefore a *storage* invariant, not a resolution-time sort. `Ui::style` is `for rule in &part.states { if s.contains(rule.when) { acc = acc.merge(rule.patch) } }` and **allocates nothing**. (R2) | §11.3 | `style_resolve_10k_parts` — **exactly 0 allocations**, ns ≤ 2× the pre-refactor `Theme::row`+`Theme::gutter` baseline |
| 2 | **The §11.1 A3 memo cache is allocation-free and statically sized.** A `[Option<(u64, Resolved)>; 256]` direct-mapped array embedded in `Ui`, keyed by a 64-bit mix of `(Family, Variant, Part, StateFlags, Surface, overlay_stack_hash)`, cleared by a generation stamp rather than by zeroing. No `HashMap`, no `Vec`, no per-frame allocation, no growth. A miss recomputes; there is no eviction policy to get wrong. | §11.1 A3 | `style_resolve_10k_parts`, `render_twice_allocates_the_same` |
| 3 | **`ItemKey` reconcile uses a generation stamp cache.** Every `XState` with a cursor stores `(cursor_key, cursor_index, stamp)` where `stamp = (len, key(first), key(last))`. `reconcile` returns `Unchanged` immediately when the stamp matches; on a mismatch it first probes the cached index (`key(&items[i]) == cursor_key`) and only then scans. `XState::invalidate()` is public for callers who mutate in place. 100 000 rows never re-hash per frame. (R1) | §12.2 | `list_100k_rows_render` — **< 500 allocs/frame**, ns ≤ 1.5× `list_1k_rows_render`; `event_dispatch_is_not_o_n` — 0 allocs, ns within 3× of the 100-row click |
| 4 | **Overlay lookup is a linear scan over a `&'static` slice, short-circuited when empty.** `Overlay::new(&'static [(Family, Variant, Part, StateFlags, StylePatch)])`; the resolution loop returns before touching the stack when `stack.is_empty()`, which is the overwhelmingly common case. No hashing on the style path. (R3) | §11.3 step 5 | `style_resolve_10k_parts_with_two_overlays` — 0 allocations, ns ≤ 2× the empty-stack case |
| 5 | **`Id` debug names are populated at construction, never at registration.** `id!` expands to a `const` path, so `Registry::names` is filled when an `Id` is built (a `once`-initialised table keyed by the literal), not by `register_*`. If a registration-time table ever proves unavoidable it is gated behind an explicit `debug-ids` cargo feature, never behind `debug_assertions`. §7.1's "populated at registration" is amended accordingly. (R4) | §7.1 | `debug_and_release_alloc_counts_match` — `frame_tablepro_grid_500x12_120x40` reports identical allocation counts in debug and release |
| 6 | **`RowUi`/`CellUi` paint via a single grapheme walk with no intermediate `String`.** `RowUi::label`, `meta`, `trailing` and `CellUi` write cells directly and pad in place. `ui::text::{fit, fit_right}` are **deleted from every render path** and survive only for non-render callers. §12.2's `RowUi` contract is amended to forbid intermediate allocation. (R5) | §12.2 | `fit_10k_grapheme_line_to_80` — the `RowUi` equivalent records **0** allocations; `frame_showcase_lists_120x40` drops from ≈160 to **< 20** allocs/frame; `grid_500x12_render` **< 100** |
| 7 | **`TextViewport` cells become `(range, width)` with windowed incremental layout.** `Cell { range: Range<u32>, w: u8, style_ix: u16 }` referencing the source `Span` text instead of an owned grapheme `String`; layout is append-only on `push` and lays out only `visible_range ± 1 page`. §14.1's "`TextViewport` — Keep" is amended: the *behaviour* is kept, the *storage* is rewritten. (perf §6.3-1) | §14.1, §12.4 | `viewport_100k_lines_push` — allocations **independent of `lines.len()`**; `viewport_layout_10k_grapheme_line` — **0** allocations; `viewport_100k_lines_render` — allocs/frame independent of buffer size |
| 8 | **`Tree` flatten is incremental and keyed.** The flat index is rebuilt only for the affected subtree on expand/collapse, `expanded` is `HashSet<ItemKey>` (no `Vec<usize>` hashing), rows borrow `label`/`meta` from the source nodes, and filtering does not lowercase per node per level. §12.4's "`Tree` keeps hierarchy, lazy children" is amended with this storage requirement. (perf §6.3, §5.2) | §12.4 | `tree_100k_nodes_flatten` — allocs **< 10 × viewport** per toggle; `tree_100k_nodes_render` — allocs/frame independent of node count; `key_tree_toggle_10k` |
| 9 | **`CodeEditor` uses an edit-counter cache and a sorted-span cursor.** The highlight cache is keyed on a monotonically incremented edit counter, never on re-hashing the document per frame; spans, diagnostics and find matches are stored sorted and consumed by a cursor advanced alongside the grapheme walk — O(graphemes + spans), not O(graphemes × spans). The seven per-frame clones are structurally impossible because `draw` takes `&self` and reads the state it needs by reference. §14.1's `CodeEditor` row is amended accordingly. (perf §6.3-2) | §14.1 | `frame_tablepro_query_editor_2k_lines` (§16.6 addition) — **< 40 allocs/frame**, ns scaling with viewport not document length |
| 10 | **`Capsule` never clones a viewport per frame.** With caller-owned `ViewportState`, jackin renders directly from the daemon's pane; `TextViewport::set_area`/`prime` and the `inert` clone dance are deleted. The per-frame `pane.term.clone()` — the single worst path in the repository — has no replacement because it has no reason to exist. (perf §8-1) | §12.4, §18.3 #23 | `frame_jackin_capsule_4panes_120x40` — **< 200 allocs/frame**; `capsule_pane_clone_4x2000` **is deleted** |
| 11 | **TablePro's grid load is one owned conversion.** The three-copy chain (`db::rows` → projection clone → `to_cell` clone) collapses to a single owned `ResultSet` that the `GridModel` borrows; `sample_widths` calls a non-allocating `CellValue::display_width()` instead of materialising a `String` per sampled cell. (perf §8-4, §6.3-4) | §12.3 | `grid_500x12_load` — **< 8 000** allocations (from ≈36 000) |
| 12 | **`Intents::take` does not scan the queue.** The runtime builds a small per-frame index (or sorts once by `Id` and binary-searches); frame cost is O(intents), never O(components × intents). (R6) | §3.3 step 7 | `intents_drain_scales_with_intents_not_components` (§16.6 addition) |
| 13 | **`KeySet` has an inverted representation.** `KeySet::AllExcept(set)` so "select all" over 100 000 rows does not materialise 100 000 `ItemKey`s; `ListAction::ToggledAll` reports the intent and the caller may keep its own bitmap. (R7) | §6.1, §12.2 | `list_100k_select_all` (§16.6 addition) — **< 100** allocations |
| 14 | **The backdrop dim resolves as a `StylePatch` and walks only the covered rect.** It runs through the same `Resolved` path as everything else, so a monochrome or no-colour theme gets it for free, and it never walks cells the layer does not cover. §11.6's "`backdrop` recipe keyed on `Role`" is amended with the rect restriction. (perf §6.3-3) | §11.6, §9.1 | `style_backdrop_full_screen_120x40` — 0 allocations, ns ≤ 1× the pre-refactor baseline |
| 15 | **Jackin's manager rebuilds rows on world change only.** `build_rows`/`build_detail`/`rebuild_actions` are gated by a world generation counter and never run from `draw` (structurally: `draw` is `&self`) nor from `on_key` before the key is examined. (perf §8-7) | §18.3, §3.4 | `key_jackin_manager_move` — **0 allocs/key**; `frame_jackin_manager_100rows_120x40` — **< 60 allocs/frame** |
| 16 | **`inert_below` suppresses background registration.** A modal layer with `inert_below: true` stops the page beneath it from registering ring entries, hit regions and cursor writes, so an open dialog does not pay the full background registration cost every frame. §9.1's `inert_below` is amended from "no interaction" to "no *registration*". | §9.1 | `frame_showcase_dialog_open` — `hits` **< 25 %** of `frame_showcase_lists_120x40` |

**Sequencing obligation.** The benchmark harness and the checked-in baseline land **first**, on a `perf/baseline` commit against the pre-refactor tree (Appendix A, WP‑0). Without that commit, "before and after" in goal §25.6 and §30 item 13 is not literal and none of the thresholds above can be asserted.

### 20.10 Intentional visual changes

Every item below changes rendered output relative to the reviewed baseline. Each is deliberate, is justified against `DESIGN.md` or a demonstrated defect (authority order, goal §3), and each names how it is reviewed. Nothing on this list may be regenerated into a baseline without an entry in `docs/visual-changes.md` (enforced by `xtask bless-guard`, §16.3).

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** (§11.4). At `ColorLevel::Mono` every state gains a symbol or modifier: focus gutter bar + bold label, marker glyphs for selected/checked, explicit reverse for pressed (never the terminal `REVERSE` attribute), faint + no marker for disabled, trailing error glyph + underline, dirty glyph for warning, underline + hardware cursor for editing, spinner for busy, active rule + bold for tabs. | **[F]** RES §1.2: mono currently collapses accent (mean 126) and error (mean 122) onto the same grey, so state is unreadable. goal §15 requires state meaning to survive without colour. | `conformance::<component>::mono_states_are_distinguishable` for every component; `render::components::*_mono` digests; capture matrix `tools/capture.sh` with `NO_COLOR=1` at 120×40 for showcase, tablepro and jackin, reviewed side by side against the truecolor capture by a fresh `opus-analyst` visual reviewer |
| 2 | **Layer compositing order** (§5 R7, §3.3 step 12). Layers paint into pooled buffers and are composited bottom-to-top after `app.draw` returns, so z-order is the *layer* order, not the call order. A popup no longer has to be "drawn last" by the caller (**[F]** `DESIGN.md:749`). | Removes the three different "draw the overlay last" conventions, the `ui/popup.rs` shared-id collision, and the six manual hit re-registration blocks. Fixes the case where two popups in one frame silently clobber each other's barrier. | `render::overlay::layer_composites_bottom_to_top_regardless_of_call_order`; `render::overlay::nested_picker_over_dialog` digest; jackin `hint_bar_stays_on_the_last_row_across_layers` (retained, must stay green); captures of `f_*` (filter editor over grid) and `j_*` (picker over dialog) before/after |
| 3 | **`RadioGroup` separates cursor from value.** Arrow keys move a cursor; Space/Enter commits the value. Today arrows change the selection while moving (`choice.rs:121-130`). | The only collection in the library that fuses cursor and selection (DOM §6.1-2); inconsistent with `List`, `Tabs`, `Picker`, `Grid`, `Tree`. | `conformance::radio_group::keyboard_and_mouse_activation_are_equivalent`; unit `choice::radio_group_separates_cursor_from_value`; showcase `forms` page digest; the two affected app tests (`showcase::form_validation_blocks_submit_and_focuses_first_error`, jackin `ChoiceDialog` flows) are re-read to confirm the new keystroke sequence still expresses the same product intent |
| 4 | **`Picker` secondary action gains a mouse equivalent.** `Delete` (secondary, e.g. close a tab from the tab list) becomes a secondary-click on the row and a visible trailing affordance, not a keyboard-only path. | goal §13: "keyboard and mouse activation of the same control should produce the same semantic component action"; `conformance::picker::keyboard_and_mouse_activation_are_equivalent` would otherwise fail by construction. | The conformance test itself; `render::components::picker::default` digest (a trailing affordance column appears when any item is secondary-able); tablepro `tab_strip_overflow_and_tab_list` capture |
| 5 | **`Dialog`'s `y`/`n` quick answers become an opt-in binding set.** Today they are hard-coded and only for `DialogBody::Text` (`dialog.rs:297-311`). | goal §13: application-domain chords must not live in generic components. TablePro and jackin opt in through their `KeyMap`; showcase's `modal_traps_focus_and_restores_it` and `settings_screen_remove_member_flow` opt in so `y` still answers. | `conformance::dialog::bindings_match_handled_keys`; the three retained app tests that press `y`; the derived `HintBar` row now shows `y Yes  n No` only where the bindings are opted in — visible in the footer of every dialog capture |
| 6 | **F10 / menu-bar drift fixes.** `MenuItem` carries an `ActionKey` *and* the `Chord` that is both rendered as the hint and registered as the binding. jackin's `run_host_menu` key-synthesis dispatcher (`app.rs:754-813`) is deleted, as is label-string dispatch in `capsule.rs:368-471`. | **[F]** DOM §2.2: `shortcut: Option<&'static str>` is a display string with no relation to key handling, so the menu and the keymap can silently disagree; the workaround was to re-synthesise key presses. Making them one declaration removes the drift class. | `conformance::menu_bar::bindings_match_handled_keys`; `conformance::conflicting_visible_bindings_are_reported`; jackin `menu_bar_opens_switches_and_runs_an_action`, `tab_context_menu_renames_and_closes_by_mouse_and_keyboard`, `inspect_changes_opens_from_the_view_menu_in_both_modes` (all retained); `j_menu_*` captures compared for shortcut-column alignment, which changes where a chord was previously mis-labelled |
| 7 | **Container / geometry defect fixes** — a group, reviewed together. (a) `dialog.rs:389`'s modal that returns before registering anything now still traps focus, because the trap belongs to the layer. (b) `panel.rs:117-122`'s framed inner rect no longer escapes the panel for `area.width ≤ 4`. (c) The 12 `usize` underflow sites in `input`/`textarea`/`grid`/`table` are `saturating_sub`. (d) The 4 ragged-row index panics in `grid`/`table` are bounds-checked. (e) `Split::vertical`/`horizontal`'s opposite collapse becomes "first pane wins on both axes". (f) `fuzzy` returns grapheme indices into the original label, so match highlighting no longer lands on the wrong bytes for labels whose lowercase form differs in length. (g) `hit_scroll` no longer returns non-scrollable regions. (h) The ~20 stale-geometry early-return sites are gone because the registry is rebuilt per frame. (i) All popups no longer share `WidgetId::of("popup.surface")`. (j) `Interaction.focus_hidden` (dead) is removed. | **[F]** §1.3 latent defects; goal §17 forbids panics, underflow, out-of-area writes and stale hit regions. | `conformance::<component>::survives_tiny_rects_0x0_to_3x3` and `draw_stays_inside_its_area` for every component; unit `layout::split_first_pane_wins_on_both_axes_when_minima_do_not_fit`, `text::fuzzy_returns_grapheme_indices_into_the_original_label`, `hit::hit_scroll_skips_regions_that_do_not_handle_the_axis`; new `render::components::*_40x10` narrow digests; captures at 60×15 and 72×20 for all three apps. Items (b), (e) and (f) change pixels in the current baseline and are called out individually in `docs/visual-changes.md`. |
| 8 | **The backdrop excludes the footer row uniformly** and is one implementation. Today `Dialog` and `Picker` each dim with byte-identical loops and both leave the last row live inconsistently with TablePro's hand-written filter-editor dim. | `DESIGN.md:537`; three implementations cannot stay consistent. | `layer::backdrop_excludes_the_footer_row`; `render::overlay::modal_over_page` digest; tablepro `f_filter_*` captures (its dim currently differs from `Dialog`'s) |
| 9 | **`StatusBar` and `segments` merge**, so TablePro's identity strip and its grid status line adopt the shared priority-drop order (centre → right → left, strongest left item never leaves) instead of two bespoke loops. | **[F]** DOM §2.7: the same concept at two fidelities, with a third hand-rolled copy. | `render::components::status_bar::*` at 80/100/120/160 columns; tablepro `every_screen_renders_at_representative_sizes` (retained); `t_strip_*` captures at 80×24 where the drop order visibly differs |
| 10 | **Hints are derived from component bindings.** The `HintBar` composes top layer ▸ temporary mode ▸ focused component's visible bindings by priority ▸ screen extras ▸ global fallback. Hint text changes wherever a hand-written table had drifted from the real bindings. | **[F]** DOM §2.8: ~700 lines of hand-written hint tables across two apps, kept in sync by hand; `capsule.rs:2478-2492` already documents the drift risk. goal §13 asks for exactly this. | `conformance::<component>::bindings_match_handled_keys`; the footer row of every capture in the matrix; jackin `hard_cases_refresh_keeps_last_good_and_help_opens_everywhere` (per-route help sections, retained); a diff of the old hand-written tables against the derived output is attached to `docs/visual-changes.md` so each drifted entry is classified as a fix or a regression |
| 11 | **Surface inheritance replaces colour-equality `lift`.** Hover elevation is ladder-index arithmetic. Under `Theme::junie()` the resolved planes are unchanged wherever the ladder is injective; where two Junie tokens happen to share a value the resolved plane may now differ from the equality-dispatch result. | **[F]** API §6.2: `lift` and `backdrop` branch on colour equality and silently land on `popover`/`surface_overlay` for any unexpected input; this is the single biggest obstacle to Scenario B. | `theme::raise_is_ladder_index_arithmetic_not_colour_equality`; the full `render::components::*` digest under `junie` (any cell that changes is enumerated in `docs/visual-changes.md` with the token pair that collided); showcase `hover_and_focus_render_differently` (retained, asserts `bg == surface_overlay` on hover) |
| 12 | **The showcase visual baseline covers the sidebar and gains three axes.** The `sidebar_area()` exclusion is removed (the sidebar becomes a `NavList`, so it is no longer hand-drawn); the matrix becomes pages × {120×40, 80×24} × {junie, paper} × {truecolor, mono}. | **[F]** APP §6: the exclusion existed only because the shell sidebar was a hand-written copy of `ListBox`. Goal §25.3 requires the custom theme and no-colour modes to be under snapshot. | The baseline file diff itself, reviewed line by line at the point of regeneration; `showcase_visual_baseline` (retained name) |
| 13 | **`Tabs`' strip window follows the logical first tab.** After an insert or reorder the visible window keeps showing the same tab, instead of the same index. | Scenario E; today both apps rescue `first` by hand across a full widget rebuild. | `conformance::tabs::item_identity_survives_reorder`; tablepro `tab_strip_overflow_and_tab_list` (retained); `t_tabs_overflow` captures before/after an insert at position 0 |
| 14 | **New cell-exact baselines for TablePro and jackin.** Neither has one today (**[F]** APP §6, §9 risk 5); their regressions would be caught only by text-substring assertions. | goal §25.3 and §26 require inspecting actual rendered output; §20.10 items 8–11 all touch TablePro and jackin surfaces that nothing currently pins. | The first generation of `apps/tablepro/tests/baselines/tablepro.txt` and `apps/jackin-preview/tests/baselines/jackin.txt` is produced **on the pre-refactor tree** at the `perf/baseline` commit (Appendix A, WP‑0) so it is a genuine before-image, then re-generated once at the end of Slice 8 with every difference classified against this table |

**Not on this list, and therefore regressions if they appear:** any change to Junie token values; any change to spacing, glyph or border-set output under `Theme::junie()` at truecolor; any change to padding or ellipsis placement caused by replacing `fit`/`truncate` with the `RowUi` grapheme painter (the painter must be byte-identical to `fit` for every input — asserted by `render::components::*` digests and by a dedicated differential test `text::row_ui_matches_fit_for_every_fixture`); any change to the exact minimum-size copy strings; any change to the eight jackin scenario contracts, the rain timing constants, or the `format_universe_duration` wording.

---

## Appendix A — Slice plan

Maps goal §27 slices 3–8 onto work packages with **disjoint file ownership**, so `fable-builder` agents can run in parallel without integration conflict. A package's owner is the only agent that writes its files during that slice. Files not listed are owned by nobody and must not be touched.

**Amendment to goal §27 (recorded, with justification).** Goal §27 Slice 4 says "migrate coherent families, continuously updating showcase pages and tests". Continuous showcase updates would make every Slice-4 owner write into `apps/showcase/`, destroying disjointness. Instead: **Slice 4 owners do not touch any application.** Each family package proves itself with unit tests, a `Conformance` registration (which runs the full 20-case matrix), a `render::components::*` digest, and one `crates/tui/examples/` file. Showcase migration is entirely Slice 5. The review cadence goal §27 asks for is preserved: a fresh read-only `opus-analyst` reviews API consistency after **each** family package lands, before the next depends on it.

### WP‑0 — Performance and visual baseline (blocking prerequisite, before Slice 3)

* **Owner:** one builder. **Files:** `crates/tui-testing/src/perf.rs`, `tests/perf.rs`, `tests/perf_baseline.txt`, `tests/baselines/` (pre-refactor tablepro + jackin digests), `.github/workflows/perf.yml`.
* Written against the **current** single-package tree, on a `perf/baseline` commit. Records every §16.6 "before" number and the two new app digests (§20.10-14).
* **Gate:** `cargo test --test perf --release -- --test-threads=1` green; `perf_baseline.txt` and the two baseline files committed; `PERF` lines archived as a build artefact.
* **Dependency:** everything. No refactor commit lands before this one.

### Slice 3 — Foundations (one owner, serial)

* **Owner:** one builder. **Files:** the whole of `crates/tui/src/` except `components/`, plus the workspace skeleton and the test crate.
  `Cargo.toml` (workspace), `crates/tui/Cargo.toml`, `crates/tui/src/{lib.rs, author.rs, id.rs, event.rs, intent.rs, response.rs, keymap.rs, focus.rs, hit.rs, capture.rs, scroll.rs, cursor.rs, layer.rs, runtime.rs, diagnostics.rs, layout.rs, measure.rs}`, `crates/tui/src/ui/**`, `crates/tui/src/text/**`, `crates/tui/src/theme/**`, `crates/tui/src/collection/**`, `crates/tui-testing/src/**`, `crates/tui/tests/{conformance.rs, architecture.rs, render.rs}` (skeletons).
* **Order inside the slice:** identity → events/response/intents → registry/focus/capture/cursor → layers → surface/`Ui`/`Cx` → theme tokens/patch/recipe/resolve/downgrade → layout/measure → text/editor → collection vocabulary (`ItemKey`, `reconcile`, `RowUi`, `RowDecor`, `EmptyState`, `scroll_region`) → `Runtime`/`App`/`run` → `author` module → `Harness` + `Conformance` driver + digest driver.
* Applications do not compile during this slice. They are excluded from the workspace default members until Slice 5–7 and re-added one at a time. `crates/tui/examples/12_author_component.rs` is written here as the first consumer, proving the `author` surface is complete before any component depends on it.
* **Gate:**
  ```bash
  cargo fmt --all --check
  cargo clippy -p junie-tui -p junie-tui-testing --all-targets --all-features -- -D warnings
  cargo test -p junie-tui -p junie-tui-testing --all-targets --all-features
  cargo test -p junie-tui --doc
  RUSTDOCFLAGS="-D warnings" cargo doc -p junie-tui --all-features --no-deps
  cargo build -p junie-tui --examples
  cargo test -p junie-tui --test architecture
  cargo test -p junie-tui --test perf --release -- --test-threads=1
  ```
  plus a fresh read-only `opus-analyst` API review of the foundation surface (goal §27 Slice 2's review, applied to the real implementation) before Slice 4 begins.

### Slice 4 — Component families (parallel; 9 owners)

Every package owns files under `crates/tui/src/components/` only, plus its own example and its `Conformance` registration line. All depend on Slice 3; **none depends on another 4x package** except where stated.

| WP | Owner scope | Files owned | Depends on |
|---|---|---|---|
| 4A | Buttons, choices, chips, brand, hints, empty chrome | `components/{button.rs, choice.rs, chip.rs, brand.rs, keyhint.rs, too_small.rs}`, `examples/01_button.rs`, `examples/05_instance_patch.rs` | Slice 3 |
| 4B | Fields, inputs, textarea, select, secrets, validation | `components/{field.rs, input.rs, textarea.rs, select.rs, secret.rs, validate.rs}`, `examples/06_validated_field.rs` | Slice 3 |
| 4C | Lists, trees, props, steps, nav | `components/{list.rs, tree.rs, props.rs, steps.rs, nav_list.rs}`, `examples/07_borrowed_rows.rs` | Slice 3 |
| 4D | Tabs | `components/tabs.rs`, `examples/08_dynamic_tabs.rs` | Slice 3 |
| 4E | Containers and scrolling | `components/{panel.rs, split.rs, scroll_region.rs, viewport.rs}` | Slice 3 |
| 4F | Overlays: dialog, menu, picker, completion, form, wizard, chain, help | `components/{dialog.rs, menu.rs, picker.rs, filter_list.rs, completion.rs, form.rs, wizard.rs, picker_chain.rs, help.rs}`, `examples/{09_composed_dialog.rs, 10_nested_overlay.rs, 11_small_app.rs}` | Slice 3; **4B** (`Form` composes `Field`), **4C** (`Picker` composes `FilterList` rows) |
| 4G | Status, hints, progress, meters | `components/{status.rs, hintbar.rs, progress.rs, meter.rs}` | Slice 3 |
| 4H | Code editor and diff | `components/{code.rs, diff.rs}` | Slice 3; **4E** (`DiffView` composes `TextViewport`) |
| 4I | Generic grid | `components/grid.rs`, `crates/tui/tests/fixtures/grid_model.rs` (a test-only model — the TablePro adapter is Slice 6) | Slice 3; **4C** (shared collection vocabulary) |

Shared, contended files are handled by convention rather than by ownership: `components/mod.rs`, `crates/tui/src/lib.rs`'s re-export list, `crates/tui/tests/conformance.rs`'s `conformance_suite!` list, and `examples/02_custom_theme.rs`/`03_partial_theme.rs`/`04_family_recipe.rs` (which touch every family's recipe defaults). Each is **append-only in a fixed, alphabetically sorted region**, so concurrent additions merge cleanly; the coordinator resolves the ordering once per slice.

* **Wave order** (to honour the dependencies above): wave 1 = 4A, 4B, 4C, 4E, 4G in parallel; wave 2 = 4D, 4F, 4H, 4I in parallel.
* **Gate (per package, then per wave):**
  ```bash
  cargo fmt --all --check
  cargo clippy -p junie-tui --all-targets --all-features -- -D warnings
  cargo test -p junie-tui --lib
  cargo test -p junie-tui --test conformance
  cargo test -p junie-tui --test render
  cargo test -p junie-tui --test architecture
  cargo test -p junie-tui --doc
  cargo build -p junie-tui --examples
  cargo test -p junie-tui --test perf --release -- --test-threads=1
  ```
  Every component in the package must appear in `conformance_suite!` and pass all 20 applicable cases. After each package, a fresh read-only `opus-analyst` reviews API consistency against §13; the coordinator applies verified corrections before the next wave.

### Slice 5 — Showcase (one owner)

* **Files:** `apps/showcase/**` in full (`Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/pages/*.rs` — all 22 — `src/data.rs`, `tests/app_tests.rs`, `tests/visual.rs`, `tests/baselines/showcase.txt`, `tests/perf.rs`).
* Deletes the shell sidebar, footer hint row, static-field renderer, button matrix, inspector panel and too-small screen in favour of library components (§18.3 #2–#7, #21). Adds the pages goal §22.1 requires: the state matrix per component, `Theme::paper()` coverage, scoped and per-instance override pages, the author-component page (example 12 rendered as a page), and deterministic navigation to every state for captures.
* Re-adds `showcase` to workspace default members. All 26 existing tests must pass with the §16.4 `Harness`.
* **Gate:** the full §26 command set scoped to `-p junie-tui -p showcase`, plus `cargo run -p showcase` driven through `tools/capture.sh` at 80×24, 100×30, 120×40, 160×50 × {truecolor, 256, 16, mono} × {junie, paper}, with every capture inspected and every baseline difference classified against §20.10.

### Slice 6 — TablePro (one owner)

* **Files:** `apps/tablepro/**` in full, including the new `src/grid_model.rs` (the `GridModel`/`GridEditor` adapter carrying `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, validators, `cmp_cells`, insert/duplicate/delete/discard/undo, `primary`/`nullable`/`references`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar) and `src/filter_editor.rs`.
* DOM §1.6's 22-capability mapping is the migration checklist; each capability is ticked off against a retained or new test before the slice closes.
* **Gate:** the §26 set scoped to `-p junie-tui -p tablepro`; all 21 existing tests green; `grid_500x12_load` and `frame_tablepro_grid_500x12_120x40` meet their §16.6 thresholds; `apps/tablepro/tests/baselines/tablepro.txt` regenerated once with every difference classified; captures of connection, editor, grid, tabs, dialog, menu, picker and results surfaces reviewed.

### Slice 7 — Jackin (one owner)

* **Files:** `apps/jackin-preview/**` in full, including the decomposition of `screens/modals.rs` (≈2 400 lines) into `screens/{file_browser.rs, op_flow.rs}` plus library `Form`/`Dialog`/`HelpOverlay` usage, and `rain.rs` rewritten onto `Role` + `Ui::dim_layer`.
* **Gate:** the §26 set scoped to `-p junie-tui -p jackin-preview`; all 22 existing tests plus the `rain`/`arbiter`/`clock`/`scenario` unit tests green; the eight scenarios reachable; the determinism assertion (two `--frame 282` runs byte-identical) green; the secret-masking assertions green; `frame_jackin_capsule_4panes_120x40 < 200 allocs` and `capsule_pane_clone_4x2000` deleted; `apps/jackin-preview/tests/baselines/jackin.txt` regenerated with differences classified; host, settings, account/usage, launch, Capsule, menu, modal, tab, status-bar and responsive surfaces captured and reviewed.

### Slice 8 — Cleanup and independent verification (one owner, then two reviewers)

* **Files:** anything, but only for deletion, visibility tightening, documentation and reviewed baseline regeneration. No new behaviour.
* Work: delete every remaining legacy path and dead module; tighten `pub` → `pub(crate)` everywhere `architecture::applications_depend_only_on_the_library_facade` allows; complete `README.md`, `DESIGN.md`, the theme-customisation guide, the component-override guide, the component-author guide and the old→new API map (goal §24); regenerate only reviewed baselines; run the full gate set; then a fresh read-only `opus-analyst` **architecture** review and a separate fresh read-only `opus-analyst` **visual** review, with the coordinator correcting every verified issue.
* **Gate (the goal §26 set, unscoped):**
  ```bash
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace --all-targets --all-features
  cargo test --workspace --doc
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
  cargo build --workspace --all-targets --all-features
  cargo test --workspace --test perf --release -- --test-threads=1
  PERF_STRICT=1 cargo test --workspace --test perf --release -- --test-threads=1
  cargo run -p showcase & cargo run -p tablepro & cargo run -p jackin-preview
  tools/capture.sh   # the full matrix, reviewed
  ```

**Dependency summary.** WP‑0 → Slice 3 → {4A,4B,4C,4E,4G} → {4D,4F,4H,4I} → Slice 5 → {Slice 6, Slice 7 — parallel, disjoint app trees} → Slice 8. Slices 6 and 7 may run concurrently because their file trees are disjoint and both depend only on the frozen library surface; if either needs a library change, the slice pauses, a fresh `opus-analyst` adjudicates, the decision is recorded in this document and `REFACTORING_STATE.md`, and the change lands as a small serial amendment before both resume.

---

## Appendix B — Package layout and crate naming (Adjudication F)

### B.1 Decision — the repository becomes a Cargo workspace; the library keeps the name `junie-tui`

**Workspace, not one package.** goal §9.5 asks for a *mechanically enforceable* boundary proving applications consume only supported public APIs. A single package cannot provide one: `pub(crate)` is visible to `src/bin/*` because the binaries are in the same crate, which is exactly how the three apps reach `HitRegistry`, `Focus` and `FocusRing` today. A workspace makes the boundary a property of the compiler rather than of a grep: an application literally cannot name a `pub(crate)` item. Every text check in §16.5 is therefore a *report*, and the enforcement is structural.

**Crate name: keep `junie-tui` (package) / `junie_tui` (lib).** Considered and rejected: renaming to a theme-neutral name such as `tui-components`.

*Why keep it.* (a) §13, which is accepted, already fixes the public paths `junie_tui::*` and `junie_tui::author::*` as the two documented API layers; changing them is a change to an accepted decision and would require a fresh adjudication for a naming preference, not for an invariant. (b) "Junie" names the *design language*, not an application domain. G8 and `architecture::no_domain_vocabulary_in_the_library` forbid TablePro and Jackin vocabulary; a theme name is neither. (c) The neutrality the rename would buy is bought instead by structure: `Theme::junie()` and `Theme::paper()` are peers under `junie_tui::theme::builtin`, no component references Junie, and `architecture::palette_literals_are_confined_to_theme_builtins` proves it. (d) The rename cost is entirely non-architectural churn — `tools/capture.sh`, `README.md`, `Cargo.toml`, every test import, the baseline fixture path — landing in the middle of a refactor that already changes every one of those files for real reasons, which makes classification of the resulting diffs harder, not easier.

*Rename implications, recorded so the decision is reversible in one commit.* If the crate is later renamed to `<new>` / `<new_snake>`:

1. `crates/tui/Cargo.toml`: `package.name`, `lib.name`.
2. `apps/*/Cargo.toml`: the `junie-tui` and `junie-tui-testing` dependency lines.
3. Every `use junie_tui::…` in `apps/**`, `crates/tui/examples/**`, `crates/tui/tests/**` and every doctest — mechanical, `cargo fix` cannot do it, a scripted rename can.
4. `tools/capture.sh`: `S=junie_cap` (tmux session name) and `BIN=${BIN:-target/debug/junie-tui}` — the latter is already stale (the package has no binary of that name) and becomes `BIN=${BIN:-target/debug/showcase}` in Slice 5 regardless of any rename.
5. `README.md` and `DESIGN.md` prose, install snippets, and the `UPDATE_BASELINE` documentation.
6. `tests/showcase_baseline.txt` moves to `apps/showcase/tests/baselines/showcase.txt` in Slice 5 regardless; its `CARGO_MANIFEST_DIR` anchor changes crates either way.
7. `shots/` filenames are unaffected.
8. `architecture::binary_names_are_preserved` and `architecture::msrv_and_edition_are_unchanged` are unaffected — the *binary* names never change.

**Binary names are preserved exactly** (goal §21): `showcase`, `tablepro`, `jackin-preview`. Each becomes its own package whose single `[[bin]]` carries the required name, so `cargo run -p showcase` and `target/debug/showcase` both work and `cargo build --workspace` produces all three. The current `default-run = "showcase"` has no workspace equivalent and is dropped; `cargo run -p showcase` replaces it and is documented in `README.md`.

**Edition and MSRV are unchanged**: edition 2024, `rust-version = "1.88"`, set once in `[workspace.package]` and inherited by every member. Dependencies are unchanged except for two small, justified additions used by the accepted architecture (`bitflags` for `StateFlags`/`Caps`, `smallvec` for `PartRecipe.states` and `Recipe.variants`); no framework-sized dependency is added, and no unrelated version churn accompanies the refactor.

### B.2 Exact layout

```
Cargo.toml                                  # [workspace] members + [workspace.package] + [workspace.dependencies]
rust-toolchain.toml                         # pinned for the PERF_STRICT job
README.md  DESIGN.md  COMPONENT_ARCHITECTURE.md  REFACTORING_GOAL.md  REFACTORING_STATE.md
docs/audit/*.md                             # the five Slice-1 audits, unchanged
docs/visual-changes.md                      # §20.10 ledger; xtask bless-guard reads it
docs/guides/{quickstart.md, theming.md, overrides.md, authoring.md, migration.md}
shots/                                      # capture artefacts
tools/{capture.sh, ansi2png.py, ansi2html.py}

crates/tui/                                 # package junie-tui, lib junie_tui
  Cargo.toml
  src/
    lib.rs            # #![deny(missing_docs)] #![forbid(unsafe_code)]; the curated facade
    author.rs         # the component-author layer (B.4)
    id.rs  event.rs  intent.rs  response.rs  keymap.rs
    focus.rs  hit.rs  capture.rs  scroll.rs  cursor.rs
    layer.rs  runtime.rs  diagnostics.rs
    layout.rs  measure.rs
    ui/{mod.rs, cx.rs, paint.rs, surface.rs, layer_buf.rs}
    text/{mod.rs, buffer.rs, editor.rs, measure.rs, fuzzy.rs}
    theme/{mod.rs, tokens.rs, role.rs, glyph.rs, recipe.rs, patch.rs, resolve.rs, downgrade.rs}
    theme/builtin/{mod.rs, junie.rs, paper.rs}
    collection/{mod.rs, key.rs, reconcile.rs, rowui.rs, decor.rs, empty.rs}
    components/{mod.rs, button.rs, choice.rs, chip.rs, brand.rs, keyhint.rs, too_small.rs,
                field.rs, input.rs, textarea.rs, select.rs, secret.rs, validate.rs,
                list.rs, tree.rs, props.rs, steps.rs, nav_list.rs, tabs.rs,
                panel.rs, split.rs, scroll_region.rs, viewport.rs,
                dialog.rs, menu.rs, picker.rs, filter_list.rs, completion.rs,
                form.rs, wizard.rs, picker_chain.rs, help.rs,
                status.rs, hintbar.rs, progress.rs, meter.rs, code.rs, diff.rs, grid.rs}
  examples/           # 01_button.rs … 12_author_component.rs  (external-style consumers)
  tests/
    conformance.rs  render.rs  architecture.rs  perf.rs  perf_baseline.txt
    baselines/components.txt
    fixtures/{grid_model.rs, rows.rs, text.rs}

crates/tui-testing/                         # package junie-tui-testing, publish = false
  Cargo.toml
  src/{lib.rs, harness.rs, digest.rs, perf.rs, conformance/mod.rs, conformance/driver.rs}

apps/showcase/            Cargo.toml  src/{main.rs, app.rs, data.rs, pages/*.rs}
                          tests/{app_tests.rs, visual.rs, perf.rs, baselines/showcase.txt}
apps/tablepro/            Cargo.toml  src/{main.rs, app.rs, workbench.rs, tabs.rs, connections.rs,
                                           grid_model.rs, filter_editor.rs, db.rs, model.rs, sql.rs}
                          tests/{app_tests.rs, visual.rs, perf.rs, baselines/tablepro.txt}
apps/jackin-preview/      Cargo.toml  src/{main.rs, app.rs, arbiter.rs, clock.rs, scenario.rs, rain.rs,
                                           screens/**, domain/**, sim/**}
                          tests/{app_tests.rs, app_tests_chrome.rs, visual.rs, perf.rs,
                                 baselines/jackin.txt}

xtask/                    Cargo.toml  src/main.rs   # boundary checks, bless-guard, capture matrix driver
```

Root `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members  = ["crates/tui", "crates/tui-testing", "apps/showcase", "apps/tablepro", "apps/jackin-preview", "xtask"]

[workspace.package]
version      = "0.1.0"
edition      = "2024"
rust-version = "1.88"
license      = "MIT"

[workspace.dependencies]
ratatui              = { version = "0.30", features = ["crossterm_0_29"] }
unicode-width        = "0.2"
unicode-segmentation = "1"
bitflags             = "2"
smallvec             = "1"
junie-tui            = { path = "crates/tui" }
junie-tui-testing    = { path = "crates/tui-testing" }

[profile.release]
lto = "thin"
codegen-units = 1
```

`apps/showcase/Cargo.toml` (the other two are identical in shape):

```toml
[package]
name = "showcase"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false

[[bin]]
name = "showcase"
path = "src/main.rs"

[dependencies]
junie-tui.workspace = true
ratatui.workspace   = true

[dev-dependencies]
junie-tui-testing.workspace = true
```

`jackin-preview` sets `[[bin]] name = "jackin-preview"` with `path = "src/main.rs"`, preserving the hyphenated binary while the package directory stays readable.

### B.3 `pub` vs `pub(crate)` policy

1. **Two documented layers, one crate.** `junie_tui::*` is the application-author surface; `junie_tui::author::*` is the component-author surface. Both are `pub` and separately documented with a module-level rustdoc header stating who the audience is (§13). Nothing else is `pub`.
2. **`lib.rs` is a curated facade, not a `pub mod` list.** Every module is `pub(crate) mod`; `lib.rs` re-exports the named items. Adding a type to the public API is a deliberate line in `lib.rs`, reviewable in a diff. `pub use` globs are forbidden.
3. **No public fields on component types** (invariant S1). Public fields exist only on plain data records with no behaviour and no geometry: `LayerSpec`, `Dismiss`, `StylePatch`, `StateRule`, `PartRecipe`, `ColorTokens`, `DesignTokens` and their sub-structs, `Role`/`GlyphRole`/`Part`/`Family`/`Variant` newtypes' constants, `FocusEntry`, `Headroom`, `Insets`, `Size`, `Constraints`, `RowDecor`, `CellDecor`, `Binding`, `Hint`, `HintLayer`, `LayoutFacts`, `Capture`, `PartRef`, `Key`, `Chord`, `Mouse`, `FieldError`. `architecture::no_public_geometry_or_cache` enforces the geometry half.
4. **`#[non_exhaustive]`** on every public struct with pub fields that a future token or capability may extend (`ColorTokens`, `DesignTokens`, `LayerSpec`, `RowDecor`, `CellDecor`, `LayoutFacts`), so adding a field is not a breaking change for downstream authors. `ColorTokens` is the deliberate exception in one direction: `map_colors` destructures it exhaustively *inside* the crate so adding a token is a compile error there (§11.4).
5. **No `#[doc(hidden)]` public items.** If something must be reachable it is documented in `author`; if it must not, it is `pub(crate)`.
6. **`#![deny(missing_docs)]` and `#![forbid(unsafe_code)]`** at the top of `crates/tui/src/lib.rs`. The single `unsafe impl GlobalAlloc` lives in `crates/tui-testing`, carries a written safety rationale, and is covered by `debug_and_release_alloc_counts_match`.
7. **Applications export nothing.** Each app is a binary-only package (`publish = false`, no `[lib]`); its tests live in `tests/` and reach the app through a small `pub` surface declared in `main.rs` behind `#[cfg(test)]`-friendly visibility — the app's `const Id`s, its `App` type and its screen enums. This is the migration contract of §16.4 item 3.

### B.4 The `author` module

`junie_tui::author` is a re-export module, not a second implementation. It is what example 12 and every downstream component author consumes, and it is the mechanical proof of Scenario G: if a component can be written with it, no private access is needed.

```rust
//! Component-author API. Everything needed to build a component that participates in
//! theme resolution, focus, hover, press, dispatch, hit testing, cursor output,
//! scrolling, overlays, capture, testing and visual capture — and nothing more.
pub mod author {
    // identity and parts
    pub use crate::id::{id, Id, ItemKey, Part, PartRef};
    // phases and plumbing
    pub use crate::ui::{Ui, Cx};
    pub use crate::intent::{Intent, IntentIter, Phase, FocusVia};
    pub use crate::response::{Response, Flow, Invalidate, StateFlags};
    pub use crate::event::{Input, Key, KeyCode, KeyModifiers, Chord, Mouse, MouseKind, Axis};
    // registration services
    pub use crate::focus::{Focusability, ScopeMode, ScopeId, FocusVis};
    pub use crate::hit::{Axes, Headroom};
    pub use crate::capture::Capture;
    pub use crate::layer::{LayerId, LayerKind, LayerSpec, Anchor, Side, CrossAlign,
                           Dismiss, Backdrop, LayerEvent, DismissReason};
    // theme resolution
    pub use crate::theme::{Theme, Family, Variant, Role, FgStep, SyntaxRole, MeterRole,
                           GlyphRole, Surface, StylePatch, Slot, StateRule, Overlay,
                           Resolved, Modifier, Density, ColorLevel, DesignTokens};
    // layout and measurement
    pub use crate::layout::{self, Track, RowAlign, Insets, SplitModel};
    pub use crate::measure::{Measure, Size, Constraints};
    pub use crate::ui::LayoutFacts;
    // text
    pub use crate::text::{TextEditorCore, EditAction, EditOutcome, CursorPos,
                          width, wrap, fuzzy, truncate, truncate_middle};
    // collections
    pub use crate::collection::{RowUi, CellUi, ColumnsUi, RowDecor, CellDecor,
                                EmptyState, RowTotal, Reconciliation, SelectMode, KeySet};
    // bindings and hints
    pub use crate::keymap::{Binding, Bindings, BindingState, KeyMap, Phase2, Hint, HintLayer};
    // errors and diagnostics
    pub use crate::{FieldError, LayoutError, Validate, NoValidate, Secret, SecretPolicy};
    pub use crate::diagnostics::Diagnostic;
    // ratatui types a painter needs
    pub use ratatui::layout::{Rect, Position};
    pub use ratatui::style::{Color, Style};
    pub use ratatui::buffer::{Buffer, Cell};
}
```

What is deliberately **not** in `author`: `Runtime`, `run`, `TerminalSession`, `Registry`, `FocusRing`, `FocusState`, `App`, and the concrete components. A component author drives none of those; an application author reaches `Runtime`/`run`/`App` from the root facade, and tests reach `FocusRing` through `Harness::ring()`. `architecture::conformance_covers_every_public_component` plus example 12 compiling with `use junie_tui::author::*;` and **no other `junie_tui` path** is the standing proof that the split is honest.

### B.5 Examples and capture tooling

* The twelve §17 examples live in `crates/tui/examples/` and are built by `cargo build -p junie-tui --examples` in every slice gate. Because Cargo compiles examples as separate crates linked against `junie_tui`, they see exactly the public API and nothing else — the "external-style consumer" requirement of goal §21 is satisfied structurally, not by convention.
* Doctests carry the condensed forms of examples 1–10 on the corresponding types and run under `cargo test --workspace --doc`.
* `tools/capture.sh` changes in two places, both in Slice 5: `BIN` defaults to `target/debug/showcase` (the current default names a binary that does not exist), and `ARGS` gains documented `--theme {junie|paper}` and `--color {truecolor|256|16|none}` pass-through so the §20.10 review matrix is scriptable. `xtask capture-matrix` drives it over the full size × theme × colour × app grid and writes into `shots/`, so the visual reviewer receives a complete, reproducible set rather than ad-hoc screenshots.
