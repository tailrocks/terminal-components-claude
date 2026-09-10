//! F21: job-control suspension proved on an owned pseudo-terminal.
//!
//! The test creates a PTY and runs a minimal job-control parent on it (this
//! test binary in helper mode): the parent is the session leader with the
//! PTY as its controlling terminal, starts the holla preview in its own
//! foreground process group, takes the terminal back when the job stops and
//! hands it back on `fg`. That is the shape of a real shell without a shell's
//! own termios bookkeeping, so the termios observed while the job is stopped
//! is exactly what the application left behind.
//!
//! Proved: an external `SIGTSTP` leaves a canonical, echoing, signalling
//! terminal identical to the launch state while the process is stopped;
//! `fg` re-enters raw mode, the alternate screen, mouse capture and
//! bracketed paste and redraws at the geometry set while suspended; a
//! second cycle behaves the same; a normal quit afterwards leaves the launch
//! termios. Nothing here touches the developer's terminal.
//!
//! Scope: macOS and Linux. `SIGSTOP`/`SIGKILL` cannot be intercepted and a
//! lost output device cannot receive escapes; those cases are documented,
//! not proved. Startup-failure and panic restoration are unit tests of the
//! setup guard in `runtime.rs`.

#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::ffi::{c_int, c_ulong, c_void};
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::{c_int, c_ulong};
    pub const SIGTSTP: c_int = 18;
    pub const SIGCONT: c_int = 19;
    pub const SIGTTIN: c_int = 21;
    pub const SIGTTOU: c_int = 22;
    pub const TIOCSCTTY: c_ulong = 0x2000_7461;
    pub const TIOCSWINSZ: c_ulong = 0x8008_7467;
    pub const ICANON: u64 = 0x100;
    pub const ECHO: u64 = 0x8;
    pub const ISIG: u64 = 0x80;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Termios {
        pub c_iflag: u64,
        pub c_oflag: u64,
        pub c_cflag: u64,
        pub c_lflag: u64,
        pub c_cc: [u8; 20],
        pub c_ispeed: u64,
        pub c_ospeed: u64,
    }
    pub fn lflag(t: &Termios) -> u64 {
        t.c_lflag
    }
}

#[cfg(target_os = "linux")]
mod sys {
    use std::ffi::{c_int, c_ulong};
    pub const SIGTSTP: c_int = 20;
    pub const SIGCONT: c_int = 18;
    pub const SIGTTIN: c_int = 21;
    pub const SIGTTOU: c_int = 22;
    pub const TIOCSCTTY: c_ulong = 0x540E;
    pub const TIOCSWINSZ: c_ulong = 0x5414;
    pub const ICANON: u64 = 0x2;
    pub const ECHO: u64 = 0x8;
    pub const ISIG: u64 = 0x1;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Termios {
        pub c_iflag: u32,
        pub c_oflag: u32,
        pub c_cflag: u32,
        pub c_lflag: u32,
        pub c_line: u8,
        pub c_cc: [u8; 32],
        pub c_ispeed: u32,
        pub c_ospeed: u32,
    }
    pub fn lflag(t: &Termios) -> u64 {
        u64::from(t.c_lflag)
    }
}

use sys::*;

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
    fn tcgetattr(fd: c_int, t: *mut Termios) -> c_int;
    fn tcsetpgrp(fd: c_int, pgrp: c_int) -> c_int;
    fn kill(pid: c_int, sig: c_int) -> c_int;
    fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int;
    fn setsid() -> c_int;
    fn setpgid(pid: c_int, pgid: c_int) -> c_int;
    fn getpgrp() -> c_int;
    fn ioctl(fd: c_int, req: c_ulong, ...) -> c_int;
    fn signal(sig: c_int, handler: usize) -> usize;
}

const SIG_IGN: usize = 1;
const WNOHANG: c_int = 1;
const WUNTRACED: c_int = 2;
const BOUND: Duration = Duration::from_secs(10);
const HELPER: &str = "JUNIE_TERMINAL_SUSPEND_HELPER";
const HELPER_BIN: &str = "JUNIE_TERMINAL_SUSPEND_BIN";

fn stopped(status: c_int) -> bool {
    status & 0xff == 0x7f
}

fn exit_code(status: c_int) -> i32 {
    if status & 0x7f == 0 {
        (status >> 8) & 0xff
    } else {
        -(status & 0x7f)
    }
}

/// The job-control parent: session leader on the PTY, one foreground job.
/// Prints `@@PID=`, `@@STOPPED`, `@@EXIT=` markers; `fg` on its stdin hands
/// the terminal back to the job and continues it.
fn helper_main() -> ! {
    let bin = std::env::var(HELPER_BIN).expect("application path");
    // SAFETY: plain libc calls on the helper's own process and descriptors.
    unsafe {
        setsid();
        ioctl(0, TIOCSCTTY, 0);
        signal(SIGTTOU, SIG_IGN);
        signal(SIGTTIN, SIG_IGN);
    }
    let mut cmd = Command::new(bin);
    cmd.args([
        "--scenario",
        "first-use",
        "--motion",
        "paused",
        "--frame",
        "0",
    ]);
    // SAFETY: setpgid is async-signal-safe.
    unsafe {
        cmd.pre_exec(|| {
            setpgid(0, 0);
            Ok(())
        });
    }
    let child = cmd.spawn().expect("spawn the application");
    let pid = child.id() as c_int;
    // SAFETY: both sides set the group so the foreground hand-off is not racy.
    unsafe {
        setpgid(pid, pid);
        tcsetpgrp(0, pid);
    }
    println!("\n@@PID={pid}");
    let stdin = std::io::stdin();
    loop {
        let mut status: c_int = 0;
        // SAFETY: waiting on the tracked job.
        let rc = unsafe { waitpid(pid, &mut status, WUNTRACED) };
        if rc != pid {
            println!("\n@@EXIT=-1");
            std::process::exit(1);
        }
        if stopped(status) {
            // SAFETY: the helper takes the terminal back like a shell.
            unsafe {
                tcsetpgrp(0, getpgrp());
            }
            println!("\n@@STOPPED");
            let mut line = String::new();
            let _ = stdin.lock().read_line(&mut line);
            if line.trim() != "fg" {
                println!("\n@@EXIT=-2");
                std::process::exit(2);
            }
            // SAFETY: foreground back to the job, then continue it.
            unsafe {
                tcsetpgrp(0, pid);
                kill(pid, SIGCONT);
            }
            continue;
        }
        println!("\n@@EXIT={}", exit_code(status));
        std::process::exit(0);
    }
}

/// One owned pseudo-terminal running the helper. Every wait is bounded;
/// the helper and its job are killed on drop.
struct Pty {
    master: File,
    helper: std::process::Child,
    output: Arc<Mutex<Vec<u8>>>,
}

impl Pty {
    fn spawn(cols: u16, rows: u16) -> Self {
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
        let helper = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "helper_entry", "--nocapture"])
            .env(HELPER, "1")
            .env(HELPER_BIN, env!("CARGO_BIN_EXE_holla"))
            .env("TERM", "xterm-256color")
            .env_remove("NO_COLOR")
            .env_remove("HOLLA_NO_MOTION")
            .stdin(Stdio::from(slave_in))
            .stdout(Stdio::from(slave_out))
            .stderr(Stdio::from(slave_err))
            .spawn()
            .expect("spawn the job-control helper");
        // SAFETY: master is an owned open descriptor.
        let master = unsafe { File::from_raw_fd(master) };
        let output = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&output);
        let mut reader = master.try_clone().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => sink.lock().unwrap().extend_from_slice(&buf[..n]),
                }
            }
        });
        Self {
            master,
            helper,
            output,
        }
    }

    fn termios(&self) -> Termios {
        // SAFETY: zeroed storage of the platform layout, filled by libc.
        let mut t: Termios = unsafe { std::mem::zeroed() };
        let rc = unsafe { tcgetattr(self.master.as_raw_fd(), &mut t) };
        assert_eq!(rc, 0, "tcgetattr on the pty");
        t
    }

    fn is_raw(&self) -> bool {
        lflag(&self.termios()) & ICANON == 0
    }

    fn wait_until(&self, what: &str, mut cond: impl FnMut(&Self) -> bool) {
        let start = Instant::now();
        while !cond(self) {
            assert!(
                start.elapsed() < BOUND,
                "timed out waiting for {what}; output so far: {:?}",
                self.output_text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn resize(&self, cols: u16, rows: u16) {
        let win = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: the pty master accepts TIOCSWINSZ with a winsize.
        let rc = unsafe { ioctl(self.master.as_raw_fd(), TIOCSWINSZ, &win as *const Winsize) };
        assert_eq!(rc, 0, "TIOCSWINSZ");
    }

    fn send(&mut self, bytes: &[u8]) {
        self.master.write_all(bytes).unwrap();
        self.master.flush().unwrap();
    }

    fn output_text(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }

    fn output_len(&self) -> usize {
        self.output.lock().unwrap().len()
    }

    fn marker(&self, name: &str) -> Option<String> {
        let text = self.output_text();
        text.lines()
            .rev()
            .find_map(|l| l.trim().strip_prefix(name).map(str::to_owned))
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        let _ = self.helper.kill();
        let _ = self.helper.wait();
    }
}

fn count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

/// Helper mode entry: only the re-invoked child runs the job-control parent.
#[test]
fn helper_entry() {
    if std::env::var_os(HELPER).is_some() {
        helper_main();
    }
}

#[test]
fn external_sigtstp_restores_the_shell_and_fg_reenters_at_the_new_geometry() {
    let mut pty = Pty::spawn(120, 40);
    let before = pty.termios();
    assert_eq!(
        lflag(&before) & (ICANON | ECHO | ISIG),
        ICANON | ECHO | ISIG,
        "a fresh pty is canonical, echoing and signalling"
    );
    pty.wait_until("the job pid", |p| p.marker("@@PID=").is_some());
    let pid: c_int = pty.marker("@@PID=").unwrap().trim().parse().unwrap();
    pty.wait_until("raw mode", |p| p.is_raw());
    pty.wait_until("first frame", |p| p.output_text().contains("holla"));
    let drawn = pty.output_len();

    for cycle in 1..=2 {
        // an external stop request: the handler defers to the event loop,
        // which restores the terminal and then stops with the default action
        // SAFETY: signalling the job whose pid the helper reported.
        assert_eq!(unsafe { kill(pid, SIGTSTP) }, 0, "kill(SIGTSTP)");
        pty.wait_until("the job to stop", |p| {
            count(&p.output_text(), "@@STOPPED") == cycle
        });
        let stopped_state = pty.termios();
        assert_eq!(
            lflag(&stopped_state) & (ICANON | ECHO | ISIG),
            ICANON | ECHO | ISIG,
            "cycle {cycle}: the shell gets canonical, echoing input back while the job is stopped"
        );
        assert_eq!(
            stopped_state, before,
            "cycle {cycle}: exact launch termios while stopped"
        );
        let text = pty.output_text();
        assert_eq!(
            count(&text, "\x1b[?1049l"),
            cycle,
            "cycle {cycle}: the alternate screen was left once per suspension"
        );
        assert!(
            text.contains("\x1b[?1000l") || text.contains("\x1b[?1006l"),
            "mouse capture released: {text:?}"
        );
        assert!(text.contains("\x1b[?2004l"), "bracketed paste released");
        assert!(
            text.contains("\x1b[?7h"),
            "line wrap re-enabled for the shell"
        );

        // geometry changes while suspended; continuation must rebuild it
        let (cols, rows) = if cycle == 1 { (100, 30) } else { (80, 24) };
        pty.resize(cols, rows);
        let seen = pty.output_len();
        pty.send(b"fg\n");
        pty.wait_until("raw mode after fg", |p| p.is_raw());
        pty.wait_until("a full redraw after fg", |p| {
            let text = p.output_text();
            count(&text, "\x1b[?1049h") == cycle + 1 && text.len() > seen + 200
        });
        let text = pty.output_text();
        let after = &text[seen..];
        assert!(
            after.contains("\x1b[?1049h"),
            "cycle {cycle}: alternate screen re-entered"
        );
        assert!(
            after.contains("\x1b[?1000h") || after.contains("\x1b[?1006h"),
            "mouse capture re-enabled"
        );
        assert!(after.contains("\x1b[?2004h"), "bracketed paste re-enabled");
        assert!(
            after.contains("holla"),
            "cycle {cycle}: the frame is drawn again after continuation: {after:?}"
        );
        // the redraw addresses the last row of the new geometry, never the old
        let new_last = format!("\x1b[{rows};");
        let old_last = format!("\x1b[{};", if cycle == 1 { 40 } else { 30 });
        assert!(
            after.contains(&new_last),
            "cycle {cycle}: the resumed frame addresses row {rows} of {cols}x{rows}: {after:?}"
        );
        assert!(
            !after.contains(&old_last),
            "cycle {cycle}: nothing is drawn at the stale geometry: {after:?}"
        );
        assert!(
            pty.marker("@@EXIT=").is_none(),
            "cycle {cycle}: still running"
        );
    }
    assert!(
        pty.output_len() > drawn,
        "frames were drawn after the first"
    );

    // a normal quit from the resumed session: Esc at an empty root
    pty.send(b"\x1b");
    pty.wait_until("exit", |p| p.marker("@@EXIT=").is_some());
    assert_eq!(pty.marker("@@EXIT=").unwrap().trim(), "0", "clean exit");
    let start = Instant::now();
    loop {
        if pty.helper.try_wait().unwrap().is_some() {
            break;
        }
        assert!(start.elapsed() < BOUND, "helper did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
    let after = pty.termios();
    assert_eq!(after, before, "the final termios equals the launch state");
    let text = pty.output_text();
    assert_eq!(
        count(&text, "\x1b[?1049l"),
        3,
        "two suspensions and one exit each left the alternate screen"
    );
    // the job was the helper's child, not ours: a wait here has no child
    let mut status = 0;
    // SAFETY: probing a pid that was never this process's child.
    assert_eq!(unsafe { waitpid(pid, &mut status, WNOHANG) }, -1);
}
