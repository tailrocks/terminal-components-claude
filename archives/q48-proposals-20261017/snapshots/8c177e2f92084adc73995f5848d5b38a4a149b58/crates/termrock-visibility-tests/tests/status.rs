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
#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(unix)]
use std::time::Duration;

use termrock_visibility_tests::{
    CliOutput, TempRepo, Tool, run_cli, run_cli_with_timeout, sha256_hex,
};

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
    queue_script: PathBuf,
}

impl StatusFixture {
    fn new() -> Self {
        let repo = TempRepo::new().expect("create disposable repository");
        let status_tool = repo
            .copy_tool_exact(Tool::Status)
            .expect("copy exact status CLI");
        let queue_tool = repo
            .copy_tool_exact(Tool::Queue)
            .expect("copy exact adjacent queue CLI");
        let fixture = Self {
            repo,
            script: status_tool.path,
            queue_script: queue_tool.path,
        };
        fixture.write_facts(&base_facts());
        fixture.write_tasks(&base_tasks());
        fixture
    }

    fn from_checked_in_records() -> Self {
        let repo = TempRepo::new().expect("create disposable repository");
        let status_tool = repo
            .copy_tool_exact(Tool::Status)
            .expect("copy exact status CLI");
        let queue_tool = repo
            .copy_tool_exact(Tool::Queue)
            .expect("copy exact adjacent queue CLI");
        for relative_path in [
            "tools/visibility/source-facts.json",
            "docs/implementation/visibility/tasks.json",
            "WORK_QUEUE.md",
        ] {
            repo.copy_project_file_exact(Path::new(relative_path))
                .expect("copy exact checked-in status input");
        }
        Self::copy_manifest_members(
            &repo,
            Path::new("docs/implementation/visibility/evidence/reports/status-source-archive-20261009/MANIFEST.json"),
            None,
        );
        Self::copy_manifest_members(
            &repo,
            Path::new("docs/implementation/visibility/evidence/deferred/product23-s1d-20261009/SHA256SUMS.json"),
            Some(Path::new("docs/implementation/visibility/evidence/deferred/product23-s1d-20261009")),
        );
        Self {
            repo,
            script: status_tool.path,
            queue_script: queue_tool.path,
        }
    }

    fn copy_manifest_members(repo: &TempRepo, manifest_path: &Path, member_root: Option<&Path>) {
        let manifest_copy = repo
            .copy_project_file_exact(manifest_path)
            .expect("copy exact evidence manifest");
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(manifest_copy.path).expect("read copied evidence manifest"),
        )
        .expect("parse copied evidence manifest");
        let members = manifest["files"].as_array().expect("manifest member array");
        for member in members {
            let member_path = Path::new(
                member["path"].as_str().expect("manifest member path"),
            );
            let relative_path = match member_root {
                Some(root) => root.join(member_path),
                None => member_path.to_path_buf(),
            };
            repo.copy_project_file_exact(&relative_path)
                .expect("copy exact manifest member");
        }
    }

    fn run(&self, args: &[&str]) -> CliOutput {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        run_cli(&self.script, &args, self.repo.root(), &[], None).expect("run status CLI")
    }

    fn run_queue(&self, args: &[&str]) -> CliOutput {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        run_cli(&self.queue_script, &args, self.repo.root(), &[], None)
            .expect("run copied queue CLI")
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
  "schema_version": 2,
  "repository": "tailrocks/terminal-components-claude",
  "observed_at": "2026-10-08T03:27:36Z",
  "observation_sources": {{ "observed_at": "2026-10-08T03:27:36Z" }},
  "latest_source_observation": {{
    "observed_at": "2026-10-08T04:30:00Z",
    "method": "read-only GitHub branch API observation",
    "candidate_remote": {{
      "branch": "termrock-implementation",
      "head_sha": "3333333333333333333333333333333333333333",
      "updated_at": "2026-10-08T04:20:00Z"
    }},
    "reference_remote": {{
      "branch": "visual-baseline",
      "head_sha": "4444444444444444444444444444444444444444",
      "updated_at": "2026-10-08T04:10:00Z"
    }},
    "prior_candidate_remote_observation": {{
      "head_sha": "5555555555555555555555555555555555555555",
      "observed_at": null,
      "qualification": "Earlier observation with no retained retrieval time."
    }},
    "observation_sources": {{
      "commands": [
        "gh api repos/tailrocks/terminal-components-claude/branches/termrock-implementation --jq '{{sha:.commit.sha, updated_at:.commit.commit.committer.date}}'",
        "gh api repos/tailrocks/terminal-components-claude/branches/visual-baseline --jq '{{sha:.commit.sha, updated_at:.commit.commit.committer.date}}'"
      ]
    }}
  }},
  "latest_local_checkout_observation": {{
    "observed_at": "2026-10-08T04:31:00Z",
    "branch": "termrock-implementation",
    "head_sha": "6666666666666666666666666666666666666666",
    "signature_status": "N",
    "signature_command": "git show -s --format='%G?' HEAD",
    "publication_status": "unpublished_pending_correction",
    "qualification": "Local checkout observation only."
  }},
  "latest_local_dco_trailer_observation": {{
    "observed_at": "2026-10-08T04:32:00Z",
    "commit_sha": "6666666666666666666666666666666666666666",
    "expected_trailer": "Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>",
    "trailer_present": false,
    "parsed_trailers": [],
    "message_subject": "Local fixture commit",
    "inspection_command": "git show -s --format=%B HEAD | git interpret-trailers --parse",
    "qualification": "Fixture body inspection only."
  }},
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

fn current_ci_observation() -> serde_json::Value {
    serde_json::json!({
        "api_capture": {
            "captured_at": "2026-10-08T04:33:53Z",
            "manifest_path": "fixtures/current-ci/manifest.json",
            "manifest_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "source_pair": {
                "candidate_head_sha": "3333333333333333333333333333333333333333",
                "reference_head_sha": "4444444444444444444444444444444444444444"
            },
            "candidate_run": {
                "run_id": "987654321",
                "workflow_path": ".github/workflows/ci.yml",
                "head_sha": "3333333333333333333333333333333333333333",
                "status": "completed",
                "conclusion": "failure",
                "created_at": "2026-10-08T04:05:40Z",
                "head_sha_query_total_count": 1,
                "job_count": 0,
                "artifact_count": 0,
                "run_url": "https://github.com/tailrocks/terminal-components-claude/actions/runs/987654321"
            },
            "reference_query": {
                "query_status": "success",
                "head_sha": "4444444444444444444444444444444444444444",
                "workflow_path": ".github/workflows/ci.yml",
                "query_total_count": 0,
                "matching_run_ids": [],
                "request_url": "https://api.github.com/repos/tailrocks/terminal-components-claude/actions/runs?head_sha=4444444444444444444444444444444444444444&per_page=100",
                "response_file": "fixtures/current-ci/runs-by-sha-reference.json"
            },
            "workflow_source": {
                "path": ".github/workflows/ci.yml",
                "commit_sha": "3333333333333333333333333333333333333333",
                "github_blob_sha": "7777777777777777777777777777777777777777",
                "size_bytes": 537470,
                "raw_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            },
            "candidate_dco": {
                "check_run_id": "888888888",
                "name": "DCO",
                "head_sha": "3333333333333333333333333333333333333333",
                "status": "completed",
                "conclusion": "action_required",
                "affected_commit_count": 2,
                "annotations_count": 0,
                "result_url": "https://github.com/tailrocks/terminal-components-claude/runs/888888888"
            },
            "reference_dco": {
                "check_run_id": "999999999",
                "name": "DCO",
                "head_sha": "4444444444444444444444444444444444444444",
                "status": "completed",
                "conclusion": "action_required",
                "affected_commit_count": 14,
                "annotations_count": 0,
                "result_url": "https://github.com/tailrocks/terminal-components-claude/runs/999999999"
            }
        },
        "provider_annotation_capture": {
            "captured_at": "2026-10-08T05:02:19Z",
            "manifest_path": "fixtures/current-ci/annotation-manifest.json",
            "manifest_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "run_id": "987654321",
            "run_page_url": "https://github.com/tailrocks/terminal-components-claude/actions/runs/987654321",
            "workflow_path": ".github/workflows/ci.yml",
            "workflow_commit_sha": "3333333333333333333333333333333333333333",
            "page_status": "Failure",
            "page_annotation_count": 1,
            "annotation": "Workflow file exceeds the maximum allowed size of 500 KB. See https://docs.github.com/actions/reference/limits#workflow-file-size for more information.",
            "raw_page_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "workflow_source_size_bytes": 537470,
            "workflow_source_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
        }
    })
}

fn facts_with_current_ci_observation() -> serde_json::Value {
    let mut facts: serde_json::Value =
        serde_json::from_str(&base_facts()).expect("parse base facts fixture");
    facts["current_ci_observation"] = current_ci_observation();
    facts
}

fn with_four_predecessors(mut facts: serde_json::Value) -> serde_json::Value {
    if facts.get("source_observation_history").is_none() {
        facts["source_observation_history"] = serde_json::json!([
            {
                "source_observation": {
                    "observed_at": "2026-10-08T04:00:00Z",
                    "method": "read-only exact-source comparison",
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": "7777777777777777777777777777777777777777"
                    },
                    "reference_remote": {
                        "branch": "visual-baseline",
                        "head_sha": "8888888888888888888888888888888888888888"
                    },
                    "branch_last_update_at": null
                },
                "source_evidence": {
                    "path": "fixtures/source-observation/history-0.json",
                    "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                },
                "current_ci_observation": null,
                "superseded_at": "2026-10-08T04:10:00Z"
            },
            {
                "source_observation": {
                    "observed_at": "2026-10-08T04:10:00Z",
                    "method": "read-only exact-source comparison",
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": "9999999999999999999999999999999999999999"
                    },
                    "reference_remote": {
                        "branch": "visual-baseline",
                        "head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    },
                    "branch_last_update_at": null
                },
                "source_evidence": {
                    "path": "fixtures/source-observation/history-1.json",
                    "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                },
                "current_ci_observation": null,
                "superseded_at": "2026-10-08T04:20:00Z"
            },
            {
                "source_observation": {
                    "observed_at": "2026-10-08T04:20:00Z",
                    "method": "read-only exact-source comparison",
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    },
                    "reference_remote": {
                        "branch": "visual-baseline",
                        "head_sha": "cccccccccccccccccccccccccccccccccccccccc"
                    },
                    "branch_last_update_at": null
                },
                "source_evidence": {
                    "path": "fixtures/source-observation/history-2.json",
                    "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                },
                "current_ci_observation": null,
                "superseded_at": "2026-10-08T04:30:00Z"
            }
        ]);
    }
    let history_len = facts["source_observation_history"]
        .as_array()
        .expect("source history array")
        .len();
    if history_len >= 4 {
        return facts;
    }
    assert_eq!(history_len, 3, "fixture has three historical source rows");

    let previous_latest = facts["latest_source_observation"].clone();
    let previous_ci = facts["current_ci_observation"].clone();
    let previous_fetch = previous_latest["fetch_provenance"].clone();
    let previous_evidence = if previous_fetch.is_object() {
        serde_json::json!({
            "path": previous_fetch["evidence_path"].clone(),
            "sha256": previous_fetch["evidence_sha256"].clone()
        })
    } else {
        serde_json::json!({
            "path": "fixtures/source-observation/latest.json",
            "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
        })
    };
    facts["source_observation_history"]
        .as_array_mut()
        .expect("source history array")
        .push(serde_json::json!({
            "source_observation": previous_latest,
            "source_evidence": previous_evidence,
            "current_ci_observation": previous_ci,
            "superseded_at": "2026-10-08T15:11:33Z"
        }));
    facts["latest_source_observation"] = serde_json::json!({
        "observed_at": "2026-10-08T15:11:33Z",
        "method": "explicit HTTPS refs/heads fetch",
        "candidate_remote": {
            "branch": "termrock-implementation",
            "head_sha": "1ea1c17707f0a8f1639af506be179013d5e2d52a",
            "commit_committer_at": "2026-10-08T14:37:23Z"
        },
        "reference_remote": {
            "branch": "visual-baseline",
            "head_sha": "b682cb26d68b353aeeccf9e51653eddf097b39f5",
            "commit_committer_at": "2026-10-08T13:06:50Z"
        },
        "branch_last_update_at": null,
        "timestamp_provenance": {
            "candidate_committer_date_source": "candidate exact-head commit API response",
            "reference_committer_date_source": "reference exact-head commit API response",
            "branch_last_update_at_source": "Branch update time was not observed.",
            "committer_dates_are_branch_update_times": false
        },
        "fetch_provenance": {
            "evidence_path": "fixtures/source-observation/resume-ref-fetch-20261008-151133.json",
            "evidence_sha256": "367121e3eb643453d46d33b598388752c41b1c5fa23e9c099e751c280e0568aa",
            "evidence_kind": "coordinator external observation record",
            "transport": "HTTPS",
            "operation": "fetch",
            "refspecs": [
                "refs/heads/termrock-implementation",
                "refs/heads/visual-baseline"
            ],
            "session_id": 45546,
            "exit_code": 0,
            "completed_at": "2026-10-08T15:11:33Z",
            "raw_fetch_output_preserved": false,
            "post_fetch_ref_query": {
                "record_id": "91806b",
                "observed_at": null,
                "candidate_head_sha": "1ea1c17707f0a8f1639af506be179013d5e2d52a",
                "reference_head_sha": "b682cb26d68b353aeeccf9e51653eddf097b39f5"
            }
        }
    });
    facts["current_ci_observation"] = serde_json::Value::Null;
    facts["observed_at"] = serde_json::json!("2026-10-08T15:22:16Z");
    facts
}

fn write_json_facts(fixture: &StatusFixture, facts: &serde_json::Value) {
    fixture.write_facts(
        &serde_json::to_string_pretty(facts).expect("serialize source facts fixture"),
    );
}

fn write_pinned_json(
    fixture: &StatusFixture,
    name: &str,
    value: &serde_json::Value,
) -> serde_json::Value {
    let relative = format!("tools/visibility/evidence/{name}");
    let path = fixture.repo.root().join(&relative);
    fs::create_dir_all(path.parent().expect("evidence fixture parent"))
        .expect("create evidence fixture directory");
    let raw = serde_json::to_vec(value).expect("serialize evidence fixture");
    fs::write(path, &raw).expect("write evidence fixture");
    serde_json::json!({"path": relative, "sha256": sha256_hex(&raw)})
}

fn mutate_holla_receipt(
    fixture: &StatusFixture,
    mutate: impl FnOnce(&mut serde_json::Value),
) {
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    let relative_receipt = facts["execution_observations"]["holla_diagnostic"]["receipt"]
        ["path"]
        .as_str()
        .expect("Holla receipt fixture path");
    let receipt_path = fixture.repo.root().join(relative_receipt);
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).expect("read Holla receipt"))
            .expect("parse Holla receipt");
    mutate(&mut receipt);
    let raw = serde_json::to_vec(&receipt).expect("serialize mutated Holla receipt");
    fs::write(&receipt_path, &raw).expect("write mutated Holla receipt");
    let receipt_sha = sha256_hex(&raw);
    facts["execution_observations"]["holla_diagnostic"]["receipt"]["sha256"] =
        serde_json::json!(&receipt_sha);

    let seal_pin = facts["execution_observations"]["holla_diagnostic"]["seal"].clone();
    let seal_path = fixture
        .repo
        .root()
        .join(seal_pin["path"].as_str().expect("Holla seal fixture path"));
    let mut seal: serde_json::Value =
        serde_json::from_slice(&fs::read(&seal_path).expect("read Holla seal"))
            .expect("parse Holla seal");
    seal["actual_receipt"]["sha256"] = serde_json::json!(&receipt_sha);
    let seal_raw = serde_json::to_vec(&seal).expect("serialize rebound Holla seal");
    fs::write(&seal_path, &seal_raw).expect("write rebound Holla seal");
    let seal_sha = sha256_hex(&seal_raw);
    facts["execution_observations"]["holla_diagnostic"]["seal"]["sha256"] =
        serde_json::json!(&seal_sha);

    let review_pin = facts["execution_observations"]["holla_diagnostic"]["review"].clone();
    let review_path = fixture
        .repo
        .root()
        .join(review_pin["path"].as_str().expect("Holla review fixture path"));
    let mut review: serde_json::Value =
        serde_json::from_slice(&fs::read(&review_path).expect("read Holla review"))
            .expect("parse Holla review");
    review["subject"]["seal_sha256"] = serde_json::json!(&seal_sha);
    review["artifact_audit"]["actual_receipt_sha256"] = serde_json::json!(&receipt_sha);
    let review_raw = serde_json::to_vec(&review).expect("serialize rebound Holla review");
    fs::write(&review_path, &review_raw).expect("write rebound Holla review");
    facts["execution_observations"]["holla_diagnostic"]["review"]["sha256"] =
        serde_json::json!(sha256_hex(&review_raw));
    write_json_facts(fixture, &facts);
}

fn mutate_holla_pinned_document(
    fixture: &StatusFixture,
    document_key: &str,
    mutate: impl FnOnce(&mut serde_json::Value),
) {
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    let pin = facts["execution_observations"]["holla_diagnostic"][document_key].clone();
    let document_path = fixture
        .repo
        .root()
        .join(pin["path"].as_str().expect("Holla pinned document path"));
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&document_path).expect("read Holla pinned document"))
            .expect("parse Holla pinned document");
    mutate(&mut document);
    let raw = serde_json::to_vec(&document).expect("serialize mutated Holla document");
    fs::write(&document_path, &raw).expect("write mutated Holla document");
    facts["execution_observations"]["holla_diagnostic"][document_key]["sha256"] =
        serde_json::json!(sha256_hex(&raw));
    write_json_facts(fixture, &facts);
}

fn execution_fixture() -> StatusFixture {
    execution_fixture_with_holla_override(None)
}

fn execution_fixture_with_holla_override(
    status_override: Option<(&str, &str)>,
) -> StatusFixture {
    let fixture = StatusFixture::new();
    let candidate = "1ea1c17707f0a8f1639af506be179013d5e2d52a";
    let reference = "b682cb26d68b353aeeccf9e51653eddf097b39f5";

    let case_results = serde_json::json!([
        {"type": "test", "event": "ok", "name": "termrock-conformance::control_states$test_ok"},
        {"type": "test", "event": "failed", "name": "termrock-conformance::control_states$test_failed"}
    ]);
    let case_map = serde_json::json!([
        {"id": "BD-01", "cases": ["W01-02"], "legacy_test": "test_ok", "result": "ok"},
        {"id": "BD-21", "cases": ["W13-05"], "legacy_test": "test_failed", "result": "failed"}
    ]);
    let deferred_seal = serde_json::json!({
        "schema": "termrock-deferred-nextest-execution-v1",
        "kind": "actual-execution",
        "run_root": "/private/tmp/deferred-fixture",
        "source": {
            "commit": candidate,
            "tree": "4444444444444444444444444444444444444444",
            "registry_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "registry_rows": 2,
            "registry_case_references": 2,
            "source_inputs_rechecked_after_run": true
        },
        "execution": {
            "start_utc": "2026-10-08T19:47:52Z",
            "end_utc": "2026-10-08T19:54:20Z",
            "nextest_version": "0.9.146",
            "build_completed": true,
            "exit_code": 100,
            "exit_classification": "real assertion failure after test execution"
        },
        "results": {
            "selected_filter_names": 2,
            "test_started_events": 2,
            "terminal_test_events": 2,
            "passed": 1,
            "failed": 1,
            "registry_rows_mapped": 2,
            "registry_rows_missing": 0,
            "case_results_file": "case-results.json",
            "case_map_file": "case-map.json",
            "suite_event_counts_are_not_added_to_test_level_counts": true
        }
    });
    let deferred_seal_pin = write_pinned_json(&fixture, "deferred-seal.json", &deferred_seal);
    let case_results_pin = write_pinned_json(&fixture, "case-results.json", &case_results);
    let case_map_pin = write_pinned_json(&fixture, "case-map.json", &case_map);
    let deferred_review = serde_json::json!({
        "schema": "termrock-independent-deferred-nextest-receipt-review-v1",
        "reviewer": "independent-reviewer",
        "receipt": {
            "sha256": deferred_seal_pin["sha256"],
            "registry_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "registry_rows": 2
        },
        "findings": {
            "test_events": {
                "started": 2,
                "terminal": 2,
                "ok": 1,
                "failed": 1,
                "case_map_result_vs_terminal_event_mismatches": 0
            },
            "failure": {"test": "test_failed"},
            "source_integrity": {
                "commit_tree_registry_pins_match_receipt_and_source_hash_file": true,
                "execution_receipt_reports_inputs_rechecked_after_run": true
            }
        }
    });
    let deferred_review_pin = write_pinned_json(&fixture, "deferred-review.json", &deferred_review);

    let source_pair = serde_json::json!([
        {"role": "candidate", "source_commit": candidate,
         "build": {"target_triple": "aarch64-apple-darwin"},
         "executable": {
             "expected_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
             "actual_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
         },
         "builder_receipt_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
         "builder_receipt": {
             "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
             "executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
             "source_commit": candidate,
             "target_triple": "aarch64-apple-darwin"
         }},
        {"role": "reference", "source_commit": reference,
         "build": {"target_triple": "aarch64-apple-darwin"},
         "executable": {
             "expected_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
             "actual_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
         },
         "builder_receipt_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
         "builder_receipt": {
             "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
             "executable_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
             "source_commit": reference,
             "target_triple": "aarch64-apple-darwin"
         }}
    ]);
    let checkpoints = ["00-boot", "01-help", "02-finder", "03-help-again"];
    let mut checks = Vec::new();
    let mut artifacts = Vec::new();
    for role in ["candidate", "reference"] {
        for checkpoint in checkpoints {
            let check = |dimension: &str, status: &str, id: &str, evidence: Vec<&str>| {
                serde_json::json!({
                    "id": id,
                    "subject_role": role,
                    "case_id": "HELP-HOLLA-004",
                    "checkpoint_id": checkpoint,
                    "dimension": dimension,
                    "status": status,
                    "evidence": evidence
                })
            };
            for (dimension, status) in [
                ("build", "PASS"),
                ("launch", "PASS"),
                ("first_frame", "PASS"),
                ("visual", "BLOCKED"),
                ("exit", "NOT_APPLICABLE"),
                ("restoration", "NOT_APPLICABLE"),
            ] {
                let id = format!("{role}:HELP-HOLLA-004:{checkpoint}:{dimension}");
                let evidence = if dimension == "first_frame" {
                    vec!["120x40"]
                } else {
                    vec![]
                };
                checks.push(check(dimension, status, &id, evidence));
            }
            let aggregate_id = format!("{role}:HELP-HOLLA-004:{checkpoint}:interaction");
            checks.push(check("interaction", "PASS", &aggregate_id, vec![]));
            let assertions: &[&str] = match checkpoint {
                "00-boot" => &[
                    "boot.disk_cursor", "boot.finder_footer", "boot.placeholder", "boot.world",
                ],
                "01-help" => &[
                    "help.finder_footer_absent", "help.finder_row", "help.footer",
                    "help.quit_row", "help.title_here",
                ],
                "02-finder" => &["finder.placeholder_restored", "finder.quit_row_absent"],
                "03-help-again" => &["reopen.title_here"],
                _ => panic!("unexpected fixture checkpoint"),
            };
            for assertion in assertions {
                let id = format!("HELP-HOLLA-004:{checkpoint}:{assertion}");
                let status = status_override
                    .filter(|(target_id, _)| *target_id == id.as_str())
                    .map(|(_, replacement)| replacement)
                    .unwrap_or("PASS");
                checks.push(check("interaction", status, &id, vec![]));
            }
            let formats = [
                "ansi", "ascii", "ascii_loss_json", "frame_json", "html", "manifest_json",
                "observations_json", "png", "png_fidelity_json", "txt",
            ];
            for format in formats {
                artifacts.push(serde_json::json!({
                    "subject_role": role,
                    "case_id": "HELP-HOLLA-004",
                    "checkpoint_id": checkpoint,
                    "format": format,
                    "path": format!("artifacts/{role}/{checkpoint}/{format}"),
                    "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "bytes": 1
                }));
            }
        }
    }
    let mut status_counts = serde_json::Map::new();
    for check in &checks {
        let status = check["status"].as_str().expect("check status").to_owned();
        let count = status_counts.entry(status).or_insert(serde_json::json!(0));
        *count = serde_json::json!(count.as_u64().expect("numeric check count") + 1);
    }
    let status_counts = serde_json::Value::Object(status_counts);
    let interaction_pass_count = checks
        .iter()
        .filter(|check| {
            check["dimension"] == "interaction" && check["status"] == "PASS"
        })
        .count();
    let failed_interactions = checks
        .iter()
        .filter(|check| {
            check["dimension"] == "interaction" && check["status"] == "FAIL"
        })
        .count();
    let not_run_interactions = checks
        .iter()
        .filter(|check| {
            check["dimension"] == "interaction" && check["status"] == "NOT_RUN"
        })
        .count();
    let qualification = if failed_interactions > 0 { "FAIL" } else { "BLOCKED" };
    let assertion_outcome = if failed_interactions > 0 {
        "A recorded interaction FAIL and visual BLOCKED result caused the selected test to fail."
    } else {
        "The visual BLOCKED result caused the selected test assertion to fail."
    };
    let holla_receipt = serde_json::json!({
        "schema": "termrock-spec/parity-run-receipt-v3",
        "run_id": "synthetic-holla-run-01",
        "source_pair": source_pair,
        "suite": {
            "revision": "termrock-e2e-2026-10-08.1",
            "digest": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "compiled_digest": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "platform": "macos"
        },
        "trust": {"status": "accepted"},
        "environment": {"common": {"COLORTERM": "truecolor"}},
        "checks": checks,
        "artifacts": artifacts
    });
    let holla_receipt_pin = write_pinned_json(&fixture, "holla-receipt.json", &holla_receipt);
    let holla_root = "/private/tmp/holla-diagnostic-test-run-01";
    let nextest_run_id = "be22e745-da96-47ba-880f-eb00605352a6";
    let holla_seal = serde_json::json!({
        "record_type": "termrock_holla_actual_paired_journey_seal_v1",
        "state": "sealed",
        "executor": {"root": holla_root},
        "process": {"exit_code": 100, "timed_out": false, "nextest_run_id": nextest_run_id},
        "selection": {"selected": 1, "passed": 0, "failed": 1, "skipped": 1},
        "actual_receipt": {
            "sha256": holla_receipt_pin["sha256"],
            "run_id": "synthetic-holla-run-01",
            "check_count": 80,
            "status_counts": status_counts.clone(),
            "qualification": qualification
        }
    });
    let holla_seal_pin = write_pinned_json(&fixture, "holla-seal.json", &holla_seal);
    let holla_review = serde_json::json!({
        "record_type": "independent_read_only_actual_holla_diagnostic_journey_review",
        "verdict": "REVIEWED_DIAGNOSTIC_EVIDENCE_ONLY",
        "subject": {
            "seal_sha256": holla_seal_pin["sha256"],
            "run_id": "holla-diagnostic-test-run-01",
            "nextest_run_id": nextest_run_id
        },
        "source_pair_and_binaries": {
            "candidate": {
                "source_commit": candidate,
                "executable_sha256_expected_and_rehashed": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "builder_receipt_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            },
            "reference": {
                "source_commit": reference,
                "executable_sha256_expected_and_rehashed": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "builder_receipt_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            }
        },
        "suite_identity": {
            "revision": "termrock-e2e-2026-10-08.1",
            "digest": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        },
        "raw_execution": {
            "exit_code": 100,
            "assertion_outcome": assertion_outcome
        },
        "checkpoint_and_assertion_results": {
            "case_id": "HELP-HOLLA-004",
            "checks_total": 80,
            "status_counts": status_counts.clone(),
            "pass_dimensions": {
                "build_hash_checks": 8,
                "launch_checks": 8,
                "first_frame": 8,
                "interaction": interaction_pass_count
            },
            "blocked_dimensions": {"visual": 8},
            "not_applicable_dimensions": {"exit": 8, "restoration": 8},
            "failed_dimensions": if failed_interactions > 0 {
                serde_json::json!({"interaction": failed_interactions})
            } else {
                serde_json::json!({})
            },
            "not_run_dimensions": if not_run_interactions > 0 {
                serde_json::json!({"interaction": not_run_interactions})
            } else {
                serde_json::json!({})
            },
            "interaction_breakdown": "Per subject, 12 assertion IDs span the four checkpoints."
        },
        "artifact_audit": {
            "actual_receipt_path": holla_receipt_pin["path"],
            "actual_receipt_sha256": holla_receipt_pin["sha256"],
            "artifact_observations": 80,
            "role_checkpoint_pairs": 8,
            "unique_formats_per_pair": 10,
            "formats": [
                "ansi", "ascii", "ascii_loss_json", "frame_json", "html", "manifest_json",
                "observations_json", "png", "png_fidelity_json", "txt"
            ],
            "hash_and_size_verification": "Independently recomputed SHA-256 and byte count for all 80 artifact paths; all match."
        }
    });
    let holla_review_pin = write_pinned_json(&fixture, "holla-review.json", &holla_review);
    let build_review = serde_json::json!({
        "record_type": "independent_read_only_actual_holla_build_evidence_review",
        "verdict": "REVIEWED_DIAGNOSTIC_BUILD_EVIDENCE_ONLY",
        "observations": {
            "attempt": {
                "run_id": "synthetic-holla-run-01",
                "process_exit_code": 0,
                "timed_out": false
            },
            "nextest": {"passed": 1, "failed": 0},
            "builds": [
                {"role": "candidate", "source_commit": candidate,
                 "target_triple": "aarch64-apple-darwin", "exit_code": 0,
                 "executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                 "builder_receipt_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"},
                {"role": "reference", "source_commit": reference,
                 "target_triple": "aarch64-apple-darwin", "exit_code": 0,
                 "executable_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                 "builder_receipt_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}
            ]
        }
    });
    let build_review_pin = write_pinned_json(&fixture, "holla-build-review.json", &build_review);

    let mut facts = with_four_predecessors(facts_with_current_ci_observation());
    facts["execution_observations"] = serde_json::json!({
        "schema": "termrock-status-execution-observations-v1",
        "provisional_local_evidence": true,
        "source_pair": {
            "candidate_commit": candidate,
            "reference_commit": reference
        },
        "deferred_nextest": {
            "seal": deferred_seal_pin,
            "case_results": case_results_pin,
            "case_map": case_map_pin,
            "review": deferred_review_pin
        },
        "holla_diagnostic": {
            "receipt": holla_receipt_pin,
            "seal": holla_seal_pin,
            "review": holla_review_pin,
            "build_review": build_review_pin
        }
    });
    write_json_facts(&fixture, &facts);
    fixture
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
    assert!(report.contains("required shared-case and per-component checkpoint sets are unknown"));
    assert!(!report.contains("| Required set | 0 |"));
    assert!(report.contains("Candidate observed commit"));
    assert!(report.contains("Reference observed commit"));
    assert!(report.contains("Report commit | Set after this report is committed"));
    assert!(!report.contains("source S"));
    assert!(!report.contains("commit P"));
    assert!(report.contains("A run receipt records execution results"));
    assert!(report.contains("NOT_RUN is a result status, not a run-receipt status"));
    assert!(report.contains("Product requirements remain in [CHECKLIST.md]"));
    assert!(!report.contains("is generated from accepted records"));
}

#[test]
fn bounded_execution_receipts_render_partial_results_without_ready_claims() {
    let fixture = execution_fixture();
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("Visibility / Complete | NOT_READY"));
    assert!(report.contains("Refactor / Ready | NOT_READY"));
    assert!(report.contains("Evidence freshness | PARTIAL"));
    assert!(report.contains("1 passed; 1 failed of 2 terminal tests"));
    assert!(report.contains("BD-21 / W13-05 (test_failed)"));
    assert!(report.contains("Holla | BLOCKED (4) | BLOCKED (4) | PASS (16) | PASS (16)"));
    assert!(report.contains("80 checks: 56 PASS, 8 BLOCKED, 16 NOT_APPLICABLE"));
    assert!(report.contains("single 120x40 truecolor case only"));
    assert!(report.contains("No API or ownership result is recorded"));
    assert!(report.contains("proposed R5 denominator is not accepted"));
    assert!(report.contains(
        "| RUN-03 | cargo run --release --bin holla | Reference: NOT_RUN; Candidate: NOT_RUN"
    ));
    assert!(!report.contains("| RUN-03 | cargo run --release --bin holla | PASS"));
}

#[test]
fn holla_valid_failure_and_not_run_rows_are_rendered_without_becoming_passes() {
    // The overridden assertion ID appears once for each subject role.
    for (status, expected_cell, expected_count) in [
        (
            "FAIL",
            "PASS (15); FAIL (1)",
            "80 checks: 54 PASS, 2 FAIL, 8 BLOCKED, 16 NOT_APPLICABLE",
        ),
        (
            "NOT_RUN",
            "PASS (15); NOT_RUN (1)",
            "80 checks: 54 PASS, 8 BLOCKED, 2 NOT_RUN, 16 NOT_APPLICABLE",
        ),
    ] {
        let fixture = execution_fixture_with_holla_override(Some((
            "HELP-HOLLA-004:00-boot:boot.disk_cursor",
            status,
        )));
        let output = fixture.run(&[]);
        let report = output_text(&output);

        assert_eq!(output.exit_code, Some(0));
        assert!(report.contains(expected_count));
        assert!(report.contains(&format!(
            "| Holla | BLOCKED (4) | BLOCKED (4) | {} | {} |",
            expected_cell,
            expected_cell,
        )));
        assert!(report.contains("Visibility / Complete | NOT_READY"));
        assert!(report.contains("Refactor / Ready | NOT_READY"));
        assert!(report.contains("No API or ownership result is recorded"));
    }
}

#[test]
fn execution_receipts_reject_source_pair_mismatch_and_changed_bytes() {
    let fixture = execution_fixture();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    facts["execution_observations"]["source_pair"]["candidate_commit"] =
        serde_json::json!("0000000000000000000000000000000000000000");
    write_json_facts(&fixture, &facts);
    let mismatched_pair = fixture.run(&[]);
    assert_eq!(mismatched_pair.exit_code, Some(2));
    assert!(error_text(&mismatched_pair)
        .contains("execution observations candidate commit does not match the selected source"));
    assert!(mismatched_pair.stdout.is_empty());

    let fixture = execution_fixture();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    let relative_receipt = facts["execution_observations"]["holla_diagnostic"]["receipt"]
        ["path"]
        .as_str()
        .expect("Holla receipt fixture path");
    let receipt_path = fixture.repo.root().join(relative_receipt);
    let mut receipt_bytes = fs::read(&receipt_path).expect("read Holla receipt fixture");
    receipt_bytes.push(b' ');
    fs::write(receipt_path, receipt_bytes).expect("change Holla receipt fixture bytes");
    let changed_receipt = fixture.run(&[]);
    assert_eq!(changed_receipt.exit_code, Some(2));
    assert!(error_text(&changed_receipt)
        .contains("Holla paired receipt bytes do not match the recorded SHA-256"));
    assert!(changed_receipt.stdout.is_empty());
}

#[test]
fn holla_executable_and_builder_receipt_hashes_are_cross_bound() {
    let mutations = [
        (
            "nested_builder_source_commit_mismatch",
            "Holla candidate nested builder receipt source commit does not match the selected source",
        ),
        (
            "nested_builder_target_triple_mismatch",
            "Holla candidate nested builder receipt target triple does not match the build",
        ),
        (
            "receipt_expected_actual_mismatch",
            "Holla candidate receipt executable SHA-256 values do not match",
        ),
        (
            "receipt_executable_sha256",
            "Holla diagnostic review candidate executable SHA-256 does not match the receipt",
        ),
        (
            "receipt_builder_receipt_sha256",
            "Holla diagnostic review candidate builder receipt SHA-256 does not match the receipt",
        ),
        (
            "review_executable_sha256",
            "Holla diagnostic review candidate executable SHA-256 does not match the receipt",
        ),
        (
            "review_builder_receipt_sha256",
            "Holla diagnostic review candidate builder receipt SHA-256 does not match the receipt",
        ),
        (
            "build_review_executable_sha256",
            "Holla candidate build review executable SHA-256 does not match the receipt",
        ),
        (
            "build_review_builder_receipt_sha256",
            "Holla candidate build review builder receipt SHA-256 does not match the receipt",
        ),
    ];

    for (mutation, expected_error) in mutations {
        let fixture = execution_fixture();
        match mutation {
            "nested_builder_source_commit_mismatch" => mutate_holla_receipt(&fixture, |receipt| {
                receipt["source_pair"][0]["builder_receipt"]["source_commit"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
            }),
            "nested_builder_target_triple_mismatch" => mutate_holla_receipt(&fixture, |receipt| {
                receipt["source_pair"][0]["builder_receipt"]["target_triple"] =
                    serde_json::json!("x86_64-unknown-linux-gnu");
            }),
            "receipt_expected_actual_mismatch" => mutate_holla_receipt(&fixture, |receipt| {
                receipt["source_pair"][0]["executable"]["expected_sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
            }),
            "receipt_executable_sha256" => mutate_holla_receipt(&fixture, |receipt| {
                let candidate = &mut receipt["source_pair"][0];
                candidate["executable"]["expected_sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                candidate["executable"]["actual_sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                candidate["builder_receipt"]["executable_sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
            }),
            "receipt_builder_receipt_sha256" => mutate_holla_receipt(&fixture, |receipt| {
                receipt["source_pair"][0]["builder_receipt_sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                receipt["source_pair"][0]["builder_receipt"]["sha256"] =
                    serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
            }),
            "review_executable_sha256" => {
                mutate_holla_pinned_document(&fixture, "review", |review| {
                    review["source_pair_and_binaries"]["candidate"]
                        ["executable_sha256_expected_and_rehashed"] =
                        serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                })
            }
            "review_builder_receipt_sha256" => {
                mutate_holla_pinned_document(&fixture, "review", |review| {
                    review["source_pair_and_binaries"]["candidate"]
                        ["builder_receipt_sha256"] = serde_json::json!(
                        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                    );
                })
            }
            "build_review_executable_sha256" => {
                mutate_holla_pinned_document(&fixture, "build_review", |build_review| {
                    build_review["observations"]["builds"][0]["executable_sha256"] =
                        serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                })
            }
            "build_review_builder_receipt_sha256" => {
                mutate_holla_pinned_document(&fixture, "build_review", |build_review| {
                    build_review["observations"]["builds"][0]["builder_receipt_sha256"] =
                        serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                })
            }
            _ => panic!("unknown Holla hash mutation `{mutation}`"),
        }

        let output = fixture.run(&[]);
        assert_eq!(output.exit_code, Some(2), "mutation `{mutation}`");
        assert!(error_text(&output).contains(expected_error), "mutation `{mutation}`");
        assert!(output.stdout.is_empty(), "mutation `{mutation}`");
    }
}

#[test]
fn holla_receipt_rejects_unexpected_or_missing_check_and_artifact_rows() {
    for (mutation, expected_error) in [
        (
            "extra_fail_check",
            "Holla check rows do not match the exact reviewed ID and dimension set",
        ),
        (
            "missing_check",
            "Holla check rows do not match the exact reviewed ID and dimension set",
        ),
        (
            "missing_artifact",
            "Holla artifact rows do not match the exact role, checkpoint, and format matrix",
        ),
        (
            "unexpected_artifact_format",
            "Holla artifact rows do not match the exact role, checkpoint, and format matrix",
        ),
    ] {
        let fixture = execution_fixture();
        mutate_holla_receipt(&fixture, |receipt| match mutation {
            "extra_fail_check" => {
                let mut row = receipt["checks"][0].clone();
                row["id"] = serde_json::json!(
                    "candidate:HELP-HOLLA-004:00-boot:unexpected"
                );
                row["status"] = serde_json::json!("FAIL");
                receipt["checks"].as_array_mut().expect("checks array").push(row);
            }
            "missing_check" => {
                receipt["checks"]
                    .as_array_mut()
                    .expect("checks array")
                    .remove(0);
            }
            "missing_artifact" => {
                receipt["artifacts"]
                    .as_array_mut()
                    .expect("artifacts array")
                    .remove(0);
            }
            "unexpected_artifact_format" => {
                let mut row = receipt["artifacts"][0].clone();
                row["format"] = serde_json::json!("unexpected-format");
                row["path"] = serde_json::json!("artifacts/unexpected-format");
                receipt["artifacts"]
                    .as_array_mut()
                    .expect("artifacts array")
                    .push(row);
            }
            _ => panic!("unknown Holla receipt mutation"),
        });
        let output = fixture.run(&[]);

        assert_eq!(output.exit_code, Some(2), "mutation `{mutation}`");
        assert!(error_text(&output).contains(expected_error), "mutation `{mutation}`");
        assert!(output.stdout.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn execution_evidence_rejects_tmp_traversal_and_symlinked_parent() {
    let fixture = execution_fixture();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    facts["execution_observations"]["holla_diagnostic"]["seal"]["path"] =
        serde_json::json!("../../etc/passwd");
    write_json_facts(&fixture, &facts);
    let traversal = fixture.run(&[]);
    assert_eq!(traversal.exit_code, Some(2));
    assert!(error_text(&traversal)
        .contains("evidence path must stay below its allowed root"));
    assert!(traversal.stdout.is_empty());

    let fixture = execution_fixture();
    let evidence_dir = fixture.repo.root().join("tools/visibility/evidence");
    let outside_dir = fixture.repo.root().join("evidence-target");
    fs::create_dir_all(&outside_dir).expect("create symlink target directory");
    fs::copy(
        evidence_dir.join("holla-seal.json"),
        outside_dir.join("holla-seal.json"),
    )
    .expect("copy target evidence");
    symlink(&outside_dir, evidence_dir.join("linked-directory"))
        .expect("create symlinked evidence parent");
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    facts["execution_observations"]["holla_diagnostic"]["seal"]["path"] =
        serde_json::json!("tools/visibility/evidence/linked-directory/holla-seal.json");
    write_json_facts(&fixture, &facts);
    let symlinked_parent = fixture.run(&[]);
    assert_eq!(symlinked_parent.exit_code, Some(2));
    assert!(error_text(&symlinked_parent)
        .contains("Holla journey execution seal cannot be opened without following symlinks"));
    assert!(symlinked_parent.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn execution_evidence_fifo_is_rejected_without_blocking_or_report() {
    let fixture = execution_fixture();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    let relative_receipt = facts["execution_observations"]["holla_diagnostic"]["receipt"]
        ["path"]
        .as_str()
        .expect("Holla receipt fixture path");
    let receipt_path = fixture.repo.root().join(relative_receipt);
    fs::remove_file(&receipt_path).expect("remove regular receipt fixture");
    let fifo_path = CString::new(receipt_path.as_os_str().as_bytes())
        .expect("temporary receipt path has no NUL byte");
    let mkfifo_result = unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) };
    assert_eq!(
        mkfifo_result,
        0,
        "create FIFO evidence fixture: {}",
        std::io::Error::last_os_error()
    );

    let args = vec!["--write".to_string()];
    let output = run_cli_with_timeout(
        &fixture.script,
        &args,
        fixture.repo.root(),
        &[],
        None,
        Duration::from_secs(2),
    )
    .expect("run status CLI with bounded timeout");

    assert!(!output.timed_out, "FIFO evidence open must not block");
    assert_eq!(output.exit_code, Some(2));
    assert!(output.stdout.is_empty(), "rejected evidence emits no report");
    assert!(!fixture.repo.root().join("STATUS.md").exists());
    assert!(error_text(&output).contains("Holla paired receipt must be a regular file"));
}

#[test]
fn execution_receipt_json_rejects_duplicate_keys() {
    let fixture = execution_fixture();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(&facts_path).expect("read source facts"))
            .expect("parse source facts");
    let relative_receipt = facts["execution_observations"]["holla_diagnostic"]["receipt"]
        ["path"]
        .as_str()
        .expect("Holla receipt fixture path");
    let receipt_path = fixture.repo.root().join(relative_receipt);
    let duplicate = br#"{"schema":"termrock-spec/parity-run-receipt-v3","schema":"termrock-spec/parity-run-receipt-v3"}"#;
    fs::write(&receipt_path, duplicate).expect("write duplicate-key receipt");
    facts["execution_observations"]["holla_diagnostic"]["receipt"]["sha256"] =
        serde_json::json!(sha256_hex(duplicate));
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("Holla paired receipt is not strict JSON"));
    assert!(output.stdout.is_empty());
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
    let latest_candidate_sha = facts["latest_source_observation"]["candidate_remote"]["head_sha"]
        .as_str()
        .expect("latest candidate tip in source facts");
    let latest_reference_sha = facts["latest_source_observation"]["reference_remote"]["head_sha"]
        .as_str()
        .expect("latest reference tip in source facts");
    let history = facts["source_observation_history"]
        .as_array()
        .expect("superseded source observation history");

    let output = fixture.run(&[]);
    let repeated = fixture.run(&[]);
    let report = output_text(&output);
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(repeated.exit_code, Some(0));
    assert_eq!(output.stdout, repeated.stdout);
    assert!(report.contains(candidate_sha));
    assert!(report.contains(reference_sha));
    assert!(report.contains(latest_candidate_sha));
    assert!(report.contains(latest_reference_sha));
    assert_eq!(candidate_sha, "85b51da2e9832dba642abf7d64d032f848cade0e");
    assert_eq!(reference_sha, "5f6e52f31861f9f4281f1db264ab012457b9bc2e");
    assert!(facts["latest_source_observation"]["branch_last_update_at"].is_null());
    assert_eq!(
        facts["latest_source_observation"]["timestamp_provenance"]
            ["committer_dates_are_branch_update_times"],
        false
    );
    assert_eq!(history.len(), 5);
    assert_eq!(
        history[0]["source_observation"]["candidate_remote"]["head_sha"],
        "931bbba00bad43de79731912f48e29fe88751cb8"
    );
    assert_eq!(
        history[1]["source_observation"]["candidate_remote"]["head_sha"],
        "bebb60a7948f9ee190483c48545d6bc35c5437df"
    );
    assert_eq!(
        history[2]["source_observation"]["candidate_remote"]["head_sha"],
        "e22d54708fe58670d92c2dac7c1ace6c97fac164"
    );
    assert_eq!(
        history[3]["source_observation"]["candidate_remote"]["head_sha"],
        "cee7e2e307514e49a70d8b8fcb028923ccdc7322"
    );
    assert_eq!(
        history[4]["source_observation"]["candidate_remote"]["head_sha"],
        "1ea1c17707f0a8f1639af506be179013d5e2d52a"
    );
    assert!(history[0]["current_ci_observation"].is_object());
    assert!(history[1]["current_ci_observation"].is_object());
    assert!(history[2]["current_ci_observation"].is_null());
    assert!(history[3]["current_ci_observation"].is_object());
    assert!(history[4]["current_ci_observation"].is_object());
    assert!(history[2]["source_observation"]["branch_last_update_at"].is_null());
    assert_eq!(latest_candidate_sha, "1d797d41c8141fcbdc3f69d7f11eb8875ab54712");
    assert_eq!(latest_reference_sha, "b274dd57f4dd078ade6e424d546d83efbd2e8526");
    assert_eq!(history[4]["superseded_at"], "2026-10-08T23:25:02Z");
    assert!(report.contains("2026-10-08T23:14:25Z"));
    assert!(report.contains("2026-10-08T16:13:33Z"));
    assert!(report.contains(
        "read-only GitHub branch API observation; local fetched refs verified afterward"
    ));
    assert!(report.contains("Branch last-update time unknown"));
    let current_ci = facts["latest_ci_snapshot"]
        .as_object()
        .expect("latest exact-source CI capture");
    assert_eq!(current_ci["candidate_run"]["run_id"], "37858314774");
    assert_eq!(
        current_ci["source_pair"]["candidate_commit"],
        latest_candidate_sha
    );
    assert_eq!(
        current_ci["source_pair"]["reference_commit"],
        latest_reference_sha
    );
    assert_eq!(current_ci["candidate_run"]["job_count"], 0);
    assert!(current_ci["artifact_count"].is_null());
    assert_eq!(current_ci["artifact_query"], "NOT_QUERIED");
    assert_eq!(current_ci["failure_cause"], "UNKNOWN_NOT_CAPTURED");
    assert_eq!(current_ci["product_execution"], "NOT_RUN");
    assert_eq!(current_ci["reference_query"]["query_total_count"], 0);
    assert_eq!(current_ci["candidate_dco"]["check_run_id"], "113587796998");
    assert!(current_ci["reference_dco"].is_null());
    let latest_ci_section = report
        .split("## Latest candidate-source CI snapshot")
        .nth(1)
        .expect("latest CI report section")
        .split("\n## Report publication observations")
        .next()
        .expect("end of latest CI report section");
    assert!(latest_ci_section.contains("37858314774"));
    assert!(latest_ci_section.contains("artifact count UNKNOWN (not queried)"));
    assert!(latest_ci_section.contains("Failure cause UNKNOWN_NOT_CAPTURED"));
    assert!(latest_ci_section.contains("0 matching runs"));
    assert!(latest_ci_section.contains("113587796998"));
    assert!(latest_ci_section.contains("Reference DCO check | NOT_CAPTURED"));
    let provider_annotation = facts["current_status_observations"]["workflow_run_observations"][0]
        ["provider_annotation"]
        .as_str()
        .expect("captured current workflow failure annotation");
    assert!(latest_ci_section.contains(provider_annotation));
    assert!(latest_ci_section.contains("workflow source 537470 bytes"));
    assert_eq!(
        facts["latest_local_checkout_observation"]["head_sha"],
        "766ae1e925e32b4b28aa9100bb3fde5279aa223b"
    );
    assert!(
        report.contains("termrock-implementation at `766ae1e925e32b4b28aa9100bb3fde5279aa223b`")
            && report.contains("| Local publication | NOT_RECORDED.")
            && report.contains("f61abd3dfba1b4f867a539ab18ed7e4760b24a18"),
        "latest and historical local checkout identities must remain distinct"
    );
    assert!(report.contains("## Previous source observations"));
    assert!(report.contains("No CI/DCO capture recorded; source-only observation."));
    assert!(report.contains("37762481719"));
    assert!(report.contains("113262062732"));
    assert!(report.contains("product execution NOT_RUN"));
    assert!(report.contains("## Report publication observations"));
    assert!(report.contains("37862074205"));
    assert!(report.contains("37864720297"));
    assert!(report.lines().any(|line| {
        line.contains("169380c7d6a4cff9f1c43ecef592abc715e6df6a")
            && line.contains("[37862074205]")
    }));
    assert!(report.lines().any(|line| {
        line.contains("b9e34b13ef47401865f1511b9babaab6e020eedc")
            && line.contains("[37864720297]")
    }));
    assert!(report.contains("UNKNOWN_NOT_QUERIED"));
    assert!(report.contains("UNKNOWN_NOT_CAPTURED"));
    assert!(report.contains("## Historical bounded execution observations"));
    assert!(report.lines().any(|line| {
        line.contains("1ea1c17707f0a8f1639af506be179013d5e2d52a")
            && line.contains("b682cb26d68b353aeeccf9e51653eddf097b39f5")
    }));
    assert!(report.contains("22 passed; 1 failed of 23 terminal tests"));
    assert!(report.contains("80 checks: 56 PASS, 8 BLOCKED, 16 NOT_APPLICABLE"));
    assert!(report.contains("Historical local E2E package"));
    assert!(report.contains("## Local package and reporter controls"));
    assert!(report.contains("f307376a1e1d3dca04642875fdf98e9b43cdee47e6988ed0952c30c62c447361"));
    assert!(report.contains("a914f8e34280f55d6a868bfee8777f771fb0354e96e5a2291a91b86c65a62ea0"));
    assert!(report.contains("1 non-product registry/precondition test passed of 1 selected"));
    assert!(report.contains("35 unique test identities reconciled as 26 + 7 + 1 + 1"));
    assert!(report.contains("this is not one full green run"));
    assert!(report.contains("The f307 Holla product result remains NOT_RUN."));
    assert!(report.contains("| Visibility / Complete | NOT_RUN |"));
    assert!(report.contains("| Refactor / Ready | NOT_READY |"));
}

#[test]
fn rejects_misbound_latest_ci_publication_and_historical_execution() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let original: serde_json::Value = serde_json::from_slice(
        &fs::read(&facts_path).expect("read source facts for source-binding probes"),
    )
    .expect("parse copied source facts");

    let mut misbound_ci = original.clone();
    misbound_ci["latest_ci_snapshot"]["source_pair"]["candidate_commit"] =
        serde_json::json!("1111111111111111111111111111111111111111");
    fixture.write_facts(&misbound_ci.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "latest CI source pair does not match the current source observation"
    ));
    assert!(output.stdout.is_empty());

    let mut misbound_publication = original.clone();
    misbound_publication["publication_ci_observations"][1]["source_commit"] =
        serde_json::json!("169380c7d6a4cff9f1c43ecef592abc715e6df6a");
    fixture.write_facts(&misbound_publication.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("latest publication source commit changed"));
    assert!(output.stdout.is_empty());

    let mut promoted_historical = original;
    promoted_historical["historical_execution_observations"][0]["source_pair"]
        ["candidate_commit"] =
        serde_json::json!("1d797d41c8141fcbdc3f69d7f11eb8875ab54712");
    fixture.write_facts(&promoted_historical.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "must match exactly one superseded source observation"
    ));
    assert!(output.stdout.is_empty());
}

#[test]
fn local_package_and_report_control_metadata_rejects_unbound_inputs() {
    for mutation in [
        "local_commit",
        "local_tree",
        "package_digest",
        "contract_result",
        "contract_count",
        "report_control_count",
        "report_review_pin",
    ] {
        let fixture = StatusFixture::from_checked_in_records();
        let facts_path = fixture
            .repo
            .root()
            .join("tools/visibility/source-facts.json");
        let mut facts: serde_json::Value = serde_json::from_slice(
            &fs::read(&facts_path).expect("read source facts for metadata mutation"),
        )
        .expect("parse source facts for metadata mutation");
        match mutation {
            "local_commit" => {
                facts["local_e2e_suite_observation"]["source_commit"] =
                    serde_json::json!("0000000000000000000000000000000000000000");
            }
            "local_tree" => {
                facts["local_e2e_suite_observation"]["source_tree_sha"] =
                    serde_json::json!("0000000000000000000000000000000000000000");
            }
            "package_digest" => {
                facts["local_e2e_suite_observation"]["package_sha256"] =
                    serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
            }
            "contract_result" => {
                facts["local_e2e_suite_observation"]["contract_gate"]["result"] =
                    serde_json::json!("NOT_RUN");
            }
            "contract_count" => {
                let receipt_path = facts["local_e2e_suite_observation"]["contract_gate"]
                    ["receipt"]["path"]
                    .as_str()
                    .expect("f307 receipt path");
                let receipt_bytes = fs::read(fixture.repo.root().join(receipt_path))
                    .expect("read copied f307 receipt");
                let mut receipt: serde_json::Value =
                    serde_json::from_slice(&receipt_bytes).expect("parse f307 receipt");
                receipt["test_results"][2]["counts"]["passed"] = serde_json::json!(0);
                let changed_bytes = serde_json::to_vec(&receipt)
                    .expect("serialize altered f307 receipt");
                let local_receipt = fixture
                    .repo
                    .root()
                    .join("tools/visibility/altered-f307-contract-receipt.json");
                fs::write(&local_receipt, &changed_bytes)
                    .expect("write altered f307 receipt in disposable repository");
                facts["local_e2e_suite_observation"]["contract_gate"]["receipt"]["path"] =
                    serde_json::json!("tools/visibility/altered-f307-contract-receipt.json");
                facts["local_e2e_suite_observation"]["contract_gate"]["receipt"]["sha256"] =
                    serde_json::json!(sha256_hex(&changed_bytes));
            }
            "report_control_count" => {
                facts["status_report_control_observation"]["unique_test_identities"] =
                    serde_json::json!(34);
            }
            "report_review_pin" => {
                facts["status_report_control_observation"]["independent_review"]["sha256"] =
                    serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
            }
            _ => unreachable!("listed metadata mutation"),
        }
        write_json_facts(&fixture, &facts);
        let output = fixture.run(&[]);
        assert_eq!(output.exit_code, Some(2), "mutation: {mutation}");
        assert!(output.stdout.is_empty(), "mutation: {mutation}");
    }
}

#[test]
fn status_renders_previewed_v1_and_v2_identically_and_rejects_invalid_v2() {
    let fixture = StatusFixture::from_checked_in_records();
    let tasks_path = fixture
        .repo
        .root()
        .join("docs/implementation/visibility/tasks.json");
    let queue_view_path = fixture.repo.root().join("WORK_QUEUE.md");
    let v1_tasks = fs::read(&tasks_path).expect("read copied schema-v1 tasks");
    let v1_queue_view = fs::read(&queue_view_path).expect("read copied v1 queue view");

    let v1_output = fixture.run(&[]);
    assert_eq!(v1_output.exit_code, Some(0));

    let preview = fixture.run_queue(&[
        "migration-preview",
        "--repository-id",
        "termrock",
        "--worktree-id",
        "coordinator",
    ]);
    assert_eq!(preview.exit_code, Some(0), "{}", error_text(&preview));
    let preview: serde_json::Value =
        serde_json::from_slice(&preview.stdout).expect("parse canonical migration preview");
    assert_eq!(preview["writes_files"], false);
    assert_eq!(fs::read(&tasks_path).expect("tasks after preview"), v1_tasks);
    assert_eq!(
        fs::read(&queue_view_path).expect("queue view after preview"),
        v1_queue_view
    );

    let v2_records = preview["records"].clone();
    assert_eq!(v2_records["schema_version"], 2);
    fixture.write_tasks(&serde_json::to_string(&v2_records).expect("serialize previewed v2"));
    let v2_output = fixture.run(&[]);
    assert_eq!(v2_output.exit_code, Some(0), "{}", error_text(&v2_output));
    assert_eq!(v2_output.stdout, v1_output.stdout);

    let mut missing_task_identity = v2_records.clone();
    missing_task_identity["tasks"][0]
        .as_object_mut()
        .expect("first v2 task")
        .remove("repository_id");
    fixture.write_tasks(
        &serde_json::to_string(&missing_task_identity).expect("serialize task missing identity"),
    );
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("tasks[0] is missing fields: repository_id"));

    let mut missing_history_identity = v2_records.clone();
    let task_with_history = missing_history_identity["tasks"]
        .as_array()
        .expect("v2 task array")
        .iter()
        .position(|task| {
            task["claim_history"]
                .as_array()
                .is_some_and(|history| !history.is_empty())
        })
        .expect("migration preview contains a claim history snapshot");
    missing_history_identity["tasks"][task_with_history]["claim_history"][0]["claim"]
        .as_object_mut()
        .expect("claim snapshot")
        .remove("worktree_id");
    fixture.write_tasks(
        &serde_json::to_string(&missing_history_identity)
            .expect("serialize history snapshot missing identity"),
    );
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("claim snapshot has missing or unknown fields"));

    let mut duplicate_discriminator = v2_records;
    duplicate_discriminator
        .as_object_mut()
        .expect("v2 records object")
        .remove("schema_version");
    let serialized = serde_json::to_string(&duplicate_discriminator)
        .expect("serialize v2 records without discriminator");
    let fields = serialized
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .expect("serialized records object delimiters");
    let raw_duplicate = format!(
        "{{\"schema_version\":2,\"schema_version\":1,{fields}}}"
    );
    fixture.write_tasks(&raw_duplicate);
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("JSON object repeats key 'schema_version'"));
}

#[cfg(unix)]
#[test]
fn status_fails_closed_when_adjacent_queue_is_missing_or_symlinked() {
    let missing = StatusFixture::new();
    fs::remove_file(&missing.queue_script).expect("remove adjacent queue module");
    let output = missing.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("trusted adjacent queue module is unavailable"));

    let symlinked = StatusFixture::new();
    fs::remove_file(&symlinked.queue_script).expect("remove adjacent queue module");
    symlink(&symlinked.script, &symlinked.queue_script)
        .expect("replace adjacent queue module with symlink");
    let output = symlinked.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("trusted adjacent queue module is unavailable"));
}

#[test]
fn rejects_ambiguous_or_nonfinite_source_facts_without_status_output() {
    let duplicate_identity = replace_once(
        base_facts(),
        "\"head_sha\": \"3333333333333333333333333333333333333333\",",
        "\"head_sha\": \"3333333333333333333333333333333333333333\", \"head_sha\": \"1111111111111111111111111111111111111111\",",
    );
    let duplicate_fixture = StatusFixture::new();
    duplicate_fixture.write_facts(&duplicate_identity);
    let output = duplicate_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("JSON object repeats key 'head_sha'"));
    assert!(output.stdout.is_empty());
    assert!(!duplicate_fixture.repo.root().join("STATUS.md").exists());

    let nonfinite_version = replace_once(
        base_facts(),
        "\"schema_version\": 2,",
        "\"schema_version\": NaN,",
    );
    let nonfinite_fixture = StatusFixture::new();
    nonfinite_fixture.write_facts(&nonfinite_version);
    let output = nonfinite_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("JSON contains unsupported non-finite number NaN"));
    assert!(output.stdout.is_empty());
    assert!(!nonfinite_fixture.repo.root().join("STATUS.md").exists());
}

#[test]
fn source_history_accepts_four_predecessors_and_rejects_malformed_bindings() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value = serde_json::from_slice(
        &fs::read(facts_path).expect("read copied checked-in source facts"),
    )
    .expect("parse checked-in source facts");
    let original = with_four_predecessors(facts);
    assert_eq!(original["source_observation_history"].as_array().unwrap().len(), 5);

    fixture.write_facts(&original.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    let report = output_text(&output);
    assert!(report.contains("1ea1c17707f0a8f1639af506be179013d5e2d52a"));
    assert!(report.contains("b682cb26d68b353aeeccf9e51653eddf097b39f5"));
    assert!(report.contains("1d797d41c8141fcbdc3f69d7f11eb8875ab54712"));
    assert!(report.contains("b274dd57f4dd078ade6e424d546d83efbd2e8526"));
    assert_eq!(
        original["latest_source_observation"]["candidate_remote"]["head_sha"],
        "1d797d41c8141fcbdc3f69d7f11eb8875ab54712"
    );
    assert_eq!(
        original["latest_source_observation"]["reference_remote"]["head_sha"],
        "b274dd57f4dd078ade6e424d546d83efbd2e8526"
    );
    assert!(report.contains("Visibility / Complete | NOT_RUN"));
    assert!(report.contains("Refactor / Ready | NOT_READY"));
    assert!(report.contains("FAILED: 22/23 passed, 1 failed, 87 filtered"));
    assert!(report.contains(
        "verified candidate API/deferred result is FAILED: 22 of 23 passed, 1 failed"
    ));

    let mut malformed_source = original.clone();
    malformed_source["source_observation_history"][0]["source_observation"]
        .as_object_mut()
        .expect("source observation")
        .remove("reference_remote");
    fixture.write_facts(&malformed_source.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("candidate and reference tips are required"));

    let mut malformed_evidence = original.clone();
    malformed_evidence["source_observation_history"][0]["source_evidence"]["sha256"] =
        serde_json::json!("bad-digest");
    fixture.write_facts(&malformed_evidence.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("source_observation_history[0].source_evidence.sha256"));

    let mut reordered = original.clone();
    reordered["source_observation_history"].as_array_mut().unwrap().swap(0, 1);
    fixture.write_facts(&reordered.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("strictly ordered by observed_at"));

    let mut duplicate_predecessor = original.clone();
    let duplicated_record = duplicate_predecessor["source_observation_history"][0].clone();
    duplicate_predecessor["source_observation_history"].as_array_mut().unwrap()[1] =
        duplicated_record;
    fixture.write_facts(&duplicate_predecessor.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("reuses a source-evidence identity"));

    let mut duplicate_source_pair = original.clone();
    let first_candidate = duplicate_source_pair["source_observation_history"][0]
        ["source_observation"]["candidate_remote"]["head_sha"]
        .clone();
    let first_reference = duplicate_source_pair["source_observation_history"][0]
        ["source_observation"]["reference_remote"]["head_sha"]
        .clone();
    let history = duplicate_source_pair["source_observation_history"]
        .as_array_mut()
        .expect("source history array");
    history[1]["source_observation"]["candidate_remote"]["head_sha"] =
        first_candidate;
    history[1]["source_observation"]["reference_remote"]["head_sha"] =
        first_reference;
    history[1]["current_ci_observation"] = serde_json::Value::Null;
    fixture.write_facts(&duplicate_source_pair.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("duplicate source pair"));

    let mut reused_evidence = original.clone();
    let first_source_evidence = reused_evidence["source_observation_history"][0]
        ["source_evidence"]
        .clone();
    reused_evidence["source_observation_history"][1]["source_evidence"] =
        first_source_evidence;
    fixture.write_facts(&reused_evidence.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("reuses a source-evidence identity"));

    let mut misbound_fetch_evidence = original.clone();
    misbound_fetch_evidence["source_observation_history"][3]["source_evidence"]["path"] =
        serde_json::json!("/private/tmp/other-source-observation.json");
    fixture.write_facts(&misbound_fetch_evidence.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("does not match its explicit-fetch provenance"));

    let mut misbound_query = original.clone();
    misbound_query["latest_source_observation"]["fetch_provenance"]
        ["post_fetch_ref_query"]["reference_head_sha"] =
        serde_json::json!("1111111111111111111111111111111111111111");
    fixture.write_facts(&misbound_query.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("post-fetch refs do not match its branch API pair"));

    let mut mismatched_capture = original.clone();
    mismatched_capture["source_observation_history"][1]
        ["current_ci_observation"]["api_capture"]["source_pair"]["candidate_head_sha"] =
        serde_json::json!("1111111111111111111111111111111111111111");
    fixture.write_facts(&mismatched_capture.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "current CI candidate source does not match the latest candidate tip"
    ));

    let mut wrong_successor_time = original;
    wrong_successor_time["source_observation_history"][1]["superseded_at"] =
        serde_json::json!("2026-10-08T10:19:24Z");
    fixture.write_facts(&wrong_successor_time.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("superseded_at must equal the next source observation time"));
}

#[test]
fn explicit_source_ci_capture_distinguishes_null_from_missing() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value = serde_json::from_slice(
        &fs::read(facts_path).expect("read copied checked-in source facts"),
    )
    .expect("parse checked-in source facts");
    let original = with_four_predecessors(facts);

    let mut explicit_null = original.clone();
    explicit_null["current_ci_observation"] = serde_json::Value::Null;
    fixture.write_facts(&explicit_null.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    let report = output_text(&output);
    assert!(!report.contains("## Current CI and DCO observations"));
    assert_eq!(
        original["latest_source_observation"]["candidate_remote"]["head_sha"],
        "1d797d41c8141fcbdc3f69d7f11eb8875ab54712"
    );
    assert_eq!(
        original["latest_source_observation"]["reference_remote"]["head_sha"],
        "b274dd57f4dd078ade6e424d546d83efbd2e8526"
    );
    assert!(report.contains("1d797d41c8141fcbdc3f69d7f11eb8875ab54712"));
    assert!(report.contains("b274dd57f4dd078ade6e424d546d83efbd2e8526"));
    assert!(report.contains("| Pair selected at | 2026-10-08T23:25:02Z |"));

    // This checked-in pair uses a branch API observation, so its top-level CI
    // capture is optional. Exercise the explicit-fetch presence rule with a
    // small synthetic source observation whose provenance is internally bound.
    let explicit_fetch_fixture = StatusFixture::new();
    let mut explicit_fetch: serde_json::Value =
        serde_json::from_str(&base_facts()).expect("parse synthetic explicit-fetch facts");
    explicit_fetch["source_observation_history"] = serde_json::json!([]);
    explicit_fetch["current_ci_observation"] = serde_json::Value::Null;
    explicit_fetch["latest_source_observation"]["method"] =
        serde_json::json!("explicit HTTPS refs/heads fetch");
    explicit_fetch["latest_source_observation"]["branch_last_update_at"] =
        serde_json::Value::Null;
    explicit_fetch["latest_source_observation"]["timestamp_provenance"] =
        serde_json::json!({
            "candidate_committer_date_source": "synthetic explicit-fetch fixture",
            "reference_committer_date_source": "synthetic explicit-fetch fixture",
            "branch_last_update_at_source": "not recorded in this synthetic fixture",
            "committer_dates_are_branch_update_times": false
        });
    explicit_fetch["latest_source_observation"]["candidate_remote"]["commit_committer_at"] =
        serde_json::json!("2026-10-08T04:20:00Z");
    assert!(explicit_fetch["latest_source_observation"]["candidate_remote"]
        .as_object_mut()
        .expect("candidate remote")
        .remove("updated_at")
        .is_some());
    explicit_fetch["latest_source_observation"]["reference_remote"]["commit_committer_at"] =
        serde_json::json!("2026-10-08T04:10:00Z");
    assert!(explicit_fetch["latest_source_observation"]["reference_remote"]
        .as_object_mut()
        .expect("reference remote")
        .remove("updated_at")
        .is_some());
    explicit_fetch["latest_source_observation"]["fetch_provenance"] =
        serde_json::json!({
            "evidence_path": "fixtures/source-observation/latest.json",
            "evidence_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "evidence_kind": "synthetic explicit-fetch test fixture",
            "transport": "HTTPS",
            "operation": "fetch",
            "refspecs": [
                "refs/heads/termrock-implementation",
                "refs/heads/visual-baseline"
            ],
            "session_id": 1,
            "exit_code": 0,
            "completed_at": "2026-10-08T04:30:00Z",
            "raw_fetch_output_preserved": false,
            "post_fetch_ref_query": {
                "record_id": "synthetic-explicit-fetch-query",
                "observed_at": null,
                "candidate_head_sha": "3333333333333333333333333333333333333333",
                "reference_head_sha": "4444444444444444444444444444444444444444"
            }
        });
    explicit_fetch_fixture.write_facts(&explicit_fetch.to_string());
    let output = explicit_fetch_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    assert!(!output_text(&output).contains("## Current CI and DCO observations"));

    let mut missing_committer_date = explicit_fetch.clone();
    assert!(missing_committer_date["latest_source_observation"]["candidate_remote"]
        .as_object_mut()
        .expect("candidate remote")
        .remove("commit_committer_at")
        .is_some());
    explicit_fetch_fixture.write_facts(&missing_committer_date.to_string());
    let output = explicit_fetch_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "latest_source_observation.candidate_remote.commit_committer_at must be text"
    ));

    let mut malformed_committer_date = explicit_fetch.clone();
    malformed_committer_date["latest_source_observation"]["reference_remote"]
        ["commit_committer_at"] = serde_json::json!("not-a-timestamp");
    explicit_fetch_fixture.write_facts(&malformed_committer_date.to_string());
    let output = explicit_fetch_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "latest_source_observation.reference_remote.commit_committer_at must be RFC3339 with a timezone"
    ));

    let mut missing = explicit_fetch;
    missing.as_object_mut()
        .expect("source facts object")
        .remove("current_ci_observation");
    explicit_fetch_fixture.write_facts(&missing.to_string());
    let output = explicit_fetch_fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "source observation requires current_ci_observation; use null if no capture exists"
    ));

    let mut missing_history_null = original;
    missing_history_null["source_observation_history"][2]
        .as_object_mut()
        .expect("source history entry")
        .remove("current_ci_observation");
    fixture.write_facts(&missing_history_null.to_string());
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "current_ci_observation must be present; use null for an absent capture"
    ));
}

#[test]
fn reports_latest_remote_refs_and_local_checkout_separately() {
    let fixture = StatusFixture::new();
    let report = output_text(&fixture.run(&[]));

    assert!(report.contains("Latest source and local checkout observations"));
    assert!(report.contains("| Pair observed at | UNKNOWN |"));
    assert!(report.contains("3333333333333333333333333333333333333333"));
    assert!(report.contains("4444444444444444444444444444444444444444"));
    assert!(report.contains("5555555555555555555555555555555555555555"));
    assert!(report.contains("6666666666666666666666666666666666666666"));
    assert!(report.contains("Local cryptographic commit signature"));
    assert!(report.contains("Git reported no cryptographic signature"));
    assert!(report.contains("Local DCO sign-off trailer"));
    assert!(report.contains(
        "Expected exact line `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` is missing"
    ));
    assert!(report.contains("parsed trailers: none"));
    assert!(report.contains("does not infer a remote DCO check"));
    assert!(report.contains("| Local publication | Local checkout observation only. |"));
    assert!(report.contains("two latest branch tips are separate observations"));
    assert!(report.contains("Recorded CI and repository checks"));
    assert!(report.contains("does not match the latest observed candidate branch tip"));
    assert!(report.contains("Visibility / Complete | NOT_RUN"));
    assert!(report.contains("Refactor / Ready | NOT_RUN"));
    assert!(report.contains("cannot show a product pass"));
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
    assert!(report.contains(
        "This recorded workflow result is historical and is not a product test result."
    ));
    assert!(report.contains("failure at"));
    assert!(report.contains("113129904697"));
    assert!(report.contains("action_required"));
    assert!(report.contains("separate from product results"));
    assert!(report.contains("cargo-nextest 0.9.146 through mise"));
}

#[test]
fn absent_optional_current_ci_capture_keeps_only_historical_rows() {
    let fixture = StatusFixture::new();
    let report = output_text(&fixture.run(&[]));

    assert!(report.contains("Recorded CI and repository checks"));
    assert!(report.contains("37721476033"));
    assert!(report.contains("113129904697"));
    assert!(!report.contains("Current CI and DCO observations"));
    assert!(!report.contains("no matching Actions runs"));
}

#[test]
fn current_ci_snapshot_binds_each_gate_and_keeps_products_not_run() {
    let fixture = StatusFixture::new();
    write_json_facts(&fixture, &facts_with_current_ci_observation());
    let output = fixture.run(&[]);
    let report = output_text(&output);

    assert_eq!(output.exit_code, Some(0));
    assert!(report.contains("Current CI and DCO observations"));
    assert!(report.contains("Candidate Actions run [987654321]"));
    assert!(report.contains("failure at `3333333333333333333333333333333333333333`; 0 jobs; 0 artifacts"));
    assert!(report.contains("returned 0 matching Actions runs for this source tip"));
    assert!(report.contains("Candidate DCO check [888888888]"));
    assert!(report.contains("2 commits are reported with sign-off problems"));
    assert!(report.contains("Reference DCO check [999999999]"));
    assert!(report.contains("14 commits are reported with sign-off problems"));
    assert!(report.contains("Workflow file exceeds the maximum allowed size of 500 KB."));
    assert!(report.contains("SHA-256 `dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd`"));
    assert!(report.contains("Workflow-level result; product execution remains NOT_RUN"));
    assert!(report.contains("Visibility / Complete | NOT_RUN"));
    assert!(report.contains("Refactor / Ready | NOT_RUN"));
    assert!(report.contains("37721476033"));
    assert!(report.contains("113129904697"));
}

#[test]
fn current_ci_source_pair_run_head_workflow_and_url_mismatches_are_rejected() {
    let mutations: [(&[&str], serde_json::Value, &str); 5] = [
        (
            &["api_capture", "source_pair", "candidate_head_sha"],
            serde_json::json!("0000000000000000000000000000000000000000"),
            "current CI candidate source does not match the latest candidate tip",
        ),
        (
            &["api_capture", "source_pair", "reference_head_sha"],
            serde_json::json!("0000000000000000000000000000000000000000"),
            "current CI reference source does not match the latest reference tip",
        ),
        (
            &["api_capture", "candidate_run", "head_sha"],
            serde_json::json!("0000000000000000000000000000000000000000"),
            "current candidate run head does not match its source pair",
        ),
        (
            &["api_capture", "workflow_source", "path"],
            serde_json::json!(".github/workflows/other.yml"),
            "current workflow source path does not match the run",
        ),
        (
            &["api_capture", "candidate_run", "run_url"],
            serde_json::json!("https://github.com/tailrocks/terminal-components-claude/actions/runs/123"),
            "current candidate run URL does not match its ID",
        ),
    ];

    for (path, value, expected_error) in mutations {
        let fixture = StatusFixture::new();
        let mut facts = facts_with_current_ci_observation();
        let path = format!("/current_ci_observation/{}", path.join("/"));
        *facts.pointer_mut(&path).expect("current CI fixture path") = value;
        write_json_facts(&fixture, &facts);
        let output = fixture.run(&[]);

        assert_eq!(output.exit_code, Some(2));
        assert!(error_text(&output).contains(expected_error));
    }
}

#[test]
fn missing_or_failed_reference_query_is_unknown_not_zero() {
    let fixture = StatusFixture::new();
    let mut missing = facts_with_current_ci_observation();
    missing["current_ci_observation"]["api_capture"]
        .as_object_mut()
        .expect("API capture object")
        .remove("reference_query");
    write_json_facts(&fixture, &missing);
    let missing_report = output_text(&fixture.run(&[]));
    assert!(missing_report.contains("query was not captured; matching-run status is unknown"));
    assert!(!missing_report.contains("returned 0 matching Actions runs"));

    let mut failed = facts_with_current_ci_observation();
    failed["current_ci_observation"]["api_capture"]["reference_query"]["query_status"] =
        serde_json::json!("error");
    failed["current_ci_observation"]["api_capture"]["reference_query"]["query_total_count"] =
        serde_json::Value::Null;
    failed["current_ci_observation"]["api_capture"]["reference_query"]["error"] =
        serde_json::json!("GitHub API request failed");
    write_json_facts(&fixture, &failed);
    let failed_report = output_text(&fixture.run(&[]));
    assert!(failed_report.contains("query failed; matching-run status is unknown"));
    assert!(!failed_report.contains("returned 0 matching Actions runs"));
}

#[test]
fn mismatched_or_absent_provider_capture_does_not_supply_failure_cause() {
    let mutations: [(&[&str], Option<serde_json::Value>); 4] = [
        (
            &["run_id"],
            Some(serde_json::json!("123456789")),
        ),
        (
            &["workflow_path"],
            Some(serde_json::json!(".github/workflows/other.yml")),
        ),
        (
            &["workflow_commit_sha"],
            Some(serde_json::json!("0000000000000000000000000000000000000000")),
        ),
        (
            &["workflow_source_sha256"],
            Some(serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")),
        ),
    ];

    for (path, value) in mutations {
        let fixture = StatusFixture::new();
        let mut facts = facts_with_current_ci_observation();
        let path = format!(
            "/current_ci_observation/provider_annotation_capture/{}",
            path.join("/")
        );
        *facts.pointer_mut(&path).expect("provider fixture path") =
            value.expect("provider mutation value");
        write_json_facts(&fixture, &facts);
        let report = output_text(&fixture.run(&[]));
        let (historical, after_current) = report
            .split_once("## Current CI and DCO observations")
            .expect("current CI section boundary");
        let (current, _) = after_current
            .split_once("Tool versions recorded at ")
            .expect("current CI section end boundary");

        assert!(historical.contains(&format!("Failure reason: {FAILURE_REASON}")));
        assert!(current.contains(
            "| Candidate Actions run [987654321](https://github.com/tailrocks/terminal-components-claude/actions/runs/987654321) | failure at `3333333333333333333333333333333333333333`; 0 jobs; 0 artifacts. No correlated provider annotation is recorded, so the failure cause is not stated. | Workflow-level result; product execution remains NOT_RUN. |"
        ));
        assert!(!current.contains("Workflow file exceeds the maximum allowed size"));
    }

    let fixture = StatusFixture::new();
    let mut absent = facts_with_current_ci_observation();
    absent["current_ci_observation"]["provider_annotation_capture"] = serde_json::Value::Null;
    write_json_facts(&fixture, &absent);
    let report = output_text(&fixture.run(&[]));
    let (historical, after_current) = report
        .split_once("## Current CI and DCO observations")
        .expect("current CI section boundary");
    let (current, _) = after_current
        .split_once("Tool versions recorded at ")
        .expect("current CI section end boundary");
    assert!(historical.contains(&format!("Failure reason: {FAILURE_REASON}")));
    assert!(current.contains(
        "| Candidate Actions run [987654321](https://github.com/tailrocks/terminal-components-claude/actions/runs/987654321) | failure at `3333333333333333333333333333333333333333`; 0 jobs; 0 artifacts. No correlated provider annotation is recorded, so the failure cause is not stated. | Workflow-level result; product execution remains NOT_RUN. |"
    ));
    assert!(!current.contains("Workflow file exceeds the maximum allowed size"));
}

#[test]
fn current_dco_heads_must_match_their_candidate_and_reference_tips() {
    for (role, expected_error) in [
        (
            "candidate_dco",
            "current candidate DCO head does not match its source tip",
        ),
        (
            "reference_dco",
            "current reference DCO head does not match its source tip",
        ),
    ] {
        let fixture = StatusFixture::new();
        let mut facts = facts_with_current_ci_observation();
        facts["current_ci_observation"]["api_capture"][role]["head_sha"] =
            serde_json::json!("0000000000000000000000000000000000000000");
        write_json_facts(&fixture, &facts);
        let output = fixture.run(&[]);

        assert_eq!(output.exit_code, Some(2));
        assert!(error_text(&output).contains(expected_error));
    }
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
fn rejects_previous_source_facts_schema_version() {
    let fixture = StatusFixture::new();
    let facts = replace_once(
        base_facts(),
        "\"schema_version\": 2",
        "\"schema_version\": 1",
    );
    write_facts(&fixture, &facts);
    let output = fixture.run(&[]);

    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains("unsupported source-facts schema; expected version 2"));
}

#[test]
fn does_not_accept_wrong_or_partial_dco_trailers() {
    let fixture = StatusFixture::new();
    for wrong_trailer in [
        "Signed-off-by: Alexey",
        "Signed-off-by: Alexey Zhokhov",
        "Signed-off-by: Alexey Zhokhov <alexey@other.example>",
        "Signed-off-by: Other Person <alexey@zhokhov.com>",
    ] {
        let facts = replace_once(
            base_facts(),
            "\"parsed_trailers\": []",
            &format!("\"parsed_trailers\": [\"{wrong_trailer}\"]"),
        );
        fixture.write_facts(&facts);
        let output = fixture.run(&[]);
        let report = output_text(&output);

        assert_eq!(output.exit_code, Some(0));
        assert!(report.contains(
            "Expected exact line `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` is missing"
        ));
        assert!(!report.contains(
            "Expected exact line `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` is present"
        ));
    }
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

#[test]
fn current_observations_render_source_bound_branch_and_candidate_failure() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(0));
    let report = output_text(&output);

    assert!(report.contains("Current implementation-branch and external-tool observations"));
    assert!(report.contains("9f1f756219b6bd42131b6e7291f54f99e1128cf1"));
    assert!(report.contains("DCO check 113663911182 completed successfully"));
    assert!(report.contains("37882112951"));
    assert!(report.contains("0 jobs; 0 artifacts; workflow source 537470 bytes"));
    assert!(report.contains("FAILED: 22/23 passed, 1 failed, 87 filtered"));
    assert!(report.contains("BD-21 case W13-05 failed in w13_filter_wide_trail_cells_clear"));
    assert!(report.contains("independent review VERIFIED_FAILED_RUN"));
    assert!(report.contains("Requirement registry: NOT_RUN"));
    assert!(report.contains("paired reference: NOT_RUN; paired visual: NOT_RUN; acceptance: NOT_RECORDED"));
    assert!(report.contains("Repository evidence archive | 59 files / 5554254 bytes verified"));
    assert!(report.contains("Historical raw-evidence coverage gap: normalized source/CI summaries for 5 retained records remain"));
    assert!(report.contains("Receipt SHA-256 c6fec50571f45a875c375bf5668e4f1b8754eb9ac87ec9aa98c95590c79e1929"));
    assert!(report.contains("R12: Nextest stopped before selecting tests; 0 selected"));
    assert!(report.contains("R13: 1 selected, 0 passed, 1 failed before PTY launch"));
    assert!(report.contains("16/27 expected tests executed (14 passed, 2 failed, 11 NOT_RUN)"));
    for conclusion in [
        "Visibility / Complete | NOT_RUN",
        "Refactor / Ready | NOT_READY",
        "Reference / Qualified | NOT_RUN",
        "Command / Ready | NOT_RUN",
        "Evidence freshness | NOT_RUN",
    ] {
        assert!(report.contains(conclusion), "missing `{conclusion}`");
    }
    assert_eq!(
        facts["current_status_observations"]["candidate_api_deferred_run"]["measurement_status"]
            .as_str(),
        Some("FAILED")
    );
    assert_eq!(
        facts["current_status_observations"]["report_readiness"]["refactor_ready"]
            .as_str(),
        Some("NOT_RUN")
    );
}

#[test]
fn rejects_changed_current_observation_evidence_pin() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    facts["current_status_observations"]["branch_tip"]["branch_response"]["sha256"] =
        serde_json::Value::String("0".repeat(64));
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "current implementation branch API response bytes do not match the recorded SHA-256"
    ));
}

#[test]
fn rejects_a_temporary_root_locator_in_checked_in_source_facts() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    facts["test_only_unbound_locator"] = serde_json::Value::String(
        "/private/tmp/termrock-vis01-branch-b106-observation-20261009-luna-r1/branch.json"
            .to_string(),
    );
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "source-facts paths must be repository-relative"
    ));
}

#[test]
fn rejects_a_stale_source_evidence_archive_manifest_pin() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    facts["source_evidence_archive"]["manifest"]["sha256"] =
        serde_json::Value::String("0".repeat(64));
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "source evidence archive manifest bytes do not match the recorded SHA-256"
    ));
}

#[test]
fn rejects_missing_archived_current_observation_input() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    let archived_path = facts["current_status_observations"]["branch_tip"]
        ["branch_response"]["path"]
        .as_str()
        .expect("repository-relative branch response path");
    fs::remove_file(fixture.repo.root().join(archived_path))
        .expect("remove copied current archive member");

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "cannot be opened without following symlinks"
    ));
}

#[test]
fn rejects_tampered_candidate_measurement_packet_member() {
    let fixture = StatusFixture::from_checked_in_records();
    let member = fixture.repo.root().join(
        "docs/implementation/visibility/evidence/deferred/product23-s1d-20261009/provenance/preflight.json",
    );
    let mut bytes = fs::read(&member).expect("read copied packet member");
    bytes.push(b' ');
    fs::write(&member, bytes).expect("tamper copied packet member");

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "bytes do not match the recorded SHA-256"
    ));
}

#[test]
fn keeps_historical_raw_capture_gap_separate_from_readiness() {
    let fixture = StatusFixture::from_checked_in_records();
    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(0));
    let report = output_text(&output);
    assert!(report.contains("Historical raw-evidence coverage gap"));
    assert!(report.contains("Visibility / Complete | NOT_RUN"));
    assert!(report.contains("Refactor / Ready | NOT_READY"));
    assert!(report.contains("Reference / Qualified | NOT_RUN"));
    assert!(report.contains("Command / Ready | NOT_RUN"));
    assert!(report.contains("Evidence freshness | NOT_RUN"));
}

#[test]
fn rejects_relabeling_a_verified_candidate_failure_as_not_run() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    facts["current_status_observations"]["candidate_api_deferred_run"]["measurement_status"] =
        serde_json::Value::String("NOT_RUN".to_string());
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "candidate deferred/API run must remain a source-bound partial failure"
    ));
}


#[test]
fn rejects_unrecorded_candidate_acceptance_promotion() {
    let fixture = StatusFixture::from_checked_in_records();
    let facts_path = fixture
        .repo
        .root()
        .join("tools/visibility/source-facts.json");
    let mut facts: serde_json::Value =
        serde_json::from_slice(&fs::read(facts_path).expect("read copied source facts"))
            .expect("parse copied source facts");
    facts["current_status_observations"]["candidate_api_deferred_run"]["acceptance_decision"] =
        serde_json::Value::String("ACCEPTED".to_string());
    write_json_facts(&fixture, &facts);

    let output = fixture.run(&[]);
    assert_eq!(output.exit_code, Some(2));
    assert!(error_text(&output).contains(
        "candidate deferred/API run must remain a source-bound partial failure"
    ));
}
