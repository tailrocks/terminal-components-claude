//! Searchable picker: a centered modal with a query field and grouped,
//! ranked rows. Used for quick switchers, tab lists and enum pickers.
//! Ranking is the owner's job (it supplies the rows for the query).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::hints::{HasHints, HintCx, nav_hints};
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Nav, NavKeys};
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::core::scroll::{ScrollPane, ScrollState, Scrollable};
use crate::theme::CursorStyle;
use crate::ui::columns::{self, Col};
use crate::ui::ctx::{Backdrop, RenderCtx, fill};
use crate::ui::popup::{Placement, place_with};
use crate::ui::text::{truncate, width};
use crate::widgets::edit::EditField;
use crate::widgets::keyhint::Hint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerItem {
    pub label: String,
    pub detail: String,
    pub glyph: &'static str,
    pub group: &'static str,
    /// Trailing hint on the row (e.g. "open").
    pub tag: Option<&'static str>,
    pub matched: Vec<usize>,
    pub disabled: bool,
    /// Owner identity the row stands for (a tab, an activity, a path).
    /// Stable across refreshes; empty when the owner keys by index.
    pub key: String,
}

impl PickerItem {
    /// The glyph of a new item; it draws as
    /// [`Glyphs::bullet`](crate::theme::Glyphs::bullet).
    pub const DEFAULT_GLYPH: &'static str = "·";

    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: String::new(),
            glyph: Self::DEFAULT_GLYPH,
            group: "",
            tag: None,
            matched: vec![],
            disabled: false,
            key: String::new(),
        }
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = d.into();
        self
    }
    pub fn glyph(mut self, g: &'static str) -> Self {
        self.glyph = g;
        self
    }
    pub fn group(mut self, g: &'static str) -> Self {
        self.group = g;
        self
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = k.into();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PickerStatus {
    #[default]
    Ready,
    /// Spinner row in the list area; Enter is refused.
    Loading(String),
    /// `! message` with an optional faint detail; Enter is refused.
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
    /// The query field: its text, caret, selection, undo and keys are the
    /// shared [`EditField`]'s, so it edits, selects (Select All, a double
    /// click on a word, a drag) and undoes like every text field. Owners
    /// read [`EditField::text`] and replace it with [`EditField::set_text`]
    /// (the caret goes to the end).
    pub query: EditField,
    pub items: Vec<PickerItem>,
    pub cursor: usize,
    /// The rows and the bar ([`Scrollable`]).
    pub scroll: ScrollPane,
    pub width: u16,
    pub max_rows: u16,
    /// Where the picker sits and how it keeps off the screen edges.
    /// Default [`Place::DEFAULT`](crate::ui::popup::Place): centred like a
    /// dialog, one free cell from every edge.
    pub fit: crate::ui::popup::Place,
    /// Where the picker opens relative to [`Self::anchor`]. Default
    /// [`Placement::Center`] (the quick-switcher position, ignoring the
    /// anchor); an anchored placement turns it into a drop-down under (or
    /// beside) a control.
    pub placement: Placement,
    /// The rectangle an anchored [`Self::placement`] opens next to.
    /// Default `Rect::ZERO`.
    pub anchor: Rect,
    /// The hint row at the bottom of the picker (e.g. "↵ open · esc
    /// close"). Default empty: no hint row, for owners with a shell-level
    /// hint bar.
    pub hints: String,
    /// Which screen rows the dimmed backdrop leaves live. Default
    /// [`Backdrop::DEFAULT`] (the footer row).
    pub backdrop: Backdrop,
    /// Optional right-aligned scope label in the query row.
    pub scope: Option<String>,
    pub empty_text: String,
    /// The theme's strings as of the last render: default text still
    /// reading them follows a change ([`crate::theme::Strings::follow`]).
    strings: crate::theme::Strings,
    /// A full empty state (title, hint, error tone) shown instead of
    /// [`Self::empty_text`] when set. Default `None`.
    pub empty: Option<crate::widgets::empty::EmptyState>,
    pub area: Rect,
    /// Show the query field (false for fixed-choice pickers).
    pub searchable: bool,
    /// How the cursor row is marked. Default [`CursorStyle::Fill`]: the
    /// query field that owns the list keeps the only focus bar.
    pub cursor_style: CursorStyle,
    /// The cursor moved since the last render: the next render pulls it
    /// into view. Wheel scrolling leaves it alone so the viewport and the
    /// selection never fight.
    cursor_dirty: bool,
    /// How a cut label or detail reveals its full text on hover (see
    /// [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// Which dismissals close the picker without choosing. Default
    /// [`Dismiss::LIGHT`](crate::ui::popup::Dismiss::LIGHT): Esc and a click outside both cancel.
    pub dismiss: crate::ui::popup::Dismiss,
    /// The last action, kept for the stack while the picker is a
    /// [`crate::ui::layer::Layer`].
    pub(crate) pending: Option<PickerAction>,
    /// Delete on the query emits [`PickerAction::Secondary`] for the
    /// cursor row (close a tab, forget an entry) instead of deleting
    /// forward. Default `true` (today's binding); off, Delete deletes the
    /// character after the caret like a search field.
    pub delete_is_secondary: bool,
    /// Which keys move the list cursor (default [`NavKeys::default`]). A
    /// searchable picker types its letters and edits the query with
    /// ←/→ and Home/End, so there only ↑/↓ and paging move; a plain list
    /// takes the whole table.
    pub nav: NavKeys,
    /// What the footer says Enter does on a row ([`HasHints`]). Default
    /// `Choose`; a tab switcher says `Switch`.
    pub choose_hint: std::borrow::Cow<'static, str>,
}

/// The footer while the picker is on top: `Type Filter` when it searches,
/// how the cursor moves (from [`Picker::nav`]), Enter
/// ([`Picker::choose_hint`]), and Esc as it acts now (`Clear` while a
/// query is typed, `Cancel` when the dismiss policy lets Esc close it).
impl HasHints for Picker {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        let mut v = vec![];
        if self.searchable {
            v.push(Hint::new("Type", "Filter"));
        }
        // a searchable picker's letters edit the query
        v.extend(nav_hints(&self.nav, self.searchable, false).into_iter().filter(|h| h.key != "Type"));
        v.push(Hint::new("Enter", self.choose_hint.clone()));
        if self.searchable && !self.query.is_empty() {
            v.push(Hint::new("Esc", "Clear"));
        } else if self.dismiss.escape {
            v.push(Hint::new("Esc", self.strings.hint_cancel));
        }
        v
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerAction {
    /// Query text changed; the owner re-supplies items.
    QueryChanged,
    Chosen(usize),
    /// Chosen with the alternate modifier (e.g. open in new tab).
    ChosenAlt(usize),
    /// Enter with a query but no eligible row: the owner may act on the
    /// query text itself (a path typed into a jump picker).
    Submit,
    /// Secondary action on the cursor row (e.g. close tab).
    Secondary(usize),
    /// Tab: the owner moves to the next scope and re-supplies items.
    NextScope,
    /// Shift+Tab: the owner moves to the previous scope (K31).
    PrevScope,
    Cancelled,
    /// Backspace on an empty query: the owner rewinds one step.
    Back,
}

impl Picker {
    pub fn new(id: WidgetId, title: &str) -> Self {
        Self {
            id,
            status: PickerStatus::Ready,
            title: title.to_owned(),
            placeholder: crate::theme::Strings::DEFAULT.search_placeholder.into(),
            query: EditField::single(""),
            items: vec![],
            cursor: 0,
            scroll: ScrollPane::default(),
            width: 64,
            max_rows: 12,
            fit: crate::ui::popup::Place::DEFAULT,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            hints: String::new(),
            backdrop: Backdrop::DEFAULT,
            scope: None,
            empty_text: crate::theme::Strings::DEFAULT.no_matches.into(),
            strings: crate::theme::Strings::DEFAULT,
            empty: None,
            area: Rect::ZERO,
            searchable: true,
            cursor_style: CursorStyle::default(),
            cursor_dirty: true,
            reveal: None,
            dismiss: crate::ui::popup::Dismiss::LIGHT,
            pending: None,
            delete_is_secondary: true,
            nav: NavKeys::default(),
            choose_hint: std::borrow::Cow::Borrowed("Choose"),
        }
    }

    /// See [`Self::choose_hint`].
    pub fn choose_hint(mut self, action: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        self.choose_hint = action.into();
        self
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn delete_is_secondary(mut self, on: bool) -> Self {
        self.delete_is_secondary = on;
        self
    }

    /// The query caret as a byte offset.
    pub fn caret(&self) -> usize {
        self.query.buffer.cursor_offset()
    }

    /// Width in cells before clamping to the screen. Default 64.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// See [`Self::placement`].
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Open next to `anchor` per `placement` (see [`Self::anchor`]).
    pub fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = anchor;
        self.placement = placement;
        self
    }

    /// See [`Self::fit`].
    pub fn fit(mut self, fit: crate::ui::popup::Place) -> Self {
        self.fit = fit;
        self
    }

    /// See [`Self::backdrop`].
    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// See [`Self::hints`].
    pub fn hints(mut self, hints: &str) -> Self {
        self.hints = hints.to_owned();
        self
    }

    /// Most rows shown before the list scrolls. Default 12.
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows;
        self
    }

    /// The query field's placeholder.
    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = text.to_owned();
        self
    }

    /// See [`Self::empty`].
    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.empty = Some(state);
        self
    }

    /// What an empty result says.
    pub fn empty_text(mut self, text: &str) -> Self {
        self.empty_text = text.to_owned();
        self
    }

    pub fn dismiss(mut self, dismiss: crate::ui::popup::Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// A click outside the picker: cancels it when its [`Self::dismiss`]
    /// policy allows.
    pub fn on_click_outside(&mut self) -> Option<PickerAction> {
        self.dismiss
            .outside_click
            .then_some(PickerAction::Cancelled)
    }

    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = cursor_style;
        self
    }

    /// The frame's height in rows before it is fitted to a screen: the
    /// title, the search field (when searchable), the rows up to
    /// [`Self::max_rows`] (two for a loading or failed status) and the
    /// hint row.
    pub fn height(&self) -> u16 {
        let rows = if self.status == PickerStatus::Ready {
            (self.items.len() as u16).clamp(1, self.max_rows)
        } else {
            (self.items.len() as u16).clamp(2, self.max_rows)
        };
        let query_rows = if self.searchable { 2 } else { 0 };
        2 + 1 + query_rows + rows + 2
    }

    /// Replace the rows for a new query: the cursor deliberately returns to
    /// the first eligible row.
    pub fn set_items(&mut self, items: Vec<PickerItem>) {
        self.items = items;
        self.cursor = self.items.iter().position(|i| !i.disabled).unwrap_or(0);
        self.scroll.v = ScrollState::new(self.items.len());
        self.cursor_dirty = true;
    }

    /// Refresh the rows while the picker stays open: the cursor follows the
    /// row's `key`, so an insertion or removal above it never retargets the
    /// selection. A vanished key falls to the first eligible row.
    pub fn refresh_items(&mut self, items: Vec<PickerItem>) {
        let keep = self
            .items
            .get(self.cursor)
            .filter(|i| !i.key.is_empty())
            .map(|i| i.key.clone());
        let offset = self.scroll.offset;
        self.items = items;
        self.cursor = keep
            .and_then(|k| self.items.iter().position(|i| i.key == k && !i.disabled))
            .unwrap_or_else(|| self.items.iter().position(|i| !i.disabled).unwrap_or(0));
        self.scroll.set_content(self.items.len());
        self.scroll.scroll_to(offset);
        self.cursor_dirty = true;
    }

    /// The row an action may target: only a real, enabled row of a ready
    /// picker. Every path (Enter, Delete, click) resolves through this.
    pub fn eligible(&self, i: usize) -> Option<usize> {
        if self.status != PickerStatus::Ready {
            return None;
        }
        self.items.get(i).filter(|it| !it.disabled).map(|_| i)
    }

    /// The key of the eligible cursor row, for owners that map identity.
    pub fn current_key(&self) -> Option<&str> {
        self.eligible(self.cursor)
            .map(|i| self.items[i].key.as_str())
            .filter(|k| !k.is_empty())
    }

    /// Pasted text enters the query as one edit: line breaks become spaces
    /// and exactly one `QueryChanged` is emitted.
    pub fn on_paste(&mut self, text: &str) -> (Outcome, Option<PickerAction>) {
        if !self.searchable || self.status == PickerStatus::Loading(String::new()) {
            return (Outcome::Consumed, None);
        }
        // a tab is a space in a one-line query; line breaks flatten in
        // the field ([`EditField::paste`])
        let flat = text.replace('\t', " ");
        if flat.is_empty() || !self.query.paste(&flat) {
            return (Outcome::Consumed, None);
        }
        (Outcome::Changed, Some(PickerAction::QueryChanged))
    }

    /// Move the cursor and ask the next render to keep it in view.
    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.items.len().saturating_sub(1));
        self.cursor_dirty = true;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::row_part(i))
    }

    fn row_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    /// A keyboard move: the next render keeps the cursor in view.
    fn move_to(&mut self, i: usize) {
        self.cursor = i;
        self.cursor_dirty = true;
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<PickerAction>) {
        // a searchable picker types its letters, and its Home/End and ←/→
        // edit the query; the list keeps ↑/↓ and paging
        let typing = self.searchable;
        let nav = self.nav.action(key).filter(|n| {
            !(typing
                && (matches!(key.code, KeyCode::Char(_))
                    || matches!(n, Nav::Home | Nav::End | Nav::Left | Nav::Right)))
        });
        let nav = nav.filter(|n| key.plain() || !matches!(n, Nav::Home | Nav::End));
        let Some(nav) = nav else {
            return self.on_other_key(key);
        };
        match self
            .cursor()
            .apply(nav, self, self.scroll.viewport_len)
            .index()
        {
            Some(i) => {
                self.move_to(i);
                (Outcome::Changed, None)
            }
            None => self.on_other_key(key),
        }
    }

    fn on_other_key(&mut self, key: &Key) -> (Outcome, Option<PickerAction>) {
        match key.code {
            KeyCode::Esc => {
                if self.searchable && !self.query.text().is_empty() {
                    self.query.record(crate::core::edit::EditKind::Other);
                    self.query.buffer.set_text("");
                    return (Outcome::Changed, Some(PickerAction::QueryChanged));
                }
                if !self.dismiss.escape {
                    return (Outcome::Consumed, None);
                }
                (Outcome::Changed, Some(PickerAction::Cancelled))
            }
            KeyCode::Enter => match self.eligible(self.cursor) {
                Some(i) => {
                    let ev = if key.alt() {
                        PickerAction::ChosenAlt(i)
                    } else {
                        PickerAction::Chosen(i)
                    };
                    (Outcome::Changed, Some(ev))
                }
                None if self.searchable && !self.query.text().trim().is_empty() => {
                    (Outcome::Changed, Some(PickerAction::Submit))
                }
                None => (Outcome::Consumed, None),
            },
            KeyCode::Char('n') | KeyCode::Char('j') if key.ctrl() => {
                if let Some(i) = self.cursor().step(1, self).index() {
                    self.move_to(i);
                }
                (Outcome::Changed, None)
            }
            KeyCode::Char('p') | KeyCode::Char('k') if key.ctrl() => {
                if let Some(i) = self.cursor().step(-1, self).index() {
                    self.move_to(i);
                }
                (Outcome::Changed, None)
            }
            KeyCode::Tab => (Outcome::Changed, Some(PickerAction::NextScope)),
            KeyCode::BackTab => (Outcome::Changed, Some(PickerAction::PrevScope)),
            KeyCode::Delete if self.delete_is_secondary || !self.searchable => {
                match self.eligible(self.cursor) {
                    Some(i) => (Outcome::Changed, Some(PickerAction::Secondary(i))),
                    None => (Outcome::Consumed, None),
                }
            }
            KeyCode::Backspace if self.query.text().is_empty() || !self.searchable => {
                (Outcome::Changed, Some(PickerAction::Back))
            }
            // everything else edits the query: typing, the caret keys,
            // Select All, the clipboard keys, undo
            _ if self.searchable => {
                let r = self.query.on_key(key);
                if r.edited {
                    (Outcome::Changed, Some(PickerAction::QueryChanged))
                } else if r.outcome == Outcome::Changed && r.exit.is_none() {
                    (Outcome::Changed, None)
                } else {
                    // the picker is modal: an unused key or chord stops here
                    (Outcome::Consumed, None)
                }
            }
            _ => (Outcome::Consumed, None),
        }
    }

    /// The row `id` is (the inverse [`Self::on_click`] needs until S15
    /// hands it the routed part).
    fn row_of(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.items.len())
    }

    pub fn on_click(&mut self, id: WidgetId) -> Option<PickerAction> {
        // a click outside lands on the picker's backdrop
        if id == crate::ui::popup::backdrop_id(self.id) {
            return self.on_click_outside();
        }
        let i = self.eligible(self.row_of(id)?)?;
        self.cursor = i;
        self.cursor_dirty = true;
        self.scroll.pointer_move();
        Some(PickerAction::Chosen(i))
    }

    /// Dim `screen` and draw the picker with its [`Self::hints`] row.
    pub fn draw(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let hints = std::mem::take(&mut self.hints);
        self.paint(screen, buf, ctx, &hints);
        self.hints = hints;
    }

    /// Draw with `hints` for this frame only.
    #[deprecated(note = "set `Picker::hints` and call `Picker::draw`")]
    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, hints: &str) {
        self.paint(screen, buf, ctx, hints);
    }

    fn paint(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, hints: &str) {
        self.scroll.begin(ctx.track_click);
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
        // dim + modal like Dialog
        ctx.modal_backdrop_with(buf, screen, self.backdrop);
        // light dismiss is the picker's: a click outside it lands on its
        // backdrop (under the modal barrier), resolved to the picker's
        // `Part::BACKDROP`. Modal: nothing beneath is reachable, so the
        // click never passes through.
        let modal = crate::ui::popup::Dismiss {
            pass_through: false,
            ..self.dismiss
        };
        crate::ui::popup::backdrop(ctx, self.id, modal, screen);
        let h = self.height().min(screen.height.saturating_sub(2));
        let w = self.width.min(screen.width.saturating_sub(4));
        let area = place_with(screen, self.anchor, w, h, self.placement, self.fit);
        let _ = Constraint::Length(0);
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
            // the scope never overwrites the title: it takes the room that is left
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
        if self.searchable {
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
            self.query.keys = ctx.theme.field_keys.unwrap_or_default();
            let w = field.width.saturating_sub(3) as usize;
            // the text and the cell after it, where the caret can sit
            let line = Rect::new(field.x + 2, y, field.width.saturating_sub(2), 1);
            let caret = if self.query.text().is_empty() {
                buf.set_string(
                    line.x,
                    y,
                    truncate(&self.placeholder, w),
                    fs.fg(t.text_muted),
                );
                line.as_position()
            } else {
                // a neutral underline: the field's bar is the one green cue;
                // a long query scrolls so its caret stays in view (M21)
                let look = crate::widgets::edit::LineLook {
                    text: fs,
                    selection: t.selection(),
                    ellipsis: fs.fg(t.text_muted),
                    ellipsis_glyph: t.glyphs.ellipsis,
                    underline: Some(t.text_secondary),
                    follow_caret: true,
                };
                self.query.draw_line(buf, line, &look)
            };
            if line.width > 0 {
                // the whole field row past the gutter is the text target
                let id = ctx.register_part(self.id, PartRef::of(Part::QUERY), line);
                ctx.pointer_shape(id, crate::core::shape::PointerShape::Text);
            }
            ctx.set_cursor(caret);
            y += 2;
        }
        // the hint row yields before the list does: a short screen still
        // shows at least one row of what Enter would choose
        let hints_fit = inner.bottom() > y + 1;
        let hints = if hints_fit { hints } else { "" };
        let list = Rect::new(
            inner.x,
            y,
            inner.width,
            inner.bottom().saturating_sub(y + u16::from(hints_fit)),
        );
        self.scroll.set_content(self.items.len());
        self.scroll.set_viewport(list.height as usize);
        // only a cursor move pulls the viewport; a wheel scroll must survive
        if self.cursor_dirty {
            self.scroll.ensure_visible(self.cursor);
            self.cursor_dirty = false;
        }
        match &self.status {
            PickerStatus::Loading(label) => {
                crate::widgets::progress::render_spinner(
                    Rect::new(list.x + 1, list.y, list.width.saturating_sub(1), 1),
                    buf,
                    ctx,
                    label,
                    bg,
                );
                // hints row still applies
                if !hints.is_empty() {
                    let hy = inner.bottom().saturating_sub(1);
                    buf.set_string(
                        inner.x,
                        hy,
                        truncate(hints, inner.width as usize),
                        t.faint().bg(bg),
                    );
                }
                return;
            }
            PickerStatus::Error { message, detail } => {
                let mut e = crate::widgets::empty::EmptyState::error(message);
                if let Some(d) = detail {
                    e = e.hint(d);
                }
                crate::widgets::empty::render(list, buf, t, &e, bg);
                if !hints.is_empty() {
                    let hy = inner.bottom().saturating_sub(1);
                    buf.set_string(
                        inner.x,
                        hy,
                        truncate(hints, inner.width as usize),
                        t.faint().bg(bg),
                    );
                }
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
        let has_sb = self.scroll.overflows();
        let row_w = list.width.saturating_sub(u16::from(has_sb));
        // column widths come from every item, not just the visible ones,
        // so scrolling never shifts the columns; one solver shares the row
        // between label and detail by what they measure, so neither is cut
        // beside cells the other does not use
        let measure = |f: &dyn Fn(&PickerItem) -> u16| self.items.iter().map(f).max().unwrap_or(0);
        let label_need = measure(&|i| width(&i.label) as u16).max(6);
        let detail_need = measure(&|i| width(&i.detail) as u16);
        let tag_col = measure(&|i| i.tag.map_or(0, |t| width(t) as u16));
        let group_col = measure(&|i| width(i.group) as u16);
        let label_cap = label_need.min((row_w * 45 / 100).max(6));
        let mut cols = vec![
            Col::content(label_cap.min(12), label_cap),
            // the detail gives way first when the row is short
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
        let mut last_group = "";
        for (k, i) in self.scroll.visible_range().enumerate() {
            let ry = list.y + k as u16;
            let it = &self.items[i];
            let rid = self.row_id(i);
            let mut s = ctx.state(rid);
            s.focused = i == self.cursor;
            s.disabled = it.disabled;
            let cr = t.cursor_row(s, bg, self.cursor_style);
            let st = cr.style;
            let row = Rect::new(list.x, ry, list.width.saturating_sub(u16::from(has_sb)), 1);
            fill(buf, row, st);
            buf.set_string(row.x, ry, cr.gutter, cr.gutter_style);
            // group label inline (first row of a group shows it right-aligned muted)
            let show_group = it.group != last_group && !it.group.is_empty();
            last_group = it.group;
            buf.set_string(
                row.x + 1,
                ry,
                if it.glyph == PickerItem::DEFAULT_GLYPH {
                    t.glyphs.bullet
                } else {
                    it.glyph
                },
                st.fg(if s.focused {
                    t.text_primary
                } else {
                    t.row_ink(&cr, t.text_muted)
                })
                .remove_modifier(Modifier::BOLD),
            );
            // fixed columns: label · detail · tag · group, so rows line up
            let mut x = row.x + 3;
            let label = truncate(&it.label, label_col as usize);
            let label_cut = !it.disabled
                && crate::ui::reveal::truncated(
                    ctx,
                    self.reveal,
                    rid,
                    Rect::new(row.x + 3, ry, label_col, 1),
                    &it.label,
                );
            for (bi, ch) in label.char_indices() {
                let mut cs = st;
                if it.matched.contains(&bi) {
                    cs = cs.add_modifier(Modifier::BOLD);
                } else if !s.focused {
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
                        it.group,
                        st.fg(t.row_ink(&cr, t.legible(t.text_muted, st.bg.unwrap_or(bg))))
                            .remove_modifier(Modifier::BOLD),
                    );
                }
            }
            if tag_col > 0 {
                rx = rx.saturating_sub(tag_col + 2);
                if let Some(tag) = it.tag {
                    buf.set_string(
                        rx,
                        ry,
                        tag,
                        st.fg(t.row_ink(&cr, t.text_secondary))
                            .remove_modifier(Modifier::BOLD),
                    );
                }
            }
            if !it.detail.is_empty() {
                let dx = row.x + 3 + label_col + 2;
                let room = (detail_col as usize).min(rx.saturating_sub(dx + 1) as usize);
                if room >= 4 {
                    buf.set_string(
                        dx,
                        ry,
                        truncate(&it.detail, room),
                        st.fg(t.row_ink(&cr, t.text_muted))
                            .remove_modifier(Modifier::BOLD),
                    );
                    if !it.disabled && !label_cut {
                        crate::ui::reveal::truncated(
                            ctx,
                            self.reveal,
                            rid,
                            Rect::new(dx, ry, room as u16, 1),
                            &it.detail,
                        );
                    }
                }
            }
            if !it.disabled {
                ctx.register_part(self.id, Self::row_part(i), row);
            }
        }
        if has_sb {
            self.scroll
                .draw_with_fade(list, buf, ctx, self.id, true, &[]);
        }
        // hints row: owners with a shell-level hint bar pass an empty string
        if !hints.is_empty() {
            let hy = inner.bottom().saturating_sub(1);
            buf.set_string(
                inner.x,
                hy,
                truncate(hints, inner.width as usize),
                t.faint().bg(bg),
            );
        }
    }
}

impl RowSource for Picker {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn enabled(&self, i: usize) -> bool {
        !self.items[i].disabled
    }
    fn label(&self, i: usize) -> &str {
        &self.items[i].label
    }
}

impl Navigable for Picker {
    fn cursor(&self) -> Cursor {
        Cursor::new(self.cursor)
    }
}

impl Scrollable for Picker {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.scroll
    }
}

/// The picker's pointer gestures: a wheel or a bar press scrolls its
/// list; a completed click on a row chooses it ([`Picker::on_click`]); a
/// click outside it lands on its backdrop and cancels it when its
/// [`Picker::dismiss`] policy allows ([`Picker::on_click_outside`]; a
/// refused one is consumed); any other click on it is consumed.
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
        // the wheel outside the picker (its backdrop) moves nothing
        if ev.wheel().is_some() && ev.part().part == Part::BACKDROP {
            return Response::ignored();
        }
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r.lift();
        }
        // the query field takes the text gestures (M21): a click places
        // the caret, a double click selects a word, a triple click the
        // query, a drag selects what it crosses
        if self.searchable && ev.part().part == Part::QUERY {
            let at = self.query.line_offset_at(ev.pos());
            match ev.phase() {
                Some(Phase::Press) => {
                    self.query.press(at, ev.count(), false);
                }
                Some(Phase::Drag) => {
                    self.query.drag(at);
                }
                Some(Phase::Click) => self.query.click(at, ev.count().max(1), false),
                _ => return Response::ignored(),
            }
            return Response::changed();
        }
        if !ev.is(Phase::Click) {
            return Response::ignored();
        }
        let chosen = if ev.part().part == Part::BACKDROP {
            self.on_click_outside()
        } else {
            match self.on_click(ev.id) {
                // Alt (Option) picks the alternative, as Alt+Enter does
                // (A24); a terminal that keeps Option-click leaves the key
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::Axis;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn the_query_caret_moves_and_edits_in_place() {
        let k = |code, mods| Key { code, mods };
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut p = picker(3);
        for c in "ordrs".chars() {
            p.on_key(&k(KeyCode::Char(c), M::NONE));
        }
        p.on_key(&k(KeyCode::Left, M::NONE));
        p.on_key(&k(KeyCode::Left, M::NONE));
        p.on_key(&k(KeyCode::Char('e'), M::NONE));
        assert_eq!(p.query.text(), "orders");
        assert_eq!(p.caret(), 4);
        p.on_key(&k(KeyCode::Backspace, M::NONE));
        assert_eq!(p.query.text(), "ordrs");
        p.on_key(&k(KeyCode::Home, M::NONE));
        assert_eq!(p.caret(), 0);
        // Delete stays the secondary action by default
        assert!(matches!(
            p.on_key(&k(KeyCode::Delete, M::NONE)).1,
            Some(PickerAction::Secondary(_)) | None
        ));
        assert_eq!(p.query.text(), "ordrs");
        let mut f = picker(3).delete_is_secondary(false);
        f.query.set_text("xyz");
        f.on_key(&k(KeyCode::Home, M::NONE));
        f.on_key(&k(KeyCode::Delete, M::NONE));
        assert_eq!(f.query.text(), "yz");
    }

    #[test]
    fn dismissal_follows_the_picker_policy() {
        use crate::ui::popup::Dismiss;
        let esc = Key {
            code: KeyCode::Esc,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut p = picker(3);
        assert_eq!(p.on_key(&esc).1, Some(PickerAction::Cancelled));
        assert_eq!(p.on_click_outside(), Some(PickerAction::Cancelled));
        let mut form = picker(3).dismiss(Dismiss::ESCAPE);
        assert_eq!(form.on_click_outside(), None);
        assert_eq!(form.on_key(&esc).1, Some(PickerAction::Cancelled));
        let mut never = picker(3).dismiss(Dismiss::NEVER);
        assert_eq!(never.on_key(&esc), (Outcome::Consumed, None));
    }

    fn picker(n: usize) -> Picker {
        let mut p = Picker::new(WidgetId::of("p"), "Palette");
        p.max_rows = 6;
        p.set_items(
            (0..n)
                .map(|i| PickerItem {
                    label: format!("Item {i:02}"),
                    detail: String::new(),
                    glyph: "·",
                    group: "",
                    tag: None,
                    matched: vec![],
                    disabled: false,
                    key: format!("k{i}"),
                })
                .collect(),
        );
        p
    }

    fn k(code: KeyCode, mods: ratatui::crossterm::event::KeyModifiers) -> Key {
        Key { code, mods }
    }

    /// M21 (S11): the query is the shared text field. A click places the
    /// caret, a double click selects the word, a drag selects, through the
    /// router (a click did nothing); Cmd+A selects the query (it was
    /// typed or dropped); a long query scrolls so the caret never sits on
    /// the ellipsis (it was cut with `…` and the caret clamped onto it).
    #[test]
    fn the_query_takes_clicks_word_selection_select_all_and_scrolls() {
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        use ratatui::crossterm::event::KeyModifiers as M;
        use ratatui::layout::Position;
        let mut p = picker(3);
        p.query.set_text("orders by date");
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(screen);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            p.draw(screen, &mut buf, &mut ctx);
        }
        let q = p.query.line_area();
        let ev = |phase, x: u16, count| {
            let pos = Position::new(x, q.y);
            let id = hits.hit(pos).expect("the query is a hit target");
            PointerEvent::new(&hits, phase, id, pos, count, M::NONE)
        };
        let _ = p.pointer(&ev(Phase::Press, q.x + 2, 1));
        let _ = p.pointer(&ev(Phase::Click, q.x + 2, 1));
        assert_eq!(p.caret(), 2, "a click places the caret");
        let _ = p.pointer(&ev(Phase::Press, q.x + 8, 2));
        let _ = p.pointer(&ev(Phase::Click, q.x + 8, 2));
        assert_eq!(p.query.buffer.selected_text(), Some("by"), "double click");
        let _ = p.pointer(&ev(Phase::Press, q.x, 1));
        let _ = p.pointer(&ev(Phase::Drag, q.x + 6, 1));
        assert_eq!(p.query.buffer.selected_text(), Some("orders"), "a drag");
        p.query.buffer.clear_selection();
        p.on_key(&k(KeyCode::Char('a'), M::SUPER));
        assert_eq!(p.query.buffer.selected_text(), Some("orders by date"));
        assert_eq!(
            p.on_key(&k(KeyCode::Char('x'), M::NONE)).1,
            Some(PickerAction::QueryChanged),
            "typing replaces the selection"
        );
        assert_eq!(p.query.text(), "x");
        // a query longer than the field keeps its caret on a character
        p.query.set_text("long ".repeat(30));
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(screen);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        p.draw(screen, &mut buf, &mut ctx);
        let caret = ctx.cursor.expect("the query caret");
        assert!(p.query.line_area().contains(caret));
        assert_ne!(buf[(caret.x, caret.y)].symbol(), "…");
        assert_ne!(buf[(caret.x.saturating_sub(1), caret.y)].symbol(), "…");
    }

    #[test]
    fn home_end_and_paging_never_land_on_a_disabled_end_row() {
        // duplication-widgets P0: `step(±n)` clamped onto a disabled end row
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut p = picker(5);
        // Home/End edit a searchable picker's query; a plain one moves
        p.searchable = false;
        p.items[0].disabled = true;
        p.items[4].disabled = true;
        p.cursor = 2;
        p.on_key(&k(KeyCode::Home, M::NONE));
        assert_eq!(p.cursor, 1, "Home stops on the first enabled row");
        p.on_key(&k(KeyCode::End, M::NONE));
        assert_eq!(p.cursor, 3, "End stops on the last enabled row");
        p.scroll.viewport_len = 10;
        p.on_key(&k(KeyCode::PageUp, M::NONE));
        assert_eq!(p.cursor, 1, "PageUp snaps to an enabled row");
        p.on_key(&k(KeyCode::PageDown, M::NONE));
        assert_eq!(p.cursor, 3, "PageDown snaps to an enabled row");
    }

    #[test]
    fn actions_only_target_eligible_rows_on_every_path() {
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut p = picker(0);
        assert_eq!(
            p.on_key(&k(KeyCode::Delete, M::NONE)),
            (Outcome::Consumed, None)
        );
        assert_eq!(
            p.on_key(&k(KeyCode::Enter, M::NONE)),
            (Outcome::Consumed, None)
        );
        let mut p = picker(3);
        p.items[0].disabled = true;
        p.cursor = 0;
        assert_eq!(
            p.on_key(&k(KeyCode::Delete, M::NONE)),
            (Outcome::Consumed, None)
        );
        p.status = PickerStatus::Loading("…".into());
        p.cursor = 1;
        assert_eq!(
            p.on_key(&k(KeyCode::Delete, M::NONE)),
            (Outcome::Consumed, None)
        );
        assert_eq!(
            p.on_key(&k(KeyCode::Enter, M::NONE)),
            (Outcome::Consumed, None)
        );
        assert_eq!(
            p.on_click(p.row_id(1)),
            None,
            "clicks refuse a loading picker"
        );
        p.status = PickerStatus::Error {
            message: "no".into(),
            detail: None,
        };
        assert_eq!(
            p.on_key(&k(KeyCode::Delete, M::NONE)),
            (Outcome::Consumed, None)
        );
        p.status = PickerStatus::Ready;
        assert_eq!(
            p.on_key(&k(KeyCode::Delete, M::NONE)),
            (Outcome::Changed, Some(PickerAction::Secondary(1)))
        );
        assert_eq!(p.current_key(), Some("k1"));
    }

    #[test]
    fn refresh_keeps_identity_and_query_reset_selects_first() {
        let mut p = picker(4);
        p.cursor = 2;
        // a row above the cursor disappears: the cursor follows its key
        let mut items = p.items.clone();
        items.remove(0);
        p.refresh_items(items);
        assert_eq!(p.cursor, 1);
        assert_eq!(p.current_key(), Some("k2"));
        // the cursor row itself disappears: first eligible row, never a
        // neighbour pretending to be it
        let mut items = p.items.clone();
        items.remove(1);
        items[0].disabled = true;
        p.refresh_items(items);
        assert_eq!(p.current_key(), Some("k3"));
        // a query reset deliberately selects the first eligible row
        p.cursor = 1;
        p.set_items(picker(3).items);
        assert_eq!(p.cursor, 0);
    }

    #[test]
    fn query_edits_are_grapheme_safe_and_paste_is_one_event() {
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut p = picker(1);
        for c in "a👩\u{200d}💻".chars() {
            p.on_key(&k(KeyCode::Char(c), M::NONE));
        }
        assert_eq!(p.query.text(), "a👩\u{200d}💻");
        let (o, ev) = p.on_key(&k(KeyCode::Backspace, M::NONE));
        assert_eq!(
            (o, ev),
            (Outcome::Changed, Some(PickerAction::QueryChanged))
        );
        assert_eq!(p.query.text(), "a", "backspace removes the whole cluster");
        p.on_key(&k(KeyCode::Backspace, M::NONE));
        assert_eq!(
            p.on_key(&k(KeyCode::Backspace, M::NONE)),
            (Outcome::Changed, Some(PickerAction::Back))
        );
        let (o, ev) = p.on_paste("gp\ndu\t");
        assert_eq!(
            (o, ev),
            (Outcome::Changed, Some(PickerAction::QueryChanged))
        );
        assert_eq!(p.query.text(), "gp du ");
        assert_eq!(p.on_paste(""), (Outcome::Consumed, None));
        assert_eq!(
            p.on_key(&k(KeyCode::Char('w'), M::CONTROL)),
            (Outcome::Changed, Some(PickerAction::QueryChanged))
        );
        assert_eq!(p.query.text(), "gp ");
        p.on_key(&k(KeyCode::Char('u'), M::CONTROL));
        assert_eq!(p.query.text(), "");
        assert_eq!(p.on_key(&k(KeyCode::Char('u'), M::CONTROL)).1, None);
        // a search-disabled picker ignores paste and typing
        p.searchable = false;
        assert_eq!(p.on_paste("x"), (Outcome::Consumed, None));
        let (_, ev) = p.on_key(&k(KeyCode::Char('x'), M::NONE));
        assert_eq!(ev, None);
        // an unassigned modified chord is consumed by the modal, not typed
        p.searchable = true;
        let (_, ev) = p.on_key(&k(KeyCode::Char('s'), M::CONTROL));
        assert_eq!(ev, None);
        assert_eq!(p.query.text(), "");
    }

    #[test]
    fn page_keys_step_by_the_viewport_and_a_short_screen_keeps_a_row() {
        let mut p = Picker::new(WidgetId::of("p"), "Pick");
        p.set_items(
            (0..40)
                .map(|i| PickerItem {
                    label: format!("item {i:02}"),
                    detail: String::new(),
                    glyph: "",
                    group: "",
                    tag: None,
                    matched: vec![],
                    disabled: false,
                    key: format!("k{i}"),
                })
                .collect(),
        );
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 8));
        p.hints = "hints".into();
        p.draw(Rect::new(0, 0, 60, 8), &mut buf, &mut ctx);
        assert!(
            p.scroll.viewport_len >= 1,
            "a short screen still shows a row"
        );
        let shown: String = (0..8)
            .map(|y| {
                (0..60)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
                    + "\n"
            })
            .collect();
        assert!(shown.contains("item 00"), "{shown}");
        let view = p.scroll.viewport_len;
        let key = Key {
            code: KeyCode::PageDown,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        p.on_key(&key);
        assert_eq!(p.cursor, view, "one page is one viewport row set");
    }

    fn render(p: &mut Picker) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        p.draw(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
        let mut out = String::new();
        for y in 0..24 {
            for x in 0..80 {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn only_the_query_field_carries_a_focus_bar_and_the_cursor_row_is_a_fill() {
        let t = Theme::junie();
        let mut p = picker(4);
        p.query.set_text("it");
        let out = render(&mut p);
        assert_eq!(
            out.matches('▎').count(),
            1,
            "the query field owns the list: one focus bar\n{out}"
        );
        let field = out.lines().position(|l| l.contains('▎')).unwrap();
        let row = out.lines().position(|l| l.contains("Item 00")).unwrap();
        assert!(field < row, "the bar sits on the field, not the cursor row");
        // the cursor row is the highlight fill, never green
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        p.draw(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
        let x = out.lines().nth(row).unwrap().find("Item 00").unwrap();
        let x = out.lines().nth(row).unwrap()[..x].chars().count() as u16;
        assert_eq!(buf[(x, row as u16)].style().bg, Some(t.highlight));
        // the query underline is neutral: one green cue per row
        let q = out.lines().nth(field).unwrap();
        let qx = q[..q.find("it").unwrap()].chars().count() as u16;
        assert_eq!(buf[(qx, field as u16)].underline_color, t.text_secondary);
        // the bar look stays available as configuration
        let mut barred = picker(4).cursor_style(CursorStyle::Bar);
        assert_eq!(render(&mut barred).matches('▎').count(), 2);
    }

    #[test]
    fn wheel_scrolls_the_rows_and_survives_the_next_render() {
        let mut p = picker(30);
        let before = render(&mut p);
        assert!(before.contains("Item 00"));
        assert!(!before.contains("Item 10"));
        p.wheel(Axis::Vertical, 3).outcome();
        let after = render(&mut p);
        assert!(!after.contains("Item 00"), "wheel moved the viewport");
        assert!(after.contains("Item 03"));
        assert_eq!(p.scroll.offset, 3);
        // a second render keeps the offset
        let again = render(&mut p);
        assert_eq!(again, after);
        assert_eq!(p.scroll.offset, 3);
        assert_eq!(p.cursor, 0, "selection is preserved while wheel scrolling");
        p.wheel(Axis::Vertical, -3).outcome();
        let back = render(&mut p);
        assert_eq!(back, before);
    }

    #[test]
    fn keyboard_navigation_pulls_the_cursor_back_into_view() {
        let mut p = picker(30);
        render(&mut p);
        p.wheel(Axis::Vertical, 10).outcome();
        render(&mut p);
        assert_eq!(p.scroll.offset, 10);
        let key = Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        p.on_key(&key);
        let s = render(&mut p);
        assert_eq!(p.cursor, 1);
        assert!(p.scroll.visible_range().contains(&p.cursor));
        assert!(s.contains("Item 01"));
    }

    #[test]
    fn wheel_at_the_boundary_is_consumed_not_changed() {
        let mut p = picker(3);
        render(&mut p);
        assert_eq!(p.wheel(Axis::Vertical, 1).outcome(), Outcome::Consumed);
        let mut p = picker(30);
        render(&mut p);
        assert_eq!(p.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
    }

    #[test]
    fn hovering_a_cut_row_reveals_its_label_then_its_detail() {
        let label = "a picker label long enough to be cut by the label column";
        let detail = "and a detail that is also far too long for what is left of the row";
        let mut p = picker(2);
        p.items[0].label = label.into();
        p.items[1].detail = detail.into();
        let draw = |p: &mut Picker, hover| {
            let theme = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 100, 24));
            p.draw(Rect::new(0, 0, 100, 24), &mut buf, &mut ctx);
            ctx.flush_overlays(&mut buf);
            (0..24)
                .map(|y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let rest = draw(&mut p, None);
        assert!(!rest.contains(label) && !rest.contains(detail), "{rest}");
        let r0 = p.row_id(0);
        assert!(draw(&mut p, Some(r0)).contains(label));
        let r1 = p.row_id(1);
        assert!(draw(&mut p, Some(r1)).contains(detail));
    }

    #[test]
    fn an_anchored_picker_opens_under_its_control_instead_of_centred() {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let screen = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(screen);
        let mut centred = Picker::new(WidgetId::of("p"), "Pick").width(40);
        centred.draw(screen, &mut buf, &mut ctx);
        assert_eq!(centred.area.x, 30, "centred by default");
        let field = Rect::new(6, 4, 20, 1);
        let mut p = Picker::new(WidgetId::of("p"), "Pick")
            .width(40)
            .anchor(field, Placement::Below);
        p.draw(screen, &mut buf, &mut ctx);
        assert_eq!((p.area.x, p.area.y), (6, 5), "{:?}", p.area);
    }

    /// The picker over a page, and the pointer event a click at `pos`
    /// resolves to in that frame (`None`: nothing there is reachable).
    fn click_over_page(
        p: &mut Picker,
        pos: ratatui::layout::Position,
    ) -> Option<crate::core::pointer::PointerEvent> {
        use crate::core::intent::Phase;
        use ratatui::crossterm::event::KeyModifiers;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(screen);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            ctx.hits.register(WidgetId::of("page"), screen);
            p.draw(screen, &mut buf, &mut ctx);
        }
        let id = hits.hit(pos)?;
        Some(crate::core::pointer::PointerEvent::new(
            &hits,
            Phase::Click,
            id,
            pos,
            1,
            KeyModifiers::NONE,
        ))
    }

    /// A24: Alt+click on a row picks its alternative, as Alt+Enter does; a
    /// plain click picks the row.
    #[test]
    fn an_alt_click_on_a_row_chooses_its_alternative() {
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        use ratatui::crossterm::event::KeyModifiers;
        let mut p = picker(3);
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let screen = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(screen);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            p.draw(screen, &mut buf, &mut ctx);
        }
        let click_at = |pos, mods| {
            let id = hits.hit(pos)?;
            Some(PointerEvent::new(&hits, Phase::Click, id, pos, 1, mods))
        };
        let (pos, row) = (0..24)
            .flat_map(|y| (0..80).map(move |x| ratatui::layout::Position::new(x, y)))
            .find_map(|pos| Some((pos, click_at(pos, KeyModifiers::NONE)?.index_of(Part::ROW)?)))
            .expect("a row under the pointer");
        let click = |mods| click_at(pos, mods).unwrap();
        assert_eq!(
            p.pointer(&click(KeyModifiers::ALT)).into_action(),
            Some(PickerAction::ChosenAlt(row))
        );
        assert_eq!(
            p.pointer(&click(KeyModifiers::NONE)).into_action(),
            Some(PickerAction::Chosen(row))
        );
    }

    /// S5 deviation 5: a click outside the picker lands on its own
    /// light-dismiss backdrop (under its modal barrier) and cancels it
    /// through its layer, as Esc does.
    #[test]
    fn a_click_outside_the_picker_lands_on_its_backdrop_and_cancels_it() {
        use crate::ui::layer::{Dismissed, LayerPointer, LayerTag, ShellLayers};
        use crate::ui::layers::LayerResult;
        let mut p = picker(3);
        let ev = click_over_page(&mut p, ratatui::layout::Position::new(0, 1))
            .expect("the picker's backdrop is under the pointer outside it");
        assert_eq!(ev.owner(), p.id);
        assert_eq!(ev.part().part, Part::BACKDROP);
        let mut focus = crate::core::focus::Focus::default();
        let ring = FocusRing::default();
        let mut stack: ShellLayers<LayerResult> = ShellLayers::default();
        stack.open(Box::new(p), LayerTag::new("picker"), (), &mut focus);
        match stack.pointer(&ev, &mut focus, &ring, &()) {
            LayerPointer::Dismissed(Dismissed::Closed(_, r)) => assert_eq!(
                r,
                Some(LayerResult::Picker(PickerAction::Cancelled)),
                "the outside click cancels it, as Esc does"
            ),
            _ => panic!("the backdrop click dismisses the picker"),
        }
        assert!(stack.is_empty());
    }

    /// A picker whose policy refuses an outside click stays open: the
    /// click on its backdrop is consumed.
    #[test]
    fn a_refused_outside_click_on_the_picker_is_consumed() {
        use crate::core::pointer::MouseTarget;
        let mut p = picker(3).dismiss(crate::ui::popup::Dismiss::ESCAPE);
        let ev = click_over_page(&mut p, ratatui::layout::Position::new(0, 1))
            .expect("the picker's backdrop is under the pointer outside it");
        let r = p.route(&ev).expect("the backdrop is the picker's");
        assert!(r.is_consumed());
        assert_eq!(r.into_action(), None, "it stays open");
    }
}
