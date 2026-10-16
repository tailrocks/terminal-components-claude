//! A question with radio options and buttons (a mount destination, the
//! branches of a dirty exit, a split direction), drawn in the shared modal
//! [`Frame`](crate::ui::modal::Frame). Moved from jackin-preview so every
//! app can use it; it holds no app types (options are strings, the result
//! is an index).

use crate::core::event::{Key, Outcome};
use crate::core::focus::{Focus, FocusRing};
use crate::core::id::WidgetId;
use crate::theme::{ButtonKind, Strings, Tone};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::truncate;
use crate::widgets::button::{Button, row_layout_right};
use crate::widgets::choice::RadioGroup;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// A question with radio options and buttons (mount destination choice,
/// dirty-exit branches, split direction…).
pub struct ChoiceDialog {
    /// Which dismissals close it. Default `Dismiss::ESCAPE`.
    pub dismiss: crate::ui::popup::Dismiss,
    pub title: String,
    pub lines: Vec<(String, Tone)>,
    pub radio: RadioGroup,
    pub buttons: Vec<Button>,
    /// Button index Esc maps to.
    pub cancel_index: usize,
    /// Result: Some(option) when the primary fired, None when cancelled.
    pub result: Option<Option<usize>>,
    /// Which button fired (for owners with several non-cancel buttons).
    pub fired: Option<usize>,
    pub width: u16,
    pub stepper: Option<String>,
    pub area: Rect,
    pub option_tones: Vec<Tone>,
    /// The theme's strings as of the last render; a cancel label still
    /// reading them follows a change ([`Strings::follow`]).
    strings: Strings,
}

impl ChoiceDialog {
    pub fn new(id: WidgetId, title: &str, label: &str, options: &[&str], selected: usize) -> Self {
        Self {
            dismiss: crate::ui::popup::Dismiss::ESCAPE,
            title: title.to_owned(),
            lines: vec![],
            radio: RadioGroup::new(id.sub("radio"), label, options, selected),
            buttons: vec![
                Button::subtle(id.sub("cancel"), Strings::DEFAULT.cancel),
                Button::primary(id.sub("ok"), Strings::DEFAULT.next),
            ],
            cancel_index: 0,
            result: None,
            fired: None,
            width: 54,
            stepper: None,
            area: Rect::ZERO,
            option_tones: vec![],
            strings: Strings::DEFAULT,
        }
    }

    pub fn buttons(mut self, buttons: Vec<Button>, cancel_index: usize) -> Self {
        self.buttons = buttons;
        self.cancel_index = cancel_index;
        self
    }

    pub fn line(mut self, text: impl Into<String>, tone: Tone) -> Self {
        self.lines.push((text.into(), tone));
        self
    }

    pub fn stepper(mut self, s: &str) -> Self {
        self.stepper = Some(s.to_owned());
        self
    }

    pub fn width(mut self, w: u16) -> Self {
        self.width = w;
        self
    }

    pub fn option_tones(mut self, tones: Vec<Tone>) -> Self {
        self.option_tones = tones;
        self
    }

    pub fn initial_focus(&self) -> WidgetId {
        self.radio.id
    }

    fn fire(&mut self, i: usize) -> Outcome {
        self.fired = Some(i);
        self.result = Some(if i == self.cancel_index {
            None
        } else {
            Some(self.radio.selected)
        });
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.radio.id) {
            if key.is(KeyCode::Enter) {
                let primary = self
                    .buttons
                    .iter()
                    .position(|b| b.kind == ButtonKind::Primary || b.kind == ButtonKind::Danger)
                    .unwrap_or(self.buttons.len() - 1);
                return self.fire(primary);
            }
            let o = self.radio.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        for i in 0..self.buttons.len() {
            if cur == Some(self.buttons[i].id) {
                let (o, fired) = self.buttons[i].on_key(key);
                if fired {
                    return self.fire(i);
                }
                if o.consumed() {
                    return o;
                }
            }
        }
        match key.code {
            KeyCode::Esc => self.fire(self.cancel_index),
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                if let Some(i) = self.buttons.iter().position(|b| Some(b.id) == cur) {
                    let n = self.buttons.len();
                    let j = if matches!(key.code, KeyCode::Left | KeyCode::Char('h')) {
                        (i + n - 1) % n
                    } else {
                        (i + 1) % n
                    };
                    focus.focus(self.buttons[j].id);
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    pub fn on_click(&mut self, id: WidgetId, focus: &mut Focus) -> Outcome {
        for i in 0..self.radio.options.len() {
            if self.radio.option_id(i) == id {
                focus.focus(self.radio.id);
                return self.radio.on_click(i);
            }
        }
        for i in 0..self.buttons.len() {
            if self.buttons[i].id == id {
                focus.focus(id);
                if self.buttons[i].on_click() {
                    return self.fire(i);
                }
                return Outcome::Changed;
            }
        }
        Outcome::Consumed
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let s = ctx.theme.strings;
        let old = std::mem::replace(&mut self.strings, s);
        for (i, b) in self.buttons.iter_mut().enumerate() {
            if i == self.cancel_index {
                Strings::follow(&mut b.label, old.cancel, s.cancel);
            } else {
                Strings::follow(&mut b.label, old.next, s.next);
            }
        }
        let body = self.lines.len() as u16 + if self.lines.is_empty() { 0 } else { 1 };
        let stepper = if self.stepper.is_some() { 2 } else { 0 };
        let h = 2 + 2 + stepper + body + self.radio.height() + 2 + 1;
        let (area, inner) = crate::ui::modal::Frame::new(self.width, h)
            .title(&self.title)
            .meta(None)
            .render(screen, buf, ctx);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let mut y = inner.y;
        if let Some(s) = &self.stepper {
            buf.set_string(
                inner.x,
                y,
                truncate(s, inner.width as usize),
                t.muted().bg(bg),
            );
            y += 2;
        }
        for (l, tone) in &self.lines {
            buf.set_string(
                inner.x,
                y,
                truncate(l, inner.width as usize),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
            y += 1;
        }
        if !self.lines.is_empty() {
            y += 1;
        }
        self.radio.render(
            Rect::new(inner.x - 1, y, inner.width + 1, self.radio.height()),
            buf,
            ctx,
            bg,
        );
        // re-tone options (e.g. a destructive last row)
        for (i, tone) in self.option_tones.iter().enumerate() {
            if let Some(r) = self.radio.areas.get(i)
                && *tone != Tone::Normal
                && i != self.radio.cursor
            {
                buf.set_string(
                    r.x + 5,
                    r.y,
                    &self.radio.options[i],
                    Style::new().fg(t.tone(*tone)).bg(bg),
                );
            }
        }
        let ay = inner.bottom().saturating_sub(1);
        let widths: Vec<u16> = self.buttons.iter().map(|b| b.width()).collect();
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        for (b, r) in self.buttons.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
        for i in 0..self.radio.options.len() {
            if let Some(r) = self.radio.areas.get(i) {
                ctx.hits.register(self.radio.option_id(i), *r);
            }
        }
        for b in &self.buttons {
            ctx.hits.register(b.id, b.area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn screen_text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn a_library_choice_dialog_draws_in_the_modal_frame_and_answers_with_an_index() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut d = ChoiceDialog::new(
            WidgetId::of("split"),
            "Split",
            "Direction",
            &["Right", "Down"],
            0,
        );
        let mut focus = Focus::default();
        focus.focus(d.initial_focus());
        {
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
            d.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
            let shown = screen_text(&buf);
            assert!(shown.contains("Split") && shown.contains("Down"), "{shown}");
        }
        d.on_key(&key(KeyCode::Down), &mut focus, &ring);
        d.on_key(&key(KeyCode::Enter), &mut focus, &ring);
        assert_eq!(d.result, Some(Some(1)));
    }
}
