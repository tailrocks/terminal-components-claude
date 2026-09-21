//! JA-065: pointer hover versus keyboard focus.
//!
//! Moving the pointer over the manager tree hovers the tree control,
//! while the menubar corner hovers nothing. The first keyboard navigation
//! clears the hover, and clicking a row moves focus to the tree, after
//! which tree selection moves under the keyboard.

use jackin_app::{Motion, Scenario};
use junie_tui::{KeyCode, MouseKind};

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA065_ID: &str = "JA-065";
/// JA-065 sizes.
pub const JA065_SIZES: [Viewport; 1] = [Viewport::new(120, 40)];

/// Hovered control over the manager tree rows.
pub const JA065_TREE_HOVER: &str = "jackin.manager.tree";

/// One size of JA-065.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja065Capture {
    /// Nothing is hovered before the pointer moves.
    pub hover_starts_none: bool,
    /// Hovered control over the tree rows.
    pub hover_tree: Option<String>,
    /// Hovered control over the menubar corner.
    pub hover_corner: Option<String>,
    /// The first keyboard navigation cleared the hover.
    pub keyboard_clears_hover: bool,
    /// Tree selection moved once the tree owned focus.
    pub selection_moves: bool,
    /// Clicking a row moved focus to the tree.
    pub click_focuses: bool,
    /// Checkpoint frames in capture order.
    pub frames: Vec<ObservedFrame>,
}

fn capture_size(viewport: Viewport) -> Ja065Capture {
    let mut frames = Vec::new();
    let mut session = DirectSession::fresh(
        JA065_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    let hover_starts_none = session.hover().is_none();
    frames.push(session.observe("no-hover"));

    session.mouse(MouseKind::Move, 10, 10);
    let hover_tree = session.hover().map(|id| format!("{id:?}"));
    let hovered = session.observe("hover-tree");
    frames.push(hovered);

    session.mouse(MouseKind::Move, 0, 0);
    let hover_corner = session.hover().map(|id| format!("{id:?}"));
    frames.push(session.observe("hover-corner"));

    session.mouse(MouseKind::Move, 10, 10);
    let hover_before_keys = session.hover().is_some();
    session.key(KeyCode::Down);
    let keyboard_clears_hover = hover_before_keys && session.hover().is_none();
    frames.push(session.observe("hover-cleared"));

    session.click(10, 10);
    let clicked = session.observe("clicked");
    let click_focuses = clicked
        .focus
        .as_deref()
        .is_some_and(|focus| focus.contains("manager.tree"));
    let selected_before = clicked.selected_row.clone();
    frames.push(clicked);
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    session.key(KeyCode::Down);
    let moved = session.observe("selection-moved");
    let selection_moves = moved.selected_row != selected_before;
    frames.push(moved);

    Ja065Capture {
        hover_starts_none,
        hover_tree,
        hover_corner,
        keyboard_clears_hover,
        selection_moves,
        click_focuses,
        frames,
    }
}

/// Capture JA-065 at all listed sizes.
#[must_use]
pub fn ja065_hover_and_focus() -> Vec<Ja065Capture> {
    JA065_SIZES.iter().map(|size| capture_size(*size)).collect()
}
