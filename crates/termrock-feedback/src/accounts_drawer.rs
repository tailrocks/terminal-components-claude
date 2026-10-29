//! Accounts list beside the open account drawer.
//!
//! The list is the narrow column. The drawer is the chrome panel: fields,
//! quota meters, and actions. Positions follow the panel edges.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::accounts_detail::{AccountRole, DetailAlert};
use crate::accounts_filter::{DetailInk, DetailLine, FilterHint};
use crate::settings_page::SettingsCluster;
use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct PaneRow<'a> {
    pub role: AccountRole,
    pub mark: Option<&'a str>,
    pub name: &'a str,
    pub ink: DrawerInk,
    pub alert: Option<&'a str>,
    pub alert_tone: DetailAlert,
    pub selected: bool,
    pub empty: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawerInk {
    Plain,
    Muted,
    Secondary,
    Warning,
    Danger,
    Info,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawerTone {
    Warning,
    Danger,
    Info,
}

pub struct DrawerMeter<'a> {
    pub label: &'a str,
    pub readout: &'a str,
    pub fill_cols: u16,
    pub track_cols: u16,
    pub tone: DrawerTone,
    pub mark: &'a str,
    pub mark_bold: bool,
    pub note: &'a str,
}

pub struct DrawerButton<'a> {
    pub label: &'a str,
    pub danger: bool,
}

pub enum DrawerLine<'a> {
    Title {
        title: &'a str,
        meta: &'a str,
    },
    Rule,
    Field {
        label: &'a str,
        value: &'a str,
        ink: DrawerInk,
    },
    Value {
        text: &'a str,
        ink: DrawerInk,
    },
    Heading {
        label: &'a str,
        meta: &'a str,
    },
    Meter(DrawerMeter<'a>),
    Spans(&'a DetailLine<'a>),
    Text(&'a str),
    Buttons(&'a [DrawerButton<'a>]),
    Blank,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaneChrome {
    /// Open drawer. The list rule is quiet and selection has no focus bar.
    Drawer,
    /// Detail with the drawer closed. The list keeps the strong border and the focus bar.
    Detail,
}

pub struct AccountsDrawer<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub status: &'a str,
    pub cluster: SettingsCluster<'a>,
    pub title: &'a str,
    pub summary: &'a str,
    pub rows: &'a [PaneRow<'a>],
    pub lines: &'a [DrawerLine<'a>],
    pub hints: &'a [FilterHint<'a>],
    pub hint_overflow: &'a str,
    pub chrome: PaneChrome,
}

struct Ink {
    canvas: Color,
    chrome: Color,
    face: Color,
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
const INFO: Color = Color::Rgb(0x79, 0xb8, 0xff);
const BUTTON_DANGER: Color = Color::Rgb(0xff, 0x6b, 0x6b);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const TRACK: Color = Color::Rgb(0x26, 0x26, 0x2a);
const LIST_W: u16 = 39;
const VALUE_AT: u16 = 22;
const LABEL_W: u16 = 22;
const BAR_W: u16 = 34;

pub fn paint_accounts_drawer(ui: &mut Ui<'_>, area: Rect, page: &AccountsDrawer<'_>) {
    if area.width < 80 || area.height < 24 {
        return;
    }
    let theme = ui.theme_ref();
    let ink = Ink {
        canvas: theme.bg(Surface::Canvas),
        chrome: theme.bg(Surface::Surface),
        face: theme.bg(Surface::Overlay),
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
    let list_x = area.x.saturating_add(1);
    let list_right = list_x.saturating_add(LIST_W).saturating_sub(1);
    let box_top = area.y.saturating_add(2);
    let box_bottom = area.bottom().saturating_sub(3);
    paint_list_top(
        ui,
        area,
        box_top,
        list_x,
        list_right,
        page,
        &ink,
        page.chrome,
    );
    let mut y = box_top.saturating_add(1);
    for row in page.rows {
        if y >= box_bottom {
            break;
        }
        paint_pane_row(ui, area, y, list_x, list_right, row, &ink, page.chrome);
        y = y.saturating_add(1);
    }
    while y < box_bottom {
        paint_pane_edges(ui, area, y, list_x, list_right, &ink, page.chrome);
        y = y.saturating_add(1);
    }
    let dashes = "─".repeat(usize::from(LIST_W.saturating_sub(2)));
    put(
        ui,
        area,
        list_x,
        box_bottom,
        &format!("╰{dashes}╯"),
        list_edge(page.chrome, &ink),
    );
    let drawer_x = list_right.saturating_add(2);
    let content_x = drawer_x.saturating_add(1);
    ui.fill(
        Rect {
            x: content_x,
            y: box_top,
            width: area.right().saturating_sub(content_x).saturating_sub(1),
            height: box_bottom.saturating_sub(box_top).saturating_add(1),
        },
        plain(ink.primary, ink.chrome),
    );
    let mut line_y = box_top;
    for line in page.lines {
        if line_y > box_bottom {
            break;
        }
        put(
            ui,
            area,
            drawer_x,
            line_y,
            "│",
            plain(ink.border, ink.canvas),
        );
        paint_line(ui, area, line_y, content_x, line, &ink, page.chrome);
        line_y = line_y.saturating_add(1);
    }
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

fn paint_header(ui: &mut Ui<'_>, area: Rect, page: &AccountsDrawer<'_>, ink: &Ink) {
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
    let secondary = plain(ink.secondary, ink.canvas);
    let cluster = &page.cluster;
    let value_x = area
        .right()
        .saturating_sub(2)
        .saturating_sub(width(cluster.value));
    let mark_x = value_x.saturating_sub(width(cluster.mark));
    let note_x = mark_x.saturating_sub(2).saturating_sub(width(cluster.note));
    let status_x = note_x.saturating_sub(2).saturating_sub(width(page.status));
    let crumb_x = status_x.saturating_sub(2).saturating_sub(width(page.crumb));
    put(ui, area, crumb_x, y, page.crumb, secondary);
    put(ui, area, status_x, y, page.status, secondary);
    put(ui, area, note_x, y, cluster.note, secondary);
    put(
        ui,
        area,
        mark_x,
        y,
        cluster.mark,
        plain(ink.accent, ink.canvas),
    );
    put(ui, area, value_x, y, cluster.value, secondary);
}

#[allow(clippy::too_many_arguments)]
fn paint_list_top(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    list_x: u16,
    list_right: u16,
    page: &AccountsDrawer<'_>,
    ink: &Ink,
    chrome: PaneChrome,
) {
    let edge = list_edge(chrome, ink);
    put(ui, area, list_x, y, "╭─", edge);
    let title_x = list_x.saturating_add(2);
    let title_style = if chrome == PaneChrome::Detail {
        bold(ink.primary, ink.canvas)
    } else {
        plain(ink.secondary, ink.canvas)
    };
    put(ui, area, title_x, y, page.title, title_style);
    let summary_x = list_right
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
            edge,
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
    put(ui, area, list_right.saturating_sub(1), y, "─╮", edge);
}

fn list_edge(chrome: PaneChrome, ink: &Ink) -> PaintStyle {
    plain(
        if chrome == PaneChrome::Detail {
            ink.border
        } else {
            RULE
        },
        ink.canvas,
    )
}

fn paint_pane_edges(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    list_x: u16,
    list_right: u16,
    ink: &Ink,
    chrome: PaneChrome,
) {
    let edge = list_edge(chrome, ink);
    put(ui, area, list_x, y, "│", edge);
    put(ui, area, list_right, y, "│", edge);
}

#[allow(clippy::too_many_arguments)]
fn paint_pane_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    list_x: u16,
    list_right: u16,
    row: &PaneRow<'_>,
    ink: &Ink,
    chrome: PaneChrome,
) {
    paint_pane_edges(ui, area, y, list_x, list_right, ink, chrome);
    if row.empty {
        return;
    }
    let gutter = list_x.saturating_add(2);
    if row.selected {
        paint_selected_row(ui, area, y, gutter, list_right, row, ink, chrome);
        return;
    }
    put(ui, area, gutter, y, " ", plain(ink.canvas, ink.canvas));
    let (mark_x, name_x) = slots(row.role, gutter);
    put(
        ui,
        area,
        mark_x,
        y,
        row.mark.unwrap_or(" "),
        plain(ink.secondary, ink.canvas),
    );
    let name_ink = match row.ink {
        DrawerInk::Muted => ink.muted,
        DrawerInk::Secondary => ink.secondary,
        _ => {
            if row.role == AccountRole::Note {
                ink.muted
            } else {
                ink.primary
            }
        }
    };
    put(ui, area, name_x, y, row.name, plain(name_ink, ink.canvas));
    if name_ink != ink.primary {
        let name_end = name_x.saturating_add(width(row.name));
        let pad_end = list_right.saturating_sub(2);
        if pad_end > name_end {
            put(
                ui,
                area,
                name_end,
                y,
                &" ".repeat(usize::from(pad_end.saturating_sub(name_end))),
                plain(name_ink, ink.canvas),
            );
        }
    }
    if let Some(alert) = row.alert {
        let tone = match row.alert_tone {
            DetailAlert::Danger => DANGER,
            DetailAlert::Warning => WARNING,
        };
        put(
            ui,
            area,
            list_right.saturating_sub(4),
            y,
            alert,
            plain(tone, ink.canvas),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_selected_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    gutter: u16,
    list_right: u16,
    row: &PaneRow<'_>,
    ink: &Ink,
    chrome: PaneChrome,
) {
    let fill_end = list_right.saturating_sub(1);
    ui.fill(
        Rect {
            x: gutter.saturating_add(1),
            y,
            width: fill_end.saturating_sub(gutter).saturating_sub(1),
            height: 1,
        },
        bold(ink.primary, SELECT_BG),
    );
    if chrome == PaneChrome::Detail {
        put(ui, area, gutter, y, "▎", bold(ink.accent, SELECT_BG));
    } else {
        put(ui, area, gutter, y, " ", bold(SELECT_BG, SELECT_BG));
    }
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
    let alert_x = list_right.saturating_sub(4);
    let pad_end = if row.alert.is_some() {
        alert_x
    } else {
        list_right.saturating_sub(2)
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
    let Some(alert) = row.alert else {
        return;
    };
    put(ui, area, alert_x, y, alert, bold(ERROR_STRONG, SELECT_BG));
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

fn paint_line(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    content_x: u16,
    line: &DrawerLine<'_>,
    ink: &Ink,
    chrome: PaneChrome,
) {
    let align = area.right().saturating_sub(3);
    match line {
        DrawerLine::Title { title, meta } => {
            if chrome == PaneChrome::Drawer {
                put(
                    ui,
                    area,
                    content_x.saturating_add(1),
                    y,
                    "▎",
                    plain(ink.accent, ink.chrome),
                );
            }
            let title_style = if chrome == PaneChrome::Drawer {
                bold(ink.primary, ink.chrome)
            } else {
                plain(ink.secondary, ink.chrome)
            };
            put(ui, area, content_x.saturating_add(2), y, title, title_style);
            put(
                ui,
                area,
                align.saturating_sub(width(meta)),
                y,
                meta,
                plain(ink.faint, ink.chrome),
            );
        }
        DrawerLine::Rule => {
            let from = content_x.saturating_add(2);
            put(
                ui,
                area,
                from,
                y,
                &"─".repeat(usize::from(align.saturating_sub(from))),
                plain(RULE, ink.chrome),
            );
        }
        DrawerLine::Field {
            label,
            value,
            ink: tone,
        } => {
            put(
                ui,
                area,
                content_x.saturating_add(2),
                y,
                label,
                plain(ink.muted, ink.chrome),
            );
            put(
                ui,
                area,
                content_x.saturating_add(VALUE_AT),
                y,
                value,
                plain(tone_color(*tone, ink), ink.chrome),
            );
        }
        DrawerLine::Value { text, ink: tone } => {
            put(
                ui,
                area,
                content_x.saturating_add(VALUE_AT),
                y,
                text,
                plain(tone_color(*tone, ink), ink.chrome),
            );
        }
        DrawerLine::Heading { label, meta } => {
            put(
                ui,
                area,
                content_x.saturating_add(2),
                y,
                label,
                bold(ink.secondary, ink.chrome),
            );
            put(
                ui,
                area,
                align.saturating_sub(width(meta)),
                y,
                meta,
                plain(ink.faint, ink.chrome),
            );
        }
        DrawerLine::Meter(meter) => paint_meter(ui, area, y, content_x, meter, ink),
        DrawerLine::Spans(line) => {
            let mut x = content_x;
            for span in line.spans {
                let style = span_style(span.ink, ink);
                put(ui, area, x, y, span.text, style);
                x = x.saturating_add(width(span.text));
            }
        }
        DrawerLine::Text(text) => put(ui, area, content_x, y, text, plain(ink.primary, ink.chrome)),
        DrawerLine::Buttons(buttons) => paint_buttons(ui, area, y, content_x, buttons, ink),
        DrawerLine::Blank => {}
    }
}

fn paint_meter(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    content_x: u16,
    meter: &DrawerMeter<'_>,
    ink: &Ink,
) {
    put(
        ui,
        area,
        content_x,
        y,
        meter.label,
        plain(ink.primary, ink.chrome),
    );
    let bar_x = content_x.saturating_add(LABEL_W);
    let fill = tone_color_tone(meter.tone);
    // `plain` records a modifier removal. A later add does not clear that
    // removal, so the bar would lose bold, italic, and the black ink.
    let style = PaintStyle::new()
        .fg(Color::Rgb(0, 0, 0))
        .bg(fill)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC);
    let mut shown = meter.readout.to_owned();
    let cols = usize::from(meter.fill_cols);
    if shown.chars().count() < cols {
        shown.push_str(&" ".repeat(cols - shown.chars().count()));
    }
    put(ui, area, bar_x, y, &shown, style);
    if meter.track_cols > 0 {
        let track = PaintStyle::new()
            .fg(ink.secondary)
            .bg(TRACK)
            .add_modifier(Modifier::ITALIC);
        put(
            ui,
            area,
            bar_x.saturating_add(meter.fill_cols),
            y,
            &" ".repeat(usize::from(meter.track_cols)),
            track,
        );
    }
    let mark_x = bar_x.saturating_add(BAR_W);
    let mark_style = if meter.mark_bold {
        bold(fill, ink.chrome)
    } else {
        plain(fill, ink.chrome)
    };
    put(ui, area, mark_x, y, meter.mark, mark_style);
    put(
        ui,
        area,
        mark_x.saturating_add(4),
        y,
        meter.note,
        plain(ink.muted, ink.chrome),
    );
}

fn paint_buttons(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    content_x: u16,
    buttons: &[DrawerButton<'_>],
    ink: &Ink,
) {
    let mut x = content_x.saturating_add(1);
    for button in buttons {
        put(ui, area, x, y, " ", plain(ink.face, ink.face));
        x = x.saturating_add(1);
        let fg = if button.danger {
            BUTTON_DANGER
        } else {
            ink.primary
        };
        put(ui, area, x, y, button.label, plain(fg, ink.face));
        x = x.saturating_add(width(button.label)).saturating_add(2);
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &AccountsDrawer<'_>, ink: &Ink) {
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

fn tone_color(tone: DrawerInk, ink: &Ink) -> Color {
    match tone {
        DrawerInk::Plain => ink.primary,
        DrawerInk::Muted => ink.muted,
        DrawerInk::Secondary => ink.secondary,
        DrawerInk::Warning => WARNING,
        DrawerInk::Danger => DANGER,
        DrawerInk::Info => INFO,
    }
}

fn tone_color_tone(tone: DrawerTone) -> Color {
    match tone {
        DrawerTone::Warning => WARNING,
        DrawerTone::Danger => DANGER,
        DrawerTone::Info => INFO,
    }
}

fn span_style(tone: DetailInk, ink: &Ink) -> PaintStyle {
    match tone {
        DetailInk::Plain => plain(ink.primary, ink.chrome),
        DetailInk::Muted => plain(ink.muted, ink.chrome),
        DetailInk::Danger => plain(DANGER, ink.chrome),
        DetailInk::DangerBold => bold(DANGER, ink.chrome),
        DetailInk::Warning => plain(WARNING, ink.chrome),
        DetailInk::Info => plain(INFO, ink.chrome),
    }
}
