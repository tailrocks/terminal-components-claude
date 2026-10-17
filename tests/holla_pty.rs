//! Fresh-process proofs of the holla preview on a tuiscotti PTY session:
//! the colour policy matrix (F23f: explicit palette, `NO_COLOR` empty and
//! non-empty, `NO_COLOR` against an explicit flag) and the bounded input
//! flood (F23d: dispatch and quit latency under a queue the terminal keeps
//! feeding). Every wait is bounded and nothing touches the developer's
//! terminal.
//!
//! The PTY, the emulator and the input writer are tuiscotti's
//! (`tuiscotti::tui`, T17): no hand-rolled pseudo-terminal here. Colour classes
//! are read from the emulated cells (`Color::Rgb` for 24-bit SGR,
//! `Color::Indexed` for `38;5;n` and the `3x`/`9x` forms), modifiers from
//! the cell `Mods`, so colour is still asserted separately from
//! reverse/bold/dim/underline and an attribute-only frame is never mistaken
//! for a coloured one.
//!
//! Scope: macOS and Linux.

#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::time::{Duration, Instant};

use tuiscotti::Observation;
use tuiscotti::frame::Color;
use tuiscotti::tui::{CancelToken, Session, Tui};

const BOUND: Duration = Duration::from_secs(15);

fn spawn(args: &[&str], env: &[(&str, Option<&str>)], cols: u16, rows: u16) -> Session {
    let mut tui = Tui::new([env!("CARGO_BIN_EXE_holla")])
        .args(args)
        .size(cols, rows)
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env_remove("NO_COLOR")
        .env_remove("HOLLA_NO_MOTION")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR");
    for (k, v) in env {
        tui = match v {
            Some(v) => tui.env(k, v),
            None => tui.env_remove(k),
        };
    }
    tui.spawn().expect("spawn holla on a tuiscotti pty")
}

fn screen_text(obs: &Observation) -> String {
    tuiscotti::render::frame_from_screen(&obs.screen, "holla_pty").text()
}

fn wait_for(session: &Session, what: &str, cond: impl Fn(&Observation) -> bool) -> Observation {
    session
        .wait_predicate(cond, Instant::now() + BOUND, &CancelToken::new())
        .unwrap_or_else(|e| panic!("timed out waiting for {what}: {e}"))
}

/// Wait for the child to exit and return its exit code.
fn wait_exit(session: &Session) -> u32 {
    let exit = session
        .wait_exit(Instant::now() + BOUND, &CancelToken::new())
        .unwrap_or_else(|e| panic!("holla did not exit: {e}"));
    assert!(
        exit.status.signal().is_none(),
        "holla was killed: {}",
        exit.status
    );
    exit.status.code()
}

/// Colour and attribute classes found on a screen, counted per cell and
/// kept apart: colour never hides behind an attribute and an attribute
/// never counts as colour. The emulator keeps crossterm's `38;5;n` with n
/// below 16 and the `3x`/`9x` forms alike as a palette index below 16, so
/// both count as the basic palette, never as the 256-colour cube.
#[derive(Debug, Default, PartialEq, Eq)]
struct Sgr {
    truecolor: usize,
    /// Palette index >= 16: the 256-colour cube and greys.
    indexed: usize,
    /// Palette index < 16.
    basic: usize,
    reverse: usize,
    bold: usize,
    dim: usize,
    underline: usize,
    /// Distinct palette indexes below 16 that were used.
    palette: Vec<u8>,
}

impl Sgr {
    fn colour(&mut self, c: Color) {
        match c {
            Color::Default => {}
            Color::Rgb(_) => self.truecolor += 1,
            Color::Indexed(n) if n < 16 => {
                self.basic += 1;
                if !self.palette.contains(&n) {
                    self.palette.push(n);
                }
            }
            Color::Indexed(_) => self.indexed += 1,
        }
    }

    fn add(&mut self, obs: &Observation) {
        for cell in obs.screen.cells() {
            self.colour(cell.fg);
            self.colour(cell.bg);
            self.colour(cell.underline_color);
            let m = cell.mods;
            self.reverse += usize::from(m.reverse);
            self.bold += usize::from(m.bold);
            self.dim += usize::from(m.dim);
            self.underline += usize::from(m.underline);
        }
    }
}

/// Every frame the run drew is classified: the first frame, the frame after
/// Tab moved focus to the preview, and the frame after Shift+Tab moved it
/// back.
fn frame(args: &[&str], env: &[(&str, Option<&str>)]) -> (Sgr, String) {
    let mut base = vec![
        "--scenario",
        "rust-dirty",
        "--motion",
        "paused",
        "--frame",
        "40",
    ];
    base.extend_from_slice(args);
    let session = spawn(&base, env, 100, 30);
    let mut sgr = Sgr::default();
    let first = wait_for(&session, "the first frame", |o| {
        let t = screen_text(o);
        t.contains("holla") && t.contains("Suggested")
    });
    sgr.add(&first);
    let settle = |s: &Session| {
        s.wait_stable_quiet(
            Instant::now() + BOUND,
            Duration::from_millis(150),
            &CancelToken::new(),
        )
        .unwrap_or_else(|e| panic!("the frame never settled: {e}"))
    };
    // a stable second frame: Tab moves to the preview and back
    session.send_bytes(b"\t").expect("Tab");
    sgr.add(&settle(&session));
    session.send_bytes(b"\x1b[Z").expect("Shift+Tab");
    let last = settle(&session);
    sgr.add(&last);
    // Ctrl+Q on a clean shell quits (Esc never quits, keyboard audit #21)
    session.send_bytes(b"\x11").expect("Ctrl+Q");
    let text = screen_text(&last);
    let code = wait_exit(&session);
    assert_eq!(code, 0, "clean exit: {text}");
    (sgr, text)
}

#[test]
fn explicit_palettes_emit_exactly_their_colour_class_and_keep_attributes() {
    let (tc, _) = frame(&["--color", "truecolor"], &[]);
    assert!(
        tc.truecolor > 0,
        "truecolor frames carry 24-bit SGR: {tc:?}"
    );
    assert_eq!(
        tc.indexed + tc.basic,
        0,
        "no palette colour in a truecolor frame: {tc:?}"
    );
    assert!(
        tc.reverse > 0 || tc.bold > 0,
        "attributes are present: {tc:?}"
    );

    let (c256, _) = frame(&["--color", "256"], &[]);
    assert!(
        c256.indexed > 0,
        "256-colour frames carry indexed SGR: {c256:?}"
    );
    assert_eq!(c256.truecolor, 0, "no 24-bit SGR at 256 colours: {c256:?}");

    let (c16, _) = frame(&["--color", "16"], &[]);
    assert!(c16.basic > 0, "16-colour frames carry basic SGR: {c16:?}");
    assert_eq!(
        c16.truecolor + c16.indexed,
        0,
        "no extended colour at 16: {c16:?}"
    );

    // Mono is a palette of four greys, not the absence of SGR: that is
    // what separates it from NO_COLOR below
    let (mono, _) = frame(&["--color", "none"], &[]);
    assert_eq!(
        mono.truecolor + mono.indexed,
        0,
        "mono frames carry no hue: {mono:?}"
    );
    let mut greys = mono.palette.clone();
    greys.sort_unstable();
    assert!(
        greys.iter().all(|n| [0, 7, 8, 15].contains(n)),
        "mono uses only black, grey, dark grey and white: {greys:?}"
    );
    assert!(
        mono.bold > 0,
        "mono keeps bold so hierarchy stays visible: {mono:?}"
    );
}

#[test]
fn no_color_policy_is_the_backend_rule_in_a_fresh_process() {
    // NO_COLOR non-empty: colour is suppressed even against an explicit flag,
    // attributes survive the suppression (the runtime resets colours, never
    // modifiers)
    let (off, _) = frame(&["--color", "truecolor"], &[("NO_COLOR", Some("1"))]);
    assert_eq!(
        off.truecolor + off.indexed + off.basic,
        0,
        "NO_COLOR=1 removes every colour SGR: {off:?}"
    );
    assert!(
        off.reverse > 0 || off.bold > 0,
        "attributes survive NO_COLOR: {off:?}"
    );
    // NO_COLOR empty is not set: colour stays
    let (empty, _) = frame(&["--color", "truecolor"], &[("NO_COLOR", Some(""))]);
    assert!(
        empty.truecolor > 0,
        "an empty NO_COLOR does not disable colour: {empty:?}"
    );
    // a force flag from the environment is not a crossterm policy: it does
    // not override NO_COLOR, and the preview never pretends it does
    let (forced, _) = frame(
        &["--color", "truecolor"],
        &[
            ("NO_COLOR", Some("1")),
            ("CLICOLOR_FORCE", Some("1")),
            ("FORCE_COLOR", Some("1")),
        ],
    );
    assert_eq!(
        forced.truecolor + forced.indexed + forced.basic,
        0,
        "NO_COLOR wins over force variables: {forced:?}"
    );
}

#[test]
fn a_bounded_input_flood_is_drained_fairly_and_a_quit_behind_it_is_honoured() {
    let session = spawn(
        &["--scenario", "first-use", "--motion", "reduced"],
        &[],
        100,
        30,
    );
    let first = wait_for(&session, "the first frame", |o| {
        screen_text(o).contains("holla")
    });
    let before = first.revision;
    // 8000 Ctrl+B presses: a single byte each (no escape-sequence boundary
    // ambiguity across pty reads) bound to nothing on the root, so every one
    // is an ignored event; the queue keeps feeding while the loop drains.
    // The flood is far longer than one terminal read (crossterm reads 1 KiB
    // per readiness), so the runtime must keep reading what the terminal
    // still holds without new input arriving: crossterm 0.29 alone leaves
    // it in the kernel on Linux and the quit behind it is never seen.
    // tuiscotti's writer thread feeds the whole flood through the pty in
    // one acknowledged write, blocking while the kernel buffer is full, so
    // the application sees the same back-pressured burst.
    let flood: Vec<u8> = vec![0x02; 8000];
    let start = Instant::now();
    session.send_bytes(&flood).unwrap_or_else(|e| {
        panic!(
            "the flood write failed: {e}; child status {:?}",
            session.poll_exit()
        )
    });
    // discovery ticks keep landing while the flood drains: the frame changes
    wait_for(&session, "a redraw during the flood", |o| {
        o.revision > before
    });
    // a quit queued behind the flood is honoured once the flood is consumed
    // (Ctrl+Q: Esc never quits, keyboard audit #21). Its latency is timed on
    // its own: the redraw wait above follows the app's tick cadence, not
    // the drain.
    let quit = Instant::now();
    session
        .send_bytes(b"\x11")
        .expect("Ctrl+Q behind the flood");
    let code = wait_exit(&session);
    let quit_latency = quit.elapsed();
    let elapsed = start.elapsed();
    assert_eq!(code, 0, "clean exit after the flood");
    assert!(
        quit_latency < Duration::from_secs(3),
        "the quit behind 8000 ignored events took {quit_latency:?}"
    );
    eprintln!(
        "F23d pty flood: 8000 ignored events + quit in {elapsed:?} (quit latency {quit_latency:?})"
    );
}
