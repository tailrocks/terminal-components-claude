use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::style::Style;

use crate::core::focus::{Focus, FocusFallback, FocusRing};
use crate::core::id::WidgetId;
use crate::ui::ctx::{Backdrop, RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, Placement, place_with};
use crate::ui::text::{truncate, width};
use crate::ui::tooltip::Tip;
use crate::widgets::icon_button::IconButton;

pub const CLOSE: WidgetId = WidgetId::of("modal.close");

pub const SURFACE: WidgetId = WidgetId::of("modal.surface");

pub const BACKDROP: WidgetId = WidgetId::of("modal.backdrop");

pub const CLOSE_WIDTH: u16 = 3;

pub fn refuse_outside_click(
    pointer: &mut crate::core::pointer::PointerState,
    dismiss: Dismiss,
    now_ms: u64,
) -> bool {
    if dismiss.outside_click || dismiss.refuse_feedback != crate::ui::popup::RefuseFeedback::Flash {
        return false;
    }
    pointer.flash(SURFACE, now_ms);
    true
}

pub fn border_style(ctx: &RenderCtx, bg: ratatui::style::Color) -> Style {
    let t = ctx.theme;
    if ctx.interaction.flash == Some(SURFACE) {
        Style::new()
            .fg(t.warning)
            .bg(bg)
            .add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        t.border(true).bg(bg)
    }
}

pub fn close_button(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    bg: ratatui::style::Color,
) -> Rect {
    close_button_with(&IconButton::close(CLOSE), area, buf, ctx, bg)
}

pub fn close_button_with(
    button: &IconButton,
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    bg: ratatui::style::Color,
) -> Rect {
    let t = ctx.theme;
    let w = button.width(t);
    if area.width < 16 || area.height < 3 || area.width < w + 10 {
        return Rect::ZERO;
    }
    let host = t.border(true).bg(bg);
    let tip = Tip::new(t.strings.tip_close).shortcut("Esc");
    button
        .with_id(CLOSE)
        .render(area.right() - 2 - w, area.y, buf, ctx, host, Some(tip))
}

pub trait Modal {
    fn dismiss(&self) -> Dismiss;
}

impl<O: crate::ui::layer::Overlay> Modal for O {
    fn dismiss(&self) -> Dismiss {
        crate::ui::layer::Overlay::dismiss(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dismissal {
    None,

    Close,

    Keep,
}

struct Entry<M> {
    modal: M,
    saved_focus: Option<WidgetId>,
}

pub struct ModalStack<M> {
    entries: Vec<Entry<M>>,
}

impl<M> Default for ModalStack<M> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<M> ModalStack<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, modal: M, focus: &mut Focus, initial: Option<WidgetId>) {
        self.entries.push(Entry {
            modal,
            saved_focus: focus.current(),
        });
        focus.set(initial);
    }

    pub fn pop(&mut self, focus: &mut Focus) -> Option<M> {
        let e = self.entries.pop()?;
        focus.set(e.saved_focus);
        Some(e.modal)
    }

    pub fn clear(&mut self, focus: &mut Focus) {
        if let Some(e) = self.entries.first() {
            focus.set(e.saved_focus);
        }
        self.entries.clear();
    }

    pub fn top(&self) -> Option<&M> {
        self.entries.last().map(|e| &e.modal)
    }
    pub fn top_mut(&mut self) -> Option<&mut M> {
        self.entries.last_mut().map(|e| &mut e.modal)
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &M> {
        self.entries.iter().map(|e| &e.modal)
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut M> {
        self.entries.iter_mut().map(|e| &mut e.modal)
    }

    pub fn retain(&mut self, mut pred: impl FnMut(&M) -> bool) {
        self.entries.retain(|e| pred(&e.modal));
    }
}

impl<M: Modal> ModalStack<M> {
    pub fn outside_click(&self) -> Dismissal {
        match self.top() {
            None => Dismissal::None,
            Some(m) if m.dismiss().outside_click => Dismissal::Close,
            Some(_) => Dismissal::Keep,
        }
    }

    pub fn escape(&self) -> Dismissal {
        match self.top() {
            None => Dismissal::None,
            Some(m) if m.dismiss().escape => Dismissal::Close,
            Some(_) => Dismissal::Keep,
        }
    }
}

pub fn ensure_focus(focus: &mut Focus, ring: &FocusRing, fallback: FocusFallback) {
    focus.ensure_valid_with(ring, fallback);
}

#[deprecated(note = "use `popup::Placement` (`Placement::Center`) with `Frame::placement`")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePlacement {
    Upper,

    Middle,
}

#[allow(deprecated, clippy::derivable_impls)]
impl Default for FramePlacement {
    fn default() -> Self {
        Self::Upper
    }
}

#[allow(deprecated)]
impl From<FramePlacement> for Placement {
    fn from(_: FramePlacement) -> Self {
        Placement::Center
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<'a> {
    pub width: u16,
    pub height: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub title: Option<&'a str>,
    pub meta: Option<&'a str>,

    pub placement: Placement,

    pub anchor: Rect,

    pub fit: Place,

    pub margin: Margin,

    pub top: u16,

    pub bottom: u16,

    pub backdrop: bool,

    pub dim: Backdrop,

    pub dismiss: Option<Dismiss>,

    pub close_button: IconButton,
}

impl<'a> Frame<'a> {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            min_width: 20,
            min_height: 5,
            title: None,
            meta: None,
            placement: Placement::Center,
            anchor: Rect::ZERO,
            fit: Place::DEFAULT,
            margin: Margin::new(3, 1),
            top: 1,
            bottom: 1,
            backdrop: true,
            dim: Backdrop::DEFAULT,
            dismiss: None,
            close_button: IconButton::close(CLOSE),
        }
    }

    pub fn close_button(mut self, button: IconButton) -> Self {
        self.close_button = button;
        self
    }

    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = Some(dismiss);
        self
    }

    pub fn overlay<O: crate::ui::layer::Overlay + ?Sized>(mut self, overlay: &O) -> Self {
        self.dismiss = Some(overlay.dismiss());
        if let Some(b) = overlay.close_button() {
            self.close_button = b.clone();
        }
        self
    }
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn meta(mut self, meta: Option<&'a str>) -> Self {
        self.meta = meta;
        self
    }

    pub fn placement(mut self, placement: impl Into<Placement>) -> Self {
        self.placement = placement.into();
        self
    }

    pub fn anchor(mut self, anchor: Rect, placement: Placement) -> Self {
        self.anchor = anchor;
        self.placement = placement;
        self
    }
    pub fn fit(mut self, fit: Place) -> Self {
        self.fit = fit;
        self
    }
    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }
    pub fn min_size(mut self, width: u16, height: u16) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    pub fn chrome(mut self, top: u16, bottom: u16) -> Self {
        self.top = top;
        self.bottom = bottom;
        self
    }
    pub fn backdrop(mut self, on: bool) -> Self {
        self.backdrop = on;
        self
    }

    pub fn dim(mut self, dim: Backdrop) -> Self {
        self.dim = dim;
        self
    }

    pub fn area(&self, screen: Rect) -> Rect {
        let bounds = crate::widgets::dialog::body_bounds(screen, self.top, self.bottom);
        let m = crate::widgets::dialog::OUTER_MARGIN;
        let w = self
            .width
            .min(bounds.width.saturating_sub(2 * m.horizontal))
            .max(self.min_width)
            .min(bounds.width);
        let h = self
            .height
            .min(bounds.height.saturating_sub(2 * m.vertical))
            .max(self.min_height)
            .min(bounds.height);
        place_with(bounds, self.anchor, w, h, self.placement, self.fit)
    }

    pub fn render(&self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) -> (Rect, Rect) {
        let t = ctx.theme;
        if self.backdrop {
            ctx.modal_backdrop_with(buf, screen, self.dim);

            if self.dismiss.is_some() && !ctx.inert {
                ctx.hits.register_part_as(
                    BACKDROP,
                    BACKDROP,
                    crate::core::id::PartRef::of(crate::core::id::Part::BACKDROP),
                    screen,
                );
            }
        }
        let area = self.area(screen);
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_set(t.glyphs.border)
            .border_style(border_style(ctx, bg));
        ratatui::widgets::Widget::render(block, area, buf);

        if !ctx.inert {
            ctx.hits.register_surface(SURFACE, area);
        }
        let close = match self.dismiss {
            Some(d) if d.shows_close() => close_button_with(&self.close_button, area, buf, ctx, bg),
            _ => Rect::ZERO,
        };

        let reserve = if close.is_empty() { 0 } else { close.width + 1 };
        if let Some(title) = self.title
            && area.width > 6 + reserve
        {
            let tt = format!(
                " {} ",
                truncate(title, area.width.saturating_sub(6 + reserve) as usize)
            );
            buf.set_string(area.x + 2, area.y, &tt, t.title().bg(bg));
            if let Some(m) = self.meta {
                let mt = format!(" {m} ");
                let mw = width(&mt) as u16;
                if area.width > width(&tt) as u16 + mw + 4 + reserve {
                    buf.set_string(
                        area.right() - 2 - reserve - mw,
                        area.y,
                        &mt,
                        t.faint().bg(bg),
                    );
                }
            }
        }
        (area, area.inner(self.margin))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct M(Dismiss);
    impl Modal for M {
        fn dismiss(&self) -> Dismiss {
            self.0
        }
    }

    #[test]
    fn nested_modals_unwind_focus_to_where_the_user_was() {
        let mut focus = Focus::default();
        let page = WidgetId::of("page.row");
        focus.focus(page);
        let mut s = ModalStack::new();
        s.push(M(Dismiss::LIGHT), &mut focus, Some(WidgetId::of("picker")));
        s.push(M(Dismiss::ESCAPE), &mut focus, Some(WidgetId::of("dialog")));
        assert_eq!(focus.current(), Some(WidgetId::of("dialog")));
        s.pop(&mut focus);
        assert_eq!(focus.current(), Some(WidgetId::of("picker")));
        s.pop(&mut focus);
        assert_eq!(focus.current(), Some(page));
        assert!(s.pop(&mut focus).is_none());
    }

    #[test]
    fn dismissal_follows_the_top_modal_policy() {
        let mut focus = Focus::default();
        let mut s = ModalStack::new();
        assert_eq!(s.outside_click(), Dismissal::None);
        s.push(M(Dismiss::LIGHT), &mut focus, None);
        assert_eq!(s.outside_click(), Dismissal::Close);
        s.push(M(Dismiss::ESCAPE), &mut focus, None);
        assert_eq!(s.outside_click(), Dismissal::Keep);
        assert_eq!(s.escape(), Dismissal::Close);
        s.push(M(Dismiss::NEVER), &mut focus, None);
        assert_eq!(s.escape(), Dismissal::Keep);
    }

    #[test]
    fn frame_clamps_into_the_body_and_centers() {
        let screen = Rect::new(0, 0, 100, 30);
        let r = Frame::new(60, 10).area(screen);
        assert_eq!((r.x, r.width, r.height), (20, 60, 10));
        let r = Frame::new(200, 80).area(screen);
        assert!(r.width <= 94 && r.height <= 26 && r.y >= 1);
        let mid = Frame::new(60, 10).area(screen);
        assert_eq!(mid.y, 1 + (28 - 10) / 2);
    }

    #[test]
    #[allow(deprecated)]
    fn the_old_frame_placements_still_resolve_to_the_centred_frame() {
        let screen = Rect::new(0, 0, 101, 31);
        for (w, h) in [(60, 10), (61, 11), (20, 5), (200, 80)] {
            let centred = Frame::new(w, h).area(screen);
            for old in [FramePlacement::Upper, FramePlacement::Middle] {
                assert_eq!(Frame::new(w, h).placement(old).area(screen), centred);
            }
        }
    }

    #[test]
    fn a_frame_can_open_under_an_anchor() {
        let screen = Rect::new(0, 0, 100, 30);
        let field = Rect::new(10, 4, 20, 1);
        let r = Frame::new(40, 8)
            .anchor(field, Placement::Below)
            .area(screen);
        assert_eq!((r.x, r.y), (10, 5));
    }

    fn framed(
        t: &crate::theme::Theme,
        title: &str,
        hover: bool,
    ) -> (Buffer, crate::core::hit::HitRegistry, Rect) {
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let inter = crate::ui::ctx::Interaction {
            hover: hover.then_some(CLOSE),
            ..Default::default()
        };
        let screen = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(screen);
        let area = {
            let mut ctx = RenderCtx::new(t, inter, &mut hits, &mut ring);
            Frame::new(30, 8)
                .title(title)
                .dismiss(Dismiss::LIGHT)
                .backdrop(false)
                .render(screen, &mut buf, &mut ctx)
                .0
        };
        (buf, hits, area)
    }

    #[test]
    fn the_frame_close_plate_keeps_the_corner_and_lights_whole() {
        for level in crate::theme::ColorLevel::ALL {
            let t = crate::theme::Theme::for_level(level);
            let (rest, hits, area) = framed(&t, "Rename task", false);
            let plate = hits.area_of(CLOSE).expect("the frame draws ×");
            assert_eq!(plate, Rect::new(area.right() - 5, area.y, CLOSE_WIDTH, 1));
            let row = |b: &Buffer| -> String {
                (area.right() - 6..area.right())
                    .map(|x| b[(x, area.y)].symbol())
                    .collect()
            };
            let corner =
                format!(" {} ─╮", t.glyphs.close).replace('─', t.glyphs.border.horizontal_top);
            let corner = corner.replace('╮', t.glyphs.border.top_right);
            assert!(row(&rest).ends_with(&corner), "{:?} {level:?}", row(&rest));
            let (lit, _, _) = framed(&t, "Rename task", true);
            assert_eq!(row(&lit), row(&rest), "hover keeps every glyph");
            let look = |b: &Buffer, x: u16| {
                let c = &b[(x, area.y)];
                (
                    (level != crate::theme::ColorLevel::Mono).then_some(c.bg),
                    c.modifier,
                )
            };
            for x in plate.left()..plate.right() {
                assert_ne!(look(&lit, x), look(&rest, x), "{x} {level:?}");
                assert_eq!(look(&lit, x), look(&lit, plate.x), "{x} {level:?}");
            }
            for x in [plate.x - 1, plate.right(), plate.right() + 1] {
                assert_eq!(look(&lit, x), look(&rest, x), "{x} {level:?}");
            }
        }

        let t = crate::theme::Theme::junie();
        let (buf, hits, area) = framed(&t, &"Long title ".repeat(6), false);
        let plate = hits.area_of(CLOSE).unwrap();
        assert_eq!(buf[(plate.x + 1, area.y)].symbol(), "×");
        assert_eq!(buf[(plate.x - 1, area.y)].symbol(), "─");
    }
}
