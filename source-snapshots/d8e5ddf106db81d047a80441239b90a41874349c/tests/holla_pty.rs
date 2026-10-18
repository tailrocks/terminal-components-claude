//! Fresh-process proofs of the holla preview on an owned pseudo-terminal:
//! the colour policy matrix (F23f: explicit palette, `NO_COLOR` empty and
//! non-empty, `NO_COLOR` against an explicit flag) and the bounded input
//! flood (F23d: dispatch and quit latency under a queue the terminal keeps
//! feeding). Every wait is bounded and nothing touches the developer's
//! terminal.
//!
//! Scope: macOS and Linux. Colour SGR is asserted separately from
//! reverse/bold/dim/underline so an attribute-only frame is never mistaken
//! for a coloured one.

#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::ffi::{c_int, c_ulong, c_void};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
const TIOCSCTTY: c_ulong = 0x2000_7461;
#[cfg(target_os = "linux")]
const TIOCSCTTY: c_ulong = 0x540E;

#[repr(C)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

#[cfg_attr(target_os = "linux", link(name = "util"))]
unsafe extern "C" {
    fn openpty(
        amaster: *mut c_int,
        aslave: *mut c_int,
        name: *mut u8,
        termp: *const c_void,
        winp: *const c_void,
    ) -> c_int;
    fn setsid() -> c_int;
    fn ioctl(fd: c_int, req: c_ulong, ...) -> c_int;
}

const BOUND: Duration = Duration::from_secs(15);

struct Pty {
    master: File,
    child: std::process::Child,
    output: Arc<Mutex<Vec<u8>>>,
}

impl Pty {
    fn spawn(args: &[&str], env: &[(&str, Option<&str>)], cols: u16, rows: u16) -> Self {
        let mut master: c_int = -1;
        let mut slave: c_int = -1;
        let win = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: valid out-pointers; the winsize outlives the call.
        let rc = unsafe {
            openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                (&win as *const Winsize).cast(),
            )
        };
        assert_eq!(rc, 0, "openpty failed");
        // SAFETY: freshly created descriptors owned here.
        let slave_in = unsafe { File::from_raw_fd(slave) };
        let slave_out = slave_in.try_clone().unwrap();
        let slave_err = slave_in.try_clone().unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_holla"));
        cmd.args(args)
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .env_remove("NO_COLOR")
            .env_remove("HOLLA_NO_MOTION")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("FORCE_COLOR")
            .stdin(Stdio::from(slave_in))
            .stdout(Stdio::from(slave_out))
            .stderr(Stdio::from(slave_err));
        for (k, v) in env {
            match v {
                Some(v) => {
                    cmd.env(k, v);
                }
                None => {
                    cmd.env_remove(k);
                }
            }
        }
        // SAFETY: only async-signal-safe calls run between fork and exec.
        unsafe {
            cmd.pre_exec(|| {
                setsid();
                ioctl(0, TIOCSCTTY, 0);
                Ok(())
            });
        }
        let child = cmd.spawn().expect("spawn holla on the pty");
        drop(cmd);
        // SAFETY: master is an owned open descriptor.
        let master = unsafe { File::from_raw_fd(master) };
        let output = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&output);
        let mut reader = master.try_clone().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => sink.lock().unwrap().extend_from_slice(&buf[..n]),
                }
            }
        });
        Self {
            master,
            child,
            output,
        }
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }

    fn wait_for(&self, what: &str, mut cond: impl FnMut(&str) -> bool) {
        let start = Instant::now();
        while !cond(&self.text()) {
            assert!(
                start.elapsed() < BOUND,
                "timed out waiting for {what}; output: {:?}",
                self.text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        // the pty input buffer is small: feed it in chunks and, on a write
        // error, say whether the application is still alive
        for (i, chunk) in bytes.chunks(512).enumerate() {
            if let Err(e) = self.master.write_all(chunk) {
                let status = self.child.try_wait().unwrap();
                panic!(
                    "write of chunk {i} to the pty failed: {e}; child status {status:?}; output tail: {:?}",
                    self.text().chars().rev().take(200).collect::<String>()
                );
            }
            self.master.flush().unwrap();
        }
    }

    fn wait_exit(&mut self) -> i32 {
        let start = Instant::now();
        loop {
            if let Some(s) = self.child.try_wait().unwrap() {
                return s.code().unwrap_or(-1);
            }
            assert!(
                start.elapsed() < BOUND,
                "holla did not exit; output: {:?}",
                self.text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// SGR classes found in a stream, counted separately: colour never hides
/// behind an attribute and an attribute never counts as colour. Crossterm
/// serializes the sixteen named colours as `38;5;n` with n below 16, so
/// that form counts as the basic palette, never as the 256-colour cube.
#[derive(Debug, Default, PartialEq, Eq)]
struct Sgr {
    truecolor: usize,
    /// `38;5;n` with n >= 16: the 256-colour cube and greys.
    indexed: usize,
    /// `3x`/`9x`/`4x`/`10x` or `38;5;n` with n < 16.
    basic: usize,
    reverse: usize,
    bold: usize,
    dim: usize,
    underline: usize,
    /// Distinct palette indexes below 16 that were used.
    palette: Vec<u32>,
}

fn classify(text: &str) -> Sgr {
    let mut s = Sgr::default();
    let mut rest = text;
    while let Some(i) = rest.find("\x1b[") {
        rest = &rest[i + 2..];
        let Some(end) = rest.find(|c: char| c.is_ascii_alphabetic()) else {
            break;
        };
        let (params, tail) = rest.split_at(end);
        let final_byte = tail.chars().next().unwrap();
        rest = &tail[1..];
        if final_byte != 'm' {
            continue;
        }
        let toks: Vec<u32> = params
            .split([';', ':'])
            .map(|t| t.parse().unwrap_or(0))
            .collect();
        let mut i = 0;
        while i < toks.len() {
            match toks[i] {
                38 | 48 | 58 => match toks.get(i + 1) {
                    Some(2) => {
                        s.truecolor += 1;
                        i += 4;
                    }
                    Some(5) => {
                        match toks.get(i + 2) {
                            Some(n) if *n < 16 => {
                                s.basic += 1;
                                if !s.palette.contains(n) {
                                    s.palette.push(*n);
                                }
                            }
                            _ => s.indexed += 1,
                        }
                        i += 2;
                    }
                    _ => {}
                },
                30..=37 | 40..=47 | 90..=97 | 100..=107 => s.basic += 1,
                7 => s.reverse += 1,
                1 => s.bold += 1,
                2 => s.dim += 1,
                4 => s.underline += 1,
                _ => {}
            }
            i += 1;
        }
    }
    s
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
    let mut pty = Pty::spawn(&base, env, 100, 30);
    pty.wait_for("the first frame", |t| {
        t.contains("holla") && t.contains("Suggested")
    });
    // a stable second frame: Tab moves to the preview and back
    pty.send(b"\t");
    std::thread::sleep(Duration::from_millis(150));
    pty.send(b"\x1b[Z");
    std::thread::sleep(Duration::from_millis(150));
    pty.send(b"\x1b");
    let code = pty.wait_exit();
    assert_eq!(code, 0, "clean exit: {:?}", pty.text());
    let text = pty.text();
    (classify(&text), text)
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
    let mut pty = Pty::spawn(
        &["--scenario", "first-use", "--motion", "reduced"],
        &[],
        100,
        30,
    );
    pty.wait_for("the first frame", |t| t.contains("holla"));
    let before = pty.text().len();
    // 8000 Ctrl+B presses: a single byte each (no escape-sequence boundary
    // ambiguity across pty reads) bound to nothing on the root, so every one
    // is an ignored event; the queue keeps feeding while the loop drains
    let flood: Vec<u8> = vec![0x02; 8000];
    let start = Instant::now();
    pty.send(&flood);
    // discovery ticks keep landing while the flood drains: the frame changes
    pty.wait_for("a redraw during the flood", |t| t.len() > before + 64);
    // a quit queued behind the flood is honoured once the flood is consumed
    pty.send(b"\x1b");
    let code = pty.wait_exit();
    let elapsed = start.elapsed();
    assert_eq!(code, 0, "clean exit after the flood: {:?}", pty.text());
    assert!(
        elapsed < Duration::from_secs(10),
        "8000 ignored events and a quit took {elapsed:?}"
    );
    eprintln!("F23d pty flood: 8000 ignored events + quit in {elapsed:?}");
}
