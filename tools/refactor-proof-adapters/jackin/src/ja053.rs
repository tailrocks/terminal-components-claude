//! JA-053: replay the menu bar journey, plus arrows, clicks, and hover.

use jackin_app::{Motion, Scenario};
use junie_tui::KeyCode;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA053_ID: &str = "JA-053";

/// Menu titles in bar order.
pub const JA053_MENUS: [&str; 5] = ["File", "Edit", "View", "Session", "Help"];

/// One size of JA-053.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja053Capture {
    /// Menu bar after `F10`.
    pub bar: ObservedFrame,
    /// After `Right` reaches the next menu.
    pub next_menu: ObservedFrame,
    /// After `Esc` dismisses the bar.
    pub dismissed: ObservedFrame,
    /// After `F10,Enter` opens the spawn picker.
    pub spawn: ObservedFrame,
    /// After `Esc` dismisses the spawn picker.
    pub spawn_dismissed: ObservedFrame,
    /// After clicking `View`.
    pub view_clicked: ObservedFrame,
    /// Coordinate `View` resolved to.
    pub view_at: Option<(u16, u16)>,
    /// After clicking `Usage`.
    pub usage_clicked: ObservedFrame,
    /// Coordinate `Usage` resolved to.
    pub usage_at: Option<(u16, u16)>,
    /// Arrow-walk frame (`Left,Right,Down,Up,Home,End`).
    pub arrows: ObservedFrame,
    /// One frame per menu-title click.
    pub menu_clicks: Vec<ObservedFrame>,
    /// Coordinates each menu title resolved to.
    pub menu_at: Vec<Option<(u16, u16)>>,
    /// Hover owner after moving across the bar.
    pub hover: Option<String>,
    /// After clicking the brand cell.
    pub brand: ObservedFrame,
    /// Coordinate the brand cell resolved to.
    pub brand_at: Option<(u16, u16)>,
}

/// Replay JA-053 at every JA-001 size.
#[must_use]
pub fn ja053_menu_bar() -> Vec<Ja053Capture> {
    crate::JA001_SIZES.into_iter().map(capture_size).collect()
}

fn fresh(viewport: Viewport) -> DirectSession {
    DirectSession::fresh(
        JA053_ID,
        Scenario::CapsuleMulti,
        Motion::Paused,
        0,
        viewport,
        CaptureColor::TrueColor,
    )
}

/// Find a menu-bar title in row 0 only (pane content reuses words like Edit).
fn find_title(session: &DirectSession, title: &str) -> Option<(u16, u16)> {
    session.find(title).filter(|(_, y)| *y == 0)
}

fn capture_size(viewport: Viewport) -> Ja053Capture {
    let mut session = fresh(viewport);
    session.key(KeyCode::F(10));
    let bar = session.observe("bar");
    session.key(KeyCode::Right);
    let next_menu = session.observe("next-menu");
    session.key(KeyCode::Esc);
    let dismissed = session.observe("dismissed");
    session.key(KeyCode::F(10));
    session.key(KeyCode::Enter);
    let spawn = session.observe("spawn");
    session.key(KeyCode::Esc);
    let spawn_dismissed = session.observe("spawn-dismissed");
    let view_at = find_title(&session, "View");
    if let Some((x, y)) = view_at {
        session.click(x, y);
    }
    let view_clicked = session.observe("view-clicked");
    let usage_at = session.find("Usage");
    if let Some((x, y)) = usage_at {
        session.click(x, y);
    }
    let usage_clicked = session.observe("usage-clicked");

    let mut arrows_session = fresh(viewport);
    arrows_session.key(KeyCode::F(10));
    for key in [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::Home,
        KeyCode::End,
    ] {
        arrows_session.key(key);
    }
    let arrows = arrows_session.observe("arrows");

    let mut menu_clicks = Vec::new();
    let mut menu_at = Vec::new();
    for title in JA053_MENUS {
        let mut click_session = fresh(viewport);
        let at = find_title(&click_session, title);
        if let Some((x, y)) = at {
            click_session.click(x, y);
        }
        menu_at.push(at);
        menu_clicks.push(click_session.observe(&format!("menu-{title}")));
    }

    let mut hover_session = fresh(viewport);
    hover_session.key(KeyCode::F(10));
    if let Some((x, y)) = find_title(&hover_session, "Edit") {
        hover_session.mouse(junie_tui::MouseKind::Move, x, y);
    }
    let hover = hover_session.hover().map(|id| format!("{id:?}"));
    let _ = hover_session.observe("hover");

    let mut brand_session = fresh(viewport);
    let brand_at = find_title(&brand_session, "jackin");
    if let Some((x, y)) = brand_at {
        brand_session.click(x, y);
    }
    let brand = brand_session.observe("brand");

    Ja053Capture {
        bar,
        next_menu,
        dismissed,
        spawn,
        spawn_dismissed,
        view_clicked,
        view_at,
        usage_clicked,
        usage_at,
        arrows,
        menu_clicks,
        menu_at,
        hover,
        brand,
        brand_at,
    }
}
