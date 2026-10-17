//! A row of action buttons with an optional status message beside them.
//!
//! ```text
//!  [ Connect ]  [ Edit… ]  [ Duplicate ]  ✓ Saved 2 s ago     (start: message after the buttons)
//!  Two steps excluded              [ Cancel ]  [ Run plan ]   (end: message before them)
//! ```
//!
//! The one place a card, page or footer lays out its buttons: packed to
//! the start (forms, cards) or the end (dialog actions, the macOS
//! convention), a fixed gap between them, never wrapping or folding (a
//! [`ButtonRow`] does that for dialogs that must). ←/→ between the
//! buttons is [`ActionRow::step`]: the one button-row navigation (K32),
//! skipping disabled buttons and stopping at the ends unless
//! [`ActionRow::wrap`].

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{truncate, width};
use crate::widgets::button::{Button, ButtonRow, RowAlign, row_layout, row_layout_right};

/// A button row with an optional message; see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRow {
    /// Where the buttons sit. Default [`RowAlign::Start`].
    pub align: RowAlign,
    /// Cells between buttons, and between the buttons and the message.
    /// Default 2.
    pub gap: u16,
    /// A status message beside the buttons, in its tone, cut to the room
    /// left (after the buttons at the start, before them at the end).
    pub message: Option<(String, Tone)>,
    /// ←/→ past the last button lands on the first. Default off (K32).
    pub wrap: bool,
}

impl Default for ActionRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionRow {
    pub fn new() -> Self {
        Self {
            align: RowAlign::Start,
            gap: 2,
            message: None,
            wrap: false,
        }
    }

    /// See [`Self::align`].
    pub fn align(mut self, align: RowAlign) -> Self {
        self.align = align;
        self
    }

    /// See [`Self::gap`].
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// See [`Self::message`].
    pub fn message(mut self, text: &str, tone: Tone) -> Self {
        self.message = Some((text.to_owned(), tone));
        self
    }

    /// See [`Self::wrap`].
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// Where each button goes in `area` (one row), without drawing.
    pub fn layout(&self, area: Rect, buttons: &[&Button]) -> Vec<Rect> {
        let widths: Vec<u16> = buttons.iter().map(|b| b.width()).collect();
        let row = Rect::new(area.x, area.y, area.width, area.height.min(1));
        match self.align {
            RowAlign::End => row_layout_right(row, &widths, self.gap),
            _ => row_layout(row, &widths, self.gap),
        }
    }

    /// Draw the buttons and the message on the first row of `area`, on
    /// the plane `bg`; returns each button's area.
    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        buttons: &mut [&mut Button],
    ) -> Vec<Rect> {
        let rects = {
            let view: Vec<&Button> = buttons.iter().map(|b| &**b).collect();
            self.layout(area, &view)
        };
        for (b, r) in buttons.iter_mut().zip(&rects) {
            b.render(*r, buf, ctx, bg);
        }
        if let Some((text, tone)) = &self.message
            && !area.is_empty()
        {
            let t = ctx.theme;
            let ink = Style::new().fg(t.tone_on(*tone, bg)).bg(bg);
            match self.align {
                RowAlign::End => {
                    let left = rects.first().map_or(area.right(), |r| r.x);
                    let room = left.saturating_sub(area.x + self.gap);
                    if room > 0 {
                        buf.set_string(area.x, area.y, truncate(text, room as usize), ink);
                    }
                }
                _ => {
                    let x = rects
                        .iter()
                        .filter(|r| r.width > 0)
                        .map(|r| r.right() + self.gap)
                        .max()
                        .unwrap_or(area.x);
                    let room = area.right().saturating_sub(x);
                    if room > 0 && x < area.right() {
                        let s = truncate(text, room as usize);
                        if width(&s) > 0 {
                            buf.set_string(x, area.y, s, ink);
                        }
                    }
                }
            }
        }
        rects
    }

    /// The button ←/→ moves to from `cur` (the next or previous enabled
    /// one), per [`Self::wrap`]; `None` at an end or when `cur` is not in
    /// the row.
    pub fn step(
        &self,
        buttons: &[&Button],
        cur: Option<WidgetId>,
        forward: bool,
    ) -> Option<WidgetId> {
        ButtonRow::new()
            .wrap(self.wrap)
            .step(buttons.iter().copied(), cur, forward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    fn draw(r: &ActionRow, w: u16, buttons: &mut [&mut Button]) -> (Buffer, Vec<Rect>) {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, w, 1);
        let mut buf = Buffer::empty(area);
        let rects = r.render(area, &mut buf, &mut ctx, t.canvas, buttons);
        (buf, rects)
    }

    #[test]
    fn start_packs_the_buttons_and_puts_the_message_after_them() {
        let mut a = Button::primary(WidgetId::of("a"), "Save");
        let mut b = Button::secondary(WidgetId::of("b"), "Cancel");
        let r = ActionRow::new().message("Saved 2 s ago", Tone::Success);
        let (buf, rects) = draw(&r, 50, &mut [&mut a, &mut b]);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[1].x, rects[0].right() + 2);
        let text = row(&buf, 0);
        let at = rects[1].right() as usize + 2;
        assert_eq!(
            &text.chars().skip(at).collect::<String>()[..13],
            "Saved 2 s ago"
        );
        let t = Theme::junie();
        assert_eq!(buf[(at as u16, 0)].fg, t.tone_on(Tone::Success, t.canvas));
        // no room: the message is cut first, the buttons stay
        let (buf, rects2) = draw(&r, rects[1].right() + 6, &mut [&mut a, &mut b]);
        assert_eq!(rects2, rects);
        assert!(row(&buf, 0).trim_end().ends_with('…'), "{}", row(&buf, 0));
    }

    #[test]
    fn end_packs_the_buttons_right_and_the_message_before_them() {
        let mut a = Button::secondary(WidgetId::of("a"), "Cancel");
        let mut b = Button::primary(WidgetId::of("b"), "Run");
        let r = ActionRow::new()
            .align(RowAlign::End)
            .message("2 steps excluded", Tone::Muted);
        let (buf, rects) = draw(&r, 50, &mut [&mut a, &mut b]);
        assert_eq!(rects[1].right(), 50);
        assert!(row(&buf, 0).starts_with("2 steps excluded"));
    }

    #[test]
    fn arrows_skip_disabled_buttons_and_stop_at_the_ends() {
        let a = Button::secondary(WidgetId::of("a"), "A");
        let b = Button::secondary(WidgetId::of("b"), "B").disabled(true);
        let c = Button::secondary(WidgetId::of("c"), "C");
        let r = ActionRow::new();
        let row = [&a, &b, &c];
        assert_eq!(r.step(&row, Some(a.id), true), Some(c.id));
        assert_eq!(r.step(&row, Some(c.id), true), None);
        assert_eq!(
            r.clone().wrap(true).step(&row, Some(c.id), true),
            Some(a.id)
        );
    }
}
