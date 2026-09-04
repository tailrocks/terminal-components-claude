# Authoring a component

This guide is for writing a **new component** — one that participates in
theme resolution, focus, hover, press, event dispatch, hit testing, cursor
output, scrolling and overlays exactly like a library component, from a
downstream crate, with no private access.

The worked example is `crates/tui/examples/12_author_component.rs`: a
`Segmented` control, N labelled segments, one selected, a roving cursor. Read
it alongside this guide. Everything below is either quoted from it or
verified against the compiler.

Written against `junie_tui`; see the note at the top of
[`quickstart.md`](quickstart.md) about the temporary crate name.

---

## What `junie_tui::author` gives you

One import line, and nothing you should not have:

```rust
use junie_tui::author::{
    Binding, BindingState, Bindings, Chord, Cx, Family, Focusability, FrameRead, GlyphRole, Id,
    Intent, ItemKey, KeyCode, Part, PartRef, Phase, Rect, Response, Slot, StateFlags, Ui, Variant,
};
```

| Group | Items |
|---|---|
| identity | `Id`, `ItemKey`, `Part`, `PartRef` (and the `id!` macro, at the crate root) |
| input | `Input`, `Key`, `KeyCode`, `KeyModifiers`, `Chord`, `Mouse`, `MouseKind`, `Axis` |
| dispatch | `Intent`, `IntentIter`, `Phase`, `FocusVia` |
| replies | `Response`, `Flow`, `Invalidate`, `StateFlags`, `Activated` |
| the two phases | `Cx`, `Ui`, `FrameRead`, `LayoutFacts` |
| registration | `Focusability`, `ScopeId`, `ScopeMode`, `FocusVis`, `RegionKind`, `Hit`, `Axes`, `Headroom`, `Capture`, `ScrollState` |
| layers | `LayerSpec`, `LayerId`, `LayerKind`, `LayerSize`, `Anchor`, `Side`, `CrossAlign`, `ScreenAlign`, `Backdrop`, `Dismiss`, `DismissReason`, `LayerEvent`, `resolve_anchor`, `backdrop_area` |
| theme | `Theme`, `Family`, `Variant`, `Role`, `FgStep`, `Surface`, `GlyphRole`, `MeterRole`, `SyntaxRole`, `Align`, `Slot`, `StylePatch`, `StateRule`, `Resolved`, `Overlay`, `OverlayRule`, `DesignTokens`, `Density`, `ColorLevel`, `Modifier`, `border` |
| layout & measurement | `layout` (the module), `Track`, `Insets`, `RowAlign`, `SplitModel`, `Constraints`, `Size`, `Measure` |
| text | `width`, `wrap`, `wrapped_rows`, `truncate`, `truncate_middle`, `fuzzy`, `Span`, `TextBuffer`, `TextEditorCore`, `EditAction`, `EditOutcome`, `CursorPos`, `Motion`, `Extend` |
| collections | `RowUi`, `CellUi`, `ColumnsUi`, `CollectionCore`, `Reconcile`, `Reconciliation`, `KeyFn`, `RowFn`, `KeySet`, `SelectMode`, `EmptyState`, `Status`, `RowDecor`, `CellDecor`, `DefaultRow`, `ByIndex`, `RowTotal` |
| keymaps & hints | `Binding`, `Bindings`, `BindingState`, `KeyMap`, `KeyPhase`, `Hint`, `HintLayer`, `binding_conflicts`, `Action`, `ActionKey` |
| fields | `FieldControl`, `Validate`, `NoValidate`, `FieldError`, `Secret`, `SecretPolicy` |
| painting | `Buffer`, `Cell`, `Position`, `Rect`, `Color`, `Style`, and `author::raw::{Line, Span, Text}` |

Deliberately **absent**: `Runtime`, `run`, `TerminalSession`, `DefaultTerminal`,
`App`, `Registry`, `FocusRing`, `FocusState` and every concrete component. A
component author drives none of those; if you need one, you are writing an
application, not a component.

`author::raw` is the escape hatch for `Ui::raw()` / `RowUi::raw()`. ratatui's
own style-carrying span is reachable only as `raw::Span`, always written
qualified, so it can never be confused with `junie_tui::Span` (which carries a
`Role`, not a `Style`).

---

## The required shape

A component is five things:

1. a **props struct** with a lifetime, built once per frame and used by both
   phases;
2. a caller-owned **`XState`** holding *durable interaction state only*;
3. **`update(&self, cx, &mut state[, data]) -> Response<XAction>`**;
4. **`draw(&self, ui, area, &state[, data]) -> Rect`**, and usually
   `measure(&self, ui, Constraints) -> Size`;
5. **`X::PARTS`** and an `impl Bindings`.

### Props are built once

The convention every component in the tree follows: one small constructor
function that takes **no `&self`**, so `update` can still hand `&mut self.field`
to it. From `crates/tui/examples/07_borrowed_rows.rs`:

```rust
/// Configuration and closures only — the rows are passed to each phase call,
/// so the props never borrow `self.orders` and the action closure is free to
/// mutate `self`.
fn orders_list() -> List<'static, Order, impl Fn(&Order) -> ItemKey, impl Fn(&Order, &mut RowUi<'_>)>
{
    List::new(ORDERS)
        .key(|o: &Order| ItemKey::num(o.id))
        .row(order_row)
        .select_mode(SelectMode::Single)
        .empty(EmptyState::Empty { title: "No orders", hint: Some("Adjust the filter") })
}
```

Both phases call `orders_list()`. They cannot disagree about the
configuration, because there is only one place it is written. The
architecture check `architecture::props_are_built_once` enforces this shape
for library components; adopt it for yours.

Data — the items, the value being edited — is passed **per phase call**, not
stored in the props. That is what keeps the borrow of `self.orders` from
outliving `update` and blocking the action closure from mutating `self`.

Builders consume `self` and are `#[must_use]`:

```rust
#[must_use]
pub const fn variant(mut self, v: Variant) -> Self {
    self.variant = v;
    self
}
```

### `XState` holds interaction state, not values

```rust
/// Durable interaction state: the roving cursor and the chosen segment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SegmentedState {
    pub cursor: usize,
    pub selected: usize,
}
```

`Default`, `Clone`, `PartialEq` and `Debug` are required by the conformance
suite. Keep it small and make it obvious that it is not the model: a
`TextInputState` holds the in-flight edit and the error, never the `String`.

### `update`

```rust
pub fn update(&self, cx: &mut Cx<'_>, st: &mut SegmentedState) -> Response<SegmentedAction> {
    let mut r = Response::ignored();
    let n = self.labels.len();
    if n == 0 {
        return r.for_id(self.id);
    }
    for it in cx.intents(self.id) {
        match it {
            Intent::Key(k) => match Binding::lookup(BINDINGS, &k) {
                Some(SegCmd::Prev) => { /* … */ }
                Some(SegCmd::Next) => { /* … */ }
                Some(SegCmd::Select) => { /* … */ }
                None => {}
            },
            Intent::Pointer {
                phase: Phase::Click,
                part: PartRef { part, item: Some(k) },
                ..
            } if part == SEGMENT => { /* … */ }
            _ => {}
        }
    }
    r.for_id(self.id)
}
```

Points that matter:

- **`cx.intents(id)` is the only input.** There is no `on_key`, no
  `on_click`, no mouse coordinate inspection and no `owns`/`locate`. The
  runtime has already decided which component and which *part* an event
  belongs to. An empty queue costs one `bool` check.
- `cx.intents` borrows only the frozen queue, so `cx`'s other services stay
  usable inside the loop — you can `cx.request_repaint()` or
  `cx.open_layer(…)` while iterating.
- **Always finish with `.for_id(self.id)`.** The runtime uses it for
  invalidation bookkeeping.
- **`Response` is `#[must_use]`.** Fold several with `|=`.

The `Intent` variants are `Key(Key)`, `Paste(&str)`, `Pointer { phase, part,
pos, local, mods }`, `Wheel { axis, delta, part, pos }`, `FocusIn { via }`,
`FocusOut { to }`, `Layer(LayerEvent)` and `Cancel`. `Phase` is `Press`,
`Release`, `Click`, `DoubleClick`, `Secondary`, and the drag phases.

### `draw`

```rust
pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SegmentedState) -> Rect {
    if area.is_empty() {
        return area; // registers nothing
    }
    ui.register_control(self.id, area, Focusability::Focusable);
    let live = ui.state(self.id);
    // … resolve, paint, register parts …
    area
}
```

`draw` takes `&self` and `&State`. Committing an edit, moving a selection or
closing a layer from `draw` is a compile error, which is how eight
render-time semantic transitions in the legacy widget set were made
unrepresentable rather than merely forbidden.

`draw` returns the `Rect` it actually painted, so a caller can lay out around
it.

**A degenerate rect registers nothing.** `0×0`, `1×1`, `3×3` — return early
before registering, and never underflow. The conformance case
`survives_tiny_rects_0x0_to_3x3` renders your component into every rect from
`0×0` to `3×3` and asserts no panic, and the suite-level
`draw_registers_nothing_when_it_cannot_draw` in
`crates/tui/tests/conformance.rs` asserts the registry stays empty. Use
`saturating_add` / `saturating_sub` throughout;
`clippy::arithmetic_side_effects` is on.

### `measure`

```rust
pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size { … }
```

`Size` is `{ min: (u16, u16), preferred: (u16, u16) }`; `Constraints` is
`{ max: (u16, u16), tight_w: bool, tight_h: bool }` with `loose(w, h)` and
`tight(w, h)` constructors, and `Size::fit(c)` clips both pairs.

`measure` takes `&Ui`, so use `Ui::resolve` (the uncached `&self` path) rather
than `Ui::style` (which is `&mut self`, memoised, and records the per-cell
roles the dim pass reads). A measurement must not evict a painting entry from
the 256-slot style memo.

> As of Slice 4, the `Measure` **trait** is in the public surface but no
> shipped component implements it; every component exposes an *inherent*
> `measure` of the same signature. Follow the inherent-method convention
> unless you specifically need trait dispatch.

### `PARTS` and `Bindings`

```rust
impl<'a> Segmented<'a> {
    /// The parts this control styles.
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, SEGMENT, Part::LABEL];
}

const SEGMENT: Part = Part::custom("segment");
const F_SEGMENTED: Family = Family::custom("segmented");

impl Bindings for Segmented<'_> {
    type Cmd = SegCmd;
    fn bindings(&self, _s: BindingState) -> &'static [Binding<SegCmd>] {
        BINDINGS
    }
}

const BINDINGS: &[Binding<SegCmd>] = &[
    Binding { chord: Chord::key(KeyCode::Left),  cmd: SegCmd::Prev,   label: "Prev",   priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Right), cmd: SegCmd::Next,   label: "Next",   priority: 40, visible: true },
    Binding { chord: Chord::key(KeyCode::Enter), cmd: SegCmd::Select, label: "Select", priority: 80, visible: true },
    Binding { chord: Chord::key(KeyCode::Char(' ')), cmd: SegCmd::Select, label: "Select", priority: 80, visible: false },
];
```

`Cmd` is a `Copy`, `const`-constructible command type. It is **not** the
action: `update` turns a `Cmd` into an `XAction` carrying the live key, so the
binding table can stay `&'static` while actions carry runtime data.

`label`, `priority` and `visible` are what the shared `HintBar` reads. The
same table is the one `update` matches against, which is the point: the hint
bar cannot show a key the component does not handle, and the conformance case
`bindings_match_handled_keys` asserts it in both directions. Give one chord
per command `visible: true` and hide the aliases.

`binding_conflicts(owner, phase, table)` reports two *visible* bindings on one
chord as a `Diagnostic`.

### The rustdoc template

Every public component carries this exact section set, in this order
(`COMPONENT_ARCHITECTURE.md` §13.2). `crates/tui/src/components/button.rs` is
the reference implementation:

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
## Layout          — the `measure` contract, degenerate-rect behaviour, what `draw` returns
## Parts           — `X::PARTS` with one line each
## Overrides       — `.patch`, `.patch_part`, `.slot` support and any part that cannot be replaced
## Identity        — how items are keyed; the `ByIndex` caveat where applicable
## Testing         — the `Conformance` case name and `Caps`; the `render::components::<x>` states
## Invariants      — component-specific invariants beyond the shared state/render rules
```

---

## Participating in theme resolution

Name a `Family`, ask for each part, paint what comes back.

```rust
let r = ui.style(F_SEGMENTED, self.variant, SEGMENT, s);
ui.fill(cell, r.over(ui.surface_style()));
match r.glyph {
    Slot::Set(g) => ui.glyph(cell, g, r.style),
    Slot::Inherit if s.contains(StateFlags::SELECTED) => {
        ui.glyph(cell, GlyphRole::Chosen, r.style)
    }
    Slot::Inherit | Slot::Clear => {}
}
let ls = ui.style(F_SEGMENTED, self.variant, Part::LABEL, s).style;
ui.paint_str(cell, label, ls);
```

`Resolved` is `{ style: Style, glyph: Slot<GlyphRole>, size: Option<u16>,
align: Option<Align> }`. Three rules:

1. **Layer over the inherited surface style**, don't replace it:
   `resolved.over(ui.surface_style())` — inherited on the left, the part's
   style on the right.
2. **Honour all three `Slot` states of `glyph`.** `Set(g)` paints `g`,
   `Inherit` leaves your fallback in control, `Clear` suppresses it. The mono
   `DISABLED` rule *clears* the marker glyph; a component that treats
   `Resolved.glyph` as an `Option` cannot express that.
3. **Declare what you style.** Every `(owner, part)` pair you resolve should
   be in `X::PARTS` — that is the contract `.patch_part` and every
   `override_*` call rely on.

   The mechanical check for this is `registry::check` in
   `crates/tui/tests/conformance.rs`. It is **not** generated by
   `conformance_suite!`, and it reads `Ui::styled_parts()`, which is
   populated by `Ui::note_styled` — a `#[cfg(feature = "testing")]` method
   the library components call through an internal helper. A downstream
   component that calls `ui.style(…)` directly records nothing, so to get the
   same check you must call `ui.note_styled(owner, family, variant, part,
   resolved)` yourself under `#[cfg(feature = "testing")]`, and write the
   `PARTS ⊇ styled` assertion in your own test. Both `Ui::note_styled` and
   `Ui::styled_parts` are public under that feature.

Resolution methods:

| Method | Phase | Notes |
|---|---|---|
| `Ui::style(f, v, p, flags)` | `draw` | memoised; records per-cell roles |
| `Ui::style_patched(f, v, p, flags, &patch)` | `draw` | same, plus precedence level 6 |
| `Ui::with_part(f, v, p, flags, |ui, r| …)` | `draw` | `style` + a closure, one statement |
| `Ui::resolve(f, v, p, flags)` | `measure` | `&self`, uncached, records nothing |
| `Theme::metrics(f, v, p, flags)` | `update` | glyph / size / align only — there is no `Surface` in `update` |

`Theme::metrics` exists because `update` has no surface and must not resolve
colours, yet still needs sizes: a dialog computing its layer height, a form
computing a field's row count. It runs the *same* accumulation as
`Theme::resolve` and reads the same slots, so the two cannot disagree about a
size.

Surfaces are contextual: `ui.with_surface(Surface::Elevated, |ui| …)` for a
card, and layers set their own. Never take a background colour as a
parameter.

### Declare your family

If your `Family` is a `Family::custom(…)`, the application's theme should
declare it with `Theme::define_family`. Two things follow if it does not:

- an undeclared family resolves through the neutral recipe, which renders
  *something* but is not yours;
- `Theme::downgrade(ColorLevel::Mono)` appends the mono fallback rules only to
  **declared** families, so an undeclared family gets none, and a focused
  control is indistinguishable from an unfocused one without colour.

Either declare the family in the theme, or paint your own capability-independent
affordance (a glyph, a modifier, a bracket) — see the conformance section
below, where this becomes a hard test failure.

---

## Focus

```rust
ui.register_control(self.id, area, Focusability::Focusable);
```

`Focusability` is `Focusable`, `FocusableReadOnly` (reachable, never editable;
also declares `READ_ONLY`), `Disabled` (registered as a hit target, never
reachable) and `ClickOnly` (a hit target with no ring entry).

For a text control use `register_editor`, which sets `swallows_typing` on the
focus entry and declares your flags in one call:

```rust
ui.register_editor(self.id, area, Focusability::Focusable, StateFlags::EDITING);
```

`swallows_typing` is what tells the runtime that a bare `Char` chord belongs
to the control rather than to an application accelerator, and it is what routes
`Intent::Paste` correctly.

Traversal order **is registration order**, which is draw order. There is no
tab-index. Wrap a region in a scope with:

```rust
ui.focus_scope(SCOPE_ID, ScopeMode::Trap, |ui| { /* … */ });
```

`ScopeMode::Trap` is a modal trap; layers push their own scope automatically
(`Modal → Trap`, `Popover`/`Tooltip → Normal`), so a component that opens a
layer never manipulates a barrier by hand.

`cx.focus(id)` requests focus. Autofocus is "request focus on the first
`update` that runs before I have ever been drawn":

```rust
if self.autofocus && cx.area(self.id).is_none() {
    cx.focus(self.id);
}
```

## Hover, press and activation

You do not track them. The runtime owns hover, press, the press flash,
double-click detection and click-outside. Read the result:

```rust
let live = ui.state(self.id);   // FrameRead::state
if live.contains(StateFlags::HOVERED) { … }
```

`FrameRead::state(id)` returns the runtime-resolved flags (`FOCUSED`,
`FOCUS_VISIBLE`, `HOVERED`, `PRESSED`) **plus** whatever the component
declared last frame.

Flags you derive yourself — `DISABLED` from props, `SELECTED` from your state,
`ERROR` from validation — are announced with:

```rust
ui.declare_state(self.id, StateFlags::EDITING | StateFlags::DIRTY);
```

Declared flags follow a **one-frame contract**: they land in *last* frame's
declared list and are read back on the next frame, the same rule as
`cx.area`. A paste in the same pass that began an edit is therefore not routed
as editing; the edit must have been declared by a previous draw.

## Dispatch and hit testing

Register the sub-regions that should receive pointer intents:

```rust
ui.register_part(self.id, PartRef::item(SEGMENT, ItemKey::index(i)), cell);
```

A `PartRef` is `(Part, Option<ItemKey>)` — 24 bytes, `Copy`. `PartRef::of(p)`
for a singleton region, `PartRef::item(p, k)` for one row of a collection.
That key comes straight back to you in `Intent::Pointer { part, .. }`, so
"which row was clicked" is answered by the runtime, not by a reverse scan over
cached rectangles.

Use `register_decor` for a region that should *block* hits without delivering
them (a dialog's chrome), and `register_scroll` for a wheel target:

```rust
ui.register_scroll(self.id, area, Axes::V, view.headroom_v());
```

`Headroom` is how much room remains in each direction, and it is what makes
the boundary-wheel rule work: a wheel at the end of a scrollable region is
**consumed without a repaint**, and only then does it bubble.

For a drag, claim pointer capture in `update`:

```rust
if cx.capture(self.id, PartRef::of(Part::THUMB)) {
    // every subsequent Pointer intent is delivered here until release,
    // even outside the region. `cx.capture_origin()` is the press position,
    // so `pos - origin` is the grab offset inside the thumb.
}
```

`cx.capture_area()`, `cx.capture_owner()` and `cx.release_capture()` complete
the set. No component reconstructs a cached rect to decide where a drag
started.

## Cursor

```rust
ui.set_cursor(self.id, Position::new(x, y));
```

Write it unconditionally. Filtering is the runtime's job: the request is kept
only if it comes from the top layer and the owner is focused, and the best
candidate wins (higher layer first, then the focused owner, then the later
write). A rejected write is reported as a `Diagnostic::CursorRejected` rather
than silently dropped.

## Scrolling

`ScrollState` is caller-owned and lives in your `XState`. The useful surface:
`set_content`, `set_viewport`, `apply_layout(viewport, content)`,
`scroll_by`, `scroll_to`, `page_up`, `page_down`, `jump_start`, `jump_end`,
`ensure_visible(i)`, `ensure_visible_on_next_layout(i)`, `visible_range()`,
`headroom_v()`, `thumb(track_len)`, `offset_for_track_pos(pos, track_len)`,
and `wheel(delta) -> Response<()>` which already implements the boundary rule.

`delta` on `Intent::Wheel` has already been multiplied by
`design.motion.wheel_rows`.

`ScrollRegion` wraps the track, thumb and capture-based thumb drag as a
reusable component if you only need a scrollbar.

## Overlays

A component that owns a layer opens it in `update`:

```rust
cx.open_layer(
    OWNER_PICK,
    LayerSpec::popover(
        OWNER_PICK,
        Anchor::Rect { rect: anchor, side: Side::Below, align: CrossAlign::Start },
    )
    .dismiss(Dismiss::ESC_AND_OUTSIDE)
    .size(owner_picker().measured_size(cx, &self.people)),
);
```

and draws it inside `ui.layer(id, |ui, area| …)`. Rules:

- **A component that owns a layer runs its `update` unconditionally, every
  frame, open or not.** The dismissal is delivered as intents addressed to
  the layer's owner in the pass *after* the layer closed, so an
  `if cx.is_open(…)` guard around the component's own `update` drops it.
  Guard the work the *caller* does besides the component, never the
  component's `update`.
- **A component computes a size, never a rect.** Placement, flip, clamp and
  clipping live in the one resolver. `LayerSize` is `Fill` or `Fixed(w, h)`;
  re-assert it with `cx.resize_layer` and move an anchor with
  `cx.reanchor_layer`.
- **z-order is the `LayerId` assigned by `open_layer`**, not the order of
  `ui.layer` calls.
- Focus trapping, the backdrop, Esc, click-outside, focus restoration and the
  hint layer all come from the layer, not from your component.

`cx.layer_event(id)` returns `LayerEvent::{Opened, Dismissed(reason)}` once.
`cx.top_layer()` and `cx.is_open(id)` are the queries.

`crates/tui/examples/10_nested_overlay.rs` is a popover opened on top of a
modal dialog, with Esc closing only the popover and focus returning to the
button beneath.

---

## Registering a conformance case

The shared suite is twenty tests. Implement `Conformance` once and the
`conformance_suite!` macro generates all of them for your component:

```rust
use junie_tui_testing::conformance::{Caps, Conformance, Fixture};
use junie_tui_testing::conformance_suite;

struct SegmentedCase;

impl Conformance for SegmentedCase {
    const NAME: &'static str = "segmented";
    const FAMILY: Family = Family::custom("segmented");
    const PARTS: &'static [Part] = Segmented::PARTS;
    type State = SegmentedState;
    type Action = SegmentedAction;
    type Cmd = SegCmd;

    fn caps() -> Caps {
        Caps::ACTIVATES | Caps::FOCUSABLE
    }

    fn id() -> Id {
        SEG
    }

    fn update(cx: &mut Cx<'_>, st: &mut SegmentedState, _f: &Fixture) -> Response<SegmentedAction> {
        Segmented::new(SEG, &LABELS).update(cx, st)
    }

    fn draw(ui: &mut Ui<'_>, area: Rect, st: &SegmentedState, f: &Fixture) {
        let patch = f
            .patch
            .map(|p| [p])
            .unwrap_or([(Part::CONTAINER, StylePatch::new())]);
        let mut c = Segmented::new(SEG, &LABELS).patch_part(&patch);
        if !f.forced().is_empty() {
            c = c.state_override(f.forced());
        }
        c.draw(ui, area, st);
    }

    fn activation_chords() -> &'static [Chord] {
        const CHORDS: [Chord; 2] = [Chord::key(KeyCode::Enter), Chord::key(KeyCode::Char(' '))];
        &CHORDS
    }

    fn activation_part() -> PartRef {
        PartRef::item(SEGMENT, ItemKey::index(0))
    }

    fn bindings(s: BindingState) -> &'static [Binding<SegCmd>] {
        Segmented::new(SEG, &LABELS).bindings(s)
    }

    fn mono_states() -> &'static [StateFlags] {
        const STATES: [StateFlags; 4] = [
            StateFlags::empty(),
            StateFlags::FOCUSED,
            StateFlags::SELECTED,
            StateFlags::PRESSED,
        ];
        &STATES
    }

    fn mono_narrowing_reason() -> &'static str {
        "DISABLED ERROR WARNING EDITING BUSY ACTIVE: a segmented control has no \
         disabled, validation, edit, busy or active-item state"
    }
}

conformance_suite!(segmented => SegmentedCase);
```

That generates `conformance::segmented::<case>` for all twenty, plus a guard
that the module identifier matches `NAME`:

| # | Case |
|---|---|
| 1 | `disabled_cannot_activate` |
| 2 | `keyboard_and_mouse_activation_are_equivalent` |
| 3 | `traversal_order_is_registration_order` |
| 4 | `hover_does_not_steal_focus` |
| 5 | `draw_twice_is_byte_identical` |
| 6 | `draw_twice_leaves_state_equal` |
| 7 | `draw_does_not_commit_or_cancel` |
| 8 | `draw_stays_inside_its_area` |
| 9 | `mono_states_are_distinguishable` |
| 10 | `local_override_does_not_mutate_the_theme` |
| 11 | `id_separator_collision_free` |
| 12 | `item_identity_survives_reorder` |
| 13 | `focus_reconcile_follows_the_rule` |
| 14 | `focus_trap_and_restore` |
| 15 | `pointer_capture_delivers_drag_and_release` |
| 16 | `wheel_at_boundary_is_consumed_without_repaint` |
| 17 | `cursor_write_is_rejected_off_top_layer` |
| 18 | `secret_never_appears_in_debug` |
| 19 | `survives_tiny_rects_0x0_to_3x3` |
| 20 | `bindings_match_handled_keys` |

`Caps` selects the capability-gated cases: `ACTIVATES`, `DISABLEABLE`,
`FOCUSABLE`, `COLLECTION`, `EDITS`, `SCROLLS`, `OVERLAY`, `CAPTURES`,
`CURSOR`, `SECRET`, `TYPES`. Declaring a capability never lets its case skip —
the driver checks that too.

### Three things example 12 does not have, and case 9 and case 10 need

`crates/tui/examples/12_author_component.rs` is a complete component, but it
is an *example*, not a registered conformance case. Registering it as written
fails two cases. Verified, both:

1. **`local_override_does_not_mutate_the_theme` (case 10)** fails with *"the
   instance patch had no effect"*. The driver puts a `StylePatch` on
   `PARTS.first()` and asserts the render changes. Example 12's `Segmented`
   has no `.patch`/`.patch_part` builder at all, so nothing happens. Add
   `patch_part(&'a [(Part, StylePatch)])`, resolve through
   `Ui::style_patched`, and make sure `PARTS.first()` is a part you actually
   paint — the driver patches the *first* declared part.
2. **`mono_states_are_distinguishable` (case 9)** fails twice over:
   - The driver forces each state with `Fixture::force`, and a component
     without a `.state_override(StateFlags)` builder never sees it, so every
     state renders identically. Add `state_override`, and have the forced
     path register **no** control and no parts — a reference rendering must
     not leave a live control behind it.
   - The fixture theme is `Theme::junie()`, in which
     `Family::custom("segmented")` is undeclared, so
     `apply_mono_fallbacks` never reaches it. The theme will not make your
     states distinguishable. Paint the affordance yourself: a
     `GlyphRole::FocusBar` in the gutter column when focused, a
     `GlyphRole::Chosen` marker when selected, a `GlyphRole::PressLeft`
     bracket when pressed. That is what the library's own components do at
     `Mono`, and it works whether or not the application declared your family.

    A mono affordance must never change geometry — reserve the cells in your
    normal layout and paint into them, rather than shifting the label right by
    one when pressed.

With those three additions the same `Segmented` passes all twenty cases.

### Beyond conformance

- **Digest tests.** `Scene::new(name, theme, color, w, h)` renders headlessly
  and produces a stable digest, with `Scene::text()` for readable failure
  output and `Scene::registry()` / `Scene::ring()` for structural assertions.
- **Interaction tests.** `Harness::new(app, theme, w, h)` drives a whole
  application deterministically: `key`, `ctrl`, `type_str`, `paste`, `click`,
  `click_id`, `click_part`, `double_click`, `drag`, `wheel`, `resize`,
  `tab_to`, `tick`, plus `buffer()`, `text()`, `find()`, `cursor()`,
  `state_of()`, `diagnostics()` and `advance_clock(ms)`. No wall clock, no
  real terminal.
- **Visual capture.** Register the states you care about under
  `render::components::<x>::{default, focused, hovered, pressed, disabled,
  selected, editing, empty}` so a visual change shows up as a baseline diff
  rather than as a surprise.

---

## Checklist

- [ ] Props struct with a lifetime; one constructor function, no `&self`;
      data passed per phase call.
- [ ] `#[must_use]` consuming builders; `const fn` where possible.
- [ ] `XState`: `Default + Clone + PartialEq + Debug`, interaction state only.
- [ ] `update(&self, cx, &mut st[, data]) -> Response<XAction>`, ending in
      `.for_id(self.id)`.
- [ ] `draw(&self, ui, area, &st[, data]) -> Rect`, `&self` only, returning
      early and registering nothing on a degenerate rect.
- [ ] `measure(&self, ui: &Ui<'_>, c: Constraints) -> Size` using
      `Ui::resolve`, not `Ui::style`.
- [ ] `PARTS`, and every part you resolve is in it.
- [ ] `impl Bindings` with a `&'static` table; `visible` set on exactly one
      chord per command.
- [ ] `.patch`, `.patch_part`, `.slot` and `.state_override` builders.
- [ ] A `Family` (declared in the theme if it is custom) and a mono affordance
      that does not depend on hue or change geometry.
- [ ] `register_control` / `register_editor`, `register_part` for every
      clickable sub-region, `register_scroll` if you scroll.
- [ ] `declare_state` for every flag you derive yourself.
- [ ] Saturating arithmetic everywhere.
- [ ] A `Conformance` impl and a `conformance_suite!` entry.
- [ ] The fifteen rustdoc sections.
