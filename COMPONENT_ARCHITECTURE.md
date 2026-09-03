# COMPONENT_ARCHITECTURE.md

**Status:** Accepted, with the Slice 2 review corrections of §21 (Adjudication J) applied. This document is the single source of truth for the refactor. Builders implement it as written; a change to any *Decision*, *invariant*, exact type, or precedence rule requires a fresh `opus-analyst` adjudication recorded here and in `REFACTORING_STATE.md` (goal §0).

**Authority:** `REFACTORING_GOAL.md` › `DESIGN.md` › existing rendered output/tests › current source. Where the Slice‑1 audits conflict, the adjudications in §3–§15 below are final; the rejected alternative and the reason are stated with each.

**Inputs adjudicated:** `docs/audit/api-audit.md` (API), `docs/audit/app-audit.md` (APP), `docs/audit/domain-boundary-audit.md` (DOM), `docs/audit/interaction-audit.md` (INT), `docs/audit/architecture-research.md` (RES). `docs/audit/performance-audit.md` (PERF) landed after §1–§15 were written; §20.9 folds its obligations in and amends earlier decisions where needed.

Every claim tagged **[F]** is a collected fact carried from an audit with its citation. Everything else is a decision or its rationale.

---

## 0. Traceability

<!-- amended by §21 item 33 -->

### 0.1 Goal §9's twenty required sections (M19)

| Goal §9 item | Where in this document |
|---|---|
| 1 Current-state diagnosis | §1 |
| 2 Design goals and non-goals | §2 |
| 3 The component model | §3 |
| 4 State ownership rules | §4 |
| 5 Rendering rules | §5 |
| 6 Event and semantic-action model | §6 |
| 7 Component identity and stable-key model | §7 |
| 8 Focus and interaction model | §8 |
| 9 Overlay and modal model | §9 |
| 10 Layout, measurement, and surface inheritance | §10 |
| 11 Theme and customization model | §11 |
| 12 Composition and component-part model | §12 |
| 13 Public API conventions | §13 |
| 14 Generic versus domain-specific boundaries | §14 |
| 15 Package or workspace boundary | **Appendix B** — the document's §15 is "Forms and text editing" (goal §19), an additional section |
| 16 Testing strategy | §16 |
| 17 Representative proposed usage examples | §17 |
| 18 Migration map for every current component | §18 |
| 19 Alternatives considered and rejected | §19 |
| 20 Known trade-offs | §20 |
| — additional | §15 Forms and text editing (goal §19); Appendix A Slice plan; §21 Slice 2 review corrections |

### 0.2 Goal §23 scenarios (M20)

| Scenario | Executable proof |
|---|---|
| A Simple interactive screen | `examples/11_small_app.rs` |
| B Custom theme | `examples/02_custom_theme.rs` + `Theme::paper()` + `theme::downgrade_works_for_a_user_supplied_theme` |
| C Local override | `examples/05_instance_patch.rs` + `render::overrides::instance_patch_changes_only_one_instance` |
| D Custom collection content | `examples/07_borrowed_rows.rs` + `no_full_collection_clone_per_frame` |
| E Dynamic identity | `examples/08_dynamic_tabs.rs` + `conformance::tabs::item_identity_survives_reorder` |
| F Nested overlays | `examples/10_nested_overlay.rs` + `layer::nested_layers_each_trap` |
| G Custom component authoring | `examples/12_author_component.rs` + `showcase::author_component_page_participates_in_focus_and_hover` |
| H TablePro adapter | `crates/tui/tests/fixtures/grid_model.rs` + `tablepro::grid_adapter_keeps_every_pending_change_capability` + `architecture::no_domain_vocabulary_in_the_library` (no §17 example; proven by Slice 6 tests) |
| I Visual preservation | §20.10 + `showcase_visual_baseline` |

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

<!-- amended by §21 items 3, 6, 11, 12, 13, 14, 15, 16, 17 -->

`Input` never touches a component directly. `Runtime<A>` owns all interaction state; the app owns only domain state and `XState`s.

```
Runtime::handle(&mut self, input: Input) -> Response<()>
 ── INPUT PHASE (no buffer in scope) ─────────────────────────────────────────
  1. normalize        Input from crossterm; drop key releases and unmapped buttons.
                      Resize -> record size, invalidate = Layout, no intents.
                      Tick   -> advance flash/motion clocks, then step 7 (app.tick).
                      Paste  -> text copied into a runtime-owned frame arena that outlives
                                step 7, so Intent::Paste(&'f str) borrows it (§21 item 6).
  2. capture keymap   app KeyMap "Capture" bindings are matched FIRST, but a chord
                      that is a bare Char is skipped while `focus_owner_swallows_typing`
                      (§11.4). A capture hit produces an app action and skips 3-8.
  3. resolve          against LAST FRAME's Registry and FocusRing (never a fresh scan
                      of the app tree):
                        pointer -> Registry::hit(pos) -> Hit{owner,part,layer,kind,local}: the
                                   topmost region REGARDLESS of layer; delivered iff
                                   hit.layer == top_layer, otherwise it is the top layer's
                                   outside-click (§21 item 12)
                                   (a live Capture short-circuits this: the capture owner
                                    receives Drag/Release with `local` against the captured area)
                        wheel   -> Registry::hit_scroll(pos, axis) -> innermost scrollable
                                   handling that axis, returned even at zero headroom
                        key     -> FocusState::current()  (None -> app bubble phase only)
                        paste   -> the focused owner iff it declared EDITING
  4. interaction      hover / hover_suppressed / press bookkeeping / 140 ms flash /
                      double-click window / capture claim+release. All of §1.2(4).
  5. focus policy     Tab, Shift+Tab and press-focuses-owner are executed by the runtime
                      against the last ring (Esc moved to step 8, §21 item 3), honouring
                      focus scopes and traps (§8). Focus changes here are staged, not
                      applied mid-queue.
  6. enqueue          Intents into the frame's IntentQueue, keyed by owner Id; the queue is
                      frozen for the whole of step 7 (§21 item 6). A staged focus change
                      enqueues Intent::FocusOut{to} to the old owner and
                      Intent::FocusIn{via} to the new one, in that order.
  7. app.update       app.update(&mut cx) -> Response<()>; screens call component
                      `update`s, which drain their own intent bucket by Id via
                      cx.intents(id). If the pass changed focus (cx.focus(id), a closed
                      layer, a reconciliation), re-run 6-7. A re-run enqueues ONLY
                      Intent::FocusOut{to} and Intent::FocusIn{via}; already-drained
                      buckets are never refilled, so no input intent is delivered twice.
                      The Response of each pass is folded into the first with `|`. If a
                      5th pass is required, the runtime emits
                      Diagnostic::FocusTransitionDidNotSettle, applies the pending FocusOut
                      AND the matching FocusIn to the last requested target without
                      re-running app.update, and continues (suite test
                      focus_transition_settles asserts the count is 0) (§21 item 11).
                      Captures whose owner's layer was closed by this pass are released
                      here as well as at step 13 (§21 item 17, F8).
  8. bubble           keys still unconsumed after step 7 are offered to (a) the app KeyMap
                      "Bubble" bindings, then (b) Dismiss.esc on the top layer, then (c) the
                      screen's Esc ladder (§21 item 3). Esc therefore reaches an editing
                      control inside a layer before the layer.
  9. finish           Intents that no component drained are dropped. An intent whose
                      resolved owner registered only Decorative regions is discarded
                      silently; `UndeliveredIntent` is recorded only when the owner
                      registered a Control or Part region and drained nothing (§21 item
                      13). Returns Response<()> whose `flow` and `invalidate` are the fold
                      of everything above.

Runtime::draw(&mut self, frame: &mut Frame)
 ── DRAW PHASE (no &mut app state in scope) ──────────────────────────────────
 10. new frame state  Registry::new(gen+1), FocusRing::new(), FrameOut::new(),
                      layer buffer pool reset, StyleStack reset to the theme.
 11. app.draw         app.draw(&mut ui) with layer 0 painting straight into the frame
                      buffer. Registrations carry {owner, part, area, layer, kind, gen}
                      with kind: RegionKind (§21 item 13).
 12. layers           ui.layer(id, |ui, area| …) (no spec: the LayerId was assigned by
                      cx.open_layer, §21 items 14, 16) executes immediately but paints
                      into a pooled layer buffer with a written-cell bitset and pushes a
                      focus scope + a hit layer; the runtime composites layers
                      bottom-to-top after app.draw returns, so z-order is the layer
                      order, NOT the call order. A second ui.layer call with the same id
                      in one frame returns None and records DuplicateLayerDraw (F10).
 13. registry swap    last = new. Captures whose owner or area vanished are released.
                      Stale intents from an older generation are dropped.
 14. focus reconcile  if FocusState::current is absent from the new ring:
                        (a) nearest surviving entry in the same scope by previous index,
                        (b) else that scope's first enabled entry,
                        (c) else the innermost active scope's first enabled entry,
                        (d) else None.
                      `focus_visible` is true iff the last input was a key.
                      Focus restoration is staged at close_layer and applied here; until
                      then FocusState::current is the restore target and key resolution
                      uses it even though it is absent from the last ring — the one
                      documented exception to "resolve against last frame" (§21 item 15).
 15. cursor           the single retained cursor write is kept iff
                      `layer == top_layer && FocusState::current() == owner`; otherwise
                      dropped (debug diagnostic). Then frame.set_cursor_position or hide.
```

Latency: pointer intents resolve against the registry the user actually saw, which is the current behaviour (**[F]** all three apps rebuild hits/ring after drawing: `showcase/app.rs:711-721`, `tablepro/app.rs:2094-2105`, `jackin/app.rs:2254-2265`). A component drawn for the first time this frame is not clickable until it has been drawn once — identical to today, and documented.

`§25.6` compliance: pointer/wheel cost is one reverse scan of the top layer's regions; key cost is one hash lookup; per-component cost is one `Cx::intents(id)` probe, and zero probes when the queue is empty (§21 item 6). No tree walk, no per-event data scan. <!-- amended by §21 item 6 -->

### 3.4 A jackin `Screen` trait method after migration

Before — **[F]** 20 methods, 11 of them input (`screens/mod.rs:231-328`), plus `Cx{focus,ring,requests}` handing every screen `&mut Focus`.

After — 6 methods; `Cx` no longer carries `Focus` or `FocusRing`. The product half of today's `Cx` (requests, `go`, `status`, `open`, `close`, `help`, `copy`, `with_form`) is jackin's own `Jx<'_>`, passed alongside the library `Cx` (§21 item 32, M12): <!-- amended by §21 items 18, 32 -->

```rust
pub trait Screen {
    /// The only input entry point. Intents are already resolved to (owner, part).
    /// Domain `Msg`s are drained by the app before this runs; `Input` has no `Msg` variant.
    fn update(&mut self, cx: &mut Cx<'_>, jx: &mut Jx<'_>, w: &mut World) -> Response<()>;
    /// Pure paint. `&self` makes a semantic mutation a compile error.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, w: &World);
    /// Product-level hints only; component bindings are contributed automatically (§13.1).
    fn hints(&self, _w: &World) -> HintLayer { HintLayer::empty() }
    fn crumb(&self, w: &World) -> String;
    fn primary_focus(&self) -> Option<Id>;
    fn on_esc_top(&mut self, _cx: &mut Cx<'_>, jx: &mut Jx<'_>, _w: &mut World) -> Response<()> {
        jx.go(Go::Manager); Response::consumed().repaint()
    }
}
```

`on_click`, `on_double_click`, `on_drag`, `on_secondary`, `on_press`, `on_release`, `on_wheel`, `on_paste`, `on_tick`, `on_msg`, `on_modal`, `picker_items`, `form_changed`, `strip_right`, `is_editing`, `animating`, `enter` are removed or subsumed: pointer/wheel/paste/focus arrive as intents inside `update`; `on_modal` becomes `LayerEvent` intents (§9.5); `is_editing`/`animating` become `cx.request_repaint_after(…)` and `StateFlags::EDITING` on the focused owner; `enter` becomes a `LayerEvent::Opened`/route-change intent; `picker_items`/`form_changed` become ordinary `update` work on the screen's own state.

A concrete migrated body (jackin Manager, condensed):

```rust
fn update(&mut self, cx: &mut Cx<'_>, jx: &mut Jx<'_>, w: &mut World) -> Response<()> {
    let mut r = Response::ignored();

    r |= Self::tree()                                    // configuration only; rows are per phase (§21 item 1)
        .update(cx, &mut self.tree_state, &self.rows)
        .on_action(|a| match a {
            TreeAction::Activated(k) => self.open(k, w, jx),
            TreeAction::Expanded(k)  => self.expand(k, w),
            _ => {}
        });

    r |= Self::detail_button().update(cx).on_activated(|| jx.go(Go::Editor(self.current)));

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
* **S5** External data change is reconciled explicitly by the component's `update`, before any action is emitted: `st.reconcile(len, key)` (`Reconcile`, §12.2, §21 item 21). No component silently retargets an edit after a reorder.
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

**R8** <!-- amended by §21 item 2 --> `Ui::cache<T>(id)` is the only mutable state reachable from `draw`. Its contents must be a pure function of (props, state, area, theme); a component that reads a value from `cache` that is not derivable from those inputs is a bug. It is runtime-owned, keyed by `(Id, TypeId)`, cleared on resize, theme change and generation gap, never observable in `Response`, never compared by `draw_twice_leaves_state_equal`, and guarded by `architecture::cache_types_are_derived_only`.

**R9** <!-- amended by §21 item 30 (A4) --> A component that declares a part must paint `Resolved.glyph` when it is `Some`; `conformance::registry::declared_parts_are_the_parts_actually_styled` checks the query and `mono_states_are_distinguishable` checks the paint.

---

## 6. Event and semantic-action model — **Adjudication C**

### 6.1 Decision — one exact type set

```rust
// ---------- raw input (unchanged in spirit; `Input` is the runtime boundary) ----------
pub enum Input { Key(Key), Mouse(Mouse), Resize(u16, u16), Paste(String), Tick }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key { pub code: KeyCode, pub mods: KeyModifiers }             // Key::is / Key::chord in §17.0 A8
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Chord { pub code: KeyCode, pub mods: KeyModifiers }   // Chord::key / Chord::with in §17.0 A8
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
pub struct Response<A = ()> {                     // <!-- amended by §21 item 4 -->
    id: Option<Id>,                                // None for Response::ignored()
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
    pub fn id(&self) -> Option<Id>;
    pub fn flow(&self) -> Flow;
    pub fn is_consumed(&self) -> bool;               // B1: readers are `is_*`; the constructors above keep their names
    pub fn invalidate(&self) -> Invalidate;
    pub fn is_changed(&self) -> bool;                // invalidate >= Paint
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
// Folding is defined for `Response<()>` ONLY: composing two action-carrying responses is a type
// error, never silent loss. flow: Consumed wins; invalidate: max; id and state: lhs — the fold is
// a control-flow summary; read `state`/`id` from the individual responses. (§21 item 4)
impl std::ops::BitOr for Response<()>       { /* … */ }
impl std::ops::BitOrAssign for Response<()> { /* … */ }
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

**Debuggability.** `Debug for Id` prints `orders.list ▸ Row(#0x1f3a)` in debug builds and `Id(3f9a…)` in release. The debug label travels with the `Id` itself (`Tail::Item(k)` carries the item key inline), so no side table exists in any build and a diagnostic prints the path. <!-- amended by §21 item 22: `Registry::names` struck --> `PartialEq`/`Hash`/`Ord` ignore the label, so debug and release compare identically (test `id_equality_ignores_debug_label`).

**Collision safety.** `Registry::register` records a duplicate as a `Diagnostic::DuplicateId { id, first, second }`, surfaced through `Runtime::diagnostics()` in debug builds and asserted by tests. Never a panic in release (goal §10).

**Where ids come from.** Components: one `Id` per component instance, supplied by the caller as a `const` (`const SAVE: Id = id!("save");`) or derived (`self.id.item(key)`). **`Id::part(p)` versus `PartRef`** <!-- amended by §21 item 16 -->: `Id::part(p)` mints a *child component id* (a `Button` inside a `Dialog`: `const OWNER_BTN: Id = DLG.part(Part::custom("owner"))`); that component registers its own `Control` region and is found by `Runtime::area_of`. `PartRef` tags a *sub-region of a single component* (a tab's close glyph, a scrollbar thumb; `PartRef::of(p)` / `PartRef::item(p, k)`) and is found by `Runtime::area_of_part`. They are never interchangeable, and a component's own sub-regions are never addressed by a derived id in application code. `Id::index` exists for genuinely positional cases and is rejected by a debug assertion when a keyed collection's length changes while `Index` keys are live.

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
    pub fn capture(&mut self, owner: Id, part: PartRef) -> bool;   // claim; false if another capture is live (§21 item 18)
    pub fn capture_owner(&self) -> Option<Id>;
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

`ui.set_cursor(owner: Id, pos: Position)` records `(layer, owner, pos)`. The runtime keeps the write iff `layer == top_layer && FocusState::current() == owner`; otherwise it drops it and records `Diagnostic::CursorRejected`. A `set_cursor` from a suppressed (inert) layer is discarded silently; `CursorRejected` is recorded only for a non-inert lower layer or an unfocused owner. <!-- amended by §21 item 15 --> A background `TextInput` still flagged `EDITING` can never place the cursor under a dialog (today only draw order prevents it).

### 8.5 Invalidation (confirms INT B10)

`Invalidate` on `Response` is the return channel. Out-of-band sources use `cx.request_repaint()` and `cx.request_repaint_after(Duration)`, which the runtime folds into a repaint deadline — deleting the per-app `animating()`/`tick_interval()` heuristics (**[F]** `showcase:308-324`, `tablepro:192-197`, `jackin:299-320`). Cadence values (`tick_ms` 80, `idle_tick_ms` 400, `press_flash_ms` 140, `status_ms` 4000/5000) are design tokens. `Invalidate::Layout` ships from day one but currently behaves as `Paint`; it is reserved for layout caching and is asserted only for ordering.

### 8.6 Hover, press, activation, double-click, click-outside — runtime-owned

Hover never changes focus. Hover is suppressed after any key press until the pointer moves (`DESIGN.md:648`). Press records the target; release activates only on the same target (or inside a capture area). Keyboard and mouse activation produce the identical action (conformance test). Double-click is a 500 ms same-target window, owned by the runtime, delivered as `Phase::DoubleClick`; the `was_focused` argument threaded into `TextInput::on_click` disappears (**[F]** `input.rs:247-261` and five app call sites). "Click outside" is `hit.layer < top_layer || hit.is_none()` — a real outside test rather than "the hit returned None"; it is real because `Registry::hit(pos)` returns the topmost region *regardless of layer* and the runtime compares layers (§21 item 12). Esc is offered to the focused component first and to the top layer's `Dismiss.esc` only in the bubble phase (§3.3 step 8, §21 item 3). <!-- amended by §21 items 3, 12 -->

---

## 9. Overlay and modal model — **Adjudication E (B7)**

### 9.1 Decision — a runtime-owned layer stack

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] pub struct LayerId(u16);   // 0 = page

pub enum LayerKind { Modal, Popover, Tooltip }        // Modal: focus+pointer trap; Popover: pointer only
pub enum Anchor {
    Screen(ScreenAlign),                               // ScreenAlign::{Center, UpperThird, Bottom} (§21 item 20)
    Rect { rect: Rect, side: Side, align: CrossAlign }, // Side::{Below, Above, Left, Right}
    Point(Position),
}
pub struct Dismiss { pub esc: bool, pub outside_click: bool, pub focus_out: bool }
#[non_exhaustive]                                      // construct through the §17.0 A6 builders (§21 item 8)
pub struct LayerSpec {
    pub kind: LayerKind,
    pub owner: Id,                 // anchor owner + focus-restore target
    pub anchor: Anchor,
    pub dismiss: Dismiss,          // focus_out is honoured only for Popover/Tooltip; a Modal traps focus (A10)
    pub restore_focus: bool,
    pub initial_focus: Option<Id>,
    pub min_size: (u16, u16),      // §21 item 20
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
| barriers | one index in each registry, set from `render` | regions carry `layer`; `hit()` returns the topmost region regardless of layer and the runtime delivers iff `layer == top` (§21 item 12) |
| inert background | unused flag | `inert_below`: no ring entries, no hit regions, no cursor writes |
| Esc | 6 widget impls + 3 app ladders | the focused component first (step 7), then app Bubble bindings, then `Dismiss.esc` on the top layer, then the screen's ladder (§21 item 3) |
| click-outside | "hit returned None" | "hit's layer < top layer, or None" — expressible because `hit()` does not filter by layer |
| focus restore | 3 `saved_focus` fields | `restore_focus` + scope memory (§8.1) |
| cursor | last writer wins | top-layer + focused owner only (§8.4) |
| hints | jackin only | the top layer contributes its hint layer automatically (§11.4) |
| lifecycle | polled `result` fields | `LayerEvent::{Opened, Dismissed, Closed}` |
| small terminal | 3 ad-hoc screens | `min_size`, then clamp, then documented degradation |

`Dialog`, `Picker`, `ContextMenu`, `MenuBar` dropdowns, `Select`'s popup, `Completion`, and jackin's `FileBrowser`/`ChoiceDialog`/`FormDialog`/`InfoDialog`/`HelpOverlay` all become **content rendered into a layer**. `begin_modal` and the shared `"popup.surface"` id are deleted, and with them the six blocks of manual hit re-registration (**[F]** APP §2.2).

**Rejected:** sorted-`z` widgets (solves paint only); modality as a render side effect; per-app stacks.

**Layer identity and draw order** <!-- amended by §21 items 14, 17 -->. `LayerId` is assigned monotonically by `Cx::open_layer` and is the stack position. `Ui::layer(id, f)` resolves `id` to its already-assigned `LayerId`, executes `f` into that layer's pooled buffer, and returns `None` without executing `f` if `id` is not open. Call order at draw time has no effect on z-order, hit filtering, or focus scope nesting (`layer::layer_id_is_assigned_at_open_not_at_draw`). A second `ui.layer` call with the same `id` in one frame returns `None` and records `Diagnostic::DuplicateLayerDraw` (F10). With `inert_below`, no scroll region is registered beneath the layer, so a wheel over the backdrop falls through to the app bubble phase; there is no outward chaining (F9).

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
    PressLeft, PressRight,                    // mono PRESSED brackets (§21 item 25)
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
    pub capability: Capability,   // { color: ColorLevel } — UnicodeLevel deleted (§21 item 19)
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

**`ThemeBuilder::build` derivation (exact)** <!-- amended by §21 item 29 -->. Given `surfaces[0]` and `accent`: `surfaces[1..4]` step L\* by +4 (dark base) or −4 (light base, detected by L\*(surfaces[0]) > 50); `fg[0..4]` step L\* by −18 from a contrast-7:1 anchor against `surfaces[0]`; `accent_hover = ΔL* +8`, `accent_pressed = ΔL* −8`, `accent_tint` = accent at 12 % over `surfaces[1]`; `focus = accent`, `focus_ring = accent_pressed`; `border_subtle = surfaces[3]`, `border_strong = fg[3]`; `danger`/`warning`/`success`/`info` tints at 12 %; `on_accent`/`on_danger` = whichever of `fg[0]`/`surfaces[0]` reaches ≥ 4.5:1. Every derived value is a pure function of the seeds; `theme::builder_derives_every_unset_token_deterministically` pins the table and `theme::derived_tokens_meet_design_contrast_ratios` checks the ratios. `Theme::paper()` (§11.7) is `from_tokens(seeds).builder()…build()` and is pinned by `theme::paper_tokens_are_pinned`.

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
pub fn downgrade_color(c: Color, level: ColorLevel) -> Color;   // §21 item 29, exact:
// nearest_256: nearest in the 6×6×6 cube ∪ 24-step greyscale by squared sRGB distance, ties to the lower index.
// nearest_16:  nearest of the 16 xterm defaults by CIE76 ΔE.
// mono:        Y = 0.2126R + 0.7152G + 0.0722B; Y < 0.35 → black, Y > 0.75 → white, else Color::Reset.
// Test: theme::downgrade_is_deterministic_per_level.
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
| `SELECTED` / `CHECKED` | `Part::MARKER` glyph = `Chosen` / `Checked`, never colour-only; when both are live, `CHECKED` wins (§21 item 25) |
| `PRESSED` | `Part::CONTAINER` `bg = Role::Fg(Primary)`, `fg = Role::Surface(Canvas)` (explicit reverse, never the terminal REVERSE attribute) **plus** `add(Modifier::BOLD)`, **and** `Part::LABEL` bracketed with `GlyphRole::PressLeft` / `GlyphRole::PressRight` (Junie `[` `]`, rendered `[Save]`) — a colour-only rule was indistinguishable under conformance case 9 (§21 item 25) |
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
| collection data | **passed per phase, never held in props** — `update(cx, &mut st, items)` / `draw(ui, area, &st, items)`, so the props never borrow the field an action closure mutates (§21 item 1) |

`&'a dyn Fn` slots and small `&dyn Trait` extension points (`Validate`, `Highlighter`, `Segmenter`, `DiffSource`, `GridModel`) are the only dynamic dispatch in a component's public surface; none is boxed and none requires `'static`. <!-- amended by §21 items 1, 22 -->

### 12.2 The one collection vocabulary

<!-- amended by §21 items 1, 5, 8, 21, 22, 30 -->

Applied to `List`, `Tree`, `Grid`, `Tabs`, `Picker`, `Completion`, `Props`, `Steps`, `Chips`:

```rust
// identity: every collection takes a key accessor; every action carries an ItemKey.
// Real defaults instead of `()`, and blanket traits the phase methods bound on (§21 item 5):
pub struct ByIndex;      // ItemKey::index(i) — UNSTABLE under reorder; call .key(..) for stable identity
pub struct DefaultRow;   // the item's `Display` via RowUi::label_fmt, no allocation
pub trait KeyFn<T> { fn key(&self, item: &T, index: usize) -> ItemKey; }
impl<T, F: Fn(&T) -> ItemKey> KeyFn<T> for F { /* … */ }
impl<T> KeyFn<T> for ByIndex { /* … */ }
pub trait RowFn<T> { fn row(&self, item: &T, u: &mut RowUi<'_>); }
impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F { /* … */ }
impl<T: core::fmt::Display> RowFn<T> for DefaultRow { /* … */ }

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
    pub fn part(&mut self, p: Part, width: u16) -> CellUi<'_>;   // reserves `width` columns from the RIGHT; `label` fills what is left
    pub fn label_fmt(&mut self, args: core::fmt::Arguments<'_>);   // in-place formatting, 0 allocations (DefaultRow)
    pub fn raw(&mut self) -> (&mut Buffer, Rect);     // escape hatch, marks the rect written
}
pub struct CellUi<'u> { /* … */ }
impl CellUi<'_> {                                     // exact (§21 item 21)
    pub fn text(&mut self, s: &str) -> &mut Self;
    pub fn num(&mut self, n: i64) -> &mut Self;        // formats in place, 0 allocations
    pub fn money(&mut self, cents: i64) -> &mut Self;  // ditto
    pub fn align(&mut self, a: Align) -> &mut Self;
    pub fn tone(&mut self, r: Role) -> &mut Self;
    pub fn italic(&mut self, yes: bool) -> &mut Self;
    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self;
    pub fn patch(&mut self, p: &StylePatch) -> &mut Self;
}

// state of the data, not of the widget
pub enum EmptyState<'a> {
    Empty  { title: &'a str, hint: Option<&'a str> },
    Loading{ label: &'a str },
    Partial{ loaded: usize, total: RowTotal, hint: &'a str },
    Error  { message: &'a str, detail: Option<&'a str> },
}
pub enum RowTotal { Exact(usize), Estimated(usize), Unknown }

// decoration supplied by the owner, never derived inside the component
#[derive(Clone, Copy, Default)]                                  // not #[non_exhaustive]: adapters build literals (§21 item 8)
pub struct RowDecor<'a>  { pub marker: Option<GlyphRole>, pub tone: Option<Role>,
                           pub strike: bool, pub faint: bool, pub flags: StateFlags,
                           pub message: Option<&'a str> }              // no 'static in a component surface (§21 item 22)
#[derive(Clone, Copy, Default)]
pub struct CellDecor<'a> { pub tone: Option<Role>, pub italic: bool, pub error: Option<&'a str>,
                           pub dirty: bool, pub suffix: Option<GlyphRole> }

// reconciliation: one rule for every collection
pub enum Reconciliation { Unchanged, CursorMoved(ItemKey), CursorLost, SelectionDropped(usize) }
/// Implemented on every collection state type (ListState, TreeState, TabsState, GridState, …). (§21 item 21)
pub trait Reconcile {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    fn invalidate(&mut self);     // caller mutated items in place without changing len/ends
}
```

**The reconcile rule (identical for every collection, tested once, table-driven):** keep the cursor/active/selected key if it is still present; else take the nearest surviving key by the previous index (forward first, then backward); else the first enabled key; else `None`. Checked sets drop vanished keys and report the count. Scroll offset is clamped to the new length. Every collection's `update` calls `reconcile` **before** emitting any action.

`SelectMode { Single, Multi, Range, None }`; cursor, selection and activation are three distinct concepts in every collection (`RadioGroup` is fixed to separate cursor from value).

**Scrolling is shared:** `ScrollRegion<'a>` is a component (`components/scroll_region.rs`, `ScrollRegionCase` in the conformance suite) providing scrollbar registration, track arithmetic, thumb drag through pointer capture, and `ensure_visible_on_next_layout`, with a `Ui::scroll_region(id, part, …)` convenience that constructs and draws it — deleting seven copies of `on_scrollbar` (**[F]** DOM §6.1(6)) (M25). The scrollbar is `Part::TRACK`/`Part::THUMB` of its container, not a separate id space; `scrollbar::id_for` is deleted.

### 12.3 Grid split (confirms DOM §1.5)

<!-- amended by §21 items 1, 22, 30 -->

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
    // External: the grid emits GridAction::EditRequested(item, col) and does NOT begin an inline
    // edit; the application opens its own editor (A8). An inline editor registers a Control region
    // AFTER the grid's cell Part region and wins the click (§21 item 30).
    fn apply_cycle(&mut self, row: usize, col: usize);
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
    fn read_only_reason(&self) -> Option<&str> { None }
}
pub trait GridCellActions: GridModel {
    fn actions(&self, row: usize, col: usize) -> &[CellAction];    // glyph + chord + ActionKey
}
```

`GridModel` is `&self` (used by both phases); `GridEditor` is `&mut self` and is reachable **only from `update`**, through `Grid::update<M>(…, model: &mut M)` while `Grid::draw<M>(…, model: &M)` sees it shared — the model is a phase parameter, never a field of `Grid<'a>` (§21 item 1) — and the phase split makes "rendering stages a database mutation" (`grid.rs:1518`) unrepresentable. Everything database-shaped moves to `apps/tablepro/src/grid_model.rs`: `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, `default_validator`, `cmp_cells`, `apply_commit_result`, insert/duplicate/toggle-delete/discard/undo, `primary`/`nullable`/`references`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar, and `Theme::change_glyph`. All 22 TablePro capabilities survive by the mapping in DOM §1.6, which is adopted verbatim as the migration checklist for Slice 6.

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
| caller state | `&mut XState` per phase | `update(cx, &mut st[, data])` / `draw(ui, area, &st[, data])` |
| collection data | passed per phase, never held in props | `update(cx, &mut st, items)` / `draw(ui, area, &st, items)`; `Grid` takes `model: &mut M` / `&M` (§21 item 1) |
| library state | runtime-managed | focus, hover, press, flash, cursor, regions, layers, captures, style stack |
| controlled value | `&mut T` in update, `&T` in draw | draft in state, value written on commit |
| uncontrolled | `XState::value()` | documented per component as the exception |
| variants & sizes | `.variant(Variant)` / `.density(Density)` | typed newtypes, `custom()` escape hatch |
| disabled vs read-only | `.disabled(bool)` / `.read_only(bool)` | read-only stays in the ring; disabled does not |
| loading/busy/error/editing | `.status(Status)` + `StateFlags` | `EDITING` is owned by state, never a prop |
| events | `Response<XAction>` | `.action_ref()`, `.activated()`, `.is_changed()`, `.is_consumed()` (§21 item 4) |
| focus | automatic | `ui.register_control(id, area, Focusability)`; `.autofocus()`; `ui.focus_scope(…)` |
| render | `update` + `draw` | never fused; `draw` is `&self` |
| measure | `measure(&self, ui, Constraints) -> Size` | `Size { min, preferred }` |
| parts | `X::PARTS: &'static [Part]` | documented per component; used by theming, overrides, tests, hints |
| local override | `.patch(&StylePatch)` / `.patch_part(&[(Part, StylePatch)])` / `.slot(Part, &dyn Fn)` | |
| item renderer | `.row(Fn(&T, &mut RowUi))` / `.cell(…)` | closures, never `fn` pointers; default `DefaultRow` |
| identity | `.key(Fn(&T) -> ItemKey)` | actions carry `ItemKey`; default `ByIndex` (unstable, documented) |
| errors | `FieldError` | typed, `Display + Error`; no panic on any interaction path (`LayoutError` deleted, §21 item 19) |
| testing | `Harness` | `Harness::new(app, theme, w, h)`, `.key()`, `.click_id(id)`, `.records()`, `.snapshot()` (§16.4) |
| API layers | `junie_tui::*` vs `junie_tui::author::*` | both `pub`, separately documented |

Additional binding rules: no boolean parameter soup (typed enums for semantically different modes); no gratuitous generics in application-visible signatures (collection generics are inferred and die at the call site); no public mutable rect or cache; no `'static` bound in any component's public surface; complete rustdoc on every public item (`#![deny(missing_docs)]`).

<!-- amended by §21 items 1, 4, 19, 30, 33 -->

**Props are built once (binding).** A component instance with any configuration beyond `new(id, …)` is built by exactly one private constructor function on the owning screen, called from both phases. The constructor takes the fields it needs as parameters, never `&self`, so `update` can still pass `&mut` to disjoint fields; a controlled `.value(&T)` added in `draw` is the documented per-phase difference. `architecture::props_are_built_once` (a `syn` check that no configured `X::new(` appears more than once per screen module for the same `const Id`) reports violations. `Form` (J2) provides `Form::field(id, …)` so a 15-field form declares each field once and `Form` drives both phases. Without this, a `disabled(…)` predicate applied in `draw` but forgotten in `update` is a silent bug the compiler cannot see.

### 13.2 Per-component rustdoc template (goal §10, M31)

Every public component's type-level rustdoc answers goal §10's 17 questions under exactly these headings, in this order; `architecture::every_component_doc_has_the_standard_sections` (rustdoc-json heading scan) fails on a missing or misspelt heading:

```text
## Construction    — `X::new(id, …)`; alternate constructors and when they apply
## Ownership       — what the caller owns (`XState`, controlled values, items/model) and what the runtime owns
## Configuration   — the consuming builders, with defaults
## Variants        — the `Variant`s this family defines and `Recipe.default_variant`
## States          — which `StateFlags` the component can wear and which it derives itself
## Actions         — the `XAction` enum, one line per variant, with the `ItemKey` it carries
## Focus           — `Focusability`, `swallows_typing`, scope/trap behaviour, `autofocus`
## Keyboard        — the `Bindings::Cmd` table per `BindingState`
## Mouse           — the `PartRef`s that deliver `Pointer`/`Wheel` intents and what each does
## Layout          — the `measure` contract, degenerate-rect behaviour (R5), what `draw` returns
## Parts           — `X::PARTS` with one line each
## Overrides       — `.patch`, `.patch_part`, `.slot` support and any part that cannot be replaced
## Identity        — how items are keyed; the `ByIndex` caveat where applicable
## Testing         — the `Conformance` case name and `Caps`; the `render::components::<x>` states
## Invariants      — component-specific invariants beyond §4–§5
```

### 13.1 Keymaps, bindings and hints (confirms INT B9)

```rust
// <!-- amended by §21 items 10, 22, 30 -->
pub struct Binding<C: 'static> { pub chord: Chord, pub cmd: C,
                                 pub label: &'static str, pub priority: u8, pub visible: bool }
/// `Cmd` is the const-constructible command (Next, Prev, Activate, Close…); `update` maps it to
/// the emitted `XAction` with the live key, because `ListAction::Chose(ItemKey)` cannot be `const`.
pub trait Bindings { type Cmd: Copy + 'static; fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>]; }
pub struct KeyMap { /* add / remove / remap, per KeyPhase; EMPTY / EMPTY_REF in §17.0 A1 */ }
pub enum KeyPhase { Capture, Bubble }                       // was `Phase2`
pub struct HintLayer { pub hints: SmallVec<[Hint; 8]>, pub badge: Option<&'static str>,
                       pub status: Option<Cow<'static, str>>, pub centered: bool }   // 0 allocs/frame when focus is unchanged (P1)
```

Components declare their bindings from small `const` tables selected by state (no per-frame allocation). The runtime resolves a key against the focused component's table after applying the app's `KeyMap` override layer, so no application chord is baked into a generic component: `Dialog`'s `y`/`n` becomes an opt-in binding set; `Picker`'s `Delete`-secondary gains a mouse equivalent; the grid's `p`/`u`/`U`/`Ctrl+]`/`Ctrl+S` move to TablePro's `KeyMap`. "App chords always win" is replaced by two explicit phases, with `Capture` skipped for bare-`Char` chords while the focused control `swallows_typing` — generalising the six ad-hoc `!editing` guards, and making the known-dead grid `Ctrl+D` *detectable* (`conformance::conflicting_visible_bindings_are_reported`) rather than merely documented.

Hints are **derived**: `HintBar` composes top layer ▸ temporary mode ▸ focused component's visible bindings by priority ▸ screen extras ▸ global fallback. The composed layer is cached in `Ui::cache` behind `(focus_id, StateFlags, top_layer)`, so an unchanged focus costs zero allocations per frame (`frame_hintbar_derived`, §21 item 30). The ~700 lines of hand-written hint tables across the apps collapse to product-level extras only. `EditAction::Apply(fn(&mut TextBuffer))` and `TextInput.validator: Option<fn…>` are deleted in favour of a `Binding` table over `EditAction` and the `Validate` trait.

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
| J2 | Form dialog / field group | **Library component** — `Form` (ordered fields, visibility, scroll-to-focused-field, action row, error row, nested popup) + `Field<C>`. Three independent form engines collapse to one. `Form::field(id, …)` declares each field once and `Form` drives both phases; its API sketch is the open research item of §21. |
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

<!-- amended by §21 items 7, 30 -->

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
    pub fn write_mask(&self, out: &mut CellUi<'_>, n: usize); // SYNTHETIC tail, never the real one; writes cells, no String (P5)
    pub fn zeroize(&mut self);
}
impl fmt::Debug   for Secret { /* "Secret([redacted])" */ }
impl fmt::Display for Secret { /* "[redacted]" */ }
// Secret is NOT Clone, NOT PartialEq, NOT Serialize.

pub struct SecretPolicy { pub mask: GlyphRole, pub synthetic_tail: usize }

// The field wrapper owns all chrome, once. It is draw-time chrome ONLY: it has no Id, never
// registers a focus stop and never runs `update`; the control keeps its own Id and its own
// `update`, and the chrome's parts are registered as Decorative regions under the control's id.
// (§21 item 7, B10)
pub trait FieldControl {
    type State;
    fn id(&self) -> Id;
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl FieldControl for TextInput<'_> { type State = TextInputState; /* … */ }   // also TextArea, Select, Checkbox, Toggle, RadioGroup
pub struct Field<'a, C: FieldControl> { label: Option<&'a str>, required: bool,
                                        optional_suffix: bool, help: Option<&'a str>,
                                        error: Option<&'a str>, plain: bool, control: C }
impl<'a, C: FieldControl> Field<'a, C> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL,
                                         Part::MARKER, Part::FIELD, Part::HELP];
    pub fn new(label: &'a str, control: C) -> Self;                // no Id — the control owns identity
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;   // design.size.field_height
}
```

**Decisions.** `Field<C>` owns label (`*` required, `optional` suffix), help/error row, gutter and height (never focus registration, which stays with the control — §21 item 7) — deleting the per-control re-implementations and the `plain_label` flag, and deleting `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic from three apps. Values are **controlled** (`&mut String`), so the "rebuild the widget to change its value" idiom (five sites) disappears. Blur is an explicit intent-driven transition with a per-control policy (`CommitAndValidate` for `TextInput`, `Commit` for `TextArea`/`CodeEditor`, `Cancel` where a dialog demands it) — removing all five render-time commits. `RadioGroup` separates cursor from value. Masked fields render a synthetic tail, never the real characters (the safety property moves from jackin into the library, closing **[F]** API §5 item 13). Manual `Debug` impls redact on `TextInput`, `TextInputState`, `Field`, `Dialog`, `Form`, `EditState`, `TextEditorCore`; `conformance::secret_never_appears_in_debug` asserts it. `TextEditorCore::zeroize` overwrites before drop.

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

<!-- amended by §21 items 3, 4, 12, 14, 15, 29, 30, 33 -->

One `#[cfg(test)] mod tests` per module. Names are given verbatim; the module path is the test path.

**`id.rs`** — identity, goal §25.1 "stable identity"
`root_sub_part_index_item_are_all_distinct`, `separator_prevents_concatenation_collision` (asserts `Id::root("a").sub("b") != Id::root("ab")` **and** `Id::root("ab").sub("") != Id::root("a").sub("b")`), `kind_tag_separates_name_from_item_with_equal_bytes`, `id_equality_ignores_debug_label`, `id_is_const_constructible`, `item_key_text_is_stable_across_runs`, `item_key_pair_is_order_sensitive`, `part_custom_lands_in_the_high_range`, `part_constants_are_unique`, `debug_prints_path_in_debug_builds`, `debug_prints_hash_in_release_builds`.

**`response.rs`** — event consumption and invalidation
`ignored_consumed_changed_action_constructors`, `bitor_takes_consumed_over_ignored`, `bitor_takes_max_invalidate`, `bitor_is_defined_only_for_unit` (compile-fail via `trybuild`, §21 item 4), `repaint_raises_relayout_raises_further`, `layout_is_strictly_greater_than_paint`, `no_repaint_lowers_to_none`, `map_action_preserves_flow_and_invalidate`, `erase_drops_the_action_only`, `must_use_is_enforced` (compile-fail via `trybuild`), `state_flags_round_trip`.

**`intent.rs` / `event.rs`**
`key_release_is_dropped`, `unmapped_mouse_button_is_dropped`, `mouse_carries_modifiers`, `chord_hashes_by_code_and_mods`, `secondary_up_is_modelled`, `wheel_carries_axis_and_delta`, `paste_reaches_only_an_editing_owner`.

**`focus.rs`** — traversal, scopes, restoration, disabled/read-only
`tab_cycles_forward_and_backward`, `shift_tab_is_the_exact_reverse`, `disabled_entries_are_registered_but_skipped`, `read_only_entries_stay_in_the_ring`, `click_only_entries_are_never_reachable`, `trap_confines_traversal_to_the_scope`, `trap_wraps_inside_the_scope`, `nested_scopes_resolve_innermost_first`, `scope_restore_returns_focus_to_the_opener`, `reconcile_prefers_nearest_surviving_entry_by_previous_index`, `reconcile_falls_back_to_scope_first_enabled`, `reconcile_falls_back_to_innermost_active_scope`, `reconcile_yields_none_when_nothing_is_reachable`, `focus_visible_is_true_only_after_a_key`, `trap_is_armed_when_the_layer_is_pushed_not_when_it_draws`, `restore_target_receives_keys_before_the_next_draw` (§21 item 15).

**`hit.rs`** — hit ordering, layers, scroll routing
`last_registration_wins`, `higher_layer_shadows_lower`, `hit_returns_a_lower_layer_region_for_the_outside_click_test` (§21 item 12), `inert_below_registers_nothing`, `hit_returns_part_ref_not_a_derived_id`, `hit_scroll_returns_the_innermost_handler_of_the_axis`, `hit_scroll_returns_a_region_at_zero_headroom`, `hit_scroll_skips_regions_that_do_not_handle_the_axis`, `duplicate_id_is_reported_as_a_diagnostic_not_a_panic`, `empty_rects_are_rejected`, `generation_bump_invalidates_stale_regions`.

**`capture.rs`** — drag capture
`capture_claims_and_rejects_a_second_claim`, `drag_and_release_go_to_the_capture_owner`, `local_is_computed_against_the_captured_area`, `pressed_stays_set_while_the_pointer_leaves`, `release_outside_the_captured_area_does_not_activate`, `capture_is_released_on_resize`, `capture_is_released_when_the_owner_disappears`, `capture_is_released_on_generation_mismatch`.

**`scroll.rs`** — nested scrolling, boundary rule
`clamps_offset_to_content`, `ensure_visible_moves_minimally`, `thumb_covers_track_proportionally`, `track_position_round_trips`, `wheel_at_the_boundary_is_consumed_without_repaint`, `ensure_visible_on_next_layout_is_set_only_by_cursor_motion`, `fields_are_private_and_every_mutator_clamps`.

**`layer.rs`** — overlay stacking
`push_and_pop_maintain_layer_order`, `modal_pushes_a_trap_and_a_pointer_barrier`, `popover_pushes_a_pointer_barrier_only`, `esc_dismisses_only_the_top_layer`, `esc_reaches_the_focused_editor_before_the_layer` (§21 item 3), `layer_id_is_assigned_at_open_not_at_draw` (§21 item 14), `outside_click_is_layer_less_than_top_or_none`, `nested_layers_each_trap` (Scenario F), `anchor_rect_flips_then_clamps`, `anchor_screen_center_sits_in_the_upper_third`, `min_size_then_clamp_then_documented_degradation`, `closed_with_action_key_emits_layer_event_closed`, `dismissed_emits_the_reason`, `backdrop_excludes_the_footer_row`.

**`cursor.rs`**
`cursor_write_is_kept_for_the_focused_owner_on_the_top_layer`, `cursor_write_from_a_lower_layer_is_rejected`, `cursor_write_from_an_unfocused_owner_is_rejected`, `rejection_records_a_diagnostic`.

**`layout.rs` / `measure.rs`**
`rows_distributes_flex_after_fixed`, `columns_respects_gap_and_rounds_deterministically`, `responsive_columns_stack_below_the_threshold`, `action_row_right_aligns_and_left_aligns`, `inset_saturates_on_tiny_rects`, `split_first_pane_wins_on_both_axes_when_minima_do_not_fit`, `split_percent_is_clamped_to_5_95`, `measure_reports_min_and_preferred`.

**`text/` (buffer, editor, measure, fuzzy)**
`insert_and_move_by_grapheme`, `selection_replaces_on_insert`, `word_motion_and_deletion`, `word_chars_are_consistent_between_buffer_and_viewport`, `multiline_vertical_motion_keeps_column`, `single_line_rejects_newline`, `wide_characters_count_as_two_columns`, `combining_marks_are_one_grapheme`, `zwj_emoji_is_one_grapheme`, `pos_of_and_offset_at_round_trip`, `fuzzy_returns_grapheme_indices_into_the_original_label`, `fuzzy_ranks_prefix_before_boundary_before_substring_before_subsequence`, `editor_apply_is_the_only_mutation_entry_point`, `zeroize_overwrites_before_drop`, `row_ui_matches_fit_for_every_fixture` (differential against the `crates/tui/tests/fixtures/text.rs` corpus, §21 item 29).

**`theme/` (tokens, patch, recipe, resolve, downgrade)**
`slot_over_prefers_the_speaking_side`, `patch_merge_identity`, `patch_merge_absorption`, `patch_merge_is_associative`, `patch_clear_resolves_to_inherited_surface_fg`, `modifier_add_then_remove_is_symmetric`, `state_rules_are_stored_in_specificity_order` (R2 invariant), `state_rules_tie_break_by_declaration_order`, `state_rule_matches_only_when_when_is_a_subset`, `precedence_family_then_variant_then_state_then_global_then_scope_then_instance`, `roles_bind_after_the_whole_chain`, `raise_is_ladder_index_arithmetic_not_colour_equality`, `raise_saturates_at_the_last_level`, `field_raises_to_field_hover`, `downgrade_maps_every_token_exhaustively`, `downgrade_works_for_a_user_supplied_theme`, `mono_appends_one_state_rule_per_family`, `paper_theme_inverts_the_plane_direction`, `custom_family_and_variant_round_trip`, `theme_is_byte_identical_after_a_scoped_render`, `builder_derives_every_unset_token_deterministically`, `derived_tokens_meet_design_contrast_ratios`, `downgrade_is_deterministic_per_level`, `paper_tokens_are_pinned` (§21 item 29).

**`collection/` (key, reconcile, rowui, decor, empty)**
`reconcile_keeps_a_surviving_key`, `reconcile_takes_the_nearest_forward_then_backward`, `reconcile_falls_back_to_the_first_enabled_key`, `reconcile_yields_cursor_lost_when_empty`, `reconcile_drops_vanished_checked_keys_and_reports_the_count`, `reconcile_clamps_the_scroll_offset`, `reconcile_runs_before_any_action_is_emitted`, `generation_stamp_skips_a_no_op_reconcile` (R1), `cached_index_probe_hits_before_a_scan` (R1), `row_ui_label_writes_cells_without_an_intermediate_string` (R5), `row_ui_meta_is_dropped_all_or_none`, `row_ui_columns_clip_to_the_row`, `empty_state_covers_empty_loading_partial_error`.

**Component state machines** (`components/*.rs`, buffer-free, no terminal) — goal §25.1 "edit begin, commit, cancel, focus loss":
`input::begin_snapshots_the_value`, `input::commit_writes_the_controlled_value`, `input::commit_runs_validation_once`, `input::cancel_restores_the_snapshot`, `input::blur_commit_and_validate_policy`, `input::blur_cancel_policy`, `input::blur_keep_policy_leaves_the_draft`, `input::external_error_survives_a_redraw`, `input::write_mask_is_synthetic` (renamed, P5), `textarea::blur_commits_without_validation`, `select::escape_closes_and_restores_the_cursor`, `select::arrows_move_the_cursor_not_the_value_while_closed`, `choice::radio_group_separates_cursor_from_value`, `list::select_all_selects_only_enabled_items`, `list::range_selection_uses_the_anchor`, `tree::expand_collapse_is_keyed_not_positional`, `tree::lazy_children_do_not_reflatten_the_world`, `tabs::close_targets_the_logical_tab_after_a_reorder`, `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row`, `grid::edit_intent_inline_cycle_external_refuse`, `grid::range_copy_is_tsv`, `grid::click_inside_an_active_inline_edit_goes_to_the_editor` (§21 item 30), `dialog::action_arming_is_evaluated_in_update`, `dialog::convenience_constructors_render_through_the_body_slot` (§21 item 33), `picker::query_change_emits_query_changed`, `wizard::rewind_retains_per_step_state`, `viewport::retention_fixes_up_selection_and_caret`, `code::edit_counter_invalidates_the_highlight_cache`, `secret::debug_and_display_redact`, `secret::is_not_clone_not_eq` (compile-fail via `trybuild`).

---

### 16.2 Shared conformance suite (goal §25.2)

<!-- amended by §21 items 10, 11, 15, 25, 27 -->

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
        const TYPES       = 1 << 10; // focus entry sets swallows_typing; bare Char chords are exempt from case 20 (§21 item 27)
    }
}

/// One registration per public component. `State = ()` for stateless components.
pub trait Conformance: 'static {
    const NAME: &'static str;                 // "button", "list", …
    const FAMILY: Family;
    const PARTS: &'static [Part];
    type State: Default + Clone + PartialEq + core::fmt::Debug;
    type Action: PartialEq + core::fmt::Debug;     // cases 2 and 12 compare actions structurally (§21 item 27)
    type Cmd: Copy + 'static;                       // §21 item 10

    fn caps() -> Caps;
    fn id() -> Id;
    /// Fixture knobs the driver varies: disabled, read_only, item count, theme, colour level.
    fn update(cx: &mut Cx<'_>, st: &mut Self::State, f: &Fixture) -> Response<Self::Action>;
    fn draw(ui: &mut Ui<'_>, area: Rect, st: &Self::State, f: &Fixture);

    // capability-gated hooks; the default panics only if the matching cap is set
    fn activation_chords() -> &'static [Chord] { &[] }             // ACTIVATES
    fn activation_part() -> PartRef { PartRef::of(Part::CONTAINER) }
    fn bindings(s: BindingState) -> &'static [Binding<Self::Cmd>] { &[] }
    fn item_keys(f: &Fixture) -> Vec<ItemKey> { Vec::new() }        // COLLECTION
    fn reorder(f: &mut Fixture, perm: &[usize]) {}                  // COLLECTION
    fn action_key_of(a: &Self::Action) -> Option<ItemKey> { None }  // COLLECTION
    fn secret_bytes() -> &'static str { "" }                        // SECRET
}

pub struct Fixture {                                  // §21 item 27: real rows, not a count
    pub disabled: bool, pub read_only: bool,
    pub theme: Theme, pub color: ColorLevel, pub area: Rect,
    pub rows: Vec<FixtureRow>,                        // `update`/`draw` borrow from here; `reorder` permutes it
}
pub struct FixtureRow { pub key: ItemKey, pub label: String, pub meta: String, pub disabled: bool }

#[macro_export] macro_rules! conformance_suite { ($($case:ty),+ $(,)?) => { … } }
```

`crates/tui/tests/conformance.rs` ends with one invocation listing **every** public component; `architecture::conformance_covers_every_public_component` (§16.5) cross-checks that list against the `pub` component inventory, so adding a component without registering it fails CI.

```rust
conformance_suite!(
    ButtonCase, ChipCase, CheckboxCase, RadioGroupCase, ToggleCase, BrandCase, KeyHintCase,
    FieldCase, TextInputCase, TextAreaCase, SelectCase,
    ListCase, NavListCase, TreeCase, PropsCase, PropsListCase, StepsCase, GridCase, TabsCase, ChipBarCase,
    PanelCase, SplitPaneCase, ScrollRegionCase, TextViewportCase, DiffViewCase, CodeEditorCase,
    DialogCase, MenuBarCase, ContextMenuCase, PickerCase, FilterListCase, CompletionCase,
    FormCase, WizardCase, PickerChainCase, HelpOverlayCase,
    ProgressBarCase, SpinnerCase, MeterCase, StatusBarCase, HintBarCase, TooSmallCase,
);
// ScrollbarCase (a part), EmptyCase (a data enum) and SecretInputCase (no such type) are removed; the
// secret path is a Caps::SECRET fixture variant of TextInputCase (§21 item 27).
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
| 9 | `mono_states_are_distinguishable` | no-colour output retains state indicators | Under `ColorLevel::Mono`, the `(symbol, modifier)` multiset differs pairwise between default / focused / selected / pressed / disabled / error / warning / editing / busy / active, colour excluded; `pressed` is distinguishable through the `BOLD` + bracket-glyph rule of §11.4 (§21 item 25) |
| 10 | `local_override_does_not_mutate_the_theme` | local overrides do not mutate the theme | Hash the `Theme` before, render inside `ui.with_overlay(&OV, …)` and with `.patch_part(…)`, hash after: equal; the overridden part's `Resolved` differs while the un-overridden sibling's does not |
| 11 | `id_separator_collision_free` | (added) | Every id and `PartRef` this component registers is unique within a frame; no two differ only by concatenation (`Diagnostic::DuplicateId` count is 0) |
| 12 | `item_identity_survives_reorder` *(COLLECTION)* | (added, Scenario E) | Set cursor/selection/checked on keys `k₁,k₂`; apply a reverse permutation and an insert+remove; after `reconcile`, cursor and checked set still name `k₁,k₂`; a click on the row now showing `k₁` emits an action carrying `k₁` |
| 13 | `focus_reconcile_follows_the_rule` *(FOCUSABLE)* | (added) | Remove the focused entry: focus lands on the nearest surviving entry by previous index; if the scope empties, on the scope's first enabled; then on the innermost active scope's first; then `None` — all four branches exercised |
| 14 | `focus_trap_and_restore` *(OVERLAY)* | (added) | Opening the layer shrinks `reachable()` to the layer's own stops; Tab wraps inside; closing restores focus to the opener; a layer that cannot draw (0×0) still traps |
| 15 | `pointer_capture_delivers_drag_and_release` *(CAPTURES)* | (added) | After a press claims capture, drags outside the component still reach it with `local` relative to the captured area; a second claim is refused; release outside the captured area does not activate |
| 16 | `wheel_at_boundary_is_consumed_without_repaint` *(SCROLLS)* | (added) | At offset 0 a wheel-up returns `Flow::Consumed` with `Invalidate::None`; mid-range returns `Consumed` + `Paint`; the event never chains to an outer scrollable and never moves focus or the cursor |
| 17 | `cursor_write_is_rejected_off_top_layer` *(CURSOR)* | (added) | Drawn under an open `Popover` (pointer barrier only, no `inert_below` — an inert layer registers nothing and is discarded silently, §21 item 15), the component's `ui.set_cursor` is dropped and one `Diagnostic::CursorRejected` is recorded; on the top layer with focus, it is kept |
| 18 | `secret_never_appears_in_debug` *(SECRET)* | (added) | `format!("{:?}")` of props, state, and any owning container (`Field`, `Dialog`, `Form`) contains neither `secret_bytes()` nor its snapshot; the rendered buffer contains neither; the `TestBackend` digest is unchanged when only the secret changes |
| 19 | `survives_tiny_rects_0x0_to_3x3` | (added) | For every `w,h ∈ 0..=3`: `draw` does not panic in a debug build, writes no cell outside the rect, registers no region outside it, and leaves no stale geometry (a click after a 0×0 frame resolves to `None`, never to last frame's rect) |
| 20 | `bindings_match_handled_keys` | (added) | Every chord in `bindings(state)` is consumed by `update` in that state, and every chord `update` consumes in that state appears in `bindings(state)` — the table and the handler cannot drift. For components declaring `Caps::TYPES` the reverse direction exempts bare `Char` chords (§21 item 27) |

Suite-level tests (emitted once, not per component), in `conformance.rs`:

* `conformance::registry::every_public_component_is_registered`
* `conformance::registry::declared_parts_are_the_parts_actually_styled` — the parts a component resolves at draw time equal `Self::PARTS`
* `conformance::conflicting_visible_bindings_are_reported` — two visible bindings on the same chord in the same phase produce a `Diagnostic::BindingConflict` (this is what makes the historically dead grid `Ctrl+D` detectable)
* `conformance::focus_transition_settles` — suite-level only (a whole-app property, §21 item 11): a scripted journey over every registered component records zero `Diagnostic::FocusTransitionDidNotSettle`
* `conformance::draw_registers_nothing_when_it_cannot_draw` — the 0×0 case across the whole registry

---

### 16.3 Rendering and snapshot tests (goal §25.3)

**Mechanism.** `TestBackend` + an FNV‑1a digest of `(symbol, fg, bg, modifier)` per cell — the existing `showcase_visual_baseline` mechanism (**[F]** APP §6, `app_tests.rs:623-668`) generalised. Digests, not golden images: a digest fails fast and the reviewer regenerates the *image* with `tools/capture.sh` to look at the difference.

```rust
// crates/tui-testing/src/digest.rs
pub struct Scene { /* theme, color level, size, name, headless FrameState: registry + ring + layers + style stack (§21 item 28) */ }
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

**Regeneration and review policy.** `BLESS=1 cargo test --workspace --test render --test visual` rewrites baselines. The rule from goal §6.10 is enforced mechanically: `xtask bless-guard` fails a commit that changes a baseline file without a matching entry in `docs/visual-changes.md` referencing a §20.10 item and a capture path under `shots/`. No baseline is regenerated because a test failed; the classification comes first. The order is fixed (A14, §21 item 30): **change → capture → classify → bless** — the capture cannot exist before the change, so `bless-guard` runs in CI on the committed tree, not locally, and the `docs/visual-changes.md` entry is written between the capture and the bless. <!-- amended by §21 items 28, 30 -->

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

**All current tests are retained**: 26 showcase, 21 tablepro, 22 jackin (17 + 5 chrome) plus the in-module `rain`/`arbiter`/`clock`/`scenario` unit tests — the exact inventories in **[F]** APP §5.1–§5.3. They move from `#[cfg(test)] mod app_tests` inside each binary to `apps/<app>/tests/app_tests.rs` (integration tests linking the app's `[lib]` target — §21 item 23), which forces each app to expose a small, deliberate test surface instead of reaching into private fields (goal §21).

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

    // §21 items 19, 24 (M13, M14): the app and its resolved styles, redraw suppression, recorded tags
    pub fn app(&self) -> &A;
    pub fn app_mut(&mut self) -> &mut A;
    pub fn resolved(&self, id: Id, p: Part) -> Resolved;
    pub fn with_auto_draw(self, yes: bool) -> Self;                      // false: `handle` does not draw; call `draw()`
    pub fn last_invalidate(&self) -> Invalidate;
    pub fn records(&self) -> &[&'static str];                            // `Cx::record` tags; replaces `actions()`
}
// `Harness::new` draws, and `handle` draws after every input, so a test never observes the one-frame
// pointer latency of §20.1; `click_id` on an id whose `area_of` is `None` returns `Response::ignored()`
// and records `Diagnostic::UnaddressableId`, never panics (F7, §21 item 17).
```

Mapping of the seven "must keep working" facts from **[F]** APP §6:

1. `handle(Input) -> Response<()>` with `is_changed()` meaning "redraw needed" <!-- amended by §21 item 4 -->. The three `Outcome` cases map as: `Outcome::Changed → assert!(r.is_changed())`; `Outcome::Consumed → assert!(r.is_consumed() && !r.is_changed())`; `Outcome::Ignored → assert!(!r.is_consumed())`. The ~60 assertions of the form `assert_eq!(h.key(…), Outcome::Changed)` become `assert!(h.key(…).is_changed())`. `update` runs inside `handle`, so the answer stays truthful (this is the §3.2 argument against `show`, discharged here).
2. Synchronous deterministic draw after every input — `Harness::handle` draws before returning.
3. Stable test-visible addressing — app `const Id`s are `pub` in the app's **library** target (`showcase_app`, `tablepro_app`, `jackin_app`; §21 item 23) and the tests `use showcase_app::…`. `FORM.sub("save")` becomes `FORM.part(Part::custom("save"))` — a child *component* id looked up with `area_of` — while a component's own sub-region is `area_of_part(FORM, PartRef::of(Part::custom("save")))`; the two are different lookups against different registries and are never interchangeable (§21 item 16) <!-- amended by §21 items 16, 23 -->; `WidgetId::of("editor.cfg").sub("form").sub("save")` becomes `screens::editor::CFG_FORM.part(SAVE)`.
4. Resolved geometry read-back — `area_of` / `area_of_part`.
5. Reachable focus ring — `ring().reachable()`.
6. Virtual-clock tick injection — `ticks(n)`; jackin's `Clock` keeps its no-wall-clock contract.
7. Exact minimum-size copy strings — the shared `TooSmall` component keeps `"Terminal too small"` and `"Need {w}×{h}, have {w}×{h}"` verbatim; `showcase::below_minimum_size_shows_reduced_state` and its two siblings are unchanged.

**Theme coupling in tests** (`focus_bar_x` compares against `Theme::junie().focus`, **[F]** APP §6) becomes `h.resolved(id, Part::GUTTER).style.fg`, so the assertion survives a theme change and also runs under `Theme::paper()`.

**New application coverage** required by goal §25.4:

`showcase::complete_navigation_visits_every_page_and_every_state`, `showcase::custom_theme_injection_repaints_every_page` (`--theme paper`), `showcase::local_override_page_shows_three_distinct_buttons`, `showcase::author_component_page_participates_in_focus_and_hover`, `tablepro::mouse_flow_full_journey` and `tablepro::keyboard_flow_full_journey` (retained, renamed from `acceptance_flow_*`), `tablepro::grid_adapter_keeps_every_pending_change_capability`, `jackin::complete_flow_keyboard_first` (retained), `jackin::nested_overlay_picker_inside_dialog`, `*::resize_across_every_supported_size`, `*::focus_is_restored_after_every_overlay_closes`, `*::no_diagnostics_are_emitted_during_the_journey` (asserts zero `DuplicateId`, `CursorRejected`, `UndeliveredIntent`, `BindingConflict`, `FocusTransitionDidNotSettle`, `UnaddressableId`, `DuplicateLayerDraw` in a full run — `UndeliveredIntent` is zero because container regions are `Decorative`, §21 item 13 — a strong, cheap regression net).

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
| `architecture::every_named_test_exists` | one-directional and scoped (§21 item 28): every name listed in §16.1, §16.2's suite-level list and §16.4 exists in `cargo test --workspace -- --list`; §16.6 perf names are checked against `cargo test --workspace --test perf --release -- --list`; `trybuild` cases against `tests/ui/*.rs` filenames; extra tests are allowed | Documentation and the suite cannot drift; the `capsule_pane_clone_4x2000` deletion is asserted by line-absence in `perf_baseline.txt` |
| `architecture::binary_names_are_preserved` | `cargo metadata` target names | `showcase`, `tablepro`, `jackin-preview` (goal §21) |
| `architecture::msrv_and_edition_are_unchanged` | `cargo metadata` | edition 2024, `rust-version = "1.88"` on every package |
| `architecture::cache_types_are_derived_only` <!-- §21 item 2 --> | `syn` scan in `xtask`: every `T` used as `ui.cache::<T>(…)` | `T` appears in no `Response` and no `XState` (R8) |
| `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library` <!-- §21 item 23 --> | `cargo metadata` | `showcase_app`, `tablepro_app`, `jackin_app` have `publish = false` and are absent from the library's dependency closure |
| `architecture::props_are_built_once` <!-- §21 item 30 --> | `syn` scan over `apps/**/src` and `crates/tui/examples/**` | no configured `X::new(` for the same `const Id` appears more than once per screen module (§13) |
| `architecture::state_override_is_used_only_in_apps_and_fixtures` <!-- §21 item 30 --> | grep for `.state_override(` | only under `apps/**`, `crates/tui/tests/fixtures/**` and `crates/tui-testing/**` |
| `architecture::every_component_doc_has_the_standard_sections` <!-- §21 item 33 --> | rustdoc-json heading scan | every public component's docs carry the 15 §13.2 headings in order |
| `architecture::no_todo_or_unimplemented` <!-- §21 item 33 --> | grep for `todo!`, `unimplemented!`, `TODO`, `FIXME` over `crates/**` and `apps/**`, empty allow-list | goal §29 "no material TODO, stub, placeholder" |
| `architecture::showcase_covers_every_public_component` <!-- §21 item 33 --> | cross-check the `conformance_suite!` list against the showcase page registry | goal §29 "the showcase demonstrates every public component" |
| `xtask doc-check` <!-- §21 item 34 --> | extracts every `Ident::method` reference and every Rust code block from `COMPONENT_ARCHITECTURE.md` §3–§17 and §21 and resolves each against the library's rustdoc-json | every reference resolves, or is on the printed "not yet built (Slice 3/4)" allow-list; run in every slice gate |

---

### 16.6 Performance (goal §25.6)

**The measurement plan of `docs/audit/performance-audit.md` §7 is adopted verbatim** — harness, assertion policy, baseline file format, CI wiring, and every test name. Nothing in it is renamed. Restated obligations:

* Harness in `crates/tui-testing/src/perf.rs` (`Counting` global allocator shim, `ALLOCS`/`BYTES`, `bench`, `Stats`, `report`); `#[global_allocator]` declared **only** in `crates/tui/tests/perf.rs` and `apps/*/tests/perf.rs`. WP‑0 landed the harness at root `tests/perf_common.rs` + `tests/perf.rs` (commit `07cb2c9`); Slice 3 moves it (Appendix A, §21 item 31).
* Allocation and byte counts are deterministic → **hard assertions**. Wall time is reported always, asserted only under `PERF_STRICT=1` against `baseline × 1.2`.
* Baseline `crates/tui/tests/perf_baseline.txt`, one `name ns allocs bytes` line, regenerated only with `PERF_BLESS=1`, reviewed in the diff.
* **The baseline is recorded on the pre-refactor tree**, on a `perf/baseline` commit, before Slice 3 begins (Appendix A, WP‑0). This is a hard sequencing constraint: without it "before and after" is not literal.
* `--test-threads=1` is mandatory; the counters are process-wide.
* Every screen benchmark also reports `hits=<registry len>` and `ring=<reachable len>`.

**Before → after thresholds.** "Before" is the recorded pre-refactor baseline; "after" is the assertion that must hold at the end of Slice 8.

| Test (perf §7) | Before | After (asserted) |
|---|---|---|
| `frame_showcase_lists_120x40` | ≈ 160 allocs/frame, 57 hits, 4 ring | **< 20 allocs/frame**; hits recorded and classified in `docs/visual-changes.md`, no unexplained growth > 25 % (P8, §21 item 30); ring ≥ 4 |
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
| `viewport_100k_lines_render` | 15 177 816 allocs/frame (baseline line 39: the whole buffer is laid out twice per frame, P-A) | allocs/frame independent of buffer size — **the binding acceptance for §20.9-7** |
| `capsule_pane_clone_4x2000` | dominant cost | **the test is deleted**; its deletion is asserted by line-absence in `perf_baseline.txt` (§21 item 28) |
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
| `debug_and_release_alloc_counts_match` | — | equal **±1 allocation** (R4; P-B in §20.9: the two profiles differ by one optimizer-elided 3-byte allocation, so exact equality is unattainable and ≤ 1 is the decided tolerance) |

**Four additions to perf §7** <!-- amended by §21 items 6, 30 -->, needed because §7 has no coverage for obligations §20.9 makes binding. They are additions, not renames, and are marked as such in `perf_baseline.txt`:

| Added test | Location | Threshold |
|---|---|---|
| `frame_tablepro_query_editor_2k_lines` | `apps/tablepro/tests/perf.rs::frames` | before ≈ 7 collections + O(graphemes × spans) per frame; after **< 40 allocs/frame** and ns scaling with *viewport*, not document length (§20.9 amendment 9) |
| `list_100k_select_all` | `crates/tui/tests/perf.rs::large` | `ToggledAll` must **not** materialise 100 000 `ItemKey`s: **< 100 allocs** (R7) |
| `intents_drain_is_o_1_when_the_queue_is_empty` (renamed, §21 item 6) | `crates/tui/tests/perf.rs::invariants` | a 500-component frame with 0 intents costs the same as a 20-component frame with 0 intents ±10 %; with 2 intents, total probe cost is ≤ 500 × 5 ns and allocations are 0 (R6, B14) |
| `frame_hintbar_derived` (§21 item 30, P1) | `crates/tui/tests/perf.rs::frames` | **0 allocs/frame** when focus is unchanged; the derived `HintLayer` is cached in `Ui::cache` behind `(focus_id, StateFlags, top_layer)` |

**CI wiring** (perf §7.3, adopted verbatim): one always-on job `cargo test --workspace --test perf --release` (allocation counts only) and one pinned-runner job `PERF_STRICT=1 cargo test --workspace --test perf --release -- --test-threads=1`. `PERF` lines are collected into a build artefact for the final report (goal §30 item 13).

---

## 17. Representative usage examples

Twelve examples, one file each under `crates/tui/examples/`, built by `cargo build -p junie-tui --examples` (`-p tui-next` during Slices 3–4) and gated by `architecture::all_examples_compile`. Every file is complete — a `main` or a `#[test]`, every `use` list exact — because Slice 2 acceptance condition 1 compiles them verbatim. They use only the public facade, so they are literal proof of the "external consumer" claim. Examples 1–10 are also condensed into rustdoc doctests on the corresponding types (`cargo test --workspace --doc`).

### 17.0 API additions

<!-- amended by §21 items 1, 2, 4–10, 13, 17–22, 24, 30, 32 -->

Everything §17 needs that §1–§15 did not spell out. These are additions to the accepted architecture; each is consistent with an existing rule and is listed here so no example invents a name. This block is the surface the examples compile against and the surface `xtask doc-check` (§21 item 34) resolves the document against.

```rust
// ---- A1. Application entry point (§3.3 named `Runtime<A>` and `app.update`/`app.draw`) ----
pub trait App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;
    fn draw(&self, ui: &mut Ui<'_>);
    fn should_quit(&self) -> bool { false }
    fn keymap(&self) -> &KeyMap { KeyMap::EMPTY_REF }               // §21 item 9
    fn min_size(&self) -> Size { Size { min: (72, 20), preferred: (120, 40) } }
}
// Domain messages (jackin's `Msg`, a tablepro `Request` reply) enter at the top of
// `App::update`, drained from the application's own queue before any screen `update`
// runs. `Input` deliberately has no `Msg` variant. (§21 item 32)
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
    pub fn diagnostics(&self) -> &[Diagnostic];   // ≤ 64 retained + a dropped count; cleared per `handle` (§21 item 30, P6)
    pub fn set_theme(&mut self, t: Theme);
}
/// Inspection for `Harness` (§16.4). Never compiled into a release binary. (§21 item 24)
#[cfg(feature = "testing")]
impl<A: App> Runtime<A> {
    pub fn hover(&self) -> Option<Id>;
    pub fn state_of(&self, id: Id) -> StateFlags;
    pub fn focus_visible(&self) -> bool;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
    pub fn cursor(&self) -> Option<Position>;
    pub fn resolved(&self, id: Id, p: Part) -> Resolved;
    pub fn last_invalidate(&self) -> Invalidate;
    pub fn records(&self) -> &[&'static str];                        // filled by `Cx::record`
}
/// Owns the terminal session (raw mode, alt screen, mouse, bracketed paste, panic hook).
pub fn run<A: App>(app: A, theme: Theme) -> std::io::Result<()>;
impl KeyMap {
    pub const EMPTY: KeyMap;
    pub const EMPTY_REF: &'static KeyMap = &KeyMap::EMPTY;          // §21 item 9
}

// ---- A2. Cx / Ui members (§4 S3, §8.2, §8.5, §9.1 name most; §21 items 2, 6, 18) ----
/// Shared read accessors — one vocabulary for both phases (review §3). Re-exported from `author`.
pub trait FrameRead {
    fn state(&self, id: Id) -> StateFlags;             // runtime-resolved focus/hover/press
    fn theme(&self) -> &Theme;
    fn design(&self) -> &DesignTokens;
    fn area(&self, id: Id) -> Option<Rect>;            // LAST frame's geometry, None on frame 1
    fn layout(&self, id: Id) -> Option<LayoutFacts>;
}
impl FrameRead for Cx<'_> { /* … */ }
impl FrameRead for Ui<'_> { /* … */ }

/// `'f` is the frame. The intent queue is built in §3.3 step 6 and frozen for the whole of
/// step 7; `Cx` holds it as `&'f IntentQueue` separately from its `&'f mut` services, so an
/// `IntentIter<'f>` never locks `Cx` and a component may call any `&mut self` service inside
/// its drain loop. (§21 item 6, B4)
pub struct Cx<'f> { /* intents: &'f IntentQueue, services: &'f mut FrameServices */ }
impl<'f> Cx<'f> {
    /// Borrows only the frozen queue; marks this owner's bucket drained through a per-bucket
    /// `Cell<bool>`. Returns an empty iterator after one `bool` check when the queue is empty
    /// and otherwise probes a `[u64; N]` open-addressed table of the ≤ 8 owners that hold
    /// intents (§20.9-12).
    pub fn intents(&self, id: Id) -> IntentIter<'f>;
    pub fn focus(&mut self, id: Id);                                  // stages a transition (§3.3 step 7)
    pub fn request_repaint(&mut self);
    pub fn request_repaint_after(&mut self, d: std::time::Duration);
    pub fn capture(&mut self, owner: Id, part: PartRef) -> bool;      // claim; false if another capture is live
    pub fn capture_owner(&self) -> Option<Id>;
    pub fn release_capture(&mut self);
    pub fn capture_origin(&self) -> Option<Position>;
    pub fn capture_area(&self) -> Option<Rect>;
    pub fn open_layer(&mut self, id: Id, spec: LayerSpec);            // assigns the LayerId (§21 item 14)
    pub fn close_layer(&mut self, id: Id, with: Option<ActionKey>);
    pub fn layer_event(&mut self, id: Id) -> Option<LayerEvent>;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
    pub fn quit(&mut self);
    #[cfg(feature = "testing")]
    pub fn record(&mut self, tag: &'static str);                      // replaces `Harness::actions` (§21 item 19)
}
pub struct IntentIter<'f> { /* … */ }
impl<'f> Iterator for IntentIter<'f> { type Item = Intent<'f>; /* … */ }

impl Ui<'_> {
    pub fn full(&self) -> Rect;                                       // current clip rect
    pub fn surface(&self) -> Surface;  pub fn bg(&self) -> Color;     // §10
    pub fn style(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved;
    pub fn with_area<R>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn with_overlay<R>(&mut self, ov: &Overlay, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn focus_scope<R>(&mut self, id: Id, mode: ScopeMode, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn register_control(&mut self, id: Id, area: Rect, f: Focusability);          // RegionKind::Control
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect);            // RegionKind::Part
    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect);           // RegionKind::Decorative (§21 item 13)
    pub fn register_scroll(&mut self, id: Id, area: Rect, axes: Axes, head: Headroom); // RegionKind::Scroll
    pub fn report_layout(&mut self, id: Id, l: LayoutFacts);
    pub fn set_cursor(&mut self, owner: Id, pos: Position);
    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>;
    pub fn dim_layer(&mut self, area: Rect, steps: u8);
    // painting (R3); every method clips to the current area and marks the layer's written-cell bitset
    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: Style);
    pub fn paint_str(&mut self, area: Rect, text: &str, s: Style) -> u16;   // columns written
    pub fn fill(&mut self, area: Rect, s: Style);
    pub fn rule(&mut self, area: Rect);                                     // GlyphRole::RuleQuiet across `area`
    pub fn frame(&mut self, area: Rect, s: Style) -> Rect;                  // theme BorderSet; returns the inner rect
    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: Style) -> u16;     // columns written
    pub fn raw(&mut self) -> (&mut Buffer, Rect);
    /// Derived, non-semantic per-component cache. Keyed by (Id, TypeId). Cleared on resize,
    /// theme change and generation gap. Never observable in `Response`, never compared by
    /// `draw_twice_leaves_state_equal`. Rule R8 (§5). (§21 item 2, B5)
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T;
    #[cfg(feature = "testing")]
    pub fn styled_parts(&self) -> &[(Id, Part)];                            // for `declared_parts_are_the_parts_actually_styled`
}
#[non_exhaustive]
pub struct LayoutFacts { pub viewport_len: usize, pub content_len: usize, pub rows: u16, pub cols: u16 }

// ---- A3. Per-phase data (§4 rule 4 and §21 item 1, made explicit) ----
// Controlled values, collection items and grid models are never held in the props struct that
// owns the phase methods; they are passed to the phase call, so the props never borrow a field
// the action closure needs to mutate:
//   fn update(&self, cx: &mut Cx<'_>, st: &mut TextInputState, value: &mut String) -> Response<TextAction>
//   fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect      // value via .value(&str)
//   fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction>
//   fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect
//   fn update<M: GridModel>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>
//   fn draw  <M: GridModel>(&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect

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

// ---- A5. Theme builder and recipe editor (§11.2 declared the entry points only; §21 items 21, 29) ----
pub struct ThemeBuilder { /* … */ }
impl ThemeBuilder {
    pub fn accent(self, c: Color) -> Self;      pub fn danger(self, c: Color) -> Self;
    pub fn warning(self, c: Color) -> Self;     pub fn success(self, c: Color) -> Self;
    pub fn info(self, c: Color) -> Self;        pub fn focus(self, c: Color) -> Self;
    pub fn selection(self, bg: Color, fg: Color) -> Self;
    pub fn highlight(self, bg: Color, fg: Color) -> Self;
    pub fn field(self, base: Color, hover: Color) -> Self;
    pub fn disabled(self, fg: Color, bg: Color) -> Self;
    pub fn surfaces(self, s: [Color; SURFACE_LEVELS]) -> Self;
    pub fn fg(self, f: [Color; FG_STEPS]) -> Self;
    pub fn borders(self, subtle: Color, strong: Color) -> Self;
    pub fn borders_set(self, b: BorderSet) -> Self;
    pub fn glyph(self, r: GlyphRole, s: &'static str) -> Self;
    pub fn space(self, s: SpaceTokens) -> Self; pub fn size(self, s: SizeTokens) -> Self;
    pub fn density(self, d: Density) -> Self;
    pub fn motion(self, m: MotionTokens) -> Self;
    /// Fills every token the caller did not set by the derivation written in §11.2
    /// (§21 item 29). Deterministic; pinned by `theme::builder_derives_every_unset_token_deterministically`.
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
impl SyntaxTokens { pub fn derive(keyword: Color, string: Color, number: Color) -> SyntaxTokens; }
impl MeterTokens  { pub fn derive(low: Color, medium: Color, high: Color) -> MeterTokens; }
impl BorderSet    { pub const ROUNDED: BorderSet; pub const SQUARE: BorderSet; pub const ASCII: BorderSet; }

// ---- A6. Layer construction (§9.1 gave the struct; §21 items 8, 20) ----
#[non_exhaustive]
pub struct LayerSpec { /* kind, owner, anchor, dismiss, restore_focus, initial_focus, min_size: (u16, u16), backdrop, inert_below */ }
impl LayerSpec {
    pub const fn modal(owner: Id) -> LayerSpec;                     // Modal, Screen(Center), esc+outside, dim, inert
    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec;   // Popover, pointer barrier only, no dim
    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec;
    pub const fn anchor(self, a: Anchor) -> Self;
    pub const fn dismiss(self, d: Dismiss) -> Self;
    pub const fn backdrop(self, b: Backdrop) -> Self;
    pub const fn initial_focus(self, id: Id) -> Self;
    pub const fn min_size(self, w: u16, h: u16) -> Self;
    pub const fn inert_below(self, yes: bool) -> Self;
    pub const fn restore_focus(self, yes: bool) -> Self;
}
impl Dismiss {
    pub const NONE: Dismiss;            // programmatic only
    pub const ESC: Dismiss;
    pub const ESC_AND_OUTSIDE: Dismiss;
    pub const ALL: Dismiss;             // esc + outside_click + focus_out (focus_out: Popover/Tooltip only, §9.1)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Backdrop { None, Dim { exclude_footer: bool } }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Side { Below, Above, Left, Right }        // Anchor::Rect
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum CrossAlign { Start, Center, End }

// ---- A7. Component constructors and builders used below (§13 fixes the conventions; §21 items 1, 5, 7) ----
impl<'a> Button<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL, Part::ICON];
    pub fn new(id: Id, label: &'a str) -> Self;
    pub fn variant(self, v: Variant) -> Self;      pub fn disabled(self, yes: bool) -> Self;
    pub fn icon(self, g: GlyphRole) -> Self;       pub fn autofocus(self) -> Self;
    pub fn status(self, s: Status) -> Self;
    pub fn patch(self, p: &'a StylePatch) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn slot(self, p: Part, f: &'a dyn Fn(&mut Ui<'_>, Rect)) -> Self;
    /// Showcase / fixture use only (A11): render in a forced state without owning it.
    /// `architecture::state_override_is_used_only_in_apps_and_fixtures` guards it.
    pub fn state_override(self, s: StateFlags) -> Self;
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
    pub fn status(self, s: Status) -> Self;      pub fn state_override(self, s: StateFlags) -> Self;
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TextInputState, value: &mut String)
        -> Response<TextAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TextInputState) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}

/// Draw-time chrome only. `Field` never registers a focus stop and never runs `update`;
/// the control keeps its own `Id` and its own `update`. (§21 item 7, B10)
pub trait FieldControl {
    type State;
    fn id(&self) -> Id;                                              // the chrome's parts are registered under this id
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl FieldControl for TextInput<'_> { type State = TextInputState; /* … */ }
// likewise TextArea, Select, Checkbox, Toggle, RadioGroup
pub struct Field<'a, C: FieldControl> { /* label, required, optional_suffix, help, error, plain, control: C */ }
impl<'a, C: FieldControl> Field<'a, C> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::LABEL,
                                         Part::MARKER, Part::FIELD, Part::HELP];
    pub fn new(label: &'a str, control: C) -> Self;                   // no Id — the control owns identity
    pub fn required(self, yes: bool) -> Self;   pub fn help(self, s: &'a str) -> Self;
    pub fn error(self, s: Option<&'a str>) -> Self;  pub fn plain(self, yes: bool) -> Self;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;      // design.size.field_height
}

/// Default key accessor: `ItemKey::index(i)` — documented UNSTABLE under insert/remove/reorder;
/// call `.key(…)` for stable identity. (§21 item 5, B2)
pub struct ByIndex;
/// Default row painter: the item's `Display` through `RowUi::label_fmt`, no allocation.
pub struct DefaultRow;
pub trait KeyFn<T> { fn key(&self, item: &T, index: usize) -> ItemKey; }
impl<T, F: Fn(&T) -> ItemKey> KeyFn<T> for F { /* self(item) */ }
impl<T> KeyFn<T> for ByIndex { /* ItemKey::index(index) */ }
pub trait RowFn<T> { fn row(&self, item: &T, u: &mut RowUi<'_>); }
impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F { /* self(item, u) */ }
impl<T: core::fmt::Display> RowFn<T> for DefaultRow { /* u.label_fmt(format_args!("{item}")) */ }

// Three impl blocks per collection: construction with real defaults, method-level generic
// builders, and phase methods under the trait bounds. Applied identically to `Tabs`, `Picker`,
// `Tree`, `NavList`, `Steps`, `ChipBar`, `RadioGroup`, `Completion`, `FilterList`, `PropsList`.
impl<'a, T> List<'a, T, ByIndex, DefaultRow> {
    pub fn new(id: Id) -> Self;                                       // items are per phase (A3)
}
impl<'a, T, K, R> List<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER,
                                         Part::LABEL, Part::META, Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2>;
    pub fn select_mode(self, m: SelectMode) -> Self;
    pub fn empty(self, e: EmptyState<'a>) -> Self;
    pub fn disabled_item(self, f: &'a dyn Fn(&T) -> bool) -> Self;
    pub fn status(self, s: Status) -> Self;
    pub fn patch(self, p: &'a StylePatch) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn slot(self, p: Part, f: &'a dyn Fn(&mut Ui<'_>, Rect)) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> List<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl<'a, T> Tabs<'a, T, ByIndex, DefaultRow> {
    pub fn new(id: Id) -> Self;
}
impl<'a, T, K, R> Tabs<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::TAB, Part::CLOSE, Part::NEW,
                                         Part::RULE, Part::OVERFLOW, Part::BADGE];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tabs<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tabs<'a, T, K, R2>;   // Part::TAB pre-styled
    pub fn allow_new(self, yes: bool) -> Self;  pub fn closable(self, yes: bool) -> Self;
    pub fn status(self, s: Status) -> Self;     pub fn state_override(self, s: StateFlags) -> Self;
    pub fn patch(self, p: &'a StylePatch) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> Tabs<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState, items: &[T]) -> Response<TabsAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TabsState, items: &[T]) -> Rect;
}
impl<'a, T> Picker<'a, T, ByIndex, DefaultRow> {
    pub fn new(id: Id) -> Self;
}
impl<'a, T, K, R> Picker<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::BORDER, Part::QUERY, Part::ROW,
                                         Part::LABEL, Part::META, Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Picker<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Picker<'a, T, K, R2>;
    pub fn scopes(self, s: &'a [ScopeKey]) -> Self;
    pub fn empty(self, e: EmptyState<'a>) -> Self;                    // was `.status(EmptyState)`; `Status` is now a type
    pub fn placeholder(self, s: &'a str) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> Picker<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut PickerState, items: &[T]) -> Response<PickerAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &PickerState, items: &[T]) -> Rect;
}
impl<'a> Grid<'a> {                                                   // no `M` on the props (§21 item 1, B15)
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::HEADER, Part::ROW, Part::CELL,
                                         Part::TRACK, Part::THUMB, Part::OVERFLOW, Part::EMPTY, Part::ACTIONS];
    pub fn new(id: Id, columns: &'a [Column<'a>]) -> Self;
    pub fn nav(self, u: NavUnit) -> Self;        pub fn select_mode(self, m: SelectMode) -> Self;
    pub fn empty(self, e: EmptyState<'a>) -> Self;
    pub fn editable(self, yes: bool) -> Self;
    pub fn actions_slot(self, f: &'a dyn Fn(&mut Ui<'_>, Rect)) -> Self;
    pub fn update<M: GridModel>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;
    pub fn draw<M: GridModel>(&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
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
    // convenience constructors over the same path (§9.2; `dialog::convenience_constructors_render_through_the_body_slot`)
    pub fn confirm(id: Id, title: &'a str, question: &'a str) -> Self;
    pub fn destructive(id: Id, title: &'a str, question: &'a str) -> Self;
    pub fn prompt(id: Id, title: &'a str, label: &'a str) -> Self;
    pub fn acknowledge(id: Id, title: &'a str, token: &'a str) -> Self;
    pub fn facts(id: Id, title: &'a str, props: &'a [(&'a str, &'a str)]) -> Self;
    pub fn choice(id: Id, title: &'a str, options: &'a [&'a str]) -> Self;
    pub fn info(id: Id, title: &'a str) -> Self;
}
impl<'a> Props<'a> {
    pub fn new(rows: &'a [(&'a str, &'a str)]) -> Self;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect;
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct ScopeKey(u16);
pub struct DialogState  { /* action cursor, prompt draft */ }
pub struct PickerState  { /* query editor core, cursor, scroll, active scope */ }
pub struct ListState    { /* cursor key, checked KeySet, scroll, gen stamp */ }
pub struct TabsState    { /* active key, cursor key, strip window, gen stamp */ }
pub struct GridState    { /* cursor cell, range anchor, row selection KeySet, two-axis scroll, edit lifecycle, gen stamp */ }
pub struct TextInputState { /* draft, editor core, phase, error */ }

// ---- A8. Status, capability and small value types (§21 items 13, 19, 20, 21, 22) ----
/// Data readiness of a component; the runtime maps it onto `StateFlags::{BUSY, LOADING, ERROR}`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Status { #[default] Ready, Busy, Loading, Error }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum ColorLevel { TrueColor, Ansi256, Ansi16, Mono }
pub struct Capability { pub color: ColorLevel }                      // `UnicodeLevel` deleted (M6)
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Density { Comfortable, Compact }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum SortDir { Asc, Desc }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Align { Left, Center, Right }               // text (StylePatch.align, CellUi::align)
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum ScreenAlign { Center, UpperThird, Bottom }  // Anchor::Screen
/// The state a binding table is selected for. `Copy`, so a table is chosen by `match` in a `const fn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct BindingState { pub flags: StateFlags }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum KeyPhase { Capture, Bubble }             // was `Phase2`
/// Selection set with an inverted representation so "select all" never materialises every key (§20.9-13).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeySet { Only(SmallVec<[ItemKey; 8]>), AllExcept(SmallVec<[ItemKey; 8]>) }
impl KeySet {
    pub fn contains(&self, k: ItemKey) -> bool;
    pub fn insert(&mut self, k: ItemKey);   pub fn remove(&mut self, k: ItemKey);   pub fn toggle(&mut self, k: ItemKey);
    pub fn all(&mut self);                  pub fn none(&mut self);
    pub fn len_in(&self, total: usize) -> usize;
    pub fn retain(&mut self, keep: impl Fn(ItemKey) -> bool) -> usize;   // returns the dropped count (reconcile)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegionKind {
    Control,     // focusable, delivers intents
    Part,        // sub-region of a Control; delivers to the Control's owner
    Scroll,      // wheel target only
    Decorative,  // paints and answers area_of; never delivers, never diagnosed
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit { pub owner: Id, pub part: PartRef, pub layer: LayerId, pub kind: RegionKind, pub local: Position }
impl Registry {
    pub fn hit(&self, pos: Position) -> Option<Hit>;                 // topmost region regardless of layer (§21 item 12)
    pub fn hit_scroll(&self, pos: Position, axis: Axis) -> Option<Hit>;
}
impl Chord { pub const fn key(c: KeyCode) -> Chord; pub const fn with(c: KeyCode, m: KeyModifiers) -> Chord; }
impl Key   { pub fn chord(&self) -> Chord; pub fn is(&self, c: KeyCode) -> bool; }   // `is`: code matches and no modifiers
impl PartRef { pub const fn of(p: Part) -> Self; pub const fn item(p: Part, k: ItemKey) -> Self; }
/// Implemented on every collection state type (ListState, TreeState, TabsState, GridState, …). (§21 item 21, M10)
pub trait Reconcile {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    fn invalidate(&mut self);     // caller mutated items in place without changing len/ends
}
#[derive(Clone, Copy, Default)]
pub struct RowDecor<'a>  { pub marker: Option<GlyphRole>, pub tone: Option<Role>, pub strike: bool, pub faint: bool,
                           pub flags: StateFlags, pub message: Option<&'a str> }
#[derive(Clone, Copy, Default)]
pub struct CellDecor<'a> { pub tone: Option<Role>, pub italic: bool, pub error: Option<&'a str>,
                           pub dirty: bool, pub suffix: Option<GlyphRole> }
impl CellUi<'_> {                                                    // §12.2 (§21 item 21, M9)
    pub fn text(&mut self, s: &str) -> &mut Self;
    pub fn num(&mut self, n: i64) -> &mut Self;        // formats in place, 0 allocations
    pub fn money(&mut self, cents: i64) -> &mut Self;  // ditto
    pub fn align(&mut self, a: Align) -> &mut Self;
    pub fn tone(&mut self, r: Role) -> &mut Self;
    pub fn italic(&mut self, yes: bool) -> &mut Self;
    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self;
    pub fn patch(&mut self, p: &StylePatch) -> &mut Self;
}
impl RowUi<'_> {
    pub fn part(&mut self, p: Part, width: u16) -> CellUi<'_>;       // reserves `width` columns from the RIGHT; `label` fills what is left
    pub fn label_fmt(&mut self, args: core::fmt::Arguments<'_>);     // in-place formatting for DefaultRow, 0 allocations
}
pub struct Column<'a> { /* key: ColumnKey, title, subtitle, align, min/max width, sortable, editable, sticky, prefix_glyph, badge */ }
pub struct CodeDiagnostic { /* range, severity, message */ }        // was `Diagnostic'` (A13)
pub struct Jx<'f> { /* jackin-owned request bus: requests, go, status, open, close, help, copy, with_form */ }

// ---- A9. Diagnostics (§7.1, §8.4, §3.3, §16.4; §21 items 11, 13, 17, 30) ----
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    DuplicateId { id: Id, first: Rect, second: Rect },
    CursorRejected { owner: Id, layer: LayerId },
    UndeliveredIntent { owner: Id },
    BindingConflict { chord: Chord, phase: KeyPhase, a: Id, b: Id },
    FocusTransitionDidNotSettle { target: Option<Id> },
    UnaddressableId { id: Id },
    DuplicateLayerDraw { id: Id },
}
// `Runtime` retains at most 64 diagnostics plus a dropped count, cleared at the start of each `handle` (P6).
```

---

**1 — Default button** (`examples/01_button.rs`)

```rust
use junie_tui::{id, layout, run, App, Button, Cx, Id, Insets, Response, Theme, Ui};   // <!-- amended by §21 item 34: `Insets` import -->

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

Nothing registers a hit region, computes hover, tracks press, or places focus: `Button::draw` calls `ui.register_control(SAVE, area, Focusability::Focusable)` and the runtime does the rest (G3). `Button::new(SAVE, "Save")` appears in both phases because it carries no configuration beyond `new` — the one case §13's "props are built once" convention exempts.

**2 — A complete custom theme** (`examples/02_custom_theme.rs`)

```rust
use junie_tui::theme::{BorderSet, ColorTokens, Density, MeterTokens, SyntaxTokens, Theme};
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

fn main() { let _ = slate(); }
// `Theme::from_tokens` fills design tokens and recipe defaults; `downgrade` works for it
// exactly as for `junie()`, because `map_colors` is an exhaustive destructure (§11.4).
// `ColorTokens` is deliberately NOT `#[non_exhaustive]` (§21 item 8): a new token is an
// intentional breaking change for downstream themes, and this literal is the proof.
```

**3 — Partial theme override** (`examples/03_partial_theme.rs`)

```rust
use junie_tui::Theme;
use ratatui::style::Color::Rgb as rgb;

fn main() {
    // Change three roles; everything else is inherited from Junie, unchanged, byte-for-byte.
    let t = Theme::junie()
        .builder()
        .accent(rgb(0xC6, 0x7A, 0x2E))   // amber instead of green
        .focus(rgb(0xC6, 0x7A, 0x2E))    // `ThemeBuilder::focus` — §21 item 21
        .danger(rgb(0xB0, 0x25, 0x25))
        .build();

    assert_eq!(t.color.surfaces, Theme::junie().color.surfaces);   // untouched roles inherit
}
```

**4 — Global family recipe override** (`examples/04_family_recipe.rs`)

```rust
use junie_tui::{Family, FgStep, GlyphRole, Modifier, Part, Role, StateFlags, StylePatch, Theme, Variant};

fn main() {
    // Every Button in the application: square gutter marker, bold label when focused,
    // tinted container when hovered. No component source is edited.
    let _t = Theme::junie().override_family(Family::BUTTON, |r| {
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
}
// The two-flag rule wins over the one-flag rule by `when.count_ones()` (§11.3 step 3),
// and the rules are stored pre-sorted so resolution never allocates (§20.9-1).
```

**5 — Per-instance part override** (`examples/05_instance_patch.rs`) — Scenario C

```rust
use junie_tui::{id, layout, Button, Id, Part, Rect, Role, RowAlign, StylePatch, Ui, Variant};

const OK: Id = id!("ok");
const RESET: Id = id!("reset");

// One patch, declared `const`, so it costs nothing per frame.
const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL,  StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

pub fn draw_actions(ui: &mut Ui<'_>, area: Rect) {
    let cols = layout::action_row(area, &[10, 12], ui.design().space.gap, RowAlign::Right);
    Button::new(OK, "OK").variant(Variant::PRIMARY).draw(ui, cols[0]);
    Button::new(RESET, "Reset").patch_part(&RESET_LABEL).draw(ui, cols[1]);
}

fn main() {}
// Both buttons use the same global theme and the same renderer; only one is patched,
// and `conformance::button::local_override_does_not_mutate_the_theme` proves the
// theme is byte-identical afterwards.
```

**6 — Text field with external validation** (`examples/06_validated_field.rs`) <!-- amended by §21 item 7 -->

```rust
use junie_tui::{id, BlurPolicy, Cx, Field, FieldError, Id, Rect, Response, TextAction,
                TextInput, TextInputState, Ui};

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

fn check_uniqueness(_s: &str) -> Option<String> { None }   // stands in for the application effect

/// The one constructor for this control, used by both phases (§13 "props are built once").
/// It takes no `&self`, so `update` can still pass `&mut self.email` alongside it.
fn email_input() -> TextInput<'static> {
    TextInput::new(EMAIL)
        .validate(&valid_email)                        // fn item, via the blanket `Validate` impl
        .blur(BlurPolicy::CommitAndValidate)
}

impl Form {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = email_input().update(cx, &mut self.email_st, &mut self.email);

        if let Some(TextAction::Committed) = r.action_ref() {
            self.server_error = check_uniqueness(&self.email);          // application effect
            self.email_st.set_error(self.server_error.as_deref()
                .map(|m| FieldError { message: m.to_owned().into(), code: Some("dup") }));
        }
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        // `Field` is draw-time chrome only: no `Id`, no focus stop, no `update`. The control
        // keeps its identity, so one id is registered per field (§21 item 7).
        Field::new("Email", email_input().value(&self.email))
            .required(true)
            .help("We only use this for sign-in.")
            .error(self.server_error.as_deref())
            .draw(ui, area, &self.email_st);
    }
}

fn main() {}
// `draw` is `&self` and takes `&TextInputState`: committing or validating from draw is a
// compile error, which is what removes the five render-time commits of §1.2(5).
```

**7 — List with borrowed domain rows and a custom renderer** (`examples/07_borrowed_rows.rs`) — Scenario D <!-- amended by §21 items 1, 5, 21 -->

```rust
use junie_tui::{id, Align, Cx, EmptyState, FgStep, GlyphRole, Id, ItemKey, List, ListAction,
                ListState, Part, Rect, Response, Role, RowUi, SelectMode, Ui};

pub struct Order { pub id: u64, pub customer: String, pub total_cents: i64, pub flagged: bool }

const ORDERS: Id = id!("orders");

struct Screen { orders: Vec<Order>, list: ListState, chosen: Option<u64> }

fn order_row(o: &Order, r: &mut RowUi<'_>) {
    if o.flagged { r.marker(GlyphRole::WarningMark); }
    r.label(&o.customer);                            // borrowed &str, one grapheme walk, 0 allocs
    r.part(Part::META, 12)                           // 12 columns reserved from the right
        .money(o.total_cents).align(Align::Right)    // formats into the cell, no String
        .tone(if o.total_cents < 0 { Role::Danger } else { Role::Fg(FgStep::Muted) });
}

/// Configuration and closures only — the rows are passed to each phase call (§21 item 1),
/// so the props never borrow `self.orders` and the action closure is free to mutate `self`.
fn orders_list() -> List<'static, Order, impl Fn(&Order) -> ItemKey, impl Fn(&Order, &mut RowUi<'_>)> {
    List::new(ORDERS)
        .key(|o: &Order| ItemKey::num(o.id))
        .row(order_row)
        .select_mode(SelectMode::Single)
        .empty(EmptyState::Empty { title: "No orders", hint: Some("Adjust the filter") })
}

impl Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        orders_list()
            .update(cx, &mut self.list, &self.orders)
            .on_action(|a| if let ListAction::Chose(k) = a {
                self.chosen = self.orders.iter().find(|o| ItemKey::num(o.id) == k).map(|o| o.id);
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        orders_list().draw(ui, area, &self.list, &self.orders);
    }
}

fn main() {}
// Nothing is converted to owned strings, only visible rows invoke the renderer, and the
// action carries `ItemKey`, never a display index.
```

**8 — Dynamic tabs with stable keys** (`examples/08_dynamic_tabs.rs`) — Scenario E <!-- amended by §21 items 1, 5 -->

```rust
use junie_tui::{id, Cx, GlyphRole, Id, ItemKey, Rect, Response, RowUi, Tabs, TabsAction, TabsState, Ui};

pub struct Doc { pub key: u64, pub title: String, pub dirty: bool }
const STRIP: Id = id!("strip");

struct Workspace { docs: Vec<Doc>, strip: TabsState, next_key: u64 }

fn tab_view(d: &Doc, r: &mut RowUi<'_>) {
    r.label(&d.title);
    if d.dirty { r.marker(GlyphRole::Dirty); }
}

fn strip() -> Tabs<'static, Doc, impl Fn(&Doc) -> ItemKey, impl Fn(&Doc, &mut RowUi<'_>)> {
    Tabs::new(STRIP)
        .key(|d: &Doc| ItemKey::num(d.key))
        .row(tab_view)
        .allow_new(true)
        .closable(true)
}

impl Workspace {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        strip()
            .update(cx, &mut self.strip, &self.docs)   // reconcile() runs first, every frame;
            .on_action(|a| match a {                    // the borrow of `self.docs` ended with `update`
                TabsAction::Activated(_k) => { /* the active key, not an index */ }
                TabsAction::Close(k)      => self.docs.retain(|d| ItemKey::num(d.key) != k),
                TabsAction::New           => {
                    self.next_key += 1;
                    self.docs.insert(0, Doc { key: self.next_key, title: "Untitled".into(), dirty: false });
                }
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        strip().draw(ui, area, &self.strip, &self.docs);
    }
}

fn main() {}
// Insert at position 0: the active tab, the strip window and any pending close still name
// the same `ItemKey`. Nothing is rebuilt; `TabsState` is never reconstructed.
```

**9 — Composed dialog with an arbitrary body** (`examples/09_composed_dialog.rs`) <!-- amended by §21 item 34: compile fixes only (`token_st`, imports, the body field's `update`) -->

```rust
use junie_tui::{id, layout, Action, ActionKey, Cx, Dialog, DialogAction, DialogState, DismissReason,
                Id, LayerSpec, Part, Props, Response, TextInput, TextInputState, Track, Ui};

const CONFIRM: Id = id!("confirm.delete");
const TOKEN: Id = CONFIRM.part(Part::FIELD);          // a child COMPONENT id inside the dialog (§21 item 16)
const K_CANCEL: ActionKey = ActionKey::CANCEL;
const K_DELETE: ActionKey = ActionKey::custom("delete");

struct Screen { dlg: DialogState, token: String, token_st: TextInputState, target: String, deleted: bool }

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) { cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM)); }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = TextInput::new(TOKEN).update(cx, &mut self.token_st, &mut self.token).erase();

        let armed = self.token.trim() == self.target;      // arming is an `update` predicate
        let actions = [
            Action::new(K_CANCEL, "Cancel"),
            Action::danger(K_DELETE, "Delete").enabled(armed),
        ];
        let d = Dialog::new(CONFIRM)
            .title("Delete table")
            .description("This cannot be undone.")
            .actions(&actions)
            .cancel(K_CANCEL)
            .update(cx, &mut self.dlg);

        match d.action_ref() {
            Some(DialogAction::Action(k)) if *k == K_DELETE => { self.deleted = true; cx.close_layer(CONFIRM, Some(K_DELETE)); }
            Some(DialogAction::Action(_)) | Some(DialogAction::Dismissed(DismissReason::Esc)) =>
                cx.close_layer(CONFIRM, Some(K_CANCEL)),
            _ => {}
        }
        r |= d.erase();
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(CONFIRM, |ui, area| {
            Dialog::new(CONFIRM).title("Delete table").width(60)
                .draw(ui, area, &self.dlg, |ui, body| {           // ARBITRARY body content
                    let rows = layout::rows(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)]);
                    Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")]).draw(ui, rows[0]);
                    ui.rule(rows[1]);
                    TextInput::new(TOKEN).value(&self.token)
                        .placeholder("Type the table name to confirm").draw(ui, rows[2], &self.token_st);
                });
        });
    }
}

fn main() {}
// `DialogBody` does not exist. The body is a closure that borrows application data.
// Focus trapping, backdrop, Esc, click-outside, focus restore and the hint layer come
// from the layer, not from the dialog. Esc reaches the editing `TextInput` first and the
// layer only afterwards (§21 item 3).
```

**10 — Nested picker inside a dialog** (`examples/10_nested_overlay.rs`) — Scenario F <!-- amended by §21 items 1, 5, 8, 16 -->

```rust
use junie_tui::{id, Action, ActionKey, Anchor, Button, CrossAlign, Cx, Dialog, DialogState, Dismiss,
                FrameRead, Id, ItemKey, LayerEvent, LayerSpec, Part, Picker, PickerAction, PickerState,
                Response, RowUi, Side, Ui};

pub struct Person { pub id: u64, pub name: String, pub team: String }

const DLG: Id = id!("dlg");
const OWNER_BTN: Id = DLG.part(Part::custom("owner"));   // a child COMPONENT id (§21 item 16, M5), not a PartRef
const OWNER_PICK: Id = id!("dlg.owner_picker");
const K_DONE: ActionKey = ActionKey::CONFIRM;

struct Screen { dlg: DialogState, pick: PickerState, people: Vec<Person>, owner: Option<u64> }

fn dialog() -> Dialog<'static> { Dialog::new(DLG).title("Edit task") }

fn owner_picker() -> Picker<'static, Person, impl Fn(&Person) -> ItemKey, impl Fn(&Person, &mut RowUi<'_>)> {
    Picker::new(OWNER_PICK)
        .key(|p: &Person| ItemKey::num(p.id))
        .row(|p: &Person, u: &mut RowUi<'_>| { u.label(&p.name); u.meta(&p.team); })
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) { cx.open_layer(DLG, LayerSpec::modal(DLG)); }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        let actions = [Action::new(K_DONE, "Done")];
        r |= dialog().actions(&actions).cancel(K_DONE)
                .update(cx, &mut self.dlg)
                .on_action(|_| cx.close_layer(DLG, Some(K_DONE)));

        // The picker opens ON TOP of the dialog as a popover anchored below the button: its
        // own focus scope and a pointer barrier, no full-screen dim (§21 item 8). The dialog
        // beneath is pointer- and key-inert until the picker closes.
        let anchor = cx.area(OWNER_BTN).unwrap_or_default();
        r |= Button::new(OWNER_BTN, "Choose owner…").update(cx)
                .on_activated(|| cx.open_layer(OWNER_PICK,
                    LayerSpec::popover(OWNER_PICK, Anchor::Rect { rect: anchor, side: Side::Below, align: CrossAlign::Start })
                        .dismiss(Dismiss::ESC_AND_OUTSIDE)));

        if cx.is_open(OWNER_PICK) {
            r |= owner_picker()
                    .update(cx, &mut self.pick, &self.people)
                    .on_action(|a| if let PickerAction::Chosen(k) = a {
                        self.owner = self.people.iter().find(|p| ItemKey::num(p.id) == k).map(|p| p.id);
                        cx.close_layer(OWNER_PICK, Some(ActionKey::CONFIRM));
                    });
        }

        if let Some(LayerEvent::Dismissed(_)) = cx.layer_event(OWNER_PICK) { /* nothing to undo */ }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(DLG, |ui, a| {
            dialog().draw(ui, a, &self.dlg, |ui, body| {
                Button::new(OWNER_BTN, "Choose owner…").draw(ui, body);
            });
        });
        ui.layer(OWNER_PICK, |ui, a| { owner_picker().draw(ui, a, &self.pick, &self.people); });
    }
}

fn main() {}
// Esc closes only the picker; the dialog stays open and regains focus at the button.
// No barrier is pushed by hand, no hit region is re-registered, and the picker draws no
// hint row of its own — the top layer contributes to the shared HintBar (§13.1).
// z-order is the `LayerId` assigned by `open_layer`, not the order of the two `ui.layer` calls (§21 item 14).
```

**11 — A small complete application on shared focus and dispatch** (`examples/11_small_app.rs`) — Scenario A <!-- amended by §21 items 1, 5, 7 -->

```rust
use junie_tui::{id, layout, run, Action, ActionKey, App, Button, Cx, Dialog, DialogAction,
                DialogState, Field, Id, Insets, ItemKey, LayerSpec, List, ListAction, ListState,
                Response, RowUi, TextInput, TextInputState, Theme, Track, Ui, Variant};

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

// One constructor per configured control, called from both phases (§13 "props are built once").
// Each takes the fields it needs as parameters, never `&self`, so `update` keeps `&mut` access.
fn add_button(name_empty: bool) -> Button<'static> {
    Button::new(ADD, "Add").variant(Variant::PRIMARY).disabled(name_empty)
}
fn people_list() -> List<'static, String, impl Fn(&String) -> ItemKey, impl Fn(&String, &mut RowUi<'_>)> {
    List::new(PEOPLE)
        .key(|s: &String| ItemKey::text(s))
        .row(|s: &String, u: &mut RowUi<'_>| u.label(s))
}
fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(CONFIRM, "Remove person", "Remove this person from the roster?")
}

impl App for Roster {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();

        r |= TextInput::new(NAME).update(cx, &mut self.name_st, &mut self.name).erase();

        r |= add_button(self.name.trim().is_empty())
                .update(cx)
                .on_activated(|| { self.people.push(std::mem::take(&mut self.name)); });

        r |= people_list()
                .update(cx, &mut self.list, &self.people)       // items per phase (§21 item 1)
                .on_action(|a| if let ListAction::Activated(k) = a {
                    self.pending_remove = Some(k);
                    cx.open_layer(CONFIRM, LayerSpec::modal(CONFIRM));
                });

        if cx.is_open(CONFIRM) {
            let actions = [Action::new(K_NO, "Cancel"), Action::danger(K_YES, "Remove")];
            r |= remove_dialog().actions(&actions).cancel(K_NO)
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
        let rows = layout::rows(body, &[Track::Fixed(3), Track::Fixed(1), Track::Flex(1)]);
        let top  = layout::columns(rows[0], &[Track::Flex(1), Track::Fixed(10)], ui.design().space.gap);

        Field::new("Name", TextInput::new(NAME).value(&self.name)).draw(ui, top[0], &self.name_st);
        add_button(self.name.trim().is_empty()).draw(ui, top[1]);
        people_list().draw(ui, rows[2], &self.list, &self.people);

        ui.layer(CONFIRM, |ui, a| { remove_dialog().draw(ui, a, &self.dlg, |_, _| {}); });
    }

    fn should_quit(&self) -> bool { self.quit }
}

fn main() -> std::io::Result<()> { run(Roster::default(), Theme::junie()) }
```

The Scenario A checklist is satisfied by omission: there is no hit region, no mouse coordinate, no hover or pressed field, no derived child id, no Tab implementation, no modal barrier, no focus save/restore, no `set_cursor_position`, and no "which row was clicked" arithmetic.

**12 — A downstream component using only `junie_tui::author`** (`examples/12_author_component.rs`) — Scenario G <!-- amended by §21 items 6, 10, 16, 18, 30 -->

```rust
use junie_tui::author::{Binding, BindingState, Bindings, Chord, Cx, Family, Focusability, FrameRead,
                        GlyphRole, Id, Intent, ItemKey, KeyCode, Part, PartRef, Phase, Rect, Response,
                        StateFlags, Ui, Variant};

/// A segmented control: N labelled segments, one selected, roving cursor.
pub struct Segmented<'a> { id: Id, labels: &'a [&'a str], variant: Variant }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SegmentedState { pub cursor: usize, pub selected: usize }

#[derive(Clone, Copy, Debug, PartialEq)] pub enum SegmentedAction { Moved, Selected(ItemKey) }

/// The const-constructible command a chord maps to. `update` turns it into a `SegmentedAction`
/// carrying the live key (§21 item 10, M11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum SegCmd { Prev, Next, Select }

const SEGMENT: Part = Part::custom("segment");
const F_SEGMENTED: Family = Family::custom("segmented");

const BINDINGS: &[Binding<SegCmd>] = &[
    Binding { chord: Chord::key(KeyCode::Left),      cmd: SegCmd::Prev,   label: "Prev",   priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Right),     cmd: SegCmd::Next,   label: "Next",   priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Enter),     cmd: SegCmd::Select, label: "Select", priority: 80, visible: true },
    Binding { chord: Chord::key(KeyCode::Char(' ')), cmd: SegCmd::Select, label: "Select", priority: 80, visible: false },
];

impl Bindings for Segmented<'_> {
    type Cmd = SegCmd;
    fn bindings(&self, _s: BindingState) -> &'static [Binding<SegCmd>] { BINDINGS }
}

impl<'a> Segmented<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, SEGMENT, Part::LABEL];
    pub fn new(id: Id, labels: &'a [&'a str]) -> Self { Self { id, labels, variant: Variant::DEFAULT } }
    pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SegmentedState) -> Response<SegmentedAction> {
        let mut r = Response::ignored();
        let n = self.labels.len();
        if n == 0 { return r.for_id(self.id); }
        // `cx.intents` borrows only the frozen queue (§21 item 6), so `cx`'s services stay
        // usable inside the loop; keys are matched through the SAME table the hint bar shows,
        // which is what `bindings_match_handled_keys` checks.
        for it in cx.intents(self.id) {
            match it {
                Intent::Key(k) => match BINDINGS.iter().find(|b| b.chord == k.chord()).map(|b| b.cmd) {
                    Some(SegCmd::Prev)   => { st.cursor = (st.cursor + n - 1) % n; r = Response::action(SegmentedAction::Moved); }
                    Some(SegCmd::Next)   => { st.cursor = (st.cursor + 1) % n;     r = Response::action(SegmentedAction::Moved); }
                    Some(SegCmd::Select) => {
                        st.selected = st.cursor;
                        r = Response::action(SegmentedAction::Selected(ItemKey::index(st.selected)));
                        cx.request_repaint();
                    }
                    None => {}
                },
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
            if let Some(g) = r.glyph { ui.glyph(cell, g, r.style); }              // a declared part paints Resolved.glyph (A4)
            else if s.contains(StateFlags::SELECTED) { ui.glyph(cell, GlyphRole::Chosen, r.style); }
            let ls = ui.style(F_SEGMENTED, self.variant, Part::LABEL, s).style;
            ui.paint_str(cell, label, ls);
            ui.register_part(self.id, PartRef::item(SEGMENT, ItemKey::index(i)), cell);   // RegionKind::Part
        }
        area
    }
}

fn main() {}
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
| — (new) | **compose** | `Intent`, `PartRef`, `Phase`, `FocusVia`, `IntentIter`, `IntentQueue` | `«tui»/intent.rs` | Pre-resolved input. Removes `owns`/`locate` from the model (§3.3 step 3). |
| — (new) | **compose** | `Binding`, `Bindings`, `KeyMap`, `KeyPhase`, `HintLayer`, `Hint` | `«tui»/keymap.rs` | Hard-coded product chords in 18 modules (**[F]** API §3.9) become `const` tables + an app override layer (§13.1). |
| `src/core/focus.rs` | **refactor** | `FocusRing`, `FocusEntry`, `FocusState`, `ScopeId`, `ScopeMode`, `Focusability`, `FocusVis` | `«tui»/focus.rs` | Single `barrier: Option<usize>` **deleted** → scopes + traps. `Focus`/`FocusRing` no longer public `&mut` to components. Adds restore map, the (a)(b)(c)(d) reconcile rule, disabled-but-registered entries. Satisfies §8.1, Scenario F. |
| `src/core/hit.rs` | **refactor** | `Registry`, `Region`, `RegionKind`, `Hit`, `Headroom`, `Axes` | `«tui»/hit.rs` | Barrier **deleted** → `layer: LayerId` per region. Regions carry `PartRef` (24 B, `Copy`), so `area_of`-by-render-order and all 12 `locate` helpers die. `hit_scroll` returns a region at zero headroom. Satisfies §8.3, §7.1. |
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
| `code` | **refactor** | `CodeEditor<'a>`, `CodeEditorState`, `Highlighter`, `Segmenter`, `CodeDiagnostic` | `«tui»/components/code.rs` | Render-time commit (`code.rs:611`) **impossible** (`&self`); `fn`-pointer `Highlighter`/`Segmenter` → `&'a dyn Fn`; per-frame `hash_text` → edit counter; per-grapheme linear span scan → sorted-span cursor; vim key table → default `KeyMap`. Satisfies §20.9-9. |
| `completion` | **compose + controller** | `Completion<'a,T,K,R>`, `CompletionState`, `CompletionController` | `«tui»/components/completion.rs` | Becomes `Popover` layer content; missing `on_scrollbar` fixed by `scroll_region`; boundary-wheel violation (`completion.rs:142-145`) fixed; the ~40 lines of editor↔popup hand-wiring in `tablepro/tabs.rs:1326-1377` collapse into the controller. Shares `Item` with `Picker`. |
| `dialog` | **decompose** | `Dialog<'a>`, `DialogState`, `Action<'a>`, `ActionKey`, `DialogAction` | `«tui»/components/dialog.rs` | `DialogBody` **deleted**; polled `result` field **deleted**; `&mut Focus` parameters **deleted**; render-time ack arming (`dialog.rs:465`) **deleted** (an `update` predicate); backdrop loop **deleted** (one layer implementation); `dialog.rs:389`'s trap-less modal **fixed** (the trap belongs to the layer). Satisfies §9.2, goal §14. |
| `diff` | **retain (composition)** | `DiffView<'a>`, `DiffViewState`, `DiffSource`, `DiffMode` | `«tui»/components/diff.rs` | Data model moves behind `DiffSource` so jackin's `sim::changes::ChangedFile` feeds it without conversion; `review_lines(f,width)` becomes `measure`; the render-time `set_follow(false)`/`scroll_to` in the layout cache moves to `update`. |
| `empty` | **retain** | `EmptyState<'a>` rendering inside each collection | `«tui»/collection/empty.rs` | The free `render(…, bg)` **deleted**; empty/loading/partial/error become one vocabulary (absorbs `PickerStatus`). |
| `field_common` | **remove → refactor** | `EditAction` table on `TextEditorCore` + `Binding` set | `«tui»/text/editor.rs`, `«tui»/keymap.rs` | `EditAction::Apply(fn(&mut TextBuffer))` **deleted** (fn pointer, API §3.12). The shared keymap becomes a `const [Binding<EditAction>]`. |
| `grid` | **decompose** | `Grid<'a>`, `GridState`, `GridModel`, `GridEditor`, `GridCellActions`, `ColumnKey`, `Column`, `CellRef`, `NavUnit`, `EditIntent`, `CellAction` | `«tui»/components/grid.rs` | Everything DB-shaped moves out (see 18.4 TablePro row). `CellValue`, `PendingChanges`, `UndoAction`, `default_validator`, `cmp_cells`, `Validator` fn pointer, `"Preview SQL"`, `primary`/`nullable`/`references`/`enum_values`, `Theme::change_glyph` **all deleted from the library**. `col_rects.clone()` per row **deleted**. `GridEditor` is `&mut self` and unreachable from `draw`. Satisfies §12.3, Scenario H, DOM §7 condition 1. |
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
| `Id → String` debug table populated at **registration** | INT B1 | 300 map inserts + 300 `String`s per frame in TablePro; visibly laggy and corrupts allocation-counting tests | The debug label travels with the `Id`; no side table exists in any build (R4, §20.9-5, §21 item 22) |
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

**20.2 Two phases means two constructions of the props struct per frame.** `update` and `draw` each build `Button::new(SAVE, "Save")`. Props are stack-allocated borrowed views, so the cost is register moves, not allocation (`frame_showcase_lists_120x40 < 20 allocs/frame`). The benefit is that `draw` can take `&self` and `&XState`, which is what makes G2 a compile error. For anything with configuration the helper is **mandatory**, not optional (§13 "props are built once", §21 item 30): drift between the two constructions is a silent bug class the compiler cannot see. The migrated jackin `Screen` in §3.4 and examples 6–11 do exactly that.

**20.3 Collection generics (`List<'a, T, K, R>`) are three type parameters.** They are always inferred at the call site and never appear in an application signature (§13); the defaults are `ByIndex`/`DefaultRow`, builders change the parameters at method level, and `Grid` carries no model parameter at all (§21 items 1, 5). The alternative — boxing the key and row closures — costs two allocations per collection per frame and a `'static` bound. `architecture::no_static_bound_in_component_surface` guards the boundary.

**20.4 `Id` is a 64-bit hash with no reverse mapping in release builds.** Collision safety is *detection* (`Diagnostic::DuplicateId`), not prevention. With kind-tagged, separator-delimited segments the accidental-collision class of §1.2(1) is eliminated; a genuine FNV collision remains theoretically possible and is reported, never panicked (goal §10). Debug builds carry a zero-cost-in-release `DebugLabel`; it makes `Id` ~48 B in debug versus 8 B in release, so every `Region` roughly doubles in debug — expected, and not what `debug_and_release_alloc_counts_match` measures (P7, §21 item 30).

**20.5 `Invalidate::Layout` ships but behaves as `Paint`.** It is reserved for future layout caching; only its ordering is asserted (§8.5). Shipping it now avoids a breaking change later; the cost is one variant that does nothing distinguishable today.

**20.6 `&'a dyn Fn` slots are the only `dyn` in a component's public surface.** They are opt-in and allocation-free, but they are dynamic dispatch on a per-part path. Measured under `style_resolve_10k_parts`; the alternative (a generic slot parameter per part) would add one type parameter per replaceable part to every component signature, which fails §13's "no gratuitous generic parameters".

**20.7 Controlled values require the caller to hold a `String` per field.** For a 15-field form that is 15 owned strings the caller must place somewhere. This is deliberate: it deletes the "rebuild the widget to change its value" idiom (five sites) and makes external synchronisation trivial. Uncontrolled mode (`XState` owns the draft) remains for throwaway fields and is documented per component (S4).

**20.8 The conformance suite is a hard cost per new component.** Registering a `Conformance` case is ~40 lines. It is mandatory (`architecture::conformance_covers_every_public_component`), which is the point: 20 contracts become free for every component, forever, and the untested-module gap of **[F]** API §8.1 (21 of 31 modules with no tests) cannot recur.

### 20.9 Performance obligations (binding)

<!-- amended by §21 items 2, 6, 22, 30 -->

These are **amendments to §3–§15**, not advice. Each folds a `docs/audit/performance-audit.md` finding into the accepted architecture and carries the acceptance test that proves it. A builder who implements §3–§15 without these has not implemented the architecture.

| # | Amendment | Amends | Acceptance test (§16.6) |
|---|---|---|---|
| 1 | **State rules are stored in specificity order at recipe-build time.** `PartEdit::when` inserts into `PartRecipe.states` sorted by `when.count_ones()` ascending, ties by declaration order. §11.3 step 3's "ordered by specificity" is therefore a *storage* invariant, not a resolution-time sort. `Ui::style` is `for rule in &part.states { if s.contains(rule.when) { acc = acc.merge(rule.patch) } }` and **allocates nothing**. (R2) | §11.3 | `style_resolve_10k_parts` — **exactly 0 allocations**, ns ≤ 2× the pre-refactor `Theme::row`+`Theme::gutter` baseline |
| 2 | **The §11.1 A3 memo cache is allocation-free and statically sized.** A `[Option<(u64, Resolved)>; 256]` direct-mapped array embedded in `Ui`, keyed by a 64-bit mix of `(Family, Variant, Part, StateFlags, Surface, overlay_stack_hash)`, cleared by a generation stamp rather than by zeroing. No `HashMap`, no `Vec`, no per-frame allocation, no growth. A miss recomputes; there is no eviction policy to get wrong. `Ui` keeps a running `stack_hash: u64` updated on `with_overlay` push/pop so no per-query stack hash is computed (P3), and `Ui` is constructed once per `Runtime`/`Scene` and reused, never per frame (P4). | §11.1 A3 | `style_resolve_10k_parts`, `render_twice_allocates_the_same` |
| 3 | **`ItemKey` reconcile uses a generation stamp cache.** Every `XState` with a cursor stores `(cursor_key, cursor_index, stamp)` where `stamp = (len, key(first), key(last))`. `reconcile` returns `Unchanged` immediately when the stamp matches; on a mismatch it first probes the cached index (`key(&items[i]) == cursor_key`) and only then scans. `XState::invalidate()` is public for callers who mutate in place. 100 000 rows never re-hash per frame. (R1) | §12.2 | `list_100k_rows_render` — **< 500 allocs/frame**, ns ≤ 1.5× `list_1k_rows_render`; `event_dispatch_is_not_o_n` — 0 allocs, ns within 3× of the 100-row click |
| 4 | **Overlay lookup is a linear scan over a `&'static` slice, short-circuited when empty.** `Overlay::new(&'static [(Family, Variant, Part, StateFlags, StylePatch)])`; the resolution loop returns before touching the stack when `stack.is_empty()`, which is the overwhelmingly common case. No hashing on the style path. (R3) | §11.3 step 5 | `style_resolve_10k_parts_with_two_overlays` — 0 allocations, ns ≤ 2× the empty-stack case |
| 5 | **`Id` debug names are populated at construction, never at registration.** `id!` expands to a `const` path and the debug label travels with the `Id` itself (`DebugLabel { root, tail }`, `Tail::Item(k)` inline), so there is no `Registry::names` and no side table in any build; nothing is populated at registration and no `debug-ids` feature is needed (R4; §7.1 amended, `Registry::names` struck — §21 item 22). | §7.1 | `debug_and_release_alloc_counts_match` — `frame_tablepro_grid_500x12_120x40` reports identical allocation counts in debug and release |
| 6 | **`RowUi`/`CellUi` paint via a single grapheme walk with no intermediate `String`.** `RowUi::label`, `meta`, `trailing` and `CellUi` write cells directly and pad in place. `ui::text::{fit, fit_right}` are **deleted from every render path** and survive only for non-render callers. §12.2's `RowUi` contract is amended to forbid intermediate allocation. (R5) | §12.2 | `fit_10k_grapheme_line_to_80` — the `RowUi` equivalent records **0** allocations; `frame_showcase_lists_120x40` drops from ≈160 to **< 20** allocs/frame; `grid_500x12_render` **< 100** |
| 7 | **`TextViewport` cells become `(range, width)` with windowed incremental layout.** `Cell { range: Range<u32>, w: u8, style_ix: u16 }` referencing the source `Span` text instead of an owned grapheme `String`; layout is append-only on `push` and lays out only `visible_range ± 1 page`. The windowed layout is a function of `area` and therefore lives in `ui.cache::<ViewportLayout>(id)` (R8, §21 item 2), keyed by the buffer generation in `ViewportState`; it is never in `ViewportState` itself. §14.1's "`TextViewport` — Keep" is amended: the *behaviour* is kept, the *storage* is rewritten. **P-A (WP‑0 finding):** today's `TextViewport::render` re-lays out the whole buffer twice per frame (at `width`, then at `width − 1` for the scrollbar), which is why `viewport_100k_lines_render` records 15.2 M allocs/frame; that test is the binding acceptance for this item. (perf §6.3-1) | §14.1, §12.4 | `viewport_100k_lines_push` — allocations **independent of `lines.len()`**; `viewport_layout_10k_grapheme_line` — **0** allocations; `viewport_100k_lines_render` — allocs/frame independent of buffer size |
| 8 | **`Tree` flatten is incremental and keyed.** The flat index is rebuilt only for the affected subtree on expand/collapse, `expanded` is `HashSet<ItemKey>` (no `Vec<usize>` hashing), rows borrow `label`/`meta` from the source nodes, and filtering does not lowercase per node per level. The flat index is consumed per draw and filtered by a query that changes in `update`, so it lives in `ui.cache::<TreeIndex>(id)` (R8, §21 item 2), rebuilt when the cached `(expand_generation, query_hash)` stamp differs from `TreeState`'s; `TreeState` holds only `expanded: HashSet<ItemKey>` and the generation. §12.4's "`Tree` keeps hierarchy, lazy children" is amended with this storage requirement. (perf §6.3, §5.2) | §12.4 | `tree_100k_nodes_flatten` — allocs **< 10 × viewport** per toggle; `tree_100k_nodes_render` — allocs/frame independent of node count; `key_tree_toggle_10k` |
| 9 | **`CodeEditor` uses an edit-counter cache and a sorted-span cursor.** The highlight cache lives in `ui.cache::<HighlightCache>(id)` (R8, §21 item 2) and is keyed on the monotonically incremented edit counter held in `CodeEditorState`, never on re-hashing the document per frame; spans, diagnostics and find matches are stored sorted and consumed by a cursor advanced alongside the grapheme walk — O(graphemes + spans), not O(graphemes × spans). The seven per-frame clones are structurally impossible because `draw` takes `&self` and reads the state it needs by reference. §14.1's `CodeEditor` row is amended accordingly. (perf §6.3-2) | §14.1 | `frame_tablepro_query_editor_2k_lines` (§16.6 addition) — **< 40 allocs/frame**, ns scaling with viewport not document length |
| 10 | **`Capsule` never clones a viewport per frame.** With caller-owned `ViewportState`, jackin renders directly from the daemon's pane; `TextViewport::set_area`/`prime` and the `inert` clone dance are deleted. The per-frame `pane.term.clone()` — the single worst path in the repository — has no replacement because it has no reason to exist. (perf §8-1) | §12.4, §18.3 #23 | `frame_jackin_capsule_4panes_120x40` — **< 200 allocs/frame**; `capsule_pane_clone_4x2000` **is deleted** |
| 11 | **TablePro's grid load is one owned conversion.** The three-copy chain (`db::rows` → projection clone → `to_cell` clone) collapses to a single owned `ResultSet` that the `GridModel` borrows; cells are **pre-formatted at load**: `ResultSet { text: Vec<String>, kind: Vec<CellKind>, … }` holds one `String` per cell produced once (6 000 for the 500×12 benchmark), `CellRef<'a> { text: &'a str, tone: Option<Role>, align: Align }` borrows it, and `CellValue` survives only in the domain model for SQL generation and validation — so `sample_widths` measures `&str` and never formats (review §5; the `CellValue::display_width` clause is deleted). (perf §8-4, §6.3-4) | §12.3 | `grid_500x12_load` — **< 8 000** allocations (from ≈36 000) |
| 12 | **`Cx::intents` does not scan the queue and does not probe when it is empty.** It returns an empty iterator without probing when the queue is empty (a single `bool` check) and, when non-empty, probes a `[u64; N]` open-addressed table of the ≤ 8 owners that actually have intents; a frame with no input performs zero probes and the cost is O(intents), never O(components × intents). (R6, B14) | §3.3 step 7 | `intents_drain_is_o_1_when_the_queue_is_empty` (§16.6 addition, renamed) |
| 13 | **`KeySet` has an inverted representation.** `KeySet::AllExcept(set)` so "select all" over 100 000 rows does not materialise 100 000 `ItemKey`s; `ListAction::ToggledAll` reports the intent and the caller may keep its own bitmap. (R7) | §6.1, §12.2 | `list_100k_select_all` (§16.6 addition) — **< 100** allocations |
| 14 | **The backdrop dim resolves as a `StylePatch` and walks only the covered rect.** It runs through the same `Resolved` path as everything else, so a monochrome or no-colour theme gets it for free, and it never walks cells the layer does not cover. §11.6's "`backdrop` recipe keyed on `Role`" is amended with the rect restriction. (perf §6.3-3) | §11.6, §9.1 | `style_backdrop_full_screen_120x40` — 0 allocations, ns ≤ 1× the pre-refactor baseline |
| 15 | **Jackin's manager rebuilds rows on world change only.** `build_rows`/`build_detail`/`rebuild_actions` are gated by a world generation counter and never run from `draw` (structurally: `draw` is `&self`) nor from `on_key` before the key is examined. (perf §8-7) | §18.3, §3.4 | `key_jackin_manager_move` — **0 allocs/key**; `frame_jackin_manager_100rows_120x40` — **< 60 allocs/frame** |
| 16 | **`inert_below` suppresses background registration.** A modal layer with `inert_below: true` stops the page beneath it from registering ring entries, hit regions and cursor writes, so an open dialog does not pay the full background registration cost every frame. §9.1's `inert_below` is amended from "no interaction" to "no *registration*". | §9.1 | `frame_showcase_dialog_open` — `hits` **< 25 %** of `frame_showcase_lists_120x40` |

**WP‑0 findings (recorded as decisions).** **P-A** — `TextViewport::render` re-lays out the whole buffer twice per frame (once at `width`, once at `width − 1`), so `viewport_100k_lines_render` (15 177 816 allocs/frame in `tests/perf_baseline.txt`) is the binding acceptance for item 7; "allocs/frame independent of buffer size" is unreachable until the windowed `ui.cache` layout lands. **P-B** — debug and release differ by exactly one optimizer-elided 3-byte allocation, so `debug_and_release_alloc_counts_match` tolerates a difference of **≤ 1 allocation**; the reason is recorded here so the tolerance is never widened silently.

**Sequencing obligation.** The benchmark harness and the checked-in baseline landed **first**, at commit `07cb2c9` against the pre-refactor tree (Appendix A, WP‑0). Without that commit, "before and after" in goal §25.6 and §30 item 13 is not literal and none of the thresholds above can be asserted.

### 20.10 Intentional visual changes

Every item below changes rendered output relative to the reviewed baseline. Each is deliberate, is justified against `DESIGN.md` or a demonstrated defect (authority order, goal §3), and each names how it is reviewed. Nothing on this list may be regenerated into a baseline without an entry in `docs/visual-changes.md` (enforced by `xtask bless-guard`, §16.3).

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** (§11.4). At `ColorLevel::Mono` every state gains a symbol or modifier: focus gutter bar + bold label, marker glyphs for selected/checked, explicit reverse + `BOLD` + `PressLeft`/`PressRight` brackets for pressed (never the terminal `REVERSE` attribute; §21 item 25), faint + no marker for disabled, trailing error glyph + underline, dirty glyph for warning, underline + hardware cursor for editing, spinner for busy, active rule + bold for tabs. | **[F]** RES §1.2: mono currently collapses accent (mean 126) and error (mean 122) onto the same grey, so state is unreadable. goal §15 requires state meaning to survive without colour. | `conformance::<component>::mono_states_are_distinguishable` for every component; `render::components::*_mono` digests; capture matrix `tools/capture.sh` with `NO_COLOR=1` at 120×40 for showcase, tablepro and jackin, reviewed side by side against the truecolor capture by a fresh `opus-analyst` visual reviewer |
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
| 15 | **Focus-ring composition changes in migrated screens** <!-- amended by §21 item 33 -->. `Field` chrome, `NavList`, `scroll_region` parts and disabled-but-registered entries change the ring's size and order; tests such as showcase's `assert_eq!(seen.len(), 8)` will see a different count. | Not a defect: the ring is now built by the library from the same registration rules everywhere (§8.1). Ring size is an observable, so it is classified like a pixel. | Per affected test, an entry in `docs/visual-changes.md` (item 15) recording old count, new count and the reason **before** the expected value is edited; `Harness::ring().reachable()` listings attached; Slice 2 acceptance condition 8 |

**Not on this list, and therefore regressions if they appear:** any change to Junie token values; any change to spacing, glyph or border-set output under `Theme::junie()` at truecolor; any change to padding or ellipsis placement caused by replacing `fit`/`truncate` with the `RowUi` grapheme painter (the painter must be byte-identical to `fit` for every input — asserted by `render::components::*` digests and by a dedicated differential test `text::row_ui_matches_fit_for_every_fixture`); any change to the exact minimum-size copy strings; any change to the eight jackin scenario contracts, the rain timing constants, or the `format_universe_duration` wording.

---

## Appendix A — Slice plan

Maps goal §27 slices 3–8 onto work packages with **disjoint file ownership**, so `fable-builder` agents can run in parallel without integration conflict. A package's owner is the only agent that writes its files during that slice. Files not listed are owned by nobody and must not be touched.

**Amendment to goal §27 (recorded, with justification).** Goal §27 Slice 4 says "migrate coherent families, continuously updating showcase pages and tests". Continuous showcase updates would make every Slice-4 owner write into `apps/showcase/`, destroying disjointness. Instead: **Slice 4 owners do not touch any application.** Each family package proves itself with unit tests, a `Conformance` registration (which runs the full 20-case matrix), a `render::components::*` digest, and one `crates/tui/examples/` file. Showcase migration is entirely Slice 5. The review cadence goal §27 asks for is preserved: a fresh read-only `opus-analyst` reviews API consistency after **each** family package lands, before the next depends on it.

### WP‑0 — Performance and visual baseline (blocking prerequisite, before Slice 3) <!-- amended by §21 item 31 -->

* **Landed** at commit `07cb2c9` ("test: add performance baseline harness and pre-refactor numbers"), in the **existing root package**, written against the current single-package tree. Actual files (the review's provisional `tests/perf_support.rs` is `tests/perf_common.rs`):
  * `tests/perf.rs` — the library and cross-app benchmarks; the single `#[global_allocator]`.
  * `tests/perf_common.rs` — the `Counting` allocator shim, `ALLOCS`/`BYTES`, `bench`, `Stats`, `report`.
  * `tests/perf_baseline.txt` — 44 lines, one `name ns allocs bytes …` line each; the "before" numbers of §16.6.
  * `src/bin/showcase/perf_tests.rs`, `src/bin/tablepro/perf_tests.rs`, `src/bin/jackin_preview/perf_tests.rs` — the app benchmarks, hooked from each `main.rs` (two lines each).
* **Still owed before Slice 3** (not in `07cb2c9`): the two pre-refactor app digests for §20.10‑14 (`tests/baselines/` on the current tree; they move to `apps/*/tests/baselines/` with their apps) and `.github/workflows/perf.yml`.
* **Moves.** Slice 3 moves `tests/perf_common.rs` to `crates/tui-testing/src/perf.rs` and the library benchmarks to `crates/tui/tests/perf.rs`; the app benchmarks stay at the root until Slices 5–7 move them with their apps. `perf_baseline.txt` moves with the harness and its line names never change, so "before/after" stays literal.
* **Findings from the run** are recorded in §20.9 (P‑A, P‑B).
* **Gate:** `cargo test --test perf --release -- --test-threads=1` green; `perf_baseline.txt` committed; `PERF` lines archived as a build artefact.
* **Dependency:** everything. No refactor commit lands before this one.

### Slice 3 — Foundations (one owner, serial) <!-- amended by §21 items 31, 34 -->

* **Owner:** one builder. **Files:** the whole of `crates/tui/src/` except `components/`, plus the workspace skeleton, the test crate and `xtask`.
  `Cargo.toml` (workspace root **and** the unchanged root package), `crates/tui/Cargo.toml`, `crates/tui/src/{lib.rs, author.rs, id.rs, event.rs, intent.rs, response.rs, keymap.rs, focus.rs, hit.rs, capture.rs, scroll.rs, cursor.rs, layer.rs, runtime.rs, diagnostics.rs, layout.rs, measure.rs}`, `crates/tui/src/ui/**`, `crates/tui/src/text/**`, `crates/tui/src/theme/**`, `crates/tui/src/collection/**`, `crates/tui-testing/src/**`, `crates/tui/tests/{conformance.rs, architecture.rs, render.rs, perf.rs}` (skeletons), `crates/tui/tests/fixtures/text.rs`, `xtask/**` (`doc-check`, `bless-guard`, the boundary checks).
* **Order inside the slice:** identity → events/response/intents → registry/focus/capture/cursor → layers → surface/`Ui`/`Cx` → theme tokens/patch/recipe/resolve/downgrade → layout/measure → text/editor → collection vocabulary (`ItemKey`, `Reconcile`, `RowUi`, `RowDecor`, `EmptyState`, `ScrollRegion`) → `Runtime`/`App`/`run` → `author` module → `Harness` + `Conformance` driver + digest driver → `xtask doc-check`.
* **Staging (review §7; replaces "Applications do not compile during this slice").** The repository stays compiling and tested throughout (goal §27). The root package `junie-tui` stays exactly as it is — package name, `default-run = "showcase"`, `[lib] name = "junie_tui"`, the three `[[bin]]`s, `src/`, `src/bin/*` and all 198 existing tests — and the root `Cargo.toml` additionally becomes the workspace root:

  ```toml
  [workspace]
  members = ["crates/tui", "crates/tui-testing", "xtask"]
  # the root package itself is automatically a member
  [package]
  name = "junie-tui"          # unchanged
  default-run = "showcase"    # unchanged
  [lib] name = "junie_tui"    # unchanged — the three apps compile untouched
  ```

  The new library is built at `crates/tui` under a **temporary** name for Slices 3–4 only:

  ```toml
  [package] name = "tui-next"
  [lib]     name = "tui_next"
  ```

  Consequences: no duplicate lib name ever exists; `cargo doc --workspace` never collides; `default-run`, the three `[[bin]]`s, `tools/capture.sh`'s `BIN` and all 198 tests are untouched; `cargo test --workspace` runs both trees. `crates/tui-testing` (package name `junie-tui-testing`, unchanged) and `xtask` depend on `tui-next` during Slices 3–4. Gate commands in Slices 3–4 therefore say `-p tui-next`; from Slice 5 they say `-p junie-tui`.

  **Start of Slice 5, one commit, no behaviour change:** delete the root package's `[lib]`, `src/`, `src/bin/*`, `default-run` and its `[[bin]]`s as the apps move to `apps/*`; then rename `tui-next` → `junie-tui` / `tui_next` → `junie_tui` by scripted `sed` over a closed, slice-owned file set (`crates/tui/**`, `crates/tui-testing/**`, `xtask/**`, `crates/tui/examples/**`, `crates/tui/tests/**`). Re-run the full Slice-4 gate. Apps' `use junie_tui::…` lines are then written **once**, in their own migration slice, where the diff belongs.

  **Rejected staging** (review §7): renaming the root package to `junie-tui-legacy` inside one workspace — (1) doc-target collision: two crates with `[lib] name = "junie_tui"` both write `target/doc/junie_tui/`, fatal under `RUSTDOCFLAGS="-D warnings"`, and avoiding it by renaming the legacy lib rewrites every `use junie_tui::` in 55K lines of apps twice; (2) duplicate `showcase`/`tablepro`/`jackin-preview` bin names from Slice 5 make `cargo run --bin showcase` ambiguous and `target/debug/showcase` whichever package built last, silently corrupting `tools/capture.sh`; (3) `default-run` is a package key with no workspace equivalent, so `cargo run` at the root would resolve to the legacy showcase for Slices 3–7. Also rejected: keeping `junie-tui` on the new crate and renaming the root package's lib — the root binaries reference their own lib by its `[lib] name`, so that rename also rewrites all three apps twice.

  **Five recorded risks** (mirror them in `REFACTORING_STATE.md` so a resumed or compacted session does not "correct" the temporary name):
  1. The name `tui-next` is deliberate and temporary; the rename is a scheduled Slice-5 step, not a defect.
  2. Appendix B.4's `junie_tui::author` paths and every `architecture::*` test name are written against the final name; during Slices 3–4 they read `tui_next::author`. One-line mapping: `junie_tui` ⇢ `tui_next`, `-p junie-tui` ⇢ `-p tui-next`, until the Slice-5 rename commit.
  3. `xtask` and `crates/tui-testing` depend on the temporary name and their dependency lines are rewritten in the same commit.
  4. `tools/capture.sh`'s `BIN=${BIN:-target/debug/junie-tui}` is **[F]** already stale (no binary of that name exists); it changes to `target/debug/showcase` in Slice 5 exactly as Appendix B.5 schedules — independent of the rename.
  5. WP‑0 landed in the root package (paths above); Slice 3 moves the harness to `crates/tui-testing/src/perf.rs` and the library benchmarks to `crates/tui/tests/perf.rs`, keeping the app benchmarks at the root until Slices 5–7 move them with their apps. `perf_baseline.txt` moves with the harness and its line names never change.

* `crates/tui/examples/12_author_component.rs` is written here as the first consumer, proving the `author` surface is complete before any component depends on it.
* **Gate:**
  ```bash
  # tui-next is the temporary Slice 3–4 name of crates/tui (§21 item 31); junie-tui from Slice 5
  cargo fmt --all --check
  cargo clippy -p tui-next -p junie-tui-testing --all-targets --all-features -- -D warnings
  cargo test -p tui-next -p junie-tui-testing --all-targets --all-features
  cargo test -p tui-next --doc
  RUSTDOCFLAGS="-D warnings" cargo doc -p tui-next --all-features --no-deps
  cargo build -p tui-next --examples
  cargo test -p tui-next --test architecture
  cargo test -p tui-next --test perf --release -- --test-threads=1
  cargo run -p xtask -- doc-check                   # §21 item 34
  cargo test --all-targets                          # the legacy root package: all 198 existing tests stay green (M30)
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
  # tui-next is the temporary Slice 3–4 name of crates/tui (§21 item 31)
  cargo fmt --all --check
  cargo clippy -p tui-next --all-targets --all-features -- -D warnings
  cargo test -p tui-next --lib
  cargo test -p tui-next --test conformance
  cargo test -p tui-next --test render
  cargo test -p tui-next --test architecture
  cargo test -p tui-next --doc
  cargo build -p tui-next --examples
  cargo test -p tui-next --test perf --release -- --test-threads=1
  cargo run -p xtask -- doc-check                   # §21 item 34
  cargo test --all-targets                          # the legacy root package stays green (M30)
  ```
  Every component in the package must appear in `conformance_suite!` and pass all 20 applicable cases. After each package, a fresh read-only `opus-analyst` reviews API consistency against §13; the coordinator applies verified corrections before the next wave.

### Slice 5 — Showcase (one owner)

* **Files:** `apps/showcase/**` in full (`Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/pages/*.rs` — all 22 — `src/data.rs`, `tests/app_tests.rs`, `tests/visual.rs`, `tests/baselines/showcase.txt`, `tests/perf.rs`).
* Deletes the shell sidebar, footer hint row, static-field renderer, button matrix, inspector panel and too-small screen in favour of library components (§18.3 #2–#7, #21). Adds the pages goal §22.1 requires: the state matrix per component, `Theme::paper()` coverage, scoped and per-instance override pages, the author-component page (example 12 rendered as a page), and deterministic navigation to every state for captures.
* Begins with the one-commit rename `tui-next` → `junie-tui` and the removal of the root package's `src/`, `src/bin/*`, `[lib]`, `default-run` and `[[bin]]`s (Slice 3 staging, §21 item 31); then adds `apps/showcase` (with its `[lib] showcase_app` + `[[bin]] showcase`, §21 item 23) to the workspace members. All 26 existing tests must pass with the §16.4 `Harness`. <!-- amended by §21 items 23, 31 -->
* **Gate:** the full §26 command set scoped to `-p junie-tui -p showcase`, plus `cargo run -p xtask -- doc-check`, plus `cargo run -p showcase` driven through `tools/capture.sh` at 80×24, 100×30, 120×40, 160×50 × {truecolor, 256, 16, mono} × {junie, paper}, with every capture inspected and every baseline difference classified against §20.10.

### Slice 6 — TablePro (one owner)

* **Files:** `apps/tablepro/**` in full, including the new `src/grid_model.rs` (the `GridModel`/`GridEditor` adapter carrying `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, validators, `cmp_cells`, insert/duplicate/delete/discard/undo, `primary`/`nullable`/`references`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar) and `src/filter_editor.rs`.
* DOM §1.6's 22-capability mapping is the migration checklist; each capability is ticked off against a retained or new test before the slice closes.
* **Gate:** the §26 set scoped to `-p junie-tui -p tablepro` plus `cargo run -p xtask -- doc-check`; all 21 existing tests green; `grid_500x12_load` and `frame_tablepro_grid_500x12_120x40` meet their §16.6 thresholds; `apps/tablepro/tests/baselines/tablepro.txt` regenerated once with every difference classified; captures of connection, editor, grid, tabs, dialog, menu, picker and results surfaces reviewed.

### Slice 7 — Jackin (one owner)

* **Files:** `apps/jackin-preview/**` in full, including the decomposition of `screens/modals.rs` (≈2 400 lines) into `screens/{file_browser.rs, op_flow.rs}` plus library `Form`/`Dialog`/`HelpOverlay` usage, and `rain.rs` rewritten onto `Role` + `Ui::dim_layer`.
* **Gate:** the §26 set scoped to `-p junie-tui -p jackin-preview` plus `cargo run -p xtask -- doc-check`; all 22 existing tests plus the `rain`/`arbiter`/`clock`/`scenario` unit tests green; the eight scenarios reachable; the determinism assertion (two `--frame 282` runs byte-identical) green; the secret-masking assertions green; `frame_jackin_capsule_4panes_120x40 < 200 allocs` and `capsule_pane_clone_4x2000` deleted; `apps/jackin-preview/tests/baselines/jackin.txt` regenerated with differences classified; host, settings, account/usage, launch, Capsule, menu, modal, tab, status-bar and responsive surfaces captured and reviewed.

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
  cargo run -p xtask -- doc-check
  cargo run -p showcase & cargo run -p tablepro & cargo run -p jackin-preview
  tools/capture.sh   # the full matrix, reviewed
  ```

**Dependency summary.** WP‑0 → Slice 3 → {4A,4B,4C,4E,4G} → {4D,4F,4H,4I} → Slice 5 → {Slice 6, Slice 7 — parallel, disjoint app trees} → Slice 8. Slices 6 and 7 may run concurrently because their file trees are disjoint and both depend only on the frozen library surface; if either needs a library change, the slice pauses, a fresh `opus-analyst` adjudicates, the decision is recorded in this document and `REFACTORING_STATE.md`, and the change lands as a small serial amendment before both resume.

---

## Appendix B — Package layout and crate naming (Adjudication F)

### B.1 Decision — the repository becomes a Cargo workspace; the library keeps the name `junie-tui`

**Workspace, not one package.** goal §9.5 asks for a *mechanically enforceable* boundary proving applications consume only supported public APIs. A single package cannot provide one: `pub(crate)` is visible to `src/bin/*` because the binaries are in the same crate, which is exactly how the three apps reach `HitRegistry`, `Focus` and `FocusRing` today. A workspace makes the boundary a property of the compiler rather than of a grep: an application literally cannot name a `pub(crate)` item. Every text check in §16.5 is therefore a *report*, and the enforcement is structural.

**Crate name: keep `junie-tui` (package) / `junie_tui` (lib).** Considered and rejected: renaming to a theme-neutral name such as `tui-components`.

**Staging of the name (§21 item 31, review §7).** During Slices 3–4 the package at `crates/tui` is **temporarily** `tui-next` / `tui_next`, because the root package keeps `junie-tui` / `junie_tui` (with `default-run`, its three `[[bin]]`s, `src/` and all 198 tests) so the repository stays compiling and tested throughout. The rename to the final name is one scripted commit at the start of Slice 5, when the root package's library and binaries are removed. The five risks of the temporary name are recorded in Appendix A (Slice 3) and mirrored in `REFACTORING_STATE.md`. Everything below describes the **final** layout (Slice 5 onward). <!-- amended by §21 item 31 -->

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
    ui/*.rs           # trybuild compile-fail cases (§21 item 28)

crates/tui-testing/                         # package junie-tui-testing, publish = false; depends on the library with features = ["testing"] (§21 item 24)
  Cargo.toml
  src/{lib.rs, harness.rs, digest.rs, perf.rs, conformance/mod.rs, conformance/driver.rs}

apps/showcase/            Cargo.toml  src/{lib.rs, main.rs, app.rs, data.rs, pages/*.rs}   # [lib] showcase_app + thin [[bin]] (§21 item 23)
                          tests/{app_tests.rs, visual.rs, perf.rs, baselines/showcase.txt}
apps/tablepro/            Cargo.toml  src/{lib.rs, main.rs, app.rs, workbench.rs, tabs.rs, connections.rs,
                                           grid_model.rs, filter_editor.rs, db.rs, model.rs, sql.rs}
                          tests/{app_tests.rs, visual.rs, perf.rs, baselines/tablepro.txt}
apps/jackin-preview/      Cargo.toml  src/{lib.rs, main.rs, app.rs, arbiter.rs, clock.rs, scenario.rs, rain.rs,
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

[lib]
name = "showcase_app"        # the whole app: App, PageId, NAV_ENTRIES, screen const Ids (§21 item 23)
path = "src/lib.rs"

[[bin]]
name = "showcase"            # binary name preserved (goal §21)
path = "src/main.rs"         # fn main() { showcase_app::run() }

[dependencies]
junie-tui.workspace = true
ratatui.workspace   = true

[dev-dependencies]
junie-tui-testing.workspace = true
```

`jackin-preview` sets `[[bin]] name = "jackin-preview"` with `path = "src/main.rs"`, preserving the hyphenated binary while the package directory stays readable. Its lib is `jackin_app`; tablepro's is `tablepro_app`.

### B.3 `pub` vs `pub(crate)` policy

1. **Two documented layers, one crate.** `junie_tui::*` is the application-author surface; `junie_tui::author::*` is the component-author surface. Both are `pub` and separately documented with a module-level rustdoc header stating who the audience is (§13). Nothing else is `pub`.
2. **`lib.rs` is a curated facade, not a `pub mod` list.** Every module is `pub(crate) mod`; `lib.rs` re-exports the named items. Adding a type to the public API is a deliberate line in `lib.rs`, reviewable in a diff. `pub use` globs are forbidden.
3. **No public fields on component types** (invariant S1). Public fields exist only on plain data records with no behaviour and no geometry: `LayerSpec`, `Dismiss`, `StylePatch`, `StateRule`, `PartRecipe`, `ColorTokens`, `DesignTokens` and their sub-structs, `Role`/`GlyphRole`/`Part`/`Family`/`Variant` newtypes' constants, `FocusEntry`, `Headroom`, `Insets`, `Size`, `Constraints`, `RowDecor`, `CellDecor`, `Binding`, `Hint`, `HintLayer`, `LayoutFacts`, `Capture`, `PartRef`, `Key`, `Chord`, `Mouse`, `FieldError`. `architecture::no_public_geometry_or_cache` enforces the geometry half.
4. **`#[non_exhaustive]`** on `LayerSpec` and `LayoutFacts` only, which are constructed through builders or by the runtime. <!-- amended by §21 item 8 --> `ColorTokens`, `DesignTokens`, `RowDecor` and `CellDecor` are **not** `#[non_exhaustive]`: example 2 builds a full `ColorTokens` literal from another crate, adapters build `RowDecor { marker: …, ..Default::default() }`, and §11.4 *wants* a new token to be a compile error. Recorded: adding a colour or design token is an intentional breaking change for downstream themes; `map_colors`'s exhaustive destructure is the mechanism.
5. **No `#[doc(hidden)]` public items.** If something must be reachable it is documented in `author`; if it must not, it is `pub(crate)`.
6. **`#![deny(missing_docs)]` and `#![forbid(unsafe_code)]`** at the top of `crates/tui/src/lib.rs`. The single `unsafe impl GlobalAlloc` lives in `crates/tui-testing`, carries a written safety rationale, and is covered by `debug_and_release_alloc_counts_match`.
7. **Applications export a test surface only.** Each app is a package with a `[lib]` (`showcase_app`, `tablepro_app`, `jackin_app`; `publish = false`) and a thin `[[bin]]` whose `main` calls `<app>::run()`. <!-- amended by §21 item 23 --> Integration tests in `tests/` link the lib and reach the app through its `pub` items — the app's `const Id`s, its `App` type and its screen enums; nothing else is `pub`. A binary-only package cannot host `tests/*.rs` (they link the library target), which is why the earlier "no `[lib]`" rule is struck. `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library` guards the boundary. This is the migration contract of §16.4 item 3.

### B.4 The `author` module

<!-- amended by §21 items 18, 19, 21, 22, 31 --> `junie_tui::author` is a re-export module, not a second implementation. (During Slices 3–4 the path reads `tui_next::author`, §21 item 31.) It is what example 12 and every downstream component author consumes, and it is the mechanical proof of Scenario G: if a component can be written with it, no private access is needed.

```rust
//! Component-author API. Everything needed to build a component that participates in
//! theme resolution, focus, hover, press, dispatch, hit testing, cursor output,
//! scrolling, overlays, capture, testing and visual capture — and nothing more.
pub mod author {
    // identity and parts
    pub use crate::id::{id, Id, ItemKey, Part, PartRef};
    // phases and plumbing
    pub use crate::ui::{Ui, Cx, FrameRead};
    pub use crate::intent::{Intent, IntentIter, Phase, FocusVia};
    pub use crate::response::{Response, Flow, Invalidate, StateFlags};
    pub use crate::event::{Input, Key, KeyCode, KeyModifiers, Chord, Mouse, MouseKind, Axis};
    // registration services
    pub use crate::focus::{Focusability, ScopeMode, ScopeId, FocusVis};
    pub use crate::hit::{Axes, Headroom, RegionKind, Hit};
    pub use crate::capture::Capture;
    pub use crate::layer::{LayerId, LayerKind, LayerSpec, Anchor, Side, CrossAlign,
                           Dismiss, Backdrop, LayerEvent, DismissReason};
    // theme resolution
    pub use crate::theme::{Theme, Family, Variant, Role, FgStep, SyntaxRole, MeterRole,
                           GlyphRole, Surface, StylePatch, Slot, StateRule, Overlay,
                           Resolved, Modifier, Density, ColorLevel, DesignTokens, Align, ScreenAlign};
    // layout and measurement
    pub use crate::layout::{self, Track, RowAlign, Insets, SplitModel};
    pub use crate::measure::{Measure, Size, Constraints};
    pub use crate::ui::LayoutFacts;
    // text
    pub use crate::text::{TextEditorCore, EditAction, EditOutcome, CursorPos,
                          width, wrap, fuzzy, truncate, truncate_middle};
    // collections
    pub use crate::collection::{RowUi, CellUi, ColumnsUi, RowDecor, CellDecor, ByIndex, DefaultRow, KeyFn, RowFn,
                                EmptyState, RowTotal, Reconciliation, Reconcile, SelectMode, KeySet, Status};
    // bindings and hints
    pub use crate::keymap::{Binding, Bindings, BindingState, KeyMap, KeyPhase, Hint, HintLayer};
    // errors and diagnostics
    pub use crate::{FieldError, Validate, NoValidate, Secret, SecretPolicy, FieldControl};   // LayoutError deleted (§21 item 19)
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

---

## 21. Adjudication J — Slice 2 review corrections

**Status:** Accepted. Source: `docs/reviews/slice2-architecture-review.md` (fresh read-only `opus-analyst`, goal §27 Slice 2). The review's verdict — *not ready as written; ready after the ordered edits* — and every finding in it (B1–B16, M1–M31, F1–F10, P1–P8, A1–A17, the §3 recommendations and the §7 staging adjudication) are accepted as-is. This section is the numbered changelog the review's §9(a) asked for: each item names the finding(s), the section(s) it amends, and the resulting normative text. Where an earlier section is edited inline, the edit carries `<!-- amended by §21 item N -->`. Nothing here reopens Adjudications A–I or Appendix B's workspace decision; three items (1–3) are narrow amendments to the accepted model and are labelled as such.

**Ordering.** Items 1–10 are the hard gate: Slice 3 does not start until Groups 1–2 are recorded here and mirrored in `REFACTORING_STATE.md`. Groups 3–6 land before the Slice 3 owner reaches the corresponding subsystem, and all of them before Slice 4 begins.

### Group 1 — Adjudicated amendments

**Item 1 — B3, B15: collection items and grid models move to the phase call.** *(adjudicated amendment)* Amends §12.1, §12.2, §12.3, §13, §17.0 A3/A7, §20.2, §20.3; rewrites examples 7, 8, 10, 11.

Decision: a props struct holds configuration and closures only. Every collection passes its items to each phase; `Grid` passes its model.

```rust
// every collection: List, Tabs, Picker, Tree, NavList, Steps, ChipBar, RadioGroup, Completion, FilterList, PropsList
pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction>;
pub fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect;
// Grid — `M` is a method parameter, not a props parameter
impl<'a> Grid<'a> {
    pub fn update<M: GridModel>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;
    pub fn draw<M: GridModel>(&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
}
// List<'a, T, K, R> then holds only: id, key fn, row fn, select_mode, empty, status, patches, slots
```

Rationale. B3: a props temporary holding `&self.docs` lives to the end of the statement, so `.on_action(|a| self.docs.retain(…))` is `E0502` on the same field; edition-2021 disjoint capture does not help. Moving the data to the phase call ends the borrow when `update` returns. B15: `GridEditor: &mut self` was unreachable through `&self` props holding `&'a M`, so the Slice-6 commit path was unimplementable. Both are the same shape as §17.0 A3's controlled value. §13's table gains the row *collection data — passed per phase, never held in props*. Rejected fallback: mandatory `into_action()` + `match` with a "drop the props before mutating the source" warning in §12.1.

Open for `opus-analyst` before Slice 4I (not decided here): whether `Grid::update` takes `M: GridEditor` with defaulted refusals on `GridEditor`, or two entry points (`update` / `update_editable`). Slice 3 does not depend on the answer.

**Item 2 — B5: `Ui::cache<T>(id)` and rendering rule R8.** *(adjudicated amendment)* Amends §5, §17.0 A2, §20.9-7/-8/-9, §16.5.

```rust
impl Ui<'_> {
    /// Derived, non-semantic per-component cache. Keyed by (Id, TypeId). Cleared on
    /// resize, theme change and generation gap. Never observable in `Response`,
    /// never compared by `draw_twice_leaves_state_equal`.
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T;
}
```

**R8** (added to §5): *`Ui::cache` is the only mutable state reachable from `draw`. Its contents must be a pure function of (props, state, area, theme); a component that reads a value from `cache` that is not derivable from those inputs is a bug.* New check `architecture::cache_types_are_derived_only` — a `syn` scan that no type used as `cache::<T>()` appears in a `Response` or an `XState`; `conformance::draw_twice_is_byte_identical` covers the behavioural half. Why it is needed: §20.9-7 (windowed `TextViewport` layout is a function of `area`, known only in `draw`), §20.9-8 (the incremental `Tree` flat index is consumed per draw), §20.9-9 (the `CodeEditor` highlight cache holds spans and is not `PartialEq`-meaningful), and `Grid`'s width-dependent column sampling all need a draw-time scratch that `FrameOut::layout` (which flows up, not sideways) cannot provide. Rejected alternative: interior mutability in `XState` — it breaks S2's `PartialEq` and conformance case 6.

**Item 3 — B6: Esc dismissal moves after `app.update`.** *(adjudicated amendment)* Amends §3.3 steps 5 and 8, §8.6, §9.1's Esc row, §16.1 `layer.rs`.

§3.3 step 5 keeps only Tab/Shift+Tab and press-focuses-owner. Step 8 becomes: *keys still unconsumed after step 7 are offered to (a) the app `KeyMap` "Bubble" bindings, then (b) `Dismiss.esc` on the top layer, then (c) the screen's Esc ladder.* A `TextInput` editing inside a modal `Form` therefore receives Esc first; `input::cancel_restores_the_snapshot` and `select::escape_closes_and_restores_the_cursor` become reachable inside a layer. `Intent::Cancel` keeps its meaning ("Esc reached this owner after layer dismissal"). New unit test `layer::esc_reaches_the_focused_editor_before_the_layer`.

### Group 2 — Compile-blocking API corrections

**Item 4 — B1, M15 and the review's §3 `Response` notes.** Amends §6.1, §13, §16.1 `response.rs`, §16.4 item 1.

```rust
pub fn is_consumed(&self) -> bool;
pub fn is_changed(&self) -> bool;      // invalidate >= Paint
```

The constructors `consumed()` / `changed()` stay (`changed` keeps its `Outcome`-era name so the ~60 assertion rewrites are mechanical). §16.4's `Outcome` mapping is three-way: `Outcome::Changed → .is_changed()`; `Outcome::Consumed → .is_consumed() && !.is_changed()`; `Outcome::Ignored → !.is_consumed()`. `BitOr` / `BitOrAssign` are implemented for `Response<()>` only — `flow`: Consumed wins; `invalidate`: max; `id` and `state`: lhs, documented as "the fold is a control-flow summary; read `state`/`id` from the individual responses". Composing two action-carrying responses is a type error, never silent loss; every §17 `r |=` operand is already `Response<()>`. `Response.id` is `Option<Id>` with `pub fn id(&self) -> Option<Id>` (`ignored()` has no id). §16.1: `response::bitor_keeps_the_first_action` (it asserted the rejected semantics) is replaced by `response::bitor_is_defined_only_for_unit` (compile-fail via `trybuild`).

**Item 5 — B2: three-impl-block collection builders.** Amends §12.2, §17.0 A7, §20.3.

```rust
pub struct ByIndex;      // default key: ItemKey::index(i) — UNSTABLE under reorder; call .key(…) for stable identity
pub struct DefaultRow;   // default row: the item's `Display` through RowUi::label_fmt, no allocation
pub trait KeyFn<T> { fn key(&self, item: &T, index: usize) -> ItemKey; }
impl<T, F: Fn(&T) -> ItemKey> KeyFn<T> for F { … }
impl<T> KeyFn<T> for ByIndex { … }
pub trait RowFn<T> { fn row(&self, item: &T, u: &mut RowUi<'_>); }
impl<T, F: Fn(&T, &mut RowUi<'_>)> RowFn<T> for F { … }
impl<T: core::fmt::Display> RowFn<T> for DefaultRow { … }

impl<'a, T> List<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }
impl<'a, T, K, R> List<'a, T, K, R> {
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2>;
    // …all non-generic config methods stay here, returning Self
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> List<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect;
}
```

Why: `key(self, k: K) -> List<'a, T, K, R>` took and returned the *same* `K` fixed by the impl header, so `List::new(…).key(|o| …)` could never typecheck, and `()` is not `Fn`. Applied identically to `Tabs`, `Picker`, `Tree`, `Grid` (columns only), `NavList`, `Steps`, `ChipBar`, `RadioGroup`, `Completion`, `FilterList`, `PropsList`. §12.2's unused `pub trait KeyFn<T>: Fn(&T) -> ItemKey {}` is replaced by the blanket-implemented traits above.

**Item 6 — B4, B14: `Cx<'f>` splits the frozen intent queue from its mutable services.** Amends §3.3 steps 1, 6, 7 and the §25.6 note; §17.0 A2; §20.9-12; §16.6.

```rust
pub struct Cx<'f> { /* intents: &'f IntentQueue (shared, frozen during step 7); services: &'f mut … */ }
impl<'f> Cx<'f> {
    /// Borrows only the frozen queue. Marks this owner's bucket drained (interior Cell flag).
    pub fn intents(&self, id: Id) -> IntentIter<'f>;
}
```

The queue is built in step 6 and immutable for the whole of step 7; "drained" is recorded through a `Cell<bool>` per bucket, so `for it in cx.intents(id) { … cx.close_layer(…) … }` compiles (`Dialog`, `SplitPane`, `List`, `Select` all touch `cx` inside the loop). `Intent::Paste(&'f str)` therefore requires the paste text to live in a runtime-owned frame arena for the whole of step 7, not in the `Input` value (§3.3 step 1). The name is unified to `Cx::intents`; `Intents::take(id)` and the type name `Intents` are struck. B14: `Cx::intents` returns an empty iterator without probing when the queue is empty (a single `bool` check) and, when non-empty, probes a `[u64; N]` open-addressed table of the ≤ 8 owners that actually have intents; a frame with no input performs zero probes. §16.6's `intents_drain_scales_with_intents_not_components` (false by construction — 500 components made 500 probes) is renamed `intents_drain_is_o_1_when_the_queue_is_empty` with the threshold *a 500-component frame with 0 intents costs the same as a 20-component frame with 0 intents ±10 %; with 2 intents, total probe cost is ≤ 500 × 5 ns and allocations are 0*.

**Item 7 — B10: `FieldControl` and an id-less `Field`.** Amends §15, §17.0 A7; rewrites examples 6 and 11.

```rust
/// Draw-time chrome only. `Field` never registers a focus stop and never runs `update`;
/// the control keeps its own `Id` and its own `update`.
pub trait FieldControl {
    type State;
    fn id(&self) -> Id;
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
pub struct Field<'a, C: FieldControl> { /* label, required, optional_suffix, help, error, plain, control: C */ }
impl<'a, C: FieldControl> Field<'a, C> {
    pub fn new(label: &'a str, control: C) -> Self;      // no Id — the control owns identity
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect;
}
```

Why: `Field<'a, C>` had no bound and could not forward a state type; and `Field::new(EMAIL, "Email", TextInput::new(EMAIL))` registered the same `Id` twice in one frame (`Diagnostic::DuplicateId`, failing `id_separator_collision_free` for every field). `Field`'s chrome parts (`GUTTER`, `LABEL`, `MARKER`, `HELP`) are registered as `Decorative` regions under the **control's** id (`FieldControl::id`), so one id exists per field. §17.0 A7's "update/draw forward to the control" comment is struck; `update` is always called on the control directly because it needs the controlled value.

**Item 8 — B8: `#[non_exhaustive]` and `LayerSpec` builders.** Amends Appendix B.3 item 4, §17.0 A6; rewrites example 10 as a popover.

`ColorTokens`, `DesignTokens`, `RowDecor`, `CellDecor` are removed from B.3 item 4 (`#[non_exhaustive]` forbids the struct literal in example 2, the `..base` update in example 10, and the `RowDecor { marker: …, ..Default::default() }` TablePro adapters need). Recorded: *adding a colour or design token is an intentional breaking change for downstream themes; `map_colors`'s exhaustive destructure is the mechanism (§11.4).* `LayerSpec` and `LayoutFacts` stay `#[non_exhaustive]`; `LayerSpec` gains const consuming builders:

```rust
impl LayerSpec {
    pub const fn modal(owner: Id) -> LayerSpec;
    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec;
    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec;
    pub const fn anchor(self, a: Anchor) -> Self;
    pub const fn dismiss(self, d: Dismiss) -> Self;
    pub const fn backdrop(self, b: Backdrop) -> Self;
    pub const fn initial_focus(self, id: Id) -> Self;
    pub const fn min_size(self, w: u16, h: u16) -> Self;
    pub const fn inert_below(self, yes: bool) -> Self;
    pub const fn restore_focus(self, yes: bool) -> Self;
}
impl Dismiss { pub const NONE: Dismiss; pub const ESC: Dismiss; pub const ESC_AND_OUTSIDE: Dismiss; pub const ALL: Dismiss; }
```

Example 10 uses `LayerSpec::popover(OWNER_PICK, Anchor::Rect { … }).dismiss(Dismiss::ESC_AND_OUTSIDE)`, which also removes the visual bug of a full-screen dim behind a dropdown-shaped picker.

**Item 9 — M24: `App::keymap`.** Amends §17.0 A1. `fn keymap(&self) -> &KeyMap { KeyMap::EMPTY_REF }` with `impl KeyMap { pub const EMPTY: KeyMap; pub const EMPTY_REF: &'static KeyMap = &KeyMap::EMPTY; }` (`KeyMap::empty()` returned a reference to a temporary).

**Item 10 — M11: `Bindings::Cmd` is separate from the emitted action.** Amends §13.1, §16.2 (`Conformance::bindings`, case 20), example 12.

```rust
pub struct Binding<C: 'static> { pub chord: Chord, pub cmd: C, pub label: &'static str, pub priority: u8, pub visible: bool }
pub trait Bindings {
    type Cmd: Copy + 'static;                       // const-constructible: Next, Prev, Activate, Close…
    fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>];
}
```

`update` maps `Cmd` → `XAction` with the live key (`ListAction::Chose(ItemKey)` is produced from `ListCmd::Choose` plus the cursor key). Why: `ListAction::Chose(ItemKey)`, `TabsAction::Close(ItemKey)` and `GridAction::Sort(ColumnKey, SortDir)` are runtime values and cannot appear in a `const` table. Example 12 declares `SegCmd { Prev, Next, Select }` and matches keys through `BINDINGS` itself.

### Group 3 — Model and ordering specifications

**Item 11 — B7: focus re-run semantics.** Amends §3.3 step 7; §16.2 suite-level list.

Appended to step 7: *A re-run enqueues only `Intent::FocusOut{to}` and `Intent::FocusIn{via}`; already-drained buckets are never refilled, so no input intent is delivered twice. The `Response` of each pass is folded into the first with `|`. If a 5th pass is required, the runtime emits `Diagnostic::FocusTransitionDidNotSettle`, applies the pending `FocusOut` **and** the matching `FocusIn` to the last requested target without re-running `app.update`, and continues.* `conformance::focus_transition_settles` is a suite-level test (it already sits in §16.2's suite-level list; it is never emitted per component) and asserts the diagnostic count is 0.

**Item 12 — B11: `Registry::hit` ignores layers; the runtime compares.** Amends §3.3 step 3, §8.6, §9.1 table, §16.1 `hit.rs`, §17.0 A8.

*`Registry::hit(pos)` returns the topmost region covering `pos` regardless of layer. The runtime then delivers the intent iff `hit.layer == top_layer`, and treats `hit.layer < top_layer` (or `None`) as an outside-click for the top layer's `Dismiss.outside_click`.* Otherwise §8.6's "real outside test" degraded to "the hit returned None". `hit::higher_layer_shadows_lower` is kept (a higher-layer region covering the same point still wins); `hit::hit_returns_a_lower_layer_region_for_the_outside_click_test` is added.

**Item 13 — B12: `RegionKind` and the meaning of `UndeliveredIntent`.** Amends §3.3 steps 9 and 11, §16.4, §17.0 A2/A8.

```rust
pub enum RegionKind {
    Control,     // focusable, delivers intents
    Part,        // sub-region of a Control; delivers to the Control's owner
    Scroll,      // wheel target only
    Decorative,  // paints and answers area_of; never delivers, never diagnosed
}
```

*An intent whose resolved owner registered only `Decorative` regions is discarded silently. `UndeliveredIntent` is recorded only when the owner registered a `Control` or `Part` region and drained nothing.* Containers (`Panel`, `Dialog`'s `CONTAINER`/`BORDER`/`BACKDROP`, `SplitPane`'s panes, `Form`, `Grid`'s header, `Field`'s chrome) register through `Ui::register_decor`; §16.4's zero-diagnostics journey test stands.

**Item 14 — M4: `LayerId` is assigned at `open_layer`.** Appended to §9.1: *`LayerId` is assigned monotonically by `Cx::open_layer` and is the stack position. `Ui::layer(id, f)` resolves `id` to its already-assigned `LayerId`, executes `f` into that layer's pooled buffer, and returns `None` without executing `f` if `id` is not open. Call order at draw time has no effect on z-order, hit filtering, or focus scope nesting.* Tests: `render::overlay::layer_composites_bottom_to_top_regardless_of_call_order` (existing) and `layer::layer_id_is_assigned_at_open_not_at_draw` (new).

**Item 15 — M29: cursor rejection under `inert_below`; focus-restore staging.** Amends §8.4, §3.3 step 14, §16.2 case 17, §16.1 `focus.rs`.

(a) §8.4: *a `set_cursor` from a suppressed (inert) layer is discarded silently; `CursorRejected` is recorded only for a non-inert lower layer or an unfocused owner.* Case 17 is exercised with a `Popover` (pointer barrier only, no `inert_below`). (b) §3.3 step 14: *focus restoration is staged at `close_layer` and applied at the next draw's reconcile. Until then, `FocusState::current` is the restore target and key resolution uses it even though it is absent from the last ring; this is the one documented exception to "resolve against last frame".* New test `focus::restore_target_receives_keys_before_the_next_draw`.

**Item 16 — M3, M5: `Ui::layer` arity; `Id::part` versus `PartRef`.** Amends §3.3 step 12, §7.1, §16.4 item 3, §17.0 A8.

`ui.layer(id, |ui, area| …)` — the `spec` is supplied by `cx.open_layer` in `update` only. Rule: *`Id::part(p)` mints a child component id (a `Button` inside a `Dialog`); that component registers its own `Control` region and is found by `Runtime::area_of`. `PartRef` tags a sub-region of a single component (a tab's close glyph, a scrollbar thumb) and is found by `Runtime::area_of_part`. They are never interchangeable.* Declared: `impl PartRef { pub const fn of(p: Part) -> Self; pub const fn item(p: Part, k: ItemKey) -> Self; }`. §16.4 item 3 no longer presents the two lookups as equivalent.

**Item 17 — F7, F8, F9, F10: one-line specifications.** Amends §16.4, §3.3 steps 7 and 13, §8.3, §9.1.

F7 (not a bug): `Harness::new` draws and `Harness::handle` draws after every input, so a test never observes the one-frame pointer latency; `Harness::click_id` on an id whose `area_of` is `None` returns `Response::ignored()` and records `Diagnostic::UnaddressableId`, never panics. F8: captures are released at the end of step 7 as well as at step 13 whenever `close_layer` removed the capture owner's layer. F9: with `inert_below`, no scroll region is registered beneath the layer, so a wheel over the backdrop falls through to the app bubble phase — no outward chaining is added. F10: a second `ui.layer` call with the same `id` in one frame returns `None` and records `Diagnostic::DuplicateLayerDraw`.

### Group 4 — Undeclared types and members

**Item 18 — M1, M2, and the review's §3 `FrameRead`.** Amends §17.0 A2, §8.2, §3.3 step 7, §3.4, §8.5.

Declared: `Ui::full()`, `Ui::register_part(owner, PartRef, Rect)`, `Ui::register_decor(owner, PartRef, Rect)`, `Cx::focus(id)`, `Cx::request_repaint()`, `Cx::request_repaint_after(Duration)`, `Cx::capture(owner, part) -> bool` (a `Cx` has no "current component"; `intents`/`area` already take the id explicitly), `Cx::capture_owner() -> Option<Id>`, and the shared read trait `FrameRead { state, theme, design, area, layout }` implemented for both `Ui` and `Cx` and re-exported from `author` — one vocabulary, two capability sets. `HintCtx` is deleted; `Screen::hints` is `fn hints(&self, w: &World) -> HintLayer`. The painting methods of R3 get exact signatures in A2.

**Item 19 — M6: declared and deleted types.** Amends §17.0 (new A8, A9), §11.2 `Capability`, §13, B.4.

Declared in A8: `Status`, `KeySet`, `ColorLevel`, `BindingState`, `Backdrop`, `Density`, `SortDir`, `Hit`, `RegionKind`, `KeyPhase`, `SyntaxTokens::derive`, `MeterTokens::derive`, `Chord::key`, `Chord::with`, `Key::is`, `Key::chord`, `Column`, `CodeDiagnostic`; in A9: `Diagnostic` (seven variants). Deleted: `UnicodeLevel` (`Capability { color: ColorLevel }` only), `LayoutError` (§13 errors row, B.4), `AppActionRecord` and `Harness::actions` — replaced by `#[cfg(feature = "testing")] Cx::record(&'static str)` and `Runtime::records()` / `Harness::records()`.

**Item 20 — M7: `ScreenAlign` versus `Align`; `LayerSpec.min_size`.** `pub enum ScreenAlign { Center, UpperThird, Bottom }` is the `Anchor::Screen` payload; `pub enum Align { Left, Center, Right }` is text alignment (`StylePatch.align`, `CellUi::align`, `Column.align`). `LayerSpec.min_size` is `(u16, u16)`, not `Size` (a min-size whose type is a min/preferred pair).

**Item 21 — M8, M9, M10.** Amends §17.0 A5, §12.2, §4 S5.

`ThemeBuilder` gains `focus(Color)`, `selection(bg, fg)`, `highlight(bg, fg)`, `field(base, hover)`, `disabled(fg, bg)`. `CellUi` gets its exact impl (`text`, `num`, `money`, `align`, `tone`, `italic`, `suffix`, `patch`, each `&mut Self`; `num`/`money` format in place with 0 allocations). `RowUi::part(p, width)` *reserves `width` columns from the right of the remaining row space; `label` fills what is left*. `RowUi::label_fmt(fmt::Arguments)` is added for `DefaultRow`. `impl XState { fn reconcile(&mut self, keys: impl Iterator<…>) }` (an O(n) walk that defeats §20.9-3) is replaced by

```rust
/// Implemented on every collection state type (ListState, TreeState, TabsState, GridState, …).
pub trait Reconcile {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    fn invalidate(&mut self);     // caller mutated items in place without changing len/ends
}
```

which is what makes the `(len, key(first), key(last))` stamp and the cached-index probe expressible.

**Item 22 — M28, A9, A12, A13, and the `Phase2` rename.** Amends §7.1, §12.1, §12.2, §13.1, §14.1, §18.1, §18.2, §20.9-5, B.4.

`Registry::names` is deleted: *the debug label travels with the `Id` (`Tail::Item(k)`), so no side table exists in any build; §7.1's "the runtime also maintains `Registry::names`" is struck; §20.9-5's `debug-ids` feature is unnecessary and `debug_and_release_alloc_counts_match` no longer has a names-table term.* `RowDecor<'a> { …, message: Option<&'a str> }` and `CellDecor<'a> { …, error: Option<&'a str> }` (no `'static` in a component surface, §2.2). §12.1's sentence becomes *`&'a dyn Fn` slots and small `&dyn Trait` extension points (`Validate`, `Highlighter`, `Segmenter`, `DiffSource`, `GridModel`) are the only dynamic dispatch in a component's public surface; none is boxed and none requires `'static`.* `Diagnostic'` is named `CodeDiagnostic`; `ColumnSpec'` is named `Column`. `Phase2` is renamed `KeyPhase` (review §3 naming; `Phase` stays the pointer phase). `Surface::Surface` is `DESIGN.md`-mandated and kept (noted in §20).

### Group 5 — Testing and migration contract

**Item 23 — B9: application packages have a `[lib]` and a thin `[[bin]]`.** Amends §16.4 item 3, Appendix B.2, B.3 item 7, §16.5.

```toml
# apps/showcase/Cargo.toml
[lib]
name = "showcase_app"
path = "src/lib.rs"          # the whole app: App, PageId, NAV_ENTRIES, screen const Ids

[[bin]]
name = "showcase"            # binary name preserved (goal §21)
path = "src/main.rs"         # fn main() { showcase_app::run() }
```

`tablepro` → `tablepro_app`, `jackin-preview` → `jackin_app`. Why: Cargo integration tests link the package's **library** target; a binary-only package has nothing to link and `use showcase::app::App;` cannot resolve (the existing suites need this reach — `app_tests.rs:124-129, :143, :156, :20-22`). `#[path]`/`include!` stay forbidden. `architecture::binary_names_are_preserved` is unaffected; new check `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library`.

**Item 24 — M13, M14: `Runtime` inspection behind the `testing` feature; `Harness` additions.** Amends §17.0 A1, §16.4.

§17.0 A1 gains `#[cfg(feature = "testing")] impl<A: App> Runtime<A> { hover, state_of, focus_visible, top_layer, is_open, cursor, resolved, last_invalidate, records }` — `crates/tui-testing` is a separate crate and can use only `pub` API. `Harness` gains `app() -> &A`, `app_mut() -> &mut A`, `resolved(id, Part) -> Resolved` (used by §16.4's theme-coupling paragraph but previously absent from the impl), `with_auto_draw(bool) -> Self` and `last_invalidate() -> Invalidate` (without them redraw suppression is unobservable because `handle` always draws), `records() -> &[&'static str]`. `crates/tui-testing` depends on the library with `features = ["testing"]`; §26's `--all-features` enables it everywhere, which is intended.

**Item 25 — B13: mono `PRESSED` gains a modifier and bracket glyphs.** Amends §11.4, §16.2 case 9, §20.10-1.

Option (a) of the review is chosen (legible in a real monochrome terminal, which is the point of §20.10-1). The mono `PRESSED` rule becomes `Part::CONTAINER bg = Role::Fg(Primary), fg = Role::Surface(Canvas), add(Modifier::BOLD)` **and** `Part::LABEL` is bracketed with `GlyphRole::PressLeft` / `GlyphRole::PressRight` (Junie: `[` and `]`, rendered `[Save]`). `GlyphRole` gains `PressLeft, PressRight`. Case 9 stays a `(symbol, modifier)` multiset comparison. When `SELECTED` and `CHECKED` are both live, `CHECKED` wins.

**Item 26 — B14.** Recorded under item 6.

**Item 27 — M16, M17, M26: conformance fixture, capability and list.** Amends §16.2.

`Fixture { disabled, read_only, theme, color, area, rows: Vec<FixtureRow> }` with `FixtureRow { key: ItemKey, label: String, meta: String, disabled: bool }`; `Conformance::update`/`draw` borrow `f.rows` (cases 2 and 12 need real keys and real data to permute); `type Action: PartialEq + core::fmt::Debug`; `type Cmd: Copy + 'static` with `fn bindings(s: BindingState) -> &'static [Binding<Self::Cmd>]`. `Caps::TYPES` (declared by any component whose focus entry sets `swallows_typing`) excludes bare `Char` chords from case 20's reverse direction — `TextInputCase`, `TextAreaCase`, `CodeEditorCase`, `PickerCase`, `FilterListCase` consume every printable `Char`. `ScrollbarCase` (a part, not a component), `EmptyCase` (a data enum rendered inside collections) and `SecretInputCase` (no such type) are removed from `conformance_suite!`; `PropsListCase` is added; the secret path is a `Caps::SECRET` fixture variant of `TextInputCase`.

**Item 28 — M18, M27.** Amends §16.5, §16.6, §16.3.

`architecture::every_named_test_exists` is one-directional and scoped: *every name listed in §16.1, §16.2's suite-level list and §16.4 exists in `cargo test --workspace -- --list`; §16.6 perf names are checked against `cargo test --workspace --test perf --release -- --list`; `trybuild` cases are checked against `tests/ui/*.rs` filenames; extra tests are allowed.* The prose-parsed deletion assertion for `capsule_pane_clone_4x2000` is replaced by line-absence in `perf_baseline.txt`. `Scene` (§16.3) owns a headless `FrameState` (registry + focus ring + layer stack + style stack) built from the theme, which is how it can construct a `Ui`.

**Item 29 — M21, M22, M23, A3, A6, A7: algorithms, corpora and their tests.** Amends §11.2, §11.4, §11.7, §16.1, §16.3, §20.10.

`ThemeBuilder::build` derivation (written into §11.2): given `surfaces[0]` and `accent`: `surfaces[1..4]` step L\* by +4 (dark base) or −4 (light base, detected by L\*(surfaces[0]) > 50); `fg[0..4]` step L\* by −18 from a contrast-7:1 anchor against `surfaces[0]`; `accent_hover = ΔL* +8`, `accent_pressed = ΔL* −8`, `accent_tint` = accent at 12 % over `surfaces[1]`; `focus = accent`, `focus_ring = accent_pressed`; `border_subtle = surfaces[3]`, `border_strong = fg[3]`; `danger/warning/success/info` tints at 12 %; `on_accent`/`on_danger` = whichever of `fg[0]`/`surfaces[0]` reaches ≥ 4.5:1. Every derived value is a pure function of the seeds. `Theme::paper()` is `from_tokens(seeds).builder()…build()` and is pinned by `theme::paper_tokens_are_pinned`. `downgrade_color` (§11.4): `nearest_256` = nearest in the 6×6×6 cube ∪ 24-step greyscale by squared sRGB distance, ties to the lower index; `nearest_16` = nearest of the 16 xterm defaults by CIE76 ΔE; `mono`: Y = 0.2126R + 0.7152G + 0.0722B, then Y < 0.35 → black, Y > 0.75 → white, else `Color::Reset`. Text corpus `crates/tui/tests/fixtures/text.rs`: ≥ 200 strings covering ASCII, CJK wide, combining marks, ZWJ emoji, RTL, and widths 0..=120. New tests: `theme::builder_derives_every_unset_token_deterministically`, `theme::derived_tokens_meet_design_contrast_ratios`, `theme::downgrade_is_deterministic_per_level`, `theme::paper_tokens_are_pinned`, `render::overrides::global_variant_override_changes_only_that_variant` (goal §15 scenario 5 had no test), `text::row_ui_matches_fit_for_every_fixture` (named in §20.10, now also in §16.1), `response::layout_is_strictly_greater_than_paint` (the only assertion on `Invalidate::Layout`, so no builder invents layout caching early).

**Item 30 — P1, P3–P8, the §20.9 restatements, and the remaining review §3/§8 items.** Amends §13, §13.1, §15, §20.2, §20.4, §20.9, §16.6, §5, §9.1, §12.3, §16.3.

* P1: `HintLayer { hints: SmallVec<[Hint; 8]>, badge: Option<&'static str>, status: Option<Cow<'static, str>>, centered: bool }`, cached in `Ui::cache` behind `(focus_id, StateFlags, top_layer)`. New perf test `frame_hintbar_derived` — **0 allocs/frame when focus is unchanged**.
* P3: `Ui` keeps a running `stack_hash: u64` updated on `with_overlay` push/pop (§20.9-2). P4: `Ui` is constructed once per `Runtime`/`Scene` and reused, never per frame (§20.9-2).
* P5: `Secret::masked_tail(&self, n) -> String` is replaced by `fn write_mask(&self, out: &mut CellUi<'_>, n: usize)`; the synthetic tail may be cached in `TextInputState` at `begin`.
* P6: `Runtime` keeps at most 64 `Diagnostic`s plus a dropped count, cleared at the start of each `handle`.
* P7: `Id` is ~48 B in debug (`DebugLabel`) versus 8 B in release, so every `Region` roughly doubles in debug; recorded in §20.4.
* P8: `frame_showcase_lists_120x40`'s "hits within ±10 %" becomes *hits recorded and classified in `docs/visual-changes.md`; no unexplained growth > 25 %* (`Field` chrome, `NavList`, `scroll_region` parts and disabled-but-registered entries change region counts materially).
* §20.9-7/-8/-9 are restated on `Ui::cache` (item 2); §20.9-11 commits to pre-formatting at load — `apps/tablepro/src/grid_model.rs` stores `ResultSet { text: Vec<String>, kind: Vec<CellKind>, … }` produced once (6 000 strings for 500×12, within `< 8 000`), `CellRef<'a> { text: &'a str, tone: Option<Role>, align: Align }`, `CellValue` survives only in the domain model, and the `CellValue::display_width` clause is deleted; §20.9-12 restated (item 6); §20.9-5 amended (item 22).
* Review §3, inline editors: a `Grid` cell's inline editor registers a `Control` region **after** the grid's cell `Part` region and therefore wins the click; the grid must not treat a click inside an active edit as a cursor move. New test `grid::click_inside_an_active_inline_edit_goes_to_the_editor`.
* Review §3, binding convention added to §13: *A component instance with any configuration beyond `new(id, …)` is built by exactly one private constructor function on the owning screen, called from both phases. The constructor takes the fields it needs as parameters, never `&self`, so `update` can still pass `&mut` to disjoint fields; a controlled `.value(&T)` added in `draw` is the documented per-phase difference.* `architecture::props_are_built_once` (a `syn` check that no `X::new(` appears more than once per screen module for the same `const Id`, ignoring unconfigured constructions) reports violations. `Form` (J2) provides `Form::field(id, …)` so a 15-field form declares each field once — its API sketch is the open research item at the end of this section.
* A4 (§5): *a component that declares a part must paint `Resolved.glyph` when `Some`; `conformance::registry::declared_parts_are_the_parts_actually_styled` checks the query, `mono_states_are_distinguishable` checks the paint.* A5: `#[cfg(feature = "testing")] Ui::styled_parts(&self) -> &[(Id, Part)]`. A8: `EditIntent::External` — *the component emits `EditRequested(item, col)` and does not begin an inline edit; the application opens its own editor.* A10 (§9.1): *`Dismiss.focus_out` is honoured only for `Popover` and `Tooltip`; a `Modal` traps focus so it can never fire.* A11: `.state_override(StateFlags)` is declared on component builders as a documented showcase/testing-only path with `architecture::state_override_is_used_only_in_apps_and_fixtures`. A14 (§16.3): the order is change → capture → classify → bless; `xtask bless-guard` runs in CI on the committed tree, not locally.

### Group 6 — Staging, traceability, gates

**Item 31 — B16, review §7, M30: staging.** Amends Appendix A (WP‑0, Slice 3, Slice 4 gates), Appendix B.1/B.2, §16.6.

The coordinator's `junie-tui-legacy` rename proposal is rejected (doc-target collision on `junie_tui` under `RUSTDOCFLAGS="-D warnings"`; duplicate `showcase`/`tablepro`/`jackin-preview` bin names making `cargo run --bin` and `target/debug/<bin>` ambiguous in the slices where visual comparison matters; `default-run` resolving to the legacy showcase). Accepted plan: the root package stays `junie-tui` untouched and becomes the workspace root; the new library at `crates/tui` is temporarily `tui-next`/`tui_next` during Slices 3–4; one scripted rename to `junie-tui`/`junie_tui` at the start of Slice 5 when root `src/`, `src/bin/*`, `[lib]`, `default-run` and the `[[bin]]`s are removed. Appendix A records the plan, the five risks and WP‑0's actual landed paths (commit `07cb2c9`: `tests/perf.rs`, `tests/perf_common.rs`, `tests/perf_baseline.txt`, `src/bin/{showcase,tablepro,jackin_preview}/perf_tests.rs`); the legacy test command `cargo test --all-targets` at the repository root is added to the Slice 3 and Slice 4 gates. `REFACTORING_STATE.md` must carry the same five risks (coordinator-owned; not part of this edit).

**Item 32 — M12: jackin's request bus `Jx` and the `Msg` channel.** Amends §3.4, §17.0 A1, §18.3 #22.

```rust
fn update(&mut self, cx: &mut Cx<'_>, jx: &mut Jx<'_>, w: &mut World) -> Response<()>;
// Jx is jackin-owned: requests, go, status, open, close, help, copy, with_form.
```

Rejected: `Cx<'f, R>` generic over a request payload — it puts a type parameter into every component's `update` signature (§13 "no gratuitous generic parameters"). `Screen::on_msg` is subsumed by: *Domain messages enter at the top of `App::update`, drained from the application's own queue before any screen `update` runs. `Input` deliberately has no `Msg` variant.* Showcase's `PageCtx` and tablepro's `Request` bus take the same two-context shape (§18.3 #22).

**Item 33 — M19, M20, M31, A15, A16, A17, §20.10-15.** Amends §0 (new), §13.2 (new), §16.5, §16.1, §20.10.

§0 gains the goal-§9 and goal-§23 traceability tables. §13.2 mandates the per-component rustdoc template (`## Construction / ## Ownership / ## Configuration / ## Variants / ## States / ## Actions / ## Focus / ## Keyboard / ## Mouse / ## Layout / ## Parts / ## Overrides / ## Identity / ## Testing / ## Invariants`) with `architecture::every_component_doc_has_the_standard_sections` (rustdoc-json heading scan). New checks: `architecture::no_todo_or_unimplemented` (grep for `todo!`, `unimplemented!`, `TODO`, `FIXME` over `crates/**` and `apps/**`, empty allow-list — goal §29), `architecture::showcase_covers_every_public_component` (cross-check `conformance_suite!` against the showcase page registry — goal §29), `dialog::convenience_constructors_render_through_the_body_slot` (digest equality between `Dialog::confirm(…)` and the hand-composed equivalent — goal §14). §20.10 item 15: focus-ring composition changes in migrated screens (`Field` chrome, `NavList`, disabled-but-registered entries) are an intentional change, classified per test in `docs/visual-changes.md` **before** any expected ring size is edited.

**Item 34 — `xtask doc-check`.** Amends Appendix A (Slice 3 deliverables and every slice gate), §16.5.

`cargo run -p xtask -- doc-check` extracts every `` `Ident::method` `` and every fenced `rust` block from `COMPONENT_ARCHITECTURE.md` §3–§17 and §21 and asserts each resolves against the compiled library's rustdoc-json, or is on an explicit "not yet built (Slice 3/4)" allow-list that the check prints. This converts the §17.0-versus-§3–§15 drift the review found into a permanent CI gate.

### Not applied — requires a fresh `opus-analyst` decision

* Review §3: *"Give §15 a `Form` API sketch before Slice 3, because 4F depends on it."* The review names the requirement (a `Form` that owns the field list and drives both phases; `Form::field(id, …)`) but no API; writing one is a design act, not a correction. Research request: sketch `Form<'a>` / `FormState` / `FormAction` against §15, §12.1 and item 30's "props are built once" convention.
* Item 1's `Grid::update` bound question (`M: GridEditor` with defaulted refusals, or two entry points).
