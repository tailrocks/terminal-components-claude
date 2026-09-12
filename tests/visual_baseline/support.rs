//! Shared driver for the visual-baseline suite.
//!
//! One PTY per capture: spawn → boot idle(200 ms) → boot-needle wait → send
//! steps (120 ms pacing) → `wait_stable`(SETTLE) → frame → store gate —
//! exactly the flow `tuisnap run` executed for the retired bash runner (its
//! [`run_once`] is reused verbatim for the ported matrix; [`boot`]/[`drive`]
//! mirror it for the pointer group, which needs the live session afterwards).
//!
//! Colour hygiene per capture: ambient `NO_COLOR` is stripped (crossterm
//! honours it by *presence* and would silently flatten every colour frame —
//! the lesson that forced the 2026-09-12 baseline re-capture) and
//! `HOLLA_NO_HISTORY=1` suppresses history side effects; `nocolor` captures
//! re-add `NO_COLOR=1` instead of a `--color` flag (backend rule, not app
//! rule).

use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;

use tuisnap::pty::{PtyOptions, Session, run_once};
use tuisnap::snapshot::{CompareOutcome, Status, Store};
use tuisnap::{Frame, Profile, Renderer, VENDORED_FACES};

pub const SHOWCASE: &str = env!("CARGO_BIN_EXE_showcase");
pub const TABLEPRO: &str = env!("CARGO_BIN_EXE_tablepro");
pub const JACKIN: &str = env!("CARGO_BIN_EXE_jackin-preview");
pub const HOLLA: &str = env!("CARGO_BIN_EXE_holla");

/// SETTLE_MS from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// TIMEOUT_MS default; boot-streaming screens override it per capture
/// ([`Case::timeout`], the bash `CAP_TIMEOUT=` prefix).
pub const TIMEOUT_MS: u64 = 8_000;

/// `--color` palette, or real `NO_COLOR=1` (the bash runner's `nocolor`).
#[derive(Clone, Copy)]
pub enum Color {
    Truecolor,
    Ansi256,
    Ansi16,
    None,
    NoColorEnv,
}

/// One capture definition — the typed form of one bash `cap` call.
pub struct Case {
    /// Store name, identical to the bash capture name
    /// (`<app>_<surface>_<state>_<cols>x<rows>_<color>`).
    pub name: &'static str,
    pub bin: &'static str,
    /// argv after the binary, before `--color` (e.g. `&["--page", "diff"]`).
    pub args: &'static [&'static str],
    pub cols: u16,
    pub rows: u16,
    pub color: Color,
    /// Boot needle on the fully-rendered first screen (`""` skips the wait);
    /// sent as the first step, before the sends, exactly as the bash runner
    /// did (the CLI's `--wait-for` would have run after them).
    pub needle: &'static str,
    /// DSL steps after the boot wait: key names, `type:<text>`,
    /// `sleep:<ms>`, `wait:<needle>`.
    pub sends: &'static [&'static str],
    pub timeout_ms: u64,
}

impl Case {
    pub const fn new(
        name: &'static str,
        bin: &'static str,
        args: &'static [&'static str],
        cols: u16,
        rows: u16,
        color: Color,
        needle: &'static str,
    ) -> Self {
        Self {
            name,
            bin,
            args,
            cols,
            rows,
            color,
            needle,
            sends: &[],
            timeout_ms: TIMEOUT_MS,
        }
    }

    pub const fn sends(self, sends: &'static [&'static str]) -> Self {
        Self { sends, ..self }
    }

    /// CAP_TIMEOUT override for screens whose boot stream outlasts
    /// TIMEOUT_MS (scrolling, terminal): the boot wait_idle shares it.
    pub const fn timeout(self, ms: u64) -> Self {
        Self {
            timeout_ms: ms,
            ..self
        }
    }
}

pub fn argv_for(case: &Case) -> Vec<String> {
    let mut argv = Vec::with_capacity(case.args.len() + 3);
    argv.push(case.bin.to_string());
    argv.extend(case.args.iter().map(|s| s.to_string()));
    let flag = match case.color {
        Color::Truecolor => Some("truecolor"),
        Color::Ansi256 => Some("256"),
        Color::Ansi16 => Some("16"),
        Color::None => Some("none"),
        Color::NoColorEnv => Option::None,
    };
    if let Some(flag) = flag {
        argv.push("--color".to_string());
        argv.push(flag.to_string());
    }
    argv
}

pub fn opts_for(case: &Case) -> PtyOptions {
    let opts = PtyOptions {
        cols: case.cols,
        rows: case.rows,
        timeout: Duration::from_millis(case.timeout_ms),
        ..PtyOptions::default()
    }
    .without_env("NO_COLOR")
    .with_env("HOLLA_NO_HISTORY", "1");
    match case.color {
        Color::NoColorEnv => opts.with_env("NO_COLOR", "1"),
        _ => opts,
    }
}

/// Boot needle first (a `wait:` step), then the sends — the bash runner's
/// step order.
fn steps_for(case: &Case) -> Vec<String> {
    let mut steps = Vec::with_capacity(case.sends.len() + 1);
    if !case.needle.is_empty() {
        steps.push(format!("wait:{}", case.needle));
    }
    steps.extend(case.sends.iter().map(|s| s.to_string()));
    steps
}

thread_local! {
    /// One renderer per test thread: font faces parsed once, glyph rasters
    /// cached across every check on the thread (no locks).
    static RENDERER: RefCell<Renderer> = RefCell::new(
        Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse"),
    );
}

pub fn store() -> Store {
    Store::new(Path::new("shots/tuisnap"))
}

/// Cell-exact + pixel-exact gate at threshold 1.0 through the thread's
/// cached renderer. Writes `actual/` artifacts even when unmatched.
pub fn gate(name: &str, frame: &Frame) -> CompareOutcome {
    RENDERER
        .with(|r| store().check_with(&mut r.borrow_mut(), name, frame, 1.0))
        .unwrap_or_else(|e| panic!("gate `{name}` failed: {e}"))
}

/// Fail-closed assertion: `matched` passes; `missing-approval` passes but is
/// logged as pending (expected for new names on a first run — explicit
/// `tuisnap accept` is the only bless, never the test); drift after
/// approval, dimension mismatch and corrupt approval fail.
pub fn assert_gated(outcome: &CompareOutcome) {
    match outcome.status {
        Status::Matched => eprintln!("baseline matched   {}", outcome.name),
        Status::MissingApproval => eprintln!("baseline PENDING   {}", outcome.name),
        _ => panic!("{}", outcome.ensure_matched().unwrap_err()),
    }
}

/// A ported-matrix capture: the same runner the `tuisnap run` CLI used.
pub fn run_and_assert(case: &Case) {
    let frame = run_once(&argv_for(case), &opts_for(case), &steps_for(case), SETTLE)
        .unwrap_or_else(|e| panic!("capture `{}` failed: {e:#}", case.name));
    assert_gated(&gate(case.name, &frame));
}

/// Spawn a case's session for the pointer group (mouse/resize captures need
/// the live session after boot).
pub fn spawn(case: &Case) -> Session {
    Session::spawn(&argv_for(case), &opts_for(case))
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name))
}

/// `run_once`'s boot wait: 200 ms of output silence before any step.
pub fn boot(session: &mut Session, needle: &str) {
    session
        .wait_idle(Duration::from_millis(200))
        .unwrap_or_else(|e| panic!("boot idle failed: {e:#}"));
    if !needle.is_empty() {
        session
            .wait_for_text(needle)
            .unwrap_or_else(|e| panic!("boot needle `{needle}` never appeared: {e:#}"));
    }
}

/// `run_once`'s step loop on a live session, pacing included.
pub fn drive(session: &mut Session, steps: &[&str]) {
    for step in steps {
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
            session
                .wait_for_text(needle)
                .unwrap_or_else(|e| panic!("`wait:{needle}` timed out: {e:#}"));
        } else if let Some(text) = step.strip_prefix("type:") {
            session.type_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            session.send_key(step).expect("send_key");
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

/// Settle the screen and gate the capture.
pub fn settle_and_gate(session: &mut Session, name: &str) {
    let frame = session
        .wait_stable(SETTLE)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e:#}"));
    assert_gated(&gate(name, &frame));
}

/// One `#[test]` per capture, generated from the static case tables so cargo
/// name filters work (`cargo test --test visual_baseline holla_ -- --ignored`).
/// The test fn name is the capture name with `-` mapped to `_`.
#[macro_export]
macro_rules! baseline_case {
    ($fn_name:ident => $case:expr) => {
        #[test]
        #[ignore = "visual baseline capture; run with --ignored"]
        fn $fn_name() {
            $crate::support::run_and_assert(&$case);
        }
    };
}
