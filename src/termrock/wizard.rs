//! Termrock Wizard multi-step modal component, step progression, and body slot composition.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::termrock::identity::{Id, ItemKey, Keyed, Revision};
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, MouseKind, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::text::{truncate, width};
use crate::termrock::theme::StylePatch;

/// A single step descriptor in a Wizard.
#[derive(Debug, Clone)]
pub struct WizardStep<'a> {
    pub key: ItemKey,
    pub title: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> WizardStep<'a> {
    pub const fn new(key: ItemKey, title: &'a str) -> Self {
        Self {
            key,
            title,
            description: None,
        }
    }

    pub fn description(mut self, desc: &'a str) -> Self {
        self.description = Some(desc);
        self
    }
}

impl<'a> Keyed for WizardStep<'a> {
    fn key(&self) -> ItemKey {
        self.key
    }
}

/// Navigation button targets in a Wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardButton {
    Cancel,
    Back,
    Next,
    Finish,
}

/// Navigation outcome emitted by Wizard interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardAction {
    Next { from: ItemKey },
    Back { from: ItemKey },
    Finish,
    Cancel,
}

/// Durable view state for a Wizard.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WizardState {
    pub current_step: Option<ItemKey>,
    pub visited: Vec<ItemKey>,
    pub focused_button: Option<WizardButton>,
    pub last_revision: Option<Revision>,
}

impl WizardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_step(step: ItemKey) -> Self {
        Self {
            current_step: Some(step),
            visited: Vec::new(),
            focused_button: None,
            last_revision: None,
        }
    }

    pub const fn current_step(&self) -> Option<ItemKey> {
        self.current_step
    }

    pub fn visited(&self) -> &[ItemKey] {
        &self.visited
    }

    pub fn is_visited(&self, key: ItemKey) -> bool {
        self.visited.contains(&key)
    }

    pub fn advance_to(&mut self, next: ItemKey) {
        if let Some(cur) = self.current_step.filter(|c| !self.visited.contains(c)) {
            self.visited.push(cur);
        }
        self.current_step = Some(next);
    }

    pub fn rewind_to(&mut self, prev: ItemKey) {
        self.current_step = Some(prev);
    }
}

/// A multi-step modal wizard container with step indicators, body slot, and navigation buttons.
#[derive(Debug, Clone)]
pub struct Wizard<'a> {
    pub id: Id,
    pub steps: &'a [WizardStep<'a>],
    pub revision: Revision,
    pub can_advance: bool,
    pub can_finish: bool,
    pub busy: bool,
    pub patch: Option<StylePatch>,
}

impl<'a> Wizard<'a> {
    pub fn new(id: Id, steps: &'a [WizardStep<'a>], revision: Revision) -> Self {
        Self {
            id,
            steps,
            revision,
            can_advance: true,
            can_finish: true,
            busy: false,
            patch: None,
        }
    }

    pub fn can_advance(mut self, eligible: bool) -> Self {
        self.can_advance = eligible;
        self
    }

    pub fn can_finish(mut self, eligible: bool) -> Self {
        self.can_finish = eligible;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn frame_rect(&self, area: Rect) -> Rect {
        let w = (area.width * 4 / 5).clamp(44, 96).min(area.width);
        let h = (area.height * 4 / 5).clamp(14, 28).min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        Rect::new(x, y, w, h)
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        constraints.clamp(Size::new(64, 18))
    }

    fn current_index(&self, state: &WizardState) -> usize {
        state
            .current_step
            .and_then(|k| self.steps.iter().position(|s| s.key == k))
            .unwrap_or(0)
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut WizardState) -> Response<WizardAction> {
        if self.steps.is_empty() {
            return Response::bubble(self.id.clone());
        }

        // Reconcile current step
        if state.current_step.is_none()
            || !self.steps.iter().any(|s| Some(s.key) == state.current_step)
        {
            state.current_step = self.steps.first().map(|s| s.key);
        }

        let cur_idx = self.current_index(state);
        let cur_key = self.steps[cur_idx].key;
        let is_first = cur_idx == 0;
        let is_last = cur_idx + 1 >= self.steps.len();

        let area = Rect::new(0, 0, 80, 24);
        let frame = self.frame_rect(area);

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Esc => {
                    if !self.busy {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(self.id.clone(), WizardAction::Cancel)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Tab | KeyCode::Right => {
                    let mut buttons = vec![WizardButton::Cancel];
                    if !is_first {
                        buttons.push(WizardButton::Back);
                    }
                    if is_last {
                        if self.can_finish && !self.busy {
                            buttons.push(WizardButton::Finish);
                        }
                    } else if self.can_advance && !self.busy {
                        buttons.push(WizardButton::Next);
                    }

                    if !buttons.is_empty() {
                        let curr_btn_idx = state
                            .focused_button
                            .and_then(|b| buttons.iter().position(|&x| x == b))
                            .unwrap_or(buttons.len().saturating_sub(1));
                        let next_idx = (curr_btn_idx + 1) % buttons.len();
                        state.focused_button = Some(buttons[next_idx]);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::BackTab | KeyCode::Left => {
                    let mut buttons = vec![WizardButton::Cancel];
                    if !is_first {
                        buttons.push(WizardButton::Back);
                    }
                    if is_last {
                        if self.can_finish && !self.busy {
                            buttons.push(WizardButton::Finish);
                        }
                    } else if self.can_advance && !self.busy {
                        buttons.push(WizardButton::Next);
                    }

                    if !buttons.is_empty() {
                        let curr_btn_idx = state
                            .focused_button
                            .and_then(|b| buttons.iter().position(|&x| x == b))
                            .unwrap_or(0);
                        let prev_idx = if curr_btn_idx == 0 {
                            buttons.len().saturating_sub(1)
                        } else {
                            curr_btn_idx - 1
                        };
                        state.focused_button = Some(buttons[prev_idx]);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Enter => {
                    let chosen_btn = state.focused_button.unwrap_or(if is_last {
                        WizardButton::Finish
                    } else {
                        WizardButton::Next
                    });

                    match chosen_btn {
                        WizardButton::Cancel => {
                            if !self.busy {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), WizardAction::Cancel)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                        WizardButton::Back => {
                            if !is_first && !self.busy {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    WizardAction::Back { from: cur_key },
                                )
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                            }
                        }
                        WizardButton::Next => {
                            if !is_last && self.can_advance && !self.busy {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    WizardAction::Next { from: cur_key },
                                )
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                            }
                        }
                        WizardButton::Finish => {
                            if is_last && self.can_finish && !self.busy {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), WizardAction::Finish)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                    }
                }
                _ => {}
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::new(m.pos.x, m.pos.y);
                if m.kind == MouseKind::Up {
                    if !frame.contains(pos) && !self.busy {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(self.id.clone(), WizardAction::Cancel)
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                    } else if frame.contains(pos) {
                        // Check button clicks in button row
                        let btn_y = frame.y + frame.height.saturating_sub(2);
                        if pos.y == btn_y {
                            // Right to left buttons: Finish/Next, Back
                            let mut curr_x = frame.x + frame.width.saturating_sub(2);

                            // Primary button: Next or Finish
                            let primary_label = if is_last { " Finish " } else { " Next › " };
                            let primary_w = (width(primary_label) + 2) as u16;
                            curr_x = curr_x.saturating_sub(primary_w);
                            let primary_rect = Rect::new(curr_x, btn_y, primary_w, 1);
                            if primary_rect.contains(pos) {
                                if is_last && self.can_finish && !self.busy {
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(self.id.clone(), WizardAction::Finish)
                                        .with_flow(Flow::Consumed)
                                        .with_invalidate(Invalidate::Paint);
                                } else if !is_last && self.can_advance && !self.busy {
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        WizardAction::Next { from: cur_key },
                                    )
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                                }
                            }

                            // Back button
                            if !is_first {
                                curr_x = curr_x.saturating_sub(1);
                                let back_label = " ‹ Back ";
                                let back_w = (width(back_label) + 2) as u16;
                                curr_x = curr_x.saturating_sub(back_w);
                                let back_rect = Rect::new(curr_x, btn_y, back_w, 1);
                                if back_rect.contains(pos) && !self.busy {
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(
                                        self.id.clone(),
                                        WizardAction::Back { from: cur_key },
                                    )
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                                }
                            }

                            // Cancel button on the left
                            let cancel_rect = Rect::new(frame.x + 2, btn_y, 10, 1);
                            if cancel_rect.contains(pos) && !self.busy {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), WizardAction::Cancel)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Response::bubble(self.id.clone())
    }

    pub fn draw<R>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &WizardState,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> R {
        if area.is_empty() {
            return body(ui, Rect::zero());
        }

        let theme = ui.theme;
        let frame = self.frame_rect(area);

        // Dim backdrop outside frame
        if frame.width < area.width || frame.height < area.height {
            let dim_style = Style::new().fg(theme.tokens.text_muted);
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    let pos = Position::new(x, y);
                    if !frame.contains(pos) {
                        ui.set_string(x, y, " ", dim_style);
                    }
                }
            }
        }

        ui.register_focus(self.id.clone(), true);
        ui.register_hit(self.id.clone(), frame);

        let bg = theme.tokens.surface_elevated;
        let border_fg = theme.tokens.border_strong;
        let border_style = Style::new().bg(bg).fg(border_fg);

        ui.fill_rect(frame, Style::new().bg(bg));

        let right_x = frame.x + frame.width.saturating_sub(1);
        let bottom_y = frame.y + frame.height.saturating_sub(1);

        // Borders
        ui.set_string(frame.x, frame.y, "┌", border_style);
        ui.set_string(right_x, frame.y, "┐", border_style);
        ui.set_string(frame.x, bottom_y, "└", border_style);
        ui.set_string(right_x, bottom_y, "┘", border_style);

        for x in (frame.x + 1)..right_x {
            ui.set_string(x, frame.y, "─", border_style);
            ui.set_string(x, bottom_y, "─", border_style);
        }
        for y in (frame.y + 1)..bottom_y {
            ui.set_string(frame.x, y, "│", border_style);
            ui.set_string(right_x, y, "│", border_style);
        }

        let cur_idx = self.current_index(state);
        let total_steps = self.steps.len().max(1);
        let cur_step = self.steps.get(cur_idx);

        // Step title in top border
        let step_title = cur_step.map_or("", |s| s.title);
        let header_str = format!(" Step {} of {}: {} ", cur_idx + 1, total_steps, step_title);
        let max_w = (frame.width.saturating_sub(4)) as usize;
        let tr_header = truncate(&header_str, max_w);
        let header_style = Style::new()
            .bg(bg)
            .fg(theme.tokens.accent)
            .add_modifier(Modifier::BOLD);
        ui.set_string(frame.x + 2, frame.y, &tr_header, header_style);

        // Separator above action buttons
        if frame.height >= 5 {
            let sep_y = bottom_y.saturating_sub(2);
            ui.set_string(frame.x, sep_y, "├", border_style);
            ui.set_string(right_x, sep_y, "┤", border_style);
            for x in (frame.x + 1)..right_x {
                ui.set_string(x, sep_y, "─", border_style);
            }
        }

        // Action buttons
        if frame.height >= 4 {
            let btn_y = bottom_y.saturating_sub(1);
            let is_first = cur_idx == 0;
            let is_last = cur_idx + 1 >= total_steps;

            // Cancel button on left
            let cancel_focused = state.focused_button == Some(WizardButton::Cancel);
            let cancel_style = Style::new()
                .bg(if cancel_focused {
                    theme.tokens.highlight
                } else {
                    bg
                })
                .fg(if self.busy {
                    theme.tokens.text_muted
                } else {
                    theme.tokens.text_secondary
                })
                .add_modifier(if cancel_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            ui.set_string(frame.x + 2, btn_y, "[ Cancel ]", cancel_style);

            // Right-aligned buttons: Back, Next/Finish
            let mut curr_x = right_x.saturating_sub(1);

            // Primary (Next / Finish)
            let primary_label = if is_last {
                "[ Finish ]"
            } else {
                "[ Next › ]"
            };
            let primary_w = width(primary_label) as u16;
            curr_x = curr_x.saturating_sub(primary_w);
            let is_primary_focused = if is_last {
                state.focused_button == Some(WizardButton::Finish)
            } else {
                state.focused_button == Some(WizardButton::Next)
            };
            let primary_enabled = if is_last {
                self.can_finish && !self.busy
            } else {
                self.can_advance && !self.busy
            };
            let (prim_bg, prim_fg) = if !primary_enabled {
                (bg, theme.tokens.text_muted)
            } else if is_primary_focused {
                (theme.tokens.accent_hover, theme.tokens.text_on_accent)
            } else {
                (theme.tokens.accent, theme.tokens.text_on_accent)
            };
            let prim_style = Style::new()
                .bg(prim_bg)
                .fg(prim_fg)
                .add_modifier(Modifier::BOLD);
            ui.set_string(curr_x, btn_y, primary_label, prim_style);

            // Back button
            if !is_first {
                curr_x = curr_x.saturating_sub(2);
                let back_label = "[ ‹ Back ]";
                let back_w = width(back_label) as u16;
                curr_x = curr_x.saturating_sub(back_w);
                let back_focused = state.focused_button == Some(WizardButton::Back);
                let back_enabled = !self.busy;
                let back_style = Style::new()
                    .bg(if back_focused {
                        theme.tokens.highlight
                    } else {
                        theme.tokens.surface
                    })
                    .fg(if back_enabled {
                        theme.tokens.text_primary
                    } else {
                        theme.tokens.text_muted
                    })
                    .add_modifier(if back_focused {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    });
                ui.set_string(curr_x, btn_y, back_label, back_style);
            }
        }

        // Body slot
        let body_top = frame.y + 1;
        let body_bottom = if frame.height >= 5 {
            bottom_y.saturating_sub(2)
        } else {
            bottom_y
        };
        let body_h = body_bottom.saturating_sub(body_top);
        let body_w = frame.width.saturating_sub(2);
        let body_rect = Rect::new(frame.x + 1, body_top, body_w, body_h);

        body(ui, body_rect)
    }
}
