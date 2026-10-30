use std::time::{Duration, Instant};

use tuiscotti::Frame;
use tuiscotti::render::frame_from_screen;

use super::{DEFAULT_WAIT, Session, screen_text, wait_screen};

const POLL: Duration = Duration::from_millis(25);

pub fn wait_state(s: &mut Session, needle: &str, what: &str) {
    wait_screen(
        s,
        DEFAULT_WAIT,
        &format!("{what}: `{needle}` never appeared"),
        |screen| screen_text(screen).contains(needle),
    );
}

pub fn wait_gone(s: &mut Session, needle: &str, what: &str) {
    let seen = screen_text(
        &s.inner
            .snapshot()
            .unwrap_or_else(|e| panic!("{what}: live sample failed: {e:#}")),
    )
    .contains(needle);
    assert!(
        seen,
        "{what}: `{needle}` never appeared — nothing to expire (vacuous absence)"
    );
    wait_screen(
        s,
        DEFAULT_WAIT,
        &format!("{what}: `{needle}` never expired"),
        |screen| !screen_text(screen).contains(needle),
    );
}

pub fn wait_size(s: &mut Session, cols: u16, rows: u16, what: &str) {
    wait_screen(
        s,
        DEFAULT_WAIT,
        &format!("{what}: never reached {cols}x{rows}"),
        |screen| screen.cols() == cols && screen.rows() == rows,
    );
}

pub fn wait_frame_where(
    s: &mut Session,
    timeout: Duration,
    what: &str,
    mut pred: impl FnMut(&Frame) -> bool,
) -> Frame {
    let deadline = Instant::now() + timeout;
    let sample = |s: &Session| {
        frame_from_screen(
            &s.inner
                .snapshot()
                .unwrap_or_else(|e| panic!("{what}: live sample failed: {e:#}")),
            "default",
        )
    };
    let mut frame = sample(s);
    loop {
        if pred(&frame) {
            return frame;
        }
        if Instant::now() >= deadline {
            panic!(
                "{what}: condition unmet after {} ms; last frame:\n{}",
                timeout.as_millis(),
                frame.text()
            );
        }
        std::thread::sleep(POLL);
        frame = sample(s);
    }
}

pub fn wait_phase(
    s: &mut Session,
    col: u16,
    row: u16,
    options: &[&str],
    timeout: Duration,
    what: &str,
) -> String {
    assert!(
        !options.is_empty(),
        "{what}: wait_phase needs ≥1 phase option"
    );
    let frame = wait_frame_where(s, timeout, what, |frame| {
        frame
            .get(col, row)
            .is_some_and(|c| options.contains(&c.symbol.as_str()))
    });
    frame
        .get(col, row)
        .map(|c| c.symbol.clone())
        .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) vanished after matching"))
}

pub fn wait_phase_change(
    s: &mut Session,
    col: u16,
    row: u16,
    from: &str,
    timeout: Duration,
    what: &str,
) -> String {
    let frame = wait_frame_where(s, timeout, what, |frame| {
        frame
            .get(col, row)
            .is_some_and(|c| c.symbol.as_str() != from && !c.symbol.trim().is_empty())
    });
    frame
        .get(col, row)
        .map(|c| c.symbol.clone())
        .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) vanished after changing"))
}

pub fn wait_quiet(s: &mut Session, quiet: Duration, what: &str) -> Frame {
    super::settle_frame(s, quiet, DEFAULT_WAIT, what)
}
