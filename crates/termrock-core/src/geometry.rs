//! Basic geometry dimensions for events and responses.

//! Terminal geometry, constraints, responsive helpers, and pure layout algorithms.
//!
//! Owns cell-exact [`Rect`] and [`Size`] arithmetic, track allocation with gaps,
//! responsive breakpoint splitting, and safe degenerate handling.

use ratatui_core::layout::Position as RatatuiPosition;
use ratatui_core::layout::Rect as RatatuiRect;

/// A 2D point in terminal cell coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl From<RatatuiPosition> for Position {
    fn from(p: RatatuiPosition) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<Position> for RatatuiPosition {
    fn from(p: Position) -> Self {
        Self { x: p.x, y: p.y }
    }
}

/// A 2D size in terminal cells.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

/// A rectangular area of terminal cells.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.x
            && pos.x < self.x.saturating_add(self.width)
            && pos.y >= self.y
            && pos.y < self.y.saturating_add(self.height)
    }

    pub fn intersect(&self, other: Rect) -> Rect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self
            .x
            .saturating_add(self.width)
            .min(other.x.saturating_add(other.width));
        let y2 = self
            .y
            .saturating_add(self.height)
            .min(other.y.saturating_add(other.height));
        if x2 > x1 && y2 > y1 {
            Rect::new(x1, y1, x2 - x1, y2 - y1)
        } else {
            Rect::new(x1, y1, 0, 0)
        }
    }

    pub fn union(&self, other: Rect) -> Rect {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return *self;
        }
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = self
            .x
            .saturating_add(self.width)
            .max(other.x.saturating_add(other.width));
        let y2 = self
            .y
            .saturating_add(self.height)
            .max(other.y.saturating_add(other.height));
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    pub fn inset(&self, dx: u16, dy: u16) -> Rect {
        let double_dx = dx.saturating_mul(2);
        let double_dy = dy.saturating_mul(2);
        if self.width <= double_dx || self.height <= double_dy {
            Rect::new(self.x.saturating_add(dx), self.y.saturating_add(dy), 0, 0)
        } else {
            Rect::new(
                self.x.saturating_add(dx),
                self.y.saturating_add(dy),
                self.width - double_dx,
                self.height - double_dy,
            )
        }
    }
}

impl From<RatatuiRect> for Rect {
    fn from(r: RatatuiRect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

impl From<Rect> for RatatuiRect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// Remaining space inside an area along all four sides.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Headroom {
    /// Rows above.
    pub up: u16,
    /// Rows below.
    pub down: u16,
    /// Columns to the left.
    pub left: u16,
    /// Columns to the right.
    pub right: u16,
}

/// Which axes a scroll region handles.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Axes {
    /// Vertical only.
    #[default]
    V,
    /// Horizontal only.
    H,
    /// Both axes.
    Both,
}


