//! JA-007: returning manager row activation variants.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA007_ID: &str = "JA-007";
/// JA-007 sizes.
pub const JA007_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// Labels selected then activated with Enter, in source variant order.
pub const JA007_LABELS: [&str; 6] = [
    "Current directory",
    "payments-platform",
    "7f3a",
    "stopped",
    "pane",
    "+ New workspace",
];

/// One activation variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja007Variant {
    /// Label requested by the source variant.
    pub label: &'static str,
    /// Frame after select.
    pub selected: ObservedFrame,
    /// Frame after Enter.
    pub activated: ObservedFrame,
}

/// One size of JA-007.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja007Capture {
    /// All listed activation variants from fresh worlds.
    pub variants: Vec<Ja007Variant>,
}

/// Capture JA-007 at both listed sizes.
#[must_use]
pub fn ja007_returning_row_activation() -> Vec<Ja007Capture> {
    JA007_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja007Capture {
    let variants = JA007_LABELS
        .into_iter()
        .map(|label| capture_variant(viewport, label))
        .collect();
    Ja007Capture { variants }
}

fn capture_variant(viewport: Viewport, label: &'static str) -> Ja007Variant {
    let mut session = DirectSession::fresh(
        JA007_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    match label {
        "Current directory" => {
            session.key(KeyCode::Home);
        }
        "+ New workspace" => {
            if let Some((x, y)) = session.find(label) {
                session.click(x, y);
            } else {
                session.key(KeyCode::End);
            }
        }
        _ => {
            if let Some((x, y)) = session.find(label) {
                session.click(x, y);
            }
        }
    }
    let checkpoint = format!("select-{label}");
    let selected = session.observe(&checkpoint);
    session.key(KeyCode::Enter);
    let checkpoint = format!("enter-{label}");
    let activated = session.observe(&checkpoint);
    Ja007Variant {
        label,
        selected,
        activated,
    }
}
