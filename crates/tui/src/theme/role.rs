//! Colour roles and surfaces (`COMPONENT_ARCHITECTURE.md` §10, §11.2).
//!
//! A [`StylePatch`](super::StylePatch) names a [`Role`], never a colour;
//! roles bind to colours at the end of resolution against the live theme,
//! the current [`Surface`] and the colour capability.

use ratatui_core::style::Color;

/// Number of levels in the surface ladder.
pub const SURFACE_LEVELS: usize = 5;
/// Number of steps in the foreground ladder.
pub const FG_STEPS: usize = 5;

/// The background plane a component sits on. The first five form the
/// ordered ladder; `Field` and `FieldHover` are the two non-ladder planes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Surface {
    /// The page.
    #[default]
    Canvas,
    /// Chrome and panels.
    Surface,
    /// Cards.
    Elevated,
    /// Overlays.
    Overlay,
    /// Popovers and menus.
    Popover,
    /// A text field.
    Field,
    /// A hovered text field.
    FieldHover,
}

impl Surface {
    /// The ladder index, or `None` for the field planes.
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

    /// The ladder surface at index `i`, saturating at the last level.
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

/// A step on the foreground ladder.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FgStep {
    /// Primary text.
    Primary,
    /// Secondary text.
    Secondary,
    /// Muted text.
    Muted,
    /// Faint text.
    Faint,
    /// Ghost text (dimmed backdrops).
    Ghost,
}

impl FgStep {
    /// The ladder index.
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

/// A syntax colour role.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SyntaxRole {
    /// Keyword.
    Keyword,
    /// Identifier.
    Ident,
    /// String literal.
    Str,
    /// Number literal.
    Number,
    /// Operator.
    Operator,
    /// Punctuation.
    Punct,
    /// Comment.
    Comment,
    /// Plain text.
    Plain,
    /// Type name.
    TypeName,
    /// Function name.
    Function,
    /// Constant.
    Constant,
    /// Invalid token.
    Invalid,
    /// Deprecated token.
    Deprecated,
    /// Find-match background.
    MatchBg,
    /// Current find-match background.
    MatchCurrentBg,
    /// Matching bracket.
    BracketMatch,
    /// Error diagnostic.
    DiagError,
    /// Warning diagnostic.
    DiagWarning,
    /// Info diagnostic.
    DiagInfo,
}

/// A meter colour role.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MeterRole {
    /// Healthy.
    Low,
    /// Needs attention.
    Medium,
    /// Critical.
    High,
    /// The track.
    Track,
    /// The unfilled remainder.
    FillRest,
    /// Stale data.
    Stale,
    /// Unknown value.
    Unknown,
    /// Series `n` (wraps over six).
    Series(u8),
}

/// The complete set a `StylePatch` may name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    /// The background of the current surface.
    CurrentSurface,
    /// The background one ladder step above the current surface.
    RaisedSurface,
    /// A specific surface's background.
    Surface(Surface),
    /// A foreground ladder step.
    Fg(FgStep),
    /// Text on an accent fill.
    OnAccent,
    /// Text on a danger fill.
    OnDanger,
    /// Text on an inverted (fg-coloured) fill.
    OnSurfaceInverse,
    /// Subtle border.
    BorderSubtle,
    /// Strong border.
    BorderStrong,
    /// The accent.
    Accent,
    /// Accent, hovered.
    AccentHover,
    /// Accent, pressed.
    AccentPressed,
    /// Accent tint (selection background).
    AccentTint,
    /// Focus indicator.
    Focus,
    /// Focus ring.
    FocusRing,
    /// Text selection background.
    SelectionBg,
    /// Text selection foreground.
    SelectionFg,
    /// Menu highlight background.
    HighlightBg,
    /// Menu highlight foreground.
    HighlightFg,
    /// Destructive menu highlight background.
    HighlightDangerBg,
    /// Destructive menu highlight foreground.
    HighlightDangerFg,
    /// Backdrop foreground.
    BackdropFg,
    /// Backdrop background.
    BackdropBg,
    /// Danger.
    Danger,
    /// Danger at rest on a neutral plane.
    DangerSoft,
    /// Danger tint.
    DangerTint,
    /// Warning.
    Warning,
    /// Warning tint.
    WarningTint,
    /// Success.
    Success,
    /// Info.
    Info,
    /// Disabled foreground.
    DisabledFg,
    /// Disabled background.
    DisabledBg,
    /// Read-only foreground.
    ReadOnlyFg,
    /// A syntax role.
    Syntax(SyntaxRole),
    /// A meter role.
    Meter(MeterRole),
    /// The one documented raw-colour escape hatch; still downgraded.
    Custom(Color),
}

/// Text alignment (`StylePatch.align`, `CellUi::align`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Align {
    /// Left.
    #[default]
    Left,
    /// Centre.
    Center,
    /// Right.
    Right,
}
