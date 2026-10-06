//! Typed pointer/keyboard input. Replaces raw SGR-1006 escapes.
//!
//! [`Input`] builders (`move_to`/`down`/`up`/`drag`/`wheel`/`focus`/`key`,
//! the tuiscotti verb set) check the terminal's reporting mode *before*
//! any byte is sent and fail the case on refused input — never silently.
//! Gestures termlens models (`drag`, `wheel`) delegate to [`Session`], which
//! encodes per the app-enabled mode; the gaps it leaves (button-less move,
//! split press/release, focus) are emitted as SGR here, mode-gated first.
//!
//! Coordinates are `(col, row)`, 0-based, like every [`Session`] input call.
//! This module never sleeps: pace with [`super::state_waits`], not pauses.

use tuisnap::pty::{MouseButton, Scroll, Session};
use tuisnap::termlens::{MouseMode, MouseModes};

/// Button-less motion report code: release-all (3) + motion flag (32).
const MOTION_CODE: u8 = 35;

/// One typed input. Build with the verbs, deliver with [`Input::send`].
#[derive(Debug, Clone, Copy)]
pub enum Input {
    /// Hover to `(col, row)`. Needs any-motion tracking (`?1003`).
    Move { col: u16, row: u16 },
    /// Button press at `(col, row)`. Needs any tracking.
    Down {
        button: MouseButton,
        col: u16,
        row: u16,
    },
    /// Button release at `(col, row)`. Needs press-release or finer.
    Up {
        button: MouseButton,
        col: u16,
        row: u16,
    },
    /// Primary-button drag from `from` to `to`. Needs press-release or finer.
    Drag { from: (u16, u16), to: (u16, u16) },
    /// `notches` wheel steps at `(col, row)` (≥1). Needs any tracking.
    Wheel {
        dir: Scroll,
        col: u16,
        row: u16,
        notches: u32,
    },
    /// Focus in (`gained`) or out. Needs focus reporting (`?1004`).
    Focus { gained: bool },
    /// Named key ([`Session::send_key`] vocabulary). No reporting mode.
    Key { name: &'static str },
}

impl Input {
    /// Hover to `(col, row)`.
    #[must_use]
    pub fn move_to(col: u16, row: u16) -> Self {
        Self::Move { col, row }
    }

    /// Press `button` at `(col, row)`.
    #[must_use]
    pub fn down(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Down { button, col, row }
    }

    /// Release `button` at `(col, row)`.
    #[must_use]
    pub fn up(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Up { button, col, row }
    }

    /// Drag from `from` to `to`, primary button.
    #[must_use]
    pub fn drag(from: (u16, u16), to: (u16, u16)) -> Self {
        Self::Drag { from, to }
    }

    /// One wheel notch at `(col, row)`; chain [`.notches(n)`](Self::notches).
    #[must_use]
    pub fn wheel(dir: Scroll, col: u16, row: u16) -> Self {
        Self::Wheel {
            dir,
            col,
            row,
            notches: 1,
        }
    }

    /// Focus in (`CSI I`).
    #[must_use]
    pub fn focus_in() -> Self {
        Self::Focus { gained: true }
    }

    /// Focus out (`CSI O`).
    #[must_use]
    pub fn focus_out() -> Self {
        Self::Focus { gained: false }
    }

    /// Named key press.
    #[must_use]
    pub fn key(name: &'static str) -> Self {
        Self::Key { name }
    }

    /// Repeat a wheel input `n` times (panics on 0 or a non-wheel input).
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

    /// Check the reporting mode, then send. Fails the case when the app
    /// refused the input (tracking/focus off, off-grid, bad key, dead PTY).
    pub fn send(&self, s: &mut Session) {
        match *self {
            Self::Move { col, row } => {
                let modes = require_mode(s, "hover", ModeNeed::Motion);
                require_in_grid("hover", modes.size, &[(col, row)]);
                s.type_text(&sgr_report(MOTION_CODE, col, row, true))
                    .unwrap_or_else(|e| panic!("hover to ({col}, {row}) failed: {e:#}"));
            }
            Self::Down { button, col, row } => {
                let modes = require_mode(s, "press", ModeNeed::AnyTracking);
                require_in_grid("press", modes.size, &[(col, row)]);
                s.type_text(&sgr_report(button_code(button), col, row, true))
                    .unwrap_or_else(|e| panic!("press at ({col}, {row}) failed: {e:#}"));
            }
            Self::Up { button, col, row } => {
                let modes = require_mode(s, "release", ModeNeed::Release);
                require_in_grid("release", modes.size, &[(col, row)]);
                s.type_text(&sgr_report(button_code(button), col, row, false))
                    .unwrap_or_else(|e| panic!("release at ({col}, {row}) failed: {e:#}"));
            }
            Self::Drag { from, to } => {
                require_mode(s, "drag", ModeNeed::Release);
                s.drag(from.0, from.1, to.0, to.1)
                    .unwrap_or_else(|e| panic!("drag {from:?}→{to:?} failed: {e:#}"));
            }
            Self::Wheel {
                dir,
                col,
                row,
                notches,
            } => {
                require_mode(s, "wheel", ModeNeed::AnyTracking);
                for n in 0..notches {
                    s.scroll(col, row, dir).unwrap_or_else(|e| {
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
                let bytes = if gained { "\x1b[I" } else { "\x1b[O" };
                s.type_text(bytes)
                    .unwrap_or_else(|e| panic!("{what} failed: {e:#}"));
            }
            Self::Key { name } => {
                s.send_key(name)
                    .unwrap_or_else(|e| panic!("key `{name}` failed: {e:#}"));
            }
        }
    }
}

/// What reporting mode an input needs from the application.
#[derive(Debug, Clone, Copy)]
enum ModeNeed {
    /// Any tracking at all (termlens `click`/`scroll` rule).
    AnyTracking,
    /// Any-motion (`?1003`): only it reports button-less motion.
    Motion,
    /// Press-release or finer (termlens `drag` rule: X10 has no release).
    Release,
}

/// Live reporting state, sampled without waiting.
struct Sampled {
    modes: MouseModes,
    mode: MouseMode,
    focus: bool,
    size: (u16, u16),
}

/// Sample the live reporting state. The always-true predicate returns the
/// current screen at once (termlens evaluates before any wait).
fn sample(s: &mut Session) -> Sampled {
    let mut out = None;
    s.wait_until(|screen| {
        out = Some(Sampled {
            modes: screen.mouse_modes(),
            mode: screen.mouse_mode(),
            focus: screen.focus_events(),
            size: screen.size(),
        });
        true
    })
    .expect("sample live screen");
    out.expect("always-true predicate samples the screen")
}

/// Enabled tracking set (for assertions). See [`sample`].
#[must_use]
pub fn tracking(s: &mut Session) -> MouseModes {
    sample(s).modes
}

/// Collapsed reporting protocol (for assertions). See [`sample`].
#[must_use]
pub fn tracking_mode(s: &mut Session) -> MouseMode {
    sample(s).mode
}

/// Whether focus reporting (`?1004`) is on (for assertions). See [`sample`].
#[must_use]
pub fn focus_reporting(s: &mut Session) -> bool {
    sample(s).focus
}

/// Fail the case unless the app enabled what `what` needs. Returns the
/// sample so the caller can bounds-check against the same observation.
fn require_mode(s: &mut Session, what: &str, need: ModeNeed) -> Sampled {
    let sampled = sample(s);
    let ok = match need {
        ModeNeed::AnyTracking => sampled.mode != MouseMode::None,
        ModeNeed::Motion => sampled.mode == MouseMode::AnyMotion,
        ModeNeed::Release => matches!(
            sampled.mode,
            MouseMode::PressRelease | MouseMode::ButtonMotion | MouseMode::AnyMotion
        ),
    };
    assert!(
        ok,
        "{what} refused: app reports in {:?} (set {:?}), needs {need:?}",
        sampled.mode, sampled.modes,
    );
    sampled
}

/// Fail the case unless focus reporting is on.
fn require_focus(s: &mut Session, gained: bool) {
    let what = if gained { "focus-in" } else { "focus-out" };
    assert!(
        sample(s).focus,
        "{what} refused: app never enabled focus reporting (no CSI ?1004 h seen)"
    );
}

/// Fail the case when any point is outside the sampled grid (mirrors
/// termlens `check_mouse_in_grid`, which raw SGR bypasses).
fn require_in_grid(what: &str, (cols, rows): (u16, u16), pts: &[(u16, u16)]) {
    for &(col, row) in pts {
        assert!(
            col < cols && row < rows,
            "{what} at ({col}, {row}) is outside the {cols}x{rows} grid"
        );
    }
}

/// SGR-1006 report, byte-identical to termlens `mouse_sgr`: press is `M`,
/// release keeps the button code with `m` (only the legacy form maps the
/// release to 3). 1-based coordinates.
fn sgr_report(code: u8, col: u16, row: u16, press: bool) -> String {
    let suffix = if press { 'M' } else { 'm' };
    format!(
        "\x1b[<{code};{};{}{suffix}",
        u32::from(col) + 1,
        u32::from(row) + 1
    )
}

/// xterm button code: Left 0, Middle 1, Right 2.
fn button_code(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        _ => panic!("unsupported mouse button: {button:?}"),
    }
}

#[test]
fn sgr_vectors_match_termlens() {
    assert_eq!(sgr_report(0, 9, 6, true), "\x1b[<0;10;7M");
    assert_eq!(sgr_report(0, 9, 6, false), "\x1b[<0;10;7m");
    assert_eq!(sgr_report(64, 0, 0, true), "\x1b[<64;1;1M");
    assert_eq!(sgr_report(MOTION_CODE, 4, 2, true), "\x1b[<35;5;3M");
    assert_eq!(button_code(MouseButton::Left), 0);
    assert_eq!(button_code(MouseButton::Middle), 1);
    assert_eq!(button_code(MouseButton::Right), 2);
}

/// Live smoke: one session through all three helpers (PTY, ignored like
/// every capture test). Exercises typed move/down/up/drag/wheel/focus/key,
/// unique-target resolve + refresh after scroll/input, and state waits.
#[test]
#[ignore = "typed-helpers smoke; run with --ignored"]
fn smoke_typed_helpers() {
    use std::time::Duration;

    use super::scoped_targets::Target;
    use super::state_waits as waits;
    use crate::support::{spawn_boot, Case, Color, SHOWCASE};

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

    // Reporting-mode checks: full mouse capture on, focus sampled.
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
    // Focus refusal is fail-closed: panics when 1004 is off, sends when on.
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

    // Unique target + typed hover (replaces find + raw SGR move).
    let mut target = Target::resolve(&mut s, "Preview");
    eprintln!("smoke: Preview at {:?}", target.point());
    assert_eq!(target.point(), (target.col(), target.row()));
    assert_eq!(target.grid(), (120, 40));
    let (col, row) = target.point();
    Input::move_to(col, row).send(&mut s);

    // Typed wheel + re-resolve after scroll.
    Input::wheel(Scroll::Down, col, row).notches(2).send(&mut s);
    target.refresh(&mut s);

    // Typed key + target stability across focus motion.
    Input::key("down").send(&mut s);
    target.refresh_same(&mut s);

    // Resize + re-resolve after geometry change (left-anchored layout
    // keeps the target in place — pinned by the smoke run).
    s.resize(100, 30).expect("resize down");
    waits::wait_size(&mut s, 100, 30, "smoke shrink");
    target.refresh_same(&mut s);
    s.resize(120, 40).expect("resize up");
    waits::wait_size(&mut s, 120, 40, "smoke grow");
    target.refresh(&mut s);

    // wait_gone refuses vacuous absence (never-present needle must panic).
    let vacuous = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        waits::wait_gone(&mut s, "needle-that-never-exists-zzz", "smoke vacuous");
    }));
    assert!(vacuous.is_err(), "wait_gone must fail vacuous absence");

    // Frame-poll primitive: trivial predicate returns at once.
    let frame = waits::wait_frame_where(&mut s, Duration::from_secs(2), "smoke frame", |_| true);
    assert_eq!((frame.cols, frame.rows), (120, 40));

    // Phase wait times out (bounded) on an impossible phase.
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

    // Typed split click + drag over the resolved span.
    Input::down(MouseButton::Left, col, row).send(&mut s);
    Input::up(MouseButton::Left, col, row).send(&mut s);
    waits::wait_state(&mut s, case.needle, "smoke after click");
    let target = Target::resolve(&mut s, "Preview");
    Input::drag(target.point(), target.span_end()).send(&mut s);

    // Quiet settle on the quiet buttons page.
    let frame = waits::wait_quiet(&mut s, Duration::from_millis(400), "smoke settle");
    assert_eq!((frame.cols, frame.rows), (120, 40));
    assert!(frame.text().contains(case.needle));
}
