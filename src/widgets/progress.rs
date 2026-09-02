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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterTone {
    #[default]
    Normal,
    Warning,
    Exhausted,
    /// Last-good data: the fill drops to faint.
    Stale,
}

/// Quota meter: `━━━━━━───── 62%` with the fill on the white ladder and a
/// two-cell tone suffix (`▲` warning, `!` exhausted). `value` is the text
/// shown after the track (`62% used`, `1,240 / 5,000 credits`). Green is
/// never used: a quota is not a completion.
pub fn render_meter(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    used_pct: u8,
    value: &str,
    tone: MeterTone,
    bg: Color,
) {
    let area = area.intersection(*buf.area());
    if area.is_empty() {
        return;
    }
    let t = ctx.theme;
    let ratio = (used_pct.min(100) as f64) / 100.0;
    let vw = width(value) as u16;
    let (fill_color, text_color, suffix) = match tone {
        MeterTone::Normal => (t.text_secondary, t.text_primary, "  "),
        MeterTone::Warning => (t.warning, t.warning, " ▲"),
        MeterTone::Exhausted => (t.error, t.error, " !"),
        MeterTone::Stale => (t.text_faint, t.text_muted, "  "),
    };
    // value + suffix always fit; the track takes what is left
    let track_w = area.width.saturating_sub(vw + 3);
    if track_w < 6 {
        buf.set_string(
            area.x,
            area.y,
            crate::ui::text::truncate(value, area.width as usize),
            ratatui::style::Style::new().fg(text_color).bg(bg),
        );
        return;
    }
    let filled = ((track_w as f64) * ratio).round() as u16;
    for i in 0..track_w {
        let (sym, st) = if i < filled {
            ("━", ratatui::style::Style::new().fg(fill_color).bg(bg))
        } else {
            ("─", ratatui::style::Style::new().fg(t.border_subtle).bg(bg))
        };
        buf.set_string(area.x + i, area.y, sym, st);
    }
    let x = area.x + track_w + 1;
    buf.set_string(
        x,
        area.y,
        value,
        ratatui::style::Style::new().fg(text_color).bg(bg),
    );
    let sx = x + vw;
    let mut st = ratatui::style::Style::new().fg(fill_color).bg(bg);
    if tone == MeterTone::Exhausted {
        st = st.add_modifier(ratatui::style::Modifier::BOLD);
    }
    buf.set_string(sx, area.y, suffix, st);
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
