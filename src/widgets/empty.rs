use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::RenderCtx;
use crate::widgets::button::Button;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyKind {
    #[default]
    Empty,

    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyAction {
    pub id: WidgetId,
    pub label: String,

    pub primary: bool,
}

impl EmptyAction {
    pub fn primary(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            primary: true,
        }
    }
    pub fn secondary(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            primary: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmptyState {
    pub title: String,
    pub hint: Option<String>,
    pub kind: EmptyKind,

    pub actions: Vec<EmptyAction>,
}

impl EmptyState {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            hint: None,
            kind: EmptyKind::Empty,
            actions: vec![],
        }
    }
    pub fn error(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            hint: None,
            kind: EmptyKind::Error,
            actions: vec![],
        }
    }

    pub fn actions(mut self, actions: Vec<EmptyAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn on_click(&self, id: WidgetId) -> Option<&EmptyAction> {
        self.actions.iter().find(|a| a.id == id)
    }

    pub fn on_key(
        &self,
        focus: WidgetId,
        key: &crate::core::event::Key,
    ) -> (crate::core::event::Outcome, Option<&EmptyAction>) {
        let Some(a) = self.actions.iter().find(|a| a.id == focus) else {
            return (crate::core::event::Outcome::Ignored, None);
        };
        let (o, act) = Button::secondary(a.id, &a.label).on_key(key);
        (o, act.then_some(a))
    }
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.to_owned());
        self
    }
}

pub fn render_ctx(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, e: &EmptyState, bg: Color) {
    let area = area.intersection(*buf.area());
    if e.actions.is_empty() || area.height < 4 {
        render(area, buf, ctx.theme, e, bg);
        return;
    }

    let text = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(2));
    render(text, buf, ctx.theme, e, bg);
    let hint_rows = e
        .hint
        .as_ref()
        .map(|h| crate::ui::text::wrap(h, area.width.saturating_sub(4).max(8) as usize).len() + 1)
        .unwrap_or(0) as u16;
    let total = 1 + hint_rows;
    let y = text.y + text.height.saturating_sub(total) / 2 + total + 1;
    if y >= area.bottom() {
        return;
    }
    let mut buttons: Vec<Button> = e
        .actions
        .iter()
        .map(|a| {
            if a.primary {
                Button::primary(a.id, &a.label)
            } else {
                Button::secondary(a.id, &a.label)
            }
        })
        .collect();
    let gap = ctx.theme.spacing.button_gap;
    let widths: Vec<u16> = buttons.iter().map(Button::width).collect();
    let total_w = widths.iter().sum::<u16>() + gap * widths.len().saturating_sub(1) as u16;
    let x = area.x + area.width.saturating_sub(total_w) / 2;
    let rects = crate::widgets::button::row_layout(
        Rect::new(x, y, total_w.min(area.width), 1),
        &widths,
        gap,
    );
    for (b, r) in buttons.iter_mut().zip(rects) {
        b.render(r, buf, ctx, bg);
    }
}

pub fn render(area: Rect, buf: &mut Buffer, t: &Theme, e: &EmptyState, bg: Color) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let hint_lines: Vec<String> = e
        .hint
        .as_ref()
        .map(|h| crate::ui::text::wrap(h, area.width.saturating_sub(4).max(8) as usize))
        .unwrap_or_default();
    let total = 1 + if hint_lines.is_empty() {
        0
    } else {
        hint_lines.len() + 1
    } as u16;
    let y0 = area.y + area.height.saturating_sub(total) / 2;
    let put = |buf: &mut Buffer, y: u16, s: &str, style: ratatui::style::Style| {
        if y >= area.bottom() {
            return;
        }
        let s = crate::ui::text::truncate(s, area.width as usize);
        let x = area.x + area.width.saturating_sub(crate::ui::text::width(&s) as u16) / 2;
        buf.set_string(x, y, &s, style.bg(bg));
    };
    match e.kind {
        EmptyKind::Empty => put(buf, y0, &e.title, t.muted()),
        EmptyKind::Error => {
            let title = format!("! {}", e.title);
            put(buf, y0, &title, t.error_fg());

            let s = crate::ui::text::truncate(&title, area.width as usize);
            let x = area.x + area.width.saturating_sub(crate::ui::text::width(&s) as u16) / 2;
            if y0 < area.bottom() {
                buf.set_string(
                    x,
                    y0,
                    "!",
                    t.error_fg()
                        .bg(bg)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );
            }
        }
    }
    for (i, l) in hint_lines.iter().enumerate() {
        put(buf, y0 + 2 + i as u16, l, t.faint());
    }
}

const SAMPLE_ID: WidgetId = WidgetId::of("empty-sample");

pub fn height(e: &EmptyState, width: u16) -> u16 {
    let hint = e
        .hint
        .as_ref()
        .map(|h| crate::ui::text::wrap(h, width.saturating_sub(4).max(8) as usize).len() + 1)
        .unwrap_or(0) as u16;
    let actions = if e.actions.is_empty() { 0 } else { 2 };
    1 + hint + actions
}

impl crate::ui::measure::Measure for EmptyState {
    fn measure(&self, _avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        let w = |s: &str| crate::ui::text::width(s) as u16;
        let title = w(&self.title) + if self.kind == EmptyKind::Error { 2 } else { 0 };
        let hint = self.hint.as_deref().map(|h| w(h) + 4).unwrap_or(0);
        let gap = ctx.theme.spacing.button_gap;
        let buttons = self
            .actions
            .iter()
            .map(|a| Button::secondary(a.id, &a.label).width())
            .sum::<u16>()
            + gap * self.actions.len().saturating_sub(1) as u16;
        let width = title.max(hint).max(buttons);
        ratatui::layout::Size::new(width, height(self, width))
    }

    fn min_width(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> u16 {
        let w = |s: &str| crate::ui::text::width(s) as u16;
        let full = self.measure(avail, ctx).width;
        let hint = self.hint.as_deref().map_or(0, |h| {
            let word = h.split_whitespace().map(w).max().unwrap_or(0);
            (w(h).min(HINT_MIN)).max(word) + 4
        });
        let title = w(&self.title) + if self.kind == EmptyKind::Error { 2 } else { 0 };
        let gap = ctx.theme.spacing.button_gap;
        let buttons = self
            .actions
            .iter()
            .map(|a| Button::secondary(a.id, &a.label).width())
            .sum::<u16>()
            + gap * self.actions.len().saturating_sub(1) as u16;
        title.max(hint).max(buttons).min(full)
    }
}

const HINT_MIN: u16 = 24;

impl crate::ui::specimen::Specimen for EmptyState {
    fn specimen_id(&self) -> WidgetId {
        self.actions.first().map_or(SAMPLE_ID, |a| a.id)
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        match state {
            Rest | Error | Overflow | Narrow => true,
            Hover | Pressed | Focused | FocusedHover => !self.actions.is_empty(),
            _ => false,
        }
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        height(self, 24) + u16::from(self.hint.is_some())
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
        let mut e = self.clone();
        match state {
            Error => e.kind = EmptyKind::Error,
            Overflow => e.title = format!("{} in a view far too narrow for it", e.title),
            _ => {}
        }
        render_ctx(area, buf, ctx, &e, bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Constraint;

    #[test]
    fn an_empty_state_at_its_measure_shows_everything() {
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let e = EmptyState::error("Could not load connections")
            .hint("Check the network, then retry the request")
            .actions(vec![
                EmptyAction::primary(WidgetId::of("retry"), "Retry"),
                EmptyAction::secondary(WidgetId::of("import"), "Import from file"),
            ]);
        let size = e.measure(ratatui::layout::Size::new(10, 3), &ctx);
        let area = Rect::new(0, 0, size.width, size.height);
        let mut buf = Buffer::empty(area);
        render_ctx(area, &mut buf, &mut ctx, &e, Color::Reset);
        let all = text(&buf);
        for part in [
            "! Could not load connections",
            "Check the network, then retry the request",
            "Retry",
            "Import from file",
        ] {
            assert!(all.contains(part), "{part:?} in {all:?}");
        }
    }

    #[test]
    fn an_empty_state_at_its_min_width_wraps_its_hint_whole() {
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let e = EmptyState::new("Nothing flagged")
            .hint("Run the second block: its unwrap() gets a warning");
        let zero = ratatui::layout::Size::ZERO;
        let full = e.measure(zero, &ctx).width;
        let min = e.min_width(zero, &ctx);
        assert!(min < full, "{min} < {full}");
        let area = Rect::new(0, 0, min, height(&e, min));
        let mut buf = Buffer::empty(area);
        render_ctx(area, &mut buf, &mut ctx, &e, Color::Reset);
        let all = text(&buf);
        for word in ["Nothing flagged", "Run", "second", "unwrap()", "warning"] {
            assert!(all.contains(word), "{word:?} in {all:?}");
        }
        assert!(!all.contains('…'), "{all:?}");
    }

    #[test]
    fn enter_or_space_on_a_focused_action_chooses_it() {
        use crate::core::event::{Key, Outcome};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let key = |code| Key {
            code,
            mods: KeyModifiers::NONE,
        };
        let new = WidgetId::of("new");
        let e = EmptyState::new("No connections")
            .actions(vec![EmptyAction::primary(new, "New connection")]);
        let (o, a) = e.on_key(new, &key(KeyCode::Enter));
        assert_eq!(o, Outcome::Changed);
        assert_eq!(a.map(|a| a.id), Some(new));
        assert_eq!(
            e.on_key(new, &key(KeyCode::Char(' '))).1.map(|a| a.id),
            Some(new)
        );
        assert!(e.on_key(new, &key(KeyCode::Char('x'))).1.is_none());
        assert_eq!(
            e.on_key(WidgetId::of("other"), &key(KeyCode::Enter)),
            (Outcome::Ignored, None)
        );
    }

    fn text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn actions_draw_as_buttons_below_the_text_and_take_clicks() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let id = WidgetId::of("new");
        let e = EmptyState::new("No connections")
            .hint("Ctrl+N creates one")
            .actions(vec![EmptyAction::primary(id, "New connection")]);
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        render_ctx(area, &mut buf, &mut ctx, &e, t.canvas);
        let s = text(&buf);
        assert!(
            s.contains("No connections") && s.contains("New connection"),
            "{s}"
        );
        let r = hits.area_of(id).expect("the action is clickable");
        assert!(r.y > 3, "the button sits below the text: {r:?}");
        assert_eq!(
            e.on_click(id).map(|a| a.label.as_str()),
            Some("New connection")
        );
    }

    #[test]
    fn lists_tables_and_pickers_take_a_full_empty_state() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let state = || EmptyState::new("No connections").hint("Press n to add one");
        let area = Rect::new(0, 0, 40, 10);

        let mut l = crate::widgets::list::ListBox::new(
            WidgetId::of("l"),
            vec![],
            crate::widgets::list::SelectMode::Single,
        )
        .empty(state());
        let mut buf = Buffer::empty(area);
        l.render(area, &mut buf, &mut ctx, t.canvas);
        assert!(text(&buf).contains("Press n to add one"));

        let cols = vec![crate::widgets::table::Column::new(
            "Name",
            Constraint::Fill(1),
        )];
        let mut tb =
            crate::widgets::table::DataTable::new(WidgetId::of("t"), cols, vec![]).empty(state());
        let mut buf = Buffer::empty(area);
        tb.render(area, &mut buf, &mut ctx, t.canvas);
        assert!(text(&buf).contains("Press n to add one"));

        let mut p = crate::widgets::picker::Picker::new(WidgetId::of("p"), "Pick").empty(state());
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        p.draw(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);

        assert!(text(&buf).contains("No connections"));
    }
}
