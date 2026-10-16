//! Modal layers: a stack that owns focus save/restore and dismissal
//! routing, and the shared centered frame modals are drawn in.
//!
//! Applications used to keep their own `Vec` of modals, their own "which
//! kinds close on an outside click" table and their own copy of the
//! backdrop + rounded frame code. Dismissal is a policy of the modal
//! ([`Modal::dismiss`]), never a check in app code, and the frame is
//! [`Frame`].

use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::style::Style;

use crate::core::focus::{Focus, FocusFallback, FocusRing};
use crate::core::id::WidgetId;
use crate::ui::ctx::{Backdrop, RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, Placement, place_with};
use crate::ui::text::{truncate, width};

/// A layer that sits above the page and decides how it may be dismissed.
pub trait Modal {
    /// Which dismissals (Esc, a click outside) close it without choosing.
    fn dismiss(&self) -> Dismiss;
}

impl Modal for crate::widgets::dialog::Dialog {
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
}

impl Modal for crate::widgets::picker::Picker {
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
}

impl Modal for crate::widgets::menu::ContextMenu {
    fn dismiss(&self) -> Dismiss {
        self.dismiss
    }
}

/// What an outside click (or Esc) on the top modal resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dismissal {
    /// No modal is open.
    None,
    /// The policy allows it: close the modal as cancelled.
    Close,
    /// The policy refuses it: the modal stays, the event is swallowed.
    Keep,
}

struct Entry<M> {
    modal: M,
    saved_focus: Option<WidgetId>,
}

/// A stack of modal layers. Opening one remembers the focus it covers;
/// closing it gives that focus back, so nested modals unwind to exactly
/// where the user was (a picker over a dialog over a page).
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

    /// Open `modal` on top, moving focus to `initial` and remembering the
    /// focus it covers.
    pub fn push(&mut self, modal: M, focus: &mut Focus, initial: Option<WidgetId>) {
        self.entries.push(Entry {
            modal,
            saved_focus: focus.current(),
        });
        focus.set(initial);
    }

    /// Close the top modal, restoring the focus it covered.
    pub fn pop(&mut self, focus: &mut Focus) -> Option<M> {
        let e = self.entries.pop()?;
        focus.set(e.saved_focus);
        Some(e.modal)
    }

    /// Close every modal, restoring the focus the bottom one covered.
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
    /// Bottom to top.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &M> {
        self.entries.iter().map(|e| &e.modal)
    }
    /// Bottom to top.
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut M> {
        self.entries.iter_mut().map(|e| &mut e.modal)
    }
    /// Remove every modal matching `pred` without touching focus (used when
    /// the content a modal belongs to goes away).
    pub fn retain(&mut self, mut pred: impl FnMut(&M) -> bool) {
        self.entries.retain(|e| pred(&e.modal));
    }
}

impl<M: Modal> ModalStack<M> {
    /// How a click outside the top modal resolves under its policy.
    pub fn outside_click(&self) -> Dismissal {
        match self.top() {
            None => Dismissal::None,
            Some(m) if m.dismiss().outside_click => Dismissal::Close,
            Some(_) => Dismissal::Keep,
        }
    }

    /// How Esc on the top modal resolves under its policy.
    pub fn escape(&self) -> Dismissal {
        match self.top() {
            None => Dismissal::None,
            Some(m) if m.dismiss().escape => Dismissal::Close,
            Some(_) => Dismissal::Keep,
        }
    }
}

/// After a frame rebuilt the focus ring: keep focus valid, falling back per
/// `fallback` when the focused control disappeared (the row a dialog just
/// deleted).
pub fn ensure_focus(focus: &mut Focus, ring: &FocusRing, fallback: FocusFallback) {
    focus.ensure_valid_with(ring, fallback);
}

/// Where a [`Frame`] sat inside its bounds, before frames took the shared
/// [`Placement`]. Both variants always resolved to the same centred
/// rectangle; they convert to [`Placement::Center`].
#[deprecated(note = "use `popup::Placement` (`Placement::Center`) with `Frame::placement`")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePlacement {
    /// Centered (despite the name, never the upper third).
    Upper,
    /// Centered on both axes.
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

/// The centered, rounded, elevated frame every modal is drawn in: dims the
/// page, starts the modal layer, clamps to the body (clear of `top` header
/// rows and `bottom` footer rows, with [`crate::widgets::dialog::OUTER_MARGIN`]
/// around it) and draws the title and meta on the top border.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<'a> {
    pub width: u16,
    pub height: u16,
    pub min_width: u16,
    pub min_height: u16,
    pub title: Option<&'a str>,
    pub meta: Option<&'a str>,
    /// Where the frame goes in the body. Default [`Placement::Center`];
    /// an anchored placement opens it next to [`Self::anchor`].
    pub placement: Placement,
    /// The rectangle an anchored [`Self::placement`] opens next to.
    /// Default `Rect::ZERO`.
    pub anchor: Rect,
    /// How the frame is fitted into the body ([`place_with`]). Default
    /// [`Place::DEFAULT`].
    pub fit: Place,
    /// Inner margin between the border and the content.
    pub margin: Margin,
    /// Header rows kept clear above the frame.
    pub top: u16,
    /// Footer rows kept clear below the frame.
    pub bottom: u16,
    /// Dim the page and start the modal layer (off for a frame drawn on an
    /// already-dimmed layer).
    pub backdrop: bool,
    /// Which screen rows the backdrop leaves live. Default
    /// [`Backdrop::DEFAULT`] (the footer row).
    pub dim: Backdrop,
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
        }
    }
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn meta(mut self, meta: Option<&'a str>) -> Self {
        self.meta = meta;
        self
    }
    /// See [`Self::placement`]. Takes a [`Placement`] (or, for old
    /// callers, a deprecated `FramePlacement`).
    pub fn placement(mut self, placement: impl Into<Placement>) -> Self {
        self.placement = placement.into();
        self
    }
    /// Open next to `anchor` per `placement`.
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
    /// Rows kept clear for the header (`top`) and footer (`bottom`).
    pub fn chrome(mut self, top: u16, bottom: u16) -> Self {
        self.top = top;
        self.bottom = bottom;
        self
    }
    pub fn backdrop(mut self, on: bool) -> Self {
        self.backdrop = on;
        self
    }
    /// See [`Self::dim`].
    pub fn dim(mut self, dim: Backdrop) -> Self {
        self.dim = dim;
        self
    }

    /// The frame's outer rectangle on `screen`.
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

    /// Draw the frame. Returns `(outer, inner)`.
    pub fn render(&self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) -> (Rect, Rect) {
        let t = ctx.theme;
        if self.backdrop {
            ctx.modal_backdrop_with(buf, screen, self.dim);
        }
        let area = self.area(screen);
        let bg = t.surface_elevated;
        fill(buf, area, Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        if let Some(title) = self.title
            && area.width > 6
        {
            let tt = format!(
                " {} ",
                truncate(title, area.width.saturating_sub(6) as usize)
            );
            buf.set_string(area.x + 2, area.y, &tt, t.title().bg(bg));
            if let Some(m) = self.meta {
                let mt = format!(" {m} ");
                let mw = width(&mt) as u16;
                if area.width > width(&tt) as u16 + mw + 4 {
                    buf.set_string(area.right() - 2 - mw, area.y, &mt, t.faint().bg(bg));
                }
            }
        }
        if !ctx.inert {
            ctx.hits.register(WidgetId::of("modal.surface"), area);
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
}
