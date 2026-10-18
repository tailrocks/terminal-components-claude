use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup};
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::core::response::Response;
use crate::core::scroll::{ScrollPane, Scrollable};
use crate::core::selection::ClickPolicy;
use crate::theme::{Theme, Tone};
use crate::ui::ctx::RenderCtx;
use crate::ui::list_core::ListCore;
use crate::ui::reveal::{Reveal, RevealLine};
use crate::ui::row::{RowLook, RowUi};
use crate::widgets::scrollbar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sheet {
    pub label_max_share: u16,

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

const STACK_INDENT: u16 = 2;

struct Row<'a> {
    prop: &'a Prop,
    label: Option<&'a str>,
    value: String,
    value_x: u16,
}

fn column(widths: impl Iterator<Item = u16> + Clone, width: u16, sheet: Sheet) -> Option<u16> {
    let full = widths.clone().max().unwrap_or(2);
    if width.saturating_sub(full) >= sheet.value_min {
        return Some(full);
    }
    let cap = (width as u32 * sheet.label_max_share.min(100) as u32 / 100) as u16;

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

fn rows(props: &[Prop], width: u16, sheet: Sheet) -> Vec<Row<'_>> {
    let column = label_column(props, width, sheet);
    let mut out = vec![];
    for p in props {
        match column {
            Some(label_w) if crate::ui::text::width(&p.label) as u16 + 2 <= label_w => {
                let vw = width.saturating_sub(label_w) as usize;
                value_rows(p, Some(&p.label), label_w, vw, &mut out);
            }

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

pub fn measure(props: &[Prop], width: u16) -> u16 {
    measure_with(props, width, Sheet::default())
}

pub fn measure_with(props: &[Prop], width: u16, sheet: Sheet) -> u16 {
    rows(props, width, sheet).len() as u16
}

pub fn width(props: &[Prop]) -> u16 {
    width_with(props, Sheet::default())
}

pub fn width_with(props: &[Prop], sheet: Sheet) -> u16 {
    if props.is_empty() {
        return 0;
    }
    let w = |s: &str| crate::ui::text::width(s) as u16;
    let label = props.iter().map(|p| w(&p.label) + 2).max().unwrap_or(2);
    let value = props
        .iter()
        .map(|p| {
            if p.wrap {
                p.value.split_whitespace().map(w).max().unwrap_or(0)
            } else {
                w(&p.value)
            }
        })
        .max()
        .unwrap_or(0);
    label + value.max(sheet.value_min)
}

pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, props: &[Prop], bg: Color) -> u16 {
    render_with(area, buf, t, props, bg, Sheet::default())
}

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

fn natural(props: &[Prop], avail: ratatui::layout::Size) -> ratatui::layout::Size {
    ratatui::layout::Size::new(avail.width, measure(props, avail.width))
}

pub fn render_in(
    view: &mut crate::ui::scroll_view::ScrollView,
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    props: &[Prop],
    bg: Color,
) -> u16 {
    let area = area.intersection(*buf.area());
    if area.is_empty() || props.is_empty() {
        return 0;
    }
    let size = view.fit(area.as_size(), |a| natural(props, a));
    let area = Rect::new(area.x, area.y, area.width, size.height.min(area.height));
    let t = ctx.theme;
    view.render(area, size, buf, ctx, |r, b, _| {
        render(r, b, t, props, bg);
    });
    area.height
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropsAction {
    Copy(usize),
    Activate(usize),
}

#[derive(Debug, Clone, Copy)]
struct Geometry {
    row_w: u16,
    label_w: u16,
    value_right: u16,
}

#[derive(Debug, Clone)]
pub struct PropsList {
    pub id: WidgetId,
    pub props: Vec<Prop>,

    pub core: ListCore,
    pub area: Rect,

    pub scrollbar_gap: u16,

    pub sheet: Sheet,

    pub keymap: KeyMap<PropsCmd>,
}

impl std::ops::Deref for PropsList {
    type Target = ListCore;
    fn deref(&self) -> &ListCore {
        &self.core
    }
}

impl std::ops::DerefMut for PropsList {
    fn deref_mut(&mut self) -> &mut ListCore {
        &mut self.core
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropsCmd {
    Copy,
}

impl Actions for PropsCmd {
    fn label(self) -> &'static str {
        match self {
            PropsCmd::Copy => "Copy",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        vec![(KeyBinding::char('y'), PropsCmd::Copy)]
    }
}

impl PropsList {
    pub fn keymap(mut self, keymap: KeyMap<PropsCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.core.nav = nav;
        self
    }

    pub fn content(mut self, content: crate::ui::content::ContentState) -> Self {
        self.core.content = content;
        self
    }

    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.core.empty = Some(state);
        self
    }

    pub fn skip_when_empty(mut self, on: bool) -> Self {
        self.core.skip_when_empty = on;
        self
    }

    pub fn new(id: WidgetId, props: Vec<Prop>) -> Self {
        let n = props.len();
        Self {
            id,
            props,
            core: ListCore {
                type_ahead: false,
                ..ListCore::new(n)
            },
            area: Rect::ZERO,
            scrollbar_gap: scrollbar::GAP,
            sheet: Sheet::default(),
            keymap: PropsCmd::default_keymap(),
        }
    }

    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.core.scroll.track_click = Some(click);
        self
    }

    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    pub fn sheet(mut self, sheet: Sheet) -> Self {
        self.sheet = sheet;
        self
    }

    pub fn reveal(mut self, config: Reveal) -> Self {
        self.core.reveal.config = Some(config);
        self
    }

    pub fn set_props(&mut self, props: Vec<Prop>) {
        self.props = props;
        self.core.cursor = self.core.cursor.min(self.props.len().saturating_sub(1));
        self.core.scroll.set_content(self.props.len());
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::row_part(i))
    }

    fn row_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    pub(crate) fn row_of(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.props.len())
    }

    fn set_cursor(&mut self, i: usize) {
        self.core.cursor = i.min(self.props.len().saturating_sub(1));
        self.core.scroll.ensure_visible(self.core.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PropsAction>) {
        if self.props.is_empty() {
            return (Outcome::Ignored, None);
        }
        if let Some(o) = self.core.reveal.on_key(key) {
            return (o, None);
        }
        if let Some(PropsCmd::Copy) = lookup(&self.keymap, key) {
            if self.props[self.core.cursor].copyable {
                return (Outcome::Changed, Some(PropsAction::Copy(self.core.cursor)));
            }
            return (Outcome::Consumed, None);
        }
        let nav = self
            .nav
            .action(key)
            .filter(|n| key.plain() || matches!(n, Nav::PageUp | Nav::PageDown | Nav::End));
        if let Some(nav) = nav {
            let moved = self
                .cursor()
                .apply(nav, self, self.core.scroll.viewport_len);
            return match moved.index() {
                Some(i) => {
                    self.set_cursor(i);
                    (Outcome::Changed, None)
                }
                None => (Outcome::Ignored, None),
            };
        }
        match key.code {
            KeyCode::Enter => (
                Outcome::Changed,
                Some(PropsAction::Activate(self.core.cursor)),
            ),
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, row: usize) -> (Outcome, Option<PropsAction>) {
        self.on_click_count(row, 1)
    }

    pub fn on_click_count(&mut self, row: usize, count: u8) -> (Outcome, Option<PropsAction>) {
        if row >= self.props.len() {
            return (Outcome::Consumed, None);
        }
        let again = self.core.cursor == row;
        self.core.scroll.pointer_move();
        self.set_cursor(row);
        let act = self.core.click.activates(count.max(1), again);
        (Outcome::Changed, act.then_some(PropsAction::Activate(row)))
    }

    pub fn click(mut self, policy: ClickPolicy) -> Self {
        self.core.click = policy;
        self
    }

    fn hint_width(p: &Prop, focused: bool) -> u16 {
        if p.copyable && focused { 8 } else { 0 }
    }

    fn geometry(&self, body: Rect, content: Rect) -> Geometry {
        let row_w = body.width;

        let value_right = content.right().min(body.right().saturating_sub(1));

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

    fn inline_rows(
        &mut self,
        ctx: &mut RenderCtx,
        geo: Geometry,
        room: u16,
    ) -> Option<Vec<RevealLine>> {
        let p = self.props.get(self.core.cursor)?;
        let vx = self.area.x + 2 + geo.label_w;
        let vw = geo
            .value_right
            .saturating_sub(vx + Self::hint_width(p, true));
        let rid = self.row_id(self.core.cursor);
        let value = p.value.clone();
        self.core.reveal.inline(ctx, rid, vw, &value, true, room)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.core.scroll.begin(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.core.reveal.begin(ctx);
        self.core.scroll.set_content(self.props.len());
        self.core.scroll.set_viewport(area.height as usize);
        let n = self.props.len();

        if self.core.is_focus_stop(n, focused) {
            ctx.control(self.id, area, false);
            ctx.scrollable(self.id, area);
        }
        if (!self.core.content.is_ready() || self.core.empty.is_some())
            && self.core.draw_content(n, area, buf, ctx, bg, None)
        {
            return;
        }
        let mut has_sb = self.core.scroll.overflows();
        let mut sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
        let mut geo = self.geometry(sp.body, sp.content);

        let mut expanded = None;
        if focused {
            expanded = self.inline_rows(ctx, geo, area.height);
            if let Some(rows) = &expanded {
                let extra = rows.len().saturating_sub(1);
                self.core
                    .scroll
                    .set_viewport((area.height as usize).saturating_sub(extra).max(1));
                self.core.scroll.ensure_visible(self.core.cursor);
                if self.core.scroll.overflows() != has_sb {
                    has_sb = self.core.scroll.overflows();
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
        for i in self.core.scroll.visible_range() {
            if y >= area.bottom() {
                break;
            }
            let p = &self.props[i];
            let rid = self.row_id(i);
            let mut s = ctx.child_state(self.id, rid, i == self.core.cursor);
            s.focused = focused && i == self.core.cursor;
            let block = expanded.as_ref().filter(|_| i == self.core.cursor);
            let h = block
                .map_or(1, |r| r.len() as u16)
                .clamp(1, area.bottom() - y);
            let row = Rect::new(area.x, y, row_w, h);
            if i == self.core.cursor {
                cursor_rows.extend(row.y..row.bottom());
            }
            let rp = RowUi::paint(ctx, buf, row, s, RowLook::Bar, bg);
            let st = rp.style();
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
                for (k, line) in lines.iter().take(h as usize).enumerate() {
                    let ls = if line.more { st.fg(t.text_muted) } else { vs };
                    let text = crate::ui::text::truncate(&line.text, vw);
                    buf.set_string(vx, y + k as u16, text, ls);
                }
            } else {
                let cell = Rect::new(vx, y, vw as u16, 1);
                rp.label(ctx, buf, &mut self.core.reveal, rid, cell, &p.value, vs);
            }
            if hint_w > 0 {
                buf.set_string(
                    row.right().saturating_sub(7),
                    y,
                    "y copy",
                    st.fg(t.text_faint).remove_modifier(Modifier::BOLD),
                );
            }
            ctx.register_part(self.id, Self::row_part(i), row);
            y += h;
        }
        if let Some(sb) = sp.track {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                sp.body,
                &self.core.scroll,
                &cursor_rows,
            );
            self.core.scroll.draw_v(sb, buf, ctx, self.id, focused);
        }
    }
}

impl Scrollable for PropsList {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.core.scroll
    }

    fn wheel_first(&mut self, axis: Axis, delta: i32) -> Option<Response<()>> {
        if axis != Axis::Vertical {
            return None;
        }
        self.core.reveal.on_wheel(delta).map(Response::from)
    }
}

impl RowSource for PropsList {
    fn len(&self) -> usize {
        self.props.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.props[i].label
    }
}

impl Navigable for PropsList {
    fn cursor(&self) -> Cursor {
        self.core.cursor_model()
    }
}

impl crate::ui::specimen::Specimen for PropsList {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Focused | Loading | Error | Overflow | Narrow)
    }

    fn specimen_height(&self, state: crate::ui::specimen::SpecimenState) -> u16 {
        use crate::ui::specimen::SpecimenState::*;
        if matches!(state, Loading | Error) {
            return 1;
        }
        self.props.len().max(1) as u16
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut list = self.clone();
        if state == crate::ui::specimen::SpecimenState::Overflow
            && let Some(p) = list.props.first_mut()
        {
            p.value = format!("{} and a value far longer than its row", p.value);
        }
        list.core.content = crate::ui::content::specimen(state);
        list.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for PropsList {
    type Action = PropsAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<PropsAction> {
        use crate::core::intent::Phase;
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r.lift();
        }
        match ev.index_of(Part::ROW) {
            Some(row) if ev.is(Phase::Click) => self.on_click_count(row, ev.count()).into(),
            _ => crate::core::response::Response::ignored(),
        }
    }
}

#[cfg(test)]
mod reveal_scroll_tests {
    use super::*;

    #[test]
    fn content_states_replace_the_facts_and_an_empty_sheet_is_skipped() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::theme::Theme;
        use crate::ui::content::ContentState;
        use crate::ui::ctx::Interaction;
        use crate::widgets::empty::EmptyState;
        let draw = |p: &mut PropsList| {
            let theme = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 4));
            {
                let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
                p.render(Rect::new(0, 0, 40, 4), &mut buf, &mut ctx, theme.canvas);
            }
            let s: String = (0..4)
                .flat_map(|y| (0..40).map(move |x| (x, y)))
                .map(|c| buf[c].symbol().to_owned())
                .collect();
            (ring.contains(p.id), s)
        };
        let mut p = PropsList::new(WidgetId::of("p"), vec![Prop::new("Host", "db1")]);
        assert!(draw(&mut p).0);
        p.content = ContentState::loading("Reading facts");
        let (stop, s) = draw(&mut p);
        assert!(
            !stop && s.contains("Reading facts") && !s.contains("db1"),
            "{s}"
        );
        p.content = ContentState::error(EmptyState::new("No access"));
        assert!(draw(&mut p).1.contains("! No access"));
        let mut empty =
            PropsList::new(WidgetId::of("e"), vec![]).empty(EmptyState::new("No facts"));
        let (stop, s) = draw(&mut empty);
        assert!(!stop && s.contains("No facts"), "{s}");
    }
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
        assert_eq!(l.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
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
        assert_eq!(l.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
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

    #[test]
    fn a_sheet_taller_than_its_area_scrolls_to_its_last_row() {
        use crate::core::pointer::PointerEvent;
        use crate::ui::scroll_view::{Axes, ScrollView};
        use ratatui::crossterm::event::KeyModifiers;
        let t = Theme::junie();
        let props: Vec<Prop> = (0..6)
            .map(|i| Prop::new(&format!("Fact{i}"), format!("value{i}")))
            .collect();
        let id = WidgetId::of("sheet");
        let mut view = ScrollView::new(id).axes(Axes::VERTICAL);
        let draw = |view: &mut ScrollView| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 6));
            let used = {
                let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
                let used = render_in(
                    view,
                    Rect::new(0, 0, 30, 3),
                    &mut buf,
                    &mut ctx,
                    &props,
                    t.canvas,
                );
                ctx.flush_overlays(&mut buf);
                used
            };
            let text: String = (0..3)
                .flat_map(|y| (0..30).map(move |x| (x, y)))
                .map(|p| buf[p].symbol().to_owned())
                .collect();
            (used, text, hits)
        };
        let (used, text, hits) = draw(&mut view);
        assert_eq!(used, 3);
        assert!(text.contains("Fact0") && !text.contains("Fact5"));

        let ev = PointerEvent::wheel_on(
            &hits,
            id,
            Axis::Vertical,
            5,
            ratatui::layout::Position::new(2, 1),
            KeyModifiers::NONE,
        );
        assert!(view.pointer(&ev).is_some());
        let (_, text, _) = draw(&mut view);
        assert!(text.contains("Fact5"), "{text}");

        let mut view = ScrollView::new(id).axes(Axes::VERTICAL);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        assert_eq!(
            render_in(
                &mut view,
                Rect::new(0, 0, 30, 10),
                &mut buf,
                &mut ctx,
                &props,
                t.canvas
            ),
            6
        );
    }

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

        let lines = sheet_text(&props, 60);
        assert_eq!(lines[0], "Name                              orders");
    }

    #[test]
    fn width_shows_every_label_and_value_whole() {
        let props = vec![
            Prop::new("Name", "orders-archive"),
            Prop::new("Keys", "Enter opens · Backspace goes up").wrap(),
        ];
        let w = width(&props);
        assert_eq!(w, 6 + 14);
        let lines = sheet_text(&props, w);
        assert_eq!(lines[0], "Name  orders-archive");
        assert_eq!(lines[1], "Keys  Enter opens ·");
        assert_eq!(lines[2], "      Backspace goes");
        assert_eq!(lines[3], "      up");
        assert_eq!(measure(&props, w), 4);
        assert!(lines.iter().all(|l| !l.contains('…')), "{lines:?}");

        assert_eq!(width(&[Prop::new("A", "b")]), 3 + 12);
    }

    #[test]
    fn a_sheet_too_narrow_for_its_values_stacks() {
        let props = vec![Prop::new("Database", "warehouse")];
        let lines = sheet_text(&props, 16);
        assert_eq!(lines[0], "Database");
        assert_eq!(lines[1], "  warehouse");

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
    use ratatui::layout::Position;

    fn pressed_at_the_bottom(click: Option<TrackClick>, app: TrackClick) -> usize {
        let props = (0..50).map(|i| Prop::new("Key", format!("v{i}"))).collect();
        let mut l = PropsList::new(WidgetId::of("p"), props);
        l.scroll.track_click = click;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        ctx.track_click = app;
        l.render(area, &mut buf, &mut ctx, theme.surface);
        assert_eq!(
            l.bar_press(Position::new(19, 9))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        l.scroll.offset
    }

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
