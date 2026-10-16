use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::reveal::{Reveal, RevealLine, RevealState};
use crate::widgets::scrollbar;

#[derive(Debug, Clone)]
pub struct ListItem {
    pub label: String,
    pub meta: Option<String>,
    pub disabled: bool,
}

impl ListItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            meta: None,
            disabled: false,
        }
    }
    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

/// What a single click on a row does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClickAction {
    /// A click activates the row (chooses it, or toggles it in a multi
    /// list). The long-standing behaviour of this list.
    #[default]
    Activate,
    /// A click only moves the cursor; a double click (or Return) activates,
    /// as in macOS table and outline views.
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Single,
    Multi,
}

/// Scrollable list. The list is one focus stop; `cursor` is the keyboard row.
/// Single mode: `›` marks the chosen item. Multi mode: `✓` marks each.
/// A cut label reveals its full text on hover and with `→` on the cursor
/// row (see [`crate::ui::reveal`]).
#[derive(Debug, Clone)]
pub struct ListBox {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    pub id: WidgetId,
    pub items: Vec<ListItem>,
    pub cursor: usize,
    pub mode: SelectMode,
    pub chosen: Option<usize>,
    pub checked: Vec<bool>,
    pub scroll: ScrollState,
    pub area: Rect,
    pub empty_text: String,
    /// Range-select anchor for Shift+arrows in multi mode.
    anchor: Option<usize>,
    /// Revealing cut labels; `reveal.config` overrides the global default.
    pub reveal: RevealState,
    /// Blank columns between the labels and the scrollbar (default
    /// [`scrollbar::GAP`]).
    pub scrollbar_gap: u16,
    /// Arrow, page and Home/End movement steps over disabled rows (they
    /// cannot be used, so the cursor never parks on them). Default `true`.
    pub skip_disabled: bool,
    /// What a single click on a row does. Default [`ClickAction::Activate`].
    pub click: ClickAction,
    /// What a press on the bare vertical track does; `None` follows
    /// [`RenderCtx::track_click`](crate::ui::ctx::RenderCtx::track_click).
    pub track_click: Option<crate::core::scroll::TrackClick>,
    /// The vertical track policy resolved at the last render.
    v_click: crate::core::scroll::TrackClick,
    /// Typing a prefix jumps to the matching row (macOS lists). Off by
    /// default: plain letters keep their vim and action bindings.
    pub type_ahead: bool,
    typeahead: crate::core::typeahead::TypeAhead,
}

impl ListBox {
    pub fn new(id: WidgetId, items: Vec<ListItem>, mode: SelectMode) -> Self {
        let n = items.len();
        Self {
            track_click: None,
            v_click: crate::core::scroll::TrackClick::Jump,
            id,
            items,
            cursor: 0,
            mode,
            chosen: None,
            checked: vec![false; n],
            scroll: ScrollState::new(n),
            area: Rect::ZERO,
            track: Rect::ZERO,
            empty_text: "Nothing here yet".to_owned(),
            anchor: None,
            reveal: RevealState::default(),
            scrollbar_gap: scrollbar::GAP,
            skip_disabled: true,
            click: ClickAction::Activate,
            type_ahead: false,
            typeahead: Default::default(),
        }
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    pub fn skip_disabled(mut self, on: bool) -> Self {
        self.skip_disabled = on;
        self
    }

    pub fn click(mut self, click: ClickAction) -> Self {
        self.click = click;
        self
    }

    pub fn mode(mut self, mode: SelectMode) -> Self {
        self.mode = mode;
        self
    }

    /// The enabled row nearest `to`, searching away from the cursor first
    /// (the direction of travel), then back toward it.
    fn enabled_near(&self, to: usize) -> usize {
        let n = self.items.len();
        if !self.skip_disabled || n == 0 || !self.items[to].disabled {
            return to;
        }
        let forward = to >= self.cursor;
        let ahead: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new(to..n)
        } else {
            Box::new((0..=to).rev())
        };
        let behind: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new((0..to).rev())
        } else {
            Box::new(to + 1..n)
        };
        ahead
            .chain(behind)
            .find(|&i| !self.items[i].disabled)
            .unwrap_or(to)
    }

    /// Blank columns between the labels and the scrollbar.
    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    /// How cut labels are revealed (default: [`RenderCtx::reveal`]).
    pub fn reveal(mut self, config: Reveal) -> Self {
        self.reveal.config = Some(config);
        self
    }

    /// The footer hint for the reveal key while the focused cursor row is
    /// cut (`→ Full text`, or `← Collapse` while it shows), as of the last
    /// render. Apps append it to the list's hints so the key is advertised
    /// exactly when it does something.
    pub fn reveal_hint(&self) -> Option<(&'static str, &'static str)> {
        self.reveal.hint()
    }

    /// Label and metadata widths of a row in `text_w` cells: the metadata
    /// hides rather than starve the label.
    fn label_width(item: &ListItem, text_w: usize) -> (usize, usize) {
        let label_w = text_w.saturating_sub(2);
        let meta_w = item
            .meta
            .as_ref()
            .map(|m| crate::ui::text::width(m))
            .unwrap_or(0);
        let meta_w = if label_w.saturating_sub(meta_w + 2) < 12 {
            0
        } else {
            meta_w
        };
        let lw = label_w.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 0 });
        (lw, meta_w)
    }

    /// The rows the focused cursor row wraps to in place, when the reveal
    /// is open in [`crate::ui::reveal::RevealMode::InlineWrap`].
    fn inline_rows(
        &mut self,
        ctx: &mut RenderCtx,
        text_w: usize,
        room: u16,
    ) -> Option<Vec<RevealLine>> {
        let item = self.items.get(self.cursor).filter(|it| !it.disabled)?;
        let (lw, _) = Self::label_width(item, text_w);
        let rid = self.row_id(self.cursor);
        let label = item.label.clone();
        self.reveal.inline(ctx, rid, lw as u16, &label, true, room)
    }

    pub fn empty_text(mut self, s: &str) -> Self {
        self.empty_text = s.to_owned();
        self
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn checked_count(&self) -> usize {
        self.checked.iter().filter(|c| **c).count()
    }

    fn move_cursor(&mut self, to: usize, extend: bool) {
        let to = self.enabled_near(to.min(self.items.len().saturating_sub(1)));
        if self.mode == SelectMode::Multi && extend {
            let anchor = *self.anchor.get_or_insert(self.cursor);
            let (a, b) = (anchor.min(to), anchor.max(to));
            for i in a..=b {
                if !self.items[i].disabled {
                    self.checked[i] = true;
                }
            }
        } else {
            self.anchor = None;
        }
        self.cursor = to;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn activate(&mut self, i: usize) -> Outcome {
        if i >= self.items.len() || self.items[i].disabled {
            return Outcome::Consumed;
        }
        match self.mode {
            SelectMode::Single => self.chosen = Some(i),
            SelectMode::Multi => self.checked[i] = !self.checked[i],
        }
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.items.is_empty() {
            return Outcome::Ignored;
        }
        if let Some(o) = self.reveal.on_key(key) {
            return o;
        }
        let shift = key.shift();
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return Outcome::Ignored;
        }
        if let KeyCode::Char(c) = key.code
            && self.type_ahead
            && !c.is_whitespace()
        {
            let prefix = self.typeahead.feed(c).to_owned();
            let found = crate::core::typeahead::TypeAhead::find(
                &prefix,
                self.items.iter().map(|it| it.label.as_str()),
                self.cursor,
                |i| !self.items[i].disabled,
            );
            if let Some(i) = found {
                self.move_cursor(i, false);
                return Outcome::Changed;
            }
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                self.move_cursor(self.cursor.saturating_sub(1), shift);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                self.move_cursor(self.cursor + 1, shift);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.move_cursor(
                    self.cursor.saturating_sub(self.scroll.viewport_len.max(1)),
                    shift,
                );
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.move_cursor(self.cursor + self.scroll.viewport_len.max(1), shift);
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.move_cursor(0, shift);
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.move_cursor(usize::MAX, shift);
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.cursor),
            KeyCode::Char('a') if self.mode == SelectMode::Multi => {
                let all = self
                    .items
                    .iter()
                    .zip(&self.checked)
                    .filter(|(it, _)| !it.disabled)
                    .all(|(_, c)| *c);
                for (it, c) in self.items.iter().zip(self.checked.iter_mut()) {
                    if !it.disabled {
                        *c = !all;
                    }
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, row: usize) -> Outcome {
        self.on_click_count(row, 1)
    }

    /// A click on `row` that is the `count`th of a multi-click (from
    /// [`crate::core::pointer::PointerState::count_click`]). Under
    /// [`ClickAction::Select`] only a double click activates.
    pub fn on_click_count(&mut self, row: usize, count: u8) -> Outcome {
        if row >= self.items.len() {
            return Outcome::Consumed;
        }
        self.cursor = row;
        self.anchor = None;
        self.scroll.ensure_visible(row);
        match self.click {
            ClickAction::Activate => self.activate(row),
            ClickAction::Select if count >= 2 => self.activate(row),
            ClickAction::Select => Outcome::Changed,
        }
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        // a revealed label longer than its rows reads on before the list moves
        if let Some(o) = self.reveal.on_wheel(delta) {
            return o;
        }
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Which visible row a widget id refers to.
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }

    /// True if `id` is the list or any of its parts.
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
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

    /// The pointer went down on the scrollbar (or a completed click): a
    /// press on the thumb grabs it, a press on the track jumps to it.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        if scrollbar::press_with(self.track(), pos, &mut self.scroll, self.v_click) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        if scrollbar::drag(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.v_click = self.track_click.unwrap_or(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.reveal.begin(ctx);
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(area.height as usize);
        // the container registers first so rows drawn later win hit-testing
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        if self.items.is_empty() {
            let msg = crate::ui::text::truncate(&self.empty_text, area.width as usize);
            let y = area.y + area.height / 2;
            let x = area.x
                + (area
                    .width
                    .saturating_sub(crate::ui::text::width(&msg) as u16))
                    / 2;
            buf.set_string(x, y, &msg, t.muted().bg(bg));
            return;
        }
        let mut has_sb = self.scroll.overflows();
        // labels stop at the content edge, the gap clear of the bar
        let mut sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
        let mut text_w = sp.content.width.saturating_sub(1) as usize;
        // an inline reveal grows the cursor row: the rows it adds come out
        // of the viewport, and the cursor row stays on screen
        let mut expanded = None;
        if focused {
            expanded = self.inline_rows(ctx, text_w, area.height);
            if let Some(rows) = &expanded {
                let extra = rows.len().saturating_sub(1);
                self.scroll
                    .set_viewport((area.height as usize).saturating_sub(extra).max(1));
                self.scroll.ensure_visible(self.cursor);
                if self.scroll.overflows() != has_sb {
                    has_sb = self.scroll.overflows();
                    sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
                    text_w = sp.content.width.saturating_sub(1) as usize;
                    expanded = self.inline_rows(ctx, text_w, area.height);
                }
            }
        }
        let mut y = area.y;
        for li in self.scroll.visible_range() {
            if y >= area.bottom() {
                break;
            }
            let item = &self.items[li];
            let block = expanded.as_ref().filter(|_| li == self.cursor);
            let h = block
                .map_or(1, |r| r.len() as u16)
                .clamp(1, area.bottom() - y);
            let row = Rect::new(area.x, y, sp.body.width, h);
            let rid = self.row_id(li);
            let mut s = ctx.child_state(self.id, rid, li == self.cursor);
            s.focused = focused && li == self.cursor;
            s.disabled = item.disabled;
            s.selected = match self.mode {
                SelectMode::Single => self.chosen == Some(li),
                SelectMode::Multi => self.checked[li],
            };
            let st = t.row(s, bg);
            fill(buf, row, st);
            for gy in row.y..row.bottom() {
                buf.set_string(
                    row.x,
                    gy,
                    t.gutter_symbol(s),
                    t.gutter(s, st.bg.unwrap_or(bg), false),
                );
            }
            let marker = match (self.mode, s.selected) {
                (SelectMode::Single, true) => "›",
                (SelectMode::Multi, true) => "✓",
                _ => " ",
            };
            let ms = if s.selected && !item.disabled {
                st.fg(if focused || s.hovered {
                    t.accent
                } else {
                    t.text_secondary
                })
            } else {
                st
            };
            buf.set_string(row.x + 1, y, marker, ms);
            let label_w = text_w.saturating_sub(2);
            let (lw, meta_w) = Self::label_width(item, text_w);
            if let Some(rows) = block {
                // the cursor row wrapped in place: every line, then a
                // marker when a long text scrolls
                for (k, line) in rows.iter().take(h as usize).enumerate() {
                    let ls = if line.more { st.fg(t.text_muted) } else { st };
                    let text = crate::ui::text::fit(&line.text, lw);
                    buf.set_string(row.x + 3, y + k as u16, text, ls);
                }
            } else {
                buf.set_string(row.x + 3, y, crate::ui::text::fit(&item.label, lw), st);
                if !item.disabled {
                    let label = Rect::new(row.x + 3, y, lw as u16, 1);
                    self.reveal.cell(ctx, rid, label, &item.label, s.focused);
                }
            }
            if let Some(m) = &item.meta
                && meta_w > 0
                && meta_w + 4 < label_w
            {
                let mx = sp
                    .content
                    .right()
                    .min(row.right().saturating_sub(1))
                    .saturating_sub(meta_w as u16);
                let ms = if item.disabled {
                    st
                } else {
                    st.fg(t.text_muted)
                };
                buf.set_string(mx, y, m, ms);
            }
            ctx.clickable(rid, row);
            y += h;
        }
        // the scroll region also sits above the rows: a wheel burst handled
        // before the next frame (see `runtime::Coalesce`) then routes to the
        // widget, not to a row whose display index it already scrolled away
        ctx.scrollable(self.id, area);
        self.track = Rect::ZERO;
        if let Some(sb) = sp.track {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                sp.body,
                &self.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}

#[cfg(test)]
mod reveal_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use crate::ui::reveal::RevealMode;
    use ratatui::crossterm::event::KeyModifiers;

    const LONG: &str = "a label far too long for the narrow list it lives in";

    fn render(l: &mut ListBox, interaction: Interaction) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 4));
        l.render(Rect::new(0, 0, 20, 4), &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
        (0..4)
            .map(|y| {
                (0..70)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn list() -> ListBox {
        ListBox::new(
            WidgetId::of("l"),
            vec![ListItem::new("short"), ListItem::new(LONG)],
            SelectMode::Single,
        )
    }

    /// Cut labels stop one blank column short of the scrollbar, never
    /// touching it (`TypeScript┃`, layout audit #14).
    #[test]
    fn a_cut_label_keeps_the_gap_before_the_scrollbar() {
        let items: Vec<ListItem> = (0..8)
            .map(|i| ListItem::new(&format!("{i} {LONG}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items.clone(), SelectMode::Single);
        let text = render(&mut l, Interaction::default());
        let row: Vec<char> = text.lines().next().unwrap().chars().take(20).collect();
        assert!(
            matches!(row[19], '│' | '┃'),
            "bar in the last column: {row:?}"
        );
        assert_eq!(row[18], ' ', "one blank column before the bar: {row:?}");
        assert_eq!(row[17], '…', "the label ends with its ellipsis: {row:?}");
        // the gap is the caller's to change
        let mut tight = ListBox::new(WidgetId::of("l"), items, SelectMode::Single).scrollbar_gap(0);
        let text = render(&mut tight, Interaction::default());
        let row: Vec<char> = text.lines().next().unwrap().chars().take(20).collect();
        assert_eq!(row[18], '…', "{row:?}");
    }

    #[test]
    fn hovering_a_cut_label_shows_it_in_full() {
        let mut l = list();
        assert!(!render(&mut l, Interaction::default()).contains(LONG));
        let hover = Interaction {
            hover: Some(l.row_id(1)),
            ..Default::default()
        };
        assert!(render(&mut l, hover).contains(LONG));
        let hover_short = Interaction {
            hover: Some(l.row_id(0)),
            ..Default::default()
        };
        assert!(!render(&mut l, hover_short).contains(LONG));
    }

    #[test]
    fn right_reveals_the_cut_cursor_row_and_left_hides_it() {
        let mut l = list();
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        let key = |code| Key {
            code,
            mods: KeyModifiers::NONE,
        };
        render(&mut l, focus);
        assert_eq!(
            l.on_key(&key(KeyCode::Right)),
            Outcome::Ignored,
            "short row"
        );
        l.on_key(&key(KeyCode::Down));
        render(&mut l, focus);
        assert_eq!(l.on_key(&key(KeyCode::Right)), Outcome::Changed);
        assert!(render(&mut l, focus).contains(LONG));
        l.on_key(&key(KeyCode::Left));
        assert!(!render(&mut l, focus).contains(LONG));
        // the policy turns it off
        let mut l = list().reveal(Reveal::OFF);
        l.on_key(&key(KeyCode::Down));
        render(&mut l, focus);
        assert_eq!(l.on_key(&key(KeyCode::Right)), Outcome::Ignored);
    }

    fn press(l: &mut ListBox, code: KeyCode) -> Outcome {
        l.on_key(&Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    /// The list alone in a `w`×`h` buffer (no room for a tooltip beside it).
    fn render_in(l: &mut ListBox, w: u16, h: u16, interaction: Interaction) -> Vec<String> {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
        l.render(Rect::new(0, 0, w, h), &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
        (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn inline_wrap_grows_the_cursor_row_in_place_and_pushes_the_rest_down() {
        let mut l = ListBox::new(
            WidgetId::of("l"),
            vec![
                ListItem::new(LONG),
                ListItem::new("second"),
                ListItem::new("third"),
            ],
            SelectMode::Single,
        )
        .reveal(Reveal::default().mode(RevealMode::InlineWrap));
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        let rows = render_in(&mut l, 24, 8, focus);
        assert!(rows[1].contains("second"), "collapsed: one row each");
        assert_eq!(l.reveal_hint(), Some(("→", "Full text")));
        assert_eq!(press(&mut l, KeyCode::Right), Outcome::Changed);
        let rows = render_in(&mut l, 24, 8, focus);
        let n = rows.iter().position(|r| r.contains("second")).unwrap();
        assert!(n > 1, "the rows below moved down: {rows:#?}");
        let label: Vec<String> = rows[..n]
            .iter()
            .map(|r| r.chars().skip(3).collect::<String>().trim().to_owned())
            .collect();
        assert_eq!(label.join(" "), LONG, "every line of the label, in place");
        // the cursor block keeps the one focus gutter on every row
        for r in &rows[..n] {
            assert!(r.starts_with('▎'), "{rows:#?}");
        }
        assert_eq!(l.reveal_hint(), Some(("←", "Collapse")));
        press(&mut l, KeyCode::Left);
        let rows = render_in(&mut l, 24, 8, focus);
        assert!(rows[1].contains("second"));
    }

    #[test]
    fn a_long_inline_reveal_scrolls_with_the_wheel_before_the_list_does() {
        let long = (1..=30)
            .map(|i| format!("w{i:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut items = vec![ListItem::new(&long)];
        items.extend((0..20).map(|i| ListItem::new(&format!("row {i}"))));
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single)
            .reveal(Reveal::default().mode(RevealMode::InlineWrap).max_rows(3));
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        render_in(&mut l, 16, 6, focus);
        press(&mut l, KeyCode::Right);
        let rows = render_in(&mut l, 16, 6, focus);
        assert!(rows[0].contains("w01"), "{rows:#?}");
        assert!(rows[2].contains("↓"), "capped at 3 rows: {rows:#?}");
        let offset = l.scroll.offset;
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        let rows = render_in(&mut l, 16, 6, focus);
        assert!(rows[0].contains("↑"), "{rows:#?}");
        assert_eq!(l.scroll.offset, offset, "the list itself did not move");
        // collapsed, the wheel scrolls the list again
        press(&mut l, KeyCode::Left);
        render_in(&mut l, 16, 6, focus);
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        assert_eq!(l.scroll.offset, offset + 1);
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;

    fn disabled_list() -> ListBox {
        ListBox::new(
            WidgetId::of("dl"),
            vec![
                ListItem::new("a").disabled(true),
                ListItem::new("b"),
                ListItem::new("c").disabled(true),
                ListItem::new("d"),
                ListItem::new("e").disabled(true),
            ],
            SelectMode::Single,
        )
    }

    fn press(l: &mut ListBox, code: KeyCode) {
        l.on_key(&Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
    }

    #[test]
    fn typing_a_prefix_jumps_to_the_row_when_type_ahead_is_on() {
        let mut l = disabled_list().type_ahead(true);
        l.cursor = 1;
        press(&mut l, KeyCode::Char('d'));
        assert_eq!(l.cursor, 3);
        press(&mut l, KeyCode::Char('x'));
        assert_eq!(l.cursor, 3, "no match leaves the cursor");
        let mut vim = disabled_list();
        vim.cursor = 1;
        press(&mut vim, KeyCode::Char('j'));
        assert_eq!(vim.cursor, 3, "off by default: j still moves down");
    }

    #[test]
    fn the_cursor_steps_over_disabled_rows_unless_configured_not_to() {
        let mut l = disabled_list();
        l.cursor = 1;
        press(&mut l, KeyCode::Down);
        assert_eq!(l.cursor, 3, "skips c");
        press(&mut l, KeyCode::End);
        assert_eq!(l.cursor, 3, "e is disabled: End stops on d");
        press(&mut l, KeyCode::Home);
        assert_eq!(l.cursor, 1, "a is disabled: Home stops on b");
        let mut plain = disabled_list().skip_disabled(false);
        plain.cursor = 1;
        press(&mut plain, KeyCode::Down);
        assert_eq!(plain.cursor, 2);
    }

    #[test]
    fn select_clicks_move_the_cursor_and_double_clicks_activate() {
        let mut l = disabled_list().click(ClickAction::Select);
        l.on_click_count(3, 1);
        assert_eq!((l.cursor, l.chosen), (3, None));
        l.on_click_count(3, 2);
        assert_eq!(l.chosen, Some(3));
        let mut act = disabled_list();
        act.on_click(1);
        assert_eq!(act.chosen, Some(1));
    }
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn render(l: &mut ListBox) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
        l.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
        (0..6)
            .map(|y| {
                (0..40)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn scroll_edges_fade_toward_hidden_content_and_never_the_cursor_row() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let theme = Theme::junie();
        let render = |l: &mut ListBox| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
            l.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
            buf
        };
        let grey = |buf: &Buffer, y: u16| match buf[(4, y)].fg {
            Color::Rgb(r, _, _) => r,
            other => panic!("{other:?}"),
        };
        // at the top: the last row hints at more below, the first row is whole
        let b = render(&mut l);
        assert_eq!(b[(4, 0)].fg, theme.text_primary, "cursor row untouched");
        assert!(
            grey(&b, 5) < grey(&b, 4),
            "bottom row fades: {}",
            grey(&b, 5)
        );
        // scrolled: the top row fades too, the cursor row never does
        l.on_wheel(10);
        let b = render(&mut l);
        assert!(grey(&b, 0) < grey(&b, 1), "top row fades");
        assert!(grey(&b, 5) < grey(&b, 4));
        // at the end: only the top row hints
        l.scroll.jump_end();
        let b = render(&mut l);
        assert!(grey(&b, 0) < grey(&b, 1));
        assert_eq!(grey(&b, 5), grey(&b, 4), "nothing below, no fade");
    }

    #[test]
    fn boundary_wheels_are_consumed_and_the_thumb_drags_with_its_grab() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        assert_eq!(l.on_wheel(-1), Outcome::Consumed, "nothing above");
        render(&mut l);
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        assert_eq!(l.on_wheel(-1), Outcome::Changed);
        assert_eq!(l.on_wheel(-1), Outcome::Consumed);
        // the track is the drawn scrollbar: six rows, thumb one row at the top
        let x = 39;
        assert_eq!(
            l.on_scrollbar(Position::new(x, 0)),
            Outcome::Consumed,
            "a press on the thumb"
        );
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 1)), Outcome::Changed);
        let after_one = l.scroll.offset;
        assert!(after_one > 0 && after_one < 10, "{after_one}");
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 5)), Outcome::Changed);
        assert_eq!(
            l.scroll.offset,
            l.scroll.max_offset(),
            "the end of the track is the end"
        );
        assert_eq!(
            l.on_scrollbar_drag(Position::new(x, 40)),
            Outcome::Consumed,
            "past the track clamps"
        );
        assert_eq!(l.on_scrollbar_drag(Position::new(x, 0)), Outcome::Changed);
        assert_eq!(l.scroll.offset, 0);
    }

    #[test]
    fn wheel_moves_the_viewport_and_keeps_the_cursor() {
        let items = (0..40)
            .map(|i| ListItem::new(&format!("Row {i:02}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let a = render(&mut l);
        assert!(a.contains("Row 00"));
        l.on_wheel(4);
        let b = render(&mut l);
        assert!(!b.contains("Row 00"));
        assert!(b.contains("Row 04"));
        assert_eq!(l.scroll.offset, 4);
        assert_eq!(render(&mut l), b, "render does not reset a wheel scroll");
        assert_eq!(l.cursor, 0);
        let key = Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        l.on_key(&key);
        render(&mut l);
        assert!(l.scroll.visible_range().contains(&l.cursor));
        l.on_wheel(-40);
        assert_eq!(l.scroll.offset, 0);
    }

    #[test]
    fn enter_on_a_focused_list_flashes_its_cursor_row() {
        let items = (0..3).map(|i| ListItem::new(&format!("Row {i}"))).collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        // applications flash the focused id, which is the list itself
        let interaction = Interaction {
            focus: Some(l.id),
            flash: Some(l.id),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 3));
        l.render(Rect::new(0, 0, 40, 3), &mut buf, &mut ctx, theme.canvas);
        let pressed = theme.ladder(crate::theme::Plane::Canvas).pressed;
        assert_eq!(buf[(4, 0)].bg, pressed, "cursor row shows the press");
        assert_eq!(buf[(4, 1)].bg, theme.canvas, "other rows stay at rest");
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::scroll::TrackClick;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_list_track_click_can_page() {
        let items: Vec<ListItem> = (0..50)
            .map(|i| ListItem::new(&format!("item {i}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single)
            .track_click(TrackClick::Page);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        l.render(area, &mut buf, &mut ctx, theme.surface);
        assert_eq!(l.on_scrollbar(Position::new(19, 9)), Outcome::Changed);
        assert_eq!(l.scroll.offset, 10, "one page, not a jump to the end");
    }
}
