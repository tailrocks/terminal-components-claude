//! TASK-009 regression witnesses: terminal session time, input and
//! capability contracts (runtime-session-time + event-response).
//!
//! Headless boundary cases run here; the two `#[ignore]`d fixtures at the
//! bottom are entry points for the external PTY driver (policy launch matrix
//! and suspend/resume), which supplies the terminal, signals and termios
//! comparison.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    reason = "deterministic boundary and contract assertions"
)]

use core::time::Duration;
use junie_tui::{
    App, Axis, ClockError, ColorLevel, Cx, FeedbackClock, FeedbackClockError, Id, Input, Key,
    KeyCode, KeyModifiers, Moment, Mouse, MouseKind, Part, PartRef, Position, Response, Runtime,
    SimulationMoment, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
use std::collections::VecDeque;

const OWNER: Id = Id::root("completion009.owner");

struct Boundary {
    repaint_at_bootstrap: Option<Duration>,
    tick_repaint: Option<Duration>,
    flash_on_bootstrap: bool,
    sync_schedule: VecDeque<SimulationMoment>,
    sync_results: Vec<Result<(), FeedbackClockError>>,
    causes: Vec<UpdateCause>,
}

impl Boundary {
    fn plain() -> Self {
        Self {
            repaint_at_bootstrap: None,
            tick_repaint: None,
            flash_on_bootstrap: false,
            sync_schedule: VecDeque::new(),
            sync_results: Vec::new(),
            causes: Vec::new(),
        }
    }
}

impl App for Boundary {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.causes.push(cx.update_cause());
        if cx.update_cause() == UpdateCause::Bootstrap {
            if let Some(delay) = self.repaint_at_bootstrap {
                cx.request_repaint_after(delay);
            }
            if self.flash_on_bootstrap {
                cx.flash_activation(OWNER);
            }
        } else if cx.update_cause() == UpdateCause::Tick {
            if let Some(delay) = self.tick_repaint {
                cx.request_repaint_after(delay);
            }
            if let Some(target) = self.sync_schedule.pop_front() {
                self.sync_results.push(cx.sync_feedback_time(target));
            }
        }
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

fn runtime(app: Boundary) -> Runtime<Boundary> {
    Runtime::new(app, Theme::junie())
}

// ── Bootstrap: exactly one, clock-free, before input/draw ──

#[test]
fn bootstrap_runs_exactly_once_without_reading_a_clock() {
    let mut rt = runtime(Boundary::plain());
    assert_eq!(rt.now(), Moment::ZERO);
    let _ = rt.initialize();
    assert_eq!(rt.now(), Moment::ZERO);
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
    let _ = rt.initialize();
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
}

#[test]
fn input_cannot_precede_bootstrap_and_presentation() {
    let mut rt = runtime(Boundary::plain());
    let _ = rt.initialize();
    // No frame was ever presented, so input waits for bootstrap/draw.
    let pending = rt.handle(Input::Tick).expect_err("must pend input");
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
    let _ = pending.into_input();
}

// ── Monotonic time: equal idempotent, backward atomically rejected ──

#[test]
fn equal_advance_is_idempotent_and_backward_advance_rejects_atomically() {
    let mut app = Boundary::plain();
    app.repaint_at_bootstrap = Some(Duration::from_millis(2200));
    let mut rt = runtime(app);
    let _ = rt.initialize();
    assert!(rt.advance_to(Moment::from_millis(100)).is_ok());
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(2200)));
    assert!(rt.advance_to(Moment::from_millis(100)).is_ok());
    assert_eq!(rt.now(), Moment::from_millis(100));
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(2200)));
    let error = rt
        .advance_to(Moment::from_millis(99))
        .expect_err("backward time must reject");
    assert_eq!(
        error,
        ClockError {
            current: Moment::from_millis(100),
            requested: Moment::from_millis(99),
        }
    );
    assert_eq!(rt.now(), Moment::from_millis(100));
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(2200)));
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
}

// ── Ticks: one delivery, one Tick; settle follows; no idle ticks ──

#[test]
fn timer_delivery_issues_tick_once_through_settling() {
    let mut app = Boundary::plain();
    app.repaint_at_bootstrap = Some(Duration::from_millis(2200));
    let mut rt = runtime(app);
    let _ = rt.initialize();
    // 2199: not due; settling is idle.
    assert!(rt.advance_to(Moment::from_millis(2199)).is_ok());
    assert!(!rt.needs_settle());
    assert_eq!(rt.settle(), Response::ignored());
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
    // 2200: due exactly once; a second settle without new delivery is idle.
    assert!(rt.advance_to(Moment::from_millis(2200)).is_ok());
    assert!(rt.needs_settle());
    let _ = rt.settle();
    assert_eq!(
        rt.app().causes,
        vec![UpdateCause::Bootstrap, UpdateCause::Tick]
    );
    assert!(!rt.needs_settle());
    assert_eq!(rt.settle(), Response::ignored());
    assert_eq!(
        rt.app().causes,
        vec![UpdateCause::Bootstrap, UpdateCause::Tick]
    );
    // A programmatic focus transition settles after the tick.
    rt.set_focus(Some(OWNER));
    assert!(rt.needs_settle());
    let _ = rt.settle();
    assert_eq!(
        rt.app().causes,
        vec![
            UpdateCause::Bootstrap,
            UpdateCause::Tick,
            UpdateCause::Settle
        ]
    );
}

#[test]
fn idle_time_advance_stages_no_unsolicited_ticks() {
    let mut rt = runtime(Boundary::plain());
    let _ = rt.initialize();
    assert_eq!(rt.next_deadline(), None);
    assert!(rt.advance_to(Moment::from_millis(60_000)).is_ok());
    assert!(!rt.needs_settle());
    assert_eq!(rt.settle(), Response::ignored());
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
}

// ── Flood fairness: input never postpones a deadline or moves time ──

#[test]
fn flooded_input_neither_postpones_the_earliest_deadline_nor_advances_time() {
    let mut app = Boundary::plain();
    app.repaint_at_bootstrap = Some(Duration::from_millis(101));
    let mut harness = Harness::new(app, Theme::junie(), 40, 10).with_auto_draw(false);
    assert_eq!(harness.next_deadline(), Some(Moment::from_millis(101)));
    let storm = [
        Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }),
        Input::Mouse(Mouse {
            kind: MouseKind::Wheel(Axis::H, 1),
            pos: Position::new(1, 1),
            mods: KeyModifiers::SHIFT,
        }),
        Input::Paste("flood".to_owned()),
        Input::Resize(40, 10),
        Input::Tick,
    ];
    for input in storm.iter().cycle().take(1000) {
        let _ = harness.handle(input.clone());
    }
    assert_eq!(harness.now(), Moment::ZERO);
    assert_eq!(harness.next_deadline(), Some(Moment::from_millis(101)));
    harness.draw();
    assert_eq!(harness.next_deadline(), Some(Moment::from_millis(101)));
}

// ── Elapsed feedback: 139 active, 140 expired, redraw without a tick ──

#[test]
fn elapsed_feedback_is_active_at_139_and_expired_at_140() {
    let mut app = Boundary::plain();
    app.flash_on_bootstrap = true;
    let mut harness =
        Harness::new_with_feedback_clock(app, Theme::junie(), 40, 10, FeedbackClock::Elapsed)
            .with_auto_draw(false);
    let active = harness.activation_feedback().expect("flash must hold");
    assert_eq!(active.owner, OWNER);
    assert_eq!(active.part, PartRef::of(Part::CONTAINER));
    assert_eq!(active.remaining, Duration::from_millis(140));
    assert_eq!(harness.next_deadline(), Some(Moment::from_millis(140)));
    let _ = harness.advance(Duration::from_millis(139));
    let active = harness.activation_feedback().expect("139 ms must hold");
    assert_eq!(active.remaining, Duration::from_millis(1));
    let _ = harness.advance(Duration::from_millis(1));
    assert!(harness.activation_feedback().is_none());
    assert_eq!(harness.next_deadline(), None);
}

#[test]
fn elapsed_feedback_expiry_requests_redraw_without_staging_a_tick() {
    let mut app = Boundary::plain();
    app.flash_on_bootstrap = true;
    let mut rt = Runtime::new_with_feedback_clock(app, Theme::junie(), FeedbackClock::Elapsed);
    let _ = rt.initialize();
    assert!(rt.advance_to(Moment::from_millis(140)).is_ok());
    assert!(rt.needs_present());
    assert!(!rt.needs_settle());
    assert_eq!(rt.settle(), Response::ignored());
    assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
}

// ── Simulation feedback: domain time only, atomic policy/clock rejects ──

#[test]
fn simulation_feedback_ages_only_from_admitted_domain_steps() {
    let mut app = Boundary::plain();
    app.flash_on_bootstrap = true;
    app.repaint_at_bootstrap = Some(Duration::from_millis(10));
    app.tick_repaint = Some(Duration::from_millis(10));
    app.sync_schedule = VecDeque::from([
        SimulationMoment::from_millis(2200),
        SimulationMoment::from_millis(2339),
    ]);
    let clock = FeedbackClock::Simulation {
        initial: SimulationMoment::from_millis(2199),
    };
    let mut rt = Runtime::new_with_feedback_clock(app, Theme::junie(), clock);
    let _ = rt.initialize();
    // Simulation expiry never enters the wall deadline; only the repaint does.
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(10)));
    // Wall time racing 10 s past expiry cannot age simulation feedback.
    assert!(rt.advance_to(Moment::from_millis(10_000)).is_ok());
    let active = rt.activation_feedback().expect("wall time must not age");
    assert_eq!(active.remaining, Duration::from_millis(140));
    // The admitted domain step to 2200 ages it by exactly 1 ms.
    assert!(rt.needs_settle());
    let _ = rt.settle();
    assert_eq!(rt.app().sync_results, vec![Ok(())]);
    let active = rt.activation_feedback().expect("2200 must hold");
    assert_eq!(active.owner, OWNER);
    assert_eq!(active.remaining, Duration::from_millis(139));
    // The coalesced step to 2339 expires it; no wall deadline remains.
    rt.app_mut().tick_repaint = None;
    assert!(rt.advance_to(Moment::from_millis(10_010)).is_ok());
    let _ = rt.settle();
    assert_eq!(rt.app().sync_results, vec![Ok(()), Ok(())]);
    assert!(rt.activation_feedback().is_none());
    assert_eq!(rt.next_deadline(), None);
    assert_eq!(
        rt.app().causes,
        vec![UpdateCause::Bootstrap, UpdateCause::Tick, UpdateCause::Tick]
    );
}

#[test]
fn simulation_feedback_rejects_backward_time_atomically() {
    let mut app = Boundary::plain();
    app.flash_on_bootstrap = true;
    app.repaint_at_bootstrap = Some(Duration::from_millis(10));
    app.tick_repaint = Some(Duration::from_millis(10));
    app.sync_schedule = VecDeque::from([
        SimulationMoment::from_millis(2200),
        SimulationMoment::from_millis(2000),
        SimulationMoment::from_millis(2200),
    ]);
    let clock = FeedbackClock::Simulation {
        initial: SimulationMoment::from_millis(2199),
    };
    let mut rt = Runtime::new_with_feedback_clock(app, Theme::junie(), clock);
    let _ = rt.initialize();
    assert!(rt.advance_to(Moment::from_millis(10)).is_ok());
    let _ = rt.settle();
    assert_eq!(rt.app().sync_results, vec![Ok(())]);
    // The backward step rejects; the record is unchanged.
    assert!(rt.advance_to(Moment::from_millis(20)).is_ok());
    let _ = rt.settle();
    assert_eq!(
        rt.app().sync_results,
        vec![
            Ok(()),
            Err(FeedbackClockError::Backwards {
                current: SimulationMoment::from_millis(2200),
                requested: SimulationMoment::from_millis(2000),
            }),
        ]
    );
    let active = rt.activation_feedback().expect("rejection keeps record");
    assert_eq!(active.remaining, Duration::from_millis(139));
    // The equal step is idempotent.
    assert!(rt.advance_to(Moment::from_millis(30)).is_ok());
    let _ = rt.settle();
    assert_eq!(
        rt.app().sync_results,
        vec![
            Ok(()),
            Err(FeedbackClockError::Backwards {
                current: SimulationMoment::from_millis(2200),
                requested: SimulationMoment::from_millis(2000),
            }),
            Ok(()),
        ]
    );
    let active = rt.activation_feedback().expect("equal keeps record");
    assert_eq!(active.remaining, Duration::from_millis(139));
}

#[test]
fn elapsed_policy_rejects_simulation_sync_without_changing_state() {
    let mut app = Boundary::plain();
    app.flash_on_bootstrap = true;
    app.repaint_at_bootstrap = Some(Duration::from_millis(10));
    app.tick_repaint = Some(Duration::from_millis(10));
    app.sync_schedule = VecDeque::from([SimulationMoment::from_millis(2200)]);
    let mut rt = Runtime::new_with_feedback_clock(app, Theme::junie(), FeedbackClock::Elapsed);
    let _ = rt.initialize();
    assert!(rt.advance_to(Moment::from_millis(10)).is_ok());
    let _ = rt.settle();
    assert_eq!(
        rt.app().sync_results,
        vec![Err(FeedbackClockError::WrongPolicy)]
    );
    let active = rt.activation_feedback().expect("rejection keeps record");
    assert_eq!(active.remaining, Duration::from_millis(130));
    assert_eq!(rt.next_deadline(), Some(Moment::from_millis(20)));
}

// ── Capability: pure decision table, 4x4 matrix, no widening ──

#[test]
fn capability_detection_follows_the_pure_precedence_table() {
    use std::ffi::OsStr;
    let decide = ColorLevel::from_env;
    // Dumb outranks force, NO_COLOR and redirection.
    assert_eq!(
        decide(
            Some(OsStr::new("1")),
            Some(OsStr::new("1")),
            Some("dumb"),
            Some("truecolor"),
            false
        ),
        ColorLevel::Mono
    );
    // Force outranks NO_COLOR and a redirected stdout.
    assert_eq!(
        decide(
            Some(OsStr::new("1")),
            Some(OsStr::new("1")),
            Some("xterm-256color"),
            None,
            false
        ),
        ColorLevel::Ansi256
    );
    // Empty force does not force; NO_COLOR then wins.
    assert_eq!(
        decide(
            Some(OsStr::new("")),
            Some(OsStr::new("1")),
            Some("xterm-256color"),
            None,
            true
        ),
        ColorLevel::Mono
    );
    // Empty NO_COLOR is not set; the pipe still mutes.
    assert_eq!(
        decide(None, Some(OsStr::new("")), Some("xterm"), None, false),
        ColorLevel::Mono
    );
    // Non-UTF8 NO_COLOR is still present.
    #[cfg(unix)]
    assert_eq!(
        decide(
            None,
            Some(std::os::unix::ffi::OsStrExt::from_bytes(b"\xff")),
            Some("xterm"),
            None,
            true
        ),
        ColorLevel::Mono
    );
    // Terminal detection ladder.
    assert_eq!(
        decide(None, None, Some("xterm"), Some("truecolor"), true),
        ColorLevel::TrueColor
    );
    assert_eq!(
        decide(None, None, Some("xterm"), Some("24bit"), true),
        ColorLevel::TrueColor
    );
    for term in ["xterm-256color", "xterm-ghostty", "kitty"] {
        assert_eq!(
            decide(None, None, Some(term), None, true),
            ColorLevel::Ansi256,
            "{term}"
        );
    }
    assert_eq!(
        decide(None, None, Some("xterm"), None, true),
        ColorLevel::Ansi16
    );
    // Absent TERM still paints the base sixteen (Windows sets no TERM).
    assert_eq!(decide(None, None, None, None, true), ColorLevel::Ansi16);
    // Downgrade is idempotent: narrowing twice equals narrowing once.
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        assert_eq!(level.narrow_to(level), level);
    }
}

#[test]
fn requested_four_by_four_matrix_never_widens_authored_capability() {
    let levels = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    for authored_level in levels {
        let authored = Theme::junie().for_level(authored_level);
        assert_eq!(authored.capability.color, authored_level);
        for requested in levels {
            let resolved = authored.for_level(requested);
            // Requested resolves exactly to the meet of authored and requested.
            assert_eq!(
                resolved.capability.color,
                authored_level.narrow_to(requested),
                "authored {authored_level:?}, requested {requested:?}"
            );
            // Never richer than authored: narrowing the result by the authored
            // level is a no-op.
            assert_eq!(
                resolved.capability.color.narrow_to(authored_level),
                resolved.capability.color
            );
        }
    }
    // An already-Mono theme stays Mono under every request, including TrueColor.
    let mono = Theme::junie().for_level(ColorLevel::Mono);
    for requested in levels {
        assert_eq!(
            mono.for_level(requested).capability.color,
            ColorLevel::Mono,
            "overrequest {requested:?} must not widen Mono"
        );
    }
}

#[test]
#[cfg(feature = "crossterm")]
fn detect_matches_for_terminal_and_policy_carries_the_request() {
    use junie_tui::TerminalColorPolicy;
    assert_eq!(
        Theme::junie().for_terminal(),
        Theme::junie().for_level(ColorLevel::detect())
    );
    assert_eq!(
        TerminalColorPolicy::Requested(ColorLevel::Ansi16),
        TerminalColorPolicy::Requested(ColorLevel::Ansi16)
    );
    assert_ne!(
        TerminalColorPolicy::Detect,
        TerminalColorPolicy::Requested(ColorLevel::TrueColor)
    );
    let policy = TerminalColorPolicy::Requested(ColorLevel::Mono);
    assert!(format!("{policy:?}").contains("Mono"));
}

// ── External PTY fixtures (driver-supplied terminal; ignored here) ──

/// Launch-policy matrix entry: the driver runs each policy case under a real
/// PTY with contradictory ambient capability and compares full frames plus
/// termios restoration. Cases: `detect`, `truecolor`, `ansi256`, `ansi16`,
/// `mono`.
#[test]
#[ignore = "real PTY fixture; external driver supplies terminal and environment"]
#[cfg(feature = "crossterm")]
#[expect(clippy::print_stderr, reason = "explicit PTY synchronization markers")]
fn completion_009_launch_policy_fixture() {
    use junie_tui::{TerminalColorPolicy, run_with_feedback_clock_and_color};
    use std::io::{self, Read};

    struct QuitOnBootstrap;
    impl App for QuitOnBootstrap {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.quit();
            }
            Response::ignored()
        }
        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    let policy = match std::env::var("COMPLETION_009_POLICY").unwrap().as_str() {
        "truecolor" => TerminalColorPolicy::Requested(ColorLevel::TrueColor),
        "ansi256" => TerminalColorPolicy::Requested(ColorLevel::Ansi256),
        "ansi16" => TerminalColorPolicy::Requested(ColorLevel::Ansi16),
        "mono" => TerminalColorPolicy::Requested(ColorLevel::Mono),
        _ => TerminalColorPolicy::Detect,
    };
    eprintln!("COMPLETION_009_LAUNCH_READY");
    io::stdin().read_exact(&mut [0]).unwrap();
    run_with_feedback_clock_and_color(
        QuitOnBootstrap,
        Theme::junie(),
        FeedbackClock::Elapsed,
        policy,
    )
    .unwrap();
    eprintln!("COMPLETION_009_LAUNCH_EXITED");
}

/// Suspend/resume entry: the driver signals SIGTSTP twice with `fg` between,
/// compares launch termios while stopped and after quit, and resizes across
/// the stop. The session quits on `q`.
#[test]
#[ignore = "real PTY fixture; external driver signals and resizes"]
#[cfg(all(unix, feature = "crossterm"))]
#[expect(clippy::print_stderr, reason = "explicit PTY synchronization markers")]
fn completion_009_suspend_fixture() {
    use junie_tui::run;

    struct QuitOnQ;
    impl App for QuitOnQ {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            for intent in cx.intents(OWNER) {
                if let junie_tui::Intent::Key(key) = intent
                    && key.code == KeyCode::Char('q')
                {
                    cx.quit();
                }
            }
            Response::ignored()
        }
        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    eprintln!("COMPLETION_009_SUSPEND_READY");
    run(QuitOnQ, Theme::junie()).unwrap();
    eprintln!("COMPLETION_009_SUSPEND_EXITED");
}
