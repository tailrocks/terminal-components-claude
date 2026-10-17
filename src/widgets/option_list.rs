//! The one body of every popup list: Select's open list, the cell
//! editors' option list (`ChoiceEditor`), Picker, Completion and the menus
//! (plan S12, `OptionList`).
//!
//! [`OptionList`] owns what those popups used to copy: the cursor
//! ([`Cursor`]), the scroll pane and its bar ([`ScrollPane`],
//! [`Scrollable`]), the wheel ([`WheelMode`]), hover (one highlight that
//! clears when the pointer leaves the rows), row hit parts, key navigation
//! and type-ahead ([`KeyStep`]), the cursor-row look, the disabled reason
//! ([`OptionList::status`]) and the popup policy the `Overlay` impls read
//! (dismiss, placement, fit, width, max rows). The owning widget keeps
//! what is its own: the closed field, the query's keys and gestures (the
//! field itself is [`OptionList::query`]), the frame and title, and
//! the row *content* ([`OptionList::paint_rows`] hands it each row).
//!
//! Rows are [`Item`]s (main's `termrock_navigation::Item` shape, owned,
//! plus a disabled `reason` and an owner `value`); any slice of a type
//! that is [`AsItem`] is a [`RowSource`].
//!
//! Standards: WAI-ARIA APG listbox (arrows, paging by a viewport,
//! Home/End, type-ahead; the wheel moves the view, never the choice);
//! macOS menus and popup buttons (the pointer moves the one highlight, and
//! it clears when the pointer leaves the menu).

use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::Key;
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Nav, NavKeys};
use crate::core::list::{Cursor, KeyStep, Moved, RowSource};
use crate::core::pointer::PointerEvent;
use crate::core::response::Response;
use crate::core::scroll::{ScrollPane, ScrollState, Scrollable};
use crate::core::typeahead::TypeAhead;
use crate::theme::CursorStyle;
use crate::ui::ctx::RenderCtx;
use crate::ui::popup::{Dismiss, Place, Placement, backdrop_id};
use crate::ui::row::{RowLook, RowPaint, RowUi};
use crate::widgets::edit::EditField;
use crate::widgets::scrollbar;

/// What ↑/↓ (and ←/→) do on a closed select.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectArrows {
    /// Step the value without opening the list.
    Step,
    /// ↑/↓ open the list (macOS popup buttons); ←/→ are left to the page.
    /// Arrows never change the value unseen.
    Open,
}

/// What the wheel does over an open popup list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WheelMode {
    /// Scroll the list; the cursor (and the value) stay where they are
    /// (macOS menus, WAI-ARIA listbox: the wheel moves the view, never the
    /// choice).
    #[default]
    Viewport,
    /// Move the cursor one option per line (the old behaviour).
    Cursor,
}

/// One option row: main's `termrock_navigation::Item` shape, owned, plus
/// why a disabled row is disabled and a payload for the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Stable identity ([`ItemKey::text`] for string keys). Default
    /// `ItemKey::Index(0)`: the owner keys by position.
    pub key: ItemKey,
    pub label: String,
    /// One-cell kind glyph (T, V, C, ƒ, ·, …).
    pub glyph: Cow<'static, str>,
    pub detail: String,
    /// Trailing hint on the row ("open", a shortcut).
    pub tag: Option<Cow<'static, str>>,
    /// The group the row belongs to (a section heading).
    pub group: Cow<'static, str>,
    /// Byte positions in `label` that matched the typed text.
    pub matched: Vec<usize>,
    /// The text a completion inserts; `None` inserts the label
    /// ([`Self::insertion`]).
    pub insert: Option<String>,
    /// Shown but never chosen.
    pub disabled: bool,
    /// Why the row is disabled, shown in the popup's status row while the
    /// cursor is on it ([`OptionList::status`]).
    pub reason: Option<String>,
    /// A payload for the owner (an index into its own data).
    pub value: usize,
}

impl Default for Item {
    fn default() -> Self {
        Self::new("")
    }
}

impl Item {
    /// An enabled row showing `label`, keyed by position.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            key: ItemKey::index(0),
            label: label.into(),
            glyph: Cow::Borrowed(""),
            detail: String::new(),
            tag: None,
            group: Cow::Borrowed(""),
            matched: vec![],
            insert: None,
            disabled: false,
            reason: None,
            value: 0,
        }
    }

    /// See [`Self::key`].
    pub fn key(mut self, key: ItemKey) -> Self {
        self.key = key;
        self
    }

    /// See [`Self::glyph`].
    pub fn glyph(mut self, glyph: impl Into<Cow<'static, str>>) -> Self {
        self.glyph = glyph.into();
        self
    }

    /// See [`Self::detail`].
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    /// See [`Self::tag`].
    pub fn tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// See [`Self::group`].
    pub fn group(mut self, group: impl Into<Cow<'static, str>>) -> Self {
        self.group = group.into();
        self
    }

    /// See [`Self::matched`].
    pub fn matched(mut self, matched: Vec<usize>) -> Self {
        self.matched = matched;
        self
    }

    /// See [`Self::insert`].
    pub fn insert(mut self, insert: impl Into<String>) -> Self {
        self.insert = Some(insert.into());
        self
    }

    /// See [`Self::disabled`].
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Disable the row and say why ([`Self::reason`]).
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self.disabled = true;
        self
    }

    /// See [`Self::value`].
    pub fn value(mut self, value: usize) -> Self {
        self.value = value;
        self
    }

    /// The text a completion inserts: [`Self::insert`], or the label.
    pub fn insertion(&self) -> &str {
        self.insert.as_deref().unwrap_or(&self.label)
    }
}

/// A row type that carries an [`Item`] (a menu item, a picker row), so a
/// slice of it is a [`RowSource`] and [`OptionList`] can read it.
pub trait AsItem {
    /// The row as an [`Item`].
    fn as_item(&self) -> &Item;
}

impl AsItem for Item {
    fn as_item(&self) -> &Item {
        self
    }
}

impl<T: AsItem> RowSource for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
    fn enabled(&self, i: usize) -> bool {
        !self[i].as_item().disabled
    }
    fn label(&self, i: usize) -> &str {
        &self[i].as_item().label
    }
    fn key(&self, i: usize) -> ItemKey {
        match self[i].as_item().key {
            // a positional default keys by the row's position
            ItemKey::Index(_) => ItemKey::index(i),
            k => k,
        }
    }
}

/// The body of a popup list. See the module docs.
#[derive(Debug, Clone)]
pub struct OptionList {
    /// The owner's id: rows are `id.part(PartRef::item(Part::ROW,
    /// ItemKey::index(i)))` ([`Self::row_id`]), the bar is the owner's.
    pub id: WidgetId,
    /// The highlighted row and its rules (wrap, disabled-row skipping).
    pub cursor: Cursor,
    /// The rows' vertical scroll state and the bar's track.
    pub scroll: ScrollPane,
    /// How the cursor row is marked. Default [`CursorStyle::Fill`]: the
    /// owner (a field, an editor, a query) keeps the only focus bar.
    pub look: CursorStyle,
    /// What the wheel does. Default [`WheelMode::Viewport`] (plan §3
    /// "Wheel in popup lists").
    pub wheel: WheelMode,
    /// The pointer moving onto a row moves the cursor there, so the list
    /// shows one highlight (macOS menus); it clears when the pointer
    /// leaves the rows. Off, a hovered row gets the hover tint beside the
    /// cursor fill. Default `true`.
    pub hover_moves_cursor: bool,
    /// Which keys move the cursor. Default [`NavKeys::default`].
    pub nav: NavKeys,
    /// Typing a prefix moves the cursor to the matching row. Default from
    /// the [`crate::core::list::NavProfile`].
    pub type_ahead: bool,
    /// The rows fade at an edge with more rows beyond it, beside the bar
    /// ([`scrollbar::render_with_fade`]). Default `true`.
    pub fade_edges: bool,
    /// How the popup closes without a choice. Default [`Dismiss::LIGHT`].
    pub dismiss: Dismiss,
    /// Which side of its anchor the popup opens on. Default
    /// [`Placement::Below`].
    pub placement: Placement,
    /// How the popup keeps off the screen edges. Default
    /// [`Place::DEFAULT`].
    pub fit: Place,
    /// Most rows shown before the list scrolls. Default 8.
    pub max_rows: u16,
    /// Popup width bounds `(min, max)` in cells, frame included. Default
    /// `(12, 40)`.
    pub width: (u16, u16),
    /// The query field that filters the rows (a Picker's search, a
    /// combo box's text), owned by the list's owner for its keys and
    /// gestures. `None`: the list has no query and its letters
    /// type-select ([`Self::type_ahead`]). Default `None`.
    pub query: Option<EditField>,
    typeahead: TypeAhead,
    /// The hover last applied to the cursor.
    hover: Option<WidgetId>,
    /// The row under the pointer (enabled or not), until a key moves the
    /// cursor: a disabled row says why while it is pointed at.
    pointed: Option<usize>,
    /// The cursor sits where the pointer put it.
    hovering: bool,
    /// The pointer left the rows it moved over: no row is highlighted.
    hidden: bool,
    /// The rows' cells in the last layout (bar column excluded).
    rows_area: Rect,
    /// The line each row starts on, plus the total, when a row may be
    /// followed by a gap line ([`Self::set_gaps`]); empty: row `i` is
    /// line `i`.
    lines: Vec<usize>,
}

impl OptionList {
    /// Default for [`Self::max_rows`].
    pub const MAX_ROWS: u16 = 8;
    /// Default for [`Self::width`].
    pub const WIDTH: (u16, u16) = (12, 40);

    /// An empty list for the popup `id`.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            cursor: Cursor::new(0),
            scroll: ScrollPane::default(),
            look: CursorStyle::default(),
            wheel: WheelMode::Viewport,
            hover_moves_cursor: true,
            nav: NavKeys::default(),
            type_ahead: crate::core::list::NavProfile::current().type_ahead(),
            fade_edges: true,
            dismiss: Dismiss::LIGHT,
            placement: Placement::Below,
            fit: Place::DEFAULT,
            max_rows: Self::MAX_ROWS,
            width: Self::WIDTH,
            query: None,
            typeahead: TypeAhead::default(),
            hover: None,
            pointed: None,
            hovering: false,
            hidden: false,
            rows_area: Rect::ZERO,
            lines: Vec::new(),
        }
    }

    /// The pause after which the next letter starts a new type-ahead
    /// prefix. Default [`TypeAhead::DEFAULT_TIMEOUT`].
    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.typeahead.timeout = timeout;
        self
    }

    /// Rows followed by a gap line (`true`): a menu's separator takes a
    /// line of its own. The view scrolls by lines, so the gaps count
    /// toward the viewport and the content; rows keep their indices.
    /// Call it again when the rows change.
    pub fn set_gaps(&mut self, gaps: impl IntoIterator<Item = bool>) {
        self.lines.clear();
        let mut line = 0;
        let mut any = false;
        for gap in gaps {
            self.lines.push(line);
            line += 1 + usize::from(gap);
            any |= gap;
        }
        self.lines.push(line);
        if !any {
            self.lines.clear();
        }
    }

    /// The line row `i` sits on ([`Self::set_gaps`]).
    pub fn line_of(&self, i: usize) -> usize {
        match self.lines.get(i) {
            Some(&l) => l,
            None => i,
        }
    }

    /// The row on line `line`, `None` on a gap line.
    fn row_on_line(&self, line: usize) -> Option<usize> {
        if self.lines.is_empty() {
            return Some(line);
        }
        let i = self.lines.binary_search(&line).ok()?;
        (i + 1 < self.lines.len()).then_some(i)
    }

    /// The lines of `len` rows: rows and their gaps.
    fn line_count(&self, len: usize) -> usize {
        if self.lines.len() == len + 1 {
            self.lines[len]
        } else {
            len
        }
    }

    /// The lines in view, top first: each with the row on it (`None` on a
    /// gap line). Draw from line `k` at `inner.y + k`.
    pub fn visible_lines(&self) -> impl Iterator<Item = (u16, Option<usize>)> + '_ {
        self.scroll
            .visible_range()
            .enumerate()
            .map(|(k, line)| (k as u16, self.row_on_line(line)))
    }

    /// Whether a row lies below the view (a menu's ▼ marker), among `len`
    /// rows.
    pub fn more_below(&self, len: usize) -> bool {
        len > 0 && self.line_of(len - 1) >= self.scroll.offset + self.scroll.viewport_len
    }

    /// See [`Self::wheel`] (named apart from [`Scrollable::wheel`]).
    pub fn wheel_mode(mut self, mode: WheelMode) -> Self {
        self.wheel = mode;
        self
    }

    /// See [`Self::hover_moves_cursor`].
    pub fn hover_moves_cursor(mut self, on: bool) -> Self {
        self.hover_moves_cursor = on;
        self
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    /// See [`Self::type_ahead`].
    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    /// See [`Self::fade_edges`].
    pub fn fade_edges(mut self, on: bool) -> Self {
        self.fade_edges = on;
        self
    }

    /// See [`Self::look`].
    pub fn look(mut self, look: CursorStyle) -> Self {
        self.look = look;
        self
    }

    /// See [`Self::dismiss`].
    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// See [`Self::placement`].
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// See [`Self::fit`].
    pub fn fit(mut self, fit: Place) -> Self {
        self.fit = fit;
        self
    }

    /// See [`Self::max_rows`].
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows;
        self
    }

    /// See [`Self::width`]; `max` never under `min`.
    pub fn width(mut self, min: u16, max: u16) -> Self {
        self.width = (min, max.max(min));
        self
    }

    /// See [`Self::query`].
    pub fn query(mut self, field: EditField) -> Self {
        self.query = Some(field);
        self
    }

    /// New rows: `len` of them, the cursor on the first, the view at the
    /// top, no hover or typed prefix carried over.
    pub fn reset(&mut self, len: usize) {
        self.cursor.index = 0;
        self.scroll.v = ScrollState::new(len);
        self.typeahead.reset();
        self.hover = None;
        self.pointed = None;
        self.hovering = false;
        self.hidden = false;
    }

    /// The highlighted row: the cursor, unless the pointer left the rows
    /// it moved the cursor over ([`Self::hover_moves_cursor`]).
    pub fn highlighted(&self) -> Option<usize> {
        (!self.hidden).then_some(self.cursor.index)
    }

    /// The hit part of row `i`.
    pub fn row_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    /// The hit id of row `i`.
    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::row_part(i))
    }

    /// The row a hit id stands for, among `len` rows.
    pub fn row_of(&self, id: WidgetId, len: usize) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..len)
    }

    /// The row a routed pointer event is on.
    pub fn row_of_event(&self, ev: &PointerEvent) -> Option<usize> {
        if !ev.on(self.id) {
            return None;
        }
        ev.index_of(Part::ROW)
            .filter(|&i| i < self.scroll.content_len)
    }

    /// Whether a routed pointer event is on the popup's light-dismiss
    /// backdrop ([`backdrop_id`]).
    pub fn on_backdrop(&self, ev: &PointerEvent) -> bool {
        ev.id == backdrop_id(self.id)
    }

    /// The row under `pos` in the last layout (a press-drag-release
    /// gesture, which has no hit part of its own).
    pub fn row_at(&self, pos: Position) -> Option<usize> {
        if !self.rows_area.contains(pos) {
            return None;
        }
        let line = self.scroll.offset + (pos.y - self.rows_area.y) as usize;
        let i = self.row_on_line(line)?;
        (line < self.scroll.content_len).then_some(i)
    }

    /// The rows' cells in the last layout.
    pub fn rows_area(&self) -> Rect {
        self.rows_area
    }

    /// A key: a navigation step per [`Self::nav`] (paging by the
    /// viewport), or a type-ahead letter when [`Self::type_ahead`] is on.
    /// `Ignored` when the key is neither.
    pub fn key<S: RowSource + ?Sized>(&mut self, key: &Key, src: &S) -> Moved {
        match KeyStep::of(key, &self.nav, self.type_ahead) {
            Some(KeyStep::Nav(nav)) => self.nav_step(nav, src),
            Some(KeyStep::Type(c)) => {
                let moved = self.cursor.type_ahead(c, &mut self.typeahead, src);
                self.keyed(moved)
            }
            None => Moved::Ignored,
        }
    }

    /// A navigation step (paging by the viewport).
    pub fn nav_step<S: RowSource + ?Sized>(&mut self, nav: Nav, src: &S) -> Moved {
        let page = self.scroll.viewport_len;
        let moved = self.cursor.apply(nav, src, page);
        self.keyed(moved)
    }

    /// Step `delta` rows (Ctrl+N / Ctrl+P in an editor's popup).
    pub fn step<S: RowSource + ?Sized>(&mut self, delta: isize, src: &S) -> Moved {
        let moved = self.cursor.step(delta, src);
        self.keyed(moved)
    }

    /// The pointer placed the cursor on row `i` (a click, a drag over the
    /// rows): it comes into view exactly, without context rows.
    pub fn select<S: RowSource + ?Sized>(&mut self, i: usize, src: &S) -> Moved {
        let moved = self.cursor.select(i, src);
        if let Some(i) = moved.index() {
            self.hidden = false;
            self.scroll.pointer_move();
            self.scroll.follow(self.line_of(i));
        }
        moved
    }

    /// A keyboard move: the highlight shows again and the cursor comes
    /// into view now (the viewport follows only a moved cursor, so a
    /// wheel scroll survives the next frame).
    fn keyed(&mut self, moved: Moved) -> Moved {
        if let Some(i) = moved.index() {
            self.hidden = false;
            self.scroll.follow(self.line_of(i));
            // a key after a hover: the next hover applies again, and the
            // status reads the cursor's row, not the pointer's
            self.hovering = false;
            self.pointed = None;
        }
        moved
    }

    /// Lay the rows out in `inner` (the popup's frame excluded) for this
    /// frame: the content and viewport, the hover (which may move the
    /// cursor), the cursor brought into view if it moved, and the bar's
    /// track recorded for presses and drags. Call it from render, before
    /// queuing the overlay that paints the rows from a copy.
    pub fn layout<S: RowSource + ?Sized>(&mut self, inner: Rect, src: &S, ctx: &RenderCtx) {
        let len = src.len();
        self.scroll.set_content(self.line_count(len));
        self.scroll.set_viewport(inner.height as usize);
        self.cursor.index = self.cursor.index.min(len.saturating_sub(1));
        let with_bar = self.scroll.overflows() && inner.width >= 2;
        self.rows_area = Rect::new(
            inner.x,
            inner.y,
            inner
                .width
                .saturating_sub(u16::from(self.scroll.overflows())),
            inner.height,
        );
        self.hover(ctx, src);
        self.scroll.follow(self.line_of(self.cursor.index));
        if with_bar {
            let track = Rect::new(inner.right() - 1, inner.y, 1, inner.height);
            self.scroll.set_track(self.id, Axis::Vertical, track);
        } else {
            self.scroll.clear_track(Axis::Vertical);
        }
    }

    /// Apply the pointer's hover ([`Self::hover_moves_cursor`]): onto an
    /// enabled row it moves the cursor there; leaving the rows it moved
    /// over clears the highlight.
    fn hover<S: RowSource + ?Sized>(&mut self, ctx: &RenderCtx, src: &S) {
        let hover = ctx.interaction.live_hover();
        if hover == self.hover {
            return;
        }
        self.hover = hover;
        let row = hover.and_then(|h| self.row_of(h, src.len()));
        self.pointed = row;
        if !self.hover_moves_cursor {
            return;
        }
        match row {
            Some(i) if src.enabled(i) => {
                self.cursor.index = i;
                self.hovering = true;
                self.hidden = false;
                // the row is on screen under the pointer: never scroll it
                self.scroll.pointer_move();
                self.scroll.follow(self.line_of(i));
            }
            // the pointer left the rows it was moving over: no row stays
            // highlighted (macOS menus)
            _ if self.hovering => {
                self.hovering = false;
                self.hidden = true;
            }
            _ => {}
        }
    }

    /// Paint the visible rows in `inner` (the area given to
    /// [`Self::layout`]) on the plane `bg`: each row's body and gutter
    /// through the one row painter ([`RowUi::paint`] with
    /// [`RowLook::Fill`] of [`Self::look`]), then `content` draws the
    /// row's own cells (its index and the [`RowPaint`]; the row's area
    /// excludes the bar column and content starts at `area.x + 1`; its
    /// state is `focused` on the highlighted row, `hovered` only where
    /// hover does not move the cursor), then its hit part is registered;
    /// last the bar (with the edge fade per [`Self::fade_edges`]).
    pub fn paint_rows<S: RowSource + ?Sized>(
        &self,
        inner: Rect,
        bg: Color,
        src: &S,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        mut content: impl FnMut(usize, &RowPaint, &mut Buffer, &mut RenderCtx),
    ) {
        let has_sb = self.scroll.overflows();
        let highlighted = self.highlighted();
        let len = src.len();
        for (k, i) in self.visible_lines() {
            if k >= inner.height {
                break;
            }
            // a gap line is the owner's to draw
            let Some(i) = i else { continue };
            if i >= len {
                break;
            }
            let y = inner.y + k;
            let mut state = ctx.state(self.row_id(i));
            state.focused = highlighted == Some(i);
            state.disabled = !src.enabled(i);
            // the cursor follows the pointer: the hover is the cursor
            state.hovered &= !self.hover_moves_cursor;
            let area = Rect::new(inner.x, y, inner.width.saturating_sub(u16::from(has_sb)), 1);
            let row = RowUi::paint(ctx, buf, area, state, RowLook::Fill(self.look), bg);
            content(i, &row, buf, ctx);
            ctx.register_part(self.id, Self::row_part(i), area);
        }
        if has_sb {
            if self.fade_edges {
                scrollbar::render_with_fade(inner, buf, ctx, self.id, &self.scroll, true);
            } else if inner.width >= 2 {
                let track = Rect::new(inner.right() - 1, inner.y, 1, inner.height);
                scrollbar::render_vertical(track, buf, ctx, self.id, &self.scroll, false);
            }
        }
    }

    /// The status line: the disabled reason of the row under the pointer,
    /// else of the highlighted row (SI21/SV15). `None` on an enabled row
    /// or with no highlight.
    pub fn status<'a, T: AsItem>(&self, items: &'a [T]) -> Option<&'a str> {
        let reason = |i: usize| {
            let it = items.get(i)?.as_item();
            if it.disabled {
                it.reason.as_deref()
            } else {
                None
            }
        };
        // the row under the pointer first (a disabled row the pointer
        // rests on says why), then the cursor's
        self.pointed
            .and_then(reason)
            .or_else(|| self.highlighted().and_then(reason))
    }
}

/// The wheel and the bar move the rows' view ([`ScrollPane`]); under
/// [`WheelMode::Cursor`] the wheel moves the cursor instead. A sideways
/// notch is consumed: what is beneath the popup does not move.
impl Scrollable for OptionList {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.scroll
    }

    fn wheel_first(&mut self, axis: Axis, delta: i32) -> Option<Response<()>> {
        if axis == Axis::Horizontal {
            return Some(Response::consumed());
        }
        if self.wheel == WheelMode::Viewport {
            return None;
        }
        let rows = if self.lines.is_empty() {
            self.scroll.content_len
        } else {
            self.lines.len() - 1
        };
        let last = rows.saturating_sub(1);
        let from = self.cursor.index.min(last);
        let to = if delta < 0 {
            from.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (from + delta as usize).min(last)
        };
        if to == from {
            return Some(Response::consumed());
        }
        self.cursor.index = to;
        self.hidden = false;
        self.scroll.follow(self.line_of(to));
        Some(Response::changed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Outcome;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn items(n: usize) -> Vec<Item> {
        (0..n).map(|i| Item::new(format!("item{i:02}"))).collect()
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    const INNER: Rect = Rect::new(1, 1, 20, 5);

    /// One frame: layout under `hover`, then the rows painted; the frame's
    /// buffer and hits come back.
    fn frame(l: &mut OptionList, rows: &[Item], hover: Option<WidgetId>) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
        {
            let mut ctx = RenderCtx::new(
                &theme,
                Interaction {
                    hover,
                    ..Interaction::default()
                },
                &mut hits,
                &mut ring,
            );
            l.layout(INNER, rows, &ctx);
            l.paint_rows(
                INNER,
                theme.surface_elevated,
                rows,
                &mut buf,
                &mut ctx,
                |i, r, buf, _| {
                    buf.set_string(r.area.x + 1, r.area.y, &rows[i].label, r.style());
                },
            );
        }
        (buf, hits)
    }

    #[test]
    fn items_build_and_insert_their_label_unless_told_otherwise() {
        let it = Item::new("fetch")
            .glyph("ƒ")
            .detail("fn")
            .tag("open")
            .group("Functions")
            .matched(vec![0])
            .key(ItemKey::text("fetch"))
            .value(7);
        assert_eq!(it.insertion(), "fetch");
        assert_eq!(it.clone().insert("fetch(").insertion(), "fetch(");
        assert_eq!(it.value, 7);
        assert!(!it.disabled);
        let off = Item::new("x").reason("Not here");
        assert!(off.disabled, "a reason disables the row");
        let rows = [it, off];
        assert!(rows.enabled(0) && !rows.enabled(1));
        assert_eq!(rows.label(0), "fetch");
        assert_eq!(rows.key(0), ItemKey::text("fetch"));
        assert_eq!(
            rows.key(1),
            ItemKey::index(1),
            "a positional key is the row"
        );
    }

    #[test]
    fn keys_step_page_by_the_viewport_and_type_ahead() {
        let rows = items(20);
        let mut l = OptionList::new(WidgetId::of("l")).type_ahead(true);
        l.reset(rows.len());
        frame(&mut l, &rows, None);
        assert_eq!(l.key(&key(KeyCode::Down), &rows[..]), Moved::To(1));
        assert_eq!(l.key(&key(KeyCode::PageDown), &rows[..]), Moved::To(6));
        assert!(
            l.scroll.visible_range().contains(&6),
            "a key brings it into view"
        );
        assert_eq!(l.key(&key(KeyCode::Tab), &rows[..]), Moved::Ignored);
        let named = ["alpha", "bravo", "charlie"].map(Item::new);
        l.reset(named.len());
        assert_eq!(l.key(&key(KeyCode::Char('c')), &named[..]), Moved::To(2));
        let quiet = OptionList::new(WidgetId::of("l")).type_ahead(false);
        assert_eq!(
            quiet.clone().key(&key(KeyCode::Char('c')), &named[..]),
            Moved::Ignored,
            "letters pass when type-ahead is off"
        );
    }

    #[test]
    fn the_viewport_wheel_scrolls_and_the_next_frame_keeps_it() {
        let rows = items(20);
        let mut l = OptionList::new(WidgetId::of("l"));
        l.reset(rows.len());
        frame(&mut l, &rows, None);
        assert_eq!(l.wheel(Axis::Vertical, 3).outcome(), Outcome::Changed);
        assert_eq!((l.scroll.offset, l.cursor.index), (3, 0));
        frame(&mut l, &rows, None);
        assert_eq!(l.scroll.offset, 3, "a frame never pulls the view back");
        assert_eq!(l.wheel(Axis::Vertical, -5).outcome(), Outcome::Changed);
        assert_eq!(l.wheel(Axis::Vertical, -1).outcome(), Outcome::Consumed);
        assert_eq!(
            l.wheel(Axis::Horizontal, 1).outcome(),
            Outcome::Consumed,
            "a sideways notch stops at the popup"
        );
    }

    #[test]
    fn the_cursor_wheel_moves_the_cursor_and_the_view_follows() {
        let rows = items(20);
        let mut l = OptionList::new(WidgetId::of("l")).wheel_mode(WheelMode::Cursor);
        l.reset(rows.len());
        frame(&mut l, &rows, None);
        assert_eq!(l.wheel(Axis::Vertical, 6).outcome(), Outcome::Changed);
        assert_eq!(l.cursor.index, 6);
        assert!(l.scroll.visible_range().contains(&6));
        assert_eq!(l.wheel(Axis::Vertical, -9).outcome(), Outcome::Changed);
        assert_eq!(l.cursor.index, 0);
        assert_eq!(l.wheel(Axis::Vertical, -1).outcome(), Outcome::Consumed);
    }

    #[test]
    fn hover_moves_the_one_highlight_and_clears_when_the_pointer_leaves() {
        let rows = items(4);
        let mut l = OptionList::new(WidgetId::of("l"));
        l.reset(rows.len());
        let row2 = l.row_id(2);
        frame(&mut l, &rows, Some(row2));
        assert_eq!(l.highlighted(), Some(2));
        frame(&mut l, &rows, Some(WidgetId::of("page")));
        assert_eq!(l.highlighted(), None, "no stale highlight outside the rows");
        assert_eq!(l.cursor.index, 2, "the cursor stays for the keys");
        l.key(&key(KeyCode::Down), &rows[..]);
        assert_eq!(l.highlighted(), Some(3), "a key shows it again");
        // off: hover tints, the cursor stays
        let mut tint = OptionList::new(WidgetId::of("l")).hover_moves_cursor(false);
        tint.reset(rows.len());
        frame(&mut tint, &rows, Some(row2));
        assert_eq!(tint.highlighted(), Some(0));
    }

    #[test]
    fn rows_register_their_parts_and_map_back() {
        let rows = items(20);
        let mut l = OptionList::new(WidgetId::of("l"));
        l.reset(rows.len());
        let (_, hits) = frame(&mut l, &rows, None);
        let first = hits
            .regions()
            .iter()
            .find(|r| r.owner == l.id && r.part == OptionList::row_part(0))
            .expect("row 0 is a part");
        assert_eq!(first.area.y, INNER.y);
        assert_eq!(
            first.area.width,
            INNER.width - 1,
            "the bar column is not a row"
        );
        assert_eq!(l.row_of(l.row_id(5), rows.len()), Some(5));
        assert_eq!(l.row_at(Position::new(INNER.x, INNER.y + 2)), Some(2));
        assert_eq!(l.row_at(Position::new(INNER.right() - 1, INNER.y)), None);
        assert!(!l.scroll.v_track().is_empty(), "the bar can be pressed");
    }

    #[test]
    fn the_status_is_the_reason_of_a_disabled_cursor_row() {
        let rows = vec![Item::new("a"), Item::new("b").reason("Not in this preview")];
        let mut l = OptionList::new(WidgetId::of("l"));
        l.reset(rows.len());
        assert_eq!(l.status(&rows), None);
        l.cursor.index = 1;
        assert_eq!(l.status(&rows), Some("Not in this preview"));
    }
}
