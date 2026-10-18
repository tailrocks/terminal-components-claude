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

#[derive(Debug, Default, PartialEq, Eq)]
struct Sgr {
    truecolor: usize,

    indexed: usize,

    basic: usize,
    reverse: usize,
    bold: usize,
    dim: usize,
    underline: usize,

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

    session.send_bytes(b"\t").expect("Tab");
    sgr.add(&settle(&session));
    session.send_bytes(b"\x1b[Z").expect("Shift+Tab");
    let last = settle(&session);
    sgr.add(&last);

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

    let (empty, _) = frame(&["--color", "truecolor"], &[("NO_COLOR", Some(""))]);
    assert!(
        empty.truecolor > 0,
        "an empty NO_COLOR does not disable colour: {empty:?}"
    );

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

    let flood: Vec<u8> = vec![0x02; 8000];
    let start = Instant::now();
    session.send_bytes(&flood).unwrap_or_else(|e| {
        panic!(
            "the flood write failed: {e}; child status {:?}",
            session.poll_exit()
        )
    });

    wait_for(&session, "a redraw during the flood", |o| {
        o.revision > before
    });

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
