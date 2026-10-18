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

/// Consumption thresholds shared by every meter: a value at or below
/// `METER_LOW_MAX` is healthy, up to `METER_MEDIUM_MAX` needs attention,
/// above that it is danger. Owners with a domain status (quota warning,
/// exhausted) pass that status instead of recomputing these.
pub const METER_LOW_MAX: u8 = 59;
pub const METER_MEDIUM_MAX: u8 = 84;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterLevel {
    Low,
    Medium,
    High,
}

impl MeterLevel {
    pub fn of(pct: u8) -> Self {
        if pct <= METER_LOW_MAX {
            MeterLevel::Low
        } else if pct <= METER_MEDIUM_MAX {
            MeterLevel::Medium
        } else {
            MeterLevel::High
        }
    }
}

/// How the meter draws its track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterVisual {
    /// Compact `━━━━────` run followed by the value.
    #[default]
    Line,
    /// The used share is a filled background block with the value inside;
    /// the remaining share is a subtle block, so the bar reads as filled.
    Block,
}

/// Semantic state of a meter. `Normal` derives the level from the value;
/// the explicit variants come from the owner's domain (a quota status,
/// a refresh in flight, a failed read).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeterTone {
    /// Healthy / attention / danger by the shared thresholds.
    #[default]
    Normal,
    /// An owner-chosen level (when the domain classifies differently).
    Level(MeterLevel),
    /// The domain says warning: warning run and `▲`.
    Warning,
    /// The domain says exhausted: error run, bold `!`.
    Exhausted,
    /// Last-good data: faint run, muted text.
    Stale,
    /// A refresh is in flight: muted run, spinner in place of the value.
    Refreshing,
    /// The read failed: no run, `!` and the message in the error tone.
    Error,
    /// No quota to show: no run, a faint `—`.
    Unknown,
}

impl MeterTone {
    /// The level a tone renders with, given the value.
    pub fn level(self, used_pct: Option<u8>) -> Option<MeterLevel> {
        match self {
            MeterTone::Normal => used_pct.map(MeterLevel::of),
            MeterTone::Level(l) => Some(l),
            MeterTone::Warning => Some(MeterLevel::Medium),
            MeterTone::Exhausted => Some(MeterLevel::High),
            _ => None,
        }
    }
}

/// A capacity meter: value, semantic tone and visual mode. Green is never
/// used: a quota is not a completion.
///
/// ```text
/// Session   ━━━━━━━━━━━───────────  38%     (Line)
/// Session   ▐ 38% used ▌            ▲       (Block: used share filled)
/// ```
#[derive(Debug, Clone)]
pub struct Meter {
    pub used_pct: Option<u8>,
    pub value: String,
    pub tone: MeterTone,
    pub visual: MeterVisual,
}

impl Meter {
    pub fn new(used_pct: Option<u8>) -> Self {
        Self {
            used_pct,
            value: used_pct.map(|p| format!("{p}%")).unwrap_or_default(),
            tone: MeterTone::Normal,
            visual: MeterVisual::Line,
        }
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn tone(mut self, tone: MeterTone) -> Self {
        self.tone = tone;
        self
    }
    pub fn visual(mut self, visual: MeterVisual) -> Self {
        self.visual = visual;
        self
    }

    /// (run colour, text colour, two-cell suffix, whether a run is drawn)
    fn palette(&self, t: &crate::theme::Theme) -> (Color, Color, &'static str, bool) {
        let level_color = |l: MeterLevel| match l {
            MeterLevel::Low => t.text_secondary,
            MeterLevel::Medium => t.warning,
            MeterLevel::High => t.error,
        };
        match self.tone {
            MeterTone::Normal | MeterTone::Level(_) => {
                let l = self.tone.level(self.used_pct).unwrap_or(MeterLevel::Low);
                let c = level_color(l);
                let text = if l == MeterLevel::Low {
                    t.text_primary
                } else {
                    c
                };
                (c, text, "  ", self.used_pct.is_some())
            }
            MeterTone::Warning => (t.warning, t.warning, " ▲", self.used_pct.is_some()),
            MeterTone::Exhausted => (t.error, t.error, " !", true),
            MeterTone::Stale => (t.text_faint, t.text_muted, "  ", self.used_pct.is_some()),
            MeterTone::Refreshing => (t.text_muted, t.text_muted, "  ", self.used_pct.is_some()),
            MeterTone::Error => (t.error, t.error, " !", false),
            MeterTone::Unknown => (t.text_faint, t.text_faint, "  ", false),
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let (fill_color, text_color, suffix, has_run) = self.palette(t);
        let value = match self.tone {
            MeterTone::Refreshing => format!("{} refreshing", spinner_frame(ctx.interaction.tick)),
            MeterTone::Unknown if self.value.is_empty() => "—".to_owned(),
            _ => self.value.clone(),
        };
        let ratio = (self.used_pct.unwrap_or(0).min(100) as f64) / 100.0;
        let vw = width(&value) as u16;
        let text_style = ratatui::style::Style::new().fg(text_color).bg(bg);
        let mut suffix_style = ratatui::style::Style::new().fg(fill_color).bg(bg);
        if self.tone == MeterTone::Exhausted {
            suffix_style = suffix_style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        if !has_run {
            // no track: the message and the marker only
            let text = crate::ui::text::truncate(&value, area.width.saturating_sub(2) as usize);
            buf.set_string(area.x, area.y, &text, text_style);
            let sx = area.x + width(&text) as u16;
            if sx + 2 <= area.right() {
                buf.set_string(sx, area.y, suffix, suffix_style);
            }
            return;
        }
        match self.visual {
            MeterVisual::Line => {
                // value + suffix always fit; the track takes what is left
                let track_w = area.width.saturating_sub(vw + 3);
                if track_w < 6 {
                    buf.set_string(
                        area.x,
                        area.y,
                        crate::ui::text::truncate(&value, area.width as usize),
                        text_style,
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
                buf.set_string(x, area.y, &value, text_style);
                buf.set_string(x + vw, area.y, suffix, suffix_style);
            }
            MeterVisual::Block => {
                // the whole width minus the marker is the bar; the value sits
                // inside it so the used share reads as a filled block
                let bar_w = area.width.saturating_sub(2);
                if bar_w < 4 {
                    buf.set_string(
                        area.x,
                        area.y,
                        crate::ui::text::truncate(&value, area.width as usize),
                        text_style,
                    );
                    return;
                }
                let filled = ((bar_w as f64) * ratio).round() as u16;
                let rest_bg = t.lift(bg);
                let on_fill = if self.tone == MeterTone::Stale {
                    t.text_secondary
                } else {
                    t.canvas
                };
                let on_rest = if matches!(self.tone, MeterTone::Stale | MeterTone::Refreshing) {
                    t.text_muted
                } else {
                    t.text_primary
                };
                let text = crate::ui::text::truncate(&value, bar_w.saturating_sub(2) as usize);
                let chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
                for i in 0..bar_w {
                    let in_fill = i < filled;
                    let cell_bg = if in_fill { fill_color } else { rest_bg };
                    let fg = if in_fill { on_fill } else { on_rest };
                    let mut st = ratatui::style::Style::new().fg(fg).bg(cell_bg);
                    if in_fill {
                        st = st.add_modifier(ratatui::style::Modifier::BOLD);
                    }
                    // text starts one cell in
                    let sym = if i >= 1 && (i as usize - 1) < chars.len() {
                        chars[i as usize - 1].as_str()
                    } else {
                        " "
                    };
                    buf.set_string(area.x + i, area.y, sym, st);
                }
                buf.set_string(area.x + bar_w, area.y, suffix, suffix_style);
            }
        }
    }
}

/// Line-mode quota meter with a known value: the compact form of
/// [`Meter`]. Kept for callers that only need the default visual.
pub fn render_meter(
    area: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    used_pct: u8,
    value: &str,
    tone: MeterTone,
    bg: Color,
) {
    Meter::new(Some(used_pct))
        .value(value)
        .tone(tone)
        .render(area, buf, ctx, bg);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn render(m: &Meter, w: u16) -> (Buffer, Theme) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 1));
        m.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, theme.canvas);
        (buf, theme)
    }

    fn text(buf: &Buffer) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, 0)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn levels_follow_the_shared_thresholds() {
        assert_eq!(MeterLevel::of(0), MeterLevel::Low);
        assert_eq!(MeterLevel::of(59), MeterLevel::Low);
        assert_eq!(MeterLevel::of(60), MeterLevel::Medium);
        assert_eq!(MeterLevel::of(84), MeterLevel::Medium);
        assert_eq!(MeterLevel::of(85), MeterLevel::High);
        assert_eq!(MeterLevel::of(100), MeterLevel::High);
        assert_eq!(MeterTone::Normal.level(Some(70)), Some(MeterLevel::Medium));
        assert_eq!(MeterTone::Stale.level(Some(70)), None);
    }

    #[test]
    fn line_mode_draws_runs_with_the_level_colour() {
        let (buf, t) = render(&Meter::new(Some(50)).value("50%"), 30);
        let s = text(&buf);
        assert!(s.contains("━"));
        assert!(s.contains("─"));
        assert!(s.ends_with("50%  "));
        assert_eq!(buf[(0, 0)].fg, t.text_secondary, "low is the white ladder");
        let (buf, t) = render(&Meter::new(Some(70)).value("70%"), 30);
        assert_eq!(buf[(0, 0)].fg, t.warning, "medium is the warning tone");
        let (buf, t) = render(&Meter::new(Some(95)).value("95%"), 30);
        assert_eq!(buf[(0, 0)].fg, t.error, "high is the error tone");
    }

    #[test]
    fn block_mode_fills_the_used_share_as_background() {
        let m = Meter::new(Some(50))
            .value("50% used")
            .visual(MeterVisual::Block);
        let (buf, t) = render(&m, 22);
        // bar is 20 cells, half filled
        assert_eq!(buf[(0, 0)].bg, t.text_secondary);
        assert_eq!(buf[(9, 0)].bg, t.text_secondary);
        assert_eq!(buf[(10, 0)].bg, t.lift(t.canvas));
        assert_eq!(buf[(19, 0)].bg, t.lift(t.canvas));
        assert!(
            text(&buf).starts_with(" 50% used"),
            "value sits inside the bar"
        );
        assert_eq!(buf[(1, 0)].fg, t.canvas, "text on the filled block is dark");
    }

    #[test]
    fn domain_states_render_their_markers() {
        let (buf, t) = render(
            &Meter::new(Some(82)).value("82%").tone(MeterTone::Warning),
            30,
        );
        assert!(text(&buf).ends_with("82% ▲"));
        assert_eq!(buf[(0, 0)].fg, t.warning);
        let (buf, t) = render(
            &Meter::new(Some(100))
                .value("100%")
                .tone(MeterTone::Exhausted),
            30,
        );
        assert!(text(&buf).ends_with("100% !"));
        assert_eq!(buf[(0, 0)].fg, t.error);
        assert!(
            buf[(29, 0)]
                .modifier
                .contains(ratatui::style::Modifier::BOLD)
        );
        let (buf, t) = render(
            &Meter::new(Some(54)).value("54%").tone(MeterTone::Stale),
            30,
        );
        assert_eq!(buf[(0, 0)].fg, t.text_faint);
        let (buf, t) = render(
            &Meter::new(None).value("read failed").tone(MeterTone::Error),
            30,
        );
        assert!(text(&buf).starts_with("read failed !"));
        assert!(!text(&buf).contains("━"));
        assert_eq!(buf[(0, 0)].fg, t.error);
        let (buf, _) = render(&Meter::new(None).tone(MeterTone::Unknown), 30);
        assert!(text(&buf).starts_with("—"));
        let (buf, _) = render(&Meter::new(Some(54)).tone(MeterTone::Refreshing), 30);
        assert!(text(&buf).contains("refreshing"));
    }
}
