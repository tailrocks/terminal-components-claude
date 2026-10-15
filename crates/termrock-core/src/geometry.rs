//! Scroll headroom and axis selection for scroll regions.
//!
//! Owns [`Headroom`], the remaining space inside an area along all four
//! sides, and [`Axes`], which axes a scroll region handles.

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

impl Axes {
    /// Whether the region handles `axis`.
    pub const fn handles(self, axis: crate::event::Axis) -> bool {
        matches!(
            (self, axis),
            (Axes::Both, _) | (Axes::V, crate::event::Axis::V) | (Axes::H, crate::event::Axis::H)
        )
    }
}
