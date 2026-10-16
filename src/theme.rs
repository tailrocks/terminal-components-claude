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
    fn get(&self, level: ColorLevel) -> Option<fn(&mut Theme)> {
        match level {
            ColorLevel::TrueColor => self.truecolor,
            ColorLevel::Ansi256 => self.ansi256,
            ColorLevel::Ansi16 => self.ansi16,
            ColorLevel::Mono => self.mono,
        }
    }
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
    pub const GREEN: Color = rgb(0x48e054);
    pub const GREEN_80: Color = rgb(0x3ab343);
    pub const GREEN_60: Color = rgb(0x2e9a38);
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
            accent_bg: GREEN_25,
            accent_bg_subtle: GREEN_10,
            focus: GREEN,
            disabled: WHITE_30,
            error: RED,
            error_bg: RED_20,
            warning: AMBER,
            success: GREEN,
            info: PURPLE,
            accent_bg_hover: GREEN_35,
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
            danger_label
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

    /// Replace one plane's ladder.
    pub fn with_ladder(mut self, plane: Plane, ladder: Ladder) -> Self {
        *self.ladder_mut(plane) = ladder;
        self
    }

    /// Replace the state modifiers.
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
        let fg = match style.fg {
            // Preserve cells with no visible foreground when dimming a plane.
            Some(c) if Some(c) == style.bg => bg,
            Some(c) if (c == self.canvas || c == self.surface) && dark_plane => bg,
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
            let mut bg = rest;
            let mut fg = self.disabled;
            if fg == bg && !glyph {
                // a grey plane at 16 colours: drop to the canvas
                bg = self.canvas;
            }
            if fg == bg {
                fg = self.text_muted;
            }
            let mut style = Style::new().fg(fg).add_modifier(m.disabled);
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
            Kind::Tab | Kind::Title => s.selected || s.focused,
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
            gutter: if bar { "▎" } else { " " },
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
        if s.focused && !s.disabled { "▎" } else { " " }
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

    /// Text selection remains visible when `NO_COLOR` suppresses colours.
    pub fn selection(&self) -> Style {
        let style = Style::new().fg(self.text_primary).bg(self.popover);
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
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
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
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
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
}
