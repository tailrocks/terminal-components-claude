//! Wide workspace manager: brand, sidebar, and a detail card.
//!
//! Column positions are the baseline 120-column split. The app supplies the
//! pinned workspace, its mounts, and its instances. This painter does not
//! invent a second list model.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::id::Id;
use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;
use termrock_controls::Brand;

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const EXPANDED_BG: Color = Color::Rgb(0x30, 0x30, 0x36);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const BADGE: Color = Color::Rgb(0x73, 0x73, 0x73);
const DANGER_BUTTON: Color = Color::Rgb(0xff, 0x6b, 0x6b);
const INDEX_ERROR: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const STALE: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const LABEL_BUDGET: u16 = 28;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SplitMode {
    /// Current directory is not a saved workspace.
    Empty,
    /// Current directory is the pinned workspace. The tree is collapsed.
    Saved,
    /// The pinned workspace row is expanded.
    Expanded,
}

pub struct SplitMount<'a> {
    pub source: &'a str,
    pub mode: &'a str,
    pub isolation: &'a str,
}

pub struct SplitInstance<'a> {
    pub id: &'a str,
    pub role: &'a str,
    pub agent: &'a str,
    pub status: &'a str,
    pub live: bool,
    pub dirty: bool,
}

pub struct SplitWorkspace<'a> {
    pub name: &'a str,
    /// A collapsed row with at least one visible instance.
    pub expandable: bool,
}

/// Discovery or daemon failure drawn on the saved split.
#[derive(Clone, Copy)]
pub struct SplitAlert<'a> {
    pub index_error: Option<&'a str>,
    pub daemon_stale: bool,
}

pub struct ManagerSplit<'a> {
    pub brand_id: Id,
    pub mode: SplitMode,
    pub menus: &'a [&'a str],
    pub workspace_name: &'a str,
    pub workdir: &'a str,
    pub cwd_display: &'a str,
    pub mounts: &'a [SplitMount<'a>],
    pub role_line: &'a str,
    pub env_line: &'a str,
    pub account_lines: &'a [&'a str],
    pub policy_line: &'a str,
    pub workspaces: &'a [SplitWorkspace<'a>],
    pub instances: &'a [SplitInstance<'a>],
    pub running: u16,
    pub live_here: u16,
    pub daemon_ago: &'a str,
    pub alert: Option<SplitAlert<'a>>,
}

struct Ink {
    canvas: Color,
    card: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    accent: Color,
    border: Color,
    button: Color,
    select_bg: Color,
    expanded_bg: Color,
    rule: Color,
    badge: Color,
    danger_button: Color,
    index_error: Color,
    stale: Color,
}

pub fn paint_manager_split(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>) {
    if area.width < 80 || area.height < 20 {
        return;
    }
    let ink = ink_from(ui);
    ui.fill(area, plain(ink.primary, ink.canvas));
    let _ = Brand::new(frame.brand_id, "jackin❯")
        .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
    paint_menus(ui, area, frame.menus, &ink);
    paint_identity(ui, area, frame, &ink);
    ui.fill(
        Rect::new(
            40,
            area.y.saturating_add(2),
            area.width.saturating_sub(41),
            36,
        ),
        plain(ink.primary, ink.card),
    );
    paint_box(ui, area, frame, &ink);
    paint_rows(ui, area, frame, &ink);
    paint_detail(ui, area, frame, &ink);
    paint_buttons(ui, area, frame, &ink);
    paint_hints(ui, area, frame, &ink);
}

fn ink_from(ui: &Ui<'_>) -> Ink {
    let theme = ui.theme_ref();
    let level = theme.capability.color;
    let downgrade = |c: Color| crate::theme::downgrade_color(c, level);
    Ink {
        canvas: theme.bg(Surface::Canvas),
        card: theme.bg(Surface::Surface),
        primary: step(theme, FgStep::Primary),
        secondary: step(theme, FgStep::Secondary),
        muted: step(theme, FgStep::Muted),
        accent: theme.color.accent,
        border: theme.color.border_strong,
        button: theme.bg(Surface::Overlay),
        select_bg: downgrade(SELECT_BG),
        expanded_bg: downgrade(EXPANDED_BG),
        rule: downgrade(RULE),
        badge: downgrade(BADGE),
        danger_button: downgrade(DANGER_BUTTON),
        index_error: downgrade(INDEX_ERROR),
        stale: downgrade(STALE),
    }
}

fn step(theme: &crate::theme::Theme, which: FgStep) -> Color {
    theme
        .color
        .fg
        .get(which.index())
        .copied()
        .unwrap_or(Color::Reset)
}

fn plain(fg: Color, bg: Color) -> PaintStyle {
    PaintStyle::new()
        .fg(fg)
        .bg(bg)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn bold(fg: Color, bg: Color) -> PaintStyle {
    plain(fg, bg).add_modifier(Modifier::BOLD)
}

fn put(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let room = area.right().saturating_sub(x);
    if room == 0 || y >= area.bottom() || text.is_empty() {
        return;
    }
    ui.paint_str(Rect::new(x, y, room, 1), text, style);
}

fn paint_menus(ui: &mut Ui<'_>, area: Rect, menus: &[&str], ink: &Ink) {
    let style = plain(ink.secondary, ink.canvas);
    let mut x = 12u16;
    for label in menus {
        let text = format!(" {label} ");
        put(ui, area, x, area.y, &text, style);
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
}

fn paint_identity(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    if frame.mode == SplitMode::Saved
        && let Some(alert) = frame.alert
    {
        paint_degraded_identity(ui, area, alert, ink);
        return;
    }
    let y = area.y;
    let (crumb, crumb_x, inside_x, running_x, running) = match frame.mode {
        SplitMode::Empty => ("Workspaces", 72, 84, 106, "no instances".to_owned()),
        SplitMode::Saved => (
            "Workspaces",
            73,
            85,
            108,
            format!(" {} running", frame.running),
        ),
        SplitMode::Expanded => (
            "Workspaces",
            73,
            85,
            108,
            format!(" {} running", frame.running),
        ),
    };
    let crumb = if frame.mode == SplitMode::Expanded {
        format!("Workspaces › {}", frame.workspace_name)
    } else {
        crumb.to_owned()
    };
    let crumb_x = if frame.mode == SplitMode::Expanded {
        53
    } else {
        crumb_x
    };
    put(
        ui,
        area,
        crumb_x,
        y,
        &crumb,
        plain(ink.secondary, ink.canvas),
    );
    put(
        ui,
        area,
        inside_x,
        y,
        "inside the Construct",
        plain(ink.secondary, ink.canvas),
    );
    if frame.mode != SplitMode::Empty {
        put(ui, area, 107, y, "●", plain(ink.accent, ink.canvas));
    }
    let running_style = if frame.mode == SplitMode::Empty {
        plain(ink.muted, ink.canvas)
    } else {
        plain(ink.secondary, ink.canvas)
    };
    put(ui, area, running_x, y, &running, running_style);
}

fn paint_degraded_identity(ui: &mut Ui<'_>, area: Rect, alert: SplitAlert<'_>, ink: &Ink) {
    let y = area.y;
    let secondary = plain(ink.secondary, ink.canvas);
    let mut x = 41u16;
    let crumb = "Workspaces";
    put(ui, area, x, y, crumb, secondary);
    x = x.saturating_add(width(crumb)).saturating_add(2);
    let inside = "inside the Construct";
    put(ui, area, x, y, inside, secondary);
    x = x.saturating_add(width(inside)).saturating_add(2);
    if let Some(error) = alert.index_error.filter(|error| !error.is_empty()) {
        let text = format!("! {error}");
        put(ui, area, x, y, &text, plain(ink.index_error, ink.canvas));
        x = x.saturating_add(width(&text)).saturating_add(2);
    }
    if alert.daemon_stale {
        put(
            ui,
            area,
            x,
            y,
            "▲ daemon stale",
            plain(ink.stale, ink.canvas),
        );
    }
}

fn box_top(summary: &str) -> (String, u16) {
    let prefix = "╭─ Workspaces ";
    let suffix = "─╮";
    let width = 37usize;
    let dashes = width
        .saturating_sub(prefix.chars().count())
        .saturating_sub(summary.chars().count())
        .saturating_sub(suffix.chars().count());
    let line = format!("{prefix}{}{summary}{suffix}", "─".repeat(dashes));
    let badge_x = 1 + prefix.chars().count() as u16 + dashes as u16;
    (line, badge_x)
}

fn paint_box(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let summary = match frame.mode {
        SplitMode::Empty => " no instances ".to_owned(),
        SplitMode::Saved | SplitMode::Expanded => format!(" {} running ", frame.running),
    };
    let (line, badge_x) = box_top(&summary);
    let border = plain(ink.border, ink.canvas);
    put(ui, area, 1, 2, &line, border);
    put(
        ui,
        area,
        3,
        2,
        " Workspaces ",
        bold(ink.primary, ink.canvas),
    );
    put(ui, area, badge_x, 2, &summary, plain(ink.badge, ink.canvas));
    for y in 3..37 {
        put(ui, area, 1, y, "│", border);
        put(ui, area, 37, y, "│", border);
        put(ui, area, 39, y, "│", border);
    }
    put(
        ui,
        area,
        1,
        37,
        "╰───────────────────────────────────╯",
        border,
    );
    put(ui, area, 39, 2, "│", border);
    put(ui, area, 39, 37, "│", border);
}

fn paint_rows(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let mut y = 3u16;
    match frame.mode {
        SplitMode::Empty => {
            paint_current(ui, area, y, true, ink);
            y = y.saturating_add(1);
            put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(ui, area, 7, y, &action, plain(ink.secondary, ink.canvas));
        }
        SplitMode::Saved => {
            paint_current(ui, area, y, true, ink);
            y = y.saturating_add(1);
            for workspace in frame.workspaces {
                put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
                let mark = if workspace.expandable { "▸" } else { " " };
                put(ui, area, 5, y, mark, plain(ink.secondary, ink.canvas));
                let label = crate::text::truncate_middle(workspace.name, LABEL_BUDGET);
                put(ui, area, 7, y, &label, plain(ink.primary, ink.canvas));
                y = y.saturating_add(1);
            }
            put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(ui, area, 7, y, &action, plain(ink.secondary, ink.canvas));
        }
        SplitMode::Expanded => {
            paint_current(ui, area, y, false, ink);
            y = y.saturating_add(1);
            paint_expanded_workspace(ui, area, y, frame.workspace_name, ink);
            y = y.saturating_add(1);
            for instance in frame.instances {
                paint_child(ui, area, y, instance, ink);
                y = y.saturating_add(1);
            }
            for workspace in frame
                .workspaces
                .iter()
                .filter(|workspace| workspace.name != frame.workspace_name)
            {
                put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
                put(ui, area, 5, y, "▸", plain(ink.secondary, ink.canvas));
                put(
                    ui,
                    area,
                    7,
                    y,
                    workspace.name,
                    plain(ink.primary, ink.canvas),
                );
                y = y.saturating_add(1);
            }
            put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(ui, area, 7, y, &action, plain(ink.secondary, ink.canvas));
        }
    }
}

fn paint_current(ui: &mut Ui<'_>, area: Rect, y: u16, selected: bool, ink: &Ink) {
    if selected {
        ui.fill(Rect::new(3, y, 33, 1), bold(ink.primary, ink.select_bg));
        put(ui, area, 3, y, "▎", bold(ink.accent, ink.select_bg));
        let mut label = "Current directory".to_owned();
        while label.chars().count() < 28 {
            label.push(' ');
        }
        put(ui, area, 7, y, &label, bold(ink.accent, ink.select_bg));
    } else {
        put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
        put(
            ui,
            area,
            7,
            y,
            "Current directory",
            plain(ink.primary, ink.canvas),
        );
    }
}

fn paint_expanded_workspace(ui: &mut Ui<'_>, area: Rect, y: u16, name: &str, ink: &Ink) {
    ui.fill(Rect::new(3, y, 33, 1), bold(ink.primary, ink.expanded_bg));
    put(ui, area, 3, y, "▎", bold(ink.accent, ink.expanded_bg));
    put(ui, area, 5, y, "▾", plain(ink.secondary, ink.expanded_bg));
    let mut label = name.to_owned();
    while label.chars().count() < 28 {
        label.push(' ');
    }
    put(ui, area, 7, y, &label, bold(ink.accent, ink.expanded_bg));
    put(ui, area, 35, y, " ", bold(ink.primary, ink.expanded_bg));
}

fn paint_child(ui: &mut Ui<'_>, area: Rect, y: u16, instance: &SplitInstance<'_>, ink: &Ink) {
    let prefix = format!(" {}  ", instance.id);
    let rest = format!("{} · {}", instance.role, instance.agent);
    let text = if instance.live {
        let body = middle_ellipsis(&prefix, &rest, 27);
        format!("{body:<29}")
    } else {
        format!("{prefix}{rest}")
    };
    put(ui, area, 3, y, " ", plain(ink.canvas, ink.canvas));
    let mark = if instance.live { "◉" } else { "◌" };
    let mark_style = if instance.live {
        plain(ink.accent, ink.canvas)
    } else {
        plain(ink.muted, ink.canvas)
    };
    put(ui, area, 7, y, mark, mark_style);
    put(ui, area, 8, y, &text, plain(ink.primary, ink.canvas));
    if instance.dirty {
        put(ui, area, 34, y, "•", plain(ink.stale, ink.canvas));
    }
}

fn middle_ellipsis(prefix: &str, rest: &str, budget: usize) -> String {
    let prefix_len = prefix.chars().count();
    if prefix_len.saturating_add(rest.chars().count()) <= budget {
        return format!("{prefix}{rest}");
    }
    let room = budget.saturating_sub(prefix_len);
    let tail = 8.min(room.saturating_sub(1));
    let head = room.saturating_sub(1).saturating_sub(tail);
    let head_s: String = rest.chars().take(head).collect();
    let tail_s: String = rest
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}{head_s}…{tail_s}")
}

fn paint_detail(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let card = plain(ink.primary, ink.card);
    let muted = plain(ink.muted, ink.card);
    let badge = plain(ink.badge, ink.card);
    let secondary = plain(ink.secondary, ink.card);
    match frame.mode {
        SplitMode::Empty => {
            put(ui, area, 42, 2, "Current directory", secondary);
            put(ui, area, 108, 2, "not saved", badge);
            put(ui, area, 42, 3, &"─".repeat(75), plain(ink.rule, ink.card));
            put(
                ui,
                area,
                60,
                17,
                "Create a workspace from this directory.",
                muted,
            );
            put(
                ui,
                area,
                49,
                19,
                &format!(
                    "{} is mounted at its own path inside the",
                    frame.cwd_display
                ),
                badge,
            );
            put(
                ui,
                area,
                45,
                20,
                "Construct. Enter launches with defaults; n creates a saved workspace.",
                badge,
            );
        }
        SplitMode::Saved | SplitMode::Expanded => {
            let (title, badge_text, badge_x) = if frame.mode == SplitMode::Expanded {
                (
                    frame.workspace_name.to_owned(),
                    format!("saved workspace · {} running", frame.live_here),
                    90,
                )
            } else {
                (
                    format!("Current directory · {}", frame.workspace_name),
                    "saved workspace".to_owned(),
                    102,
                )
            };
            put(ui, area, 42, 2, &title, secondary);
            put(ui, area, badge_x, 2, &badge_text, badge);
            put(ui, area, 42, 3, &"─".repeat(75), plain(ink.rule, ink.card));
            let mut y = 4u16;
            put(ui, area, 42, y, "Working dir", muted);
            put(ui, area, 56, y, frame.workdir, card);
            y = y.saturating_add(1);
            for (index, mount) in frame.mounts.iter().enumerate() {
                if index == 0 {
                    put(ui, area, 42, y, "Mounts", muted);
                }
                put(
                    ui,
                    area,
                    56,
                    y,
                    &format!("{} · {} {}", mount.source, mount.mode, mount.isolation),
                    card,
                );
                y = y.saturating_add(1);
            }
            put(ui, area, 42, y, "Roles", muted);
            put(ui, area, 56, y, frame.role_line, card);
            y = y.saturating_add(1);
            put(ui, area, 42, y, "Environments", muted);
            put(ui, area, 56, y, frame.env_line, card);
            y = y.saturating_add(1);
            for (index, line) in frame.account_lines.iter().enumerate() {
                if index == 0 {
                    put(ui, area, 42, y, "Accounts", muted);
                }
                put(ui, area, 56, y, line, card);
                y = y.saturating_add(1);
            }
            put(ui, area, 42, y, "Policies", muted);
            put(ui, area, 56, y, frame.policy_line, card);
            y = y.saturating_add(2);
            put(ui, area, 42, y, "Instances", bold(ink.secondary, ink.card));
            let stale = frame.mode == SplitMode::Saved
                && frame.alert.is_some_and(|alert| alert.daemon_stale);
            let meta = if stale {
                format!("▲ daemon stale · {}", frame.daemon_ago)
            } else {
                format!("daemon · {}", frame.daemon_ago)
            };
            let meta_x = 117u16.saturating_sub(width(&meta));
            put(ui, area, meta_x, y, &meta, badge);
            y = y.saturating_add(1);
            for instance in frame.instances {
                let mark = if instance.live { "◉ " } else { "◌ " };
                let mark_style = if instance.live {
                    plain(ink.accent, ink.card)
                } else {
                    muted
                };
                put(ui, area, 42, y, mark, mark_style);
                let body = format!("{}  {} · {} · ", instance.id, instance.role, instance.agent);
                if instance.live {
                    put(ui, area, 44, y, &body, card);
                    let status_x = 44u16.saturating_add(width(&body));
                    put(
                        ui,
                        area,
                        status_x,
                        y,
                        instance.status,
                        plain(ink.accent, ink.card),
                    );
                } else {
                    put(
                        ui,
                        area,
                        44,
                        y,
                        &format!("{body}{}", instance.status),
                        secondary,
                    );
                }
                y = y.saturating_add(1);
            }
        }
    }
}

fn paint_buttons(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let cap = plain(ink.button, ink.button);
    let label = plain(ink.primary, ink.button);
    match frame.mode {
        SplitMode::Empty => {
            ui.fill(Rect::new(41, 36, 8, 1), label);
            ui.fill(Rect::new(51, 36, 18, 1), label);
            put(ui, area, 41, 36, " ", cap);
            put(ui, area, 42, 36, "Launch", label);
            put(ui, area, 51, 36, " ", cap);
            put(ui, area, 52, 36, "Create workspace", label);
        }
        SplitMode::Saved | SplitMode::Expanded => {
            ui.fill(Rect::new(41, 36, 8, 1), label);
            ui.fill(Rect::new(51, 36, 6, 1), label);
            put(ui, area, 41, 36, " ", cap);
            put(ui, area, 42, 36, "Launch", label);
            put(ui, area, 51, 36, " ", cap);
            put(ui, area, 52, 36, "Edit", label);
            if frame.mode == SplitMode::Expanded {
                ui.fill(Rect::new(59, 36, 9, 1), label);
                ui.fill(Rect::new(70, 36, 9, 1), label);
                put(ui, area, 59, 36, " ", cap);
                put(ui, area, 60, 36, "Prewarm ", label);
                put(ui, area, 70, 36, " ", cap);
                put(ui, area, 71, 36, "Delete… ", label.fg(ink.danger_button));
            }
        }
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    ui.paint_style(
        Rect::new(area.x, y, area.width, 1),
        plain(ink.primary, ink.canvas),
    );
    if frame.mode == SplitMode::Saved
        && let Some(alert) = frame.alert
    {
        paint_degraded_hints(ui, area, y, alert, ink);
        return;
    }
    let mode = frame.mode;
    let key = bold(ink.primary, ink.canvas);
    let hint = plain(ink.muted, ink.canvas);
    let pairs: &[(&str, &str)] = match mode {
        SplitMode::Empty => &[
            ("Enter", "Launch"),
            ("n", "New"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("s", "Settings"),
            ("?", "Help"),
            ("q", "Quit"),
        ],
        SplitMode::Saved => &[
            ("Enter", "Launch"),
            ("n", "New"),
            ("e", "Edit"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("s", "Settings"),
            ("?", "Help"),
            ("q", "Quit"),
        ],
        SplitMode::Expanded => &[
            ("Enter", "Launch"),
            ("→", "Expand"),
            ("e", "Edit"),
            ("n", "New"),
            ("d", "Delete…"),
            ("w", "Prewarm"),
            ("o", "GitHub"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("?", "Help"),
        ],
    };
    let starts: &[u16] = match mode {
        SplitMode::Empty => &[1, 15, 22, 35, 47, 56, 68, 76],
        SplitMode::Saved => &[1, 15, 22, 30, 43, 55, 64, 76, 84],
        SplitMode::Expanded => &[1, 15, 25, 33, 40, 51, 62, 72, 85, 97, 106],
    };
    for (index, (key_text, label)) in pairs.iter().enumerate() {
        let x = starts.get(index).copied().unwrap_or(1);
        put(ui, area, x, y, key_text, key);
        let label_x = x.saturating_add(width(key_text)).saturating_add(1);
        put(ui, area, label_x, y, label, hint);
    }
    if mode == SplitMode::Expanded {
        put(ui, area, 114, y, "…", plain(ink.badge, ink.canvas));
    }
}

fn paint_degraded_hints(ui: &mut Ui<'_>, area: Rect, y: u16, alert: SplitAlert<'_>, ink: &Ink) {
    let key = bold(ink.primary, ink.canvas);
    let hint = plain(ink.muted, ink.canvas);
    put(ui, area, 1, y, "Enter", key);
    put(ui, area, 7, y, "Launch", hint);
    put(ui, area, 15, y, "?", key);
    put(ui, area, 17, y, "Help", hint);
    put(ui, area, 23, y, "…", plain(ink.badge, ink.canvas));
    if alert.daemon_stale {
        put(ui, area, 30, y, "▲", bold(ink.stale, ink.canvas));
    }
    if let Some(error) = alert.index_error.filter(|error| !error.is_empty()) {
        let text =
            format!(" Could not confirm running instances: {error} · list may be incomplete");
        put(ui, area, 31, y, &text, plain(ink.stale, ink.canvas));
    }
}

#[cfg(test)]
mod tests {
    use super::middle_ellipsis;

    #[test]
    fn live_child_keeps_the_baseline_middle_cut() {
        let text = middle_ellipsis(" 7f3a  ", "the-architect · Claude Code", 27);
        assert_eq!(text, " 7f3a  the-archite…ude Code");
    }

    #[test]
    fn long_workspace_uses_the_baseline_middle_cut() {
        let name = "billing-reconciliation-service-with-a-very-long-name-for-truncation";
        assert_eq!(
            crate::text::truncate_middle(name, super::LABEL_BUDGET),
            "billing-reconcilia…runcation"
        );
    }
}
