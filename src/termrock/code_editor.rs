//! Termrock CodeEditor component: text editing with gutter, line numbers, diagnostics, and find support.
//!
//! Provides the canonical [`CodeEditor`], [`CodeEditorState`], [`CodeAction`],
//! [`Diagnostic`], and [`DiagnosticSeverity`].

#![allow(clippy::collapsible_if)]

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::MouseKind;
use crate::termrock::identity::{Id, ItemKey};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{TextEditorCore, TextPosition, TextRange, TextSource, width};
use crate::termrock::theme::{Role, StylePatch, Surface, Tone};

/// Severity level for code editor diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiagnosticSeverity {
    #[default]
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Error => "!",
            Self::Warning => "▲",
            Self::Info => "ℹ",
            Self::Hint => "·",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::Error => Tone::Danger,
            Self::Warning => Tone::Warning,
            Self::Info => Tone::Primary,
            Self::Hint => Tone::Muted,
        }
    }
}

/// Diagnostic message anchored to a line and column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

impl Diagnostic {
    pub fn new(
        line: usize,
        column: usize,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            line,
            column,
            severity,
            message: message.into(),
        }
    }

    pub fn error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::new(line, column, DiagnosticSeverity::Error, message)
    }

    pub fn warning(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::new(line, column, DiagnosticSeverity::Warning, message)
    }
}

/// Tab key behavior in editing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabBehavior {
    #[default]
    Indent,
    Leave,
}

/// Inline find query state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FindState {
    pub query: String,
    pub matches: Vec<TextRange>,
    pub current: usize,
    pub active: bool,
}

impl FindState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current: 0,
            active: true,
        }
    }

    pub fn with_query(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            matches: Vec::new(),
            current: 0,
            active: true,
        }
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = if self.current == 0 {
                self.matches.len() - 1
            } else {
                self.current - 1
            };
        }
    }
}

/// Durable state for the CodeEditor.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodeEditorState {
    pub cursor: TextPosition,
    pub selection: Option<TextRange>,
    pub scroll: ScrollState,
    pub h_offset: usize,
    pub editing: bool,
    pub find: Option<FindState>,
    pub draft: TextEditorCore,
    pub last_area: Rect,
    pub drag_anchor: Option<TextPosition>,
}

impl CodeEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: &str) -> Self {
        let mut draft = TextEditorCore::new().with_multiline(true);
        draft.set_text(text);
        Self {
            draft,
            ..Self::default()
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn cursor_position(&self) -> TextPosition {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: TextPosition) {
        self.cursor = pos;
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
}

/// Typed actions emitted by CodeEditor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeAction {
    Edited,
    Commit { text: String },
    Cancelled,
    SelectionChanged(Option<TextRange>),
    CompletionRequested { position: TextPosition },
    SegmentActivated { key: ItemKey },
}

/// Code editor component with gutter, syntax spans, diagnostics, and find support.
#[derive(Clone)]
pub struct CodeEditor<'a> {
    pub id: Id,
    pub document: &'a dyn TextSource,
    pub read_only: bool,
    pub line_numbers: bool,
    pub diagnostics: &'a [Diagnostic],
    pub tab_behavior: TabBehavior,
    pub indent_size: usize,
    pub placeholder: &'a str,
    pub patch: Option<StylePatch>,
}

impl<'a> CodeEditor<'a> {
    pub fn new(id: Id, document: &'a dyn TextSource) -> Self {
        Self {
            id,
            document,
            read_only: false,
            line_numbers: true,
            diagnostics: &[],
            tab_behavior: TabBehavior::Indent,
            indent_size: 2,
            placeholder: "",
            patch: None,
        }
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    pub fn diagnostics(mut self, diagnostics: &'a [Diagnostic]) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn tab_behavior(mut self, tab_behavior: TabBehavior) -> Self {
        self.tab_behavior = tab_behavior;
        self
    }

    pub fn indent_size(mut self, indent_size: usize) -> Self {
        self.indent_size = indent_size;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    fn gutter_width(&self, line_count: usize) -> u16 {
        if !self.line_numbers {
            return 2; // Marker column only
        }
        let digits = line_count.max(1).to_string().len() as u16;
        digits + 4 // "▎ ! nn "
    }

    /// Update code editor state from input events.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut CodeEditorState) -> Response<CodeAction> {
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.id);

        let area = cx
            .published_geometry
            .and_then(|g| g.get(&self.id))
            .copied()
            .unwrap_or(state.last_area);

        let line_count = self.document.line_count();

        // Initial cursor reconciliation
        if state.cursor.line == ItemKey::default() && line_count > 0 {
            if let Some(first) = self.document.line(0) {
                state.cursor.line = first.key;
            }
        }

        let footer_h = 1;
        let viewport_h = area.height.saturating_sub(footer_h) as usize;
        state.scroll.total = line_count;
        state.scroll.viewport = viewport_h;
        state.scroll.clamp();

        // Find current line index
        let cur_line_idx = (0..line_count)
            .find(|&i| self.document.line(i).map(|l| l.key) == Some(state.cursor.line))
            .unwrap_or(0);

        // Mouse input
        if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
            let pos: crate::termrock::layout::Position = m.pos.into();
            match m.kind {
                MouseKind::WheelUp => {
                    if state.scroll.scroll_up(3) {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                }
                MouseKind::WheelDown => {
                    if state.scroll.scroll_down(3) {
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                }
                MouseKind::Down if is_target || area.contains(pos) => {
                    cx.request_focus(self.id.clone());
                    let gutter_w = self.gutter_width(line_count);
                    let text_rect = Rect::new(
                        area.x.saturating_add(gutter_w),
                        area.y,
                        area.width.saturating_sub(gutter_w),
                        area.height.saturating_sub(footer_h),
                    );

                    if text_rect.contains(pos) {
                        let rel_y = pos.y.saturating_sub(area.y) as usize;
                        let line_idx = state.scroll.offset + rel_y;
                        if line_idx < line_count {
                            if let Some(line) = self.document.line(line_idx) {
                                state.cursor.line = line.key;
                                let rel_x = pos.x.saturating_sub(text_rect.x) as usize;
                                state.cursor.byte = rel_x.min(line.text.len());
                                if !self.read_only {
                                    state.editing = true;
                                }
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Keyboard input
        if let UpdateCause::Input(Input::Key(k), _) = cx.cause() {
            if is_target {
                // Editing mode
                if state.editing && !self.read_only {
                    match k.code {
                        KeyCode::Esc => {
                            state.editing = false;
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CodeAction::Cancelled)
                                .with_flow(Flow::Consumed);
                        }
                        KeyCode::Enter if k.mods.contains(KeyModifiers::CONTROL) => {
                            state.editing = false;
                            let text = state.draft.text().to_owned();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CodeAction::Commit { text })
                                .with_flow(Flow::Consumed);
                        }
                        KeyCode::Tab => {
                            if self.tab_behavior == TabBehavior::Leave {
                                state.editing = false;
                                let text = state.draft.text().to_owned();
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(
                                    self.id.clone(),
                                    CodeAction::Commit { text },
                                )
                                .with_flow(Flow::Consumed);
                            } else {
                                state.draft.insert_str(&" ".repeat(self.indent_size));
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::action(self.id.clone(), CodeAction::Edited)
                                    .with_flow(Flow::Consumed);
                            }
                        }
                        KeyCode::Backspace => {
                            state.draft.delete_backward();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CodeAction::Edited)
                                .with_flow(Flow::Consumed);
                        }
                        KeyCode::Delete => {
                            state.draft.delete_forward();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CodeAction::Edited)
                                .with_flow(Flow::Consumed);
                        }
                        KeyCode::Left => {
                            state.draft.move_left(false);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Right => {
                            state.draft.move_right(false);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                        KeyCode::Char(c) if !k.mods.contains(KeyModifiers::CONTROL) => {
                            state.draft.insert_char(c);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(self.id.clone(), CodeAction::Edited)
                                .with_flow(Flow::Consumed);
                        }
                        _ => {}
                    }
                }

                // Navigation mode
                match k.code {
                    KeyCode::Char('i') | KeyCode::Enter if !self.read_only => {
                        state.editing = true;
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('a') if !self.read_only => {
                        state.editing = true;
                        state.draft.move_right(false);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if cur_line_idx > 0 {
                            if let Some(prev) = self.document.line(cur_line_idx - 1) {
                                state.cursor.line = prev.key;
                                state.cursor.byte = state.cursor.byte.min(prev.text.len());
                                if cur_line_idx - 1 < state.scroll.offset {
                                    state.scroll.scroll_to(cur_line_idx - 1);
                                }
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone());
                            }
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cur_line_idx + 1 < line_count {
                            if let Some(next) = self.document.line(cur_line_idx + 1) {
                                state.cursor.line = next.key;
                                state.cursor.byte = state.cursor.byte.min(next.text.len());
                                if cur_line_idx + 1 >= state.scroll.offset + state.scroll.viewport {
                                    state
                                        .scroll
                                        .scroll_to(cur_line_idx + 2 - state.scroll.viewport);
                                }
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone());
                            }
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        state.cursor.byte = state.cursor.byte.saturating_sub(1);
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if let Some(line) = self.document.line(cur_line_idx) {
                            state.cursor.byte = (state.cursor.byte + 1).min(line.text.len());
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    KeyCode::Home | KeyCode::Char('0') => {
                        state.cursor.byte = 0;
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::End | KeyCode::Char('$') => {
                        if let Some(line) = self.document.line(cur_line_idx) {
                            state.cursor.byte = line.text.len();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    KeyCode::PageUp => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_up(delta);
                        if let Some(l) = self.document.line(state.scroll.offset) {
                            state.cursor.line = l.key;
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageDown => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_down(delta);
                        let target = (state.scroll.offset
                            + state.scroll.viewport.saturating_sub(1))
                        .min(line_count.saturating_sub(1));
                        if let Some(l) = self.document.line(target) {
                            state.cursor.line = l.key;
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('/') => {
                        state.find = Some(FindState::new());
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('n') => {
                        if let Some(ref mut f) = state.find {
                            f.next_match();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    KeyCode::Char('N') => {
                        if let Some(ref mut f) = state.find {
                            f.prev_match();
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        if is_target {
            Response::consumed(self.id.clone())
        } else {
            Response::bubble(self.id.clone())
        }
    }

    /// Measure minimum and preferred sizes.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let count = self.document.line_count();
        let gutter_w = self.gutter_width(count);
        let needed = Size::new(gutter_w + 40, (count as u16 + 1).max(3));
        constraints.clamp(needed)
    }

    /// Read-only draw pass.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CodeEditorState) -> Rect {
        if area.is_empty() {
            return area;
        }

        ui.register_hit(self.id.clone(), area);

        let theme = ui.theme;
        let bg_color = match ui.current_surface {
            Surface::Elevated => theme.tokens.surface_elevated,
            Surface::Canvas => theme.tokens.canvas,
            _ => theme.tokens.surface,
        };
        ui.fill_rect(area, Style::new().bg(bg_color));

        let line_count = self.document.line_count();
        let gutter_w = self.gutter_width(line_count);
        let footer_h = 1;
        let body_h = area.height.saturating_sub(footer_h) as usize;

        let cur_line_idx = (0..line_count)
            .find(|&i| self.document.line(i).map(|l| l.key) == Some(state.cursor.line))
            .unwrap_or(0);

        let visible_range = state.scroll.visible_range();

        for (slot, line_idx) in visible_range.enumerate() {
            if slot >= body_h {
                break;
            }
            let y = area.y.saturating_add(slot as u16);
            let is_current_line = line_idx == cur_line_idx;

            // Current line highlight
            if is_current_line {
                let cur_style = Style::new().bg(theme.tokens.surface_elevated);
                ui.fill_rect(Rect::new(area.x, y, area.width, 1), cur_style);
            }

            // Gutter markers and line numbers
            let focus_marker = if is_current_line { "▎" } else { " " };
            let diag_marker = self.diagnostics.iter().find(|d| d.line == line_idx + 1);

            let (d_mark, d_tone) = match diag_marker {
                Some(d) => (d.severity.marker(), d.severity.tone()),
                None => (" ", Tone::Normal),
            };

            ui.set_string(
                area.x,
                y,
                focus_marker,
                Style::new().fg(theme.tokens.accent),
            );
            ui.set_string(
                area.x + 1,
                y,
                d_mark,
                Style::new().fg(theme.tone_color(d_tone)),
            );

            if self.line_numbers {
                let num_w = gutter_w.saturating_sub(4) as usize;
                let num_str = format!("{:>width$}", line_idx + 1, width = num_w);
                let num_style = if is_current_line {
                    Style::new()
                        .fg(theme.tokens.text_primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(theme.tokens.text_muted)
                };
                ui.set_string(area.x + 2, y, &num_str, num_style);
            }

            // Text line content
            let text_x = area.x.saturating_add(gutter_w);
            let max_text_w = area.width.saturating_sub(gutter_w) as usize;

            if let Some(line) = self.document.line(line_idx) {
                let mut x = text_x;
                for (byte_idx, g) in line.text.grapheme_indices(true) {
                    if (x - text_x) as usize >= max_text_w {
                        break;
                    }

                    let mut span_style = Style::new().fg(theme.tokens.text_primary);
                    // Check spans
                    for span in line.spans {
                        if byte_idx >= span.start && byte_idx < span.end {
                            span_style = Style::new().fg(theme.tone_color(span.tone));
                        }
                    }

                    // Check selection
                    if let Some(ref sel) = state.selection {
                        if sel.start.line == line.key
                            && byte_idx >= sel.start.byte
                            && byte_idx < sel.end.byte
                        {
                            span_style = theme.resolve_style(Role::Selection, ui.current_surface);
                        }
                    }

                    ui.set_string(x, y, g, span_style);
                    x = x.saturating_add(width(g) as u16);
                }
            }
        }

        // Draw footer
        let footer_y = area.y.saturating_add(area.height.saturating_sub(footer_h));
        let footer_rect = Rect::new(area.x, footer_y, area.width, 1);
        ui.fill_rect(footer_rect, Style::new().bg(theme.tokens.surface_elevated));

        // Left footer message
        if let Some(ref find) = state.find {
            let find_text = format!("Find: {} ({} matches)", find.query, find.matches.len());
            ui.set_string(
                area.x + 1,
                footer_y,
                &find_text,
                Style::new().fg(theme.tokens.accent),
            );
        } else if let Some(diag) = self.diagnostics.iter().find(|d| d.line == cur_line_idx + 1) {
            let diag_text = format!("{} {}", diag.severity.marker(), diag.message);
            ui.set_string(
                area.x + 1,
                footer_y,
                &diag_text,
                Style::new().fg(theme.tone_color(diag.severity.tone())),
            );
        }

        // Right footer position label
        let pos_label = format!(
            "ln {}/{} · col {}",
            cur_line_idx + 1,
            line_count.max(1),
            state.cursor.byte + 1
        );
        let pos_w = pos_label.len() as u16;
        if area.width > pos_w + 2 {
            ui.set_string(
                area.x + area.width - pos_w - 1,
                footer_y,
                &pos_label,
                Style::new().fg(theme.tokens.text_muted),
            );
        }

        area
    }
}
