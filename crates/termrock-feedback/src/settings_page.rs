//! Settings page: menu, tabs, checkbox sections, actions, and hints.
//!
//! Positions follow the labels and the screen edges. The selected tab
//! carries the focus gutter and the heavy underline.

use std::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::measure::{grapheme_width, graphemes};
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
    pub kind: Option<&'a str>,
    pub source: Option<&'a str>,
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

/// One masked environment variable row.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnvRow<'a> {
    key: &'a str,
    scope: &'a str,
    stars: usize,
    source: &'a str,
    selected: bool,
}

impl<'a> SettingsEnvRow<'a> {
    /// Create a row. `stars` is the masked value width in terminal cells.
    pub const fn new(key: &'a str, scope: &'a str, stars: usize, source: &'a str) -> Self {
        Self {
            key,
            scope,
            stars,
            source,
            selected: false,
        }
    }

    /// Set whether this row has the list selection fill.
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// A titled environment group with zero or more rows.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnvGroup<'a> {
    title: &'a str,
    right: Option<&'a str>,
    rows: &'a [SettingsEnvRow<'a>],
    add: Option<&'a str>,
}

impl<'a> SettingsEnvGroup<'a> {
    /// Create a group from borrowed rows.
    pub const fn new(title: &'a str, rows: &'a [SettingsEnvRow<'a>]) -> Self {
        Self {
            title,
            right: None,
            rows,
            add: None,
        }
    }

    /// Add a right-aligned summary to the group heading.
    pub const fn right(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    /// Add a trailing action row after the group's variables.
    pub const fn add(mut self, label: &'a str) -> Self {
        self.add = Some(label);
        self
    }
}

/// One role-specific environment group.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnvRole<'a> {
    name: &'a str,
    right: Option<&'a str>,
    rows: &'a [SettingsEnvRow<'a>],
    add: Option<&'a str>,
}

impl<'a> SettingsEnvRole<'a> {
    /// Create a role from borrowed rows.
    pub const fn new(name: &'a str, rows: &'a [SettingsEnvRow<'a>]) -> Self {
        Self {
            name,
            right: None,
            rows,
            add: None,
        }
    }

    /// Add a right-aligned summary to the role heading.
    pub const fn right(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    /// Add a trailing action row after the role's variables.
    pub const fn add(mut self, label: &'a str) -> Self {
        self.add = Some(label);
        self
    }
}

/// Borrowed environment content for the shared settings painter.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnv<'a> {
    groups: &'a [SettingsEnvGroup<'a>],
    overrides: Option<SettingsEnvOverrides<'a>>,
}

#[derive(Clone, Copy, Debug)]
struct SettingsEnvOverrides<'a> {
    title: &'a str,
    right: Option<&'a str>,
    roles: &'a [SettingsEnvRole<'a>],
    add_role: Option<&'a str>,
}

impl<'a> SettingsEnv<'a> {
    /// Create environment content from any number of global groups.
    pub const fn new(groups: &'a [SettingsEnvGroup<'a>]) -> Self {
        Self {
            groups,
            overrides: None,
        }
    }

    /// Add a role-overrides group with any number of roles.
    pub const fn role_overrides(
        mut self,
        title: &'a str,
        right: Option<&'a str>,
        roles: &'a [SettingsEnvRole<'a>],
        add_role: Option<&'a str>,
    ) -> Self {
        self.overrides = Some(SettingsEnvOverrides {
            title,
            right,
            roles,
            add_role,
        });
        self
    }
}

/// Failure to paint all environment rows inside the supplied page area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsEnvPaintError {
    /// The legacy page still contains sections or mounts that would overlap the Env body.
    LegacyBodyNotEmpty,
    /// The page is too narrow to show the row marker and all four data columns.
    AreaTooNarrow { minimum: u16, actual: u16 },
    /// The page is too short for the shared header and footer.
    AreaTooShort { minimum: u16, actual: u16 },
    /// All content cannot fit above the action row.
    ContentTooTall {
        required_lines: usize,
        available_lines: usize,
    },
}

impl fmt::Display for SettingsEnvPaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacyBodyNotEmpty => {
                f.write_str("legacy settings sections and mounts must be empty")
            }
            Self::AreaTooNarrow { minimum, actual } => {
                write!(f, "settings Env needs {minimum} columns, received {actual}")
            }
            Self::AreaTooShort { minimum, actual } => {
                write!(f, "settings page needs {minimum} rows, received {actual}")
            }
            Self::ContentTooTall {
                required_lines,
                available_lines,
            } => write!(
                f,
                "settings Env needs {required_lines} body rows, but area provides {available_lines}"
            ),
        }
    }
}

impl std::error::Error for SettingsEnvPaintError {}

const SETTINGS_ENV_MIN_WIDTH: u16 = 16;
const SETTINGS_ENV_MIN_HEIGHT: u16 = 8;
const SETTINGS_ENV_BODY_TOP: u16 = 6;
const SETTINGS_ACTION_OFFSET: u16 = 3;

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
    let ink = settings_ink(ui);
    paint_settings_frame(ui, area, page, &ink);
    paint_sections(ui, area, page, &ink);
    paint_mounts(ui, area, page, &ink);
    paint_settings_footer(ui, area, page, &ink);
}

/// Paint an Env body with the existing page chrome.
///
/// `page.sections`, `page.mounts`, and `page.add_mount` must be empty. The
/// function borrows all rows and paints every row or returns an error before
/// drawing. Horizontal text clips to the supplied area. Content taller than
/// the available body space is rejected; this entry point does not scroll or
/// handle input and focus.
pub fn paint_settings_page_with_env(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    env: &SettingsEnv<'_>,
) -> Result<(), SettingsEnvPaintError> {
    if !page.sections.is_empty() || !page.mounts.is_empty() || page.add_mount.is_some() {
        return Err(SettingsEnvPaintError::LegacyBodyNotEmpty);
    }
    if area.width < SETTINGS_ENV_MIN_WIDTH {
        return Err(SettingsEnvPaintError::AreaTooNarrow {
            minimum: SETTINGS_ENV_MIN_WIDTH,
            actual: area.width,
        });
    }
    if area.height < SETTINGS_ENV_MIN_HEIGHT {
        return Err(SettingsEnvPaintError::AreaTooShort {
            minimum: SETTINGS_ENV_MIN_HEIGHT,
            actual: area.height,
        });
    }
    let required_lines = settings_env_required_lines(env);
    let available_lines = usize::from(
        area.height
            .saturating_sub(SETTINGS_ENV_BODY_TOP.saturating_add(SETTINGS_ACTION_OFFSET)),
    );
    if required_lines > available_lines {
        return Err(SettingsEnvPaintError::ContentTooTall {
            required_lines,
            available_lines,
        });
    }

    let ink = settings_ink(ui);
    paint_settings_frame(ui, area, page, &ink);
    paint_settings_env(ui, area, env, &ink);
    paint_settings_footer(ui, area, page, &ink);
    Ok(())
}

fn settings_ink(ui: &Ui<'_>) -> Ink {
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
}

fn paint_settings_frame(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    ui.fill(area, plain(ink.primary, ink.canvas));
    paint_header(ui, area, page, ink);
    let (selected_at, selected_span) = paint_tabs(ui, area, page, ink);
    paint_underline(ui, area, selected_at, selected_span, ink);
}

fn paint_settings_footer(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    paint_actions(ui, area, page, ink);
    paint_hints(ui, area, page, ink);
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
    let width = area.right().saturating_sub(x);
    put_clipped(ui, area, x, y, text, width, style);
}

fn put_cols(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, cols: u16, style: PaintStyle) {
    put_clipped(ui, area, x, y, text, cols, style);
}

fn put_clipped(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    width: u16,
    style: PaintStyle,
) {
    if width == 0 || y < area.y || y >= area.bottom() || text.is_empty() {
        return;
    }

    let right = x.saturating_add(width).min(area.right());
    if right <= area.x {
        return;
    }

    let mut draw_x = x;
    let mut text_start = 0;
    if draw_x < area.x {
        for (byte, grapheme) in graphemes(text) {
            if draw_x >= area.x {
                text_start = byte;
                break;
            }

            draw_x = draw_x.saturating_add(grapheme_width(grapheme));
            text_start = byte.saturating_add(grapheme.len());
        }
    }

    if draw_x < area.x || draw_x >= right {
        return;
    }
    let Some(visible_text) = text.get(text_start..) else {
        return;
    };
    if visible_text.is_empty() {
        return;
    }

    ui.paint_str(
        Rect::new(draw_x, y, right.saturating_sub(draw_x), 1),
        visible_text,
        style,
    );
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

fn settings_env_required_lines(env: &SettingsEnv<'_>) -> usize {
    let mut lines = 0_usize;
    for (index, group) in env.groups.iter().enumerate() {
        if index > 0 {
            lines = lines.saturating_add(1);
        }
        lines = lines.saturating_add(1).saturating_add(group.rows.len());
        if group.add.is_some() {
            lines = lines.saturating_add(1);
        }
    }
    if let Some(overrides) = env.overrides {
        if !env.groups.is_empty() {
            lines = lines.saturating_add(1);
        }
        lines = lines.saturating_add(1);
        for (index, role) in overrides.roles.iter().enumerate() {
            if index > 0 {
                lines = lines.saturating_add(1);
            }
            lines = lines.saturating_add(1).saturating_add(role.rows.len());
            if role.add.is_some() {
                lines = lines.saturating_add(1);
            }
        }
        if overrides.add_role.is_some() {
            lines = lines.saturating_add(1);
        }
    }
    lines
}

fn paint_settings_env(ui: &mut Ui<'_>, area: Rect, env: &SettingsEnv<'_>, ink: &Ink) {
    let mut y = area.y.saturating_add(SETTINGS_ENV_BODY_TOP);
    for (index, group) in env.groups.iter().enumerate() {
        if index > 0 {
            y = y.saturating_add(1);
        }
        paint_env_heading(
            ui,
            area,
            y,
            area.x.saturating_add(4),
            group.title,
            group.right,
            ink,
        );
        y = y.saturating_add(1);
        for row in group.rows {
            paint_env_row(ui, area, y, row, ink);
            y = y.saturating_add(1);
        }
        if let Some(add) = group.add {
            paint_env_add(ui, area, y, add, ink);
            y = y.saturating_add(1);
        }
    }
    if let Some(overrides) = env.overrides {
        if !env.groups.is_empty() {
            y = y.saturating_add(1);
        }
        paint_env_heading(
            ui,
            area,
            y,
            area.x.saturating_add(4),
            overrides.title,
            overrides.right,
            ink,
        );
        y = y.saturating_add(1);
        for (index, role) in overrides.roles.iter().enumerate() {
            if index > 0 {
                y = y.saturating_add(1);
            }
            put_cols(
                ui,
                area,
                area.x.saturating_add(4),
                y,
                "▾",
                1,
                plain(ink.secondary, ink.canvas),
            );
            paint_env_heading(
                ui,
                area,
                y,
                area.x.saturating_add(6),
                role.name,
                role.right,
                ink,
            );
            y = y.saturating_add(1);
            for row in role.rows {
                paint_env_row(ui, area, y, row, ink);
                y = y.saturating_add(1);
            }
            if let Some(add) = role.add {
                paint_env_add(ui, area, y, add, ink);
                y = y.saturating_add(1);
            }
        }
        if let Some(add) = overrides.add_role {
            paint_env_add(ui, area, y, add, ink);
        }
    }
}

fn paint_env_heading(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    x: u16,
    title: &str,
    right: Option<&str>,
    ink: &Ink,
) {
    let end = area.right().saturating_sub(2);
    let available = end.saturating_sub(x);
    let right_cols = right
        .filter(|text| !text.is_empty())
        .map_or(0, |text| width(text).min(available / 2));
    let gap = if right_cols > 0 { 1 } else { 0 };
    let title_cols = available.saturating_sub(right_cols).saturating_sub(gap);
    put_cols(
        ui,
        area,
        x,
        y,
        title,
        title_cols,
        bold(ink.secondary, ink.canvas),
    );
    if right_cols > 0 {
        put_cols(
            ui,
            area,
            end.saturating_sub(right_cols),
            y,
            right.unwrap_or(""),
            right_cols,
            plain(ink.faint, ink.canvas),
        );
    }
}

struct SettingsEnvColumns {
    key: u16,
    scope: u16,
    stars: u16,
    source: u16,
}

fn settings_env_columns(area: Rect) -> SettingsEnvColumns {
    let content_start = area.x.saturating_add(4);
    let content_end = area.right().saturating_sub(2);
    let field_budget = content_end.saturating_sub(content_start).saturating_sub(3);
    let key = scaled_env_column(field_budget, 20, 20);
    let scope = scaled_env_column(field_budget, 14, 14);
    let stars = scaled_env_column(field_budget, 22, 22);
    let source = field_budget.saturating_sub(key.saturating_add(scope).saturating_add(stars));
    SettingsEnvColumns {
        key,
        scope,
        stars,
        source,
    }
}

fn scaled_env_column(total: u16, weight: u32, maximum: u16) -> u16 {
    let weighted = u32::from(total).saturating_mul(weight) / 105;
    u16::try_from(weighted)
        .unwrap_or(u16::MAX)
        .min(maximum)
        .max(1)
}

fn paint_env_row(ui: &mut Ui<'_>, area: Rect, y: u16, row: &SettingsEnvRow<'_>, ink: &Ink) {
    let gutter_x = area.x.saturating_add(2);
    let text_x = area.x.saturating_add(4);
    let selected = row.selected;
    let selected_bg = if selected { SELECT_BG } else { ink.canvas };
    if selected {
        ui.fill(
            Rect::new(gutter_x, y, area.width.saturating_sub(4), 1),
            bold(ink.primary, SELECT_BG),
        );
        put_cols(ui, area, gutter_x, y, "▎›", 2, bold(ink.accent, SELECT_BG));
    } else {
        put_cols(ui, area, gutter_x, y, " ", 1, plain(ink.canvas, ink.canvas));
        put_cols(
            ui,
            area,
            gutter_x.saturating_add(1),
            y,
            " ",
            1,
            plain(ink.secondary, ink.canvas),
        );
    }

    let columns = settings_env_columns(area);
    let key_style = if selected {
        bold(ink.primary, selected_bg)
    } else {
        plain(ink.primary, selected_bg)
    };
    let scope_style = if selected {
        bold(ink.muted, selected_bg)
    } else {
        plain(ink.muted, selected_bg)
    };
    let stars_style = if selected {
        bold(ink.secondary, selected_bg)
    } else {
        plain(ink.secondary, selected_bg)
    };
    let source_style = if selected {
        bold(ink.faint, selected_bg)
    } else {
        plain(ink.faint, selected_bg)
    };

    let mut x = text_x;
    put_cols(ui, area, x, y, row.key, columns.key, key_style);
    x = x.saturating_add(columns.key).saturating_add(1);
    put_cols(ui, area, x, y, row.scope, columns.scope, scope_style);
    x = x.saturating_add(columns.scope).saturating_add(1);
    paint_env_stars(ui, area, x, y, row.stars, columns.stars, stars_style);
    x = x.saturating_add(columns.stars).saturating_add(1);
    put_cols(ui, area, x, y, row.source, columns.source, source_style);
}

fn paint_env_stars(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    stars: usize,
    columns: u16,
    style: PaintStyle,
) {
    const STAR_CHUNK: &str = "********************************";

    let visible = area.right().saturating_sub(x).min(columns);
    let mut remaining = stars.min(usize::from(visible));
    let mut column = x;
    while remaining > 0 {
        let count = remaining.min(STAR_CHUNK.len());
        let text = &STAR_CHUNK[..count];
        let width = u16::try_from(count).unwrap_or(u16::MAX);
        put_cols(ui, area, column, y, text, width, style);
        column = column.saturating_add(width);
        remaining -= count;
    }
}

fn paint_env_add(ui: &mut Ui<'_>, area: Rect, y: u16, label: &str, ink: &Ink) {
    put_cols(
        ui,
        area,
        area.x.saturating_add(2),
        y,
        " ",
        1,
        plain(ink.canvas, ink.canvas),
    );
    put_cols(
        ui,
        area,
        area.x.saturating_add(4),
        y,
        label,
        area.right()
            .saturating_sub(2)
            .saturating_sub(area.x.saturating_add(4)),
        plain(ink.secondary, ink.canvas),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_core::layout::Position;
    use termrock::{ColorLevel, Theme};
    use termrock_test_support::Scene;

    fn legacy_page<'a>(
        sections: &'a [SettingsSection<'a>],
        mounts: &'a [SettingsMount<'a>],
        add_mount: Option<&'a str>,
    ) -> SettingsPage<'a> {
        SettingsPage {
            brand: "Settings",
            menus: &[],
            crumb: "Settings › global",
            tabs: &[],
            sections,
            cancel: "Cancel",
            save: "Save",
            save_enabled: true,
            hints: &[],
            hint_overflow: None,
            mounts,
            add_mount,
            cluster: None,
        }
    }

    #[test]
    fn legacy_page_literal_still_paints_general_and_mount_content() {
        let checks = [SettingsCheck {
            label: "Use cache",
            checked: true,
        }];
        let sections = [SettingsSection {
            title: "General",
            checks: &checks,
            note: None,
            body: None,
        }];
        let page = legacy_page(&sections, &[], None);
        let mut general = Scene::new(
            "settings_general_legacy_api",
            Theme::junie(),
            ColorLevel::TrueColor,
            80,
            24,
        );
        general.draw(|ui, area| paint_settings_page(ui, area, &page));
        assert!(general.text().contains("Settings › global"));
        assert!(general.text().contains("General"));
        assert!(general.text().contains("Use cache"));

        let mounts = [SettingsMount {
            path: "/workspace",
            scope: "global",
            mode: "read-write",
            isolation: "host",
            kind: None,
            source: None,
            selected: true,
        }];
        let page = legacy_page(&[], &mounts, Some("Add mount"));
        let mut mount_scene = Scene::new(
            "settings_mounts_legacy_api",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            40,
        );
        mount_scene.draw(|ui, area| paint_settings_page(ui, area, &page));
        assert!(mount_scene.text().contains("Destination"));
        assert!(mount_scene.text().contains("/workspace"));
        assert!(mount_scene.text().contains("Add mount"));
    }

    #[test]
    fn put_preserves_the_logical_ascii_origin_and_stays_inside_its_area() {
        let area = Rect::new(4, 2, 6, 3);
        let mut scene = Scene::new(
            "settings_put_offset_ascii",
            Theme::junie(),
            ColorLevel::TrueColor,
            16,
            8,
        );
        scene.draw_over(
            |buffer| {
                for pos in buffer.area().positions() {
                    if let Some(cell) = buffer.cell_mut(pos) {
                        cell.set_symbol("!");
                    }
                }
            },
            |ui, _| put(ui, area, 1, 3, "abcdefgh", PaintStyle::new()),
        );

        for (x, expected) in [(4, "d"), (5, "e"), (6, "f"), (7, "g"), (8, "h")] {
            assert_eq!(
                scene.buffer().cell((x, 3)).map(|cell| cell.symbol()),
                Some(expected)
            );
        }
        assert_eq!(
            scene.buffer().cell((9, 3)).map(|cell| cell.symbol()),
            Some("!")
        );
        for pos in scene.area().positions() {
            if !area.contains(pos) {
                assert_eq!(
                    scene.buffer().cell(pos).map(|cell| cell.symbol()),
                    Some("!"),
                    "put changed a cell outside the supplied area at ({}, {})",
                    pos.x,
                    pos.y,
                );
            }
        }
    }

    #[test]
    fn put_cols_keeps_its_original_width_budget_after_left_clipping() {
        let area = Rect::new(4, 2, 12, 3);
        let mut scene = Scene::new(
            "settings_put_cols_offset_budget",
            Theme::junie(),
            ColorLevel::TrueColor,
            18,
            8,
        );
        scene.draw_over(
            |buffer| {
                for pos in buffer.area().positions() {
                    if let Some(cell) = buffer.cell_mut(pos) {
                        cell.set_symbol("!");
                    }
                }
            },
            |ui, _| put_cols(ui, area, 1, 3, "abcdefghi", 6, PaintStyle::new()),
        );

        assert_eq!(
            scene.buffer().cell((4, 3)).map(|cell| cell.symbol()),
            Some("d")
        );
        assert_eq!(
            scene.buffer().cell((5, 3)).map(|cell| cell.symbol()),
            Some("e")
        );
        assert_eq!(
            scene.buffer().cell((6, 3)).map(|cell| cell.symbol()),
            Some("f")
        );
        assert_eq!(
            scene.buffer().cell((7, 3)).map(|cell| cell.symbol()),
            Some("!")
        );
        for pos in scene.area().positions() {
            if !area.contains(pos) {
                assert_eq!(
                    scene.buffer().cell(pos).map(|cell| cell.symbol()),
                    Some("!"),
                    "put_cols changed a cell outside the supplied area at ({}, {})",
                    pos.x,
                    pos.y,
                );
            }
        }
    }

    #[test]
    fn put_clips_only_at_complete_grapheme_boundaries_on_both_sides() {
        let family = "👨‍👩‍👧‍👦";
        let mut scene = Scene::new(
            "settings_put_unicode_clipping",
            Theme::junie(),
            ColorLevel::TrueColor,
            12,
            8,
        );
        let areas = [
            Rect::new(1, 1, 8, 1),
            Rect::new(1, 2, 8, 1),
            Rect::new(1, 3, 8, 1),
            Rect::new(1, 4, 8, 1),
            Rect::new(3, 5, 1, 1),
            Rect::new(3, 6, 1, 1),
        ];
        scene.draw_over(
            |buffer| {
                for pos in buffer.area().positions() {
                    if let Some(cell) = buffer.cell_mut(pos) {
                        cell.set_symbol("!");
                    }
                }
            },
            |ui, _| {
                put(ui, areas[0], 0, 1, "e\u{301}X", PaintStyle::new());
                put(ui, areas[1], 0, 2, "\u{301}XY", PaintStyle::new());
                put(ui, areas[2], 0, 3, "界X", PaintStyle::new());
                put(ui, areas[3], 0, 4, &format!("{family}Z"), PaintStyle::new());
                put(ui, areas[4], 3, 5, "界X", PaintStyle::new());
                put(ui, areas[5], 3, 6, family, PaintStyle::new());
            },
        );

        assert_eq!(
            scene.buffer().cell((1, 1)).map(|cell| cell.symbol()),
            Some("X")
        );
        assert_eq!(
            scene.buffer().cell((1, 2)).map(|cell| cell.symbol()),
            Some("Y")
        );
        assert_eq!(
            scene.buffer().cell((1, 3)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            scene.buffer().cell((2, 3)).map(|cell| cell.symbol()),
            Some("X")
        );
        assert_eq!(
            scene.buffer().cell((1, 4)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            scene.buffer().cell((2, 4)).map(|cell| cell.symbol()),
            Some("Z")
        );
        assert_eq!(
            scene.buffer().cell((3, 5)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            scene.buffer().cell((3, 6)).map(|cell| cell.symbol()),
            Some("!")
        );
    }

    #[test]
    fn chrome_keeps_combining_marks_attached_to_group_padding() {
        let tabs = [SettingsTab {
            label: "\u{301}General",
            selected: false,
        }];
        let mut page = legacy_page(&[], &[], None);
        page.brand = "\u{301}Settings";
        page.menus = &["\u{301}Files"];
        page.tabs = &tabs;
        let mut scene = Scene::new(
            "settings_grouped_combining_chrome",
            Theme::junie(),
            ColorLevel::TrueColor,
            80,
            24,
        );
        scene.draw(|ui, area| paint_settings_page(ui, area, &page));

        assert_eq!(
            scene.buffer().cell((1, 0)).map(|cell| cell.symbol()),
            Some(" \u{301}")
        );
        assert_eq!(
            scene.buffer().cell((2, 3)).map(|cell| cell.symbol()),
            Some(" \u{301}")
        );
    }

    #[test]
    fn env_content_renders_variable_groups_roles_and_every_row() {
        let global_rows = [
            SettingsEnvRow::new("GH_TOKEN", "global", 8, "credential store").selected(true),
            SettingsEnvRow::new("APP_HOME", "global", 0, "shell"),
        ];
        let project_rows: [SettingsEnvRow<'_>; 0] = [];
        let groups = [
            SettingsEnvGroup::new("Global", &global_rows)
                .right("2 variables")
                .add("Add global variable"),
            SettingsEnvGroup::new("Project", &project_rows).right("0 variables"),
        ];
        let build_rows = [
            SettingsEnvRow::new("BUILD_TOKEN", "role", 5, "credential store"),
            SettingsEnvRow::new("WORKDIR", "role", 0, "inherited"),
        ];
        let deploy_rows = [SettingsEnvRow::new("DEPLOY_KEY", "role", 4, "keychain")];
        let roles = [
            SettingsEnvRole::new("builder", &build_rows)
                .right("2 variables")
                .add("Add role variable"),
            SettingsEnvRole::new("deployer", &deploy_rows).right("1 variable"),
        ];
        let env = SettingsEnv::new(&groups).role_overrides(
            "Role overrides",
            Some("2 roles"),
            &roles,
            Some("Add role"),
        );
        let page = legacy_page(&[], &[], None);
        let mut scene = Scene::new(
            "settings_env_content",
            Theme::junie(),
            ColorLevel::TrueColor,
            100,
            34,
        );
        scene.draw(|ui, area| {
            assert_eq!(paint_settings_page_with_env(ui, area, &page, &env), Ok(()));
        });
        let text = scene.text();
        for expected in [
            "Global",
            "Project",
            "GH_TOKEN",
            "APP_HOME",
            "Add global variable",
            "Role overrides",
            "builder",
            "deployer",
            "BUILD_TOKEN",
            "WORKDIR",
            "DEPLOY_KEY",
            "Add role variable",
            "Add role",
        ] {
            assert!(
                text.contains(expected),
                "missing {expected:?} from Env page"
            );
        }
    }

    #[test]
    fn env_content_clips_unicode_and_masked_values_to_offset_area() {
        let rows = [SettingsEnvRow::new(
            "非常に長い認証トークン",
            "共有スコープ",
            usize::MAX,
            "credential source tail must be clipped",
        )];
        let groups = [
            SettingsEnvGroup::new("長い環境グループの見出し", &rows).right("右側の長い件数メモ")
        ];
        let env = SettingsEnv::new(&groups);
        let mut page = legacy_page(&[], &[], None);
        page.crumb = "An intentionally long Unicode settings breadcrumb";
        page.cancel = "Cancel with a deliberately long label";
        page.save = "Save with a deliberately long label";
        let area = Rect::new(9, 3, 24, 26);
        let mut scene = Scene::new(
            "settings_env_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            48,
            36,
        );
        scene.draw_over(
            |buffer| {
                for pos in buffer.area().positions() {
                    if let Some(cell) = buffer.cell_mut(pos) {
                        cell.set_symbol("!");
                    }
                }
            },
            |ui, _| {
                assert_eq!(paint_settings_page_with_env(ui, area, &page, &env), Ok(()));
            },
        );
        let text = scene.text();
        assert!(text.contains("非"));
        assert!(text.contains("***"));
        assert!(!text.contains("tail must be clipped"));
        assert_ne!(
            scene
                .buffer()
                .cell((area.x, area.y))
                .map(|cell| cell.symbol()),
            Some("!")
        );
        for pos in scene.area().positions() {
            if !area.contains(Position::new(pos.x, pos.y)) {
                assert_eq!(
                    scene.buffer().cell(pos).map(|cell| cell.symbol()),
                    Some("!"),
                    "paint escaped supplied area at ({}, {})",
                    pos.x,
                    pos.y,
                );
            }
        }
    }

    #[test]
    fn env_style_uses_theme_channels_for_selected_fields() {
        let rows = [
            SettingsEnvRow::new("TOKEN", "global", 3, "vault").selected(true),
            SettingsEnvRow::new("OTHER", "shell", 1, "inherited"),
        ];
        let groups = [SettingsEnvGroup::new("Global", &rows).right("2 variables")];
        let env = SettingsEnv::new(&groups);
        let page = legacy_page(&[], &[], None);
        let area = Rect::new(0, 0, 80, 20);
        for theme in [Theme::junie(), Theme::paper()] {
            let mut scene = Scene::new(
                "settings_env_styles",
                theme.clone(),
                ColorLevel::TrueColor,
                80,
                20,
            );
            scene.draw(|ui, _| {
                assert_eq!(paint_settings_page_with_env(ui, area, &page, &env), Ok(()));
            });
            let columns = settings_env_columns(area);
            let scope_x = 4_u16.saturating_add(columns.key).saturating_add(1);
            let stars_x = scope_x.saturating_add(columns.scope).saturating_add(1);
            let source_x = stars_x.saturating_add(columns.stars).saturating_add(1);
            assert_eq!(
                scene.buffer().cell((2, 7)).map(|cell| cell.fg),
                Some(theme.color.accent)
            );
            assert_eq!(
                scene.buffer().cell((4, 7)).map(|cell| (cell.fg, cell.bg)),
                Some((step(&theme, FgStep::Primary), SELECT_BG))
            );
            assert_eq!(
                scene.buffer().cell((scope_x, 7)).map(|cell| cell.fg),
                Some(step(&theme, FgStep::Muted))
            );
            assert_eq!(
                scene.buffer().cell((stars_x, 7)).map(|cell| cell.fg),
                Some(step(&theme, FgStep::Secondary))
            );
            assert_eq!(
                scene.buffer().cell((source_x, 7)).map(|cell| cell.fg),
                Some(step(&theme, FgStep::Faint))
            );
            assert_eq!(
                scene.buffer().cell((4, 6)).map(|cell| cell.fg),
                Some(step(&theme, FgStep::Secondary))
            );
        }
    }

    #[test]
    fn env_layout_rejects_overflow_and_invalid_areas_before_painting() {
        let rows = [
            SettingsEnvRow::new("ONE", "global", 1, "shell"),
            SettingsEnvRow::new("TWO", "global", 1, "shell"),
            SettingsEnvRow::new("THREE", "global", 1, "shell"),
            SettingsEnvRow::new("FOUR", "global", 1, "shell"),
            SettingsEnvRow::new("FIVE", "global", 1, "shell"),
            SettingsEnvRow::new("SIX", "global", 1, "shell"),
            SettingsEnvRow::new("SEVEN", "global", 1, "shell"),
        ];
        let groups = [SettingsEnvGroup::new("Global", &rows)];
        let env = SettingsEnv::new(&groups);
        let page = legacy_page(&[], &[], None);
        let area = Rect::new(0, 0, 32, 16);
        let mut scene = Scene::new(
            "settings_env_overflow",
            Theme::junie(),
            ColorLevel::TrueColor,
            32,
            16,
        );
        scene.draw_over(
            |buffer| {
                for pos in buffer.area().positions() {
                    if let Some(cell) = buffer.cell_mut(pos) {
                        cell.set_symbol("!");
                    }
                }
            },
            |ui, _| {
                assert_eq!(
                    paint_settings_page_with_env(ui, area, &page, &env),
                    Err(SettingsEnvPaintError::ContentTooTall {
                        required_lines: 8,
                        available_lines: 7,
                    })
                );
            },
        );
        assert!(
            scene
                .buffer()
                .content
                .iter()
                .all(|cell| cell.symbol() == "!")
        );

        let mut narrow = Scene::new(
            "settings_env_too_narrow",
            Theme::junie(),
            ColorLevel::TrueColor,
            15,
            16,
        );
        narrow.draw(|ui, _| {
            assert_eq!(
                paint_settings_page_with_env(ui, Rect::new(0, 0, 15, 16), &page, &env),
                Err(SettingsEnvPaintError::AreaTooNarrow {
                    minimum: SETTINGS_ENV_MIN_WIDTH,
                    actual: 15,
                })
            );
        });
        assert!(narrow.text().is_empty() || narrow.text().trim().is_empty());
    }
}
