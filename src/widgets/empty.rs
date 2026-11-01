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
