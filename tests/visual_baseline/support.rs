//! Shared driver for the visual-baseline suite (tuiscotti main, Workflow 2).
//!
//! One PTY per capture: `tuiscotti::tui::Tui::new([bin, args…])` with an
//! explicit environment → boot needle (content-gated `Locator` wait) or a
//! non-blank + quiet wait when a case has none → send steps (120 ms pacing)
//! → `wait_stable_quiet(SETTLE)` → `render::frame_from_screen` → gate.
//! [`run_and_assert`] runs the ported matrix; [`spawn_boot`]/[`drive`] hand
//! the live [`Session`] to the pointer group.
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
//! tuisnap engine. Only groups listed in [`REAPPROVED_PREFIXES`] gate on the
//! new engine; every other capture is skipped unless `VISUAL_FULL=1` (see
//! `docs/visual-testing/tuiscotti-migration.md`).

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::ansi_cursor;
use tuiscotti::diff;
use tuiscotti::locate::Locator;
use tuiscotti::render::{self, Renderer};
use tuiscotti::tui::{CancelToken, Key, KeyMods, MouseMods, Tui, WaitError, Wheel};
use tuiscotti::{Frame, Observation, Profile, VENDORED_FACES};

pub use tuiscotti::MouseButton;

pub const SHOWCASE: &str = env!("CARGO_BIN_EXE_showcase");
pub const TABLEPRO: &str = env!("CARGO_BIN_EXE_tablepro");
pub const JACKIN: &str = env!("CARGO_BIN_EXE_jackin-preview");
pub const HOLLA: &str = env!("CARGO_BIN_EXE_holla");

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
/// Accounts is a custom loop with the same name shape ([`AUDIT_PREFIX_JACKIN_ACCOUNTS`]).
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

pub fn audit_default_name(prefix: &str, cols: u16, rows: u16, color: Color) -> String {
    format!("{prefix}/{cols}x{rows}/{}", color.suffix())
}

/// Every capture name the suite produces: the canonical 5×5 expansion of each
/// Case::new root and the data-driven audit matrices. `Case::dynamic` loops are
/// not parsed; their names come from the audit constants below.
pub fn suite_capture_names() -> BTreeSet<String> {
    let mut names = parse_case_new_names();
    names.extend(generated_matrix_names());
    names
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

/// Parse only the representative in each `baseline_case*!` invocation. Variant
/// arrays can contain additional exact per-combo declarations; they select at
/// runtime and must not create duplicate canonical roots in the inventory.
fn extract_case_new_roots(
    src: &str,
    names: &mut BTreeSet<String>,
    declared_roots: &mut BTreeSet<String>,
) {
    const MACRO_MARK: &str = "crate::baseline_case";
    const MARK: &str = "Case::new(";
    let mut rest = src;
    while let Some(macro_i) = rest.find(MACRO_MARK) {
        // Flow tests declare representatives outside macros. Parse those raw
        // declarations before skipping the entire macro invocation.
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

/// Return the offset just past a balanced `baseline_case*!(...)` invocation.
/// Sends can contain arbitrary text, so `;` is not a safe invocation boundary.
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

/// Exact-name filter for macOS Finder metadata. Not snapshot content; every
/// other unknown file remains a store-integrity failure.
pub fn is_macos_platform_metadata(path: &Path) -> bool {
    path.file_name() == Some(std::ffi::OsStr::new(".DS_Store"))
}

/// `--color` palette, or real `NO_COLOR=1` (the baseline's `nocolor`).
#[derive(Clone, Copy)]
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

    /// Owned-name form for the data-driven matrices (audit 5×5, audit-flows).
    pub fn dynamic(
        name: String,
        bin: &'static str,
        args: &'static [&'static str],
        cols: u16,
        rows: u16,
        color: Color,
        needle: &'static str,
    ) -> Self {
        Self {
            name: Cow::Owned(name),
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

    /// CAP_TIMEOUT override for screens whose boot stream outlasts
    /// TIMEOUT_MS (scrolling, terminal): the boot wait_idle shares it.
    pub fn timeout(self, ms: u64) -> Self {
        Self {
            timeout_ms: ms,
            ..self
        }
    }

    /// Re-root a representative declaration at another canonical combo while
    /// preserving its argv, boot needle, sends, and timeout drift.
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

    /// Re-root a resize capture at its target geometry while retaining the
    /// representative's initial PTY geometry and capture behavior.
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

/// Capture-name prefixes already re-approved on tuiscotti main. Captures
/// under these gate (with `--run-ignored`); everything else is pending
/// re-approval and runs only with `VISUAL_FULL=1`.
pub const REAPPROVED_PREFIXES: &[&str] = &[];

pub fn is_reapproved(name: &str) -> bool {
    REAPPROVED_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
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

/// A ported-matrix capture: boot, sends, settle, gate.
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
///
/// Captures still pending re-approval are skipped here (counted as passed)
/// unless `VISUAL_FULL=1`; every matrix funnels through this function.
pub fn collect_matrix(combo: &str, body: impl FnOnce()) -> bool {
    if !capture_enabled(combo) {
        return true;
    }
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
