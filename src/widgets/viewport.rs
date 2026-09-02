//! Selectable read-only text viewport: styled lines with bounded retention,
//! tail-follow, optional wrapping, wheel/scrollbar scrolling, drag and word
//! selection, copy, and an optional caret exposed as the hardware cursor.
//! Serves log bodies and simulated terminals alike; it knows nothing about
//! what the text means.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::Tone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::width;
use crate::widgets::scrollbar;

/// One styled run. Tone maps through `Theme::tone`; bold/italic/underline
/// are the only modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Draw reversed (canvas on text-primary): a terminal's cursor block.
    pub reversed: bool,
}

impl Span {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            bold: false,
            italic: false,
            underline: false,
            reversed: false,
        }
    }
    pub fn plain(text: impl Into<String>) -> Self {
        Self::new(text, Tone::Normal)
    }
    pub fn muted(text: impl Into<String>) -> Self {
        Self::new(text, Tone::Muted)
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }
}

pub type Line = Vec<Span>;

pub fn line_text(line: &[Span]) -> String {
    line.iter().map(|s| s.text.as_str()).collect()
}

/// Logical position: line index and display column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellPos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportEvent {
    /// `y` with a selection: the owner puts the text on its clipboard.
    Copy(String),
    SelectionChanged,
    FollowChanged(bool),
}

#[derive(Debug, Clone)]
struct Cell {
    g: String,
    w: usize,
    tone: Tone,
    bold: bool,
    italic: bool,
    underline: bool,
    reversed: bool,
}

#[derive(Debug, Clone)]
struct VisualRow {
    line: usize,
    /// Cell index range within the logical line.
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub struct TextViewport {
    pub id: WidgetId,
    pub lines: Vec<Line>,
    pub max_lines: Option<usize>,
    pub scroll: ScrollState,
    pub follow: bool,
    pub wrap: bool,
    /// Caret exposed as the hardware cursor while following and focused.
    pub caret: Option<CellPos>,
    pub caret_visible: bool,
    pub area: Rect,
    selection: Option<(CellPos, CellPos)>,
    drag_anchor: Option<CellPos>,
    cells: Vec<Vec<Cell>>,
    visual: Vec<VisualRow>,
    layout_width: u16,
    dirty: bool,
}

impl TextViewport {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            lines: vec![],
            max_lines: None,
            scroll: ScrollState::default(),
            follow: true,
            wrap: false,
            caret: None,
            caret_visible: true,
            area: Rect::ZERO,
            selection: None,
            drag_anchor: None,
            cells: vec![],
            visual: vec![],
            layout_width: 0,
            dirty: true,
        }
    }

    pub fn with_lines(id: WidgetId, lines: Vec<Line>) -> Self {
        let mut v = Self::new(id);
        v.lines = lines;
        v
    }

    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    pub fn max_lines(mut self, n: usize) -> Self {
        self.max_lines = Some(n);
        self
    }

    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
        if let Some(max) = self.max_lines
            && self.lines.len() > max
        {
            let drop = self.lines.len() - max;
            self.lines.drain(..drop);
            if let Some((a, b)) = self.selection.as_mut() {
                a.line = a.line.saturating_sub(drop);
                b.line = b.line.saturating_sub(drop);
            }
            if let Some(c) = self.caret.as_mut() {
                c.line = c.line.saturating_sub(drop);
            }
        }
        self.dirty = true;
    }

    pub fn set_lines(&mut self, lines: Vec<Line>) {
        self.lines = lines;
        self.clamp_positions();
        self.dirty = true;
    }

    /// Replace the last line (a terminal updating its live row).
    pub fn replace_last(&mut self, line: Line) {
        if let Some(last) = self.lines.last_mut() {
            *last = line;
        } else {
            self.lines.push(line);
        }
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.selection = None;
        self.drag_anchor = None;
        self.caret = None;
        self.dirty = true;
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn clamp_positions(&mut self) {
        let n = self.lines.len();
        if n == 0 {
            self.selection = None;
            self.caret = None;
            return;
        }
        if let Some((a, b)) = self.selection.as_mut() {
            a.line = a.line.min(n - 1);
            b.line = b.line.min(n - 1);
        }
    }

    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        let (a, b) = self.selection?;
        if a == b {
            return None;
        }
        Some((a.min(b), a.max(b)))
    }

    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    pub fn clear_selection(&mut self) -> Outcome {
        if self.selection.take().is_some() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn is_at_tail(&self) -> bool {
        self.scroll.offset >= self.scroll.max_offset()
    }

    /// Lines behind the live tail (scrollback depth).
    pub fn scrollback_depth(&self) -> usize {
        self.scroll.max_offset().saturating_sub(self.scroll.offset)
    }

    pub fn set_follow(&mut self, on: bool) {
        self.follow = on;
        if on {
            self.scroll.jump_end();
        }
    }

    fn ensure_layout(&mut self, text_w: u16) {
        if !self.dirty && self.layout_width == text_w {
            return;
        }
        self.layout_width = text_w;
        self.dirty = false;
        self.cells = self
            .lines
            .iter()
            .map(|line| {
                let mut out = vec![];
                for sp in line {
                    for g in
                        unicode_segmentation::UnicodeSegmentation::graphemes(sp.text.as_str(), true)
                    {
                        let w = width(g).max(if g == "\t" { 4 } else { 0 });
                        if g == "\t" {
                            for _ in 0..4 {
                                out.push(Cell {
                                    g: " ".into(),
                                    w: 1,
                                    tone: sp.tone,
                                    bold: sp.bold,
                                    italic: sp.italic,
                                    underline: sp.underline,
                                    reversed: sp.reversed,
                                });
                            }
                            continue;
                        }
                        if w == 0 {
                            continue;
                        }
                        out.push(Cell {
                            g: g.to_owned(),
                            w,
                            tone: sp.tone,
                            bold: sp.bold,
                            italic: sp.italic,
                            underline: sp.underline,
                            reversed: sp.reversed,
                        });
                    }
                }
                out
            })
            .collect();
        let w = text_w.max(1) as usize;
        let mut visual = vec![];
        for (li, cells) in self.cells.iter().enumerate() {
            if !self.wrap || cells.is_empty() {
                visual.push(VisualRow {
                    line: li,
                    start: 0,
                    end: cells.len(),
                });
                continue;
            }
            let mut start = 0;
            let mut acc = 0;
            for (ci, c) in cells.iter().enumerate() {
                if acc + c.w > w && ci > start {
                    visual.push(VisualRow {
                        line: li,
                        start,
                        end: ci,
                    });
                    start = ci;
                    acc = 0;
                }
                acc += c.w;
            }
            visual.push(VisualRow {
                line: li,
                start,
                end: cells.len(),
            });
        }
        self.visual = visual;
        self.scroll.set_content(self.visual.len());
    }

    /// Column (display cells) of a cell index within a line.
    fn col_of(&self, line: usize, cell: usize) -> usize {
        self.cells
            .get(line)
            .map(|cs| cs.iter().take(cell).map(|c| c.w).sum())
            .unwrap_or(0)
    }

    /// Cell index at a display column.
    fn cell_at(&self, line: usize, col: usize) -> usize {
        let Some(cs) = self.cells.get(line) else {
            return 0;
        };
        let mut acc = 0;
        for (i, c) in cs.iter().enumerate() {
            if acc + c.w > col {
                return i;
            }
            acc += c.w;
        }
        cs.len()
    }

    /// Map a screen position to a logical position (line, column).
    pub fn pos_at(&self, pos: Position) -> Option<CellPos> {
        if self.area.is_empty() {
            return None;
        }
        let row = (pos.y.saturating_sub(self.area.y) as usize + self.scroll.offset)
            .min(self.visual.len().saturating_sub(1));
        let vr = self.visual.get(row)?;
        let x = pos.x.saturating_sub(self.area.x) as usize;
        let base = self.col_of(vr.line, vr.start);
        let line_w = self.col_of(vr.line, vr.end);
        Some(CellPos {
            line: vr.line,
            col: (base + x).min(line_w),
        })
    }

    pub fn selected_text(&self) -> Option<String> {
        let (a, b) = self.selection()?;
        let mut out = String::new();
        for li in a.line..=b.line {
            let Some(cs) = self.cells.get(li) else { break };
            let from = if li == a.line {
                self.cell_at(li, a.col)
            } else {
                0
            };
            let to = if li == b.line {
                self.cell_at(li, b.col)
            } else {
                cs.len()
            };
            let text: String = cs[from.min(cs.len())..to.min(cs.len())]
                .iter()
                .map(|c| c.g.as_str())
                .collect();
            out.push_str(text.trim_end());
            if li != b.line {
                out.push('\n');
            }
        }
        Some(out)
    }

    /// Double-click: select the word under the pointer.
    pub fn select_word_at(&mut self, pos: Position) -> Outcome {
        let Some(p) = self.pos_at(pos) else {
            return Outcome::Ignored;
        };
        let Some(cs) = self.cells.get(p.line) else {
            return Outcome::Ignored;
        };
        let ci = self.cell_at(p.line, p.col).min(cs.len().saturating_sub(1));
        if cs.is_empty() {
            return Outcome::Ignored;
        }
        let is_word = |c: &Cell| {
            c.g.chars()
                .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '/' || ch == '.')
        };
        if !is_word(&cs[ci]) {
            return Outcome::Consumed;
        }
        let mut s = ci;
        while s > 0 && is_word(&cs[s - 1]) {
            s -= 1;
        }
        let mut e = ci + 1;
        while e < cs.len() && is_word(&cs[e]) {
            e += 1;
        }
        let a = CellPos {
            line: p.line,
            col: self.col_of(p.line, s),
        };
        let b = CellPos {
            line: p.line,
            col: self.col_of(p.line, e),
        };
        self.selection = Some((a, b));
        self.drag_anchor = None;
        Outcome::Changed
    }

    /// Mouse down: anchor a drag; no selection yet.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        let had = self.selection.is_some();
        self.selection = None;
        self.drag_anchor = self.pos_at(pos);
        if had {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Drag: extend the selection from the anchor; auto-scroll at the
    /// vertical edges.
    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        let Some(anchor) = self.drag_anchor else {
            return Outcome::Ignored;
        };
        if pos.y < self.area.y {
            self.scroll.scroll_by(-1);
            self.follow = false;
        } else if pos.y >= self.area.bottom() {
            self.scroll.scroll_by(1);
        }
        let clamped = Position::new(
            pos.x
                .clamp(self.area.x, self.area.right().saturating_sub(1)),
            pos.y
                .clamp(self.area.y, self.area.bottom().saturating_sub(1)),
        );
        let Some(head) = self.pos_at(clamped) else {
            return Outcome::Consumed;
        };
        self.selection = Some((anchor, head));
        Outcome::Changed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.scroll.scroll_by(delta as isize);
        self.follow = self.is_at_tail();
        Outcome::Changed
    }

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        self.follow = self.is_at_tail();
        Outcome::Changed
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id)
    }

    /// ↑↓ j k · PgUp PgDn · Home End g G · `f` follow · `y` copy · Esc clears.
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ViewportEvent>) {
        let before = self.scroll.offset;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => self.scroll.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.scroll.scroll_by(1),
            KeyCode::PageUp => self.scroll.page_up(),
            KeyCode::PageDown => self.scroll.page_down(),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.scroll.jump_start(),
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll.jump_end();
                self.follow = true;
                return (Outcome::Changed, Some(ViewportEvent::FollowChanged(true)));
            }
            KeyCode::Char('f') if key.plain() => {
                self.follow = !self.follow;
                if self.follow {
                    self.scroll.jump_end();
                }
                return (
                    Outcome::Changed,
                    Some(ViewportEvent::FollowChanged(self.follow)),
                );
            }
            KeyCode::Char('y') if key.plain() => {
                return match self.selected_text() {
                    Some(t) => (Outcome::Changed, Some(ViewportEvent::Copy(t))),
                    None => (Outcome::Consumed, None),
                };
            }
            KeyCode::Esc => {
                return match self.clear_selection() {
                    Outcome::Changed => (Outcome::Changed, Some(ViewportEvent::SelectionChanged)),
                    _ => (Outcome::Ignored, None),
                };
            }
            _ => return (Outcome::Ignored, None),
        }
        if self.scroll.offset != before {
            self.follow = self.is_at_tail();
        }
        (Outcome::Changed, None)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        // lay out for the width without a scrollbar first; add one on overflow
        let mut text_w = area.width;
        self.ensure_layout(text_w);
        self.scroll.set_viewport(area.height as usize);
        let has_sb = self.scroll.overflows();
        if has_sb {
            text_w = area.width.saturating_sub(1);
            self.ensure_layout(text_w);
            self.scroll.set_viewport(area.height as usize);
        }
        if self.follow {
            self.scroll.jump_end();
        }
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        let sel = self.selection();
        fill(buf, area, Style::new().bg(bg));
        for (k, vi) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let vr = &self.visual[vi];
            let cells = &self.cells[vr.line];
            let mut x = area.x;
            let mut col = self.col_of(vr.line, vr.start);
            for c in &cells[vr.start..vr.end] {
                if x + c.w as u16 > area.x + text_w {
                    break;
                }
                let mut st = Style::new().fg(t.tone(c.tone)).bg(bg);
                if c.bold {
                    st = st.add_modifier(Modifier::BOLD);
                }
                if c.italic {
                    st = st.add_modifier(Modifier::ITALIC);
                }
                if c.underline {
                    st = st.add_modifier(Modifier::UNDERLINED);
                }
                if c.reversed {
                    st = Style::new().fg(t.canvas).bg(t.text_primary);
                }
                let p = CellPos { line: vr.line, col };
                if let Some((a, b)) = sel
                    && p >= a
                    && p < b
                {
                    st = t.selection().add_modifier(st.add_modifier);
                }
                buf.set_string(x, y, &c.g, st);
                x += c.w as u16;
                col += c.w;
            }
            // a selection that spans to the line end paints the trailing gap
            if let Some((a, b)) = sel
                && vr.line >= a.line
                && vr.line < b.line
                && vr.end == cells.len()
            {
                let tail = Rect::new(x, y, (area.x + text_w).saturating_sub(x).min(1), 1);
                fill(buf, tail, t.selection());
            }
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
        if self.follow
            && focused
            && self.caret_visible
            && let Some(c) = self.caret
            && let Some(vi) = self.visual.iter().position(|v| {
                v.line == c.line
                    && self.col_of(v.line, v.start) <= c.col
                    && (c.col < self.col_of(v.line, v.end)
                        || v.end == self.cells.get(v.line).map_or(0, Vec::len))
            })
            && self.scroll.visible_range().contains(&vi)
        {
            let vr = &self.visual[vi];
            let base = self.col_of(vr.line, vr.start);
            let cx = area.x + (c.col.saturating_sub(base) as u16).min(text_w.saturating_sub(1));
            let cy = area.y + (vi - self.scroll.offset) as u16;
            ctx.set_cursor(Position::new(cx, cy));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn lines(n: usize) -> Vec<Line> {
        (0..n)
            .map(|i| vec![Span::plain(format!("line {i} alpha beta"))])
            .collect()
    }

    fn render(v: &mut TextViewport, w: u16, h: u16) -> Buffer {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        v.render(Rect::new(0, 0, w, h), &mut buf, &mut ctx, t.canvas);
        buf
    }

    #[test]
    fn follows_tail_and_wheel_leaves_it() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(50));
        render(&mut v, 40, 10);
        assert!(v.is_at_tail());
        assert_eq!(v.scroll.offset, 40);
        v.on_wheel(-3);
        assert!(!v.follow);
        assert_eq!(v.scrollback_depth(), 3);
        v.on_key(&Key {
            code: KeyCode::End,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert!(v.follow);
    }

    #[test]
    fn drag_selects_and_copies_text() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(5));
        v.follow = false;
        render(&mut v, 40, 10);
        v.on_click(Position::new(0, 1));
        v.on_drag(Position::new(6, 2));
        assert_eq!(
            v.selected_text().as_deref(),
            Some("line 1 alpha beta\nline 2")
        );
        let (_, ev) = v.on_key(&Key {
            code: KeyCode::Char('y'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert!(matches!(ev, Some(ViewportEvent::Copy(t)) if t.starts_with("line 1")));
        v.select_word_at(Position::new(8, 0));
        assert_eq!(v.selected_text().as_deref(), Some("alpha"));
    }

    #[test]
    fn wraps_long_lines_and_bounds_retention() {
        let mut v = TextViewport::new(WidgetId::of("v")).wrap(true).max_lines(3);
        for i in 0..5 {
            v.push(vec![Span::plain(format!("{i} {}", "x".repeat(30)))]);
        }
        assert_eq!(v.len(), 3);
        render(&mut v, 20, 10);
        assert_eq!(v.visual.len(), 6, "each line wraps into two visual rows");
    }
}
