#![doc = r" Typed pointer/keyboard input. Replaces raw SGR-1006 escapes."]
#![doc = r""]
#![doc = r" [`Input`] builders (`move_to`/`down`/`up`/`drag`/`wheel`/`focus`/`key`)"]
#![doc = r" check the terminal's reporting mode *before* any byte is sent and fail"]
#![doc = r" the case on refused input — never silently. Delivery goes through the"]
#![doc = r" typed inner-[`Session`] calls, which encode per the app-enabled mode and"]
#![doc = r" refuse with their own mode errors as backstop; the gaps the old"]
#![doc = r" engine left (button-less move, split press/release, focus) are"]
#![doc = r" first-class calls now, so no raw SGR remains. [`Input::Key`] goes"]
#![doc = r" through the wrapper's chord grammar, shared with the send-step loop."]
#![doc = r""]
#![doc = r" Coordinates are `(col, row)`, 0-based, like every inner-session input call."]
#![doc = r" This module never sleeps: pace with [`super::state_waits`], not pauses."]

use tuiscotti::tui::{MouseButton, MouseMods, Wheel};

use super::Session;

#[doc = r" Enabled DEC tracking modes, sampled live (for assertions)."]
pub type MouseModes = Vec<u16>;

#[doc = r" Collapsed reporting protocol (for assertions): the finest tracking the"]
#[doc = r" application enabled. Derived from the live DEC mode set."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    #[doc = r" No mouse tracking."]
    None,
    #[doc = r" Click tracking (`?1000`): press + release, no motion."]
    Press,
    #[doc = r" Press-release tracking (`?1002`; button-motion collapses here —"]
    #[doc = r" the backend reports one number for both)."]
    PressRelease,
    #[doc = r" Any-motion tracking (`?1003`)."]
    AnyMotion,
}

impl MouseMode {
    fn collapse(modes: &[u16]) -> Self {
        if modes.contains(&1003) {
            Self::AnyMotion
        } else if modes.contains(&1002) {
            Self::PressRelease
        } else if modes.contains(&1000) {
            Self::Press
        } else {
            Self::None
        }
    }
}

#[doc = r" One typed input. Build with the verbs, deliver with [`Input::send`]."]
#[derive(Debug, Clone, Copy)]
pub enum Input {
    #[doc = r" Hover to `(col, row)`. Needs any-motion tracking (`?1003`)."]
    Move { col: u16, row: u16 },
    #[doc = r" Button press at `(col, row)`. Needs any tracking."]
    Down {
        button: MouseButton,
        col: u16,
        row: u16,
    },
    #[doc = r" Button release at `(col, row)`. Needs press-release or finer."]
    Up {
        button: MouseButton,
        col: u16,
        row: u16,
    },
    #[doc = r" Primary-button drag from `from` to `to`. Needs press-release or finer."]
    Drag { from: (u16, u16), to: (u16, u16) },
    #[doc = r" `notches` wheel steps at `(col, row)` (≥1). Needs any tracking."]
    Wheel {
        dir: Wheel,
        col: u16,
        row: u16,
        notches: u32,
    },
    #[doc = r" Focus in (`gained`) or out. Needs focus reporting (`?1004`)."]
    Focus { gained: bool },
    #[doc = r" Named key (the send-step vocabulary). No reporting mode."]
    Key { name: &'static str },
}

impl Input {
    #[doc = r" Hover to `(col, row)`."]
    #[must_use]
    pub fn move_to(col: u16, row: u16) -> Self {
        Self::Move { col, row }
    }

    #[doc = r" Press `button` at `(col, row)`."]
    #[must_use]
    pub fn down(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Down { button, col, row }
    }

    #[doc = r" Release `button` at `(col, row)`."]
    #[must_use]
    pub fn up(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Up { button, col, row }
    }

    #[doc = r" Drag from `from` to `to`, primary button."]
    #[must_use]
    pub fn drag(from: (u16, u16), to: (u16, u16)) -> Self {
        Self::Drag { from, to }
    }

    #[doc = r" One wheel notch at `(col, row)`; chain [`.notches(n)`](Self::notches)."]
    #[must_use]
    pub fn wheel(dir: Wheel, col: u16, row: u16) -> Self {
        Self::Wheel {
            dir,
            col,
            row,
            notches: 1,
        }
    }

    #[doc = r" Focus in (`CSI I`)."]
    #[must_use]
    pub fn focus_in() -> Self {
        Self::Focus { gained: true }
    }

    #[doc = r" Focus out (`CSI O`)."]
    #[must_use]
    pub fn focus_out() -> Self {
        Self::Focus { gained: false }
    }

    #[doc = r" Named key press."]
    #[must_use]
    pub fn key(name: &'static str) -> Self {
        Self::Key { name }
    }

    #[doc = r" Repeat a wheel input `n` times (panics on 0 or a non-wheel input)."]
    #[must_use]
    pub fn notches(self, n: u32) -> Self {
        let Self::Wheel { dir, col, row, .. } = self else {
            panic!("notches() applies only to Input::wheel, got {self:?}");
        };
        assert!(n >= 1, "wheel needs ≥1 notch, got {n}");
        Self::Wheel {
            dir,
            col,
            row,
            notches: n,
        }
    }

    #[doc = r" Check the reporting mode, then send. Fails the case when the app"]
    #[doc = r" refused the input (tracking/focus off, off-grid, bad key, dead PTY)."]
    pub fn send(&self, s: &mut Session) {
        match *self {
            Self::Move { col, row } => {
                let modes = require_mode(s, "hover", ModeNeed::Motion);
                require_in_grid("hover", modes.size, &[(col, row)]);
                s.inner
                    .mouse_move(col, row, MouseMods::NONE)
                    .unwrap_or_else(|e| panic!("hover to ({col}, {row}) failed: {e:#}"));
            }
            Self::Down { button, col, row } => {
                let modes = require_mode(s, "press", ModeNeed::AnyTracking);
                require_in_grid("press", modes.size, &[(col, row)]);
                s.inner
                    .mouse_down(button, col, row, MouseMods::NONE)
                    .unwrap_or_else(|e| panic!("press at ({col}, {row}) failed: {e:#}"));
            }
            Self::Up { button, col, row } => {
                let modes = require_mode(s, "release", ModeNeed::Release);
                require_in_grid("release", modes.size, &[(col, row)]);
                s.inner
                    .mouse_up(button, col, row, MouseMods::NONE)
                    .unwrap_or_else(|e| panic!("release at ({col}, {row}) failed: {e:#}"));
            }
            Self::Drag { from, to } => {
                let modes = require_mode(s, "drag", ModeNeed::Release);
                require_in_grid("drag", modes.size, &[from, to]);
                super::drag_path(s, from, to);
            }
            Self::Wheel {
                dir,
                col,
                row,
                notches,
            } => {
                let modes = require_mode(s, "wheel", ModeNeed::AnyTracking);
                require_in_grid("wheel", modes.size, &[(col, row)]);
                for n in 0..notches {
                    s.inner
                        .mouse_wheel(dir, col, row, MouseMods::NONE)
                        .unwrap_or_else(|e| {
                            panic!(
                                "wheel notch {}/{notches} at ({col}, {row}) failed: {e:#}",
                                n + 1
                            )
                        });
                }
            }
            Self::Focus { gained } => {
                require_focus(s, gained);
                let what = if gained { "focus-in" } else { "focus-out" };
                let outcome = if gained {
                    s.inner.focus_in()
                } else {
                    s.inner.focus_out()
                };
                outcome.unwrap_or_else(|e| panic!("{what} failed: {e:#}"));
            }
            Self::Key { name } => {
                s.send_key(name)
                    .unwrap_or_else(|e| panic!("key `{name}` failed: {e:#}"));
            }
        }
    }
}

#[doc = r" What reporting mode an input needs from the application."]
#[derive(Debug, Clone, Copy)]
enum ModeNeed {
    #[doc = r" Any tracking at all (`?1000`/`?1002`/`?1003`)."]
    AnyTracking,
    #[doc = r" Any-motion (`?1003`): only it reports button-less motion."]
    Motion,
    #[doc = r" Press-release or finer (`?1000`/`?1002`/`?1003`): X10-style"]
    #[doc = r" press-only tracking has no release."]
    Release,
}

#[doc = r" Live reporting state, sampled without waiting."]
struct Sampled {
    modes: MouseModes,
    mode: MouseMode,
    focus: bool,
    size: (u16, u16),
}

#[doc = r" Sample the live reporting state from one fresh observation."]
fn sample(s: &mut Session) -> Sampled {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("sample live screen: {e:#}"));
    let mut modes = obs.state.modes.known().cloned().unwrap_or_default();
    modes.sort_unstable();
    Sampled {
        mode: MouseMode::collapse(&modes),
        focus: modes.contains(&1004),
        size: (obs.screen.cols(), obs.screen.rows()),
        modes,
    }
}

#[doc = r" Enabled tracking set (for assertions). See [`sample`]."]
#[must_use]
pub fn tracking(s: &mut Session) -> MouseModes {
    sample(s).modes
}

#[doc = r" Collapsed reporting protocol (for assertions). See [`sample`]."]
#[must_use]
pub fn tracking_mode(s: &mut Session) -> MouseMode {
    sample(s).mode
}

#[doc = r" Whether focus reporting (`?1004`) is on (for assertions). See [`sample`]."]
#[must_use]
pub fn focus_reporting(s: &mut Session) -> bool {
    sample(s).focus
}

#[doc = r" Fail the case unless the app enabled what `what` needs. Returns the"]
#[doc = r" sample so the caller can bounds-check against the same observation."]
fn require_mode(s: &mut Session, what: &str, need: ModeNeed) -> Sampled {
    let sampled = sample(s);
    let ok = match need {
        ModeNeed::AnyTracking => sampled.mode != MouseMode::None,
        ModeNeed::Motion => sampled.mode == MouseMode::AnyMotion,
        ModeNeed::Release => !matches!(sampled.mode, MouseMode::None),
    };
    assert!(
        ok,
        "{what} refused: app reports in {:?} (modes {:?}), needs {need:?}",
        sampled.mode, sampled.modes,
    );
    sampled
}

#[doc = r" Fail the case unless focus reporting is on."]
fn require_focus(s: &mut Session, gained: bool) {
    let what = if gained { "focus-in" } else { "focus-out" };
    assert!(
        sample(s).focus,
        "{what} refused: app never enabled focus reporting (no CSI ?1004 h seen)"
    );
}

#[doc = r" Fail the case when any point is outside the sampled grid."]
fn require_in_grid(what: &str, (cols, rows): (u16, u16), pts: &[(u16, u16)]) {
    for &(col, row) in pts {
        assert!(
            col < cols && row < rows,
            "{what} at ({col}, {row}) is outside the {cols}x{rows} grid"
        );
    }
}

#[doc = r" Live smoke: one session through all three helpers (PTY, ignored like"]
#[doc = r" every capture test). Exercises typed move/down/up/drag/wheel/focus/key,"]
#[doc = r" unique-target resolve + refresh after scroll/input, and state waits."]
#[test]
#[ignore = "typed-helpers smoke; run with --ignored"]
fn smoke_typed_helpers() {
    use std::time::Duration;

    use super::scoped_targets::Target;
    use super::state_waits as waits;
    use crate::support::{Case, Color, SHOWCASE, spawn_boot};

    let case = Case::new(
        "smoke/typed_helpers",
        SHOWCASE,
        &["--page", "buttons"],
        120,
        40,
        Color::Truecolor,
        "Junie Design system",
    );
    let mut s = spawn_boot(&case);
    waits::wait_state(&mut s, case.needle, "smoke boot");

    assert_eq!(
        tracking_mode(&mut s),
        MouseMode::AnyMotion,
        "showcase must enable any-motion tracking"
    );
    eprintln!(
        "smoke: modes={:?} protocol={:?} focus={}",
        tracking(&mut s),
        tracking_mode(&mut s),
        focus_reporting(&mut s)
    );

    let focus_on = focus_reporting(&mut s);
    for (name, input) in [("in", Input::focus_in()), ("out", Input::focus_out())] {
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            input.send(&mut s);
        }));
        assert_eq!(
            attempt.is_ok(),
            focus_on,
            "focus-{name} must send iff 1004 is on"
        );
    }

    let mut target = Target::resolve(&mut s, "Preview");
    eprintln!("smoke: Preview at {:?}", target.point());
    assert_eq!(target.point(), (target.col(), target.row()));
    assert_eq!(target.grid(), (120, 40));
    let (col, row) = target.point();
    Input::move_to(col, row).send(&mut s);

    Input::wheel(Wheel::Down, col, row).notches(2).send(&mut s);
    target.refresh(&mut s);

    Input::key("down").send(&mut s);
    target.refresh_same(&mut s);

    s.resize(100, 30).expect("resize down");
    waits::wait_size(&mut s, 100, 30, "smoke shrink");
    target.refresh_same(&mut s);
    s.resize(120, 40).expect("resize up");
    waits::wait_size(&mut s, 120, 40, "smoke grow");
    target.refresh(&mut s);

    let vacuous = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        waits::wait_gone(&mut s, "needle-that-never-exists-zzz", "smoke vacuous");
    }));
    assert!(vacuous.is_err(), "wait_gone must fail vacuous absence");

    let frame = waits::wait_frame_where(&mut s, Duration::from_secs(2), "smoke frame", |_| true);
    assert_eq!((frame.cols, frame.rows), (120, 40));

    let phase = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        waits::wait_phase(
            &mut s,
            0,
            0,
            &["never-a-phase-zzz"],
            Duration::from_millis(150),
            "smoke phase-timeout",
        );
    }));
    assert!(phase.is_err(), "impossible phase must time out");

    let settled =
        waits::wait_frame_where(&mut s, Duration::from_secs(2), "smoke phase-cell", |_| true);
    let settled_symbol = settled
        .get(0, 0)
        .map(|c| c.symbol.clone())
        .unwrap_or_default();
    let no_change = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        waits::wait_phase_change(
            &mut s,
            0,
            0,
            &settled_symbol,
            Duration::from_millis(150),
            "smoke phase-change-timeout",
        );
    }));
    assert!(
        no_change.is_err(),
        "phase change on a quiet cell must time out"
    );

    Input::down(MouseButton::Left, col, row).send(&mut s);
    Input::up(MouseButton::Left, col, row).send(&mut s);
    waits::wait_state(&mut s, case.needle, "smoke after click");
    let target = Target::resolve(&mut s, "Preview");
    Input::drag(target.point(), target.span_end()).send(&mut s);

    let frame = waits::wait_quiet(&mut s, Duration::from_millis(400), "smoke settle");
    assert_eq!((frame.cols, frame.rows), (120, 40));
    assert!(frame.text().contains(case.needle));
}
