//! Settings page: menu, tabs, checkbox sections, actions, and hints.
//!
//! Positions follow the labels and the screen edges. The selected tab
//! carries the focus gutter and the heavy underline.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct SettingsTab<'a> {
    pub label: &'a str,
    pub selected: bool,
}

pub struct SettingsCheck<'a> {
    pub label: &'a str,
    pub checked: bool,
}

pub struct SettingsSection<'a> {
    pub title: &'a str,
    pub checks: &'a [SettingsCheck<'a>],
    pub note: Option<&'a str>,
    pub body: Option<&'a str>,
}

pub struct SettingsHint<'a> {
    pub key: &'a str,
    pub action: &'a str,
}

pub struct SettingsMount<'a> {
    pub path: &'a str,
    pub scope: &'a str,
    pub mode: &'a str,
    pub isolation: &'a str,
    pub selected: bool,
}

#[derive(Clone, Copy)]
pub struct SettingsCluster<'a> {
    pub note: &'a str,
    pub mark: &'a str,
    pub value: &'a str,
}

pub struct SettingsPage<'a> {
    pub brand: &'a str,
    pub menus: &'a [&'a str],
    pub crumb: &'a str,
    pub tabs: &'a [SettingsTab<'a>],
    pub sections: &'a [SettingsSection<'a>],
    pub cancel: &'a str,
    pub save: &'a str,
    pub save_enabled: bool,
    pub hints: &'a [SettingsHint<'a>],
    pub hint_overflow: Option<&'a str>,
    pub mounts: &'a [SettingsMount<'a>],
    pub add_mount: Option<&'a str>,
    /// Wide header: crumb, note, mark, and value, packed from the right edge.
    /// Absent on the narrow page, which keeps the ellipsis before the crumb.
    pub cluster: Option<SettingsCluster<'a>>,
}

struct Ink {
    canvas: Color,
    face: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
    border: Color,
}

pub fn paint_settings_page(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>) {
    if area.width < 40 || area.height < 8 {
        return;
    }
    let ink = {
        let theme = ui.theme_ref();
        Ink {
            canvas: theme.bg(Surface::Canvas),
            face: theme.bg(Surface::Overlay),
            primary: step(theme, FgStep::Primary),
            secondary: step(theme, FgStep::Secondary),
            muted: step(theme, FgStep::Muted),
            faint: step(theme, FgStep::Faint),
            accent: theme.color.accent,
            on_accent: theme.color.on_accent,
            border: theme.color.border_strong,
        }
    };
    ui.fill(area, plain(ink.primary, ink.canvas));
    paint_header(ui, area, page, &ink);
    let (selected_at, selected_span) = paint_tabs(ui, area, page, &ink);
    paint_underline(ui, area, selected_at, selected_span, &ink);
    paint_sections(ui, area, page, &ink);
    paint_mounts(ui, area, page, &ink);
    paint_actions(ui, area, page, &ink);
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

fn paint_header(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
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
    if let Some(cluster) = page.cluster {
        let value_x = area
            .right()
            .saturating_sub(2)
            .saturating_sub(width(cluster.value));
        let mark_x = value_x
            .saturating_sub(1)
            .saturating_sub(width(cluster.mark));
        let note_x = mark_x.saturating_sub(2).saturating_sub(width(cluster.note));
        let crumb_x = note_x.saturating_sub(2).saturating_sub(width(page.crumb));
        put(ui, area, crumb_x, y, page.crumb, secondary);
        put(ui, area, note_x, y, cluster.note, secondary);
        put(
            ui,
            area,
            mark_x,
            y,
            cluster.mark,
            plain(ink.accent, ink.canvas),
        );
        // The gap keeps the value ink. The mark is recolored on top of that run.
        put(
            ui,
            area,
            mark_x.saturating_add(width(cluster.mark)),
            y,
            " ",
            secondary,
        );
        put(ui, area, value_x, y, cluster.value, secondary);
        return;
    }
    let crumb_w = width(page.crumb);
    let crumb_x = area.right().saturating_sub(2).saturating_sub(crumb_w);
    put(
        ui,
        area,
        crumb_x.saturating_sub(3),
        y,
        "…",
        plain(ink.muted, ink.canvas),
    );
    put(ui, area, crumb_x, y, page.crumb, secondary);
}

fn tab_span(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_tabs(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) -> (u16, u16) {
    let y = area.y.saturating_add(3);
    let origin = area.x.saturating_add(2);
    let mut x = origin;
    let mut selected_at = origin;
    let mut selected_span = 0;
    for tab in page.tabs {
        let span = tab_span(tab.label);
        let idle = format!(" {}  ", tab.label);
        if tab.selected {
            selected_at = x;
            selected_span = span;
            let face = bold(ink.primary, ink.face);
            if x == origin {
                put(ui, area, x, y, "▎", bold(ink.accent, ink.face));
                put(
                    ui,
                    area,
                    x.saturating_add(1),
                    y,
                    &format!("{}  ", tab.label),
                    face,
                );
            } else {
                put(ui, area, x, y, &idle, face);
            }
        } else {
            put(ui, area, x, y, &idle, plain(ink.secondary, ink.canvas));
        }
        x = x.saturating_add(span).saturating_add(1);
    }
    (selected_at, selected_span)
}

fn paint_underline(ui: &mut Ui<'_>, area: Rect, selected_at: u16, selected_span: u16, ink: &Ink) {
    let y = area.y.saturating_add(4);
    let start = area.x.saturating_add(2);
    let rule = plain(Color::Rgb(0x3a, 0x3a, 0x3a), ink.canvas);
    let heavy = plain(ink.accent, ink.canvas);
    if selected_at > start {
        put(
            ui,
            area,
            start,
            y,
            &"─".repeat(usize::from(selected_at.saturating_sub(start))),
            rule,
        );
    }
    if selected_span > 0 {
        put(
            ui,
            area,
            selected_at,
            y,
            &"━".repeat(usize::from(selected_span)),
            heavy,
        );
    }
    let after = selected_at.saturating_add(selected_span);
    let rule_end = area.right().saturating_sub(2);
    if rule_end > after {
        put(
            ui,
            area,
            after,
            y,
            &"─".repeat(usize::from(rule_end.saturating_sub(after))),
            rule,
        );
    }
}

fn paint_sections(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let mut y = area.y.saturating_add(6);
    let title = bold(ink.secondary, ink.canvas);
    let label = plain(ink.primary, ink.canvas);
    let mark = plain(ink.accent, ink.canvas);
    let note = plain(ink.faint, ink.canvas);
    let body = plain(ink.muted, ink.canvas);
    for section in page.sections {
        put(ui, area, area.x.saturating_add(4), y, section.title, title);
        y = y.saturating_add(1);
        for check in section.checks {
            let box_mark = if check.checked { "[✓]" } else { "[ ]" };
            put(
                ui,
                area,
                area.x.saturating_add(4),
                y,
                " ",
                plain(ink.canvas, ink.canvas),
            );
            put(ui, area, area.x.saturating_add(5), y, box_mark, mark);
            put(
                ui,
                area,
                area.x.saturating_add(8),
                y,
                &format!(" {}", check.label),
                label,
            );
            y = y.saturating_add(1);
        }
        if let Some(text) = section.note {
            y = y.saturating_add(1);
            put(ui, area, area.x.saturating_add(4), y, text, note);
            y = y.saturating_add(1);
        }
        if let Some(text) = section.body {
            put(ui, area, area.x.saturating_add(4), y, text, body);
            y = y.saturating_add(1);
        }
        y = y.saturating_add(1);
    }
}

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const PATH_AT: u16 = 4;
const PATH_W: u16 = 42;
const SCOPE_AT: u16 = 46;
const SCOPE_W: u16 = 14;
const MODE_AT: u16 = 62;
const MODE_W: u16 = 4;
const ISO_AT: u16 = 68;
const ISO_W: u16 = 9;

fn fit(text: &str, cols: u16) -> String {
    let have = width(text);
    if have >= cols {
        return text.to_owned();
    }
    let mut out = text.to_owned();
    out.push_str(&" ".repeat(usize::from(cols - have)));
    out
}

fn path_cell(path: &str) -> String {
    fit(&format!("    {path}"), PATH_W)
}

fn paint_mounts(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    if page.mounts.is_empty() {
        return;
    }
    let y0 = area.y.saturating_add(6);
    let head = plain(ink.muted, ink.canvas);
    put(
        ui,
        area,
        area.x.saturating_add(PATH_AT + 4),
        y0,
        "Destination",
        head,
    );
    put(ui, area, area.x.saturating_add(SCOPE_AT), y0, "Scope", head);
    put(ui, area, area.x.saturating_add(MODE_AT), y0, "Mode", head);
    // The header cell is only 8 columns. The scroll mark owns the rest.
    put(
        ui,
        Rect {
            x: area.x.saturating_add(ISO_AT),
            y: y0,
            width: 8,
            height: 1,
        },
        area.x.saturating_add(ISO_AT),
        y0,
        "Isolation",
        head,
    );
    put(
        ui,
        area,
        area.x.saturating_add(76),
        y0,
        "2›",
        plain(ink.faint, ink.canvas),
    );
    for (i, row) in page.mounts.iter().enumerate() {
        let y = y0.saturating_add(1).saturating_add(i as u16);
        paint_mount_row(ui, area, y, row, ink);
    }
    if let Some(add) = page.add_mount {
        let y = y0
            .saturating_add(1)
            .saturating_add(page.mounts.len() as u16);
        put(
            ui,
            area,
            area.x.saturating_add(2),
            y,
            " ",
            plain(ink.canvas, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(3),
            y,
            " ",
            plain(ink.secondary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(8),
            y,
            add,
            plain(ink.secondary, ink.canvas),
        );
    }
}

fn paint_mount_row(ui: &mut Ui<'_>, area: Rect, y: u16, row: &SettingsMount<'_>, ink: &Ink) {
    let path = path_cell(row.path);
    let scope = fit(row.scope, SCOPE_W);
    let mode = fit(row.mode, MODE_W);
    let isolation = fit(row.isolation, ISO_W);
    if row.selected {
        ui.fill(
            Rect {
                x: area.x.saturating_add(2),
                y,
                width: ISO_AT + ISO_W - 1,
                height: 1,
            },
            bold(ink.primary, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(2),
            y,
            "▎›",
            bold(ink.accent, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(PATH_AT),
            y,
            &path,
            bold(ink.primary, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(SCOPE_AT),
            y,
            row.scope,
            bold(ink.secondary, SELECT_BG),
        );
        let pad = SCOPE_W.saturating_sub(width(row.scope));
        if pad > 0 {
            put(
                ui,
                area,
                area.x
                    .saturating_add(SCOPE_AT)
                    .saturating_add(width(row.scope)),
                y,
                &" ".repeat(usize::from(pad)),
                bold(ink.muted, SELECT_BG),
            );
        }
        put(
            ui,
            area,
            area.x.saturating_add(MODE_AT),
            y,
            &mode,
            bold(ink.secondary, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(ISO_AT),
            y,
            &isolation,
            bold(ink.secondary, SELECT_BG),
        );
    } else {
        put(
            ui,
            area,
            area.x.saturating_add(2),
            y,
            " ",
            plain(ink.canvas, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(3),
            y,
            " ",
            plain(ink.secondary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(PATH_AT),
            y,
            &path,
            plain(ink.primary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(SCOPE_AT),
            y,
            &scope,
            plain(ink.muted, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(MODE_AT),
            y,
            &mode,
            plain(ink.secondary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(ISO_AT),
            y,
            &isolation,
            plain(ink.secondary, ink.canvas),
        );
    }
}

fn paint_actions(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(3);
    let save = format!("{} ", page.save);
    let cancel = format!("{} ", page.cancel);
    let save_x = area.right().saturating_sub(5).saturating_sub(width(&save));
    let save_ink = if page.save_enabled {
        ink.primary
    } else {
        ink.border
    };
    put(
        ui,
        area,
        save_x.saturating_sub(1),
        y,
        " ",
        plain(ink.face, ink.face),
    );
    put(ui, area, save_x, y, &save, plain(save_ink, ink.face));
    let cancel_x = save_x.saturating_sub(4).saturating_sub(width(&cancel));
    put(
        ui,
        area,
        cancel_x.saturating_sub(1),
        y,
        " ",
        plain(ink.canvas, ink.canvas),
    );
    put(
        ui,
        area,
        cancel_x,
        y,
        &cancel,
        plain(ink.secondary, ink.canvas),
    );
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    let key_style = bold(ink.primary, ink.canvas);
    let action_style = plain(ink.muted, ink.canvas);
    for hint in page.hints {
        put(ui, area, x, y, hint.key, key_style);
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, action_style);
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
    if let Some(mark) = page.hint_overflow {
        put(ui, area, x, y, mark, plain(ink.faint, ink.canvas));
    }
}
