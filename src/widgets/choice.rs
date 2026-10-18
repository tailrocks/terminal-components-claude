use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Nav, NavKeys};
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::ui::ctx::RenderCtx;
use crate::ui::row::{RowLook, RowUi};

fn row_part(buf: &mut Buffer, row: Rect, offset: u16, text: &str, style: Style) {
    if offset < row.width {
        buf.set_stringn(
            row.x + offset,
            row.y,
            text,
            usize::from(row.width - offset),
            style,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckHit {
    #[default]
    Label,

    Row,
}

#[derive(Debug, Clone)]
pub struct Checkbox {
    pub id: WidgetId,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,

    pub mixed: bool,

    pub hit: CheckHit,
    pub area: Rect,
}

impl Checkbox {
    pub fn new(id: WidgetId, label: &str, checked: bool) -> Self {
        Self {
            id,
            label: label.to_owned(),
            checked,
            disabled: false,
            mixed: false,
            hit: CheckHit::Label,
            area: Rect::ZERO,
        }
    }

    pub fn hit(mut self, hit: CheckHit) -> Self {
        self.hit = hit;
        self
    }

    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
        self
    }

    fn toggle(&mut self) {
        self.checked = self.mixed || !self.checked;
        self.mixed = false;
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        if key.is_char(' ') || key.is(KeyCode::Enter) {
            self.toggle();
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_click(&mut self) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.toggle();
        Outcome::Changed
    }

    pub fn width(&self) -> u16 {
        crate::ui::text::width(&self.label) as u16 + 6
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        let area = Rect::new(area.x, area.y, area.width, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        s.selected = false;
        if self.disabled {
            s.hovered = false;
        }
        let st = RowUi::paint(ctx, buf, area, s, RowLook::Bar, bg).style();
        let g = &t.glyphs;
        let mark = match (area.width < 4, self.checked, self.mixed) {
            (true, _, true) => g.dash,
            (false, _, true) => g.check_mixed,
            (true, true, _) => g.tick,
            (true, false, _) => g.check_off_narrow,
            (false, true, _) => g.check_on,
            (false, false, _) => g.check_off,
        };
        let mark_style = if self.disabled {
            st
        } else if self.checked || self.mixed {
            st.fg(t.chosen_in(s))
        } else {
            st.fg(t.text_muted)
        };
        row_part(buf, area, 1, mark, mark_style);
        let label = crate::ui::text::truncate(&self.label, area.width.saturating_sub(6) as usize);
        row_part(buf, area, 5, &label, st);

        let target = match self.hit {
            CheckHit::Row => area,
            CheckHit::Label => {
                let w = 6 + crate::ui::text::width(&label) as u16;
                Rect::new(area.x, area.y, w.min(area.width), area.height)
            }
        };
        ctx.control(self.id, target, self.disabled);
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroup {
    pub id: WidgetId,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub cursor: usize,
    pub disabled: bool,
    pub areas: Vec<Rect>,

    pub nav: NavKeys,
}

impl RadioGroup {
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn new(id: WidgetId, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            id,
            label: label.to_owned(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            selected,
            cursor: selected,
            disabled: false,
            areas: vec![],
            nav: NavKeys::default(),
        }
    }

    pub fn height(&self) -> u16 {
        self.options.len().saturating_add(1).min(u16::MAX as usize) as u16
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        let to = match self.nav.action(key) {
            Some(nav @ (Nav::Up | Nav::Down | Nav::Home | Nav::End)) if key.plain() => {
                self.cursor().apply(nav, self, 1).index()
            }
            _ => None,
        };
        if let Some(to) = to {
            self.cursor = to;
            self.selected = self.cursor;
            return Outcome::Changed;
        }
        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter if key.plain() => {
                self.selected = self.cursor;
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, index: usize) -> Outcome {
        if self.disabled || index >= self.options.len() {
            return Outcome::Consumed;
        }
        self.cursor = index;
        self.selected = index;
        Outcome::Changed
    }

    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::option_part(i))
    }

    pub fn option_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    pub fn width(&self) -> u16 {
        let w = |t: &str| crate::ui::text::width(t) as u16;
        let options = self.options.iter().map(|o| w(o) + 6).max().unwrap_or(0);
        (w(&self.label) + 2).max(options)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.areas.clear();
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        let label_style = if self.disabled {
            t.faint().bg(bg)
        } else {
            t.label(focused).bg(bg)
        };
        row_part(
            buf,
            area,
            2,
            &crate::ui::text::truncate(&self.label, area.width.saturating_sub(2) as usize),
            label_style,
        );
        for (i, opt) in self
            .options
            .iter()
            .take(area.height.saturating_sub(1) as usize)
            .enumerate()
        {
            let y = area.y + 1 + i as u16;
            let row = Rect::new(area.x, y, area.width, 1);
            self.areas.push(row);
            let mut s = ctx.child_state(self.id, self.option_id(i), i == self.cursor);
            s.focused = focused && i == self.cursor;
            s.disabled = self.disabled;
            let st = RowUi::paint(ctx, buf, row, s, RowLook::Bar, bg).style();
            let on = i == self.selected;
            let g = &t.glyphs;
            let mark = match (row.width < 4, on) {
                (true, true) => g.dot_on,
                (true, false) => g.dot_off,
                (false, true) => g.radio_on,
                (false, false) => g.radio_off,
            };
            let ms = if self.disabled {
                st
            } else if on {
                st.fg(ctx.chosen(focused))
            } else {
                st.fg(t.text_muted)
            };
            row_part(buf, row, 1, mark, ms);
            row_part(
                buf,
                row,
                5,
                &crate::ui::text::truncate(opt, row.width.saturating_sub(5) as usize),
                st,
            );
            ctx.register_part(self.id, Self::option_part(i), row);
        }

        if !ctx.inert && !self.disabled && !self.areas.is_empty() {
            ctx.ring.register(self.id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toggle {
    pub id: WidgetId,
    pub label: String,
    pub on: bool,
    pub disabled: bool,
    pub area: Rect,
}

impl Toggle {
    pub fn new(id: WidgetId, label: &str, on: bool) -> Self {
        Self {
            id,
            label: label.to_owned(),
            on,
            disabled: false,
            area: Rect::ZERO,
        }
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        if key.is_char(' ') || key.is(KeyCode::Enter) {
            self.on = !self.on;
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    pub fn on_click(&mut self) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        self.on = !self.on;
        Outcome::Changed
    }

    pub fn width(&self) -> u16 {
        crate::ui::text::width(&self.label) as u16 + 10
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        let area = Rect::new(area.x, area.y, area.width, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);
        s.disabled = self.disabled;
        if self.disabled {
            s.hovered = false;
        }
        let st = RowUi::paint(ctx, buf, area, s, RowLook::Bar, bg).style();
        let g = &t.glyphs;
        let (sw, ss) = if self.disabled {
            (if self.on { g.toggle_on } else { g.toggle_off }, st)
        } else if self.on {
            (g.toggle_on, st.fg(t.chosen_in(s)))
        } else {
            (g.toggle_off, st.fg(t.text_muted))
        };
        let sw = if area.width < 4 {
            if self.on { g.dot_on } else { g.dot_off }
        } else {
            sw
        };
        row_part(buf, area, 1, sw, ss);
        row_part(
            buf,
            area,
            5,
            &crate::ui::text::truncate(&self.label, area.width.saturating_sub(5) as usize),
            st,
        );
        let state = if self.on { "on" } else { "off" };
        let state_offset = 6usize.saturating_add(crate::ui::text::width(&self.label));
        if state_offset.saturating_add(3) < usize::from(area.width) {
            row_part(
                buf,
                area,
                state_offset as u16,
                state,
                st.fg(if self.disabled {
                    t.disabled
                } else {
                    t.text_muted
                }),
            );
        }
        ctx.control(self.id, area, self.disabled);
    }
}

impl RowSource for RadioGroup {
    fn len(&self) -> usize {
        self.options.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.options[i]
    }
}

impl Navigable for RadioGroup {
    fn cursor(&self) -> Cursor {
        Cursor::new(self.cursor)
    }
}

impl crate::ui::specimen::Specimen for Checkbox {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover | Pressed | Focused | FocusedHover | Disabled | Overflow
        )
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut c = self.clone();
        c.disabled |= state.visual().disabled;
        if state == crate::ui::specimen::SpecimenState::Overflow {
            c.label = format!("{} with a long label", c.label);
        }
        c.render(area, buf, ctx, bg);
    }
}

impl crate::ui::specimen::Specimen for Toggle {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover | Pressed | Focused | FocusedHover | Disabled | Overflow
        )
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut t = self.clone();
        t.disabled |= state.visual().disabled;
        if state == crate::ui::specimen::SpecimenState::Overflow {
            t.label = format!("{} with a long label", t.label);
        }
        t.render(area, buf, ctx, bg);
    }
}

impl crate::ui::specimen::Specimen for RadioGroup {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Hover | Focused | FocusedHover | Disabled)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        self.height()
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        let mut g = self.clone();
        let v = state.visual();
        g.disabled |= v.disabled;

        if v.hovered {
            ctx.interaction.hover = Some(g.option_id(g.cursor));
        }
        g.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for Checkbox {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        if !ev.is(crate::core::intent::Phase::Click) {
            return crate::core::response::Response::ignored();
        }
        self.on_click().into()
    }
}

impl crate::core::pointer::MouseTarget for RadioGroup {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        match ev.index_of(Part::ROW) {
            Some(i) if ev.is(crate::core::intent::Phase::Click) => self.on_click(i).into(),
            _ => crate::core::response::Response::ignored(),
        }
    }
}

impl crate::core::pointer::MouseTarget for Toggle {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        if !ev.is(crate::core::intent::Phase::Click) {
            return crate::core::response::Response::ignored();
        }
        self.on_click().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_toggle_at_its_width_shows_its_label_and_state() {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut t = Toggle::new(WidgetId::of("t"), "Auto-merge approved PRs", false);
        let area = Rect::new(0, 0, t.width(), 1);
        let mut buf = Buffer::empty(area);
        t.render(area, &mut buf, &mut ctx, theme.canvas);
        let row: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            row.contains("Auto-merge approved PRs") && row.contains("off"),
            "{row:?}"
        );
    }

    #[test]
    fn a_checkbox_and_a_radio_group_at_their_widths_show_every_label() {
        let theme = Theme::junie();
        let rows = |w: u16, h: u16, draw: &mut dyn FnMut(Rect, &mut Buffer, &mut RenderCtx)| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            draw(area, &mut buf, &mut ctx);
            (0..h)
                .map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let mut cb = Checkbox::new(WidgetId::of("c"), "Run tests before opening a PR", true);
        let w = cb.width();
        let text = rows(w, 1, &mut |a, b, c| cb.render(a, b, c, theme.canvas));
        assert!(text.contains("Run tests before opening a PR"), "{text}");
        let text = rows(w - 2, 1, &mut |a, b, c| cb.render(a, b, c, theme.canvas));
        assert!(!text.contains("Run tests before opening a PR"), "{text}");
        let mut rg = RadioGroup::new(WidgetId::of("r"), "Mode", &["Fast", "Thoroughly"], 0);
        let (w, h) = (rg.width(), rg.height());
        let text = rows(w, h, &mut |a, b, c| rg.render(a, b, c, theme.canvas));
        assert!(
            text.contains("Thoroughly") && text.contains("Mode"),
            "{text}"
        );
        let text = rows(w - 2, h, &mut |a, b, c| rg.render(a, b, c, theme.canvas));
        assert!(!text.contains("Thoroughly"), "{text}");
    }

    fn mark(cb: &mut Checkbox) -> String {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        cb.render(Rect::new(0, 0, 20, 1), &mut buf, &mut ctx, t.canvas);
        (1..4).map(|x| buf[(x, 0)].symbol()).collect()
    }

    #[test]
    fn a_mixed_checkbox_shows_a_dash_and_toggles_to_checked() {
        let mut cb = Checkbox::new(WidgetId::of("c"), "On call", false).mixed(true);
        assert_eq!(mark(&mut cb), "[–]");
        assert_eq!(cb.on_click(), Outcome::Changed);
        assert!(cb.checked && !cb.mixed, "mixed → checked, as on macOS");
        assert_eq!(mark(&mut cb), "[✓]");
        let mut cb = Checkbox::new(WidgetId::of("c"), "On call", true).mixed(true);
        let space = Key {
            code: KeyCode::Char(' '),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        cb.on_key(&space);
        assert!(cb.checked && !cb.mixed);
        cb.on_key(&space);
        assert!(!cb.checked);
    }

    #[test]
    fn only_the_box_and_label_take_clicks_by_default() {
        let t = Theme::junie();
        let hit_at = |cb: &mut Checkbox, x: u16| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
            cb.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
            drop(ctx);
            hits.hit(ratatui::layout::Position::new(x, 0))
        };
        let id = WidgetId::of("c");
        let mut cb = Checkbox::new(id, "On call", false);
        assert_eq!(hit_at(&mut cb, 2), Some(id), "the box");
        assert_eq!(hit_at(&mut cb, 6), Some(id), "the label");
        assert_eq!(hit_at(&mut cb, 12), Some(id), "the padding cell");
        assert_eq!(hit_at(&mut cb, 30), None, "blank space");
        let mut row = cb.clone().hit(CheckHit::Row);
        assert_eq!(hit_at(&mut row, 30), Some(id), "the whole row on request");
    }
}
