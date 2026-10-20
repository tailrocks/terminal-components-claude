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

fn mark(path: &Path, line: &str) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open the marker file");
    writeln!(f, "{line}").expect("write a marker");
    f.flush().expect("flush a marker");
}

fn helper_main() -> ! {
    let bin = std::env::var(HELPER_BIN).expect("application path");
    let markers = PathBuf::from(std::env::var(HELPER_MARKERS).expect("marker file"));

    unsafe {
        libc::setsid();
        libc::ioctl(0, libc::TIOCSCTTY as _, 0);
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }

    let tty = unsafe { libc::ttyname(0) };
    assert!(!tty.is_null(), "the helper's stdin is a terminal");

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

    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    #[allow(clippy::zombie_processes)]
    let child = cmd.spawn().expect("spawn the application");
    let pid = child.id() as c_int;

    unsafe {
        libc::setpgid(pid, pid);
        libc::tcsetpgrp(0, pid);
    }
    mark(&markers, &format!("PID={pid}"));
    loop {
        let mut status: c_int = 0;

        let rc = unsafe { libc::waitpid(pid, &mut status, libc::WUNTRACED) };
        if rc != pid {
            mark(&markers, "EXIT=-1");
            std::process::exit(1);
        }
        if stopped(status) {
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

fn marker_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "junie-terminal-suspend-{}-{nanos}.markers",
        std::process::id()
    ))
}

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

    fn termios(&self) -> libc::termios {
        let tty = self.marker("TTY=").expect("the helper reported its tty");
        let dev: File = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(&tty)
            .unwrap_or_else(|e| panic!("open {tty}: {e}"));

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

        let (cols, rows) = if cycle == 1 { (100, 30) } else { (80, 24) };
        let expected = fresh_frame(cols, rows);
        job.session
            .resize(cols, rows)
            .expect("resize while stopped");
        job.session.send_text("fg\n").expect("fg");
        job.wait_until("raw mode after fg", Job::is_raw);

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

    let mut status = 0;

    assert_eq!(
        unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
        -1
    );
}
