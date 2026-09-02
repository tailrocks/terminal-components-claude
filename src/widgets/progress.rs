use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;

pub const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn spinner_frame(tick: u64) -> &'static str {
    SPINNER[(tick % SPINNER.len() as u64) as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStatus {
    Active,
    Done,
    Error,
    Paused,
}

/// Determinate progress bar: `label ━━━━━━───── 64%`.
pub fn render_bar(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    label: &str,
    ratio: f64,
    status: ProgressStatus,
    bg: Color,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let t = ctx.theme;
    let ratio = ratio.clamp(0.0, 1.0);
    let pct = format!("{:>4}", format!("{}%", (ratio * 100.0).round() as u32));
    let label_w = width(label) as u16;
    let has_label = label_w > 0 && area.width > label_w + 8;
    let mut x = area.x;
    if has_label {
        buf.set_string(x, area.y, label, t.primary().bg(bg));
        x += label_w + 2;
    }
    let pct_w = 5u16;
    // fixed 2-cell trailing glyph column keeps percentages aligned
    let suffix = match status {
        ProgressStatus::Done => " ✓",
        ProgressStatus::Error => " !",
        ProgressStatus::Paused => " ‖",
        ProgressStatus::Active => "  ",
    };
    let track_w = area.right().saturating_sub(x).saturating_sub(pct_w + 2);
    if track_w < 6 {
        // too narrow for a meaningful bar: percentage only
        buf.set_string(x, area.y, pct.trim_start(), t.secondary().bg(bg));
        return;
    }
    let filled = ((track_w as f64) * ratio).round() as u16;
    // green is reserved for completion; a running bar is white 70 %
    let fill_color = match status {
        ProgressStatus::Active => t.text_secondary,
        ProgressStatus::Done => t.success,
        ProgressStatus::Error => t.error,
        ProgressStatus::Paused => t.text_muted,
    };
    for i in 0..track_w {
        let (sym, st) = if i < filled {
            ("━", ratatui::style::Style::new().fg(fill_color).bg(bg))
        } else {
            ("─", ratatui::style::Style::new().fg(t.border_subtle).bg(bg))
        };
        buf.set_string(x + i, area.y, sym, st);
    }
    x += track_w;
    buf.set_string(x, area.y, format!(" {pct}"), t.secondary().bg(bg));
    let st = ratatui::style::Style::new().fg(fill_color).bg(bg);
    buf.set_string(x + pct_w, area.y, suffix, st);
}

/// Indeterminate bar: a short accent segment sweeping over the track.
pub fn render_indeterminate(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    label: &str,
    bg: Color,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let t = ctx.theme;
    let label_w = width(label) as u16;
    let has_label = label_w > 0 && area.width > label_w + 8;
    let mut x = area.x;
    if has_label {
        buf.set_string(x, area.y, label, t.primary().bg(bg));
        x += label_w + 2;
    }
    let track_w = area.right().saturating_sub(x).max(1) as i64;
    let seg = (track_w / 5).clamp(2, 8);
    let period = track_w + seg;
    let pos = (ctx.interaction.tick as i64 % period) - seg;
    for i in 0..track_w {
        let in_seg = i >= pos && i < pos + seg;
        let (sym, st) = if in_seg {
            ("━", ratatui::style::Style::new().fg(t.accent).bg(bg))
        } else {
            ("─", ratatui::style::Style::new().fg(t.border_subtle).bg(bg))
        };
        buf.set_string(x + i as u16, area.y, sym, st);
    }
}

/// Compact activity state: `⠋ label`.
pub fn render_spinner(area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, label: &str, bg: Color) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let t = ctx.theme;
    buf.set_string(
        area.x,
        area.y,
        spinner_frame(ctx.interaction.tick),
        t.accent_fg().bg(bg),
    );
    buf.set_string(area.x + 2, area.y, label, t.secondary().bg(bg));
}
