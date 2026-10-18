


use junie_tui::runtime::Application;
use std::time::Duration;

use clap::Parser;
use junie_tui::core::event::{Input, Outcome};
use junie_tui::core::id::WidgetId;
use ratatui::crossterm::event::KeyCode;

use crate::Cli;
use crate::app::Motion;
use crate::app_tests::H;

fn parse(args: &[&str]) -> Result<(Motion, u64), clap::Error> {
    let cli = Cli::try_parse_from(std::iter::once("tablepro").chain(args.iter().copied()))?;
    Ok((cli.common.motion.unwrap_or_default(), cli.common.frame))
}



fn loading() -> H {
    let mut h = H::connected(120, 40);
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Right);
    assert!(h.app.animating(), "expanding a table starts a column load");
    h
}

#[test]
fn motion_and_frame_flags_parse_like_the_other_binaries() {
    assert_eq!(parse(&[]).unwrap(), (Motion::Full, 0));
    assert_eq!(
        parse(&["--motion", "paused", "--frame", "40"]).unwrap(),
        (Motion::Paused, 40)
    );
    assert_eq!(
        parse(&["-m", "reduced", "-f", "3"]).unwrap(),
        (Motion::Reduced, 3)
    );
    assert_eq!(parse(&["--motion", "full"]).unwrap(), (Motion::Full, 0));
    assert!(parse(&["--motion", "slow"]).is_err());
    assert!(parse(&["--frame", "-1"]).is_err());
}



#[test]
fn paused_freezes_the_frame() {
    let mut live = loading();
    let before = live.text();
    live.ticks(1);
    assert_ne!(live.text(), before, "a live tick advances the spinner");

    let mut h = loading();
    h.app.set_motion(Motion::Paused, 0);
    h.draw();
    let pinned = h.text();
    for _ in 0..50 {
        assert_eq!(h.app.handle(Input::Tick), Outcome::Ignored);
    }
    h.draw();
    assert_eq!(h.text(), pinned);
    assert!(
        h.app.workbench.as_ref().unwrap().animating(),
        "still loading"
    );
}



#[test]
fn paused_frame_matches_live_ticks() {
    let mut live = loading();
    live.ticks(3);
    let mut paused = loading();
    paused.app.set_motion(Motion::Paused, 3);
    paused.draw();
    assert_eq!(paused.app.tick, 3);
    assert!(!paused.app.workbench.as_ref().unwrap().animating());
    assert_eq!(paused.text(), live.text());
}



#[test]
fn paused_holds_the_press_flash() {
    let id = WidgetId::of("motion-test");
    let mut full = H::connected(80, 24);
    full.app.shell.pointer.flash(id, 0);
    let mut paused = H::connected(80, 24);
    paused.app.set_motion(Motion::Paused, 0);
    paused.app.shell.pointer.flash(id, 0);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(full.app.interaction().flash, None);
    assert_eq!(paused.app.interaction().flash, Some(id));
}



#[test]
fn reduced_advances_the_world_without_animating_glyphs() {
    let mut h = loading();
    h.app.set_motion(Motion::Reduced, 0);
    let before = h.text();
    h.ticks(1);
    assert_eq!(
        h.text(),
        before,
        "no spinner animation under reduced motion"
    );
    h.ticks(2);
    assert_eq!(h.app.tick, 0);
    assert!(!h.app.workbench.as_ref().unwrap().animating());
}
