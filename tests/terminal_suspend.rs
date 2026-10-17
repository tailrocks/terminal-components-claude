//! F21: job-control suspension proved on a tuiscotti PTY session.
//!
//! tuiscotti (`tuiscotti::tui`, T17) owns the pseudo-terminal and its
//! emulator; no hand-rolled pseudo-terminal here. The session runs a minimal
//! job-control parent (this test binary in helper mode): the parent is the
//! session leader with the PTY as its controlling terminal, starts the holla
//! preview in its own foreground process group, takes the terminal back when
//! the job stops and hands it back on `fg`. That is the shape of a real
//! shell without a shell's own termios bookkeeping, so the termios observed
//! while the job is stopped is exactly what the application left behind.
//!
//! The helper reports through a marker file (its pty output is the screen
//! the emulator draws, and markers printed there could land on the
//! alternate screen and be overwritten). The test reads termios through the
//! helper's terminal device, opened without becoming its controlling
//! terminal, and reads terminal modes (1049, 1000/1002/1003/1006, 2004,
//! 1004, 7) from tuiscotti's observation.
//!
//! Proved: an external `SIGTSTP` leaves a canonical, echoing, signalling
//! terminal identical to the launch state while the process is stopped,
//! with the alternate screen, mouse capture, bracketed paste and focus
//! reporting released and line wrap back on; `fg` re-enters raw mode, the
//! alternate screen, mouse capture, bracketed paste and focus reporting and
//! redraws exactly the frame a fresh launch draws at the geometry set while
//! suspended; a second cycle behaves the same; a normal quit afterwards
//! leaves the launch termios and the main screen. Nothing here touches the
//! developer's terminal.
//!
//! Scope: macOS and Linux. `SIGSTOP`/`SIGKILL` cannot be intercepted and a
//! lost output device cannot receive escapes; those cases are documented,
//! not proved. Startup-failure and panic restoration are unit tests of the
//! setup guard in `runtime.rs`.

#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::ffi::{CStr, c_int};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use tuiscotti::Observation;
use tuiscotti::tui::{CancelToken, Session, Tui};

const BOUND: Duration = Duration::from_secs(10);
const HELPER: &str = "JUNIE_TERMINAL_SUSPEND_HELPER";
const HELPER_BIN: &str = "JUNIE_TERMINAL_SUSPEND_BIN";
const HELPER_MARKERS: &str = "JUNIE_TERMINAL_SUSPEND_MARKERS";
const APP_ARGS: [&str; 6] = [
    "--scenario",
    "first-use",
    "--motion",
    "paused",
    "--frame",
    "0",
];

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

/// One marker line, appended and flushed so the test never reads half.
fn mark(path: &Path, line: &str) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open the marker file");
    writeln!(f, "{line}").expect("write a marker");
    f.flush().expect("flush a marker");
}

/// The job-control parent: session leader on the PTY, one foreground job.
/// Writes `TTY=`, `PID=`, `STOPPED` and `EXIT=` markers; waits for `start`
/// on its stdin before launching the job (so the test reads the launch
/// termios first), and `fg` hands the terminal back to the job and
/// continues it.
fn helper_main() -> ! {
    let bin = std::env::var(HELPER_BIN).expect("application path");
    let markers = PathBuf::from(std::env::var(HELPER_MARKERS).expect("marker file"));
    // SAFETY: plain libc calls on the helper's own process and descriptors.
    // tuiscotti already made it a session leader with the PTY as its
    // controlling terminal; repeating both is harmless and keeps the helper
    // a shell shape on its own.
    unsafe {
        libc::setsid();
        libc::ioctl(0, libc::TIOCSCTTY as _, 0);
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }
    // SAFETY: ttyname returns a static buffer or null.
    let tty = unsafe { libc::ttyname(0) };
    assert!(!tty.is_null(), "the helper's stdin is a terminal");
    // SAFETY: non-null, NUL-terminated per ttyname's contract.
    let tty = unsafe { CStr::from_ptr(tty) }
        .to_string_lossy()
        .into_owned();
    mark(&markers, &format!("TTY={tty}"));
    let stdin = std::io::stdin();
    let mut line = String::new();
    let _ = stdin.lock().read_line(&mut line);
    if line.trim() != "start" {
        mark(&markers, "EXIT=-3");
        std::process::exit(3);
    }
    let mut cmd = Command::new(bin);
    cmd.args(APP_ARGS);
    // SAFETY: setpgid is async-signal-safe.
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
    // the job is reaped by the raw waitpid loop below, not by `Child::wait`
    #[allow(clippy::zombie_processes)]
    let child = cmd.spawn().expect("spawn the application");
    let pid = child.id() as c_int;
    // SAFETY: both sides set the group so the foreground hand-off is not racy.
    unsafe {
        libc::setpgid(pid, pid);
        libc::tcsetpgrp(0, pid);
    }
    mark(&markers, &format!("PID={pid}"));
    loop {
        let mut status: c_int = 0;
        // SAFETY: waiting on the tracked job.
        let rc = unsafe { libc::waitpid(pid, &mut status, libc::WUNTRACED) };
        if rc != pid {
            mark(&markers, "EXIT=-1");
            std::process::exit(1);
        }
        if stopped(status) {
            // SAFETY: the helper takes the terminal back like a shell.
            unsafe {
                libc::tcsetpgrp(0, libc::getpgrp());
            }
            mark(&markers, "STOPPED");
            let mut line = String::new();
            let _ = stdin.lock().read_line(&mut line);
            if line.trim() != "fg" {
                mark(&markers, "EXIT=-2");
                std::process::exit(2);
            }
            // SAFETY: foreground back to the job, then continue it.
            unsafe {
                libc::tcsetpgrp(0, pid);
                libc::kill(pid, libc::SIGCONT);
            }
            continue;
        }
        mark(&markers, &format!("EXIT={}", exit_code(status)));
        std::process::exit(0);
    }
}

/// A fresh marker file path, unique per test process.
fn marker_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "junie-terminal-suspend-{}-{nanos}.markers",
        std::process::id()
    ))
}

/// The helper on a tuiscotti PTY session. Every wait is bounded; the
/// session kills the helper on drop and the marker file is removed.
struct Job {
    session: Session,
    markers: PathBuf,
}

impl Job {
    fn spawn(cols: u16, rows: u16) -> Self {
        let markers = marker_path();
        let session = Tui::new([std::env::current_exe().unwrap()])
            .args(["--exact", "helper_entry", "--nocapture"])
            .size(cols, rows)
            .env(HELPER, "1")
            .env(HELPER_BIN, env!("CARGO_BIN_EXE_holla"))
            .env(HELPER_MARKERS, &markers)
            .env("TERM", "xterm-256color")
            .env_remove("NO_COLOR")
            .env_remove("HOLLA_NO_MOTION")
            .spawn()
            .expect("spawn the job-control helper on a tuiscotti pty");
        Self { session, markers }
    }

    fn marker_lines(&self) -> Vec<String> {
        std::fs::read_to_string(&self.markers)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn marker(&self, name: &str) -> Option<String> {
        self.marker_lines()
            .iter()
            .rev()
            .find_map(|l| l.strip_prefix(name).map(str::to_owned))
    }

    fn count(&self, line: &str) -> usize {
        self.marker_lines().iter().filter(|l| *l == line).count()
    }

    fn wait_until(&self, what: &str, mut cond: impl FnMut(&Self) -> bool) {
        let start = Instant::now();
        while !cond(self) {
            assert!(
                start.elapsed() < BOUND,
                "timed out waiting for {what}; markers {:?}; screen:\n{}",
                self.marker_lines(),
                self.observe().map(|o| text(&o)).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn observe(&self) -> Option<Observation> {
        self.session.observe_now().ok()
    }

    /// The PTY's termios, read through the helper's terminal device opened
    /// without becoming its controlling terminal.
    fn termios(&self) -> libc::termios {
        let tty = self.marker("TTY=").expect("the helper reported its tty");
        let dev: File = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(&tty)
            .unwrap_or_else(|e| panic!("open {tty}: {e}"));
        // SAFETY: zeroed storage of the platform layout, filled by libc.
        let mut t: libc::termios = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::tcgetattr(dev.as_raw_fd(), &mut t) };
        assert_eq!(rc, 0, "tcgetattr on {tty}");
        t
    }

    fn is_raw(&self) -> bool {
        self.termios().c_lflag & libc::ICANON == 0
    }

    fn wait_screen(&self, what: &str, cond: impl Fn(&Observation) -> bool) -> Observation {
        self.session
            .wait_predicate(cond, Instant::now() + BOUND, &CancelToken::new())
            .unwrap_or_else(|e| panic!("timed out waiting for {what}: {e}"))
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        let _ = self.session.close();
        let _ = std::fs::remove_file(&self.markers);
    }
}

fn termios_bytes(t: &libc::termios) -> Vec<u8> {
    // SAFETY: a plain-old-data C struct viewed as its bytes.
    unsafe {
        std::slice::from_raw_parts(
            (t as *const libc::termios).cast::<u8>(),
            std::mem::size_of::<libc::termios>(),
        )
    }
    .to_vec()
}

fn lflag(t: &libc::termios) -> libc::tcflag_t {
    t.c_lflag
}

const SHELL_LFLAGS: libc::tcflag_t = libc::ICANON | libc::ECHO | libc::ISIG;

fn text(obs: &Observation) -> String {
    tuiscotti::render::frame_from_screen(&obs.screen, "terminal_suspend").text()
}

fn modes(obs: &Observation) -> Vec<u16> {
    obs.state
        .modes
        .known()
        .cloned()
        .expect("the backend reports terminal modes")
}

fn has_mouse(m: &[u16]) -> bool {
    m.iter().any(|n| [1000, 1002, 1003, 1006].contains(n))
}

/// The frame a fresh launch of the same scenario draws at `cols`x`rows`:
/// what the resumed job must redraw, cell text for cell text.
fn fresh_frame(cols: u16, rows: u16) -> String {
    let session = Tui::new([env!("CARGO_BIN_EXE_holla")])
        .args(APP_ARGS)
        .size(cols, rows)
        .env("TERM", "xterm-256color")
        .env_remove("NO_COLOR")
        .env_remove("HOLLA_NO_MOTION")
        .spawn()
        .expect("spawn a fresh holla");
    session
        .wait_predicate(
            |o| text(o).contains("holla"),
            Instant::now() + BOUND,
            &CancelToken::new(),
        )
        .expect("a fresh first frame");
    let obs = session
        .wait_stable_quiet(
            Instant::now() + BOUND,
            Duration::from_millis(300),
            &CancelToken::new(),
        )
        .expect("a settled fresh frame");
    let _ = session.close();
    text(&obs)
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
    let job = Job::spawn(120, 40);
    job.wait_until("the helper's tty", |j| j.marker("TTY=").is_some());
    let before = job.termios();
    assert_eq!(
        lflag(&before) & SHELL_LFLAGS,
        SHELL_LFLAGS,
        "a fresh pty is canonical, echoing and signalling"
    );
    job.session.send_text("start\n").expect("start the job");
    job.wait_until("the job pid", |j| j.marker("PID=").is_some());
    let pid: c_int = job.marker("PID=").unwrap().trim().parse().unwrap();
    job.wait_until("raw mode", Job::is_raw);
    let first = job.wait_screen("first frame", |o| {
        text(o).contains("holla") && modes(o).contains(&1049)
    });
    let drawn = first.revision;

    for cycle in 1..=2 {
        // an external stop request: the handler defers to the event loop,
        // which restores the terminal and then stops with the default action
        // SAFETY: signalling the job whose pid the helper reported.
        assert_eq!(
            unsafe { libc::kill(pid, libc::SIGTSTP) },
            0,
            "kill(SIGTSTP)"
        );
        job.wait_until("the job to stop", |j| j.count("STOPPED") == cycle);
        let stopped_state = job.termios();
        assert_eq!(
            lflag(&stopped_state) & SHELL_LFLAGS,
            SHELL_LFLAGS,
            "cycle {cycle}: the shell gets canonical, echoing input back while the job is stopped"
        );
        assert_eq!(
            termios_bytes(&stopped_state),
            termios_bytes(&before),
            "cycle {cycle}: exact launch termios while stopped"
        );
        // the job wrote its restore before it stopped; the observation
        // after the stop carries it
        let obs = job.observe().expect("an observation while stopped");
        let m = modes(&obs);
        assert!(
            !m.contains(&1049),
            "cycle {cycle}: the alternate screen was left: {m:?}"
        );
        assert!(!has_mouse(&m), "mouse capture released: {m:?}");
        assert!(!m.contains(&2004), "bracketed paste released: {m:?}");
        assert!(!m.contains(&1004), "focus reporting released (SIg1): {m:?}");
        assert!(m.contains(&7), "line wrap re-enabled for the shell: {m:?}");

        // geometry changes while suspended; continuation must rebuild it
        let (cols, rows) = if cycle == 1 { (100, 30) } else { (80, 24) };
        let expected = fresh_frame(cols, rows);
        job.session
            .resize(cols, rows)
            .expect("resize while stopped");
        job.session.send_text("fg\n").expect("fg");
        job.wait_until("raw mode after fg", Job::is_raw);
        // the redraw is the frame a fresh launch draws at the new geometry:
        // nothing at the stale geometry, nothing half-written
        let resumed = job.session.wait_predicate(
            |o| modes(o).contains(&1049) && text(o) == expected,
            Instant::now() + BOUND,
            &CancelToken::new(),
        );
        let resumed = resumed.unwrap_or_else(|e| {
            panic!(
                "cycle {cycle}: the resumed frame is a fresh {cols}x{rows} frame: {e}\n--- expected ---\n{expected}"
            )
        });
        assert_eq!(
            (resumed.screen.cols(), resumed.screen.rows()),
            (cols, rows),
            "cycle {cycle}: the emulator follows the new geometry"
        );
        let m = modes(&resumed);
        assert!(
            m.contains(&1049),
            "cycle {cycle}: alternate screen re-entered"
        );
        assert!(has_mouse(&m), "mouse capture re-enabled: {m:?}");
        assert!(m.contains(&2004), "bracketed paste re-enabled: {m:?}");
        assert!(
            m.contains(&1004),
            "focus reporting re-enabled (SIg1): {m:?}"
        );
        assert!(
            text(&resumed).contains("holla"),
            "cycle {cycle}: the frame is drawn again after continuation"
        );
        assert!(
            job.marker("EXIT=").is_none(),
            "cycle {cycle}: still running"
        );
    }
    assert!(
        job.session.revision() > drawn,
        "frames were drawn after the first"
    );

    // a normal quit from the resumed session: Ctrl+Q on a clean shell
    // (Esc never quits, keyboard audit #21)
    job.session.send_bytes(b"\x11").expect("Ctrl+Q");
    job.wait_until("exit", |j| j.marker("EXIT=").is_some());
    assert_eq!(job.marker("EXIT=").unwrap().trim(), "0", "clean exit");
    let exit = job
        .session
        .wait_exit(Instant::now() + BOUND, &CancelToken::new())
        .expect("the helper exits");
    assert!(exit.status.success(), "helper exit: {}", exit.status);
    let after = job.termios();
    assert_eq!(
        termios_bytes(&after),
        termios_bytes(&before),
        "the final termios equals the launch state"
    );
    let m = modes(&exit.observation);
    assert!(
        !m.contains(&1049) && !has_mouse(&m) && !m.contains(&2004) && !m.contains(&1004),
        "the exit left the alternate screen and released every mode: {m:?}"
    );
    // the job was the helper's child, not ours: a wait here has no child
    let mut status = 0;
    // SAFETY: probing a pid that was never this process's child.
    assert_eq!(
        unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
        -1
    );
}
