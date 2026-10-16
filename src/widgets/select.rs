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
use crate::theme::CursorStyle;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Placement, backdrop_id, place_with, surface_for};

/// What ↑/↓ (and ←/→) do on a closed select.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectArrows {
    /// Step the value without opening the list.
    Step,
    /// ↑/↓ open the list (macOS popup buttons); ←/→ are left to the page.
    /// Arrows never change the value unseen.
    Open,
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
    typeahead: crate::core::typeahead::TypeAhead,
    /// An open list closes (without changing the value) when the field
    /// loses focus ([`Self::on_blur`]). Default `true`.
    pub close_on_blur: bool,
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
            typeahead: Default::default(),
            close_on_blur: true,
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
        if let KeyCode::Char(c) = key.code
            && self.type_ahead
            && key.plain()
            && !c.is_whitespace()
            && !(self.open && matches!(c, 'j' | 'k'))
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
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.open_list();
                (Outcome::Changed, None)
            }
            KeyCode::Up | KeyCode::Down if key.plain() && self.arrows == SelectArrows::Open => {
                self.open_list();
                (Outcome::Changed, None)
            }
            KeyCode::Left | KeyCode::Right if self.arrows == SelectArrows::Open => {
                (Outcome::Ignored, None)
            }
            KeyCode::Up | KeyCode::Left if key.plain() => {
                let n = self.selected.saturating_sub(1);
                (Outcome::Changed, self.choose(n))
            }
            KeyCode::Down | KeyCode::Right if key.plain() => {
                let n = (self.selected + 1).min(self.options.len().saturating_sub(1));
                (Outcome::Changed, self.choose(n))
            }
            _ => (Outcome::Ignored, None),
        }
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
        if !ctx.interaction.has_focus(self.id) {
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
        let pa = place_with(screen, field, w, rows + 2, self.placement, self.popup_fit);
        let visible = pa.height.saturating_sub(2).max(1) as usize;
        self.visible_rows = visible;
        // keep the cursor inside the window
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + visible {
            self.scroll = self.cursor + 1 - visible;
        }
        self.scroll = self.scroll.min(self.options.len().saturating_sub(visible));
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
            for (row_i, (rid, opt)) in options.iter().enumerate() {
                let i = scroll + row_i;
                let y = inner.y + row_i as u16;
                if y >= inner.bottom() {
                    break;
                }
                let mut rs = ctx.state(*rid);
                rs.focused = i == cursor;
                rs.selected = i == selected;
                let cr = t.cursor_row(rs, t.surface_elevated, cursor_style);
                let st = cr.style;
                let row = Rect::new(inner.x, y, inner.width, 1);
                fill(buf, row, st);
                buf.set_string(row.x, y, cr.gutter, cr.gutter_style);
                if rs.selected {
                    // the chosen value is a fact, not a position: neutral
                    buf.set_string(row.x + 1, y, "›", st.fg(t.row_ink(&cr, t.text_secondary)));
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
}
