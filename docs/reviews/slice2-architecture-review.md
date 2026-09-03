# Slice 2 architecture review — COMPONENT_ARCHITECTURE.md

**Reviewer:** fresh read-only `opus-analyst` (goal §27 Slice 2, §28 "Independent verifier"). No prior work assumed correct.
**Scope read:** `REFACTORING_GOAL.md` §5, §9–§23, §25, §27, §29; `COMPONENT_ARCHITECTURE.md` in full (2 768 lines); spot verification against `Cargo.toml`, `src/core/id.rs`, `src/bin/showcase/app_tests.rs`, `src/bin/jackin_preview/screens/mod.rs`.
**Method:** §16–§20 and Appendices A/B were treated as a *consumer* of §3–§15 and every type, method, module path, test name, precedence rule and file path used downstream was resolved against §3–§15 or §17.0. Every §17 example was read as Rust that must compile under `architecture::all_examples_compile`.

**Facts collected** are marked `[F]`. Everything else is inference or judgment.

---

## 0. Summary

The five load-bearing adjudications are sound and I would not reopen any of them: the two-phase `update`/`draw` split (A), the single `Response<A>` (C), kind-tagged `Id` + `ItemKey` (B), the runtime-owned layer stack (E), and concrete `Theme` data + typed recipes (D). The rejection of immediate-mode `show()` on harness-truthfulness grounds is correct and independently verifiable — `[F]` `src/bin/showcase/app_tests.rs:33-46` returns `Outcome` from `handle` and asserts on it, exactly as §3.2 claims.

What is *not* ready is the surface. §16–§20 and §17's examples were written against an API that §3–§15 does not actually declare. I found **16 blockers**, **31 majors** and **24 minors**. Three of the blockers (B2, B3, B5) are not typos — they are places where the accepted model, taken literally, cannot express what the document elsewhere requires, and they need an architectural amendment, not an edit.

**Verdict: not ready to implement. Ready after the ordered edits in §9 below.** None of the fixes reopen an adjudication; all are surface repairs plus three narrow amendments (data-to-phase parameter, `Ui::cache`, Esc ordering).

---

## 1. BLOCKERS

### B1 — `Response::consumed` and `Response::changed` are declared twice each
**§6.1.** The impl block declares `pub fn consumed() -> Self` and `pub fn consumed(&self) -> bool`; likewise `pub fn changed() -> Self` and `pub fn changed(&self) -> bool`. Rust has no receiver-based overloading: two items with the same name in one inherent impl is `E0592`. This is the most-used type in the architecture and it does not compile.

**Fix (exact):** keep the constructors, rename the readers.
```rust
pub fn is_consumed(&self) -> bool;
pub fn is_changed(&self) -> bool;      // invalidate >= Paint
```
Propagate to §16.4 mapping item 1: `assert!(h.key(…).changed())` → `assert!(h.key(…).is_changed())`, and to §16.1 `response.rs` name `no_repaint_lowers_to_none` (unaffected).

### B2 — Collection builders cannot change their type parameters
**§17.0 A7 (`List`, `Tabs`), §12.2, §20.3.**
```rust
impl<'a, T, K, R> List<'a, T, K, R> where K: Fn(&T)->ItemKey, R: Fn(&T,&mut RowUi) {
    pub fn new(id: Id, items: &'a [T]) -> List<'a, T, (), ()>;   // () is not Fn -> unsatisfied bound
    pub fn key(self, k: K) -> List<'a, T, K, R>;                 // K is fixed by the impl header
```
`key(self, k: K) -> List<'a,T,K,R>` takes and returns the *same* `K`, so `List::new(...).key(|o| …)` can never typecheck. `Tabs::key(self, k: K) -> Self` is worse. This is the central API of §12, §17 examples 7/8/10/11 and Scenarios D/E.

**Fix (exact):** three impl blocks, method-level generics, and real default types (no `()`).
```rust
pub struct ByIndex;                     impl<T> Fn-equivalent via trait KeyFn
pub struct DefaultRow;                  // renders Display/label via a Row trait
impl<'a, T> List<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id, items: &'a [T]) -> Self; }
impl<'a, T, K, R> List<'a, T, K, R> {
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> List<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> List<'a, T, K, R2>;
    // …all non-generic config methods stay here, returning Self
}
impl<'a, T, K, R> List<'a, T, K, R>
where K: Fn(&T) -> ItemKey, R: Fn(&T, &mut RowUi<'_>) {
    pub fn update(&self, …); pub fn draw(&self, …);
}
```
Apply identically to `Tabs`, `Picker`, `Tree`, `Grid`, `NavList`, `Steps`, `ChipBar`, `RadioGroup`, `Completion`, `FilterList`, `PropsList`. Also give `KeyFn`/`RowFn` real blanket impls or delete `pub trait KeyFn<T>: Fn(&T) -> ItemKey {}` (§12.2), which is currently declared and never used anywhere.

### B3 — The props-borrow / mutating-closure idiom does not compile
**§17 examples 8 and 11; §12.1 "borrowed content"; Scenario D/E.** Example 8:
```rust
Tabs::new(STRIP, &self.docs)            // temporary holds &self.docs for the whole statement
    .update(cx, &mut self.strip)
    .on_action(|a| match a {
        TabsAction::Close(k) => self.docs.retain(…),   // &mut self.docs — E0502
```
The `Tabs<'_>` temporary lives to the end of the statement, so the immutable borrow of `self.docs` is still live when the closure takes it mutably. Edition-2021 disjoint capture does not help: it is the *same* field. Example 7 and 10 survive only by accident (their closures touch disjoint fields).

This is not an example bug. It is the predictable consequence of putting the borrowed collection in the props struct that also owns the phase methods, and it will bite every screen that removes a row, closes a tab, or reorders.

**Fix (recommended amendment — mirrors §17.0 A3's controlled-value rule):** move the *data* out of the props and into the phase call, exactly as `value: &mut String` already is.
```rust
pub fn update(&self, cx: &mut Cx<'_>, st: &mut ListState, items: &[T]) -> Response<ListAction>;
pub fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &ListState, items: &[T]) -> Rect;
// List<'a, T, K, R> then holds only: id, key fn, row fn, select_mode, empty, patches, slots
```
The props no longer borrow `self.docs`, so the closure is free. Add to §13's table a row: *"collection data — passed per phase, never held in props"*. Rewrite examples 7, 8, 10, 11 accordingly.
**Fallback if the amendment is rejected:** make `into_action()` + `match` the mandatory documented shape and rewrite the examples; but then §12.1's "borrowed content" row must warn that the props value must be dropped before mutating the source.

### B4 — `Cx::intents(id)` locks `Cx` for the whole loop
**§17.0 A2, §17 example 12, §3.3 step 7.** `pub fn intents(&mut self, id: Id) -> IntentIter<'_>` borrows `Cx` mutably for the iterator's life. Any real component must touch `cx` inside the loop: `Dialog` calls `cx.close_layer`, `SplitPane` calls `cx.capture`, `List` reads `cx.area`, `Select` calls `cx.open_layer`. All are `E0499`. Example 12 compiles only because a segmented control happens to need nothing.

`Intent<'f>` carrying `Paste(&'f str)` is the right idea and is exactly the escape route, but it is never wired: `'f` is not related to anything in `Cx<'_>`.

**Fix (exact):** split `Cx`'s intent queue from its mutable services and give both the frame lifetime.
```rust
pub struct Cx<'f> { /* intents: &'f IntentQueue (shared, frozen during step 7), rest: &'f mut … */ }
impl<'f> Cx<'f> {
    /// Borrows only the frozen queue. Marks this owner's bucket drained (interior Cell flag).
    pub fn intents(&self, id: Id) -> IntentIter<'f>;
}
```
State in §3.3 that the queue is built in step 6 and is immutable for the whole of step 7, and that "drained" is recorded through a `Cell<bool>` per bucket. Then `for it in cx.intents(self.id) { … cx.close_layer(…) … }` compiles. Also unify the name: §20.9-12 calls it `Intents::take(id)`, §3.3 step 6 calls the type `Intents`, §17.0 A2 calls the method `Cx::intents`. Pick `Cx::intents` and rewrite §3.3 and §20.9-12.

### B5 — Per-frame derived caches have no home, yet three amendments require them
**§5 R2, §4 S2/S6, §20.9-7, §20.9-8, §20.9-9, §12.4.**
`draw(&self, ui, area, st: &XState)` forbids mutation, and S2 forbids `XState` from holding a `Rect`. But:
* §20.9-7 requires `TextViewport` layout to be "windowed" — the window is a function of `area`, known only in `draw`.
* §20.9-8 requires an incremental `Tree` flat index — rebuilt on expand (an `update` op) but *consumed* per draw, and filtered by a query that changes in `update`.
* §20.9-9 requires a `CodeEditor` highlight cache keyed by an edit counter — the counter can live in `XState`, the cache cannot (it holds spans and is not `PartialEq`-meaningful).
* `Grid` column-width sampling is width-dependent, i.e. draw-time.

`FrameOut::layout` / `ui.report_layout` (§4 S6) flows facts *up* to the runtime; it does not give the component a scratch pad. As written, these three amendments are unimplementable.

**Fix (recommended amendment):** add a runtime-owned, explicitly non-semantic per-id scratch to §17.0 A2 and §5.
```rust
impl Ui<'_> {
    /// Derived, non-semantic per-component cache. Keyed by (Id, TypeId). Cleared on
    /// resize, theme change and generation gap. Never observable in `Response`,
    /// never compared by `draw_twice_leaves_state_equal`.
    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T;
}
```
Add **R8** to §5: *"`Ui::cache` is the only mutable state reachable from `draw`. Its contents must be a pure function of (props, state, area, theme); a component that reads a value from `cache` that is not derivable from those inputs is a bug."* Add `architecture::cache_types_are_derived_only` (a `syn` scan that no `cache::<T>()` type appears in a `Response` or an `XState`) and `conformance::draw_twice_is_byte_identical` already covers the behavioural half. Then restate §20.9-7/-8/-9 in terms of `Ui::cache`.
Also close the alternative explicitly (interior mutability in `XState`) and say why it is rejected: it breaks S2's `PartialEq` and case 6.

### B6 — Esc reaches the layer before the focused component, so edit-cancel inside a dialog is unreachable
**§3.3 step 5, §8.6, §9.1, §15.** Step 5 executes "Esc-to-dismiss-top-layer" *before* intents are enqueued (step 6) and before `app.update` (step 7). A `TextInput` editing inside a modal `Form` therefore never receives Esc: the dialog closes instead. §15 requires "explicit begin, commit, **cancel**"; §16.1 names `input::cancel_restores_the_snapshot` and `select::escape_closes_and_restores_the_cursor` — both unreachable inside a layer.

**Fix (exact):** move layer dismissal to the bubble phase.
* §3.3 step 5 keeps only Tab/Shift+Tab and press-focuses-owner. Delete "Esc-to-dismiss-top-layer".
* §3.3 step 8 becomes: `8. bubble  keys still unconsumed after step 7 are offered to (a) the app KeyMap "Bubble" bindings, then (b) Dismiss.esc on the top layer, then (c) the screen's Esc ladder.`
* §8.6 and the §9.1 table row "Esc" are updated to the same order.
* Add unit test `layer::esc_reaches_the_focused_editor_before_the_layer`.

`Intent::Cancel` ("Esc reached this owner after layer dismissal") then means what it says.

### B7 — The 4-pass focus loop has undefined re-entry semantics
**§3.3 step 7.** "If the pass changed focus … re-run 6-7. Bounded at 4 passes; the 5th is a debug_assert and a dropped transition."
Unspecified, and each answer produces a different system:
1. Does the re-run re-enqueue the *original* key/pointer intents? If yes, `Button::on_activated(|| self.people.push(…))` (example 11) fires twice on one Enter. If no, the semantics are correct but nothing says so.
2. How are the `Response`s of passes 2..n combined with pass 1's?
3. When the 5th pass is dropped, which half of the `FocusOut`/`FocusIn` pair is dropped? Dropping `FocusIn` leaves `BlurPolicy::CommitAndValidate` committed with no new focus; dropping `FocusOut` leaves an editor in `EDITING` forever.

**Fix (exact), append to step 7:**
> A re-run enqueues **only** `Intent::FocusOut{to}` and `Intent::FocusIn{via}`; already-drained buckets are never refilled, so no input intent is delivered twice. The `Response` of each pass is folded into the first with `|`. If a 5th pass is required, the runtime emits `Diagnostic::FocusTransitionDidNotSettle`, applies the pending `FocusOut` **and** the matching `FocusIn` to the last requested target without re-running `app.update`, and continues. `conformance::focus_transition_settles` asserts the diagnostic count is 0.

Also demote `conformance::focus_transition_settles` from per-component to a suite-level test (it is a whole-app property; a single component cannot exercise a 4-pass loop).

### B8 — `#[non_exhaustive]` contradicts every struct literal in §17
**Appendix B.3 item 4 vs §17 examples 2, 10; §12.2/§12.3 default bodies.**
B.3 marks `ColorTokens`, `DesignTokens`, `LayerSpec`, `RowDecor`, `CellDecor`, `LayoutFacts` `#[non_exhaustive]`. `#[non_exhaustive]` forbids struct-literal construction *and* functional-update (`..base`) syntax from other crates. But:
* Example 2 builds `ColorTokens { … }` with a full literal from `crates/tui/examples/`, a separate crate.
* Example 10 builds `LayerSpec { anchor: …, ..LayerSpec::modal(OWNER_PICK) }`.
* §11.4 requires `ColorTokens` to be *exhaustively destructured* so adding a token is a compile error — the opposite intent.
* TablePro's `GridModel` adapters will want `RowDecor { marker: …, ..Default::default() }`.

**Fix (exact):**
* Remove `ColorTokens`, `DesignTokens`, `RowDecor`, `CellDecor` from B.3 item 4. Record: *"adding a colour or design token is an intentional breaking change for downstream themes; `map_colors`'s exhaustive destructure is the mechanism (§11.4)."*
* Keep `LayerSpec` and `LayoutFacts` `#[non_exhaustive]` and give `LayerSpec` consuming builders in §17.0 A6:
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
```
Rewrite example 10 to `LayerSpec::popover(OWNER_PICK, Anchor::Rect{…}).dismiss(Dismiss::ESC_AND_OUTSIDE)` — which also fixes the visual bug of a full-screen dim behind a dropdown-shaped picker.

### B9 — Binary-only app packages cannot host `apps/*/tests/*.rs`
**§16.4, Appendix B.3 item 7, B.2.**
B.3 item 7: *"Each app is a binary-only package (`publish = false`, no `[lib]`); its tests live in `tests/` and reach the app through a small `pub` surface declared in `main.rs`."* Cargo integration tests (`tests/*.rs`) are separate crates that link against the package's **library** target. A package with no `[lib]` has nothing to link; `use showcase::app::App;` cannot resolve. The only workarounds are `#[path]`/`include!`, which §16.5 `architecture::applications_depend_only_on_the_library_facade` explicitly forbids.

`[F]` The existing tests need exactly this reach: `src/bin/showcase/app_tests.rs:124-129` reads `self.app.focus.current()` and `self.app.hits.area_of(f)`; `:143` reads `h.app.quit`; `:156` reads `crate::app::NAV_ENTRIES`; `:20-22` calls `App::new` + `app.goto(page)`.

**Fix (exact):** each app package gets both targets.
```toml
# apps/showcase/Cargo.toml
[lib]
name = "showcase_app"
path = "src/lib.rs"          # the whole app: App, PageId, NAV_ENTRIES, screen const Ids

[[bin]]
name = "showcase"            # binary name preserved (goal §21)
path = "src/main.rs"         # fn main() { showcase_app::run() }
```
Update B.2's tree, B.3 item 7, and §16.4 item 3. `architecture::binary_names_are_preserved` is unaffected. Add to §16.5: `architecture::app_libs_are_not_published_and_are_not_depended_on_by_the_library`.

### B10 — `Field<C>` has no control trait, and `Field` shares its control's `Id`
**§15, §17.0 A7, §17 examples 6 and 11.**
Two defects in one:
1. `Field::draw(ui, area, &self.email_st)` requires `Field<'a, C>` to know `C`'s state type. No `C: Control` bound and no associated type exist anywhere in the document. `Field` is generic over an unconstrained `C` and cannot forward anything.
2. Example 6 constructs `Field::new(EMAIL, "Email", TextInput::new(EMAIL)…)` — the wrapper and the control register the **same `Id`** in one frame. §7.1 makes that `Diagnostic::DuplicateId` and §16.2 case 11 `id_separator_collision_free` fails for every field in the catalogue.

Also §17.0 A7's comment "update/draw forward to the control and own the chrome" is false: both examples call `TextInput::update` directly in `update` (it needs the extra `value: &mut String`) and only `Field::draw` in `draw`.

**Fix (exact):**
```rust
/// Draw-time chrome only. `Field` never registers a focus stop and never runs `update`;
/// the control keeps its own `Id` and its own `update`.
pub trait FieldControl {
    type State;
    fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &Self::State) -> Rect;
    fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
pub struct Field<'a, C: FieldControl> { /* label, required, help, error, plain, control: C */ }
impl<'a, C: FieldControl> Field<'a, C> {
    pub fn new(label: &'a str, control: C) -> Self;      // no Id — the control owns identity
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &C::State) -> Rect;
}
```
Rewrite examples 6 and 11 (`Field::new("Email", TextInput::new(EMAIL).value(&self.email))`), and correct §15's `Field` struct (which currently carries `id: Id`) and §17.0 A7.

### B11 — `Registry::hit` layer filtering contradicts the click-outside rule
**§3.3 step 3, §9.1 table, §8.6.**
The §9.1 table says barriers work because "regions carry `layer`; `hit()` filters `layer == top`". §8.6 defines click-outside as `hit.layer < top_layer || hit.is_none()`. If `hit()` only ever returns top-layer regions, `hit.layer < top_layer` is unreachable and the "real outside test" §8.6 boasts of degrades to exactly the "hit returned None" test it replaces.

**Fix (exact):** §3.3 step 3 and §9.1 —
> `Registry::hit(pos)` returns the topmost region covering `pos` **regardless of layer**. The runtime then: delivers the intent iff `hit.layer == top_layer`; treats `hit.layer < top_layer` (or `None`) as outside-click for the top layer's `Dismiss.outside_click`.

Update §16.1 `hit.rs` test `higher_layer_shadows_lower` and add `hit::hit_returns_a_lower_layer_region_for_the_outside_click_test`.

### B12 — `UndeliveredIntent` must be non-zero in normal operation, but §16.4 asserts zero
**§3.3 step 9, §16.4 `*::no_diagnostics_are_emitted_during_the_journey`, §18.1 (`kind` in the region tuple), §12.1.**
Containers register regions: `Panel`, `Dialog` (`Part::CONTAINER`/`BORDER`/`BACKDROP`), `SplitPane`, `Form`, `Grid`'s header. A click in a `Panel`'s padding hits the panel's region; `Panel` is stateless and has no `update`, so nothing drains that bucket. §3.3 step 9 records `Diagnostic::UndeliveredIntent`. §16.4's journey test asserts **zero** `UndeliveredIntent`. Both cannot hold.

Additionally, `RegionKind` is referenced twice (`{owner, part, area, layer, kind, gen}` in §3.3 step 11; "region lists (owner, part, area, layer, kind)" in §16.2 case 5) and **never defined**.

**Fix (exact):** define the kind and change the diagnostic's meaning.
```rust
pub enum RegionKind {
    Control,     // focusable, delivers intents
    Part,        // sub-region of a Control; delivers to the Control's owner
    Scroll,      // wheel target only
    Decorative,  // paints and answers area_of; never delivers, never diagnosed
}
```
§3.3 step 9: *"An intent whose resolved owner registered only `Decorative` regions is discarded silently. `UndeliveredIntent` is recorded only when the owner registered a `Control` or `Part` region and drained nothing."* §16.4's zero-diagnostics test stands with that definition.

### B13 — `mono_states_are_distinguishable` fails by construction for `PRESSED`
**§11.4 mono table, §16.2 case 9, §20.10-1.**
The mono rule for `PRESSED` is `Part::CONTAINER bg = Role::Fg(Primary), fg = Role::Surface(Canvas)` — a **colour-only** change. Case 9 compares "the `(symbol, modifier)` multiset … colour excluded". Under mono, `default` and `pressed` therefore produce identical symbol/modifier multisets for every button, chip, menu item and tab. The test is red for every `ACTIVATES` component on day one.

**Fix (exact), choose one and write it:**
* (a) Add a modifier to the mono `PRESSED` rule: `Part::CONTAINER … add(Modifier::BOLD)` **and** `Part::LABEL` bracket glyphs (`GlyphRole` additions `PressLeft`/`PressRight`, rendered `[Save]`). Preferred — it is legible in a real monochrome terminal, which is the point of §20.10-1.
* (b) Redefine case 9 to compare `(symbol, modifier, resolved-role-identity)` rather than raw colour. Weaker: it passes a test that a monochrome user cannot see.

Also: the table's `SELECTED` and `CHECKED` share one row with two glyphs; state which applies when both flags are live (`CHECKED` wins).

### B14 — `intents_drain_scales_with_intents_not_components` is false by construction
**§16.6 additions, §20.9-12, §3.3 "per-component cost is one `Intents::take(id)` hash probe".**
The threshold is *"a frame with 500 registered components and 2 intents costs the same as one with 20 components and 2 intents, ±10 %"*. Every component calls `cx.intents(self.id)` unconditionally every frame (example 12 does). 500 components make 500 probes; 20 make 20. The cost is O(components) by design, and the assertion cannot pass.

**Fix (exact):** add the O(1) fast path *and* restate the threshold.
* §20.9-12: *"`Cx::intents` returns an empty iterator without probing when the queue is empty (a single `bool` check) and, when non-empty, probes a `[u64; N]` open-addressed table of the ≤ 8 owners that actually have intents. A frame with no input performs zero probes."*
* §16.6: rename to `intents_drain_is_o_1_when_the_queue_is_empty`, threshold *"a 500-component frame with 0 intents costs the same as a 20-component frame with 0 intents ±10 %; with 2 intents, total probe cost is ≤ 500 × 5 ns and allocations are 0."*

### B15 — `GridEditor: &mut self` is unreachable through props holding `&'a M`
**§12.3, §5 R1, Scenario H.**
"`GridModel` is `&self` (used by both phases); `GridEditor` is `&mut self` and is reachable **only from `update`**." But `Grid<'a, M>` holds the model, and both `update(&self, …)` and `draw(&self, …)` take `&self` on the props. There is no path from `&self` → `&mut M`. The Slice-6 commit path (`commit_cell`, `apply_cycle`) is therefore unimplementable.

**Fix (exact) — same shape as A3's controlled value, and the same shape as B3's amendment:**
```rust
pub fn update(&self, cx: &mut Cx<'_>, st: &mut GridState, model: &mut M) -> Response<GridAction>;
pub fn draw  (&self, ui: &mut Ui<'_>, area: Rect, st: &GridState, model: &M) -> Rect;
// Grid<'a> holds: id, columns, nav unit, select mode, empty state, action slot, patches
```
This also removes `M` from the props type entirely, simplifying §20.3.

### B16 — WP‑0's file paths presuppose the workspace it is required to precede, and Slice 3 contradicts goal §27
**Appendix A WP‑0 and Slice 3.**
WP‑0 owns `crates/tui-testing/src/perf.rs`, `tests/perf.rs`, `tests/baselines/`, and is "written against the **current** single-package tree". `[F]` The current tree has no `crates/` (root `Cargo.toml` is a single package with three `[[bin]]`s and `default-run = "showcase"`). The workspace skeleton is created in Slice 3, which depends on WP‑0. Circular.

Separately, Slice 3 states *"Applications do not compile during this slice."* Goal §27 Slice 3 requires *"Keep the repository compiling and tested throughout."* These are incompatible, and the incompatibility is the substance of the coordinator's staging question (§7 below).

**Fix (exact):** see §7 — WP‑0 lands the harness at root `tests/perf.rs` + `tests/perf_support.rs` in the existing single package; Slice 3 *moves* it into `crates/tui-testing/`; and the "applications do not compile" sentence is struck and replaced by the two-package staging plan.

---

## 2. Internal consistency — §16–§20/Appendices against §3–§15

Everything below is a name, signature or path used downstream that §3–§15 or §17.0 does not declare, or declares differently.

### M1 — Undeclared `Ui` / `Cx` members used in §17 and §3.3
| Used at | Symbol | Status |
|---|---|---|
| ex 1, ex 11 | `ui.full()` | never declared |
| ex 12 | `ui.register_part(id, PartRef, Rect)` | never declared (`register_control`, `register_scroll` are) |
| §3.3 step 7 | `cx.focus(id)` | never declared |
| §8.5 | `cx.request_repaint()`, `cx.request_repaint_after(Duration)` | prose only, absent from §17.0 A2 |
| §3.4 | `cx.go(Go::Manager)` | library `Cx` has no request bus (→ M12) |
| §3.4 | `HintCtx<'_>` | never declared |

**Fix:** add to §17.0 A2 —
```rust
impl Ui<'_> {
    pub fn full(&self) -> Rect;                                       // current clip rect
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect);
}
impl Cx<'_> {
    pub fn focus(&mut self, id: Id);                                  // stages a transition
    pub fn request_repaint(&mut self);
    pub fn request_repaint_after(&mut self, d: std::time::Duration);
}
```
and either declare `HintCtx` in §13.1 or change §3.4's `Screen::hints` to `fn hints(&self, w: &World) -> HintLayer`.

### M2 — `Cx::capture(part)` cannot know its owner
**§8.2.** `pub fn capture(&mut self, part: PartRef) -> bool` has no owner. `Cx` carries no "current component" notion — `Cx::intents(id)` and `Cx::area(id)` both take the id explicitly, establishing the convention.
**Fix:** `pub fn capture(&mut self, owner: Id, part: PartRef) -> bool;` and `pub fn capture_owner(&self) -> Option<Id>;`.

### M3 — `Ui::layer` arity differs between §3.3 and §9.1
§3.3 step 12: `ui.layer(id, spec, |ui, area| …)`. §9.1 and examples 9/10/11: `ui.layer(id, f)`.
**Fix:** delete `spec` from §3.3 step 12. The spec is supplied by `cx.open_layer` in `update`; `Ui::layer` looks up an already-open layer and returns `None` when closed — which is why it returns `Option<R>`.

### M4 — `LayerId` assignment order is unspecified (compositing race)
**§3.3 step 12, §9.1.** "z-order is the layer order, NOT the call order" — but nothing says whether `LayerId` is assigned by `cx.open_layer` (update, stack order) or by `ui.layer` (draw, call order). A builder can implement either; they differ whenever a screen draws a dialog after a picker it opened first.
**Fix (exact), append to §9.1:** *"`LayerId` is assigned monotonically by `Cx::open_layer` and is the stack position. `Ui::layer(id, f)` resolves `id` to its already-assigned `LayerId`, executes `f` into that layer's pooled buffer, and returns `None` without executing `f` if `id` is not open. Call order at draw time has no effect on z-order, hit filtering, or focus scope nesting."* Add `render::overlay::layer_composites_bottom_to_top_regardless_of_call_order` (already named in §16.3 — good) plus `layer::layer_id_is_assigned_at_open_not_at_draw`.

### M5 — `Id::part` versus `PartRef` are conflated
**§7.1, §16.4 item 3, ex 10.** §7.1 states children are "**never** addressed by a derived id in application code: they are addressed by `PartRef`". Yet §16.4 item 3 says `FORM.sub("save")` becomes `FORM.part(Part::custom("save"))` **or** `area_of_part(FORM, PartRef::of(…))`, presented as equivalent, and ex 10 uses `DLG.part(Part::custom("owner"))` as a Button's own `Id`. These are two different lookups against two different registries.
**Fix:** state the rule —
> `Id::part(p)` mints a **child component id** (a Button inside a Dialog); that component registers its own `Control` region and is found by `Runtime::area_of`. `PartRef` tags a **sub-region of a single component** (a tab's close glyph, a scrollbar thumb) and is found by `Runtime::area_of_part`. They are never interchangeable.

Also declare `impl PartRef { pub const fn of(p: Part) -> Self; pub const fn item(p: Part, k: ItemKey) -> Self; }` — used in §16.2, §16.4, §17.0 A7 and never declared.

### M6 — Types used but never declared
| Symbol | Used at | Note |
|---|---|---|
| `Status` | §13 `.status(Status)` | the loading/busy/error API for **every** component |
| `KeySet` | §18.2, §20.9-13, B.4 | needs `Only(SmallVec)` / `AllExcept(SmallVec)` + API |
| `ColorLevel` | §11.4, §16.2, §16.3, §17.0 | variants only implied |
| `UnicodeLevel` | §11.2 `Capability` | never used by anything either |
| `BindingState` | §13.1, §16.2, B.4 | determines whether binding tables can be `const` |
| `Diagnostic` | §7.1, §8.4, §3.3, §16.4 | four variants named, no enum |
| `RegionKind` | §3.3 step 11, §16.2 case 5 | → B12 |
| `Backdrop` | §9.1 | comment only |
| `Density` | §11.2 | comment only |
| `SortDir` | §6.1 `GridAction::Sort` | — |
| `Hit` | §3.3 step 3, §18.1 | fields named in prose only |
| `SyntaxTokens::derive`, `MeterTokens::derive` | ex 2 | — |
| `Chord::key(KeyCode)`, `Key::is(KeyCode)` | ex 12 | — |
| `AppActionRecord` | §16.4 `Harness::actions()` | no mechanism exists for app actions to reach the harness |
| `LayoutError` | §13, B.4 | declared nowhere, used nowhere → delete |

**Fix:** declare each in §17.0 (new subsections A8 "Status and capability", A9 "Diagnostics"), or delete (`UnicodeLevel`, `LayoutError`, `AppActionRecord` + `Harness::actions`).

### M7 — Two `Align`s and two `Size`s
* `Align` is a **screen** alignment in `Anchor::Screen(Align)` (§9.1: `Center, UpperThird, Bottom`) and a **text** alignment in `StylePatch.align: Slot<Align>` and `CellUi::align(Align::Right)` (ex 7).
* `Size { min: (u16,u16), preferred: (u16,u16) }` (§10) is used as `LayerSpec.min_size: Size` (§9.1) — a min-size field whose type is itself a min/preferred pair.

**Fix:** rename to `ScreenAlign { Center, UpperThird, Bottom, … }` and keep `Align { Left, Center, Right }`; change `LayerSpec.min_size` to `(u16, u16)`.

### M8 — `ThemeBuilder` is missing methods the examples call
**§17.0 A5 vs ex 3.** Example 3 calls `.focus(rgb(…))`; `ThemeBuilder` has no `focus`. Also missing: `selection`, `highlight`, `field`, `disabled`.
**Fix:** add `pub fn focus(self, c: Color) -> Self;` plus `selection(bg, fg)`, `highlight(bg, fg)`, `field(base, hover)`.

### M9 — `RowUi` / `CellUi` have no signatures
**§12.2 vs ex 7.** `CellUi` is declared as `pub struct CellUi<'u> { /* text, tone, align, italic, suffix glyph, patch */ }` — a comment. Example 7 chains `c.money(o.total_cents).align(Align::Right).tone(Role::…)`, requiring `&mut Self` returns and a `money` method that does not exist. `RowUi::part(p, area_hint: u16)` never explains what `area_hint` means (does it consume width from the right? from the cursor?).
**Fix:** give §12.2 the exact `impl CellUi`:
```rust
impl CellUi<'_> {
    pub fn text(&mut self, s: &str) -> &mut Self;
    pub fn num(&mut self, n: i64) -> &mut Self;        // formats in place, 0 allocations
    pub fn money(&mut self, cents: i64) -> &mut Self;  // ditto
    pub fn align(&mut self, a: Align) -> &mut Self;
    pub fn tone(&mut self, r: Role) -> &mut Self;
    pub fn italic(&mut self, yes: bool) -> &mut Self;
    pub fn suffix(&mut self, g: GlyphRole) -> &mut Self;
    pub fn patch(&mut self, p: &StylePatch) -> &mut Self;
}
```
and define `RowUi::part(p, width)` as *"reserves `width` columns from the **right** of the remaining row space; `label` fills what is left"*.

### M10 — `XState::reconcile` signature defeats its own R1 amendment
**§12.2 vs §20.9-3.** `fn reconcile(&mut self, keys: impl Iterator<Item = ItemKey>) -> Reconciliation` forces an O(n) walk every frame. §20.9-3 requires an O(1) stamp `(len, key(first), key(last))` and a cached-index probe — neither is expressible over an iterator without consuming it. `XState` is also a metavariable, not a type, and `XState::invalidate()` (named in §20.9-3) is never declared.
**Fix (exact):**
```rust
/// Implemented on every collection state type (ListState, TreeState, TabsState, GridState, …).
pub trait Reconcile {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation;
    fn invalidate(&mut self);     // caller mutated items in place without changing len/ends
}
```

### M11 — `Binding<A>` cannot be a `const` table for actions that carry keys
**§13.1, §16.2 case 20, ex 12.** `Bindings::bindings(&self, st) -> &'static [Binding<Self::Action>]` requires the emitted `XAction` to be const-constructible. `ListAction::Chose(ItemKey)`, `TabsAction::Close(ItemKey)`, `GridAction::Sort(ColumnKey, SortDir)` are runtime values. Example 12 exposes the problem exactly: it maps `Enter` to `SegmentedAction::Moved` because `Selected(ItemKey)` cannot appear in a `const`.
**Fix:** separate the declarative command from the emitted action.
```rust
pub trait Bindings {
    type Cmd: Copy + 'static;                       // const-constructible: Next, Prev, Activate, Close…
    fn bindings(&self, st: BindingState) -> &'static [Binding<Self::Cmd>];
}
```
`update` maps `Cmd` → `XAction` with the live key. Fix example 12 (`Binding { chord: Enter, cmd: SegCmd::Select, label: "Select", … }`).

### M12 — jackin's `Screen` cannot reach its request bus
**§3.4 vs §18.3 #22.** §3.4 shows `fn update(&mut self, cx: &mut Cx<'_>, w: &mut World)` and then calls `cx.go(Go::Manager)`. Library `Cx` has no `go`/`status`/`open`/`close`/`help`/`copy`/`with_form`. `[F]` The current `Cx` (`src/bin/jackin_preview/screens/mod.rs:193-227`) carries `requests: Vec<Request>` and eight helpers, all product-specific. §18.3 #22 correctly says the product half stays per-app — which implies **two** contexts, contradicting §3.4's one.
**Fix (adjudicate, recommend the two-parameter form):**
```rust
fn update(&mut self, cx: &mut Cx<'_>, jx: &mut Jx<'_>, w: &mut World) -> Response<()>;
// Jx is jackin-owned: requests, go, status, help, copy, with_form.
```
Rejected alternative to record: `Cx<'f, R>` generic over a request payload — it puts a type parameter into *every component's* `update` signature, violating §13's "no gratuitous generic parameters".

Also: `Screen::on_msg(&Msg, …)` is listed as "removed or subsumed", but `Msg` is a **domain** event from the arbiter, not an `Input`, and `App::update(&mut self, cx)` has no message channel. **Add to §17.0 A1:** *"Domain messages enter at the top of `App::update`, drained from the application's own queue before any screen `update` runs. `Input` deliberately has no `Msg` variant."*

### M13 — `Harness` cannot be built from a separate crate
**§16.4, Appendix B.3.** `crates/tui-testing` is a separate crate and can use only `pub` API. `Harness` needs `hover()`, `state_of(id)`, `top_layer()`, `is_open(id)`, `focus_visible()`, `resolved(id, part)`, `cursor()` — none of which `Runtime` exposes in §17.0 A1. `Harness::resolved` is used in §16.4's theme-coupling paragraph but is absent from the `impl Harness` block. `Harness` has no `app()`/`app_mut()` even though `[F]` existing tests read `h.app.quit` (`app_tests.rs:143`) and `h.app.focus.current()` (`:191`).
**Fix:**
```rust
// §17.0 A1, gated so it never ships in a release binary:
#[cfg(feature = "testing")]
impl<A: App> Runtime<A> {
    pub fn hover(&self) -> Option<Id>;
    pub fn state_of(&self, id: Id) -> StateFlags;
    pub fn focus_visible(&self) -> bool;
    pub fn top_layer(&self) -> LayerId;
    pub fn is_open(&self, id: Id) -> bool;
    pub fn cursor(&self) -> Option<Position>;
    pub fn resolved(&self, id: Id, p: Part) -> Resolved;
}
// §16.4:
impl<A: App> Harness<A> {
    pub fn app(&self) -> &A;
    pub fn app_mut(&mut self) -> &mut A;
    pub fn resolved(&self, id: Id, p: Part) -> Resolved;
    pub fn with_auto_draw(self, yes: bool) -> Self;   // see M14
}
```
`crates/tui-testing` depends on `junie-tui` with `features = ["testing"]`; §26's `--all-features` enables it everywhere, which is intended.

### M14 — `Harness::handle` always draws, so redraw suppression cannot be tested
**§16.4.** `handle` "draws before returning", yet `pub fn draw(&mut self)` exists "for tests that assert on redraw suppression". With an unconditional draw, no test can observe suppression.
**Fix:** `Harness::with_auto_draw(false)` (M13) plus `pub fn last_invalidate(&self) -> Invalidate;`.

### M15 — `Outcome` → `Response` mapping covers only one of three cases
**§16.4 item 1.** `[F]` The existing suites assert `Outcome::Changed`, `Outcome::Consumed` and `Outcome::Ignored`. The mapping table gives only `Changed`.
**Fix:** state all three: `Changed → .is_changed()`; `Consumed → .is_consumed() && !.is_changed()`; `Ignored → !.is_consumed()`.

### M16 — `Conformance::Fixture` holds no item data
**§16.2.** `Fixture { disabled, read_only, items: usize, theme, color, area }` — `items` is a *count*. But `fn update(cx, st, f: &Fixture)` must build props that borrow `&'a [T]`, `item_keys(f) -> Vec<ItemKey>` must return real keys, and `reorder(f, perm)` must permute data that does not exist. Cases 12 and 2 cannot run.
**Fix:** `pub struct Fixture { …, pub rows: Vec<FixtureRow> }` with `pub struct FixtureRow { pub key: ItemKey, pub label: String, pub meta: String, pub disabled: bool }`, and `update`/`draw` borrow from `f.rows`. Also bound `type Action: PartialEq + core::fmt::Debug` (case 2 compares actions structurally; case 12 inspects them).

### M17 — `bindings_match_handled_keys` is false for every text-editing component
**§16.2 case 20.** "every chord `update` consumes in that state appears in `bindings(state)`" — a `TextInput` consumes every printable `Char(c)`. `TextInputCase`, `TextAreaCase`, `CodeEditorCase`, `SecretInputCase`, `PickerCase` (query) and `FilterListCase` all fail by construction.
**Fix:** add `Caps::TYPES` (declared by any component whose focus entry sets `swallows_typing`) and exclude bare `Char` chords from case 20's reverse direction for those components. State it in the case-20 row.

### M18 — `architecture::every_named_test_exists` will be permanently red
**§16.5.** It "parses this section's names out of `COMPONENT_ARCHITECTURE.md` and compares against `cargo test -- --list`". `--list` cannot show: `trybuild` compile-fail cases (`must_use_is_enforced`, `secret::is_not_clone_not_eq`), perf tests (a different profile/binary), or the macro-generated per-component names in the exact form the prose writes them. And `capsule_pane_clone_4x2000`'s *deletion* is asserted by parsing prose — precisely the fragile text check goal §25.5 discourages.
**Fix:** make it one-directional and scoped: *"every name listed in §16.1, §16.2's suite-level list and §16.4 exists in `cargo test --workspace -- --list`; §16.6 perf names are checked against `cargo test --workspace --test perf --release -- --list`; `trybuild` cases are checked against `tests/ui/*.rs` filenames; extra tests are allowed."* Delete the deletion-assertion clause and instead add `perf_baseline.txt` line-absence as the check.

### M19 — Numbering does not track goal §9's 20 required sections
**Goal §9 item 15 = "Package or workspace boundary"; the document's §15 = "Forms and text editing"**, and the package boundary lives in Appendix B. A verifier checking "all 20 sections present" scores a miss on item 15. All 20 topics *are* covered.
**Fix:** add a §0 traceability table mapping goal §9 items 1–20 → document sections, noting §15 (forms, goal §19) is an additional section and goal item 15 is Appendix B.

### M20 — No §23 scenario traceability
Scenarios A–I are each satisfied somewhere, but nothing maps them. Scenario H (TablePro adapter) has **no example** — only Slice-6 tests.
**Fix:** add a table: A→`examples/11`, B→`examples/02` + `Theme::paper()` + `downgrade_works_for_a_user_supplied_theme`, C→`examples/05` + `render::overrides::instance_patch_*`, D→`examples/07` + `no_full_collection_clone_per_frame`, E→`examples/08` + `conformance::tabs::item_identity_survives_reorder`, F→`examples/10` + `layer::nested_layers_each_trap`, G→`examples/12` + `showcase::author_component_page_*`, H→`crates/tui/tests/fixtures/grid_model.rs` + `tablepro::grid_adapter_keeps_every_pending_change_capability` + `architecture::no_domain_vocabulary_in_the_library`, I→§20.10 + `showcase_visual_baseline`.

### M21 — Goal §15 scenarios 3 and 5 have no executable acceptance
Scenario 3 ("change a small set of semantic roles and inherit or derive the rest safely") maps to `ThemeBuilder::build`, whose derivation algorithm is unspecified (→ M22) and untested. Scenario 5 ("override one component **variant** globally") maps to `Theme::override_variant`, which has no named test — `render::overrides::*` covers family, scope and instance only.
**Fix:** add `theme::builder_derives_every_unset_token_deterministically`, `theme::derived_tokens_meet_design_contrast_ratios`, and `render::overrides::global_variant_override_changes_only_that_variant`.

### M22 — `ThemeBuilder::build`'s derivation is aspirational
**§17.0 A5.** *"Fills every token the caller did not set by deriving from the ones they did, preserving `DESIGN.md`'s contrast relationships. Deterministic and tested."* No algorithm. Two builders will produce two different `Theme::from_tokens` results, and `Theme::paper()`'s ~25 unspecified tokens depend on it.
**Fix (exact), write the algorithm into §11.2:**
> Given `surfaces[0]` and `accent`: `surfaces[1..4]` step L\* by +4 (dark base) or −4 (light base, detected by `L*(surfaces[0]) > 50`); `fg[0..4]` step L\* by −18 from a contrast-7:1 anchor against `surfaces[0]`; `accent_hover = ΔL* +8`, `accent_pressed = ΔL* −8`, `accent_tint = accent at 12 % over surfaces[1]`; `focus = accent`, `focus_ring = accent_pressed`; `border_subtle = surfaces[3]`, `border_strong = fg[3]`; `danger/warning/success/info` tints at 12 %; `on_accent`/`on_danger` = whichever of `fg[0]`/`surfaces[0]` reaches ≥ 4.5:1. Every derived value is a pure function of the seeds; `theme::builder_derives_every_unset_token_deterministically` pins the table.

### M23 — `downgrade_color` has no metric
**§11.4.** *"nearest_256 / nearest_16 / mono"* — three different implementations are possible and they produce three different `ansi16` palettes, all of which get frozen into `render::components::*_16` digests.
**Fix:** specify. `nearest_256`: nearest in the 6×6×6 cube ∪ 24-step greyscale by squared sRGB distance, ties to the lower index. `nearest_16`: nearest of the 16 xterm defaults by CIE76 ΔE. `mono`: `Y = 0.2126R + 0.7152G + 0.0722B`, then `Y < 0.35 → black`, `Y > 0.75 → white`, else `Color::Reset`. Name the test `theme::downgrade_is_deterministic_per_level`.

### M24 — `App::keymap` returns a reference to a temporary
**§17.0 A1.** `fn keymap(&self) -> &KeyMap { KeyMap::empty() }` does not compile.
**Fix:** `fn keymap(&self) -> &KeyMap { KeyMap::EMPTY_REF }` with `impl KeyMap { pub const EMPTY: KeyMap; pub const EMPTY_REF: &'static KeyMap = &KeyMap::EMPTY; }`.

### M25 — `scroll_region` is both a `Ui` helper and a component file
**§12.2 ("one `scroll_region(part)` helper on `Ui`/`Cx`") vs §18.2 / B.2 (`components/scroll_region.rs`) vs §16.2 (`ScrollRegionCase` in `conformance_suite!`).**
**Fix:** make it a component (`ScrollRegion<'a>` in `components/scroll_region.rs`) with a `Ui::scroll_region(id, part, …)` convenience that constructs and draws it, and say so in §12.2.

### M26 — `Conformance` list contains three non-components
`ScrollbarCase` (§18.2 says the scrollbar is *a part*, not a component), `EmptyCase` (`EmptyState` is a data enum rendered *inside* collections), `SecretInputCase` (no `SecretInput` type exists — `TextInput::secret(policy)` is the mechanism). `PropsList` is missing.
**Fix:** delete `ScrollbarCase`, `EmptyCase`, `SecretInputCase`; add `PropsListCase`; register the secret path as a `Caps::SECRET` fixture variant of `TextInputCase`.

### M27 — §16.3 `Scene` cannot build a `Ui`
`Scene::draw(&mut self, f: impl FnOnce(&mut Ui<'_>, Rect))` needs a registry, focus ring, layer stack and style stack. Nothing says where they come from.
**Fix:** `Scene` owns a headless `FrameState` (registry + ring + layers + style stack) built from the theme; declare it in §16.3.

### M28 — `Registry::names` is redundant and, for item ids, impossible
**§7.1 vs §20.9-5.** §7.1 keeps `Registry::names: HashMap<u64, DebugLabel>` "populated at registration"; §20.9-5 moves population to construction. But `Id::item(k)` is runtime, so item ids can never be in a const table — yet `DebugLabel { root, tail: Tail::Item(ItemKey) }` **already carries the label inline on the `Id`**.
**Fix:** delete `Registry::names` entirely. Record: *"the debug label travels with the `Id` (`Tail::Item(k)`), so no side table exists in any build. §7.1's 'the runtime also maintains `Registry::names`' is struck."* This makes §20.9-5's `debug-ids` feature unnecessary and `debug_and_release_alloc_counts_match` trivially true.

### M29 — `inert_below` vs `CursorRejected` vs focus restore
**§9.1, §20.9-16, §8.4, §16.2 case 17, §3.3 step 14.**
* §20.9-16 says `inert_below` suppresses *registration*, so a suppressed page never calls `set_cursor` and no `CursorRejected` diagnostic is produced. Case 17 asserts exactly one is. Contradiction.
* With the page registering nothing, the page's focus entries vanish from the ring while a modal is open. Between `cx.close_layer` (in `update`) and the next `draw`, `FocusState::current` names an id absent from the *last* ring, so a key in that window resolves to `None`. Race.

**Fix:** (a) §8.4: *"a `set_cursor` from a suppressed (inert) layer is discarded silently; `CursorRejected` is recorded only for a non-inert lower layer or an unfocused owner."* Case 17 uses a `Popover` (pointer barrier only, no `inert_below`). (b) §3.3 step 14: *"focus restoration is staged at `close_layer` and applied at the next draw's reconcile. Until then, `FocusState::current` is the restore target and key resolution uses it even though it is absent from the last ring; this is the one documented exception to 'resolve against last frame'."* Add `focus::restore_target_receives_keys_before_the_next_draw`.

### M30 — Slice 3 / Slice 4 gates cannot catch a legacy regression
**Appendix A.** Every Slice 3/4 gate is scoped `-p junie-tui -p junie-tui-testing`. Under the staging plan (§7), the legacy package's 198 tests must stay green throughout, and nothing runs them.
**Fix:** append `cargo test -p <legacy-package> --all-targets` to the Slice 3 and Slice 4 gate blocks.

### M31 — No per-component answer to goal §10's 17 questions
Goal §10: *"Every public component must have documented and consistent answers to…"* (17 questions). §13's table answers them **globally**; there is no per-component surface and no check.
**Fix:** mandate a rustdoc template on every component (`## Construction / ## Ownership / ## Configuration / ## Variants / ## States / ## Actions / ## Focus / ## Keyboard / ## Mouse / ## Layout / ## Parts / ## Overrides / ## Identity / ## Testing / ## Invariants`) and add `architecture::every_component_doc_has_the_standard_sections` (rustdoc-json heading scan).

---

## 3. Ergonomics, naming, ownership, lifetimes, testability, event flow, author experience

Read as a downstream user of §17.

### The `Cx` / `Ui` split is correct — keep it
This is the strongest design decision in the document and I would resist any move to a single phase-checked type. Two disjoint capability sets is a *compile-time* encoding of G2 at the call site: a component in `update` has no `Ui` and physically cannot paint; a component in `draw` has no `Cx` and cannot open a layer or move focus. A single `Ui` with runtime phase assertions would move that back to a review rule — the exact regression §3.2 rejects `show()` for.

**But** the duplicated read accessors (`state`, `theme`, `design`, `area`, `layout`) will teach authors two vocabularies for the same query. **Recommend:**
```rust
pub trait FrameRead {
    fn state(&self, id: Id) -> StateFlags;
    fn theme(&self) -> &Theme;
    fn design(&self) -> &DesignTokens;
    fn area(&self, id: Id) -> Option<Rect>;
    fn layout(&self, id: Id) -> Option<LayoutFacts>;
}
impl FrameRead for Ui<'_> {} impl FrameRead for Cx<'_> {}
```
Re-export in `author`. One vocabulary, two capability sets.

### `update`/`draw` on props with external `XState` for a 15-field form
Honest assessment: **it is workable but noticeably heavy, and §20.7 understates the cost.**

A 15-field form costs the caller 15 `String`s + 15 `TextInputState`s + one `FormState`, and *both* phases must reconstruct 15 props structs with matching builders. Drift between the two constructions is a silent bug class the compiler cannot see: example 6 already demonstrates it — `update` builds `TextInput::new(EMAIL).validate(&valid_email).blur(CommitAndValidate)`, `draw` builds `TextInput::new(EMAIL).value(&self.email)`; the two share no code and differ in three builder calls. Nothing detects a `disabled(…)` predicate that is applied in `draw` but forgotten in `update` — which is exactly the class of bug that makes a disabled control still activate.

§20.2's mitigation ("callers can factor a `fn button(&self) -> Button<'_>` helper") is the right answer but is optional; it should be **mandatory** for anything with configuration.

**Recommend, add to §13 as a binding convention:**
> A component instance with any configuration beyond `new(id, …)` is built by exactly one private constructor function on the owning screen, called from both phases. `architecture::props_are_built_once` (a `syn` check that no `X::new(` appears more than once per screen module for the same `const Id`) reports violations. `J2`'s `Form` provides `Form::field(id, …)` so a 15-field form declares each field once and `Form` drives both phases.

Additionally, **`Form` (J2) must be specified as the answer to the 15-field case and is currently only a table row.** Without a `Form` that owns the field list and forwards both phases, §19's "three form engines collapse to one" is not achieved and every jackin/TablePro form becomes 30 hand-written call sites. **Give §15 a `Form` API sketch** before Slice 3, because 4F depends on it.

### `Response<A>` BitOr — which action wins?
As written: "action: first Some". This is **unsound for any `A ≠ ()`** and silently drops a semantic action. Nothing in §17 actually relies on it: every `r |=` operand has already been reduced to `Response<()>` by `on_action`/`on_activated`/`erase`.

**Recommend (exact):**
```rust
impl std::ops::BitOr for Response<()> { /* flow: Consumed wins; invalidate: max; id: lhs; state: lhs */ }
impl std::ops::BitOrAssign for Response<()> {}
```
Restrict to `()`. Composing two action-carrying responses becomes a type error instead of silent data loss, and every §17 usage still compiles. Also specify `id` and `state` under BitOr (currently undefined) — `id` = lhs, `state` = lhs, documented as "the fold is a control-flow summary; read `state`/`id` from the individual `Response`s".

Related: `Response::ignored()` produces an `id` field with no value, and §6.1 declares `pub fn id(&self) -> Id` (non-optional). **Fix:** `id: Option<Id>` with `pub fn id(&self) -> Option<Id>`, or add `pub const Id::NONE`.

### `Intent<'f>` inside `update` while `XState` is `&mut`
The `Paste(&'f str)` borrow is *not* the problem — `st` and `cx` are disjoint. The problem is the iterator's borrow of `Cx` (B4). Once B4's split is applied (`intents: &'f IntentQueue` shared, services `&mut`), `for it in cx.intents(id) { st.foo(); cx.bar(); }` compiles cleanly and `'f` outlives the loop naturally.

One remaining wrinkle to document: `Intent::Paste(&'f str)` means the paste text must live in a runtime-owned frame arena for the whole of step 7, not in the `Input` value. Say so in §3.3 step 1.

### `draw(&self, …, st: &XState)` and per-frame caches
See **B5**. The design deliberately blocks them, and three perf amendments require them. `LayoutFacts` does not solve it (it flows up, not sideways). `Ui::cache(id)` is the right home: runtime-owned, keyed by `(Id, TypeId)`, cleared on resize/theme change, contractually derived-only. Without it, §20.9-7/-8/-9 must be withdrawn and the `TextViewport`, `Tree` and `CodeEditor` perf thresholds cannot be met.

### `Intents::take(id)` for parts and nested containers
* **Parts and items: yes, cleanly.** A click on a list row registers `owner = list_id, part = ROW, item = key`. `cx.intents(list_id)` delivers it and the component pattern-matches `PartRef`. This is a genuine improvement over 12 `owns`/`locate` pairs.
* **Nested containers (Panel containing Button): correct by last-registration-wins**, since the Button draws inside the Panel and registers later. But it produces the `UndeliveredIntent` contradiction of **B12** for clicks in the Panel's padding. `RegionKind::Decorative` resolves it.
* **A third case the document misses:** a `Grid` cell containing an inline editor (`EditIntent::Inline`). Two owners (grid, editor) both want the click. Say explicitly that the inline editor registers a `Control` region *after* the grid's cell `Part` region and therefore wins, and that the grid must not treat a click inside an active edit as a cursor move. Add `grid::click_inside_an_active_inline_edit_goes_to_the_editor`.

### Naming
* `Phase` (pointer phases) and `Phase2` (`Capture | Bubble`) — `Phase2` is a placeholder that will ship. **Rename to `KeyPhase`.**
* `Surface::Surface` (a variant named after its enum, §11.1 A1) reads badly in every match arm. It is `DESIGN.md`-mandated, so keep it, but note it in §20 trade-offs.
* `Response::changed()` constructor vs `Invalidate::Paint` — "changed" meaning "repaint" is a mild misnomer inherited from `Outcome::Changed`; keep it for migration continuity (it makes the ~60 assertion rewrites mechanical) and say so.

---

## 4. The frame model — race and ordering findings

| # | Issue | Verdict |
|---|---|---|
| F1 | Esc dismisses the layer before the focused editor sees it | **BLOCKER B6** |
| F2 | Focus re-run loop re-entry semantics undefined → double activation | **BLOCKER B7** |
| F3 | `hit()` layer filter contradicts click-outside | **BLOCKER B11** |
| F4 | `LayerId` assignment order (open vs draw) unspecified → z-order race | **MAJOR M4** |
| F5 | Focus restore between `close_layer` and the next draw | **MAJOR M29(b)** |
| F6 | `inert_below` + cursor rejection contradiction | **MAJOR M29(a)** |
| F7 | One-frame pointer latency vs `handle` truthfulness | **Not a bug.** `Harness::new` draws, and `Harness::handle` draws after every input, so a test never observes the latency; `click_id` always resolves. **State this in §16.4** and specify `click_id`'s behaviour when `area_of` is `None` (return `Response::ignored()` + a `Diagnostic::UnaddressableId`, never panic). |
| F8 | Capture released at draw (step 13) but drag already delivered at step 3 | **MINOR.** A drag can reach an owner whose layer closed in the previous `update`. Fix: release captures at the *end of step 7* as well as step 13, whenever `close_layer` removed the capture owner's layer. |
| F9 | Wheel over an inert backdrop | **MINOR.** With `inert_below`, no scroll regions are registered beneath, so the wheel falls through to app bubble. Correct — but say so, otherwise a builder may add outward chaining. |
| F10 | Two `ui.layer` calls with the same `id` in one frame | Unspecified. **MINOR:** second call returns `None` and records `Diagnostic::DuplicateLayerDraw`. |

The two-phase model itself is sound. Every finding here is an ordering or a specification gap, not a structural flaw.

---

## 5. Feasibility of the 55K-line migration

### jackin `Screen` trait reduction — feasible, with one unresolved dependency
`[F]` The current trait (`screens/mod.rs:231-328`) has **23** methods (the document says 20), 11 of them pointer/key input, plus `Cx { focus: &mut Focus, ring: &FocusRing, requests: Vec<Request> }` (`:193-197`). Reducing to 6 is realistic: pointer/wheel/paste/focus genuinely collapse into `Intent`s, `on_modal` genuinely becomes `LayerEvent`, and `is_editing`/`animating` genuinely become `StateFlags::EDITING` + `request_repaint_after`.

Blocking dependency: **M12** (the request bus) and the `on_msg`/`Msg` channel. Resolve both before Slice 7 planning; ideally before Slice 3, because `Cx`'s shape is a Slice-3 deliverable.

### TablePro grid adapter — feasible, but §20.9-11 is self-contradictory
§20.9-11 requires "a single owned `ResultSet` that the `GridModel` borrows" **and** "`sample_widths` calls a non-allocating `CellValue::display_width()` instead of materialising a `String`". If cells are pre-formatted strings (which is what `CellRef<'_>` = "borrowed text" implies and what `< 100 allocs/frame` requires), `display_width` is trivial and `CellValue` need not exist at draw time. If cells stay typed values, `CellRef` cannot be "borrowed text" and every frame formats.

**Fix:** commit to pre-formatting at load. `apps/tablepro/src/grid_model.rs` stores `ResultSet { text: Vec<String>, kind: Vec<CellKind>, … }` (one `String` per cell, produced once — 6 000 for the 500×12 benchmark, within the `< 8 000` budget); `CellRef<'a> { text: &'a str, tone: Option<Role>, align: Align }`; `CellValue` survives only in the *domain* model for SQL generation and validation. Delete the `CellValue::display_width` clause. Also apply **B15** (`model: &mut M` in `update`).

DOM §1.6's 22-capability checklist is the right instrument for Slice 6 and I have no objection to it.

### Test-harness contract vs the existing suites — the largest concrete risk
`[F]` Verified against `src/bin/showcase/app_tests.rs`:

| Existing usage | §16.4 coverage | Gap |
|---|---|---|
| `Harness::new(w, h, page)` then `app.goto(page)` (`:20-22`) | `Harness::new(app, theme, w, h)` | needs app lib target — **B9** |
| `self.app.focus.current()` (`:126, :191`) | `Harness::focus()` | ✓ |
| `self.app.hits.area_of(f)` (`:128`) | `Harness::area_of(id)` | ✓ |
| `h.app.quit` (`:143`) | — | **M13** (`Harness::app()`) |
| `crate::app::NAV_ENTRIES` (`:156`) | — | **B9** |
| `h.term.backend_mut().resize` + `Input::Resize` (`:182-183`) | `Harness::resize(w,h)` | ✓ |
| `assert_eq!(h.key(…), Outcome::Changed)` | `.is_changed()` | **M15** (3-way mapping) |
| `focus_bar_x` compares `cell.fg == Theme::junie().focus` (`:112-118`) | `h.resolved(id, Part::GUTTER).style.fg` | **M13** (`Harness::resolved` missing from the impl block) |
| `ring.reachable()` (jackin) | `Harness::ring().reachable()` | ✓ |
| `WidgetId::of("editor.cfg").sub("form").sub("save")` | `screens::editor::CFG_FORM.part(SAVE)` | **B9** + **M5** |
| `assert_eq!(seen.len(), 8)` — exact ring size (`:207`) | — | Ring size **will change** (`Field` chrome, `NavList`, disabled-but-registered entries). Not a defect, but §20.10 must list "focus-ring composition changes in migrated screens" as an intentional change with a per-test classification, or Slice 5 will hit unclassified red tests. **Add as §20.10 item 15.** |

`Harness::actions() -> Vec<AppActionRecord>` has no mechanism at all — delete it, or add `Cx::record(&'static str)` gated behind the `testing` feature.

### Overall feasibility
With B9, B15, M12, M13 and the §7 staging plan resolved, I judge the migration tractable within Slices 5–7. Without B9 it is not: §16.4's entire contract is unbuildable.

---

## 6. Performance amendments §20.9 — are R1–R7 folded?

**Yes, all seven, correctly mapped:** R1 → 20.9-3 (reconcile stamp + index probe), R2 → 20.9-1 (pre-sorted state rules), R3 → 20.9-4 (linear overlay scan, short-circuit on empty), R4 → 20.9-5 (debug names at construction), R5 → 20.9-6 (`RowUi` single grapheme walk), R6 → 20.9-12 (`Intents` index), R7 → 20.9-13 (`KeySet::AllExcept`). Each carries a named acceptance test in §16.6. This is the strongest-executed part of the document.

### Remaining hot-path problems

| # | Sev | Finding | Fix |
|---|---|---|---|
| P1 | MAJOR | **`HintLayer` allocates per frame.** §13.1 derives hints from the focused component's bindings each frame; `HintLayer { hints: Vec<Hint>, status: Option<String> }` and `Screen::hints(…) -> HintLayer` (by value) are a `Vec` + `String` on every frame of every app. No §16.6 line covers it. | `HintLayer { hints: SmallVec<[Hint; 8]>, status: Option<Cow<'static, str>> }`, cached behind `(focus_id, StateFlags, top_layer)` in `Ui::cache`. Add `frame_hintbar_derived` — **0 allocs/frame when focus is unchanged**. |
| P2 | MAJOR | **`intents_drain_scales_with_intents_not_components` is unachievable.** | **B14**. |
| P3 | MINOR | **`overlay_stack_hash` recomputed per style query** (§11.1 A3 / §20.9-2). At grid scale that is one hash per part per cell. | Keep a running `stack_hash: u64` on `Ui`, updated on `with_overlay` push/pop. State it in §20.9-2. |
| P4 | MINOR | **The memo cache is ~8 KB inside `Ui`** (`[Option<(u64, Resolved)>; 256]`). `Scene` and `Harness` construct a `Ui` per frame. The generation-stamp clearing (already specified) avoids the memset; say explicitly that `Ui` is constructed once per `Runtime`/`Scene` and reused, not per frame. |
| P5 | MINOR | **`Secret::masked_tail(&self, n) -> String` allocates.** If a masked field calls it from `draw`, that is one alloc/frame per secret field. | Return into the row painter: `fn write_mask(&self, out: &mut CellUi<'_>, n: usize)`, or cache the synthetic tail in `TextInputState` at `begin`. |
| P6 | MINOR | **`Diagnostic` vector is unbounded.** A persistent `DuplicateId` grows the vec every frame. | Cap at 64 with a dropped-count, and clear at the start of each `handle`. State it in §17.0's `Diagnostic` declaration. |
| P7 | MINOR | **`Id` is ~48 B in debug** (`DebugLabel { root: &'static str, tail: Tail }`) vs 8 B in release, so every `Region` roughly doubles. `debug_and_release_alloc_counts_match` checks allocations, not size. Acceptable; record it in §20.4 so nobody is surprised by debug registry memory. |
| P8 | MINOR | **`frame_showcase_lists_120x40` "hits within ±10 %"** cannot hold: `Field` chrome, `NavList`, `scroll_region` parts and disabled-but-registered ring entries all change region counts materially. | Replace with "hits recorded and classified in `docs/visual-changes.md`; no unexplained growth > 25 %". |

No remaining O(n)-per-event path is left in the design once B14's fast path lands: pointer is one reverse scan of the top layer, wheel likewise, key is one focus lookup, and per-component drain is one probe.

---

## 7. Coordinator staging (adjudication)

**The proposal as stated is rejected.** Renaming the root package to `junie-tui-legacy` while both packages live in one workspace fails on three concrete points:

1. **Doc-target collision.** Renaming the *package* but leaving `[lib] name = "junie_tui"` gives two crates whose lib name is `junie_tui`. `cargo doc --workspace` writes both to `target/doc/junie_tui/` — an output-filename collision, which under `RUSTDOCFLAGS="-D warnings"` (a §26 gate) is fatal. Avoiding it forces `[lib] name = "junie_tui_legacy"`, which means rewriting every `use junie_tui::` in the three apps **twice**: once to `junie_tui_legacy` now, once to `junie_tui` at migration. `[F]` The apps are 55K lines across `src/bin/{showcase,tablepro,jackin_preview}`; that is a large, meaningless diff landing in the middle of slices whose diffs must be reviewable.
2. **Duplicate binary names.** `[F]` The root package declares `[[bin]] showcase`, `tablepro`, `jackin-preview` (`Cargo.toml:15-25`). From Slice 5, `apps/showcase` declares `showcase` too. Cargo permits duplicate bin names across packages, but `cargo run --bin showcase` becomes ambiguous and `target/debug/showcase` is whichever package built last — which silently corrupts `tools/capture.sh` and every manual run in the exact slices where visual comparison matters.
3. **`default-run`.** `[F]` `Cargo.toml:9` sets `default-run = "showcase"`. It is a package key with no workspace equivalent (Appendix B already records this). Keeping it on a legacy package that also owns three bins means `cargo run` at the workspace root resolves to the *legacy* showcase for the whole of Slices 3–7.

### Recommended plan: temporary library name, single rename, no legacy churn

* **Slices 3–4.** The repository root stays a package **and** becomes the workspace root:
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
  Consequences: no duplicate lib name ever exists; `cargo doc --workspace` never collides; `default-run`, the three `[[bin]]`s, `tools/capture.sh`'s `BIN`, and all 198 tests are untouched; `cargo test --workspace` runs both trees, so "keep green throughout" (goal §27) is literally true.

* **Start of Slice 5, one commit, no behaviour change:** delete the root package's `[lib]`, `src/`, `src/bin/*`, `default-run` and its `[[bin]]`s as the apps move to `apps/*`; then rename `tui-next` → `junie-tui` / `tui_next` → `junie_tui` by scripted `sed` over a closed, slice-owned file set (`crates/tui/**`, `crates/tui-testing/**`, `xtask/**`, `crates/tui/examples/**`, `crates/tui/tests/**`). Re-run the full Slice-4 gate. Apps' `use junie_tui::…` lines are then written **once**, in their own migration slice, where the diff belongs.

* **Strike** Appendix A Slice 3's sentence *"Applications do not compile during this slice. They are excluded from the workspace default members until Slice 5–7"* and replace it with the above.

* **Risks to record in `REFACTORING_STATE.md`** so a resumed or compacted session does not "correct" the temporary name:
  * The name `tui-next` is deliberate and temporary; the rename is a scheduled Slice-5 step, not a defect.
  * Appendix B.4's `junie_tui::author` paths and every `architecture::*` test name are written against the final name; during Slices 3–4 they read `tui_next::author`. Record the one-line mapping.
  * `xtask` and `crates/tui-testing` depend on the temporary name and are renamed in the same commit.
  * `tools/capture.sh`'s `BIN=${BIN:-target/debug/junie-tui}` is `[F]` already stale (no binary of that name exists); it changes to `target/debug/showcase` in Slice 5 exactly as Appendix B.5 schedules — independent of the rename.
  * WP‑0 (**B16**) lands `tests/perf.rs` + `tests/perf_support.rs` in the **existing root package**; Slice 3 moves the harness to `crates/tui-testing/src/perf.rs` and the library benchmarks to `crates/tui/tests/perf.rs`, keeping the app benchmarks at the root until Slices 5–7 move them with their apps. `perf_baseline.txt` moves with the harness and its line names never change, so "before/after" stays literal.

* **Alternative considered and rejected:** keeping `junie-tui` on the new crate and renaming the root package's lib. Rejected because the root package's binaries reference their own lib by its `[lib] name`, so the rename necessarily rewrites all three apps twice (point 1 above).

---

## 8. Aspirational, vague, or ambiguous enough to be implemented two ways

| # | Location | Problem | Fix |
|---|---|---|---|
| A1 | §17.0 A5 `ThemeBuilder::build` | "derives … preserving `DESIGN.md`'s contrast relationships" — no algorithm | **M22** |
| A2 | §11.4 `downgrade_color` | "nearest_256 / nearest_16 / mono" — no metric | **M23** |
| A3 | §11.7 `Theme::paper()` | 5 surfaces + 8 roles given; ~25 `ColorTokens` fields unspecified | Resolved once A1 lands; state that `paper()` is `from_tokens(seeds).builder()…build()` and pin it with `theme::paper_tokens_are_pinned` |
| A4 | §11.5 focus indicator | "the *glyph* is `GlyphSet::FocusBar`, *which parts wear it* is the recipe's `Part::GUTTER`" — nothing says the component must paint `Resolved.glyph` | Add to §5: *"a component that declares a part must paint `Resolved.glyph` when `Some`; `conformance::registry::declared_parts_are_the_parts_actually_styled` checks the query, `mono_states_are_distinguishable` checks the paint."* |
| A5 | §16.2 suite-level `declared_parts_are_the_parts_actually_styled` | Requires the runtime to log `Ui::style` calls; no mechanism | Add `#[cfg(feature = "testing")] Ui::styled_parts(&self) -> &[(Id, Part)]` |
| A6 | §20.10 "the painter must be byte-identical to `fit` for every input" | "every input" is not testable | Name the corpus: `crates/tui/tests/fixtures/text.rs` — ≥ 200 strings covering ASCII, CJK wide, combining marks, ZWJ emoji, RTL, and widths 0..=120; test `text::row_ui_matches_fit_for_every_fixture` (currently named in §20.10 but **absent from §16.1's `text/` list** — add it) |
| A7 | §8.5 / §20.5 `Invalidate::Layout` | Ships but "behaves as `Paint`" | Acceptable and documented; keep, but add `response::layout_is_strictly_greater_than_paint` as the only assertion so no builder invents layout caching early |
| A8 | §12.3 `EditIntent::External` | Named in the enum, never explained | Define: *"the component emits `EditRequested(item, col)` and does not begin an inline edit; the application opens its own editor."* |
| A9 | §12.2 `RowDecor.message: Option<&'static str>` | `'static` in a component surface contradicts §2.2's "no `'static` requirement anywhere in a component's public surface" | Change to `Option<&'a str>` and add `RowDecor<'a>`; same for `CellDecor.error` |
| A10 | §9.1 `Dismiss.focus_out` | Meaningless for `Modal` (which traps focus) | Document: *"`focus_out` is honoured only for `Popover` and `Tooltip`; a `Modal` traps focus so it can never fire."* |
| A11 | §13 "`EDITING` is owned by state, never a prop" vs §18.3 #3 `.state_override(StateFlags)` | The showcase's static state matrix needs to *force* `EDITING`, `HOVERED`, `PRESSED` for display | Declare `.state_override(StateFlags)` in §17.0 A7 as a documented showcase/testing-only builder, and add `architecture::state_override_is_used_only_in_apps_and_fixtures` |
| A12 | §12.1 "`&'a dyn Fn` … the only `dyn` in a component's public surface" | `Validate` (§15), `Highlighter`/`Segmenter` (§14.1), `DiffSource` (§14.1), `GridModel` are also dynamic or trait-based | Restate: *"`&'a dyn Fn` slots and small `&dyn Trait` extension points are the only dynamic dispatch in a component's public surface; none is boxed and none requires `'static`."* |
| A13 | §14.1 `CodeEditor` "Diagnostic'" and `ColumnSpec'` | Primed names with no definition — evidently placeholders for "a renamed variant of an existing type" | Name them: `CodeDiagnostic`, `Column` |
| A14 | §16.3 `Baseline` regeneration | `BLESS=1` and `UPDATE_BASELINE=1` are aliases; `xtask bless-guard` requires an entry in `docs/visual-changes.md` "referencing a §20.10 item **and** a capture path under `shots/`" — but the capture cannot exist before the change is made | Specify the order: change → capture → classify → bless; `bless-guard` runs in CI on the committed tree, not locally |
| A15 | Goal §29 "no material TODO, stub, placeholder" | No check | Add `architecture::no_todo_or_unimplemented` (grep for `todo!`, `unimplemented!`, `TODO`, `FIXME` over `crates/**` and `apps/**`, empty allow-list) |
| A16 | Goal §29 "the showcase demonstrates every public component" | Only navigation coverage is asserted | Add `architecture::showcase_covers_every_public_component` (cross-check `conformance_suite!` against the showcase page registry) |
| A17 | Goal §14 "convenience APIs … on top of the same primitives rather than a separate rendering path" | No test | Add `dialog::convenience_constructors_render_through_the_body_slot` — digest equality between `Dialog::confirm(…)` and the hand-composed equivalent |

---

## 9. (a) Verdict, (b) ordered edits, (c) Slice 2 acceptance conditions

### (a) Verdict

**Not ready to implement as written. Ready after the edits in (b).**

The architecture's *decisions* are correct and I found no reason to reopen Adjudications A–I or Appendix B's workspace decision. What is not ready is the **specified surface**: §16–§20 and §17's twelve examples were written against an API that §3–§15 does not declare, and three of the defects (B2 collection builders, B3 props-borrow vs mutating closure, B5 per-frame caches) are structural rather than editorial — they need the three narrow amendments named below, all of which are consistent with the accepted model and two of which simply extend a rule the document already applies to controlled values (§17.0 A3).

The document is also **too large to be self-consistent by hand** at 2 768 lines with two synthesis passes. §17.0 exists precisely because §16–§20 outran §3–§15; the fix is not more prose but a mechanical check. I recommend adding, in Slice 3, an `xtask doc-check` that extracts every `` `Ident::method` `` and every ```` ```rust ```` block from `COMPONENT_ARCHITECTURE.md` and asserts each resolves against the compiled `junie_tui` rustdoc-json. That converts this entire review category into a permanent CI gate.

**Do not** apply the edits as a re-synthesis. Apply them as a numbered changelog appended to the document (a new `§21 Adjudication J — Slice 2 review corrections`) with each item citing the section it amends, exactly as §20.9 does for performance. That preserves reviewability and keeps `REFACTORING_STATE.md` able to record which corrections have landed.

### (b) Ordered edits Fable must apply before Slice 3

Ordered by dependency: each group is safe to apply once the group above it has landed.

**Group 1 — the three amendments (require recording as adjudications, not edits)**
1. **B3/B15** — collection and model **data moves to the phase call**: `update(&self, cx, st, items: &[T])` / `draw(&self, ui, area, st, items: &[T])`; `Grid::update(…, model: &mut M)` / `Grid::draw(…, model: &M)`. Props hold configuration and closures only. Amend §12.1, §12.2, §12.3, §13's table, §17.0 A7, §20.2, §20.3, and rewrite examples 7, 8, 10, 11.
2. **B5** — add `Ui::cache<T>(id)` + rendering rule **R8**; restate §20.9-7, -8, -9 in its terms; add `architecture::cache_types_are_derived_only`.
3. **B6** — move Esc dismissal from §3.3 step 5 to step 8, after `app.update`; update §8.6 and §9.1's Esc row; add `layer::esc_reaches_the_focused_editor_before_the_layer`.

**Group 2 — compile-blocking API corrections**
4. **B1** `is_consumed` / `is_changed`; propagate to §16.4 item 1 and **M15**'s three-way mapping.
5. **B2** three-impl-block collection builders + `ByIndex`/`DefaultRow`; delete or blanket-impl `KeyFn`.
6. **B4** split `Cx<'f>`'s frozen intent queue from its mutable services; `intents(&self, id) -> IntentIter<'f>`; unify the name across §3.3, §17.0 A2, §20.9-12.
7. **B10** `FieldControl` trait; `Field::new(label, control)` with no `Id`; correct §15's struct and §17.0 A7; rewrite examples 6 and 11.
8. **B8** remove `ColorTokens`/`DesignTokens`/`RowDecor`/`CellDecor` from B.3 item 4; add `LayerSpec` builders to §17.0 A6; rewrite example 10 as a `popover`.
9. **M24** `App::keymap` → `KeyMap::EMPTY_REF`.
10. **M11** split `Bindings::Cmd` from the emitted action; fix example 12's `BINDINGS`.

**Group 3 — model/ordering specifications**
11. **B7** focus re-run semantics paragraph in §3.3 step 7; demote `focus_transition_settles` to suite level.
12. **B11** `Registry::hit` returns the topmost region regardless of layer; caller compares.
13. **B12** define `RegionKind`; redefine `UndeliveredIntent`.
14. **M4** `LayerId` assigned at `open_layer`; `Ui::layer` resolves and returns `None` when closed.
15. **M29** cursor-rejection under `inert_below`; focus-restore staging paragraph.
16. **M3** delete `spec` from §3.3 step 12. **M5** `Id::part` vs `PartRef` rule + `PartRef::of`/`::item`.
17. **F8**, **F9**, **F10** one-line specifications.

**Group 4 — undeclared types and members**
18. **M1** `ui.full`, `ui.register_part`, `cx.focus`, `cx.request_repaint*`; **M2** `Cx::capture(owner, part)`.
19. **M6** declare `Status`, `KeySet`, `ColorLevel`, `BindingState`, `Diagnostic`, `Backdrop`, `Density`, `SortDir`, `Hit`, `SyntaxTokens::derive`, `MeterTokens::derive`, `Chord::key`, `Key::is`; delete `UnicodeLevel`, `LayoutError`, `AppActionRecord`.
20. **M7** `ScreenAlign` vs `Align`; `LayerSpec.min_size: (u16, u16)`.
21. **M8** `ThemeBuilder::focus/selection/highlight/field`. **M9** full `CellUi` impl + `RowUi::part` semantics. **M10** `Reconcile` trait.
22. **M28** delete `Registry::names`. **A9** `RowDecor<'a>`/`CellDecor<'a>`. **A12**, **A13** wording.

**Group 5 — testing and migration contract**
23. **B9** app packages get `[lib]` + thin `[[bin]]`; update B.2, B.3 item 7, §16.4 item 3.
24. **M13** `Runtime` `testing`-feature inspection block; `Harness::app/app_mut/resolved/with_auto_draw`; **M14**.
25. **B13** mono `PRESSED` gains a modifier and bracket glyphs; add `GlyphRole::{PressLeft, PressRight}`.
26. **B14** `Cx::intents` O(1) empty fast path; rename and restate the perf test.
27. **M16** `Fixture` carries `rows`; bound `Conformance::Action: PartialEq + Debug`. **M17** `Caps::TYPES`. **M26** conformance list corrections.
28. **M18** one-directional `every_named_test_exists`. **M27** `Scene` owns a headless `FrameState`.
29. **M21/M22/M23** derivation and downgrade algorithms + their three test names. **A6** the text fixture corpus + add `row_ui_matches_fit_for_every_fixture` to §16.1.
30. **P1** `HintLayer` `SmallVec` + cache + `frame_hintbar_derived`; **P3**–**P8**.

**Group 6 — staging, traceability, gates**
31. **B16 / §7** replace WP‑0's file list with the root-package form; strike Slice 3's "applications do not compile"; write the `tui-next` temporary-name plan and its five recorded risks into Appendix B.1 and `REFACTORING_STATE.md`. **M30** add the legacy test command to the Slice 3/4 gates.
32. **M12** adjudicate and record jackin's request bus (`Jx`) and the `Msg` channel in §3.4 / §17.0 A1.
33. **M19** §0 goal-§9 traceability table. **M20** §23 scenario table. **M31** per-component rustdoc template + check. **A15**, **A16**, **A17** three new checks. Add §20.10 item 15 (focus-ring composition changes).
34. Add `xtask doc-check` to the Slice 3 deliverables and to every slice gate.

Items 1–10 are the hard gate: **Slice 3 must not start until Group 1 and Group 2 are recorded.** Groups 3–6 must land before the Slice 3 owner reaches the corresponding subsystem, and all of them before Slice 4 begins.

### (c) Executable acceptance conditions for the Slice 2 prototype

Goal §27 Slice 2 requires a *representative prototype*, not the foundations. The prototype exists to prove the corrected API before Slice 3 commits to it, and it must be discarded or absorbed, not extended.

**Components in scope (goal §27 Slice 2's list, minimum viable form):**
`Button`, `Field` + `TextInput`, `List`, `Tabs`, `Dialog` (as layer content), one scrollable (`ScrollRegion` over a `List`), plus the runtime slice needed to drive them: `Id`/`ItemKey`/`Part`/`PartRef`, `Response`/`Flow`/`Invalidate`/`StateFlags`, `Intent`/`Cx`/`Ui`, `Registry`/`FocusRing`/`FocusState`, the layer stack, `Surface`, and enough of `Theme` for `junie()` + one partial override + `Theme::paper()`'s surfaces and accent.

**Explicitly out of scope:** `Grid`, `CodeEditor`, `TextViewport`, `Form`, `Wizard`, `PickerChain`, capability downgrade beyond mono, the perf harness, and every `apps/` migration.

**Example app:** `crates/tui/examples/11_small_app.rs` (the Roster) as the primary, **plus** one migrated showcase surface — the **Buttons page**, chosen because `[F]` it is the surface the existing suite exercises most directly (`app_tests.rs:189-238`: `tab_traversal_is_deterministic_and_wraps`, `disabled_buttons_are_skipped_and_cannot_activate`, `mouse_click_activates_and_keyboard_enter_activates`) and because §18.3 #4's hand-written button state matrix is the concrete case for `.state_override`.

**The prototype is accepted, and Slice 3 foundations are declared stable, when all of the following are literally true:**

1. `cargo build -p <lib> --examples` compiles examples **01, 05, 06, 07, 08, 09, 10, 11, 12 verbatim as written in the corrected §17** — no fragments, every one a complete file with a `main` or a `#[test]`, every `use` list correct. This is the single strongest check in the list: it discharges B1, B2, B3, B8, B10, M1, M6, M8, M9, M11, M24 mechanically.
2. `cargo build -p <lib> --example 12_author_component` with `use junie_tui::author::*;` and **no other `junie_tui` path** in the file (Scenario G, Appendix B.4's standing proof).
3. `cargo test -p <lib> --lib` green, including at minimum: `id::separator_prevents_concatenation_collision`, `id::kind_tag_separates_name_from_item_with_equal_bytes`, `response::bitor_takes_consumed_over_ignored`, `response::bitor_takes_max_invalidate`, `focus::trap_is_armed_when_the_layer_is_pushed_not_when_it_draws`, `focus::reconcile_prefers_nearest_surviving_entry_by_previous_index`, `hit::hit_returns_part_ref_not_a_derived_id`, `hit::hit_returns_a_lower_layer_region_for_the_outside_click_test`, `layer::esc_dismisses_only_the_top_layer`, `layer::esc_reaches_the_focused_editor_before_the_layer`, `layer::layer_id_is_assigned_at_open_not_at_draw`, `scroll::wheel_at_the_boundary_is_consumed_without_repaint`, `theme::raise_is_ladder_index_arithmetic_not_colour_equality`, `theme::precedence_family_then_variant_then_state_then_global_then_scope_then_instance`, `collection::reconcile_runs_before_any_action_is_emitted`, `input::cancel_restores_the_snapshot`, `tabs::close_targets_the_logical_tab_after_a_reorder`.
4. `cargo test -p <lib> --test conformance` green with `conformance_suite!(ButtonCase, TextInputCase, FieldCase, ListCase, TabsCase, DialogCase, ScrollRegionCase)` and **all 20 cases** running for each (capability-gated where applicable), including the four that the review found broken by construction: `disabled_cannot_activate`, `bindings_match_handled_keys` (with `Caps::TYPES` on `TextInputCase`), `mono_states_are_distinguishable` (with the corrected `PRESSED` rule), `item_identity_survives_reorder` on `ListCase` and `TabsCase`.
5. `cargo test -p <lib> --test render` green for `render::components::{button,text_input,field,list,tabs,dialog}::{default,focused,hovered,pressed,disabled,selected,editing,empty}` × `{junie, paper}` × `{truecolor, mono}` at `120×40` and `40×10`, with checked-in digests.
6. `render::overrides::{global_family_override_changes_every_button, global_variant_override_changes_only_that_variant, scoped_overlay_changes_only_the_subtree, instance_patch_changes_only_one_instance, part_slot_replaces_the_part_and_keeps_hit_regions}` green — goal §15 scenarios 4, 5, 6, 7, 8 proven before the API is frozen.
7. `render::overlay::{modal_over_page, nested_picker_over_dialog, layer_composites_bottom_to_top_regardless_of_call_order, backdrop_excludes_the_footer}` green — Scenario F and §20.10-2 proven.
8. **The migrated showcase Buttons page passes the three retained tests unmodified in intent**: `tab_traversal_is_deterministic_and_wraps`, `disabled_buttons_are_skipped_and_cannot_activate`, `mouse_click_activates_and_keyboard_enter_activates`, expressed through `Harness` (`h.key`, `h.click_id`, `h.ring().reachable()`, `h.focus()`, `h.resolved(id, Part::GUTTER)`). Any change to the expected focus-ring size is written into `docs/visual-changes.md` §20.10 item 15 **before** the test is edited.
9. `*::no_diagnostics_are_emitted_during_the_journey` green over a scripted Roster + Buttons-page journey: zero `DuplicateId`, zero `CursorRejected`, zero `UndeliveredIntent`, zero `BindingConflict`, zero `FocusTransitionDidNotSettle`.
10. `architecture::{draw_takes_shared_self, no_public_geometry_or_cache, no_raw_background_parameter, no_fn_pointer_extension_points, no_static_bound_in_component_surface, cache_types_are_derived_only}` green over the prototype's files.
11. `RUSTDOCFLAGS="-D warnings" cargo doc -p <lib> --no-deps` green with `#![deny(missing_docs)]`, and `cargo test -p <lib> --doc` green for the condensed forms of examples 1, 5, 6, 7, 8.
12. `xtask doc-check` green: every ```` ```rust ```` block and every `` `Type::method` `` reference in the corrected §3–§17 of `COMPONENT_ARCHITECTURE.md` resolves against the prototype's rustdoc-json, **or** is on an explicit "not yet built (Slice 3/4)" allow-list that the check prints. This is what prevents the §17.0-versus-§3–§15 drift from recurring.
13. `[F]` The legacy tree is untouched and green: `cargo test --all-targets` at the repository root still passes all 198 existing tests (26 showcase, 21 tablepro, 22 jackin + in-module units), and `cargo run --bin showcase` still runs, proving the §7 staging plan.
14. A fresh read-only `opus-analyst` reviews the prototype's **actual public API** — not the document — against §13's conventions and this review's findings, and reports zero unresolved BLOCKER or MAJOR items. Goal §27 Slice 2's "do not continue with an awkward API merely because code has already been written" is discharged here, and only here.

If any of 1, 4, 8, 12 or 13 fails, the correct action is to revise the architecture, not the test.

---

## Files referenced

- `/Users/donbeave/Projects/terminal-components-claude/COMPONENT_ARCHITECTURE.md` (reviewed in full)
- `/Users/donbeave/Projects/terminal-components-claude/REFACTORING_GOAL.md`
- `/Users/donbeave/Projects/terminal-components-claude/Cargo.toml` (single package, three `[[bin]]`, `default-run = "showcase"`, edition 2024, rust-version 1.88)
- `/Users/donbeave/Projects/terminal-components-claude/src/core/id.rs` (confirms §1.2(1): `sub`/`child` are unseparated FNV continuations, `Debug` prints only a hash)
- `/Users/donbeave/Projects/terminal-components-claude/src/bin/showcase/app_tests.rs` (confirms the harness contract facts and the private-field reach)
- `/Users/donbeave/Projects/terminal-components-claude/src/bin/jackin_preview/screens/mod.rs` (confirms the 23-method `Screen` trait and the `Cx { focus, ring, requests }` shape)

**Note for the coordinator:** this review is intended for `docs/reviews/slice2-architecture-review.md`. I am read-only and did not write it; the text above is the file's content verbatim.
