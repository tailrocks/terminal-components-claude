//! Finite colour-mode axis for default page frames.

use junie_tui::ColorLevel;

/// Colour modes used by the 368 default page frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorSpec {
    /// 24-bit / truecolor.
    TrueColor,
    /// 256-colour palette.
    Ansi256,
    /// 16 ANSI colours.
    Ansi16,
    /// Monochrome / none / nocolor observation alias.
    Mono,
}

impl ColorSpec {
    /// Default SC-BASE colour axis (truecolor, 256, 16, mono).
    pub const DEFAULT_AXIS: [Self; 4] = [Self::TrueColor, Self::Ansi256, Self::Ansi16, Self::Mono];

    /// Stable identity token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "256",
            Self::Ansi16 => "16",
            Self::Mono => "mono",
        }
    }

    /// Production theme capability.
    #[must_use]
    pub const fn level(self) -> ColorLevel {
        match self {
            Self::TrueColor => ColorLevel::TrueColor,
            Self::Ansi256 => ColorLevel::Ansi256,
            Self::Ansi16 => ColorLevel::Ansi16,
            Self::Mono => ColorLevel::Mono,
        }
    }

    /// Parse a CLI/scenario colour token, including oracle aliases.
    ///
    /// Mirrors `apps/showcase/src/cli.rs` `--color` tokens; `nocolor` is the
    /// frozen-suite alias for the same monochrome observation.
    #[must_use]
    pub fn from_token(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "truecolor" | "24bit" => Some(Self::TrueColor),
            "256" | "ansi256" => Some(Self::Ansi256),
            "16" | "ansi16" => Some(Self::Ansi16),
            "none" | "mono" | "nocolor" => Some(Self::Mono),
            _ => None,
        }
    }
}

/// Terminal size used by scenario expansion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TerminalSize {
    /// Columns.
    pub cols: u16,
    /// Rows.
    pub rows: u16,
}

impl TerminalSize {
    /// Default SC-BASE sizes.
    pub const DEFAULT_AXIS: [Self; 4] = [
        Self { cols: 80, rows: 24 },
        Self {
            cols: 100,
            rows: 30,
        },
        Self {
            cols: 120,
            rows: 40,
        },
        Self {
            cols: 160,
            rows: 50,
        },
    ];

    /// Construct a size.
    #[must_use]
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }

    /// Identity token `80x24`.
    #[must_use]
    pub fn token(self) -> String {
        format!("{}x{}", self.cols, self.rows)
    }

    /// Parse `80x24`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        let (cols, rows) = value.split_once('x')?;
        Some(Self {
            cols: cols.parse().ok()?,
            rows: rows.parse().ok()?,
        })
    }

    /// Buffer/frame rectangle origin at (0, 0).
    #[must_use]
    pub const fn rect(self) -> junie_tui::Rect {
        junie_tui::Rect::new(0, 0, self.cols, self.rows)
    }
}
