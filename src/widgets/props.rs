//! Label / value facts, e.g. a connection's details or a plan node's
//! metadata. Labels are muted and right-padded to a shared width.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::theme::{Theme, Tone};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::reveal::{Reveal, RevealLine, RevealState};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,
    /// This row may be copied with `y`; everything else is read-only.
    pub copyable: bool,
}

impl Prop {
    pub fn new(label: &str, value: impl Into<String>) -> Self {
        Self {
            label: label.to_owned(),
            value: value.into(),
            tone: Tone::Normal,
            wrap: false,
            copyable: false,
        }
    }
    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

/// How a property sheet shares its width between labels and values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sheet {
    /// When the full label column would leave the values fewer than
    /// `value_min` cells, the column shrinks to this share of the width, in
    /// percent: a label longer than that sits on its own row with its value
    /// below it, instead of pushing every value off the edge. Default `40`.
    pub label_max_share: u16,
    /// The fewest cells the values keep beside the labels. Below it the
    /// label column is capped ([`Sheet::label_max_share`]), and if that is
    /// still too little the whole sheet stacks: each label on its own row,
    /// its value indented below. Default `12`.
    pub value_min: u16,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            label_max_share: 40,
            value_min: 12,
        }
    }
}

/// Indent of a value under its own label in a stacked row.
const STACK_INDENT: u16 = 2;

/// One visual row of a property sheet: the property it belongs to, its
/// label (first row only), the value text for that row and the column the
/// value starts at, relative to the sheet.
struct Row<'a> {
    prop: &'a Prop,
    label: Option<&'a str>,
    value: String,
    value_x: u16,
}

/// The label column at `width` for labels of these widths (gap included):
/// as wide as the widest label while the values keep `value_min` cells,
/// otherwise the widest label within the sheet's share, or `None` when even
/// that leaves too little and the sheet stacks.
fn column(widths: impl Iterator<Item = u16> + Clone, width: u16, sheet: Sheet) -> Option<u16> {
    let full = widths.clone().max().unwrap_or(2);
    if width.saturating_sub(full) >= sheet.value_min {
        return Some(full);
    }
    let cap = (width as u32 * sheet.label_max_share.min(100) as u32 / 100) as u16;
    // the widest label that fits the cap sets the column; longer ones stack
    let label_w = widths.filter(|w| *w <= cap).max().unwrap_or(cap.max(1));
    (width.saturating_sub(label_w) >= sheet.value_min).then_some(label_w)
}

fn label_column(props: &[Prop], width: u16, sheet: Sheet) -> Option<u16> {
    column(
        props
            .iter()
            .map(|p| crate::ui::text::width(&p.label) as u16 + 2),
        width,
        sheet,
    )
}

fn value_rows<'a>(p: &'a Prop, label: Option<&'a str>, x: u16, vw: usize, out: &mut Vec<Row<'a>>) {
    if p.wrap {
        let lines = crate::ui::text::wrap(&p.value, vw.max(4));
        for (i, line) in lines.into_iter().enumerate() {
            out.push(Row {
                prop: p,
                label: if i == 0 { label } else { None },
                value: line,
                value_x: x,
            });
        }
    } else {
        out.push(Row {
            prop: p,
            label,
            value: crate::ui::text::truncate(&p.value, vw),
            value_x: x,
        });
    }
}

/// The rows a sheet occupies at `width`, computed once for measuring and
/// rendering so a dialog sized by `measure` never overflows its frame.
fn rows(props: &[Prop], width: u16, sheet: Sheet) -> Vec<Row<'_>> {
    let column = label_column(props, width, sheet);
    let mut out = vec![];
    for p in props {
        match column {
            // the label fits its column: label and value share a row
            Some(label_w) if crate::ui::text::width(&p.label) as u16 + 2 <= label_w => {
                let vw = width.saturating_sub(label_w) as usize;
                value_rows(p, Some(&p.label), label_w, vw, &mut out);
            }
            // a label too long for the column, or a stacked sheet: the
            // label takes its own row and the value goes under it
            _ => {
                out.push(Row {
                    prop: p,
                    label: Some(&p.label),
                    value: String::new(),
                    value_x: 0,
                });
                let x = column.unwrap_or(STACK_INDENT.min(width.saturating_sub(1)));
                let vw = width.saturating_sub(x) as usize;
                value_rows(p, None, x, vw, &mut out);
            }
        }
    }
    out
}

/// Rows the sheet needs at `width`, including wrapped values.
pub fn measure(props: &[Prop], width: u16) -> u16 {
    measure_with(props, width, Sheet::default())
}

/// [`measure`] with explicit column rules.
pub fn measure_with(props: &[Prop], width: u16, sheet: Sheet) -> u16 {
    rows(props, width, sheet).len() as u16
}

/// Draw the sheet; returns the number of rows used.
pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, props: &[Prop], bg: Color) -> u16 {
    render_with(area, buf, t, props, bg, Sheet::default())
}

/// [`render`] with explicit column rules.
pub fn render_with(
    area: Rect,
    buf: &mut Buffer,
    t: &Theme,
    props: &[Prop],
    bg: Color,
    sheet: Sheet,
) -> u16 {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return 0;
    }
    let mut y = area.y;
    for row in rows(props, area.width, sheet) {
        if y >= area.bottom() {
            break;
        }
        if let Some(label) = row.label {
            let shown = crate::ui::text::truncate(label, area.width as usize);
            buf.set_string(area.x, y, &shown, t.muted().bg(bg));
            let cut = Rect::new(area.x, y, crate::ui::text::width(&shown) as u16, 1);
            crate::ui::reveal::queue_cut(cut, label);
        }
        let style = ratatui::style::Style::new()
            .fg(t.tone(row.prop.tone))
            .bg(bg);
        if !row.value.is_empty() {
            buf.set_string(area.x + row.value_x, y, &row.value, style);
            if !row.prop.wrap {
                // a value cut to the sheet reveals its full text on hover
                let cut = Rect::new(
                    area.x + row.value_x,
                    y,
                    crate::ui::text::width(&row.value) as u16,
                    1,
                );
                crate::ui::reveal::queue_cut(cut, &row.prop.value);
            }
        }
        y += 1;
    }
    y - area.y
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropsEvent {
    Copy(usize),
    Activate(usize),
}

/// The columns of a [`PropsList`] row at one scrollbar split.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    row_w: u16,
    label_w: u16,
    value_right: u16,
}

/// Interactive facts: one focus stop with a row cursor, `y` copies the
/// copyable row under the cursor, Enter activates it.
///
/// A value cut to fit keeps its `…` and reveals the full text on demand
/// (see [`crate::ui::reveal`]): hovering it shows an expansion tooltip, and
/// the reveal key (`→` by default) expands the cursor row until `←` or Esc.
/// With [`RevealMode::InlineWrap`](crate::ui::reveal::RevealMode) the
/// cursor row's value wraps in place instead, and the rows below move down.
/// Copy always copies the full value.
#[derive(Debug, Clone)]
pub struct PropsList {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    pub id: WidgetId,
    pub props: Vec<Prop>,
    pub cursor: usize,
    pub scroll: ScrollState,
    pub area: Rect,
    /// Revealing cut values; `reveal.config` overrides the global default.
    pub reveal: RevealState,
    /// Blank columns between the values and the scrollbar (default
    /// [`scrollbar::GAP`]).
    pub scrollbar_gap: u16,
    /// How the label column shares a row with the values: when the full
    /// column would starve the values it is capped, and labels longer than
    /// the cap are cut with `…` (rows never stack in a list). Default
    /// [`Sheet::default`].
    pub sheet: Sheet,
    /// What a press on the bare vertical track does; `None` follows
    /// [`RenderCtx::track_click`].
    pub track_click: Option<crate::core::scroll::TrackClick>,
    /// The vertical track policy resolved at the last render.
    v_click: crate::core::scroll::TrackClick,
}

impl PropsList {
    pub fn new(id: WidgetId, props: Vec<Prop>) -> Self {
        let n = props.len();
        Self {
            id,
            props,
            cursor: 0,
            scroll: ScrollState::new(n),
            area: Rect::ZERO,
            track: Rect::ZERO,
            reveal: RevealState::default(),
            scrollbar_gap: scrollbar::GAP,
            sheet: Sheet::default(),
            track_click: None,
            v_click: crate::core::scroll::TrackClick::Jump,
        }
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    /// Blank columns between the values and the scrollbar.
    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    /// How the label column shares a row with the values.
    pub fn sheet(mut self, sheet: Sheet) -> Self {
        self.sheet = sheet;
        self
    }

    /// How cut values are revealed (default: [`RenderCtx::reveal`]).
    pub fn reveal(mut self, config: Reveal) -> Self {
        self.reveal.config = Some(config);
        self
    }

    pub fn set_props(&mut self, props: Vec<Prop>) {
        self.props = props;
        self.cursor = self.cursor.min(self.props.len().saturating_sub(1));
        self.scroll.set_content(self.props.len());
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| self.row_id(i) == id)
    }
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.props.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PropsEvent>) {
        if self.props.is_empty() {
            return (Outcome::Ignored, None);
        }
        if let Some(o) = self.reveal.on_key(key) {
            return (o, None);
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                self.set_cursor(self.cursor.saturating_sub(1))
            }
            KeyCode::Down | KeyCode::Char('j') if key.plain() => self.set_cursor(self.cursor + 1),
            KeyCode::PageUp => {
                self.set_cursor(self.cursor.saturating_sub(self.scroll.viewport_len.max(1)))
            }
            KeyCode::PageDown => self.set_cursor(self.cursor + self.scroll.viewport_len.max(1)),
            KeyCode::Home | KeyCode::Char('g') if key.plain() => self.set_cursor(0),
            KeyCode::End | KeyCode::Char('G') => self.set_cursor(usize::MAX),
            KeyCode::Enter => return (Outcome::Changed, Some(PropsEvent::Activate(self.cursor))),
            KeyCode::Char('y') if key.plain() => {
                if self.props[self.cursor].copyable {
                    return (Outcome::Changed, Some(PropsEvent::Copy(self.cursor)));
                }
                return (Outcome::Consumed, None);
            }
            _ => return (Outcome::Ignored, None),
        }
        (Outcome::Changed, None)
    }

    pub fn on_click(&mut self, row: usize) -> (Outcome, Option<PropsEvent>) {
        if row >= self.props.len() {
            return (Outcome::Consumed, None);
        }
        self.set_cursor(row);
        (Outcome::Changed, Some(PropsEvent::Activate(row)))
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        // a revealed value longer than its rows reads on before the sheet moves
        if let Some(o) = self.reveal.on_wheel(delta) {
            return o;
        }
        if self.scroll.scroll_by(delta as isize) {
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

    /// The pointer went down on the scrollbar (or a completed click): a
    /// press on the thumb grabs it, a press on the track follows
    /// [`Self::track_click`].
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

    /// Cells kept for the `y copy` hint on the focused copyable row.
    fn hint_width(p: &Prop, focused: bool) -> u16 {
        if p.copyable && focused { 8 } else { 0 }
    }

    /// Where the label column and the values sit, for a row body and
    /// content area from [`scrollbar::split`].
    fn geometry(&self, body: Rect, content: Rect) -> Geometry {
        let row_w = body.width;
        // values end one cell short of the row, or the gap short of the bar
        let value_right = content.right().min(body.right().saturating_sub(1));
        // the labels start after the 2-cell gutter; a list never stacks, so
        // a starved sheet keeps its capped column and cuts the longer labels
        let label_area = row_w.saturating_sub(2);
        let widths = self
            .props
            .iter()
            .map(|p| crate::ui::text::width(&p.label) as u16 + 2);
        let label_w = column(widths, label_area, self.sheet).unwrap_or_else(|| {
            (label_area as u32 * self.sheet.label_max_share.min(100) as u32 / 100) as u16
        });
        Geometry {
            row_w,
            label_w: label_w.max(3),
            value_right,
        }
    }

    /// The rows the focused cursor row's value wraps to in place, when the
    /// reveal is open in [`crate::ui::reveal::RevealMode::InlineWrap`].
    fn inline_rows(
        &mut self,
        ctx: &mut RenderCtx,
        geo: Geometry,
        room: u16,
    ) -> Option<Vec<RevealLine>> {
        let p = self.props.get(self.cursor)?;
        let vx = self.area.x + 2 + geo.label_w;
        let vw = geo
            .value_right
            .saturating_sub(vx + Self::hint_width(p, true));
        let rid = self.row_id(self.cursor);
        let value = p.value.clone();
        self.reveal.inline(ctx, rid, vw, &value, true, room)
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
        self.scroll.set_content(self.props.len());
        self.scroll.set_viewport(area.height as usize);
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        let mut has_sb = self.scroll.overflows();
        let mut sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
        let mut geo = self.geometry(sp.body, sp.content);
        // an inline reveal grows the cursor row: the rows it adds come out
        // of the viewport, and the cursor row stays on screen
        let mut expanded = None;
        if focused {
            expanded = self.inline_rows(ctx, geo, area.height);
            if let Some(rows) = &expanded {
                let extra = rows.len().saturating_sub(1);
                self.scroll
                    .set_viewport((area.height as usize).saturating_sub(extra).max(1));
                self.scroll.ensure_visible(self.cursor);
                if self.scroll.overflows() != has_sb {
                    has_sb = self.scroll.overflows();
                    sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
                    geo = self.geometry(sp.body, sp.content);
                    expanded = self.inline_rows(ctx, geo, area.height);
                }
            }
        }
        let Geometry {
            row_w,
            label_w,
            value_right,
        } = geo;
        let mut y = area.y;
        let mut cursor_rows = Vec::new();
        for i in self.scroll.visible_range() {
            if y >= area.bottom() {
                break;
            }
            let p = &self.props[i];
            let rid = self.row_id(i);
            let mut s = ctx.child_state(self.id, rid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            let block = expanded.as_ref().filter(|_| i == self.cursor);
            let h = block
                .map_or(1, |r| r.len() as u16)
                .clamp(1, area.bottom() - y);
            let row = Rect::new(area.x, y, row_w, h);
            if i == self.cursor {
                cursor_rows.extend(row.y..row.bottom());
            }
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
            buf.set_string(
                row.x + 2,
                y,
                crate::ui::text::truncate(&p.label, label_w.saturating_sub(2) as usize),
                st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
            );
            let vx = row.x + 2 + label_w;
            let hint_w = Self::hint_width(p, s.focused);
            let vw = value_right.saturating_sub(vx + hint_w) as usize;
            let vs = st.fg(t.tone(p.tone));
            if let Some(lines) = block {
                // the cursor row wrapped in place: every line of the value,
                // then a marker when a long value scrolls
                for (k, line) in lines.iter().take(h as usize).enumerate() {
                    let ls = if line.more { st.fg(t.text_muted) } else { vs };
                    let text = crate::ui::text::truncate(&line.text, vw);
                    buf.set_string(vx, y + k as u16, text, ls);
                }
            } else {
                buf.set_string(vx, y, crate::ui::text::truncate(&p.value, vw), vs);
                self.reveal.cell(
                    ctx,
                    rid,
                    Rect::new(vx, y, vw as u16, 1),
                    &p.value,
                    s.focused,
                );
            }
            if hint_w > 0 {
                buf.set_string(
                    row.right().saturating_sub(7),
                    y,
                    "y copy",
                    st.fg(t.text_faint).remove_modifier(Modifier::BOLD),
                );
            }
            ctx.clickable(rid, row);
            y += h;
        }
        self.track = Rect::ZERO;
        if let Some(sb) = sp.track {
            crate::ui::fade::scroll_edges_except(buf, ctx, sp.body, &self.scroll, &cursor_rows);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}

#[cfg(test)]
mod reveal_scroll_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    #[test]
    fn the_wheel_reads_a_long_revealed_value_before_the_sheet_moves() {
        let long = (1..=60)
            .map(|i| format!("v{i:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut props = vec![Prop::new("Path", long)];
        props.extend((0..12).map(|i| Prop::new("Key", format!("value {i}"))));
        let mut l = PropsList::new(WidgetId::of("p"), props);
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        let frame = |l: &mut PropsList| {
            let theme = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, focus, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 24, 5));
            l.render(Rect::new(0, 0, 24, 5), &mut buf, &mut ctx, theme.canvas);
            ctx.flush_overlays(&mut buf);
        };
        frame(&mut l);
        let right = Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        };
        assert_eq!(l.on_key(&right).0, Outcome::Changed);
        frame(&mut l);
        let offset = l.scroll.offset;
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        assert_eq!(
            l.scroll.offset, offset,
            "the reveal scrolled, not the sheet"
        );
    }

    fn rows_of(l: &mut PropsList, w: u16, h: u16, focus: Interaction) -> Vec<String> {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, focus, &mut hits, &mut ring);
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        l.render(area, &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
        (0..h)
            .map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect()
    }

    /// `RevealMode::InlineWrap` wraps the cursor row's value in place (it
    /// used to fall back to the tooltip): the rows below move down, the
    /// focus gutter covers the whole block, and `←` puts it back.
    #[test]
    fn inline_wrap_grows_the_cursor_row_and_pushes_the_rest_down() {
        let long = "alpha bravo charlie delta echo foxtrot golf hotel";
        let mut l = PropsList::new(
            WidgetId::of("p"),
            vec![
                Prop::new("Path", long),
                Prop::new("Next", "second"),
                Prop::new("Last", "third"),
            ],
        )
        .reveal(Reveal::default().mode(crate::ui::reveal::RevealMode::InlineWrap));
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        let rows = rows_of(&mut l, 24, 8, focus);
        assert!(rows[1].contains("second"), "collapsed: {rows:#?}");
        let right = Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        };
        assert_eq!(l.on_key(&right).0, Outcome::Changed);
        let rows = rows_of(&mut l, 24, 8, focus);
        let n = rows.iter().position(|r| r.contains("second")).unwrap();
        assert!(n > 1, "the rows below moved down: {rows:#?}");
        // the value column starts after the label column ("Path" + gap)
        let vx = rows[0]
            .find("alpha")
            .map(|b| rows[0][..b].chars().count())
            .unwrap();
        let value: Vec<String> = rows[..n]
            .iter()
            .map(|r| r.chars().skip(vx).collect::<String>().trim().to_owned())
            .collect();
        assert_eq!(value.join(" "), long, "every line of the value, in place");
        assert!(!rows.iter().any(|r| r.contains('…')), "{rows:#?}");
        for r in &rows[..n] {
            assert!(r.starts_with('▎'), "one gutter over the block: {rows:#?}");
        }
        let left = Key {
            code: KeyCode::Left,
            mods: KeyModifiers::NONE,
        };
        assert_eq!(l.on_key(&left).0, Outcome::Changed);
        let rows = rows_of(&mut l, 24, 8, focus);
        assert!(rows[1].contains("second"));
    }

    /// A value longer than the sheet wraps up to its rows and scrolls with
    /// the wheel before the sheet itself does.
    #[test]
    fn a_long_inline_value_scrolls_before_the_sheet() {
        let long = (1..=40)
            .map(|i| format!("w{i:02}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut props = vec![Prop::new("K", &long)];
        props.extend((0..20).map(|i| Prop::new("K", format!("row {i}"))));
        let mut l = PropsList::new(WidgetId::of("p"), props).reveal(
            Reveal::default()
                .mode(crate::ui::reveal::RevealMode::InlineWrap)
                .max_rows(3),
        );
        let focus = Interaction {
            focus: Some(l.id),
            ..Default::default()
        };
        rows_of(&mut l, 20, 6, focus);
        let right = Key {
            code: KeyCode::Right,
            mods: KeyModifiers::NONE,
        };
        l.on_key(&right);
        let rows = rows_of(&mut l, 20, 6, focus);
        assert!(rows[0].contains("w01"), "{rows:#?}");
        assert!(rows[2].contains('↓'), "capped at 3 rows: {rows:#?}");
        assert!(rows[3].contains("row 0"), "{rows:#?}");
        let offset = l.scroll.offset;
        assert_eq!(l.on_wheel(1), Outcome::Changed);
        let rows = rows_of(&mut l, 20, 6, focus);
        assert!(rows[0].contains('↑'), "{rows:#?}");
        assert_eq!(l.scroll.offset, offset, "the sheet itself did not move");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;

    fn sheet_text(props: &[Prop], w: u16) -> Vec<String> {
        let t = Theme::junie();
        let area = Rect::new(0, 0, w, 8);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &t, props, t.canvas);
        (0..8)
            .map(|y| {
                (0..w)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect()
    }

    /// A long label no longer pushes every value to the edge: it takes its
    /// own row and the others keep a short column (layout audit #14).
    #[test]
    fn a_label_wider_than_its_share_sits_above_its_value() {
        let props = vec![
            Prop::new("Name", "orders"),
            Prop::new("A rather long label for one fact", "value"),
        ];
        let lines = sheet_text(&props, 40);
        assert_eq!(lines[0], "Name  orders");
        assert_eq!(lines[1], "A rather long label for one fact");
        assert_eq!(lines[2], "      value");
        assert_eq!(measure(&props, 40), 3);
        // with room for the values the column stays as wide as its labels
        let lines = sheet_text(&props, 60);
        assert_eq!(lines[0], "Name                              orders");
    }

    #[test]
    fn a_sheet_too_narrow_for_its_values_stacks() {
        let props = vec![Prop::new("Database", "warehouse")];
        let lines = sheet_text(&props, 16);
        assert_eq!(lines[0], "Database");
        assert_eq!(lines[1], "  warehouse");
        // with the rules relaxed it keeps one row
        let t = Theme::junie();
        let area = Rect::new(0, 0, 16, 2);
        let mut buf = Buffer::empty(area);
        let sheet = Sheet {
            label_max_share: 100,
            value_min: 4,
        };
        assert_eq!(render_with(area, &mut buf, &t, &props, t.canvas, sheet), 1);
    }

    #[test]
    fn a_props_list_caps_its_label_column() {
        let mut l = PropsList::new(
            WidgetId::of("p"),
            vec![
                Prop::new("A rather long label for one fact", "value"),
                Prop::new("Name", "orders"),
            ],
        );
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 40, 2);
        let mut buf = Buffer::empty(area);
        l.render(area, &mut buf, &mut ctx, t.canvas);
        let row: String = (0..40).map(|x| buf[(x, 1)].symbol()).collect();
        // the label column is capped at 40% of the row: values stay visible
        assert!(row.contains("orders"), "{row:?}");
        let first: String = (0..40).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(first.contains('…') && first.contains("value"), "{first:?}");
    }
}

#[cfg(test)]
mod static_reveal_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    const PATH: &str = "/Users/alex/work/ide/a/b/c/d/e/deep.iml";

    fn frame(hover: Option<WidgetId>) -> (String, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            hover,
            ..Default::default()
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 3));
        {
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            let props = [Prop::new("Mode", "Trash"), Prop::new("Path", PATH)];
            render(
                Rect::new(0, 0, 24, 3),
                &mut buf,
                &theme,
                &props,
                theme.canvas,
            );
            ctx.flush_overlays(&mut buf);
        }
        let text = (0..3)
            .map(|y| (0..60).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        (text, hits)
    }

    #[test]
    fn a_static_sheet_reveals_a_cut_value_on_hover() {
        let (text, hits) = frame(None);
        assert!(!text.contains(PATH), "cut: {text}");
        // labels are 4 cells + 2: the value starts at column 6, row 1
        let id = hits
            .hover_at(Position::new(8, 1))
            .expect("the cut value hovers");
        assert_eq!(id, crate::ui::reveal::cut_id(Rect::new(6, 1, 1, 1)));
        assert_eq!(hits.hover_at(Position::new(8, 0)), None, "Trash fits");
        let (text, _) = frame(Some(id));
        assert!(text.contains(PATH), "{text}");
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::scroll::TrackClick;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::ui::ctx::Interaction;

    fn pressed_at_the_bottom(click: Option<TrackClick>, app: TrackClick) -> usize {
        let props = (0..50).map(|i| Prop::new("Key", format!("v{i}"))).collect();
        let mut l = PropsList::new(WidgetId::of("p"), props);
        l.track_click = click;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        ctx.track_click = app;
        l.render(area, &mut buf, &mut ctx, theme.surface);
        assert_eq!(l.on_scrollbar(Position::new(19, 9)), Outcome::Changed);
        l.scroll.offset
    }

    /// The vertical bar follows the track-click policy (it always jumped).
    #[test]
    fn a_props_track_click_can_page() {
        assert_eq!(
            pressed_at_the_bottom(Some(TrackClick::Page), TrackClick::Jump),
            10
        );
        assert_eq!(
            pressed_at_the_bottom(None, TrackClick::Page),
            10,
            "app-wide"
        );
        assert!(
            pressed_at_the_bottom(None, TrackClick::Jump) > 10,
            "default jumps"
        );
    }
}
