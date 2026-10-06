//! Determinate, indeterminate and compact activity indicators.

use std::time::Duration;

use ratatui::style::{Color, Modifier, Style};
use termrock::{
    Button, Constraints, Cx, FgStep, FrameRead, Id, Moment, Panel, Part, Rect, Response, Role,
    StylePatch, Ui, Variant, id,
};

use super::{Page, PageStatus, PageUpdate, frame};

const LIVE_PANEL: Id = id!("progress.live.panel");
const STATES_PANEL: Id = id!("progress.states.panel");
const RESTART: Id = id!("progress.restart");
const PAUSE: Id = id!("progress.pause");

const PANEL_PARTS: &[(Part, StylePatch)] = &[
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(Modifier::BOLD),
    ),
    (
        Part::DETAIL,
        StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
    ),
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterTone {
    Normal,
    Warning,
    Exhausted,
    Stale,
    Refreshing,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterVisual {
    Line,
    Block,
}

fn restart_button() -> Button<'static> {
    Button::new(RESTART, "Restart").variant(Variant::SECONDARY)
}

fn pause_button(paused: bool) -> Button<'static> {
    Button::new(PAUSE, if paused { "Resume" } else { "Pause" }).variant(Variant::SECONDARY)
}

fn live_panel() -> Panel<'static> {
    Panel::new(LIVE_PANEL)
        .title("Live")
        .meta("ticks at 80 ms")
        .patch_part(PANEL_PARTS)
}

fn states_panel() -> Panel<'static> {
    Panel::new(STATES_PANEL)
        .title("States")
        .meta("static")
        .patch_part(PANEL_PARTS)
}

#[derive(Clone, Copy)]
struct ThemeColors {
    text_primary: Color,
    text_secondary: Color,
    text_muted: Color,
    text_faint: Color,
    accent: Color,
    border_subtle: Color,
    success: Color,
    error: Color,
    warning: Color,
    canvas: Color,
    surface: Color,
    surface_elevated: Color,
    surface_overlay: Color,
}

impl ThemeColors {
    fn from_ui(ui: &Ui<'_>) -> Self {
        let t = ui.theme();
        Self {
            text_primary: t.color.fg[0],
            text_secondary: t.color.fg[1],
            text_muted: t.color.fg[2],
            text_faint: t.color.fg[3],
            accent: t.color.accent,
            border_subtle: t.color.border_subtle,
            success: t.color.success,
            error: t.color.danger,
            warning: t.color.warning,
            canvas: t.color.surfaces[0],
            surface: t.color.surfaces[1],
            surface_elevated: t.color.surfaces[2],
            surface_overlay: t.color.surfaces[3],
        }
    }

    #[expect(clippy::if_same_then_else)]
    fn lift(&self, bg: Color) -> Color {
        if bg == self.canvas {
            self.surface_elevated
        } else if bg == self.surface || bg == self.surface_elevated {
            self.surface_overlay
        } else {
            self.surface_overlay
        }
    }
}

fn fixed_rows<const N: usize>(area: Rect, heights: &[u16; N]) -> [Rect; N] {
    let mut y = area.y;
    let mut rows = [Rect::default(); N];
    for (index, (row, height)) in rows.iter_mut().zip(heights.iter().copied()).enumerate() {
        let height = if index.saturating_add(1) == heights.len() {
            area.bottom().saturating_sub(y)
        } else {
            height.min(area.bottom().saturating_sub(y))
        };
        *row = Rect {
            x: area.x,
            y,
            width: area.width,
            height,
        };
        y = y.saturating_add(height);
    }
    rows
}

fn row_layout(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    let mut x = area.x;
    let mut out = Vec::new();
    for &w in widths {
        let w = w.min(area.right().saturating_sub(x));
        out.push(Rect::new(x, area.y, w, area.height.min(1)));
        x = x.saturating_add(w).saturating_add(gap);
    }
    out
}

fn render_bar(
    ui: &mut Ui<'_>,
    area: Rect,
    label: &str,
    ratio: f64,
    status: ProgressStatus,
    colors: &ThemeColors,
    bg: Color,
) {
    if area.is_empty() {
        return;
    }
    let ratio = ratio.clamp(0.0, 1.0);
    let pct = format!("{:>4}", format!("{}%", (ratio * 100.0).round() as u32));
    let label_w = termrock::width(label);
    let has_label = label_w > 0 && area.width > label_w + 8;
    let mut x = area.x;
    if has_label {
        let primary_st = Style::new().fg(colors.text_primary).bg(bg);
        let _ = ui.paint_str(Rect::new(x, area.y, label_w, 1), label, primary_st);
        x += label_w + 2;
    }
    let pct_w = 5u16;
    let suffix = match status {
        ProgressStatus::Done => " ✓",
        ProgressStatus::Error => " !",
        ProgressStatus::Paused => " ‖",
        ProgressStatus::Active => "  ",
    };
    let track_w = area.right().saturating_sub(x).saturating_sub(pct_w + 2);
    if track_w < 6 {
        let sec_st = Style::new().fg(colors.text_secondary).bg(bg);
        let _ = ui.paint_str(
            Rect::new(x, area.y, area.right().saturating_sub(x), 1),
            pct.trim_start(),
            sec_st,
        );
        return;
    }
    let filled = ((track_w as f64) * ratio).round() as u16;
    let fill_color = match status {
        ProgressStatus::Active => colors.text_secondary,
        ProgressStatus::Done => colors.success,
        ProgressStatus::Error => colors.error,
        ProgressStatus::Paused => colors.text_muted,
    };
    let fill_st = Style::new().fg(fill_color).bg(bg);
    let subtle_st = Style::new().fg(colors.border_subtle).bg(bg);
    for i in 0..track_w {
        let (sym, st) = if i < filled {
            ("━", fill_st)
        } else {
            ("─", subtle_st)
        };
        let _ = ui.paint_str(Rect::new(x + i, area.y, 1, 1), sym, st);
    }
    x += track_w;
    let sec_st = Style::new().fg(colors.text_secondary).bg(bg);
    let _ = ui.paint_str(Rect::new(x, area.y, pct_w, 1), &format!(" {pct}"), sec_st);
    let _ = ui.paint_str(Rect::new(x + pct_w, area.y, 2, 1), suffix, fill_st);
}

fn render_indeterminate(
    ui: &mut Ui<'_>,
    area: Rect,
    label: &str,
    tick: u64,
    colors: &ThemeColors,
    bg: Color,
) {
    if area.is_empty() {
        return;
    }
    let label_w = termrock::width(label);
    let has_label = label_w > 0 && area.width > label_w + 8;
    let mut x = area.x;
    if has_label {
        let primary_st = Style::new().fg(colors.text_primary).bg(bg);
        let _ = ui.paint_str(Rect::new(x, area.y, label_w, 1), label, primary_st);
        x += label_w + 2;
    }
    let track_w = (area.right().saturating_sub(x).max(1)) as i64;
    let seg = (track_w / 5).clamp(2, 8);
    let period = track_w + seg;
    let pos = (tick as i64 % period) - seg;
    let accent_st = Style::new().fg(colors.accent).bg(bg);
    let subtle_st = Style::new().fg(colors.border_subtle).bg(bg);
    for i in 0..track_w {
        let in_seg = i >= pos && i < pos + seg;
        let (sym, st) = if in_seg {
            ("━", accent_st)
        } else {
            ("─", subtle_st)
        };
        let _ = ui.paint_str(Rect::new(x + i as u16, area.y, 1, 1), sym, st);
    }
}

fn render_spinner(
    ui: &mut Ui<'_>,
    area: Rect,
    label: &str,
    tick: u64,
    colors: &ThemeColors,
    bg: Color,
) {
    if area.is_empty() {
        return;
    }
    let accent_st = Style::new().fg(colors.accent).bg(bg);
    let sec_st = Style::new().fg(colors.text_secondary).bg(bg);
    let _ = ui.paint_str(
        Rect::new(area.x, area.y, 1, 1),
        spinner_frame(tick),
        accent_st,
    );
    let _ = ui.paint_str(
        Rect::new(area.x + 2, area.y, termrock::width(label), 1),
        label,
        sec_st,
    );
}

#[expect(clippy::too_many_arguments)]
fn render_meter(
    ui: &mut Ui<'_>,
    area: Rect,
    used_pct: Option<u8>,
    value: &str,
    tone: MeterTone,
    visual: MeterVisual,
    tick: u64,
    colors: &ThemeColors,
    bg: Color,
) {
    if area.is_empty() {
        return;
    }
    let (fill_color, text_color, suffix, has_run) = match tone {
        MeterTone::Normal => {
            let l = used_pct.unwrap_or(0);
            let c = if l <= 59 {
                colors.text_secondary
            } else if l <= 84 {
                colors.warning
            } else {
                colors.error
            };
            let text = if l <= 59 { colors.text_primary } else { c };
            (c, text, "  ", used_pct.is_some())
        }
        MeterTone::Warning => (colors.warning, colors.warning, " ▲", used_pct.is_some()),
        MeterTone::Exhausted => (colors.error, colors.error, " !", true),
        MeterTone::Stale => (
            colors.text_faint,
            colors.text_muted,
            "  ",
            used_pct.is_some(),
        ),
        MeterTone::Refreshing => (
            colors.text_muted,
            colors.text_muted,
            "  ",
            used_pct.is_some(),
        ),
        MeterTone::Error => (colors.error, colors.error, " !", false),
        MeterTone::Unknown => (colors.text_faint, colors.text_faint, "  ", false),
    };
    let value_str = match tone {
        MeterTone::Refreshing => format!("{} refreshing", spinner_frame(tick)),
        MeterTone::Unknown if value.is_empty() => "—".to_owned(),
        _ => value.to_string(),
    };
    let ratio = (used_pct.unwrap_or(0).min(100) as f64) / 100.0;
    let vw = termrock::width(&value_str);
    let text_style = Style::new().fg(text_color).bg(bg);
    let mut suffix_style = Style::new().fg(fill_color).bg(bg);
    if tone == MeterTone::Exhausted {
        suffix_style = suffix_style.add_modifier(Modifier::BOLD);
    }
    if !has_run {
        let text = termrock::truncate(&value_str, area.width.saturating_sub(2));
        let tw = termrock::width(&text);
        let _ = ui.paint_str(Rect::new(area.x, area.y, tw, 1), &text, text_style);
        let sx = area.x + tw;
        if sx + 2 <= area.right() {
            let _ = ui.paint_str(Rect::new(sx, area.y, 2, 1), suffix, suffix_style);
        }
        return;
    }
    match visual {
        MeterVisual::Line => {
            let track_w = area.width.saturating_sub(vw + 3);
            if track_w < 6 {
                let text = termrock::truncate(&value_str, area.width);
                let tw = termrock::width(&text);
                let _ = ui.paint_str(Rect::new(area.x, area.y, tw, 1), &text, text_style);
                return;
            }
            let filled = ((track_w as f64) * ratio).round() as u16;
            let fill_st = Style::new().fg(fill_color).bg(bg);
            let subtle_st = Style::new().fg(colors.border_subtle).bg(bg);
            for i in 0..track_w {
                let (sym, st) = if i < filled {
                    ("━", fill_st)
                } else {
                    ("─", subtle_st)
                };
                let _ = ui.paint_str(Rect::new(area.x + i, area.y, 1, 1), sym, st);
            }
            let x = area.x + track_w + 1;
            let _ = ui.paint_str(Rect::new(x, area.y, vw, 1), &value_str, text_style);
            let _ = ui.paint_str(Rect::new(x + vw, area.y, 2, 1), suffix, suffix_style);
        }
        MeterVisual::Block => {
            let bar_w = area.width.saturating_sub(2);
            if bar_w < 4 {
                let text = termrock::truncate(&value_str, area.width);
                let tw = termrock::width(&text);
                let _ = ui.paint_str(Rect::new(area.x, area.y, tw, 1), &text, text_style);
                return;
            }
            let filled = ((bar_w as f64) * ratio).round() as u16;
            let rest_bg = colors.lift(bg);
            let on_fill = if tone == MeterTone::Stale {
                colors.text_secondary
            } else {
                colors.canvas
            };
            let on_rest = if matches!(tone, MeterTone::Stale | MeterTone::Refreshing) {
                colors.text_muted
            } else {
                colors.text_primary
            };
            let text = termrock::truncate(&value_str, bar_w.saturating_sub(2));
            let chars: Vec<char> = text.chars().collect();
            for i in 0..bar_w {
                let in_fill = i < filled;
                let cell_bg = if in_fill { fill_color } else { rest_bg };
                let fg = if in_fill { on_fill } else { on_rest };
                let mut st = Style::new().fg(fg).bg(cell_bg);
                if in_fill {
                    st = st.add_modifier(Modifier::BOLD);
                }
                let sym = if i >= 1 && (i as usize - 1) < chars.len() {
                    chars[i as usize - 1]
                } else {
                    ' '
                };
                let mut buf = [0u8; 4];
                let _ = ui.paint_str(
                    Rect::new(area.x + i, area.y, 1, 1),
                    sym.encode_utf8(&mut buf),
                    st,
                );
            }
            let _ = ui.paint_str(
                Rect::new(area.x + bar_w, area.y, 2, 1),
                suffix,
                suffix_style,
            );
        }
    }
}

/// Live progress owns only values and animation state; controls remain public
/// facade components so focus and activation are still runtime-owned.
#[derive(Debug)]
pub struct ProgressPage {
    frame: usize,
    build: f64,
    paused: bool,
    next_tick: Option<Moment>,
}

impl ProgressPage {
    pub fn new() -> Self {
        Self {
            frame: 0,
            build: 0.0,
            paused: false,
            next_tick: None,
        }
    }
}

impl Default for ProgressPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for ProgressPage {
    fn seek_paused(&mut self, frame: usize) {
        self.frame = frame;
        self.build = ((frame as f64) * 0.006).min(1.0);
        self.next_tick = None;
    }

    fn title(&self) -> &'static str {
        "Progress"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        let restart = restart_button().update(cx);
        if restart.activated() {
            self.build = 0.0;
        }
        response |= restart.erase();
        let pause = pause_button(self.paused).update(cx);
        if pause.activated() {
            self.paused = !self.paused;
        }
        response |= pause.erase();

        let now = cx.now();
        let interval = Duration::from_millis(80);
        let deadline = *self
            .next_tick
            .get_or_insert_with(|| now.saturating_add(interval));
        if cx.update_cause() == termrock::UpdateCause::Tick && now >= deadline {
            if !self.paused && self.build < 1.0 {
                self.build = (self.build + 0.006).min(1.0);
                if self.build >= 1.0 {
                    status = Some(PageStatus("Build finished ✓".to_owned()));
                }
            }
            self.frame = self.frame.wrapping_add(1);
            self.next_tick = Some(now.saturating_add(interval));
            response = response.repaint();
        }
        if cx.top_layer() == termrock::LayerId::PAGE
            && let Some(deadline) = self.next_tick
        {
            cx.request_repaint_at(deadline);
        }
        let _ = live_panel();
        let _ = states_panel();
        let _ = restart_button();
        let _ = pause_button(self.paused);
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Determinate, indeterminate, compact activity, terminal states",
            |ui, body| {
                let rows = fixed_rows(body, &[12, 1, 0]);
                let live = rows[0];
                live_panel().draw(ui, live, |ui, inner| {
                    let colors = ThemeColors::from_ui(ui);
                    let bg = ui.theme().bg(ui.surface());
                    let w = inner.width.min(70);
                    let status = if self.build >= 1.0 {
                        ProgressStatus::Done
                    } else if self.paused {
                        ProgressStatus::Paused
                    } else {
                        ProgressStatus::Active
                    };
                    render_bar(
                        ui,
                        Rect::new(inner.x, inner.y, w, 1),
                        "Building  ",
                        self.build,
                        status,
                        &colors,
                        bg,
                    );
                    render_indeterminate(
                        ui,
                        Rect::new(inner.x, inner.y + 2, w, 1),
                        "Resolving ",
                        self.frame as u64,
                        &colors,
                        bg,
                    );
                    render_spinner(
                        ui,
                        Rect::new(inner.x, inner.y + 4, w, 1),
                        "Waiting for the test runner",
                        self.frame as u64,
                        &colors,
                        bg,
                    );
                    let spin = spinner_frame(self.frame as u64);
                    let sec_st = Style::new().fg(colors.text_secondary).bg(bg);
                    let acc_st = Style::new().fg(colors.accent).bg(bg);
                    let msg = format!("{spin} 3 of 12 files");
                    let _ = ui.paint_str(
                        Rect::new(inner.x, inner.y + 5, termrock::width(&msg), 1),
                        &msg,
                        sec_st,
                    );
                    let _ = ui.paint_str(Rect::new(inner.x, inner.y + 5, 1, 1), spin, acc_st);

                    let restart = restart_button();
                    let pause = pause_button(self.paused);
                    let widths = [
                        restart
                            .measure(ui, Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                        pause
                            .measure(ui, Constraints::loose(inner.width, 1))
                            .preferred
                            .0,
                    ];
                    let rects =
                        row_layout(Rect::new(inner.x, inner.y + 7, inner.width, 1), &widths, 2);
                    if let Some(r) = rects.first().copied() {
                        restart.draw(ui, r);
                    }
                    if let Some(r) = rects.get(1).copied() {
                        pause.draw(ui, r);
                    }
                });

                let states_area =
                    Rect::new(rows[2].x, rows[2].y, rows[2].width, rows[2].height.min(22));
                let card_surface = ui.theme().raise(ui.surface());
                let mut overflow_caption = None;
                states_panel().draw(ui, states_area, |ui, inner| {
                    let colors = ThemeColors::from_ui(ui);
                    let bg = ui.theme().bg(ui.surface());
                    let w = inner.width.min(70);
                    let samples = [
                        ("Queued    ", 0.0, ProgressStatus::Active),
                        ("Halfway   ", 0.5, ProgressStatus::Active),
                        ("Completed ", 1.0, ProgressStatus::Done),
                        ("Failed    ", 0.64, ProgressStatus::Error),
                        ("Paused    ", 0.3, ProgressStatus::Paused),
                    ];
                    for (i, (label, r, s)) in samples.iter().enumerate() {
                        let y = inner.y + i as u16;
                        if y < inner.bottom() {
                            render_bar(
                                ui,
                                Rect::new(inner.x, y, w, 1),
                                label,
                                *r,
                                *s,
                                &colors,
                                bg,
                            );
                        }
                    }
                    if inner.height > 6 {
                        let muted_st = Style::new().fg(colors.text_muted).bg(bg);
                        let caption = "Narrow bars keep the percentage and drop the label:";
                        let _ = ui.paint_str(
                            Rect::new(inner.x, inner.y + 6, termrock::width(caption), 1),
                            caption,
                            muted_st,
                        );
                        render_bar(
                            ui,
                            Rect::new(inner.x, inner.y + 7, 14, 1),
                            "",
                            0.42,
                            ProgressStatus::Active,
                            &colors,
                            bg,
                        );
                    }
                    if inner.height > 13 {
                        let muted_st = Style::new().fg(colors.text_muted).bg(bg);
                        let primary_st = Style::new().fg(colors.text_primary).bg(bg);
                        let caption = "Capacity meters are never green: low ≤ 59 % white, medium ≤ 84 % warning, high error. Line and block visuals.";
                        overflow_caption = Some((inner.x, inner.y + 9, caption, muted_st));
                        let meters: [(&str, Option<u8>, MeterTone); 9] = [
                            ("Low       ", Some(38), MeterTone::Normal),
                            ("Medium    ", Some(72), MeterTone::Normal),
                            ("High      ", Some(91), MeterTone::Normal),
                            ("Warning   ", Some(82), MeterTone::Warning),
                            ("Exhausted ", Some(100), MeterTone::Exhausted),
                            ("Stale     ", Some(54), MeterTone::Stale),
                            ("Refreshing", Some(54), MeterTone::Refreshing),
                            ("Error     ", None, MeterTone::Error),
                            ("Unknown   ", None, MeterTone::Unknown),
                        ];
                        let col_w = inner.width.saturating_sub(12) / 2;
                        for (i, (label, pct, tone)) in meters.iter().enumerate() {
                            let y = inner.y + 10 + i as u16;
                            if y >= inner.bottom() {
                                break;
                            }
                            let _ = ui.paint_str(
                                Rect::new(inner.x, y, termrock::width(label), 1),
                                label,
                                primary_st,
                            );
                            let value = match tone {
                                MeterTone::Error => "quota read failed".to_owned(),
                                MeterTone::Unknown => String::new(),
                                _ => format!("{}% used", pct.unwrap_or(0)),
                            };
                            for (k, visual) in
                                [MeterVisual::Line, MeterVisual::Block].into_iter().enumerate()
                            {
                                let x = inner.x + 11 + k as u16 * (col_w + 1);
                                render_meter(
                                    ui,
                                    Rect::new(x, y, col_w.min(34), 1),
                                    *pct,
                                    &value,
                                    *tone,
                                    visual,
                                    self.frame as u64,
                                    &colors,
                                    bg,
                                );
                            }
                        }
                    }
                });
                if let Some((x, y, caption, muted_st)) = overflow_caption {
                    ui.with_surface(card_surface, |ui| {
                        let avail = states_area.right().saturating_sub(x);
                        let _ = ui.paint_str(
                            Rect::new(x, y, termrock::width(caption).min(avail), 1),
                            caption,
                            muted_st,
                        );
                    });
                }
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter", "Activate")]
    }
}

#[cfg(test)]
mod motion_tests {
    use super::*;

    #[test]
    fn paused_seek_is_deterministic_and_caps_completed_progress() {
        let mut page = ProgressPage::new();
        page.seek_paused(80);
        assert_eq!(page.frame, 80);
        assert!((page.build - 0.48).abs() < f64::EPSILON);
        page.seek_paused(usize::MAX);
        assert!((page.build - 1.0).abs() < f64::EPSILON);
    }
}
