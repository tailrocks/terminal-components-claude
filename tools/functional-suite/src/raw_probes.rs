use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::sha256;
use serde_json::{Value, json};
use tui_test::Backend;
use tui_test::api::{
    AutomaticRecording, Operation, OperationResult, RawOutputCaptureOptions, RunOptions, Timeouts,
};
use tui_test::launch::{LaunchEnvMode, LaunchOptions};
use tui_test::profile::{Profile, Rgb};
use tui_test::render::style::Style;
use tui_test::runtime::Session;
use tui_test::snapshot::{SnapshotCursorShape, SnapshotScope, TerminalSnapshot};
use tui_test::wait::{WaitQuietMode, WaitQuietOptions};

const COLS: u16 = 120;
const ROWS: u16 = 40;
const CHILD_WAIT: Duration = Duration::from_secs(12);
const CHILD_WAIT_MS: u64 = 12_000;
const EXPECTED_FLOW_DENOMINATOR: usize = 4;
const CURSOR_STAGE_A: &[u8] =
    b"\x1b[2J\x1b[H\x1b[?25h\x1b[5 q\x1b]12;#123456\x07CV3-BLINK-ON\x1b[3;7H";
const CURSOR_STAGE_B: &[u8] =
    b"\x1b[H\x1b[2KCV3-BLINK-OFF\x1b[6 q\x1b]12;#654321\x07\x1b[?25l\x1b[4;9H";

type AppResult<T> = Result<T, Box<dyn Error>>;

pub fn run(args: Vec<OsString>) -> AppResult<()> {
    let mut out = None;
    let mut index = 0;
    while index < args.len() {
        let key = args[index].to_string_lossy();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {key}"))?;
        match key.as_ref() {
            "--out" => out = Some(PathBuf::from(value)),
            other => return Err(format!("unknown raw-probe argument {other:?}").into()),
        }
        index += 2;
    }
    let out = out.ok_or("probe-raw-capture requires --out")?;
    prepare_empty_output(&out)?;
    let out = out.canonicalize()?;
    let started_at = unix_ms();
    let mut results = Vec::new();
    let mut failed = 0_usize;
    let mut not_run = 0_usize;

    for (name, probe) in [
        (
            "exact-binary-atomic-prefix",
            run_exact_binary_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "forced-incomplete-drain",
            run_forced_drain_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "setup-and-lifecycle-rejection",
            run_setup_lifecycle_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "runtime-write-failure",
            run_runtime_write_failure_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "CV3-CURSOR",
            run_cursor_blink_probe as fn(&Path) -> AppResult<Value>,
        ),
    ] {
        let probe_dir = out.join(name);
        fs::create_dir_all(&probe_dir)?;
        let probe_started_at = unix_ms();
        let result = match probe(&probe_dir) {
            Ok(mut value) => {
                if let Some(object) = value.as_object_mut()
                    && !object.contains_key("status")
                {
                    object.insert("status".to_string(), json!("passed"));
                }
                value
            }
            Err(error) => {
                json!({"status":"failed", "error":error.to_string()})
            }
        };
        match result["status"].as_str() {
            Some("passed") => {}
            Some("not_run") => not_run += 1,
            _ => failed += 1,
        }
        let mut result = result;
        if let Some(object) = result.as_object_mut() {
            object.insert("probe".to_string(), json!(name));
            object.insert("started_at_unix_ms".to_string(), json!(probe_started_at));
            object.insert("finished_at_unix_ms".to_string(), json!(unix_ms()));
        }
        write_json_new(&probe_dir.join("result.json"), &result)?;
        results.push(result);
    }

    let run_status = if failed == 0 && not_run == 0 {
        "passed"
    } else {
        "failed"
    };
    let result = json!({
        "schema":"termrock-live-raw-probes.v1",
        "status":run_status,
        "started_at_unix_ms":started_at,
        "finished_at_unix_ms":unix_ms(),
        "probe_denominator":5,
        "probes_completed":5,
        "probes_failed":failed,
        "probes_not_run":not_run,
        "application_flows_affected":0,
        "application_flow_denominator":EXPECTED_FLOW_DENOMINATOR,
        "counts_toward_application_flow_denominator":false,
        "results":results,
        "limits":[
            "These probes exercise the raw-capture API with real child processes.",
            "The controlled PTY writes contain no carriage return or line feed, so PTY output newline translation does not alter the expected byte stream.",
            "CV3-CURSOR checks two fixed cursor states through the existing Alacritty producer path. It checks capture infrastructure, not an application journey.",
            "Overflow and File::flush failure are source-reviewed only; no deterministic runtime path was used.",
        ],
    });
    write_json_new(&out.join("run-result.json"), &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    if failed != 0 || not_run != 0 {
        return Err(format!(
            "raw-capture probes did not all pass: {failed} failed, {not_run} not run; see run-result.json"
        )
        .into());
    }
    Ok(())
}

pub fn run_child() -> AppResult<()> {
    let mode = std::env::args_os()
        .nth(2)
        .ok_or("raw probe child mode is required")?;
    match mode.to_string_lossy().as_ref() {
        "binary-stream" => child_binary_stream(),
        "lifecycle" => child_lifecycle(),
        "drain-normal" => child_drain_parent(true),
        "drain-explicit" => child_drain_parent(false),
        "drain-holder" => child_drain_holder(),
        "write-failure" => child_write_failure(),
        "cursor-blink" => child_cursor_blink(),
        other => Err(format!("unknown raw probe child mode {other:?}").into()),
    }
}

fn run_cursor_blink_probe(out: &Path) -> AppResult<Value> {
    let session = Session::new("raw-probe-cv3-cursor".to_string());
    let raw_path = out.join("raw.ansi");
    let expected = [CURSOR_STAGE_A, CURSOR_STAGE_B].concat();
    if expected.contains(&b'\r') || expected.contains(&b'\n') {
        return Err("CV3-CURSOR producer bytes contain a carriage return or line feed".into());
    }
    let mut child_state = None;
    let operation = (|| -> AppResult<Value> {
        let launch = child_launch(out, "cursor-blink", &[])?;
        session.spawn_with_raw_output_capture(
            launch,
            run_options(out),
            RawOutputCaptureOptions::new(&raw_path),
        )?;

        wait_file(&out.join("stage-a.ready"), CHILD_WAIT)?;
        session
            .get_by_text("CV3-BLINK-ON")
            .wait_with_timeout(Some(CHILD_WAIT_MS))?;
        wait_visual_quiet(&session)?;
        let (snapshot_a, offset_a) =
            session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
        let offset_a = offset_a.ok_or("stage A processed byte offset is unavailable")?;
        if CURSOR_STAGE_A.len() != 49 || offset_a != CURSOR_STAGE_A.len() as u64 {
            return Err(
                format!("stage A processed offset is {offset_a}, expected 49 bytes").into(),
            );
        }
        let raw_a = fs::read(&raw_path)?;
        if raw_a != CURSOR_STAGE_A {
            return Err(format!(
                "stage A raw prefix differs: got {} bytes, expected 49",
                raw_a.len()
            )
            .into());
        }
        let failures_a = verify_cursor_stage(
            &snapshot_a,
            "CV3-BLINK-ON",
            6,
            2,
            true,
            Some(true),
            Rgb::new(18, 52, 86),
        );
        write_cursor_stage(out, "stage-a", &snapshot_a, &raw_a, offset_a, &failures_a)?;

        fs::write(out.join("continue-b"), b"")?;
        wait_file(&out.join("stage-b.ready"), CHILD_WAIT)?;
        session
            .get_by_text("CV3-BLINK-OFF")
            .wait_with_timeout(Some(CHILD_WAIT_MS))?;
        wait_visual_quiet(&session)?;
        let (snapshot_b, offset_b) =
            session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
        let offset_b = offset_b.ok_or("stage B processed byte offset is unavailable")?;
        if CURSOR_STAGE_B.len() != 50 || offset_b != expected.len() as u64 {
            return Err(
                format!("stage B processed offset is {offset_b}, expected 99 bytes").into(),
            );
        }
        let raw_b = fs::read(&raw_path)?;
        if raw_b != expected {
            return Err(format!(
                "stage B raw prefix differs: got {} bytes, expected 99",
                raw_b.len()
            )
            .into());
        }
        let failures_b = verify_cursor_stage(
            &snapshot_b,
            "CV3-BLINK-OFF",
            8,
            3,
            false,
            Some(false),
            Rgb::new(101, 67, 33),
        );
        write_cursor_stage(out, "stage-b", &snapshot_b, &raw_b, offset_b, &failures_b)?;

        fs::write(out.join("finish"), b"")?;
        wait_for_exit(&session)?;
        let state = read_state(&session)?;
        require_clean_exit(&state)?;
        child_state = Some(state.clone());
        let status = session
            .raw_output_capture_status()
            .ok_or("cursor probe raw capture status is unavailable after child exit")?;
        verify_complete_status(&status, expected.len() as u64)?;
        let raw = fs::read(&raw_path)?;
        if raw != expected {
            return Err(format!(
                "final cursor raw stream differs: got {} bytes, expected 99",
                raw.len()
            )
            .into());
        }
        if offset_a != CURSOR_STAGE_A.len() as u64
            || offset_b != expected.len() as u64
            || status.processed_output_offset != Some(expected.len() as u64)
            || status.captured_output_bytes != Some(expected.len() as u64)
        {
            return Err(format!(
                "cursor offsets disagree: A={offset_a}, B={offset_b}, status={status:?}"
            )
            .into());
        }
        let assertion_failures = failures_a
            .iter()
            .chain(failures_b.iter())
            .cloned()
            .collect::<Vec<_>>();
        Ok(json!({
            "status":if assertion_failures.is_empty() { "passed" } else { "failed" },
            "id":"CV3-CURSOR",
            "application_flows_affected":0,
            "counts_toward_application_flow_denominator":false,
            "producer":"existing common-suite raw probe child",
            "expected_bytes":expected.len(),
            "expected_offsets":{"stage_a":49,"stage_b":99},
            "observed_offsets":{"stage_a":offset_a,"stage_b":offset_b},
            "raw_sha256":sha256(&raw),
            "raw_path":"raw.ansi",
            "stage_a":{
                "marker":"CV3-BLINK-ON",
                "cursor":{"col":6,"row":2,"visible":true,"shape":"bar","color":[18,52,86],"blink":true},
                "observed_cursor":{"full_grid":{"col":snapshot_a.cursor().col(),"row":snapshot_a.cursor().row()},"viewport_origin":snapshot_a.viewport_origin(),"cursor_local":snapshot_a.cursor_local()},
                "assertion_failures":failures_a,
            },
            "stage_b":{
                "marker":"CV3-BLINK-OFF",
                "cursor":{"col":8,"row":3,"visible":false,"shape":"bar","color":[101,67,33],"blink":false},
                "observed_cursor":{"full_grid":{"col":snapshot_b.cursor().col(),"row":snapshot_b.cursor().row()},"viewport_origin":snapshot_b.viewport_origin(),"cursor_local":snapshot_b.cursor_local()},
                "assertion_failures":failures_b,
            },
            "assertion_failures":assertion_failures,
            "child":state,
            "capture_status":status,
            "limits":["Infrastructure conformance only; not application behavior evidence."],
        }))
    })();

    if operation.is_err() {
        let _ = fs::write(out.join("continue-b"), b"");
        let _ = fs::write(out.join("finish"), b"");
        let _ = session.execute(Operation::WaitExit {
            timeout_ms: Some(1_000),
        });
    }
    let close_result = session.close();
    let final_status = session.raw_output_capture_status();
    if let Some(status) = &final_status {
        write_json_new(&out.join("raw-status-final.json"), status)?;
    }
    match operation {
        Ok(mut value) => {
            if let Err(error) = close_result {
                return Err(format!("cursor probe session close failed: {error}").into());
            }
            let status = final_status.ok_or("final cursor capture status is unavailable")?;
            verify_complete_status(&status, expected.len() as u64)?;
            if let Some(object) = value.as_object_mut() {
                object.insert("capture_status_after_close".to_string(), json!(status));
                object.insert(
                    "child_exit_observed_before_close".to_string(),
                    json!(child_state),
                );
            }
            Ok(value)
        }
        Err(error) => {
            let close_note = close_result.err().map(|close| close.to_string());
            if let Some(status) = final_status {
                Err(format!(
                    "{error}; final capture status after close: {status:?}; close error: {close_note:?}"
                )
                .into())
            } else {
                Err(format!(
                    "{error}; final capture status is unavailable; close error: {close_note:?}"
                )
                .into())
            }
        }
    }
}

fn verify_cursor_stage(
    snapshot: &TerminalSnapshot,
    marker: &str,
    col: u32,
    row: u32,
    visible: bool,
    blink: Option<bool>,
    color: Rgb,
) -> Vec<String> {
    let mut failures = Vec::new();
    if snapshot.cols() != u32::from(COLS) || snapshot.rows() != u32::from(ROWS) {
        failures.push(format!(
            "snapshot dimensions are {}x{}, expected {}x{}",
            snapshot.cols(),
            snapshot.rows(),
            COLS,
            ROWS
        ));
    }
    let first_row = snapshot
        .cells()
        .iter()
        .take(COLS as usize)
        .map(|cell| cell.text())
        .collect::<String>();
    if !first_row.contains(marker) {
        failures.push(format!("{marker} is not on row 0"));
    }
    let cursor = snapshot.cursor();
    let cursor_local = snapshot.cursor_local();
    if cursor_local != Some((col, row)) {
        failures.push(format!(
            "capture-local cursor position is {cursor_local:?}, expected Some(({col}, {row})); full-grid position is ({}, {}), viewport origin is {}",
            cursor.col(),
            cursor.row(),
            snapshot.viewport_origin()
        ));
    }
    if !cursor.in_capture() {
        failures.push("cursor is outside the active-screen capture".to_string());
    }
    if cursor.visible() != visible {
        failures.push(format!(
            "cursor visibility is {}, expected {visible}",
            cursor.visible()
        ));
    }
    if cursor.shape() != SnapshotCursorShape::Bar {
        failures.push(format!(
            "cursor shape is {:?}, expected bar",
            cursor.shape()
        ));
    }
    if cursor.color() != color {
        failures.push(format!(
            "cursor color is {:?}, expected {:?}",
            cursor.color(),
            color
        ));
    }
    if cursor.blink() != blink {
        failures.push(format!(
            "cursor blink is {:?}, expected {:?}",
            cursor.blink(),
            blink
        ));
    }
    failures
}

fn write_cursor_stage(
    out: &Path,
    label: &str,
    snapshot: &TerminalSnapshot,
    raw: &[u8],
    offset: u64,
    assertion_failures: &[String],
) -> AppResult<()> {
    let directory = out.join(label);
    fs::create_dir_all(&directory)?;
    write_new(&directory.join("raw-prefix.ansi"), raw)?;
    write_json_new(&directory.join("decoded-snapshot.json"), snapshot)?;
    write_json_new(
        &directory.join("capture.json"),
        &json!({
            "processed_output_offset":offset,
            "raw_prefix_bytes":raw.len(),
            "raw_prefix_sha256":sha256(raw),
            "assertion_failures":assertion_failures,
        }),
    )?;
    Ok(())
}

fn run_exact_binary_probe(out: &Path) -> AppResult<Value> {
    let session = Session::new("raw-probe-exact-binary".to_string());
    let raw_path = out.join("raw.ansi");
    let first = chunk_one();
    let burst = vec![b'B'; 10_000];
    let tail = b"\x1b[2J\x1b[HTAIL:\xe7\xb5\x82";
    let expected = [first.as_slice(), burst.as_slice(), tail.as_slice()].concat();
    let mut child_state = None;
    let operation = (|| -> AppResult<Value> {
        let launch = child_launch(out, "binary-stream", &[])?;
        session.spawn_with_raw_output_capture(
            launch,
            run_options(out),
            RawOutputCaptureOptions::new(&raw_path),
        )?;

        wait_file(&out.join("stage-one.ready"), CHILD_WAIT)?;
        session
            .get_by_text("chunk-one:界")
            .wait_with_timeout(Some(CHILD_WAIT_MS))?;
        wait_visual_quiet(&session)?;
        let (first_snapshot, first_offset) =
            session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
        let first_offset = first_offset.ok_or("first processed byte offset is unavailable")?;
        verify_decoded_marker(&first_snapshot, "chunk-one:界", "界")?;
        if first_offset != first.len() as u64 {
            return Err(format!(
                "first processed offset is {first_offset}, expected {}",
                first.len()
            )
            .into());
        }
        let first_raw = fs::read(&raw_path)?;
        if first_raw != first {
            return Err(format!(
                "first raw prefix differs: got {} bytes, expected {}",
                first_raw.len(),
                first.len()
            )
            .into());
        }
        write_new(&out.join("prefix-one.raw"), &first_raw)?;
        write_json_new(&out.join("snapshot-one.json"), &first_snapshot)?;

        fs::write(out.join("continue-one"), b"continue")?;
        wait_file(&out.join("stage-two.ready"), CHILD_WAIT)?;
        session
            .get_by_text("TAIL:終")
            .wait_with_timeout(Some(CHILD_WAIT_MS))?;
        wait_visual_quiet(&session)?;
        let (second_snapshot, second_offset) =
            session.capture_snapshot_with_raw_offset(SnapshotScope::ActiveScreen)?;
        let second_offset = second_offset.ok_or("second processed byte offset is unavailable")?;
        verify_decoded_marker(&second_snapshot, "TAIL:終", "終")?;
        if second_offset != expected.len() as u64 {
            return Err(format!(
                "second processed offset is {second_offset}, expected {}",
                expected.len()
            )
            .into());
        }
        let second_raw = fs::read(&raw_path)?;
        if second_raw != expected {
            return Err(format!(
                "raw stream differs: got {} bytes, expected {}",
                second_raw.len(),
                expected.len()
            )
            .into());
        }
        write_new(
            &out.join("prefix-two.raw"),
            &second_raw[..second_offset as usize],
        )?;
        write_json_new(&out.join("snapshot-two.json"), &second_snapshot)?;

        fs::write(out.join("finish-child"), b"finish")?;
        wait_for_exit(&session)?;
        let state = read_state(&session)?;
        require_clean_exit(&state)?;
        child_state = Some(state.clone());
        let status = session
            .raw_output_capture_status()
            .ok_or("raw capture status is unavailable after child exit")?;
        verify_complete_status(&status, expected.len() as u64)?;
        write_json_new(&out.join("raw-status-before-close.json"), &status)?;
        Ok(json!({
            "raw_bytes":expected.len(),
            "first_prefix_bytes":first.len(),
            "first_processed_offset":first_offset,
            "second_processed_offset":second_offset,
            "burst_bytes":burst.len(),
            "tail_bytes":tail.len(),
            "child":state,
            "capture_status":status,
            "pty_output_processing":"No CR or LF is present. The raw bytes are read from the PTY master before terminal decoding.",
            "decoded_markers":["chunk-one:界", "TAIL:終"],
        }))
    })();

    if operation.is_err() {
        let _ = fs::write(out.join("continue-one"), b"continue");
        let _ = fs::write(out.join("finish-child"), b"finish");
        let _ = session.execute(Operation::WaitExit {
            timeout_ms: Some(1_000),
        });
    }
    let close_result = session.close();
    let final_status = session.raw_output_capture_status();
    if let Some(status) = &final_status {
        write_json_new(&out.join("raw-status-final.json"), status)?;
    }
    match operation {
        Ok(mut value) => {
            if let Err(error) = close_result {
                return Err(format!("session close failed after clean child exit: {error}").into());
            }
            let status =
                final_status.ok_or("final raw capture status is unavailable after close")?;
            verify_complete_status(&status, expected.len() as u64)?;
            if let Some(object) = value.as_object_mut() {
                object.insert("capture_status".to_string(), json!(status));
                object.insert(
                    "child_exit_observed_before_close".to_string(),
                    json!(child_state),
                );
            }
            Ok(value)
        }
        Err(error) => {
            let close_note = close_result.err().map(|close| close.to_string());
            if let Some(status) = final_status {
                Err(format!(
                    "{error}; final capture status after close: {status:?}; close error: {close_note:?}"
                )
                .into())
            } else {
                Err(format!(
                    "{error}; final capture status is unavailable; close error: {close_note:?}"
                )
                .into())
            }
        }
    }
}

fn run_forced_drain_probe(out: &Path) -> AppResult<Value> {
    let normal_dir = out.join("normal-exit");
    let explicit_dir = out.join("explicit-close");
    fs::create_dir_all(&normal_dir)?;
    fs::create_dir_all(&explicit_dir)?;
    let normal = record_subprobe(
        &normal_dir.join("result.json"),
        run_one_forced_drain(&normal_dir, "drain-normal", true),
    )?;
    let explicit = record_subprobe(
        &explicit_dir.join("result.json"),
        run_one_forced_drain(&explicit_dir, "drain-explicit", false),
    )?;
    if normal["status"] != "passed" || explicit["status"] != "passed" {
        return Err(
            format!("forced-drain subprobe failed: normal={normal}, explicit={explicit}").into(),
        );
    }
    Ok(json!({"normal_exit":normal,"explicit_close":explicit}))
}

fn record_subprobe(path: &Path, result: AppResult<Value>) -> AppResult<Value> {
    let value = match result {
        Ok(value) => json!({"status":"passed", "result":value}),
        Err(error) => json!({"status":"failed", "error":error.to_string()}),
    };
    write_json_new(path, &value)?;
    Ok(value)
}

fn run_one_forced_drain(out: &Path, mode: &str, expect_clean_exit: bool) -> AppResult<Value> {
    let session = Session::new(format!("raw-probe-{mode}"));
    let raw_path = out.join("raw.ansi");
    let launch = child_launch(out, mode, &[])?;
    session.spawn_with_raw_output_capture(
        launch,
        run_options(out),
        RawOutputCaptureOptions::new(&raw_path),
    )?;
    let operation = (|| -> AppResult<(Value, u64)> {
        let marker = if expect_clean_exit {
            "DRAIN-NORMAL-READY"
        } else {
            "DRAIN-EXPLICIT-READY"
        };
        session
            .get_by_text(marker)
            .wait_with_timeout(Some(CHILD_WAIT_MS))?;
        let descendant_pid = read_pid(&out.join("descendant.pid"))?;
        if !pid_alive(descendant_pid) {
            return Err("PTY-holding descendant exited before the 250 ms interval".into());
        }
        let clean_child = if expect_clean_exit {
            wait_for_exit(&session)?;
            let state = read_state(&session)?;
            require_clean_exit(&state)?;
            wait_file(&out.join("drain-holder-after-parent-exit"), CHILD_WAIT)?;
            if !pid_alive(descendant_pid) {
                return Err("PTY-holding descendant exited with the normal child".into());
            }
            fs::write(out.join("drain-holder-write"), b"write")?;
            wait_for_holder_output(out, CHILD_WAIT)?;
            let initial_bytes = b"\x1b[2J\x1b[HDRAIN-NORMAL-READY".len() as u64;
            wait_capture_offset(
                &session,
                &raw_path,
                initial_bytes,
                b"DRAIN-HOLD",
                CHILD_WAIT,
            )?;
            Some(state)
        } else {
            std::thread::sleep(Duration::from_millis(350));
            if !pid_alive(descendant_pid) {
                return Err("PTY-holding descendant did not retain the slave past 250 ms".into());
            }
            let state = read_state(&session)?;
            if state["exited"].is_number() || !state["exit_signal"].is_null() {
                return Err(format!("explicit-close child exited before close: {state}").into());
            }
            None
        };
        if expect_clean_exit {
            let status = session
                .raw_output_capture_status()
                .ok_or("raw status is missing after normal child exit")?;
            if status.complete || status.errors.is_empty() {
                return Err(format!(
                    "normal exit did not report the incomplete PTY drain: {status:?}"
                )
                .into());
            }
            write_json_new(&out.join("raw-status-before-close.json"), &status)?;
        }
        let close_started = Instant::now();
        session.close()?;
        let close_elapsed_ms = close_started.elapsed().as_millis() as u64;
        if close_elapsed_ms > 3_000 {
            return Err(format!("close took {close_elapsed_ms} ms, over the 3000 ms bound").into());
        }
        let final_status = session
            .raw_output_capture_status()
            .ok_or("final raw status is unavailable after close")?;
        if final_status.complete || final_status.drained || final_status.errors.is_empty() {
            return Err(format!(
                "forced close did not report an incomplete drain: {final_status:?}"
            )
            .into());
        }
        write_json_new(&out.join("raw-status-final.json"), &final_status)?;
        fs::write(out.join("drain-holder-stop"), b"stop")?;
        let descendant_cleanup = wait_descendant_cleanup(out, descendant_pid)?;
        Ok((
            json!({
                "child_exit_observed":clean_child.is_some(),
                "clean_child_exit":clean_child,
                "close_elapsed_ms":close_elapsed_ms,
                "final_capture_status":final_status,
                "descendant_pid":descendant_pid,
                "descendant_cleanup":descendant_cleanup,
            }),
            close_elapsed_ms,
        ))
    })();

    if operation.is_err() {
        let _ = session.close();
        let _ = fs::write(out.join("drain-holder-stop"), b"stop");
        if let Ok(pid) = read_pid(&out.join("descendant.pid")) {
            let _ = wait_descendant_cleanup(out, pid);
        }
    }
    let (value, _) = operation?;
    Ok(value)
}

fn run_setup_lifecycle_probe(out: &Path) -> AppResult<Value> {
    let mut results = serde_json::Map::new();
    let mut failures = Vec::new();
    for (name, directory, probe) in [
        (
            "preexisting_sink",
            "preexisting-sink",
            run_preexisting_sink_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "live_uncaptured_to_capture",
            "live-uncaptured-to-captured",
            run_live_capture_rejection_probe as fn(&Path) -> AppResult<Value>,
        ),
        (
            "captured_restart",
            "captured-restart",
            run_captured_restart_rejection_probe as fn(&Path) -> AppResult<Value>,
        ),
    ] {
        let subdir = out.join(directory);
        fs::create_dir_all(&subdir)?;
        let result = match probe(&subdir) {
            Ok(value) => json!({"status":"passed", "result":value}),
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                json!({"status":"failed", "error":error.to_string()})
            }
        };
        write_json_new(&subdir.join("result.json"), &result)?;
        results.insert(name.to_string(), result);
    }
    if !failures.is_empty() {
        return Err(failures.join("; ").into());
    }
    Ok(Value::Object(results))
}

fn run_preexisting_sink_probe(out: &Path) -> AppResult<Value> {
    let session = Session::new("raw-probe-preexisting-sink".to_string());
    let sink = out.join("raw.ansi");
    let sentinel = b"existing raw capture file";
    write_new(&sink, sentinel)?;
    let startup = out.join("child-started.txt");
    let launch = child_launch(
        out,
        "lifecycle",
        &[("PROBE_STARTED_PATH", startup.as_os_str())],
    )?;
    let spawn_result = session.spawn_with_raw_output_capture(
        launch,
        run_options(out),
        RawOutputCaptureOptions::new(&sink),
    );
    let error = spawn_result
        .err()
        .ok_or("pre-existing capture sink did not reject setup")?;
    let actual = fs::read(&sink)?;
    if actual != sentinel {
        return Err("pre-existing capture sink changed after rejected setup".into());
    }
    if startup.exists() || session.is_open() {
        return Err("child started or session opened after capture setup failed".into());
    }
    Ok(json!({
        "setup_error":error.to_string(),
        "sentinel_unchanged":true,
        "child_startup_marker_absent":true,
        "session_open":false,
    }))
}

fn run_live_capture_rejection_probe(out: &Path) -> AppResult<Value> {
    let session = Session::new("raw-probe-live-capture-rejection".to_string());
    let startup = out.join("child-started.txt");
    let stop = out.join("child-stop");
    let launch = child_launch(
        out,
        "lifecycle",
        &[("PROBE_STARTED_PATH", startup.as_os_str())],
    )?;
    let first = session.spawn_with(launch, run_options(out))?;
    let pid = first
        .shell_pid
        .ok_or("uncaptured child PID is unavailable")?;
    session
        .get_by_text("LIVE-CHILD-READY")
        .wait_with_timeout(Some(CHILD_WAIT_MS))?;
    let capture_path = out.join("late-capture.ansi");
    let second_launch = child_launch(
        out,
        "lifecycle",
        &[("PROBE_STARTED_PATH", startup.as_os_str())],
    )?;
    let rejection = session.spawn_with_raw_output_capture(
        second_launch,
        run_options(out),
        RawOutputCaptureOptions::new(&capture_path),
    );
    let error = rejection
        .err()
        .ok_or("capture request attached to an existing uncaptured session")?;
    let state = read_state(&session)?;
    if state["exited"].is_number() || !state["exit_signal"].is_null() || !pid_alive(pid) {
        return Err("existing child did not remain alive after capture rejection".into());
    }
    if capture_path.exists() {
        return Err("rejected capture request created a capture sink".into());
    }
    if count_lines(&startup)? != 1 {
        return Err("capture rejection started another child".into());
    }

    fs::write(&stop, b"stop")?;
    wait_for_exit(&session)?;
    let clean_state = read_state(&session)?;
    require_clean_exit(&clean_state)?;
    session.close()?;
    Ok(json!({
        "rejection_error":error.to_string(),
        "prior_child_pid":pid,
        "prior_child_alive_after_rejection":true,
        "requested_sink_created":false,
        "startup_count":1,
        "prior_child_final_state":clean_state,
    }))
}

fn run_captured_restart_rejection_probe(out: &Path) -> AppResult<Value> {
    let session = Session::new("raw-probe-captured-restart".to_string());
    let raw_path = out.join("raw.ansi");
    let startup = out.join("child-started.txt");
    let stop = out.join("child-stop");
    let launch = child_launch(
        out,
        "lifecycle",
        &[("PROBE_STARTED_PATH", startup.as_os_str())],
    )?;
    let opened = session.spawn_with_raw_output_capture(
        launch,
        run_options(out),
        RawOutputCaptureOptions::new(&raw_path),
    )?;
    let pid = opened
        .shell_pid
        .ok_or("captured child PID is unavailable")?;
    session
        .get_by_text("LIVE-CHILD-READY")
        .wait_with_timeout(Some(CHILD_WAIT_MS))?;
    let restart = session.execute(Operation::Restart {
        graceful_timeout_ms: 500,
    });
    let error = restart
        .err()
        .ok_or("captured restart was not rejected before teardown")?;
    let state = read_state(&session)?;
    if state["exited"].is_number() || !state["exit_signal"].is_null() || !pid_alive(pid) {
        return Err("captured restart rejected after changing the prior child".into());
    }
    if count_lines(&startup)? != 1 {
        return Err("captured restart started an additional child".into());
    }
    let status = session
        .raw_output_capture_status()
        .ok_or("capture status is missing after rejected restart")?;
    if status.path != raw_path {
        return Err("rejected restart changed the capture sink".into());
    }
    fs::write(&stop, b"stop")?;
    wait_for_exit(&session)?;
    let clean_state = read_state(&session)?;
    require_clean_exit(&clean_state)?;
    session.close()?;
    let final_status = session
        .raw_output_capture_status()
        .ok_or("final capture status is missing after close")?;
    if !final_status.complete {
        return Err(
            format!("capture did not finish after clean child exit: {final_status:?}").into(),
        );
    }
    write_json_new(&out.join("raw-status-final.json"), &final_status)?;
    Ok(json!({
        "rejection_error":error.to_string(),
        "prior_child_pid":pid,
        "prior_child_alive_after_rejection":true,
        "startup_count":1,
        "capture_path_preserved":true,
        "prior_child_final_state":clean_state,
        "final_capture_status":final_status,
    }))
}

#[cfg(unix)]
fn run_runtime_write_failure_probe(out: &Path) -> AppResult<Value> {
    let limit = FileSizeLimit::set(1024)?;
    let session = Session::new("raw-probe-write-failure".to_string());
    let raw_path = out.join("raw.ansi");
    let launch = child_launch(out, "write-failure", &[])?;
    session.spawn_with_raw_output_capture(
        launch,
        run_options(out),
        RawOutputCaptureOptions::new(&raw_path),
    )?;
    wait_for_exit(&session)?;
    let state = read_state(&session)?;
    require_clean_exit(&state)?;
    session.close()?;
    let status = session
        .raw_output_capture_status()
        .ok_or("raw capture status is missing after write failure")?;
    drop(limit);
    write_json_new(&out.join("raw-status-final.json"), &status)?;
    if status.complete
        || !status
            .errors
            .iter()
            .any(|error| error.contains("raw output write failed"))
    {
        return Err(format!("runtime sink write failure was not reported: {status:?}").into());
    }
    let file_bytes = fs::metadata(&raw_path)?.len();
    if file_bytes != 1024 {
        return Err(format!("limited capture file has {file_bytes} bytes, expected 1024").into());
    }
    Ok(json!({
        "child":state,
        "file_size_limit_bytes":1024,
        "capture_file_bytes":file_bytes,
        "capture_status":status,
        "sigxfsz":"ignored in the probe process and restored after capture finalization",
    }))
}

#[cfg(not(unix))]
fn run_runtime_write_failure_probe(_out: &Path) -> AppResult<Value> {
    Ok(json!({"status":"not_run", "reason":"RLIMIT_FSIZE is not available on this platform"}))
}

fn child_binary_stream() -> AppResult<()> {
    let out = probe_dir()?;
    let mut stdout = io::stdout().lock();
    let first = chunk_one();
    stdout.write_all(&first)?;
    stdout.flush()?;
    fs::write(out.join("stage-one.ready"), b"ready")?;
    wait_file(&out.join("continue-one"), Duration::from_secs(30))?;
    let burst = vec![b'B'; 10_000];
    stdout.write_all(&burst)?;
    stdout.write_all(b"\x1b[2J\x1b[HTAIL:\xe7\xb5\x82")?;
    stdout.flush()?;
    fs::write(out.join("stage-two.ready"), b"ready")?;
    wait_file(&out.join("finish-child"), Duration::from_secs(30))?;
    Ok(())
}

fn child_cursor_blink() -> AppResult<()> {
    let out = probe_dir()?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(CURSOR_STAGE_A)?;
    stdout.flush()?;
    fs::write(out.join("stage-a.ready"), b"ready")?;
    wait_file(&out.join("continue-b"), Duration::from_secs(30))?;
    stdout.write_all(CURSOR_STAGE_B)?;
    stdout.flush()?;
    fs::write(out.join("stage-b.ready"), b"ready")?;
    wait_file(&out.join("finish"), Duration::from_secs(30))?;
    Ok(())
}

fn child_lifecycle() -> AppResult<()> {
    let out = probe_dir()?;
    if let Some(started_path) = std::env::var_os("PROBE_STARTED_PATH") {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(started_path)?;
        writeln!(file, "{}", std::process::id())?;
        file.flush()?;
    }
    io::stdout().write_all(b"\x1b[2J\x1b[HLIVE-CHILD-READY")?;
    io::stdout().flush()?;
    wait_file(&out.join("child-stop"), Duration::from_secs(30))?;
    Ok(())
}

fn child_drain_parent(exit_after_start: bool) -> AppResult<()> {
    let out = probe_dir()?;
    let executable = std::env::current_exe()?;
    let mut command = Command::new(executable);
    command.arg("--raw-probe-child").arg("drain-holder");
    command.env("RAW_DRAIN_PARENT_PID", std::process::id().to_string());
    let child = command.spawn()?;
    fs::write(out.join("descendant.pid"), child.id().to_string())?;
    wait_file(&out.join("drain-holder-started"), Duration::from_secs(5))?;
    let marker = if exit_after_start {
        b"\x1b[2J\x1b[HDRAIN-NORMAL-READY".as_slice()
    } else {
        b"\x1b[2J\x1b[HDRAIN-EXPLICIT-READY".as_slice()
    };
    io::stdout().write_all(marker)?;
    io::stdout().flush()?;
    if !exit_after_start {
        wait_file(&out.join("drain-parent-stop"), Duration::from_secs(30))?;
    }
    Ok(())
}

fn child_drain_holder() -> AppResult<()> {
    let out = probe_dir()?;
    #[cfg(unix)]
    let slave_path = pty_slave_path_from_stdout()?;
    #[cfg(unix)]
    let slave = open_pty_slave(&slave_path)?;
    // Keep the slave open after the PTY session leader exits.
    #[cfg(unix)]
    if unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN) } == libc::SIG_ERR {
        return Err(io::Error::last_os_error().into());
    }
    #[cfg(unix)]
    let holder_status = format!(
        "pid={} slave_fd={} ignore_sighup=true",
        std::process::id(),
        slave.as_raw_fd()
    );
    #[cfg(not(unix))]
    let holder_status = format!("pid={} detached=false", std::process::id());
    fs::write(out.join("drain-holder-started"), holder_status)?;
    let parent_pid = std::env::var("RAW_DRAIN_PARENT_PID")?.parse::<u32>()?;
    loop {
        if out.join("drain-holder-stop").exists() {
            fs::write(out.join("drain-holder-exited"), b"exited")?;
            return Ok(());
        }
        if !pid_alive(parent_pid) {
            fs::write(out.join("drain-holder-after-parent-exit"), b"parent-exited")?;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    #[cfg(unix)]
    wait_for_holder_write(&out, &slave_path)?;
    #[cfg(not(unix))]
    wait_for_holder_write(&out)?;
    wait_file(&out.join("drain-holder-stop"), Duration::from_secs(30))?;
    fs::write(out.join("drain-holder-exited"), b"exited")?;
    Ok(())
}

#[cfg(unix)]
fn wait_for_holder_write(out: &Path, slave_path: &str) -> AppResult<()> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !out.join("drain-holder-write").exists() {
        if out.join("drain-holder-stop").exists() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for the PTY holder write signal".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut slave = match open_pty_slave(slave_path) {
        Ok(slave) => slave,
        Err(error) => {
            fs::write(out.join("drain-holder-write-error"), error.to_string())?;
            return Ok(());
        }
    };
    if let Err(error) = slave.write_all(b"DRAIN-HOLD") {
        fs::write(out.join("drain-holder-write-error"), error.to_string())?;
        return Ok(());
    }
    if let Err(error) = slave.flush() {
        fs::write(out.join("drain-holder-write-error"), error.to_string())?;
        return Ok(());
    }
    fs::write(out.join("drain-holder-after-parent-output"), b"written")?;
    Ok(())
}

#[cfg(not(unix))]
fn wait_for_holder_write(out: &Path) -> AppResult<()> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !out.join("drain-holder-write").exists() {
        if out.join("drain-holder-stop").exists() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for the PTY holder write signal".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut stdout = io::stdout().lock();
    stdout.write_all(b"DRAIN-HOLD")?;
    stdout.flush()?;
    fs::write(out.join("drain-holder-after-parent-output"), b"written")?;
    Ok(())
}

fn wait_for_holder_output(out: &Path, timeout: Duration) -> AppResult<()> {
    let output = out.join("drain-holder-after-parent-output");
    let write_error = out.join("drain-holder-write-error");
    let deadline = Instant::now() + timeout;
    loop {
        if output.exists() {
            return Ok(());
        }
        if write_error.exists() {
            return Err(format!(
                "PTY holder could not write after parent exit: {}",
                fs::read_to_string(write_error)?
            )
            .into());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for PTY holder output".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn pty_slave_path_from_stdout() -> AppResult<String> {
    use std::ffi::CStr;

    let mut name = [0_i8; 1024];
    let result =
        unsafe { libc::ttyname_r(io::stdout().as_raw_fd(), name.as_mut_ptr(), name.len()) };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result).into());
    }
    Ok(unsafe { CStr::from_ptr(name.as_ptr()) }
        .to_str()?
        .to_string())
}

#[cfg(unix)]
fn open_pty_slave(path: &str) -> AppResult<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(path)?)
}

fn child_write_failure() -> AppResult<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(&vec![b'W'; 16_384])?;
    stdout.flush()?;
    Ok(())
}

fn chunk_one() -> Vec<u8> {
    [
        b"\x01\x00\xff\xf0\x28\x8c\x28\x1b]0;raw-probe\x07\x1b[2J\x1b[Hchunk-one:".as_slice(),
        "界".as_bytes(),
    ]
    .concat()
}

fn child_launch(
    out: &Path,
    mode: &str,
    extra_env: &[(&str, &std::ffi::OsStr)],
) -> AppResult<LaunchOptions> {
    let executable = std::env::current_exe()?;
    let mut launch = LaunchOptions::new(executable.into_os_string())
        .with_args(["--raw-probe-child", mode])
        .with_cwd(out)
        .with_env_mode(LaunchEnvMode::Cleared)
        .with_env("RAW_PROBE_DIR", out.as_os_str().to_owned());
    for (key, value) in extra_env {
        launch = launch.with_env(*key, value.to_os_string());
    }
    Ok(launch)
}

fn run_options(out: &Path) -> RunOptions {
    RunOptions {
        backend: Backend::default(),
        program: String::new(),
        args: Vec::new(),
        profile: Profile::default(),
        cols: COLS,
        rows: ROWS,
        cwd: Some(out.to_string_lossy().into_owned()),
        env: Vec::new(),
        wait_ready: Some(false),
        restart: false,
        timeouts: Timeouts::default(),
        recording: AutomaticRecording::default(),
        style: Style::default(),
    }
}

fn verify_decoded_marker(
    snapshot: &TerminalSnapshot,
    marker: &str,
    wide_character: &str,
) -> AppResult<()> {
    let decoded = snapshot
        .cells()
        .iter()
        .map(|cell| cell.text())
        .collect::<String>();
    if !decoded.contains(marker) {
        return Err(format!("decoded snapshot is missing {marker:?}").into());
    }
    let index = snapshot
        .cells()
        .iter()
        .position(|cell| cell.text() == wide_character)
        .ok_or_else(|| format!("decoded snapshot has no {wide_character:?} cell"))?;
    if snapshot.cells()[index].width() != 2 {
        return Err(format!("{wide_character:?} cell does not have width two").into());
    }
    if snapshot
        .cells()
        .get(index + 1)
        .is_none_or(|cell| cell.width() != 0)
    {
        return Err(format!("{wide_character:?} has no continuation cell").into());
    }
    Ok(())
}

fn wait_visual_quiet(session: &Session) -> AppResult<()> {
    let options = WaitQuietOptions::new(Duration::from_secs(5), Duration::from_millis(100))?
        .with_mode(WaitQuietMode::Visual);
    session.wait_quiet(&options)?;
    Ok(())
}

fn wait_for_exit(session: &Session) -> AppResult<()> {
    session.execute(Operation::WaitExit {
        timeout_ms: Some(CHILD_WAIT_MS),
    })?;
    Ok(())
}

fn read_state(session: &Session) -> AppResult<Value> {
    match session.execute(Operation::State)? {
        OperationResult::State(state) => Ok(json!({
            "exited":state.exited,
            "exit_signal":state.exit_signal,
            "cols":state.cols,
            "rows":state.rows,
        })),
        other => Err(format!("expected process state, got {other:?}").into()),
    }
}

fn require_clean_exit(state: &Value) -> AppResult<()> {
    if state["exited"] != 0 || !state["exit_signal"].is_null() {
        return Err(format!("child did not exit 0 without a signal: {state}").into());
    }
    Ok(())
}

fn verify_complete_status(
    status: &tui_test::api::RawOutputCaptureStatus,
    expected_bytes: u64,
) -> AppResult<()> {
    if !status.setup_complete
        || !status.complete
        || !status.drained
        || !status.flushed
        || !status.errors.is_empty()
        || status.captured_output_bytes != Some(expected_bytes)
        || status.processed_output_offset != Some(expected_bytes)
    {
        return Err(format!("raw capture status is not complete: {status:?}").into());
    }
    Ok(())
}

fn wait_capture_offset(
    session: &Session,
    raw_path: &Path,
    initial_offset: u64,
    expected_bytes: &[u8],
    timeout: Duration,
) -> AppResult<()> {
    let started = Instant::now();
    loop {
        let status = session
            .raw_output_capture_status()
            .ok_or("raw capture status is unavailable while waiting for holder output")?;
        if status
            .processed_output_offset
            .is_some_and(|offset| offset > initial_offset)
        {
            let raw = fs::read(raw_path)?;
            if raw.ends_with(expected_bytes) {
                return Ok(());
            }
        }
        if started.elapsed() >= timeout {
            let write_error =
                fs::read_to_string(raw_path.with_file_name("drain-holder-write-error"))
                    .unwrap_or_default();
            return Err(format!(
                "holder output did not reach the raw reader; offset={:?}, errors={:?}, write_error={write_error:?}",
                status.processed_output_offset, status.errors
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn probe_dir() -> AppResult<PathBuf> {
    std::env::var_os("RAW_PROBE_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "RAW_PROBE_DIR is missing".into())
}

fn wait_file(path: &Path, timeout: Duration) -> io::Result<()> {
    let started = Instant::now();
    while !path.exists() {
        if started.elapsed() >= timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out waiting for {}", path.display()),
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn read_pid(path: &Path) -> AppResult<u32> {
    wait_file(path, CHILD_WAIT)?;
    Ok(String::from_utf8(fs::read(path)?)?.trim().parse()?)
}

fn count_lines(path: &Path) -> AppResult<usize> {
    Ok(fs::read_to_string(path)?.lines().count())
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    false
}

fn wait_descendant_cleanup(out: &Path, pid: u32) -> AppResult<Value> {
    fs::write(out.join("drain-holder-stop"), b"stop")?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while pid_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut sigterm_sent = false;
    if pid_alive(pid) {
        #[cfg(unix)]
        unsafe {
            if libc::kill(pid as libc::pid_t, libc::SIGTERM) == 0 {
                sigterm_sent = true;
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while pid_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    if pid_alive(pid) {
        return Err(format!("owned PTY-holding descendant {pid} did not exit").into());
    }
    Ok(json!({
        "pid":pid,
        "exited":true,
        "sigterm_sent":sigterm_sent,
        "exit_marker":out.join("drain-holder-exited").exists(),
    }))
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

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(unix)]
struct FileSizeLimit {
    limit: libc::rlimit,
    signal: libc::sighandler_t,
}

#[cfg(unix)]
impl FileSizeLimit {
    fn set(bytes: libc::rlim_t) -> AppResult<Self> {
        let mut previous = unsafe { std::mem::zeroed::<libc::rlimit>() };
        if unsafe { libc::getrlimit(libc::RLIMIT_FSIZE, &mut previous) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
        let signal = unsafe { libc::signal(libc::SIGXFSZ, libc::SIG_IGN) };
        if signal == libc::SIG_ERR {
            return Err(io::Error::last_os_error().into());
        }
        let limited = libc::rlimit {
            rlim_cur: bytes.min(previous.rlim_max),
            rlim_max: previous.rlim_max,
        };
        if unsafe { libc::setrlimit(libc::RLIMIT_FSIZE, &limited) } != 0 {
            let error = io::Error::last_os_error();
            unsafe {
                libc::signal(libc::SIGXFSZ, signal);
            }
            return Err(error.into());
        }
        Ok(Self {
            limit: previous,
            signal,
        })
    }
}

#[cfg(unix)]
impl Drop for FileSizeLimit {
    fn drop(&mut self) {
        unsafe {
            libc::setrlimit(libc::RLIMIT_FSIZE, &self.limit);
            libc::signal(libc::SIGXFSZ, self.signal);
        }
    }
}
