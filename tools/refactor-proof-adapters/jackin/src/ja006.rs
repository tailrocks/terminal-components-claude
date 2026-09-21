//! JA-006: returning manager pointer hits, wheel, and seam drag.

use jackin_app::screens::manager::{DETAIL, TREE};
use jackin_app::{Motion, Scenario};
use junie_tui::{Axis, KeyCode};

use crate::observe::{CaptureColor, DirectSession, Viewport};
use crate::{JA001_SIZES, ObservedFrame};

/// Scenario id.
pub const JA006_ID: &str = "JA-006";

/// Pointer sequence for one size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja006Capture {
    /// After expanding the tree.
    pub expanded: ObservedFrame,
    /// After clicking the payments-platform workspace label, if painted.
    pub click_workspace: ObservedFrame,
    /// After double-clicking that workspace label, if painted.
    pub double_click_workspace: ObservedFrame,
    /// After clicking instance 7f3a, if painted.
    pub click_instance: ObservedFrame,
    /// After three tree-region wheel events.
    pub wheel_tree: ObservedFrame,
    /// After three detail-region wheel events.
    pub wheel_detail: ObservedFrame,
    /// After dragging the manager seam four cells right.
    pub drag_right: ObservedFrame,
    /// After dragging the manager seam four cells left.
    pub drag_left: ObservedFrame,
}

/// Capture JA-006 at every JA-001 size.
#[must_use]
pub fn ja006_returning_manager_pointer() -> Vec<Ja006Capture> {
    JA001_SIZES.into_iter().map(capture_size).collect()
}

fn capture_size(viewport: Viewport) -> Ja006Capture {
    let mut session = DirectSession::fresh(
        JA006_ID,
        Scenario::Returning,
        Motion::Full,
        0,
        viewport,
        CaptureColor::TrueColor,
    );
    session.key(KeyCode::Home);
    session.key(KeyCode::Right);
    let expanded = session.observe("expanded");

    if let Some((x, y)) = session.find("payments-platform") {
        session.click(x, y);
    }
    let click_workspace = session.observe("click-workspace");

    if let Some((x, y)) = session.find("payments-platform") {
        session.double_click(x, y);
    }
    let double_click_workspace = session.observe("double-click-workspace");

    if let Some((x, y)) = session.find("7f3a") {
        session.click(x, y);
    }
    let click_instance = session.observe("click-instance");

    if let Some(area) = session.area_of(TREE) {
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        for _ in 0..3 {
            session.wheel(Axis::V, 1, x, y);
        }
    }
    let wheel_tree = session.observe("wheel-tree");

    if let Some(area) = session.area_of(DETAIL) {
        let x = area.x.saturating_add(area.width / 2);
        let y = area.y.saturating_add(area.height / 2);
        for _ in 0..3 {
            session.wheel(Axis::V, 1, x, y);
        }
    }
    let wheel_detail = session.observe("wheel-detail");

    if let Some(area) = session.area_of(TREE) {
        let x = area.right().saturating_sub(1);
        let y = area.y.saturating_add(area.height / 2);
        session.drag((x, y), (x.saturating_add(4), y));
    }
    let drag_right = session.observe("drag-seam-right");

    if let Some(area) = session.area_of(TREE) {
        let x = area.right().saturating_sub(1);
        let y = area.y.saturating_add(area.height / 2);
        session.drag((x, y), (x.saturating_sub(4), y));
    }
    let drag_left = session.observe("drag-seam-left");

    Ja006Capture {
        expanded,
        click_workspace,
        double_click_workspace,
        click_instance,
        wheel_tree,
        wheel_detail,
        drag_right,
        drag_left,
    }
}
