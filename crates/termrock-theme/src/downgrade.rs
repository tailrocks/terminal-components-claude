#![doc = r" Capability downgrade and the mono rule (`COMPONENT_ARCHITECTURE.md` §11.4, §21 items 25, 29)."]
#![doc = r""]
#![doc = r" `downgrade_color` is exact integer/float arithmetic over the 6×6×6 cube,"]
#![doc = r" the 24-step greyscale and the 16 xterm defaults; `Theme::downgrade` maps"]
#![doc = r" every token through `ColorTokens::map_colors`, then protects the"]
#![doc = r" foreground ladder of light themes from ANSI16's bright grey entries."]
#![doc = r" Explicit authored capability tables compose over that generic result by"]
#![doc = r" semantic slot and source provenance; they never change RGB conversion."]

use ratatui_core::style::{Color, Modifier};

use super::Theme;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Recipes};
use super::role::{FgStep, Role, Surface};
use super::tokens::{ColorLevel, ColorTokens};
use crate::id::Part;
use crate::response::StateFlags;

#[doc = r" The 16 xterm default colours as `(r, g, b)`, indexed like `Color::Indexed(0..16)`."]
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

#[doc = r" The cube channel value for a cube step `0..6`."]
const fn cube_value(step: u8) -> u8 {
    if step == 0 {
        0
    } else {
        55u8.saturating_add(step.saturating_mul(40))
    }
}

#[doc = r" The `(r, g, b)` of a 256-palette index."]
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

#[doc = r" The reference `(r, g, b)` of any colour, `None` for `Reset`."]
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

#[doc = r" Nearest of the 6×6×6 cube ∪ the 24-step greyscale by squared sRGB"]
#[doc = r" distance; ties resolve to the lower index."]
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

#[doc = r" CIE L*a*b* under D65."]
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

#[doc = r" Nearest of the 16 xterm defaults by **hue family and brightness class**"]
#[doc = r" (`DESIGN.md:320`), not by perceptual distance."]
#[doc = r""]
#[doc = r" A colour whose channel spread is under 40 collapses to the grey ladder by"]
#[doc = r" ITU-R BT.601 luma (`≤30 Black`, `≤110 DarkGray`, `≤200 Gray`, else"]
#[doc = r" `White`); otherwise the dominant channel selects the hue family (with"]
#[doc = r" `r ≥ g,b ∧ g > 120 ∧ b < 80` reading as Yellow) and `max(r,g,b) > 180`"]
#[doc = r" selects the light half."]
#[doc = r""]
#[doc = r#" Recorded rejection: nearest-by-CIE76 ΔE. It is the more "correct""#]
#[doc = r" perceptual answer and the wrong design answer — it maps Junie's accent"]
#[doc = r" `#48e054` and error `#e44545` into the *dark* half, discarding the"]
#[doc = r" brightness contrast the accent system rests on, and collapses"]
#[doc = r" `danger_soft` onto a grey. `DESIGN.md:320` fixes the outcome (accent"]
#[doc = r" `LightGreen`, error `LightRed`) and the authority order puts it above the"]
#[doc = r" implementation spec."]
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

#[doc = r" Relative luminance `Y = 0.2126R + 0.7152G + 0.0722B` over linear channels."]
pub(crate) fn luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * to_lin(rgb.0) + 0.7152 * to_lin(rgb.1) + 0.0722 * to_lin(rgb.2)
}

#[doc = r" Mono: `Y < 0.35 → Black`, `Y > 0.75 → White`, else `Reset`."]
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

#[doc = r" Map a colour to a capability level (§21 item 29, exact)."]
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

#[doc = r" Keep semantic foreground text legible when a light theme enters the tiny"]
#[doc = r" ANSI16 palette. The palette's `Gray`/`White` entries are intentionally"]
#[doc = r" bright; on a light canvas they erase the foreground/background contrast."]
#[doc = r" This post-map is limited to the foreground ladder, preserving hue mapping"]
#[doc = r" for accents and every other token while keeping the downgrade generic for"]
#[doc = r" user-supplied light themes."]
fn repair_ansi16_light_foreground(source: &ColorTokens, mapped: &mut ColorTokens) {
    let Some(canvas) = rgb_of(source.surfaces[0]) else {
        return;
    };
    if luminance(canvas) < 0.5 {
        return;
    }
    for foreground in &mut mapped.fg {
        if matches!(*foreground, Color::Gray | Color::White) {
            *foreground = Color::DarkGray;
        }
    }
}

struct PaletteProjection<'a> {
    source: &'a [super::MeterFillRest],
    target: &'a [super::MeterFillRest],
    generic: &'a [super::MeterFillRest],
    eligible: &'a mut [bool],
    index: usize,
    level: ColorLevel,
}
impl PaletteProjection<'_> {
    fn project(&mut self, current: super::MeterFillRest) -> super::MeterFillRest {
        let i = self.index;
        self.index = self.index.saturating_add(1);
        if let Some((((expected, authored), generic), eligible)) = self
            .source
            .get(i)
            .zip(self.target.get(i))
            .zip(self.generic.get(i))
            .zip(self.eligible.get_mut(i))
        {
            if current != *expected {
                *eligible = false;
            }
            if *eligible { *authored } else { *generic }
        } else {
            match current {
                super::MeterFillRest::Color(color) => {
                    super::MeterFillRest::Color(downgrade_color(color, self.level))
                }
                super::MeterFillRest::ReferenceLift => current,
            }
        }
    }
}
impl super::tokens::TokenMapper for PaletteProjection<'_> {
    fn color(&mut self, current: Color) -> Color {
        match self.project(super::MeterFillRest::Color(current)) {
            super::MeterFillRest::Color(color) => color,
            super::MeterFillRest::ReferenceLift => downgrade_color(current, self.level),
        }
    }
    fn meter_rest(&mut self, current: super::MeterFillRest) -> super::MeterFillRest {
        self.project(current)
    }
}

impl Theme {
    #[doc = r" Every token mapped through [`downgrade_color`]; at `Mono` the mono"]
    #[doc = r" fallback rules are applied by resolution (§11.4). Works for any theme."]
    #[doc = r""]
    #[doc = r" A theme already at `level` is returned unchanged: the token map is not"]
    #[doc = r" re-run, so a theme authored at that depth — or one a caller already"]
    #[doc = r" took down to it — keeps its exact colours instead of being pushed"]
    #[doc = r" through a second, lossy approximation of colours that are already"]
    #[doc = r" representable."]
    #[must_use]
    pub fn downgrade(&self, level: ColorLevel) -> Theme {
        if self.capability.color == level {
            return self.clone();
        }
        let mut out = self.clone();
        out.capability.color = level;
        out.color = self.color.map_colors(&mut |c| downgrade_color(c, level));
        if level == ColorLevel::Ansi16 {
            repair_ansi16_light_foreground(&self.color, &mut out.color);
        }
        if let Some(palettes) = &self.capability_palettes {
            let expected = palettes
                .projected
                .filter(|(level, _)| *level == self.capability.color)
                .map(|(_, tokens)| tokens)
                .or_else(|| palettes.get(self.capability.color))
                .unwrap_or_else(|| {
                    let mut expected = palettes
                        .source
                        .map_colors(&mut |c| downgrade_color(c, self.capability.color));
                    if self.capability.color == ColorLevel::Ansi16 {
                        repair_ansi16_light_foreground(&palettes.source, &mut expected);
                    }
                    expected
                });
            let source = expected.semantic_colors();
            let target = palettes.get(level).unwrap_or(out.color).semantic_colors();
            let generic = out.color.semantic_colors();
            let mut eligibility = palettes.eligible.clone();
            out.color = self.color.map_with(&mut PaletteProjection {
                source: &source,
                target: &target,
                generic: &generic,
                eligible: &mut eligibility,
                index: 0,
                level,
            });
            if let Some(palettes) = &mut out.capability_palettes {
                let palettes = std::sync::Arc::make_mut(palettes);
                palettes.eligible = eligibility;
                palettes.projected = Some((level, out.color));
            }
        }
        out
    }

    #[doc = r" This theme narrowed to what `detected` can paint. Pure: the testable"]
    #[doc = r" seam under [`Theme::for_terminal`] (§34.3)."]
    #[doc = r""]
    #[doc = r" Narrows and **never widens**, because `capability.color` is the depth"]
    #[doc = r" the theme's tokens are actually at, not a request. A plain"]
    #[doc = r" `downgrade(detected)` would raise the field on a theme a caller had"]
    #[doc = r" deliberately taken down to `Mono`, leaving it claiming `TrueColor` over"]
    #[doc = r" black-and-white tokens — a field lying about its own content, which then"]
    #[doc = r" mis-binds [`Role::Custom`]."]
    #[must_use]
    pub fn for_level(&self, detected: ColorLevel) -> Theme {
        self.downgrade(self.capability.color.narrow_to(detected))
    }

    #[doc = r" [`Theme::for_level`] at [`ColorLevel::detect`] — the one impure call,"]
    #[doc = r" made once by `run` when the terminal is finally known."]
    #[must_use]
    pub fn for_terminal(&self) -> Theme {
        self.for_level(ColorLevel::detect())
    }
}

#[doc = r" One entry of the §11.4 mono fallback manifest: the part it targets, the"]
#[doc = r" state that arms it, and the patch merged when that state is live."]
#[doc = r""]
#[doc = r" The shape a theme author passes to"]
#[doc = r" [`ThemeBuilder::mono_rules`](super::ThemeBuilder::mono_rules), and the"]
#[doc = r" shape the built-in tables below are written in."]
pub type MonoRule = (Part, StateFlags, StylePatch);

#[doc = r" The immutable generic mono fallback manifest applied by the resolver."]
fn mono_rules() -> [MonoRule; 15] {
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
        (
            Part::FIELD,
            StateFlags::DISABLED,
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

#[doc = r" Rules that share a slot with the table above but are keyed on a second"]
#[doc = r" flag (`DIRTY` beside `WARNING`, `ACTIVE` for tabs), plus the scrollbar"]
#[doc = r" half of `PRESSED`."]
fn mono_rules_extra() -> [MonoRule; 5] {
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
        (
            Part::ICON,
            StateFlags::LOADING,
            p().add(Modifier::UNDERLINED),
        ),
        (Part::THUMB, StateFlags::PRESSED, p().add(Modifier::BOLD)),
    ]
}

#[doc = r" The number of **generic** entries in the static mono fallback manifest:"]
#[doc = r" `mono_rules()` plus `mono_rules_extra()`, applied once after family and"]
#[doc = r" variant states to every resolvable recipe — each `by_family` entry and the"]
#[doc = r" neutral recipe — without touching recipe storage."]
#[doc = r""]
#[doc = r" The name is historical and is **not** a per-family total: it is cited by"]
#[doc = r" name in §16.1 and §20.10 item 18, so it keeps it. Six built-in families"]
#[doc = r" declare targeted rules on top of these (`VIEWPORT` 1, `GRID` 2, `MENU` 2,"]
#[doc = r" `HELP` 2, `PICKER` 3, `SELECT` 3) — `PICKER`'s `(LABEL, PRESSED)` retargets"]
#[doc = r" a pair the generic set already covers, so `PICKER` reaches 22 `(part,"]
#[doc = r" state)` pairs where `SELECT` reaches 23 — and a theme author can give any family —"]
#[doc = r" including a `Family::custom` one — its own targeted set through"]
#[doc = r" [`ThemeBuilder::mono_rules`](super::ThemeBuilder::mono_rules). The count"]
#[doc = r" that is constant across families is exactly this generic one; the total a"]
#[doc = r" given family applies is this number plus its targeted set's length."]
#[doc = r" `SCROLLBAR` is the one family that applies *fewer*: the generic"]
#[doc = r" `(CONTAINER, PRESSED)` rule is excluded for it (`apply_mono_fallback`), so"]
#[doc = r" its generic total is `MONO_RULES_PER_FAMILY - 1`."]
pub const MONO_RULES_PER_FAMILY: usize = 20;

fn viewport_mono_rules() -> [MonoRule; 1] {
    [(
        Part::TEXT,
        StateFlags::SELECTED,
        StylePatch::new()
            .set_fg(Role::Surface(Surface::Canvas))
            .set_bg(Role::Fg(FgStep::Primary))
            .add(Modifier::UNDERLINED),
    )]
}

fn grid_mono_rules() -> [MonoRule; 2] {
    [
        (
            Part::CELL,
            StateFlags::ERROR,
            StylePatch::new().add(Modifier::UNDERLINED),
        ),
        (
            Part::ROW,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
    ]
}

fn menu_mono_rules() -> [MonoRule; 2] {
    [
        (
            Part::ROW,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
        (
            Part::TITLE,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_glyph(GlyphRole::PressLeft)
                .add(Modifier::BOLD),
        ),
    ]
}

fn help_mono_rules() -> [MonoRule; 2] {
    [
        (
            Part::BORDER,
            StateFlags::FOCUSED,
            StylePatch::new().add(Modifier::BOLD),
        ),
        (
            Part::TITLE,
            StateFlags::FOCUSED,
            StylePatch::new().add(Modifier::UNDERLINED),
        ),
    ]
}

fn picker_mono_rules() -> [MonoRule; 3] {
    [
        (
            Part::GUTTER,
            StateFlags::ERROR,
            StylePatch::new().set_glyph(GlyphRole::Error),
        ),
        (
            Part::GUTTER,
            StateFlags::BUSY,
            StylePatch::new().set_glyph(GlyphRole::MoreRows),
        ),
        (
            Part::LABEL,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ]
}

fn select_mono_rules() -> [MonoRule; 3] {
    [
        (
            Part::FIELD,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_fg(Role::Surface(Surface::Canvas))
                .set_bg(Role::Fg(FgStep::Primary))
                .add(Modifier::BOLD),
        ),
        (
            Part::GUTTER,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_glyph(GlyphRole::PressLeft)
                .add(Modifier::BOLD),
        ),
        (
            Part::MARKER,
            StateFlags::PRESSED,
            StylePatch::new()
                .set_glyph(GlyphRole::PressRight)
                .add(Modifier::BOLD),
        ),
    ]
}

#[doc = r" Apply the private §11.4 static fallback layer after family and variant"]
#[doc = r" states, before every override layer."]
#[doc = r""]
#[doc = r" Three layers merge here, in order: the generic manifest"]
#[doc = r" ([`MONO_RULES_PER_FAMILY`] rules, family-independent), then the family's"]
#[doc = r" targeted manifest, then nothing else — the targeted manifest is the"]
#[doc = r" author's whole set for that family when the theme declares one"]
#[doc = r" ([`Recipes::mono_rules`]), and the built-in one otherwise."]
#[doc = r""]
#[doc = r" The static layer still never *writes* into recipe storage: an author's"]
#[doc = r" mono rules are in `Recipes` because the author put them there through"]
#[doc = r" [`ThemeBuilder::mono_rules`](super::ThemeBuilder::mono_rules), and"]
#[doc = r" [`Theme::downgrade`] adds none."]
pub(crate) fn apply_mono_fallback(
    acc: StylePatch,
    recipes: &Recipes,
    family: Family,
    part: Part,
    live: StateFlags,
) -> StylePatch {
    apply_mono_with_defaults(acc, recipes, family, part, live, None)
}

pub(crate) fn apply_mono_with_defaults(
    mut acc: StylePatch,
    recipes: &Recipes,
    family: Family,
    part: Part,
    live: StateFlags,
    authored: Option<&[MonoRule]>,
) -> StylePatch {
    let rules = mono_rules();
    let extra = mono_rules_extra();
    let builtin: &[MonoRule] = match family {
        Family::VIEWPORT => &viewport_mono_rules(),
        Family::GRID => &grid_mono_rules(),
        Family::MENU => &menu_mono_rules(),
        Family::HELP => &help_mono_rules(),
        Family::PICKER => &picker_mono_rules(),
        Family::SELECT => &select_mono_rules(),
        _ => &[],
    };

    let targeted: &[MonoRule] = recipes.mono_rules(family).or(authored).unwrap_or(builtin);
    for (rule_part, when, patch) in rules.iter().chain(extra.iter()).chain(targeted) {
        let applies = family != Family::SCROLLBAR
            || *rule_part != Part::CONTAINER
            || *when != StateFlags::PRESSED;
        if applies && *rule_part == part && live.contains(*when) {
            acc = acc.merge(*patch);
        }
    }
    acc
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
        let t = Theme::from_tokens(Theme::junie().color);
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

        assert_eq!(
            downgrade_color(Color::Rgb(0x48, 0xe0, 0x54), ColorLevel::Ansi256),
            Color::Indexed(77)
        );

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

        assert_eq!(
            downgrade_color(Color::Rgb(0x26, 0x26, 0x26), ColorLevel::Ansi16),
            Color::DarkGray
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x11, 0x11, 0x11), ColorLevel::Ansi16),
            Color::Black
        );

        assert_eq!(
            downgrade_color(Color::Rgb(8, 8, 8), ColorLevel::Ansi256),
            Color::Indexed(232)
        );
        assert_eq!(rgb_of_index(232), (8, 8, 8));
        assert_eq!(rgb_of_index(16), (0, 0, 0));
        assert_eq!(rgb_of_index(231), (255, 255, 255));
    }

    #[doc = r" `DESIGN.md:320` is the contract: a 16-colour downgrade preserves hue"]
    #[doc = r" family and brightness class, it does not minimise perceptual distance."]
    #[test]
    fn ansi16_preserves_hue_family_and_brightness() {
        let c = Theme::junie().color;
        let at16 = |x| downgrade_color(x, ColorLevel::Ansi16);

        assert_eq!(at16(c.accent), Color::LightGreen);
        assert_eq!(at16(c.danger), Color::LightRed);

        assert_eq!(at16(c.danger_soft), Color::LightRed);

        assert_eq!(at16(c.border_subtle), Color::DarkGray);
        assert_eq!(at16(c.surfaces[1]), Color::Black);
        assert_eq!(at16(c.fg[1]), Color::Gray);
        assert_eq!(at16(c.fg[0]), Color::White);
        assert_eq!(at16(c.warning), Color::Yellow);

        assert_eq!(at16(Color::Rgb(0x2b, 0x86, 0x32)), Color::Green);
        assert_eq!(at16(Color::Rgb(0x7a, 0x2a, 0x2a)), Color::Red);
        assert_eq!(at16(c.info), Color::LightBlue);
    }

    #[test]
    fn ansi16_light_theme_keeps_foreground_ladder_contrasting() {
        let paper = Theme::paper().downgrade(ColorLevel::Ansi16);

        assert_eq!(paper.color.fg[0], Color::Black);
        assert_eq!(paper.color.fg[1], Color::DarkGray);
        assert_eq!(paper.color.fg[2], Color::DarkGray);
        assert_eq!(paper.color.fg[3], Color::DarkGray);
        assert_eq!(paper.color.fg[4], Color::DarkGray);

        let muted = paper.resolve(
            Family::PROPS,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            Surface::Canvas,
        );
        assert_eq!(muted.style.fg, Some(Color::DarkGray));
    }

    #[doc = r" Downgrade is idempotent, and its static mono fallback layer never"]
    #[doc = r" mutates public recipe storage."]
    #[test]
    fn downgrade_is_idempotent_per_level() {
        let t = Theme::junie();
        for level in LEVELS {
            let once = t.downgrade(level);
            assert_eq!(
                once,
                once.downgrade(level),
                "downgrade({level:?}) is not idempotent"
            );
        }

        assert_eq!(t.downgrade(ColorLevel::Mono).recipes, t.recipes);
    }

    #[doc = r" §34.3: `for_level` narrows and never widens."]
    #[doc = r""]
    #[doc = r" The widening case is the one with teeth. `capability.color` is a claim"]
    #[doc = r" about the tokens, so raising it on an already-mono theme would leave the"]
    #[doc = r" field asserting `TrueColor` over black-and-white colours — and both"]
    #[doc = r" halves are checked here, because a fix that only pinned the field would"]
    #[doc = r" still be wrong if the colours moved."]
    #[test]
    fn for_level_narrows_but_never_widens() {
        let mono = Theme::from_tokens(Theme::junie().color).downgrade(ColorLevel::Mono);
        let widened = mono.for_level(ColorLevel::TrueColor);
        assert_eq!(
            widened.capability.color,
            ColorLevel::Mono,
            "for_level widened a deliberately downgraded theme"
        );
        for c in widened.color.colors() {
            assert!(
                matches!(c, Color::Black | Color::White | Color::Reset),
                "{c:?} came back after a widening for_level"
            );
        }
        assert_eq!(
            widened, mono,
            "for_level is not a no-op when it cannot narrow"
        );

        assert_eq!(
            Theme::junie().for_level(ColorLevel::Mono),
            Theme::junie().downgrade(ColorLevel::Mono),
            "for_level did not narrow a TrueColor theme to Mono"
        );

        assert_eq!(
            Theme::junie().for_level(ColorLevel::TrueColor),
            Theme::junie(),
            "for_level changed a theme already at the detected level"
        );
    }

    #[doc = r" `for_level` activates mono resolution without changing recipe storage."]
    #[test]
    fn for_level_reaches_the_neutral_recipe_once() {
        let base = Theme::junie();
        let m = base.for_level(ColorLevel::Mono);
        assert_eq!(m.capability.color, ColorLevel::Mono);
        assert_eq!(m.recipes, base.recipes);
    }

    #[test]
    fn mono_resolver_applies_once_without_recipe_storage() {
        let t = Theme::junie();
        let m = t.downgrade(ColorLevel::Mono);
        assert_eq!(
            MONO_RULES_PER_FAMILY,
            mono_rules().len() + mono_rules_extra().len()
        );
        assert_eq!(
            m.recipes, t.recipes,
            "mono fallback leaked into recipe storage"
        );

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

    #[doc = r" F1: static mono resolution must reach the neutral-backed path for"]
    #[doc = r" undeclared families without inserting anything into recipe storage."]
    #[test]
    fn mono_fallbacks_reach_the_neutral_recipe() {
        let f = Family::custom("segmented");
        for base in [Theme::junie(), Theme::paper()] {
            assert!(
                base.recipes.get(f).is_none(),
                "`segmented` is now a declared family; pick an undeclared fixture"
            );
            let m = base.downgrade(ColorLevel::Mono);
            assert!(m.recipes.get(f).is_none(), "`segmented` became declared");

            assert_eq!(m.recipes.neutral(), base.recipes.neutral());

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

    #[doc = r" §11.4, `PRESSED`: a dragged scrollbar thumb must stay visible at"]
    #[doc = r" `Mono`, without applying the generic pressed-container treatment to"]
    #[doc = r" the whole region. The sole authored `PRESSED` rule on `SCROLLBAR` is"]
    #[doc = r" `set_fg(Role::Accent)` — colour, which conformance case 9 excludes from"]
    #[doc = r" its comparison. The thumb genuinely wears `PRESSED`: a live capture"]
    #[doc = r" keeps it (`components/scroll_region.rs:36-38`)."]
    #[test]
    fn mono_pressed_reaches_the_scrollbar_thumb() {
        for base in [Theme::junie(), Theme::paper()] {
            let m = base.downgrade(ColorLevel::Mono);
            for part in [Part::CONTAINER, Part::TRACK] {
                assert_eq!(
                    m.resolve(
                        Family::SCROLLBAR,
                        Variant::DEFAULT,
                        part,
                        StateFlags::PRESSED,
                        Surface::Canvas,
                    ),
                    m.resolve(
                        Family::SCROLLBAR,
                        Variant::DEFAULT,
                        part,
                        StateFlags::empty(),
                        Surface::Canvas,
                    ),
                    "{part:?} inherited the thumb's pressed affordance"
                );
            }
            let rest = m.resolve(
                Family::SCROLLBAR,
                Variant::DEFAULT,
                Part::THUMB,
                StateFlags::empty(),
                Surface::Canvas,
            );
            let pressed = m.resolve(
                Family::SCROLLBAR,
                Variant::DEFAULT,
                Part::THUMB,
                StateFlags::PRESSED,
                Surface::Canvas,
            );
            assert_eq!(pressed.glyph, rest.glyph);
            assert_eq!(pressed.glyph, Slot::Set(GlyphRole::ScrollThumb));
            assert!(
                pressed.style.add_modifier.contains(Modifier::BOLD),
                "a dragged thumb has no non-colour affordance at Mono: {:?}",
                pressed.style
            );
        }
    }

    #[test]
    fn mono_viewport_selection_is_inverted_underlined_and_applied_once() {
        for base in [Theme::junie(), Theme::paper()] {
            let mono = base.downgrade(ColorLevel::Mono);
            assert_eq!(mono.recipes, base.recipes);

            let resolve = |state| {
                mono.resolve(
                    Family::VIEWPORT,
                    Variant::DEFAULT,
                    Part::TEXT,
                    state,
                    Surface::Canvas,
                )
            };
            let base_text = resolve(StateFlags::empty());
            let selected = resolve(StateFlags::SELECTED);
            let canvas = crate::theme::resolve::bind_role(
                &mono,
                Role::Surface(Surface::Canvas),
                Surface::Canvas,
            );
            let primary =
                crate::theme::resolve::bind_role(&mono, Role::Fg(FgStep::Primary), Surface::Canvas);
            assert_ne!(selected.style, base_text.style);
            assert_eq!(selected.style.fg, canvas);
            assert_eq!(selected.style.bg, primary);
            assert_ne!(selected.style.fg, selected.style.bg);
            assert!(selected.style.add_modifier.contains(Modifier::UNDERLINED));
        }
    }

    #[test]
    fn mono_grid_error_is_underlined_and_applied_once() {
        for base in [Theme::junie(), Theme::paper()] {
            let mono = base.downgrade(ColorLevel::Mono);
            assert_eq!(mono.recipes, base.recipes);

            let resolve = |state| {
                mono.resolve(
                    Family::GRID,
                    Variant::DEFAULT,
                    Part::CELL,
                    state,
                    Surface::Canvas,
                )
            };
            let base_cell = resolve(StateFlags::empty());
            let error = resolve(StateFlags::ERROR);
            assert!(!base_cell.style.add_modifier.contains(Modifier::UNDERLINED));
            assert!(error.style.add_modifier.contains(Modifier::UNDERLINED));
            assert_ne!(error.style.add_modifier, base_cell.style.add_modifier);
        }
    }

    #[test]
    fn mono_targeted_fallbacks_declare_omitted_families() {
        let recipes = Theme::junie().recipes;
        let selected = apply_mono_fallback(
            StylePatch::new(),
            &recipes,
            Family::VIEWPORT,
            Part::TEXT,
            StateFlags::SELECTED,
        );
        assert!(selected.add.contains(Modifier::UNDERLINED));
        let error = apply_mono_fallback(
            StylePatch::new(),
            &recipes,
            Family::GRID,
            Part::CELL,
            StateFlags::ERROR,
        );
        assert!(error.add.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn author_override_precedence_beats_the_static_mono_layer() {
        let mono = Theme::junie()
            .override_family(Family::BUTTON, |recipe| {
                recipe.part(Part::LABEL).when(
                    StateFlags::PRESSED,
                    StylePatch::new().remove(Modifier::BOLD),
                );
            })
            .downgrade(ColorLevel::Mono);
        let pressed = mono.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::LABEL,
            StateFlags::PRESSED,
            Surface::Canvas,
        );
        assert!(!pressed.style.add_modifier.contains(Modifier::BOLD));
        assert!(pressed.style.sub_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn mono_parts_exactly_cover_every_reserved_rule_part() {
        let rules = mono_rules();
        let extra = mono_rules_extra();
        let viewport = viewport_mono_rules();
        let grid = grid_mono_rules();
        let menu = menu_mono_rules();
        let help = help_mono_rules();
        let picker = picker_mono_rules();
        let select = select_mono_rules();
        assert_eq!(rules.len(), 15);
        assert_eq!(extra.len(), 5);
        assert!(extra.iter().any(|(part, when, patch)| {
            *part == Part::ICON
                && *when == StateFlags::LOADING
                && patch.add.contains(Modifier::UNDERLINED)
        }));
        assert_eq!(viewport.len(), 1);
        assert_eq!(grid.len(), 2);
        assert_eq!(menu.len(), 2);
        assert_eq!(help.len(), 2);
        assert_eq!(picker.len(), 3);
        assert_eq!(select.len(), 3);
        assert!(grid.iter().any(|(part, when, patch)| {
            *part == Part::ROW && *when == StateFlags::PRESSED && patch.add.contains(Modifier::BOLD)
        }));
        assert!(picker.iter().any(|(part, when, patch)| {
            *part == Part::LABEL
                && *when == StateFlags::PRESSED
                && patch.fg == Slot::Set(Role::Surface(Surface::Canvas))
                && patch.bg == Slot::Set(Role::Fg(FgStep::Primary))
                && patch.add.contains(Modifier::BOLD)
                && patch.add.contains(Modifier::UNDERLINED)
                && !patch.add.contains(Modifier::REVERSED)
        }));
        assert!(select.iter().any(|(part, when, patch)| {
            *part == Part::FIELD
                && *when == StateFlags::PRESSED
                && patch.fg == Slot::Set(Role::Surface(Surface::Canvas))
                && patch.bg == Slot::Set(Role::Fg(FgStep::Primary))
                && patch.add.contains(Modifier::BOLD)
                && !patch.add.contains(Modifier::REVERSED)
        }));
        assert!(select.iter().any(|(part, when, patch)| {
            *part == Part::GUTTER
                && *when == StateFlags::PRESSED
                && patch.glyph == Slot::Set(GlyphRole::PressLeft)
                && patch.add.contains(Modifier::BOLD)
        }));
        assert!(select.iter().any(|(part, when, patch)| {
            *part == Part::MARKER
                && *when == StateFlags::PRESSED
                && patch.glyph == Slot::Set(GlyphRole::PressRight)
                && patch.add.contains(Modifier::BOLD)
        }));
    }

    #[doc = r" §29 + §11.4: at `Mono` a disabled control must stay **readable**, not"]
    #[doc = r" merely colourless. The rule that produced `Fg(Faint)` here resolved to"]
    #[doc = r" `Black` on a `Black` canvas — invisible — because `mono()` collapses"]
    #[doc = r" every step below `Y = 0.35` onto the background."]
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

    #[doc = r" Every `(part, state)` pair the static mono layer answers for `f`,"]
    #[doc = r" probed through `apply_mono_fallback` rather than read off the tables,"]
    #[doc = r" so the count measures what resolution does."]
    fn responding_pairs(recipes: &Recipes, f: Family) -> usize {
        let mut pairs: Vec<(Part, StateFlags)> = mono_rules()
            .iter()
            .chain(mono_rules_extra().iter())
            .chain(viewport_mono_rules().iter())
            .chain(grid_mono_rules().iter())
            .chain(menu_mono_rules().iter())
            .chain(help_mono_rules().iter())
            .chain(picker_mono_rules().iter())
            .chain(select_mono_rules().iter())
            .map(|(part, when, _)| (*part, *when))
            .collect();
        pairs.sort_by_key(|(part, when)| (part.raw(), when.bits()));
        pairs.dedup();
        pairs
            .iter()
            .filter(|(part, when)| {
                apply_mono_fallback(StylePatch::new(), recipes, f, *part, *when)
                    != StylePatch::new()
            })
            .count()
    }

    #[doc = r" The constant counts the **generic** manifest only; a family's real"]
    #[doc = r" total is that plus its targeted set, minus `SCROLLBAR`'s one exclusion"]
    #[doc = r" and minus any pair a targeted rule shares with the generic set. These"]
    #[doc = r" are the numbers `MONO_RULES_PER_FAMILY`'s documentation states."]
    #[test]
    fn mono_rule_counts_match_the_documented_totals() {
        assert_eq!(
            MONO_RULES_PER_FAMILY,
            mono_rules().len() + mono_rules_extra().len()
        );
        let r = Theme::junie().recipes;

        assert_eq!(responding_pairs(&r, Family::BUTTON), MONO_RULES_PER_FAMILY);
        assert_eq!(
            responding_pairs(&r, Family::custom("seg")),
            MONO_RULES_PER_FAMILY
        );

        assert_eq!(
            responding_pairs(&r, Family::SCROLLBAR),
            MONO_RULES_PER_FAMILY - 1
        );
        for (f, pairs) in [
            (Family::VIEWPORT, 21),
            (Family::GRID, 22),
            (Family::MENU, 22),
            (Family::HELP, 22),
            (Family::PICKER, 22),
            (Family::SELECT, 23),
        ] {
            assert_eq!(responding_pairs(&r, f), pairs, "{f:?}");
        }
    }

    #[doc = r" S6: a downstream family had no way to declare mono affordances — the"]
    #[doc = r" six targeted tables are private and closed. `ThemeBuilder::mono_rules`"]
    #[doc = r" is that seam, and it must reach resolution for a `Family::custom`"]
    #[doc = r" family without declaring the family in recipe storage."]
    #[test]
    fn builder_mono_rules_reach_a_custom_family_at_mono_only() {
        let seg = Family::custom("seg");
        let rule = [(
            Part::MARKER,
            StateFlags::ACTIVE,
            StylePatch::new().set_glyph(GlyphRole::Chosen),
        )];
        let base = Theme::junie().builder().mono_rules(seg, &rule).build();

        assert!(base.recipes.get(seg).is_none());
        assert_eq!(base.recipes.mono_rules(seg), Some(&rule[..]));

        assert_eq!(
            base.resolve(
                seg,
                Variant::DEFAULT,
                Part::MARKER,
                StateFlags::ACTIVE,
                Surface::Canvas
            )
            .glyph,
            Slot::Inherit
        );

        let m = base.downgrade(ColorLevel::Mono);
        assert_eq!(
            m.resolve(
                seg,
                Variant::DEFAULT,
                Part::MARKER,
                StateFlags::ACTIVE,
                Surface::Canvas
            )
            .glyph,
            Slot::Set(GlyphRole::Chosen)
        );

        assert!(
            m.resolve(
                seg,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::FOCUSED,
                Surface::Canvas
            )
            .style
            .add_modifier
            .contains(Modifier::BOLD)
        );

        let overridden = base
            .clone()
            .override_family(seg, |recipe| {
                recipe.part(Part::MARKER).when(
                    StateFlags::ACTIVE,
                    StylePatch {
                        glyph: Slot::Clear,
                        ..StylePatch::new()
                    },
                );
            })
            .downgrade(ColorLevel::Mono);
        assert_eq!(
            overridden
                .resolve(
                    seg,
                    Variant::DEFAULT,
                    Part::MARKER,
                    StateFlags::ACTIVE,
                    Surface::Canvas
                )
                .glyph,
            Slot::Clear
        );
    }

    #[doc = r" Whole-set semantics: the manifest a theme sets **is** the family's"]
    #[doc = r" targeted set. Repeating the call replaces rather than accumulates, an"]
    #[doc = r" empty slice silences the built-in set, and the generic manifest is"]
    #[doc = r" untouched in every case."]
    #[test]
    fn builder_mono_rules_replace_the_whole_targeted_set() {
        let underlined = [(
            Part::FIELD,
            StateFlags::PRESSED,
            StylePatch::new().add(Modifier::UNDERLINED),
        )];
        let replaced = Theme::junie()
            .builder()
            .mono_rules(Family::SELECT, &underlined)
            .build()
            .downgrade(ColorLevel::Mono);
        let at = |t: &Theme, part: Part| {
            apply_mono_fallback(
                StylePatch::new(),
                &t.recipes,
                Family::SELECT,
                part,
                StateFlags::PRESSED,
            )
        };

        assert_eq!(
            at(&replaced, Part::FIELD),
            StylePatch::new().add(Modifier::UNDERLINED)
        );
        assert_eq!(at(&replaced, Part::GUTTER), StylePatch::new());
        assert_eq!(
            at(&replaced, Part::MARKER).glyph,
            Slot::Inherit,
            "the built-in `(MARKER, PRESSED)` rule survived a whole-set replacement"
        );

        assert_eq!(
            responding_pairs(&replaced.recipes, Family::SELECT),
            MONO_RULES_PER_FAMILY + 1
        );

        let last = Theme::junie()
            .builder()
            .mono_rules(Family::SELECT, &underlined)
            .mono_rules(
                Family::SELECT,
                &[(
                    Part::FIELD,
                    StateFlags::PRESSED,
                    StylePatch::new().add(Modifier::BOLD),
                )],
            )
            .build()
            .downgrade(ColorLevel::Mono);
        assert_eq!(
            at(&last, Part::FIELD),
            StylePatch::new().add(Modifier::BOLD)
        );

        let cleared = Theme::junie()
            .builder()
            .mono_rules(Family::SELECT, &[])
            .build()
            .downgrade(ColorLevel::Mono);
        assert_eq!(cleared.recipes.mono_rules(Family::SELECT), Some(&[][..]));
        assert_eq!(at(&cleared, Part::FIELD), StylePatch::new());
        assert_eq!(
            responding_pairs(&cleared.recipes, Family::SELECT),
            MONO_RULES_PER_FAMILY
        );

        assert_eq!(
            responding_pairs(&cleared.recipes, Family::PICKER),
            responding_pairs(&Theme::junie().recipes, Family::PICKER)
        );
    }

    use crate::theme::recipe::{Family, Variant};
}
