//! Panels: the surface container. Two flavours:
//! - **card**: filled `surface` rectangle, no border, title row. The default.
//! - **framed**: rounded subtle border, used when a region must read as a
//!   distinct pane (split views, dialogs).
//!
//! Focus at container level is shown by the border/title only; the accent
//! gutter bar belongs to the focused control inside.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Card,
    Framed,
}

/// Cells a title keeps when a long meta competes for the title row.
const TITLE_MIN: u16 = 4;

pub struct Panel<'a> {
    pub title: Option<&'a str>,
    pub kind: PanelKind,
    pub focused: bool,
    /// Right-aligned text in the title row (position label, badge).
    pub meta: Option<&'a str>,
    pub badge: Option<(&'a str, crate::theme::BadgeKind)>,
    pub bg_override: Option<Color>,
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
        }
    }
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    pub fn meta(mut self, m: &'a str) -> Self {
        self.meta = Some(m);
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
        };
        let row = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), 1);
        fill(buf, row, Style::new().bg(bg));
        late.title_row(row.x, row.y, row.width, buf, t, bg);
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
                if self.focused && self.title.is_some() {
                    // container focus: the same bar as a control, in the padding column
                    buf.set_string(area.x + 1, area.y, "▎", Style::new().fg(t.focus).bg(bg));
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
        // the meta (a scroll position) is state the reader relies on: when
        // both do not fit, the title yields first down to a few cells that
        // still name the panel, then the meta shortens
        let title_min = self
            .title
            .map(|t| (crate::ui::text::width(t) as u16).min(TITLE_MIN))
            .unwrap_or(0);
        let meta = self.meta.map(|m| {
            let room = w.saturating_sub(pad + if title_min > 0 { title_min + 1 } else { 0 });
            if crate::ui::text::width(m) as u16 > room {
                crate::ui::text::truncate(m, room as usize)
            } else {
                m.to_owned()
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
            let title = crate::ui::text::truncate(title, room as usize);
            let title = if framed { format!(" {title} ") } else { title };
            buf.set_string(cx, y, &title, style);
            cx += crate::ui::text::width(&title) as u16;
        }
        let mut right = x + w;
        if let Some(meta) = meta {
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
}

impl ScrollPanel {
    pub fn new(id: WidgetId, lines: Vec<String>) -> Self {
        Self {
            id,
            lines,
            scroll: ScrollState::default(),
            follow: false,
            tail: false,
            wrap: false,
            area: Rect::ZERO,
            track: Rect::ZERO,
            wrapped_cache: (0, vec![]),
        }
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
        let moved = match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => self.scroll.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.scroll.scroll_by(1),
            KeyCode::PageUp => self.scroll.page_up(),
            KeyCode::PageDown => self.scroll.page_down(),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.scroll.jump_start(),
            KeyCode::End | KeyCode::Char('G') => self.scroll.jump_end(),
            KeyCode::Char('f') if key.plain() && self.tail => {
                let was = self.follow;
                if was {
                    self.follow = false;
                } else {
                    self.scroll.jump_end();
                    self.follow = true;
                }
                return Outcome::Changed;
            }
            _ => return Outcome::Ignored,
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

    /// The pointer went down on the scrollbar (or a completed click).
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let moved = scrollbar::press(self.track(), pos, &mut self.scroll);
        self.settle_follow();
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
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
        self.scroll.set_content(len);
        self.scroll.set_viewport(area.height as usize);
        if self.follow {
            self.scroll.jump_end();
        }
    }

    /// Render into `area` (already the inner area of a panel).
    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        style_line: fn(&Theme, &str) -> Style,
    ) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
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
            let text = crate::ui::text::fit(line, text_w as usize);
            buf.set_string(area.x, y, &text, st);
        }
        if self.scroll.overflows() {
            crate::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    area.height,
                ),
                &self.scroll,
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        } else {
            self.track = Rect::ZERO;
        }
    }
}

#[cfg(test)]
mod scroll_panel_tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

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

    #[test]
    fn the_meta_is_never_dropped_the_title_yields_first() {
        let t = Theme::junie();
        // 24 columns: the title has 20; the meta needs 13 of them
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 3));
        Panel::card(Some("A rather long title"))
            .meta("70–100 of 120")
            .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
        let title = row(&buf, 0);
        assert!(title.contains("70–100 of 120"), "{title}");
        assert!(
            title.contains("A ra") && title.contains('…'),
            "the title shortened: {title}"
        );
        // narrower than the meta itself: the meta shortens, still present
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 3));
        Panel::card(Some("Log"))
            .meta("371–401 of 401 · following")
            .render(Rect::new(0, 0, 12, 3), &mut buf, &t);
        let title = row(&buf, 0);
        assert!(title.contains("Log"), "the title keeps its name: {title}");
        assert!(title.contains("371"), "{title}");
        assert!(title.contains('…'));
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
