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

/// How the cursor row of a list is marked. See [`Theme::cursor_row`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorStyle {
    /// Highlight fill, no focus bar: for lists whose focus belongs to an
    /// owner (popup lists under a search field, an editor, a select).
    #[default]
    Fill,
    /// Focus bar plus the focused-row tint: for lists that are themselves
    /// the focus stop.
    Bar,
}

/// A resolved interactive control: its body style and its gutter cell.
/// Returned by [`Theme::interactive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactive {
    pub style: Style,
    pub gutter: &'static str,
    pub gutter_style: Style,
    /// The control sits on a coloured fill (a cursor row, a primary or a
    /// destructive fill): secondary ink must use [`Theme::row_ink`].
    pub on_fill: bool,
}

/// A resolved list row (see [`Theme::cursor_row`]).
pub type CursorRow = Interactive;

/// The plane a control sits on. Every plane owns a rest → hover → pressed
/// [`Ladder`]; see [`Theme::ladder`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    /// The page itself (`Theme::canvas`).
    Canvas,
    /// Chrome and panels (`Theme::surface`).
    Surface,
    /// Cards and dialogs (`Theme::surface_elevated`).
    Card,
    /// Text field bodies (`Theme::field`).
    Field,
    /// Raised controls: secondary buttons, chips, status chips
    /// (`Theme::surface_overlay`).
    Raised,
    /// Menus and popovers (`Theme::popover`).
    Popover,
    /// The cursor row of a popup or menu list (`Theme::highlight`).
    Highlight,
    /// The cursor row of a destructive menu item.
    HighlightDanger,
    /// The accent fill of a primary control.
    Accent,
    /// A destructive fill.
    Danger,
}

impl Plane {
    pub const ALL: [Plane; 10] = [
        Plane::Canvas,
        Plane::Surface,
        Plane::Card,
        Plane::Field,
        Plane::Raised,
        Plane::Popover,
        Plane::Highlight,
        Plane::HighlightDanger,
        Plane::Accent,
        Plane::Danger,
    ];

    /// A coloured fill whose hue carries meaning (cursor, primary,
    /// destructive): in monochrome it renders as reversed video.
    pub fn is_fill(self) -> bool {
        matches!(
            self,
            Plane::Highlight | Plane::HighlightDanger | Plane::Accent | Plane::Danger
        )
    }

    /// The plane one step up: where an active tab, an open menu title or a
    /// raised control sits relative to its container.
    pub fn up(self) -> Plane {
        match self {
            Plane::Canvas | Plane::Surface | Plane::Card | Plane::Field => Plane::Raised,
            Plane::Raised | Plane::Popover => Plane::Popover,
            fill => fill,
        }
    }
}

/// What kind of interactive element is being resolved; selects its ink,
/// its body plane and whether it carries a focus gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A row of a list, tree, table or form that is itself the focus stop.
    Row,
    /// A list row whose cursor is marked per [`CursorStyle`].
    CursorRow(CursorStyle),
    Button(ButtonKind),
    /// A tab; `selected` is the active tab.
    Tab,
    /// A text-like clickable: menu-bar title, header link, overflow badge,
    /// status item, `+`. `selected` is "open".
    Title,
    /// A chip: filter chip, status chip, segment. `selected` is "on".
    Chip,
    /// A close or remove glyph (`×`).
    Close,
    /// A text field body (input, textarea, select field).
    Field,
    /// A draggable thumb (scrollbar). Foreground only.
    Handle,
    /// A draggable seam (splitter). Foreground only.
    Seam,
}

impl Kind {
    /// Kinds that paint only a foreground glyph and keep the container's
    /// background.
    pub fn is_glyph(self) -> bool {
        matches!(self, Kind::Handle | Kind::Seam)
    }
}

/// Rest → hover → pressed colours of one plane (or one glyph ink).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ladder {
    pub rest: Color,
    pub hover: Color,
    pub pressed: Color,
}

impl Ladder {
    pub const fn new(rest: Color, hover: Color, pressed: Color) -> Self {
        Self {
            rest,
            hover,
            pressed,
        }
    }

    /// The step for a state: pressed wins over hover.
    pub fn step(&self, hovered: bool, pressed: bool) -> Color {
        if pressed {
            self.pressed
        } else if hovered {
            self.hover
        } else {
            self.rest
        }
    }

    fn contains(&self, c: Color) -> bool {
        self.rest == c || self.hover == c || self.pressed == c
    }
}

/// Every plane's ladder plus the glyph inks of handles and seams. All
/// values are explicit per colour level (see [`Theme::for_level`]); never
/// derived by nearest-match at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ladders {
    pub canvas: Ladder,
    pub surface: Ladder,
    pub card: Ladder,
    pub field: Ladder,
    pub raised: Ladder,
    pub popover: Ladder,
    pub highlight: Ladder,
    pub highlight_danger: Ladder,
    pub accent: Ladder,
    pub danger: Ladder,
    /// Scrollbar thumb ink.
    pub handle: Ladder,
    /// Splitter seam ink.
    pub seam: Ladder,
}

/// Modifiers that carry state where colour cannot (16 colours, `NO_COLOR`).
/// Per colour level; configurable through [`Theme::mods`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateModifiers {
    /// Added to every hovered control.
    pub hover: Modifier,
    /// Added on hover when the plane's hover colour equals its rest colour.
    pub hover_flat: Modifier,
    /// Added to every pressed control on a neutral plane.
    pub pressed: Modifier,
    /// Added to a coloured fill (cursor, primary, destructive) at rest.
    pub fill: Modifier,
    /// Pressed on a fill: [`Self::fill`] is removed and this is added.
    pub pressed_fill: Modifier,
    /// Added to a focused control (with the `▎` gutter where it has one).
    pub focused: Modifier,
    /// Added to a disabled control.
    pub disabled: Modifier,
}

impl StateModifiers {
    /// Truecolor and 256 colours: the planes carry state, weight marks
    /// pressed and focus.
    pub const COLOR: Self = Self {
        hover: Modifier::empty(),
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::BOLD,
        fill: Modifier::empty(),
        pressed_fill: Modifier::BOLD,
        focused: Modifier::BOLD,
        disabled: Modifier::empty(),
    };
    /// 16 colours: four greys cannot carry three steps, so pressed is
    /// reversed video and hover on a grey plane underlines.
    pub const ANSI16: Self = Self {
        hover: Modifier::empty(),
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::REVERSED.union(Modifier::BOLD),
        fill: Modifier::empty(),
        pressed_fill: Modifier::REVERSED.union(Modifier::BOLD),
        focused: Modifier::BOLD,
        disabled: Modifier::empty(),
    };
    /// `NO_COLOR`: only modifiers survive. Hover underlines, pressed is
    /// reversed, fills are reversed and pressing one drops the reverse.
    pub const MONO: Self = Self {
        hover: Modifier::UNDERLINED,
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::REVERSED.union(Modifier::BOLD),
        fill: Modifier::REVERSED,
        pressed_fill: Modifier::BOLD.union(Modifier::UNDERLINED),
        focused: Modifier::BOLD,
        disabled: Modifier::DIM,
    };
}

/// Explicit per-level adjustments applied after the built-in token tables
/// by [`Theme::for_level_with`]: pin a token or a ladder for one colour
/// level instead of relying on the defaults.
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelOverrides {
    pub truecolor: Option<fn(&mut Theme)>,
    pub ansi256: Option<fn(&mut Theme)>,
    pub ansi16: Option<fn(&mut Theme)>,
    pub mono: Option<fn(&mut Theme)>,
}

impl LevelOverrides {
    /// Mono draws with [`Glyphs::ASCII`] and [`Strings::ASCII`]; every
    /// other level keeps its defaults. For a terminal that has no colour
    /// and no Unicode either.
    pub const ASCII_MONO: Self = Self {
        truecolor: None,
        ansi256: None,
        ansi16: None,
        mono: Some(ascii),
    };

    fn get(&self, level: ColorLevel) -> Option<fn(&mut Theme)> {
        match level {
            ColorLevel::TrueColor => self.truecolor,
            ColorLevel::Ansi256 => self.ansi256,
            ColorLevel::Ansi16 => self.ansi16,
            ColorLevel::Mono => self.mono,
        }
    }
}

/// A [`LevelOverrides`] hook that switches the theme to [`Glyphs::ASCII`].
pub fn ascii_glyphs(t: &mut Theme) {
    t.glyphs = Glyphs::ASCII;
}

/// A [`LevelOverrides`] hook that switches the theme to 7-bit output:
/// [`Glyphs::ASCII`] and [`Strings::ASCII`].
pub fn ascii(t: &mut Theme) {
    t.glyphs = Glyphs::ASCII;
    t.strings = Strings::ASCII;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorLevel {
    pub const ALL: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];

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
    pub const INPUT_HOVER: Color = rgb(0x2f2f34);
    pub const OVERLAY: Color = rgb(0x27272a);
    pub const POPOVER: Color = rgb(0x3f3f46);
    /// Menu selection: a cool blue that does not compete with the accent.
    pub const HIGHLIGHT: Color = rgb(0x2f5aa8);
    pub const WHITE: Color = rgb(0xffffff);
    pub const WHITE_70: Color = rgb(0xb3b3b3);
    pub const WHITE_50: Color = rgb(0x808080);
    pub const WHITE_30: Color = rgb(0x4d4d4d);
    pub const WHITE_15: Color = rgb(0x262626);
    /// Faint metadata: at least 3:1 on every dark plane (the 30% step was
    /// 2.1:1 on a card), and one step above the disabled grey.
    pub const WHITE_43: Color = rgb(0x737373);
    /// Subtle borders: visible on the canvas, surfaces and cards.
    pub const WHITE_23: Color = rgb(0x3a3a3a);
    /// Success: its own green, apart from the accent that marks focus.
    pub const SUCCESS: Color = rgb(0x3fb950);
    pub const GREEN: Color = rgb(0x48e054);
    pub const GREEN_80: Color = rgb(0x3ab343);
    pub const GREEN_60: Color = rgb(0x2e9a38);
    pub const GREEN_10: Color = rgb(0x0a1c0c);
    pub const ON_GREEN: Color = rgb(0x19191c);
    pub const RED: Color = rgb(0xe44545);
    pub const RED_20: Color = rgb(0x2e0f0f);
    /// Soft rose: a destructive label at rest on a neutral plane.
    pub const RED_SOFT: Color = rgb(0xe8a0a0);
    /// Deep red: the highlight of a destructive menu row under the cursor.
    pub const RED_45: Color = rgb(0x7a2a2a);
    pub const AMBER: Color = rgb(0xf59e09);
    pub const PURPLE: Color = rgb(0x8787ff);

    // Interactive ladders: rest → hover → pressed per plane. Adjacent steps
    // are at least 1.2:1 apart so each state reads on its own.
    pub const CANVAS_HOVER: Color = rgb(0x1c1c1f);
    pub const CANVAS_PRESSED: Color = rgb(0x303036);
    pub const CHROME_HOVER: Color = rgb(0x26262a);
    pub const CHROME_PRESSED: Color = rgb(0x38383e);
    pub const CARD_HOVER: Color = rgb(0x2a2a2f);
    pub const CARD_PRESSED: Color = rgb(0x3c3c43);
    pub const INPUT_PRESSED: Color = rgb(0x3a3a40);
    pub const OVERLAY_HOVER: Color = rgb(0x37373d);
    pub const OVERLAY_PRESSED: Color = rgb(0x4a4a52);
    pub const POPOVER_HOVER: Color = rgb(0x4f4f58);
    pub const POPOVER_PRESSED: Color = rgb(0x63636d);
    pub const HIGHLIGHT_HOVER: Color = rgb(0x3a6bc4);
    pub const HIGHLIGHT_PRESSED: Color = rgb(0x244a8c);
    pub const RED_45_HOVER: Color = rgb(0x8c3030);
    pub const RED_45_PRESSED: Color = rgb(0x5a1f1f);
    /// Destructive fill: dark enough for white text (≥ 4.5:1) at every step.
    pub const RED_FILL: Color = rgb(0xd93636);
    pub const RED_FILL_HOVER: Color = rgb(0xc22e2e);
    pub const RED_FILL_PRESSED: Color = rgb(0x9e2424);
    /// A destructive label at full strength on a raised or popover plane.
    pub const RED_LABEL: Color = rgb(0xff6b6b);
    /// Focused-selected tint and its hover step.
    pub const GREEN_25: Color = rgb(0x173f1c);
    pub const GREEN_35: Color = rgb(0x1e5024);
    /// Text selection and grid ranges.
    pub const BLUE_20: Color = rgb(0x1f3460);
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

    /// Text selection and grid range tint (see [`Self::selection`]).
    pub selection_bg: Color,
    /// Hover step of the focused-selected tint ([`Self::accent_bg`]): hover
    /// keeps the selection hue instead of replacing it.
    pub accent_bg_hover: Color,
    /// A destructive label at rest on a raised or popover plane (danger
    /// buttons): full strength, readable at ≥ 4.5:1.
    pub danger_label: Color,
    /// Rest → hover → pressed per plane. See [`Theme::interactive`].
    pub ladders: Ladders,
    /// Modifiers that carry state at this colour level.
    pub mods: StateModifiers,
    /// Whether the focused-selected row carries the `accent_bg` tint. Off
    /// at 16 colours and in monochrome, where the tint would collide with
    /// a hover or pressed plane; the selection marker glyph carries it.
    pub tint_selection: bool,
    /// Footer hint bar layout: alignment, the status's share of the row,
    /// compact mode and which keys are never dropped.
    pub hintbar: HintBarStyle,
    /// Gaps and container insets shared by every component.
    pub spacing: Spacing,
    /// Every glyph a widget draws: focus bar, markers, chevrons, checks,
    /// scrollbars, spinner and borders. Default [`Glyphs::UNICODE`].
    pub glyphs: Glyphs,
    /// The library's own copy (button labels, empty and loading states,
    /// validation messages, widget hints). Default [`Strings::DEFAULT`].
    pub strings: Strings,
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
            border_subtle: WHITE_23,
            border_strong: WHITE_30,
            text_primary: WHITE,
            text_secondary: WHITE_70,
            text_muted: WHITE_50,
            text_faint: WHITE_43,
            text_ghost: WHITE_15,
            text_on_accent: ON_GREEN,
            accent: GREEN,
            accent_hover: GREEN_80,
            accent_pressed: GREEN_60,
            accent_bg: GREEN_25,
            accent_bg_subtle: GREEN_10,
            focus: GREEN,
            disabled: WHITE_30,
            error: RED,
            error_bg: RED_20,
            warning: AMBER,
            success: SUCCESS,
            info: PURPLE,
            accent_bg_hover: GREEN_35,
            selection_bg: BLUE_20,
            danger_label: RED_LABEL,
            ladders: Ladders {
                canvas: Ladder::new(BLACK, CANVAS_HOVER, CANVAS_PRESSED),
                surface: Ladder::new(CHROME, CHROME_HOVER, CHROME_PRESSED),
                card: Ladder::new(CARD, CARD_HOVER, CARD_PRESSED),
                field: Ladder::new(INPUT, INPUT_HOVER, INPUT_PRESSED),
                raised: Ladder::new(OVERLAY, OVERLAY_HOVER, OVERLAY_PRESSED),
                popover: Ladder::new(POPOVER, POPOVER_HOVER, POPOVER_PRESSED),
                highlight: Ladder::new(HIGHLIGHT, HIGHLIGHT_HOVER, HIGHLIGHT_PRESSED),
                highlight_danger: Ladder::new(RED_45, RED_45_HOVER, RED_45_PRESSED),
                accent: Ladder::new(GREEN, GREEN_80, GREEN_60),
                danger: Ladder::new(RED_FILL, RED_FILL_HOVER, RED_FILL_PRESSED),
                handle: Ladder::new(WHITE_50, WHITE_70, WHITE),
                seam: Ladder::new(WHITE_30, WHITE_50, WHITE),
            },
            mods: StateModifiers::COLOR,
            tint_selection: true,
            hintbar: HintBarStyle::DEFAULT,
            spacing: Spacing::DEFAULT,
            glyphs: Glyphs::UNICODE,
            strings: Strings::DEFAULT,
        }
    }

    /// Junie theme resolved for the given colour capability.
    pub fn for_level(level: ColorLevel) -> Self {
        Self::for_level_with(level, &LevelOverrides::default())
    }

    /// [`Self::for_level`] plus explicit per-level adjustments.
    pub fn for_level_with(level: ColorLevel, overrides: &LevelOverrides) -> Self {
        let mut t = Self::junie();
        t.level = level;
        if level != ColorLevel::TrueColor {
            t.downgrade_all(level);
            t.apply_level_table(level);
        }
        if let Some(apply) = overrides.get(level) {
            apply(&mut t);
        }
        t
    }

    /// Nearest-match fallback for every token, ladder and modifier set.
    fn downgrade_all(&mut self, level: ColorLevel) {
        macro_rules! map {
            ($($f:ident),*) => { $( self.$f = downgrade(self.$f, level); )* };
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
            info,
            accent_bg_hover,
            danger_label,
            selection_bg
        );
        let l = &mut self.ladders;
        for ladder in [
            &mut l.canvas,
            &mut l.surface,
            &mut l.card,
            &mut l.field,
            &mut l.raised,
            &mut l.popover,
            &mut l.highlight,
            &mut l.highlight_danger,
            &mut l.accent,
            &mut l.danger,
            &mut l.handle,
            &mut l.seam,
        ] {
            ladder.rest = downgrade(ladder.rest, level);
            ladder.hover = downgrade(ladder.hover, level);
            ladder.pressed = downgrade(ladder.pressed, level);
        }
        self.mods = match level {
            ColorLevel::TrueColor | ColorLevel::Ansi256 => StateModifiers::COLOR,
            ColorLevel::Ansi16 => StateModifiers::ANSI16,
            ColorLevel::Mono => StateModifiers::MONO,
        };
        self.tint_selection = matches!(level, ColorLevel::TrueColor | ColorLevel::Ansi256);
    }

    /// Hand-picked tokens for reduced colour levels. Nearest-match collapses
    /// dark planes onto one grey and dark saturated fills onto greys, so
    /// every token a state depends on is pinned here explicitly.
    fn apply_level_table(&mut self, level: ColorLevel) {
        match level {
            ColorLevel::TrueColor => {}
            ColorLevel::Ansi256 => self.apply_256(),
            ColorLevel::Ansi16 => self.apply_16(),
            ColorLevel::Mono => self.apply_mono(),
        }
    }

    fn apply_256(&mut self) {
        let i = Color::Indexed;
        let lad = |r, h, p| Ladder::new(i(r), i(h), i(p));
        self.canvas = i(16);
        self.surface = i(232);
        self.surface_elevated = i(233);
        self.field = i(234);
        self.field_hover = i(237);
        self.surface_overlay = i(235);
        self.popover = i(237);
        self.highlight = i(25);
        self.highlight_danger = i(88);
        self.border_subtle = i(238);
        self.border_strong = i(242);
        self.text_primary = i(231);
        self.text_secondary = i(249);
        self.text_muted = i(245);
        self.text_faint = i(242);
        self.text_ghost = i(238);
        self.text_on_accent = i(232);
        self.accent = i(78);
        self.accent_hover = i(71);
        self.accent_pressed = i(65);
        self.accent_bg = i(22);
        self.accent_bg_hover = i(28);
        self.selection_bg = i(17);
        self.accent_bg_subtle = i(22);
        self.focus = i(78);
        self.success = i(71);
        self.disabled = i(239);
        self.error = i(167);
        self.error_soft = i(174);
        self.danger_label = i(203);
        self.error_bg = i(52);
        self.ladders = Ladders {
            canvas: lad(16, 234, 236),
            surface: lad(232, 235, 237),
            card: lad(233, 235, 237),
            field: lad(234, 237, 239),
            raised: lad(235, 238, 240),
            popover: lad(237, 239, 241),
            highlight: lad(25, 68, 24),
            highlight_danger: lad(88, 124, 52),
            accent: lad(78, 71, 65),
            danger: lad(160, 124, 88),
            handle: lad(245, 250, 231),
            seam: lad(239, 245, 231),
        };
    }

    fn apply_16(&mut self) {
        use Color::*;
        let black = Ladder::new(Black, DarkGray, Black);
        let grey = Ladder::new(DarkGray, DarkGray, DarkGray);
        self.canvas = Black;
        self.surface = Black;
        self.surface_elevated = Black;
        self.field = Black;
        self.field_hover = DarkGray;
        self.surface_overlay = DarkGray;
        self.popover = DarkGray;
        self.highlight = Blue;
        self.highlight_danger = Red;
        self.border_subtle = DarkGray;
        self.border_strong = Gray;
        self.text_primary = White;
        self.text_secondary = Gray;
        self.text_muted = Gray;
        // faint text must stay readable on the grey planes; hierarchy is
        // carried by weight at this level
        self.text_faint = Gray;
        self.text_ghost = DarkGray;
        self.text_on_accent = Black;
        self.accent = LightGreen;
        self.accent_hover = Green;
        self.accent_pressed = Green;
        self.accent_bg = Black;
        self.accent_bg_hover = DarkGray;
        self.selection_bg = Blue;
        self.accent_bg_subtle = Black;
        self.focus = LightGreen;
        self.success = Green;
        self.disabled = DarkGray;
        self.error = LightRed;
        self.error_soft = LightRed;
        self.danger_label = LightRed;
        self.error_bg = Black;
        self.ladders = Ladders {
            canvas: black,
            surface: black,
            card: black,
            field: black,
            raised: grey,
            popover: grey,
            highlight: Ladder::new(Blue, Blue, Blue),
            highlight_danger: Ladder::new(Red, Red, Red),
            accent: Ladder::new(LightGreen, Green, LightGreen),
            danger: Ladder::new(Red, Red, Red),
            handle: Ladder::new(DarkGray, Gray, White),
            seam: Ladder::new(DarkGray, Gray, White),
        };
    }

    fn apply_mono(&mut self) {
        use Color::*;
        // the backend strips every colour under NO_COLOR; these values only
        // keep planes and inks apart for code that compares them, and every
        // state is carried by `StateModifiers::MONO`
        let flat = |c| Ladder::new(c, c, c);
        self.canvas = Black;
        self.surface = Black;
        self.surface_elevated = Black;
        self.field = Black;
        self.field_hover = Black;
        self.surface_overlay = DarkGray;
        self.popover = DarkGray;
        self.highlight = DarkGray;
        self.highlight_danger = DarkGray;
        self.border_subtle = DarkGray;
        self.border_strong = Gray;
        self.text_primary = White;
        self.text_secondary = Gray;
        self.text_muted = Gray;
        self.text_faint = Gray;
        self.text_ghost = DarkGray;
        self.text_on_accent = Black;
        self.accent = White;
        self.accent_hover = White;
        self.accent_pressed = White;
        self.accent_bg = Black;
        self.accent_bg_hover = Black;
        self.selection_bg = DarkGray;
        self.accent_bg_subtle = Black;
        self.focus = White;
        self.success = White;
        self.disabled = DarkGray;
        self.error = White;
        self.error_soft = White;
        self.danger_label = White;
        self.error_bg = Black;
        self.ladders = Ladders {
            canvas: flat(Black),
            surface: flat(Black),
            card: flat(Black),
            field: flat(Black),
            raised: flat(DarkGray),
            popover: flat(DarkGray),
            highlight: flat(DarkGray),
            highlight_danger: flat(DarkGray),
            accent: flat(White),
            danger: flat(Gray),
            handle: Ladder::new(Gray, Gray, White),
            seam: Ladder::new(DarkGray, Gray, White),
        };
    }

    /// Replace one plane's ladder.
    pub fn with_ladder(mut self, plane: Plane, ladder: Ladder) -> Self {
        *self.ladder_mut(plane) = ladder;
        self
    }

    /// Replace the state modifiers.
    /// The same theme drawing with `glyphs` (for example [`Glyphs::ASCII`]
    /// on a terminal without Unicode).
    pub const fn with_glyphs(mut self, glyphs: Glyphs) -> Self {
        self.glyphs = glyphs;
        self
    }

    /// The same theme with the library copy `strings` (a translation, or
    /// [`Strings::ASCII`]).
    pub const fn with_strings(mut self, strings: Strings) -> Self {
        self.strings = strings;
        self
    }

    pub fn with_state_modifiers(mut self, mods: StateModifiers) -> Self {
        self.mods = mods;
        self
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
        // a dark plane under the cell: canvas-coloured ink there is a blank,
        // not text; on any lighter fill it is reversed text and must stay
        let dark_plane = style.bg.is_none_or(|c| {
            [
                self.canvas,
                self.surface,
                self.surface_elevated,
                self.field,
                self.field_hover,
            ]
            .contains(&c)
        });
        // Preserve cells with no visible foreground when dimming a plane.
        let blank = match style.fg {
            Some(c) if Some(c) == style.bg => true,
            Some(c) => (c == self.canvas || c == self.surface) && dark_plane,
            None => false,
        };
        let fg = match style.fg {
            _ if blank => bg,
            Some(c) if c == self.canvas || c == self.surface => self.text_muted,
            // focus and accent cues (an active tab rule, a selection
            // marker) drop to the faint tier: under a modal nothing on the
            // page may still read as "here" or "active"
            Some(c) if c == self.accent || c == self.focus => self.text_faint,
            Some(c) if c == self.text_primary || c == self.error || c == self.warning => {
                self.text_muted
            }
            Some(c) if c == self.text_secondary || c == self.text_on_accent => self.text_faint,
            _ => self.text_ghost,
        };
        // 16 colours and monochrome have no tiers left to dim through: the
        // ladder lands on the backdrop's own grey. Keep a readable ink and
        // dim it with the intensity attribute instead.
        if !blank && (fg == bg || matches!(self.level, ColorLevel::Ansi16 | ColorLevel::Mono)) {
            let ink = if self.text_muted != bg {
                self.text_muted
            } else {
                self.text_primary
            };
            return Style::new().fg(ink).bg(bg).add_modifier(Modifier::DIM);
        }
        Style::new().fg(fg).bg(bg)
    }

    // --- component resolvers ----------------------------------------------
    //
    // All resolvers take the container background so a control looks right
    // on the canvas, on a surface, or inside a dialog.

    /// Disabled text uses dim intensity in monochrome, where colour alone
    /// cannot distinguish an unavailable control.
    pub fn disabled_style(&self) -> Style {
        let style = Style::new().fg(self.disabled);
        if self.level == ColorLevel::Mono {
            style.add_modifier(Modifier::DIM)
        } else {
            style
        }
    }

    // --- interactive state model -----------------------------------------
    //
    // One resolver for every clickable: rest, hover and pressed walk the
    // plane's ladder; focus is the `▎` gutter plus weight; selection is a
    // marker glyph plus (on the focused list only) the accent tint;
    // disabled ignores hover and press. Widgets never pick state colours.

    /// The ladder of one plane.
    pub fn ladder(&self, plane: Plane) -> Ladder {
        let l = &self.ladders;
        match plane {
            Plane::Canvas => l.canvas,
            Plane::Surface => l.surface,
            Plane::Card => l.card,
            Plane::Field => l.field,
            Plane::Raised => l.raised,
            Plane::Popover => l.popover,
            Plane::Highlight => l.highlight,
            Plane::HighlightDanger => l.highlight_danger,
            Plane::Accent => l.accent,
            Plane::Danger => l.danger,
        }
    }

    fn ladder_mut(&mut self, plane: Plane) -> &mut Ladder {
        let l = &mut self.ladders;
        match plane {
            Plane::Canvas => &mut l.canvas,
            Plane::Surface => &mut l.surface,
            Plane::Card => &mut l.card,
            Plane::Field => &mut l.field,
            Plane::Raised => &mut l.raised,
            Plane::Popover => &mut l.popover,
            Plane::Highlight => &mut l.highlight,
            Plane::HighlightDanger => &mut l.highlight_danger,
            Plane::Accent => &mut l.accent,
            Plane::Danger => &mut l.danger,
        }
    }

    /// The plane a container background belongs to. Accepts the plane
    /// tokens and any step of their ladders (a control inside a hovered
    /// row). A colour that is no plane is a caller bug.
    pub fn plane_of(&self, bg: Color) -> Plane {
        let tokens = [
            (self.canvas, Plane::Canvas),
            (self.surface, Plane::Surface),
            (self.surface_elevated, Plane::Card),
            (self.field, Plane::Field),
            (self.surface_overlay, Plane::Raised),
            (self.popover, Plane::Popover),
            (self.field_hover, Plane::Field),
            (self.highlight, Plane::Highlight),
            (self.highlight_danger, Plane::HighlightDanger),
            (self.accent, Plane::Accent),
            (self.error, Plane::Danger),
            // tinted rows: controls on them read as on the card
            (self.accent_bg, Plane::Card),
            (self.accent_bg_hover, Plane::Card),
            (self.accent_bg_subtle, Plane::Card),
            (self.error_bg, Plane::Card),
        ];
        if let Some(&(_, p)) = tokens.iter().find(|(c, _)| *c == bg) {
            return p;
        }
        if let Some(&p) = Plane::ALL.iter().find(|&&p| self.ladder(p).contains(bg)) {
            return p;
        }
        debug_assert!(
            matches!(bg, Color::Reset),
            "{bg:?} is not a theme plane; pass a plane token"
        );
        Plane::Canvas
    }

    /// Resolve an interactive element of `kind` in state `s` sitting on
    /// `plane` (its container). States apply in a fixed order: disabled
    /// (wins, ignores hover and press), pressed, hover, selected, focused.
    pub fn interactive(&self, kind: Kind, s: VisualState, plane: Plane) -> Interactive {
        let m = self.mods;
        // the plane the control's own body paints
        let body = match kind {
            Kind::Button(ButtonKind::Primary) => Plane::Accent,
            Kind::Button(ButtonKind::Subtle) => plane,
            Kind::Button(_) | Kind::Chip => Plane::Raised,
            Kind::CursorRow(CursorStyle::Fill) if s.focused && !s.disabled => {
                if s.error {
                    Plane::HighlightDanger
                } else {
                    Plane::Highlight
                }
            }
            Kind::Tab | Kind::Title if s.selected => plane.up(),
            Kind::Field => Plane::Field,
            _ => plane,
        };
        let glyph = kind.is_glyph();

        if s.disabled {
            // the kind's rest plane, never a fill: a disabled primary is a
            // grey control, not a dim green one
            let rest = if body.is_fill() {
                match kind {
                    Kind::Button(_) => self.ladder(Plane::Raised).rest,
                    _ => self.ladder(plane).rest,
                }
            } else {
                self.ladder(body).rest
            };
            let bg = rest;
            let mut fg = self.disabled;
            let mut dim = m.disabled;
            if fg == bg {
                // a grey plane at 16 colours: the disabled grey is the plane
                // itself, so keep the plane and dim a readable ink instead
                fg = self.text_muted;
                dim |= Modifier::DIM;
            }
            let mut style = Style::new().fg(fg).add_modifier(dim);
            if glyph {
                // a glyph keeps the container's plane; dim intensity tells
                // it from a seam at rest
                style = style.add_modifier(Modifier::DIM);
            } else {
                style = style.bg(bg);
            }
            return Interactive {
                style,
                gutter: " ",
                gutter_style: Style::new().fg(bg).bg(bg),
                on_fill: false,
            };
        }

        if glyph {
            let lad = match kind {
                Kind::Handle => self.ladders.handle,
                _ => self.ladders.seam,
            };
            // focus raises the ink one step and hover one more; pressed is
            // the top step plus weight
            let fg = if s.pressed {
                lad.pressed
            } else if s.hovered {
                if s.focused { lad.pressed } else { lad.hover }
            } else if s.focused {
                lad.hover
            } else {
                lad.rest
            };
            let mut style = Style::new().fg(fg);
            if s.pressed {
                style = style.add_modifier(m.pressed);
            } else if s.hovered {
                style = style.add_modifier(m.hover | m.hover_flat);
            }
            return Interactive {
                style,
                gutter: " ",
                gutter_style: Style::new(),
                on_fill: false,
            };
        }

        let lad = self.ladder(body);
        let fill = body.is_fill();
        let mut mods = Modifier::empty();
        if fill {
            mods |= m.fill;
        }
        let row_like = matches!(kind, Kind::Row | Kind::CursorRow(CursorStyle::Bar));
        let tinted = row_like && s.selected && s.focused && self.tint_selection;
        let (rest, hover) = if tinted {
            (self.accent_bg, self.accent_bg_hover)
        } else {
            (lad.rest, lad.hover)
        };
        // a destructive button presses into the danger fill
        let danger_press = kind == Kind::Button(ButtonKind::Danger) && s.pressed;
        let mut bg = rest;
        if s.pressed {
            bg = if danger_press {
                self.ladders.danger.pressed
            } else {
                lad.pressed
            };
            if fill {
                mods.remove(m.fill);
                mods |= m.pressed_fill;
            } else {
                mods |= m.pressed;
            }
        } else if s.hovered {
            bg = hover;
            mods |= m.hover;
            if hover == rest {
                mods |= m.hover_flat;
            }
        }

        // ink
        let lifted = s.hovered || s.pressed || s.focused;
        let mut fg = match kind {
            Kind::Row | Kind::CursorRow(_) | Kind::Field => {
                if s.error && !fill {
                    self.error
                } else {
                    self.text_primary
                }
            }
            Kind::Button(ButtonKind::Primary) => self.text_on_accent,
            Kind::Button(ButtonKind::Secondary | ButtonKind::Toggle) => self.text_primary,
            Kind::Button(ButtonKind::Danger) => {
                if danger_press {
                    self.text_primary
                } else {
                    self.danger_label
                }
            }
            Kind::Button(ButtonKind::Subtle)
            | Kind::Tab
            | Kind::Title
            | Kind::Chip
            | Kind::Close => {
                if lifted || s.selected {
                    self.text_primary
                } else {
                    self.text_secondary
                }
            }
            Kind::Handle | Kind::Seam => unreachable!("glyph kinds resolve above"),
        };
        if s.busy {
            fg = if fill {
                self.text_on_accent
            } else {
                self.text_secondary
            };
        }
        fg = self.legible(fg, bg);

        // weight: focus, the always-bold fills and the active tab
        let weighted = match kind {
            Kind::Button(ButtonKind::Primary) => true,
            Kind::CursorRow(CursorStyle::Fill) => fill,
            Kind::Tab | Kind::Title | Kind::Chip => s.selected || s.focused,
            Kind::Field | Kind::Close => false,
            _ => s.focused,
        };
        if weighted && !s.busy {
            mods |= m.focused;
        }
        let style = Style::new().fg(fg).bg(bg).add_modifier(mods);

        let bar = s.focused
            && !matches!(
                kind,
                Kind::CursorRow(CursorStyle::Fill) | Kind::Close | Kind::Field
            );
        let gutter_fg = if !bar {
            bg
        } else if fill || danger_press {
            // on a fill the bar takes the fill's ink: the accent green on
            // a green or red fill is unreadable
            fg
        } else {
            self.focus
        };
        Interactive {
            style,
            gutter: if bar { self.glyphs.focus_bar } else { " " },
            gutter_style: Style::new().fg(gutter_fg).bg(bg),
            on_fill: fill,
        }
    }

    /// `fg` if it reads on `bg` (≥ 4.5:1), otherwise the most legible of
    /// the primary and on-accent inks. Colours the terminal resolves (named
    /// and indexed) are measured against the xterm defaults.
    pub fn legible(&self, fg: Color, bg: Color) -> Color {
        let Some(c) = contrast_ratio(fg, bg) else {
            return fg;
        };
        if c >= 4.5 {
            return fg;
        }
        [self.text_primary, self.text_on_accent]
            .into_iter()
            .filter_map(|ink| contrast_ratio(ink, bg).map(|c| (ink, c)))
            .fold(
                (fg, c),
                |best, cand| if cand.1 > best.1 { cand } else { best },
            )
            .0
    }

    /// Row-like control (nav item, list item, table row, tree node).
    pub fn row(&self, s: VisualState, bg: Color) -> Style {
        self.interactive(Kind::Row, s, self.plane_of(bg)).style
    }

    /// A row of a list whose keyboard focus belongs to something else (a
    /// popup list driven by a search field, an editor or a select field).
    /// With [`CursorStyle::Fill`] the cursor row is a highlight fill and
    /// carries no focus bar, so the owner's `▎` stays the only focus
    /// indicator on screen; [`CursorStyle::Bar`] is the in-page list look.
    pub fn cursor_row(&self, s: VisualState, bg: Color, cursor: CursorStyle) -> CursorRow {
        self.interactive(Kind::CursorRow(cursor), s, self.plane_of(bg))
    }

    /// Secondary ink (details, groups, tags, kind glyphs) on a row from
    /// [`Self::cursor_row`]: muted tones are unreadable on the highlight
    /// fill, so there they become primary text and keep hierarchy by
    /// weight instead.
    pub fn row_ink(&self, row: &CursorRow, ink: Color) -> Color {
        if row.on_fill { self.text_primary } else { ink }
    }

    /// Ink for a chosen-value mark (a checked box, the on radio, a selected
    /// node): the accent only while its control has keyboard focus, primary
    /// text otherwise, so an active-but-unfocused value never competes
    /// with the one focus indicator.
    pub fn chosen(&self, focused: bool) -> Color {
        if focused {
            self.accent
        } else {
            self.text_primary
        }
    }

    /// One ladder step up from `bg`: rest lifts to hover, hover to pressed.
    /// Prefer [`Self::interactive`], which picks the step from the state.
    pub fn lift(&self, bg: Color) -> Color {
        let l = self.ladder(self.plane_of(bg));
        if bg == l.hover { l.pressed } else { l.hover }
    }

    /// Focus gutter content, independent of terminal colour support.
    ///
    /// Always render the returned cell, including the blank for unfocused or
    /// disabled controls. Painting a focus bar in the background colour cannot
    /// hide it when a backend suppresses colours (for example with `NO_COLOR`).
    /// Pair this with [`Self::gutter`] for its style.
    pub fn gutter_symbol(&self, s: VisualState) -> &'static str {
        if s.focused && !s.disabled {
            self.glyphs.focus_bar
        } else {
            " "
        }
    }

    /// Style for [`Self::gutter_symbol`]. `on_accent` is used when the control
    /// itself is filled with the accent (primary button): the bar then takes
    /// the on-accent ink, never the accent on the accent.
    pub fn gutter(&self, s: VisualState, bg: Color, on_accent: bool) -> Style {
        let fg = if !s.focused || s.disabled {
            bg
        } else if on_accent {
            self.text_on_accent
        } else {
            self.focus
        };
        Style::new().fg(fg).bg(bg)
    }

    pub fn button(&self, kind: ButtonKind, s: VisualState, bg: Color) -> Style {
        self.interactive(Kind::Button(kind), s, self.plane_of(bg))
            .style
    }

    /// Text field body (input, textarea, editable cell). Editing suppresses
    /// hover: the caret is the state.
    pub fn field_style(&self, s: VisualState) -> Style {
        let s = VisualState {
            hovered: s.hovered && !s.editing,
            focused: false,
            ..s
        };
        self.interactive(Kind::Field, s, Plane::Field).style
    }

    pub fn placeholder(&self, s: VisualState) -> Style {
        self.field_style(s).fg(if s.disabled {
            self.disabled
        } else {
            self.text_muted
        })
    }

    /// A close glyph (`×`) inside a control styled `host`: at rest it takes
    /// the host's plane, hovered or pressed it walks the close ladder of the
    /// plane it sits on, so its hover never reads as the host's hover.
    pub fn close_glyph(&self, s: VisualState, host: Style) -> Style {
        let bg = host.bg.unwrap_or(self.canvas);
        let s = VisualState {
            focused: false,
            ..s
        };
        if s.hovered || s.pressed {
            self.interactive(Kind::Close, s, self.plane_of(bg)).style
        } else {
            host.fg(self.legible(self.text_secondary, bg))
                .remove_modifier(Modifier::BOLD)
        }
    }

    /// The cursor cell of a table or data grid: the cursor fill on the cell
    /// only (never the white flood a pressed control once used), the
    /// destructive fill for a cell with an error.
    pub fn cell_cursor(&self, error: bool, pressed: bool) -> Style {
        let s = VisualState {
            focused: true,
            error,
            pressed,
            ..Default::default()
        };
        self.interactive(Kind::CursorRow(CursorStyle::Fill), s, Plane::Canvas)
            .style
    }

    /// A rule or separator that reads on `bg`: the subtle border on the dark
    /// planes, the strong one on raised, popover and field planes where the
    /// subtle one disappears.
    pub fn rule_on(&self, bg: Color) -> Color {
        let rule = match self.plane_of(bg) {
            Plane::Raised | Plane::Popover | Plane::Field => self.border_strong,
            _ => self.border_subtle,
        };
        if rule == bg { self.text_muted } else { rule }
    }

    /// The current search or find match: the accent fill with on-accent ink,
    /// apart from the selection tint and from the other matches (underlined).
    /// Reversed video only where colour is gone.
    pub fn match_current(&self) -> Style {
        let style = Style::new()
            .fg(self.text_on_accent)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD);
        if self.level == ColorLevel::Mono {
            style.add_modifier(Modifier::REVERSED)
        } else {
            style
        }
    }

    /// Text selection and a grid range: a cool tint of its own, apart
    /// from the pressed planes and from the current find match (popover).
    /// Stays visible when `NO_COLOR` suppresses colours.
    pub fn selection(&self) -> Style {
        let style = Style::new().fg(self.text_primary).bg(self.selection_bg);
        if self.level == ColorLevel::Mono {
            style.add_modifier(Modifier::REVERSED)
        } else {
            style
        }
    }

    pub fn scrollbar_track(&self) -> Style {
        Style::new().fg(self.border_subtle)
    }

    pub fn scrollbar_thumb(&self, focused: bool, hovered: bool) -> Style {
        let s = VisualState {
            focused,
            hovered,
            ..Default::default()
        };
        self.interactive(Kind::Handle, s, Plane::Canvas).style
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

/// Where the footer hints sit in their row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HintAlign {
    /// Lead from the left edge (after the mode badge), like a desktop
    /// status bar.
    #[default]
    Start,
    /// Centre the hints that fit in the row.
    Center,
}

/// Every glyph the widgets draw, so a terminal without Unicode (or an app
/// with its own taste) can swap them in one place. Set on
/// [`Theme::glyphs`]; [`Glyphs::UNICODE`] is the default look and
/// [`Glyphs::ASCII`] draws with 7-bit characters only.
///
/// Each glyph keeps the cell width of its [`Glyphs::UNICODE`] counterpart
/// (one cell unless noted), because widgets lay out around it. A
/// replacement set must do the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    /// Keyboard-focus gutter ([`Theme::gutter_symbol`]). `▎`.
    pub focus_bar: &'static str,
    /// The current or chosen row of a list, menu or select. `›`.
    pub current: &'static str,
    /// A select field whose list is open. `▴`.
    pub chevron_open: &'static str,
    /// A select field whose list is closed. `▾`.
    pub chevron_closed: &'static str,
    /// An expanded tree branch. `▾`.
    pub disclosure_open: &'static str,
    /// A collapsed tree branch. `▸`.
    pub disclosure_closed: &'static str,
    /// More rows above a scrolled list or menu. `▴`.
    pub more_above: &'static str,
    /// More rows below a scrolled list or menu. `▾`.
    pub more_below: &'static str,
    /// Ascending sort on a column header. `▴`.
    pub sort_asc: &'static str,
    /// Descending sort on a column header. `▾`.
    pub sort_desc: &'static str,
    /// Cut text and the overflow button. `…`.
    pub ellipsis: &'static str,
    /// Between parts of a status or summary (three cells). ` · `.
    pub separator: &'static str,
    /// Checked checkbox (three cells). `[✓]`.
    pub check_on: &'static str,
    /// Unchecked checkbox (three cells). `[ ]`.
    pub check_off: &'static str,
    /// Partly checked checkbox (three cells). `[–]`.
    pub check_mixed: &'static str,
    /// A check mark: a checked narrow checkbox, a chosen row of a
    /// multi-select list. `✓`.
    pub tick: &'static str,
    /// An unchecked narrow checkbox. `□`.
    pub check_off_narrow: &'static str,
    /// A partly checked narrow checkbox; also a range dash. `–`.
    pub dash: &'static str,
    /// Chosen radio button (three cells). `(●)`.
    pub radio_on: &'static str,
    /// Radio button (three cells). `( )`.
    pub radio_off: &'static str,
    /// An "on" dot: narrow radio and toggle, toggle buttons. `●`.
    pub dot_on: &'static str,
    /// An "off" dot. `○`.
    pub dot_off: &'static str,
    /// Switch turned on (three cells). `──●`.
    pub toggle_on: &'static str,
    /// Switch turned off (three cells). `○──`.
    pub toggle_off: &'static str,
    /// Close a tab or remove a chip. `×`.
    pub close: &'static str,
    /// Items hidden to the left of a strip. `‹`.
    pub overflow_left: &'static str,
    /// Items hidden to the right of a strip. `›`.
    pub overflow_right: &'static str,
    /// The menu-bar title that holds the titles that do not fit
    /// ([`crate::widgets::menu::Overflow::Chevron`]). `»`.
    pub more_titles: &'static str,
    /// Vertical scrollbar track; also a vertical divider. `│`.
    pub scroll_track: &'static str,
    /// Vertical scrollbar thumb; also a held divider. `┃`.
    pub scroll_thumb: &'static str,
    /// Horizontal scrollbar track; also a horizontal divider. `─`.
    pub scroll_h_track: &'static str,
    /// Horizontal scrollbar thumb; also a held divider. `━`.
    pub scroll_h_thumb: &'static str,
    /// A scrollbar thumb being dragged, both axes. `█`.
    pub scroll_held: &'static str,
    /// Spinner animation frames, one cell each. Braille dots.
    pub spinner: &'static [&'static str],
    /// A finished task or step. `✓`.
    pub done: &'static str,
    /// A cut or renamed value continues: `old → new`. `→`.
    pub arrow: &'static str,
    /// Filled part of a horizontal bar (progress, meter, active tab). `━`.
    pub bar_fill: &'static str,
    /// Empty part of a horizontal bar; also a separator rule. `─`.
    pub bar_empty: &'static str,
    /// Unsaved changes. `•`.
    pub dirty: &'static str,
    /// A removed row or count. `−`.
    pub removed: &'static str,
    /// A warning marker. `▲`.
    pub warning: &'static str,
    /// A paused task. `‖`.
    pub paused: &'static str,
    /// No value. `—`.
    pub none: &'static str,
    /// A masked character in a secret field. `•`.
    pub mask: &'static str,
    /// The default bullet of a picker item. `·`.
    pub bullet: &'static str,
    /// A filtered column header. `∇`.
    pub filter: &'static str,
    /// A primary-key column header. `⚷`.
    pub key: &'static str,
    /// A newline shown inside a one-line cell. `↵`.
    pub newline: &'static str,
    /// A tab shown inside a one-line cell. `⇥`.
    pub tab: &'static str,
    /// Any other control character shown inside a cell. `·`.
    pub control: &'static str,
    /// Arrow-key names in hints: up, down, left, right. `↑ ↓ ← →`.
    pub arrows: [&'static str; 4],
    /// Popup, dialog, menu and panel borders. Rounded box drawing.
    pub border: ratatui::symbols::border::Set<'static>,
}

impl Glyphs {
    /// Today's look: box drawing, geometric shapes and Braille.
    pub const UNICODE: Self = Self {
        focus_bar: "▎",
        current: "›",
        chevron_open: "▴",
        chevron_closed: "▾",
        disclosure_open: "▾",
        disclosure_closed: "▸",
        more_above: "▴",
        more_below: "▾",
        sort_asc: "▴",
        sort_desc: "▾",
        ellipsis: "…",
        separator: " · ",
        check_on: "[✓]",
        check_off: "[ ]",
        check_mixed: "[–]",
        tick: "✓",
        check_off_narrow: "□",
        dash: "–",
        radio_on: "(●)",
        radio_off: "( )",
        dot_on: "●",
        dot_off: "○",
        toggle_on: "──●",
        toggle_off: "○──",
        close: "×",
        overflow_left: "‹",
        overflow_right: "›",
        more_titles: "»",
        scroll_track: "│",
        scroll_thumb: "┃",
        scroll_h_track: "─",
        scroll_h_thumb: "━",
        scroll_held: "█",
        spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        done: "✓",
        arrow: "→",
        bar_fill: "━",
        bar_empty: "─",
        dirty: "•",
        removed: "−",
        warning: "▲",
        paused: "‖",
        none: "—",
        mask: "•",
        bullet: "·",
        filter: "∇",
        key: "⚷",
        newline: "↵",
        tab: "⇥",
        control: "·",
        arrows: ["↑", "↓", "←", "→"],
        border: ratatui::symbols::border::ROUNDED,
    };

    /// 7-bit ASCII only, for terminals and fonts without Unicode. Widths
    /// match [`Self::UNICODE`], so layouts do not move.
    pub const ASCII: Self = Self {
        focus_bar: "|",
        current: ">",
        chevron_open: "^",
        chevron_closed: "v",
        disclosure_open: "v",
        disclosure_closed: ">",
        more_above: "^",
        more_below: "v",
        sort_asc: "^",
        sort_desc: "v",
        ellipsis: "~",
        separator: " | ",
        check_on: "[x]",
        check_off: "[ ]",
        check_mixed: "[-]",
        tick: "x",
        check_off_narrow: ".",
        dash: "-",
        radio_on: "(*)",
        radio_off: "( )",
        dot_on: "*",
        dot_off: "o",
        toggle_on: "--*",
        toggle_off: "o--",
        close: "x",
        overflow_left: "<",
        overflow_right: ">",
        more_titles: ">",
        scroll_track: "|",
        scroll_thumb: "#",
        scroll_h_track: "-",
        scroll_h_thumb: "=",
        scroll_held: "@",
        spinner: &["|", "/", "-", "\\"],
        done: "+",
        arrow: ">",
        bar_fill: "=",
        bar_empty: "-",
        dirty: "*",
        removed: "-",
        warning: "!",
        paused: "=",
        none: "-",
        mask: "*",
        bullet: ".",
        filter: "f",
        key: "k",
        newline: "~",
        tab: ">",
        control: ".",
        arrows: ["^", "v", "<", ">"],
        border: ratatui::symbols::border::Set {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        },
    };

    /// The spinner frame for animation tick `tick`.
    pub fn spinner_frame(&self, tick: u64) -> &'static str {
        match self.spinner.len() {
            0 => " ",
            n => self.spinner[(tick % n as u64) as usize],
        }
    }

    /// `s` with every arrow-key character (`↑ ↓ ← →`) replaced by
    /// [`Self::arrows`] and the range dash `–` by [`Self::dash`], for key
    /// labels built from the Unicode names.
    pub fn key_label<'a>(&self, s: &'a str) -> std::borrow::Cow<'a, str> {
        if self.arrows == Self::UNICODE.arrows && self.dash == Self::UNICODE.dash {
            return std::borrow::Cow::Borrowed(s);
        }
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '↑' => out.push_str(self.arrows[0]),
                '↓' => out.push_str(self.arrows[1]),
                '←' => out.push_str(self.arrows[2]),
                '→' => out.push_str(self.arrows[3]),
                '–' => out.push_str(self.dash),
                c => out.push(c),
            }
        }
        std::borrow::Cow::Owned(out)
    }
}

impl Default for Glyphs {
    fn default() -> Self {
        Self::UNICODE
    }
}

/// The English copy the library itself draws: button labels, empty and
/// loading states, validation messages and the hints widgets add to a
/// footer. Set on [`Theme::strings`] to translate or reword it in one
/// place; [`Strings::DEFAULT`] is today's copy and [`Strings::ASCII`] the
/// same words with 7-bit punctuation.
///
/// A widget label still holding the text it was built with follows the
/// theme's strings when drawn ([`Strings::follow`]); a per-widget builder (`Dialog::cancel_label`,
/// `Picker::placeholder`, `TextInput::required_message`,
/// `DataGrid::bar_labels`, ...) sets text that the theme no longer
/// changes. Templates fill each `{}` in order (see [`Strings::fill`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strings {
    /// The cancel button of dialogs. `Cancel`.
    pub cancel: &'static str,
    /// The save button of form dialogs. `Save`.
    pub save: &'static str,
    /// The typed acknowledgement of a dangerous dialog; `{}` is the token.
    /// `Type {} to confirm`.
    pub type_to_confirm: &'static str,
    /// The hint while the acknowledgement is unmet; `{}` is the token.
    /// `"{}" to confirm` (after the key `Type`).
    pub type_to_confirm_hint: &'static str,
    /// The error under a typed acknowledgement when Enter is pressed and
    /// the text does not match the token. `Does not match`.
    pub ack_mismatch: &'static str,
    /// The last shown line of a cut preview; `{}` is the hidden count.
    /// `… {} more`.
    pub more_cut: &'static str,
    /// A count of hidden lines after an arrow glyph. `{} more`.
    pub more_count: &'static str,
    /// A required field left empty. `Required`.
    pub required: &'static str,
    /// The suffix on a field label that is not required. `optional`.
    pub optional: &'static str,
    /// The primary button of a step in a choice dialog. `Next`.
    pub next: &'static str,
    /// A search with no result. `No matches`.
    pub no_matches: &'static str,
    /// The query field of a searchable picker. `Type to search…`.
    pub search_placeholder: &'static str,
    /// An empty list. `Nothing here yet`.
    pub empty_list: &'static str,
    /// An empty table or grid. `No rows`.
    pub no_rows: &'static str,
    /// A grid waiting for its first rows. `Loading rows…`.
    pub loading_rows: &'static str,
    /// The grid's last row while more rows load (after a spinner).
    /// `fetching…`.
    pub fetching: &'static str,
    /// The grid's last row when more rows can be fetched (after an arrow);
    /// `{}` is the loaded count. `{} loaded · Enter fetches more`.
    pub fetch_more: &'static str,
    /// The grid's pending-changes bar buttons. `Preview SQL`.
    pub preview_sql: &'static str,
    /// `Discard`.
    pub discard: &'static str,
    /// `Save` (the grid's bar).
    pub save_changes: &'static str,
    /// Diff review columns. `Old`.
    pub diff_old: &'static str,
    /// `New`.
    pub diff_new: &'static str,
    /// A diff view with no file. `No file selected`.
    pub diff_no_file: &'static str,
    /// A file whose diff has no text. `(no textual changes)`.
    pub diff_no_changes: &'static str,
    /// The review layout's title suffix. `review`.
    pub diff_review: &'static str,
    /// Reveal hint while the text is cut. `Full text`.
    pub reveal_open: &'static str,
    /// Reveal hint while the full text shows. `Collapse`.
    pub reveal_close: &'static str,
    /// Dialog hint: arrows move between actions. `Choose`.
    pub hint_choose: &'static str,
    /// Dialog hint: arrows scroll the body. `Scroll`.
    pub hint_scroll: &'static str,
    /// Dialog hint: Shift+arrows scroll the code preview. `Scroll code`.
    pub hint_scroll_code: &'static str,
    /// Hint: Esc cancels. `Cancel`.
    pub hint_cancel: &'static str,
    /// Hint: Esc closes a dialog without a cancel action. `Close`.
    pub hint_close: &'static str,
    /// Hint: Enter or Tab moves on. `Next`.
    pub hint_next: &'static str,
    /// Hint: Enter applies a field edit with no primary action. `Apply`.
    pub hint_apply: &'static str,
    /// Hint: Enter starts editing a field. `Edit`.
    pub hint_edit: &'static str,
    /// The help overlay's title. `Keyboard shortcuts`.
    pub help_title: &'static str,
    /// The help overlay's footer. `↑↓ Scroll · Esc Close`.
    pub help_footer: &'static str,
    /// The reason on a menu item a preview or demo does not implement.
    /// `Not in this preview`.
    pub not_in_preview: &'static str,
}

impl Strings {
    /// Today's copy.
    pub const DEFAULT: Self = Self {
        cancel: "Cancel",
        save: "Save",
        type_to_confirm: "Type {} to confirm",
        type_to_confirm_hint: "\"{}\" to confirm",
        ack_mismatch: "Does not match",
        more_cut: "… {} more",
        more_count: "{} more",
        required: "Required",
        optional: "optional",
        next: "Next",
        no_matches: "No matches",
        search_placeholder: "Type to search…",
        empty_list: "Nothing here yet",
        no_rows: "No rows",
        loading_rows: "Loading rows…",
        fetching: "fetching…",
        fetch_more: "{} loaded · Enter fetches more",
        preview_sql: "Preview SQL",
        discard: "Discard",
        save_changes: "Save",
        diff_old: "Old",
        diff_new: "New",
        diff_no_file: "No file selected",
        diff_no_changes: "(no textual changes)",
        diff_review: "review",
        reveal_open: "Full text",
        reveal_close: "Collapse",
        hint_choose: "Choose",
        hint_scroll: "Scroll",
        hint_scroll_code: "Scroll code",
        hint_cancel: "Cancel",
        hint_close: "Close",
        hint_next: "Next",
        hint_apply: "Apply",
        hint_edit: "Edit",
        help_title: "Keyboard shortcuts",
        help_footer: "↑↓ Scroll · Esc Close",
        not_in_preview: "Not in this preview",
    };

    /// [`Self::DEFAULT`] with 7-bit punctuation (`...`, `|`, `^v`), to pair
    /// with [`Glyphs::ASCII`].
    pub const ASCII: Self = Self {
        more_cut: "... {} more",
        search_placeholder: "Type to search...",
        loading_rows: "Loading rows...",
        fetching: "fetching...",
        fetch_more: "{} loaded | Enter fetches more",
        help_footer: "^v Scroll | Esc Close",
        ..Self::DEFAULT
    };

    /// `template` with each `{}` replaced by the next of `args`; extra
    /// placeholders stay as written.
    pub fn fill(template: &str, args: &[&dyn std::fmt::Display]) -> String {
        let mut out = String::with_capacity(template.len() + 8);
        let mut rest = template;
        let mut args = args.iter();
        while let Some(i) = rest.find("{}") {
            out.push_str(&rest[..i]);
            match args.next() {
                Some(a) => out.push_str(&a.to_string()),
                None => out.push_str("{}"),
            }
            rest = &rest[i + 2..];
        }
        out.push_str(rest);
        out
    }

    /// Move `text` from `old` to `new` when it still reads `old`: how a
    /// widget's default label follows a change of [`Theme::strings`] while
    /// a label the caller set stays.
    pub fn follow(text: &mut String, old: &str, new: &str) {
        if old != new && text == old {
            new.clone_into(text);
        }
    }
}

impl Default for Strings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Padding inside a container, in cells: `x` on the left and on the right,
/// `y` above and below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Insets {
    pub x: u16,
    pub y: u16,
}

impl Insets {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Spacing tokens: one value per kind of gap, so dialogs, panels, popups
/// and button rows agree instead of each picking its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing {
    /// Cells between buttons in a row. Default 2.
    pub button_gap: u16,
    /// Inside a dialog's border. Default 2 columns, 1 row.
    pub dialog: Insets,
    /// Inside a framed panel's border. Default 2 columns, 0 rows.
    pub panel: Insets,
    /// Inside a popup surface (menus, dropdowns). Default 1 column, 0 rows.
    pub popup: Insets,
    /// Cells between the screen edge and the text of a chrome row (menu
    /// bar, identity strip, segment bars), the same on both sides. Default 2.
    pub chrome_edge: u16,
}

impl Spacing {
    pub const DEFAULT: Self = Self {
        button_gap: 2,
        dialog: Insets::new(2, 1),
        panel: Insets::new(2, 0),
        popup: Insets::new(1, 0),
        chrome_edge: 2,
    };
}

impl Default for Spacing {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// How a footer hint bar lays out one row. Set once on the theme so every
/// screen of an app agrees; a single layer may still override the
/// alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintBarStyle {
    /// Default [`HintAlign::Start`].
    pub align: HintAlign,
    /// A status long enough to crowd out the pinned hints (Esc, help) is
    /// cut to leave them room, but keeps at least this share of the row,
    /// in percent. A status that leaves the pinned hints room is never
    /// cut for them. Default 50.
    pub status_min_share: u8,
    /// When the full hints do not fit, show the normal ones as keys only
    /// (pinned hints keep their action) before dropping any. Off by
    /// default: a bare key without its verb rarely tells the user what it
    /// does, so dropping from the right reads better. Default false.
    pub compact: bool,
    /// Keys whose hints are pinned by default: never dropped while
    /// anything else can go, so the way out and the way to help stay
    /// visible. Default `Esc`, `?`, `F1`.
    pub pinned_keys: &'static [&'static str],
}

impl HintBarStyle {
    pub const DEFAULT: Self = Self {
        align: HintAlign::Start,
        status_min_share: 50,
        compact: false,
        pinned_keys: &["Esc", "?", "F1"],
    };
}

impl Default for HintBarStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Approximate sRGB of a colour as a terminal renders it (xterm defaults
/// for named and indexed colours). `None` for `Reset`.
fn approx_rgb(c: Color) -> Option<(u8, u8, u8)> {
    const BASIC: [(u8, u8, u8); 16] = [
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
    Some(match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) if i < 16 => BASIC[i as usize],
        Color::Indexed(i) if i < 232 => {
            let i = i - 16;
            let v = |n: u8| if n == 0 { 0 } else { 55 + n * 40 };
            (v(i / 36), v((i / 6) % 6), v(i % 6))
        }
        Color::Indexed(i) => {
            let g = 8 + (i - 232) * 10;
            (g, g, g)
        }
        Color::Black => BASIC[0],
        Color::Red => BASIC[1],
        Color::Green => BASIC[2],
        Color::Yellow => BASIC[3],
        Color::Blue => BASIC[4],
        Color::Magenta => BASIC[5],
        Color::Cyan => BASIC[6],
        Color::Gray => BASIC[7],
        Color::DarkGray => BASIC[8],
        Color::LightRed => BASIC[9],
        Color::LightGreen => BASIC[10],
        Color::LightYellow => BASIC[11],
        Color::LightBlue => BASIC[12],
        Color::LightMagenta => BASIC[13],
        Color::LightCyan => BASIC[14],
        Color::White => BASIC[15],
        Color::Reset => return None,
    })
}

/// WCAG relative luminance.
fn rel_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let ch = |v: u8| {
        let v = v as f64 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
}

/// WCAG contrast ratio of two colours as the terminal renders them; `None`
/// when either is the terminal default.
pub fn contrast_ratio(a: Color, b: Color) -> Option<f64> {
    let (x, y) = (rel_luminance(approx_rgb(a)?), rel_luminance(approx_rgb(b)?));
    Some((x.max(y) + 0.05) / (x.min(y) + 0.05))
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
    // a saturated colour keeps its hue: dark greens and reds must not snap
    // to the grey that a neutral plane already uses
    let chroma = r.max(g).max(b) as i32 - r.min(g).min(b) as i32;
    if gray_err < cube_err && chroma <= 30 {
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

    fn luminance(c: Color) -> f64 {
        let Color::Rgb(r, g, b) = c else {
            panic!("truecolor expected, got {c:?}")
        };
        let ch = |v: u8| {
            let v = v as f64 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
    }

    fn contrast(a: Color, b: Color) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn the_modal_backdrop_keeps_reversed_text_readable() {
        let t = Theme::junie();
        // dark ink on a light fill (a pressed or reversed cell) under a modal
        let st = t.backdrop(Style::new().fg(t.canvas).bg(t.text_primary));
        assert_ne!(st.fg, st.bg);
        // canvas ink on the canvas is a blank and stays one
        let st = t.backdrop(Style::new().fg(t.canvas).bg(t.canvas));
        assert_eq!(st.fg, st.bg);
    }

    #[test]
    fn row_text_stays_readable_in_every_state_including_retinted_values() {
        let t = Theme::junie();
        for bg in [t.canvas, t.surface, t.surface_elevated] {
            for bits in 0u8..16 {
                let s = crate::ui::ctx::VisualState {
                    focused: bits & 1 != 0,
                    hovered: bits & 2 != 0,
                    pressed: bits & 4 != 0,
                    selected: bits & 8 != 0,
                    ..Default::default()
                };
                let st = t.row(s, bg);
                let fill = st.bg.unwrap();
                // widgets re-tint values with the normal tone after row()
                for fg in [st.fg.unwrap(), t.text_primary] {
                    let c = contrast(fg, fill);
                    assert!(c >= 4.5, "{s:?} on {bg:?}: {fg:?}/{fill:?} = {c:.2}");
                }
            }
        }
    }

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

    const KINDS: [Kind; 15] = [
        Kind::Row,
        Kind::CursorRow(CursorStyle::Fill),
        Kind::CursorRow(CursorStyle::Bar),
        Kind::Button(ButtonKind::Primary),
        Kind::Button(ButtonKind::Secondary),
        Kind::Button(ButtonKind::Subtle),
        Kind::Button(ButtonKind::Danger),
        Kind::Button(ButtonKind::Toggle),
        Kind::Tab,
        Kind::Title,
        Kind::Chip,
        Kind::Close,
        Kind::Field,
        Kind::Handle,
        Kind::Seam,
    ];

    /// Two resolved controls look different on this level's terminal: in
    /// monochrome only modifiers and the gutter glyph survive.
    fn looks_distinct(t: &Theme, kind: Kind, a: &Interactive, b: &Interactive) -> bool {
        let mods = a.style.add_modifier != b.style.add_modifier || a.gutter != b.gutter;
        if t.level == ColorLevel::Mono {
            return mods;
        }
        let ink = if kind.is_glyph() {
            a.style.fg != b.style.fg
        } else {
            a.style.bg != b.style.bg
        };
        mods || ink
    }

    fn st(f: impl FnOnce(&mut VisualState)) -> VisualState {
        let mut s = VisualState::default();
        f(&mut s);
        s
    }

    #[test]
    fn every_kind_plane_and_level_keeps_rest_hover_and_pressed_apart() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for kind in KINDS {
                for plane in Plane::ALL {
                    if kind.is_glyph() && !glyph_plane(plane) {
                        continue;
                    }
                    // the same ladder walk with and without focus, selection
                    // and (for a fill list) the cursor on the row
                    for base in [
                        VisualState::default(),
                        st(|s| s.focused = true),
                        st(|s| s.selected = true),
                        st(|s| {
                            s.selected = true;
                            s.focused = true
                        }),
                    ] {
                        let r = |f: fn(&mut VisualState)| {
                            let mut s = base;
                            f(&mut s);
                            t.interactive(kind, s, plane)
                        };
                        let rest = r(|_| {});
                        let hover = r(|s| s.hovered = true);
                        let pressed = r(|s| {
                            s.hovered = true;
                            s.pressed = true
                        });
                        let flash = r(|s| s.pressed = true);
                        let ctx = format!("{level:?} {kind:?} on {plane:?} from {base:?}");
                        assert!(
                            looks_distinct(&t, kind, &rest, &hover),
                            "rest = hover: {ctx}"
                        );
                        assert!(
                            looks_distinct(&t, kind, &hover, &pressed),
                            "hover = pressed: {ctx}"
                        );
                        assert!(
                            looks_distinct(&t, kind, &rest, &pressed),
                            "rest = pressed: {ctx}"
                        );
                        assert!(
                            looks_distinct(&t, kind, &rest, &flash),
                            "rest = flash: {ctx}"
                        );

                        // disabled ignores hover and press, and differs from rest
                        let dis = |f: fn(&mut VisualState)| {
                            let mut s = base;
                            s.disabled = true;
                            f(&mut s);
                            t.interactive(kind, s, plane)
                        };
                        let d = dis(|_| {});
                        assert_eq!(d, dis(|s| s.hovered = true), "disabled hover: {ctx}");
                        assert_eq!(
                            d,
                            dis(|s| {
                                s.hovered = true;
                                s.pressed = true
                            }),
                            "disabled press: {ctx}"
                        );

                        for (name, it) in [
                            ("rest", rest),
                            ("hover", hover),
                            ("pressed", pressed),
                            ("disabled", d),
                        ] {
                            let fg = it.style.fg.unwrap();
                            let bg = it.style.bg.unwrap_or_else(|| t.ladder(plane).rest);
                            assert_ne!(fg, bg, "{name} ink = fill: {ctx}");
                            if level == ColorLevel::TrueColor && name != "disabled" {
                                // text ≥ 4.5:1; handles and seams are glyphs (≥ 3:1)
                                // and their rest step is a quiet decoration
                                let min = if kind.is_glyph() { 3.0 } else { 4.5 };
                                if !(kind.is_glyph() && name == "rest") {
                                    let c = contrast(fg, bg);
                                    assert!(c >= min, "{name} {fg:?} on {bg:?} = {c:.2}: {ctx}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Planes scrollbars and splitters live on.
    fn glyph_plane(p: Plane) -> bool {
        matches!(
            p,
            Plane::Canvas | Plane::Surface | Plane::Card | Plane::Field
        )
    }

    #[test]
    fn interactive_states_differ_from_focus_and_disabled_at_every_level() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for kind in KINDS {
                for plane in [Plane::Canvas, Plane::Card, Plane::Popover] {
                    let i = |s: VisualState| t.interactive(kind, s, plane);
                    let focused = i(st(|s| s.focused = true));
                    let rest = i(VisualState::default());
                    let disabled = i(st(|s| s.disabled = true));
                    let ctx = format!("{level:?} {kind:?} on {plane:?}");
                    for (name, other) in [
                        ("hover", i(st(|s| s.hovered = true))),
                        ("pressed", i(st(|s| s.pressed = true))),
                    ] {
                        assert!(
                            looks_distinct(&t, kind, &other, &disabled),
                            "{name} = disabled: {ctx}"
                        );
                        // a fill-list cursor is a plain row until the cursor
                        // is on it; Close and Field carry no focus mark
                        if !matches!(kind, Kind::CursorRow(_) | Kind::Close | Kind::Field) {
                            assert!(
                                looks_distinct(&t, kind, &other, &focused),
                                "{name} = focused: {ctx}"
                            );
                        }
                    }
                    if !matches!(kind, Kind::Close) || level != ColorLevel::Mono {
                        assert!(
                            looks_distinct(&t, kind, &rest, &disabled)
                                || rest.style.fg != disabled.style.fg,
                            "rest = disabled: {ctx}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn reduced_levels_pin_state_tokens_instead_of_snapping_to_greys() {
        let is_grey = |c: Color| matches!(c, Color::Indexed(i) if i >= 232);
        let t = Theme::for_level(ColorLevel::Ansi256);
        // primary button: three greens, the pressed one still green and readable
        let a = t.ladder(Plane::Accent);
        assert_eq!(
            (a.rest, a.hover, a.pressed),
            (Color::Indexed(78), Color::Indexed(71), Color::Indexed(65))
        );
        assert!(contrast_ratio(t.text_on_accent, a.pressed).unwrap() >= 4.5);
        // dark saturated tokens keep their hue
        for c in [
            t.accent_pressed,
            t.highlight_danger,
            t.accent_bg,
            t.error_bg,
        ] {
            assert!(!is_grey(c), "{c:?} snapped to a grey");
        }
        assert!(!is_grey(Color::Indexed(nearest_256(0x0f, 0x2e, 0x13))));
        // a destructive menu cursor is not grey on grey
        assert_ne!(t.highlight_danger, t.popover);
        // the selection tint is not the card
        assert_ne!(t.accent_bg, t.surface_elevated);

        let t = Theme::for_level(ColorLevel::Ansi16);
        // a disabled button label stays visible on its plane
        for kind in [
            ButtonKind::Primary,
            ButtonKind::Secondary,
            ButtonKind::Danger,
        ] {
            let st = t.button(
                kind,
                VisualState {
                    disabled: true,
                    ..Default::default()
                },
                t.canvas,
            );
            assert_ne!(st.fg, st.bg, "{kind:?}");
        }
        // an off chip label is not grey on grey
        let chip = t.interactive(Kind::Chip, VisualState::default(), Plane::Canvas);
        assert_ne!(chip.style.fg, chip.style.bg);
        // a destructive label on a menu stays readable
        let ink = t.legible(t.error_soft, t.popover);
        assert!(contrast_ratio(ink, t.popover).unwrap() >= 4.5);
        // borders are visible on the black planes
        assert_ne!(t.border_subtle, t.canvas);
        assert_ne!(t.border_strong, t.popover);

        let t = Theme::for_level(ColorLevel::Mono);
        assert_ne!(t.border_subtle, t.canvas);
    }

    #[test]
    fn no_color_buttons_and_menu_cursors_carry_state_in_modifiers() {
        let t = Theme::for_level(ColorLevel::Mono);
        let pressed = VisualState {
            hovered: true,
            pressed: true,
            ..Default::default()
        };
        let hovered = VisualState {
            hovered: true,
            ..Default::default()
        };
        for kind in [
            ButtonKind::Secondary,
            ButtonKind::Subtle,
            ButtonKind::Danger,
        ] {
            let p = t.button(kind, pressed, t.canvas);
            assert!(p.add_modifier.contains(Modifier::REVERSED), "{kind:?}");
            let h = t.button(kind, hovered, t.canvas);
            assert!(h.add_modifier.contains(Modifier::UNDERLINED), "{kind:?}");
        }
        // the menu cursor is reversed video, not bold on the same plane
        let cursor = VisualState {
            focused: true,
            ..Default::default()
        };
        let row = t.cursor_row(cursor, t.popover, CursorStyle::Fill);
        assert!(row.style.add_modifier.contains(Modifier::REVERSED));
        let rest = t.cursor_row(VisualState::default(), t.popover, CursorStyle::Fill);
        assert!(!rest.style.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn backdrop_text_never_lands_on_the_backdrop_grey_at_reduced_levels() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for bg in [
                t.canvas,
                t.surface_elevated,
                t.surface_overlay,
                t.popover,
                t.accent,
            ] {
                for fg in [
                    t.text_primary,
                    t.text_secondary,
                    t.text_muted,
                    t.text_faint,
                    t.text_ghost,
                    t.accent,
                ] {
                    if fg == bg {
                        continue;
                    }
                    let st = t.backdrop(Style::new().fg(fg).bg(bg));
                    assert_ne!(st.fg, st.bg, "{level:?}: {fg:?} on {bg:?}");
                }
            }
        }
    }

    #[test]
    fn faint_text_reads_and_stays_apart_from_disabled_and_success_from_focus() {
        let t = Theme::junie();
        for bg in [
            t.canvas,
            t.surface,
            t.surface_elevated,
            t.field,
            t.surface_overlay,
        ] {
            let c = contrast(t.text_faint, bg);
            assert!(c >= 3.0, "faint on {bg:?} = {c:.2}");
            let b = contrast(t.border_subtle, bg);
            assert!(
                b > 1.3 || bg == t.surface_overlay,
                "border on {bg:?} = {b:.2}"
            );
        }
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            assert_ne!(t.text_faint, t.disabled, "{level:?}");
            if level != ColorLevel::Mono {
                assert_ne!(t.success, t.accent, "{level:?}");
            }
        }
    }
}
