//! Shared driver for the visual-baseline suite.
//!
//! One PTY per capture: spawn → boot needle (or idle if none) → send
//! steps (120 ms pacing) → `wait_stable`(SETTLE) → frame → store gate.
//! [`run_once`] runs the ported matrix; [`boot`]/[`drive`] mirror it for
//! the pointer group, which needs the live session afterwards. A leading
//! `wait:` needle is readiness — live clocks skip the 200 ms quiet window.
//!
//! Store: the grouped multi-artifact store (`tuiscotti::grouped`). Approved
//! frames live at `snapshots/<group>/<sub_group>/<name>.{ansi,txt,png,html}`
//! (committed, exactly four artifacts per scenario); actuals, diffs and the
//! HTML report are scratch under `target/tuiscotti/` (gitignored). The capture
//! name is the grouped path, e.g. `holla/parity/discovery/120x40/truecolor`.
//!
//! Colour hygiene per capture: ambient `NO_COLOR` is stripped (crossterm
//! honours it by *presence* and would silently flatten every colour frame —
//! the lesson that forced the 2026-09-12 baseline re-capture), the motion
//! kill-switches `HOLLA_NO_MOTION`/`JACKIN_NO_MOTION` and the colour-forcing
//! `CLICOLOR_FORCE`/`FORCE_COLOR` are stripped for the same reason, and
//! `HOLLA_NO_HISTORY=1` suppresses history side effects; `nocolor` captures
//! re-add `NO_COLOR=1` instead of a `--color` flag (backend rule, not app
//! rule).

pub mod scoped_targets;
pub mod state_waits;
pub mod typed_input;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};

use tuiscotti::formats::{
    assert_no_escapes, assert_normalized_sgr, assert_opaque_rgb, assert_seven_bit,
    assert_static_offline, capture_all, parse_canonical,
};
use tuiscotti::grouped::{GroupedOutcome, GroupedStore};
use tuiscotti::render::frame_from_screen;
use tuiscotti::snapshot::{CellDiff, MAX_CELL_DIFFS, Status};
pub use tuiscotti::tui::MouseButton;
use tuiscotti::tui::Tui;
use tuiscotti::tui::{CancelToken, MouseMods};
pub use tuiscotti::{Cell, Frame, Mods, Profile, Renderer, Rgb, UnderlineStyle, VENDORED_FACES};

pub const SHOWCASE: &str = "showcase";
pub const TABLEPRO: &str = "tablepro";
pub const JACKIN: &str = "jackin-preview";
pub const HOLLA: &str = "holla";

/// SETTLE_MS from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// TIMEOUT_MS default; boot-streaming screens override it per capture
/// ([`Case::timeout`], the bash `CAP_TIMEOUT=` prefix).
pub const TIMEOUT_MS: u64 = 8_000;
/// Wait bound where no case timeout applies (helper-level waits). Matches
/// the session deadline the old helpers inherited.
pub const DEFAULT_WAIT: Duration = Duration::from_millis(TIMEOUT_MS);

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

/// Prefixes for the 9 `audit_matrix` fixtures (`{prefix}/{cols}x{rows}/{color}`).
pub const AUDIT_PREFIX_HOLLA_RUST: &str = "holla/audit/rust";
pub const AUDIT_PREFIX_HOLLA_UPGRADE: &str = "holla/audit/upgrade";
pub const AUDIT_PREFIX_JACKIN_CAPSULE: &str = "jackin/audit/capsule";
pub const AUDIT_PREFIX_SHOWCASE_BUTTONS: &str = "showcase/audit/buttons";
pub const AUDIT_PREFIX_SHOWCASE_DIFF: &str = "showcase/audit/diff";
pub const AUDIT_PREFIX_SHOWCASE_FORMS: &str = "showcase/audit/forms";
pub const AUDIT_PREFIX_SHOWCASE_INPUTS: &str = "showcase/audit/inputs";
pub const AUDIT_PREFIX_SHOWCASE_TEXTAREAS: &str = "showcase/audit/textareas";
pub const AUDIT_PREFIX_TABLEPRO_PRODUCTION: &str = "tablepro/audit/production";
pub const AUDIT_PREFIX_JACKIN_ACCOUNTS: &str = "jackin/audit/accounts";

#[derive(serde::Deserialize)]
struct RootMapEntry {
    legacy_root: String,
    target_root: String,
}

#[derive(serde::Deserialize)]
struct MigrationMapHeader {
    root_mappings: Vec<RootMapEntry>,
}

static ROOT_MAP: std::sync::OnceLock<std::collections::BTreeMap<String, String>> =
    std::sync::OnceLock::new();

pub fn screen_first_path(p: &str) -> String {
    let map = ROOT_MAP.get_or_init(|| {
        let json_str = include_str!("../../../../docs/verification/snapshot-migration-map.json");
        let header: MigrationMapHeader =
            serde_json::from_str(json_str).expect("parse migration map header");
        header
            .root_mappings
            .into_iter()
            .map(|r| (r.legacy_root, r.target_root))
            .collect()
    });

    let parts: Vec<&str> = p.split('/').collect();
    if parts.len() >= 3 {
        let color = parts[parts.len() - 1];
        let size = parts[parts.len() - 2];
        let root = parts[..parts.len() - 2].join("/");
        if let Some(target_root) = map.get(&root) {
            return format!("{target_root}/{size}/{color}");
        }
    }
    p.to_string()
}

pub fn audit_default_name(prefix: &str, cols: u16, rows: u16, color: Color) -> String {
    format!("{prefix}/{cols}x{rows}/{}", color.suffix())
}

/// Every capture name the suite produces: the dimensions × capabilities
/// expansion of each `LEGACY:` root in the vendored case registry
/// (`tests/conformance/required_cases.json`, copied from the
/// `origin/termrock-refactor` planning history), mapped to screen-first
/// paths. The suite never parses its own sources or the store, so the
/// `store_integrity` store==suite check proves registry coverage.
pub fn suite_capture_names() -> BTreeSet<String> {
    const REGISTRY_JSON: &str = include_str!("../../../../tests/conformance/required_cases.json");
    let manifest: serde_json::Value =
        serde_json::from_str(REGISTRY_JSON).expect("parse required_cases.json");
    assert_eq!(
        manifest["legacy_roots_count"].as_u64(),
        Some(302),
        "registry must declare exactly 302 legacy roots"
    );
    let cases = manifest["cases"].as_array().expect("registry cases array");
    let mut names = BTreeSet::new();
    let mut roots = 0u64;
    for case in cases {
        let id = case["id"].as_str().expect("registry case id");
        let Some(legacy_root) = id.strip_prefix("LEGACY:") else {
            continue;
        };
        roots += 1;
        let dimensions = case["dimensions"]
            .as_array()
            .expect("registry case dimensions");
        let capabilities = case["capabilities"]
            .as_array()
            .expect("registry case capabilities");
        for dim in dimensions {
            let cols = dim[0].as_u64().expect("dimension cols");
            let rows = dim[1].as_u64().expect("dimension rows");
            for capability in capabilities {
                let capability = capability.as_str().expect("capability");
                names.insert(screen_first_path(&format!(
                    "{legacy_root}/{cols}x{rows}/{capability}"
                )));
            }
        }
    }
    assert_eq!(roots, 302, "registry must carry exactly 302 LEGACY cases");
    assert!(
        !names.is_empty(),
        "no capture names expanded from the registry"
    );
    names
}

/// `name` is `<root>/<cols>x<rows>/<color>`; resize roots use the same
/// canonical matrix as every other capture root.
fn canonical_root(name: &str) -> Option<&str> {
    name.rsplit_once('/')
        .and_then(|(without_color, _)| without_color.rsplit_once('/'))
        .map(|(root, _)| root)
}

fn canonical_name(root: &str, cols: u16, rows: u16, color: Color) -> String {
    format!("{root}/{cols}x{rows}/{}", color.suffix())
}

pub fn is_macos_platform_metadata(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(".DS_Store"))
}

#[derive(Clone, Copy)]
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
        let sf_name = screen_first_path(name);
        Self {
            name: Cow::Owned(sf_name),
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

    pub fn dynamic(
        name: String,
        bin: &'static str,
        args: &'static [&'static str],
        cols: u16,
        rows: u16,
        color: Color,
        needle: &'static str,
    ) -> Self {
        let sf_name = screen_first_path(&name);
        Self {
            name: Cow::Owned(sf_name),
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

    pub fn sends(self, sends: &'static [&'static str]) -> Self {
        Self { sends, ..self }
    }

    pub fn timeout(self, ms: u64) -> Self {
        Self {
            timeout_ms: ms,
            ..self
        }
    }

    fn variant(&self, cols: u16, rows: u16, color: Color) -> Self {
        let root = canonical_root(&self.name).unwrap_or_else(|| {
            panic!(
                "`{}` is not a canonical `<root>/<size>/<color>` capture",
                self.name
            )
        });
        Self {
            name: Cow::Owned(canonical_name(root, cols, rows, color)),
            bin: self.bin,
            args: self.args,
            cols,
            rows,
            color,
            needle: self.needle,
            sends: self.sends,
            timeout_ms: self.timeout_ms,
        }
    }

    pub fn resize_variant(&self, cols: u16, rows: u16, color: Color) -> Self {
        let root = canonical_root(&self.name).unwrap_or_else(|| {
            panic!(
                "`{}` is not a canonical `<root>/<size>/<color>` capture",
                self.name
            )
        });
        Self {
            name: Cow::Owned(canonical_name(root, cols, rows, color)),
            bin: self.bin,
            args: self.args,
            cols: self.cols,
            rows: self.rows,
            color,
            needle: self.needle,
            sends: self.sends,
            timeout_ms: self.timeout_ms,
        }
    }
}

/// Opt-in marking a run as explicitly partial/diagnostic (non-acceptance).
/// Set `TERMROCK_PARTIAL_RUN=1` to allow `COMBO_FILTER` subset runs or
/// mid-test-built subjects without failing the acceptance verdict. Filtered
/// or mid-test-built runs without this flag FAIL completeness accounting;
/// results produced with it must never be cited as parity evidence.
pub const PARTIAL_RUN_ENV: &str = "TERMROCK_PARTIAL_RUN";

/// Local-iteration escape hatch: set `TERMROCK_ALLOW_MIDTEST_BUILD=1` to
/// permit the legacy mid-test `cargo build --bin` fallback when the subject
/// binary is missing. Using it poisons the run: the acceptance verdict fails
/// unless [`PARTIAL_RUN_ENV`] is also set, so hatch-built subjects can never
/// silently become an acceptance subject.
pub const MIDTEST_BUILD_ENV: &str = "TERMROCK_ALLOW_MIDTEST_BUILD";

pub fn partial_run_allowed() -> bool {
    std::env::var(PARTIAL_RUN_ENV).as_deref() == Ok("1")
}

pub fn midtest_build_allowed() -> bool {
    std::env::var(MIDTEST_BUILD_ENV).as_deref() == Ok("1")
}

static MIDTEST_BUILD_USED: AtomicBool = AtomicBool::new(false);

pub fn midtest_build_used() -> bool {
    MIDTEST_BUILD_USED.load(Ordering::SeqCst)
}

fn mark_midtest_build_used() {
    MIDTEST_BUILD_USED.store(true, Ordering::SeqCst);
}

/// The executed subject binary, bound to its content digest at resolve time.
#[derive(Clone, Debug)]
pub struct BinSubject {
    pub path: PathBuf,
    pub sha256: String,
    pub len: u64,
    pub mtime_unix: Option<u64>,
}

/// Digest `path` into a [`BinSubject`]. Returns `None` when the file cannot
/// be read (missing binary, permissions); callers fail loudly instead.
pub fn digest_file(path: &Path) -> Option<BinSubject> {
    let bytes = std::fs::read(path).ok()?;
    let meta = std::fs::metadata(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = format!("{:x}", hasher.finalize());
    let mtime_unix = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    Some(BinSubject {
        path: path.to_path_buf(),
        sha256,
        len: meta.len(),
        mtime_unix,
    })
}

static SUBJECT_CACHE: std::sync::OnceLock<Mutex<HashMap<String, BinSubject>>> =
    std::sync::OnceLock::new();

fn cached_subject(name: &str, path: &Path) -> Option<BinSubject> {
    let cache = SUBJECT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cached = cache.lock().ok()?.get(name).cloned()?;
    if cached.path != path {
        return None;
    }
    let meta = std::fs::metadata(path).ok()?;
    let mtime_unix = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    if cached.len == meta.len() && cached.mtime_unix == mtime_unix {
        Some(cached)
    } else {
        None
    }
}

fn store_subject(name: &str, subject: &BinSubject) {
    if let Ok(mut cache) = SUBJECT_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(name.to_string(), subject.clone());
    }
}

fn candidate_bins(name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let key = format!("CARGO_BIN_EXE_{}", name.replace('-', "_"));
    if let Ok(path) = std::env::var(&key) {
        candidates.push(PathBuf::from(path));
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(Path::parent).unwrap_or(manifest);
    candidates.push(root.join("target").join("debug").join(name));
    candidates.push(root.join("target").join("release").join(name));
    candidates
}

/// Resolve `name` to its executed path and bind it to its sha256 digest.
///
/// This never builds: a missing binary is a loud error naming every searched
/// path plus the build instructions, so a stale or absent binary cannot
/// silently become the execution subject. The only build path is the
/// [`MIDTEST_BUILD_ENV`] escape hatch inside [`resolve_bin`], which poisons
/// the acceptance verdict (see [`acceptance_verdict`]).
pub fn try_resolve_bin(name: &str) -> Result<BinSubject, String> {
    for candidate in candidate_bins(name) {
        if !candidate.exists() {
            continue;
        }
        if let Some(subject) = cached_subject(name, &candidate) {
            return Ok(subject);
        }
        if let Some(subject) = digest_file(&candidate) {
            eprintln!(
                "subject bin={name} path={} sha256={} len={} mtime={}",
                subject.path.display(),
                subject.sha256,
                subject.len,
                subject
                    .mtime_unix
                    .map_or_else(|| "-".to_string(), |m| m.to_string()),
            );
            store_subject(name, &subject);
            return Ok(subject);
        }
    }
    let searched = candidate_bins(name)
        .iter()
        .map(|p| format!("  {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "subject binary `{name}` missing: not built from the reviewed commit; refusing to substitute a stale subject.\n\
         searched paths:\n{searched}\n\
         build the reviewed tree first (e.g. `cargo build --bin {name}` or run via nextest, which builds first), then re-run.\n\
         local iteration only: set {MIDTEST_BUILD_ENV}=1 to allow a mid-test build plus {PARTIAL_RUN_ENV}=1 to mark the run non-acceptance."
    ))
}

fn midtest_build(name: &str) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(Path::parent).unwrap_or(manifest);
    static BUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    if let Ok(_guard) = BUILD_LOCK.lock() {
        let _ = std::process::Command::new("cargo")
            .args(["build", "--bin", name])
            .current_dir(root)
            .status();
    }
}

pub fn resolve_bin(name: &str) -> PathBuf {
    match try_resolve_bin(name) {
        Ok(subject) => subject.path,
        Err(err) => {
            if !midtest_build_allowed() {
                panic!("{err}");
            }
            eprintln!(
                "NON-ACCEPTANCE: mid-test `cargo build --bin {name}` fallback engaged via \
                 {MIDTEST_BUILD_ENV}=1; this run cannot pass acceptance unless {PARTIAL_RUN_ENV}=1 is also set."
            );
            mark_midtest_build_used();
            midtest_build(name);
            match try_resolve_bin(name) {
                Ok(subject) => subject.path,
                Err(_) => panic!("{err}"),
            }
        }
    }
}

pub fn argv_for(case: &Case) -> Vec<String> {
    let mut argv = Vec::with_capacity(case.args.len() + 3);
    argv.push(resolve_bin(case.bin).display().to_string());
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

/// One row of visible text: continuation cells skipped, symbols
/// concatenated. Callers trim; matching never depends on trailing blanks.
pub fn row_text(screen: &tuiscotti::Screen, row: u16) -> String {
    let mut s = String::new();
    for x in 0..screen.cols() {
        if let Some(c) = screen.get(x, row)
            && !c.continuation
        {
            s.push_str(&c.symbol);
        }
    }
    s
}

/// Full visible text: rows joined with `\n` (the `wait_for_text` surface).
pub fn screen_text(screen: &tuiscotti::Screen) -> String {
    (0..screen.rows())
        .map(|y| row_text(screen, y))
        .collect::<Vec<_>>()
        .join("\n")
}

/// First occurrence of `needle` as `(row, col)` — top-to-bottom, leftmost
/// per row. The byte offset becomes a display column via the prefix width,
/// so wide-cell rows resolve exactly.
pub fn screen_find(screen: &tuiscotti::Screen, needle: &str) -> Option<(u16, u16)> {
    if needle.is_empty() {
        return None;
    }
    for row in 0..screen.rows() {
        let line = row_text(screen, row);
        if let Some(off) = line.find(needle) {
            let col = unicode_width::UnicodeWidthStr::width(&line[..off]) as u16;
            return Some((row, col));
        }
    }
    None
}

pub struct Session {
    pub inner: tuiscotti::tui::Session,
    pub cols: u16,
    pub rows: u16,
}

#[allow(dead_code)]
impl Session {
    pub fn wait_until<F>(&mut self, pred: F) -> Result<(), tuiscotti::tui::WaitError>
    where
        F: FnMut(&tuiscotti::Screen) -> bool,
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let cancel = CancelToken::new();
        let pred_cell = std::cell::RefCell::new(pred);
        self.inner
            .wait_predicate(|obs| pred_cell.borrow_mut()(&obs.screen), deadline, &cancel)?;
        Ok(())
    }

    pub fn wait_for_text(&mut self, needle: &str) -> Result<(), tuiscotti::tui::WaitError> {
        self.wait_until(|screen| frame_from_screen(screen, "default").text().contains(needle))
    }

    pub fn wait_idle(&mut self, quiet: Duration) -> Result<(), tuiscotti::tui::WaitError> {
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let cancel = CancelToken::new();
        self.inner.wait_stable_quiet(deadline, quiet, &cancel)?;
        Ok(())
    }

    pub fn wait_stable(&mut self, quiet: Duration) -> Result<Frame, tuiscotti::tui::WaitError> {
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let cancel = CancelToken::new();
        let obs = self.inner.wait_stable_quiet(deadline, quiet, &cancel)?;
        Ok(frame_from_screen(&obs.screen, "default"))
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), tuiscotti::tui::TuiError> {
        self.inner.send_text(text)
    }

    pub fn send_key(&mut self, key: &str) -> Result<(), tuiscotti::tui::TuiError> {
        press_chord(&self.inner, key)
    }

    pub fn click_with(
        &mut self,
        button: MouseButton,
        col: u16,
        row: u16,
    ) -> Result<(), tuiscotti::tui::TuiError> {
        self.inner
            .mouse_down(button, col, row, MouseMods::default())?;
        std::thread::sleep(Duration::from_millis(50));
        self.inner.mouse_up(button, col, row, MouseMods::default())
    }

    pub fn click(&mut self, col: u16, row: u16) -> Result<(), tuiscotti::tui::TuiError> {
        self.click_with(MouseButton::Left, col, row)
    }

    pub fn drag(
        &mut self,
        from_col: u16,
        from_row: u16,
        to_col: u16,
        to_row: u16,
    ) -> Result<(), tuiscotti::tui::TuiError> {
        self.inner
            .mouse_down(MouseButton::Left, from_col, from_row, MouseMods::default())?;
        std::thread::sleep(Duration::from_millis(20));
        self.inner
            .mouse_drag(MouseButton::Left, to_col, to_row, MouseMods::default())?;
        std::thread::sleep(Duration::from_millis(20));
        self.inner
            .mouse_up(MouseButton::Left, to_col, to_row, MouseMods::default())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), tuiscotti::tui::TuiError> {
        self.cols = cols;
        self.rows = rows;
        self.inner.resize(cols, rows)
    }

    pub fn snapshot(&mut self) -> Result<Frame, tuiscotti::tui::TuiError> {
        let screen = self.inner.snapshot()?;
        Ok(frame_from_screen(&screen, "default"))
    }
}

pub fn spawn(case: &Case) -> Session {
    let argv = argv_for(case);
    let mut builder = Tui::new(&argv).size(case.cols, case.rows);
    builder = builder.env_remove("NO_COLOR");
    builder = builder.env_remove("HOLLA_NO_MOTION");
    builder = builder.env_remove("JACKIN_NO_MOTION");
    builder = builder.env_remove("CLICOLOR_FORCE");
    builder = builder.env_remove("FORCE_COLOR");
    builder = builder.env("HOLLA_NO_HISTORY", "1");
    if let Color::NoColorEnv = case.color {
        builder = builder.env("NO_COLOR", "1");
    }

    let inner = builder
        .spawn()
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name));
    Session {
        inner,
        cols: case.cols,
        rows: case.rows,
    }
}

pub fn spawn_boot(case: &Case) -> Session {
    let timeout = Duration::from_millis(case.timeout_ms);
    let mut session = spawn(case);
    boot(&mut session, case.needle, timeout);
    if !case.sends.is_empty() {
        drive_with_timeout(&mut session, case.sends, case.timeout_ms);
    }
    session
}

/// Boot: needle first, bounded by the case timeout. Live clocks starve a
/// quiet-window wait, so an empty needle alone takes the idle path.
pub fn boot(session: &mut Session, needle: &str, timeout: Duration) {
    if !needle.is_empty() {
        wait_screen(
            session,
            timeout,
            &format!("boot needle `{needle}` never appeared"),
            |screen| screen_text(screen).contains(needle),
        );
        return;
    }
    let deadline = std::time::Instant::now() + timeout;
    let cancel = CancelToken::new();
    session
        .inner
        .wait_stable_quiet(deadline, Duration::from_millis(200), &cancel)
        .unwrap_or_else(|e| panic!("boot idle failed: {e:#}"));
}

/// Bounded wait until `pred` holds, returning the outcome for callers
/// that report their own timeout evidence.
pub fn try_wait_screen(
    s: &mut Session,
    timeout: Duration,
    pred: impl FnMut(&tuiscotti::Screen) -> bool,
) -> Result<tuiscotti::Observation, tuiscotti::tui::WaitError> {
    // The poll loop calls the predicate sequentially (no reentrancy), so a
    // RefCell bridges the FnMut caller surface to the Fn wait surface.
    let pred = RefCell::new(pred);
    s.inner
        .wait_predicate_timeout(|o| pred.borrow_mut()(&o.screen), timeout)
}

/// Bounded wait until `pred` holds on a fresh observation. Returns the
/// matching observation; the predicate sees the live screen, as before.
pub fn wait_screen(
    s: &mut Session,
    timeout: Duration,
    what: &str,
    pred: impl FnMut(&tuiscotti::Screen) -> bool,
) -> tuiscotti::Observation {
    try_wait_screen(s, timeout, pred).unwrap_or_else(|e| panic!("{what}: {e:#}"))
}

/// Primary-button drag from `from` to `to`: press, one held-motion
/// report per cell crossed (straight-line interpolation, as a terminal
/// sends), release. Applications may act along the path, not just on
/// its ends, so a single endpoint hop would under-report.
pub fn drag_path(s: &mut Session, from: (u16, u16), to: (u16, u16)) {
    s.inner
        .mouse_down(MouseButton::Left, from.0, from.1, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("drag press at {from:?} failed: {e:#}"));
    for (col, row) in cells_between(from, to) {
        s.inner
            .mouse_drag(MouseButton::Left, col, row, MouseMods::NONE)
            .unwrap_or_else(|e| panic!("drag motion to ({col}, {row}) failed: {e:#}"));
    }
    s.inner
        .mouse_up(MouseButton::Left, to.0, to.1, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("drag release at {to:?} failed: {e:#}"));
}

/// Cells on the straight line from `from` to `to`, exclusive of `from`,
/// inclusive of `to`: round-to-nearest on both axes so the short axis
/// turns over mid-run. A drag to the same cell still reports one motion.
fn cells_between(from: (u16, u16), to: (u16, u16)) -> Vec<(u16, u16)> {
    let (from_col, from_row) = (i64::from(from.0), i64::from(from.1));
    let (to_col, to_row) = (i64::from(to.0), i64::from(to.1));
    let d_col = to_col - from_col;
    let d_row = to_row - from_row;
    let steps = d_col.abs().max(d_row.abs());
    if steps == 0 {
        return vec![to];
    }
    (1..=steps)
        .map(|step| {
            let col = from_col + (d_col * step + d_col.signum() * steps / 2) / steps;
            let row = from_row + (d_row * step + d_row.signum() * steps / 2) / steps;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            (col as u16, row as u16)
        })
        .collect()
}

/// Settle the screen and return the settled frame: no new output for
/// `quiet`, bounded by `timeout`.
pub fn settle_frame(
    session: &mut Session,
    quiet: Duration,
    timeout: Duration,
    name: &str,
) -> Frame {
    let cancel = CancelToken::new();
    let obs = session
        .inner
        .wait_stable_quiet(std::time::Instant::now() + timeout, quiet, &cancel)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e:#}"));
    frame_from_screen(&obs.screen, "default")
}

fn normalize_chord(step: &str) -> String {
    if step.eq_ignore_ascii_case("backtab") {
        return "shift+tab".to_string();
    }
    let parts: Vec<&str> = step.split('-').collect();
    if parts.len() > 1 {
        let modifiers = [
            "ctrl", "control", "ctl", "alt", "opt", "meta", "shift", "super", "cmd", "win",
            "windows", "command",
        ];
        let mut has_mod = false;
        for p in &parts[..parts.len() - 1] {
            if modifiers.contains(&p.to_ascii_lowercase().as_str()) {
                has_mod = true;
            }
        }
        if has_mod {
            return parts.join("+");
        }
    }
    step.to_string()
}

fn press_chord(
    inner: &tuiscotti::tui::Session,
    chord_str: &str,
) -> Result<(), tuiscotti::tui::TuiError> {
    let chord = normalize_chord(chord_str);
    let mut parts: Vec<&str> = chord.split('+').collect();
    let key_name = parts.pop().unwrap_or_default();
    if key_name.eq_ignore_ascii_case("f") {
        let mut mods = tuiscotti::tui::KeyMods::NONE;
        for m in parts {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" | "ctl" => mods.ctrl = true,
                "alt" | "opt" | "meta" => mods.alt = true,
                "shift" => mods.shift = true,
                "super" | "cmd" | "win" | "windows" | "command" => mods.ext.sup = true,
                _ => {}
            }
        }
        let ch = key_name.chars().next().unwrap_or('f');
        return inner.press_key(tuiscotti::tui::Key::Char(ch), mods);
    }
    inner.press(&chord)
}

#[allow(dead_code)]
pub fn drive(session: &mut Session, steps: &[&str]) {
    drive_with_timeout(session, steps, 8_000);
}

pub fn drive_with_timeout(session: &mut Session, steps: &[&str], timeout_ms: u64) {
    for step in steps {
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
            let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms.max(8_000));
            let cancel = CancelToken::new();
            let last_text = std::sync::Mutex::new(String::new());
            session
                .inner
                .wait_predicate(
                    |obs| {
                        let t = frame_from_screen(&obs.screen, "default").text();
                        let found = t.contains(needle);
                        if let Ok(mut l) = last_text.lock() {
                            *l = t;
                        }
                        found
                    },
                    deadline,
                    &cancel,
                )
                .unwrap_or_else(|e| {
                    let text = last_text.lock().map_or_else(|_| String::new(), |l| l.clone());
                    panic!("`wait:{needle}` timed out: {e:#}\n--- SCREEN TEXT ---\n{text}\n--- END SCREEN TEXT ---")
                });
        } else if let Some(text) = step.strip_prefix("type:") {
            session.inner.send_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            press_chord(&session.inner, step).expect("send_key");
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

#[allow(dead_code)]
pub fn settle_and_gate(session: &mut Session, name: &str) {
    settle_and_gate_with_timeout(session, name, 8_000);
}

pub fn settle_and_gate_with_timeout(session: &mut Session, name: &str, timeout_ms: u64) {
    let cancel = CancelToken::new();
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms.max(8_000));
    let obs = session
        .inner
        .wait_stable_quiet(deadline, SETTLE, &cancel)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e:#}"));
    let frame = frame_from_screen(&obs.screen, "default");
    assert_gated(&gate(name, &frame));
}

thread_local! {
    static RENDERER: RefCell<Renderer> = RefCell::new(
        Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse"),
    );
}

pub const DEFAULT_BASELINE_STORE: &str = "baselines/tuiscotti-v1";

pub fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

pub fn baseline_store_root() -> PathBuf {
    let workspace_root = workspace_root();
    match std::env::var("VISUAL_BASELINE_STORE").as_deref() {
        Ok("tuiscotti" | "tuiscotti-v1" | "baselines/tuiscotti-v1") => {
            let path = workspace_root.join("baselines/tuiscotti-v1");
            if !path.is_dir() {
                panic!(
                    "explicit VISUAL_BASELINE_STORE target does not exist: {}",
                    path.display()
                );
            }
            path
        }
        Ok("snapshots" | "legacy") => {
            let path = workspace_root.join("snapshots");
            if !path.is_dir() {
                panic!(
                    "explicit VISUAL_BASELINE_STORE target does not exist: {}",
                    path.display()
                );
            }
            path
        }
        Ok(other) => {
            panic!("unknown VISUAL_BASELINE_STORE `{other}` (expected 'baselines/tuiscotti-v1')")
        }
        Err(std::env::VarError::NotPresent) => {
            let path = workspace_root.join(DEFAULT_BASELINE_STORE);
            if !path.is_dir() {
                panic!(
                    "default baseline store does not exist: {} (set VISUAL_BASELINE_STORE or create store)",
                    path.display()
                );
            }
            path
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("VISUAL_BASELINE_STORE is not valid unicode")
        }
    }
}

pub fn store() -> GroupedStore {
    let base = baseline_store_root();
    let ws_root = workspace_root();
    GroupedStore::new(&base)
        .with_actual_root(&ws_root.join("target/tuiscotti/actual"))
        .with_diff_root(&ws_root.join("target/tuiscotti/diff"))
        .with_report_path(&ws_root.join("target/tuiscotti/report.html"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_artifact_strictly(path: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create dir {}: {e}", parent.display()))?;
    }
    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temp_path, content)
        .map_err(|e| format!("failed to write temp file {}: {e}", temp_path.display()))?;
    std::fs::rename(&temp_path, path).map_err(|e| {
        format!(
            "failed to rename {} to {}: {e}",
            temp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn now_iso8601() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let rem_secs = rem_secs % 3600;
    let minutes = rem_secs / 60;
    let seconds = rem_secs % 60;

    let mut year = 1970;
    let mut d = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        month += 1;
    }
    let day = d + 1;
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

pub fn write_10_artifact_bundle(
    root: &Path,
    name: &str,
    frame: &Frame,
    renderer: &mut Renderer,
) -> Result<(), String> {
    let bundle = capture_all(renderer, frame, name)
        .map_err(|e| format!("capture_all `{name}` failed: {e}"))?;

    assert_seven_bit(&bundle.ascii.text).map_err(|e| format!("ascii 7-bit `{name}`: {e}"))?;
    assert_no_escapes(&bundle.txt).map_err(|e| format!("txt clean `{name}`: {e}"))?;
    assert_normalized_sgr(&bundle.ansi).map_err(|e| format!("ansi normalized `{name}`: {e}"))?;
    assert_opaque_rgb(&bundle.png).map_err(|e| format!("png opaque `{name}`: {e}"))?;
    assert_static_offline(&bundle.html).map_err(|e| format!("html static `{name}`: {e}"))?;
    let _ = parse_canonical(&bundle.json).map_err(|e| format!("valid frame json `{name}`: {e}"))?;

    let rendered = renderer
        .render(frame)
        .map_err(|e| format!("renderer.render `{name}` failed: {e}"))?;
    let fidelity_json = rendered.fidelity.to_json();

    let substitutions: Vec<serde_json::Value> = bundle
        .ascii
        .substitutions
        .iter()
        .map(|s| {
            serde_json::json!({
                "x": s.x,
                "y": s.y,
                "original": s.original,
                "replacement": s.replacement,
            })
        })
        .collect();
    let ascii_loss_obj = serde_json::json!({
        "lossy": bundle.ascii.lossy(),
        "substitutions_count": bundle.ascii.substitutions.len(),
        "substitutions": substitutions,
    });
    let ascii_loss_json = serde_json::to_string_pretty(&ascii_loss_obj)
        .map_err(|e| format!("serialize ascii loss `{name}`: {e}"))?;

    let observations_obj = serde_json::json!({
        "name": name,
        "cols": frame.cols,
        "rows": frame.rows,
        "cursor": {
            "x": frame.cursor.x,
            "y": frame.cursor.y,
            "visible": frame.cursor.visible,
        },
        "provenance": {
            "tool": frame.provenance.tool,
            "tool_version": frame.provenance.tool_version,
            "profile": frame.provenance.profile,
            "source": frame.provenance.source,
            "argv": frame.provenance.argv,
            "created_unix": frame.provenance.created_unix,
        },
        "cell_count": frame.cells.len(),
        "non_empty_cells": frame.cells.iter().filter(|c| c.symbol != " ").count(),
        "frame_digest": frame.digest(),
    });
    let observations_json = serde_json::to_string_pretty(&observations_obj)
        .map_err(|e| format!("serialize observations `{name}`: {e}"))?;

    // 1-6: 6 primary data formats
    let frame_json_path = root.join(format!("{name}.frame.json"));
    let ansi_path = root.join(format!("{name}.ansi"));
    let txt_path = root.join(format!("{name}.txt"));
    let png_path = root.join(format!("{name}.png"));
    let html_path = root.join(format!("{name}.html"));
    let ascii_path = root.join(format!("{name}.ascii"));

    // 7-9: 3 companion artifacts
    let ascii_loss_path = root.join(format!("{name}.ascii.loss.json"));
    let png_fidelity_path = root.join(format!("{name}.png.fidelity.json"));
    let observations_path = root.join(format!("{name}.observations.json"));

    // 10: manifest artifact
    let manifest_path = root.join(format!("{name}.manifest.json"));

    write_artifact_strictly(&frame_json_path, bundle.json.as_bytes())?;
    write_artifact_strictly(&ansi_path, bundle.ansi.as_bytes())?;
    write_artifact_strictly(&txt_path, bundle.txt.as_bytes())?;
    write_artifact_strictly(&png_path, &bundle.png)?;
    write_artifact_strictly(&html_path, bundle.html.as_bytes())?;
    write_artifact_strictly(&ascii_path, bundle.ascii.text.as_bytes())?;
    write_artifact_strictly(&ascii_loss_path, ascii_loss_json.as_bytes())?;
    write_artifact_strictly(&png_fidelity_path, fidelity_json.as_bytes())?;
    write_artifact_strictly(&observations_path, observations_json.as_bytes())?;

    let manifest_obj = serde_json::json!({
        "schema_version": 1,
        "name": name,
        "generation": bundle.generation.id,
        "frame_digest": bundle.generation.frame_digest,
        "profile": bundle.generation.profile,
        "dimensions": {
            "cols": frame.cols,
            "rows": frame.rows,
        },
        "ascii_substitutions_count": bundle.ascii.substitutions.len(),
        "artifacts": {
            "frame_json": {
                "file": format!("{name}.frame.json"),
                "sha256": sha256_hex(bundle.json.as_bytes()),
                "bytes": bundle.json.len(),
            },
            "ansi": {
                "file": format!("{name}.ansi"),
                "sha256": sha256_hex(bundle.ansi.as_bytes()),
                "bytes": bundle.ansi.len(),
            },
            "txt": {
                "file": format!("{name}.txt"),
                "sha256": sha256_hex(bundle.txt.as_bytes()),
                "bytes": bundle.txt.len(),
            },
            "png": {
                "file": format!("{name}.png"),
                "sha256": sha256_hex(&bundle.png),
                "bytes": bundle.png.len(),
            },
            "html": {
                "file": format!("{name}.html"),
                "sha256": sha256_hex(bundle.html.as_bytes()),
                "bytes": bundle.html.len(),
            },
            "ascii": {
                "file": format!("{name}.ascii"),
                "sha256": sha256_hex(bundle.ascii.text.as_bytes()),
                "bytes": bundle.ascii.text.len(),
            },
            "ascii_loss_json": {
                "file": format!("{name}.ascii.loss.json"),
                "sha256": sha256_hex(ascii_loss_json.as_bytes()),
                "bytes": ascii_loss_json.len(),
            },
            "png_fidelity_json": {
                "file": format!("{name}.png.fidelity.json"),
                "sha256": sha256_hex(fidelity_json.as_bytes()),
                "bytes": fidelity_json.len(),
            },
            "observations_json": {
                "file": format!("{name}.observations.json"),
                "sha256": sha256_hex(observations_json.as_bytes()),
                "bytes": observations_json.len(),
            },
        },
        "timestamp_utc": now_iso8601(),
    });
    let manifest_json = serde_json::to_string_pretty(&manifest_obj)
        .map_err(|e| format!("serialize manifest `{name}`: {e}"))?;

    write_artifact_strictly(&manifest_path, manifest_json.as_bytes())?;

    // Verify all 10 artifacts exist and are non-empty
    let paths = [
        &frame_json_path,
        &ansi_path,
        &txt_path,
        &png_path,
        &html_path,
        &ascii_path,
        &ascii_loss_path,
        &png_fidelity_path,
        &observations_path,
        &manifest_path,
    ];
    for p in paths {
        let meta = std::fs::metadata(p)
            .map_err(|e| format!("verify artifact {} failed: {e}", p.display()))?;
        if meta.len() == 0 {
            return Err(format!("artifact {} was written with 0 bytes", p.display()));
        }
    }

    Ok(())
}

/// Every manifest-pinned artifact the actual-vs-approved gate compares, as
/// `(manifest key, file extension)`.
///
/// Six primary formats (implementation-goal §7: "ANSI, HTML, PNG, ASCII, TXT
/// and canonical frame JSON from the same observation") plus three companions
/// ("loss/fidelity/semantic metadata"). The tenth bundle member — the
/// manifest itself — is the pin source: it authenticates both sides and is
/// never compared across sides (`timestamp_utc` legitimately differs per
/// write, and artifact hashes are comparison inputs, not outputs).
///
/// `.ascii` is both a §7 format and a lossy diagnostic. Here it is gated
/// byte-exact as an exporter-determinism check (same frame + same exporter
/// must emit the same bytes); it is never the Unicode/style oracle — that
/// role belongs to `.frame.json` (decoded cell structure) and `.ansi`
/// (normalized SGR), per §7 "ASCII is lossy diagnostic output, never a
/// Unicode/style equality oracle".
const COMPARED_ARTIFACTS: [(&str, &str); 9] = [
    ("frame_json", "frame.json"),
    ("ansi", "ansi"),
    ("txt", "txt"),
    ("png", "png"),
    ("html", "html"),
    ("ascii", "ascii"),
    ("ascii_loss_json", "ascii.loss.json"),
    ("png_fidelity_json", "png.fidelity.json"),
    ("observations_json", "observations.json"),
];

/// Maximum JSON field-difference lines folded into one failure note.
const MAX_JSON_DIFF_LINES: usize = 10;

/// Human summary of one cell for failure diagnostics: glyph plus resolved
/// colors, modifiers, underline color, and continuation state.
///
/// Mirrors the native tuiscotti gate's cell summary (same fields, same
/// tokens) so harness diagnostics read exactly like native ones; the native
/// helper is private, hence this local mirror.
fn summarize_cell(cell: &Cell) -> String {
    if cell.continuation {
        return "…".to_string();
    }
    let (fg, bg) = Frame::resolve_cell(cell, Rgb::new(0xd0, 0xd0, 0xd0), Rgb::new(0, 0, 0));
    let mut mods = String::new();
    if cell.mods.hidden {
        mods.push_str("+hidden");
    }
    if cell.mods.blink {
        mods.push_str("+blink");
    }
    if cell.mods.bold {
        mods.push_str("+bold");
    }
    if cell.mods.dim {
        mods.push_str("+dim");
    }
    if cell.mods.italic {
        mods.push_str("+italic");
    }
    match cell.mods.effective_underline_style() {
        UnderlineStyle::None => {}
        UnderlineStyle::Single => mods.push_str("+ul"),
        UnderlineStyle::Double => mods.push_str("+ul2"),
        UnderlineStyle::Curly => mods.push_str("+ulcurl"),
        UnderlineStyle::Dotted => mods.push_str("+uldot"),
        UnderlineStyle::Dashed => mods.push_str("+uldash"),
    }
    if cell.mods.strikethrough {
        mods.push_str("+strike");
    }
    if cell.mods.reverse {
        mods.push_str("+rev");
    }
    if !cell.underline_color.is_default() {
        let uc = match cell.underline_color {
            tuiscotti::Color::Default => fg,
            tuiscotti::Color::Indexed(i) => Rgb::from_indexed(i),
            tuiscotti::Color::Rgb(r) => r,
        };
        write!(mods, "+ulc={}", uc.to_hex()).ok();
    }
    let symbol = &cell.symbol;
    let fg_hex = fg.to_hex();
    let bg_hex = bg.to_hex();
    format!("{symbol:?} fg={fg_hex} bg={bg_hex}{mods}")
}

/// Decoded cell-structure comparison between the live actual frame and the
/// approved frame: the full differing-cell count plus the FIRST N [`CellDiff`]s
/// in row-major order (N = [`MAX_CELL_DIFFS`], the native cap).
///
/// Cursor-only drift reports cursor state instead of cell text: every
/// reported position holds equal cells, so cell summaries would print the
/// same string twice — the diagnostic must say what actually changed
/// (mirrors the native gate). Returns `None` on dimension mismatch (a
/// status, not a diff); the caller reports dimensions + digests instead.
fn first_cell_diffs(actual: &Frame, approved: &Frame) -> Option<(Vec<CellDiff>, usize)> {
    let positions = actual.diff_cells(approved).ok()?;
    let total = positions.len();
    let mut diffs = Vec::new();
    let cells_equal = positions.iter().all(|(x, y)| {
        approved.get(*x, *y).map(summarize_cell) == actual.get(*x, *y).map(summarize_cell)
    });
    if cells_equal && total > 0 {
        diffs.push(CellDiff {
            x: actual.cursor.x,
            y: actual.cursor.y,
            expected: Frame::summarize_cursor(&approved.cursor),
            actual: Frame::summarize_cursor(&actual.cursor),
        });
    } else {
        for (x, y) in positions.into_iter().take(MAX_CELL_DIFFS) {
            let expected = approved
                .get(x, y)
                .map_or_else(|| "∅".to_string(), summarize_cell);
            let actual_text = actual
                .get(x, y)
                .map_or_else(|| "∅".to_string(), summarize_cell);
            diffs.push(CellDiff {
                x,
                y,
                expected,
                actual: actual_text,
            });
        }
    }
    Some((diffs, total))
}

/// Render a JSON value compactly for diagnostics, truncated to stay readable.
fn json_snippet(value: &serde_json::Value) -> String {
    const LIMIT: usize = 160;
    let text = serde_json::to_string(value).unwrap_or_else(|_| "?".to_string());
    if text.chars().count() > LIMIT {
        let head: String = text.chars().take(LIMIT).collect();
        format!("{head}…")
    } else {
        text
    }
}

/// Collect `path: expected <e> | actual <a>` lines for values that differ,
/// recursing through objects (union of keys; absent reads as `∅`) and arrays
/// (by index; length mismatch noted). Stops after [`MAX_JSON_DIFF_LINES`].
fn json_diffs(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    path: &str,
    out: &mut Vec<String>,
) {
    if out.len() >= MAX_JSON_DIFF_LINES || expected == actual {
        return;
    }
    match (expected, actual) {
        (serde_json::Value::Object(expected_map), serde_json::Value::Object(actual_map)) => {
            let keys: BTreeSet<&String> = expected_map.keys().chain(actual_map.keys()).collect();
            for key in keys {
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                match (expected_map.get(key), actual_map.get(key)) {
                    (Some(expected_value), Some(actual_value)) => {
                        json_diffs(expected_value, actual_value, &child, out);
                    }
                    (Some(expected_value), None) => out.push(format!(
                        "{child}: expected {} | actual ∅",
                        json_snippet(expected_value)
                    )),
                    (None, Some(actual_value)) => out.push(format!(
                        "{child}: expected ∅ | actual {}",
                        json_snippet(actual_value)
                    )),
                    (None, None) => {}
                }
                if out.len() >= MAX_JSON_DIFF_LINES {
                    return;
                }
            }
        }
        (serde_json::Value::Array(expected_items), serde_json::Value::Array(actual_items)) => {
            if expected_items.len() != actual_items.len() {
                out.push(format!(
                    "{path}: array length differs: expected {} | actual {}",
                    expected_items.len(),
                    actual_items.len()
                ));
            }
            for (index, (expected_value, actual_value)) in
                expected_items.iter().zip(actual_items.iter()).enumerate()
            {
                json_diffs(
                    expected_value,
                    actual_value,
                    &format!("{path}[{index}]"),
                    out,
                );
                if out.len() >= MAX_JSON_DIFF_LINES {
                    return;
                }
            }
        }
        _ => out.push(format!(
            "{path}: expected {} | actual {}",
            json_snippet(expected),
            json_snippet(actual)
        )),
    }
}

/// Project `.observations.json` onto the stable semantic subset compared
/// actual-vs-approved: dimensions, cursor, cell counts, frame digest, and
/// renderer/tool identity. Deliberately excluded:
/// - `name` / `legacy_name`: bundle identity (fixed by path), not state;
/// - `provenance.argv` / `provenance.created_unix`: volatile per-run metadata
///   (`Provenance` documents `created_unix` as informational only; argv carries
///   machine-specific binary paths in general).
fn stable_observations(value: &serde_json::Value) -> serde_json::Value {
    let provenance = value.get("provenance");
    let provenance_field = |field: &str| {
        provenance
            .and_then(|provenance| provenance.get(field))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };
    serde_json::json!({
        "cols": value.get("cols"),
        "rows": value.get("rows"),
        "cursor": value.get("cursor"),
        "cell_count": value.get("cell_count"),
        "non_empty_cells": value.get("non_empty_cells"),
        "frame_digest": value.get("frame_digest"),
        "provenance": {
            "tool": provenance_field("tool"),
            "tool_version": provenance_field("tool_version"),
            "profile": provenance_field("profile"),
            "source": provenance_field("source"),
        },
    })
}

/// Gate one actual bundle against its approved bundle: all six formats plus
/// all three companions, after manifest-pin authentication of both sides.
///
/// Comparison semantics per artifact (see [`COMPARED_ARTIFACTS`]):
/// - `.ansi` / `.txt` / `.ascii` / `.html`: exact byte equality;
/// - `.png`: exact decoded-pixel identity via native `compare_png`
///   (dimensions plus `pixels_equal`; the score is diagnostic only, never a
///   threshold);
/// - `.frame.json`: decoded cell-structure comparison (`diff_cells`) of the
///   approved bytes against the LIVE actual frame — the in-memory execution
///   subject, not its serialization — yielding real first-N diagnostics; the
///   actual file must also describe the live frame (skew is
///   `CaptureIncomplete`, never a pass);
/// - `.ascii.loss.json` / `.png.fidelity.json`: exact recorded-value equality
///   (parsed JSON; no tolerance invented);
/// - `.observations.json`: exact equality over the stable semantic subset
///   ([`stable_observations`]; volatile provenance excluded).
///
/// Fail-closed hierarchy: missing approved artifacts → `MissingApproval`;
/// approved bytes failing their manifest pin, or unparseable approved JSON →
/// `CorruptApproval` (never compare against unauthenticated bytes); actual
/// bundle inconsistent with its own manifest or frame → `CaptureIncomplete`
/// (our write skewed, never a pass). Content drift → `CellsDiffer`
/// (cell/semantic level) or `PixelsDiffer` (render level), with first-N cell
/// diagnostics plus a state summary in the note.
pub fn compare_bundle_dirs(
    name: &str,
    actual_frame: &Frame,
    actual_root: &Path,
    approved_root: &Path,
    diff_root: &Path,
) -> Result<GroupedOutcome, String> {
    let actual_paths = tuiscotti::grouped::ArtifactPaths {
        ansi: actual_root.join(format!("{name}.ansi")),
        txt: actual_root.join(format!("{name}.txt")),
        png: actual_root.join(format!("{name}.png")),
        html: actual_root.join(format!("{name}.html")),
        frame_json: actual_root.join(format!("{name}.frame.json")),
    };

    let approved_paths = tuiscotti::grouped::ArtifactPaths {
        ansi: approved_root.join(format!("{name}.ansi")),
        txt: approved_root.join(format!("{name}.txt")),
        png: approved_root.join(format!("{name}.png")),
        html: approved_root.join(format!("{name}.html")),
        frame_json: approved_root.join(format!("{name}.frame.json")),
    };

    let shell = |status: Status, note: String| GroupedOutcome {
        outcome: tuiscotti::snapshot::CompareOutcome {
            name: name.to_string(),
            status,
            cell_diffs: Vec::new(),
            cell_diff_total: 0,
            pixel_score: None,
            approved_png_regenerated: false,
            digest_expected: None,
            digest_actual: actual_frame.digest().to_string(),
            actual_frame: actual_paths.frame_json.clone(),
            actual_png: actual_paths.png.clone(),
            expected_frame: approved_paths.frame_json.clone(),
            expected_png: None,
            expected_png_bytes: None,
            diff_png: None,
            note,
        },
        ansi_match: None,
        txt_match: None,
        html_match: None,
        actual: actual_paths.clone(),
        approved: approved_paths.clone(),
    };

    // Authenticate both sides against their manifest pins BEFORE comparing.
    let approved_manifest_path = approved_root.join(format!("{name}.manifest.json"));
    let approved_manifest_bytes = match std::fs::read(&approved_manifest_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(shell(
                Status::MissingApproval,
                format!("missing approved {name}.manifest.json"),
            ));
        }
    };
    let approved_manifest: serde_json::Value =
        match serde_json::from_slice(&approved_manifest_bytes) {
            Ok(manifest) => manifest,
            Err(e) => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!("approved {name}.manifest.json unparseable: {e}"),
                ));
            }
        };
    let actual_manifest_path = actual_root.join(format!("{name}.manifest.json"));
    let actual_manifest_bytes = match std::fs::read(&actual_manifest_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(shell(
                Status::CaptureIncomplete,
                format!("missing actual {name}.manifest.json"),
            ));
        }
    };
    let actual_manifest: serde_json::Value = match serde_json::from_slice(&actual_manifest_bytes) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Ok(shell(
                Status::CaptureIncomplete,
                format!("actual {name}.manifest.json unparseable: {e}"),
            ));
        }
    };

    let pin_for = |manifest: &serde_json::Value, key: &str| -> Option<String> {
        manifest
            .get("artifacts")?
            .get(key)?
            .get("sha256")?
            .as_str()
            .map(str::to_string)
    };

    // Read + pin-authenticate all nine artifacts on both sides.
    let mut approved_bytes: HashMap<&str, Vec<u8>> = HashMap::new();
    let mut actual_bytes_map: HashMap<&str, Vec<u8>> = HashMap::new();
    for (key, ext) in COMPARED_ARTIFACTS {
        let file = format!("{name}.{ext}");
        let approved = match std::fs::read(approved_root.join(&file)) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Ok(shell(
                    Status::MissingApproval,
                    format!("missing approved {file}"),
                ));
            }
        };
        match pin_for(&approved_manifest, key) {
            Some(pin) if sha256_hex(&approved) == pin => {}
            Some(pin) => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!(
                        "approved {file} fails manifest pin: pinned {pin}, got {}",
                        sha256_hex(&approved)
                    ),
                ));
            }
            None => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!("approved manifest missing pin for '{key}' ({file})"),
                ));
            }
        }
        let actual = match std::fs::read(actual_root.join(&file)) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("missing actual {file}"),
                ));
            }
        };
        match pin_for(&actual_manifest, key) {
            Some(pin) if sha256_hex(&actual) == pin => {}
            Some(pin) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!(
                        "actual {file} fails manifest pin: pinned {pin}, got {}",
                        sha256_hex(&actual)
                    ),
                ));
            }
            None => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("actual manifest missing pin for '{key}' ({file})"),
                ));
            }
        }
        approved_bytes.insert(key, approved);
        actual_bytes_map.insert(key, actual);
    }

    let byte_match = |key: &str| actual_bytes_map[key] == approved_bytes[key];
    let ansi_matched = byte_match("ansi");
    let txt_matched = byte_match("txt");
    let ascii_matched = byte_match("ascii");
    let html_matched = byte_match("html");

    let verdict = tuiscotti::diff::compare_png(&approved_bytes["png"], &actual_bytes_map["png"])
        .map_err(|e| format!("compare png: {e}"))?;

    let approved_frame: Frame = match std::str::from_utf8(&approved_bytes["frame_json"])
        .ok()
        .and_then(|text| serde_json::from_str::<Frame>(text).ok())
    {
        Some(frame) => frame,
        None => {
            return Ok(shell(
                Status::CorruptApproval,
                format!("approved {name}.frame.json is not valid frame JSON"),
            ));
        }
    };
    if let Err(e) = approved_frame.validate() {
        return Ok(shell(
            Status::CorruptApproval,
            format!("approved {name}.frame.json failed validation: {e}"),
        ));
    }

    // The actual frame file must describe the gated live frame; a skew is an
    // interrupted write, never a pass. Parsed without normalization (same as
    // the approved side) so raw modifiers compare symmetrically.
    let actual_file_frame: Option<Frame> = std::str::from_utf8(&actual_bytes_map["frame_json"])
        .ok()
        .and_then(|text| serde_json::from_str::<Frame>(text).ok());
    match actual_file_frame {
        Some(file_frame) => match file_frame.diff_cells(actual_frame) {
            Ok(positions) if positions.is_empty() => {}
            Ok(positions) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!(
                        "actual {name}.frame.json skews from the gated frame: {} cell(s) differ",
                        positions.len()
                    ),
                ));
            }
            Err(e) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("actual {name}.frame.json skews from the gated frame: {e}"),
                ));
            }
        },
        None => {
            return Ok(shell(
                Status::CaptureIncomplete,
                format!("actual {name}.frame.json is not valid frame JSON"),
            ));
        }
    }

    let (cell_diffs, cell_diff_total, frame_dims_match) =
        match first_cell_diffs(actual_frame, &approved_frame) {
            Some((diffs, total)) => (diffs, total, true),
            None => (Vec::new(), 0, false),
        };

    let loss_approved: serde_json::Value =
        match serde_json::from_slice(&approved_bytes["ascii_loss_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!("approved {name}.ascii.loss.json unparseable: {e}"),
                ));
            }
        };
    let loss_actual: serde_json::Value =
        match serde_json::from_slice(&actual_bytes_map["ascii_loss_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("actual {name}.ascii.loss.json unparseable: {e}"),
                ));
            }
        };
    let mut loss_diffs = Vec::new();
    json_diffs(
        &loss_approved,
        &loss_actual,
        "ascii.loss.json",
        &mut loss_diffs,
    );

    let fidelity_approved: serde_json::Value =
        match serde_json::from_slice(&approved_bytes["png_fidelity_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!("approved {name}.png.fidelity.json unparseable: {e}"),
                ));
            }
        };
    let fidelity_actual: serde_json::Value =
        match serde_json::from_slice(&actual_bytes_map["png_fidelity_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("actual {name}.png.fidelity.json unparseable: {e}"),
                ));
            }
        };
    let mut fidelity_diffs = Vec::new();
    json_diffs(
        &fidelity_approved,
        &fidelity_actual,
        "png.fidelity.json",
        &mut fidelity_diffs,
    );

    let observations_approved: serde_json::Value =
        match serde_json::from_slice(&approved_bytes["observations_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CorruptApproval,
                    format!("approved {name}.observations.json unparseable: {e}"),
                ));
            }
        };
    let observations_actual: serde_json::Value =
        match serde_json::from_slice(&actual_bytes_map["observations_json"]) {
            Ok(value) => value,
            Err(e) => {
                return Ok(shell(
                    Status::CaptureIncomplete,
                    format!("actual {name}.observations.json unparseable: {e}"),
                ));
            }
        };
    let mut observations_diffs = Vec::new();
    json_diffs(
        &stable_observations(&observations_approved),
        &stable_observations(&observations_actual),
        "observations",
        &mut observations_diffs,
    );

    let mut status = Status::Matched;
    let mut notes = Vec::new();
    let mut diff_png = None;

    if !txt_matched {
        status = Status::CellsDiffer;
        notes.push("txt differs".to_string());
    }
    if !ansi_matched {
        status = Status::CellsDiffer;
        notes.push("ansi differs".to_string());
    }
    if !ascii_matched {
        status = Status::CellsDiffer;
        notes.push("ascii differs".to_string());
    }
    if !frame_dims_match {
        status = Status::DimensionMismatch;
        notes.push(format!(
            "frame dimensions differ: {}x{} vs {}x{}",
            approved_frame.cols, approved_frame.rows, actual_frame.cols, actual_frame.rows
        ));
    } else if cell_diff_total > 0 {
        status = Status::CellsDiffer;
        notes.push(format!("frame cells differ: {cell_diff_total} cell(s)"));
    }
    if !loss_diffs.is_empty() {
        status = Status::CellsDiffer;
        notes.push(format!(
            "ascii.loss.json differs: {}",
            loss_diffs.join("; ")
        ));
    }
    if !observations_diffs.is_empty() {
        status = Status::CellsDiffer;
        notes.push(format!(
            "observations.json differs: {}",
            observations_diffs.join("; ")
        ));
    }

    if !verdict.dims_equal {
        status = Status::DimensionMismatch;
        notes.push(format!(
            "png dimensions differ: {:?} vs {:?}",
            verdict.expected_dims, verdict.actual_dims
        ));
    } else if !verdict.pixels_equal {
        if status.matched() {
            status = Status::PixelsDiffer;
        }
        let diff_path = diff_root.join(format!("{name}.png"));
        if let Some(parent) = diff_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&diff_path, &verdict.diff_png);
        diff_png = Some(diff_path);
        notes.push(format!("pixel similarity: {:.6}", verdict.score));
    }

    if !html_matched {
        if status.matched() {
            status = Status::PixelsDiffer;
        }
        notes.push("html differs".to_string());
    }
    if !fidelity_diffs.is_empty() {
        if status.matched() {
            status = Status::PixelsDiffer;
        }
        notes.push(format!(
            "png.fidelity.json differs: {}",
            fidelity_diffs.join("; ")
        ));
    }

    if !status.matched() {
        notes.push(format!(
            "state: dims expected {}x{} vs actual {}x{}; digest expected {} vs actual {}; cursor expected {} vs actual {}; {cell_diff_total} differing cell(s)",
            approved_frame.cols,
            approved_frame.rows,
            actual_frame.cols,
            actual_frame.rows,
            approved_frame.digest(),
            actual_frame.digest(),
            Frame::summarize_cursor(&approved_frame.cursor),
            Frame::summarize_cursor(&actual_frame.cursor),
        ));
    }

    Ok(GroupedOutcome {
        outcome: tuiscotti::snapshot::CompareOutcome {
            name: name.to_string(),
            status,
            cell_diffs,
            cell_diff_total,
            pixel_score: Some(verdict.score),
            approved_png_regenerated: false,
            digest_expected: Some(approved_frame.digest().to_string()),
            digest_actual: actual_frame.digest().to_string(),
            actual_frame: actual_paths.frame_json.clone(),
            actual_png: actual_paths.png.clone(),
            expected_frame: approved_paths.frame_json.clone(),
            expected_png: Some(approved_paths.png.clone()),
            expected_png_bytes: approved_bytes.remove("png"),
            diff_png,
            note: notes.join("; "),
        },
        ansi_match: Some(ansi_matched),
        txt_match: Some(txt_matched),
        html_match: Some(html_matched),
        actual: actual_paths,
        approved: approved_paths,
    })
}

pub fn check_baseline_bundle(
    name: &str,
    frame: &Frame,
    renderer: &mut Renderer,
) -> Result<GroupedOutcome, String> {
    let ws_root = workspace_root();
    let actual_root = ws_root.join("target/tuiscotti/actual");
    let diff_root = ws_root.join("target/tuiscotti/diff");
    let approved_root = baseline_store_root();

    write_10_artifact_bundle(&actual_root, name, frame, renderer)?;

    compare_bundle_dirs(name, frame, &actual_root, &approved_root, &diff_root)
}

/// The HTML gate masks one volatile field before it compares: the absolute
/// bless-worktree path in `provenance.argv[0]`
/// ([`forgive_html_argv0_drift`]). Every other byte stays exact.
pub fn gate(name: &str, frame: &Frame) -> GroupedOutcome {
    let target_name = screen_first_path(name);
    let mut outcome = RENDERER.with(|r| {
        let mut renderer = r.borrow_mut();
        check_baseline_bundle(&target_name, frame, &mut renderer)
            .unwrap_or_else(|e| panic!("gate `{target_name}` failed: {e}"))
    });
    forgive_html_argv0_drift(&target_name, &mut outcome);
    outcome
}

/// Placeholder that replaces the `provenance.argv[0]` value on both sides
/// of the HTML comparison. In-memory only; never written to any artifact.
const NORMALIZED_ARGV0: &str = "argv0-normalized";

/// Byte span of the `provenance.argv[0]` string value (content without the
/// quotes) inside a standalone `.html` render.
///
/// tuiscotti embeds the canonical frame JSON in one
/// `<script type="application/json">` block with `provenance` as its last
/// field, so the LAST `"argv"` key is the provenance one: any earlier
/// occurrence can only be cell text or SVG overlay text. Parsing is strict
/// (`"argv"` `:` `[` `"` value `"` with optional ASCII whitespace); an
/// empty argv or any malformed shape returns `None` (fail closed).
fn html_argv0_value_span(html: &[u8]) -> Option<(usize, usize)> {
    const KEY: &[u8] = b"\"argv\"";
    let key_at = html.windows(KEY.len()).rposition(|window| window == KEY)?;
    let mut i = key_at + KEY.len();
    while i < html.len() && html[i].is_ascii_whitespace() {
        i += 1;
    }
    if html.get(i) != Some(&b':') {
        return None;
    }
    i += 1;
    while i < html.len() && html[i].is_ascii_whitespace() {
        i += 1;
    }
    if html.get(i) != Some(&b'[') {
        return None;
    }
    i += 1;
    while i < html.len() && html[i].is_ascii_whitespace() {
        i += 1;
    }
    if html.get(i) != Some(&b'"') {
        return None;
    }
    i += 1;
    let start = i;
    while i < html.len() {
        match html[i] {
            b'\\' => {
                if i + 1 >= html.len() {
                    return None;
                }
                i += 2;
            }
            b'"' => return Some((start, i)),
            0x00..=0x1F => return None,
            _ => i += 1,
        }
    }
    None
}

/// Copy of `html` with the `provenance.argv[0]` value replaced by
/// [`NORMALIZED_ARGV0`]. `None` when the field is absent or malformed.
fn mask_html_argv0(html: &[u8]) -> Option<Vec<u8>> {
    let (start, end) = html_argv0_value_span(html)?;
    let mut masked = Vec::with_capacity(html.len() - (end - start) + NORMALIZED_ARGV0.len());
    masked.extend_from_slice(&html[..start]);
    masked.extend_from_slice(NORMALIZED_ARGV0.as_bytes());
    masked.extend_from_slice(&html[end..]);
    Some(masked)
}

/// `true` iff the two renders are byte-identical except for the
/// `provenance.argv[0]` value. Any masking failure reads as a difference.
fn html_equal_modulo_argv0(approved: &[u8], actual: &[u8]) -> bool {
    match (mask_html_argv0(approved), mask_html_argv0(actual)) {
        (Some(masked_approved), Some(masked_actual)) => masked_approved == masked_actual,
        _ => false,
    }
}

/// Forgive an HTML-only mismatch that is exactly the volatile bless path.
///
/// The tuiscotti HTML gate compares whole renders byte-exact. Each render
/// embeds `provenance.argv[0]`: the absolute path of the capture binary at
/// bless time. A rerun from another worktree changes only that path, so the
/// gate fails without any visual change. This wrapper re-compares the two
/// renders with only that value masked (see [`mask_html_argv0`]), and only
/// when the ansi, txt, and pixel gates already passed. The sealed verdict
/// is corrected to the normalized check so reports agree with the gate.
///
/// ASD-STE100 NOTE. The HTML file contains provenance.argv[0]. This value
/// is the absolute path of the binary at bless time. The path changes with
/// the worktree. It is volatile environment metadata (GOAL section 8). It
/// is not visual data. It is not semantic data. Mask only this value before
/// the compare. Compare all other bytes exactly. If other bytes differ, the
/// gate fails. The sealed verdict records the normalized check.
///
/// CANDIDATE ADAPTATION. [`compare_bundle_dirs`] also maps a
/// `png.fidelity.json` drift to [`Status::PixelsDiffer`], which the shape
/// above cannot see, so forgiveness additionally requires the bundle note
/// to record the HTML gate as the sole failing gate
/// ([`note_has_only_html_finding`]); any other finding keeps the failure.
fn forgive_html_argv0_drift(name: &str, outcome: &mut GroupedOutcome) {
    if outcome.ansi_match != Some(true)
        || outcome.txt_match != Some(true)
        || outcome.html_match != Some(false)
        || !matches!(outcome.outcome.status, Status::PixelsDiffer)
        || !outcome
            .outcome
            .pixel_score
            .is_some_and(|score| score >= 1.0)
        || !note_has_only_html_finding(&outcome.outcome.note)
    {
        return;
    }
    let approved_html = std::fs::read(&outcome.approved.html)
        .unwrap_or_else(|e| panic!("read {}: {e}", outcome.approved.html.display()));
    let actual_html = std::fs::read(&outcome.actual.html)
        .unwrap_or_else(|e| panic!("read {}: {e}", outcome.actual.html.display()));
    if !html_equal_modulo_argv0(&approved_html, &actual_html) {
        return;
    }
    let note = normalized_html_note(&outcome.outcome.note);
    outcome.html_match = Some(true);
    outcome.outcome.status = Status::Matched;
    outcome.outcome.note = note.clone();
    rewrite_sealed_verdict_for_argv0(name, outcome, &note);
}

/// Verdict note for a normalized HTML match: drop the stale byte-gate
/// finding, keep any other finding, record the normalization.
fn normalized_html_note(previous: &str) -> String {
    const NORMALIZATION: &str =
        "html matches with provenance.argv0 normalized (volatile bless path ignored)";
    let mut kept: Vec<&str> = previous
        .split("; ")
        .filter(|segment| !segment.starts_with("html differs"))
        .filter(|segment| !is_html_byte_count_fragment(segment))
        .filter(|segment| !segment.is_empty())
        .collect();
    kept.push(NORMALIZATION);
    kept.join("; ")
}

/// `approved <n> bytes, actual <m> bytes`: the tail of the HTML byte-gate
/// finding (the first-difference detail joins it with `"; "`, so it
/// survives a naive split on the finding prefix). ASCII digits only;
/// anything else is kept as evidence.
fn is_html_byte_count_fragment(segment: &str) -> bool {
    let Some(rest) = segment.strip_prefix("approved ") else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(" bytes") else {
        return false;
    };
    match rest.split_once(" bytes, actual ") {
        Some((approved, actual)) => {
            !approved.is_empty()
                && !actual.is_empty()
                && approved.bytes().all(|b| b.is_ascii_digit())
                && actual.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

/// `true` iff the bundle note records the HTML byte gate as the sole
/// failing gate: every `"; "`-separated segment is the `"html differs"`
/// finding, an HTML byte-count tail (see [`is_html_byte_count_fragment`]),
/// or the trailing `state: ...` failure diagnostics
/// ([`compare_bundle_dirs`] appends that diagnostic block last on any
/// failure, so everything from the `state: ` segment on is diagnostics,
/// never a finding).
fn note_has_only_html_finding(note: &str) -> bool {
    let mut saw_html_finding = false;
    let mut in_diagnostics = false;
    for segment in note.split("; ") {
        if segment.starts_with("state: ") {
            in_diagnostics = true;
            continue;
        }
        if in_diagnostics {
            continue;
        }
        if segment == "html differs" {
            saw_html_finding = true;
        } else if !is_html_byte_count_fragment(segment) {
            return false;
        }
    }
    saw_html_finding
}

/// Correct the sealed `<name>.verdict.json` after argv0 forgiveness.
///
/// `check_with` already sealed the byte-exact verdict (`pixels-differ`,
/// `html_match: false`). Reports reuse a fresh sealed verdict verbatim, so
/// leaving it would keep the report red after the gate turned green. This
/// rewrites only the verdict fields (status, html_match, note, the
/// html-gate check label); artifact hashes stay untouched, so the verdict
/// stays fresh and reports show the normalized check. Any unexpected shape
/// panics (fail closed) instead of recording a half-corrected verdict.
///
/// CANDIDATE ADAPTATION. [`compare_bundle_dirs`] seals no verdict (the
/// outcome is returned in-memory), so a missing verdict file is the
/// expected shape here and there is nothing to correct; a present but
/// malformed verdict still panics (fail closed).
fn rewrite_sealed_verdict_for_argv0(name: &str, outcome: &GroupedOutcome, note: &str) {
    assert!(
        note.bytes().all(|b| b >= 0x20 && b != b'"' && b != b'\\'),
        "normalized note must be plain ASCII without quotes or backslashes: {note:?}"
    );
    let verdict_path = outcome.actual.html.with_extension("verdict.json");
    if !verdict_path.exists() {
        return;
    }
    let text = std::fs::read_to_string(&verdict_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", verdict_path.display()));
    let replace_once = |text: &str, from: &str, to: &str| -> String {
        assert_eq!(
            text.matches(from).count(),
            1,
            "verdict `{name}` lost its expected shape at `{from}` (fail closed)"
        );
        text.replacen(from, to, 1)
    };
    let text = replace_once(
        &text,
        "\"status\": \"pixels-differ\"",
        "\"status\": \"matched\"",
    );
    let text = replace_once(&text, "\"html_match\": false", "\"html_match\": true");
    let text = replace_once(
        &text,
        "\"html-byte-gate\"",
        "\"html-byte-gate+argv0-normalized\"",
    );
    let text = replace_verdict_note(&text, name, note);
    tuiscotti::snapshot::write_atomic(verdict_path.as_path(), text.as_bytes())
        .unwrap_or_else(|e| panic!("rewrite {}: {e}", verdict_path.display()));
    let reread = std::fs::read_to_string(&verdict_path)
        .unwrap_or_else(|e| panic!("re-read {}: {e}", verdict_path.display()));
    assert!(
        reread.contains("\"status\": \"matched\"")
            && reread.contains("\"html_match\": true")
            && reread.contains(note),
        "verdict `{name}` failed to record the normalized check (fail closed)"
    );
}

/// Replace the `"note"` value inside a sealed verdict document. The old
/// value is scanned as a JSON string (backslash escapes honored), so a
/// quote inside the old note cannot corrupt the document shape.
fn replace_verdict_note(text: &str, name: &str, note: &str) -> String {
    const KEY: &str = "\"note\": \"";
    let value_at = text
        .find(KEY)
        .unwrap_or_else(|| panic!("verdict `{name}` has no note (fail closed)"))
        + KEY.len();
    let bytes = text.as_bytes();
    let mut i = value_at;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                assert!(
                    i + 1 < bytes.len(),
                    "verdict `{name}` note is truncated (fail closed)"
                );
                i += 2;
            }
            b'"' => break,
            _ => i += 1,
        }
    }
    assert!(
        i < bytes.len(),
        "verdict `{name}` note is unterminated (fail closed)"
    );
    format!("{}{}{}", &text[..value_at], note, &text[i..])
}

/// Mask-test render: the candidate static HTML writer embeds no provenance
/// (no committed baseline render carries `provenance.argv`), so the fixture
/// embeds the documented shape — frame JSON with `provenance` last inside
/// one `<script type="application/json">` block — synthetically around a
/// realistic payload. The argv span sits past byte 1000 so the negative
/// control flips a byte outside it (enforced by assert in the test).
fn argv0_test_render() -> Vec<u8> {
    let mut html = String::from("<!DOCTYPE html><html><body><pre>");
    html.push_str(&"x".repeat(1100));
    html.push_str("</pre><script type=\"application/json\">");
    html.push_str(
        r#"{"cells":[],"provenance":{"tool":"tuiscotti","argv":["/repo/tests/harness/target/debug/jackin-preview","--scenario","first-use"]}}"#,
    );
    html.push_str("</script></body></html>");
    html.into_bytes()
}

#[test]
fn html_argv0_mask_replaces_only_the_argv0_value() {
    let html = argv0_test_render();
    let (start, end) = html_argv0_value_span(&html).expect("argv0 span exists");
    assert!(end > start, "argv0 value is non-empty");
    assert!(
        html[start..end].contains(&b'/'),
        "argv0 is an absolute path"
    );
    let original = html[start..end].to_vec();
    let masked = mask_html_argv0(&html).expect("mask applies");
    assert_eq!(&masked[..start], &html[..start], "prefix untouched");
    assert_eq!(
        &masked[start + NORMALIZED_ARGV0.len()..],
        &html[end..],
        "suffix untouched"
    );
    assert_eq!(
        &masked[start..start + NORMALIZED_ARGV0.len()],
        NORMALIZED_ARGV0.as_bytes(),
        "span holds the placeholder"
    );
    assert!(
        !masked
            .windows(original.len())
            .any(|window| window == original.as_slice()),
        "original bless path is gone"
    );
    let twice = mask_html_argv0(&masked).expect("mask applies twice");
    assert_eq!(masked, twice, "masking is stable");
}

#[test]
fn html_argv0_mask_ignores_worktree_drift() {
    let approved = argv0_test_render();
    // Simulate a rerun from another worktree: same render, other argv0.
    let (start, end) = html_argv0_value_span(&approved).expect("argv0 span exists");
    let mut rerun = approved.clone();
    rerun.splice(
        start..end,
        b"/tmp/other-worktree/tests/harness/target/debug/jackin-preview"
            .iter()
            .copied(),
    );
    assert_ne!(approved, rerun, "raw bytes still differ");
    assert!(
        html_equal_modulo_argv0(&approved, &rerun),
        "masked compare ignores argv0 drift"
    );
}

#[test]
fn html_argv0_mask_detects_any_other_byte() {
    let approved = argv0_test_render();
    let (span_start, _) = html_argv0_value_span(&approved).expect("argv0 span exists");
    // NEGATIVE control 1: one flipped byte in the payload region.
    let mut corrupted = approved.clone();
    let flip_at = 1000;
    assert!(flip_at < span_start, "flip sits outside the masked span");
    corrupted[flip_at] ^= 0x01;
    assert!(
        !html_equal_modulo_argv0(&approved, &corrupted),
        "a non-provenance byte still fails the gate"
    );
    // NEGATIVE control 2: one changed byte in argv[1] (only argv[0] is masked).
    let mut flag_change = approved.clone();
    let argv1_start = {
        let (_, argv0_end) = html_argv0_value_span(&approved).expect("argv0 span exists");
        let mut i = argv0_end + 1; // skip argv0 closing quote
        assert_eq!(approved[i], b',', "argv continues past argv0");
        i += 1;
        while approved[i].is_ascii_whitespace() {
            i += 1;
        }
        assert_eq!(approved[i], b'"', "argv[1] opens");
        i + 1
    };
    flag_change[argv1_start] ^= 0x01;
    assert!(
        !html_equal_modulo_argv0(&approved, &flag_change),
        "argv[1] stays gated"
    );
}

#[test]
fn html_normalized_note_drops_the_stale_finding() {
    let previous = "html differs; state: dims expected 120x40 vs actual 120x40; digest expected aaa vs actual aaa; cursor expected none vs actual none; 0 differing cell(s)";
    assert_eq!(
        normalized_html_note(previous),
        "state: dims expected 120x40 vs actual 120x40; digest expected aaa vs actual aaa; cursor expected none vs actual none; 0 differing cell(s); html matches with provenance.argv0 normalized (volatile bless path ignored)"
    );
    // Unknown findings are kept as evidence, never dropped.
    let other = "png dimensions differ: approved (1, 2), actual (3, 4)";
    assert!(normalized_html_note(other).starts_with(other));
}

#[test]
fn html_forgiveness_requires_the_html_gate_to_fail_alone() {
    let html_only = "html differs; state: dims expected 120x40 vs actual 120x40; digest expected aaa vs actual aaa; cursor expected none vs actual none; 0 differing cell(s)";
    assert!(
        note_has_only_html_finding(html_only),
        "html-only failure is forgivable"
    );
    // A genuine fidelity delta beside the html finding blocks forgiveness.
    let with_fidelity = "html differs; png.fidelity.json differs: pixels.above_threshold: 3 vs 0; state: dims expected 120x40 vs actual 120x40; digest expected aaa vs actual bbb; cursor expected none vs actual none; 0 differing cell(s)";
    assert!(
        !note_has_only_html_finding(with_fidelity),
        "fidelity drift is never forgiven"
    );
    // No html finding at all: nothing to forgive.
    assert!(!note_has_only_html_finding("txt differs"));
    assert!(!note_has_only_html_finding(""));
}

#[test]
fn html_argv0_span_rejects_empty_or_absent_argv() {
    assert_eq!(
        html_argv0_value_span(br#"{"provenance":{"argv":[]}}"#),
        None
    );
    assert_eq!(html_argv0_value_span(b"no json here"), None);
    assert_eq!(html_argv0_value_span(br#"{"argv":["unterminated}"#), None);
    // An escaped quote inside the value does not end the span early.
    let escaped = br#"{"argv":["a\"b"]}"#;
    let (start, end) = html_argv0_value_span(escaped).expect("span");
    assert_eq!(&escaped[start..end], b"a\\\"b");
}

/// Fail-closed assertion: only `matched` passes. Missing approval remains
/// pending after capture, but the test fails until the full suite is
/// generated and explicitly blessed (`tuiscotti accept --grouped` is the only
/// bless, never the test); drift, dimension mismatch and corrupt approval
/// also fail.
pub fn assert_gated(outcome: &GroupedOutcome) {
    match outcome.status() {
        Status::Matched => eprintln!("baseline matched   {}", outcome.outcome.name),
        Status::MissingApproval => panic!(
            "baseline missing approval: {} (generate, review, then bless explicitly)",
            outcome.outcome.name
        ),
        _ => panic!("{}", outcome.ensure_matched().unwrap_err()),
    }
}

/// A ported-matrix capture running through the PTY session.
pub fn run_and_assert(case: &Case) {
    let mut session = spawn_boot(case);
    settle_and_gate_with_timeout(&mut session, &case.name, case.timeout_ms);
}

/// Expand one representative static Case::new root through the full canonical
/// matrix. The representative's sends/timeout apply to every combo; choose a
/// representative whose determinism contract is size-independent.
pub fn run_canonical(representative: &Case) {
    let mut failures = Vec::new();
    let mut executed = 0usize;
    let mut skipped = 0usize;
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = representative.variant(cols, rows, color);
            let name = case.name.to_string();
            if let Ok(filter) = std::env::var("COMBO_FILTER") {
                if !name.contains(&filter) {
                    skipped += 1;
                    continue;
                }
            }
            executed += 1;
            if !collect_matrix(&name, || run_and_assert(&case)) {
                failures.push(name);
            }
        }
    }
    finish_matrix_accounted(&failures, executed, skipped);
}

/// Expand one representative while preserving exact legacy declarations.
///
/// A root may have had distinct settings at specific old size/color combos
/// (for example, a larger boot frame or an extra readiness wait). Those full
/// declarations are passed explicitly and win for their exact combo; every
/// combo without an old declaration inherits the representative. Duplicate
/// declarations for one combo are a configuration conflict, not a precedence
/// rule.
pub fn run_canonical_with_variants(representative: &Case, variants: &[Case]) {
    let root = canonical_root(&representative.name).unwrap_or_else(|| {
        panic!(
            "`{}` is not a canonical `<root>/<size>/<color>` capture",
            representative.name
        )
    });
    let mut selected = std::collections::BTreeMap::new();
    for variant in variants {
        let variant_root = canonical_root(&variant.name).unwrap_or_else(|| {
            panic!(
                "variant `{}` is not a canonical `<root>/<size>/<color>` capture",
                variant.name
            )
        });
        assert!(
            variant_root == root,
            "variant `{variant_root}` does not belong to representative root `{root}`"
        );
        let expected_name = canonical_name(variant_root, variant.cols, variant.rows, variant.color);
        assert!(
            variant.name.as_ref() == expected_name,
            "variant `{}` conflicts with its declared {}/{}/{} combo",
            variant.name,
            variant.cols,
            variant.rows,
            variant.color.suffix()
        );
        let key = (variant.cols, variant.rows, variant.color.suffix());
        assert!(
            selected.insert(key, variant).is_none(),
            "conflicting legacy declarations for `{root}` at {}/{}/{}",
            variant.cols,
            variant.rows,
            variant.color.suffix()
        );
    }

    let mut failures = Vec::new();
    let mut executed = 0usize;
    let mut skipped = 0usize;
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = selected
                .get(&(cols, rows, color.suffix()))
                .copied()
                .unwrap_or(representative)
                .variant(cols, rows, color);
            let name = case.name.to_string();
            if let Ok(filter) = std::env::var("COMBO_FILTER") {
                if !name.contains(&filter) {
                    skipped += 1;
                    continue;
                }
            }
            executed += 1;
            if !collect_matrix(&name, || run_and_assert(&case)) {
                failures.push(name);
            }
        }
    }
    finish_matrix_accounted(&failures, executed, skipped);
}

/// Expand a live pointer/keyboard/manual-flow root through the same matrix.
/// `interact` runs after the centrally driven boot + case sends and before the
/// centrally settled/gated capture.
pub fn run_canonical_live(representative: &Case, mut interact: impl FnMut(&mut Session, &Case)) {
    let mut failures = Vec::new();
    let mut executed = 0usize;
    let mut skipped = 0usize;
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = representative.variant(cols, rows, color);
            let name = case.name.to_string();
            if let Ok(filter) = std::env::var("COMBO_FILTER") {
                if !name.contains(&filter) {
                    skipped += 1;
                    continue;
                }
            }
            executed += 1;
            if !collect_matrix(&name, || {
                let mut session = spawn_boot(&case);
                interact(&mut session, &case);
                settle_and_gate_with_timeout(&mut session, &case.name, case.timeout_ms);
            }) {
                failures.push(name);
            }
        }
    }
    finish_matrix_accounted(&failures, executed, skipped);
}

/// Expand a representative live root, substituting a compact send chain for
/// terminal widths at or below `max_cols`. This is for responsive layouts
/// that need an explicit drawer/detail step which the wide representative's
/// sends cannot express; coverage remains the full canonical 5×5 matrix.
pub fn run_canonical_live_with_compact_sends(
    representative: &Case,
    max_cols: u16,
    compact_sends: &'static [&'static str],
    mut interact: impl FnMut(&mut Session, &Case),
) {
    let mut failures = Vec::new();
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let mut case = representative.variant(cols, rows, color);
            if cols <= max_cols {
                case.sends = compact_sends;
            }
            let name = case.name.to_string();
            if !collect_matrix(&name, || {
                let mut session = spawn_boot(&case);
                interact(&mut session, &case);
                settle_and_gate_with_timeout(&mut session, &case.name, case.timeout_ms);
            }) {
                failures.push(name);
            }
        }
    }
    finish_matrix(&failures);
}

/// Run `body` for each combo of a data-driven matrix without stopping at the
/// first failure: every combo's actuals are written before the fn panics, so
/// an intentional-change run regenerates the whole matrix in one pass. The
/// fn still fails loudly, with every failed combo named.
pub fn collect_matrix(combo: &str, body: impl FnOnce()) -> bool {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
        Ok(()) => true,
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".into());
            eprintln!("matrix combo FAILED {combo}: {msg}");
            false
        }
    }
}

/// Pure acceptance verdict over one matrix run: failures, required
/// completeness (executed vs skipped-by-filter), and execution-subject
/// integrity (mid-test-built binaries poison acceptance). `Err` carries the
/// failure message; filtered or hatch-built runs pass only when explicitly
/// flagged via [`PARTIAL_RUN_ENV`] (non-acceptance, never parity evidence).
pub fn acceptance_verdict(
    failures: &[String],
    executed: usize,
    skipped: usize,
    partial_allowed: bool,
    midtest_build_used: bool,
) -> Result<(), String> {
    if !failures.is_empty() {
        return Err(format!(
            "{} matrix capture(s) failed: {}",
            failures.len(),
            failures.join(", ")
        ));
    }
    if skipped > 0 && !partial_allowed {
        let required = executed + skipped;
        let filter = std::env::var("COMBO_FILTER").unwrap_or_else(|_| "-".to_string());
        return Err(format!(
            "matrix incomplete: executed {executed}/{required} required, skipped {skipped} by COMBO_FILTER=`{filter}`; \
             a filtered run cannot pass acceptance. Re-run without COMBO_FILTER, or set {PARTIAL_RUN_ENV}=1 \
             for an explicitly partial/diagnostic (non-acceptance) run."
        ));
    }
    if midtest_build_used && !partial_allowed {
        return Err(format!(
            "execution subject unbound: a subject binary was built mid-test via {MIDTEST_BUILD_ENV}=1 instead of \
             from the reviewed commit; this run cannot pass acceptance. Build the reviewed tree first and re-run, \
             or set {PARTIAL_RUN_ENV}=1 for an explicitly partial/diagnostic (non-acceptance) run."
        ));
    }
    Ok(())
}

/// Panic if any [`collect_matrix`] call reported a failure, or if a mid-test
/// build poisoned this run's execution subject without the partial-run flag.
pub fn finish_matrix(failures: &[String]) {
    if let Err(msg) =
        acceptance_verdict(failures, 0, 0, partial_run_allowed(), midtest_build_used())
    {
        panic!("{msg}");
    }
}

/// [`finish_matrix`] plus required-completeness accounting: records
/// executed/skipped-vs-required counts and fails the acceptance verdict when
/// a case filter skipped combos without the partial-run opt-in. No silent
/// subset pass.
pub fn finish_matrix_accounted(failures: &[String], executed: usize, skipped: usize) {
    let required = executed + skipped;
    eprintln!("matrix completeness: executed {executed}/{required} required (skipped {skipped})");
    if skipped > 0 && partial_run_allowed() {
        eprintln!(
            "NON-ACCEPTANCE: partial run executed {executed}/{required} required combos; \
             results must not be cited as parity evidence."
        );
    }
    if let Err(msg) = acceptance_verdict(
        failures,
        executed,
        skipped,
        partial_run_allowed(),
        midtest_build_used(),
    ) {
        panic!("{msg}");
    }
}

/// One `#[test]` per canonical root, generated from the representative static
/// case tables so cargo name filters work. The test fn name comes from the
/// representative capture; `run_canonical` expands `Case.name`'s root to all
/// 5 sizes × 5 colours.
#[macro_export]
macro_rules! baseline_case {
    ($fn_name:ident => $case:expr) => {
        #[test]
        #[ignore = "visual baseline capture; run with --ignored"]
        fn $fn_name() {
            $crate::support::run_canonical(&$case);
        }
    };
}

/// One test per canonical root, with exact declarations for legacy combos.
#[macro_export]
macro_rules! baseline_case_with_variants {
    ($fn_name:ident => $case:expr, [$($variant:expr),* $(,)?] $(,)?) => {
        #[test]
        #[ignore = "visual baseline capture; run with --ignored"]
        fn $fn_name() {
            let variants = [$($variant),*];
            $crate::support::run_canonical_with_variants(&$case, &variants);
        }
    };
}

#[cfg(test)]
mod execution_subject_tests {
    use super::*;

    /// sha256("abc"), NIST vector.
    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn verdict_full_matrix_ok() {
        assert!(acceptance_verdict(&[], 25, 0, false, false).is_ok());
    }

    #[test]
    fn verdict_failures_listed() {
        let failures = vec!["a/80x24/truecolor".to_string(), "b/80x24/16".to_string()];
        let err = acceptance_verdict(&failures, 25, 0, false, false).unwrap_err();
        assert!(err.contains("2 matrix capture(s) failed"), "{err}");
        assert!(err.contains("a/80x24/truecolor"), "{err}");
        assert!(err.contains("b/80x24/16"), "{err}");
    }

    #[test]
    fn verdict_filtered_without_optin_fails() {
        let err = acceptance_verdict(&[], 3, 22, false, false).unwrap_err();
        assert!(err.contains("executed 3/25"), "{err}");
        assert!(err.contains("skipped 22"), "{err}");
        assert!(err.contains(PARTIAL_RUN_ENV), "{err}");
    }

    #[test]
    fn verdict_filtered_with_optin_ok() {
        assert!(acceptance_verdict(&[], 3, 22, true, false).is_ok());
    }

    #[test]
    fn verdict_failures_beat_partial_optin() {
        let failures = vec!["a/80x24/truecolor".to_string()];
        let err = acceptance_verdict(&failures, 3, 22, true, false).unwrap_err();
        assert!(err.contains("1 matrix capture(s) failed"), "{err}");
    }

    #[test]
    fn verdict_midtest_poison_without_optin_fails() {
        let err = acceptance_verdict(&[], 25, 0, false, true).unwrap_err();
        assert!(err.contains("unbound"), "{err}");
        assert!(err.contains(MIDTEST_BUILD_ENV), "{err}");
        assert!(err.contains(PARTIAL_RUN_ENV), "{err}");
    }

    #[test]
    fn verdict_midtest_poison_with_optin_ok() {
        assert!(acceptance_verdict(&[], 25, 0, true, true).is_ok());
    }

    #[test]
    fn digest_file_known_vector() {
        let probe = tempfile::NamedTempFile::new().expect("temp probe");
        std::fs::write(probe.path(), b"abc").expect("write probe");
        let subject = digest_file(probe.path()).expect("digest probe");
        assert_eq!(subject.sha256, ABC_SHA256);
        assert_eq!(subject.len, 3);
        assert_eq!(subject.path, probe.path());
        assert!(subject.mtime_unix.is_some());
    }

    #[test]
    fn digest_file_missing_returns_none() {
        assert!(digest_file(Path::new("/tmp/termrock-q05-definitely-missing-probe")).is_none());
    }

    #[test]
    fn try_resolve_missing_errors_loudly_without_building() {
        unsafe { std::env::remove_var("CARGO_BIN_EXE_termrockq05nosuchbin") };
        let before_debug = candidate_bins("termrockq05nosuchbin");
        assert!(
            before_debug.iter().all(|p| !p.exists()),
            "probe precondition: no such binary on disk"
        );
        let err = try_resolve_bin("termrockq05nosuchbin").unwrap_err();
        assert!(err.contains("termrockq05nosuchbin"), "{err}");
        assert!(err.contains("target/debug"), "{err}");
        assert!(err.contains("target/release"), "{err}");
        assert!(err.contains(MIDTEST_BUILD_ENV), "{err}");
        assert!(
            candidate_bins("termrockq05nosuchbin")
                .iter()
                .all(|p| !p.exists()),
            "try_resolve_bin must never build: no binary may appear as a side effect"
        );
    }

    #[test]
    fn try_resolve_binds_digest_to_executed_path() {
        let probe = tempfile::NamedTempFile::new().expect("temp probe");
        std::fs::write(probe.path(), b"abc").expect("write probe");
        unsafe {
            std::env::set_var(
                "CARGO_BIN_EXE_termrockq05digestprobe",
                probe.path().as_os_str(),
            )
        };
        let subject = try_resolve_bin("termrockq05digestprobe").expect("resolve probe");
        unsafe { std::env::remove_var("CARGO_BIN_EXE_termrockq05digestprobe") };
        assert_eq!(subject.path, probe.path());
        assert_eq!(subject.sha256, ABC_SHA256);
        assert_eq!(subject.len, 3);
    }

    #[test]
    fn resolve_bin_missing_panics_without_hatch() {
        if midtest_build_allowed() {
            eprintln!("{MIDTEST_BUILD_ENV}=1 set; skipping panic-behavior probe");
            return;
        }
        unsafe { std::env::remove_var("CARGO_BIN_EXE_termrockq05nosuchbinresolve") };
        let caught = std::panic::catch_unwind(|| {
            resolve_bin("termrockq05nosuchbinresolve");
        });
        let panic_value = caught.expect_err("resolve_bin must panic on a missing binary");
        let msg = panic_value
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic_value.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains("termrockq05nosuchbinresolve"), "{msg}");
        assert!(msg.contains(MIDTEST_BUILD_ENV), "{msg}");
    }
}

/// Q06 (FIX-012 A1/A5): full-bundle gate + first-N diagnostics.
///
/// All fixtures are synthetic tempdir bundles built with the real exporter
/// (`write_10_artifact_bundle`); nothing here reads or writes `baselines/`
/// or any real expected-output file.
#[cfg(test)]
mod bundle_gate_tests {
    use super::*;
    use tuiscotti::Provenance;

    const PROBE_NAME: &str = "q06/probe/16x8/truecolor";

    fn probe_provenance() -> Provenance {
        Provenance {
            tool: "tuiscotti".to_string(),
            tool_version: "0.2.0".to_string(),
            profile: "default".to_string(),
            source: "screen".to_string(),
            argv: Vec::new(),
            created_unix: 0,
        }
    }

    fn plain_cell(x: u16, y: u16, symbol: &str) -> Cell {
        Cell {
            x,
            y,
            symbol: symbol.to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: Mods::default(),
            underline_color: tuiscotti::Color::Default,
        }
    }

    /// Small deterministic frame with styled + non-ASCII content (exercises
    /// the glyph, color, modifier, and ascii-substitution paths).
    fn probe_frame_a() -> Frame {
        let mut frame = Frame::blank(16, 8, probe_provenance());
        frame.set(Cell {
            x: 2,
            y: 1,
            symbol: "A".to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Indexed(1),
            bg: tuiscotti::Color::Default,
            mods: Mods {
                bold: true,
                ..Mods::default()
            },
            underline_color: tuiscotti::Color::Default,
        });
        frame.set(plain_cell(5, 2, "─"));
        frame.set(plain_cell(6, 2, "❯"));
        frame
    }

    struct BundlePair {
        _tmp: tempfile::TempDir,
        actual_root: PathBuf,
        approved_root: PathBuf,
        diff_root: PathBuf,
    }

    fn write_pair(approved_frame: &Frame, actual_frame: &Frame) -> BundlePair {
        let tmp = tempfile::tempdir().expect("tempdir");
        let actual_root = tmp.path().join("actual");
        let approved_root = tmp.path().join("approved");
        let diff_root = tmp.path().join("diff");
        let mut renderer = Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse");
        write_10_artifact_bundle(&approved_root, PROBE_NAME, approved_frame, &mut renderer)
            .expect("write approved bundle");
        write_10_artifact_bundle(&actual_root, PROBE_NAME, actual_frame, &mut renderer)
            .expect("write actual bundle");
        BundlePair {
            _tmp: tmp,
            actual_root,
            approved_root,
            diff_root,
        }
    }

    fn compare(pair: &BundlePair, actual_frame: &Frame) -> GroupedOutcome {
        compare_bundle_dirs(
            PROBE_NAME,
            actual_frame,
            &pair.actual_root,
            &pair.approved_root,
            &pair.diff_root,
        )
        .expect("compare bundles")
    }

    fn artifact_bytes(root: &Path, ext: &str) -> Vec<u8> {
        std::fs::read(root.join(format!("{PROBE_NAME}.{ext}"))).expect("read artifact")
    }

    /// Rewrite one artifact file, then re-pin its manifest entry (bytes +
    /// sha256) so the bundle stays internally consistent: the gate must fail
    /// on the cross-side difference, not on the pin.
    fn rewrite_pinned(root: &Path, key: &str, ext: &str, bytes: &[u8]) {
        let path = root.join(format!("{PROBE_NAME}.{ext}"));
        std::fs::write(&path, bytes).expect("rewrite artifact");
        let manifest_path = root.join(format!("{PROBE_NAME}.manifest.json"));
        let manifest_bytes = std::fs::read(&manifest_path).expect("read manifest");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&manifest_bytes).expect("parse manifest");
        let entry = manifest
            .get_mut("artifacts")
            .and_then(|artifacts| artifacts.get_mut(key))
            .expect("manifest entry");
        entry["sha256"] = serde_json::Value::String(sha256_hex(bytes));
        entry["bytes"] = serde_json::Value::Number(serde_json::Number::from(bytes.len()));
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("encode manifest"),
        )
        .expect("write manifest");
    }

    fn rewrite_json_pinned(
        root: &Path,
        key: &str,
        ext: &str,
        patch: impl Fn(&mut serde_json::Value),
    ) {
        let mut value: serde_json::Value =
            serde_json::from_slice(&artifact_bytes(root, ext)).expect("parse artifact json");
        patch(&mut value);
        let bytes = serde_json::to_vec_pretty(&value).expect("encode patched json");
        rewrite_pinned(root, key, ext, &bytes);
    }

    fn flip_first_byte_and_repin(pair: &BundlePair, key: &str, ext: &str) {
        let mut bytes = artifact_bytes(&pair.actual_root, ext);
        assert!(
            !bytes.is_empty(),
            "fixture artifact {ext} must be non-empty"
        );
        bytes[0] ^= 0xff;
        rewrite_pinned(&pair.actual_root, key, ext, &bytes);
    }

    #[test]
    fn identical_bundles_match() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        let outcome = compare(&pair, &frame);
        assert!(
            outcome.matched(),
            "identical bundles must match: {}",
            outcome.outcome.note
        );
        assert_eq!(outcome.outcome.cell_diff_total, 0);
        assert!(outcome.outcome.cell_diffs.is_empty());
        assert_eq!(outcome.outcome.pixel_score, Some(1.0));
        assert_eq!(outcome.ansi_match, Some(true));
        assert_eq!(outcome.txt_match, Some(true));
        assert_eq!(outcome.html_match, Some(true));
    }

    #[test]
    fn observations_volatile_fields_ignored() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        rewrite_json_pinned(
            &pair.actual_root,
            "observations_json",
            "observations.json",
            |value| {
                value["provenance"]["created_unix"] = serde_json::json!(1_700_000_000u64);
                value["provenance"]["argv"] = serde_json::json!(["/machine/specific/bin"]);
                value["legacy_name"] = serde_json::json!("ignored-alias");
            },
        );
        let outcome = compare(&pair, &frame);
        assert!(
            outcome.matched(),
            "volatile observations fields must not fail the gate: {}",
            outcome.outcome.note
        );
    }

    #[test]
    fn txt_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        flip_first_byte_and_repin(&pair, "txt", "txt");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("txt differs"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn ansi_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        flip_first_byte_and_repin(&pair, "ansi", "ansi");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("ansi differs"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn ascii_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        flip_first_byte_and_repin(&pair, "ascii", "ascii");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("ascii differs"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn html_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        flip_first_byte_and_repin(&pair, "html", "html");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::PixelsDiffer);
        assert!(
            outcome.outcome.note.contains("html differs"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn png_mismatch_detected_with_diff_artifact() {
        let approved = probe_frame_a();
        let mut other = probe_frame_a();
        other.set(plain_cell(9, 6, "Z"));
        let pair = write_pair(&approved, &approved);
        // Splice in a valid-but-different PNG rendered from another frame.
        let scratch = write_pair(&other, &other);
        let other_png = artifact_bytes(&scratch.approved_root, "png");
        rewrite_pinned(&pair.actual_root, "png", "png", &other_png);
        let outcome = compare(&pair, &approved);
        assert_eq!(outcome.status(), Status::PixelsDiffer);
        assert!(
            outcome.outcome.note.contains("pixel similarity"),
            "{}",
            outcome.outcome.note
        );
        assert!(
            outcome.outcome.pixel_score.is_some_and(|score| score < 1.0),
            "pixel score must record the sub-1.0 verdict"
        );
        assert!(
            pair.diff_root.join(format!("{PROBE_NAME}.png")).is_file(),
            "pixel mismatch must write a diff png"
        );
    }

    #[test]
    fn frame_cell_mismatch_reports_first_n_with_coordinates() {
        let approved = probe_frame_a();
        let mut actual = probe_frame_a();
        // Glyph change at (2,1), keeping the bold red style.
        let mut changed = actual.get(2, 1).cloned().expect("cell (2,1)");
        changed.symbol = "B".to_string();
        actual.set(changed);
        // Color-only change at (7,4): blank cell with a green foreground.
        actual.set(Cell {
            x: 7,
            y: 4,
            symbol: " ".to_string(),
            width: 1,
            continuation: false,
            fg: tuiscotti::Color::Indexed(2),
            bg: tuiscotti::Color::Default,
            mods: Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        let pair = write_pair(&approved, &actual);
        let outcome = compare(&pair, &actual);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("frame cells differ"),
            "{}",
            outcome.outcome.note
        );
        assert_eq!(outcome.outcome.cell_diff_total, 2);
        let coords: Vec<(u16, u16)> = outcome
            .outcome
            .cell_diffs
            .iter()
            .map(|diff| (diff.x, diff.y))
            .collect();
        assert_eq!(coords, vec![(2, 1), (7, 4)], "row-major first-N order");
        let glyph_diff = &outcome.outcome.cell_diffs[0];
        assert!(
            glyph_diff.expected.contains("\"A\"") && glyph_diff.actual.contains("\"B\""),
            "expected vs actual glyphs must be reported: {glyph_diff:?}"
        );
        assert!(
            glyph_diff.actual.contains("+bold"),
            "modifiers must be reported: {glyph_diff:?}"
        );
        // The failure itself must carry the diagnostic, not just a verdict.
        let message = outcome.ensure_matched().unwrap_err().to_string();
        assert!(
            message.contains("2 differing cell(s), first 2"),
            "{message}"
        );
        assert!(message.contains("(2,1)"), "{message}");
        assert!(message.contains("(7,4)"), "{message}");
        assert!(message.contains("\"A\""), "{message}");
        assert!(message.contains("\"B\""), "{message}");
    }

    #[test]
    fn cell_diagnostic_cap_respected() {
        let approved = Frame::blank(16, 8, probe_provenance());
        let mut actual = Frame::blank(16, 8, probe_provenance());
        for y in 0..8 {
            for x in 0..16 {
                actual.set(plain_cell(x, y, "Z"));
            }
        }
        let pair = write_pair(&approved, &actual);
        let outcome = compare(&pair, &actual);
        assert_eq!(outcome.outcome.cell_diff_total, 128);
        assert_eq!(outcome.outcome.cell_diffs.len(), MAX_CELL_DIFFS);
        assert!(
            outcome.outcome.cell_diffs.len() < outcome.outcome.cell_diff_total,
            "stored diagnostics must be capped while the total stays exact"
        );
        let first = &outcome.outcome.cell_diffs[0];
        assert_eq!((first.x, first.y), (0, 0), "first-N starts row-major");
    }

    #[test]
    fn cursor_only_drift_reported_as_state() {
        let approved = probe_frame_a();
        let mut actual = probe_frame_a();
        actual.cursor.x = 4;
        actual.cursor.y = 5;
        actual.cursor.visible = true;
        let pair = write_pair(&approved, &actual);
        let outcome = compare(&pair, &actual);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert_eq!(outcome.outcome.cell_diff_total, 1);
        assert_eq!(outcome.outcome.cell_diffs.len(), 1);
        let diff = &outcome.outcome.cell_diffs[0];
        assert!(
            diff.expected.contains("cursor") && diff.actual.contains("cursor"),
            "cursor-only drift must report cursor state, not identical cell text: {diff:?}"
        );
        assert!(
            outcome.outcome.note.contains("cursor"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn modifier_and_continuation_in_diagnostics() {
        let approved = probe_frame_a();
        let mut actual = probe_frame_a();
        // Modifier-only change at (2,1): bold red 'A' becomes dim
        // double-underlined 'A'.
        let mut changed = actual.get(2, 1).cloned().expect("cell (2,1)");
        changed.mods = Mods {
            dim: true,
            underline: true,
            underline_style: UnderlineStyle::Double,
            ..Mods::default()
        };
        actual.set(changed);
        // Wide glyph at (10,3) with its continuation at (11,3).
        actual.set(Cell {
            x: 10,
            y: 3,
            symbol: "漢".to_string(),
            width: 2,
            continuation: false,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        actual.set(Cell {
            x: 11,
            y: 3,
            symbol: String::new(),
            width: 0,
            continuation: true,
            fg: tuiscotti::Color::Default,
            bg: tuiscotti::Color::Default,
            mods: Mods::default(),
            underline_color: tuiscotti::Color::Default,
        });
        let pair = write_pair(&approved, &actual);
        let outcome = compare(&pair, &actual);
        assert_eq!(outcome.outcome.cell_diff_total, 3);
        let by_coord: HashMap<(u16, u16), &CellDiff> = outcome
            .outcome
            .cell_diffs
            .iter()
            .map(|diff| ((diff.x, diff.y), diff))
            .collect();
        let modifier_diff = by_coord[&(2, 1)];
        assert!(
            modifier_diff.actual.contains("+dim") && modifier_diff.actual.contains("+ul2"),
            "modifier tokens must be reported: {modifier_diff:?}"
        );
        let wide_diff = by_coord[&(10, 3)];
        assert!(
            wide_diff.actual.contains("漢"),
            "wide glyph must be reported: {wide_diff:?}"
        );
        let continuation_diff = by_coord[&(11, 3)];
        assert_eq!(
            continuation_diff.actual, "…",
            "continuation cells must be reported as continuations: {continuation_diff:?}"
        );
    }

    #[test]
    fn actual_frame_json_skew_is_capture_incomplete() {
        let approved = probe_frame_a();
        let mut live = probe_frame_a();
        live.set(plain_cell(1, 1, "Q"));
        // Actual files describe A while the gated live frame is different.
        let pair = write_pair(&approved, &approved);
        let outcome = compare(&pair, &live);
        assert_eq!(outcome.status(), Status::CaptureIncomplete);
        assert!(
            outcome.outcome.note.contains("skews from the gated frame"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn loss_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        rewrite_json_pinned(
            &pair.actual_root,
            "ascii_loss_json",
            "ascii.loss.json",
            |value| {
                let count = value["substitutions_count"].as_u64().expect("count");
                value["substitutions_count"] = serde_json::json!(count + 1);
            },
        );
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("ascii.loss.json differs"),
            "{}",
            outcome.outcome.note
        );
        assert!(
            outcome.outcome.note.contains("substitutions_count"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn fidelity_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        rewrite_json_pinned(
            &pair.actual_root,
            "png_fidelity_json",
            "png.fidelity.json",
            |value| {
                value["approximate"] = serde_json::json!(true);
            },
        );
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::PixelsDiffer);
        assert!(
            outcome.outcome.note.contains("png.fidelity.json differs"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn observations_mismatch_detected() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        rewrite_json_pinned(
            &pair.actual_root,
            "observations_json",
            "observations.json",
            |value| {
                value["cursor"]["x"] = serde_json::json!(7u64);
            },
        );
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CellsDiffer);
        assert!(
            outcome.outcome.note.contains("observations.json differs"),
            "{}",
            outcome.outcome.note
        );
        assert!(
            outcome.outcome.note.contains("cursor.x"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn approved_pin_mismatch_is_corrupt_approval() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        // Tamper without re-pinning: the pin must catch it.
        let path = pair.approved_root.join(format!("{PROBE_NAME}.txt"));
        let mut bytes = std::fs::read(&path).expect("read approved txt");
        bytes[0] ^= 0xff;
        std::fs::write(&path, bytes).expect("tamper approved txt");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CorruptApproval);
        assert!(
            outcome.outcome.note.contains("manifest pin"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn actual_pin_mismatch_is_capture_incomplete() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        let path = pair.actual_root.join(format!("{PROBE_NAME}.txt"));
        let mut bytes = std::fs::read(&path).expect("read actual txt");
        bytes[0] ^= 0xff;
        std::fs::write(&path, bytes).expect("tamper actual txt");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::CaptureIncomplete);
        assert!(
            outcome.outcome.note.contains("manifest pin"),
            "{}",
            outcome.outcome.note
        );
    }

    #[test]
    fn missing_approved_artifact_is_missing_approval() {
        let frame = probe_frame_a();
        let pair = write_pair(&frame, &frame);
        std::fs::remove_file(pair.approved_root.join(format!("{PROBE_NAME}.html")))
            .expect("remove approved html");
        let outcome = compare(&pair, &frame);
        assert_eq!(outcome.status(), Status::MissingApproval);
        assert!(
            outcome.outcome.note.contains("missing approved"),
            "{}",
            outcome.outcome.note
        );
        assert!(
            outcome.outcome.note.contains("html"),
            "{}",
            outcome.outcome.note
        );
    }
}
