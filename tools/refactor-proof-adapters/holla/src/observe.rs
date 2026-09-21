//! Direct observation of production App/Harness without repairing product UX.
//!
//! [`Session`] drives production `holla_app::App` through the real
//! `junie_tui::App` path inside `junie-tui-testing::Harness`: the same
//! runtime, focus settlement, and frame publication the shipped binary uses.
//! Checkpoints capture complete cells (plain text plus a symbol/style hash),
//! cursor, focus, and virtual milliseconds. Nothing here edits domain logic,
//! themes, or layouts to make an observation pass.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use holla_app::{App, Motion};
use junie_tui::{ColorLevel, FeedbackClock, SimulationMoment, Theme};
use junie_tui_testing::Harness;

use crate::construct::{Construction, UnavailableReason, construct};

/// Capture sizes required by HO-BASE rows.
pub const ORACLE_SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];

/// Palettes required by HO rows (`TrueColor,Ansi256,Ansi16,Mono` in the
/// frozen register).
pub const ORACLE_PALETTES: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

/// Status row in `draw_frame` is `area.bottom() - 2`.
///
/// Both the frozen oracle (`src/bin/holla/app.rs::draw_frame` in the
/// `visual-baseline` tag) and current production place the one-row status
/// surface two rows above the bottom edge.
#[must_use]
pub const fn status_row(height: u16) -> u16 {
    height.saturating_sub(2)
}

/// A settled direct observation of one production frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// Scenario or branch identity.
    pub identity: String,
    /// Terminal width.
    pub width: u16,
    /// Terminal height.
    pub height: u16,
    /// Color mode.
    pub palette: ColorLevel,
    /// Observed virtual milliseconds.
    pub now_ms: i64,
    /// Plain-text frame.
    pub text: String,
    /// Hash of cell symbols and styles, used for repeat identity.
    pub cells_hash: u64,
    /// Cursor position if visible.
    pub cursor: Option<(u16, u16)>,
    /// Focused control debug label.
    pub focus: Option<String>,
}

/// Production session driven through the real `junie_tui::App` path.
#[derive(Debug)]
pub struct Session {
    harness: Harness<App>,
    /// Interned oracle world.
    pub world: &'static str,
    /// Oracle tick ordinal used at construction.
    pub tick: u64,
    width: u16,
    height: u16,
    palette: ColorLevel,
}

/// Session could not be opened because production lacks the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionUnavailable {
    /// Requested world.
    pub world: String,
    /// Oracle tick.
    pub tick: u64,
    /// Cause.
    pub reason: UnavailableReason,
}

impl Session {
    /// Open a production harness for an oracle world/tick/size/palette.
    pub fn open(
        world: &str,
        motion: Motion,
        tick: u64,
        width: u16,
        height: u16,
        palette: ColorLevel,
    ) -> Result<Self, SessionUnavailable> {
        match construct(world, motion, tick) {
            Construction::Ready {
                world, tick, app, ..
            } => Ok(Self::from_app(*app, world, tick, width, height, palette)),
            Construction::Unavailable {
                world,
                tick,
                reason,
            } => Err(SessionUnavailable {
                world,
                tick,
                reason,
            }),
        }
    }

    fn from_app(
        app: App,
        world: &'static str,
        tick: u64,
        width: u16,
        height: u16,
        palette: ColorLevel,
    ) -> Self {
        let initial_ms = u64::try_from(app.fixture_time_ms()).unwrap_or(0);
        let harness = Harness::new_with_feedback_clock(
            app,
            Theme::junie(),
            width,
            height,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(initial_ms),
            },
        )
        .with_color(palette);
        Self {
            harness,
            world,
            tick,
            width,
            height,
            palette,
        }
    }

    /// Observed virtual milliseconds.
    #[must_use]
    pub fn now_ms(&self) -> i64 {
        self.harness.app().fixture_time_ms()
    }

    /// Deliver `n` explicit ticks at unchanged elapsed time.
    pub fn ticks(&mut self, n: usize) {
        self.harness.ticks(n);
    }

    /// Redraw the current frame.
    pub fn draw(&mut self) {
        self.harness.draw();
    }

    /// Advance simulation elapsed time by `ms` and run one scheduler turn.
    pub fn advance_ms(&mut self, ms: u64) {
        let _ = self.harness.advance(core::time::Duration::from_millis(ms));
    }

    /// Plain-text frame.
    #[must_use]
    pub fn text(&self) -> String {
        self.harness.text()
    }

    /// Capture a checkpoint.
    #[must_use]
    pub fn checkpoint(&self, identity: &str) -> Checkpoint {
        let cursor = self.harness.cursor().map(|pos| (pos.x, pos.y));
        let focus = self.harness.focus().map(|id| format!("{id:?}"));
        Checkpoint {
            identity: identity.to_owned(),
            width: self.width,
            height: self.height,
            palette: self.palette,
            now_ms: self.now_ms(),
            text: self.text(),
            cells_hash: cells_hash(&self.harness),
            cursor,
            focus,
        }
    }
}

fn cells_hash(harness: &Harness<App>) -> u64 {
    let mut hasher = DefaultHasher::new();
    let buffer = harness.buffer();
    let area = *buffer.area();
    area.width.hash(&mut hasher);
    area.height.hash(&mut hasher);
    let mut y = area.y;
    while y < area.bottom() {
        let mut x = area.x;
        while x < area.right() {
            if let Some(cell) = buffer.cell((x, y)) {
                cell.symbol().hash(&mut hasher);
                format!("{cell:?}").hash(&mut hasher);
            }
            x = x.saturating_add(1);
        }
        y = y.saturating_add(1);
    }
    hasher.finish()
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn status_row_is_bottom_minus_two() {
        assert_eq!(status_row(24), 22);
        assert_eq!(status_row(30), 28);
        assert_eq!(status_row(40), 38);
        assert_eq!(status_row(50), 48);
    }
}
