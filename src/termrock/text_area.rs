//! Multiline controlled text editor with scrolling, line numbers, and plain/secret modes.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;

use ratatui::crossterm::event::KeyCode;
use ratatui::style::Style;
use unicode_width::UnicodeWidthStr;

use crate::core::event::{Input, MouseKind};
use crate::termrock::field::Field;
use crate::termrock::identity::{Id, Revision};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::response::{Flow, Invalidate, Response, UpdateCause, VisualState};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::secret::{Plain, Secret, SecretText, TextAction, ValidationMessage};
use crate::termrock::text::{TextEditorCore, width};
use crate::termrock::text_input::{ConflictPolicy, ConflictResolution, EditPhase, TextMode};
use crate::termrock::theme::StylePatch;

/// Wrapping mode for multiline text display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    #[default]
    None,
    Word,
    Character,
}

/// Durable caller-owned state for a multiline text area component.
pub struct TextAreaState<Mode: TextMode = Plain> {
    pub(crate) phase: EditPhase,
    pub(crate) core: TextEditorCore,
    pub(crate) snapshot: String,
    pub(crate) last_revision: Option<Revision>,
    pub(crate) scroll_top: usize,
    pub(crate) scroll_left: usize,
    pub(crate) follow_caret: bool,
    pub(crate) conflict: bool,
    pub(crate) error: Option<ValidationMessage>,
    pub(crate) was_focused: bool,
    pub(crate) _mode: PhantomData<Mode>,
}

impl<Mode: TextMode> Default for TextAreaState<Mode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Mode: TextMode> TextAreaState<Mode> {
    pub fn new() -> Self {
        Self {
            phase: EditPhase::Navigation,
            core: TextEditorCore::new().with_multiline(true),
            snapshot: String::new(),
            last_revision: None,
            scroll_top: 0,
            scroll_left: 0,
            follow_caret: true,
            conflict: false,
            error: None,
            was_focused: false,
            _mode: PhantomData,
        }
    }

    pub fn phase(&self) -> EditPhase {
        self.phase
    }

    pub fn is_editing(&self) -> bool {
        self.phase == EditPhase::Editing
    }

    pub fn caret(&self) -> usize {
        self.core.caret()
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.core.selection()
    }

    pub fn scroll_top(&self) -> usize {
        self.scroll_top
    }

    pub fn scroll_left(&self) -> usize {
        self.scroll_left
    }

    pub fn error(&self) -> Option<&ValidationMessage> {
        self.error.as_ref()
    }

    pub fn has_conflict(&self) -> bool {
        self.conflict
    }

    pub fn resolve_conflict(&mut self, res: ConflictResolution, external: &str) {
        self.conflict = false;
        match res {
            ConflictResolution::KeepDraft => {}
            ConflictResolution::ReloadExternal => {
                self.core.set_text(external);
                self.snapshot = external.to_string();
            }
        }
    }

    pub fn reset(&mut self, text: &str) {
        self.core.set_text(text);
        self.snapshot = text.to_string();
        self.phase = EditPhase::Navigation;
    }
}

impl TextAreaState<Plain> {
    /// Safe inspection of active plain text draft.
    pub fn draft(&self) -> &str {
        self.core.text()
    }

    /// Alias for draft inspection.
    pub fn text(&self) -> &str {
        self.core.text()
    }
}

impl Clone for TextAreaState<Plain> {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase,
            core: self.core.clone(),
            snapshot: self.snapshot.clone(),
            last_revision: self.last_revision,
            scroll_top: self.scroll_top,
            scroll_left: self.scroll_left,
            follow_caret: self.follow_caret,
            conflict: self.conflict,
            error: self.error.clone(),
            was_focused: self.was_focused,
            _mode: PhantomData,
        }
    }
}

impl fmt::Debug for TextAreaState<Plain> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaState<Plain>")
            .field("phase", &self.phase)
            .field("draft", &self.core.text())
            .field("caret", &self.core.caret())
            .field("scroll_top", &self.scroll_top)
            .field("scroll_left", &self.scroll_left)
            .finish()
    }
}

impl fmt::Debug for TextAreaState<SecretText> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextAreaState<SecretText>")
            .field("phase", &self.phase)
            .field("draft", &"[REDACTED]")
            .field("caret", &self.core.caret())
            .finish()
    }
}

impl<Mode: TextMode> Drop for TextAreaState<Mode> {
    fn drop(&mut self) {
        if Mode::is_secret() {
            let bytes = unsafe { self.snapshot.as_bytes_mut() };
            Secret::zeroize_buffer(bytes);
            self.snapshot.clear();
            self.core.set_text("");
        }
    }
}

/// Multiline controlled text editor.
pub struct TextArea<'a, Mode: TextMode = Plain> {
    pub id: Id,
    pub value_str: &'a str,
    pub secret: Option<&'a Secret>,
    pub revision: Revision,
    pub label: Option<&'a str>,
    pub help: Option<&'a str>,
    pub rows: u16,
    pub disabled: bool,
    pub read_only: bool,
    pub validation: Option<&'a ValidationMessage>,
    pub wrap: WrapMode,
    pub patch: StylePatch,
    pub conflict_policy: ConflictPolicy,
    pub _mode: PhantomData<Mode>,
}

impl<'a> TextArea<'a, Plain> {
    pub fn new(id: Id, value: &'a str, revision: Revision) -> Self {
        Self {
            id,
            value_str: value,
            secret: None,
            revision,
            label: None,
            help: None,
            rows: 5,
            disabled: false,
            read_only: false,
            validation: None,
            wrap: WrapMode::None,
            patch: StylePatch::default(),
            conflict_policy: ConflictPolicy::PreserveDraftAndReport,
            _mode: PhantomData,
        }
    }
}

impl<'a> TextArea<'a, SecretText> {
    pub fn secret(id: Id, value: &'a Secret, revision: Revision) -> Self {
        Self {
            id,
            value_str: "",
            secret: Some(value),
            revision,
            label: None,
            help: None,
            rows: 5,
            disabled: false,
            read_only: false,
            validation: None,
            wrap: WrapMode::None,
            patch: StylePatch::default(),
            conflict_policy: ConflictPolicy::PreserveDraftAndReport,
            _mode: PhantomData,
        }
    }
}

impl<'a, Mode: TextMode> TextArea<'a, Mode> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = if label.is_empty() { None } else { Some(label) };
        self
    }

    pub fn help(mut self, help: &'a str) -> Self {
        self.help = if help.is_empty() { None } else { Some(help) };
        self
    }

    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows.max(1);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn validation(mut self, validation: Option<&'a ValidationMessage>) -> Self {
        self.validation = validation;
        self
    }

    pub fn wrap(mut self, wrap: WrapMode) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = patch;
        self
    }

    pub fn conflict_policy(mut self, conflict_policy: ConflictPolicy) -> Self {
        self.conflict_policy = conflict_policy;
        self
    }

    /// Reconcile state with props.
    pub fn reconcile(&self, state: &mut TextAreaState<Mode>) {
        if state.last_revision != Some(self.revision) {
            if state.phase == EditPhase::Navigation {
                if !Mode::is_secret() {
                    state.core.set_text(self.value_str);
                    state.snapshot = self.value_str.to_string();
                } else if let Some(sec) = self.secret {
                    sec.expose(|s| {
                        state.core.set_text(s);
                        state.snapshot = s.to_string();
                    });
                }
                state.last_revision = Some(self.revision);
                state.conflict = false;
            } else if self.conflict_policy == ConflictPolicy::PreserveDraftAndReport {
                state.conflict = true;
            } else {
                if !Mode::is_secret() {
                    state.core.set_text(self.value_str);
                    state.snapshot = self.value_str.to_string();
                } else if let Some(sec) = self.secret {
                    sec.expose(|s| {
                        state.core.set_text(s);
                        state.snapshot = s.to_string();
                    });
                }
                state.last_revision = Some(self.revision);
            }
        }
    }

    /// Measure the text area geometry.
    pub fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let field = Field::new(self.label.unwrap_or(""))
            .help(self.help.unwrap_or(""))
            .error(self.validation);

        let child_size = Size::new(constraints.min.width.max(20), self.rows);
        field.measure(cx, child_size, constraints)
    }

    /// Process input events and update state.
    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TextAreaState<Mode>,
    ) -> Response<TextAction<Mode::Value>> {
        self.reconcile(state);

        let has_focus = cx.has_focus(&self.id);

        if self.disabled {
            state.was_focused = false;
            if has_focus {
                return Response::consumed(self.id.clone())
                    .with_state(VisualState::empty().focused(true).disabled(true));
            }
            return Response::bubble(self.id.clone())
                .with_state(VisualState::empty().disabled(true));
        }

        // Blur handling: commit on blur if editing
        if state.was_focused && !has_focus && state.phase == EditPhase::Editing {
            state.was_focused = false;
            state.phase = EditPhase::Navigation;
            let val = Mode::to_value(state.core.text());
            return Response::action(self.id.clone(), TextAction::Commit { value: val })
                .with_flow(Flow::Consumed)
                .with_invalidate(Invalidate::Paint);
        }
        state.was_focused = has_focus;

        if state.conflict {
            return Response::action(self.id.clone(), TextAction::Conflict)
                .with_flow(Flow::Consumed);
        }

        let total_lines = state.core.text().lines().count().max(1);
        let visible_rows = self.rows as usize;

        let cause = cx.cause().clone();

        match cause {
            UpdateCause::Input(Input::Mouse(m), _) => {
                let intended = cx.intended_owner() == Some(&self.id);
                match m.kind {
                    MouseKind::WheelUp => {
                        state.scroll_top = state.scroll_top.saturating_sub(3);
                        state.follow_caret = false;
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    MouseKind::WheelDown => {
                        state.scroll_top =
                            (state.scroll_top + 3).min(total_lines.saturating_sub(visible_rows));
                        state.follow_caret = false;
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    MouseKind::Up if intended => {
                        cx.request_focus(self.id.clone());
                        if !self.read_only {
                            if state.phase == EditPhase::Navigation {
                                state.snapshot = state.core.text().to_string();
                                state.phase = EditPhase::Editing;
                            }
                            // Map click to line and column
                            state.follow_caret = true;
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone())
                            .with_invalidate(Invalidate::Paint);
                    }
                    _ => {}
                }
            }
            UpdateCause::Input(Input::Paste(text), _) => {
                // In TextArea: Paste requires editing already active! Navigation-mode paste is ignored!
                if state.phase == EditPhase::Editing && !self.disabled && !self.read_only {
                    state.core.insert_str(&text);
                    state.follow_caret = true;
                    self.ensure_caret_visible(state, visible_rows);
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::action(self.id.clone(), TextAction::Edited)
                        .with_flow(Flow::Consumed)
                        .with_invalidate(Invalidate::Paint);
                }
            }
            UpdateCause::Input(Input::Key(k), _) if has_focus => {
                if state.phase == EditPhase::Navigation {
                    match k.code {
                        KeyCode::Enter | KeyCode::F(2) => {
                            if !self.read_only {
                                state.snapshot = state.core.text().to_string();
                                state.phase = EditPhase::Editing;
                                state.follow_caret = true;
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone())
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') if k.plain() => {
                            state.scroll_top = state.scroll_top.saturating_sub(1);
                            state.follow_caret = false;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Down | KeyCode::Char('j') if k.plain() => {
                            if state.scroll_top + 1 < total_lines {
                                state.scroll_top += 1;
                            }
                            state.follow_caret = false;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Left | KeyCode::Char('h') if k.plain() => {
                            state.scroll_left = state.scroll_left.saturating_sub(1);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Right | KeyCode::Char('l') if k.plain() => {
                            state.scroll_left += 1;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::PageUp => {
                            state.scroll_top = state.scroll_top.saturating_sub(visible_rows);
                            state.follow_caret = false;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::PageDown => {
                            state.scroll_top = (state.scroll_top + visible_rows)
                                .min(total_lines.saturating_sub(1));
                            state.follow_caret = false;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Home => {
                            state.scroll_left = 0;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        _ => {}
                    }
                } else if state.phase == EditPhase::Editing {
                    match k.code {
                        KeyCode::Esc => {
                            // INTENTIONAL TEXTAREA EXCEPTION:
                            // Escape finishes and COMMITS editing in TextArea!
                            state.phase = EditPhase::Navigation;
                            let val = Mode::to_value(state.core.text());
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                TextAction::Commit { value: val },
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Enter => {
                            // Enter in TextArea editing inserts a newline!
                            if !self.read_only {
                                state.core.insert_char('\n');
                                state.follow_caret = true;
                                self.ensure_caret_visible(state, visible_rows);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), TextAction::Edited)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                        KeyCode::Tab => {
                            // Tab / Shift+Tab commits and traverses!
                            state.phase = EditPhase::Navigation;
                            let val = Mode::to_value(state.core.text());
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                TextAction::Commit { value: val },
                            )
                            .with_flow(Flow::Consumed)
                            .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Backspace => {
                            if !self.read_only && state.core.delete_backward() {
                                state.follow_caret = true;
                                self.ensure_caret_visible(state, visible_rows);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), TextAction::Edited)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                        KeyCode::Delete => {
                            if !self.read_only && state.core.delete_forward() {
                                state.follow_caret = true;
                                self.ensure_caret_visible(state, visible_rows);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), TextAction::Edited)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                        }
                        KeyCode::Left => {
                            state.core.move_left(k.shift());
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Right => {
                            state.core.move_right(k.shift());
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Up => {
                            self.move_caret_vertical(state, -1);
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Down => {
                            self.move_caret_vertical(state, 1);
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::PageUp => {
                            self.move_caret_vertical(state, -(visible_rows as isize));
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::PageDown => {
                            self.move_caret_vertical(state, visible_rows as isize);
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Home => {
                            state.core.move_home(k.shift());
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::End => {
                            state.core.move_end(k.shift());
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone())
                                .with_invalidate(Invalidate::Paint);
                        }
                        KeyCode::Char(c) if k.ctrl() => match c {
                            'a' => {
                                state.core.select_all();
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone())
                                    .with_invalidate(Invalidate::Paint);
                            }
                            'z' => {
                                if !self.read_only && state.core.undo() {
                                    state.follow_caret = true;
                                    self.ensure_caret_visible(state, visible_rows);
                                    cx.request_invalidate(Invalidate::Paint);
                                    return Response::action(self.id.clone(), TextAction::Edited)
                                        .with_flow(Flow::Consumed)
                                        .with_invalidate(Invalidate::Paint);
                                }
                            }
                            'y' if !self.read_only && state.core.redo() => {
                                state.follow_caret = true;
                                self.ensure_caret_visible(state, visible_rows);
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), TextAction::Edited)
                                    .with_flow(Flow::Consumed)
                                    .with_invalidate(Invalidate::Paint);
                            }
                            _ => {}
                        },
                        KeyCode::Char(c) if !k.ctrl() && !k.alt() && !self.read_only => {
                            state.core.insert_char(c);
                            state.follow_caret = true;
                            self.ensure_caret_visible(state, visible_rows);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), TextAction::Edited)
                                .with_flow(Flow::Consumed)
                                .with_invalidate(Invalidate::Paint);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        Response::bubble(self.id.clone()).with_state(VisualState::empty().focused(has_focus))
    }

    fn ensure_caret_visible(&self, state: &mut TextAreaState<Mode>, visible_rows: usize) {
        if !state.follow_caret {
            return;
        }
        let (caret_line, _caret_col) = self.caret_to_line_col(state);
        if caret_line < state.scroll_top {
            state.scroll_top = caret_line;
        } else if caret_line >= state.scroll_top + visible_rows {
            state.scroll_top = caret_line.saturating_sub(visible_rows.saturating_sub(1));
        }
    }

    fn caret_to_line_col(&self, state: &TextAreaState<Mode>) -> (usize, usize) {
        let text = state.core.text();
        let caret = state.core.caret().min(text.len());
        let mut line = 0;
        let mut line_start = 0;
        for (i, b) in text[..caret].bytes().enumerate() {
            if b == b'\n' {
                line += 1;
                line_start = i + 1;
            }
        }
        let col = width(&text[line_start..caret]);
        (line, col)
    }

    fn move_caret_vertical(&self, state: &mut TextAreaState<Mode>, delta: isize) {
        let text = state.core.text();
        let (cur_line, cur_col) = self.caret_to_line_col(state);
        let target_line = if delta < 0 {
            cur_line.saturating_sub((-delta) as usize)
        } else {
            cur_line + (delta as usize)
        };

        let lines: Vec<&str> = text.split('\n').collect();
        if target_line >= lines.len() {
            state.core.set_caret(text.len());
            return;
        }

        let mut byte_offset = 0;
        for (i, l) in lines.iter().enumerate() {
            if i == target_line {
                // Walk columns in this line
                let mut col = 0;
                let mut char_offset = 0;
                for c in l.chars() {
                    let cw = c.len_utf8();
                    let w = UnicodeWidthStr::width(&l[char_offset..char_offset + cw]);
                    if col + w > cur_col {
                        break;
                    }
                    col += w;
                    char_offset += cw;
                }
                state.core.set_caret(byte_offset + char_offset);
                return;
            }
            byte_offset += l.len() + 1; // + 1 for \n
        }
    }

    /// Render the text area component.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TextAreaState<Mode>) -> Rect {
        if area.is_empty() {
            return Rect::zero();
        }

        let theme = ui.theme;
        let has_focus = ui.is_focused(&self.id);

        let err = self.validation.or(state.error.as_ref());
        let field = Field::new(self.label.unwrap_or(""))
            .help(self.help.unwrap_or(""))
            .error(err);

        field.draw(ui, area, |ui, child_area| {
            if child_area.is_empty() {
                return;
            }

            let bg_color = if self.disabled {
                theme.tokens.surface
            } else if state.phase == EditPhase::Editing {
                theme.tokens.field
            } else {
                theme.tokens.surface
            };

            ui.fill_rect(child_area, Style::new().bg(bg_color));

            // Focus gutter (column 0)
            let gutter_sym = if has_focus { "▎" } else { " " };
            let gutter_style = if has_focus {
                Style::new().fg(theme.tokens.focus).bg(bg_color)
            } else {
                Style::new().fg(theme.tokens.text_muted).bg(bg_color)
            };
            for y in 0..child_area.height {
                ui.set_string(
                    child_area.x,
                    child_area.y.saturating_add(y),
                    gutter_sym,
                    gutter_style,
                );
            }

            let text_x = child_area.x.saturating_add(2); // 2-cell inset
            let avail_w = child_area.width.saturating_sub(3) as usize; // reserve 1 column for scrollbar
            let body_h = child_area.height.saturating_sub(1); // reserve bottom line for footer info

            let text = state.core.text();
            let total_lines = text.as_bytes().iter().filter(|&&b| b == b'\n').count() + 1;
            let total_lines = total_lines.max(1);

            let (caret_line, caret_col) = self.caret_to_line_col(state);

            // Draw visible lines
            for (row, l) in text
                .split('\n')
                .skip(state.scroll_top)
                .take(body_h as usize)
                .enumerate()
            {
                let line_idx = state.scroll_top + row;
                let row_y = child_area.y.saturating_add(row as u16);
                let text_style = if self.disabled {
                    Style::new().fg(theme.tokens.text_muted).bg(bg_color)
                } else if has_focus && state.phase == EditPhase::Editing && line_idx == caret_line {
                    Style::new()
                        .fg(theme.tokens.text_primary)
                        .bg(bg_color)
                        .underlined()
                } else {
                    Style::new().fg(theme.tokens.text_primary).bg(bg_color)
                };

                let line_display = if state.scroll_left > 0 {
                    if width(l) > state.scroll_left {
                        &l[state.scroll_left..]
                    } else {
                        ""
                    }
                } else {
                    l
                };
                ui.set_string(text_x, row_y, line_display, text_style);
            }

            // Scrollbar on the right if needed
            if total_lines > (body_h as usize) {
                let sb_x = child_area
                    .x
                    .saturating_add(child_area.width.saturating_sub(1));
                let thumb_pos = (state.scroll_top * (body_h as usize)) / total_lines;
                for row in 0..body_h {
                    let sb_y = child_area.y.saturating_add(row);
                    let sym = if (row as usize) == thumb_pos {
                        "█"
                    } else {
                        "│"
                    };
                    ui.set_string(
                        sb_x,
                        sb_y,
                        sym,
                        Style::new().fg(theme.tokens.border_subtle).bg(bg_color),
                    );
                }
            }

            // Footer line numbers (e.g. "ln 3/12")
            if child_area.height > 1 {
                let footer_y = child_area
                    .y
                    .saturating_add(child_area.height.saturating_sub(1));
                let info = format!("ln {}/{}", caret_line + 1, total_lines);
                let info_w = info.len() as u16;
                let info_x = child_area
                    .x
                    .saturating_add(child_area.width.saturating_sub(info_w + 1));
                ui.set_string(
                    info_x,
                    footer_y,
                    &info,
                    Style::new().fg(theme.tokens.text_muted).bg(bg_color),
                );
            }

            // Caret cursor if editing and focused
            if has_focus
                && state.phase == EditPhase::Editing
                && caret_line >= state.scroll_top
                && caret_line < state.scroll_top + (body_h as usize)
            {
                let row_idx = (caret_line - state.scroll_top) as u16;
                let col_offset = caret_col.saturating_sub(state.scroll_left);
                let cx_pos = text_x.saturating_add(col_offset as u16);
                if (cx_pos as usize) < (text_x as usize) + avail_w {
                    let cy_pos = child_area.y.saturating_add(row_idx);
                    ui.request_cursor(
                        self.id.clone(),
                        crate::termrock::layout::Position::new(cx_pos, cy_pos),
                    );
                }
            }

            // Register hit region and focus candidate
            ui.register_hit(self.id.clone(), child_area);
            ui.register_focus(self.id.clone(), !self.disabled);
        });

        area
    }
}
