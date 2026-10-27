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

const COLS: u16 = 120;
const ROWS: u16 = 40;
const FLOW_DENOMINATOR: usize = 4;
const CHECKPOINT_DENOMINATOR: usize = 12;
const HELP_TIMEOUT_MS: u64 = 15_000;
const EXIT_TIMEOUT_MS: u64 = 30_000;
const QUIET_TIMEOUT: Duration = Duration::from_secs(15);
const QUIET_INTERVAL: Duration = Duration::from_millis(400);
const ENGINE_COMMIT: &str = "5994dbf57047fe41f7401aaaa9197d49b8dfc0ae";
const TUISCOTTI_COMMIT: &str = "9de60de9d2b9e883e5901dad07ecee508ac6dd76";
const PR26_COMMIT: &str = "5b3dbd7ae799f325efbb4f4cc11b2f5518dec948";
const PR38_COMMIT: &str = "573a595e97451646cdd910ec392274c6a425f23b";
const ENGINE_PATH: &str = "/Users/donbeave/Downloads/tui-test-raw-capture-20261027";
const CHECKPOINTS: [&str; 3] = ["initial", "help", "restored"];

#[derive(Clone, Copy)]
struct Case {
    id: &'static str,
    binary: &'static str,
    args: &'static [&'static str],
    initial_marker: &'static str,
    help_key: &'static str,
    help_marker: &'static str,
    quit_key: &'static str,
}

const CASES: [Case; FLOW_DENOMINATOR] = [
    Case {
        id: "first/showcase",
        binary: "showcase",
        args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
        initial_marker: "Overview",
        help_key: "?",
        help_marker: "Keyboard & mouse",
        quit_key: "q",
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
    },
    Case {
        id: "first/jackin",
        binary: "jackin",
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
        initial_marker: "jackin❯",
        help_key: "?",
        help_marker: "Keyboard shortcuts",
        quit_key: "q",
    },
    Case {
        id: "first/tablepro",
        binary: "tablepro",
        args: &["--motion", "paused", "--frame", "0", "--color", "truecolor"],
        initial_marker: "TablePro",
        help_key: "?",
        help_marker: "Keyboard shortcuts",
        quit_key: "q",
    },
];

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
    if group == "all" {
        return Err(
            "--group all is incomplete: the full parity inventory is not implemented".into(),
        );
    }
    if group != "first" {
        return Err(format!("unsupported group {group:?}; expected first or all").into());
    }

    let mode = std::env::var("TERMROCK_LIVE_MODE")
        .map_err(|_| "TERMROCK_LIVE_MODE is required; set capture or check explicitly")?;
    if mode != "capture" && mode != "check" {
        return Err(format!("TERMROCK_LIVE_MODE must be capture or check, got {mode:?}").into());
    }
    let root = suite_root()?;
    prepare_empty_output(&out_dir)?;
    for case in CASES {
        let binary = bin_dir.join(case.binary);
        if !binary.is_file() {
            return Err(format!("required staged binary is missing: {}", binary.display()).into());
        }
    }
    let binary_hashes = CASES
        .iter()
        .map(|case| {
            let path = bin_dir.join(case.binary);
            Ok((case.binary, sha256_file(&path)?))
        })
        .collect::<AppResult<Vec<_>>>()?;

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
    let mut flow_results = Vec::with_capacity(FLOW_DENOMINATOR);
    let mut completed_flows = 0_usize;
    let mut passed_flows = 0_usize;
    let mut completed_checkpoints = 0_usize;
    let mut passed_checkpoints = 0_usize;
    let mut failed_flows = 0_usize;
    for case in CASES {
        let result = run_case(
            case,
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
        || completed_flows != FLOW_DENOMINATOR
        || completed_checkpoints != CHECKPOINT_DENOMINATOR
    {
        "failed"
    } else if mode == "capture" {
        "captured-unapproved"
    } else if passed_flows == FLOW_DENOMINATOR && passed_checkpoints == CHECKPOINT_DENOMINATOR {
        "passed"
    } else {
        "failed"
    };
    let process_status = if failed_flows == 0
        && completed_flows == FLOW_DENOMINATOR
        && completed_checkpoints == CHECKPOINT_DENOMINATOR
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
        "flow_denominator": FLOW_DENOMINATOR,
        "flows_completed": completed_flows,
        "flow_percent": percent(completed_flows, FLOW_DENOMINATOR),
        "flows_passed": passed_flows,
        "flow_pass_percent": percent(passed_flows, FLOW_DENOMINATOR),
        "checkpoint_denominator": CHECKPOINT_DENOMINATOR,
        "checkpoints_completed": completed_checkpoints,
        "checkpoint_percent": percent(completed_checkpoints, CHECKPOINT_DENOMINATOR),
        "checkpoints_passed": passed_checkpoints,
        "checkpoint_pass_percent": percent(passed_checkpoints, CHECKPOINT_DENOMINATOR),
        "binary_hashes_sha256": binary_hashes,
        "source_pins": {
            "pr26": PR26_COMMIT,
            "pr38": PR38_COMMIT,
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
    let mut checkpoints = Vec::with_capacity(CHECKPOINTS.len());
    let mut actions = Vec::new();
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
            cols: COLS,
            rows: ROWS,
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
        let initial_text = match session.execute(Operation::Text { full: true })? {
            OperationResult::Text(text) => text,
            other => return Err(format!("unexpected text result: {other:?}").into()),
        };
        if initial_text.trim().is_empty() {
            return Err("initial visible marker appeared but screen text is empty".into());
        }
        checkpoints.push(capture_checkpoint(
            &session,
            &sample_profile,
            case,
            "initial",
            &case_out,
        )?);

        actions.push(json!({"event":"key_press","key":case.help_key,"at_unix_ms":unix_ms()}));
        send_key(&session, case.help_key)?;
        session
            .get_by_text(case.help_marker)
            .wait_with_timeout(Some(HELP_TIMEOUT_MS))?;
        wait_visual_quiet(&session)?;
        checkpoints.push(capture_checkpoint(
            &session,
            &sample_profile,
            case,
            "help",
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
        let restored = capture_checkpoint(&session, &sample_profile, case, "restored", &case_out)?;
        checkpoints.push(restored);

        actions.push(json!({"event":"key_press","key":case.quit_key,"at_unix_ms":unix_ms()}));
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
                    if baseline_path.is_file() {
                        if let Ok(bytes) = fs::read(baseline_path) {
                            if let Ok(baseline) = Sample::from_bytes(&bytes) {
                                if let Some(object) = check_value.as_object_mut() {
                                    object.insert(
                                        "first_difference".to_string(),
                                        json!(first_difference(
                                            &baseline.snapshot(),
                                            &checkpoint.snapshot
                                        )),
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
                        }
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
        closed_for_cleanup = true;
        if let Err(error) = session.close() {
            cleanup_error = Some(error.to_string());
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
    write_json_new(
        &case_out.join("actions.json"),
        &json!({
            "schema":"termrock-live-actions.v1",
            "started_at_unix_ms":started_at,
            "finished_at_unix_ms":unix_ms(),
            "actions":actions,
        }),
    )?;

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
    let result = json!({
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
        "expected_checkpoints":CHECKPOINTS,
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
    write_json_new(&case_out.join("result.json"), &result)?;
    Ok(result)
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
    if snapshot.cols() != u32::from(COLS) || snapshot.rows() != u32::from(ROWS) {
        return Err(format!(
            "snapshot dimensions are {}x{}, expected {COLS}x{ROWS}",
            snapshot.cols(),
            snapshot.rows()
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
    verify_complete_capture_run(&left)?;
    verify_complete_capture_run(&right)?;
    let mut comparisons = Vec::with_capacity(CHECKPOINT_DENOMINATOR);
    let mut differing = 0_usize;
    for case in CASES {
        for label in CHECKPOINTS {
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
    if comparisons.len() != CHECKPOINT_DENOMINATOR {
        return Err(format!("incomplete comparison: {} checkpoints", comparisons.len()).into());
    }
    let result = json!({
        "schema":"termrock-live-stability.v1",
        "status":if differing == 0 {"stable-unapproved"} else {"different"},
        "denominator":CHECKPOINT_DENOMINATOR,
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

fn verify_complete_capture_run(path: &Path) -> AppResult<()> {
    let result: Value = read_json(&path.join("run-result.json"))?;
    if result["status"] != "captured-unapproved"
        || result["mode"] != "capture"
        || result["process_status"] != "complete"
    {
        return Err(format!("{} is not a complete capture-only run", path.display()).into());
    }
    if result["flows_completed"] != FLOW_DENOMINATOR as u64
        || result["checkpoints_completed"] != CHECKPOINT_DENOMINATOR as u64
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
    if flows.len() != FLOW_DENOMINATOR {
        return Err(format!("{} has an incomplete flow record list", path.display()).into());
    }
    for case in CASES {
        let case_out = path.join(case.id);
        let flow = flows
            .iter()
            .find(|flow| flow["id"] == case.id)
            .ok_or_else(|| format!("{} is missing flow {}", path.display(), case.id))?;
        if flow["status"] != "captured-unapproved"
            || flow["process_status"] != "complete"
            || flow["checkpoints_completed"] != CHECKPOINTS.len() as u64
            || flow["checkpoint_records"].as_array().map(Vec::len) != Some(CHECKPOINTS.len())
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
        if captured != processed
            || u64::try_from(fs::metadata(case_out.join("raw.ansi"))?.len())? != captured
        {
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

fn sha256_file(path: &Path) -> AppResult<String> {
    Ok(sha256(&fs::read(path)?))
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
    "usage: termrock-live-suite --bin-dir BIN --out OUT --group first | all\n       termrock-live-suite compare-runs --left A --right B --out RESULT.json\n       termrock-live-suite probe-raw-capture --out OUT".into()
}
