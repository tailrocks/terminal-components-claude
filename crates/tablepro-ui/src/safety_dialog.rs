use termrock::{
    FgStep, Focusability, Id, KeyCode, Modifier, Rect, Role, StylePatch, Surface, Ui, wrap,
};

pub const SAFETY_DIALOG: Id = Id::root("tablepro.safety-dialog");
pub const SAFETY_INPUT: Id = Id::root("tablepro.safety-input");
pub const SAFETY_CANCEL: Id = Id::root("tablepro.safety-cancel");
pub const SAFETY_CONFIRM: Id = Id::root("tablepro.safety-confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Secondary,
    Warning,
    Error,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,
}

impl Prop {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone: Tone::Normal,
            wrap: false,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyFocus {
    Input,
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyDialogAction {
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyIntent {
    Query,
    Commit,
}

#[derive(Debug, Clone)]
pub struct SafetyDialog {
    pub id: Id,
    pub title: String,
    pub facts: Vec<Prop>,
    pub code: Vec<String>,
    pub token: Option<String>,
    pub input_text: String,
    pub input_editing: bool,
    pub confirm_label: String,
    pub confirm_danger: bool,
    pub width: u16,
    pub focus: SafetyFocus,
    pub intent: SafetyIntent,
}

impl SafetyDialog {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        title: impl Into<String>,
        facts: Vec<Prop>,
        code: Vec<String>,
        token: Option<String>,
        confirm_label: impl Into<String>,
        confirm_danger: bool,
        width: u16,
        intent: SafetyIntent,
    ) -> Self {
        let tok = token;
        let initial_focus = if tok.is_some() {
            SafetyFocus::Input
        } else if confirm_danger {
            SafetyFocus::Cancel
        } else {
            SafetyFocus::Confirm
        };
        Self {
            id: Id::root("tablepro.safety-dialog"),
            title: title.into(),
            facts,
            code,
            token: tok,
            input_text: String::new(),
            input_editing: false,
            confirm_label: confirm_label.into(),
            confirm_danger,
            width,
            focus: initial_focus,
            intent,
        }
    }

    pub fn armed(&self) -> bool {
        match &self.token {
            Some(tok) => self.input_text.trim() == tok.trim(),
            None => true,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.input_editing
    }

    pub fn label_width(&self) -> usize {
        self.facts.iter().map(|p| p.label.len()).max().unwrap_or(0) + 2
    }

    fn code_rows(&self) -> u16 {
        if self.code.is_empty() {
            0
        } else {
            self.code.len().min(6) as u16 + 1
        }
    }

    fn ack_rows(&self) -> u16 {
        if self.token.is_some() {
            4 // 1 gap + 3 input field
        } else {
            0
        }
    }

    pub fn height(&self) -> u16 {
        let label_w = self.label_width();
        let inner_w = (self.width.saturating_sub(6) as usize).saturating_sub(label_w);
        let mut facts_rows = 0u16;
        for p in &self.facts {
            if p.wrap {
                let lines = wrap(&p.value, inner_w.max(4) as u16);
                facts_rows += lines.len() as u16;
            } else {
                facts_rows += 1;
            }
        }
        let body_h = facts_rows + self.code_rows() + self.ack_rows();
        // border(2) + pad(1) + title(1) + gap(1) + body + gap(1) + actions(1) + pad(1) = body_h + 8
        body_h + 8
    }

    pub fn on_key(&mut self, key: termrock::Key) -> Option<SafetyDialogAction> {
        match self.focus {
            SafetyFocus::Input => {
                if !self.input_editing {
                    match key.code {
                        KeyCode::Enter => {
                            self.input_editing = true;
                        }
                        KeyCode::Tab | KeyCode::Right | KeyCode::Down => {
                            self.focus = SafetyFocus::Cancel;
                        }
                        KeyCode::Esc => {
                            return Some(SafetyDialogAction::Cancel);
                        }
                        KeyCode::Char(c) => {
                            self.input_editing = true;
                            self.input_text.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Enter => {
                            self.input_editing = false;
                            self.focus = SafetyFocus::Cancel;
                        }
                        KeyCode::Esc => {
                            self.input_editing = false;
                        }
                        KeyCode::Backspace => {
                            self.input_text.pop();
                        }
                        KeyCode::Char(c) => {
                            self.input_text.push(c);
                        }
                        _ => {}
                    }
                }
            }
            SafetyFocus::Cancel => match key.code {
                KeyCode::Esc => {
                    return Some(SafetyDialogAction::Cancel);
                }
                KeyCode::Enter => {
                    return Some(SafetyDialogAction::Cancel);
                }
                KeyCode::Left | KeyCode::BackTab if self.token.is_some() => {
                    self.focus = SafetyFocus::Input;
                }
                KeyCode::Right | KeyCode::Tab if self.armed() => {
                    self.focus = SafetyFocus::Confirm;
                }
                _ => {}
            },
            SafetyFocus::Confirm => match key.code {
                KeyCode::Esc => {
                    return Some(SafetyDialogAction::Cancel);
                }
                KeyCode::Enter => {
                    if self.armed() {
                        return Some(SafetyDialogAction::Confirm);
                    }
                }
                KeyCode::Left | KeyCode::BackTab => {
                    self.focus = SafetyFocus::Cancel;
                }
                _ => {}
            },
        }
        None
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let elevated_style = ui.surface_style().patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_bg(Role::Surface(Surface::Elevated))
                    .set_fg(Role::Fg(FgStep::Muted)),
            ),
        );
        ui.fill(area, elevated_style);

        let border_style =
            elevated_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::BorderStrong)));

        // Corners
        ui.paint_str(Rect::new(area.x, area.y, 1, 1), "╭", border_style);
        ui.paint_str(
            Rect::new(area.right().saturating_sub(1), area.y, 1, 1),
            "╮",
            border_style,
        );
        ui.paint_str(
            Rect::new(area.x, area.bottom().saturating_sub(1), 1, 1),
            "╰",
            border_style,
        );
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(1),
                area.bottom().saturating_sub(1),
                1,
                1,
            ),
            "╯",
            border_style,
        );

        // Horizontal borders
        if area.width > 2 {
            let hbar = "─".repeat(area.width as usize - 2);
            ui.paint_str(
                Rect::new(area.x + 1, area.y, area.width - 2, 1),
                &hbar,
                border_style,
            );
            ui.paint_str(
                Rect::new(
                    area.x + 1,
                    area.bottom().saturating_sub(1),
                    area.width - 2,
                    1,
                ),
                &hbar,
                border_style,
            );
        }

        // Vertical borders
        for y in (area.y + 1)..area.bottom().saturating_sub(1) {
            ui.paint_str(Rect::new(area.x, y, 1, 1), "│", border_style);
            ui.paint_str(
                Rect::new(area.right().saturating_sub(1), y, 1, 1),
                "│",
                border_style,
            );
        }

        let inner = Rect::new(
            area.x + 3,
            area.y + 2,
            area.width.saturating_sub(6),
            area.height.saturating_sub(4),
        );
        if inner.is_empty() {
            return;
        }

        // Title
        let title_style = elevated_style.patch(
            ui.paint_patch(
                &StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .add(Modifier::BOLD),
            ),
        );
        ui.paint_str(
            Rect::new(inner.x, inner.y, inner.width, 1),
            &self.title,
            title_style,
        );

        // Facts
        let actions_y = area.bottom().saturating_sub(3);
        let fixed = self.code_rows() + self.ack_rows();
        let facts_bottom = actions_y.saturating_sub(1 + fixed);
        let label_w = self.label_width();
        let vw = (inner.width as usize).saturating_sub(label_w);
        let mut y = inner.y + 2;
        let muted_style = elevated_style
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
        let primary_style = elevated_style
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        let secondary_style = elevated_style
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
        let warning_style =
            elevated_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Warning)));
        let danger_style =
            elevated_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)));

        for prop in &self.facts {
            if y >= facts_bottom {
                break;
            }
            let tone_style = match prop.tone {
                Tone::Normal => primary_style,
                Tone::Secondary => secondary_style,
                Tone::Warning => warning_style,
                Tone::Error => danger_style,
                Tone::Muted => muted_style,
            };
            let lines = if prop.wrap {
                wrap(&prop.value, vw.max(4) as u16)
            } else {
                vec![prop.value.clone()]
            };
            for (i, line) in lines.iter().enumerate() {
                if y >= facts_bottom {
                    break;
                }
                if i == 0 {
                    ui.paint_str(
                        Rect::new(inner.x, y, label_w as u16, 1),
                        &prop.label,
                        muted_style,
                    );
                }
                ui.paint_str(
                    Rect::new(
                        inner.x + label_w as u16,
                        y,
                        inner.width.saturating_sub(label_w as u16),
                        1,
                    ),
                    line,
                    tone_style,
                );
                y += 1;
            }
        }

        // Code
        if !self.code.is_empty() {
            y += 1;
            for line in self.code.iter().take(6) {
                ui.paint_str(Rect::new(inner.x, y, inner.width, 1), line, secondary_style);
                y += 1;
            }
        }

        // Token Input
        if let Some(tok) = &self.token {
            y += 1;
            let is_focused = self.focus == SafetyFocus::Input;
            let label_style = if is_focused {
                title_style
            } else {
                secondary_style
            };
            let label_raw = format!("Type {tok} to confirm");
            let label_w = (inner.width.saturating_sub(1) as usize).max(label_raw.len());
            let label_text = format!("{label_raw:<label_w$}");
            ui.paint_str(
                Rect::new(inner.x + 1, y, inner.width.saturating_sub(1), 1),
                &label_text,
                label_style,
            );

            // Field row
            let field_rect = Rect::new(area.x + 2, y + 1, area.width.saturating_sub(5), 1);
            ui.register_control(SAFETY_INPUT, field_rect, Focusability::Focusable);
            let field_style = ui
                .surface_style()
                .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(Surface::Field))));
            ui.fill(field_rect, field_style);

            if is_focused {
                let accent_gutter =
                    field_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
                ui.paint_str(Rect::new(field_rect.x, y + 1, 1, 1), "▎", accent_gutter);
            }
            if !self.input_text.is_empty() {
                let text_style = field_style
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
                ui.paint_str(
                    Rect::new(
                        field_rect.x + 2,
                        y + 1,
                        field_rect.width.saturating_sub(2),
                        1,
                    ),
                    &self.input_text,
                    text_style,
                );
            }
        }

        // Actions row
        let cancel_w = 8u16;
        let confirm_w = (self.confirm_label.len() + 2) as u16;
        let confirm_x = area.right().saturating_sub(3 + confirm_w);
        let cancel_x = confirm_x.saturating_sub(1 + cancel_w);

        // Cancel button
        let overlay_style = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_bg(Role::Surface(Surface::Overlay))));
        let cancel_btn_style = overlay_style
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        let cancel_rect = Rect::new(cancel_x, actions_y, cancel_w, 1);
        ui.register_control(SAFETY_CANCEL, cancel_rect, Focusability::Focusable);
        ui.fill(cancel_rect, cancel_btn_style);
        if self.focus == SafetyFocus::Cancel {
            let gutter_style =
                cancel_btn_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
            let text_style =
                cancel_btn_style.patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
            ui.paint_str(Rect::new(cancel_x, actions_y, 1, 1), "▎", gutter_style);
            ui.paint_str(
                Rect::new(cancel_x + 1, actions_y, cancel_w - 1, 1),
                "Cancel ",
                text_style,
            );
        } else {
            let gutter_style = cancel_btn_style
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(Surface::Overlay))));
            ui.paint_str(Rect::new(cancel_x, actions_y, 1, 1), " ", gutter_style);
            ui.paint_str(
                Rect::new(cancel_x + 1, actions_y, cancel_w - 1, 1),
                "Cancel ",
                cancel_btn_style,
            );
        }

        // Confirm button
        let confirm_rect = Rect::new(confirm_x, actions_y, confirm_w, 1);
        ui.register_control(SAFETY_CONFIRM, confirm_rect, Focusability::Focusable);
        let armed = self.armed();
        if !armed {
            let disabled_style =
                overlay_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::DisabledFg)));
            ui.fill(confirm_rect, disabled_style);
            let gutter_style = disabled_style
                .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Surface(Surface::Overlay))));
            ui.paint_str(Rect::new(confirm_x, actions_y, 1, 1), " ", gutter_style);
            let text = format!("{} ", self.confirm_label);
            ui.paint_str(
                Rect::new(confirm_x + 1, actions_y, confirm_w - 1, 1),
                &text,
                disabled_style,
            );
        } else if self.confirm_danger {
            let danger_btn_style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_bg(Role::Danger)
                        .set_fg(Role::OnDanger),
                ),
            );
            ui.fill(confirm_rect, danger_btn_style);
            if self.focus == SafetyFocus::Confirm {
                let gutter_style = danger_btn_style
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
                ui.paint_str(Rect::new(confirm_x, actions_y, 1, 1), "▎", gutter_style);
                let text = format!("{} ", self.confirm_label);
                let text_style =
                    danger_btn_style.patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
                ui.paint_str(
                    Rect::new(confirm_x + 1, actions_y, confirm_w - 1, 1),
                    &text,
                    text_style,
                );
            } else {
                let gutter_style =
                    danger_btn_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Danger)));
                ui.paint_str(Rect::new(confirm_x, actions_y, 1, 1), " ", gutter_style);
                let text = format!("{} ", self.confirm_label);
                ui.paint_str(
                    Rect::new(confirm_x + 1, actions_y, confirm_w - 1, 1),
                    &text,
                    danger_btn_style,
                );
            }
        } else {
            let accent_btn_style = ui.surface_style().patch(
                ui.paint_patch(
                    &StylePatch::new()
                        .set_bg(Role::Accent)
                        .set_fg(Role::OnAccent),
                ),
            );
            ui.fill(confirm_rect, accent_btn_style);
            if self.focus == SafetyFocus::Confirm {
                let gutter_style = accent_btn_style
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
                ui.paint_str(Rect::new(confirm_x, actions_y, 1, 1), "▎", gutter_style);
                let text = format!("{} ", self.confirm_label);
                let text_style =
                    accent_btn_style.patch(ui.paint_patch(&StylePatch::new().add(Modifier::BOLD)));
                ui.paint_str(
                    Rect::new(confirm_x + 1, actions_y, confirm_w - 1, 1),
                    &text,
                    text_style,
                );
            } else {
                let gutter_style =
                    accent_btn_style.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
                ui.paint_str(Rect::new(confirm_x, actions_y, 1, 1), " ", gutter_style);
                let text = format!("{} ", self.confirm_label);
                ui.paint_str(
                    Rect::new(confirm_x + 1, actions_y, confirm_w - 1, 1),
                    &text,
                    accent_btn_style,
                );
            }
        }
    }
}
