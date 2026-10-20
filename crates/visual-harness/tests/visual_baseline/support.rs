use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tuiscotti::grouped::{ArtifactSet, GroupedStore};
use tuiscotti::locate::{Locator, Span};
use tuiscotti::render::{self, Renderer};
use tuiscotti::tui::{CancelToken, MouseMods, Tui, WaitError, Wheel};
use tuiscotti::{Frame, Observation, Profile, VENDORED_FACES};

pub use tuiscotti::MouseButton;

pub const SETTLE: Duration = Duration::from_millis(400);

pub const TIMEOUT_MS: u64 = 8_000;

pub const CANONICAL_SIZES: [(u16, u16); 5] = [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)];
pub const CANONICAL_COLORS: [Color; 5] = [
    Color::Truecolor,
    Color::Ansi256,
    Color::Ansi16,
    Color::None,
    Color::NoColorEnv,
];

pub fn is_macos_platform_metadata(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(".DS_Store"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Truecolor,
    Ansi256,
    Ansi16,
    None,
    NoColorEnv,
}

impl Color {
    pub fn suffix(self) -> &'static str {
        match self {
            Color::Truecolor => "truecolor",
            Color::Ansi256 => "256",
            Color::Ansi16 => "16",
            Color::None => "none",
            Color::NoColorEnv => "nocolor",
        }
    }
}

pub struct Case {
    pub name: Cow<'static, str>,
    pub bin: &'static str,

    pub args: &'static [&'static str],
    pub cols: u16,
    pub rows: u16,
    pub color: Color,

    pub needle: &'static str,

    pub sends: &'static [&'static str],
    pub timeout_ms: u64,

    pub env: &'static [(&'static str, &'static str)],
}

impl Case {
    pub fn new(
        name: &'static str,
        bin: &'static str,
        args: &'static [&'static str],
        cols: u16,
        rows: u16,
        color: Color,
        needle: &'static str,
    ) -> Self {
        Self {
            name: Cow::Borrowed(name),
            bin,
            args,
            cols,
            rows,
            color,
            needle,
            sends: &[],
            timeout_ms: TIMEOUT_MS,
            env: &[],
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

pub const ARTIFACTS: ArtifactSet = ArtifactSet::AnsiCursor;

pub const COMMITTED: &[&str] = &["ansi"];

pub const LEGACY_EXTS: [&str; 4] = ["ansi", "txt", "png", "html"];

pub fn is_reapproved(name: &str) -> bool {
    let root = approved_root();
    LEGACY_EXTS.iter().all(|ext| {
        let committed = COMMITTED.contains(ext);
        root.join(format!("{name}.{ext}")).exists() == committed
    })
}

pub fn visual_full() -> bool {
    std::env::var_os("VISUAL_FULL").is_some_and(|v| v == "1")
}

pub fn review_enabled() -> bool {
    std::env::var_os("VISUAL_REVIEW").is_none_or(|v| v != "0")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|dir| dir.parent())
        .expect("crates/visual-harness is two directories under the workspace")
        .to_path_buf()
}

pub fn manifest_path(rel: &str) -> PathBuf {
    workspace_root().join(rel)
}

pub fn approved_root() -> PathBuf {
    if crate::oracle::active() {
        crate::oracle::root()
    } else {
        snapshots_root()
    }
}

pub fn snapshots_root() -> PathBuf {
    manifest_path("snapshots")
}

pub fn actual_root() -> PathBuf {
    manifest_path("target/tuiscotti/actual")
}

pub fn diff_root() -> PathBuf {
    manifest_path("target/tuiscotti/diff")
}

pub fn store() -> GroupedStore {
    GroupedStore::new(&approved_root())
        .with_artifacts(ARTIFACTS)
        .with_actual_root(&actual_root())
        .with_diff_root(&diff_root())
        .with_review_evidence(review_enabled())
}

const ACTUAL_EXTS: [&str; 8] = [
    "ansi",
    "manifest.json",
    "verdict.json",
    "frame.json",
    "txt",
    "png",
    "png.fidelity.json",
    "html",
];

pub fn clear_actual(name: &str) {
    for ext in ACTUAL_EXTS {
        let path = actual_root().join(format!("{name}.{ext}"));
        if let Err(e) = std::fs::remove_file(&path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            panic!("remove {}: {e}", path.display());
        }
    }
}

pub fn clear_actual_leaves_nothing() {
    let name = format!("probe/clear-check-{}/1x1/truecolor", std::process::id());
    for ext in ACTUAL_EXTS {
        let path = actual_root().join(format!("{name}.{ext}"));
        std::fs::create_dir_all(path.parent().expect("nested name"))
            .unwrap_or_else(|e| panic!("mkdir for {}: {e}", path.display()));
        std::fs::write(&path, b"stale").unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }
    clear_actual(&name);
    clear_actual(&name);
    for ext in ACTUAL_EXTS {
        let path = actual_root().join(format!("{name}.{ext}"));
        assert!(!path.exists(), "{} survived clear_actual", path.display());
    }
    let probe = actual_root().join(format!("probe/clear-check-{}", std::process::id()));
    std::fs::remove_dir_all(&probe).unwrap_or_else(|e| panic!("remove {}: {e}", probe.display()));
}

thread_local! {


    static RENDERER: RefCell<Renderer> = RefCell::new(
        Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse"),
    );
}

fn profile_name() -> String {
    Profile::default_profile().name
}

pub fn assert_gated(name: &str, frame: &Frame) {
    let outcome = RENDERER
        .with(|r| store().check_with(&mut r.borrow_mut(), name, frame, 1.0))
        .unwrap_or_else(|e| panic!("baseline `{name}` could not be checked: {e}"));
    match outcome.ensure_matched() {
        Ok(()) => eprintln!("baseline matched   {name}"),
        Err(e) => panic!(
            "baseline `{name}` failed: {e}\n  actual: {}",
            outcome.actual.ansi.display()
        ),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Scroll {
    Up,
    Down,
}

pub struct Screen<'a>(&'a tuiscotti::Screen);

impl Screen<'_> {
    pub fn text(&self) -> String {
        render::frame_from_screen(self.0, "query").text()
    }

    pub fn size(&self) -> (u16, u16) {
        (self.0.cols(), self.0.rows())
    }
}

pub struct Session {
    inner: tuiscotti::Session,
    timeout: Duration,

    sent_at: Option<u64>,
}

fn wait_failure(what: &str, err: &WaitError) -> String {
    format!("{what}: {err}")
}

impl Session {
    fn wait(&self, pred: impl Fn(&Observation) -> bool) -> Result<Observation, WaitError> {
        self.inner
            .wait_predicate(pred, Instant::now() + self.timeout, &CancelToken::new())
    }

    pub fn wait_until(&mut self, pred: impl Fn(&Screen<'_>) -> bool) -> Result<(), String> {
        self.wait(|obs| pred(&Screen(&obs.screen)))
            .map(|_| ())
            .map_err(|e| wait_failure("wait_until", &e))
    }

    fn expect_visible(&self, locator: Locator, what: &str) -> Result<Span, String> {
        self.inner
            .get_by(locator)
            .any()
            .expect_visible_within(self.timeout)
            .map_err(|e| {
                let screen = self
                    .inner
                    .observe_now()
                    .map(|obs| render::frame_from_screen(&obs.screen, "evidence").text())
                    .unwrap_or_else(|e| format!("<no observation: {e}>"));
                format!("{what}: {e}\n--- last screen ---\n{screen}")
            })
    }

    pub fn find(&mut self, needle: &str) -> Result<(u16, u16), String> {
        let span = self.expect_visible(
            Locator::text(needle).physical_rows(),
            &format!("`{needle}` never appeared"),
        )?;
        Ok((span.y, span.x))
    }

    pub fn wait_for_text(&mut self, needle: &str) -> Result<(), String> {
        self.find(needle).map(|_| ())
    }

    pub fn assert_wait_not_vacuous(&self, needle: &str) {
        let obs = self.inner.observe_now().expect("observe live session");
        let screen = render::frame_from_screen(&obs.screen, "evidence").text();
        assert!(
            !screen.lines().any(|line| line.contains(needle)),
            "vacuous `wait:{needle}`: it is already on screen before the inputs it \
             should prove; wait for text only the new state shows\n\
             --- screen ---\n{screen}"
        );
    }

    fn sending(&mut self) {
        self.sent_at = Some(self.inner.revision());
    }

    pub fn wait_answer(&mut self) -> Result<(), String> {
        let Some(before) = self.sent_at.take() else {
            return Ok(());
        };
        self.wait(|obs| obs.revision > before)
            .map(|_| ())
            .map_err(|e| wait_failure("the screen never changed after the last input", &e))
    }

    pub fn wait_idle(&mut self, quiet: Duration) -> Result<(), String> {
        let painted = Locator::regex(r"\S").expect("`\\S` is a valid locator regex");
        self.expect_visible(painted, "the screen stayed blank")?;
        self.inner
            .wait_stable_quiet(Instant::now() + self.timeout, quiet, &CancelToken::new())
            .map(|_| ())
            .map_err(|e| wait_failure("idle", &e))
    }

    pub fn wait_stable(&mut self, quiet: Duration) -> Result<Frame, String> {
        self.wait_answer()?;
        let obs = self
            .inner
            .wait_stable_quiet(Instant::now() + self.timeout, quiet, &CancelToken::new())
            .map_err(|e| wait_failure("settle", &e))?;
        Ok(render::frame_from_screen(&obs.screen, &profile_name()))
    }

    pub fn send_key(&mut self, name: &str) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.press(name)
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.send_text(text)
    }

    pub fn advance_clock(&mut self, ms: u64) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.paste(&format!("junie-tui:advance-clock:{ms}"))
    }

    pub fn mouse_move(&mut self, col: u16, row: u16) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.mouse_move(col, row, MouseMods::NONE)
    }

    pub fn scroll(&mut self, col: u16, row: u16, dir: Scroll) -> Result<(), tuiscotti::TuiError> {
        let wheel = match dir {
            Scroll::Up => Wheel::Up,
            Scroll::Down => Wheel::Down,
        };
        self.sending();
        self.inner.mouse_wheel(wheel, col, row, MouseMods::NONE)
    }

    pub fn click_with(
        &mut self,
        button: MouseButton,
        col: u16,
        row: u16,
    ) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.click(button, col, row, MouseMods::NONE)
    }

    pub fn mouse_down(
        &mut self,
        button: MouseButton,
        col: u16,
        row: u16,
    ) -> Result<(), tuiscotti::TuiError> {
        self.sending();
        self.inner.mouse_down(button, col, row, MouseMods::NONE)
    }

    pub fn drag(&mut self, c0: u16, r0: u16, c1: u16, r1: u16) -> Result<(), tuiscotti::TuiError> {
        let left = MouseButton::Left;
        self.sending();
        self.inner.mouse_down(left, c0, r0, MouseMods::NONE)?;
        let steps = c0.abs_diff(c1).max(r0.abs_diff(r1)).max(1);
        for step in 1..=steps {
            let lerp = |a: u16, b: u16| {
                let (a, b, s, n) = (
                    i32::from(a),
                    i32::from(b),
                    i32::from(step),
                    i32::from(steps),
                );
                u16::try_from(a + (b - a) * s / n).expect("cell on screen")
            };
            self.inner
                .mouse_drag(left, lerp(c0, c1), lerp(r0, r1), MouseMods::NONE)?;
        }
        self.inner.mouse_up(left, c1, r1, MouseMods::NONE)
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), tuiscotti::TuiError> {
        self.inner.resize(cols, rows)
    }

    pub fn modes(&self) -> Vec<u16> {
        let obs = self.inner.observe_now().expect("observe live session");
        obs.state
            .modes
            .known()
            .cloned()
            .expect("tuiscotti reports terminal modes on the live path")
    }
}

pub fn scenario_home(name: &str) -> PathBuf {
    manifest_path("target/tuiscotti/home").join(name)
}

pub const PINNED_ENV: [(&str, &str); 9] = [
    ("SHELL", "/bin/sh"),
    ("JUNIE_TUI_CLIENT_PLATFORM", "macos"),
    ("TERM", "xterm-256color"),
    ("COLORTERM", "truecolor"),
    ("LANG", "C.UTF-8"),
    ("LC_ALL", "C.UTF-8"),
    ("TZ", "UTC"),
    ("PATH", "/usr/bin:/bin:/usr/sbin:/sbin"),
    ("HOLLA_NO_HISTORY", "1"),
];

pub fn spawn(case: &Case) -> Session {
    spawn_with_env(case, &[])
}

pub fn spawn_with_env(case: &Case, extra: &[(&str, &str)]) -> Session {
    let home = scenario_home(&case.name);
    match std::fs::remove_dir_all(&home) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => panic!("empty {}: {e}", home.display()),
    }
    std::fs::create_dir_all(&home).unwrap_or_else(|e| panic!("mkdir {}: {e}", home.display()));
    let mut tui = Tui::new(argv_for(case))
        .size(case.cols, case.rows)
        .env_clear(true)
        .env("HOME", &home)
        .cwd(home.clone());
    for (key, value) in PINNED_ENV {
        tui = tui.env(key, value);
    }
    if matches!(case.color, Color::NoColorEnv) {
        tui = tui.env("NO_COLOR", "1");
    }

    if case.sends.iter().any(|s| s.starts_with("advance:")) {
        tui = tui.env("JUNIE_TUI_TEST_CLOCK", "1");
    }
    for (key, value) in case.env.iter().chain(extra) {
        tui = tui.env(key, value);
    }
    let inner = tui
        .spawn()
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name));
    Session {
        inner,
        timeout: Duration::from_millis(case.timeout_ms),
        sent_at: None,
    }
}

pub fn capture_enabled(name: &str) -> bool {
    if is_reapproved(name) || visual_full() {
        return true;
    }
    eprintln!("baseline skipped   {name} (pending tuiscotti re-approval; VISUAL_FULL=1 runs it)");
    false
}

pub fn spawn_boot(case: &Case) -> Session {
    let mut session = spawn(case);
    boot(&mut session, case.needle);
    if !case.sends.is_empty() {
        drive(&mut session, case.sends);
    }
    session
}

pub fn boot(session: &mut Session, needle: &str) {
    if !needle.is_empty() {
        session
            .wait_for_text(needle)
            .unwrap_or_else(|e| panic!("boot needle `{needle}` never appeared: {e}"));
        return;
    }
    session
        .wait_idle(Duration::from_millis(200))
        .unwrap_or_else(|e| panic!("boot idle failed: {e}"));
}

pub fn drive(session: &mut Session, steps: &[&str]) {
    let mut segment_start = true;
    for (i, step) in steps.iter().enumerate() {
        if step.starts_with("wait:") {
            segment_start = true;
        } else if segment_start && !step.starts_with("sleep:") {
            segment_start = false;
            if let Some(needle) = steps[i..].iter().find_map(|s| s.strip_prefix("wait:")) {
                session.assert_wait_not_vacuous(needle);
            }
        }
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
            session
                .wait_for_text(needle)
                .unwrap_or_else(|e| panic!("`wait:{needle}` timed out: {e}"));
        } else if let Some(ms) = step.strip_prefix("advance:") {
            session
                .advance_clock(ms.parse().expect("advance:<ms>"))
                .expect("advance_clock");
            std::thread::sleep(Duration::from_millis(120));
        } else if let Some(text) = step.strip_prefix("type:") {
            session.type_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            session
                .send_key(step)
                .unwrap_or_else(|e| panic!("send_key `{step}`: {e}"));
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

pub fn settle_and_gate(session: &mut Session, name: &str) {
    let frame = session
        .wait_stable(SETTLE)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e}"));
    assert_gated(name, &frame);
}
