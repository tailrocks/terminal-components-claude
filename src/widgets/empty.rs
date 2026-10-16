//! Empty states: a quiet title, an optional hint, never a big glyph.

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
    /// A failed state: bold `!` in error tone before the title.
    Error,
}

/// A button under an empty state's text: the way out of the empty state
/// by pointer (WAI-ARIA and HIG: an empty view offers the action that
/// fills it, not only a shortcut to type).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyAction {
    pub id: WidgetId,
    pub label: String,
    /// Drawn as the primary button (one per state). Otherwise secondary.
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
    /// Buttons drawn under the text by [`render_ctx`]. Default none.
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
    /// See [`Self::actions`].
    pub fn actions(mut self, actions: Vec<EmptyAction>) -> Self {
        self.actions = actions;
        self
    }
    /// The action a completed click on `id` chose.
    pub fn on_click(&self, id: WidgetId) -> Option<&EmptyAction> {
        self.actions.iter().find(|a| a.id == id)
    }
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.to_owned());
        self
    }
}

/// Render centred in `area` with the [`EmptyState::actions`] as buttons
/// below the text, registered for hover, press and click.
pub fn render_ctx(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, e: &EmptyState, bg: Color) {
    let area = area.intersection(*buf.area());
    if e.actions.is_empty() || area.height < 4 {
        render(area, buf, ctx.theme, e, bg);
        return;
    }
    // the text above, one blank row, then the button row
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

/// Render centred in `area`.
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
            // the `!` is bold on its own
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Constraint;

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
        assert!(s.contains("No connections") && s.contains("New connection"), "{s}");
        let r = hits.area_of(id).expect("the action is clickable");
        assert!(r.y > 3, "the button sits below the text: {r:?}");
        assert_eq!(e.on_click(id).map(|a| a.label.as_str()), Some("New connection"));
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
        // a picker sizes to its rows: one row shows the title
        assert!(text(&buf).contains("No connections"));
    }
}
