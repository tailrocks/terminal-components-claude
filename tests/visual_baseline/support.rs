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
//! `COLORTERM=truecolor`, `LANG=C.UTF-8`, `TZ=UTC`, a fixed `HOME` under
//! `target/tuiscotti/home`, a minimal `PATH`, `HOLLA_NO_HISTORY=1`, and the
//! crate root as cwd. `nocolor` captures add `NO_COLOR=1` instead of a
//! `--color` flag (backend rule, not app rule). Clearing the environment
//! also keeps ambient `NO_COLOR`, `*_NO_MOTION`, `CLICOLOR_FORCE` and
//! `FORCE_COLOR` out, which used to need explicit stripping.
//!
//! Re-approval status: the committed store was produced by the retired
//! tuisnap engine. Only scenarios already re-approved ([`is_reapproved`]) gate on the
//! new engine; every other capture is skipped unless `VISUAL_FULL=1` (see
//! `docs/visual-testing/tuiscotti-migration.md`).

use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::ansi_cursor;
use tuiscotti::diff;
use tuiscotti::locate::Locator;
use tuiscotti::render::{self, Renderer};
use tuiscotti::tui::{CancelToken, Key, KeyMods, MouseMods, Tui, WaitError, Wheel};
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
    /// `sleep:<ms>`, `wait:<needle>`.
    pub sends: &'static [&'static str],
    pub timeout_ms: u64,
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

/// One committable/gateable artifact kind of a scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "the committed set is switchable; unused kinds stay available"
)]
pub enum Artifact {
    /// tuiscotti `ansi_dump` + cursor trailer ([`ansi_cursor::encode`]):
    /// every cell (text, colours, attributes), grid size and the cursor.
    Ansi,
    /// Plain text rows (`Frame::text`).
    Txt,
    /// Rasterized frame (vendored fonts): decoded-pixel gate.
    Png,
    /// Standalone HTML view (PNG + frame JSON): byte gate.
    Html,
}

impl Artifact {
    pub const fn ext(self) -> &'static str {
        match self {
            Artifact::Ansi => "ansi",
            Artifact::Txt => "txt",
            Artifact::Png => "png",
            Artifact::Html => "html",
        }
    }
}

/// THE switch: which artifacts are committed under `snapshots/` and gated.
/// Decision (docs/visual-testing/tuiscotti-migration.md): one `.ansi` with a
/// cursor trailer per scenario. `.png`/`.html` are review evidence only,
/// written under `target/tuiscotti/` on a mismatch. Changing this list
/// changes the gate, `accept_actuals` and `store_integrity` together.
pub const COMMITTED: &[Artifact] = &[Artifact::Ansi];

/// Extensions the retired tuisnap engine committed for every scenario; the
/// not-yet-re-approved part of the store still carries exactly these.
pub const LEGACY_EXTS: [&str; 4] = ["ansi", "txt", "png", "html"];

/// A scenario is re-approved on tuiscotti when its approved files are
/// exactly the [`COMMITTED`] set (the retired tuisnap engine left all four
/// [`LEGACY_EXTS`]). Re-approved scenarios gate (with `--run-ignored`);
/// every other one is pending re-approval and runs only with
/// `VISUAL_FULL=1`. Re-approving a group is therefore a pure `snapshots/`
/// change made by `accept_actuals`; no code list has to follow it.
pub fn is_reapproved(name: &str) -> bool {
    let root = approved_root();
    LEGACY_EXTS.iter().all(|ext| {
        let committed = COMMITTED.iter().any(|a| a.ext() == *ext);
        root.join(format!("{name}.{ext}")).exists() == committed
    })
}

/// `VISUAL_FULL=1` opts into captures still pending re-approval.
pub fn visual_full() -> bool {
    std::env::var_os("VISUAL_FULL").is_some_and(|v| v == "1")
}

pub fn manifest_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

pub fn approved_root() -> PathBuf {
    manifest_path("snapshots")
}

pub fn actual_root() -> PathBuf {
    manifest_path("target/tuiscotti/actual")
}

pub fn expected_root() -> PathBuf {
    manifest_path("target/tuiscotti/expected")
}

pub fn diff_root() -> PathBuf {
    manifest_path("target/tuiscotti/diff")
}

fn artifact_path(root: &Path, name: &str, ext: &str) -> PathBuf {
    root.join(format!("{name}.{ext}"))
}

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    }
    std::fs::write(path, bytes).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

thread_local! {
    /// One renderer per test thread: font faces parsed once, glyph rasters
    /// cached across every render on the thread (no locks).
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

/// `(png, html)` for `frame`, through the thread's cached renderer.
fn render_png_html(frame: &Frame, title: &str) -> (Vec<u8>, Vec<u8>) {
    let artifacts = RENDERER
        .with(|r| r.borrow_mut().render_artifacts(frame, title))
        .unwrap_or_else(|e| panic!("render `{title}`: {e}"));
    (artifacts.png, artifacts.html.into_bytes())
}

/// Gate one frame against the [`COMMITTED`] approved artifacts.
///
/// Always writes `target/tuiscotti/actual/<name>.ansi` and `.frame.json`.
/// On a mismatch it also writes review evidence: actual `.png`/`.html`,
/// expected `.png`/`.html` rebuilt from the approved `.ansi`
/// (`target/tuiscotti/expected/`), and a red-overlay diff PNG
/// (`target/tuiscotti/diff/`). Fail-closed: a missing approved artifact
/// fails; approval is never implicit (`accept_actuals` in `main.rs`).
pub fn gate(name: &str, frame: &Frame) -> Result<(), String> {
    tuiscotti::grouped::validate_name(name).map_err(|e| e.to_string())?;
    frame
        .validate()
        .map_err(|e| format!("invalid frame `{name}`: {e}"))?;
    let actual = actual_root();
    let approved = approved_root();
    let ansi = ansi_cursor::encode(frame);
    write_file(&artifact_path(&actual, name, "ansi"), ansi.as_bytes());
    write_file(
        &artifact_path(&actual, name, "frame.json"),
        frame.to_json().as_bytes(),
    );
    let mut rendered: Option<(Vec<u8>, Vec<u8>)> = None;
    let render = |rendered: &mut Option<(Vec<u8>, Vec<u8>)>| -> (Vec<u8>, Vec<u8>) {
        rendered
            .get_or_insert_with(|| {
                let (png, html) = render_png_html(frame, name);
                write_file(&artifact_path(&actual, name, "png"), &png);
                write_file(&artifact_path(&actual, name, "html"), &html);
                (png, html)
            })
            .clone()
    };

    let mut problems = Vec::new();
    for &artifact in COMMITTED {
        let path = artifact_path(&approved, name, artifact.ext());
        let Ok(expected) = std::fs::read(&path) else {
            problems.push(format!("missing approval {}", path.display()));
            continue;
        };
        let differs = match artifact {
            Artifact::Ansi => expected != ansi.as_bytes(),
            Artifact::Txt => expected != frame.text().as_bytes(),
            Artifact::Html => expected != render(&mut rendered).1,
            Artifact::Png => {
                let verdict = diff::compare_png(&expected, &render(&mut rendered).0)
                    .map_err(|e| format!("png compare `{name}`: {e}"))?;
                !verdict.dims_equal || verdict.score < 1.0
            }
        };
        if differs {
            problems.push(format!(
                "{} differs from {}",
                artifact.ext(),
                path.display()
            ));
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    let (actual_png, _) = render(&mut rendered);
    let evidence = write_expected_evidence(name, &actual_png);
    Err(format!(
        "baseline `{name}` failed: {}\n  actual: {}{evidence}",
        problems.join("; "),
        artifact_path(&actual, name, "{ansi,png,html}").display(),
    ))
}

/// Rebuild expected PNG/HTML from the approved `.ansi` and write the diff
/// PNG. Returns a note for the failure message (never fails the caller).
fn write_expected_evidence(name: &str, actual_png: &[u8]) -> String {
    let approved = artifact_path(&approved_root(), name, "ansi");
    let Ok(text) = std::fs::read_to_string(&approved) else {
        return String::new();
    };
    let provenance =
        render::frame_from_screen(&tuiscotti::Screen::blank(1, 1), &profile_name()).provenance;
    let Some(expected) = ansi_cursor::decode(&text, provenance) else {
        return format!("\n  expected: {} is not decodable", approved.display());
    };
    let (png, html) = render_png_html(&expected, name);
    write_file(&artifact_path(&expected_root(), name, "png"), &png);
    write_file(&artifact_path(&expected_root(), name, "html"), &html);
    let mut note = format!(
        "\n  expected: {}",
        artifact_path(&expected_root(), name, "{png,html}").display()
    );
    if let Ok(verdict) = diff::compare_png(&png, actual_png)
        && verdict.dims_equal
        && verdict.score < 1.0
    {
        let path = artifact_path(&diff_root(), name, "png");
        write_file(&path, &verdict.diff_png);
        note.push_str(&format!(
            "\n  diff: {} (pixel score {:.4})",
            path.display(),
            verdict.score
        ));
    }
    note
}

/// Render the approved `.ansi` of `name` to
/// `target/tuiscotti/render/<name>.{png,html}` (`cargo xtask visual render`).
pub fn render_approved(name: &str) -> Result<(PathBuf, PathBuf), String> {
    let approved = artifact_path(&approved_root(), name, "ansi");
    let text = std::fs::read_to_string(&approved)
        .map_err(|e| format!("read {}: {e}", approved.display()))?;
    let provenance =
        render::frame_from_screen(&tuiscotti::Screen::blank(1, 1), &profile_name()).provenance;
    let frame = ansi_cursor::decode(&text, provenance)
        .ok_or_else(|| format!("{} is not decodable", approved.display()))?;
    let (png, html) = render_png_html(&frame, name);
    let root = manifest_path("target/tuiscotti/render");
    let (png_path, html_path) = (
        artifact_path(&root, name, "png"),
        artifact_path(&root, name, "html"),
    );
    write_file(&png_path, &png);
    write_file(&html_path, &html);
    Ok((png_path, html_path))
}

/// Fail-closed assertion: only a full match passes.
pub fn assert_gated(name: &str, frame: &Frame) {
    match gate(name, frame) {
        Ok(()) => eprintln!("baseline matched   {name}"),
        Err(e) => panic!("{e}"),
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
    /// First occurrence of `needle` on one physical row, as `(row, col)`.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        Locator::text(needle)
            .physical_rows()
            .resolve(self.0, 0)
            .ok()?
            .into_iter()
            .min_by_key(|span| (span.y, span.x))
            .map(|span| (span.y, span.x))
    }

    /// Plain text, rows joined with `\n`, trailing blanks trimmed.
    pub fn text(&self) -> String {
        render::frame_from_screen(self.0, "query").text()
    }

    /// `(cols, rows)`.
    pub fn size(&self) -> (u16, u16) {
        (self.0.cols(), self.0.rows())
    }

    fn is_blank(&self) -> bool {
        self.text().trim().is_empty()
    }
}

/// Live PTY session: a thin adapter over `tuiscotti::Session` keeping the
/// suite's vocabulary (key names, `(col, row)` pointer calls, one deadline
/// per wait taken from the case timeout).
pub struct Session {
    inner: tuiscotti::Session,
    timeout: Duration,
}

fn wait_failure(what: &str, err: &WaitError) -> String {
    let evidence = err
        .evidence()
        .map(|obs| render::frame_from_screen(&obs.screen, "evidence").text())
        .unwrap_or_default();
    format!("{what}: {err}\n--- last screen ---\n{evidence}")
}

/// Translate the suite's key names (`ctrl-l`, `alt-0`, `backtab`, `f10`,
/// `pagedown`, single characters) to tuiscotti `+` chords.
fn chord(name: &str) -> String {
    if name == "backtab" {
        return "shift+tab".to_string();
    }
    if name.chars().count() > 1
        && let Some((mods, key)) = name.rsplit_once('-')
        && !key.is_empty()
        && mods
            .split('-')
            .all(|m| matches!(m, "ctrl" | "alt" | "shift"))
    {
        return format!("{}+{key}", mods.replace('-', "+"));
    }
    name.to_string()
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

    /// Content-gated readiness: wait for `needle` (tuiscotti text
    /// `Locator`, physical rows), returning its first `(row, col)`.
    pub fn find(&mut self, needle: &str) -> Result<(u16, u16), String> {
        let locator = Locator::text(needle).physical_rows();
        let obs = self
            .wait(|obs| {
                locator
                    .resolve_obs(obs)
                    .is_ok_and(|spans| !spans.is_empty())
            })
            .map_err(|e| wait_failure(&format!("`{needle}` never appeared"), &e))?;
        Ok(Screen(&obs.screen)
            .find(needle)
            .expect("locator matched on the returned observation"))
    }

    pub fn wait_for_text(&mut self, needle: &str) -> Result<(), String> {
        self.find(needle).map(|_| ())
    }

    /// No needle: wait for a non-blank first paint, then a quiet window.
    pub fn wait_idle(&mut self, quiet: Duration) -> Result<(), String> {
        self.wait_until(|screen| !screen.is_blank())?;
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

    pub fn send_key(&mut self, name: &str) -> Result<(), tuiscotti::TuiError> {
        // A bare character goes straight to `Key::Char`: tuiscotti's chord
        // parser reads a lone `f`/`F` as a malformed function key.
        let mut chars = name.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return self.inner.press_key(Key::Char(c), KeyMods::NONE);
        }
        self.inner.press(&chord(name))
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), tuiscotti::TuiError> {
        self.inner.send_text(text)
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

/// Spawn a case's session with the explicit capture environment.
pub fn spawn(case: &Case) -> Session {
    let home = manifest_path("target/tuiscotti/home");
    std::fs::create_dir_all(&home).unwrap_or_else(|e| panic!("mkdir {}: {e}", home.display()));
    let mut tui = Tui::new(argv_for(case))
        .size(case.cols, case.rows)
        .env_clear(true)
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .env("TZ", "UTC")
        .env("HOME", &home)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOLLA_NO_HISTORY", "1")
        .cwd(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    if matches!(case.color, Color::NoColorEnv) {
        tui = tui.env("NO_COLOR", "1");
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
