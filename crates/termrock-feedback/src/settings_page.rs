use std::fmt;

use ratatui_core::layout::{Position, Rect};
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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsGeneralFacts<'a> {
    default_agent: &'a str,
    workspace: &'a str,
    dco_signoff: &'a str,
    secret_policy: &'a str,
    trust_label: &'a str,
}

impl<'a> SettingsGeneralFacts<'a> {
    pub const fn new(
        default_agent: &'a str,
        workspace: &'a str,
        dco_signoff: &'a str,
        secret_policy: &'a str,
        trust_label: &'a str,
    ) -> Self {
        Self {
            default_agent,
            workspace,
            dco_signoff,
            secret_policy,
            trust_label,
        }
    }
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

    pub cluster: Option<SettingsCluster<'a>>,
}

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
    pub const fn new(key: &'a str, scope: &'a str, stars: usize, source: &'a str) -> Self {
        Self {
            key,
            scope,
            stars,
            source,
            selected: false,
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnvGroup<'a> {
    title: &'a str,
    right: Option<&'a str>,
    rows: &'a [SettingsEnvRow<'a>],
    add: Option<&'a str>,
}

impl<'a> SettingsEnvGroup<'a> {
    pub const fn new(title: &'a str, rows: &'a [SettingsEnvRow<'a>]) -> Self {
        Self {
            title,
            right: None,
            rows,
            add: None,
        }
    }

    pub const fn right(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    pub const fn add(mut self, label: &'a str) -> Self {
        self.add = Some(label);
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnvRole<'a> {
    name: &'a str,
    right: Option<&'a str>,
    rows: &'a [SettingsEnvRow<'a>],
    add: Option<&'a str>,
}

impl<'a> SettingsEnvRole<'a> {
    pub const fn new(name: &'a str, rows: &'a [SettingsEnvRow<'a>]) -> Self {
        Self {
            name,
            right: None,
            rows,
            add: None,
        }
    }

    pub const fn right(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    pub const fn add(mut self, label: &'a str) -> Self {
        self.add = Some(label);
        self
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsEnv<'a> {
    groups: &'a [SettingsEnvGroup<'a>],
    overrides: Option<SettingsEnvOverrides<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorEnvironmentValue<'a> {
    PlainMasked { stars: usize },

    OnePassword { display_path: &'a str },

    HostEnv { name: &'a str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EditorEnvironmentRow<'a> {
    key: &'a str,
    value: EditorEnvironmentValue<'a>,
    selected: bool,
}

impl<'a> EditorEnvironmentRow<'a> {
    pub const fn new(key: &'a str, value: EditorEnvironmentValue<'a>) -> Self {
        Self {
            key,
            value,
            selected: false,
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EditorEnvironmentRole<'a> {
    name: &'a str,
    right: &'a str,
    rows: &'a [EditorEnvironmentRow<'a>],
}

impl<'a> EditorEnvironmentRole<'a> {
    pub const fn new(name: &'a str, right: &'a str, rows: &'a [EditorEnvironmentRow<'a>]) -> Self {
        Self { name, right, rows }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EditorEnvironmentBody<'a> {
    workspace_right: &'a str,
    workspace_rows: &'a [EditorEnvironmentRow<'a>],
    overrides_right: &'a str,
    roles: &'a [EditorEnvironmentRole<'a>],
}

impl<'a> EditorEnvironmentBody<'a> {
    pub const fn new(
        workspace_right: &'a str,
        workspace_rows: &'a [EditorEnvironmentRow<'a>],
        overrides_right: &'a str,
        roles: &'a [EditorEnvironmentRole<'a>],
    ) -> Self {
        Self {
            workspace_right,
            workspace_rows,
            overrides_right,
            roles,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorRolesMetaTone {
    Error,

    Warning,

    Muted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EditorRolesRow<'a> {
    name: &'a str,
    allowed: bool,
    is_default: bool,
    metadata: &'a str,
    metadata_tone: EditorRolesMetaTone,
}

impl<'a> EditorRolesRow<'a> {
    pub const fn new(
        name: &'a str,
        allowed: bool,
        is_default: bool,
        metadata: &'a str,
        metadata_tone: EditorRolesMetaTone,
    ) -> Self {
        Self {
            name,
            allowed,
            is_default,
            metadata,
            metadata_tone,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EditorRolesBody<'a> {
    allowed_count: usize,
    total_roles: usize,
    default_role: &'a str,
    rows: &'a [EditorRolesRow<'a>],
}

impl<'a> EditorRolesBody<'a> {
    pub const fn new(
        allowed_count: usize,
        total_roles: usize,
        default_role: &'a str,
        rows: &'a [EditorRolesRow<'a>],
    ) -> Self {
        Self {
            allowed_count,
            total_roles,
            default_role,
            rows,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsTrustSourceKind {
    Git,

    Path,

    Unknown,
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsAgentRow<'a> {
    name: &'a str,
    mode: &'a str,
    account: &'a str,
    selected: bool,
}

impl<'a> SettingsAgentRow<'a> {
    pub const fn new(name: &'a str, mode: &'a str, account: &'a str) -> Self {
        Self {
            name,
            mode,
            account,
            selected: false,
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

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

    pub const fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

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

    pub const fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsHintOverflowTone {
    Faint,

    Secondary,
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct SettingsHintOverflow<'a> {
    text: &'a str,
    at: Option<u16>,
    tone: SettingsHintOverflowTone,
}

impl<'a> SettingsHintOverflow<'a> {
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            at: None,
            tone: SettingsHintOverflowTone::Faint,
        }
    }

    pub const fn at(mut self, offset: u16) -> Self {
        self.at = Some(offset);
        self
    }

    pub const fn tone(mut self, tone: SettingsHintOverflowTone) -> Self {
        self.tone = tone;
        self
    }
}

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
    pub const fn new() -> Self {
        Self {
            tab_note: None,
            crumb_note: None,
            hint_offsets: None,
            overflow: None,
            hide_hints: false,
        }
    }

    pub const fn tab_note(mut self, note: &'a str) -> Self {
        self.tab_note = Some(note);
        self
    }

    pub const fn crumb_note(mut self, note: &'a str) -> Self {
        self.crumb_note = Some(note);
        self
    }

    pub const fn hint_offsets(mut self, offsets: &'a [u16]) -> Self {
        self.hint_offsets = Some(offsets);
        self
    }

    pub const fn overflow(mut self, overflow: SettingsHintOverflow<'a>) -> Self {
        self.overflow = Some(overflow);
        self
    }

    pub const fn hide_hints(mut self, hide: bool) -> Self {
        self.hide_hints = hide;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsBodyPaintError {
    LegacyBodyNotEmpty,

    AreaTooNarrow { minimum: u16, actual: u16 },

    AreaTooShort { minimum: u16, actual: u16 },

    TooManyRows { maximum: usize, actual: usize },

    TooManyNotes { maximum: usize, actual: usize },

    HintOffsetsMismatch { hints: usize, offsets: usize },

    ConflictingHintOverflow,

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
    pub const fn new(groups: &'a [SettingsEnvGroup<'a>]) -> Self {
        Self {
            groups,
            overrides: None,
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsEnvPaintError {
    LegacyBodyNotEmpty,

    AreaTooNarrow {
        minimum: u16,
        actual: u16,
    },

    AreaTooShort {
        minimum: u16,
        actual: u16,
    },

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorEnvironmentPaintError {
    AreaTooNarrow {
        minimum: u16,
        actual: u16,
    },

    AreaTooShort {
        minimum: u16,
        actual: u16,
    },

    ContentTooTall {
        required_lines: usize,
        available_lines: usize,
    },
}

impl fmt::Display for EditorEnvironmentPaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AreaTooNarrow { minimum, actual } => {
                write!(f, "Editor Env needs {minimum} columns, received {actual}")
            }
            Self::AreaTooShort { minimum, actual } => {
                write!(f, "Editor Env needs {minimum} rows, received {actual}")
            }
            Self::ContentTooTall {
                required_lines,
                available_lines,
            } => write!(
                f,
                "Editor Env needs {required_lines} body rows, but area provides {available_lines}"
            ),
        }
    }
}

impl std::error::Error for EditorEnvironmentPaintError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorRolesPaintError {
    AreaTooNarrow { minimum: u16, actual: u16 },

    AreaTooShort { minimum: u16, actual: u16 },

    TooManyRows { maximum: usize, actual: usize },

    TotalRolesBelowVisible { visible: usize, total: usize },

    UnsupportedContent,
}

impl fmt::Display for EditorRolesPaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AreaTooNarrow { minimum, actual } => {
                write!(f, "Editor Roles needs {minimum} columns, received {actual}")
            }
            Self::AreaTooShort { minimum, actual } => {
                write!(f, "Editor Roles needs {minimum} rows, received {actual}")
            }
            Self::TooManyRows { maximum, actual } => {
                write!(
                    f,
                    "Editor Roles allows {maximum} visible rows, received {actual}"
                )
            }
            Self::TotalRolesBelowVisible { visible, total } => write!(
                f,
                "Editor Roles has {visible} visible rows for a {total}-role source list"
            ),
            Self::UnsupportedContent => {
                f.write_str("Editor Roles content exceeds the proven single-cell columns")
            }
        }
    }
}

impl std::error::Error for EditorRolesPaintError {}

const SETTINGS_ENV_MIN_WIDTH: u16 = 16;
const SETTINGS_ENV_MIN_HEIGHT: u16 = 8;
const SETTINGS_ENV_BODY_TOP: u16 = 6;
const SETTINGS_ACTION_OFFSET: u16 = 3;
const EDITOR_ENV_MIN_WIDTH: u16 = 120;
const EDITOR_ENV_MIN_HEIGHT: u16 = 40;
const EDITOR_ENV_BODY_TOP: u16 = 6;
const EDITOR_ENV_FOOTER_ROW: u16 = 37;
const EDITOR_ENV_RIGHT_EDGE: u16 = 117;
const EDITOR_ROLES_MIN_WIDTH: u16 = 120;
const EDITOR_ROLES_MIN_HEIGHT: u16 = 40;
const EDITOR_ROLES_BODY_TOP: u16 = 6;
const EDITOR_ROLES_MAX_ROWS: usize = 29;
const EDITOR_ROLES_VISIBLE_ROWS: u16 = 29;
const EDITOR_ROLES_SELECTED_BAR_ROWS: usize = 17;
const EDITOR_ROLES_META_WIDTH: u16 = 92;
const EDITOR_ROLES_DEFAULT_LABEL_WIDTH: u16 = 17;
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

pub fn paint_editor_environment_body(
    ui: &mut Ui<'_>,
    area: Rect,
    body: &EditorEnvironmentBody<'_>,
) -> Result<(), EditorEnvironmentPaintError> {
    if area.width < EDITOR_ENV_MIN_WIDTH {
        return Err(EditorEnvironmentPaintError::AreaTooNarrow {
            minimum: EDITOR_ENV_MIN_WIDTH,
            actual: area.width,
        });
    }
    if area.height < EDITOR_ENV_MIN_HEIGHT {
        return Err(EditorEnvironmentPaintError::AreaTooShort {
            minimum: EDITOR_ENV_MIN_HEIGHT,
            actual: area.height,
        });
    }

    let body_area = Rect {
        height: EDITOR_ENV_FOOTER_ROW,
        ..area
    };
    let required_lines = editor_environment_required_lines(body);
    let available_lines = usize::from(body_area.height.saturating_sub(EDITOR_ENV_BODY_TOP));
    if required_lines > available_lines {
        return Err(EditorEnvironmentPaintError::ContentTooTall {
            required_lines,
            available_lines,
        });
    }

    let content = Rect::new(
        area.x,
        area.y.saturating_add(EDITOR_ENV_BODY_TOP),
        area.width,
        body_area.height.saturating_sub(EDITOR_ENV_BODY_TOP),
    );
    ui.fill(content, plain(EDITOR_ENV_WHITE, EDITOR_ENV_BLACK));
    paint_editor_environment(ui, body_area, body);
    Ok(())
}

pub fn paint_editor_roles_body(
    ui: &mut Ui<'_>,
    area: Rect,
    body: &EditorRolesBody<'_>,
) -> Result<(), EditorRolesPaintError> {
    if area.width < EDITOR_ROLES_MIN_WIDTH {
        return Err(EditorRolesPaintError::AreaTooNarrow {
            minimum: EDITOR_ROLES_MIN_WIDTH,
            actual: area.width,
        });
    }
    if area.height < EDITOR_ROLES_MIN_HEIGHT {
        return Err(EditorRolesPaintError::AreaTooShort {
            minimum: EDITOR_ROLES_MIN_HEIGHT,
            actual: area.height,
        });
    }
    if body.rows.len() > EDITOR_ROLES_MAX_ROWS {
        return Err(EditorRolesPaintError::TooManyRows {
            maximum: EDITOR_ROLES_MAX_ROWS,
            actual: body.rows.len(),
        });
    }
    if body.total_roles < body.rows.len() {
        return Err(EditorRolesPaintError::TotalRolesBelowVisible {
            visible: body.rows.len(),
            total: body.total_roles,
        });
    }
    if !editor_roles_content_supported(body) {
        return Err(EditorRolesPaintError::UnsupportedContent);
    }

    let fixed_area = Rect::new(
        area.x,
        area.y,
        EDITOR_ROLES_MIN_WIDTH,
        EDITOR_ROLES_MIN_HEIGHT,
    );
    paint_editor_roles(ui, fixed_area, body);
    Ok(())
}

fn editor_roles_content_supported(body: &EditorRolesBody<'_>) -> bool {
    editor_roles_text_supported(body.default_role, EDITOR_ROLES_DEFAULT_LABEL_WIDTH)
        && body.rows.iter().enumerate().all(|(index, row)| {
            let name_width = if row.allowed && index == 0 {
                if row.is_default { 15 } else { 17 }
            } else {
                14
            };
            !row.name.is_empty()
                && editor_roles_text_supported(row.name, name_width)
                && !row.metadata.is_empty()
                && editor_roles_text_supported(row.metadata, EDITOR_ROLES_META_WIDTH)
        })
}

fn editor_roles_text_supported(text: &str, maximum: u16) -> bool {
    let Ok(character_width) = u16::try_from(text.chars().count()) else {
        return false;
    };
    text.chars().all(|character| {
        (character.is_ascii() && !character.is_ascii_control()) || character == '·'
    }) && character_width <= maximum
        && width(text) == character_width
}

fn paint_editor_roles(ui: &mut Ui<'_>, area: Rect, body: &EditorRolesBody<'_>) {
    let heading_y = area.y.saturating_add(EDITOR_ROLES_BODY_TOP);
    let allowed = format!(
        "Allowed roles  {} of {}",
        body.allowed_count, body.total_roles
    );
    put_editor_roles(
        ui,
        area,
        area.x.saturating_add(4),
        heading_y,
        &allowed,
        bold(EDITOR_ROLES_SECONDARY, EDITOR_ROLES_BLACK),
    );
    let default = format!("default ★ {}", body.default_role);
    put_editor_roles(
        ui,
        area,
        area.x.saturating_add(93),
        heading_y,
        &default,
        plain(EDITOR_ROLES_DARK_MUTED, EDITOR_ROLES_BLACK),
    );

    for (index, row) in body.rows.iter().enumerate() {
        let y = heading_y.saturating_add(1).saturating_add(index as u16);
        let selected = index == 0;
        paint_editor_roles_row(ui, area, y, row, selected);
        let (track, color) = if index < EDITOR_ROLES_SELECTED_BAR_ROWS {
            ("┃", Color::Rgb(0x80, 0x80, 0x80))
        } else {
            ("┆", Color::Rgb(0x47, 0x47, 0x47))
        };
        ui.paint_cell(
            Position::new(area.x.saturating_add(117), y),
            track,
            plain(color, EDITOR_ROLES_BLACK),
        );
    }

    let mut scroll = crate::ui::ScrollState::new(body.total_roles);
    scroll.set_viewport(usize::from(EDITOR_ROLES_VISIBLE_ROWS));
    ui.scroll_edges(
        Rect::new(
            area.x.saturating_add(2),
            heading_y.saturating_add(1),
            115,
            EDITOR_ROLES_VISIBLE_ROWS,
        ),
        &scroll,
    );
}

fn paint_editor_roles_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &EditorRolesRow<'_>,
    selected: bool,
) {
    let left = area.x.saturating_add(2);
    let background = if selected {
        EDITOR_ROLES_BADGE
    } else {
        EDITOR_ROLES_BLACK
    };
    let selected_style = selected;
    if selected {
        put_editor_roles(
            ui,
            area,
            left,
            y,
            &" ".repeat(115),
            bold(EDITOR_ROLES_WHITE, background),
        );
        let accent = ui.theme_ref().color.accent;
        ui.paint_cell(Position::new(left, y), "▎", bold(accent, background));
        ui.paint_cell(
            Position::new(left.saturating_add(1), y),
            "›",
            bold(accent, background),
        );
    } else {
        put_editor_roles(
            ui,
            area,
            left,
            y,
            " ",
            plain(EDITOR_ROLES_BLACK, EDITOR_ROLES_BLACK),
        );
        put_editor_roles(
            ui,
            area,
            left.saturating_add(1),
            y,
            " ",
            plain(EDITOR_ROLES_SECONDARY, EDITOR_ROLES_BLACK),
        );
    }
    put_editor_roles(
        ui,
        area,
        area.x.saturating_add(5),
        y,
        " ",
        if selected_style {
            bold(EDITOR_ROLES_AMBER, background)
        } else {
            plain(EDITOR_ROLES_AMBER, background)
        },
    );

    if row.allowed {
        let marker = if selected && row.is_default {
            " ★"
        } else {
            ""
        };
        let columns = if selected { 22 } else { 19 };
        let label = format!(" [✓] {}{marker}", row.name);
        let label = format!("{label:<width$}", width = usize::from(columns));
        put_editor_roles_cols(
            ui,
            area,
            area.x.saturating_add(6),
            y,
            &label,
            columns,
            if selected {
                bold(EDITOR_ROLES_WHITE, background)
            } else {
                plain(EDITOR_ROLES_WHITE, background)
            },
        );
    } else {
        put_editor_roles(
            ui,
            area,
            area.x.saturating_add(7),
            y,
            "[ ]",
            if selected {
                bold(EDITOR_ROLES_MUTED, background)
            } else {
                plain(EDITOR_ROLES_MUTED, background)
            },
        );
        let label = format!(" {:<14}", row.name);
        put_editor_roles(
            ui,
            area,
            area.x.saturating_add(10),
            y,
            &label,
            if selected {
                bold(EDITOR_ROLES_WHITE, background)
            } else {
                plain(EDITOR_ROLES_WHITE, background)
            },
        );
    }
    if !selected {
        put_editor_roles(
            ui,
            area,
            area.x.saturating_add(25),
            y,
            " ",
            plain(EDITOR_ROLES_SECONDARY, background),
        );
    }
    paint_editor_roles_metadata(ui, area, y, row, selected, background);
}

fn paint_editor_roles_metadata(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &EditorRolesRow<'_>,
    selected: bool,
    background: Color,
) {
    let start = area.x.saturating_add(28);
    if !selected {
        let foreground = match row.metadata_tone {
            EditorRolesMetaTone::Error => EDITOR_ROLES_ERROR,
            EditorRolesMetaTone::Warning => EDITOR_ROLES_AMBER,
            EditorRolesMetaTone::Muted => EDITOR_ROLES_MUTED,
        };
        put_editor_roles(
            ui,
            area,
            start,
            y,
            row.metadata,
            plain(foreground, background),
        );
        return;
    }

    let mut column = start;
    let mut word = String::new();
    let flush = |ui: &mut Ui<'_>, column: &mut u16, word: &mut String| {
        if word.is_empty() {
            return;
        }
        put_editor_roles(
            ui,
            area,
            *column,
            y,
            word,
            bold(EDITOR_ROLES_SECONDARY, background),
        );
        *column = column.saturating_add(word.chars().count() as u16);
        word.clear();
    };
    for character in row.metadata.chars() {
        if character == ' ' || character == '·' {
            flush(ui, &mut column, &mut word);
            let text = character.to_string();
            let foreground = if character == '·' {
                EDITOR_ROLES_SECONDARY
            } else {
                EDITOR_ROLES_MUTED
            };
            put_editor_roles(ui, area, column, y, &text, bold(foreground, background));
            column = column.saturating_add(1);
        } else {
            word.push(character);
        }
    }
    flush(ui, &mut column, &mut word);
}

fn put_editor_roles(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let visible_area = area.intersection(ui.full());
    let width = area.right().saturating_sub(x);
    put_clipped(ui, visible_area, x, y, text, width, style);
}

fn put_editor_roles_cols(
    ui: &mut Ui<'_>,
    area: Rect,
    x: u16,
    y: u16,
    text: &str,
    columns: u16,
    style: PaintStyle,
) {
    let visible_area = area.intersection(ui.full());
    put_clipped(ui, visible_area, x, y, text, columns, style);
}

const EDITOR_ROLES_BLACK: Color = Color::Rgb(0x00, 0x00, 0x00);
const EDITOR_ROLES_WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const EDITOR_ROLES_SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const EDITOR_ROLES_MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const EDITOR_ROLES_DARK_MUTED: Color = Color::Rgb(0x73, 0x73, 0x73);
const EDITOR_ROLES_BADGE: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const EDITOR_ROLES_AMBER: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const EDITOR_ROLES_ERROR: Color = Color::Rgb(0xf0, 0x5d, 0x5d);

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

pub fn paint_settings_general_body(ui: &mut Ui<'_>, area: Rect, facts: &SettingsGeneralFacts<'_>) {
    const SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
    const WHITE: Color = Color::Rgb(255, 255, 255);
    const BLACK: Color = Color::Rgb(0, 0, 0);
    let x = area.x.saturating_add(4);
    let rows = [
        ("Default agent", facts.default_agent, 6, 7),
        ("Workspace", facts.workspace, 10, 11),
        ("DCO signoff", facts.dco_signoff, 14, 15),
        ("Secret policy", facts.secret_policy, 18, 19),
        ("Trust", facts.trust_label, 22, 23),
    ];

    ui.with_area(area, |ui| {
        for (label, value, label_row, value_row) in rows {
            ui.paint_str(
                Rect::new(
                    x,
                    area.y.saturating_add(label_row),
                    u16::try_from(label.chars().count()).unwrap_or(u16::MAX),
                    1,
                ),
                label,
                PaintStyle::new().fg(SECONDARY).bg(BLACK),
            );
            ui.paint_str(
                Rect::new(
                    x,
                    area.y.saturating_add(value_row),
                    u16::try_from(value.chars().count()).unwrap_or(u16::MAX),
                    1,
                ),
                value,
                PaintStyle::new().fg(WHITE).bg(BLACK),
            );
        }
    });
}

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
const EDITOR_ENV_BLACK: Color = Color::Rgb(0x00, 0x00, 0x00);
const EDITOR_ENV_WHITE: Color = Color::Rgb(0xff, 0xff, 0xff);
const EDITOR_ENV_SECONDARY: Color = Color::Rgb(0xb3, 0xb3, 0xb3);
const EDITOR_ENV_MUTED: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const EDITOR_ENV_DARK_MUTED: Color = Color::Rgb(0x73, 0x73, 0x73);
const EDITOR_ENV_BADGE: Color = Color::Rgb(0x17, 0x3f, 0x1c);

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

fn editor_environment_required_lines(body: &EditorEnvironmentBody<'_>) -> usize {
    let workspace = 1_usize
        .saturating_add(body.workspace_rows.len())
        .saturating_add(1);
    let roles = body.roles.iter().fold(0_usize, |lines, role| {
        lines
            .saturating_add(1)
            .saturating_add(role.rows.len())
            .saturating_add(1)
    });
    workspace
        .saturating_add(1)
        .saturating_add(roles)
        .saturating_add(1)
}

fn paint_editor_environment(ui: &mut Ui<'_>, area: Rect, body: &EditorEnvironmentBody<'_>) {
    let mut y = area.y.saturating_add(EDITOR_ENV_BODY_TOP);
    paint_editor_environment_heading(
        ui,
        area,
        y,
        area.x.saturating_add(6),
        "Workspace",
        body.workspace_right,
    );
    y = y.saturating_add(1);
    for row in body.workspace_rows {
        paint_editor_environment_row(ui, area, y, row);
        y = y.saturating_add(1);
    }
    paint_editor_environment_add(ui, area, y, "+ Add environment variable");
    y = y.saturating_add(1);

    paint_editor_environment_heading(
        ui,
        area,
        y,
        area.x.saturating_add(6),
        "Role overrides",
        body.overrides_right,
    );
    y = y.saturating_add(1);
    for role in body.roles {
        let title = format!("Role: {}", role.name);
        put(
            ui,
            area,
            area.x.saturating_add(4),
            y,
            "▾",
            plain(EDITOR_ENV_SECONDARY, EDITOR_ENV_BLACK),
        );
        paint_editor_environment_heading(ui, area, y, area.x.saturating_add(6), &title, role.right);
        y = y.saturating_add(1);
        for row in role.rows {
            paint_editor_environment_row(ui, area, y, row);
            y = y.saturating_add(1);
        }
        let add = format!("+ Add {} environment variable", role.name);
        paint_editor_environment_add(ui, area, y, &add);
        y = y.saturating_add(1);
    }
    paint_editor_environment_add(ui, area, y, "+ Add role override…");
}

fn paint_editor_environment_heading(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    x: u16,
    title: &str,
    right: &str,
) {
    put(
        ui,
        area,
        x,
        y,
        title,
        bold(EDITOR_ENV_SECONDARY, EDITOR_ENV_BLACK),
    );
    let right_width = width(right);
    let right_x = area
        .x
        .saturating_add(EDITOR_ENV_RIGHT_EDGE.saturating_sub(right_width));
    put(
        ui,
        area,
        right_x,
        y,
        right,
        plain(EDITOR_ENV_DARK_MUTED, EDITOR_ENV_BLACK),
    );
}

fn paint_editor_environment_add(ui: &mut Ui<'_>, area: Rect, y: u16, label: &str) {
    put(
        ui,
        area,
        area.x.saturating_add(2),
        y,
        " ",
        plain(EDITOR_ENV_BLACK, EDITOR_ENV_BLACK),
    );
    put(
        ui,
        area,
        area.x.saturating_add(3),
        y,
        " ",
        plain(EDITOR_ENV_SECONDARY, EDITOR_ENV_BLACK),
    );
    put(
        ui,
        area,
        area.x.saturating_add(8),
        y,
        label,
        plain(EDITOR_ENV_SECONDARY, EDITOR_ENV_BLACK),
    );
}

fn paint_editor_environment_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    row: &EditorEnvironmentRow<'_>,
) {
    let gutter_x = area.x.saturating_add(2);
    let key_x = area.x.saturating_add(4);
    let value_x = area.x.saturating_add(28);
    let source_x = area.x.saturating_add(54);
    let selected = row.selected;
    let background = if selected {
        EDITOR_ENV_BADGE
    } else {
        EDITOR_ENV_BLACK
    };

    if selected {
        ui.fill(
            Rect::new(gutter_x, y, 116, 1),
            bold(EDITOR_ENV_WHITE, EDITOR_ENV_BADGE),
        );
        let accent = ui.theme_ref().color.accent;
        put(ui, area, gutter_x, y, "▎›", bold(accent, EDITOR_ENV_BADGE));
    } else {
        put(
            ui,
            area,
            gutter_x,
            y,
            " ",
            plain(EDITOR_ENV_BLACK, EDITOR_ENV_BLACK),
        );
        put(
            ui,
            area,
            gutter_x.saturating_add(1),
            y,
            " ",
            plain(EDITOR_ENV_SECONDARY, EDITOR_ENV_BLACK),
        );
    }

    let key_style = if selected {
        bold(EDITOR_ENV_WHITE, background)
    } else {
        plain(EDITOR_ENV_WHITE, background)
    };
    let value_style = if selected {
        bold(EDITOR_ENV_SECONDARY, background)
    } else {
        plain(EDITOR_ENV_SECONDARY, background)
    };
    let source_style = if selected {
        bold(EDITOR_ENV_SECONDARY, background)
    } else {
        plain(EDITOR_ENV_MUTED, background)
    };
    ui.fill(Rect::new(key_x, y, 24, 1), key_style);
    ui.fill(Rect::new(value_x, y, 24, 1), value_style);
    ui.fill(Rect::new(source_x, y, 63, 1), source_style);

    let key = format!("    {:<20}", row.key);
    put_cols(ui, area, key_x, y, &key, 24, key_style);

    let (source, stars, public_value) = match row.value {
        EditorEnvironmentValue::PlainMasked { stars } => ("plain", stars.min(16), None),
        EditorEnvironmentValue::OnePassword { display_path } => {
            ("", 16, Some(format!("[op] {display_path}")))
        }
        EditorEnvironmentValue::HostEnv { name } => ("host env", 0, Some(format!("${name}"))),
    };
    match row.value {
        EditorEnvironmentValue::HostEnv { .. } => {
            if let Some(value) = public_value.as_deref() {
                put_cols(ui, area, value_x, y, value, 24, value_style);
            }
        }
        EditorEnvironmentValue::PlainMasked { .. } | EditorEnvironmentValue::OnePassword { .. } => {
            paint_env_stars(ui, area, value_x, y, stars, 24, value_style);
        }
    }

    let source = public_value
        .as_deref()
        .filter(|_| matches!(row.value, EditorEnvironmentValue::OnePassword { .. }))
        .unwrap_or(source);
    if selected {
        let source_width = width(source).min(EDITOR_ENV_RIGHT_EDGE.saturating_sub(54));
        put_cols(ui, area, source_x, y, source, source_width, source_style);
        let tail_x = source_x.saturating_add(source_width);
        let tail_width = area
            .x
            .saturating_add(EDITOR_ENV_RIGHT_EDGE)
            .saturating_sub(tail_x);
        if tail_width > 0 {
            put_cols(
                ui,
                area,
                tail_x,
                y,
                &" ".repeat(usize::from(tail_width)),
                tail_width,
                bold(EDITOR_ENV_MUTED, EDITOR_ENV_BADGE),
            );
        }
    } else {
        put_cols(ui, area, source_x, y, source, 63, source_style);
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
