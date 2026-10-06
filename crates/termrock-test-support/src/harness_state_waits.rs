//! State waits with explicit deadlines for [`Harness`](crate::harness::Harness).
//!
//! Mirror of the reference `state_waits` helper, adapted to the candidate
//! harness API (audit G3/P3). The reference replaces terminal `sleep:N`
//! sends with `wait:` needles on a wall-clock deadline; the candidate
//! harness runs on an explicit clock, so waits here advance simulated
//! time instead of sleeping. Each step delivers one update tick at the
//! current moment and then jumps the clock to the next armed deadline
//! (or a 10 ms step), so timer-driven and tick-driven apps both make
//! progress. A wait that exhausts its deadline panics with the full
//! screen text, mirroring the conformance `wait:` failure shape.

use core::time::Duration;

use termrock::{App, Id, StateFlags};

use crate::harness::Harness;

/// Default wait deadline: 8 s of simulated time, mirroring the
/// `timeout_ms.max(8_000)` precedent.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);

/// Clock step when no runtime deadline is armed.
const STEP: Duration = Duration::from_millis(10);

/// Advance simulated time until `pred` holds or `timeout` elapses.
///
/// The predicate sees the settled frame after every tick and every clock
/// jump. Panics with `label` and the full screen text on timeout.
pub fn wait_until<A: App>(
    h: &mut Harness<A>,
    mut pred: impl FnMut(&Harness<A>) -> bool,
    timeout: Duration,
    label: &str,
) {
    let deadline = h.now().saturating_add(timeout);
    loop {
        if pred(h) {
            return;
        }
        let now = h.now();
        if now >= deadline {
            break;
        }
        let _ = h.tick();
        if pred(h) {
            return;
        }
        let step_to = match h.next_deadline() {
            Some(d) if d > now => d.min(deadline),
            _ => now.saturating_add(STEP).min(deadline),
        };
        let _ = h.advance_to(step_to).expect("forward clock movement");
    }
    panic!(
        "state wait '{label}' timed out after {timeout:?}\n--- SCREEN TEXT ---\n{}\n--- END SCREEN TEXT ---",
        h.text()
    );
}

/// Advance simulated time until `needle` appears on screen.
pub fn wait_for_text<A: App>(h: &mut Harness<A>, needle: &str, timeout: Duration) {
    assert!(!needle.is_empty(), "state wait: needle must not be empty");
    wait_until(
        h,
        |h| h.text().contains(needle),
        timeout,
        &format!("text '{needle}'"),
    );
}

/// Advance simulated time until `id` owns focus.
pub fn wait_for_focus<A: App>(h: &mut Harness<A>, id: Id, timeout: Duration) {
    wait_until(
        h,
        |h| h.focus() == Some(id),
        timeout,
        &format!("focus on {id:?}"),
    );
}

/// Advance simulated time until `id` carries at least `flags`.
pub fn wait_for_state<A: App>(h: &mut Harness<A>, id: Id, flags: StateFlags, timeout: Duration) {
    wait_until(
        h,
        |h| h.state_of(id).contains(flags),
        timeout,
        &format!("state {flags:?} on {id:?}"),
    );
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;
    use termrock::{
        Cx, Family, Focusability, FrameRead, ItemKey, Response, RowUi, Theme, Ui, Variant,
    };

    use super::*;

    const R0: Id = Id::root("waits.r0");

    struct Slow {
        ticks: u32,
    }

    impl App for Slow {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            self.ticks = self.ticks.saturating_add(1);
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let row = Rect::new(0, 0, 20, 1);
            ui.register_control(R0, row, Focusability::Focusable);
            let mut r = RowUi::new(
                ui,
                R0,
                Family::LIST,
                Variant::DEFAULT,
                ui.state(R0),
                ItemKey::index(0),
                row,
            );
            r.gutter();
            r.label(if self.ticks >= 3 { "ready" } else { "booting" });
        }
    }

    fn harness() -> Harness<Slow> {
        Harness::new(Slow { ticks: 0 }, Theme::junie(), 20, 3)
    }

    #[test]
    fn text_appearing_after_ticks_is_found() {
        let mut h = harness();
        wait_for_text(&mut h, "ready", Duration::from_secs(1));
        assert!(h.text().contains("ready"));
    }

    #[test]
    #[should_panic(expected = "timed out after 50ms")]
    fn missing_text_times_out_with_screen_text() {
        let mut h = harness();
        wait_for_text(&mut h, "never-appears", Duration::from_millis(50));
    }

    #[test]
    fn focus_wait_returns_once_focused() {
        let mut h = harness();
        assert!(h.tab_to(R0));
        wait_for_focus(&mut h, R0, Duration::from_millis(50));
    }

    #[test]
    #[should_panic(expected = "timed out")]
    fn focus_wait_on_unknown_id_times_out() {
        let mut h = harness();
        wait_for_focus(&mut h, Id::root("waits.nope"), Duration::from_millis(30));
    }

    #[test]
    fn state_wait_sees_focused_flag() {
        let mut h = harness();
        assert!(h.tab_to(R0));
        wait_for_state(&mut h, R0, StateFlags::FOCUSED, Duration::from_millis(50));
    }
}
