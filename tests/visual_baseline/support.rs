//! Shared driver for the visual-baseline suite.
//!
//! One PTY per capture: spawn → boot needle (or idle if none) → send
//! steps (120 ms pacing) → `wait_stable`(SETTLE) → frame → store gate.
//! [`run_once`] runs the ported matrix; [`boot`]/[`drive`] mirror it for
//! the pointer group, which needs the live session afterwards. A leading
//! `wait:` needle is readiness — live clocks skip the 200 ms quiet window.
//!
//! Store: the grouped multi-artifact store (`tuisnap::grouped`). Approved
//! frames live at `snapshots/<group>/<sub_group>/<name>.{ansi,txt,png,html}`
//! (committed, exactly four artifacts per scenario); actuals, diffs and the
//! HTML report are scratch under `target/tuisnap/` (gitignored). The capture
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

use tuisnap::grouped::{GroupedOutcome, GroupedStore};
use tuisnap::pty::{PtyOptions, Session, run_once};
use tuisnap::snapshot::Status;
use tuisnap::{Frame, Profile, Renderer, VENDORED_FACES};

pub const SHOWCASE: &str = env!("CARGO_BIN_EXE_showcase");
pub const TABLEPRO: &str = env!("CARGO_BIN_EXE_tablepro");
pub const JACKIN: &str = env!("CARGO_BIN_EXE_jackin-preview");
pub const HOLLA: &str = env!("CARGO_BIN_EXE_holla");

/// SETTLE_MS from the bash runner.
pub const SETTLE: Duration = Duration::from_millis(400);
/// TIMEOUT_MS default; boot-streaming screens override it per capture
/// ([`Case::timeout`], the bash `CAP_TIMEOUT=` prefix).
pub const TIMEOUT_MS: u64 = 8_000;

/// Audit-matrix axes (docs/baseline/snapshots-v2.md §taxonomy): every audit
/// fixture is captured at all 5 sizes × 5 colours.
pub const AUDIT_SIZES: [(u16, u16); 5] = [(72, 20), (80, 24), (100, 30), (120, 40), (160, 50)];
pub const AUDIT_COLORS: [Color; 5] = [
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

/// Remaining 8 size×colour combos of the audit-flow matrix (the proven
/// 120x40/truecolor leaf is a static Case::new).
pub const FLOW_VARIANTS: [(u16, u16, Color); 8] = [
    (80, 24, Color::Truecolor),
    (80, 24, Color::None),
    (80, 24, Color::NoColorEnv),
    (120, 40, Color::None),
    (120, 40, Color::NoColorEnv),
    (160, 50, Color::Truecolor),
    (160, 50, Color::None),
    (160, 50, Color::NoColorEnv),
];

pub const FLOW_LEAF_DIFF_REVIEW: &str = "diff/review";
pub const FLOW_LEAF_DIFF_EMPTY: &str = "diff/empty";
pub const FLOW_LEAF_FORMS_INVALID: &str = "forms/invalid";
pub const FLOW_LEAF_INPUTS_SELECTED: &str = "inputs/selected";
pub const FLOW_LEAF_DIFF_DRAG_SELECTED: &str = "diff/drag-selected";

pub const FLOW_VARIANT_LEAVES: [&str; 4] = [
    FLOW_LEAF_DIFF_REVIEW,
    FLOW_LEAF_DIFF_EMPTY,
    FLOW_LEAF_FORMS_INVALID,
    FLOW_LEAF_INPUTS_SELECTED,
];

pub fn audit_default_name(prefix: &str, cols: u16, rows: u16, color: Color) -> String {
    format!("{prefix}/{cols}x{rows}/{}", color.suffix())
}

pub fn showcase_flow_name(leaf: &str, cols: u16, rows: u16, color: Color) -> String {
    format!("showcase/flows/{leaf}/{cols}x{rows}/{}", color.suffix())
}

/// Every capture name the suite produces: Case::new first-arg literals plus
/// the data-driven audit / flow / drag matrices (same consts the tests use).
pub fn suite_capture_names() -> BTreeSet<String> {
    let mut names = parse_case_new_names();
    names.extend(generated_matrix_names());
    names
}

fn generated_matrix_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for prefix in AUDIT_MATRIX_PREFIXES {
        for &(cols, rows) in &AUDIT_SIZES {
            for color in AUDIT_COLORS {
                names.insert(audit_default_name(prefix, cols, rows, color));
            }
        }
    }
    for &(cols, rows) in &AUDIT_SIZES {
        for color in AUDIT_COLORS {
            names.insert(audit_default_name(
                AUDIT_PREFIX_JACKIN_ACCOUNTS,
                cols,
                rows,
                color,
            ));
        }
    }
    for leaf in FLOW_VARIANT_LEAVES {
        for &(cols, rows, color) in &FLOW_VARIANTS {
            names.insert(showcase_flow_name(leaf, cols, rows, color));
        }
    }
    for &(cols, rows, color) in &FLOW_VARIANTS {
        names.insert(showcase_flow_name(
            FLOW_LEAF_DIFF_DRAG_SELECTED,
            cols,
            rows,
            color,
        ));
    }
    names
}

fn parse_case_new_names() -> BTreeSet<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/visual_baseline");
    let mut names = BTreeSet::new();
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
        extract_case_new_names(&src, &mut names);
    }
    assert!(
        !names.is_empty(),
        "no Case::new names parsed from {}",
        dir.display()
    );
    names
}

fn extract_case_new_names(src: &str, names: &mut BTreeSet<String>) {
    let mut rest = src;
    const MARK: &str = "Case::new(";
    while let Some(i) = rest.find(MARK) {
        rest = rest[i + MARK.len()..].trim_start();
        let Some(body) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(end) = body.find('"') else {
            panic!("unterminated Case::new string literal");
        };
        names.insert(body[..end].to_string());
        rest = &body[end + 1..];
    }
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
    steps.extend(case.sends.iter().map(|s| s.to_string()));
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

/// The grouped store: approved tree at `snapshots/` (committed), scratch
/// (actuals, diffs, report) under `target/tuisnap/` (gitignored).
pub fn store() -> GroupedStore {
    GroupedStore::new(Path::new("snapshots"))
        .with_actual_root(Path::new("target/tuisnap/actual"))
        .with_diff_root(Path::new("target/tuisnap/diff"))
        .with_report_path(Path::new("target/tuisnap/report.html"))
}

/// Cell-exact (ansi) + content (txt) + render-level (html) byte gates +
/// pixel-exact gate at threshold 1.0 through the thread's cached renderer.
/// Writes `target/tuisnap/actual/` artifacts even when unmatched.
pub fn gate(name: &str, frame: &Frame) -> GroupedOutcome {
    RENDERER
        .with(|r| store().check_with(&mut r.borrow_mut(), name, frame, 1.0))
        .unwrap_or_else(|e| panic!("gate `{name}` failed: {e}"))
}

/// Fail-closed assertion: `matched` passes; `missing-approval` passes but is
/// logged as pending (expected for new names on a first run — explicit
/// `tuisnap accept --grouped` is the only bless, never the test); drift
/// after approval, dimension mismatch and corrupt approval fail.
pub fn assert_gated(outcome: &GroupedOutcome) {
    match outcome.status() {
        Status::Matched => eprintln!("baseline matched   {}", outcome.outcome.name),
        Status::MissingApproval => eprintln!("baseline PENDING   {}", outcome.outcome.name),
        _ => panic!("{}", outcome.ensure_matched().unwrap_err()),
    }
}

/// A ported-matrix capture: the same runner the `tuisnap run` CLI used.
pub fn run_and_assert(case: &Case) {
    let frame = run_once(&argv_for(case), &opts_for(case), &steps_for(case), SETTLE)
        .unwrap_or_else(|e| panic!("capture `{}` failed: {e:#}", case.name));
    assert_gated(&gate(&case.name, &frame));
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

/// Spawn a case's session for the pointer group (mouse/resize captures need
/// the live session after boot).
pub fn spawn(case: &Case) -> Session {
    Session::spawn(&argv_for(case), &opts_for(case))
        .unwrap_or_else(|e| panic!("spawn `{}` failed: {e:#}", case.name))
}

/// Boot: needle first. Live clocks starve a quiet-window wait_idle.
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

/// One `#[test]` per capture, generated from the static case tables so cargo
/// name filters work (`cargo nextest run --run-ignored only -E 'test(holla_)'`).
/// The test fn name is the capture name with `-` and `/` mapped to `_` (fn
/// names can't contain either); `Case.name` carries the full grouped name.
#[macro_export]
macro_rules! baseline_case {
    ($fn_name:ident => $case:expr) => {
        #[test]
        #[ignore = "visual baseline capture; run with --ignored"]
        fn $fn_name() {
            $crate::support::run_and_assert(&$case);
        }
    };
}
