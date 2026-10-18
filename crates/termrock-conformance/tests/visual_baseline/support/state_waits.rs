#![doc = r" State/frame waits. Replaces fixed sleeps-as-success."]
#![doc = r""]
#![doc = r" Every wait is bounded and fail-closed: presence/absence/geometry waits"]
#![doc = r" ride [`DEFAULT_WAIT`](super::DEFAULT_WAIT), [`wait_frame_where`] and the"]
#![doc = r" phase checks take an explicit per-call deadline. Sleeps appear only as"]
#![doc = r" the 25 ms poll interval inside [`wait_frame_where`] — never as success."]
#![doc = r""]
#![doc = r" Quiet-screen waits are for quiet states only: [`wait_quiet`] on a"]
#![doc = r" live-clock/spinner screen times out *by design* — assert animated"]
#![doc = r" states with [`wait_phase`]/[`wait_phase_change`] instead."]
#![doc = r""]
#![doc = r" Repaint counting has no backend: synchronized output (DEC 2026) is"]
#![doc = r" unsupported, so there is no repaint-wait helper — poll frames instead."]

use std::time::{Duration, Instant};

use tuiscotti::Frame;
use tuiscotti::render::frame_from_screen;

use super::{DEFAULT_WAIT, Session, screen_text, wait_screen};

#[doc = r" Poll interval for the [`Frame`]-based waits. A wake cadence, not a verdict."]
const POLL: Duration = Duration::from_millis(25);

#[doc = r" Wait ([`DEFAULT_WAIT`]) for `needle` on screen. `what` names the state."]
pub fn wait_state(s: &mut Session, needle: &str, what: &str) {
    wait_screen(
        s,
        DEFAULT_WAIT,
        &format!("{what}: `{needle}` never appeared"),
        |screen| screen_text(screen).contains(needle),
    );
}

#[doc = r" Wait ([`DEFAULT_WAIT`]) for `needle` to leave the screen. Fails when"]
#[doc = r" it never appeared first — a wait on an absence that was never present"]
#[doc = r" passes vacuously and proves nothing."]
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

#[doc = r" Wait ([`DEFAULT_WAIT`]) for the emulator to report `cols`×`rows`."]
pub fn wait_size(s: &mut Session, cols: u16, rows: u16, what: &str) {
    wait_screen(
        s,
        DEFAULT_WAIT,
        &format!("{what}: never reached {cols}x{rows}"),
        |screen| screen.cols() == cols && screen.rows() == rows,
    );
}

#[doc = r" Poll the live screen until `pred` holds or `timeout` passes."]
#[doc = r" The bounded primitive for [`Frame`]-shaped assertions predicates"]
#[doc = r" cannot name; returns the matching frame. `what` names it."]
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

#[doc = r" Wait (`timeout`) for cell `(col, row)` to show one of `options`."]
#[doc = r" Returns the observed phase. The animation-aware alternative to"]
#[doc = r" sleeping past a spinner: assert the phase, then [`wait_phase_change`]."]
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

#[doc = r" Wait (`timeout`) for cell `(col, row)` to stop showing `from`."]
#[doc = r" Returns the new symbol. The proof an animation advances a tick."]
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

#[doc = r" Settle a quiet state and return the settled frame. Call only when the"]
#[doc = r" target state is quiet (no clocks, spinners, streams): on an animated"]
#[doc = r" screen this times out by design — use the phase checks there."]
pub fn wait_quiet(s: &mut Session, quiet: Duration, what: &str) -> Frame {
    super::settle_frame(s, quiet, DEFAULT_WAIT, what)
}
