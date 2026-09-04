# COMPONENT_ARCHITECTURE.md

**Status:** Accepted, with the Slice 2 review corrections of §21 (Adjudication J), the modern-API and dependency policy of §22 (Adjudication L), the `Form` API / `Grid::update` decisions of §23 (Adjudication K) the re-export / ASCII-border / `FieldKind` decisions of §24 (Adjudication M), the Slice 3 foundations review's eight adjudications, thirteen deviation verdicts and correction obligations F1–F26 of §25, the layer-sizing / `Measure` style-access decisions of §26 (Adjudication N), and the Slice 3 foundations follow-ups of §27 (Adjudication O) applied. This document is the single source of truth for the refactor. Builders implement it as written; a change to any *Decision*, *invariant*, exact type, or precedence rule requires a fresh `opus-analyst` adjudication recorded here and in `REFACTORING_STATE.md` (goal §0).

**Authority:** `REFACTORING_GOAL.md` › `DESIGN.md` › existing rendered output/tests › current source. Where the Slice‑1 audits conflict, the adjudications in §3–§15 below are final; the rejected alternative and the reason are stated with each.

**Inputs adjudicated:** `docs/audit/api-audit.md` (API), `docs/audit/app-audit.md` (APP), `docs/audit/domain-boundary-audit.md` (DOM), `docs/audit/interaction-audit.md` (INT), `docs/audit/architecture-research.md` (RES). `docs/audit/performance-audit.md` (PERF) landed after §1–§15 were written; §20.9 folds its obligations in and amends earlier decisions where needed. `docs/audit/modern-api-audit.md` (MOD), `docs/reviews/adjudication-k-form-grid.md` (ADJ‑K) and `docs/reviews/adjudication-m-small-items.md` (ADJ‑M) landed after §21; §22, §23 and §24 record them as binding, and every earlier section they change carries an inline `<!-- amended by §22 -->` / `<!-- amended by §23 -->` / `<!-- amended by §24 -->` marker. `docs/reviews/slice3-foundations-review.md` (the fresh read-only Slice 3 foundations review, at commit `18afddd`) and `docs/reviews/adjudication-n-layer-measure.md` (ADJ‑N) landed after §24; §25 and §26 record them as binding, and every earlier section they change carries an inline `<!-- amended by §25 -->` / `<!-- amended by §26 -->` marker. `docs/reviews/adjudication-o-foundations-followups.md` (ADJ‑O) landed after §26; §27 records it as binding, and every earlier section it changes carries an inline `<!-- amended by §27 -->` marker.

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
| — additional | §15 Forms and text editing (goal §19) with §15.1 `Form`; Appendix A Slice plan; §21 Slice 2 review corrections; §22 Adjudication L (modern API and dependency policy); §23 Adjudication K (`Form` API, `Grid::update` bound); §24 Adjudication M (re-exports, ASCII border set, `FieldKind`); §25 Slice 3 foundations review (eight adjudications, deviations D‑1…D‑13, corrections F1–F26); §26 Adjudication N (layer sizing, `Measure` style access); §27 Adjudication O (Slice 3 foundations follow-ups: memo associativity, ASCII glyph coupling, the `border_subtle` downgrade correction, two perf substitutes) |

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

<!-- amended by §21 items 3, 6, 11, 12, 13, 14, 15, 16, 17; §25 (step 1 resize/F16, step 3 hit ordering/F8 and wheel layer filtering/MI‑5, step 4 capture origin/F15, step 7 give-up path/MI‑7, step 14 branch (d)/MI‑2); §28 (step 9 undelivered-intent guard, P3) --> <!-- amended by §28 -->

`Input` never touches a component directly. `Runtime<A>` owns all interaction state; the app owns only domain state and `XState`s.

```
Runtime::handle(&mut self, input: Input) -> Response<()>
 ── INPUT PHASE (no buffer in scope) ─────────────────────────────────────────
  1. normalize        Input from crossterm; drop key releases and unmapped buttons.
                      Resize -> record size, invalidate = Layout, no input intents; the pass
                                still runs steps 6-7 so staged focus intents are delivered
                                (§25 F16, MA-7).
                      Tick   -> advance flash/motion clocks, then step 7 (app.tick).
                      Paste  -> text copied into a runtime-owned frame arena that outlives
                                step 7, so Intent::Paste(&'f str) borrows it (§21 item 6).
  2. capture keymap   app KeyMap "Capture" bindings are matched FIRST, but a chord
                      that is a bare Char is skipped while `focus_owner_swallows_typing`
                      (§11.4). A capture hit produces an app action and skips 3-8.
  3. resolve          against LAST FRAME's Registry and FocusRing (never a fresh scan
                      of the app tree):
                        pointer -> Registry::hit(pos) -> Hit{owner,part,layer,kind,local}: the
                                   topmost region REGARDLESS of layer — highest layer first,
                                   then latest registration, never registration order alone
                                   (§25 F8, MA-1); delivered iff
                                   hit.layer == top_layer, otherwise it is the top layer's
                                   outside-click (§21 item 12)
                                   (a live Capture short-circuits this: the capture owner
                                    receives Drag/Release with `local` against the captured area)
                        wheel   -> Registry::hit_scroll(pos, axis) -> innermost scrollable
                                   handling that axis, returned even at zero headroom;
                                   delivered only when hit.layer == top_layer, so a wheel
                                   over the page under a popover scrolls nothing (§25 MI-5)
                        key     -> FocusState::current()  (None -> app bubble phase only)
                        paste   -> the focused owner iff it declared EDITING
  4. interaction      hover / hover_suppressed / press bookkeeping / 140 ms flash /
                      double-click window / capture claim+release. All of §1.2(4).
                      A Move under a live capture is delivered as Phase::Drag; the
                      Capture's origin is the live press position (§25 F15, MA-5).
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
                      Diagnostic::FocusTransitionDidNotSettle, applies the pending focus
                      change to the last requested target without re-running app.update,
                      and delivers the matching FocusOut AND FocusIn pair on the NEXT
                      handle through pending_focus (§25 F16, MI-7: enqueuing then clearing
                      in the same pass discarded them); continues (suite test
                      focus_transition_settles asserts the count is 0) (§21 item 11).
                      Captures whose owner's layer was closed by this pass are released
                      here as well as at step 13 (§21 item 17, F8).
  8. bubble           keys still unconsumed after step 7 are offered to (a) the app KeyMap
                      "Bubble" bindings, then (b) Dismiss.esc on the top layer, then (c) the
                      screen's Esc ladder (§21 item 3). Esc therefore reaches an editing
                      control inside a layer before the layer.
  9. finish           Intents that no component drained are dropped; every undrained bucket
                      is diagnosed FIRST, before the queue is cleared. `UndeliveredIntent`
                      is recorded when the owner registered a Control or Part region and
                      drained nothing (§21 item 13), OR when the bucket holds an intent the
                      RUNTIME addressed - Layer, Cancel, FocusIn, FocusOut - whatever that
                      owner registered (§28 P3):
                        `if registry.delivers_to(owner) || intents.has_runtime_addressed(owner)`.
                      ~~An intent whose resolved owner registered only Decorative regions is
                      discarded silently~~ survives for POINTER intents only: `deliverable`
                      already keeps a pointer intent away from a Decorative owner, so §21
                      item 13's escape for container regions never applied to a runtime-
                      addressed one, and a layer owner that registers only decor would
                      otherwise lose its own dismissal in silence.
                      Returns Response<()> whose `flow` and `invalidate` are the fold
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
                        (d) else the first reachable entry in any scope (§25 MI-2),
                        (e) else None.
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

**R2** `draw` may: write cells through `Ui` painting methods; register hit/scroll/focus regions; report layout facts; report a cursor request; declare next-frame state flags (`Ui::declare_state`, §17.0 A2 — a non-semantic draw-phase write the runtime reads on the **next** frame, the same one-frame contract as `report_layout` and `cx.area`, §25 D‑6) <!-- amended by §25 -->; push style scopes, surfaces, focus scopes and layers. It may not mutate the app, the state, or the theme — enforced by `&self`/`&XState`/`&Theme`.

**R3** <!-- amended by §22 --> All painting goes through `Ui` (`ui.paint_cell`, `ui.paint_str`, `ui.paint_style`, `ui.fill`, `ui.rule`, `ui.frame`, `ui.glyph`), so a layer's written-cell bitset is always correct. `ui.raw() -> (&mut Buffer, Rect)` is the documented escape hatch; it marks the whole rect written and is the only way to reach the buffer. <!-- amended by §25 --> `Ui::paint_spans` paints span-by-span through `Buffer::set_span` — the sanctioned per-span writer beside `set_line` (§22 R‑3, F4) — with no intermediate `Vec`; the internal callers `CellUi::drop` and `RowUi::raw` never use `raw()` but the crate-private `Ui::buffer_in(area)`, which marks only `area`, so the written-cell bitset and the per-cell roles `dim_layer` reads stay exact (F3, `layer::composite_copies_only_painted_cells`, `ui::dim_layer_uses_the_role_of_the_painted_cell`).

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

// #[non_exhaustive] on Intent, Phase, FocusVia: runtime-produced, matched by the caller (§22, MOD §3.2)
#[non_exhaustive]
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
#[non_exhaustive] pub enum Phase { Press, Release, Click, DoubleClick, Secondary, DragStart, Drag, DragEnd }
#[non_exhaustive] pub enum FocusVia { Keyboard, Pointer, Programmatic, Restore }

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
// error, never silent loss. flow: Consumed wins; invalidate: max; id: lhs.id.or(rhs.id) — the first
// Some, so an `ignored()` on the left never erases the right's id (§25 §4(c)); state: lhs — the fold
// is a control-flow summary; read `state`/`id` from the individual responses. (§21 item 4) <!-- amended by §25 -->
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
pub enum SelectAction     { Chose(ItemKey), Opened, Closed }                     // <!-- amended by §24 M3: declared by the §17 self-check -->
pub enum RadioGroupAction { Chose(ItemKey) }                                     // cursor motion is `Moved`-less: cursor ≠ value (§15)
pub enum ChipBarAction    { Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey) }
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
// <!-- amended by §25 (adjudication 2, D‑2) --> structural derives, not hash-only manual impls
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id { hash: u64, #[cfg(debug_assertions)] label: DebugLabel }   // label: 0 bytes in release

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

**Debuggability.** `Debug for Id` prints `orders.list ▸ Row(#0x1f3a)` in debug builds and `Id(3f9a…)` in release. The debug label travels with the `Id` itself (`Tail::Item(k)` carries the item key inline), so no side table exists in any build and a diagnostic prints the path. <!-- amended by §21 item 22: `Registry::names` struck --> <!-- amended by §25 --> Equality, hashing and ordering are **structural** (derived), because a `const Id` used as a `match` pattern requires structural equality (§15.1's `FormData`, example 13). The debug label is a pure function of the segments the hash was computed over — `root` from the root segment, `tail` from the last — so two ids with equal hashes carry equal labels and debug and release compare identically; only a genuine FNV collision could differ, and there the label is the more honest answer (test `id_equality_is_exactly_hash_equality`; `id_equality_ignores_debug_label` survives as a `#[cfg(debug_assertions)]`-only assertion that a label never changes an answer).

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
pub struct Capture { pub owner: Id, pub part: PartRef, pub origin: Position,   // origin = the LIVE press position, never area's top-left (§25 F15) <!-- amended by §25 -->
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

`ui.set_cursor(owner: Id, pos: Position)` records `(layer, owner, pos)`. The runtime keeps the write iff `layer == top_layer && FocusState::current() == owner`; otherwise it drops it and records `Diagnostic::CursorRejected`. A `set_cursor` from a suppressed (inert) layer is discarded silently; `CursorRejected` is recorded only for a non-inert lower layer or an unfocused owner. <!-- amended by §21 item 15 --> A background `TextInput` still flagged `EDITING` can never place the cursor under a dialog (today only draw order prevents it). <!-- amended by §25 F6 --> When two owners on the same layer write in one frame (two `EDITING` inputs in a `Form`), `Ui::set_cursor` keeps the best candidate by `(layer, owner-is-focused)` — a request whose owner carries `FOCUSED`, then the higher layer, then the later write — never the first arrival; `CursorRejected` is recorded for the loser only when it is non-inert (`cursor::the_focused_owners_write_wins_on_the_same_layer`). Components write unconditionally; filtering is the runtime's job.

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
/// <!-- amended by §26 (Adjudication N1) --> How large a layer asks to be. The resolver clamps to the
/// screen; it never grows a layer, so a `Fixed` size is a maximum as well as a request ("size, then
/// clamp, then documented degradation"). `Fixed(0, _)`/`Fixed(_, 0)` resolves to `Rect::ZERO`, never to
/// the screen — the old `(0,0) ⇒ whole screen` sentinel was the defect.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerSize {
    /// The whole screen. The content lays itself out; `Anchor` is ignored. Help overlays, `TooSmall`.
    Fill,
    /// Exactly `w × h` cells before clamping.
    Fixed(u16, u16),
}
#[non_exhaustive]                                      // construct through the §17.0 A6 builders (§21 item 8)
pub struct LayerSpec {
    pub kind: LayerKind,
    pub owner: Id,                 // anchor owner + focus-restore target
    pub anchor: Anchor,
    pub dismiss: Dismiss,          // focus_out is honoured only for Popover/Tooltip; a Modal traps focus (A10)
    pub restore_focus: bool,
    pub initial_focus: Option<Id>,
    pub size: LayerSize,           // §21 item 20 as amended by §26: was `min_size: (u16, u16)` — it was never a minimum <!-- amended by §26 -->
    pub backdrop: Backdrop,        // None | Dim { exclude_footer: bool }
    pub inert_below: bool,         // Modal: true
}
#[non_exhaustive] pub enum LayerEvent { Opened, Dismissed(DismissReason), Closed(ActionKey) }   // §22 (MOD §3.2)
#[non_exhaustive] pub enum DismissReason { Esc, OutsideClick, FocusOut, Programmatic }

impl Cx<'_> {                                   // opened / closed from `update`
    pub fn open_layer(&mut self, id: Id, spec: LayerSpec);
    pub fn close_layer(&mut self, id: Id, with: Option<ActionKey>);
    pub fn layer_event(&mut self, id: Id) -> Option<LayerEvent>;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
    // <!-- amended by §26 --> geometry is the ONE part of an open spec that may change while open
    pub fn resize_layer(&mut self, id: Id, size: LayerSize);    // no-op when not open or unchanged; the next draw re-resolves; call every frame
    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor);   // a popover whose owner moved
}
impl Ui<'_> {                                   // content is drawn from `draw`
    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>;
}
```

The stack is runtime state; the **content** is drawn by the app inside `ui.layer(...)`, so layer content borrows app data freely and nothing is boxed or `'static`. Placement, flip, clamp and clip are one resolver (`Anchor` + `Side` + `CrossAlign`), replacing the two independent `Placement` enums and algorithms (`ui/popup.rs:25-56`, `menu.rs:143-171`) and the hand-written `Rect::centered` in `dialog.rs:376`. <!-- amended by §22 --> The resolver reuses `Rect` geometry rather than re-deriving it: `Anchor::Screen(ScreenAlign)` resolves through `Rect::centered_horizontally` / `Rect::centered_vertically`, and the "flip **then clamp**" clamp step is `Rect::clamp`, never fresh min/max arithmetic (§22 R‑12). The backdrop dim is one implementation (replacing three byte-identical loops) and it excludes the footer row (`DESIGN.md:537`). <!-- amended by §26 --> The resolver is given a size; it clamps and never grows. **The size is the opener's, the placement is the runtime's; a component computes a size, never a rect.** A layer whose content changes size re-asserts it from `update` with `Cx::resize_layer`. The `Rect` handed to `Ui::layer`'s closure is the resolved layer area and is already the clip; a layer's content lays out inside it and never re-anchors, re-flips or re-clamps. `Anchor::Point` flips (above/left of the pointer when the content does not fit below/right) instead of sliding over it — a visual change classified as §20.10 item 17.

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
| small terminal | 3 ad-hoc screens | `LayerSize::Fixed`, then clamp, then documented degradation (`layer::fixed_size_is_clamped_never_grown`) <!-- amended by §26 --> |

`Dialog`, `Picker`, `ContextMenu`, `MenuBar` dropdowns, `Select`'s popup, `Completion`, and jackin's `FileBrowser`/`ChoiceDialog`/`FormDialog`/`InfoDialog`/`HelpOverlay` all become **content rendered into a layer**. `begin_modal` and the shared `"popup.surface"` id are deleted, and with them the six blocks of manual hit re-registration (**[F]** APP §2.2).

**Rejected:** sorted-`z` widgets (solves paint only); modality as a render side effect; per-app stacks.

**Layer identity and draw order** <!-- amended by §21 items 14, 17 -->. `LayerId` is assigned monotonically by `Cx::open_layer` and is the stack position. `Ui::layer(id, f)` resolves `id` to its already-assigned `LayerId`, executes `f` into that layer's pooled buffer, and returns `None` without executing `f` if `id` is not open. Call order at draw time has no effect on z-order, hit filtering, or focus scope nesting (`layer::layer_id_is_assigned_at_open_not_at_draw`). A second `ui.layer` call with the same `id` in one frame returns `None` and records `Diagnostic::DuplicateLayerDraw` (F10). With `inert_below`, no scroll region is registered beneath the layer, so a wheel over the backdrop falls through to the app bubble phase; there is no outward chaining (F9). <!-- amended by §26 --> The spec is fixed at open except its geometry (`size`, `anchor`), which `Cx::resize_layer` / `Cx::reanchor_layer` may change; `kind`, `inert_below`, `restore_focus` and `initial_focus` are armed at open (§3.3 step 11) and are immutable — re-deriving them mid-life would desync the focus scope and the inert floor (`layer::spec_geometry_is_the_only_mutable_field`).

### 9.2 Dialog content is open

`DialogBody` is deleted. `Dialog::show`-equivalent takes a body slot; `confirm`, `destructive`, `prompt`, `acknowledge`, `facts`, `choice`, `info` are convenience constructors over the same primitive and the same rendering path (goal §14). Action arming is a predicate evaluated in `update`, never a `disabled` flag flipped during draw. <!-- amended by §26 --> `Dialog` sizes its own layer — `Dialog::layer(cx)`, `body_rows`, `measured_width`/`measured_height` (§17.0 A7) — as a pure function of props and `DesignTokens` (`text::wrapped_rows` is the single wrap both `measured_height` and `draw` use), and re-asserts it at the top of every `update` with `cx.resize_layer(self.id, LayerSize::Fixed(w, h))` (**invariant D1**), so a description that grows, an error row that appears or a theme swap corrects the layer on the next draw without the opener predicting anything. `Select`, `Picker`, `ContextMenu` and `MenuBar` do the same with their own arithmetic (`Select`: `popup_min_width ≤ w ≤ popup_max_width` from the labels it receives per phase, `h = min(items, popup_max_rows) + 2`). No overlay component opens a bare `LayerSpec` and none computes a rect: `! rg -n 'centered|centered_horizontally|centered_vertically|resolve_anchor' crates/tui/src/components/`.

---

## 10. Layout, measurement and surface inheritance

```rust
pub struct Size { pub min: (u16, u16), pub preferred: (u16, u16) }
pub struct Constraints { pub max: (u16, u16), pub tight_w: bool, pub tight_h: bool }
pub struct Insets { pub l: u16, pub t: u16, pub r: u16, pub b: u16 }

pub trait Measure { fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size; }  // small, optional
// <!-- amended by §25 D‑13; §26 (Adjudication N2) --> `measure` is `&Ui` and stays so: it resolves styles through
// `Ui::resolve` (uncached, records no roles and no styled parts) and glyphs through `Ui::glyph_str`, never through
// `Ui::style` (`&mut self`, the painting query). Update-phase sizing (`Form` F4, `Dialog::layer`) uses `Theme::metrics`.
impl Constraints { pub const fn loose(w: u16, h: u16) -> Self; }        // max = (w, h), tight_w = tight_h = false — declared by the §25 self-check
impl Size { pub const fn exact(w: u16, h: u16) -> Self; pub fn fit(self, c: Constraints) -> Size; }   // min = preferred = (w, h); clamp to c.max

pub mod layout {
    // <!-- amended by §25 (adjudication 7, Track::Auto) --> the `_measured` variants are the home for Measure-derived sizes
    pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect>;      // Track::{Fixed(u16), Flex(u16), Auto}
    pub fn rows_measured(area: Rect, heights: &[Track], natural: &[u16]) -> Vec<Rect>;
    pub fn columns(area: Rect, widths: &[Track], spacing: u16) -> Vec<Rect>;
    pub fn columns_measured(area: Rect, widths: &[Track], spacing: u16, natural: &[u16]) -> Vec<Rect>;
    pub fn distribute_into(total: u16, tracks: &[Track], spacing: u16, out: &mut [u16]);  // 0-alloc, RowUi::columns
    pub fn responsive_columns(area: Rect, spec: &[Track], gap: u16, stack_below: u16) -> Vec<Rect>;
    pub fn action_row(area: Rect, widths: &[u16], spacing: u16, align: RowAlign) -> Vec<Rect>;   // `spacing`, not `gap` (§22)
    pub fn inset(area: Rect, i: Insets) -> Rect;
    pub fn split_v(area: Rect, at: u16) -> (Rect, Rect);
    pub fn split_h(area: Rect, at: u16) -> (Rect, Rect);
}
pub struct SplitModel { pub percent: u8, pub min_first: u16, pub min_second: u16,
                        pub maximized: Maximized, pub axis: SplitAxis }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Track { Fixed(u16), Flex(u16), Auto }   // declared here (was only a comment above) <!-- amended by §22 -->
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum RowAlign { Start, End }                  // reads against Flex::{Start, End} (§22)
```

<!-- amended by §25 (adjudication 7) --> `Track::Auto` is content-sized. Without a measurement the primitive gives it **one cell** when explicit `Flex` tracks exist (so `Auto` never starves a `Flex`) and an **equal share of the remainder** when there are none. Supply the natural size through `rows_measured` / `columns_measured` to get the content size; a component that has a `Measure` impl should always do so (`layout::auto_takes_one_cell_beside_flex_and_an_equal_share_without_it`, `layout::rows_measured_uses_the_natural_size`; pinned by `layout::rows_distributes_flex_after_fixed`). Every `Track::Auto` in §17 was re-checked against this rule (example 9 now uses `rows_measured`); `xtask doc-check` cannot catch this class, so it sits on the Slice-4 wave-1 review checklist.

<!-- amended by §24 --> `Size { min, preferred }` is the library's own type and keeps its name at the root and in `author`; `ratatui_core::layout::Size` (`{ width, height }`, a different shape under the same name) is named by no `pub` signature and is **never re-exported** (§24 M1).

**Decisions.** One `Split` implementation parameterised by axis, so the vertical/horizontal collapse asymmetry disappears; when both minima cannot fit, **the first pane wins on both axes** (documented, tested). `button::row_layout`/`row_layout_right` move into `layout::action_row` (they are the generic action-row primitive already used by dialog and grid). `showcase/pages/mod.rs:120-168`'s `rows`/`columns`/`caption` become library primitives. No general constraint solver.

<!-- amended by §22 --> **Borrow ratatui's vocabulary, not its engine.** `RowAlign::{Start, End}` (not `Left`/`Right`) and the parameter name `spacing` read against ratatui's `Flex`/`Spacing`; `layout::action_row(area, widths, spacing, RowAlign::End)` is the exact analogue of `Layout::horizontal(…).flex(Flex::End).spacing(Spacing::Space(spacing))` and its rustdoc says so. `Track::{Fixed, Flex, Auto}` stays a hand-written 3-case integer distribution — deterministic, 0-alloc, and able to express `Auto`, which `Constraint` cannot — so `Layout`/`Constraint`/`Flex`/`Spacing` never appear under `components/**` (§22 R‑13) and `ratatui-core/layout-cache` stays off. `Rect` geometry is reused, never re-derived (§22 R‑12): `layout::inset` is `Rect::inner(Margin)` for the symmetric case and keeps `Insets` for the asymmetric one; `Rect::ZERO` replaces `Rect::new(0, 0, 0, 0)`; centering and clamping are `Rect::{centered, centered_horizontally, centered_vertically, clamp}`.

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
* **A3** `Recipe` resolution is memoised per `(Family, Variant, Part, StateFlags, overlay-stack-hash)` in a small per-frame array cache (≤ 256 entries, cleared each frame), because rows resolve the same tuple repeatedly. <!-- amended by §25 §4(f); §26 --> `Surface` is deliberately **not** in the key: the memo caches steps 1–5 of §11.3, which are role-level and surface-independent; roles bind to colours afterwards in `bind`, per query. The memo serves the **painting path only** (`Ui::style` / `Ui::style_patched`); `Ui::resolve` — the `&self` path `Measure::measure` uses — bypasses it, so measurement can never evict a painting entry, and `StyleCache::stats` (promoted to `#[cfg(feature = "testing")]`, `Runtime::style_cache_stats()`) is the cache-health assertion §16.6 binds (hit rate ≥ 90 %).
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
    SecretMask,                               // password mask: Junie `•` — the same glyph as Dirty, a DISTINCT role, so a theme that changes the dirty marker never changes masking (§25 D‑11) <!-- amended by §25 -->
}
pub struct GlyphSet { /* one &'static str per GlyphRole; ScrollTrack/ScrollThumb are read from a typed
                         ratatui_core::symbols::scrollbar::Set<'static> holding the Junie values `│`/`┃` (no built-in
                         set matches them), RuleQuiet/RuleActive from a ratatui_core::symbols::line::Set<'static> (§22) */ }
/// <!-- amended by §22 --> A type alias, not a bespoke six-field struct. Eight fields (`top_left`, `top_right`,
/// `bottom_left`, `bottom_right`, `vertical_left`, `vertical_right`, `horizontal_top`, `horizontal_bottom`), so a
/// theme hands over `symbols::border::{PLAIN, ROUNDED, DOUBLE}` directly instead of retyping glyphs (MOD §2.12, R‑11).
/// Junie's rounded set is `symbols::border::ROUNDED` verbatim (verified against current output before blessing).
/// <!-- amended by §24 --> The named sets live in `crates/tui/src/theme/border.rs`, reachable as `junie_tui::theme::border`
/// and `junie_tui::author::border`: `border::{Set, PLAIN, ROUNDED, DOUBLE}` are ratatui's, re-exported; `border::ASCII`
/// (`+ - |`) is OURS, a plain `const ASCII: Set<'static>` — a const of a foreign type is not an `impl` and needs no
/// coherence exception, which is what makes the §11.2 triple `rounded | square | ascii` expressible on a type alias.
/// `Theme::junie()` → `border::ROUNDED`; `Theme::paper()` → `border::PLAIN`; `border::ASCII` is used by NO builtin and
/// is opt-in through `ThemeBuilder::borders_set(border::ASCII)`. Nothing selects ASCII automatically: `Capability` has
/// no unicode axis (§21 item 19) and a border-only auto-switch would leave every other `GlyphSet` glyph unicode (§24 M2).
/// <!-- amended by §27 (Adjudication O2) --> `ThemeBuilder::borders_set(border::ASCII)` **also applies**
/// `ThemeBuilder::ascii_glyphs()`, which rebinds the typed `line` and `scrollbar` sets. This is not a widening of
/// §24 M2's scope: `RuleQuiet`, `RuleActive`, `ScrollTrack` and `ScrollThumb` are **exactly** the `GlyphRole`s whose
/// Junie binding falls in `U+2500..=U+257F`, the block `theme::ascii_theme_renders_without_box_drawing_glyphs` scans —
/// verified role by role against every entry of Junie's 39-glyph table and its typed sets (§27, O2). A
/// `borders_set(ASCII)` that left `─` in every divider would produce precisely the outcome §24 M2 rejected automatic
/// selection for ("ASCII at the edges and unicode everywhere else, worse than either consistent choice"). The swap
/// replaces the **whole typed sets**, not four fields, so `scrollbar::Set.begin`/`.end` and `line::Set`'s seam
/// junctions — which no `GlyphRole` names — are covered too, and no new `GlyphRole` is added (§11.2's role list is
/// unchanged). `ascii_glyphs()` is public and idempotent; a later `.glyph(..)` overrides it, an earlier one is
/// silently replaced. The remaining ~31 roles stay unicode: that is §24 M2 risk 3, scheduled for **Slice 4E**.
pub type BorderSet = ratatui_core::symbols::border::Set<'static>;

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
    pub capability: Capability,   // { color: ColorLevel } — UnicodeLevel deleted (§21 item 19); exactly one field, pinned by `architecture::capability_has_no_unicode_field` (§24 M2)
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

`Family` and `Variant` mirror `Part`: `u16` newtypes, library constants in the low range, `custom(&'static str)` const fn into the high range. Families: `BUTTON, CHOICE, CHIP, FIELD, INPUT, TEXTAREA, CODE, SELECT, LIST, TREE, GRID, PROPS, STEPS, TABS, PANEL, SPLIT, SCROLLBAR, VIEWPORT, DIFF, DIALOG, OVERLAY, MENU, PICKER, COMPLETION, FORM, HELP, WIZARD, STATUSBAR, HINTBAR, PROGRESS, METER, EMPTY, BRAND, KEYHINT`. Variants: `DEFAULT, PRIMARY, SECONDARY, SUBTLE, DANGER, TOGGLE, QUIET, GHOST` + custom. <!-- amended by §25 F14 (MA‑6) --> A custom family (`Family::custom(..)`) starts from the **neutral recipe** — `row_like`'s `CONTAINER/GUTTER/MARKER/LABEL/META` set, used whenever `Recipes::get(f)` misses — and `define_family` replaces it. Without this a downstream family resolved to an empty patch and rendered invisible, the worst first-run experience in the surface (`theme::a_custom_family_resolves_through_the_neutral_recipe`; example 12's expectations follow it).

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
pub struct PartRecipe { pub base: StylePatch, pub states: Vec<StateRule>,      // Vec, never SmallVec (§22, MOD §4.2)
                        pub glyph: Slot<GlyphRole>, pub size: Slot<u16> }
pub struct Recipe  { pub default_variant: Variant, pub parts: PartMap<PartRecipe>,
                     pub variants: Vec<(Variant, PartMap<PartRecipe>)> }        // built once at theme construction; resolution only reads
pub struct Recipes { by_family: Box<[Recipe]> }
pub struct Overlay { /* borrowed scope override; const-constructible */ }
pub struct Resolved { pub style: Style, pub glyph: Option<GlyphRole>, pub size: Option<u16>, pub align: Option<Align> }   // <!-- amended by §25 D‑5; §26 --> `align` was the omission (`StylePatch.align` already existed)
impl Resolved {
    /// This part's style layered over an inherited one — §11.3's final step, `Style::patch` semantics
    /// (modifier symmetry, §22 R‑9). Call site: `ui.fill(area, r.over(ui.surface_style()))`. <!-- amended by §26 -->
    #[must_use] pub fn over(self, inherited: Style) -> Style { inherited.patch(self.style) }
}
/// <!-- amended by §26 (Adjudication N2) --> The surface-independent half of resolution: everything §11.3
/// settles before roles bind to colours. Available in `update`, which has no `Surface`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PartMetrics { pub glyph: Option<GlyphRole>, pub size: Option<u16>, pub align: Option<Align> }
impl Theme {
    /// The whole chain plus colour binding, `&self`, uncached. The `update`- and test-phase resolution path.
    pub fn resolve(&self, f: Family, v: Variant, p: Part, s: StateFlags, surface: Surface) -> Resolved;
    /// <!-- amended by §26 --> Sizes, glyphs and alignment for a part, with **no** colour binding and **no**
    /// overlay stack (an `update` has neither a surface nor a draw-time scope). The sizing path for `Cx`-phase
    /// arithmetic — `Form`'s field height (§15.1 F4) and `Dialog::layer` (§26 N1). `Theme::resolve` is
    /// refactored to `metrics` + colour binding, so there is exactly ONE `accumulate` path and the two cannot
    /// drift (`resolve::bind` already separates the concerns; `theme::metrics_are_surface_independent`).
    pub fn metrics(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> PartMetrics;
}
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

<!-- amended by §22 --> The final application of `Resolved.style` over the inherited surface style is `inherited.patch(resolved.style)`: `Style::patch` is the one layering operation with correct `add_modifier`/`sub_modifier` semantics, and the modifier-symmetry law above *is* its semantics (`theme::patch_merge_matches_ratatui_style_patch_for_modifiers`, MOD §2.10, R‑9). `StylePatch::merge` is a role-level merge and is not replaceable by `Style::patch`. <!-- amended by §26 --> The fused form is `Resolved::over`, with `Ui::surface_style() -> Style` as the left operand (`bg = theme.bg(ui.surface())`, `fg = Role::Fg(FgStep::Primary)` bound on that surface, no modifiers): `ui.fill(area, r.over(ui.surface_style()))`. §22.2 item 10 is on the Slice-4 per-package checklist because no Slice-3 production call site performs the layering yet (`ui::surface_style_is_the_left_operand_of_the_final_patch`; `theme::patch_merge_matches_ratatui_style_patch_for_modifiers` routes through `Resolved::over`). Styles are spelled `Style::new()`, never `Style::default()`, and `Stylize` shorthands are banned in library and application code (§22 R‑8).

**Invariant:** overlays are borrowed and never mutate the `Theme`. `conformance::local_override_does_not_mutate_the_theme` asserts the theme is byte-identical before and after a scoped render.

### 11.4 Capability downgrade and the mono rule

```rust
impl ColorTokens {
    /// Exhaustive destructure: adding a field is a compile error here.
    pub fn map_colors(&self, f: &mut impl FnMut(Color) -> Color) -> ColorTokens;
}
pub fn downgrade_color(c: Color, level: ColorLevel) -> Color;   // §21 item 29, exact:
// nearest_256: nearest in the 6×6×6 cube ∪ 24-step greyscale by squared sRGB distance, ties to the lower index.
// nearest_16:  <!-- amended by §25 (adjudication 3, D‑3 REJECTED) --> NOT a ΔE minimisation. A colour whose channel spread
//              (max−min) is under 40 collapses to the grey ladder by ITU-R BT.601 luma (≤30 Black, ≤110 DarkGray,
//              ≤200 Gray, else White); otherwise the dominant channel selects the hue family (r ≥ g,b ∧ g > 120 ∧ b < 80
//              reads as Yellow) and max(r,g,b) > 180 selects the light half. Exact code in §25.3. Recorded rejection:
//              nearest-by-CIE76 ΔE — the more "correct" perceptual answer and the wrong design answer: it maps Junie's
//              accent #48e054 and error #e44545 into the DARK half and collapses danger_soft onto a grey. DESIGN.md:320
//              fixes the outcome (accent LightGreen, error LightRed) and the authority order puts it above this document.
// mono:        Y = 0.2126R + 0.7152G + 0.0722B; Y < 0.35 → black, Y > 0.75 → white, else Color::Reset.
// Test: theme::downgrade_is_deterministic_per_level.
impl Theme {
    pub fn downgrade(&self, level: ColorLevel) -> Theme {
        let mut out = self.clone();
        out.capability.color = level;
        out.color = self.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == ColorLevel::Mono { out.recipes.apply_mono_fallbacks(); }   // `&mut self` on Recipes; `(&mut out)` was a borrow error (§25 D‑4) <!-- amended by §25 -->
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
| `DISABLED` | no gutter glyph, no marker, **`DIM` added and every other modifier removed, on `LABEL`, `MARKER`, `FIELD` and `TEXT`**. At `Mono` the foreground is set to `Fg(Primary)`, **not** ~~`fg = Role::Fg(Faint)`~~: `mono()` maps every step below `Y = 0.35` to `Black`, so a faint disabled foreground on a dark canvas is black-on-black — invisible rather than merely colourless (goal §29; under `Theme::junie()` `disabled_fg #4d4d4d` and `Fg(Faint) #262626` both fall in that band and `surfaces[0]` is `#000000`). Colour is excluded from case 9's comparison, so `DIM` plus the absent glyphs is the whole signal, and it is enough (§25 D‑4). A rule reaching only `LABEL`/`GUTTER`/`MARKER` never reached a text control's content, which is why `TextInputCase` could narrow `DISABLED` away (§28 P6). <!-- amended by §25 --> <!-- amended by §28 --> |
| `ERROR` | trailing `GlyphRole::Error` in `Part::MARKER` + `UNDERLINED` on `Part::FIELD` |
| `WARNING` / `DIRTY` | `GlyphRole::Dirty` in `Part::MARKER` |
| `EDITING` | `UNDERLINED` on `Part::TEXT` + the hardware cursor |
| `BUSY` / `LOADING` | **a component obligation, not a `StateRule`**: a component that can enter `BUSY`/`LOADING` paints `Part::ICON` with `design.motion.spinner_frames`. The spinner is a *symbol*, so it satisfies case 9 without any theme rule, and a `StateRule` could not express it — a rule binds one `GlyphRole`, the spinner is a frame sequence. A component with no icon slot must not accept `.status(…)`. ~~spinner glyph in `Part::ICON`~~ read as a mono `StateRule` obligation and produced none: no mono rule mentions `BUSY` or `LOADING`. <!-- amended by §28 --> |
| `ACTIVE` (tabs) | `Part::RULE` glyph = `RuleActive` + `BOLD` label |

Test `conformance::mono_states_are_distinguishable` compares `(symbol, modifier)` pairs only, colour excluded, for every component × every state. <!-- amended by §25 MI‑13 --> `apply_mono_fallbacks` appends its rules to the family's part maps **and** to every variant map, because under the §11.3 precedence a variant that re-declares `PRESSED` would otherwise beat the mono bracket rule; the interaction is recorded here so it is not rediscovered.

### 11.5 Where each concern lives (binding)

Colour roles → `ColorTokens`. Spacing → `design.space`. Dimensions → `design.size`, overridable per recipe via `PartRecipe.size`. Glyphs → `design.glyphs`. Border sets → `design.borders`. Focus indicator: the *glyph* is `GlyphSet::FocusBar`, *which parts wear it* is the recipe's `Part::GUTTER`. Selection indicator: glyph in `GlyphSet`, placement in `Part::MARKER`. Scrollbar symbols → `GlyphSet`, tone → the `SCROLLBAR` recipe's `TRACK`/`THUMB`. Animation cadence → `design.motion`. Density → `design.density` + per-instance `.density(...)`. Variant defaults → `Recipe.default_variant`. Meter thresholds → `design.meter`. Layer geometry → `LayerSpec.size` (the component) + `resolve_anchor` (the runtime) <!-- amended by §26 -->.

### 11.6 Junie-specific structural assumptions, resolved

**[F]** API §6.2 lists five that are *not* literal colours. Their resolutions: `lift` equality-dispatch → ladder index arithmetic (§10); `backdrop` equality-dispatch → a `backdrop` recipe keyed on `Role`, applied per resolved role rather than per `Color`; accent-budget rules ("only the lockup fills with accent", "one accent underline per screen", "a quota is never green") → recipe *defaults* in `Theme::junie()`, not component code; the reserved menu hue (`highlight`, `highlight_danger`, `danger_soft`) → first-class tokens every theme must supply; glyph/spacing literals → design tokens.

`rain::dim_buffer`'s colour-identity reverse lookup (**[F]** APP §8) is replaced by `Ui::dim_layer(area, steps)`, a runtime service that walks the **role** recorded per painted cell (`FrameOut::roles`, a parallel `Vec<Option<Role>>` filled by `Ui` painting methods) and steps it down the ladder semantically. Jackin's rain keeps its own `Tone` enum but maps it through `Role`, satisfying goal §22.3.

### 11.7 The distinct non-Junie theme

`Theme::paper()` — a **light** theme: `surfaces = [#fbfaf8, #f2f0ec, #e8e5df, #ded9d0, #cfc8bb]`, fg ladder from `#1b1a17` down to `#c6c0b6`, accent `#3b5bdb` (indigo), danger `#b02525`, warning `#a86400`, success `#1f7a3d`, `border_subtle #ded9d0` / `border_strong #9c948a`, square borders (`theme::border::PLAIN`, §22 <!-- amended by §24 -->; `border::ASCII` is opt-in for both builtins via `borders_set`, never a builtin default), `Density::Compact`, and `default_variant` for `BUTTON` set to `SECONDARY`. It inverts the plane direction (hover *darkens*), changes hue family, changes glyph border set and density — the four axes that expose hidden Junie assumptions.

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

**Forced state composes downwards (A11).** A container that can be forced into a reference state forces every component it owns, through the crate-internal `inherit_forced`; a reference rendering registers nothing, at any depth. `Overrides::inherit_forced` and `Button::inherit_forced` are `pub(crate)` and deliberately not spelled `.state_override(`, so `architecture::state_override_is_used_only_in_apps_and_fixtures` still sees every *caller* use; `FieldControl` carries the same hook as an identity-defaulted trait method, and `Field::draw` calls it before drawing its control. `inherit_forced` may appear only under `crates/tui/src/components/**` and `crates/tui/src/field_control.rs`, and never in a `pub fn` signature outside the trait default (§28 P5). <!-- amended by §28 -->

### 12.2 The one collection vocabulary

<!-- amended by §21 items 1, 5, 8, 21, 22, 30; §24 -->

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
    pub fn label_spans(&mut self, spans: &[Span<'_>]);   // `Span` is OURS (role-carrying, `crates/tui/src/text/span.rs`, §24 M1); paints span-by-span through Buffer::set_span INSIDE ui/paint.rs, 0 allocations (§22 R‑3 as amended by §25 F4). `Ui::paint_spans(area, spans, base)` (A2) is the same paint without a row; `base` is the LABEL part style the spans inherit. <!-- amended by §25 -->
    pub fn meta(&mut self, s: &str);                  // dropped all-or-none (DESIGN.md:478)
    pub fn trailing(&mut self, s: &str, p: &StylePatch);
    pub fn columns(&mut self, widths: &[Track]) -> ColumnsUi<'_>;   // ≤ MAX_COLUMNS = 16 tracks, extra tracks ignored (documented cap, §25 MI‑8); 0-alloc via layout::distribute_into <!-- amended by §25 -->
    pub fn indent(&mut self, depth: u16);
    pub fn part(&mut self, p: Part, width: u16) -> CellUi<'_>;   // reserves `width` columns from the RIGHT; `label` fills what is left
    pub fn label_fmt(&mut self, args: core::fmt::Arguments<'_>);   // in-place formatting, 0 allocations (DefaultRow)
    pub fn raw(&mut self) -> (&mut Buffer, Rect);     // escape hatch, marks the ROW's rect written (Ui::buffer_in, never Ui::raw — §25 F3)
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

**Scrolling is shared:** `ScrollRegion<'a>` is a component (`components/scroll_region.rs`, `ScrollRegionCase` in the conformance suite) providing scrollbar registration, track arithmetic, thumb drag through pointer capture, and `ensure_visible_on_next_layout`, with a `Ui::scroll_region(id, part, …)` convenience that constructs and draws it — deleting seven copies of `on_scrollbar` (**[F]** DOM §6.1(6)) (M25). <!-- amended by §25 --> `Ui::scroll_region`'s exact signature is not yet in §17.0 A2; it is a Slice-3-owned `Ui` method and an open item (§25.7) that blocks 4E, not 4A or 4F. The scrollbar is `Part::TRACK`/`Part::THUMB` of its container, not a separate id space; `scrollbar::id_for` is deleted.

### 12.3 Grid split (confirms DOM §1.5)

<!-- amended by §21 items 1, 22, 30; §23 K2 -->

`DataTable` is deleted; `Grid` is the one tabular component. The reusable `Grid` keeps: columns (`key: ColumnKey`, `title`, `subtitle`, `align`, `min/max width`, `sortable`, `editable`, `sticky`, `prefix_glyph`, `badge`), two-axis viewport and column width sampling, `‹N`/`N›` overflow, cursor cell, rectangular range selection, row selection, copy-as-TSV, fetch-more row, `EmptyState`, an explicit edit lifecycle, an **action-surface slot**, `rows_label`/`cols_label`, and `NavUnit::{Row, Cell}`.

```rust
pub trait GridModel {
    type Row;
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> ItemKey;
    fn cell(&self, row: usize, col: usize) -> CellRef<'_>;         // borrowed text + tone + align
    fn row_decor (&self, row: usize)             -> RowDecor<'_>  { RowDecor::default() }
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> { CellDecor::default() }
    fn total(&self) -> RowTotal { RowTotal::Unknown }
    fn has_more(&self) -> bool { false }
    // moved down from GridEditor (§23 K2, G3): `draw<M: GridModel>` renders the reason and must see it
    fn read_only_reason(&self) -> Option<&str> { None }
    // absorbed from the deleted `GridCellActions` (§23 K2, G3): `draw` paints the `→` affordance and
    // registers its hot zone, so it must see it; the `&[]` default keeps read-only models four-method
    fn actions(&self, _row: usize, _col: usize) -> &[CellAction] { &[] }   // glyph + chord + ActionKey
}
pub trait GridEditor: GridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent;   // Inline{initial}|Cycle|External|Refuse{reason}
    // External: the grid emits GridAction::EditRequested(item, col) and does NOT begin an inline
    // edit; the application opens its own editor (A8). An inline editor registers a Control region
    // AFTER the grid's cell Part region and wins the click (§21 item 30).
    fn apply_cycle(&mut self, row: usize, col: usize);
    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError>;
    fn is_editable(&self, row: usize, col: usize) -> bool;
}
// `GridCellActions` is DELETED (§23 K2): as a third trait it was unreachable from `draw<M: GridModel>`,
// and a second bound on `draw` would have forced every read-only model to implement it.

impl<'a> Grid<'a> {
    /// Read-only navigation, selection, sorting, copy, fetch-more, filter and cell actions.
    /// `&M`: a read-only grid CANNOT mutate its model — a compile-time fact, not a runtime refusal (G1).
    pub fn update<M: GridModel + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &M) -> Response<GridAction>;
    /// Everything `update` does, plus the inline edit lifecycle: begin, cycle, commit, cancel, blur.
    /// The ONLY place `GridEditor`'s `&mut self` methods are reachable (G2).
    pub fn update_editable<M: GridEditor + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;
    /// One draw for both. Bound is the base trait, symmetric with `update`.
    pub fn draw<M: GridModel + ?Sized>(
        &self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
}
```

`GridModel` is `&self` (used by both phases). Capability is chosen by the **entry point**, never by a flag: `Grid::editable(bool)` does not exist (§23 K2, G4 — it was the boolean soup §13 forbids and could contradict the model's own `is_editable`). `update` and `draw` carry the same bound and the same shared borrow, so a navigating screen writes one bound in both phases; `update_editable` is `update`'s body plus the edit arms over one private `fn navigate(…)`. The model is a phase parameter, never a field of `Grid<'a>` (§21 item 1), and the phase split makes "rendering stages a database mutation" (`grid.rs:1518`) unrepresentable. TablePro's read-only-with-reason grids (views, no-PK tables, unknown-source results with `local_sort`) call `update_editable` with an adapter whose `is_editable` returns `false` and whose `read_only_reason` returns `Some` — a runtime property of an editor-capable model, no wrapper type; display-only grids (`tests/fixtures/grid_model.rs`, the six Structure-tab models, the showcase grid page) implement `GridModel` alone and call `update`. Invariants G1–G7 and the rejected alternatives are in §23 K2. Everything database-shaped moves to `apps/tablepro/src/grid_model.rs`: `CellValue`, `PendingChanges`, `UndoAction`, `RowState` derivation, `default_validator`, `cmp_cells`, `apply_commit_result`, insert/duplicate/toggle-delete/discard/undo, `primary`/`nullable`/`references`/`enum_values`, `pending_label`, the Save/Discard/Preview action bar, and `Theme::change_glyph`. All 22 TablePro capabilities survive by the mapping in DOM §1.6, which is adopted verbatim as the migration checklist for Slice 6.

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
| collection data | passed per phase, never held in props | `update(cx, &mut st, items)` / `draw(ui, area, &st, items)`; `Grid::update` takes `&M: GridModel`, `Grid::update_editable` takes `&mut M: GridEditor`, `Grid::draw` takes `&M` (§21 item 1, §23 K2) |
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
| errors | `FieldError` | typed, `Display + core::error::Error` (`core`, not `std`, matching ratatui — §22); no panic on any interaction path (`LayoutError` deleted, §21 item 19) |
| testing | `Harness` | `Harness::new(app, theme, w, h)`, `.key()`, `.click_id(id)`, `.records()`, `.snapshot()` (§16.4) |
| API layers | `junie_tui::*` vs `junie_tui::author::*` | both `pub`, separately documented |

Additional binding rules: no boolean parameter soup (typed enums for semantically different modes); no gratuitous generics in application-visible signatures (collection generics are inferred and die at the call site); no public mutable rect or cache; no `'static` bound in any component's public surface; complete rustdoc on every public item (`#![deny(missing_docs)]`).

<!-- amended by §21 items 1, 4, 19, 30, 33 -->

**A layer owner's `update` runs unconditionally (binding).** A component that owns a layer runs its `update` **every frame**, whether or not the layer is open. `cx.is_open(id)` guards the work the **caller** does besides the component — opening the layer, sizing it from live data, closing it from an action — never the component's own `update`: dismissal is delivered as intents addressed to the layer's owner in the pass **after** the layer closed, and a gated call drains nothing, so the guard skips exactly the pass carrying the event its author wanted (§28 P3). <!-- amended by §28 -->

**Props are built once (binding).** A component instance with any configuration beyond `new(id, …)` is built by exactly one private constructor function on the owning screen, called from both phases. The constructor takes the fields it needs as parameters, never `&self`, so `update` can still pass `&mut` to disjoint fields; a controlled `.value(&T)` added in `draw` is the documented per-phase difference. `architecture::props_are_built_once` (a `syn` check that no configured `X::new(` appears more than once per screen module for the same `const Id`) reports violations. `Form` (J2, §15.1) declares each field once as a `FieldSpec::new(id, …)` inside the `&[FieldSpec]` returned by one private constructor, and `Form` drives both phases <!-- amended by §23 -->. Without this, a `disabled(…)` predicate applied in `draw` but forgotten in `update` is a silent bug the compiler cannot see.

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
pub struct KeyMap { /* add / remove / remap, per KeyPhase; EMPTY / EMPTY_REF in §17.0 A1; conflicts() is computed once per keymap change (or under debug_assertions), never per input — §25 MI‑4 */ }
pub enum KeyPhase { Capture, Bubble }                       // was `Phase2`
pub struct HintLayer { pub hints: Vec<Hint>, pub badge: Option<&'static str>,                     // Vec, not SmallVec (§22)
                       pub status: Option<Cow<'static, str>>, pub centered: bool }   // 0 allocs/frame when focus is unchanged (P1): cached in Ui::cache
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
| J2 | Form dialog / field group | **Library component** — `Form` (ordered fields, visibility, scroll-to-focused-field, action row, error row, nested popup) + `Field<C>`. Three independent form engines collapse to one. Each field is declared once as a `FieldSpec::new(id, …)` and `Form` drives both phases; the exact API is §15.1 (§23 K1). <!-- amended by §23 --> |
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

pub struct SecretPolicy { pub mask: GlyphRole, pub synthetic_tail: usize }   // Default: mask = GlyphRole::SecretMask, never Dirty (§25 D‑11) <!-- amended by §25 -->

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

**Decisions.** `Field<C>` owns label (`*` required, `optional` suffix), help/error row, gutter and height (never focus registration, which stays with the control — §21 item 7) — deleting the per-control re-implementations and the `plain_label` flag, and deleting `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic from three apps. Values are **controlled** (`&mut String`), so the "rebuild the widget to change its value" idiom (five sites) disappears. Blur is an explicit intent-driven transition with a per-control policy (`CommitAndValidate` for `TextInput`, `Commit` for `TextArea`/`CodeEditor`, `Cancel` where a dialog demands it) — removing all five render-time commits. `RadioGroup` separates cursor from value. Masked fields render a synthetic tail, never the real characters (the safety property moves from jackin into the library, closing **[F]** API §5 item 13). Manual `Debug` impls redact on `TextInput`, `TextInputState`, `Field`, `Dialog`, `Form`, `EditState`, `TextEditorCore`; `conformance::secret_never_appears_in_debug` asserts it. `TextEditorCore::zeroize` overwrites before drop. <!-- amended by §25 MA‑13 --> `Secret::zeroize` / `TextEditorCore::zeroize` fill the bytes and then `core::hint::black_box(&bytes)` + `compiler_fence(Ordering::SeqCst)` so the stores are not elided, under `#![forbid(unsafe_code)]`; the property tested in safe Rust (`text::zeroize_overwrites_before_drop`) is that the capacity is released and a fresh `expose()` is empty, and the compiler-elision risk is recorded in the code as a known limit of safe-Rust zeroization.

### 15.1 `Form` — the declared-field form component (Adjudication K, K1)

<!-- added by §23 K1; amended by §24 M3 -->

**Decision.** `Form` is a library component (`components/form.rs`, work package 4F), **not** a `FormState` + `layout::form` helper pair. It is a *composition* (disposition C), not an overlay: it opens no layer, traps no focus, paints no frame; it is placed inside a `Dialog` body slot, a `Panel`, or a bare rect. Reasons, in decreasing weight: (1) three required capabilities — Enter-submits arbitration against the focused control's `EDITING`/`swallows_typing`, validate-then-focus-first-error, and commit-of-the-in-flight-edit before validation — are `update`-phase semantics; a draw-time helper cannot own any of them under `&self`/`&XState` (§3.1); (2) scroll-to-focused-field is today a render-time mutation (**[F]** `modals.rs:1356-1371`) whose only legal home is `update`, which needs the field order, field heights and focus — the component's own knowledge; (3) `architecture::conformance_covers_every_public_component` makes registration mandatory and `FormCase` is already in `conformance_suite!` (§16.2); a helper pair would drop `draw_does_not_commit_or_cancel` and `secret_never_appears_in_debug` for the highest-risk surface in the repository; (4) §14.2 J2 already decided "library component". The capability inventory each of the three current engines carries (ordered data-declared fields; input/select/checkbox/radio/chooser/note/textarea/toggle kinds; secret field; conditional visibility; sections/tabs; two-column and half-width pairs; automatic Tab order; scroll-to-focused-field; per-field and form-level validation with focus-to-first-error; dirty tracking; an action row with submit/cancel/extra; Enter-submits policy; Left/Right traversal of the action row; nested popover; nesting inside a dialog; chooser field; note rows; cross-field effects) survives by the mapping in F1–F13.

**Exact API.**

```rust
// ───────────────────────────── identity ─────────────────────────────
// A field is addressed by its own `Id`. There is NO new key type: §7.1's `Id` is
// Copy + Eq + Hash, and "matching a screen's own const Ids" is the one form of
// manual dispatch §14.1 leaves in application code.

// ───────────────────────────── declaration ──────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum FieldSpan { Full, Half }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct GroupKey(u16);
impl GroupKey { pub const ALL: GroupKey; pub const fn custom(name: &'static str) -> GroupKey; }

// <!-- amended by §24 M3 --> The three choice controls are ordinary collections (`<'a, T, K, R>`, items per
// phase, §18.2 / §21 items 1 and 5). A form field holds their DEFAULT instantiation under three aliases;
// `ByIndex` is forced by the value channel (`FieldMut::Choice(&mut usize)` is positional, M3‑3), not a leaked default.
pub type LabelSelect<'a> = Select    <'a, &'a str, ByIndex, DefaultRow>;
pub type LabelRadio <'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;
pub type LabelChips <'a> = ChipBar   <'a, &'a str, ByIndex, DefaultRow>;

/// Configuration only — no values, no `&mut`, no `Id`-less chrome, no item slice. Every variant is a
/// props struct built by the same consuming builders it has standalone (§13). Closed and non-generic:
/// one lifetime, zero type parameters (`architecture::field_kind_has_no_type_parameters`, §24 M3).
pub enum FieldKind<'a> {
    Text   (TextInput<'a>),          // `.secret(SecretPolicy)` already applied for secrets
    Area   (TextArea<'a>),
    Select (LabelSelect<'a>),        // options arrive through `FormData::options` (below), never here
    Radio  (LabelRadio<'a>),
    Chips  (LabelChips<'a>),
    Check  (Checkbox<'a>),
    Toggle (Toggle<'a>),
    Chooser(Button<'a>),             // button + a read-only value line + optional detail — the escape hatch
                                     // for a keyed / custom-row / non-string collection (M3‑4)
    Note,                            // static rows; no focus stop, Decorative regions only
}

pub struct FieldSpec<'a> {
    pub id:       Id,
    pub label:    &'a str,
    pub kind:     FieldKind<'a>,
    pub required: bool,
    pub help:     Option<&'a str>,
    pub span:     FieldSpan,         // Full | Half — the two-column form
    pub group:    GroupKey,          // section/tab membership; GroupKey::ALL is always shown
    pub plain:    bool,              // forwarded to Field::plain (kills `TextInput::plain_label`)
}
impl<'a> FieldSpec<'a> {
    pub const fn new(id: Id, label: &'a str, kind: FieldKind<'a>) -> Self;
    pub const fn required(self, yes: bool) -> Self;
    pub const fn help(self, s: &'a str) -> Self;
    pub const fn span(self, s: FieldSpan) -> Self;
    pub const fn group(self, g: GroupKey) -> Self;
    pub const fn plain(self, yes: bool) -> Self;
}

// ───────────────────────────── the data channel ─────────────────────
// Values are the CALLER's. They are passed to each phase call, exactly like `Grid`'s
// model (§21 item 1) and `List`'s items. `Form<'a>` never holds a value or a `&mut`.
pub enum FieldMut<'d> {
    Text  (&'d mut String),
    Secret(&'d mut Secret),
    Choice(&'d mut usize),
    Flag  (&'d mut bool),
    Chips (&'d mut KeySet),
    ReadOnly,                        // Chooser / Note: no controlled value
}
pub enum FieldRef<'d> {
    Text   (&'d str),
    Secret (&'d Secret),             // masked by Secret::write_mask; never stringified
    Choice (usize),
    Flag   (bool),
    Chips  (&'d KeySet),
    Display{ value: &'d str, detail: Option<&'d str> },   // Chooser
    Note   (&'d [(&'d str, Role)]),
}

pub trait FormData {
    fn value    (&self,     id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;

    /// <!-- amended by §24 M3 --> Option labels for a `Select` / `Radio` / `Chips` field; `&[]` for every
    /// other kind. Borrowed from the caller — never `'static` (§21 item 22). Painted, never returned in a
    /// `FormAction` (F5). `Form::draw` calls `value(id)` + `options(id)` (two shared borrows).
    fn options(&self, _id: Id) -> &[&str] { &[] }

    /// <!-- amended by §24 M3 --> The controlled value and the option list under ONE borrow, so
    /// `Form::update` can drive a choice control without a second `&mut` (E0502). A data type with option
    /// tables overrides it by destructuring its own disjoint fields; the default is correct for every
    /// non-choice kind, and a choice field that forgets it renders an empty list on the first frame.
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), &[])
    }

    fn visible  (&self, _id: Id) -> bool { true }
    fn disabled (&self, _id: Id) -> bool { false }
    /// External / async / server-side errors. Per-field local errors live in `FormState`.
    fn error    (&self, _id: Id) -> Option<&str> { None }
    fn validate (&self, _id: Id, _v: FieldRef<'_>) -> Result<(), FieldError> { Ok(()) }
    /// Cross-field rules. Runs after every per-field check passes.
    fn validate_all(&self) -> Result<(), (Id, FieldError)> { Ok(()) }
}

// ───────────────────────────── the component ────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterPolicy { SubmitsWhenIdle, Never }   // typed enum, not a bool (§13 "no boolean soup")

pub struct Form<'a> { /* id, fields, actions, submit, cancel, enter, columns, group */ }

impl<'a> Form<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::BODY, Part::ACTIONS,
                                         Part::HELP, Part::MARKER, Part::TRACK, Part::THUMB];
    pub fn new(id: Id, fields: &'a [FieldSpec<'a>]) -> Self;
    pub fn actions(self, a: &'a [Action<'a>]) -> Self;    // §17.0 A4; `.chord()` gives Ctrl+S
    pub fn submit(self, k: ActionKey) -> Self;            // default ActionKey::SAVE
    pub fn cancel(self, k: ActionKey) -> Self;            // default ActionKey::CANCEL
    pub fn enter(self, p: EnterPolicy) -> Self;           // default SubmitsWhenIdle
    pub fn columns(self, n: u8) -> Self;                  // default 1
    pub fn group(self, g: GroupKey) -> Self;              // active section; default GroupKey::ALL
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;

    pub fn update<D: FormData + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut FormState, data: &mut D)
        -> Response<FormAction>;
    pub fn draw<D: FormData + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &FormState, data: &D)
        -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormAction {
    Changed  (Id),          // a control reported a value change (draft level)
    Committed(Id),          // a control committed; validation for that field has run
    Chose    (Id),          // a Chooser button fired — the owner opens its own picker
    Action   (ActionKey),   // any action-row button, INCLUDING submit once it validates
    Invalid  (Id),          // submit refused; `Id` is the first invalid field, already focused
}
// FormAction carries no value of any kind. This is the secret-containment invariant (F5).

pub struct FormState { /* slots: Vec<FieldSlot> keyed by Id (Vec, not SmallVec — §22), scroll: ScrollState,
                          errors: Vec<(Id, FieldError)>, dirty: bool, gen stamp */ }
impl FormState {
    pub fn is_dirty(&self) -> bool;
    pub fn mark_clean(&mut self);
    pub fn error(&self, id: Id) -> Option<&FieldError>;
    pub fn set_error(&mut self, id: Id, e: Option<FieldError>);
    pub fn clear_errors(&mut self);
    pub fn reveal(&mut self, id: Id);          // request scroll-to-field on the next layout
    pub fn zeroize(&mut self);                 // overwrites every secret draft (§15)
}
impl Default for FormState { /* empty; slots are created by the first `update` */ }
impl Reconcile for FormState { /* slots follow the declared field ids, §21 item 21 */ }
impl fmt::Debug for FormState { /* manual; every draft renders as "[redacted]" */ }
```

**Invariants (each a test below).**

* **F1 — Props hold no data.** `Form<'a>` holds `&'a [FieldSpec<'a>]`, `&'a [Action<'a>]` and scalars. It never holds a `String`, a `&mut`, or `&self` of a screen. One private constructor per screen returns the field array; both phases call it (`architecture::props_are_built_once`).
* **F2 — Tab order is declaration order.** `Form::draw` registers each visible, focusable field's control in `fields` order; §8.1's *traversal order is registration order* supplies Tab and Shift+Tab. `Form` never calls `cx.focus` for traversal. Deletes `modals.rs:1084-1091` and `connections.rs:587-588`.
* **F3 — Hidden fields register nothing and keep their drafts.** `FormData::visible(id) == false` ⇒ no ring entry, no region, no measure contribution; the `FieldSlot` survives in `FormState` keyed by `Id`, so toggling back restores the draft and the cursor. Replaces `modals.rs:815/866-869/984-988` and the render-time `disabled` write at `connections.rs:1205-1206`.
* **F4 — Field height is a pure function of `(FieldSpec, DesignTokens, width)`.** Both phases compute it identically; `update` reaches the tokens through `FrameRead::design()` on `Cx` and the body width through `cx.area(self.id)` (last frame; `None` on frame 1 is a documented no-op, S3). This is what lets `update` own scroll-to-focused-field without `Ui`. Deletes the `TextInput::HEIGHT`/`Select::HEIGHT`/`RadioGroup::height()` arithmetic at `connections.rs:1144-1186` and `modals.rs:871-880`.
* **F5 — No value ever leaves the form.** There is no `values()`. `FormAction` carries only `Id` and `ActionKey`. See "secrets" below.
* **F6 — `draw` commits nothing.** `Form::draw` is `&self` + `&FormState`; the blur commit is an `Intent::FocusOut`-driven transition in `update` with the control's `BlurPolicy` (§15). Covered by `conformance::form::draw_does_not_commit_or_cancel` (case 7).
* **F7 — At most one `FormAction` per frame.** `Response<A>` holds `Option<A>` and `BitOr` is defined only for `Response<()>` (§21 item 4). `Form::update` folds each control's `flow`/`invalidate` and keeps the **first** action in declaration order, ordering action-row buttons last; the rest are recorded as `invalidate` only. This is the rule the ladder implements today by `return`ing on the first hit (`modals.rs:1112`, `:1126`).
* **F8 — Nested popovers are layers, not draw order.** An open `Select` opens a `Popover` via `cx.open_layer` in its own `update`; its content is painted inside `ui.layer` from `Select::draw`. Z-order is the `LayerId` assigned at open, not the call order (§21 item 14). `Form` needs no `any_open_select()` guard (`modals.rs:1068-1072`), no deferred re-render (`modals.rs:1514-1539`), and no manual hit re-registration (`modals.rs:1491-1512`). Esc reaches the `Select` before the layer and before the enclosing dialog (§21 item 3).
* **F9 — `Form` is layer-agnostic.** Inside a `Dialog`, the trap, backdrop, Esc and focus restore belong to the layer (§9.1). `Form` registers its container and section chrome as `Decorative` (§21 item 13), so a click on empty form background is an ordinary miss and never records `UndeliveredIntent`.
* **F10 — Submit sequence is fixed and total.** On the submit `ActionKey`: (1) blur-commit the focused control if editing; (2) for every **visible** field in declaration order, `FormData::validate(id, value)`; (3) `FormData::validate_all()`; (4) on the first failure — `st.set_error`, `st.reveal(id)`, `cx.focus(id)`, emit `FormAction::Invalid(id)`; (5) otherwise emit `FormAction::Action(submit)`. Replaces `connections.rs:246-250` + `:704-711` and gives jackin's `FormDialog` the validation it has none of (**[F]** DOM §3.2).
* **F11 — Enter-submits is arbitrated, not guessed.** `EnterPolicy::SubmitsWhenIdle` submits only when the focused control's focus entry does not set `swallows_typing` and does not carry `StateFlags::EDITING`. Generalises `modals.rs:1204-1212`'s `if !editing` and the six ad-hoc `!editing` guards §13.1 names. A submit chord (`Ctrl+S`, `connections.rs:577`) is declared as `Action::new(SAVE, "Save").chord(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL))` — no new API (§17.0 A4).
* **F12 — Dirty is set only by a committed change.** `FormAction::Committed` and a toggle/choice change set `st.dirty`; a keystroke inside a draft does not. This corrects `modals.rs:1119-1121`, where a mid-edit `InputEvent::Changed` sets `dirty` before anything is committed.
* **F13 — `FormState` redacts.** Manual `Debug` on `FormState` and every `FieldSlot`; `zeroize()` overwrites secret drafts before drop (§15).
* <!-- amended by §24 --> **M3‑1 … M3‑4** (§24) — `FormState` stores no props (`SlotValue { None, Text, Choice(usize), Flag(bool), Chips(KeySet) }` is `Clone + PartialEq + Eq`; no `FieldKind` is reachable from state); `FieldKind` holds configuration only and the item slice never enters it; the form's identity model is positional by construction (`ByIndex`); `Chooser` is the escape hatch for a richer control.

**How values reach the application without cloning secrets — exactly.** There is no `values()`; the mechanism is elimination, not redaction. **[F]** Today `FormDialog::values()` (`modals.rs:1039-1044`) clones **every** field name and value into `FormValues = Vec<(String, FieldValue)>`, including password fields, and DOM §3.3 traces that vector through `ModalResult::Form(Some(values))` into `accounts.rs` — every API key in jackin is copied at least twice per submit. Under this design the values were never inside the form: the caller's draft struct is an ordinary field of the screen; `Form::update` reaches each value through `FormData::value_mut(id)` for the duration of one control's `update` call and writes through it; `Form::draw` reaches it through `FormData::value(id)` and reads it; both borrows end when the phase method returns. On submit the screen already owns the values, so the "result" crosses no boundary — `FormAction::Action(ActionKey::SAVE)` is a 2-word `Copy` enum. Secrets specifically: `FieldMut::Secret(&mut Secret)` / `FieldRef::Secret(&Secret)`; `Secret` is **not `Clone`, not `PartialEq`, not `Serialize`** (§15), so a clone is a compile error, not a review item. `Form::draw` renders it with `Secret::write_mask(&mut CellUi, n)` (§21 item 30 P5), which writes cells with a **synthetic** tail — no `String` of the secret is ever constructed. The in-flight draft lives in the field's `TextInputState` inside `FormState`, whose `Debug` is manual and redacting (F13) and whose `zeroize()` overwrites it. Closing the enclosing layer calls `FormState::zeroize()` on `LayerEvent::Dismissed`/`Closed`, so a cancelled password is overwritten rather than dropped.

**Rejected alternatives.**

| Rejected | Why |
|---|---|
| **`Form` holds `&'a mut` per field** (a `FormFields<'a>` built fresh in `update`) | The props struct would be built differently in the two phases — the exact silent-bug class §13's props-built-once rule exists to kill, and `architecture::props_are_built_once` would flag it. It also re-creates the B3 borrow conflict §21 item 1 removed. |
| **`FormState` + `layout::form` helper pair** | Enter arbitration, validate-then-focus, blur-commit and scroll-to-field are `update`-phase semantics; a `draw` helper cannot own any of them. Also drops `FormCase` out of `conformance_suite!`. |
| **Wide `FormData` with one accessor per type** (`text/text_mut/choice/set_choice/flag/…`) | 12+ methods, all defaulted, so a forgotten override compiles into a silently empty field. The two-accessor `FieldMut`/`FieldRef` shape makes the screen's `match` exhaustive over its own `const Id`s. |
| **A `FieldKey(u16)` newtype** | A second key space over the `Id` that already keys focus, hits, `cx.area`, `Response::for_id` and `Harness::area_of`. §7.1 already forbids parallel identity. |
| **`FormData::on_change(id)` hook** | Redundant with `FormAction::Committed(id)`, and a `&mut self` callback re-entered from inside `update` re-opens draw-order-style implicit ordering. |
| **`values() -> FormValues` kept for convenience** | It is the secret-leak mechanism, and `Secret: !Clone` makes it uncompilable for secret fields anyway. |
| **`bool` parameters for enter/columns/sections** | §13 "no boolean parameter soup"; hence `EnterPolicy`, `FieldSpan`, `GroupKey`. |
| **`Form` opening its own layer** | Duplicates §9's stack; a form is used both inside a dialog (`modals.rs:1340`) and inline on a page (`connections.rs:1130-1132`). |

**Acceptance conditions (executable).** <!-- amended by §28: the render line names both targets -->

```bash
cargo test -p junie-tui --lib form::
cargo test -p junie-tui --test conformance conformance::form::
cargo test -p junie-tui --test render --test render_components render::components::form::   # both targets (§16.3, §28)
cargo test -p junie-tui --test architecture architecture::props_are_built_once
cargo test -p tablepro -p jackin-preview
cargo test -p junie-tui --test perf --release -- --test-threads=1 frame_tablepro_connection_form
```

Named tests (listed in §16.1, §16.4 and §16.6 so `architecture::every_named_test_exists` covers them): `form::tab_order_follows_declaration_order_skipping_hidden`, `form::hidden_field_registers_no_ring_entry_and_keeps_its_draft`, `form::field_height_is_a_pure_function_of_spec_and_design_tokens`, `form::scroll_reveals_the_focused_field_from_update_not_draw`, `form::submit_commits_the_in_flight_edit_before_validating`, `form::submit_validates_every_visible_field_then_focuses_the_first_error`, `form::submit_skips_hidden_fields_during_validation`, `form::enter_submits_only_when_the_focused_control_is_not_editing`, `form::submit_chord_is_declared_on_the_action_not_baked_in`, `form::dirty_is_set_by_a_commit_not_by_a_keystroke`, `form::chooser_activation_emits_chose_with_the_field_id`, `form::note_rows_register_only_decorative_regions`, `form::at_most_one_action_per_frame_in_declaration_order`, `form::open_select_popover_traps_focus_and_esc_closes_only_the_popover`, `form::form_action_variants_carry_no_value` (exhaustive `match`, one arm per variant), `form::zeroize_overwrites_every_secret_draft`, `form::every_declared_field_resolves_a_value` (debug-assert-backed, run by the app suites over each real `FormData`), <!-- amended by §24 --> `form::select_field_options_come_from_form_data`, `form::changing_options_between_frames_does_not_rebuild_props`, `form::state_holds_no_props`, `form::value_and_options_is_a_single_borrow` (§24 M3); generated: `conformance::form::draw_does_not_commit_or_cancel` (`Caps::EDITS`), `conformance::form::secret_never_appears_in_debug` (`Caps::SECRET`), `conformance::form::survives_tiny_rects_0x0_to_3x3`, `conformance::form::draw_twice_leaves_state_equal`; application: `tablepro::connection_form_keyboard_and_mouse_reach_every_field`, `tablepro::connection_form_focuses_the_first_invalid_field`, `tablepro::connection_password_is_masked_and_absent_from_the_frame` (closes the **[F]** `connections.rs:155-157` defect — the password is a plain `TextInput` today), `jackin::form_dialog_toggles_visibility_and_keeps_drafts`, `jackin::form_dialog_secret_never_reaches_the_screen_as_a_string`; perf: `frame_tablepro_connection_form_120x40` — **< 40 allocs/frame**.

Grep conditions:

```bash
! rg -n 'fn values\(|FormValues|FieldValue::' src/bin apps/           # the cloning channel is gone
! rg -n 'focus\.(next|prev|focus)\(' apps/tablepro/src/connections.rs apps/jackin-preview/src/screens/
! rg -n 'TextInput::HEIGHT|Select::HEIGHT|\.height\(\) *\+' apps/      # manual field arithmetic
```

**What disappears** in TablePro: `on_form_key` (`connections.rs:573-687`, 115 lines with the `input!` macro), `on_form_click` (`:793-861`), `on_paste` (`:863-887`), `render_form`'s height arithmetic (`:1144-1228`), `validate()` (`:246-250`), the focus-to-first-error block (`:704-711`), the widget rebuild at `:629-631`, and the render-time writes at `:1134` and `:1205-1206`. `ConnForm` (`:62-87`) becomes a 15-owned-value `Draft`, which §20.7 already accepted as the cost of controlled values.

**Risks.**

1. **`FieldKind` is a closed enum.** A downstream control cannot be a form field. Mitigation: `FieldKind` is the *library* set; a custom control is composed beside the form, or `FieldKind::Custom(&'a dyn FieldControl<State = …>)` is added later — deliberately deferred, because the associated `State` type makes a uniform `dyn` slot non-trivial and no current consumer needs it. <!-- amended by §24 --> A keyed, custom-row or non-string choice inside a form is `FieldKind::Chooser` plus the owner's own `Picker`/`Select` layer (§24 M3‑4); `FieldKind` never grows type parameters (§24 M3 rejected b1).
2. **`FieldSlot` is a per-kind enum inside `FormState`**, so `FormState: Clone + PartialEq` (S2) requires every control state to be. All are (§16.2 `Conformance::State`).
3. **First-frame scroll.** `cx.area(FORM)` is `None` on the first frame, so scroll-to-focused-field is a no-op then (S3). Visible only if a form opens already scrolled; documented, and one frame later it corrects.
4. **`FormData` match ladders are unchecked for coverage.** A missing arm falls to `_ => FieldRef::Text("")`. Mitigation: the `_` arm should be `unreachable_field(id)` in app code, and `form::every_declared_field_resolves_a_value` runs over each real `FormData`.
5. **`Secret` in the draft means `Draft: !Clone`.** TablePro's `ConnForm::to_connection(base)` currently clones (`connections.rs:213-244`); the migration must move to a by-reference build. Slice 6 work, listed in the DOM §1.6 checklist style.

---

---

## 16. Testing strategy

Every test named below is a real, runnable name. Builders create them with exactly these names; §16.5's `architecture::every_named_test_exists` asserts the inventory in this section matches the compiled test list, so a renamed or missing test is a build failure rather than a silent gap.

**Where tests live.**

| Level | Location | Runner |
|---|---|---|
| unit | `#[cfg(test)] mod tests` inside each library module | `cargo test -p junie-tui --lib` |
| conformance | `crates/tui/tests/conformance.rs` + `crates/tui-testing/src/conformance/` | `cargo test -p junie-tui --test conformance` |
| rendering / digest | `crates/tui/tests/render.rs` + `crates/tui/tests/render_components.rs` (§16.3: the test *path* is the contract, not the file), `apps/*/tests/visual.rs` | `cargo test --workspace --test render --test render_components --test visual` <!-- amended by §28 --> |
| application integration | `apps/*/tests/app_tests*.rs` (moved out of the binaries, see §18) | `cargo test -p showcase -p tablepro -p jackin-preview` |
| architecture | `crates/tui/tests/architecture.rs` + `xtask` | `cargo test --workspace --test architecture` |
| performance | `crates/tui/tests/perf.rs`, `apps/*/tests/perf.rs` | `cargo test --workspace --test perf --release -- --test-threads=1` |

`crates/tui-testing` is a **dev-only** crate (`publish = false`, depended on with `[dev-dependencies]` only) so the counting allocator, the `Harness` and the conformance driver never reach a shipped binary.

---

### 16.1 Unit tests (goal §25.1)

<!-- amended by §21 items 3, 4, 12, 14, 15, 29, 30, 33; §22; §23; §24; §25; §26 -->

One `#[cfg(test)] mod tests` per module. Names are given verbatim; the module path is the test path.

**`id.rs`** — identity, goal §25.1 "stable identity"
`root_sub_part_index_item_are_all_distinct`, `separator_prevents_concatenation_collision` (asserts `Id::root("a").sub("b") != Id::root("ab")` **and** `Id::root("ab").sub("") != Id::root("a").sub("b")`), `kind_tag_separates_name_from_item_with_equal_bytes`, `id_equality_is_exactly_hash_equality` (§25: over a ~200-id corpus built by every derivation, `a == b ⇔ a.hash() == b.hash()` and `a.cmp(b) == a.hash().cmp(&b.hash())`), `id_equality_ignores_debug_label` (`#[cfg(debug_assertions)]` only: a label never changes an answer), `id_is_const_constructible`, `item_key_text_is_stable_across_runs`, `item_key_pair_is_order_sensitive`, `part_custom_lands_in_the_high_range`, `part_constants_are_unique`, `debug_prints_path_in_debug_builds`, `debug_prints_hash_in_release_builds`.

**`response.rs`** — event consumption and invalidation
`ignored_consumed_changed_action_constructors`, `bitor_takes_consumed_over_ignored`, `bitor_takes_max_invalidate`, `bitor_is_defined_only_for_unit` (compile-fail via `trybuild`, §21 item 4), `repaint_raises_relayout_raises_further`, `layout_is_strictly_greater_than_paint`, `no_repaint_lowers_to_none`, `map_action_preserves_flow_and_invalidate`, `erase_drops_the_action_only`, `must_use_is_enforced` (compile-fail via `trybuild`), `state_flags_round_trip`.

**`intent.rs` / `event.rs`**
`key_release_is_dropped`, `unmapped_mouse_button_is_dropped`, `mouse_carries_modifiers`, `chord_hashes_by_code_and_mods`, `secondary_up_is_modelled`, `wheel_carries_axis_and_delta`, `paste_reaches_only_an_editing_owner`.

**`focus.rs`** — traversal, scopes, restoration, disabled/read-only
`tab_cycles_forward_and_backward`, `shift_tab_is_the_exact_reverse`, `disabled_entries_are_registered_but_skipped`, `read_only_entries_stay_in_the_ring`, `click_only_entries_are_never_reachable`, `trap_confines_traversal_to_the_scope`, `trap_wraps_inside_the_scope`, `nested_scopes_resolve_innermost_first`, `scope_restore_returns_focus_to_the_opener`, `reconcile_prefers_nearest_surviving_entry_by_previous_index`, `reconcile_falls_back_to_scope_first_enabled`, `reconcile_falls_back_to_innermost_active_scope`, `reconcile_yields_none_when_nothing_is_reachable`, `focus_visible_is_true_only_after_a_key`, `trap_is_armed_when_the_layer_is_pushed_not_when_it_draws`, `restore_target_receives_keys_before_the_next_draw` (§21 item 15). <!-- amended by §25 MI‑3 --> `read_only_entries_stay_in_the_ring`, `click_only_entries_are_never_reachable` and `restore_target_receives_keys_before_the_next_draw` are runtime-level tests (the mechanism lives in `Ui::register_entry` and `Runtime`, not in `FocusRing`); `innermost_scope` is the **latest** scope on the highest layer (MI‑1).

**`hit.rs`** — hit ordering, layers, scroll routing
`last_registration_wins`, `higher_layer_shadows_lower` (strengthened: the lower layer is registered **last**), `a_lower_layer_region_registered_later_does_not_shadow_a_higher_one` (§25 F8: `hit()` selects `max_by_key(|r| (r.layer, index))`), `hit_returns_a_lower_layer_region_for_the_outside_click_test` (§21 item 12), `inert_below_registers_nothing` (runtime-level, §25 MI‑3), `hit_returns_part_ref_not_a_derived_id`, `hit_scroll_returns_the_innermost_handler_of_the_axis`, `hit_scroll_returns_a_region_at_zero_headroom`, `hit_scroll_skips_regions_that_do_not_handle_the_axis`, `duplicate_id_is_reported_as_a_diagnostic_not_a_panic`, `empty_rects_are_rejected`, `generation_bump_invalidates_stale_regions`.

**`capture.rs`** — drag capture
`capture_claims_and_rejects_a_second_claim`, `drag_and_release_go_to_the_capture_owner`, `local_is_computed_against_the_captured_area`, `pressed_stays_set_while_the_pointer_leaves`, `release_outside_the_captured_area_does_not_activate`, `capture_is_released_on_resize`, `capture_is_released_when_the_owner_disappears`, `capture_is_released_on_generation_mismatch`, `origin_is_the_press_position` (§25 F15: exercises `Cx::capture`, not a hand-built `Capture`).

**`scroll.rs`** — nested scrolling, boundary rule
`clamps_offset_to_content`, `ensure_visible_moves_minimally`, `thumb_covers_track_proportionally`, `track_position_round_trips`, `wheel_at_the_boundary_is_consumed_without_repaint`, `ensure_visible_on_next_layout_is_set_only_by_cursor_motion`, `fields_are_private_and_every_mutator_clamps`.

**`layer.rs`** — overlay stacking
`push_and_pop_maintain_layer_order`, `modal_pushes_a_trap_and_a_pointer_barrier`, `popover_pushes_a_pointer_barrier_only`, `esc_dismisses_only_the_top_layer`, `esc_reaches_the_focused_editor_before_the_layer` (§21 item 3), `layer_id_is_assigned_at_open_not_at_draw` (§21 item 14), `outside_click_is_layer_less_than_top_or_none`, `nested_layers_each_trap` (Scenario F), `anchor_rect_flips_then_clamps`, `anchor_screen_center_sits_in_the_upper_third`, <!-- amended by §26 --> `fill_resolves_to_the_whole_screen` (replaces the `(0,0)` arm), `fixed_size_is_clamped_never_grown` (renamed from `min_size_then_clamp_then_documented_degradation`: `Fixed(54,20)` on a 40×10 screen equals the screen; `Fixed(0, 8)` equals `Rect::ZERO`), `popover_flips_above_when_the_content_does_not_fit_below` (the `Anchor::Rect` flip, now reachable), `point_anchor_flips_instead_of_covering_the_pointer`, `resize_layer_re_resolves_the_anchor_on_the_next_draw` (open at `Fixed(20,4)`, `resize_layer` to `Fixed(40,10)` in `update`, the drawn area changes in the **same** frame), `spec_geometry_is_the_only_mutable_field` (compile-level: `Cx` exposes no other spec mutator), <!-- amended by §25 --> `composite_copies_only_painted_cells` (F3: a `RowUi` with a right-aligned `part()` inside a layer over a sentinel-filled page leaves unpainted cells intact), `closed_with_action_key_emits_layer_event_closed`, `dismissed_emits_the_reason`, `backdrop_excludes_the_footer_row`.

**`cursor.rs`**
`cursor_write_is_kept_for_the_focused_owner_on_the_top_layer`, `cursor_write_from_a_lower_layer_is_rejected`, `cursor_write_from_an_unfocused_owner_is_rejected`, `rejection_records_a_diagnostic`, `the_focused_owners_write_wins_on_the_same_layer` (§25 F6).

**`layout.rs` / `measure.rs`**
`rows_distributes_flex_after_fixed`, `columns_respects_gap_and_rounds_deterministically`, `responsive_columns_stack_below_the_threshold`, `action_row_right_aligns_and_left_aligns`, `inset_saturates_on_tiny_rects`, `split_first_pane_wins_on_both_axes_when_minima_do_not_fit`, `split_percent_is_clamped_to_5_95`, `measure_reports_min_and_preferred`, <!-- amended by §25 --> `auto_takes_one_cell_beside_flex_and_an_equal_share_without_it`, `rows_measured_uses_the_natural_size`, <!-- amended by §26 --> `measure::ui_resolve_equals_ui_style_for_every_family_variant_part` (differential over the built-in recipe set × both themes × a state sweep, with and without a pushed `Overlay`: `ui.resolve(..) == ui.style(..)` field-for-field — the test that keeps the second resolution path from drifting), `measure::measure_records_no_roles_and_no_styled_parts` (1 000 `Ui::resolve` calls leave `roles_at(pos)` untouched and `styled_parts()` empty), `measure::measure_does_not_touch_the_style_cache` (`StyleCache::stats()` identical before and after 1 000 measures), `measure::natural_width_follows_the_themed_glyph` (a theme that rebinds `GlyphRole::FocusBar` to a 2-cell glyph widens `Button::measure` by exactly one column), `measure::measure_is_allocation_free` (perf: 10 000 `Button::measure` calls record 0 allocations).

**`text/` (buffer, editor, measure, fuzzy)**
`insert_and_move_by_grapheme`, `selection_replaces_on_insert`, `word_motion_and_deletion`, `word_chars_are_consistent_between_buffer_and_viewport`, `multiline_vertical_motion_keeps_column`, `single_line_rejects_newline`, `wide_characters_count_as_two_columns`, `combining_marks_are_one_grapheme`, `zwj_emoji_is_one_grapheme`, `pos_of_and_offset_at_round_trip`, `fuzzy_returns_grapheme_indices_into_the_original_label`, `fuzzy_ranks_prefix_before_boundary_before_substring_before_subsequence`, `editor_apply_is_the_only_mutation_entry_point`, `zeroize_overwrites_before_drop` (§25 MA‑13: this name, not `zeroize_clears`; asserts capacity released and a fresh `expose()` empty), `row_ui_matches_fit_for_every_fixture` (differential against the `crates/tui/tests/fixtures/text.rs` corpus, §21 item 29; <!-- amended by §25 MA‑3 --> the reference is the legacy grapheme-walking `fit` verbatim, no non-ASCII `continue`, and cell symbols are compared **including** trailing padding), `width_matches_ratatui_cell_width` (§22: `text::width` equals `<str as ratatui_core::buffer::CellWidth>::cell_width` over a corpus including `"ｶﾞ"`, `"あ"`, `"a\u{FF9E}"`, `"\r\n"`, a ZWJ family emoji, a combining-mark cluster and `"\u{7}"` — the pin that keeps `wide_characters_count_as_two_columns` and `zwj_emoji_is_one_grapheme` honest).

**`theme/` (tokens, patch, recipe, resolve, downgrade)**
`slot_over_prefers_the_speaking_side`, `patch_merge_identity`, `patch_merge_absorption`, `patch_merge_is_associative`, `patch_clear_resolves_to_inherited_surface_fg`, `modifier_add_then_remove_is_symmetric`, `state_rules_are_stored_in_specificity_order` (R2 invariant), `state_rules_tie_break_by_declaration_order`, `state_rule_matches_only_when_when_is_a_subset`, `precedence_family_then_variant_then_state_then_global_then_scope_then_instance`, `roles_bind_after_the_whole_chain`, `raise_is_ladder_index_arithmetic_not_colour_equality`, `raise_saturates_at_the_last_level`, `field_raises_to_field_hover`, `downgrade_maps_every_token_exhaustively`, `downgrade_works_for_a_user_supplied_theme`, `mono_appends_one_state_rule_per_family` (<!-- amended by §28 --> §28 P6: `MONO_RULES_PER_FAMILY == 18`, was 16 — the three added `DISABLED` rules), <!-- amended by §28 --> `theme::mono_disabled_is_dim_and_readable` (§28 P6: under `ColorLevel::Mono`, for `Family::{INPUT, FIELD, LIST, BUTTON}` × `Part::{FIELD, TEXT, LABEL}` the resolved style contains `Modifier::DIM` and its `fg` differs from the resolved `bg` of `Surface::Canvas` — the black-on-black assertion), `paper_theme_inverts_the_plane_direction`, `custom_family_and_variant_round_trip`, `theme_is_byte_identical_after_a_scoped_render`, `builder_derives_every_unset_token_deterministically`, `derived_tokens_meet_design_contrast_ratios`, `downgrade_is_deterministic_per_level` (asserts `LightGreen`/`LightRed`/`Yellow`, §25 F5), `paper_tokens_are_pinned` (§21 item 29), <!-- amended by §25 --> `state_rules_beat_a_variant_base` and `family_and_variant_state_rules_interleave_by_specificity` (F1; and `precedence_family_then_variant_then_state_then_global_then_scope_then_instance`'s "3 over 2" arm uses a role whose bound colour differs from the variant's under **both** built-in themes, e.g. state `Role::Warning` vs variant base `Role::Accent`), `ansi16_preserves_hue_family_and_brightness` (F5: pins `DESIGN.md:320` — accent `LightGreen`, error `LightRed` — plus `danger_soft → LightRed`, `warning → Yellow`, `info → LightBlue`, and the grey ladder `surfaces[1]` `#111111`, BT.601 luma 17 → `Black`; `border_subtle` `#262626`, luma 38 → `DarkGray`; `fg[1]` `#b3b3b3`, luma 179 → `Gray`; `fg[0] → White`; plus the dark half `#2b8632 → Green` and `#7a2a2a → Red`. <!-- amended by §27 (Adjudication O3) --> ~~`border_subtle` → `Black`~~ is **struck**: it was an unverified *(estimate)* that no test ever carried. `#262626` has channel spread 0 and BT.601 luma 38, which lands in the `31..=110` `DarkGray` band; `#111111`, luma 17, is the value that reaches `Black`, and it is `surfaces[1]`. The legacy pin `theme::tests::accent_survives_downgrade` constrains only `accent`, `error` and `canvas`; it never constrained `border_subtle`. No baseline is re-blessed), `a_custom_family_resolves_through_the_neutral_recipe` (F14), <!-- amended by §26 --> `metrics_are_surface_independent` (`theme.metrics(..) == PartMetrics::from(theme.resolve(.., s))` for every `Surface`), `metrics_is_the_sizing_path_for_update` (`Form`'s field height and `Dialog::measured_height` computed from `Cx` equal what `draw` lays out against), `patch_merge_matches_ratatui_style_patch_for_modifiers` (§22: the final `inherited.patch(resolved.style)` step and §11.3's modifier-symmetry law agree with `ratatui_core::style::Style::patch`), <!-- amended by §24 --> `theme::ascii_border_set_is_pure_ascii` (each of `border::ASCII`'s eight fields satisfies `s.is_ascii() && s.len() == 1`, which also pins `text::width(s) == 1`), `theme::builtin_border_sets_are_ratatui_sets` (`Theme::junie().design.borders == border::ROUNDED`, `Theme::paper().design.borders == border::PLAIN`), `theme::ascii_theme_renders_without_box_drawing_glyphs` (<!-- amended by §27 (Adjudication O2) --> a whole-frame `Harness::text()` scan — **not** a `Scene` digest, which is what the earlier wording said — over `Theme::junie().builder().borders_set(border::ASCII).build()`, painting a frame **and** a `ui.rule(..)`, contains no char in `U+2500..=U+257F`; the scan is the box-drawing block only, deliberately — §24 M2 risk 3, and it is not vacuous because the same test asserts plain Junie *does* emit box drawing), <!-- amended by §27 (Adjudication O2) --> `theme::ascii_glyph_set_has_no_box_drawing` (component-free, so it is not hostage to which painters exist: over `Theme::junie().builder().borders_set(border::ASCII).build()`, every `GlyphRole::ALL` binding read through `GlyphSet::get` **plus every field** of the typed `scrollbar()`, `rule_quiet()` and `rule_active()` sets is outside `U+2500..=U+257F` — the assertion the render test can only approximate), `theme::builder::ascii_glyphs_is_idempotent_and_glyph_overrides_it` (`.ascii_glyphs().ascii_glyphs()` equals `.ascii_glyphs()`, and `.borders_set(border::ASCII).glyph(GlyphRole::RuleQuiet, "~")` yields `"~"`), <!-- amended by §27 (Adjudication O1) --> `theme::cache_generation_wrap_does_not_serve_a_stale_entry` (§20.9-2: seed a key at generation 1, force the stamp to `u32::MAX`, `clear()`, and the same key must **miss** — the wrap back to 1 would otherwise serve a stale `StylePatch`).

**`ui/paint.rs`** <!-- amended by §24; §25; §26 -->
`ui::paint_spans_matches_row_ui_label_spans` (differential: `Ui::paint_spans(area, spans, base)` and `RowUi::label_spans(spans)` over the `crates/tui/tests/fixtures/text.rs` corpus produce byte-identical cells and the same column count, §24 M1; **plus** the §25 F4 allocation assertion — painting 500 rows × 3 spans records 0 allocations), `ui::dim_layer_uses_the_role_of_the_painted_cell` (F3), `ui::with_part_resolves_once_and_records_the_role` (§26: one cache miss, roles set at the painted cells), `ui::surface_style_is_the_left_operand_of_the_final_patch` (§26: differential against a hand-written `inherited.patch(..)` over every surface × both themes).

**`collection/` (key, reconcile, rowui, decor, empty)**
`reconcile_keeps_a_surviving_key`, `reconcile_takes_the_nearest_forward_then_backward`, `reconcile_falls_back_to_the_first_enabled_key`, `reconcile_yields_cursor_lost_when_empty`, `reconcile_drops_vanished_checked_keys_and_reports_the_count`, `reconcile_clamps_the_scroll_offset`, `reconcile_runs_before_any_action_is_emitted`, `generation_stamp_skips_a_no_op_reconcile` (R1), `cached_index_probe_hits_before_a_scan` (R1), `row_ui_label_writes_cells_without_an_intermediate_string` (R5), `row_ui_meta_is_dropped_all_or_none`, `row_ui_columns_clip_to_the_row`, `empty_state_covers_empty_loading_partial_error`, `key_set_stays_sorted_after_insert_remove_toggle_retain`, `key_set_contains_is_binary_search` (§22: asserts the comparison count, not just the answer).

**`runtime.rs`** (§22)
`panic_hook_restores_before_delegating` — the chained hook installed by `TerminalSession` restores the terminal before delegating to the previous hook, mirroring the ordering of ratatui's `try_init` (`ratatui-0.30.2/src/init.rs:196-197`).

<!-- amended by §28 (Adjudication P3) --> `runtime::a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it` — the *gated* Roster shape (`if cx.is_open(CONFIRM) { … }` around `Dialog::update`), Esc on the open modal, exactly one `Diagnostic::UndeliveredIntent` whose owner is `CONFIRM`, even though the dialog registered only `Decorative` regions. `runtime::a_decorative_owner_is_not_diagnosed_for_a_pointer_intent` — the §21 item 13 exemption still holds for pointer intents: a click on a decorative container produces no bucket and no diagnostic.

**`crates/tui-testing/src/harness.rs`** <!-- amended by §25 F7 -->
`harness::resolved_reports_the_family_the_component_actually_queried` — `Harness::resolved(id, part)` returns the `Resolved` the component recorded through `Ui::style` for `(family, variant)`, never a hard-coded `Family::BUTTON`; `resolved_in(f, v, id, p)` is the explicit escape hatch.

**Component state machines** (`components/*.rs`, buffer-free, no terminal) — goal §25.1 "edit begin, commit, cancel, focus loss":
`input::begin_snapshots_the_value`, `input::commit_writes_the_controlled_value`, `input::commit_runs_validation_once`, `input::cancel_restores_the_snapshot`, `input::blur_commit_and_validate_policy`, `input::blur_cancel_policy`, `input::blur_keep_policy_leaves_the_draft`, `input::external_error_survives_a_redraw`, `input::write_mask_is_synthetic` (renamed, P5), `textarea::blur_commits_without_validation`, `select::escape_closes_and_restores_the_cursor`, `select::arrows_move_the_cursor_not_the_value_while_closed`, `choice::radio_group_separates_cursor_from_value`, `list::select_all_selects_only_enabled_items`, `list::range_selection_uses_the_anchor`, `tree::expand_collapse_is_keyed_not_positional`, `tree::lazy_children_do_not_reflatten_the_world`, `tabs::close_targets_the_logical_tab_after_a_reorder`, `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row`, `grid::edit_intent_inline_cycle_external_refuse` (reached only via `update_editable`, §23 K2), `grid::range_copy_is_tsv`, `grid::click_inside_an_active_inline_edit_goes_to_the_editor` (§21 item 30), `grid::read_only_update_takes_a_shared_model` (compile-fail via `trybuild`: `Grid::update` cannot reach `commit_cell`/`apply_cycle`, §23 K2), `grid::update_editable_commits_through_the_editor` (begin → type → Enter → `commit_cell` observed once; a failing `commit_cell` leaves the editor open with the returned `FieldError`), `grid::read_only_reason_is_rendered_from_a_grid_model` (a model implementing **only** `GridModel` renders its reason), `grid::cell_actions_affordance_is_painted_for_a_read_only_model` (the `→` glyph and its hot zone appear for a `GridModel`-only model; a click emits `GridAction::CellAction(item, col, key)`), `dialog::action_arming_is_evaluated_in_update`, `dialog::convenience_constructors_render_through_the_body_slot` (§21 item 33), <!-- amended by §26 --> `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` (same props + two themes ⇒ two deterministic sizes; same props twice ⇒ identical), `dialog::draw_lays_out_against_the_height_it_asked_for` (the rect `draw` returns equals the layer area; no `Rect::centered*` call in `dialog.rs`), `dialog::confirm_is_centred_by_the_resolver_not_by_the_dialog`, `dialog::a_growing_body_resizes_the_layer_on_the_next_frame`, <!-- amended by §28 --> `dialog::an_unconditional_update_receives_the_dismissal` (§28 P3: the unconditional shape yields `DialogAction::Dismissed(DismissReason::Esc)` and zero diagnostics), `dialog::a_prompt_dialog_sizes_its_own_field_row` (§28 P4: the drawn `Field` rect's height equals `d.size.field_height`, and `measured_height` minus the other terms equals `input_rows`), `dialog::a_forced_dialog_registers_no_control` (§28 P5: under `.state_override(..)`, `area_of(action_id(0))` and `area_of(id)` are both `None` and the ring is empty), `field::a_forced_field_registers_no_control` (§28 P5: the same for `Field` over a `TextInput`, through `FieldControl`'s `inherit_forced`; it fails today), `picker::query_change_emits_query_changed`, `wizard::rewind_retains_per_step_state`, `viewport::retention_fixes_up_selection_and_caret`, `code::edit_counter_invalidates_the_highlight_cache`, `secret::debug_and_display_redact`, `secret::is_not_clone_not_eq` (compile-fail via `trybuild`), <!-- amended by §24 --> `select::standalone_select_takes_items_per_phase` (`Select::new(id)` carries no items; `update`/`draw` receive `&[T]` and a `T` that is not `&str` with `.key(..)`/`.row(..)` compiles and reconciles — §24 M3).

**`components/form.rs`** (§15.1, §23 K1) — the form state machine, buffer-free:
`form::tab_order_follows_declaration_order_skipping_hidden`, `form::hidden_field_registers_no_ring_entry_and_keeps_its_draft`, `form::field_height_is_a_pure_function_of_spec_and_design_tokens`, `form::scroll_reveals_the_focused_field_from_update_not_draw`, `form::submit_commits_the_in_flight_edit_before_validating`, `form::submit_validates_every_visible_field_then_focuses_the_first_error`, `form::submit_skips_hidden_fields_during_validation`, `form::enter_submits_only_when_the_focused_control_is_not_editing`, `form::submit_chord_is_declared_on_the_action_not_baked_in`, `form::dirty_is_set_by_a_commit_not_by_a_keystroke`, `form::chooser_activation_emits_chose_with_the_field_id`, `form::note_rows_register_only_decorative_regions`, `form::at_most_one_action_per_frame_in_declaration_order`, `form::open_select_popover_traps_focus_and_esc_closes_only_the_popover`, `form::form_action_variants_carry_no_value`, `form::zeroize_overwrites_every_secret_draft`, `form::every_declared_field_resolves_a_value`, <!-- amended by §24 M3 --> `form::select_field_options_come_from_form_data` (the painted list is `FormData::options(id)`; a `FieldKind::Select` built with no items renders it), `form::changing_options_between_frames_does_not_rebuild_props` (the `&[FieldSpec]` array is byte-identical across two frames whose `options` differ), `form::state_holds_no_props` (static assertion: `FormState: Clone + PartialEq + Default`; `SlotValue: Clone + PartialEq + Eq`), `form::value_and_options_is_a_single_borrow` (a `Form::update`-shaped body over one `value_and_options` call compiles; the two-call form is a compile-fail via `trybuild`, E0502).

---

### 16.2 Shared conformance suite (goal §25.2)

<!-- amended by §21 items 10, 11, 15, 25, 27; §25 D‑8, MA‑8, MA‑9; §26 -->

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

// <!-- amended by §25 D‑8 --> the module ident is written explicitly (a macro cannot derive it from the `NAME` const);
// the macro emits, per entry, `#[test] fn name_matches_the_module() { assert_eq!(<$case as Conformance>::NAME, stringify!($name)); }`
// so the two cannot drift.
#[macro_export] macro_rules! conformance_suite { ($($name:ident => $case:ty),+ $(,)?) => { … } }
```

`crates/tui/tests/conformance.rs` ends with one invocation listing **every** public component; `architecture::conformance_covers_every_public_component` (§16.5) cross-checks that list against the `pub` component inventory, so adding a component without registering it fails CI.

```rust
conformance_suite!(   // <!-- amended by §25 D‑8: `name => Case` form -->
    button => ButtonCase, chip => ChipCase, checkbox => CheckboxCase, radio_group => RadioGroupCase,
    toggle => ToggleCase, brand => BrandCase, key_hint => KeyHintCase,
    field => FieldCase, text_input => TextInputCase, text_area => TextAreaCase, select => SelectCase,
    list => ListCase, nav_list => NavListCase, tree => TreeCase, props => PropsCase, props_list => PropsListCase,
    steps => StepsCase, grid => GridCase, tabs => TabsCase, chip_bar => ChipBarCase,
    panel => PanelCase, split_pane => SplitPaneCase, scroll_region => ScrollRegionCase,
    text_viewport => TextViewportCase, diff_view => DiffViewCase, code_editor => CodeEditorCase,
    dialog => DialogCase, menu_bar => MenuBarCase, context_menu => ContextMenuCase, picker => PickerCase,
    filter_list => FilterListCase, completion => CompletionCase,
    form => FormCase, wizard => WizardCase, picker_chain => PickerChainCase, help_overlay => HelpOverlayCase,
    progress_bar => ProgressBarCase, spinner => SpinnerCase, meter => MeterCase,
    status_bar => StatusBarCase, hint_bar => HintBarCase, too_small => TooSmallCase,
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
| 9 | `mono_states_are_distinguishable` | no-colour output retains state indicators | Under `ColorLevel::Mono`, the `(symbol, modifier)` multiset differs pairwise between default / focused / selected / pressed / disabled / error / warning / editing / busy / active, colour excluded; `pressed` is distinguishable through the `BOLD` + bracket-glyph rule of §11.4 (§21 item 25). <!-- amended by §25 MA‑8 --> The driver's **default** state list is the full ten; a component's `mono_states()` may only **narrow** it, and the driver asserts every state its `Caps` imply is present — a five-state default silently weakened the check. <!-- amended by §28 --> The driver **makes the forced state real**: a state whose affordance comes from props (`Status`, `checked`, `disabled`) is set on the props too, or the case proves nothing about it — forcing `StateFlags::BUSY` while `Fixture` carries no `status` left `Button::busy()` false and the spinner unpainted (§28 P6). `mono_states_required_by` is a **union** over `Caps`, not an `if/else if` chain: a case declaring `EDITS | DISABLEABLE` must keep `EDITING` **and** `DISABLED` |
| 10 | `local_override_does_not_mutate_the_theme` | local overrides do not mutate the theme | Hash the `Theme` before, render inside `ui.with_overlay(&OV, …)` and with `.patch_part(…)`, hash after: equal; the overridden part's `Resolved` differs while the un-overridden sibling's does not |
| 11 | `id_separator_collision_free` | (added) | Every id and `PartRef` this component registers is unique within a frame; no two differ only by concatenation (`Diagnostic::DuplicateId` count is 0) |
| 12 | `item_identity_survives_reorder` *(COLLECTION)* | (added, Scenario E) | Set cursor/selection/checked on keys `k₁,k₂`; apply a reverse permutation and an insert+remove; after `reconcile`, cursor and checked set still name `k₁,k₂`; a click on the row now showing `k₁` emits an action carrying `k₁`. <!-- amended by §25 MA‑9 --> The driver **sets** cursor/checked on `k₁,k₂` and asserts they survive `reconcile`; click identity alone is not the case |
| 13 | `focus_reconcile_follows_the_rule` *(FOCUSABLE)* | (added) | Remove the focused entry: focus lands on the nearest surviving entry by previous index; if the scope empties, on the scope's first enabled; then on the innermost active scope's first; then `None` — all four branches exercised |
| 14 | `focus_trap_and_restore` *(OVERLAY)* | (added) | Opening the layer shrinks `reachable()` to the layer's own stops; Tab wraps inside; closing restores focus to the opener; a layer that cannot draw (0×0) still traps |
| 15 | `pointer_capture_delivers_drag_and_release` *(CAPTURES)* | (added) | After a press claims capture, drags outside the component still reach it with `local` relative to the captured area; a second claim is refused; release outside the captured area does not activate |
| 16 | `wheel_at_boundary_is_consumed_without_repaint` *(SCROLLS)* | (added) | At offset 0 a wheel-up returns `Flow::Consumed` with `Invalidate::None`; mid-range returns `Consumed` + `Paint`; the event never chains to an outer scrollable and never moves focus or the cursor |
| 17 | `cursor_write_is_rejected_off_top_layer` *(CURSOR)* | (added) | Drawn under an open `Popover` (pointer barrier only, no `inert_below` — an inert layer registers nothing and is discarded silently, §21 item 15), the component's `ui.set_cursor` is dropped and one `Diagnostic::CursorRejected` is recorded; on the top layer with focus, it is kept |
| 18 | `secret_never_appears_in_debug` *(SECRET)* | (added) | `format!("{:?}")` of props, state, and any owning container (`Field`, `Dialog`, `Form`) contains neither `secret_bytes()` nor its snapshot; the rendered buffer contains neither; the `TestBackend` digest is unchanged when only the secret changes |
| 19 | `survives_tiny_rects_0x0_to_3x3` | (added) | For every `w,h ∈ 0..=3`: `draw` does not panic in a debug build, writes no cell outside the rect, registers no region outside it, and leaves no stale geometry (a click after a 0×0 frame resolves to `None`, never to last frame's rect). <!-- amended by §26 --> For `Caps::OVERLAY` the driver opens the layer through the component's **own** `LayerSpec` at each of the 16 screens and asserts `area ⊆ screen`, no registration outside, no panic |
| 20 | `bindings_match_handled_keys` | (added) | Every chord in `bindings(state)` is consumed by `update` in that state, and every chord `update` consumes in that state appears in `bindings(state)` — the table and the handler cannot drift. For components declaring `Caps::TYPES` the reverse direction exempts bare `Char` chords (§21 item 27) |

<!-- amended by §23 --> For `GridCase`, one registration selects `Grid::update` or `Grid::update_editable` from the existing `Fixture.read_only` knob, so every case — in particular 7 (`draw_does_not_commit_or_cancel`) and 12 (`item_identity_survives_reorder`) — runs through **both** entry points from a single registration (§23 K2). `FormCase` declares `Caps::EDITS | Caps::SECRET | Caps::FOCUSABLE | Caps::SCROLLS | Caps::TYPES` (§15.1).

Suite-level tests (emitted once, not per component), in `conformance.rs`:

* `conformance::registry::every_public_component_is_registered`
* `conformance::registry::declared_parts_are_the_parts_actually_styled` — the parts a component resolves at draw time equal `Self::PARTS`
* `conformance::conflicting_visible_bindings_are_reported` — two visible bindings on the same chord in the same phase produce a `Diagnostic::BindingConflict` (this is what makes the historically dead grid `Ctrl+D` detectable)
* `conformance::focus_transition_settles` — suite-level only (a whole-app property, §21 item 11): a scripted journey over every registered component records zero `Diagnostic::FocusTransitionDidNotSettle`
* `conformance::mono_states_required_by_is_a_union` <!-- amended by §28 --> — a synthetic `Caps::EDITS | Caps::DISABLEABLE` requires **both** `EDITING` and `DISABLED`; the `if/else if` chain it replaces returned one state and is what let `TextInputCase` drop `DISABLED` while declaring `Caps::DISABLEABLE` (§28 P6)
* `conformance::draw_registers_nothing_when_it_cannot_draw` — the 0×0 case across the whole registry; <!-- amended by §26 --> extended to `LayerSize::Fixed(0, h)` (a zero-size request is an empty layer, never the screen)

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

**Regeneration and review policy.** `BLESS=1 cargo test --workspace --test render --test render_components --test visual` <!-- amended by §28 --> rewrites baselines. The rule from goal §6.10 is enforced mechanically: `xtask bless-guard` fails a commit that changes a baseline file without a matching entry in `docs/visual-changes.md` referencing a §20.10 item and a capture path under `shots/`. No baseline is regenerated because a test failed; the classification comes first. The order is fixed (A14, §21 item 30): **change → capture → classify → bless** — the capture cannot exist before the change, so `bless-guard` runs in CI on the committed tree, not locally, and the `docs/visual-changes.md` entry is written between the capture and the bless. <!-- amended by §21 items 28, 30 -->

**The matrix.** <!-- amended by §28 --> The binding contract is the **test path**, not the file: `render::components::<component>::<state>`, `render::overrides::<case>`, `render::overlay::<case>`. ~~`crates/tui/tests/render.rs`~~ is not the contract. During Slices 3–4 these live in **two** targets, because two work packages own them: `crates/tui/tests/render.rs` (foundations — `render::overrides::*`, `render::overlay::*`, painted straight onto `Ui`) and `crates/tui/tests/render_components.rs` (components — `render::components::*`, one function per matrix cell over `{junie, paper} × {truecolor, mono} × {120×40, 40×10}`). Both are ordinary `cargo test` targets whose module paths are identical to the single-file form (`mod render { mod components { … } }`), so every name §16.3 and the slice acceptance conditions quote resolves unchanged. **Every gate command that runs the render tests must name both targets** (`--test render --test render_components`), or the command silently runs half the matrix. Merge into one target at Slice 5, when one owner holds both.


* Test `render::components::<component>::<state>` for every registered `Conformance` case × states `{default, focused, focus_visible, hovered, focus_plus_hover, pressed, selected, disabled, read_only, busy, loading, error, warning, editing, empty, overflow}` where meaningful (the driver derives which are meaningful from `Caps`).
* Themes: `Theme::junie()` and `Theme::paper()` — test names get a `_paper` suffix.
* Colour levels: `truecolor`, `ansi256`, `ansi16`, `mono` — suffix `_256` / `_16` / `_mono`.
* Sizes: `80x24`, `100x30`, `120x40`, `160x50`, plus `40x10` for the narrow/overflow states.
* Overrides: `render::overrides::global_family_override_changes_every_button`, `render::overrides::scoped_overlay_changes_only_the_subtree`, `render::overrides::instance_patch_changes_only_one_instance`, `render::overrides::part_slot_replaces_the_part_and_keeps_hit_regions`.
* Composition: `render::overlay::modal_over_page`, `render::overlay::nested_picker_over_dialog`, `render::overlay::popover_anchored_below_then_flipped`, `render::overlay::backdrop_excludes_the_footer`, `render::overlay::layer_composites_bottom_to_top_regardless_of_call_order`.

**Showcase digest contract** (from **[F]** APP §6): `apps/showcase/tests/visual.rs::showcase_visual_baseline` keeps its exact shape — for each page in `NAV_ENTRIES` × `{120×40, 80×24}`, focus the first control, hash every cell, write `"{w}x{h} {label} {hash:016x}"`, regenerate under an env var. Two changes, both recorded in §20.10: the sidebar rect is no longer excluded, and the matrix gains `× {junie, paper} × {truecolor, mono}` (four times the lines). `UPDATE_BASELINE=1` is preserved as an alias of `BLESS=1` so the documented workflow (`README.md:325-328`) still works.

---

### 16.4 Application integration tests (goal §25.4)

<!-- amended by §25 F7: `Harness::resolved` / `Runtime::resolved` return the recorded resolution, never a hard-coded `Family::BUTTON`; `resolved_in` is the explicit escape hatch -->

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
    pub fn resolved(&self, id: Id, p: Part) -> Resolved;                // the Resolved the component RECORDED for (family, variant); never a hard-coded family (§25 F7) <!-- amended by §25 -->
    pub fn resolved_in(&self, f: Family, v: Variant, id: Id, p: Part) -> Resolved;   // explicit escape hatch (§25 F7)
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

**Theme coupling in tests** (`focus_bar_x` compares against `Theme::junie().focus`, **[F]** APP §6) becomes `h.resolved(id, Part::GUTTER).style.fg`, so the assertion survives a theme change and also runs under `Theme::paper()`. <!-- amended by §25 F7 --> This contract holds only because `resolved` returns the **recorded** resolution: under `#[cfg(feature = "testing")]` `FrameState::styled_parts` is `Vec<(Id, Family, Variant, Part, Resolved)>`, written by `RowUi::style_of` and by `Ui::note_styled` at each component's own query, and `Runtime::resolved(id, part)` returns that record (falling back to `resolved_in` only when nothing was recorded). A `Family::BUTTON` hard-coded in `Runtime::resolved` would return a colour a `List`, `Tabs` or `Field` never painted, and every migrated assertion in Slices 5–7 would be silently wrong (`harness::resolved_reports_the_family_the_component_actually_queried`).

**New application coverage** required by goal §25.4:

`showcase::complete_navigation_visits_every_page_and_every_state`, `showcase::custom_theme_injection_repaints_every_page` (`--theme paper`), `showcase::local_override_page_shows_three_distinct_buttons`, `showcase::author_component_page_participates_in_focus_and_hover`, `tablepro::mouse_flow_full_journey` and `tablepro::keyboard_flow_full_journey` (retained, renamed from `acceptance_flow_*`), `tablepro::grid_adapter_keeps_every_pending_change_capability`, `jackin::complete_flow_keyboard_first` (retained), `jackin::nested_overlay_picker_inside_dialog`, `tablepro::connection_form_keyboard_and_mouse_reach_every_field`, `tablepro::connection_form_focuses_the_first_invalid_field`, `tablepro::connection_password_is_masked_and_absent_from_the_frame`, `jackin::form_dialog_toggles_visibility_and_keeps_drafts`, `jackin::form_dialog_secret_never_reaches_the_screen_as_a_string` (§15.1, §23 K1), `tablepro::view_grid_is_read_only_with_a_reason`, `tablepro::result_grid_sorts_locally_and_refuses_edits` (DOM §1.6 capabilities 3 and 4, §23 K2), `*::resize_across_every_supported_size`, `*::focus_is_restored_after_every_overlay_closes`, `*::no_diagnostics_are_emitted_during_the_journey` (asserts zero `DuplicateId`, `CursorRejected`, `UndeliveredIntent`, `BindingConflict`, `FocusTransitionDidNotSettle`, `UnaddressableId`, `DuplicateLayerDraw` in a full run — `UndeliveredIntent` is zero because container regions are `Decorative`, §21 item 13 — a strong, cheap regression net).

---

### 16.5 Architecture checks (goal §25.5)

`crates/tui/tests/architecture.rs` plus an `xtask` binary for the checks that need to read the workspace. Preference order per goal §25.5: **Cargo/visibility first, `cargo tree` second, text allow-lists last.**

| Test name | Mechanism | Asserts |
|---|---|---|
| `architecture::library_has_no_application_dependency` <!-- amended by §22 --> | `cargo metadata` from `xtask`: the dependency closure of `junie-tui` | `showcase`, `tablepro`, `jackin-preview` are absent; the only direct normal deps are `ratatui-core`, `ratatui-crossterm`, `unicode-width`, `unicode-segmentation`, `bitflags` (never `ratatui`, `ratatui-widgets`, `ratatui-macros`, `smallvec`, a direct `crossterm`) |
| `architecture::applications_depend_only_on_the_library_facade` <!-- amended by §22 --> | `cargo tree -p <app> -e normal --depth 1` — a one-line assertion, because each app's only normal dependency is `junie-tui` and `junie-tui` re-exports every ratatui type the public API mentions (§22 §1.2); plus a source scan for `junie_tui::` paths | `cargo tree` prints `junie-tui` and nothing else; every path resolves under `junie_tui::` or `junie_tui::author::`; there is no `#[path]`, no `include!`, and no `pub(crate)` reachable from an app (guaranteed structurally — a separate crate cannot name a `pub(crate)` item, so the scan is a belt-and-braces report, not the enforcement) |
| `architecture::examples_are_external_consumers` | `cargo build -p junie-tui --examples` in CI plus a check that no example uses `#[path]` | The 13 §17 examples compile against the public API only <!-- amended by §23 --> |
| `architecture::all_examples_compile` | `cargo test --workspace --doc` + `--examples` gate | goal §25.5 "all examples compile" <!-- amended by §25 F12: absent from `xtask`'s `CHECKS` at `18afddd` together with `every_named_test_exists`, `conformance_covers_every_public_component` and `state_override_is_used_only_in_apps_and_fixtures`; all four are binding correction obligations --> |
| `architecture::public_items_are_documented` | `#![deny(missing_docs)]` in `crates/tui/src/lib.rs` + `RUSTDOCFLAGS="-D warnings" cargo doc` | Every public item has rustdoc |
| `architecture::no_unsafe` | `#![forbid(unsafe_code)]` in the library; `crates/tui-testing` carries the single documented `unsafe impl GlobalAlloc` with a safety comment | goal §26 |
| `architecture::no_domain_vocabulary_in_the_library` | grep allow-list over `crates/tui/src/**` for `(?i)\b(sql|schema|primary key|nullable|foreign|references|not null|tablepro|jackin|workspace|instance|daemon|capsule|construct|catalog)\b`, with an allow-list file `crates/tui/tests/allow/domain.txt` (currently empty) <!-- amended by §25 §4(j): scans **code lines only**, deliberately — `\bworkspace\b` and `\binstance\b` appear in ordinary architectural prose ("per-instance patch") and a reflowed `///` line must not fire it --> | DOM §7 acceptance conditions 1 and 2 |
| `architecture::palette_literals_are_confined_to_theme_builtins` <!-- amended by §22 --> | grep for `Color::Rgb(` / `Color::from_u32(` / `#[0-9a-fA-F]{6}` over `crates/tui/src/**`, allow-listed to `theme/builtin/junie.rs`, `theme/builtin/paper.rs`, and `tests/fixtures/*.rs`, plus <!-- amended by §25 D‑10 --> the **path** allow-list entries `theme/downgrade.rs` and `theme/builder.rs` (computed `Color::Rgb(r, g, b)` from the downgrade and L\* derivation arithmetic; a path allow does not feed the "`legacy_api.txt` must be empty" condition, and the regex is never narrowed to hide it) (the `from_u32` arm is necessary: `Color::from_u32` is the one sanctioned literal constructor, §22 R‑10, and without it every literal would move one call deeper) | goal §25.5 |
| `architecture::no_raw_background_parameter` | grep for `bg:\s*(ratatui_core::style::)?Color` in any `pub fn` signature under `crates/tui/src` <!-- amended by §22: rule 21 of `no_deprecated_or_legacy_api_usage` --> | The 24 `bg: Color` sites (**[F]** API §3.6) are gone; `Role::Custom(Color)` inside a `StylePatch` is the one allowed raw colour and is allow-listed by name |
| `architecture::no_owns_or_locate_in_applications` | grep for `\.owns\(`, `\.locate`, `scrollbar::id_for`, `\.child\(` over `apps/**/src` | Target 0; the allow-list file `apps/allow/dispatch.txt` must be empty and any entry requires a justification comment that the test prints |
| `architecture::no_generic_component_copies_in_applications` | grep for `fn render(` + `Style::new()` + `Block::default()` + `buf.set_string` over `apps/**/src`, allow-listed to `apps/jackin-preview/src/rain.rs` | goal §25.5 "application directories do not contain generic component copies"; rain is the single documented exception (goal §22.3) |
| `architecture::no_public_geometry_or_cache` | grep for `pub area`, `pub areas`, `pub anchor`, `pub .*_rects`, `pub scroll` under `crates/tui/src` | Invariant S1; kills the 21 `pub area` + 3 `pub areas` sites (**[F]** API §3.8) |
| `architecture::no_fn_pointer_extension_points` | grep for `: fn\(`, `Option<fn(`, `type \w+ = fn\(` under `crates/tui/src` | The 6 sites in **[F]** API §3.12 are gone (DOM §7 condition 6) |
| `architecture::draw_takes_shared_self` | `syn`-based scan in `xtask`: every `fn draw` in `crates/tui/src/components/**` has `&self` and, if it takes a state parameter, `&XState` | G2 — the structural form of "render cannot change semantics" |
| `architecture::no_static_bound_in_component_surface` | `syn` scan for `'static` bounds on public component types and their builder parameters, allow-listed to `Binding<A: 'static>` and `Conformance: 'static` | goal §2.2 |
| `architecture::conformance_covers_every_public_component` | `syn` scan of `pub struct`s in `components/**` vs the `conformance_suite!` list | G10 / §16.2 |
| `architecture::every_named_test_exists` <!-- amended by §25 F12: must exist in `xtask`'s `CHECKS` and `tests/architecture.rs`; it was absent at `18afddd` and is the gate that makes §25's renamed/missing names visible --> | one-directional and scoped (§21 item 28): every name listed in §16.1, §16.2's suite-level list and §16.4 exists in `cargo test --workspace -- --list`; §16.6 perf names are checked against `cargo test --workspace --test perf --release -- --list`; `trybuild` cases against `tests/ui/*.rs` filenames; extra tests are allowed | Documentation and the suite cannot drift; the `capsule_pane_clone_4x2000` deletion is asserted by line-absence in `perf_baseline.txt` |
| `architecture::binary_names_are_preserved` | `cargo metadata` target names | `showcase`, `tablepro`, `jackin-preview` (goal §21) |
| `architecture::msrv_and_edition_are_unchanged` <!-- amended by §22 --> | `cargo metadata` **plus** the blocking CI job `msrv`: `cargo +1.88.0 check --workspace --all-targets --all-features` — the metadata proves the *field*, the job proves the code compiles on 1.88 (on a 1.98 toolchain a builder could otherwise use a 1.95 API and every gate would pass) | edition 2024, `rust-version = "1.88"` on every package, held deliberately (§22 §5) |
| `architecture::cache_types_are_derived_only` <!-- §21 item 2 --> | `syn` scan in `xtask`: every `T` used as `ui.cache::<T>(…)` | `T` appears in no `Response` and no `XState` (R8) |
| `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library` <!-- §21 item 23 --> | `cargo metadata` | `showcase_app`, `tablepro_app`, `jackin_app` have `publish = false` and are absent from the library's dependency closure |
| `architecture::props_are_built_once` <!-- §21 item 30 --> | `syn` scan over `apps/**/src` and `crates/tui/examples/**` | no configured `X::new(` for the same `const Id` appears more than once per screen module (§13) |
| `architecture::state_override_is_used_only_in_apps_and_fixtures` <!-- §21 item 30 --> | grep for `.state_override(` | only under `apps/**`, `crates/tui/tests/fixtures/**` and `crates/tui-testing/**` |
| `architecture::every_component_doc_has_the_standard_sections` <!-- §21 item 33 --> | rustdoc-json heading scan | every public component's docs carry the 15 §13.2 headings in order |
| `architecture::no_todo_or_unimplemented` <!-- §21 item 33 --> | grep for `todo!`, `unimplemented!`, `TODO`, `FIXME` over `crates/**` and `apps/**`, empty allow-list | goal §29 "no material TODO, stub, placeholder" |
| `architecture::showcase_covers_every_public_component` <!-- §21 item 33 --> | cross-check the `conformance_suite!` list against the showcase page registry | goal §29 "the showcase demonstrates every public component" |
| `architecture::no_deprecated_or_legacy_api_usage` <!-- §22; amended by §25: rules 27 and 27a added; the scan covers whole files — `non_test_lines` skips only the `#[cfg(test)]`-attributed item, never the file tail (F9, MA‑2) --> | `xtask` scan of `crates/tui/src/**`, `crates/tui-testing/src/**`, `apps/**/src/**` against the forbidden-pattern table of §22 §6.2 (26 rows + 27, 27a); allow-list `crates/tui/tests/allow/legacy_api.txt`, every entry with a same-line justification the test prints on failure **and** on success | the allow-list is **empty**; no deprecated `Buffer::get`, raw `\x1b[`, `Stylize`, `Masked`, `SmallVec`, umbrella-crate / `ratatui_widgets::` / `ratatui_macros::` paths, `KeyboardEnhancementFlags`, `LazyLock`/`OnceLock`/`static mut`, `#[allow(`, nested `for y … for x` over a rect, `Style::default()`, or off-theme `.fg(`/`.bg(` |
| `architecture::dependency_graph_is_exactly_the_declared_set` <!-- §22 --> | `xtask`, `cargo metadata` | (1) `junie-tui`'s direct normal deps are exactly `{ratatui-core, ratatui-crossterm, unicode-width, unicode-segmentation, bitflags}`; (2) <!-- amended by §25 (adjudication 5) --> split into 2a–2d as §22.7 records: `ratatui`/`ratatui-widgets`/`ratatui-macros` absent from the **entire** closure; `critical-section`/`palette` absent from the **entire** closure; `smallvec`, `parking_lot*`, `lock_api`, `scopeguard`, `libc`, `mio`, `signal-hook*` reachable **only beneath `ratatui-crossterm`** (inverted-tree assertion); no direct `smallvec` and no direct `crossterm`; (3) each app's direct normal deps are exactly `{junie-tui}`; (4) `unicode-width`, `unicode-segmentation`, `bitflags` each resolve to **one** version (a second `unicode-width` would mean two disagreeing width tables — R‑1 rests on this); (5) `ratatui-core`'s enabled features are exactly `{std, underline-color}` |
| `architecture::every_foreign_type_in_the_public_surface_is_re_exported` <!-- §24 M1; amended by §25 MA‑11: at `18afddd` this is a substring grep over `lib.rs`, recorded as a **deviation** with a Slice‑8 upgrade to the rustdoc-json form below — the grep cannot detect a `pub` signature naming an unexported foreign type --> | `xtask`, rustdoc-json: for every non-local type named in a `pub` item reachable from `junie_tui::`, a `pub use` path exists under `junie_tui::`; likewise for `junie_tui::author::` | the facade is *complete*, not merely the only edge: the day a `ratatui_core::text::Line` enters a signature the check fails and prints the type, the signature that names it and the missing facade line — instead of a downstream `ratatui-core` dependency line appearing |
| `architecture::capability_has_no_unicode_field` <!-- §24 M2 --> | rustdoc-json | `Capability`'s field set is exactly `{color}`; no automatic ASCII-border selection can exist (§21 item 19, §24 M2) |
| `architecture::field_kind_has_no_type_parameters` <!-- §24 M3 --> | rustdoc-json | `FieldKind` has one lifetime and zero type parameters; `Form`, `FormState` and every screen's `&[FieldSpec]` stay non-generic (§13) |
| `architecture::no_boolean_capability_parameter_on_grid` <!-- §23 K2 --> | grep: `! rg -n 'fn editable\(' crates/tui/src/components/grid.rs` and `! rg -n 'trait GridCellActions' crates/tui/src` | G4: capability is chosen by the entry point (`update` / `update_editable`), never by a flag; `GridCellActions` is deleted |
| CI gate `core_is_backend_free` <!-- §22; amended by §25 (adjudication 1) --> | `cargo check -p junie-tui --no-default-features` | proves that nothing outside `runtime/session.rs` needs a backend: the `crossterm` feature gates the terminal session only, and `ratatui-crossterm` (a normal, non-optional dependency taken for its version-unified `crossterm::event` vocabulary) still compiles. The **stronger** claim — that the widget layer is backend-independent — is proved by forbidden-pattern rule 27 (`CrosstermBackend` only in `runtime/session.rs`) and by `architecture::ratatui_crossterm_is_named_in_exactly_two_files` |
| `architecture::ratatui_crossterm_is_named_in_exactly_two_files` <!-- §25 F20 --> | `xtask` boundary check | `ratatui_crossterm` is named in exactly `src/event.rs` (the `crossterm::event` vocabulary — `KeyCode`, `KeyModifiers`, `Input::from_crossterm`) and `src/runtime/session.rs` (the backend); nowhere else |
| `architecture::no_unreachable_spin_loops` <!-- §25 F2 --> | `xtask` rule 27a: `loop {` with `spin_loop` forbidden in `crates/tui/src/**` | no `unreachable_*` helper hangs the process with raw mode on — a livelock with the alternate screen entered is strictly worse than a panic, whose hook restores the terminal; `Vec::insert(i, _)` makes `get_mut(i)` infallible and is written as one documented `#[expect(clippy::expect_used, reason = …)]` |
| CI gate `readme_compiles` <!-- §22 --> | `cargo test --workspace --doc` with `#![doc = include_str!("../README.md")]` at the top of `crates/tui/src/lib.rs` | every README code fence is valid Rust or tagged ` ```text `/` ```ignore ` (goal §24, §25.5) |
| `xtask semver` <!-- §22 §3.4: DEFERRED --> | `cargo semver-checks --baseline-rev <tag>` | **Not a shipped gate during the refactor**: during a total public-API rewrite every check fails by construction. Added at the end of Slice 8 against tag `v0.1.0`; blocking in CI from `v0.1.1` onward |
| `xtask doc-check` <!-- §21 item 34; amended by §23, §24, §25 F23 --> | extracts every `Ident::method` reference and every Rust code block from `COMPONENT_ARCHITECTURE.md` §3–§17 and §21–§26 and resolves each against the library's rustdoc-json. At `18afddd` the range stopped at §23 and the resolver is a heuristic whose `foreign_members()` allow-lists legacy names (`Theme::row`, `Interaction::pressed`) as if they were foreign API — recorded as a **deviation** (MA‑14): the range covers §24–§26 now, the legacy names move into an explicit `doc_check_allow.txt`, and the rustdoc-json resolver is a Slice‑8 upgrade | every reference resolves, or is on the printed "not yet built (Slice 3/4)" allow-list; run in every slice gate |

---

### 16.6 Performance (goal §25.6)

**The measurement plan of `docs/audit/performance-audit.md` §7 is adopted verbatim** — harness, assertion policy, baseline file format, CI wiring, and every test name. Nothing in it is renamed. Restated obligations:

* Harness in `crates/tui-testing/src/perf.rs` (`Counting` global allocator shim, `ALLOCS`/`BYTES`, `bench`, `Stats`, `report`); `#[global_allocator]` declared **only** in `crates/tui/tests/perf.rs` and `apps/*/tests/perf.rs`. WP‑0 landed the harness at root `tests/perf_common.rs` + `tests/perf.rs` (commit `07cb2c9`); Slice 3 moves it (Appendix A, §21 item 31).
* Allocation and byte counts are deterministic → **hard assertions**. Wall time is reported always, asserted only under `PERF_STRICT=1` against `baseline × 1.2`. <!-- amended by §25 MI‑14 --> There are exactly two knobs, `PERF_STRICT` and `PERF_BLESS`; the undeclared `PERF_TARGET` is **folded into `PERF_STRICT`** — every wall-clock ratio it gated is asserted under `PERF_STRICT=1` and reported otherwise.
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
| `style_resolve_10k_parts` <!-- amended by §25 (adjudication 8) --> | `Theme::row`+`gutter`, 0 allocs, ≈1.1 ns/query | **exactly 0 allocations** (R2, hard, deterministic); **cache hit rate ≥ 90 %** over the 10 k-part frame (`StyleCache::stats`, promoted to `#[cfg(feature = "testing")]`, `Runtime::style_cache_stats()`) — the memo of §11.1 A3 is the mechanism and a broken key shows up here and nowhere else; ns **recorded** in `perf_baseline.txt` and asserted only under `PERF_STRICT=1` against that baseline × 1.2. <!-- amended by §27 (Adjudication O1, O4a) --> Two amendments. (a) The ≥ 90 % figure is a **key-correctness floor**, not a performance bound and not a guarantee: a broken key drops it to ≈ 0.3 %, the struck one-way memo shape to ≈ 87 %, and the shipped two-way memo measures ≈ 99.7 % (§20.9-2 for the geometry). The deterministic guarantee is `theme::cache_hits_after_the_first_query_and_clears_by_generation`, which is hash-independent. (b) Added, and now the **binding** style-cost bound: an absolute per-query budget under `PERF_STRICT=1` — `ns / 10 000 × 2 000 ≤ 32 000`, i.e. **≤ 16.0 ns per query**, machine-independent, one-sided, currently 12.0 ns. That is §25.8's "≈ 13 ns × ~2 000 queries per frame ≈ 26 µs" sentence turned into code, and it replaces the frame-share bound deferred to Slice 5 |
| `style_resolve_per_frame` *(added, §25)* | — | <!-- amended by §27 (Adjudication O4a) --> ~~the style-resolution share of `frame_showcase_lists_120x40` is **≤ 5 %** of that frame's total ns, asserted under `PERF_STRICT=1`~~ is **deferred to Slice 5**: `frame_showcase_lists_120x40` lives in `apps/showcase/tests/perf.rs`, which Slice 5 owns, and does not exist yet. Until then the test measures a 40-row × 5-part frame twice — styles resolved per row versus hoisted — and extrapolates the **difference**: arm A performs 200 `ui.style` calls, arm B 40, so the delta covers **160** queries and the multiplier is **× 12.5**, not × 10 — `resolution_ns × 2 000 / 160 ≤ 32 000` — asserted under `PERF_STRICT=1` as a second, looser net beside the per-query budget above. The in-situ share is **reported, not asserted**: the stand-in is the style-densest frame constructible from foundations (five resolutions per painted row, no chrome, no borders, no status bar), so its share is an upper bound on the real one, and asserting an upper bound against a threshold written for the real frame either fails spuriously or passes vacuously. The test **carries no baseline line, deliberately** — a baselined `ns` for a differential invites a meaningless × 1.2 regression check — so `perf_baseline.txt` names it in the `#` header instead, which is how it satisfies the mark-additions rule below. Slice 5 reinstates the ≤ 5 % share as `style_resolve_share_of_frame_showcase_lists_120x40` against the real frame, at which point this row's extrapolation drops from *asserted* to *reported*. This replaces the struck "ns ≤ 2× the pre-refactor `Theme::row`+`gutter` baseline", which compared a 30-field `Copy` read against a six-level precedence resolution and was unmeetable by construction; the measured ≈12× per-query cost (≈13 ns) is recorded and accepted — ≈26 µs per realistic 2 000-query frame, under 0.2 % of a 16 ms budget |
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
| `fit_10k_grapheme_line_to_80` <!-- amended by §25 (adjudication 4) --> | exactly 3 | the `RowUi` equivalent (driving `RowUi::label`, the ellipsis path it is named for) records **0** over a corpus whose graphemes fit ratatui `Cell`'s inline `CompactString` symbol storage (ASCII + CJK + combining marks; no ZWJ sequences) — `≤ 8` was **rejected** as a magic constant hiding the invariant (R5) |
| `fit_10k_grapheme_line_to_80_wide` *(added, §25)* | — | the ZWJ-emoji corpus, **reported**; allocations are ratatui `Cell` heap symbols — a property of the buffer, not the painter — **bounded by the columns painted and independent of line length**: a 10 k and a 100 k line into the same 80 columns record equal counts, `≤ 80` |
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
| `intents_drain_is_o_1_when_the_queue_is_empty` (renamed, §21 item 6) <!-- amended by §25 (adjudication 6) --> | `crates/tui/tests/perf.rs::invariants` | a 500-component frame with 0 intents performs **0 bucket probes** and **0 allocations**, and costs the same as a 20-component frame with 0 intents — that zero is *structural*, not statistical: `IntentQueue::iter` returns on `used == 0` before `bucket_index`, the only site that bumps the counter. <!-- amended by §27 (Adjudication O4b) --> With the queue non-empty, exactly **one probe per `cx.intents` call**, stated in **differential** form because `probes()` is cumulative since construction and also counts the enqueue path, so no absolute count is stable: a 500-component frame performs exactly **480 more** probes than a 20-component frame in the same single update pass, and allocations are 0. The 480 encodes "one update pass" — `Runtime::handle`'s focus re-run loop is bounded at four passes (§3.3 step 7), and a legitimate second pass makes the delta 960, which is a real behaviour change worth failing on, so the assertion stays an **equality** and is not relaxed to a multiple. ~~with 2 intents, probes are exactly one per drain call (500)~~ and ~~total probe cost is ≤ 500 × 5 ns~~ are **struck**: the absolute 500 is unattainable, and 2.5 µs against a measured 632 ns for the whole 500-control `handle` is not a bound, it is a tautology. Probes are counted deterministically (`IntentQueue::probes()` / `Runtime::intent_probes()` under `testing`, a `Cell<usize>` bumped in `bucket_index`). The **raw** wall-clock ratio is reported and never asserted — it measures 14.9× because the stub application's own `for i in 0..n` update loop is O(n) by construction, which is the workload, not a defect. What is asserted under `PERF_STRICT=1`, with a 1.25× band, is the **normalised** per-control ratio `intents_drain_ns_per_control = (ns₅₀₀ × 20)/(ns₂₀ × 500)`, which measures **0.60** and would read ≈ 25 if per-drain cost ever became O(n) (R6, B14) |
| `frame_hintbar_derived` (§21 item 30, P1) | `crates/tui/tests/perf.rs::frames` | **0 allocs/frame** when focus is unchanged; the derived `HintLayer` is cached in `Ui::cache` behind `(focus_id, StateFlags, top_layer)` |
| `frame_tablepro_connection_form_120x40` (§15.1, §23 K1) | `apps/tablepro/tests/perf.rs::frames` | **< 40 allocs/frame** for the 15-field `Form` inside its dialog |
| `paint_spans` allocation assertion (§25 F4, inside `ui::paint_spans_matches_row_ui_label_spans`) | `crates/tui/tests/perf.rs` | painting 500 rows × 3 spans through `Ui::paint_spans` / `RowUi::label_spans` records **0** allocations — the `Vec<RawSpan>` per call that made `frame_showcase_lists_120x40 < 20`, `grid_500x12_render < 100` and `viewport_100k_lines_render` unreachable is gone |
| `measure_is_allocation_free` (§26 N2) | `crates/tui/tests/perf.rs` | 10 000 `Button::measure` calls record **0** allocations; the companion ns line (~13 ns per uncached part) is reported, asserted only under `PERF_STRICT=1` |

<!-- amended by §22 --> Removing `SmallVec` from `PartRecipe`, `Recipe`, `KeySet` and `HintLayer` (§22 §4.2) changes `Theme`/`Recipes` sizes and therefore shifts the perf baseline; the §16.6 pre-refactor baseline is taken on the *unmodified* tree (WP‑0) and the `Vec` decision is recorded as one of the "after" explanations. The `KeySet` sorted-`Vec` representation is what keeps `list_100k_rows_render` inside its 1.5× bound with a 5 000-row selection (O(40 · log 5000) instead of O(5000) per visible row).

**CI wiring** (perf §7.3, adopted verbatim): one always-on job `cargo test --workspace --test perf --release` (allocation counts only) and one pinned-runner job `PERF_STRICT=1 cargo test --workspace --test perf --release -- --test-threads=1`. `PERF` lines are collected into a build artefact for the final report (goal §30 item 13).

---

## 17. Representative usage examples

Thirteen examples <!-- amended by §23: example 13 -->, one file each under `crates/tui/examples/`, built by `cargo build -p junie-tui --examples` (`-p tui-next` during Slices 3–4) and gated by `architecture::all_examples_compile`. Every file is complete — a `main` or a `#[test]`, every `use` list exact — because Slice 2 acceptance condition 1 compiles them verbatim. They use only the public facade, so they are literal proof of the "external consumer" claim. Examples 1–10 are also condensed into rustdoc doctests on the corresponding types (`cargo test --workspace --doc`).

### 17.0 API additions

<!-- amended by §21 items 1, 2, 4–10, 13, 17–22, 24, 30, 32; §22; §23; §24 -->

Everything §17 needs that §1–§15 did not spell out. These are additions to the accepted architecture; each is consistent with an existing rule and is listed here so no example invents a name. This block is the surface the examples compile against and the surface `xtask doc-check` (§21 item 34) resolves the document against.

```rust
// ---- A1. Application entry point (§3.3 named `Runtime<A>` and `app.update`/`app.draw`) ----
pub trait App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;
    fn draw(&self, ui: &mut Ui<'_>);
    fn should_quit(&self) -> bool { false }
    fn keymap(&self) -> &KeyMap { KeyMap::EMPTY_REF }               // §21 item 9
    fn min_size(&self) -> Size { Size { min: (72, 20), preferred: (120, 40) } }
    /// <!-- amended by §25 D‑7 --> §3.3 step 8(c)'s application hook. The spec put the Esc ladder on
    /// `Screen` and left `App` without one, so an app with no screen stack had no place to answer Esc.
    /// Runs after the focused component (step 7) and after app Bubble bindings, before the top layer's
    /// `Dismiss.esc` (§21 item 3).
    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> { Response::ignored() }
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
/// <!-- amended by §22 --> Behind the default-on `crossterm` feature. `TerminalSession` is a faithful mirror of
/// ratatui's `try_init`/`try_restore` (umbrella-crate-only and therefore unavailable to us, §22 §2.7): the chained
/// panic hook is installed BEFORE the first mode change; raw mode, alternate screen, mouse capture, bracketed paste
/// and line-wrap are typed `crossterm::{terminal, event}` commands in one `execute!` (never a raw `\x1b[` string,
/// R‑18); restore is one reverse-order `execute!` and `leave()` is idempotent. `terminal.draw(|f| …)` stays because
/// the render closure is infallible; `try_draw` is the documented alternative if a fallible `App::draw` ever exists.
pub fn run<A: App>(app: A, theme: Theme) -> std::io::Result<()>;
#[cfg(feature = "crossterm")]
pub type DefaultTerminal = ratatui_core::terminal::Terminal<ratatui_crossterm::CrosstermBackend<std::io::Stdout>>;   // our own alias (§22 §2.7)
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
    /// <!-- amended by §25 D‑7 --> The channel by which a matched `KeyMap` chord reaches `App::update`.
    /// §3.3 step 2 said "produces an app action" and declared no channel; this is it. `None` when the
    /// frame's input matched no binding.
    pub fn command(&self) -> Option<ActionKey>;
    // <!-- amended by §26 (Adjudication N1) --> geometry is the ONE part of an open spec that may change
    // while open; `kind`, `inert_below`, `restore_focus` and `initial_focus` are armed at open (§9.1).
    pub fn resize_layer(&mut self, id: Id, size: LayerSize);          // no-op when not open or unchanged; safe every frame
    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor);         // a popover whose owner moved
    pub fn quit(&mut self);
    #[cfg(feature = "testing")]
    pub fn record(&mut self, tag: &'static str);                      // replaces `Harness::actions` (§21 item 19)
}
pub struct IntentIter<'f> { /* … */ }
impl<'f> Iterator for IntentIter<'f> { type Item = Intent<'f>; /* … */ }

impl Ui<'_> {
    pub fn full(&self) -> Rect;                                       // current clip rect
    pub fn surface(&self) -> Surface;  pub fn bg(&self) -> Color;     // §10
    /// <!-- amended by §25 D‑13; §26 (Adjudication N2) --> The PAINTING queries, and the only ones that
    /// record: `&mut self` is required by the §11.1 A3 memo and by the per-cell role recording `dim_layer`
    /// reads. `style_patched` adds precedence 6 (the per-instance patch).
    pub fn style(&mut self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved;
    pub fn style_patched(&mut self, f: Family, v: Variant, p: Part, s: StateFlags, patch: &StylePatch) -> Resolved;
    /// <!-- amended by §26 (N2) --> The `&self` path: the full §11.3 chain WITHOUT the memo and WITHOUT
    /// recording roles or styled parts — for `Measure::measure` and any read that must not paint. Same
    /// family/variant/state chain, same live overlay stack, same current surface; it differs only in what
    /// it does not do. Excludes precedence 6 (merge it with `StylePatch::merge` if the caller has one).
    /// ~13 ns, 0 allocations. A measurement must not evict a painting entry from the 256-slot memo.
    pub fn resolve(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> Resolved;
    /// <!-- amended by §26 (N2) --> The glyph a role currently maps to (`design.glyphs`). `&self`, so it is
    /// reachable from `measure`; pair with `text::width` for its cell width.
    pub fn glyph_str(&self, g: GlyphRole) -> &'static str;
    /// <!-- amended by §26 --> The style a child inherits from the current surface: `bg = theme.bg(ui.surface())`,
    /// `fg = Role::Fg(FgStep::Primary)` bound on that surface, no modifiers. The LEFT operand of §11.3's final
    /// layering; the fused call site is `ui.fill(area, r.over(ui.surface_style()))` (§11.3, §22.2 item 10).
    pub fn surface_style(&self) -> Style;
    /// <!-- amended by §26 --> Resolve `part` once and paint with it — `style` plus the memo lookup and the
    /// role recording, expressible as one statement instead of a bound temporary. Binds a VALUE only: it
    /// pushes no clip and no surface (use `with_area` / `with_surface` for those). A convenience, not a
    /// replacement: `style_patched` has no `with_` form, so a component with a per-instance patch keeps the
    /// two-step shape.
    pub fn with_part<R>(&mut self, f: Family, v: Variant, p: Part, s: StateFlags,
                        g: impl FnOnce(&mut Ui<'_>, Resolved) -> R) -> R;
    pub fn with_area<R>(&mut self, area: Rect, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn with_surface<R>(&mut self, s: Surface, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn with_overlay<R>(&mut self, ov: &Overlay, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn focus_scope<R>(&mut self, id: Id, mode: ScopeMode, f: impl FnOnce(&mut Ui<'_>) -> R) -> R;
    pub fn register_control(&mut self, id: Id, area: Rect, f: Focusability);          // RegionKind::Control
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect);            // RegionKind::Part
    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect);           // RegionKind::Decorative (§21 item 13)
    pub fn register_scroll(&mut self, id: Id, area: Rect, axes: Axes, head: Headroom); // RegionKind::Scroll
    /// <!-- amended by §25 D‑6 --> Declares that `id` owns a text editor this frame, so paste and IME text
    /// route to it (§3.3 step 4).
    pub fn register_editor(&mut self, id: Id, area: Rect);
    /// <!-- amended by §25 D‑6 --> Declares next-frame state flags for `id`. A non-semantic draw-phase write
    /// the runtime reads on the NEXT frame (declared flags live in last frame's list), the same one-frame
    /// contract as `report_layout` and `cx.area` (S3). Consequence, recorded deliberately: `focused_is_editing`
    /// reads last frame's flags, so a paste in the same `handle` that began an edit is not routed.
    pub fn declare_state(&mut self, id: Id, s: StateFlags);
    pub fn report_layout(&mut self, id: Id, l: LayoutFacts);
    pub fn set_cursor(&mut self, owner: Id, pos: Position);
    pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>;
    pub fn dim_layer(&mut self, area: Rect, steps: u8);                    // deliberate re-implementation of ratatui_widgets::Dimmed: walks FrameOut::roles (role arithmetic, §11.6), excludes the footer row (§22)
    // painting (R3); every method clips to the current area and marks the layer's written-cell bitset
    // <!-- amended by §22 --> exact ratatui primitives behind each method (MOD §2.1, §2.2, §2.18; R‑2, R‑4, R‑6):
    pub fn paint_cell(&mut self, pos: Position, symbol: &str, s: Style);    // MUST reset the cells shadowed by a wide grapheme, as set_stringn does (buffer.rs:359-368), or the diff corrupts
    pub fn paint_str(&mut self, area: Rect, text: &str, s: Style) -> u16;   // = buf.set_stringn(area.x, area.y, text, area.width as usize, s): clips, skips zero-width/control, returns columns written; NEVER pre-truncates
    pub fn paint_style(&mut self, area: Rect, s: Style);                    // = buf.set_style(clip.intersection(area), s): restyle without touching symbols
    pub fn fill(&mut self, area: Rect, s: Style);                           // per-position set_symbol(" ").set_style(s) over area.positions() — a deliberate re-implementation of ratatui_widgets::Fill (it cannot mark the written-cell bitset)
    pub fn rule(&mut self, area: Rect);                                     // GlyphRole::RuleQuiet across `area`
    pub fn frame(&mut self, area: Rect, s: Style) -> Rect;                  // theme BorderSet; returns the inner rect
    pub fn glyph(&mut self, area: Rect, g: GlyphRole, s: Style) -> u16;     // columns written
    /// <!-- amended by §24 M1; §25 D‑13, F4 --> Multi-style single-line paint in OUR vocabulary. Resolves each
    /// `Span`'s `Role` against the live theme and surface and writes it through `Buffer::set_span`, span by span,
    /// accumulating the x cursor and the per-span role marks — **never** collecting a `Vec<RawSpan>` first (the
    /// per-call allocation on the row path that made `frame_showcase_lists_120x40 < 20`, `grid_500x12_render < 100`
    /// and `viewport_100k_lines_render` unreachable). `base` is the part style the spans inherit; without it
    /// `RowUi::label_spans` could not honour the `LABEL` recipe. Returns columns written. Removes the only
    /// realistic reason to reach `author::raw::Span`. `Buffer::set_span` is a sanctioned per-span writer
    /// alongside `set_line` (§22 R‑3), so width accounting cannot drift from `set_stringn`'s.
    pub fn paint_spans(&mut self, area: Rect, spans: &[Span<'_>], base: Style) -> u16;
    pub fn raw(&mut self) -> (&mut Buffer, Rect);
    /// Derived, non-semantic per-component cache. Keyed by (Id, TypeId). Cleared on resize,
    /// theme change and generation gap. Never observable in `Response`, never compared by
    /// `draw_twice_leaves_state_equal`. Rule R8 (§5). (§21 item 2, B5)
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T;
    #[cfg(feature = "testing")]
    /// <!-- amended by §25 F7 --> Widened to carry the resolution key, so `Runtime::resolved(id, part)` can
    /// return the `Resolved` the component actually recorded instead of a hard-coded `Family::BUTTON`.
    /// Written by the painting queries only (invariant M1, §26): `Ui::resolve`, `Theme::resolve` and
    /// `Theme::metrics` record nothing, or a part a component merely MEASURED would be reported as styled
    /// and `declared_parts_are_the_parts_actually_styled` would pass on a false positive.
    pub fn styled_parts(&self) -> &[(Id, Family, Variant, Part, Resolved)];
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
//   fn update         <M: GridModel  + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &M)     -> Response<GridAction>   // §23 K2
//   fn update_editable<M: GridEditor + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>
//   fn draw           <M: GridModel  + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect
//   fn update<D: FormData + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut FormState, data: &mut D) -> Response<FormAction>   // §15.1, §23 K1
//   fn draw  <D: FormData + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &FormState, data: &D) -> Rect

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
// <!-- amended by §25 D‑12 --> The builder's parameters are named POSITIONALLY here (`selection(bg, fg)`,
// `highlight(bg, fg)`, `field(base, hover)`, `disabled(fg, bg)`); the implementation's own spellings are
// compatible and no rename is required. `focus`, `selection`, `highlight`, `field` and `disabled` are all
// present (§21 item 21) and `build()`'s derivation fills every token the caller did not set
// (`theme::builder_derives_every_unset_token_deterministically`, `theme::derived_tokens_meet_design_contrast_ratios`
// — `architecture::every_named_test_exists` settles their existence mechanically, §25 F12).
// The named border sets, including OUR `border::ASCII` (§24 M2), are declared below; nothing about them changes.
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
    pub fn borders_set(self, b: BorderSet) -> Self;   // <!-- amended by §27 (O2) --> `border::ASCII` also applies `ascii_glyphs()`
    /// <!-- amended by §27 (Adjudication O2) --> Rebind every glyph whose default falls in the
    /// box-drawing block (`U+2500..=U+257F`) to its ASCII equivalent — the quiet rule, the active
    /// rule and the scrollbar track, thumb and caps — by replacing the whole typed `line` and
    /// `scrollbar` sets (`glyph::ASCII_RULE_QUIET`, `ASCII_RULE_ACTIVE`, `ASCII_SCROLLBAR`).
    /// Idempotent; call `.glyph(..)` **after** it to override any of them (§24.2, §27.2).
    pub fn ascii_glyphs(self) -> Self;
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
// <!-- amended by §22; §24 M2 --> `BorderSet` is a type alias of `ratatui_core::symbols::border::Set<'static>` (§11.2), so it carries no
// inherent consts; the named sets are ratatui's, re-exported for applications (which depend on `junie-tui` alone), plus ONE
// const of ours. In crates/tui/src/theme/border.rs — reachable as `junie_tui::theme::border` and `junie_tui::author::border`:
pub mod border {
    pub use ratatui_core::symbols::border::{Set, DOUBLE, PLAIN, ROUNDED};          // `ThemeBuilder::borders_set(border::PLAIN)`
    /// Pure-ASCII border set, for terminals and fonts without box-drawing glyphs. Not shipped by ratatui;
    /// declared here as a plain `const` because `BorderSet` is a type alias of a foreign type and can carry no
    /// inherent items (§11.2). Opt in with `Theme::junie().builder().borders_set(border::ASCII).build()`.
    /// Borders only: every other `GlyphSet` glyph stays as the theme set it (§24 M2, deferred root cause).
    pub const ASCII: Set<'static> = Set {
        top_left:         "+", top_right:         "+",
        bottom_left:      "+", bottom_right:      "+",
        vertical_left:    "|", vertical_right:    "|",
        horizontal_top:   "-", horizontal_bottom: "-",
    };
}
// `Theme::junie()` → `design.borders = border::ROUNDED`; `Theme::paper()` → `design.borders = border::PLAIN` (§11.7).

// ---- A6. Layer construction (§9.1 gave the struct; §21 items 8, 20; §26 Adjudication N1) ----
// <!-- amended by §26 --> `LayerSize { Fill, Fixed(u16, u16) }` is declared in §9.1. It replaces
// `min_size: (u16, u16)`, whose `(0, 0)` sentinel resolved to the whole screen and whose name was false:
// the resolver clamps down and never grows, so the field was always a maximum. `Fixed(0, _)` / `Fixed(_, 0)`
// now resolves to `Rect::ZERO` — a zero-size request is an empty layer, which is what §16.2 case 19 and
// `conformance::draw_registers_nothing_when_it_cannot_draw` assume.
#[non_exhaustive]
pub struct LayerSpec { /* kind, owner, anchor, dismiss, restore_focus, initial_focus, size: LayerSize, backdrop, inert_below */ }
impl LayerSpec {
    // <!-- amended by §26 --> all three default to `size: LayerSize::Fill`: `Fill` is the honest primitive
    // default (a full-screen modal is a real case) and `modal()` is `const`, so it cannot read
    // `design.size.dialog_width`. The COMPONENT supplies the size — `Dialog`, `Select`, `Picker`,
    // `ContextMenu` and `MenuBar` never open a bare spec (§9.2).
    pub const fn modal(owner: Id) -> LayerSpec;                     // Modal, Screen(Center), esc+outside, dim, inert
    pub const fn popover(owner: Id, anchor: Anchor) -> LayerSpec;   // Popover, pointer barrier only, no dim
    pub const fn tooltip(owner: Id, at: Position) -> LayerSpec;
    pub const fn anchor(self, a: Anchor) -> Self;
    pub const fn dismiss(self, d: Dismiss) -> Self;
    pub const fn backdrop(self, b: Backdrop) -> Self;
    pub const fn initial_focus(self, id: Id) -> Self;
    pub const fn size(self, s: LayerSize) -> Self;                  // <!-- amended by §26 --> was `min_size(w, h)`; `LayerSize` is declared in §9.1
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
    /// <!-- amended by §26 (Adjudication N1) --> The size this overlay asks its layer for, from the items it
    /// receives per phase (A3): `popup_min_width ≤ w ≤ popup_max_width` over the labels, `h = min(items,
    /// popup_max_rows) + 2`. Pure in `(props, items, DesignTokens)`; re-asserted every `update` with
    /// `Cx::resize_layer` (invariant D1, §9.2). `Select`, `ContextMenu` and `MenuBar` carry the same method.
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize;
}
// <!-- amended by §24 M3 --> The three choice controls are ordinary collections under the same three-block shape;
// `FieldKind` (A10) holds their `<'a, &'a str, ByIndex, DefaultRow>` instantiation and the form passes the option
// list as the per-phase items (`FormData::options` / `value_and_options`). No constructor takes items.
impl<'a, T> Select<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }    // items are per phase (A3); const-constructible
impl<'a, T, K, R> Select<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::FIELD, Part::LABEL, Part::MARKER, Part::ROW,
                                         Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Select<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Select<'a, T, K, R2>;
    pub fn placeholder(self, s: &'a str) -> Self;
    pub fn popup_rows(self, n: u16) -> Self;
    pub fn read_only(self, yes: bool) -> Self;   pub fn disabled(self, yes: bool) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> Select<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SelectState, items: &[T]) -> Response<SelectAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
    pub fn measured_size(&self, cx: &Cx<'_>, items: &[T]) -> LayerSize;   // the popup layer's size (§26 N1)
}
impl<'a, T> RadioGroup<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }
impl<'a, T, K, R> RadioGroup<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::GUTTER, Part::MARKER, Part::LABEL];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> RadioGroup<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> RadioGroup<'a, T, K, R2>;
    pub fn read_only(self, yes: bool) -> Self;   pub fn disabled(self, yes: bool) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> RadioGroup<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut RadioGroupState, items: &[T]) -> Response<RadioGroupAction>;   // cursor ≠ value (§15)
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &RadioGroupState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl<'a, T> ChipBar<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }
impl<'a, T, K, R> ChipBar<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::LABEL, Part::CLOSE, Part::OVERFLOW];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> ChipBar<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> ChipBar<'a, T, K, R2>;   // Part::LABEL pre-styled per chip
    pub fn select_mode(self, m: SelectMode) -> Self;   pub fn closable(self, yes: bool) -> Self;
    pub fn read_only(self, yes: bool) -> Self;         pub fn disabled(self, yes: bool) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> ChipBar<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut ChipBarState, items: &[T]) -> Response<ChipBarAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &ChipBarState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
impl<'a> Grid<'a> {                                                   // no `M` on the props (§21 item 1, B15)
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::HEADER, Part::ROW, Part::CELL,
                                         Part::TRACK, Part::THUMB, Part::OVERFLOW, Part::EMPTY, Part::ACTIONS];
    pub fn new(id: Id, columns: &'a [Column<'a>]) -> Self;
    pub fn nav(self, u: NavUnit) -> Self;        pub fn select_mode(self, m: SelectMode) -> Self;
    pub fn empty(self, e: EmptyState<'a>) -> Self;
    // <!-- amended by §23 K2 --> no `editable(bool)`: capability is chosen by the entry point (G4)
    pub fn actions_slot(self, f: &'a dyn Fn(&mut Ui<'_>, Rect)) -> Self;
    pub fn update<M: GridModel + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &M) -> Response<GridAction>;               // read-only (G1)
    pub fn update_editable<M: GridEditor + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;  // + inline edit lifecycle (G2)
    pub fn draw<M: GridModel + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
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
    // <!-- amended by §26 (Adjudication N1) --> `Dialog` sizes its own layer. Pure in `(props, DesignTokens)`,
    // computed identically in both phases — the rule §15.1 F4 already imposes on field height.
    /// Rows the body slot needs. The dialog never sees the body closure before `draw`, so the caller states
    /// it; the convenience constructors set it (`confirm` and friends pass 0).
    pub const fn body_rows(self, n: u16) -> Self;
    /// `.width(w)` when set, else `design.size.dialog_width` (54).
    pub fn measured_width(&self, d: &DesignTokens) -> u16;
    /// border(2) + title(1) + `text::wrapped_rows(description, inner)` + `input_rows` + [blank + body] + [blank + actions].
    /// `input_rows` is `0` / `field_height` / `field_height + 1` for none / `prompt` / `acknowledge` (§26.1, §28 P4). <!-- amended by §28 -->
    /// `draw` lays out against this number (`dialog::draw_lays_out_against_the_height_it_asked_for`).
    pub fn measured_height(&self, d: &DesignTokens) -> u16;
    /// The layer this dialog wants — `LayerSpec::modal(id).size(LayerSize::Fixed(w, h))`. Call it at the
    /// moment of opening: `cx.open_layer(CONFIRM, confirm_dialog().layer(cx))`, where `confirm_dialog()` is
    /// the single props constructor §13 already requires, so this also satisfies `architecture::props_are_built_once`.
    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec;
    /// **Invariant D1.** `update` begins by re-asserting the size —
    /// `cx.resize_layer(self.id, LayerSize::Fixed(self.measured_width(cx.design()), self.measured_height(cx.design())))`
    /// — so a growing description, an appearing error row or a theme swap corrects the layer on the next draw
    /// without the opener predicting anything (`dialog::a_growing_body_resizes_the_layer_on_the_next_frame`).
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
    /// <!-- amended by §25 (adjudication 7) --> Declared because `Track::Auto` without a measurement is
    /// one cell: example 9 feeds `preferred.1` to `layout::rows_measured`.
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub struct ScopeKey(u16);
pub struct DialogState  { /* action cursor, prompt draft */ }
pub struct PickerState  { /* query editor core, cursor, scroll, active scope */ }
pub struct ListState    { /* cursor key, checked KeySet, scroll, gen stamp */ }
pub struct TabsState    { /* active key, cursor key, strip window, gen stamp */ }
pub struct GridState    { /* cursor cell, range anchor, row selection KeySet, two-axis scroll, edit lifecycle, gen stamp */ }
pub struct TextInputState { /* draft, editor core, phase, error */ }
pub struct SelectState     { /* cursor key, value key, open flag, popup scroll, gen stamp */ }          // §24 M3
pub struct RadioGroupState { /* cursor key, gen stamp — the VALUE is controlled by the caller (§15) */ }
pub struct ChipBarState    { /* cursor key, checked KeySet, strip window, gen stamp */ }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum SelectAction { Chose(ItemKey), Opened, Closed }   // §6.1 <!-- amended by §24 -->
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum RadioGroupAction { Chose(ItemKey) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum ChipBarAction { Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey) }

// ---- A8. Status, capability and small value types (§21 items 13, 19, 20, 21, 22) ----
/// Data readiness of a component; the runtime maps it onto `StateFlags::{BUSY, LOADING, ERROR}`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)] #[non_exhaustive]                                 // §22 (MOD §3.2)
pub enum Status { #[default] Ready, Busy, Loading, Error }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] #[non_exhaustive] pub enum ColorLevel { TrueColor, Ansi256, Ansi16, Mono }
pub struct Capability { pub color: ColorLevel }                      // `UnicodeLevel` deleted (M6)
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Density { Comfortable, Compact }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum SortDir { Asc, Desc }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Align { Left, Center, Right }               // text (StylePatch.align, CellUi::align)
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum ScreenAlign { Center, UpperThird, Bottom }  // Anchor::Screen
/// The state a binding table is selected for. `Copy`, so a table is chosen by `match` in a `const fn`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct BindingState { pub flags: StateFlags }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum KeyPhase { Capture, Bubble }             // was `Phase2`
/// Selection set with an inverted representation so "select all" never materialises every key (§20.9-13).
/// <!-- amended by §22 --> `Vec<ItemKey>` KEPT SORTED (never SmallVec, MOD §4.2): `contains` is a binary search, so a
/// 5 000-row selection over a 100k grid costs O(log n) per visible row instead of O(n). `AllExcept(Vec::new())` is 0 allocs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeySet { Only(Vec<ItemKey>), AllExcept(Vec<ItemKey>) }
impl KeySet {
    pub fn contains(&self, k: ItemKey) -> bool;        // binary search (`key_set_contains_is_binary_search`)
    pub fn insert(&mut self, k: ItemKey);   pub fn remove(&mut self, k: ItemKey);   pub fn toggle(&mut self, k: ItemKey);
    pub fn all(&mut self);                  pub fn none(&mut self);
    pub fn len_in(&self, total: usize) -> usize;
    pub fn retain(&mut self, keep: impl Fn(ItemKey) -> bool) -> usize;   // returns the dropped count (reconcile)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)] #[non_exhaustive]                                          // §22 (MOD §3.2)
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

// <!-- amended by §26 (Adjudication N1) --> Text primitives this document names (`crates/tui/src/text/`,
// Slice 3 owns them). `wrapped_rows` is the SINGLE wrap `Dialog::measured_height` and `Dialog::draw` share;
// without it "a pure function of props and design tokens" is unverifiable and the two can drift (risk 2).
pub mod text {
    pub fn width(s: &str) -> u16;                    // = `<str as ratatui_core::buffer::CellWidth>::cell_width` (§22 §2.3, R‑1)
    pub fn wrapped_rows(s: &str, width: u16) -> u16; // grapheme/word walk, 0 allocations
}

// ---- A9. Diagnostics (§7.1, §8.4, §3.3, §16.4; §21 items 11, 13, 17, 30) ----
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]                      // §22 (MOD §3.2): apps must not rely on exhaustive matching; jackin's intent loop already uses `_ => {}`
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

// ---- A10. Form (§15.1, §23 K1) — the surface example 13 compiles against; semantics in §15.1 ----
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum FieldSpan { Full, Half }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct GroupKey(u16);
impl GroupKey { pub const ALL: GroupKey; pub const fn custom(name: &'static str) -> GroupKey; }
// <!-- amended by §24 M3 --> closed, non-generic; the choice variants hold the default collection instantiation
pub type LabelSelect<'a> = Select<'a, &'a str, ByIndex, DefaultRow>;
pub type LabelRadio<'a>  = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;
pub type LabelChips<'a>  = ChipBar<'a, &'a str, ByIndex, DefaultRow>;
pub enum FieldKind<'a> { Text(TextInput<'a>), Area(TextArea<'a>), Select(LabelSelect<'a>), Radio(LabelRadio<'a>),
                         Chips(LabelChips<'a>), Check(Checkbox<'a>), Toggle(Toggle<'a>), Chooser(Button<'a>), Note }
pub struct FieldSpec<'a> { pub id: Id, pub label: &'a str, pub kind: FieldKind<'a>, pub required: bool,
                           pub help: Option<&'a str>, pub span: FieldSpan, pub group: GroupKey, pub plain: bool }
impl<'a> FieldSpec<'a> {
    pub const fn new(id: Id, label: &'a str, kind: FieldKind<'a>) -> Self;
    pub const fn required(self, yes: bool) -> Self;   pub const fn help(self, s: &'a str) -> Self;
    pub const fn span(self, s: FieldSpan) -> Self;    pub const fn group(self, g: GroupKey) -> Self;
    pub const fn plain(self, yes: bool) -> Self;
}
pub enum FieldMut<'d> { Text(&'d mut String), Secret(&'d mut Secret), Choice(&'d mut usize), Flag(&'d mut bool),
                        Chips(&'d mut KeySet), ReadOnly }
pub enum FieldRef<'d> { Text(&'d str), Secret(&'d Secret), Choice(usize), Flag(bool), Chips(&'d KeySet),
                        Display { value: &'d str, detail: Option<&'d str> }, Note(&'d [(&'d str, Role)]) }
pub trait FormData {
    fn value(&self, id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;
    fn options(&self, _id: Id) -> &[&str] { &[] }                                                   // §24 M3: the option list is data
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) { (self.value_mut(id), &[]) }  // ONE borrow for `update`
    fn visible(&self, _id: Id) -> bool { true }
    fn disabled(&self, _id: Id) -> bool { false }
    fn error(&self, _id: Id) -> Option<&str> { None }
    fn validate(&self, _id: Id, _v: FieldRef<'_>) -> Result<(), FieldError> { Ok(()) }
    fn validate_all(&self) -> Result<(), (Id, FieldError)> { Ok(()) }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum EnterPolicy { SubmitsWhenIdle, Never }
pub struct Form<'a> { /* id, fields, actions, submit, cancel, enter, columns, group */ }
impl<'a> Form<'a> {
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::BODY, Part::ACTIONS, Part::HELP, Part::MARKER, Part::TRACK, Part::THUMB];
    pub fn new(id: Id, fields: &'a [FieldSpec<'a>]) -> Self;
    pub fn actions(self, a: &'a [Action<'a>]) -> Self;   pub fn submit(self, k: ActionKey) -> Self;
    pub fn cancel(self, k: ActionKey) -> Self;           pub fn enter(self, p: EnterPolicy) -> Self;
    pub fn columns(self, n: u8) -> Self;                 pub fn group(self, g: GroupKey) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn update<D: FormData + ?Sized>(&self, cx: &mut Cx<'_>, st: &mut FormState, data: &mut D) -> Response<FormAction>;
    pub fn draw<D: FormData + ?Sized>(&self, ui: &mut Ui<'_>, area: Rect, st: &FormState, data: &D) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormAction { Changed(Id), Committed(Id), Chose(Id), Action(ActionKey), Invalid(Id) }   // carries no value (F5)
pub struct FormState { /* slots: Vec<FieldSlot> keyed by Id, scroll: ScrollState, errors: Vec<(Id, FieldError)>, dirty, gen stamp */ }
impl FormState {
    pub fn is_dirty(&self) -> bool;   pub fn mark_clean(&mut self);
    pub fn error(&self, id: Id) -> Option<&FieldError>;   pub fn set_error(&mut self, id: Id, e: Option<FieldError>);
    pub fn clear_errors(&mut self);   pub fn reveal(&mut self, id: Id);   pub fn zeroize(&mut self);
}
impl Default for FormState { /* … */ }   impl Reconcile for FormState { /* … */ }   impl fmt::Debug for FormState { /* redacting */ }
// Control constructors example 13 uses inside `FieldKind`. <!-- amended by §24 M3 --> `Select::new(id)`, `RadioGroup::new(id)`
// and `ChipBar::new(id)` are the A7 collection constructors — no items, no `&[&str]`; `T = &'a str` is inferred from the
// `FieldKind` variant and the option list reaches the control through `FormData::options` / `value_and_options`:
impl<'a> TextArea<'a>   { pub fn new(id: Id, rows: u16) -> Self; }
impl<'a> Checkbox<'a>   { pub fn new(id: Id, label: &'a str) -> Self; }
impl<'a> Toggle<'a>     { pub fn new(id: Id, label: &'a str) -> Self; }
impl Default for SecretPolicy { /* the library's default mask glyph and synthetic-tail length (§15) */ }
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

**2 — A complete custom theme** (`examples/02_custom_theme.rs`) <!-- amended by §22; §24 M2 -->

```rust
use junie_tui::theme::{border, ColorTokens, Density, MeterTokens, SyntaxTokens, Theme};   // <!-- amended by §22: no umbrella-crate path; `Color::from_u32` is the one literal constructor (R‑10) -->
use junie_tui::Color;

fn slate() -> Theme {
    Theme::from_tokens(ColorTokens {
        surfaces: [Color::from_u32(0x0A0C10), Color::from_u32(0x10131A), Color::from_u32(0x171B24), Color::from_u32(0x1F2430), Color::from_u32(0x282E3D)],
        field: Color::from_u32(0x10131A), field_hover: Color::from_u32(0x171B24),
        fg: [Color::from_u32(0xE8ECF4), Color::from_u32(0xBAC1D0), Color::from_u32(0x8C94A6), Color::from_u32(0x61697C), Color::from_u32(0x404758)],
        on_accent: Color::from_u32(0x080A0E), on_danger: Color::from_u32(0xFFF5F5), on_surface_inverse: Color::from_u32(0x0A0C10),
        border_subtle: Color::from_u32(0x1F2430), border_strong: Color::from_u32(0x485062),
        accent: Color::from_u32(0x7AA2F7), accent_hover: Color::from_u32(0x93B4FA), accent_pressed: Color::from_u32(0x608AE8),
        accent_tint: Color::from_u32(0x162036),
        focus: Color::from_u32(0x7AA2F7), focus_ring: Color::from_u32(0x608AE8),
        selection_bg: Color::from_u32(0x1F2C48), selection_fg: Color::from_u32(0xE8ECF4),
        highlight_bg: Color::from_u32(0x263454), highlight_fg: Color::from_u32(0xE8ECF4),
        highlight_danger_bg: Color::from_u32(0x542026), highlight_danger_fg: Color::from_u32(0xFFEBEB),
        backdrop_fg: Color::from_u32(0x404758), backdrop_bg: Color::from_u32(0x080A0E),
        danger: Color::from_u32(0xF06E78), danger_soft: Color::from_u32(0x602A32), danger_tint: Color::from_u32(0x30161C),
        warning: Color::from_u32(0xE0A850), warning_tint: Color::from_u32(0x382A14),
        success: Color::from_u32(0x7EC88C), info: Color::from_u32(0x78B4DC),
        disabled_fg: Color::from_u32(0x4A5264), disabled_bg: Color::from_u32(0x10131A), read_only_fg: Color::from_u32(0x8C94A6),
        syntax: SyntaxTokens::derive(Color::from_u32(0x7AA2F7), Color::from_u32(0x7EC88C), Color::from_u32(0xE0A850)),
        meter:  MeterTokens::derive(Color::from_u32(0x7EC88C), Color::from_u32(0xE0A850), Color::from_u32(0xF06E78)),
    })
    .builder()
    .borders_set(border::PLAIN)          // the square set is ratatui's PLAIN; `BorderSet` is its type alias (§11.2)
    .density(Density::Compact)
    .build()
}

/// <!-- amended by §24 M2 --> The same theme for a terminal without box-drawing glyphs: `border::ASCII` is a plain
/// const beside ratatui's sets, chosen by the theme author, never by capability detection.
fn slate_ascii() -> Theme { slate().builder().borders_set(border::ASCII).build() }

fn main() { let _ = slate(); let _ = slate_ascii(); }
// `Theme::from_tokens` fills design tokens and recipe defaults; `downgrade` works for it
// exactly as for `junie()`, because `map_colors` is an exhaustive destructure (§11.4).
// `ColorTokens` is deliberately NOT `#[non_exhaustive]` (§21 item 8): a new token is an
// intentional breaking change for downstream themes, and this literal is the proof.
```

**3 — Partial theme override** (`examples/03_partial_theme.rs`)

```rust
use junie_tui::{Color, Theme};   // <!-- amended by §22: `Color::from_u32`, never an umbrella-crate path -->

fn main() {
    // Change three roles; everything else is inherited from Junie, unchanged, byte-for-byte.
    let t = Theme::junie()
        .builder()
        .accent(Color::from_u32(0xC67A2E))   // amber instead of green
        .focus(Color::from_u32(0xC67A2E))    // `ThemeBuilder::focus` — §21 item 21
        .danger(Color::from_u32(0xB02525))
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
    let cols = layout::action_row(area, &[10, 12], ui.design().space.gap, RowAlign::End);   // RowAlign::{Start, End} (§22)
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

**9 — Composed dialog with an arbitrary body** (`examples/09_composed_dialog.rs`) <!-- amended by §21 item 34: compile fixes only (`token_st`, imports, the body field's `update`); §25 (adjudication 7: `Track::Auto` needs a measurement, so the body uses `layout::rows_measured`); §26 (Adjudication N1: the dialog sizes its own layer through `confirm_dialog().layer(cx)`, and the props are built once) -->

```rust
use junie_tui::{id, layout, Action, ActionKey, Constraints, Cx, Dialog, DialogAction, DialogState,
                DismissReason, Id, Part, Props, Response, TextInput, TextInputState, Track, Ui};

const CONFIRM: Id = id!("confirm.delete");
const TOKEN: Id = CONFIRM.part(Part::FIELD);          // a child COMPONENT id inside the dialog (§21 item 16)
const K_CANCEL: ActionKey = ActionKey::CANCEL;
const K_DELETE: ActionKey = ActionKey::custom("delete");

struct Screen { dlg: DialogState, token: String, token_st: TextInputState, target: String, deleted: bool }

// The single props constructor §13 requires (`architecture::props_are_built_once`). It is also what
// sizes the layer: `body_rows` states what the body slot needs, and `Dialog::layer` turns
// `(props, DesignTokens)` into `LayerSpec::modal(CONFIRM).size(LayerSize::Fixed(w, h))` (§26 N1).
fn confirm_dialog() -> Dialog<'static> {
    Dialog::new(CONFIRM)
        .title("Delete table")
        .description("This cannot be undone.")
        .width(60)
        .body_rows(4)                                  // props (2) + rule (1) + token field (1)
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) { cx.open_layer(CONFIRM, confirm_dialog().layer(cx)); }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = TextInput::new(TOKEN).update(cx, &mut self.token_st, &mut self.token).erase();

        let armed = self.token.trim() == self.target;      // arming is an `update` predicate
        let actions = [
            Action::new(K_CANCEL, "Cancel"),
            Action::danger(K_DELETE, "Delete").enabled(armed),
        ];
        // `Dialog::update` re-asserts the layer size first (invariant D1), so a longer description or a
        // taller body corrects the layer on the next draw without the opener predicting anything.
        let d = confirm_dialog()
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
            confirm_dialog()
                .draw(ui, area, &self.dlg, |ui, body| {           // ARBITRARY body content
                    // `Track::Auto` without a measurement is ONE cell when explicit `Flex` tracks exist
                    // (§10, §25 adjudication 7), which would clip this two-row `Props`. Supply the
                    // natural size: that is what `rows_measured` is for.
                    let props = Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")]);
                    let natural = [props.measure(ui, Constraints::loose(body.width, body.height)).preferred.1];
                    let rows = layout::rows_measured(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)], &natural);
                    props.draw(ui, rows[0]);
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
// The dialog computes a SIZE, never a rect: placement, flip and clamp stay in the one
// resolver (§9.1, §26 N1). `Rect::centered*` appears nowhere in a component.
```

**10 — Nested picker inside a dialog** (`examples/10_nested_overlay.rs`) — Scenario F <!-- amended by §21 items 1, 5, 8, 16; §26 (Adjudication N1: both overlays supply their own `LayerSize`) -->

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

fn dialog() -> Dialog<'static> { Dialog::new(DLG).title("Edit task").body_rows(1) }

fn owner_picker() -> Picker<'static, Person, impl Fn(&Person) -> ItemKey, impl Fn(&Person, &mut RowUi<'_>)> {
    Picker::new(OWNER_PICK)
        .key(|p: &Person| ItemKey::num(p.id))
        .row(|p: &Person, u: &mut RowUi<'_>| { u.label(&p.name); u.meta(&p.team); })
}

impl Screen {
    fn open(&mut self, cx: &mut Cx<'_>) { cx.open_layer(DLG, dialog().layer(cx)); }

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
                        .dismiss(Dismiss::ESC_AND_OUTSIDE)
                        // the picker's OWN arithmetic over the items it receives per phase (§26 N1);
                        // `Picker::update` re-asserts it every frame with `cx.resize_layer`
                        .size(owner_picker().measured_size(cx, &self.people))));

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
// Each overlay states a SIZE; the runtime anchors, flips and clamps it (§9.1). A popover that would
// not fit below the button is placed above it, and `Anchor::Point` flips rather than covering the
// pointer (§20.10 item 17).
// Esc closes only the picker; the dialog stays open and regains focus at the button.
// No barrier is pushed by hand, no hit region is re-registered, and the picker draws no
// hint row of its own — the top layer contributes to the shared HintBar (§13.1).
// z-order is the `LayerId` assigned by `open_layer`, not the order of the two `ui.layer` calls (§21 item 14).
```

**11 — A small complete application on shared focus and dispatch** (`examples/11_small_app.rs`) — Scenario A <!-- amended by §21 items 1, 5, 7; §28 P3 (the `Dialog` update is unconditional; the opener sizes the layer) --> <!-- amended by §28 -->

```rust
use junie_tui::{id, layout, run, Action, ActionKey, App, Button, Cx, Dialog, DialogAction,
                DialogState, Field, Id, Insets, ItemKey, List, ListAction, ListState,
                Response, RowUi, TextInput, TextInputState, Theme, Track, Ui, Variant};
// `LayerSpec` is no longer imported: the dialog sizes its own layer (§26 N1, §28 P3).

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
                    // the dialog sizes its own layer (§26 N1): a bare LayerSpec::modal
                    // would ask for LayerSize::Fill and flash for one frame.
                    cx.open_layer(CONFIRM, remove_dialog().layer(cx));
                });

        // unconditional: a layer owner's `update` runs every frame (§13, §28 P3).
        // dismissal (Esc, outside click) is delivered in the pass AFTER the layer
        // closed, so `if cx.is_open(CONFIRM) { … }` would drain nothing exactly then.
        let actions = [Action::new(K_NO, "Cancel"), Action::danger(K_YES, "Remove")];
        r |= remove_dialog().actions(&actions).cancel(K_NO)
                .update(cx, &mut self.dlg)
                .on_action(|a| {
                    if let DialogAction::Action(K_YES) = a {
                        if let Some(k) = self.pending_remove.take() {
                            self.people.retain(|s| ItemKey::text(s) != k);
                        }
                    }
                    if cx.is_open(CONFIRM) { cx.close_layer(CONFIRM, None); }   // the CALLER's work is guarded
                });
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

<!-- amended by §28 --> ~~`if cx.is_open(CONFIRM) { … }` around `Dialog::update`~~ is struck, and with it the whole gated shape: the runtime addresses `Intent::Cancel` and `Intent::Layer(Dismissed)` to `CONFIRM` in the pass after `dismiss_top` closed the layer, so a gated `update` never drains them, `DialogAction::Dismissed` is never emitted, and — because `Dialog` registers only `Decorative` regions for its own id — the loss was **silent** until §28 P3 widened the diagnostic. Examples 9 and 10 already had the unconditional shape; example 11 was the outlier.

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

**13 — The 15-field connection form on `Form`** (`examples/13_connection_form.rs`) — §15.1, §23 K1 <!-- added by §23; amended by §24 M3: option lists move to `ConnDraft::options`, `Select::new(id)`/`RadioGroup::new(id)`, no `use FieldKind::*` -->. Condensed: the file completes the elided helpers (`save`, `begin_test`, `close`, `port_rule`, `default_port`) and the four option tables (`ENGINES`, `ENVS`, `GROUPS`, `MODES`: `const ENGINES: &[&str] = &[…];` etc.), all trivial.

```rust
use junie_tui::{id, Action, ActionKey, Button, Checkbox, Cx, EnterPolicy, FieldKind, FieldRef,
                FieldMut, FieldSpan, FieldSpec, Form, FormAction, FormData, FormState, GroupKey,
                Id, KeyCode, KeyModifiers, Chord, RadioGroup, Response, Secret, SecretPolicy,
                Select, TextArea, TextInput, Toggle, Ui, Rect, FieldError};

const FORM: Id = id!("connections.form");
const NAME:  Id = id!("connections.form.name");     const ENGINE: Id = id!("connections.form.engine");
const HOST:  Id = id!("connections.form.host");     const PORT:   Id = id!("connections.form.port");
const DB:    Id = id!("connections.form.db");       const USER:   Id = id!("connections.form.user");
const PW:    Id = id!("connections.form.pw");       const ASKPW:  Id = id!("connections.form.askpw");
const ENV:   Id = id!("connections.form.env");      const GROUP:  Id = id!("connections.form.group");
const SAFE:  Id = id!("connections.form.safe");     const SSL:    Id = id!("connections.form.ssl");
const SSH:   Id = id!("connections.form.ssh");      const SSHH:   Id = id!("connections.form.sshhost");
const START: Id = id!("connections.form.startup");
const BASIC: GroupKey = GroupKey::custom("basic");  const ADV: GroupKey = GroupKey::custom("adv");
const K_TEST: ActionKey = ActionKey::custom("test");
const K_SAVE_CONNECT: ActionKey = ActionKey::custom("save+connect");

/// The 15 declarations, written ONCE, called from both phases (§13 props-built-once).
/// Configuration only: no option list enters a `FieldKind` (§24 M3‑2) — the lists are DATA and
/// arrive through `ConnDraft::options`. Spelled-out `FieldKind::…` variants: a `use FieldKind::*`
/// would shadow the imported `Select` type in the value namespace (§24 M3).
fn conn_fields<'a>() -> [FieldSpec<'a>; 15] {
    [
        FieldSpec::new(NAME,  "Name",     FieldKind::Text(TextInput::new(NAME))).required(true).group(BASIC),
        FieldSpec::new(ENGINE,"Engine",   FieldKind::Select(Select::new(ENGINE))).group(BASIC),
        FieldSpec::new(HOST,  "Host",     FieldKind::Text(TextInput::new(HOST).placeholder("localhost")))
            .help("Blank: driver default").span(FieldSpan::Half).group(BASIC),
        FieldSpec::new(PORT,  "Port",     FieldKind::Text(TextInput::new(PORT).validate(&port_rule)))
            .span(FieldSpan::Half).group(BASIC),
        FieldSpec::new(DB,    "Database", FieldKind::Text(TextInput::new(DB)))
            .help("Required for PostgreSQL").group(BASIC),
        FieldSpec::new(USER,  "Username", FieldKind::Text(TextInput::new(USER))).group(BASIC),
        FieldSpec::new(PW,    "Password", FieldKind::Text(TextInput::new(PW).secret(SecretPolicy::default())))
            .help("Never written to connections.json").group(BASIC),
        FieldSpec::new(ASKPW, "",         FieldKind::Check(Checkbox::new(ASKPW, "Prompt for password on connect")))
            .plain(true).group(BASIC),
        FieldSpec::new(ENV,   "Environment", FieldKind::Radio(RadioGroup::new(ENV))).group(BASIC),
        FieldSpec::new(GROUP, "Group",    FieldKind::Select(Select::new(GROUP))).group(BASIC),
        FieldSpec::new(SAFE,  "Safe Mode",FieldKind::Radio(RadioGroup::new(SAFE))).group(BASIC),
        FieldSpec::new(SSL,   "",         FieldKind::Toggle(Toggle::new(SSL, "Use SSL / TLS"))).plain(true).group(ADV),
        FieldSpec::new(SSH,   "",         FieldKind::Toggle(Toggle::new(SSH, "SSH tunnel"))).plain(true).group(ADV),
        FieldSpec::new(SSHH,  "SSH host", FieldKind::Text(TextInput::new(SSHH).placeholder("bastion.example.com")))
            .group(ADV),
        FieldSpec::new(START, "Startup commands", FieldKind::Area(TextArea::new(START, 3)))
            .help("Run after every connect, one per line").group(ADV),
    ]
}

fn conn_actions() -> [Action<'static>; 4] {
    [Action::quiet(K_TEST, "Test connection"),
     Action::new(ActionKey::CANCEL, "Cancel"),
     Action::new(ActionKey::SAVE, "Save").chord(Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL)),
     Action::new(K_SAVE_CONNECT, "Save & connect")]
}

/// The caller owns every value. This struct IS the connection draft.
#[derive(Default)]
struct ConnDraft { name: String, engine: usize, host: String, port: String, db: String,
                   user: String, pw: Secret, ask_pw: bool, env: usize, group: usize, safe: usize,
                   ssl: bool, ssh: bool, ssh_host: String, startup: String }

impl ConnDraft {
    /// The option tables, keyed by field id — one place, read by both `options` and `value_and_options`.
    fn option_table(id: Id) -> &'static [&'static str] {
        match id { ENGINE => ENGINES, ENV => ENVS, GROUP => GROUPS, SAFE => MODES, _ => &[] }
    }
}

impl FormData for ConnDraft {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            NAME => FieldRef::Text(&self.name),        ENGINE => FieldRef::Choice(self.engine),
            HOST => FieldRef::Text(&self.host),        PORT   => FieldRef::Text(&self.port),
            DB   => FieldRef::Text(&self.db),          USER   => FieldRef::Text(&self.user),
            PW   => FieldRef::Secret(&self.pw),        ASKPW  => FieldRef::Flag(self.ask_pw),
            ENV  => FieldRef::Choice(self.env),        GROUP  => FieldRef::Choice(self.group),
            SAFE => FieldRef::Choice(self.safe),       SSL    => FieldRef::Flag(self.ssl),
            SSH  => FieldRef::Flag(self.ssh),          SSHH   => FieldRef::Text(&self.ssh_host),
            START=> FieldRef::Text(&self.startup),     _      => FieldRef::Text(""),
        }
    }
    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            NAME => FieldMut::Text(&mut self.name),    ENGINE => FieldMut::Choice(&mut self.engine),
            HOST => FieldMut::Text(&mut self.host),    PORT   => FieldMut::Text(&mut self.port),
            DB   => FieldMut::Text(&mut self.db),      USER   => FieldMut::Text(&mut self.user),
            PW   => FieldMut::Secret(&mut self.pw),    ASKPW  => FieldMut::Flag(&mut self.ask_pw),
            ENV  => FieldMut::Choice(&mut self.env),   GROUP  => FieldMut::Choice(&mut self.group),
            SAFE => FieldMut::Choice(&mut self.safe),  SSL    => FieldMut::Flag(&mut self.ssl),
            SSH  => FieldMut::Flag(&mut self.ssh),     SSHH   => FieldMut::Text(&mut self.ssh_host),
            START=> FieldMut::Text(&mut self.startup), _      => FieldMut::ReadOnly,
        }
    }
    // the option lists are data (§24 M3): painted by `draw`, driven by `update` under ONE borrow
    fn options(&self, id: Id) -> &[&str] { Self::option_table(id) }
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), Self::option_table(id))
    }
    // conditional visibility — replaces the render-time `disabled` write at connections.rs:1205-1206
    fn visible(&self, id: Id) -> bool { if id == SSHH { self.ssh } else { true } }
    fn validate(&self, id: Id, v: FieldRef<'_>) -> Result<(), FieldError> {
        match (id, v) {
            (NAME, FieldRef::Text(s)) if s.trim().is_empty() =>
                Err(FieldError { message: "Required".into(), code: Some("required") }),
            (PORT, FieldRef::Text(s)) => port_rule(s),
            _ => Ok(()),
        }
    }
    fn validate_all(&self) -> Result<(), (Id, FieldError)> {
        if self.engine == 0 && self.db.trim().is_empty() {
            return Err((DB, FieldError { message: "PostgreSQL needs a database".into(), code: None }));
        }
        Ok(())
    }
}

struct ConnScreen { draft: ConnDraft, form: FormState, tab: GroupKey, /* … */ }

impl ConnScreen {
    fn form_props<'a>(&self, f: &'a [FieldSpec<'a>], a: &'a [Action<'a>]) -> Form<'a> {
        Form::new(FORM, f).actions(a).submit(ActionKey::SAVE).cancel(ActionKey::CANCEL)
            .enter(EnterPolicy::SubmitsWhenIdle).columns(2).group(self.tab)
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let (fields, actions) = (conn_fields(), conn_actions());
        self.form_props(&fields, &actions)
            .update(cx, &mut self.form, &mut self.draft)          // data per phase (§21 item 1); options via value_and_options
            .on_action(|a| match a {
                // the ONE cross-field rule; no widget is rebuilt (cf. connections.rs:629-631)
                FormAction::Committed(ENGINE) =>
                    self.draft.port = default_port(self.draft.engine).to_owned(),
                FormAction::Action(ActionKey::SAVE)   => self.save(false, cx),
                FormAction::Action(K_SAVE_CONNECT)    => self.save(true,  cx),
                FormAction::Action(K_TEST)            => self.begin_test(),
                FormAction::Action(ActionKey::CANCEL) => self.close(cx),
                FormAction::Invalid(_)                => cx.record("form.invalid"),
                _ => {}
            })
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let (fields, actions) = (conn_fields(), conn_actions());
        self.form_props(&fields, &actions).draw(ui, area, &self.form, &self.draft);   // options via FormData::options
    }
}

fn main() {}
// No `on_form_key` ladder, no `on_form_click`, no height arithmetic, no `validate() && validate()`,
// no focus-to-first-error block, no widget rebuild to change a value, no render-time `disabled`
// write. `ConnDraft: !Clone` because it holds a `Secret`; nothing copies the password (§15.1 F5).
// The field array is byte-identical across frames whose option tables differ (§24 M3,
// `form::changing_options_between_frames_does_not_rebuild_props`).
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
| — (new, from `ui/text.rs`) | **move** | `width`, `wrap`, `fuzzy`, `thousands`, `truncate`, `truncate_middle` | `«tui»/text/{measure.rs, fuzzy.rs}` | `fit`/`fit_right` **removed from every render path** (R5); `fuzzy` returns grapheme indices into the *original* label (rewritten over `grapheme_indices(true)`), fixing the latent mis-highlight. <!-- amended by §22 --> `measure.rs::width` is the **one** width function in the workspace and delegates to `ratatui_core::buffer::CellWidth::cell_width` (halfwidth katakana sound marks count as one column, matching what `Buffer::set_stringn` consumes — R‑1, §20.10-16); `unicode_width::` may be imported nowhere else. <!-- amended by §26 (Adjudication N1) --> `measure.rs` also owns `wrapped_rows(s, width) -> u16` (a 0-allocation grapheme/word walk), the single wrap `Dialog::measured_height` and `Dialog::draw` share so the two cannot drift. Satisfies §20.9-7. |
| `src/runtime.rs` | **refactor** | `Runtime<A>`, `App`, `TerminalSession`, `DefaultTerminal`, `run`, `drain_pending_input`, `Diagnostic` | `«tui»/runtime.rs`, `«tui»/runtime/session.rs`, `«tui»/diagnostics.rs` | `Application::render(&mut self)` **deleted** — the sanction for render-time mutation at the top of the stack. Adds the two-phase frame (§3.3), the layer compositor, `request_repaint_after`, and `diagnostics()`. <!-- amended by §22 --> `TerminalSession` mirrors ratatui's `try_init`/`try_restore` (unavailable through `ratatui-core`), replaces the raw `ENABLE_WRAP = "\x1b[?7h"` string (`runtime.rs:42`) with typed `EnableLineWrap`/`DisableLineWrap`, and is the only file allowed to name `enable_raw_mode`/`EnterAlternateScreen`/`EnableMouseCapture`/`EnableBracketedPaste` (§22 §6.2 rules 2–4). Mouse normalisation matches `MouseEventKind` exhaustively and keeps `modifiers` (R‑15). Satisfies G2. <!-- amended by §25 (adjudication 1, D‑1) --> The `crossterm` feature gates **only** `TerminalSession`, `run` and `DefaultTerminal`. `Input::from_crossterm` and the `event.rs` re-export of `KeyCode`/`KeyModifiers` are **not** feature-gated — they need no backend, only crossterm's event vocabulary — so `cargo check --no-default-features` still compiles `ratatui-crossterm` and proves only that nothing outside `runtime/session.rs` needs a *backend*; `CrosstermBackend` is confined by forbidden-pattern rule 27 and `architecture::ratatui_crossterm_is_named_in_exactly_two_files`. |
| `src/theme.rs` | **decompose** | `Theme`, `ColorTokens`, `DesignTokens`, `Role`, `GlyphRole`, `GlyphSet`, `BorderSet`, `Recipe`, `Recipes`, `PartRecipe`, `StateRule`, `StylePatch`, `Slot`, `Overlay`, `Resolved`, `Family`, `Variant`, `Capability`, `ColorLevel` | `«tui»/theme/{mod,tokens,role,glyph,recipe,patch,resolve,downgrade}.rs`, `«tui»/theme/builtin/{junie,paper}.rs` | Flat 30-field `Copy` struct **deleted**; `lift`/`backdrop` colour-equality dispatch **deleted** (ladder index arithmetic, §10); the 30-field `for_level` macro **deleted** (`map_colors` exhaustive destructure); `Theme::change_glyph` (added from `grid.rs`) **deleted**. Junie token values preserved verbatim. Satisfies G6, §11. |
| `src/ui/ctx.rs` | **decompose** | `Ui`, `Cx`, `Surface`, `StyleStack`, `LayoutFacts` | `«tui»/ui/{mod,cx,paint,surface,layer_buf}.rs` | `RenderCtx`, `Interaction`, `begin_modal`, `focus_hidden` (dead), public `hits`/`ring` **all deleted**. Adds clip rect, surface stack, style stack, written-cell bitset, `raw()` escape hatch. Satisfies R2–R4, §10. |
| `src/ui/layout.rs` | **refactor + merge** | `layout::{rows, columns, responsive_columns, action_row, inset, split_v, split_h}`, `Track`, `Insets`, `RowAlign`, `SplitModel`, `Constraints`, `Size`, `Measure` | `«tui»/layout.rs`, `«tui»/measure.rs` | `Split`'s vertical/horizontal minima asymmetry **fixed** (first pane wins on both axes). Absorbs `button::row_layout*` and `showcase/pages/mod.rs`'s `rows`/`columns`/`caption`. Module doc's "the workbench" (domain leak) removed. |
| `src/ui/popup.rs` | **remove → compose** | `LayerId`, `LayerKind`, `LayerSpec`, `Anchor`, `Side`, `CrossAlign`, `Dismiss`, `Backdrop`, `LayerEvent`, `DismissReason` | `«tui»/layer.rs` | Both `Placement` enums and both placement algorithms **deleted**; the shared `WidgetId::of("popup.surface")` **deleted**; the hand-written `Rect::centered` in `dialog.rs` **deleted** — replaced by `ratatui_core`'s `Rect::centered`/`centered_horizontally`/`centered_vertically`/`clamp` (§22 R‑12). One resolver: flip, clamp, clip, min-size. Satisfies §9. |
| `src/ui/text.rs` | **move** | see `text/measure.rs` above | `«tui»/text/` | Module removed; no `ui::text` namespace remains. |
| — (new, out of `viewport.rs`) <!-- amended by §24 M1 --> | **compose** | `Span<'a>` (ours, role-carrying) | `«tui»/text/span.rs` | Consumed by `RowUi::label_spans` (`collection/`), `Ui::paint_spans` (`ui/paint.rs`) and `TextViewport` (`components/`); lives in `text/` so `collection → components` is never a dependency edge for a type name. Re-exported at the root and in `author` as `Span`; ratatui's `text::Span` is reachable only as `author::raw::Span`. |

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
| `grid` | **decompose** | `Grid<'a>`, `GridState`, `GridModel`, `GridEditor`, `ColumnKey`, `Column`, `CellRef`, `NavUnit`, `EditIntent`, `CellAction` (`GridCellActions` **struck** — its `actions` is a defaulted `GridModel` method, §23 K2) <!-- amended by §23 --> | `«tui»/components/grid.rs` | Everything DB-shaped moves out (see 18.4 TablePro row). `CellValue`, `PendingChanges`, `UndoAction`, `default_validator`, `cmp_cells`, `Validator` fn pointer, `"Preview SQL"`, `primary`/`nullable`/`references`/`enum_values`, `Theme::change_glyph` **all deleted from the library**. `col_rects.clone()` per row **deleted**. `GridEditor` is `&mut self` and reachable only from `Grid::update_editable`; `Grid::update` takes `&M: GridModel` (§23 K2). Satisfies §12.3, Scenario H, DOM §7 condition 1. |
| `hintbar` | **retain + wire** | `HintBar`, `HintLayer`, `Hint` | `«tui»/components/hintbar.rs` | Layers are now *derived*: top layer ▸ mode ▸ focused component's visible bindings ▸ screen extras ▸ global. The ~700 lines of hand-written hint tables across the apps are deleted. |
| `input` | **decompose** | `TextInput<'a>`, `TextInputState`, `Secret`, `SecretPolicy`, `BlurPolicy` | `«tui»/components/input.rs`, `«tui»/secret.rs` <!-- amended by §25 D‑9: `secret.rs`, `validate.rs` and `field_control.rs` live at the CRATE ROOT — they are foundation vocabulary consumed by `components/input.rs`, not components, and `components/` is a Slice-4 directory the Slice-3 owner may not write --> | Render-time commit+validate (`input.rs:282`) **impossible**; `validator: Option<fn>` → `&dyn Validate`; `plain_label` **deleted** (`Field` owns chrome); `HEIGHT` **deleted** (`measure`); `reveal_tail` **re-specified** to a synthetic tail; 5 tiny-rect underflows fixed with `saturating_sub`. Satisfies §15, API §5. |
| `keyhint` | **retain** | `KeyHint` | `«tui»/components/keyhint.rs` | Rendered by `HintBar`; the free-function entry point stays for one-off chips. |
| `list` | **refactor** | `List<'a,T,K,R>`, `ListState`, `SelectMode`, `KeySet` | `«tui»/components/list.rs` | Owned `ListItem` **deleted** (borrowed `&'a [T]`); `row_id`/`locate`/`owns` **deleted**; boundary-wheel violation (`list.rs:180`) fixed; `SelectMode` gains `Range`/`None`; `KeySet` gets `AllExcept` (R7) over a sorted `Vec<ItemKey>` with binary-search `contains` (§22). Satisfies Scenario D. |
| `menu` | **retain + extend** | `MenuBar<'a>`, `ContextMenu<'a>`, `MenuItem<'a>`, `MenuState` | `«tui»/components/menu.rs` | Render-time cursor move on hover (`menu.rs:243`) → an explicit `Intent::Pointer{Move}`; `shortcut: &'static str` → a real `Chord` that both renders and binds (kills jackin's `run_host_menu` key synthesis); label-string dispatch → `ActionKey`; `MenuItem::submenu` added; own `Placement` merged into `Anchor`; becomes layer content. |
| `panel` | **split** | `Panel<'a>` retained; `ScrollPanel` **removed** | `«tui»/components/panel.rs` | `bg: Color`, `Panel::bg(t)` and `pub bg_override` **all deleted** (contextual `Surface`). Framed inner rect escaping the panel for `width ≤ 4` **fixed**. `ScrollPanel` callers migrate to `TextViewport` with tone-carrying spans. |
| `picker` | **decompose** | `FilterList<'a,T,K,R>` (headless), `Picker<'a,…>` overlay, `CommandPalette`, `PickerState`, `ScopeKey` | `«tui»/components/{filter_list,picker}.rs` | `hints: &str` **deleted** (a `HintLayer` contribution); positional `row_id` → `ItemKey`; `scopes` first-class; `PickerStatus` folded into `EmptyState`; `Delete`-secondary gains a mouse equivalent (§20.10-4); duplicated backdrop dim **deleted**. |
| `progress` | **decompose** | `Spinner`, `ProgressBar`, `Meter`, `MeterTone`, `MeterVisual` | `«tui»/components/{progress,meter}.rs` | Five `bg: Color` parameters **deleted**; `METER_LOW_MAX`/`MEDIUM_MAX` and `SPINNER` move to `DesignTokens` (A4); `MeterTone::{Warning,Exhausted,Stale,Refreshing}` (jackin quota lifecycle) move to jackin; `MeterTone::from_ratio` helper kills the app-side duplicate matches (J12). |
| `props` | **refactor** | `Props<'a>` (static) + `PropsList<'a,T,K,R>` as a two-column `List` variant | `«tui»/components/props.rs` | The **two independent render paths** (free fn vs `PropsList::render`) collapse to one; `locate`/`owns`/`row_id` deleted; used by `Dialog::facts`. |
| `scrollbar` | **retain as a part** | `Part::TRACK` / `Part::THUMB` of a `scroll_region` | `«tui»/components/scroll_region.rs` | `scrollbar::id_for` **deleted** (26 showcase + 18 tablepro + ≥4 jackin call sites). One `on_scrollbar` implementation replaces seven copies; thumb drag uses pointer capture. <!-- amended by §22 --> The loose `TRACK`/`THUMB` consts (`scrollbar.rs:8-9`) become a typed `ratatui_core::symbols::scrollbar::Set<'static>` holding the Junie glyphs; `ratatui_widgets::Scrollbar`/`ScrollbarState` are rejected (no `Part` hit regions, no recipe resolution, a second source of truth beside `ScrollState`). |
| `segments` | **merge** | absorbed by `StatusBar` | `«tui»/components/status.rs` | Two priority-drop loops become one `Left/Center/Right` item strip; `bg: Color` deleted. |
| `select` | **rebuild** | `Select<'a,T,K,R>`, `SelectState`, `SelectAction` (`Select::new(id)`, items per phase; `LabelSelect<'a>` is the `FieldKind` alias — §24 M3 <!-- amended by §24 -->) | `«tui»/components/select.rs` | Render-time overlay close (`select.rs:161-167`) **impossible**; the popup becomes a `Popover` layer (so the focus barrier bug in `ui/popup.rs` disappears); 10-row clip → a real scroll region; `HEIGHT` deleted. |
| `splitter` | **merge** | `SplitPane<'a>`, `SplitPaneState` (with `SplitModel`) | `«tui»/components/split.rs` | `Splitter` + `ui::layout::Split` become one component that owns its container rect from its own draw; caller-held `seam_container: Rect` fields in three jackin screens **deleted**; drag through capture; optional keyboard resize as a binding. |
| `statusbar` | **retain + promote** | `StatusBar<'a>`, `StatusItem<'a>`, `Emphasis` | `«tui»/components/status.rs` | Absorbs `segments`; gains TablePro's identity strip and grid status line as consumers, deleting two hand-written priority-drop loops; `STATUS_METER_TRACK` moves to `design.size.meter_track`. |
| `steps` | **refactor** | `Steps<'a,T,K,R>`, `StepsState`, `StepState` | `«tui»/components/steps.rs` | Stays a *display* rail with a frontier (the meaningful difference, DOM §6.2); gains keys and a row renderer; the step *flow* becomes the separate `Wizard` (J7). |
| `table` | **remove** | absorbed by `Grid` with `NavUnit::{Row, Cell}` | — (`«tui»/components/grid.rs`) | `DataTable` **deleted**: its `Column`, `Cell`, third `EditState`, string sort, `validator: fn`, `locate`/`locate_header`, double cell registration, and 4 ragged-row panics all go. TablePro's Structure tab becomes six `GridModel`s. |
| `tabs` | **refactor** | `Tabs<'a,T,K,R>`, `TabsState`, `TabsAction` | `«tui»/components/tabs.rs` | Positional `tab_id(i)`/`close_id(i)` **deleted** → `ItemKey`; per-frame `areas`/`widths` `Vec`s **deleted**; the "rebuild the whole widget and rescue `first`/`active`" idiom in both apps **deleted**; strip window follows the logical first tab (§20.10-13). Satisfies Scenario E. |
| `textarea` | **refactor** | `TextArea<'a>`, `TextAreaState` | `«tui»/components/textarea.rs` | Render-time commit (`textarea.rs:202`) **impossible**; shares `TextEditorCore` with `input`/`code`; missing `owns`/`on_scrollbar` supplied by `scroll_region`; 1-cell-width underflow fixed. |
| `tree` | **refactor** | `Tree<'a,T,K,R>`, `TreeState`, `TreeNode<'a>`, `TreeAction` | `«tui»/components/tree.rs` | `path: Vec<usize>` identity → `ItemKey` (`TreeNode::keyed`); `expanded: HashSet<Vec<usize>>` → `HashSet<ItemKey>`; `flatten()` becomes **incremental and borrow-based** (§20.9-8); `FlatRow`'s duplicate `label`/`meta` **deleted**; row renderer added (kills TablePro's paint-over-the-tree hack); `object_at`/`schema_at` path reconstruction **deleted**. |
| `viewport` | **retain (rewritten storage)** | `TextViewport<'a>`, `ViewportState`, `ViewportAction` (`Span<'a>` moves to `«tui»/text/span.rs`, §24 M1 <!-- amended by §24 -->) | `«tui»/components/viewport.rs` | Best-in-class behaviour preserved verbatim. `Cell { g: String }` → `{ range: Range<u32>, w: u8, style_ix: u16 }`; layout becomes **incremental + windowed**; `set_area`/`prime`/the `inert` clone dance **deleted** once view state is caller-owned. Satisfies §20.9-7. |

### 18.3 The 23 app-side reusable controls (**[F]** APP §3)

| # | Current control | Disposition | Target type(s) | Target file | Notes |
|---|---|---|---|---|---|
| 1 | `NavList` + `NavItem` (`showcase/pages/sidebars.rs:16-165`) | **move** | `NavList<'a,T,K,R>`, `NavListState` | `«tui»/components/nav_list.rs` | Sections, collapsed icon-only mode, badges and disabled skipping become `List` features; the control's own `ctx.ring.register` and reverse `locate` scan are deleted. |
| 2 | Shell nav sidebar (`showcase/app.rs:868-926, 461-492, 696-698`) | **compose** | uses #1 | `«showcase»/app.rs` | `nav_index_at`'s 22-id reverse scan and the hand-written sidebar key table are deleted; the digest baseline now covers the sidebar (§20.10). |
| 3 | `static_field` (`showcase/pages/inputs.rs:65-106`) | **compose** | `TextInput` + `Field` + `.state_override(StateFlags)` | `«showcase»/pages/inputs.rs` | Needs the documented "render in state X without owning state" path: the showcase supplies a `TextInputState` fixture per cell. The fake cursor cell and manual underline are deleted. |
| 4 | Button state matrix (`showcase/pages/buttons.rs:143-176`) | **compose** | `Button` × `Variant` × `StateFlags` fixtures | `«showcase»/pages/buttons.rs` | Same mechanism as #3; the re-implemented renderer (`t.button` + `t.gutter` + `set_string`) is deleted. <!-- amended by §28 --> **Until `apps/showcase` exists (Slice 5) the migrated page lives in two halves**: the runnable page in `crates/tui/examples/showcase_buttons.rs`, and this reference **state matrix** — which needs `.state_override` (A11) — in `crates/tui/tests/showcase_buttons.rs`, where `architecture::state_override_is_used_only_in_apps_and_fixtures` admits it. An example is a binary no test may import (`#[path]`/`include!` are forbidden by the same check), so folding the matrix into the example would delete the only assertion in the tree that a forced rendering registers no ids. This is a **recorded, expiring deviation**: at Slice 5 both halves move into `apps/showcase/src/pages/buttons.rs` and `apps/showcase/tests/`, the split disappears, and this paragraph is struck. The check's allow-list is **not** widened in the meantime — neither to `examples/**` nor to a `showcase_*` path prefix (§28 P1). |
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

<!-- amended by §22, §23, §24: rows for MOD, ADJ‑K and ADJ‑M appended; the K1 `Form` rejections are tabulated in §15.1 -->

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
| The `ratatui` umbrella crate as the library's dependency | MOD §1 (§22) | Its defaults drag `ratatui-widgets`, `ratatui-macros`, `layout-cache` + `critical-section` into the graph; upstream's own guidance is that widget libraries depend on `ratatui-core`; the only cost is losing `init`/`restore`/`run`, which `TerminalSession` mirrors | `ratatui-core` (`std`, `underline-color`) + `ratatui-crossterm` behind a default-on `crossterm` feature; apps depend on `junie-tui` alone (§22 §1) |
| `ratatui-macros` (`constraints!`, `line!`, `span!`, `row!`) | MOD §1.5 (§22) | A second layout vocabulary beside `Track`; per-invocation `Span`/`Line` allocations against §16.6's 0-alloc row path; `row!` targets the deleted `DataTable`; goal §5/§2.2 "no macro DSL" | `layout::*` + `RowUi::label_fmt`; the only macro stays `id!` |
| `Stylize` shorthands (`"x".red().bold()`) | MOD §2.9 (§22) | Every shorthand names a literal ANSI colour, which goal §15 and §11.3 A2 forbid inside components — a deliberate inversion of the common ratatui idiom | `ui.style(family, variant, part, flags) -> Resolved`; `Style::new()` only; banned in library and app code (R‑8) |
| `ratatui_widgets::{Scrollbar, ScrollbarState}` | MOD §2.13 (§22) | A `Widget` cannot register `Part::TRACK`/`THUMB` hit regions for thumb drag, cannot resolve through the recipe system, and `ScrollbarState` would be a second source of truth beside `ScrollState` | `ScrollRegion` + typed `symbols::scrollbar::Set` |
| `Block`/`Padding`/`Fill`/`Dimmed`/`Shadow` | MOD §2.1, §2.14 (§22) | Foreign `Widget`s write straight to the `Buffer` and cannot mark the written-cell bitset; `Dimmed` is colour-space arithmetic where §11.3 A2 needs role arithmetic and cannot exclude the footer row; `Padding` duplicates `Insets` | `Ui::frame`, `Ui::fill`, `Ui::paint_style`, `Ui::dim_layer`, `Insets` — recorded as deliberate re-implementations |
| `ratatui_core::text::Masked` | MOD §2.15 (§22) | Its `Debug` prints the raw secret verbatim (`masked.rs:50-56`) | `Secret` + `SecretPolicy` with a synthetic tail (§15); `Masked` forbidden (R‑19) |
| `ToSpan`/`ToLine` as the `Display → row label` bridge | MOD §2.16 (§22) | Go through `to_string()` and allocate | `RowUi::label_fmt(core::fmt::Arguments)` |
| Keyboard-enhancement flags (`PushKeyboardEnhancementFlags`) | MOD §2.6 (§22) | `DISAMBIGUATE_ESCAPE_CODES` turns bare Esc into a CSI-u sequence (a second path through the Esc ladder); `REPORT_EVENT_TYPES` delivers releases §3.3 drops anyway; terminal-dependent, contradicting §16.4's determinism | Never pushed; `KeyboardEnhancementFlags` is a forbidden pattern (R‑17); `key_release_is_dropped` synthesises its `Release` event |
| Our own `KeyCode`/`KeyModifiers` enums | MOD §1.2 (§22) | ~40 hand-written variants plus `From` impls to replace well-understood code (goal §21) | `crossterm::event::{KeyCode, KeyModifiers}` reached only through `ratatui_crossterm::crossterm` (R‑14) |
| `smallvec` for `PartRecipe.states`, `Recipe.variants`, `KeySet` | MOD §4.2 (§22) | The inline bounds are not real bounds (public `RecipeEdit::when`, mono fallbacks, 5 000-row selections); resolution only *reads* these containers so `Vec` is equally 0-alloc; ~320 inline bytes per `PartRecipe` × ~34 × ~34 makes `Theme::clone()` measurably worse; a new dependency node; smallvec 2 is alpha | `Vec`; `KeySet` as a sorted `Vec<ItemKey>` with binary-search `contains` |
| `ratatui-core/layout-cache`, `palette`, `scrolling-regions` features | MOD §1.4 (§22) | `layout-cache` is a process-global LRU that perturbs §16.6's deterministic allocation counts and caches a solver §10 does not use; `palette` adds only `From<Srgb>` (there is no `Color::from_hsl`) and pulls `libm`; `scrolling-regions` affects inline viewports only | features exactly `{std, underline-color}` |
| Raising the MSRV above 1.88 | MOD §5 (§22) | Every direct dependency declares 1.88; nothing the architecture uses is newer than 1.88; const-trait support (the one feature that would matter) is not stable in any released toolchain | `rust-version = "1.88"` held and *verified* by a `cargo +1.88.0 check` CI job |
| `cargo-semver-checks` as a gate during the refactor | MOD §3.4 (§22) | Every check fails by construction during a total public-API rewrite | `xtask semver` added at `v0.1.0`, blocking from `v0.1.1` |
| `Grid::update<M: GridEditor>` with defaulted refusals | ADJ‑K K2.4(a) (§23) | Every read-only model implements a trait named "Editor" (~10 of ~14 sites); `update` takes `&mut M` for a grid that cannot edit; a forgotten `commit_cell` override compiles into a silent refusal; `read_only_reason`/`actions` still unreachable from `draw` | Two entry points `update` / `update_editable`; `read_only_reason` and `actions` on `GridModel` |
| Blanket `impl<M: GridModel> GridEditor for ReadOnly<M>` | ADJ‑K K2.4(c) (§23) | Needs an `unsafe` `repr(transparent)` reinterpretation or storing the wrapper in app state (contradicting §21 item 1); a second blanket for actions risks coherence conflicts; still "update always requires `GridEditor`" | as above |
| `update_readonly` / `update` naming | ADJ‑K K2.4(d) (§23) | `update` and `draw` should share one bound; read-only call sites outnumber editable ones; the capability is named where the extra capability is used | `update` / `update_editable` |
| Renaming our `Size`/`Span` (`Extent`, `RoleSpan`) or theirs on export (`TermSize`, `TextSpan`) | ADJ‑M M1 (§24) | Renames the whole `measure`/`label_spans` vocabulary to make room for two types no signature mentions; or invents names that exist in no upstream doc for types only ever seen behind `raw()` | Neither exported; ours keep their names; `author::raw::{Line, Span, Text}` qualified-only (§24 M1) |
| A `junie_tui::ratatui` / `junie_tui::tty` umbrella submodule for foreign types | ADJ‑M M1 (§24) | `junie_tui::ratatui::` trips §22.7 forbidden-pattern rule 14 (`\bratatui::`) at every use site; `tty` misnames text and geometry types | `author::raw`, named for what it is for (§24 M1) |
| Dropping the ASCII border set, or a `BorderSet(Set)` newtype to hang consts on | ADJ‑M M2 (§24) | Dropping removes a capability §11.2 declared while `borders_set` already supports it; a newtype re-adds the wrapper §22.2 item 12 deleted and breaks `borders_set(border::PLAIN)` | `pub const ASCII: Set<'static>` beside the re-exported ratatui sets (§24 M2) |
| Automatic ASCII border selection from terminal capability | ADJ‑M M2 (§24) | Needs runtime detection (contradicts §16.4 determinism, as §22.2 item 6 rejected for key flags); ASCII edges with unicode `GlyphSet` glyphs everywhere else; re-adds the `Capability` axis §21 item 19 deleted | Manual opt-in via `ThemeBuilder::borders_set(border::ASCII)`; a full `GlyphSet` fallback is a named deferred adjudication (§24 M2) |
| Generic `FieldKind<'a, TS,KS,RS, …>` | ADJ‑M M3 (b1) (§24) | A closed enum cannot carry a per-variant type parameter; nine parameters, and every element of `&[FieldSpec]` must share one instantiation, so a form could not hold two `Select`s over different item types; leaks generics into `Form`, `FormState` and every screen (§13) | Non-generic `FieldKind` over `LabelSelect`/`LabelRadio`/`LabelChips` aliases (§24 M3) |
| `FieldKind::Control(&'a dyn FieldControl)` | ADJ‑M M3 (b2) (§24) | `FieldControl` has an associated `State` and is not dyn-safe; erasing it erases what `FormState.slots` keys on, and `Form` still needs the value-shape discriminant — the enum comes back beside the trait object | as above |
| Option slice inside the variant (`Select(LabelSelect, &[&str])`) or a `Select::labels(id, &[&str])` constructor | ADJ‑M M3 (a′, a″) (§24) | Puts data back in props (§21 item 1) and re-opens the disjoint-borrow question; a second constructor is one name too many when `T = &str` is inferred from the variant | `FormData::options` / `value_and_options` (§24 M3) |
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
| 1 | **State rules are stored in specificity order at recipe-build time.** `PartEdit::when` inserts into `PartRecipe.states` sorted by `when.count_ones()` ascending, ties by declaration order. §11.3 step 3's "ordered by specificity" is therefore a *storage* invariant, not a resolution-time sort. `Ui::style` is `for rule in &part.states { if s.contains(rule.when) { acc = acc.merge(rule.patch) } }` and **allocates nothing**. (R2) <!-- amended by §25 (adjudication 8) --> Two corrections. (a) The precedence the loop implements is `family base → variant base → family AND variant state rules merged in ascending specificity, family first on a tie` — the family's state rules must not be applied before the variant's base, or any variant that sets a base colour silently defeats the family's `HOVERED`/`FOCUSED`/`PRESSED`/`ERROR` rules (BL‑1, F1). Both rule lists are stored pre-sorted, so the merge is a stable two-way walk: still allocation-free, still O(n+m). (b) ~~ns ≤ 2× the pre-refactor `Theme::row`+`Theme::gutter` baseline~~ is **struck**: it compared a field read on a 30-field `Copy` struct against a six-level precedence resolution and was unmeetable by any correct implementation of §11.3. | §11.3 | `style_resolve_10k_parts` — **exactly 0 allocations** plus **cache hit rate ≥ 90 %**; ns recorded and asserted only under `PERF_STRICT=1` against the baseline × 1.2, plus an absolute **≤ 16 ns per query** budget under `PERF_STRICT=1`; <!-- amended by §27 (Adjudication O4a) --> ~~the frame-level bound is `style_resolve_per_frame` (≤ 5 % of `frame_showcase_lists_120x40`'s ns)~~ is **deferred to Slice 5** — that frame lives in `apps/showcase/tests/perf.rs`, which Slice 5 owns, and does not exist, so the share cannot be asserted against it. The standing bound is the per-query budget, which is §25.8's own ≈ 26 µs-per-2 000-query arithmetic made executable and machine-independent. Precedence: `theme::state_rules_beat_a_variant_base`, `theme::family_and_variant_state_rules_interleave_by_specificity` |
| 2 | **The §11.1 A3 memo cache is allocation-free and statically sized.** <!-- amended by §27 (Adjudication O1) --> A `[(u64, u32, StylePatch); 256]` behind **one** `Box`, owned by the runtime's frame core (`self.core.style_cache`) and reused across frames, keyed by a 64-bit mix of `(Family, Variant, Part, StateFlags, overlay_stack_hash)` with the low bit forced so `0` stays the empty sentinel, and cleared by a generation stamp rather than by zeroing. The 256 entries are grouped into **128 two-way sets**, insert-at-most-recent. ~~A `[Option<(u64, Resolved)>; 256]` direct-mapped array embedded in `Ui`~~ is **struck** on all four counts: there is no `Option` (the sentinel is `key \| 1`), the value is a `StylePatch` and not a `Resolved` (as this cell's own prose already said), the array lives behind a `Box` on the runtime core rather than by value in `Ui`, and it is not ~~direct-mapped~~. A one-way table of 256 entries cannot meet §16.6's ≥ 90 % hit rate for *any* realistic key set: with `k` hot keys the expected number of colliding pairs is `C(k,2)/256`, and a colliding pair in a round-robin loop misses on **every** access. `style_resolve_10k_parts` touches 32 keys (4 parts × 8 states); `C(32,2)/256 ≈ 1.94` pairs collide, ≈ 3.65 keys thrash, and the measured rate is **87.2 %** whatever the hash — this is the birthday load of 32 keys over 256 buckets, not a property of FNV, so no re-hashing recovers it, and only a perfect hash over a statically known key set would, which this key set is not. A realistic frame resolves 100–300 distinct tuples, so 87.2 % is a synthetic *best* case, not a floor. Two ways make a miss require three keys in one set (`C(32,3)/128² ≈ 0.28` expected sets, ≈ 99.7 % expected hit rate), which is what makes the memo's health assertable at all. The array shape, the single construction-time allocation, the absence of any per-frame allocation or growth, and the generation stamp are **unchanged**. <!-- amended by §25 §4(f); §26 --> `Surface` is deliberately **out** of the key: the memo caches §11.3 steps 1–5, which are role-level and surface-independent, and roles bind to colours afterwards in `bind`, per query. The stored value is therefore a `StylePatch`, not a `Resolved`. The memo serves the **painting path only**; `Ui::resolve` (the `&self` path `Measure::measure` uses) bypasses it entirely, so a measurement can never evict a painting entry. No `HashMap`, no `Vec`, no per-frame allocation, no growth. A miss recomputes; <!-- amended by §27 (Adjudication O1) --> ~~there is no eviction policy to get wrong~~ is **struck** — the eviction policy is exactly *shift way 0 into way 1, insert at way 0, promote a way-1 hit*, bounded at two entries, O(1), and independent of the key count. Generation-stamp clearing must handle wrap: at `u32::MAX` the stamp is reset to 1 **and the slots filled**, or a slot stamped with the original generation 1 becomes a false hit serving a stale `StylePatch` (`theme::cache_generation_wrap_does_not_serve_a_stale_entry`). `Ui` keeps a running `stack_hash: u64` updated on `with_overlay` push/pop so no per-query stack hash is computed (P3), and `Ui` is constructed once per `Runtime`/`Scene` and reused, never per frame (P4). | §11.1 A3 | `style_resolve_10k_parts`, `render_twice_allocates_the_same` |
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
| 16 | **Display width follows `CellWidth::cell_width`, not raw `unicode-width`** <!-- amended by §22 -->. Any string containing U+FF9E/U+FF9F (halfwidth katakana sound marks) now measures one column wider per mark, so layout reserves exactly the columns `Buffer::set_stringn` consumes. | **[F]** MOD §2.3: today's `width()` (`ui/text.rs:6-8`) reports these as zero-width while the buffer paints them as one cell — an off-by-*k* overflow past the reserved rect, the R4/R5 class. | `text::width_matches_ratatui_cell_width`; the `render::components::*` and app digests re-blessed **with** a `docs/visual-changes.md` entry under this item (realistically zero cells change in the three apps' fixtures, which makes the diff a confirmation rather than a review burden) |

| 17 | **`Anchor::Point` flips instead of covering the pointer** <!-- amended by §26 (Adjudication N1) -->. A tooltip or context menu opened near the bottom or right edge is placed **above** / **left of** the pointer when the content does not fit below/right, instead of sliding over it. `Anchor::Rect` already flipped; `Point` placed and clamped, so the overlay covered the cell the user was pointing at. | The flip was unreachable while every `LayerSpec` asked for the whole screen (`min_size: (0,0)` ⇒ the screen), so the defect was invisible; with `LayerSize::Fixed` the resolver has a real size and the two anchors must behave alike (§9.1 "one resolver"). | `layer::point_anchor_flips_instead_of_covering_the_pointer`; any tooltip/context-menu capture near a screen edge is re-blessed **with** a `docs/visual-changes.md` entry under this item |


| 18 | **Mono `DISABLED` gains `DIM` on `FIELD`/`TEXT` and stops tinting the foreground into the background** <!-- amended by §28 (Adjudication P6) -->. At `ColorLevel::Mono` the `DISABLED` rules add `DIM` (and remove every other modifier) on `LABEL`, `MARKER`, `FIELD` and `TEXT`, and set `fg = Role::Fg(Primary)` instead of `Role::Fg(Faint)`. `MONO_RULES_PER_FAMILY` goes 16 → 18. `TextInput` additionally paints `design.motion.spinner_frames[0]` in its trailing marker cell for `BUSY`/`LOADING` and declares `Part::ICON`. | Two defects in one place. (a) No mono rule reached the parts a *text control* paints (`FIELD`, `TEXT`), so a disabled `TextInput` was indistinguishable from an enabled one and `TextInputCase` narrowed `DISABLED` away rather than fail. (b) Worse, `Fg(Faint) #262626` and `disabled_fg #4d4d4d` both have `Y < 0.35`, which `mono()` maps to `Black` on a `Black` canvas under `Theme::junie()` — a disabled control was **invisible**, not merely colourless, and goal §29 asks for *readable*. `Theme::paper()` escaped only by landing in the `Reset` band. | `theme::mono_disabled_is_dim_and_readable` (the `fg != bg(Surface::Canvas)` assertion that would have caught it); `theme::mono_appends_one_state_rule_per_family` (== 18); `conformance::{text_input,button,field,list,tabs}::mono_states_are_distinguishable` with the un-narrowed state lists; the **mono half only** of `render::components::{text_input,field,list,button,tabs}::disabled` in `crates/tui/tests/baselines/components.txt` re-blessed in the fixed order **change → capture → classify → bless**, with a `docs/visual-changes.md` entry under this item before the bless (truecolor lines are untouched — mono rules are appended only at `ColorLevel::Mono`) |

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
* **Gate:** <!-- amended by §28: the `render::components::` reachability line (P2) -->
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
  cargo test --workspace -- --list | rg -c '^render::components::'   # non-zero: the matrix is reachable
                                                    # under the documented path from BOTH targets (§16.3, §28 P2)
  cargo run -p xtask -- doc-check                   # §21 item 34
  cargo check -p tui-next --no-default-features     # §22: the core is backend-free
  cargo +1.88.0 check --workspace --all-targets --all-features   # §22: the MSRV is a fact, not a field
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
| 4F | Overlays: dialog, menu, picker, completion, form, wizard, chain, help | `components/{dialog.rs, menu.rs, picker.rs, filter_list.rs, completion.rs, form.rs, wizard.rs, picker_chain.rs, help.rs}`, `examples/{09_composed_dialog.rs, 10_nested_overlay.rs, 11_small_app.rs, 13_connection_form.rs}` | Slice 3; **4B** (`Form` composes `Field`), **4C** (`Picker` composes `FilterList` rows). <!-- amended by §23 --> `Form`'s API is fixed by §15.1 (Adjudication K, K1); it is no longer an open research item |
| 4G | Status, hints, progress, meters | `components/{status.rs, hintbar.rs, progress.rs, meter.rs}` | Slice 3 |
| 4H | Code editor and diff | `components/{code.rs, diff.rs}` | Slice 3; **4E** (`DiffView` composes `TextViewport`) |
| 4I | Generic grid | `components/grid.rs`, `crates/tui/tests/fixtures/grid_model.rs` (a test-only model — the TablePro adapter is Slice 6) | Slice 3; **4C** (shared collection vocabulary). <!-- amended by §23 --> `Grid::update`/`update_editable`/`draw` bounds, `GridModel::{read_only_reason, actions}` and the deletion of `GridCellActions`/`Grid::editable` are fixed by §12.3 (Adjudication K, K2); the fixture implements `GridModel` only |

Shared, contended files are handled by convention rather than by ownership: `components/mod.rs`, `crates/tui/src/lib.rs`'s re-export list, `crates/tui/tests/conformance.rs`'s `conformance_suite!` list, and `examples/02_custom_theme.rs`/`03_partial_theme.rs`/`04_family_recipe.rs` (which touch every family's recipe defaults). Each is **append-only in a fixed, alphabetically sorted region**, so concurrent additions merge cleanly; the coordinator resolves the ordering once per slice.

* **Wave order** (to honour the dependencies above): wave 1 = 4A, 4B, 4C, 4E, 4G in parallel; wave 2 = 4D, 4F, 4H, 4I in parallel.
* **Gate (per package, then per wave):** <!-- amended by §28: the render line names both targets (P2) -->
  ```bash
  # tui-next is the temporary Slice 3–4 name of crates/tui (§21 item 31)
  cargo fmt --all --check
  cargo clippy -p tui-next --all-targets --all-features -- -D warnings
  cargo test -p tui-next --lib
  cargo test -p tui-next --test conformance
  cargo test -p tui-next --test render --test render_components   # both targets; --test render alone runs half the matrix (§16.3, §28)
  cargo test -p tui-next --test architecture
  cargo test -p tui-next --doc
  cargo build -p tui-next --examples
  cargo test -p tui-next --test perf --release -- --test-threads=1
  cargo run -p xtask -- doc-check                   # §21 item 34
  cargo check -p tui-next --no-default-features     # §22
  cargo +1.88.0 check --workspace --all-targets --all-features   # §22
  cargo test --all-targets                          # the legacy root package stays green (M30)
  ```
  Every component in the package must appear in `conformance_suite!` and pass all 20 applicable cases. After each package, a fresh read-only `opus-analyst` reviews API consistency against §13; the coordinator applies verified corrections before the next wave.

### Slice 5 — Showcase (one owner)

* **Files:** `apps/showcase/**` in full (`Cargo.toml`, `src/main.rs`, `src/app.rs`, `src/pages/*.rs` — all 22 — `src/data.rs`, `tests/app_tests.rs`, `tests/visual.rs`, `tests/baselines/showcase.txt`, `tests/perf.rs`).
* Deletes the shell sidebar, footer hint row, static-field renderer, button matrix, inspector panel and too-small screen in favour of library components (§18.3 #2–#7, #21). Adds the pages goal §22.1 requires: the state matrix per component, `Theme::paper()` coverage, scoped and per-instance override pages, the author-component page (example 12 rendered as a page), and deterministic navigation to every state for captures.
* Begins with the one-commit rename `tui-next` → `junie-tui` and the removal of the root package's `src/`, `src/bin/*`, `[lib]`, `default-run` and `[[bin]]`s (Slice 3 staging, §21 item 31); then adds `apps/showcase` (with its `[lib] showcase_app` + `[[bin]] showcase`, §21 item 23) to the workspace members. All 26 existing tests must pass with the §16.4 `Harness`. <!-- amended by §21 items 23, 31 -->
* **Gate:** the full §26 command set scoped to `-p junie-tui -p showcase`, plus `cargo run -p xtask -- doc-check`, `cargo check -p junie-tui --no-default-features` and the `cargo +1.88.0 check` MSRV job (§22), plus `cargo run -p showcase` driven through `tools/capture.sh` at 80×24, 100×30, 120×40, 160×50 × {truecolor, 256, 16, mono} × {junie, paper}, with every capture inspected and every baseline difference classified against §20.10.

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
  cargo check -p junie-tui --no-default-features                    # §22: the core is backend-free
  cargo +1.88.0 check --workspace --all-targets --all-features      # §22: MSRV job, blocking
  cargo test  --workspace --test architecture no_deprecated_or_legacy_api_usage dependency_graph_is_exactly_the_declared_set   # §22
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

**Edition and MSRV are unchanged**: edition 2024, `rust-version = "1.88"`, set once in `[workspace.package]` and inherited by every member. <!-- amended by §22 --> The dependency set is exactly `{ratatui-core, ratatui-crossterm, unicode-width, unicode-segmentation, bitflags}` (§22 §1.3): `ratatui-core` + `ratatui-crossterm` replace the `ratatui` umbrella crate, `bitflags` is a justified addition already present in the graph via `ratatui-core` (`StateFlags`/`Caps`), and `smallvec` is **rejected** (§22 §4.2 — `Vec` everywhere). No framework-sized dependency is added, and no unrelated version churn accompanies the refactor.

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
    lib.rs            # #![deny(missing_docs)] #![forbid(unsafe_code)] #![doc = include_str!("../README.md")]; the curated facade (§22)
    author.rs         # the component-author layer (B.4); includes the qualified-only `author::raw` escape
                      # module as an INLINE `pub mod raw` (§24 M1's exact Rust already shows it inline;
                      # there is no `author/raw.rs` file) <!-- amended by §25 §3 -->
    id.rs  event.rs  intent.rs  response.rs  keymap.rs
    secret.rs  validate.rs  field_control.rs   # foundation vocabulary consumed BY components/input.rs, not components
                      # themselves; at the crate root, not under components/ — components/ is a Slice-4 directory the
                      # Slice-3 owner may not write, and these are Slice-3 types (§25 D‑9) <!-- amended by §25 -->
    focus.rs  hit.rs  capture.rs  scroll.rs  cursor.rs
    layer.rs  runtime.rs  runtime/session.rs  diagnostics.rs      # session.rs: the only file naming raw-mode/alt-screen commands (§22 §6.2)
    layout.rs  measure.rs
    ui/{mod.rs, cx.rs, paint.rs, surface.rs, layer_buf.rs}
    text/{mod.rs, buffer.rs, editor.rs, measure.rs, fuzzy.rs, span.rs}                       # span.rs: OUR role-carrying Span (§24 M1)
    theme/{mod.rs, tokens.rs, role.rs, glyph.rs, recipe.rs, patch.rs, resolve.rs, downgrade.rs, border.rs}   # border.rs: Set/PLAIN/ROUNDED/DOUBLE re-exports + const ASCII (§24 M2)
    theme/builtin/{mod.rs, junie.rs, paper.rs}
    collection/{mod.rs, key.rs, reconcile.rs, rowui.rs, decor.rs, empty.rs}
    components/{mod.rs, button.rs, choice.rs, chip.rs, brand.rs, keyhint.rs, too_small.rs,
                field.rs, input.rs, textarea.rs, select.rs,
                list.rs, tree.rs, props.rs, steps.rs, nav_list.rs, tabs.rs,
                panel.rs, split.rs, scroll_region.rs, viewport.rs,
                dialog.rs, menu.rs, picker.rs, filter_list.rs, completion.rs,
                form.rs, wizard.rs, picker_chain.rs, help.rs,
                status.rs, hintbar.rs, progress.rs, meter.rs, code.rs, diff.rs, grid.rs}
  examples/           # 01_button.rs … 13_connection_form.rs  (external-style consumers; 13 added by §23)
  tests/
    conformance.rs  render.rs  architecture.rs  perf.rs  perf_baseline.txt
    baselines/components.txt
    fixtures/{grid_model.rs, rows.rs, text.rs}
    ui/*.rs           # trybuild compile-fail cases (§21 item 28)
    allow/{domain.txt, legacy_api.txt}   # architecture allow-lists, both empty (§16.5, §22 §6.2)

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

Root `Cargo.toml` (virtual; exact, §22 §1.3) <!-- amended by §22 -->:

```toml
[workspace]
resolver = "3"                      # explicit: a virtual manifest has no edition to imply it
members         = ["crates/tui", "crates/tui-testing", "apps/showcase", "apps/tablepro", "apps/jackin-preview", "xtask"]
default-members = ["crates/tui", "crates/tui-testing", "apps/showcase", "apps/tablepro", "apps/jackin-preview"]   # `cargo build` never compiles xtask

[workspace.package]
version      = "0.1.0"
edition      = "2024"
rust-version = "1.88"
license      = "MIT"
repository   = "…"

[workspace.dependencies]
junie-tui         = { path = "crates/tui" }
junie-tui-testing = { path = "crates/tui-testing" }
ratatui-core      = { version = "0.1.2", default-features = false, features = ["std", "underline-color"] }
ratatui-crossterm = { version = "0.1.2" }          # default = crossterm_0_29 + underline-color
unicode-width        = "0.2.2"
unicode-segmentation = "1.13"
bitflags             = "2.13"

[profile.release]
lto = "thin"
codegen-units = 1

[workspace.lints.rust]
unsafe_code                   = "deny"      # crates/tui raises to forbid at the crate root
missing_docs                  = "warn"      # crates/tui raises to deny at the crate root
unreachable_pub               = "warn"
missing_debug_implementations = "warn"
unused_qualifications         = "warn"
rust_2018_idioms              = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all      = { level = "deny",  priority = -1 }   # correctness + suspicious + style + complexity + perf
pedantic = { level = "warn",  priority = -1 }

# denied individually — each maps to a stated invariant, not to taste
panic                     = "deny"   # goal §10 "no panics during normal interaction"
indexing_slicing          = "deny"   # the structural fix for §1.3's ragged-row panics
unwrap_used               = "deny"
expect_used               = "deny"
todo                      = "deny"   # goal §29 "no material TODO / stub"
unimplemented             = "deny"
dbg_macro                 = "deny"
print_stdout              = "deny"   # a TUI library must never write to stdout
print_stderr              = "deny"
undocumented_unsafe_blocks= "deny"
mem_forget                = "deny"

# warned, not denied — real signal, too noisy to block on
arithmetic_side_effects   = "warn"   # the rule is saturating_* (R5); deny would flood
missing_panics_doc        = "warn"
missing_errors_doc        = "warn"

# allowed with reasons
must_use_candidate        = "allow"  # noise; #[must_use] is applied deliberately (Response)
module_name_repetitions   = "allow"
cast_possible_truncation  = "allow"  # u16 terminal coordinates — ratatui itself allows these four
cast_sign_loss            = "allow"  #   (ratatui-core-0.1.2/Cargo.toml:179-182); staying
cast_precision_loss       = "allow"  #   consistent with the ecosystem we live inside
cast_possible_wrap        = "allow"
```

`crates/tui/Cargo.toml` (§22 §1.3):

```toml
[package]
name = "junie-tui"
version.workspace = true; edition.workspace = true; rust-version.workspace = true
license.workspace = true; repository.workspace = true

[lints]
workspace = true

[features]
default   = ["crossterm"]
# <!-- amended by §25 (adjudication 1, D‑1) --> `[]`, NOT `["dep:ratatui-crossterm"]`. The feature gates the
# terminal SESSION — `TerminalSession`, `run`, `DefaultTerminal` — and nothing else. `ratatui-crossterm` is a
# normal, non-optional dependency taken for its version-unified `crossterm::event` vocabulary (`KeyCode`,
# `KeyModifiers`, `Input::from_crossterm`), which is unconditional; making it optional would make the shape of
# `Intent`/`Chord`/`KeyMap` depend on a feature. `CrosstermBackend` is confined to `runtime/session.rs` by
# forbidden-pattern rule 27 and `architecture::ratatui_crossterm_is_named_in_exactly_two_files`.
crossterm = []                          # gates TerminalSession, run(), DefaultTerminal
testing   = []                          # Runtime/Ui inspection surface (§17.0 A1, A2)

[dependencies]
ratatui-core.workspace = true
ratatui-crossterm.workspace = true      # normal and NON-optional (§25 D‑1); never for `CrosstermBackend`
unicode-width.workspace = true
unicode-segmentation.workspace = true
bitflags.workspace = true

[dev-dependencies]
junie-tui-testing.workspace = true      # dev-only cycle, permitted by Cargo; `testing` never leaks into `cargo build -p showcase`
trybuild = "1"                          # §16.1 must_use_is_enforced, secret::is_not_clone_not_eq, bitor_is_defined_only_for_unit, grid::read_only_update_takes_a_shared_model
```

`crates/tui-testing/Cargo.toml` (§22 §1.3):

```toml
[package]
name = "junie-tui-testing"
publish = false
version.workspace = true; edition.workspace = true; rust-version.workspace = true

[lints]
workspace = true

[dependencies]
junie-tui   = { workspace = true, features = ["testing"] }
ratatui-core.workspace = true           # TestBackend, Terminal, Buffer
bitflags.workspace = true               # conformance Caps (§16.2)
```

`xtask/Cargo.toml`: `syn` (features `full`, `visit`, `parsing`), `cargo_metadata`, `walkdir`, `regex` — versions pinned at implementation time; nothing depends on `xtask`, so none of these reach the library or the apps.

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

[lints]
workspace = true

[dependencies]
junie-tui.workspace = true              # the ONLY normal dependency (§22 §1.2); no `ratatui*` line

[dev-dependencies]
junie-tui-testing.workspace = true
```

`jackin-preview` sets `[[bin]] name = "jackin-preview"` with `path = "src/main.rs"`, preserving the hyphenated binary while the package directory stays readable. Its lib is `jackin_app`; tablepro's is `tablepro_app`.

### B.3 `pub` vs `pub(crate)` policy

1. **Two documented layers, one crate.** `junie_tui::*` is the application-author surface; `junie_tui::author::*` is the component-author surface. Both are `pub` and separately documented with a module-level rustdoc header stating who the audience is (§13). Nothing else is `pub`.
2. **`lib.rs` is a curated facade, not a `pub mod` list.** Every module is `pub(crate) mod`; `lib.rs` re-exports the named items. Adding a type to the public API is a deliberate line in `lib.rs`, reviewable in a diff. `pub use` globs are forbidden. <!-- amended by §25 §4(a), MI‑11, F21 --> Exactly **two** modules are `pub` by decision, and the rule is otherwise absolute: `pub mod theme` (sanctioned by B.4 — `theme::border`, §24 M2) and `pub mod layout` (required by §17 example 1's `use junie_tui::{…, layout, …}`). **`text` is `pub(crate) mod`** with curated re-exports: a `pub mod text` leaks `TextBuffer`, `grapheme_width`, `is_word_char` and `thousands`, none of which appears in B.3 item 4's or B.4's lists. Likewise `layout`'s public surface is the named functions plus `Track`/`RowAlign`, including `distribute_into`; nothing else.
3. **No public fields on component types** (invariant S1). Public fields exist only on plain data records with no behaviour and no geometry: `LayerSpec`, `Dismiss`, `StylePatch`, `StateRule`, `PartRecipe`, `ColorTokens`, `DesignTokens` and their sub-structs, `Role`/`GlyphRole`/`Part`/`Family`/`Variant` newtypes' constants, `FocusEntry`, `Headroom`, `Insets`, `Size`, `Constraints`, `RowDecor`, `CellDecor`, `Binding`, `Hint`, `HintLayer`, `LayoutFacts`, `Capture`, `PartRef`, `Key`, `Chord`, `Mouse`, `FieldError`. `architecture::no_public_geometry_or_cache` enforces the geometry half.
4. **`#[non_exhaustive]`** — applied narrowly <!-- amended by §21 item 8; §22 (MOD §3.2) -->: on `LayerSpec` and `LayoutFacts` (constructed through builders or by the runtime) **and** on the enums the runtime produces and the caller only matches, which will grow: `Diagnostic`, `Intent`, `Phase`, `FocusVia`, `LayerEvent`, `DismissReason`, `Status`, `ColorLevel`, `RegionKind`. Trade-off stated out loud: `#[non_exhaustive]` on an enum forces a wildcard arm downstream and *destroys* the "adding a variant is a compile error" property, so it is applied only where downstream exhaustiveness is not wanted; `StylePatch`, `RowDecor`, `CellDecor`, `Insets`, `Headroom` are constructed by users with struct literals and are **not** marked. `ColorTokens`, `DesignTokens`, `RowDecor` and `CellDecor` are **not** `#[non_exhaustive]`: example 2 builds a full `ColorTokens` literal from another crate, adapters build `RowDecor { marker: …, ..Default::default() }`, and §11.4 *wants* a new token to be a compile error. Recorded: adding a colour or design token is an intentional breaking change for downstream themes; `map_colors`'s exhaustive destructure is the mechanism.
5. **No `#[doc(hidden)]` public items.** If something must be reachable it is documented in `author`; if it must not, it is `pub(crate)`.
6. **`#![deny(missing_docs)]`, `#![forbid(unsafe_code)]` and `#![doc = include_str!("../README.md")]`** at the top of `crates/tui/src/lib.rs` <!-- amended by §22 -->. `[workspace.lints.rust] unsafe_code = "deny"` is the workspace level; `crates/tui` and every app root tighten it to `forbid`; **`crates/tui-testing` stays at `deny`** (`forbid` cannot be overridden) because it carries the single documented `unsafe impl GlobalAlloc` under a local `#[expect(unsafe_code, reason = "counting allocator; see SAFETY")]` with a `// SAFETY:` comment, covered by `debug_and_release_alloc_counts_match`. `#[allow]` is never used; every suppression is `#[expect(lint, reason = "…")]`. No process-global state (`LazyLock`, `OnceLock`, `static mut`, `thread_local!`) exists in `crates/tui/src` — the `Runtime` owns all state.
7. **Applications export a test surface only.** Each app is a package with a `[lib]` (`showcase_app`, `tablepro_app`, `jackin_app`; `publish = false`) and a thin `[[bin]]` whose `main` calls `<app>::run()`. <!-- amended by §21 item 23 --> Integration tests in `tests/` link the lib and reach the app through its `pub` items — the app's `const Id`s, its `App` type and its screen enums; nothing else is `pub`. A binary-only package cannot host `tests/*.rs` (they link the library target), which is why the earlier "no `[lib]`" rule is struck. `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library` guards the boundary. This is the migration contract of §16.4 item 3.

### B.4 The `author` module

<!-- amended by §21 items 18, 19, 21, 22, 31; §22; §24 M1 --> `junie_tui::author` is a re-export module, not a second implementation. (During Slices 3–4 the path reads `tui_next::author`, §21 item 31.) It is what example 12 and every downstream component author consumes, and it is the mechanical proof of Scenario G: if a component can be written with it, no private access is needed.

**The root facade's foreign re-exports (exact, §24 M1).** MOD §1.2's rule is *"`junie-tui` re-exports the ratatui types the public API mentions"*; applying the rule rather than its pre-§22.10 parenthetical list, every foreign type a `pub` signature names is re-exported at the layer that names it, and nothing else is. `ratatui_core::layout::Size` and `ratatui_core::text::{Line, Span, Text}` are named by **no** `pub` signature (`Frame::area()` returns `Rect`; resize is `Input::Resize{w,h}`; `Backend::size`/`WindowSize` never leave `runtime/session.rs`; `RowUi::label_spans` builds its `Line` inside `ui/paint.rs`), so our `Size` and our `Span` keep their names and the ratatui ones are not exported at the root. `Frame` is named only by `Runtime::draw` (A1) — a host concern — so it is **root only**. `architecture::every_foreign_type_in_the_public_surface_is_re_exported` (§16.5) keeps the list complete mechanically.

```rust
// crates/tui/src/lib.rs — the application-author facade (B.3 item 2: one curated line each)
pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Modifier, Style};
pub use ratatui_core::terminal::Frame;          // named by Runtime::draw (A1) — host concern, root only
pub use crate::text::Span;                      // OURS: role-carrying, used by RowUi::label_spans / Ui::paint_spans
pub mod theme;                                  // theme::border (M2), ColorTokens, Theme, …
// …plus every library type the root surface names (components, Form vocabulary, Runtime, run, App, …; §23.1)
```

```rust
//! Component-author API. Everything needed to build a component that participates in
//! theme resolution, focus, hover, press, dispatch, hit testing, cursor output,
//! scrolling, overlays, capture, testing and visual capture — and nothing more.
pub mod author {
    // identity and parts — the NAMED items, never `pub use crate::id::*` or the module itself
    // (re-exporting the module widens the surface unintentionally; the `id!` macro is `#[macro_export]`
    // and is already reachable at the root) <!-- amended by §25 §4(a), F21 -->
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
    pub use crate::layer::{LayerId, LayerKind, LayerSpec, LayerSize, Anchor, Side, CrossAlign,
                           Dismiss, Backdrop, LayerEvent, DismissReason};   // LayerSize added (§26 N1)
    // theme resolution
    pub use crate::theme::{Theme, Family, Variant, Role, FgStep, SyntaxRole, MeterRole,
                           GlyphRole, Surface, StylePatch, Slot, StateRule, Overlay,
                           Resolved, PartMetrics, Modifier, Density, ColorLevel, DesignTokens,
                           Align, ScreenAlign};   // PartMetrics added (§26 N2)
    // layout and measurement
    pub use crate::layout::{self, Track, RowAlign, Insets, SplitModel};
    pub use crate::measure::{Measure, Size, Constraints};
    pub use crate::ui::LayoutFacts;
    // text
    // `text` is `pub(crate) mod`; these named items ARE its public surface (B.3 item 2)
    pub use crate::text::{TextEditorCore, EditAction, EditOutcome, CursorPos,
                          width, wrapped_rows, wrap, fuzzy, truncate, truncate_middle};   // wrapped_rows added (§26 N1)
    // collections
    pub use crate::collection::{RowUi, CellUi, ColumnsUi, RowDecor, CellDecor, ByIndex, DefaultRow, KeyFn, RowFn,
                                EmptyState, RowTotal, Reconciliation, Reconcile, SelectMode, KeySet, Status};
    // bindings and hints
    pub use crate::keymap::{Binding, Bindings, BindingState, KeyMap, KeyPhase, Hint, HintLayer};
    // errors and diagnostics
    pub use crate::{FieldError, Validate, NoValidate, Secret, SecretPolicy, FieldControl};   // LayoutError deleted (§21 item 19)
    pub use crate::diagnostics::Diagnostic;
    // ratatui-core types a painter needs (§22: `ratatui_core::` paths, never the umbrella crate)
    pub use ratatui_core::layout::{Rect, Position};
    pub use ratatui_core::style::{Color, Style, Modifier};
    pub use ratatui_core::buffer::{Buffer, Cell};
    pub use crate::theme::border;                 // symbols::border::{Set, PLAIN, ROUNDED, DOUBLE} + our ASCII (§17.0 A5, §24 M2)
    // <!-- amended by §24 M1 --> our role-carrying span; same type as the root `Span`. `Frame` is deliberately ABSENT:
    // a component author receives `Ui`, never a `Frame` (the same reason `Runtime`/`run` are absent).
    pub use crate::text::Span;
    /// Types needed only to drive the `Ui::raw()` / `RowUi::raw()` escape hatch (`Buffer::set_line` / `set_span`).
    /// The ONLY re-export not forced by a signature. `raw::Span` is ratatui's style-carrying span and is written
    /// qualified, always: `raw::Span` — never flat-imported beside ours. If `raw::Span` starts appearing in
    /// components, `Ui::paint_spans` is under-specified; that is the signal, not a naming problem (§24 M1 risk 2).
    // <!-- amended by §25 §3 --> INLINE, exactly as §24.1's exact Rust writes it; there is no `author/raw.rs`.
    pub mod raw { pub use ratatui_core::text::{Line, Span, Text}; }
}
```

What is deliberately **not** in `author`: `Runtime`, `run`, `TerminalSession`, `Registry`, `FocusRing`, `FocusState`, `App`, `Frame` <!-- amended by §24 -->, and the concrete components. A component author drives none of those; an application author reaches `Runtime`/`run`/`App` from the root facade, and tests reach `FocusRing` through `Harness::ring()`. `architecture::conformance_covers_every_public_component` plus example 12 compiling with `use junie_tui::author::*;` and **no other `junie_tui` path** is the standing proof that the split is honest.

### B.5 Examples and capture tooling

* The thirteen §17 examples live in `crates/tui/examples/` and are built by `cargo build -p junie-tui --examples` in every slice gate. Because Cargo compiles examples as separate crates linked against `junie_tui`, they see exactly the public API and nothing else — the "external-style consumer" requirement of goal §21 is satisfied structurally, not by convention.
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
// every collection: List, Tabs, Picker, Tree, NavList, Steps, ChipBar, RadioGroup, Select, Completion, FilterList, PropsList
// <!-- amended by §24 M3: clarified, not changed --> `Select` was always in this class; a form field holds its default
// instantiation and passes `FormData::options` as the per-phase items, so the rule holds at every form field too.
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

Open for `opus-analyst` before Slice 4I (not decided here): whether `Grid::update` takes `M: GridEditor` with defaulted refusals on `GridEditor`, or two entry points (`update` / `update_editable`). Slice 3 does not depend on the answer. <!-- amended by §23: RESOLVED by Adjudication K2 — two entry points; the `update<M: GridModel>(…, &mut M)` line above is superseded by §12.3 -->

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

The constructors `consumed()` / `changed()` stay (`changed` keeps its `Outcome`-era name so the ~60 assertion rewrites are mechanical). §16.4's `Outcome` mapping is three-way: `Outcome::Changed → .is_changed()`; `Outcome::Consumed → .is_consumed() && !.is_changed()`; `Outcome::Ignored → !.is_consumed()`. `BitOr` / `BitOrAssign` are implemented for `Response<()>` only — `flow`: Consumed wins; `invalidate`: max; `id`: <!-- amended by §25 §4(c) --> `lhs.id.or(rhs.id)` (the first `Some`, so an `ignored()` on the left never erases the right's id — a benign improvement on the "id: lhs" written here, recorded because §6.1 now states it); `state`: lhs, documented as "the fold is a control-flow summary; read `state`/`id` from the individual responses". Composing two action-carrying responses is a type error, never silent loss; every §17 `r |=` operand is already `Response<()>`. `Response.id` is `Option<Id>` with `pub fn id(&self) -> Option<Id>` (`ignored()` has no id). §16.1: `response::bitor_keeps_the_first_action` (it asserted the rejected semantics) is replaced by `response::bitor_is_defined_only_for_unit` (compile-fail via `trybuild`).

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

The queue is built in step 6 and immutable for the whole of step 7; "drained" is recorded through a `Cell<bool>` per bucket, so `for it in cx.intents(id) { … cx.close_layer(…) … }` compiles (`Dialog`, `SplitPane`, `List`, `Select` all touch `cx` inside the loop). `Intent::Paste(&'f str)` therefore requires the paste text to live in a runtime-owned frame arena for the whole of step 7, not in the `Input` value (§3.3 step 1). The name is unified to `Cx::intents`; `Intents::take(id)` and the type name `Intents` are struck. B14: `Cx::intents` returns an empty iterator without probing when the queue is empty (a single `bool` check) and, when non-empty, probes a `[u64; N]` open-addressed table of the ≤ 8 owners that actually have intents; a frame with no input performs zero probes. §16.6's `intents_drain_scales_with_intents_not_components` (false by construction — 500 components made 500 probes) is renamed `intents_drain_is_o_1_when_the_queue_is_empty` with the threshold *a 500-component frame with 0 intents costs the same as a 20-component frame with 0 intents ±10 %; with 2 intents, ~~total probe cost is ≤ 500 × 5 ns~~ and allocations are 0*. <!-- amended by §27 (Adjudication O4b) --> The struck clause is a tautology (2.5 µs against a measured 632 ns for the whole 500-control `handle`), and the ±10 % wall-clock band was already replaced by §25.6; the binding form is the differential probe count (**480**) plus the normalised per-control ratio, both in §16.6.

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
    pub const fn size(self, s: LayerSize) -> Self;                  // <!-- amended by §26 --> was `min_size(w, h)`; `LayerSize` is declared in §9.1
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

Appended to step 7: *A re-run enqueues only `Intent::FocusOut{to}` and `Intent::FocusIn{via}`; already-drained buckets are never refilled, so no input intent is delivered twice. The `Response` of each pass is folded into the first with `|`. If a 5th pass is required, the runtime emits `Diagnostic::FocusTransitionDidNotSettle`, applies the pending `FocusOut` **and** the matching `FocusIn` to the last requested target without re-running `app.update`, and continues.* <!-- amended by §25 MI‑7 --> "Applies" means **delivered**: the give-up path must not enqueue the pair and then `intents.clear()` in the same `handle`, which is what the implementation did — the pair is re-staged through `pending_focus` and delivered on the next `handle`, and `runtime::a_fifth_focus_pass_is_diagnosed_and_applied` asserts the two intents arrive, not merely that `focus().is_some()`. `conformance::focus_transition_settles` is a suite-level test (it already sits in §16.2's suite-level list; it is never emitted per component) and asserts the diagnostic count is 0.

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

**Item 20 — M7: `ScreenAlign` versus `Align`; `LayerSpec.min_size`.** `pub enum ScreenAlign { Center, UpperThird, Bottom }` is the `Anchor::Screen` payload; `pub enum Align { Left, Center, Right }` is text alignment (`StylePatch.align`, `CellUi::align`, `Column.align`). ~~`LayerSpec.min_size` is `(u16, u16)`, not `Size` (a min-size whose type is a min/preferred pair).~~ <!-- amended by §26 (Adjudication N1) --> **Struck.** `LayerSpec.size` is `LayerSize`, a two-variant `#[non_exhaustive]` enum (`Fill`, `Fixed(u16, u16)`), declared in §9.1. The `(u16, u16)` field with its `(0, 0)` ⇒ whole-screen sentinel is gone, and so is the name: it was **never** a minimum — `resolve_anchor` clamps to the screen and `Rect::clamp`s, so the field was always a maximum, and every reader who trusted the name would size dialogs wrongly. `Fixed(0, _)` / `Fixed(_, 0)` resolves to `Rect::ZERO`. The builder is `.size(LayerSize)`, not `.min_size(w, h)`. Retained from the original item: `ScreenAlign` versus `Align` is unchanged, and a layer's size is still not a `Size` (a min/preferred pair has no meaning for a layer, which is clamped and never grown).

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

`ThemeBuilder::build` derivation (written into §11.2): given `surfaces[0]` and `accent`: `surfaces[1..4]` step L\* by +4 (dark base) or −4 (light base, detected by L\*(surfaces[0]) > 50); `fg[0..4]` step L\* by −18 from a contrast-7:1 anchor against `surfaces[0]`; `accent_hover = ΔL* +8`, `accent_pressed = ΔL* −8`, `accent_tint` = accent at 12 % over `surfaces[1]`; `focus = accent`, `focus_ring = accent_pressed`; `border_subtle = surfaces[3]`, `border_strong = fg[3]`; `danger/warning/success/info` tints at 12 %; `on_accent`/`on_danger` = whichever of `fg[0]`/`surfaces[0]` reaches ≥ 4.5:1. Every derived value is a pure function of the seeds. `Theme::paper()` is `from_tokens(seeds).builder()…build()` and is pinned by `theme::paper_tokens_are_pinned`. `downgrade_color` (§11.4): `nearest_256` = nearest in the 6×6×6 cube ∪ 24-step greyscale by squared sRGB distance, ties to the lower index; `nearest_16` = <!-- amended by §25 (adjudication 3, D‑3 REJECTED) --> ~~nearest of the 16 xterm defaults by CIE76 ΔE~~ **struck**; it is the legacy **categorical** metric: channel spread (max−min) under 40 collapses to the grey ladder by ITU‑R BT.601 luma (≤30 `Black`, ≤110 `DarkGray`, ≤200 `Gray`, else `White`); otherwise the dominant channel selects the hue family (`r ≥ g,b ∧ g > 120 ∧ b < 80` reads as `Yellow`) and `max(r,g,b) > 180` selects the light half. Exact code in §25.3. `DESIGN.md:320` fixes the outcome (accent `LightGreen`, error `LightRed`) and the authority order (line 5) puts `DESIGN.md` and the existing rendered output above this document, so CIE76 was a regression, not a decision; `mono`: Y = 0.2126R + 0.7152G + 0.0722B, then Y < 0.35 → black, Y > 0.75 → white, else `Color::Reset`. Text corpus `crates/tui/tests/fixtures/text.rs`: ≥ 200 strings covering ASCII, CJK wide, combining marks, ZWJ emoji, RTL, and widths 0..=120. New tests: `theme::builder_derives_every_unset_token_deterministically`, `theme::derived_tokens_meet_design_contrast_ratios`, `theme::downgrade_is_deterministic_per_level`, `theme::paper_tokens_are_pinned`, `render::overrides::global_variant_override_changes_only_that_variant` (goal §15 scenario 5 had no test), `text::row_ui_matches_fit_for_every_fixture` (named in §20.10, now also in §16.1), `response::layout_is_strictly_greater_than_paint` (the only assertion on `Invalidate::Layout`, so no builder invents layout caching early).

**Item 30 — P1, P3–P8, the §20.9 restatements, and the remaining review §3/§8 items.** Amends §13, §13.1, §15, §20.2, §20.4, §20.9, §16.6, §5, §9.1, §12.3, §16.3.

* P1: `HintLayer { hints: Vec<Hint>, badge: Option<&'static str>, status: Option<Cow<'static, str>>, centered: bool }` (`Vec`, not `SmallVec` — §22 <!-- amended by §22 -->), cached in `Ui::cache` behind `(focus_id, StateFlags, top_layer)`. New perf test `frame_hintbar_derived` — **0 allocs/frame when focus is unchanged**.
* P3: `Ui` keeps a running `stack_hash: u64` updated on `with_overlay` push/pop (§20.9-2). P4: `Ui` is constructed once per `Runtime`/`Scene` and reused, never per frame (§20.9-2).
* P5: `Secret::masked_tail(&self, n) -> String` is replaced by `fn write_mask(&self, out: &mut CellUi<'_>, n: usize)`; the synthetic tail may be cached in `TextInputState` at `begin`.
* P6: `Runtime` keeps at most 64 `Diagnostic`s plus a dropped count, cleared at the start of each `handle`.
* P7: `Id` is ~48 B in debug (`DebugLabel`) versus 8 B in release, so every `Region` roughly doubles in debug; recorded in §20.4.
* P8: `frame_showcase_lists_120x40`'s "hits within ±10 %" becomes *hits recorded and classified in `docs/visual-changes.md`; no unexplained growth > 25 %* (`Field` chrome, `NavList`, `scroll_region` parts and disabled-but-registered entries change region counts materially).
* §20.9-7/-8/-9 are restated on `Ui::cache` (item 2); §20.9-11 commits to pre-formatting at load — `apps/tablepro/src/grid_model.rs` stores `ResultSet { text: Vec<String>, kind: Vec<CellKind>, … }` produced once (6 000 strings for 500×12, within `< 8 000`), `CellRef<'a> { text: &'a str, tone: Option<Role>, align: Align }`, `CellValue` survives only in the domain model, and the `CellValue::display_width` clause is deleted; §20.9-12 restated (item 6); §20.9-5 amended (item 22).
* Review §3, inline editors: a `Grid` cell's inline editor registers a `Control` region **after** the grid's cell `Part` region and therefore wins the click; the grid must not treat a click inside an active edit as a cursor move. New test `grid::click_inside_an_active_inline_edit_goes_to_the_editor`.
* Review §3, binding convention added to §13: *A component instance with any configuration beyond `new(id, …)` is built by exactly one private constructor function on the owning screen, called from both phases. The constructor takes the fields it needs as parameters, never `&self`, so `update` can still pass `&mut` to disjoint fields; a controlled `.value(&T)` added in `draw` is the documented per-phase difference.* `architecture::props_are_built_once` (a `syn` check that no `X::new(` appears more than once per screen module for the same `const Id`, ignoring unconfigured constructions) reports violations. `Form` (J2) declares each field once so a 15-field form drives both phases from one constructor — the API sketch was the open research item at the end of this section and is now §15.1 (`FieldSpec::new(id, …)`, Adjudication K1, §23). <!-- amended by §23 -->
* A4 (§5): *a component that declares a part must paint `Resolved.glyph` when `Some`; `conformance::registry::declared_parts_are_the_parts_actually_styled` checks the query, `mono_states_are_distinguishable` checks the paint.* A5: `#[cfg(feature = "testing")] Ui::styled_parts(&self) -> &[(Id, Part)]` <!-- amended by §25 F7 --> widened to `&[(Id, Family, Variant, Part, Resolved)]`, so `Runtime::resolved(id, part)` returns the resolution the component actually recorded instead of a hard-coded `Family::BUTTON` (`resolved_in(f, v, id, p)` stays as the explicit escape hatch). Written by the painting queries only — `Ui::resolve`, `Theme::resolve` and `Theme::metrics` record nothing (invariant M1, §26), or a part a component merely *measured* would be reported as styled. A8: `EditIntent::External` — *the component emits `EditRequested(item, col)` and does not begin an inline edit; the application opens its own editor.* A10 (§9.1): *`Dismiss.focus_out` is honoured only for `Popover` and `Tooltip`; a `Modal` traps focus so it can never fire.* A11: `.state_override(StateFlags)` is declared on component builders as a documented showcase/testing-only path with `architecture::state_override_is_used_only_in_apps_and_fixtures`. A14 (§16.3): the order is change → capture → classify → bless; `xtask bless-guard` runs in CI on the committed tree, not locally.

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

`cargo run -p xtask -- doc-check` extracts every `` `Ident::method` `` and every fenced `rust` block from `COMPONENT_ARCHITECTURE.md` §3–§17 and §21–§23 <!-- amended by §23 --> and asserts each resolves against the compiled library's rustdoc-json, or is on an explicit "not yet built (Slice 3/4)" allow-list that the check prints. This converts the §17.0-versus-§3–§15 drift the review found into a permanent CI gate.

### Formerly "Not applied" — resolved by §23

The two items this section left open — the `Form` API sketch (review §3; `Form::field(id, …)` was named but no API) and item 1's `Grid::update` bound question — are decided by **Adjudication K (§23)**: K1 is recorded as §15.1 with §17.0 A10 and §17 example 13; K2 amends §12.3, §17.0 A3/A7 and §16.1. Nothing else in §21 is reopened.

---

## 22. Adjudication L — Modern API and dependency policy

**Status:** Accepted. Source: `docs/audit/modern-api-audit.md` (MOD; every **[F]** in it was read from the unpacked registry sources of `ratatui-0.30.2`, `ratatui-core-0.1.2`, `ratatui-widgets-0.3.2`, `ratatui-crossterm-0.1.2`, `ratatui-macros-0.7.2`, `crossterm-0.29.0`, `unicode-width-0.2.2`). The audit is **binding in full**; this section records its decisions in the document's own vocabulary and lists the sections it amends. Each inline edit carries `<!-- amended by §22 -->`. Nothing here reopens Adjudications A–K; where an earlier decision named a ratatui path or a container type, this section fixes the exact modern form of it.

**Amends:** §5 R3, §6.1, §9.1, §10, §11.2, §11.3, §11.7, §12.2, §13, §13.1, §16.1, §16.5, §16.6, §17.0 A1/A2/A5/A8/A9, §17 examples 2, 3, 5, §18.1, §18.2, §19, §20.10 (item 16), Appendix A gates, Appendix B.1–B.4, §21 item 30 (P1).

### 22.1 Crate choice — `ratatui-core`, not `ratatui` (MOD §1)

**Decision.** `crates/tui` (`junie-tui`) depends on **`ratatui-core`** for everything paint/theme/layout/text, and on **`ratatui-crossterm`** behind a default-on `crossterm` feature for the input boundary and the terminal session. It **never** depends on `ratatui`, `ratatui-widgets`, or `ratatui-macros`. **The applications depend on `junie-tui` alone.**

**[F]** Upstream's own guidance: *"Widget libraries should generally depend on `ratatui-core`, benefiting from a stable API and reducing the need for frequent updates"* (`ratatui-core-0.1.2/src/lib.rs:16-18`). **[F]** `ratatui-core` 0.1.2 exports `buffer::{Buffer, Cell, CellWidth, …}`, `layout::{Rect, Position, Size, Margin, Layout, Constraint, Flex, …}`, `style::{Color, Style, Modifier, Stylize, …}`, `symbols::{border, line, scrollbar, …}`, `terminal::{Terminal, Frame, …}`, `text::{Line, Span, Text, Masked, …}`, `widgets::{Widget, StatefulWidget}` only, and `backend::{Backend, TestBackend}` (unconditional). **[F]** `Block`, `Padding`, `Clear`, `Fill`, `Paragraph`, `List`, `Table`, `Tabs`, `Scrollbar*`, `Shadow`, `Dimmed` live in `ratatui-widgets`; `init`/`try_init`/`restore`/`run`/`DefaultTerminal` live in `ratatui` only (`ratatui-0.30.2/src/init.rs`); `CrosstermBackend` and the version-unified `crossterm` re-export live in `ratatui-crossterm`. **[F]** `ratatui`'s defaults (`all-widgets`, `crossterm`, `layout-cache`, `macros`, `underline-color`) force `ratatui-widgets`, `ratatui-macros`, `layout-cache` + `critical-section` into the graph; `ratatui-core` is `default = []`, `#![no_std]`; `ratatui-crossterm` defaults to `crossterm_0_29` + `underline-color` and does not turn on `ratatui-core/std`. All three declare `edition = "2024"`, `rust-version = "1.88.0"`.

Consequences recorded:

* `architecture::applications_depend_only_on_the_library_facade` (§16.5) becomes a one-line `cargo tree -p showcase -e normal --depth 1` assertion, because `junie-tui` re-exports the handful of ratatui types the public API mentions — `Rect`, `Position`, `Size`, `Buffer`, `Frame`, `Color`, `Modifier`, `Style`, `Line`, `Span`, `Text` — under `junie_tui::` and `junie_tui::author::` (see open item 1 below for the two names that collide with the document's own types).
* §17 example 2's `use ratatui::style::Color::Rgb as rgb;` is rewritten to `junie_tui::Color::from_u32(…)` (applied; example 3 likewise), otherwise the boundary claim is false at the first example.
* <!-- amended by §25 (adjudication 1, D‑1) --> `ratatui-crossterm` is a **normal, non-optional** dependency of `junie-tui`, taken for its version-unified `crossterm` re-export — the key vocabulary `Key`/`Chord`/`KeyMap` name (§6.1, R‑14) — never for `CrosstermBackend`. The `crossterm` feature gates the *terminal session* (`TerminalSession`, `run`, `DefaultTerminal`) and nothing else; `crossterm = []` is therefore the correct manifest form, **not** `crossterm = ["dep:ratatui-crossterm"]`. `cargo check -p junie-tui --no-default-features` remains a gate: it proves that nothing outside `runtime/session.rs` needs a backend. The **stronger** claim — that the widget layer is backend-independent — is proved by forbidden-pattern rule 27 (`CrosstermBackend` only in `crates/tui/src/runtime/session.rs`) and by `architecture::ratatui_crossterm_is_named_in_exactly_two_files` (`src/event.rs` and `src/runtime/session.rs`, nowhere else). Rejected alternatives, restated: owning `KeyCode`/`KeyModifiers` (≈40 hand-written variants plus `From` impls, and it loses crossterm's `PartialEq`/`Hash` ASCII-case normalisation that `Chord: Eq + Hash` relies on) and gating `Intent::Key` (it would make the shape of `Intent`, `Chord`, `Binding`, `KeyMap` and `BindingState` depend on a feature — a core whose intent enum changes shape under `--no-default-features` is not a core). It is a CI gate (§16.5, Appendix A).
* **Key vocabulary.** `Key`, `Chord`, `KeyMap` keep `crossterm::event::{KeyCode, KeyModifiers}` (§6.1), obtained **only** through `ratatui_crossterm::crossterm`, never through a second direct `crossterm = "0.29"` line — the re-export exists precisely to prevent version skew. Rejected: our own `KeyCode`/`KeyModifiers` enums (~40 hand-written variants plus `From` impls to replace well-understood code, goal §21).
* **The one real cost:** `ratatui::init`/`restore`/`run`/`DefaultTerminal` are unavailable. `TerminalSession` is kept and made a faithful mirror of `try_init` plus our two extra modes (§22.2, item 7).

**Exact manifests** are recorded verbatim in Appendix B.2 (root, `crates/tui`, `crates/tui-testing`, `apps/*`, `xtask`). Points that are easy to get wrong: `resolver = "3"` is explicit because a virtual manifest has no edition to imply it; `default-members` excludes `xtask` so `cargo build` never compiles it; the `junie-tui` ⇄ `junie-tui-testing` cycle is a **dev-dependency** cycle, which Cargo permits, and under resolver 3 the `testing` feature it enables does not leak into `cargo build -p showcase` — this is what keeps `Runtime::hover()`/`state_of()`/`resolved()` out of shipped binaries.

**Feature flags (binding).**

| Feature | Decision | Reason |
|---|---|---|
| `ratatui-core/std` | **on — mandatory** | `default = []` and `#![no_std]`. We need `HashMap` (`FocusState.restore`), `Instant`, the panic hook, and `Terminal`'s cursor-restore `eprintln` path. |
| `ratatui-core/underline-color` | **on — mandatory** | `StylePatch.underline: Slot<Role>` (§11.3) is inert without it: `Style::underline_color` and the backend's `SetUnderlineColor` are both feature-gated. Must be on for **both** core and crossterm or the backend silently drops the colour; `ratatui-crossterm`'s default already includes it and forwards to core. |
| `ratatui-crossterm/crossterm_0_29` | **on (default)** | Highest supported crossterm; selects the re-exported crate. |
| `ratatui-core/layout-cache` | **off** | Caches `Layout::split` behind `critical-section` + an LRU. §10 uses no constraint solver, so there is nothing to cache; a process-global LRU perturbs the deterministic allocation counts §16.6 asserts (`style_resolve_10k_parts` "exactly 0 allocs", `debug_and_release_alloc_counts_match`). Revisit only if a component adopts `Layout`, and re-bless `perf_baseline.txt` in the same commit. |
| `ratatui-core/palette` | **off** | Adds only `From<Srgb>`/`From<LinSrgb>` for `Color`. **There is no `Color::from_hsl` in 0.30** — the only constructor is `Color::from_u32`, `const` and unfeatured. `downgrade_color` (§11.4) is exact integer arithmetic; pulling `palette` + `libm` for two `From` impls is a framework-sized dependency for a small amount of well-understood code. |
| `ratatui-core/scrolling-regions` | **off** | Only affects `Terminal::insert_before` for inline viewports. We render fullscreen with full-frame diffing. |
| `unstable-widget-ref` / `unstable-*` | **off** | `ratatui`-only; we implement no ratatui widget trait (§3.2 rejects a universal trait). Adopting an unstable API in a library that claims a stable public surface (G1) is a contradiction. |
| `serde`, `anstyle`, `portable-atomic`, `all-widgets`, `widget-calendar`, `macros` | **off** | Unused; `macros`/`all-widgets` are `ratatui`-only and would drag in crates we reject. |
| `junie-tui/crossterm` | **on by default** | Apps need it; `--no-default-features` is the boundary proof. |
| `junie-tui/testing` | **off by default** | Enabled only via the dev-dependency path. |

**`ratatui-macros` — rejected.** (1) `constraints!`/`vertical!`/`horizontal!` produce `Layout` + `Constraint`; §10 fixes `Track::{Fixed, Flex, Auto}` and "no general constraint solver" — adopting them introduces a **second layout vocabulary** in the same crate, the API-inconsistency class this refactor exists to remove (G1). (2) `line!`/`span!`/`text!` allocate a `Span`/`Line`/`Text` per invocation and are `format!`-shaped; §16.6 demands 0 allocs/frame on the row path and §17.0 A8 already specifies `RowUi::label_fmt(core::fmt::Arguments<'_>)`; `line!` also shadows `std::line!`. (3) `row!` builds `ratatui_widgets::table::Row`, and `DataTable` is deleted (§18.2). (4) Goal §5 and §2.2 ("no macro DSL"). The library's only macro stays `id!` (§7.1).

### 22.2 Current-code API drift — decisions (MOD §2)

Each item names the exact modern primitive; the `Ui` signatures are in §17.0 A2.

1. **Fill and restyle** (MOD §2.1). `Ui::paint_style(area, style)` = `buf.set_style(clip.intersection(area), style)` (style-only, already intersection-clipped); `Ui::fill(area, style)` = per-position `set_symbol(" ").set_style(style)` over `area.positions()`. **No nested `for y … for x` over a rect anywhere** (R‑4; kills `ctx.rs:136-137`). `ratatui_widgets::Fill`/`Dimmed` are **not** taken: foreign `Widget`s write straight to the `Buffer` and cannot mark the layer's written-cell bitset (R3); `Dimmed` halves RGB channels and falls back to `Color::Black` for non-RGB — colour-space arithmetic where §11.3 A2 requires role arithmetic — and cannot exclude the footer row. `Ui::fill` and `Ui::dim_layer` are recorded as **deliberate re-implementations** so a later reader does not "fix" them.
2. **Painting a string** (MOD §2.2). `Ui::paint_str(area, text, style) -> u16` **is** `buf.set_stringn(area.x, area.y, text, area.width as usize, style)`: it clips to `max_width` and the buffer's right edge, filters zero-width graphemes and control characters, resets the trailing cells of multi-width graphemes, and returns the end column — everything `ui::text::fit` did, allocation-free. `fit`/`fit_right` are deleted from every render path (§18.1; `fit_10k_grapheme_line_to_80`'s `RowUi` equivalent records **0**); `truncate`/`truncate_middle` survive only for non-render callers that need an owned ellipsised string.
3. **ONE width function** (MOD §2.3 — a correctness bug class today). **[F]** ratatui 0.30 measures with its own `CellWidth` trait: `impl CellWidth for str` returns `1` for single-byte strings and otherwise `unicode_width + count_halfwidth_sound_marks` — **+1 per U+FF9E/U+FF9F**, which `unicode-width` reports as zero-width while terminals render one cell; `Buffer::set_stringn` consumes columns with `cell_width`. Today's `width()` (`ui/text.rs:6-8`) disagrees, so for halfwidth-katakana text layout reserves N columns and the buffer consumes N+k — the R4/R5 overflow class. Decision: `crates/tui/src/text/measure.rs::width(s: &str) -> u16` delegates to `<str as ratatui_core::buffer::CellWidth>::cell_width`; every measurement (`RowUi`, `CellUi`, `truncate`, `wrap`, `TextEditorCore`, the viewport's `Cell.w`, `StatusBar`'s priority drop) goes through it; `unicode_width::` may be imported nowhere else (R‑1); `width_cjk` is never used (no East-Asian context switch in the architecture); `unicode-width` stays a direct dependency only so `measure.rs` can name it, and its `0.2.2` unifies with `ratatui-core`'s `>=0.2.0` to a **single** copy — required, otherwise two width tables could disagree (`dependency_graph_is_exactly_the_declared_set` assertion 4). Test `text::width_matches_ratatui_cell_width` (§16.1). Visual consequence recorded as §20.10 item 16.
4. **Segmentation** (MOD §2.4). `ui::text::fuzzy` is rewritten over `grapheme_indices(true)` on the **original** label comparing case-folded graphemes, which fixes the §1.3 mis-highlight and satisfies `fuzzy_returns_grapheme_indices_into_the_original_label` in one change. `unicode_segmentation::` is banned outside `crates/tui/src/text/**`.
5. **Mouse normalisation** (MOD §2.5). **[F]** crossterm 0.29's `MouseEvent` carries `modifiers`, and `MouseEventKind` is a closed 8-variant enum. §6.1's `Mouse { kind, pos, mods }` is implemented with an **exhaustive match — no `_` arm** over `MouseEventKind`, so a crossterm 0.30 variant is a compile error instead of a silently dropped event; `ScrollLeft`/`ScrollRight` map to `Wheel(Axis::H, ±1)` (R‑15).
6. **Key events; no keyboard-enhancement flags** (MOD §2.6). Press/repeat detection uses `Event::as_key_press_event` / `KeyEvent::is_press/is_repeat/is_release` (R‑16). **[F]** `KeyEvent`'s `PartialEq`/`Hash` normalise ASCII case against `SHIFT`, which is exactly what `Chord: Eq + Hash` wants — documented, not re-implemented. **`PushKeyboardEnhancementFlags`/`PopKeyboardEnhancementFlags` are never pushed** (R‑17): `DISAMBIGUATE_ESCAPE_CODES` makes bare Esc arrive as a CSI-u sequence (a second code path through the §9 Esc ladder); `REPORT_EVENT_TYPES` starts delivering `Release` events §3.3 step 1 drops anyway; support is terminal-dependent, contradicting §16.4's determinism; §13.1's chords need only Ctrl/Alt/Shift. If ever wanted: gate on `supports_keyboard_enhancement()` and a `Capability` field, and re-bless the app baselines. Corollary: `key_release_is_dropped` (§16.1) **synthesises** a `KeyEvent` with `kind: Release` — on Unix without the flags no terminal produces one.
7. **Terminal session** (MOD §2.7). `TerminalSession` mirrors `ratatui::try_init` literally and cites it in the module docs: the chained panic hook is installed **before** the first mode change; `type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>` is our own alias (§17.0 A1); the raw `ENABLE_WRAP = "\x1b[?7h"` string (`runtime.rs:42`, `:91`) is replaced by the typed `crossterm::terminal::{DisableLineWrap, EnableLineWrap}` inside the same `execute!` (a raw escape byte string in a library is the exact "hand-rolled where a typed command exists" smell; the grep forbids `\x1b[`); restore is one reverse-order `execute!`; `leave()` stays idempotent; `terminal.draw(|f| …)` stays because the render closure is infallible, `try_draw` documented as the alternative. Mouse capture and bracketed paste remain ours to enable — none of ratatui's `init` variants do (R‑18). Test `runtime::panic_hook_restores_before_delegating`.
8. **Cursor** (MOD §2.8). **Exactly one** `frame.set_cursor_position` call site in the workspace, in `Runtime::draw` (§3.3 step 15, §8.4); components call `ui.set_cursor(owner, pos)` only (R‑7; grep rule 23).
9. **`Stylize` — banned, contrary to the usual advice** (MOD §2.9). Every `Stylize` colour shorthand names a literal ANSI colour, which goal §15 forbids and §11.3 A2 forbids structurally. The only legitimate style source inside a component is `ui.style(family, variant, part, flags) -> Resolved`. Banned in `crates/tui/src/**` and `apps/**/src/**`; permitted in `crates/tui-testing` fixtures and doctests where a literal colour is the point. Also one spelling: **`Style::new()`**, never `Style::default()` (R‑8). This is a deliberate inversion of the common ratatui idiom and is recorded so a reviewer does not "modernise" it back.
10. **`Style::patch`** (MOD §2.10). The **final** application of `Resolved.style` over the inherited surface style is `inherited.patch(resolved.style)` — it is the only layering with correct `add_modifier`/`sub_modifier` semantics and §11.3's modifier-symmetry law is exactly its semantics. `StylePatch::merge` (role level) is not replaceable by it. Never layer by field reassignment (today's `theme.rs:333-357` cannot express "remove BOLD"). Test `theme::patch_merge_matches_ratatui_style_patch_for_modifiers` (R‑9).
11. **Layout: borrow the vocabulary, not the engine** (MOD §2.11). Keep §10's hand-written `layout::{rows, columns, responsive_columns, action_row, inset, split_v, split_h}` (deterministic, 0-alloc, expresses `Auto`); stop re-implementing what `Rect` already does — `Rect::centered`/`centered_horizontally`/`centered_vertically` for `Anchor::Screen`, `Rect::clamp` for §9.1's clamp step, `Rect::inner(Margin)` for symmetric insets, `Rect::ZERO`; name `RowAlign::{Start, End}` and the gap parameter `spacing` so they read against `Flex`/`Spacing` (applied to §10 and example 5); `Layout::`/`Constraint::`/`Flex::`/`Spacing::` may not appear under `components/**` (R‑12, R‑13).
12. **Symbols** (MOD §2.12). `pub type BorderSet = ratatui_core::symbols::border::Set<'static>` (§11.2) — ours was strictly narrower (6 fields vs 8) and could not express asymmetric runs; `ThemeBuilder::borders_set(border::PLAIN)` replaces `BorderSet::SQUARE`; Junie's rounded set is `symbols::border::ROUNDED` verbatim (verify against current output before blessing). The scrollbar keeps the **Junie glyphs** (`│`/`┃`, which no built-in set matches — the built-in thumbs are `█`) but types them as `symbols::scrollbar::Set<'static>`, so `GlyphRole::{ScrollTrack, ScrollThumb}` resolve from a typed set; `scrollbar.rs:8-9` deleted. Rules and seams: `symbols::line::Set` (R‑11).
13. **`ratatui_widgets::Scrollbar`/`ScrollbarState` — rejected** (MOD §2.13), for three structural reasons: (a) a `Widget` that paints into a `Buffer` cannot register `Part::TRACK`/`Part::THUMB` hit regions, which §12.2 requires for thumb drag through pointer capture; (b) it cannot resolve through `ui.style(SCROLLBAR, …)`, bypassing the recipe system (G6); (c) `ScrollbarState` would be a second source of truth beside `ScrollState` (§18.1). Only its **symbol sets** are adopted. Forbidden pattern (rule 15).
14. **`Block`, `Padding`, titles** (MOD §2.14). All stay out with `ratatui-widgets`. `Ui::frame(area, style) -> Rect` draws the theme `BorderSet` and returns the inner rect — the 20 lines `Block::bordered().inner()` would give, but participating in the clip rect, the written-cell bitset and role resolution. `Padding` is `Insets` (§10). **[F]** There is no `Title` type in 0.30; titles are `Line`s. `Shadow` is noted as the one tempting piece for popovers — left out; taking `ratatui-widgets` for one widget must be re-argued if §9's chrome ever wants it.
15. **Deprecated and trap APIs** (MOD §2.15). `Buffer::get`/`get_mut` are `#[deprecated]`; cells are reached by `Buffer::cell`/`cell_mut` (`Option`), never by the panicking index (R‑5). **Security trap:** `ratatui_core::text::Masked`'s `Debug` prints the raw secret verbatim (`masked.rs:50-56`); any `Masked` reachable from a `#[derive(Debug)]` struct leaks. `Masked` is **forbidden** in library and apps (R‑19); `Secret` + `SecretPolicy` with a manual redacting `Debug` and a synthetic tail is the only masking path; `conformance::secret_never_appears_in_debug` (§16.2 case 18) is extended to assert that no `Masked` is constructible from a `Secret`.
16. **`Text`/`Line`/`Span`** (MOD §2.16). Keep our own span type (it stores a `Tone`/`Role`, not a resolved `Style`, so a viewport re-themes without rebuilding and `Ui::dim_layer` can walk roles; storage per §18.2). But `RowUi::label_spans` paints through a ratatui writer <!-- amended by §25 F4: `Buffer::set_span`, span by span, accumulating the x cursor and the per-span role marks — never a collected `Vec<RawSpan>`, which allocates once per call on the row path (§20.9‑6, R5) --> (per-span clipping and `line.style.patch(span.style)` for free, and it cannot drift from `set_stringn`'s width accounting) instead of a hand-written span cursor (R‑3). `ToSpan`/`ToLine` are rejected as the `Display → row label` bridge (they allocate through `to_string()`); `RowUi::label_fmt` is the reason.
17. **Colour literals** (MOD §2.17). `mod palette::rgb` (`theme.rs:56-65`) is deleted; literals are `Color::from_u32(0x00RRGGBB)` (`const`, unfeatured) and only in `theme/builtin/{junie,paper}.rs` and `tests/fixtures/**` (R‑10). `architecture::palette_literals_are_confined_to_theme_builtins` greps `Color::from_u32(` too (applied, §16.5), or the check would pass while every literal moves one call deeper.
18. **Multi-width safety in `Ui::paint_cell`** (MOD §2.18). `set_stringn` resets the cells shadowed by a multi-width grapheme and the diff assumes "no double-width cell is followed by a non-blank cell". `Ui::paint_cell` must replicate that reset or a wide grapheme written cell-by-cell corrupts the diff — a documented invariant covered in `render::components::*` with a CJK fixture (R‑6).

### 22.3 Rust 2024 / MSRV-1.88 practice (MOD §3)

**Language.** Every feature the architecture needs is available at 1.88: edition 2024 (1.85), let-chains (1.88, already idiomatic in this codebase — keep), `#[expect(lint, reason = "…")]` (1.81, **replaces every `#[allow]`**), `core::error::Error` (1.81) for `FieldError` — `core`, not `std`, matching ratatui's own `Backend::Error` and `ParseColorError` — RPIT in inherent fns (`FocusRing::reachable`). **Avoid** RPITIT (makes a trait non-dyn-safe; §12.1 relies on `&dyn Fn` slots). `IntentIter<'f>` (§17.0 A2) is correctly a **named** type — that is what lets it outlive the `&Cx` borrow (§21 item 6); do not "modernise" it to `impl Iterator`. `LazyLock` is not used in `crates/tui`. Every `const fn` in §7.1/§17.0 operates on integers and `&'static str` and needs no const-trait support.

**Attribute and structure policy (binding).**

* `#![forbid(unsafe_code)]` in `crates/tui/src/lib.rs` and every app crate root. **`crates/tui-testing` uses `#![deny(unsafe_code)]`**, not `forbid` (`forbid` cannot be overridden; that crate carries the documented `unsafe impl GlobalAlloc`, §16.6, under a local `#[expect(unsafe_code, reason = "counting allocator; see SAFETY")]` plus a `// SAFETY:` comment). Correspondingly `[workspace.lints.rust] unsafe_code = "deny"`, tightened to `forbid` at the `crates/tui` root; `forbid` at workspace level would make `tui-testing` uncompilable.
* `#![deny(missing_docs)]` in `crates/tui` (§16.5) plus `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` in CI (goal §26).
* **`#![doc = include_str!("../README.md")]`** at the top of `crates/tui/src/lib.rs`, so `cargo test --workspace --doc` compile-checks every README example — the cheapest way to satisfy goal §24 + §25.5 "all examples compile" for the quick-start. Caveat: every README code fence must be valid Rust or tagged ```` ```text ````/```` ```ignore ````.
* **Doctests hide setup only.** `#`-hidden lines hold fixture construction (`# let mut ui = junie_tui::testing::ui();`); never hide a `use` a real downstream caller would need — a doctest that compiles only because a hidden line imports a private path is a false proof of §17's "external consumer" claim.
* **`#[non_exhaustive]` — apply narrowly.** Yes on types the runtime produces and the caller only matches, which will grow: `Diagnostic`, `Intent`, `Phase`, `FocusVia`, `LayerEvent`, `DismissReason`, `Status`, `ColorLevel`, `RegionKind`, plus the already-marked `LayerSpec` and `LayoutFacts` (applied inline in §6.1, §9.1, §17.0 A8/A9, B.3 item 4). **No** on `ColorTokens` (§17 example 2: a new token must be a compile error for downstream themes; `map_colors`'s exhaustive destructure depends on it), `StylePatch`, `RowDecor`, `CellDecor`, `Insets`, `Headroom` — all constructed by users with struct literals. Trade-off stated: `#[non_exhaustive]` on an enum forces a wildcard arm downstream and destroys the "adding a variant is a compile error" property; apply only where downstream exhaustiveness is not wanted.
* **No process-global state in `crates/tui/src`.** `LazyLock`, `OnceLock`, `static mut`, `thread_local!` are forbidden (rule 18). The `Runtime` owns all state (§4); a global would break `Harness` isolation and `--test-threads=1`-free testing. This is a second, independent reason `ratatui-core/layout-cache` stays off — it *is* such a global.
* `[lints] workspace = true` in every member; `[workspace.lints]` defined once at the root (verbatim in Appendix B.2), so `architecture::msrv_and_edition_are_unchanged` and the lint policy each read one place.

**Lint policy notes.** `clippy::all` at **deny** is the workhorse; `pedantic` at **warn** because `-D warnings` in CI promotes it to a hard failure while leaving local development usable. **`nursery` is rejected** as a group (unstable, churns between toolchains, would break a pinned-MSRV job on a compiler upgrade unrelated to our code). **`restriction` is never enabled as a group** — only the individual lints listed. `indexing_slicing = "deny"` is aggressive and is *the point*: it makes the `grid.rs:1458/505` and `table.rs:220/294/317/700`-class panics impossible to reintroduce; relax for tests only, at the crate root: `#![cfg_attr(test, allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic))]`. The four `cast_*` allows are justified **by citation**, not by fatigue: ratatui-core, ratatui and ratatui-crossterm allow exactly these four for exactly this reason (`ratatui-core-0.1.2/Cargo.toml:179-182`).

**`cargo-semver-checks`** (MOD §3.4): adopted, **not during the refactor** — against a published or git baseline every check fails by construction during a total public-API rewrite. `xtask semver` wraps `cargo semver-checks --baseline-rev <tag>` at the end of Slice 8, tag `v0.1.0`, blocking in CI from `v0.1.1`. Recorded in §16.5 as a deferred check, not a shipped one.

### 22.4 `bitflags` adopted, `smallvec` rejected (MOD §4)

**`bitflags 2.13` — adopt.** **[F]** Already in the graph: `ratatui-core` depends on `bitflags = "2.12"` (for `Modifier`), crossterm 0.29 uses it for `KeyModifiers`/`KeyEventState`/`KeyboardEnhancementFlags`; requesting `"2.13"` unifies to a **single** copy — zero new nodes. Uses: `StateFlags` (16 bits, §6.1), `Caps: u32` (§16.2). The architecture actually needs `count_ones()` (§11.3 specificity ordering), a readable `Debug` (`FOCUSED | HOVERED`, G5), `iter()`/`iter_names()` (§11.4 mono walk, `state_flags_round_trip`), `from_bits_truncate`, `all()`, `Sub`, `Not`, `BitXor` (example 12's masking). Hand-rolling that is ~200 lines of test-hungry code to avoid a dependency that is already compiled.

**`smallvec` — reject.** Architecture uses were `PartRecipe.states: SmallVec<[StateRule; 8]>`, `Recipe.variants: SmallVec<[…; 6]>` (§11.3), `KeySet::{Only, AllExcept}(SmallVec<[ItemKey; 8]>)` (§17.0 A8), `HintLayer.hints: SmallVec<[Hint; 8]>` (§13.1). **[F]** `smallvec` is not in the current graph — a genuinely new node; smallvec 2.0 is alpha and ineligible. Four reasons: (1) **the stated bounds are not real bounds** — `RecipeEdit::when` is a public builder and `apply_mono_fallbacks` appends one rule per family, so `states` can exceed 8; `define_variant` is public; `KeySet` must hold a 5 000-row multi-selection over a 100k grid; (2) **it buys nothing on any hot path** — `Recipes` is built once at theme construction and resolution only *reads* these containers, where `Vec` is equally 0-alloc (`style_resolve_10k_parts`), and `KeySet` mutates on input events, not per frame (`AllExcept(Vec::new())` is 0 allocs, satisfying `list_100k_select_all`); (3) **it makes `Theme` bigger and `Theme::clone()` more expensive** — `StateRule` ≈ 40 bytes × 8 inline ≈ 320 bytes per `PartRecipe` × ~34 parts × ~34 families ≈ hundreds of KB of mostly-empty inline storage per `Theme`, all copied by `Theme::downgrade`'s `self.clone()`; (4) goal §21 "keep dependencies focused". **Decision:** `Vec<StateRule>`, `Vec<(Variant, PartMap<PartRecipe>)>`, `Vec<Hint>`; `KeySet` is a **`Vec<ItemKey>` kept sorted**, so `contains` is a binary search — which fixes a complexity problem SmallVec never touched: an unsorted container with 5 000 selected rows over a 100k grid costs ~200 000 comparisons per frame and would breach `list_100k_rows_render`'s "≤ 1.5× `list_1k_rows_render`" regardless of where the bytes live; sorted makes it O(40 · log 5000). Tests `key_set_stays_sorted_after_insert_remove_toggle_retain`, `key_set_contains_is_binary_search` (§16.1). Applied inline: §11.3, §13.1, §16.5, §17.0 A8, §21 item 30, Appendix B.1/B.2.

### 22.5 MSRV — `rust-version = "1.88"` held, and verified (MOD §5)

**[F]** Every direct dependency declares `rust-version = "1.88.0"`; 1.88 is the ecosystem floor. Every capability the architecture depends on is available at 1.88 (table in §22.3); the only capability that would materially change the design — const trait support / `const fn` in traits, which would let `StylePatch::merge` and a fully `const` `Overlay` evaluate at compile time — is not stable in any released Rust up to and including the 1.98 toolchain in use, so no bump reaches it. Raising the MSRV would exceed every dependency's own floor and buy no feature, which goal §21 forbids. **But the pin is a declaration, not a fact:** `msrv_and_edition_are_unchanged` reads `cargo metadata` and proves the *field*, not that the code compiles on 1.88. Closed by the blocking CI job `msrv`: `cargo +1.88.0 check --workspace --all-targets --all-features` (or `cargo-msrv verify`); optionally a `rust-toolchain.toml` `channel = "1.88.0"` verification profile, keeping the **primary** CI job on stable so new diagnostics are seen early. The MSRV was re-examined during the refactor and deliberately held, with MOD §5 as the justification (to be mirrored in `REFACTORING_STATE.md`, coordinator-owned).

### 22.6 Binding rules for builders (MOD §6.1)

| # | Rule | Exact API |
|---|---|---|
| R‑1 | **One width function.** All display width goes through `crates/tui/src/text/measure.rs::width`, which delegates to `CellWidth::cell_width`. No file outside that one may import `unicode_width`. | `ratatui_core::buffer::CellWidth` (`ratatui-core-0.1.2/src/buffer/cell_width.rs:19-46`) |
| R‑2 | **Never pre-truncate for painting.** Use the clipping writer; it returns the end column. `fit`/`fit_right` are banned on every render path. | `Buffer::set_stringn` (`buffer.rs:336-370`) |
| R‑3 | **Multi-span rows use a ratatui writer**, not a hand-rolled span cursor. <!-- amended by §25 F4 (BL‑4) --> `Buffer::set_span` is a sanctioned per-span writer **alongside** `set_line`, and is the one `Ui::paint_spans` must use: collecting a `Vec<RawSpan>` to hand `set_line` allocates once per call on the row path, which §20.9‑6 (R5) forbids. Either writer keeps the guarantee the rule exists for — width accounting cannot drift from `set_stringn`'s. | `Buffer::set_line` (`buffer.rs:373-392`), `Buffer::set_span` |
| R‑4 | **No nested `for y … for x` over a rect.** Iterate positions, or restyle wholesale. | `Rect::positions()` / `rows()` / `columns()`; `Buffer::set_style(area, style)` (`buffer.rs:405-413`) |
| R‑5 | **Cells are reached by `Option`, never by the deprecated accessors.** | `Buffer::cell` / `cell_mut` (`buffer.rs:179-214`); `Buffer::get`/`get_mut` are `#[deprecated]` |
| R‑6 | **`Ui::paint_cell` must reset the cells shadowed by a wide grapheme**, matching the writer and the diff assumption. | `buffer.rs:359-368`, `:476-477` |
| R‑7 | **One cursor write per frame**, in `Runtime::draw`. Components call `ui.set_cursor(owner, pos)`. | `Frame::set_cursor_position` (`ratatui-core/src/terminal.rs:11-12`, `:369-372`) |
| R‑8 | **No style literal outside the theme.** Every style comes from `ui.style(family, variant, part, flags) -> Resolved`. `Stylize` shorthands are banned in library and app code. Spell `Style::new()`, never `Style::default()`. | `ratatui_core::style::Style` (`src/style.rs:74-76`, `:131-139`) |
| R‑9 | **Layer a style with `patch`, never by field reassignment** — the only form with correct `sub_modifier` semantics. | `Style::patch` (used at `buffer.rs:385`) |
| R‑10 | **Colour literals are `Color::from_u32`, and only inside `theme/builtin/`.** There is no `Color::from_hsl`. | `Color::from_u32` (`style/color.rs:133-138`) |
| R‑11 | **Border and scrollbar glyph sets are ratatui symbol sets**, not bespoke structs. `BorderSet` is a type alias. | `symbols::border::{Set, PLAIN, ROUNDED, DOUBLE}`; `symbols::scrollbar::{Set, VERTICAL}`; `symbols::line` |
| R‑12 | **Reuse `Rect` geometry; do not re-derive it.** Centering, clamping, margins, emptiness, zero. | `Rect::{centered, centered_horizontally, centered_vertically, clamp, intersection, union, inner, outer, offset, is_empty, ZERO}` (`layout/rect.rs:38-67`, `:153`) |
| R‑13 | **No constraint solver inside components.** `Layout`/`Constraint`/`Flex` may not appear under `components/**`; use `layout::{rows, columns, action_row, inset}`. (This is what keeps `layout-cache` off.) | `ratatui_core::layout::{Layout, Constraint, Flex, Spacing}` — vocabulary reference only |
| R‑14 | **Crossterm is reached only through the backend re-export**, never a direct `crossterm` dependency. | `ratatui_crossterm::crossterm` (`ratatui-crossterm-0.1.2/src/lib.rs:86-98`) |
| R‑15 | **Input normalisation matches `MouseEventKind` exhaustively** (no `_` arm) and carries `modifiers`. | `crossterm::event::{MouseEvent, MouseEventKind}` (`crossterm-0.29.0/src/event.rs:777-817`) |
| R‑16 | **Key press/repeat detection uses the accessors**, not hand-matched `kind`. | `Event::as_key_press_event`, `KeyEvent::is_press/is_repeat/is_release` |
| R‑17 | **No keyboard-enhancement flags.** They are a rejected design, not an oversight. | `PushKeyboardEnhancementFlags` / `PopKeyboardEnhancementFlags` / `KeyboardEnhancementFlags` |
| R‑18 | **Terminal modes are typed commands, never raw escape strings.** Mouse capture and bracketed paste are ours to enable; raw mode, alt screen and the chained panic hook mirror `try_init`. | `crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, EnableLineWrap, DisableLineWrap}`, `event::{EnableMouseCapture, EnableBracketedPaste}`; reference `ratatui-0.30.2/src/init.rs:397-399`, `:182-197` |
| R‑19 | **`ratatui_core::text::Masked` is forbidden** — its `Debug` prints the raw secret. `Secret` is the only masking path. | `ratatui-core-0.1.2/src/text/masked.rs:50-56` |
| R‑20 | **Nothing from `ratatui`, `ratatui-widgets`, or `ratatui-macros`** enters the workspace. `Block`, `Padding`, `Clear`, `Fill`, `Scrollbar`, `ScrollbarState`, `Shadow`, `Dimmed`, `Paragraph`, `List`, `Table` are re-implemented on `Ui` or deliberately absent, with the reason recorded. | `ratatui-0.30.2/src/widgets.rs:668-691`; `ratatui-widgets-0.3.2/src/lib.rs:115-138` |

### 22.7 `architecture::no_deprecated_or_legacy_api_usage` and the dependency-graph check (MOD §6.2)

New test in `crates/tui/tests/architecture.rs`, driven from `xtask` (it reads the whole workspace). Scans `crates/tui/src/**`, `crates/tui-testing/src/**`, `apps/**/src/**`. <!-- amended by §25 MA‑2, F9 --> The scan covers **whole files**: `non_test_lines` skips only the `#[cfg(test)]`-attributed **item** (brace-matched through `syn`, which `xtask` already depends on), never the file tail. Breaking at the first `#[cfg(test)]` line left everything after a mid-file `#[cfg(test)] pub(crate) const fn stats` unscanned by all 26 rules — in practice all of `theme/resolve.rs` past line 239 and `runtime.rs` past line 1028. Allow-list `crates/tui/tests/allow/legacy_api.txt`; every entry requires a same-line justification the test **prints on failure and on success**, so a growing allow-list is visible in CI output. **Pass condition: the allow-list is empty.**

| # | Forbidden pattern (regex) | Where allowed | Why |
|---|---|---|---|
| 1 | `Buffer::get\b\|Buffer::get_mut\b` | nowhere | deprecated |
| 2 | `enable_raw_mode\|disable_raw_mode` | `crates/tui/src/runtime/session.rs` | R‑18 |
| 3 | `EnterAlternateScreen\|LeaveAlternateScreen\|Enable(Mouse\|Bracketed)\|Disable(Mouse\|Bracketed)` | same file | R‑18 |
| 4 | `\\x1b\[\|\\u\{1b\}\[` | nowhere | raw ANSI; replaces `runtime.rs:42`, `:91` |
| 5 | `KeyboardEnhancementFlags` | nowhere | R‑17 |
| 6 | `for\s+\w+\s+in\s+\w+\.top\(\)\.\.\|\.left\(\)\.\.` | `crates/tui/src/ui/paint.rs` | R‑4; kills `ctx.rs:136-137` |
| 7 | `Rect::new\(` | `crates/tui/src/{layout.rs,ui/**}`, tests | components receive rects; kills `list.rs:236`, `viewport.rs:645`, `button.rs:110` |
| 8 | `Style::default\(\)` | nowhere | R‑8, one spelling |
| 9 | `\.fg\(\|\.bg\(\|add_modifier\(\|remove_modifier\(\|underline_color\(` | `crates/tui/src/theme/**`, `crates/tui/src/ui/paint.rs` | R‑8; kills `button.rs:127-159`, `list.rs:253-296`, `viewport.rs:615-627` |
| 10 | `style::Stylize\|\.(red\|green\|blue\|yellow\|magenta\|cyan\|white\|black\|gray\|on_[a-z]+)\(\)` | `crates/tui-testing/**`, doctests | R‑8 |
| 11 | `Masked\b` | nowhere | R‑19 |
| 12 | `unicode_width::\|UnicodeWidth(Str\|Char)` | `crates/tui/src/text/measure.rs` | R‑1 |
| 13 | `unicode_segmentation::` | `crates/tui/src/text/**` | one segmentation site |
| 14 | `\bratatui::\|ratatui_widgets::\|ratatui_macros::` | nowhere | R‑20 |
| 15 | `Scrollbar\b\|ScrollbarState\|ScrollbarOrientation\|ScrollDirection` | nowhere | §22.2 item 13 |
| 16 | `Block::\|Paragraph::\|Padding::\|BorderType::\|Borders::\|Clear\b\|Fill::new\|Shadow::\|Dimmed\b` | nowhere | R‑20 |
| 17 | `#\[allow\(` | `crates/tui-testing/**` (documented) | `#[expect(…, reason=…)]` only (§22.3) |
| 18 | `LazyLock\|OnceLock\|static mut\|thread_local!` | `crates/tui-testing/**`, `xtask/**` | no process-global state (§22.3) |
| 19 | `\.unwrap\(\)\|\.expect\(\|panic!\|todo!\|unimplemented!` | `#[cfg(test)]`, `crates/tui-testing/**`, `xtask/**` | goal §10; belt-and-braces beside clippy |
| 20 | `fn render\(&mut self\|fn draw\(&mut self` | nowhere under `components/**` | G2; companion to `draw_takes_shared_self` |
| 21 | `bg:\s*(ratatui_core::style::)?Color` in any `pub fn` | `Role::Custom` only | §16.5 existing rule, restated |
| 22 | `Color::Rgb\(\|Color::from_u32\(\|#[0-9a-fA-F]{6}` <!-- amended by §25 D‑10: the regex is the BROAD one and stays broad; narrowing it to `Color::Rgb\(\s*\d\|Color::from_u32\(\s*0x` (as `xtask` did) lets `Color::Rgb(r, g, b)` from computed values through anywhere in the crate --> | `crates/tui/src/theme/builtin/{junie,paper}.rs`, `tests/fixtures/**`, **plus the named paths** `crates/tui/src/theme/downgrade.rs` and `crates/tui/src/theme/builder.rs` (computed `Color::Rgb(r, g, b)` from the downgrade and L\* derivation arithmetic). A **path** allow does not feed the "`legacy_api.txt` must be empty" condition: a narrowed regex hides the exception, a named path shows it. | §16.5 existing rule + R‑10 (the `from_u32` arm is new and necessary) |
| 23 | `set_cursor_position` | `crates/tui/src/runtime.rs` | R‑7 |
| 24 | `Layout::\|Constraint::\|Flex::\|Spacing::` | nowhere under `components/**` | R‑13 |
| 25 | `\.child\(\|\.owns\(\|\.locate\|scrollbar::id_for\|WidgetId` | nowhere | §16.5 existing rule, extended to `WidgetId` |
| 26 | `SmallVec\|smallvec::` | nowhere | §22.4 |
| 27 <!-- amended by §25 (adjudication 1, D‑1) --> | `CrosstermBackend\|ratatui_crossterm::(?!crossterm::event)` | `crates/tui/src/runtime/session.rs` | the mechanical proof that the backend is confined to one file. `cargo check --no-default-features` cannot prove it — `ratatui-crossterm` is a normal, non-optional dependency (§22.1) — so this rule, together with `architecture::ratatui_crossterm_is_named_in_exactly_two_files`, is what makes the backend-independence claim true instead of theatre |
| 27a <!-- amended by §25 F2 (BL‑2) --> | `loop\s*\{[^}]*spin_loop` (a `loop {` whose body calls `core::hint::spin_loop`) | nowhere in `crates/tui/src/**` | an "unreachable" arm that **hangs** the process is strictly worse than a panic: `TerminalSession`'s chained hook restores the terminal on a panic, while a livelock with raw mode on and the alternate screen entered leaves the user with an unusable terminal and no stack. The two occurrences existed only to satisfy `clippy::panic`/`expect_used` at deny; the correct form is one documented `#[expect(clippy::expect_used, reason = "Vec::insert(i, _) makes get_mut(i) infallible")]`, or a restructure that returns the reference from the insert branch. Test `architecture::no_unreachable_spin_loops` |

**Companion check `architecture::dependency_graph_is_exactly_the_declared_set`** (`xtask`, `cargo metadata`): (1) `junie-tui`'s direct normal dependencies are exactly `{ratatui-core, ratatui-crossterm, unicode-width, unicode-segmentation, bitflags}`; (2) <!-- amended by §25 (adjudication 5) --> split into four parts, because "`smallvec` is absent from the normal closure" was simply **false** — it arrives through `ratatui-crossterm → crossterm → parking_lot → smallvec`, and `ratatui-crossterm` is mandatory (§22.1). Pruning the crossterm subtree makes the check pass but silently deletes a whole subtree without asserting anything about it. **(2a)** `ratatui`, `ratatui-widgets` and `ratatui-macros` are absent from `junie-tui`'s **entire** normal closure. **(2b)** `critical-section` and `palette` are absent from the **entire** (unpruned) closure — they can only arrive through `ratatui-core` features we disable. **(2c)** `smallvec`, `parking_lot`, `parking_lot_core`, `lock_api`, `scopeguard`, `libc`, `mio` and `signal-hook*` may appear **only beneath `ratatui-crossterm`**: asserted positively with `cargo tree -p junie-tui -e normal --invert <crate>` for each of the eight names, requiring `ratatui-crossterm` on every printed path. They are crossterm's internals, not a choice of ours; §22.4's decision is about *our* containers and is enforced by forbidden-pattern rule 26 over our source. The pruned subtree is printed once on success so the exception is visible in CI output. **(2d)** `junie-tui`'s **direct** normal dependencies contain no `smallvec` and no direct `crossterm`; (3) each app's direct normal dependencies are exactly `{junie-tui}`; (4) `unicode-width`, `unicode-segmentation` and `bitflags` each resolve to **one** version in the graph; (5) enabled features on `ratatui-core` are exactly `{std, underline-color}`. **Companion CI gates** (beyond goal §26's list): `cargo check -p junie-tui --no-default-features`; `cargo +1.88.0 check --workspace --all-targets --all-features`; `cargo test --workspace --doc` with the README included. All three are in §16.5 and Appendix A's gates.

### 22.8 Risks (MOD §7)

1. **Losing `ratatui::init`/`restore`/`run` is a real cost of the `ratatui-core` decision.** `TerminalSession` must be maintained by hand and kept in sync with upstream's hook ordering (`ratatui-0.30.2/src/init.rs:196-197`). Mitigation: mirror `try_init` literally, cite it in the module docs, `runtime::panic_hook_restores_before_delegating` (§16.1).
2. **The `CellWidth` switch will move rendered output.** Any string containing U+FF9E/U+FF9F now measures one column wider; §16.3 baselines are re-blessed **with** a `docs/visual-changes.md` entry under §20.10 item 16, never silently. Realistically zero cells change in the three apps' fixtures.
3. **`indexing_slicing = "deny"` will be loud on first application** to migrated code, especially the grid. That is the intended signal (§1.3), front-loaded in Slices 3–4; do not weaken it to `warn` under time pressure.
4. **Removing `SmallVec` changes `ColorTokens`/`Recipes` sizes**, which shifts the perf baseline. The §16.6 pre-refactor baseline is taken on the *unmodified* tree (WP‑0) and the `Vec` decision is recorded as one of the "after" explanations (§16.6).
5. **The dev-dependency cycle `junie-tui` ⇄ `junie-tui-testing`** is legal but occasionally surprises tooling (`cargo tree`, some IDEs). Fallback if it becomes a problem: move the conformance driver into `crates/tui/tests/` with `#[cfg(test)]`-only helpers — at the cost of the `publish = false` isolation §16 wants. Prefer the cycle; document it.
6. **`#[non_exhaustive]` on `Diagnostic` and `Intent` forces wildcard arms downstream**, weakening "new variant is a compile error" in app code. Deliberate (§22.3); apps must not rely on exhaustive matching of those enums (jackin's intent loop already uses `_ => {}`, §3.4).

### 22.9 Acceptance conditions (executable, MOD §8)

```bash
cargo +1.88.0 check --workspace --all-targets --all-features        # MSRV is a fact, not a field
cargo check -p junie-tui --no-default-features                      # core is backend-free
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc                                       # README + §17 examples compile
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo test  --workspace --test architecture                         # incl. the two new checks below
cargo tree  -p showcase -e normal --depth 1                         # => junie-tui, and nothing else
cargo tree  -p junie-tui -e normal | grep -E 'ratatui-widgets|ratatui-macros|^ratatui |smallvec'  # => no matches
cargo test  -p junie-tui --lib text::width_matches_ratatui_cell_width
cargo test  --workspace --test architecture no_deprecated_or_legacy_api_usage
cargo test  --workspace --test architecture dependency_graph_is_exactly_the_declared_set
```

Pass condition for the two new architecture tests: `crates/tui/tests/allow/legacy_api.txt` is **empty**, and the five `dependency_graph_is_exactly_the_declared_set` assertions hold. During Slices 3–4 the crate is `tui-next` (§21 item 31), so `-p junie-tui` reads `-p tui-next`.

### 22.10 Open items surfaced while recording — resolved by §24 <!-- amended by §24 -->

1. **Re-export name collisions** — decided as **§24 M1**: neither of our types is renamed and neither colliding ratatui type is exported; `Frame` is root-only; `Line`/`Span`/`Text` are reachable only as `author::raw::{…}`; `Ui::paint_spans` covers the multi-style paint case in our vocabulary; the exact root and `author` lists are Appendix B.4.
2. **The ASCII border set** — decided as **§24 M2**: `junie_tui::theme::border::ASCII`, a plain `const` of the foreign `Set<'static>`; Junie = `ROUNDED`, Paper = `PLAIN`; no automatic selection.

---

## 23. Adjudication K — `Form` API and `Grid::update` bound

**Status:** Accepted. Source: `docs/reviews/adjudication-k-form-grid.md` (ADJ‑K). Resolves the two items §21 left open ("Not applied — requires a fresh `opus-analyst` decision"). Nothing here reopens Adjudications A–J or L. Each inline edit carries `<!-- amended by §23 -->`.

**Facts the two decisions rest on** (ADJ‑K §0). **[F]** Three hand-built form engines exist: jackin `FormDialog` (`src/bin/jackin_preview/screens/modals.rs:787-1541` — data-declared fields, 5 kinds, one key router, one click router, scroll, action row), TablePro `ConnForm` (`connections.rs:62-87`, routers `:573-687`, `:793-861`, `:863-887`, layout `:1120-1256` — 17 controls + a `Tabs` strip + 4 buttons, three `if f == …` ladders, manual height arithmetic), TablePro `FilterEditor` (`app.rs:99-109`, `:1368-1433` — rebuilt wholesale on every open). **[F]** DOM §4.1 J2 records them as "three independent form engines" and names the required capabilities; §14.2 J2 already adjudicated "library component `Form` + `Field<C>`". **[F]** TablePro's password is a *plain* `TextInput` with a placeholder, not `.masked()` (`connections.rs:155-157`) — a live defect. **[F]** Scroll-to-focused-field mutates `ScrollState` from `render` (`modals.rs:1356-1371`); TablePro sets `disabled` and the tab error flag *inside render* (`connections.rs:1205-1206`, `:1134`); the engine→default-port effect rebuilds the `TextInput` (`:621-634`). Constraints honoured: props built once, never from `&self` (§13); data passed to phase calls, never held in props (§21 item 1); `FieldControl` is draw-time chrome only and `Field` has no `Id` (§21 item 7); one action per `Response`, `BitOr` only for `Response<()>` (§21 item 4); controlled `&mut` values (§4 rule 4, S4); `draw` is `&self` + `&XState` (§3.1, R2); containers register `Decorative` regions (§21 item 13); Esc reaches the focused editor before the layer (§21 item 3).

### 23.1 K1 — the `Form` API

**Decision: `Form` is a library component (`components/form.rs`, work package 4F), not a `FormState` + `layout::form` helper pair.** The full API — `FieldSpan`, `GroupKey`, `FieldKind`, `FieldSpec`, `FieldMut`/`FieldRef`, `FormData`, `EnterPolicy`, `Form<'a>` with its builders and `update`/`draw`/`measure` signatures, `FormAction`, `FormState` — is recorded as **§15.1**, together with invariants **F1–F13**, the no-`values()` secret-containment mechanism, the rejected alternatives, the acceptance commands, the named tests and the risks. The compile surface example 13 uses is restated in **§17.0 A10**; the condensed 15-field connection form is **§17 example 13** (`examples/13_connection_form.rs`, owned by 4F). §16.1, §16.4 and §16.6 carry the named tests; §16.2 notes `FormCase`'s `Caps`. §13's props-built-once paragraph, §14.2 J2, §21 item 30 and Appendix A 4F are amended to point at `FieldSpec::new(id, …)` + §15.1 instead of the never-defined `Form::field(id, …)`.

Two consequences of recording K1 under §22 (not new decisions): `FormState`'s internal `slots`/`errors` containers are `Vec`, not `SmallVec` (§22.4); and the K1 example's `use junie_tui::{…}` list is the public facade, so every name in it (`FieldKind`, `FieldRef`, `FieldMut`, `FieldSpan`, `FieldSpec`, `Form`, `FormAction`, `FormData`, `FormState`, `GroupKey`, `EnterPolicy`, `Checkbox`, `Toggle`, `RadioGroup`, `Select`, `TextArea`, `KeyCode`, `KeyModifiers`, `Chord`, `Secret`, `SecretPolicy`) is a root re-export.

**Sequencing.** K1 blocks **4F** (wave 2, depends on 4B); it does not block Slice 3. `xtask doc-check` (§21 item 34) now covers §15.1, §17.0 A10 and example 13, so every identifier there must resolve against the built rustdoc-json or sit on its printed allow-list.

**Open item — resolved by §24 M3** <!-- amended by §24 -->. ADJ‑K K1.2's non-generic `FieldKind<'a>` and §21 item 5's generic `Select<'a,T,K,R>` / `RadioGroup<'a,T,K,R>` / `ChipBar<'a,T,K,R>` are reconciled without changing either: the three controls stay ordinary collections with `new(id)` and per-phase items; `FieldKind` holds their default instantiation through the `LabelSelect`/`LabelRadio`/`LabelChips` aliases; the option list is data and reaches the control through `FormData::options` / `value_and_options`. §15.1, §17.0 A7/A10 and example 13 carry the amended text.

### 23.2 K2 — the `Grid::update` bound

**Decision: two entry points, with the base trait owning the base name.**

```rust
impl<'a> Grid<'a> {
    /// Read-only navigation, selection, sorting, copy, fetch-more, filter and cell actions.
    /// `&M`: a read-only grid CANNOT mutate its model — a compile-time fact, not a runtime refusal.
    pub fn update<M: GridModel + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &M) -> Response<GridAction>;

    /// Everything `update` does, plus the inline edit lifecycle: begin, cycle, commit, cancel, blur.
    pub fn update_editable<M: GridEditor + ?Sized>(
        &self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;

    /// One draw for both. Bound is the base trait, symmetric with `update`.
    pub fn draw<M: GridModel + ?Sized>(
        &self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
}
```

Two consequential corrections come with it, applied in §12.3: **`read_only_reason` moves from `GridEditor` down to `GridModel`** (defaulted `None`), and **`GridCellActions` is deleted, its `actions(row, col) -> &[CellAction]` absorbed into `GridModel`** (defaulted `&[]`). **`Grid::editable(bool)` (formerly §17.0 A7) is deleted**: capability is chosen by the entry point; the boolean was the soup §13 forbids and could contradict the model's own `is_editable`.

**Why `GridCellActions` and `read_only_reason` must move** — the finding that settles the shape, independent of which option is chosen. **[F]** `Grid::draw` is bound `M: GridModel`; `read_only_reason` was declared on `GridEditor`; the read-only reason is *rendered* (today a `DataGrid` field consumed by drawing, `src/widgets/grid.rs:348`; DOM §1.6 capability 3 keeps it). A method on `GridEditor` is **unreachable from `draw<M: GridModel>`** — as written, §12.3 could not render the reason at all. **[F]** Same for cell actions: `GridCellActions` was a third trait, yet the `→` affordance is **painted** (`grid.rs:1858-1868`, DOM §1.6 capability 16) and its hot zone registered in `draw`. Adding a second bound to `draw` would force *every* model — including the six read-only Structure-tab models — to implement it. Absorbing both as defaulted `GridModel` methods matches the precedent §12.3 already set for `row_decor`, `cell_decor`, `total`, `has_more`, and §12.2's "decoration supplied by the owner, never derived inside the component".

**Evaluation against the required criteria.**

* *§13 conventions.* "No boolean parameter soup; typed enums for semantically different modes." Read-only and editable are semantically different modes; two typed entry points are the sanctioned remedy and mirror the vocabulary §13 already uses for exactly this distinction — `.disabled(bool)`/`.read_only(bool)` with *read-only stays in the ring, disabled does not*, and `Focusability::{Focusable, FocusableReadOnly}` (§8.1). "One predictable vocabulary" is served by `update` and `draw` carrying the **same** bound and the same `&M` shape.
* *TablePro read-only-with-reason grids.* **[F]** Views / no-PK tables set a reason on the *same* adapter type that edits elsewhere (`tabs.rs:394-403`); result grids do the same when the source table is unknown, plus `local_sort = true` (`tabs.rs:2062-2068`; DOM §5.4). These call `update_editable` with an adapter whose `is_editable` returns `false` and whose `read_only_reason` returns `Some` — a *runtime* property of an editor-capable model, which is what it is today. No wrapper, no second type, no `Refuse` string per attempt.
* *Showcase demo grids, fixtures, Structure tab.* **[F]** `crates/tui/tests/fixtures/grid_model.rs` is a test-only model; DOM §2.12 turns the Structure tab into "six `GridModel`s over catalog data"; the showcase grid page is a display. These implement `GridModel` only — four required methods — and call `update`. Under option (a) each would implement a trait literally named `GridEditor`, a vocabulary lie in the type that carries it.
* *Cell-action composition.* `GridModel::actions` with a `&[]` default: FK-follow affordances work on read-only grids (a view can carry FK columns) under both entry points and are visible to `draw`.
* *Conformance.* **[F]** `Fixture` already carries `read_only: bool` (§16.2). One `GridCase` registration selects the entry point from that knob, so both paths run the full 20-case matrix from a single registration, and case 7 (`draw_does_not_commit_or_cancel`) is exercised in both. Under a single `M: GridEditor` entry point the read-only path would be a defaulted refusal, so case 7's read-only variant would assert nothing new.

**Rejected alternatives** (also tabulated in §19).

* **(a) `Grid::update<M: GridEditor>` with defaulted refusals on `GridEditor`.** Every read-only model implements a trait named "Editor" — the name lies at ~10 of ~14 known model sites; `update` would take `&mut M` while `draw` takes `&M` on the *same* model for a grid that cannot edit, conveying a capability that does not exist and blocking a shared borrow in the same frame; a default `edit_intent` would synthesise `EditIntent::Refuse { reason }` from `read_only_reason()`, allocating a `String` per refused keystroke, and an editable model that *forgets* to override `commit_cell` inherits a default that silently refuses — a wrong-but-compiling grid; it does not fix the reachability problem.
* **(c) blanket `impl<M: GridModel> GridEditor for ReadOnly<M>`.** `update` needs `&mut ReadOnly<M>`; deriving one from `&mut M` requires a `repr(transparent)` reinterpretation — `unsafe`, forbidden by `#![forbid(unsafe_code)]`; the alternative (storing `ReadOnly<Model>` in app state) contradicts §21 item 1's "the model is a phase parameter, never a field"; it needs a second blanket for actions and risks coherence conflicts; net effect is still "`update` always requires `GridEditor`" plus a wrapper.
* **(d) `update_readonly` / `update`.** Rejected in favour of `update` / `update_editable` (the phrasing §21 item 1 itself used): `update` and `draw` then share one bound; read-only call sites outnumber editable ones across the three consumers; the capability is named where the extra capability is used, matching the `read_only`/`disabled` conventions.

**Invariants.**

* **G1** `Grid::update` takes `&M`. A read-only grid is structurally incapable of mutating its model. `draw` and `update` carry the same bound and the same shared borrow.
* **G2** `Grid::update_editable` is the **only** place `GridEditor`'s `&mut self` methods are reachable. With `draw`'s `&self`/`&GridState`, "rendering stages a database mutation" (**[F]** `grid.rs:1518-1520`) is unrepresentable — the §21 item 1 / B15 rationale is preserved intact.
* **G3** `read_only_reason` and `actions` live on `GridModel` because `draw` renders them.
* **G4** No boolean capability parameter exists on `Grid`; `Grid::editable(bool)` is deleted (`architecture::no_boolean_capability_parameter_on_grid`, §16.5).
* **G5** `EditIntent::External` emits `GridAction::EditRequested(item, col)` and begins no inline edit (§21 item 30 A8) — reachable only from `update_editable`.
* **G6** An inline editor registers its `Control` region **after** the grid's cell `Part` region and wins the click (§21 item 30) — unchanged.
* **G7** Both entry points call `GridState::reconcile` before emitting any action (§12.2).

**Acceptance conditions (executable).**

```bash
cargo test -p junie-tui --lib grid::
cargo test -p junie-tui --test conformance conformance::grid::
cargo test -p junie-tui --test architecture
cargo test -p tablepro tablepro::grid_adapter_keeps_every_pending_change_capability
cargo test -p junie-tui --test perf --release -- --test-threads=1 grid_
! rg -n 'fn editable\(' crates/tui/src/components/grid.rs
! rg -n 'trait GridCellActions' crates/tui/src
```

Named tests (in §16.1 / §16.2 / §16.4 / §16.5): `grid::read_only_update_takes_a_shared_model` (a `trybuild` compile-fail proving `Grid::update` cannot reach `commit_cell`/`apply_cycle`), `grid::update_editable_commits_through_the_editor`, `grid::read_only_reason_is_rendered_from_a_grid_model` (fails the pre-K2 §12.3 as written), `grid::cell_actions_affordance_is_painted_for_a_read_only_model`, `grid::edit_intent_inline_cycle_external_refuse` (retained; now reached only via `update_editable`), `grid::sort_is_a_permutation_and_edits_stay_bound_to_the_source_row` (retained), `grid::click_inside_an_active_inline_edit_goes_to_the_editor` (retained), `conformance::grid::draw_does_not_commit_or_cancel` and `conformance::grid::item_identity_survives_reorder` (both entry points from `Fixture.read_only`), `architecture::no_boolean_capability_parameter_on_grid`, `tablepro::view_grid_is_read_only_with_a_reason`, `tablepro::result_grid_sorts_locally_and_refuses_edits`.

**Risks.**

1. **A model that later becomes editable changes call site, not type.** A screen upgrading a grid from read-only to editable switches `update` → `update_editable` and holds `&mut`. This is the intended, visible cost; a defaulted-refusal design hides it and lets a half-implemented editor ship.
2. **Two entry points means two `GridAction` paths to keep in sync.** Mitigated structurally: `update_editable` is `update`'s body plus the edit arms over one private `fn navigate(…)`, and conformance runs the whole matrix through both.
3. **`GridModel` widens to 9 methods (5 defaulted).** Acceptable — the shape §12.3 already chose for `row_decor`/`cell_decor`/`total`/`has_more` — and the only way `draw` can render either.
4. **`GridCellActions` deletion is a documented API change** relative to the pre-K2 §12.3 and DOM §1.5 (`docs/audit/domain-boundary-audit.md:147-149`). Recorded as an amendment in §12.3 and §18.2; must be mirrored in `REFACTORING_STATE.md` (coordinator-owned) before work package 4I starts, per the change-control rule at the top of this document.

**Sequencing.** Neither decision blocks Slice 3 (§21 item 1 already records this for K2). K1 blocks **4F**, K2 blocks **4I** — both wave 2 (Appendix A). Both are mirrored in `REFACTORING_STATE.md` before wave 2 begins.

## 24. Adjudication M — re-exports, ASCII border set, `FieldKind`

**Status:** Accepted. Source: `docs/reviews/adjudication-m-small-items.md` (ADJ‑M). Resolves the two items §22.10 left open and §23.1's open item 1. Nothing here reopens Adjudications A–L. Each inline edit carries `<!-- amended by §24 -->`.

**Amends:** §0.1, §6.1 (three action enums declared by the §17 self-check), §10 (nothing — `Size` confirmed unchanged; one clarifying sentence), §11.2, §11.7, §12.2, §15.1, §16.1, §16.5, §17.0 A2/A5/A7/A10, §17 examples 2 and 13, §18.1, §18.2, §19, §21 item 1 (clarified, not changed), §22.10, §23.1, Appendix B.2, Appendix B.4.

**Facts the three decisions rest on** (ADJ‑M §0). **[F]** `ratatui_core::layout::Size` is `{ width: u16, height: u16 }` (`ratatui-core-0.1.2/src/layout/size.rs:1-40`); ours is `Size { min, preferred }` (§10) — different shape, same name. **[F]** `ratatui_core::text` exports `Line`, `Span`, `Text`, `Masked`, `StyledGrapheme`, `ToLine`, `ToSpan`, `ToText` (`text.rs:51-64`); ours is the role-carrying `Span<'a>` §22.2 item 16 keeps. **[F]** `Frame::area() -> Rect`; `Frame` has no `Size` (`terminal/frame.rs:60-70`); `Frame`'s only appearance in our surface is `Runtime::draw(&mut self, frame: &mut Frame<'_>)` (A1). **[F]** `symbols::border::Set<'a>` is a plain struct with eight `pub` `&'a str` fields deriving `Debug, Clone, Copy, Eq, PartialEq, Hash` (`symbols/border.rs:3-19`); consts shipped: `PLAIN`, `ROUNDED`, `DOUBLE`, `THICK`, six dashed sets, `QUADRANT_*`, `ONE_EIGHTH_*`; **there is no `+-|` ASCII set**; `Default` is `PLAIN`. **[F]** `Capability` has exactly one field, `color: ColorLevel` (§21 item 19). **[F]** `Theme::paper()` already specifies `PLAIN` (§11.7) and §11.2 records Junie's set as `ROUNDED`. **[F]** Pre-§24, `FieldKind<'a>` wrapped `Select<'a>` / `RadioGroup<'a>` / `ChipBar<'a>` without collection generics and A10 gave `Select::new(id, options: &'a [&'a str])`, while §18.2 typed them `<'a,T,K,R>` and §21 items 1 and 5 put them under the per-phase item channel and the three-impl-block scheme.

### 24.1 M1 — Root and `author` re-export set

**Decision.** **Neither of our types is renamed and neither colliding ratatui type is exported.** MOD §1.2's binding rule is *"`junie-tui` re-exports the ratatui types the public API mentions"*; its parenthetical list predates §22.10's finding that two of its eleven names are already ours. Applying the rule rather than the list:

* `ratatui_core::layout::Size` — **not exported anywhere.** No `pub` signature names it (`Frame::area()` returns `Rect`; resize enters as `Input::Resize{w,h}`; `Backend::size`/`WindowSize` never leave `runtime/session.rs`). Our `Size` keeps the name at the root and in `author` (§10, unchanged).
* `ratatui_core::text::{Line, Span, Text}` — **not exported at the root and not exported flat in `author`.** No `pub` signature names them: `RowUi::label_spans` builds its borrowed `Line<'_>` *inside* `crates/tui/src/ui/paint.rs`. Our `Span<'a>` keeps the name.
* Everything the surface does name is re-exported at the layer that names it: `Buffer`, `Cell`, `Position`, `Rect`, `Color`, `Modifier`, `Style` and our `Span` at **both** root and `author` (`Ui` is a root type whose `raw()` names `Buffer`, so an app must be able to name it without reaching into `author`); `Frame` at the **root only** (a component author receives `Ui`, never a `Frame`, for the same reason B.4 excludes `Runtime`/`run`); the `theme` module (with `theme::border`) at the root and `border` in `author`.
* `Ui::raw()`/`RowUi::raw()` hand out `&mut Buffer`, whose `set_line`/`set_span` would otherwise be unreachable. Two closures, both cheap: `Ui::paint_spans` in our own vocabulary (A2), and the qualified-only escape module `author::raw::{Line, Span, Text}` — the only re-export not forced by a signature.
* `Span<'a>` moves from `components/viewport.rs` to `crates/tui/src/text/span.rs`: `RowUi` lives in `collection/`, and a `collection → components` dependency for a type name is backwards; `TextViewport` and `RowUi` are both consumers.

**Exact Rust.**

```rust
// crates/tui/src/lib.rs — the application-author facade (B.3 item 2: one curated line each)
pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Modifier, Style};
pub use ratatui_core::terminal::Frame;          // named by Runtime::draw (A1) — host concern, root only
pub use crate::text::Span;                      // OURS: role-carrying, used by RowUi::label_spans
pub mod theme;                                  // theme::border (M2), ColorTokens, Theme, …

// crates/tui/src/theme/border.rs — reachable as junie_tui::theme::border and junie_tui::author::border
pub use ratatui_core::symbols::border::{Set, DOUBLE, PLAIN, ROUNDED};
pub type BorderSet = Set<'static>;              // §11.2, unchanged

// crates/tui/src/author.rs
pub mod author {
    // …everything already listed in Appendix B.4, plus:
    pub use crate::text::Span;                  // OURS
    // `Frame` is NOT re-exported here (root only).
    /// Types needed only to drive the `Ui::raw()` / `RowUi::raw()` escape hatch.
    /// The ONLY re-export not forced by a signature. `raw::Span` is ratatui's
    /// style-carrying span and is written qualified, always: `raw::Span`.
    pub mod raw {
        pub use ratatui_core::text::{Line, Span, Text};
    }
}

// crates/tui/src/ui/paint.rs — new, so the role-carrying Span is the only span in normal use (R‑3)
impl Ui<'_> {
    /// Multi-style single-line paint. Resolves each `Span`'s `Role` against the live
    /// theme and surface and writes it through `Buffer::set_span`, span by span.
    /// Returns columns written.
    /// <!-- amended by §25 D‑13, F4 --> Two corrections to this signature: it takes a third
    /// `base: Style` (the part style the spans inherit — without it `RowUi::label_spans`
    /// could not honour the `LABEL` recipe), and it must NOT collect a `Vec<RawSpan>` to
    /// hand `Buffer::set_line`, which allocates once per call on the row path (§20.9-6, R5).
    pub fn paint_spans(&mut self, area: Rect, spans: &[Span<'_>], base: Style) -> u16;
}
```

**Mechanical proof that "apps depend on `junie-tui` only" stays true.** `architecture::applications_depend_only_on_the_library_facade` (§16.5) proves the *dependency edge*; it does not prove the facade is *complete* — one signature naming an unexported type forces an app back to a `ratatui-core` line. Closed by **`architecture::every_foreign_type_in_the_public_surface_is_re_exported`** (`xtask`, rustdoc-json): for every non-local type named in a `pub` item reachable from `junie_tui::`, a `pub use` path exists under `junie_tui::`; likewise for `junie_tui::author::`. Failure prints the type, the signature that names it and the missing facade line. This makes the decision self-maintaining: the day someone puts a `Line` in a signature, the check fails and the exporting decision is forced rather than discovered downstream.

**Rejected alternatives.**

* **Rename ours (`Measured`/`Extent` for `Size`, `RoleSpan` for `Span`).** Renames the type used by `App::min_size`, `Measure::measure`, every component's `measure` and `label_spans` — the whole public vocabulary — to make room for two types no signature mentions; `RowUi::label_spans(&[RoleSpan])` also reads worse. Cost with no benefit.
* **Rename theirs on export (`TermSize`, `TextSpan`/`StyledSpan`).** A name that exists in no upstream doc, error message or answer, for types that only ever appear behind `raw()`; does not reduce the count of exported names.
* **A single `junie_tui::tty` / `junie_tui::ratatui` submodule.** `tty` is a lie (text and geometry types, not terminal-session types); `junie_tui::ratatui::` trips §22.7 forbidden-pattern rule 14 (`\bratatui::`) at every use site — a mechanical conflict with an accepted gate. `author::raw` says exactly what it is for and collides with nothing.

**Tests.** `architecture::every_foreign_type_in_the_public_surface_is_re_exported`, `architecture::applications_depend_only_on_the_library_facade` (existing, unchanged), `ui::paint_spans_matches_row_ui_label_spans` (differential over the §16.1 text corpus).

### 24.2 M2 — The ASCII border set

**Decision.** **Keep ASCII, as a plain `const` of the foreign type, in `junie_tui::theme::border`.** A `const` of a foreign type is not an `impl` and is subject to no coherence rule; `Set`'s eight fields are all `pub` **[F]**. The `BorderSet` alias stays exactly as §11.2 and §22 R‑11 wrote it. `Theme::junie()` uses `border::ROUNDED`; `Theme::paper()` uses `border::PLAIN`; `border::ASCII` is used by neither builtin — it is opt-in through the already-specified `ThemeBuilder::borders_set` (A5). **`Capability` has no `unicode` field** **[F]**, so **nothing selects ASCII automatically**, and nothing should.

**Exact Rust.**

```rust
// crates/tui/src/theme/border.rs
pub use ratatui_core::symbols::border::{Set, DOUBLE, PLAIN, ROUNDED};

/// Pure-ASCII border set, for terminals and fonts without box-drawing glyphs.
/// Not shipped by ratatui; declared here as a plain `const` because `BorderSet`
/// is a type alias of a foreign type and can carry no inherent items (§11.2).
/// Opt in with `Theme::junie().builder().borders_set(border::ASCII).build()`.
pub const ASCII: Set<'static> = Set {
    top_left:         "+", top_right:         "+",
    bottom_left:      "+", bottom_right:      "+",
    vertical_left:    "|", vertical_right:    "|",
    horizontal_top:   "-", horizontal_bottom: "-",
};
```

`Theme::junie()` → `design.borders = border::ROUNDED`. `Theme::paper()` → `design.borders = border::PLAIN` (§11.7, unchanged).

<!-- amended by §27 (Adjudication O2) --> **`borders_set(border::ASCII)` also applies `ThemeBuilder::ascii_glyphs()`.** `Ui::rule` reads `GlyphRole::RuleQuiet` from `design.glyphs`, never from `design.borders`, so a border-only swap leaves `─` in every divider and `│`/`┃` in every scrollbar — precisely the outcome reason (2) below rejects automatic selection for. The rebinding is not a widening of M2's scope: `RuleQuiet`, `RuleActive`, `ScrollTrack` and `ScrollThumb` are **exactly** the `GlyphRole`s whose Junie binding falls in `U+2500..=U+257F`, verified role by role over all 39 array entries and the typed sets — every other Junie glyph (`▎`, `›`, `✓`, `●`, `•`, `−`, `▲`, `▸`, `▾`, `▴`, `∇`, `▪`, `→`, `↓`, `‹`, `…`, `×`, `◆`, `◇`, `∥`) lies outside that block. `ascii_glyphs()` is a **public, named, idempotent** step that replaces the **whole typed sets** — `glyph::ASCII_RULE_QUIET` (`-`), `glyph::ASCII_RULE_ACTIVE` (`=`, which must read heavier than quiet and does so in monochrome, where weight is the only channel left), `glyph::ASCII_SCROLLBAR` (`|` track and caps, `#` thumb, an unambiguous density step over the track) — so `scrollbar::Set.begin`/`.end` and `line::Set`'s ten seam fields, which **no** `GlyphRole` names and which no `.glyph(..)` call could otherwise reach, are covered without adding a `GlyphRole` (rejected: it widens `GlyphRole::ALL` to 41 and touches every `GlyphSet` literal, to solve by enumeration what a whole-set setter solves structurally). Ordering is last-write-wins and now documented on a method that *says* it changes glyphs: `.borders_set(ASCII).glyph(RuleQuiet, "~")` keeps `"~"`; the reverse order discards it; and `.borders_set(ASCII).borders_set(PLAIN)` keeps the ASCII rules, which is accepted and documented rather than fixed, because restoring the theme's own glyphs would clobber a deliberate `.glyph(..)`. Narrowing `theme::ascii_theme_renders_without_box_drawing_glyphs` to border cells was the alternative and is **rejected**: the whole-frame scan is the only mechanism that made the coupling visible.

**Rationale.** Three glyphs, eight fields, ten lines, zero new types, and `Set: PartialEq` **[F]** makes the builtins assertable. §11.2's declared triple (`rounded | square | ascii`) survives without the deleted `UnicodeLevel` returning. Automatic selection is rejected on three grounds: (1) it needs runtime terminal-capability detection, which contradicts §16.4's determinism claim for the same reason §22.2 item 6 rejected keyboard-enhancement flags; (2) borders are 8 of ~35 glyph slots — a border-only auto-switch renders a frame that is ASCII at the edges and unicode everywhere else (`GlyphRole::{Chosen, Checked, SortAsc, FollowRef, …}`), worse than either consistent choice; (3) it would reintroduce the `Capability` axis §21 item 19 deliberately deleted, and every §16.3 baseline would fork by terminal.

**Deferred root cause, named.** If unicode-capability fallback is ever wanted, the correct shape is a `Capability` field **plus** a full `GlyphSet` fallback table applied the way §11.4's `apply_mono_fallbacks` applies colour fallbacks, plus re-blessed baselines under §20.10. That is a fresh adjudication, not a border const. `border::ASCII` is the manual, deterministic, theme-author-visible half of it and does not prejudge it. <!-- amended by §27 (Adjudication O2) --> The full `GlyphSet` ASCII fallback table is **scheduled for Slice 4E, not deferred indefinitely.** 4E ships `ScrollRegion`, the first component to paint `scrollbar::Set.begin`/`.end`; until O2's whole-set swap those two glyphs were unreachable by any `GlyphRole` and would have made `theme::ascii_theme_renders_without_box_drawing_glyphs` fail the day 4E landed — the new `theme::ascii_glyph_set_has_no_box_drawing` converts that latent 4E failure into a Slice-3 one, deliberately. `ascii_glyphs()` covers the box-drawing block completely; the remaining ~31 roles (`›`, `✓`, `▎`, `…`, `×`, `▸`, the spinner frames) are a **visual-design** decision against `DESIGN.md`'s marker table and belong with 4E's own review, together with re-blessed baselines under §20.10. Scheduling it now in Slice 3 is rejected for the reason M2 already gave; leaving it open-ended is rejected equally — 4E is a dated forcing function. `Capability` still gains no unicode axis (§21 item 19); the table stays a manual, theme-author-visible opt-in.

**Rejected alternatives.**

* **Drop ASCII.** Leaves no supported way to theme for a terminal without box-drawing glyphs, while the mechanism (`borders_set`) already exists and is already tested — dropping saves ten lines and removes a capability §11.2 declared.
* **`pub struct BorderSet(Set<'static>)` newtype.** Re-adds the wrapper §22.2 item 12 just deleted; forces `.0` or a `Deref` at every read; breaks `ThemeBuilder::borders_set(border::PLAIN)`; buys only the ability to hang inherent consts, which a module of consts already provides with better `use` ergonomics.

**Tests.** `theme::ascii_border_set_is_pure_ascii` (each of the eight fields satisfies `s.is_ascii() && s.len() == 1`, which also pins `text::width(s) == 1`), `theme::builtin_border_sets_are_ratatui_sets` (`assert_eq!(Theme::junie().design.borders, border::ROUNDED)`, `assert_eq!(Theme::paper().design.borders, border::PLAIN)`), `theme::ascii_theme_renders_without_box_drawing_glyphs` (<!-- amended by §27 (Adjudication O2) --> a whole-frame `Harness::text()` scan, ~~a `Scene` digest~~, over `Theme::junie().builder().borders_set(border::ASCII).build()`, painting a frame **and** a `ui.rule(..)`, contains no `U+2500..=U+257F`), <!-- amended by §27 --> `theme::ascii_glyph_set_has_no_box_drawing` (every `GlyphRole::ALL` binding plus every field of the typed `scrollbar()`, `rule_quiet()` and `rule_active()` sets), `theme::builder::ascii_glyphs_is_idempotent_and_glyph_overrides_it`, `architecture::capability_has_no_unicode_field` (rustdoc-json: `Capability`'s field set is exactly `{color}`).

### 24.3 M3 — `FieldKind` versus the collection generics

**Decision — option (a), with the option list on the data channel, not on the props.** **`Select`, `RadioGroup` and `ChipBar` keep `<'a, T, K, R>` and the per-phase item channel, exactly like every other collection (§18.2, §21 items 1 and 5, unamended). `FieldKind<'a>` stays a closed, non-generic enum holding the default instantiation `<'a, &'a str, ByIndex, DefaultRow>`. The option list a form field needs reaches the control through `FormData`, which is already the form's single data channel.** This is the only reconciliation that changes neither §21 item 1 nor §18.2 nor `FieldKind`'s shape; it costs two defaulted `FormData` methods and three type aliases.

**Exact Rust.**

```rust
// ── §17.0 A7 / §18.2: the three controls are ordinary collections ────────────
impl<'a, T> Select<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }   // items are per phase (A3)
impl<'a, T, K, R> Select<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::FIELD, Part::LABEL, Part::MARKER, Part::ROW,
                                         Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Select<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Select<'a, T, K, R2>;
    pub fn placeholder(self, s: &'a str) -> Self;
    pub fn popup_rows(self, n: u16) -> Self;
    pub fn read_only(self, yes: bool) -> Self;   pub fn disabled(self, yes: bool) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> Select<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SelectState, items: &[T]) -> Response<SelectAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
// identical three-block shape for RadioGroup<'a,T,K,R> and ChipBar<'a,T,K,R> (§21 item 5; written out in A7)

// ── §15.1 / §17.0 A10: FieldKind stays closed and non-generic ────────────────
pub type LabelSelect<'a> = Select    <'a, &'a str, ByIndex, DefaultRow>;
pub type LabelRadio <'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;
pub type LabelChips <'a> = ChipBar   <'a, &'a str, ByIndex, DefaultRow>;

pub enum FieldKind<'a> {
    Text   (TextInput<'a>),
    Area   (TextArea<'a>),
    Select (LabelSelect<'a>),
    Radio  (LabelRadio<'a>),
    Chips  (LabelChips<'a>),
    Check  (Checkbox<'a>),
    Toggle (Toggle<'a>),
    Chooser(Button<'a>),
    Note,
}

// ── §15.1: the option list is data, and travels with the value ───────────────
pub trait FormData {
    fn value    (&self,     id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;

    /// Option labels for a `Select` / `Radio` / `Chips` field; `&[]` for every other kind.
    /// Borrowed from the caller — never `'static` (§21 item 22). Painted, never returned
    /// in a `FormAction` (F5).
    fn options(&self, _id: Id) -> &[&str] { &[] }

    /// The controlled value and the option list under ONE borrow, so `Form::update` can
    /// drive a choice control without a second `&mut` (E0502). A data type with option
    /// tables overrides it by destructuring its own disjoint fields.
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), &[])
    }

    fn visible (&self, _id: Id) -> bool { true }
    fn disabled(&self, _id: Id) -> bool { false }
    fn error   (&self, _id: Id) -> Option<&str> { None }
    fn validate(&self, _id: Id, _v: FieldRef<'_>) -> Result<(), FieldError> { Ok(()) }
    fn validate_all(&self) -> Result<(), (Id, FieldError)> { Ok(()) }
}
```

`Form::draw` calls `data.value(id)` + `data.options(id)` (two shared borrows). `Form::update` calls `data.value_and_options(id)` (one mutable borrow). `FieldSpec::new` stays `const fn` — `Select::new(id)` is `const`-constructible.

**Amendments this forces.** §17.0 A10's `Select::new(id, options)` and `RadioGroup::new(id, options)` become `new(id)` (declared once, in A7); §17 example 13's `conn_fields` loses its `engines/envs/groups/modes` parameters (they move to `ConnDraft::options` / `value_and_options`) and drops `use FieldKind::*;` in favour of spelled-out `FieldKind::Select(…)` — the glob shadowed the imported `Select` type in the value namespace and compiled only by an explicit-beats-glob accident.

**Invariants.**

* **M3‑1 — `FormState` stores no props.** `FieldSlot`'s value is `enum SlotValue { None, Text, Choice(usize), Flag(bool), Chips(KeySet) }`, `Clone + PartialEq + Eq`; text drafts live in the slot's `TextInputState`/`TextEditorCore` (manual redacting `Debug`, `zeroize`), never as a `Secret` field, so `FormState: Clone + PartialEq + Default` holds (S2, conformance case 6) and no `FieldKind` — which holds `&dyn Fn` slots and is neither `Clone` nor `PartialEq` — is ever reachable from state.
* **M3‑2 — Props are built once and hold no values.** `Form<'a>` holds `&'a [FieldSpec<'a>]` and scalars (F1). `FieldKind` holds control *configuration* only; the item slice never enters it, so §21 item 1's B3 borrow hazard cannot arise at a form field.
* **M3‑3 — The form's identity model is positional by construction.** `FieldMut::Choice(&mut usize)` / `FieldRef::Choice(usize)` are index-based, so a keyed `Select` inside a `FieldKind` would have no channel to report its `ItemKey`. `ByIndex` in the aliases is therefore forced by the value channel, not a default that leaked. Documented consequence: a form whose option table is reordered *at runtime* must map indices itself, or use `FieldKind::Chooser`.
* **M3‑4 — `Chooser` is the escape hatch for a richer control.** A field needing a keyed, custom-row, non-string collection is `FieldKind::Chooser(Button<'a>)`: it emits `FormAction::Chose(id)` and the owner opens its own `Picker`/`Select` layer with the full `<T,K,R>` surface. No new API; already in §15.1 and example 13's shape.

**Rejected alternatives.**

* **(b1) Generic `FieldKind`.** A closed enum cannot carry a *per-variant* type parameter; it would need nine (`FieldKind<'a, TS,KS,RS, TR,KR,RR, TC,KC,RC>`), and every element of a `&'a [FieldSpec<'a>]` must be the same instantiation — so a form could not hold two `Select`s over different item types. Fatal, not merely ugly. It would also put type parameters into `Form`, `FormState` and every screen's field-array signature, which §13 forbids and §11.1 already rejected for themes.
* **(b2) `FieldKind::Control(&'a dyn FieldControl)`.** `FieldControl` has an associated type `State` (§15) and is not dyn-safe; erasing `State` erases the very thing `FormState.slots` keys (`TextInputState` vs `SelectState`), breaking M3‑1. Even erased, `Form` still needs a discriminant for each field's *value shape* — which is the enum, re-added beside the trait object. Collapses to (a) plus indirection.
* **(a′) `FieldKind` variants hold the option slice (`Select(LabelSelect<'a>, &'a [&'a str])`).** Puts data back in props, which §21 item 1 removed for exactly this class, and re-opens the disjointness question (the props array and `&mut D` must borrow disjoint fields of the screen). `FormData` already carries per-field data keyed by `Id`; a second parallel channel is the API-inconsistency class G1 removes.
* **(a″) `Select::labels(id, &[&str])` as a separate constructor.** Redundant: `Select::new(id)` is already the non-generic default instantiation and `T = &'a str` is inferred from the `FieldKind::Select` variant. One name, not two.

**Tests.** `form::select_field_options_come_from_form_data`, `form::changing_options_between_frames_does_not_rebuild_props`, `form::state_holds_no_props` (static assertion: `FormState: Clone + PartialEq + Default`; `SlotValue: Clone + PartialEq + Eq`), `form::value_and_options_is_a_single_borrow` (a `Form::update`-shaped body over one call compiles; the two-call body is a `trybuild` compile-fail, E0502), `form::chooser_activation_emits_chose_with_the_field_id` (retained from §15.1, now also the escape-hatch proof), `architecture::field_kind_has_no_type_parameters` (rustdoc-json: one lifetime, zero type params), `select::standalone_select_takes_items_per_phase`, `select::escape_closes_and_restores_the_cursor` (retained), `choice::radio_group_separates_cursor_from_value` (retained).

### 24.4 Declared by the §17 reference self-check (recorded, not decided)

Re-running the self-check over §17 examples 1–13 after the amendments found no reference an example uses that the document does not declare. Applying M3's "identical three-block shape" to `RadioGroup` and `ChipBar` in A7 required three names §6.1's exact action set and the §18.2 state list had not spelled out; they are declared, not adjudicated, and follow existing conventions only: `SelectAction { Chose(ItemKey), Opened, Closed }`, `RadioGroupAction { Chose(ItemKey) }`, `ChipBarAction { Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey) }` (§6.1; the `XAction` naming of §13.2), the contents of `SelectState` / `RadioGroupState` / `ChipBarState` (A7; the state-holds-no-props rule of §4), and `RadioGroup::PARTS` / `ChipBar::PARTS` over existing `Part` constants only (`CONTAINER, GUTTER, MARKER, LABEL` / `CONTAINER, LABEL, CLOSE, OVERFLOW`). The 4A/4B owners may refine the variant lists under §13.2's rustdoc template; a change to the *names* is an Opus decision.

### 24.5 Acceptance conditions (executable)

```bash
cargo check -p junie-tui --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo build -p junie-tui --examples                     # examples 2 and 13 compile as amended
cargo tree  -p showcase -e normal --depth 1             # => junie-tui, and nothing else

cargo test --workspace --test architecture every_foreign_type_in_the_public_surface_is_re_exported
cargo test --workspace --test architecture capability_has_no_unicode_field
cargo test --workspace --test architecture field_kind_has_no_type_parameters
cargo test -p junie-tui --lib theme::ascii_border_set_is_pure_ascii
cargo test -p junie-tui --lib theme::builtin_border_sets_are_ratatui_sets
cargo test -p junie-tui --lib ui::paint_spans_matches_row_ui_label_spans
cargo test -p junie-tui --lib form::
cargo test -p junie-tui --test conformance conformance::form::

# M1: neither colliding ratatui type is re-exported, and no umbrella path appears
! rg -n 'pub use ratatui_core::(layout::\{[^}]*\bSize\b|text::\{)' crates/tui/src/lib.rs
# <!-- amended by §25 §3 --> the whole-file negative is wrong: `raw` is an INLINE module inside author.rs and
# legitimately contains that line. `xtask` asserts it with `syn`, scoped to the `author` module's own items.
  cargo run -p xtask -- boundary   # author_re_exports_no_ratatui_text_outside_raw
  rg -n 'pub mod raw' crates/tui/src/author.rs
! test -e crates/tui/src/author/raw.rs
# M2: ASCII is a const, never an impl on the alias
! rg -n 'impl (BorderSet|border::Set)' crates/tui/src
  rg -n 'pub const ASCII: Set' crates/tui/src/theme/border.rs
# M3: no items in a form control's constructor; no glob over FieldKind
! rg -n 'fn new\(id: Id, (options|labels|items): &' crates/tui/src/components/{select,choice,chip}.rs
! rg -n 'use FieldKind::\*' crates/tui/examples
```

Pass condition: all commands succeed, the four `!`-prefixed greps return no matches, and `crates/tui/tests/allow/legacy_api.txt` stays empty. During Slices 3–4 the crate is `tui-next` (§21 item 31), so `-p junie-tui` reads `-p tui-next`. ~~**File placement forced by the two `author.rs` greps (recorded, not decided):** … ADJ‑M's inline `pub mod raw { … }` sketch is therefore realised as a file-backed module.~~ <!-- amended by §25 §3 --> **Struck.** `author::raw` is an **inline** `pub mod raw` inside `crates/tui/src/author.rs`, as §24.1's exact Rust already writes it; there is no `author/raw.rs` file, and Appendix B.2's earlier line was the outlier (corrected). The greps were what forced the wrong conclusion, so the negative one is replaced: the check is that the `author` module's **own** re-export list names no `ratatui_core::text` item — a `syn`-scoped assertion in `xtask`, not a whole-file `rg`, because the nested `raw` module legitimately contains exactly that line. Nothing about the path `junie_tui::author::raw::{Line, Span, Text}` changes.

### 24.6 Risks

1. **`Frame` at the root puts a `ratatui-core` type in our published surface.** Already true of `Rect`/`Style`/`Color`, so it adds no new class of exposure; `cargo-semver-checks` from `v0.1.1` (§22.3) is the detector. If ratatui-core ever moves `Frame`, one facade line changes and `every_foreign_type_in_the_public_surface_is_re_exported` fails loudly rather than an app's build breaking.
2. **`author::raw` re-exports a second `Span`.** Mitigated by module qualification (`raw::Span`, never a flat `use`) and by `Ui::paint_spans`, which removes the only realistic reason to reach for it. If `raw::Span` starts appearing in components, that is a signal `paint_spans` is under-specified — not a naming problem.
3. **`border::ASCII` themes still get unicode `GlyphSet` glyphs.** Stated in the const's rustdoc and above as the deferred root cause. `theme::ascii_theme_renders_without_box_drawing_glyphs` scans only the border range, deliberately, so it does not create a false impression of full ASCII safety.
4. **`FormData` widens to seven methods (five defaulted).** Same shape §12.3 chose for `GridModel` and §23 K2 reinforced. `value_and_options`'s default is correct for every kind except the three choice kinds, so a data type that forgets to override it renders an empty option list — visible immediately in `form::select_field_options_come_from_form_data` and in the first frame, not a silent wrong answer.
5. **Example 13 and §17.0 A7/A10 change text.** Both are gated by `xtask doc-check` (§21 item 34) and `architecture::all_examples_compile`, so the amendment is verified rather than asserted; `REFACTORING_STATE.md` (coordinator-owned) must mirror the three decisions before work packages 4A/4B/4F/4G start, per the change-control rule at the top of this document.

**Sequencing.** M1 and M2 land in Slice 3 (`lib.rs`, `author.rs`, `text/span.rs`, `theme/border.rs`, `ui/paint.rs` are foundation files); M3 blocks **4B** (`select.rs`), **4A** (`choice.rs`, `chip.rs`) and **4F** (`form.rs`), all of which are already sequenced after Slice 3 (Appendix A).

---

## 25. Slice 3 foundations review — accepted adjudications and deviations

**Source.** `docs/reviews/slice3-foundations-review.md` — a fresh read-only `opus-analyst` review of `crates/tui` (package `tui-next`, lib `tui_next`), `crates/tui-testing`, `xtask`, `crates/tui/tests/**`, `crates/tui/examples/12_author_component.rs` and `crates/tui/README.md` at commit `18afddd`, read against §3–§13, §16, §17.0, §21–§24, Appendix B and `docs/audit/modern-api-audit.md` §1–§2. **Accepted as written.** This section records it; it does not re-decide it. Every earlier section it changes carries an inline `<!-- amended by §25 -->` marker.

**Verdict recorded.** *Components may build on this surface: **no** as it stands; **yes** after the seven blockers and the eight adjudications are applied.* The intent queue, focus ring, capture, scroll, layout, reconcile core and the conformance driver are ready. Seven defects are load-bearing for Slice 4 and live in files Slice 3 owns, so a 4x owner cannot fix them; four document amendments are required so the gates stop asserting things that are false. Numeric colour claims the review marks *(estimate)* were hand arithmetic and must be re-derived before blessing.

### 25.1 Adjudication 1 — `ratatui-crossterm` as a normal dependency: **CONFIRMED**, with a gate that bites

The dependency stands. The alternatives stay rejected: **owning `KeyCode`/`KeyModifiers`** (≈40 hand-written variants plus `From` impls, and it loses crossterm's `PartialEq`/`Hash` ASCII-case normalisation that `Chord: Eq + Hash` relies on, §22.2 item 6) and **gating `Intent::Key`** (it would make the shape of `Intent`, `Chord`, `Binding`, `KeyMap` and `BindingState` depend on a feature — a core whose intent enum changes shape under `--no-default-features` is not a core).

What was false was the *claim* and the *gate*, not the dependency. Recorded, and applied to §22.1's consequences bullet, §16.5's `core_is_backend_free` row, §18.1's `runtime.rs` row and Appendix B.2's manifest:

> `ratatui-crossterm` is a **normal, non-optional** dependency of `junie-tui`, taken for its version-unified `crossterm` re-export — the key vocabulary `Key`/`Chord`/`KeyMap` name (§6.1, R‑14) — never for `CrosstermBackend`. The `crossterm` feature gates the *terminal session* (`TerminalSession`, `run`, `DefaultTerminal`) and nothing else; `crossterm = []` is therefore the correct manifest form, not `crossterm = ["dep:ratatui-crossterm"]`. `cargo check -p junie-tui --no-default-features` remains a gate: it proves that nothing outside `runtime/session.rs` needs a backend. The stronger claim — that the widget layer is backend-independent — is proved by forbidden-pattern **rule 27** (`CrosstermBackend|ratatui_crossterm::(?!crossterm::event)` allowed only in `crates/tui/src/runtime/session.rs`) and by `architecture::ratatui_crossterm_is_named_in_exactly_two_files` (`src/event.rs` for the `crossterm::event` vocabulary, `src/runtime/session.rs` for the backend; nowhere else).

`Input::from_crossterm` is **not** feature-gated: it needs no backend (§18.1).

### 25.2 Adjudication 2 — `Id` structural equality: **CONFIRMED**, and the vacuous test is replaced

Structural derives are **required**, not a convenience: a `const Id` used as a `match` pattern needs `StructuralPartialEq`, which §15.1's `FormData::value(id) → match id { NAME => … }` and example 13 depend on. Every field is structural-match (`u64`, `&'static str`, `Part(u16)`, `ItemKey`, `usize`). §7.1's manual hash-only impls are replaced by

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id { hash: u64, #[cfg(debug_assertions)] label: DebugLabel }
```

**Rationale (recorded, because it is the reason debug and release cannot disagree).** The debug label is a pure function of the segments that produced the hash: `root` is carried unchanged from the root segment and `tail` records the last segment, both determined by the segment chain the hash was computed over. Therefore equal hashes ⇒ equal segment chains ⇒ equal labels, and debug and release compare identically. Only a genuine FNV collision could differ, and there the label is the more honest answer.

**Test.** `id_equality_ignores_debug_label` as written compared two ids with *identical* labels and proved nothing. It is replaced by `id_equality_is_exactly_hash_equality`, over a corpus of ~200 ids built by every derivation:

```rust
for a in &corpus { for b in &corpus {
    assert_eq!(a == b, a.hash() == b.hash());
    assert_eq!(a.cmp(b), a.hash().cmp(&b.hash()));
}}
```

The old name survives only as a `#[cfg(debug_assertions)]`-only assertion that a label never changes an answer. No code change to `id.rs` itself.

### 25.3 Adjudication 3 — Ansi16 downgrade: **CIE76 REJECTED**, the legacy categorical metric restored

**Decision: revert `nearest_16` to the legacy categorical metric, verbatim.** Four reasons, in order of weight.

1. **Authority.** The order at the top of this document is `REFACTORING_GOAL.md › DESIGN.md › existing rendered output/tests › current source`. `DESIGN.md:320` states *"At 16 colours the accent is LightGreen and error is LightRed"* and the legacy test pins it. §21 item 29's "CIE76" is implementation spec, which is subordinate, and §20.10 lists no 16-colour change — so CIE76 is a **regression** by this document's own definition, not a decision.
2. **The metric answers the wrong question.** A 16-colour downgrade must preserve *hue identity and brightness class*, not minimise perceptual distance. ΔE minimisation demonstrably loses both: it maps Junie's accent and error into the **dark** half, so "the accent is the brightest signal on screen" — the property the whole accent system rests on — is gone, and `danger_soft` `#d98a8a`, which the legacy metric keeps as `LightRed`, lands on `DarkGray` *(<!-- amended by §27 (Adjudication O3) --> re-derived, not estimated: `#d98a8a` is L\*a\*b\* (65.6, 30.2, 12.7); CIE76 ΔE **35.0** to `DarkGray` against **62.6** to `Red`, 75.1 to `LightRed`, 41.4 to `Gray` and 47.5 to `White`, so `DarkGray` is the minimum and the conclusion holds — the earlier ~~≈ 30~~/~~≈ 61~~ were hand arithmetic)*, so a destructive label at rest stops being red at all.
3. Both `#48e054` and `#e44545` genuinely minimise ΔE against the dark primaries *(<!-- amended by §27 (Adjudication O3) --> re-derived: L\* **79.2** for `#48e054` against **72.0** for `Green` and **87.7** for `LightGreen`; full ΔE **17.8** to `Green` against **34.9** to `LightGreen` — the dark primary wins by nearly 2×, wider than the earlier ~~≈ 78 vs 72/88~~ estimate suggested)*, so no tie-break or bias recovers `DESIGN.md`'s answer while keeping ΔE. The metric must change, not be tuned.
4. The legacy metric is exact integer arithmetic, `const`-friendly, and already has a blessed baseline.

```rust
/// Nearest of the 16 xterm defaults by hue family and brightness class
/// (`DESIGN.md:320`): a near-grey collapses to the grey ladder; otherwise the
/// dominant channel picks the hue and `max > 180` picks the light half.
fn nearest_16(rgb: (u8, u8, u8)) -> Color {
    let (r, g, b) = rgb;
    let lum = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000;
    let max = u32::from(r.max(g).max(b));
    let min = u32::from(r.min(g).min(b));
    if max.saturating_sub(min) < 40 {
        return match lum { 0..=30 => Color::Black, 31..=110 => Color::DarkGray,
                           111..=200 => Color::Gray, _ => Color::White };
    }
    let bright = max > 180;
    match (r >= g && r >= b, g >= r && g >= b) {
        (true, _) if g > 120 && b < 80 => Color::Yellow,
        (true, _) => if bright { Color::LightRed }   else { Color::Red },
        (_, true) => if bright { Color::LightGreen } else { Color::Green },
        _         => if bright { Color::LightBlue }  else { Color::Blue },
    }
}
```

`lab_of` stays (`ThemeBuilder`'s L\* derivation uses it); `nearest_256` and `mono` are unchanged. The documentation sentence written into §11.4 and §21 item 29:

> `nearest_16`: **not** a ΔE minimisation. A colour whose channel spread is under 40 collapses to the grey ladder by ITU‑R BT.601 luma (`≤30 Black`, `≤110 DarkGray`, `≤200 Gray`, else `White`); otherwise the dominant channel selects the hue family (with `r ≥ g,b ∧ g > 120 ∧ b < 80` reading as Yellow) and `max(r,g,b) > 180` selects the light half. **Recorded rejection: nearest-by-CIE76 ΔE.** It is the more "correct" perceptual answer and the wrong design answer — it maps Junie's accent `#48e054` and error `#e44545` into the dark half, discarding the brightness contrast the accent system rests on, and collapses `danger_soft` onto a grey. `DESIGN.md:320` fixes the outcome and the authority order puts it above this document.

`theme::downgrade_is_deterministic_per_level` asserts `LightGreen`/`LightRed`/`Yellow`; `theme::ansi16_preserves_hue_family_and_brightness` pins `DESIGN.md:320` plus `danger_soft → LightRed`, `warning → Yellow`, `info → LightBlue`, `surfaces[1] → Black`, **`border_subtle → DarkGray`** and `fg[1] → Gray`. <!-- amended by §27 (Adjudication O3) --> ~~`border_subtle` → `Black`~~ was one of this section's *(estimate)* claims and is **wrong**: `#262626` has channel spread 0 and BT.601 luma 38, inside the `31..=110` `DarkGray` band, and the implementation and its own unit pins have always said so. It was never a carried fact — the legacy pin `theme::tests::accent_survives_downgrade` asserts only `accent → LightGreen`, `error → LightRed` and `canvas → Black`. Changing the implementation to reach `Black` — by widening the `0..=30` band or special-casing chrome — is **rejected outright**: it would retune `nearest_16`'s categorical bands to satisfy a sentence that was never a contract, inverting the authority order reason 1 rests on. **No baseline is re-blessed for this change** — it restores the recorded output.

### 25.4 Adjudication 4 — `fit_10k_grapheme_line_to_80`: "≤ 8" **REJECTED**, the benchmark is split

The obligation (§20.9‑6, R5) is *the painter allocates nothing*. The observed allocations belong to ratatui `Cell`'s symbol storage — a ZWJ family emoji exceeds the inline `CompactString` capacity, so the **buffer** heap-allocates — which is a property of the buffer, not of the painter. Relaxing the painter's assertion to a magic constant hides the invariant. The benchmark also measured `Ui::paint_str`, so it never exercised the ellipsis path it is named for.

* **`fit_10k_grapheme_line_to_80`** — corpus restricted to graphemes that fit `Cell`'s inline symbol storage (ASCII + CJK + combining marks; no ZWJ sequences), painting through **`RowUi::label`**, asserting **exactly 0** allocations. This stays §16.6's row.
* **`fit_10k_grapheme_line_to_80_wide`** *(added)* — the ZWJ corpus, **reported**, with the binding assertion *allocations are bounded by the columns painted and independent of line length*: a 10 000- and a 100 000-grapheme line into the same 80 columns record **equal** counts, `≤ 80`.

Code: `unicode_line_inline(n)` beside `unicode_line(n)` in `crates/tui-testing/src/perf.rs`; both benchmarks drive `RowUi::label`.

### 25.5 Adjudication 5 — `smallvec` in the closure: **CONFIRMED unavoidable**; §22.7 assertion (2) was false

`smallvec` arrives via `ratatui-crossterm → crossterm → parking_lot → smallvec`, and `ratatui-crossterm` is mandatory (§25.1). `xtask` already prunes the crossterm subtree and checks the forbidden set against the remainder — which works but silently deletes a subtree without asserting anything about it, and the document's clause ("absent from the normal closure") was simply false. The prune stays; a positive assertion is added; the clause becomes four parts (applied in §22.7 and §16.5):

> **(2a)** `ratatui`, `ratatui-widgets` and `ratatui-macros` are absent from `junie-tui`'s **entire** normal closure.
> **(2b)** `critical-section` and `palette` are absent from the **entire** normal closure (they can only arrive through `ratatui-core` features we disable).
> **(2c)** `smallvec`, `parking_lot`, `parking_lot_core`, `lock_api`, `scopeguard`, `libc`, `mio` and `signal-hook*` may appear **only beneath `ratatui-crossterm`**: every path from `junie-tui` to each of them passes through `ratatui-crossterm`. They are crossterm's internals, not a choice of ours; §22.4's decision is about *our* containers and is enforced by forbidden-pattern rule 26 over our source.
> **(2d)** `junie-tui`'s **direct** normal dependencies contain no `smallvec` and no direct `crossterm`.

Mechanism: keep the pruned check for (2a)/(2d), move `critical-section`/`palette` to the **unpruned** closure for (2b), and add (2c) as `cargo tree -p tui-next -e normal --invert <crate>` per name, asserting `ratatui-crossterm` on every printed path. Print the pruned subtree once on success so the exception is visible in CI output.

### 25.6 Adjudication 6 — `intents_drain_is_o_1_when_the_queue_is_empty`: a deterministic probe count replaces the wall-clock band

The measurement is ~600 ns of `Runtime::handle`, of which the 500 probes at ~1.2 ns each are ≈0.1 %. A ±10 % band on the total cannot detect a regression in the thing it names, and is inside the noise of a shared runner. The benchmark stays; the **binding** assertion moves onto something deterministic, using the mechanism `Cx::intents` already has (`if self.used == 0 { return … }`).

```rust
// intent.rs, under #[cfg(feature = "testing")]
impl IntentQueue { pub(crate) fn probes(&self) -> usize; }   // Cell<usize>, bumped in bucket_index
// runtime.rs, under #[cfg(feature = "testing")]
impl<A: App> Runtime<A> { pub fn intent_probes(&self) -> usize; }
```

Asserted: a 500-component frame with an **empty** queue performs **exactly 0** probes — structurally, because `IntentQueue::iter` returns on `used == 0` before `bucket_index`, the only site that bumps the counter — **0** allocations, and `probes(500 components) == probes(20 components)`. <!-- amended by §27 (Adjudication O4b) --> ~~with 2 intents it performs **exactly 500** probes (one per `cx.intents` call)~~ is **struck**: `probes()` is cumulative since construction and also counts the enqueue path (`bucket_slot → bucket_index`) and `was_drained`, so no absolute count is stable, and resetting the counter per frame — which would make absolute counts work — is rejected because it turns `probes()` into a per-frame statistic and breaks the since-construction contract this section wrote. The invariant is the **difference**, which cancels every constant: with the queue non-empty, a 500-component frame performs exactly **480 more** probes than a 20-component frame in the same single update pass, allocations 0. The 480 encodes "one update pass"; §3.3 step 7's focus re-run loop is bounded at four passes and a legitimate second pass makes it 960, so the equality is kept deliberately and must not be relaxed to a multiple. The **raw** wall-clock ratio is reported and never asserted: it measures 14.9× because the stub application's `update` is `for i in 0..n { cx.intents(..).count() }`, O(n) by construction — an application that does *not* call `cx.intents` per component is not measuring the drain path at all, so the O(n) loop is the workload and normalising it is the correct response. What is asserted under `PERF_STRICT=1`, with a 1.25× band, is the **normalised** per-control ratio `intents_drain_ns_per_control = (ns₅₀₀ × 20)/(ns₂₀ × 500)`, measured **0.60** against baseline `ns₅₀₀ = 632 ns`; with `s = C + n·k` it reads `(20C + 10000k)/(500C + 10000k)`, so a per-drain cost that became O(n) — total O(n²) — reads ≈ 25 and fails. That is exactly the "costs the same *per control*" property §16.6 meant, and its residual risk is named: `ns₂₀ ≈ 42 ns` is a small median, mitigated by the 2× margin and by the deterministic probe assertions being the binding ones (§16.6). The undeclared third knob `PERF_TARGET` is **folded into `PERF_STRICT`** (MI‑14); there are exactly two knobs, `PERF_STRICT` and `PERF_BLESS`.

### 25.7 Adjudication 7 — `Track::Auto` semantics: **ACCEPTED**, declared, and §17 example 9 rewritten

The implemented rule is deterministic, allocation-light, degrades sensibly, and keeps `Auto` expressible without a measurement pass — which is exactly why §10 kept `Auto`. The `_measured` variants are the correct home for `Measure`-derived sizes. §10's `layout` block becomes

```rust
pub fn rows(area: Rect, heights: &[Track]) -> Vec<Rect>;
pub fn rows_measured(area: Rect, heights: &[Track], natural: &[u16]) -> Vec<Rect>;
pub fn columns(area: Rect, widths: &[Track], spacing: u16) -> Vec<Rect>;
pub fn columns_measured(area: Rect, widths: &[Track], spacing: u16, natural: &[u16]) -> Vec<Rect>;
pub fn distribute_into(total: u16, tracks: &[Track], spacing: u16, out: &mut [u16]);  // 0-alloc, RowUi::columns
```

and `Track` carries the sentence:

> `Track::Auto` is content-sized. Without a measurement the primitive gives it **one cell** when explicit `Flex` tracks exist (so `Auto` never starves a `Flex`) and an **equal share of the remainder** when there are none. Supply the natural size through `rows_measured` / `columns_measured` to get the content size; a component that has a `Measure` impl should always do so.

**§17 example 9 had to change**, because `layout::rows(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)])` gives a two-row `Props` one row and clips it:

```rust
let props = Props::new(&[("Table", self.target.as_str()), ("Rows", "12,481")]);
let natural = [props.measure(ui, Constraints::loose(body.width, body.height)).preferred.1];
let rows = layout::rows_measured(body, &[Track::Auto, Track::Fixed(1), Track::Flex(1)], &natural);
```

Every other `Track::Auto` in §17 was re-checked the same way (there are none). `xtask doc-check` cannot catch this class, so it sits on the Slice‑4 wave‑1 review checklist. Tests: `layout::auto_takes_one_cell_beside_flex_and_an_equal_share_without_it`, `layout::rows_measured_uses_the_natural_size`.

### 25.8 Adjudication 8 — style-resolution cost: a per-frame budget, not a per-query ratio

§20.9‑1 wrote "ns ≤ 2× the pre-refactor `Theme::row`+`gutter` baseline". The pre-refactor operation is a field read on a 30-field `Copy` struct; the post-refactor operation is a six-level precedence resolution with a memo lookup. **A 2× bound between them was written without a measurement and cannot be met by any correct implementation of §11.3.** Goal §25.6 is about frames and events, not a micro-operation: ≈13 ns × ~2 000 style queries per realistic frame ≈ **26 µs**, under 0.2 % of a 16 ms budget and a small fraction of one `Terminal::draw` diff. The ≈12× per-query figure is the honest price of making §11.3's precedence chain real, and it is not frame-visible.

The bound moves, and a deterministic **cache-health** assertion replaces it as the thing that actually bites:

| Test | Binding assertion |
|---|---|
| `style_resolve_10k_parts` | **exactly 0 allocations** (R2, hard, deterministic); **cache hit rate ≥ 90 %** over the 10 k-part frame (`StyleCache::stats`, promoted from `#[cfg(test)]` to `#[cfg(feature = "testing")]`, exposed as `Runtime::style_cache_stats()`) — the §11.1 A3 memo is the mechanism and a broken key shows up here and nowhere else; ns **recorded** in `perf_baseline.txt`, asserted only under `PERF_STRICT=1` against that baseline × 1.2. <!-- amended by §27 (Adjudication O1, O4a) --> The ≥ 90 % figure is a **key-correctness floor**, not a performance bound and not a guarantee: a broken key drops it to ≈ 0.3 %, the struck one-way memo shape to ≈ 87 %, and the shipped two-way memo measures ≈ 99.7 %. At 256 entries no associativity makes a hit rate a *guarantee* — with 32 keys over 128 two-way sets, a hash configuration that puts three keys in one set yields ≈ 90.3 % and two such sets ≈ 81 %, i.e. a ≈ 5 % chance of a sub-90 % rate under an unrelated renumbering of `Part`/`Family`/`Variant` or a change to `fnv1a`. The **deterministic** guarantee is `theme::cache_hits_after_the_first_query_and_clears_by_generation` (`stats() == (1, 1)` after two identical queries, `(1, 2)` after a `clear()`), which is hash-independent and is the assertion that actually proves the mechanism; should the perf floor ever trip, diagnose the cache geometry from the `PERF-CACHE` line before suspecting the key. Added beside it, and now the binding style-cost bound: an absolute **≤ 16.0 ns per query** (`ns / 10 000 × 2 000 ≤ 32 000`) under `PERF_STRICT=1`, currently 12.0 ns — this section's own ≈ 26 µs arithmetic turned into code |
| `style_resolve_per_frame` *(added)* | <!-- amended by §27 (Adjudication O4a) --> ~~the style-resolution share of `frame_showcase_lists_120x40` is **≤ 5 %** of that frame's total ns, under `PERF_STRICT=1`~~ is **deferred to Slice 5**, which owns `apps/showcase/tests/perf.rs`; the frame does not exist. Until then: a 40-row × 5-part A/B differential (200 `ui.style` calls against 40 hoisted, so the delta covers **160** queries and the multiplier is **× 12.5**), asserting `resolution_ns × 2 000 / 160 ≤ 32 000` as a looser second net, reporting the in-situ share, and carrying **no baseline line** deliberately |

### 25.9 The thirteen deviations and the §24.4 names

| # | Deviation | Verdict | Recorded where |
|---|---|---|---|
| D‑1 | `crossterm` feature gates only the session; `ratatui-crossterm` non-optional | **Accept** | §25.1; §22.1, §16.5, §18.1, Appendix B.2 |
| D‑2 | `Id` derives structural `PartialEq/Eq/Hash/Ord` | **Accept** | §25.2; §7.1, §16.1 |
| D‑3 | Ansi16 CIE76 | **REJECT** | §25.3; §11.4, §21 item 29 |
| D‑4 | `Recipes::apply_mono_fallbacks(&mut self)`; mono `DISABLED` adds `DIM` | **Accept both** | §11.4: the call is `out.recipes.apply_mono_fallbacks()` — the sketch's `(&mut out)` was a borrow error. `DISABLED` becomes "no gutter glyph, no marker, `fg = Role::Fg(Faint)`, all modifiers removed **and `DIM` added**", because colour is excluded from case 9's comparison and a colour-only disabled rule is indistinguishable from default. MI‑13 recorded with it: the fallbacks must be appended to every **variant** map too, or a variant that re-declares `PRESSED` beats the mono bracket rule |
| D‑5 | `Resolved.align` | **Accept** | §11.3: `pub align: Option<Align>` added to `Resolved` (`StylePatch.align` already existed; `Resolved` was the omission) |
| D‑6 | `Ui::register_editor` / `Ui::declare_state` | **Accept** | §17.0 A2, both declared, plus the invariant: *declared flags are read back through `FrameRead::state` on the **next** frame (they live in last frame's `declared` list), the same one-frame contract as `cx.area` (S3)*. Consequence recorded deliberately: `focused_is_editing` reads last frame's flags, so a paste in the same `handle` that began an edit is not routed. §5 R2 names `declare_state` alongside `report_layout` |
| D‑7 | `App::on_esc` + `Cx::command()` | **Accept both** | §17.0 A1 gains `fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> { Response::ignored() }` as §3.3 step 8(c)'s application hook (the spec put the ladder on `Screen` and left `App` without one); §17.0 A2 gains `fn command(&self) -> Option<ActionKey>` as the channel by which a matched `KeyMap` chord reaches `App::update` (§3.3 step 2 said "produces an app action" and declared no channel) |
| D‑8 | `conformance_suite!(name => Case)` | **Accept** | §16.2: a macro cannot derive a module ident from the `NAME` const, so the ident is written explicitly. **Guard added**: the macro emits, per entry, `#[test] fn name_matches_the_module() { assert_eq!(<$case as Conformance>::NAME, stringify!($name)); }`, so the two cannot drift. The invocation is rewritten as `button => ButtonCase, chip => ChipCase, …` |
| D‑9 | `validate` / `secret` / `field_control` at the crate root | **Accept** | Appendix B.2: they are foundation vocabulary consumed **by** `components/input.rs`, not components; `components/` is a Slice‑4 directory the Slice‑3 owner may not write |
| D‑10 | Rule‑22 regex narrowed | **AMEND, not accept as-is** | §22.7: the **broad** regex `Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}` is restored, and `crates/tui/src/theme/downgrade.rs` and `crates/tui/src/theme/builder.rs` are added to the rule's **path** allow-list (which does not feed the "`legacy_api.txt` must be empty" condition). A narrowed regex hides the exception; a named path shows it. As narrowed, `Color::Rgb(r, g, b)` from computed values escaped anywhere in the crate |
| D‑11 | `SecretPolicy::default().mask = GlyphRole::Dirty` | **AMEND** | `GlyphRole::Dirty` is the *uncommitted-changes* marker and §11.4's mono rule already binds `MARKER + WARNING/DIRTY → Dirty`; overloading it means a theme that changes the dirty marker changes password masking. §11.2 gains **`GlyphRole::SecretMask`** (Junie `•` — the same glyph, a distinct role) and it is the default `SecretPolicy::mask` |
| D‑12 | `ThemeBuilder` parameter names | **Accept** | §17.0 A5 names them positionally; the implementation's spellings are compatible. `focus`, `selection`, `highlight`, `field`, `disabled` are all present and `derive_unset` fills the rest. `theme::builder_derives_every_unset_token_deterministically` and `theme::derived_tokens_meet_design_contrast_ratios` were **not located** in this pass; `every_named_test_exists` must settle their existence mechanically |
| D‑13 | *(not on the list; found by the review)* `Ui::paint_spans(area, spans, base)`; `Ui::style`/`style_patched` take `&mut self` | **Accept both** | §17.0 A2. `base` is the part style the spans inherit — without it `RowUi::label_spans` could not honour the `LABEL` recipe. `&mut self` is required by the §11.1 A3 memo and by the per-cell role recording `dim_layer` depends on. Consequence recorded in §10: `Measure::measure(&self, ui: &Ui<'_>, …)` cannot call `Ui::style`; §26 N2 settles how it resolves instead |

**§24.4 self-declared names — all accepted.** `SelectAction { Chose(ItemKey), Opened, Closed }`, `RadioGroupAction { Chose(ItemKey) }`, `ChipBarAction { Toggled(ItemKey), Closed(ItemKey), Activated(ItemKey) }` and the three state structs follow §6.1's `XAction` convention and §4's state-holds-no-props rule. None is implemented yet (they are 4A/4B/4F types), so nothing beyond the naming is verifiable, and the naming is correct.

**One correction to Appendix B.2.** `author::raw` is an **inline** `pub mod raw` inside `crates/tui/src/author.rs`, not a separate `author/raw.rs`. §24.1's exact Rust already shows it inline; Appendix B.2's line was the outlier and is corrected, together with the whole-file grep in §24.5 that had forced the wrong conclusion.

### 25.10 The blocker and major inventory — binding correction obligations F1–F26

Serial; each step independently testable. F1–F7 are the blockers. These are **obligations**, not suggestions: the Slice 3 gate does not pass until each is done and its named test exists.

| # | Obligation | Named test(s) |
|---|---|---|
| **F1** | **BL‑1 — style precedence is wrong and its guard test is vacuous.** `PartRecipe::apply` merges base *and* state rules together, so the order is `family.base → family.states → variant.base → variant.states`; §11.3 fixes it as `family base → variant delta → all state rules by specificity`. Any variant that sets a base colour silently defeats the family's `HOVERED`/`FOCUSED`/`PRESSED`/`ERROR` rules; the built-ins dodge it only because `button_variant` re-declares the full state set per variant. The guard test asserts "3 over 2" against `Theme::junie()`, whose `focus` and `accent` are the **same colour**, so it passes under either ordering. Split into `apply_base` / `apply_states`; in `accumulate`, apply family base, then variant base, then a stable two-way merge of both state lists in ascending specificity (family first on a tie) — both lists are stored pre-sorted, so it stays allocation-free and O(n+m) | `theme::precedence_family_then_variant_then_state_then_global_then_scope_then_instance` (its "3 over 2" arm rewritten with a role whose bound colour differs from the variant's under **both** built-in themes, e.g. state `Role::Warning` vs variant base `Role::Accent`), `theme::state_rules_beat_a_variant_base`, `theme::family_and_variant_state_rules_interleave_by_specificity` |
| **F2** | **BL‑2 — two "unreachable" arms are infinite spin loops, not panics.** `fn unreachable_entry<T>() -> T { loop { core::hint::spin_loop(); } }` exists only to satisfy `clippy::panic`/`expect_used` at deny. A library that hangs the process with raw mode on and the alternate screen entered is strictly worse than a panic, whose hook restores the terminal. Both arms are genuinely dead. Delete them; restructure `PartMap::entry`, `Recipes::get_mut`, `Recipe::variant_mut` and `Ui::cache` to return from the insert branch, or use one documented `#[expect(clippy::expect_used, reason = "Vec::insert(i, _) makes get_mut(i) infallible")]` | `architecture::no_unreachable_spin_loops` (§22.7 rule 27a) |
| **F3** | **BL‑3 — `Ui::raw()` marks the whole clip rect written and clobbers the recorded roles**, and `CellUi::drop` calls it on every right-aligned cell. Two consequences: inside a layer `LayerDraw::written` becomes all-true for the component's clip, so `composite_onto` copies **unpainted** cells over the page (the bitset §3.3 step 12 and R3 rest on is defeated by any grid or list with a right-aligned cell inside a dialog); and on the page `mark_area` writes the component's roles into every cell of the clip, so `dim_layer`'s role walk dims with the wrong role. Add `Ui::buffer_in(area) -> (&mut Buffer, Rect)`, which marks `area` (intersected with the clip), not the clip, and use it from `CellUi::drop` and `RowUi::raw`; `Ui::raw()` keeps whole-clip marking as the documented public escape hatch | `layer::composite_copies_only_painted_cells`, `ui::dim_layer_uses_the_role_of_the_painted_cell` |
| **F4** | **BL‑4 — `Ui::paint_spans` allocates a `Vec` per call, on the row path.** `RowUi::label_spans` routes through it, so every span-rendered row costs one allocation per row per frame — contradicting §20.9‑6 (R5) and putting `frame_showcase_lists_120x40 < 20`, `grid_500x12_render < 100` and `viewport_100k_lines_render` out of reach for `TextViewport` and `DiffView`, which a Slice‑4 owner cannot fix because `ui/` is Slice 3's. Paint span-by-span through `Buffer::set_span`, accumulating the x cursor and the per-span role marks. §22 R‑3 names `set_span` as a sanctioned writer; §17.0 A2 takes the three-argument signature | `ui::paint_spans_matches_row_ui_label_spans` (differential **plus** the allocation assertion: 500 rows × 3 spans records 0 allocations) |
| **F5** | **BL‑5 — Ansi16 downgrade contradicts `DESIGN.md` and the existing rendered output** (§25.3). Restore the legacy categorical `nearest_16` | `theme::downgrade_is_deterministic_per_level` (asserting `LightGreen`/`LightRed`/`Yellow`), `theme::ansi16_preserves_hue_family_and_brightness`, and the legacy pin `theme::tests::accent_survives_downgrade` stays green |
| **F6** | **BL‑6 — `Ui::set_cursor` keeps the *first* writer on a layer, not the *focused* one.** With two same-layer writers (two `TextInput`s in a `Form`, both `EDITING`), the first drawn wins, the focused one is rejected with a `CursorRejected` diagnostic, and §3.3 step 15 then drops the retained request because its owner is not focused: **no cursor at all, plus a spurious diagnostic**, failing `*::no_diagnostics_are_emitted_during_the_journey`. §8.4 makes filtering the runtime's job, so components are entitled to write unconditionally. Keep the best candidate by `(layer, owner-is-focused)`, storing `focused` on `CursorRequest`; record `CursorRejected` for the loser only when it is non-inert | `cursor::the_focused_owners_write_wins_on_the_same_layer` |
| **F7** | **BL‑7 — `Harness::resolved` / `Runtime::resolved` hardcode `Family::BUTTON`.** §16.4's theme-coupling contract replaces `assert_eq!(fg, Theme::junie().focus)` with `h.resolved(id, Part::GUTTER).style.fg`; for a `List`, `Tabs` or `Field` that resolves the **button** recipe and returns a colour the component never painted, so every migrated assertion in Slices 5–7 would be silently wrong. Record the resolution key: widen `FrameState::styled_parts` (under `testing`) to `Vec<(Id, Family, Variant, Part, Resolved)>`, written by `RowUi::style_of` and by `Ui::note_styled` at each component's own query; `Runtime::resolved(id, part)` returns the recorded `Resolved`, falling back to `resolved_in` only when nothing was recorded. `resolved_in(f, v, id, p)` stays as the explicit escape hatch | `harness::resolved_reports_the_family_the_component_actually_queried` |
| **F8** | **MA‑1 — `Registry::hit` orders by registration index, not by layer.** A page control drawn *after* `ui.layer(POPOVER, …)` shadows the popover; the runtime then sees `hit.layer < top_layer` and treats a click **on** the popover as an outside click, dismissing it. Masked for modals only because `inert_below` suppresses page registration. §9.1's "z-order is the layer order, NOT the call order" must hold for hit-testing too. `hit()` selects `max_by_key(|r| (r.layer, index))` | `hit::higher_layer_shadows_lower` (strengthened: the lower layer registers **last**), `hit::a_lower_layer_region_registered_later_does_not_shadow_a_higher_one` |
| **F9** | **MA‑2 — `xtask`'s source scan stops at the first `#[cfg(test)]` in a file.** A mid-file `#[cfg(test)] pub(crate) const fn stats` leaves everything after it unscanned by all 26 rules — in practice all of `theme/resolve.rs` past line 239 and `runtime.rs` past line 1028. Skip only the `#[cfg(test)]`-attributed **item** (brace-matched through `syn`, already an `xtask` dependency), never the file tail; then re-run the whole rule set and fix whatever it newly reports | `architecture::no_deprecated_or_legacy_api_usage` (with the corrected scan), `cargo run -p xtask -- boundary` reporting `ok` with empty allow-lists |
| **F10** | **MA‑3 — `text::row_ui_matches_fit_for_every_fixture` skips exactly the cases it exists to protect.** The reference `fit` is re-implemented in the test over **chars** while the legacy one walks graphemes; `cellable` drops control/ZWSP/BOM inputs; the loop `continue`s on every non-ASCII multi-byte input that needs truncation (CJK, emoji, combining marks at the cut); and both sides are `trim_end_matches(' ')`, discarding padding. §20.10's "any change to padding or ellipsis placement … is a regression" rests entirely on this test. Use the legacy grapheme-walking `fit` **verbatim** as the reference, remove the `continue`, compare cell symbols **including** trailing padding | `text::row_ui_matches_fit_for_every_fixture` |
| **F11** | **MA‑12 — `trybuild` is absent, so three named compile-fail tests do not exist**, and `crates/tui/tests/ui/` does not exist. These are precisely the tests that pin the type-level guarantees §6.1 and §15 claim. Add the dev-dependency, the directory, and the cases | `response::must_use_is_enforced`, `response::bitor_is_defined_only_for_unit`, `secret::is_not_clone_not_eq` |
| **F12** | **MA‑10 — `every_named_test_exists` does not exist.** The single most valuable gate for this review is absent from `xtask`'s `CHECKS` and from `crates/tui/tests/architecture.rs`; without it the missing and renamed names below are invisible to CI, which is why the review's inventory had to be assembled by hand. Also missing from the check set: `conformance_covers_every_public_component`, `state_override_is_used_only_in_apps_and_fixtures`, `all_examples_compile` / `examples_are_external_consumers`. Add all four and fix whatever they report | `architecture::every_named_test_exists`, `architecture::conformance_covers_every_public_component`, `architecture::state_override_is_used_only_in_apps_and_fixtures`, `architecture::all_examples_compile` |
| **F13** | The four §16.2 **suite-level** tests are missing | `conformance::registry::declared_parts_are_the_parts_actually_styled`, `conformance::conflicting_visible_bindings_are_reported`, `conformance::focus_transition_settles`, `conformance::draw_registers_nothing_when_it_cannot_draw` |
| **F14** | **MA‑6 — `Family::custom(..)` resolves to nothing, so Scenario G renders invisible.** `Recipes::get(f)` misses, `accumulate` returns an empty patch and `bind` yields `Style::new()`, so example 12 paints an unstyled control and `mono_states_are_distinguishable` could never pass for it. §11.2 did not say what a custom family inherits. Add a neutral fallback recipe (`row_like`'s `CONTAINER/GUTTER/MARKER/LABEL/META` set) used when the lookup misses; `define_family` replaces it. This is the worst first-run experience in the surface | `theme::a_custom_family_resolves_through_the_neutral_recipe` |
| **F15** | **MA‑5 — `Capture.origin` is documented as the pointer position but set to the area's top-left.** `Cx::capture_origin()` exists so a splitter or scrollbar can compute `pos - origin`; with the top-left the delta is wrong by the press offset within the thumb. The unit test hand-builds a `Capture` and never exercises `Cx::capture`. The runtime records the live press position in `Interaction::press`; expose it to `Cx` and use it as `origin` | `capture::origin_is_the_press_position` (exercising `Cx::capture`) |
| **F16** | **MA‑7 — focus intents enqueued for a `pending_focus` are dropped on resize** (the resize path returns early without running `app.update`, and the next `intents.clear()` discards the `FocusOut`/`FocusIn` pair). Handle the resize, then fall through to `run_update(None)` — still with no input intents — or re-stage `pending_focus`. **MI‑7** with it: `run_update`'s give-up path calls `apply_staged_focus()` and then immediately `intents.clear()`, so §21 item 11's "applies the pending `FocusOut` **and** the matching `FocusIn`" does not happen; deliver the pair on the next `handle` via `pending_focus` | `runtime::a_fifth_focus_pass_is_diagnosed_and_applied` (strengthened to assert the pair arrives, not merely `focus().is_some()`) |
| **F17** | **MA‑8 / MA‑9 — conformance cases 9 and 12 are weaker than §16.2 specifies.** Case 9's `DEFAULT_MONO_STATES` is `{default, focused, selected, pressed, disabled}` where §16.2 requires the full ten, so a component silently gets a five-state check; make the default the full ten, let `mono_states()` only **narrow** it, and have the driver assert that every state the component's `Caps` imply is present. Case 12 checks click identity across a reorder but never sets cursor/checked on `k₁,k₂` nor asserts they survive `reconcile`, which `CollectionCore` already supports | `conformance::<component>::mono_states_are_distinguishable`, `conformance::<component>::item_identity_survives_reorder` |
| **F18** | Adjudication changes 4, 6 and 8: split `fit_…` into inline/wide; probe-count assertions for `intents_drain_…`; cache-hit-rate and per-frame budget for `style_resolve_…`. **Re-bless `perf_baseline.txt` in the same commit**, with a note | `fit_10k_grapheme_line_to_80`, `fit_10k_grapheme_line_to_80_wide`, `intents_drain_is_o_1_when_the_queue_is_empty`, `style_resolve_10k_parts`, `style_resolve_per_frame` |
| **F19** | Adjudication 5: the dependency check gains the inverted-tree (2c) assertion; `critical-section`/`palette` move to the unpruned closure | `architecture::dependency_graph_is_exactly_the_declared_set` |
| **F20** | Adjudication 1: `xtask` rule 27 and the two-file boundary check | `architecture::ratatui_crossterm_is_named_in_exactly_two_files`, `no_deprecated_or_legacy_api_usage` rule 27 |
| **F21** | **MI‑11 — facade tidy.** `pub mod text` leaks `TextBuffer`, `grapheme_width`, `is_word_char` and `thousands`, none of which is in Appendix B.3/B.4's curated lists; make it `pub(crate) mod` with curated re-exports. `author` re-exports the whole `crate::id` module rather than `{Id, ItemKey, Part, PartRef}` — a small unintended widening (`id!` is `#[macro_export]` and already reachable at the root) | `architecture::every_foreign_type_in_the_public_surface_is_re_exported`, `architecture::applications_depend_only_on_the_library_facade` |
| **F22** | D‑11 (`GlyphRole::SecretMask`); **MA‑13** — `zeroize` can be elided: `secret.rs` fills a moved-out `Vec` that is immediately dropped, and LLVM may remove the dead stores; add `core::hint::black_box(&bytes)` and a `compiler_fence(Ordering::SeqCst)` after the fill (keeping `#![forbid(unsafe_code)]`), rename `zeroize_clears` to the §16.1 name, assert the observable property available in safe Rust — the capacity is released and a fresh `expose()` is empty — and comment the compiler-elision risk as a known limit of safe-Rust zeroization; D‑10 (rule‑22 broad regex with named path exceptions) | `text::zeroize_overwrites_before_drop`, `architecture::palette_literals_are_confined_to_theme_builtins` |
| **F23** | **MA‑14 — `doc-check` misses §24 and is a heuristic, not rustdoc-json.** Its range stopped at §23, so §24's `SelectAction`, `RadioGroupAction`, `ChipBarAction`, `border::ASCII`, `FormData::options` and the example‑13 rewrite were unchecked; and `foreign_members()` allow-lists legacy names (`Theme::row`, `Theme::gutter`, `Interaction::pressed`, `Interaction::focus_hidden`) as if they were foreign API. Extend the range to §24–§26, move the legacy names into an explicit `doc_check_allow.txt`, and record the rustdoc-json resolver §21 item 34 specifies as a **Slice‑8 upgrade** | `xtask doc-check` |
| **F24** | **MI‑9 — layer sizing** must be decided before 4F starts, and the decision recorded in §9.1 | **Discharged by §26 N1** (`LayerSize`, `Cx::resize_layer`, `Dialog::layer`) |
| **F25** | The remaining MINORs: **MI‑1** `FocusRing::innermost_scope` uses `.rev().max_by(layer)`, returning the **earliest** scope on the highest layer while the doc says "latest" (harmless today with one scope per layer; fix the code or the doc — §16.1 records "latest"). **MI‑2** `FocusRing::reconcile` appends `.or_else(|| self.reachable().next())` beyond §3.3 step 14's `(d) None` — better behaviour, recorded as step 14 (d)/(e) rather than left undeclared. **MI‑3** `focus::click_only_entries_are_never_reachable`, `focus::read_only_entries_stay_in_the_ring`, `focus::restore_target_receives_keys_before_the_next_draw` and `hit::inert_below_registers_nothing` exercise none of the mechanism their names claim (it lives in `Ui::register_entry` and `Runtime`); move them to runtime-level tests. **MI‑4** `Runtime::handle` calls `keymap().conflicts()` on **every** input, an O(n²) scan per event; compute once per keymap change or under `debug_assertions`. **MI‑5** wheel routing is not layer-filtered — a wheel over the page below a **popover** scrolls the page; deliver only when `hit_scroll(...).layer == top_layer`, matching `deliverable()`. **MI‑6** `dismiss_top` enqueues `Intent::Cancel` for *every* dismissal reason while §6.1 defines `Cancel` as "Esc reached this owner after layer dismissal"; gate it on `DismissReason::Esc`. **MI‑8** `RowUi::columns` silently truncates past `MAX_COLUMNS = 16`; the cap is documented in §12.2. **MI‑10** `fuzzy` allocates three `Vec`s per call — fine at Slice 3, 300 k allocations for a 100 k-item `Picker` filter; flagged for 4F. **MI‑12** `Theme::fingerprint` formats the whole theme into a `String` on a public path; hash the tokens structurally or move it behind `testing`. **MI‑13** `apply_mono_fallbacks` appends to `recipe.parts` only, never to variant maps (recorded in §11.4). **MI‑14** `PERF_TARGET` folded into `PERF_STRICT`. **MI‑16** `Measure` cannot resolve styles | **MI‑16 discharged by §26 N2**; the rest are ordinary corrections with the tests already named in §16.1 |
| **F26** | Apply every document amendment in the review's §2 and §3 to `COMPONENT_ARCHITECTURE.md` (**this section and its inline markers**) and mirror the adjudications in `REFACTORING_STATE.md`, per the change-control rule at the top of this document | `rg -n 'Slice 3 foundations review' REFACTORING_STATE.md` |

### 25.11 Independent API review — what is confirmed, and the two ceremony costs

Recorded because they are decisions about the surface, not defects to fix.

* **`Ui`/`Cx` phase separation is honest, structurally.** `Ui` has no `&mut` path to app state, no `Cx`, no `Response` and no layer mutation — `Ui::layer` can only *draw into* a layer `Cx::open_layer` already assigned, returning `None` otherwise; `Cx` has no `Buffer`, no painting method and no `Ui`; `App::draw(&self, …)` makes the compile-error claim real at the top of the stack. Two caveats are named rather than hidden: `Ui::cache` (R8) is real mutable state reachable from `draw`, guarded only by a regex heuristic (`architecture::cache_types_are_derived_only`, a `syn` upgrade deferred); and `Ui::declare_state` is a draw-phase write the runtime consumes next frame (D‑6, now named in §5 R2).
* **`Response` composition and `Intent<'f>` ergonomics are correct.** `BitOr` is `Response<()>`-only as specified; the `Intent<'f>` split works exactly as §21 item 6 intended — `cx.request_repaint()` **inside** the drain loop compiles, which is the whole point. `BitOr` takes `self.id.or(rhs.id)` rather than the document's "id: lhs" — a benign improvement, now stated in §6.1 and §21 item 4. Example 12 should use `Binding::lookup` instead of a hand-written `BINDINGS.iter().find(…)`.
* **`RowUi` paints with no intermediate `String`** for `label`/`label_patched`/`meta`/`trailing`/`label_fmt`, `num`/`money` (stack buffers) and `columns` (`[u16; 16]`). **Not** for `label_spans` (F4) and **not** during `CellUi::drop`'s alignment shift (F3). Also noted: `label_in` does not pad to the full width in the *label* style — the row was filled with the *container* style first — a behavioural difference from the legacy `fit` that F10's weakened test could not detect.
* **The `author` surface is sufficient for example 12** — Scenario G's mechanical proof holds — and for Button and Tabs. Field/TextInput and List are "almost" (MI‑16 and F4); ScrollRegion is "almost" (F15). **Dialog-as-layer** and **any custom-family component** were **not** sufficient: F24/§26 N1 and F14 respectively. Two further gaps: `Ui::surface_style()` (§26) and a declared `Ui::scroll_region(id, part, …)` convenience — §12.2 names it and `components/scroll_region.rs` is 4E's file, but the `Ui` half is Slice 3's. **`Ui::scroll_region` is an open item, not decided here.**
* **Ceremony costs a downstream author feels**, in order: (1) a custom family gets no styling at all (F14) — the first thing an author writes; (2) `ui.style(...)` returns a `Resolved` whose `.style` must be threaded manually to every paint call, and `Ui::style` is `&mut self`, so `ui.fill(cell, ui.style(...).style)` is a borrow error — answered by `Ui::with_part` (§26); (3) two-phase props construction is honest but verbose, and §13's "props are built once" helper convention must be in the authoring guide from day one, not discovered.
* **What the boundary check misses** (beyond F9 and D‑10): rule 9 cannot distinguish field *assignment* (`st.fg = …`, which R‑9 forbids as a layering form) from construction; `CrosstermBackend` was not a forbidden pattern anywhere (rule 27); and `no_domain_vocabulary_in_the_library`'s regex includes `\bworkspace\b` and `\binstance\b`, which appear in ordinary architectural prose ("per-instance patch") — it passes today only because comments are stripped, so it will fire the first time a `///` line is reflowed. Narrow the regex or scan code lines only, **deliberately** (recorded in §16.5).

### 25.12 Slice 3 gate additions

The Appendix A Slice 3 gate stands, with these additions; all commands from the workspace root, and during Slices 3–4 `-p junie-tui` reads `-p tui-next`.

<!-- amended by §28: the render line names both targets (P2) -->

```bash
# F1: precedence is real, not a colour coincidence
cargo test -p tui-next --lib theme::resolve::tests::precedence_family_then_variant_then_state_then_global_then_scope_then_instance
cargo test -p tui-next --lib theme::resolve::tests::state_rules_beat_a_variant_base
# F2: no hang-instead-of-panic path survives
! rg -n 'spin_loop' crates/tui/src
cargo test --workspace --test architecture no_unreachable_spin_loops
# F3/F4: the written-cell bitset and the row path
cargo test -p tui-next --test render --test render_components layer::composite_copies_only_painted_cells   # §28: both targets
cargo test -p tui-next --test perf --release -- --test-threads=1 paint_spans
# F5: DESIGN.md:320 is the contract
cargo test -p tui-next --lib theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness
cargo test --all-targets theme::tests::accent_survives_downgrade      # the legacy pin, still green
# F6/F7: cursor selection and the theme-coupling migration contract
cargo test -p tui-next --lib cursor::tests::the_focused_owners_write_wins_on_the_same_layer
cargo test -p tui-next --all-features harness::resolved_reports_the_family_the_component_actually_queried
# F9: the boundary check scans whole files
cargo run -p xtask -- boundary                 # `ok` for every check, allow-lists printed and empty
test -s crates/tui/tests/allow/legacy_api.txt && exit 1
# F11/F12/F13: the named-test inventory is machine-checked
cargo test --workspace --test architecture every_named_test_exists
cargo test --workspace --test architecture conformance_covers_every_public_component
cargo test -p tui-next --test conformance conformance::registry::
cargo test -p tui-next --test conformance conformance::focus_transition_settles
cargo test -p tui-next --test conformance conformance::conflicting_visible_bindings_are_reported
cargo test -p tui-next --test conformance conformance::draw_registers_nothing_when_it_cannot_draw
# F19/F20: the dependency story is asserted, not assumed
cargo test --workspace --test architecture dependency_graph_is_exactly_the_declared_set
cargo test --workspace --test architecture ratatui_crossterm_is_named_in_exactly_two_files
cargo tree -p tui-next -e normal --invert smallvec    # every path passes through ratatui-crossterm
# F18: the perf contract, re-blessed in the same commit as the code change
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1
git diff --exit-code crates/tui/tests/perf_baseline.txt
# F26: the document and the state ledger carry the adjudications
rg -n 'Slice 3 foundations review' COMPONENT_ARCHITECTURE.md REFACTORING_STATE.md
```

**Gate pass condition.** Every command exits 0; `crates/tui/tests/allow/legacy_api.txt` and `crates/tui/tests/allow/domain.txt` are empty; `xtask boundary` prints `ok` for every check including the three new ones; `every_named_test_exists` reports no missing name from §16.1, §16.2's suite-level list and §16.4; and this document carries the amendments to §6.1, §7.1, §9.1, §10, §11.1 A3, §11.2, §11.3, §11.4, §16.1, §16.2, §16.5, §16.6, §17.0 A1/A2/A5, §17 example 9, §18.1, §20.9‑1/‑2, §21 items 4/11/29/30, §22.1, §22.2, §22.7, §24.5 and Appendix B.2/B.3/B.4, each mirrored in `REFACTORING_STATE.md`. **Then, and only then, Slice 4 wave 1 (4A, 4B, 4C, 4E, 4G) may start.** F24 (MI‑9) additionally had to be decided before 4F was scheduled and F25's MI‑16 before 4A — both are discharged by §26.

---

## 26. Adjudication N — layer sizing and `Measure` style access

**Source.** `docs/reviews/adjudication-n-layer-measure.md`, adjudicating the two Slice‑3 foundation decisions the Slice 3 foundations review left open: **MI‑9** (layer sizing, blocking 4F) and **MI‑16** (`Measure` cannot resolve styles, blocking 4A). Read against §9.1, §9.2, §10, §11.1 A3, §11.3, §16.1, §16.2 cases 14/19, §17.0 A2/A6/A7, §17 examples 9–10 and §21 items 14/20/30. **Accepted as written.** Every earlier section it changes carries an inline `<!-- amended by §26 -->` marker.

### 26.1 N1 — layer sizing

**Decision.** The size is a **typed field on `LayerSpec` supplied by the opener**, and the component that owns the layer re-asserts it from `update` every frame through a new `Cx::resize_layer`. The runtime keeps the single resolver; no measure callback, no two-pass draw, and **no component computes a rect**.

Three changes: a typed `LayerSize` replacing the `(0,0)` sentinel; two narrow geometry mutators on `Cx`; and `Anchor::Point` flipping instead of covering the pointer.

**1. The size becomes a type, not a sentinel** (§9.1, §17.0 A6).

```rust
/// How large a layer asks to be. The resolver clamps to the screen; it never
/// grows a layer, so a `Fixed` size is a maximum as well as a request
/// ("size, then clamp, then documented degradation", §9.1).
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayerSize {
    /// The whole screen. The content is responsible for its own internal
    /// layout; `Anchor` is ignored. Help overlays, file browsers, `TooSmall`.
    Fill,
    /// Exactly `w × h` cells before clamping.
    Fixed(u16, u16),
}

pub struct LayerSpec {
    // … kind, owner, anchor, dismiss, restore_focus, initial_focus unchanged …
    /// The requested content size (§9.1).
    pub size: LayerSize,          // was `min_size: (u16, u16)`
    // … backdrop, inert_below unchanged …
}

impl LayerSpec {
    /// Set the requested size.
    #[must_use]
    pub const fn size(mut self, s: LayerSize) -> Self { self.size = s; self }
}

/// Resolve a layer's area: anchor, flip, then clamp (§9.1).
///
/// `LayerSize::Fill` yields the whole screen. A `Fixed` size is clipped to the
/// screen first, then anchored, then flipped if the chosen side has no room,
/// then `Rect::clamp`ed. A layer is never grown to meet its request.
pub fn resolve_anchor(screen: Rect, anchor: Anchor, size: LayerSize) -> Rect {
    let (w, h) = match size {
        LayerSize::Fill => return screen,
        LayerSize::Fixed(w, h) if w == 0 || h == 0 => return Rect::ZERO,
        LayerSize::Fixed(w, h) => (w.min(screen.width), h.min(screen.height)),
    };
    // … existing Screen / Rect / Point arms, unchanged except Point (below) …
}
```

`Fixed(0, _)` now resolves to `Rect::ZERO`, not to the screen — a zero-size request is an empty layer, which is what §16.2 case 19 and `conformance::draw_registers_nothing_when_it_cannot_draw` assume. **The old `(0,0) ⇒ screen` conflation is the whole defect**: `min_size` was a *maximum* (the resolver clamps down and never grows), so the name was false, and every constructor asking for `(0,0)` meant every dialog had to re-implement centering — the opposite of §9.1's "one resolver".

**2. `Anchor::Point` flips.** The degenerate-rect form of the `Rect` arm, so a tooltip or context menu near an edge is placed above/left of the pointer instead of sliding over it:

```rust
Anchor::Point(p) => {
    let below = p.y.saturating_add(1);
    let fits_below = below.saturating_add(h) <= screen.bottom();
    let fits_above = p.y >= screen.y.saturating_add(h);
    let y = if fits_below || !fits_above { below } else { p.y.saturating_sub(h) };
    let fits_right = p.x.saturating_add(w) <= screen.right();
    let fits_left  = p.x >= screen.x.saturating_add(w);
    let x = if fits_right || !fits_left { p.x } else { p.x.saturating_sub(w) };
    Rect { x, y, width: w, height: h }
}
```

`Anchor::Rect`'s flip already consumed the size and was simply **unreachable** while every spec asked for the whole screen.

**3. Constructor defaults.** `modal`, `popover` and `tooltip` keep `size: LayerSize::Fill`. `Fill` is the honest primitive default (a full-screen modal is a real case) and `modal()` is `const`, so it cannot read `design.size.dialog_width`; the **component** supplies the size, and `Dialog`/`Select`/`Picker`/`ContextMenu` never open a bare spec.

**4. Geometry is the one part of a spec that may change while open** (§17.0 A2, §9.1).

```rust
impl Cx<'_> {
    /// Update an open layer's requested size. No-op when `id` is not open or
    /// the size is unchanged; the next `draw` re-resolves the anchor. Safe to
    /// call unconditionally every frame — that is the intended use.
    pub fn resize_layer(&mut self, id: Id, size: LayerSize);

    /// Update an open layer's anchor (a popover whose owner moved).
    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor);
}
// backed by, in crates/tui/src/layer.rs:
impl LayerStack {
    pub(crate) fn spec_mut(&mut self, id: Id) -> Option<&mut LayerSpec> {
        self.open.iter_mut().find(|l| l.id == id).map(|l| &mut l.spec)
    }
}
```

Both set `services.repaint = true` **only when the value actually changed**. Nothing else on `LayerSpec` is mutable after open: `kind`, `inert_below`, `restore_focus` and `initial_focus` are lifecycle facts the runtime armed at open (§3.3 step 11), and re-deriving them mid-life would desync the focus scope and the inert floor.

**5. The runtime.** The arming loop passes `l.spec.size` to `resolve_anchor`; nothing else changes. Ordering is already correct: `handle` runs `app.update` (where `resize_layer` lands) and `draw` runs afterwards, so a size asserted in `update` takes effect in the very next draw — the same frame, **no flash**.

**6. `Ui::layer` — no signature change.** One doc sentence added: *the `Rect` is the resolved layer area and is already the clip; a layer's content lays out inside it and never re-anchors, re-flips or re-clamps.* **7. `Anchor` — no change**: the enum, `Side`, `CrossAlign` and `ScreenAlign` were all correct, merely unreachable.

**How `Dialog` sets its size.** `Dialog` owns its size as a pure function of props + `DesignTokens`, computed identically in both phases — the rule §15.1 F4 already imposes on field height.

```rust
impl<'a> Dialog<'a> {
    /// Rows the body slot needs. The dialog never sees the body closure before
    /// `draw`, so the caller states it; the convenience constructors set it.
    pub const fn body_rows(mut self, n: u16) -> Self { self.body_rows = n; self }

    /// `.width(w)` when set, else `design.size.dialog_width` (54).
    pub fn measured_width(&self, d: &DesignTokens) -> u16 {
        self.width.unwrap_or(d.size.dialog_width)
    }

    /// border(2) + title(1) + wrapped description + input_rows + [blank + body] + [blank + actions].
    /// ~~border(2) + title(1) + wrapped description + [blank + body] + [blank + actions]~~ was
    /// written before `prompt`/`acknowledge` existed (§28 P4). <!-- amended by §28 -->
    /// Pure in `(props, DesignTokens)`; `draw` lays out against this number.
    pub fn measured_height(&self, d: &DesignTokens) -> u16 {
        let inner = self.measured_width(d)
            .saturating_sub(2)
            .saturating_sub(d.space.dialog_inset.saturating_mul(2));
        let desc    = self.description.map_or(0, |s| text::wrapped_rows(s, inner));
        let input   = self.input_rows(d);                       // NEW (§28 P4)
        let body    = if self.body_rows == 0 { 0 } else { self.body_rows.saturating_add(1) };
        let actions = if self.actions.is_empty() { 0 } else { 2 };
        3u16.saturating_add(desc).saturating_add(input).saturating_add(body).saturating_add(actions)
    }

    /// The layer this dialog wants. Call from `update` at the moment of opening.
    pub fn layer(&self, cx: &Cx<'_>) -> LayerSpec {
        let d = cx.design();
        LayerSpec::modal(self.id)
            .size(LayerSize::Fixed(self.measured_width(d), self.measured_height(d)))
    }

    pub fn confirm(id: Id, title: &'a str, question: &'a str) -> Self {
        Dialog::new(id).title(title).description(question).body_rows(0)
    }
}
```

<!-- amended by §28 --> **The amended formula (§28 P4).** The prompt/acknowledgement control is a term of its own; without it a `prompt` dialog asks the resolver for a layer three rows shorter than its own content and `draw` clamps the field to `field_h.min(actions_y - y)` — the prompt is silently squeezed or lost, the class of defect N1 exists to remove.

```text
measured_width(d)  = self.width.unwrap_or(d.size.dialog_width)
inner_width(d)     = measured_width(d) − 2 (border) − 2 · d.space.dialog_inset
measured_height(d) = 3                                   // border(2) + title(1)
                   + wrapped_rows(description, inner_width(d))
                   + input_rows(d)                        // NEW
                   + body_block(d)
                   + actions_block

input_rows(d)   = 0                              when there is no prompt and no acknowledgement
                = d.size.field_height            with `.prompt(label)`
                = d.size.field_height + 1        with `.acknowledge(token)`   // the token echo row
body_block(d)   = 0                              when body_rows == 0
                = body_rows + 1                  otherwise (the blank separator row)
                  where body_rows defaults to d.size.code_preview_lines for `Dialog::new`
                  and to 0 for confirm / destructive / prompt / acknowledge
actions_block   = 0 when `actions` is empty, else 2 (the blank row + the action row)
```

The `3` is charged unconditionally: a dialog with no title still reserves its row, so `measured_height` stays a function of the props' *shape* and not of their content. `draw` lays out against exactly these terms, and both halves wrap the description through the same `text::wrapped_rows`, which is why the two cannot drift. `input_rows` is a pure function of `(props, DesignTokens)`, so invariant D1 keeps re-asserting a size the dialog can actually lay out; folding the prompt into `body_rows` and making the *caller* state it was rejected — it re-exports `field_height` into every call site and breaks the moment a theme changes it, the defect §15.1 F4 removed from `Form`.

**Invariant D1 — the dialog re-asserts its size every frame.** `Dialog::update` begins with

```rust
cx.resize_layer(self.id, LayerSize::Fixed(self.measured_width(cx.design()),
                                          self.measured_height(cx.design())));
```

so a description that grows, an error row that appears, or a theme swap corrects the layer on the next draw **without the opener predicting anything**. `Select`, `Picker`, `ContextMenu` and `MenuBar` do the same with their own arithmetic (`Select`: `popup_min_width ≤ w ≤ popup_max_width` from the item labels it already receives per phase, `h = min(items, popup_max_rows) + 2`) — declared as `measured_size(&self, cx, items) -> LayerSize` (§17.0 A7).

Example 9's opener becomes `cx.open_layer(CONFIRM, confirm_dialog().layer(cx));`, where `confirm_dialog()` is the single props constructor §13 already requires — so this also satisfies `architecture::props_are_built_once`.

**One new text primitive is required** (Slice 3 owns `text/`): `pub fn wrapped_rows(s: &str, width: u16) -> u16` — a grapheme/word walk returning the row count, 0 allocations, and **the same function `Dialog::draw` uses** to lay the description out. Without it "pure function of props and tokens" is unverifiable.

**Rationale.** The resolver already runs exactly once per frame, in the runtime, before any drawing; supplying it a real size is the *smallest* change that makes §9.1's "one resolver" true, and the only option that touches no phase boundary. The size is knowable in `update` for every overlay in the inventory: dialogs from tokens + props, popups from the item slice `update` already receives per phase (§17.0 A3), tooltips from the string — and `Cx` already carries `design()` and last frame's `area`/`layout`, which is precisely the toolkit §15.1 F4 uses for the identical problem. Making the size mutable-while-open removes the last reason to want a measure callback: content growth is handled by the component that owns the content, once per frame, in the phase allowed to change runtime state. Renaming `min_size` → `size` corrects a documentation defect, not just a shape.

**Rejected alternatives.**

| Rejected | Why |
|---|---|
| **(a) `LayerSize::Measured` + a measure callback on `Ui::layer`, or a two-pass draw** | Structurally infeasible. `LayerSpec` is `Copy`, `const`-constructible and stored in `LayerStack` **across frames**; a callback that measures real content must borrow app data, which cannot live in runtime state that outlives the frame. The two-pass alternative needs an area for pass 1 — the thing being computed — and doubles a whole draw traversal against §20.9's frame budget. The runtime also cannot construct a `Ui` before the arming loop, because the loop fills `frame.layers`, which `Ui` borrows mutably. |
| **(b) `layout::place(anchor, size, screen)` called by each overlay's content** | Puts flip and clamp back into every component — literally the `ui/popup.rs` + `menu.rs` duplication §9.1 was written to delete. It also makes `LayerDraw.area` permanently the whole screen, so `Ui::layer`'s clip stops bounding anything, cases 8 and 19 become per-component obligations again, and §9.1's "small terminal" row has no owner. |
| **`LayerSize` keeping true minimum semantics (`Min(w,h)` growing to fit)** | Nothing can grow a layer without measuring content, which is (a). A minimum that is silently a maximum **is** the present bug. |
| **`(u16,u16)` retained with `LayerSpec::modal` defaulting to `Fixed(dialog_width, 0)`** | A second sentinel (`h == 0` meaning "unknown") in the field whose first sentinel caused this adjudication; and `modal()` is `const` and cannot read `design.size.dialog_width`. |
| **`Cx::update_layer(id, impl FnOnce(LayerSpec) -> LayerSpec)`** | Lets a caller flip `kind`/`inert_below`/`initial_focus` mid-life, desyncing the focus-scope mode and inert floor the runtime armed at open (§21 item 14). Two narrow geometry setters carry the same ergonomics with none of that. |

**Tests** (§16.1 `layer.rs`, so `every_named_test_exists` stays honest): `layer::fill_resolves_to_the_whole_screen` (replaces the `(0,0)` arm of `anchor_screen_center_sits_in_the_upper_third`); `layer::fixed_size_is_clamped_never_grown` (renames `min_size_then_clamp_then_documented_degradation`; `Fixed(54,20)` on a 40×10 screen equals the screen, `Fixed(0, 8)` equals `Rect::ZERO`); `layer::popover_flips_above_when_the_content_does_not_fit_below`; `layer::point_anchor_flips_instead_of_covering_the_pointer`; `layer::resize_layer_re_resolves_the_anchor_on_the_next_draw`; `layer::spec_geometry_is_the_only_mutable_field`. In `components/dialog.rs` (4F): `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens`, `dialog::draw_lays_out_against_the_height_it_asked_for` (no `Rect::centered*` call appears in `dialog.rs` — an `xtask` forbidden pattern scoped to `components/`), `dialog::confirm_is_centred_by_the_resolver_not_by_the_dialog`, `dialog::a_growing_body_resizes_the_layer_on_the_next_frame`. Conformance: case 19 opens the layer through the component's own spec at each of the 16 tiny screens for `Caps::OVERLAY`; case 14 is unchanged (arming precedes drawing); `conformance::draw_registers_nothing_when_it_cannot_draw` extends to `LayerSize::Fixed(0, h)`.

### 26.2 N2 — `Measure` cannot resolve styles

**Decision.** `Measure::measure(&self, ui: &Ui<'_>, c: Constraints) -> Size` **stands unchanged**. `Ui` gains a `&self` resolution path that bypasses the memo and records nothing; `Theme` gains the surface-independent half for `update`-phase sizing. `Ui::style` / `Ui::style_patched` keep `&mut self` and remain the **only** queries that record roles or styled parts.

```rust
// crates/tui/src/ui/mod.rs
impl Ui<'_> {
    /// Resolve a part through the full §11.3 chain **without** the memo and
    /// **without** recording roles — the `&self` path, for `Measure::measure`
    /// and any read that must not paint.
    ///
    /// Identical to [`Ui::style`] in result (same family/variant/state chain,
    /// same live overlay stack, same current surface); it differs only in what
    /// it does *not* do. Excludes the per-instance patch (precedence 6), which
    /// the caller merges with `StylePatch::merge` if it has one.
    ///
    /// Costs one uncached accumulation (~13 ns) and zero allocations. Use
    /// [`Ui::style`] on the painting path: a measurement must not evict a
    /// painting entry from the 256-slot memo (§11.1 A3, §20.9-2).
    pub fn resolve(
        &self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
    ) -> Resolved {
        crate::theme::resolve::resolve_uncached(
            self.theme, family, variant, part, flags, self.surface,
            &self.core.overlays, None,
        )
    }

    /// The glyph a role currently maps to (`design.glyphs`). `&self`, so it is
    /// reachable from `measure`; pair with `text::width` for its cell width.
    pub fn glyph_str(&self, g: GlyphRole) -> &'static str {
        self.theme.design.glyphs.get(g)
    }
}
```

```rust
// crates/tui/src/theme/mod.rs
/// The surface-independent half of resolution: everything §11.3 settles before
/// roles bind to colours. Available in `update`, where there is no `Surface`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PartMetrics {
    pub glyph: Option<GlyphRole>,
    pub size:  Option<u16>,
    pub align: Option<Align>,
}

impl Theme {
    /// Sizes, glyphs and alignment for a part, with no colour binding and no
    /// overlay stack (an `update` has neither a surface nor a draw-time scope).
    /// This is the sizing path for `Cx`-phase arithmetic — `Form`'s field
    /// height (§15.1 F4) and `Dialog::layer` (N1).
    pub fn metrics(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> PartMetrics;

    /// Unchanged; already present.
    pub fn resolve(&self, f: Family, v: Variant, p: Part,
                   s: StateFlags, surface: Surface) -> Resolved;
}
```

`Theme::resolve` is refactored to `metrics` + colour binding so there is exactly **one** `accumulate` path; `resolve::bind` already separates the two concerns (it uses `surface` only for colour, copying `glyph`/`size`/`align` straight off the accumulated patch), so this is a factoring, not new logic.

**Worked example — the case that forced this** (4A, a button with a leading glyph):

```rust
impl Measure for Button<'_> {
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let flags = ui.state(self.id);
        let gutter = ui.resolve(Family::BUTTON, self.variant, Part::GUTTER, flags);
        let g = gutter.glyph.map_or(0, |r| text::width(ui.glyph_str(r)));
        let pad = ui.design().space.inline;
        let w = g.saturating_add(pad)
                 .saturating_add(text::width(self.label))
                 .saturating_add(pad);
        let h = ui.resolve(Family::BUTTON, self.variant, Part::CONTAINER, flags)
                  .size.unwrap_or(1);
        Size::exact(w, h).fit(c)
    }
}
```

**Invariant M1.** `Ui::style` and `Ui::style_patched` are the *painting* queries: they alone write the per-cell roles `dim_layer` reads and they alone feed the `testing` record. **`Ui::resolve`, `Theme::resolve` and `Theme::metrics` record nothing.** This is required, not incidental: `conformance::registry::declared_parts_are_the_parts_actually_styled` compares recorded parts against `Self::PARTS`, so a component that *measures* a part it never paints (common — measuring `Part::META` to decide it does not fit) would otherwise report a part it never styled and the check would pass on a false positive. Symmetrically, a component that resolves through `Ui::resolve` and then paints must go through `Ui::style` for the paint, which it does anyway, because painting needs the roles set. §25 F7's widening of `styled_parts` to `(Id, Family, Variant, Part, Resolved)` lands on the painting path only and is unaffected.

**Cache and allocation obligations (§20.9‑2).** The memo stays `Box<[(u64, u32, StylePatch); 256]>` — one allocation at construction, generation-stamped, unchanged. `Ui::resolve` performs **zero** cache reads and writes and **zero** allocations, so `style_resolve_10k_parts`' 0-alloc assertion and the ≥ 90 % hit-rate assertion of §25.8 are both unperturbed by measurement. Measurement is O(components) per frame, not O(cells) — the reason the memo exists (rows re-resolving one tuple) does not apply to it, and routing measurement through the cache would cost hit rate on the path that actually needs it.

**Rejected alternatives.**

| Rejected | Why |
|---|---|
| **(b) `measure(&self, ui: &mut Ui<'_>, c)`** | (i) Rewrites **eleven** already-declared signatures for one capability the `&self` path already provides. (ii) It hands `measure` the whole painting and registration surface, destroying the property `layout::rows_measured` depends on — that measuring has no frame side effects — so `draw_twice_is_byte_identical` (case 5) would then depend on how many times a layout primitive measured. (iii) It lets measurement thrash the 256-slot memo. (iv) `FieldControl::measure` is called from `Field::measure` while the caller holds `&mut Ui` for the surrounding paint; §17 example 9's `rows_measured(body, &tracks, &[props.measure(ui, c).preferred.1])` shape becomes borrow-fragile. |
| **(c) `measure(&self, theme: &Theme, design: &DesignTokens, c)`** | Cannot see `Surface`, so it cannot produce a `Resolved` at all; cannot see the **overlay stack**, so a scoped `Overlay` that sets `.size(n)` would be honoured by painting and ignored by measurement — the component measures 10 and paints 12, with no test able to see it; and cannot see `FrameRead::{state, area, layout}`, which `Form::measure` (F4) and `List::measure` need. It also duplicates two parameters `Ui` already carries, for a doc-wide rewrite. |
| **Interior mutability on `StyleCache` so `Ui::style(&self)` works** | Makes the memo's hit/miss statistics — the mechanism §25.8 promoted to the binding assertion — observable from a `&self` context, and re-opens the "is `draw` pure" question §5 R2 closed. The roles write is genuinely `&mut` state and cannot be laundered. |

**Tests** (§16.1): `measure::ui_resolve_equals_ui_style_for_every_family_variant_part` — a differential over the built-in recipe set × both themes × a state sweep, with and without a pushed `Overlay`, field-for-field; this is the test that keeps a second resolution path from drifting. Plus `measure::measure_records_no_roles_and_no_styled_parts`, `measure::measure_does_not_touch_the_style_cache`, `measure::natural_width_follows_the_themed_glyph` (a theme rebinding `GlyphRole::FocusBar` to a 2-cell glyph widens `Button::measure` by exactly one column; the same button under `Theme::junie()` does not), `measure::measure_is_allocation_free` (perf), `theme::metrics_are_surface_independent`, `theme::metrics_is_the_sizing_path_for_update`.

### 26.3 The three confirmations

**`Ui::with_part(...)` — CONFIRMED, with one amendment.** The review's ceremony complaint is real: `Ui::style` is `&mut self`, so `ui.fill(cell, ui.style(...).style)` is a borrow error and every component must bind a temporary.

```rust
impl Ui<'_> {
    /// Resolve `part` once and paint with it. Equivalent to binding
    /// `let r = ui.style(family, variant, part, flags);` and then painting —
    /// including the memo lookup and the per-cell role recording `dim_layer`
    /// reads — but expressible as one statement.
    ///
    /// Binds a value only: it pushes **no** clip and **no** surface. Use
    /// `with_area` / `with_surface` for those.
    pub fn with_part<R>(
        &mut self,
        family: Family,
        variant: Variant,
        part: Part,
        flags: StateFlags,
        f: impl FnOnce(&mut Ui<'_>, Resolved) -> R,
    ) -> R {
        let r = self.style(family, variant, part, flags);
        f(&mut self.reborrow(), r)
    }
}
```

*Amendment:* the name must not imply a pushed scope, so the doc says so explicitly, and it is a **convenience, not a replacement** — `style_patched` (precedence 6) has no `with_` form and components with a per-instance patch keep the two-step shape. Test `ui::with_part_resolves_once_and_records_the_role` (one cache miss; roles set at the painted cells).

**`Ui::surface_style() -> Style` — CONFIRMED, plus one amendment that makes it hard to forget.** It returns the style a child inherits from the current surface: `bg = theme.bg(ui.surface())`, `fg = Role::Fg(FgStep::Primary)` bound on that surface, no modifiers — the **left** operand of §11.3's final layering. §22.2 item 10 requires `inherited.patch(resolved.style)` at every paint, and the review found **no production call site performing it**; leaving it as a two-noun idiom guarantees half the Slice‑4 components will forget. So `Resolved` gains the fused form, which is the shape a component actually wants:

```rust
impl Resolved {
    /// This part's style layered over an inherited one — §11.3's final step,
    /// `Style::patch` semantics (modifier symmetry, §22 R‑9).
    #[must_use]
    pub fn over(self, inherited: Style) -> Style { inherited.patch(self.style) }
}
```

Call site: `ui.fill(area, r.over(ui.surface_style()))`. Tests: `ui::surface_style_is_the_left_operand_of_the_final_patch` (differential against a hand-written `inherited.patch(..)` over every surface × both themes) and `theme::patch_merge_matches_ratatui_style_patch_for_modifiers` extended to route through `Resolved::over`.

**`Ui::scroll_region(...)` — out of scope.** It is a Slice‑3-owned `Ui` method blocking 4E, not 4A or 4F. **Recorded as an open item, not decided here** (§25.11).

### 26.4 Risks

1. **`LayerSize` is `#[non_exhaustive]`, so a future `Measured`/`Content` variant is additive** — but adding one still requires solving the callback-lifetime problem above. Recorded so a later slice does not re-litigate N1 by adding a variant it cannot implement.
2. **`Dialog::measured_height` and `Dialog::draw` can drift.** Mitigated by `dialog::draw_lays_out_against_the_height_it_asked_for` and by `text::wrapped_rows` being the single wrap both use. Same risk and same mitigation as §15.1 F4's field height.
3. **`Ui::resolve` and `Ui::style` can drift.** Mitigated by `measure::ui_resolve_equals_ui_style_for_every_family_variant_part`, a full differential rather than a spot check.
4. **`resize_layer` called every frame is a per-frame write.** A compare-and-store on a `Copy` field, no allocation, no repaint unless the value changed; `frame_showcase_lists_120x40`'s allocation budget is unaffected.
5. **Uncached measurement cost.** ~13 ns per part × the handful of parts a component measures × the components measured per frame — orders below the ≤ 5 % per-frame style budget §25.8 sets. Reported by `measure::measure_is_allocation_free`'s companion ns line, asserted only under `PERF_STRICT=1`.
6. **`Anchor::Point` flipping is a visual change** for any existing tooltip or context menu near a screen edge. It is classified as **§20.10 item 17** and requires a `docs/visual-changes.md` entry before any baseline that moves is blessed (`xtask bless-guard`, §16.3).

### 26.5 Acceptance conditions (executable, from the workspace root)

```bash
# N1 — the resolver is given a real size, and it clamps rather than grows
cargo test -p tui-next --lib layer::tests::fill_resolves_to_the_whole_screen
cargo test -p tui-next --lib layer::tests::fixed_size_is_clamped_never_grown
cargo test -p tui-next --lib layer::tests::point_anchor_flips_instead_of_covering_the_pointer
cargo test -p tui-next --lib layer::tests::popover_flips_above_when_the_content_does_not_fit_below
cargo test -p tui-next --lib layer::runtime_tests::resize_layer_re_resolves_the_anchor_on_the_next_draw
! rg -n 'min_size' crates/tui/src            # the false name is gone from the source
# N1 — no component computes an overlay rect (the "one resolver" claim, mechanically)
! rg -n 'centered|centered_horizontally|centered_vertically|resolve_anchor' crates/tui/src/components/
# N2 — the two resolution paths cannot drift, and measurement has no side effects
cargo test -p tui-next --lib measure::tests::ui_resolve_equals_ui_style_for_every_family_variant_part
cargo test -p tui-next --lib measure::tests::measure_records_no_roles_and_no_styled_parts
cargo test -p tui-next --all-features measure::tests::measure_does_not_touch_the_style_cache
cargo test -p tui-next --lib theme::tests::metrics_are_surface_independent
# the confirmations
cargo test -p tui-next --lib ui::tests::with_part_resolves_once_and_records_the_role
cargo test -p tui-next --lib ui::tests::surface_style_is_the_left_operand_of_the_final_patch
# the memo and allocation contracts are unperturbed
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve
cargo test -p tui-next --test perf --release -- --test-threads=1 measure_is_allocation_free
# the inventory and the document agree
cargo test --workspace --test architecture every_named_test_exists
cargo run -p xtask -- doc-check
rg -n 'Adjudication N' COMPONENT_ARCHITECTURE.md REFACTORING_STATE.md
```

In this document `min_size` survives **only** as struck historical prose in §21 item 20 and in the two places where the false name is named in order to be retired (§9.1's `LayerSpec.size` comment and §17.0 A6); `App::min_size` is an unrelated and unchanged name.

**Gate.** Every command exits 0; the §9.1 / §9.2 / §10 / §11.1 A3 / §11.3 / §11.5 / §16.1 / §16.2 / §17.0 A2·A6·A7 / §17 examples 9–10 / §18.1 / §20.10 item 17 / §21 item 20 amendments above are applied and mirrored in `REFACTORING_STATE.md`. On that condition **4A may start** (N2 unblocks `Button::measure`) and **4F may be scheduled** (N1 unblocks `Dialog`, `Select`'s popup, `Picker` and `ContextMenu`).

---

## 27. Adjudication O — Slice 3 foundations follow-ups

**Source.** `docs/reviews/adjudication-o-foundations-followups.md`, adjudicating the four research requests the Slice 3 foundations correction pass (F1–F26 + Adjudication N) returned. Each was a conflict between two already-accepted statements; each item decides only **which side moves**. Nothing here reopens Adjudications A–N. **Accepted as written.** Every earlier section it changes carries an inline `<!-- amended by §27 -->` marker: §11.2, §16.1, §16.6 (three rows), §20.9‑1, §20.9‑2, §21 item 6, §24.2 (three places), §25.3 (three places), §25.6, §25.8 (two rows). Arithmetic marked **[derived]** was recomputed from the checked-in token values; the review's `(estimate)` figures are corrected where they were wrong. `REFACTORING_STATE.md` (coordinator-owned) mirrors the four decisions per the change-control rule at the top of this document.

### 27.1 O1 — the style memo is **two-way set-associative**, not one-way; ≥ 90 % is re-purposed

**Decision. Accept the shipped 2-way memo. Amend §20.9‑2 and the module doc that contradicts it. Keep ≥ 90 % as the perf-gate floor, but restate in §16.6 and §25.8 what it is for, because it is a statistical property and the document read as though it were a guarantee.**

**What §20.9‑2 actually asserted survives unchanged**: statically sized, one construction-time allocation, no per-frame allocation, no growth, no `HashMap`, no `Vec`, generation-stamp clearing. The single clause 2-way contradicts is ~~there is no eviction policy to get wrong~~, struck in §20.9‑2 and replaced honestly rather than glossed. ~~direct-mapped~~ is a sketch detail of the same class as ~~`[Option<(u64, Resolved)>; 256]`~~ — which §25/§26 already corrected in prose — not an invariant. §11.1 A3 needed **no** change: it says only *"a small per-frame array cache (≤ 256 entries, cleared each frame)"* and never claimed one-way mapping.

**The collision arithmetic [derived].** The shipped benchmark key set is `Family::LIST × Variant::DEFAULT × {CONTAINER, GUTTER, MARKER, META} × 8 states` = **32 distinct keys**, 10 000 queries per draw, cache cleared per draw.

* **Colliding pairs, one-way, 256 slots:** `C(32,2)/256 = 496/256 = 1.94`.
* **Keys that thrash:** balls-in-bins gives `32·(255/256)^31 = 28.35` keys alone, so `≈ 3.65` keys sit in multi-key slots. Access order is round-robin over all 32 keys, so **every** key in a shared slot misses on **every** access — LRU-hostile by construction.
* **Predicted rate:** `(10 000 − 3.65·312.5 − 28)/10 000 = 88.3 %`; **measured 87.2 %**. Consistent.
* **No hash fixes it.** 88 % is the birthday load of 32 keys over 256 buckets — a property of *any* uniform hash, not of FNV. Only a perfect hash over a statically known key set would help, and the key set is not statically known.
* **Two ways:** a miss needs ≥ 3 keys in one set. Poisson with `λ = 32/128 = 0.25` gives `P(set ≥ 3) = 0.00216`, × 128 sets = **0.28 expected sets**, matching `C(32,3)/128² ≈ 0.3`. Expected rate **≈ 99.7 %**.
* **The benchmark understates the problem.** A real frame does not resolve 32 keys: §16.6's `frame_showcase_lists_120x40` covers a sidebar list, buttons, tabs, panels and a status bar — order 100–300 distinct `(family, variant, part, flags)` tuples. At 200 keys a 256-slot one-way table has load factor 0.78 and thrashes catastrophically. 87.2 % is a synthetic **best** case, not a floor.

**Allocation and worst-case-latency consequences.**

| | One-way, 256 slots (the struck sketch) | Two-way, 128 sets (shipped) |
|---|---|---|
| Allocations | one `Box`, construction time | **identical** — `style_resolve_10k_parts` allocs = 0 |
| Memory | 256 entries | **identical** (`CACHE_SLOTS = 256`) |
| Per-frame cost of `clear` | one `u32` bump | **identical** |
| Hit, way 0 | 1 tag + generation compare, no write | **identical** |
| Hit, way 1 | — | 2 compares + `promote` copies 2 entries ≈ 128 B — the only new hit-path cost, and only for keys sharing a set |
| Miss | 1 compare + `accumulate` + 1 write | 2 compares + `accumulate` + a 1-entry shift + 1 write |
| Measured | 87.2 % hit rate | **12.0 ns/query** (`120 141 ns / 10 000`), inside §25.8's accepted ≈ 13 ns; ≈ 99.7 % hit rate |

Every path is O(1) and independent of the key count. **No new worst case.**

**Rejected alternatives.**

* **Revert to one-way and lower the threshold below 87 %.** Rejected: it re-admits a memo that misses ≈ 12 % on a 32-key set and far worse on a real frame, for no saving — two ways cost the same 256 entries, the same one allocation, and one extra tag compare on a miss. Lowering the bar to accommodate a worse mechanism inverts the purpose of the gate.
* **Change the benchmark's key distribution.** Rejected on the evidence: a realistic frame has *more* distinct keys, so a "realistic" distribution makes the rate worse and the assertion more fragile. The benchmark is not unrepresentative in the direction the request assumed.
* **`WAYS = 4` (64 sets).** Would make ≥ 90 % robust under any hash (expected sets with ≥ 5 of 32 keys ≈ 0.014). Rejected: a 4-way tag scan touches four ~64-byte entries — four cache lines — on the **hit** path, which is the 12 ns budget §25.8 accepted, and the robustness it buys is already provided deterministically by the unit test. Revisit only if tags are split into their own array.
* **Drop `promote` (insert-at-way-0, never promote on hit).** Provably equivalent for ≤ 2 keys per set, i.e. 99.7 % of sets, and it removes the only *write* from the hit path. Not adopted: the difference is unmeasurable, the code is landed and gated green, and MRU is the standard, least-surprising policy. Recorded so a later reader does not re-derive it.

**Code obligations (owned by the foundations builder, not by this section).**

1. `crates/tui/src/theme/resolve.rs`'s module doc still reads *"statically sized ~~direct-mapped~~ cache"* 273 lines above its own `WAYS` const — replace with *"statically sized two-way set-associative cache (256 entries in 128 sets)"*. A file whose header contradicts its own constant is a landmine.
2. **Generation wrap is a latent correctness bug.** `self.generation.wrapping_add(1).max(1)` returns to 1 after 2³² clears, at which point a slot stamped with the original generation 1 becomes a false hit serving a stale `StylePatch`. Fix: at `u32::MAX`, fill the slots and reset the stamp to 1; otherwise `saturating_add(1)`. Cost is one comparison per frame, and the 256-entry fill runs once per 2³² frames.
3. `crates/tui/tests/perf_baseline.txt` — record the measured two-way hit rate in the `#` header beside the F18 notes, so the number is reviewed in a diff rather than living only in CI scrollback.

**Risk.** At 256 entries, no associativity makes a hit rate a *guarantee*: with 32 keys over 128 two-way sets, a hash configuration putting three keys in one set yields ≈ 90.3 % and two such sets ≈ 81 % — a ≈ 5 % chance of a sub-90 % rate under an unrelated renumbering of `Part`/`Family`/`Variant` or a change to `fnv1a`. Mitigated by the deterministic unit test, which is hash-independent, and by diagnosing the geometry from the `PERF-CACHE` line before suspecting the key.

**Tests and acceptance.**

```bash
cargo test -p tui-next --lib theme::resolve::tests::cache_hits_after_the_first_query_and_clears_by_generation
cargo test -p tui-next --lib theme::resolve::tests::cache_generation_wrap_does_not_serve_a_stale_entry   # new
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve_10k_parts
! rg -n 'direct.mapped' crates/tui/src            # the literal survives in this document only inside ~~strikeouts~~
rg -n 'two-way set-associative' COMPONENT_ARCHITECTURE.md crates/tui/src/theme/resolve.rs
```

`theme::cache_generation_wrap_does_not_serve_a_stale_entry` — seed a key at generation 1, set `generation = u32::MAX`, `clear()`, assert the same key **misses**.

### 27.2 O2 — `borders_set(border::ASCII)` rebinds glyphs: coupling **confirmed**, mechanism amended, full table **scheduled for Slice 4E**

**Decision — the middle design.** (a) is confirmed on the substance: the coupling is necessary and the four glyph values are right. (b) as stated — a separate `ascii_glyphs()` step *plus* narrowing the test to border cells — is rejected. The mechanism moves to a named, reusable, whole-set step, and the full table is dated.

**Why the coupling is necessary.** `Ui::rule` reads `GlyphRole::RuleQuiet` from `theme.design.glyphs`; `Ui::frame` reads `design.borders`. They are independent stores, so a border-only swap leaves `─` in every divider. §24 M2's own rejection of automatic ASCII selection rests on reason (2) — *"a border-only auto-switch renders a frame that is ASCII at the edges and unicode everywhere else, worse than either consistent choice"* — and that argument applies with identical force here. A border-only `borders_set(ASCII)` **is** the outcome M2 called worse than either consistent choice.

**Role-by-role verification [derived].** Auditing every entry of Junie's 39-glyph table plus the typed sets: `▎ U+258E`, `› U+203A`, `✓ U+2713`, `● U+25CF`, `• U+2022`, `− U+2212`, `▲ U+25B2`, `▸ U+25B8`, `▾ U+25BE`, `▴ U+25B4`, `∇ U+2207`, `▪ U+25AA`, `→ U+2192`, `↓ U+2193`, `‹ U+2039`, `… U+2026`, `× U+00D7`, `◆ U+25C6`, `◇ U+25C7`, `∥ U+2225` — **none** falls in `U+2500..=U+257F`. `RuleQuiet`, `RuleActive`, `ScrollTrack` and `ScrollThumb` are *exactly* the roles whose Junie binding lies in the box-drawing block, which is exactly the scan range of `theme::ascii_theme_renders_without_box_drawing_glyphs`. The coupling is complete with respect to a **statable principle** — *every glyph whose default falls in the box-drawing block* — not to whatever the test happened to catch.

**The four values, against `DESIGN.md:552-563`.**

| Role | Junie | ASCII | Verdict |
|---|---|---|---|
| `RuleQuiet` | `─` | `-` | The direct ASCII equivalent, one stroke, width 1 (`DESIGN.md:555`) |
| `RuleActive` | `━` | `=` | Must read **heavier** than quiet: two strokes against one, the conventional ASCII heavy rule, and it survives monochrome where weight is the only channel left (`DESIGN.md:557`, `:321-322`). `#` was the alternative and reads as hatch/fill, not a rule |
| `ScrollTrack` | `│` | `\|` | Direct equivalent, width 1 (`DESIGN.md:491`, `:560`) |
| `ScrollThumb` | `┃` | `#` | Must read **denser** than the track in monochrome: one stroke → crosshatch is an unambiguous density step. `H`, `*`, `+` were considered — `+` collides with `border::ASCII`'s corners, `*` reads as a marker, `H` as a letter |

All four are ASCII, one byte, width 1 — the same property `theme::ascii_border_set_is_pure_ascii` pins for the border set.

**Mechanism — a whole-set swap, not four fields.** `ThemeBuilder::ascii_glyphs()` becomes a public, named, idempotent, `#[must_use]` step that replaces the **typed sets**: `glyph::ASCII_RULE_QUIET`, `glyph::ASCII_RULE_ACTIVE` and `glyph::ASCII_SCROLLBAR`, through three whole-set mutators on `GlyphSet` beside the per-role ones. `borders_set(border::ASCII)` calls it. This closes, in the same edit, two gaps the per-role form could not:

1. **`scrollbar::Set.begin` / `.end` stayed `│`.** No `GlyphRole` reaches them, so nothing — not even a manual `.glyph(..)` — could make them ASCII. Invisible today only because no component paints scrollbar caps; `ScrollRegion` is 4E's file, and this would have broken `theme::ascii_theme_renders_without_box_drawing_glyphs` the day 4E landed.
2. **`line::Set`'s other ten fields stayed box-drawing** — `vertical`, `cross`, four corners, four tees. §22.2 item 12 designates `line::Set` for *"rules and seams"*; the first seam painter would have leaked.

Two further findings are recorded, not fixed: the swap is **sticky and order-dependent** — `borders_set(ASCII).borders_set(PLAIN)` keeps ASCII rules, and `.glyph(RuleQuiet, "~").borders_set(ASCII)` silently discards the author's explicit glyph, unlike every colour setter, which honours `Explicit`; and Junie's `GLYPHS` array slots for `─ ━ │ ┃` are **shadowed dead data**, since `GlyphSet::get`/`set` route those four roles to the typed sets and never index the array — harmless, misleading, and it would mislead the author of the ASCII table, so those entries are named dead in the source.

**Scheduling.** The full `GlyphSet` ASCII fallback table is **scheduled for Slice 4E, not deferred indefinitely** — see §24.2. `ascii_glyphs()` covers the box-drawing block completely; the remaining ~31 roles are a visual-design decision against `DESIGN.md`'s marker table and belong with 4E's own review plus re-blessed baselines under §20.10.

**Rejected alternatives.**

* **`ascii_glyphs()` as a *separate* step, with the test narrowed to border cells.** Rejected twice over. Narrowing the scan destroys the only mechanism that surfaced the coupling — the test would then pass on a frame that is ASCII at the edges and box-drawing everywhere else, which §24 M2 declared worse than either consistent choice. And it makes correctness depend on the author remembering a second call, with a silent, visible-only-at-runtime failure mode. `ascii_glyphs()` is kept as a public, named step; what is rejected is making it the **only** path.
* **Add `GlyphRole::ScrollBegin` / `ScrollEnd`.** Would reach `begin`/`end` through the per-role API, but widens `GlyphRole::ALL` from 39 to 41, touches every `GlyphSet` literal in `junie.rs` and `paper.rs`, and is a §11.2 amendment — all to solve by enumeration what a whole-set setter solves structurally. The argument holds tenfold for `line::Set`'s eleven fields.
* **Schedule the full table now, in Slice 3.** Rejected: ~31 glyph choices against `DESIGN.md`'s marker table plus re-blessed baselines is a fresh visual adjudication, and §24 M2 already ruled it so. Rejected equally is leaving it open-ended — 4E is a dated forcing function.

**Risks.**

1. `ascii_glyphs()` still leaves ~31 unicode glyph roles; an author may read "ASCII theme" as a full guarantee. Mitigated by the rustdoc on `ascii_glyphs()` and by §24 M2 risk 3, both of which say the scan is the box-drawing block only.
2. `borders_set(ASCII).borders_set(PLAIN)` keeps ASCII rules. Documented, not fixed: restoring the theme's own glyphs would clobber a deliberate `.glyph(..)`.
3. The new `theme::ascii_glyph_set_has_no_box_drawing` **fails today** on `scroll.begin`/`.end`. That is the intended effect — it converts a latent 4E failure into a Slice-3 one.

**Tests and acceptance.**

<!-- amended by §28: the render line names both targets (P2) -->

```bash
cargo test -p tui-next --test render --test render_components theme::ascii_theme_renders_without_box_drawing_glyphs   # §28: both targets
cargo test -p tui-next --lib theme::glyph::tests::ascii_glyph_set_has_no_box_drawing                      # new
cargo test -p tui-next --lib theme::builder::tests::ascii_glyphs_is_idempotent_and_glyph_overrides_it     # new
cargo test -p tui-next --lib theme::border::tests::ascii_border_set_is_pure_ascii
rg -n 'ascii_glyphs' crates/tui/src/theme/builder.rs COMPONENT_ARCHITECTURE.md
rg -n 'Slice 4E' COMPONENT_ARCHITECTURE.md
```

`theme::ascii_glyph_set_has_no_box_drawing` is **component-free**, so it is not hostage to which painters exist: build `Theme::junie().builder().borders_set(border::ASCII).build()`, iterate `GlyphRole::ALL` through `GlyphSet::get` **plus every field** of `scrollbar()`, `rule_quiet()` and `rule_active()`, and assert no `char` in `'\u{2500}'..='\u{257F}'`. This is the assertion the render test can only approximate. `ascii_glyphs_is_idempotent_and_glyph_overrides_it` — `.ascii_glyphs().ascii_glyphs()` equals `.ascii_glyphs()`; `.borders_set(ASCII).glyph(GlyphRole::RuleQuiet, "~")` yields `"~"`.

### 27.3 O3 — `border_subtle` downgrades to **`DarkGray`**; the document was wrong, and two ΔE estimates with it

**Decision. The implementation is right and this document was wrong.** ~~`border_subtle` → `Black`~~ is struck in §16.1 and §25.3. `border_subtle = WHITE_15 = #262626 = (38, 38, 38)`: channel spread `38 − 38 = 0 < 40` → grey ladder; BT.601 luma `(38·299 + 38·587 + 38·114)/1000 = 38`; `38 ∈ 31..=110` → **`Color::DarkGray`**. `#111111` (luma 17) is the value that reaches `Black`, and it is `surfaces[1]`, not `border_subtle`.

**It was never a carried fact.** The legacy pin `theme::tests::accent_survives_downgrade` asserts exactly three things — `accent → LightGreen`, `error → LightRed`, `canvas → Black` (`canvas = #000000`, luma 0). It says nothing about `border_subtle`. The claim was invented while paraphrasing the legacy contract, and the review marked its own colour arithmetic *(estimate)* for exactly this reason: the re-derivation obligation did its job.

**Every colour claim in §16.1 / §25.3, re-derived [derived]** against `theme/downgrade.rs` and `theme/builtin/junie.rs`:

| Token | Value | spread | luma / bright | Result | Doc said | Verdict |
|---|---|---|---|---|---|---|
| `accent` | `#48e054` | 152 | max 224 > 180 | `LightGreen` | LightGreen | ✔ |
| `danger` | `#e44545` | 159 | max 228 > 180 | `LightRed` | LightRed | ✔ |
| `danger_soft` | `#d98a8a` | 79 | max 217 > 180; `g = 138 > 120` but `b = 138 ≮ 80`, so not Yellow | `LightRed` | LightRed | ✔ |
| `border_subtle` | `#262626` | 0 | luma 38 | **`DarkGray`** | ~~Black~~ | ✘ **wrong, corrected** |
| `fg[1]` | `#b3b3b3` | 0 | luma 179 ∈ 111..=200 | `Gray` | Gray | ✔ |
| `fg[0]` | `#ffffff` | 0 | luma 255 | `White` | — | ✔ (pinned in code) |
| `surfaces[1]` | `#111111` | 0 | luma 17 ∈ 0..=30 | `Black` | — | ✔ (pinned in code) |
| `warning` | `#f59e09` | 236 | `g = 158 > 120 ∧ b = 9 < 80` | `Yellow` | Yellow | ✔ |
| `info` | `#8787ff` | 120 | neither r nor g dominant, max 255 > 180 | `LightBlue` | — | ✔ (pinned in code) |
| `accent_pressed` | `#2b8632` | 91 | max 134 ≤ 180 | `Green` | — | ✔ — proves the dark half is reachable |
| `highlight_danger_bg` | `#7a2a2a` | 80 | max 122 ≤ 180 | `Red` | — | ✔ (pinned in code) |
| `Indexed(196)` | → `(255, 0, 0)` | 255 | max 255 > 180 | `LightRed` | — | ✔ (pinned in code) |

Also confirmed: the restored `nearest_16` is the legacy `src/theme.rs` implementation **verbatim**, modulo saturating arithmetic and a dropped unused match arm. F5 is discharged correctly.

**The two CIE76 estimates, re-derived.** Both were flagged *(estimate: re-derive before blessing)*. They are rationale for a **rejected** metric, so they gate nothing, but the obligation stands and both were off:

* **§25.3 reason 2** — `danger_soft #d98a8a` is L\*a\*b\* (65.6, 30.2, 12.7). ΔE to `DarkGray (127,127,127)` = **35.0**; to `Red (205,0,0)` = **62.6**; to `LightRed (255,0,0)` = 75.1; to `Gray (229,229,229)` = 41.4; to `White` = 47.5. The minimum is `DarkGray`, so the **conclusion is confirmed**; the numbers `≈ 30` / `≈ 61` become **35.0** / **62.6**.
* **§25.3 reason 3** — `#48e054` is L\* **79.2** (doc: ≈ 78); `Green (0,205,0)` is **72.0**; `LightGreen (0,255,0)` is **87.7**. Full ΔE: **17.8** to `Green` against **34.9** to `LightGreen`. The dark primary wins by nearly 2× — **confirmed strongly**; the review understated its own case.

**Rejected alternative. Change the implementation so `border_subtle` reaches `Black`**, by widening the `0..=30` band or special-casing chrome. Rejected outright: it would alter `nearest_16`'s categorical bands to satisfy a sentence that was never a contract, while `DESIGN.md:313-322` fixes only accent, error and the surviving glyph/modifier language. Restoring the legacy metric verbatim is F5's whole point (§25.3 reason 4); tuning it to match a paraphrase inverts the authority order.

**Risk.** Only that the corrected value is itself unverified — it is not: the arithmetic is exact integer BT.601 over a checked-in `const`, and the unit suite already pins both `#262626 → DarkGray` and `#111111 → Black` as separate assertions. **No baseline is re-blessed**; the correction restores the recorded output.

**Tests and acceptance.**

```bash
cargo test -p tui-next --lib theme::downgrade::tests::ansi16_preserves_hue_family_and_brightness
cargo test -p tui-next --lib theme::downgrade::tests::downgrade_is_deterministic_per_level
cargo test --all-targets theme::tests::accent_survives_downgrade          # the legacy pin, unchanged
git diff --exit-code crates/tui/tests/perf_baseline.txt                   # O3 re-blesses nothing
```

### 27.4 O4 — the two perf substitutes, with three corrections

#### (a) `style_resolve_per_frame` — substitute **confirmed in principle**, reinstated in **Slice 5**

`frame_showcase_lists_120x40` lives in `apps/showcase/tests/perf.rs`, which **Slice 5 owns**, and does not exist; the stand-in measures a 40-row × 5-part frame twice, styles resolved per row versus hoisted, and takes the difference. Substituting §25.8's own machine-independent arithmetic — *"≈ 13 ns × ~2 000 style queries per realistic frame ≈ 26 µs, under 0.2 % of a 16 ms budget"* — for a share of a frame that does not exist is the right move, and it is the same move §25.8 made when it struck the unmeetable 2× ratio. The 0.027–0.060 share straddling ≤ 5 % is exactly the symptom of measuring a 3–6 % difference between two independently-taken medians on a shared runner.

**Correction 1 — the extrapolation multiplier is wrong.** Arm A performs **200** `ui.style` calls (40 rows × 5 parts); arm B performs **40** (8 states × 5 parts, hoisted out of the loop). The difference covers **160** queries, not 200, so `× 10` extrapolates to 1 600 queries and the assertion is ~20 % **weaker** than it claims. It must be `× 12.5`: `resolution_ns × 2 000 / 160 ≤ 32 000`.

**Correction 2 — the asserted budget must not come from the noisy estimator.** `resolution_ns = a.ns − b.ns` is 3–6 % of `a.ns`, while run-to-run median noise on `a.ns` alone is comparable. The binding budget moves to the low-noise measurement that already exists: `style_resolve_10k_parts` is a pure resolution loop at `120 141 ns / 10 000 = 12.0 ns` per query with no differencing at all. So **assert** there, under `PERF_STRICT`, `ns / 10 000 × 2 000 ≤ 32 000` — **≤ 16.0 ns per query**, absolute, machine-independent, one-sided, 33 % headroom, and *literally* §25.8's sentence turned into code — and **report** the in-situ share in `style_resolve_per_frame`, keeping the corrected `× 12.5` extrapolation as a second, looser strict-mode net, because it is the only measurement that includes real painting alongside resolution and it is the number Slice 5 will compare against the real frame.

**Correction 3 — the test emits no baseline line.** `style_resolve_per_frame` never calls `report`, so it is absent from `perf_baseline.txt` while §16.6 requires additions to be marked there. Resolved by adding a `#`-header line naming it as a differential test that deliberately carries **no** baseline, in preference to calling `report`: a baselined `ns` for a differential invites a meaningless `× 1.2` regression check.

**Reinstatement — exactly when and against what.** **Slice 5**, in `apps/showcase/tests/perf.rs::frames`, against `frame_showcase_lists_120x40`, as part of the Slice 5 gate. Not 4x: no work package before Slice 5 owns a showcase frame. When the showcase list page exists, add `style_resolve_share_of_frame_showcase_lists_120x40`, measuring the same A/B differential against the real frame and asserting **≤ 5 %** under `PERF_STRICT`; at that point `style_resolve_per_frame`'s extrapolation drops from *asserted* to *reported*.

**Rejected alternatives.** *Assert the ≤ 5 % share against the stand-in* — rejected: the stand-in is the style-densest frame constructible from foundations (five resolutions per painted row, no chrome, no borders, no status bar), so its share is an upper bound on the real one, and asserting an upper bound against a threshold written for the real frame either fails spuriously or passes vacuously. *Delete the test until Slice 5* — rejected: §25.8's budget is the only style-cost bound that binds today, and deleting it would leave §20.9‑1's acceptance column naming a test that does not exist, the exact failure mode `every_named_test_exists` (F12) was added to prevent.

**Risk.** The `× 12.5` differential can still flake at ~2× noise. Named and accepted, because the **binding** budget now comes from `style_resolve_10k_parts`; the differential is a secondary net.

#### (b) `intents_drain_is_o_1_when_the_queue_is_empty` — substitutes **confirmed**, two document corrections

**Facts, all verified.** The 14.9× raw ratio is real and is the stub's **own** cost: `Probes::update` is `for i in 0..self.0 { cx.intents(..).count() }` and `Probes::draw` registers `n` controls — O(n) by construction — and the raw ratio is reported with `strict = false`. Zero probes on an empty queue is **structural**, not statistical: `IntentQueue::iter` returns before `bucket_index` when `used == 0`, and `bucket_index` is the only site that bumps the counter. `probes(500) − probes(20) == 480` is the differential form, and it is necessary: `probes()` also counts the enqueue path and `was_drained`, so no absolute count is stable, while the difference cancels every constant. Allocations are 0 on both the empty and the one-intent path. The normalised ratio **[derived]**: baseline `ns₅₀₀ = 632 ns`, so at 14.9× `ns₂₀ ≈ 42 ns`, and `(632 × 20)/(42 × 500) = 0.602` — the reported 0.60 confirmed, asserted `≤ 1.25` under `PERF_STRICT`. It is a genuine detector, not theatre: with `s = C + n·k` it reads `(20C + 10000k)/(500C + 10000k)`, so a per-drain cost that became O(n) — total O(n²) — reads ≈ 25 and fails. That is exactly the "costs the same *per control*" property §16.6 meant, which the 1.25× wall-clock band never could measure.

**Correction 1 — §16.6 and §25.6 stated a count the code cannot assert.** Both said *"with 2 intents, probes are exactly one per drain call (500)"*. The test drives **one** intent and asserts the **difference** 480, because an absolute 500 is unattainable — the enqueue path probes too. Both are amended to the differential form, and ~~total probe cost is ≤ 500 × 5 ns~~ is **struck** in §16.6, §21 item 6 and §25.6: 2.5 µs against a measured 632 ns for the whole 500-control `handle` is not a bound, it is a tautology.

**Correction 2 — the constant 480 silently encodes "one update pass."** `Runtime::handle`'s focus re-run loop is bounded at four passes (§3.3 step 7), and a legitimate second pass makes the delta 960. That is a real behaviour change worth catching, so the **equality** is kept — and said so in the test comment and in §16.6, or the next reader will "fix" it to `% 480 == 0`.

**Rejected alternatives.** *Keep the 1.25× raw wall-clock band and make the stub's `update` O(1)* — rejected: an application that does **not** call `cx.intents` per component is not measuring the drain path at all; the O(n) loop is the workload and normalising it is the correct response. *Assert an absolute probe count* — rejected: `probes()` is cumulative since construction and counts the enqueue path, so no absolute number survives a change to focus staging. *Reset the counter per frame so absolute counts work* — rejected: it would make `probes()` a per-frame statistic and break the "since construction" contract §25.6 wrote, for no gain over the differential.

**Risks.** `ns₂₀ ≈ 42 ns` is a small median and the normalised ratio inherits its noise — mitigated by the 2× margin (0.60 against 1.25) and by the deterministic probe assertions being the binding ones. The 480 constant couples the test to the update-pass count, named above.

**Tests and acceptance.**

```bash
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 style_resolve_per_frame
PERF_STRICT=1 cargo test -p tui-next --test perf --release -- --test-threads=1 intents_drain
#   PERF-PROBES probes_500 - probes_20 == 480; PERF-RATIO intents_drain_ns_per_control <= 1.25
rg -n 'style_resolve_per_frame' COMPONENT_ARCHITECTURE.md   # names Slice 5 + frame_showcase_lists_120x40
```

### 27.5 Gate

```bash
rg -n 'Adjudication O' COMPONENT_ARCHITECTURE.md REFACTORING_STATE.md
cargo run -p xtask -- doc-check
cargo test --workspace --test architecture every_named_test_exists
```

Every command in §27.1–§27.4 exits 0; `crates/tui/tests/perf_baseline.txt` changes only in its `#` header (O1's recorded hit rate, O4a's baseline-free note) unless `ascii_glyphs()` moves an allocation count; `every_named_test_exists` reports no missing name once `theme::ascii_glyph_set_has_no_box_drawing`, `theme::builder::ascii_glyphs_is_idempotent_and_glyph_overrides_it` and `theme::cache_generation_wrap_does_not_serve_a_stale_entry` — all three named in §16.1 — exist in the sources; and the amendments to §11.2, §16.1, §16.6, §20.9‑1/‑2, §21 item 6, §24.2, §25.3, §25.6 and §25.8 are applied and mirrored in `REFACTORING_STATE.md`.

**Slice 4 wave 1 is not blocked by any of the four.** O2's whole-set swap must land before **4E** (`ScrollRegion` paints `scrollbar.begin`/`.end`); the full `GlyphSet` ASCII table is scheduled **with** 4E; O4a's ≤ 5 % share is reinstated in **Slice 5**.

---

## 28. Adjudication P — prototype decisions

**Source.** `docs/reviews/adjudication-p-prototype-decisions.md`, adjudicating the six decisions the Slice 2/3 prototype builder returned. **Accepted as written.** Two of the six premises handed to the adjudicator were **wrong as stated**, and both corrections are load-bearing: the corrected version is what is recorded here, not the original claim (P3 in full; half of P6's narrowing claim). Every earlier section this changes carries an inline `<!-- amended by §28 -->` marker: §3.3 step 9, §11.4 (two rows), §12.1, §13, §16.1 (four lists), §16.2 case 9 and its suite-level bullets, §16.3 (plus §16's "Where tests live" runner cell, the bless command, §15.1's acceptance block, Appendix A's Slice 3 and Slice 4 gates, §25.12 and §27.5's render lines), §17 example 11, §17.0 A7, §18.3 #4, §20.10 item 18, §26.1. `REFACTORING_STATE.md` (coordinator-owned) mirrors the six decisions and P1's Slice-5 obligation per the change-control rule at the top of this document.

**Facts the six rest on**, read from the tree at `HEAD 8ec40c1`: the 13 + 3 mono rules reach `GUTTER`, `MARKER` and `LABEL` for `DISABLED` and nothing else (`theme/downgrade.rs`, `MONO_RULES_PER_FAMILY = 16`); `TextInput::PARTS` is `[FIELD, TEXT, PLACEHOLDER, MARKER, GUTTER]` and never `LABEL`; an unset slot binds to `None` and `Buffer::set_stringn` patches only `Some` fields, so a `TEXT` style that says nothing inherits the `FIELD` fill's colour **and modifiers** per cell; `mono()` maps `Y < 0.35 → Black`, and Junie's `disabled_fg #4d4d4d`, `Fg(Faint) #262626` and `surfaces[0] #000000` all land there; `Fixture` carries no `status`; `mono_states_required_by` is an `if/else if` chain; `Dialog` registers only `Decorative` regions for its own id while `Registry::delivers_to` requires `Control` or `Part`; `Tabs` reaches `ACTIVE` only through forced `SELECTED`; `state_override_is_used_only_in_apps_and_fixtures` scans `crates/tui/examples/**`.

### 28.1 P1 — `.state_override` in `crates/tui/examples/**`

**Decision. Keep the split until Slice 5. Do not widen the exemption — not to `examples/**`, and not to a `showcase_*` path prefix.** Documentation only: no code change, no gate change, no allow-list entry. §18.3 #4 records the two halves and their expiry; `crates/tui/examples/showcase_buttons.rs` drops the `Track::Fixed(11)` row that reserves eleven dead rows for a matrix it does not draw, so the runnable page is honest about what it shows, and keeps its explanatory comment verbatim.

**Rationale.** The check protects two things and only one is about production code: (a) no component forces its own visual state where behaviour depends on it — an example cannot violate this; (b) the thirteen §17 examples are the mechanical proof that the *public* API suffices for an external consumer (`architecture::examples_are_external_consumers`, §16.5). Widening to `examples/**` weakens (b) for all thirteen. Widening to a `showcase_*` prefix is better shaped and precedented (§25 D‑10, "a named path shows the exception"), but buys the demonstration at the price of the matrix's **assertions**: an example is a binary no test may import (`#[path]`/`include!` are forbidden by the same check), so moving the matrix into the example deletes the only assertion in the tree that a forced rendering registers no ids, with nothing to replace it until Slice 5. One slice of an incomplete demo is a smaller loss than one slice of an unasserted reference rendering **plus** a permanently looser boundary rule. At Slice 5 `apps/showcase` has a `[lib]` target (§21 item 23) and its own `tests/`, so the problem is temporary by construction — and the correct handling of a temporary problem is a recorded deviation, not a hole in a gate.

**Rejected.** Widen to `crates/tui/examples/**` (applies the exemption to twelve files with no claim to it and makes "A11 is showcase/fixture-only" unenforceable for the rest of the refactor). Widen to `crates/tui/examples/showcase_*.rs` (costs the matrix's assertions for the same one slice and leaves an allow-list entry a later reader must remember to delete). Move the page to `crates/tui/tests/fixtures/` (a fixture is not runnable; goal §29's "the showcase demonstrates every public component" is not satisfied by a file nobody can start).

**Tests.** `cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures` exits 0 **with no allow-list**; `cargo test -p tui-next --test showcase_buttons` keeps asserting `b.area_of(MATRIX.index(0).index(0)).is_none()`; `cargo build -p tui-next --example showcase_buttons` still builds the page. **Slice-5 obligation** (recorded in `REFACTORING_STATE.md`, and the whole mitigation for the risk that a convention-only deviation becomes permanent): *the two halves become one file under `apps/showcase`, and §18.3 #4's deviation paragraph is struck.*

### 28.2 P2 — `render::components::*` file placement

**Decision. Confirm the split. `crates/tui/tests/render_components.rs` stays a separate target through Slice 4; §16.3 names the *test path*, not the file, as the contract.** The paths are produced by the nested `mod render { mod components { … } }`, so they are byte-identical either way and nothing downstream reads the file name. `every_named_test_exists` scans §16.1, §16.2's suite-level bullets, §16.4 and §16.6 only, so §16.3 is not machine-checked in either layout — folding the file would not add one assertion. The split buys a real thing during Slices 3–4: two work packages edit two files instead of contending on one, and a component-matrix re-bless does not touch the foundations target's baseline lines. Merge at Slice 5, when one owner holds both.

**The one genuine cost is closed explicitly**: a gate command that says `--test render` alone silently runs half the matrix. §16.3 now requires **every** gate command that runs render tests to name both targets, and every such command in this document has been rewritten (`rg -n -- '--test render\b' COMPONENT_ARCHITECTURE.md | rg -v render_components` is empty).

**Rejected.** *Require the fold now* — buys nothing testable, creates a cross-work-package file, and would be undone if the two owners diverge again in Slice 4.

**Tests.** `cargo test -p tui-next --test render --test render_components` runs the 384 component cells plus the foundations digests; `cargo test -p tui-next --test render_components render::components::` lists the matrix under the documented path; added to the Slice 3 gate: `cargo test --workspace -- --list | rg -c '^render::components::'` is non-zero.

### 28.3 P3 — §17 example 11 and the undelivered-intent guard

**Premise correction (it changes the decision).** The claim "example 11 leaks intents" is **not reproducible**: for a `Dialog`-owned modal the runtime records **no** `Diagnostic::UndeliveredIntent`. The dialog registers only `Decorative` regions under `CONFIRM`, and the diagnostic is gated on `Registry::delivers_to`, which requires `Control` or `Part`. The prototype's `FINDING` comment asserted the diagnostic, but the test exercised the *unconditional* shape, so the claim was never observed. What the gated shape actually does is **worse**: `Intent::Cancel` and `Intent::Layer(Dismissed)` are addressed to `CONFIRM`, nobody drains them, `finish()` clears the queue, and the dismissal is lost **silently** — `DialogAction::Dismissed` is never emitted and no diagnostic can see it. (Inferred from the cited code, not executed; the first acceptance command below verifies it before the prototype's comment is edited.)

**Decision — all three.** (1) §17 example 11 takes the **unconditional** shape, matching example 9, `tests/overlay.rs` and the passing journey fixture, and opens with `cx.open_layer(CONFIRM, remove_dialog().layer(cx))` rather than a bare `LayerSpec::modal(CONFIRM)` — §26 N1 requires the component to size its own layer, the bare spec defaults to `LayerSize::Fill`, and invariant D1 corrects it a frame later, which is a self-inflicted flash an example must not teach. `crates/tui/examples/11_small_app.rs` mirrors it. (2) The runtime keeps diagnosing a bucket whose layer closed during the same `handle`, and **starts diagnosing it for a decorative owner too**: the `delivers_to` guard is **widened, not narrowed**, to `delivers_to(owner) || intents.has_runtime_addressed(owner)`, with `IntentQueue::has_runtime_addressed(&self, owner: Id) -> bool` beside `undrained()`, true when the bucket holds any `Stored::{Layer, Cancel, FocusIn, FocusOut}`. (3) §13 gains the rule the two shapes differ by: a component that owns a layer runs its `update` unconditionally, every frame, and `cx.is_open(id)` guards only the **caller's** work.

**The invariant, restated verbatim — this is what `*::no_diagnostics_are_emitted_during_the_journey` now asserts.**

> **Every intent the runtime addresses to an owner is drained by that owner within the same `handle`, or it is reported as `Diagnostic::UndeliveredIntent { owner }`.** Runtime-addressed intents are `Layer`, `Cancel`, `FocusIn` and `FocusOut`; they are addressed to a known owner by the runtime itself, so a lost one is always a defect. Pointer intents keep §21 item 13's exemption — a `Decorative` region never receives one (`runtime.rs:331-333`), so a container that registers only decor still contributes zero diagnostics. A journey with zero diagnostics therefore proves that **every layer lifecycle event and every `Cancel` reached its owner**, which is strictly more than it proved before.

**Rationale.** The gated shape is not a style preference: the dismissal path (`handle` → `dismiss_top` → `run_update`) evaluates `cx.is_open` *after* the layer closed, so the guard is guaranteed to skip exactly the pass carrying the event the guard's author wanted to react to. Making the runtime silent about that would enshrine the silent-loss class the diagnostic exists to catch, and would do so precisely for `Dialog`, where it is already silent today. Widening costs one bucket scan per undrained owner per pass, on a path already O(undrained).

**Rejected.** *Stop diagnosing a bucket whose layer closed during the same `handle`* — hides the only mechanical signal that an app dropped a dismissal, and since the diagnostic does not fire for a decorative owner today, adopting it makes the invariant vacuous for every `Dialog`. *Make `Dialog` register a `Part` region for its container so the existing guard fires* — turns the dialog surface into a pointer target and re-opens the "is a click in the dialog chrome an outside click" question §21 item 13 settled; the fix belongs in the diagnostic, not the registry. *Deliver layer events to the last frame's drawer instead of the spec owner* — needs a per-layer "who drew this" map across frames and is wrong for a layer that was never drawn.

**Tests** (§16.1, so `every_named_test_exists` covers them): `runtime::a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it`, `runtime::a_decorative_owner_is_not_diagnosed_for_a_pointer_intent`, `dialog::an_unconditional_update_receives_the_dismissal`.

### 28.4 P4 — `Dialog::measured_height` and the prompt/acknowledgement control

**Decision. Confirm `input_rows`.** §26's formula was written before `prompt`/`acknowledge` existed: it is incomplete, not wrong, and the added term is the minimal correct completion. It keeps the "pure function of props and design tokens" property `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` asserts. The amended formula — `3 + wrapped_rows(description) + input_rows(d) + body_block(d) + actions_block`, with `input_rows` = `0` / `d.size.field_height` / `d.size.field_height + 1` for none / `.prompt(label)` / `.acknowledge(token)` — is written out in §26.1.

**Rationale.** Without `input_rows` a `prompt` dialog asks the resolver for a layer three rows shorter than its own content, and `draw` clamps the field to `field_h.min(actions_y - y)` — the prompt is silently squeezed or lost, the class of defect §26 N1 exists to remove. The implementation matches the amended formula term for term, and both halves wrap the description through the same `text::wrapped_rows`, so they cannot drift.

**Rejected.** *Fold the prompt into `body_rows` and let the caller state it.* The convenience constructors set `body_rows = 0` precisely because the caller supplies no body for a prompt; requiring the caller to know a prompt costs `field_height` rows re-exports a design token into every call site and breaks the moment a theme changes `field_height` — the defect §15.1 F4 removed from `Form`.

**Tests.** `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` extended with a `prompt` and an `acknowledge` case across both themes; new `dialog::a_prompt_dialog_sizes_its_own_field_row`. Both named in §16.1.

### 28.5 P5 — `Dialog::state_override` and `Overrides::inherit_forced`

**Decision. Confirm both, with one amendment that makes the property general instead of dialog-specific, and one that keeps the boundary check honest.**

**Confirmed as built.** `Overrides::inherit_forced` is `pub(crate)`, sets the forced state only when the container has one, and is deliberately not spelled `.state_override(`, so `xtask`'s regex still sees every *caller* use; `Button::inherit_forced` is `pub(crate)`; `Dialog::draw` passes `ov.forced_state()` into each action button and suppresses every `register_decor` under `forced`, so a forced dialog leaves no live, clickable control; `Dialog::state_override` is the public A11 surface, matched by the check's own-builder exemption.

**Amendment 1 — generalise the composition half.** `Field` has the same problem and does not solve it: `Field::state_override` forces the chrome, but the control it draws keeps its own state and **registers a live control**; the tests hide this by forcing both halves separately, which is a convention, not an invariant. `FieldControl` gains an identity-defaulted `fn inherit_forced(self, s: Option<StateFlags>) -> Self where Self: Sized`, implemented on `TextInput` by forwarding to `self.ov.inherit_forced(s)` and called from `Field::draw` before `self.control.draw(..)`. §12.1 records the property: *a container that can be forced into a reference state forces every component it owns, through the crate-internal `inherit_forced`; a reference rendering registers nothing, at any depth.*

**Amendment 2 — keep the escape hatch closed.** A new `xtask` boundary rule: `inherit_forced` may appear only under `crates/tui/src/components/**` and `crates/tui/src/field_control.rs`, and never in a `pub fn` signature outside the trait default. Without it a later slice can make the crate-internal path public and the A11 boundary check becomes decorative.

**Rationale.** A11's contract is *"a forced rendering is a picture, not a control."* Half a picture with a live text input in it is a `DuplicateId` and a focus stop waiting to happen the first time a showcase page renders the same control twice — the defect the button matrix's registration assertion was written to catch. The mechanism is already right; only its reach was short.

**Rejected.** *Make `state_override` public on the container and propagate through the public builder* — every propagation site would then match `\.state_override(` and the boundary check would have to allow-list library source, destroying it. *A `Ui`-level "forced" scope pushed around the reference rendering* — silently disables registration for everything drawn inside, including a control the page wants live; scope-shaped state that changes registration semantics is the `begin_modal` mistake (§1.2(5), §18.1) in a new place.

**Tests.** `dialog::a_forced_dialog_registers_no_control`, `field::a_forced_field_registers_no_control` (fails today). Both named in §16.1. Boundary: `! rg -n 'pub fn inherit_forced' crates/tui/src --glob '!field_control.rs'`. Risk: the trait change is additive, but `TextInput`'s impl must forward or the fix is inert, and `architecture::every_component_doc_has_the_standard_sections` wants the method documented.

### 28.6 P6 — a disabled text control is unreadable under `ColorLevel::Mono` — **priority**

This is the one item of the six that is a **goal §29 defect**, not a placement or shape decision, and it is three failures stacked in one place.

**Premise correction.** Two of the three narrowings named in the request are not what the tree does: `ListCase` **keeps** `DISABLED` (it narrows `EDITING`/`BUSY`/`ACTIVE`); only `TextInputCase` narrows `DISABLED`. `ListCase` passes the `disabled` row for the same reason `FieldCase` does — it paints `Part::LABEL` through `RowUi`, which the `(LABEL, DISABLED)` mono rule reaches. **The rule is exactly: a component survives case 9's `disabled` row iff it paints `LABEL`, `GUTTER` or `MARKER`. A text control paints none of them for its content.**

**(i) The goal-level defect.** Under `ColorLevel::Mono` the `DISABLED` colour rules resolve to `Black` on a `Black` canvas for `Theme::junie()`: `disabled_fg #4d4d4d` and `Fg(Faint) #262626` both have `Y < 0.35`, and `mono()` maps that band to `Black`; `surfaces[0]` is `#000000`. A disabled control at mono is therefore not merely indistinguishable from an enabled one — **it is invisible**. §11.4's `DISABLED` row prescribing `fg = Role::Fg(Faint)` is the instruction that produces it, so this is the one place where the *specification*, not the implementation, is wrong. `Theme::paper()`'s disabled step lands in the `Reset` band and escapes by luck. Goal §29 asks for **readable**, not only distinguishable, so both halves are in scope.

**(ii) The guard that let the narrowing through.** `mono_states_required_by` is an `if / else if` chain returning **one** state, not a union: a case declaring `EDITS | DISABLEABLE` is only ever required to keep `EDITING`. That is precisely why `TextInputCase` could drop `DISABLED` while declaring `Caps::DISABLEABLE` and keep MA‑8's guard green. It becomes a union — `FOCUSABLE → FOCUSED`, `ACTIVATES → PRESSED`, `DISABLEABLE → DISABLED`, `EDITS → EDITING`, `COLLECTION → SELECTED`, always plus the empty state — which is the assertion MA‑8 was written to make.

**(iii) The case-9 blind spot.** The driver forces theme flags only: `Fixture` carries `state_override`, `disabled`, `read_only`, `patch` and `secret`, and **no `status`**. Forcing `StateFlags::BUSY` therefore does not make `Button::busy()` true, so the spinner `Button` genuinely paints is never painted under the forced state. **A state whose affordance comes from props is unreachable by case 9 as written** — the case would report "distinguishable" or "indistinguishable" about a frame the component never had a chance to paint.

**Decision.**

**(a) Three mono `DISABLED` rules, and stop tinting at mono.** `mono_rules()` gains `(FIELD, DISABLED)` and `(TEXT, DISABLED)` — `set_fg(Role::Fg(FgStep::Primary)).remove(Modifier::all()).add(Modifier::DIM)` — and the existing `(LABEL, DISABLED)` and `(MARKER, DISABLED)` rules move to `Fg(Primary)` for the same reason. They are inserted **with the other `DISABLED` rules, before the `ERROR` rules**: state rules of equal specificity apply in declaration order, so `ERROR`'s `UNDERLINED` must land after `DISABLED`'s `remove(Modifier::all())` or it is erased. `MONO_RULES_PER_FAMILY` becomes **18**. `Part::PLACEHOLDER` needs no rule (it is painted over the `FIELD` fill and inherits its modifiers per cell); `Part::CONTAINER` needs none (a text control fills `FIELD`, not `CONTAINER`). §11.4's `DISABLED` and `BUSY`/`LOADING` rows are amended accordingly.

**(b) `BUSY`/`LOADING` are a component obligation, not a `StateRule`.** A rule binds one `GlyphRole`; a spinner is a frame sequence, so no rule can express it, and no mono rule mentions `BUSY` or `LOADING` today. The obligation: a component that can enter `BUSY`/`LOADING` paints `Part::ICON` with `design.motion.spinner_frames`; a component with no icon slot must not accept `.status(…)`. `Button` already discharges it. Three changes make it visible: `Fixture` gains `pub status: Status` (default `Status::Ready`) which the driver derives from the forced state (`BUSY → Busy`, `LOADING → Loading`, `ERROR → Error`) and each `Case` wires into its props; `TextInput::draw` paints `design.motion.spinner_frames[0]` into its existing trailing marker cell when `live` contains `BUSY | LOADING`, and declares `Part::ICON` in `PARTS`; §16.2 case 9 records that the driver makes the forced state real.

**(c) The narrowings to revert, once (a)–(c) land.** `TextInputCase` adds `DISABLED` (and `BUSY` once the spinner lands). `ButtonCase` adds `BUSY` — `Button` already paints the spinner; only the fixture was missing it. `FieldCase` is unchanged but drops the forced-`LABEL` dependence from its rationale: after (a) it passes on `FIELD`/`TEXT` too, so case 9 proves what §29 requires rather than what the chrome happens to paint. `ListCase` keeps `BUSY`/`LOADING`/`EDITING`/`ACTIVE` narrowed **with the comment corrected**: `BUSY` stays narrowed only until `List` paints a readiness affordance (4E/4F), which is a named obligation, not a permanent exemption. `TabsCase` keeps `ACTIVE` narrowed **and documents why**: a tab strip's `ACTIVE` is reached through forced `SELECTED`, forcing `ACTIVE` directly paints nothing, and making it paint would make `SELECTED` and `ACTIVE` produce identical output and fail case 9's pairwise distinctness. `DialogCase`, `ScrollRegionCase` and `PropsCase` are unchanged; the union guard in (ii) confirms that mechanically. An undocumented narrowing is indistinguishable from an oversight, so every remaining one carries its reason.

**Rationale.** §29 is a goal-level requirement and MA‑8 exists to stop a component narrowing its way out of it — yet the mono table had no rule reaching the parts a text control actually paints, so `TextInputCase` had no honest choice but to narrow. Fixing the case without fixing the table would be a lie; fixing the table without fixing the `if/else if` chain leaves the same escape open for the next component; and leaving `Fg(Faint)` at mono keeps a rule that actively produces an unreadable frame. Passing case 9 (which excludes colour) while the frame is invisible is a test proving the wrong thing.

**Rejected.** *`(Part::CONTAINER, DISABLED)` instead of `FIELD`/`TEXT`* — does not reach the defect (a text control fills `FIELD`), and moves every component's mono `disabled` baseline while leaving `TextInput` exactly as broken. *A strike-through or bracket glyph for `DISABLED`, like mono `PRESSED`* — `STRIKETHROUGH` is not universally rendered and reads as "deleted"; a bracket glyph steals two columns from a field's content, and a mono fallback must never change geometry. *Mono `StateRule`s for `BUSY`/`LOADING` binding a static "busy" `GlyphRole`* — requires a new `GlyphRole` (there is none), duplicates `design.motion.spinner_frames` as a second source of truth, and still paints nothing in a component that lays out no icon cell, i.e. it is inert exactly where the gap is. *Leave `Fg(Faint)` and accept black-on-black at mono* — satisfies case 9 while failing the sentence case 9 exists to enforce.

**Tests.** `theme::mono_disabled_is_dim_and_readable` (under `ColorLevel::Mono`, for `Family::{INPUT, FIELD, LIST, BUTTON}` × `Part::{FIELD, TEXT, LABEL}`: the resolved style contains `Modifier::DIM` and its `fg` differs from the resolved `bg` of `Surface::Canvas` — the assertion that would have caught the black-on-black); `theme::mono_appends_one_state_rule_per_family` (now 18); `conformance::mono_states_required_by_is_a_union`; `conformance::{text_input,button,field,list,tabs}::mono_states_are_distinguishable` with the widened state lists; `render::components::{text_input,field,list,button}::disabled` re-blessed under §20.10 item 18. The narrowing must stay visible in one place, each entry carrying a reason: `! rg -n 'fn mono_states' crates/tui/tests/conformance.rs -A2 | rg -v '///|//|const|&STATES|STATES:|\]|\}'`.

### 28.7 Risks

1. **Baseline movement (P6).** The **mono half only** of `crates/tui/tests/baselines/components.txt` moves for `text_input`, `field`, `list`, `button` and `tabs`; truecolor lines are untouched, because mono rules are appended only at `ColorLevel::Mono`. §16.3's order is binding — **change → capture → classify → bless** — with the §20.10 item 18 entry in `docs/visual-changes.md` written before the bless or `xtask bless-guard` fails.
2. **`inherit_forced` on `FieldControl` (P5)** is a public-trait change with a defaulted method: additive for the trait, inert unless `TextInput`'s impl forwards it.
3. **Widening the diagnostic (P3)** may surface `UndeliveredIntent` in code that "works" today by silently dropping a `FocusOut`. That is the point, but it can turn `*::no_diagnostics_are_emitted_during_the_journey` red in Slice 4 for reasons unrelated to the slice; run it **before** the slice opens.
4. **P1's split expires by convention, not by a gate.** If the Slice-5 obligation is not carried in `REFACTORING_STATE.md`, the two halves stay split forever. That ledger entry is the whole mitigation.
5. **P2's two-target split fails open** if any gate command names `--test render` without also naming `--test render_components`; the §16.3/§16 amendments are the mitigation and must land in the same commit.
6. **P3's premise correction is inferred, not executed.** Verify with the first acceptance command below **before** editing the prototype's `FINDING` comment.

### 28.8 Acceptance conditions (executable, from the workspace root)

```bash
# ── P3, first: confirm the premise correction before changing anything ──
cargo test -p tui-next --test showcase_buttons no_diagnostics_are_emitted_during_the_journey
cargo test -p tui-next --lib runtime::tests::a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it
cargo test -p tui-next --lib runtime::tests::a_decorative_owner_is_not_diagnosed_for_a_pointer_intent
cargo test -p tui-next --test overlay                      # the unconditional shape stays green

# ── P6 (priority) ──
cargo test -p tui-next --lib theme::downgrade::tests::mono_appends_one_state_rule_per_family
cargo test -p tui-next --lib theme::downgrade::tests::mono_disabled_is_dim_and_readable
cargo test -p tui-next --test conformance conformance::text_input::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::button::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::field::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::list::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::tabs::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::mono_states_required_by_is_a_union
! rg -n 'fn mono_states' crates/tui/tests/conformance.rs -A2 | rg -v '///|//|const|&STATES|STATES:|\]|\}'

# ── P6 baselines: change → capture → classify → bless, in that order ──
cargo test -p tui-next --test render --test render_components render::components::   # RED on the mono disabled cells
BLESS=1 cargo test -p tui-next --test render --test render_components
git diff --stat crates/tui/tests/baselines/components.txt              # mono lines only
rg -n 'mono DISABLED' docs/visual-changes.md                           # the classification exists

# ── P4 ──
cargo test -p tui-next --lib components::dialog::tests::layer_size_is_a_pure_function_of_props_and_design_tokens
cargo test -p tui-next --lib components::dialog::tests::a_prompt_dialog_sizes_its_own_field_row
! rg -n 'centered|resolve_anchor' crates/tui/src/components/           # §26.5 still holds

# ── P5 ──
cargo test -p tui-next --lib components::dialog::tests::a_forced_dialog_registers_no_control
cargo test -p tui-next --lib components::field::tests::a_forced_field_registers_no_control
cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures
! rg -n 'pub fn inherit_forced' crates/tui/src --glob '!field_control.rs'

# ── P1 ──
cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures   # unchanged allow-list
cargo test -p tui-next --test showcase_buttons                         # the matrix stays asserted
cargo build -p tui-next --example showcase_buttons                     # the page still runs
rg -n 'apps/showcase' REFACTORING_STATE.md                             # the Slice-5 obligation is recorded

# ── P2 ──
cargo test -p tui-next --test render --test render_components
cargo test --workspace -- --list | rg -c '^render::components::'       # non-zero
rg -n -- '--test render\b' COMPONENT_ARCHITECTURE.md | rg -v 'render_components' && exit 1   # no gate names one target

# ── the whole gate ──
cargo run -p xtask -- boundary
cargo run -p xtask -- doc-check
cargo test --workspace --test architecture every_named_test_exists
```

**Gate pass condition.** Every command exits 0; `crates/tui/tests/allow/legacy_api.txt` and `allow/domain.txt` stay empty; `docs/visual-changes.md` carries the mono-`DISABLED` entry before any baseline is blessed; §3.3 step 9, §11.4, §12.1, §13, §16.1, §16.2 case 9, §16.3, §17 example 11, §18.3 #4, §20.10 item 18 and §26.1 carry the amendments above; and `every_named_test_exists` reports no missing name once the eight names §28 introduces exist in the sources or are deferred in `xtask/named_tests_allow.txt` with the owning slice.
