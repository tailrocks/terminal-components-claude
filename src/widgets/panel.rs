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
use crate::ui::edit_menu::{
    EditCaps, EditMenu, EditMenuHost, edit_menu_key, edit_menu_pointer, edit_menu_route,
};
use crate::widgets::field_common::PointerSelect;
use crate::widgets::scrollbar;
use crate::widgets::text_select::{self, TextSelect};
use crate::widgets::viewport::CellPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Card,
    Framed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelFocus {
    #[default]
    Title,

    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CardHeading {
    Plain,

    #[default]
    Rule,

    Band,

    Emphasis,
}

impl CardHeading {
    pub const ALL: [CardHeading; 4] = [
        CardHeading::Plain,
        CardHeading::Rule,
        CardHeading::Band,
        CardHeading::Emphasis,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CardHeading::Plain => "plain",
            CardHeading::Rule => "rule",
            CardHeading::Band => "band",
            CardHeading::Emphasis => "emphasis",
        }
    }
}

pub const TITLE_MIN: u16 = 12;

const META_MIN: u16 = 4;

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

    pub meta: Option<&'a str>,
    pub badge: Option<(&'a str, crate::theme::BadgeKind)>,
    pub bg_override: Option<Color>,

    pub focus_mark: PanelFocus,

    pub title_min: u16,

    pub meta_short: Option<&'a str>,

    pub pad_x: u16,

    pub heading: Option<CardHeading>,
}

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
            heading: None,
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
            heading: None,
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

    pub fn meta_short(mut self, m: &'a str) -> Self {
        self.meta_short = Some(m);
        self
    }

    pub fn pad_x(mut self, cells: u16) -> Self {
        self.pad_x = cells;
        self
    }

    pub fn heading_style(mut self, style: CardHeading) -> Self {
        self.heading = Some(style);
        self
    }

    pub(crate) fn heading_for(&self, t: &Theme) -> CardHeading {
        match self.kind {
            PanelKind::Card => self.heading.unwrap_or(t.card_heading),
            PanelKind::Framed => CardHeading::Plain,
        }
    }

    fn head_bg(&self, t: &Theme) -> Color {
        let bg = self.bg(t);
        match self.heading_for(t) {
            CardHeading::Band if self.title.is_some() => t.ladder(t.plane_of(bg).up()).rest,
            _ => bg,
        }
    }

    fn title_span(&self, area: Rect) -> (u16, u16) {
        let pad = match self.kind {
            PanelKind::Card => self.pad_x,
            PanelKind::Framed => 2,
        };
        (area.x + pad, area.width.saturating_sub(pad * 2))
    }

    pub fn title_min(mut self, cells: u16) -> Self {
        self.title_min = cells;
        self
    }

    pub fn bg(&self, t: &Theme) -> Color {
        if let Some(bg) = self.bg_override {
            return bg;
        }
        match self.kind {
            PanelKind::Card => t.surface,
            PanelKind::Framed => t.canvas,
        }
    }

    pub fn draw_meta(&self, area: Rect, buf: &mut Buffer, t: &Theme, meta: &str) {
        let area = area.intersection(*buf.area());
        if area.is_empty() || (self.kind == PanelKind::Framed && area.width <= 4) {
            return;
        }
        let bg = self.head_bg(t);
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
            heading: self.heading,
        };
        let (x, w) = self.title_span(area);
        let row = Rect::new(x, area.y, w, 1);
        fill(buf, row, Style::new().bg(bg));
        late.title_row(row.x, row.y, row.width, buf, t, bg);
    }

    pub fn mark_clipped(&self, inner: Rect, buf: &mut Buffer, t: &Theme) {
        let inner = inner.intersection(*buf.area());
        if inner.is_empty() {
            return;
        }
        let bg = self.bg(t);
        let (x, y) = (inner.right() - 1, inner.bottom() - 1);
        buf.set_string(x, y, t.glyphs.ellipsis, t.faint().bg(bg));
    }

    pub fn try_render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> Option<Rect> {
        let inner = self.render(area, buf, t);
        (!inner.is_empty()).then_some(inner)
    }

    pub fn outer(&self, content: ratatui::layout::Size) -> ratatui::layout::Size {
        let w = |s: &str| crate::ui::text::width(s) as u16;
        let title = self.title.map(w).unwrap_or(0);
        let meta = self.meta.filter(|m| !m.is_empty()).map(w).unwrap_or(0);
        match self.kind {
            PanelKind::Card => {
                let pad = self.pad_x * 2;
                let row = title + if meta > 0 { meta + 1 } else { 0 };
                let head = if self.title.is_some() { 3 } else { 2 };
                ratatui::layout::Size::new(
                    (content.width + pad).max(row + pad),
                    content.height + head,
                )
            }
            PanelKind::Framed => {
                let row = if meta > 0 {
                    title + meta + 5
                } else {
                    title + 2
                };
                ratatui::layout::Size::new((content.width + 5).max(row + 4), content.height + 2)
            }
        }
    }

    pub fn inner(&self, outer: ratatui::layout::Size) -> ratatui::layout::Size {
        let (w, h) = match self.kind {
            PanelKind::Card => (self.pad_x * 2, if self.title.is_some() { 3 } else { 2 }),
            PanelKind::Framed => (5, 2),
        };
        ratatui::layout::Size::new(
            outer.width.saturating_sub(w),
            outer.height.saturating_sub(h),
        )
    }

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
                let heading = self.heading_for(t);
                let head_bg = self.head_bg(t);
                if heading == CardHeading::Band && self.title.is_some() {
                    fill(
                        buf,
                        Rect::new(area.x, area.y, area.width, 1),
                        Style::new().bg(head_bg),
                    );
                }
                if self.focused && self.title.is_some() && self.focus_mark == PanelFocus::Bar {
                    let s = crate::ui::ctx::VisualState {
                        focused: true,
                        ..Default::default()
                    };
                    buf.set_string(
                        area.x + self.pad_x.saturating_sub(1),
                        area.y,
                        t.gutter_symbol(s),
                        t.gutter(s, head_bg, false),
                    );
                }
                let (x, w) = self.title_span(area);
                self.title_row(x, area.y, w, buf, t, head_bg);
                if heading == CardHeading::Rule && self.title.is_some() && area.height > 2 {
                    let rule = t.glyphs.h_divider.repeat(w as usize);
                    buf.set_string(x, area.y + 1, rule, Style::new().fg(t.rule_on(bg)).bg(bg));
                }
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

    fn title_style(&self, t: &Theme, bg: Color) -> Style {
        use crate::theme::ColorLevel;
        use ratatui::style::Modifier;
        let mono = t.level == ColorLevel::Mono;
        let base = if self.focused {
            t.title().bg(bg)
        } else {
            t.secondary().bg(bg)
        };
        match self.heading_for(t) {
            CardHeading::Plain | CardHeading::Rule => base,
            CardHeading::Band => {
                let fg = t.legible(base.fg.unwrap_or(t.text_primary), bg);
                let s = base.fg(fg);
                if mono {
                    s.add_modifier(Modifier::UNDERLINED)
                } else {
                    s
                }
            }
            CardHeading::Emphasis => {
                let s = base.add_modifier(Modifier::BOLD);
                if mono && self.focused {
                    s.add_modifier(Modifier::UNDERLINED)
                } else {
                    s
                }
            }
        }
    }

    fn meta_style(&self, t: &Theme, bg: Color) -> Style {
        match self.heading_for(t) {
            CardHeading::Band => Style::new().fg(t.legible(t.text_faint, bg)).bg(bg),
            _ => t.faint().bg(bg),
        }
    }

    fn title_row(&self, x: u16, y: u16, w: u16, buf: &mut Buffer, t: &Theme, bg: Color) {
        if w == 0 {
            return;
        }
        let mut cx = x;
        let framed = self.kind == PanelKind::Framed;
        let pad: u16 = if framed { 2 } else { 0 };

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
            let style = self.title_style(t, bg);
            let room = if meta_w > 0 {
                w.saturating_sub(meta_w + 1 + pad)
            } else {
                w.saturating_sub(pad)
            };
            let full = title;
            let title = crate::ui::text::truncate(full, room as usize);

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

        if let Some(meta) = meta.filter(|m| !m.is_empty()) {
            let text = if framed { format!(" {meta} ") } else { meta };
            let tw = crate::ui::text::width(&text) as u16;
            if right >= cx + tw + u16::from(cx > x) {
                right = right.saturating_sub(tw);
                buf.set_string(right, y, &text, self.meta_style(t, bg));
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

#[derive(Debug, Clone)]
pub struct ScrollPanel {
    pub id: WidgetId,
    pub lines: Vec<String>,

    pub scroll: ScrollPane,

    pub follow: bool,

    pub tail: bool,
    pub wrap: bool,
    pub area: Rect,

    wrapped_cache: (u16, Vec<String>, Vec<usize>),

    pub horizontal: Option<HScroll>,

    pub nav: NavKeys,

    pub keymap: KeyMap<PanelCmd>,

    pub pointer_select: PointerSelect,

    select: TextSelect<CellPos>,

    select_w: u16,

    pub edit_menu: EditMenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelCmd {
    Follow,

    Copy,

    SelectAll,
}

impl Actions for PanelCmd {
    fn label(self) -> &'static str {
        match self {
            PanelCmd::Follow => "Follow",
            PanelCmd::Copy => "Copy",
            PanelCmd::SelectAll => "Select all",
        }
    }

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
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

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
            edit_menu: EditMenu::default(),
        }
    }

    pub fn context_menu(mut self, on: bool) -> Self {
        self.edit_menu.enabled = on;
        self
    }

    pub fn pointer_select(mut self, policy: PointerSelect) -> Self {
        self.pointer_select = policy;
        self
    }

    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.horizontal = Some(policy);
        self
    }

    fn h_enabled(&self) -> bool {
        !self.wrap && self.scroll.h_cfg.enabled
    }

    fn scroll_h(&mut self, delta: isize) -> Outcome {
        self.wheel(Axis::Horizontal, delta as i32).outcome()
    }

    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }

    pub fn tail(mut self, t: bool) -> Self {
        self.tail = t;
        self.follow = t;
        self
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
        self.wrapped_cache.0 = 0;
    }

    fn settle_follow(&mut self) {
        self.follow = self.tail && self.scroll.at_end();
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if let Some(o) = edit_menu_key(self, key) {
            return o;
        }

        let cmd = lookup(&self.keymap, key).or_else(|| {
            (key.ctrl_char('c') && self.selection().is_some()).then_some(PanelCmd::Copy)
        });
        match cmd {
            Some(PanelCmd::SelectAll) => return self.select_all(),
            Some(PanelCmd::Copy) => {
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
        self.render_panel(area, buf, ctx, bg, style_line);

        self.edit_menu.render(self.area, buf, ctx);
    }

    fn render_panel(
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

        let focused = ctx
            .interaction
            .focused_under(self.id, self.edit_menu.is_open());
        let text_w = area.width.saturating_sub(2);
        let lines: &Vec<String> = if self.wrap {
            &self.wrapped_cache.1
        } else {
            &self.lines
        };
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);

        let rows_h = (self.scroll.viewport_len as u16).min(area.height);
        ctx.autoscroll(Rect::new(area.x, area.y, area.width, rows_h));
        let sel = self.select.range();
        if sel.is_some() {
            ctx.selection(self.id);
        }
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

impl ScrollPanel {
    fn rows(&self) -> &[String] {
        if self.wrap {
            &self.wrapped_cache.1
        } else {
            &self.lines
        }
    }

    fn source_of(&self, row: usize) -> usize {
        if self.wrap {
            self.wrapped_cache.2.get(row).copied().unwrap_or(row)
        } else {
            row
        }
    }

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

    pub fn selection(&self) -> Option<(CellPos, CellPos)> {
        self.select.range()
    }

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

    pub fn clear_selection(&mut self) -> Outcome {
        self.select.clear()
    }

    fn on_press_with(&mut self, pos: Position, count: u8, mods: KeyModifiers) -> Outcome {
        let at = self.pos_at(pos);
        let word = at.and_then(|p| self.word_at(p));
        let line = at.and_then(|p| self.line_at(p));
        self.select
            .press(self.pointer_select, at, count, mods, word, line)
    }

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

    fn wheel_first(&mut self, axis: Axis, _delta: i32) -> Option<Response<()>> {
        (axis == Axis::Horizontal && !self.h_enabled()).then(Response::ignored)
    }

    fn after_scroll(&mut self, axis: Axis, _moved: bool) -> bool {
        if axis != Axis::Vertical {
            return false;
        }
        let was = self.follow;
        self.settle_follow();
        was != self.follow
    }
}

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
            heading: self.heading,
        };
        let inner = panel.render(area, buf, t);
        if !inner.is_empty() {
            let text = crate::ui::text::truncate("content", inner.width as usize);
            buf.set_string(inner.x, inner.y, text, t.secondary().bg(panel.bg(t)));
        }
    }
}

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

impl EditMenuHost for ScrollPanel {
    fn edit_menu(&mut self) -> &mut EditMenu {
        &mut self.edit_menu
    }
    fn edit_owner(&self) -> WidgetId {
        self.id
    }
    fn edit_caps(&self) -> EditCaps {
        EditCaps::read_only(
            self.select.range().is_some(),
            self.lines.iter().any(|l| !l.is_empty()),
        )
    }
    fn edit_shortcut(&self, cmd: crate::core::edit::TextCmd) -> Option<String> {
        use crate::core::edit::TextCmd;
        let cmd = match cmd {
            TextCmd::Copy => PanelCmd::Copy,
            TextCmd::SelectAll => PanelCmd::SelectAll,
            _ => return None,
        };
        crate::core::keymap::label_for(&self.keymap, cmd)
    }
    fn edit_target(&self, ev: &crate::core::pointer::PointerEvent) -> bool {
        ev.id == self.id
    }
    fn edit_run(&mut self, cmd: crate::core::edit::TextCmd) -> bool {
        use crate::core::edit::TextCmd;
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

        assert_eq!(p.on_key(&key(KeyCode::Right)), Outcome::Changed);
        assert_eq!(p.scroll.h.offset, HScroll::default().step);
        let (buf, _) = draw_panel(&mut p, area);
        assert!(row(&buf, 0).starts_with('…'), "{}", row(&buf, 0));
        assert!(!row(&buf, 3).starts_with(scrollbar::H_THUMB));

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

    #[test]
    fn the_title_keeps_title_min_and_the_meta_shortens_first() {
        let r = title_of("Languages", "1–13 of 24", None, 30);
        assert!(r.contains("Languages") && r.contains("1–13 of 24"), "{r}");

        let r = title_of("Languages", "1–13 of 24", None, 22);
        assert!(r.contains("Languages"), "the title stays whole: {r}");
        assert!(r.contains("1–13/24"), "{r}");

        let r = title_of(
            "customers",
            "rows 1–10 of 40 loaded · ~4,812 total",
            Some("1–10/40"),
            24,
        );
        assert!(r.contains("customers") && r.contains("1–10/40"), "{r}");

        let r = title_of("Languages", "1–13 of 24", None, 18);
        assert!(r.contains("Languages") && !r.contains('1'), "{r}");

        let r = title_of("Log", "Enter copies the row", None, 20);
        assert!(
            r.contains("Log") && r.contains("Enter cop") && r.contains('…'),
            "{r}"
        );

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

        let r = title_of("A rather long title", "70–100 of 120", None, 28);
        assert!(r.contains("A rather lo") && r.contains("70–100/120"), "{r}");

        let t = Theme::junie();
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 3));
        Panel::card(Some("A rather long title"))
            .meta("70–100 of 120")
            .title_min(4)
            .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
        assert!(row(&buf, 0).contains("70–100 of 120"), "{}", row(&buf, 0));
    }

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

        assert_eq!(log.wheel(Axis::Vertical, 3).outcome(), Outcome::Consumed);
        assert!(log.follow);

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

        assert_eq!(top(&buf), " Components     25 ");
        panel.draw_meta(area, &mut buf, &theme, "3/25");
        assert_eq!(top(&buf), " Components   3/25 ");

        let framed =
            Panel::framed(Some("F"))
                .pad_x(1)
                .render(area, &mut Buffer::empty(area), &theme);
        let plain = Panel::framed(Some("F")).render(area, &mut Buffer::empty(area), &theme);
        assert_eq!(framed, plain);
    }
}

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

        click(&mut p, &hits, (1, 2), 1, KeyModifiers::SHIFT);
        assert_eq!(
            copied(&mut p).as_deref(),
            Some("beta gamma\nsecond line here\nt")
        );

        click(&mut p, &hits, (8, 1), 1, none);
        click(&mut p, &hits, (8, 1), 2, none);
        assert_eq!(copied(&mut p).as_deref(), Some("line"));
        click(&mut p, &hits, (1, 0), 1, none);
        click(&mut p, &hits, (1, 0), 2, none);
        click(&mut p, &hits, (1, 0), 3, none);
        assert_eq!(copied(&mut p).as_deref(), Some("alpha beta gamma"));

        click(&mut p, &hits, (1, 0), 1, none);
        assert_eq!(copied(&mut p), None);
        let _ = p.on_key(&Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::CONTROL,
        });
        assert_eq!(copied(&mut p).as_deref(), Some(lines.join("\n").as_str()));

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

#[cfg(test)]
mod outer_tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::layout::Size;

    #[test]
    fn outer_gives_back_the_content_and_a_whole_title_row() {
        let t = Theme::junie();
        let content = Size::new(10, 4);
        for panel in [
            Panel::card(Some("Location")).meta("clickable(false)"),
            Panel::card(None),
            Panel::framed(Some("Explorer")).meta("4 items"),
            Panel::framed(Some("A")),
        ] {
            let size = panel.outer(content);
            let area = Rect::new(0, 0, size.width, size.height);
            let mut buf = Buffer::empty(area);
            let inner = panel.render(area, &mut buf, &t);
            assert_eq!(panel.inner(size), inner.as_size());
            assert!(inner.width >= content.width, "{inner:?}");
            assert_eq!(inner.height, content.height);
            let row: String = (0..size.width).map(|x| buf[(x, 0)].symbol()).collect();
            for part in panel.title.iter().chain(panel.meta.iter()) {
                assert!(row.contains(part), "{row:?} lacks {part}");
            }
        }
    }
}

#[cfg(test)]
mod heading_tests {
    use super::*;
    use crate::theme::{ColorLevel, contrast_ratio};
    use ratatui::style::Modifier;

    const LEVELS: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    const AREA: Rect = Rect::new(0, 0, 30, 5);

    fn card(t: &Theme, heading: Option<CardHeading>, focused: bool) -> (Buffer, Rect) {
        let mut buf = Buffer::empty(AREA);
        let mut p = Panel::card(Some("Settings"))
            .meta("3 items")
            .focused(focused);
        if let Some(h) = heading {
            p = p.heading_style(h);
        }
        let inner = p.render(AREA, &mut buf, t);
        buf.set_string(inner.x, inner.y, "body text", t.secondary().bg(p.bg(t)));
        (buf, inner)
    }

    fn line(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    #[test]
    fn rule_is_the_default_and_the_theme_and_the_panel_choose_it() {
        assert_eq!(CardHeading::default(), CardHeading::Rule);
        for level in LEVELS {
            let t = Theme::for_level(level);
            assert_eq!(t.card_heading, CardHeading::Rule, "{level:?}");
            for focused in [false, true] {
                let (rule, _) = card(&t, None, focused);
                let (own, _) = card(&t, Some(CardHeading::Rule), focused);
                assert_eq!(rule, own, "{level:?}: Rule is the default");

                let plain_theme = t.with_card_heading(CardHeading::Plain);
                let (themed, _) = card(&plain_theme, None, focused);
                assert_ne!(themed, rule, "{level:?}: the theme's style applies");
                let (plain, _) = card(&t, Some(CardHeading::Plain), focused);
                assert_eq!(themed, plain, "{level:?}: Plain stays available");

                let (back, _) = card(&plain_theme, Some(CardHeading::Rule), focused);
                assert_eq!(back, rule, "{level:?}: the panel's style wins");
            }
        }
    }

    #[test]
    fn a_framed_panel_ignores_the_heading_style() {
        for level in LEVELS {
            let t = Theme::for_level(level);
            let draw = |t: &Theme, h: Option<CardHeading>| {
                let mut buf = Buffer::empty(AREA);
                let mut p = Panel::framed(Some("Preview")).meta("1–9 of 24");
                if let Some(h) = h {
                    p = p.heading_style(h);
                }
                p.render(AREA, &mut buf, t);
                buf
            };
            let plain = draw(&t, None);
            for h in CardHeading::ALL {
                assert_eq!(draw(&t, Some(h)), plain, "{level:?} {h:?}");
                assert_eq!(
                    draw(&t.with_card_heading(h), None),
                    plain,
                    "{level:?} {h:?}"
                );
            }
        }
    }

    #[test]
    fn every_style_reads_and_sets_the_heading_apart_at_every_level() {
        for level in LEVELS {
            let t = Theme::for_level(level);
            for focused in [false, true] {
                let (plain, plain_inner) = card(&t, None, focused);
                for h in [CardHeading::Rule, CardHeading::Band, CardHeading::Emphasis] {
                    let at = format!("{level:?} {h:?} focused={focused}");
                    let (buf, inner) = card(&t, Some(h), focused);
                    assert_eq!(inner, plain_inner, "{at}: the content does not move");
                    assert_eq!(line(&buf, 2), line(&plain, 2), "{at}: body untouched");
                    let title = &buf[(2, 0)];
                    assert_eq!(title.symbol(), "S", "{at}");
                    if let Some(c) = contrast_ratio(title.fg, title.bg) {
                        assert!(c >= 4.5, "{at}: title contrast {c:.2}");
                    }
                    let meta = &buf[(27, 0)];
                    assert_eq!(meta.symbol(), "s", "{at}: {}", line(&buf, 0));
                    assert_ne!(meta.fg, meta.bg, "{at}: meta readable");
                    let body = &buf[(2, 2)];
                    let plain_title = &plain[(2, 0)];
                    match h {
                        CardHeading::Rule => {
                            let rule = t.glyphs.h_divider;
                            assert!(
                                (2..28).all(|x| buf[(x, 1)].symbol() == rule),
                                "{at}: {}",
                                line(&buf, 1)
                            );
                            assert_ne!(buf[(2, 1)].fg, buf[(2, 1)].bg, "{at}: rule shows");
                        }
                        CardHeading::Band => {
                            if level == ColorLevel::Mono {
                                assert!(
                                    title.modifier.contains(Modifier::UNDERLINED),
                                    "{at}: the band's mono cue"
                                );
                            } else {
                                assert_ne!(title.bg, body.bg, "{at}: the band's plane");
                                assert!(
                                    (0..30).all(|x| buf[(x, 0)].bg == title.bg),
                                    "{at}: the band spans the card"
                                );
                            }
                        }
                        CardHeading::Emphasis => {
                            assert!(title.modifier.contains(Modifier::BOLD), "{at}");
                            assert_ne!(
                                (title.modifier, title.fg),
                                (body.modifier, body.fg),
                                "{at}: the title differs from the body"
                            );
                            if level == ColorLevel::Mono && focused {
                                assert!(title.modifier.contains(Modifier::UNDERLINED), "{at}");
                            }
                            if !focused {
                                assert!(
                                    !plain_title.modifier.contains(Modifier::BOLD),
                                    "{at}: plain at rest is not bold"
                                );
                            }
                        }
                        CardHeading::Plain => unreachable!(),
                    }
                }
            }
        }
    }

    #[test]
    fn draw_meta_keeps_the_band() {
        let t = Theme::junie().with_card_heading(CardHeading::Band);
        let mut buf = Buffer::empty(AREA);
        let p = Panel::card(Some("Log"));
        p.render(AREA, &mut buf, &t);
        p.draw_meta(AREA, &mut buf, &t, "1–7 of 60");
        let band = buf[(0, 0)].bg;
        assert_ne!(band, buf[(0, 2)].bg);
        assert!((0..30).all(|x| buf[(x, 0)].bg == band), "{}", line(&buf, 0));
        assert!(line(&buf, 0).contains("1–7 of 60"), "{}", line(&buf, 0));
    }

    #[test]
    fn an_untitled_card_has_no_heading_to_set_apart() {
        for h in CardHeading::ALL {
            let t = Theme::junie().with_card_heading(h);
            let draw = |t: &Theme| {
                let mut buf = Buffer::empty(AREA);
                Panel::card(None).render(AREA, &mut buf, t);
                buf
            };
            assert_eq!(draw(&t), draw(&Theme::junie()), "{h:?}");
        }
    }
}
