//! Black-box status CLI contract tests.
//!
//! Migration map from the former Python test module:
//!
//! | Python test | Rust CLI case |
//! | --- | --- |
//! | `test_initial_conclusions_are_not_run_and_source_pair_is_exact` | `initial_status_keeps_products_not_run_and_pairs_exact` |
//! | `test_all_seven_commands_and_each_layer_are_visible` | `renders_all_seven_exact_commands_and_layers` |
//! | `test_role_changes_local_presentation_and_keeps_candidate_queue_link` | `role_changes_only_local_label_and_queue_target` |
//! | `test_cli_accepts_reference_role` | `reference_role_cli_renders_reference_presentation` |
//! | `test_visual_interaction_api_and_ownership_are_separate` | `keeps_visual_interaction_api_ownership_distinct` |
//! | `test_ci_and_dco_remain_separate_from_product_results` | `keeps_ci_dco_observations_separate` |
//! | `test_changed_frozen_tag_is_rejected` | `rejects_changed_frozen_tag` |
//! | `test_candidate_remote_mismatch_is_rejected` | `rejects_candidate_remote_identity_mismatch` |
//! | `test_new_typed_observations_do_not_require_reporter_code_changes` | `accepts_new_typed_observations_and_unknown_conclusions` |
//! | `test_unknown_ci_state_and_invalid_count_are_rejected` | `rejects_unknown_state_negative_count_and_non_https_reason_source` |
//! | `test_duplicate_or_unknown_task_records_are_rejected` | `rejects_duplicate_task_ids_and_unknown_states` |
//! | `test_generation_is_deterministic` | `generation_is_byte_deterministic` |
//!
//! `checked_in_records_render_from_exact_copies` additionally covers the real
//! source-facts and accepted-task files in the same disposable-CLI path.
//!
//! Every case invokes the copied production script through its CLI in a
//! disposable repository. These tests do not import Python modules or use a
//! production test hook.

use std::fs;
use std::path::{Path, PathBuf};

use termrock_visibility_tests::{CliOutput, TempRepo, Tool, run_cli};

const CANDIDATE_SHA: &str = "1111111111111111111111111111111111111111";
const REFERENCE_SHA: &str = "2222222222222222222222222222222222222222";
const TAG_COMMIT: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const TAG_OBJECT: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const FAILURE_REASON: &str = "Workflow file exceeds the maximum allowed size of 500 KB.";
const FAILURE_URL: &str =
    "https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033";

struct StatusFixture {
    repo: TempRepo,
    script: PathBuf,
}

impl StatusFixture {
    fn new() -> Self {
        let repo = TempRepo::new().expect("create disposable repository");
        let tool = repo
            .copy_tool_exact(Tool::Status)
            .expect("copy exact status CLI");
        let fixture = Self {
            repo,
            script: tool.path,
        };
        fixture.write_facts(&base_facts());
        fixture.write_tasks(&base_tasks());
        fixture
    }

    fn from_checked_in_records() -> Self {
        let repo = TempRepo::new().expect("create disposable repository");
        let tool = repo
            .copy_tool_exact(Tool::Status)
            .expect("copy exact status CLI");
        for relative_path in [
            "tools/visibility/source-facts.json",
            "docs/implementation/visibility/tasks.json",
        ] {
            repo.copy_project_file_exact(Path::new(relative_path))
                .expect("copy exact checked-in status input");
        }
        Self {
            repo,
            script: tool.path,
        }
    }

    fn run(&self, args: &[&str]) -> CliOutput {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        run_cli(&self.script, &args, self.repo.root(), &[], None).expect("run status CLI")
    }

    fn write_facts(&self, facts: &str) {
        let path = self.repo.root().join("tools/visibility/source-facts.json");
        fs::write(path, facts).expect("write source-facts fixture");
    }

    fn write_tasks(&self, tasks: &str) {
        let path = self
            .repo
            .root()
            .join("docs/implementation/visibility/tasks.json");
        fs::create_dir_all(path.parent().expect("task fixture parent"))
            .expect("create accepted-task fixture directory");
        fs::write(path, tasks).expect("write accepted-task fixture");
    }

    fn report(&self) -> String {
        fs::read_to_string(self.repo.root().join("STATUS.md")).expect("read generated status")
    }
}

fn base_facts() -> String {
    format!(
        r#"{{
  "schema_version": 1,
  "repository": "tailrocks/terminal-components-claude",
  "observed_at": "2026-10-08T03:27:36Z",
  "candidate": {{
    "branch": "termrock-implementation",
    "head_sha": "{candidate}",
    "local_head_sha": "{candidate}",
    "remote_head_sha": "{candidate}"
  }},
  "reference": {{
    "branch": "visual-baseline",
    "head_sha": "{reference}"
  }},
  "visual_authority": {{
    "tag": "visual-baseline",
    "tag_object_sha": "{tag_object}",
    "commit_sha": "{tag_commit}"
  }},
  "ci_observation": {{
    "run_id": "37721476033",
    "workflow": "CI",
    "created_at": "2026-10-08T03:10:42Z",
    "head_sha": "{candidate}",
    "status": "completed",
    "conclusion": "failure",
    "job_count": 0,
    "artifact_count": 0,
    "url": "{failure_url}",
    "failure_reason": {{
      "reason": "{failure_reason}",
      "source_url": "{failure_url}"
    }}
  }},
  "dco_observation": {{
    "check_run_id": "113129904697",
    "name": "DCO",
    "head_sha": "{candidate}",
    "status": "completed",
    "conclusion": "action_required",
    "affected_commit_count": 2,
    "details_url": "https://github.com/cncf/dco2"
  }},
  "ambient_tools": {{
    "python": "3.9.6",
    "mise_rust": "1.98.1",
    "cargo_nextest": "0.9.146",
    "nextest_command": "mise exec -- cargo nextest --version",
    "host": "aarch64-apple-darwin"
  }}
}}"#,
        candidate = CANDIDATE_SHA,
        reference = REFERENCE_SHA,
        tag_object = TAG_OBJECT,
        tag_commit = TAG_COMMIT,
        failure_url = FAILURE_URL,
        failure_reason = FAILURE_REASON,
    )
}

fn base_tasks() -> String {
    r#"{
  "schema_version": 1,
  "queue_revision": 7,
  "tasks": [
    {
      "work_id": "VIS-01",
      "priority": "P0",
      "state": "in_progress",
      "owner": "/root/status_luna",
      "reviewer": "/root/technical_review",
      "priority_reason": "Publish measured visibility evidence."
    },
    {
      "work_id": "VIS-02",
      "priority": "P0",
      "state": "ready",
      "owner": "/root/suite_luna",
      "reviewer": "/root/technical_review",
      "priority_reason": "Run paired binary checks."
    }
  ]
}"#
    .to_string()
}

fn output_text(output: &CliOutput) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn error_text(output: &CliOutput) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn replace_once(input: String, old: &str, new: &str) -> String {
    assert!(input.contains(old), "fixture does not contain `{old}`");
    let (before, after) = input.split_once(old).expect("fixture match");
    format!("{before}{new}{after}")
}

fn write_facts(fixture: &StatusFixture, facts: &str) {
    fixture.write_facts(facts);
}

#[test]
fn initial_status_keeps_products_not_run_and_pairs_exact() {
    let fixture = StatusFixture::new();
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    for conclusion in [
        "Visibility / Complete | NOT_RUN",
        "Refactor / Ready | NOT_RUN",
        "Reference / Qualified | NOT_RUN",
        "Command / Ready | NOT_RUN",
        "Evidence freshness | NOT_RUN",
    ] {
        assert!(report.contains(conclusion), "missing `{conclusion}`");
    }
    assert!(report.contains(CANDIDATE_SHA));
    assert!(report.contains(REFERENCE_SHA));
    assert!(report.contains(
        "Ready has its own column, separate from Build, Launch, First frame, input and interaction, Exit, Restoration, Visual, and Ownership."
    ));
    assert!(report.contains("required shared-case and per-component checkpoint sets are Unknown"));
    assert!(!report.contains("| Required set | 0 |"));
    assert!(report.contains("Candidate observed commit"));
    assert!(report.contains("Reference observed commit"));
    assert!(report.contains("Report commit | Set after this report is committed"));
    assert!(!report.contains("source S"));
    assert!(!report.contains("commit P"));
    assert!(report.contains("A run receipt is a record of execution results"));
    assert!(report.contains("NOT_RUN is a result status, not a run-receipt status"));
    assert!(report.contains("Product requirements remain in [CHECKLIST.md]"));
    assert!(!report.contains("is generated from accepted records"));
}

#[test]
fn checked_in_records_render_from_exact_copies() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied checked-in source facts"))
            .expect("parse copied checked-in source facts");
    let candidate_sha = facts["candidate"]["head_sha"]
        .as_str()
        .expect("candidate SHA in source facts");
    let reference_sha = facts["reference"]["head_sha"]
        .as_str()
        .expect("reference SHA in source facts");

    let output = fixture.run(&[]);
    let repeated = fixture.run(&[]);
    let report = output_text(&output);
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(repeated.exit_code, Some(0));
    assert_eq!(output.stdout, repeated.stdout);
    assert!(report.contains(candidate_sha));
    assert!(report.contains(reference_sha));
}

#[test]
fn renders_all_seven_exact_commands_and_layers() {
    let fixture = StatusFixture::new();
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    let commands = [
        ("RUN-01", "cargo run --release --bin showcase"),
        ("RUN-02", "cargo run --release --bin jackin-preview"),
        ("RUN-03", "cargo run --release --bin holla"),
        ("RUN-04", "cargo run --release --bin tablepro"),
        (
            "RUN-05",
            "cargo run --release --bin tablepro -- --connect Production",
        ),
        (
            "RUN-06",
            "cargo run --release --bin jackin-preview -- --scenario accounts-mixed",
        ),
        (
            "RUN-07",
            "cargo run --release --bin holla -- --scenario remote-host",
        ),
    ];
    for (id, command) in commands {
        assert!(report.contains(&format!("| {id} | {command} |")));
    }
    assert_eq!(
        report
            .lines()
            .filter(|line| line.starts_with("| RUN-"))
            .count(),
        7
    );
    for layer in [
        "Build",
        "Launch",
        "First frame",
        "Interaction",
        "Exit",
        "Restoration",
        "Visual",
        "Ownership",
        "Ready",
    ] {
        assert!(report.contains(layer), "missing layer `{layer}`");
    }
}

#[test]
fn role_changes_only_local_label_and_queue_target() {
    let fixture = StatusFixture::new();
    let candidate = output_text(&fixture.run(&[]));
    let reference = output_text(&fixture.run(&["--role", "reference"]));
    let queue_url = "https://github.com/tailrocks/terminal-components-claude/blob/termrock-implementation/WORK_QUEUE.md";

    assert!(candidate.contains("| Local role | Candidate (termrock-implementation) |"));
    assert!(reference.contains("| Local role | Reference (visual-baseline) |"));
    assert!(reference.contains(&format!("[WORK_QUEUE.md]({queue_url})")));
    assert!(reference.contains(&format!("[Work queue]({queue_url})")));
    for (id, command) in [
        ("RUN-01", "cargo run --release --bin showcase"),
        ("RUN-02", "cargo run --release --bin jackin-preview"),
        ("RUN-03", "cargo run --release --bin holla"),
        ("RUN-04", "cargo run --release --bin tablepro"),
        (
            "RUN-05",
            "cargo run --release --bin tablepro -- --connect Production",
        ),
        (
            "RUN-06",
            "cargo run --release --bin jackin-preview -- --scenario accounts-mixed",
        ),
        (
            "RUN-07",
            "cargo run --release --bin holla -- --scenario remote-host",
        ),
    ] {
        let row = format!("| {id} | {command} |");
        assert!(candidate.contains(&row));
        assert!(reference.contains(&row));
    }

    let candidate_common = candidate
        .replace(
            "| Local role | Candidate (termrock-implementation) |",
            "| Local role | LOCAL_ROLE |",
        )
        .replace("](WORK_QUEUE.md)", &format!("]({queue_url})"));
    let reference_common = reference.replace(
        "| Local role | Reference (visual-baseline) |",
        "| Local role | LOCAL_ROLE |",
    );
    assert_eq!(candidate_common, reference_common);
}

#[test]
fn reference_role_cli_renders_reference_presentation() {
    let fixture = StatusFixture::new();
    let output = fixture.run(&["--role", "reference"]);

    assert_eq!(output.exit_code, Some(0));
    assert!(output_text(&output).contains("| Local role | Reference (visual-baseline) |"));
}

#[test]
fn keeps_visual_interaction_api_ownership_distinct() {
    let fixture = StatusFixture::new();
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    for column in [
        "Reference visual",
        "Candidate visual",
        "Reference interaction",
        "Candidate interaction",
        "Reference API",
        "Candidate API",
        "Reference ownership",
        "Candidate ownership",
    ] {
        assert!(report.contains(column), "missing column `{column}`");
    }
    assert!(report.contains("NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN"));
}

#[test]
fn keeps_ci_dco_observations_separate() {
    let fixture = StatusFixture::new();
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("37721476033"));
    assert!(report.contains(FAILURE_REASON));
    assert!(report.contains(&format!("[source]({FAILURE_URL})")));
    assert!(report.contains("0 jobs; 0 artifacts"));
    assert!(report.contains("This workflow result is not a product test result"));
    assert!(report.contains("failure at"));
    assert!(report.contains("113129904697"));
    assert!(report.contains("action_required"));
    assert!(report.contains("separate from product results"));
    assert!(report.contains("cargo-nextest 0.9.146 through mise"));
}

#[test]
fn rejects_changed_frozen_tag() {
    let fixture = StatusFixture::new();
    let facts = replace_once(
        base_facts(),
        TAG_COMMIT,
        "0000000000000000000000000000000000000000",
    );
    write_facts(&fixture, &facts);
    let output = fixture.run(&["--write"]);

    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("frozen visual tag identity changed"));
    assert!(!fixture.repo.root().join("STATUS.md").exists());
}

#[test]
fn rejects_candidate_remote_identity_mismatch() {
    let fixture = StatusFixture::new();
    let facts = replace_once(
        base_facts(),
        &format!("\"remote_head_sha\": \"{CANDIDATE_SHA}\""),
        "\"remote_head_sha\": \"0000000000000000000000000000000000000000\"",
    );
    write_facts(&fixture, &facts);
    let output = fixture.run(&[]);

    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("remote candidate identity differs"));
}

#[test]
fn accepts_new_typed_observations_and_unknown_conclusions() {
    let fixture = StatusFixture::new();
    let mut facts = base_facts();
    facts = replace_once(
        facts,
        "\"run_id\": \"37721476033\"",
        "\"run_id\": \"123456\"",
    );
    facts = replace_once(facts, "actions/runs/37721476033", "actions/runs/123456");
    facts = replace_once(
        facts,
        "\"conclusion\": \"failure\"",
        "\"conclusion\": \"success\"",
    );
    facts = replace_once(facts, "\"job_count\": 0", "\"job_count\": 4");
    facts = replace_once(facts, "\"artifact_count\": 0", "\"artifact_count\": 1");
    facts = replace_once(
        facts,
        "\"check_run_id\": \"113129904697\"",
        "\"check_run_id\": \"654321\"",
    );
    facts = replace_once(
        facts,
        "\"conclusion\": \"action_required\"",
        "\"conclusion\": \"success\"",
    );
    facts = replace_once(
        facts,
        "\"affected_commit_count\": 2",
        "\"affected_commit_count\": 0",
    );
    facts = replace_once(
        facts,
        "\"cargo_nextest\": \"0.9.146\"",
        "\"cargo_nextest\": \"0.9.200\"",
    );
    facts = replace_once(
        facts,
        &format!(
            "\"failure_reason\": {{\n      \"reason\": \"{FAILURE_REASON}\",\n      \"source_url\": \"{FAILURE_URL}\"\n    }}"
        ),
        "\"failure_reason\": null",
    );
    write_facts(&fixture, &facts);
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("CI run 123456"));
    assert!(report.contains("4 jobs; 1 artifacts"));
    assert!(report.contains("DCO check 654321"));
    assert!(report.contains("cargo-nextest 0.9.200"));
    assert!(report.contains("success at"));
    assert!(report.contains("No commits are reported with sign-off problems"));
    assert!(!report.contains("Failure reason"));
    assert!(!report.contains("workflow failure"));

    let mut unknown = facts.clone();
    unknown = replace_once(
        unknown,
        "\"conclusion\": \"success\"",
        "\"conclusion\": null",
    );
    unknown = replace_once(
        unknown,
        "\"conclusion\": \"success\"",
        "\"conclusion\": null",
    );
    write_facts(&fixture, &unknown);
    let output = fixture.run(&[]);
    let report = output_text(&output);
    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("unknown at"));
    assert!(report.contains("DCO check 654321"));
    assert!(report.contains("| unknown; No commits are reported"));

    let mut changed_failure = facts;
    changed_failure = replace_once(
        changed_failure,
        "\"conclusion\": \"success\"",
        "\"conclusion\": \"failure\"",
    );
    changed_failure = replace_once(
        changed_failure,
        "\"failure_reason\": null",
        "\"failure_reason\": {\"reason\": \"A different recorded workflow failure.\", \"source_url\": \"https://example.test/run/123456\"}",
    );
    write_facts(&fixture, &changed_failure);
    let output = fixture.run(&[]);
    let report = output_text(&output);
    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("Failure reason: A different recorded workflow failure."));
    assert!(report.contains("[source](https://example.test/run/123456)"));
}

#[test]
fn rejects_unknown_state_negative_count_and_non_https_reason_source() {
    let fixture = StatusFixture::new();
    let unknown = replace_once(
        base_facts(),
        "\"conclusion\": \"failure\"",
        "\"conclusion\": \"invented\"",
    );
    fixture.write_facts(&unknown);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("unknown CI conclusion"));

    let negative = replace_once(base_facts(), "\"job_count\": 0", "\"job_count\": -1");
    fixture.write_facts(&negative);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("nonnegative integer"));

    let bad_url = replace_once(
        base_facts(),
        &format!("\"source_url\": \"{FAILURE_URL}\""),
        "\"source_url\": \"file:///tmp/failure\"",
    );
    fixture.write_facts(&bad_url);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("source URL must be HTTPS"));
}

#[test]
fn rejects_duplicate_task_ids_and_unknown_states() {
    let fixture = StatusFixture::new();
    let duplicate = replace_once(
        base_tasks(),
        "\"work_id\": \"VIS-02\"",
        "\"work_id\": \"VIS-01\"",
    );
    fixture.write_tasks(&duplicate);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("duplicate work ID"));

    let unknown = replace_once(
        base_tasks(),
        "\"state\": \"ready\"",
        "\"state\": \"passed\"",
    );
    fixture.write_tasks(&unknown);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("unknown task state"));
}

#[test]
fn generation_is_byte_deterministic() {
    let fixture = StatusFixture::new();
    let first = fixture.run(&[]);
    let second = fixture.run(&[]);
    assert_eq!(first.exit_code, Some(0));
    assert_eq!(second.exit_code, Some(0));
    assert_eq!(first.stdout, second.stdout);

    let write_one = fixture.run(&["--write"]);
    let report_one = fixture.report();
    let write_two = fixture.run(&["--write"]);
    let report_two = fixture.report();
    let check = fixture.run(&["--check"]);
    assert_eq!(write_one.exit_code, Some(0));
    assert_eq!(write_two.exit_code, Some(0));
    assert_eq!(report_one, report_two);
    assert_eq!(check.exit_code, Some(0));
}
