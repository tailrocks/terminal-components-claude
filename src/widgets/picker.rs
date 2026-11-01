use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::edit::TextCmd;
use crate::core::event::{Key, Outcome};
use crate::core::hints::{HasHints, HintCx, nav_hints};
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Nav, NavKeys};
use crate::core::list::{Cursor, KeyStep, Navigable, RowSource};
use crate::theme::CursorStyle;
use crate::ui::columns::{self, Col};
use crate::ui::ctx::{Backdrop, RenderCtx, fill};
use crate::ui::edit_menu::{
    EditCaps, EditMenu, EditMenuHost, edit_menu_key, edit_menu_pointer, edit_menu_route,
    point_field,
};
use crate::ui::popup::{Placement, place_with};
use crate::ui::text::{truncate, wider_than, width};
use crate::widgets::edit::EditField;
use crate::widgets::keyhint::Hint;
use crate::widgets::option_list::{Item, OptionList};

pub const DEFAULT_GLYPH: &str = "·";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PickerStatus {
    #[default]
    Ready,

    Loading(String),

    Error {
        message: String,
        detail: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Picker {
    pub id: WidgetId,
    pub status: PickerStatus,
    pub title: String,
    pub placeholder: String,

    pub items: Vec<Item>,

    pub list: OptionList,

    pub anchor: Rect,

    pub hints: String,

    pub backdrop: Backdrop,

    pub scope: Option<String>,
    pub empty_text: String,

    strings: crate::theme::Strings,

    pub empty: Option<crate::widgets::empty::EmptyState>,
    pub area: Rect,

    pub reveal: Option<crate::ui::reveal::Reveal>,

    pub(crate) pending: Option<PickerAction>,

    pub delete_is_secondary: bool,

    pub choose_hint: std::borrow::Cow<'static, str>,

    pub edit_menu: EditMenu,
}

impl HasHints for Picker {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        let searchable = self.is_searchable();
        let mut v = vec![];
        if searchable {
            v.push(Hint::new("Type", "Filter"));
        }

        v.extend(
            nav_hints(&self.list.nav, searchable, false)
                .into_iter()
                .filter(|h| h.key != "Type"),
        );
        v.push(Hint::new("Enter", self.choose_hint.clone()));
        if !self.query().is_empty() {
            v.push(Hint::new("Esc", "Clear"));
        } else if self.list.dismiss.escape {
            v.push(Hint::new("Esc", self.strings.hint_cancel));
        }
        v
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerAction {
    QueryChanged,
    Chosen(usize),

    ChosenAlt(usize),

    Submit,

    Secondary(usize),

    NextScope,

    PrevScope,
    Cancelled,

    Back,
}

impl Picker {
    pub const MAX_ROWS: u16 = 12;

    pub const WIDTH: u16 = 64;

    pub fn new(id: WidgetId, title: &str) -> Self {
        let list = OptionList::new(id)
            .query(EditField::single(""))
            .max_rows(Self::MAX_ROWS)
            .width(Self::WIDTH, Self::WIDTH)
            .placement(Placement::Center)
            .hover_moves_cursor(false);
        Self {
            id,
            status: PickerStatus::Ready,
            title: title.to_owned(),
            placeholder: crate::theme::Strings::DEFAULT.search_placeholder.into(),
            items: vec![],
            list,
            anchor: Rect::ZERO,
            hints: String::new(),
            backdrop: Backdrop::DEFAULT,
            scope: None,
            empty_text: crate::theme::Strings::DEFAULT.no_matches.into(),
            strings: crate::theme::Strings::DEFAULT,
            empty: None,
            area: Rect::ZERO,
            reveal: None,
            pending: None,
            delete_is_secondary: true,
            choose_hint: std::borrow::Cow::Borrowed("Choose"),
            edit_menu: EditMenu::default(),
        }
    }

    pub fn choose_hint(mut self, action: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        self.choose_hint = action.into();
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.list.nav = nav;
        self
    }

    pub fn delete_is_secondary(mut self, on: bool) -> Self {
        self.delete_is_secondary = on;
        self
    }

    pub fn searchable(mut self, on: bool) -> Self {
        self.set_searchable(on);
        self
    }

    pub fn set_searchable(&mut self, on: bool) {
        match (on, self.list.query.is_some()) {
            (true, false) => self.list.query = Some(Box::new(EditField::single(""))),
            (false, true) => self.list.query = None,
            _ => {}
        }
    }

    pub fn is_searchable(&self) -> bool {
        self.list.query.is_some()
    }

    pub fn query(&self) -> &str {
        self.list.query.as_deref().map_or("", EditField::text)
    }

    pub fn set_query(&mut self, text: &str) {
        if let Some(q) = &mut self.list.query {
            q.set_text(text);
        }
    }

    pub fn caret(&self) -> usize {
        self.list
            .query
            .as_ref()
            .map_or(0, |q| q.buffer.cursor_offset())
    }

    pub fn width(mut self, width: u16) -> Self {
        self.list = self.list.width(width, width);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.list.placement = placement;
        self
    }

    pub fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = anchor;
        self.list.placement = placement;
        self
    }

    pub fn fit(mut self, fit: crate::ui::popup::Place) -> Self {
        self.list.fit = fit;
        self
    }

    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn hints(mut self, hints: &str) -> Self {
        self.hints = hints.to_owned();
        self
    }

    pub fn max_rows(mut self, rows: u16) -> Self {
        self.list.max_rows = rows;
        self
    }

    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = text.to_owned();
        self
    }

    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.empty = Some(state);
        self
    }

    pub fn empty_text(mut self, text: &str) -> Self {
        self.empty_text = text.to_owned();
        self
    }

    pub fn dismiss(mut self, dismiss: crate::ui::popup::Dismiss) -> Self {
        self.list.dismiss = dismiss;
        self
    }

    pub fn on_click_outside(&mut self) -> Option<PickerAction> {
        self.list
            .dismiss
            .outside_click
            .then_some(PickerAction::Cancelled)
    }

    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.list.look = cursor_style;
        self
    }

    pub fn height(&self) -> u16 {
        let max = self.list.max_rows;
        let rows = if self.status == PickerStatus::Ready {
            (self.items.len() as u16).clamp(1, max)
        } else {
            (self.items.len() as u16).clamp(2, max)
        };
        let query_rows = if self.is_searchable() { 2 } else { 0 };
        2 + 1 + query_rows + rows + 2
    }

    fn first_enabled(&self) -> usize {
        self.items.iter().position(|i| !i.disabled).unwrap_or(0)
    }

    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items;
        self.list.reset(self.items.len());
        self.list.cursor.index = self.first_enabled();
    }

    pub fn refresh_items(&mut self, items: Vec<Item>) {
        let keep = self
            .items
            .get(self.list.cursor.index)
            .map(|i| i.key)
            .filter(|k| !matches!(k, ItemKey::Index(_)));
        let offset = self.list.scroll.offset;
        self.items = items;
        self.list.cursor.index = keep
            .and_then(|k| self.items.iter().position(|i| i.key == k && !i.disabled))
            .unwrap_or_else(|| self.first_enabled());
        self.list.scroll.set_content(self.items.len());
        self.list.scroll.scroll_to(offset);
    }

    pub fn eligible(&self, i: usize) -> Option<usize> {
        if self.status != PickerStatus::Ready {
            return None;
        }
        self.items.get(i).filter(|it| !it.disabled).map(|_| i)
    }

    pub fn current(&self) -> Option<&Item> {
        self.eligible(self.list.cursor.index)
            .map(|i| &self.items[i])
    }

    pub fn current_key(&self) -> Option<ItemKey> {
        self.current()
            .map(|it| it.key)
            .filter(|k| !matches!(k, ItemKey::Index(_)))
    }

    pub fn on_paste(&mut self, text: &str) -> (Outcome, Option<PickerAction>) {
        if self.status == PickerStatus::Loading(String::new()) {
            return (Outcome::Consumed, None);
        }
        let Some(query) = &mut self.list.query else {
            return (Outcome::Consumed, None);
        };

        let flat = text.replace('\t', " ");
        if flat.is_empty() || !query.paste(&flat) {
            return (Outcome::Consumed, None);
        }
        (Outcome::Changed, Some(PickerAction::QueryChanged))
    }

    pub fn set_cursor(&mut self, i: usize) {
        self.list.cursor.index = i.min(self.items.len().saturating_sub(1));
        self.list.scroll.ensure_visible(self.list.cursor.index);
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.list.row_id(i)
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PickerAction>) {
        if self.list.query.is_some()
            && let Some(o) = edit_menu_key(self, key)
        {
            let ev = self
                .edit_menu
                .take_edited()
                .then_some(PickerAction::QueryChanged);
            return (o, ev);
        }

        let typing = self.is_searchable();

        if !typing
            && self.list.type_ahead
            && matches!(
                KeyStep::of(key, &self.list.nav, true),
                Some(KeyStep::Type(_))
            )
        {
            let moved = self.list.key(key, &self.items[..]);
            if moved.index().is_some() {
                let o = if moved.moved() {
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                };
                return (o, None);
            }
        }
        let nav = self.list.nav.action(key).filter(|n| {
            !(typing
                && (matches!(key.code, KeyCode::Char(_))
                    || matches!(n, Nav::Home | Nav::End | Nav::Left | Nav::Right)))
        });
        let nav = nav.filter(|n| key.plain() || !matches!(n, Nav::Home | Nav::End));
        let Some(nav) = nav else {
            return self.on_other_key(key);
        };
        match self.list.nav_step(nav, &self.items[..]).index() {
            Some(_) => (Outcome::Changed, None),
            None => self.on_other_key(key),
        }
    }

    fn on_other_key(&mut self, key: &Key) -> (Outcome, Option<PickerAction>) {
        let searchable = self.is_searchable();
        match key.code {
            KeyCode::Esc => {
                if let Some(q) = &mut self.list.query
                    && !q.text().is_empty()
                {
                    q.record(crate::core::edit::EditKind::Other);
                    q.buffer.set_text("");
                    return (Outcome::Changed, Some(PickerAction::QueryChanged));
                }
                if !self.list.dismiss.escape {
                    return (Outcome::Consumed, None);
                }
                (Outcome::Changed, Some(PickerAction::Cancelled))
            }
            KeyCode::Enter => match self.eligible(self.list.cursor.index) {
                Some(i) => {
                    let ev = if key.alt() {
                        PickerAction::ChosenAlt(i)
                    } else {
                        PickerAction::Chosen(i)
                    };
                    (Outcome::Changed, Some(ev))
                }
                None if searchable && !self.query().trim().is_empty() => {
                    (Outcome::Changed, Some(PickerAction::Submit))
                }
                None => (Outcome::Consumed, None),
            },
            KeyCode::Char('n') | KeyCode::Char('j') if key.ctrl() => {
                self.list.step(1, &self.items[..]);
                (Outcome::Changed, None)
            }
            KeyCode::Char('p') | KeyCode::Char('k') if key.ctrl() => {
                self.list.step(-1, &self.items[..]);
                (Outcome::Changed, None)
            }
            KeyCode::Tab => (Outcome::Changed, Some(PickerAction::NextScope)),
            KeyCode::BackTab => (Outcome::Changed, Some(PickerAction::PrevScope)),
            KeyCode::Delete if self.delete_is_secondary || !searchable => {
                match self.eligible(self.list.cursor.index) {
                    Some(i) => (Outcome::Changed, Some(PickerAction::Secondary(i))),
                    None => (Outcome::Consumed, None),
                }
            }
            KeyCode::Backspace if self.query().is_empty() => {
                (Outcome::Changed, Some(PickerAction::Back))
            }

            _ => match &mut self.list.query {
                Some(q) => {
                    let r = q.on_key(key);
                    if r.edited {
                        (Outcome::Changed, Some(PickerAction::QueryChanged))
                    } else if r.outcome == Outcome::Changed && r.exit.is_none() {
                        (Outcome::Changed, None)
                    } else {
                        (Outcome::Consumed, None)
                    }
                }
                None => (Outcome::Consumed, None),
            },
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> Option<PickerAction> {
        if id == crate::ui::popup::backdrop_id(self.id) {
            return self.on_click_outside();
        }
        let i = self.eligible(self.list.row_of(id, self.items.len())?)?;
        self.list.select(i, &self.items[..]);
        Some(PickerAction::Chosen(i))
    }

    pub fn draw(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let hints = std::mem::take(&mut self.hints);
        self.paint(
            screen,
            buf,
            ctx,
            &crate::widgets::keyhint::deliverable_line(&hints),
        );
        self.hints = hints;

        self.edit_menu.render(self.area, buf, ctx);
    }

    #[deprecated(note = "set `Picker::hints` and call `Picker::draw`")]
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, hints: &str) {
        self.paint(screen, buf, ctx, hints);
    }

    fn paint(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, hints: &str) {
        self.list.scroll.begin(ctx.track_click);
        let s = ctx.theme.strings;
        let old = std::mem::replace(&mut self.strings, s);
        use crate::theme::Strings;
        Strings::follow(
            &mut self.placeholder,
            old.search_placeholder,
            s.search_placeholder,
        );
        Strings::follow(&mut self.empty_text, old.no_matches, s.no_matches);
        let t = ctx.theme;

        ctx.modal_backdrop_with(buf, screen, self.backdrop);

        let modal = crate::ui::popup::Dismiss {
            pass_through: false,
            ..self.list.dismiss
        };
        crate::ui::popup::backdrop(ctx, self.id, modal, screen);
        let h = self.height().min(screen.height.saturating_sub(2));
        let w = self.list.width.1.min(screen.width.saturating_sub(4));
        let area = place_with(
            screen,
            self.anchor,
            w,
            h,
            self.list.placement,
            self.list.fit,
        );
        self.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(t.glyphs.border)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        ctx.hits.register(self.id, area);
        let inner = area.inner(ratatui::layout::Margin::new(2, 1));
        if inner.is_empty() {
            return;
        }
        let mut y = inner.y;
        buf.set_string(
            inner.x,
            y,
            truncate(&self.title, inner.width as usize),
            t.title().bg(bg),
        );
        if let Some(scope) = &self.scope {
            let room = inner.width.saturating_sub(width(&self.title) as u16 + 3) as usize;
            if room >= 6 {
                let scope = truncate(scope, room);
                let sw = width(&scope) as u16;
                buf.set_string(
                    inner.right().saturating_sub(sw),
                    y,
                    &scope,
                    t.muted().bg(bg),
                );
            }
        }
        y += 1;
        if let Some(query) = &mut self.list.query {
            let field = Rect::new(inner.x, y, inner.width, 1);
            let fstate = crate::ui::ctx::VisualState {
                focused: true,
                editing: true,
                ..Default::default()
            };
            let fs = t.field_style(fstate);
            fill(buf, field, fs);
            buf.set_string(
                field.x,
                y,
                t.gutter_symbol(fstate),
                t.gutter(fstate, fs.bg.unwrap_or(bg), false),
            );
            query.keys = ctx.theme.field_keys.unwrap_or_default();
            let w = field.width.saturating_sub(3) as usize;

            let line = Rect::new(field.x + 2, y, field.width.saturating_sub(2), 1);
            let caret = if query.text().is_empty() {
                buf.set_string(
                    line.x,
                    y,
                    truncate(&self.placeholder, w),
                    fs.fg(t.text_muted),
                );
                line.as_position()
            } else {
                let look = crate::widgets::edit::LineLook {
                    text: fs,
                    selection: t.selection(),
                    ellipsis: fs.fg(t.text_muted),
                    ellipsis_glyph: t.glyphs.ellipsis,
                    underline: Some(t.text_secondary),
                    follow_caret: true,
                };
                query.draw_line(buf, line, &look)
            };
            if line.width > 0 {
                let id = ctx.register_part(self.id, PartRef::of(Part::QUERY), line);
                ctx.pointer_shape(id, crate::core::shape::PointerShape::Text);
            }
            ctx.set_cursor(caret);
            y += 2;
        }

        let hints_fit = inner.bottom() > y + 1;
        let hints = if hints_fit { hints } else { "" };
        let list = Rect::new(
            inner.x,
            y,
            inner.width,
            inner.bottom().saturating_sub(y + u16::from(hints_fit)),
        );

        self.list.layout(list, &self.items[..], ctx);
        let hint_row = |buf: &mut Buffer| {
            if !hints.is_empty() {
                let hy = inner.bottom().saturating_sub(1);
                buf.set_string(
                    inner.x,
                    hy,
                    truncate(hints, inner.width as usize),
                    t.faint().bg(bg),
                );
            }
        };
        match &self.status {
            PickerStatus::Loading(label) => {
                crate::widgets::progress::render_spinner(
                    Rect::new(list.x + 1, list.y, list.width.saturating_sub(1), 1),
                    buf,
                    ctx,
                    label,
                    bg,
                );
                hint_row(buf);
                return;
            }
            PickerStatus::Error { message, detail } => {
                let mut e = crate::widgets::empty::EmptyState::error(message);
                if let Some(d) = detail {
                    e = e.hint(d);
                }
                crate::widgets::empty::render(list, buf, t, &e, bg);
                hint_row(buf);
                return;
            }
            PickerStatus::Ready => {}
        }
        if self.items.is_empty() {
            crate::widgets::empty::render(
                list,
                buf,
                t,
                &self
                    .empty
                    .clone()
                    .unwrap_or_else(|| crate::widgets::empty::EmptyState::new(&self.empty_text)),
                bg,
            );
        }
        let has_sb = self.list.scroll.overflows();
        let row_w = list.width.saturating_sub(u16::from(has_sb));

        let measure = |f: &dyn Fn(&Item) -> u16| self.items.iter().map(f).max().unwrap_or(0);
        let label_need = measure(&|i| width(&i.label) as u16).max(6);
        let detail_need = measure(&|i| width(&i.detail) as u16);
        let tag_col = measure(&|i| i.tag.as_deref().map_or(0, |t| width(t) as u16));
        let group_col = measure(&|i| width(&i.group) as u16);
        let label_cap = label_need.min((row_w * 45 / 100).max(6));
        let mut cols = vec![
            Col::content(label_cap.min(12), label_cap),
            Col::content(detail_need.min(4), detail_need)
                .flex(1, u16::MAX)
                .droppable(0),
        ];
        if tag_col > 0 {
            cols.push(Col::fixed(tag_col));
        }
        if group_col > 0 {
            cols.push(Col::fixed(group_col));
        }
        let widths = columns::solve(&cols, row_w.saturating_sub(4), 2);
        let label_col = widths[0];
        let detail_col = widths[1];
        let items = &self.items[..];
        let reveal = self.reveal;
        let mut last_group = "";
        self.list
            .paint_rows(list, bg, items, buf, ctx, |i, r, buf, ctx| {
                let it = &items[i];
                let (row, ry) = (r.area, r.area.y);
                let rid = self.list.row_id(i);
                let st = r.style();
                let focused = r.state.focused;

                let show_group = it.group != last_group && !it.group.is_empty();
                last_group = it.group.as_ref();
                let glyph: &str = if it.glyph.is_empty() || it.glyph == DEFAULT_GLYPH {
                    t.glyphs.bullet
                } else {
                    &it.glyph
                };
                buf.set_string(
                    row.x + 1,
                    ry,
                    glyph,
                    st.fg(if focused {
                        t.text_primary
                    } else {
                        r.ink(ctx, t.text_muted)
                    })
                    .remove_modifier(Modifier::BOLD),
                );

                let mut x = row.x + 3;
                let label = truncate(&it.label, label_col as usize);
                let label_cut = !it.disabled
                    && crate::ui::reveal::truncated(
                        ctx,
                        reveal,
                        rid,
                        Rect::new(row.x + 3, ry, label_col, 1),
                        &it.label,
                    );
                for (bi, ch) in label.char_indices() {
                    let mut cs = st;
                    if it.matched.contains(&bi) {
                        cs = cs.add_modifier(Modifier::BOLD);
                    } else if !focused {
                        cs = cs.remove_modifier(Modifier::BOLD);
                    }
                    let g = ch.to_string();
                    buf.set_string(x, ry, &g, cs);
                    x += width(&g) as u16;
                }
                let mut rx = row.right();
                if group_col > 0 {
                    rx = rx.saturating_sub(group_col + 1);
                    if show_group {
                        buf.set_string(
                            rx,
                            ry,
                            &it.group,
                            st.fg(r.ink(ctx, t.legible(t.text_muted, r.bg())))
                                .remove_modifier(Modifier::BOLD),
                        );
                    }
                }
                if tag_col > 0 {
                    rx = rx.saturating_sub(tag_col + 2);
                    if let Some(tag) = &it.tag {
                        buf.set_string(
                            rx,
                            ry,
                            tag,
                            st.fg(r.ink(ctx, t.text_secondary))
                                .remove_modifier(Modifier::BOLD),
                        );
                    }
                }
                let mut detail_cut = false;
                if !it.detail.is_empty() {
                    let dx = row.x + 3 + label_col + 2;
                    let room = (detail_col as usize).min(rx.saturating_sub(dx + 1) as usize);
                    if room >= 4 {
                        buf.set_string(
                            dx,
                            ry,
                            truncate(&it.detail, room),
                            st.fg(r.ink(ctx, t.text_muted))
                                .remove_modifier(Modifier::BOLD),
                        );
                        detail_cut = wider_than(&it.detail, room);
                        if !it.disabled && !label_cut {
                            crate::ui::reveal::truncated(
                                ctx,
                                reveal,
                                rid,
                                Rect::new(dx, ry, room as u16, 1),
                                &it.detail,
                            );
                        }
                    } else {
                        detail_cut = true;
                    }
                }

                if focused && !it.disabled && (label_cut || detail_cut) {
                    let full = if it.detail.is_empty() {
                        it.label.clone()
                    } else {
                        format!("{} — {}", it.label, it.detail)
                    };
                    crate::ui::reveal::peek(ctx, &full);
                }
            });
        hint_row(buf);
    }
}

impl RowSource for Picker {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn enabled(&self, i: usize) -> bool {
        self.items[..].enabled(i)
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].label
    }
    fn key(&self, i: usize) -> ItemKey {
        self.items[..].key(i)
    }
}

impl Navigable for Picker {
    fn cursor(&self) -> Cursor {
        self.list.cursor
    }
}

impl crate::core::pointer::MouseTarget for Picker {
    type Action = PickerAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<PickerAction> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;

        if let Some(o) = edit_menu_pointer(self, ev) {
            if self.edit_menu.take_edited() {
                return Response::emit(PickerAction::QueryChanged);
            }
            return o.into();
        }

        if ev.wheel().is_some() && ev.part().part == Part::BACKDROP {
            return Response::ignored();
        }
        if let Some(r) = crate::core::pointer::scroll_input(&mut self.list, ev) {
            return r.lift();
        }

        if let Some(query) = &mut self.list.query
            && let Some(r) = crate::widgets::option_list::query_pointer(query, ev)
        {
            return r.lift();
        }
        if !ev.is(Phase::Click) {
            return Response::ignored();
        }
        let chosen = if ev.part().part == Part::BACKDROP {
            self.on_click_outside()
        } else {
            match self.on_click(ev.id) {
                Some(PickerAction::Chosen(i))
                    if ev
                        .mods()
                        .contains(ratatui::crossterm::event::KeyModifiers::ALT) =>
                {
                    Some(PickerAction::ChosenAlt(i))
                }
                other => other,
            }
        };
        match chosen {
            Some(a) => Response::emit(a),
            None => Response::consumed(),
        }
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<PickerAction>> {
        edit_menu_route(self, ev)
    }
}

impl EditMenuHost for Picker {
    fn edit_menu(&mut self) -> &mut EditMenu {
        &mut self.edit_menu
    }
    fn edit_owner(&self) -> WidgetId {
        self.id
    }
    fn edit_caps(&self) -> EditCaps {
        self.list
            .query
            .as_ref()
            .map(|q| EditCaps::of(q))
            .unwrap_or_default()
    }
    fn edit_shortcut(&self, cmd: TextCmd) -> Option<String> {
        self.list.query.as_ref().and_then(|q| q.shortcut(cmd))
    }
    fn edit_target(&self, ev: &crate::core::pointer::PointerEvent) -> bool {
        self.list.query.is_some() && ev.part().part == Part::QUERY
    }
    fn edit_point(&mut self, pos: ratatui::layout::Position) {
        if let Some(q) = self.list.query.as_mut() {
            let at = q.line_offset_at(pos);
            point_field(q, at);
        }
    }
    fn edit_run(&mut self, cmd: TextCmd) -> bool {
        self.list.query.as_mut().is_some_and(|q| q.run(cmd).edited)
    }
}
