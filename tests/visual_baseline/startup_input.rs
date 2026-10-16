//! Key presses typed before an app starts (into the launching shell, while
//! the binary builds or loads) must not act on its first screen. Every app
//! starts through `junie_tui::shell::run`, which discards that input before
//! the event loop reads anything (plan S2 drift fix 3).
//!
//! A terminal in its usual line-buffered mode holds keys typed without
//! Enter where `poll` cannot see them; they only become readable once raw
//! mode starts. So the drain has to run in raw mode, or a stray `q` typed
//! before launch still quits the app the moment it starts.
//!
//! `JUNIE_TUI_START_DELAY_MS` (shell test instrumentation) holds the app
//! before that drain, so the test's key is queued "before startup" every
//! time. The key is the app's quit key: if it reaches the app, the app
//! leaves the alternate screen and its first screen disappears.

use std::time::Duration;

use crate::scenarios::Bin;
use crate::support::{self, Case, Color};

fn typed_before_start_is_discarded(name: &'static str, bin: &'static str, needle: &'static str) {
    let case = Case::new(name, bin, &[], 100, 30, Color::Truecolor, needle);
    let mut s = support::spawn_with_env(&case, &[("JUNIE_TUI_START_DELAY_MS", "800")]);
    // typed into a line-buffered terminal, without Enter, before the drain
    s.type_text("q").expect("type");
    support::boot(&mut s, case.needle);
    // the first frame is drawn before the first read: give a leaked `q`
    // time to quit the app before looking again
    std::thread::sleep(Duration::from_millis(600));
    assert!(
        s.find(needle).is_ok(),
        "`{name}`: a key typed before startup reached the app (it quit)"
    );
}

pub fn showcase_discards_keys_typed_before_start() {
    typed_before_start_is_discarded(
        "startup_input/showcase/q/100x30/truecolor",
        Bin::Showcase.path(),
        "Junie Design system",
    );
}

pub fn tablepro_discards_keys_typed_before_start() {
    typed_before_start_is_discarded(
        "startup_input/tablepro/q/100x30/truecolor",
        Bin::Tablepro.path(),
        "Production",
    );
}
