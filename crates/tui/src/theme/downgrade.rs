//! Capability downgrade and the mono rule (`COMPONENT_ARCHITECTURE.md` §11.4, §21 items 25, 29).
//!
//! `downgrade_color` is exact integer/float arithmetic over the 6×6×6 cube,
//! the 24-step greyscale and the 16 xterm defaults; `Theme::downgrade` maps
//! every token through `ColorTokens::map_colors`, so any theme downgrades.

use ratatui_core::style::{Color, Modifier};

use super::Theme;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::Recipes;
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

const fn named_of(i: usize) -> Color {
    match i {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        _ => Color::White,
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

/// Nearest of the 16 xterm defaults by CIE76 ΔE.
fn nearest_16(rgb: (u8, u8, u8)) -> Color {
    let lab = lab_of(rgb);
    let mut best = (15usize, f64::MAX);
    for (i, c) in XTERM16.iter().enumerate() {
        let l = lab_of(*c);
        let de = ((lab.0 - l.0).powi(2) + (lab.1 - l.1).powi(2) + (lab.2 - l.2).powi(2)).sqrt();
        if de < best.1 {
            best = (i, de);
        }
    }
    named_of(best.0)
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
fn mono_rules() -> [(Part, StateFlags, StylePatch); 13] {
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
                ..p().set_fg(Role::Fg(FgStep::Faint)).remove(Modifier::all())
            },
        ),
        (
            Part::LABEL,
            StateFlags::DISABLED,
            p().set_fg(Role::Fg(FgStep::Faint))
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

/// The number of rules `apply_mono_fallbacks` appends per family.
pub const MONO_RULES_PER_FAMILY: usize = 16;

impl Recipes {
    /// Append the §11.4 mono rules to every family, so state survives
    /// without hue. A mono `PRESSED` label whose glyph resolves to
    /// `PressLeft` is painted bracketed: `PressLeft`, label, `PressRight`.
    pub fn apply_mono_fallbacks(&mut self) {
        let rules = mono_rules();
        let extra = mono_rules_extra();
        for (_, recipe) in self.iter_mut() {
            for (part, when, patch) in rules.iter().chain(extra.iter()) {
                recipe.parts.entry(*part).when(*when, *patch);
            }
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
        // CIE76 nearest of the xterm defaults (§21 item 29): Junie's accent and
        // error land on the dark pair, the pure primaries on the light pair
        assert_eq!(
            downgrade_color(Color::Rgb(0x48, 0xe0, 0x54), ColorLevel::Ansi16),
            Color::Green
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0xe4, 0x45, 0x45), ColorLevel::Ansi16),
            Color::Red
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
        // greyscale wins over the cube for a mid grey, ties to the lower index
        assert_eq!(
            downgrade_color(Color::Rgb(8, 8, 8), ColorLevel::Ansi256),
            Color::Indexed(232)
        );
        assert_eq!(rgb_of_index(232), (8, 8, 8));
        assert_eq!(rgb_of_index(16), (0, 0, 0));
        assert_eq!(rgb_of_index(231), (255, 255, 255));
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

    use crate::theme::recipe::{Family, Variant};
}
