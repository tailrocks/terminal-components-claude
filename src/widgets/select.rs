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
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Placement, backdrop_id, place, surface_for};

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
    /// First option row shown in the open list.
    pub scroll: usize,
    /// Option rows the open list showed in the last frame.
    visible_rows: usize,
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
            scroll: 0,
            visible_rows: Self::MAX_ROWS as usize,
        }
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
    pub fn popup_width(mut self, min: u16, max: u16) -> Self {
        self.popup_width = (min.max(1), max.max(min.max(1)));
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
        id == self.id || id == backdrop_id(self.id) || self.locate(id).is_some()
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
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        if self.open {
            return match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.cursor = self.cursor.saturating_sub(1);
                    (Outcome::Changed, None)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.cursor = (self.cursor + 1).min(self.last());
                    (Outcome::Changed, None)
                }
                KeyCode::PageUp => {
                    self.cursor = self.cursor.saturating_sub(self.visible_rows.max(1));
                    (Outcome::Changed, None)
                }
                KeyCode::PageDown => {
                    self.cursor = (self.cursor + self.visible_rows.max(1)).min(self.last());
                    (Outcome::Changed, None)
                }
                KeyCode::Home => {
                    self.cursor = 0;
                    (Outcome::Changed, None)
                }
                KeyCode::End => {
                    self.cursor = self.last();
                    (Outcome::Changed, None)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.open = false;
                    let changed = self.cursor != self.selected;
                    self.selected = self.cursor;
                    (
                        Outcome::Changed,
                        changed.then_some(SelectEvent::Changed(self.selected)),
                    )
                }
                KeyCode::Esc if self.dismiss.escape => {
                    self.open = false;
                    self.cursor = self.selected;
                    (Outcome::Changed, None)
                }
                _ => (Outcome::Consumed, None),
            };
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.open_list();
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Left if key.plain() => {
                let n = self.selected.saturating_sub(1);
                let changed = n != self.selected;
                self.selected = n;
                (Outcome::Changed, changed.then_some(SelectEvent::Changed(n)))
            }
            KeyCode::Down | KeyCode::Right if key.plain() => {
                let n = (self.selected + 1).min(self.options.len().saturating_sub(1));
                let changed = n != self.selected;
                self.selected = n;
                (Outcome::Changed, changed.then_some(SelectEvent::Changed(n)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<SelectEvent>) {
        if self.disabled {
            return (Outcome::Consumed, None);
        }
        if id == self.id {
            if self.open {
                self.open = false;
                self.cursor = self.selected;
            } else {
                self.open_list();
            }
            return (Outcome::Changed, None);
        }
        if id == backdrop_id(self.id) {
            return (self.on_click_outside(), None);
        }
        if let Some(i) = self.locate(id) {
            self.open = false;
            let changed = i != self.selected;
            self.selected = i;
            return (Outcome::Changed, changed.then_some(SelectEvent::Changed(i)));
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

    /// Wheel over the open list moves the cursor.
    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if !self.open || self.disabled {
            return Outcome::Ignored;
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
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
            self.open = false;
        }
        if !s.focused {
            self.open = false;
        }
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(s.focused).bg(bg)
        };
        buf.set_string(area.x + 2, area.y, &self.label, label_style);
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
        let value = crate::ui::text::truncate(self.value(), field.width.saturating_sub(5) as usize);
        buf.set_string(field.x + 2, field.y, &value, fs);
        buf.set_string(
            field.right().saturating_sub(2),
            field.y,
            if self.open { "▴" } else { "▾" },
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
        let pa = place(screen, field, w, rows + 2, self.placement);
        let visible = pa.height.saturating_sub(2).max(1) as usize;
        self.visible_rows = visible;
        // keep the cursor inside the window
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + visible {
            self.scroll = self.cursor + 1 - visible;
        }
        self.scroll = self.scroll.min(self.options.len().saturating_sub(visible));
        let (id, cursor, selected, scroll, dismiss) = (
            self.id,
            self.cursor,
            self.selected,
            self.scroll,
            self.dismiss,
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
            for (row_i, (rid, opt)) in options.iter().enumerate() {
                let i = scroll + row_i;
                let y = inner.y + row_i as u16;
                if y >= inner.bottom() {
                    break;
                }
                let mut rs = ctx.state(*rid);
                rs.focused = i == cursor;
                rs.selected = i == selected;
                let st = t.row(rs, t.surface_elevated);
                let row = Rect::new(inner.x, y, inner.width, 1);
                fill(buf, row, st);
                buf.set_string(
                    row.x,
                    y,
                    t.gutter_symbol(rs),
                    t.gutter(rs, st.bg.unwrap_or(t.surface_elevated), false),
                );
                if rs.selected {
                    buf.set_string(row.x + 1, y, "›", st.fg(t.accent));
                }
                buf.set_string(
                    row.x + 3,
                    y,
                    crate::ui::text::truncate(opt, row.width.saturating_sub(3) as usize),
                    st,
                );
                ctx.clickable(*rid, row);
            }
            // more options than rows: arrows on the frame say which way
            let edge = t.text_secondary;
            let x = pa.right().saturating_sub(3);
            if more_above {
                buf.set_string(
                    x,
                    pa.y,
                    "▴",
                    ratatui::style::Style::new().fg(edge).bg(t.surface_elevated),
                );
            }
            if more_below {
                buf.set_string(
                    x,
                    pa.bottom().saturating_sub(1),
                    "▾",
                    ratatui::style::Style::new().fg(edge).bg(t.surface_elevated),
                );
            }
        });
    }
}
