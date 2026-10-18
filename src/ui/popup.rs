use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dismiss {
    pub escape: bool,

    pub outside_click: bool,

    pub pass_through: bool,

    pub close_button: bool,

    pub refuse_feedback: RefuseFeedback,

    pub focus_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefuseFeedback {
    #[default]
    Flash,

    None,
}

impl Dismiss {
    pub const LIGHT: Self = Self {
        escape: true,
        outside_click: true,
        pass_through: false,
        close_button: true,
        refuse_feedback: RefuseFeedback::Flash,
        focus_out: true,
    };

    pub const ESCAPE: Self = Self {
        escape: true,
        outside_click: false,
        pass_through: false,
        close_button: true,
        refuse_feedback: RefuseFeedback::Flash,
        focus_out: false,
    };

    pub const NEVER: Self = Self {
        escape: false,
        outside_click: false,
        pass_through: false,
        close_button: false,
        refuse_feedback: RefuseFeedback::Flash,
        focus_out: false,
    };
}

impl Dismiss {
    pub const fn pass_through(mut self, on: bool) -> Self {
        self.pass_through = on;
        self
    }

    pub const fn close_button(mut self, on: bool) -> Self {
        self.close_button = on;
        self
    }

    pub const fn refuse_feedback(mut self, feedback: RefuseFeedback) -> Self {
        self.refuse_feedback = feedback;
        self
    }

    pub const fn focus_out(mut self, on: bool) -> Self {
        self.focus_out = on;
        self
    }

    pub const fn allows(self, reason: crate::core::intent::DismissReason) -> bool {
        use crate::core::intent::DismissReason as R;
        match reason {
            R::Esc | R::CloseButton => self.escape,
            R::OutsideClick => self.outside_click,
            R::FocusOut => self.focus_out,
            R::Programmatic => true,
        }
    }

    pub const fn shows_close(self) -> bool {
        self.close_button && self.escape
    }
}

impl Default for Dismiss {
    fn default() -> Self {
        Self::LIGHT
    }
}

pub const fn backdrop_id(owner: WidgetId) -> WidgetId {
    owner.sub("popup.backdrop")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Below,

    Above,

    Right,

    Left,

    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Shrink,

    Slide,
}

pub fn backdrop(ctx: &mut RenderCtx, owner: WidgetId, dismiss: Dismiss, screen: Rect) {
    if !ctx.inert {
        ctx.hits.register_backdrop_of(
            backdrop_id(owner),
            owner,
            screen,
            dismiss.outside_click && dismiss.pass_through,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Center,

    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    pub margin: u16,

    pub v_align: VAlign,

    pub text_inset: u16,

    pub vertical_margin: Option<u16>,

    pub overflow: Overflow,
}

impl Place {
    pub const DEFAULT: Self = Self {
        margin: 1,
        v_align: VAlign::Center,
        text_inset: 0,
        vertical_margin: None,
        overflow: Overflow::Shrink,
    };

    pub const FLUSH: Self = Self {
        margin: 0,
        v_align: VAlign::Center,
        text_inset: 0,
        vertical_margin: None,
        overflow: Overflow::Shrink,
    };

    pub const MENU: Self = Self {
        margin: 1,
        v_align: VAlign::Center,
        text_inset: 0,
        vertical_margin: Some(0),
        overflow: Overflow::Slide,
    };

    pub const fn vertical_margin(mut self, margin: Option<u16>) -> Self {
        self.vertical_margin = margin;
        self
    }

    pub const fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub const fn margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    pub const fn v_align(mut self, v_align: VAlign) -> Self {
        self.v_align = v_align;
        self
    }

    pub const fn text_inset(mut self, inset: u16) -> Self {
        self.text_inset = inset;
        self
    }
}

impl Default for Place {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn place(screen: Rect, anchor: Rect, width: u16, height: u16, placement: Placement) -> Rect {
    place_with(screen, anchor, width, height, placement, Place::DEFAULT)
}

pub fn place_with(
    screen: Rect,
    anchor: Rect,
    width: u16,
    height: u16,
    placement: Placement,
    fit: Place,
) -> Rect {
    let width = width.min(screen.width).max(1);
    let height = height.min(screen.height).max(1);

    let mx = fit.margin.min(screen.width.saturating_sub(width) / 2);
    let my = fit
        .vertical_margin
        .unwrap_or(fit.margin)
        .min(screen.height.saturating_sub(height) / 2);
    let bounds = Rect::new(
        screen.x + mx,
        screen.y + my,
        screen.width - 2 * mx,
        screen.height - 2 * my,
    );
    match placement {
        Placement::Center => {
            let x = bounds.x + bounds.width.saturating_sub(width) / 2;
            let y = match fit.v_align {
                VAlign::Center => bounds.y + bounds.height.saturating_sub(height) / 2,
                VAlign::Third => bounds.y + (bounds.height.saturating_sub(height) / 3).max(1),
            };
            Rect::new(
                x,
                y.min(bounds.bottom().saturating_sub(height)),
                width,
                height,
            )
        }
        Placement::Below | Placement::Above => {
            let (y, height) = side(
                (bounds.y, bounds.bottom()),
                (anchor.y, anchor.bottom()),
                height,
                placement == Placement::Below,
                fit.overflow,
            );
            let x = anchor
                .x
                .saturating_sub(fit.text_inset)
                .min(bounds.right().saturating_sub(width))
                .max(bounds.x);
            Rect::new(x, y, width, height)
        }
        Placement::Right | Placement::Left => {
            let (x, width) = side(
                (bounds.x, bounds.right()),
                (anchor.x, anchor.right()),
                width,
                placement == Placement::Right,
                fit.overflow,
            );
            let y = anchor
                .y
                .min(bounds.bottom().saturating_sub(height))
                .max(bounds.y);
            Rect::new(x, y, width, height)
        }
    }
}

fn side(
    bounds: (u16, u16),
    anchor: (u16, u16),
    len: u16,
    after: bool,
    overflow: Overflow,
) -> (u16, u16) {
    let (lo, hi) = bounds;
    let past = anchor.1.clamp(lo, hi);
    let room_after = hi.saturating_sub(past);
    let room_before = anchor.0.saturating_sub(lo).min(hi - lo);
    let at_after = |n: u16| (past, n);
    let at_before = |n: u16| (anchor.0 - n, n);
    let (first_room, second_room) = if after {
        (room_after, room_before)
    } else {
        (room_before, room_after)
    };
    let pick = |is_after: bool, n: u16| if is_after { at_after(n) } else { at_before(n) };
    if first_room >= len {
        return pick(after, len);
    }
    if second_room >= len {
        return pick(!after, len);
    }
    match overflow {
        Overflow::Slide => {
            if after {
                (hi.saturating_sub(len).max(lo), len)
            } else {
                (lo, len)
            }
        }
        Overflow::Shrink => {
            let after_wins = if after {
                room_after >= room_before
            } else {
                room_after > room_before
            };
            if after_wins && room_after > 0 {
                at_after(room_after)
            } else if room_before > 0 {
                at_before(room_before)
            } else if room_after > 0 {
                at_after(room_after)
            } else {
                (hi.saturating_sub(len), len)
            }
        }
    }
}

pub fn surface(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, t: &Theme) -> Rect {
    surface_for(area, buf, ctx, t, None)
}

pub fn surface_for(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    t: &Theme,
    owner: Option<(WidgetId, Dismiss)>,
) -> Rect {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return area;
    }
    fill(buf, area, Style::new().bg(t.surface_elevated));
    let block = ratatui::widgets::Block::new()
        .borders(ratatui::widgets::Borders::ALL)
        .border_set(t.glyphs.border)
        .border_style(t.border(true).bg(t.surface_elevated));
    ratatui::widgets::Widget::render(block, area, buf);

    ctx.hits.push_barrier();
    if let Some((owner, dismiss)) = owner {
        backdrop(ctx, owner, dismiss, *buf.area());
    }
    ctx.hits.register(WidgetId::of("popup.surface"), area);
    area.inner(ratatui::layout::Margin::new(1, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_below_then_flips_then_clamps() {
        let screen = Rect::new(0, 0, 100, 30);
        let anchor = Rect::new(10, 5, 5, 1);
        assert_eq!(
            place(screen, anchor, 40, 8, Placement::Below),
            Rect::new(10, 6, 40, 8)
        );
        let low = Rect::new(10, 26, 5, 1);
        assert_eq!(
            place(screen, low, 40, 8, Placement::Below),
            Rect::new(10, 18, 40, 8)
        );

        let right = Rect::new(90, 5, 5, 1);
        assert_eq!(place(screen, right, 40, 8, Placement::Below).right(), 99);
        assert_eq!(
            place_with(screen, right, 40, 8, Placement::Below, Place::FLUSH).right(),
            100
        );
        let tall = place(screen, anchor, 40, 60, Placement::Below);
        assert!(tall.height <= 30);
    }

    #[test]
    fn a_popup_that_fits_neither_side_shrinks_on_the_roomier_side_and_never_covers_the_anchor() {
        let screen = Rect::new(0, 0, 80, 20);

        let anchor = Rect::new(4, 7, 20, 1);
        let r = place(screen, anchor, 30, 16, Placement::Below);
        assert_eq!(r, Rect::new(4, 8, 30, 11));

        let anchor = Rect::new(4, 15, 20, 1);
        let r = place(screen, anchor, 30, 18, Placement::Below);
        assert_eq!(r, Rect::new(4, 1, 30, 14));
        assert!(r.bottom() <= anchor.y);
    }

    #[test]
    fn centers_like_a_dialog_or_in_the_upper_third_on_request() {
        let screen = Rect::new(0, 0, 120, 40);
        let r = place(screen, Rect::ZERO, 60, 12, Placement::Center);
        assert_eq!(r.x, 30);
        assert_eq!(r.y, 14);
        let third = Place::DEFAULT.v_align(VAlign::Third);
        let r = place_with(screen, Rect::ZERO, 60, 12, Placement::Center, third);
        assert_eq!(r.y, 9);
    }

    #[test]
    fn a_popup_keeps_off_the_screen_edges_unless_it_would_not_fit() {
        let screen = Rect::new(0, 0, 40, 10);

        let r = place(screen, Rect::new(0, 0, 1, 1), 20, 4, Placement::Below);
        assert_eq!((r.x, r.y), (1, 1));

        let r = place(screen, Rect::new(0, 0, 1, 1), 40, 4, Placement::Below);
        assert_eq!((r.x, r.width), (0, 40));
    }

    #[test]
    fn an_anchored_popup_lines_its_text_up_with_the_anchor() {
        let screen = Rect::new(0, 0, 80, 20);
        let anchor = Rect::new(20, 5, 1, 1);
        let fit = Place::DEFAULT.text_inset(4);
        let r = place_with(screen, anchor, 30, 6, Placement::Below, fit);
        assert_eq!(r.x + 4, anchor.x);
    }

    #[test]
    fn above_prefers_the_top_and_flips_below() {
        let screen = Rect::new(0, 0, 80, 20);
        let anchor = Rect::new(10, 10, 5, 1);
        assert_eq!(
            place(screen, anchor, 20, 6, Placement::Above),
            Rect::new(10, 4, 20, 6)
        );
        let high = Rect::new(10, 2, 5, 1);
        assert_eq!(
            place(screen, high, 20, 6, Placement::Above),
            Rect::new(10, 3, 20, 6)
        );
    }

    #[test]
    fn right_opens_beside_the_anchor_and_flips_left_at_the_edge() {
        let screen = Rect::new(0, 0, 80, 20);
        let anchor = Rect::new(10, 4, 12, 1);
        assert_eq!(
            place(screen, anchor, 20, 6, Placement::Right),
            Rect::new(22, 4, 20, 6)
        );

        let edge = Rect::new(60, 4, 12, 1);
        let r = place(screen, edge, 20, 6, Placement::Right);
        assert_eq!(r, Rect::new(40, 4, 20, 6));
        assert_eq!(
            place(screen, anchor, 20, 6, Placement::Left),
            Rect::new(22, 4, 20, 6),
            "no room on the left: flips right"
        );
        let narrow = Rect::new(0, 0, 40, 20);
        assert_eq!(
            place(narrow, Rect::new(10, 4, 20, 1), 20, 6, Placement::Left),
            Rect::new(1, 4, 9, 6),
            "too little room either side: the preferred side on a tie, narrower"
        );

        let low = Rect::new(10, 17, 12, 1);
        assert_eq!(place(screen, low, 20, 6, Placement::Right).bottom(), 19);
    }

    #[test]
    fn slide_keeps_the_full_size_when_neither_side_has_room() {
        let screen = Rect::new(0, 0, 40, 12);
        let anchor = Rect::new(2, 5, 4, 1);
        let shrunk = place(screen, anchor, 20, 8, Placement::Below);
        assert!(shrunk.height < 8);
        let menu = place_with(screen, anchor, 20, 8, Placement::Below, Place::MENU);
        assert_eq!(menu, Rect::new(2, 4, 20, 8));
        let up = place_with(screen, anchor, 20, 8, Placement::Above, Place::MENU);
        assert_eq!(up, Rect::new(2, 0, 20, 8));
    }
}
