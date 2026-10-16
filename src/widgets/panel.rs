//! Panels: the surface container. Two flavours:
//! - **card**: filled `surface` rectangle, no border, title row. The default.
//! - **framed**: rounded subtle border, used when a region must read as a
//!   distinct pane (split views, dialogs).
//!
//! Focus at container level is shown by the border/title only; the accent
//! gutter bar belongs to the focused control inside.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::scroll::{HScroll, ScrollState, ScrollbarVisibility};
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

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
/// than [`META_MIN`].
fn cut_meta(meta: &str, room: u16) -> Option<String> {
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
    (crate::ui::text::width(body) as u16 + 1 >= META_MIN).then(|| format!("{body}…"))
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
}

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
        };
        let row = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), 1);
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
        buf.set_string(inner.right() - 1, inner.bottom() - 1, "…", t.faint().bg(bg));
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
                let inner = area.inner(ratatui::layout::Margin::new(2, 1));
                if self.focused && self.title.is_some() && self.focus_mark == PanelFocus::Bar {
                    // opt-in: the container itself is the focus stop
                    let s = crate::ui::ctx::VisualState {
                        focused: true,
                        ..Default::default()
                    };
                    buf.set_string(
                        area.x + 1,
                        area.y,
                        t.gutter_symbol(s),
                        t.gutter(s, bg, false),
                    );
                }
                self.title_row(area.x + 2, area.y, area.width.saturating_sub(4), buf, t, bg);
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
                    .border_type(BorderType::Rounded)
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
                cut_meta(&short, room)
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
#[derive(Debug, Clone)]
pub struct ScrollPanel {
    pub id: WidgetId,
    pub lines: Vec<String>,
    pub scroll: ScrollState,
    /// Following the tail right now (only ever true for a tailing panel).
    pub follow: bool,
    /// The panel tails its content: new lines keep the end in view.
    pub tail: bool,
    pub wrap: bool,
    pub area: Rect,
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    wrapped_cache: (u16, Vec<String>),
    /// Horizontal position in cells (only while not wrapping): lines wider
    /// than the panel scroll sideways instead of being cut.
    pub hscroll: ScrollState,
    /// Horizontal scrolling policy; `None` follows [`RenderCtx::hscroll`].
    pub horizontal: Option<HScroll>,
    /// The policy resolved at the last render (events use it).
    h_cfg: HScroll,
    /// The horizontal scrollbar track as last drawn.
    htrack: Rect,
    /// The last scrollbar press landed on the horizontal bar.
    h_held: bool,
    /// What a press on the bare vertical track does; `None` follows
    /// [`RenderCtx::track_click`](crate::ui::ctx::RenderCtx::track_click).
    pub track_click: Option<crate::core::scroll::TrackClick>,
    /// The vertical track policy resolved at the last render.
    v_click: crate::core::scroll::TrackClick,
    /// Which keys scroll (default [`NavKeys::DEFAULT`]; ←/→ scroll
    /// sideways, since the panel has no caret).
    pub nav: NavKeys,
    /// The panel's own action keys (default [`PanelAction::default_keymap`]).
    pub keymap: KeyMap<PanelAction>,
}

/// A [`ScrollPanel`] action a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelAction {
    /// A tailing panel: pause following, or jump to the end and follow
    /// again. Default `f`.
    Follow,
}

impl Actions for PanelAction {
    fn default_keymap() -> KeyMap<Self> {
        vec![(KeyBinding::char('f'), PanelAction::Follow)]
    }
}

impl ScrollPanel {
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::keymap`].
    pub fn keymap(mut self, keymap: KeyMap<PanelAction>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn new(id: WidgetId, lines: Vec<String>) -> Self {
        Self {
            track_click: None,
            v_click: crate::core::scroll::TrackClick::Jump,
            id,
            lines,
            scroll: ScrollState::default(),
            follow: false,
            tail: false,
            wrap: false,
            area: Rect::ZERO,
            track: Rect::ZERO,
            wrapped_cache: (0, vec![]),
            hscroll: ScrollState::default(),
            horizontal: None,
            h_cfg: HScroll::default(),
            htrack: Rect::ZERO,
            h_held: false,
            nav: NavKeys::DEFAULT,
            keymap: PanelAction::default_keymap(),
        }
    }

    /// Override the app-wide horizontal scrolling policy.
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    /// The panel itself or one of its scrollbars.
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || scrollbar::is_bar_of(id, self.id)
    }

    /// Lines scroll sideways: not wrapping and the policy allows it.
    fn h_enabled(&self) -> bool {
        !self.wrap && self.h_cfg.enabled
    }

    /// Scroll sideways by `delta` steps; reports a change like
    /// [`on_wheel`](Self::on_wheel).
    fn scroll_h(&mut self, delta: isize) -> Outcome {
        // nothing wider than the panel: the arrows stay free for the page
        if !self.h_enabled() || !self.hscroll.overflows() {
            return Outcome::Ignored;
        }
        let step = self.h_cfg.step.max(1) as isize;
        if self.hscroll.scroll_by(delta * step) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Horizontal wheel (or Shift+wheel): `step` cells per notch.
    pub fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        self.scroll_h(delta.signum() as isize)
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
        if let Some(PanelAction::Follow) = lookup(&self.keymap, key)
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

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        let moved = self.scroll.scroll_by(delta as isize);
        let was = self.follow;
        self.settle_follow();
        if moved || was != self.follow {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    fn track(&self) -> Rect {
        if self.track.is_empty() {
            Rect::new(
                self.area.right().saturating_sub(1),
                self.area.y,
                1,
                self.area.height,
            )
        } else {
            self.track
        }
    }

    /// The pointer went down on either scrollbar (or a completed click);
    /// a press on the horizontal bar ([`scrollbar::h_id_for`]) moves
    /// sideways.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        self.h_held = !self.htrack.is_empty() && self.htrack.contains(pos);
        if self.h_held {
            let click = self.h_cfg.track_click;
            return if scrollbar::press_h(self.htrack, pos, &mut self.hscroll, click) {
                Outcome::Changed
            } else {
                Outcome::Consumed
            };
        }
        let moved = scrollbar::press_with(self.track(), pos, &mut self.scroll, self.v_click);
        self.settle_follow();
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged while holding `pressed` (either of this
    /// widget's scrollbar ids): the axis follows the id, so a drag works
    /// whether or not the press was routed first.
    pub fn on_bar_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        self.h_held = pressed == scrollbar::h_id_for(self.id);
        self.on_scrollbar_drag(pos)
    }

    /// The pointer dragged along the scrollbar it pressed.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        if self.h_held && !self.htrack.is_empty() {
            return if scrollbar::drag_h(self.htrack, pos, &mut self.hscroll) {
                Outcome::Changed
            } else {
                Outcome::Consumed
            };
        }
        let moved = scrollbar::drag(self.track(), pos, &mut self.scroll);
        self.settle_follow();
        if moved {
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
            for l in &self.lines {
                out.extend(crate::ui::text::wrap(l, text_w as usize));
            }
            self.wrapped_cache = (text_w, out);
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
        self.hscroll.set_content(widest);
        self.hscroll.set_viewport(text_w as usize);
        let has_hb = self.h_enabled() && area.height > 1 && self.h_cfg.shows_bar(&self.hscroll);
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
        self.track_click = Some(click);
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
        self.v_click = self.track_click.unwrap_or(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.h_cfg = self.horizontal.unwrap_or(ctx.hscroll);
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
        for (i, li) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let line = &lines[li];
            let st = style_line(t, line).bg(bg);
            let text = if self.h_enabled() && self.hscroll.offset > 0 {
                crate::ui::text::fit(
                    &crate::ui::text::slice_cells(line, self.hscroll.offset, text_w as usize),
                    text_w as usize,
                )
            } else {
                crate::ui::text::fit(line, text_w as usize)
            };
            buf.set_string(area.x, y, &text, st);
        }
        // rows the vertical viewport covers (the horizontal bar takes the
        // last one)
        let rows_h = (self.scroll.viewport_len as u16).min(area.height);
        if self.scroll.overflows() {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    rows_h,
                ),
                &self.scroll,
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, rows_h);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        } else {
            self.track = Rect::ZERO;
        }
        self.htrack = Rect::ZERO;
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
            self.htrack = hb;
            scrollbar::render_horizontal_with(
                hb,
                buf,
                ctx,
                self.id,
                &self.hscroll,
                focused,
                self.h_cfg.bar == ScrollbarVisibility::Always,
            );
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
        assert_eq!(p.hscroll.offset, HScroll::default().step);
        let (buf, _) = draw_panel(&mut p, area);
        assert!(row(&buf, 0).starts_with('…'), "{}", row(&buf, 0));
        assert!(!row(&buf, 3).starts_with(scrollbar::H_THUMB));

        // the wheel and a drag on the bar reach the end
        assert_eq!(p.on_wheel_h(1), Outcome::Changed);
        let (buf, _) = draw_panel(&mut p, area);
        let thumb = (0..20)
            .find(|&x| buf[(x, 3)].symbol() == scrollbar::H_THUMB)
            .unwrap();
        assert_eq!(p.on_scrollbar(Position::new(thumb, 3)), Outcome::Consumed);
        assert_eq!(p.on_scrollbar_drag(Position::new(40, 3)), Outcome::Changed);
        assert_eq!(p.hscroll.offset, p.hscroll.max_offset());
        assert_eq!(p.scroll.offset, 0, "rows did not move");

        // wrapping text never scrolls sideways; neither does a disabled policy
        let mut wrapped = ScrollPanel::new(WidgetId::of("w"), p.lines.clone()).wrap(true);
        let (buf, _) = draw_panel(&mut wrapped, area);
        assert!(!row(&buf, 3).contains(scrollbar::H_THUMB));
        assert_eq!(wrapped.on_key(&key(KeyCode::Right)), Outcome::Ignored);
        let mut off =
            ScrollPanel::new(WidgetId::of("o"), p.lines.clone()).horizontal(HScroll::off());
        draw_panel(&mut off, area);
        assert_eq!(off.on_wheel_h(1), Outcome::Ignored);
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
        assert_eq!(cut_meta("1–13/24", 4), None, "never \"1–1…\"");
        assert_eq!(
            cut_meta("rows 1–10/40 · ~4,812", 16).as_deref(),
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
        assert_eq!(log.on_wheel(3), Outcome::Consumed);
        assert!(log.follow);
        // scrolling up pauses; the wheel back to the end resumes
        assert_eq!(log.on_wheel(-3), Outcome::Changed);
        assert!(!log.follow);
        log.push("line 31".into());
        drawn(&mut log, 20, 5);
        assert!(
            !row(&drawn(&mut log, 20, 5), 4).contains("line 31"),
            "paused: new lines stay below"
        );
        assert_eq!(log.on_wheel(6), Outcome::Changed);
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
        drawn(&mut log, 20, 5);
        assert_eq!(log.on_scrollbar(Position::new(19, 0)), Outcome::Changed);
        assert!(!log.follow);
        assert_eq!(
            log.on_scrollbar_drag(Position::new(19, 4)),
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
        assert_eq!(p.on_scrollbar(Position::new(19, 9)), Outcome::Changed);
        assert_eq!(p.scroll.offset, 10, "one page, not a jump to the end");
        // the widget's own setting wins over the app default
        let mut p = ScrollPanel::new(WidgetId::of("p"), lines).track_click(TrackClick::Jump);
        draw(&mut p, TrackClick::Page);
        p.on_scrollbar(Position::new(19, 9));
        assert_eq!(p.scroll.offset, 40);
    }
}
