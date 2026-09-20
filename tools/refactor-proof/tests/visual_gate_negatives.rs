//! Disposable fail-closed negatives for the grouped visual gate.
//!
//! Copies the tiny seed fixture, mutates only the copy, and requires nonzero
//! grouped-check exits for missing/duplicated keys, missing artifacts,
//! cell/style/geometry/cursor drift, wrong HTML argv identity, and stale
//! approved bytes. HTML comparison uses `full_render` so `provenance.argv`
//! is not masked by the cell-match fast path.

#![expect(
    clippy::print_stderr,
    reason = "per-case grouped-check exits are evidence, not library output"
)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tuisnap::grouped::{GroupedCheckOptions, GroupedStore};
use tuisnap::snapshot::Status;
use tuisnap::{Frame, Profile, Renderer, VENDORED_FACES};

const SEED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/visual-gate/seed.frame.json"
);
const NAME: &str = "gate/negatives/4x1/truecolor";
const MISSING_NAME: &str = "gate/missing/4x1/truecolor";
const DUP_NAME: &str = "gate/negatives/dup/4x1/truecolor";
const FROZEN_BIN: &str = "/Users/donbeave/Projects/terminal-components-claude/target/debug/holla";
const WRONG_BIN: &str =
    "/Users/donbeave/Projects/terminal-components-claude/.codex-runs/wrong/target/debug/holla";

const FULL_RENDER: GroupedCheckOptions = GroupedCheckOptions { full_render: true };

type TestError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
struct CaseResult {
    status: String,
    html_match: String,
    ansi_match: String,
    exit: i32,
    note: String,
}

fn log_root() -> PathBuf {
    if let Ok(path) = std::env::var("VISUAL_GATE_NEGATIVE_LOG") {
        let path = PathBuf::from(path);
        drop(fs::create_dir_all(&path));
        return path;
    }
    std::env::temp_dir().join("visual-gate-negatives")
}

fn copy_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn load_seed() -> Result<Frame, TestError> {
    let text = fs::read_to_string(SEED)?;
    let frame = Frame::from_json(&text).map_err(|error| error.to_string())?;
    let argv0 = frame.provenance.argv.first().map_or("", String::as_str);
    if argv0 != FROZEN_BIN {
        return Err(format!("seed argv[0] is {argv0}, want {FROZEN_BIN}").into());
    }
    Ok(frame)
}

fn store_at(root: &Path) -> GroupedStore {
    GroupedStore::new(&root.join("approved"))
        .with_actual_root(&root.join("actual"))
        .with_diff_root(&root.join("diff"))
}

fn renderer() -> Result<Renderer, TestError> {
    Profile::default_profile()
        .renderer(&VENDORED_FACES)
        .map_err(|error| error.to_string().into())
}

fn seed_approved(
    renderer: &mut Renderer,
    root: &Path,
    frame: &Frame,
) -> Result<GroupedStore, TestError> {
    let store = store_at(root);
    let first = store
        .check_with_options(renderer, NAME, frame, 1.0, &FULL_RENDER)
        .map_err(|error| error.to_string())?;
    if first.status() != Status::MissingApproval {
        return Err(format!("seed first check status {:?}", first.status()).into());
    }
    store.accept(NAME).map_err(|error| error.to_string())?;
    let matched = store
        .check_with_options(renderer, NAME, frame, 1.0, &FULL_RENDER)
        .map_err(|error| error.to_string())?;
    if matched.status() != Status::Matched {
        return Err(format!(
            "seed must match itself: {:?} {}",
            matched.status(),
            matched.outcome.note
        )
        .into());
    }
    let html = fs::read_to_string(&matched.approved.html)?;
    if !html.contains(FROZEN_BIN) {
        return Err("seed HTML does not embed frozen argv[0]".into());
    }
    Ok(store)
}

fn clone_store(src: &GroupedStore, dst: &Path) -> Result<GroupedStore, TestError> {
    copy_tree(src.approved_root(), &dst.join("approved"))?;
    Ok(store_at(dst))
}

fn gate_exit(
    renderer: &mut Renderer,
    store: &GroupedStore,
    name: &str,
    frame: &Frame,
) -> (i32, CaseResult) {
    match store.check_with_options(renderer, name, frame, 1.0, &FULL_RENDER) {
        Err(error) => (
            2,
            CaseResult {
                status: "error".into(),
                html_match: "none".into(),
                ansi_match: "none".into(),
                exit: 2,
                note: error.to_string(),
            },
        ),
        Ok(outcome) => {
            let exit = i32::from(outcome.ensure_matched().is_err());
            (
                exit,
                CaseResult {
                    status: outcome.status().as_str().to_owned(),
                    html_match: match outcome.html_match {
                        Some(true) => "true".into(),
                        Some(false) => "false".into(),
                        None => "none".into(),
                    },
                    ansi_match: match outcome.ansi_match {
                        Some(true) => "true".into(),
                        Some(false) => "false".into(),
                        None => "none".into(),
                    },
                    exit,
                    note: outcome.outcome.note,
                },
            )
        }
    }
}

fn write_case(log: &Path, case: &str, result: &CaseResult) -> Result<(), TestError> {
    fs::create_dir_all(log)?;
    let body = format!(
        "case={case}\nstatus={}\nansi_match={}\nhtml_match={}\nexit={}\nnote={}\n",
        result.status, result.ansi_match, result.html_match, result.exit, result.note
    );
    fs::write(log.join(format!("{case}.log")), body.as_bytes())?;
    eprintln!(
        "visual-gate-negative {case} status={} html_match={} ansi_match={} exit={}",
        result.status, result.html_match, result.ansi_match, result.exit
    );
    Ok(())
}

fn mutate_cell_and_style(frame: &Frame) -> Result<Frame, TestError> {
    let mut mutated = frame.clone();
    let cell = mutated.cells.first_mut().ok_or("seed frame has no cells")?;
    cell.symbol = "X".into();
    cell.mods.bold = true;
    Ok(mutated)
}

fn mutate_argv(frame: &Frame) -> Result<Frame, TestError> {
    let mut mutated = frame.clone();
    let argv0 = mutated
        .provenance
        .argv
        .first_mut()
        .ok_or("seed frame has no argv")?;
    *argv0 = WRONG_BIN.into();
    Ok(mutated)
}

fn mutate_html_argv(html: &str) -> Result<String, TestError> {
    if !html.contains(FROZEN_BIN) {
        return Err("approved HTML missing frozen argv[0]".into());
    }
    Ok(html.replace(FROZEN_BIN, WRONG_BIN))
}

fn mutate_geometry(frame: &Frame) -> Frame {
    Frame::blank(
        frame.cols.saturating_add(1),
        frame.rows,
        frame.provenance.clone(),
    )
}

fn mutate_cursor(frame: &Frame) -> Result<Frame, TestError> {
    let mut mutated = frame.clone();
    mutated.cursor.x = mutated.cursor.x.saturating_add(1);
    mutated.cursor.visible = true;
    Ok(mutated)
}

fn exclusive_names_exit(store: &GroupedStore, allowed: &[&str]) -> Result<i32, TestError> {
    let names = store.approved_names().map_err(|error| error.to_string())?;
    let extra = names
        .iter()
        .filter(|name| !allowed.contains(&name.as_str()))
        .count();
    let missing = allowed
        .iter()
        .filter(|name| !names.iter().any(|have| have == *name))
        .count();
    Ok(i32::from(
        extra != 0 || missing != 0 || names.len() != allowed.len(),
    ))
}

#[test]
fn visual_gate_fails_closed_on_missing_mutated_and_argv() -> Result<(), TestError> {
    let log = log_root();
    fs::create_dir_all(&log)?;
    let seed = load_seed()?;
    let scratch = tempfile::tempdir()?;
    let mut renderer = renderer()?;
    let original = seed_approved(&mut renderer, &scratch.path().join("original"), &seed)?;

    let missing_root = scratch.path().join("missing-artifact");
    let missing_store = clone_store(&original, &missing_root)?;
    fs::remove_file(missing_store.approved_root().join(format!("{NAME}.html")))?;
    let (missing_exit, missing) = gate_exit(&mut renderer, &missing_store, NAME, &seed);
    write_case(&log, "missing_artifact", &missing)?;

    let (missing_key_exit, missing_key) = gate_exit(&mut renderer, &original, MISSING_NAME, &seed);
    write_case(&log, "missing_key", &missing_key)?;

    let dup_root = scratch.path().join("duplicated-key");
    let dup_store = clone_store(&original, &dup_root)?;
    let dup_first = dup_store
        .check_with_options(&mut renderer, DUP_NAME, &seed, 1.0, &FULL_RENDER)
        .map_err(|error| error.to_string())?;
    if dup_first.status() != Status::MissingApproval {
        return Err(format!("dup seed status {:?}", dup_first.status()).into());
    }
    dup_store
        .accept(DUP_NAME)
        .map_err(|error| error.to_string())?;
    let duplicated_exit = exclusive_names_exit(&dup_store, &[NAME])?;
    write_case(
        &log,
        "duplicated_key",
        &CaseResult {
            status: "extra-key".into(),
            html_match: "none".into(),
            ansi_match: "none".into(),
            exit: duplicated_exit,
            note: "approved store contains a second key besides the exclusive set".into(),
        },
    )?;

    let mutated_frame = mutate_cell_and_style(&seed)?;
    let (mutated_exit, mutated) = gate_exit(&mut renderer, &original, NAME, &mutated_frame);
    write_case(&log, "mutated_cell_style", &mutated)?;

    let geometry_frame = mutate_geometry(&seed);
    let (geometry_exit, geometry) = gate_exit(&mut renderer, &original, NAME, &geometry_frame);
    write_case(&log, "mutated_geometry", &geometry)?;

    let cursor_frame = mutate_cursor(&seed)?;
    let (cursor_exit, cursor) = gate_exit(&mut renderer, &original, NAME, &cursor_frame);
    write_case(&log, "mutated_cursor", &cursor)?;

    let argv_frame = mutate_argv(&seed)?;
    let (argv_exit, argv) = gate_exit(&mut renderer, &original, NAME, &argv_frame);
    write_case(&log, "html_argv", &argv)?;

    let html_root = scratch.path().join("html-path");
    let html_store = clone_store(&original, &html_root)?;
    let approved_html = html_store.approved_root().join(format!("{NAME}.html"));
    let rewritten = mutate_html_argv(&fs::read_to_string(&approved_html)?)?;
    if rewritten == fs::read_to_string(&approved_html)? {
        return Err("HTML argv rewrite did not change bytes".into());
    }
    fs::write(&approved_html, rewritten.as_bytes())?;
    let (html_file_exit, html_file) = gate_exit(&mut renderer, &html_store, NAME, &seed);
    write_case(&log, "html_argv_file", &html_file)?;

    let stale_root = scratch.path().join("stale-evidence");
    let stale_store = clone_store(&original, &stale_root)?;
    let approved_ansi = stale_store.approved_root().join(format!("{NAME}.ansi"));
    let mut stale_bytes = fs::read(&approved_ansi)?;
    stale_bytes.extend_from_slice(b"STALE");
    fs::write(&approved_ansi, stale_bytes)?;
    let (stale_exit, stale) = gate_exit(&mut renderer, &stale_store, NAME, &seed);
    write_case(&log, "stale_evidence", &stale)?;

    let exits = format!(
        "missing_artifact={missing_exit}\nmissing_key={missing_key_exit}\nduplicated_key={duplicated_exit}\nmutated_cell_style={mutated_exit}\nmutated_geometry={geometry_exit}\nmutated_cursor={cursor_exit}\nhtml_argv={argv_exit}\nhtml_argv_file={html_file_exit}\nstale_evidence={stale_exit}\n"
    );
    fs::write(log.join("exits.txt"), exits.as_bytes())?;
    eprintln!("visual-gate-negative exits\n{exits}");

    assert_ne!(
        missing_exit, 0,
        "missing artifact must fail closed: {missing:?}"
    );
    assert_eq!(missing.status, Status::MissingApproval.as_str());
    assert_ne!(
        missing_key_exit, 0,
        "missing key must fail closed: {missing_key:?}"
    );
    assert_eq!(missing_key.status, Status::MissingApproval.as_str());
    assert_ne!(
        duplicated_exit, 0,
        "duplicated extra key must fail exclusive inventory"
    );
    assert_ne!(
        mutated_exit, 0,
        "mutated cell/style must fail closed: {mutated:?}"
    );
    assert_eq!(mutated.status, Status::CellsDiffer.as_str());
    assert_eq!(mutated.ansi_match, "false");
    assert_ne!(
        geometry_exit, 0,
        "mutated geometry must fail closed: {geometry:?}"
    );
    assert_ne!(
        cursor_exit, 0,
        "mutated cursor must fail closed: {cursor:?}"
    );
    assert_eq!(cursor.html_match, "false");
    assert_ne!(
        argv_exit, 0,
        "HTML argv identity must fail closed: {argv:?}"
    );
    assert_eq!(argv.html_match, "false");
    assert_ne!(
        html_file_exit, 0,
        "wrong binary path in HTML must fail closed: {html_file:?}"
    );
    assert_eq!(html_file.html_match, "false");
    assert_ne!(
        stale_exit, 0,
        "stale approved bytes must fail closed: {stale:?}"
    );
    Ok(())
}
