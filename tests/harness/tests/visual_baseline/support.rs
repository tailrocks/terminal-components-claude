//! Shared driver for the visual-baseline suite.
//!
//! One PTY per capture: spawn → boot needle (or idle if none) → send
//! steps (120 ms pacing) → `wait_stable`(SETTLE) → frame → store gate.
//! [`run_and_assert`] runs the ported matrix; [`boot`]/[`drive`] mirror it
//! for the pointer group, which needs the live session afterwards. A
//! leading `wait:` needle is readiness — live clocks skip the 200 ms quiet
//! window.
//!
//! Store: the grouped multi-artifact store (`tuiscotti::grouped`). Approved
//! frames live at the repo-root
//! `snapshots/<group>/<sub_group>/<name>.{ansi,txt,png,html}` (committed,
//! exactly four artifacts per scenario); actuals, diffs and the HTML report
//! are scratch under this harness crate's `target/tuiscotti/` (gitignored).
//! The capture name is the grouped path, e.g.
//! `holla/parity/discovery/120x40/truecolor`.
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
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tuiscotti::grouped::{GroupedOutcome, GroupedStore};
use tuiscotti::snapshot::Status;
use tuiscotti::tui::{CancelToken, MouseButton, MouseMods, Session, Tui, WaitError};
use tuiscotti::{Frame, Observation, Profile, Provenance, Renderer, Screen, VENDORED_FACES};

pub const SHOWCASE: &str = env!("CARGO_BIN_EXE_showcase");
pub const TABLEPRO: &str = env!("CARGO_BIN_EXE_tablepro");
pub const JACKIN: &str = env!("CARGO_BIN_EXE_jackin-preview");
pub const HOLLA: &str = env!("CARGO_BIN_EXE_holla");

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

/// PTY builder for a case: geometry, the `TERM` preset from the default
/// profile (`xterm-256color`, as before), the `COLORTERM`/`LINES`/`COLUMNS`
/// presets, and the colour-hygiene env (see the module docs). Waits take
/// their bound from [`Case::timeout_ms`] explicitly — sessions no longer
/// carry a default deadline.
pub fn tui_for(case: &Case) -> Tui {
    let mut tui = Tui::new(argv_for(case))
        .size(case.cols, case.rows)
        .env("COLORTERM", "truecolor")
        .env("LINES", case.rows.to_string())
        .env("COLUMNS", case.cols.to_string())
        .env_remove("NO_COLOR")
        .env_remove("HOLLA_NO_MOTION")
        .env_remove("JACKIN_NO_MOTION")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .env("HOLLA_NO_HISTORY", "1");
    if matches!(case.color, Color::NoColorEnv) {
        tui = tui.env("NO_COLOR", "1");
    }
    tui
}

/// One row of visible text: continuation cells skipped, symbols
/// concatenated. Callers trim; matching never depends on trailing blanks.
pub fn row_text(screen: &Screen, row: u16) -> String {
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
pub fn screen_text(screen: &Screen) -> String {
    (0..screen.rows())
        .map(|y| row_text(screen, y))
        .collect::<Vec<_>>()
        .join("\n")
}

/// First occurrence of `needle` as `(row, col)` — top-to-bottom, leftmost
/// per row. The byte offset becomes a display column via the prefix width,
/// so wide-cell rows resolve exactly.
pub fn screen_find(screen: &Screen, needle: &str) -> Option<(u16, u16)> {
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

/// Canonical frame from a live screen: cells copied verbatim (grid-local
/// coordinates at origin (0,0)), cursor carried over. Provenance is
/// informational (excluded from digests).
pub fn frame_from_screen(screen: &Screen, provenance: Provenance) -> Frame {
    let mut frame = Frame::blank(screen.cols(), screen.rows(), provenance);
    frame.cells = screen.cells().to_vec();
    frame.cursor = *screen.cursor();
    frame
}

/// PTY capture provenance for a case (argv pinned, profile renamed).
pub fn pty_provenance(case: &Case) -> Provenance {
    Provenance::now("tuiscotti-default", "pty", argv_for(case))
}

thread_local! {
    /// One renderer per test thread: font faces parsed once, glyph rasters
    /// cached across every check on the thread (no locks).
    static RENDERER: RefCell<Renderer> = RefCell::new(
        Profile::default_profile()
            .renderer(&VENDORED_FACES)
            .expect("vendored faces parse"),
    );
}

/// The grouped store: approved tree at the repo-root `snapshots/`
/// (committed), scratch (actuals, diffs, report) under this harness
/// crate's `target/tuiscotti/` (gitignored). Both are anchored on
/// `CARGO_MANIFEST_DIR`, never on the process working directory.
pub fn store() -> GroupedStore {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let approved = root.join("../../snapshots");
    let actual = root.join("target/tuiscotti/actual");
    let diff = root.join("target/tuiscotti/diff");
    let report = root.join("target/tuiscotti/report.html");
    GroupedStore::new(&approved)
        .with_actual_root(&actual)
        .with_diff_root(&diff)
        .with_report_path(&report)
}

/// Cell-exact (ansi) + content (txt) + render-level (html) byte gates +
/// pixel-exact gate at threshold 1.0 through the thread's cached renderer.
/// Writes `target/tuiscotti/actual/` artifacts even when unmatched.
pub fn gate(name: &str, frame: &Frame) -> GroupedOutcome {
    RENDERER
        .with(|r| store().check_with(&mut r.borrow_mut(), name, frame, 1.0))
        .unwrap_or_else(|e| panic!("gate `{name}` failed: {e}"))
}

/// Fail-closed assertion: only `matched` passes. Missing approval remains
/// pending after capture, but the test fails until the full suite is
/// generated and explicitly blessed (`tuiscotti accept --grouped` is the
/// only bless, never the test); drift, dimension mismatch and corrupt
/// approval also fail.
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

/// A ported-matrix capture: the bash runner's flow (boot needle or idle,
/// sends, settle, gate) on one fresh session per capture.
pub fn run_and_assert(case: &Case) {
    let timeout = Duration::from_millis(case.timeout_ms);
    let mut session = spawn(case);
    boot(&mut session, case.needle, timeout);
    drive(&mut session, case.sends, timeout);
    settle_and_gate(&mut session, case);
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
                settle_and_gate(&mut session, &case);
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
                settle_and_gate(&mut session, &case);
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

/// Bounded wait until `pred` holds, returning the outcome for callers
/// that report their own timeout evidence.
pub fn try_wait_screen(
    s: &mut Session,
    timeout: Duration,
    pred: impl FnMut(&Screen) -> bool,
) -> Result<Observation, WaitError> {
    // The poll loop calls the predicate sequentially (no reentrancy), so a
    // RefCell bridges the FnMut caller surface to the Fn wait surface.
    let pred = RefCell::new(pred);
    s.wait_predicate_timeout(|o| pred.borrow_mut()(&o.screen), timeout)
}

/// Bounded wait until `pred` holds on a fresh observation. Returns the
/// matching observation; the predicate sees the live screen, as before.
pub fn wait_screen(
    s: &mut Session,
    timeout: Duration,
    what: &str,
    pred: impl FnMut(&Screen) -> bool,
) -> Observation {
    try_wait_screen(s, timeout, pred).unwrap_or_else(|e| panic!("{what}: {e:#}"))
}

/// Bounded wait until the visible text contains `needle`.
pub fn wait_for_text(s: &mut Session, needle: &str, timeout: Duration) {
    wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| screen_text(screen).contains(needle),
    );
}

/// Translate one send step to the chord grammar: the old `ctrl-`/`alt-`/
/// `shift-` modifier form becomes `ctrl+`/`alt+`/`shift+`, and `backtab`
/// becomes `shift+tab` (same CSI Z bytes). All other names — lowercase
/// keys, single chars, f1..f12 — parse verbatim.
pub fn key_chord(step: &str) -> Cow<'_, str> {
    if step.eq_ignore_ascii_case("backtab") {
        return Cow::Borrowed("shift+tab");
    }
    if let Some((mods, _)) = step.rsplit_once('-')
        && !mods.is_empty()
        && mods
            .split('-')
            .all(|m| matches!(m.to_ascii_lowercase().as_str(), "ctrl" | "alt" | "shift"))
    {
        return Cow::Owned(step.replace('-', "+"));
    }
    Cow::Borrowed(step)
}

/// Spawn a case's session for the pointer group (mouse/resize captures need
/// the live session after boot).
pub fn spawn(case: &Case) -> Session {
    tui_for(case)
        .spawn()
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name))
}

/// Central live-session setup: boot needle first, then all case sends. This
/// mirrors [`run_and_assert`] while handing the connected session back for
/// pointer or manual-flow assertions.
pub fn spawn_boot(case: &Case) -> Session {
    let timeout = Duration::from_millis(case.timeout_ms);
    let mut session = spawn(case);
    boot(&mut session, case.needle, timeout);
    if !case.sends.is_empty() {
        drive(&mut session, case.sends, timeout);
    }
    session
}

/// Boot: needle first. Live clocks starve a quiet-window wait.
pub fn boot(session: &mut Session, needle: &str, timeout: Duration) {
    if !needle.is_empty() {
        wait_for_text(session, needle, timeout);
        return;
    }
    let cancel = CancelToken::new();
    session
        .wait_stable_quiet(
            Instant::now() + timeout,
            Duration::from_millis(200),
            &cancel,
        )
        .unwrap_or_else(|e| panic!("boot idle failed: {e:#}"));
}

/// The step loop on a live session, pacing included: key names (chord
/// grammar), `type:<text>`, `sleep:<ms>`, `wait:<needle>`.
pub fn drive(session: &mut Session, steps: &[&str], timeout: Duration) {
    for step in steps {
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
            wait_screen(
                session,
                timeout,
                &format!("`wait:{needle}` timed out"),
                |screen| screen_text(screen).contains(needle),
            );
        } else if let Some(text) = step.strip_prefix("type:") {
            session.send_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            let chord = key_chord(step);
            session
                .press(&chord)
                .unwrap_or_else(|e| panic!("key `{step}` failed: {e:#}"));
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

/// Primary-button drag from `from` to `to`: press, one held-motion
/// report per cell crossed (straight-line interpolation, as a terminal
/// sends), release. Applications may act along the path, not just on
/// its ends, so a single endpoint hop would under-report.
pub fn drag_path(s: &mut Session, from: (u16, u16), to: (u16, u16)) {
    s.mouse_down(MouseButton::Left, from.0, from.1, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("drag press at {from:?} failed: {e:#}"));
    for (col, row) in cells_between(from, to) {
        s.mouse_drag(MouseButton::Left, col, row, MouseMods::NONE)
            .unwrap_or_else(|e| panic!("drag motion to ({col}, {row}) failed: {e:#}"));
    }
    s.mouse_up(MouseButton::Left, to.0, to.1, MouseMods::NONE)
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
    provenance: Provenance,
) -> Frame {
    let cancel = CancelToken::new();
    let obs = session
        .wait_stable_quiet(Instant::now() + timeout, quiet, &cancel)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e:#}"));
    frame_from_screen(&obs.screen, provenance)
}

/// Settle the screen and gate the capture.
pub fn settle_and_gate(session: &mut Session, case: &Case) {
    let frame = settle_frame(
        session,
        SETTLE,
        Duration::from_millis(case.timeout_ms),
        &case.name,
        pty_provenance(case),
    );
    assert_gated(&gate(&case.name, &frame));
}

/// Capture a production (ratatui 0.30) buffer through the component path.
/// tuiscotti 0.2.0 pins ratatui =0.29.0, so cells are adapted field-by-field
/// onto a legacy buffer first; the canonical conversion (wide cells,
/// continuations, cursor) stays inside `tuiscotti::ratatui::from_buffer`.
pub fn capture_buffer(buf: &ratatui::buffer::Buffer, provenance: Provenance) -> Frame {
    let area = buf.area;
    let mut legacy = ratatui029::buffer::Buffer::empty(ratatui029::layout::Rect::new(
        area.x,
        area.y,
        area.width,
        area.height,
    ));
    for y in 0..area.height {
        for x in 0..area.width {
            let src = &buf[(x, y)];
            let dst = &mut legacy[(x, y)];
            dst.set_symbol(src.symbol());
            dst.fg = map_color(src.fg);
            dst.bg = map_color(src.bg);
            dst.underline_color = map_color(src.underline_color);
            dst.modifier = map_modifier(src.modifier);
            dst.set_skip(matches!(
                src.diff_option,
                ratatui::buffer::CellDiffOption::Skip
            ));
        }
    }
    tuiscotti::ratatui::from_buffer(&legacy, area.width, area.height, None, provenance)
}

fn map_color(c: ratatui::style::Color) -> ratatui029::style::Color {
    use ratatui::style::Color as New;
    use ratatui029::style::Color as Old;
    match c {
        New::Reset => Old::Reset,
        New::Black => Old::Black,
        New::Red => Old::Red,
        New::Green => Old::Green,
        New::Yellow => Old::Yellow,
        New::Blue => Old::Blue,
        New::Magenta => Old::Magenta,
        New::Cyan => Old::Cyan,
        New::Gray => Old::Gray,
        New::DarkGray => Old::DarkGray,
        New::LightRed => Old::LightRed,
        New::LightGreen => Old::LightGreen,
        New::LightYellow => Old::LightYellow,
        New::LightBlue => Old::LightBlue,
        New::LightMagenta => Old::LightMagenta,
        New::LightCyan => Old::LightCyan,
        New::White => Old::White,
        New::Rgb(r, g, b) => Old::Rgb(r, g, b),
        New::Indexed(i) => Old::Indexed(i),
    }
}

fn map_modifier(m: ratatui::style::Modifier) -> ratatui029::style::Modifier {
    use ratatui::style::Modifier as New;
    use ratatui029::style::Modifier as Old;
    let mut out = Old::empty();
    for (new, old) in [
        (New::BOLD, Old::BOLD),
        (New::DIM, Old::DIM),
        (New::ITALIC, Old::ITALIC),
        (New::UNDERLINED, Old::UNDERLINED),
        (New::SLOW_BLINK, Old::SLOW_BLINK),
        (New::RAPID_BLINK, Old::RAPID_BLINK),
        (New::REVERSED, Old::REVERSED),
        (New::HIDDEN, Old::HIDDEN),
        (New::CROSSED_OUT, Old::CROSSED_OUT),
    ] {
        if m.contains(new) {
            out.insert(old);
        }
    }
    out
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
