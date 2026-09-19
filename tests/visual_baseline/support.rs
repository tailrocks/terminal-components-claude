//! Shared driver for the visual-baseline suite.
//!
//! One PTY per capture: spawn → boot needle (or idle if none) → send
//! steps (120 ms pacing) → `wait_stable`(SETTLE) → frame → store gate.
//! [`run_once`] runs the ported matrix; [`boot`]/[`drive`] mirror it for
//! the pointer group, which needs the live session afterwards. A leading
//! `wait:` needle is readiness — live clocks skip the 200 ms quiet window.
//!
//! Store: the grouped multi-artifact store (`tuisnap::grouped`). Approved
//! frames are the external oracle at `VISUAL_BASELINE_STORE` (fail closed if
//! missing). Never write that tree; never use in-tree `snapshots/` as the
//! oracle. Actuals, diffs and the HTML report live under
//! `VISUAL_BASELINE_ACTUAL` or `$RUN_DIR/tuisnap` — never under the oracle
//! import and never under the worktree `snapshots/` directory. The capture
//! name is the grouped path, e.g. `holla/parity/discovery/120x40/truecolor`.
//!
//! Binaries: `VISUAL_BASELINE_BIN_DIR` or `$CARGO_TARGET_DIR/debug`. Do not
//! realpath; HTML provenance `argv[0]` is the literal path string. Known-good
//! HTML match uses
//! `/Users/donbeave/Projects/terminal-components-claude/target/debug/{showcase,tablepro,jackin-preview,holla}`.
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
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tuisnap::grouped::{GroupedOutcome, GroupedStore};
use tuisnap::pty::{PtyOptions, Session, run_once};
use tuisnap::snapshot::Status;
use tuisnap::{Frame, Profile, Renderer, VENDORED_FACES};

/// Frozen oracle key count. Four artifacts per key.
pub const EXPECTED_STORE_KEYS: usize = 7_550;

/// Case tables pass these names; [`argv_for`] resolves the executable path.
pub const SHOWCASE: &str = "showcase";
pub const TABLEPRO: &str = "tablepro";
pub const JACKIN: &str = "jackin-preview";
pub const HOLLA: &str = "holla";

/// `SETTLE_MS` from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// `TIMEOUT_MS` default; boot-streaming screens override it per capture
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
/// `Case::new` root and the data-driven audit matrices. `Case::dynamic` loops are
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
    // This package lives at tests/visual_baseline; case tables sit beside
    // support.rs rather than under a nested tests/visual_baseline/.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
            || file_name == "lib.rs"
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

    /// `CAP_TIMEOUT` override for screens whose boot stream outlasts
    /// `TIMEOUT_MS` (scrolling, terminal): the boot `wait_idle` shares it.
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
    argv.push(resolve_bin(case.bin));
    argv.extend(case.args.iter().map(ToString::to_string));
    let flag = match case.color {
        Color::Truecolor => Some("truecolor"),
        Color::Ansi256 => Some("256"),
        Color::Ansi16 => Some("16"),
        Color::None => Some("none"),
        Color::NoColorEnv => None,
    };
    if let Some(flag) = flag {
        argv.push("--color".to_string());
        argv.push(flag.to_string());
    }
    argv
}

pub fn opts_for(case: &Case) -> PtyOptions {
    let opts = PtyOptions {
        cols: case.cols,
        rows: case.rows,
        timeout: Duration::from_millis(case.timeout_ms),
        ..PtyOptions::default()
    }
    .without_env("NO_COLOR")
    .without_env("HOLLA_NO_MOTION")
    .without_env("JACKIN_NO_MOTION")
    .without_env("CLICOLOR_FORCE")
    .without_env("FORCE_COLOR")
    .with_env("HOLLA_NO_HISTORY", "1");
    match case.color {
        Color::NoColorEnv => opts.with_env("NO_COLOR", "1"),
        _ => opts,
    }
}

/// Boot needle first (a `wait:` step), then the sends — the bash runner's
/// step order.
fn steps_for(case: &Case) -> Vec<String> {
    let mut steps = Vec::with_capacity(case.sends.len() + 1);
    if !case.needle.is_empty() {
        steps.push(format!("wait:{}", case.needle));
    }
    steps.extend(case.sends.iter().map(ToString::to_string));
    steps
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

/// Oracle store root. Fail closed if `VISUAL_BASELINE_STORE` is missing.
pub fn oracle_store_path() -> PathBuf {
    refuse_bless();
    let raw = env::var("VISUAL_BASELINE_STORE").unwrap_or_else(|_| {
        panic!(
            "VISUAL_BASELINE_STORE is required (external grouped oracle; in-tree snapshots/ is not the gate)"
        )
    });
    assert!(
        !raw.is_empty(),
        "VISUAL_BASELINE_STORE is empty; refusing to guess snapshots/"
    );
    let path = PathBuf::from(raw);
    assert!(
        path.is_dir(),
        "VISUAL_BASELINE_STORE is not a directory: {}",
        path.display()
    );
    let worktree_snapshots = worktree_snapshots_dir();
    assert!(
        !paths_overlap(&path, &worktree_snapshots),
        "VISUAL_BASELINE_STORE {} must not be the worktree snapshots/ tree {}",
        path.display(),
        worktree_snapshots.display()
    );
    path
}

/// Scratch root for actuals/diffs/report. Never the oracle, never worktree snapshots/.
pub fn scratch_root() -> PathBuf {
    refuse_bless();
    let oracle = oracle_store_path();
    let raw = env::var("VISUAL_BASELINE_ACTUAL")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var("RUN_DIR")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|run| PathBuf::from(run).join("tuisnap"))
        })
        .unwrap_or_else(|| {
            panic!(
                "VISUAL_BASELINE_ACTUAL or RUN_DIR is required for tuisnap scratch (never oracle, never worktree snapshots/)"
            )
        });
    let worktree_snapshots = worktree_snapshots_dir();
    assert!(
        !paths_overlap(&raw, &oracle),
        "tuisnap scratch {} overlaps oracle store {}",
        raw.display(),
        oracle.display()
    );
    assert!(
        !paths_overlap(&raw, &worktree_snapshots),
        "tuisnap scratch {} overlaps worktree snapshots/ {}",
        raw.display(),
        worktree_snapshots.display()
    );
    raw
}

/// The grouped store: approved tree from `VISUAL_BASELINE_STORE`, scratch
/// under `VISUAL_BASELINE_ACTUAL` or `$RUN_DIR/tuisnap`.
pub fn store() -> GroupedStore {
    let approved = oracle_store_path();
    let scratch = scratch_root();
    GroupedStore::new(&approved)
        .with_actual_root(&scratch.join("actual"))
        .with_diff_root(&scratch.join("diff"))
        .with_report_path(&scratch.join("report.html"))
}

fn refuse_bless() {
    for name in [
        "BLESS",
        "UPDATE_BASELINE",
        "TUISNAP_ACCEPT",
        "TUISNAP_BLESS",
    ] {
        assert!(
            env::var_os(name).is_none(),
            "refusing {name}; never bless/accept from this suite"
        );
    }
}

fn worktree_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn worktree_snapshots_dir() -> PathBuf {
    worktree_root().join("snapshots")
}

fn lexical_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn paths_overlap(a: &Path, b: &Path) -> bool {
    let a = a.canonicalize().unwrap_or_else(|_| lexical_absolute(a));
    let b = b.canonicalize().unwrap_or_else(|_| lexical_absolute(b));
    a == b || a.starts_with(&b) || b.starts_with(&a)
}

fn bin_dir() -> PathBuf {
    if let Ok(dir) = env::var("VISUAL_BASELINE_BIN_DIR") {
        assert!(
            !dir.is_empty(),
            "VISUAL_BASELINE_BIN_DIR is empty; refusing to guess"
        );
        return PathBuf::from(dir);
    }
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        assert!(
            !dir.is_empty(),
            "CARGO_TARGET_DIR is empty; set VISUAL_BASELINE_BIN_DIR or CARGO_TARGET_DIR"
        );
        return PathBuf::from(dir).join("debug");
    }
    panic!(
        "set VISUAL_BASELINE_BIN_DIR or CARGO_TARGET_DIR (HTML argv[0] is the literal debug bin path)"
    );
}

fn resolve_bin(name: &str) -> String {
    // Do not realpath: frozen HTML provenance.argv[0] is this exact string.
    bin_dir()
        .join(name)
        .into_os_string()
        .into_string()
        .unwrap_or_else(|raw| panic!("bin path for {name} is not UTF-8: {}", raw.display()))
}

/// Cell-exact (ansi) + content (txt) + render-level (html) byte gates +
/// pixel-exact gate at threshold 1.0 through the thread's cached renderer.
/// Writes scratch actuals even when unmatched; never writes the oracle.
pub fn gate(name: &str, frame: &Frame) -> GroupedOutcome {
    RENDERER
        .with(|r| store().check_with(&mut r.borrow_mut(), name, frame, 1.0))
        .unwrap_or_else(|e| panic!("gate `{name}` failed: {e}"))
}

/// Fail-closed assertion: only `matched` passes. Missing approval remains
/// pending after capture, but the test fails until the full suite is
/// generated and explicitly blessed (`tuisnap accept --grouped` is the only
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

/// A ported-matrix capture: the same runner the `tuisnap run` CLI used.
pub fn run_and_assert(case: &Case) {
    let frame = run_once(&argv_for(case), &opts_for(case), &steps_for(case), SETTLE)
        .unwrap_or_else(|e| panic!("capture `{}` failed: {e:#}", case.name));
    assert_gated(&gate(&case.name, &frame));
}

/// Expand one representative static `Case::new` root through the full canonical
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
                .or_else(|| e.downcast_ref::<&str>().map(ToString::to_string))
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

/// Spawn a case's session for the pointer group (mouse/resize captures need
/// the live session after boot).
pub fn spawn(case: &Case) -> Session {
    Session::spawn(&argv_for(case), &opts_for(case))
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name))
}

/// Central live-session setup: boot needle first, then all case sends. This
/// mirrors [`run_once`] while handing the connected session back for pointer
/// or manual-flow assertions.
pub fn spawn_boot(case: &Case) -> Session {
    let mut session = spawn(case);
    boot(&mut session, case.needle);
    if !case.sends.is_empty() {
        drive(&mut session, case.sends);
    }
    session
}

/// Boot: needle first. Live clocks starve a quiet-window `wait_idle`.
pub fn boot(session: &mut Session, needle: &str) {
    if !needle.is_empty() {
        session
            .wait_for_text(needle)
            .unwrap_or_else(|e| panic!("boot needle `{needle}` never appeared: {e:#}"));
        return;
    }
    session
        .wait_idle(Duration::from_millis(200))
        .unwrap_or_else(|e| panic!("boot idle failed: {e:#}"));
}

/// `run_once`'s step loop on a live session, pacing included.
pub fn drive(session: &mut Session, steps: &[&str]) {
    for step in steps {
        if let Some(ms) = step.strip_prefix("sleep:") {
            std::thread::sleep(Duration::from_millis(ms.parse().expect("sleep:<ms>")));
        } else if let Some(needle) = step.strip_prefix("wait:") {
            session
                .wait_for_text(needle)
                .unwrap_or_else(|e| panic!("`wait:{needle}` timed out: {e:#}"));
        } else if let Some(text) = step.strip_prefix("type:") {
            session.type_text(text).expect("type_text");
            std::thread::sleep(Duration::from_millis(120));
        } else {
            session.send_key(step).expect("send_key");
            std::thread::sleep(Duration::from_millis(120));
        }
    }
}

/// Settle the screen and gate the capture.
pub fn settle_and_gate(session: &mut Session, name: &str) {
    let frame = session
        .wait_stable(SETTLE)
        .unwrap_or_else(|e| panic!("`{name}` never settled: {e:#}"));
    assert_gated(&gate(name, &frame));
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
