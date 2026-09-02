use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::core::text::TextBuffer;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::field_common::{EditAction, edit_key};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub width: Constraint,
    pub align: Align,
    pub editable: bool,
    pub sortable: bool,
}

impl Column {
    pub fn new(title: &str, width: Constraint) -> Self {
        Self {
            title: title.to_owned(),
            width,
            align: Align::Left,
            editable: false,
            sortable: true,
        }
    }
    pub fn right(mut self) -> Self {
        self.align = Align::Right;
        self
    }
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
    pub fn min_width(&self) -> u16 {
        match self.width {
            Constraint::Length(n) | Constraint::Min(n) => n,
            _ => 6,
        }
    }
}

/// A cell value with an optional per-cell error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    pub error: Option<String>,
    pub tone: Tone,
}

pub use crate::theme::Tone;

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            error: None,
            tone: Tone::Normal,
        }
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub row: usize,
    pub col: usize,
    pub buffer: TextBuffer,
    pub error: Option<String>,
}

/// Data table with header sorting, row/cell keyboard navigation, hover and
/// optional in-place cell editing.
#[derive(Debug, Clone)]
pub struct DataTable {
    pub id: WidgetId,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Cell>>,
    /// Display order → source row index (sorting is a permutation, so edits
    /// land in the right row).
    order: Vec<usize>,
    pub sort: Option<(usize, SortDir)>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// When false the whole row is the navigation unit.
    pub cell_nav: bool,
    pub selected: Option<usize>,
    pub scroll: ScrollState,
    pub hscroll: ScrollState,
    pub edit: Option<EditState>,
    pub validator: Option<fn(col: usize, &str) -> Option<String>>,
    pub empty_text: String,
    pub area: Rect,
    body: Rect,
    col_rects: Vec<Rect>,
    /// Numeric sort for these columns.
    pub numeric: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableEvent {
    Committed {
        row: usize,
        col: usize,
    },
    Cancelled,
    Activated(usize),
    /// Editing ended via Tab at the last cell; caller may move focus.
    LeaveForward,
    LeaveBackward,
}

impl DataTable {
    pub fn new(id: WidgetId, columns: Vec<Column>, rows: Vec<Vec<Cell>>) -> Self {
        let n = rows.len();
        let numeric = vec![false; columns.len()];
        Self {
            id,
            columns,
            rows,
            order: (0..n).collect(),
            sort: None,
            cursor_row: 0,
            cursor_col: 0,
            cell_nav: false,
            selected: None,
            scroll: ScrollState::new(n),
            hscroll: ScrollState::default(),
            edit: None,
            validator: None,
            empty_text: "No rows".to_owned(),
            area: Rect::ZERO,
            body: Rect::ZERO,
            col_rects: vec![],
            numeric,
        }
    }

    pub fn cell_nav(mut self, on: bool) -> Self {
        self.cell_nav = on;
        self
    }
    pub fn numeric(mut self, cols: &[usize]) -> Self {
        for &c in cols {
            if c < self.numeric.len() {
                self.numeric[c] = true;
            }
        }
        self
    }
    pub fn validator(mut self, v: fn(usize, &str) -> Option<String>) -> Self {
        self.validator = Some(v);
        self
    }
    pub fn empty_text(mut self, s: &str) -> Self {
        self.empty_text = s.to_owned();
        self
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Source index of the row at display position `i`.
    pub fn source_row(&self, i: usize) -> usize {
        self.order[i]
    }

    pub fn header_id(&self, col: usize) -> WidgetId {
        self.id.sub("header").child(col)
    }
    pub fn row_id(&self, display: usize) -> WidgetId {
        self.id.child(display)
    }
    pub fn cell_id(&self, display: usize, col: usize) -> WidgetId {
        self.id.child(display).child(col)
    }

    pub fn set_rows(&mut self, rows: Vec<Vec<Cell>>) {
        self.rows = rows;
        self.order = (0..self.rows.len()).collect();
        self.scroll.set_content(self.rows.len());
        if let Some((c, d)) = self.sort {
            self.apply_sort(c, d);
        }
        self.cursor_row = self.cursor_row.min(self.rows.len().saturating_sub(1));
    }

    fn apply_sort(&mut self, col: usize, dir: SortDir) {
        let numeric = self.numeric.get(col).copied().unwrap_or(false);
        let rows = &self.rows;
        self.order.sort_by(|&a, &b| {
            let ca = &rows[a][col].text;
            let cb = &rows[b][col].text;
            let ord = if numeric {
                // non-numeric cells (e.g. "—") sort before every number
                parse_num(ca).cmp(&parse_num(cb))
            } else {
                ca.to_lowercase().cmp(&cb.to_lowercase())
            };
            let ord = ord.then_with(|| a.cmp(&b));
            match dir {
                SortDir::Asc => ord,
                SortDir::Desc => ord.reverse(),
            }
        });
    }

    /// Cycle sort on a column: asc → desc → none.
    pub fn sort_by(&mut self, col: usize) -> Outcome {
        if col >= self.columns.len() || !self.columns[col].sortable {
            return Outcome::Consumed;
        }
        // keep the cursor on the same source row
        let src = self.order.get(self.cursor_row).copied();
        self.sort = match self.sort {
            Some((c, SortDir::Asc)) if c == col => Some((col, SortDir::Desc)),
            Some((c, SortDir::Desc)) if c == col => None,
            _ => Some((col, SortDir::Asc)),
        };
        match self.sort {
            Some((c, d)) => self.apply_sort(c, d),
            None => self.order = (0..self.rows.len()).collect(),
        }
        if let Some(src) = src
            && let Some(pos) = self.order.iter().position(|&r| r == src)
        {
            self.cursor_row = pos;
            self.scroll.ensure_visible(pos);
        }
        Outcome::Changed
    }

    fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows.len().saturating_sub(1));
        self.cursor_col = col.min(self.columns.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor_row);
        self.ensure_col_visible();
    }

    fn ensure_col_visible(&mut self) {
        // horizontal scroll is in columns
        let n = self.columns.len();
        if self.hscroll.viewport_len == 0 {
            return;
        }
        if self.cursor_col < self.hscroll.offset {
            self.hscroll.offset = self.cursor_col;
        } else if self.cursor_col >= self.hscroll.offset + self.hscroll.viewport_len {
            self.hscroll.offset = self.cursor_col + 1 - self.hscroll.viewport_len;
        }
        self.hscroll.offset = self
            .hscroll
            .offset
            .min(n.saturating_sub(self.hscroll.viewport_len));
    }

    pub fn begin_edit(&mut self) -> Outcome {
        if self.rows.is_empty() {
            return Outcome::Consumed;
        }
        let col = self.cursor_col;
        if !self.columns[col].editable {
            return Outcome::Consumed;
        }
        let src = self.order[self.cursor_row];
        let text = self.rows[src][col].text.clone();
        self.edit = Some(EditState {
            row: self.cursor_row,
            col,
            buffer: TextBuffer::single(text),
            error: None,
        });
        Outcome::Changed
    }

    pub fn commit_edit(&mut self) -> Option<TableEvent> {
        let e = self.edit.take()?;
        let src = self.order[e.row];
        let text = e.buffer.text().to_owned();
        let err = self.validator.and_then(|v| v(e.col, &text));
        if let Some(err) = err {
            // keep editing, show error
            self.edit = Some(EditState {
                error: Some(err),
                ..e
            });
            return None;
        }
        self.rows[src][e.col].text = text;
        self.rows[src][e.col].error = None;
        if let Some((c, d)) = self.sort
            && c == e.col
        {
            self.apply_sort(c, d);
            if let Some(pos) = self.order.iter().position(|&r| r == src) {
                self.cursor_row = pos;
                self.scroll.ensure_visible(pos);
            }
        }
        Some(TableEvent::Committed {
            row: src,
            col: e.col,
        })
    }

    pub fn cancel_edit(&mut self) -> Option<TableEvent> {
        self.edit.take().map(|_| TableEvent::Cancelled)
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TableEvent>) {
        if let Some(e) = self.edit.as_mut() {
            return match edit_key(key, false) {
                EditAction::Commit => match self.commit_edit() {
                    Some(ev) => (Outcome::Changed, Some(ev)),
                    None => (Outcome::Changed, None),
                },
                EditAction::Cancel => (Outcome::Changed, self.cancel_edit()),
                EditAction::Tab { backward } => {
                    let ev = self.commit_edit();
                    if ev.is_none() {
                        return (Outcome::Changed, None);
                    }
                    // move to next editable cell
                    let n = self.columns.len();
                    let mut col = self.cursor_col;
                    loop {
                        if backward {
                            if col == 0 {
                                return (Outcome::Changed, Some(TableEvent::LeaveBackward));
                            }
                            col -= 1;
                        } else {
                            col += 1;
                            if col >= n {
                                return (Outcome::Changed, Some(TableEvent::LeaveForward));
                            }
                        }
                        if self.columns[col].editable {
                            break;
                        }
                    }
                    self.set_cursor(self.cursor_row, col);
                    self.begin_edit();
                    (Outcome::Changed, ev)
                }
                EditAction::Apply(f) => {
                    f(&mut e.buffer);
                    (Outcome::Changed, None)
                }
                EditAction::Insert(c) => {
                    e.buffer.insert_char(c);
                    if e.error.is_some() {
                        e.error = self.validator.and_then(|v| v(e.col, e.buffer.text()));
                    }
                    (Outcome::Changed, None)
                }
                EditAction::None => (Outcome::Consumed, None),
            };
        }
        if self.rows.is_empty() {
            return (Outcome::Ignored, None);
        }
        let (r, c) = (self.cursor_row, self.cursor_col);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.set_cursor(r.saturating_sub(1), c);
                (Outcome::Changed, None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.set_cursor(r + 1, c);
                (Outcome::Changed, None)
            }
            KeyCode::Left | KeyCode::Char('h') if self.cell_nav => {
                self.set_cursor(r, c.saturating_sub(1));
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') if self.cell_nav => {
                self.set_cursor(r, c + 1);
                (Outcome::Changed, None)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.hscroll.scroll_by(-1);
                (Outcome::Changed, None)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.hscroll.scroll_by(1);
                (Outcome::Changed, None)
            }
            KeyCode::PageUp => {
                self.set_cursor(r.saturating_sub(self.scroll.viewport_len.max(1)), c);
                (Outcome::Changed, None)
            }
            KeyCode::PageDown => {
                self.set_cursor(r + self.scroll.viewport_len.max(1), c);
                (Outcome::Changed, None)
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.set_cursor(0, c);
                (Outcome::Changed, None)
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.set_cursor(usize::MAX, c);
                (Outcome::Changed, None)
            }
            KeyCode::Char('s') => (self.sort_by(c), None),
            KeyCode::Enter | KeyCode::F(2) if self.cell_nav && self.columns[c].editable => {
                (self.begin_edit(), None)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.selected = Some(self.order[r]);
                (Outcome::Changed, Some(TableEvent::Activated(self.order[r])))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        match self.edit.as_mut() {
            Some(e) => {
                e.buffer.insert_str(text);
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_click_header(&mut self, col: usize) -> Outcome {
        if self.edit.is_some() {
            self.commit_edit();
        }
        self.sort_by(col)
    }

    /// Click on a row (display index) / cell. Double click is emulated by
    /// clicking the already-current cell.
    pub fn on_click_cell(
        &mut self,
        display: usize,
        col: usize,
        pos: Position,
    ) -> (Outcome, Option<TableEvent>) {
        if display >= self.rows.len() {
            return (Outcome::Consumed, None);
        }
        let col = col.min(self.columns.len().saturating_sub(1));
        if let Some(e) = &self.edit {
            if e.row == display && e.col == col {
                // click inside the editor places the cursor
                if let Some(rect) = self.col_rects.get(col) {
                    let c = pos.x.saturating_sub(rect.x) as usize;
                    self.edit.as_mut().unwrap().buffer.set_cursor_line_col(0, c);
                }
                return (Outcome::Changed, None);
            }
            self.commit_edit();
        }
        let same = self.cursor_row == display && (!self.cell_nav || self.cursor_col == col);
        self.set_cursor(display, if self.cell_nav { col } else { self.cursor_col });
        if same && self.cell_nav && self.columns[col].editable {
            return (self.begin_edit(), None);
        }
        if !self.cell_nav {
            self.selected = Some(self.order[display]);
            return (
                Outcome::Changed,
                Some(TableEvent::Activated(self.order[display])),
            );
        }
        (Outcome::Changed, None)
    }

    /// Horizontal wheel: pages the visible columns.
    pub fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        let before = self.hscroll.offset;
        self.hscroll.scroll_by(delta as isize);
        if self.hscroll.offset == before {
            Outcome::Consumed
        } else {
            Outcome::Changed
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.body.y,
            1,
            self.body.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
    }

    /// Compute column rects for the visible columns.
    fn layout_columns(&mut self, area: Rect) {
        let n = self.columns.len();
        // decide how many columns fit starting at hscroll.offset
        let gap = 2u16;
        let mut fit = 0usize;
        let mut used = 0u16;
        for i in self.hscroll.offset..n {
            let w = self.columns[i].min_width();
            let need = if fit == 0 { w } else { w + gap };
            if used + need > area.width {
                break;
            }
            used += need;
            fit += 1;
        }
        let fit = fit.max(1).min(n - self.hscroll.offset.min(n));
        self.hscroll.content_len = n;
        self.hscroll.viewport_len = fit;
        self.hscroll.clamp();
        let visible = self.hscroll.offset..(self.hscroll.offset + fit).min(n);
        let constraints: Vec<Constraint> = visible.clone().map(|i| self.columns[i].width).collect();
        let rects = Layout::horizontal(constraints).spacing(gap).split(area);
        self.col_rects = vec![Rect::ZERO; n];
        for (k, i) in visible.enumerate() {
            self.col_rects[i] = rects[k];
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() || area.height < 2 {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let editing = self.edit.is_some() && focused;
        if !focused && self.edit.is_some() {
            self.commit_edit();
        }

        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        // gutter (1) + columns + scrollbar (1)
        self.scroll.set_content(self.rows.len());
        self.scroll.set_viewport(area.height as usize - 1);
        let has_sb = self.scroll.overflows();
        let cols_area = Rect::new(
            area.x + 3,
            area.y,
            area.width.saturating_sub(5 + if has_sb { 1 } else { 0 }),
            area.height,
        );
        self.layout_columns(cols_area);
        let header_y = area.y;
        let body = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
        self.body = body;

        // header
        let header_bg = bg;
        fill(
            buf,
            Rect::new(area.x, header_y, area.width, 1),
            Style::new().bg(header_bg),
        );
        for (ci, col) in self.columns.iter().enumerate() {
            let r = self.col_rects[ci];
            if r.is_empty() {
                continue;
            }
            let hid = self.header_id(ci);
            let hovered = ctx.interaction.hovered(hid);
            let sorted = self.sort.map(|(c, _)| c == ci).unwrap_or(false);
            let mut st = if sorted {
                t.primary().bg(header_bg)
            } else {
                t.muted().bg(header_bg)
            };
            if hovered && col.sortable {
                st = st.fg(t.text_primary).add_modifier(Modifier::UNDERLINED);
            }
            let ind = match self.sort {
                Some((c, SortDir::Asc)) if c == ci => " ▴",
                Some((c, SortDir::Desc)) if c == ci => " ▾",
                _ => "",
            };
            let ind_w = crate::ui::text::width(ind);
            let title = format!(
                "{}{}",
                crate::ui::text::truncate(&col.title, (r.width as usize).saturating_sub(ind_w)),
                ind
            );
            let text = match col.align {
                Align::Left => crate::ui::text::fit(&title, r.width as usize),
                Align::Right => crate::ui::text::fit_right(&title, r.width as usize),
            };
            buf.set_string(r.x, header_y, &text, st);
            if col.sortable {
                ctx.clickable(hid, Rect::new(r.x, header_y, r.width, 1));
            }
        }
        // horizontal overflow indicators
        if self.hscroll.overflows() {
            let more_right = self.hscroll.offset + self.hscroll.viewport_len < self.columns.len();
            let more_left = self.hscroll.offset > 0;
            if more_left {
                buf.set_string(area.x + 1, header_y, "…", t.faint().bg(header_bg));
            }
            if more_right {
                buf.set_string(
                    cols_area.right() + 1,
                    header_y,
                    "…",
                    t.faint().bg(header_bg),
                );
            }
        }

        if self.rows.is_empty() {
            let msg = crate::ui::text::truncate(&self.empty_text, area.width as usize);
            let y = body.y + body.height / 2;
            let x = body.x
                + body
                    .width
                    .saturating_sub(crate::ui::text::width(&msg) as u16)
                    / 2;
            buf.set_string(x, y, &msg, t.muted().bg(bg));
            return;
        }

        // rows
        for (i, di) in self.scroll.visible_range().enumerate() {
            let y = body.y + i as u16;
            let src = self.order[di];
            let row_rect = Rect::new(
                area.x,
                y,
                area.width.saturating_sub(if has_sb { 1 } else { 0 }),
                1,
            );
            let rid = self.row_id(di);
            let mut s = ctx.state(rid);
            // cell hover counts as row hover
            let hovered_cell =
                (0..self.columns.len()).find(|&c| ctx.interaction.hovered(self.cell_id(di, c)));
            if hovered_cell.is_some() {
                s.hovered = true;
            }
            s.focused = focused && di == self.cursor_row;
            s.selected = self.selected == Some(src);
            let row_style = t.row(s, bg);
            fill(buf, row_rect, row_style);
            buf.set_string(
                row_rect.x,
                y,
                "▎",
                t.gutter(s, row_style.bg.unwrap_or(bg), false),
            );
            let marker = if s.selected { "›" } else { " " };
            let ms = if s.selected {
                row_style.fg(if focused { t.accent } else { t.text_secondary })
            } else {
                row_style
            };
            buf.set_string(row_rect.x + 1, y, marker, ms);
            for (ci, col) in self.columns.iter().enumerate() {
                let r = self.col_rects[ci];
                if r.is_empty() {
                    continue;
                }
                let cell_rect = Rect::new(r.x, y, r.width, 1);
                let cell = &self.rows[src][ci];
                let is_cursor_cell = self.cell_nav && s.focused && ci == self.cursor_col;
                let is_edit = editing
                    && self
                        .edit
                        .as_ref()
                        .map(|e| e.row == di && e.col == ci)
                        .unwrap_or(false);
                let mut st = row_style;
                st = match cell.tone {
                    Tone::Normal => st,
                    other => st.fg(t.tone(other)),
                };
                if s.pressed {
                    st = row_style;
                }
                if is_edit {
                    let e = self.edit.as_ref().unwrap();
                    let es = t.field_style(crate::ui::ctx::VisualState {
                        editing: true,
                        ..Default::default()
                    });
                    fill(buf, cell_rect, es);
                    let text = e.buffer.text();
                    let cw = r.width.saturating_sub(1) as usize;
                    let cur = e.buffer.cursor_pos().col;
                    let off = cur.saturating_sub(cw.saturating_sub(1));
                    let mut shown: String = text.chars().skip(off).take(cw).collect();
                    if off > 0 {
                        shown.replace_range(..shown.chars().next().map_or(0, char::len_utf8), "…");
                    }
                    let mut ts = es
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                    if e.error.is_some() {
                        ts = ts.underline_color(t.error);
                    }
                    buf.set_string(r.x, y, &shown, ts);
                    ctx.set_cursor(Position::new(r.x + (cur - off) as u16, y));
                    if e.error.is_some() {
                        buf.set_string(
                            r.right().saturating_sub(1),
                            y,
                            "!",
                            es.fg(t.error).add_modifier(Modifier::BOLD),
                        );
                    }
                } else {
                    if is_cursor_cell {
                        // cell cursor: reversed on the cell only
                        st = Style::new()
                            .fg(t.canvas)
                            .bg(t.text_primary)
                            .add_modifier(Modifier::BOLD);
                        if cell.error.is_some() {
                            st = st.bg(t.error).fg(t.text_primary);
                        }
                    } else if cell.error.is_some() {
                        st = st.fg(t.error);
                    } else if col.editable && self.cell_nav && s.hovered && hovered_cell == Some(ci)
                    {
                        st = st
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(t.border_strong);
                    }
                    let text = match col.align {
                        Align::Left => crate::ui::text::fit(&cell.text, r.width as usize),
                        Align::Right => crate::ui::text::fit_right(&cell.text, r.width as usize),
                    };
                    buf.set_string(r.x, y, &text, st);
                    if cell.error.is_some() && !is_cursor_cell {
                        buf.set_string(
                            r.right().saturating_sub(1),
                            y,
                            "!",
                            st.add_modifier(Modifier::BOLD),
                        );
                    }
                }
                ctx.clickable(self.cell_id(di, ci), cell_rect);
            }
            ctx.clickable(rid, row_rect);
            // re-register cells on top so cell hover resolves before row
            for ci in 0..self.columns.len() {
                let r = self.col_rects[ci];
                if !r.is_empty() {
                    ctx.clickable(self.cell_id(di, ci), Rect::new(r.x, y, r.width, 1));
                }
            }
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, body.y, 1, body.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }

    /// Resolve which (display row, col) a widget id refers to.
    pub fn locate(&self, id: WidgetId) -> Option<(usize, Option<usize>)> {
        for di in self.scroll.visible_range() {
            if self.row_id(di) == id {
                return Some((di, None));
            }
            for ci in 0..self.columns.len() {
                if self.cell_id(di, ci) == id {
                    return Some((di, Some(ci)));
                }
            }
        }
        None
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == scrollbar::id_for(self.id)
            || self.locate(id).is_some()
            || self.locate_header(id).is_some()
    }

    pub fn locate_header(&self, id: WidgetId) -> Option<usize> {
        (0..self.columns.len()).find(|&c| self.header_id(c) == id)
    }

    pub fn edit_error(&self) -> Option<&str> {
        self.edit.as_ref().and_then(|e| e.error.as_deref())
    }
}

use ratatui::style::Style;

/// Numeric key with a total order: `None` for cells without a number.
fn parse_num(s: &str) -> Option<ordered::F64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|f| !f.is_nan())
        .map(ordered::F64)
}

mod ordered {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct F64(pub f64);
    impl Eq for F64 {}
    impl PartialOrd for F64 {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for F64 {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.total_cmp(&other.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> DataTable {
        let cols = vec![
            Column::new("Name", Constraint::Min(8)).editable(),
            Column::new("N", Constraint::Length(4)).right().editable(),
        ];
        let rows = vec![
            vec![Cell::new("beta"), Cell::new("2")],
            vec![Cell::new("alpha"), Cell::new("10")],
            vec![Cell::new("Gamma"), Cell::new("1")],
        ];
        DataTable::new(WidgetId::of("t"), cols, rows)
            .cell_nav(true)
            .numeric(&[1])
    }

    #[test]
    fn sort_cycles_asc_desc_none() {
        let mut t = table();
        t.sort_by(0);
        assert_eq!(t.sort, Some((0, SortDir::Asc)));
        assert_eq!(t.order, vec![1, 0, 2]);
        t.sort_by(0);
        assert_eq!(t.sort, Some((0, SortDir::Desc)));
        assert_eq!(t.order, vec![2, 0, 1]);
        t.sort_by(0);
        assert_eq!(t.sort, None);
        assert_eq!(t.order, vec![0, 1, 2]);
    }

    #[test]
    fn numeric_sort_is_not_lexicographic() {
        let mut t = table();
        t.sort_by(1);
        assert_eq!(t.order, vec![2, 0, 1]);
    }

    #[test]
    fn sort_keeps_cursor_on_same_row() {
        let mut t = table();
        t.cursor_row = 1; // alpha
        t.sort_by(0);
        assert_eq!(t.source_row(t.cursor_row), 1);
    }

    #[test]
    fn edit_commit_and_cancel() {
        let mut t = table();
        t.begin_edit();
        assert!(t.is_editing());
        let e = t.edit.as_mut().unwrap();
        e.buffer.select_all();
        e.buffer.insert_str("zeta");
        let ev = t.commit_edit();
        assert_eq!(ev, Some(TableEvent::Committed { row: 0, col: 0 }));
        assert_eq!(t.rows[0][0].text, "zeta");
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.insert_str("!!!");
        t.cancel_edit();
        assert_eq!(t.rows[0][0].text, "zeta");
    }

    #[test]
    fn validation_blocks_commit() {
        let mut t = table().validator(|col, s| {
            if col == 1 && s.parse::<u32>().is_err() {
                Some("Must be a number".into())
            } else {
                None
            }
        });
        t.cursor_col = 1;
        t.begin_edit();
        t.edit.as_mut().unwrap().buffer.insert_str("x");
        assert!(t.commit_edit().is_none());
        assert!(t.is_editing());
        assert_eq!(t.edit_error(), Some("Must be a number"));
    }

    #[test]
    fn tab_moves_to_next_editable_cell_and_leaves_at_end() {
        let mut t = table();
        t.begin_edit();
        let tab = Key {
            code: KeyCode::Tab,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let (_, ev) = t.on_key(&tab);
        assert_eq!(ev, Some(TableEvent::Committed { row: 0, col: 0 }));
        assert_eq!(t.cursor_col, 1);
        assert!(t.is_editing());
        let (_, ev) = t.on_key(&tab);
        assert_eq!(ev, Some(TableEvent::LeaveForward));
        assert!(!t.is_editing());
    }
}
