use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::core::id::WidgetId;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{truncate, width};
use crate::widgets::button::{Button, ButtonRow, RowAlign, row_layout, row_layout_right};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRow {
    pub align: RowAlign,

    pub gap: u16,

    pub message: Option<(String, Tone)>,

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

    pub fn align(mut self, align: RowAlign) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn message(mut self, text: &str, tone: Tone) -> Self {
        self.message = Some((text.to_owned(), tone));
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn width(&self, buttons: &[&Button]) -> u16 {
        let n = buttons.len() as u16;
        let row = buttons.iter().map(|b| b.width()).sum::<u16>() + self.gap * n.saturating_sub(1);
        match &self.message {
            Some((text, _)) => row + self.gap + width(text) as u16,
            None => row,
        }
    }

    pub fn layout(&self, area: Rect, buttons: &[&Button]) -> Vec<Rect> {
        let widths: Vec<u16> = buttons.iter().map(|b| b.width()).collect();
        let row = Rect::new(area.x, area.y, area.width, area.height.min(1));
        match self.align {
            RowAlign::End => row_layout_right(row, &widths, self.gap),
            _ => row_layout(row, &widths, self.gap),
        }
    }

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
    fn width_shows_the_buttons_and_the_whole_message() {
        for align in [RowAlign::Start, RowAlign::End] {
            let mut a = Button::primary(WidgetId::of("a"), "Save");
            let mut b = Button::secondary(WidgetId::of("b"), "Cancel");
            let r = ActionRow::new()
                .align(align)
                .message("Saved 2 s ago", Tone::Success);
            let w = r.width(&[&a, &b]);
            let (buf, _) = draw(&r, w, &mut [&mut a, &mut b]);
            let text = row(&buf, 0);
            for part in ["Save", "Cancel", "Saved 2 s ago"] {
                assert!(text.contains(part), "{align:?}: {text:?} lacks {part}");
            }
            let (buf, _) = draw(&r, w - 1, &mut [&mut a, &mut b]);
            assert!(!row(&buf, 0).contains("Saved 2 s ago"), "{align:?}");
        }
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
