//! Complete-frame observation records taken from a production draw.

/// One terminal cell after production `App::draw`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedCell {
    /// Grapheme painted in this cell; empty on a wide-cell continuation.
    pub symbol: String,
    /// Debug-formatted foreground.
    pub fg: String,
    /// Debug-formatted background.
    pub bg: String,
    /// Packed style modifiers.
    pub modifier: u16,
}

impl ObservedCell {
    pub(crate) fn missing() -> Self {
        Self {
            symbol: String::new(),
            fg: "missing".to_owned(),
            bg: "missing".to_owned(),
            modifier: 0,
        }
    }

    /// Whether this cell is a wide-grapheme continuation (empty symbol).
    #[must_use]
    pub fn is_continuation(&self) -> bool {
        self.symbol.is_empty()
    }
}

/// Cursor location from the last production draw, when visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedCursor {
    /// Column.
    pub x: u16,
    /// Row.
    pub y: u16,
}

/// One complete observed frame: cells, cursor, route, and provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFrame {
    /// Scenario identity (`JA-001/{world}/…`).
    pub identity: String,
    /// CLI world name.
    pub scenario: String,
    /// Motion name (`paused`, `reduced`, `full`).
    pub motion: String,
    /// Pinned virtual frame used at construction.
    pub construct_frame: u64,
    /// App-reported virtual frame after draw.
    pub app_frame: u64,
    /// Viewport width.
    pub width: u16,
    /// Viewport height.
    pub height: u16,
    /// Color-level label.
    pub color: String,
    /// Theme label; oracle captures use Junie.
    pub theme: String,
    /// Production route after draw.
    pub route: String,
    /// Debug-formatted focus id, when the runtime has one.
    pub focus: Option<String>,
    /// Cursor after the production draw.
    pub cursor: Option<ObservedCursor>,
    /// Fixture clock milliseconds.
    pub now_ms: i64,
    /// Fixture clock seconds (epoch plus elapsed).
    pub now_secs: i64,
    /// Whether the fixture clock is running. Paused worlds stay `false`.
    pub clock_running: bool,
    /// Atmosphere seed observed from the production rain module.
    pub motion_seed: u64,
    /// Fixture epoch seconds.
    pub epoch_secs: i64,
    /// FNV-1a cell digest matching `junie_tui_testing::Scene`.
    pub digest: u64,
    /// Plain-text rows joined by newlines.
    pub text: String,
    /// Dense row-major cells, length `width * height`.
    pub cells: Vec<ObservedCell>,
    /// Whether production requested quit.
    pub quit: bool,
    /// Manager stable row key after the draw.
    pub selected_row: String,
}

impl ObservedFrame {
    /// Cell at `(x, y)` in the dense grid.
    #[must_use]
    pub fn cell_at(&self, x: u16, y: u16) -> Option<&ObservedCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = usize::from(y)
            .saturating_mul(usize::from(self.width))
            .saturating_add(usize::from(x));
        self.cells.get(idx)
    }

    /// Whether every matrix slot was recorded.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.cells.len() == usize::from(self.width).saturating_mul(usize::from(self.height))
    }

    /// Whether this frame was drawn at the size where `App::draw` overpaints.
    #[must_use]
    pub const fn at_historical_paint_size(&self) -> bool {
        self.width == crate::HISTORICAL_PAINT_SIZE.0
            && self.height == crate::HISTORICAL_PAINT_SIZE.1
    }
}
