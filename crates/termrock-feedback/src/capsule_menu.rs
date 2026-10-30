//! Capsule tab menu over a split session.
//!
//! The menu is a popup. Pane lines are the clipped session text beside it.

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct MenuEntry<'a> {
    pub label: &'a str,
    pub shortcut: &'a str,
    pub danger: bool,
    pub selected: bool,
}

pub struct CapsuleMenu<'a> {
    pub title: &'a str,
    pub entries: &'a [MenuEntry<'a>],
    pub separator_after: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipTone {
    Primary,
    Secondary,
    Muted,
    Faint,
    Warning,
    Danger,
    Ok,
    Border,
    Seam,
    SeamHeavy,
    Rule,
    Dim,
    Faded,
    Shell,
    Soft,
}

pub struct ClipSpan<'a> {
    pub x: u16,
    pub text: &'a str,
    pub tone: ClipTone,
    pub bold: bool,
}

pub struct CapsuleMenuFrame<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub overflow: &'a str,
    pub tabs: &'a [crate::zoom_pane::ZoomTab<'a>],
    pub menu: CapsuleMenu<'a>,
    pub rows: &'a [&'a [ClipSpan<'a>]],
    pub status: &'a [crate::zoom_pane::ZoomSpan<'a>],
    pub hints: &'a [crate::zoom_pane::ZoomHint<'a>],
    pub hint_overflow: &'a str,
    pub attached: &'a str,
}

const MENU_BG: Color = Color::Rgb(0x3f, 0x3f, 0x46);
const MENU_EDGE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const SELECT_BG: Color = Color::Rgb(0x2f, 0x5a, 0xa8);
const CLOSE: Color = Color::Rgb(0xe8, 0xa0, 0xa0);
const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const OK: Color = Color::Rgb(0x3f, 0xb9, 0x50);
const DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const SEAM: Color = Color::Rgb(0x47, 0x47, 0x47);
const SEAM_HEAVY: Color = Color::Rgb(0x80, 0x80, 0x80);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const DIM: Color = Color::Rgb(0x62, 0x62, 0x62);
const FADED: Color = Color::Rgb(0x8c, 0x8c, 0x8c);
const SHELL_FG: Color = Color::Rgb(0x4c, 0x4c, 0x4c);
const SOFT: Color = Color::Rgb(0xcc, 0xcc, 0xcc);

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

pub fn paint_capsule_menu(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>) {
    if area.width < 40 || area.height < 16 {
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
    paint_header(ui, area, frame, &ink);
    paint_tabs(ui, area, frame, &ink);
    let top = area.y.saturating_add(3);
    for (index, row) in frame.rows.iter().enumerate() {
        let y = top.saturating_add(index as u16);
        for span in *row {
            put(
                ui,
                area,
                area.x.saturating_add(span.x),
                y,
                span.text,
                clip_style(&ink, span),
            );
        }
    }
    paint_menu(ui, area, frame, &ink);
    paint_status(ui, area, frame, &ink);
    paint_hints(ui, area, frame, &ink);
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
    // `paint_str` restarts at a clipped rect, which shifts popup overwrites.
    // One cell at its absolute column keeps the page and the layer aligned.
    if y >= area.bottom() || text.is_empty() {
        return;
    }
    let mut cursor = x;
    for ch in text.chars() {
        if cursor >= area.right() {
            break;
        }
        let mut buf = [0u8; 4];
        let symbol = ch.encode_utf8(&mut buf);
        let cols = width(symbol);
        if cols == 0 {
            continue;
        }
        ui.paint_cell(Position::new(cursor, y), symbol, style);
        cursor = cursor.saturating_add(cols);
    }
}

fn clip_style(ink: &Ink, span: &ClipSpan<'_>) -> PaintStyle {
    let color = match span.tone {
        ClipTone::Primary => ink.primary,
        ClipTone::Secondary => ink.secondary,
        ClipTone::Muted => ink.muted,
        ClipTone::Faint => ink.faint,
        ClipTone::Warning => WARNING,
        ClipTone::Danger => DANGER,
        ClipTone::Ok => OK,
        ClipTone::Border => ink.border,
        ClipTone::Seam => SEAM,
        ClipTone::SeamHeavy => SEAM_HEAVY,
        ClipTone::Rule => RULE,
        ClipTone::Dim => DIM,
        ClipTone::Faded => FADED,
        ClipTone::Shell => SHELL_FG,
        ClipTone::Soft => SOFT,
    };
    if span.bold {
        bold(color, ink.canvas)
    } else {
        plain(color, ink.canvas)
    }
}

fn paint_header(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>, ink: &Ink) {
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
    for label in frame.menus {
        let text = format!(" {label} ");
        put(ui, area, x, y, &text, plain(ink.secondary, ink.canvas));
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
    let mark_x = area.right().saturating_sub(3);
    put(
        ui,
        area,
        mark_x,
        y,
        frame.overflow,
        plain(ink.muted, ink.canvas),
    );
}

fn tab_span(tab: &crate::zoom_pane::ZoomTab<'_>) -> String {
    let mut text = format!(" {} {}", tab.index, tab.label);
    if tab.live {
        text.push_str(" ●");
    }
    text.push_str("  ");
    text
}

fn paint_tabs(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>, ink: &Ink) {
    let y = area.y.saturating_add(2);
    let mut x = area.x.saturating_add(1);
    for tab in frame.tabs {
        let span = width(&tab_span(tab));
        if tab.selected {
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
        }
        x = x.saturating_add(span).saturating_add(1);
    }
}

const MENU_W: u16 = 29;
const TITLE_LEAD: u16 = 2;
const TITLE_BOLD_TAIL: u16 = 11;

fn paint_menu(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>, ink: &Ink) {
    let x = area.x.saturating_add(1);
    let mut row = area.y.saturating_add(3);
    let face = plain(MENU_EDGE, MENU_BG);
    put(
        ui,
        area,
        x,
        row,
        &format!("╭{}╮", "─".repeat(usize::from(MENU_W.saturating_sub(2)))),
        face,
    );
    row = row.saturating_add(1);
    paint_title_row(ui, area, x, row, frame.menu.title, ink);
    row = row.saturating_add(1);
    for (index, entry) in frame.menu.entries.iter().enumerate() {
        if index == frame.menu.separator_after {
            paint_separator_row(ui, area, x, row, ink);
            row = row.saturating_add(1);
        }
        paint_entry_row(ui, area, x, row, entry, ink);
        row = row.saturating_add(1);
    }
    put(
        ui,
        area,
        x,
        row,
        &format!("╰{}╯", "─".repeat(usize::from(MENU_W.saturating_sub(2)))),
        face,
    );
}

fn paint_title_row(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, title: &str, ink: &Ink) {
    let face = plain(MENU_EDGE, MENU_BG);
    put(ui, area, x, y, "│", face);
    put(
        ui,
        area,
        x.saturating_add(MENU_W).saturating_sub(1),
        y,
        "│",
        face,
    );
    let inner = x.saturating_add(1);
    let inner_w = MENU_W.saturating_sub(2);
    let bold_w = TITLE_LEAD
        .saturating_add(width(title))
        .saturating_add(TITLE_BOLD_TAIL)
        .min(inner_w);
    ui.fill(
        Rect {
            x: inner,
            y,
            width: bold_w,
            height: 1,
        },
        bold(ink.primary, MENU_BG),
    );
    put(
        ui,
        area,
        inner.saturating_add(TITLE_LEAD),
        y,
        title,
        bold(ink.muted, MENU_BG),
    );
    let warn_x = inner.saturating_add(bold_w);
    put(ui, area, warn_x, y, " ", plain(WARNING, MENU_BG));
    let border_x = warn_x.saturating_add(1);
    let border_w = inner.saturating_add(inner_w).saturating_sub(border_x);
    if border_w > 0 {
        put(
            ui,
            area,
            border_x,
            y,
            &" ".repeat(usize::from(border_w)),
            plain(ink.border, MENU_BG),
        );
    }
}

fn paint_separator_row(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, ink: &Ink) {
    let face = plain(MENU_EDGE, MENU_BG);
    put(ui, area, x, y, "│", face);
    put(
        ui,
        area,
        x.saturating_add(MENU_W).saturating_sub(1),
        y,
        "│",
        face,
    );
    let inner = x.saturating_add(1);
    let inner_w = MENU_W.saturating_sub(2);
    put(ui, area, inner, y, " ", plain(OK, MENU_BG));
    let rule_w = inner_w.saturating_sub(2);
    put(
        ui,
        area,
        inner.saturating_add(1),
        y,
        &"─".repeat(usize::from(rule_w)),
        plain(ink.border, MENU_BG),
    );
    put(
        ui,
        area,
        inner.saturating_add(inner_w).saturating_sub(1),
        y,
        " ",
        plain(ink.secondary, MENU_BG),
    );
}

fn paint_entry_row(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, entry: &MenuEntry<'_>, ink: &Ink) {
    let face = plain(MENU_EDGE, MENU_BG);
    put(ui, area, x, y, "│", face);
    put(
        ui,
        area,
        x.saturating_add(MENU_W).saturating_sub(1),
        y,
        "│",
        face,
    );
    let bg = if entry.selected { SELECT_BG } else { MENU_BG };
    let label_fg = if entry.danger { CLOSE } else { ink.primary };
    let body = if entry.selected {
        bold(label_fg, bg)
    } else {
        plain(label_fg, bg)
    };
    ui.fill(
        Rect {
            x: x.saturating_add(1),
            y,
            width: MENU_W.saturating_sub(2),
            height: 1,
        },
        body,
    );
    put(
        ui,
        area,
        x.saturating_add(1),
        y,
        &format!("  {}", entry.label),
        body,
    );
    let shortcut_x = x
        .saturating_add(MENU_W)
        .saturating_sub(2)
        .saturating_sub(width(entry.shortcut));
    let shortcut_fg = if entry.selected {
        ink.primary
    } else {
        ink.secondary
    };
    put(
        ui,
        area,
        shortcut_x,
        y,
        entry.shortcut,
        plain(shortcut_fg, bg),
    );
}

fn paint_status(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>, ink: &Ink) {
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
    for span in frame.status {
        let color = match span.tone {
            crate::zoom_pane::ZoomTone::Primary => ink.primary,
            crate::zoom_pane::ZoomTone::Muted => ink.muted,
            crate::zoom_pane::ZoomTone::Warning => WARNING,
            crate::zoom_pane::ZoomTone::Faint => RULE,
            crate::zoom_pane::ZoomTone::Secondary => ink.secondary,
            crate::zoom_pane::ZoomTone::Danger => DANGER,
            crate::zoom_pane::ZoomTone::Ok => OK,
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

fn paint_hints(ui: &mut Ui<'_>, area: Rect, frame: &CapsuleMenuFrame<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    for hint in frame.hints {
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
        frame.hint_overflow,
        plain(ink.faint, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(24),
        y,
        frame.attached,
        plain(ink.secondary, ink.canvas),
    );
}
