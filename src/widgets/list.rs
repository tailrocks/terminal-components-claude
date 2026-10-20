use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{
    Actions, KeyBinding, KeyMap, NavKeys, lookup, select_all_bindings, unassigned_chord,
};
use crate::core::list::{Cursor, KeyStep, Moved, Navigable, RowSource, TypeAheadPrecedence};
use crate::core::response::Response;
use crate::core::scroll::{ScrollPane, Scrollable};
use crate::core::selection::{ClickPolicy, RowSelection, SelectGesture};
use crate::theme::Tone;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::list_core::ListCore;
use crate::ui::reveal::{Reveal, RevealLine};
use crate::ui::row::{MarkKind, RowLook, RowUi};
use crate::widgets::scrollbar;

#[derive(Debug, Clone)]
pub struct ListItem {
    pub label: String,
    pub meta: Option<String>,

    pub meta_tone: Tone,
    pub disabled: bool,
}

impl ListItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            meta: None,
            meta_tone: Tone::Muted,
            disabled: false,
        }
    }
    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }

    pub fn meta_tone(mut self, tone: Tone) -> Self {
        self.meta_tone = tone;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Single,
    Multi,
}

#[derive(Debug, Clone)]
pub struct ListBox {
    pub id: WidgetId,
    pub items: Vec<ListItem>,
    pub mode: SelectMode,
    pub chosen: Option<usize>,

    pub core: ListCore,
    pub area: Rect,

    pub scrollbar_gap: u16,

    pub toggle_mods: KeyModifiers,

    pub keymap: KeyMap<ListCmd>,
}

impl std::ops::Deref for ListBox {
    type Target = ListCore;
    fn deref(&self) -> &ListCore {
        &self.core
    }
}

impl std::ops::DerefMut for ListBox {
    fn deref_mut(&mut self) -> &mut ListCore {
        &mut self.core
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListCmd {
    ToggleAll,

    SelectAll,
}

impl Actions for ListCmd {
    fn label(self) -> &'static str {
        match self {
            ListCmd::ToggleAll => "Toggle all",
            ListCmd::SelectAll => "Select all",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        let mut map = vec![(KeyBinding::char('a'), ListCmd::ToggleAll)];
        map.extend(select_all_bindings(ListCmd::SelectAll));
        map
    }
}

impl ListBox {
    pub fn new(id: WidgetId, items: Vec<ListItem>, mode: SelectMode) -> Self {
        let n = items.len();
        Self {
            id,
            items,
            mode,
            chosen: None,
            core: ListCore::new(n),
            area: Rect::ZERO,
            scrollbar_gap: scrollbar::GAP,
            toggle_mods: SelectGesture::toggle_mods(),
            keymap: ListCmd::default_keymap(),
        }
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.core.nav = nav;
        self
    }

    pub fn inactive_selection(mut self, look: crate::theme::InactiveSelection) -> Self {
        self.core.inactive_selection = Some(look);
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<ListCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.core.wrap = on;
        self
    }

    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.core.typeahead.timeout = timeout;
        self
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.core.type_ahead = on;
        self
    }

    pub fn type_ahead_precedence(mut self, precedence: TypeAheadPrecedence) -> Self {
        self.core.type_ahead_precedence = Some(precedence);
        self
    }

    pub fn skip_disabled(mut self, on: bool) -> Self {
        self.core.skip_disabled = on;
        self
    }

    pub fn click(mut self, click: ClickPolicy) -> Self {
        self.core.click = click;
        self
    }

    pub fn toggle_mods(mut self, mods: KeyModifiers) -> Self {
        self.toggle_mods = mods;
        self
    }

    fn order(&self) -> Vec<usize> {
        (0..self.items.len()).collect()
    }

    pub fn select(&mut self, g: SelectGesture, to: usize) -> bool {
        let order = self.order();
        let items = &self.items;
        self.core.selection.apply(g, to, &order, |i| {
            items.get(i).is_some_and(|it| !it.disabled)
        })
    }

    pub fn remove_rows(&mut self, rows: &[usize]) -> usize {
        let gone: std::collections::BTreeSet<usize> = rows
            .iter()
            .copied()
            .filter(|&r| r < self.items.len())
            .collect();
        if gone.is_empty() {
            return 0;
        }
        let map = |r: usize| (!gone.contains(&r)).then(|| r - gone.range(..r).count());
        let mut i = 0;
        self.items.retain(|_| {
            let keep = !gone.contains(&i);
            i += 1;
            keep
        });
        self.core.selection.remap(map);
        self.chosen = self.chosen.and_then(map);
        let n = self.items.len();
        self.core.cursor = map(self.core.cursor)
            .unwrap_or_else(|| self.core.cursor - gone.range(..self.core.cursor).count())
            .min(n.saturating_sub(1));
        gone.len()
    }

    pub fn is_checked(&self, i: usize) -> bool {
        self.core.selection.contains(i)
    }

    pub fn set_checked(&mut self, i: usize, on: bool) {
        if i < self.items.len() {
            self.core.selection.set(i, on);
        }
    }

    pub fn mode(mut self, mode: SelectMode) -> Self {
        self.mode = mode;
        self
    }

    fn nav_cursor(&self) -> Cursor {
        Cursor::new(self.core.cursor)
            .wrap(self.core.wrap)
            .skip_disabled(self.core.skip_disabled)
    }

    pub fn scrollbar_gap(mut self, gap: u16) -> Self {
        self.scrollbar_gap = gap;
        self
    }

    pub fn reveal(mut self, config: Reveal) -> Self {
        self.core.reveal.config = Some(config);
        self
    }

    pub fn reveal_hint(&self) -> Option<(&'static str, &'static str)> {
        self.core.reveal.hint()
    }

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

    fn inline_rows(
        &mut self,
        ctx: &mut RenderCtx,
        text_w: usize,
        room: u16,
    ) -> Option<Vec<RevealLine>> {
        let item = self.items.get(self.core.cursor).filter(|it| !it.disabled)?;
        let (lw, _) = Self::label_width(item, text_w);
        let rid = self.row_id(self.core.cursor);
        let label = item.label.clone();
        self.core
            .reveal
            .inline(ctx, rid, lw as u16, &label, true, room)
    }

    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.core.empty = Some(state);
        self
    }

    pub fn empty_text(mut self, s: &str) -> Self {
        self.core.empty = Some(crate::widgets::empty::EmptyState::new(s));
        self
    }

    pub fn content(mut self, content: crate::ui::content::ContentState) -> Self {
        self.core.content = content;
        self
    }

    pub fn skip_when_empty(mut self, on: bool) -> Self {
        self.core.skip_when_empty = on;
        self
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::row_part(i))
    }

    fn row_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    pub fn mark_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::mark_part(i))
    }

    fn mark_part(i: usize) -> PartRef {
        PartRef::item(Part::MARKER, ItemKey::index(i))
    }

    pub fn checked_count(&self) -> usize {
        self.core.selection.len()
    }

    fn move_cursor(&mut self, to: usize, extend: bool) {
        let to = to.min(self.items.len().saturating_sub(1));
        let from = self.core.cursor;
        self.core.cursor = to;
        if self.mode == SelectMode::Multi && extend {
            self.select(SelectGesture::Extend { from }, to);
        }
        self.core.scroll.ensure_visible(self.core.cursor);
    }

    pub fn activate(&mut self, i: usize) -> Outcome {
        if i >= self.items.len() || self.items[i].disabled {
            return Outcome::Consumed;
        }
        match self.mode {
            SelectMode::Single => self.chosen = Some(i),
            SelectMode::Multi => {
                self.select(SelectGesture::Toggle, i);
            }
        }
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.items.is_empty() {
            return Outcome::Ignored;
        }
        if let Some(o) = self.core.reveal.on_key(key) {
            return o;
        }
        let shift = key.shift();
        if unassigned_chord(key, &self.keymap) {
            return Outcome::Ignored;
        }
        if self.mode == SelectMode::Single && lookup(&self.keymap, key) == Some(ListCmd::SelectAll)
        {
            return Outcome::Ignored;
        }

        if let Some(action) = lookup(&self.keymap, key)
            && self.mode == SelectMode::Multi
        {
            return match action {
                ListCmd::ToggleAll => {
                    let items = &self.items;
                    self.core
                        .selection
                        .toggle_all(0..items.len(), |i| !items[i].disabled);
                    Outcome::Changed
                }
                ListCmd::SelectAll => {
                    if self.select(SelectGesture::All, 0) {
                        Outcome::Changed
                    } else {
                        Outcome::Consumed
                    }
                }
            };
        }
        let items = &self.items[..];
        match self.core.key_step(key, items) {
            Some((KeyStep::Type(_), moved)) => {
                if let Moved::To(i) = moved {
                    self.move_cursor(i, false);
                }
                return moved.outcome();
            }
            Some((KeyStep::Nav(_), moved)) => {
                let Some(to) = moved.index() else {
                    return Outcome::Ignored;
                };
                self.move_cursor(to, shift);
                return Outcome::Changed;
            }
            None => {}
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.core.cursor),
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, row: usize) -> Outcome {
        self.on_click_count(row, 1)
    }

    pub fn on_click_count(&mut self, row: usize, count: u8) -> Outcome {
        self.on_click_at(row, count, KeyModifiers::NONE, false)
    }

    pub fn on_click_at(
        &mut self,
        row: usize,
        count: u8,
        mods: KeyModifiers,
        on_box: bool,
    ) -> Outcome {
        if row >= self.items.len() {
            return Outcome::Consumed;
        }
        let from = self.core.cursor;
        let again = from == row;
        self.core.cursor = row;
        self.core.scroll.pointer_move();
        self.core.scroll.ensure_visible(row);
        match self.mode {
            SelectMode::Single => {
                if self.core.click.activates(count.max(1), again) {
                    return self.activate(row);
                }
            }
            SelectMode::Multi => {
                let plain = !mods.intersects(self.toggle_mods | KeyModifiers::SHIFT);
                let g = if plain && self.core.click.toggles(on_box) {
                    SelectGesture::Toggle
                } else {
                    SelectGesture::from_mods(mods, self.toggle_mods, from)
                };

                if g == SelectGesture::Only && self.items[row].disabled {
                    return Outcome::Changed;
                }
                self.select(g, row);
            }
        }
        Outcome::Changed
    }

    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.core.scroll.track_click = Some(click);
        self
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
        self.core.scroll.set_content(self.items.len());
        self.core.scroll.set_viewport(area.height as usize);
        let n = self.items.len();

        if self.core.is_focus_stop(n, focused) {
            ctx.control(self.id, area, false);
            ctx.scrollable(self.id, area);
        }
        if self
            .core
            .draw_content(n, area, buf, ctx, bg, Some(t.strings.empty_list))
        {
            return;
        }
        let mut has_sb = self.core.scroll.overflows();

        let mut sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
        let mut text_w = sp.content.width.saturating_sub(1) as usize;

        let mut expanded = None;
        if focused {
            expanded = self.inline_rows(ctx, text_w, area.height);
            if let Some(rows) = &expanded {
                let extra = rows.len().saturating_sub(1);
                self.core
                    .scroll
                    .set_viewport((area.height as usize).saturating_sub(extra).max(1));
                self.core.scroll.ensure_visible(self.core.cursor);
                if self.core.scroll.overflows() != has_sb {
                    has_sb = self.core.scroll.overflows();
                    sp = scrollbar::split(area, has_sb, self.scrollbar_gap);
                    text_w = sp.content.width.saturating_sub(1) as usize;
                    expanded = self.inline_rows(ctx, text_w, area.height);
                }
            }
        }
        let mut y = area.y;
        for li in self.core.scroll.visible_range() {
            if y >= area.bottom() {
                break;
            }
            let item = &self.items[li];
            let block = expanded.as_ref().filter(|_| li == self.core.cursor);
            let h = block
                .map_or(1, |r| r.len() as u16)
                .clamp(1, area.bottom() - y);
            let row = Rect::new(area.x, y, sp.body.width, h);
            let rid = self.row_id(li);
            let mut s = ctx.child_state(self.id, rid, li == self.core.cursor);
            if self.mode == SelectMode::Multi {
                let mark = self.mark_id(li);
                s.hovered |= ctx.interaction.hovered(mark);
                s.pressed |= ctx.interaction.pressed(mark);
            }
            s.focused = focused && li == self.core.cursor;
            s.disabled = item.disabled;
            s.selected = match self.mode {
                SelectMode::Single => self.chosen == Some(li),
                SelectMode::Multi => self.core.selection.contains(li),
            };
            s.inactive_selection = self.core.inactive_selection;
            let rp = RowUi::paint(ctx, buf, row, s, RowLook::Bar, bg);
            let st = rp.style();
            if s.selected {
                let kind = match self.mode {
                    SelectMode::Single => MarkKind::Current,
                    SelectMode::Multi => MarkKind::Tick,
                };
                rp.mark(ctx, buf, row.x + 1, y, kind, focused);
            }
            let label_w = text_w.saturating_sub(2);
            let (lw, meta_w) = Self::label_width(item, text_w);
            if let Some(rows) = block {
                for (k, line) in rows.iter().take(h as usize).enumerate() {
                    let ls = if line.more { st.fg(t.text_muted) } else { st };
                    let text = crate::ui::text::fit(&line.text, lw);
                    buf.set_string(row.x + 3, y + k as u16, text, ls);
                }
            } else {
                let label = Rect::new(row.x + 3, y, lw as u16, 1);
                rp.label(ctx, buf, &mut self.core.reveal, rid, label, &item.label, st);
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
                } else if item.meta_tone == Tone::Muted {
                    st.fg(t.text_muted)
                } else {
                    st.fg(t.tone_on(item.meta_tone, st.bg.unwrap_or(t.canvas)))
                };
                buf.set_string(mx, y, m, ms);
            }
            ctx.register_part(self.id, Self::row_part(li), row);
            if self.mode == SelectMode::Multi {
                let mark = Rect::new(row.x, row.y, row.width.min(3), row.height);
                ctx.register_part(self.id, Self::mark_part(li), mark);
            }
            y += h;
        }

        ctx.scrollable(self.id, area);
        if let Some(sb) = sp.track {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                sp.body,
                &self.core.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.core.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            self.core.scroll.draw_v(sb, buf, ctx, self.id, focused);
        }
    }
}

impl crate::ui::measure::Measure for ListBox {
    fn measure(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        if self.items.is_empty() {
            return match &self.core.empty {
                Some(e) => e.measure(avail, ctx),
                None => ratatui::layout::Size::new(0, 0),
            };
        }
        let width = self
            .items
            .iter()
            .map(|item| {
                let label = crate::ui::text::width(&item.label) as u16;

                let text = match item.meta.as_deref().filter(|m| !m.is_empty()) {
                    Some(m) => 2 + label.max(12) + 2 + crate::ui::text::width(m) as u16,
                    None => 2 + label,
                };
                text + 1
            })
            .max()
            .unwrap_or(0);
        ratatui::layout::Size::new(width, self.items.len() as u16)
    }
}

impl Scrollable for ListBox {
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

impl RowSource for [ListItem] {
    fn len(&self) -> usize {
        <[ListItem]>::len(self)
    }
    fn enabled(&self, i: usize) -> bool {
        !self[i].disabled
    }
    fn label(&self, i: usize) -> &str {
        &self[i].label
    }
}

impl RowSource for ListBox {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn enabled(&self, i: usize) -> bool {
        self.items[..].enabled(i)
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].label
    }
}

impl Navigable for ListBox {
    fn cursor(&self) -> Cursor {
        self.nav_cursor()
    }
}

impl crate::ui::specimen::Specimen for ListBox {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover
                | Pressed
                | Focused
                | FocusedHover
                | Selected
                | Disabled
                | Empty
                | Loading
                | Error
                | Overflow
                | Narrow
        )
    }

    fn specimen_height(&self, state: crate::ui::specimen::SpecimenState) -> u16 {
        use crate::ui::specimen::SpecimenState::*;
        if matches!(state, Empty | Loading | Error) {
            1
        } else {
            self.items.len().clamp(1, 3) as u16
        }
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
        let mut l = self.clone();
        l.items.truncate(3);
        l.selection = RowSelection::new();
        l.chosen = None;
        l.cursor = 1.min(l.items.len().saturating_sub(1));
        l.scroll = ScrollPane::new(l.items.len());
        let c = l.cursor;
        match state {
            Selected => match l.mode {
                SelectMode::Single => l.chosen = Some(c),
                SelectMode::Multi => l.set_checked(c, true),
            },
            Disabled => {
                if let Some(it) = l.items.get_mut(c) {
                    it.disabled = true;
                }
            }
            Empty => {
                l.items.clear();
                l.selection = RowSelection::new();
            }
            Loading | Error => l.content = crate::ui::content::specimen(state),
            Overflow => {
                if let Some(it) = l.items.get_mut(c) {
                    it.label = format!("{} and a label far longer than its row", it.label);
                }
            }
            _ => {}
        }

        let v = state.visual();
        if (v.hovered || state == Disabled) && l.has_rows(l.items.len()) {
            ctx.interaction.hover = Some(l.row_id(c));
            if v.pressed {
                ctx.interaction.pressed = Some(l.row_id(c));
            }
        }
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        l.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for ListBox {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(&mut self, ev: &crate::core::pointer::PointerEvent) -> Response<()> {
        use crate::core::intent::Phase;
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r;
        }
        if !ev.is(Phase::Click) {
            return Response::ignored();
        }
        let hit = match (ev.index_of(Part::MARKER), ev.index_of(Part::ROW)) {
            (Some(row), _) => Some((row, true)),
            (None, Some(row)) => Some((row, false)),
            _ => None,
        };
        match hit {
            Some((row, on_box)) => self
                .on_click_at(row, ev.count().max(1), ev.mods(), on_box)
                .into(),
            None => Response::ignored(),
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

    #[test]
    fn content_states_replace_the_rows_and_empty_lists_are_skipped() {
        use crate::ui::content::ContentState;
        use crate::widgets::empty::{EmptyAction, EmptyState};
        let theme = Theme::junie();
        let draw = |l: &mut ListBox| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 6));
            {
                let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
                l.render(Rect::new(0, 0, 30, 6), &mut buf, &mut ctx, theme.canvas);
            }
            let s: String = (0..6)
                .flat_map(|y| (0..30).map(move |x| (x, y)))
                .map(|p| buf[p].symbol().to_owned())
                .collect();
            (ring.contains(l.id), s, hits)
        };
        let mut l = list();
        assert!(draw(&mut l).0, "rows: a focus stop");
        l.content = ContentState::loading("Loading files");
        let (stop, s, _) = draw(&mut l);
        assert!(
            !stop && s.contains("Loading files") && !s.contains("short"),
            "{s}"
        );
        let retry = WidgetId::of("retry");
        l.content = ContentState::error(
            EmptyState::new("Could not list files")
                .actions(vec![EmptyAction::primary(retry, "Retry")]),
        );
        let (stop, s, hits) = draw(&mut l);
        assert!(!stop && s.contains("Could not list files"), "{s}");
        assert!(hits.regions().iter().any(|r| r.id == retry));
        let mut empty = ListBox::new(WidgetId::of("e"), vec![], SelectMode::Single);
        let (stop, s, _) = draw(&mut empty);
        assert!(!stop, "K37: an empty list is skipped");
        assert!(s.contains(theme.strings.empty_list), "{s}");
        let mut kept = ListBox::new(WidgetId::of("e"), vec![], SelectMode::Single)
            .skip_when_empty(false)
            .empty_text("Nothing yet");
        let (stop, s, _) = draw(&mut kept);
        assert!(stop && s.contains("Nothing yet"), "knob: stays a stop");
    }

    #[test]
    fn the_chosen_mark_is_accent_only_while_the_list_has_focus() {
        let theme = Theme::junie().with_inactive_selection(crate::theme::InactiveSelection::None);
        let mark = |interaction: Interaction| {
            let mut l = list();
            l.chosen = Some(1);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
            l.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, theme.canvas);
            buf[(1, 1)].clone()
        };
        let rest = mark(Interaction::default());
        assert_eq!(rest.symbol(), theme.glyphs.current);
        assert_eq!(rest.fg, theme.text_primary);
        let l = list();
        let hover = mark(Interaction {
            hover: Some(l.row_id(1)),
            ..Default::default()
        });
        assert_eq!(hover.fg, theme.text_primary, "hover is not focus");
        let focused = mark(Interaction {
            focus: Some(l.id),
            ..Default::default()
        });
        assert_eq!(focused.fg, theme.accent);
    }

    #[test]
    fn the_chosen_row_is_bold_at_every_colour_level() {
        for level in crate::theme::ColorLevel::ALL {
            let theme = Theme::for_level(level);
            let mut l = list();
            l.chosen = Some(1);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
            l.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, theme.canvas);
            assert!(
                buf[(3, 1)]
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD),
                "{level:?}"
            );
            assert!(
                !buf[(3, 0)]
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD),
                "{level:?}"
            );
        }
    }

    #[test]
    fn a_focused_cut_cursor_row_peeks_its_full_label() {
        let theme = Theme::junie();
        let peek = |cursor: usize, focused: bool| {
            let mut l = list();
            l.cursor = cursor;
            let interaction = Interaction {
                focus: focused.then_some(l.id),
                ..Default::default()
            };
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
            l.render(Rect::new(0, 0, 20, 3), &mut buf, &mut ctx, theme.canvas);
            ctx.peek
        };
        assert_eq!(peek(1, true).as_deref(), Some(LONG));
        assert_eq!(peek(1, false), None);
        assert_eq!(peek(0, true), None);
    }

    #[test]
    fn meta_tone_colours_the_metadata_and_defaults_to_muted() {
        let theme = Theme::junie();
        let items = vec![
            ListItem::new("fast").meta("38 ms"),
            ListItem::new("slow")
                .meta("1.82 s")
                .meta_tone(Tone::Warning),
        ];
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 4));
        l.render(Rect::new(0, 0, 40, 4), &mut buf, &mut ctx, theme.canvas);
        let at = |y: u16, s: &str| {
            (0..40)
                .find(|&x| buf[(x, y)].symbol() == s)
                .map(|x| buf[(x, y)].clone())
                .unwrap()
        };
        assert_eq!(at(0, "3").fg, theme.text_muted);
        let slow = at(1, "1");
        assert_eq!(slow.fg, theme.tone_on(Tone::Warning, slow.bg));
    }

    #[test]
    fn a_cut_label_keeps_the_gap_before_the_scrollbar() {
        let items: Vec<ListItem> = (0..8)
            .map(|i| ListItem::new(&format!("{i} {LONG}")))
            .collect();
        let mut l = ListBox::new(WidgetId::of("l"), items.clone(), SelectMode::Single);
        let text = render(&mut l, Interaction::default());
        let row: Vec<char> = text.lines().next().unwrap().chars().take(20).collect();
        assert!(
            matches!(row[19], '┆' | '┃'),
            "bar in the last column: {row:?}"
        );
        assert_eq!(row[18], ' ', "one blank column before the bar: {row:?}");
        assert_eq!(row[17], '…', "the label ends with its ellipsis: {row:?}");

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
    fn a_list_at_its_measure_shows_every_label_and_meta() {
        use crate::ui::measure::Measure;
        let mut l = ListBox::new(
            WidgetId::of("l"),
            vec![
                ListItem::new("orders").meta("1,194"),
                ListItem::new("customer_addresses").meta("88,201"),
                ListItem::new("a"),
                ListItem::new("a label far longer than the rest of them"),
            ],
            SelectMode::Single,
        );
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let size = l.measure(ratatui::layout::Size::new(10, 2), &ctx);
        drop(ctx);
        assert_eq!(size.height, 4);
        let rows = render_in(&mut l, size.width, size.height, Interaction::default());
        for (row, parts) in rows.iter().zip([
            &["orders", "1,194"][..],
            &["customer_addresses", "88,201"],
            &["a"],
            &["a label far longer than the rest of them"],
        ]) {
            for p in parts {
                assert!(row.contains(p), "{p:?} in {row:?}");
            }
        }
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
        assert_eq!(l.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
        let rows = render_in(&mut l, 16, 6, focus);
        assert!(rows[0].contains("↑"), "{rows:#?}");
        assert_eq!(l.scroll.offset, offset, "the list itself did not move");

        press(&mut l, KeyCode::Left);
        render_in(&mut l, 16, 6, focus);
        assert_eq!(l.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
        assert_eq!(l.scroll.offset, offset + 1);
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;
    use ratatui::layout::Position;

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
        let mut desk = disabled_list();
        desk.cursor = 1;
        press(&mut desk, KeyCode::Char('d'));
        assert_eq!(desk.cursor, 3, "on by default (Desktop profile, K34)");
        let mut vim = disabled_list()
            .nav(crate::core::keymap::NavKeys::VIM)
            .type_ahead(false);
        vim.cursor = 1;
        press(&mut vim, KeyCode::Char('j'));
        assert_eq!(vim.cursor, 3, "off with vim keys: j moves down");
    }

    #[test]
    fn a_type_ahead_miss_is_consumed_and_a_bound_letter_still_runs() {
        let mut l = disabled_list().type_ahead(true);
        l.cursor = 1;
        let o = l.on_key(&Key {
            code: KeyCode::Char('x'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert_eq!(o, Outcome::Consumed);
        assert_eq!(l.cursor, 1);

        let mut m = ListBox::new(
            WidgetId::of("m"),
            vec![ListItem::new("apple"), ListItem::new("bean")],
            SelectMode::Multi,
        )
        .type_ahead(true);
        m.cursor = 1;
        press(&mut m, KeyCode::Char('a'));
        assert_eq!(m.selection.to_vec(), vec![0, 1], "a toggles all");
        assert_eq!(m.cursor, 1);
    }

    #[test]
    fn the_type_ahead_pause_is_configurable() {
        let items = || {
            vec![
                ListItem::new("alpha"),
                ListItem::new("bravo"),
                ListItem::new("romeo"),
            ]
        };
        let mut patient = ListBox::new(WidgetId::of("p"), items(), SelectMode::Single);
        press(&mut patient, KeyCode::Char('b'));
        std::thread::sleep(std::time::Duration::from_millis(5));
        press(&mut patient, KeyCode::Char('r'));
        assert_eq!(patient.cursor, 1, "`br` within 1 s stays on bravo");
        let mut quick = ListBox::new(WidgetId::of("q"), items(), SelectMode::Single)
            .type_ahead_timeout(std::time::Duration::ZERO);
        press(&mut quick, KeyCode::Char('b'));
        std::thread::sleep(std::time::Duration::from_millis(5));
        press(&mut quick, KeyCode::Char('r'));
        assert_eq!(quick.cursor, 2, "after the pause `r` starts over: romeo");
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
        let mut l = disabled_list();
        l.on_click_count(3, 1);
        assert_eq!((l.cursor, l.chosen), (3, None));
        l.on_click_count(3, 1);
        assert_eq!(l.chosen, None, "a slow second click only selects");
        l.on_click_count(3, 2);
        assert_eq!(l.chosen, Some(3));

        let mut act = disabled_list().click(ClickPolicy::NAVIGATION);
        act.on_click(1);
        assert_eq!(act.chosen, Some(1));
    }

    fn multi() -> ListBox {
        let items = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|l| ListItem::new(l).disabled(*l == "d"))
            .collect();
        ListBox::new(WidgetId::of("m"), items, SelectMode::Multi)
            .toggle_mods(SelectGesture::TOGGLE_CTRL_CMD)
    }

    #[test]
    fn multi_clicks_follow_the_selection_gestures() {
        let none = KeyModifiers::NONE;
        let mut l = multi();
        l.on_click_at(1, 1, none, false);
        assert_eq!(l.selection.to_vec(), vec![1], "a plain click selects only");
        l.on_click_at(4, 1, KeyModifiers::SHIFT, false);
        assert_eq!(
            l.selection.to_vec(),
            vec![1, 2, 4],
            "Shift extends, skips d"
        );
        l.on_click_at(2, 1, KeyModifiers::CONTROL, false);
        assert_eq!(l.selection.to_vec(), vec![1, 4], "Ctrl toggles");
        l.on_click_at(0, 1, none, true);
        assert_eq!(l.selection.to_vec(), vec![0, 1, 4], "the ✓ column toggles");
        l.on_click_at(3, 1, none, false);
        assert_eq!(l.cursor, 3, "a disabled row takes the cursor");
        assert_eq!(
            l.selection.to_vec(),
            vec![0, 1, 4],
            "...and the selection stays"
        );

        let mut c = multi().click(ClickPolicy::DATA.check(crate::core::selection::CheckClick::Row));
        c.on_click_at(0, 1, none, false);
        c.on_click_at(2, 1, none, false);
        assert_eq!(c.selection.to_vec(), vec![0, 2]);
        c.on_click_at(0, 1, none, false);
        assert_eq!(c.selection.to_vec(), vec![2]);
    }

    #[test]
    fn a_click_on_the_mark_column_toggles_through_the_hit_regions() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        use crate::theme::Theme;
        use crate::ui::ctx::Interaction;
        let mut l = multi();
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        l.render(area, &mut buf, &mut ctx, theme.canvas);
        drop(ctx);
        let click = |l: &mut ListBox, pos: Position| {
            let id = hits.hit(pos).expect("a part");
            let ev = PointerEvent::new(&hits, Phase::Click, id, pos, 1, KeyModifiers::NONE);
            let _ = l.pointer(&ev);
        };
        click(&mut l, Position::new(1, 0));
        click(&mut l, Position::new(1, 2));
        assert_eq!(l.selection.to_vec(), vec![0, 2], "the ✓ column toggles");
        click(&mut l, Position::new(8, 4));
        assert_eq!(l.selection.to_vec(), vec![4], "the label selects only");
        assert_eq!(l.cursor, 4);
    }

    #[test]
    fn shift_arrows_extend_and_shrink_from_the_clicked_anchor() {
        let shift = |code| Key {
            code,
            mods: KeyModifiers::SHIFT,
        };
        let mut l = multi();
        l.on_click_at(1, 1, KeyModifiers::NONE, false);
        l.on_key(&shift(KeyCode::Down));
        l.on_key(&shift(KeyCode::Down));
        assert_eq!(l.selection.to_vec(), vec![1, 2, 4], "d is skipped");
        l.on_key(&shift(KeyCode::Up));
        assert_eq!(l.selection.to_vec(), vec![1, 2], "the range shrinks");
        l.on_key(&shift(KeyCode::Up));
        l.on_key(&shift(KeyCode::Up));
        assert_eq!(l.selection.to_vec(), vec![0, 1], "and crosses the anchor");
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

        let b = render(&mut l);
        assert_eq!(b[(4, 0)].fg, theme.text_primary, "cursor row untouched");
        assert!(
            grey(&b, 5) < grey(&b, 4),
            "bottom row fades: {}",
            grey(&b, 5)
        );

        l.wheel(Axis::Vertical, 10).outcome();
        let b = render(&mut l);
        assert!(grey(&b, 0) < grey(&b, 1), "top row fades");
        assert!(grey(&b, 5) < grey(&b, 4));

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
        assert_eq!(
            l.wheel(Axis::Vertical, -1).outcome(),
            Outcome::Ignored,
            "not laid out yet: nothing to scroll, the notch passes on"
        );
        render(&mut l);
        assert_eq!(l.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
        assert_eq!(l.wheel(Axis::Vertical, -1).outcome(), Outcome::Changed);
        assert_eq!(l.wheel(Axis::Vertical, -1).outcome(), Outcome::Consumed);

        let x = 39;
        assert_eq!(
            l.bar_press(Position::new(x, 0))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Consumed,
            "a press on the thumb"
        );
        assert_eq!(l.bar_drag(Position::new(x, 1)).outcome(), Outcome::Changed);
        let after_one = l.scroll.offset;
        assert!(after_one > 0 && after_one < 10, "{after_one}");
        assert_eq!(l.bar_drag(Position::new(x, 5)).outcome(), Outcome::Changed);
        assert_eq!(
            l.scroll.offset,
            l.scroll.max_offset(),
            "the end of the track is the end"
        );
        assert_eq!(
            l.bar_drag(Position::new(x, 40)).outcome(),
            Outcome::Consumed,
            "past the track clamps"
        );
        assert_eq!(l.bar_drag(Position::new(x, 0)).outcome(), Outcome::Changed);
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
        l.wheel(Axis::Vertical, 4).outcome();
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
        l.wheel(Axis::Vertical, -40).outcome();
        assert_eq!(l.scroll.offset, 0);
    }

    #[test]
    fn enter_on_a_focused_list_flashes_its_cursor_row() {
        let items = (0..3).map(|i| ListItem::new(&format!("Row {i}"))).collect();
        let mut l = ListBox::new(WidgetId::of("l"), items, SelectMode::Single);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();

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
    use ratatui::layout::Position;

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
        assert_eq!(
            l.bar_press(Position::new(19, 9))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert_eq!(l.scroll.offset, 10, "one page, not a jump to the end");
    }
}

#[cfg(test)]
mod select_all_tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers as M;

    fn items() -> Vec<ListItem> {
        vec![
            ListItem::new("a"),
            ListItem::new("b").disabled(true),
            ListItem::new("c"),
        ]
    }

    #[test]
    fn multi_selects_every_enabled_row_single_ignores_it() {
        for mods in [M::SUPER, M::CONTROL] {
            let key = Key {
                code: KeyCode::Char('a'),
                mods,
            };
            let mut multi = ListBox::new(WidgetId::of("m"), items(), SelectMode::Multi);
            assert_eq!(multi.on_key(&key), Outcome::Changed);
            assert_eq!(multi.selection.to_vec(), vec![0, 2]);
            assert_eq!(multi.on_key(&key), Outcome::Consumed, "already all");
            let mut single =
                ListBox::new(WidgetId::of("s"), items(), SelectMode::Single).type_ahead(true);
            assert_eq!(single.on_key(&key), Outcome::Ignored);
            assert_eq!(single.cursor, 0, "never a type-ahead `a`");
        }
    }
}
