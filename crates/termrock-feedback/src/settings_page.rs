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
    /// Column offset from the page left edge. `None` flows after the
    /// previous hint. The fixed-cell heritage pages pin every hint.
    pub at: Option<u16>,
}

/// Trailing text after the hint row, e.g. the mounts `…` or the env page's
/// clipped source peek.
pub struct SettingsOverflow<'a> {
    pub text: &'a str,
    /// Column offset from the page left edge. `None` flows after the hints.
    pub at: Option<u16>,
    /// Secondary ink instead of the faint mark.
    pub secondary: bool,
}

pub struct SettingsMount<'a> {
    pub path: &'a str,
    pub scope: &'a str,
    pub mode: &'a str,
    pub isolation: &'a str,
    pub kind: Option<&'a str>,
    pub source: Option<&'a str>,
    pub selected: bool,
}

pub struct SettingsEnvRow<'a> {
    pub key: &'a str,
    pub scope: &'a str,
    /// Masked value length; the painter renders one `*` per unit.
    pub stars: usize,
    pub source: &'a str,
    pub selected: bool,
}

pub struct SettingsEnvGroup<'a> {
    pub title: &'a str,
    pub right: &'a str,
    pub rows: &'a [SettingsEnvRow<'a>],
    pub add: &'a str,
}

pub struct SettingsEnvRole<'a> {
    pub name: &'a str,
    pub right: &'a str,
    pub rows: &'a [SettingsEnvRow<'a>],
    pub add: &'a str,
}

/// Environments body: global group, role-overrides group with one expanded
/// role, then the trailing add-role row.
pub struct SettingsEnv<'a> {
    pub global: SettingsEnvGroup<'a>,
    pub overrides_title: &'a str,
    pub overrides_right: &'a str,
    pub role: SettingsEnvRole<'a>,
    pub add_role: &'a str,
}

pub struct SettingsAgentRow<'a> {
    pub name: &'a str,
    pub mode: &'a str,
    pub account: &'a str,
    pub selected: bool,
}

/// Agents body: title row, column header, agent rows, then footnote lines.
pub struct SettingsAgents<'a> {
    pub title: &'a str,
    pub right: &'a str,
    pub header: &'a str,
    pub rows: &'a [SettingsAgentRow<'a>],
    pub notes: &'a [&'a str],
}

pub struct SettingsTrustRow<'a> {
    pub source: &'a str,
    pub trusted: bool,
    pub dirty: bool,
    pub count: &'a str,
    pub selected: bool,
}

/// Trust body: title row, one row per role source, then footnote lines.
/// `preview` dims the selected gutter to the plain cursor while the save
/// preview dialog owns the layer.
pub struct SettingsTrust<'a> {
    pub title: &'a str,
    pub right: &'a str,
    pub rows: &'a [SettingsTrustRow<'a>],
    pub notes: &'a [&'a str],
    pub preview: bool,
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
    /// Note appended to the selected tab label (trust save preview).
    /// A note also dims the active underline from accent to seam.
    pub tab_note: Option<&'a str>,
    /// Note painted after the crumb (trust save preview change count).
    pub crumb_note: Option<&'a str>,
    pub tabs: &'a [SettingsTab<'a>],
    pub sections: &'a [SettingsSection<'a>],
    pub cancel: &'a str,
    pub save: &'a str,
    pub save_enabled: bool,
    pub hints: &'a [SettingsHint<'a>],
    pub hint_overflow: Option<SettingsOverflow<'a>>,
    /// Skip the hint row; the page paints its own footer (trust preview).
    pub hide_hints: bool,
    pub mounts: &'a [SettingsMount<'a>],
    pub add_mount: Option<&'a str>,
    pub env: Option<SettingsEnv<'a>>,
    pub agents: Option<SettingsAgents<'a>>,
    pub trust: Option<SettingsTrust<'a>>,
    /// Wide header: crumb, note, mark, and value, packed from the right edge.
    /// Absent on the narrow page, which keeps the ellipsis before the crumb.
    pub cluster: Option<SettingsCluster<'a>>,
    /// Fixed-cell header anchor: the crumb ends at column 83, the note sits
    /// at 85 and the mark at 107 (env/agents/trust heritage frames).
    /// Without it the cluster packs from the right edge (general/mounts).
    pub legacy_header: bool,
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
    paint_underline(
        ui,
        area,
        selected_at,
        selected_span,
        &ink,
        page.tab_note.is_some(),
    );
    paint_sections(ui, area, page, &ink);
    paint_mounts(ui, area, page, &ink);
    paint_env(ui, area, page, &ink);
    paint_agents(ui, area, page, &ink);
    paint_trust(ui, area, page, &ink);
    paint_actions(ui, area, page, &ink);
    if !page.hide_hints {
        paint_hints(ui, area, page, &ink);
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
        if page.legacy_header {
            paint_legacy_cluster(ui, area, page, cluster, ink);
            return;
        }
        let value_x = area
            .right()
            .saturating_sub(2)
            .saturating_sub(width(cluster.value));
        let mark_x = value_x
            .saturating_sub(1)
            .saturating_sub(width(cluster.mark));
        let note_x = mark_x.saturating_sub(2).saturating_sub(width(cluster.note));
        let mut crumb_x = note_x.saturating_sub(2).saturating_sub(width(page.crumb));
        if let Some(note) = page.crumb_note {
            crumb_x = crumb_x.saturating_sub(width(note));
        }
        put(ui, area, crumb_x, y, page.crumb, secondary);
        if let Some(note) = page.crumb_note {
            put(
                ui,
                area,
                crumb_x.saturating_add(width(page.crumb)),
                y,
                note,
                plain(ink.primary, ink.canvas),
            );
        }
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

fn paint_legacy_cluster(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    cluster: SettingsCluster<'_>,
    ink: &Ink,
) {
    let y = area.y;
    let secondary = plain(ink.secondary, ink.canvas);
    let crumb_w = width(page.crumb);
    let crumb_x = match page.crumb_note {
        Some(note) => area
            .x
            .saturating_add(85)
            .saturating_sub(crumb_w.saturating_add(width(note))),
        None => area.x.saturating_add(83).saturating_sub(crumb_w),
    };
    put(ui, area, crumb_x, y, page.crumb, secondary);
    if let Some(note) = page.crumb_note {
        put(
            ui,
            area,
            crumb_x.saturating_add(crumb_w),
            y,
            note,
            plain(ink.primary, ink.canvas),
        );
    }
    put(
        ui,
        area,
        area.x.saturating_add(85),
        y,
        cluster.note,
        secondary,
    );
    put(
        ui,
        area,
        area.x.saturating_add(107),
        y,
        cluster.mark,
        plain(ink.accent, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(108),
        y,
        &format!(" {}", cluster.value),
        secondary,
    );
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
            let face = bold(ink.primary, ink.face);
            if let Some(note) = page.tab_note {
                let label = tab.label;
                let noted = format!(" {label} {note}  ");
                selected_span = width(&noted);
                put(ui, area, x, y, &noted, face);
            } else {
                selected_span = span;
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
            }
        } else {
            put(ui, area, x, y, &idle, plain(ink.secondary, ink.canvas));
        }
        x = x.saturating_add(span).saturating_add(1);
    }
    (selected_at, selected_span)
}

fn paint_underline(
    ui: &mut Ui<'_>,
    area: Rect,
    selected_at: u16,
    selected_span: u16,
    ink: &Ink,
    seam_heavy: bool,
) {
    let y = area.y.saturating_add(4);
    let start = area.x.saturating_add(2);
    let rule = plain(Color::Rgb(0x3a, 0x3a, 0x3a), ink.canvas);
    let heavy = plain(
        if seam_heavy {
            Color::Rgb(0x3a, 0x3a, 0x3a)
        } else {
            ink.accent
        },
        ink.canvas,
    );
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

const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);

/// Column geometry of the heritage 120-column pages. Positions are offsets
/// from the page left edge; the approved frames are 120 wide.
fn gutter(ui: &mut Ui<'_>, area: Rect, y: u16, ink: &Ink) {
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
}

fn right_note(ui: &mut Ui<'_>, area: Rect, y: u16, text: &str, ink: &Ink) {
    let x = area.right().saturating_sub(3).saturating_sub(width(text));
    put(ui, area, x, y, text, plain(ink.faint, ink.canvas));
}

fn selected_fill(ui: &mut Ui<'_>, area: Rect, y: u16, ink: &Ink) {
    ui.fill(
        Rect {
            x: area.x.saturating_add(2),
            y,
            width: area.width.saturating_sub(4),
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
}

/// Paint `text` word by word: words in `word`, gaps in `gap`. Ports the
/// heritage split painters, which recolored gaps inside selected fields.
fn paint_runs(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    word: PaintStyle,
    gap: PaintStyle,
) {
    let mut column = x;
    let mut current = String::new();
    let mut spaces = false;
    let mut flush = |column: &mut u16, current: &mut String, spaces: bool| {
        if current.is_empty() {
            return;
        }
        put(
            ui,
            area,
            *column,
            y,
            current,
            if spaces { gap } else { word },
        );
        *column = column.saturating_add(width(current));
        current.clear();
    };
    for ch in text.chars() {
        let is_space = ch == ' ';
        if !current.is_empty() && is_space != spaces {
            flush(&mut column, &mut current, spaces);
        }
        spaces = is_space;
        current.push(ch);
    }
    flush(&mut column, &mut current, spaces);
}

fn clip(text: &str, max: u16) -> String {
    crate::text::truncate(text, max)
}

fn pad(text: &str, cols: u16) -> String {
    let have = width(text);
    if have >= cols {
        return text.to_owned();
    }
    let mut out = text.to_owned();
    out.push_str(&" ".repeat(usize::from(cols - have)));
    out
}

fn paint_env(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let Some(env) = &page.env else {
        return;
    };
    let mut y = area.y.saturating_add(6);
    let title = bold(ink.secondary, ink.canvas);
    y = paint_env_group(ui, area, y, &env.global, ink, title);
    put(
        ui,
        area,
        area.x.saturating_add(6),
        y,
        env.overrides_title,
        title,
    );
    right_note(ui, area, y, env.overrides_right, ink);
    y = y.saturating_add(1);
    put(
        ui,
        area,
        area.x.saturating_add(4),
        y,
        "▾",
        plain(ink.secondary, ink.canvas),
    );
    put(ui, area, area.x.saturating_add(6), y, env.role.name, title);
    right_note(ui, area, y, env.role.right, ink);
    y = y.saturating_add(1);
    for row in env.role.rows {
        paint_env_row(ui, area, y, row, ink);
        y = y.saturating_add(1);
    }
    paint_add_row(ui, area, y, env.role.add, ink);
    y = y.saturating_add(1);
    paint_add_row(ui, area, y, env.add_role, ink);
}

fn paint_env_group(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    group: &SettingsEnvGroup<'_>,
    ink: &Ink,
    title: PaintStyle,
) -> u16 {
    let mut y = y;
    put(ui, area, area.x.saturating_add(6), y, group.title, title);
    right_note(ui, area, y, group.right, ink);
    y = y.saturating_add(1);
    for row in group.rows {
        paint_env_row(ui, area, y, row, ink);
        y = y.saturating_add(1);
    }
    paint_add_row(ui, area, y, group.add, ink);
    y.saturating_add(1)
}

fn paint_add_row(ui: &mut Ui<'_>, area: Rect, y: u16, label: &str, ink: &Ink) {
    gutter(ui, area, y, ink);
    put(
        ui,
        area,
        area.x.saturating_add(8),
        y,
        label,
        plain(ink.secondary, ink.canvas),
    );
}

fn paint_env_row(ui: &mut Ui<'_>, area: Rect, y: u16, row: &SettingsEnvRow<'_>, ink: &Ink) {
    let key = format!("    {:<20}", row.key);
    let scope = pad(row.scope, 14);
    let value = format!("{:<22}", "*".repeat(row.stars));
    let source = clip(row.source, 49);
    if row.selected {
        selected_fill(ui, area, y, ink);
        put(
            ui,
            area,
            area.x.saturating_add(4),
            y,
            &key,
            bold(ink.primary, SELECT_BG),
        );
        paint_runs(
            ui,
            area,
            area.x.saturating_add(28),
            y,
            &scope,
            bold(ink.secondary, SELECT_BG),
            bold(ink.muted, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(44),
            y,
            &value,
            bold(ink.secondary, SELECT_BG),
        );
        paint_runs(
            ui,
            area,
            area.x.saturating_add(68),
            y,
            &source,
            bold(ink.secondary, SELECT_BG),
            bold(ink.muted, SELECT_BG),
        );
    } else {
        gutter(ui, area, y, ink);
        put(
            ui,
            area,
            area.x.saturating_add(4),
            y,
            &key,
            plain(ink.primary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(28),
            y,
            &scope,
            plain(ink.muted, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(44),
            y,
            &value,
            plain(ink.secondary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(68),
            y,
            &pad(&source, 49),
            plain(ink.muted, ink.canvas),
        );
    }
}

fn paint_agents(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let Some(agents) = &page.agents else {
        return;
    };
    let y0 = area.y.saturating_add(6);
    let title = bold(ink.secondary, ink.canvas);
    put(ui, area, area.x.saturating_add(4), y0, agents.title, title);
    put(
        ui,
        area,
        area.x.saturating_add(77),
        y0,
        agents.right,
        plain(ink.faint, ink.canvas),
    );
    put(
        ui,
        area,
        area.x.saturating_add(7),
        y0.saturating_add(1),
        agents.header,
        plain(ink.muted, ink.canvas),
    );
    for (index, row) in agents.rows.iter().enumerate() {
        paint_agent_row(
            ui,
            area,
            y0.saturating_add(2).saturating_add(index as u16),
            row,
            ink,
        );
    }
    let mut y = y0
        .saturating_add(2)
        .saturating_add(agents.rows.len() as u16)
        .saturating_add(1);
    for note in agents.notes {
        put(
            ui,
            area,
            area.x.saturating_add(4),
            y,
            note,
            plain(ink.faint, ink.canvas),
        );
        y = y.saturating_add(1);
    }
}

fn paint_agent_row(ui: &mut Ui<'_>, area: Rect, y: u16, row: &SettingsAgentRow<'_>, ink: &Ink) {
    let bg = if row.selected { SELECT_BG } else { ink.canvas };
    if row.selected {
        selected_fill(ui, area, y, ink);
    } else {
        gutter(ui, area, y, ink);
    }
    let dot = if row.selected {
        bold(AMBER, bg)
    } else {
        plain(AMBER, bg)
    };
    put(ui, area, area.x.saturating_add(5), y, " ", dot);
    let name = if row.selected {
        bold(ink.primary, bg)
    } else {
        plain(ink.primary, bg)
    };
    put(ui, area, area.x.saturating_add(6), y, row.name, name);
    let mode = if row.selected {
        bold(ink.secondary, bg)
    } else {
        plain(ink.secondary, bg)
    };
    put(ui, area, area.x.saturating_add(21), y, row.mode, mode);
    if row.selected {
        paint_runs(
            ui,
            area,
            area.x.saturating_add(34),
            y,
            row.account,
            bold(ink.secondary, bg),
            bold(ink.muted, bg),
        );
    } else {
        put(
            ui,
            area,
            area.x.saturating_add(34),
            y,
            row.account,
            plain(ink.muted, ink.canvas),
        );
    }
}

fn paint_trust(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let Some(trust) = &page.trust else {
        return;
    };
    let y0 = area.y.saturating_add(6);
    let title = bold(ink.secondary, ink.canvas);
    put(ui, area, area.x.saturating_add(4), y0, trust.title, title);
    put(
        ui,
        area,
        area.right()
            .saturating_sub(4)
            .saturating_sub(width(trust.right)),
        y0,
        trust.right,
        plain(ink.faint, ink.canvas),
    );
    for (index, row) in trust.rows.iter().enumerate() {
        paint_trust_row(
            ui,
            area,
            y0.saturating_add(1).saturating_add(index as u16),
            row,
            trust.preview,
            ink,
        );
    }
    // Footnotes sit on fixed rows below the four-row heritage table.
    let mut y = area.y.saturating_add(12);
    for note in trust.notes {
        put(
            ui,
            area,
            area.x.saturating_add(4),
            y,
            note,
            plain(ink.faint, ink.canvas),
        );
        y = y.saturating_add(1);
    }
}

fn paint_trust_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &SettingsTrustRow<'_>,
    preview: bool,
    ink: &Ink,
) {
    let bg = if row.selected { SELECT_BG } else { ink.canvas };
    if row.selected {
        ui.fill(
            Rect {
                x: area.x.saturating_add(2),
                y,
                width: area.width.saturating_sub(4),
                height: 1,
            },
            bold(ink.primary, SELECT_BG),
        );
        if preview {
            put(ui, area, area.x.saturating_add(2), y, " ", plain(bg, bg));
            put(
                ui,
                area,
                area.x.saturating_add(3),
                y,
                "›",
                bold(ink.accent, bg),
            );
        } else {
            put(
                ui,
                area,
                area.x.saturating_add(2),
                y,
                "▎›",
                bold(ink.accent, bg),
            );
        }
    } else {
        gutter(ui, area, y, ink);
    }
    let dot = if row.selected {
        bold(AMBER, bg)
    } else {
        plain(AMBER, bg)
    };
    put(
        ui,
        area,
        area.x.saturating_add(5),
        y,
        if row.dirty { "•" } else { " " },
        dot,
    );
    let source = if row.selected {
        bold(ink.primary, bg)
    } else {
        plain(ink.primary, bg)
    };
    put(
        ui,
        area,
        area.x.saturating_add(6),
        y,
        &format!(" {:<42}", row.source),
        source,
    );
    let kind = format!("{:<5}", trust_kind(row.source));
    if row.selected {
        paint_runs(
            ui,
            area,
            area.x.saturating_add(49),
            y,
            &kind,
            bold(ink.secondary, bg),
            bold(ink.muted, bg),
        );
    } else {
        put(
            ui,
            area,
            area.x.saturating_add(49),
            y,
            &kind,
            plain(ink.muted, ink.canvas),
        );
    }
    if row.trusted {
        let state = if row.selected {
            bold(ink.primary, bg)
        } else {
            plain(ink.primary, bg)
        };
        put(
            ui,
            area,
            area.x.saturating_add(54),
            y,
            " [✓] trusted    ",
            state,
        );
    } else {
        put(
            ui,
            area,
            area.x.saturating_add(55),
            y,
            "[ ] untrusted",
            plain(AMBER, ink.canvas),
        );
    }
    if row.selected {
        paint_runs(
            ui,
            area,
            area.x.saturating_add(70),
            y,
            row.count,
            bold(ink.secondary, bg),
            bold(ink.faint, bg),
        );
    } else {
        put(
            ui,
            area,
            area.x.saturating_add(70),
            y,
            row.count,
            plain(ink.faint, ink.canvas),
        );
    }
}

fn trust_kind(source: &str) -> &'static str {
    if source.starts_with('~') || source.starts_with('/') {
        "path"
    } else {
        "git"
    }
}

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);

struct MountCols {
    path_at: u16,
    path_w: u16,
    scope_at: u16,
    scope_w: u16,
    mode_at: u16,
    mode_w: u16,
    iso_at: u16,
    iso_w: u16,
    kind_at: Option<u16>,
    kind_w: u16,
    source_at: Option<u16>,
    source_w: u16,
    fill_w: u16,
    clip_iso: bool,
    scroll_at: Option<u16>,
}

fn narrow_cols() -> MountCols {
    MountCols {
        path_at: 4,
        path_w: 42,
        scope_at: 46,
        scope_w: 14,
        mode_at: 62,
        mode_w: 4,
        iso_at: 68,
        iso_w: 9,
        kind_at: None,
        kind_w: 0,
        source_at: None,
        source_w: 0,
        fill_w: 68 + 9 - 1,
        clip_iso: true,
        scroll_at: Some(76),
    }
}

fn wide_cols(area_width: u16) -> MountCols {
    let source_at = 80;
    // Pad ends on the cell before the last selected cell. The last two
    // columns stay canvas.
    let source_w = area_width.saturating_sub(3).saturating_sub(source_at);
    MountCols {
        path_at: 4,
        path_w: 35,
        scope_at: 39,
        scope_w: 14,
        mode_at: 55,
        mode_w: 4,
        iso_at: 61,
        iso_w: 9,
        kind_at: Some(72),
        kind_w: 6,
        source_at: Some(source_at),
        source_w,
        fill_w: area_width.saturating_sub(4),
        clip_iso: false,
        scroll_at: None,
    }
}

fn fit(text: &str, cols: u16) -> String {
    let have = width(text);
    if have >= cols {
        return text.to_owned();
    }
    let mut out = text.to_owned();
    out.push_str(&" ".repeat(usize::from(cols - have)));
    out
}

fn path_cell(path: &str, cols: u16) -> String {
    fit(&format!("    {path}"), cols)
}

fn paint_mounts(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    if page.mounts.is_empty() {
        return;
    }
    let cols = if page.mounts.iter().any(|row| row.kind.is_some()) {
        wide_cols(area.width)
    } else {
        narrow_cols()
    };
    let y0 = area.y.saturating_add(6);
    let head = plain(ink.muted, ink.canvas);
    put(
        ui,
        area,
        area.x.saturating_add(cols.path_at + 4),
        y0,
        "Destination",
        head,
    );
    put(
        ui,
        area,
        area.x.saturating_add(cols.scope_at),
        y0,
        "Scope",
        head,
    );
    put(
        ui,
        area,
        area.x.saturating_add(cols.mode_at),
        y0,
        "Mode",
        head,
    );
    let iso_rect = if cols.clip_iso {
        Rect {
            x: area.x.saturating_add(cols.iso_at),
            y: y0,
            width: 8,
            height: 1,
        }
    } else {
        area
    };
    put(
        ui,
        iso_rect,
        area.x.saturating_add(cols.iso_at),
        y0,
        "Isolation",
        head,
    );
    if let Some(at) = cols.kind_at {
        put(ui, area, area.x.saturating_add(at), y0, "Kind", head);
    }
    if let Some(at) = cols.source_at {
        put(ui, area, area.x.saturating_add(at), y0, "Source", head);
    }
    if let Some(at) = cols.scroll_at {
        put(
            ui,
            area,
            area.x.saturating_add(at),
            y0,
            "2›",
            plain(ink.faint, ink.canvas),
        );
    }
    for (i, row) in page.mounts.iter().enumerate() {
        let y = y0.saturating_add(1).saturating_add(i as u16);
        paint_mount_row(ui, area, y, row, ink, &cols);
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

struct FieldRowPaint {
    area: Rect,
    y: u16,
    word: PaintStyle,
    pad: PaintStyle,
}

fn paint_field_parts(ui: &mut Ui<'_>, row: &FieldRowPaint, x: u16, text: &str, cols: u16) {
    put(ui, row.area, x, row.y, text, row.word);
    let gap = cols.saturating_sub(width(text));
    if gap > 0 {
        put(
            ui,
            row.area,
            x.saturating_add(width(text)),
            row.y,
            &" ".repeat(usize::from(gap)),
            row.pad,
        );
    }
}

fn paint_mount_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &SettingsMount<'_>,
    ink: &Ink,
    cols: &MountCols,
) {
    let path = path_cell(row.path, cols.path_w);
    let scope = fit(row.scope, cols.scope_w);
    let mode = fit(row.mode, cols.mode_w);
    let isolation = fit(row.isolation, cols.iso_w);
    if row.selected {
        let field_paint = FieldRowPaint {
            area,
            y,
            word: bold(ink.secondary, SELECT_BG),
            pad: bold(ink.muted, SELECT_BG),
        };
        ui.fill(
            Rect {
                x: area.x.saturating_add(2),
                y,
                width: cols.fill_w,
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
            area.x.saturating_add(cols.path_at),
            y,
            &path,
            bold(ink.primary, SELECT_BG),
        );
        paint_field_parts(
            ui,
            &field_paint,
            area.x.saturating_add(cols.scope_at),
            row.scope,
            cols.scope_w,
        );
        put(
            ui,
            area,
            area.x.saturating_add(cols.mode_at),
            y,
            &mode,
            bold(ink.secondary, SELECT_BG),
        );
        put(
            ui,
            area,
            area.x.saturating_add(cols.iso_at),
            y,
            &isolation,
            bold(ink.secondary, SELECT_BG),
        );
        if let (Some(at), Some(kind)) = (cols.kind_at, row.kind) {
            paint_field_parts(
                ui,
                &field_paint,
                area.x.saturating_add(at),
                kind,
                cols.kind_w,
            );
        }
        if let (Some(at), Some(source)) = (cols.source_at, row.source) {
            paint_field_parts(
                ui,
                &field_paint,
                area.x.saturating_add(at),
                source,
                cols.source_w,
            );
        }
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
            area.x.saturating_add(cols.path_at),
            y,
            &path,
            plain(ink.primary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(cols.scope_at),
            y,
            &scope,
            plain(ink.muted, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(cols.mode_at),
            y,
            &mode,
            plain(ink.secondary, ink.canvas),
        );
        put(
            ui,
            area,
            area.x.saturating_add(cols.iso_at),
            y,
            &isolation,
            plain(ink.secondary, ink.canvas),
        );
        if let (Some(at), Some(kind)) = (cols.kind_at, row.kind) {
            put(
                ui,
                area,
                area.x.saturating_add(at),
                y,
                &fit(kind, cols.kind_w),
                plain(ink.muted, ink.canvas),
            );
        }
        if let (Some(at), Some(source)) = (cols.source_at, row.source) {
            put(
                ui,
                area,
                area.x.saturating_add(at),
                y,
                &fit(source, cols.source_w),
                plain(ink.muted, ink.canvas),
            );
        }
    }
}

fn paint_actions(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(3);
    let save = format!("{} ", page.save);
    let cancel = format!("{} ", page.cancel);
    let save_x = area.right().saturating_sub(5).saturating_sub(width(&save));
    // An armed Save paints secondary, matching the only approved enabled
    // frame (trust save preview), not a primary call to action.
    let save_ink = if page.save_enabled {
        ink.secondary
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
        if let Some(at) = hint.at {
            x = area.x.saturating_add(at);
        }
        put(ui, area, x, y, hint.key, key_style);
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, action_style);
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
    if let Some(tail) = &page.hint_overflow {
        if let Some(at) = tail.at {
            x = area.x.saturating_add(at);
        }
        put(
            ui,
            area,
            x,
            y,
            tail.text,
            plain(
                if tail.secondary {
                    ink.secondary
                } else {
                    ink.faint
                },
                ink.canvas,
            ),
        );
    }
}
