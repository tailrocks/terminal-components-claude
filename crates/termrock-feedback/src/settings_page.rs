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

/// Explicit source kind for a trusted role source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsTrustSourceKind {
    /// Source uses a Git transport.
    Git,
    /// Source is a local path.
    Path,
    /// Caller has no verified source-kind metadata.
    Unknown,
}

/// One borrowed agent row for the shared Settings renderer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsAgentRow<'a> {
    name: &'a str,
    mode: &'a str,
    account: &'a str,
    selected: bool,
}

impl<'a> SettingsAgentRow<'a> {
    /// Create an agent row. The renderer clips each column to its cell budget.
    pub const fn new(name: &'a str, mode: &'a str, account: &'a str) -> Self {
        Self {
            name,
            mode,
            account,
            selected: false,
        }
    }

    /// Set whether the row carries the selected-row treatment.
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Borrowed Agents table content for the shared Settings renderer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsAgents<'a> {
    title: &'a str,
    right: &'a str,
    header: &'a str,
    rows: &'a [SettingsAgentRow<'a>],
    notes: &'a [&'a str],
}

impl<'a> SettingsAgents<'a> {
    /// Create an Agents table with a title, column header, rows, and fixed notes.
    pub const fn new(
        title: &'a str,
        right: &'a str,
        header: &'a str,
        rows: &'a [SettingsAgentRow<'a>],
        notes: &'a [&'a str],
    ) -> Self {
        Self {
            title,
            right,
            header,
            rows,
            notes,
        }
    }
}

/// One borrowed trust-source row for the shared Settings renderer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsTrustRow<'a> {
    source: &'a str,
    kind: SettingsTrustSourceKind,
    trusted: bool,
    dirty: bool,
    count: &'a str,
    selected: bool,
}

impl<'a> SettingsTrustRow<'a> {
    /// Create a trust row with explicit kind metadata and caller-owned count text.
    pub const fn new(
        source: &'a str,
        kind: SettingsTrustSourceKind,
        trusted: bool,
        count: &'a str,
    ) -> Self {
        Self {
            source,
            kind,
            trusted,
            dirty: false,
            count,
            selected: false,
        }
    }

    /// Set whether the trusted value differs from its saved value.
    pub const fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Set whether the row carries the selected-row treatment.
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Borrowed Trust table content for the shared Settings renderer.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsTrust<'a> {
    title: &'a str,
    right: &'a str,
    rows: &'a [SettingsTrustRow<'a>],
    notes: &'a [&'a str],
    preview: bool,
}

impl<'a> SettingsTrust<'a> {
    /// Create a Trust table with caller-owned summary and notes.
    pub const fn new(
        title: &'a str,
        right: &'a str,
        rows: &'a [SettingsTrustRow<'a>],
        notes: &'a [&'a str],
    ) -> Self {
        Self {
            title,
            right,
            rows,
            notes,
            preview: false,
        }
    }

    /// Use the plain selected gutter while a caller-owned preview layer is open.
    pub const fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }
}

/// Ink used for optional text after a Settings hint row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsHintOverflowTone {
    /// Use the faint text channel.
    Faint,
    /// Use the secondary text channel.
    Secondary,
}

/// Typed trailing text for the additive Agents and Trust painters.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsHintOverflow<'a> {
    text: &'a str,
    at: Option<u16>,
    tone: SettingsHintOverflowTone,
}

impl<'a> SettingsHintOverflow<'a> {
    /// Create trailing text that follows the final hint.
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            at: None,
            tone: SettingsHintOverflowTone::Faint,
        }
    }

    /// Place the text at a column offset from the page origin.
    pub const fn at(mut self, offset: u16) -> Self {
        self.at = Some(offset);
        self
    }

    /// Set the text channel.
    pub const fn tone(mut self, tone: SettingsHintOverflowTone) -> Self {
        self.tone = tone;
        self
    }
}

/// Semantic header and footer options for the additive Agents and Trust pages.
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct SettingsPageChrome<'a> {
    tab_note: Option<&'a str>,
    crumb_note: Option<&'a str>,
    hint_offsets: Option<&'a [u16]>,
    overflow: Option<SettingsHintOverflow<'a>>,
    hide_hints: bool,
}

impl<'a> SettingsPageChrome<'a> {
    /// Create default header and footer options.
    pub const fn new() -> Self {
        Self {
            tab_note: None,
            crumb_note: None,
            hint_offsets: None,
            overflow: None,
            hide_hints: false,
        }
    }

    /// Add a note to the selected tab and dim its underline.
    pub const fn tab_note(mut self, note: &'a str) -> Self {
        self.tab_note = Some(note);
        self
    }

    /// Add a styled note immediately after the breadcrumb.
    pub const fn crumb_note(mut self, note: &'a str) -> Self {
        self.crumb_note = Some(note);
        self
    }

    /// Place each `SettingsHint` at the matching page-relative column.
    pub const fn hint_offsets(mut self, offsets: &'a [u16]) -> Self {
        self.hint_offsets = Some(offsets);
        self
    }

    /// Add typed trailing text after the hints or at its explicit offset.
    pub const fn overflow(mut self, overflow: SettingsHintOverflow<'a>) -> Self {
        self.overflow = Some(overflow);
        self
    }

    /// Hide the standard hint row when another layer owns the footer.
    pub const fn hide_hints(mut self, hide: bool) -> Self {
        self.hide_hints = hide;
        self
    }
}

/// Failure to paint an Agents or Trust body inside the supplied page area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsBodyPaintError {
    /// Legacy sections or mounts would overlap the new table body.
    LegacyBodyNotEmpty,
    /// The page is too narrow for the Settings frame.
    AreaTooNarrow { minimum: u16, actual: u16 },
    /// The fixed body, actions, and footer cannot fit in the page height.
    AreaTooShort { minimum: u16, actual: u16 },
    /// The body exceeds its fixed row capacity.
    TooManyRows { maximum: usize, actual: usize },
    /// The fixed note area exceeds its row capacity.
    TooManyNotes { maximum: usize, actual: usize },
    /// Fixed hint offsets must have one entry per hint.
    HintOffsetsMismatch { hints: usize, offsets: usize },
    /// Legacy and typed overflow fields cannot both own the hint tail.
    ConflictingHintOverflow,
    /// A breadcrumb note needs a running-context cluster for its fixed anchor.
    CrumbNoteNeedsCluster,
}

impl fmt::Display for SettingsBodyPaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacyBodyNotEmpty => {
                f.write_str("legacy settings sections and mounts must be empty")
            }
            Self::AreaTooNarrow { minimum, actual } => {
                write!(
                    f,
                    "settings body needs {minimum} columns, received {actual}"
                )
            }
            Self::AreaTooShort { minimum, actual } => {
                write!(f, "settings body needs {minimum} rows, received {actual}")
            }
            Self::TooManyRows { maximum, actual } => {
                write!(f, "settings body allows {maximum} rows, received {actual}")
            }
            Self::TooManyNotes { maximum, actual } => {
                write!(f, "settings body allows {maximum} notes, received {actual}")
            }
            Self::HintOffsetsMismatch { hints, offsets } => write!(
                f,
                "settings hint layout has {offsets} offsets for {hints} hints"
            ),
            Self::ConflictingHintOverflow => {
                f.write_str("legacy and typed settings hint overflow cannot both be set")
            }
            Self::CrumbNoteNeedsCluster => {
                f.write_str("settings breadcrumb note needs a running-context cluster")
            }
        }
    }
}

impl std::error::Error for SettingsBodyPaintError {}

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
const SETTINGS_BODY_MIN_WIDTH: u16 = 40;
const SETTINGS_AGENT_MAX_ROWS: usize = 6;
const SETTINGS_TRUST_MAX_ROWS: usize = 4;
const SETTINGS_FIXED_NOTE_MAX_ROWS: usize = 2;

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

struct SettingsBodyBounds {
    maximum_rows: usize,
    row_floor: u16,
    row_start: u16,
}

#[derive(Clone, Copy)]
struct SettingsWordGapStyles {
    word: PaintStyle,
    gap: PaintStyle,
}

#[derive(Clone, Copy)]
struct SettingsTextRun {
    start: usize,
    end: usize,
    column: u16,
    columns: u16,
    style: PaintStyle,
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

/// Paint an Agents table with the existing Settings page chrome.
///
/// `page.sections`, `page.mounts`, and `page.add_mount` must be empty. The
/// renderer supports at most six rows and two fixed notes. It validates the
/// complete layout before it paints any cell. This function does not handle
/// input or focus.
pub fn paint_settings_page_with_agents(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    agents: &SettingsAgents<'_>,
    chrome: SettingsPageChrome<'_>,
) -> Result<(), SettingsBodyPaintError> {
    validate_settings_body(
        page,
        area,
        chrome,
        agents.rows.len(),
        agents.notes.len(),
        SettingsBodyBounds {
            maximum_rows: SETTINGS_AGENT_MAX_ROWS,
            row_floor: 7,
            row_start: 8,
        },
    )?;

    let ink = settings_ink(ui);
    paint_settings_body_frame(ui, area, page, chrome, &ink);
    paint_settings_agents(ui, area, agents, &ink);
    paint_settings_body_footer(ui, area, page, chrome, false, &ink);
    Ok(())
}

/// Paint only the fixed Agents content band within a Settings page area.
///
/// The band is page-relative rows 6 through 16, clipped to both `area` and the
/// current Ui clip. Unlike the full-page painter, this entrypoint does not draw
/// chrome or impose a minimum page height. Row and note capacities are checked
/// before any cell is changed.
pub fn paint_settings_agents_body(
    ui: &mut Ui<'_>,
    area: Rect,
    agents: &SettingsAgents<'_>,
) -> Result<(), SettingsBodyPaintError> {
    if agents.rows.len() > SETTINGS_AGENT_MAX_ROWS {
        return Err(SettingsBodyPaintError::TooManyRows {
            maximum: SETTINGS_AGENT_MAX_ROWS,
            actual: agents.rows.len(),
        });
    }
    if agents.notes.len() > SETTINGS_FIXED_NOTE_MAX_ROWS {
        return Err(SettingsBodyPaintError::TooManyNotes {
            maximum: SETTINGS_FIXED_NOTE_MAX_ROWS,
            actual: agents.notes.len(),
        });
    }

    let body = Rect::new(area.x, area.y.saturating_add(6), area.width, 11)
        .intersection(area)
        .intersection(ui.full());
    if body.is_empty() {
        return Ok(());
    }

    let ink = settings_ink(ui);
    ui.with_area(body, |ui| {
        ui.fill(body, plain(ink.primary, ink.canvas));
        paint_settings_agents(ui, area, agents, &ink);
    });
    Ok(())
}

/// Paint only the fixed Trust content band within a Settings page area.
///
/// The band is page-relative rows 6 through 13, clipped to both `area` and
/// the current Ui clip. Unlike the full-page painter, this entrypoint does
/// not draw chrome or impose a minimum page height. Row and note capacities
/// are checked before any cell is changed. This function does not handle
/// input, counts, or a caller-owned save preview.
///
/// # Errors
///
/// Returns [`SettingsBodyPaintError::TooManyRows`] or
/// [`SettingsBodyPaintError::TooManyNotes`] when the input exceeds the fixed
/// Trust body capacities. Either error is returned before painting.
pub fn paint_settings_trust_body(
    ui: &mut Ui<'_>,
    area: Rect,
    trust: &SettingsTrust<'_>,
) -> Result<(), SettingsBodyPaintError> {
    if trust.rows.len() > SETTINGS_TRUST_MAX_ROWS {
        return Err(SettingsBodyPaintError::TooManyRows {
            maximum: SETTINGS_TRUST_MAX_ROWS,
            actual: trust.rows.len(),
        });
    }
    if trust.notes.len() > SETTINGS_FIXED_NOTE_MAX_ROWS {
        return Err(SettingsBodyPaintError::TooManyNotes {
            maximum: SETTINGS_FIXED_NOTE_MAX_ROWS,
            actual: trust.notes.len(),
        });
    }

    let body = Rect::new(area.x, area.y.saturating_add(6), area.width, 8)
        .intersection(area)
        .intersection(ui.full());
    if body.is_empty() {
        return Ok(());
    }

    let ink = settings_ink(ui);
    ui.with_area(body, |ui| {
        ui.fill(body, plain(ink.primary, ink.canvas));
        paint_settings_trust(ui, area, trust, &ink);
    });
    Ok(())
}

/// Paint only the borrowed Mounts table within a Settings page area.
///
/// The table starts at page-relative row 6 and is clipped to the supplied
/// page area, the available body band above the native footer, and the current
/// Ui clip. It paints the shared Mounts columns and rows without page chrome,
/// actions, hints, input handling, or focus ownership.
pub fn paint_settings_mounts_body(
    ui: &mut Ui<'_>,
    area: Rect,
    mounts: &[SettingsMount<'_>],
    add_mount: Option<&str>,
) {
    if mounts.is_empty() {
        return;
    }
    let mount_rows = u16::try_from(mounts.len()).unwrap_or(u16::MAX);
    let add_row = if add_mount.is_some() { 1 } else { 0 };
    let content_height = 1u16.saturating_add(mount_rows).saturating_add(add_row);
    let body_height = content_height.min(area.height.saturating_sub(9));
    let body = Rect::new(area.x, area.y.saturating_add(6), area.width, body_height)
        .intersection(area)
        .intersection(ui.full());
    if body.is_empty() {
        return;
    }

    let ink = settings_ink(ui);
    ui.with_area(body, |ui| {
        ui.fill(body, plain(ink.primary, ink.canvas));
        paint_mount_rows(ui, area, mounts, add_mount, &ink);
    });
}

/// Paint a Trust table with the existing Settings page chrome.
///
/// `page.sections`, `page.mounts`, and `page.add_mount` must be empty. The
/// renderer supports at most four rows and two fixed notes. It validates the
/// complete layout before it paints any cell. This function does not handle
/// input, counts, or a caller-owned save preview.
pub fn paint_settings_page_with_trust(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    trust: &SettingsTrust<'_>,
    chrome: SettingsPageChrome<'_>,
) -> Result<(), SettingsBodyPaintError> {
    validate_settings_body(
        page,
        area,
        chrome,
        trust.rows.len(),
        trust.notes.len(),
        SettingsBodyBounds {
            maximum_rows: SETTINGS_TRUST_MAX_ROWS,
            row_floor: 6,
            row_start: 7,
        },
    )?;

    let ink = settings_ink(ui);
    paint_settings_body_frame(ui, area, page, chrome, &ink);
    paint_settings_trust(ui, area, trust, &ink);
    paint_settings_body_footer(ui, area, page, chrome, trust.preview, &ink);
    Ok(())
}

fn validate_settings_body(
    page: &SettingsPage<'_>,
    area: Rect,
    chrome: SettingsPageChrome<'_>,
    row_count: usize,
    note_count: usize,
    bounds: SettingsBodyBounds,
) -> Result<(), SettingsBodyPaintError> {
    if !page.sections.is_empty() || !page.mounts.is_empty() || page.add_mount.is_some() {
        return Err(SettingsBodyPaintError::LegacyBodyNotEmpty);
    }
    if area.width < SETTINGS_BODY_MIN_WIDTH {
        return Err(SettingsBodyPaintError::AreaTooNarrow {
            minimum: SETTINGS_BODY_MIN_WIDTH,
            actual: area.width,
        });
    }
    if row_count > bounds.maximum_rows {
        return Err(SettingsBodyPaintError::TooManyRows {
            maximum: bounds.maximum_rows,
            actual: row_count,
        });
    }
    if note_count > SETTINGS_FIXED_NOTE_MAX_ROWS {
        return Err(SettingsBodyPaintError::TooManyNotes {
            maximum: SETTINGS_FIXED_NOTE_MAX_ROWS,
            actual: note_count,
        });
    }
    if let Some(offsets) = chrome.hint_offsets
        && offsets.len() != page.hints.len()
    {
        return Err(SettingsBodyPaintError::HintOffsetsMismatch {
            hints: page.hints.len(),
            offsets: offsets.len(),
        });
    }
    if page.hint_overflow.is_some() && chrome.overflow.is_some() {
        return Err(SettingsBodyPaintError::ConflictingHintOverflow);
    }
    if chrome.crumb_note.is_some() && page.cluster.is_none() {
        return Err(SettingsBodyPaintError::CrumbNoteNeedsCluster);
    }

    let last_row = if row_count == 0 {
        bounds.row_floor
    } else {
        bounds
            .row_start
            .saturating_add(u16::try_from(row_count.saturating_sub(1)).unwrap_or(u16::MAX))
    };
    let note_start = bounds
        .row_floor
        .saturating_add(u16::try_from(bounds.maximum_rows).unwrap_or(u16::MAX))
        .saturating_add(2);
    let last_note = if note_count == 0 {
        0
    } else {
        note_start.saturating_add(u16::try_from(note_count.saturating_sub(1)).unwrap_or(u16::MAX))
    };
    let last_content = last_row.max(last_note);
    let minimum_height = last_content.saturating_add(4).max(8);
    if area.height < minimum_height {
        return Err(SettingsBodyPaintError::AreaTooShort {
            minimum: minimum_height,
            actual: area.height,
        });
    }
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

fn paint_settings_body_frame(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    chrome: SettingsPageChrome<'_>,
    ink: &Ink,
) {
    ui.fill(area, plain(ink.primary, ink.canvas));
    if page.cluster.is_some() {
        paint_anchored_header(ui, area, page, chrome, ink);
    } else {
        paint_header(ui, area, page, ink);
    }
    let (selected_at, selected_span) = paint_tabs_with_note(ui, area, page, chrome.tab_note, ink);
    paint_underline_with_note(
        ui,
        area,
        selected_at,
        selected_span,
        chrome.tab_note.is_some(),
        ink,
    );
}

fn paint_settings_footer(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
    paint_actions(ui, area, page, ink);
    paint_hints(ui, area, page, ink);
}

fn paint_settings_body_footer(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    chrome: SettingsPageChrome<'_>,
    preview: bool,
    ink: &Ink,
) {
    paint_actions_with_preview(ui, area, page, preview, ink);
    if !chrome.hide_hints {
        paint_settings_hints(ui, area, page, chrome, ink);
    }
}

fn paint_settings_agents(ui: &mut Ui<'_>, area: Rect, agents: &SettingsAgents<'_>, ink: &Ink) {
    let y0 = area.y.saturating_add(6);
    put(
        ui,
        area,
        area.x.saturating_add(4),
        y0,
        agents.title,
        bold(ink.secondary, ink.canvas),
    );
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
        let row_offset = u16::try_from(index).unwrap_or(u16::MAX);
        let y = y0.saturating_add(2).saturating_add(row_offset);
        paint_settings_agent_row(ui, area, y, row, ink);
    }
    let notes_y = area.y.saturating_add(15);
    for (index, note) in agents.notes.iter().enumerate() {
        let note_offset = u16::try_from(index).unwrap_or(u16::MAX);
        put(
            ui,
            area,
            area.x.saturating_add(4),
            notes_y.saturating_add(note_offset),
            note,
            plain(ink.faint, ink.canvas),
        );
    }
}

fn paint_settings_agent_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &SettingsAgentRow<'_>,
    ink: &Ink,
) {
    let bg = if row.selected { SELECT_BG } else { ink.canvas };
    if row.selected {
        paint_settings_selected_fill(ui, area, y, false, ink);
    } else {
        paint_settings_gutter(ui, area, y, ink);
    }
    put_cols(
        ui,
        area,
        area.x.saturating_add(5),
        y,
        " ",
        1,
        if row.selected {
            bold(AMBER, bg)
        } else {
            plain(AMBER, bg)
        },
    );
    put_cols(
        ui,
        area,
        area.x.saturating_add(6),
        y,
        row.name,
        15,
        if row.selected {
            bold(ink.primary, bg)
        } else {
            plain(ink.primary, bg)
        },
    );
    put_cols(
        ui,
        area,
        area.x.saturating_add(21),
        y,
        row.mode,
        13,
        if row.selected {
            bold(ink.secondary, bg)
        } else {
            plain(ink.secondary, bg)
        },
    );
    let account_x = area.x.saturating_add(34);
    let account_width = area.right().saturating_sub(account_x);
    if row.selected {
        paint_settings_word_gap_runs(
            ui,
            area,
            account_x,
            y,
            row.account,
            account_width,
            SettingsWordGapStyles {
                word: bold(ink.secondary, bg),
                gap: bold(ink.muted, bg),
            },
        );
    } else {
        put_cols(
            ui,
            area,
            account_x,
            y,
            row.account,
            account_width,
            plain(ink.muted, bg),
        );
    }
}

fn paint_settings_trust(ui: &mut Ui<'_>, area: Rect, trust: &SettingsTrust<'_>, ink: &Ink) {
    let title_y = area.y.saturating_add(6);
    put(
        ui,
        area,
        area.x.saturating_add(4),
        title_y,
        trust.title,
        bold(ink.secondary, ink.canvas),
    );
    put(
        ui,
        area,
        area.right()
            .saturating_sub(4)
            .saturating_sub(width(trust.right)),
        title_y,
        trust.right,
        plain(ink.faint, ink.canvas),
    );
    for (index, row) in trust.rows.iter().enumerate() {
        let row_offset = u16::try_from(index).unwrap_or(u16::MAX);
        let y = area.y.saturating_add(7).saturating_add(row_offset);
        paint_settings_trust_row(ui, area, y, row, trust.preview, ink);
    }
    let notes_y = area.y.saturating_add(12);
    for (index, note) in trust.notes.iter().enumerate() {
        let note_offset = u16::try_from(index).unwrap_or(u16::MAX);
        put(
            ui,
            area,
            area.x.saturating_add(4),
            notes_y.saturating_add(note_offset),
            note,
            plain(ink.faint, ink.canvas),
        );
    }
}

fn paint_settings_trust_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &SettingsTrustRow<'_>,
    preview: bool,
    ink: &Ink,
) {
    let bg = if row.selected { SELECT_BG } else { ink.canvas };
    if row.selected {
        paint_settings_selected_fill(ui, area, y, preview, ink);
    } else {
        paint_settings_gutter(ui, area, y, ink);
    }
    put_cols(
        ui,
        area,
        area.x.saturating_add(5),
        y,
        if row.dirty { "•" } else { " " },
        1,
        if row.selected {
            bold(AMBER, bg)
        } else {
            plain(AMBER, bg)
        },
    );
    let source_style = if row.selected {
        bold(ink.primary, bg)
    } else {
        plain(ink.primary, bg)
    };
    put_cols(ui, area, area.x.saturating_add(6), y, " ", 1, source_style);
    put_cols(
        ui,
        area,
        area.x.saturating_add(7),
        y,
        row.source,
        42,
        source_style,
    );

    let kind = match row.kind {
        SettingsTrustSourceKind::Git => "git  ",
        SettingsTrustSourceKind::Path => "path ",
        SettingsTrustSourceKind::Unknown => "unk  ",
    };
    let kind_x = area.x.saturating_add(49);
    if row.selected {
        paint_settings_word_gap_runs(
            ui,
            area,
            kind_x,
            y,
            kind,
            5,
            SettingsWordGapStyles {
                word: bold(ink.secondary, bg),
                gap: bold(ink.muted, bg),
            },
        );
    } else {
        put_cols(ui, area, kind_x, y, kind, 5, plain(ink.muted, bg));
    }

    if row.trusted {
        put_cols(
            ui,
            area,
            area.x.saturating_add(54),
            y,
            " [✓] trusted    ",
            16,
            if row.selected {
                bold(ink.primary, bg)
            } else {
                plain(ink.primary, bg)
            },
        );
    } else {
        put_cols(
            ui,
            area,
            area.x.saturating_add(55),
            y,
            "[ ] untrusted",
            13,
            plain(AMBER, ink.canvas),
        );
    }

    let count_x = area.x.saturating_add(70);
    let count_width = area.right().saturating_sub(count_x);
    if row.selected {
        paint_settings_word_gap_runs(
            ui,
            area,
            count_x,
            y,
            row.count,
            count_width,
            SettingsWordGapStyles {
                word: bold(ink.secondary, bg),
                gap: bold(ink.faint, bg),
            },
        );
    } else {
        put_cols(
            ui,
            area,
            count_x,
            y,
            row.count,
            count_width,
            plain(ink.faint, bg),
        );
    }
}

fn paint_settings_selected_fill(ui: &mut Ui<'_>, area: Rect, y: u16, preview: bool, ink: &Ink) {
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
        put_cols(
            ui,
            area,
            area.x.saturating_add(2),
            y,
            " ",
            1,
            plain(SELECT_BG, SELECT_BG),
        );
        put_cols(
            ui,
            area,
            area.x.saturating_add(3),
            y,
            "›",
            1,
            bold(ink.accent, SELECT_BG),
        );
    } else {
        put_cols(
            ui,
            area,
            area.x.saturating_add(2),
            y,
            "▎›",
            2,
            bold(ink.accent, SELECT_BG),
        );
    }
}

fn paint_settings_gutter(ui: &mut Ui<'_>, area: Rect, y: u16, ink: &Ink) {
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
        area.x.saturating_add(3),
        y,
        " ",
        1,
        plain(ink.secondary, ink.canvas),
    );
}

fn paint_settings_word_gap_runs(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    columns: u16,
    styles: SettingsWordGapStyles,
) {
    let visible = area.right().saturating_sub(x).min(columns);
    if visible == 0 {
        return;
    }

    let mut run_start = 0_usize;
    let mut run_end = 0_usize;
    let mut run_column = 0_u16;
    let mut run_width = 0_u16;
    let mut run_gap: Option<bool> = None;
    let mut used = 0_u16;
    for (byte, grapheme) in graphemes(text) {
        let grapheme_width = grapheme_width(grapheme);
        if used.saturating_add(grapheme_width) > visible {
            break;
        }
        let gap = grapheme == " ";
        if run_gap.is_some_and(|current| current != gap) {
            paint_settings_run(
                ui,
                area,
                x,
                y,
                text,
                SettingsTextRun {
                    start: run_start,
                    end: run_end,
                    column: run_column,
                    columns: run_width,
                    style: if run_gap == Some(true) {
                        styles.gap
                    } else {
                        styles.word
                    },
                },
            );
            run_start = byte;
            run_column = used;
            run_width = 0;
        } else if run_gap.is_none() {
            run_start = byte;
            run_column = used;
        }
        run_gap = Some(gap);
        run_end = byte.saturating_add(grapheme.len());
        run_width = run_width.saturating_add(grapheme_width);
        used = used.saturating_add(grapheme_width);
    }
    if run_gap.is_some() {
        paint_settings_run(
            ui,
            area,
            x,
            y,
            text,
            SettingsTextRun {
                start: run_start,
                end: run_end,
                column: run_column,
                columns: run_width,
                style: if run_gap == Some(true) {
                    styles.gap
                } else {
                    styles.word
                },
            },
        );
    }
}

fn paint_settings_run(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    run: SettingsTextRun,
) {
    if let Some(run_text) = text.get(run.start..run.end) {
        put_cols(
            ui,
            area,
            x.saturating_add(run.column),
            y,
            run_text,
            run.columns,
            run.style,
        );
    }
}

fn paint_settings_hints(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    chrome: SettingsPageChrome<'_>,
    ink: &Ink,
) {
    let y = area.bottom().saturating_sub(1);
    let mut next_x = area.x.saturating_add(1);
    let key_style = bold(ink.primary, ink.canvas);
    let action_style = plain(ink.muted, ink.canvas);
    for (index, hint) in page.hints.iter().enumerate() {
        let x = chrome
            .hint_offsets
            .and_then(|offsets| offsets.get(index).copied())
            .map_or(next_x, |offset| area.x.saturating_add(offset));
        put(ui, area, x, y, hint.key, key_style);
        let action_x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, action_x, y, hint.action, action_style);
        next_x = action_x
            .saturating_add(width(hint.action))
            .saturating_add(2);
    }

    if let Some(overflow) = chrome.overflow {
        let x = overflow
            .at
            .map_or(next_x, |offset| area.x.saturating_add(offset));
        let fg = match overflow.tone {
            SettingsHintOverflowTone::Faint => ink.faint,
            SettingsHintOverflowTone::Secondary => ink.secondary,
        };
        put(ui, area, x, y, overflow.text, plain(fg, ink.canvas));
    } else if let Some(overflow) = page.hint_overflow {
        put(ui, area, next_x, y, overflow, plain(ink.faint, ink.canvas));
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
    paint_header_brand_and_menus(ui, area, page, ink);
    let y = area.y;
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

fn paint_header_brand_and_menus(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) {
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
}

fn paint_anchored_header(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    chrome: SettingsPageChrome<'_>,
    ink: &Ink,
) {
    paint_header_brand_and_menus(ui, area, page, ink);
    let Some(cluster) = page.cluster else {
        return;
    };
    let y = area.y;
    let secondary = plain(ink.secondary, ink.canvas);
    let crumb_x = match chrome.crumb_note {
        Some(note) => area
            .x
            .saturating_add(85)
            .saturating_sub(width(page.crumb).saturating_add(width(note))),
        None => area.x.saturating_add(83).saturating_sub(width(page.crumb)),
    };
    put(ui, area, crumb_x, y, page.crumb, secondary);
    if let Some(note) = chrome.crumb_note {
        put(
            ui,
            area,
            crumb_x.saturating_add(width(page.crumb)),
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
    let running = format!(" {}", cluster.value);
    put(ui, area, area.x.saturating_add(108), y, &running, secondary);
}

fn tab_span(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_tabs(ui: &mut Ui<'_>, area: Rect, page: &SettingsPage<'_>, ink: &Ink) -> (u16, u16) {
    paint_tabs_with_note(ui, area, page, None, ink)
}

fn paint_tabs_with_note(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    tab_note: Option<&str>,
    ink: &Ink,
) -> (u16, u16) {
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
            if let Some(note) = tab_note {
                let noted = format!(" {} {note}  ", tab.label);
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

fn paint_underline(ui: &mut Ui<'_>, area: Rect, selected_at: u16, selected_span: u16, ink: &Ink) {
    paint_underline_with_note(ui, area, selected_at, selected_span, false, ink);
}

fn paint_underline_with_note(
    ui: &mut Ui<'_>,
    area: Rect,
    selected_at: u16,
    selected_span: u16,
    seam_heavy: bool,
    ink: &Ink,
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

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);

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
    paint_mount_rows(ui, area, page.mounts, page.add_mount, ink);
}

fn paint_mount_rows(
    ui: &mut Ui<'_>,
    area: Rect,
    mounts: &[SettingsMount<'_>],
    add_mount: Option<&str>,
    ink: &Ink,
) {
    if mounts.is_empty() {
        return;
    }
    let cols = if mounts.iter().any(|row| row.kind.is_some()) {
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
    for (i, row) in mounts.iter().enumerate() {
        let y = y0.saturating_add(1).saturating_add(i as u16);
        paint_mount_row(ui, area, y, row, ink, &cols);
    }
    if let Some(add) = add_mount {
        let y = y0.saturating_add(1).saturating_add(mounts.len() as u16);
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
    paint_actions_with_preview(ui, area, page, false, ink);
}

fn paint_actions_with_preview(
    ui: &mut Ui<'_>,
    area: Rect,
    page: &SettingsPage<'_>,
    preview: bool,
    ink: &Ink,
) {
    let y = area.bottom().saturating_sub(3);
    let save = format!("{} ", page.save);
    let cancel = format!("{} ", page.cancel);
    let save_x = area.right().saturating_sub(5).saturating_sub(width(&save));
    let save_ink = if preview {
        ink.secondary
    } else if page.save_enabled {
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
    use termrock::{ColorLevel, FgStep, Surface, Theme};
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

    fn heritage_page<'a>(
        crumb: &'a str,
        tabs: &'a [SettingsTab<'a>],
        hints: &'a [SettingsHint<'a>],
        cluster: Option<SettingsCluster<'a>>,
    ) -> SettingsPage<'a> {
        SettingsPage {
            brand: "jackin❯",
            menus: &[],
            crumb,
            tabs,
            sections: &[],
            cancel: "Cancel",
            save: "Save…",
            save_enabled: false,
            hints,
            hint_overflow: None,
            mounts: &[],
            add_mount: None,
            cluster,
        }
    }

    #[test]
    fn mounts_body_respects_nested_clip_and_short_content_band() {
        let rows = [SettingsMount {
            path: "/workspace/資料/long-project-name",
            scope: "global",
            mode: "ro",
            isolation: "shared",
            kind: Some("host"),
            source: Some("~/sources/資料/project"),
            selected: true,
        }];
        let mut wide = Scene::new(
            "settings_mounts_body_wide_grapheme_values",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            24,
        );
        wide.draw(|ui, area| {
            paint_settings_mounts_body(ui, area, &rows, Some("+ Add mount"));
        });
        assert!(wide.text().contains("/workspace/資料/long-project-na"));
        assert!(wide.text().contains("~/sources/資料/project"));

        let sentinel = plain(Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6));
        let mut clipped = Scene::new(
            "settings_mounts_body_nested_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            80,
            20,
        );
        clipped.draw(|ui, area| {
            ui.paint_cell(Position::new(3, 7), "!", sentinel);
            ui.paint_cell(Position::new(50, 7), "!", sentinel);
            ui.paint_cell(Position::new(4, 9), "!", sentinel);
            ui.with_area(Rect::new(4, 6, 40, 3), |ui| {
                paint_settings_mounts_body(ui, area, &rows, Some("+ Add mount"));
            });
        });
        assert_eq!(
            clipped.buffer().cell((3, 7)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            clipped.buffer().cell((50, 7)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            clipped.buffer().cell((4, 9)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            clipped.buffer().cell((8, 6)).map(|cell| cell.symbol()),
            Some("D")
        );
        assert_eq!(
            clipped.buffer().cell((8, 7)).map(|cell| cell.symbol()),
            Some("/")
        );

        let mut offset = Scene::new(
            "settings_mounts_body_offset_area",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            30,
        );
        offset.draw(|ui, _| {
            ui.paint_cell(Position::new(9, 9), "!", sentinel);
            ui.paint_cell(Position::new(70, 9), "!", sentinel);
            ui.paint_cell(Position::new(10, 11), "!", sentinel);
            ui.with_area(Rect::new(10, 8, 60, 5), |ui| {
                paint_settings_mounts_body(ui, Rect::new(5, 2, 80, 20), &rows, Some("+ Add mount"));
            });
        });
        assert_eq!(
            offset.buffer().cell((9, 9)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            offset.buffer().cell((70, 9)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            offset.buffer().cell((10, 11)).map(|cell| cell.symbol()),
            Some("!")
        );
        assert_eq!(
            offset.buffer().cell((13, 8)).map(|cell| cell.symbol()),
            Some("D")
        );
        assert_eq!(
            offset.buffer().cell((14, 9)).map(|cell| cell.symbol()),
            Some("/")
        );

        let mut short = Scene::new(
            "settings_mounts_body_short_page_area",
            Theme::junie(),
            ColorLevel::TrueColor,
            80,
            20,
        );
        short.draw(|ui, _| {
            ui.paint_cell(Position::new(4, 8), "!", sentinel);
            paint_settings_mounts_body(ui, Rect::new(0, 0, 80, 11), &rows, Some("+ Add mount"));
        });
        assert_eq!(
            short.buffer().cell((8, 6)).map(|cell| cell.symbol()),
            Some("D")
        );
        assert_eq!(
            short.buffer().cell((8, 7)).map(|cell| cell.symbol()),
            Some("/")
        );
        assert_eq!(
            short.buffer().cell((4, 8)).map(|cell| cell.symbol()),
            Some("!")
        );
    }

    #[test]
    fn mounts_body_uses_semantic_theme_palette_at_every_capability() {
        fn column(scene: &Scene, y: u16, x: u16, width: u16) -> String {
            let mut text = String::new();
            for column in x..x.saturating_add(width) {
                text.push_str(
                    scene
                        .buffer()
                        .cell((column, y))
                        .expect("Mounts body cell")
                        .symbol(),
                );
            }
            text.trim().to_owned()
        }

        let rows = [SettingsMount {
            path: "/home/agent/.gitconfig",
            scope: "global",
            mode: "ro",
            isolation: "shared",
            kind: Some("host"),
            source: Some("~/.gitconfig"),
            selected: true,
        }];
        for width in [80, 120] {
            for level in [
                ColorLevel::TrueColor,
                ColorLevel::Ansi256,
                ColorLevel::Ansi16,
                ColorLevel::Mono,
            ] {
                let mut scene = Scene::new(
                    "settings_mounts_body_semantic_palette",
                    Theme::junie(),
                    level,
                    width,
                    24,
                );
                scene.draw(|ui, area| {
                    paint_settings_mounts_body(ui, area, &rows, Some("+ Add mount"));
                });

                let resolved = Theme::junie().downgrade(level);
                let heading = scene
                    .buffer()
                    .cell((8, 6))
                    .expect("shared Destination heading");
                assert_eq!(
                    heading.fg,
                    resolved.color.fg[FgStep::Muted.index()],
                    "header uses the Muted semantic role at {level:?}"
                );
                assert_eq!(heading.bg, resolved.bg(Surface::Canvas));
                assert_eq!(column(&scene, 7, 72, 6), "host");
                assert_eq!(column(&scene, 8, 8, 11), "+ Add mount");
                if width >= 120 {
                    assert_eq!(column(&scene, 7, 80, 12), "~/.gitconfig");
                }
            }
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
    fn agents_and_trust_painters_keep_fixed_heritage_cells_and_preview_options() {
        let agent_tabs = [SettingsTab {
            label: "Agents",
            selected: true,
        }];
        let agent_hints = [SettingsHint {
            key: "Space",
            action: "Cycle mode",
        }];
        let agent_offsets = [1];
        let cluster = SettingsCluster {
            note: "inside the Construct",
            mark: "●",
            value: "2 running",
        };
        let agent_page = heritage_page(
            "Settings › global › Agents",
            &agent_tabs,
            &agent_hints,
            Some(cluster),
        );
        let agent_rows = [
            SettingsAgentRow::new("Claude Code", "sync", "Claude · Personal").selected(true),
            SettingsAgentRow::new("Codex", "api key", "Codex · Primary"),
        ];
        let agent_notes = [
            "sync mirrors the host login",
            "ignore withholds credentials",
        ];
        let agents = SettingsAgents::new(
            "Agent runtime mode",
            "accounts are registered in Accounts (c)",
            "Agent         Mode         Registry default",
            &agent_rows,
            &agent_notes,
        );
        let mut agent_scene = Scene::new(
            "settings_agents_shared_painter",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            40,
        );
        agent_scene.draw(|ui, area| {
            assert_eq!(
                paint_settings_page_with_agents(
                    ui,
                    area,
                    &agent_page,
                    &agents,
                    SettingsPageChrome::new()
                        .hint_offsets(&agent_offsets)
                        .overflow(
                            SettingsHintOverflow::new("…")
                                .at(112)
                                .tone(SettingsHintOverflowTone::Secondary),
                        ),
                ),
                Ok(())
            );
        });
        let agent_text = agent_scene.text();
        for expected in [
            "Settings › global › Agents",
            "inside the Construct",
            "Agent runtime mode",
            "Claude Code",
            "sync mirrors the host login",
            "Cycle mode",
        ] {
            assert!(agent_text.contains(expected), "missing {expected:?}");
        }
        assert_eq!(
            agent_scene.buffer().cell((83, 0)).map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_eq!(
            agent_scene.buffer().cell((85, 0)).map(|cell| cell.symbol()),
            Some("i")
        );
        assert_eq!(
            agent_scene
                .buffer()
                .cell((107, 0))
                .map(|cell| cell.symbol()),
            Some("●")
        );
        assert_eq!(
            agent_scene
                .buffer()
                .cell((108, 0))
                .map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_eq!(
            agent_scene.buffer().cell((2, 8)).map(|cell| cell.symbol()),
            Some("▎")
        );
        assert_eq!(
            agent_scene.buffer().cell((4, 15)).map(|cell| cell.symbol()),
            Some("s")
        );
        assert_eq!(
            agent_scene.buffer().cell((1, 39)).map(|cell| cell.symbol()),
            Some("S")
        );
        assert_eq!(
            agent_scene
                .buffer()
                .cell((112, 39))
                .map(|cell| cell.symbol()),
            Some("…")
        );

        let trust_tabs = [SettingsTab {
            label: "Trust",
            selected: true,
        }];
        let trust_hints = [SettingsHint {
            key: "Space",
            action: "Toggle trust",
        }];
        let trust_offsets = [1];
        let trust_page = heritage_page(
            "Settings › global › Trust",
            &trust_tabs,
            &trust_hints,
            Some(cluster),
        );
        let trust_rows = [SettingsTrustRow::new(
            "relative/roles",
            SettingsTrustSourceKind::Unknown,
            false,
            "2 roles",
        )
        .dirty(true)
        .selected(true)];
        let trust_notes = ["The selected row keeps its verified kind."];
        let trust = SettingsTrust::new(
            "Role sources",
            "0 trusted · 1 untrusted",
            &trust_rows,
            &trust_notes,
        )
        .preview(true);
        let chrome = SettingsPageChrome::new()
            .tab_note("•")
            .crumb_note("  • 1 change  ")
            .hint_offsets(&trust_offsets)
            .hide_hints(true);
        let mut trust_scene = Scene::new(
            "settings_trust_shared_painter",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            40,
        );
        trust_scene.draw(|ui, area| {
            assert_eq!(
                paint_settings_page_with_trust(ui, area, &trust_page, &trust, chrome),
                Ok(())
            );
        });
        let trust_text = trust_scene.text();
        for expected in [
            "Settings › global › Trust",
            "• 1 change",
            "Role sources",
            "relative/roles",
            "unk",
            "2 roles",
            "The selected row keeps its verified kind.",
        ] {
            assert!(trust_text.contains(expected), "missing {expected:?}");
        }
        assert!(!trust_text.contains("Toggle trust"));
        assert_eq!(
            trust_scene.buffer().cell((2, 7)).map(|cell| cell.symbol()),
            Some(" ")
        );
        assert_eq!(
            trust_scene.buffer().cell((3, 7)).map(|cell| cell.symbol()),
            Some("›")
        );
        assert_eq!(
            trust_scene.buffer().cell((49, 7)).map(|cell| cell.symbol()),
            Some("u")
        );
        assert_eq!(
            trust_scene.buffer().cell((5, 7)).map(|cell| cell.symbol()),
            Some("•")
        );
    }

    #[test]
    fn body_renderer_clips_wide_text_to_offset_area_and_keeps_sentinels_outside() {
        let tabs = [SettingsTab {
            label: "Trust",
            selected: true,
        }];
        let hints: [SettingsHint<'_>; 0] = [];
        let page = heritage_page("Settings › global › Trust", &tabs, &hints, None);
        let source = "界".repeat(100);
        let count = "界 roles ".repeat(1000);
        let rows = [
            SettingsTrustRow::new(&source, SettingsTrustSourceKind::Unknown, true, &count)
                .selected(true),
        ];
        let notes = ["note e\u{301} clips at the supplied page edge"];
        let trust = SettingsTrust::new("Role sources", "1 trusted · 0 untrusted", &rows, &notes);
        let area = Rect::new(4, 2, 120, 40);
        let mut scene = Scene::new(
            "settings_trust_offset_unicode_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            130,
            46,
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
                    paint_settings_page_with_trust(
                        ui,
                        area,
                        &page,
                        &trust,
                        SettingsPageChrome::new(),
                    ),
                    Ok(())
                );
            },
        );
        assert!(scene.text().contains("界界"));
        assert!(scene.text().contains("unk"));
        for pos in scene.area().positions() {
            if !area.contains(pos) {
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
    fn trust_body_matches_the_shared_page_table_at_narrow_and_wide_sizes() {
        let rows = [
            SettingsTrustRow::new(
                "github.com/chainargos/roles",
                SettingsTrustSourceKind::Git,
                true,
                "4 roles",
            )
            .selected(true),
            SettingsTrustRow::new(
                "github.com/acme-labs/roles-experimental",
                SettingsTrustSourceKind::Git,
                false,
                "1 role",
            ),
            SettingsTrustRow::new("~/roles", SettingsTrustSourceKind::Path, true, "1 role")
                .dirty(true),
            SettingsTrustRow::new(
                "git@corp:infra/roles",
                SettingsTrustSourceKind::Git,
                true,
                "2 roles",
            ),
        ];
        let notes = [
            "An untrusted source blocks + Load role until it is trusted here or",
            "in the trust dialog that the load opens.",
        ];
        let tabs = [SettingsTab {
            label: "Trust",
            selected: true,
        }];
        let hints = [SettingsHint {
            key: "Space",
            action: "Toggle trust",
        }];

        for preview in [false, true] {
            let trust =
                SettingsTrust::new("Role sources", "3 trusted · 1 untrusted", &rows, &notes)
                    .preview(preview);
            let page = heritage_page("Settings › global › Trust", &tabs, &hints, None);
            for (width, height) in [(80, 24), (120, 40)] {
                let mut full = Scene::new(
                    "settings_trust_body_full_page_reference",
                    Theme::junie(),
                    ColorLevel::TrueColor,
                    width,
                    height,
                );
                full.draw(|ui, area| {
                    assert_eq!(
                        paint_settings_page_with_trust(
                            ui,
                            area,
                            &page,
                            &trust,
                            SettingsPageChrome::new(),
                        ),
                        Ok(())
                    );
                });

                let mut body = Scene::new(
                    "settings_trust_body_only",
                    Theme::junie(),
                    ColorLevel::TrueColor,
                    width,
                    height,
                );
                body.draw(|ui, area| {
                    assert_eq!(paint_settings_trust_body(ui, area, &trust), Ok(()));
                });

                for y in 6..14 {
                    for x in 0..width {
                        let expected = full.buffer().cell((x, y)).expect("full-page cell");
                        let actual = body.buffer().cell((x, y)).expect("body-only cell");
                        assert_eq!(
                            format!("{actual:?}"),
                            format!("{expected:?}"),
                            "native cell changed at {width}x{height} ({x}, {y}), preview={preview}"
                        );
                    }
                }
                assert!(body.text().contains("github.com/chainargos/roles"));
                assert!(body.text().contains("trust dialog that the load opens"));
                let gutter = body.buffer().cell((2, 7)).expect("selected gutter");
                if preview {
                    assert_eq!(gutter.symbol(), " ");
                } else {
                    assert_eq!(gutter.symbol(), "▎");
                }
            }
        }
    }

    #[test]
    fn trust_body_respects_offset_nested_clip_and_short_area() {
        use ratatui_core::style::{Modifier, Style};
        use std::cell::RefCell;

        let rows = [
            SettingsTrustRow::new("source-one", SettingsTrustSourceKind::Git, true, "2 roles")
                .selected(true),
            SettingsTrustRow::new("source-two", SettingsTrustSourceKind::Path, false, "1 role"),
        ];
        let notes: [&str; 0] = [];
        let trust = SettingsTrust::new("Role sources", "1 trusted · 1 untrusted", &rows, &notes);
        let page = Rect::new(4, 2, 100, 10);
        let clip = Rect::new(10, 8, 30, 2);
        let sentinel_style = Style::default()
            .fg(Color::Rgb(0x11, 0x22, 0x33))
            .bg(Color::Rgb(0x44, 0x55, 0x66))
            .add_modifier(Modifier::ITALIC);
        let sentinel = RefCell::new(None);
        let mut scene = Scene::new(
            "settings_trust_body_nested_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            20,
        );
        scene.draw_over(
            |buffer| {
                buffer.set_style(Rect::new(0, 0, 120, 20), sentinel_style);
                *sentinel.borrow_mut() =
                    Some(format!("{:?}", buffer.cell((0, 0)).expect("sentinel cell")));
            },
            |ui, _| {
                ui.with_area(clip, |ui| {
                    assert_eq!(paint_settings_trust_body(ui, page, &trust), Ok(()));
                });
            },
        );
        let sentinel = sentinel.into_inner().expect("recorded sentinel");
        for y in 0..20 {
            for x in 0..120 {
                let cell = scene.buffer().cell((x, y)).expect("scene cell");
                if clip.contains(Position::new(x, y)) {
                    assert_ne!(format!("{cell:?}"), sentinel, "clip cell ({x}, {y})");
                } else {
                    assert_eq!(format!("{cell:?}"), sentinel, "outside clip ({x}, {y})");
                }
            }
        }
        assert_eq!(
            scene.buffer().cell((10, 8)).map(|cell| cell.symbol()),
            Some("R"),
            // `put_clipped` trims to the page, then `Ui::paint_str` intersects
            // the destination with the nested clip and starts the untrimmed
            // string at that clipped left edge.
            "the nested clip starts the title at its leading cell"
        );
    }

    #[test]
    fn trust_body_rejections_leave_the_prefilled_buffer_unchanged() {
        use ratatui_core::style::Style;
        use std::cell::RefCell;

        let too_many_rows: Vec<_> = (0..5)
            .map(|_| SettingsTrustRow::new("source", SettingsTrustSourceKind::Unknown, true, "1"))
            .collect();
        let too_many_notes = ["one", "two", "three"];
        let invalid_rows = SettingsTrust::new("Trust", "", &too_many_rows, &[]);
        let invalid_notes = SettingsTrust::new("Trust", "", &[], &too_many_notes);
        let area = Rect::new(0, 0, 80, 24);
        let mut scene = Scene::new(
            "settings_trust_body_invalid_capacity",
            Theme::junie(),
            ColorLevel::TrueColor,
            80,
            24,
        );
        let sentinel = RefCell::new(None);
        scene.draw_over(
            |buffer| {
                buffer.set_style(area, Style::default().fg(Color::Rgb(0x11, 0x22, 0x33)));
                *sentinel.borrow_mut() =
                    Some(format!("{:?}", buffer.cell((0, 0)).expect("sentinel cell")));
            },
            |ui, _| {
                assert_eq!(
                    paint_settings_trust_body(ui, area, &invalid_rows),
                    Err(SettingsBodyPaintError::TooManyRows {
                        maximum: 4,
                        actual: 5,
                    })
                );
                assert_eq!(
                    paint_settings_trust_body(ui, area, &invalid_notes),
                    Err(SettingsBodyPaintError::TooManyNotes {
                        maximum: 2,
                        actual: 3,
                    })
                );
            },
        );
        let sentinel = sentinel.into_inner().expect("recorded sentinel");
        for pos in area.positions() {
            assert_eq!(
                format!("{:?}", scene.buffer().cell(pos).expect("scene cell")),
                sentinel,
                "invalid Trust body wrote ({}, {})",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn body_rejections_leave_the_entire_prefilled_buffer_unchanged() {
        let tabs = [SettingsTab {
            label: "Agents",
            selected: true,
        }];
        let hints: [SettingsHint<'_>; 0] = [];
        let page = heritage_page("Settings › global › Agents", &tabs, &hints, None);
        let too_many_rows: Vec<_> = (0..7)
            .map(|_| SettingsAgentRow::new("Agent", "sync", "account"))
            .collect();
        let notes = ["one", "two"];
        let agents = SettingsAgents::new("Agents", "right", "header", &too_many_rows, &notes);
        let area = Rect::new(0, 0, 120, 40);
        let mut scene = Scene::new(
            "settings_body_rejection_sentinel",
            Theme::junie(),
            ColorLevel::TrueColor,
            120,
            40,
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
                    paint_settings_page_with_agents(
                        ui,
                        area,
                        &page,
                        &agents,
                        SettingsPageChrome::new(),
                    ),
                    Err(SettingsBodyPaintError::TooManyRows {
                        maximum: 6,
                        actual: 7,
                    })
                );
                assert_eq!(
                    paint_settings_page_with_agents(
                        ui,
                        Rect::new(0, 0, 120, 19),
                        &page,
                        &SettingsAgents::new("Agents", "right", "header", &[], &notes),
                        SettingsPageChrome::new(),
                    ),
                    Err(SettingsBodyPaintError::AreaTooShort {
                        minimum: 20,
                        actual: 19,
                    })
                );
                assert_eq!(
                    paint_settings_page_with_agents(
                        ui,
                        area,
                        &page,
                        &SettingsAgents::new("Agents", "right", "header", &[], &notes),
                        SettingsPageChrome::new().hint_offsets(&[1]),
                    ),
                    Err(SettingsBodyPaintError::HintOffsetsMismatch {
                        hints: 0,
                        offsets: 1,
                    })
                );
            },
        );
        assert!(
            scene
                .buffer()
                .content
                .iter()
                .all(|cell| cell.symbol() == "!"),
            "rejected render changed a sentinel cell"
        );
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

    #[test]
    fn agents_body_matches_shared_body_cells_and_keeps_both_notes() {
        let tabs = [SettingsTab {
            label: "Agents",
            selected: true,
        }];
        let hints = [SettingsHint {
            key: "Space",
            action: "Cycle mode",
        }];
        let page = heritage_page("Settings › global › Agents", &tabs, &hints, None);
        let rows = [
            SettingsAgentRow::new("Claude Code", "sync", "Claude · Personal").selected(true),
            SettingsAgentRow::new("Codex", "api key", "Codex · Primary"),
            SettingsAgentRow::new("Amp", "sync", "~/.config/amp/secrets.json"),
            SettingsAgentRow::new(
                "Kimi Code",
                "ignore",
                "no credentials handed to the container",
            ),
            SettingsAgentRow::new("OpenCode", "sync", "OpenCode · Go subscription"),
            SettingsAgentRow::new("Grok Build", "api key", "Grok · Team"),
        ];
        let notes = [
            "sync mirrors the host login · api key and oauth token take material from the registry account",
            "ignore starts the agent without credentials and removes it from session pickers",
        ];
        let agents = SettingsAgents::new(
            "Agent runtime mode",
            "accounts are registered in Accounts (c)",
            "Agent         Mode         Registry default",
            &rows,
            &notes,
        );
        for (width, height) in [(80, 24), (100, 30)] {
            let mut full = Scene::new(
                "settings_agents_body_full_page_reference",
                Theme::junie(),
                ColorLevel::TrueColor,
                width,
                height,
            );
            full.draw(|ui, area| {
                assert_eq!(
                    paint_settings_page_with_agents(
                        ui,
                        area,
                        &page,
                        &agents,
                        SettingsPageChrome::new(),
                    ),
                    Ok(())
                );
            });

            let mut body = Scene::new(
                "settings_agents_body_only",
                Theme::junie(),
                ColorLevel::TrueColor,
                width,
                height,
            );
            body.draw(|ui, area| {
                assert_eq!(paint_settings_agents_body(ui, area, &agents), Ok(()));
            });

            for y in 6..=16 {
                for x in 0..width {
                    let expected = full.buffer().cell((x, y)).expect("full-page cell");
                    let actual = body.buffer().cell((x, y)).expect("body-only cell");
                    assert_eq!(
                        format!("{actual:?}"),
                        format!("{expected:?}"),
                        "native cell changed at {width}x{height} ({x}, {y})"
                    );
                }
            }
            for expected in [
                "Agent runtime mode",
                "Claude Code",
                "Codex",
                "Amp",
                "Kimi Code",
                "OpenCode",
                "Grok Build",
                "sync mirrors the host login",
                "ignore starts the agent without credentials",
            ] {
                assert!(body.text().contains(expected), "missing {expected:?}");
            }
            let selected = body.buffer().cell((2, 8)).expect("selected gutter");
            assert_eq!(selected.symbol(), "▎");
            assert_eq!(selected.bg, SELECT_BG);
            assert!(selected.modifier.contains(Modifier::BOLD));
        }
    }

    #[test]
    fn agents_body_respects_page_and_parent_clips_and_checks_bounds_first() {
        use ratatui_core::style::{Modifier, Style};
        use std::cell::RefCell;

        let rows = [SettingsAgentRow::new(
            "Claude Code",
            "sync",
            "Claude · Personal",
        )];
        let notes = ["first note", "second note"];
        let agents = SettingsAgents::new("Agents", "", "Agent", &rows, &notes);
        let mut scene = Scene::new(
            "settings_agents_body_nested_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            40,
            24,
        );
        let sentinel = RefCell::new(None);
        scene.draw_over(
            |buffer| {
                buffer.set_style(
                    Rect::new(0, 0, 40, 24),
                    Style::default()
                        .fg(Color::Rgb(0x11, 0x22, 0x33))
                        .bg(Color::Rgb(0x44, 0x55, 0x66))
                        .add_modifier(Modifier::ITALIC),
                );
                *sentinel.borrow_mut() =
                    Some(format!("{:?}", buffer.cell((0, 0)).expect("sentinel cell")));
            },
            |ui, _| {
                let page = Rect::new(3, 3, 24, 9);
                ui.with_area(Rect::new(8, 9, 10, 2), |ui| {
                    assert_eq!(paint_settings_agents_body(ui, page, &agents), Ok(()));
                });
            },
        );
        let sentinel = sentinel.into_inner().expect("recorded sentinel");
        let changed = Rect::new(8, 9, 10, 2);
        for y in 0..24 {
            for x in 0..40 {
                let cell = scene.buffer().cell((x, y)).expect("scene cell");
                if changed.contains(Position::new(x, y)) {
                    assert_ne!(format!("{cell:?}"), sentinel, "clip cell ({x}, {y})");
                } else {
                    assert_eq!(format!("{cell:?}"), sentinel, "outside clip ({x}, {y})");
                }
            }
        }

        let empty_page = Rect::new(0, 0, 30, 6);
        let mut empty = Scene::new(
            "settings_agents_body_empty_clip",
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            6,
        );
        let empty_sentinel = RefCell::new(None);
        empty.draw_over(
            |buffer| {
                buffer.set_style(
                    empty_page,
                    Style::default()
                        .fg(Color::Rgb(0x11, 0x22, 0x33))
                        .bg(Color::Rgb(0x44, 0x55, 0x66)),
                );
                *empty_sentinel.borrow_mut() =
                    Some(format!("{:?}", buffer.cell((0, 0)).expect("sentinel cell")));
            },
            |ui, _| assert_eq!(paint_settings_agents_body(ui, empty_page, &agents), Ok(())),
        );
        let empty_sentinel = empty_sentinel.into_inner().expect("recorded sentinel");
        for y in 0..6 {
            for x in 0..30 {
                let cell = empty.buffer().cell((x, y)).expect("scene cell");
                assert_eq!(
                    format!("{cell:?}"),
                    empty_sentinel,
                    "empty clip wrote ({x}, {y})"
                );
            }
        }

        let seven_rows = [
            SettingsAgentRow::new("a", "", ""),
            SettingsAgentRow::new("b", "", ""),
            SettingsAgentRow::new("c", "", ""),
            SettingsAgentRow::new("d", "", ""),
            SettingsAgentRow::new("e", "", ""),
            SettingsAgentRow::new("f", "", ""),
            SettingsAgentRow::new("g", "", ""),
        ];
        let too_many_rows = SettingsAgents::new("Agents", "", "Agent", &seven_rows, &notes);
        let three_notes = ["first", "second", "third"];
        let too_many_notes = SettingsAgents::new("Agents", "", "Agent", &rows, &three_notes);
        let mut invalid = Scene::new(
            "settings_agents_body_invalid_capacity",
            Theme::junie(),
            ColorLevel::TrueColor,
            40,
            24,
        );
        let invalid_sentinel = RefCell::new(None);
        invalid.draw_over(
            |buffer| {
                buffer.set_style(
                    Rect::new(0, 0, 40, 24),
                    Style::default()
                        .fg(Color::Rgb(0x11, 0x22, 0x33))
                        .bg(Color::Rgb(0x44, 0x55, 0x66)),
                );
                *invalid_sentinel.borrow_mut() =
                    Some(format!("{:?}", buffer.cell((0, 0)).expect("sentinel cell")));
            },
            |ui, area| {
                assert_eq!(
                    paint_settings_agents_body(ui, area, &too_many_rows),
                    Err(SettingsBodyPaintError::TooManyRows {
                        maximum: 6,
                        actual: 7
                    })
                );
                assert_eq!(
                    paint_settings_agents_body(ui, area, &too_many_notes),
                    Err(SettingsBodyPaintError::TooManyNotes {
                        maximum: 2,
                        actual: 3
                    })
                );
            },
        );
        let invalid_sentinel = invalid_sentinel.into_inner().expect("recorded sentinel");
        for y in 0..24 {
            for x in 0..40 {
                let cell = invalid.buffer().cell((x, y)).expect("scene cell");
                assert_eq!(
                    format!("{cell:?}"),
                    invalid_sentinel,
                    "invalid wrote ({x}, {y})"
                );
            }
        }
    }
}
