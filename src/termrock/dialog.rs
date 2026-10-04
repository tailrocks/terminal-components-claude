//! Termrock Dialog modal component, tones, action buttons, and backdrop management.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::termrock::button::ButtonVariant;
use crate::termrock::empty::ActionMeta;
use crate::termrock::identity::{ActionKey, Id, Part};
use crate::termrock::layers::{DismissPolicy, DismissReason};
use crate::termrock::layout::{Constraints, Position, Rect, Size};
use crate::termrock::response::{
    ActivationOrigin, Flow, Input, Invalidate, MouseKind, Response, UpdateCause,
};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{truncate, width};
use crate::termrock::theme::StylePatch;

/// Semantic visual tone for a Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DialogTone {
    #[default]
    Info,
    Prompt,
    Confirm,
    Error,
    Destructive,
}

impl DialogTone {
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Prompt => "?",
            Self::Confirm => "✓",
            Self::Error => "✗",
            Self::Destructive => "⚠",
        }
    }
}

/// Action outcome emitted by Dialog interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogAction {
    Choose {
        action: ActionKey,
        origin: ActivationOrigin,
    },
    Dismiss {
        reason: DismissReason,
    },
}

impl DialogAction {
    pub const fn action(&self) -> Option<ActionKey> {
        match self {
            Self::Choose { action, .. } => Some(*action),
            Self::Dismiss { .. } => None,
        }
    }

    pub const fn is_dismiss(&self) -> bool {
        matches!(self, Self::Dismiss { .. })
    }
}

/// Durable view state for a Dialog.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DialogState {
    pub focused_action: Option<ActionKey>,
    pub scroll: ScrollState,
}

impl DialogState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_focused_action(mut self, action: ActionKey) -> Self {
        self.focused_action = Some(action);
        self
    }
}

/// A modal dialog frame with elevated border, dim backdrop, title, body slot, and action row.
#[derive(Debug, Clone)]
pub struct Dialog<'a> {
    pub id: Id,
    pub title: &'a str,
    pub actions: &'a [ActionMeta<'a>],
    pub tone: DialogTone,
    pub dismiss_policy: DismissPolicy,
    pub default_action: Option<ActionKey>,
    pub body_size: Option<Size>,
    pub patch: Option<StylePatch>,
    pub part_patches: Vec<(Part, StylePatch)>,
}

impl<'a> Dialog<'a> {
    pub fn new(id: Id, title: &'a str, actions: &'a [ActionMeta<'a>]) -> Self {
        Self {
            id,
            title,
            actions,
            tone: DialogTone::Info,
            dismiss_policy: DismissPolicy::Both,
            default_action: None,
            body_size: None,
            patch: None,
            part_patches: Vec::new(),
        }
    }

    pub fn tone(mut self, tone: DialogTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn dismiss(mut self, policy: DismissPolicy) -> Self {
        self.dismiss_policy = policy;
        self
    }

    pub fn default_action(mut self, action: Option<ActionKey>) -> Self {
        self.default_action = action;
        self
    }

    pub fn body_size(mut self, size: Size) -> Self {
        self.body_size = Some(size);
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn patch_part(mut self, part: Part, patch: StylePatch) -> Self {
        self.part_patches.push((part, patch));
        self
    }

    /// Resolve the frame rect centered in area.
    pub fn frame_rect(&self, area: Rect) -> Rect {
        let (needed_w, needed_h) = if let Some(bs) = self.body_size {
            let title_w = (width(self.title) + 6) as u16;
            let actions_w = self
                .actions
                .iter()
                .map(|a| (width(a.label) + 4) as u16)
                .sum::<u16>()
                + 4;
            let w = bs.width.max(title_w).max(actions_w).max(36) + 4;
            let h = bs.height + 6;
            (w, h)
        } else {
            let title_w = (width(self.title) + 6) as u16;
            let actions_w = self
                .actions
                .iter()
                .map(|a| (width(a.label) + 4) as u16)
                .sum::<u16>()
                + 4;
            let w = (area.width * 3 / 4)
                .max(title_w)
                .max(actions_w)
                .max(36)
                .min(area.width);
            let h = (area.height * 2 / 3).max(8).min(area.height);
            (w, h)
        };

        let w = needed_w.min(area.width);
        let h = needed_h.min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        Rect::new(x, y, w, h)
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let title_w = (width(self.title) + 6) as u16;
        let actions_w = self
            .actions
            .iter()
            .map(|a| (width(a.label) + 4) as u16)
            .sum::<u16>()
            + 4;

        let (body_w, body_h) = self.body_size.map_or((28, 4), |s| (s.width, s.height));

        let total_w = body_w.max(title_w).max(actions_w) + 4;
        let total_h = body_h + 6;
        constraints.clamp(Size::new(total_w, total_h))
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut DialogState) -> Response<DialogAction> {
        let area = Rect::new(0, 0, 80, 24); // Fallback reference area for frame check
        let frame = self.frame_rect(area);

        // Ensure focused action points to an existing action
        if state.focused_action.is_none() && !self.actions.is_empty() {
            state.focused_action = self
                .default_action
                .or_else(|| self.actions.first().map(|a| a.key));
        }

        match cx.cause() {
            UpdateCause::Input(Input::Key(k), _) => match k.code {
                KeyCode::Esc => {
                    if self.dismiss_policy.on_escape() {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(
                            self.id.clone(),
                            DialogAction::Dismiss {
                                reason: DismissReason::Escape,
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Tab | KeyCode::Right => {
                    if !self.actions.is_empty() {
                        let curr_idx = state
                            .focused_action
                            .and_then(|fk| self.actions.iter().position(|a| a.key == fk))
                            .unwrap_or(0);
                        let next_idx = (curr_idx + 1) % self.actions.len();
                        state.focused_action = Some(self.actions[next_idx].key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::BackTab | KeyCode::Left => {
                    if !self.actions.is_empty() {
                        let curr_idx = state
                            .focused_action
                            .and_then(|fk| self.actions.iter().position(|a| a.key == fk))
                            .unwrap_or(0);
                        let prev_idx = if curr_idx == 0 {
                            self.actions.len().saturating_sub(1)
                        } else {
                            curr_idx - 1
                        };
                        state.focused_action = Some(self.actions[prev_idx].key);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                }
                KeyCode::Enter => {
                    let target = state
                        .focused_action
                        .or(self.default_action)
                        .or_else(|| self.actions.first().map(|a| a.key));
                    if let Some(action_key) = target {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(
                            self.id.clone(),
                            DialogAction::Choose {
                                action: action_key,
                                origin: ActivationOrigin::Keyboard,
                            },
                        )
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                    }
                }
                _ => {}
            },
            UpdateCause::Input(Input::Mouse(m), _) => {
                let pos = Position::new(m.pos.x, m.pos.y);
                match m.kind {
                    MouseKind::Down | MouseKind::Up => {
                        // Check if outside dialog frame
                        if !frame.contains(pos) {
                            if self.dismiss_policy.on_outside_pointer() && m.kind == MouseKind::Up {
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    DialogAction::Dismiss {
                                        reason: DismissReason::OutsidePointer,
                                    },
                                )
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                            }
                        } else {
                            // Check button clicks in action bar
                            let mut action_x =
                                frame.x.saturating_add(frame.width).saturating_sub(2);
                            let action_y = frame.y.saturating_add(frame.height).saturating_sub(2);

                            for action in self.actions.iter().rev() {
                                let btn_w = (width(action.label) + 4) as u16;
                                action_x = action_x.saturating_sub(btn_w);
                                let btn_rect = Rect::new(action_x, action_y, btn_w, 1);
                                if btn_rect.contains(pos) {
                                    state.focused_action = Some(action.key);
                                    if m.kind == MouseKind::Up {
                                        cx.request_invalidate(Invalidate::Paint);
                                        return Response::action(
                                            self.id.clone(),
                                            DialogAction::Choose {
                                                action: action.key,
                                                origin: ActivationOrigin::Pointer,
                                            },
                                        )
                                        .with_flow(Flow::Consumed)
                                        .with_invalidate(Invalidate::Paint);
                                    }
                                    return Response::consumed(self.id.clone())
                                        .with_invalidate(Invalidate::Paint);
                                }
                                action_x = action_x.saturating_sub(1);
                            }
                        }
                    }
                    _ => {}
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
        state: &DialogState,
        body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
    ) -> R {
        if area.is_empty() {
            let dummy_rect = Rect::zero();
            return body(ui, dummy_rect);
        }

        let theme = ui.theme;
        let frame = self.frame_rect(area);

        // Dim backdrop if dialog area is smaller than container area
        if frame.width < area.width || frame.height < area.height {
            let dim_style = Style::new().fg(theme.tokens.text_muted);
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    let pos = Position::new(x, y);
                    if !frame.contains(pos) {
                        // Soft dim styling
                        ui.set_string(x, y, " ", dim_style);
                    }
                }
            }
        }

        ui.register_focus(self.id.clone(), true);
        ui.register_hit(self.id.clone(), frame);

        // Elevated surface frame
        let bg = theme.tokens.surface_elevated;
        let border_fg = match self.tone {
            DialogTone::Destructive | DialogTone::Error => theme.tokens.danger,
            DialogTone::Prompt => theme.tokens.accent,
            DialogTone::Confirm => theme.tokens.success,
            DialogTone::Info => theme.tokens.border_strong,
        };
        let frame_style = Style::new().bg(bg).fg(border_fg);

        // Fill background of frame
        ui.fill_rect(frame, Style::new().bg(bg));

        // Draw rounded borders
        let right_x = frame.x + frame.width.saturating_sub(1);
        let bottom_y = frame.y + frame.height.saturating_sub(1);

        // Corners
        ui.set_string(frame.x, frame.y, "┌", frame_style);
        ui.set_string(right_x, frame.y, "┐", frame_style);
        ui.set_string(frame.x, bottom_y, "└", frame_style);
        ui.set_string(right_x, bottom_y, "┘", frame_style);

        // Horizontal edges
        for x in (frame.x + 1)..right_x {
            ui.set_string(x, frame.y, "─", frame_style);
            ui.set_string(x, bottom_y, "─", frame_style);
        }

        // Vertical edges
        for y in (frame.y + 1)..bottom_y {
            ui.set_string(frame.x, y, "│", frame_style);
            ui.set_string(right_x, y, "│", frame_style);
        }

        // Title and Tone icon on top border
        let tone_icon = self.tone.icon();
        let title_text = format!(" {} {} ", tone_icon, self.title);
        let max_title_w = (frame.width.saturating_sub(4)) as usize;
        let truncated_title = truncate(&title_text, max_title_w);
        let title_style = Style::new()
            .bg(bg)
            .fg(border_fg)
            .add_modifier(Modifier::BOLD);
        ui.set_string(frame.x + 2, frame.y, &truncated_title, title_style);

        // Separator above action row
        if frame.height >= 5 {
            let sep_y = bottom_y.saturating_sub(2);
            ui.set_string(frame.x, sep_y, "├", frame_style);
            ui.set_string(right_x, sep_y, "┤", frame_style);
            for x in (frame.x + 1)..right_x {
                ui.set_string(x, sep_y, "─", frame_style);
            }
        }

        // Render Action Buttons
        if frame.height >= 4 && !self.actions.is_empty() {
            let action_y = bottom_y.saturating_sub(1);
            let mut action_x = right_x.saturating_sub(1);

            for action in self.actions.iter().rev() {
                let btn_w = (width(action.label) + 4) as u16;
                action_x = action_x.saturating_sub(btn_w);
                if action_x > frame.x {
                    let is_focused = state.focused_action == Some(action.key);
                    let btn_rect = Rect::new(action_x, action_y, btn_w, 1);
                    ui.register_hit(self.id.sub(action.key.name()), btn_rect);

                    let (btn_bg, btn_fg, btn_mod) = if is_focused {
                        (
                            theme.tokens.accent,
                            theme.tokens.text_on_accent,
                            Modifier::BOLD,
                        )
                    } else {
                        match action.variant {
                            ButtonVariant::Primary => (
                                theme.tokens.accent,
                                theme.tokens.text_on_accent,
                                Modifier::BOLD,
                            ),
                            ButtonVariant::Danger => (
                                theme.tokens.danger,
                                theme.tokens.text_on_accent,
                                Modifier::BOLD,
                            ),
                            _ => (
                                theme.tokens.surface,
                                theme.tokens.text_primary,
                                Modifier::empty(),
                            ),
                        }
                    };

                    let btn_style = Style::new().bg(btn_bg).fg(btn_fg).add_modifier(btn_mod);
                    ui.fill_rect(btn_rect, btn_style);
                    let label_str = format!(" {} ", action.label);
                    ui.set_string(action_x + 1, action_y, &label_str, btn_style);
                }
                action_x = action_x.saturating_sub(1);
            }
        }

        // Body area calculation
        let body_top = frame.y + 1;
        let body_bottom = if frame.height >= 5 {
            bottom_y.saturating_sub(2)
        } else {
            bottom_y
        };
        let body_h = body_bottom.saturating_sub(body_top);
        let body_w = frame.width.saturating_sub(2);
        let body_rect = Rect::new(frame.x + 1, body_top, body_w, body_h);

        // Draw body slot
        body(ui, body_rect)
    }
}
