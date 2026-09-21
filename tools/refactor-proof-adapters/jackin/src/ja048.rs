//! JA-048: capsule new tab and split creation, cancellation, and shell.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA048_ID: &str = "JA-048";

/// Creation variants in contract order: new tab, split right, split below.
pub const JA048_VARIANTS: [char; 3] = ['c', '%', '"'];

/// One size of JA-048.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja048Capture {
    /// Tabs/panes before any creation.
    pub before: (usize, usize),
    /// One picker frame per creation variant.
    pub pickers: Vec<ObservedFrame>,
    /// One frame per completed creation variant.
    pub created: Vec<ObservedFrame>,
    /// Tabs/panes after each completed creation.
    pub after: Vec<(usize, usize)>,
    /// One frame per cancelled picker.
    pub cancelled: Vec<ObservedFrame>,
    /// Shell spawn picker and completion frames.
    pub shell: (ObservedFrame, ObservedFrame),
    /// Tabs/panes after the shell spawn.
    pub shell_after: (usize, usize),
}

/// Capture JA-048 at every JA-001 size.
#[must_use]
pub fn ja048_topology_growth() -> Vec<Ja048Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA048_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

fn topology(session: &DirectSession) -> (usize, usize) {
    session
        .app()
        .world
        .instances
        .iter()
        .find(|instance| instance.status == jackin_app::domain::instance::InstanceStatus::Running)
        .and_then(|instance| session.app().world.daemons.get(&instance.id))
        .map_or((0, 0), |daemon| (daemon.tabs.len(), daemon.panes.len()))
}

fn capture_size(viewport: Viewport) -> Ja048Capture {
    let before = topology(&fresh(viewport));

    let mut pickers = Vec::new();
    let mut created = Vec::new();
    let mut after = Vec::new();
    for (index, key) in JA048_VARIANTS.into_iter().enumerate() {
        let mut session = fresh(viewport);
        session.ctrl('b');
        session.key(KeyCode::Char(key));
        pickers.push(session.observe(&format!("picker-{index}")));
        session.key(KeyCode::Enter);
        session.key(KeyCode::Enter);
        created.push(session.observe(&format!("created-{index}")));
        after.push(topology(&session));
    }

    let mut cancelled = Vec::new();
    for (index, key) in JA048_VARIANTS.into_iter().enumerate() {
        let mut session = fresh(viewport);
        session.ctrl('b');
        session.key(KeyCode::Char(key));
        session.key(KeyCode::Esc);
        cancelled.push(session.observe(&format!("cancelled-{index}")));
    }

    let mut shell = fresh(viewport);
    shell.ctrl('b');
    shell.key(KeyCode::Char('c'));
    let shell_picker = shell.observe("shell-picker");
    // Last row of the agent picker is the plain shell.
    shell.key(KeyCode::End);
    shell.key(KeyCode::Enter);
    let shell_done = shell.observe("shell-done");
    let shell_after = topology(&shell);

    Ja048Capture {
        before,
        pickers,
        created,
        after,
        cancelled,
        shell: (shell_picker, shell_done),
        shell_after,
    }
}
