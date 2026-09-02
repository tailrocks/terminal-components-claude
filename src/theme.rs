//! Design tokens derived from junie.jetbrains.com.
//!
//! Evidence (computed styles / CSS custom properties on the live site):
//! - canvas `#000000` (`--colors-bg: black`), chrome/panels `#111111`,
//!   cards `#18181b` (`--color-card`, zinc-900), input surface `#27272a`
//!   (`--color-input`, zinc-800), popover `#3f3f46` (zinc-700).
//! - accent `#48e054` (`--colors-primary`), hover = accent at 80% over black
//!   (`#3ab343`), tints = accent at 10–20% alpha.
//! - text is an alpha ladder on white: 100% / 70% / 50% / 30%.
//! - borders are white at 10% (subtle) and 30% (strong); selection uses a
//!   2px white/green ring, never a colour flood.
//! - destructive `#e44545` (red-400), warning `#f59e09`, info `#8787ff`.
//!
//! Everything in the UI goes through these tokens; rendering code never
//! spells out an RGB value.

use ratatui::style::{Color, Modifier, Style};

use crate::ui::ctx::VisualState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorLevel {
    pub fn detect() -> Self {
        if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return ColorLevel::Mono;
        }
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return ColorLevel::TrueColor;
        }
        let term = std::env::var("TERM").unwrap_or_default();
        if term.contains("256color") || term.contains("ghostty") || term.contains("kitty") {
            return ColorLevel::Ansi256;
        }
        ColorLevel::Ansi16
    }

    pub fn label(self) -> &'static str {
        match self {
            ColorLevel::TrueColor => "truecolor",
            ColorLevel::Ansi256 => "256 colors",
            ColorLevel::Ansi16 => "16 colors",
            ColorLevel::Mono => "no color",
        }
    }
}

/// Raw palette. Values are the Junie references (see module docs).
mod palette {
    use ratatui::style::Color;

    pub const fn rgb(hex: u32) -> Color {
        Color::Rgb(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    pub const BLACK: Color = rgb(0x000000);
    pub const CHROME: Color = rgb(0x111111);
    pub const CARD: Color = rgb(0x18181b);
    pub const INPUT: Color = rgb(0x1e1e22);
    pub const INPUT_HOVER: Color = rgb(0x232328);
    pub const OVERLAY: Color = rgb(0x27272a);
    pub const POPOVER: Color = rgb(0x3f3f46);
    /// Menu selection: a cool blue that does not compete with the accent.
    pub const HIGHLIGHT: Color = rgb(0x2f5aa8);
    pub const WHITE: Color = rgb(0xffffff);
    pub const WHITE_70: Color = rgb(0xb3b3b3);
    pub const WHITE_50: Color = rgb(0x808080);
    pub const WHITE_30: Color = rgb(0x4d4d4d);
    pub const WHITE_15: Color = rgb(0x262626);
    pub const GREEN: Color = rgb(0x48e054);
    pub const GREEN_80: Color = rgb(0x3ab343);
    pub const GREEN_60: Color = rgb(0x2b8632);
    pub const GREEN_20: Color = rgb(0x0f2e13);
    pub const GREEN_10: Color = rgb(0x0a1c0c);
    pub const ON_GREEN: Color = rgb(0x19191c);
    pub const RED: Color = rgb(0xe44545);
    pub const RED_20: Color = rgb(0x2e0f0f);
    /// Soft rose: a destructive label at rest on a neutral plane.
    pub const RED_SOFT: Color = rgb(0xd98a8a);
    /// Deep red: the highlight of a destructive menu row under the cursor.
    pub const RED_45: Color = rgb(0x7a2a2a);
    pub const AMBER: Color = rgb(0xf59e09);
    pub const PURPLE: Color = rgb(0x8787ff);
}

/// Semantic tokens. Field names are the vocabulary used everywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub level: ColorLevel,

    pub canvas: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_overlay: Color,
    pub field: Color,
    pub field_hover: Color,
    pub popover: Color,
    /// The cursor row of an anchored menu: a hue reserved for transient
    /// command lists so the accent keeps its meaning.
    pub highlight: Color,
    /// The cursor row of a destructive menu item: a deep red fill.
    pub highlight_danger: Color,
    /// A destructive label at rest on a neutral plane: desaturated so it
    /// sits in tone with grey rather than shouting.
    pub error_soft: Color,

    pub border_subtle: Color,
    pub border_strong: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_faint: Color,
    /// One step below faint: dimmed backdrops only.
    pub text_ghost: Color,
    pub text_on_accent: Color,

    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub accent_bg: Color,
    pub accent_bg_subtle: Color,
    pub focus: Color,

    pub disabled: Color,
    pub error: Color,
    pub error_bg: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
}

impl Theme {
    pub const fn junie() -> Self {
        use palette::*;
        Self {
            level: ColorLevel::TrueColor,
            canvas: BLACK,
            surface: CHROME,
            surface_elevated: CARD,
            surface_overlay: OVERLAY,
            field: INPUT,
            field_hover: INPUT_HOVER,
            popover: POPOVER,
            highlight: HIGHLIGHT,
            highlight_danger: RED_45,
            error_soft: RED_SOFT,
            border_subtle: WHITE_15,
            border_strong: WHITE_30,
            text_primary: WHITE,
            text_secondary: WHITE_70,
            text_muted: WHITE_50,
            text_faint: WHITE_30,
            text_ghost: WHITE_15,
            text_on_accent: ON_GREEN,
            accent: GREEN,
            accent_hover: GREEN_80,
            accent_pressed: GREEN_60,
            accent_bg: GREEN_20,
            accent_bg_subtle: GREEN_10,
            focus: GREEN,
            disabled: WHITE_30,
            error: RED,
            error_bg: RED_20,
            warning: AMBER,
            success: GREEN,
            info: PURPLE,
        }
    }

    /// Junie theme resolved for the given colour capability.
    pub fn for_level(level: ColorLevel) -> Self {
        let mut t = Self::junie();
        t.level = level;
        if level == ColorLevel::TrueColor {
            return t;
        }
        macro_rules! map {
            ($($f:ident),*) => { $( t.$f = downgrade(t.$f, level); )* };
        }
        map!(
            canvas,
            surface,
            surface_elevated,
            surface_overlay,
            field,
            field_hover,
            popover,
            highlight,
            highlight_danger,
            error_soft,
            border_subtle,
            border_strong,
            text_primary,
            text_secondary,
            text_muted,
            text_faint,
            text_ghost,
            text_on_accent,
            accent,
            accent_hover,
            accent_pressed,
            accent_bg,
            accent_bg_subtle,
            focus,
            disabled,
            error,
            error_bg,
            warning,
            success,
            info
        );
        t
    }

    // --- base styles -------------------------------------------------------

    pub fn base(&self) -> Style {
        Style::new().fg(self.text_primary).bg(self.canvas)
    }

    pub fn on(&self, bg: Color) -> Style {
        Style::new().fg(self.text_primary).bg(bg)
    }

    pub fn primary(&self) -> Style {
        Style::new().fg(self.text_primary)
    }

    pub fn secondary(&self) -> Style {
        Style::new().fg(self.text_secondary)
    }

    pub fn muted(&self) -> Style {
        Style::new().fg(self.text_muted)
    }

    pub fn faint(&self) -> Style {
        Style::new().fg(self.text_faint)
    }

    pub fn accent_fg(&self) -> Style {
        Style::new().fg(self.accent)
    }

    pub fn error_fg(&self) -> Style {
        Style::new().fg(self.error)
    }

    pub fn title(&self) -> Style {
        Style::new()
            .fg(self.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn label(&self, focused: bool) -> Style {
        if focused {
            self.title()
        } else {
            self.secondary()
        }
    }

    pub fn key_hint_key(&self) -> Style {
        Style::new()
            .fg(self.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn key_hint_action(&self) -> Style {
        Style::new().fg(self.text_muted)
    }

    pub fn border(&self, focused: bool) -> Style {
        Style::new().fg(if focused {
            self.border_strong
        } else {
            self.border_subtle
        })
    }

    /// Style for a backdrop cell under a modal: surfaces stay so the page
    /// keeps its shape, every colour collapses to the faint text tier, and
    /// any coloured fill (accent, error, selection tint, reversed cursor)
    /// drops to a neutral overlay.
    pub fn backdrop(&self, style: Style) -> Style {
        let bg = match style.bg {
            Some(c) if c == self.canvas || c == self.surface || c == self.surface_elevated => c,
            Some(c) if c == self.field || c == self.field_hover => self.surface_elevated,
            Some(_) => self.surface_overlay,
            None => self.canvas,
        };
        // scale the alpha ladder instead of collapsing it: hierarchy survives
        let fg = match style.fg {
            // a glyph painted in its own background is a hidden gutter: keep it hidden
            Some(c) if Some(c) == style.bg => bg,
            Some(c) if c == self.canvas || c == self.surface => bg,
            Some(c)
                if c == self.text_primary
                    || c == self.accent
                    || c == self.error
                    || c == self.warning =>
            {
                self.text_muted
            }
            Some(c) if c == self.text_secondary || c == self.text_on_accent => self.text_faint,
            _ => self.text_ghost,
        };
        Style::new().fg(fg).bg(bg)
    }

    // --- component resolvers ----------------------------------------------
    //
    // All resolvers take the container background so a control looks right
    // on the canvas, on a surface, or inside a dialog.

    /// Row-like control (nav item, list item, table row, tree node).
    pub fn row(&self, s: VisualState, bg: Color) -> Style {
        if s.disabled {
            return Style::new().fg(self.disabled).bg(bg);
        }
        let mut st = Style::new().fg(self.text_primary).bg(bg);
        // selection tint only where the keyboard is (focused row); elsewhere
        // the marker glyph alone carries "selected"
        if s.selected && s.focused {
            st = st.bg(self.accent_bg);
        }
        // hover is always exactly one plane up, never a colour
        if s.hovered {
            st = st.bg(self.lift(bg));
        }
        if s.error {
            st = st.fg(self.error);
        }
        if s.busy {
            st = st.fg(self.text_secondary);
        }
        if s.focused {
            st = st.add_modifier(Modifier::BOLD);
        }
        if s.pressed {
            st = Style::new()
                .fg(self.canvas)
                .bg(self.text_primary)
                .add_modifier(Modifier::BOLD);
        }
        st
    }

    /// One step lighter than `bg`, used for hover.
    pub fn lift(&self, bg: Color) -> Color {
        if bg == self.canvas {
            self.surface_elevated
        } else if bg == self.surface || bg == self.surface_elevated {
            self.surface_overlay
        } else if bg == self.field {
            self.field_hover
        } else {
            self.popover
        }
    }

    /// Focus gutter glyph style. `on_accent` is used when the control itself
    /// is filled with the accent (primary button).
    pub fn gutter(&self, s: VisualState, bg: Color, on_accent: bool) -> Style {
        let fg = if !s.focused {
            bg
        } else if on_accent {
            self.text_primary
        } else {
            self.focus
        };
        Style::new().fg(fg).bg(bg)
    }

    pub fn button(&self, kind: ButtonKind, s: VisualState, bg: Color) -> Style {
        if s.disabled {
            return Style::new()
                .fg(self.disabled)
                .bg(if kind == ButtonKind::Subtle {
                    bg
                } else {
                    self.lift(bg)
                });
        }
        match kind {
            ButtonKind::Primary => {
                let b = if s.pressed {
                    self.accent_pressed
                } else if s.hovered {
                    self.accent_hover
                } else {
                    self.accent
                };
                Style::new()
                    .fg(self.text_on_accent)
                    .bg(b)
                    .add_modifier(Modifier::BOLD)
            }
            ButtonKind::Secondary | ButtonKind::Toggle => {
                let mut st = Style::new().fg(self.text_primary).bg(self.surface_overlay);
                if s.hovered {
                    st = st.bg(self.popover);
                }
                if s.focused {
                    st = st.add_modifier(Modifier::BOLD);
                }
                if s.pressed {
                    st = Style::new().fg(self.canvas).bg(self.text_primary);
                }
                st
            }
            ButtonKind::Subtle => {
                let mut st = Style::new().fg(self.text_secondary).bg(bg);
                if s.hovered {
                    st = st.fg(self.text_primary).bg(self.lift(bg));
                }
                if s.focused {
                    st = st.fg(self.text_primary).add_modifier(Modifier::BOLD);
                }
                if s.pressed {
                    st = Style::new().fg(self.canvas).bg(self.text_primary);
                }
                st
            }
            ButtonKind::Danger => {
                let mut st = Style::new().fg(self.error).bg(self.surface_overlay);
                if s.hovered {
                    st = st.bg(self.popover);
                }
                if s.focused {
                    st = st.add_modifier(Modifier::BOLD);
                }
                if s.pressed {
                    st = Style::new().fg(self.text_primary).bg(self.error);
                }
                st
            }
        }
    }

    /// Text field body (input, textarea, editable cell).
    pub fn field_style(&self, s: VisualState) -> Style {
        if s.disabled {
            return Style::new().fg(self.disabled).bg(self.field);
        }
        let bg = if s.hovered && !s.editing {
            self.field_hover
        } else {
            self.field
        };
        Style::new().fg(self.text_primary).bg(bg)
    }

    pub fn placeholder(&self, s: VisualState) -> Style {
        self.field_style(s).fg(if s.disabled {
            self.disabled
        } else {
            self.text_muted
        })
    }

    pub fn selection(&self) -> Style {
        Style::new().fg(self.text_primary).bg(self.popover)
    }

    pub fn scrollbar_track(&self) -> Style {
        Style::new().fg(self.border_subtle)
    }

    pub fn scrollbar_thumb(&self, focused: bool, hovered: bool) -> Style {
        Style::new().fg(if focused {
            self.text_primary
        } else if hovered {
            self.text_secondary
        } else {
            self.text_muted
        })
    }

    pub fn tone(&self, tone: Tone) -> Color {
        match tone {
            Tone::Normal => self.text_primary,
            Tone::Secondary => self.text_secondary,
            Tone::Muted => self.text_muted,
            Tone::Faint => self.text_faint,
            Tone::Error => self.error,
            Tone::Warning => self.warning,
            Tone::Success => self.success,
        }
    }

    /// Restrained syntax palette: structure through weight and the text
    /// ladder, not hue.
    pub fn syntax(&self, tone: SyntaxTone) -> Style {
        match tone {
            SyntaxTone::Keyword => Style::new()
                .fg(self.text_primary)
                .add_modifier(Modifier::BOLD),
            SyntaxTone::Ident | SyntaxTone::Plain => Style::new().fg(self.text_primary),
            SyntaxTone::Str => Style::new().fg(self.text_secondary),
            SyntaxTone::Number => Style::new().fg(self.text_secondary),
            SyntaxTone::Operator | SyntaxTone::Punct => Style::new().fg(self.text_muted),
            SyntaxTone::Comment => Style::new()
                .fg(self.text_faint)
                .add_modifier(Modifier::ITALIC),
        }
    }

    pub fn badge(&self, kind: BadgeKind) -> Style {
        match kind {
            BadgeKind::Edit => Style::new()
                .fg(self.text_on_accent)
                .bg(self.accent)
                .add_modifier(Modifier::BOLD),
        }
    }
}

/// Text tone for values, segments, cells. Maps to the alpha ladder plus the
/// three semantic colours; never to the accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Normal,
    Secondary,
    Muted,
    Faint,
    Error,
    Warning,
    Success,
}

/// Language-agnostic syntax classes for the code editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTone {
    Keyword,
    Ident,
    Number,
    Str,
    Operator,
    Punct,
    Comment,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Subtle,
    Danger,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Edit,
}

fn downgrade(c: Color, level: ColorLevel) -> Color {
    let Color::Rgb(r, g, b) = c else { return c };
    match level {
        ColorLevel::TrueColor => c,
        ColorLevel::Ansi256 => Color::Indexed(nearest_256(r, g, b)),
        ColorLevel::Ansi16 => nearest_16(r, g, b),
        ColorLevel::Mono => match (r as u32 + g as u32 + b as u32) / 3 {
            0..=40 => Color::Black,
            41..=110 => Color::DarkGray,
            111..=190 => Color::Gray,
            _ => Color::White,
        },
    }
}

fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let step = |v: u8| -> u8 { ((v as u32 * 5 + 127) / 255) as u8 };
    let cube = 16 + 36 * step(r) + 6 * step(g) + step(b);
    let cube_val = |i: u8| -> i32 { if i == 0 { 0 } else { 55 + i as i32 * 40 } };
    let (cr, cg, cb) = (cube_val(step(r)), cube_val(step(g)), cube_val(step(b)));
    let cube_err = (cr - r as i32).pow(2) + (cg - g as i32).pow(2) + (cb - b as i32).pow(2);
    let avg = (r as i32 + g as i32 + b as i32) / 3;
    let gi = ((avg - 8).max(0) / 10).min(23);
    let gv = 8 + gi * 10;
    let gray_err = (gv - r as i32).pow(2) + (gv - g as i32).pow(2) + (gv - b as i32).pow(2);
    if gray_err < cube_err {
        232 + gi as u8
    } else {
        cube
    }
}

fn nearest_16(r: u8, g: u8, b: u8) -> Color {
    let lum = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
    let max = r.max(g).max(b) as u32;
    let min = r.min(g).min(b) as u32;
    if max - min < 40 {
        return match lum {
            0..=30 => Color::Black,
            31..=110 => Color::DarkGray,
            111..=200 => Color::Gray,
            _ => Color::White,
        };
    }
    let bright = max > 180;
    match (r >= g && r >= b, g >= r && g >= b, b >= r && b >= g) {
        (true, _, _) if g > 120 && b < 80 => Color::Yellow,
        (true, _, _) => {
            if bright {
                Color::LightRed
            } else {
                Color::Red
            }
        }
        (_, true, _) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_survives_downgrade() {
        let t = Theme::for_level(ColorLevel::Ansi256);
        assert!(matches!(t.accent, Color::Indexed(_)));
        let t16 = Theme::for_level(ColorLevel::Ansi16);
        assert_eq!(t16.accent, Color::LightGreen);
        assert_eq!(t16.error, Color::LightRed);
        assert_eq!(t16.canvas, Color::Black);
    }

    #[test]
    fn hover_and_focus_are_distinct_styles() {
        let t = Theme::junie();
        let base = t.row(VisualState::default(), t.canvas);
        let hovered = t.row(
            VisualState {
                hovered: true,
                ..Default::default()
            },
            t.canvas,
        );
        let focused = t.row(
            VisualState {
                focused: true,
                ..Default::default()
            },
            t.canvas,
        );
        assert_ne!(base.bg, hovered.bg);
        assert_eq!(base.bg, focused.bg);
        assert!(focused.add_modifier.contains(Modifier::BOLD));
        assert!(!hovered.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn disabled_button_ignores_hover() {
        let t = Theme::junie();
        let d = VisualState {
            disabled: true,
            ..Default::default()
        };
        let dh = VisualState {
            disabled: true,
            hovered: true,
            ..Default::default()
        };
        assert_eq!(
            t.button(ButtonKind::Primary, d, t.surface),
            t.button(ButtonKind::Primary, dh, t.surface)
        );
    }
}
