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

mod raw_probes;
mod source_manifest;

const DEFAULT_COLS: u16 = 120;
const DEFAULT_ROWS: u16 = 40;
const FIRST_FLOW_DENOMINATOR: usize = 4;
const FIRST_CHECKPOINT_DENOMINATOR: usize = 12;
const FIRST_PLUS_HELP_FLOW_DENOMINATOR: usize = 5;
const FIRST_PLUS_HELP_CHECKPOINT_DENOMINATOR: usize = 18;
const HOLLA_FIRST_USE_FLOW_DENOMINATOR: usize = 1;
const HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR: usize = 7;
const HOLLA_RESULT_LIST_X_START: u32 = 0;
const HOLLA_RESULT_LIST_X_END: u32 = 78;
const HOLLA_RESULT_LIST_Y_START: u32 = 6;
const HOLLA_RESULT_LIST_Y_END: u32 = 38;
const HELP_TIMEOUT_MS: u64 = 15_000;
const EXIT_TIMEOUT_MS: u64 = 30_000;
const QUIET_TIMEOUT: Duration = Duration::from_secs(15);
const QUIET_INTERVAL: Duration = Duration::from_millis(400);
const ENGINE_COMMIT: &str = "438563a86a3c5d78e6da6e12511fa089c6f15c61";
const TUISCOTTI_COMMIT: &str = "f40513120727f83c0c9713ae8d04eac70dfc955e";
const ENGINE_PATH: &str = "/Users/donbeave/Downloads/tui-test-raw-capture-r7-20261027";
const FIRST_CHECKPOINTS: [&str; 3] = ["initial", "help", "restored"];
const SHOWCASE_HELP_CHECKPOINTS: [&str; 6] = [
    "initial",
    "help-start",
    "help-scrolled",
    "restored",
    "help-reopened",
    "restored-after-modal-q",
];
const HOLLA_FIRST_USE_CHECKPOINTS: [&str; HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR] = [
    "initial",
    "initial-resource",
    "home-restored",
    "query-selected",
    "query-resource",
    "query-restored",
    "query-cleared",
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
        args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
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

type AppResult<T> = Result<T, Box<dyn Error>>;

fn main() -> std::process::ExitCode {
    if std::env::args_os()
        .nth(1)
        .is_some_and(|argument| argument == "--raw-probe-child")
    {
        return match raw_probes::run_child() {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("raw-probe-child: {error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("termrock-live-suite: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let mut args = std::env::args_os().skip(1);
    let Some(first) = args.next() else {
        return Err(usage_error().into());
    };
    if first == "compare-runs" {
        return compare_command(args.collect());
    }
    if first == "probe-raw-capture" {
        verify_engine_identity()?;
        return raw_probes::run(args.collect());
    }
    if first == "write-source-manifest" {
        return source_manifest::write(args.collect());
    }
    verify_engine_identity()?;

    let mut bin_dir = None;
    let mut out_dir = None;
    let mut group = None;
    let mut pending = Some(first);
    loop {
        let key = if let Some(key) = pending.take() {
            key
        } else if let Some(key) = args.next() {
            key
        } else {
            break;
        };
        let Some(value) = args.next() else {
            return Err(format!("missing value after {}", key.to_string_lossy()).into());
        };
        match key.to_string_lossy().as_ref() {
            "--bin-dir" => bin_dir = Some(PathBuf::from(value)),
            "--out" => out_dir = Some(PathBuf::from(value)),
            "--group" => group = Some(value.to_string_lossy().into_owned()),
            other => return Err(format!("unknown argument {other:?}").into()),
        }
    }

    let bin_dir = bin_dir.ok_or("--bin-dir is required")?;
    let out_dir = out_dir.ok_or("--out is required")?;
    let group = group.ok_or("--group is required")?;
    let (cases, flow_denominator, checkpoint_denominator) = cases_for_group(&group)?;

    let mode = std::env::var("TERMROCK_LIVE_MODE")
        .map_err(|_| "TERMROCK_LIVE_MODE is required; set capture or check explicitly")?;
    if mode != "capture" && mode != "check" {
        return Err(format!("TERMROCK_LIVE_MODE must be capture or check, got {mode:?}").into());
    }
    let root = suite_root()?;
    let source_manifest = source_manifest::load_verified(&bin_dir)?;
    prepare_empty_output(&out_dir)?;

    let frozen_store = if mode == "check" {
        Some(FrozenStore::new(
            &root.join("expected"),
            &out_dir.join("frozen-checks"),
        )?)
    } else {
        None
    };
    let start = Instant::now();
    let started_at = unix_ms();
    let mut flow_results = Vec::with_capacity(flow_denominator);
    let mut completed_flows = 0_usize;
    let mut passed_flows = 0_usize;
    let mut completed_checkpoints = 0_usize;
    let mut passed_checkpoints = 0_usize;
    let mut failed_flows = 0_usize;
    for case in &cases {
        let result = run_case(
            *case,
            &bin_dir.join(case.binary),
            &root,
            &out_dir,
            frozen_store.as_ref(),
        )?;
        if result["process_status"] == "complete" {
            completed_flows += 1;
        } else {
            failed_flows += 1;
        }
        if result["status"] == "passed" {
            passed_flows += 1;
        }
        completed_checkpoints +=
            result["checkpoints_completed"].as_u64().unwrap_or_default() as usize;
        passed_checkpoints += result["checkpoints_passed"].as_u64().unwrap_or_default() as usize;
        flow_results.push(result);
    }
    let elapsed_ms = start.elapsed().as_millis() as u64;
    let run_status = if failed_flows > 0
        || completed_flows != flow_denominator
        || completed_checkpoints != checkpoint_denominator
    {
        "failed"
    } else if mode == "capture" {
        "captured-unapproved"
    } else if passed_flows == flow_denominator && passed_checkpoints == checkpoint_denominator {
        "passed"
    } else {
        "failed"
    };
    let process_status = if failed_flows == 0
        && completed_flows == flow_denominator
        && completed_checkpoints == checkpoint_denominator
    {
        "complete"
    } else {
        "incomplete"
    };
    let result = json!({
        "schema": "termrock-live-run.v1",
        "status": run_status,
        "process_status": process_status,
        "mode": mode,
        "group": group,
        "started_at_unix_ms": started_at,
        "finished_at_unix_ms": unix_ms(),
        "live_process_elapsed_ms": elapsed_ms,
        "flow_denominator": flow_denominator,
        "flows_completed": completed_flows,
        "flow_percent": percent(completed_flows, flow_denominator),
        "flows_passed": passed_flows,
        "flow_pass_percent": percent(passed_flows, flow_denominator),
        "checkpoint_denominator": checkpoint_denominator,
        "checkpoints_completed": completed_checkpoints,
        "checkpoint_percent": percent(completed_checkpoints, checkpoint_denominator),
        "checkpoints_passed": passed_checkpoints,
        "checkpoint_pass_percent": percent(passed_checkpoints, checkpoint_denominator),
        "binary_source_manifest": source_manifest,
        "suite_dependency_sources": {
            "tuiscotti": TUISCOTTI_COMMIT,
            "tui_test_derived": ENGINE_COMMIT,
        },
        "flows": flow_results,
        "limits": [
            "Full parity inventory is not run.",
            "The existing 302-screen claim is not fresh evidence.",
            "Reference approval is independent and outside this runner.",
        ],
    });
    write_json_new(&out_dir.join("run-result.json"), &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    if run_status != "passed" {
        let reason = if run_status == "captured-unapproved" {
            "capture-only output is unapproved; frozen check mode is required for acceptance"
        } else {
            "run incomplete or one or more frozen checks failed"
        };
        return Err(format!("{reason}; see run-result.json").into());
    }
    Ok(())
}

fn cases_for_group(group: &str) -> AppResult<(Vec<Case>, usize, usize)> {
    let (cases, flow_denominator, checkpoint_denominator) = match group {
        "first" => (
            CASES.to_vec(),
            FIRST_FLOW_DENOMINATOR,
            FIRST_CHECKPOINT_DENOMINATOR,
        ),
        "first-plus-help" => (
            CASES
                .iter()
                .copied()
                .chain(std::iter::once(SHOWCASE_HELP_CASE))
                .collect(),
            FIRST_PLUS_HELP_FLOW_DENOMINATOR,
            FIRST_PLUS_HELP_CHECKPOINT_DENOMINATOR,
        ),
        "holla-first-use-action" => (
            vec![HOLLA_FIRST_USE_ACTION_CASE],
            HOLLA_FIRST_USE_FLOW_DENOMINATOR,
            HOLLA_FIRST_USE_CHECKPOINT_DENOMINATOR,
        ),
        "all" => {
            return Err(
                "--group all is incomplete: the full parity inventory is not implemented".into(),
            );
        }
        other => {
            return Err(format!(
                "unsupported group {other:?}; expected first, first-plus-help, holla-first-use-action, or all"
            )
            .into());
        }
    };
    let actual_checkpoints: usize = cases.iter().map(|case| case.checkpoint_labels.len()).sum();
    if cases.is_empty()
        || cases.len() != flow_denominator
        || actual_checkpoints != checkpoint_denominator
    {
        return Err(format!(
            "group {group:?} selection does not match its frozen flow and checkpoint counts"
        )
        .into());
    }
    Ok((cases, flow_denominator, checkpoint_denominator))
}

fn run_case(
    case: Case,
    binary: &Path,
    suite: &Path,
    out_root: &Path,
    frozen_store: Option<&FrozenStore>,
) -> AppResult<Value> {
    let case_out = out_root.join(case.id);
    fs::create_dir_all(&case_out)?;
    let case_start = Instant::now();
    let started_at = unix_ms();
    let raw_path = case_out.join("raw.ansi");
    let cwd = suite.join("data").join(case.id);
    fs::create_dir_all(&cwd)?;
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
                "exited": state.exited,
                "exit_signal": state.exit_signal,
                "cols": state.cols,
                "rows": state.rows,
            }),
            other => return Err(format!("unexpected process state result: {other:?}").into()),
        };
        child_exited = true;
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
        let raw = fs::read(&raw_path)?;
        if u64::try_from(raw.len())? != captured {
            return Err(format!(
                "raw file length {} does not match reported byte count {captured}",
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

        if let Some(store) = frozen_store {
            for checkpoint in &checkpoints {
                let outcome = store.check(&checkpoint.sample_bytes)?;
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
        } else {
            for checkpoint in &checkpoints {
                let checkpoint_dir = case_out.join("checkpoints").join(&checkpoint.label);
                write_json_new(
                    &checkpoint_dir.join("check.json"),
                    &json!({"status":"capture-only","baseline_checked":false}),
                )?;
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
                "observed_after_cleanup_close".to_string(),
                json!(closed_for_cleanup),
            );
            object.insert("cleanup_error".to_string(), json!(cleanup_error.clone()));
        }
        write_json_new(&case_out.join("raw-status.json"), &status_value)?;
    }
    let checkpoint_evidence_errors =
        if work.is_err() && matches!(case.journey, Journey::HollaFirstUseAction) {
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
    let (check_failures, checkpoints_passed) =
        if frozen_store.is_some() && process_status == "complete" {
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
    } else if frozen_store.is_some() {
        "passed"
    } else {
        "captured-unapproved"
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
        object.insert(
            "checkpoint_evidence_errors".to_string(),
            json!(checkpoint_evidence_errors),
        );
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

// The PR26 120x40 FirstUse snapshot places the Finder query at row 4.
// Finder renders the master/detail body two rows below the query. Its 34%
// detail share gives 40 columns at this width; with the two-column gap, the
// master result list occupies x=0..77 and y=6..37. Keep row-absence checks in
// this pane so a valid preview alternative is not treated as an unfiltered row.
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
    let sample_id = format!("{checkpoint_id}/pixels-{}", platform_key());
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

fn compare_command(args: Vec<std::ffi::OsString>) -> AppResult<()> {
    let mut left = None;
    let mut right = None;
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let key = args[index].to_string_lossy();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {key}"))?;
        match key.as_ref() {
            "--left" => left = Some(PathBuf::from(value)),
            "--right" => right = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            other => return Err(format!("unknown compare-runs argument {other:?}").into()),
        }
        index += 2;
    }
    let left = left.ok_or("compare-runs requires --left")?;
    let right = right.ok_or("compare-runs requires --right")?;
    let out = out.ok_or("compare-runs requires --out")?;
    let left_run: Value = read_json(&left.join("run-result.json"))?;
    let right_run: Value = read_json(&right.join("run-result.json"))?;
    let group = left_run["group"]
        .as_str()
        .ok_or_else(|| format!("{} has no selected group", left.display()))?;
    if right_run["group"] != group {
        return Err(format!(
            "comparison groups differ: left={group:?}, right={:?}",
            right_run["group"]
        )
        .into());
    }
    let (cases, flow_denominator, checkpoint_denominator) = cases_for_group(group)?;
    verify_complete_capture_run(
        &left,
        group,
        &cases,
        flow_denominator,
        checkpoint_denominator,
    )?;
    verify_complete_capture_run(
        &right,
        group,
        &cases,
        flow_denominator,
        checkpoint_denominator,
    )?;
    let mut comparisons = Vec::with_capacity(checkpoint_denominator);
    let mut differing = 0_usize;
    for case in &cases {
        for label in case.checkpoint_labels {
            let relative = PathBuf::from(case.id).join("checkpoints").join(label);
            let left_bytes = fs::read(left.join(&relative).join("sample.tui.json"))?;
            let right_bytes = fs::read(right.join(&relative).join("sample.tui.json"))?;
            let left_sample = Sample::from_bytes(&left_bytes)?;
            let right_sample = Sample::from_bytes(&right_bytes)?;
            let first = first_difference(left_sample.snapshot(), right_sample.snapshot());
            let contracts_equal = left_sample.render_contract() == right_sample.render_contract();
            let left_image = left_sample.image().ok_or("left sample has no image")?;
            let right_image = right_sample.image().ok_or("right sample has no image")?;
            let pixels_equal = left_image.rgba_sha256() == right_image.rgba_sha256();
            let state_equal = first.is_none() && contracts_equal && pixels_equal;
            if !state_equal {
                differing += 1;
            }
            let left_capture: Value = read_json(&left.join(&relative).join("capture.json"))?;
            let right_capture: Value = read_json(&right.join(&relative).join("capture.json"))?;
            let checkpoint_id = format!("{}/{label}", case.id);
            let sample_id = format!("{checkpoint_id}/pixels-{}", platform_key());
            for capture in [&left_capture, &right_capture] {
                if capture["checkpoint_id"] != checkpoint_id || capture["sample_id"] != sample_id {
                    return Err(format!(
                        "capture identity mismatch for {checkpoint_id}: {:?}",
                        capture
                    )
                    .into());
                }
            }
            let left_prefix = raw_prefix(&left, case.id, &left_capture)?;
            let right_prefix = raw_prefix(&right, case.id, &right_capture)?;
            comparisons.push(json!({
                "id":checkpoint_id,
                "sample_id":sample_id,
                "status":if state_equal {"stable"} else {"different"},
                "first_difference":first,
                "render_contract_equal":contracts_equal,
                "rgba_equal":pixels_equal,
                "left_rgba_sha256":left_image.rgba_sha256(),
                "right_rgba_sha256":right_image.rgba_sha256(),
                "raw_prefix_equal":left_prefix == right_prefix,
                "left_raw_prefix_sha256":sha256(&left_prefix),
                "right_raw_prefix_sha256":sha256(&right_prefix),
                "raw_prefix_difference_is_informational":true,
            }));
        }
    }
    if comparisons.len() != checkpoint_denominator {
        return Err(format!("incomplete comparison: {} checkpoints", comparisons.len()).into());
    }
    let result = json!({
        "schema":"termrock-live-stability.v1",
        "status":if differing == 0 {"stable-unapproved"} else {"different"},
        "group":group,
        "flow_denominator":flow_denominator,
        "denominator":checkpoint_denominator,
        "compared":comparisons.len(),
        "stable":comparisons.len()-differing,
        "different":differing,
        "left":left,
        "right":right,
        "comparisons":comparisons,
    });
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_new(&out, &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    if differing > 0 {
        return Err("reference runs are not stable; see compare-runs result".into());
    }
    Err(
        "stable reference candidates remain unapproved; explicit independent approval is required"
            .into(),
    )
}

fn verify_complete_capture_run(
    path: &Path,
    group: &str,
    cases: &[Case],
    flow_denominator: usize,
    checkpoint_denominator: usize,
) -> AppResult<()> {
    let result: Value = read_json(&path.join("run-result.json"))?;
    if result["status"] != "captured-unapproved"
        || result["mode"] != "capture"
        || result["process_status"] != "complete"
        || result["group"] != group
    {
        return Err(format!("{} is not a complete capture-only run", path.display()).into());
    }
    if result["flow_denominator"] != flow_denominator as u64
        || result["flows_completed"] != flow_denominator as u64
        || result["checkpoint_denominator"] != checkpoint_denominator as u64
        || result["checkpoints_completed"] != checkpoint_denominator as u64
    {
        return Err(format!(
            "{} has an incomplete flow or checkpoint count",
            path.display()
        )
        .into());
    }
    let flows = result["flows"]
        .as_array()
        .ok_or_else(|| format!("{} has no flow records", path.display()))?;
    if flows.len() != flow_denominator {
        return Err(format!("{} has an incomplete flow record list", path.display()).into());
    }
    for case in cases {
        let case_out = path.join(case.id);
        let flow = flows
            .iter()
            .find(|flow| flow["id"] == case.id)
            .ok_or_else(|| format!("{} is missing flow {}", path.display(), case.id))?;
        if flow["status"] != "captured-unapproved"
            || flow["process_status"] != "complete"
            || flow["checkpoints_completed"] != case.checkpoint_labels.len() as u64
            || flow["checkpoint_records"].as_array().map(Vec::len)
                != Some(case.checkpoint_labels.len())
        {
            return Err(format!("{} has an incomplete flow {}", path.display(), case.id).into());
        }
        let raw_status: Value = read_json(&case_out.join("raw-status.json"))?;
        if raw_status["setup_complete"] != true
            || raw_status["complete"] != true
            || raw_status["drained"] != true
            || raw_status["flushed"] != true
            || !raw_status["errors"]
                .as_array()
                .is_some_and(|errors| errors.is_empty())
        {
            return Err(format!("{} has incomplete raw capture", case_out.display()).into());
        }
        let captured = raw_status["captured_output_bytes"]
            .as_u64()
            .ok_or_else(|| format!("{} has no captured byte count", case_out.display()))?;
        let processed = raw_status["processed_output_offset"]
            .as_u64()
            .ok_or_else(|| format!("{} has no processed byte offset", case_out.display()))?;
        if captured != processed || fs::metadata(case_out.join("raw.ansi"))?.len() != captured {
            return Err(format!("{} raw byte counts do not match", case_out.display()).into());
        }
    }
    Ok(())
}

fn raw_prefix(root: &Path, case_id: &str, capture: &Value) -> AppResult<Vec<u8>> {
    let raw = fs::read(root.join(case_id).join("raw.ansi"))?;
    let offset = capture["processed_output_offset"]
        .as_u64()
        .ok_or("capture record has no processed output offset")?;
    let end = usize::try_from(offset)?;
    if end > raw.len() {
        return Err(format!(
            "raw prefix offset {end} exceeds stream length {}",
            raw.len()
        )
        .into());
    }
    Ok(raw[..end].to_vec())
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

fn verify_engine_identity() -> AppResult<()> {
    let head = git_output(&["rev-parse", "HEAD"])?;
    if head != ENGINE_COMMIT {
        return Err(format!("raw capture engine is at {head}, expected {ENGINE_COMMIT}").into());
    }
    let status = git_output(&["status", "--porcelain", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err(format!("raw capture engine worktree is dirty: {status}").into());
    }
    Ok(())
}

fn git_output(args: &[&str]) -> AppResult<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(ENGINE_PATH)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {:?} failed for frozen engine: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn prepare_empty_output(path: &Path) -> AppResult<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(format!("output path is not a directory: {}", path.display()).into());
        }
        if fs::read_dir(path)?.next().is_some() {
            return Err(format!("output directory must be empty: {}", path.display()).into());
        }
    } else {
        fs::create_dir_all(path)?;
    }
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

fn read_json(path: &Path) -> AppResult<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
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

fn percent(value: usize, denominator: usize) -> f64 {
    (value as f64 * 100.0) / denominator as f64
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn usage_error() -> String {
    "usage: termrock-live-suite --bin-dir BIN --out OUT --group first | first-plus-help | holla-first-use-action | all\n       termrock-live-suite compare-runs --left A --right B --out RESULT.json\n       termrock-live-suite probe-raw-capture --out OUT\n       termrock-live-suite write-source-manifest --bin-dir BIN --source-dir REPO --source-commit SHA --source-tree SHA --build-log FILE --binary-sha256-file FILE".into()
}
