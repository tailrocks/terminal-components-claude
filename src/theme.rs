use ratatui::style::{Color, Modifier, Style};

use crate::ui::ctx::VisualState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorStyle {
    #[default]
    Fill,

    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactive {
    pub style: Style,
    pub gutter: &'static str,
    pub gutter_style: Style,

    pub on_fill: bool,
}

pub type CursorRow = Interactive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    Canvas,

    Surface,

    Card,

    Field,

    Raised,

    Popover,

    Highlight,

    HighlightDanger,

    Accent,

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

    pub fn is_fill(self) -> bool {
        matches!(
            self,
            Plane::Highlight | Plane::HighlightDanger | Plane::Accent | Plane::Danger
        )
    }

    pub fn up(self) -> Plane {
        match self {
            Plane::Canvas | Plane::Surface | Plane::Card | Plane::Field => Plane::Raised,
            Plane::Raised | Plane::Popover => Plane::Popover,
            fill => fill,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Row,

    CursorRow(CursorStyle),
    Button(ButtonKind),

    Tab,

    Title,

    Chip,

    Close,

    Field,

    Handle,

    Seam,
}

impl Kind {
    pub fn is_glyph(self) -> bool {
        matches!(self, Kind::Handle | Kind::Seam)
    }
}

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

    pub handle: Ladder,

    pub seam: Ladder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateModifiers {
    pub hover: Modifier,

    pub hover_flat: Modifier,

    pub pressed: Modifier,

    pub fill: Modifier,

    pub pressed_fill: Modifier,

    pub focused: Modifier,

    pub selected: Modifier,

    pub disabled: Modifier,

    pub thumb_hover: Modifier,

    pub thumb_pressed: Modifier,
}

impl StateModifiers {
    pub const COLOR: Self = Self {
        hover: Modifier::empty(),
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::BOLD,
        fill: Modifier::empty(),
        pressed_fill: Modifier::BOLD,
        focused: Modifier::BOLD,
        selected: Modifier::BOLD,
        disabled: Modifier::empty(),
        thumb_hover: Modifier::empty(),
        thumb_pressed: Modifier::empty(),
    };

    pub const ANSI16: Self = Self {
        hover: Modifier::empty(),
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::REVERSED.union(Modifier::BOLD),
        fill: Modifier::empty(),
        pressed_fill: Modifier::REVERSED.union(Modifier::BOLD),
        focused: Modifier::BOLD,
        selected: Modifier::BOLD,
        disabled: Modifier::empty(),
        thumb_hover: Modifier::empty(),
        thumb_pressed: Modifier::empty(),
    };

    pub const MONO: Self = Self {
        hover: Modifier::UNDERLINED,
        hover_flat: Modifier::UNDERLINED,
        pressed: Modifier::REVERSED.union(Modifier::BOLD),
        fill: Modifier::REVERSED,
        pressed_fill: Modifier::BOLD.union(Modifier::UNDERLINED),
        focused: Modifier::BOLD,
        selected: Modifier::BOLD,
        disabled: Modifier::DIM,
        thumb_hover: Modifier::BOLD,
        thumb_pressed: Modifier::REVERSED,
    };
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LevelOverrides {
    pub truecolor: Option<fn(&mut Theme)>,
    pub ansi256: Option<fn(&mut Theme)>,
    pub ansi16: Option<fn(&mut Theme)>,
    pub mono: Option<fn(&mut Theme)>,
}

impl LevelOverrides {
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

pub fn ascii_glyphs(t: &mut Theme) {
    t.glyphs = Glyphs::ASCII;
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    #[default]
    Unicode,

    Ascii,
}

impl Charset {
    pub const OVERRIDE_ENV: &'static str = "JUNIE_TUI_GLYPHS";

    pub const ASCII_TERMS: [&'static str; 5] = ["linux", "vt", "cons25", "dumb", "pcansi"];

    pub const fn glyphs(self) -> Glyphs {
        match self {
            Self::Unicode => Glyphs::UNICODE,
            Self::Ascii => Glyphs::ASCII,
        }
    }

    pub const fn strings(self) -> Strings {
        match self {
            Self::Unicode => Strings::DEFAULT,
            Self::Ascii => Strings::ASCII,
        }
    }

    pub fn apply(self, theme: Theme) -> Theme {
        theme
            .with_glyphs(self.glyphs())
            .with_strings(self.strings())
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "unicode" | "utf8" | "utf-8" => Some(Self::Unicode),
            "ascii" => Some(Self::Ascii),
            _ => None,
        }
    }

    pub fn from_env(get: impl Fn(&str) -> Option<String>, windows: bool) -> Self {
        if let Some(charset) = get(Self::OVERRIDE_ENV).as_deref().and_then(Self::parse) {
            return charset;
        }
        let term = get("TERM").filter(|t| !t.is_empty());
        if let Some(term) = &term {
            let term = term.to_ascii_lowercase();
            let base = term.split('-').next().unwrap_or_default();
            let vt = base
                .strip_prefix("vt")
                .is_some_and(|n| n.starts_with(|c: char| c.is_ascii_digit()));
            if vt || Self::ASCII_TERMS.iter().any(|t| *t != "vt" && *t == base) {
                return Self::Ascii;
            }
        }
        let set = |k: &str| get(k).is_some_and(|v| !v.is_empty());
        if windows
            && term.is_none()
            && !set("WT_SESSION")
            && !set("TERM_PROGRAM")
            && get("ConEmuANSI").is_none_or(|v| v != "ON")
        {
            return Self::Ascii;
        }
        Self::Unicode
    }

    pub fn detect() -> Self {
        Self::from_env(|k| std::env::var(k).ok(), cfg!(windows))
    }
}

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

    pub const HIGHLIGHT: Color = rgb(0x2f5aa8);

    pub const INDIGO: Color = rgb(0x4b3fa8);
    pub const WHITE: Color = rgb(0xffffff);
    pub const WHITE_70: Color = rgb(0xb3b3b3);
    pub const WHITE_50: Color = rgb(0x808080);
    pub const WHITE_30: Color = rgb(0x4d4d4d);
    pub const WHITE_15: Color = rgb(0x262626);

    pub const WHITE_43: Color = rgb(0x737373);

    pub const WHITE_23: Color = rgb(0x3a3a3a);

    pub const WHITE_28: Color = rgb(0x474747);

    pub const SUCCESS: Color = rgb(0x3fb950);
    pub const GREEN: Color = rgb(0x48e054);
    pub const GREEN_80: Color = rgb(0x3ab343);
    pub const GREEN_60: Color = rgb(0x2e9a38);
    pub const GREEN_10: Color = rgb(0x0a1c0c);
    pub const ON_GREEN: Color = rgb(0x19191c);

    pub const RED: Color = rgb(0xf05d5d);

    pub const RED_STRONG: Color = rgb(0xff8a8a);

    pub const SUCCESS_STRONG: Color = rgb(0x56d364);

    pub const MUTED: Color = rgb(0x8a8a8a);
    pub const RED_20: Color = rgb(0x2e0f0f);

    pub const RED_SOFT: Color = rgb(0xe8a0a0);

    pub const RED_45: Color = rgb(0x7a2a2a);
    pub const AMBER: Color = rgb(0xf59e09);

    pub const BLUE_INFO: Color = rgb(0x79b8ff);

    pub const SYNTAX_STRING: Color = rgb(0x8bd5ca);

    pub const SYNTAX_LITERAL: Color = rgb(0xc6a0f6);

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

    pub const RED_FILL: Color = rgb(0xd93636);
    pub const RED_FILL_HOVER: Color = rgb(0xc22e2e);
    pub const RED_FILL_PRESSED: Color = rgb(0x9e2424);

    pub const RED_LABEL: Color = rgb(0xff6b6b);

    pub const GREEN_25: Color = rgb(0x173f1c);
    pub const GREEN_35: Color = rgb(0x1e5024);

    pub const BLUE_20: Color = rgb(0x1f3460);

    pub const GREY_INACTIVE: Color = rgb(0x33363e);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InactiveSelection {
    #[default]
    Accent,

    Neutral,

    None,
}

impl InactiveSelection {
    pub const ALL: [InactiveSelection; 3] = [
        InactiveSelection::Accent,
        InactiveSelection::Neutral,
        InactiveSelection::None,
    ];
}

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

    pub highlight: Color,

    pub highlight_danger: Color,

    pub error_soft: Color,

    pub border_subtle: Color,
    pub border_strong: Color,

    pub scroll_track: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_faint: Color,

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

    pub link: Color,

    pub error_strong: Color,

    pub success_strong: Color,

    pub syntax_keyword: Color,

    pub syntax_string: Color,

    pub syntax_literal: Color,

    pub selection_bg: Color,

    pub highlight_on_selection: Color,

    pub accent_bg_hover: Color,

    pub selection_inactive: Color,

    pub inactive_selection: InactiveSelection,

    pub danger_label: Color,

    pub ladders: Ladders,

    pub mods: StateModifiers,

    pub tint_selection: bool,

    pub hintbar: HintBarStyle,

    pub spacing: Spacing,

    pub glyphs: Glyphs,

    pub strings: Strings,

    pub edit_entry: crate::core::edit::EditEntry,

    pub field_keys: Option<crate::core::edit::FieldKeys>,

    pub card_heading: crate::widgets::panel::CardHeading,
}

impl Theme {
    pub const DEFAULT_EDIT_ENTRY: crate::core::edit::EditEntry =
        crate::core::edit::EditEntry::Immediate;

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
            scroll_track: WHITE_28,
            text_primary: WHITE,
            text_secondary: WHITE_70,
            text_muted: MUTED,
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
            info: BLUE_INFO,
            link: BLUE_INFO,
            error_strong: RED_STRONG,
            success_strong: SUCCESS_STRONG,
            syntax_keyword: BLUE_INFO,
            syntax_string: SYNTAX_STRING,
            syntax_literal: SYNTAX_LITERAL,
            accent_bg_hover: GREEN_35,
            selection_inactive: GREY_INACTIVE,
            inactive_selection: InactiveSelection::Accent,
            selection_bg: BLUE_20,
            highlight_on_selection: INDIGO,
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
                handle: Ladder::new(WHITE_50, WHITE_70, GREEN),
                seam: Ladder::new(WHITE_30, WHITE_50, GREEN),
            },
            mods: StateModifiers::COLOR,
            tint_selection: true,
            hintbar: HintBarStyle::DEFAULT,
            spacing: Spacing::DEFAULT,
            glyphs: Glyphs::UNICODE,
            strings: Strings::DEFAULT,
            edit_entry: Self::DEFAULT_EDIT_ENTRY,
            field_keys: None,
            card_heading: crate::widgets::panel::CardHeading::Rule,
        }
    }

    pub fn for_level(level: ColorLevel) -> Self {
        Self::for_level_with(level, &LevelOverrides::default())
    }

    pub fn centred_hints(mut self) -> Self {
        self.hintbar.align = HintAlign::Center;
        self
    }

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
            scroll_track,
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
            link,
            error_strong,
            success_strong,
            syntax_keyword,
            syntax_string,
            syntax_literal,
            accent_bg_hover,
            selection_inactive,
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
        self.highlight_on_selection = i(55);
        self.border_subtle = i(238);
        self.border_strong = i(242);
        self.scroll_track = i(239);
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
        self.selection_inactive = i(238);
        self.selection_bg = i(17);
        self.accent_bg_subtle = i(22);
        self.focus = i(78);
        self.success = i(71);
        self.success_strong = i(77);
        self.disabled = i(239);

        self.error = i(203);
        self.error_strong = i(210);
        self.warning = i(214);
        self.info = i(111);
        self.link = i(111);
        self.syntax_keyword = i(111);
        self.syntax_string = i(116);
        self.syntax_literal = i(183);
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
            handle: lad(245, 252, 78),
            seam: lad(239, 245, 78),
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

        self.highlight_on_selection = Blue;
        self.border_subtle = DarkGray;
        self.border_strong = Gray;

        self.scroll_track = DarkGray;
        self.text_primary = White;
        self.text_secondary = Gray;
        self.text_muted = Gray;

        self.text_faint = Gray;
        self.text_ghost = DarkGray;
        self.text_on_accent = Black;
        self.accent = LightGreen;
        self.accent_hover = Green;
        self.accent_pressed = Green;
        self.accent_bg = Black;
        self.accent_bg_hover = DarkGray;

        self.selection_bg = Black;
        self.accent_bg_subtle = Black;
        self.focus = LightGreen;
        self.success = Green;
        self.success_strong = Green;
        self.disabled = DarkGray;
        self.error = LightRed;
        self.error_strong = LightRed;
        self.warning = Yellow;
        self.info = LightBlue;
        self.link = LightBlue;
        self.syntax_keyword = LightBlue;
        self.syntax_string = Cyan;
        self.syntax_literal = LightMagenta;
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
            handle: Ladder::new(Gray, White, LightGreen),
            seam: Ladder::new(DarkGray, Gray, LightGreen),
        };
    }

    fn apply_mono(&mut self) {
        use Color::*;

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
        self.highlight_on_selection = DarkGray;
        self.border_subtle = DarkGray;
        self.border_strong = Gray;

        self.scroll_track = DarkGray;
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
        self.success_strong = White;
        self.disabled = DarkGray;
        self.error = White;
        self.error_strong = White;

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

    pub fn with_ladder(mut self, plane: Plane, ladder: Ladder) -> Self {
        *self.ladder_mut(plane) = ladder;
        self
    }

    pub const fn with_glyphs(mut self, glyphs: Glyphs) -> Self {
        self.glyphs = glyphs;
        self
    }

    pub const fn with_strings(mut self, strings: Strings) -> Self {
        self.strings = strings;
        self
    }

    pub const fn with_edit_entry(mut self, entry: crate::core::edit::EditEntry) -> Self {
        self.edit_entry = entry;
        self
    }

    pub const fn with_inactive_selection(mut self, look: InactiveSelection) -> Self {
        self.inactive_selection = look;
        self
    }

    pub const fn with_card_heading(mut self, style: crate::widgets::panel::CardHeading) -> Self {
        self.card_heading = style;
        self
    }

    pub const fn with_field_keys(mut self, keys: crate::core::edit::FieldKeys) -> Self {
        self.field_keys = Some(keys);
        self
    }

    pub fn with_state_modifiers(mut self, mods: StateModifiers) -> Self {
        self.mods = mods;
        self
    }

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

    pub fn backdrop(&self, style: Style) -> Style {
        let bg = match style.bg {
            Some(c) if c == self.canvas || c == self.surface || c == self.surface_elevated => c,
            Some(c) if c == self.field || c == self.field_hover => self.surface_elevated,
            Some(_) => self.surface_overlay,
            None => self.canvas,
        };

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

        let reads = |fg: Color, bg: Color| contrast_ratio(fg, bg).is_some_and(|r| r >= 3.0);
        let plane = style.bg.unwrap_or(self.canvas);
        let blank = match style.fg {
            Some(c) if Some(c) == style.bg => true,
            Some(c) => (c == self.canvas || c == self.surface) && dark_plane && !reads(c, plane),
            None => false,
        };
        let fg = match style.fg {
            _ if blank => bg,
            Some(c) if c == self.canvas || c == self.surface => self.text_muted,

            Some(c) if c == self.accent || c == self.focus => self.text_faint,
            Some(c) if c == self.text_primary || c == self.error || c == self.warning => {
                self.text_muted
            }
            Some(c) if c == self.text_secondary || c == self.text_on_accent => self.text_faint,
            _ => self.text_ghost,
        };

        let fg = match style.fg {
            Some(c) if !blank && reads(c, plane) && !reads(fg, bg) => {
                [self.text_faint, self.text_muted, self.text_secondary]
                    .into_iter()
                    .find(|&ink| reads(ink, bg))
                    .unwrap_or(self.text_primary)
            }
            _ => fg,
        };

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

    pub fn disabled_style(&self) -> Style {
        let style = Style::new().fg(self.disabled);
        if self.level == ColorLevel::Mono {
            style.add_modifier(Modifier::DIM)
        } else {
            style
        }
    }

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

    pub fn interactive(&self, kind: Kind, s: VisualState, plane: Plane) -> Interactive {
        let m = self.mods;

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
                fg = self.text_muted;
                dim |= Modifier::DIM;
            }
            let mut style = Style::new().fg(fg).add_modifier(dim);
            if glyph {
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
            let lad = if kind == Kind::Handle {
                self.ladders.handle
            } else {
                self.ladders.seam
            };
            let mut style = Style::new().fg(lad.step(s.hovered, s.pressed));
            if s.pressed {
                style = style.add_modifier(m.thumb_pressed);
            } else if s.hovered {
                style = style.add_modifier(m.thumb_hover);
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

        let inactive = self.inactive_look(s);
        let sel = row_like && s.selected && self.tint_selection;
        let (rest, hover) = if sel && (s.focused || inactive == InactiveSelection::Accent) {
            (self.accent_bg, self.accent_bg_hover)
        } else if sel && inactive == InactiveSelection::Neutral {
            (self.selection_inactive, self.selection_inactive)
        } else {
            (lad.rest, lad.hover)
        };

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

        if s.selected && row_like {
            mods |= m.selected;
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
            fg
        } else {
            self.legible_mark(self.focus, bg)
        };
        Interactive {
            style,
            gutter: if bar { self.glyphs.focus_bar } else { " " },
            gutter_style: Style::new().fg(gutter_fg).bg(bg),
            on_fill: fill,
        }
    }

    pub fn legible(&self, fg: Color, bg: Color) -> Color {
        let Some(c) = contrast_ratio(fg, bg) else {
            return fg;
        };
        if c >= 4.5 {
            return fg;
        }
        let reads = |ink: Color| contrast_ratio(ink, bg).is_some_and(|r| r >= 4.5);
        let strong = if fg == self.error || fg == self.danger_label {
            Some(self.error_strong)
        } else if fg == self.success {
            Some(self.success_strong)
        } else {
            None
        };
        if let Some(ink) = strong.filter(|&ink| reads(ink)) {
            return ink;
        }
        let ladder = [self.text_faint, self.text_muted, self.text_secondary];
        if let Some(pos) = ladder.iter().position(|&ink| ink == fg)
            && let Some(&ink) = ladder[pos + 1..].iter().find(|&&ink| reads(ink))
        {
            return ink;
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

    pub fn legible_mark(&self, fg: Color, bg: Color) -> Color {
        const MARK: f64 = 3.0;
        let reads = |ink: Color| contrast_ratio(ink, bg).is_none_or(|r| r >= MARK);
        if reads(fg) {
            return fg;
        }
        let strong = if fg == self.error || fg == self.danger_label {
            Some(self.error_strong)
        } else if fg == self.success {
            Some(self.success_strong)
        } else {
            None
        };
        if let Some(ink) = bright(fg).into_iter().chain(strong).find(|&c| reads(c)) {
            return ink;
        }
        if reads(self.text_primary) {
            return self.text_primary;
        }
        [self.text_primary, self.text_on_accent]
            .into_iter()
            .filter_map(|ink| contrast_ratio(ink, bg).map(|c| (ink, c)))
            .fold(
                (fg, 0.0),
                |best, cand| {
                    if cand.1 > best.1 { cand } else { best }
                },
            )
            .0
    }

    pub fn beside_label(&self, ink: Color, bg: Color, label: Color) -> Color {
        let reads = |c: Color| contrast_ratio(c, bg).is_none_or(|r| r >= 4.5);
        if reads(ink) {
            return ink;
        }
        let ladder = [self.text_faint, self.text_muted, self.text_secondary];
        ladder
            .iter()
            .position(|&c| c == ink)
            .and_then(|pos| ladder[pos + 1..].iter().copied().find(|&c| reads(c)))
            .unwrap_or(label)
    }

    pub fn frame_on(&self, bg: Color) -> Color {
        let subtle = self.border_subtle;
        if matches!(self.level, ColorLevel::TrueColor | ColorLevel::Ansi256) && subtle != bg {
            return subtle;
        }
        let reads = |c: Color| c != bg && contrast_ratio(c, bg).is_none_or(|r| r >= 3.0);
        [
            subtle,
            self.border_strong,
            self.text_muted,
            self.text_primary,
        ]
        .into_iter()
        .find(|&c| reads(c))
        .unwrap_or(self.text_primary)
    }

    pub fn row(&self, s: VisualState, bg: Color) -> Style {
        self.interactive(Kind::Row, s, self.plane_of(bg)).style
    }

    pub fn cursor_row(&self, s: VisualState, bg: Color, cursor: CursorStyle) -> CursorRow {
        self.interactive(Kind::CursorRow(cursor), s, self.plane_of(bg))
    }

    pub fn row_ink(&self, row: &CursorRow, ink: Color) -> Color {
        if row.on_fill { self.text_primary } else { ink }
    }

    pub fn chosen(&self, focused: bool) -> Color {
        if focused {
            self.accent
        } else {
            self.text_primary
        }
    }

    pub fn chosen_in(&self, s: VisualState) -> Color {
        if s.focused || self.inactive_look(s) == InactiveSelection::Accent {
            self.accent
        } else {
            self.text_primary
        }
    }

    pub fn inactive_look(&self, s: VisualState) -> InactiveSelection {
        match s.inactive_selection.unwrap_or(self.inactive_selection) {
            InactiveSelection::Accent if s.backdrop => InactiveSelection::Neutral,
            look => look,
        }
    }

    pub fn lift(&self, bg: Color) -> Color {
        let l = self.ladder(self.plane_of(bg));
        if bg == l.hover { l.pressed } else { l.hover }
    }

    pub fn gutter_symbol(&self, s: VisualState) -> &'static str {
        if s.focused && !s.disabled {
            self.glyphs.focus_bar
        } else {
            " "
        }
    }

    pub fn gutter(&self, s: VisualState, bg: Color, on_accent: bool) -> Style {
        let fg = if !s.focused || s.disabled {
            bg
        } else if on_accent {
            self.text_on_accent
        } else {
            self.legible_mark(self.focus, bg)
        };
        Style::new().fg(fg).bg(bg)
    }

    pub fn button(&self, kind: ButtonKind, s: VisualState, bg: Color) -> Style {
        self.interactive(Kind::Button(kind), s, self.plane_of(bg))
            .style
    }

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

    pub fn close_plate(&self, s: VisualState, host: Style, danger: bool) -> Style {
        self.danger_plate(s, self.close_glyph(s, host), danger)
    }

    pub fn danger_plate(&self, s: VisualState, style: Style, danger: bool) -> Style {
        if !danger || s.disabled {
            return style;
        }
        if s.pressed {
            let bg = self.ladders.danger.pressed;
            style.bg(bg).fg(self.legible(self.text_primary, bg))
        } else if s.hovered {
            let bg = style.bg.unwrap_or(self.canvas);
            style.fg(self.legible(self.danger_label, bg))
        } else {
            style
        }
    }

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

    pub fn cursor_in_selection(&self, error: bool, pressed: bool) -> Style {
        let st = self.cell_cursor(error, pressed);
        if error || pressed {
            return st;
        }
        if self.level == ColorLevel::Mono {
            return st
                .remove_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD);
        }
        let bg = self.highlight_on_selection;
        st.bg(bg)
            .fg(self.legible(st.fg.unwrap_or(self.text_primary), bg))
    }

    pub fn rule_on(&self, bg: Color) -> Color {
        let rule = match self.plane_of(bg) {
            Plane::Raised | Plane::Popover | Plane::Field => self.border_strong,
            _ => self.border_subtle,
        };
        if rule == bg { self.text_muted } else { rule }
    }

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

    pub fn selection(&self) -> Style {
        let style = Style::new().fg(self.text_primary).bg(self.selection_bg);
        if matches!(self.level, ColorLevel::Ansi16 | ColorLevel::Mono) {
            style.add_modifier(Modifier::REVERSED)
        } else {
            style
        }
    }

    pub fn scrollbar_track(&self) -> Style {
        Style::new().fg(self.scroll_track)
    }

    pub fn scrollbar_thumb(&self, hovered: bool, pressed: bool) -> Style {
        let s = VisualState {
            hovered,
            pressed,
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
            Tone::Info => self.info,
            Tone::Active => self.accent,
            Tone::Danger => self.danger_label,
        }
    }

    pub fn tone_on(&self, tone: Tone, bg: Color) -> Color {
        self.legible(self.tone(tone), bg)
    }

    pub fn link_style(&self, hovered_or_focused: bool) -> Style {
        let style = Style::new().fg(self.link);
        if hovered_or_focused || self.level == ColorLevel::Mono {
            style.add_modifier(Modifier::UNDERLINED)
        } else {
            style
        }
    }

    pub fn status(&self, kind: StatusKind) -> (&'static str, Style) {
        self.status_at(kind, 0)
    }

    pub fn status_at(&self, kind: StatusKind, tick: u64) -> (&'static str, Style) {
        let g = &self.glyphs;
        let glyph = match kind {
            StatusKind::Ok => g.done,
            StatusKind::Fail => g.fail,
            StatusKind::Warn => g.warning,
            StatusKind::Pending => g.dirty,
            StatusKind::Running => g.spinner_frame(tick),
            StatusKind::Queued => g.dot_off,
            StatusKind::Info => g.info,
            StatusKind::Skipped => g.dash,
        };
        let mut style = Style::new().fg(self.status_color(kind));
        if kind == StatusKind::Fail {
            style = style.add_modifier(Modifier::BOLD);
        }
        (glyph, style)
    }

    pub fn status_label(&self, kind: StatusKind) -> Style {
        Style::new().fg(match kind {
            StatusKind::Running => self.info,
            k => self.status_color(k),
        })
    }

    fn status_color(&self, kind: StatusKind) -> Color {
        self.tone(kind.tone())
    }

    pub fn env(&self, env: EnvKind) -> (&'static str, Style) {
        let style = Style::new().fg(self.tone(env.tone()));
        let style = if env == EnvKind::Production {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        };
        (self.glyphs.env(env), style)
    }

    pub fn syntax(&self, tone: SyntaxTone) -> Style {
        match tone {
            SyntaxTone::Keyword => Style::new()
                .fg(self.syntax_keyword)
                .add_modifier(Modifier::BOLD),
            SyntaxTone::Ident | SyntaxTone::Plain => Style::new().fg(self.text_primary),
            SyntaxTone::Str => Style::new().fg(self.syntax_string),
            SyntaxTone::Number | SyntaxTone::Literal => Style::new()
                .fg(self.syntax_literal)
                .add_modifier(Modifier::ITALIC),
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

    Info,

    Active,

    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Ok,

    Fail,

    Warn,

    Pending,

    Running,

    Queued,

    Info,

    Skipped,
}

impl StatusKind {
    pub const ALL: [StatusKind; 8] = [
        StatusKind::Ok,
        StatusKind::Fail,
        StatusKind::Warn,
        StatusKind::Pending,
        StatusKind::Running,
        StatusKind::Queued,
        StatusKind::Info,
        StatusKind::Skipped,
    ];

    pub const fn tone(self) -> Tone {
        match self {
            StatusKind::Ok => Tone::Success,
            StatusKind::Fail => Tone::Error,
            StatusKind::Warn | StatusKind::Pending => Tone::Warning,
            StatusKind::Running => Tone::Active,
            StatusKind::Queued => Tone::Muted,
            StatusKind::Info => Tone::Info,
            StatusKind::Skipped => Tone::Faint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvKind {
    Production,
    Staging,
    Development,
    Local,
}

impl EnvKind {
    pub const fn tone(self) -> Tone {
        match self {
            EnvKind::Production => Tone::Danger,
            EnvKind::Staging => Tone::Warning,
            EnvKind::Development => Tone::Info,
            EnvKind::Local => Tone::Muted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTone {
    Keyword,
    Ident,
    Number,

    Literal,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HintAlign {
    #[default]
    Start,

    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    pub focus_bar: &'static str,

    pub current: &'static str,

    pub chevron_open: &'static str,

    pub chevron_closed: &'static str,

    pub disclosure_open: &'static str,

    pub disclosure_closed: &'static str,

    pub more_above: &'static str,

    pub more_below: &'static str,

    pub sort_asc: &'static str,

    pub sort_desc: &'static str,

    pub ellipsis: &'static str,

    pub separator: &'static str,

    pub check_on: &'static str,

    pub check_off: &'static str,

    pub check_mixed: &'static str,

    pub tick: &'static str,

    pub check_off_narrow: &'static str,

    pub dash: &'static str,

    pub radio_on: &'static str,

    pub radio_off: &'static str,

    pub dot_on: &'static str,

    pub dot_off: &'static str,

    pub toggle_on: &'static str,

    pub toggle_off: &'static str,

    pub close: &'static str,

    pub overflow_left: &'static str,

    pub overflow_right: &'static str,

    pub more_titles: &'static str,

    pub scroll_track: &'static str,

    pub scroll_thumb: &'static str,

    pub scroll_h_track: &'static str,

    pub scroll_h_thumb: &'static str,

    pub divider: &'static str,

    pub h_divider: &'static str,

    pub spinner: &'static [&'static str],

    pub done: &'static str,

    pub fail: &'static str,

    pub info: &'static str,

    pub env_production: &'static str,

    pub env_staging: &'static str,

    pub env_development: &'static str,

    pub arrow: &'static str,

    pub bar_fill: &'static str,

    pub bar_empty: &'static str,

    pub bar_stale: &'static str,

    pub dirty: &'static str,

    pub removed: &'static str,

    pub warning: &'static str,

    pub paused: &'static str,

    pub none: &'static str,

    pub mask: &'static str,

    pub bullet: &'static str,

    pub filter: &'static str,

    pub key: &'static str,

    pub newline: &'static str,

    pub tab: &'static str,

    pub control: &'static str,

    pub arrows: [&'static str; 4],

    pub shift: &'static str,

    pub border: ratatui::symbols::border::Set<'static>,
}

impl Glyphs {
    pub fn env(&self, env: EnvKind) -> &'static str {
        match env {
            EnvKind::Production => self.env_production,
            EnvKind::Staging => self.env_staging,
            EnvKind::Development => self.env_development,
            EnvKind::Local => "",
        }
    }

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
        scroll_track: "┆",
        scroll_thumb: "┃",
        scroll_h_track: "┄",
        scroll_h_thumb: "━",
        divider: "│",
        h_divider: "─",
        spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        done: "✓",
        fail: "✗",
        info: "i",
        env_production: "◆",
        env_staging: "◇",
        env_development: "·",
        arrow: "→",
        bar_fill: "━",
        bar_empty: "─",
        bar_stale: "╍",
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
        shift: "⇧",
        border: ratatui::symbols::border::ROUNDED,
    };

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

        scroll_track: ":",
        scroll_thumb: "#",
        scroll_h_track: ".",
        scroll_h_thumb: "#",
        divider: "|",
        h_divider: "-",
        spinner: &["|", "/", "-", "\\"],
        done: "+",
        fail: "X",
        info: "i",
        env_production: "#",
        env_staging: "~",
        env_development: ".",
        arrow: ">",
        bar_fill: "=",
        bar_empty: "-",
        bar_stale: "~",
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
        shift: "S",
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

    pub fn spinner_frame(&self, tick: u64) -> &'static str {
        match self.spinner.len() {
            0 => " ",
            n => self.spinner[(tick % n as u64) as usize],
        }
    }

    pub fn key_label<'a>(&self, s: &'a str) -> std::borrow::Cow<'a, str> {
        if self.arrows == Self::UNICODE.arrows
            && self.dash == Self::UNICODE.dash
            && self.shift == Self::UNICODE.shift
        {
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
                '⇧' => out.push_str(self.shift),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strings {
    pub cancel: &'static str,

    pub save: &'static str,

    pub close: &'static str,

    pub type_to_confirm: &'static str,

    pub type_to_confirm_hint: &'static str,

    pub ack_mismatch: &'static str,

    pub grid_read_only: &'static str,

    pub grid_row_deleted: &'static str,

    pub leave_unsaved_title: &'static str,

    pub leave_unsaved_body: &'static str,

    pub leave_discard_body: &'static str,

    pub leave_unsaved_some: &'static str,

    pub leave_unsaved_one: &'static str,

    pub leave_unsaved_many: &'static str,

    pub leave_running_title: &'static str,

    pub leave_running_body: &'static str,

    pub leave_stop: &'static str,

    pub leave_stay: &'static str,

    pub more_cut: &'static str,

    pub more_count: &'static str,

    pub required: &'static str,

    pub optional: &'static str,

    pub next: &'static str,

    pub no_matches: &'static str,

    pub search_placeholder: &'static str,

    pub empty_list: &'static str,

    pub no_rows: &'static str,

    pub loading_rows: &'static str,

    pub fetching: &'static str,

    pub fetch_more: &'static str,

    pub preview_sql: &'static str,

    pub discard: &'static str,

    pub save_changes: &'static str,

    pub diff_old: &'static str,

    pub diff_new: &'static str,

    pub diff_no_file: &'static str,

    pub diff_no_changes: &'static str,

    pub diff_review: &'static str,

    pub reveal_open: &'static str,

    pub reveal_close: &'static str,

    pub hint_choose: &'static str,

    pub hint_scroll: &'static str,

    pub hint_scroll_code: &'static str,

    pub hint_cancel: &'static str,

    pub hint_close: &'static str,

    pub hint_next: &'static str,

    pub hint_apply: &'static str,

    pub hint_edit: &'static str,

    pub help_title: &'static str,

    pub help_footer: &'static str,

    pub not_in_preview: &'static str,

    pub done: &'static str,

    pub tip_close: &'static str,

    pub tip_close_tab: &'static str,

    pub tip_remove: &'static str,

    pub tip_new_tab: &'static str,

    pub tip_overflow: &'static str,
}

impl Strings {
    pub const DEFAULT: Self = Self {
        cancel: "Cancel",
        save: "Save",
        close: "Close",
        type_to_confirm: "Type {} to confirm",
        type_to_confirm_hint: "\"{}\" to confirm",
        ack_mismatch: "Does not match",
        grid_read_only: "{} is read-only",
        grid_row_deleted: "This row is marked for deletion",
        leave_unsaved_title: "Unsaved changes",
        leave_unsaved_body: "Save before leaving? {} would be lost.",
        leave_discard_body: "{} will be discarded.",
        leave_unsaved_some: "Unsaved changes",
        leave_unsaved_one: "1 unsaved change",
        leave_unsaved_many: "{} unsaved changes",
        leave_running_title: "Still running",
        leave_running_body: "{} still running. Leaving stops it.",
        leave_stop: "Stop and leave",
        leave_stay: "Stay",
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
        done: "Done",
        tip_close: "Close",
        tip_close_tab: "Close tab",
        tip_remove: "Remove",
        tip_new_tab: "New tab",
        tip_overflow: "Show hidden",
    };

    pub const ASCII: Self = Self {
        more_cut: "... {} more",
        search_placeholder: "Type to search...",
        loading_rows: "Loading rows...",
        fetching: "fetching...",
        fetch_more: "{} loaded | Enter fetches more",
        help_footer: "^v Scroll | Esc Close",
        ..Self::DEFAULT
    };

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing {
    pub button_gap: u16,

    pub dialog: Insets,

    pub panel: Insets,

    pub popup: Insets,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintBarStyle {
    pub align: HintAlign,

    pub status_min_share: u8,

    pub compact: bool,

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

fn bright(c: Color) -> Option<Color> {
    Some(match c {
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::Gray => Color::White,
        _ => return None,
    })
}

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
    fn an_unfocused_selected_row_keeps_its_inactive_selection() {
        for level in ColorLevel::ALL {
            let base = Theme::for_level(level);
            for plane in [Plane::Canvas, Plane::Surface, Plane::Card] {
                let lad = base.ladder(plane);
                let row = |t: &Theme, s: VisualState| t.interactive(Kind::Row, s, plane).style;
                let sel = VisualState {
                    selected: true,
                    ..Default::default()
                };
                let hover = row(
                    &base,
                    VisualState {
                        hovered: true,
                        ..Default::default()
                    },
                );
                let focused = row(
                    &base,
                    VisualState {
                        focused: true,
                        ..sel
                    },
                );
                for look in InactiveSelection::ALL {
                    let t = base.with_inactive_selection(look);
                    let st = row(&t, sel);
                    let bg = st.bg.unwrap();
                    let want = match (look, base.tint_selection) {
                        (_, false) | (InactiveSelection::None, _) => lad.rest,
                        (InactiveSelection::Accent, _) => base.accent_bg,
                        (InactiveSelection::Neutral, _) => base.selection_inactive,
                    };
                    assert_eq!(bg, want, "{level:?} {plane:?} {look:?}");

                    assert!(
                        st.add_modifier.contains(base.mods.selected)
                            && !base.mods.selected.is_empty()
                            || bg != lad.rest,
                        "{level:?} {plane:?} {look:?}: no cue"
                    );
                    assert_ne!(st, hover, "{level:?} {plane:?} {look:?}: like hover");
                    if look == InactiveSelection::Neutral && base.tint_selection {
                        assert_ne!(bg, lad.hover, "{level:?} {plane:?}: neutral is hover");
                        assert_ne!(st, focused, "{level:?} {plane:?}: neutral is focused");
                    }
                    if let (Some(fg), Some(r)) =
                        (st.fg, st.fg.and_then(|fg| contrast_ratio(fg, bg)))
                    {
                        assert!(
                            r >= 4.5,
                            "{level:?} {plane:?} {look:?}: {fg:?} on {bg:?} {r:.2}"
                        );
                    }

                    let under = row(
                        &t,
                        VisualState {
                            backdrop: true,
                            ..sel
                        },
                    );
                    let neutral = row(
                        &base.with_inactive_selection(InactiveSelection::Neutral),
                        sel,
                    );
                    if look != InactiveSelection::None {
                        assert_eq!(under, neutral, "{level:?} {plane:?} {look:?} under a layer");
                    }

                    let own = row(
                        &t,
                        VisualState {
                            inactive_selection: Some(InactiveSelection::None),
                            ..sel
                        },
                    );
                    assert_eq!(own.bg, Some(lad.rest));

                    let ink = t.chosen_in(sel);
                    if look == InactiveSelection::Accent {
                        assert_eq!(ink, t.accent);
                        if let Some(r) = contrast_ratio(ink, lad.rest) {
                            assert!(r >= 3.0, "{level:?} {plane:?}: mark {r:.2}");
                        }
                    } else {
                        assert_eq!(ink, t.text_primary);
                    }
                    assert_eq!(
                        t.chosen_in(VisualState {
                            backdrop: true,
                            ..sel
                        }),
                        t.text_primary,
                        "{level:?}: no accent under a layer"
                    );
                }
            }
        }
    }

    #[test]
    fn the_modal_backdrop_keeps_reversed_text_readable() {
        let t = Theme::junie();

        let st = t.backdrop(Style::new().fg(t.canvas).bg(t.text_primary));
        assert_ne!(st.fg, st.bg);

        let st = t.backdrop(Style::new().fg(t.canvas).bg(t.canvas));
        assert_eq!(st.fg, st.bg);
    }

    #[test]
    fn the_modal_backdrop_dims_text_without_erasing_it() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let planes = [
                t.canvas,
                t.surface,
                t.surface_elevated,
                t.field,
                t.field_hover,
                t.surface_overlay,
                t.popover,
            ];
            for bg in planes {
                for fg in [
                    t.canvas,
                    t.text_ghost,
                    t.text_faint,
                    t.text_muted,
                    t.border_strong,
                ] {
                    if contrast_ratio(fg, bg).is_none_or(|r| r < 3.0) {
                        continue;
                    }
                    let st = t.backdrop(Style::new().fg(fg).bg(bg));
                    let (ink, plane) = (st.fg.unwrap(), st.bg.unwrap());
                    let r = contrast_ratio(ink, plane).unwrap();
                    assert!(
                        r >= 3.0,
                        "{level:?}: {fg:?} on {bg:?} dims to {ink:?} on {plane:?} ({r:.2})"
                    );
                }
            }

            let st = t.backdrop(Style::new().fg(t.text_ghost).bg(t.canvas));
            if level == ColorLevel::TrueColor {
                assert_eq!(st.fg, Some(t.text_ghost));
            }
        }
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

                for fg in [st.fg.unwrap(), t.text_primary] {
                    let c = contrast(fg, fill);
                    assert!(c >= 4.5, "{s:?} on {bg:?}: {fg:?}/{fill:?} = {c:.2}");
                }
            }
        }
    }

    use super::*;

    #[test]
    fn charset_follows_the_terminal() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(key, _)| *key == k)
                    .map(|(_, v)| (*v).to_string())
            }
        };
        for term in [
            "linux",
            "linux-16color",
            "LINUX",
            "vt100",
            "vt102",
            "vt220",
            "vt52",
            "vt100-am",
            "cons25",
            "dumb",
        ] {
            let get = move |k: &str| (k == "TERM").then(|| term.to_string());
            assert_eq!(Charset::from_env(get, false), Charset::Ascii, "{term}");
            assert_eq!(Charset::from_env(get, true), Charset::Ascii, "{term}");
        }
        for term in [
            "xterm-256color",
            "xterm-ghostty",
            "xterm-kitty",
            "screen-256color",
            "tmux-256color",
            "alacritty",
            "vte",
            "linuxish",
        ] {
            let get = move |k: &str| (k == "TERM").then(|| term.to_string());
            assert_eq!(Charset::from_env(get, false), Charset::Unicode, "{term}");
        }

        assert_eq!(Charset::from_env(env(&[]), false), Charset::Unicode);
        assert_eq!(Charset::from_env(env(&[]), true), Charset::Ascii);
        for modern in [
            &[("WT_SESSION", "1b2c")][..],
            &[("TERM_PROGRAM", "vscode")],
            &[("ConEmuANSI", "ON")],
            &[("TERM", "xterm-256color")],
        ] {
            let get = move |k: &str| {
                modern
                    .iter()
                    .find(|(key, _)| *key == k)
                    .map(|(_, v)| (*v).to_string())
            };
            assert_eq!(Charset::from_env(get, true), Charset::Unicode, "{modern:?}");
        }

        let forced = env(&[("TERM", "linux"), ("JUNIE_TUI_GLYPHS", "Unicode")]);
        assert_eq!(Charset::from_env(forced, false), Charset::Unicode);
        let forced = env(&[("TERM", "xterm-256color"), ("JUNIE_TUI_GLYPHS", "ascii")]);
        assert_eq!(Charset::from_env(forced, false), Charset::Ascii);
        let junk = env(&[("TERM", "linux"), ("JUNIE_TUI_GLYPHS", "fancy")]);
        assert_eq!(Charset::from_env(junk, false), Charset::Ascii);

        let t = Charset::Ascii.apply(Theme::junie());
        assert_eq!((t.glyphs, t.strings), (Glyphs::ASCII, Strings::ASCII));
        let t = Charset::Unicode.apply(t);
        assert_eq!((t.glyphs, t.strings), (Glyphs::UNICODE, Strings::DEFAULT));
    }

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

        let a = t.ladder(Plane::Accent);
        assert_eq!(
            (a.rest, a.hover, a.pressed),
            (Color::Indexed(78), Color::Indexed(71), Color::Indexed(65))
        );
        assert!(contrast_ratio(t.text_on_accent, a.pressed).unwrap() >= 4.5);

        for c in [
            t.accent_pressed,
            t.highlight_danger,
            t.accent_bg,
            t.error_bg,
        ] {
            assert!(!is_grey(c), "{c:?} snapped to a grey");
        }
        assert!(!is_grey(Color::Indexed(nearest_256(0x0f, 0x2e, 0x13))));

        assert_ne!(t.highlight_danger, t.popover);

        assert_ne!(t.accent_bg, t.surface_elevated);

        let t = Theme::for_level(ColorLevel::Ansi16);

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

        let chip = t.interactive(Kind::Chip, VisualState::default(), Plane::Canvas);
        assert_ne!(chip.style.fg, chip.style.bg);

        let ink = t.legible(t.error_soft, t.popover);
        assert!(contrast_ratio(ink, t.popover).unwrap() >= 4.5);

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

    #[test]
    fn the_thumb_ladder_is_neutral_lighter_then_accent_at_every_level() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let h = t.ladders.handle;
            let thumb = |hovered, pressed| t.scrollbar_thumb(hovered, pressed);
            let (rest, hover, pressed) =
                (thumb(false, false), thumb(true, false), thumb(true, true));
            if level == ColorLevel::Mono {
                assert_eq!(rest.add_modifier, Modifier::empty());
                assert_eq!(hover.add_modifier, Modifier::BOLD);
                assert_eq!(pressed.add_modifier, Modifier::REVERSED);
                continue;
            }
            assert!(
                h.rest != h.hover && h.hover != h.pressed && h.rest != h.pressed,
                "{level:?}: {h:?}"
            );
            assert_eq!(h.pressed, t.accent, "{level:?}: pressed is the accent");
            assert_ne!(
                h.rest, t.border_subtle,
                "{level:?}: the thumb stands off the track"
            );
            assert_ne!(h.hover, t.accent, "{level:?}");
            assert_eq!(
                (rest.fg, hover.fg, pressed.fg),
                (Some(h.rest), Some(h.hover), Some(h.pressed))
            );
            for st in [rest, hover, pressed] {
                assert_eq!(
                    st.add_modifier,
                    Modifier::empty(),
                    "{level:?}: colour carries it"
                );
            }

            let focused = t.interactive(
                Kind::Handle,
                VisualState {
                    focused: true,
                    ..Default::default()
                },
                Plane::Canvas,
            );
            assert_eq!(focused.style, rest, "{level:?}");
        }

        let t = Theme::junie();
        assert!(luminance(t.ladders.handle.hover) > luminance(t.ladders.handle.rest));
    }

    fn text_planes(t: &Theme) -> [(&'static str, Color); 5] {
        [
            ("canvas", t.canvas),
            ("surface", t.surface),
            ("card", t.surface_elevated),
            ("field", t.field),
            ("raised", t.surface_overlay),
        ]
    }

    #[test]
    fn semantic_hues_read_on_every_plane_they_can_land_on() {
        let t = Theme::junie();
        let tokens = [
            ("info", t.info),
            ("link", t.link),
            ("error", t.error),
            ("warning", t.warning),
            ("success", t.success),
            ("danger_label", t.danger_label),
            ("syntax_keyword", t.syntax_keyword),
            ("syntax_string", t.syntax_string),
            ("syntax_literal", t.syntax_literal),
            ("accent", t.accent),
        ];
        for (name, fg) in tokens {
            for (plane, bg) in text_planes(&t) {
                let c = contrast(fg, bg);
                assert!(c >= 4.5, "{name} on {plane} = {c:.2}");
            }
        }

        for (plane, bg) in &text_planes(&t)[..4] {
            let c = contrast(t.text_muted, *bg);
            assert!(c >= 4.5, "text_muted on {plane} = {c:.2}");
        }

        for tone in [
            Tone::Error,
            Tone::Success,
            Tone::Warning,
            Tone::Info,
            Tone::Danger,
            Tone::Muted,
            Tone::Faint,
            Tone::Secondary,
        ] {
            for bg in [t.popover, t.surface_overlay, t.field_hover] {
                let ink = t.tone_on(tone, bg);
                let c = contrast(ink, bg);
                assert!(c >= 4.5, "{tone:?} on {bg:?} = {c:.2} ({ink:?})");
            }
        }
        assert_eq!(t.tone_on(Tone::Error, t.popover), t.error_strong);
        assert_eq!(t.tone_on(Tone::Success, t.popover), t.success_strong);
        assert_eq!(t.tone_on(Tone::Error, t.surface_elevated), t.error);
    }

    #[test]
    fn frames_focus_bars_and_tab_marks_read_on_their_plane_at_every_level() {
        let states = [
            VisualState {
                selected: true,
                ..Default::default()
            },
            VisualState {
                selected: true,
                hovered: true,
                ..Default::default()
            },
            VisualState {
                selected: true,
                hovered: true,
                pressed: true,
                ..Default::default()
            },
        ];
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let ratio = |a, b| contrast_ratio(a, b).unwrap();

            let frame = t.frame_on(t.popover);
            assert_ne!(frame, t.popover, "{level:?}: frame vanishes");
            let rule = t.rule_on(t.popover);
            assert_ne!(rule, t.popover, "{level:?}: separator vanishes");

            if matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono) {
                for (name, ink) in [("frame", frame), ("separator", rule)] {
                    let r = ratio(ink, t.popover);
                    assert!(r >= 3.0, "{level:?}: popover {name} {r:.2}");
                }
            }

            for s in states {
                for strip in [Plane::Canvas, Plane::Surface, Plane::Card] {
                    let look = t.interactive(Kind::Tab, s, strip).style;
                    let (fill, label) = (look.bg.unwrap(), look.fg.unwrap());
                    let at = format!("{level:?} {strip:?} {s:?}");
                    let text = [
                        ("label", label),
                        ("prefix", t.beside_label(t.text_muted, fill, label)),
                        ("error", t.tone_on(Tone::Error, fill)),
                    ];
                    for (name, ink) in text {
                        let r = ratio(ink, fill);
                        assert!(r >= 4.5, "{at}: {name} {r:.2}");
                    }
                    let marks = [
                        ("spinner", t.legible_mark(t.accent, fill)),
                        ("dirty", t.legible_mark(t.warning, fill)),
                        ("suffix", t.legible_mark(t.text_secondary, fill)),
                        ("done", t.tone_on(Tone::Success, fill)),
                    ];
                    for (name, ink) in marks {
                        let r = ratio(ink, fill);
                        assert!(r >= 3.0, "{at}: {name} {r:.2}");
                    }
                }
            }
        }

        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let kinds = [
                Kind::Row,
                Kind::CursorRow(CursorStyle::Bar),
                Kind::Button(ButtonKind::Secondary),
                Kind::Button(ButtonKind::Subtle),
                Kind::Button(ButtonKind::Toggle),
                Kind::Tab,
                Kind::Title,
                Kind::Chip,
            ];
            for plane in Plane::ALL.into_iter().filter(|p| !p.is_fill()) {
                for kind in kinds {
                    for (hovered, pressed, selected) in [
                        (false, false, false),
                        (true, false, false),
                        (true, true, false),
                        (false, false, true),
                    ] {
                        let s = VisualState {
                            focused: true,
                            hovered,
                            pressed,
                            selected,
                            ..Default::default()
                        };
                        let look = t.interactive(kind, s, plane);
                        if look.gutter != t.glyphs.focus_bar {
                            continue;
                        }
                        let (fg, bg) = (look.gutter_style.fg, look.gutter_style.bg);
                        let r = contrast_ratio(fg.unwrap(), bg.unwrap()).unwrap();
                        assert!(
                            r >= 3.0,
                            "{level:?} {kind:?} on {plane:?} {s:?}: bar {r:.2}"
                        );
                    }
                }
                let bg = t.ladder(plane).rest;
                let g = t.gutter(
                    VisualState {
                        focused: true,
                        ..Default::default()
                    },
                    bg,
                    false,
                );
                let r = contrast_ratio(g.fg.unwrap(), bg).unwrap();
                assert!(r >= 3.0, "{level:?} gutter on {plane:?}: {r:.2}");
            }
        }

        let t = Theme::for_level(ColorLevel::Ansi16);
        assert_eq!(t.legible_mark(t.warning, t.popover), Color::LightYellow);
        assert_eq!(t.legible_mark(t.accent, t.canvas), t.accent);
        let t = Theme::junie();
        assert_eq!(t.legible_mark(t.accent, t.surface_overlay), t.accent);
        assert_eq!(t.frame_on(t.popover), t.border_subtle);
    }

    #[test]
    fn semantic_hues_read_at_256_colours() {
        let t = Theme::for_level(ColorLevel::Ansi256);
        let pins = [
            (t.info, 111),
            (t.link, 111),
            (t.warning, 214),
            (t.success, 71),
            (t.error, 203),
            (t.text_muted, 245),
            (t.syntax_keyword, 111),
            (t.syntax_string, 116),
            (t.syntax_literal, 183),
        ];
        for (c, i) in pins {
            assert_eq!(c, Color::Indexed(i));
        }
        for fg in [
            t.info,
            t.error,
            t.warning,
            t.success,
            t.syntax_string,
            t.syntax_literal,
        ] {
            for bg in [
                t.canvas,
                t.surface,
                t.surface_elevated,
                t.field,
                t.surface_overlay,
            ] {
                let c = contrast_ratio(fg, bg).unwrap();
                assert!(c >= 4.5, "{fg:?} on {bg:?} = {c:.2}");
            }
        }
        for tone in [Tone::Error, Tone::Success, Tone::Muted] {
            let ink = t.tone_on(tone, t.popover);
            let c = contrast_ratio(ink, t.popover).unwrap();
            assert!(c >= 4.5, "{tone:?} on the popover = {c:.2}");
        }
    }

    #[test]
    fn semantic_hues_stay_apart_at_reduced_levels() {
        for level in [ColorLevel::Ansi256, ColorLevel::Ansi16] {
            let t = Theme::for_level(level);
            let hues = [
                ("info", t.info),
                ("warning", t.warning),
                ("success", t.success),
                ("error", t.error),
                ("accent", t.accent),
            ];
            for (i, (a, x)) in hues.iter().enumerate() {
                for (b, y) in &hues[i + 1..] {
                    assert_ne!(x, y, "{level:?}: {a} and {b} collide");
                }
            }
            let syntax = [t.syntax_keyword, t.syntax_string, t.syntax_literal];
            for (i, a) in syntax.iter().enumerate() {
                assert!(
                    !syntax[i + 1..].contains(a),
                    "{level:?}: syntax hues collide"
                );
                assert_ne!(*a, t.text_primary, "{level:?}");
            }

            let sel = t.selection();
            assert!(
                sel.bg != Some(t.highlight) || sel.add_modifier.contains(Modifier::REVERSED),
                "{level:?}"
            );

            let row = t.cursor_row(
                VisualState {
                    focused: true,
                    ..Default::default()
                },
                t.popover,
                CursorStyle::Fill,
            );
            assert_eq!(t.row_ink(&row, t.info), t.text_primary, "{level:?}");
        }
        let t = Theme::for_level(ColorLevel::Ansi16);
        assert_eq!(t.info, Color::LightBlue);
        assert_ne!(t.selection_bg, t.highlight);
        assert!(t.selection().add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn statuses_carry_a_glyph_at_every_level_and_glyph_set() {
        for level in ColorLevel::ALL {
            for glyphs in [Glyphs::UNICODE, Glyphs::ASCII] {
                let t = Theme::for_level(level).with_glyphs(glyphs);
                let mut seen = vec![];
                for kind in StatusKind::ALL {
                    let (g, st) = t.status(kind);
                    assert_eq!(crate::ui::text::width(g), 1, "{kind:?}: one cell");
                    assert!(st.fg.is_some(), "{kind:?}");
                    if kind != StatusKind::Running {
                        assert!(!seen.contains(&g), "{level:?} {kind:?}: {g} reused");
                        seen.push(g);
                    }
                    if glyphs == Glyphs::ASCII {
                        assert!(g.is_ascii(), "{kind:?}: {g}");
                    }
                }

                assert_eq!(
                    t.status_at(StatusKind::Running, 1).0,
                    t.glyphs.spinner_frame(1)
                );

                for kind in StatusKind::ALL {
                    assert_eq!(t.status(kind).1.fg, Some(t.tone(kind.tone())), "{kind:?}");
                }
                for env in [
                    EnvKind::Production,
                    EnvKind::Staging,
                    EnvKind::Development,
                    EnvKind::Local,
                ] {
                    assert_eq!(t.env(env).1.fg, Some(t.tone(env.tone())), "{env:?}");
                }

                let (g, st) = t.env(EnvKind::Production);
                assert!(st.add_modifier.contains(Modifier::BOLD));
                assert_eq!(crate::ui::text::width(g), 1);
                let envs: Vec<_> = [EnvKind::Production, EnvKind::Staging, EnvKind::Development]
                    .map(|e| t.env(e).0)
                    .to_vec();
                assert!(envs[0] != envs[1] && envs[1] != envs[2] && envs[0] != envs[2]);
            }
        }
        let t = Theme::junie();
        assert_eq!(t.status(StatusKind::Ok), ("✓", Style::new().fg(t.success)));
        assert_eq!(t.status_label(StatusKind::Running).fg, Some(t.info));
        assert_eq!(t.status(StatusKind::Running).1.fg, Some(t.accent));

        assert!(
            !t.link_style(false)
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(
            t.link_style(true)
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        let mono = Theme::for_level(ColorLevel::Mono);
        assert!(
            mono.link_style(false)
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
    }

    #[test]
    fn syntax_classes_differ_without_colour() {
        let t = Theme::for_level(ColorLevel::Mono);
        let kw = t.syntax(SyntaxTone::Keyword);
        let lit = t.syntax(SyntaxTone::Literal);
        let id = t.syntax(SyntaxTone::Ident);
        assert!(kw.add_modifier.contains(Modifier::BOLD));
        assert!(lit.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(id.add_modifier, Modifier::empty());
        assert_eq!(t.syntax(SyntaxTone::Number), lit);

        let t = Theme::junie();
        assert_eq!(t.syntax(SyntaxTone::Keyword).fg, Some(t.syntax_keyword));
        assert_eq!(t.syntax(SyntaxTone::Str).fg, Some(t.syntax_string));
        assert_eq!(t.syntax(SyntaxTone::Literal).fg, Some(t.syntax_literal));
        assert_eq!(t.syntax(SyntaxTone::Ident).fg, Some(t.text_primary));
    }

    #[test]
    fn the_cursor_inside_a_selection_reads_and_stands_apart() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let c = t.cursor_in_selection(false, false);
            let sel = t.selection();
            assert_ne!(
                (c.bg, c.add_modifier),
                (sel.bg, sel.add_modifier),
                "{level:?}: the cursor looks like the range"
            );
            if level == ColorLevel::Mono {
                continue;
            }
            let bg = c.bg.unwrap();
            let r = contrast_ratio(c.fg.unwrap(), bg).unwrap();
            assert!(r >= 4.5, "{level:?}: cursor text {r:.2}");
            if level != ColorLevel::Ansi16 {
                assert_ne!(bg, t.selection_bg, "{level:?}");
                assert_ne!(bg, t.highlight, "{level:?}");
            }
        }
    }

    #[test]
    fn legible_steps_up_the_text_ladder_instead_of_jumping_to_primary() {
        let t = Theme::junie();

        assert_eq!(t.legible(t.text_faint, t.canvas), t.text_muted);

        assert_eq!(t.legible(t.text_muted, t.popover), t.text_secondary);

        assert_eq!(t.legible(t.text_faint, t.canvas), t.text_muted);
        assert_eq!(t.legible(t.text_secondary, t.canvas), t.text_secondary);

        assert_eq!(t.legible(t.text_muted, t.accent), t.text_on_accent);
    }
}
