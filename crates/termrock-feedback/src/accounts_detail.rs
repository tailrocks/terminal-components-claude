//! Accounts list with one account selected and its detail band open.
//!
//! Column positions come from the labels, the scrollbar, and the screen edges.
//! The last faded row keeps 55% of its ink, the same ratio as the list edge.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::accounts_filter::{DetailInk, DetailLine, FilterHint};
use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub use crate::accounts_filter::FilterHint as DetailHint;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccountRole {
    Root,
    Group,
    Item,
    Note,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetailAlert {
    Danger,
    Warning,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetailTail {
    Muted,
    Warning,
    Secondary,
}

pub struct AccountRow<'a> {
    pub role: AccountRole,
    pub mark: Option<&'a str>,
    pub name: &'a str,
    pub alert: Option<&'a str>,
    pub alert_tone: DetailAlert,
    pub tail: Option<&'a str>,
    pub tail_tone: DetailTail,
    pub selected: bool,
    pub faded: bool,
}

pub struct AccountsDetail<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub status: &'a str,
    pub title: &'a str,
    pub summary: &'a str,
    pub rows: &'a [AccountRow<'a>],
    pub total: u16,
    pub offset: u16,
    pub heading: &'a str,
    pub heading_meta: &'a str,
    pub facts: &'a [DetailLine<'a>],
    pub hints: &'a [FilterHint<'a>],
    pub hint_overflow: &'a str,
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
}

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const ERROR_STRONG: Color = Color::Rgb(0xff, 0x8a, 0x8a);
const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const SEAM: Color = Color::Rgb(0x47, 0x47, 0x47);
const SEAM_HEAVY: Color = Color::Rgb(0x80, 0x80, 0x80);
const FADE_KEEP: f32 = 0.55;

pub fn paint_accounts_detail(ui: &mut Ui<'_>, area: Rect, page: &AccountsDetail<'_>) {
    if area.width < 40 || area.height < 16 {
        return;
    }
    let theme = ui.theme_ref();
    let ink = Ink {
        canvas: theme.bg(Surface::Canvas),
        chrome: theme.bg(Surface::Surface),
        primary: step(theme, FgStep::Primary),
        secondary: step(theme, FgStep::Secondary),
        muted: step(theme, FgStep::Muted),
        faint: step(theme, FgStep::Faint),
        accent: theme.color.accent,
        on_accent: theme.color.on_accent,
        border: theme.color.border_strong,
    };
    ui.fill(area, plain(ink.primary, ink.canvas));
    paint_header(ui, area, page, &ink);
    let border_x = area.x.saturating_add(area.width.saturating_sub(2));
    let detail_y = area.y.saturating_add(area.height.saturating_sub(8));
    let box_top = area.y.saturating_add(2);
    let box_bottom = detail_y.saturating_sub(2);
    paint_box_top(ui, area, box_top, border_x, page, &ink);
    let scroll_x = border_x.saturating_sub(2);
    let visible = u16::try_from(page.rows.len()).unwrap_or(u16::MAX);
    let thumb = thumb_len(visible, page.total);
    let thumb_at = thumb_at(page.offset, visible, page.total, thumb);
    let mut y = box_top.saturating_add(1);
    for (index, row) in page.rows.iter().enumerate() {
        if y >= box_bottom {
            break;
        }
        let on_thumb = u16::try_from(index).unwrap_or(u16::MAX) >= thumb_at
            && u16::try_from(index).unwrap_or(u16::MAX) < thumb_at.saturating_add(thumb);
        paint_row(ui, area, y, border_x, scroll_x, on_thumb, row, &ink);
        y = y.saturating_add(1);
    }
    while y < box_bottom {
        put(
            ui,
            area,
            area.x.saturating_add(1),
            y,
            "│",
            plain(ink.border, ink.canvas),
        );
        put(ui, area, border_x, y, "│", plain(ink.border, ink.canvas));
        y = y.saturating_add(1);
    }
    let dashes = "─".repeat(usize::from(
        border_x.saturating_sub(area.x).saturating_sub(2),
    ));
    put(
        ui,
        area,
        area.x.saturating_add(1),
        box_bottom,
        &format!("╰{dashes}╯"),
        plain(ink.border, ink.canvas),
    );
    paint_detail(ui, area, detail_y, border_x, page, &ink);
    paint_hints(ui, area, page, &ink);
}

fn thumb_len(visible: u16, total: u16) -> u16 {
    if visible == 0 || total == 0 {
        return 1;
    }
    visible
        .saturating_mul(visible)
        .saturating_div(total)
        .max(1)
        .min(visible)
}

fn thumb_at(offset: u16, visible: u16, total: u16, thumb: u16) -> u16 {
    let span = visible.saturating_sub(thumb);
    let travel = total.saturating_sub(visible);
    if travel == 0 {
        0
    } else {
        offset.saturating_mul(span).saturating_div(travel)
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

fn paint_header(ui: &mut Ui<'_>, area: Rect, page: &AccountsDetail<'_>, ink: &Ink) {
    let y = area.y;
    let brand = format!(" {} ", page.brand);
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
    for label in page.menus {
        let text = format!(" {label} ");
        put(ui, area, x, y, &text, menu);
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
    let status_x = area
        .right()
        .saturating_sub(2)
        .saturating_sub(width(page.status));
    let crumb_x = status_x.saturating_sub(2).saturating_sub(width(page.crumb));
    put(
        ui,
        area,
        crumb_x.saturating_sub(3),
        y,
        "…",
        plain(ink.muted, ink.canvas),
    );
    put(
        ui,
        area,
        crumb_x,
        y,
        page.crumb,
        plain(ink.secondary, ink.canvas),
    );
    put(
        ui,
        area,
        status_x,
        y,
        page.status,
        plain(ink.secondary, ink.canvas),
    );
}

fn paint_box_top(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    border_x: u16,
    page: &AccountsDetail<'_>,
    ink: &Ink,
) {
    let left = area.x.saturating_add(1);
    put(ui, area, left, y, "╭─", plain(ink.border, ink.canvas));
    let title_x = left.saturating_add(2);
    put(
        ui,
        area,
        title_x,
        y,
        page.title,
        bold(ink.primary, ink.canvas),
    );
    let summary_x = border_x
        .saturating_sub(1)
        .saturating_sub(width(page.summary));
    let dash_from = title_x.saturating_add(width(page.title));
    if summary_x > dash_from {
        put(
            ui,
            area,
            dash_from,
            y,
            &"─".repeat(usize::from(summary_x.saturating_sub(dash_from))),
            plain(ink.border, ink.canvas),
        );
    }
    put(
        ui,
        area,
        summary_x,
        y,
        page.summary,
        plain(ink.faint, ink.canvas),
    );
    put(
        ui,
        area,
        border_x.saturating_sub(1),
        y,
        "─╮",
        plain(ink.border, ink.canvas),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    border_x: u16,
    scroll_x: u16,
    on_thumb: bool,
    row: &AccountRow<'_>,
    ink: &Ink,
) {
    let edge = plain(ink.border, ink.canvas);
    put(ui, area, area.x.saturating_add(1), y, "│", edge);
    put(ui, area, border_x, y, "│", edge);
    let gutter = area.x.saturating_add(3);
    if row.selected {
        paint_selected(ui, area, y, gutter, scroll_x, row, ink);
    } else {
        paint_idle(ui, area, y, gutter, scroll_x, row, ink);
        if row.faded {
            ui.recolor_rgb(
                Rect {
                    x: gutter.saturating_add(1),
                    y,
                    width: scroll_x.saturating_sub(gutter).saturating_sub(1),
                    height: 1,
                },
                fade_keep,
            );
        }
    }
    let glyph = if on_thumb { "┃" } else { "┆" };
    let seam = if on_thumb { SEAM_HEAVY } else { SEAM };
    put(ui, area, scroll_x, y, glyph, plain(seam, ink.canvas));
}

fn paint_selected(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    gutter: u16,
    scroll_x: u16,
    row: &AccountRow<'_>,
    ink: &Ink,
) {
    ui.fill(
        Rect {
            x: gutter,
            y,
            width: scroll_x.saturating_sub(gutter),
            height: 1,
        },
        bold(ink.primary, SELECT_BG),
    );
    put(ui, area, gutter, y, "▎", bold(ink.accent, SELECT_BG));
    let (mark_x, name_x) = slots(row.role, gutter);
    put(
        ui,
        area,
        mark_x,
        y,
        row.mark.unwrap_or(" "),
        plain(ink.secondary, SELECT_BG),
    );
    put(ui, area, name_x, y, row.name, bold(ink.accent, SELECT_BG));
    let name_end = name_x.saturating_add(width(row.name));
    let tail_x = row
        .tail
        .map(|tail| scroll_x.saturating_sub(1).saturating_sub(width(tail)));
    let pad_end = match (tail_x, row.alert) {
        (Some(tail_x), Some(_)) => tail_x.saturating_sub(4),
        (Some(tail_x), None) => tail_x,
        (None, _) => scroll_x.saturating_sub(1),
    };
    if pad_end > name_end {
        put(
            ui,
            area,
            name_end,
            y,
            &" ".repeat(usize::from(pad_end.saturating_sub(name_end))),
            bold(ink.accent, SELECT_BG),
        );
    }
    if let Some(tail_x) = tail_x {
        if let Some(alert) = row.alert {
            put(
                ui,
                area,
                tail_x.saturating_sub(4),
                y,
                alert,
                bold(ERROR_STRONG, SELECT_BG),
            );
        }
        put(
            ui,
            area,
            tail_x,
            y,
            row.tail.unwrap_or(""),
            plain(tail_color(row.tail_tone, ink), SELECT_BG),
        );
    }
}

fn paint_idle(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    gutter: u16,
    scroll_x: u16,
    row: &AccountRow<'_>,
    ink: &Ink,
) {
    put(ui, area, gutter, y, " ", plain(ink.canvas, ink.canvas));
    let (mark_x, name_x) = slots(row.role, gutter);
    let mark = row.mark.unwrap_or(" ");
    put(ui, area, mark_x, y, mark, plain(ink.secondary, ink.canvas));
    let name_ink = if row.role == AccountRole::Note {
        ink.muted
    } else {
        ink.primary
    };
    put(ui, area, name_x, y, row.name, plain(name_ink, ink.canvas));
    let Some(tail) = row.tail else {
        return;
    };
    let tail_x = scroll_x.saturating_sub(1).saturating_sub(width(tail));
    if let Some(alert) = row.alert {
        let tone = match row.alert_tone {
            DetailAlert::Danger => DANGER,
            DetailAlert::Warning => WARNING,
        };
        put(
            ui,
            area,
            tail_x.saturating_sub(4),
            y,
            alert,
            plain(tone, ink.canvas),
        );
    }
    put(
        ui,
        area,
        tail_x,
        y,
        tail,
        plain(tail_color(row.tail_tone, ink), ink.canvas),
    );
}

fn slots(role: AccountRole, gutter: u16) -> (u16, u16) {
    match role {
        AccountRole::Root | AccountRole::Group => {
            (gutter.saturating_add(2), gutter.saturating_add(4))
        }
        AccountRole::Item | AccountRole::Note => {
            (gutter.saturating_add(4), gutter.saturating_add(6))
        }
    }
}

fn tail_color(tone: DetailTail, ink: &Ink) -> Color {
    match tone {
        DetailTail::Muted => ink.muted,
        DetailTail::Warning => WARNING,
        DetailTail::Secondary => ink.secondary,
    }
}

fn fade_keep(color: Color) -> Color {
    let Color::Rgb(red, green, blue) = color else {
        return color;
    };
    let mix = |channel: u8| -> u8 { (f32::from(channel) * FADE_KEEP).round() as u8 };
    Color::Rgb(mix(red), mix(green), mix(blue))
}

fn paint_detail(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    border_x: u16,
    page: &AccountsDetail<'_>,
    ink: &Ink,
) {
    let band = plain(ink.primary, ink.chrome);
    ui.fill(
        Rect {
            x: area.x.saturating_add(1),
            y,
            width: border_x.saturating_sub(area.x),
            height: 6,
        },
        band,
    );
    put(
        ui,
        area,
        area.x.saturating_add(3),
        y,
        page.heading,
        plain(ink.secondary, ink.chrome),
    );
    let meta_end = border_x.saturating_sub(1);
    put(
        ui,
        area,
        meta_end.saturating_sub(width(page.heading_meta)),
        y,
        page.heading_meta,
        plain(ink.faint, ink.chrome),
    );
    let rule_from = area.x.saturating_add(3);
    put(
        ui,
        area,
        rule_from,
        y.saturating_add(1),
        &"─".repeat(usize::from(meta_end.saturating_sub(rule_from))),
        plain(RULE, ink.chrome),
    );
    for (index, fact) in page.facts.iter().enumerate() {
        let mut x = area.x.saturating_add(1);
        let row = y.saturating_add(2).saturating_add(index as u16);
        for span in fact.spans {
            let style = match span.ink {
                DetailInk::Plain => band,
                DetailInk::Muted => plain(ink.muted, ink.chrome),
                DetailInk::Danger => plain(DANGER, ink.chrome),
                DetailInk::DangerBold => bold(DANGER, ink.chrome),
                DetailInk::Warning => plain(WARNING, ink.chrome),
                DetailInk::Info => plain(Color::Rgb(0x79, 0xb8, 0xff), ink.chrome),
            };
            put(ui, area, x, row, span.text, style);
            x = x.saturating_add(width(span.text));
        }
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &AccountsDetail<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    for hint in page.hints {
        put(ui, area, x, y, hint.key, bold(ink.primary, ink.canvas));
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, plain(ink.muted, ink.canvas));
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
    put(
        ui,
        area,
        x,
        y,
        page.hint_overflow,
        plain(ink.faint, ink.canvas),
    );
}
