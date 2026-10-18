use crate::pages::Pages;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::MouseKind;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::editor::ROLES;
use junie_tui::ui::modal::{BACKDROP, CLOSE, SURFACE};

fn outside(h: &H) -> Position {
    let area = h.term.backend().buffer().area;
    for y in 1..area.height.saturating_sub(1) {
        for x in 0..area.width {
            let p = Position::new(x, y);
            if matches!(h.app.shell.hits.hit(p), None | Some(BACKDROP)) {
                return p;
            }
        }
    }
    panic!("no free cell outside the modal");
}

fn hard_editor_roles() -> H {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    for _ in 0..8 {
        h.ticks(3);
        if h.app.route() == Route::Manager {
            break;
        }
        h.key(KeyCode::Enter);
    }
    assert!(h.app.world.roles.len() > 100);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('3'));
    h
}

#[test]
fn the_wheel_scrolls_the_roles_list_without_snapping_back_to_the_cursor() {
    let mut h = hard_editor_roles();
    let area = h
        .app
        .shell
        .hits
        .area_of(ROLES)
        .expect("the Roles list must be on screen");
    let at = Position::new(area.x + 2, area.y + 1);
    let first_row = |t: &str| t.lines().nth(area.y as usize).unwrap_or("").to_owned();
    let top = first_row(&h.text());
    for _ in 0..3 {
        h.mouse(MouseKind::WheelDown, at.x, at.y);
    }
    let scrolled = first_row(&h.text());
    assert_ne!(scrolled, top, "the wheel did not move the Roles list");

    h.ticks(2);
    h.mouse(MouseKind::Move, at.x, at.y);
    assert_eq!(
        h.app.shell.hits.area_of(ROLES),
        Some(area),
        "the list moved"
    );
    assert_eq!(
        first_row(&h.text()),
        scrolled,
        "the view snapped back to the cursor"
    );
}

#[test]
fn help_closes_by_its_close_button_its_done_button_and_a_click_beside_it() {
    let open = || {
        let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
        h.key(KeyCode::Char('?'));
        assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
        h
    };
    let mut h = open();
    let frame = h.app.shell.hits.area_of(SURFACE).expect("help frame");
    assert!(
        frame.width <= 96 && frame.height <= 32,
        "help covers the page: {frame:?}"
    );
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("help draws a close button");
    assert_eq!(x.y, frame.y, "the close button sits in the top border");
    assert!(h.text().lines().nth(x.y as usize).unwrap().contains('×'));
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("Keyboard shortcuts"), "× left help open");

    let mut h = open();
    let (dx, dy) = h.find("Done").expect("help draws a Done button");
    h.click(dx, dy);
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "Done left help open"
    );

    let mut h = open();
    let p = outside(&h);
    h.click(p.x, p.y);
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "a click beside help left it open"
    );
}

fn new_account_form() -> H {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Accounts);
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("New account"), "{}", h.text());
    h
}

#[test]
fn a_form_closes_by_the_close_button_in_its_frame() {
    let mut h = new_account_form();
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("form draws a close button");
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("New account"), "{}", h.text());
    assert_eq!(h.app.route(), Route::Accounts);
}

#[test]
fn a_refused_outside_click_flashes_the_form_frame() {
    let mut h = new_account_form();
    let frame = h.app.shell.hits.area_of(SURFACE).expect("form frame");
    let corner = |h: &H| h.term.backend().buffer()[(frame.x, frame.y)].fg;
    let rest = corner(&h);
    let p = outside(&h);
    h.mouse(MouseKind::Down, p.x, p.y);
    h.mouse(MouseKind::Up, p.x, p.y);
    assert!(
        h.text().contains("New account"),
        "the refused click closed it"
    );
    assert_eq!(corner(&h), h.app.theme.warning, "the frame did not flash");
    assert_ne!(rest, h.app.theme.warning);

    h.ticks(20);
    assert_eq!(corner(&h), rest);
}

#[test]
fn a_click_skips_the_intro() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Intro);
    h.ticks(3);
    h.click(60, 20);
    assert_eq!(h.app.route(), Route::Manager, "{}", h.text());

    let mut off = H::new(Scenario::FirstUse, Motion::Reduced, 0, 120, 40);
    off.app.shell.pointer.config = off.app.shell.pointer.config.click_skips_animation(false);
    off.ticks(3);
    off.click(60, 20);
    assert_eq!(off.app.route(), Route::Intro);
}

#[test]
fn the_cockpit_cancels_by_button_and_the_build_log_closes_by_its_close_button() {
    let mut h = H::new(Scenario::LaunchRunning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Cockpit);
    let (x, y) = h.find("Cancel launch…").expect("a Cancel launch button");
    h.click(x + 1, y);
    assert!(h.text().contains("Cancel the launch?"), "{}", h.text());
    h.key(KeyCode::Esc);

    for _ in 0..80 {
        if h.app
            .shell
            .hits
            .area_of(crate::screens::cockpit::CHIP_ACTIVITY)
            .is_some()
        {
            break;
        }
        h.ticks(3);
    }
    let chip = h
        .app
        .shell
        .hits
        .area_of(crate::screens::cockpit::CHIP_ACTIVITY)
        .expect("the activity line links to the build log");
    h.click(chip.x, chip.y);
    assert!(h.text().contains("Docker build"), "{}", h.text());
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("the log has a close button");
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("Docker build"), "{}", h.text());
}

#[test]
fn a_taken_over_capsule_reconnects_by_mouse() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Capsule);
    h.app.router.capsule_mut().unwrap().takeover = Some("laptop".into());
    h.draw();
    assert!(h.text().contains("Attach taken over"));
    let r = h
        .app
        .shell
        .hits
        .area_of(crate::screens::capsule::TAKEOVER_RECONNECT)
        .expect("a Reconnect button");
    assert!(
        h.text()
            .lines()
            .nth(r.y as usize)
            .unwrap()
            .contains("Reconnect")
    );
    h.click(r.x + 1, r.y);
    assert!(!h.text().contains("Attach taken over"), "{}", h.text());
    assert_eq!(h.app.route(), Route::Capsule);

    h.app.router.capsule_mut().unwrap().takeover = Some("laptop".into());
    h.draw();
    let r = h
        .app
        .shell
        .hits
        .area_of(crate::screens::capsule::TAKEOVER_LEAVE)
        .expect("a Workspace manager button");
    h.click(r.x + 1, r.y);
    assert_ne!(h.app.route(), Route::Capsule, "{}", h.text());
}

#[test]
fn an_app_modal_frame_closes_by_its_close_button() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let chip = h
        .app
        .shell
        .hits
        .area_of(crate::screens::capsule::USAGE_CHIP)
        .expect("the usage chip");
    let x = (chip.x..chip.right())
        .find(|&x| {
            h.app.shell.hits.hit(Position::new(x, chip.y))
                == Some(crate::screens::capsule::USAGE_CHIP)
        })
        .expect("a cell of the usage chip takes the click");
    h.click(x, chip.y);
    assert!(h.text().contains("read-only projection"), "{}", h.text());
    let x = h
        .app
        .shell
        .hits
        .area_of(CLOSE)
        .expect("the Usage modal has ×");
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("read-only projection"), "{}", h.text());
    assert_eq!(h.app.route(), Route::Capsule);
}

fn roster_rows(h: &H) -> Vec<(u16, u16, String)> {
    let Some(card) = h.app.shell.hits.area_of(crate::screens::manager::ROSTER) else {
        return vec![];
    };
    let mut rows = vec![];
    for (y, line) in h.text().lines().enumerate() {
        let Some(at) = line.find("◉ ") else {
            continue;
        };
        let x = line[..at].chars().count() as u16;
        if !card.contains(Position::new(x, y as u16)) {
            continue;
        }
        let rest = &line[at + "◉ ".len()..];
        let id = rest.split_whitespace().next().unwrap_or("").to_owned();
        if !id.is_empty() {
            rows.push((x, y as u16, id));
        }
    }
    rows
}

#[test]
fn a_roster_click_selects_the_instance_of_that_row() {
    use crate::screens::manager::RowKey;
    let mut checked = 0;
    for scenario in Scenario::ALL {
        let mut h = H::new(scenario, Motion::Reduced, 0, 200, 50);
        for _ in 0..8 {
            h.ticks(3);
            if h.app.route() == Route::Manager {
                break;
            }
            h.key(KeyCode::Enter);
        }
        if h.app.route() != Route::Manager {
            continue;
        }
        h.draw();
        let rows = roster_rows(&h);
        if rows.len() < 2 {
            continue;
        }
        for (x, y, short) in rows.iter().rev() {
            h.click(*x, *y);
            let RowKey::Instance(id) = &h.app.router.manager().selected else {
                panic!(
                    "{scenario:?}: the row at y {y} selected {:?}",
                    h.app.router.manager().selected
                );
            };
            assert_eq!(
                id.trim_start_matches("jk-"),
                short,
                "{scenario:?}: row at y {y}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 2, "no scenario shows two running instances");
}

#[test]
fn the_wheel_over_a_form_field_scrolls_the_form() {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 80, 24);
    h.key(KeyCode::Char('a'));
    let (x, y) = h.find("Purpose label").expect("the account form");

    for _ in 0..4 {
        h.mouse(MouseKind::WheelDown, x + 4, y);
    }
    let moved = h.find("Purpose label").is_none_or(|(_, y2)| y2 < y);
    assert!(moved, "the form did not scroll:\n{}", h.text());
}
