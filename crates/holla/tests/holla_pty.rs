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

const BOUND: Duration = Duration::from_secs(30);

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

        // SAFETY: `master` and `slave` are distinct writable integers, and `win` is an
        // initialized Winsize that remains live for this synchronous call. The optional
        // name and termios pointers are null.
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

        // SAFETY: `openpty` succeeded, so `slave` is open and not yet owned by a File.
        // This transfers ownership once; later clones are separate Stdio descriptors.
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

        unsafe {
            cmd.pre_exec(|| {
                setsid();
                ioctl(0, TIOCSCTTY, 0);
                Ok(())
            });
        }
        let child = match cmd.spawn() {
            Ok(child) => child,
            Err(error) => panic!(
                "spawn holla on the pty: {error:?}; command: {:?}; args: {args:?}; terminal dimensions: {cols}x{rows}",
                env!("CARGO_BIN_EXE_holla"),
            ),
        };
        drop(cmd);

        // SAFETY: `openpty` returned this open master descriptor, which has not been
        // wrapped or closed in the parent. File takes ownership once before cloning.
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

#[derive(Debug, Default, PartialEq, Eq)]
struct Sgr {
    truecolor: usize,
    indexed: usize,
    basic: usize,
    reverse: usize,
    bold: usize,
    dim: usize,
    underline: usize,
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

    pty.send(b"\t");
    std::thread::sleep(Duration::from_millis(150));
    pty.send(b"\x1b[Z");
    std::thread::sleep(Duration::from_millis(150));

    pty.send(b"\x03");
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
    let mut pty = Pty::spawn(
        &["--scenario", "first-use", "--motion", "reduced"],
        &[],
        100,
        30,
    );
    pty.wait_for("the first frame", |t| t.contains("holla"));
    let before = pty.text().len();

    let flood: Vec<u8> = vec![0x02; 2000];
    let start = Instant::now();
    pty.send(&flood);

    pty.wait_for("a redraw during the flood", |t| t.len() > before + 64);

    pty.send(b"\x03");
    let code = pty.wait_exit();
    let elapsed = start.elapsed();
    assert_eq!(code, 0, "clean exit after the flood: {:?}", pty.text());
    assert!(
        elapsed < Duration::from_secs(60),
        "2000 ignored events and a quit took {elapsed:?}"
    );

    eprintln!("F23d pty flood: 2000 ignored events + quit in {elapsed:?}");
}

const KEYBOARD_QUERY: &str = termrock::core::keys::keyboard::QUERY;
const PRIMARY_DEVICE_ATTRIBUTES_QUERY: &str = "\x1b[c";
const KEYBOARD_POP: &str = termrock::core::keys::keyboard::POP;
const LEAVE_ALTERNATE_SCREEN: &str = "\x1b[?1049l";

fn keyboard_policy_pty(policy: &str) -> Pty {
    Pty::spawn(
        &[
            "--scenario",
            "rust-dirty",
            "--motion",
            "paused",
            "--frame",
            "40",
            "--color",
            "truecolor",
        ],
        &[
            (
                termrock::core::keys::keyboard::KeyboardProtocol::ENV,
                Some(policy),
            ),
            (
                termrock::core::keys::keyboard::KeyboardProtocol::LEGACY_ENV,
                None,
            ),
        ],
        100,
        30,
    )
}

fn wait_for_holla_ready(pty: &Pty) {
    pty.wait_for("the normal first frame", |text| {
        text.contains("holla") && text.contains("Suggested")
    });
}

fn wire_count(output: &str, sequence: &str) -> usize {
    output.matches(sequence).count()
}

fn finish_keyboard_policy_child(mut pty: Pty) -> String {
    // Holla's `q` opens a confirmation dialog; Ctrl-C is the existing tested
    // bounded shutdown event and makes no exact-once ordinary-key claim.
    pty.send(b"\x03");
    let status = pty.wait_exit();
    assert_eq!(status, 0, "controlled PTY shutdown: {:?}", pty.text());
    pty.wait_for("alternate-screen restoration", |text| {
        text.contains(LEAVE_ALTERNATE_SCREEN)
    });
    pty.text()
}

fn assert_terminal_restore_order(output: &str, owned_pop: bool) {
    let cleanup = concat!(
        "\x1b[?25h",
        "\x1b[0 q",
        "\x1b[?25h",
        "\x1b[?7h",
        "\x1b[?2004l",
        "\x1b[?1006l\x1b[?1015l\x1b[?1003l\x1b[?1002l\x1b[?1000l",
        "\x1b[?1049l",
    );
    if owned_pop {
        assert_eq!(
            wire_count(output, KEYBOARD_POP),
            1,
            "one owned keyboard POP"
        );
        let mut expected = String::from(KEYBOARD_POP);
        expected.push_str(cleanup);
        assert_eq!(
            wire_count(output, &expected),
            1,
            "owned POP precedes the contiguous terminal cleanup wire sequence"
        );
    } else {
        assert_eq!(
            wire_count(output, KEYBOARD_POP),
            0,
            "no unowned keyboard POP"
        );
        assert_eq!(
            wire_count(output, cleanup),
            1,
            "the exact terminal cleanup wire sequence is emitted once"
        );
    }
    // Pty observes terminal wire output and child exit, not kernel termios.
}

fn assert_query_waiting_without_push(output: &str) {
    let push = termrock::core::keys::keyboard::push(termrock::core::keys::keyboard::FLAGS);
    assert_eq!(
        wire_count(output, KEYBOARD_QUERY),
        1,
        "one real keyboard query"
    );
    assert_eq!(wire_count(output, &push), 0, "no push before a query reply");
    assert_eq!(
        wire_count(output, PRIMARY_DEVICE_ATTRIBUTES_QUERY),
        0,
        "the bridged query does not retry through Crossterm's DA probe"
    );
}

fn assert_query_then_push_before_ready(output: &str) {
    let push = termrock::core::keys::keyboard::push(termrock::core::keys::keyboard::FLAGS);
    assert_eq!(
        wire_count(output, KEYBOARD_QUERY),
        1,
        "one real keyboard query"
    );
    assert_eq!(wire_count(output, &push), 1, "one owned keyboard PUSH");
    assert_eq!(
        wire_count(output, PRIMARY_DEVICE_ATTRIBUTES_QUERY),
        0,
        "the bridged query does not retry through Crossterm's DA probe"
    );
    let query_at = output.find(KEYBOARD_QUERY).unwrap_or(usize::MAX);
    let push_at = output.find(&push).unwrap_or(usize::MAX);
    let ready_at = output.find("Suggested").unwrap_or(usize::MAX);
    assert!(
        query_at < push_at && push_at < ready_at,
        "query, push, then ready"
    );
}

fn assert_ready_before_push(output: &str) {
    let push = termrock::core::keys::keyboard::push(termrock::core::keys::keyboard::FLAGS);
    assert_eq!(
        wire_count(output, KEYBOARD_QUERY),
        0,
        "explicit policy does not query"
    );
    assert_eq!(
        wire_count(output, PRIMARY_DEVICE_ATTRIBUTES_QUERY),
        0,
        "explicit policy does not invoke a DA probe"
    );
    assert_eq!(wire_count(output, &push), 1, "one owned keyboard PUSH");
    let push_at = output.find(&push).unwrap_or(usize::MAX);
    let ready_at = output.find("Suggested").unwrap_or(usize::MAX);
    assert!(
        push_at < ready_at,
        "explicit On pushes before the ready frame"
    );
}

#[test]
fn normal_session_auto_accepts_zero_keyboard_flags() {
    let mut pty = keyboard_policy_pty("auto");
    pty.wait_for("the real keyboard query", |text| {
        text.contains(KEYBOARD_QUERY)
    });
    assert_query_waiting_without_push(&pty.text());
    pty.send(b"\x1b[?0u");
    wait_for_holla_ready(&pty);
    let before_exit = pty.text();
    assert_query_then_push_before_ready(&before_exit);

    let output = finish_keyboard_policy_child(pty);
    assert_query_then_push_before_ready(&output);
    assert_terminal_restore_order(&output, true);
}

#[test]
fn normal_session_auto_accepts_nonzero_keyboard_flags() {
    let mut pty = keyboard_policy_pty("auto");
    pty.wait_for("the real keyboard query", |text| {
        text.contains(KEYBOARD_QUERY)
    });
    assert_query_waiting_without_push(&pty.text());
    pty.send(b"\x1b[?5u");
    wait_for_holla_ready(&pty);
    let before_exit = pty.text();
    assert_query_then_push_before_ready(&before_exit);

    let output = finish_keyboard_policy_child(pty);
    assert_query_then_push_before_ready(&output);
    assert_terminal_restore_order(&output, true);
}

#[test]
fn normal_session_auto_without_reply_does_not_push() {
    let pty = keyboard_policy_pty("auto");
    pty.wait_for("the real keyboard query", |text| {
        text.contains(KEYBOARD_QUERY)
    });
    assert_query_waiting_without_push(&pty.text());
    let query_observed_at = Instant::now();
    wait_for_holla_ready(&pty);
    let elapsed = query_observed_at.elapsed();
    eprintln!("Auto without reply: first frame observed after {elapsed:?}");
    let before_exit = pty.text();
    let push = termrock::core::keys::keyboard::push(termrock::core::keys::keyboard::FLAGS);
    assert_eq!(
        wire_count(&before_exit, KEYBOARD_QUERY),
        1,
        "one real query"
    );
    assert_eq!(wire_count(&before_exit, PRIMARY_DEVICE_ATTRIBUTES_QUERY), 0);
    assert_eq!(
        wire_count(&before_exit, &push),
        0,
        "no push without a reply"
    );

    let output = finish_keyboard_policy_child(pty);
    assert_eq!(wire_count(&output, KEYBOARD_QUERY), 1, "no retry query");
    assert_eq!(wire_count(&output, PRIMARY_DEVICE_ATTRIBUTES_QUERY), 0);
    assert_eq!(wire_count(&output, &push), 0, "no push without a reply");
    assert_terminal_restore_order(&output, false);
}

#[test]
fn normal_session_on_pushes_without_query() {
    let pty = keyboard_policy_pty("on");
    wait_for_holla_ready(&pty);
    let before_exit = pty.text();
    assert_ready_before_push(&before_exit);

    let output = finish_keyboard_policy_child(pty);
    assert_ready_before_push(&output);
    assert_terminal_restore_order(&output, true);
}

#[test]
fn normal_session_off_never_queries_or_pushes() {
    let pty = keyboard_policy_pty("off");
    wait_for_holla_ready(&pty);
    let before_exit = pty.text();
    let push = termrock::core::keys::keyboard::push(termrock::core::keys::keyboard::FLAGS);
    assert_eq!(wire_count(&before_exit, KEYBOARD_QUERY), 0);
    assert_eq!(wire_count(&before_exit, PRIMARY_DEVICE_ATTRIBUTES_QUERY), 0);
    assert_eq!(wire_count(&before_exit, &push), 0);

    let output = finish_keyboard_policy_child(pty);
    assert_eq!(wire_count(&output, KEYBOARD_QUERY), 0);
    assert_eq!(wire_count(&output, PRIMARY_DEVICE_ATTRIBUTES_QUERY), 0);
    assert_eq!(wire_count(&output, &push), 0);
    assert_terminal_restore_order(&output, false);
}
