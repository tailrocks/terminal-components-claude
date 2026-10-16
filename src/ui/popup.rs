//! Anchored, non-modal popups (completion lists, filter editors, pickers).
//!
//! A popup is drawn last, on top of everything, and claims a hit barrier so
//! nothing beneath it is clickable. Unlike a dialog it does not dim the page
//! and does not necessarily trap focus: the owner decides which keys it
//! swallows.
//!
//! Dismissal is a policy, not page code: [`Dismiss`] says whether Esc and a
//! click outside close a popup or a dialog. A popup that light-dismisses
//! registers a backdrop region over the whole screen, owned by its widget
//! ([`backdrop_id`]), so a click anywhere outside reaches the owner instead
//! of falling into a dead zone under the hit barrier.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};

/// How a popup or a dialog may be closed without choosing anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dismiss {
    /// Esc closes it.
    pub escape: bool,
    /// A click outside it (on the page, or on a dialog's dimmed backdrop)
    /// closes it.
    pub outside_click: bool,
    /// The outside click that closes a popup also reaches the control
    /// beneath it (Windows-style). Off (the default, as with macOS menus
    /// and popovers): the click only closes. Dialogs are modal and ignore
    /// it: nothing beneath a dialog is reachable.
    pub pass_through: bool,
}

impl Dismiss {
    /// Esc and an outside click both close (menus, selects, plain dialogs).
    pub const LIGHT: Self = Self {
        escape: true,
        outside_click: true,
        pass_through: false,
    };
    /// Only Esc closes: a stray click cannot throw away a decision in
    /// progress (destructive confirmations, typed acknowledgements).
    pub const ESCAPE: Self = Self {
        escape: true,
        outside_click: false,
        pass_through: false,
    };
    /// Only an explicit action closes it.
    pub const NEVER: Self = Self {
        escape: false,
        outside_click: false,
        pass_through: false,
    };
}

impl Dismiss {
    /// This policy with the outside click also reaching what is beneath.
    pub const fn pass_through(mut self, on: bool) -> Self {
        self.pass_through = on;
        self
    }
}

impl Default for Dismiss {
    fn default() -> Self {
        Self::LIGHT
    }
}

/// The id a light-dismiss backdrop registers for `owner`. A completed click
/// on it means "clicked outside the popup".
pub const fn backdrop_id(owner: WidgetId) -> WidgetId {
    owner.sub("popup.backdrop")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Below the anchor, flipping above when there is no room.
    Below,
    /// Centered on the screen (quick switcher).
    Center,
}

/// Register `owner`'s light-dismiss backdrop over `screen` per `dismiss`.
/// Call it right after pushing the popup's hit barrier.
pub fn backdrop(ctx: &mut RenderCtx, owner: WidgetId, dismiss: Dismiss, screen: Rect) {
    if dismiss.outside_click && !ctx.inert {
        ctx.hits
            .register_backdrop(backdrop_id(owner), screen, dismiss.pass_through);
    }
}

/// Vertical position of a [`Placement::Center`] popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    /// Centred vertically, like a dialog (default).
    Center,
    /// In the upper third of the screen, the quick-switcher position.
    Third,
}

/// How a popup is fitted to the screen, beyond its [`Placement`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    /// Cells kept free between the popup and every edge of the screen, so
    /// its frame never lands on a panel border underneath ("││", "╰╰").
    /// Given up only when the popup would not fit otherwise.
    pub margin: u16,
    /// Vertical position of a centred popup.
    pub v_align: VAlign,
    /// Columns between the popup's left edge and its text (frame plus
    /// padding). An anchored popup moves left by this much so its text
    /// starts in the anchor's column (completion under typed text).
    pub text_inset: u16,
}

impl Place {
    /// The default: one free cell from every edge, centred, no inset.
    pub const DEFAULT: Self = Self {
        margin: 1,
        v_align: VAlign::Center,
        text_inset: 0,
    };
    /// Flush with the screen edges, for tooltips drawn over the very cell
    /// they explain.
    pub const FLUSH: Self = Self {
        margin: 0,
        v_align: VAlign::Center,
        text_inset: 0,
    };

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

/// Compute the popup rectangle for a desired size next to `anchor` inside
/// `screen`, with the [`Place::DEFAULT`] fit. See [`place_with`].
pub fn place(screen: Rect, anchor: Rect, width: u16, height: u16, placement: Placement) -> Rect {
    place_with(screen, anchor, width, height, placement, Place::DEFAULT)
}

/// Compute the popup rectangle for a desired size next to `anchor` inside
/// `screen`. Never returns a rect outside the screen. An anchored popup
/// never covers its anchor: it goes below, flips above, or, when neither
/// side has the full height, takes the roomier side and gets shorter (the
/// owner scrolls its content). `fit` keeps a margin from the screen edges
/// when there is room for it, aligns a centred popup, and lines an
/// anchored popup's text up with its anchor.
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
    // the margin is given up, never the popup's size
    let mx = fit.margin.min(screen.width.saturating_sub(width) / 2);
    let my = fit.margin.min(screen.height.saturating_sub(height) / 2);
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
        Placement::Below => {
            let below = anchor.bottom().clamp(bounds.y, bounds.bottom());
            let room_below = bounds.bottom().saturating_sub(below);
            let room_above = anchor.y.saturating_sub(bounds.y).min(bounds.height);
            let (y, height) = if room_below >= height {
                (below, height)
            } else if room_above >= height {
                (anchor.y - height, height)
            } else if room_below >= room_above && room_below > 0 {
                (below, room_below)
            } else if room_above > 0 {
                (bounds.y, room_above)
            } else {
                (bounds.bottom().saturating_sub(height), height)
            };
            let x = anchor
                .x
                .saturating_sub(fit.text_inset)
                .min(bounds.right().saturating_sub(width))
                .max(bounds.x);
            Rect::new(x, y, width, height)
        }
    }
}

/// Draw the popup surface (elevated, strong border on the top edge only is
/// too fussy: popups use the elevated plane and a subtle frame) and claim
/// the hit barrier. Returns the inner content area.
pub fn surface(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, t: &Theme) -> Rect {
    surface_for(area, buf, ctx, t, None)
}

/// [`surface`] for a popup owned by `owner` under `dismiss`: with
/// `outside_click`, every cell outside the popup becomes the owner's
/// [`backdrop_id`] region, so a click outside reaches the owner.
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
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(t.border(true).bg(t.surface_elevated));
    ratatui::widgets::Widget::render(block, area, buf);
    // everything registered before the popup is unreachable by the mouse;
    // keyboard focus is left to the owner
    ctx.hits.push_barrier();
    if let Some((owner, dismiss)) = owner
        && dismiss.outside_click
    {
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
        // one free cell from the screen edge
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
        // 11 rows below, 6 above (one row kept off each edge): below
        // wins, 11 rows tall
        let anchor = Rect::new(4, 7, 20, 1);
        let r = place(screen, anchor, 30, 16, Placement::Below);
        assert_eq!(r, Rect::new(4, 8, 30, 11));
        // 3 rows below, 14 above: above wins, 14 rows tall
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
        // an anchor in the corner: the popup moves one cell in
        let r = place(screen, Rect::new(0, 0, 1, 1), 20, 4, Placement::Below);
        assert_eq!((r.x, r.y), (1, 1));
        // a popup as wide as the screen keeps its width
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
}
