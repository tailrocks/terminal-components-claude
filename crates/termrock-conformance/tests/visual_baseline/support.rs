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
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
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

pub const AUDIT_MATRIX_PREFIXES: [&str; 9] = [
    AUDIT_PREFIX_HOLLA_RUST,
    AUDIT_PREFIX_HOLLA_UPGRADE,
    AUDIT_PREFIX_JACKIN_CAPSULE,
    AUDIT_PREFIX_SHOWCASE_BUTTONS,
    AUDIT_PREFIX_SHOWCASE_DIFF,
    AUDIT_PREFIX_SHOWCASE_FORMS,
    AUDIT_PREFIX_SHOWCASE_INPUTS,
    AUDIT_PREFIX_SHOWCASE_TEXTAREAS,
    AUDIT_PREFIX_TABLEPRO_PRODUCTION,
];

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

/// Every capture name the suite produces: the canonical 5×5 expansion of each
/// Case::new root and the data-driven audit matrices.
pub fn suite_capture_names() -> BTreeSet<String> {
    let mut names = parse_case_new_names();
    names.extend(generated_matrix_names());
    names.into_iter().map(|n| screen_first_path(&n)).collect()
}

fn generated_matrix_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for prefix in AUDIT_MATRIX_PREFIXES {
        for &(cols, rows) in &CANONICAL_SIZES {
            for color in CANONICAL_COLORS {
                names.insert(audit_default_name(prefix, cols, rows, color));
            }
        }
    }
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            names.insert(audit_default_name(
                AUDIT_PREFIX_JACKIN_ACCOUNTS,
                cols,
                rows,
                color,
            ));
        }
    }
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

fn parse_case_new_names() -> BTreeSet<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/visual_baseline");
    let mut names = BTreeSet::new();
    let mut declared_roots = BTreeSet::new();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
            .path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if path.extension().and_then(|s| s.to_str()) != Some("rs")
            || file_name == "support.rs"
            || file_name == "main.rs"
        {
            continue;
        }
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        extract_case_new_roots(&src, &mut names, &mut declared_roots);
    }
    assert!(
        !names.is_empty(),
        "no Case::new names parsed from {}",
        dir.display()
    );
    names
}

fn extract_case_new_roots(
    src: &str,
    names: &mut BTreeSet<String>,
    declared_roots: &mut BTreeSet<String>,
) {
    const MACRO_MARK: &str = "crate::baseline_case";
    const MARK: &str = "Case::new(";
    let mut rest = src;
    while let Some(macro_i) = rest.find(MACRO_MARK) {
        extract_raw_case_new_roots(&rest[..macro_i], names, declared_roots);
        let invocation_end = baseline_invocation_end(&rest[macro_i..]);
        let invocation = &rest[macro_i..macro_i + invocation_end];
        let Some(case_i) = invocation.find(MARK) else {
            rest = &rest[macro_i + MACRO_MARK.len()..];
            continue;
        };
        let body = invocation[case_i + MARK.len()..].trim_start();
        let Some(body) = body.strip_prefix('"') else {
            panic!("baseline_case representative is missing its name literal");
        };
        let Some(end) = body.find('"') else {
            panic!("unterminated Case::new string literal");
        };
        let name = &body[..end];
        if let Some(root) = canonical_root(name) {
            assert!(
                declared_roots.insert(root.to_string()),
                "duplicate representative declaration for canonical root `{root}`"
            );
            for &(cols, rows) in &CANONICAL_SIZES {
                for color in CANONICAL_COLORS {
                    names.insert(canonical_name(root, cols, rows, color));
                }
            }
        } else {
            names.insert(name.to_string());
        }
        rest = &rest[macro_i + invocation_end..];
    }
    extract_raw_case_new_roots(rest, names, declared_roots);
}

fn baseline_invocation_end(src: &str) -> usize {
    let open = src.find('(').expect("baseline_case invocation missing `(`");
    let mut depth = 1;
    let mut in_string = false;
    let mut escaped = false;
    for (relative, byte) in src[open + 1..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == '\\' {
                escaped = true;
            } else if byte == '"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return open + 1 + relative + byte.len_utf8();
                }
            }
            _ => {}
        }
    }
    panic!("unterminated baseline_case invocation");
}

fn extract_raw_case_new_roots(
    src: &str,
    names: &mut BTreeSet<String>,
    declared_roots: &mut BTreeSet<String>,
) {
    const MARK: &str = "Case::new(";
    let mut rest = src;
    while let Some(i) = rest.find(MARK) {
        rest = rest[i + MARK.len()..].trim_start();
        let Some(body) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(end) = body.find('"') else {
            panic!("unterminated Case::new string literal");
        };
        let name = &body[..end];
        if let Some(root) = canonical_root(name) {
            assert!(
                declared_roots.insert(root.to_string()),
                "duplicate representative declaration for canonical root `{root}`"
            );
            for &(cols, rows) in &CANONICAL_SIZES {
                for color in CANONICAL_COLORS {
                    names.insert(canonical_name(root, cols, rows, color));
                }
            }
        } else {
            names.insert(name.to_string());
        }
        rest = &body[end + 1..];
    }
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

pub fn resolve_bin(name: &str) -> PathBuf {
    let key = format!("CARGO_BIN_EXE_{}", name.replace('-', "_"));
    if let Ok(path) = std::env::var(&key) {
        let p = PathBuf::from(path);
        if p.exists() {
            return p;
        }
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(Path::parent).unwrap_or(manifest);
    let debug = root.join("target").join("debug").join(name);
    if debug.exists() {
        return debug;
    }
    let release = root.join("target").join("release").join(name);
    if release.exists() {
        return release;
    }
    PathBuf::from(name)
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
        self.inner.press(key)
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
        drive(&mut session, case.sends);
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

pub fn drive(session: &mut Session, steps: &[&str]) {
    for step in steps {
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
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
                .unwrap_or_else(|e| panic!("`wait:{needle}` timed out: {e:#}"));
        } else if let Some(text) = step.strip_prefix("type:") {
            session.inner.send_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            let chord = normalize_chord(step);
            session.inner.press(&chord).expect("send_key");
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

pub fn settle_and_gate(session: &mut Session, name: &str) {
    let cancel = CancelToken::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
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
    settle_and_gate(&mut session, &case.name);
}

/// Expand one representative static Case::new root through the full canonical
/// matrix. The representative's sends/timeout apply to every combo; choose a
/// representative whose determinism contract is size-independent.
pub fn run_canonical(representative: &Case) {
    let mut failures = Vec::new();
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = representative.variant(cols, rows, color);
            let name = case.name.to_string();
            if !collect_matrix(&name, || run_and_assert(&case)) {
                failures.push(name);
            }
        }
    }
    finish_matrix(&failures);
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
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = selected
                .get(&(cols, rows, color.suffix()))
                .copied()
                .unwrap_or(representative)
                .variant(cols, rows, color);
            let name = case.name.to_string();
            if !collect_matrix(&name, || run_and_assert(&case)) {
                failures.push(name);
            }
        }
    }
    finish_matrix(&failures);
}

/// Expand a live pointer/keyboard/manual-flow root through the same matrix.
/// `interact` runs after the centrally driven boot + case sends and before the
/// centrally settled/gated capture.
pub fn run_canonical_live(representative: &Case, mut interact: impl FnMut(&mut Session, &Case)) {
    let mut failures = Vec::new();
    for &(cols, rows) in &CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = representative.variant(cols, rows, color);
            let name = case.name.to_string();
            if !collect_matrix(&name, || {
                let mut session = spawn_boot(&case);
                interact(&mut session, &case);
                settle_and_gate(&mut session, &case.name);
            }) {
                failures.push(name);
            }
        }
    }
    finish_matrix(&failures);
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
                settle_and_gate(&mut session, &case.name);
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

/// Panic if any [`collect_matrix`] call reported a failure.
pub fn finish_matrix(failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{} matrix capture(s) failed: {}",
        failures.len(),
        failures.join(", ")
    );
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
