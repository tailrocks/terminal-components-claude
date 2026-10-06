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

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
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
use tuiscotti::snapshot::Status;
pub use tuiscotti::tui::MouseButton;
use tuiscotti::tui::Tui;
use tuiscotti::tui::{CancelToken, MouseMods, Wheel};
pub use tuiscotti::{Frame, Profile, Renderer, VENDORED_FACES};

pub const SHOWCASE: &str = "showcase";
pub const TABLEPRO: &str = "tablepro";
pub const JACKIN: &str = "jackin-preview";
pub const HOLLA: &str = "holla";

/// SETTLE_MS from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// TIMEOUT_MS default; boot-streaming screens override it per capture
/// ([`Case::timeout`], the bash `CAP_TIMEOUT=` prefix).
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    Up,
    Down,
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

pub trait ScreenExt {
    fn find(&self, needle: &str) -> Option<(u16, u16)>;
    fn text(&self) -> String;
    fn size(&self) -> (u16, u16);
}

impl ScreenExt for tuiscotti::Screen {
    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        for y in 0..self.rows() {
            let mut row_str = String::with_capacity(self.cols() as usize);
            let mut col_offsets = Vec::with_capacity(self.cols() as usize);
            for x in 0..self.cols() {
                if let Some(cell) = self.get(x, y).filter(|c| !c.continuation) {
                    col_offsets.push((row_str.len(), x));
                    row_str.push_str(&cell.symbol);
                }
            }
            if let Some(byte_idx) = row_str.find(needle) {
                for &(b_idx, col) in col_offsets.iter().rev() {
                    if b_idx <= byte_idx {
                        return Some((y, col));
                    }
                }
                return Some((y, 0));
            }
        }
        None
    }

    fn text(&self) -> String {
        frame_from_screen(self, "default").text()
    }

    fn size(&self) -> (u16, u16) {
        (self.cols(), self.rows())
    }
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

    pub fn scroll(
        &mut self,
        col: u16,
        row: u16,
        scroll: Scroll,
    ) -> Result<(), tuiscotti::tui::TuiError> {
        let wheel = match scroll {
            Scroll::Up => Wheel::Up,
            Scroll::Down => Wheel::Down,
        };
        self.inner
            .mouse_wheel(wheel, col, row, MouseMods::default())
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
    let mut session = spawn(case);
    boot(&mut session, case.needle);
    if !case.sends.is_empty() {
        drive_with_timeout(&mut session, case.sends, case.timeout_ms);
    }
    session
}

pub fn boot(session: &mut Session, needle: &str) {
    if !needle.is_empty() {
        let deadline = std::time::Instant::now() + Duration::from_secs(8);
        let cancel = CancelToken::new();
        session
            .inner
            .wait_predicate(
                |obs| {
                    frame_from_screen(&obs.screen, "default")
                        .text()
                        .contains(needle)
                },
                deadline,
                &cancel,
            )
            .unwrap_or_else(|e| panic!("boot needle `{needle}` never appeared: {e:#}"));
        return;
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let cancel = CancelToken::new();
    session
        .inner
        .wait_stable_quiet(deadline, Duration::from_millis(200), &cancel)
        .unwrap_or_else(|e| panic!("boot idle failed: {e:#}"));
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

fn press_chord(inner: &tuiscotti::tui::Session, chord_str: &str) -> Result<(), tuiscotti::tui::TuiError> {
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

    let approved_ansi_bytes = match std::fs::read(&approved_paths.ansi) {
        Ok(b) => b,
        Err(_) => {
            return Ok(GroupedOutcome {
                outcome: tuiscotti::snapshot::CompareOutcome {
                    name: name.to_string(),
                    status: Status::MissingApproval,
                    cell_diffs: Vec::new(),
                    cell_diff_total: 0,
                    pixel_score: None,
                    approved_png_regenerated: false,
                    digest_expected: None,
                    digest_actual: frame.digest().to_string(),
                    actual_frame: actual_paths.frame_json.clone(),
                    actual_png: actual_paths.png.clone(),
                    expected_frame: approved_paths.frame_json.clone(),
                    expected_png: None,
                    expected_png_bytes: None,
                    diff_png: None,
                    note: "missing approved ansi".to_string(),
                },
                ansi_match: None,
                txt_match: None,
                html_match: None,
                actual: actual_paths,
                approved: approved_paths,
            });
        }
    };

    let approved_txt_bytes =
        std::fs::read(&approved_paths.txt).map_err(|e| format!("read approved txt: {e}"))?;
    let approved_png_bytes =
        std::fs::read(&approved_paths.png).map_err(|e| format!("read approved png: {e}"))?;
    let approved_html_bytes =
        std::fs::read(&approved_paths.html).map_err(|e| format!("read approved html: {e}"))?;
    let approved_frame_bytes = std::fs::read(&approved_paths.frame_json)
        .map_err(|e| format!("read approved frame.json: {e}"))?;

    let actual_ansi_bytes =
        std::fs::read(&actual_paths.ansi).map_err(|e| format!("read actual ansi: {e}"))?;
    let actual_txt_bytes =
        std::fs::read(&actual_paths.txt).map_err(|e| format!("read actual txt: {e}"))?;
    let actual_png_bytes =
        std::fs::read(&actual_paths.png).map_err(|e| format!("read actual png: {e}"))?;
    let actual_html_bytes =
        std::fs::read(&actual_paths.html).map_err(|e| format!("read actual html: {e}"))?;

    let ansi_matched = approved_ansi_bytes == actual_ansi_bytes;
    let txt_matched = approved_txt_bytes == actual_txt_bytes;
    let html_matched = approved_html_bytes == actual_html_bytes;

    let verdict = tuiscotti::diff::compare_png(&approved_png_bytes, &actual_png_bytes)
        .map_err(|e| format!("compare png: {e}"))?;

    let approved_frame: Frame = serde_json::from_slice(&approved_frame_bytes)
        .map_err(|e| format!("parse approved frame: {e}"))?;

    let mut status = Status::Matched;
    let mut notes = Vec::new();
    let mut diff_png = None;

    if !txt_matched {
        status = Status::CellsDiffer;
        notes.push("txt differs".to_string());
    } else if !ansi_matched {
        status = Status::CellsDiffer;
        notes.push("ansi differs".to_string());
    } else if approved_frame.digest() != frame.digest() {
        status = Status::CellsDiffer;
        notes.push("frame digest differs".to_string());
    }

    if !verdict.dims_equal {
        status = Status::DimensionMismatch;
        notes.push(format!(
            "dimensions differ: {:?} vs {:?}",
            verdict.expected_dims, verdict.actual_dims
        ));
    } else if verdict.score < 1.0 {
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

    Ok(GroupedOutcome {
        outcome: tuiscotti::snapshot::CompareOutcome {
            name: name.to_string(),
            status,
            cell_diffs: Vec::new(),
            cell_diff_total: 0,
            pixel_score: Some(verdict.score),
            approved_png_regenerated: false,
            digest_expected: Some(approved_frame.digest().to_string()),
            digest_actual: frame.digest().to_string(),
            actual_frame: actual_paths.frame_json.clone(),
            actual_png: actual_paths.png.clone(),
            expected_frame: approved_paths.frame_json.clone(),
            expected_png: Some(approved_paths.png.clone()),
            expected_png_bytes: Some(approved_png_bytes),
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

pub fn gate(name: &str, frame: &Frame) -> GroupedOutcome {
    let target_name = screen_first_path(name);
    RENDERER.with(|r| {
        let mut renderer = r.borrow_mut();
        check_baseline_bundle(&target_name, frame, &mut renderer)
            .unwrap_or_else(|e| panic!("gate `{target_name}` failed: {e}"))
    })
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
