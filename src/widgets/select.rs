//! Dropdown field: closed it looks like a text field with a trailing `▾`;
//! open it shows an anchored popup list. One focus stop.
//!
//! The list is an overlay ([`RenderCtx::overlay`]): it is drawn above the
//! whole page whatever is rendered after the field, flips or shrinks to stay
//! on screen, and scrolls when it has more options than rows. How it closes
//! without a choice is the [`Dismiss`] policy (default: Esc or a click
//! outside).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::keymap::{Nav, NavKeys};
use crate::core::scroll::{ScrollState, TrackClick};
use crate::theme::CursorStyle;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Placement, backdrop_id, place_with, surface_for};
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

/// What the wheel does over the open list.
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

/// A press held since it landed on the field (see [`Select::on_press`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Press {
    /// The press opened the list: its own release keeps the list open.
    opened: bool,
    /// The press was on the label ([`Select::label_hit`]).
    label: bool,
    origin: Position,
    /// The pointer has left the cell the press started on.
    moved: bool,
}

#[derive(Debug, Clone)]
pub struct Select {
    pub id: WidgetId,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub cursor: usize,
    pub open: bool,
    pub disabled: bool,
    pub help: String,
    pub area: Rect,
    /// How the open list closes without a choice.
    pub dismiss: Dismiss,
    /// Most option rows the open list shows before it scrolls.
    pub max_rows: u16,
    /// Where the open list goes: under the field (flipping above when
    /// there is no room) or centered on the screen.
    pub placement: Placement,
    /// Width range of the open list, frame included; inside it the list
    /// takes the field's width.
    pub popup_width: (u16, u16),
    /// How the open list keeps off the screen edges and where a centred
    /// list sits. Default [`Place::DEFAULT`](crate::ui::popup::Place):
    /// one free cell from every edge.
    pub popup_fit: crate::ui::popup::Place,
    /// First option row shown in the open list.
    pub scroll: usize,
    /// How the open list marks its cursor row. Default
    /// [`CursorStyle::Fill`]: the field owns the list and keeps the only
    /// focus bar.
    pub cursor_style: CursorStyle,
    /// Option rows the open list showed in the last frame.
    visible_rows: usize,
    /// How a cut value or option reveals its full text on hover (see
    /// [`crate::ui::reveal`]); `None` follows [`RenderCtx::reveal`].
    pub reveal: Option<crate::ui::reveal::Reveal>,
    /// What arrows do while the list is closed. Default
    /// [`SelectArrows::Step`] (the long-standing behaviour; `Open` is the macOS
    /// popup-button convention).
    pub arrows: SelectArrows,
    /// A placeholder shown instead of the value while the field stands for
    /// several differing values (a bulk edit: `— mixed —`). Choosing any
    /// option clears it. Default `None`.
    pub mixed: Option<String>,
    /// Typing a prefix picks the matching option (closed) or moves the
    /// list cursor (open), as macOS popup buttons do. Off by default:
    /// plain letters stay with the page's own shortcuts.
    pub type_ahead: bool,
    /// Stepping past either end (open list ↑/↓, closed field arrows)
    /// lands on the other end. Default off: it stops at the end.
    pub wrap: bool,
    typeahead: crate::core::typeahead::TypeAhead,
    /// An open list closes (without changing the value) when the field
    /// loses focus ([`Self::on_blur`]). Default `true`.
    pub close_on_blur: bool,
    /// Which keys move the open list's cursor (default
    /// [`NavKeys::DEFAULT`]). The closed field steps its value with the
    /// arrows only, so letters stay with the page's own shortcuts.
    pub nav: NavKeys,
    /// What the wheel does over the open list. Default
    /// [`WheelMode::Viewport`].
    pub wheel: WheelMode,
    /// The pointer moving over an option moves the list cursor there, so
    /// the list shows one highlight, as macOS menus do. Off, a hovered row
    /// gets the hover tint beside the cursor fill. Default `true`.
    pub hover_moves_cursor: bool,
    /// Press the closed field, drag onto an option and release to choose
    /// it in one gesture (macOS popup buttons). The press opens the list;
    /// a release without movement leaves it open for a click. Needs the
    /// owner to route [`Self::on_press`], [`Self::on_drag`] and
    /// [`Self::on_release`]. Default `true`.
    pub press_drag: bool,
    /// A click on the label focuses the field (the HTML `<label>`
    /// convention) without opening the list. Default `true`.
    pub label_hit: bool,
    /// What a press on the bare track of the list's scrollbar does.
    /// `None` (default) follows [`RenderCtx::track_click`].
    pub track_click: Option<TrackClick>,
    v_click: TrackClick,
    /// The list's scroll bookkeeping for its scrollbar (thumb grab).
    bar: ScrollState,
    /// The list's scrollbar track in the last frame.
    bar_track: Rect,
    /// The option rows of the open list in the last frame.
    rows_area: Rect,
    /// The label row in the last frame.
    label_area: Rect,
    /// Hover target applied to the cursor last ([`Self::hover_moves_cursor`]).
    last_hover: Option<WidgetId>,
    /// Cursor the list last showed: a cursor that moved is scrolled into
    /// view, a list scrolled by the wheel or the bar stays put.
    shown_cursor: Option<usize>,
    press: Option<Press>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectEvent {
    Changed(usize),
}

impl Select {
    pub const HEIGHT: u16 = 3;
    /// Default for [`Select::max_rows`].
    pub const MAX_ROWS: u16 = 8;
    /// Default for [`Select::popup_width`].
    pub const POPUP_WIDTH: (u16, u16) = (12, 40);

    pub fn new(id: WidgetId, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            id,
            label: label.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            selected,
            cursor: selected,
            open: false,
            disabled: false,
            help: String::new(),
            area: Rect::ZERO,
            dismiss: Dismiss::LIGHT,
            max_rows: Self::MAX_ROWS,
            placement: Placement::Below,
            popup_width: Self::POPUP_WIDTH,
            popup_fit: crate::ui::popup::Place::DEFAULT,
            scroll: 0,
            cursor_style: CursorStyle::default(),
            visible_rows: Self::MAX_ROWS as usize,
            reveal: None,
            arrows: SelectArrows::Step,
            mixed: None,
            type_ahead: false,
            wrap: false,
            typeahead: Default::default(),
            close_on_blur: true,
            nav: NavKeys::DEFAULT,
            wheel: WheelMode::Viewport,
            hover_moves_cursor: true,
            press_drag: true,
            label_hit: true,
            track_click: None,
            v_click: TrackClick::default(),
            bar: ScrollState::default(),
            bar_track: Rect::ZERO,
            rows_area: Rect::ZERO,
            label_area: Rect::ZERO,
            last_hover: None,
            shown_cursor: None,
            press: None,
        }
    }

    /// See [`Self::wheel`].
    pub fn wheel(mut self, mode: WheelMode) -> Self {
        self.wheel = mode;
        self
    }

    /// See [`Self::hover_moves_cursor`].
    pub fn hover_moves_cursor(mut self, on: bool) -> Self {
        self.hover_moves_cursor = on;
        self
    }

    /// See [`Self::press_drag`].
    pub fn press_drag(mut self, on: bool) -> Self {
        self.press_drag = on;
        self
    }

    /// See [`Self::label_hit`].
    pub fn label_hit(mut self, on: bool) -> Self {
        self.label_hit = on;
        self
    }

    /// See [`Self::track_click`].
    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    /// Hit id of the open list's scrollbar.
    pub fn bar_id(&self) -> WidgetId {
        scrollbar::id_for(self.id)
    }

    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }
    pub fn dismiss(mut self, d: Dismiss) -> Self {
        self.dismiss = d;
        self
    }
    pub fn max_rows(mut self, rows: u16) -> Self {
        self.max_rows = rows.max(1);
        self
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    /// See [`Self::popup_fit`].
    pub fn popup_fit(mut self, fit: crate::ui::popup::Place) -> Self {
        self.popup_fit = fit;
        self
    }

    pub fn popup_width(mut self, min: u16, max: u16) -> Self {
        self.popup_width = (min.max(1), max.max(min.max(1)));
        self
    }
    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = cursor_style;
        self
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.to_owned();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Show `placeholder` instead of the value (see [`Self::mixed`]).
    pub fn mixed(mut self, placeholder: &str) -> Self {
        self.mixed = Some(placeholder.to_owned());
        self
    }

    /// Set or clear the mixed-values placeholder.
    pub fn set_mixed(&mut self, placeholder: Option<&str>) {
        self.mixed = placeholder.map(str::to_owned);
    }

    /// Choose option `i`, leaving the mixed state. Returns the change
    /// event when the shown value changed.
    fn choose(&mut self, i: usize) -> Option<SelectEvent> {
        let changed = i != self.selected || self.mixed.is_some();
        self.selected = i;
        self.mixed = None;
        changed.then_some(SelectEvent::Changed(i))
    }

    pub fn value(&self) -> &str {
        self.options
            .get(self.selected)
            .map(String::as_str)
            .unwrap_or("")
    }
    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        (0..self.options.len()).find(|&i| self.option_id(i) == id)
    }
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id
            || id == backdrop_id(self.id)
            || id == self.bar_id()
            || self.locate(id).is_some()
    }

    fn last(&self) -> usize {
        self.options.len().saturating_sub(1)
    }

    fn open_list(&mut self) {
        self.open = true;
        self.cursor = self.selected;
        self.scroll = self
            .cursor
            .saturating_sub(self.visible_rows.saturating_sub(1));
        self.shown_cursor = None;
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        if let KeyCode::Char(c) = key.code
            && self.type_ahead
            && key.plain()
            && !c.is_whitespace()
            && !(self.open && self.nav.action(key).is_some())
        {
            let prefix = self.typeahead.feed(c).to_owned();
            let from = if self.open {
                self.cursor
            } else {
                self.selected
            };
            let Some(i) = crate::core::typeahead::TypeAhead::find(
                &prefix,
                self.options.iter().map(String::as_str),
                from,
                |_| true,
            ) else {
                return (Outcome::Ignored, None);
            };
            if self.open {
                self.cursor = i;
                return (Outcome::Changed, None);
            }
            self.cursor = i;
            return (Outcome::Changed, self.choose(i));
        }
        if self.open {
            if let Some(nav) = self.nav.action(key) {
                return match nav {
                    Nav::Up => {
                        self.cursor = crate::core::nav::step_index(
                            self.cursor,
                            self.last() + 1,
                            -1,
                            self.wrap,
                        );
                        (Outcome::Changed, None)
                    }
                    Nav::Down => {
                        self.cursor = crate::core::nav::step_index(
                            self.cursor,
                            self.last() + 1,
                            1,
                            self.wrap,
                        );
                        (Outcome::Changed, None)
                    }
                    Nav::PageUp => {
                        self.cursor = self.cursor.saturating_sub(self.visible_rows.max(1));
                        (Outcome::Changed, None)
                    }
                    Nav::PageDown => {
                        self.cursor = (self.cursor + self.visible_rows.max(1)).min(self.last());
                        (Outcome::Changed, None)
                    }
                    Nav::Home => {
                        self.cursor = 0;
                        (Outcome::Changed, None)
                    }
                    Nav::End => {
                        self.cursor = self.last();
                        (Outcome::Changed, None)
                    }
                    Nav::Left | Nav::Right => (Outcome::Consumed, None),
                };
            }
            return match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.open = false;
                    (Outcome::Changed, self.choose(self.cursor))
                }
                KeyCode::Esc if self.dismiss.escape => {
                    self.open = false;
                    self.cursor = self.selected;
                    (Outcome::Changed, None)
                }
                _ => (Outcome::Consumed, None),
            };
        }
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            self.open_list();
            return (Outcome::Changed, None);
        }
        // the closed field: arrows only (letters belong to the page)
        let nav = self
            .nav
            .action(key)
            .filter(|_| !matches!(key.code, KeyCode::Char(_)));
        match nav {
            Some(Nav::Up | Nav::Down) if key.plain() && self.arrows == SelectArrows::Open => {
                self.open_list();
                (Outcome::Changed, None)
            }
            Some(Nav::Left | Nav::Right) if self.arrows == SelectArrows::Open => {
                (Outcome::Ignored, None)
            }
            Some(Nav::Up | Nav::Left) if key.plain() => {
                let n =
                    crate::core::nav::step_index(self.selected, self.options.len(), -1, self.wrap);
                (Outcome::Changed, self.choose(n))
            }
            Some(Nav::Down | Nav::Right) if key.plain() => {
                let n =
                    crate::core::nav::step_index(self.selected, self.options.len(), 1, self.wrap);
                (Outcome::Changed, self.choose(n))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.type_ahead = on;
        self
    }

    /// What arrows do while the list is closed.
    pub fn arrows(mut self, arrows: SelectArrows) -> Self {
        self.arrows = arrows;
        self
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Consumed, None);
        }
        if id == self.bar_id() {
            // the press already moved the list
            self.bar.release_track();
            return (Outcome::Consumed, None);
        }
        if id == self.id {
            // the release of a press that opened the list (or landed on
            // the label) leaves it as the press made it
            if let Some(p) = self.press.take()
                && (p.opened || p.label)
            {
                return (Outcome::Changed, None);
            }
            if self.open {
                self.open = false;
                self.cursor = self.selected;
            } else {
                self.open_list();
            }
            return (Outcome::Changed, None);
        }
        if id == backdrop_id(self.id) {
            self.press = None;
            return (self.on_click_outside(), None);
        }
        if let Some(i) = self.locate(id) {
            self.open = false;
            return (Outcome::Changed, self.choose(i));
        }
        (Outcome::Ignored, None)
    }

    /// Close without changing, whatever the [`Dismiss`] policy (focus moved
    /// away, the owner closes it).
    pub fn close(&mut self) -> Outcome {
        if self.open {
            self.open = false;
            self.cursor = self.selected;
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    /// See [`Self::close_on_blur`].
    pub fn close_on_blur(mut self, on: bool) -> Self {
        self.close_on_blur = on;
        self
    }

    /// Focus left the field (see [`crate::core::focus::Focus::take_changes`]):
    /// an open list closes per [`Self::close_on_blur`], so the next key
    /// never sees a stale `open`.
    pub fn on_blur(&mut self) -> Outcome {
        if self.close_on_blur {
            self.close()
        } else {
            Outcome::Ignored
        }
    }

    /// A click outside the open list: closes it when the policy allows,
    /// and is consumed either way while the list is open.
    pub fn on_click_outside(&mut self) -> Outcome {
        if !self.open {
            return Outcome::Ignored;
        }
        if self.dismiss.outside_click {
            self.close()
        } else {
            Outcome::Consumed
        }
    }

    /// Button down on one of the select's targets (`id` from
    /// [`Self::owns`]). On the closed field it opens the list
    /// ([`Self::press_drag`]), so a drag can carry on to an option; on the
    /// label it only marks the press ([`Self::label_hit`]); on the list's
    /// scrollbar it grabs the thumb or pages the track.
    pub fn on_press(&mut self, id: WidgetId, pos: Position) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        if id == self.bar_id() {
            if !self.open || self.bar_track.is_empty() {
                return Outcome::Ignored;
            }
            self.sync_bar();
            scrollbar::press_with(self.bar_track, pos, &mut self.bar, self.v_click);
            self.scroll = self.bar.offset;
            return Outcome::Changed;
        }
        if id != self.id {
            return Outcome::Ignored;
        }
        let label = self.label_hit && self.label_area.contains(pos);
        let opened = !label && self.press_drag && !self.open;
        if opened {
            self.open_list();
        }
        self.press = Some(Press {
            opened,
            label,
            origin: pos,
            moved: false,
        });
        if opened {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer moved with the button held since a press on `pressed`:
    /// on the scrollbar the thumb follows; from the field the list cursor
    /// follows the pointer over the options.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if self.disabled || !self.open {
            return Outcome::Ignored;
        }
        if pressed == self.bar_id() {
            if self.bar_track.is_empty() {
                return Outcome::Ignored;
            }
            self.sync_bar();
            scrollbar::drag(self.bar_track, pos, &mut self.bar);
            self.scroll = self.bar.offset;
            return Outcome::Changed;
        }
        if pressed != self.id || !self.press_drag {
            return Outcome::Ignored;
        }
        let row = self.row_at(pos);
        let Some(p) = self.press.as_mut() else {
            return Outcome::Ignored;
        };
        p.moved |= pos != p.origin;
        if let Some(i) = row {
            self.cursor = i;
        }
        Outcome::Changed
    }

    /// The button went up after a press on the field that the pointer
    /// carried elsewhere (the owner's pointer layer reports no click: the
    /// press and the release landed on different targets). Over an option
    /// it chooses that option; back on the field, or with no movement, the
    /// list stays open; anywhere else it closes unchanged.
    pub fn on_release(&mut self, pos: Position) -> (Outcome, Option<SelectEvent>) {
        self.bar.release_track();
        let Some(p) = self.press.take() else {
            return (Outcome::Ignored, None);
        };
        if self.disabled || !self.open || !p.moved || !self.press_drag {
            return (Outcome::Consumed, None);
        }
        if let Some(i) = self.row_at(pos) {
            self.open = false;
            return (Outcome::Changed, self.choose(i));
        }
        if self.area.contains(pos) {
            return (Outcome::Consumed, None);
        }
        (self.close(), None)
    }

    /// The option under `pos` in the open list.
    fn row_at(&self, pos: Position) -> Option<usize> {
        if !self.rows_area.contains(pos) {
            return None;
        }
        let i = self.scroll + (pos.y - self.rows_area.y) as usize;
        (i < self.options.len()).then_some(i)
    }

    /// Bring the scrollbar's state up to the list's window.
    fn sync_bar(&mut self) {
        self.bar.set_content(self.options.len());
        self.bar.set_viewport(self.visible_rows);
        self.bar.offset = self.scroll;
    }

    /// Wheel over the open list: scrolls the list under
    /// [`WheelMode::Viewport`] (default), moves the cursor under
    /// [`WheelMode::Cursor`].
    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if !self.open || self.disabled {
            return Outcome::Ignored;
        }
        if self.wheel == WheelMode::Viewport {
            let max = self.options.len().saturating_sub(self.visible_rows);
            let to = if delta < 0 {
                self.scroll.saturating_sub(delta.unsigned_abs() as usize)
            } else {
                (self.scroll + delta as usize).min(max)
            };
            if to == self.scroll {
                return Outcome::Consumed;
            }
            self.scroll = to;
            return Outcome::Changed;
        }
        let before = self.cursor;
        self.cursor = if delta < 0 {
            self.cursor.saturating_sub(delta.unsigned_abs() as usize)
        } else {
            (self.cursor + delta as usize).min(self.last())
        };
        if before == self.cursor {
            Outcome::Consumed
        } else {
            Outcome::Changed
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        self.v_click = self.track_click.unwrap_or(ctx.track_click);
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
            self.open = false;
        }
        // an open list that lost focus is closed by `on_blur`, never here
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(s.focused).bg(bg)
        };
        buf.set_string(area.x + 2, area.y, &self.label, label_style);
        self.label_area = Rect::ZERO;
        if self.label_hit && !self.label.is_empty() {
            let lw = crate::ui::text::width(&self.label).min(area.width.saturating_sub(2) as usize);
            self.label_area = Rect::new(area.x, area.y, 2 + lw as u16, 1);
            ctx.control(self.id, self.label_area, self.disabled);
        }
        if area.height < 2 {
            return;
        }
        let field = Rect::new(area.x, area.y + 1, area.width, 1);
        self.area = field;
        let fs = t.field_style(s);
        fill(buf, field, fs);
        buf.set_string(
            field.x,
            field.y,
            t.gutter_symbol(s),
            t.gutter(s, fs.bg.unwrap_or(bg), false),
        );
        let shown = self.mixed.as_deref().unwrap_or(self.value());
        let value = crate::ui::text::truncate(shown, field.width.saturating_sub(5) as usize);
        let vs = if self.mixed.is_some() {
            // a placeholder, not a value: the text field's placeholder ink
            fs.patch(t.placeholder(s))
        } else {
            fs
        };
        buf.set_string(field.x + 2, field.y, &value, vs);
        if !self.disabled && !self.open && self.mixed.is_none() {
            let cut = Rect::new(field.x + 2, field.y, field.width.saturating_sub(5), 1);
            crate::ui::reveal::truncated(ctx, self.reveal, self.id, cut, self.value());
        }
        buf.set_string(
            field.right().saturating_sub(2),
            field.y,
            if self.open {
                t.glyphs.chevron_open
            } else {
                t.glyphs.chevron_closed
            },
            fs.fg(if self.disabled {
                t.disabled
            } else {
                t.text_secondary
            }),
        );
        ctx.control(self.id, field, self.disabled);
        if area.height >= 3 && !self.help.is_empty() {
            buf.set_string(
                area.x + 2,
                area.y + 2,
                crate::ui::text::truncate(&self.help, area.width.saturating_sub(2) as usize),
                t.muted().bg(bg),
            );
        }
        if self.open {
            self.queue_list(field, buf, ctx);
        }
    }

    /// Place the open list and queue it as an overlay. Placement and the
    /// scroll window are resolved now, so key handling in the next event
    /// knows how many rows are visible.
    fn queue_list(&mut self, field: Rect, buf: &Buffer, ctx: &mut RenderCtx) {
        let screen = *buf.area();
        let rows = (self.options.len() as u16).clamp(1, self.max_rows);
        let (min_w, max_w) = self.popup_width;
        let w = field.width.clamp(min_w, max_w);
        let pa = place_with(screen, field, w, rows + 2, self.placement, self.popup_fit);
        let visible = pa.height.saturating_sub(2).max(1) as usize;
        self.visible_rows = visible;
        let inner = pa.intersection(screen).inner(Margin::new(1, 1));
        let overflows = self.options.len() > visible;
        // the scrollbar takes the right column of the rows
        let with_bar = overflows && inner.width > 1;
        let rows_w = inner.width.saturating_sub(u16::from(with_bar));
        self.rows_area = Rect::new(inner.x, inner.y, rows_w, inner.height);
        self.bar_track = if with_bar {
            Rect::new(inner.right() - 1, inner.y, 1, inner.height)
        } else {
            Rect::ZERO
        };
        // the pointer moving onto an option moves the cursor there: one
        // highlight, as in a macOS menu
        let hover = ctx.interaction.live_hover();
        if self.hover_moves_cursor && hover != self.last_hover {
            if let Some(i) = hover.and_then(|h| self.locate(h)) {
                self.cursor = i;
                self.shown_cursor = Some(i);
            }
            self.last_hover = hover;
        }
        // a cursor that moved comes into view; a list the wheel or the bar
        // scrolled stays where it was left
        if self.shown_cursor != Some(self.cursor) {
            if self.cursor < self.scroll {
                self.scroll = self.cursor;
            } else if self.cursor >= self.scroll + visible {
                self.scroll = self.cursor + 1 - visible;
            }
        }
        self.shown_cursor = Some(self.cursor);
        self.scroll = self.scroll.min(self.options.len().saturating_sub(visible));
        self.sync_bar();
        let bar_state = self.bar.clone();
        let bar_track = self.bar_track;
        let hover_tint = !self.hover_moves_cursor;
        let (id, cursor, selected, scroll, dismiss, cursor_style, reveal) = (
            self.id,
            self.cursor,
            // mixed values: no option is the current one
            if self.mixed.is_some() {
                usize::MAX
            } else {
                self.selected
            },
            self.scroll,
            self.dismiss,
            self.cursor_style,
            self.reveal,
        );
        let options: Vec<(WidgetId, String)> = self
            .options
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible)
            .map(|(i, o)| (self.option_id(i), o.clone()))
            .collect();
        let more_above = scroll > 0;
        let more_below = scroll + visible < self.options.len();
        ctx.overlay(move |buf, ctx| {
            let t = ctx.theme;
            let inner = surface_for(pa, buf, ctx, t, Some((id, dismiss)));
            // the list scrolls under the wheel like any container
            ctx.scrollable(id, inner);
            let rows_w = inner.width.saturating_sub(u16::from(!bar_track.is_empty()));
            for (row_i, (rid, opt)) in options.iter().enumerate() {
                let i = scroll + row_i;
                let y = inner.y + row_i as u16;
                if y >= inner.bottom() {
                    break;
                }
                let mut rs = ctx.state(*rid);
                rs.focused = i == cursor;
                rs.selected = i == selected;
                // the cursor follows the pointer: the hover is the cursor
                rs.hovered &= hover_tint;
                let cr = t.cursor_row(rs, t.surface_elevated, cursor_style);
                let st = cr.style;
                let row = Rect::new(inner.x, y, rows_w, 1);
                fill(buf, row, st);
                buf.set_string(row.x, y, cr.gutter, cr.gutter_style);
                if rs.selected {
                    // the chosen value is a fact, not a position: neutral
                    let ink = st.fg(t.row_ink(&cr, t.text_secondary));
                    buf.set_string(row.x + 1, y, t.glyphs.current, ink);
                }
                buf.set_string(
                    row.x + 3,
                    y,
                    crate::ui::text::truncate(opt, row.width.saturating_sub(3) as usize),
                    st,
                );
                ctx.clickable(*rid, row);
                let cut = Rect::new(row.x + 3, y, row.width.saturating_sub(3), 1);
                crate::ui::reveal::truncated(ctx, reveal, *rid, cut, opt);
            }
            if !bar_track.is_empty() {
                scrollbar::render_vertical(bar_track, buf, ctx, id, &bar_state, false);
            }
            // more options than rows: arrows on the frame say which way
            let edge = t.text_secondary;
            let x = pa.right().saturating_sub(3);
            if more_above {
                buf.set_string(
                    x,
                    pa.y,
                    t.glyphs.more_above,
                    ratatui::style::Style::new().fg(edge).bg(t.surface_elevated),
                );
            }
            if more_below {
                buf.set_string(
                    x,
                    pa.bottom().saturating_sub(1),
                    t.glyphs.more_below,
                    ratatui::style::Style::new().fg(edge).bg(t.surface_elevated),
                );
            }
        });
    }
}

impl crate::core::focus::BlurTarget for Select {
    fn blur_id(&self) -> WidgetId {
        self.id
    }
    fn blur(&mut self) -> Outcome {
        self.on_blur()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;

    #[test]
    fn rendering_without_focus_never_closes_the_list() {
        use crate::ui::ctx::Interaction;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut s = Select::new(WidgetId::of("r"), "S", &["a", "b"], 0);
        s.open_list();
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
        s.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, theme.canvas);
        assert!(s.open, "render is not a blur: on_blur closes it");
        assert_eq!(s.on_blur(), Outcome::Changed);
    }

    #[test]
    fn blur_closes_an_open_list_unless_configured_not_to() {
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b"], 0);
        assert_eq!(s.on_blur(), Outcome::Ignored, "closed: nothing to do");
        s.open_list();
        s.cursor = 1;
        assert_eq!(s.on_blur(), Outcome::Changed);
        assert!(!s.open);
        assert_eq!(s.selected, 0, "blur never changes the value");
        let mut keep = Select::new(WidgetId::of("k"), "K", &["a", "b"], 0).close_on_blur(false);
        keep.open_list();
        assert_eq!(keep.on_blur(), Outcome::Ignored);
        assert!(keep.open);
    }

    #[test]
    fn typing_picks_the_matching_option() {
        let k = |c| Key {
            code: KeyCode::Char(c),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut s =
            Select::new(WidgetId::of("s"), "S", &["Alpha", "Beta", "Gamma"], 0).type_ahead(true);
        assert_eq!(s.on_key(&k('g')).1, Some(SelectEvent::Changed(2)));
        assert_eq!(
            s.on_key(&k('q')).0,
            Outcome::Ignored,
            "no match: the page keeps it"
        );
        let mut off = Select::new(WidgetId::of("s"), "S", &["Alpha", "Beta"], 0);
        assert_eq!(off.on_key(&k('b')).1, None);
        assert_eq!(off.selected, 0);
    }

    #[test]
    fn closed_arrows_step_or_open_per_config() {
        let k = |code| Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let opts = ["a", "b", "c"];
        let mut step = Select::new(WidgetId::of("s"), "S", &opts, 0);
        assert_eq!(
            step.on_key(&k(KeyCode::Down)).1,
            Some(SelectEvent::Changed(1))
        );
        assert!(!step.open);
        let mut open = Select::new(WidgetId::of("s"), "S", &opts, 0).arrows(SelectArrows::Open);
        assert_eq!(open.on_key(&k(KeyCode::Down)).1, None);
        assert!(open.open, "the arrow opened the list");
        assert_eq!(open.selected, 0, "the value did not change unseen");
        let mut side = Select::new(WidgetId::of("s"), "S", &opts, 0).arrows(SelectArrows::Open);
        assert_eq!(side.on_key(&k(KeyCode::Right)).0, Outcome::Ignored);
    }
    use crate::ui::ctx::Interaction;

    fn draw(sel: &mut Select) -> Buffer {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            focus: Some(sel.id),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 16));
        sel.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, t.canvas);
        ctx.flush_overlays(&mut buf);
        buf
    }

    #[test]
    fn the_open_list_marks_its_cursor_with_a_fill_and_the_choice_neutrally() {
        let t = Theme::junie();
        let mut sel = Select::new(WidgetId::of("s"), "Mode", &["One", "Two", "Three"], 0);
        sel.open = true;
        sel.cursor = 1;
        let buf = draw(&mut sel);
        let bars = buf.content().iter().filter(|c| c.symbol() == "▎").count();
        assert_eq!(bars, 1, "only the field carries the focus bar");
        let chosen = (0..16)
            .find_map(|y| {
                (0..40)
                    .find(|&x| buf[(x, y)].symbol() == "›")
                    .map(|x| (x, y))
            })
            .expect("chosen marker");
        assert_eq!(
            buf[chosen].fg, t.text_secondary,
            "chosen is neutral, not accent"
        );
        // the cursor row (one below the chosen row) is the highlight fill
        let cursor = (chosen.0 + 2, chosen.1 + 1);
        assert_eq!(buf[cursor].bg, t.highlight);
        // the bar look stays available
        let mut barred = sel.clone().cursor_style(CursorStyle::Bar);
        let buf = draw(&mut barred);
        let bars = buf.content().iter().filter(|c| c.symbol() == "▎").count();
        assert_eq!(bars, 2);
    }

    #[test]
    fn hovering_a_cut_value_or_option_reveals_it() {
        let long = "a choice far too long for a thirty column field";
        let text = |buf: &Buffer| {
            (0..16)
                .map(|y| (0..40).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let mut sel = Select::new(WidgetId::of("s"), "Mode", &[long, "Short"], 0);
        assert!(!text(&draw(&mut sel)).contains(long));
        let t = Theme::junie();
        let hovered = |sel: &mut Select, hover| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover: Some(hover),
                focus: Some(sel.id),
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 60, 16));
            sel.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, t.canvas);
            ctx.flush_overlays(&mut buf);
            buf
        };
        let wide = |buf: &Buffer| {
            (0..16)
                .map(|y| (0..60).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let id = sel.id;
        assert!(
            wide(&hovered(&mut sel, id)).contains(long),
            "the field value"
        );
        sel.open = true;
        let opt = sel.option_id(0);
        let all = wide(&hovered(&mut sel, opt));
        assert!(all.contains(long), "the option: {all}");
    }

    #[test]
    fn a_mixed_field_shows_its_placeholder_until_an_option_is_chosen() {
        let t = Theme::junie();
        let row = |buf: &Buffer, y: u16| (0..30).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let mut sel =
            Select::new(WidgetId::of("s"), "Role", &["Admin", "Editor"], 0).mixed("— mixed —");
        let buf = draw(&mut sel);
        assert!(row(&buf, 1).contains("— mixed —"), "{}", row(&buf, 1));
        assert!(!row(&buf, 1).contains("Admin"));
        assert_ne!(buf[(2, 1)].fg, t.text_primary, "a placeholder is muted");
        // the open list marks no option as the current one
        sel.open = true;
        let buf = draw(&mut sel);
        assert!(!buf.content().iter().any(|c| c.symbol() == "›"));
        // choosing the option already under the cursor still changes
        let key = Key {
            code: KeyCode::Enter,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(sel.on_key(&key).1, Some(SelectEvent::Changed(0)));
        assert_eq!(sel.mixed, None);
        let buf = draw(&mut sel);
        assert!(row(&buf, 1).contains("Admin"));
        // a click on an option clears it as well
        sel.set_mixed(Some("— mixed —"));
        assert_eq!(
            sel.on_click(sel.option_id(1)).1,
            Some(SelectEvent::Changed(1))
        );
        assert_eq!(sel.mixed, None);
    }

    const MANY: [&str; 20] = [
        "o0", "o1", "o2", "o3", "o4", "o5", "o6", "o7", "o8", "o9", "o10", "o11", "o12", "o13",
        "o14", "o15", "o16", "o17", "o18", "o19",
    ];

    /// Draw `sel` (field at the top of a 40x24 screen) with the pointer
    /// over `hover`; the frame's hit regions come back with the buffer.
    fn draw_with(sel: &mut Select, hover: Option<WidgetId>) -> (Buffer, HitRegistry) {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            focus: Some(sel.id),
            hover,
            ..Default::default()
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 24));
        {
            let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
            sel.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, t.canvas);
            ctx.flush_overlays(&mut buf);
        }
        (buf, hits)
    }

    fn many() -> Select {
        Select::new(WidgetId::of("m"), "Many", &MANY, 0).max_rows(5)
    }

    #[test]
    fn the_wheel_scrolls_the_open_list_and_leaves_the_cursor() {
        let mut sel = many();
        sel.on_click(sel.id);
        draw_with(&mut sel, None);
        assert_eq!(sel.on_wheel(3), Outcome::Changed);
        assert_eq!(
            (sel.scroll, sel.cursor),
            (3, 0),
            "the view moves, not the choice"
        );
        draw_with(&mut sel, None);
        assert_eq!(sel.scroll, 3, "the next frame keeps the scrolled view");
        // the end stops it
        sel.on_wheel(30);
        assert_eq!(sel.scroll, 15);
        assert_eq!(sel.on_wheel(1), Outcome::Consumed);
        // a key move brings the cursor back into view
        sel.on_key(&Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        draw_with(&mut sel, None);
        assert_eq!((sel.cursor, sel.scroll), (1, 1));
        // the old behaviour stays available
        let mut by_cursor = many().wheel(WheelMode::Cursor);
        by_cursor.on_click(by_cursor.id);
        draw_with(&mut by_cursor, None);
        by_cursor.on_wheel(3);
        assert_eq!(by_cursor.cursor, 3);
    }

    #[test]
    fn hovering_an_option_moves_the_cursor_so_one_row_is_lit() {
        let t = Theme::junie();
        let mut sel = Select::new(WidgetId::of("s"), "Mode", &["One", "Two", "Three"], 0);
        sel.on_click(sel.id);
        draw_with(&mut sel, None);
        let (buf, _) = {
            let h = sel.option_id(2);
            draw_with(&mut sel, Some(h))
        };
        assert_eq!(sel.cursor, 2, "the cursor follows the pointer");
        let lit: std::collections::BTreeSet<u16> = (0..24)
            .filter(|&y| (0..40).any(|x| buf[(x, y)].bg == t.highlight))
            .collect();
        assert_eq!(
            lit.len(),
            1,
            "one highlighted row, not a cursor and a hover"
        );
        // a key moves the cursor away while the pointer rests: the key wins
        sel.on_key(&Key {
            code: KeyCode::Up,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        {
            let h = sel.option_id(2);
            draw_with(&mut sel, Some(h))
        };
        assert_eq!(sel.cursor, 1);
        // off: hover tints, the cursor stays
        let mut off = Select::new(WidgetId::of("o"), "Mode", &["One", "Two", "Three"], 0)
            .hover_moves_cursor(false);
        off.on_click(off.id);
        {
            let h = off.option_id(2);
            draw_with(&mut off, Some(h))
        };
        assert_eq!(off.cursor, 0);
    }

    #[test]
    fn the_open_list_has_a_draggable_scrollbar() {
        let mut sel = many().track_click(TrackClick::Page);
        sel.on_click(sel.id);
        let (_, hits) = draw_with(&mut sel, None);
        let bar = hits
            .bar(sel.bar_id())
            .expect("the list registers its scrollbar");
        assert_eq!(bar.owner, sel.id);
        assert!(sel.owns(sel.bar_id()));
        let track = bar.track;
        // the bare track below the thumb pages
        let bottom = Position::new(track.x, track.bottom() - 1);
        assert_eq!(sel.on_press(sel.bar_id(), bottom), Outcome::Changed);
        assert_eq!(sel.scroll, 5);
        assert_eq!(sel.on_click(sel.bar_id()).0, Outcome::Consumed);
        assert!(sel.open, "a click on the bar never closes the list");
        draw_with(&mut sel, None);
        assert_eq!(sel.scroll, 5);
        // grab the thumb and drag it to the bottom
        let (_, hits) = draw_with(&mut sel, None);
        let track = hits.bar(sel.bar_id()).unwrap().track;
        let thumb = (track.y..track.bottom())
            .find(|&y| {
                let mut s = ScrollState::default();
                s.set_content(20);
                s.set_viewport(5);
                s.offset = sel.scroll;
                let (start, len) = s.thumb(track.height as usize);
                (start..start + len).contains(&((y - track.y) as usize))
            })
            .unwrap();
        sel.on_press(sel.bar_id(), Position::new(track.x, thumb));
        sel.on_drag(sel.bar_id(), Position::new(track.x, track.bottom() + 3));
        assert_eq!(
            sel.scroll, 15,
            "dragging the thumb past the end scrolls to the end"
        );
        assert_eq!(sel.cursor, 0, "the bar moves the view, not the choice");
    }

    #[test]
    fn press_drag_release_chooses_in_one_gesture() {
        let mut sel = Select::new(WidgetId::of("p"), "Mode", &["One", "Two", "Three"], 0);
        draw_with(&mut sel, None);
        let field = Position::new(sel.area.x + 3, sel.area.y);
        assert_eq!(sel.on_press(sel.id, field), Outcome::Changed);
        assert!(sel.open, "the press opens the list");
        draw_with(&mut sel, None);
        let third = Position::new(sel.rows_area.x + 4, sel.rows_area.y + 2);
        sel.on_drag(sel.id, third);
        assert_eq!(sel.cursor, 2);
        assert_eq!(
            sel.on_release(third),
            (Outcome::Changed, Some(SelectEvent::Changed(2)))
        );
        assert!(!sel.open);
        assert_eq!(sel.selected, 2);

        // a press and release in place leaves the list open for a click
        sel.on_press(sel.id, field);
        draw_with(&mut sel, None);
        assert_eq!(sel.on_release(field).1, None);
        assert!(sel.open);
        // a click released on the field itself keeps it open as well
        let mut c = Select::new(WidgetId::of("c"), "Mode", &["One", "Two"], 0);
        draw_with(&mut c, None);
        c.on_press(c.id, Position::new(c.area.x + 3, c.area.y));
        c.on_click(c.id);
        assert!(c.open, "the click that opened the list does not close it");
        c.on_press(c.id, Position::new(c.area.x + 3, c.area.y));
        c.on_click(c.id);
        assert!(!c.open, "a second click on the field closes it");

        // dragged out to nowhere: closes unchanged
        let mut d = Select::new(WidgetId::of("d"), "Mode", &["One", "Two"], 1);
        draw_with(&mut d, None);
        d.on_press(d.id, Position::new(d.area.x + 3, d.area.y));
        draw_with(&mut d, None);
        d.on_drag(d.id, Position::new(39, 23));
        assert_eq!(d.on_release(Position::new(39, 23)).1, None);
        assert!(!d.open);
        assert_eq!(d.selected, 1);

        // off: the press does not open the list
        let mut off = Select::new(WidgetId::of("x"), "Mode", &["One", "Two"], 0).press_drag(false);
        draw_with(&mut off, None);
        off.on_press(off.id, Position::new(off.area.x + 3, off.area.y));
        assert!(!off.open);
    }

    #[test]
    fn clicking_the_label_focuses_without_opening() {
        let mut sel = Select::new(WidgetId::of("l"), "Mode", &["One", "Two"], 0);
        let (_, hits) = draw_with(&mut sel, None);
        let label = Position::new(3, 0);
        assert_eq!(
            hits.hit(label),
            Some(sel.id),
            "the label is the field's target"
        );
        assert_eq!(hits.hit(Position::new(20, 0)), None);
        sel.on_press(sel.id, label);
        sel.on_click(sel.id);
        assert!(!sel.open, "a label click focuses only");
        let mut off = Select::new(WidgetId::of("o"), "Mode", &["One", "Two"], 0).label_hit(false);
        let (_, hits) = draw_with(&mut off, None);
        assert_eq!(hits.hit(label), None);
    }
}
