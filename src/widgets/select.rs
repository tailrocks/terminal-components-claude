use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::{Part, WidgetId};
use crate::core::keymap::Nav;
use crate::core::keymap::NavKeys;
use crate::core::keymap::unassigned_chord;
use crate::core::list::{Cursor, KeyStep, Moved, Navigable, RowSource};
use crate::core::scroll::{Scrollable, TrackClick};
use crate::theme::CursorStyle;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Placement, backdrop_id, place_with, surface_for};
use crate::widgets::option_list::OptionList;
use crate::widgets::scrollbar;

pub use crate::widgets::option_list::{SelectArrows, WheelMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabInOpen {
    #[default]
    Choose,

    Close,

    Swallow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Press {
    opened: bool,

    label: bool,
    origin: Position,

    moved: bool,
}

#[derive(Debug, Clone)]
pub struct Select {
    pub id: WidgetId,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub open: bool,
    pub disabled: bool,
    pub help: String,
    pub area: Rect,

    pub list: OptionList,

    pub reveal: Option<crate::ui::reveal::Reveal>,

    pub arrows: SelectArrows,

    pub tab_in_open: TabInOpen,

    pub mixed: Option<String>,

    pub type_ahead_precedence: Option<crate::core::list::TypeAheadPrecedence>,

    pub press_drag: bool,

    pub label_hit: bool,

    label_area: Rect,
    press: Option<Press>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectAction {
    Changed(usize),
}

impl Select {
    pub const HEIGHT: u16 = 3;

    pub const MAX_ROWS: u16 = 8;

    pub const POPUP_WIDTH: (u16, u16) = (12, 40);

    pub fn new(id: WidgetId, label: &str, options: &[&str], selected: usize) -> Self {
        let mut list = OptionList::new(id)
            .max_rows(Self::MAX_ROWS)
            .width(Self::POPUP_WIDTH.0, Self::POPUP_WIDTH.1)
            .clear_on_leave(false)
            .fade_edges(false);
        list.cursor = Cursor::new(selected);

        list.scroll.v.context = 0;
        list.scroll.v.viewport_len = Self::MAX_ROWS as usize;
        Self {
            id,
            label: label.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            selected,
            open: false,
            disabled: false,
            help: String::new(),
            area: Rect::ZERO,
            list,
            reveal: None,
            arrows: SelectArrows::Open,
            tab_in_open: TabInOpen::Choose,
            mixed: None,
            type_ahead_precedence: None,
            press_drag: true,
            label_hit: true,
            label_area: Rect::ZERO,
            press: None,
        }
    }

    pub fn wheel(mut self, mode: WheelMode) -> Self {
        self.list.wheel = mode;
        self
    }

    pub fn hover_moves_cursor(mut self, on: bool) -> Self {
        self.list.hover_moves_cursor = on;
        self
    }

    pub fn press_drag(mut self, on: bool) -> Self {
        self.press_drag = on;
        self
    }

    pub fn label_hit(mut self, on: bool) -> Self {
        self.label_hit = on;
        self
    }

    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.list.scroll.track_click = Some(click);
        self
    }

    pub fn bar_id(&self) -> WidgetId {
        scrollbar::id_for(self.id)
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.list.nav = nav;
        self
    }

    pub fn dismiss(mut self, d: Dismiss) -> Self {
        self.list.dismiss = d;
        self
    }

    pub fn max_rows(mut self, rows: u16) -> Self {
        self.list.max_rows = rows.max(1);
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.list.placement = placement;
        self
    }

    pub fn popup_fit(mut self, fit: crate::ui::popup::Place) -> Self {
        self.list.fit = fit;
        self
    }

    pub fn popup_width(mut self, min: u16, max: u16) -> Self {
        let min = min.max(1);
        self.list.width = (min, max.max(min));
        self
    }

    pub fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.list.look = cursor_style;
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

    pub fn mixed(mut self, placeholder: &str) -> Self {
        self.mixed = Some(placeholder.to_owned());
        self
    }

    pub fn set_mixed(&mut self, placeholder: Option<&str>) {
        self.mixed = placeholder.map(str::to_owned);
    }

    fn choose(&mut self, i: usize) -> Option<SelectAction> {
        let changed = i != self.selected || self.mixed.is_some();
        self.selected = i;
        self.mixed = None;
        changed.then_some(SelectAction::Changed(i))
    }

    pub fn value(&self) -> &str {
        self.options
            .get(self.selected)
            .map(String::as_str)
            .unwrap_or("")
    }
    pub fn option_id(&self, i: usize) -> WidgetId {
        self.list.row_id(i)
    }

    fn option_of(&self, id: WidgetId) -> Option<usize> {
        self.list.row_of(id, self.options.len())
    }

    pub(crate) fn contains_id(&self, id: WidgetId) -> bool {
        id == self.id
            || id == backdrop_id(self.id)
            || id == self.bar_id()
            || self.option_of(id).is_some()
    }

    fn value_cursor(&self) -> Cursor {
        Cursor::new(self.selected).wrap(self.list.cursor.wrap)
    }

    fn open_list(&mut self) {
        self.open = true;
        self.list.cursor.index = self.selected;

        let visible = self.list.scroll.viewport_len.max(1);
        self.list.scroll.offset = self.selected.saturating_sub(visible - 1);
    }

    fn shut(&mut self) {
        self.open = false;
        self.list.cursor.index = self.selected;
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<SelectAction>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        if unassigned_chord::<()>(key, &[]) {
            return (Outcome::Ignored, None);
        }

        let precedence = if self.open {
            crate::core::list::TypeAheadPrecedence::TypeAhead
        } else {
            self.type_ahead_precedence
                .unwrap_or_else(crate::core::list::TypeAheadPrecedence::current)
        };
        let step = KeyStep::ordered(key, &self.list.nav, self.list.type_ahead, precedence);
        if let Some(KeyStep::Type(c)) = step {
            if !self.open {
                self.list.cursor.index = self.selected;
            }
            let moved = self.list.type_char(c, &self.options[..]);
            if let Moved::To(i) = moved
                && !self.open
            {
                return (Outcome::Changed, self.choose(i));
            }
            return (moved.outcome(), None);
        }
        if self.open {
            if let Some(KeyStep::Nav(nav)) = step {
                return match self.list.nav_step(nav, &self.options[..]).index() {
                    Some(_) => (Outcome::Changed, None),

                    None => (Outcome::Consumed, None),
                };
            }
            return match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.open = false;
                    (Outcome::Changed, self.choose(self.list.cursor.index))
                }
                KeyCode::Esc if self.list.dismiss.escape => {
                    self.shut();
                    (Outcome::Changed, None)
                }

                KeyCode::Tab | KeyCode::BackTab => match self.tab_in_open {
                    TabInOpen::Choose => {
                        self.open = false;
                        (Outcome::Ignored, self.choose(self.list.cursor.index))
                    }
                    TabInOpen::Close => {
                        self.shut();
                        (Outcome::Ignored, None)
                    }
                    TabInOpen::Swallow => (Outcome::Consumed, None),
                },

                KeyCode::Char('?') | KeyCode::F(_) => {
                    self.shut();
                    (Outcome::Ignored, None)
                }
                _ => (Outcome::Consumed, None),
            };
        }
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            self.open_list();
            return (Outcome::Changed, None);
        }

        let nav = self
            .list
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
            Some(nav @ (Nav::Up | Nav::Left | Nav::Down | Nav::Right)) if key.plain() => {
                let delta = if matches!(nav, Nav::Up | Nav::Left) {
                    -1
                } else {
                    1
                };
                match self.value_cursor().step(delta, &self.options[..]).index() {
                    Some(n) => {
                        self.list.cursor.index = n;
                        (Outcome::Changed, self.choose(n))
                    }
                    None => (Outcome::Ignored, None),
                }
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.list.cursor.wrap = on;
        self
    }

    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.list = self.list.type_ahead_timeout(timeout);
        self
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.list.type_ahead = on;
        self
    }

    pub fn type_ahead_precedence(
        mut self,
        precedence: crate::core::list::TypeAheadPrecedence,
    ) -> Self {
        self.type_ahead_precedence = Some(precedence);
        self
    }

    pub fn tab_in_open(mut self, tab: TabInOpen) -> Self {
        self.tab_in_open = tab;
        self
    }

    pub fn arrows(mut self, arrows: SelectArrows) -> Self {
        self.arrows = arrows;
        self
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<SelectAction>) {
        if self.disabled {
            return (Outcome::Consumed, None);
        }
        if id == self.bar_id() {
            self.list.scroll.release();
            return (Outcome::Consumed, None);
        }
        if id == self.id {
            if let Some(p) = self.press.take()
                && (p.opened || p.label)
            {
                return (Outcome::Changed, None);
            }
            if self.open {
                self.shut();
            } else {
                self.open_list();
            }
            return (Outcome::Changed, None);
        }
        if id == backdrop_id(self.id) {
            self.press = None;
            return (self.on_click_outside(), None);
        }
        if let Some(i) = self.option_of(id) {
            self.open = false;
            return (Outcome::Changed, self.choose(i));
        }
        (Outcome::Ignored, None)
    }

    pub fn close(&mut self) -> Outcome {
        if self.open {
            self.shut();
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn close_on_blur(mut self, on: bool) -> Self {
        self.list.dismiss.focus_out = on;
        self
    }

    pub fn on_blur(&mut self) -> Outcome {
        if self.list.dismiss.focus_out {
            self.close()
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_click_outside(&mut self) -> Outcome {
        if !self.open {
            return Outcome::Ignored;
        }
        if self.list.dismiss.outside_click {
            self.close()
        } else {
            Outcome::Consumed
        }
    }

    pub fn on_press(&mut self, id: WidgetId, pos: Position) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        if id == self.bar_id() {
            if !self.open || self.list.scroll.v_track().is_empty() {
                return Outcome::Ignored;
            }
            return self.list.scroll.press(pos).outcome();
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

    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if self.disabled || !self.open {
            return Outcome::Ignored;
        }
        if pressed == self.bar_id() {
            if self.list.scroll.v_track().is_empty() {
                return Outcome::Ignored;
            }
            return self.list.scroll.drag(pos).outcome();
        }
        if pressed != self.id || !self.press_drag {
            return Outcome::Ignored;
        }
        let row = self.list.row_at(pos);
        let Some(p) = self.press.as_mut() else {
            return Outcome::Ignored;
        };
        p.moved |= pos != p.origin;
        if let Some(i) = row {
            self.list.cursor.index = i;
        }
        Outcome::Changed
    }

    pub fn on_release(&mut self, pos: Position) -> (Outcome, Option<SelectAction>) {
        self.list.scroll.release();
        let Some(p) = self.press.take() else {
            return (Outcome::Ignored, None);
        };
        if self.disabled || !self.open || !p.moved || !self.press_drag {
            return (Outcome::Consumed, None);
        }
        if let Some(i) = self.list.row_at(pos) {
            self.open = false;
            return (Outcome::Changed, self.choose(i));
        }
        if self.area.contains(pos) {
            return (Outcome::Consumed, None);
        }
        (self.close(), None)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        self.list.scroll.begin(ctx.track_click);
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
            self.open = false;
        }

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
            ctx.clickable(self.id, self.label_area);
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

    fn queue_list(&mut self, field: Rect, buf: &Buffer, ctx: &mut RenderCtx) {
        let screen = *buf.area();
        let rows = (self.options.len() as u16).clamp(1, self.list.max_rows);
        let (min_w, max_w) = self.list.width;
        let w = field.width.clamp(min_w, max_w);
        let pa = place_with(
            screen,
            field,
            w,
            rows + 2,
            self.list.placement,
            self.list.fit,
        );
        let inner = pa.intersection(screen).inner(Margin::new(1, 1));

        self.list.layout(inner, &self.options[..], ctx);

        self.list.chosen = self.mixed.is_none().then_some(self.selected);
        let list = self.list.clone();
        let options = self.options.clone();
        let (id, dismiss, reveal) = (self.id, self.list.dismiss, self.reveal);
        let scroll = list.scroll.offset;
        let visible = list.scroll.viewport_len;
        let more_above = scroll > 0;
        let more_below = scroll + visible < options.len();
        ctx.overlay(move |buf, ctx| {
            let t = ctx.theme;
            let inner = surface_for(pa, buf, ctx, t, Some((id, dismiss)));

            ctx.scrollable(id, inner);
            list.paint_rows(
                inner,
                t.surface_elevated,
                &options[..],
                buf,
                ctx,
                |i, row, buf, ctx| {
                    let opt = &options[i];
                    let area = row.area;
                    let st = row.style();
                    if row.state.selected {
                        let ink = st.fg(row.ink(ctx, t.text_secondary));
                        buf.set_string(area.x + 1, area.y, t.glyphs.current, ink);
                    }
                    buf.set_string(
                        area.x + 3,
                        area.y,
                        crate::ui::text::truncate(opt, area.width.saturating_sub(3) as usize),
                        st,
                    );
                    let cut = Rect::new(area.x + 3, area.y, area.width.saturating_sub(3), 1);
                    crate::ui::reveal::truncated(ctx, reveal, list.row_id(i), cut, opt);
                },
            );

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

impl RowSource for Select {
    fn len(&self) -> usize {
        self.options.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.options[i]
    }
}

impl Navigable for Select {
    fn cursor(&self) -> Cursor {
        if self.open {
            self.list.cursor
        } else {
            self.value_cursor()
        }
    }
}

impl crate::ui::specimen::Specimen for Select {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        match state {
            Rest | Hover | Pressed | Focused | FocusedHover | Disabled | Empty | Overflow
            | Narrow => true,
            Selected | Busy | Error | Editing | Loading => false,
        }
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        2
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState;
        let mut s = self.clone();
        s.open = false;
        s.help.clear();
        s.disabled |= state.visual().disabled;
        match state {
            SpecimenState::Empty => s.set_mixed(Some("Mixed")),
            SpecimenState::Overflow => {
                if let Some(o) = s.options.get_mut(s.selected) {
                    *o = format!("{o} and a value longer than the field");
                }
            }
            _ => {}
        }
        s.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for Select {
    type Action = SelectAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<SelectAction> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if let Some((axis, delta)) = ev.wheel() {
            if ev.part().part == Part::BACKDROP || !self.open || self.disabled {
                return Response::ignored();
            }
            return self.list.wheel(axis, delta).lift();
        }
        let (id, pos) = (ev.id, ev.pos());
        match ev.phase() {
            Some(Phase::Press) => self.on_press(id, pos).into(),
            Some(Phase::Drag) => self.on_drag(id, pos).into(),
            Some(Phase::Release) => self.on_release(pos).into(),
            Some(Phase::Click) => self.on_click(id).into(),
            _ => Response::ignored(),
        }
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
        s.list.cursor.index = 1;
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
        assert_eq!(s.on_key(&k('g')).1, Some(SelectAction::Changed(2)));
        assert_eq!(
            s.on_key(&k('q')).0,
            Outcome::Consumed,
            "no match: consumed, never half-typed to the page (plan §3)"
        );
        assert_eq!(s.selected, 2);
        let mut on = Select::new(WidgetId::of("s"), "S", &["Alpha", "Beta"], 0);
        assert_eq!(
            on.on_key(&k('b')).1,
            Some(SelectAction::Changed(1)),
            "on by default (Desktop profile, K34)"
        );
        let mut off = Select::new(WidgetId::of("s"), "S", &["Alpha", "Beta"], 0).type_ahead(false);
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

        let mut default = Select::new(WidgetId::of("s"), "S", &opts, 0);
        assert_eq!(default.on_key(&k(KeyCode::Down)).1, None);
        assert!(default.open, "the arrow opened the list");
        assert_eq!(default.selected, 0, "the value did not change unseen");
        let mut step = Select::new(WidgetId::of("s"), "S", &opts, 0).arrows(SelectArrows::Step);
        assert_eq!(
            step.on_key(&k(KeyCode::Down)).1,
            Some(SelectAction::Changed(1))
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
        sel.list.cursor.index = 1;
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

        let cursor = (chosen.0 + 2, chosen.1 + 1);
        assert_eq!(buf[cursor].bg, t.highlight);

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

        sel.open = true;
        let buf = draw(&mut sel);
        assert!(!buf.content().iter().any(|c| c.symbol() == "›"));

        let key = Key {
            code: KeyCode::Enter,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(sel.on_key(&key).1, Some(SelectAction::Changed(0)));
        assert_eq!(sel.mixed, None);
        let buf = draw(&mut sel);
        assert!(row(&buf, 1).contains("Admin"));

        sel.set_mixed(Some("— mixed —"));
        assert_eq!(
            sel.on_click(sel.option_id(1)).1,
            Some(SelectAction::Changed(1))
        );
        assert_eq!(sel.mixed, None);
    }

    const MANY: [&str; 20] = [
        "o0", "o1", "o2", "o3", "o4", "o5", "o6", "o7", "o8", "o9", "o10", "o11", "o12", "o13",
        "o14", "o15", "o16", "o17", "o18", "o19",
    ];

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

    fn wheel(sel: &mut Select, delta: i32) -> Outcome {
        use crate::core::hit::Axis;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let (_, hits) = draw_with(sel, None);
        let row = sel.option_id(sel.list.scroll.offset);
        let at = sel.list.rows_area();
        let pos = Position::new(at.x + 1, at.y);
        let none = ratatui::crossterm::event::KeyModifiers::NONE;
        let ev = PointerEvent::wheel_on(&hits, row, Axis::Vertical, delta, pos, none);
        sel.pointer(&ev).outcome()
    }

    fn many() -> Select {
        Select::new(WidgetId::of("m"), "Many", &MANY, 0).max_rows(5)
    }

    #[test]
    fn the_wheel_scrolls_the_open_list_and_leaves_the_cursor() {
        let mut sel = many();
        sel.on_click(sel.id);
        draw_with(&mut sel, None);
        assert_eq!(wheel(&mut sel, 3), Outcome::Changed);
        assert_eq!(
            (sel.list.scroll.offset, sel.list.cursor.index),
            (3, 0),
            "the view moves, not the choice"
        );
        draw_with(&mut sel, None);
        assert_eq!(
            sel.list.scroll.offset, 3,
            "the next frame keeps the scrolled view"
        );

        for _ in 0..10 {
            wheel(&mut sel, 30);
        }
        assert_eq!(sel.list.scroll.offset, 15);
        assert_eq!(wheel(&mut sel, 1), Outcome::Consumed);

        sel.on_key(&Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        draw_with(&mut sel, None);
        assert_eq!((sel.list.cursor.index, sel.list.scroll.offset), (1, 1));

        let mut by_cursor = many().wheel(WheelMode::Cursor);
        by_cursor.on_click(by_cursor.id);
        draw_with(&mut by_cursor, None);
        wheel(&mut by_cursor, 3);
        assert_eq!(by_cursor.list.cursor.index, 3);
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
        assert_eq!(sel.list.cursor.index, 2, "the cursor follows the pointer");
        let lit: std::collections::BTreeSet<u16> = (0..24)
            .filter(|&y| (0..40).any(|x| buf[(x, y)].bg == t.highlight))
            .collect();
        assert_eq!(
            lit.len(),
            1,
            "one highlighted row, not a cursor and a hover"
        );

        sel.on_key(&Key {
            code: KeyCode::Up,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        {
            let h = sel.option_id(2);
            draw_with(&mut sel, Some(h))
        };
        assert_eq!(sel.list.cursor.index, 1);

        let mut off = Select::new(WidgetId::of("o"), "Mode", &["One", "Two", "Three"], 0)
            .hover_moves_cursor(false);
        off.on_click(off.id);
        {
            let h = off.option_id(2);
            draw_with(&mut off, Some(h))
        };
        assert_eq!(off.list.cursor.index, 0);
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
        let track = bar.track;

        let bottom = Position::new(track.x, track.bottom() - 1);
        assert_eq!(sel.on_press(sel.bar_id(), bottom), Outcome::Changed);
        assert_eq!(sel.list.scroll.offset, 5);
        assert_eq!(sel.on_click(sel.bar_id()).0, Outcome::Consumed);
        assert!(sel.open, "a click on the bar never closes the list");
        draw_with(&mut sel, None);
        assert_eq!(sel.list.scroll.offset, 5);

        let (_, hits) = draw_with(&mut sel, None);
        let track = hits.bar(sel.bar_id()).unwrap().track;
        let thumb = (track.y..track.bottom())
            .find(|&y| {
                let mut s = crate::core::scroll::ScrollState::default();
                s.set_content(20);
                s.set_viewport(5);
                s.offset = sel.list.scroll.offset;
                let (start, len) = s.thumb(track.height as usize);
                (start..start + len).contains(&((y - track.y) as usize))
            })
            .unwrap();
        sel.on_press(sel.bar_id(), Position::new(track.x, thumb));
        sel.on_drag(sel.bar_id(), Position::new(track.x, track.bottom() + 3));
        assert_eq!(
            sel.list.scroll.offset, 15,
            "dragging the thumb past the end scrolls to the end"
        );
        assert_eq!(
            sel.list.cursor.index, 0,
            "the bar moves the view, not the choice"
        );
    }

    #[test]
    fn press_drag_release_chooses_in_one_gesture() {
        let mut sel = Select::new(WidgetId::of("p"), "Mode", &["One", "Two", "Three"], 0);
        draw_with(&mut sel, None);
        let field = Position::new(sel.area.x + 3, sel.area.y);
        assert_eq!(sel.on_press(sel.id, field), Outcome::Changed);
        assert!(sel.open, "the press opens the list");
        draw_with(&mut sel, None);
        let third = Position::new(sel.list.rows_area().x + 4, sel.list.rows_area().y + 2);
        sel.on_drag(sel.id, third);
        assert_eq!(sel.list.cursor.index, 2);
        assert_eq!(
            sel.on_release(third),
            (Outcome::Changed, Some(SelectAction::Changed(2)))
        );
        assert!(!sel.open);
        assert_eq!(sel.selected, 2);

        sel.on_press(sel.id, field);
        draw_with(&mut sel, None);
        assert_eq!(sel.on_release(field).1, None);
        assert!(sel.open);

        let mut c = Select::new(WidgetId::of("c"), "Mode", &["One", "Two"], 0);
        draw_with(&mut c, None);
        c.on_press(c.id, Position::new(c.area.x + 3, c.area.y));
        c.on_click(c.id);
        assert!(c.open, "the click that opened the list does not close it");
        c.on_press(c.id, Position::new(c.area.x + 3, c.area.y));
        c.on_click(c.id);
        assert!(!c.open, "a second click on the field closes it");

        let mut d = Select::new(WidgetId::of("d"), "Mode", &["One", "Two"], 1);
        draw_with(&mut d, None);
        d.on_press(d.id, Position::new(d.area.x + 3, d.area.y));
        draw_with(&mut d, None);
        d.on_drag(d.id, Position::new(39, 23));
        assert_eq!(d.on_release(Position::new(39, 23)).1, None);
        assert!(!d.open);
        assert_eq!(d.selected, 1);

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

    #[test]
    fn tab_in_an_open_list_chooses_and_moves_on() {
        let k = |code| Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut s = Select::new(WidgetId::of("s"), "S", &["a", "b", "c"], 0);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&k(KeyCode::Down));
        assert_eq!(
            s.on_key(&k(KeyCode::Tab)),
            (Outcome::Ignored, Some(SelectAction::Changed(1)))
        );
        assert!(!s.open);
        assert_eq!(s.selected, 1);
        s.on_key(&k(KeyCode::Enter));
        s.on_key(&k(KeyCode::Down));
        assert_eq!(s.on_key(&k(KeyCode::Char('?'))), (Outcome::Ignored, None));
        assert!(!s.open, "help closes the list");
        assert_eq!(s.selected, 1, "without choosing");
        let mut close =
            Select::new(WidgetId::of("s"), "S", &["a", "b"], 0).tab_in_open(TabInOpen::Close);
        close.on_key(&k(KeyCode::Enter));
        close.on_key(&k(KeyCode::Down));
        assert_eq!(close.on_key(&k(KeyCode::BackTab)), (Outcome::Ignored, None));
        assert_eq!((close.open, close.selected), (false, 0));
        let mut keep =
            Select::new(WidgetId::of("s"), "S", &["a", "b"], 0).tab_in_open(TabInOpen::Swallow);
        keep.on_key(&k(KeyCode::Enter));
        assert_eq!(keep.on_key(&k(KeyCode::Tab)), (Outcome::Consumed, None));
        assert!(keep.open);
    }
}
