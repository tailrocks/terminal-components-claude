//! Production-state observation for a reached checkpoint.

use junie_tui::App;
use junie_tui_testing::Harness;
use tablepro_app::{Screen, Surface, Tab, TableProApp};

use crate::scenario::ColorProfile;

/// One complete direct-lane observation of production app + frame state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    /// Parent scenario identity.
    pub scenario: String,
    /// Preset that launched the process.
    pub preset: String,
    /// Named checkpoint.
    pub checkpoint: String,
    /// Terminal width.
    pub width: u16,
    /// Terminal height.
    pub height: u16,
    /// Color-profile label.
    pub profile: String,
    /// `TableProApp::screen`.
    pub screen: Screen,
    /// `TableProApp::surface` after the real constructor, not a seeded label.
    pub surface: Surface,
    /// Connection-list filter text.
    pub filter: String,
    /// Selected connection-list row.
    pub selected: usize,
    /// Whether the connection form is mounted.
    pub form_open: bool,
    /// Quit flag.
    pub quit: bool,
    /// Latest status text.
    pub status: String,
    /// Workbench query counter after `App::new`'s `new_query`.
    pub query_counter: usize,
    /// Open tab count.
    pub tab_count: usize,
    /// Active query tab name, when the active tab is a query.
    pub active_query_name: Option<String>,
    /// Focused control hash, if any.
    pub focus: Option<u64>,
    /// Focus-ring ids in registration order.
    pub ring: Vec<u64>,
    /// Cursor cell, when visible.
    pub cursor: Option<(u16, u16)>,
    /// Cell digest of the current frame.
    pub digest: u64,
    /// Plain-text frame, rows joined by newlines.
    pub text: String,
    /// Connection-list error, when present.
    pub error: Option<String>,
    /// Hovered control hash, if any.
    pub hover: Option<u64>,
}

/// Observe the current production harness state at `checkpoint`.
#[must_use]
pub fn observe(
    harness: &Harness<TableProApp>,
    scenario: &str,
    preset: &str,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    let app = harness.app();
    let area = *harness.buffer().area();
    let active_query_name = match app.workbench.active() {
        Some(Tab::Query(tab)) => Some(tab.name.clone()),
        _ => None,
    };
    Observation {
        scenario: scenario.to_owned(),
        preset: preset.to_owned(),
        checkpoint: checkpoint.to_owned(),
        width: area.width,
        height: area.height,
        profile: profile.label().to_owned(),
        screen: app.screen(),
        surface: app.surface(),
        filter: app.connections_screen.filter.clone(),
        selected: app.connections_screen.selected,
        form_open: app.connection_form_open(),
        quit: app.should_quit(),
        status: app.status().to_owned(),
        query_counter: app.workbench.query_counter,
        tab_count: app.workbench.tabs().len(),
        active_query_name,
        focus: harness.focus().map(junie_tui::Id::hash),
        ring: harness
            .ring()
            .entries()
            .iter()
            .map(|entry| entry.id.hash())
            .collect(),
        cursor: harness.cursor().map(|pos| (pos.x, pos.y)),
        digest: harness.snapshot().digest(),
        text: harness.text(),
        error: app.connections_screen.error.clone(),
        hover: harness.hover().map(junie_tui::Id::hash),
    }
}
