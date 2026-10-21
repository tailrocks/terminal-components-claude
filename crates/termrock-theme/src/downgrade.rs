use ratatui_core::style::{Color, Modifier};

use super::Theme;
use super::glyph::GlyphRole;
use super::patch::{Slot, StylePatch};
use super::recipe::{Family, Recipes};
use super::role::{FgStep, Role, Surface};
use super::tokens::{ColorLevel, ColorTokens};
use crate::id::Part;
use crate::response::StateFlags;

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

const fn cube_value(step: u8) -> u8 {
    if step == 0 {
        0
    } else {
        55u8.saturating_add(step.saturating_mul(40))
    }
}

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

pub(crate) fn rgb_of(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(i) => Some(rgb_of_index(i)),
        Color::Reset => None,
        named => named_index(named).and_then(|i| XTERM16.get(i).copied()),
    }
}

fn nearest_256(rgb: (u8, u8, u8)) -> u8 {
    let (r, g, b) = rgb;
    let step = |value: u8| ((u32::from(value) * 5 + 127) / 255) as u8;
    let (red, green, blue) = (step(r), step(g), step(b));
    let cube = 16 + 36 * red + 6 * green + blue;
    let cube_value = |index: u8| -> i32 {
        if index == 0 {
            0
        } else {
            55 + i32::from(index) * 40
        }
    };
    let distance = |candidate: (i32, i32, i32)| {
        (candidate.0 - i32::from(r)).pow(2)
            + (candidate.1 - i32::from(g)).pow(2)
            + (candidate.2 - i32::from(b)).pow(2)
    };
    let cube_error = distance((cube_value(red), cube_value(green), cube_value(blue)));

    let average = (i32::from(r) + i32::from(g) + i32::from(b)) / 3;
    let gray_step = ((average - 8).max(0) / 10).min(23);
    let gray_value = 8 + gray_step * 10;
    let gray_error = distance((gray_value, gray_value, gray_value));
    let chroma = i32::from(r.max(g).max(b)) - i32::from(r.min(g).min(b));

    if gray_error < cube_error && chroma <= 30 {
        232 + gray_step as u8
    } else {
        cube
    }
}

fn to_lin(c: u8) -> f64 {
    let v = f64::from(c) / 255.0;
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

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

pub(crate) fn luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * to_lin(rgb.0) + 0.7152 * to_lin(rgb.1) + 0.0722 * to_lin(rgb.2)
}

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

/// Baseline `--color none` ink for one raw RGB cell.
///
/// `menu_row` is the top brand chip. That chip keeps the reverse attribute.
/// The same green elsewhere becomes black on white without reverse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegacyMono {
    pub fg: Color,
    pub bg: Color,
    pub underline: bool,
    pub reverse: bool,
}

pub fn legacy_mono(fg: Color, bg: Color, menu_row: bool) -> Option<LegacyMono> {
    let fg_rgb = rgb_of(fg)?;
    let bg_rgb = rgb_of(bg)?;
    let (out_fg, out_bg, underline, reverse) = match (fg_rgb, bg_rgb) {
        ((0x19, 0x19, 0x1c), (0x48, 0xe0, 0x54)) => (Color::Black, Color::White, false, menu_row),
        ((0x79, 0xb8, 0xff), (0, 0, 0)) => (Color::Gray, Color::Black, true, false),
        ((0xff, 0xff, 0xff), (0x27, 0x27, 0x2a)) => (Color::Black, Color::DarkGray, false, false),
        ((0x27, 0x27, 0x2a), (0x27, 0x27, 0x2a)) => {
            (Color::DarkGray, Color::DarkGray, false, false)
        }
        ((0x26, 0x26, 0x26), (0, 0, 0)) => (Color::DarkGray, Color::Black, false, false),
        ((0x80, 0x80, 0x80), (0, 0, 0)) => (Color::Gray, Color::Black, false, false),
        ((0x47, 0x47, 0x47), (0, 0, 0)) => (Color::DarkGray, Color::Black, false, false),
        ((0x8a, 0x8a, 0x8a) | (0xb3, 0xb3, 0xb3) | (0x73, 0x73, 0x73), (0, 0, 0)) => {
            (Color::Gray, Color::Black, false, false)
        }
        ((0x48, 0xe0, 0x54) | (0x3f, 0xb9, 0x50), (0, 0, 0)) => {
            (Color::White, Color::Black, false, false)
        }
        ((0xff, 0xff, 0xff), (0, 0, 0)) => (Color::White, Color::Black, false, false),
        ((0, 0, 0), (0, 0, 0)) => (Color::Black, Color::Black, false, false),
        _ => return None,
    };
    Some(LegacyMono {
        fg: out_fg,
        bg: out_bg,
        underline,
        reverse,
    })
}

/// Downgrade a painted pair. On 16 colors, white on dark gray becomes black
/// so the label keeps the baseline contrast.
pub fn downgrade_pair(fg: Color, bg: Color, level: ColorLevel) -> (Color, Color) {
    let mut fg = downgrade_color(fg, level);
    let bg = downgrade_color(bg, level);
    if level == ColorLevel::Ansi16 && fg == Color::White && bg == Color::DarkGray {
        fg = Color::Black;
    }
    (fg, bg)
}

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

    #[must_use]
    pub fn for_level(&self, detected: ColorLevel) -> Theme {
        self.downgrade(self.capability.color.narrow_to(detected))
    }

    #[must_use]
    pub fn for_terminal(&self) -> Theme {
        self.for_level(ColorLevel::detect())
    }
}

pub type MonoRule = (Part, StateFlags, StylePatch);

const BUTTON_GENERIC_MARKER_GLYPH_RULES: [(Part, StateFlags); 2] = [
    (Part::MARKER, StateFlags::SELECTED),
    (Part::MARKER, StateFlags::CHECKED),
];

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
            // Frozen `--color none` disabled ink is dark gray plus dim
            // (index 8), not primary white plus dim.
            p().set_fg(Role::DisabledFg)
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

pub const MONO_RULES_PER_FAMILY: usize = 20;

pub(crate) static VIEWPORT_MONO_RULES: [MonoRule; 1] = [(
    Part::TEXT,
    StateFlags::SELECTED,
    StylePatch::new()
        .set_fg(Role::Surface(Surface::Canvas))
        .set_bg(Role::Fg(FgStep::Primary))
        .add(Modifier::UNDERLINED),
)];

pub(crate) static GRID_MONO_RULES: [MonoRule; 2] = [
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
];

pub(crate) static MENU_MONO_RULES: [MonoRule; 2] = [
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
];

pub(crate) static HELP_MONO_RULES: [MonoRule; 2] = [
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
];

pub(crate) static PICKER_MONO_RULES: [MonoRule; 3] = [
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
            .add(Modifier::BOLD.union(Modifier::UNDERLINED)),
    ),
];

pub(crate) static SELECT_MONO_RULES: [MonoRule; 3] = [
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
];

pub(crate) static BUTTON_MONO_RULES: [MonoRule; 1] = [(
    Part::GUTTER,
    StateFlags::FOCUS_VISIBLE,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];

pub(crate) fn builtin_family_mono_rules(family: Family) -> &'static [MonoRule] {
    match family {
        Family::BUTTON => &BUTTON_MONO_RULES,
        Family::VIEWPORT => &VIEWPORT_MONO_RULES,
        Family::GRID => &GRID_MONO_RULES,
        Family::MENU => &MENU_MONO_RULES,
        Family::HELP => &HELP_MONO_RULES,
        Family::PICKER => &PICKER_MONO_RULES,
        Family::SELECT => &SELECT_MONO_RULES,
        _ => &[],
    }
}

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
    let builtin: &'static [MonoRule] = builtin_family_mono_rules(family);

    let targeted: &[MonoRule] = recipes
        .mono_rules(family)
        .or(authored.filter(|rules| !rules.is_empty()))
        .unwrap_or(builtin);
    for (rule_part, when, patch) in rules.iter().chain(extra.iter()) {
        if family == Family::BUTTON && *rule_part == Part::GUTTER && *when == StateFlags::FOCUSED {
            continue;
        }
        // Buttons use the semantic DotOn/DotOff roles for their checked
        // marker. The generic mono marker glyphs are intended for checkbox
        // and selection controls; applying them here replaces an authored
        // button glyph (and turns the frozen dot into a checkmark).
        if family == Family::BUTTON
            && BUTTON_GENERIC_MARKER_GLYPH_RULES.contains(&(*rule_part, *when))
        {
            continue;
        }
        let applies = family != Family::SCROLLBAR
            || *rule_part != Part::CONTAINER
            || *when != StateFlags::PRESSED;
        if applies && *rule_part == part && live.contains(*when) {
            acc = acc.merge(*patch);
        }
    }
    for (rule_part, when, patch) in targeted {
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

    /// The reserved per-family mono rule sets, read back from the built-in
    /// tables so the coverage check below stays bound to the real rules.
    fn viewport_mono_rules() -> Vec<MonoRule> {
        VIEWPORT_MONO_RULES.to_vec()
    }

    fn grid_mono_rules() -> Vec<MonoRule> {
        GRID_MONO_RULES.to_vec()
    }

    fn menu_mono_rules() -> Vec<MonoRule> {
        MENU_MONO_RULES.to_vec()
    }

    fn help_mono_rules() -> Vec<MonoRule> {
        HELP_MONO_RULES.to_vec()
    }

    fn picker_mono_rules() -> Vec<MonoRule> {
        PICKER_MONO_RULES.to_vec()
    }

    fn select_mono_rules() -> Vec<MonoRule> {
        SELECT_MONO_RULES.to_vec()
    }

    fn button_mono_rules() -> Vec<MonoRule> {
        BUTTON_MONO_RULES.to_vec()
    }

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
            Color::Indexed(78)
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x3a, 0x3a, 0x3a), ColorLevel::Ansi256),
            Color::Indexed(237)
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x73, 0x73, 0x73), ColorLevel::Ansi256),
            Color::Indexed(242)
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x47, 0x47, 0x47), ColorLevel::Ansi256),
            Color::Indexed(238)
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x80, 0x80, 0x80), ColorLevel::Ansi256),
            Color::Indexed(244)
        );
        assert_eq!(
            downgrade_color(Color::Rgb(0x17, 0x3f, 0x1c), ColorLevel::Ansi256),
            Color::Indexed(23)
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

    #[test]
    fn ansi256_raw_rgb_uses_the_frozen_general_cube_and_gray_rule() {
        // These samples exercise the cube path, the low-chroma gray path, and
        // the chroma cutoff. The rule is the frozen baseline's general
        // nearest_256 algorithm, not a lookup table for selected theme colors.
        for (rgb, expected) in [
            ((0x3a, 0x3a, 0x3a), 237),
            ((0x47, 0x47, 0x47), 238),
            ((0x80, 0x80, 0x80), 244),
            ((0x48, 0xe0, 0x54), 78),
            ((0x17, 0x3f, 0x1c), 23),
            ((0x23, 0x4b, 0x64), 60),
        ] {
            assert_eq!(
                downgrade_color(Color::Rgb(rgb.0, rgb.1, rgb.2), ColorLevel::Ansi256),
                Color::Indexed(expected),
                "raw RGB {rgb:?}"
            );
        }
    }

    #[test]
    fn legacy_mono_matches_the_frozen_launch_inks() {
        let ink = |fg: Color, bg: Color, menu: bool| legacy_mono(fg, bg, menu).expect("known ink");
        let brand = ink(
            Color::Rgb(0x19, 0x19, 0x1c),
            Color::Rgb(0x48, 0xe0, 0x54),
            true,
        );
        assert_eq!(brand.fg, Color::Black);
        assert_eq!(brand.bg, Color::White);
        assert!(brand.reverse);
        assert!(
            !ink(
                Color::Rgb(0x19, 0x19, 0x1c),
                Color::Rgb(0x48, 0xe0, 0x54),
                false
            )
            .reverse
        );
        let trace = ink(Color::Rgb(0x79, 0xb8, 0xff), Color::Rgb(0, 0, 0), false);
        assert_eq!(trace.fg, Color::Gray);
        assert!(trace.underline);
        assert_eq!(
            ink(Color::Rgb(0xb3, 0xb3, 0xb3), Color::Rgb(0, 0, 0), false).fg,
            Color::Gray
        );
        assert_eq!(
            ink(Color::Rgb(0x26, 0x26, 0x26), Color::Rgb(0, 0, 0), false).fg,
            Color::DarkGray
        );
        assert_eq!(
            ink(Color::Rgb(0x80, 0x80, 0x80), Color::Rgb(0, 0, 0), false).fg,
            Color::Gray
        );
        assert_eq!(
            ink(Color::Rgb(0x47, 0x47, 0x47), Color::Rgb(0, 0, 0), false).fg,
            Color::DarkGray
        );
        let button = ink(
            Color::Rgb(0xff, 0xff, 0xff),
            Color::Rgb(0x27, 0x27, 0x2a),
            false,
        );
        assert_eq!(button.fg, Color::Black);
        assert_eq!(button.bg, Color::DarkGray);
    }

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

    fn sorted_rule_pairs(mut pairs: Vec<(Part, StateFlags)>) -> Vec<(Part, StateFlags)> {
        pairs.sort_by_key(|(part, when)| (part.raw(), when.bits()));
        pairs.dedup();
        pairs
    }

    fn responding_rule_pairs(recipes: &Recipes, f: Family) -> Vec<(Part, StateFlags)> {
        let pairs: Vec<(Part, StateFlags)> = mono_rules()
            .iter()
            .chain(mono_rules_extra().iter())
            .chain(button_mono_rules().iter())
            .chain(viewport_mono_rules().iter())
            .chain(grid_mono_rules().iter())
            .chain(menu_mono_rules().iter())
            .chain(help_mono_rules().iter())
            .chain(picker_mono_rules().iter())
            .chain(select_mono_rules().iter())
            .map(|(part, when, _)| (*part, *when))
            .collect();
        sorted_rule_pairs(pairs)
            .iter()
            .filter(|(part, when)| {
                apply_mono_fallback(StylePatch::new(), recipes, f, *part, *when)
                    != StylePatch::new()
            })
            .copied()
            .collect()
    }

    fn responding_pairs(recipes: &Recipes, f: Family) -> usize {
        responding_rule_pairs(recipes, f).len()
    }

    #[test]
    fn mono_rule_counts_match_the_documented_totals() {
        assert_eq!(
            MONO_RULES_PER_FAMILY,
            mono_rules().len() + mono_rules_extra().len()
        );
        let r = Theme::junie().recipes;

        assert_eq!(
            BUTTON_GENERIC_MARKER_GLYPH_RULES,
            [
                (Part::MARKER, StateFlags::SELECTED),
                (Part::MARKER, StateFlags::CHECKED),
            ]
        );
        let generic = sorted_rule_pairs(
            mono_rules()
                .iter()
                .chain(mono_rules_extra().iter())
                .map(|(part, when, _)| (*part, *when))
                .collect(),
        );
        assert_eq!(
            generic,
            sorted_rule_pairs(vec![
                (Part::GUTTER, StateFlags::FOCUSED),
                (Part::LABEL, StateFlags::FOCUSED),
                (Part::MARKER, StateFlags::SELECTED),
                (Part::MARKER, StateFlags::CHECKED),
                (Part::CONTAINER, StateFlags::PRESSED),
                (Part::LABEL, StateFlags::PRESSED),
                (Part::GUTTER, StateFlags::DISABLED),
                (Part::MARKER, StateFlags::DISABLED),
                (Part::LABEL, StateFlags::DISABLED),
                (Part::FIELD, StateFlags::DISABLED),
                (Part::TEXT, StateFlags::DISABLED),
                (Part::MARKER, StateFlags::ERROR),
                (Part::FIELD, StateFlags::ERROR),
                (Part::MARKER, StateFlags::WARNING),
                (Part::TEXT, StateFlags::EDITING),
                (Part::MARKER, StateFlags::DIRTY),
                (Part::RULE, StateFlags::ACTIVE),
                (Part::LABEL, StateFlags::ACTIVE),
                (Part::ICON, StateFlags::LOADING),
                (Part::THUMB, StateFlags::PRESSED),
            ])
        );
        assert_eq!(
            responding_rule_pairs(&r, Family::BUTTON),
            sorted_rule_pairs(vec![
                (Part::LABEL, StateFlags::FOCUSED),
                (Part::CONTAINER, StateFlags::PRESSED),
                (Part::LABEL, StateFlags::PRESSED),
                (Part::GUTTER, StateFlags::DISABLED),
                (Part::MARKER, StateFlags::DISABLED),
                (Part::LABEL, StateFlags::DISABLED),
                (Part::FIELD, StateFlags::DISABLED),
                (Part::TEXT, StateFlags::DISABLED),
                (Part::MARKER, StateFlags::ERROR),
                (Part::FIELD, StateFlags::ERROR),
                (Part::MARKER, StateFlags::WARNING),
                (Part::TEXT, StateFlags::EDITING),
                (Part::MARKER, StateFlags::DIRTY),
                (Part::RULE, StateFlags::ACTIVE),
                (Part::LABEL, StateFlags::ACTIVE),
                (Part::ICON, StateFlags::LOADING),
                (Part::THUMB, StateFlags::PRESSED),
                (Part::GUTTER, StateFlags::FOCUS_VISIBLE),
            ])
        );
        assert_eq!(
            responding_pairs(&r, Family::BUTTON),
            MONO_RULES_PER_FAMILY - 2
        );
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

    #[test]
    fn button_focus_indicator_is_scoped_to_focus_visible_in_every_profile() {
        let variants = [
            Variant::DEFAULT,
            Variant::PRIMARY,
            Variant::SECONDARY,
            Variant::SUBTLE,
            Variant::DANGER,
            Variant::TOGGLE,
            Variant::QUIET,
            Variant::GHOST,
        ];
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let theme = Theme::junie().downgrade(level);
            for variant in variants {
                let pointer = theme.resolve(
                    Family::BUTTON,
                    variant,
                    Part::GUTTER,
                    StateFlags::FOCUSED,
                    Surface::Canvas,
                );
                assert_ne!(
                    pointer.glyph,
                    Slot::Set(GlyphRole::FocusBar),
                    "{level:?}/{variant:?} made pointer focus visible"
                );
                let keyboard = theme.resolve(
                    Family::BUTTON,
                    variant,
                    Part::GUTTER,
                    StateFlags::FOCUS_VISIBLE,
                    Surface::Canvas,
                );
                assert_eq!(
                    keyboard.glyph,
                    Slot::Set(GlyphRole::FocusBar),
                    "{level:?}/{variant:?} omitted keyboard focus"
                );
            }
            let list = theme.resolve(
                Family::LIST,
                Variant::DEFAULT,
                Part::GUTTER,
                StateFlags::FOCUSED,
                Surface::Canvas,
            );
            assert_eq!(
                list.glyph,
                Slot::Set(GlyphRole::FocusBar),
                "Button policy leaked into list gutter at {level:?}"
            );
        }
    }

    #[test]
    fn authored_button_mono_focus_override_keeps_precedence() {
        let theme = Theme::junie()
            .builder()
            .mono_rules(
                Family::BUTTON,
                &[
                    (
                        Part::GUTTER,
                        StateFlags::FOCUSED,
                        StylePatch::new().set_glyph(GlyphRole::RuleQuiet),
                    ),
                    (
                        Part::GUTTER,
                        StateFlags::FOCUS_VISIBLE,
                        StylePatch::new().set_glyph(GlyphRole::RuleQuiet),
                    ),
                ],
            )
            .build()
            .downgrade(ColorLevel::Mono);
        let focused = theme.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::FOCUSED,
            Surface::Canvas,
        );
        assert_eq!(focused.glyph, Slot::Set(GlyphRole::RuleQuiet));
        let keyboard = theme.resolve(
            Family::BUTTON,
            Variant::DEFAULT,
            Part::GUTTER,
            StateFlags::FOCUS_VISIBLE,
            Surface::Canvas,
        );
        assert_eq!(keyboard.glyph, Slot::Set(GlyphRole::RuleQuiet));
    }

    #[test]
    fn authored_scrollbar_container_press_rule_keeps_the_legacy_filter() {
        let authored = [
            (
                Part::CONTAINER,
                StateFlags::PRESSED,
                StylePatch::new().set_glyph(GlyphRole::WarningMark),
            ),
            (
                Part::THUMB,
                StateFlags::PRESSED,
                StylePatch::new().set_glyph(GlyphRole::WarningMark),
            ),
        ];
        let theme = Theme::junie()
            .builder()
            .mono_rules(Family::SCROLLBAR, &authored)
            .build()
            .downgrade(ColorLevel::Mono);
        let pressed = theme.resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::PRESSED,
            Surface::Canvas,
        );

        assert_ne!(pressed.glyph, Slot::Set(GlyphRole::WarningMark));
        let thumb = theme.resolve(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::THUMB,
            StateFlags::PRESSED,
            Surface::Canvas,
        );
        assert_eq!(thumb.glyph, Slot::Set(GlyphRole::WarningMark));
    }

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
