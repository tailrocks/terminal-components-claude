//! Panels: the surface container. Two flavours:
//! - **card**: filled `surface` rectangle, no border, title row. The default.
//! - **framed**: rounded subtle border, used when a region must read as a
//!   distinct pane (split views, dialogs).
//!
//! Focus at container level is shown by the border/title only; the accent
//! gutter bar belongs to the focused control inside.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::response::Response;
use crate::core::scroll::{HScroll, ScrollPane, Scrollable};
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::field_common::PointerSelect;
use crate::widgets::scrollbar;
use crate::widgets::text_select::{self, TextSelect};
use crate::widgets::viewport::CellPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Card,
    Framed,
}

/// How a card panel shows that focus is inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelFocus {
    /// The title turns primary and bold; the only `▎` on screen stays with
    /// the focused control inside.
    #[default]
    Title,
    /// The title also carries a `▎` bar (a container that is itself the
    /// focus stop, with no focusable control inside).
    Bar,
}

/// Cells a title keeps when a long meta competes for the title row: the
/// default of [`Panel::title_min`].
pub const TITLE_MIN: u16 = 12;

/// The fewest cells a cut meta is drawn in; below that it is dropped.
const META_MIN: u16 = 4;

/// `meta` cut to `room` cells with an ellipsis, never in the middle of a
/// number or a range (`"1–1…"` would misstate the position): the cut backs
/// up to the last space before it. `None` when what is left is shorter
/// than [`META_MIN`]. `ellipsis` is the one-cell mark it ends with.
fn cut_meta(meta: &str, room: u16, ellipsis: &str) -> Option<String> {
    if room < META_MIN {
        return None;
    }
    let cut = crate::ui::text::truncate(meta, room as usize);
    let body = cut.trim_end_matches('…');
    let next = meta[body.len()..].chars().next();
    let numeric = |c: char| c.is_ascii_digit() || matches!(c, '–' | '-' | '/' | ',' | '.' | '~');
    let mid_number = body.chars().last().is_some_and(numeric) && next.is_some_and(numeric);
    let body = if mid_number {
        body.rsplit_once(' ').map(|(head, _)| head).unwrap_or("")
    } else {
        body
    }
    .trim_end_matches([' ', '·']);
    (crate::ui::text::width(body) as u16 + 1 >= META_MIN).then(|| format!("{body}{ellipsis}"))
}

pub struct Panel<'a> {
    pub title: Option<&'a str>,
    pub kind: PanelKind,
    pub focused: bool,
    /// Right-aligned text in the title row (position label, badge).
    pub meta: Option<&'a str>,
    pub badge: Option<(&'a str, crate::theme::BadgeKind)>,
    pub bg_override: Option<Color>,
    /// How focus-within shows on a card. Default [`PanelFocus::Title`].
    pub focus_mark: PanelFocus,
    /// Cells the title keeps before the meta gives way. Default
    /// [`TITLE_MIN`] (12); a shorter title keeps its whole width.
    pub title_min: u16,
    /// A short form of the meta, used when the full one does not fit
    /// beside the title (`"1–9/24"` for `"1–9 of 24"`). `None`: a position
    /// label `"a–b of n"` shortens to `"a–b/n"` on its own. When the short
    /// form is still too long, its first `" · "`-separated fact is shown,
    /// then a cut that never splits a number, then nothing.
    pub meta_short: Option<&'a str>,
    /// Columns a card keeps blank on each side of its title and content.
    /// Default [`CARD_PAD`] (2); a narrow pane (a navigation sidebar)
    /// takes 1 so its rows keep their labels. A framed panel ignores it.
    pub pad_x: u16,
}

/// Columns a card keeps blank on each side: the default of
/// [`Panel::pad_x`].
pub const CARD_PAD: u16 = 2;

impl<'a> Panel<'a> {
    pub fn card(title: Option<&'a str>) -> Self {
        Self {
            title,
            kind: PanelKind::Card,
            focused: false,
            meta: None,
            badge: None,
            bg_override: None,
            focus_mark: PanelFocus::Title,
            title_min: TITLE_MIN,
            meta_short: None,
            pad_x: CARD_PAD,
        }
    }
    pub fn framed(title: Option<&'a str>) -> Self {
        Self {
            title,
            kind: PanelKind::Framed,
            focused: false,
            meta: None,
            badge: None,
            bg_override: None,
            focus_mark: PanelFocus::Title,
            title_min: TITLE_MIN,
            meta_short: None,
            pad_x: CARD_PAD,
        }
    }
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    pub fn focus_mark(mut self, mark: PanelFocus) -> Self {
        self.focus_mark = mark;
        self
    }
    pub fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
        self
    }
    /// The short form of the meta for a narrow title row.
    pub fn meta_short(mut self, m: &'a str) -> Self {
        self.meta_short = Some(m);
        self
    }
    /// Columns a card keeps blank on each side (default [`CARD_PAD`]).
    pub fn pad_x(mut self, cells: u16) -> Self {
        self.pad_x = cells;
        self
    }

    /// The title row's left edge and width inside `area`.
    fn title_span(&self, area: Rect) -> (u16, u16) {
        let pad = match self.kind {
            PanelKind::Card => self.pad_x,
            PanelKind::Framed => 2,
        };
        (area.x + pad, area.width.saturating_sub(pad * 2))
    }

    /// Cells the title keeps before the meta gives way (default
    /// [`TITLE_MIN`]).
    pub fn title_min(mut self, cells: u16) -> Self {
        self.title_min = cells;
        self
    }

    /// Background colour the content will sit on.
    pub fn bg(&self, t: &Theme) -> Color {
        if let Some(bg) = self.bg_override {
            return bg;
        }
        match self.kind {
            PanelKind::Card => t.surface,
            PanelKind::Framed => t.canvas,
        }
    }

    /// Redraw the title row with `meta` after the content was rendered, so
    /// a scroll position computed from the content's own layout is fresh
    /// on the first frame, after a resize and after a tick.
    pub fn draw_meta(&self, area: Rect, buf: &mut Buffer, t: &Theme, meta: &str) {
        let area = area.intersection(*buf.area());
        if area.is_empty() || (self.kind == PanelKind::Framed && area.width <= 4) {
            return;
        }
        let bg = self.bg(t);
        let late = Panel {
            title: self.title,
            kind: self.kind,
            focused: self.focused,
            meta: Some(meta),
            badge: self.badge,
            bg_override: self.bg_override,
            focus_mark: self.focus_mark,
            title_min: self.title_min,
            meta_short: self.meta_short,
            pad_x: self.pad_x,
        };
        let (x, w) = self.title_span(area);
        let row = Rect::new(x, area.y, w, 1);
        fill(buf, row, Style::new().bg(bg));
        late.title_row(row.x, row.y, row.width, buf, t, bg);
    }

    /// Mark a card body whose content was cut at its bottom edge: a faint
    /// `…` in the last cell of `inner`'s last row, so clipped prose never
    /// looks complete. Call it after drawing the content.
    pub fn mark_clipped(&self, inner: Rect, buf: &mut Buffer, t: &Theme) {
        let inner = inner.intersection(*buf.area());
        if inner.is_empty() {
            return;
        }
        let bg = self.bg(t);
        let (x, y) = (inner.right() - 1, inner.bottom() - 1);
        buf.set_string(x, y, t.glyphs.ellipsis, t.faint().bg(bg));
    }

    /// Draw the panel chrome and return the content area, or `None` when
    /// the panel has no content row at all (a card squeezed to its title,
    /// a rect past the bottom of the screen). Prefer it to [`Self::render`]
    /// for content that must not be drawn when there is no room for it:
    /// `render` returns an empty rect in that case, whose `x`/`y` still
    /// point at the panel's edge.
    pub fn try_render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> Option<Rect> {
        let inner = self.render(area, buf, t);
        (!inner.is_empty()).then_some(inner)
    }

    /// Draw the panel chrome and return the inner content area.
    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> Rect {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return area;
        }
        let bg = self.bg(t);
        match self.kind {
            PanelKind::Card => {
                fill(buf, area, Style::new().bg(bg));
                let inner = area.inner(ratatui::layout::Margin::new(self.pad_x, 1));
                if self.focused && self.title.is_some() && self.focus_mark == PanelFocus::Bar {
                    // opt-in: the container itself is the focus stop
                    let s = crate::ui::ctx::VisualState {
                        focused: true,
                        ..Default::default()
                    };
                    buf.set_string(
                        area.x + self.pad_x.saturating_sub(1),
                        area.y,
                        t.gutter_symbol(s),
                        t.gutter(s, bg, false),
                    );
                }
                let (x, w) = self.title_span(area);
                self.title_row(x, area.y, w, buf, t, bg);
                if self.title.is_some() {
                    Rect::new(
                        inner.x,
                        inner.y + 1,
                        inner.width,
                        inner.height.saturating_sub(1),
                    )
                } else {
                    inner
                }
            }
            PanelKind::Framed => {
                fill(buf, area, Style::new().bg(bg));
                let block = Block::new()
                    .borders(Borders::ALL)
                    .border_set(t.glyphs.border)
                    .border_style(t.border(self.focused).bg(bg));
                block.render(area, buf);
                if area.width > 4 {
                    self.title_row(area.x + 2, area.y, area.width.saturating_sub(4), buf, t, bg);
                }
                let inner = area.inner(ratatui::layout::Margin::new(1, 1));
                Rect::new(
                    inner.x + 2,
                    inner.y,
                    inner.width.saturating_sub(3),
                    inner.height,
                )
            }
        }
    }

    fn title_row(&self, x: u16, y: u16, w: u16, buf: &mut Buffer, t: &Theme, bg: Color) {
        if w == 0 {
            return;
        }
        let mut cx = x;
        let framed = self.kind == PanelKind::Framed;
        let pad: u16 = if framed { 2 } else { 0 };
        // the title keeps `title_min` cells (its whole width when shorter):
        // a panel must still say what it is. The meta gives way first: its
        // short form, then an ellipsis, and it is dropped when not even a
        // few cells are left
        let title_min = self
            .title
            .map(|t| (crate::ui::text::width(t) as u16).min(self.title_min))
            .unwrap_or(0);
        let ell = t.glyphs.ellipsis;
        let meta = self.meta.filter(|m| !m.is_empty()).and_then(|m| {
            let room = w.saturating_sub(pad + if title_min > 0 { title_min + 1 } else { 0 });
            let fits = |m: &str| crate::ui::text::width(m) as u16 <= room;
            if fits(m) {
                return Some(m.to_owned());
            }
            let short = self
                .meta_short
                .map(str::to_owned)
                .unwrap_or_else(|| m.replace(" of ", "/"));
            // then only its first fact ("388–401/401" of
            // "388–401/401 · following")
            let head = short.split(" · ").next().unwrap_or_default().to_owned();
            if fits(&short) {
                Some(short)
            } else if fits(&head) && !head.is_empty() {
                Some(head)
            } else {
                cut_meta(&short, room, ell)
            }
        });
        let meta_w = meta
            .as_ref()
            .map(|m| crate::ui::text::width(m) as u16 + pad)
            .unwrap_or(0);
        if let Some(title) = self.title {
            let style = if self.focused {
                t.title().bg(bg)
            } else {
                t.secondary().bg(bg)
            };
            let room = if meta_w > 0 {
                w.saturating_sub(meta_w + 1 + pad)
            } else {
                w.saturating_sub(pad)
            };
            let full = title;
            let title = crate::ui::text::truncate(full, room as usize);
            // a cut title reveals its full text on hover
            let cut = Rect::new(
                cx + u16::from(framed),
                y,
                crate::ui::text::width(&title) as u16,
                1,
            );
            crate::ui::reveal::queue_cut(cut, full);
            let title = if framed { format!(" {title} ") } else { title };
            buf.set_string(cx, y, &title, style);
            cx += crate::ui::text::width(&title) as u16;
        }
        let mut right = x + w;
        // an empty meta draws nothing, not a blank gap in the frame
        if let Some(meta) = meta.filter(|m| !m.is_empty()) {
            let text = if framed { format!(" {meta} ") } else { meta };
            let tw = crate::ui::text::width(&text) as u16;
            if right >= cx + tw + u16::from(cx > x) {
                right = right.saturating_sub(tw);
                buf.set_string(right, y, &text, t.faint().bg(bg));
            }
        }
        if let Some((badge, kind)) = self.badge {
            let text = format!(" {badge} ");
            let bw = crate::ui::text::width(&text) as u16;
            if right > cx + bw + 1 {
                right = right.saturating_sub(bw + 1);
                buf.set_string(right, y, &text, t.badge(kind));
            }
        }
    }
}

use ratatui::style::Style;

/// A scrollable read-only text panel (log output, prose). It is a focus stop
/// itself because it has no focusable children.
///
/// A panel that `tail`s (a log) follows new lines while the viewport is at
/// the end; scrolling away pauses following and scrolling back to the end,
/// by any means, resumes it. A panel that does not tail (prose) never
/// follows: `End` is only a jump.
///
/// Its text selects like a [`TextViewport`](crate::widgets::viewport::TextViewport)'s,
/// through the same selection model: a drag selects, Shift with a press
/// extends the selection from its anchor, a double click selects a word
/// and a triple click the line ([`Self::pointer_select`]); Select All
/// ([`PanelCmd::SelectAll`]) selects everything, Copy ([`PanelCmd::Copy`])
/// puts the selection on the clipboard and Esc clears it. Positions are
/// display rows (a wrapped line's rows; a triple click selects all of
/// them) and columns.
#[derive(Debug, Clone)]
pub struct ScrollPanel {
    pub id: WidgetId,
    pub lines: Vec<String>,
    /// Rows (`scroll`, the vertical state) and cells (`scroll.h`) with both bars
    /// ([`Scrollable`]).
    pub scroll: ScrollPane,
    /// Following the tail right now (only ever true for a tailing panel).
    pub follow: bool,
    /// The panel tails its content: new lines keep the end in view.
    pub tail: bool,
    pub wrap: bool,
    pub area: Rect,
    /// The wrap width, the wrapped rows and the line each row comes from.
    wrapped_cache: (u16, Vec<String>, Vec<usize>),
    /// Horizontal scrolling policy; `None` follows [`RenderCtx::hscroll`].
    pub horizontal: Option<HScroll>,
    /// Which keys scroll (default [`NavKeys::default`]; ←/→ scroll
    /// sideways, since the panel has no caret).
    pub nav: NavKeys,
    /// The panel's own action keys (default [`PanelCmd::default_keymap`]).
    pub keymap: KeyMap<PanelCmd>,
    /// What a drag, a double and a triple click select (the text fields'
    /// policy). Default [`PointerSelect::DEFAULT`] (macOS): a drag
    /// selects, a double click a word (a drag from it extends by words),
    /// a triple click the line; Shift with a press extends the selection.
    pub pointer_select: PointerSelect,
    /// The text selection and the pointer gesture in progress, over
    /// display rows (the model [`crate::widgets::viewport::TextViewport`]
    /// shares).
    select: TextSelect<CellPos>,
    /// The text width the selection's rows were laid out for: a wrapped
    /// panel that reflows drops its selection.
    select_w: u16,
}

/// A [`ScrollPanel`] action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelCmd {
    /// A tailing panel: pause following, or jump to the end and follow
    /// again. Default `f`.
    Follow,
    /// Put the selection on the clipboard
    /// ([`crate::core::edit::clipboard::copy`]). Default Cmd+C where the
    /// terminal delivers Cmd, and `y` (as on a `TextViewport`).
    Copy,
    /// Select every line, ready for Copy. Default Cmd+A where the
    /// terminal delivers Cmd, and Ctrl+A ([`crate::core::keymap::SELECT_ALL`]).
    SelectAll,
}

impl Actions for PanelCmd {
    fn default_keymap() -> KeyMap<Self> {
        let mut map = vec![
            (KeyBinding::char('f'), PanelCmd::Follow),
            (KeyBinding::cmd('c'), PanelCmd::Copy),
            (KeyBinding::char('y'), PanelCmd::Copy),
        ];
        map.extend(crate::core::keymap::select_all_bindings(
            PanelCmd::SelectAll,
        ));
        map
    }
}

impl ScrollPanel {
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<PanelCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn new(id: WidgetId, lines: Vec<String>) -> Self {
        Self {
            id,
            lines,
            scroll: ScrollPane::default(),
            follow: false,
            tail: false,
            wrap: false,
            area: Rect::ZERO,
            wrapped_cache: (0, vec![], vec![]),
            horizontal: None,
            nav: NavKeys::default(),
            keymap: PanelCmd::default_keymap(),
            pointer_select: PointerSelect::DEFAULT,
            select: TextSelect::default(),
            select_w: 0,
        }
    }

    /// See [`Self::pointer_select`].
    pub fn pointer_select(mut self, policy: PointerSelect) -> Self {
        self.pointer_select = policy;
        self
    }

    /// Override the app-wide horizontal scrolling policy.
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    /// Lines scroll sideways: not wrapping and the policy allows it.
    fn h_enabled(&self) -> bool {
        !self.wrap && self.scroll.h_cfg.enabled
    }

    /// Scroll sideways by `delta` steps (the arrows); the wheel's rule:
    /// nothing wider than the panel leaves the arrows free for the page.
    fn scroll_h(&mut self, delta: isize) -> Outcome {
        self.wheel(Axis::Horizontal, delta as i32).outcome()
    }

    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    /// A log: follow new lines while the end is in view.
    pub fn tail(mut self, t: bool) -> Self {
        self.tail = t;
        self.follow = t;
        self
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
        self.wrapped_cache.0 = 0;
    }

    /// Following resumes at the end and pauses anywhere else; a panel
    /// that does not tail never follows.
    fn settle_follow(&mut self) {
        self.follow = self.tail && self.scroll.at_end();
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match lookup(&self.keymap, key) {
            Some(PanelCmd::SelectAll) => return self.select_all(),
            Some(PanelCmd::Copy) => {
                // nothing selected: the key stays free for the page
                let Some(text) = self.selected_text() else {
                    return Outcome::Ignored;
                };
                crate::core::edit::clipboard::copy(&text);
                return Outcome::Consumed;
            }
            _ => {}
        }
        if key.is(ratatui::crossterm::event::KeyCode::Esc) && key.plain() {
            return self.clear_selection();
        }
        if let Some(PanelCmd::Follow) = lookup(&self.keymap, key)
            && self.tail
        {
            let was = self.follow;
            if was {
                self.follow = false;
            } else {
                self.scroll.jump_end();
                self.follow = true;
            }
            return Outcome::Changed;
        }
        let Some(nav) = self.nav.action(key).filter(|_| key.plain()) else {
            return Outcome::Ignored;
        };
        let moved = match nav {
            Nav::Up => self.scroll.scroll_by(-1),
            Nav::Down => self.scroll.scroll_by(1),
            Nav::PageUp => self.scroll.page_up(),
            Nav::PageDown => self.scroll.page_down(),
            Nav::Home => self.scroll.jump_start(),
            Nav::End => self.scroll.jump_end(),
            // the panel has no caret, so the arrows scroll sideways
            Nav::Left => return self.scroll_h(-1),
            Nav::Right => return self.scroll_h(1),
        };
        let was = self.follow;
        self.settle_follow();
        if moved || was != self.follow {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Lay the content out for `area` without drawing: the wrap cache,
    /// content and viewport lengths and the tail follow are settled, so a
    /// position label read afterwards is exact. `render` calls it too.
    pub fn measure(&mut self, area: Rect) {
        self.area = area;
        let text_w = area.width.saturating_sub(2);
        if self.wrap && self.wrapped_cache.0 != text_w {
            let mut out = Vec::new();
            let mut src = Vec::new();
            for (i, l) in self.lines.iter().enumerate() {
                let rows = crate::ui::text::wrap(l, text_w as usize);
                src.extend(std::iter::repeat_n(i, rows.len()));
                out.extend(rows);
            }
            self.wrapped_cache = (text_w, out, src);
        }
        // a wrapped panel that reflows moves its rows under the selection
        if self.select_w != text_w {
            if self.wrap {
                self.select = TextSelect::default();
            }
            self.select_w = text_w;
        }
        let len = if self.wrap {
            self.wrapped_cache.1.len()
        } else {
            self.lines.len()
        };
        // the horizontal axis: the widest line against the text width
        let widest = if self.h_enabled() {
            self.lines
                .iter()
                .map(|l| crate::ui::text::width(l))
                .max()
                .unwrap_or(0)
        } else {
            0
        };
        self.scroll.h.set_content(widest);
        self.scroll.h.set_viewport(text_w as usize);
        let has_hb =
            self.h_enabled() && area.height > 1 && self.scroll.h_cfg.shows_bar(&self.scroll.h);
        self.scroll.set_content(len);
        self.scroll
            .set_viewport(area.height.saturating_sub(u16::from(has_hb)) as usize);
        if self.follow {
            self.scroll.jump_end();
        }
    }

    /// Render into `area` (already the inner area of a panel).
    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.scroll.track_click = Some(click);
        self
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        style_line: fn(&Theme, &str) -> Style,
    ) {
        self.scroll.begin(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.scroll.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
        self.measure(area);
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let text_w = area.width.saturating_sub(2);
        let lines: &Vec<String> = if self.wrap {
            &self.wrapped_cache.1
        } else {
            &self.lines
        };
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        // rows the vertical viewport covers (the horizontal bar takes the
        // last one)
        let rows_h = (self.scroll.viewport_len as u16).min(area.height);
        ctx.autoscroll(Rect::new(area.x, area.y, area.width, rows_h));
        let sel = self.select.range();
        let hoff = if self.h_enabled() {
            self.scroll.h.offset
        } else {
            0
        };
        for (i, li) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let line = &lines[li];
            let st = style_line(t, line).bg(bg);
            let text = if hoff > 0 {
                crate::ui::text::fit(
                    &crate::ui::text::slice_cells(line, hoff, text_w as usize),
                    text_w as usize,
                )
            } else {
                crate::ui::text::fit(line, text_w as usize)
            };
            buf.set_string(area.x, y, &text, st);
            // the selected columns of the row; a selection that runs on to
            // the next row paints the cell after the line end too
            if let Some((a, b)) = sel
                && (a.line..=b.line).contains(&li)
            {
                let from = if li == a.line { a.col } else { 0 };
                let to = if li == b.line {
                    b.col
                } else {
                    crate::ui::text::width(line) + 1
                };
                for col in from.max(hoff)..to {
                    let x = col - hoff;
                    if x >= text_w as usize {
                        break;
                    }
                    buf[(area.x + x as u16, y)].set_style(t.selection());
                }
            }
        }
        if self.scroll.overflows() {
            let body = Rect::new(area.x, area.y, area.width, rows_h);
            self.scroll
                .draw_with_fade(body, buf, ctx, self.id, focused, &[]);
        }
        if rows_h < area.height && self.h_enabled() {
            // the corner under the vertical bar stays blank
            let y = area.y + rows_h;
            fill(
                buf,
                Rect::new(area.x, y, area.width, 1),
                Style::new().bg(bg),
            );
            let w = area
                .width
                .saturating_sub(u16::from(self.scroll.overflows()));
            let hb = Rect::new(area.x, y, w, 1);
            self.scroll.draw_h(hb, buf, ctx, self.id, focused);
        }
    }
}

/// Text selection: the shared [`TextSelect`] model over display rows.
impl ScrollPanel {
    /// The rows drawn: the wrapped rows, or the lines.
    fn rows(&self) -> &[String] {
        if self.wrap {
            &self.wrapped_cache.1
        } else {
            &self.lines
        }
    }

    /// The line a display row comes from.
    fn source_of(&self, row: usize) -> usize {
        if self.wrap {
            self.wrapped_cache.2.get(row).copied().unwrap_or(row)
        } else {
            row
        }
    }

    /// The text position (display row, column) under a screen position,
    /// clamped into the text; `None` with no text laid out.
    fn pos_at(&self, pos: Position) -> Option<CellPos> {
        let rows = self.rows();
        let rows_h = (self.scroll.viewport_len as u16).min(self.area.height);
        if self.area.is_empty() || rows.is_empty() || rows_h == 0 {
            return None;
        }
        let y = pos.y.clamp(self.area.y, self.area.y + rows_h - 1) - self.area.y;
        let line = (y as usize + self.scroll.offset).min(rows.len() - 1);
        let text_w = self.area.width.saturating_sub(2).max(1);
        let x = pos.x.saturating_sub(self.area.x).min(text_w - 1) as usize;
        let hoff = if self.h_enabled() {
            self.scroll.h.offset
        } else {
            0
        };
        let col = (x + hoff).min(crate::ui::text::width(&rows[line]));
        Some(CellPos { line, col })
    }

    /// The word at a text position.
    fn word_at(&self, p: CellPos) -> Option<(CellPos, CellPos)> {
        let row = self.rows().get(p.line)?;
        let cells = text_select::plain_cells(row);
        let at = text_select::plain_cell_at(&cells, p.col);
        let (s, e) = text_select::word_bounds(&cells, at, |c| text_select::is_word(c.g))?;
        let col = |i| text_select::plain_col_of(row, &cells, i);
        Some((
            CellPos {
                line: p.line,
                col: col(s),
            },
            CellPos {
                line: p.line,
                col: col(e),
            },
        ))
    }

    /// The line at a text position: every display row of it.
    fn line_at(&self, p: CellPos) -> Option<(CellPos, CellPos)> {
        let rows = self.rows();
        rows.get(p.line)?;
        let src = self.source_of(p.line);
        let mut first = p.line;
        while first > 0 && self.source_of(first - 1) == src {
            first -= 1;
        }
        let mut last = p.line;
        while last + 1 < rows.len() && self.source_of(last + 1) == src {
            last += 1;
        }
        Some((
            CellPos {
                line: first,
                col: 0,
            },
            CellPos {
                line: last,
                col: crate::ui::text::width(&rows[last]),
            },
        ))
    }

    /// The selection as (start, end) text positions in reading order.
    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        self.select.range()
    }

    /// The selected text: rows of one wrapped line rejoin with a space,
    /// lines with a newline; trailing spaces of each row are dropped.
    pub fn selected_text(&self) -> Option<String> {
        let (a, b) = self.selection()?;
        let rows = self.rows();
        let mut out = String::new();
        let last = b.line.min(rows.len().saturating_sub(1));
        for (li, row) in rows.iter().enumerate().take(last + 1).skip(a.line) {
            if li > a.line {
                out.push(if self.source_of(li) == self.source_of(li - 1) {
                    ' '
                } else {
                    '\n'
                });
            }
            let from = if li == a.line { a.col } else { 0 };
            let to = if li == b.line { b.col } else { usize::MAX };
            out.push_str(text_select::plain_slice(row, from, to).trim_end_matches(' '));
        }
        Some(out)
    }

    /// Select All: every line, ready for Copy. `Changed` when the
    /// selection changed.
    pub fn select_all(&mut self) -> Outcome {
        let rows = self.rows();
        let Some(last) = rows.len().checked_sub(1) else {
            return Outcome::Consumed;
        };
        let all = (
            CellPos { line: 0, col: 0 },
            CellPos {
                line: last,
                col: crate::ui::text::width(&rows[last]),
            },
        );
        if self.select.sel == Some(all) {
            return Outcome::Consumed;
        }
        self.select.set(all.0, all.1);
        Outcome::Changed
    }

    /// Drop the selection. `Changed` when there was one.
    pub fn clear_selection(&mut self) -> Outcome {
        self.select.clear()
    }

    /// A press with its multi-click `count` and modifiers (per
    /// [`Self::pointer_select`]): anchors a drag; Shift extends the
    /// selection from its anchor; a double press selects the word, a
    /// triple the line.
    fn on_press_with(&mut self, pos: Position, count: u8, mods: KeyModifiers) -> Outcome {
        let at = self.pos_at(pos);
        let word = at.and_then(|p| self.word_at(p));
        let line = at.and_then(|p| self.line_at(p));
        self.select
            .press(self.pointer_select, at, count, mods, word, line)
    }

    /// A drag held since a press: extend the selection (by words after a
    /// double press); past the top or bottom edge it scrolls a row per
    /// cell of distance.
    fn on_drag(&mut self, pos: Position) -> Outcome {
        if self.select.press.is_none() {
            return Outcome::Ignored;
        }
        let rows_h = (self.scroll.viewport_len as u16).min(self.area.height);
        let bottom = self.area.y + rows_h;
        if pos.y < self.area.y {
            self.scroll.scroll_by(-((self.area.y - pos.y) as isize));
            self.settle_follow();
        } else if pos.y >= bottom {
            self.scroll.scroll_by((pos.y + 1 - bottom) as isize);
            self.settle_follow();
        }
        let head = self.pos_at(pos);
        let word = if self.select.by_word() {
            head.and_then(|p| self.word_at(p))
        } else {
            None
        };
        self.select.drag(self.pointer_select, head, pos, word)
    }

    /// A completed click with its multi-click `count`: a double click
    /// selects the word, a triple click the line; the release that ends a
    /// drag or a Shift press keeps the selection.
    fn on_click_with(&mut self, pos: Position, count: u8) -> Outcome {
        let at = self.pos_at(pos);
        let word = at.and_then(|p| self.word_at(p));
        let line = at.and_then(|p| self.line_at(p));
        self.select
            .click(self.pointer_select, pos, count, word, line)
    }
}

impl Scrollable for ScrollPanel {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.scroll
    }

    /// Wrapped lines never scroll sideways.
    fn wheel_first(&mut self, axis: Axis, _delta: i32) -> Option<Response<()>> {
        (axis == Axis::Horizontal && !self.h_enabled()).then(Response::ignored)
    }

    /// Following resumes at the end and pauses anywhere else.
    fn after_scroll(&mut self, axis: Axis, _moved: bool) -> bool {
        if axis != Axis::Vertical {
            return false;
        }
        let was = self.follow;
        self.settle_follow();
        was != self.follow
    }
}

/// A panel drawn in a forced state for a States grid
/// ([`crate::ui::specimen`]): its chrome (title row, meta, the card fill
/// or the frame) over one content row. Focused shows how focus-within
/// reads under its [`PanelFocus`] (the title, or the title with the bar,
/// which the grid depicts); overflow gives it a meta far longer than its
/// title row, which gives way to the title.
impl crate::ui::specimen::Specimen for Panel<'static> {
    fn specimen_id(&self) -> WidgetId {
        WidgetId::of("panel-specimen")
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Focused | Overflow | Narrow)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        4
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        _bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let t = ctx.theme;
        let panel = Panel {
            title: self.title,
            kind: self.kind,
            focused: state == Focused,
            meta: if state == Overflow {
                Some("a meta far longer than the title row")
            } else {
                self.meta
            },
            badge: self.badge,
            bg_override: self.bg_override,
            focus_mark: self.focus_mark,
            title_min: self.title_min,
            meta_short: self.meta_short,
            pad_x: self.pad_x,
        };
        let inner = panel.render(area, buf, t);
        if !inner.is_empty() {
            let text = crate::ui::text::truncate("content", inner.width as usize);
            buf.set_string(inner.x, inner.y, text, t.secondary().bg(panel.bg(t)));
        }
    }
}

/// A scrolling panel drawn in a forced state for a States grid
/// ([`crate::ui::specimen`]), four rows tall in the secondary ink. The
/// pointer states are its scrollbar's: hover lights the thumb, a press
/// holds it in the accent (the one-glyph thumb rule). Overflow has more
/// lines than rows (the faded edge and the bar), empty has none, and the
/// narrow slot cuts its lines.
impl crate::ui::specimen::Specimen for ScrollPanel {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Hover | Pressed | Empty | Overflow | Narrow)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        4
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut panel = self.clone();
        match state {
            Empty => panel.lines.clear(),
            // the bar's states need a bar: content longer than the rows
            Hover | Pressed | Overflow => {
                let n = panel.lines.len();
                panel
                    .lines
                    .extend((n + 1..=12.max(n)).map(|i| format!("line {i}")));
            }
            _ => {}
        }
        let bar = scrollbar::id_for(panel.id);
        match state {
            Hover => ctx.interaction.hover = Some(bar),
            Pressed => {
                ctx.interaction.hover = Some(bar);
                ctx.interaction.pressed = Some(bar);
            }
            _ => {}
        }
        panel.render(area, buf, ctx, bg, |t, _| t.secondary());
    }
}

/// The panel's pointer gestures: a wheel or a bar press scrolls it
/// ([`crate::core::pointer::scroll_input`]); a press, a drag and a click
/// select text ([`ScrollPanel::on_press_with`], [`ScrollPanel::on_drag`],
/// [`ScrollPanel::on_click_with`]).
impl crate::core::pointer::MouseTarget for ScrollPanel {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::intent::Phase;
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
}

#[cfg(test)]
mod scroll_panel_tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Position;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect::<String>()
    }

    /// Draw `sample` in `state` on a 20×4 inert frame.
    fn sample(
        sample: &dyn crate::ui::specimen::Specimen,
        state: crate::ui::specimen::SpecimenState,
    ) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        sample.specimen(state, area, &mut buf, &mut ctx, theme.canvas);
        buf
    }

    #[test]
    fn a_scroll_panel_sample_lights_and_holds_its_thumb() {
        use crate::ui::specimen::SpecimenState::*;
        let theme = Theme::junie();
        let p = ScrollPanel::new(WidgetId::of("s"), vec!["one".into()]);
        // the thumb sits at the top of the bar column (offset 0)
        let thumb = |state| sample(&p, state)[(19, 0)].clone();
        let (rest, hover, held) = (thumb(Overflow), thumb(Hover), thumb(Pressed));
        for c in [&rest, &hover, &held] {
            assert_eq!(c.symbol(), scrollbar::THUMB, "one glyph in every state");
        }
        assert_ne!(rest.fg, hover.fg, "hover lights the thumb");
        assert_eq!(held.fg, theme.accent, "a held thumb is the accent");
        assert_eq!(row(&sample(&p, Empty), 0).trim(), "", "empty draws nothing");
        assert_ne!(
            sample(&p, Rest)[(19, 0)].symbol(),
            scrollbar::THUMB,
            "fits: no bar"
        );
    }

    #[test]
    fn a_panel_sample_shows_focus_within_by_its_mark() {
        use crate::ui::specimen::SpecimenState::*;
        let card = Panel::card(Some("Log")).meta("live");
        let bar = Panel::card(Some("Log"))
            .meta("live")
            .focus_mark(PanelFocus::Bar);
        assert!(!row(&sample(&card, Focused), 0).contains('▎'));
        assert!(row(&sample(&bar, Focused), 0).contains('▎'));
        assert!(!row(&sample(&bar, Rest), 0).contains('▎'));
        assert!(row(&sample(&card, Rest), 2).contains("content"));
        assert!(
            !row(&sample(&card, Overflow), 0).contains("live"),
            "the long meta gives way"
        );
    }

    fn draw_panel(p: &mut ScrollPanel, area: Rect) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(area);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            p.render(area, &mut buf, &mut ctx, theme.surface, |t, _| t.muted());
        }
        (buf, hits)
    }

    #[test]
    fn lines_wider_than_the_panel_scroll_sideways_with_a_visible_position() {
        let area = Rect::new(0, 0, 20, 4);
        let lines = vec![
            "CREATE TABLE customers (id uuid PRIMARY KEY, email text)".to_owned(),
            "short".to_owned(),
        ];
        let mut p = ScrollPanel::new(WidgetId::of("ddl"), lines);
        let (buf, hits) = draw_panel(&mut p, area);
        let bar = row(&buf, 3);
        assert!(
            bar.starts_with(scrollbar::H_THUMB),
            "bar on the last row: {bar:?}"
        );
        assert!(bar.contains(scrollbar::H_TRACK));
        assert_eq!(
            hits.hit(Position::new(10, 3)),
            Some(scrollbar::h_id_for(p.id))
        );
        assert!(row(&buf, 0).starts_with("CREATE TABLE"));

        // → scrolls by the step, the line shows a leading … and the
        // thumb moves right
        assert_eq!(p.on_key(&key(KeyCode::Right)), Outcome::Changed);
        assert_eq!(p.scroll.h.offset, HScroll::default().step);
        let (buf, _) = draw_panel(&mut p, area);
        assert!(row(&buf, 0).starts_with('…'), "{}", row(&buf, 0));
        assert!(!row(&buf, 3).starts_with(scrollbar::H_THUMB));

        // the wheel and a drag on the bar reach the end
        assert_eq!(p.wheel(Axis::Horizontal, 1).outcome(), Outcome::Changed);
        let (buf, _) = draw_panel(&mut p, area);
        let thumb = (0..20)
            .find(|&x| buf[(x, 3)].symbol() == scrollbar::H_THUMB)
            .unwrap();
        assert_eq!(
            p.bar_press(Position::new(thumb, 3))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Consumed
        );
        assert_eq!(p.bar_drag(Position::new(40, 3)).outcome(), Outcome::Changed);
        assert_eq!(p.scroll.h.offset, p.scroll.h.max_offset());
        assert_eq!(p.scroll.offset, 0, "rows did not move");

        // wrapping text never scrolls sideways; neither does a disabled policy
        let mut wrapped = ScrollPanel::new(WidgetId::of("w"), p.lines.clone()).wrap(true);
        let (buf, _) = draw_panel(&mut wrapped, area);
        assert!(!row(&buf, 3).contains(scrollbar::H_THUMB));
        assert_eq!(wrapped.on_key(&key(KeyCode::Right)), Outcome::Ignored);
        let mut off =
            ScrollPanel::new(WidgetId::of("o"), p.lines.clone()).horizontal(HScroll::off());
        draw_panel(&mut off, area);
        assert_eq!(off.wheel(Axis::Horizontal, 1).outcome(), Outcome::Ignored);
    }

    #[test]
    fn a_focused_card_shows_focus_by_its_title_not_a_second_bar() {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 3));
        Panel::card(Some("Language"))
            .focused(true)
            .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
        assert!(!row(&buf, 0).contains('▎'), "{}", row(&buf, 0));
        assert_eq!(buf[(2, 0)].fg, t.text_primary, "focus-within: the title");
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 3));
        Panel::card(Some("Language"))
            .focused(true)
            .focus_mark(PanelFocus::Bar)
            .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
        assert_eq!(buf[(1, 0)].symbol(), "▎", "the bar stays available");
    }

    fn title_of(title: &str, meta: &str, short: Option<&str>, w: u16) -> String {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 3));
        let mut p = Panel::card(Some(title)).meta(meta);
        if let Some(s) = short {
            p = p.meta_short(s);
        }
        p.render(Rect::new(0, 0, w, 3), &mut buf, &t);
        row(&buf, 0)
    }

    /// Layout audit #7: the title gave way to the meta down to 4 cells
    /// ("Lan… 1–13 of …", both cut). The title now keeps `title_min` cells
    /// (12 by default, its whole width when shorter) and the meta yields
    /// first: its short form, then an ellipsis, then nothing.
    #[test]
    fn the_title_keeps_title_min_and_the_meta_shortens_first() {
        // room for both
        let r = title_of("Languages", "1–13 of 24", None, 30);
        assert!(r.contains("Languages") && r.contains("1–13 of 24"), "{r}");
        // a position label shortens on its own before the title is cut
        let r = title_of("Languages", "1–13 of 24", None, 22);
        assert!(r.contains("Languages"), "the title stays whole: {r}");
        assert!(r.contains("1–13/24"), "{r}");
        // an explicit short form wins
        let r = title_of(
            "customers",
            "rows 1–10 of 40 loaded · ~4,812 total",
            Some("1–10/40"),
            24,
        );
        assert!(r.contains("customers") && r.contains("1–10/40"), "{r}");
        // no room left: a number is never cut mid-way ("1–1…" misstates the
        // position), so the meta is dropped; the title never is
        let r = title_of("Languages", "1–13 of 24", None, 18);
        assert!(r.contains("Languages") && !r.contains('1'), "{r}");
        // a text meta is cut with an ellipsis
        let r = title_of("Log", "Enter copies the row", None, 20);
        assert!(
            r.contains("Log") && r.contains("Enter cop") && r.contains('…'),
            "{r}"
        );
        // a long meta keeps its first fact whole rather than cutting
        // inside the next number
        let r = title_of(
            "customers",
            "rows 1–10 of 40 loaded · ~4,812 total",
            None,
            40,
        );
        assert!(r.contains("rows 1–10/40 loaded"), "{r}");
        assert!(!r.contains("~4"), "{r}");
        assert_eq!(cut_meta("1–13/24", 4, "…"), None, "never \"1–1…\"");
        assert_eq!(
            cut_meta("rows 1–10/40 · ~4,812", 16, "…").as_deref(),
            Some("rows 1–10/40…")
        );
        // a long title is cut to title_min, not to 4 cells
        let r = title_of("A rather long title", "70–100 of 120", None, 28);
        assert!(r.contains("A rather lo") && r.contains("70–100/120"), "{r}");
        // configurable: a title_min of 4 restores the old balance
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 3));
        Panel::card(Some("A rather long title"))
            .meta("70–100 of 120")
            .title_min(4)
            .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
        assert!(row(&buf, 0).contains("70–100 of 120"), "{}", row(&buf, 0));
    }

    /// Layout audit #17: card prose cut at the bottom read as complete.
    #[test]
    fn a_clipped_card_body_ends_in_an_ellipsis() {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));
        let p = Panel::card(Some("Notes"));
        let inner = p.render(Rect::new(0, 0, 20, 4), &mut buf, &t);
        p.mark_clipped(inner, &mut buf, &t);
        assert_eq!(buf[(inner.right() - 1, inner.bottom() - 1)].symbol(), "…");
    }

    #[test]
    fn draw_meta_repaints_the_title_row_after_the_content() {
        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 4));
        let panel = Panel::card(Some("Log"));
        panel.render(Rect::new(0, 0, 30, 4), &mut buf, &t);
        assert!(!row(&buf, 0).contains("of"));
        panel.draw_meta(Rect::new(0, 0, 30, 4), &mut buf, &t, "1–2 of 9 · following");
        assert!(
            row(&buf, 0).contains("1–2 of 9 · following"),
            "{}",
            row(&buf, 0)
        );
        assert!(row(&buf, 0).contains("Log"));
    }

    fn drawn(p: &mut ScrollPanel, w: u16, h: u16) -> Buffer {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        p.render(
            Rect::new(0, 0, w, h),
            &mut buf,
            &mut ctx,
            theme.canvas,
            |t, _| t.secondary(),
        );
        buf
    }

    #[test]
    fn a_tailing_panel_resumes_at_the_end_and_a_prose_panel_never_follows() {
        let lines: Vec<String> = (1..=30).map(|i| format!("line {i}")).collect();
        let mut log = ScrollPanel::new(WidgetId::of("log"), lines.clone()).tail(true);
        drawn(&mut log, 20, 5);
        assert!(log.follow && log.scroll.at_end());
        // a wheel down at the tail moves nothing and keeps following
        assert_eq!(log.wheel(Axis::Vertical, 3).outcome(), Outcome::Consumed);
        assert!(log.follow);
        // scrolling up pauses; the wheel back to the end resumes
        assert_eq!(log.wheel(Axis::Vertical, -3).outcome(), Outcome::Changed);
        assert!(!log.follow);
        log.push("line 31".into());
        drawn(&mut log, 20, 5);
        assert!(
            !row(&drawn(&mut log, 20, 5), 4).contains("line 31"),
            "paused: new lines stay below"
        );
        assert_eq!(log.wheel(Axis::Vertical, 6).outcome(), Outcome::Changed);
        assert!(log.follow, "the end resumes following");
        assert_eq!(
            log.on_key(&key(KeyCode::Down)),
            Outcome::Consumed,
            "nothing below"
        );
        assert!(log.follow, "a no-op keeps following");
        assert_eq!(log.on_key(&key(KeyCode::PageUp)), Outcome::Changed);
        assert!(!log.follow);
        assert_eq!(log.on_key(&key(KeyCode::Char('f'))), Outcome::Changed);
        assert!(log.follow && log.scroll.at_end());
        // the scrollbar behaves the same: the thumb dragged to the end follows
        log.scroll.track_click = Some(crate::core::scroll::TrackClick::Jump);
        drawn(&mut log, 20, 5);
        assert_eq!(
            log.bar_press(Position::new(19, 0))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert!(!log.follow);
        assert_eq!(
            log.bar_drag(Position::new(19, 4)).outcome(),
            Outcome::Changed
        );
        assert!(log.follow);

        let mut prose = ScrollPanel::new(WidgetId::of("prose"), lines).wrap(true);
        drawn(&mut prose, 20, 5);
        assert_eq!(
            prose.on_key(&key(KeyCode::Char('f'))),
            Outcome::Ignored,
            "no follow verb"
        );
        assert_eq!(prose.on_key(&key(KeyCode::End)), Outcome::Changed);
        assert!(!prose.follow, "End is a jump, never a follow");
        // a resize after End keeps the offset clamped, not re-anchored
        prose.on_key(&key(KeyCode::Up));
        let before = prose.scroll.offset;
        drawn(&mut prose, 20, 10);
        assert!(prose.scroll.offset <= before);
        assert!(!prose.follow);
    }

    #[test]
    fn measure_makes_the_label_exact_before_drawing() {
        let mut p = ScrollPanel::new(
            WidgetId::of("p"),
            vec!["a word a word a word a word a word a word".into()],
        )
        .wrap(true);
        assert_eq!(
            scrollbar::position_label(&p.scroll),
            "",
            "nothing laid out yet"
        );
        p.measure(Rect::new(0, 0, 12, 2));
        assert!(p.scroll.overflows());
        assert_eq!(
            scrollbar::position_label(&p.scroll),
            format!("1–2 of {}", p.scroll.content_len)
        );
    }
}

#[cfg(test)]
mod title_reveal_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;
    use crate::ui::reveal::cut_id;
    use ratatui::layout::Position;

    const TITLE: &str = "Connections to the production analytics warehouse";

    /// A framed panel squeezed to 24 columns inside a 70-column screen,
    /// with the pointer on `hover`: the screen text and the hit registry.
    fn frame(hover: Option<WidgetId>) -> (String, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            hover,
            ..Default::default()
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 4));
        {
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            ctx.clickable(WidgetId::of("card"), Rect::new(0, 0, 24, 4));
            Panel::framed(Some(TITLE)).render(Rect::new(0, 0, 24, 4), &mut buf, &theme);
            ctx.flush_overlays(&mut buf);
        }
        let text = (0..4)
            .map(|y| (0..70).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        (text, hits)
    }

    #[test]
    fn a_cut_title_reveals_on_hover_and_leaves_clicks_to_the_panel() {
        let (text, hits) = frame(None);
        assert!(text.contains('…'), "the title is cut: {text}");
        assert!(!text.contains(TITLE));
        // the title's cells hover as the title, click as the panel
        let on_title = Position::new(4, 0);
        let id = hits
            .hover_at(on_title)
            .expect("a hover region on the title");
        assert_eq!(id, cut_id(Rect::new(3, 0, 1, 1)));
        assert_eq!(hits.hit(on_title), Some(WidgetId::of("card")));
        let (text, _) = frame(Some(id));
        assert!(text.contains(TITLE), "hover shows the full title: {text}");
    }

    #[test]
    fn a_title_that_fits_registers_nothing() {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 4));
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        Panel::framed(Some("Short")).render(Rect::new(0, 0, 40, 4), &mut buf, &theme);
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        assert_eq!(hits.hover_at(Position::new(4, 0)), None);
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::scroll::TrackClick;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    fn draw(p: &mut ScrollPanel, click: TrackClick) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        ctx.track_click = click;
        p.render(area, &mut buf, &mut ctx, theme.surface, |t, _| t.muted());
    }

    #[test]
    fn the_app_default_makes_a_vertical_track_click_page() {
        let lines: Vec<String> = (0..50).map(|i| format!("line {i}")).collect();
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines.clone());
        draw(&mut p, TrackClick::Page);
        assert_eq!(
            p.bar_press(Position::new(19, 9))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert_eq!(p.scroll.offset, 10, "one page, not a jump to the end");
        // the widget's own setting wins over the app default
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines).track_click(TrackClick::Jump);
        draw(&mut p, TrackClick::Page);
        p.bar_press(Position::new(19, 9))
            .outcome()
            .or(Outcome::Consumed);
        assert_eq!(p.scroll.offset, 40);
    }
}

#[cfg(test)]
mod pad_tests {
    use super::*;

    fn top(buf: &Buffer) -> String {
        (0..buf.area.width).map(|x| buf[(x, 0)].symbol()).collect()
    }

    #[test]
    fn a_card_keeps_its_padding_and_a_narrow_one_can_take_less() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 19, 6);
        let mut buf = Buffer::empty(area);
        let inner = Panel::card(Some("Components")).render(area, &mut buf, &theme);
        assert_eq!(inner, Rect::new(2, 2, 15, 3), "the default card inset");
        assert_eq!(top(&buf), "  Components       ");

        let mut buf = Buffer::empty(area);
        let panel = Panel::card(Some("Components")).pad_x(1).meta("25");
        let inner = panel.render(area, &mut buf, &theme);
        assert_eq!(inner, Rect::new(1, 2, 17, 3));
        // the title and its meta keep one blank column at either edge
        assert_eq!(top(&buf), " Components     25 ");
        panel.draw_meta(area, &mut buf, &theme, "3/25");
        assert_eq!(top(&buf), " Components   3/25 ");

        // a framed panel keeps its own geometry
        let framed =
            Panel::framed(Some("F"))
                .pad_x(1)
                .render(area, &mut Buffer::empty(area), &theme);
        let plain = Panel::framed(Some("F")).render(area, &mut Buffer::empty(area), &theme);
        assert_eq!(framed, plain);
    }
}

/// M31: a `ScrollPanel` (a log pane) selects its text with the pointer,
/// the way a `TextViewport` does, and copies it.
#[cfg(test)]
mod selection_tests {
    use super::*;
    use crate::core::edit::clipboard;
    use crate::core::intent::Phase;
    use crate::core::pointer::{MouseTarget, PointerEvent};
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Position;

    const AREA: Rect = Rect::new(0, 0, 30, 4);

    fn draw(p: &mut ScrollPanel) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(AREA);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            p.render(AREA, &mut buf, &mut ctx, theme.surface, |t, _| t.muted());
        }
        (buf, hits)
    }

    fn send(
        p: &mut ScrollPanel,
        hits: &HitRegistry,
        phase: Phase,
        at: (u16, u16),
        count: u8,
        mods: KeyModifiers,
    ) {
        let ev = PointerEvent::new(hits, phase, p.id, Position::new(at.0, at.1), count, mods);
        let _ = p.pointer(&ev);
    }

    fn click(
        p: &mut ScrollPanel,
        hits: &HitRegistry,
        at: (u16, u16),
        count: u8,
        mods: KeyModifiers,
    ) {
        send(p, hits, Phase::Press, at, count, mods);
        send(p, hits, Phase::Click, at, count, mods);
    }

    fn copied(p: &mut ScrollPanel) -> Option<String> {
        clipboard::copy("");
        let _ = p.on_key(&Key {
            code: KeyCode::Char('y'),
            mods: KeyModifiers::NONE,
        });
        clipboard::text().filter(|t| !t.is_empty())
    }

    fn selected(buf: &Buffer, x: u16, y: u16) -> bool {
        buf[(x, y)].bg == Theme::junie().selection().bg.unwrap()
    }

    #[test]
    fn a_scroll_panel_selects_by_drag_shift_click_word_and_line_and_copies() {
        let lines = ["alpha beta gamma", "second line here", "third"];
        let mut p = ScrollPanel::new(
            WidgetId::of("log"),
            lines.iter().map(|l| l.to_string()).collect(),
        );
        let none = KeyModifiers::NONE;
        let (_, hits) = draw(&mut p);

        // a drag from "beta" to the second row selects across the rows
        send(&mut p, &hits, Phase::Press, (6, 0), 1, none);
        send(&mut p, &hits, Phase::Drag, (3, 1), 1, none);
        send(&mut p, &hits, Phase::Click, (3, 1), 1, none);
        let (buf, _) = draw(&mut p);
        assert!(
            selected(&buf, 6, 0) && selected(&buf, 2, 1),
            "drawn selected"
        );
        assert!(!selected(&buf, 5, 0) && !selected(&buf, 3, 1));
        assert_eq!(copied(&mut p).as_deref(), Some("beta gamma\nsec"));

        // Shift+click extends from the anchor
        click(&mut p, &hits, (1, 2), 1, KeyModifiers::SHIFT);
        assert_eq!(
            copied(&mut p).as_deref(),
            Some("beta gamma\nsecond line here\nt")
        );

        // a double click selects the word, a triple click the line
        click(&mut p, &hits, (8, 1), 1, none);
        click(&mut p, &hits, (8, 1), 2, none);
        assert_eq!(copied(&mut p).as_deref(), Some("line"));
        click(&mut p, &hits, (1, 0), 1, none);
        click(&mut p, &hits, (1, 0), 2, none);
        click(&mut p, &hits, (1, 0), 3, none);
        assert_eq!(copied(&mut p).as_deref(), Some("alpha beta gamma"));

        // a single click clears; Select All then Copy copies every line
        click(&mut p, &hits, (1, 0), 1, none);
        assert_eq!(copied(&mut p), None);
        let _ = p.on_key(&Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        });
        assert_eq!(copied(&mut p).as_deref(), Some(lines.join("\n").as_str()));
        // Esc clears it
        let o = p.on_key(&Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        });
        assert_eq!((o, copied(&mut p)), (Outcome::Changed, None));
    }

    #[test]
    fn a_wrapped_panel_selects_a_whole_line_and_copies_it_rejoined() {
        let mut p = ScrollPanel::new(
            WidgetId::of("prose"),
            vec!["one two three four five six seven".into(), "next".into()],
        )
        .wrap(true);
        let (_, hits) = draw(&mut p);
        for count in 1..=3 {
            click(&mut p, &hits, (1, 0), count, KeyModifiers::NONE);
        }
        assert_eq!(
            copied(&mut p).as_deref(),
            Some("one two three four five six seven")
        );
    }
}
