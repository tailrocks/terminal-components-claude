//! JA-008: workspace edit/delete/prewarm/GitHub keys.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA008_ID: &str = "JA-008";
/// JA-008 sizes.
pub const JA008_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];
/// Worlds in source order.
pub const JA008_WORLDS: [Scenario; 2] = [Scenario::Returning, Scenario::FirstUse];

/// One key-sequence variant from a fresh world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja008Variant {
    /// Variant name.
    pub name: &'static str,
    /// Frames after each listed action.
    pub frames: Vec<ObservedFrame>,
}

/// One world/size of JA-008.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja008Capture {
    /// Starting world.
    pub world: &'static str,
    /// Workspace action variants.
    pub variants: Vec<Ja008Variant>,
}

/// Capture JA-008 at both worlds and sizes.
#[must_use]
pub fn ja008_workspace_actions() -> Vec<Ja008Capture> {
    let mut out = Vec::new();
    for viewport in JA008_SIZES {
        for world in JA008_WORLDS {
            out.push(capture_world(world, viewport));
        }
    }
    out
}

fn capture_world(world: Scenario, viewport: Viewport) -> Ja008Capture {
    Ja008Capture {
        world: world.name(),
        variants: vec![
            capture_keys(world, viewport, "edit", &[KeyCode::Char('e')]),
            capture_keys(
                world,
                viewport,
                "delete-cancel",
                &[KeyCode::Char('d'), KeyCode::Esc],
            ),
            capture_keys(world, viewport, "prewarm", &[KeyCode::Char('w')]),
            capture_keys(world, viewport, "github", &[KeyCode::Char('o')]),
            capture_delete_confirm(world, viewport),
        ],
    }
}

fn open_manager(world: Scenario, viewport: Viewport) -> DirectSession {
    let motion = if world == Scenario::FirstUse {
        Motion::Reduced
    } else {
        Motion::Full
    };
    let mut session = DirectSession::fresh(
        JA008_ID,
        world,
        motion,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    if world == Scenario::FirstUse {
        session.ticks(3);
        session.key(KeyCode::Enter);
    }
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    if let Some((x, y)) = session.find("payments-platform") {
        session.click(x, y);
    } else if world == Scenario::Returning {
        session.key(KeyCode::Down);
    }
    session
}

fn capture_keys(
    world: Scenario,
    viewport: Viewport,
    name: &'static str,
    keys: &[KeyCode],
) -> Ja008Variant {
    let mut session = open_manager(world, viewport);
    let mut frames = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        session.key(*key);
        frames.push(session.observe(&format!("{name}-{index}")));
    }
    Ja008Variant { name, frames }
}

fn capture_delete_confirm(world: Scenario, viewport: Viewport) -> Ja008Variant {
    let mut session = open_manager(world, viewport);
    session.key(KeyCode::Char('d'));
    let after_d = session.observe("delete-confirm-d");
    session.key(KeyCode::Right);
    let after_right = session.observe("delete-confirm-right");
    session.key(KeyCode::Enter);
    let after_enter = session.observe("delete-confirm-enter");
    session.ticks(20);
    let after_ticks = session.observe("delete-confirm-t20");
    Ja008Variant {
        name: "delete-confirm",
        frames: vec![after_d, after_right, after_enter, after_ticks],
    }
}
