use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::hit::Axis;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::PointerEvent;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{DismissReason, Layer, LayerCx, LayerOut, LayerSpec};
use junie_tui::ui::text::{truncate, width, wrap};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::scrollbar;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::ModalResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLine {
    pub key: String,
    pub value: String,
    pub tone: Tone,
    pub heading: bool,
}

impl TextLine {
    pub fn kv(key: &str, value: &str) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            tone: Tone::Secondary,
            heading: false,
        }
    }
    pub fn text(value: &str, tone: Tone) -> Self {
        Self {
            key: String::new(),
            value: value.into(),
            tone,
            heading: false,
        }
    }
    pub fn blank() -> Self {
        Self::text("", Tone::Normal)
    }
}

pub struct TextModal {
    pub id: WidgetId,
    pub title: String,
    pub subtitle: String,
    pub lines: Vec<TextLine>,
    pub scroll: ScrollState,
    pub width: u16,
    pub closed: bool,
    pub area: Rect,

    pub dismiss: junie_tui::ui::popup::Dismiss,
    scroll_area: Rect,
    wrapped: Vec<(String, String, Tone, bool)>,
    wrapped_for: u16,
}

impl TextModal {
    pub fn new(id: WidgetId, title: &str, lines: Vec<TextLine>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: String::new(),
            lines,
            scroll: ScrollState::default(),
            width: 72,
            closed: false,
            area: Rect::ZERO,
            dismiss: junie_tui::ui::popup::Dismiss::LIGHT,
            scroll_area: Rect::ZERO,
            wrapped: vec![],
            wrapped_for: 0,
        }
    }
    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    fn key_col(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !l.heading)
            .map(|l| width(&l.key))
            .max()
            .unwrap_or(0)
    }

    fn layout(&mut self, inner_w: u16) {
        if self.wrapped_for == inner_w {
            return;
        }
        self.wrapped_for = inner_w;
        let kc = self.key_col();
        let vw = (inner_w as usize)
            .saturating_sub(if kc > 0 { kc + 2 } else { 0 })
            .max(8);
        let mut out = vec![];
        for l in &self.lines {
            if l.heading {
                out.push((String::new(), l.value.clone(), l.tone, true));
                continue;
            }
            let parts = if l.value.is_empty() {
                vec![String::new()]
            } else {
                wrap(&l.value, vw)
            };
            for (i, p) in parts.into_iter().enumerate() {
                out.push((
                    if i == 0 { l.key.clone() } else { String::new() },
                    p,
                    l.tone,
                    false,
                ));
            }
        }
        self.wrapped = out;
        self.scroll.set_content(self.wrapped.len());
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.closed = true;
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.scroll.page_up();
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.scroll.page_down();
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.scroll.jump_end();
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;

        let width_ = self.width;
        let frame = move |h: u16| {
            junie_tui::ui::modal::Frame::new(width_, h)
                .min_size(30, 8)
                .chrome(1, 2)
                .margin(ratatui::layout::Margin::new(3, 2))
        };
        let inner_w = frame(0).area(screen).width.saturating_sub(6);
        self.layout(inner_w);
        let content_h = self.wrapped.len() as u16;
        let (area, inner) = frame(content_h + 6).render(screen, buf, ctx);
        self.area = area;
        let bg = t.surface_elevated;
        ctx.hits.register(self.id, area);
        ctx.ring.register(self.id);
        if inner.is_empty() {
            return;
        }
        buf.set_string(
            inner.x,
            inner.y,
            truncate(&self.title, inner.width as usize),
            t.title().bg(bg),
        );
        if !self.subtitle.is_empty() {
            let sw = width(&self.subtitle) as u16;
            let room = inner.width.saturating_sub(width(&self.title) as u16 + 3);
            if sw <= room {
                buf.set_string(
                    inner.right().saturating_sub(sw),
                    inner.y,
                    &self.subtitle,
                    t.muted().bg(bg),
                );
            }
        }
        let body = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );
        self.scroll_area = body;
        self.scroll.set_viewport(body.height as usize);
        ctx.scrollable(self.id, body);
        let kc = self.key_col() as u16;
        let has_sb = self.scroll.overflows();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = body.y + k as u16;
            let (key, value, tone, heading) = &self.wrapped[i];
            if *heading {
                buf.set_string(
                    body.x,
                    y,
                    truncate(value, body.width as usize),
                    t.faint().bg(bg),
                );
                continue;
            }
            if kc > 0 {
                buf.set_string(body.x, y, key, t.key_hint_key().bg(bg));
                let vx = body.x + kc + 2;
                buf.set_string(
                    vx,
                    y,
                    truncate(
                        value,
                        body.width.saturating_sub(kc + 2 + u16::from(has_sb)) as usize,
                    ),
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            } else {
                buf.set_string(
                    body.x,
                    y,
                    truncate(value, body.width.saturating_sub(u16::from(has_sb)) as usize),
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            }
        }
        if has_sb {
            scrollbar::render_with_fade(
                Rect::new(body.x, body.y, body.width, body.height),
                buf,
                ctx,
                self.id,
                &self.scroll,
                true,
            );
        }
    }
}

impl Layer<ModalResult> for TextModal {
    fn spec(&self) -> LayerSpec {
        LayerSpec::modal(self.dismiss).initial_focus(self.id)
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _: &()) {
        TextModal::render(self, screen, buf, ctx);
    }
    fn on_key(&mut self, key: &Key, _: &mut LayerCx<'_>) -> Outcome {
        TextModal::on_key(self, key)
    }

    fn pointer(&mut self, ev: &PointerEvent, _: &mut LayerCx<'_>) -> Outcome {
        if let Some((axis, delta)) = ev.wheel() {
            if axis == Axis::Vertical {
                self.scroll.scroll_by(delta as isize);
                return Outcome::Changed;
            }
            return Outcome::Consumed;
        }
        let bar = ev.id == scrollbar::id_for(self.id);
        let moved = match ev.phase() {
            Some(Phase::Click) if bar => {
                scrollbar::press(self.scroll_area, ev.pos(), &mut self.scroll)
            }
            Some(Phase::Drag) if bar => {
                scrollbar::drag(self.scroll_area, ev.pos(), &mut self.scroll)
            }
            Some(Phase::Click) => return Outcome::Changed,
            _ => return Outcome::Ignored,
        };
        if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }
    fn hints(&self, _: Option<WidgetId>) -> Vec<Hint> {
        vec![hint("↑↓", "Scroll"), hint("Esc", "Close")]
    }
    fn on_dismiss(&mut self, _: DismissReason) {
        self.closed = true;
    }
    fn take_result(&mut self) -> Option<LayerOut<ModalResult>> {
        if !self.closed {
            return None;
        }
        self.closed = false;
        Some(LayerOut::Close(ModalResult::Closed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use junie_tui::core::focus::FocusRing;
    use junie_tui::core::hit::HitRegistry;
    use junie_tui::theme::Theme;
    use junie_tui::ui::ctx::Interaction;

    #[test]
    fn the_text_modal_sits_in_the_shared_frame_where_it_always_did() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let screen = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(screen);
        let lines = (0..4)
            .map(|i| TextLine::kv("k", &format!("line {i}")))
            .collect();
        let mut m = TextModal::new(WidgetId::of("about"), "About", lines);
        m.render(screen, &mut buf, &mut ctx);

        assert_eq!(m.area, Rect::new(14, 9, 72, 10));
    }
}
