use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tui_test::Backend;
use tui_test::api::{
    AutomaticRecording, KeyAction, Operation, OperationResult, RawOutputCaptureOptions, RunOptions,
    Timeouts,
};
use tui_test::launch::{LaunchEnvMode, LaunchOptions};
use tui_test::profile::Profile;
use tui_test::render::style::Style;
use tui_test::runtime::Session;
use tui_test::snapshot::{SnapshotScope, TerminalSnapshot};
use tui_test::terminal::cell::Attrs;
use tui_test::wait::{WaitQuietMode, WaitQuietOptions};
use tuiscotti::check::{CheckOutcome, FrozenStore};
use tuiscotti::render_profile::{PROFILE_NAME, platform_key, resolve_profile};
use tuiscotti::sample::{Observation, Sample};

mod source_manifest;

const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 40;
const FIRST_FLOW_DENOMINATOR: usize = 4;
const HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR: usize = 8;
const HOLLA_RESULT_LIST_X_START: u32 = 0;
const HOLLA_RESULT_LIST_X_END: u32 = 78;
const HOLLA_RESULT_LIST_Y_START: u32 = 6;
const HOLLA_RESULT_LIST_Y_END: u32 = 38;
const HELP_TIMEOUT_MS: u64 = 15_000;
const EXIT_TIMEOUT_MS: u64 = 30_000;
const QUIET_TIMEOUT: Duration = Duration::from_secs(15);
const QUIET_INTERVAL: Duration = Duration::from_millis(400);
const BUILD_EXPORT_SHA256: &str = env!("TERMROCK_SUITE_EXPORT_SHA256");
const EXPORT_LOADER: &str = r###"
import hashlib
import json
import os
import pathlib
import re
import stat
import sys

def fail(message):
    raise SystemExit(message)

def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail("duplicate manifest key")
        result[key] = value
    return result

def safe_read(path):
    if not path.is_absolute() or ".." in path.parts:
        fail("unsafe export file path")
    components = str(path).split("/")
    if components[0] != "" or any(part in ("", ".", "..") for part in components[1:]):
        fail("unsafe export file path")
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for component in path.parts[1:-1]:
            info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                fail("symlink or non-directory in export file path")
            next_descriptor = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        info = os.stat(path.parts[-1], dir_fd=descriptor, follow_symlinks=False)
        if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
            fail("export file is not regular")
        file_descriptor = os.open(path.parts[-1], os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptor)
        try:
            if not stat.S_ISREG(os.fstat(file_descriptor).st_mode):
                fail("export file is not regular")
            chunks = []
            while True:
                chunk = os.read(file_descriptor, 1024 * 1024)
                if not chunk:
                    return b"".join(chunks)
                chunks.append(chunk)
        finally:
            os.close(file_descriptor)
    finally:
        os.close(descriptor)

arguments = sys.argv[1:]
if len(arguments) < 4:
    fail("missing export authentication arguments")
manifest_path = pathlib.Path(arguments[0])
helper_path = pathlib.Path(arguments[1])
expected = arguments[2]
helper_arguments = arguments[3:]
if re.fullmatch(r"[0-9a-f]{64}", expected) is None:
    fail("invalid export seal")
if manifest_path.parent != helper_path.parent or manifest_path.name != "export-manifest.json" or helper_path.name != "export.py":
    fail("unexpected export helper paths")
manifest_bytes = safe_read(manifest_path)
if hashlib.sha256(manifest_bytes).hexdigest() != expected:
    fail("export manifest seal mismatch")
try:
    manifest = json.loads(manifest_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
except (UnicodeDecodeError, json.JSONDecodeError) as error:
    fail("invalid sealed export manifest: " + str(error))
files = manifest.get("files") if isinstance(manifest, dict) else None
if not isinstance(files, list):
    fail("sealed export manifest has no file entries")
helpers = [entry for entry in files if isinstance(entry, dict) and entry.get("path") == "export.py"]
if len(helpers) != 1:
    fail("sealed export manifest must name one helper")
entry = helpers[0]
if set(entry) != {"path", "mode", "size", "sha256"}:
    fail("sealed helper entry has unknown fields")
if type(entry["size"]) is not int or entry["size"] < 0 or re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]) is None:
    fail("sealed helper entry is malformed")
helper_bytes = safe_read(helper_path)
if len(helper_bytes) != entry["size"] or hashlib.sha256(helper_bytes).hexdigest() != entry["sha256"]:
    fail("sealed helper digest mismatch")
sys.argv = [str(helper_path), *helper_arguments]
exec(compile(helper_bytes, str(helper_path), "exec"), {"__name__": "__main__", "__file__": str(helper_path)})
"###;
const FIRST_CHECKPOINTS: [&str; 4] = ["opened", "help", "returned", "exited"];
const SHOWCASE_HELP_CHECKPOINTS: [&str; 7] = [
    "opened",
    "help-start",
    "help-scrolled",
    "returned",
    "help-reopened",
    "restored-after-modal-q",
    "exited",
];
const HOLLA_FIRST_USE_CHECKPOINTS: [&str; HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR] = [
    "opened",
    "initial-resource",
    "home-restored",
    "query-selected",
    "query-resource",
    "query-restored",
    "query-cleared",
    "exited",
];

#[derive(Clone, Copy)]
enum Journey {
    First,
    ShowcaseHelp80x24,
    HollaFirstUseAction,
}

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    binary: &'static str,
    args: &'static [&'static str],
    initial_marker: &'static str,
    help_key: &'static str,
    help_marker: &'static str,
    quit_key: &'static str,
    cols: u16,
    rows: u16,
    checkpoint_labels: &'static [&'static str],
    journey: Journey,
}

const CASES: [Case; FIRST_FLOW_DENOMINATOR] = [
    Case {
        id: "first/showcase",
        binary: "showcase",
        args: &[
            "--page",
            "overview",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "Overview",
        help_key: "?",
        help_marker: "Keyboard & mouse",
        quit_key: "q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        checkpoint_labels: &FIRST_CHECKPOINTS,
        journey: Journey::First,
    },
    Case {
        id: "first/holla",
        binary: "holla",
        args: &[
            "--scenario",
            "first-use",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "holla❯",
        help_key: "F1",
        help_marker: "Key reference",
        quit_key: "Ctrl+q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        checkpoint_labels: &FIRST_CHECKPOINTS,
        journey: Journey::First,
    },
    Case {
        id: "first/jackin",
        binary: "jackin",
        args: &[
            "--scenario",
            "returning",
            "--motion",
            "paused",
            "--frame",
            "0",
            "--color",
            "truecolor",
        ],
        initial_marker: "Current directory",
        help_key: "?",
        help_marker: "Keyboard shortcuts",
        quit_key: "q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        checkpoint_labels: &FIRST_CHECKPOINTS,
        journey: Journey::First,
    },
    Case {
        id: "first/tablepro",
        binary: "tablepro",
        args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
        initial_marker: "TablePro",
        help_key: "?",
        help_marker: "Keyboard shortcuts",
        quit_key: "q",
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        checkpoint_labels: &FIRST_CHECKPOINTS,
        journey: Journey::First,
    },
];

const SHOWCASE_HELP_CASE: Case = Case {
    id: "showcase-help/80x24-v1",
    binary: "showcase",
    args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
    initial_marker: "Overview",
    help_key: "?",
    help_marker: "Keyboard & mouse",
    quit_key: "q",
    cols: 80,
    rows: 24,
    checkpoint_labels: &SHOWCASE_HELP_CHECKPOINTS,
    journey: Journey::ShowcaseHelp80x24,
};

const HOLLA_FIRST_USE_ACTION_CASE: Case = Case {
    id: "holla-first-use/action-identity-v1",
    binary: "holla",
    args: &[
        "--scenario",
        "first-use",
        "--motion",
        "paused",
        "--frame",
        "0",
        "--color",
        "truecolor",
    ],
    initial_marker: "holla❯",
    help_key: "",
    help_marker: "",
    quit_key: "Ctrl+q",
    cols: DEFAULT_COLS,
    rows: DEFAULT_ROWS,
    checkpoint_labels: &HOLLA_FIRST_USE_CHECKPOINTS,
    journey: Journey::HollaFirstUseAction,
};

#[derive(Debug)]
struct Checkpoint {
    label: String,
    checkpoint_id: String,
    sample_id: String,
    offset: u64,
    captured_at_unix_ms: u128,
    snapshot: TerminalSnapshot,
    sample_bytes: Vec<u8>,
    png_sha256: String,
    rgba_sha256: String,
}

struct FailureScreenEvidence {
    record: Value,
    snapshot: TerminalSnapshot,
    state_observation: Value,
    child_active: bool,
    screen_observable: bool,
}

pub(crate) type AppResult<T> = Result<T, Box<dyn Error>>;

pub(crate) fn run_case_id(case_id: &str) -> AppResult<()> {
    let case = case_by_id(case_id).ok_or_else(|| format!("unknown functional case {case_id:?}"))?;
    let suite = suite_root()?;
    let bin_dir = required_absolute_directory("TERMROCK_BIN_DIR")?;
    let out_root = required_absolute_directory("TERMROCK_OUT_DIR")?;
    let case_out = out_root.join(case.id);
    let export_provenance = verify_export(&bin_dir, &case_out)?;
    prepare_empty_output(&case_out)?;

    let source_manifest = match source_manifest::load_verified(&bin_dir) {
        Ok(manifest) => manifest,
        Err(error) => {
            write_json_new(
                &case_out.join("setup-error.json"),
                &json!({"stage":"source-manifest","error":error.to_string()}),
            )?;
            return Err(error);
        }
    };
    if let Err(error) = verify_application_root(&source_manifest, &export_provenance) {
        write_json_new(
            &case_out.join("setup-error.json"),
            &json!({"stage":"application-source-binding","error":error.to_string()}),
        )?;
        return Err(error);
    }

    let cwd = case_out.join("working-directory");
    if let Err(error) = copy_case_fixture(&suite.join("data").join(case.id), &cwd) {
        write_json_new(
            &case_out.join("setup-error.json"),
            &json!({"stage":"fixture-copy","error":error.to_string()}),
        )?;
        return Err(error);
    }
    let frozen_store =
        match FrozenStore::new(&suite.join("expected"), &case_out.join("frozen-checks")) {
            Ok(store) => store,
            Err(error) => {
                write_json_new(
                    &case_out.join("setup-error.json"),
                    &json!({"stage":"frozen-store","error":error.to_string()}),
                )?;
                return Err(error.into());
            }
        };
    let executable = std::env::current_exe()?.canonicalize()?;
    let executable_sha256_before = sha256_file(&executable)?;
    let execution = json!({
        "compiled_export_sha256": BUILD_EXPORT_SHA256,
        "suite_lock_sha256": export_provenance["suite_lock_sha256"],
        "tuiscotti_commit": export_provenance["tuiscotti_commit"],
        "engine": export_provenance["engine"],
        "application_source": source_manifest,
        "test_executable": executable,
        "test_executable_sha256_before": executable_sha256_before,
        "bin_dir": bin_dir,
        "out_dir": out_root,
        "case_output": case_out,
    });
    let result = run_case(
        case,
        &bin_dir.join(case.binary),
        &suite,
        &case_out,
        &cwd,
        &frozen_store,
        &execution,
    )?;
    let executable_sha256_after = sha256_file(&executable)?;
    let execution_record = json!({
        "schema":"termrock-functional-execution.v1",
        "case_id":case.id,
        "compiled_export_sha256":BUILD_EXPORT_SHA256,
        "test_executable":executable,
        "test_executable_sha256_before":executable_sha256_before,
        "test_executable_sha256_after":executable_sha256_after,
        "test_executable_unchanged":executable_sha256_before == executable_sha256_after,
        "suite_lock_sha256":export_provenance["suite_lock_sha256"],
        "tuiscotti_commit":export_provenance["tuiscotti_commit"],
        "engine":export_provenance["engine"],
        "bin_dir":bin_dir,
        "out_dir":out_root,
        "case_output":case_out,
    });
    write_json_new(
        &case_out.join("execution-provenance.json"),
        &execution_record,
    )?;
    if executable_sha256_before != executable_sha256_after {
        return Err("functional test executable changed during the test".into());
    }
    if result["status"] != "passed" {
        return Err(format!(
            "case {} failed after saving evidence under {}: {}",
            case.id,
            case_out.display(),
            result["error"]
        )
        .into());
    }
    Ok(())
}

fn case_by_id(case_id: &str) -> Option<Case> {
    CASES
        .iter()
        .copied()
        .chain([SHOWCASE_HELP_CASE, HOLLA_FIRST_USE_ACTION_CASE])
        .find(|case| case.id == case_id)
}

fn required_absolute_directory(name: &str) -> AppResult<PathBuf> {
    let value = std::env::var_os(name).ok_or_else(|| format!("{name} is required"))?;
    let path = PathBuf::from(value);
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        return Err(format!("{name} must be a normalized absolute path").into());
    }
    let canonical = path.canonicalize()?;
    if canonical != path || !canonical.is_dir() {
        return Err(format!("{name} must name a canonical directory").into());
    }
    Ok(canonical)
}

fn copy_case_fixture(source: &Path, destination: &Path) -> AppResult<()> {
    let source_info = fs::symlink_metadata(source)?;
    if source_info.file_type().is_symlink() || !source_info.is_dir() {
        return Err(format!(
            "case fixture is not a regular directory: {}",
            source.display()
        )
        .into());
    }
    if fs::symlink_metadata(destination).is_ok() {
        return Err(format!(
            "case working directory already exists: {}",
            destination.display()
        )
        .into());
    }
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let info = fs::symlink_metadata(&source_path)?;
        if info.file_type().is_symlink() {
            return Err(
                format!("case fixture contains a symlink: {}", source_path.display()).into(),
            );
        }
        if info.is_dir() {
            copy_case_fixture(&source_path, &destination_path)?;
        } else if info.is_file() {
            write_new(&destination_path, &fs::read(&source_path)?)?;
            fs::set_permissions(&destination_path, info.permissions())?;
        } else {
            return Err(format!(
                "case fixture contains a special file: {}",
                source_path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn run_case(
    case: Case,
    binary: &Path,
    suite: &Path,
    case_out: &Path,
    cwd: &Path,
    frozen_store: &FrozenStore,
    execution: &Value,
) -> AppResult<Value> {
    let case_out = case_out.to_path_buf();
    let cwd = cwd.to_path_buf();
    let case_start = Instant::now();
    let started_at = unix_ms();
    let raw_path = case_out.join("raw.ansi");
    let session = Session::new(format!("termrock-live-{}", case.id.replace('/', "-")));
    let mut spawned = false;
    let mut child_exited = false;
    let mut final_quit_attempted = false;
    let mut checkpoints = Vec::with_capacity(case.checkpoint_labels.len());
    let mut actions = Vec::new();
    let mut semantic_assertions = Vec::new();
    let mut failure_finalization = Value::Null;
    let mut process_state = Value::Null;
    let work = (|| -> AppResult<()> {
        let launch = LaunchOptions::new(binary.as_os_str().to_owned())
            .with_args(case.args.iter().copied())
            .with_cwd(&cwd)
            .with_env_mode(LaunchEnvMode::Cleared)
            .with_envs([
                ("LANG", "C.UTF-8"),
                ("LC_ALL", "C.UTF-8"),
                ("TZ", "UTC"),
                ("TERM", "xterm-256color"),
                ("COLORTERM", "truecolor"),
            ]);
        let run_options = RunOptions {
            backend: Backend::default(),
            program: String::new(),
            args: Vec::new(),
            profile: Profile::default(),
            cols: case.cols,
            rows: case.rows,
            cwd: Some(cwd.to_string_lossy().into_owned()),
            env: Vec::new(),
            wait_ready: Some(false),
            restart: false,
            timeouts: Timeouts::default(),
            recording: AutomaticRecording::default(),
            style: Style::default(),
        };
        session.spawn_with_raw_output_capture(
            launch,
            run_options,
            RawOutputCaptureOptions::new(&raw_path),
        )?;
        spawned = true;

        let sample_profile = resolve_profile(PROFILE_NAME)?;
        ensure_child_active(&session)?;
        session
            .get_by_text(case.initial_marker)
            .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
        wait_visual_quiet(&session)?;
        if !matches!(case.journey, Journey::HollaFirstUseAction) {
            let initial_text = match session.execute(Operation::Text { full: true })? {
                OperationResult::Text(text) => text,
                other => return Err(format!("unexpected text result: {other:?}").into()),
            };
            if initial_text.trim().is_empty() {
                return Err("initial visible marker appeared but screen text is empty".into());
            }
        }
        checkpoints.push(capture_checkpoint(
            &session,
            sample_profile,
            case,
            case.checkpoint_labels[0],
            &case_out,
        )?);
        if matches!(case.journey, Journey::HollaFirstUseAction)
            && snapshot_text(last_checkpoint_snapshot(&checkpoints)?)
                .trim()
                .is_empty()
        {
            return Err("initial visible marker appeared but captured screen is empty".into());
        }

        match case.journey {
            Journey::First => {
                actions
                    .push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
                send_key(&session, case.help_key)?;
                session
                    .get_by_text(case.help_marker)
                    .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
                wait_visual_quiet(&session)?;
                checkpoints.push(capture_checkpoint(
                    &session,
                    sample_profile,
                    case,
                    case.checkpoint_labels[1],
                    &case_out,
                )?);

                actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
                send_key(&session, "Escape")?;
                session
                    .get_by_text(case.help_marker)
                    .wait_hidden(Some(HELP_TIMEOUT_MS))?;
                session
                    .get_by_text(case.initial_marker)
                    .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
                wait_visual_quiet(&session)?;
                checkpoints.push(capture_checkpoint(
                    &session,
                    sample_profile,
                    case,
                    case.checkpoint_labels[2],
                    &case_out,
                )?);
            }
            Journey::ShowcaseHelp80x24 => run_showcase_help_steps(
                &session,
                sample_profile,
                case,
                &case_out,
                &mut checkpoints,
                &mut actions,
            )?,
            Journey::HollaFirstUseAction => run_holla_first_use_action_steps(
                &session,
                sample_profile,
                case,
                &case_out,
                &mut checkpoints,
                &mut actions,
                &mut semantic_assertions,
            )?,
        }

        actions.push(json!({"event":"key_press","key":case.quit_key,"at_unix_ms":unix_ms()}));
        final_quit_attempted = true;
        send_key(&session, case.quit_key)?;
        session.execute(Operation::WaitExit {
            timeout_ms: Some(EXIT_TIMEOUT_MS),
        })?;
        process_state = match session.execute(Operation::State)? {
            OperationResult::State(state) => json!({
                "wait_exit_completed": true,
                "exited": state.exited,
                "exit_signal": state.exit_signal,
                "cols": state.cols,
                "rows": state.rows,
            }),
            other => return Err(format!("unexpected process state result: {other:?}").into()),
        };
        child_exited = true;
        write_json_new(&case_out.join("process-state.json"), &process_state)?;
        if process_state["exited"] != 0 || !process_state["exit_signal"].is_null() {
            return Err(format!("child did not exit cleanly: {process_state}").into());
        }

        let status = session
            .raw_output_capture_status()
            .ok_or("raw output capture status is unavailable")?;
        if !status.setup_complete || !status.complete || !status.errors.is_empty() {
            return Err(format!("raw capture is incomplete or has errors: {status:?}").into());
        }
        let processed = status
            .processed_output_offset
            .ok_or("processed output byte offset overflowed")?;
        let captured = status
            .captured_output_bytes
            .ok_or("captured output byte count overflowed")?;
        if !status.drained || !status.flushed || captured != processed {
            return Err(format!(
                "raw capture drain mismatch: drained={}, flushed={}, captured={}, processed={}",
                status.drained, status.flushed, captured, processed
            )
            .into());
        }
        wait_visual_quiet(&session)?;
        let exit_label = case
            .checkpoint_labels
            .last()
            .copied()
            .ok_or("case has no exit checkpoint")?;
        if exit_label != "exited" {
            return Err(format!("case ends with unexpected checkpoint {exit_label:?}").into());
        }
        checkpoints.push(capture_checkpoint(
            &session,
            sample_profile,
            case,
            exit_label,
            &case_out,
        )?);
        let raw = fs::read(&raw_path)?;
        if u64::try_from(raw.len())? != captured {
            return Err(format!(
                "raw file length {} does not match reported byte count {captured}",
                raw.len()
            )
            .into());
        }
        let exit_checkpoint = checkpoints
            .last()
            .ok_or("exit checkpoint was not captured")?;
        if exit_checkpoint.offset != captured
            || exit_checkpoint.offset != processed
            || exit_checkpoint.offset != u64::try_from(raw.len())?
        {
            return Err(format!(
                "exit checkpoint offset {} does not match captured {}, processed {}, and raw file length {}",
                exit_checkpoint.offset,
                captured,
                processed,
                raw.len()
            )
            .into());
        }
        for checkpoint in &checkpoints {
            if checkpoint.offset > captured {
                return Err(format!(
                    "checkpoint {} offset {} exceeds drained stream {captured}",
                    checkpoint.checkpoint_id, checkpoint.offset
                )
                .into());
            }
            let end = usize::try_from(checkpoint.offset)?;
            let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
            fs::create_dir_all(&checkpoint_dir)?;
            write_new(&checkpoint_dir.join("raw-prefix.ansi"), &raw[..end])?;
            write_json_new(
                &checkpoint_dir.join("capture.json"),
                &json!({
                    "checkpoint_id": checkpoint.checkpoint_id,
                    "sample_id": checkpoint.sample_id,
                    "processed_output_offset": checkpoint.offset,
                    "raw_prefix_sha256": sha256(&raw[..end]),
                    "raw_stream_sha256": sha256(&raw),
                    "raw_stream_bytes": captured,
                }),
            )?;
        }

        {
            for checkpoint in &checkpoints {
                let outcome = frozen_store.check(&checkpoint.sample_bytes)?;
                let mut check_value = check_json(&outcome);
                if let CheckOutcome::Failure(failure) = &outcome {
                    let baseline_path = suite
                        .join("expected")
                        .join(format!("{}.tui.json", checkpoint.sample_id));
                    if baseline_path.is_file()
                        && let Ok(bytes) = fs::read(baseline_path)
                        && let Ok(baseline) = Sample::from_bytes(&bytes)
                        && let Some(object) = check_value.as_object_mut()
                    {
                        object.insert(
                            "first_difference".to_string(),
                            json!(first_difference(baseline.snapshot(), &checkpoint.snapshot)),
                        );
                        object.insert(
                            "changed_fields".to_string(),
                            json!(failure.changed_fields()),
                        );
                        object.insert(
                            "changed_cells".to_string(),
                            json!(
                                failure
                                    .changed_cells()
                                    .iter()
                                    .map(|cell| json!({
                                        "col": cell.col(),
                                        "row": cell.row(),
                                        "fields": cell.fields(),
                                        "actual_text": cell.actual_text(),
                                        "baseline_text": cell.baseline_text(),
                                    }))
                                    .collect::<Vec<_>>()
                            ),
                        );
                    }
                }
                let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
                write_json_new(&checkpoint_dir.join("check.json"), &check_value)?;
            }
        }
        Ok(())
    })();

    let mut cleanup_error = None;
    let mut closed_for_cleanup = false;
    if work.is_err() && spawned {
        if matches!(case.journey, Journey::HollaFirstUseAction) && !final_quit_attempted {
            let (finalization, state, exited) = finalize_holla_after_error(&session, &mut actions);
            failure_finalization = finalization;
            process_state = state;
            child_exited = exited;
        }
        if !child_exited {
            closed_for_cleanup = true;
            if let Err(error) = session.close() {
                cleanup_error = Some(error.to_string());
            }
        }
    }
    if let Some(status) = session.raw_output_capture_status() {
        let mut status_value = serde_json::to_value(&status)?;
        if let Some(object) = status_value.as_object_mut() {
            object.insert(
                "raw_file_bytes".to_string(),
                json!(fs::metadata(&raw_path).ok().map(|metadata| metadata.len())),
            );
            object.insert(
                "exit_checkpoint_offset".to_string(),
                json!(
                    checkpoints
                        .last()
                        .filter(|checkpoint| checkpoint.label == "exited")
                        .map(|checkpoint| checkpoint.offset)
                ),
            );
            object.insert(
                "observed_after_cleanup_close".to_string(),
                json!(closed_for_cleanup),
            );
            object.insert("cleanup_error".to_string(), json!(cleanup_error.clone()));
        }
        write_json_new(&case_out.join("raw-status.json"), &status_value)?;
    }
    let checkpoint_evidence_errors = if work.is_err() {
        let errors = preserve_reached_checkpoint_prefixes(&raw_path, &case_out, &checkpoints);
        if !errors.is_empty() {
            write_json_new(&case_out.join("checkpoint-evidence-errors.json"), &errors)?;
        }
        errors
    } else {
        Vec::new()
    };
    write_json_new(
        &case_out.join("actions.json"),
        &json!({
            "schema":"termrock-live-actions.v1",
            "started_at_unix_ms":started_at,
            "finished_at_unix_ms":unix_ms(),
            "actions":actions,
        }),
    )?;
    if matches!(case.journey, Journey::HollaFirstUseAction) {
        write_json_new(
            &case_out.join("semantic-assertions.json"),
            &semantic_assertions,
        )?;
    }

    let (process_status, work_error) = match work {
        Ok(()) => ("complete", None),
        Err(error) => ("incomplete", Some(error.to_string())),
    };
    let (check_failures, checkpoints_passed) = if process_status == "complete" {
        let mut failures = 0;
        let mut passed = 0;
        for checkpoint in &checkpoints {
            let path = case_out
                .join("checkpoints")
                .join(&checkpoint.label)
                .join("check.json");
            let is_pass = fs::read(path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .is_some_and(|value| value["status"] == "pass");
            if is_pass {
                passed += 1;
            } else {
                failures += 1;
            }
        }
        (failures, passed)
    } else {
        (0, 0)
    };
    let final_status = if process_status != "complete" || check_failures > 0 {
        "failed"
    } else {
        "passed"
    };
    let error = if let Some(error) = work_error {
        json!(error)
    } else if check_failures > 0 {
        json!(format!("{check_failures} frozen checkpoint checks failed"))
    } else {
        Value::Null
    };
    let mut result = json!({
        "schema":"termrock-live-flow.v1",
        "id":case.id,
        "execution":execution,
        "binary":case.binary,
        "initial_marker":case.initial_marker,
        "status":final_status,
        "process_status":process_status,
        "error":error,
        "cleanup_error":cleanup_error,
        "started_at_unix_ms":started_at,
        "finished_at_unix_ms":unix_ms(),
        "live_process_elapsed_ms":case_start.elapsed().as_millis() as u64,
        "expected_checkpoints":case.checkpoint_labels,
        "checkpoints_completed":checkpoints.len(),
        "checkpoints_passed":checkpoints_passed,
        "checkpoint_records":checkpoints.iter().map(|checkpoint| json!({
            "id":checkpoint.checkpoint_id,
            "sample_id":checkpoint.sample_id,
            "label":checkpoint.label,
            "captured_at_unix_ms":checkpoint.captured_at_unix_ms,
            "processed_output_offset":checkpoint.offset,
            "sample_sha256":sha256(&checkpoint.sample_bytes),
            "png_sha256":checkpoint.png_sha256,
            "rgba_sha256":checkpoint.rgba_sha256,
            "sample_path":format!("{}/checkpoints/{}/sample.tui.json", case.id, checkpoint.label),
            "expected_path":format!("expected/{}.tui.json", checkpoint.sample_id),
        })).collect::<Vec<_>>(),
        "process_state":process_state,
        "checkpoint_evidence_errors":checkpoint_evidence_errors,
        "check_failures":check_failures,
        "reference_approval":"not performed",
    });
    if matches!(case.journey, Journey::HollaFirstUseAction)
        && let Some(object) = result.as_object_mut()
    {
        object.insert(
            "semantic_assertions".to_string(),
            json!(semantic_assertions),
        );
        object.insert("failure_finalization".to_string(), failure_finalization);
    }
    write_json_new(&case_out.join("result.json"), &result)?;
    Ok(result)
}

fn run_showcase_help_steps(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    actions: &mut Vec<Value>,
) -> AppResult<()> {
    actions.push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
    send_key(session, case.help_key)?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[1],
        case_out,
    )?);
    let help_start_index = checkpoints.len() - 1;
    session
        .get_by_text("Mouse")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    actions.push(json!({"event":"key_press","key":"PageDown","at_unix_ms":unix_ms()}));
    send_key(session, "PageDown")?;
    session
        .get_by_text("Mouse")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    let help_scrolled =
        capture_checkpoint(session, profile, case, case.checkpoint_labels[2], case_out)?;
    let scrolled_text_changed = visible_text_changed(
        &checkpoints[help_start_index].snapshot,
        &help_scrolled.snapshot,
    );
    checkpoints.push(help_scrolled);
    if !scrolled_text_changed {
        return Err("PageDown did not change visible help text or range".into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text(case.help_marker)
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[3],
        case_out,
    )?);

    actions.push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
    send_key(session, case.help_key)?;
    session
        .get_by_text(case.help_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    let help_reopened =
        capture_checkpoint(session, profile, case, case.checkpoint_labels[4], case_out)?;
    let help_reopened_index = checkpoints.len();
    checkpoints.push(help_reopened);
    assert_same_checkpoint_state(
        &checkpoints[help_start_index],
        &checkpoints[help_reopened_index],
        "reopened help did not return to the initial help state",
    )?;

    actions.push(json!({"event":"key_press","key":"q","at_unix_ms":unix_ms()}));
    send_key(session, "q")?;
    session
        .get_by_text(case.help_marker)
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("Quit the showcase?")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    ensure_child_active(session)?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[5],
        case_out,
    )?);
    Ok(())
}

fn run_holla_first_use_action_steps(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    checkpoints: &mut Vec<Checkpoint>,
    actions: &mut Vec<Value>,
    semantic_assertions: &mut Vec<Value>,
) -> AppResult<()> {
    let raw_path = case_out.join("raw.ansi");
    let initial_assertion = holla_home_assertion(
        "initial",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        true,
        false,
    );
    semantic_assertions.push(initial_assertion.clone());

    actions.push(json!({"event":"key_press","key":"Enter","at_unix_ms":unix_ms()}));
    send_key(session, "Enter")?;
    if let Err(wait_error) = session
        .get_by_text("System resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))
    {
        let quiet_error = wait_visual_quiet(session)
            .err()
            .map(|error| error.to_string());
        match capture_failure_evidence(
            session,
            profile,
            case,
            case_out,
            &raw_path,
            "initial-resource",
        ) {
            Ok(evidence) => {
                let actual_text = snapshot_text(&evidence.snapshot);
                let mut assertion =
                    holla_resource_page_assertion("initial-resource", &evidence.snapshot);
                let expected_page_visible = assertion_passed(&assertion);
                let screen_quiet = quiet_error.is_none();
                let wrong_destination = evidence.child_active
                    && evidence.screen_observable
                    && screen_quiet
                    && actual_text.contains("System snapshot")
                    && !actual_text.contains("System resources");
                if let Some(object) = assertion.as_object_mut() {
                    object.insert(
                        "locator_wait_error".to_string(),
                        json!(wait_error.to_string()),
                    );
                    object.insert("visual_quiet_error".to_string(), json!(quiet_error));
                    object.insert("failure_evidence".to_string(), evidence.record.clone());
                    object.insert(
                        "failure_state_observation".to_string(),
                        evidence.state_observation.clone(),
                    );
                    object.insert(
                        "screen_observable".to_string(),
                        json!(evidence.screen_observable),
                    );
                    object.insert("screen_quiet".to_string(), json!(screen_quiet));
                    object.insert(
                        "wrong_destination_confirmed".to_string(),
                        json!(wrong_destination),
                    );
                    if !evidence.child_active || !evidence.screen_observable {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("active screen or live child could not be verified"),
                        );
                    } else if !screen_quiet {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("captured screen did not reach visual quiet"),
                        );
                    } else if expected_page_visible {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("locator wait timed out although the captured snapshot showed the expected page"),
                        );
                    }
                }
                semantic_assertions.push(assertion.clone());
                let evidence_path = evidence.record["directory"]
                    .as_str()
                    .unwrap_or("failure-evidence");
                if wrong_destination {
                    return Err(format!(
                        "initial-resource opened the visible System snapshot dialog instead of the System resources page; evidence: {evidence_path}"
                    )
                    .into());
                }
                return Err(format!(
                    "initial-resource did not show the System resources page before timeout; wrong destination was not confirmed; evidence: {evidence_path}"
                )
                .into());
            }
            Err(capture_error) => {
                semantic_assertions.push(json!({
                    "id":"initial-resource",
                    "status":"unverified",
                    "expected_visible":["System resources","12.4 of 16 GB","swap 1.1 GB","rust-analyzer","4812"],
                    "locator_wait_error":wait_error.to_string(),
                    "visual_quiet_error":quiet_error,
                    "failure_evidence_error":capture_error.to_string(),
                    "wrong_destination_confirmed":false,
                }));
                return Err(format!(
                    "initial-resource wait failed and active-screen evidence could not be captured: {capture_error}"
                )
                .into());
            }
        }
    }

    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[1],
        case_out,
    )?);
    let initial_resource_assertion =
        holla_resource_page_assertion("initial-resource", last_checkpoint_snapshot(checkpoints)?);
    semantic_assertions.push(initial_resource_assertion.clone());
    if !assertion_passed(&initial_resource_assertion) {
        return Err(assertion_error(&initial_resource_assertion).into());
    }
    session
        .get_by_text("System snapshot")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    if !assertion_passed(&initial_assertion) {
        return Err(assertion_error(&initial_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("System resources")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("4812")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[2],
        case_out,
    )?);
    let home_restored_assertion = holla_home_assertion(
        "home-restored",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        false,
        false,
    );
    semantic_assertions.push(home_restored_assertion.clone());
    if !assertion_passed(&home_restored_assertion) {
        return Err(assertion_error(&home_restored_assertion).into());
    }

    for key in ["r", "e", "s", "o", "u", "r", "c", "e", "s"] {
        actions.push(json!({"event":"key_press","key":key,"at_unix_ms":unix_ms()}));
        send_key(session, key)?;
    }
    session
        .get_by_text("resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("Show system resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[3],
        case_out,
    )?);
    let query_selected_assertion = holla_home_assertion(
        "query-selected",
        last_checkpoint_snapshot(checkpoints)?,
        "resources",
        false,
        true,
    );
    semantic_assertions.push(query_selected_assertion.clone());
    if !assertion_passed(&query_selected_assertion) {
        return Err(assertion_error(&query_selected_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Enter","at_unix_ms":unix_ms()}));
    send_key(session, "Enter")?;
    if let Err(wait_error) = session
        .get_by_text("System resources")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))
    {
        let quiet_error = wait_visual_quiet(session)
            .err()
            .map(|error| error.to_string());
        match capture_failure_evidence(
            session,
            profile,
            case,
            case_out,
            &raw_path,
            "query-resource",
        ) {
            Ok(evidence) => {
                let mut assertion =
                    holla_resource_page_assertion("query-resource", &evidence.snapshot);
                let expected_page_visible = assertion_passed(&assertion);
                let screen_quiet = quiet_error.is_none();
                if let Some(object) = assertion.as_object_mut() {
                    object.insert(
                        "locator_wait_error".to_string(),
                        json!(wait_error.to_string()),
                    );
                    object.insert("visual_quiet_error".to_string(), json!(quiet_error));
                    object.insert("failure_evidence".to_string(), evidence.record.clone());
                    object.insert(
                        "failure_state_observation".to_string(),
                        evidence.state_observation.clone(),
                    );
                    object.insert(
                        "screen_observable".to_string(),
                        json!(evidence.screen_observable),
                    );
                    object.insert("screen_quiet".to_string(), json!(screen_quiet));
                    if !evidence.child_active || !evidence.screen_observable {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("active screen or live child could not be verified"),
                        );
                    } else if !screen_quiet {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("captured screen did not reach visual quiet"),
                        );
                    } else if expected_page_visible {
                        object.insert("status".to_string(), json!("unverified"));
                        object.insert(
                            "infrastructure_status".to_string(),
                            json!("locator wait timed out although the captured snapshot showed the expected page"),
                        );
                    }
                }
                semantic_assertions.push(assertion);
                return Err(
                    "query-resource did not show the System resources page before timeout; evidence: failure-evidence/query-resource"
                        .into(),
                );
            }
            Err(capture_error) => {
                semantic_assertions.push(json!({
                    "id":"query-resource",
                    "status":"unverified",
                    "expected_visible":["System resources","12.4 of 16 GB","swap 1.1 GB","rust-analyzer","4812"],
                    "locator_wait_error":wait_error.to_string(),
                    "visual_quiet_error":quiet_error,
                    "failure_evidence_error":capture_error.to_string(),
                    "screen_observable":false,
                }));
                return Err(format!(
                    "query-resource wait failed and active-screen evidence could not be captured: {capture_error}"
                )
                .into());
            }
        }
    }
    wait_visual_quiet(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[4],
        case_out,
    )?);
    let query_resource_assertion =
        holla_resource_page_assertion("query-resource", last_checkpoint_snapshot(checkpoints)?);
    semantic_assertions.push(query_resource_assertion.clone());
    if !assertion_passed(&query_resource_assertion) {
        return Err(assertion_error(&query_resource_assertion).into());
    }
    session
        .get_by_text("System snapshot")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("System resources")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text("4812")
        .wait_hidden(Some(HELP_TIMEOUT_MS))?;
    session
        .get_by_text(case.initial_marker)
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[5],
        case_out,
    )?);
    let query_restored_assertion = holla_home_assertion(
        "query-restored",
        last_checkpoint_snapshot(checkpoints)?,
        "resources",
        false,
        false,
    );
    semantic_assertions.push(query_restored_assertion.clone());
    if !assertion_passed(&query_restored_assertion) {
        return Err(assertion_error(&query_restored_assertion).into());
    }

    actions.push(json!({"event":"key_press","key":"Escape","at_unix_ms":unix_ms()}));
    send_key(session, "Escape")?;
    session
        .get_by_text("Search actions and resources…")
        .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
    wait_visual_quiet(session)?;
    ensure_child_active(session)?;
    checkpoints.push(capture_checkpoint(
        session,
        profile,
        case,
        case.checkpoint_labels[6],
        case_out,
    )?);
    let query_cleared_assertion = holla_home_assertion(
        "query-cleared",
        last_checkpoint_snapshot(checkpoints)?,
        "empty",
        false,
        false,
    );
    semantic_assertions.push(query_cleared_assertion.clone());
    if !assertion_passed(&query_cleared_assertion) {
        return Err(assertion_error(&query_cleared_assertion).into());
    }

    Ok(())
}

fn last_checkpoint_snapshot(checkpoints: &[Checkpoint]) -> AppResult<&TerminalSnapshot> {
    checkpoints
        .last()
        .map(|checkpoint| &checkpoint.snapshot)
        .ok_or_else(|| "no captured checkpoint is available for semantic assertions".into())
}

fn snapshot_text(snapshot: &TerminalSnapshot) -> String {
    (0..snapshot.rows())
        .filter_map(|row_index| snapshot.row(row_index))
        .map(|row| {
            row.iter()
                .map(|cell| {
                    if cell.has(Attrs::INVISIBLE) {
                        ""
                    } else {
                        cell.text()
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn holla_result_list_text(snapshot: &TerminalSnapshot) -> String {
    let x_start = HOLLA_RESULT_LIST_X_START;
    let x_end = HOLLA_RESULT_LIST_X_END.min(snapshot.cols());
    let y_start = HOLLA_RESULT_LIST_Y_START.min(snapshot.rows());
    let y_end = HOLLA_RESULT_LIST_Y_END.min(snapshot.rows());
    (y_start..y_end)
        .filter_map(|row_index| snapshot.row(row_index))
        .map(|row| {
            row.iter()
                .skip(x_start as usize)
                .take(x_end.saturating_sub(x_start) as usize)
                .map(|cell| {
                    if cell.has(Attrs::INVISIBLE) {
                        ""
                    } else {
                        cell.text()
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn holla_home_assertion(
    id: &str,
    snapshot: &TerminalSnapshot,
    query_state: &str,
    initial: bool,
    filtered: bool,
) -> Value {
    let actual_text = snapshot_text(snapshot);
    let result_list_text = holla_result_list_text(snapshot);
    let mut required = vec!["› Show system resources", "holla snapshot system", "mbp"];
    if initial {
        required.extend(["CPU 78% for 4 min", "still discovering…", "no project"]);
    }
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|marker| !actual_text.contains(marker))
        .collect();
    let forbidden = ["System snapshot"];
    let forbidden_visible: Vec<&str> = forbidden
        .iter()
        .copied()
        .filter(|marker| actual_text.contains(marker))
        .collect();
    let query_line = actual_text.lines().nth(4).unwrap_or_default();
    let query_matches = match query_state {
        "empty" => query_line.contains("Search actions and resources…"),
        "resources" => {
            query_line.contains("resources")
                && !query_line.contains("Search actions and resources…")
        }
        _ => false,
    };
    let unexpected_rows: Vec<&str> = if filtered {
        ["Analyze disk usage", "Open btm"]
            .into_iter()
            .filter(|marker| result_list_text.contains(marker))
            .collect()
    } else {
        Vec::new()
    };
    let status = if missing.is_empty()
        && forbidden_visible.is_empty()
        && unexpected_rows.is_empty()
        && query_matches
    {
        "pass"
    } else {
        "fail"
    };
    json!({
        "id":id,
        "status":status,
        "required_visible":required,
        "missing_visible":missing,
        "forbidden_visible":forbidden_visible,
        "unexpected_default_rows":unexpected_rows,
        "observed_result_list_text":result_list_text,
        "query_line":query_line,
        "query_state":query_state,
        "query_matches":query_matches,
        "observed_visible_text":actual_text,
    })
}

fn holla_resource_page_assertion(id: &str, snapshot: &TerminalSnapshot) -> Value {
    let actual_text = snapshot_text(snapshot);
    let required = [
        "System resources",
        "mbp",
        "CPU",
        "Load",
        "Cores",
        "Memory",
        "Pressure",
        "12.4 of 16 GB",
        "swap 1.1 GB",
        "rust-analyzer",
        "4812",
    ];
    let missing: Vec<&str> = required
        .into_iter()
        .filter(|marker| !actual_text.contains(marker))
        .collect();
    let forbidden = ["System snapshot"];
    let forbidden_visible: Vec<&str> = forbidden
        .into_iter()
        .filter(|marker| actual_text.contains(marker))
        .collect();
    let status = if missing.is_empty() && forbidden_visible.is_empty() {
        "pass"
    } else {
        "fail"
    };
    json!({
        "id":id,
        "status":status,
        "required_visible":required,
        "missing_visible":missing,
        "forbidden_visible":forbidden_visible,
        "wrong_destination_visible":actual_text.contains("System snapshot")
            && !actual_text.contains("System resources"),
        "observed_visible_text":actual_text,
    })
}

fn assertion_passed(assertion: &Value) -> bool {
    assertion["status"] == "pass"
}

fn assertion_error(assertion: &Value) -> String {
    let missing = assertion["missing_visible"].to_string();
    let forbidden = assertion["forbidden_visible"].to_string();
    let unexpected = assertion["unexpected_default_rows"].to_string();
    format!(
        "{} semantic assertion failed: missing={missing}, forbidden={forbidden}, unexpected_rows={unexpected}",
        assertion["id"].as_str().unwrap_or("unknown")
    )
}

fn capture_failure_evidence(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    case_out: &Path,
    raw_path: &Path,
    stage: &str,
) -> AppResult<FailureScreenEvidence> {
    let (snapshot, offset) =
        session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
    let (state_observation, child_active) = failure_state_observation(session);
    let offset = offset.ok_or_else(|| {
        format!(
            "processed PTY byte offset is unavailable or overflowed; state: {state_observation}"
        )
    })?;
    if snapshot.cols() != u32::from(case.cols) || snapshot.rows() != u32::from(case.rows) {
        return Err(format!(
            "failure snapshot dimensions are {}x{}, expected {}x{}; state: {state_observation}",
            snapshot.cols(),
            snapshot.rows(),
            case.cols,
            case.rows
        )
        .into());
    }
    let actual_text = snapshot_text(&snapshot);
    let screen_observable = !actual_text.trim().is_empty();
    let directory = case_out.join("failure-evidence").join(stage);
    let sample_id = format!(
        "{}/failure-evidence/{stage}/pixels-{}",
        case.id,
        platform_key()
    );
    let sample = Sample::with_pixels(&sample_id, &snapshot, Observation::empty(), profile)?;
    let sample_bytes = sample.to_bytes()?;
    let image = sample
        .image()
        .ok_or("failure evidence sample has no PNG image")?;
    let png = base64::engine::general_purpose::STANDARD.decode(image.png_base64())?;
    let raw = fs::read(raw_path)?;
    let prefix_end = usize::try_from(offset)?;
    if prefix_end > raw.len() {
        return Err(format!(
            "failure snapshot offset {prefix_end} exceeds raw file length {}",
            raw.len()
        )
        .into());
    }
    fs::create_dir_all(&directory)?;
    write_new(&directory.join("sample.tui.json"), &sample_bytes)?;
    write_new(
        &directory.join("decoded-snapshot.json"),
        &serde_json::to_vec_pretty(&snapshot)?,
    )?;
    write_new(&directory.join("screen.png"), &png)?;
    write_new(&directory.join("visible-text.txt"), actual_text.as_bytes())?;
    write_new(&directory.join("raw-prefix.ansi"), &raw[..prefix_end])?;
    write_json_new(
        &directory.join("capture.json"),
        &json!({
            "stage":stage,
            "sample_id":sample_id,
            "processed_output_offset":offset,
            "state_observation":state_observation,
            "child_active":child_active,
            "screen_observable":screen_observable,
            "raw_prefix_sha256":sha256(&raw[..prefix_end]),
            "raw_stream_bytes_at_capture":raw.len(),
            "raw_stream_sha256_at_capture":sha256(&raw),
        }),
    )?;
    Ok(FailureScreenEvidence {
        record: json!({
            "directory":format!("failure-evidence/{stage}"),
            "sample_id":sample_id,
            "processed_output_offset":offset,
            "child_active":child_active,
            "screen_observable":screen_observable,
            "state_observation":state_observation,
            "sample_sha256":sha256(&sample_bytes),
            "png_sha256":image.png_sha256(),
            "rgba_sha256":image.rgba_sha256(),
            "raw_prefix_sha256":sha256(&raw[..prefix_end]),
        }),
        snapshot,
        state_observation,
        child_active,
        screen_observable,
    })
}

fn failure_state_observation(session: &Session) -> (Value, bool) {
    match session.execute(Operation::State) {
        Ok(OperationResult::State(state)) => {
            let child_active = state.exited.is_none() && state.exit_signal.is_none();
            (
                json!({
                    "status":"available",
                    "exited":state.exited,
                    "exit_signal":state.exit_signal,
                    "cols":state.cols,
                    "rows":state.rows,
                    "child_active":child_active,
                }),
                child_active,
            )
        }
        Ok(other) => (
            json!({
                "status":"unavailable",
                "error":format!("unexpected process state result: {other:?}"),
            }),
            false,
        ),
        Err(error) => (
            json!({"status":"unavailable","error":error.to_string()}),
            false,
        ),
    }
}

fn finalize_holla_after_error(session: &Session, actions: &mut Vec<Value>) -> (Value, Value, bool) {
    actions.push(json!({
        "event":"cleanup_key_press",
        "key":"Ctrl+q",
        "reason":"bounded_finalization_after_assertion_failure",
        "at_unix_ms":unix_ms(),
    }));
    let key_result = send_key(session, "Ctrl+q");
    let wait_result = session.execute(Operation::WaitExit {
        timeout_ms: Some(EXIT_TIMEOUT_MS),
    });
    let state_result = session.execute(Operation::State);
    let (state_value, process_state, child_exited, state_error) = match state_result {
        Ok(OperationResult::State(state)) => {
            let exited = state.exited.is_some() || state.exit_signal.is_some();
            let value = json!({
                "exited":state.exited,
                "exit_signal":state.exit_signal,
                "cols":state.cols,
                "rows":state.rows,
            });
            (value.clone(), value, exited, Value::Null)
        }
        Ok(other) => {
            let error = format!("unexpected process state result: {other:?}");
            (Value::Null, Value::Null, false, json!(error))
        }
        Err(error) => (Value::Null, Value::Null, false, json!(error.to_string())),
    };
    let (wait_exit_completed, wait_exit_error) = match wait_result {
        Ok(_) => (true, Value::Null),
        Err(error) => (false, json!(error.to_string())),
    };
    let clean_exit = wait_exit_completed
        && state_error.is_null()
        && process_state["exited"] == 0
        && process_state["exit_signal"].is_null();
    let finalization = json!({
        "attempted":true,
        "quit_key":"Ctrl+q",
        "quit_key_sent":key_result.is_ok(),
        "quit_key_error":key_result.err().map(|error| error.to_string()),
        "wait_exit_completed":wait_exit_completed,
        "wait_exit_error":wait_exit_error,
        "state":state_value,
        "state_error":state_error,
        "child_exited":child_exited,
        "clean_exit":clean_exit,
    });
    (finalization, process_state, child_exited)
}

fn preserve_reached_checkpoint_prefixes(
    raw_path: &Path,
    case_out: &Path,
    checkpoints: &[Checkpoint],
) -> Vec<String> {
    let raw = match fs::read(raw_path) {
        Ok(raw) => raw,
        Err(error) => return vec![format!("cannot read raw stream: {error}")],
    };
    let mut errors = Vec::new();
    for checkpoint in checkpoints {
        let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
        let prefix_path = checkpoint_dir.join("raw-prefix.ansi");
        let capture_path = checkpoint_dir.join("capture.json");
        if prefix_path.is_file() && capture_path.is_file() {
            continue;
        }
        let end = match usize::try_from(checkpoint.offset) {
            Ok(end) if end <= raw.len() => end,
            Ok(end) => {
                errors.push(format!(
                    "{} offset {end} exceeds final raw length {}",
                    checkpoint.checkpoint_id,
                    raw.len()
                ));
                continue;
            }
            Err(error) => {
                errors.push(format!(
                    "{} offset conversion failed: {error}",
                    checkpoint.checkpoint_id
                ));
                continue;
            }
        };
        if let Err(error) = fs::create_dir_all(&checkpoint_dir) {
            errors.push(format!(
                "{} output directory failed: {error}",
                checkpoint.checkpoint_id
            ));
            continue;
        }
        if !prefix_path.exists()
            && let Err(error) = write_new(&prefix_path, &raw[..end])
        {
            errors.push(format!(
                "{} raw prefix failed: {error}",
                checkpoint.checkpoint_id
            ));
            continue;
        }
        if !capture_path.exists()
            && let Err(error) = write_json_new(
                &capture_path,
                &json!({
                    "checkpoint_id":checkpoint.checkpoint_id,
                    "sample_id":checkpoint.sample_id,
                    "processed_output_offset":checkpoint.offset,
                    "raw_prefix_sha256":sha256(&raw[..end]),
                    "raw_stream_sha256":sha256(&raw),
                    "raw_stream_bytes":raw.len(),
                }),
            )
        {
            errors.push(format!(
                "{} capture record failed: {error}",
                checkpoint.checkpoint_id
            ));
        }
    }
    errors
}

fn capture_checkpoint(
    session: &Session,
    profile: &tuiscotti::render_profile::Profile,
    case: Case,
    label: &str,
    case_out: &Path,
) -> AppResult<Checkpoint> {
    let (snapshot, offset) =
        session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
    let offset = offset.ok_or("processed PTY byte offset is unavailable or overflowed")?;
    let captured_at_unix_ms = unix_ms();
    if snapshot.cols() != u32::from(case.cols) || snapshot.rows() != u32::from(case.rows) {
        return Err(format!(
            "snapshot dimensions are {}x{}, expected {}x{}",
            snapshot.cols(),
            snapshot.rows(),
            case.cols,
            case.rows
        )
        .into());
    }
    let checkpoint_id = format!("{}/{label}", case.id);
    let sample_suffix = match label {
        "opened" => "initial",
        "returned" => "restored",
        "exited" => "exit",
        other => other,
    };
    let sample_id = format!("{}/{sample_suffix}/pixels-{}", case.id, platform_key());
    let sample = Sample::with_pixels(&sample_id, &snapshot, Observation::empty(), profile)?;
    let sample_bytes = sample.to_bytes()?;
    let image = sample.image().ok_or("pixel sample has no PNG image")?;
    let png = base64::engine::general_purpose::STANDARD.decode(image.png_base64())?;
    let checkpoint_dir = case_out.join("checkpoints").join(label);
    fs::create_dir_all(&checkpoint_dir)?;
    write_new(&checkpoint_dir.join("sample.tui.json"), &sample_bytes)?;
    write_new(
        &checkpoint_dir.join("decoded-snapshot.json"),
        &serde_json::to_vec_pretty(&snapshot)?,
    )?;
    write_new(&checkpoint_dir.join("screen.png"), &png)?;
    Ok(Checkpoint {
        label: label.to_string(),
        checkpoint_id,
        sample_id,
        offset,
        captured_at_unix_ms,
        snapshot,
        sample_bytes,
        png_sha256: image.png_sha256().to_string(),
        rgba_sha256: image.rgba_sha256().to_string(),
    })
}

fn visible_text_changed(left: &TerminalSnapshot, right: &TerminalSnapshot) -> bool {
    for row in 0..left.rows().min(right.rows()) {
        for col in 0..left.cols().min(right.cols()) {
            let Some(left_cell) = left.cell_at(col, row) else {
                continue;
            };
            let Some(right_cell) = right.cell_at(col, row) else {
                continue;
            };
            if left_cell.text() != right_cell.text() {
                return true;
            }
        }
    }
    false
}

fn assert_same_checkpoint_state(
    left: &Checkpoint,
    right: &Checkpoint,
    message: &str,
) -> AppResult<()> {
    if let Some(difference) = first_difference(&left.snapshot, &right.snapshot) {
        return Err(format!("{message}: {difference}").into());
    }
    let left_sample = Sample::from_bytes(&left.sample_bytes)?;
    let right_sample = Sample::from_bytes(&right.sample_bytes)?;
    if left_sample.render_contract() != right_sample.render_contract() {
        return Err(format!("{message}: render contracts differ").into());
    }
    if left.png_sha256 != right.png_sha256 || left.rgba_sha256 != right.rgba_sha256 {
        return Err(format!("{message}: rendered pixels differ").into());
    }
    Ok(())
}

macro_rules! compare_cell_field {
    ($prefix:expr, $field:expr, $left:expr, $right:expr) => {
        if $left != $right {
            return Some(format!(
                "{}.{}: left={:?}, right={:?}",
                $prefix, $field, $left, $right
            ));
        }
    };
}

fn first_difference(left: &TerminalSnapshot, right: &TerminalSnapshot) -> Option<String> {
    macro_rules! compare_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Some(format!("{}: left={:?}, right={:?}", $field, $left, $right));
            }
        };
    }
    compare_field!("cols", left.cols(), right.cols());
    compare_field!("rows", left.rows(), right.rows());
    compare_field!("scope", left.scope(), right.scope());
    compare_field!(
        "viewport_origin",
        left.viewport_origin(),
        right.viewport_origin()
    );
    compare_field!("palette", left.palette(), right.palette());
    compare_field!(
        "default_foreground",
        left.default_foreground(),
        right.default_foreground()
    );
    compare_field!(
        "default_background",
        left.default_background(),
        right.default_background()
    );
    compare_field!(
        "default_cursor",
        left.default_cursor(),
        right.default_cursor()
    );
    compare_field!("title", left.title(), right.title());
    let a = left.cursor();
    let b = right.cursor();
    compare_field!("cursor.col", a.col(), b.col());
    compare_field!("cursor.row", a.row(), b.row());
    compare_field!("cursor.in_capture", a.in_capture(), b.in_capture());
    compare_field!("cursor.visible", a.visible(), b.visible());
    compare_field!("cursor.shape", a.shape(), b.shape());
    compare_field!("cursor.color", a.color(), b.color());
    compare_field!("cursor.blink", a.blink(), b.blink());
    for row in 0..left.rows().min(right.rows()) {
        for col in 0..left.cols().min(right.cols()) {
            let Some(a) = left.cell_at(col, row) else {
                continue;
            };
            let Some(b) = right.cell_at(col, row) else {
                continue;
            };
            let prefix = format!("cell[{col},{row}]");
            if a.text() != b.text() {
                return Some(format!(
                    "{prefix}.text: left={:?}, right={:?}",
                    a.text(),
                    b.text()
                ));
            }
            compare_cell_field!(&prefix, "width", a.width(), b.width());
            compare_cell_field!(&prefix, "fg", a.fg(), b.fg());
            compare_cell_field!(&prefix, "bg", a.bg(), b.bg());
            for (name, attr) in [
                ("bold", Attrs::BOLD),
                ("dim", Attrs::DIM),
                ("italic", Attrs::ITALIC),
                ("inverse", Attrs::INVERSE),
                ("invisible", Attrs::INVISIBLE),
                ("strike", Attrs::STRIKE),
                ("blink", Attrs::BLINK),
            ] {
                compare_cell_field!(&prefix, name, a.has(attr), b.has(attr));
            }
            compare_cell_field!(&prefix, "underline", a.underline(), b.underline());
            compare_cell_field!(
                &prefix,
                "underline_color",
                a.underline_color(),
                b.underline_color()
            );
            compare_cell_field!(&prefix, "link_uri", a.link_uri(), b.link_uri());
            compare_cell_field!(&prefix, "link_id", a.link_id(), b.link_id());
        }
    }
    None
}

fn check_json(outcome: &CheckOutcome) -> Value {
    match outcome {
        CheckOutcome::Pass(success) => json!({
            "status":"pass",
            "test_id":success.test_id(),
            "candidate_sha256":success.candidate_sha256(),
            "baseline_sha256":success.baseline_sha256(),
        }),
        CheckOutcome::Failure(failure) => json!({
            "status":"failure",
            "test_id":failure.test_id(),
            "reason":failure.reason().as_str(),
            "candidate_sha256":failure.candidate_sha256(),
            "baseline_sha256":failure.baseline_sha256(),
            "candidate_path":failure.candidate_path(),
            "evidence_dir":failure.evidence_dir(),
            "report_path":failure.report_path(),
            "changed_fields":failure.changed_fields(),
            "changed_cell_total":failure.changed_cell_total(),
            "pixel_changed":failure.pixel_changed(),
            "pixel_compared":failure.pixel_compared(),
            "pixel_bounds":failure.pixel_bounds(),
        }),
    }
}

fn ensure_child_active(session: &Session) -> AppResult<()> {
    let state = match session.execute(Operation::State)? {
        OperationResult::State(state) => state,
        other => return Err(format!("unexpected state result: {other:?}").into()),
    };
    if state.exited.is_some() || state.exit_signal.is_some() {
        return Err(format!(
            "child exited before the initial checkpoint: exited={:?}, signal={:?}",
            state.exited, state.exit_signal
        )
        .into());
    }
    Ok(())
}

fn wait_visual_quiet(session: &Session) -> AppResult<()> {
    let options =
        WaitQuietOptions::new(QUIET_TIMEOUT, QUIET_INTERVAL)?.with_mode(WaitQuietMode::Visual);
    session.wait_quiet(&options)?;
    Ok(())
}

fn send_key(session: &Session, key: &str) -> AppResult<()> {
    session.execute(Operation::Key {
        keys: vec![key.to_string()],
        action: KeyAction::Press,
    })?;
    Ok(())
}

fn suite_root() -> AppResult<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest.canonicalize()?)
}

fn verify_export(bin_dir: &Path, run_out: &Path) -> AppResult<Value> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("export-manifest.json");
    let helper = root.join("export.py");
    let output = std::process::Command::new("python3")
        .arg("-I")
        .arg("-c")
        .arg(EXPORT_LOADER)
        .arg(&manifest)
        .arg(&helper)
        .arg(BUILD_EXPORT_SHA256)
        .arg("verify")
        .arg("--root")
        .arg(&root)
        .arg("--sha256")
        .arg(BUILD_EXPORT_SHA256)
        .arg("--bin-dir")
        .arg(bin_dir)
        .arg("--run-out")
        .arg(run_out)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "sealed export verification failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let provenance: Value = serde_json::from_slice(&output.stdout)?;
    if provenance["export_sha256"] != BUILD_EXPORT_SHA256 {
        return Err("export verifier returned a different seal".into());
    }
    Ok(provenance)
}

fn verify_application_root(source_manifest: &Value, export_provenance: &Value) -> AppResult<()> {
    let source_worktree = source_manifest["source_worktree"]
        .as_str()
        .ok_or("application source manifest has no source_worktree")?;
    let boundaries = &export_provenance["boundaries"];
    let reference = boundaries["reference_application_root"].as_str();
    let candidate = boundaries["candidate_application_root"].as_str();
    if reference != Some(source_worktree) && candidate != Some(source_worktree) {
        return Err("application source_worktree is not a sealed application root".into());
    }
    Ok(())
}

fn prepare_empty_output(path: &Path) -> AppResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            return Err(format!("output path already exists: {}", path.display()).into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path)?;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

fn write_json_new(path: &Path, value: &impl serde::Serialize) -> AppResult<()> {
    write_new(path, &serde_json::to_vec_pretty(value)?)
}

fn sha256_file(path: &Path) -> AppResult<String> {
    Ok(sha256(&fs::read(path)?))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut result = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut result, "{byte:02x}");
    }
    result
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
