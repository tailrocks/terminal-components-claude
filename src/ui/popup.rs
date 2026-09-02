//! Anchored, non-modal popups (completion lists, filter editors, pickers).
//!
//! A popup is drawn last, on top of everything, and claims a hit barrier so
//! nothing beneath it is clickable. Unlike a dialog it does not dim the page
//! and does not necessarily trap focus: the owner decides which keys it
//! swallows.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::theme::Theme;
use crate::ui::ctx::{RenderCtx, fill};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Below the anchor, flipping above when there is no room.
    Below,
    /// Centered on the screen (quick switcher).
    Center,
}

/// Compute the popup rectangle for a desired size next to `anchor` inside
/// `screen`. Never returns a rect outside the screen.
pub fn place(screen: Rect, anchor: Rect, width: u16, height: u16, placement: Placement) -> Rect {
    let width = width.min(screen.width).max(1);
    let height = height.min(screen.height).max(1);
    match placement {
        Placement::Center => {
            let x = screen.x + screen.width.saturating_sub(width) / 2;
            let y = screen.y + (screen.height.saturating_sub(height) / 3).max(1);
            Rect::new(
                x,
                y.min(screen.bottom().saturating_sub(height)),
                width,
                height,
            )
        }
        Placement::Below => {
            let below = anchor.bottom();
            let room_below = screen.bottom().saturating_sub(below);
            let y = if room_below >= height {
                below
            } else if anchor.y >= screen.y + height {
                anchor.y - height
            } else {
                screen.bottom().saturating_sub(height)
            };
            let x = anchor
                .x
                .min(screen.right().saturating_sub(width))
                .max(screen.x);
            Rect::new(x, y, width, height)
        }
    }
}

/// Draw the popup surface (elevated, strong border on the top edge only is
/// too fussy: popups use the elevated plane and a subtle frame) and claim
/// the hit barrier. Returns the inner content area.
pub fn surface(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, t: &Theme) -> Rect {
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
    ctx.hits
        .register(crate::core::id::WidgetId::of("popup.surface"), area);
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
        assert_eq!(place(screen, right, 40, 8, Placement::Below).right(), 100);
        let tall = place(screen, anchor, 40, 60, Placement::Below);
        assert!(tall.height <= 30);
    }

    #[test]
    fn centers_in_upper_third() {
        let screen = Rect::new(0, 0, 120, 40);
        let r = place(screen, Rect::ZERO, 60, 12, Placement::Center);
        assert_eq!(r.x, 30);
        assert_eq!(r.y, 9);
    }
}
