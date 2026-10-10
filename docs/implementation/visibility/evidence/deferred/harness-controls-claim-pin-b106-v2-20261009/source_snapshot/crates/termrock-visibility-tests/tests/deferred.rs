//! Black-box coverage for the deferred behavior CLI.
//!
//! Migration map from `tools/visibility/tests/test_deferred.py`:
//!
//! | Former Python test | Rust CLI test |
//! | --- | --- |
//! | `test_inventory_has_23_ignored_tests_and_25_case_references` | `inventory_uses_canonical_23_rows_and_25_case_references` |
//! | `test_filter_uses_exact_names_and_explicit_writer_exclusion` | `filter_uses_exact_names_and_explicit_writer_exclusion` |
//! | `test_claim_records_accepted_revision_separately_from_current_queue` | `receipt_separates_accepted_and_current_queue_revisions` |
//! | `test_environment_pins_toolchain_jobs_and_aqua_nextest_without_mbx` | `environment_pins_toolchain_jobs_and_aqua_nextest_after_path_sanitizing` |
//! | `test_digest_tracks_workspace_sources_and_ignores_nonworkspace_crates` | `source_digest_tracks_workspace_and_ignores_nonworkspace_crates` |
//! | `test_run_command_keeps_failures_red_and_runs_all_selected_tests` | `run_command_keeps_all_selected_failures_visible` |
//! | `test_list_requires_exact_ignored_set_in_one_target` | `list_requires_the_exact_canonical_selection` |
//! | `test_list_reconciles_selected_ids_separately_from_nextest_total` | `list_reconciles_selected_count_separately_from_nextest_total` |
//! | `test_list_rejects_total_smaller_than_selected_set` | `list_rejects_total_smaller_than_the_selection` |
//! | `test_list_rejects_missing_or_new_test` | `list_rejects_missing_and_writer_like_selected_tests` |
//! | `test_list_rejects_nonignored_selection` | `list_accepts_mixed_registration_and_rejects_metadata_drift` |
//! | `test_list_rejects_wrong_package_or_binary` | `list_rejects_a_wrong_package_or_binary` |
//! | `test_assertion_failure_stays_failed_without_expected_failure_inversion` | `assertion_failure_stays_failed_without_expected_failure_inversion` |
//! | `test_receipt_rows_retain_stable_deferral_ids_and_test_mapping` | `receipt_rows_keep_stable_ids_and_test_metadata` |
//! | `test_prefixed_nextest_event_name_maps_to_stable_test_name` | `prefixed_nextest_event_names_map_to_stable_test_names` |
//! | `test_unselected_libtest_cases_must_finish_ignored` | `unselected_case_events_are_rejected_with_run_ignored_all` |
//! | `test_unselected_nonignored_result_is_rejected` | `unselected_terminal_results_are_rejected` |
//! | `test_build_error_is_blocked_with_raw_positive_exit` | `build_error_is_blocked_with_raw_exit_and_zero_executions` |
//! | `test_zero_exit_with_result_validation_error_is_blocked_nonzero` | `zero_exit_with_invalid_results_is_blocked_nonzero` |
//! | `test_signal_termination_maps_to_shell_status_and_cannot_pass` | `signal_termination_is_blocked_and_maps_to_shell_status` |
//! | `test_assertion_failure_remains_failed_even_if_another_event_is_unreadable` | `assertion_failure_remains_failed_with_an_unreadable_event` |
//!
//! Each Rust test copies and invokes the production CLI in a disposable Git
//! repository. Nextest output comes from the compiled Rust protocol fixture;
//! no Python test module or test runner is used.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use termrock_visibility_tests::{
    CliOutput, TempRepo, Tool, fixture_env, install_fixture_aliases, run_cli,
};

const EXPECTED_DEFERRED_IDS: [&str; 23] = [
    "BD-01", "BD-02", "BD-03", "BD-04", "BD-05", "BD-06", "BD-07", "BD-08", "BD-09", "BD-10",
    "BD-11", "BD-12", "BD-13", "BD-14", "BD-15", "BD-16", "BD-17", "BD-18", "BD-20", "BD-21",
    "BD-22", "BD-23", "BD-24",
];

const WRITER_EXCLUSION: &str =
    "test(/(?i)(admission|approve|baseline|capture|snapshot|publish|write|rebuild_review_html)/)";
const EXPECTED_FILTER_PREFIX: &str = "package(=termrock-conformance) & binary(=control_states) & (";
const RUST_TEST_SOURCE: &str = "crates/termrock-visibility-tests/tests/deferred.rs";
const DEFERRED_REGISTRY_SOURCE: &str =
    "crates/termrock-e2e/cases/deferred-obligations.json";
const HISTORICAL_REASONS_SOURCE: &str =
    "docs/implementation/visibility/evidence/deferred/deferred-reasons-9deb66b.json";
const CONTROL_TEST_SOURCE: &str = "crates/termrock-conformance/tests/control_states.rs";
const DEFERRED_REGISTRY_JSON: &str =
    include_str!("../../termrock-e2e/cases/deferred-obligations.json");
const HISTORICAL_REASONS_JSON: &str = include_str!(
    "../../../docs/implementation/visibility/evidence/deferred/deferred-reasons-9deb66b.json"
);
const CONTROL_STATES_TEXT: &str =
    include_str!("../../termrock-conformance/tests/control_states.rs");
const BRANCH: &str = "termrock-implementation";
const OWNER: &str = "/root";
const CLAIM_TOKEN: &str = "visibility-deferred-root-q33-20261009-01";
const OBSOLETE_OWNER: &str = "/root/current_branch_comparison_luna";
const OBSOLETE_CLAIM_TOKEN: &str = "visibility-deferred-recovery-20261008-04";
const ACCEPTED_QUEUE_REVISION: u64 = 34;
const CURRENT_QUEUE_REVISION: u64 = 38;

fn canonical_rows() -> Vec<Value> {
    let registry: Value =
        serde_json::from_str(DEFERRED_REGISTRY_JSON).expect("parse canonical deferred registry");
    registry["rows"]
        .as_array()
        .expect("canonical deferred rows")
        .clone()
}

fn deferred_ids() -> Vec<String> {
    canonical_rows()
        .iter()
        .map(|row| row["id"].as_str().expect("canonical deferred ID").to_owned())
        .collect()
}

fn deferred_test_names() -> Vec<String> {
    canonical_rows()
        .iter()
        .map(|row| {
            row["legacy_test"]
                .as_str()
                .expect("canonical deferred legacy_test")
                .to_owned()
        })
        .collect()
}

fn test_name_for_id(id: &str) -> String {
    canonical_rows()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("canonical registry has no {id}"))["legacy_test"]
        .as_str()
        .expect("canonical legacy test name")
        .to_owned()
}

fn fixed_mixed_control_source() -> String {
    let canonical_names = deferred_test_names();
    let retained_ignore = test_name_for_id("BD-21");
    let lines: Vec<&str> = CONTROL_STATES_TEXT.lines().collect();
    let mut output = String::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("#[ignore") {
            let following_function = lines[index + 1..]
                .iter()
                .map(|candidate| candidate.trim())
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with("#["))
                .and_then(|candidate| candidate.strip_prefix("fn "))
                .and_then(|rest| rest.split('(').next());
            if following_function.is_some_and(|name| {
                canonical_names.iter().any(|canonical| canonical == name)
                    && name != retained_ignore
            }) {
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn fixture_test_ignore_metadata() -> BTreeMap<String, bool> {
    let mut result = BTreeMap::new();
    let mut awaiting_function = false;
    let mut ignored = false;
    let source = fixed_mixed_control_source();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "#[test]" {
            awaiting_function = true;
            ignored = false;
            continue;
        }
        if !awaiting_function {
            continue;
        }
        if trimmed.starts_with("#[ignore") {
            ignored = true;
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if let Some(name) = rest.split('(').next() {
                result.insert(name.to_owned(), ignored);
            }
            awaiting_function = false;
            ignored = false;
        } else if !trimmed.starts_with("//") {
            awaiting_function = false;
            ignored = false;
        }
    }
    result
}

struct DeferredFixture {
    repo: TempRepo,
    script: PathBuf,
    fixture_bin: PathBuf,
    path_value: String,
    list_path: PathBuf,
    run_path: PathBuf,
    claim_pin_path: PathBuf,
    trace_path: PathBuf,
    run_exit: i32,
}

struct Observation {
    output: CliOutput,
    receipt: Value,
}

impl DeferredFixture {
    fn new(list: Value, run: &[u8], run_exit: i32) -> Self {
        let repo = TempRepo::new().expect("create temporary deferred repository");
        let script = repo
            .copy_tool_exact(Tool::Deferred)
            .expect("copy exact deferred CLI")
            .path;
        for relative in [
            "component-ownership.json",
            "tests/conformance/required_cases.json",
            "crates/termrock-conformance/Cargo.toml",
            "crates/termrock-conformance/tests/parity_burndown.rs",
            CONTROL_TEST_SOURCE,
            DEFERRED_REGISTRY_SOURCE,
            HISTORICAL_REASONS_SOURCE,
            RUST_TEST_SOURCE,
        ] {
            repo.copy_project_file_exact(Path::new(relative))
                .unwrap_or_else(|error| panic!("copy deferred fixture input {relative}: {error}"));
        }
        repo.write_file(
            Path::new(CONTROL_TEST_SOURCE),
            fixed_mixed_control_source().as_bytes(),
        )
        .expect("prepare mixed normal/ignored Rust source fixture");

        let root = repo.root();
        write(
            root,
            "Cargo.toml",
            b"[workspace]\nmembers = [\"crates/termrock-conformance\"]\n",
        );
        write(
            root,
            "Cargo.lock",
            b"version = 4\n\n# Synthetic CLI fixture; no Cargo build runs here.\n",
        );
        write(root, "mise.toml", b"[tools]\ncargo-nextest = \"0.9.146\"\n");
        write(
            root,
            "crates/termrock-conformance/src/lib.rs",
            b"pub fn fixture_input() {}\n",
        );
        write(
            root,
            "crates/termrock-e2e/src/lib.rs",
            b"pub fn outside_workspace() {}\n",
        );
        write(
            root,
            "fixtures/nextest/list.json",
            &serde_json::to_vec(&list).expect("encode list JSON"),
        );
        write(root, "fixtures/nextest/run.jsonl", run);
        write(
            root,
            "docs/implementation/visibility/tasks.json",
            &fixture_tasks("pending"),
        );
        let claim_pin_path = root.join("fixtures/claim-pin.json");
        write(
            root,
            "fixtures/claim-pin.json",
            &serde_json::to_vec(&claim_pin(
                OWNER,
                CLAIM_TOKEN,
                json!(ACCEPTED_QUEUE_REVISION),
            ))
            .expect("encode caller-pinned VIS-04 claim"),
        );

        git(root, &["init", "--quiet"]);
        git(root, &["checkout", "--quiet", "-b", BRANCH]);
        git(root, &["add", "--all"]);
        git_with_identity(
            root,
            &["commit", "--quiet", "-m", "deferred CLI fixture base"],
        );
        let base_sha = git_output(root, &["rev-parse", "HEAD"]);
        write(
            root,
            "docs/implementation/visibility/tasks.json",
            &fixture_tasks(&base_sha),
        );

        let list_path = root.join("fixtures/nextest/list.json");
        let run_path = root.join("fixtures/nextest/run.jsonl");
        let trace_path = root.join("fixtures/nextest/trace.jsonl");
        let fixture_bin = root.join("fixture-bin");
        let path_value = install_fixture_aliases(
            Path::new(env!("CARGO_BIN_EXE_fixture")),
            &fixture_bin,
            &["rustup", "mise", "cargo", "cargo-nextest"],
        )
        .expect("install compiled protocol fixture aliases")
        .to_string_lossy()
        .into_owned();

        Self {
            repo,
            script,
            fixture_bin,
            path_value,
            list_path,
            run_path,
            claim_pin_path,
            trace_path,
            run_exit,
        }
    }

    fn root(&self) -> &Path {
        self.repo.root()
    }

    fn set_list(&self, value: &Value) {
        fs::write(
            &self.list_path,
            serde_json::to_vec(value).expect("encode list metadata"),
        )
        .expect("write Nextest list fixture");
    }

    fn set_list_bytes(&self, bytes: &[u8]) {
        fs::write(&self.list_path, bytes).expect("write raw Nextest list fixture");
    }

    fn set_run(&self, bytes: &[u8]) {
        fs::write(&self.run_path, bytes).expect("write Nextest run fixture");
    }

    fn set_claim_pin(&self, bytes: &[u8]) {
        fs::write(&self.claim_pin_path, bytes).expect("write caller-pinned VIS-04 claim");
    }

    fn set_tasks(&self, bytes: &[u8]) {
        write(
            self.root(),
            "docs/implementation/visibility/tasks.json",
            bytes,
        );
    }

    fn run(&self) -> Observation {
        self.run_with(&[])
    }

    fn run_with(&self, overrides: &[(&str, String)]) -> Observation {
        let mut environment = BTreeMap::from([
            ("PATH".to_owned(), self.path_value.clone()),
            (
                fixture_env::NEXTTEST_LIST.to_owned(),
                self.list_path.display().to_string(),
            ),
            (
                fixture_env::NEXTTEST_RUN.to_owned(),
                self.run_path.display().to_string(),
            ),
            (
                fixture_env::NEXTTEST_EXIT.to_owned(),
                self.run_exit.to_string(),
            ),
            (fixture_env::NEXTTEST_LIST_EXIT.to_owned(), "0".to_owned()),
            (
                fixture_env::TRACE.to_owned(),
                self.trace_path.display().to_string(),
            ),
        ]);
        for (name, value) in overrides {
            environment.insert((*name).to_owned(), value.clone());
        }
        let borrowed_environment: Vec<(&str, &str)> = environment
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let output = run_cli(
            &self.script,
            &[
                "run".to_owned(),
                "--claim-pin".to_owned(),
                self.claim_pin_path.display().to_string(),
            ],
            self.root(),
            &borrowed_environment,
            None,
        )
        .expect("invoke production deferred CLI");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let printed_path = stdout
            .lines()
            .last()
            .and_then(|line| line.rsplit_once(": "))
            .map(|(path, _)| path)
            .unwrap_or_else(|| panic!("CLI did not print a receipt path; stdout={stdout:?}"));
        let receipt_path = self.root().join(printed_path);
        let receipt: Value =
            serde_json::from_slice(&fs::read(&receipt_path).unwrap_or_else(|error| {
                panic!(
                    "read receipt {}: {error}; stderr={}",
                    receipt_path.display(),
                    String::from_utf8_lossy(&output.stderr)
                )
            }))
            .unwrap_or_else(|error| panic!("parse receipt {}: {error}", receipt_path.display()));
        Observation { output, receipt }
    }

    fn run_with_source_drift(&self, action: &str) -> Observation {
        let trace_path = self.trace_path.clone();
        let source_path = self.root().join(DEFERRED_REGISTRY_SOURCE);
        let action = action.to_owned();
        let watcher = thread::spawn(move || {
            for _ in 0..5000 {
                if let Ok(trace) = fs::read_to_string(&trace_path) {
                    let found_action = trace.lines().filter_map(|line| {
                        serde_json::from_str::<Value>(line).ok()
                    }).any(|event| {
                        event["program"] == "rustup"
                            && event["argv"].as_array().is_some_and(|argv| {
                                argv.get(0).and_then(Value::as_str) == Some("run")
                                    && argv.get(2).and_then(Value::as_str) == Some("cargo")
                                    && argv.get(3).and_then(Value::as_str) == Some("nextest")
                                    && argv.get(4).and_then(Value::as_str) == Some(action.as_str())
                            })
                    });
                    if found_action {
                        let mut source = fs::OpenOptions::new()
                            .append(true)
                            .open(&source_path)
                            .expect("open source input for drift fixture");
                        source
                            .write_all(b"\n")
                            .expect("change source input at selected checkpoint");
                        return;
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
            panic!("did not observe Nextest {action} invocation");
        });
        let observed = self.run();
        watcher.join().expect("source drift watcher completed");
        observed
    }

    fn trace(&self) -> Vec<Value> {
        let bytes = match fs::read(&self.trace_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(error) => panic!("read synthetic tool trace: {error}"),
        };
        bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).expect("parse tool trace row"))
            .collect()
    }
}

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture path parent")).expect("create fixture parent");
    fs::write(path, bytes).expect("write fixture input");
}

fn fixture_tasks(base_sha: &str) -> Vec<u8> {
    serde_json::to_vec_pretty(&json!({
        "queue_revision": CURRENT_QUEUE_REVISION,
        "tasks": [{
            "work_id": "VIS-04",
            "owner": OWNER,
            "claim_token": CLAIM_TOKEN,
            "branch": BRANCH,
            "state": "claimed",
            "base_sha": base_sha,
            "accepted_queue_revision": ACCEPTED_QUEUE_REVISION,
            "allowed_paths": [
                "tools/visibility/deferred.py",
                "tools/visibility/tests/test_deferred.py",
                "docs/implementation/visibility/evidence/deferred/**",
                RUST_TEST_SOURCE
            ]
        }]
    }))
    .expect("encode accepted claim fixture")
}

fn claim_pin(owner: &str, token: &str, accepted_revision: Value) -> Value {
    json!({
        "schema": "termrock-visibility-claim-pin/v1",
        "work_id": "VIS-04",
        "expected_owner": owner,
        "expected_claim_token": token,
        "expected_accepted_queue_revision": accepted_revision
    })
}

fn list_metadata(total_count: usize, extras: &[(&str, bool, &str)]) -> Value {
    let mut testcases = Map::new();
    let ignore_metadata = fixture_test_ignore_metadata();
    for name in deferred_test_names() {
        let ignored = *ignore_metadata
            .get(&name)
            .unwrap_or_else(|| panic!("canonical test is not registered: {name}"));
        testcases.insert(
            name,
            json!({
                "ignored": ignored,
                "filter-match": {"status": "matches"}
            }),
        );
    }
    for (name, ignored, match_status) in extras {
        testcases.insert(
            (*name).to_owned(),
            json!({
                "ignored": ignored,
                "filter-match": {"status": match_status}
            }),
        );
    }
    json!({
        "test-count": total_count,
        "rust-suites": {
            "termrock-conformance::control_states": {
                "package-name": "termrock-conformance",
                "binary-name": "control_states",
                "kind": "test",
                "testcases": Value::Object(testcases)
            }
        }
    })
}

fn run_events(statuses: &[(&str, &str)], prefixed_names: bool, extra: &[Value]) -> Vec<u8> {
    let mut events = vec![json!({
        "type": "suite",
        "event": "started",
        "test_count": 23,
        "nextest": {
            "crate": "termrock-conformance",
            "test_binary": "control_states",
            "kind": "test"
        }
    })];
    for (name, terminal) in statuses {
        let event_name = if prefixed_names {
            format!("termrock-conformance::control_states${name}")
        } else {
            (*name).to_owned()
        };
        events.push(json!({"type": "test", "event": "started", "name": event_name}));
        let mut terminal_event = json!({
            "type": "test",
            "event": terminal,
            "name": event_name
        });
        if *terminal == "failed" {
            terminal_event["stdout"] = json!("assertion failed: expected deferred behavior");
        }
        events.push(terminal_event);
    }
    events.extend(extra.iter().cloned());
    let mut output = Vec::new();
    for event in events {
        serde_json::to_writer(&mut output, &event).expect("encode Nextest event");
        output.push(b'\n');
    }
    output
}

fn all_pass_events() -> Vec<u8> {
    let names = deferred_test_names();
    let statuses: Vec<(&str, &str)> = names.iter().map(|name| (name.as_str(), "ok")).collect();
    run_events(&statuses, false, &[])
}

fn all_fail_events() -> Vec<u8> {
    let names = deferred_test_names();
    let statuses: Vec<(&str, &str)> = names
        .iter()
        .map(|name| (name.as_str(), "failed"))
        .collect();
    run_events(&statuses, false, &[])
}

fn ordinary_extra_event(name: &str, event: &str) -> Value {
    json!({
        "type": "test",
        "event": event,
        "name": format!("termrock-conformance::control_states${name}")
    })
}

fn assert_cli_exit(observation: &Observation, expected: i32) {
    assert_eq!(
        observation.output.exit_code,
        Some(expected),
        "CLI exit mismatch; stdout={} stderr={}",
        String::from_utf8_lossy(&observation.output.stdout),
        String::from_utf8_lossy(&observation.output.stderr)
    );
    assert_eq!(observation.output.signal, None);
}

fn selected_ids(receipt: &Value) -> Vec<String> {
    receipt["selection"]["listed_tests"]
        .as_array()
        .expect("listed test IDs")
        .iter()
        .map(|value| value.as_str().expect("string ID").to_owned())
        .collect()
}

#[test]
fn inventory_uses_canonical_23_rows_and_25_case_references() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);

    let expected = observed.receipt["selection"]["expected_tests"]
        .as_array()
        .expect("expected test metadata");
    assert_eq!(expected.len(), 23);
    assert_eq!(observed.receipt["selection"]["count"], 23);
    assert_eq!(
        expected
            .iter()
            .map(|row| row["requirement_id"].as_str().unwrap())
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        EXPECTED_DEFERRED_IDS.map(str::to_owned).to_vec()
    );
    assert_eq!(
        deferred_ids(),
        EXPECTED_DEFERRED_IDS.map(str::to_owned).to_vec()
    );
    assert_eq!(
        expected
            .iter()
            .map(|row| row["case_ids"].as_array().unwrap().len())
            .sum::<usize>(),
        25
    );
    assert!(expected.iter().all(|row| row["registry_status"] == "NOT_RUN"));
    assert!(expected.iter().all(|row| {
        row["source_recorded_reason"]
            .as_str()
            .is_some_and(|reason| !reason.trim().is_empty())
            && row["source_reason_source"]["commit"] == "9deb66b48c99f674f23bb5ba183b3a8d0e57e534"
            && row["source_reason_source"]["path"]
                == "crates/termrock-conformance/tests/parity_burndown.rs"
            && row["source_reason_source"]["git_blob"] == "d7f588218e5128db53a374f641199f5736267b7b"
            && row["source_reason_source"]["raw_sha256"]
                == "037f56cd5b40585b1321c3919bd589031b7f2d56ef0b78ff0338f8b592899014"
            && row["source_reason_source"]["sidecar_path"] == HISTORICAL_REASONS_SOURCE
            && row["source_reason_source"]["sidecar_sha256"]
                == "90a08a85047f0b30fa159ec6a2779375db37472d734199d74b097f5edcdd9d9d"
    }));
    assert_eq!(observed.receipt["selection"]["registered_ignored_count"], 1);
    assert_eq!(observed.receipt["selection"]["registered_normal_count"], 22);
    assert_eq!(
        observed.receipt["selection"]["evidence_class"],
        "candidate-direct-assertions"
    );
    assert_eq!(observed.receipt["selection"]["qualifies_paired_pty_evidence"], false);
    assert!(expected.iter().any(|row| row["ignore_reason"].is_null()));
    assert!(expected.iter().any(|row| {
        row["requirement_id"] == "BD-21"
            && row["registered_ignored"] == true
            && row["ignore_reason"].as_str().is_some_and(|reason| reason.starts_with("PARITY "))
    }));

    let historical: Value = serde_json::from_str(HISTORICAL_REASONS_JSON)
        .expect("parse source-bound historical reason registry");
    let historical_rows = historical["rows"].as_array().expect("historical rows");
    assert_eq!(historical_rows.len(), 23);
    for (actual, historical) in expected.iter().zip(historical_rows) {
        assert_eq!(actual["requirement_id"], historical["id"]);
        assert_eq!(
            actual["source_recorded_reason"],
            historical["source_recorded_reason"]
        );
    }
}

#[test]
fn inventory_rejects_non_not_run_status_and_changed_reason_sidecar() {
    let status_fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let registry_path = status_fixture.root().join(DEFERRED_REGISTRY_SOURCE);
    let mut registry: Value = serde_json::from_slice(
        &fs::read(&registry_path).expect("read canonical registry fixture"),
    )
    .expect("parse canonical registry fixture");
    registry["rows"][0]["status"] = json!("PASS");
    fs::write(
        &registry_path,
        serde_json::to_vec(&registry).expect("encode changed canonical registry"),
    )
    .expect("write changed canonical registry");
    let status_observation = status_fixture.run();
    assert_cli_exit(&status_observation, 2);
    assert!(
        status_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("non-NOT_RUN status")
    );
    assert_eq!(status_observation.receipt["tool"]["run_argv"], Value::Null);

    let reason_fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let reasons_path = reason_fixture.root().join(HISTORICAL_REASONS_SOURCE);
    let mut reasons: Value = serde_json::from_slice(
        &fs::read(&reasons_path).expect("read historical reason sidecar fixture"),
    )
    .expect("parse historical reason sidecar fixture");
    reasons["source"]["git_blob"] = json!("changed");
    fs::write(
        &reasons_path,
        serde_json::to_vec(&reasons).expect("encode changed reason sidecar"),
    )
    .expect("write changed historical reason sidecar");
    let reason_observation = reason_fixture.run();
    assert_cli_exit(&reason_observation, 2);
    assert!(
        reason_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("sidecar content hash changed")
    );
    assert_eq!(reason_observation.receipt["tool"]["run_argv"], Value::Null);
}

#[test]
fn filter_uses_exact_names_and_explicit_writer_exclusion() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);

    let filter = observed.receipt["selection"]["filterset"]
        .as_str()
        .expect("filterset");
    assert!(filter.starts_with(EXPECTED_FILTER_PREFIX));
    for name in deferred_test_names() {
        assert!(
            filter.contains(&format!("test(={name})")),
            "missing exact filter for {name}"
        );
    }
    assert!(filter.ends_with(&format!(") - {WRITER_EXCLUSION}")));
    let first_name = test_name_for_id("BD-01");
    let last_name = test_name_for_id("BD-24");
    assert!(filter.contains(&format!("test(={first_name})")));
    assert!(filter.contains(&format!("test(={last_name})")));
    assert!(!filter.contains(&format!("test({first_name})")));
    assert!(WRITER_EXCLUSION.contains("rebuild_review_html"));

    let list_trace = fixture
        .trace()
        .into_iter()
        .find(|row| {
            row["program"] == "rustup"
                && row["argv"].as_array().is_some_and(|argv| {
                    argv.windows(5).any(|window| {
                        window[0].as_str() == Some("run")
                            && window[1].as_str() == Some("1.98.1")
                            && window[2].as_str() == Some("cargo")
                            && window[3].as_str() == Some("nextest")
                            && window[4].as_str() == Some("list")
                    })
                })
        })
        .expect("synthetic rustup received the exact Nextest list command");
    let list_args = list_trace["argv"].as_array().unwrap();
    let user_config_index = list_args
        .iter()
        .position(|argument| argument.as_str() == Some("--user-config-file"))
        .expect("list argv disables user Nextest config");
    assert_eq!(list_args[user_config_index + 1].as_str(), Some("none"));
    let filter_index = list_args
        .iter()
        .position(|argument| argument.as_str() == Some("--filterset"))
        .expect("list argv has --filterset");
    assert_eq!(list_args[filter_index + 1].as_str(), Some(filter));
}

#[test]
fn receipt_separates_accepted_and_current_queue_revisions() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);

    let claim = &observed.receipt["tool_inputs"]["claim"];
    assert_eq!(claim["accepted_queue_revision"], ACCEPTED_QUEUE_REVISION);
    assert_eq!(claim["queue_revision"], CURRENT_QUEUE_REVISION);
    assert_eq!(claim["claim_token"], CLAIM_TOKEN);
    let pin_bytes = fs::read(&fixture.claim_pin_path).expect("read caller-pinned claim");
    let expected_pin_hash = format!("{:x}", Sha256::digest(&pin_bytes));
    assert_eq!(claim["claim_pin_sha256"], expected_pin_hash);
    assert_eq!(
        observed.receipt["subject"]["accepted_base_is_ancestor"],
        true
    );
}

#[test]
fn claim_pin_rejects_obsolete_or_mismatched_identity_before_tool_setup() {
    let mut extra_field_pin = claim_pin(OWNER, CLAIM_TOKEN, json!(ACCEPTED_QUEUE_REVISION));
    extra_field_pin["unexpected"] = json!(true);
    let invalid_pins = [
        (
            "obsolete owner, token, and accepted revision",
            serde_json::to_vec(&claim_pin(
                OBSOLETE_OWNER,
                OBSOLETE_CLAIM_TOKEN,
                json!(24),
            ))
            .expect("encode obsolete caller pin"),
        ),
        (
            "wrong owner",
            serde_json::to_vec(&claim_pin(
                "/wrong-owner",
                CLAIM_TOKEN,
                json!(ACCEPTED_QUEUE_REVISION),
            ))
            .expect("encode caller pin with wrong owner"),
        ),
        (
            "wrong token",
            serde_json::to_vec(&claim_pin(
                OWNER,
                "wrong-token",
                json!(ACCEPTED_QUEUE_REVISION),
            ))
            .expect("encode caller pin with wrong token"),
        ),
        (
            "wrong accepted revision",
            serde_json::to_vec(&claim_pin(
                OWNER,
                CLAIM_TOKEN,
                json!(ACCEPTED_QUEUE_REVISION - 1),
            ))
            .expect("encode caller pin with wrong accepted revision"),
        ),
        (
            "non-integer expected accepted revision",
            serde_json::to_vec(&claim_pin(OWNER, CLAIM_TOKEN, json!(true)))
                .expect("encode caller pin with non-integer revision"),
        ),
        (
            "wrong work ID",
            serde_json::to_vec(&json!({
                "schema": "termrock-visibility-claim-pin/v1",
                "work_id": "VIS-05",
                "expected_owner": OWNER,
                "expected_claim_token": CLAIM_TOKEN,
                "expected_accepted_queue_revision": ACCEPTED_QUEUE_REVISION
            }))
            .expect("encode caller pin with wrong work ID"),
        ),
        (
            "unexpected pin field",
            serde_json::to_vec(&extra_field_pin).expect("encode caller pin with extra field"),
        ),
        (
            "duplicate pin field",
            br#"{"schema":"termrock-visibility-claim-pin/v1","work_id":"VIS-04","expected_owner":"/root","expected_owner":"/root","expected_claim_token":"visibility-deferred-root-q33-20261009-01","expected_accepted_queue_revision":34}"#.to_vec(),
        ),
    ];

    for (case, pin_bytes) in invalid_pins {
        let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
        fixture.set_claim_pin(&pin_bytes);
        let observed = fixture.run();
        assert_cli_exit(&observed, 2);
        assert_eq!(
            observed.receipt["execution"]["result"], "blocked",
            "invalid pin must block: {case}"
        );
        assert_eq!(observed.receipt["execution"]["counts"]["executed"], 0);
        assert_eq!(observed.receipt["tool"]["version"], Value::Null);
        assert_eq!(observed.receipt["tool"]["list_argv"], Value::Null);
        assert!(
            fixture.trace().is_empty(),
            "invalid pin must fail before version/setup probes: {case}"
        );
    }
}

#[test]
fn claim_registry_rejects_non_integer_revisions_before_tool_setup() {
    for accepted_revision_is_bool in [false, true] {
        let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
        let mut records: Value = serde_json::from_slice(
            &fs::read(
                fixture
                    .root()
                    .join("docs/implementation/visibility/tasks.json"),
            )
            .expect("read accepted task fixture"),
        )
        .expect("parse accepted task fixture");
        if accepted_revision_is_bool {
            records["tasks"][0]["accepted_queue_revision"] = json!(true);
        } else {
            records["queue_revision"] = json!(true);
        }
        fixture.set_tasks(
            &serde_json::to_vec(&records).expect("encode malformed queue revision fixture"),
        );

        let observed = fixture.run();
        assert_cli_exit(&observed, 2);
        assert_eq!(observed.receipt["execution"]["result"], "blocked");
        assert_eq!(observed.receipt["execution"]["counts"]["executed"], 0);
        assert_eq!(observed.receipt["tool"]["version"], Value::Null);
        assert!(
            fixture.trace().is_empty(),
            "boolean revision must fail before version/setup probes"
        );
    }
}

#[test]
fn environment_pins_toolchain_jobs_and_aqua_nextest_after_path_sanitizing() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let executable = Path::new(env!("CARGO_BIN_EXE_fixture"));
    let mbx_bin = fixture.root().join("Application Support/mbx/bin");
    let wrappers_bin = fixture.root().join("command-wrappers/bin");
    for directory in [&mbx_bin, &wrappers_bin] {
        install_fixture_aliases(
            executable,
            directory,
            &["rustup", "mise", "cargo", "cargo-nextest"],
        )
        .expect("install deliberately filtered PATH aliases");
    }
    let mut entries = vec![mbx_bin, wrappers_bin, fixture.fixture_bin.clone()];
    entries.extend(env::split_paths(
        env::var_os("PATH").as_deref().expect("host PATH"),
    ));
    let polluted_path = env::join_paths(entries)
        .expect("join test PATH")
        .to_string_lossy()
        .into_owned();

    let observed = fixture.run_with(&[("PATH", polluted_path)]);
    assert_cli_exit(&observed, 0);
    let tool = &observed.receipt["tool"];
    let inputs = &tool["environment_inputs"];
    assert_eq!(inputs["RUSTUP_TOOLCHAIN"], "1.98.1");
    assert_eq!(inputs["CARGO_BUILD_JOBS"], "2");
    assert_eq!(inputs["CARGO_TERM_COLOR"], "never");
    assert_eq!(inputs["NEXTEST_EXPERIMENTAL_LIBTEST_JSON"], "1");
    assert!(tool["version"].as_str().unwrap().contains("0.9.146"));
    assert_eq!(
        tool["cargo_path"],
        fixture.fixture_bin.join("cargo").display().to_string()
    );
    assert_eq!(
        tool["cargo_nextest_path"],
        fixture
            .fixture_bin
            .join("cargo-nextest")
            .display()
            .to_string()
    );
    assert_eq!(
        inputs["CARGO"],
        fixture.fixture_bin.join("cargo").display().to_string()
    );
}

#[test]
fn source_digest_tracks_workspace_and_ignores_nonworkspace_crates() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let first = fixture.run();
    assert_cli_exit(&first, 0);
    let first_digest = first.receipt["subject"]["source_inputs_sha256"]
        .as_str()
        .expect("initial source digest")
        .to_owned();

    let member_source = fixture
        .root()
        .join("crates/termrock-conformance/src/lib.rs");
    fs::write(
        &member_source,
        b"pub fn fixture_input() { let _changed = true; }\n",
    )
    .expect("edit workspace fixture source");
    let second = fixture.run();
    assert_cli_exit(&second, 0);
    let second_digest = second.receipt["subject"]["source_inputs_sha256"]
        .as_str()
        .expect("changed source digest")
        .to_owned();
    assert_ne!(first_digest, second_digest);

    let canonical_registry = fixture.root().join(DEFERRED_REGISTRY_SOURCE);
    let mut registry_bytes = fs::read(&canonical_registry).expect("read canonical registry fixture");
    registry_bytes.push(b'\n');
    fs::write(&canonical_registry, registry_bytes).expect("edit canonical registry fixture");
    let canonical = fixture.run();
    assert_cli_exit(&canonical, 0);
    let canonical_digest = canonical.receipt["subject"]["source_inputs_sha256"]
        .as_str()
        .expect("canonical source digest")
        .to_owned();
    assert_ne!(canonical_digest, second_digest);

    let runner_test = fixture.root().join(RUST_TEST_SOURCE);
    let mut runner_test_bytes = fs::read(&runner_test).expect("read runner test fixture");
    runner_test_bytes.push(b'\n');
    fs::write(&runner_test, runner_test_bytes).expect("edit runner test fixture");
    let runner_test_changed = fixture.run();
    assert_cli_exit(&runner_test_changed, 0);
    let runner_test_digest = runner_test_changed.receipt["subject"]["source_inputs_sha256"]
        .as_str()
        .expect("runner test source digest")
        .to_owned();
    assert_ne!(runner_test_digest, canonical_digest);

    let unrelated_source = fixture.root().join("crates/termrock-e2e/src/lib.rs");
    fs::write(
        &unrelated_source,
        b"pub fn outside_workspace() { let _ignored = true; }\n",
    )
    .expect("edit non-workspace fixture source");
    write(
        fixture.root(),
        "target/deferred-cache/fixture.bin",
        b"cache output is not a source input",
    );
    write(
        fixture.root(),
        "crates/termrock-conformance/target/deferred-cache/fixture.bin",
        b"nested target output is not a source input",
    );
    write(
        fixture.root(),
        "cache/deferred-cache/fixture.bin",
        b"cache output is not a source input",
    );
    write(
        fixture.root(),
        "crates/termrock-conformance/cache/deferred-cache/fixture.bin",
        b"nested cache output is not a source input",
    );
    write(
        fixture.root(),
        "docs/implementation/visibility/evidence/deferred/manual-note.txt",
        b"evidence output is not a source input",
    );
    let third = fixture.run();
    assert_cli_exit(&third, 0);
    assert_eq!(
        third.receipt["subject"]["source_inputs_sha256"],
        runner_test_changed.receipt["subject"]["source_inputs_sha256"]
    );
}

#[test]
fn source_drift_after_list_or_during_run_blocks_the_receipt() {
    for (action, expected_reason, run_expected) in [
        ("list", "changed after nextest list", false),
        ("run", "changed during nextest run", true),
    ] {
        let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
        let observed = fixture.run_with_source_drift(action);
        assert_cli_exit(&observed, 2);
        assert_eq!(observed.receipt["execution"]["result"], "blocked");
        assert!(
            observed.receipt["execution"]["block_reason"]
                .as_str()
                .unwrap()
                .contains(expected_reason)
        );
        let run_invoked = fixture.trace().iter().any(|row| {
            row["program"] == "rustup"
                && row["argv"].as_array().is_some_and(|argv| {
                    argv.get(4).and_then(Value::as_str) == Some("run")
                })
        });
        assert_eq!(run_invoked, run_expected);
    }
}

#[test]
fn run_command_keeps_all_selected_failures_visible() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_fail_events(), 100);
    let observed = fixture.run();
    assert_cli_exit(&observed, 100);

    let execution = &observed.receipt["execution"];
    assert_eq!(execution["result"], "failed");
    assert_eq!(execution["exit_code"], 100);
    assert_eq!(execution["counts"]["expected"], 23);
    assert_eq!(execution["counts"]["listed"], 23);
    assert_eq!(execution["counts"]["executed"], 23);
    assert_eq!(execution["counts"]["failed"], 23);
    assert_eq!(execution["expected_failure_treatment"], "none");

    let argv = observed.receipt["tool"]["run_argv"]
        .as_array()
        .expect("recorded Nextest run argv");
    let args: Vec<&str> = argv.iter().map(|arg| arg.as_str().unwrap()).collect();
    assert_eq!(&args[..5], &["rustup", "run", "1.98.1", "cargo", "nextest"]);
    assert!(args.contains(&"--locked"));
    assert_eq!(
        args[args
            .iter()
            .position(|arg| *arg == "--user-config-file")
            .expect("run argv disables user Nextest config")
            + 1],
        "none"
    );
    assert!(!args.contains(&"--offline"));
    assert!(args.contains(&"--no-fail-fast"));
    assert!(!args.contains(&"--max-fail"));
    assert_eq!(
        args[args.iter().position(|arg| *arg == "--no-tests").unwrap() + 1],
        "fail"
    );
    assert_eq!(
        args[args.iter().position(|arg| *arg == "--retries").unwrap() + 1],
        "0"
    );
    assert_eq!(
        args[args
            .iter()
            .position(|arg| *arg == "--test-threads")
            .unwrap()
            + 1],
        "1"
    );
    assert_eq!(
        args[args.iter().position(|arg| *arg == "--run-ignored").unwrap() + 1],
        "all"
    );
    assert!(!args.join(" ").to_lowercase().contains("xfail"));

    let run_trace = fixture
        .trace()
        .into_iter()
        .find(|row| {
            row["program"] == "rustup"
                && row["argv"].as_array().is_some_and(|argv| {
                    argv.windows(5).any(|window| {
                        window[0].as_str() == Some("run")
                            && window[1].as_str() == Some("1.98.1")
                            && window[2].as_str() == Some("cargo")
                            && window[3].as_str() == Some("nextest")
                            && window[4].as_str() == Some("run")
                    })
                })
        })
        .expect("synthetic rustup received the exact Nextest run command");
    let trace_args: Vec<&str> = run_trace["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    assert_eq!(trace_args.as_slice(), &args[1..]);
}

#[test]
fn list_requires_the_exact_canonical_selection() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);
    assert_eq!(observed.receipt["execution"]["result"], "passed");
    assert_eq!(
        selected_ids(&observed.receipt),
        EXPECTED_DEFERRED_IDS.map(str::to_owned)
    );
    let mut expected_names = deferred_test_names();
    expected_names.sort();
    assert_eq!(
        observed.receipt["selection"]["listed_test_names"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>(),
        expected_names
    );
    let listed = observed.receipt["selection"]["listed_test_metadata"]
        .as_array()
        .expect("listed test metadata");
    assert_eq!(listed.len(), 23);
    assert_eq!(listed.iter().filter(|row| row["ignored"] == true).count(), 1);
    assert_eq!(listed.iter().filter(|row| row["ignored"] == false).count(), 22);
    assert_eq!(
        listed
            .iter()
            .find(|row| row["ignored"] == true)
            .expect("one ignored canonical test")["requirement_id"],
        "BD-21"
    );
}

#[test]
fn list_reconciles_selected_count_separately_from_nextest_total() {
    let fixture = DeferredFixture::new(
        list_metadata(
            31,
            &[
                ("ordinary_unselected_test", false, "does-not-match"),
                ("snapshot_writer", false, "does-not-match"),
            ],
        ),
        &all_pass_events(),
        0,
    );
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);
    assert_eq!(observed.receipt["selection"]["listed_total_count"], 31);
    assert_eq!(observed.receipt["execution"]["counts"]["listed"], 23);
    assert_eq!(selected_ids(&observed.receipt).len(), 23);
}

#[test]
fn list_rejects_total_smaller_than_the_selection() {
    let fixture = DeferredFixture::new(list_metadata(22, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert_eq!(observed.receipt["execution"]["result"], "blocked");
    assert!(
        observed.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("fewer than the 23 selected tests")
    );
    assert_eq!(observed.receipt["tool"]["run_argv"], Value::Null);
}

#[test]
fn list_rejects_missing_and_writer_like_selected_tests() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let mut missing = list_metadata(23, &[]);
    let first_test = deferred_test_names()[0].clone();
    missing["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        .as_object_mut()
        .unwrap()
        .remove(&first_test);
    fixture.set_list(&missing);
    let missing_observation = fixture.run();
    assert_cli_exit(&missing_observation, 2);
    assert!(
        missing_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("selection mismatch")
    );

    let mut noncanonical = list_metadata(24, &[]);
    noncanonical["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        .as_object_mut()
        .unwrap()
        .insert(
            "ordinary_noncanonical_case".to_owned(),
            json!({"ignored": false, "filter-match": {"status": "matches"}}),
        );
    fixture.set_list(&noncanonical);
    let noncanonical_observation = fixture.run();
    assert_cli_exit(&noncanonical_observation, 2);
    assert!(
        noncanonical_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("selection mismatch")
    );

    for writer_name in ["admission_writer", "publish_writer", "rebuild_review_html"] {
        let mut writer = list_metadata(24, &[]);
        writer["rust-suites"]["termrock-conformance::control_states"]["testcases"]
            .as_object_mut()
            .unwrap()
            .insert(
                writer_name.to_owned(),
                json!({"ignored": false, "filter-match": {"status": "matches"}}),
            );
        fixture.set_list(&writer);
        let writer_observation = fixture.run();
        assert_cli_exit(&writer_observation, 2);
        assert!(
            writer_observation.receipt["execution"]["block_reason"]
                .as_str()
                .unwrap()
                .contains("writer-like")
        );
    }
}

#[test]
fn list_accepts_mixed_registration_and_rejects_metadata_drift() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let accepted = fixture.run();
    assert_cli_exit(&accepted, 0);
    assert_eq!(accepted.receipt["selection"]["registered_ignored_count"], 1);
    assert_eq!(accepted.receipt["selection"]["registered_normal_count"], 22);

    let ignored_name = test_name_for_id("BD-21");
    let normal_name = deferred_test_names()
        .into_iter()
        .find(|name| name != &ignored_name)
        .expect("one canonical normal test");
    let mut normal_drift = list_metadata(23, &[]);
    normal_drift["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        [normal_name.as_str()]["ignored"] = json!(true);
    fixture.set_list(&normal_drift);
    let normal_observation = fixture.run();
    assert_cli_exit(&normal_observation, 2);
    assert!(
        normal_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("ignored metadata differs")
    );

    let mut ignored_drift = list_metadata(23, &[]);
    ignored_drift["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        [ignored_name.as_str()]["ignored"] = json!(false);
    fixture.set_list(&ignored_drift);
    let ignored_observation = fixture.run();
    assert_cli_exit(&ignored_observation, 2);
    assert!(
        ignored_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("ignored metadata differs")
    );

    let mut missing_metadata = list_metadata(23, &[]);
    missing_metadata["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        [normal_name.as_str()]
        .as_object_mut()
        .unwrap()
        .remove("ignored");
    fixture.set_list(&missing_metadata);
    let missing_observation = fixture.run();
    assert_cli_exit(&missing_observation, 2);
    assert!(
        missing_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("omitted ignored metadata")
    );
}

#[test]
fn list_rejects_malformed_json_before_run() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    fixture.set_list_bytes(b"{not-json");
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert_eq!(observed.receipt["execution"]["result"], "blocked");
    assert!(
        observed.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("cannot read")
    );
    assert_eq!(observed.receipt["tool"]["run_argv"], Value::Null);
}

#[test]
fn list_rejects_a_wrong_package_or_binary() {
    for (field, value, expected_reason) in [
        ("package-name", "wrong-package", "unexpected suite"),
        ("binary-name", "visual_baseline", "unexpected suite"),
        ("kind", "lib", "non-integration-test binary"),
    ] {
        let mut metadata = list_metadata(23, &[]);
        metadata["rust-suites"]["termrock-conformance::control_states"][field] = json!(value);
        let fixture = DeferredFixture::new(metadata, &all_pass_events(), 0);
        let observed = fixture.run();
        assert_cli_exit(&observed, 2);
        assert!(
            observed.receipt["execution"]["block_reason"]
                .as_str()
                .unwrap()
                .contains(expected_reason)
        );
        assert_eq!(observed.receipt["tool"]["run_argv"], Value::Null);
    }

    let mut multiple = list_metadata(23, &[]);
    let suites = multiple["rust-suites"].as_object_mut().unwrap();
    let mut second = suites["termrock-conformance::control_states"].clone();
    let first_name = deferred_test_names()[0].clone();
    let mut second_cases = Map::new();
    second_cases.insert(
        first_name.clone(),
        json!({
            "ignored": false,
            "filter-match": {"status": "matches"}
        }),
    );
    second["testcases"] = Value::Object(second_cases);
    suites["termrock-conformance::control_states"]["testcases"]
        .as_object_mut()
        .unwrap()
        .remove(&first_name);
    suites.insert("termrock-conformance::control_states::duplicate".to_owned(), second);
    let fixture = DeferredFixture::new(multiple, &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert!(
        observed.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("one selected control_states test binary")
    );
}

#[test]
fn list_rejects_a_duplicate_selected_name_across_suites() {
    let mut metadata = list_metadata(23, &[]);
    let suites = metadata["rust-suites"].as_object_mut().unwrap();
    let suite = suites["termrock-conformance::control_states"].clone();
    let first_name = deferred_test_names()[0].clone();
    suites.insert("termrock-conformance::control_states::duplicate".to_owned(), suite);
    let fixture = DeferredFixture::new(metadata, &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert!(
        observed.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains(&format!("duplicate selected test {first_name}"))
    );
}

#[test]
fn assertion_failure_stays_failed_without_expected_failure_inversion() {
    let first_test = test_name_for_id("BD-01");
    let statuses = [(first_test.as_str(), "failed")];
    let fixture = DeferredFixture::new(
        list_metadata(23, &[]),
        &run_events(&statuses, false, &[]),
        100,
    );
    let observed = fixture.run();
    assert_cli_exit(&observed, 100);

    let execution = &observed.receipt["execution"];
    assert_eq!(execution["result"], "failed");
    assert_eq!(execution["exit_code"], 100);
    assert_eq!(execution["block_reason"], Value::Null);
    assert_eq!(execution["counts"]["executed"], 1);
    assert_eq!(execution["counts"]["failed"], 1);
    assert_eq!(execution["counts"]["not_run"], 22);
    assert_eq!(execution["expected_failure_treatment"], "none");
}

#[test]
fn non_success_terminal_events_stay_incomplete_and_cannot_follow_a_pass() {
    let names = deferred_test_names();
    let first_test = test_name_for_id("BD-01");
    let all_ok: Vec<(&str, &str)> = names
        .iter()
        .map(|name| (name.as_str(), "ok"))
        .collect();

    for terminal in ["timeout", "cancelled", "exec-failed"] {
        let single_status = [(first_test.as_str(), terminal)];
        let single_fixture = DeferredFixture::new(
            list_metadata(23, &[]),
            &run_events(&single_status, false, &[]),
            0,
        );
        let single = single_fixture.run();
        assert_cli_exit(&single, 2);
        assert_eq!(single.receipt["execution"]["result"], "blocked");
        let single_row = single.receipt["execution"]["executed_tests"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["test_name"].as_str() == Some(first_test.as_str()))
            .expect("non-success terminal result row");
        assert_eq!(single_row["status"], "incomplete", "{terminal}");

        let duplicate_fixture = DeferredFixture::new(
            list_metadata(23, &[]),
            &run_events(
                &all_ok,
                false,
                &[ordinary_extra_event(&first_test, terminal)],
            ),
            0,
        );
        let duplicate = duplicate_fixture.run();
        assert_cli_exit(&duplicate, 2);
        let execution = &duplicate.receipt["execution"];
        assert_eq!(execution["result"], "blocked", "{terminal}");
        assert_eq!(execution["exit_code"], 0, "{terminal} raw Nextest exit");
        assert_eq!(execution["counts"]["passed"], 22, "{terminal}");
        let duplicate_row = execution["executed_tests"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["test_name"].as_str() == Some(first_test.as_str()))
            .expect("contradictory terminal result row");
        assert_eq!(duplicate_row["status"], "incomplete", "{terminal}");
        let diagnostic = format!(
            "duplicate terminal result for {first_test}: ok then {terminal}"
        );
        assert!(execution["parse_errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error.as_str() == Some(diagnostic.as_str())));
    }

    let failure_statuses: Vec<(&str, &str)> = names
        .iter()
        .map(|name| {
            (
                name.as_str(),
                if name == &first_test { "failed" } else { "ok" },
            )
        })
        .collect();
    let failure_fixture = DeferredFixture::new(
        list_metadata(23, &[]),
        &run_events(
            &failure_statuses,
            false,
            &[ordinary_extra_event(&first_test, "ok")],
        ),
        100,
    );
    let failure = failure_fixture.run();
    assert_cli_exit(&failure, 100);
    let execution = &failure.receipt["execution"];
    assert_eq!(execution["result"], "failed");
    assert_eq!(execution["counts"]["failed"], 1);
    let failure_row = execution["executed_tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["test_name"] == first_test)
        .expect("real assertion failure row");
    assert_eq!(failure_row["status"], "failed");
    assert!(execution["parse_errors"][0]
        .as_str()
        .unwrap()
        .contains("failed then ok"));
    let stdout_path = failure_fixture.root().join(
        failure.receipt["artifacts"]["stdout"]["path"]
            .as_str()
            .expect("captured Nextest stdout artifact path"),
    );
    let captured_stdout =
        fs::read_to_string(stdout_path).expect("read captured Nextest event output");
    assert!(captured_stdout.contains("assertion failed: expected deferred behavior"));
}

#[test]
fn receipt_rows_keep_stable_ids_and_test_metadata() {
    let first_test = test_name_for_id("BD-01");
    let statuses = [(first_test.as_str(), "failed")];
    let fixture = DeferredFixture::new(
        list_metadata(23, &[]),
        &run_events(&statuses, false, &[]),
        100,
    );
    let observed = fixture.run();
    let rows = observed.receipt["execution"]["executed_tests"]
        .as_array()
        .expect("receipt outcome rows");
    assert_eq!(rows.len(), 23);
    assert_eq!(rows[0]["requirement_id"], "BD-01");
    assert_eq!(rows[0]["case_ids"], json!(["W01-02"]));
    assert_eq!(rows[0]["component_id"], "W01");
    assert_eq!(rows[0]["component_owner"], "termrock-controls");
    assert_eq!(rows[0]["dimension"], "behavior");
    assert_eq!(rows[0]["application"], Value::Null);
    assert_eq!(rows[0]["test_name"], json!(first_test));
    assert_eq!(rows[0]["registry_status"], "NOT_RUN");
    assert_eq!(rows[0]["status"], "failed");
    assert!(rows[0]["source_recorded_reason"].as_str().is_some());
    assert_eq!(rows[0]["source_reason_source"]["meaning"],
        "historical source-recorded context only; not a current diagnosis, ignore reason, assertion outcome, or PASS/FAIL result");
    assert_eq!(rows[0]["registered_ignored"], false);
    assert_eq!(rows[0]["ignore_reason"], Value::Null);
    assert_eq!(rows[1]["requirement_id"], "BD-02");
    assert_eq!(rows[1]["status"], "not_run");
    let bd21 = rows
        .iter()
        .find(|row| row["requirement_id"] == "BD-21")
        .expect("ignored canonical row");
    assert_eq!(bd21["registry_status"], "NOT_RUN");
    assert_eq!(bd21["registered_ignored"], true);
    assert!(bd21["ignore_reason"].as_str().is_some());
    assert_eq!(
        observed.receipt["tool_inputs"]["runner_test_source"]["path"],
        RUST_TEST_SOURCE
    );
}

#[test]
fn prefixed_nextest_event_names_map_to_stable_test_names() {
    let first_test = test_name_for_id("BD-01");
    let statuses = [(first_test.as_str(), "ok")];
    let fixture =
        DeferredFixture::new(list_metadata(23, &[]), &run_events(&statuses, true, &[]), 1);
    let observed = fixture.run();
    assert_cli_exit(&observed, 1);
    let row = &observed.receipt["execution"]["executed_tests"][0];
    assert_eq!(row["test_name"], json!(first_test));
    assert_eq!(row["status"], "passed");
    assert_eq!(observed.receipt["execution"]["result"], "blocked");
}

#[test]
fn unselected_case_events_are_rejected_with_run_ignored_all() {
    let extra = [("ordinary_case", false, "does-not-match")];
    let first_test = test_name_for_id("BD-01");
    let statuses = [(first_test.as_str(), "failed")];
    let events = [
        ordinary_extra_event("ordinary_case", "started"),
        ordinary_extra_event("ordinary_case", "ignored"),
    ];
    let fixture = DeferredFixture::new(
        list_metadata(24, &extra),
        &run_events(&statuses, false, &events),
        100,
    );
    let observed = fixture.run();
    assert_cli_exit(&observed, 100);
    assert_eq!(observed.receipt["execution"]["result"], "failed");
    assert!(
        observed.receipt["execution"]["parse_errors"][0]
            .as_str()
            .unwrap()
            .contains("unselected test emitted an event")
    );
}

#[test]
fn unselected_terminal_results_are_rejected() {
    let extra = [("writer_case", false, "does-not-match")];
    let events = [
        ordinary_extra_event("writer_case", "started"),
        ordinary_extra_event("writer_case", "ok"),
    ];
    let fixture = DeferredFixture::new(list_metadata(24, &extra), &[], 0);
    fixture.set_run(&run_events(&[], false, &events));
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert_eq!(observed.receipt["execution"]["result"], "blocked");
    assert!(
        observed.receipt["execution"]["parse_errors"][0]
            .as_str()
            .unwrap()
            .contains("unselected test emitted an event")
    );
}

#[test]
fn run_requires_one_terminal_event_per_canonical_test() {
    let missing_fixture = DeferredFixture::new(list_metadata(23, &[]), &[], 0);
    let missing = missing_fixture.run();
    assert_cli_exit(&missing, 2);
    assert_eq!(missing.receipt["execution"]["result"], "blocked");
    assert_eq!(missing.receipt["execution"]["counts"]["not_run"], 23);

    let first_name = test_name_for_id("BD-01");
    let mut duplicate_events = all_pass_events();
    serde_json::to_writer(
        &mut duplicate_events,
        &ordinary_extra_event(&first_name, "ok"),
    )
    .expect("encode duplicate terminal event");
    duplicate_events.push(b'\n');
    let duplicate_fixture =
        DeferredFixture::new(list_metadata(23, &[]), &duplicate_events, 0);
    let duplicate = duplicate_fixture.run();
    assert_cli_exit(&duplicate, 2);
    assert_eq!(duplicate.receipt["execution"]["result"], "blocked");
    assert!(duplicate.receipt["execution"]["parse_errors"][0]
        .as_str()
        .unwrap()
        .contains("duplicate terminal result"));

    let mut unknown_events = all_pass_events();
    serde_json::to_writer(
        &mut unknown_events,
        &ordinary_extra_event("unknown_test", "ok"),
    )
    .expect("encode unknown terminal event");
    unknown_events.push(b'\n');
    let unknown_fixture = DeferredFixture::new(list_metadata(23, &[]), &unknown_events, 0);
    let unknown = unknown_fixture.run();
    assert_cli_exit(&unknown, 2);
    assert_eq!(unknown.receipt["execution"]["result"], "blocked");
    assert!(unknown.receipt["execution"]["parse_errors"][0]
        .as_str()
        .unwrap()
        .contains("unexpected test name"));
}

#[test]
fn build_error_is_blocked_with_raw_exit_and_zero_executions() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &[], 101);
    let observed = fixture.run();
    assert_cli_exit(&observed, 101);
    let execution = &observed.receipt["execution"];
    assert_eq!(execution["exit_code"], 101);
    assert_eq!(execution["result"], "blocked");
    assert!(
        execution["block_reason"]
            .as_str()
            .unwrap()
            .contains("before any tests")
    );
    assert_eq!(execution["counts"]["expected"], 23);
    assert_eq!(execution["counts"]["listed"], 23);
    assert_eq!(execution["counts"]["executed"], 0);
    assert_eq!(execution["counts"]["failed"], 0);
    assert_eq!(execution["counts"]["not_run"], 23);
}

#[test]
fn zero_exit_with_invalid_results_is_blocked_nonzero() {
    let mut output = all_pass_events();
    output.extend_from_slice(b"not-json\n");
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &output, 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    let execution = &observed.receipt["execution"];
    assert_eq!(execution["result"], "blocked");
    assert_eq!(execution["counts"]["passed"], 23);
    assert!(
        execution["block_reason"]
            .as_str()
            .unwrap()
            .contains("could not be reconciled")
    );
    assert!(!execution["parse_errors"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn signal_termination_is_blocked_and_maps_to_shell_status() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run_with(&[(fixture_env::NEXTTEST_SIGNAL, "9".to_owned())]);
    assert_cli_exit(&observed, 137);
    let execution = &observed.receipt["execution"];
    assert_eq!(execution["exit_code"], -9);
    assert_eq!(execution["result"], "blocked");
    assert!(
        execution["block_reason"]
            .as_str()
            .unwrap()
            .contains("signal 9")
    );
    assert_eq!(execution["counts"]["passed"], 23);
}

#[test]
fn assertion_failure_remains_failed_with_an_unreadable_event() {
    let first_test = test_name_for_id("BD-01");
    let statuses = [(first_test.as_str(), "failed")];
    let mut output = run_events(&statuses, false, &[]);
    output.extend_from_slice(b"not-json\n");
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &output, 1);
    let observed = fixture.run();
    assert_cli_exit(&observed, 1);
    let execution = &observed.receipt["execution"];
    assert_eq!(execution["result"], "failed");
    assert_eq!(execution["block_reason"], Value::Null);
    assert_eq!(execution["exit_code"], 1);
    assert_eq!(execution["counts"]["failed"], 1);
    assert!(!execution["parse_errors"].as_array().unwrap().is_empty());
}

fn git(root: &Path, args: &[&str]) {
    let output = git_command(root, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_with_identity(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Deferred CLI Tests")
        .env("GIT_AUTHOR_EMAIL", "deferred-tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "Deferred CLI Tests")
        .env("GIT_COMMITTER_EMAIL", "deferred-tests@example.invalid")
        .output()
        .expect("start Git fixture command");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_command(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("start Git fixture command")
}

fn git_output(root: &Path, args: &[&str]) -> String {
    let output = git_command(root, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git output is UTF-8")
        .trim()
        .to_owned()
}
