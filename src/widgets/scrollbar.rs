use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::RenderCtx;

pub const TRACK: &str = "│";
pub const THUMB: &str = "┃";

/// Id used for a container's scrollbar hit region.
pub fn id_for(container: WidgetId) -> WidgetId {
    container.sub("scrollbar")
}

/// Render a vertical scrollbar into a 1-column area. Draws nothing when the
/// content fits. Registers a clickable region for track clicks and drags.
pub fn render_vertical(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    container: WidgetId,
    scroll: &ScrollState,
    focused: bool,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() || !scroll.overflows() {
        return;
    }
    let t = ctx.theme;
    let sb_id = id_for(container);
    let hovered = ctx.interaction.hovered(sb_id) || ctx.interaction.pressed == Some(sb_id);
    let track_len = area.height as usize;
    let (start, len) = scroll.thumb(track_len);
    for i in 0..track_len {
        let y = area.y + i as u16;
        if i >= start && i < start + len {
            buf.set_string(area.x, y, THUMB, t.scrollbar_thumb(focused, hovered));
        } else {
            buf.set_string(area.x, y, TRACK, t.scrollbar_track());
        }
    }
    ctx.clickable(sb_id, area);
}

/// Map a pointer position on a vertical track to a scroll offset.
pub fn offset_for_click(track: Rect, pos: Position, scroll: &ScrollState) -> usize {
    let rel = pos.y.saturating_sub(track.y) as usize;
    scroll.offset_for_track_pos(rel, track.height as usize)
}

/// "12–24 of 120" style position label.
pub fn position_label(scroll: &ScrollState) -> String {
    if !scroll.overflows() {
        return String::new();
    }
    let r = scroll.visible_range();
    format!("{}–{} of {}", r.start + 1, r.end, scroll.content_len)
}
