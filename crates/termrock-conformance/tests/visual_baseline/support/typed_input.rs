use tuiscotti::tui::{MouseButton, MouseMods, Wheel};

use super::Session;

pub type MouseModes = Vec<u16>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    None,

    Press,

    PressRelease,

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

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Move {
        col: u16,
        row: u16,
    },

    Down {
        button: MouseButton,
        col: u16,
        row: u16,
    },

    Up {
        button: MouseButton,
        col: u16,
        row: u16,
    },

    Drag {
        from: (u16, u16),
        to: (u16, u16),
    },

    Wheel {
        dir: Wheel,
        col: u16,
        row: u16,
        notches: u32,
    },

    Focus {
        gained: bool,
    },

    Key {
        name: &'static str,
    },
}

impl Input {
    #[must_use]
    pub fn move_to(col: u16, row: u16) -> Self {
        Self::Move { col, row }
    }

    #[must_use]
    pub fn down(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Down { button, col, row }
    }

    #[must_use]
    pub fn up(button: MouseButton, col: u16, row: u16) -> Self {
        Self::Up { button, col, row }
    }

    #[must_use]
    pub fn drag(from: (u16, u16), to: (u16, u16)) -> Self {
        Self::Drag { from, to }
    }

    #[must_use]
    pub fn wheel(dir: Wheel, col: u16, row: u16) -> Self {
        Self::Wheel {
            dir,
            col,
            row,
            notches: 1,
        }
    }

    #[must_use]
    pub fn focus_in() -> Self {
        Self::Focus { gained: true }
    }

    #[must_use]
    pub fn focus_out() -> Self {
        Self::Focus { gained: false }
    }

    #[must_use]
    pub fn key(name: &'static str) -> Self {
        Self::Key { name }
    }

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

#[derive(Debug, Clone, Copy)]
enum ModeNeed {
    AnyTracking,

    Motion,

    Release,
}

struct Sampled {
    modes: MouseModes,
    mode: MouseMode,
    focus: bool,
    size: (u16, u16),
}

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

#[must_use]
pub fn tracking(s: &mut Session) -> MouseModes {
    sample(s).modes
}

#[must_use]
pub fn tracking_mode(s: &mut Session) -> MouseMode {
    sample(s).mode
}

#[must_use]
pub fn focus_reporting(s: &mut Session) -> bool {
    sample(s).focus
}

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

fn require_focus(s: &mut Session, gained: bool) {
    let what = if gained { "focus-in" } else { "focus-out" };
    assert!(
        sample(s).focus,
        "{what} refused: app never enabled focus reporting (no CSI ?1004 h seen)"
    );
}

fn require_in_grid(what: &str, (cols, rows): (u16, u16), pts: &[(u16, u16)]) {
    for &(col, row) in pts {
        assert!(
            col < cols && row < rows,
            "{what} at ({col}, {row}) is outside the {cols}x{rows} grid"
        );
    }
}

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
