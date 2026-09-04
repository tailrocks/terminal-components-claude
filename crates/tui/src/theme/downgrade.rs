//! Capability downgrade and the mono rule (`COMPONENT_ARCHITECTURE.md` §11.4, §21 items 25, 29).
//!
//! `downgrade_color` is exact integer/float arithmetic over the 6×6×6 cube,
//! the 24-step greyscale and the 16 xterm defaults; `Theme::downgrade` maps
//! every token through `ColorTokens::map_colors`, so any theme downgrades.

use ratatui_core::style::{Color, Modifier};

use super::Theme;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Recipe, Recipes};
use super::role::{FgStep, Role, Surface};
use super::tokens::ColorLevel;
use crate::id::Part;
use crate::response::StateFlags;

/// The 16 xterm default colours as `(r, g, b)`, indexed like `Color::Indexed(0..16)`.
const XTERM16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

const fn named_index(c: Color) -> Option<usize> {
    match c {
        Color::Black => Some(0),
        Color::Red => Some(1),
        Color::Green => Some(2),
        Color::Yellow => Some(3),
        Color::Blue => Some(4),
        Color::Magenta => Some(5),
        Color::Cyan => Some(6),
        Color::Gray => Some(7),
        Color::DarkGray => Some(8),
        Color::LightRed => Some(9),
        Color::LightGreen => Some(10),
        Color::LightYellow => Some(11),
        Color::LightBlue => Some(12),
        Color::LightMagenta => Some(13),
        Color::LightCyan => Some(14),
        Color::White => Some(15),
        Color::Reset | Color::Rgb(..) | Color::Indexed(_) => None,
    }
}

/// The cube channel value for a cube step `0..6`.
const fn cube_value(step: u8) -> u8 {
    if step == 0 {
        0
    } else {
        55u8.saturating_add(step.saturating_mul(40))
    }
}

/// The `(r, g, b)` of a 256-palette index.
fn rgb_of_index(i: u8) -> (u8, u8, u8) {
    if let Some(&c) = XTERM16.get(usize::from(i)) {
        return c;
    }
    if i >= 232 {
        let v = 8u8.saturating_add(i.saturating_sub(232).saturating_mul(10));
        return (v, v, v);
    }
    let n = i.saturating_sub(16);
    (
        cube_value(n / 36),
        cube_value((n / 6) % 6),
        cube_value(n % 6),
    )
}

/// The reference `(r, g, b)` of any colour, `None` for `Reset`.
pub(crate) fn rgb_of(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(i) => Some(rgb_of_index(i)),
        Color::Reset => None,
        named => named_index(named).and_then(|i| XTERM16.get(i).copied()),
    }
}

fn sq_dist(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let d = |x: u8, y: u8| {
        let v = i32::from(x).saturating_sub(i32::from(y));
        v.saturating_mul(v) as u32
    };
    d(a.0, b.0)
        .saturating_add(d(a.1, b.1))
        .saturating_add(d(a.2, b.2))
}

/// Nearest of the 6×6×6 cube ∪ the 24-step greyscale by squared sRGB
/// distance; ties resolve to the lower index.
fn nearest_256(rgb: (u8, u8, u8)) -> u8 {
    let mut best = (16u8, u32::MAX);
    for i in 16..=255u8 {
        let d = sq_dist(rgb, rgb_of_index(i));
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

fn to_lin(c: u8) -> f64 {
    let v = f64::from(c) / 255.0;
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// CIE L*a*b* under D65.
pub(crate) fn lab_of(rgb: (u8, u8, u8)) -> (f64, f64, f64) {
    let (red, green, blue) = (to_lin(rgb.0), to_lin(rgb.1), to_lin(rgb.2));
    let x = (red * 0.412_456_4 + green * 0.357_576_1 + blue * 0.180_437_5) / 0.950_47;
    let y = red * 0.212_672_9 + green * 0.715_152_2 + blue * 0.072_175_0;
    let z = (red * 0.019_333_9 + green * 0.119_192_0 + blue * 0.950_304_1) / 1.088_83;
    let f = |t: f64| {
        if t > 0.008_856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

/// Nearest of the 16 xterm defaults by **hue family and brightness class**
/// (`DESIGN.md:320`), not by perceptual distance.
///
/// A colour whose channel spread is under 40 collapses to the grey ladder by
/// ITU-R BT.601 luma (`≤30 Black`, `≤110 DarkGray`, `≤200 Gray`, else
/// `White`); otherwise the dominant channel selects the hue family (with
/// `r ≥ g,b ∧ g > 120 ∧ b < 80` reading as Yellow) and `max(r,g,b) > 180`
/// selects the light half.
///
/// Recorded rejection: nearest-by-CIE76 ΔE. It is the more "correct"
/// perceptual answer and the wrong design answer — it maps Junie's accent
/// `#48e054` and error `#e44545` into the *dark* half, discarding the
/// brightness contrast the accent system rests on, and collapses
/// `danger_soft` onto a grey. `DESIGN.md:320` fixes the outcome (accent
/// `LightGreen`, error `LightRed`) and the authority order puts it above the
/// implementation spec.
fn nearest_16(rgb: (u8, u8, u8)) -> Color {
    let (r, g, b) = rgb;
    let lum = (u32::from(r)
        .saturating_mul(299)
        .saturating_add(u32::from(g).saturating_mul(587))
        .saturating_add(u32::from(b).saturating_mul(114)))
        / 1000;
    let max = u32::from(r.max(g).max(b));
    let min = u32::from(r.min(g).min(b));
    if max.saturating_sub(min) < 40 {
        return match lum {
            0..=30 => Color::Black,
            31..=110 => Color::DarkGray,
            111..=200 => Color::Gray,
            _ => Color::White,
        };
    }
    let bright = max > 180;
    match (r >= g && r >= b, g >= r && g >= b) {
        (true, _) if g > 120 && b < 80 => Color::Yellow,
        (true, _) => {
            if bright {
                Color::LightRed
            } else {
                Color::Red
            }
        }
        (_, true) => {
            if bright {
                Color::LightGreen
            } else {
                Color::Green
            }
        }
        _ => {
            if bright {
                Color::LightBlue
            } else {
                Color::Blue
            }
        }
    }
}

/// Relative luminance `Y = 0.2126R + 0.7152G + 0.0722B` over linear channels.
pub(crate) fn luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * to_lin(rgb.0) + 0.7152 * to_lin(rgb.1) + 0.0722 * to_lin(rgb.2)
}

/// Mono: `Y < 0.35 → Black`, `Y > 0.75 → White`, else `Reset`.
fn mono(rgb: (u8, u8, u8)) -> Color {
    let y = luminance(rgb);
    if y < 0.35 {
        Color::Black
    } else if y > 0.75 {
        Color::White
    } else {
        Color::Reset
    }
}

/// Map a colour to a capability level (§21 item 29, exact).
pub fn downgrade_color(c: Color, level: ColorLevel) -> Color {
    let Some(rgb) = rgb_of(c) else {
        return c;
    };
    match level {
        ColorLevel::TrueColor => c,
        ColorLevel::Ansi256 => match c {
            Color::Rgb(..) => Color::Indexed(nearest_256(rgb)),
            other => other,
        },
        ColorLevel::Ansi16 => match c {
            Color::Rgb(..) | Color::Indexed(_) => nearest_16(rgb),
            other => other,
        },
        ColorLevel::Mono => mono(rgb),
    }
}

impl Theme {
    /// Every token mapped through [`downgrade_color`]; at `Mono` the mono
    /// fallback rules are appended (§11.4). Works for any theme.
    #[must_use]
    pub fn downgrade(&self, level: ColorLevel) -> Theme {
        let mut out = self.clone();
        out.capability.color = level;
        out.color = self.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == ColorLevel::Mono {
            out.recipes.apply_mono_fallbacks();
        }
        out
    }
}

/// The mono fallback rules, one per state, appended to every family.
fn mono_rules() -> [(Part, StateFlags, StylePatch); 15] {
    let p = StylePatch::new;
    [
        (
            Part::GUTTER,
            StateFlags::FOCUSED,
            p().set_glyph(GlyphRole::FocusBar),
        ),
        (Part::LABEL, StateFlags::FOCUSED, p().add(Modifier::BOLD)),
        (
            Part::MARKER,
            StateFlags::SELECTED,
            p().set_glyph(GlyphRole::Chosen),
        ),
        (
            Part::MARKER,
            StateFlags::CHECKED,
            p().set_glyph(GlyphRole::Checked),
        ),
        (
            Part::CONTAINER,
            StateFlags::PRESSED,
            p().set_bg(Role::Fg(FgStep::Primary))
                .set_fg(Role::Surface(Surface::Canvas))
                .add(Modifier::BOLD),
        ),
        (
            Part::LABEL,
            StateFlags::PRESSED,
            p().set_glyph(GlyphRole::PressLeft).add(Modifier::BOLD),
        ),
        (
            Part::GUTTER,
            StateFlags::DISABLED,
            StylePatch {
                glyph: Slot::Clear,
                ..p()
            },
        ),
        (
            Part::MARKER,
            StateFlags::DISABLED,
            StylePatch {
                glyph: Slot::Clear,
                ..p()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .remove(Modifier::all())
            },
        ),
        (
            Part::LABEL,
            StateFlags::DISABLED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        // The two parts a *text* control paints for its own content. Without
        // them a disabled `TextInput` is indistinguishable from an enabled one
        // at `Mono` (§29, MA-8): `PARTS` is `FIELD, TEXT, PLACEHOLDER, MARKER,
        // GUTTER`, and no mono rule reached the first two. `PLACEHOLDER` needs
        // none — it is painted over the `FIELD` fill and inherits its
        // modifiers per cell; `CONTAINER` needs none — a text control fills
        // `FIELD`, so a `CONTAINER` rule would not reach the defect.
        //
        // Declaration order is load-bearing: state rules of equal specificity
        // apply in declaration order, so these `remove(Modifier::all())` rules
        // must precede the `ERROR` rules below or `ERROR`'s `UNDERLINED` is
        // erased.
        (
            Part::FIELD,
            StateFlags::DISABLED,
            // `Fg(Primary)`, NOT `Fg(Faint)`: `mono()` maps every step below
            // `Y = 0.35` to `Black`, and both `disabled_fg` and `Fg(Faint)`
            // are below it — on a `Black` canvas §11.4's prescribed faint
            // foreground is invisible, not merely colourless.
            p().set_fg(Role::Fg(FgStep::Primary))
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        (
            Part::TEXT,
            StateFlags::DISABLED,
            p().set_fg(Role::Fg(FgStep::Primary))
                .remove(Modifier::all())
                .add(Modifier::DIM),
        ),
        (
            Part::MARKER,
            StateFlags::ERROR,
            p().set_glyph(GlyphRole::Error),
        ),
        (
            Part::FIELD,
            StateFlags::ERROR,
            p().add(Modifier::UNDERLINED),
        ),
        (
            Part::MARKER,
            StateFlags::WARNING,
            p().set_glyph(GlyphRole::Dirty),
        ),
        (
            Part::TEXT,
            StateFlags::EDITING,
            p().add(Modifier::UNDERLINED),
        ),
    ]
}

/// Rules that share a slot with the table above but are keyed on a second
/// flag (`DIRTY` beside `WARNING`, `ACTIVE` for tabs).
fn mono_rules_extra() -> [(Part, StateFlags, StylePatch); 3] {
    let p = StylePatch::new;
    [
        (
            Part::MARKER,
            StateFlags::DIRTY,
            p().set_glyph(GlyphRole::Dirty),
        ),
        (
            Part::RULE,
            StateFlags::ACTIVE,
            p().set_glyph(GlyphRole::RuleActive),
        ),
        (Part::LABEL, StateFlags::ACTIVE, p().add(Modifier::BOLD)),
    ]
}

/// The number of rules `apply_mono_fallbacks` appends per **resolvable
/// recipe** — every declared family *and* the neutral recipe that undeclared
/// families resolve through (`Recipes`' resolvable-set invariant).
///
/// The name is historical (§16.1 cites it) and is kept deliberately: the
/// count is unchanged, only the set it applies to is stated correctly.
pub const MONO_RULES_PER_FAMILY: usize = 18;

/// Append the §11.4 mono rules to one recipe.
///
/// Split out of [`Recipes::apply_mono_fallbacks`] so the rules are written
/// once and the *enumeration* of recipes is a separate, auditable decision:
/// the enumeration is what previously omitted the neutral recipe.
fn apply_mono_fallbacks_to(recipe: &mut Recipe) {
    let rules = mono_rules();
    let extra = mono_rules_extra();
    for (part, when, patch) in rules.iter().chain(extra.iter()) {
        recipe.parts.entry(*part).when(*when, *patch);
    }
    // MI-13: the fallbacks must reach the variant maps too. §11.3's
    // step 3 merges family and variant state rules in one specificity
    // order, so a variant that re-declares `PRESSED` would otherwise
    // be applied after the family's mono rule and erase the bracket
    // glyph that makes `pressed` distinguishable without colour.
    for (_, map) in &mut recipe.variants {
        for (part, when, patch) in rules.iter().chain(extra.iter()) {
            if map.get(*part).is_some() {
                map.entry(*part).when(*when, *patch);
            }
        }
    }
}

impl Recipes {
    /// Append the §11.4 mono rules to every recipe resolution can reach, so
    /// state survives without hue. A mono `PRESSED` label whose glyph
    /// resolves to `PressLeft` is painted bracketed: `PressLeft`, label,
    /// `PressRight`.
    ///
    /// "Every recipe resolution can reach" is `Recipes::resolvable_mut`: the
    /// declared families **and** the neutral recipe. An undeclared
    /// `Family::custom` paints through the neutral recipe, so covering only
    /// the declared families left it with no non-colour state signal at all
    /// (F1).
    ///
    /// Not idempotent: each call appends its rules unconditionally, so a
    /// chained `.downgrade(Mono).downgrade(Mono)` doubles them. Harmless in
    /// effect — the duplicates are identical patches applied in order — but
    /// it means rule counts are only meaningful against a single downgrade.
    pub fn apply_mono_fallbacks(&mut self) {
        for recipe in self.resolvable_mut() {
            apply_mono_fallbacks_to(recipe);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVELS: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];

    #[test]
    fn downgrade_maps_every_token_exhaustively() {
        let t = Theme::junie();
        let d = t.downgrade(ColorLevel::Ansi256);
        for c in d.color.colors() {
            assert!(!matches!(c, Color::Rgb(..)), "{c:?} survived the downgrade");
        }
        let m = t.downgrade(ColorLevel::Mono);
        for c in m.color.colors() {
            assert!(
                matches!(c, Color::Black | Color::White | Color::Reset),
                "{c:?}"
            );
        }
        assert_eq!(m.capability.color, ColorLevel::Mono);
    }

    #[test]
    fn downgrade_works_for_a_user_supplied_theme() {
        let t = Theme::paper();
        let d16 = t.downgrade(ColorLevel::Ansi16);
        for c in d16.color.colors() {
            assert!(!matches!(c, Color::Rgb(..) | Color::Indexed(_)), "{c:?}");
        }
        assert_eq!(d16.color.surfaces.first().copied(), Some(Color::White));
    }

    #[test]
    fn downgrade_is_deterministic_per_level() {
        let t = Theme::junie();
        for level in LEVELS {
            assert_eq!(t.downgrade(level), t.downgrade(level));
        }
        // the exact metrics of §21 item 29
        assert_eq!(
            downgrade_color(Color::Rgb(0x48, 0xe0, 0x54), ColorLevel::Ansi256),
            Color::Indexed(77)
        );
        // the categorical hue-family metric (DESIGN.md:320): Junie's accent
        // and error keep their hue *and* their brightness class
        assert_eq!(
            downgrade_color(Color::Rgb(0x48, 0xe0, 0x54), ColorLevel::Ansi16),
            Color::LightGreen
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0xe4, 0x45, 0x45), ColorLevel::Ansi16),
            Color::LightRed
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0xf5, 0x9e, 0x09), ColorLevel::Ansi16),
            Color::Yellow
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0, 0, 0), ColorLevel::Mono),
            Color::Black
        );
        assert_eq!(
            downgrade_color(Color::Rgb(255, 255, 255), ColorLevel::Mono),
            Color::White
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x80, 0x80, 0x80), ColorLevel::Mono),
            Color::Black
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0xb3, 0xb3, 0xb3), ColorLevel::Mono),
            Color::Reset
        );
        assert_eq!(
            downgrade_color(Color::Reset, ColorLevel::Mono),
            Color::Reset
        );
        assert_eq!(
            downgrade_color(Color::Indexed(196), ColorLevel::Ansi16),
            Color::LightRed
        );
        // a near-grey collapses to the grey ladder, not to a hue
        assert_eq!(
            downgrade_color(Color::Rgb(0x26, 0x26, 0x26), ColorLevel::Ansi16),
            Color::DarkGray
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x11, 0x11, 0x11), ColorLevel::Ansi16),
            Color::Black
        );
        // greyscale wins over the cube for a mid grey, ties to the lower index
        assert_eq!(
            downgrade_color(Color::Rgb(8, 8, 8), ColorLevel::Ansi256),
            Color::Indexed(232)
        );
        assert_eq!(rgb_of_index(232), (8, 8, 8));
        assert_eq!(rgb_of_index(16), (0, 0, 0));
        assert_eq!(rgb_of_index(231), (255, 255, 255));
    }

    /// `DESIGN.md:320` is the contract: a 16-colour downgrade preserves hue
    /// family and brightness class, it does not minimise perceptual distance.
    #[test]
    fn ansi16_preserves_hue_family_and_brightness() {
        let c = Theme::junie().color;
        let at16 = |x| downgrade_color(x, ColorLevel::Ansi16);
        // DESIGN.md:320 — "at 16 colours the accent is LightGreen and error is LightRed"
        assert_eq!(at16(c.accent), Color::LightGreen);
        assert_eq!(at16(c.danger), Color::LightRed);
        // a destructive label at rest stays red rather than collapsing to grey
        assert_eq!(at16(c.danger_soft), Color::LightRed);
        // the grey ladder: subtle chrome is grey, never a hue. `border_subtle`
        // is `#262626`, BT.601 luma 38, so it lands on `DarkGray` — the
        // review's "Black" was an unverified estimate; `#111111` (luma 17) is
        // the colour that reaches `Black`.
        assert_eq!(at16(c.border_subtle), Color::DarkGray);
        assert_eq!(at16(c.surfaces[1]), Color::Black);
        assert_eq!(at16(c.fg[1]), Color::Gray);
        assert_eq!(at16(c.fg[0]), Color::White);
        assert_eq!(at16(c.warning), Color::Yellow);
        // the dark half is reachable: the same hue at low brightness
        assert_eq!(at16(Color::Rgb(0x2b, 0x86, 0x32)), Color::Green);
        assert_eq!(at16(Color::Rgb(0x7a, 0x2a, 0x2a)), Color::Red);
        assert_eq!(at16(c.info), Color::LightBlue);
    }

    #[test]
    fn mono_appends_one_state_rule_per_family() {
        let t = Theme::junie();
        let m = t.downgrade(ColorLevel::Mono);
        for (f, before) in t.recipes.iter() {
            let after = m.recipes.get(f).map_or(0, |r| {
                r.parts.iter().map(|(_, p)| p.states.len()).sum::<usize>()
            });
            let base: usize = before.parts.iter().map(|(_, p)| p.states.len()).sum();
            assert_eq!(after.saturating_sub(base), MONO_RULES_PER_FAMILY, "{f:?}");
        }
        // CHECKED wins over SELECTED when both are live
        let acc = crate::theme::resolve::accumulate(
            &m,
            Family::LIST,
            Variant::DEFAULT,
            Part::MARKER,
            StateFlags::SELECTED | StateFlags::CHECKED,
            &[],
        );
        assert_eq!(acc.glyph, Slot::Set(GlyphRole::Checked));
        let pressed = crate::theme::resolve::accumulate(
            &m,
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::PRESSED,
            &[],
        );
        assert_eq!(pressed.glyph, Slot::Set(GlyphRole::PressLeft));
        assert!(pressed.add.contains(Modifier::BOLD));
    }

    /// F1: the mono fallbacks must reach the **neutral** recipe, not only the
    /// declared families. `Recipes::get_or_neutral` makes the neutral recipe a
    /// live painting path for every family nobody declared — exactly what
    /// `examples/12_author_component.rs` writes with `Family::custom` and no
    /// `define_family` — so a pass that enumerates `by_family` alone leaves
    /// that path with no non-colour state signal at all at `Mono`.
    ///
    /// `mono_appends_one_state_rule_per_family` is structurally unable to see
    /// this: it iterates `t.recipes.iter()`, the same `by_family`-only
    /// enumeration as the code it checks. The assertions below therefore go
    /// through `Theme::resolve`, the path painting uses, so a fix that only
    /// inflates a rule counter cannot satisfy them.
    #[test]
    fn mono_fallbacks_reach_the_neutral_recipe() {
        let states =
            |r: &Recipe| -> usize { r.parts.iter().map(|(_, p)| p.states.len()).sum::<usize>() };
        let f = Family::custom("segmented");
        for base in [Theme::junie(), Theme::paper()] {
            // The fixture must stay undeclared, or the test silently stops
            // exercising the neutral path and asserts nothing.
            assert!(
                base.recipes.get(f).is_none(),
                "`segmented` is now a declared family; pick an undeclared fixture"
            );
            let m = base.downgrade(ColorLevel::Mono);
            assert!(m.recipes.get(f).is_none(), "`segmented` became declared");

            assert_eq!(
                states(m.recipes.neutral()).saturating_sub(states(base.recipes.neutral())),
                MONO_RULES_PER_FAMILY,
                "the neutral recipe did not receive the mono fallbacks"
            );

            // …and they are observable where it counts.
            let at = |p: Part, s: StateFlags| m.resolve(f, Variant::DEFAULT, p, s, Surface::Canvas);
            assert!(
                at(Part::LABEL, StateFlags::FOCUSED)
                    .style
                    .add_modifier
                    .contains(Modifier::BOLD),
                "mono FOCUSED label of an undeclared family carries no BOLD"
            );
            assert_eq!(
                at(Part::MARKER, StateFlags::ERROR).glyph,
                Slot::Set(GlyphRole::Error),
                "mono ERROR marker of an undeclared family has no glyph"
            );

            // Expressed as DIM plus "not the canvas colour" rather than as a
            // literal token, so it stays a readability assertion instead of a
            // theme-palette snapshot.
            let canvas = crate::theme::resolve::bind_role(
                &m,
                Role::Surface(Surface::Canvas),
                Surface::Canvas,
            );
            let disabled = at(Part::LABEL, StateFlags::DISABLED);
            assert!(
                disabled.style.add_modifier.contains(Modifier::DIM),
                "mono DISABLED label of an undeclared family carries no DIM"
            );
            assert!(
                disabled.style.fg.is_some() && disabled.style.fg != canvas,
                "mono DISABLED label of an undeclared family has fg {:?}, the canvas {canvas:?}",
                disabled.style.fg
            );
        }
    }

    /// §29 + §11.4: at `Mono` a disabled control must stay **readable**, not
    /// merely colourless. The rule that produced `Fg(Faint)` here resolved to
    /// `Black` on a `Black` canvas — invisible — because `mono()` collapses
    /// every step below `Y = 0.35` onto the background.
    #[test]
    fn mono_disabled_is_dim_and_readable() {
        for base in [Theme::junie(), Theme::paper()] {
            let m = base.downgrade(ColorLevel::Mono);
            let canvas = crate::theme::resolve::bind_role(
                &m,
                Role::Surface(Surface::Canvas),
                Surface::Canvas,
            );
            for f in [Family::INPUT, Family::FIELD, Family::LIST, Family::BUTTON] {
                for p in [Part::FIELD, Part::TEXT, Part::LABEL] {
                    let r = crate::theme::resolve::resolve_uncached(
                        &m,
                        f,
                        Variant::DEFAULT,
                        p,
                        StateFlags::DISABLED,
                        Surface::Canvas,
                        &[],
                        None,
                    );
                    assert!(
                        r.style.add_modifier.contains(Modifier::DIM),
                        "{f:?}/{p:?}: mono DISABLED carries no DIM"
                    );
                    assert!(
                        r.style.fg.is_some() && r.style.fg != canvas,
                        "{f:?}/{p:?}: mono DISABLED fg {:?} is the canvas {canvas:?}",
                        r.style.fg
                    );
                }
            }
        }
    }

    use crate::theme::recipe::{Family, Variant};
}
