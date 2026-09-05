# Component overrides

You have a global theme and it is nearly right. One button in one dialog
needs a warning-coloured label. A list in the sidebar needs a different
marker glyph. A whole family of buttons needs a square gutter instead of a
bar.

There are five mechanisms for this, and they differ in **scope**, not in
power. Picking the smallest one that reaches what you want is the whole
skill.

| Mechanism | Scope | Written where | Cost |
|---|---|---|---|
| `Theme::define_family` / `define_variant` | the recipe for a family / variant you own | theme construction | none per frame |
| `Theme::override_family` / `override_variant` | every instance of a family / variant, application-wide | theme construction | none per frame |
| `Ui::with_overlay` | one subtree of one frame | `draw` | one `&'static` slice, no allocation |
| `.patch` / `.patch_part` | one instance | the call site | a `const`, no allocation |
| `.slot` | one part's *painting* on one instance | the call site | one `&dyn Fn` |

They are precedence levels 1–2, 4, 5 and 6 respectively (level 3 is state
rules, which every level can write). Higher wins. See
[`theming.md`](theming.md#precedence) for the full chain.

None of these requires editing a component's source, and none of them mutates
the theme.

---

## `.patch_part` — one instance, named parts

The common case. A `&'a [(Part, StylePatch)]` slice, usually a `const`:

```rust
use junie_tui::{Button, Part, Role, StylePatch};

const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL, StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

Button::new(RESET, "Reset").patch_part(&RESET_LABEL).draw(ui, area);
```

`StylePatch` is `const`-constructible end to end (`set_fg`, `clear_fg`,
`set_bg`, `clear_bg`, `set_underline`, `add`, `remove`, `set_glyph`,
`clear_glyph`, `set_size`, `set_align` are all `const fn`), so the whole override is baked
into the binary and costs one slice scan per styled part at render time.

## `.patch` — one instance, every part

`.patch(&StylePatch)` applies to **every** part the component resolves. Use
it for something genuinely uniform — dimming a whole control, forcing a
background plane:

```rust
const DIMMED: StylePatch = StylePatch::new().add(Modifier::DIM);

TextInput::new(FIELD).patch(&DIMMED).draw(ui, area, &st);
```

If you find yourself writing `.patch` and then mentally excluding parts, you
wanted `.patch_part`.

## Part slots — replace a part's painting

`.slot(Part, &dyn Fn(&mut Ui<'_>, Rect))` hands you the part's rect and lets
you paint it yourself. Everything else the component does — layout,
measurement, hit-region registration, focus registration, the other parts —
is unchanged:

```rust
let bang = |ui: &mut Ui<'_>, r: Rect| {
    let st = ui.surface_style();
    ui.paint_str(r, "!", st);
};

Button::new(OK, "Label")
    .slot(Part::GUTTER, &bang)
    .draw(ui, Rect::new(0, 0, 20, 1));
```

Verified: the gutter cell now reads `!`, the label is still the component's
own, and the button is still hit-testable and still in the focus ring at the
same geometry.

```rust
assert_eq!(s.buffer().cell((0, 0)).map(Cell::symbol), Some("!"));
assert_eq!(s.buffer().cell((1, 0)).map(Cell::symbol), Some("L"));

let area = s.registry().and_then(|r| r.area_of(OK)).unwrap();
let hit  = s.registry()
    .and_then(|r| r.hit(Position::new(area.x + 1, area.y)))
    .expect("still hit-testable");
assert_eq!(hit.owner, OK);
```

Each component's rustdoc `## Overrides` section names any part that **cannot**
be replaced by a slot. For `Button` that is `CONTAINER`: its fill *is* the
button. `crates/tui/tests/overrides.rs::part_slot_replaces_the_part_and_keeps_hit_regions`
is the executable form of the guarantee above.

Reach for a slot when you want different *content*, not a different colour. A
colour is a patch.

---

## Custom variants

A `Variant` is a `u16` newtype with library constants (`DEFAULT`, `PRIMARY`,
`SECONDARY`, `SUBTLE`, `DANGER`, `TOGGLE`, `QUIET`, `GHOST`) and a
`Variant::custom("name")` constructor that hashes into a separate high range,
so a downstream variant can never collide with a library one. You are not
confined to the enum, and you do not lose type safety to a string map.

Define the delta once, on the theme:

```rust
const OUTLINE: Variant = Variant::custom("outline");

let t = Theme::junie().define_variant(Family::BUTTON, OUTLINE, |r| {
    r.part(Part::CONTAINER)
        .base(StylePatch::new().set_bg(Role::CurrentSurface));
    r.part(Part::LABEL)
        .base(StylePatch::new().set_fg(Role::Accent));
});
```

Then use it like any other:

```rust
Button::new(SAVE, "Save").variant(OUTLINE).draw(ui, area);
```

Verified: `OUTLINE`'s label resolves to the accent, and `PRIMARY`'s does not.
A variant delta is precedence level 2, so it sits *under* state rules, global
overrides, overlays and instance patches — a custom variant participates in
all of them without any extra work.

`Variant::DEFAULT` is not a variant; it means "whatever this family's
`Recipe.default_variant` says". Under `Theme::junie()`,
`Family::BUTTON.default_variant` is `Variant::DEFAULT` itself — the family's
base recipe already paints the default look. Under `Theme::paper()` it is
`Variant::SECONDARY`, so an unadorned `Button` picks up the secondary delta.
Change it with `r.default_variant(v)` inside any of the four recipe
editors.

## Custom families and recipes

`Family::custom("badge")` works the same way, and `Theme::define_family`
declares its recipe:

```rust
const BADGE: Family = Family::custom("badge");

let t = Theme::junie().define_family(BADGE, |r| {
    r.part(Part::LABEL)
        .base(StylePatch::new().set_fg(Role::OnAccent))
        .size(1);
    r.part(Part::CONTAINER)
        .base(StylePatch::new().set_bg(Role::Accent));
});
```

Inside a recipe editor (`RecipeEdit`) you get:

- `default_variant(Variant)`
- `part(Part) -> &mut PartEdit`

and inside a `PartEdit`:

- `base(StylePatch)` — the part's base patch (merged if called twice)
- `when(StateFlags, StylePatch)` — a state rule, inserted at its specificity
  position so declaration order breaks ties
- `glyph(GlyphRole)` — the part's glyph
- `size(u16)` — the part's size

Two traps, both covered in [`theming.md`](theming.md#scenario-10--an-application-specific-component-on-the-same-theme):
family edits are sparse, so an **empty** edit keeps the neutral fallback rather
than creating an empty recipe. An **undeclared** custom family receives the
generic mono fallback manifest but no family-targeted or authored mono rules at
`ColorLevel::Mono`. Use a `clear_*` patch explicitly when suppressing one
inherited value, including `clear_glyph` for a reserved glyph cell.

### `define_*` versus `override_*`

| | Level | Semantics |
|---|---|---|
| `define_family` | 1 | *this is* the family's base recipe; merges into it |
| `define_variant` | 2 | *this is* the variant's delta; merges into it |
| `override_family` | 4 | applied over whatever the recipe resolved to, for every variant |
| `override_variant` | 4 | the same, for one variant |

Use `define_*` for a family you own. Use `override_*` to restyle a family
somebody else defined — which for the library components means all of them.

---

## Scoped overrides

`Ui::with_overlay` pushes a rule slice for the duration of a closure. It is
the right tool for "this whole panel is quieter" or "everything inside this
dialog uses the danger palette", where naming every instance would be
tedious and a global override would be wrong.

```rust
static QUIET_LABELS: [OverlayRule; 1] = [(
    Family::BUTTON,
    Variant::PRIMARY,
    Part::LABEL,
    StateFlags::empty(),
    StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
)];

ui.with_overlay(&Overlay::new(&QUIET_LABELS), |ui| {
    // every PRIMARY button drawn in here has a muted label
});
```

`OverlayRule` is `(Family, Variant, Part, StateFlags, StylePatch)`. Matching
is exact on family, variant and part, and `live.contains(when)` on state.
Overlays nest: the stack applies outermost → innermost, so an inner scope
wins. The stack's identity is folded into the style memo key, so a scoped
render is cached correctly rather than being cached *wrongly*.

An overlay is `Copy` and holds only a `&'static [OverlayRule]`. It cannot
allocate and it cannot outlive the frame.

---

## Scenario C: two buttons, one overridden locally

The litmus test the architecture is judged against: *two buttons using the
same global theme appear in one screen; one uses the default component
recipe, the other overrides a meaningful part or state locally, without
requiring a new global theme or a copied renderer.*

`crates/tui/examples/05_instance_patch.rs`, in full:

```rust
use junie_tui::{
    Button, FrameRead, Id, Part, Rect, Role, RowAlign, StylePatch, Ui, Variant, id, layout,
};

const OK: Id = id!("ok");
const RESET: Id = id!("reset");

// One patch, declared `const`, so it costs nothing per frame.
const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL, StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

pub fn draw_actions(ui: &mut Ui<'_>, area: Rect) {
    let cols = layout::action_row(area, &[10, 12], ui.design().space.gap, RowAlign::End);
    Button::new(OK, "OK")
        .variant(Variant::PRIMARY)
        .draw(ui, cols[0]);
    Button::new(RESET, "Reset")
        .patch_part(&RESET_LABEL)
        .draw(ui, cols[1]);
}
```

Both buttons use the same global theme and the same renderer. Only one is
patched. Three things are true and each is asserted somewhere in the tree:

1. **The theme is byte-identical afterwards.** `Theme::fingerprint()` before
   and after the frame are equal — an instance patch is not a theme edit.
   (`conformance::button::local_override_does_not_mutate_the_theme`, and case
   10 runs for every registered component.)
2. **The sibling is untouched.** The `OK` button renders exactly as it would
   without the patch, so the override is not a variant-level change wearing an
   instance's name.
   (`crates/tui/tests/overrides.rs::instance_patch_changes_only_one_instance`.)
3. **Nothing was copied.** There is no second button renderer, no forked
   recipe and no `if is_reset { … }` inside `Button::draw`.

---

## Choosing

Work outwards from the call site.

- **One instance, one or two parts** → `.patch_part`.
- **One instance, uniformly** → `.patch`.
- **One instance, different content in one region** → `.slot`.
- **A repeated look you will use in several places** → a custom `Variant`
  with `define_variant`, then `.variant(…)` at each call site. This is the
  right answer far more often than a pile of identical `patch_part` consts.
- **A whole subtree** → `Ui::with_overlay`.
- **Every instance in the application** → `override_family` /
  `override_variant` on the theme.
- **A family you are inventing** → `define_family`, and read
  [`authoring.md`](authoring.md).

If you are reaching for `Role::Custom(Color)` or an `if` inside a component,
stop: the answer is a missing `Role` or a missing `Part`, and both are cheap
to add.
