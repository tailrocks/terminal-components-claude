//! Filtered accounts list: menu, boxed rows, detail band, and hints.
//!
//! Column positions come from the labels and the screen edges.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct FilterHint<'a> {
    pub key: &'a str,
    pub action: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TailTone {
    Muted,
    Warning,
}

pub struct FilterRow<'a> {
    pub depth: u8,
    pub twist: Option<&'a str>,
    pub name: &'a str,
    pub alert: bool,
    pub tail: Option<&'a str>,
    pub tail_tone: TailTone,
    pub selected: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetailInk {
    Plain,
    Muted,
    Danger,
    DangerBold,
    Warning,
    Info,
}

pub struct DetailSpan<'a> {
    pub text: &'a str,
    pub ink: DetailInk,
}

pub struct DetailLine<'a> {
    pub spans: &'a [DetailSpan<'a>],
}

pub struct AccountsFilter<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub status: &'a str,
    pub title: &'a str,
    pub summary: &'a str,
    pub rows: &'a [FilterRow<'a>],
    pub heading: &'a str,
    pub heading_meta: &'a str,
    pub facts: &'a [DetailLine<'a>],
    pub hints: &'a [FilterHint<'a>],
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
const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const INFO: Color = Color::Rgb(0x79, 0xb8, 0xff);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);

pub fn paint_accounts_filter(ui: &mut Ui<'_>, area: Rect, page: &AccountsFilter<'_>) {
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
    let mut y = box_top.saturating_add(1);
    for row in page.rows {
        if y >= box_bottom {
            break;
        }
        paint_row(ui, area, y, border_x, row, &ink);
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

fn paint_header(ui: &mut Ui<'_>, area: Rect, page: &AccountsFilter<'_>, ink: &Ink) {
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
    page: &AccountsFilter<'_>,
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

fn paint_row(ui: &mut Ui<'_>, area: Rect, y: u16, border_x: u16, row: &FilterRow<'_>, ink: &Ink) {
    let left = area.x.saturating_add(1);
    let edge = plain(ink.border, ink.canvas);
    put(ui, area, left, y, "│", edge);
    put(ui, area, border_x, y, "│", edge);
    let inner = left.saturating_add(2);
    if row.selected {
        ui.fill(
            Rect {
                x: inner,
                y,
                width: border_x.saturating_sub(inner).saturating_sub(1),
                height: 1,
            },
            bold(ink.primary, SELECT_BG),
        );
        put(ui, area, inner, y, "▎", bold(ink.accent, SELECT_BG));
        put(
            ui,
            area,
            inner.saturating_add(1),
            y,
            " ",
            bold(ink.primary, SELECT_BG),
        );
        put(
            ui,
            area,
            inner.saturating_add(2),
            y,
            " ",
            plain(ink.secondary, SELECT_BG),
        );
        put(
            ui,
            area,
            inner.saturating_add(3),
            y,
            " ",
            bold(ink.primary, SELECT_BG),
        );
        let name_x = inner.saturating_add(4);
        put(ui, area, name_x, y, row.name, bold(ink.accent, SELECT_BG));
        let pad_end = border_x.saturating_sub(2);
        let pad_from = name_x.saturating_add(width(row.name));
        if pad_end > pad_from {
            put(
                ui,
                area,
                pad_from,
                y,
                &" ".repeat(usize::from(pad_end.saturating_sub(pad_from))),
                bold(ink.accent, SELECT_BG),
            );
        }
        return;
    }
    put(ui, area, inner, y, " ", plain(ink.canvas, ink.canvas));
    let name_x = if row.depth >= 2 {
        put(
            ui,
            area,
            inner.saturating_add(4),
            y,
            " ",
            plain(ink.secondary, ink.canvas),
        );
        inner.saturating_add(6)
    } else {
        if let Some(twist) = row.twist {
            put(
                ui,
                area,
                inner.saturating_add(2),
                y,
                twist,
                plain(ink.secondary, ink.canvas),
            );
        }
        inner.saturating_add(4)
    };
    put(
        ui,
        area,
        name_x,
        y,
        row.name,
        plain(ink.primary, ink.canvas),
    );
    if let Some(tail) = row.tail {
        let tail_end = border_x.saturating_sub(2);
        let tail_x = tail_end.saturating_sub(width(tail));
        let tone = match row.tail_tone {
            TailTone::Muted => ink.muted,
            TailTone::Warning => WARNING,
        };
        put(ui, area, tail_x, y, tail, plain(tone, ink.canvas));
        if row.alert {
            put(
                ui,
                area,
                tail_x.saturating_sub(4),
                y,
                "!",
                plain(DANGER, ink.canvas),
            );
        }
    }
}

fn paint_detail(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    border_x: u16,
    page: &AccountsFilter<'_>,
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
    let rule_w = meta_end.saturating_sub(rule_from);
    put(
        ui,
        area,
        rule_from,
        y.saturating_add(1),
        &"─".repeat(usize::from(rule_w)),
        plain(RULE, ink.chrome),
    );
    for (i, line) in page.facts.iter().enumerate() {
        let mut x = area.x.saturating_add(1);
        let row = y.saturating_add(2).saturating_add(i as u16);
        for span in line.spans {
            let style = match span.ink {
                DetailInk::Plain => plain(ink.primary, ink.chrome),
                DetailInk::Muted => plain(ink.muted, ink.chrome),
                DetailInk::Danger => plain(DANGER, ink.chrome),
                DetailInk::DangerBold => bold(DANGER, ink.chrome),
                DetailInk::Warning => plain(WARNING, ink.chrome),
                DetailInk::Info => plain(INFO, ink.chrome),
            };
            put(ui, area, x, row, span.text, style);
            x = x.saturating_add(width(span.text));
        }
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &AccountsFilter<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    for hint in page.hints {
        put(ui, area, x, y, hint.key, bold(ink.primary, ink.canvas));
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, plain(ink.muted, ink.canvas));
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
}
