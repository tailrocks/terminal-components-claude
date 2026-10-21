//! Zoomed session pane: menu, tabs, bordered transcript, status, and hints.
//!
//! Positions follow the labels and the screen edges.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct ZoomTab<'a> {
    pub index: &'a str,
    pub label: &'a str,
    pub live: bool,
    pub selected: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoomTone {
    Primary,
    Secondary,
    Muted,
    Faint,
    Warning,
    Danger,
    Ok,
}

pub struct ZoomSpan<'a> {
    pub text: &'a str,
    pub tone: ZoomTone,
    pub bold: bool,
    pub italic: bool,
}

pub struct ZoomHint<'a> {
    pub key: &'a str,
    pub action: &'a str,
}

pub struct ZoomedPane<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub tabs: &'a [ZoomTab<'a>],
    pub title: &'a str,
    pub title_mark: &'a str,
    pub zoom_label: &'a str,
    pub lines: &'a [&'a [ZoomSpan<'a>]],
    pub status: &'a [ZoomSpan<'a>],
    pub hints: &'a [ZoomHint<'a>],
    pub hint_overflow: &'a str,
    pub zoom_hint: &'a str,
}

struct Ink {
    canvas: Color,
    face: Color,
    elevated: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
    border: Color,
}

const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const OK: Color = Color::Rgb(0x3f, 0xb9, 0x50);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);

pub fn paint_zoomed_pane(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>) {
    if area.width < 40 || area.height < 12 {
        return;
    }
    let theme = ui.theme_ref();
    let ink = Ink {
        canvas: theme.bg(Surface::Canvas),
        face: theme.bg(Surface::Overlay),
        elevated: theme.bg(Surface::Elevated),
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
    let selected = paint_tabs(ui, area, page, &ink);
    paint_rule(ui, area, selected, &ink);
    paint_box(ui, area, page, &ink);
    paint_status(ui, area, page, &ink);
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

fn tone(ink: &Ink, which: ZoomTone, bold_text: bool, italic: bool) -> PaintStyle {
    let color = match which {
        ZoomTone::Primary => ink.primary,
        ZoomTone::Secondary => ink.secondary,
        ZoomTone::Muted => ink.muted,
        ZoomTone::Faint => ink.faint,
        ZoomTone::Warning => WARNING,
        ZoomTone::Danger => DANGER,
        ZoomTone::Ok => OK,
    };
    let mut style = if bold_text {
        bold(color, ink.canvas)
    } else {
        plain(color, ink.canvas)
    };
    if italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    style
}

fn put(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let room = area.right().saturating_sub(x);
    if room == 0 || y >= area.bottom() || text.is_empty() {
        return;
    }
    ui.paint_str(Rect::new(x, y, room, 1), text, style);
}

fn tab_text(tab: &ZoomTab<'_>) -> String {
    let mut text = format!(" {} {}", tab.index, tab.label);
    if tab.live {
        text.push_str(" ●");
    }
    text.push_str("  ");
    text
}

fn paint_header(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>, ink: &Ink) {
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
    for label in page.menus {
        let text = format!(" {label} ");
        put(ui, area, x, y, &text, plain(ink.secondary, ink.canvas));
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
    let crumb_end = area.right().saturating_sub(2);
    let crumb_x = crumb_end.saturating_sub(width(page.crumb));
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
        bold(ink.primary, ink.canvas),
    );
}

fn paint_tabs(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>, ink: &Ink) -> (u16, u16) {
    let y = area.y.saturating_add(2);
    let mut x = area.x.saturating_add(1);
    let mut selected = (x, 0);
    for tab in page.tabs {
        let text = tab_text(tab);
        let span = width(&text);
        if tab.selected {
            selected = (x, span);
            ui.fill(
                Rect {
                    x,
                    y,
                    width: span,
                    height: 1,
                },
                bold(ink.primary, ink.face),
            );
            let mut cursor = x.saturating_add(1);
            put(
                ui,
                area,
                cursor,
                y,
                tab.index,
                plain(ink.secondary, ink.face),
            );
            cursor = cursor.saturating_add(width(tab.index));
            let label = format!(" {} ", tab.label);
            put(ui, area, cursor, y, &label, bold(ink.primary, ink.face));
            cursor = cursor.saturating_add(width(&label));
            if tab.live {
                put(ui, area, cursor, y, "●", plain(ink.secondary, ink.face));
            }
            x = x.saturating_add(span).saturating_add(1);
        } else {
            put(ui, area, x, y, " ", plain(ink.secondary, ink.canvas));
            let mut cursor = x.saturating_add(1);
            put(ui, area, cursor, y, tab.index, plain(ink.muted, ink.canvas));
            cursor = cursor.saturating_add(width(tab.index));
            let label = if tab.live {
                format!(" {} ", tab.label)
            } else {
                format!(" {}", tab.label)
            };
            put(
                ui,
                area,
                cursor,
                y,
                &label,
                plain(ink.secondary, ink.canvas),
            );
            let mut pad_at = cursor.saturating_add(width(&label));
            if tab.live {
                put(ui, area, pad_at, y, "●", plain(ink.muted, ink.canvas));
                pad_at = pad_at.saturating_add(1);
            }
            put(ui, area, pad_at, y, "  ", plain(ink.secondary, ink.canvas));
            x = x.saturating_add(span).saturating_add(1);
        }
    }
    selected
}

fn paint_rule(ui: &mut Ui<'_>, area: Rect, selected: (u16, u16), ink: &Ink) {
    let y = area.y.saturating_add(3);
    let start = area.x.saturating_add(1);
    let end = area.right().saturating_sub(1);
    let (sel_x, sel_w) = selected;
    if sel_x > start {
        put(
            ui,
            area,
            start,
            y,
            &"─".repeat(usize::from(sel_x.saturating_sub(start))),
            plain(RULE, ink.canvas),
        );
    }
    put(
        ui,
        area,
        sel_x,
        y,
        &"━".repeat(usize::from(sel_w)),
        plain(ink.accent, ink.canvas),
    );
    let after = sel_x.saturating_add(sel_w);
    if end > after {
        put(
            ui,
            area,
            after,
            y,
            &"─".repeat(usize::from(end.saturating_sub(after))),
            plain(RULE, ink.canvas),
        );
    }
}

fn paint_box(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>, ink: &Ink) {
    let top = area.y.saturating_add(4);
    let bottom = area.bottom().saturating_sub(3);
    let right = area.right().saturating_sub(1);
    let edge = plain(ink.border, ink.canvas);
    put(ui, area, area.x, top, "╭─", edge);
    put(
        ui,
        area,
        area.x.saturating_add(2),
        top,
        page.title,
        bold(ink.primary, ink.canvas),
    );
    let mark_x = area.x.saturating_add(2).saturating_add(width(page.title));
    put(
        ui,
        area,
        mark_x,
        top,
        page.title_mark,
        plain(WARNING, ink.canvas),
    );
    let label_x = right
        .saturating_sub(1)
        .saturating_sub(width(page.zoom_label));
    let dash_from = mark_x.saturating_add(width(page.title_mark));
    if label_x > dash_from {
        put(
            ui,
            area,
            dash_from,
            top,
            &"─".repeat(usize::from(label_x.saturating_sub(dash_from))),
            edge,
        );
    }
    put(
        ui,
        area,
        label_x,
        top,
        page.zoom_label,
        plain(ink.faint, ink.canvas),
    );
    put(ui, area, right.saturating_sub(1), top, "─╮", edge);
    let mut y = top.saturating_add(1);
    for line in page.lines {
        if y >= bottom {
            break;
        }
        put(ui, area, area.x, y, "│", edge);
        put(ui, area, right, y, "│", edge);
        let mut x = area.x.saturating_add(1);
        for span in *line {
            put(
                ui,
                area,
                x,
                y,
                span.text,
                tone(ink, span.tone, span.bold, span.italic),
            );
            x = x.saturating_add(width(span.text));
        }
        y = y.saturating_add(1);
    }
    while y < bottom {
        put(ui, area, area.x, y, "│", edge);
        put(ui, area, right, y, "│", edge);
        y = y.saturating_add(1);
    }
    let dashes = "─".repeat(usize::from(area.width.saturating_sub(2)));
    put(ui, area, area.x, bottom, &format!("╰{dashes}╯"), edge);
}

fn paint_status(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(2);
    ui.fill(
        Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        },
        plain(ink.primary, ink.elevated),
    );
    let mut x = area.x;
    for span in page.status {
        let color = match span.tone {
            ZoomTone::Primary => ink.primary,
            ZoomTone::Secondary => ink.secondary,
            ZoomTone::Warning => WARNING,
            ZoomTone::Muted => ink.muted,
            ZoomTone::Danger => DANGER,
            ZoomTone::Ok => OK,
            // Unfilled meter segments use the rule grey, not faint text.
            ZoomTone::Faint => RULE,
        };
        let mut style = if span.bold {
            bold(color, ink.elevated)
        } else {
            plain(color, ink.elevated)
        };
        if span.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        put(ui, area, x, y, span.text, style);
        x = x.saturating_add(width(span.text));
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &ZoomedPane<'_>, ink: &Ink) {
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
    let note_x = area
        .right()
        .saturating_sub(1)
        .saturating_sub(width(page.zoom_hint));
    put(
        ui,
        area,
        note_x,
        y,
        page.zoom_hint,
        plain(ink.secondary, ink.canvas),
    );
}
