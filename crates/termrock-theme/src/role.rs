#![doc = r" Colour roles and surfaces (`COMPONENT_ARCHITECTURE.md` §10, §11.2)."]
#![doc = r""]
#![doc = r" A [`StylePatch`](super::StylePatch) names a [`Role`], never a colour;"]
#![doc = r" roles bind to colours at the end of resolution against the live theme,"]
#![doc = r" the current [`Surface`] and the colour capability."]

use ratatui_core::style::Color;

#[doc = r" Number of levels in the surface ladder."]
pub const SURFACE_LEVELS: usize = 5;
#[doc = r" Number of steps in the foreground ladder."]
pub const FG_STEPS: usize = 5;

#[doc = r" The background plane a component sits on. The first five form the"]
#[doc = r" ordered ladder; `Field` and `FieldHover` are the two non-ladder planes."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Surface {
    #[doc = r" The page."]
    #[default]
    Canvas,
    #[doc = r" Chrome and panels."]
    Surface,
    #[doc = r" Cards."]
    Elevated,
    #[doc = r" Overlays."]
    Overlay,
    #[doc = r" Popovers and menus."]
    Popover,
    #[doc = r" A text field."]
    Field,
    #[doc = r" A hovered text field."]
    FieldHover,
}

impl Surface {
    #[doc = r" The ladder index, or `None` for the field planes."]
    pub const fn level(self) -> Option<usize> {
        match self {
            Surface::Canvas => Some(0),
            Surface::Surface => Some(1),
            Surface::Elevated => Some(2),
            Surface::Overlay => Some(3),
            Surface::Popover => Some(4),
            Surface::Field | Surface::FieldHover => None,
        }
    }

    #[doc = r" The ladder surface at index `i`, saturating at the last level."]
    pub const fn from_level(i: usize) -> Surface {
        match i {
            0 => Surface::Canvas,
            1 => Surface::Surface,
            2 => Surface::Elevated,
            3 => Surface::Overlay,
            _ => Surface::Popover,
        }
    }
}

#[doc = r" A step on the foreground ladder."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FgStep {
    #[doc = r" Primary text."]
    Primary,
    #[doc = r" Secondary text."]
    Secondary,
    #[doc = r" Muted text."]
    Muted,
    #[doc = r" Faint text."]
    Faint,
    #[doc = r" Ghost text (dimmed backdrops)."]
    Ghost,
}

impl FgStep {
    #[doc = r" The ladder index."]
    pub const fn index(self) -> usize {
        match self {
            FgStep::Primary => 0,
            FgStep::Secondary => 1,
            FgStep::Muted => 2,
            FgStep::Faint => 3,
            FgStep::Ghost => 4,
        }
    }
}

#[doc = r" A syntax colour role."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SyntaxRole {
    #[doc = r" Keyword."]
    Keyword,
    #[doc = r" Identifier."]
    Ident,
    #[doc = r" String literal."]
    Str,
    #[doc = r" Number literal."]
    Number,
    #[doc = r" Operator."]
    Operator,
    #[doc = r" Punctuation."]
    Punct,
    #[doc = r" Comment."]
    Comment,
    #[doc = r" Plain text."]
    Plain,
    #[doc = r" Type name."]
    TypeName,
    #[doc = r" Function name."]
    Function,
    #[doc = r" Constant."]
    Constant,
    #[doc = r" Invalid token."]
    Invalid,
    #[doc = r" Deprecated token."]
    Deprecated,
    #[doc = r" Find-match background."]
    MatchBg,
    #[doc = r" Current find-match background."]
    MatchCurrentBg,
    #[doc = r" Matching bracket."]
    BracketMatch,
    #[doc = r" Error diagnostic."]
    DiagError,
    #[doc = r" Warning diagnostic."]
    DiagWarning,
    #[doc = r" Info diagnostic."]
    DiagInfo,
}

#[doc = r" A meter colour role."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MeterRole {
    #[doc = r" Healthy."]
    Low,
    #[doc = r" Needs attention."]
    Medium,
    #[doc = r" Critical."]
    High,
    #[doc = r" The track."]
    Track,
    #[doc = r" The unfilled remainder."]
    FillRest,
    #[doc = r" Stale data."]
    Stale,
    #[doc = r" Unknown value."]
    Unknown,
    #[doc = r" Series `n` (wraps over six)."]
    Series(u8),
}

#[doc = r" The complete set a `StylePatch` may name."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    #[doc = r" The background of the current surface."]
    CurrentSurface,
    #[doc = r" The background one ladder step above the current surface."]
    RaisedSurface,
    #[doc = r" Junie row hover plane: `Canvas → Elevated`, `Surface/Elevated → Overlay`,"]
    #[doc = r" `Field → FieldHover`, and `Overlay/Popover/FieldHover → Popover`."]
    #[doc = r" Unlike `RaisedSurface`, this expresses interaction rather than ladder depth."]
    HoverSurface,
    #[doc = r" A specific surface's background."]
    Surface(Surface),
    #[doc = r" A foreground ladder step."]
    Fg(FgStep),
    #[doc = r" Text on an accent fill."]
    OnAccent,
    #[doc = r" Text on a danger fill."]
    OnDanger,
    #[doc = r" Text on an inverted (fg-coloured) fill."]
    OnSurfaceInverse,
    #[doc = r" Subtle border."]
    BorderSubtle,
    #[doc = r" Strong border."]
    BorderStrong,
    #[doc = r" The accent."]
    Accent,
    #[doc = r" Accent, hovered."]
    AccentHover,
    #[doc = r" Accent, pressed."]
    AccentPressed,
    #[doc = r" Accent tint (selection background)."]
    AccentTint,
    #[doc = r" Focus indicator."]
    Focus,
    #[doc = r" Focus ring."]
    FocusRing,
    #[doc = r" Text selection background."]
    SelectionBg,
    #[doc = r" Text selection foreground."]
    SelectionFg,
    #[doc = r" Menu highlight background."]
    HighlightBg,
    #[doc = r" Menu highlight foreground."]
    HighlightFg,
    #[doc = r" Destructive menu highlight background."]
    HighlightDangerBg,
    #[doc = r" Destructive menu highlight foreground."]
    HighlightDangerFg,
    #[doc = r" Backdrop foreground."]
    BackdropFg,
    #[doc = r" Backdrop background."]
    BackdropBg,
    #[doc = r" Danger."]
    Danger,
    #[doc = r" Danger at rest on a neutral plane."]
    DangerSoft,
    #[doc = r" Danger tint."]
    DangerTint,
    #[doc = r" Warning."]
    Warning,
    #[doc = r" Warning tint."]
    WarningTint,
    #[doc = r" Success."]
    Success,
    #[doc = r" Info."]
    Info,
    #[doc = r" Disabled foreground."]
    DisabledFg,
    #[doc = r" Disabled background."]
    DisabledBg,
    #[doc = r" Read-only foreground."]
    ReadOnlyFg,
    #[doc = r" A syntax role."]
    Syntax(SyntaxRole),
    #[doc = r" A meter role."]
    Meter(MeterRole),
    #[doc = r" The one documented raw-colour escape hatch; still downgraded."]
    Custom(Color),
}

#[doc = r" Text alignment (`StylePatch.align`, `CellUi::align`)."]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Align {
    #[doc = r" Left."]
    #[default]
    Left,
    #[doc = r" Centre."]
    Center,
    #[doc = r" Right."]
    Right,
}

impl Role {
    #[doc = r" Retain the source surface's neutral backdrop plane in per-cell"]
    #[doc = r" provenance; modal dimming happens after the source scope has exited."]
    pub const fn painted(self, surface: Surface) -> Self {
        match self {
            Self::HoverSurface => Self::Surface(match surface {
                Surface::Canvas | Surface::Field => Surface::Elevated,
                _ => Surface::Overlay,
            }),
            other => other,
        }
    }
}
