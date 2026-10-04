//! Selectable, scrollable text projection component.
//!
//! Provides the canonical [`TextViewport`], [`ViewportMode`], [`ViewportState`],
//! and [`ViewportAction`] supporting Prose and Log recipes, wrapping, selection,
//! copying, scrollbar interaction, and optional tail following.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::event::MouseKind;
use crate::termrock::identity::{Id, ItemKey};
use crate::termrock::layout::{Axis, Constraints, Position, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::{ScrollRegion, ScrollState};
use crate::termrock::text::{StyleSpan, TextPosition, TextRange, TextSource, width};
use crate::termrock::theme::{Role, StylePatch, Surface};

/// Visual mode for the text viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportMode {
    /// Prose recipe: manual scrolling, `End` is a jump, no auto-follow.
    #[default]
    Prose,
    /// Log recipe: follows tail while at end, pauses on scroll away.
    Log,
}

/// Durable state for the text viewport.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportState {
    pub scroll: ScrollState,
    pub cursor: Option<TextPosition>,
    pub selection: Option<TextRange>,
    pub follow_tail: bool,
    pub last_area: Rect,
    pub drag_anchor: Option<TextPosition>,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::default(),
            cursor: None,
            selection: None,
            follow_tail: false,
            last_area: Rect::zero(),
            drag_anchor: None,
        }
    }

    pub fn with_follow_tail(mut self, follow: bool) -> Self {
        self.follow_tail = follow;
        self.scroll.set_follow(follow);
        self
    }

    pub fn set_follow_tail(&mut self, follow: bool) {
        self.follow_tail = follow;
        self.scroll.set_follow(follow);
    }
}

/// Typed actions emitted by TextViewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportAction {
    CopyRequested { text: String },
    SelectionChanged(Option<TextRange>),
}

/// Visual row segment generated during projection.
#[derive(Debug, Clone)]
struct VisualRow<'a> {
    _line_idx: usize,
    key: ItemKey,
    text: &'a str,
    byte_offset: usize,
    spans: &'a [StyleSpan],
}

/// Selectable, scrollable text projection over caller-owned text source.
#[derive(Clone)]
pub struct TextViewport<'a> {
    pub id: Id,
    pub source: &'a dyn TextSource,
    pub mode: ViewportMode,
    pub wrap: bool,
    pub patch: Option<StylePatch>,
}

impl<'a> TextViewport<'a> {
    pub fn new(id: Id, source: &'a dyn TextSource) -> Self {
        Self {
            id,
            source,
            mode: ViewportMode::Prose,
            wrap: false,
            patch: None,
        }
    }

    pub fn mode(mut self, mode: ViewportMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Extract selected source text for a range.
    pub fn extract_selection(&self, range: &TextRange) -> String {
        let count = self.source.line_count();
        if count == 0 {
            return String::new();
        }

        let mut start_idx = None;
        let mut end_idx = None;

        for i in 0..count {
            if let Some(l) = self.source.line(i) {
                if l.key == range.start.line && start_idx.is_none() {
                    start_idx = Some(i);
                }
                if l.key == range.end.line && end_idx.is_none() {
                    end_idx = Some(i);
                }
            }
        }

        let (mut s_idx, mut s_byte, mut e_idx, mut e_byte) = match (start_idx, end_idx) {
            (Some(s), Some(e)) => (s, range.start.byte, e, range.end.byte),
            _ => return String::new(),
        };

        if s_idx > e_idx || (s_idx == e_idx && s_byte > e_byte) {
            std::mem::swap(&mut s_idx, &mut e_idx);
            std::mem::swap(&mut s_byte, &mut e_byte);
        }

        let mut lines = Vec::new();
        for i in s_idx..=e_idx {
            if let Some(line) = self.source.line(i) {
                let text = line.text;
                let len = text.len();
                if s_idx == e_idx {
                    let sb = s_byte.min(len);
                    let eb = e_byte.min(len);
                    lines.push(text[sb..eb].to_string());
                } else if i == s_idx {
                    let sb = s_byte.min(len);
                    lines.push(text[sb..].to_string());
                } else if i == e_idx {
                    let eb = e_byte.min(len);
                    lines.push(text[..eb].to_string());
                } else {
                    lines.push(text.to_string());
                }
            }
        }
        lines.join("\n")
    }

    fn compute_visual_rows(&self, max_width: usize) -> Vec<VisualRow<'a>> {
        let count = self.source.line_count();
        let mut rows = Vec::new();

        for i in 0..count {
            if let Some(line) = self.source.line(i) {
                if !self.wrap || max_width == 0 {
                    rows.push(VisualRow {
                        _line_idx: i,
                        key: line.key,
                        text: line.text,
                        byte_offset: 0,
                        spans: line.spans,
                    });
                } else if line.text.is_empty() {
                    rows.push(VisualRow {
                        _line_idx: i,
                        key: line.key,
                        text: "",
                        byte_offset: 0,
                        spans: line.spans,
                    });
                } else {
                    let mut current_start = 0;
                    let mut current_w = 0;
                    for (byte_idx, g) in line.text.grapheme_indices(true) {
                        let gw = width(g);
                        if current_w + gw > max_width && current_w > 0 {
                            rows.push(VisualRow {
                                _line_idx: i,
                                key: line.key,
                                text: &line.text[current_start..byte_idx],
                                byte_offset: current_start,
                                spans: line.spans,
                            });
                            current_start = byte_idx;
                            current_w = 0;
                        }
                        current_w += gw;
                    }
                    if current_start < line.text.len() || current_w > 0 {
                        rows.push(VisualRow {
                            _line_idx: i,
                            key: line.key,
                            text: &line.text[current_start..],
                            byte_offset: current_start,
                            spans: line.spans,
                        });
                    }
                }
            }
        }
        rows
    }

    /// Update viewport state from input events.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut ViewportState) -> Response<ViewportAction> {
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.id);

        let area = cx
            .published_geometry
            .and_then(|g| g.get(&self.id))
            .copied()
            .unwrap_or(state.last_area);

        let content_w = area.width.saturating_sub(1) as usize;
        let rows = self.compute_visual_rows(content_w);
        let total_rows = rows.len();
        state.scroll.total = total_rows;
        state.scroll.viewport = area.height as usize;

        if state.follow_tail && self.mode == ViewportMode::Log {
            state.scroll.offset = state.scroll.max_offset();
        } else {
            state.scroll.clamp();
        }

        // Delegate scrollbar hit/drag to shared ScrollRegion for mouse events
        if matches!(cx.cause(), UpdateCause::Input(Input::Mouse(_), _)) {
            let scroll_region = ScrollRegion::new(
                self.id.clone(),
                Axis::Vertical,
                total_rows,
                state.scroll.viewport,
            );
            let scroll_resp = scroll_region.update(cx, &mut state.scroll);
            if scroll_resp.action.is_some() {
                if self.mode == ViewportMode::Log {
                    state.follow_tail = state.scroll.offset == state.scroll.max_offset();
                }
                cx.request_invalidate(Invalidate::Paint);
                return Response::consumed(self.id.clone());
            }
        }

        if let UpdateCause::Input(Input::Key(k), _) = cx.cause() {
            if is_target {
                match k.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let changed = state.scroll.scroll_up(1);
                        if changed && self.mode == ViewportMode::Log {
                            state.follow_tail = false;
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let changed = state.scroll.scroll_down(1);
                        if changed && self.mode == ViewportMode::Log {
                            state.follow_tail = state.scroll.offset == state.scroll.max_offset();
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageUp => {
                        let delta = state.scroll.viewport.max(1);
                        let changed = state.scroll.scroll_up(delta);
                        if changed && self.mode == ViewportMode::Log {
                            state.follow_tail = false;
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageDown => {
                        let delta = state.scroll.viewport.max(1);
                        let changed = state.scroll.scroll_down(delta);
                        if changed && self.mode == ViewportMode::Log {
                            state.follow_tail = state.scroll.offset == state.scroll.max_offset();
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Home | KeyCode::Char('g') => {
                        let changed = state.scroll.scroll_to(0);
                        if changed && self.mode == ViewportMode::Log {
                            state.follow_tail = false;
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::End | KeyCode::Char('G') => {
                        let changed = state.scroll.scroll_to(state.scroll.max_offset());
                        if self.mode == ViewportMode::Log {
                            state.follow_tail = true;
                        }
                        if changed {
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('f') if self.mode == ViewportMode::Log => {
                        state.follow_tail = !state.follow_tail;
                        if state.follow_tail {
                            state.scroll.scroll_to(state.scroll.max_offset());
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('y') => {
                        if let Some(ref sel) = state.selection {
                            let text = self.extract_selection(sel);
                            return Response::action(
                                self.id.clone(),
                                ViewportAction::CopyRequested { text },
                            )
                            .with_flow(Flow::Consumed);
                        }
                    }
                    KeyCode::Char('c') if k.mods.contains(KeyModifiers::CONTROL) => {
                        if let Some(ref sel) = state.selection {
                            let text = self.extract_selection(sel);
                            return Response::action(
                                self.id.clone(),
                                ViewportAction::CopyRequested { text },
                            )
                            .with_flow(Flow::Consumed);
                        }
                    }
                    KeyCode::Esc if state.selection.is_some() => {
                        state.selection = None;
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::action(
                            self.id.clone(),
                            ViewportAction::SelectionChanged(None),
                        )
                        .with_flow(Flow::Consumed);
                    }
                    _ => {}
                }
            }
        } else if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
            match m.kind {
                MouseKind::Down if is_target || cx.contains_point(&self.id, m.pos.into()) => {
                    let rel_y = m.pos.y.saturating_sub(area.y) as usize;
                    let v_idx = state.scroll.offset.saturating_add(rel_y);
                    if let Some(row) = rows.get(v_idx) {
                        let rel_x = m.pos.x.saturating_sub(area.x) as usize;
                        let byte = self.find_byte_offset(row.text, rel_x);
                        let pos = TextPosition::new(row.key, row.byte_offset + byte);
                        state.drag_anchor = Some(pos);
                        state.cursor = Some(pos);
                        cx.capture_pointer(self.id.clone());
                        return Response::consumed(self.id.clone());
                    }
                }
                MouseKind::Drag if cx.pointer_capture.as_ref() == Some(&self.id) => {
                    let rel_y = m.pos.y.saturating_sub(area.y) as usize;
                    let v_idx = state.scroll.offset.saturating_add(rel_y);
                    let target_v = v_idx.min(total_rows.saturating_sub(1));
                    if let Some(row) = rows.get(target_v) {
                        let rel_x = m.pos.x.saturating_sub(area.x) as usize;
                        let byte = self.find_byte_offset(row.text, rel_x);
                        let pos = TextPosition::new(row.key, row.byte_offset + byte);
                        if let Some(anchor) = state.drag_anchor {
                            state.selection =
                                Some(TextRange::new(self.source.revision(), anchor, pos));
                            state.cursor = Some(pos);
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::action(
                                self.id.clone(),
                                ViewportAction::SelectionChanged(state.selection.clone()),
                            )
                            .with_flow(Flow::Consumed);
                        }
                    }
                    return Response::consumed(self.id.clone());
                }
                MouseKind::Up if cx.pointer_capture.as_ref() == Some(&self.id) => {
                    cx.release_capture();
                    state.drag_anchor = None;
                    return Response::consumed(self.id.clone());
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

    fn find_byte_offset(&self, text: &str, target_width: usize) -> usize {
        let mut curr_w = 0;
        for (byte_idx, g) in text.grapheme_indices(true) {
            let gw = width(g);
            if curr_w + gw > target_width {
                return byte_idx;
            }
            curr_w += gw;
        }
        text.len()
    }

    /// Measure required size bounded by constraints.
    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        let count = self.source.line_count();
        let needed = Size::new(constraints.min.width, count as u16);
        constraints.clamp(needed)
    }

    /// Draw text projection, selection, and scrollbar.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ViewportState) -> Rect {
        if area.is_empty() {
            return area;
        }

        ui.register_hit(self.id.clone(), area);

        let content_w = area.width.saturating_sub(1) as usize;
        let rows = self.compute_visual_rows(content_w);
        let total_rows = rows.len();

        let mut scroll = state.scroll;
        scroll.total = total_rows;
        scroll.viewport = area.height as usize;
        if state.follow_tail && self.mode == ViewportMode::Log {
            scroll.offset = scroll.max_offset();
        } else {
            scroll.clamp();
        }

        let bg_color = match ui.current_surface {
            Surface::Elevated => ui.theme.tokens.surface_elevated,
            Surface::Canvas => ui.theme.tokens.canvas,
            _ => ui.theme.tokens.surface,
        };
        let bg_style = ratatui::style::Style::new().bg(bg_color);
        ui.fill_rect(area, bg_style);

        let visible = scroll.visible_range();
        for (idx, r_idx) in visible.clone().enumerate() {
            if let Some(row) = rows.get(r_idx) {
                let y = area.y.saturating_add(idx as u16);
                let mut x = area.x;

                for (byte_in_row, g) in row.text.grapheme_indices(true) {
                    let abs_byte = row.byte_offset + byte_in_row;
                    let is_selected = self.is_byte_selected(state, row.key, abs_byte);

                    let style = if is_selected {
                        ui.theme.resolve_style(Role::Selection, ui.current_surface)
                    } else {
                        self.resolve_span_style(ui, row.spans, abs_byte)
                    };

                    ui.set_string(x, y, g, style);
                    x = x.saturating_add(width(g) as u16);
                }
            }
        }

        // Draw hardware cursor intent if cursor is visible
        if let Some(cursor) = state.cursor {
            for (idx, r_idx) in visible.enumerate() {
                if let Some(row) = rows.get(r_idx)
                    && row.key == cursor.line
                    && cursor.byte >= row.byte_offset
                    && cursor.byte <= row.byte_offset + row.text.len()
                {
                    let prefix = &row.text[..(cursor.byte - row.byte_offset).min(row.text.len())];
                    let cx = area.x.saturating_add(width(prefix) as u16);
                    let cy = area.y.saturating_add(idx as u16);
                    ui.request_cursor(self.id.clone(), Position::new(cx, cy));
                    break;
                }
            }
        }

        // Shared Scrollbar integration
        let region = ScrollRegion::new(
            self.id.clone(),
            Axis::Vertical,
            total_rows,
            area.height as usize,
        );
        region.draw(ui, area, &scroll);

        area
    }

    fn is_byte_selected(&self, state: &ViewportState, line: ItemKey, byte: usize) -> bool {
        let sel = match state.selection {
            Some(ref s) => s,
            None => return false,
        };

        let mut start = sel.start;
        let mut end = sel.end;

        if start.line == end.line {
            if start.line != line {
                return false;
            }
            let (min_b, max_b) = (start.byte.min(end.byte), start.byte.max(end.byte));
            return byte >= min_b && byte < max_b;
        }

        let count = self.source.line_count();
        let mut start_idx = None;
        let mut end_idx = None;
        let mut current_idx = None;

        for i in 0..count {
            if let Some(l) = self.source.line(i) {
                if l.key == start.line && start_idx.is_none() {
                    start_idx = Some(i);
                }
                if l.key == end.line && end_idx.is_none() {
                    end_idx = Some(i);
                }
                if l.key == line && current_idx.is_none() {
                    current_idx = Some(i);
                }
            }
        }

        let (s_i, e_i, cur) = match (start_idx, end_idx, current_idx) {
            (Some(s), Some(e), Some(c)) => (s, e, c),
            _ => return false,
        };

        if s_i > e_i {
            std::mem::swap(&mut start, &mut end);
        }
        let (first_i, last_i) = (s_i.min(e_i), s_i.max(e_i));

        if cur < first_i || cur > last_i {
            false
        } else if cur == first_i {
            byte >= start.byte
        } else if cur == last_i {
            byte < end.byte
        } else {
            true
        }
    }

    fn resolve_span_style(
        &self,
        ui: &Ui<'_>,
        spans: &[StyleSpan],
        byte: usize,
    ) -> ratatui::style::Style {
        for span in spans {
            if byte >= span.start && byte < span.end {
                let role = match span.tone {
                    crate::termrock::theme::Tone::Primary => Role::Primary,
                    crate::termrock::theme::Tone::Accent => Role::Accent,
                    crate::termrock::theme::Tone::Success => Role::Success,
                    crate::termrock::theme::Tone::Warning => Role::Warning,
                    crate::termrock::theme::Tone::Danger => Role::Danger,
                    crate::termrock::theme::Tone::Muted => Role::TextMuted,
                    crate::termrock::theme::Tone::Normal => Role::Text,
                };
                return ui.theme.resolve_style(role, ui.current_surface);
            }
        }
        ui.theme.resolve_style(Role::Text, ui.current_surface)
    }
}
