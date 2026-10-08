//! Black-box coverage for the deferred behavior CLI.
//!
//! Migration map from `tools/visibility/tests/test_deferred.py`:
//!
//! | Former Python test | Rust CLI test |
//! | --- | --- |
//! | `test_inventory_has_23_ignored_tests_and_25_case_references` | `inventory_has_exactly_23_ignored_tests_and_25_case_references` |
//! | `test_filter_uses_exact_names_and_explicit_writer_exclusion` | `filter_uses_exact_names_and_explicit_writer_exclusion` |
//! | `test_claim_records_accepted_revision_separately_from_current_queue` | `receipt_separates_accepted_and_current_queue_revisions` |
//! | `test_environment_pins_toolchain_jobs_and_aqua_nextest_without_mbx` | `environment_pins_toolchain_jobs_and_aqua_nextest_after_path_sanitizing` |
//! | `test_digest_tracks_workspace_sources_and_ignores_nonworkspace_crates` | `source_digest_tracks_workspace_and_ignores_nonworkspace_crates` |
//! | `test_run_command_keeps_failures_red_and_runs_all_selected_tests` | `run_command_keeps_all_selected_failures_visible` |
//! | `test_list_requires_exact_ignored_set_in_one_target` | `list_requires_the_exact_ignored_selection` |
//! | `test_list_reconciles_selected_ids_separately_from_nextest_total` | `list_reconciles_selected_count_separately_from_nextest_total` |
//! | `test_list_rejects_total_smaller_than_selected_set` | `list_rejects_total_smaller_than_the_selection` |
//! | `test_list_rejects_missing_or_new_test` | `list_rejects_missing_and_writer_like_selected_tests` |
//! | `test_list_rejects_nonignored_selection` | `list_rejects_a_selected_test_that_is_not_ignored` |
//! | `test_list_rejects_wrong_package_or_binary` | `list_rejects_a_wrong_package_or_binary` |
//! | `test_assertion_failure_stays_failed_without_expected_failure_inversion` | `assertion_failure_stays_failed_without_expected_failure_inversion` |
//! | `test_receipt_rows_retain_stable_deferral_ids_and_test_mapping` | `receipt_rows_keep_stable_ids_and_test_metadata` |
//! | `test_prefixed_nextest_event_name_maps_to_stable_test_name` | `prefixed_nextest_event_names_map_to_stable_test_names` |
//! | `test_unselected_libtest_cases_must_finish_ignored` | `unselected_started_cases_must_finish_ignored` |
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
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Map, Value, json};
use termrock_visibility_tests::{
    CliOutput, TempRepo, Tool, fixture_env, install_fixture_aliases, run_cli,
};

const DEFERRED_IDS: [&str; 23] = [
    "BD-01", "BD-02", "BD-03", "BD-04", "BD-05", "BD-06", "BD-07", "BD-08", "BD-09", "BD-10",
    "BD-11", "BD-12", "BD-13", "BD-14", "BD-15", "BD-16", "BD-17", "BD-18", "BD-20", "BD-21",
    "BD-22", "BD-23", "BD-24",
];

const DEFERRED_TESTS: [&str; 23] = [
    "w01_brand_press_paints_distinct_from_hover",
    "w02_button_disabled_suppresses_focus_gutter",
    "w03_checkbox_compact_markers",
    "w03_checkbox_checked_focus_hover_independent",
    "w04_toggle_compact_markers",
    "w05_radio_per_option_disabled",
    "w05_radio_no_horizontal",
    "w06_chipbar_closable_keyboard_dead",
    "w06_chipbar_close_press_then_reorder_or_delete",
    "w08_input_masked_clicks_follow_display",
    "w08_input_down_only_does_not_edit",
    "w08_input_navigation_paste_begins_editing",
    "w08_input_external_change_is_not_silently_overwritten",
    "w09_area_nav_enter_begins_without_newline",
    "w09_area_nav_page_home_end_scroll_without_editing",
    "w09_area_wheel_holds_past_caret_until_it_moves",
    "w10_select_popup_edge_fade",
    "w11_form_busy_enter_submit_blocked",
    "w13_filter_backspace_removes_grapheme",
    "w13_filter_wide_trail_cells_clear",
    "w13_filter_reseed_selects_first_eligible",
    "w16_steps_running_spinner_cycles_phases",
    "w17_tabs_focused_pressed_bold",
];

const WRITER_EXCLUSION: &str =
    "test(/(?i)(admission|approve|baseline|capture|snapshot|publish|write)/)";
const EXPECTED_FILTER_PREFIX: &str = "package(=termrock-conformance) & binary(=control_states) & (";
const RUST_TEST_SOURCE: &str = "crates/termrock-visibility-tests/tests/deferred.rs";
const BRANCH: &str = "termrock-implementation";
const OWNER: &str = "/root/deferred_execution";
const CLAIM_TOKEN: &str = "visibility-deferred-20261008-04";

struct DeferredFixture {
    repo: TempRepo,
    script: PathBuf,
    fixture_bin: PathBuf,
    path_value: String,
    list_path: PathBuf,
    run_path: PathBuf,
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
            "crates/termrock-conformance/tests/control_states.rs",
            RUST_TEST_SOURCE,
        ] {
            repo.copy_project_file_exact(Path::new(relative))
                .unwrap_or_else(|error| panic!("copy deferred fixture input {relative}: {error}"));
        }

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

    fn set_run(&self, bytes: &[u8]) {
        fs::write(&self.run_path, bytes).expect("write Nextest run fixture");
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
            &["run".to_owned()],
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

    fn trace(&self) -> Vec<Value> {
        let bytes = fs::read(&self.trace_path).expect("read synthetic tool trace");
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
        "queue_revision": 17,
        "tasks": [{
            "work_id": "VIS-04",
            "owner": OWNER,
            "claim_token": CLAIM_TOKEN,
            "branch": BRANCH,
            "state": "in_progress",
            "base_sha": base_sha,
            "accepted_queue_revision": 4,
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

fn list_metadata(total_count: usize, extras: &[(&str, bool, &str)]) -> Value {
    let mut testcases = Map::new();
    for name in DEFERRED_TESTS {
        testcases.insert(
            name.to_owned(),
            json!({
                "ignored": true,
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
    let statuses: Vec<(&str, &str)> = DEFERRED_TESTS.iter().map(|name| (*name, "ok")).collect();
    run_events(&statuses, false, &[])
}

fn all_fail_events() -> Vec<u8> {
    let statuses: Vec<(&str, &str)> = DEFERRED_TESTS
        .iter()
        .map(|name| (*name, "failed"))
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
fn inventory_has_exactly_23_ignored_tests_and_25_case_references() {
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
        DEFERRED_IDS.map(str::to_owned).to_vec()
    );
    assert_eq!(
        expected
            .iter()
            .map(|row| row["case_ids"].as_array().unwrap().len())
            .sum::<usize>(),
        25
    );
    assert!(expected.iter().all(|row| {
        row["ignore_reason"]
            .as_str()
            .is_some_and(|reason| reason.starts_with("PARITY "))
    }));
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
    for name in DEFERRED_TESTS {
        assert!(
            filter.contains(&format!("test(={name})")),
            "missing exact filter for {name}"
        );
    }
    assert!(filter.ends_with(&format!(") - {WRITER_EXCLUSION}")));
    assert!(filter.contains("test(=w01_brand_press_paints_distinct_from_hover)"));
    assert!(filter.contains("test(=w17_tabs_focused_pressed_bold)"));
    assert!(!filter.contains("test(w01_brand_press_paints_distinct_from_hover)"));

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
    assert_eq!(claim["accepted_queue_revision"], 4);
    assert_eq!(claim["queue_revision"], 17);
    assert_eq!(claim["claim_token"], CLAIM_TOKEN);
    assert_eq!(
        observed.receipt["subject"]["accepted_base_is_ancestor"],
        true
    );
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

    let unrelated_source = fixture.root().join("crates/termrock-e2e/src/lib.rs");
    fs::write(
        &unrelated_source,
        b"pub fn outside_workspace() { let _ignored = true; }\n",
    )
    .expect("edit non-workspace fixture source");
    let third = fixture.run();
    assert_cli_exit(&third, 0);
    assert_eq!(
        third.receipt["subject"]["source_inputs_sha256"],
        second.receipt["subject"]["source_inputs_sha256"]
    );
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
        "only"
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
fn list_requires_the_exact_ignored_selection() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let observed = fixture.run();
    assert_cli_exit(&observed, 0);
    assert_eq!(observed.receipt["execution"]["result"], "passed");
    assert_eq!(
        selected_ids(&observed.receipt),
        DEFERRED_IDS.map(str::to_owned)
    );
    let mut expected_names = DEFERRED_TESTS.map(str::to_owned).to_vec();
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
    missing["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        .as_object_mut()
        .unwrap()
        .remove(DEFERRED_TESTS[0]);
    fixture.set_list(&missing);
    let missing_observation = fixture.run();
    assert_cli_exit(&missing_observation, 2);
    assert!(
        missing_observation.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("selection mismatch")
    );

    let mut writer = list_metadata(24, &[]);
    writer["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        .as_object_mut()
        .unwrap()
        .insert(
            "admission_writer".to_owned(),
            json!({"ignored": true, "filter-match": {"status": "matches"}}),
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

#[test]
fn list_rejects_a_selected_test_that_is_not_ignored() {
    let fixture = DeferredFixture::new(list_metadata(23, &[]), &all_pass_events(), 0);
    let mut metadata = list_metadata(23, &[]);
    metadata["rust-suites"]["termrock-conformance::control_states"]["testcases"]
        [DEFERRED_TESTS[0]]["ignored"] = json!(false);
    fixture.set_list(&metadata);
    let observed = fixture.run();
    assert_cli_exit(&observed, 2);
    assert!(
        observed.receipt["execution"]["block_reason"]
            .as_str()
            .unwrap()
            .contains("no longer ignored")
    );
}

#[test]
fn list_rejects_a_wrong_package_or_binary() {
    for (field, value) in [
        ("package-name", "wrong-package"),
        ("binary-name", "visual_baseline"),
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
                .contains("unexpected suite")
        );
        assert_eq!(observed.receipt["tool"]["run_argv"], Value::Null);
    }
}

#[test]
fn assertion_failure_stays_failed_without_expected_failure_inversion() {
    let statuses = [(DEFERRED_TESTS[0], "failed")];
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
fn receipt_rows_keep_stable_ids_and_test_metadata() {
    let statuses = [(DEFERRED_TESTS[0], "failed")];
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
    assert_eq!(rows[0]["test_name"], DEFERRED_TESTS[0]);
    assert_eq!(rows[0]["status"], "failed");
    assert_eq!(rows[1]["requirement_id"], "BD-02");
    assert_eq!(rows[1]["status"], "not_run");
    assert_eq!(
        observed.receipt["tool_inputs"]["runner_test_source"]["path"],
        RUST_TEST_SOURCE
    );
}

#[test]
fn prefixed_nextest_event_names_map_to_stable_test_names() {
    let statuses = [(DEFERRED_TESTS[0], "ok")];
    let fixture =
        DeferredFixture::new(list_metadata(23, &[]), &run_events(&statuses, true, &[]), 1);
    let observed = fixture.run();
    assert_cli_exit(&observed, 1);
    let row = &observed.receipt["execution"]["executed_tests"][0];
    assert_eq!(row["test_name"], DEFERRED_TESTS[0]);
    assert_eq!(row["status"], "passed");
    assert_eq!(observed.receipt["execution"]["result"], "blocked");
}

#[test]
fn unselected_started_cases_must_finish_ignored() {
    let extra = [("ordinary_case", false, "does-not-match")];
    let statuses = [(DEFERRED_TESTS[0], "failed")];
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
    assert_eq!(observed.receipt["execution"]["parse_errors"], json!([]));
    assert_eq!(observed.receipt["execution"]["result"], "failed");
    assert_eq!(
        observed.receipt["execution"]["executed_tests"][0]["status"],
        "failed"
    );

    let incomplete = [ordinary_extra_event("ordinary_case", "started")];
    fixture.set_run(&run_events(&statuses, false, &incomplete));
    let incomplete_observation = fixture.run();
    assert_cli_exit(&incomplete_observation, 100);
    assert!(
        incomplete_observation.receipt["execution"]["parse_errors"][0]
            .as_str()
            .unwrap()
            .contains("did not finish as ignored")
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
            .contains("unselected test was not filtered")
    );
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
    let statuses = [(DEFERRED_TESTS[0], "failed")];
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
