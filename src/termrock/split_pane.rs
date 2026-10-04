//! Termrock two-pane split container and seam interaction component.
//!
//! Provides the canonical [`SplitPane`], [`SplitAreas`], [`SplitPaneState`], and [`SplitAction`]
//! supporting horizontal/vertical allocation, drag resize, minima clamping, and maximize.

use crate::core::event::MouseKind;
use crate::termrock::identity::{Id, Part};
use crate::termrock::layout::{Axis, Constraints, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::theme::{Role, StylePatch};
use ratatui::crossterm::event::KeyCode;

/// Layout regions calculated for a two-pane split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitAreas {
    pub first: Rect,
    pub second: Rect,
    pub seam: Rect,
}

/// Durable state for a two-pane split.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitPaneState {
    pub ratio: f32,
    pub maximized: Option<usize>,
    pub last_area: Rect,
    pub last_click_moment: Option<crate::termrock::response::Moment>,
}

impl Default for SplitPaneState {
    fn default() -> Self {
        Self::new(0.5)
    }
}

impl SplitPaneState {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.clamp(0.05, 0.95),
            maximized: None,
            last_area: Rect::zero(),
            last_click_moment: None,
        }
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.05, 0.95);
    }

    pub fn maximized(&self) -> Option<usize> {
        self.maximized
    }

    pub fn set_maximized(&mut self, pane: Option<usize>) {
        self.maximized = pane;
    }

    pub fn toggle_maximize(&mut self, pane: usize) {
        if self.maximized == Some(pane) {
            self.maximized = None;
        } else {
            self.maximized = Some(pane);
        }
    }
}

/// Typed actions emitted by a SplitPane.
#[derive(Debug, Clone, PartialEq)]
pub enum SplitAction {
    Resized { ratio: f32 },
    Maximized(Option<usize>),
}

/// Axis-parameterized two-pane allocation and seam interaction mechanism.
#[derive(Debug, Clone)]
pub struct SplitPane {
    pub id: Id,
    pub axis: Axis,
    pub min_first: u16,
    pub min_second: u16,
    pub seam_width: u16,
    pub resizable: bool,
    pub patch: Option<StylePatch>,
}

impl SplitPane {
    pub fn new(id: Id, axis: Axis) -> Self {
        Self {
            id,
            axis,
            min_first: 3,
            min_second: 3,
            seam_width: 1,
            resizable: true,
            patch: None,
        }
    }

    pub fn minima(mut self, first: u16, second: u16) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    pub fn seam_width(mut self, seam_width: u16) -> Self {
        self.seam_width = seam_width;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Calculate pane and seam rectangles for an area and state.
    pub fn layout(&self, area: Rect, state: &SplitPaneState) -> SplitAreas {
        if area.is_empty() {
            return SplitAreas {
                first: Rect::zero(),
                second: Rect::zero(),
                seam: Rect::zero(),
            };
        }

        match state.maximized {
            Some(0) => SplitAreas {
                first: area,
                second: Rect::zero(),
                seam: Rect::zero(),
            },
            Some(1) => SplitAreas {
                first: Rect::zero(),
                second: area,
                seam: Rect::zero(),
            },
            _ => match self.axis {
                Axis::Horizontal => {
                    let seam_w = if self.resizable { self.seam_width } else { 0 };
                    let usable = area.width.saturating_sub(seam_w);
                    if usable < self.min_first.saturating_add(self.min_second) {
                        return SplitAreas {
                            first: area,
                            second: Rect::zero(),
                            seam: Rect::zero(),
                        };
                    }

                    let mut first_w = ((usable as f32) * state.ratio).round() as u16;
                    first_w = first_w.clamp(self.min_first, usable.saturating_sub(self.min_second));
                    let second_w = usable.saturating_sub(first_w);

                    let first = Rect::new(area.x, area.y, first_w, area.height);
                    let seam =
                        Rect::new(area.x.saturating_add(first_w), area.y, seam_w, area.height);
                    let second = Rect::new(
                        area.x.saturating_add(first_w).saturating_add(seam_w),
                        area.y,
                        second_w,
                        area.height,
                    );
                    SplitAreas {
                        first,
                        second,
                        seam,
                    }
                }
                Axis::Vertical => {
                    let seam_h = if self.resizable { self.seam_width } else { 0 };
                    let usable = area.height.saturating_sub(seam_h);
                    if usable < self.min_first.saturating_add(self.min_second) {
                        return SplitAreas {
                            first: area,
                            second: Rect::zero(),
                            seam: Rect::zero(),
                        };
                    }

                    let mut first_h = ((usable as f32) * state.ratio).round() as u16;
                    first_h = first_h.clamp(self.min_first, usable.saturating_sub(self.min_second));
                    let second_h = usable.saturating_sub(first_h);

                    let first = Rect::new(area.x, area.y, area.width, first_h);
                    let seam =
                        Rect::new(area.x, area.y.saturating_add(first_h), area.width, seam_h);
                    let second = Rect::new(
                        area.x,
                        area.y.saturating_add(first_h).saturating_add(seam_h),
                        area.width,
                        second_h,
                    );
                    SplitAreas {
                        first,
                        second,
                        seam,
                    }
                }
            },
        }
    }

    /// Process input and return typed actions.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut SplitPaneState) -> Response<SplitAction> {
        let seam_id = self.id.sub("seam");
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id) || intended == Some(&seam_id);

        if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
            match m.kind {
                MouseKind::Down if self.resizable => {
                    if intended == Some(&seam_id) || cx.contains_point(&seam_id, m.pos.into()) {
                        let now = cx.moment();
                        let is_double_click = state.last_click_moment.is_some_and(|last| {
                            now.as_millis().saturating_sub(last.as_millis()) <= 350
                        });
                        state.last_click_moment = Some(now);

                        if is_double_click {
                            state.toggle_maximize(0);
                            cx.request_invalidate(Invalidate::Layout);
                            return Response::action(
                                self.id.clone(),
                                SplitAction::Maximized(state.maximized),
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Layout);
                        }

                        cx.capture_pointer(seam_id.clone());
                        return Response::consumed(self.id.clone());
                    }
                }
                MouseKind::Drag
                    if self.resizable && cx.pointer_capture.as_ref() == Some(&seam_id) =>
                {
                    let area = cx
                        .published_geometry
                        .and_then(|g| g.get(&self.id))
                        .copied()
                        .unwrap_or(state.last_area);

                    if !area.is_empty() {
                        let new_ratio = match self.axis {
                            Axis::Horizontal => {
                                let usable = area.width.saturating_sub(self.seam_width);
                                if usable > self.min_first.saturating_add(self.min_second) {
                                    let offset = m.pos.x.saturating_sub(area.x);
                                    let first = offset.clamp(
                                        self.min_first,
                                        usable.saturating_sub(self.min_second),
                                    );
                                    (first as f32) / (usable as f32)
                                } else {
                                    state.ratio
                                }
                            }
                            Axis::Vertical => {
                                let usable = area.height.saturating_sub(self.seam_width);
                                if usable > self.min_first.saturating_add(self.min_second) {
                                    let offset = m.pos.y.saturating_sub(area.y);
                                    let first = offset.clamp(
                                        self.min_first,
                                        usable.saturating_sub(self.min_second),
                                    );
                                    (first as f32) / (usable as f32)
                                } else {
                                    state.ratio
                                }
                            }
                        };

                        let clamped = new_ratio.clamp(0.05, 0.95);
                        if (clamped - state.ratio).abs() > 0.001 {
                            state.ratio = clamped;
                            cx.request_invalidate(Invalidate::Layout);
                            return Response::action(
                                self.id.clone(),
                                SplitAction::Resized { ratio: state.ratio },
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Layout);
                        }
                    }
                    return Response::consumed(self.id.clone());
                }
                MouseKind::Up if cx.pointer_capture.as_ref() == Some(&seam_id) => {
                    cx.release_capture();
                    return Response::consumed(self.id.clone());
                }
                _ => {}
            }
        } else if let UpdateCause::Input(Input::Key(k), _) = cx.cause()
            && is_target
        {
            match k.code {
                KeyCode::Char('+') | KeyCode::Char('=') if self.resizable => {
                    let next = (state.ratio + 0.05).clamp(0.05, 0.95);
                    if (next - state.ratio).abs() > 0.001 {
                        state.ratio = next;
                        cx.request_invalidate(Invalidate::Layout);
                        return Response::action(
                            self.id.clone(),
                            SplitAction::Resized { ratio: state.ratio },
                        )
                        .with_flow(Flow::Consumed);
                    }
                }
                KeyCode::Char('-') if self.resizable => {
                    let next = (state.ratio - 0.05).clamp(0.05, 0.95);
                    if (next - state.ratio).abs() > 0.001 {
                        state.ratio = next;
                        cx.request_invalidate(Invalidate::Layout);
                        return Response::action(
                            self.id.clone(),
                            SplitAction::Resized { ratio: state.ratio },
                        )
                        .with_flow(Flow::Consumed);
                    }
                }
                KeyCode::Char('m') | KeyCode::Char('z') => {
                    state.toggle_maximize(0);
                    cx.request_invalidate(Invalidate::Layout);
                    return Response::action(
                        self.id.clone(),
                        SplitAction::Maximized(state.maximized),
                    )
                    .with_flow(Flow::Consumed);
                }
                _ => {}
            }
        }

        if is_target {
            Response::consumed(self.id.clone())
        } else {
            Response::bubble(self.id.clone())
        }
    }

    /// Measure minimum size required for the split container.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let needed = match self.axis {
            Axis::Horizontal => Size::new(
                self.min_first
                    .saturating_add(self.min_second)
                    .saturating_add(self.seam_width),
                constraints.min.height,
            ),
            Axis::Vertical => Size::new(
                constraints.min.width,
                self.min_first
                    .saturating_add(self.min_second)
                    .saturating_add(self.seam_width),
            ),
        };
        constraints.clamp(needed)
    }

    /// Draw the split pane and seam, invoking the body closure with the resulting areas.
    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &SplitPaneState,
        body: impl FnOnce(&mut Ui<'_>, SplitAreas) -> R,
    ) -> R {
        let areas = self.layout(area, state);

        ui.register_hit(self.id.clone(), area);

        if self.resizable && !areas.seam.is_empty() {
            let seam_id = self.id.sub("seam");
            ui.register_hit(seam_id.clone(), areas.seam);

            let hovered = ui.hovered.as_ref() == Some(&seam_id);
            let pressed = ui.pressed.as_ref() == Some(&seam_id);

            let glyph = match (self.axis, pressed) {
                (Axis::Horizontal, false) => "│",
                (Axis::Horizontal, true) => "┃",
                (Axis::Vertical, false) => "─",
                (Axis::Vertical, true) => "━",
            };

            let tone = if pressed {
                Role::BorderFocused
            } else if hovered {
                Role::BorderSelected
            } else {
                Role::Border
            };

            let style = ui.theme.resolve_style(tone, ui.current_surface);
            ui.part(self.id.clone(), Part::new("seam"), areas.seam, |_p| {});

            for y in areas.seam.y..areas.seam.bottom() {
                for x in areas.seam.x..areas.seam.right() {
                    ui.set_string(x, y, glyph, style);
                }
            }
        }

        body(ui, areas)
    }
}
