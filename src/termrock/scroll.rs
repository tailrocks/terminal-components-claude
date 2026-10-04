//! Shared scroll model, scrollbar drawing, thumb drag capture, and edge fade.
//!
//! Provides the canonical [`ScrollState`], [`ScrollRegion`], and [`ScrollAction`]
//! used across all Termrock collection and viewport components.

use std::ops::Range;

use crate::termrock::identity::{Id, Part};
use crate::termrock::layout::{Axis, Constraints, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, MouseKind, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};

/// Durable scroll position and extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollState {
    pub offset: usize,
    pub total: usize,
    pub viewport: usize,
    pub follow: bool,
}

impl ScrollState {
    pub const fn new(offset: usize, total: usize, viewport: usize) -> Self {
        Self {
            offset,
            total,
            viewport,
            follow: false,
        }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn total(&self) -> usize {
        self.total
    }

    pub const fn viewport(&self) -> usize {
        self.viewport
    }

    pub const fn follow(&self) -> bool {
        self.follow
    }

    pub fn set_follow(&mut self, follow: bool) {
        self.follow = follow;
        if follow {
            self.offset = self.max_offset();
        }
    }

    pub const fn max_offset(&self) -> usize {
        self.total.saturating_sub(self.viewport)
    }

    pub const fn is_overflowing(&self) -> bool {
        self.total > self.viewport
    }

    pub fn ratio(&self) -> f64 {
        let max = self.max_offset();
        if max == 0 {
            0.0
        } else {
            (self.offset as f64) / (max as f64)
        }
    }

    pub fn visible_range(&self) -> Range<usize> {
        let start = self.offset.min(self.total);
        let end = (self.offset + self.viewport).min(self.total);
        start..end
    }

    pub fn clamp(&mut self) {
        let max = self.max_offset();
        if self.offset > max {
            self.offset = max;
        }
    }

    pub fn scroll_to(&mut self, target: usize) -> bool {
        let max = self.max_offset();
        let clamped = target.min(max);
        if self.offset != clamped {
            self.offset = clamped;
            self.follow = self.offset == max;
            true
        } else {
            false
        }
    }

    pub fn scroll_by(&mut self, delta: isize) -> bool {
        let current = self.offset as isize;
        let target = (current + delta).max(0) as usize;
        self.scroll_to(target)
    }

    pub fn scroll_up(&mut self, delta: usize) -> bool {
        self.scroll_by(-(delta as isize))
    }

    pub fn scroll_down(&mut self, delta: usize) -> bool {
        self.scroll_by(delta as isize)
    }

    pub fn ensure_visible(&mut self, index: usize) -> bool {
        if self.viewport == 0 || index >= self.total {
            return false;
        }
        if index < self.offset {
            self.scroll_to(index)
        } else if index >= self.offset + self.viewport {
            let new_offset = index + 1 - self.viewport;
            self.scroll_to(new_offset)
        } else {
            false
        }
    }
}

/// Visibility policy for scrollbars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScrollbarPolicy {
    /// Render scrollbar only when content extent exceeds viewport.
    #[default]
    Auto,
    /// Always render scrollbar track and thumb.
    Always,
    /// Never render scrollbars.
    Never,
}

/// Edge fade hint policy for overflowing content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FadePolicy {
    /// Automatically fade edges when hidden content exists and viewport is tall enough.
    #[default]
    Auto,
    /// Always compute edge fade.
    Always,
    /// Never apply edge fade.
    Never,
}

/// A protected row or range that must not be edge-faded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtectedRange {
    pub start: usize,
    pub end: usize,
}

impl ProtectedRange {
    pub const fn single(row: usize) -> Self {
        Self {
            start: row,
            end: row + 1,
        }
    }

    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn contains(&self, row: usize) -> bool {
        row >= self.start && row < self.end
    }
}

/// Typed actions emitted by a scroll region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollAction {
    OffsetChanged { offset: usize },
    FollowChanged(bool),
}

/// Depth of edge fade (in terminal rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeDepth {
    None,
    OneRow,
    TwoRows,
}

/// Shared scroll region component.
#[derive(Debug, Clone)]
pub struct ScrollRegion<'a> {
    pub owner: Id,
    pub axis: Axis,
    pub content_length: usize,
    pub viewport_length: usize,
    pub bars: ScrollbarPolicy,
    pub fade: FadePolicy,
    pub protected: &'a [ProtectedRange],
}

impl<'a> ScrollRegion<'a> {
    pub fn new(owner: Id, axis: Axis, content_length: usize, viewport_length: usize) -> Self {
        Self {
            owner,
            axis,
            content_length,
            viewport_length,
            bars: ScrollbarPolicy::Auto,
            fade: FadePolicy::Auto,
            protected: &[],
        }
    }

    pub fn bars(mut self, policy: ScrollbarPolicy) -> Self {
        self.bars = policy;
        self
    }

    pub fn fade(mut self, policy: FadePolicy) -> Self {
        self.fade = policy;
        self
    }

    pub fn protected(mut self, protected: &'a [ProtectedRange]) -> Self {
        self.protected = protected;
        self
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut ScrollState) -> Response<ScrollAction> {
        let track_id = self.owner.sub("scrollbar").sub("track");
        let thumb_id = self.owner.sub("scrollbar").sub("thumb");

        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.owner)
            || intended == Some(&track_id)
            || intended == Some(&thumb_id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.owner);

        let mut changed = false;

        if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
            match m.kind {
                MouseKind::WheelUp | MouseKind::WheelLeft if is_target => {
                    changed = state.scroll_by(-3);
                }
                MouseKind::WheelDown | MouseKind::WheelRight if is_target => {
                    changed = state.scroll_by(3);
                }
                MouseKind::Down => {
                    if intended == Some(&thumb_id) {
                        cx.capture_pointer(thumb_id.clone());
                    } else if intended == Some(&track_id) {
                        // Click on track: jump page or towards click
                        let viewport = state.viewport.max(1);
                        if (m.pos.y as usize) < state.offset {
                            changed = state.scroll_by(-(viewport as isize));
                        } else {
                            changed = state.scroll_by(viewport as isize);
                        }
                    }
                }
                MouseKind::Drag if cx.pointer_capture.as_ref() == Some(&thumb_id) => {
                    let total = state.total;
                    let viewport = state.viewport;
                    if total > viewport && viewport > 0 {
                        let track_len = viewport as f64;
                        let rel_y = (m.pos.y as f64).clamp(0.0, track_len);
                        let new_ratio = rel_y / track_len;
                        let new_offset = (new_ratio * state.max_offset() as f64).round() as usize;
                        changed = state.scroll_to(new_offset);
                    }
                }
                MouseKind::Up if cx.pointer_capture.as_ref() == Some(&thumb_id) => {
                    cx.release_capture();
                }
                _ => {}
            }
        }

        if changed {
            cx.request_invalidate(Invalidate::Paint);
            Response::action(
                self.owner.clone(),
                ScrollAction::OffsetChanged {
                    offset: state.offset,
                },
            )
            .with_flow(Flow::Consumed)
            .with_invalidate(Invalidate::Paint)
        } else if is_target {
            Response::consumed(self.owner.clone())
        } else {
            Response::bubble(self.owner.clone())
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ScrollState) -> Rect {
        let show_bar = match self.bars {
            ScrollbarPolicy::Always => true,
            ScrollbarPolicy::Never => false,
            ScrollbarPolicy::Auto => state.is_overflowing(),
        };

        if show_bar && !area.is_empty() {
            let track_id = self.owner.sub("scrollbar").sub("track");
            let thumb_id = self.owner.sub("scrollbar").sub("thumb");

            match self.axis {
                Axis::Vertical => {
                    let bar_x = area.x + area.width.saturating_sub(1);
                    let track_rect = Rect::new(bar_x, area.y, 1, area.height);

                    let total = state.total.max(1);
                    let viewport = state.viewport.max(1);
                    let track_h = area.height as usize;

                    let thumb_len = ((viewport * track_h) / total).clamp(1, track_h) as u16;
                    let free_h = area.height.saturating_sub(thumb_len);
                    let thumb_y = area.y + (free_h as f64 * state.ratio()).round() as u16;
                    let thumb_rect = Rect::new(bar_x, thumb_y, 1, thumb_len);

                    ui.register_hit(track_id.clone(), track_rect);
                    ui.register_hit(thumb_id.clone(), thumb_rect);

                    ui.part(self.owner.clone(), Part::TRACK, track_rect, |_p| {});
                    ui.part(self.owner.clone(), Part::THUMB, thumb_rect, |_p| {});
                }
                Axis::Horizontal => {
                    let bar_y = area.y + area.height.saturating_sub(1);
                    let track_rect = Rect::new(area.x, bar_y, area.width, 1);

                    let total = state.total.max(1);
                    let viewport = state.viewport.max(1);
                    let track_w = area.width as usize;

                    let thumb_len = ((viewport * track_w) / total).clamp(1, track_w) as u16;
                    let free_w = area.width.saturating_sub(thumb_len);
                    let thumb_x = area.x + (free_w as f64 * state.ratio()).round() as u16;
                    let thumb_rect = Rect::new(thumb_x, bar_y, thumb_len, 1);

                    ui.register_hit(track_id.clone(), track_rect);
                    ui.register_hit(thumb_id.clone(), thumb_rect);

                    ui.part(self.owner.clone(), Part::TRACK, track_rect, |_p| {});
                    ui.part(self.owner.clone(), Part::THUMB, thumb_rect, |_p| {});
                }
            }
        }

        area
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let needed = match self.axis {
            Axis::Vertical => Size::new(1, self.content_length as u16),
            Axis::Horizontal => Size::new(self.content_length as u16, 1),
        };
        constraints.clamp(needed)
    }

    /// Calculate edge fade depths for top and bottom edges.
    ///
    /// Respects the baseline height boundaries:
    /// - Height < 3: no fade
    /// - Height 3..11: 1 row fade
    /// - Height >= 12: 2 rows fade
    pub fn edge_fades(&self, height: u16, state: &ScrollState) -> (FadeDepth, FadeDepth) {
        if matches!(self.fade, FadePolicy::Never) || height < 3 || !state.is_overflowing() {
            return (FadeDepth::None, FadeDepth::None);
        }

        let max_depth = if height >= 12 {
            FadeDepth::TwoRows
        } else {
            FadeDepth::OneRow
        };

        let top = if state.offset > 0 {
            max_depth
        } else {
            FadeDepth::None
        };

        let bottom = if state.offset < state.max_offset() {
            max_depth
        } else {
            FadeDepth::None
        };

        (top, bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_clamping_and_ratio() {
        let mut state = ScrollState::new(0, 100, 20);
        assert_eq!(state.max_offset(), 80);
        assert_eq!(state.ratio(), 0.0);

        state.scroll_to(40);
        assert_eq!(state.offset, 40);
        assert!((state.ratio() - 0.5).abs() < 1e-6);

        state.scroll_to(200);
        assert_eq!(state.offset, 80);
        assert!((state.ratio() - 1.0).abs() < 1e-6);

        state.scroll_by(-10);
        assert_eq!(state.offset, 70);

        state.scroll_by(-100);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_ensure_visible() {
        let mut state = ScrollState::new(10, 100, 10); // visible 10..20
        assert!(!state.ensure_visible(15)); // already visible
        assert_eq!(state.offset, 10);

        assert!(state.ensure_visible(5)); // scroll up
        assert_eq!(state.offset, 5);

        assert!(state.ensure_visible(25)); // scroll down
        assert_eq!(state.offset, 16);
    }

    #[test]
    fn test_edge_fade_height_thresholds() {
        let region = ScrollRegion::new(Id::new("test"), Axis::Vertical, 100, 20);
        let middle_state = ScrollState::new(10, 100, 10);

        // Height < 3: no fade
        assert_eq!(
            region.edge_fades(2, &middle_state),
            (FadeDepth::None, FadeDepth::None)
        );

        // Height 3..11: 1 row fade
        assert_eq!(
            region.edge_fades(3, &middle_state),
            (FadeDepth::OneRow, FadeDepth::OneRow)
        );
        assert_eq!(
            region.edge_fades(11, &middle_state),
            (FadeDepth::OneRow, FadeDepth::OneRow)
        );

        // Height >= 12: 2 rows fade
        assert_eq!(
            region.edge_fades(12, &middle_state),
            (FadeDepth::TwoRows, FadeDepth::TwoRows)
        );

        // Top boundary: only bottom fades
        let top_state = ScrollState::new(0, 100, 10);
        assert_eq!(
            region.edge_fades(12, &top_state),
            (FadeDepth::None, FadeDepth::TwoRows)
        );

        // Bottom boundary: only top fades
        let bottom_state = ScrollState::new(90, 100, 10);
        assert_eq!(
            region.edge_fades(12, &bottom_state),
            (FadeDepth::TwoRows, FadeDepth::None)
        );
    }
}
