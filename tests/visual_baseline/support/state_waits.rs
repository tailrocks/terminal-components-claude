//! State/frame waits. Replaces fixed sleeps-as-success.
//!
//! Every wait is bounded and fail-closed: presence/absence/geometry waits
//! ride the session deadline, [`wait_frame_where`] and the phase checks
//! take an explicit per-call deadline. Sleeps appear only as the 25 ms
//! poll interval inside [`wait_frame_where`] — never as success.
//!
//! Quiet-screen waits are for quiet states only: [`wait_quiet`] on a
//! live-clock/spinner screen times out *by design* — assert animated
//! states with [`wait_phase`]/[`wait_phase_change`] instead.

use std::time::{Duration, Instant};

use tuisnap::pty::Session;
use tuisnap::Frame;

/// Poll interval for the [`Frame`]-based waits. A wake cadence, not a verdict.
const POLL: Duration = Duration::from_millis(25);

/// Wait (session deadline) for `needle` on screen. `what` names the state.
pub fn wait_state(s: &mut Session, needle: &str, what: &str) {
    s.wait_for_text(needle)
        .unwrap_or_else(|e| panic!("{what}: `{needle}` never appeared: {e:#}"));
}

/// Wait (session deadline) for `needle` to leave the screen. Fails when
/// it never appeared first — a wait on an absence that was never present
/// passes vacuously and proves nothing.
pub fn wait_gone(s: &mut Session, needle: &str, what: &str) {
    let mut seen = false;
    s.wait_until(|screen| {
        seen = screen.contains(needle);
        true
    })
    .expect("sample live screen");
    assert!(
        seen,
        "{what}: `{needle}` never appeared — nothing to expire (vacuous absence)"
    );
    s.wait_until(|screen| !screen.contains(needle))
        .unwrap_or_else(|e| panic!("{what}: `{needle}` never expired: {e:#}"));
}

/// Wait (session deadline) for the emulator to report `cols`×`rows`.
pub fn wait_size(s: &mut Session, cols: u16, rows: u16, what: &str) {
    s.wait_until(|screen| screen.size() == (cols, rows))
        .unwrap_or_else(|e| panic!("{what}: never reached {cols}x{rows}: {e:#}"));
}

/// Wait (session deadline) for the repaint count to advance by `n` — the
/// proof an input made the app paint. Counts DEC 2026 synchronized
/// updates only: the current apps never emit them (measured: no advance
/// after key input on showcase), so this is for future DEC 2026 emitters —
/// the timeout names that (not a hang, a diagnosis).
pub fn wait_repaints(s: &mut Session, n: u64, what: &str) {
    assert!(n >= 1, "{what}: wait_repaints needs ≥1 repaint, got {n}");
    let mut before = 0;
    s.wait_until(|screen| {
        before = screen.repaints();
        true
    })
    .expect("sample live screen");
    s.wait_until(|screen| screen.repaints() >= before + n)
        .unwrap_or_else(|e| {
            panic!("{what}: no {n} repaint(s) after {before} (app may not emit DEC 2026): {e:#}")
        });
}

/// Poll [`Session::snapshot`] until `pred` holds or `timeout` passes.
/// The bounded primitive for [`Frame`]-shaped assertions termlens
/// predicates cannot name; returns the matching frame. `what` names it.
pub fn wait_frame_where(
    s: &mut Session,
    timeout: Duration,
    what: &str,
    mut pred: impl FnMut(&Frame) -> bool,
) -> Frame {
    let deadline = Instant::now() + timeout;
    let mut frame = s.snapshot();
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
        frame = s.snapshot();
    }
}

/// Wait (`timeout`) for cell `(col, row)` to show one of `options`.
/// Returns the observed phase. The animation-aware alternative to
/// sleeping past a spinner: assert the phase, then [`wait_phase_change`].
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

/// Wait (`timeout`) for cell `(col, row)` to stop showing `from`.
/// Returns the new symbol. The proof an animation advances a tick.
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

/// Settle a quiet state and return the gated frame. Call only when the
/// target state is quiet (no clocks, spinners, streams): on an animated
/// screen this times out by design — use the phase checks there.
pub fn wait_quiet(s: &mut Session, quiet: Duration, what: &str) -> Frame {
    s.wait_stable(quiet).unwrap_or_else(|e| {
        panic!("{what}: never settled (animated state? use phase waits): {e:#}")
    })
}
