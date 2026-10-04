//! Prepared-cell terminal presentation component and terminal edge contracts.
//!
//! Provides the canonical [`TerminalView`], [`TerminalInteraction`],
//! [`TerminalSelection`], [`LinkKey`], [`InputToken`], [`TerminalAction`],
//! and [`TerminalViewState`].

use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Modifier, Style};

use crate::core::event::MouseKind;
use crate::termrock::author::TerminalSource;
use crate::termrock::identity::{Id, ItemKey, Part, Revision};
use crate::termrock::layout::{Axis, Constraints, Position, Rect, Size};
use crate::termrock::response::{Flow, Input, Invalidate, Key, Response, UpdateCause};
use crate::termrock::runtime::{Cx, MeasureCx, Ui};
use crate::termrock::scroll::{ScrollRegion, ScrollState};
use crate::termrock::theme::StylePatch;

/// Interactive mode policy for `TerminalView`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TerminalInteraction {
    /// Full interaction: forwards input events, allows selection and scrolling.
    #[default]
    Interactive,
    /// Read-only view: allows scrolling, no input forwarding or selection mutations.
    ReadOnly,
    /// Selection-only mode: allows cursor navigation, scrolling, and copy selection, no raw input forwarding.
    SelectionOnly,
}

impl TerminalInteraction {
    pub const fn allows_forwarding(&self) -> bool {
        matches!(self, Self::Interactive)
    }

    pub const fn allows_selection(&self) -> bool {
        matches!(self, Self::Interactive | Self::SelectionOnly)
    }
}

/// Selection span in stable line/column coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TerminalSelection {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub revision: Revision,
}

impl TerminalSelection {
    pub const fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            revision: Revision::zero(),
        }
    }

    pub const fn with_revision(mut self, revision: Revision) -> Self {
        self.revision = revision;
        self
    }

    /// Normalized range `(start_line, start_col, end_line, end_col)` where start <= end.
    pub const fn normalized(&self) -> (usize, usize, usize, usize) {
        if self.start_line < self.end_line
            || (self.start_line == self.end_line && self.start_col <= self.end_col)
        {
            (self.start_line, self.start_col, self.end_line, self.end_col)
        } else {
            (self.end_line, self.end_col, self.start_line, self.start_col)
        }
    }

    /// Check if a coordinate is within the selection.
    pub fn contains(&self, line: usize, col: usize) -> bool {
        let (s_line, s_col, e_line, e_col) = self.normalized();
        if line < s_line || line > e_line {
            return false;
        }
        if s_line == e_line {
            line == s_line && col >= s_col && col <= e_col
        } else if line == s_line {
            col >= s_col
        } else if line == e_line {
            col <= e_col
        } else {
            true
        }
    }
}

/// Typed key identifying a hyperlink or actionable terminal target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkKey(pub ItemKey);

impl LinkKey {
    pub fn new(key: impl Into<ItemKey>) -> Self {
        Self(key.into())
    }

    pub fn as_item_key(&self) -> &ItemKey {
        &self.0
    }
}

impl From<ItemKey> for LinkKey {
    fn from(key: ItemKey) -> Self {
        Self(key)
    }
}

impl From<&'static str> for LinkKey {
    fn from(s: &'static str) -> Self {
        Self(ItemKey::from_str(s))
    }
}

impl From<String> for LinkKey {
    fn from(s: String) -> Self {
        Self(ItemKey::from_str(&s))
    }
}

impl From<u64> for LinkKey {
    fn from(id: u64) -> Self {
        Self(ItemKey::new(id))
    }
}

impl std::ops::Deref for LinkKey {
    type Target = ItemKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Representation of an input event to forward back to the host/session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputToken {
    /// Normalized input event.
    Input(Input),
    /// Single key event.
    Key(Key),
    /// Host-retained raw byte representation.
    Raw(Vec<u8>),
    /// Opaque token ID referencing host-retained input.
    Opaque(u64),
}

impl InputToken {
    pub fn new(input: Input) -> Self {
        Self::Input(input)
    }

    pub fn from_key(key: Key) -> Self {
        Self::Key(key)
    }

    pub fn from_raw(bytes: Vec<u8>) -> Self {
        Self::Raw(bytes)
    }

    pub fn opaque(id: u64) -> Self {
        Self::Opaque(id)
    }

    pub fn as_input(&self) -> Option<&Input> {
        match self {
            Self::Input(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_key(&self) -> Option<&Key> {
        match self {
            Self::Key(k) => Some(k),
            Self::Input(Input::Key(k)) => Some(k),
            _ => None,
        }
    }

    pub fn as_raw(&self) -> Option<&[u8]> {
        match self {
            Self::Raw(b) => Some(b.as_slice()),
            _ => None,
        }
    }
}

impl From<Input> for InputToken {
    fn from(input: Input) -> Self {
        Self::Input(input)
    }
}

impl From<Key> for InputToken {
    fn from(key: Key) -> Self {
        Self::Key(key)
    }
}

impl From<Vec<u8>> for InputToken {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Raw(bytes)
    }
}

/// Typed actions emitted by `TerminalView`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalAction {
    /// Forward an input event to the external session/process.
    Forward { token: InputToken },
    /// Request copying of a selection span in source coordinates.
    CopyRequested(TerminalSelection),
    /// Request opening an identified hyperlink.
    OpenLink { key: LinkKey },
    /// Viewport size change reported to host adapter.
    Resize { rows: u16, cols: u16 },
}

/// Durable state for `TerminalView`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalViewState {
    pub scroll: ScrollState,
    pub selection: Option<TerminalSelection>,
    pub cursor_visible: bool,
    pub selection_anchor: Option<(usize, usize)>,
    pub selecting: bool,
    pub last_area: Rect,
    pub last_reported_size: Option<Size>,
}

impl TerminalViewState {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::default(),
            selection: None,
            cursor_visible: true,
            selection_anchor: None,
            selecting: false,
            last_area: Rect::zero(),
            last_reported_size: None,
        }
    }

    pub fn with_scroll(mut self, scroll: ScrollState) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn with_selection(mut self, selection: Option<TerminalSelection>) -> Self {
        self.selection = selection;
        self
    }

    pub fn with_cursor_visible(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
        self.selecting = false;
    }
}

/// Prepared-cell terminal presentation component.
pub struct TerminalView<'a> {
    pub id: Id,
    pub source: &'a dyn TerminalSource,
    pub interaction: TerminalInteraction,
    pub selection: Option<TerminalSelection>,
    pub dimmed: bool,
    pub patch: Option<StylePatch>,
    pub link_resolver: Option<&'a (dyn Fn(usize, usize) -> Option<LinkKey> + 'a)>,
}

impl<'a> TerminalView<'a> {
    pub fn new(id: Id, source: &'a dyn TerminalSource) -> Self {
        Self {
            id,
            source,
            interaction: TerminalInteraction::Interactive,
            selection: None,
            dimmed: false,
            patch: None,
            link_resolver: None,
        }
    }

    pub fn interaction(mut self, interaction: TerminalInteraction) -> Self {
        self.interaction = interaction;
        self
    }

    pub fn selection(mut self, selection: Option<TerminalSelection>) -> Self {
        self.selection = selection;
        self
    }

    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }

    pub fn patch(mut self, patch: StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    pub fn link_resolver(
        mut self,
        resolver: &'a (dyn Fn(usize, usize) -> Option<LinkKey> + 'a),
    ) -> Self {
        self.link_resolver = Some(resolver);
        self
    }

    /// Extract text content of a selection from the terminal source.
    pub fn extract_selection(&self, selection: &TerminalSelection) -> String {
        let (s_line, s_col, e_line, e_col) = selection.normalized();
        let mut lines = Vec::new();
        let src_size = self.source.size();

        for line_idx in s_line..=e_line {
            if line_idx >= src_size.height as usize {
                break;
            }
            let col_start = if line_idx == s_line { s_col } else { 0 };
            let col_end = if line_idx == e_line {
                e_col
            } else {
                src_size.width.saturating_sub(1) as usize
            };

            let mut line_str = String::new();
            for col_idx in col_start..=col_end {
                if col_idx >= src_size.width as usize {
                    break;
                }
                if let Some(cell) = self
                    .source
                    .cell(Position::new(col_idx as u16, line_idx as u16))
                    && !cell.continuation
                {
                    line_str.push_str(cell.symbol);
                }
            }
            lines.push(line_str);
        }
        lines.join("\n")
    }

    pub fn measure(&self, _cx: &MeasureCx<'_>, constraints: Constraints) -> Size {
        constraints.clamp(self.source.size())
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TerminalViewState,
    ) -> Response<TerminalAction> {
        let intended = cx.intended_owner();
        let is_target = intended == Some(&self.id)
            || intended.and_then(|id| id.parent()).as_ref() == Some(&self.id)
            || cx.focus.as_ref() == Some(&self.id)
            || (intended.is_none() && cx.focus.is_none());

        let area = cx
            .published_geometry
            .and_then(|g| g.get(&self.id))
            .copied()
            .unwrap_or(state.last_area);

        if !area.is_empty() {
            state.last_area = area;
        }

        let src_size = self.source.size();
        state.scroll.total = src_size.height as usize;
        state.scroll.viewport = area.height as usize;
        state.scroll.clamp();

        // Check for stale history rejection on selection
        let mut had_stale_selection = false;
        if let Some(ref sel) = state.selection
            && (sel.revision != Revision::zero() && sel.revision != self.source.revision())
        {
            state.clear_selection();
            cx.request_invalidate(Invalidate::Paint);
            had_stale_selection = true;
        }

        // Handle viewport resize facts if geometry changed
        if !area.is_empty() {
            let current_size = Size::new(area.width, area.height);
            if state.last_reported_size.is_some() && state.last_reported_size != Some(current_size)
            {
                state.last_reported_size = Some(current_size);
                return Response::action(
                    self.id.clone(),
                    TerminalAction::Resize {
                        rows: area.height,
                        cols: area.width,
                    },
                );
            }
            if state.last_reported_size.is_none() {
                state.last_reported_size = Some(current_size);
            }
        }

        // Handle mouse events (wheel, selection, links)
        if let UpdateCause::Input(Input::Mouse(m), _) = cx.cause() {
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
                MouseKind::Down => {
                    if area.contains(m.pos.into()) {
                        let rel_y = (m.pos.y.saturating_sub(area.y)) as usize + state.scroll.offset;
                        let rel_x = (m.pos.x.saturating_sub(area.x)) as usize;

                        // Check link
                        if let Some(resolver) = self.link_resolver
                            && let Some(key) = resolver(rel_y, rel_x)
                        {
                            return Response::action(
                                self.id.clone(),
                                TerminalAction::OpenLink { key },
                            )
                            .with_flow(Flow::Consumed);
                        }

                        // Selection start
                        if self.interaction.allows_selection() {
                            state.selection_anchor = Some((rel_y, rel_x));
                            state.selecting = true;
                            state.selection = Some(
                                TerminalSelection::new(rel_y, rel_x, rel_y, rel_x)
                                    .with_revision(self.source.revision()),
                            );
                            cx.request_invalidate(Invalidate::Paint);
                            return Response::consumed(self.id.clone());
                        }
                    }
                }
                MouseKind::Drag => {
                    if state.selecting
                        && self.interaction.allows_selection()
                        && let Some((anchor_y, anchor_x)) = state.selection_anchor
                    {
                        let rel_y = (m.pos.y.saturating_sub(area.y)) as usize + state.scroll.offset;
                        let rel_x = (m.pos.x.saturating_sub(area.x)) as usize;
                        state.selection = Some(
                            TerminalSelection::new(anchor_y, anchor_x, rel_y, rel_x)
                                .with_revision(self.source.revision()),
                        );
                        cx.request_invalidate(Invalidate::Paint);
                        return Response::consumed(self.id.clone());
                    }
                }
                MouseKind::Up if state.selecting => {
                    state.selecting = false;
                    cx.request_invalidate(Invalidate::Paint);
                    return Response::consumed(self.id.clone());
                }
                _ => {}
            }
        }

        // Handle keyboard input
        if let UpdateCause::Input(Input::Key(k), _) = cx.cause() {
            // Copy request: Ctrl+C or Ctrl+Shift+C (or 'y' in selection-only mode)
            let is_copy_key = (k.ctrl()
                && (k.code == KeyCode::Char('c') || k.code == KeyCode::Char('C')))
                || (k.code == KeyCode::Char('y')
                    && self.interaction == TerminalInteraction::SelectionOnly);

            if is_copy_key {
                if had_stale_selection {
                    return Response::consumed(self.id.clone());
                }
                if let Some(ref sel) = state.selection {
                    return Response::action(self.id.clone(), TerminalAction::CopyRequested(*sel))
                        .with_flow(Flow::Consumed);
                }
            }

            // Escape clears selection
            if k.code == KeyCode::Esc && state.selection.is_some() {
                state.clear_selection();
                cx.request_invalidate(Invalidate::Paint);
                return Response::consumed(self.id.clone());
            }

            // Scrolling keys
            if k.code == KeyCode::PageUp {
                let delta = state.scroll.viewport.max(1);
                if state.scroll.scroll_up(delta) {
                    cx.request_invalidate(Invalidate::Paint);
                }
                return Response::consumed(self.id.clone());
            }
            if k.code == KeyCode::PageDown {
                let delta = state.scroll.viewport.max(1);
                if state.scroll.scroll_down(delta) {
                    cx.request_invalidate(Invalidate::Paint);
                }
                return Response::consumed(self.id.clone());
            }

            // Normal mode: forward input if target
            if self.interaction.allows_forwarding() && is_target {
                return Response::action(
                    self.id.clone(),
                    TerminalAction::Forward {
                        token: InputToken::from_key(*k),
                    },
                )
                .with_flow(Flow::Consumed);
            }
        }

        // Paste forwarding
        if let UpdateCause::Input(Input::Paste(text), _) = cx.cause()
            && self.interaction.allows_forwarding()
            && is_target
        {
            return Response::action(
                self.id.clone(),
                TerminalAction::Forward {
                    token: InputToken::new(Input::Paste(text.clone())),
                },
            )
            .with_flow(Flow::Consumed);
        }

        // Direct Resize input
        if let UpdateCause::Input(Input::Resize(cols, rows), _) = cx.cause() {
            return Response::action(
                self.id.clone(),
                TerminalAction::Resize {
                    rows: *rows,
                    cols: *cols,
                },
            )
            .with_flow(Flow::Consumed);
        }

        Response::bubble(self.id.clone())
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TerminalViewState) -> Rect {
        let clipped = ui.clip_area().intersect(area);
        ui.register_hit(self.id.clone(), clipped);
        ui.register_focus(self.id.clone(), true);

        if clipped.is_empty() {
            return area;
        }

        ui.part(self.id.clone(), Part::CONTAINER, clipped, |_p| {});

        let is_focused = ui.is_focused(&self.id);
        let src_size = self.source.size();
        let scroll_offset = state.scroll.offset;
        let rev = self.source.revision();

        // Effective selection (validating against source revision)
        let effective_sel = self
            .selection
            .as_ref()
            .or(state.selection.as_ref())
            .filter(|s| s.revision == Revision::zero() || s.revision == rev);

        // Blit cells
        if let Some(buf) = ui.buffer.as_deref_mut() {
            for row in 0..clipped.height {
                let abs_y = clipped.y + row;
                if abs_y >= buf.area().height {
                    break;
                }
                let src_line = scroll_offset + row as usize;
                if src_line >= src_size.height as usize {
                    continue;
                }

                for col in 0..clipped.width {
                    let abs_x = clipped.x + col;
                    if abs_x >= buf.area().width {
                        break;
                    }
                    let src_col = col as usize;
                    if src_col >= src_size.width as usize {
                        continue;
                    }

                    if let Some(tcell) = self
                        .source
                        .cell(Position::new(src_col as u16, src_line as u16))
                    {
                        let buf_cell = &mut buf[(abs_x, abs_y)];
                        if tcell.continuation {
                            buf_cell.set_symbol("");
                        } else {
                            buf_cell.set_symbol(tcell.symbol);
                        }

                        let mut style = Style::default();
                        if let Some(fg) = tcell.fg {
                            style = style.fg(fg);
                        }
                        if let Some(bg) = tcell.bg {
                            style = style.bg(bg);
                        }
                        style = style.add_modifier(tcell.modifier);

                        if self.dimmed {
                            style = style.add_modifier(Modifier::DIM);
                        }

                        if let Some(sel) = effective_sel
                            && sel.contains(src_line, src_col)
                        {
                            style = style.add_modifier(Modifier::REVERSED);
                        }

                        buf_cell.set_style(style);
                    }
                }
            }
        }

        // Draw / request cursor
        if state.cursor_visible
            && let Some(cursor) = self.source.cursor()
            && cursor.visible
        {
            let cur_col = cursor.pos.x as usize;
            let cur_line = cursor.pos.y as usize;
            if cur_line >= scroll_offset {
                let view_row = cur_line - scroll_offset;
                if view_row < clipped.height as usize && cur_col < clipped.width as usize {
                    let cursor_abs_x = clipped.x + cur_col as u16;
                    let cursor_abs_y = clipped.y + view_row as u16;
                    ui.request_cursor(self.id.clone(), Position::new(cursor_abs_x, cursor_abs_y));

                    if let Some(buf) = ui.buffer.as_deref_mut()
                        && cursor_abs_x < buf.area().width
                        && cursor_abs_y < buf.area().height
                    {
                        let cell = &mut buf[(cursor_abs_x, cursor_abs_y)];
                        if is_focused {
                            cell.modifier.insert(Modifier::REVERSED);
                        } else {
                            cell.modifier.insert(Modifier::UNDERLINED);
                        }
                    }
                }
            }
        }

        // Draw scrollbar if content overflows vertically
        if state.scroll.is_overflowing() {
            let scroll_region = ScrollRegion::new(
                self.id.sub("scroll"),
                Axis::Vertical,
                src_size.height as usize,
                clipped.height as usize,
            );
            scroll_region.draw(ui, clipped, &state.scroll);
        }

        area
    }
}
