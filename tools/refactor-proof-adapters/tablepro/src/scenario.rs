//! Finite TP scenario axes owned by this adapter.

use junie_tui::ColorLevel;

/// TP-001 parent identity.
pub const SCENARIO_TP001: &str = "TP-001";
/// TP-002 parent identity.
pub const SCENARIO_TP002: &str = "TP-002";
/// Preset C: Connections launch with no `--connect`.
pub const PRESET_C: &str = "C";
/// Preset W: `--connect Production` then the Workbench landing path.
pub const PRESET_W: &str = "W";
/// Preset L: `--connect "Local PostgreSQL"`.
pub const PRESET_L: &str = "L";
/// Preset T: W then Ctrl+O, `orders`, Enter.
pub const PRESET_T: &str = "T";
/// Preset Q: W then Ctrl+T, `i`, SQL, Esc.
pub const PRESET_Q: &str = "Q";
/// Preset QL: L then Ctrl+T, `i`, SQL, Esc.
pub const PRESET_QL: &str = "QL";
/// Named first-frame checkpoint of TP-001.
pub const CHECKPOINT_INITIAL: &str = "initial";
/// Fixture ordinal of Local PostgreSQL in `db::connections()`.
pub const FIXTURE_LOCAL_POSTGRESQL: usize = 0;
/// Fixture ordinal of Staging (auth-failed) in `db::connections()`.
pub const FIXTURE_STAGING: usize = 2;
/// Fixture ordinal of Analytics (unreachable) in `db::connections()`.
pub const FIXTURE_ANALYTICS: usize = 3;
/// Fixture ordinal of Production in `db::connections()`.
pub const FIXTURE_PRODUCTION: usize = 4;

/// `ALL` sizes from the `TablePro` scenario contract: 80×24, 100×30, 120×40, 160×50.
pub const ALL_SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];

/// Canonical frozen-oracle sizes: `ALL` plus the 72×20 minimum shell.
/// The frozen `visual-baseline` store carries all five widths for every
/// `tablepro` scenario; the adapter must not drop 72×20.
pub const CANONICAL_SIZES: [(u16, u16); 5] = [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)];

/// Frozen `visual-baseline` `tablepro` scenario leaves (44).
/// From `snapshots/tablepro/<scenario>/<size>/<color>` at the pinned tag.
/// Order is grouped-store order: ack, audit, connections, fade, overlays,
/// query, resize, table, workbench.
pub const ORACLE_SCENARIOS: [&str; 44] = [
    "ack/armed",
    "ack/delete_gate",
    "ack/executed",
    "ack/gate",
    "audit/production",
    "connections/default",
    "connections/delete_dialog",
    "connections/duplicated",
    "connections/filter",
    "connections/form_advanced",
    "connections/form_new",
    "connections/form_new-filled",
    "connections/production_detail",
    "fade/table_wheel",
    "overlays/help_overlay",
    "overlays/history_tab",
    "overlays/picker_open",
    "overlays/safemode_picker",
    "overlays/tablist_open",
    "overlays/tablist_tables",
    "query/close_confirm",
    "query/completion",
    "query/completion_columns",
    "query/error",
    "query/explain",
    "query/explain_analyze",
    "query/new_tab",
    "query/results",
    "resize/workbench_grown",
    "resize/workbench_shrunk",
    "table/cell_editing",
    "table/data",
    "table/dirty",
    "table/filter_editor",
    "table/filtered",
    "table/row_duplicated",
    "table/scrolled_right",
    "table/sorted",
    "table/sorted-filtered",
    "table/structure",
    "workbench/commit_dialog",
    "workbench/explorer_hidden",
    "workbench/maximized",
    "workbench/quit_confirm",
];

/// Expected frozen `tablepro` key count: 44 scenarios × 5 sizes × 5 colors.
pub const ORACLE_KEY_COUNT: usize = 44 * 5 * 5;

/// Format a frozen oracle key: `tablepro/<scenario>/<cols>x<rows>/<color>`.
///
/// # Panics
///
/// Never panics at runtime; formatting a short key cannot fail.
#[must_use]
pub fn oracle_key(scenario: &str, width: u16, height: u16, profile: ColorProfile) -> String {
    format!("tablepro/{scenario}/{width}x{height}/{}", profile.label())
}

/// Color-profile expansion: truecolor, 256, 16, explicit mono/`none`, and `NO_COLOR`.
pub const COLOR_PROFILES: [ColorProfile; 5] = [
    ColorProfile::Truecolor,
    ColorProfile::Ansi256,
    ColorProfile::Ansi16,
    ColorProfile::None,
    ColorProfile::NoColor,
];

/// One finite color axis of an ALL-row expansion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorProfile {
    /// `--color truecolor` (and alias `24bit`).
    Truecolor,
    /// `--color 256` (and alias `ansi256`).
    Ansi256,
    /// `--color 16` (and alias `ansi16`).
    Ansi16,
    /// `--color none` (and alias `mono`).
    None,
    /// `NO_COLOR` present with a non-empty value; not a `--color` flag.
    NoColor,
}

impl ColorProfile {
    /// Stable profile label used in observation identity.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Truecolor => "truecolor",
            Self::Ansi256 => "256",
            Self::Ansi16 => "16",
            Self::None => "none",
            Self::NoColor => "nocolor",
        }
    }

    /// `--color` value that selects this profile, if the profile is a CLI flag.
    #[must_use]
    pub const fn cli_color_value(self) -> Option<&'static str> {
        match self {
            Self::Truecolor => Some("truecolor"),
            Self::Ansi256 => Some("256"),
            Self::Ansi16 => Some("16"),
            Self::None => Some("none"),
            Self::NoColor => None,
        }
    }

    /// Production color level for this profile.
    ///
    /// `NoColor` uses the same `ColorLevel::from_env` table as `ColorLevel::detect`.
    #[must_use]
    pub fn color_level(self) -> ColorLevel {
        match self {
            Self::Truecolor => ColorLevel::TrueColor,
            Self::Ansi256 => ColorLevel::Ansi256,
            Self::Ansi16 => ColorLevel::Ansi16,
            Self::None => ColorLevel::Mono,
            Self::NoColor => ColorLevel::from_env(
                None,
                Some(std::ffi::OsStr::new("1")),
                Some("xterm-256color"),
                Some("truecolor"),
                true,
            ),
        }
    }
}
