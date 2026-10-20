use std::ops::Range;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::edit::TextCmd;
use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::response::Response;
use crate::core::scroll::{HScroll, ScrollPane, ScrollState, Scrollable, TrackClick};
use crate::theme::Tone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::edit_menu::{
    EditCaps, EditMenu, EditMenuHost, edit_menu_key, edit_menu_pointer, edit_menu_route,
};
use crate::ui::text::width;
use crate::widgets::field_common::PointerSelect;
use crate::widgets::scrollbar;
use crate::widgets::text_select::{self, TextSelect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub tone: Tone,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,

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

pub fn render_spans(
    buf: &mut ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    max: u16,
    t: &crate::theme::Theme,
    line: &[Span],
    bg: Color,
) -> u16 {
    let full = line_text(line);
    let shown = crate::ui::text::truncate(&full, max as usize);
    let w = width(&shown) as u16;
    if w == 0 {
        return 0;
    }
    buf.set_string(x, y, &shown, Style::new());
    let all = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::REVERSED;
    let mut at = x;
    for s in line {
        let sw = width(&s.text) as u16;
        let to = at.saturating_add(sw).min(x + w);
        if to > at {
            let mut m = Modifier::empty();
            if s.bold {
                m |= Modifier::BOLD;
            }
            if s.italic {
                m |= Modifier::ITALIC;
            }
            if s.underline {
                m |= Modifier::UNDERLINED;
            }
            if s.reversed {
                m |= Modifier::REVERSED;
            }
            let st = Style::new()
                .fg(t.tone_on(s.tone, bg))
                .add_modifier(m)
                .remove_modifier(all - m);
            buf.set_style(ratatui::layout::Rect::new(at, y, to - at, 1), st);
        }
        at = at.saturating_add(sw);
    }
    w
}

pub fn wrap_spans(line: &[Span], w: usize) -> Vec<Line> {
    let full: Vec<(char, usize)> = line
        .iter()
        .enumerate()
        .flat_map(|(i, s)| s.text.chars().map(move |c| (c, i)))
        .collect();
    let text: String = full.iter().map(|(c, _)| c).collect();
    let mut at = 0;
    let mut out = Vec::new();
    for part in crate::ui::text::wrap_with(&text, w, crate::ui::text::WrapOpts::PLAIN) {
        let mut pieces: Line = Vec::new();
        for pc in part.chars() {
            let found = full[at..].iter().position(|(c, _)| *c == pc);
            let si = match found {
                Some(k) => {
                    at += k + 1;
                    full[at - 1].1
                }

                None => full[at.saturating_sub(1).min(full.len().saturating_sub(1))].1,
            };
            let src = &line[si];
            match pieces.last_mut() {
                Some(p) if same_style(p, src) => p.text.push(pc),
                _ => pieces.push(Span {
                    text: pc.to_string(),
                    ..src.clone()
                }),
            }
        }
        out.push(pieces);
    }
    out
}

fn same_style(a: &Span, b: &Span) -> bool {
    a.tone == b.tone
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.reversed == b.reversed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellPos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    pub line: usize,
    pub range: Range<usize>,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportAction {
    Copy(String),
    SelectionChanged,
    FollowChanged(bool),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkCounters {
    pub segmented_lines: usize,

    pub reflowed_lines: usize,

    pub index_rebuilds: usize,

    pub index_extends: usize,

    pub index_shifts: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellStyle {
    tone: Tone,
    bold: bool,
    italic: bool,
    underline: bool,
    reversed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Text,

    TabPad,

    Control,
}

#[derive(Debug, Clone)]
struct Cell {
    g: String,
    w: usize,

    src: Range<usize>,
    style: CellStyle,
    kind: CellKind,
}

type RowCache = ((u16, bool), Vec<(usize, usize)>);

#[derive(Debug, Clone)]
struct Logical {
    id: u64,
    spans: Line,

    text: String,
    cells: Vec<Cell>,

    width: usize,

    segmented: bool,

    rows: Vec<RowCache>,
}

impl Logical {
    fn new(id: u64, spans: Line) -> Self {
        let text = line_text(&spans);
        Self {
            id,
            spans,
            text,
            cells: vec![],
            width: 0,
            segmented: false,
            rows: vec![],
        }
    }

    fn set(&mut self, spans: Line) {
        self.text = line_text(&spans);
        self.spans = spans;
        self.segmented = false;
        self.rows.clear();
    }

    fn has_rows(&self, key: (u16, bool)) -> bool {
        self.rows.iter().any(|(k, _)| *k == key)
    }

    fn rows_at(&self, key: (u16, bool)) -> &[(usize, usize)] {
        self.rows
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, r)| r.as_slice())
            .unwrap_or(&[])
    }

    fn segment(&mut self) {
        let mut cells: Vec<Cell> = vec![];

        let mut bounds: Vec<(usize, CellStyle)> = vec![];
        let mut at = 0;
        for sp in &self.spans {
            bounds.push((
                at,
                CellStyle {
                    tone: sp.tone,
                    bold: sp.bold,
                    italic: sp.italic,
                    underline: sp.underline,
                    reversed: sp.reversed,
                },
            ));
            at += sp.text.len();
        }
        let style_at = |b: usize| -> CellStyle {
            bounds
                .iter()
                .rev()
                .find(|(s, _)| *s <= b)
                .map(|(_, st)| *st)
                .unwrap_or(CellStyle {
                    tone: Tone::Normal,
                    bold: false,
                    italic: false,
                    underline: false,
                    reversed: false,
                })
        };
        for (start, g) in self.text.grapheme_indices(true) {
            let src = start..start + g.len();
            let style = style_at(start);
            if g == "\t" {
                for _ in 0..crate::ui::text::TAB_SPACES.len() {
                    cells.push(Cell {
                        g: " ".into(),
                        w: 1,
                        src: src.clone(),
                        style,
                        kind: CellKind::TabPad,
                    });
                }
                continue;
            }
            let first = g.chars().next().unwrap_or(' ');
            if g.chars().count() == 1 && (first.is_control() || first == '\u{7f}') {
                let shown = if first == '\u{7f}' {
                    "^?".to_owned()
                } else {
                    format!("^{}", ((first as u8) + 0x40) as char)
                };
                cells.push(Cell {
                    g: shown,
                    w: 2,
                    src,
                    style,
                    kind: CellKind::Control,
                });
                continue;
            }
            let w = width(g);
            if w == 0 {
                if let Some(prev) = cells.last_mut() {
                    prev.g.push_str(g);
                    prev.src.end = src.end;
                    continue;
                }
                cells.push(Cell {
                    g: g.to_owned(),
                    w: 1,
                    src,
                    style,
                    kind: CellKind::Text,
                });
                continue;
            }
            cells.push(Cell {
                g: g.to_owned(),
                w,
                src,
                style,
                kind: CellKind::Text,
            });
        }
        self.width = cells.iter().map(|c| c.w).sum();
        self.cells = cells;
        self.segmented = true;
        self.rows.clear();
    }

    fn reflow(&mut self, text_w: u16, wrap: bool) {
        let w = text_w.max(1) as usize;
        let mut rows = vec![];
        if !wrap || self.cells.is_empty() {
            rows.push((0, self.cells.len()));
        } else {
            let mut start = 0;
            let mut acc = 0;
            for (ci, c) in self.cells.iter().enumerate() {
                if acc + c.w > w && ci > start {
                    rows.push((start, ci));
                    start = ci;
                    acc = 0;
                }
                acc += c.w;
            }
            rows.push((start, self.cells.len()));
        }
        if self.rows.len() >= 2 {
            self.rows.remove(0);
        }
        self.rows.push(((text_w, wrap), rows));
    }

    fn col_of(&self, cell: usize) -> usize {
        self.cells.iter().take(cell).map(|c| c.w).sum()
    }

    fn cell_at(&self, col: usize) -> usize {
        let mut acc = 0;
        for (i, c) in self.cells.iter().enumerate() {
            if acc + c.w > col {
                return i;
            }
            acc += c.w;
        }
        self.cells.len()
    }

    fn total_width(&self) -> usize {
        self.width
    }
}

#[derive(Debug, Clone, Copy)]
struct VisualRow {
    line: usize,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Anchor {
    id: u64,
    col: usize,
}

#[derive(Debug, Clone)]
pub struct TextViewport {
    pub id: WidgetId,

    pub scroll: ScrollPane,
    pub follow: bool,
    pub wrap: bool,

    pub caret: Option<CellPos>,
    pub caret_visible: bool,
    pub area: Rect,
    lines: Vec<Logical>,
    max_lines: Option<usize>,
    next_id: u64,
    revision: u64,

    select: TextSelect<Anchor>,

    browse: Option<Anchor>,
    marks: Vec<Mark>,
    visual: Vec<VisualRow>,
    layout_width: u16,
    layout_wrap: bool,
    index_valid: bool,

    indexed_lines: usize,

    layout_size: Option<(u16, u16, bool, bool)>,

    rows_h: u16,

    reading: Option<(u64, usize)>,
    work: WorkCounters,

    pub scrollbar_gap: u16,

    pub horizontal: Option<HScroll>,

    pub nav: NavKeys,

    pub keymap: KeyMap<ViewportCmd>,

    pub pointer_select: PointerSelect,

    pub edit_menu: EditMenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportCmd {
    ToggleFollow,

    Copy,

    SelectAll,
}

impl Actions for ViewportCmd {
    fn label(self) -> &'static str {
        match self {
            ViewportCmd::ToggleFollow => "Follow",
            ViewportCmd::Copy => "Copy",
            ViewportCmd::SelectAll => "Select all",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        let mut map = vec![
            (KeyBinding::char('f'), ViewportCmd::ToggleFollow),
            (KeyBinding::cmd('c'), ViewportCmd::Copy),
            (KeyBinding::char('y'), ViewportCmd::Copy),
        ];
        map.extend(crate::core::keymap::select_all_bindings(
            ViewportCmd::SelectAll,
        ));
        map
    }
}

impl TextViewport {
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<ViewportCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            scroll: ScrollPane::default(),
            follow: true,
            wrap: false,
            caret: None,
            caret_visible: true,
            area: Rect::ZERO,
            lines: vec![],
            max_lines: None,
            next_id: 1,
            revision: 0,
            select: TextSelect::default(),
            pointer_select: PointerSelect::DEFAULT,
            edit_menu: EditMenu::default(),
            browse: None,
            marks: vec![],
            visual: vec![],
            layout_width: 0,
            layout_wrap: false,
            index_valid: false,
            indexed_lines: 0,
            layout_size: None,
            reading: None,
            work: WorkCounters::default(),
            scrollbar_gap: scrollbar::GAP,
            rows_h: 0,
            horizontal: None,
            nav: NavKeys::default(),
            keymap: ViewportCmd::default_keymap(),
        }
    }

    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    pub fn pointer_select(mut self, policy: PointerSelect) -> Self {
        self.pointer_select = policy;
        self
    }

    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.scroll.track_click = Some(click);
        self
    }

    pub fn h_enabled(&self) -> bool {
        !self.wrap && self.scroll.h_cfg.enabled
    }

    fn hoff(&self) -> usize {
        if self.h_enabled() {
            self.scroll.h.offset
        } else {
            0
        }
    }

    fn text_w(&self) -> usize {
        self.layout_width as usize
    }

    fn text_rect(&self) -> Rect {
        let h = if self.rows_h == 0 {
            self.area.height
        } else {
            self.rows_h.min(self.area.height)
        };
        Rect::new(self.area.x, self.area.y, self.area.width, h)
    }

    pub fn reveal_cols(&mut self, cols: Range<usize>) -> bool {
        if !self.h_enabled() {
            return false;
        }
        let before = self.scroll.h.offset;
        if cols.end > cols.start {
            self.scroll.h.ensure_visible(cols.end - 1);
        }
        self.scroll.h.ensure_visible(cols.start);
        self.scroll.h.offset != before
    }

    fn cols_of_bytes(&self, line: usize, range: &Range<usize>) -> Option<Range<usize>> {
        let l = self.lines.get(line)?;
        let mut col = 0;
        let mut start = None;
        let mut end = 0;
        for c in &l.cells {
            if c.src.start < range.end && range.start < c.src.end {
                start.get_or_insert(col);
                end = col + c.w;
            }
            col += c.w;
        }
        start.map(|s| s..end)
    }

    pub fn reveal_mark(&mut self, line: usize, range: Range<usize>) {
        self.reveal_line(line);
        if let Some(cols) = self.cols_of_bytes(line, &range) {
            self.reveal_cols(cols);
        }
    }

    fn scroll_h(&mut self, delta: isize) -> Outcome {
        self.wheel(Axis::Horizontal, delta as i32).outcome()
    }

    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    pub fn with_lines(id: WidgetId, lines: Vec<Line>) -> Self {
        let mut v = Self::new(id);
        v.set_lines(lines);
        v
    }

    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    pub fn max_lines(mut self, n: usize) -> Self {
        self.set_max_lines(Some(n));
        self
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn work_counters(&self) -> WorkCounters {
        self.work
    }

    pub fn retention(&self) -> Option<usize> {
        self.max_lines
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter().map(|l| &l.spans)
    }

    pub fn line(&self, i: usize) -> Option<&Line> {
        self.lines.get(i).map(|l| &l.spans)
    }

    pub fn line_text(&self, i: usize) -> Option<&str> {
        self.lines.get(i).map(|l| l.text.as_str())
    }

    pub fn last(&self) -> Option<&Line> {
        self.lines.last().map(|l| &l.spans)
    }

    fn alloc(&mut self, spans: Line) -> Logical {
        let id = self.next_id;
        self.next_id += 1;
        Logical::new(id, spans)
    }

    fn touched(&mut self) {
        self.revision += 1;
        self.index_valid = false;
    }

    fn appended(&mut self) {
        self.revision += 1;
    }

    fn shift_index(&mut self, drop: usize) {
        if !self.index_valid {
            return;
        }
        let gone = self.visual.iter().take_while(|v| v.line < drop).count();
        self.visual.drain(..gone);
        for v in &mut self.visual {
            v.line -= drop;
        }
        self.indexed_lines = self.indexed_lines.saturating_sub(drop);
        self.work.index_shifts += 1;
        self.scroll.set_content(self.visual.len());
        if !self.follow {
            self.scroll.offset = self.scroll.offset.saturating_sub(gone);
        }
    }

    fn enforce_cap(&mut self) -> usize {
        let Some(max) = self.max_lines else {
            return 0;
        };
        if self.lines.len() <= max {
            return 0;
        }
        let drop = self.lines.len() - max;
        let evicted: Vec<u64> = self.lines.drain(..drop).map(|l| l.id).collect();
        self.reconcile(drop, &evicted);
        self.shift_index(drop);
        drop
    }

    fn reconcile(&mut self, drop: usize, evicted: &[u64]) {
        let gone = |id: u64| evicted.contains(&id);

        if let Some((a, b)) = self.select.sel {
            let (lo, hi) = if (a.id, a.col) <= (b.id, b.col) {
                (a, b)
            } else {
                (b, a)
            };
            if gone(hi.id) {
                self.select.sel = None;
            } else if gone(lo.id) {
                let first = self.lines.first().map(|l| l.id).unwrap_or(hi.id);
                let clipped = Anchor { id: first, col: 0 };
                self.select.sel = Some(if a.id == lo.id {
                    (clipped, b)
                } else {
                    (a, clipped)
                });
                if self.select.sel.is_some_and(|(x, y)| x == y) {
                    self.select.sel = None;
                }
            }
        }
        if self
            .select
            .word
            .is_some_and(|(a, b)| gone(a.id) || gone(b.id))
        {
            self.select.word = None;
        }
        if self.select.press.is_some_and(|d| gone(d.id)) {
            self.select.press = self.lines.first().map(|l| Anchor { id: l.id, col: 0 });
        }
        if self.browse.is_some_and(|b| gone(b.id)) {
            self.browse = None;
        }
        if let Some(c) = self.caret {
            if c.line < drop {
                self.caret = None;
            } else {
                self.caret = Some(CellPos {
                    line: c.line - drop,
                    col: c.col,
                });
            }
        }
        if self.reading.is_some_and(|(id, _)| gone(id)) {
            self.reading = self.lines.first().map(|l| (l.id, 0));
        }
        self.marks.retain(|m| m.line >= drop);
        for m in &mut self.marks {
            m.line -= drop;
        }
    }

    pub fn set_lines(&mut self, lines: Vec<Line>) {
        let mut fresh = Vec::with_capacity(lines.len());
        for l in lines {
            let lg = self.alloc(l);
            fresh.push(lg);
        }
        self.lines = fresh;
        self.select.sel = None;
        self.select.press = None;
        self.select.word = None;
        self.browse = None;
        self.caret = None;
        self.reading = None;
        self.marks.clear();
        self.touched();
        self.enforce_cap();
    }

    pub fn push(&mut self, line: Line) {
        let lg = self.alloc(line);
        self.lines.push(lg);
        self.appended();
        self.enforce_cap();
    }

    pub fn extend(&mut self, lines: Vec<Line>) {
        if lines.is_empty() {
            return;
        }
        for l in lines {
            let lg = self.alloc(l);
            self.lines.push(lg);
        }
        self.appended();
        self.enforce_cap();
    }

    pub fn replace_last(&mut self, line: Line) {
        match self.lines.last_mut() {
            Some(last) => {
                if last.spans == line {
                    return;
                }
                last.set(line);
                self.touched();
            }
            None => self.push(line),
        }
    }

    pub fn pop(&mut self) -> Option<Line> {
        let l = self.lines.pop()?;
        let evicted = [l.id];
        self.reconcile_removed_tail(&evicted, self.lines.len());
        self.touched();
        Some(l.spans)
    }

    fn reconcile_removed_tail(&mut self, evicted: &[u64], new_len: usize) {
        let gone = |id: u64| evicted.contains(&id);
        if self
            .select
            .sel
            .is_some_and(|(a, b)| gone(a.id) || gone(b.id))
        {
            self.select.sel = None;
        }
        if self
            .select
            .word
            .is_some_and(|(a, b)| gone(a.id) || gone(b.id))
        {
            self.select.word = None;
        }
        if self.select.press.is_some_and(|d| gone(d.id)) {
            self.select.press = None;
        }
        if self.browse.is_some_and(|b| gone(b.id)) {
            self.browse = None;
        }
        if self.caret.is_some_and(|c| c.line >= new_len) {
            self.caret = None;
        }
        if self.reading.is_some_and(|(id, _)| gone(id)) {
            self.reading = self.lines.last().map(|l| (l.id, 0));
        }
        self.marks.retain(|m| m.line < new_len);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.select.sel = None;
        self.select.press = None;
        self.select.word = None;
        self.browse = None;
        self.caret = None;
        self.reading = None;
        self.marks.clear();
        self.touched();
    }

    pub fn set_max_lines(&mut self, n: Option<usize>) {
        self.max_lines = n;
        if self.enforce_cap() > 0 {
            self.touched();
        }
    }

    pub fn set_marks(&mut self, marks: Vec<Mark>) {
        self.marks = marks;
    }

    pub fn clear_marks(&mut self) {
        self.marks.clear();
    }

    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    pub fn reveal_line(&mut self, line: usize) {
        self.ensure_index();
        if let Some(vi) = self.visual.iter().position(|v| v.line == line) {
            self.follow = false;
            self.scroll.ensure_visible(vi);
            self.remember_reading();
        }
        let current = self
            .marks
            .iter()
            .find(|m| m.line == line && m.current)
            .map(|m| m.range.clone());
        if let Some(cols) = current.and_then(|r| self.cols_of_bytes(line, &r)) {
            self.reveal_cols(cols);
        }
    }

    fn ensure_layout(&mut self, text_w: u16) {
        let wrap = self.wrap;
        let same_key = self.layout_width == text_w && self.layout_wrap == wrap;
        if self.index_valid && same_key {
            let from = self.indexed_lines.min(self.lines.len());
            let key = (text_w, wrap);
            let mut extended = false;
            for li in from..self.lines.len() {
                let l = &mut self.lines[li];
                if !l.segmented {
                    l.segment();
                    self.work.segmented_lines += 1;
                }
                if !l.has_rows(key) {
                    l.reflow(text_w, wrap);
                    self.work.reflowed_lines += 1;
                }
                for &(s, e) in self.lines[li].rows_at(key) {
                    self.visual.push(VisualRow {
                        line: li,
                        start: s,
                        end: e,
                    });
                }
                extended = true;
            }
            self.indexed_lines = self.lines.len();
            if extended {
                self.work.index_extends += 1;
                self.scroll.set_content(self.visual.len());
                self.clip_selection();
            }
            return;
        }
        for l in &mut self.lines {
            if !l.segmented {
                l.segment();
                self.work.segmented_lines += 1;
            }
            if !l.has_rows((text_w, wrap)) {
                l.reflow(text_w, wrap);
                self.work.reflowed_lines += 1;
            }
        }
        self.layout_width = text_w;
        self.layout_wrap = wrap;
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        let key = (self.layout_width, self.layout_wrap);
        let mut visual = Vec::with_capacity(self.lines.len());
        for (li, l) in self.lines.iter().enumerate() {
            for &(s, e) in l.rows_at(key) {
                visual.push(VisualRow {
                    line: li,
                    start: s,
                    end: e,
                });
            }
        }
        self.visual = visual;
        self.index_valid = true;
        self.indexed_lines = self.lines.len();
        self.work.index_rebuilds += 1;
        self.scroll.set_content(self.visual.len());
        self.clip_selection();
    }

    fn clip_selection(&mut self) {
        if let Some((a, b)) = self.select.sel {
            let clip = |x: Anchor, lines: &[Logical]| -> Anchor {
                match lines.iter().find(|l| l.id == x.id) {
                    Some(l) => Anchor {
                        id: x.id,
                        col: x.col.min(l.total_width()),
                    },
                    None => x,
                }
            };
            let na = clip(a, &self.lines);
            let nb = clip(b, &self.lines);
            self.select.sel = if na == nb { None } else { Some((na, nb)) };
        }
        if !self.follow {
            self.restore_reading();
        }
    }

    fn ensure_index(&mut self) {
        if !self.index_valid || self.indexed_lines < self.lines.len() {
            let w = if self.layout_width == 0 {
                self.area.width.max(1)
            } else {
                self.layout_width
            };
            self.ensure_layout(w);
        }
    }

    fn remember_reading(&mut self) {
        if self.follow || !self.index_valid {
            self.reading = None;
            return;
        }
        let Some(vr) = self.visual.get(self.scroll.offset) else {
            self.reading = None;
            return;
        };
        let Some(line) = self.lines.get(vr.line) else {
            self.reading = None;
            return;
        };
        let key = (self.layout_width, self.layout_wrap);
        let sub = line
            .rows_at(key)
            .iter()
            .position(|&(s, _)| s == vr.start)
            .unwrap_or(0);
        self.reading = Some((line.id, sub));
    }

    fn restore_reading(&mut self) {
        let Some((id, sub)) = self.reading else {
            return;
        };
        let Some(li) = self.lines.iter().position(|l| l.id == id) else {
            self.scroll.jump_start();
            return;
        };
        let key = (self.layout_width, self.layout_wrap);
        let rows = self.lines[li].rows_at(key);
        let sub = sub.min(rows.len().saturating_sub(1));
        let start = rows.get(sub).map(|r| r.0).unwrap_or(0);
        let target = self
            .visual
            .iter()
            .position(|v| v.line == li && v.start == start)
            .unwrap_or(0);
        self.scroll.scroll_to(target);
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
        let h_on = self.h_enabled();
        let size = (area.width, area.height, self.wrap, h_on);
        let dirty = !self.index_valid || self.indexed_lines < self.lines.len();
        if dirty || self.layout_size != Some(size) {
            let mut rows = area.height;
            self.lay_out(area.width, rows);
            if h_on {
                self.sync_h();
                if area.height > 1 && self.scroll.h_cfg.shows_bar(&self.scroll.h) {
                    rows = area.height - 1;
                    self.lay_out(area.width, rows);
                    self.sync_h();
                }
            } else {
                self.scroll.h = ScrollState::default();
            }
            self.rows_h = rows;
            self.layout_size = Some(size);

            if rows < area.height && h_on {
                let w = area
                    .width
                    .saturating_sub(u16::from(self.scroll.overflows()));
                let track = Rect::new(area.x, area.y + rows, w, 1);
                self.scroll.set_track(self.id, Axis::Horizontal, track);
            } else {
                self.scroll.clear_track(Axis::Horizontal);
            }
        } else {
            self.scroll.set_viewport(self.rows_h as usize);
        }
        if self.follow {
            self.scroll.jump_end();
        }
    }

    fn sync_h(&mut self) {
        let widest = self.lines.iter().map(|l| l.width).max().unwrap_or(0);
        self.scroll.h.set_content(widest);
        self.scroll.h.set_viewport(self.text_w());
    }

    fn lay_out(&mut self, width: u16, rows: u16) {
        let narrow = width.saturating_sub(scrollbar::reserve(self.scrollbar_gap));
        let preferred = if self.layout_width == narrow && self.wrap == self.layout_wrap {
            narrow
        } else {
            width
        };
        self.ensure_layout(preferred);
        self.scroll.set_viewport(rows as usize);
        if self.scroll.overflows() && preferred == width {
            self.ensure_layout(narrow);
        } else if !self.scroll.overflows() && preferred == narrow {
            self.ensure_layout(width);
            self.scroll.set_viewport(rows as usize);
            if self.scroll.overflows() {
                self.ensure_layout(narrow);
            }
        }
    }

    fn index_of(&self, id: u64) -> Option<usize> {
        self.lines.iter().position(|l| l.id == id)
    }

    fn resolve(&self, a: Anchor) -> Option<CellPos> {
        let line = self.index_of(a.id)?;
        Some(CellPos { line, col: a.col })
    }

    fn anchor_at(&self, p: CellPos) -> Option<Anchor> {
        let l = self.lines.get(p.line)?;
        Some(Anchor {
            id: l.id,
            col: p.col,
        })
    }

    pub fn has_anchor(&self) -> bool {
        self.select.press.is_some()
    }

    pub fn has_selection(&self) -> bool {
        self.selection().is_some()
    }

    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        let (a, b) = self.select.sel?;
        let a = self.resolve(a)?;
        let b = self.resolve(b)?;
        if a == b {
            return None;
        }
        Some((a.min(b), a.max(b)))
    }

    pub fn select_all(&mut self) -> Outcome {
        self.ensure_index();
        let (Some(first), Some(last)) = (self.lines.first(), self.lines.last()) else {
            return Outcome::Consumed;
        };
        let all = (
            Anchor {
                id: first.id,
                col: 0,
            },
            Anchor {
                id: last.id,
                col: last.col_of(last.cells.len()),
            },
        );
        if self.select.sel == Some(all) {
            return Outcome::Consumed;
        }
        self.select.set(all.0, all.1);
        self.browse = Some(all.1);
        Outcome::Changed
    }

    pub fn clear_selection(&mut self) -> Outcome {
        self.browse = None;
        self.select.clear()
    }

    pub fn is_at_tail(&self) -> bool {
        self.scroll.offset >= self.scroll.max_offset()
    }

    pub fn scrollback_depth(&self) -> usize {
        self.scroll.max_offset().saturating_sub(self.scroll.offset)
    }

    pub fn set_follow(&mut self, on: bool) {
        self.follow = on;
        if on {
            self.scroll.jump_end();
            self.reading = None;
        } else {
            self.remember_reading();
        }
    }

    fn col_of(&self, line: usize, cell: usize) -> usize {
        self.lines.get(line).map(|l| l.col_of(cell)).unwrap_or(0)
    }

    fn cell_at(&self, line: usize, col: usize) -> usize {
        self.lines.get(line).map(|l| l.cell_at(col)).unwrap_or(0)
    }

    pub fn pos_at(&self, pos: Position) -> Option<CellPos> {
        if self.area.is_empty() || self.visual.is_empty() {
            return None;
        }
        let text = self.text_rect();
        let y = pos.y.min(text.bottom().saturating_sub(1));
        let row = (y.saturating_sub(self.area.y) as usize + self.scroll.offset)
            .min(self.visual.len().saturating_sub(1));
        let vr = self.visual.get(row)?;
        let x = pos.x.saturating_sub(self.area.x) as usize + self.hoff();
        let base = self.col_of(vr.line, vr.start);
        let line_w = self.col_of(vr.line, vr.end);
        Some(CellPos {
            line: vr.line,
            col: (base + x).min(line_w),
        })
    }

    fn source_slice(&self, line: usize, from: usize, to: usize) -> String {
        let Some(l) = self.lines.get(line) else {
            return String::new();
        };
        let from = from.min(l.cells.len());
        let to = to.min(l.cells.len());
        if from >= to {
            return String::new();
        }
        let start = l.cells[from].src.start;
        let end = l.cells[to - 1].src.end;
        l.text[start..end].to_owned()
    }

    pub fn selected_text(&self) -> Option<String> {
        let (a, b) = self.selection()?;
        let mut out = String::new();
        for li in a.line..=b.line {
            let Some(l) = self.lines.get(li) else { break };
            let from = if li == a.line {
                self.cell_at(li, a.col)
            } else {
                0
            };
            let to = if li == b.line {
                self.cell_at(li, b.col)
            } else {
                l.cells.len()
            };
            let text = self.source_slice(li, from, to);
            out.push_str(text.trim_end_matches(' '));
            if li != b.line {
                out.push('\n');
            }
        }
        Some(out)
    }

    fn is_word(c: &Cell) -> bool {
        c.kind == CellKind::Text && text_select::is_word(&c.g)
    }

    fn word_bounds(&self, line: usize, ci: usize) -> Option<(usize, usize)> {
        text_select::word_bounds(&self.lines.get(line)?.cells, ci, Self::is_word)
    }

    fn word_at(&self, p: CellPos) -> Option<(Anchor, Anchor)> {
        let ci = self.cell_at(p.line, p.col);
        let (s, e) = self.word_bounds(p.line, ci)?;
        let a = CellPos {
            line: p.line,
            col: self.col_of(p.line, s),
        };
        let b = CellPos {
            line: p.line,
            col: self.col_of(p.line, e),
        };
        self.anchor_at(a).zip(self.anchor_at(b))
    }

    fn line_at(&self, p: CellPos) -> Option<(Anchor, Anchor)> {
        let l = self.lines.get(p.line)?;
        (!l.cells.is_empty()).then_some((
            Anchor { id: l.id, col: 0 },
            Anchor {
                id: l.id,
                col: l.total_width(),
            },
        ))
    }

    fn follow_head(&mut self) {
        if let Some((_, head)) = self.select.sel {
            self.browse = Some(head);
        }
    }

    pub fn select_word_at(&mut self, pos: Position) -> Outcome {
        let Some(p) = self.pos_at(pos) else {
            return Outcome::Ignored;
        };
        let Some((a, b)) = self.word_at(p) else {
            return if self.lines.get(p.line).is_some_and(|l| l.cells.is_empty()) {
                Outcome::Ignored
            } else {
                Outcome::Consumed
            };
        };
        self.select.set(a, b);
        self.browse = Some(b);
        Outcome::Changed
    }

    pub fn on_click(&mut self, pos: Position) -> Outcome {
        if self.select.dragged.take() == Some(pos) && self.select.sel.is_some() {
            return Outcome::Consumed;
        }
        self.on_press_with(pos, 1, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    pub fn on_press_with(
        &mut self,
        pos: Position,
        count: u8,
        mods: ratatui::crossterm::event::KeyModifiers,
    ) -> Outcome {
        let p = self.pos_at(pos);
        let at = p.and_then(|p| self.anchor_at(p));
        let word = p.and_then(|p| self.word_at(p));
        let line = p.and_then(|p| self.line_at(p));
        self.browse = None;
        let o = self
            .select
            .press(self.pointer_select, at, count, mods, word, line);
        self.follow_head();
        o
    }

    pub fn on_click_with(&mut self, pos: Position, count: u8) -> Outcome {
        let p = self.pos_at(pos);
        let word = p.and_then(|p| self.word_at(p));
        let line = p.and_then(|p| self.line_at(p));
        let o = self
            .select
            .click(self.pointer_select, pos, count, word, line);
        self.follow_head();
        o
    }

    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        if self.select.press.is_none() {
            return Outcome::Ignored;
        }
        let text = self.text_rect();

        if pos.y < text.y {
            self.scroll.scroll_by(-((text.y - pos.y) as isize));
            self.follow = false;
            self.remember_reading();
        } else if pos.y >= text.bottom() {
            self.scroll.scroll_by((pos.y + 1 - text.bottom()) as isize);
            self.remember_reading();
        }

        let text_right = if self.h_enabled() {
            self.area.x + (self.text_w() as u16).min(self.area.width)
        } else {
            self.area.right()
        };
        if self.h_enabled() {
            if pos.x < self.area.x {
                self.scroll_h(-1);
            } else if pos.x >= text_right {
                self.scroll_h(1);
            }
        }
        let clamped = Position::new(
            pos.x.clamp(self.area.x, text_right.saturating_sub(1)),
            pos.y.clamp(text.y, text.bottom().saturating_sub(1)),
        );
        let p = self.pos_at(clamped);
        let head = p.and_then(|p| self.anchor_at(p));
        let word = if self.select.by_word() {
            p.and_then(|p| self.word_at(p))
        } else {
            None
        };
        let o = self.select.drag(self.pointer_select, head, pos, word);
        self.follow_head();
        o
    }

    fn browse_caret(&self) -> Option<CellPos> {
        if let Some(b) = self.browse.and_then(|b| self.resolve(b)) {
            return Some(b);
        }
        if self.lines.is_empty() {
            return None;
        }
        let last_visible = self
            .visual
            .get(self.scroll.visible_range().end.saturating_sub(1))
            .map(|v| v.line)
            .unwrap_or(self.lines.len() - 1);
        Some(CellPos {
            line: last_visible,
            col: self.lines[last_visible].total_width(),
        })
    }

    fn move_caret(&self, from: CellPos, code: KeyCode, word: bool) -> CellPos {
        let n = self.lines.len();
        let line_w = |li: usize| self.lines.get(li).map(|l| l.total_width()).unwrap_or(0);
        match code {
            KeyCode::Left => {
                let ci = self.cell_at(from.line, from.col);
                if ci == 0 {
                    if from.line > 0 {
                        return CellPos {
                            line: from.line - 1,
                            col: line_w(from.line - 1),
                        };
                    }
                    return from;
                }
                let target = if word {
                    let cs = &self.lines[from.line].cells;
                    let mut i = ci;
                    while i > 0 && !Self::is_word(&cs[i - 1]) {
                        i -= 1;
                    }
                    while i > 0 && Self::is_word(&cs[i - 1]) {
                        i -= 1;
                    }
                    i
                } else {
                    ci - 1
                };
                CellPos {
                    line: from.line,
                    col: self.col_of(from.line, target),
                }
            }
            KeyCode::Right => {
                let cs_len = self
                    .lines
                    .get(from.line)
                    .map(|l| l.cells.len())
                    .unwrap_or(0);
                let ci = self.cell_at(from.line, from.col);
                if ci >= cs_len {
                    if from.line + 1 < n {
                        return CellPos {
                            line: from.line + 1,
                            col: 0,
                        };
                    }
                    return from;
                }
                let target = if word {
                    let cs = &self.lines[from.line].cells;
                    let mut i = ci;
                    while i < cs.len() && !Self::is_word(&cs[i]) {
                        i += 1;
                    }
                    while i < cs.len() && Self::is_word(&cs[i]) {
                        i += 1;
                    }
                    i
                } else {
                    ci + 1
                };
                CellPos {
                    line: from.line,
                    col: self.col_of(from.line, target),
                }
            }
            KeyCode::Up => {
                if from.line == 0 {
                    return CellPos { line: 0, col: 0 };
                }
                CellPos {
                    line: from.line - 1,
                    col: from.col.min(line_w(from.line - 1)),
                }
            }
            KeyCode::Down => {
                if from.line + 1 >= n {
                    return CellPos {
                        line: from.line,
                        col: line_w(from.line),
                    };
                }
                CellPos {
                    line: from.line + 1,
                    col: from.col.min(line_w(from.line + 1)),
                }
            }
            KeyCode::Home => {
                if word {
                    CellPos { line: 0, col: 0 }
                } else {
                    CellPos {
                        line: from.line,
                        col: 0,
                    }
                }
            }
            KeyCode::End => {
                if word {
                    CellPos {
                        line: n.saturating_sub(1),
                        col: line_w(n.saturating_sub(1)),
                    }
                } else {
                    CellPos {
                        line: from.line,
                        col: line_w(from.line),
                    }
                }
            }
            _ => from,
        }
    }

    fn extend_selection(&mut self, code: KeyCode, word: bool) -> Outcome {
        self.ensure_index();
        let Some(from) = self.browse_caret() else {
            return Outcome::Consumed;
        };
        let to = self.move_caret(from, code, word);
        let anchor = match self.select.sel {
            Some((a, _)) => a,
            None => match self.anchor_at(from) {
                Some(a) => a,
                None => return Outcome::Consumed,
            },
        };
        let Some(head) = self.anchor_at(to) else {
            return Outcome::Consumed;
        };
        self.select.sel = Some((anchor, head));
        self.browse = Some(head);
        self.select.press = None;

        if let Some(vi) = self.visual.iter().position(|v| {
            v.line == to.line
                && self.col_of(v.line, v.start) <= to.col
                && (to.col < self.col_of(v.line, v.end) || v.end == self.lines[v.line].cells.len())
        }) {
            self.follow = false;
            self.scroll.ensure_visible(vi);
            self.remember_reading();
        }

        self.reveal_cols(to.col..to.col + 1);
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ViewportAction>) {
        if let Some(o) = edit_menu_key(self, key) {
            return (o, None);
        }
        if key.shift()
            && matches!(
                key.code,
                KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
            )
        {
            let word = key.ctrl() || key.alt();
            let o = self.extend_selection(key.code, word);
            return (o, Some(ViewportAction::SelectionChanged));
        }

        let cmd = lookup(&self.keymap, key)
            .or_else(|| (key.ctrl_char('c') && self.has_selection()).then_some(ViewportCmd::Copy));
        match cmd {
            Some(ViewportCmd::ToggleFollow) => {
                self.set_follow(!self.follow);
                return (
                    Outcome::Changed,
                    Some(ViewportAction::FollowChanged(self.follow)),
                );
            }
            Some(ViewportCmd::SelectAll) => {
                return match self.select_all() {
                    Outcome::Changed => (Outcome::Changed, Some(ViewportAction::SelectionChanged)),
                    o => (o, None),
                };
            }
            Some(ViewportCmd::Copy) => {
                return match self.selected_text() {
                    Some(t) => (Outcome::Changed, Some(ViewportAction::Copy(t))),
                    None => (Outcome::Consumed, None),
                };
            }
            None => {}
        }
        let nav = self.nav.action(key).filter(|_| key.plain());
        let moved = match (nav, key.code) {
            (Some(Nav::Left), _) if self.caret.is_none() => {
                return (self.scroll_h(-1), None);
            }
            (Some(Nav::Right), _) if self.caret.is_none() => {
                return (self.scroll_h(1), None);
            }
            (Some(Nav::Up), _) => self.scroll.scroll_by(-1),
            (Some(Nav::Down), _) => self.scroll.scroll_by(1),
            (Some(Nav::PageUp), _) => self.scroll.page_up(),
            (Some(Nav::PageDown), _) => self.scroll.page_down(),
            (Some(Nav::Home), _) => self.scroll.jump_start(),
            (Some(Nav::End), _) => {
                self.scroll.jump_end();
                self.follow = true;
                self.reading = None;
                return (Outcome::Changed, Some(ViewportAction::FollowChanged(true)));
            }
            (_, KeyCode::Esc) if key.plain() => {
                return match self.clear_selection() {
                    Outcome::Changed => (Outcome::Changed, Some(ViewportAction::SelectionChanged)),
                    _ => (Outcome::Ignored, None),
                };
            }
            _ => return (Outcome::Ignored, None),
        };
        if !moved {
            return (Outcome::Consumed, None);
        }
        let was = self.follow;
        self.follow = self.is_at_tail();
        self.remember_reading();
        let ev = (was != self.follow).then_some(ViewportAction::FollowChanged(self.follow));
        (Outcome::Changed, ev)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.render_view(area, buf, ctx, bg);

        self.edit_menu.render(self.area, buf, ctx);
    }

    fn render_view(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;

        let focused = ctx
            .interaction
            .focused_under(self.id, self.edit_menu.is_open());
        let h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        if h_cfg != self.scroll.h_cfg {
            self.scroll.h_cfg = h_cfg;
            self.layout_size = None;
        }
        self.scroll.begin(ctx.track_click);
        self.set_area(area);
        let has_sb = self.scroll.overflows();
        let text_w = self.layout_width;
        let hoff = self.hoff();
        let text = self.text_rect();
        let right = area.x + text_w;
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        ctx.autoscroll(text);
        let sel = self.selection();
        if sel.is_some() {
            ctx.selection(self.id);
        }
        fill(buf, area, Style::new().bg(bg));
        for (k, vi) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            if y >= text.bottom() {
                break;
            }
            let Some(vr) = self.visual.get(vi).copied() else {
                break;
            };
            let l = &self.lines[vr.line];
            let line_marks: Vec<&Mark> = self.marks.iter().filter(|m| m.line == vr.line).collect();

            let left = l.col_of(vr.start) + hoff;
            let mut col = l.col_of(vr.start);
            for c in &l.cells[vr.start..vr.end] {
                let at = col;
                col += c.w;

                if at < left {
                    continue;
                }
                let x = area.x + (at - left).min(u16::MAX as usize) as u16;
                if x + c.w as u16 > right {
                    break;
                }
                let mut st = Style::new().fg(t.tone(c.style.tone)).bg(bg);
                if c.style.bold {
                    st = st.add_modifier(Modifier::BOLD);
                }
                if c.style.italic {
                    st = st.add_modifier(Modifier::ITALIC);
                }
                if c.style.underline {
                    st = st.add_modifier(Modifier::UNDERLINED);
                }
                if c.kind == CellKind::Control {
                    st = st.fg(t.text_muted);
                }
                if c.style.reversed {
                    st = Style::new().fg(t.canvas).bg(t.text_primary);
                }
                if let Some(m) = line_marks
                    .iter()
                    .find(|m| m.range.start < c.src.end && c.src.start < m.range.end)
                {
                    st = if m.current {
                        t.match_current()
                    } else {
                        st.add_modifier(Modifier::UNDERLINED).bg(t.lift(bg))
                    };
                }
                let p = CellPos {
                    line: vr.line,
                    col: at,
                };
                if let Some((a, b)) = sel
                    && p >= a
                    && p < b
                {
                    st = t.selection().add_modifier(st.add_modifier);
                }
                buf.set_string(x, y, &c.g, st);
            }

            let end_col = l.col_of(vr.end);
            if let Some((a, b)) = sel
                && vr.line >= a.line
                && vr.line < b.line
                && vr.end == l.cells.len()
                && end_col >= left
            {
                let x = area.x + (end_col - left).min(u16::MAX as usize) as u16;
                if x < right {
                    fill(buf, Rect::new(x, y, 1, 1), t.selection());
                }
            }
        }
        if has_sb {
            let body = Rect::new(area.x, area.y, area.width, text.height);
            self.scroll
                .draw_with_fade(body, buf, ctx, self.id, focused, &[]);
        }
        if text.height < area.height && self.h_enabled() {
            let w = area.width.saturating_sub(u16::from(has_sb));
            let hb = Rect::new(area.x, text.bottom(), w, 1);
            self.scroll.draw_h(hb, buf, ctx, self.id, focused);
        }

        let cursor_pos = if focused {
            match self.browse.and_then(|b| self.resolve(b)) {
                Some(b) if self.select.sel.is_some() => Some(b),
                _ if self.follow && self.caret_visible => self.caret,
                _ => None,
            }
        } else {
            None
        };
        if let Some(p) = cursor_pos.and_then(|c| self.screen_pos(c)) {
            ctx.set_cursor(p);
        }
    }

    pub fn screen_pos(&self, c: CellPos) -> Option<Position> {
        let vi = self.visual.iter().position(|v| {
            v.line == c.line
                && self.col_of(v.line, v.start) <= c.col
                && (c.col < self.col_of(v.line, v.end)
                    || v.end == self.lines.get(v.line).map_or(0, |l| l.cells.len()))
        })?;
        if !self.scroll.visible_range().contains(&vi) {
            return None;
        }
        let vr = self.visual[vi];
        let hoff = self.hoff();
        let base = self.col_of(vr.line, vr.start) + hoff;
        let text_w = self.layout_width;

        let inside = c.col >= base && c.col - base < text_w as usize;
        if !inside && hoff > 0 {
            return None;
        }
        let cx = self.area.x + (c.col.saturating_sub(base) as u16).min(text_w.saturating_sub(1));
        let cy = self.area.y + (vi - self.scroll.offset) as u16;
        Some(Position::new(cx, cy))
    }

    pub fn context_point(&self) -> Position {
        if let Some((_, end)) = self.selection()
            && let Some(p) = self.screen_pos(end)
        {
            return p;
        }
        if self.caret_visible
            && let Some(c) = self.caret
            && let Some(p) = self.screen_pos(c)
        {
            return p;
        }
        let top = Position::new(self.area.x, self.area.y);
        if self.follow || self.is_at_tail() {
            let drawn = self.scroll.visible_range().len() as u16;
            if drawn > 0 {
                return Position::new(self.area.x, self.area.y + drawn.min(self.area.height) - 1);
            }
        }
        top
    }

    pub fn hbar(&self) -> Rect {
        self.scroll.h_track()
    }
}

impl crate::ui::measure::Measure for TextViewport {
    fn measure(&self, _avail: ratatui::layout::Size, _ctx: &RenderCtx) -> ratatui::layout::Size {
        let widest = self.lines.iter().map(|l| width(&l.text)).max().unwrap_or(0) as u16;
        let bar = 1 + self.scrollbar_gap;
        ratatui::layout::Size::new(
            widest.saturating_add(bar),
            self.lines.len().min(u16::MAX as usize) as u16,
        )
    }

    fn min_width(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> u16 {
        self.measure(avail, ctx).width.min(MIN_W)
    }
}

const MIN_W: u16 = 24;

impl Scrollable for TextViewport {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.scroll
    }

    fn wheel_first(&mut self, axis: Axis, _delta: i32) -> Option<Response<()>> {
        (axis == Axis::Horizontal && !self.h_enabled()).then(Response::ignored)
    }

    fn after_scroll(&mut self, axis: Axis, moved: bool) -> bool {
        if axis == Axis::Vertical && moved {
            self.follow = self.is_at_tail();
            self.remember_reading();
        }
        false
    }
}

impl crate::core::pointer::MouseTarget for TextViewport {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if let Some(o) = edit_menu_pointer(self, ev) {
            return o.into();
        }
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r;
        }
        match ev.phase() {
            Some(Phase::Press) => self.on_press_with(ev.pos(), ev.count(), ev.mods()).into(),
            Some(Phase::Drag) => self.on_drag(ev.pos()).into(),

            Some(Phase::Click) => self.on_click_with(ev.pos(), ev.count()).into(),
            _ => Response::ignored(),
        }
    }

    fn route(&mut self, ev: &crate::core::pointer::PointerEvent) -> Option<Response<()>> {
        edit_menu_route(self, ev)
    }
}

impl EditMenuHost for TextViewport {
    fn edit_menu(&mut self) -> &mut EditMenu {
        &mut self.edit_menu
    }
    fn edit_owner(&self) -> WidgetId {
        self.id
    }
    fn edit_caps(&self) -> EditCaps {
        EditCaps::read_only(self.select.range().is_some(), !self.is_empty())
    }
    fn edit_shortcut(&self, cmd: TextCmd) -> Option<String> {
        let cmd = match cmd {
            TextCmd::Copy => ViewportCmd::Copy,
            TextCmd::SelectAll => ViewportCmd::SelectAll,
            _ => return None,
        };
        crate::core::keymap::label_for(&self.keymap, cmd)
    }
    fn edit_target(&self, ev: &crate::core::pointer::PointerEvent) -> bool {
        ev.id == self.id
    }
    fn edit_run(&mut self, cmd: TextCmd) -> bool {
        match cmd {
            TextCmd::SelectAll => {
                self.select_all();
            }
            TextCmd::Copy => {
                if let Some(t) = self.selected_text() {
                    crate::core::edit::clipboard::copy(&t);
                }
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn toned_spans_wrap_by_word_and_keep_their_tones() {
        let line = vec![
            Span::new("degraded", Tone::Warning),
            Span::new(" · ", Tone::Muted),
            Span::new("3 failed", Tone::Error).bold(),
            Span::new(" · 2 identities unresolved", Tone::Muted),
        ];
        let rows = wrap_spans(&line, 20);
        let text: Vec<String> = rows.iter().map(|l| line_text(l)).collect();
        assert_eq!(
            text,
            ["degraded · 3 failed", "· 2 identities", "unresolved"],
            "{rows:?}"
        );
        assert_eq!(rows[0][0].tone, Tone::Warning);
        let failed = rows[0]
            .iter()
            .find(|s| s.text == "3 failed")
            .expect("its own span");
        assert!(failed.bold && failed.tone == Tone::Error, "{failed:?}");
        assert!(
            rows[1].iter().all(|s| s.tone == Tone::Muted),
            "{:?}",
            rows[1]
        );
    }

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
    fn at_its_measured_width_no_line_wraps() {
        use crate::ui::measure::Measure;
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(12)).wrap(true);
        v.push(vec![Span::plain(
            "a longer line: error SMTP 421 at the end",
        )]);
        let size = {
            let t = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            v.measure(ratatui::layout::Size::ZERO, &ctx)
        };
        assert_eq!(size.height, 13);

        let buf = render(&mut v, size.width, 5);
        assert!(v.scroll.overflows());
        assert_eq!(row(&buf, 4), "a longer line: error SMTP 421 at the end");

        let buf = render(&mut v, size.width - 1, 5);
        assert_ne!(row(&buf, 4), "a longer line: error SMTP 421 at the end");
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect::<String>()
            .trim_end_matches([' ', '┆', '┃'])
            .to_owned()
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn a_line_of_spans_keeps_each_tone_and_cuts_with_an_ellipsis() {
        let t = Theme::junie();
        let line = vec![
            Span::new("degraded", Tone::Error).bold(),
            Span::new(" · ", Tone::Normal),
            Span::new("1 stale", Tone::Warning),
        ];
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        assert_eq!(render_spans(&mut buf, 0, 0, 30, &t, &line, t.canvas), 18);
        assert_eq!(row(&buf, 0), "degraded · 1 stale");
        assert_eq!(buf[(0, 0)].fg, t.error);
        assert!(buf[(0, 0)].modifier.contains(Modifier::BOLD));
        assert_eq!(buf[(9, 0)].fg, t.text_primary);
        assert!(!buf[(9, 0)].modifier.contains(Modifier::BOLD));
        assert_eq!(buf[(11, 0)].fg, t.warning);

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        render_spans(&mut buf, 0, 0, 30, &t, &line, t.popover);
        assert_eq!(buf[(0, 0)].fg, t.legible(t.error, t.popover));

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        assert_eq!(render_spans(&mut buf, 0, 0, 10, &t, &line, t.canvas), 10);
        assert_eq!(row(&buf, 0), "degraded …");
    }

    #[test]
    fn follows_tail_and_wheel_leaves_it() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(50));
        render(&mut v, 40, 10);
        assert!(v.is_at_tail());
        assert_eq!(v.scroll.offset, 40);
        v.wheel(Axis::Vertical, -3).outcome();
        assert!(!v.follow);
        assert_eq!(v.scrollback_depth(), 3);
        v.on_key(&key(KeyCode::End, KeyModifiers::NONE));
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
        let (_, ev) = v.on_key(&key(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(matches!(ev, Some(ViewportAction::Copy(t)) if t.starts_with("line 1")));
        v.select_word_at(Position::new(8, 0));
        assert_eq!(v.selected_text().as_deref(), Some("alpha"));
    }

    #[test]
    fn a_drag_held_below_the_text_scrolls_faster_the_farther_it_is() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(100));
        v.follow = false;
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        v.render(Rect::new(0, 0, 40, 10), &mut buf, &mut ctx, t.canvas);
        assert_eq!(
            hits.autoscroll_at(Position::new(3, 3)),
            Some(v.text_rect()),
            "the text is the autoscroll body"
        );
        v.on_click(Position::new(0, 1));

        v.on_drag(Position::new(4, 10));
        assert_eq!(v.scroll.offset, 1);

        v.on_drag(Position::new(4, 12));
        v.on_drag(Position::new(4, 12));
        assert_eq!(v.scroll.offset, 7);
        assert!(v.selected_text().is_some_and(|t| t.starts_with("line 1")));
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

    #[test]
    fn unchanged_overflow_redraw_reuses_layout() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(4000)).wrap(true);
        let expected = render(&mut v, 100, 30);
        let work = v.work_counters();
        let started = std::time::Instant::now();
        for _ in 0..10 {
            v.set_area(Rect::new(0, 0, 100, 30));
            assert_eq!(render(&mut v, 100, 30), expected);
        }
        eprintln!(
            "4000-line viewport, 10 unchanged redraws: {:?}",
            started.elapsed()
        );
        assert_eq!(
            v.work_counters(),
            work,
            "unchanged scrollback must not reflow"
        );
    }

    #[test]
    fn cached_layout_reflows_after_resize_wrap_and_content_changes() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(20)).wrap(true);
        for (w, h, wrap) in [(40, 10, true), (8, 6, true), (40, 30, true), (8, 30, false)] {
            v.wrap = wrap;
            let mut fresh = TextViewport::with_lines(v.id, v.lines().cloned().collect()).wrap(wrap);
            assert_eq!(render(&mut v, w, h), render(&mut fresh, w, h));
            assert_eq!(v.scroll.v, fresh.scroll.v);
        }
        v.set_lines(lines(2));
        let mut fresh = TextViewport::with_lines(v.id, lines(2));
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.push(vec![Span::plain("appended")]);
        fresh.push(vec![Span::plain("appended")]);
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.replace_last(vec![Span::plain("replaced")]);
        fresh.replace_last(vec![Span::plain("replaced")]);
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
        v.clear();
        fresh.clear();
        assert_eq!(render(&mut v, 40, 10), render(&mut fresh, 40, 10));
    }

    #[test]
    fn split_styles_never_split_a_grapheme_cluster() {
        let mut one =
            TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("👩\u{200d}💻X")]]);
        one.follow = false;
        let a = render(&mut one, 10, 1);

        let mut split = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("👩"),
                Span::new("\u{200d}💻", Tone::Warning),
                Span::plain("X"),
            ]],
        );
        split.follow = false;
        let b = render(&mut split, 10, 1);
        assert_eq!(row(&a, 0), row(&b, 0));
        assert_eq!(one.lines[0].cells.len(), 2);
        assert_eq!(split.lines[0].cells.len(), 2);
        assert_eq!(split.lines[0].col_of(1), 2, "X sits at column 2 both ways");

        let mut acc = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("a"),
                Span::new("\u{301}", Tone::Error),
                Span::plain("X"),
            ]],
        );
        acc.follow = false;
        render(&mut acc, 10, 1);
        acc.select.sel = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 2 }));
        assert_eq!(acc.selected_text().as_deref(), Some("a\u{301}X"));

        assert_eq!(acc.lines[0].cells[0].style.tone, Tone::Normal);

        let mut flags = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![
                Span::plain("🇯"),
                Span::new("🇵", Tone::Muted),
                Span::plain("☕\u{fe0f}!"),
            ]],
        );
        flags.follow = false;
        render(&mut flags, 12, 1);
        assert_eq!(flags.lines[0].cells.len(), 3);
        assert_eq!(flags.lines[0].total_width(), 5);
    }

    #[test]
    fn narrow_clipping_and_nonzero_origin_keep_cells_whole() {
        let mut v =
            TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("ab日本cd")]]);
        v.follow = false;
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 3));
        v.render(Rect::new(5, 1, 3, 1), &mut buf, &mut ctx, t.canvas);
        assert_eq!(
            row(&buf, 1).trim(),
            "ab",
            "a wide cell never paints past the edge"
        );
        assert_eq!(
            v.pos_at(Position::new(6, 1)),
            Some(CellPos { line: 0, col: 1 })
        );
    }

    #[test]
    fn tabs_and_controls_display_visibly_and_copy_as_source() {
        let mut v = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![Span::plain("A\tB\u{1b}[0m\u{7}C\r")]],
        );
        v.follow = false;
        let buf = render(&mut v, 30, 1);
        assert_eq!(row(&buf, 0), "A    B^[[0m^GC^M");
        assert_eq!(v.lines[0].cells[1].kind, CellKind::TabPad);
        assert_eq!(
            v.pos_at(Position::new(3, 0)),
            Some(CellPos { line: 0, col: 3 })
        );
        v.select.sel = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 30 }));
        assert_eq!(
            v.selected_text().as_deref(),
            Some("A\tB\u{1b}[0m\u{7}C\r"),
            "copy returns the source bytes"
        );

        v.select.sel = Some((Anchor { id: 1, col: 2 }, Anchor { id: 1, col: 4 }));
        assert_eq!(v.selected_text().as_deref(), Some("\t"));
    }

    #[test]
    fn retention_is_enforced_on_every_ingestion_path() {
        let eight = lines(8);
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.set_lines(eight.clone());
        assert_eq!(v.len(), 3);
        assert_eq!(v.line_text(0), Some("line 5 alpha beta"));
        let mut w = TextViewport::with_lines(WidgetId::of("v"), eight.clone()).max_lines(3);
        assert_eq!(w.len(), 3);
        w.set_max_lines(Some(1));
        assert_eq!(w.line_text(0), Some("line 7 alpha beta"));
        w.extend(lines(4));
        assert_eq!(w.len(), 1);
        assert_eq!(w.line_text(0), Some("line 3 alpha beta"));
        let mut z = TextViewport::new(WidgetId::of("v")).max_lines(0);
        z.push(vec![Span::plain("x")]);
        z.replace_last(vec![Span::plain("y")]);
        assert_eq!(z.len(), 0, "a zero cap retains nothing");
        let mut e = TextViewport::new(WidgetId::of("v")).max_lines(2);
        e.replace_last(vec![Span::plain("first")]);
        assert_eq!(e.len(), 1, "replace on empty appends");
        let mut n = TextViewport::new(WidgetId::of("v"));
        n.extend(lines(5000));
        assert_eq!(n.len(), 5000, "no cap keeps everything");
        assert!(v.revision() > 0);
    }

    #[test]
    fn eviction_never_retargets_a_selection() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 5);
        v.on_click(Position::new(0, 0));
        v.on_drag(Position::new(1, 0));
        assert_eq!(v.selected_text().as_deref(), Some("A"));
        v.push(vec![Span::plain("D")]);
        render(&mut v, 10, 5);
        assert_eq!(v.selected_text(), None, "an evicted selection is cleared");

        let mut p = TextViewport::new(WidgetId::of("v")).max_lines(3);
        p.follow = false;
        for s in ["A", "B", "C"] {
            p.push(vec![Span::plain(s)]);
        }
        render(&mut p, 10, 5);
        p.on_click(Position::new(0, 0));
        p.on_drag(Position::new(1, 2));
        assert_eq!(p.selected_text().as_deref(), Some("A\nB\nC"));
        p.push(vec![Span::plain("D")]);
        render(&mut p, 10, 5);
        assert_eq!(p.selected_text().as_deref(), Some("B\nC"));
    }

    #[test]
    fn active_drag_survives_eviction_with_the_right_source() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 5);
        v.on_click(Position::new(0, 1));
        v.push(vec![Span::plain("D")]);
        render(&mut v, 10, 5);
        v.on_drag(Position::new(1, 2));
        assert_eq!(v.selected_text().as_deref(), Some("B\nC\nD"));
        let mut a = TextViewport::new(WidgetId::of("v")).max_lines(3);
        a.follow = false;
        for s in ["A", "B", "C"] {
            a.push(vec![Span::plain(s)]);
        }
        render(&mut a, 10, 5);
        a.on_click(Position::new(0, 0));
        a.push(vec![Span::plain("D")]);
        a.push(vec![Span::plain("E")]);
        render(&mut a, 10, 5);
        a.on_drag(Position::new(1, 0));
        assert_eq!(
            a.selected_text().as_deref(),
            Some("C"),
            "an evicted anchor restarts at the first retained line"
        );
    }

    #[test]
    fn reading_position_is_retained_while_follow_is_off() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["A", "B", "C"] {
            v.push(vec![Span::plain(s)]);
        }
        render(&mut v, 10, 2);
        v.scroll.scroll_to(1);
        v.remember_reading();
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "B");
        v.push(vec![Span::plain("D")]);
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "B", "B survives, so B stays on top");
        v.push(vec![Span::plain("E")]);
        let buf = render(&mut v, 10, 2);
        assert_eq!(row(&buf, 0), "C", "B is gone: the first retained line");

        let mut w = TextViewport::new(WidgetId::of("w")).wrap(true);
        w.follow = false;
        w.push(vec![Span::plain("aaaaaaaaa bbbbbbbbb ccccccccc")]);
        w.push(vec![Span::plain("second")]);
        render(&mut w, 12, 2);
        w.scroll.scroll_to(1);
        w.remember_reading();
        w.push(vec![Span::plain("third")]);
        let buf = render(&mut w, 12, 2);
        assert_eq!(row(&buf, 0), "bbbbbbbbb");
    }

    #[test]
    fn producer_caret_follows_eviction_and_replacement() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(2);
        v.push(vec![Span::plain("A")]);
        v.push(vec![Span::plain("B")]);
        v.caret = Some(CellPos { line: 1, col: 1 });
        v.push(vec![Span::plain("C")]);
        assert_eq!(v.caret, Some(CellPos { line: 0, col: 1 }));
        v.push(vec![Span::plain("D")]);
        assert_eq!(v.caret, None, "the caret's line was evicted");
        v.caret = Some(CellPos { line: 1, col: 0 });
        v.pop();
        assert_eq!(v.caret, None);
    }

    #[test]
    fn replace_last_keeps_identity_and_clips_selection() {
        let mut v = TextViewport::with_lines(
            WidgetId::of("v"),
            vec![vec![Span::plain("first")], vec![Span::plain("second line")]],
        );
        v.follow = false;
        render(&mut v, 20, 2);
        v.on_click(Position::new(0, 1));
        v.on_drag(Position::new(10, 1));
        assert_eq!(v.selected_text().as_deref(), Some("second lin"));
        let rev = v.revision();
        v.replace_last(vec![Span::plain("short")]);
        assert!(v.revision() > rev);
        render(&mut v, 20, 2);
        assert_eq!(v.selected_text().as_deref(), Some("short"));
        let rev = v.revision();
        v.replace_last(vec![Span::plain("short")]);
        assert_eq!(
            v.revision(),
            rev,
            "an identical replacement is not a mutation"
        );
    }

    #[test]
    fn every_mutation_is_visible_next_frame() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), vec![vec![Span::plain("OLD")]]);
        v.follow = false;
        let a = render(&mut v, 10, 1);
        assert_eq!(row(&a, 0), "OLD");
        v.replace_last(vec![Span::plain("NEW")]);
        let b = render(&mut v, 10, 1);
        assert_eq!(row(&b, 0), "NEW", "same byte length edit is rendered");
        v.select.sel = Some((Anchor { id: 1, col: 0 }, Anchor { id: 1, col: 3 }));
        assert_eq!(v.selected_text().as_deref(), Some("NEW"));
        v.set_lines(vec![vec![Span::plain("x")], vec![Span::plain("y")]]);
        assert!(v.selection().is_none(), "replacement resets identity");
        let c = render(&mut v, 10, 2);
        assert_eq!(row(&c, 1), "y");
        v.pop();
        let d = render(&mut v, 10, 2);
        assert_eq!(row(&d, 1), "");
    }

    #[test]
    fn tail_update_reparses_only_the_changed_line() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(1000)).wrap(true);
        render(&mut v, 80, 20);
        let w0 = v.work_counters();
        for i in 0..20 {
            v.replace_last(vec![Span::plain(format!("tail {i}"))]);
            render(&mut v, 80, 20);
        }
        let w1 = v.work_counters();
        assert_eq!(w1.segmented_lines - w0.segmented_lines, 20);

        assert!(w1.reflowed_lines - w0.reflowed_lines <= 40);
        for i in 0..20 {
            v.push(vec![Span::plain(format!("append {i}"))]);
            render(&mut v, 80, 20);
        }
        let w2 = v.work_counters();
        assert_eq!(w2.segmented_lines - w1.segmented_lines, 20);

        let mut fresh =
            TextViewport::with_lines(WidgetId::of("v"), v.lines().cloned().collect()).wrap(true);
        assert_eq!(render(&mut v, 80, 20), render(&mut fresh, 80, 20));

        let w3 = v.work_counters();
        render(&mut v, 60, 20);
        let w4 = v.work_counters();
        assert_eq!(w4.segmented_lines, w3.segmented_lines);
        assert!(w4.reflowed_lines - w3.reflowed_lines <= 2 * v.len());
        assert!(w4.reflowed_lines - w3.reflowed_lines >= v.len());
    }

    #[test]
    fn keyboard_selection_matches_mouse_selection() {
        let text = vec![
            vec![Span::plain("café 日本 👩\u{200d}💻 done")],
            vec![Span::plain("second línea here")],
        ];
        let mut k = TextViewport::with_lines(WidgetId::of("v"), text.clone());
        k.follow = false;
        render(&mut k, 40, 4);
        let mut m = TextViewport::with_lines(WidgetId::of("v"), text);
        m.follow = false;
        render(&mut m, 40, 4);

        m.on_click(Position::new(5, 0));
        m.on_drag(Position::new(6, 1));

        k.on_click(Position::new(5, 0));
        k.browse = k.anchor_at(CellPos { line: 0, col: 5 });
        k.on_key(&key(KeyCode::Down, KeyModifiers::SHIFT));
        k.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(k.selected_text(), m.selected_text());
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond")
        );

        k.on_key(&key(
            KeyCode::Right,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        ));
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond línea")
        );
        k.on_key(&key(KeyCode::End, KeyModifiers::SHIFT));
        assert_eq!(
            k.selected_text().as_deref(),
            Some("日本 👩\u{200d}💻 done\nsecond línea here")
        );
        k.on_key(&key(
            KeyCode::Home,
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        ));
        assert_eq!(k.selected_text().as_deref(), Some("café"));

        let (o, _) = k.on_key(&key(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Changed);
        assert!(!k.has_selection());
        let (o, _) = k.on_key(&key(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Ignored);

        let mut e = TextViewport::new(WidgetId::of("e"));
        render(&mut e, 10, 2);
        let (o, _) = e.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(o, Outcome::Consumed);

        let (o, ev) = e.on_key(&key(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!((o, ev), (Outcome::Consumed, None));
    }

    #[test]
    fn keyboard_selection_pauses_follow_and_survives_append_and_resize() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(30)).wrap(true);
        render(&mut v, 40, 5);
        assert!(v.follow);
        v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT));
        v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT | KeyModifiers::ALT));
        assert!(!v.follow, "selecting pauses the tail");
        assert_eq!(v.selected_text().as_deref(), Some("beta"));
        v.push(vec![Span::plain("appended")]);
        render(&mut v, 40, 5);
        assert_eq!(v.selected_text().as_deref(), Some("beta"));
        render(&mut v, 12, 5);
        assert_eq!(
            v.selected_text().as_deref(),
            Some("beta"),
            "wrapped rows keep identity"
        );
        let buf = render(&mut v, 40, 5);
        let sel = v.selection().unwrap();
        assert!(
            v.scroll
                .visible_range()
                .any(|vi| v.visual[vi].line == sel.1.line)
        );
        let _ = buf;
    }

    #[test]
    fn marks_paint_and_survive_eviction() {
        let mut v = TextViewport::new(WidgetId::of("v")).max_lines(3);
        v.follow = false;
        for s in ["alpha", "beta", "gamma"] {
            v.push(vec![Span::plain(s)]);
        }
        v.set_marks(vec![
            Mark {
                line: 1,
                range: 0..2,
                current: true,
            },
            Mark {
                line: 2,
                range: 1..3,
                current: false,
            },
        ]);
        let buf = render(&mut v, 10, 3);

        let t = crate::theme::Theme::junie();
        assert_eq!(buf[(0, 1)].bg, t.accent);
        assert_eq!(buf[(0, 1)].fg, t.text_on_accent);
        assert!(!buf[(0, 1)].modifier.contains(Modifier::REVERSED));
        assert!(buf[(1, 2)].modifier.contains(Modifier::UNDERLINED));
        v.push(vec![Span::plain("delta")]);
        assert_eq!(v.marks().len(), 2);
        assert_eq!(v.marks()[0].line, 0);
        v.reveal_line(2);
        assert!(!v.follow);
    }

    fn wide() -> TextViewport {
        let mut ls = lines(10);
        ls[2] = vec![Span::plain("0123456789".repeat(8))];
        let mut v = TextViewport::with_lines(WidgetId::of("v"), ls);
        v.follow = false;
        v
    }

    fn bar_row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn long_lines_scroll_sideways_under_a_horizontal_bar() {
        let mut v = wide();
        let buf = render(&mut v, 30, 6);

        assert_eq!(v.scroll.viewport_len, 5);
        assert!(v.scroll.h.overflows());
        assert_eq!(v.scroll.h.content_len, 80);
        let bar0 = bar_row(&buf, 5);
        assert!(bar0.starts_with(scrollbar::H_THUMB), "{bar0}");
        assert!(bar0.contains(scrollbar::H_TRACK), "{bar0}");
        assert!(row(&buf, 2).starts_with("0123456789"));
        assert_eq!(v.wheel(Axis::Horizontal, 1).outcome(), Outcome::Changed);
        assert_eq!(v.scroll.h.offset, 4, "one notch is HScroll::step cells");
        let buf = render(&mut v, 30, 6);
        assert!(row(&buf, 2).starts_with("456789"), "{}", row(&buf, 2));

        assert!(row(&buf, 0).starts_with(" 0 alpha"), "{}", row(&buf, 0));
        assert_ne!(bar_row(&buf, 5), bar0, "the thumb moved");

        let (o, _) = v.on_key(&key(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Changed);
        assert_eq!(v.scroll.h.offset, 8);
        v.on_key(&key(KeyCode::Left, KeyModifiers::NONE));
        v.on_key(&key(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(v.scroll.h.offset, 0);

        for _ in 0..40 {
            v.wheel(Axis::Horizontal, 1).outcome();
        }
        assert_eq!(v.scroll.h.offset, v.scroll.h.max_offset());
        assert_eq!(v.scroll.h.offset, 80 - v.scroll.h.viewport_len);
    }

    #[test]
    fn a_producer_caret_keeps_the_arrows() {
        let mut v = wide();
        v.caret = Some(CellPos { line: 9, col: 0 });
        render(&mut v, 30, 6);
        let (o, _) = v.on_key(&key(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(o, Outcome::Ignored);
        assert_eq!(v.scroll.h.offset, 0);
        assert_eq!(
            v.wheel(Axis::Horizontal, 1).outcome(),
            Outcome::Changed,
            "the wheel still scrolls"
        );
    }

    #[test]
    fn mouse_selection_maps_through_the_horizontal_offset() {
        let mut v = wide();
        render(&mut v, 30, 6);
        v.scroll.h.scroll_to(13);
        render(&mut v, 30, 6);
        assert_eq!(
            v.pos_at(Position::new(0, 2)),
            Some(CellPos { line: 2, col: 13 })
        );
        v.on_click(Position::new(0, 2));
        v.on_drag(Position::new(5, 2));
        assert_eq!(v.selected_text().as_deref(), Some("34567"));
        let buf = render(&mut v, 30, 6);
        let t = Theme::junie();
        let sel_bg = t.selection().bg.unwrap();
        assert_eq!(buf[(0, 2)].bg, sel_bg, "selection painted at the view");
        assert_ne!(buf[(5, 2)].bg, sel_bg);

        v.on_drag(Position::new(40, 2));
        assert_eq!(v.scroll.h.offset, 17);
        let sel = v.selected_text().unwrap();
        assert!(sel.starts_with("3456789"), "{sel}");

        v.scroll.h.scroll_to(4);
        v.select_word_at(Position::new(4, 0));
        assert_eq!(v.selected_text().as_deref(), Some("alpha"));
    }

    #[test]
    fn a_search_jump_scrolls_the_match_into_view_sideways() {
        let mut v = wide();
        render(&mut v, 30, 6);
        v.set_marks(vec![Mark {
            line: 2,
            range: 61..64,
            current: true,
        }]);
        v.reveal_line(2);
        let visible = v.scroll.h.visible_range();
        assert!(
            visible.contains(&61) && visible.contains(&63),
            "{visible:?}"
        );
        let buf = render(&mut v, 30, 6);
        let x = (61 - v.scroll.h.offset) as u16;
        let t = Theme::junie();
        assert_eq!(
            buf[(x, 2)].bg,
            t.accent,
            "the current match is drawn in view"
        );
        assert_eq!(buf[(x, 2)].symbol(), "1");

        v.reveal_mark(2, 2..3);
        assert!(v.scroll.h.visible_range().contains(&2));

        v.scroll.h.scroll_to(0);
        v.on_click(Position::new(0, 2));
        v.on_drag(Position::new(1, 2));
        for _ in 0..40 {
            v.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT));
        }
        assert!(v.scroll.h.visible_range().contains(&40), "{:?}", v.scroll.h);
    }

    #[test]
    fn follow_keeps_the_tail_with_a_horizontal_offset() {
        let mut v = wide();
        v.set_follow(true);
        render(&mut v, 30, 6);
        assert!(v.is_at_tail());
        v.wheel(Axis::Horizontal, 1).outcome();
        v.wheel(Axis::Horizontal, 1).outcome();
        assert!(v.follow, "sideways scrolling does not pause follow");
        for i in 0..5 {
            v.push(vec![Span::plain(format!("new {i}"))]);
        }
        render(&mut v, 30, 6);
        assert!(v.follow && v.is_at_tail());
        assert_eq!(v.scroll.h.offset, 8, "the horizontal position is kept");
        assert_eq!(v.scroll.viewport_len, 5);

        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &t,
            Interaction {
                focus: Some(v.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        v.caret = Some(CellPos { line: 14, col: 2 });
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 6));
        v.render(Rect::new(0, 0, 30, 6), &mut buf, &mut ctx, t.canvas);
        assert_eq!(ctx.cursor, None);

        v.scroll.h.scroll_to(0);
        v.render(Rect::new(0, 0, 30, 6), &mut buf, &mut ctx, t.canvas);
        assert_eq!(ctx.cursor, Some(Position::new(2, 4)));
    }

    #[test]
    fn wrapping_switches_horizontal_scrolling_off() {
        let mut v = wide().wrap(true);
        let buf = render(&mut v, 30, 6);
        assert!(!v.scroll.h.overflows());
        assert_eq!(v.wheel(Axis::Horizontal, 1).outcome(), Outcome::Ignored);
        assert!(!bar_row(&buf, 5).contains(scrollbar::H_TRACK));

        let mut v = wide().horizontal(HScroll::off());
        let buf = render(&mut v, 30, 6);
        assert_eq!(v.wheel(Axis::Horizontal, 1).outcome(), Outcome::Ignored);
        assert!(!bar_row(&buf, 5).contains(scrollbar::H_TRACK));
        assert_eq!(v.scroll.viewport_len, 6, "no row is reserved");
    }

    #[test]
    fn the_horizontal_bar_clicks_and_drags() {
        let mut v = wide().horizontal(HScroll::default().track_click(TrackClick::Jump));
        render(&mut v, 30, 6);

        assert_eq!(v.scroll.h_track(), Rect::new(0, 5, 29, 1));
        assert_eq!(
            v.bar_press(Position::new(28, 5))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert_eq!(v.scroll.h.offset, v.scroll.h.max_offset());
        assert_eq!(v.scroll.offset, 0, "the vertical axis did not move");
        assert_eq!(
            match v.scroll.bar_of(scrollbar::h_id_for(v.id)) {
                Some(bar) => v.bar_input(&bar, Position::new(0, 5), false).outcome(),
                None => Outcome::Consumed,
            },
            Outcome::Changed
        );
        assert_eq!(v.scroll.h.offset, 0);

        let mut v = wide();
        render(&mut v, 30, 6);
        v.bar_press(Position::new(28, 5))
            .outcome()
            .or(Outcome::Consumed);
        assert_eq!(v.scroll.h.offset, v.scroll.h.viewport_len);
    }

    #[test]
    fn the_vertical_bar_can_page_on_a_track_click() {
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines(50));
        v.follow = false;
        render(&mut v, 30, 10);
        assert_eq!(
            v.bar_press(Position::new(29, 9))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert_eq!(v.scroll.offset, 10, "one page down, not a jump to the end");

        let mut v =
            TextViewport::with_lines(WidgetId::of("v"), lines(50)).track_click(TrackClick::Jump);
        v.follow = false;
        render(&mut v, 30, 10);
        v.bar_press(Position::new(29, 9))
            .outcome()
            .or(Outcome::Consumed);
        assert_eq!(v.scroll.offset, 40);
    }
}

#[cfg(test)]
mod select_all_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers as M;

    #[test]
    fn cmd_or_ctrl_a_selects_everything_and_y_copies_it() {
        for mods in [M::SUPER, M::CONTROL] {
            let lines = (0..30)
                .map(|i| vec![Span::plain(format!("line {i}"))])
                .collect();
            let mut v = TextViewport::with_lines(WidgetId::of("v"), lines);
            let t = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 5));
            v.render(Rect::new(0, 0, 30, 5), &mut buf, &mut ctx, t.canvas);
            let (out, ev) = v.on_key(&Key {
                code: KeyCode::Char('a'),
                mods,
            });
            assert_eq!(out, Outcome::Changed);
            assert_eq!(ev, Some(ViewportAction::SelectionChanged));
            let all = v.selected_text().unwrap();
            assert!(all.starts_with("line 0\nline 1\n"), "{all}");
            assert!(all.ends_with("line 29"), "{all}");
            let (_, ev) = v.on_key(&Key {
                code: KeyCode::Char('y'),
                mods: M::NONE,
            });
            assert_eq!(ev, Some(ViewportAction::Copy(all)));

            assert_eq!(
                v.on_key(&Key {
                    code: KeyCode::Char('a'),
                    mods,
                })
                .0,
                Outcome::Consumed
            );
        }
    }
}

#[cfg(test)]
mod cmd_copy_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::core::keymap::hint_key;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers as M;

    #[test]
    fn cmd_a_then_cmd_c_copies_and_the_hint_falls_back_to_y() {
        let lines = (0..3)
            .map(|i| vec![Span::plain(format!("line {i}"))])
            .collect();
        let mut v = TextViewport::with_lines(WidgetId::of("v"), lines);
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 5));
        v.render(Rect::new(0, 0, 30, 5), &mut buf, &mut ctx, t.canvas);
        let cmd = |c| Key {
            code: KeyCode::Char(c),
            mods: M::SUPER,
        };
        v.on_key(&cmd('a'));
        let (out, ev) = v.on_key(&cmd('c'));
        assert_eq!(out, Outcome::Changed);
        assert_eq!(
            ev,
            Some(ViewportAction::Copy("line 0\nline 1\nline 2".into()))
        );
        assert_eq!(hint_key(&v.keymap, ViewportCmd::Copy), Some("y"));
        crate::core::keyboard::set_super_delivered(true);
        let label = KeyBinding::cmd('c').label();
        assert_eq!(hint_key(&v.keymap, ViewportCmd::Copy), Some(label.as_str()));
    }

    #[test]
    fn shift_click_extends_and_a_triple_click_selects_the_line() {
        use ratatui::crossterm::event::KeyModifiers;
        let lines = |n: usize| -> Vec<Line> {
            (0..n)
                .map(|i| vec![Span::plain(format!("line {i} alpha beta"))])
                .collect()
        };
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let draw = |v: &mut TextViewport| {
            let t = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 4));
            v.render(Rect::new(0, 0, 30, 4), &mut buf, &mut ctx, t.canvas);
            drop(ctx);
            hits
        };
        let id = WidgetId::of("v");
        let mut v = TextViewport::with_lines(id, lines(3));
        v.set_follow(false);
        let hits = draw(&mut v);
        let send = |v: &mut TextViewport, phase, x, y, count, mods| {
            let ev = PointerEvent::new(&hits, phase, id, Position::new(x, y), count, mods);
            let _ = v.pointer(&ev);
        };
        let none = KeyModifiers::NONE;
        let shift = KeyModifiers::SHIFT;

        send(&mut v, Phase::Press, 7, 0, 1, none);
        send(&mut v, Phase::Drag, 15, 0, 1, none);
        send(&mut v, Phase::Click, 15, 0, 1, none);
        assert_eq!(v.selected_text().as_deref(), Some("alpha be"));

        send(&mut v, Phase::Press, 4, 1, 1, shift);
        send(&mut v, Phase::Click, 4, 1, 1, shift);
        assert_eq!(
            v.selected_text().as_deref(),
            Some("alpha beta\nline"),
            "Shift-click extends from the anchor"
        );

        send(&mut v, Phase::Press, 2, 0, 1, shift);
        send(&mut v, Phase::Click, 2, 0, 1, shift);
        assert_eq!(v.selected_text().as_deref(), Some("ne 0"));

        for count in 1..=3 {
            send(&mut v, Phase::Press, 9, 2, count, none);
            send(&mut v, Phase::Click, 9, 2, count, none);
        }
        assert_eq!(v.selected_text().as_deref(), Some("line 2 alpha beta"));

        send(&mut v, Phase::Press, 14, 1, 1, none);
        send(&mut v, Phase::Click, 14, 1, 1, none);
        send(&mut v, Phase::Press, 14, 1, 2, none);
        send(&mut v, Phase::Click, 14, 1, 2, none);
        assert_eq!(v.selected_text().as_deref(), Some("beta"));

        send(&mut v, Phase::Press, 1, 1, 1, none);
        send(&mut v, Phase::Click, 1, 1, 1, none);
        assert_eq!(v.selected_text(), None);

        let mut v = TextViewport::with_lines(id, lines(3))
            .pointer_select(crate::widgets::field_common::PointerSelect::DEFAULT.line(false));
        draw(&mut v);
        for count in 1..=3 {
            send(&mut v, Phase::Press, 9, 2, count, none);
            send(&mut v, Phase::Click, 9, 2, count, none);
        }
        assert_eq!(v.selected_text(), None);
    }
}
