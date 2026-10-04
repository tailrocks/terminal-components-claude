//! Termrock DiffView presentation component: renders unified diff or side-by-side review layout.
//!
//! Provides the canonical [`DiffView`], [`DiffMode`], [`DiffLineKind`], [`DiffRow`],
//! [`DiffSource`], [`DiffViewState`], and [`DiffAction`].

#![allow(clippy::collapsible_if)]

use std::ops::Range;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use crate::core::event::MouseKind;
use crate::termrock::identity::{Id, ItemKey, Revision};
use crate::termrock::layout::{Constraints, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::ScrollState;
use crate::termrock::text::{TextRange, truncate};
use crate::termrock::theme::{StylePatch, Surface, Tone};

/// Presentation mode for DiffView.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiffMode {
    /// Unified listing with old/new gutter and diff markers.
    #[default]
    Unified,
    /// Side-by-side review mode with paired old and new columns.
    Review,
}

impl DiffMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Unified => Self::Review,
            Self::Review => Self::Unified,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unified => "unified",
            Self::Review => "review",
        }
    }
}

/// Semantic kind of a diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiffLineKind {
    #[default]
    Context,
    Add,
    Remove,
    Header,
}

impl DiffLineKind {
    pub fn marker(self) -> &'static str {
        match self {
            Self::Context => " ",
            Self::Add => "+",
            Self::Remove => "-",
            Self::Header => "@",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::Context => Tone::Normal,
            Self::Add => Tone::Success,
            Self::Remove => Tone::Danger,
            Self::Header => Tone::Primary,
        }
    }
}

/// A single row in a diff source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRow<'a> {
    pub key: ItemKey,
    pub kind: DiffLineKind,
    pub old_lineno: Option<usize>,
    pub new_lineno: Option<usize>,
    pub text: &'a str,
    pub intraline_highlights: &'a [Range<usize>],
}

impl<'a> DiffRow<'a> {
    pub fn new(key: ItemKey, kind: DiffLineKind, text: &'a str) -> Self {
        Self {
            key,
            kind,
            old_lineno: None,
            new_lineno: None,
            text,
            intraline_highlights: &[],
        }
    }

    pub fn context(key: ItemKey, old_no: usize, new_no: usize, text: &'a str) -> Self {
        Self {
            key,
            kind: DiffLineKind::Context,
            old_lineno: Some(old_no),
            new_lineno: Some(new_no),
            text,
            intraline_highlights: &[],
        }
    }

    pub fn add(key: ItemKey, new_no: usize, text: &'a str) -> Self {
        Self {
            key,
            kind: DiffLineKind::Add,
            old_lineno: None,
            new_lineno: Some(new_no),
            text,
            intraline_highlights: &[],
        }
    }

    pub fn remove(key: ItemKey, old_no: usize, text: &'a str) -> Self {
        Self {
            key,
            kind: DiffLineKind::Remove,
            old_lineno: Some(old_no),
            new_lineno: None,
            text,
            intraline_highlights: &[],
        }
    }

    pub fn header(key: ItemKey, text: &'a str) -> Self {
        Self {
            key,
            kind: DiffLineKind::Header,
            old_lineno: None,
            new_lineno: None,
            text,
            intraline_highlights: &[],
        }
    }

    pub fn with_highlights(mut self, highlights: &'a [Range<usize>]) -> Self {
        self.intraline_highlights = highlights;
        self
    }
}

/// File modification status for diff header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
}

impl DiffStatus {
    pub fn marker(&self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed { .. } => "R",
        }
    }

    pub fn tone(&self) -> Tone {
        match self {
            Self::Added => Tone::Success,
            Self::Modified => Tone::Warning,
            Self::Deleted => Tone::Danger,
            Self::Renamed { .. } => Tone::Muted,
        }
    }
}

/// Read-only source model for DiffView.
pub trait DiffSource {
    fn revision(&self) -> Revision;
    fn row_count(&self) -> usize;
    fn row(&self, index: usize) -> Option<DiffRow<'_>>;
    fn file_header(&self) -> Option<&str> {
        None
    }
    fn file_status(&self) -> Option<DiffStatus> {
        None
    }
}

/// In-memory row item for MemoryDiffSource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDiffItem {
    pub key: ItemKey,
    pub kind: DiffLineKind,
    pub old_no: Option<usize>,
    pub new_no: Option<usize>,
    pub text: String,
}

/// In-memory implementation of DiffSource.
#[derive(Debug, Clone, Default)]
pub struct MemoryDiffSource {
    pub revision: Revision,
    pub header: Option<String>,
    pub status: Option<DiffStatus>,
    pub items: Vec<MemoryDiffItem>,
}

impl MemoryDiffSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn with_status(mut self, status: DiffStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn add_context(
        &mut self,
        key: ItemKey,
        old_no: usize,
        new_no: usize,
        text: impl Into<String>,
    ) {
        self.items.push(MemoryDiffItem {
            key,
            kind: DiffLineKind::Context,
            old_no: Some(old_no),
            new_no: Some(new_no),
            text: text.into(),
        });
    }

    pub fn add_addition(&mut self, key: ItemKey, new_no: usize, text: impl Into<String>) {
        self.items.push(MemoryDiffItem {
            key,
            kind: DiffLineKind::Add,
            old_no: None,
            new_no: Some(new_no),
            text: text.into(),
        });
    }

    pub fn add_removal(&mut self, key: ItemKey, old_no: usize, text: impl Into<String>) {
        self.items.push(MemoryDiffItem {
            key,
            kind: DiffLineKind::Remove,
            old_no: Some(old_no),
            new_no: None,
            text: text.into(),
        });
    }

    pub fn add_header(&mut self, key: ItemKey, text: impl Into<String>) {
        self.items.push(MemoryDiffItem {
            key,
            kind: DiffLineKind::Header,
            old_no: None,
            new_no: None,
            text: text.into(),
        });
    }
}

impl DiffSource for MemoryDiffSource {
    fn revision(&self) -> Revision {
        self.revision
    }

    fn row_count(&self) -> usize {
        self.items.len()
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        self.items.get(index).map(|item| DiffRow {
            key: item.key,
            kind: item.kind,
            old_lineno: item.old_no,
            new_lineno: item.new_no,
            text: &item.text,
            intraline_highlights: &[],
        })
    }

    fn file_header(&self) -> Option<&str> {
        self.header.as_deref()
    }

    fn file_status(&self) -> Option<DiffStatus> {
        self.status.clone()
    }
}

/// Durable interaction state for DiffView.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffViewState {
    pub mode: DiffMode,
    pub scroll: ScrollState,
    pub cursor_row: Option<ItemKey>,
    pub selection: Option<TextRange>,
    pub last_area: Rect,
    pub drag_anchor: Option<ItemKey>,
}

impl DiffViewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mode(mut self, mode: DiffMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn toggle_mode(&mut self) -> DiffMode {
        self.mode = self.mode.toggled();
        self.mode
    }
}

/// Typed actions emitted by DiffView.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffAction {
    SelectionChanged(Option<TextRange>),
    CopyRequested { text: String },
    HunkActivated { key: ItemKey },
}

/// Read-only DiffView presentation component.
#[derive(Clone)]
pub struct DiffView<'a> {
    pub id: Id,
    pub source: &'a dyn DiffSource,
    pub mode: DiffMode,
    pub wrap: bool,
    pub patch: Option<StylePatch>,
}

impl<'a> DiffView<'a> {
    pub fn new(id: Id, source: &'a dyn DiffSource) -> Self {
        Self {
            id,
            source,
            mode: DiffMode::Unified,
            wrap: false,
            patch: None,
        }
    }

    pub fn mode(mut self, mode: DiffMode) -> Self {
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

    /// Calculate effective presentation mode based on available width.
    ///
    /// Review mode requires at least 42 cells width:
    /// 2 * 16 min text cells + 8 gutter cells + 1 separator + 1 scrollbar.
    /// When width is below this threshold, Unified mode is displayed as fallback,
    /// but durable `state.mode` remains Review so widening restores it automatically.
    pub fn effective_mode(&self, width: u16, state: &DiffViewState) -> DiffMode {
        let requested = if state.mode != DiffMode::default() {
            state.mode
        } else {
            self.mode
        };

        if requested == DiffMode::Review && width >= 42 {
            DiffMode::Review
        } else {
            DiffMode::Unified
        }
    }

    /// Update diff view state from input events.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut DiffViewState) -> Response<DiffAction> {
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.id);

        let area = cx
            .published_geometry
            .and_then(|g| g.get(&self.id))
            .copied()
            .unwrap_or(state.last_area);

        let row_count = self.source.row_count();
        let header_h = if self.source.file_header().is_some() {
            1
        } else {
            0
        };
        let body_h = area.height.saturating_sub(header_h) as usize;

        state.scroll.total = row_count;
        state.scroll.viewport = body_h;
        state.scroll.clamp();

        if state.cursor_row.is_none() && row_count > 0 {
            if let Some(first) = self.source.row(0) {
                state.cursor_row = Some(first.key);
            }
        }

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
                    let rel_y = pos.y.saturating_sub(area.y + header_h) as usize;
                    let row_idx = state.scroll.offset + rel_y;
                    if row_idx < row_count {
                        if let Some(r) = self.source.row(row_idx) {
                            state.cursor_row = Some(r.key);
                            cx.request_invalidate(Invalidate::Paint);
                            if r.kind == DiffLineKind::Header {
                                return Response::action(
                                    self.id.clone(),
                                    DiffAction::HunkActivated { key: r.key },
                                )
                                .with_flow(Flow::Consumed);
                            }
                            return Response::consumed(self.id.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        // Keyboard input
        if let UpdateCause::Input(Input::Key(k), _) = cx.cause() {
            if is_target {
                let cur_row_idx = (0..row_count)
                    .find(|&i| self.source.row(i).map(|r| r.key) == state.cursor_row)
                    .unwrap_or(0);

                match k.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if cur_row_idx > 0 {
                            if let Some(prev) = self.source.row(cur_row_idx - 1) {
                                state.cursor_row = Some(prev.key);
                                if cur_row_idx - 1 < state.scroll.offset {
                                    state.scroll.scroll_to(cur_row_idx - 1);
                                }
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone());
                            }
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cur_row_idx + 1 < row_count {
                            if let Some(next) = self.source.row(cur_row_idx + 1) {
                                state.cursor_row = Some(next.key);
                                if cur_row_idx + 1 >= state.scroll.offset + state.scroll.viewport {
                                    state
                                        .scroll
                                        .scroll_to(cur_row_idx + 2 - state.scroll.viewport);
                                }
                                cx.request_invalidate(Invalidate::Paint);
                                return Response::consumed(self.id.clone());
                            }
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageUp => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_up(delta);
                        if let Some(r) = self.source.row(state.scroll.offset) {
                            state.cursor_row = Some(r.key);
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::PageDown => {
                        let delta = state.scroll.viewport.max(1);
                        state.scroll.scroll_down(delta);
                        let target = (state.scroll.offset
                            + state.scroll.viewport.saturating_sub(1))
                        .min(row_count.saturating_sub(1));
                        if let Some(r) = self.source.row(target) {
                            state.cursor_row = Some(r.key);
                        }
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Home => {
                        if row_count > 0 {
                            state.scroll.scroll_to(0);
                            if let Some(r) = self.source.row(0) {
                                state.cursor_row = Some(r.key);
                            }
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::End => {
                        if row_count > 0 {
                            state.scroll.scroll_to(state.scroll.max_offset());
                            if let Some(r) = self.source.row(row_count - 1) {
                                state.cursor_row = Some(r.key);
                            }
                            cx.request_invalidate(Invalidate::Paint);
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Tab | KeyCode::Char('m') => {
                        state.toggle_mode();
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(r) = self.source.row(cur_row_idx) {
                            if r.kind == DiffLineKind::Header {
                                return Response::action(
                                    self.id.clone(),
                                    DiffAction::HunkActivated { key: r.key },
                                )
                                .with_flow(Flow::Consumed);
                            }
                        }
                        return Response::consumed(self.id.clone());
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Clean copy without gutters or diff markers
                        if let Some(r) = self.source.row(cur_row_idx) {
                            let text = r.text.to_owned();
                            return Response::action(
                                self.id.clone(),
                                DiffAction::CopyRequested { text },
                            )
                            .with_flow(Flow::Consumed);
                        }
                    }
                    KeyCode::Char('c') if k.mods.contains(KeyModifiers::CONTROL) => {
                        if let Some(r) = self.source.row(cur_row_idx) {
                            let text = r.text.to_owned();
                            return Response::action(
                                self.id.clone(),
                                DiffAction::CopyRequested { text },
                            )
                            .with_flow(Flow::Consumed);
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
        let count = self.source.row_count();
        let header_h = if self.source.file_header().is_some() {
            1
        } else {
            0
        };
        let needed = Size::new(
            constraints.min.width.max(42),
            (header_h + count as u16).max(3),
        );
        constraints.clamp(needed)
    }

    /// Read-only draw pass.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &DiffViewState) -> Rect {
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

        let row_count = self.source.row_count();
        let mut body_y = area.y;

        // Render file header if present
        if let Some(header) = self.source.file_header() {
            let header_rect = Rect::new(area.x, body_y, area.width, 1);
            ui.fill_rect(header_rect, Style::new().bg(theme.tokens.surface_elevated));

            let status_marker = self.source.file_status().map(|s| s.marker()).unwrap_or("M");
            let status_tone = self
                .source
                .file_status()
                .map(|s| s.tone())
                .unwrap_or(Tone::Warning);

            ui.set_string(
                area.x + 1,
                body_y,
                status_marker,
                Style::new()
                    .fg(theme.tone_color(status_tone))
                    .add_modifier(Modifier::BOLD),
            );
            ui.set_string(
                area.x + 3,
                body_y,
                header,
                Style::new()
                    .fg(theme.tokens.text_primary)
                    .add_modifier(Modifier::BOLD),
            );
            body_y += 1;
        }

        if row_count == 0 {
            ui.set_string(
                area.x + 2,
                body_y + 1,
                "(no textual changes)",
                Style::new().fg(theme.tokens.text_muted),
            );
            return area;
        }

        let body_h = (area.y + area.height).saturating_sub(body_y) as usize;
        let mode = self.effective_mode(area.width, state);

        let visible_range = state.scroll.visible_range();

        if mode == DiffMode::Unified {
            // Unified Mode
            for (slot, row_idx) in visible_range.enumerate() {
                if slot >= body_h {
                    break;
                }
                let y = body_y.saturating_add(slot as u16);
                let row = match self.source.row(row_idx) {
                    Some(r) => r,
                    None => continue,
                };

                let is_cursor = state.cursor_row == Some(row.key);
                let cursor_marker = if is_cursor { "▎" } else { " " };
                ui.set_string(
                    area.x,
                    y,
                    cursor_marker,
                    Style::new().fg(theme.tokens.accent),
                );

                if row.kind == DiffLineKind::Header {
                    let header_style = Style::new()
                        .fg(theme.tokens.accent)
                        .add_modifier(Modifier::BOLD);
                    ui.set_string(area.x + 2, y, row.text, header_style);
                    continue;
                }

                // Old line number
                let old_str = row
                    .old_lineno
                    .map(|n| format!("{:>3}", n))
                    .unwrap_or_else(|| "   ".into());
                ui.set_string(
                    area.x + 1,
                    y,
                    &old_str,
                    Style::new().fg(theme.tokens.text_muted),
                );

                // New line number
                let new_str = row
                    .new_lineno
                    .map(|n| format!("{:>3}", n))
                    .unwrap_or_else(|| "   ".into());
                ui.set_string(
                    area.x + 5,
                    y,
                    &new_str,
                    Style::new().fg(theme.tokens.text_muted),
                );

                // Diff marker (+ / - / space)
                let marker = row.kind.marker();
                let marker_style = Style::new()
                    .fg(theme.tone_color(row.kind.tone()))
                    .add_modifier(Modifier::BOLD);
                ui.set_string(area.x + 9, y, marker, marker_style);

                // Line text with background highlight if added/removed
                let text_style = match row.kind {
                    DiffLineKind::Add => Style::new().fg(theme.tokens.success),
                    DiffLineKind::Remove => Style::new().fg(theme.tokens.danger),
                    _ => Style::new().fg(theme.tokens.text_primary),
                };

                let text_x = area.x + 11;
                let max_w = area.width.saturating_sub(11) as usize;
                let display = truncate(row.text, max_w);
                ui.set_string(text_x, y, &display, text_style);
            }
        } else {
            // Review Mode: side-by-side
            let gutter_w = 4u16;
            let pane_w = (area.width.saturating_sub(gutter_w * 2 + 3)) / 2;
            let sep_x = area.x + gutter_w + pane_w + 1;

            for (slot, row_idx) in visible_range.enumerate() {
                if slot >= body_h {
                    break;
                }
                let y = body_y.saturating_add(slot as u16);
                let row = match self.source.row(row_idx) {
                    Some(r) => r,
                    None => continue,
                };

                let is_cursor = state.cursor_row == Some(row.key);
                let cursor_marker = if is_cursor { "▎" } else { " " };
                ui.set_string(
                    area.x,
                    y,
                    cursor_marker,
                    Style::new().fg(theme.tokens.accent),
                );

                // Separator
                ui.set_string(sep_x, y, "│", Style::new().fg(theme.tokens.border_subtle));

                if row.kind == DiffLineKind::Header {
                    let header_style = Style::new()
                        .fg(theme.tokens.accent)
                        .add_modifier(Modifier::BOLD);
                    ui.set_string(area.x + 2, y, row.text, header_style);
                    continue;
                }

                // Left pane (Old side)
                if row.kind == DiffLineKind::Context || row.kind == DiffLineKind::Remove {
                    let old_str = row
                        .old_lineno
                        .map(|n| format!("{:>3}", n))
                        .unwrap_or_else(|| "   ".into());
                    ui.set_string(
                        area.x + 1,
                        y,
                        &old_str,
                        Style::new().fg(theme.tokens.text_muted),
                    );

                    let style = if row.kind == DiffLineKind::Remove {
                        Style::new().fg(theme.tokens.danger)
                    } else {
                        Style::new().fg(theme.tokens.text_primary)
                    };
                    let text = truncate(row.text, pane_w as usize);
                    ui.set_string(area.x + gutter_w + 1, y, &text, style);
                }

                // Right pane (New side)
                if row.kind == DiffLineKind::Context || row.kind == DiffLineKind::Add {
                    let new_x = sep_x + 1;
                    let new_str = row
                        .new_lineno
                        .map(|n| format!("{:>3}", n))
                        .unwrap_or_else(|| "   ".into());
                    ui.set_string(new_x, y, &new_str, Style::new().fg(theme.tokens.text_muted));

                    let style = if row.kind == DiffLineKind::Add {
                        Style::new().fg(theme.tokens.success)
                    } else {
                        Style::new().fg(theme.tokens.text_primary)
                    };
                    let text = truncate(row.text, pane_w as usize);
                    ui.set_string(new_x + gutter_w, y, &text, style);
                }
            }
        }

        area
    }
}
