//! Narrow workspace manager: menu, boxed list, detail band, hint row.
//!
//! Column positions follow the labels and the screen width. The last list
//! row keeps 55% of its ink, which is the baseline bottom fade on black.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const SEAM: Color = Color::Rgb(0x80, 0x80, 0x80);
const SEAM_IDLE: Color = Color::Rgb(0x47, 0x47, 0x47);
const DETAIL_RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const FADE_KEEP: f32 = 0.55;

pub struct ManagerLeaf<'a> {
    pub name: &'a str,
    pub expanded: bool,
    pub status: &'a str,
    /// Danger ink, such as `! failed`.
    pub danger: bool,
    /// Muted ink, such as `1 record`. Idle rows use `faint` and the dotted seam.
    pub muted: bool,
    pub faint: bool,
}

pub struct ManagerFrame<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub alert: &'a str,
    pub title: &'a str,
    pub summary: &'a str,
    pub selected: &'a str,
    pub saved: &'a str,
    pub saved_as: &'a str,
    pub saved_target: &'a str,
    pub rows: &'a [ManagerLeaf<'a>],
    pub detail_heading: &'a str,
    pub detail_side: &'a str,
    pub detail_path: &'a str,
    pub detail_roles: &'a str,
    pub instance_mark: &'a str,
    pub instance_text: &'a str,
    pub instance_state: &'a str,
    pub hint_key: &'a str,
    pub hint_label: &'a str,
    pub warning_mark: &'a str,
    pub warning: &'a str,
}

struct Ink {
    canvas: Color,
    chrome: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
    border: Color,
    danger: Color,
    warning: Color,
}

pub fn paint_manager_frame(ui: &mut Ui<'_>, area: Rect, frame: &ManagerFrame<'_>) {
    if area.width < 40 || area.height < 20 {
        return;
    }
    let ink = ink_from(ui);
    let canvas = plain(ink.primary, ink.canvas);
    ui.fill(area, canvas);
    paint_header(ui, area, frame, &ink);
    let row_y = area.y.saturating_add(3);
    let list_h = 1u16.saturating_add(frame.rows.len() as u16);
    paint_box_top(ui, area, frame, &ink);
    paint_selected(ui, area, row_y, frame, &ink);
    for (i, row) in frame.rows.iter().enumerate() {
        let y = row_y.saturating_add(1).saturating_add(i as u16);
        let fade = i + 1 == frame.rows.len();
        paint_leaf(ui, area, y, row, fade, &ink);
    }
    let bottom_y = row_y.saturating_add(list_h);
    paint_box_bottom(ui, area, bottom_y, &ink);
    paint_detail(ui, area, bottom_y.saturating_add(2), frame, &ink);
    paint_hint(ui, area, frame, &ink);
}

fn ink_from(ui: &Ui<'_>) -> Ink {
    let theme = ui.theme_ref();
    Ink {
        canvas: theme.bg(Surface::Canvas),
        chrome: theme.bg(Surface::Surface),
        primary: step(theme, FgStep::Primary),
        secondary: step(theme, FgStep::Secondary),
        muted: step(theme, FgStep::Muted),
        faint: step(theme, FgStep::Faint),
        accent: theme.color.accent,
        on_accent: theme.color.on_accent,
        border: theme.color.border_strong,
        danger: Color::Rgb(0xf0, 0x5d, 0x5d),
        warning: theme.color.warning,
    }
}

fn step(theme: &crate::theme::Theme, step: FgStep) -> Color {
    theme
        .color
        .fg
        .get(step.index())
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

fn paint_header(ui: &mut Ui<'_>, area: Rect, frame: &ManagerFrame<'_>, ink: &Ink) {
    let y = area.y;
    let brand = format!(" {} ", frame.brand);
    put(
        ui,
        area,
        area.x.saturating_add(1),
        y,
        &brand,
        bold(ink.on_accent, ink.accent),
    );
    let mut x = area
        .x
        .saturating_add(1)
        .saturating_add(width(&brand))
        .saturating_add(2);
    let menu = plain(ink.secondary, ink.canvas);
    for label in frame.menus {
        let text = format!(" {label} ");
        put(ui, area, x, y, &text, menu);
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
    let alert_w = width(frame.alert);
    let alert_x = area.right().saturating_sub(2).saturating_sub(alert_w);
    put(
        ui,
        area,
        alert_x.saturating_sub(3),
        y,
        "…",
        plain(ink.muted, ink.canvas),
    );
    put(
        ui,
        area,
        alert_x,
        y,
        frame.alert,
        plain(ink.danger, ink.canvas),
    );
}

fn paint_box_top(ui: &mut Ui<'_>, area: Rect, frame: &ManagerFrame<'_>, ink: &Ink) {
    let y = area.y.saturating_add(2);
    let border = plain(ink.border, ink.canvas);
    let title = format!(" {} ", frame.title);
    let summary = format!(" {} ", frame.summary);
    let summary_x = area
        .right()
        .saturating_sub(3)
        .saturating_sub(width(&summary));
    let title_x = area.x.saturating_add(3);
    let title_end = title_x.saturating_add(width(&title));
    put(ui, area, area.x.saturating_add(1), y, "╭─", border);
    put(ui, area, title_x, y, &title, bold(ink.primary, ink.canvas));
    if summary_x > title_end {
        let dashes = "─".repeat(usize::from(summary_x - title_end));
        put(ui, area, title_end, y, &dashes, border);
    }
    put(
        ui,
        area,
        summary_x,
        y,
        &summary,
        plain(ink.faint, ink.canvas),
    );
    put(
        ui,
        area,
        summary_x.saturating_add(width(&summary)),
        y,
        "─╮",
        border,
    );
}

fn paint_selected(ui: &mut Ui<'_>, area: Rect, y: u16, frame: &ManagerFrame<'_>, ink: &Ink) {
    let seam_x = area.right().saturating_sub(4);
    let border = plain(ink.border, ink.canvas);
    put(ui, area, area.x.saturating_add(1), y, "│", border);
    put(ui, area, area.right().saturating_sub(2), y, "│", border);
    ui.fill(
        Rect {
            x: area.x.saturating_add(3),
            y,
            width: seam_x.saturating_sub(area.x).saturating_sub(3),
            height: 1,
        },
        bold(ink.primary, SELECT_BG),
    );
    put(
        ui,
        area,
        area.x.saturating_add(3),
        y,
        "▎",
        bold(ink.accent, SELECT_BG),
    );
    let name_x = area.x.saturating_add(7);
    let meta = meta_width(frame);
    let meta_x = seam_x.saturating_sub(1).saturating_sub(meta);
    let green = bold(ink.accent, SELECT_BG);
    if meta_x > name_x.saturating_add(2) {
        let pad = meta_x.saturating_sub(2).saturating_sub(name_x);
        put(ui, area, name_x, y, &" ".repeat(usize::from(pad)), green);
    }
    put(ui, area, name_x, y, frame.selected, green);
    put(
        ui,
        area,
        meta_x.saturating_sub(2),
        y,
        "  ",
        bold(ink.primary, SELECT_BG),
    );
    let word = plain(ink.secondary, SELECT_BG);
    let gap = plain(ink.muted, SELECT_BG);
    put(ui, area, meta_x, y, frame.saved, word);
    let mut x = meta_x.saturating_add(width(frame.saved));
    put(ui, area, x, y, " ", gap);
    x = x.saturating_add(1);
    put(ui, area, x, y, frame.saved_as, word);
    x = x.saturating_add(width(frame.saved_as));
    put(ui, area, x, y, " ", gap);
    x = x.saturating_add(1);
    put(ui, area, x, y, frame.saved_target, word);
    put(ui, area, seam_x, y, "┃", plain(SEAM, ink.canvas));
}

fn meta_width(frame: &ManagerFrame<'_>) -> u16 {
    width(frame.saved)
        .saturating_add(1)
        .saturating_add(width(frame.saved_as))
        .saturating_add(1)
        .saturating_add(width(frame.saved_target))
}

fn paint_leaf(ui: &mut Ui<'_>, area: Rect, y: u16, row: &ManagerLeaf<'_>, fade: bool, ink: &Ink) {
    let seam_x = area.right().saturating_sub(4);
    let border = plain(ink.border, ink.canvas);
    put(ui, area, area.x.saturating_add(1), y, "│", border);
    put(ui, area, area.right().saturating_sub(2), y, "│", border);
    put(
        ui,
        area,
        area.x.saturating_add(3),
        y,
        " ",
        plain(ink.canvas, ink.canvas),
    );
    let mark = if row.expanded { "▸" } else { " " };
    put(
        ui,
        area,
        area.x.saturating_add(5),
        y,
        mark,
        plain(ink.secondary, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(6),
        y,
        &format!(" {}", row.name),
        plain(ink.primary, ink.canvas),
    );
    let status_x = seam_x.saturating_sub(1).saturating_sub(width(row.status));
    let status_ink = if row.danger {
        ink.danger
    } else if row.faint {
        ink.faint
    } else if row.muted {
        ink.muted
    } else {
        ink.secondary
    };
    put(
        ui,
        area,
        status_x,
        y,
        row.status,
        plain(status_ink, ink.canvas),
    );
    let (glyph, seam_ink) = if row.faint {
        ("┆", SEAM_IDLE)
    } else {
        ("┃", SEAM)
    };
    put(ui, area, seam_x, y, glyph, plain(seam_ink, ink.canvas));
    if fade {
        let x = area.x.saturating_add(4);
        ui.recolor_rgb(
            Rect {
                x,
                y,
                width: seam_x.saturating_sub(x),
                height: 1,
            },
            fade_black,
        );
    }
}

fn fade_black(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(scale(r), scale(g), scale(b)),
        other => other,
    }
}

fn scale(channel: u8) -> u8 {
    (f32::from(channel) * FADE_KEEP).round() as u8
}

fn paint_box_bottom(ui: &mut Ui<'_>, area: Rect, y: u16, ink: &Ink) {
    let width = area.width.saturating_sub(2);
    let mut line = String::from("╰");
    line.push_str(&"─".repeat(usize::from(width.saturating_sub(2))));
    line.push('╯');
    put(
        ui,
        area,
        area.x.saturating_add(1),
        y,
        &line,
        plain(ink.border, ink.canvas),
    );
}

fn paint_detail(ui: &mut Ui<'_>, area: Rect, y: u16, frame: &ManagerFrame<'_>, ink: &Ink) {
    let x = area.x.saturating_add(1);
    let band = area.width.saturating_sub(2);
    ui.fill(
        Rect {
            x,
            y,
            width: band,
            height: 6,
        },
        plain(ink.primary, ink.chrome),
    );
    let text_x = area.x.saturating_add(3);
    put(
        ui,
        area,
        text_x,
        y,
        frame.detail_heading,
        plain(ink.secondary, ink.chrome),
    );
    put(
        ui,
        area,
        area.right().saturating_sub(14),
        y,
        frame.detail_side,
        plain(ink.faint, ink.chrome),
    );
    let rule = "─".repeat(usize::from(band.saturating_sub(4)));
    put(
        ui,
        area,
        text_x,
        y.saturating_add(1),
        &rule,
        plain(DETAIL_RULE, ink.chrome),
    );
    put(
        ui,
        area,
        text_x,
        y.saturating_add(2),
        frame.detail_path,
        plain(ink.secondary, ink.chrome),
    );
    put(
        ui,
        area,
        text_x,
        y.saturating_add(3),
        frame.detail_roles,
        plain(ink.muted, ink.chrome),
    );
    put(
        ui,
        area,
        text_x,
        y.saturating_add(4),
        frame.instance_mark,
        plain(ink.accent, ink.chrome),
    );
    let text_at = text_x.saturating_add(width(frame.instance_mark));
    put(
        ui,
        area,
        text_at,
        y.saturating_add(4),
        frame.instance_text,
        plain(ink.secondary, ink.chrome),
    );
    put(
        ui,
        area,
        text_at.saturating_add(width(frame.instance_text)),
        y.saturating_add(4),
        frame.instance_state,
        plain(ink.accent, ink.chrome),
    );
}

fn paint_hint(ui: &mut Ui<'_>, area: Rect, frame: &ManagerFrame<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    put(
        ui,
        area,
        area.x.saturating_add(1),
        y,
        frame.hint_key,
        bold(ink.primary, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(3),
        y,
        frame.hint_label,
        plain(ink.muted, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(9),
        y,
        "…",
        plain(ink.faint, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(13),
        y,
        frame.warning_mark,
        bold(ink.warning, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(14),
        y,
        frame.warning,
        plain(ink.warning, ink.canvas),
    );
}
