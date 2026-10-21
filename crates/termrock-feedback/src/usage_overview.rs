//! Narrow usage overview: menu, framed account tree, scrollbar, hints.
//!
//! Width under 100 keeps the list and drops the detail column. Positions
//! follow the labels, the chrome inset, and the screen edges.

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const DANGER: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const WARNING: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const THUMB: Color = Color::Rgb(0x80, 0x80, 0x80);
const TRACK: Color = Color::Rgb(0x47, 0x47, 0x47);
const EDGE: u16 = 2;
const OUTER_KEEP: f32 = 0.55;
const INNER_KEEP: f32 = 0.8;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsageMark {
    None,
    Danger,
    Warning,
}

pub struct UsageLine {
    pub heading: bool,
    /// Overview text starts three columns in. Account names start at five.
    pub overview: bool,
    pub label: String,
    pub muted: bool,
    pub mark: UsageMark,
    pub meta: String,
    pub selected: bool,
}

pub struct UsageHint<'a> {
    pub key: &'a str,
    pub action: &'a str,
}

pub struct UsageTrail<'a> {
    pub mark: Option<&'a str>,
    pub text: &'a str,
    pub muted: bool,
}

pub struct UsageOverview<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub note: &'a str,
    pub trail: Option<UsageTrail<'a>>,
    pub title: &'a str,
    pub status: &'a str,
    pub rows: &'a [UsageLine],
    pub hints: &'a [UsageHint<'a>],
}

struct Ink {
    canvas: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
    border: Color,
}

struct Seg {
    text: String,
    fg: Color,
    mark: Option<(String, Color)>,
    priority: u8,
}

pub fn paint_usage_overview(ui: &mut Ui<'_>, area: Rect, page: &UsageOverview<'_>) {
    if area.width >= 100 || area.width < 40 || area.height < 16 {
        return;
    }
    let theme = ui.theme_ref();
    let ink = Ink {
        canvas: theme.bg(Surface::Canvas),
        primary: step(theme, FgStep::Primary),
        secondary: step(theme, FgStep::Secondary),
        muted: step(theme, FgStep::Muted),
        faint: step(theme, FgStep::Faint),
        accent: theme.color.accent,
        on_accent: theme.color.on_accent,
        border: theme.color.border_strong,
    };
    ui.fill(area, plain(ink.primary, ink.canvas));
    let used = paint_menus(ui, area, page, &ink);
    paint_identity(ui, area, used, page, &ink);
    let panel = panel_rect(area);
    paint_frame(ui, panel, page, &ink);
    let list = list_rect(panel);
    paint_rows(ui, list, page.rows, &ink);
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
    if room == 0 || y < area.y || y >= area.bottom() || text.is_empty() {
        return;
    }
    ui.paint_str(Rect::new(x, y, room, 1), text, style);
}

fn paint_menus(ui: &mut Ui<'_>, area: Rect, page: &UsageOverview<'_>, ink: &Ink) -> u16 {
    let y = area.y;
    let brand = format!(" {} ", page.brand);
    let brand_x = area.x.saturating_add(1);
    put(
        ui,
        area,
        brand_x,
        y,
        &brand,
        bold(ink.on_accent, ink.accent),
    );
    let mut x = brand_x.saturating_add(width(&brand)).saturating_add(2);
    let menu = plain(ink.secondary, ink.canvas);
    let mut used = brand_x.saturating_add(width(&brand));
    for label in page.menus {
        let text = format!(" {label} ");
        put(ui, area, x, y, &text, menu);
        x = x.saturating_add(width(&text));
        used = x;
        x = x.saturating_add(1);
    }
    used
}

fn paint_identity(ui: &mut Ui<'_>, area: Rect, used: u16, page: &UsageOverview<'_>, ink: &Ink) {
    let rest_x = used.saturating_add(2);
    if rest_x >= area.right() {
        return;
    }
    let mut items = vec![
        Seg {
            text: page.crumb.to_owned(),
            fg: ink.secondary,
            mark: None,
            priority: 7,
        },
        Seg {
            text: page.note.to_owned(),
            fg: ink.secondary,
            mark: None,
            priority: 6,
        },
    ];
    if let Some(ref trail) = page.trail {
        items.push(Seg {
            text: trail.text.to_owned(),
            fg: if trail.muted {
                ink.muted
            } else {
                ink.secondary
            },
            mark: trail.mark.map(|mark| (mark.to_owned(), ink.accent)),
            priority: 5,
        });
    }
    let rest = Rect::new(rest_x, area.y, area.right().saturating_sub(rest_x), 1);
    paint_segments(ui, rest, &items, ink);
}

fn shown(seg: &Seg) -> String {
    match &seg.mark {
        Some((glyph, _)) => format!("{glyph} {}", seg.text),
        None => seg.text.clone(),
    }
}

fn segment_total(items: &[Seg], keep: &[bool], width_limit: u16) -> u16 {
    let mut sum = 0u16;
    let mut any = false;
    let mut dropped = false;
    for (seg, kept) in items.iter().zip(keep.iter()) {
        if *kept {
            any = true;
            sum = sum.saturating_add(width(&shown(seg)).saturating_add(2));
        } else {
            dropped = true;
        }
    }
    if !any {
        return width_limit.saturating_add(1);
    }
    let more = if dropped { 3 } else { 0 };
    sum.saturating_add(more)
        .saturating_sub(2)
        .saturating_add(EDGE.saturating_mul(2))
}

fn paint_segments(ui: &mut Ui<'_>, area: Rect, items: &[Seg], ink: &Ink) {
    let mut keep = vec![true; items.len()];
    loop {
        if segment_total(items, &keep, area.width) <= area.width {
            break;
        }
        let mut best: Option<(u8, usize)> = None;
        for (index, seg) in items.iter().enumerate() {
            if !keep.get(index).copied().unwrap_or(false) {
                continue;
            }
            if best.is_none_or(|(priority, _)| seg.priority < priority) {
                best = Some((seg.priority, index));
            }
        }
        let Some((_, index)) = best else {
            break;
        };
        if let Some(slot) = keep.get_mut(index) {
            *slot = false;
        }
    }
    let mut rx = area.right().saturating_sub(EDGE);
    for (seg, kept) in items.iter().zip(keep.iter()).rev() {
        if !kept {
            continue;
        }
        let text = shown(seg);
        let columns = width(&text);
        rx = rx.saturating_sub(columns);
        put(ui, area, rx, area.y, &text, plain(seg.fg, ink.canvas));
        if let Some((glyph, color)) = &seg.mark {
            put(ui, area, rx, area.y, glyph, plain(*color, ink.canvas));
        }
        rx = rx.saturating_sub(2);
    }
    if keep.iter().any(|kept| !kept) && area.width > EDGE.saturating_mul(2) {
        let at = rx.saturating_sub(1).max(area.x.saturating_add(EDGE));
        put(ui, area, at, area.y, "…", plain(ink.muted, ink.canvas));
    }
}

fn panel_rect(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(2),
        area.width.saturating_sub(2),
        area.height.saturating_sub(4),
    )
}

fn list_rect(panel: Rect) -> Rect {
    let inner_w = panel.width.saturating_sub(2);
    Rect::new(
        panel.x.saturating_add(3),
        panel.y.saturating_add(1),
        inner_w.saturating_sub(3),
        panel.height.saturating_sub(2),
    )
}

fn paint_frame(ui: &mut Ui<'_>, panel: Rect, page: &UsageOverview<'_>, ink: &Ink) {
    if panel.width < 6 || panel.height < 3 {
        return;
    }
    let border = plain(ink.border, ink.canvas);
    let dashes = "─".repeat(usize::from(panel.width.saturating_sub(2)));
    put(ui, panel, panel.x, panel.y, &format!("╭{dashes}╮"), border);
    put(
        ui,
        panel,
        panel.x,
        panel.bottom().saturating_sub(1),
        &format!("╰{dashes}╯"),
        border,
    );
    for y in panel.y.saturating_add(1)..panel.bottom().saturating_sub(1) {
        put(ui, panel, panel.x, y, "│", border);
        put(ui, panel, panel.right().saturating_sub(1), y, "│", border);
    }
    let title = format!(" {} ", page.title);
    put(
        ui,
        panel,
        panel.x.saturating_add(2),
        panel.y,
        &title,
        bold(ink.primary, ink.canvas),
    );
    if !page.status.is_empty() {
        let meta = format!(" {} ", page.status);
        let span = panel.width.saturating_sub(4);
        let meta_x = panel
            .x
            .saturating_add(2)
            .saturating_add(span)
            .saturating_sub(width(&meta));
        put(
            ui,
            panel,
            meta_x,
            panel.y,
            &meta,
            plain(ink.faint, ink.canvas),
        );
    }
}

fn paint_rows(ui: &mut Ui<'_>, list: Rect, rows: &[UsageLine], ink: &Ink) {
    if list.is_empty() {
        return;
    }
    let content = u16::try_from(rows.len()).unwrap_or(u16::MAX);
    let viewport = list.height;
    let bar = content > viewport && list.width >= 2;
    let row_w = list.width.saturating_sub(u16::from(bar));
    let shown = usize::from(viewport).min(rows.len());
    for (index, row) in rows.iter().take(shown).enumerate() {
        let y = list.y.saturating_add(u16::try_from(index).unwrap_or(0));
        if row.heading {
            put(
                ui,
                list,
                list.x.saturating_add(2),
                y,
                &row.label,
                plain(ink.muted, ink.canvas),
            );
            continue;
        }
        paint_account(ui, list, y, row_w, row, ink);
    }
    if bar {
        fade_tail(ui, list, row_w, viewport, content, ink.canvas);
        paint_bar(ui, list, viewport, content, ink.canvas);
    }
}

fn paint_account(ui: &mut Ui<'_>, list: Rect, y: u16, row_w: u16, row: &UsageLine, ink: &Ink) {
    let row_area = Rect::new(list.x, y, row_w, 1);
    let (fg, bg, face) = if row.selected {
        (ink.primary, SELECT_BG, bold(ink.primary, SELECT_BG))
    } else {
        (ink.primary, ink.canvas, plain(ink.primary, ink.canvas))
    };
    ui.fill(row_area, face);
    if row.selected {
        put(ui, list, list.x, y, "▎", bold(ink.accent, SELECT_BG));
        put(
            ui,
            list,
            list.x.saturating_add(1),
            y,
            "›",
            bold(ink.accent, SELECT_BG),
        );
    } else {
        put(ui, list, list.x, y, " ", plain(ink.canvas, ink.canvas));
    }
    let mark_cols: u16 = if row.mark == UsageMark::None { 0 } else { 2 };
    let meta_cols = width(&row.meta);
    let reserve = meta_cols.saturating_add(mark_cols);
    let text_at: u16 = if row.overview { 3 } else { 5 };
    let label_room = row_w
        .saturating_sub(text_at)
        .saturating_sub(reserve)
        .saturating_sub(2);
    let label = crate::text::truncate(&row.label, label_room);
    let label_fg = if row.muted { ink.muted } else { fg };
    let label_style = if row.selected {
        bold(label_fg, bg)
    } else {
        plain(label_fg, bg)
    };
    put(
        ui,
        list,
        list.x.saturating_add(text_at),
        y,
        &label,
        label_style,
    );
    let mut rx = list.x.saturating_add(row_w).saturating_sub(1);
    if !row.meta.is_empty() {
        rx = rx.saturating_sub(meta_cols);
        put(ui, list, rx, y, &row.meta, plain(ink.muted, bg));
        rx = rx.saturating_sub(1);
    }
    if row.mark != UsageMark::None {
        rx = rx.saturating_sub(1);
        let color = if row.mark == UsageMark::Danger {
            DANGER
        } else {
            WARNING
        };
        let glyph = if row.mark == UsageMark::Danger {
            "!"
        } else {
            "▲"
        };
        put(ui, list, rx, y, glyph, plain(color, bg));
    }
}

fn fade_tail(ui: &mut Ui<'_>, list: Rect, row_w: u16, viewport: u16, content: u16, canvas: Color) {
    if viewport < 4 || content <= viewport {
        return;
    }
    let depth: u16 = if viewport >= 12 { 2 } else { 1 };
    let bottom = list.y.saturating_add(viewport);
    fade_row(
        ui,
        list.x,
        row_w,
        bottom.saturating_sub(1),
        OUTER_KEEP,
        canvas,
    );
    if depth == 2 {
        fade_row(
            ui,
            list.x,
            row_w,
            bottom.saturating_sub(2),
            INNER_KEEP,
            canvas,
        );
    }
}

fn fade_row(ui: &mut Ui<'_>, x0: u16, row_w: u16, y: u16, keep: f32, canvas: Color) {
    let (buf, _) = ui.raw();
    let right = x0.saturating_add(row_w);
    for x in x0..right {
        let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
            continue;
        };
        if cell.bg != canvas || cell.modifier.contains(Modifier::REVERSED) {
            continue;
        }
        if let Color::Rgb(red, green, blue) = cell.fg {
            cell.fg = Color::Rgb(mix(red, 0, keep), mix(green, 0, keep), mix(blue, 0, keep));
        } else if keep <= OUTER_KEEP {
            cell.modifier.insert(Modifier::DIM);
        }
    }
}

fn mix(fg: u8, bg: u8, keep: f32) -> u8 {
    (bg as f32 + (fg as f32 - bg as f32) * keep)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn paint_bar(ui: &mut Ui<'_>, list: Rect, viewport: u16, content: u16, canvas: Color) {
    let len = u32::from(viewport)
        .saturating_mul(u32::from(viewport))
        .checked_div(u32::from(content.max(1)))
        .unwrap_or(1)
        .max(1)
        .min(u32::from(viewport));
    let len = u16::try_from(len).unwrap_or(viewport);
    let x = list.right().saturating_sub(1);
    for index in 0..viewport {
        let glyph = if index < len { "┃" } else { "┆" };
        let color = if index < len { THUMB } else { TRACK };
        put(
            ui,
            list,
            x,
            list.y.saturating_add(index),
            glyph,
            plain(color, canvas),
        );
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &UsageOverview<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    for hint in page.hints {
        put(ui, area, x, y, hint.key, bold(ink.primary, ink.canvas));
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, plain(ink.muted, ink.canvas));
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
}
