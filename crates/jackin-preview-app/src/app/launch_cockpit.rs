//! Paused launch cockpit for sizes other than the 120×40 presentation blit.
//! Geometry follows the baseline `CockpitScreen::render` plus the shell strip.

#![allow(clippy::indexing_slicing)]

use termrock::author::{PaintStyle, put_cell_at};
use termrock::{Buffer, Color, Modifier, Position, Rect, Style, Ui};

use super::App;
use crate::sim::launch::{LaunchRun, Stage};

const EDGE: u16 = 2;
const BRAND: &str = " jackin❯ ";
const ROLE: &str = "the-architect";
const WORKSPACE: &str = "payments-platform";
const AGENT_LINE: &str =
    "Claude Code · Anthropic / Claude · account Claude · Work (session choice)";
/// First 103 columns of the baseline credential line, plus a tail so `cut` ellipsizes.
const CREDENTIALS: &str = "credentials  5 accounts · Claude · Work (1Password) · Claude · Personal · Codex · Primary · Grok · Team · Codex · Experiments";
const CREDENTIAL_VALUE: &str = "quota readable · session choice";
const SPINNER: &str = "⠋";

/// Left column of the stage rail. The 120×40 info frame keeps this column
/// undimmed.
pub(super) fn stage_rail_left(screen: Rect) -> Option<Rect> {
    let body = body_rect(screen);
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    if rail_h < 4 || rail_w == 0 {
        return None;
    }
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let y = body.y.saturating_add(6);
    Some(Rect::new(rail_x, y, 1, rail_h))
}

/// Footer for the 120×40 info frame. The presentation blit still shows the
/// closed-cockpit hints, and the dim leaves the last row alone.
pub(super) fn paint_open_footer(ui: &mut Ui<'_>, cancel: bool) {
    let screen = ui.full();
    if screen.height == 0 {
        return;
    }
    let mut buf = Buffer::empty(screen);
    let y = screen.bottom().saturating_sub(1);
    for x in 0..screen.width {
        put(&mut buf, x, y, ' ', ink(0x00ff_ffff, 0, false));
    }
    let hints: &[(&str, &str)] = if cancel {
        &[
            ("←→", "Choose"),
            ("Enter", "Keep launching"),
            ("Esc", "Keep launching"),
        ]
    } else {
        &[("↑↓", "Move"), ("y", "Copy"), ("Esc", "Close")]
    };
    paint_footer(&mut buf, screen, hints);
    emphasize_footer(&mut buf, screen, cancel);
    blit(ui, &buf, Rect::new(screen.x, y, screen.width, 1));
}

pub(super) fn paint_failure(app: &App, ui: &mut Ui<'_>) {
    let screen = ui.full();
    let Some(launch) = app.launch() else {
        return;
    };
    let mut buf = Buffer::empty(screen);
    fill(&mut buf, screen, ink(0x00ff_ffff, 0, false));
    let body = body_rect(screen);
    paint_rain_at(&mut buf, body, 231, false, true);
    mute_white(&mut buf, screen);
    paint_strip(app, &mut buf, screen, true);
    paint_failure_hero(&mut buf, body, launch);
    paint_failure_dialog(&mut buf, launch);
    paint_failure_status(&mut buf, screen, launch);
    let footer_y = screen.bottom().saturating_sub(1);
    for x in 0..screen.width {
        put(&mut buf, x, footer_y, ' ', ink(0x00ff_ffff, 0, false));
    }
    let hints: &[(&str, &str)] = &[("↑↓", "Move"), ("y", "Copy"), ("Esc", "Close")];
    paint_footer(&mut buf, screen, hints);
    emphasize_failure_footer(&mut buf, screen);
    blit(ui, &buf, screen);
}

fn paint_rain_at(buf: &mut Buffer, body: Rect, tick: u64, running: bool, failed: bool) {
    if body.is_empty() {
        return;
    }
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let ident_y = body.y.saturating_add(1);
    let exclude = [
        Rect::new(
            rail_x.saturating_sub(2),
            ident_y,
            rail_w.saturating_add(4),
            rail_h.saturating_add(6),
        ),
        Rect::new(body.x, body.bottom().saturating_sub(3), body.width, 3),
    ];
    let theme = termrock::Theme::junie();
    crate::rain::paint_atmosphere(buf, body, &exclude, tick, running, failed, &theme);
}

fn mute_white(buf: &mut Buffer, screen: Rect) {
    let white = Color::Rgb(0xff, 0xff, 0xff);
    let black = Color::Rgb(0, 0, 0);
    let muted = ink(0x008a_8a8a, 0, false);
    for y in screen.y..screen.bottom() {
        for x in screen.x..screen.right() {
            let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
                continue;
            };
            if cell.fg == white && cell.bg == black {
                cell.set_style(muted);
            }
        }
    }
}

fn paint_failure_hero(buf: &mut Buffer, body: Rect, launch: &LaunchRun) {
    if body.height < 6 {
        return;
    }
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let ident_y = body.y.saturating_add(1);
    let faint = ink(0x0073_7373, 0, false);
    let brand = ink(0x0073_7373, 0x0027_272a, false);
    center(buf, body, ident_y, BRAND, brand);
    center(
        buf,
        body,
        ident_y.saturating_add(1),
        &format!("Loading {ROLE} into workspace {WORKSPACE}"),
        ink(0x008a_8a8a, 0, false),
    );
    center(buf, body, ident_y.saturating_add(2), AGENT_LINE, faint);
    let (done, skipped) = launch.counts();
    let frontier = launch.states.iter().position(|state| !state.terminal());
    let stage_line = match frontier {
        Some(index) => format!(
            "stage {} of {} · {} · {done} done · {skipped} skipped",
            index.saturating_add(1),
            Stage::ALL.len(),
            Stage::ALL[index].label()
        ),
        None if launch.done => "all stages complete".to_owned(),
        None => "stopped".to_owned(),
    };
    center(buf, body, ident_y.saturating_add(3), &stage_line, faint);
    let rail = Rect::new(rail_x, ident_y.saturating_add(5), rail_w, rail_h);
    paint_rail(buf, rail, launch, true);
    let cred_y = rail.bottom().saturating_add(1);
    let x = rail.x.saturating_sub(4);
    let width = body.right().saturating_sub(x.saturating_add(2));
    // Resolved line: cockpit rewrites the value once Credentials finishes.
    let resolved = format!("resolved in memory · discarded after injection · {CREDENTIAL_VALUE}");
    put_str(
        buf,
        x,
        cred_y,
        &truncate(CREDENTIALS, usize::from(width)),
        faint,
    );
    put_str(
        buf,
        x.saturating_add(13),
        cred_y.saturating_add(1),
        &truncate(&resolved, usize::from(width.saturating_sub(13))),
        faint,
    );
}

fn paint_failure_dialog(buf: &mut Buffer, launch: &LaunchRun) {
    let card = Color::Rgb(0x18, 0x18, 0x1b);
    let left = 25u16;
    let right = 95u16;
    let top = 9u16;
    let bottom = 31u16;
    for y in top.saturating_add(1)..bottom.saturating_sub(1) {
        for x in left.saturating_add(1)..right.saturating_sub(1) {
            let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
                continue;
            };
            let fg = cell.fg;
            cell.set_symbol(" ");
            cell.set_style(Style::new().fg(fg).bg(card));
        }
    }
    let border = ink(0x004d_4d4d, 0x0018_181b, false);
    let muted = ink(0x008a_8a8a, 0x0018_181b, false);
    let white = ink(0x00ff_ffff, 0x0018_181b, false);
    let bold_white = ink(0x00ff_ffff, 0x0018_181b, true);
    let faint = ink(0x0073_7373, 0x0018_181b, false);
    let secondary = ink(0x00b3_b3b3, 0x0018_181b, false);
    let danger = ink(0x00f0_5d5d, 0x0018_181b, true);
    let rule = "─".repeat(10);
    put_str(buf, left, top, "╭─", border);
    put_str(buf, left.saturating_add(2), top, " ", bold_white);
    put_str(buf, left.saturating_add(3), top, "!", danger);
    put_str(
        buf,
        left.saturating_add(4),
        top,
        " Launch failed ",
        bold_white,
    );
    put_str(buf, left.saturating_add(19), top, &rule, border);
    put_str(
        buf,
        left.saturating_add(29),
        top,
        " run id is the only copyable value ",
        faint,
    );
    put_str(buf, left.saturating_add(64), top, "─ ", border);
    put_str(buf, left.saturating_add(66), top, "×", secondary);
    put_str(buf, left.saturating_add(67), top, " ─╮", border);
    for y in top.saturating_add(1)..bottom.saturating_sub(1) {
        put_str(buf, left, y, "│", border);
        put_str(buf, right.saturating_sub(1), y, "│", border);
    }
    put_str(
        buf,
        left,
        bottom.saturating_sub(1),
        &format!("╰{}╯", "─".repeat(68)),
        border,
    );
    let Some(failure) = launch.failure.as_ref() else {
        return;
    };
    put_str(
        buf,
        left.saturating_add(3),
        top.saturating_add(1),
        &failure.summary,
        white,
    );
    put_str(
        buf,
        left.saturating_add(3),
        top.saturating_add(2),
        &format!("Loading {ROLE} into workspace {WORKSPACE}"),
        muted,
    );
    paint_failure_field(
        buf,
        top.saturating_add(4),
        "Stage",
        failure.stage.label(),
        true,
    );
    let run = launch.trace_label.clone();
    paint_failure_field(buf, top.saturating_add(5), "Run id", &run, false);
    let step = truncate(&failure.next_step, 51);
    paint_failure_field(buf, top.saturating_add(6), "Next step", &step, false);
    let suffix = launch.trace_label.rsplit('-').next().unwrap_or("b5df");
    let container = format!("{}-{suffix}", launch.container);
    paint_failure_field(buf, top.saturating_add(7), "Container", &container, false);
    let mut lines = Vec::new();
    for line in &failure.detail {
        lines.extend(wrap_words(line, 59));
    }
    let view = 10usize;
    let thumb = if lines.len() > view {
        (view.saturating_mul(view) / lines.len()).max(1)
    } else {
        view
    };
    for (index, line) in lines.iter().take(view).enumerate() {
        let y = top.saturating_add(9).saturating_add(index as u16);
        put_str(buf, left.saturating_add(3), y, line, secondary);
        let mark = if index < thumb { '┃' } else { '┆' };
        let mark_ink = if index < thumb {
            ink(0x0080_8080, 0x0018_181b, false)
        } else {
            ink(0x0047_4747, 0x0018_181b, false)
        };
        put(buf, left.saturating_add(68), y, mark, mark_ink);
        if index + 1 == view {
            fade_log_row(buf, left, y);
        }
    }
    put(
        buf,
        left.saturating_add(60),
        top.saturating_add(20),
        ' ',
        ink(0x0027_272a, 0x0027_272a, false),
    );
    put_str(
        buf,
        left.saturating_add(61),
        top.saturating_add(20),
        "Close ",
        ink(0x00ff_ffff, 0x0027_272a, false),
    );
}

fn paint_failure_field(buf: &mut Buffer, y: u16, label: &str, value: &str, selected: bool) {
    let muted = ink(0x008a_8a8a, 0x0018_181b, false);
    let white = ink(0x00ff_ffff, 0x0018_181b, false);
    let bold_white = ink(0x00ff_ffff, 0x0018_181b, true);
    let accent = ink(0x0048_e054, 0x0018_181b, true);
    if selected {
        put_str(buf, 28, y, &" ".repeat(64), bold_white);
        put_str(buf, 27, y, "▎", accent);
        put_str(buf, 29, y, label, muted);
        put_str(buf, 40, y, value, ink(0x00f0_5d5d, 0x0018_181b, true));
        return;
    }
    put_str(buf, 27, y, " ", ink(0x0018_181b, 0x0018_181b, false));
    put_str(buf, 28, y, " ", white);
    put_str(buf, 29, y, label, muted);
    let label_end = 29u16.saturating_add(u16::try_from(label.chars().count()).unwrap_or(0));
    let pad = usize::from(92u16.saturating_sub(label_end));
    put_str(buf, label_end, y, &" ".repeat(pad), white);
    put_str(buf, 40, y, value, white);
}

fn fade_log_row(buf: &mut Buffer, left: u16, y: u16) {
    let card = Color::Rgb(0x18, 0x18, 0x1b);
    for x in left..left.saturating_add(68) {
        let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
            continue;
        };
        let Color::Rgb(red, green, blue) = cell.fg else {
            continue;
        };
        let mixed = Color::Rgb(
            mix_toward(red, 0x18, 0.55),
            mix_toward(green, 0x18, 0.55),
            mix_toward(blue, 0x1b, 0.55),
        );
        cell.set_style(Style::new().fg(mixed).bg(card));
    }
}

fn mix_toward(channel: u8, bg: u8, keep: f32) -> u8 {
    (f32::from(bg) + (f32::from(channel) - f32::from(bg)) * keep)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn wrap_words(text: &str, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let next = if line.is_empty() {
            word.chars().count()
        } else {
            line.chars().count() + 1 + word.chars().count()
        };
        if !line.is_empty() && next > limit {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn paint_failure_status(buf: &mut Buffer, screen: Rect, launch: &LaunchRun) {
    let muted = ink(0x008a_8a8a, 0, false);
    let faint = ink(0x0073_7373, 0, false);
    let stage = launch
        .failure
        .as_ref()
        .map(|failure| failure.stage.label())
        .unwrap_or("Network");
    put_str(buf, 0, 36, &format!(" ! {stage} failed"), muted);
    let suffix = launch.trace_label.rsplit('-').next().unwrap_or("b5df");
    let chip = format!(" {}-{suffix} ", launch.container);
    let chip_x = screen.width.saturating_sub(
        u16::try_from(chip.chars().count())
            .unwrap_or(0)
            .saturating_add(1),
    );
    put_str(buf, chip_x, 36, &chip, faint);
    put_str(
        buf,
        1,
        37,
        &format!(
            "{} lines · b build log",
            crate::sim::launch::BUILD_LOG.len()
        ),
        faint,
    );
}

fn emphasize_failure_footer(buf: &mut Buffer, screen: Rect) {
    let y = screen.bottom().saturating_sub(1);
    let Some(cell) = buf.cell_mut(Position::new(14, y)) else {
        return;
    };
    cell.set_style(cell.style().add_modifier(Modifier::BOLD));
    for x in [32u16, 33, 34, 35, 36, 37, 45, 46, 47, 48, 49, 50] {
        let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
            continue;
        };
        cell.set_style(cell.style().add_modifier(Modifier::BOLD));
    }
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>) {
    let screen = ui.full();
    let Some(launch) = app.launch() else {
        return;
    };
    let mut buf = Buffer::empty(screen);
    fill(&mut buf, screen, ink(0x00ff_ffff, 0, false));

    let body = body_rect(screen);
    paint_rain(&mut buf, body);
    paint_strip(app, &mut buf, screen, false);
    paint_body(&mut buf, body, launch);
    let hints: &[(&str, &str)] = if app.cockpit_info_open {
        &[("↑↓", "Move"), ("y", "Copy"), ("Esc", "Close")]
    } else if app.cancel_open {
        &[
            ("←→", "Choose"),
            ("Enter", "Keep launching"),
            ("Esc", "Keep launching"),
        ]
    } else {
        &[
            ("i", "Container info"),
            ("x", "Cancel launch…"),
            ("Ctrl+Q", "Quit"),
            ("Ctrl+C", "Abort now"),
        ]
    };
    paint_footer(&mut buf, screen, hints);
    if app.cockpit_info_open || app.cancel_open {
        // The baseline hint row keeps bold on two fixed columns. The glyphs
        // there are whatever the hints already painted.
        emphasize_footer(&mut buf, screen, app.cancel_open);
    }
    blit(ui, &buf, screen);
}

fn emphasize_footer(buf: &mut Buffer, screen: Rect, cancel: bool) {
    let y = screen.bottom().saturating_sub(1);
    let mut columns = vec![37, 38, 39, 40, 41, 42, 50, 51, 52, 53, 54, 55];
    if cancel {
        columns.push(19);
    }
    for x in columns {
        if x >= screen.width {
            continue;
        }
        let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
            continue;
        };
        cell.set_style(cell.style().add_modifier(Modifier::BOLD));
    }
}

fn body_rect(screen: Rect) -> Rect {
    let top = 2u16.min(screen.height);
    let bottom = 2u16.min(screen.height.saturating_sub(top));
    Rect::new(
        screen.x,
        screen.y.saturating_add(top),
        screen.width,
        screen.height.saturating_sub(top.saturating_add(bottom)),
    )
}

fn paint_rain(buf: &mut Buffer, body: Rect) {
    if body.is_empty() {
        return;
    }
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let ident_y = body.y.saturating_add(1);
    let exclude = [
        Rect::new(
            rail_x.saturating_sub(2),
            ident_y,
            rail_w.saturating_add(4),
            rail_h.saturating_add(6),
        ),
        Rect::new(body.x, body.bottom().saturating_sub(3), body.width, 3),
    ];
    // Rain samples the truecolor junie ladder. The blit downgrades it, so a
    // 256 or mono theme does not bake the nearest index in first.
    let theme = termrock::Theme::junie();
    crate::rain::paint_atmosphere(buf, body, &exclude, 40, true, false, &theme);
}

fn paint_strip(app: &App, buf: &mut Buffer, screen: Rect, failed: bool) {
    if screen.width < 12 || screen.height == 0 {
        return;
    }
    let brand = if failed {
        ink(0x0073_7373, 0x0027_272a, false)
    } else {
        ink(0x0019_191c, 0x0048_e054, true)
    };
    put_str(buf, 1, 0, BRAND, brand);
    let area = Rect::new(10, 0, screen.width.saturating_sub(10), 1);
    let running = app.world.running_count();
    let left = [
        Seg {
            text: "entering the Construct".to_owned(),
            lead: None,
            priority: 9,
            clickable: false,
            tone: if failed { Tone::Muted } else { Tone::Normal },
        },
        Seg {
            text: format!("{running} running"),
            lead: Some("●"),
            priority: 8,
            clickable: false,
            tone: if failed { Tone::Faint } else { Tone::Secondary },
        },
        Seg {
            text: format!("Launch › {WORKSPACE} › {ROLE}"),
            lead: None,
            priority: 7,
            clickable: false,
            tone: if failed { Tone::Faint } else { Tone::Secondary },
        },
    ];
    let (done, _) = app.launch().map(LaunchRun::counts).unwrap_or((0, 0));
    let right = [
        Seg {
            text: format!("{done}/{} stages", Stage::ALL.len()),
            lead: None,
            priority: 5,
            clickable: false,
            tone: if failed { Tone::Faint } else { Tone::Muted },
        },
        Seg {
            text: "? help".to_owned(),
            lead: None,
            priority: 4,
            clickable: true,
            tone: if failed { Tone::Faint } else { Tone::Muted },
        },
    ];
    paint_segments(buf, area, &left, &right);
    if std::env::var_os("NO_COLOR").is_some() {
        let brand = Style::new()
            .fg(Color::Rgb(0, 0, 0))
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED);
        put_str(buf, 1, 0, BRAND, brand);
    }
}

fn paint_body(buf: &mut Buffer, body: Rect, launch: &LaunchRun) {
    if body.height < 6 {
        return;
    }
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let ident_y = body.y.saturating_add(1);
    let white = ink(0x00ff_ffff, 0, false);
    let bold_white = ink(0x00ff_ffff, 0, true);
    let secondary = ink(0x00b3_b3b3, 0, false);
    let muted = ink(0x008a_8a8a, 0, false);

    center(
        buf,
        body,
        ident_y,
        BRAND,
        ink(0x0019_191c, 0x0048_e054, true),
    );
    center(
        buf,
        body,
        ident_y.saturating_add(1),
        &format!("Loading {ROLE} into workspace {WORKSPACE}"),
        bold_white,
    );
    center(buf, body, ident_y.saturating_add(2), AGENT_LINE, secondary);
    let (done, skipped) = launch.counts();
    let frontier = launch.states.iter().position(|state| !state.terminal());
    let stage_line = match frontier {
        Some(index) => format!(
            "stage {} of {} · {} · {done} done · {skipped} skipped",
            index.saturating_add(1),
            Stage::ALL.len(),
            Stage::ALL[index].label()
        ),
        None if launch.done => "all stages complete".to_owned(),
        None => "stopped".to_owned(),
    };
    center(buf, body, ident_y.saturating_add(3), &stage_line, muted);

    let rail = Rect::new(rail_x, ident_y.saturating_add(5), rail_w, rail_h);
    paint_rail(buf, rail, launch, false);

    let mut cred_y = rail.bottom().saturating_add(1);
    if cred_y.saturating_add(1) < body.bottom().saturating_sub(3) {
        clear_rows(buf, body.x, cred_y, body.width, 2, white);
        let x = rail.x.saturating_sub(4);
        let width = body.right().saturating_sub(x.saturating_add(2));
        put_str(
            buf,
            x,
            cred_y,
            &truncate(CREDENTIALS, usize::from(width)),
            muted,
        );
        put_str(
            buf,
            x.saturating_add(13),
            cred_y.saturating_add(1),
            &truncate(CREDENTIAL_VALUE, usize::from(width.saturating_sub(13))),
            secondary,
        );
        cred_y = cred_y.saturating_add(3);
    }
    let _ = cred_y;

    let ay = body.bottom().saturating_sub(2);
    clear_rows(buf, body.x, ay.saturating_sub(1), body.width, 3, white);
    put_str(
        buf,
        body.x.saturating_add(1),
        ay,
        &format!("{SPINNER} Resolving credentials…"),
        secondary,
    );
    paint_cancel(buf, body, ay.saturating_add(1));
}

fn paint_rail(buf: &mut Buffer, rail: Rect, launch: &LaunchRun, failed: bool) {
    if rail.is_empty() {
        return;
    }
    let n = Stage::ALL.len();
    let has_bar = n > usize::from(rail.height);
    let row_w = rail.width.saturating_sub(u16::from(has_bar));
    let view = usize::from(rail.height).min(n);
    for k in 0..view {
        let y = rail.y.saturating_add(u16::try_from(k).unwrap_or(0));
        let state = launch.states[k];
        let stage = Stage::ALL[k];
        put(buf, rail.x, y, ' ', ink(0, 0, false));
        let glyph_style = match state {
            termrock::StepState::Done if failed => ink(0x0073_7373, 0, false),
            termrock::StepState::Done => ink(0x003f_b950, 0, false),
            termrock::StepState::Running => ink(0x0048_e054, 0, false),
            _ if failed => ink(0x008a_8a8a, 0, false),
            _ => ink(0x00ff_ffff, 0, false),
        };
        let glyph = match state {
            termrock::StepState::Done => "✓",
            termrock::StepState::Running => SPINNER,
            _ => " ",
        };
        put_str(buf, rail.x.saturating_add(1), y, glyph, glyph_style);
        let number_style = if state == termrock::StepState::Running {
            ink(0x00b3_b3b3, 0, false)
        } else {
            ink(0x0073_7373, 0, false)
        };
        put_str(
            buf,
            rail.x.saturating_add(3),
            y,
            &format!("{:02}", k.saturating_add(1)),
            number_style,
        );
        let label_style = match state {
            termrock::StepState::Queued if failed => ink(0x0073_7373, 0, false),
            termrock::StepState::Queued => ink(0x008a_8a8a, 0, false),
            termrock::StepState::Running => ink(0x00ff_ffff, 0, true),
            termrock::StepState::Done if failed => ink(0x0073_7373, 0, false),
            termrock::StepState::Done => ink(0x00b3_b3b3, 0, false),
            termrock::StepState::Skipped | termrock::StepState::Blocked => {
                ink(0x0073_7373, 0, false)
            }
            termrock::StepState::Failed => ink(0x00ff_ffff, 0, true),
        };
        let meta = shown_meta(state, launch.durations[k]);
        let row_right = rail.x.saturating_add(row_w);
        let label_x = rail.x.saturating_add(6);
        let avail = usize::from(row_right.saturating_sub(label_x.saturating_add(1)));
        let meta_w = meta.as_ref().map(|text| text.chars().count()).unwrap_or(0);
        let show_meta = meta_w > 0 && avail >= meta_w.saturating_add(12);
        let label_w = if show_meta {
            avail.saturating_sub(meta_w.saturating_add(2))
        } else {
            avail
        };
        put_str(
            buf,
            label_x,
            y,
            &truncate(stage.label(), label_w),
            label_style,
        );
        if show_meta && let Some(meta) = meta {
            let meta_x =
                row_right.saturating_sub(u16::try_from(meta_w).unwrap_or(0).saturating_add(1));
            put_str(buf, meta_x, y, &meta, ink(0x0073_7373, 0, false));
        }
    }
    if has_bar {
        let thumb = (view.saturating_mul(view) / n).max(1);
        let x = rail.right().saturating_sub(1);
        for k in 0..view {
            let y = rail.y.saturating_add(u16::try_from(k).unwrap_or(0));
            if k < thumb {
                put(buf, x, y, '┃', ink(0x0080_8080, 0, false));
            } else {
                put(buf, x, y, '┆', ink(0x0047_4747, 0, false));
            }
        }
    }
}

/// Bottom content row of an overflowing rail. Truecolor mixes toward black.
/// Indexed and mono frames keep the ink and set `dim`, as the baseline fade does
/// once the color is no longer raw RGB.
fn fade_band(screen: Rect) -> Option<(u16, u16, u16)> {
    let body = body_rect(screen);
    let rail_h = 11u16.min(body.height.saturating_sub(9));
    let rail_w = 44u16.min(body.width.saturating_sub(4));
    if rail_h < 4 || usize::from(rail_h) >= Stage::ALL.len() {
        return None;
    }
    let rail_x = body.x.saturating_add(body.width.saturating_sub(rail_w) / 2);
    let y = body
        .y
        .saturating_add(6)
        .saturating_add(rail_h)
        .saturating_sub(1);
    let row_w = rail_w.saturating_sub(1);
    Some((rail_x, rail_x.saturating_add(row_w), y))
}

fn mix_channel(channel: u8) -> u8 {
    (f32::from(channel) * 0.55).round().clamp(0.0, 255.0) as u8
}

fn shown_meta(state: termrock::StepState, duration: u64) -> Option<String> {
    match state {
        termrock::StepState::Done => Some(format!("{:.1} s", duration as f64 * 0.033 * 3.0)),
        termrock::StepState::Queued => Some("queued".to_owned()),
        termrock::StepState::Skipped => Some("skipped".to_owned()),
        termrock::StepState::Blocked => Some("blocked".to_owned()),
        _ => None,
    }
}

fn paint_cancel(buf: &mut Buffer, body: Rect, y: u16) {
    let label = "Cancel launch…";
    let label_w = u16::try_from(label.chars().count()).unwrap_or(0);
    let bw = label_w.saturating_add(2);
    let bx = body.right().saturating_sub(bw.saturating_add(1));
    if bx <= body.x.saturating_add(24) {
        return;
    }
    let gutter = ink(0x0027_272a, 0x0027_272a, false);
    let face = ink(0x00ff_ffff, 0x0027_272a, false);
    put(buf, bx, y, ' ', gutter);
    put_str(buf, bx.saturating_add(1), y, label, face);
    put(buf, bx.saturating_add(bw.saturating_sub(1)), y, ' ', face);
}

fn paint_footer(buf: &mut Buffer, screen: Rect, hints: &[(&str, &str)]) {
    if screen.height == 0 {
        return;
    }
    let y = screen.bottom().saturating_sub(1);
    let key = ink(0x00ff_ffff, 0, true);
    let action = ink(0x008a_8a8a, 0, false);
    let mut x = 1u16;
    for (chord, label) in hints {
        put_str(buf, x, y, chord, key);
        x = x.saturating_add(u16::try_from(chord.chars().count()).unwrap_or(0));
        x = x.saturating_add(1);
        put_str(buf, x, y, label, action);
        x = x.saturating_add(u16::try_from(label.chars().count()).unwrap_or(0));
        x = x.saturating_add(2);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Normal,
    Secondary,
    Muted,
    Faint,
}

struct Seg {
    text: String,
    lead: Option<&'static str>,
    priority: u8,
    clickable: bool,
    tone: Tone,
}

impl Seg {
    fn shown(&self) -> String {
        match self.lead {
            Some(glyph) => format!("{glyph} {}", self.text),
            None => self.text.clone(),
        }
    }

    fn width(&self) -> u16 {
        u16::try_from(self.shown().chars().count()).unwrap_or(0)
    }
}

fn paint_segments(buf: &mut Buffer, area: Rect, left: &[Seg], right: &[Seg]) {
    if area.is_empty() {
        return;
    }
    let mut keep_l = vec![true; left.len()];
    let mut keep_r = vec![true; right.len()];
    let total = |keep_l: &[bool], keep_r: &[bool]| -> u16 {
        let left_w: u16 = left
            .iter()
            .zip(keep_l)
            .filter(|(_, keep)| **keep)
            .map(|(seg, _)| seg.width().saturating_add(2))
            .sum();
        let right_w: u16 = right
            .iter()
            .zip(keep_r)
            .filter(|(_, keep)| **keep)
            .map(|(seg, _)| seg.width().saturating_add(2))
            .sum();
        let dropped = keep_l.iter().chain(keep_r).any(|keep| !keep);
        let more = if dropped { 3 } else { 0 };
        left_w
            .saturating_add(right_w)
            .saturating_add(more)
            .saturating_sub(2)
            .saturating_add(EDGE.saturating_mul(2))
    };
    while total(&keep_l, &keep_r) > area.width {
        let mut best: Option<(u8, bool, usize)> = None;
        for (index, seg) in left.iter().enumerate() {
            if keep_l[index] && best.is_none_or(|item| seg.priority < item.0) {
                best = Some((seg.priority, true, index));
            }
        }
        for (index, seg) in right.iter().enumerate() {
            if keep_r[index] && best.is_none_or(|item| seg.priority <= item.0) {
                best = Some((seg.priority, false, index));
            }
        }
        match best {
            Some((_, true, index)) => keep_l[index] = false,
            Some((_, false, index)) => keep_r[index] = false,
            None => break,
        }
    }
    let mut x = area.x.saturating_add(EDGE);
    for (seg, keep) in left.iter().zip(&keep_l) {
        if !keep {
            continue;
        }
        draw_seg(buf, x, area.y, seg);
        x = x.saturating_add(seg.width()).saturating_add(2);
    }
    let mut rx = area.right().saturating_sub(EDGE);
    for (seg, keep) in right.iter().zip(&keep_r).rev() {
        if !keep {
            continue;
        }
        rx = rx.saturating_sub(seg.width());
        let start = if seg.clickable {
            rx.saturating_sub(1)
        } else {
            rx
        };
        draw_seg(buf, start, area.y, seg);
        rx = rx.saturating_sub(2);
    }
    let hidden = left
        .iter()
        .zip(&keep_l)
        .chain(right.iter().zip(&keep_r))
        .any(|(seg, keep)| !keep && !seg.text.is_empty());
    if hidden && area.width > EDGE.saturating_mul(2) {
        let at = if right.is_empty() {
            x.min(area.right().saturating_sub(EDGE.saturating_add(1)))
        } else {
            rx.saturating_sub(1).max(area.x.saturating_add(EDGE))
        };
        put(buf, at, area.y, '…', ink(0x008a_8a8a, 0, false));
    }
}

fn draw_seg(buf: &mut Buffer, x: u16, y: u16, seg: &Seg) {
    let style = match seg.tone {
        Tone::Normal => ink(0x00ff_ffff, 0, false),
        Tone::Secondary => ink(0x00b3_b3b3, 0, false),
        Tone::Muted => ink(0x008a_8a8a, 0, false),
        Tone::Faint => ink(0x0073_7373, 0, false),
    };
    let text = if seg.clickable {
        format!(" {} ", seg.shown())
    } else {
        seg.shown()
    };
    put_str(buf, x, y, &text, style);
    if let Some(glyph) = seg.lead {
        let lead = if seg.tone == Tone::Faint {
            ink(0x0073_7373, 0, false)
        } else {
            ink(0x0048_e054, 0, false)
        };
        put_str(buf, x, y, glyph, lead);
    }
}

fn center(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let n = u16::try_from(text.chars().count()).unwrap_or(0);
    let x = area.x.saturating_add(area.width.saturating_sub(n) / 2);
    put_str(buf, x, y, &truncate(text, usize::from(area.width)), style);
}

fn clear_rows(buf: &mut Buffer, x: u16, y: u16, width: u16, rows: u16, style: Style) {
    for row in 0..rows {
        for col in 0..width {
            put(
                buf,
                x.saturating_add(col),
                y.saturating_add(row),
                ' ',
                style,
            );
        }
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn fill(buf: &mut Buffer, area: Rect, style: Style) {
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            put(buf, x, y, ' ', style);
        }
    }
}

fn put_str(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    let mut col = x;
    for ch in text.chars() {
        put(buf, col, y, ch, style);
        col = col.saturating_add(1);
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
        return;
    };
    cell.set_symbol(&ch.to_string());
    cell.set_style(style);
}

fn ink(fg: u32, bg: u32, bold: bool) -> Style {
    let mut style = Style::new()
        .fg(Color::from_u32(fg))
        .bg(Color::from_u32(bg))
        .remove_modifier(
            Modifier::BOLD
                | Modifier::DIM
                | Modifier::ITALIC
                | Modifier::UNDERLINED
                | Modifier::REVERSED
                | Modifier::CROSSED_OUT,
        );
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn blit(ui: &mut Ui<'_>, buf: &Buffer, area: Rect) {
    let level = ui.theme_ref().capability.color;
    let indexed = matches!(
        level,
        termrock::ColorLevel::Ansi256 | termrock::ColorLevel::Ansi16
    );
    let mono_frame = level == termrock::ColorLevel::Mono && std::env::var_os("NO_COLOR").is_none();
    let fade = fade_band(area);
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let Some(cell) = buf.cell(Position::new(x, y)) else {
                continue;
            };
            let mut style = cell.style();
            if mono_frame {
                if let (Some(fg), Some(bg)) = (style.fg, style.bg)
                    && let Some(mapped) = termrock::theme::legacy_mono(fg, bg, y == 0)
                {
                    style.fg = Some(mapped.fg);
                    style.bg = Some(mapped.bg);
                    if mapped.underline {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if mapped.reverse {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                }
            } else if indexed {
                if let (Some(fg), Some(bg)) = (style.fg, style.bg) {
                    let (fg, bg) = termrock::theme::downgrade_pair(fg, bg, level);
                    style.fg = Some(fg);
                    style.bg = Some(bg);
                } else if let Some(fg) = style.fg {
                    style.fg = Some(termrock::theme::downgrade_color(fg, level));
                } else if let Some(bg) = style.bg {
                    style.bg = Some(termrock::theme::downgrade_color(bg, level));
                }
            }
            if let Some((x0, x1, fade_y)) = fade
                && y == fade_y
                && x >= x0
                && x < x1
            {
                if level == termrock::ColorLevel::TrueColor {
                    if let Some(Color::Rgb(red, green, blue)) = style.fg {
                        style.fg = Some(Color::Rgb(
                            mix_channel(red),
                            mix_channel(green),
                            mix_channel(blue),
                        ));
                    }
                } else {
                    style = style.add_modifier(Modifier::DIM);
                }
            }
            let mut paint = PaintStyle::new();
            if let Some(fg) = style.fg {
                paint = paint.fg(fg);
            }
            if let Some(bg) = style.bg {
                paint = paint.bg(bg);
            }
            paint = paint.remove_modifier(
                Modifier::BOLD
                    | Modifier::DIM
                    | Modifier::ITALIC
                    | Modifier::UNDERLINED
                    | Modifier::REVERSED
                    | Modifier::CROSSED_OUT,
            );
            if style.add_modifier.contains(Modifier::BOLD) {
                paint = paint.add_modifier(Modifier::BOLD);
            }
            if style.add_modifier.contains(Modifier::UNDERLINED) {
                paint = paint.add_modifier(Modifier::UNDERLINED);
            }
            if style.add_modifier.contains(Modifier::REVERSED) {
                paint = paint.add_modifier(Modifier::REVERSED);
            }
            if style.add_modifier.contains(Modifier::DIM) {
                paint = paint.add_modifier(Modifier::DIM);
            }
            put_cell_at(ui, Position::new(x, y), cell.symbol(), paint);
        }
    }
}
