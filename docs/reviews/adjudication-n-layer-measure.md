# Adjudication N — layer sizing (MI‑9) and `Measure` style access (MI‑16)

**Scope.** Two Slice‑3 foundation decisions blocking Slice 4 packages 4A (buttons/measure) and 4F (dialog/overlays). Read against `COMPONENT_ARCHITECTURE.md` §9.1, §9.2, §10, §11.1 A3, §11.3, §16.1, §16.2 cases 14/19, §17.0 A2/A6/A7, §17 examples 9 and 10, §21 items 14/20/30, and the code at the commit after `18afddd`.

**Method.** Facts are `path:line`. Everything under *Decision*, *Rationale*, *Rejected* is inference from those facts.

---

## 0. Collected facts

**Layer sizing.**

1. `LayerSpec.min_size: (u16, u16)`, documented "`(0, 0)` means the whole screen" — `crates/tui/src/layer.rs:158-159`.
2. All three constructors set `min_size: (0, 0)` — `layer.rs:176` (modal), `:193` (popover), `:208` (tooltip).
3. `resolve_anchor` returns the **whole screen** for `(0,0)` — `layer.rs:300-303`.
4. The field is a **maximum, never a minimum**: `let w = size.0.min(screen.width); let h = size.1.min(screen.height);` — `layer.rs:304-305`, then `raw.clamp(screen)` at `:365`. The name `min_size` is false as implemented, and `layer::min_size_then_clamp_then_documented_degradation` (`layer.rs:628-634`) asserts exactly the shrink behaviour.
5. The resolver runs **once per frame, in the runtime, before `app.draw`**: `let rect = resolve_anchor(area, l.spec.anchor, l.spec.min_size);` — `crates/tui/src/runtime.rs:790`, inside the arming loop `:788-806` that also pushes the pooled buffer (`:791`) and the focus scope (`:803-805`).
6. `Ui::layer` sets `self.clip = area` from `LayerDraw.area` and hands that rect to the closure — `crates/tui/src/ui/mod.rs:455-486`, specifically `:470-471`, `:482`.
7. `LayerSpec` is stored at open and is **not mutable afterwards**: `LayerStack::open` returns `None` for an already-open id (`layer.rs:456-458`); `Cx` exposes no spec mutator (`crates/tui/src/ui/cx.rs:244-266`).
8. `Anchor::Rect`'s flip already consumes the size (`layer.rs:327-334`, `:340-347`) — with `(0,0)` it is unreachable. `Anchor::Point` **never flips**; it places at the point and clamps (`layer.rs:358-363` + `:365`), so a menu near the bottom-right slides up and *covers* the pointer.
9. §9.1 fixes the contract: "Placement, flip, clamp and clip are one resolver" (`COMPONENT_ARCHITECTURE.md:662`) and "small terminal … `min_size`, then clamp, then documented degradation" (`:676`). §21 item 20 pins the type as `(u16,u16)` (`:4335`).
10. §16.2 case 14 requires "a layer that cannot draw (0×0) still traps" (`:1738`); case 19 `survives_tiny_rects_0x0_to_3x3` (`:1743`); suite-level `conformance::draw_registers_nothing_when_it_cannot_draw` (`:1754`).

**Measure and styles.**

11. `Measure::measure(&self, ui: &Ui<'_>, c: Constraints) -> Size` — `crates/tui/src/measure.rs:71-74`; declared identically in §10 (`:697`) and in **eleven** places across §17.0 A7 / §21 item 7: `:2287`, `:2302`, `:2311`, `:2323`, `:2361`, `:2414`, `:2428`, `:2443`, `:2624`, `:4241`.
12. `Ui::style(&mut self, …)` and `Ui::style_patched(&mut self, …)` — `ui/mod.rs:217-239`, `:242-265`. `&mut` is required by two things and only two: the memo (`self.core.style_cache.accumulate`, `:224`, `:250`) and the per-cell role recording `dim_layer` reads (`self.roles = …`, `:234-237`, `:260-263`).
13. §17.0 A2 still declares `pub fn style(&self, …) -> Resolved` (`:2120`) — the document is behind the code; D‑13 already accepted `&mut`.
14. **`Theme::resolve(&self, f, v, p, s, surface) -> Resolved` already exists**, `&self`, pure, uncached — `crates/tui/src/theme/mod.rs:188-197`, delegating to `resolve::resolve_uncached`.
15. `FrameRead::{theme, design, state, area, layout}` are `&self` and implemented for `Ui` — `ui/mod.rs:615-635`, trait at `ui/cx.rs:105-117`. `Ui::surface()` is `const fn (&self)` — `ui/mod.rs:197-199`.
16. Glyphs are reachable through `&self` today: `DesignTokens.glyphs: GlyphSet` (`theme/tokens.rs:460`), `GlyphSet::get(&self, GlyphRole) -> &'static str` is `const` (`theme/glyph.rs:168-176`).
17. The overlay stack is private to `UiCore` (`ui/mod.rs:92-97`); `accumulate` takes it as `&[Overlay]` (`theme/resolve.rs:35-82`), and an `Overlay` may set `size` (`StylePatch.size`, §11.3 `:893`).
18. The memo is `Box<[(u64, u32, StylePatch); 256]>`, one allocation at construction, generation-stamped — `theme/resolve.rs:204-237`. §11.1 A3 (`:754`) and §20.9‑2 require it stay statically sized and allocation-free.
19. `bind` uses `surface` only for **colour** binding; `glyph`, `size` and `align` are copied straight off the accumulated patch — `theme/resolve.rs:180-202`.
20. `FrameState.styled_parts: Vec<(Id, Part)>` under `testing`, written by `Ui::note_styled` — `ui/mod.rs:53-54`, `:268-271`; consumed by `conformance::registry::declared_parts_are_the_parts_actually_styled` (`:1751`).

---

## N1 — Layer sizing

### Decision

**Option (c), amended: the size is a typed field on `LayerSpec` supplied by the opener, and the component that owns the layer re-asserts it from `update` every frame through a new `Cx::resize_layer`.** The runtime keeps the single resolver; no measure callback, no two-pass draw, and no component computes a rect.

Three changes: a typed `LayerSize` replacing the `(0,0)` sentinel; two narrow geometry mutators on `Cx`; and `Anchor::Point` flips instead of covering the pointer.

### Exact Rust

**1. `crates/tui/src/layer.rs` — the size becomes a type, not a sentinel.**

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

`Fixed(0, _)` now resolves to `Rect::ZERO`, not to the screen — a zero-size request is an empty layer, which is what case 19 and `draw_registers_nothing_when_it_cannot_draw` assume. The old `(0,0) ⇒ screen` conflation is the whole defect.

**2. `Anchor::Point` flips.** Replace `layer.rs:358-363` with the degenerate-rect form of the `Rect` arm, so a tooltip or context menu near an edge is placed above/left of the pointer instead of sliding over it:

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

**3. Constructor defaults.** `modal`, `popover` and `tooltip` keep `size: LayerSize::Fill`. `Fill` is the honest primitive default (a full-screen modal is a real case); the *component* supplies the size, and `Dialog`/`Select`/`Picker`/`ContextMenu` never open a bare spec (see 5).

**4. `crates/tui/src/ui/cx.rs` — geometry is the one part of a spec that may change while open.**

```rust
impl Cx<'_> {
    /// Update an open layer's requested size. No-op when `id` is not open or
    /// the size is unchanged; the next `draw` re-resolves the anchor. Safe to
    /// call unconditionally every frame — that is the intended use.
    pub fn resize_layer(&mut self, id: Id, size: LayerSize);

    /// Update an open layer's anchor (a popover whose owner moved).
    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor);
}
```

backed by

```rust
// crates/tui/src/layer.rs
impl LayerStack {
    pub(crate) fn spec_mut(&mut self, id: Id) -> Option<&mut LayerSpec> {
        self.open.iter_mut().find(|l| l.id == id).map(|l| &mut l.spec)
    }
}
```

Both set `self.services.repaint = true` only when the value actually changed. Nothing else on `LayerSpec` is mutable after open: `kind`, `inert_below`, `restore_focus` and `initial_focus` are lifecycle facts the runtime armed at open (`runtime.rs:788-806`) and re-deriving them mid-life would desync the focus scope and the inert floor.

**5. `crates/tui/src/runtime.rs:790`** becomes `resolve_anchor(area, l.spec.anchor, l.spec.size)`. Nothing else in the runtime changes. Ordering is already correct: `handle` runs `app.update` (where `resize_layer` lands) and `draw` runs afterwards, so a size asserted in `update` takes effect in the very next draw — the same frame, no flash.

**6. `Ui::layer` — no signature change.** `pub fn layer<R>(&mut self, id: Id, f: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>` stands (`ui/mod.rs:455`, §17.0 A2 `:2131`). One doc sentence is added: *the `Rect` is the resolved layer area and is already the clip; a layer's content lays out inside it and never re-anchors, re-flips or re-clamps.*

**7. `Anchor` — no change.** The enum, `Side`, `CrossAlign` and `ScreenAlign` are all correct; they were simply unreachable while every spec asked for the whole screen.

### How `Dialog::confirm` sets its size

`Dialog` owns its size as a pure function of props + `DesignTokens`, computed identically in both phases — the same rule §15.1 F4 already imposes on field height.

```rust
impl<'a> Dialog<'a> {
    /// Rows the body slot needs. The dialog never sees the body closure before
    /// `draw`, so the caller states it; the convenience constructors set it.
    pub const fn body_rows(mut self, n: u16) -> Self { self.body_rows = n; self }

    /// `.width(w)` when set, else `design.size.dialog_width` (54).
    pub fn measured_width(&self, d: &DesignTokens) -> u16 {
        self.width.unwrap_or(d.size.dialog_width)
    }

    /// border(2) + title(1) + wrapped description + [blank + body] + [blank + actions].
    /// Pure in `(props, DesignTokens)`; `draw` lays out against this number.
    pub fn measured_height(&self, d: &DesignTokens) -> u16 {
        let inner = self.measured_width(d)
            .saturating_sub(2)
            .saturating_sub(d.space.dialog_inset.saturating_mul(2));
        let desc    = self.description.map_or(0, |s| text::wrapped_rows(s, inner));
        let body    = if self.body_rows == 0 { 0 } else { self.body_rows.saturating_add(1) };
        let actions = if self.actions.is_empty() { 0 } else { 2 };
        3u16.saturating_add(desc).saturating_add(body).saturating_add(actions)
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

**Invariant D1 — the dialog re-asserts its size every frame.** `Dialog::update` begins with

```rust
cx.resize_layer(self.id, LayerSize::Fixed(self.measured_width(cx.design()),
                                          self.measured_height(cx.design())));
```

so a description that grows, an error row that appears, or a theme swap corrects the layer on the next draw without the opener predicting anything. `Select`, `Picker`, `ContextMenu` and `MenuBar` do the same with their own arithmetic (`Select`: `popup_min_width ≤ w ≤ popup_max_width` from the item labels it already receives per phase, `h = min(items, popup_max_rows) + 2`).

Example 9's opener becomes `cx.open_layer(CONFIRM, confirm_dialog().layer(cx));`, where `confirm_dialog()` is the single props constructor §13 already requires — so this also satisfies `architecture::props_are_built_once`.

**One new text primitive is required** (Slice 3 owns `text/`): `pub fn wrapped_rows(s: &str, width: u16) -> u16` — a grapheme/word walk returning the row count, 0 allocations, and the same function `Dialog::draw` uses to lay the description out. Without it "pure function of props and tokens" is unverifiable.

### Rationale

- The resolver already runs exactly once per frame, in the runtime, before any drawing (fact 5). Supplying it a real size is the *smallest* change that makes §9.1's "one resolver" true, and it is the only option that touches no phase boundary.
- The size is knowable in `update` for every overlay in the inventory: dialogs from tokens + props, popups from the item slice `update` already receives per phase (§17.0 A3), tooltips from the string. `Cx` already carries `design()` and last frame's `area`/`layout` (fact 15), which is precisely the toolkit §15.1 F4 uses for the identical problem.
- Making the size mutable-while-open (`resize_layer`) is what removes the last reason to want a measure callback: content growth is handled by the component that owns the content, once per frame, in the phase that is allowed to change runtime state.
- Renaming `min_size` → `size` corrects a documentation defect, not just a shape: the field has always been a maximum (fact 4), and every reader who trusted the name would size dialogs wrongly.

### Rejected alternatives

| Rejected | Why |
|---|---|
| **(a) `LayerSize::Measured` + a measure callback on `Ui::layer` or a two-pass draw** | Structurally infeasible. `LayerSpec` is `Copy`, `const`-constructible, and stored in `LayerStack` **across frames** (`layer.rs:395-402`, `:450-468`); a callback that measures real content must borrow app data, which cannot live in runtime state that outlives the frame. The two-pass alternative needs an area for pass 1 — the thing being computed — and doubles a whole draw traversal against §20.9's frame budget. The runtime also cannot construct a `Ui` before the arming loop, because the loop fills `frame.layers` which `Ui` borrows mutably (`runtime.rs:788-816`). |
| **(b) `layout::place(anchor, size, screen)` called by each overlay's content** | Puts flip and clamp back into every component — literally the `ui/popup.rs:25-56` + `menu.rs:143-171` duplication §9.1 was written to delete. It also makes `LayerDraw.area` permanently the whole screen, so `Ui::layer`'s clip stops bounding anything, `draw_stays_inside_its_area` (case 8) and `survives_tiny_rects` (case 19) become per-component obligations again, and the §9.1 table row "small terminal: `min_size`, then clamp, then documented degradation" has no owner. |
| **`LayerSize` keeping true minimum semantics (`Min(w,h)` growing to fit)** | Nothing can grow a layer without measuring content, which is (a). A minimum that is silently a maximum is the present bug. |
| **`(u16, u16)` with `(0,0)` retained and `LayerSpec::modal` defaulting to `Fixed(dialog_width, 0)`** | A second sentinel (`h == 0` meaning "unknown") in the field whose first sentinel caused this adjudication; and `modal()` is `const` and cannot read `design.size.dialog_width`. |
| **`Cx::update_layer(id, impl FnOnce(LayerSpec) -> LayerSpec)`** | Lets a caller flip `kind`/`inert_below`/`initial_focus` mid-life, desyncing the focus scope mode and inert floor the runtime armed at open (§21 item 14's "armed when the layer is pushed, not when it draws"). Two narrow geometry setters carry the same ergonomics with none of that. |

### Tests

Renamed / rewritten in `crates/tui/src/layer.rs` (and in §16.1's `layer.rs` list, so `every_named_test_exists` stays honest):

- `layer::fill_resolves_to_the_whole_screen` — replaces the `(0,0)` arm of `anchor_screen_center_sits_in_the_upper_third` (`layer.rs:621-624`).
- `layer::fixed_size_is_clamped_never_grown` — renames `min_size_then_clamp_then_documented_degradation` (`layer.rs:627-634`); asserts `Fixed(54,20)` on a 40×10 screen equals the screen, and `Fixed(0, 8)` equals `Rect::ZERO`.
- `layer::popover_flips_above_when_the_content_does_not_fit_below` — the `Anchor::Rect` flip, now reachable.
- `layer::point_anchor_flips_instead_of_covering_the_pointer` — new, pins change 2.
- `layer::resize_layer_re_resolves_the_anchor_on_the_next_draw` — open at `Fixed(20,4)`, `resize_layer` to `Fixed(40,10)` in `update`, assert the drawn area in the **same** frame.
- `layer::spec_geometry_is_the_only_mutable_field` — compile-level: `Cx` exposes no other spec mutator.

New in `components/dialog.rs` (4F):

- `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` — same props + two themes ⇒ two deterministic sizes; same props twice ⇒ identical.
- `dialog::draw_lays_out_against_the_height_it_asked_for` — the rect `draw` returns equals the layer area; no `Rect::centered*` call appears in `dialog.rs` (xtask forbidden pattern, scoped to `components/`).
- `dialog::confirm_is_centred_by_the_resolver_not_by_the_dialog`.
- `dialog::a_growing_body_resizes_the_layer_on_the_next_frame`.

Conformance (`crates/tui-testing/src/conformance/driver.rs`):

- Case 19 `survives_tiny_rects_0x0_to_3x3` — for `Caps::OVERLAY` the driver opens the layer through the component's own spec at each of the 16 screens and asserts `area ⊆ screen`, no registration outside, no panic.
- Case 14 `focus_trap_and_restore` — unchanged; still passes because arming precedes drawing (`runtime.rs:788-806`).
- `conformance::draw_registers_nothing_when_it_cannot_draw` — extended to `LayerSize::Fixed(0, h)`.

---

## N2 — `Measure` cannot resolve styles

### Decision

**Option (a).** `Measure::measure(&self, ui: &Ui<'_>, c: Constraints) -> Size` stands unchanged. `Ui` gains a `&self` resolution path that bypasses the memo and records nothing; `Theme` gains the surface-independent half for `update`-phase sizing. `Ui::style`/`style_patched` keep `&mut self` and remain the **only** queries that record roles or styled parts.

### Exact Rust

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

`UiCore.overlays` becomes `pub(crate)` (it is already `pub(crate)` on `style_cache`; `overlays` at `ui/mod.rs:95` is private to the module and `Ui` lives in the same module, so no visibility change is actually required).

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
    /// height (§15.1 F4) and `Dialog::layer` (Adjudication N1).
    pub fn metrics(&self, f: Family, v: Variant, p: Part, s: StateFlags) -> PartMetrics;

    /// Unchanged; already present at `theme/mod.rs:188-197`.
    pub fn resolve(&self, f: Family, v: Variant, p: Part,
                   s: StateFlags, surface: Surface) -> Resolved;
}
```

`Theme::resolve` is refactored to `metrics` + colour binding so there is exactly one `accumulate` path; `resolve::bind` (`theme/resolve.rs:180-202`) already separates the two concerns (fact 19), so this is a factoring, not new logic.

**Worked example — the case that forced this (4A, button with a leading glyph):**

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

### Effect on recorded roles and `styled_parts`

**Invariant M1.** `Ui::style` and `Ui::style_patched` are the *painting* queries: they alone write `self.roles` (`ui/mod.rs:234-237`, `:260-263`) and alone feed the `testing` record. `Ui::resolve`, `Theme::resolve` and `Theme::metrics` record **nothing**.

This is required, not incidental: `conformance::registry::declared_parts_are_the_parts_actually_styled` (`:1751`) compares recorded parts against `Self::PARTS`. A component that *measures* a part it never paints (common: measuring `Part::META` to decide it does not fit) would otherwise report a part it never styled, and the check would pass on a false positive. Symmetrically, a component that resolves through `Ui::resolve` and then paints must go through `Ui::style` for the paint — which it does anyway, because painting needs the roles set for `dim_layer` (§11.6).

BL‑7's widening of `styled_parts` to `(Id, Family, Variant, Part, Resolved)` lands on the painting path only and is unaffected by this decision.

### Cache and allocation obligations (§20.9‑2)

- The memo stays `Box<[(u64, u32, StylePatch); 256]>` (`theme/resolve.rs:212`), one allocation at construction, generation-stamped, unchanged.
- `Ui::resolve` performs **zero** cache reads and writes and **zero** allocations, so `style_resolve_10k_parts`' 0-alloc assertion and the ≥ 90 % hit-rate assertion added by adjudication 2.8 are both unperturbed by measurement.
- Measurement is O(components) per frame, not O(cells) — the reason the memo exists (rows re-resolving one tuple) does not apply to it. Routing measurement through the cache would cost hit rate on the path that actually needs it.

### Rejected alternatives

| Rejected | Why |
|---|---|
| **(b) `measure(&self, ui: &mut Ui<'_>, c)`** | (i) Rewrites **eleven** already-declared signatures (`:697`, `:2287`, `:2302`, `:2311`, `:2323`, `:2361`, `:2414`, `:2428`, `:2443`, `:2624`, `:4241`) for one capability the `&self` path already provides. (ii) It hands `measure` the whole painting and registration surface, destroying the property `layout::rows_measured` depends on — that measuring has no frame side effects. `draw_twice_is_byte_identical` (case 5) would then depend on how many times a layout primitive measured. (iii) It lets measurement thrash the 256-slot memo. (iv) `FieldControl::measure` (`:2311`) is called from `Field::measure` while the caller holds `&mut Ui` for the surrounding paint; §17 example 9's `layout::rows_measured(body, &tracks, &[props.measure(ui, c).preferred.1])` shape becomes borrow-fragile. |
| **(c) `measure(&self, theme: &Theme, design: &DesignTokens, c)`** | Cannot see `Surface`, so it cannot resolve a `Resolved` at all; cannot see the **overlay stack** (fact 17), so a scoped `Overlay` that sets `.size(n)` would be honoured by painting and ignored by measurement — the component measures 10 and paints 12, with no test able to see it; and cannot see `FrameRead::{state, area, layout}`, which `Form::measure` (F4) and `List::measure` need. It also duplicates two parameters `Ui` already carries, for a doc-wide rewrite. |
| **Interior mutability on `StyleCache` so `Ui::style(&self)` works** | Makes the memo's hit/miss statistics — the mechanism adjudication 2.8 promoted to the binding assertion — observable from a `&self` context, and re-opens the "is `draw` pure" question §5 R2 closed. The roles write is genuinely `&mut` state and cannot be laundered. |

### Tests

New in `crates/tui/src/measure.rs` and `ui/mod.rs` (added to §16.1's `layout.rs / measure.rs` and `ui/paint.rs` lists):

- `measure::ui_resolve_equals_ui_style_for_every_family_variant_part` — differential over the built-in recipe set × both themes × a state sweep, with and without a pushed `Overlay`: `ui.resolve(..) == ui.style(..)` field-for-field. This is the test that keeps a second resolution path from drifting.
- `measure::measure_records_no_roles_and_no_styled_parts` — 1 000 `Ui::resolve` calls leave `roles_at(pos)` untouched and `styled_parts()` empty.
- `measure::measure_does_not_touch_the_style_cache` — `StyleCache::stats()` (promoted to `testing` by adjudication 2.8) is identical before and after 1 000 measures.
- `measure::natural_width_follows_the_themed_glyph` — a theme that rebinds `GlyphRole::FocusBar` to a 2-cell glyph widens `Button::measure` by exactly one column; the same button under `Theme::junie()` does not.
- `measure::measure_is_allocation_free` (perf) — 10 000 `Button::measure` calls record 0 allocations.
- `theme::metrics_are_surface_independent` — `theme.metrics(..) == PartMetrics::from(theme.resolve(.., s))` for every `Surface`.
- `theme::metrics_is_the_sizing_path_for_update` — `Form`'s field height and `Dialog::measured_height` computed from `Cx` equal the values `draw` lays out against.

---

## Confirmations requested by the review

### `Ui::with_part(...)` — **CONFIRMED**, with one amendment

The review's §4(k)‑2 complaint is real: `Ui::style` is `&mut self`, so `ui.fill(cell, ui.style(...).style)` is a borrow error and every component must bind a temporary. Exact signature:

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

Amendment: the name must not imply a pushed scope, so the doc says so explicitly, and it is a **convenience, not a replacement** — `style_patched` (precedence 6) has no `with_` form and components with a per-instance patch keep the two-step shape. Test: `ui::with_part_resolves_once_and_records_the_role` (assert one cache miss, roles set at the painted cells).

### `Ui::surface_style() -> Style` — **CONFIRMED**, plus one amendment that makes it hard to forget

```rust
impl Ui<'_> {
    /// The style a child inherits from the current surface: `bg` is
    /// `theme.bg(ui.surface())`, `fg` is `Role::Fg(FgStep::Primary)` bound on
    /// that surface, no modifiers. The **left** operand of §11.3's final
    /// layering: `ui.surface_style().patch(resolved.style)`.
    pub fn surface_style(&self) -> Style;
}
```

**Amendment.** §22.2 item 10 requires `inherited.patch(resolved.style)` at every paint, and the review found **no production call site performs it** (§4(f)). Leaving it as a two-noun idiom guarantees half the Slice‑4 components will forget. Add the fused form on `Resolved`, which is the shape a component actually wants:

```rust
impl Resolved {
    /// This part's style layered over an inherited one — §11.3's final step,
    /// `Style::patch` semantics (modifier symmetry, §22 R‑9).
    #[must_use]
    pub fn over(self, inherited: Style) -> Style { inherited.patch(self.style) }
}
```

so the call site is `ui.fill(area, r.over(ui.surface_style()))`. Tests: `ui::surface_style_is_the_left_operand_of_the_final_patch` (differential against a hand-written `inherited.patch(..)` over every surface × both themes) and `theme::patch_merge_matches_ratatui_style_patch_for_modifiers` extended to route through `Resolved::over`.

### `Ui::scroll_region(...)` (review §4(h), second gap)

Out of scope for this adjudication. It is a Slice‑3-owned `Ui` method blocking 4E, not 4A or 4F; record it as an open item, not as decided here.

---

## Document amendments (binding; mirror in `REFACTORING_STATE.md` per the change-control rule, `COMPONENT_ARCHITECTURE.md:3`)

| Section | Change |
|---|---|
| §9.1 (`:636-646`) | `min_size: (u16, u16)` → `size: LayerSize`; declare `LayerSize { Fill, Fixed(u16,u16) }`; add the sentence *"the size is the opener's, the placement is the runtime's; a component computes a size, never a rect."* |
| §9.1 (`:662`) | After "one resolver", add: *"the resolver is given a size; it clamps and never grows. A layer whose content changes size re-asserts it from `update` with `Cx::resize_layer`."* |
| §9.1 table (`:676`) | "small terminal" row: `min_size` → `LayerSize::Fixed`, then clamp, then documented degradation. |
| §9.1 (`:682`, item 14 para) | Add: *the spec is fixed at open except its geometry (`size`, `anchor`), which `Cx::resize_layer` / `Cx::reanchor_layer` may change; `kind`, `inert_below`, `restore_focus` and `initial_focus` are armed at open and are immutable.* |
| §9.2 (`:686`) | Add: *`Dialog` sizes its own layer (`Dialog::layer(cx)`, `body_rows`, `measured_width`/`measured_height`) as a pure function of props and `DesignTokens`, and re-asserts it every `update`.* |
| §10 (`:697`) | Keep `Measure` verbatim; append the D‑13 consequence: *`measure` is `&Ui`; it resolves through `Ui::resolve` (uncached, no roles recorded), never `Ui::style`. Update-phase sizing uses `Theme::metrics`.* |
| §11.1 A3 (`:754`) | Drop `Surface` from the memo key (already carried by adjudication §4(f)); add *the memo serves the painting path only; `Ui::resolve` bypasses it so measurement cannot evict painting entries.* |
| §11.3 (`:912`) | `Resolved` gains `align` (D‑5) and `pub fn over(self, inherited: Style) -> Style`. |
| §17.0 A2 (`:2120`) | `style`/`style_patched` are `&mut self` (D‑13); add `resolve(&self, …)`, `glyph_str(&self, …)`, `surface_style(&self)`, `with_part(…)`, `register_editor`, `declare_state` (D‑6). |
| §17.0 A6 (`:2249-2261`) | `min_size: (u16,u16)` → `size: LayerSize`; `.min_size(w,h)` → `.size(LayerSize)`; declare `LayerSize`. |
| §17.0 A2 `Cx` (`:2105-2109`) | Add `resize_layer`, `reanchor_layer`. |
| §17.0 A7 Dialog (`:2457-2477`) | Add `body_rows`, `measured_width`, `measured_height`, `layer`. |
| §17 example 9 (`:2949`, `:2976-2987`) | `open` becomes `cx.open_layer(CONFIRM, confirm_dialog().layer(cx))`; keep adjudication 2.7's `rows_measured` rewrite of `:2980`. |
| §17 example 10 (`:3022`, `:3036-3039`) | `LayerSpec::modal(DLG)` → `dialog().layer(cx)`; the popover gains `.size(LayerSize::Fixed(w, h))` from the picker's own arithmetic. |
| §21 item 20 (`:4335`) | *`LayerSpec.min_size` is `(u16,u16)`, not `Size`* → *`LayerSpec.size` is `LayerSize`, a two-variant enum; the earlier `(u16,u16)` with a `(0,0)` sentinel is struck (Adjudication N1). It was never a minimum: the resolver clamps down and never grows.* |
| §16.1 `layer.rs` (`:1615`) | `min_size_then_clamp_then_documented_degradation` → `fixed_size_is_clamped_never_grown`; add `fill_resolves_to_the_whole_screen`, `popover_flips_above_when_the_content_does_not_fit_below`, `point_anchor_flips_instead_of_covering_the_pointer`, `resize_layer_re_resolves_the_anchor_on_the_next_draw`. |
| §16.1 `layout.rs / measure.rs` (`:1621`) | Add the six `measure::` / two `theme::metrics` names above. |
| §16.2 case 19 (`:1743`) | Append: *for `Caps::OVERLAY`, the driver opens the layer through the component's own `LayerSpec` at each tiny screen.* |
| §11.5 (`:975`) | Add: *layer geometry → `LayerSpec.size` (the component) + `resolve_anchor` (the runtime).* |

---

## Risks

1. **`LayerSize` is `#[non_exhaustive]`, so a future `Measured`/`Content` variant is additive** — but adding one still requires solving the callback-lifetime problem above. Recorded so a later slice does not re-litigate N1 by adding a variant it cannot implement.
2. **`Dialog::measured_height` and `Dialog::draw` can drift.** Mitigated by `dialog::draw_lays_out_against_the_height_it_asked_for` and by `text::wrapped_rows` being the single wrap used by both. This is the same risk §15.1 F4 already carries for field height, and the same mitigation.
3. **`Ui::resolve` and `Ui::style` can drift.** Mitigated by `measure::ui_resolve_equals_ui_style_for_every_family_variant_part`, which is a full differential, not a spot check.
4. **`resize_layer` called every frame is a per-frame write.** It is a compare-and-store on a `Copy` field with no allocation and no repaint unless the value changed; `frame_showcase_lists_120x40`'s allocation budget is unaffected.
5. **Uncached measurement cost.** ~13 ns per part × the handful of parts a component measures × the components measured per frame — orders below the ≤ 5 % per-frame style budget adjudication 2.8 sets. Reported by `measure::measure_is_allocation_free`'s companion ns line, asserted only under `PERF_STRICT=1`.
6. **`Anchor::Point` flipping is a visual change** for any existing tooltip/context menu near a screen edge. Classify in `docs/visual-changes.md` against §20.10 before blessing any baseline that moves.

---

## Acceptance conditions (executable, from the workspace root)

```bash
# N1 — the resolver is given a real size, and it clamps rather than grows
cargo test -p tui-next --lib layer::tests::fill_resolves_to_the_whole_screen
cargo test -p tui-next --lib layer::tests::fixed_size_is_clamped_never_grown
cargo test -p tui-next --lib layer::tests::point_anchor_flips_instead_of_covering_the_pointer
cargo test -p tui-next --lib layer::tests::popover_flips_above_when_the_content_does_not_fit_below
cargo test -p tui-next --lib layer::runtime_tests::resize_layer_re_resolves_the_anchor_on_the_next_draw
! rg -n 'min_size' crates/tui/src COMPONENT_ARCHITECTURE.md      # the false name is gone everywhere

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

**Gate.** Every command exits 0; the §9.1 / §10 / §11.1 A3 / §11.3 / §11.5 / §16.1 / §16.2 / §17.0 A2·A6·A7 / §17 examples 9–10 / §21 item 20 amendments above are applied and mirrored in `REFACTORING_STATE.md`. On that condition **4A may start** (N2 unblocks `Button::measure`) and **4F may be scheduled** (N1 unblocks `Dialog`, `Select`'s popup, `Picker`, `ContextMenu`).
