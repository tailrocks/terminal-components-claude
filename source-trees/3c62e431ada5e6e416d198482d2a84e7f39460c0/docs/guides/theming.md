# Theming and customization

Every colour a component paints comes from a **semantic role**, never from a
literal. A component asks for `Role::Accent` or `Role::Fg(FgStep::Muted)`;
the theme decides what that is on the current surface at the current colour
capability. That is the whole reason a custom theme can restyle the library
without editing a component.

This guide walks the twelve customisation scenarios the project requires, in
order, each with code. Then it explains the precedence chain those scenarios
sit on, and the `Slot` merge law that makes "unset" different from "cleared".

Everything here is written against the final `junie_tui` public crate path.

---

## The shape of a theme

```rust
pub struct Theme {
    pub color: ColorTokens,      // every colour the theme supplies
    pub design: DesignTokens,    // spacing, sizes, glyphs, borders, motion, density
    pub recipes: Recipes,        // per-family, per-variant, per-part style rules
    pub capability: Capability,  // the colour depth it is resolved for
}
```

The split is deliberate:

- **`ColorTokens`** — the surface ladder (`canvas`, `surface`, `elevated`,
  `overlay`, `popover`), the two field planes, the foreground ladder
  (`primary`, `secondary`, `muted`, `faint`, `ghost`), accent and on-accent,
  borders, focus, selection, highlight, backdrop, danger / warning / success /
  info, disabled and read-only, plus nested `SyntaxTokens` and `MeterTokens`.
- **`DesignTokens`** — `SpaceTokens` (gutter, inline, gap, column gap, form
  gap, card / frame / dialog insets, tree indent), `SizeTokens` (field
  height, tab strip height, dialog widths, popup bounds, minimum terminal
  size, scrollbar width, meter track, code preview lines), `GlyphSet`,
  `BorderSet`, `MotionTokens` (spinner frames, tick periods, press flash,
  status duration, wheel rows, double-click window), `MeterThresholds` and
  `Density`.
- **`Recipes`** — which parts a family paints, per variant, per state. This
  is where "a focused button's label is bold" lives.
- **`Capability`** — the `ColorLevel` the tokens have already been mapped to.

So: spacing, dimensions, glyphs, border glyph sets, animation cadence and
density are **design tokens**; focus indicators, selection indicators,
scrollbar symbols and variant defaults are **recipes**; colour is **colour
tokens**. Nothing is a loose constant inside a component.

`ColorTokens` is deliberately **not** `#[non_exhaustive]`. Adding a token is
an intentional breaking change for downstream themes, and
`ColorTokens::map_colors`'s exhaustive destructure is the mechanism that
makes every capability downgrade cover every token.

---

## Scenario 1 — the default, with no configuration

```rust
use junie_tui::{Theme, run};

fn main() -> std::io::Result<()> {
    run(Counter::default(), Theme::junie())   // `Counter` is quickstart.md's app
}
```

`Theme::junie()` is the approved default; its token values are unchanged from
the pre-refactor baseline. `Theme::paper()` is the second built-in, and it is
deliberately not an accent-colour variation of Junie: it is light rather than
dark, indigo rather than green, square rather than rounded, `Density::Compact`
rather than comfortable, and its surface ladder runs *darker* as it rises
where Junie's runs lighter. It exists to expose assumptions that only hold
for Junie.

---

## Scenario 2 — a complete custom theme

`Theme::from_tokens` takes a full `ColorTokens` literal, fills in design
tokens and recipe defaults, and derives every token you left as
`Color::Reset`.

```rust
use junie_tui::Color;
use junie_tui::theme::{ColorTokens, Density, MeterTokens, SyntaxTokens, Theme, border};

fn slate() -> Theme {
    Theme::from_tokens(ColorTokens {
        surfaces: [
            Color::from_u32(0x0A0C10),
            Color::from_u32(0x10131A),
            Color::from_u32(0x171B24),
            Color::from_u32(0x1F2430),
            Color::from_u32(0x282E3D),
        ],
        field: Color::from_u32(0x10131A),
        field_hover: Color::from_u32(0x171B24),
        fg: [
            Color::from_u32(0xE8ECF4),
            Color::from_u32(0xBAC1D0),
            Color::from_u32(0x8C94A6),
            Color::from_u32(0x61697C),
            Color::from_u32(0x404758),
        ],
        on_accent: Color::from_u32(0x080A0E),
        on_danger: Color::from_u32(0xFFF5F5),
        on_surface_inverse: Color::from_u32(0x0A0C10),
        border_subtle: Color::from_u32(0x1F2430),
        border_strong: Color::from_u32(0x485062),
        accent: Color::from_u32(0x7AA2F7),
        accent_hover: Color::from_u32(0x93B4FA),
        accent_pressed: Color::from_u32(0x608AE8),
        accent_tint: Color::from_u32(0x162036),
        focus: Color::from_u32(0x7AA2F7),
        focus_ring: Color::from_u32(0x608AE8),
        selection_bg: Color::from_u32(0x1F2C48),
        selection_fg: Color::from_u32(0xE8ECF4),
        highlight_bg: Color::from_u32(0x263454),
        highlight_fg: Color::from_u32(0xE8ECF4),
        highlight_danger_bg: Color::from_u32(0x542026),
        highlight_danger_fg: Color::from_u32(0xFFEBEB),
        backdrop_fg: Color::from_u32(0x404758),
        backdrop_bg: Color::from_u32(0x080A0E),
        danger: Color::from_u32(0xF06E78),
        danger_soft: Color::from_u32(0x602A32),
        danger_tint: Color::from_u32(0x30161C),
        warning: Color::from_u32(0xE0A850),
        warning_tint: Color::from_u32(0x382A14),
        success: Color::from_u32(0x7EC88C),
        info: Color::from_u32(0x78B4DC),
        disabled_fg: Color::from_u32(0x4A5264),
        disabled_bg: Color::from_u32(0x10131A),
        read_only_fg: Color::from_u32(0x8C94A6),
        syntax: SyntaxTokens::derive(
            Color::from_u32(0x7AA2F7),
            Color::from_u32(0x7EC88C),
            Color::from_u32(0xE0A850),
        ),
        meter: MeterTokens::derive(
            Color::from_u32(0x7EC88C),
            Color::from_u32(0xE0A850),
            Color::from_u32(0xF06E78),
        ),
    })
    .builder()
    .borders_set(border::PLAIN)
    .density(Density::Compact)
    .build()
}
```

`SyntaxTokens::derive` and `MeterTokens::derive` take three hues each and
leave the rest `Color::Reset`; `Theme::from_tokens` fills those from the main
tokens. `Color::from_u32` is the one literal colour constructor used in this
codebase.

For a terminal or font without box-drawing glyphs:

```rust
fn slate_ascii() -> Theme {
    slate().builder().borders_set(border::ASCII).build()
}
```

`border::ASCII` is a plain `const` beside ratatui's `PLAIN`, `ROUNDED` and
`DOUBLE`. Choosing it also swaps the rule and scrollbar glyph sets to ASCII
(`ThemeBuilder::ascii_glyphs`), because a theme that draws `+---+` corners and
then paints `─` in a divider is ASCII at the edges and Unicode everywhere
else. The swap is a **theme author's** decision, never a capability
detection; and it covers the box-drawing block only — `›`, `✓`, `…`, `×` and
the spinner frames stay Unicode. Call `.glyph(role, "…")` *after*
`.borders_set(…)` to override any of them; the last write wins.

---

## Scenario 3 — change a few roles, derive the rest safely

`Theme::builder()` starts from an existing theme. Setting a *seed* token
resets the tokens derived from it, unless you set those explicitly in the same
builder.

```rust
use junie_tui::{Color, Theme};

fn amber() -> Theme {
    Theme::junie()
        .builder()
        .accent(Color::from_u32(0xC67A2E))
        .danger(Color::from_u32(0xB02525))
        .build()
}
```

That is the whole change. Verified:

```rust
let t = amber();
assert_eq!(t.color.surfaces, Theme::junie().color.surfaces);   // untouched roles inherit
assert_eq!(t.color.fg, Theme::junie().color.fg);
assert_eq!(t.color.focus, Color::from_u32(0xC67A2E));          // derived from the new accent
assert_ne!(t.color.accent_hover, Theme::junie().color.accent_hover);
```

Setting `accent` re-derives `accent_hover`, `accent_pressed`, `accent_tint`,
`focus`, `focus_ring` and `on_accent`. Setting `danger` re-derives
`danger_soft`, `danger_tint` and `on_danger`. Setting `warning` re-derives
`warning_tint`. Setting `surfaces` re-derives `border_subtle`; setting `fg`
re-derives `border_strong`. Call `.focus(c)` or `.borders(subtle, strong)`
after the seed to pin a derived token instead.

The derivation is deterministic — the same inputs always produce the same
theme — and it is the same code path `Theme::from_tokens` uses for
`Color::Reset` tokens, so a hand-written theme and a derived one cannot
disagree.

Other builder setters: `success`, `info`, `selection(bg, fg)`,
`highlight(bg, fg)`, `field(base, hover)`, `disabled(fg, bg)`, `surfaces`,
`fg`, `borders`, `space`, `size`, `density`, `motion`, `glyph`,
`borders_set`, `ascii_glyphs`.

---

## Scenario 4 — override one component family globally

```rust
use junie_tui::{
    Family, FgStep, GlyphRole, Modifier, Part, Role, StateFlags, StylePatch, Theme, Variant,
};

let t = Theme::junie().override_family(Family::BUTTON, |r| {
    r.default_variant(Variant::SECONDARY);
    r.part(Part::GUTTER).glyph(GlyphRole::FocusBar);
    r.part(Part::LABEL)
        .base(StylePatch::new().set_fg(Role::Fg(FgStep::Primary)))
        .when(StateFlags::FOCUSED, StylePatch::new().add(Modifier::BOLD))
        .when(
            StateFlags::DISABLED,
            StylePatch::new()
                .set_fg(Role::DisabledFg)
                .remove(Modifier::BOLD),
        );
    r.part(Part::CONTAINER)
        .when(
            StateFlags::HOVERED,
            StylePatch::new().set_bg(Role::AccentTint),
        )
        .when(
            StateFlags::HOVERED | StateFlags::PRESSED,
            StylePatch::new()
                .set_bg(Role::AccentPressed)
                .set_fg(Role::OnAccent),
        );
});
```

One call reaches every button of every variant in the application, and
nothing outside `Family::BUTTON`. No component source is edited. The two-flag
`HOVERED | PRESSED` rule beats the one-flag `HOVERED` rule because it is more
specific; see [Precedence](#precedence) below.

---

## Scenario 5 — override one variant globally

```rust
let t = Theme::junie().override_variant(Family::BUTTON, Variant::PRIMARY, |r| {
    r.part(Part::LABEL)
        .base(StylePatch::new().set_fg(Role::Warning));
});
```

`PRIMARY` buttons change; `SECONDARY` buttons are byte-identical to the
un-overridden render. `crates/tui/tests/overrides.rs::global_variant_override_changes_only_that_variant`
asserts exactly that, by reading colour off the painted cells.

---

## Scenario 6 — override within a scope or subtree

An `Overlay` is a borrowed, `const`-constructible slice of rules pushed onto a
draw-time stack. It restyles the subtree it wraps and unwinds afterwards, and
it **never mutates the theme**.

```rust
use junie_tui::{
    Button, Family, FgStep, Id, Overlay, OverlayRule, Part, Rect, Role, StateFlags, StylePatch,
    Ui, Variant,
};

const A: Id = Id::root("guide.a");
const B: Id = Id::root("guide.b");

static QUIET_LABELS: [OverlayRule; 1] = [(
    Family::BUTTON,
    Variant::PRIMARY,
    Part::LABEL,
    StateFlags::empty(),
    StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
)];

fn draw_two(ui: &mut Ui<'_>) {
    Button::new(A, "Save")
        .variant(Variant::PRIMARY)
        .draw(ui, Rect::new(0, 0, 20, 1));
    ui.with_overlay(&Overlay::new(&QUIET_LABELS), |ui| {
        Button::new(B, "Save")
            .variant(Variant::PRIMARY)
            .draw(ui, Rect::new(0, 2, 20, 1));
    });
}
```

An `OverlayRule` is `(Family, Variant, Part, StateFlags, StylePatch)`. A rule
matches when the family, variant and part are equal and `live.contains(when)`.
Because the rules are `&'static`, a scope costs nothing per frame.

Under the `testing` feature you can assert the "never mutates" half directly:

```rust
let theme = Theme::junie();
let before = theme.fingerprint();
// … render with the scope pushed …
assert_eq!(theme.fingerprint(), before);
```

---

## Scenario 7 — override one instance

```rust
use junie_tui::{Button, Part, Role, StylePatch};

const RESET_LABEL: [(Part, StylePatch); 2] = [
    (Part::LABEL, StylePatch::new().set_fg(Role::Warning)),
    (Part::GUTTER, StylePatch::new().set_fg(Role::Warning)),
];

Button::new(RESET, "Reset").patch_part(&RESET_LABEL).draw(ui, area);
```

Declared `const`, so it costs nothing per frame. The sibling button of the
same variant is untouched: an instance patch is not a variant-level change
wearing an instance's name. See [`overrides.md`](overrides.md) for the whole
per-instance vocabulary, including `.patch` (every part at once) and `.slot`
(replace a part's painting entirely).

---

## Scenario 8 — override a logical part

Parts are the named regions a component paints. They are the same identities
used for theming, hit-region metadata, testing and composition. The library
constants are:

`CONTAINER`, `BORDER`, `BACKDROP`, `GUTTER`, `MARKER`, `ICON`, `LABEL`,
`META`, `HELP`, `TITLE`, `BODY`, `ACTIONS`, `FIELD`, `TEXT`, `PLACEHOLDER`,
`ROW`, `CELL`, `HEADER`, `TRACK`, `THUMB`, `RULE`, `TAB`, `CLOSE`, `PREFIX`,
`BADGE`, `OVERFLOW`, `NEW`, `EMPTY`, `QUERY`, `SEAM`, `SUMMARY`, `DETAIL`,
`KEY`, `ACTION`.

`Part::custom("segment")` mints a downstream part in a separate high range, so
a custom part can never collide with a library one.

Every component publishes the parts it styles as `X::PARTS` and documents
them in its rustdoc under `## Parts`. `Button::PARTS` is
`[CONTAINER, GUTTER, LABEL, ICON, MARKER]`. Any of them can be targeted at
any precedence level — globally:

```rust
Theme::junie().override_family(Family::LIST, |r| {
    r.part(Part::META)
        .base(StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
});
```

or on one instance, with `patch_part` as in scenario 7.

---

## Scenario 9 — override states

`StateFlags` is a bitflag set the runtime resolves and the component
declares: `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED`, `PRESSED`, `SELECTED`,
`ACTIVE`, `CHECKED`, `DISABLED`, `READ_ONLY`, `ERROR`, `WARNING`, `BUSY`,
`EDITING`, `DIRTY`, `EXPANDED`, `LOADING`.

A state rule is `when` + a patch, and it applies when `live.contains(when)` —
a **subset** test, not equality. Rules are ordered by specificity
(`when.count_ones()`), ascending, and ties break by declaration order, so a
more specific rule always wins:

```rust
let t = Theme::junie().override_family(Family::BUTTON, |r| {
    r.part(Part::CONTAINER)
        .when(StateFlags::HOVERED, StylePatch::new().set_bg(Role::AccentTint))
        .when(
            StateFlags::HOVERED | StateFlags::PRESSED,
            StylePatch::new().set_bg(Role::AccentPressed),
        );
});

let hover = t.resolve(
    Family::BUTTON, Variant::DEFAULT, Part::CONTAINER,
    StateFlags::HOVERED, Surface::Canvas,
);
let press = t.resolve(
    Family::BUTTON, Variant::DEFAULT, Part::CONTAINER,
    StateFlags::HOVERED | StateFlags::PRESSED, Surface::Canvas,
);
assert_eq!(hover.style.bg, Some(t.color.accent_tint));
assert_eq!(press.style.bg, Some(t.color.accent_pressed));
```

Rules are stored pre-sorted at theme-construction time, so resolution is one
allocation-free scan.

---

## Scenario 10 — an application-specific component on the same theme

A component you write in your own crate consumes the identical theme system.
It names a `Family`, asks `Ui::style` for each part it paints, and gets
`Resolved` back:

```rust
use junie_tui::author::{
    Family, FrameRead, Id, Part, Rect, Resolved, Ui, Variant, width,
};

/// An application-specific badge. It names roles, never colours, so it
/// changes with the theme like every library component.
#[derive(Debug)]
pub struct Badge<'a> {
    id: Id,
    text: &'a str,
    variant: Variant,
}

impl<'a> Badge<'a> {
    pub const FAMILY: Family = Family::custom("badge");
    pub const PARTS: &'static [Part] = &[Part::CONTAINER, Part::LABEL];

    pub const fn new(id: Id, text: &'a str) -> Self {
        Badge { id, text, variant: Variant::DEFAULT }
    }

    /// Pure paint: no `Cx`, no state, nothing to commit.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        let w = width(self.text).saturating_add(2).min(area.width);
        let cell = Rect { width: w, height: 1, ..area };
        let flags = ui.state(self.id);
        let container: Resolved = ui.style(Self::FAMILY, self.variant, Part::CONTAINER, flags);
        ui.fill(cell, container.over(ui.surface_style()));
        let label = ui.style(Self::FAMILY, self.variant, Part::LABEL, flags);
        let inner = Rect {
            x: cell.x.saturating_add(1),
            width: cell.width.saturating_sub(2),
            ..cell
        };
        ui.paint_str(inner, self.text, label.style);
        cell
    }
}
```

`Family::custom("badge")` lands in the same high range as `Part::custom`, so
it cannot collide with a library family. The theme then styles it exactly like
a library family:

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

`define_family` is precedence level 1 (a family base) and `define_variant` is
level 2 (a variant delta) — they *define* the recipe. `override_family` and
`override_variant` are level 4 — they *override* whatever a recipe says. Use
`define_*` for a family you own; use `override_*` to restyle a family somebody
else defined.

> **A family you never declare resolves through the neutral recipe.**
> `Recipes::get_or_neutral` falls back to a neutral row-like recipe
> (`CONTAINER` / `GUTTER` / `MARKER` / `LABEL` / `META` …) so an undeclared
> `Family::custom("x")` renders something instead of resolving to an empty
> style. Two consequences that are easy to trip over:
>
> 1. The generic mono fallback manifest — which the capability downgrade runs
>    at `ColorLevel::Mono` — applies to every **resolvable** recipe, including
>    the neutral recipe. An *undeclared* custom family therefore gets generic
>    state signals, but no family-targeted or authored mono rules. Declare the
>    family with `define_family` when it needs a targeted affordance (or paint
>    the affordance yourself; see [`authoring.md`](authoring.md)).
> 2. Declaring a family replaces the neutral fallback. In particular,
>    `define_family(F, |_| {})` creates an empty recipe; declaring a family
>    means declaring its parts. Use `clear_*` when suppressing a specific
>    value in a recipe that already owns that part.

---

## Scenario 11 — truecolor, 256, 16 and no colour

```rust
use junie_tui::{ColorLevel, Theme};

let t = my_theme().downgrade(ColorLevel::Ansi256);
```

`Theme::downgrade` maps **every** token through `downgrade_color` using
`ColorTokens::map_colors`'s exhaustive destructure, so it works identically
for `Theme::junie()`, `Theme::paper()` and any user theme — there is no
per-token table to keep in sync, and a new token is a compile error until it
is mapped.

| Level | Mapping |
|---|---|
| `TrueColor` | identity |
| `Ansi256` | nearest of the 6×6×6 cube ∪ the 24-step greyscale by squared sRGB distance; ties resolve to the lower index |
| `Ansi16` | nearest of the 16 xterm defaults by **hue family and brightness class**, not by perceptual distance |
| `Mono` | relative luminance: `Y < 0.35 → Black`, `Y > 0.75 → White`, else `Color::Reset` |

The `Ansi16` rule is deliberate and was chosen over nearest-by-CIE76 ΔE.
CIE76 is the more perceptually "correct" answer and the wrong design answer:
it maps Junie's accent and error hues into the *dark* half of the palette,
discarding the brightness contrast the whole accent system rests on, and
collapses `danger_soft` onto a grey. A colour whose channel spread is under 40
collapses to the grey ladder by BT.601 luma; otherwise the dominant channel
selects the hue family and `max(r, g, b) > 180` selects the light half.

For a light theme at `Ansi16`, the mapped foreground ladder additionally
replaces bright `Gray`/`White` entries with `DarkGray`; ANSI16's bright greys
would otherwise disappear against a light canvas. Accent and status hues, and
all non-foreground tokens, keep the mapping above.

`Role::Custom(Color)` — the one documented raw-colour escape hatch — is
downgraded too.

`ColorLevel::detect()` reads `NO_COLOR`, then `COLORTERM` (`truecolor` /
`24bit`), then `TERM` (`256color`, `ghostty`, `kitty`), and falls back to
`Ansi16`. The runtime does not call it for you:

```rust
run(app, Theme::junie().downgrade(ColorLevel::detect()))
```

To switch theme or capability while running, call `Runtime::set_theme`.

---

## Scenario 12 — state meaning without colour

At `ColorLevel::Mono`, `downgrade` additionally applies the generic static
fallback manifest to every resolvable recipe — each declared family and the
neutral fallback — so state survives without hue. The manifest currently has
20 generic entries; built-in families and explicitly authored families may add
targeted entries:

| State | Part | Fallback |
|---|---|---|
| `FOCUSED` | `GUTTER` | glyph `FocusBar` |
| `FOCUSED` | `LABEL` | `BOLD` |
| `SELECTED` | `MARKER` | glyph `Chosen` |
| `CHECKED` | `MARKER` | glyph `Checked` |
| `PRESSED` | `CONTAINER` | inverted fill (`fg` ← canvas, `bg` ← primary fg), `BOLD` |
| `PRESSED` | `LABEL` | glyph `PressLeft` (a bracket) + `BOLD` |
| `DISABLED` | `GUTTER` | glyph cleared |
| `DISABLED` | `MARKER` | glyph cleared, primary fg, all modifiers removed |
| `DISABLED` | `LABEL` / `FIELD` / `TEXT` | primary fg, all modifiers removed, `DIM` |
| `ERROR` | `MARKER` | glyph `Error` |
| `ERROR` | `FIELD` | `UNDERLINED` |
| `WARNING` / `DIRTY` | `MARKER` | glyph `Dirty` |
| `EDITING` | `TEXT` | `UNDERLINED` |
| `ACTIVE` | `RULE` / `LABEL` | glyph `RuleActive` / `BOLD` |

Two details worth knowing:

- The disabled rules use `Role::Fg(FgStep::Primary)` with `DIM`, not
  `Role::DisabledFg` or `Fg(Faint)`. Both of those map below `Y = 0.35` and
  therefore to `Black` — on a `Black` canvas a disabled control would be
  *invisible*, not merely colourless. Declaration order is load-bearing here:
  the `remove(Modifier::all())` disabled rules must precede the `ERROR` rules,
  or `ERROR`'s `UNDERLINED` is erased.
- The fallbacks are also applied to each variant map that re-declares the same
  part, because family and variant state rules are merged in one specificity
  order — otherwise a variant's own `PRESSED` rule would land after the family
  mono rule and erase the bracket.

Verified for both built-in themes:

```rust
for base in [Theme::junie(), Theme::paper()] {
    let mono = base.downgrade(ColorLevel::Mono);
    let gutter = mono.resolve(
        Family::BUTTON, Variant::DEFAULT, Part::GUTTER,
        StateFlags::FOCUSED, Surface::Canvas,
    );
    assert_eq!(gutter.glyph, Slot::Set(GlyphRole::FocusBar));

    let label = mono.resolve(
        Family::BUTTON, Variant::DEFAULT, Part::LABEL,
        StateFlags::FOCUSED, Surface::Canvas,
    );
    assert!(label.style.add_modifier.contains(Modifier::BOLD));
}
```

The conformance suite enforces this per component:
`conformance::<component>::mono_states_are_distinguishable` renders each state
the component can wear at `ColorLevel::Mono` and asserts that no two produce
the same `(symbol, modifier)` multiset. A component may narrow the state list
it is checked against only by supplying
`Conformance::mono_narrowing_reason()` naming every dropped flag.

---

## Precedence

Six levels, lowest to highest:

| # | Level | Written with |
|---|---|---|
| 1 | family base | `Theme::define_family` (and the built-in recipes) |
| 2 | variant delta | `Theme::define_variant` |
| 3 | state rules | `PartEdit::when` — family and variant rules merged as **one** level, ascending by `when.count_ones()`, ties family-first then declaration order |
| 4 | theme-level global override | `Theme::override_family`, `Theme::override_variant` (applied in the order they were pushed; family-wide entries and variant-keyed entries share one list) |
| 5 | scope overlay stack | `Ui::with_overlay`, outermost → innermost |
| 6 | per-instance patch | `.patch`, `.patch_part` on a component |

Only **after** all six does a `Role` bind to a colour, against
`(theme.color, the current Surface, theme.capability)`. That ordering is the
reason `Role::CurrentSurface` and `Role::RaisedSurface` work at all: they are
resolved against wherever the component ended up being drawn, not where its
recipe was written.

Levels 1–5 are memoised in a fixed-size two-way set-associative cache (256
entries in 128 sets) keyed by a 64-bit mix of the family, variant, part,
state and overlay-stack hash. `Ui::style` uses the memo; `Ui::resolve` is the
uncached `&self` path for measurement, so a measurement never evicts a
painting entry.

`Surface` is contextual, not a parameter you thread through render calls. The
ladder is `Canvas → Surface → Elevated → Overlay → Popover`, plus the two
non-ladder planes `Field` and `FieldHover`. `Theme::raise` is index
arithmetic (`min(level + 1, last)`), never colour equality, so a theme with
two identical plane colours still raises correctly. Push a plane with
`ui.with_surface(Surface::Elevated, |ui| …)`; a layer sets its own
(`Modal → Overlay`, `Popover`/`Tooltip → Popover`).

---

## `Slot`: inherit, set, clear

Every slot of a `StylePatch` is a `Slot<T>`:

```rust
pub enum Slot<T> {
    Inherit,  // say nothing; the lower layer wins
    Set(T),   // this value
    Clear,    // resolve to "no value" — the inherited surface shows through
}
```

`Inherit` is the default, so a patch that mentions only `fg` leaves `bg`,
`underline`, `glyph`, `size` and `align` entirely alone. `Clear` is *not* the
same as `Inherit`: it is an explicit statement that overrides a lower layer's
`Set` and resolves to `None`. That is the "explicit way to replace or clear a
default" the design requires.

The merge law is `over` on each slot: `a.merge(b)` takes `b` wherever `b`
speaks (`Set` or `Clear`), and `a` otherwise. Worked:

```rust
let base    = StylePatch::new().set_fg(Role::Accent).add(Modifier::BOLD);
let silent  = StylePatch::new().set_bg(Role::Danger);   // says nothing about fg
let cleared = StylePatch::new().clear_fg();

// `Inherit` says nothing: the lower layer survives.
assert_eq!(base.merge(silent).fg, Slot::Set(Role::Accent));

// `Set` wins over whatever is below it.
assert_eq!(
    base.merge(StylePatch::new().set_fg(Role::Danger)).fg,
    Slot::Set(Role::Danger)
);

// `Clear` also wins, and resolves to "no colour" — the surface shows through.
assert_eq!(base.merge(cleared).fg, Slot::Clear);
assert_eq!(base.merge(cleared).fg.get(), None);

// Modifiers are symmetric: the later word wins, in both directions.
assert!(base.merge(StylePatch::new().remove(Modifier::BOLD)).add.is_empty());
```

`merge` is associative, has `StylePatch::new()` as its identity, and is
idempotent (`a.merge(a) == a`), which is what lets the six levels compose in
any nesting without an ordering surprise. `add` and `remove` are kept as two
disjoint `Modifier` sets: `add(BOLD)` clears `BOLD` from `remove` and vice
versa, so the last word about a modifier always wins.

The same three-way distinction appears in `Resolved.glyph`, which is a
`Slot<GlyphRole>`, and a component is expected to honour all three: `Set(g)`
paints `g`, `Inherit` leaves the component's own fallback in control, and
`Clear` suppresses the glyph. `crates/tui/examples/12_author_component.rs`
shows the `match`.

---

## Where each concern lives

| Concern | Home |
|---|---|
| a colour | `ColorTokens` (named by a `Role`) |
| spacing, insets, gaps | `DesignTokens::space` |
| widths, heights, popup bounds | `DesignTokens::size` |
| glyphs (`›`, `✓`, spinner frames) | `DesignTokens::glyphs`, named by a `GlyphRole` |
| border glyph set | `DesignTokens::borders` (`BorderSet` = ratatui's `symbols::border::Set`) |
| tick period, press flash, double-click window, wheel rows | `DesignTokens::motion` |
| row density | `DesignTokens::density` |
| meter thresholds | `DesignTokens::meter` |
| which part is bold when focused | a `Recipe` state rule |
| the default variant of a family | `Recipe.default_variant` |
| one instance's deviation | a `StylePatch` on the instance |
| one subtree's deviation | an `Overlay` |

If you find yourself wanting a literal colour inside a component, the answer
is almost always a missing `Role`. `Role::Custom(Color)` exists, is
documented, and is still downgraded — but it is the escape hatch, not the
path.
