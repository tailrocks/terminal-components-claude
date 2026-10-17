//! Shared driver for the visual-baseline suite (tuiscotti main, Workflow 2).
//!
//! One PTY per capture: `tuiscotti::tui::Tui::new([bin, args…])` with an
//! explicit environment → boot needle (content-gated `Locator` wait) or a
//! non-blank + quiet wait when a case has none → send steps (120 ms pacing)
//! → `wait_stable_quiet(SETTLE)` → `render::frame_from_screen` → gate.
//! [`spawn_boot`]/[`drive`] hand the live [`Session`] to `scenarios::interact`
//! for pointer, dismissal and resize steps; [`settle_and_gate`] captures.
//!
//! Store layout is unchanged:
//! `snapshots/<app>/<sub_group>/<cols>x<rows>/<palette>.<ext>`. Which
//! extensions are committed and gated is decided in ONE place,
//! [`COMMITTED`]. Actuals, diffs and the frame JSON sidecar are scratch
//! under `target/tuiscotti/` (gitignored). The capture name is the grouped
//! path, e.g. `holla/parity/discovery/120x40/truecolor`.
//!
//! Environment per capture (never inherited, `env_clear(true)`): `TERM`,
//! `COLORTERM=truecolor`, `LANG=C.UTF-8`, `TZ=UTC`, a minimal `PATH`,
//! `HOLLA_NO_HISTORY=1`, and a per-scenario `HOME` that is also the cwd
//! ([`scenario_home`]: `target/tuiscotti/home/<name>`, emptied before each
//! spawn). `nocolor` captures add `NO_COLOR=1` instead of a `--color` flag
//! (backend rule, not app rule). Clearing the environment also keeps
//! ambient `NO_COLOR`, `*_NO_MOTION`, `CLICOLOR_FORCE` and `FORCE_COLOR`
//! out, which used to need explicit stripping; the `env_isolation` check
//! proves it.
//!
//! Re-approval status: the committed store was produced by the retired
//! tuisnap engine. Only scenarios already re-approved ([`is_reapproved`]) gate on the
//! new engine; every other capture is skipped unless `VISUAL_FULL=1` (see
//! `docs/visual-testing/tuiscotti-migration.md`).

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

/// SETTLE_MS from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// TIMEOUT_MS default; boot-streaming screens override it per capture
/// (`Root::timeout`, the bash `CAP_TIMEOUT=` prefix).
pub const TIMEOUT_MS: u64 = 8_000;

/// Canonical matrix axes: every canonical capture root and audit fixture is
/// captured at all 5 sizes × 5 colours.
pub const CANONICAL_SIZES: [(u16, u16); 5] = [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)];
pub const CANONICAL_COLORS: [Color; 5] = [
    Color::Truecolor,
    Color::Ansi256,
    Color::Ansi16,
    Color::None,
    Color::NoColorEnv,
];

/// Exact-name filter for macOS Finder metadata. Not snapshot content; every
/// other unknown file remains a store-integrity failure.
pub fn is_macos_platform_metadata(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(".DS_Store"))
}

/// `--color` palette, or real `NO_COLOR=1` (the baseline's `nocolor`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Truecolor,
    Ansi256,
    Ansi16,
    None,
    NoColorEnv,
}

impl Color {
    /// Leaf suffix in capture names (`no_color` is spelled `nocolor`).
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

/// One capture definition — the typed form of one bash `cap` call.
pub struct Case {
    /// Grouped store name (`<app>/<sub_group>/<surface>_<state>_<cols>x<rows>_<color>`);
    /// owned when built by the data-driven matrices.
    pub name: Cow<'static, str>,
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
    /// `sleep:<ms>`, `wait:<needle>`, `advance:<ms>` (an explicit clock
    /// step of a paused app, [`Session::advance_clock`]).
    pub sends: &'static [&'static str],
    pub timeout_ms: u64,
    /// Variables set over [`PINNED_ENV`] for this capture only: a scenario
    /// that opts into a state the pinned set turns off (holla's usage
    /// history, `HOLLA_NO_HISTORY=0`). Empty for almost every capture.
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

// ----------------------------------------------------------- artifacts --

/// THE switch: what is committed under the store and gated, as tuiscotti's
/// grouped-store artifact mode. Decision
/// (docs/visual-testing/tuiscotti-migration.md): one `.ansi` with a VT
/// cursor trailer per scenario (`render::ansi_dump_with_cursor`).
/// `.png`/`.html` and the diff PNG are review evidence only, rendered by
/// the store under `target/tuiscotti/` on a mismatch.
pub const ARTIFACTS: ArtifactSet = ArtifactSet::AnsiCursor;

/// The extensions [`ARTIFACTS`] commits per scenario; `store_integrity`
/// checks the store holds exactly these for every re-approved scenario.
pub const COMMITTED: &[&str] = &["ansi"];

/// Extensions the retired tuisnap engine committed for every scenario; the
/// not-yet-re-approved part of the store still carries exactly these.
pub const LEGACY_EXTS: [&str; 4] = ["ansi", "txt", "png", "html"];

/// A scenario is re-approved on tuiscotti when its approved files are
/// exactly the [`COMMITTED`] set (the retired tuisnap engine left all four
/// [`LEGACY_EXTS`]). Re-approved scenarios gate (with `--run-ignored`);
/// every other one is pending re-approval and runs only with
/// `VISUAL_FULL=1`. Re-approving a group is therefore a pure `snapshots/`
/// change made by `tuiscotti accept`; no code list has to follow it.
pub fn is_reapproved(name: &str) -> bool {
    let root = approved_root();
    LEGACY_EXTS.iter().all(|ext| {
        let committed = COMMITTED.contains(ext);
        root.join(format!("{name}.{ext}")).exists() == committed
    })
}

/// `VISUAL_FULL=1` opts into captures still pending re-approval.
pub fn visual_full() -> bool {
    std::env::var_os("VISUAL_FULL").is_some_and(|v| v == "1")
}

/// `VISUAL_REVIEW=0` skips the review PNG/HTML/diff on a mismatch (seconds
/// per capture; the store's `with_review_evidence(false)`). The gate
/// verdict and the sealed actual `.ansi` are unchanged; `cargo xtask
/// visual determinism` sets it.
pub fn review_enabled() -> bool {
    std::env::var_os("VISUAL_REVIEW").is_none_or(|v| v != "0")
}

pub fn manifest_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// The store the gate reads: `baselines/oracle/` once a capture was
/// imported into it (`ORACLE.toml` `pending = false`, see
/// [`crate::oracle`]), else the legacy [`snapshots_root`].
pub fn approved_root() -> PathBuf {
    if crate::oracle::active() {
        crate::oracle::root()
    } else {
        snapshots_root()
    }
}

/// The legacy store (tuisnap approvals and the re-approved pilot).
pub fn snapshots_root() -> PathBuf {
    manifest_path("snapshots")
}

pub fn actual_root() -> PathBuf {
    manifest_path("target/tuiscotti/actual")
}

pub fn diff_root() -> PathBuf {
    manifest_path("target/tuiscotti/diff")
}

/// The grouped store the gate checks against: [`ARTIFACTS`] mode over
/// [`approved_root`], scratch under `target/tuiscotti/{actual,expected,diff}`
/// (`expected` follows `actual`). tuiscotti seals every candidate
/// (`<name>.manifest.json`, `<name>.verdict.json`, stamped with the
/// nextest run id) so `tuiscotti accept` takes only reviewed actuals of one
/// run, and refuses under `CI`.
pub fn store() -> GroupedStore {
    GroupedStore::new(&approved_root())
        .with_artifacts(ARTIFACTS)
        .with_actual_root(&actual_root())
        .with_diff_root(&diff_root())
        .with_review_evidence(review_enabled())
}

/// Everything a capture can leave under [`actual_root`] for one scenario
/// (the store's candidate, its seals and the mismatch-only evidence).
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

/// Remove what an earlier run left under [`actual_root`] for `name`, so a
/// run that skips the scenario or fails before its capture leaves no actual
/// behind for `tuiscotti accept` to pick up.
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

/// Default-run check: [`clear_actual`] removes every file a capture can
/// leave, and a missing one is not an error.
pub fn clear_actual_leaves_nothing() {
    let name = format!("probe/clear-check-{}/1x1/truecolor", std::process::id());
    for ext in ACTUAL_EXTS {
        let path = actual_root().join(format!("{name}.{ext}"));
        std::fs::create_dir_all(path.parent().expect("nested name"))
            .unwrap_or_else(|e| panic!("mkdir for {}: {e}", path.display()));
        std::fs::write(&path, b"stale").unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }
    clear_actual(&name);
    clear_actual(&name); // nothing left is not an error
    for ext in ACTUAL_EXTS {
        let path = actual_root().join(format!("{name}.{ext}"));
        assert!(!path.exists(), "{} survived clear_actual", path.display());
    }
    let probe = actual_root().join(format!("probe/clear-check-{}", std::process::id()));
    std::fs::remove_dir_all(&probe).unwrap_or_else(|e| panic!("remove {}: {e}", probe.display()));
}

thread_local! {
    /// One renderer per test thread: font faces parsed once, glyph rasters
    /// cached across every review render on the thread (no locks).
    static RENDERER: RefCell<Renderer> = RefCell::new(
        Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse"),
    );
}

/// Render profile name recorded in each frame's provenance.
fn profile_name() -> String {
    Profile::default_profile().name
}

/// Fail-closed gate of one frame through [`store`]: only a full match
/// passes, a missing approval fails, and approval is never implicit
/// (`tuiscotti accept`, local only). On a mismatch the store writes the
/// actual and expected `.png`/`.html` and a diff PNG under
/// `target/tuiscotti/`.
pub fn assert_gated(name: &str, frame: &Frame) {
    // 1.0: any pixel difference counts (only review evidence renders
    // pixels in `AnsiCursor` mode; the `.ansi` bytes decide).
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

// ------------------------------------------------------------- session --

/// Wheel direction for [`Session::scroll`].
#[derive(Clone, Copy, Debug)]
pub enum Scroll {
    Up,
    Down,
}

/// A live screen with the queries the suite needs.
pub struct Screen<'a>(&'a tuiscotti::Screen);

impl Screen<'_> {
    /// Plain text, rows joined with `\n`, trailing blanks trimmed.
    pub fn text(&self) -> String {
        render::frame_from_screen(self.0, "query").text()
    }

    /// `(cols, rows)`.
    pub fn size(&self) -> (u16, u16) {
        (self.0.cols(), self.0.rows())
    }
}

/// Live PTY session: a thin adapter over `tuiscotti::Session` keeping the
/// suite's vocabulary (key names, `(col, row)` pointer calls, one deadline
/// per wait taken from the case timeout).
pub struct Session {
    inner: tuiscotti::Session,
    timeout: Duration,
}

/// tuiscotti's `WaitError` already ends with the last screen.
fn wait_failure(what: &str, err: &WaitError) -> String {
    format!("{what}: {err}")
}

impl Session {
    fn wait(&self, pred: impl Fn(&Observation) -> bool) -> Result<Observation, WaitError> {
        self.inner
            .wait_predicate(pred, Instant::now() + self.timeout, &CancelToken::new())
    }

    /// Wait until `pred` holds on the live screen.
    pub fn wait_until(&mut self, pred: impl Fn(&Screen<'_>) -> bool) -> Result<(), String> {
        self.wait(|obs| pred(&Screen(&obs.screen)))
            .map(|_| ())
            .map_err(|e| wait_failure("wait_until", &e))
    }

    /// Wait (case timeout) until `locator` has a visible match, returning
    /// the first in row-major order. A timeout carries the last screen.
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

    /// Content-gated readiness: wait for `needle` (tuiscotti text locator,
    /// physical rows), returning its first `(row, col)`.
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

    /// No needle: wait for a non-blank first paint (any non-whitespace
    /// cell), then a quiet window.
    pub fn wait_idle(&mut self, quiet: Duration) -> Result<(), String> {
        let painted = Locator::regex(r"\S").expect("`\\S` is a valid locator regex");
        self.expect_visible(painted, "the screen stayed blank")?;
        self.inner
            .wait_stable_quiet(Instant::now() + self.timeout, quiet, &CancelToken::new())
            .map(|_| ())
            .map_err(|e| wait_failure("idle", &e))
    }

    /// Settle (no new revision for `quiet`) and adapt the screen to a frame.
    pub fn wait_stable(&mut self, quiet: Duration) -> Result<Frame, String> {
        let obs = self
            .inner
            .wait_stable_quiet(Instant::now() + self.timeout, quiet, &CancelToken::new())
            .map_err(|e| wait_failure("settle", &e))?;
        Ok(render::frame_from_screen(&obs.screen, &profile_name()))
    }

    /// Press one key named in the suite's vocabulary (`ctrl-l`, `alt-0`,
    /// `backtab`, `f10`, `pagedown`, single characters such as `f`), which
    /// is tuiscotti's chord grammar (`tui::parse_chord`).
    pub fn send_key(&mut self, name: &str) -> Result<(), tuiscotti::TuiError> {
        self.inner.press(name)
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), tuiscotti::TuiError> {
        self.inner.send_text(text)
    }

    /// An explicit clock step of `ms` virtual milliseconds (`advance:<ms>`):
    /// a paused app runs its stopped clock for that long, so jobs on
    /// virtual time complete, then stops it again and the frame settles
    /// (`junie_tui::clock::ClockStep`, `shell::App::advance_clock`). It is
    /// sent in band, after the inputs before it.
    pub fn advance_clock(&mut self, ms: u64) -> Result<(), tuiscotti::TuiError> {
        self.inner.paste(&junie_tui::clock::ClockStep::request(ms))
    }

    pub fn mouse_move(&mut self, col: u16, row: u16) -> Result<(), tuiscotti::TuiError> {
        self.inner.mouse_move(col, row, MouseMods::NONE)
    }

    pub fn scroll(&mut self, col: u16, row: u16, dir: Scroll) -> Result<(), tuiscotti::TuiError> {
        let wheel = match dir {
            Scroll::Up => Wheel::Up,
            Scroll::Down => Wheel::Down,
        };
        self.inner.mouse_wheel(wheel, col, row, MouseMods::NONE)
    }

    pub fn click_with(
        &mut self,
        button: MouseButton,
        col: u16,
        row: u16,
    ) -> Result<(), tuiscotti::TuiError> {
        self.inner.click(button, col, row, MouseMods::NONE)
    }

    /// Button down with no release: the pressed state stays visible for as
    /// long as the button is held.
    pub fn mouse_down(
        &mut self,
        button: MouseButton,
        col: u16,
        row: u16,
    ) -> Result<(), tuiscotti::TuiError> {
        self.inner.mouse_down(button, col, row, MouseMods::NONE)
    }

    /// Primary-button drag: press at the start, one held-motion report per
    /// cell crossed (as a terminal sends), release at the end.
    pub fn drag(&mut self, c0: u16, r0: u16, c1: u16, r1: u16) -> Result<(), tuiscotti::TuiError> {
        let left = MouseButton::Left;
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

    /// DEC private modes the application currently has set, from the live
    /// emulator state (not part of any snapshot format).
    pub fn modes(&self) -> Vec<u16> {
        let obs = self.inner.observe_now().expect("observe live session");
        obs.state
            .modes
            .known()
            .cloned()
            .expect("tuiscotti reports terminal modes on the live path")
    }
}

/// The isolated `HOME` and cwd of one capture:
/// `target/tuiscotti/home/<name>` (plan §B3). Names are unique per
/// scenario, so concurrent captures never share a directory.
pub fn scenario_home(name: &str) -> PathBuf {
    manifest_path("target/tuiscotti/home").join(name)
}

/// The variables every spawn sets after `env_clear(true)`, except
/// `HOME` (per scenario) and `NO_COLOR` (only the `nocolor` level).
/// `SHELL` is pinned because the PTY layer otherwise fills it in from the
/// user's account even after `env_clear` (found by `env_isolation`).
/// `JUNIE_TUI_CLIENT_PLATFORM` pins `core::event::ClientPlatform`: with no
/// terminal variables it falls back to the build target, so the
/// Option+click / Ctrl+click hints would differ between macOS and the
/// ubuntu CI runner. macOS is the interaction reference (AGENTS.md).
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

/// Spawn a case's session with the explicit capture environment: nothing
/// is inherited, `HOME` and the cwd are [`scenario_home`], emptied first so
/// nothing an earlier run left there can reach the app.
pub fn spawn(case: &Case) -> Session {
    spawn_with_env(case, &[])
}

/// [`spawn`] with extra variables on top of the capture environment (test
/// instrumentation such as `JUNIE_TUI_FIRST_POLL_DELAY_MS`).
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
    for (key, value) in case.env.iter().chain(extra) {
        tui = tui.env(key, value);
    }
    let inner = tui
        .spawn()
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name));
    Session {
        inner,
        timeout: Duration::from_millis(case.timeout_ms),
    }
}

/// `false` (and a note) when `name` is still pending re-approval on the
/// tuiscotti engine and `VISUAL_FULL=1` is not set.
pub fn capture_enabled(name: &str) -> bool {
    if is_reapproved(name) || visual_full() {
        return true;
    }
    eprintln!("baseline skipped   {name} (pending tuiscotti re-approval; VISUAL_FULL=1 runs it)");
    false
}

/// Central live-session setup: boot needle first, then all case sends,
/// handing the connected session back for pointer or manual-flow steps.
pub fn spawn_boot(case: &Case) -> Session {
    let mut session = spawn(case);
    boot(&mut session, case.needle);
    if !case.sends.is_empty() {
        drive(&mut session, case.sends);
    }
    session
}

/// Boot: needle first (content-gated). Live clocks starve a quiet-window
/// wait, so the quiet fallback is only for cases without a needle.
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

/// The step loop on a live session, 120 ms pacing after each input.
pub fn drive(session: &mut Session, steps: &[&str]) {
    for step in steps {
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

/// Settle the screen and gate the capture.
pub fn settle_and_gate(session: &mut Session, name: &str) {
    let frame = session
        .wait_stable(SETTLE)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e}"));
    assert_gated(name, &frame);
}
