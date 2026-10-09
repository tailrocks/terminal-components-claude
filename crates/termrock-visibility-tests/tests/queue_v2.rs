//! Black-box checks for read-only queue v2 records and registry roles.
//!
//! These tests launch the exact production queue CLI against disposable Git
//! repositories. They do not import Python modules or use a Python test runner.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::{Value, json};
use termrock_visibility_tests::{CliOutput, TempRepo, Tool, run_cli, sha256_hex};

const CANDIDATE_BRANCH: &str = "termrock-implementation";
const FROZEN_VISUAL_TAG_OBJECT_SHA: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const FROZEN_VISUAL_TAG_COMMIT_SHA: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
struct RegistryFixture {
    candidate: TempRepo,
    origin: TempRepo,
    control: TempRepo,
    reference_root: PathBuf,
    base_sha: String,
    tag_object_sha: String,
    script: PathBuf,
}

struct ExternalWorktrees {
    repository: TempRepo,
    origin: TempRepo,
    source_pin_root: PathBuf,
    scratch_root: PathBuf,
    head_sha: String,
    tree_sha: String,
}

impl ExternalWorktrees {
    fn new(control_root: &Path) -> Self {
        let repository = TempRepo::new().expect("create separate external repository");
        let origin = TempRepo::new().expect("create separate external origin");
        fs::write(
            repository.root().join("external-fixture.txt"),
            b"external source\n",
        )
        .expect("write external seed file");
        git(origin.root(), &["init", "--bare", "--quiet"]);
        git(repository.root(), &["init", "--quiet"]);
        git(repository.root(), &["checkout", "--quiet", "-b", "main"]);
        git(repository.root(), &["add", "--all"]);
        git_with_identity(
            repository.root(),
            &["commit", "--quiet", "-m", "external source pin"],
        );
        let origin_path = path_text(origin.root());
        git(
            repository.root(),
            &["remote", "add", "origin", origin_path.as_str()],
        );
        let head_sha = git_output(repository.root(), &["rev-parse", "--verify", "HEAD"]);
        let tree_sha = git_output(repository.root(), &["rev-parse", "--verify", "HEAD^{tree}"]);
        let source_pin_root = control_root.join("velnor-source-pin");
        let scratch_root = control_root.join("velnor-scratch");
        let source_pin_path = path_text(&source_pin_root);
        git(
            repository.root(),
            &[
                "worktree",
                "add",
                "--quiet",
                "--detach",
                source_pin_path.as_str(),
                head_sha.as_str(),
            ],
        );
        let scratch_path = path_text(&scratch_root);
        git(
            repository.root(),
            &[
                "worktree",
                "add",
                "--quiet",
                "--detach",
                scratch_path.as_str(),
                head_sha.as_str(),
            ],
        );
        Self {
            repository,
            origin,
            source_pin_root,
            scratch_root,
            head_sha,
            tree_sha,
        }
    }

    fn add_to_registry(&self, registry: &mut Value) {
        let allowed_paths = vec!["external-fixture.txt"];
        let policy = json!({
            "repository_id": "velnor-new",
            "source_pin": {
                "commit_sha": self.head_sha,
                "role": "immutable-source-pin",
                "tree_sha": self.tree_sha,
                "worktree_id": "velnor-source-pin"
            },
            "scratch": {
                "allowed_paths": allowed_paths.clone(),
                "role": "writable-isolated-scratch",
                "worktree_id": "velnor-scratch"
            },
            "schema": "termrock-source-selection-path-policy/v1"
        });
        let policy_json = serde_json::to_vec(&policy).expect("serialize source path policy");
        let mut policy_input = b"termrock-source-selection-path-policy/v1\0".to_vec();
        policy_input.extend_from_slice(&policy_json);
        let policy_sha256 = sha256_hex(&policy_input);
        let review_record = json!({
            "schema": "termrock-source-selection-review/v1",
            "reviewer": "independent-source-reviewer",
            "decision": "approved_for_source_selection",
            "reviewed_at": "2026-10-08T00:00:00Z",
            "subject": {
                "repository_id": "velnor-new",
                "commit_sha": self.head_sha,
                "tree_sha": self.tree_sha,
                "path_policy_sha256": policy_sha256
            },
            "evidence": ["https://example.invalid/source-review/fixture"]
        });
        let review_bytes = serde_json::to_vec(&review_record).expect("serialize review receipt");
        let review_path = self
            .source_pin_root
            .parent()
            .unwrap()
            .join("source-selection-review.json");
        fs::write(&review_path, &review_bytes).expect("write synthetic source-selection receipt");

        registry["repositories"]["velnor-new"] = json!({
            "origin_url": path_text(self.origin.root())
        });
        registry["worktrees"]["velnor-source-pin"] = json!({
            "repository_id": "velnor-new",
            "root": path_text(&self.source_pin_root),
            "common_dir": path_text(&self.repository.root().join(".git")),
            "mode": "source-pin",
            "branch": null,
            "pinned_sha": self.head_sha,
            "writable": false,
            "source_pin_id": null,
            "allowed_paths": [],
            "source_selection_review": {
                "review_record_path": path_text(&review_path),
                "review_record_sha256": sha256_hex(&review_bytes)
            }
        });
        registry["worktrees"]["velnor-scratch"] = json!({
            "repository_id": "velnor-new",
            "root": path_text(&self.scratch_root),
            "common_dir": path_text(&self.repository.root().join(".git")),
            "mode": "scratch",
            "branch": null,
            "pinned_sha": self.head_sha,
            "writable": true,
            "source_pin_id": "velnor-source-pin",
            "allowed_paths": allowed_paths,
            "source_selection_review": null
        });
    }
}

impl RegistryFixture {
    fn new() -> Self {
        let candidate = TempRepo::new().expect("create candidate repository root");
        let origin = TempRepo::new().expect("create bare origin root");
        let control = TempRepo::new().expect("create external registry root");
        let tool = candidate
            .copy_tool_exact(Tool::Queue)
            .expect("copy exact queue CLI");
        let source_repository =
            fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .expect("resolve checked-in source repository for frozen tag fixture");
        let source_objects = git_output(
            &source_repository,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "objects",
            ],
        );

        fs::write(
            candidate.root().join("fixture.txt"),
            b"registry test fixture\n",
        )
        .expect("write Git seed file");
        git(origin.root(), &["init", "--bare", "--quiet"]);
        git(candidate.root(), &["init", "--quiet"]);
        for object_dir in [
            candidate.root().join(".git/objects/info"),
            origin.root().join("objects/info"),
        ] {
            fs::create_dir_all(&object_dir).expect("create fixture Git alternate directory");
            fs::write(object_dir.join("alternates"), format!("{source_objects}\n"))
                .expect("point disposable Git fixture at read-only source object database");
        }
        git(
            candidate.root(),
            &["checkout", "--quiet", "-b", CANDIDATE_BRANCH],
        );
        git(candidate.root(), &["add", "--all"]);
        git_with_identity(
            candidate.root(),
            &["commit", "--quiet", "-m", "registry fixture base"],
        );
        git(
            candidate.root(),
            &[
                "update-ref",
                "refs/tags/visual-baseline",
                FROZEN_VISUAL_TAG_OBJECT_SHA,
            ],
        );
        let base_sha = FROZEN_VISUAL_TAG_COMMIT_SHA.to_owned();
        let tag_object_sha = git_output(
            candidate.root(),
            &["rev-parse", "--verify", "refs/tags/visual-baseline^{tag}"],
        );
        assert_eq!(tag_object_sha, FROZEN_VISUAL_TAG_OBJECT_SHA);
        assert_eq!(
            git_output(
                candidate.root(),
                &[
                    "rev-parse",
                    "--verify",
                    "refs/tags/visual-baseline^{commit}"
                ]
            ),
            FROZEN_VISUAL_TAG_COMMIT_SHA
        );
        let origin_path = path_text(origin.root());
        git(
            candidate.root(),
            &["remote", "add", "origin", origin_path.as_str()],
        );
        git(
            origin.root(),
            &[
                "update-ref",
                "refs/heads/visual-baseline",
                base_sha.as_str(),
            ],
        );
        git(
            candidate.root(),
            &[
                "update-ref",
                "refs/remotes/origin/visual-baseline",
                base_sha.as_str(),
            ],
        );
        let reference_root = control.root().join("reference-worktree");
        let reference_path = path_text(&reference_root);
        git(
            candidate.root(),
            &[
                "worktree",
                "add",
                "--quiet",
                "--no-checkout",
                "-b",
                "visual-baseline",
                reference_path.as_str(),
                base_sha.as_str(),
            ],
        );
        git(&reference_root, &["read-tree", "HEAD"]);
        let indexed_paths = git_output_bytes(&reference_root, &["ls-files", "-z"]);
        assert!(
            indexed_paths.len() <= 32 * 1024 * 1024,
            "frozen reference fixture path metadata exceeded 32 MiB"
        );
        git_with_input(
            &reference_root,
            &["update-index", "--skip-worktree", "-z", "--stdin"],
            &indexed_paths,
        );
        assert_eq!(
            fs::read_dir(&reference_root)
                .expect("read sparse reference fixture root")
                .count(),
            1,
            "frozen visual fixture must not extract baseline files"
        );

        Self {
            candidate,
            origin,
            control,
            reference_root,
            base_sha,
            tag_object_sha,
            script: tool.path,
        }
    }

    fn candidate_root(&self) -> &Path {
        self.candidate.root()
    }

    fn base_registry(&self) -> Value {
        json!({
            "schema_version": 1,
            "repositories": {
                "terminal-components-claude": {
                    "origin_url": path_text(self.origin.root())
                }
            },
            "worktrees": {
                "candidate": {
                    "repository_id": "terminal-components-claude",
                    "root": path_text(self.candidate.root()),
                    "common_dir": path_text(&self.candidate.root().join(".git")),
                    "mode": "branch",
                    "branch": CANDIDATE_BRANCH,
                    "pinned_sha": null,
                    "writable": true,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                },
                "reference": {
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&self.reference_root),
                    "common_dir": path_text(&self.candidate.root().join(".git")),
                    "mode": "branch",
                    "branch": "visual-baseline",
                    "pinned_sha": self.base_sha,
                    "writable": true,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                }
            },
            "visual_authority": {
                "tag": "visual-baseline",
        "tag_object_sha": FROZEN_VISUAL_TAG_OBJECT_SHA,
        "peeled_commit_sha": FROZEN_VISUAL_TAG_COMMIT_SHA
            }
        })
    }

    fn write_registry(&self, registry: &Value) -> (PathBuf, String) {
        let raw = serde_json::to_vec(registry).expect("serialize registry fixture");
        let path = self
            .control
            .write_file(Path::new("registry.json"), &raw)
            .expect("write external registry fixture");
        (path, sha256_hex(&raw))
    }

    fn run_check(
        &self,
        registry: &Value,
        selected_worktree: &str,
        root_override: Option<&Path>,
        registry_hash_override: Option<&str>,
        environment: &[(&str, &str)],
    ) -> CliOutput {
        let (registry_path, registry_hash) = self.write_registry(registry);
        let root = root_override.unwrap_or(self.candidate.root());
        let args = vec![
            "--root".to_owned(),
            path_text(root),
            "registry-check".to_owned(),
            "--registry".to_owned(),
            path_text(&registry_path),
            "--registry-sha256".to_owned(),
            registry_hash_override.unwrap_or(&registry_hash).to_owned(),
            "--worktree".to_owned(),
            selected_worktree.to_owned(),
        ];
        run_cli(&self.script, &args, root, environment, None).expect("run queue registry check")
    }

    fn run_check_with_registry_path(
        &self,
        registry: &Value,
        selected_worktree: &str,
        registry_path: &Path,
    ) -> CliOutput {
        let raw = serde_json::to_vec(registry).expect("serialize registry fixture");
        fs::write(registry_path, &raw).expect("write registry fixture at requested path");
        let args = vec![
            "--root".to_owned(),
            path_text(self.candidate.root()),
            "registry-check".to_owned(),
            "--registry".to_owned(),
            path_text(registry_path),
            "--registry-sha256".to_owned(),
            sha256_hex(&raw),
            "--worktree".to_owned(),
            selected_worktree.to_owned(),
        ];
        run_cli(&self.script, &args, self.candidate.root(), &[], None)
            .expect("run queue registry check")
    }
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn git(root: &Path, arguments: &[&str]) {
    let output = git_command(root, arguments);
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_with_identity(root: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_AUTHOR_NAME", "Queue v2 Tests")
        .env("GIT_AUTHOR_EMAIL", "queue-v2-tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "Queue v2 Tests")
        .env("GIT_COMMITTER_EMAIL", "queue-v2-tests@example.invalid")
        .output()
        .expect("start Git fixture command");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_command(root: &Path, arguments: &[&str]) -> Output {
    Command::new("git")
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .output()
        .expect("start Git fixture command")
}

fn git_output(root: &Path, arguments: &[&str]) -> String {
    let output = git_command(root, arguments);
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git output is UTF-8")
        .trim()
        .to_owned()
}

fn git_output_bytes(root: &Path, arguments: &[&str]) -> Vec<u8> {
    let output = git_command(root, arguments);
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn git_with_input(root: &Path, arguments: &[&str], input: &[u8]) {
    let mut child = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Git fixture command with stdin");
    let mut stdin = child.stdin.take().expect("Git fixture stdin is piped");
    let write_result = stdin.write_all(input);
    drop(stdin);
    let output = child
        .wait_with_output()
        .expect("wait for Git fixture command");
    if let Err(error) = write_result {
        panic!(
            "write Git fixture stdin failed for {arguments:?}: {error}; status={:?}, stdout={:?}, stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn add_velnor_pair(
    registry: &mut Value,
    repository_id: &str,
    origin_url: &str,
    source_pin_sha: &str,
    scratch_sha: &str,
) {
    registry["repositories"][repository_id] = json!({"origin_url": origin_url});
    registry["worktrees"]["velnor-source-pin"] = json!({
        "repository_id": repository_id,
        "root": "/private/tmp/velnor-source-pin",
        "common_dir": "/private/tmp/velnor-repository/.git",
        "mode": "source-pin",
        "branch": null,
        "pinned_sha": source_pin_sha,
        "writable": false,
        "source_pin_id": null,
        "allowed_paths": [],
        "source_selection_review": {
            "review_record_path": "/private/tmp/source-selection-review.json",
            "review_record_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        }
    });
    registry["worktrees"]["velnor-scratch"] = json!({
        "repository_id": repository_id,
        "root": "/private/tmp/velnor-scratch",
        "common_dir": "/private/tmp/velnor-repository/.git",
        "mode": "scratch",
        "branch": null,
        "pinned_sha": scratch_sha,
        "writable": true,
        "source_pin_id": "velnor-source-pin",
        "allowed_paths": ["external-fixture.txt"],
        "source_selection_review": null
    });
}

fn assert_success(output: &CliOutput) {
    assert_eq!(
        output.exit_code,
        Some(0),
        "CLI stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.signal, None, "CLI was killed by a signal");
    assert!(!output.timed_out, "CLI exceeded its timeout");
}

fn assert_error(output: &CliOutput, expected: &str) {
    assert_ne!(output.exit_code, Some(0), "CLI unexpectedly succeeded");
    assert_eq!(output.signal, None, "CLI was killed by a signal");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "expected stderr to contain {expected:?}, got {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registry_check_reports_candidate_identity_and_clean_state() {
    let fixture = RegistryFixture::new();
    let index_path = fixture.candidate_root().join(".git/index");
    let index_before = fs::read(&index_path).expect("read candidate Git index before preflight");
    let output = fixture.run_check(&fixture.base_registry(), "candidate", None, None, &[]);
    assert_success(&output);

    let report: Value = serde_json::from_slice(&output.stdout).expect("parse registry report");
    assert_eq!(report["worktree_id"], "candidate");
    assert_eq!(report["repository_id"], "terminal-components-claude");
    assert_eq!(report["mode"], "branch");
    assert_eq!(report["branch"], CANDIDATE_BRANCH);
    assert_eq!(report["clean"], true);
    assert_eq!(report["head_sha"].as_str().unwrap().len(), 40);
    assert_eq!(
        report["authority_root"],
        path_text(fixture.candidate_root())
    );
    assert_eq!(report["registry_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(
        report["visual_authority"]["tag_object_sha"],
        fixture.tag_object_sha
    );
    assert_eq!(
        report["visual_authority"]["peeled_commit_sha"],
        fixture.base_sha
    );
    assert!(
        !fixture
            .candidate_root()
            .join("docs/implementation/visibility/tasks.json")
            .exists(),
        "registry-check must not create accepted task records"
    );
    assert!(
        !fixture.candidate_root().join("WORK_QUEUE.md").exists(),
        "registry-check must not create the generated queue view"
    );
    assert_eq!(
        fs::read(&index_path).expect("read candidate Git index after preflight"),
        index_before,
        "registry-check must leave the Git index unchanged"
    );
}

#[test]
fn registry_check_rejects_visual_tag_identity_drift() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    registry["visual_authority"]["tag_object_sha"] =
        json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "visual tag object does not match the caller-pinned authority",
    );
}

#[test]
fn registry_check_rejects_retargeted_immutable_visual_tag() {
    let fixture = RegistryFixture::new();
    git_with_identity(
        fixture.candidate_root(),
        &[
            "tag",
            "--force",
            "--annotate",
            "visual-baseline",
            "--message",
            "retargeted test tag",
            "HEAD",
        ],
    );

    let output = fixture.run_check(&fixture.base_registry(), "candidate", None, None, &[]);
    assert_error(
        &output,
        "visual-baseline tag no longer matches the immutable frozen authority",
    );
}

#[test]
fn registry_check_binds_external_scratch_to_separate_clean_source_pin() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).expect("parse registry report");
    assert_eq!(report["worktree_id"], "velnor-scratch");
    assert_eq!(report["repository_id"], "velnor-new");
    assert_eq!(report["mode"], "scratch");
    assert_eq!(report["branch"], Value::Null);
    assert_eq!(report["head_sha"], external.head_sha);
    assert_eq!(report["clean"], true);
    assert_eq!(
        report["source_selection_review"]["commit_sha"],
        external.head_sha
    );
    assert_eq!(
        report["source_selection_review"]["tree_sha"],
        external.tree_sha
    );
    assert_eq!(
        report["source_selection_review"]["path_policy_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
}

#[test]
fn registry_check_rejects_source_selection_review_raw_digest_drift() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
        json!("0".repeat(64));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(
        &output,
        "source-selection review raw SHA-256 does not match the registry pin",
    );
}

#[test]
fn registry_schema_requires_external_source_selection_receipt_path() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let inside_source_pin = external.source_pin_root.join("external-fixture.txt");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"] =
        json!(path_text(&inside_source_pin));

    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "source-selection review records must be outside every registered worktree root",
    );
}

#[test]
fn registry_check_rejects_source_selection_policy_subject_drift() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let receipt_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    let mut receipt: Value = serde_json::from_slice(
        &fs::read(&receipt_path).expect("read source-selection fixture receipt"),
    )
    .expect("parse source-selection fixture receipt");
    receipt["subject"]["path_policy_sha256"] = json!("f".repeat(64));
    let raw = serde_json::to_vec(&receipt).expect("serialize altered source-selection receipt");
    fs::write(&receipt_path, &raw).expect("write altered source-selection receipt");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
        json!(sha256_hex(&raw));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(
        &output,
        "source-selection review subject path policy does not match the registered scratch allowlist",
    );
}

#[test]
fn registry_check_rejects_source_selection_commit_and_tree_drift() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let receipt_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    let original = fs::read(&receipt_path).expect("read source-selection fixture receipt");
    for (field, expected) in [
        (
            "commit_sha",
            "source-selection review subject commit_sha does not match the source pin",
        ),
        (
            "tree_sha",
            "source-selection review subject tree_sha does not match the source-pin Git tree",
        ),
    ] {
        let mut receipt: Value =
            serde_json::from_slice(&original).expect("parse source-selection fixture receipt");
        receipt["subject"][field] = json!("f".repeat(40));
        let raw = serde_json::to_vec(&receipt).expect("serialize altered source-selection receipt");
        fs::write(&receipt_path, &raw).expect("write altered source-selection receipt");
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
            json!(sha256_hex(&raw));

        let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
        assert_error(&output, expected);
    }
}

#[test]
fn registry_check_rejects_duplicate_keys_and_nonfinite_review_json() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let receipt_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    for (raw, expected) in [
        (
            br#"{"schema":"termrock-source-selection-review/v1","schema":"termrock-source-selection-review/v1"}"#.to_vec(),
            "JSON object repeats key 'schema'",
        ),
        (
            br#"{"schema":NaN}"#.to_vec(),
            "JSON contains unsupported non-finite number NaN",
        ),
    ] {
        fs::write(&receipt_path, &raw).expect("write malformed source-selection receipt");
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
            json!(sha256_hex(&raw));
        let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
        assert_error(&output, expected);
    }
}

#[test]
fn registry_check_limits_review_decision_to_source_selection() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let receipt_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    let mut receipt: Value = serde_json::from_slice(
        &fs::read(&receipt_path).expect("read source-selection fixture receipt"),
    )
    .expect("parse source-selection fixture receipt");
    receipt["decision"] = json!("approved_for_release");
    let raw = serde_json::to_vec(&receipt).expect("serialize altered source-selection receipt");
    fs::write(&receipt_path, &raw).expect("write altered source-selection receipt");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
        json!(sha256_hex(&raw));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(
        &output,
        "source-selection review decision must be approved_for_source_selection",
    );
}

#[test]
fn registry_check_rejects_unbounded_or_overlapping_scratch_allowlists() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut recursive = fixture.base_registry();
    external.add_to_registry(&mut recursive);
    recursive["worktrees"]["velnor-scratch"]["allowed_paths"] = json!(["external-fixture.txt/**"]);
    let output = fixture.run_check(&recursive, "candidate", None, None, &[]);
    assert_error(&output, "must use exact paths without recursive globs");

    let mut overlapping = fixture.base_registry();
    external.add_to_registry(&mut overlapping);
    overlapping["worktrees"]["velnor-scratch"]["allowed_paths"] =
        json!(["external-fixture.txt", "external-fixture.txt/child"]);
    let output = fixture.run_check(&overlapping, "candidate", None, None, &[]);
    assert_error(&output, "has overlapping paths");
}

#[test]
fn registry_check_rejects_oversized_source_selection_receipt() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let receipt_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    let oversized = vec![b'x'; 64 * 1024 + 1];
    fs::write(&receipt_path, &oversized).expect("write oversized source-selection receipt");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_sha256"] =
        json!(sha256_hex(&oversized));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(
        &output,
        "source-selection review record exceeds the 65536-byte limit",
    );
}

#[cfg(unix)]
#[test]
fn registry_check_rejects_symlinked_source_selection_receipt() {
    use std::os::unix::fs::symlink;

    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let original_path = PathBuf::from(
        registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"]
            .as_str()
            .unwrap(),
    );
    let alias = fixture.control.root().join("receipt-alias.json");
    symlink(&original_path, &alias).expect("create receipt leaf symlink");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"] =
        json!(path_text(&alias));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(&output, "cannot safely open source-selection review record");
}

#[cfg(unix)]
#[test]
fn registry_check_rejects_symlinked_source_selection_receipt_parent() {
    use std::os::unix::fs::symlink;

    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);
    let alias_dir = fixture.control.root().join("receipt-parent-alias");
    symlink(fixture.control.root(), &alias_dir).expect("create receipt parent symlink");
    let alias_path = alias_dir.join("source-selection-review.json");
    registry["worktrees"]["velnor-source-pin"]["source_selection_review"]["review_record_path"] =
        json!(path_text(&alias_path));

    let output = fixture.run_check(&registry, "velnor-scratch", None, None, &[]);
    assert_error(&output, "cannot safely open parent directory");
}

#[test]
fn registry_check_requires_caller_pinned_raw_registry_bytes() {
    let fixture = RegistryFixture::new();
    let output = fixture.run_check(
        &fixture.base_registry(),
        "candidate",
        None,
        Some(&"0".repeat(64)),
        &[],
    );
    assert_error(&output, "raw SHA-256 does not match the caller's pin");
}

#[test]
fn registry_check_binds_candidate_to_authority_root() {
    let fixture = RegistryFixture::new();
    let other_root = TempRepo::new().expect("create alternate authority root");
    let output = fixture.run_check(
        &fixture.base_registry(),
        "candidate",
        Some(other_root.root()),
        None,
        &[],
    );
    assert_error(&output, "candidate root does not exactly match --root");
}

#[test]
fn registry_check_requires_registry_path_outside_all_registered_worktrees() {
    let fixture = RegistryFixture::new();
    let external = ExternalWorktrees::new(fixture.control.root());
    let mut registry = fixture.base_registry();
    external.add_to_registry(&mut registry);

    for registry_path in [
        fixture.candidate.root().join("local-registry.json"),
        external.scratch_root.join("local-registry.json"),
    ] {
        let output = fixture.run_check_with_registry_path(&registry, "candidate", &registry_path);
        assert_error(
            &output,
            "root registry file must be outside every registered worktree root",
        );
    }
}

#[test]
fn registry_check_rejects_unknown_worktree_id() {
    let fixture = RegistryFixture::new();
    let output = fixture.run_check(&fixture.base_registry(), "unregistered", None, None, &[]);
    assert_error(&output, "unknown registered worktree unregistered");
}

#[test]
fn registry_check_verifies_reference_worktree_and_branch_base() {
    let fixture = RegistryFixture::new();
    let output = fixture.run_check(&fixture.base_registry(), "reference", None, None, &[]);
    assert_success(&output);

    let report: Value = serde_json::from_slice(&output.stdout).expect("parse registry report");
    assert_eq!(report["worktree_id"], "reference");
    assert_eq!(report["branch"], "visual-baseline");
    assert_eq!(report["head_sha"], fixture.base_sha);
    assert_eq!(report["candidate_worktree"]["branch"], CANDIDATE_BRANCH);
}

#[test]
fn registry_check_requires_reference_pin_reachable_from_origin_visual_baseline() {
    let fixture = RegistryFixture::new();
    let unrelated = git_output(fixture.candidate_root(), &["rev-parse", "--verify", "HEAD"]);
    git(
        fixture.candidate_root(),
        &[
            "update-ref",
            "refs/remotes/origin/visual-baseline",
            unrelated.as_str(),
        ],
    );

    let output = fixture.run_check(&fixture.base_registry(), "reference", None, None, &[]);
    assert_error(
        &output,
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b is not reachable from refs/remotes/origin/visual-baseline",
    );
}

#[test]
fn registry_schema_rejects_nested_worktree_roots() {
    let fixture = RegistryFixture::new();
    let mut nested_reference = fixture.base_registry();
    nested_reference["worktrees"]["reference"]["root"] = json!(path_text(
        &fixture.candidate.root().join("nested-reference")
    ));
    let output = fixture.run_check(&nested_reference, "candidate", None, None, &[]);
    assert_error(
        &output,
        "registered worktree roots must not contain one another",
    );

    let external = ExternalWorktrees::new(fixture.control.root());
    let mut nested_scratch = fixture.base_registry();
    external.add_to_registry(&mut nested_scratch);
    nested_scratch["worktrees"]["velnor-scratch"]["root"] =
        json!(path_text(&external.source_pin_root.join("nested-scratch")));
    let output = fixture.run_check(&nested_scratch, "candidate", None, None, &[]);
    assert_error(
        &output,
        "registered worktree roots must not contain one another",
    );
}

#[test]
fn registry_check_rejects_wrong_branch_and_wrong_origin() {
    let fixture = RegistryFixture::new();
    git(
        fixture.candidate_root(),
        &["checkout", "--quiet", "-b", "wrong-branch"],
    );
    let output = fixture.run_check(&fixture.base_registry(), "candidate", None, None, &[]);
    assert_error(&output, "is on the wrong branch");

    git(
        fixture.candidate_root(),
        &["checkout", "--quiet", CANDIDATE_BRANCH],
    );
    let mut wrong_origin = fixture.base_registry();
    wrong_origin["repositories"]["terminal-components-claude"]["origin_url"] = json!(
        fixture
            .control
            .root()
            .join("not-the-origin")
            .to_string_lossy()
            .to_string()
    );
    let output = fixture.run_check(&wrong_origin, "candidate", None, None, &[]);
    assert_error(
        &output,
        "origin does not match its canonical repository URL",
    );
}

#[test]
fn registry_check_rejects_dirty_worktree() {
    let fixture = RegistryFixture::new();
    fixture
        .candidate
        .write_file(Path::new("untracked.txt"), b"dirty\n")
        .expect("add untracked dirty file");
    let output = fixture.run_check(&fixture.base_registry(), "candidate", None, None, &[]);
    assert_error(&output, "must be clean");
}

#[test]
fn registry_check_ignores_inherited_git_repository_overrides() {
    let fixture = RegistryFixture::new();
    let git_dir = path_text(fixture.origin.root());
    let output = fixture.run_check(
        &fixture.base_registry(),
        "candidate",
        None,
        None,
        &[("GIT_DIR", git_dir.as_str())],
    );
    assert_success(&output);
}

#[cfg(unix)]
#[test]
fn registry_check_rejects_symlinked_candidate_root() {
    use std::os::unix::fs::symlink;

    let fixture = RegistryFixture::new();
    let alias = fixture.control.root().join("candidate-alias");
    symlink(fixture.candidate_root(), &alias).expect("create candidate root symlink");
    let mut registry = fixture.base_registry();
    registry["worktrees"]["candidate"]["root"] = json!(path_text(&alias));
    registry["worktrees"]["candidate"]["common_dir"] = json!(path_text(&alias.join(".git")));
    let output = fixture.run_check(&registry, "candidate", Some(&alias), None, &[]);
    assert_error(
        &output,
        "candidate root contains a symlink or is unavailable",
    );
}

#[cfg(unix)]
#[test]
fn registry_check_rejects_symlinked_common_directory() {
    use std::os::unix::fs::symlink;

    let fixture = RegistryFixture::new();
    let alias = fixture.control.root().join("common-dir-alias");
    symlink(fixture.candidate_root().join(".git"), &alias).expect("create common-dir symlink");
    let mut registry = fixture.base_registry();
    registry["worktrees"]["candidate"]["common_dir"] = json!(path_text(&alias));
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "root/common-dir contains a symlink or is unavailable",
    );
}

#[test]
fn registry_schema_rejects_unknown_worktree_field() {
    let fixture = RegistryFixture::new();
    let mut unknown = fixture.base_registry();
    unknown["worktrees"]["candidate"]["unexpected"] = json!(true);
    let output = fixture.run_check(&unknown, "candidate", None, None, &[]);
    assert_error(&output, "has unknown fields: unexpected");
}

#[test]
fn registry_schema_rejects_nontext_worktree_mode() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    registry["worktrees"]["candidate"]["mode"] = json!(["branch"]);
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(&output, ".mode must be branch, source-pin, or scratch");
}

#[test]
fn registry_schema_requires_candidate_and_reference_roots() {
    let fixture = RegistryFixture::new();
    let mut missing_reference = fixture.base_registry();
    missing_reference["worktrees"]
        .as_object_mut()
        .unwrap()
        .remove("reference");
    let output = fixture.run_check(&missing_reference, "candidate", None, None, &[]);
    assert_error(&output, "must bind candidate and reference worktrees");
}

#[test]
fn registry_schema_rejects_scratch_with_different_source_pin_sha() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    add_velnor_pair(
        &mut registry,
        "velnor-new",
        "https://example.invalid/velnor.git",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "separate root at its matching immutable source pin",
    );
}

#[test]
fn registry_schema_requires_source_pin_and_scratch_to_share_git_common_dir() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    add_velnor_pair(
        &mut registry,
        "velnor-new",
        "https://example.invalid/velnor.git",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    registry["worktrees"]["velnor-scratch"]["common_dir"] =
        json!("/private/tmp/unrelated-velnor-repository/.git");
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "separate root at its matching immutable source pin",
    );
}

#[test]
fn registry_schema_requires_velnor_source_pin_and_scratch_pair() {
    let fixture = RegistryFixture::new();
    let mut incomplete_velnor = fixture.base_registry();
    incomplete_velnor["worktrees"]["velnor-source-pin"] = json!({
        "repository_id": "velnor-new",
        "root": "/private/tmp/velnor-source-pin",
        "common_dir": "/private/tmp/velnor-source-pin/.git",
        "mode": "source-pin",
        "branch": null,
        "pinned_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "writable": false,
        "source_pin_id": null,
        "allowed_paths": [],
        "source_selection_review": {
            "review_record_path": "/private/tmp/source-selection-review.json",
            "review_record_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        }
    });
    incomplete_velnor["repositories"]["velnor-new"] =
        json!({"origin_url": "https://example.invalid/velnor.git"});
    let output = fixture.run_check(&incomplete_velnor, "candidate", None, None, &[]);
    assert_error(&output, "must include both source pin and isolated scratch");
}

#[test]
fn registry_schema_rejects_velnor_repository_alias_of_candidate() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    add_velnor_pair(
        &mut registry,
        "terminal-components-claude",
        "https://example.invalid/velnor.git",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "must use a repository identity and origin distinct from candidate",
    );
}

#[test]
fn registry_schema_rejects_velnor_origin_alias_of_candidate() {
    let fixture = RegistryFixture::new();
    let mut registry = fixture.base_registry();
    let candidate_origin = registry["repositories"]["terminal-components-claude"]["origin_url"]
        .as_str()
        .unwrap()
        .to_owned();
    add_velnor_pair(
        &mut registry,
        "velnor-new",
        &candidate_origin,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    let output = fixture.run_check(&registry, "candidate", None, None, &[]);
    assert_error(
        &output,
        "must use a repository identity and origin distinct from candidate",
    );
}

struct QueueV2Fixture {
    repository: TempRepo,
    script: PathBuf,
}

impl QueueV2Fixture {
    fn new() -> Self {
        let repository = TempRepo::new().expect("create read-only queue fixture root");
        let tool = repository
            .copy_tool_exact(Tool::Queue)
            .expect("copy exact queue CLI");
        Self {
            repository,
            script: tool.path,
        }
    }

    fn run(&self, tail: &[String]) -> CliOutput {
        let mut arguments = vec!["--root".to_owned(), path_text(self.repository.root())];
        arguments.extend_from_slice(tail);
        run_cli(&self.script, &arguments, self.repository.root(), &[], None)
            .expect("run read-only queue fixture command")
    }

    fn registry_path(&self) -> PathBuf {
        self.repository.root().join("control/registry.json")
    }

    fn base_role_registry(&self) -> Value {
        let root = self.repository.root();
        let termrock_common = path_text(&root.join("git/termrock.git"));
        json!({
            "schema_version": 2,
            "queue_authority": {
                "repository_id": "terminal-components-claude",
                "worktree_id": "queue-coordinator"
            },
            "repositories": {
                "terminal-components-claude": {
                    "origin_url": "https://github.com/tailrocks/terminal-components-claude.git"
                }
            },
            "worktrees": {
                "queue-coordinator": {
                    "role": "queue-coordinator",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&root.join("worktrees/queue-coordinator")),
                    "common_dir": termrock_common,
                    "mode": "branch",
                    "branch": CANDIDATE_BRANCH,
                    "pinned_sha": null,
                    "writable": true,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                },
                "candidate-source": {
                    "role": "candidate-source-authority",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&root.join("worktrees/candidate-source")),
                    "common_dir": termrock_common,
                    "mode": "source-pin",
                    "branch": null,
                    "pinned_sha": "cccccccccccccccccccccccccccccccccccccccc",
                    "writable": false,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                },
                "reference": {
                    "role": "reference-source",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&root.join("worktrees/reference")),
                    "common_dir": termrock_common,
                    "mode": "branch",
                    "branch": "visual-baseline",
                    "pinned_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "writable": false,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                }
            },
            "visual_authority": {
                "tag": "visual-baseline",
                "tag_object_sha": FROZEN_VISUAL_TAG_OBJECT_SHA,
                "peeled_commit_sha": FROZEN_VISUAL_TAG_COMMIT_SHA
            }
        })
    }

    fn add_velnor_pair(&self, registry: &mut Value) {
        let root = self.repository.root();
        let common_dir = path_text(&root.join("git/velnor.git"));
        let source_sha = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let review_path = root.join("control/source-selection-review.json");
        registry["repositories"]["velnor-new"] = json!({
            "origin_url": "https://github.com/tailrocks/velnor-new.git"
        });
        registry["worktrees"]["velnor-source-pin"] = json!({
            "role": "immutable-source-pin",
            "repository_id": "velnor-new",
            "root": path_text(&root.join("worktrees/velnor-source-pin")),
            "common_dir": common_dir,
            "mode": "source-pin",
            "branch": null,
            "pinned_sha": source_sha,
            "writable": false,
            "source_pin_id": null,
            "allowed_paths": [],
            "source_selection_review": {
                "review_record_path": path_text(&review_path),
                "review_record_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            }
        });
        registry["worktrees"]["velnor-scratch"] = json!({
            "role": "writable-isolated-scratch",
            "repository_id": "velnor-new",
            "root": path_text(&root.join("worktrees/velnor-scratch")),
            "common_dir": common_dir,
            "mode": "scratch",
            "branch": null,
            "pinned_sha": source_sha,
            "writable": true,
            "source_pin_id": "velnor-source-pin",
            "allowed_paths": ["src/lib.rs"],
            "source_selection_review": null
        });
    }

    fn validate_registry(&self, registry: &Value) -> CliOutput {
        let path = self.registry_path();
        fs::create_dir_all(path.parent().expect("registry parent"))
            .expect("create registry control directory");
        let raw = serde_json::to_vec(registry).expect("serialize role registry fixture");
        fs::write(&path, &raw).expect("write role registry fixture");
        self.run(&[
            "registry-validate-v2".to_owned(),
            "--registry".to_owned(),
            path_text(&path),
            "--registry-sha256".to_owned(),
            sha256_hex(&raw),
        ])
    }

    fn install_current_v1_queue(&self) -> (PathBuf, PathBuf, Vec<u8>, Vec<u8>) {
        let source_root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .expect("resolve source repository root");
        let source_tasks = fs::read(source_root.join("docs/implementation/visibility/tasks.json"))
            .expect("read checked-in accepted queue records");
        let source_view = fs::read(source_root.join("WORK_QUEUE.md"))
            .expect("read checked-in generated queue view");
        let tasks_path = self
            .repository
            .root()
            .join("docs/implementation/visibility/tasks.json");
        let view_path = self.repository.root().join("WORK_QUEUE.md");
        fs::create_dir_all(tasks_path.parent().expect("tasks parent"))
            .expect("create fixture task-record directory");
        fs::write(&tasks_path, &source_tasks).expect("copy accepted records to fixture");
        fs::write(&view_path, &source_view).expect("copy generated view to fixture");
        (tasks_path, view_path, source_tasks, source_view)
    }

    fn identity_map_args(&self) -> Vec<String> {
        vec![
            "migration-preview".to_owned(),
            "--repository-id".to_owned(),
            "terminal-components-claude".to_owned(),
            "--worktree-id".to_owned(),
            "queue-coordinator".to_owned(),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnapshotEntry {
    root_label: String,
    relative_path: String,
    kind: &'static str,
    mode: u32,
    bytes: Option<Vec<u8>>,
}

struct SnapshotRoot {
    label: String,
    path: PathBuf,
}

#[cfg(unix)]
fn snapshot_mode(metadata: &fs::Metadata) -> u32 {
    metadata.permissions().mode() & 0o7777
}

#[cfg(not(unix))]
fn snapshot_mode(metadata: &fs::Metadata) -> u32 {
    u32::from(metadata.permissions().readonly())
}

#[cfg(unix)]
fn snapshot_path_bytes(path: &Path) -> Vec<u8> {
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn snapshot_path_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}

fn collect_snapshot_tree(
    root: &SnapshotRoot,
    path: &Path,
    entries: &mut Vec<SnapshotEntry>,
) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    let (kind, bytes) = if file_type.is_symlink() {
        ("symlink", Some(snapshot_path_bytes(&fs::read_link(path)?)))
    } else if file_type.is_file() {
        ("file", Some(fs::read(path)?))
    } else if file_type.is_dir() {
        ("directory", None)
    } else {
        ("special", None)
    };
    let relative = path
        .strip_prefix(&root.path)
        .expect("snapshot path stays below its root");
    let relative_path = if relative.as_os_str().is_empty() {
        ".".to_owned()
    } else {
        relative.to_string_lossy().into_owned()
    };
    entries.push(SnapshotEntry {
        root_label: root.label.clone(),
        relative_path,
        kind,
        mode: snapshot_mode(&metadata),
        bytes,
    });

    if file_type.is_dir() {
        let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(|child| child.file_name());
        for child in children {
            collect_snapshot_tree(root, &child.path(), entries)?;
        }
    }
    Ok(())
}

fn snapshot_entry<'a>(
    entries: &'a [SnapshotEntry],
    root_label: &str,
    relative_path: &str,
) -> Option<&'a SnapshotEntry> {
    entries
        .iter()
        .find(|entry| entry.root_label == root_label && entry.relative_path == relative_path)
}

fn snapshot_entry_summary(entry: Option<&SnapshotEntry>) -> String {
    match entry {
        Some(entry) => {
            let content = match &entry.bytes {
                Some(bytes) => format!("{} bytes sha256={}", bytes.len(), sha256_hex(bytes)),
                None => "no byte payload".to_owned(),
            };
            format!(
                "{}/{} type={} mode={:04o} {}",
                entry.root_label, entry.relative_path, entry.kind, entry.mode, content
            )
        }
        None => "absent".to_owned(),
    }
}

struct RegistryCheckV2Fixture {
    base: RegistryFixture,
    candidate_source_root: PathBuf,
    candidate_source_sha: String,
    common_dir: PathBuf,
    script: PathBuf,
    git_dirs: Vec<(&'static str, PathBuf)>,
    index_paths: Vec<(&'static str, PathBuf)>,
    registry_path: PathBuf,
}

impl RegistryCheckV2Fixture {
    fn new() -> Self {
        let base = RegistryFixture::new();
        let tasks_path = base
            .candidate_root()
            .join("docs/implementation/visibility/tasks.json");
        fs::create_dir_all(tasks_path.parent().expect("tasks parent"))
            .expect("create coordinator task-record directory");
        fs::write(
            &tasks_path,
            b"{\"schema_version\":1,\"queue_revision\":0,\"tasks\":[]}\n",
        )
        .expect("write coordinator task-record fixture");
        fs::write(
            base.candidate_root().join("WORK_QUEUE.md"),
            b"# Fixture queue\n",
        )
        .expect("write coordinator queue-view fixture");
        git(base.candidate_root(), &["add", "--all"]);
        git_with_identity(
            base.candidate_root(),
            &[
                "commit",
                "--quiet",
                "-m",
                "registry check v2 authority fixture",
            ],
        );
        git(
            base.candidate_root(),
            &[
                "remote",
                "set-url",
                "origin",
                "https://github.com/tailrocks/terminal-components-claude.git",
            ],
        );

        let candidate_source_sha =
            git_output(base.candidate_root(), &["rev-parse", "--verify", "HEAD"]);
        let candidate_source_root = base.control.root().join("candidate-source-worktree");
        let candidate_source_path = path_text(&candidate_source_root);
        git(
            base.candidate_root(),
            &[
                "worktree",
                "add",
                "--quiet",
                "--detach",
                candidate_source_path.as_str(),
                candidate_source_sha.as_str(),
            ],
        );

        let common_dir = PathBuf::from(git_output(
            base.candidate_root(),
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        ));
        let script = candidate_source_root.join("tools/visibility/queue.py");
        let worktrees = [
            ("coordinator", base.candidate_root()),
            ("candidate-source", candidate_source_root.as_path()),
            ("reference", base.reference_root.as_path()),
        ];
        let mut git_dirs = Vec::new();
        let mut index_paths = Vec::new();
        for (role, root) in worktrees {
            git_dirs.push((
                role,
                PathBuf::from(git_output(
                    root,
                    &["rev-parse", "--path-format=absolute", "--git-dir"],
                )),
            ));
            index_paths.push((
                role,
                PathBuf::from(git_output(
                    root,
                    &["rev-parse", "--path-format=absolute", "--git-path", "index"],
                )),
            ));
        }

        let registry_path = base.control.root().join("registry-control/registry.json");
        Self {
            base,
            candidate_source_root,
            candidate_source_sha,
            common_dir,
            script,
            git_dirs,
            index_paths,
            registry_path,
        }
    }

    fn base_registry(&self) -> Value {
        let common_dir = path_text(&self.common_dir);
        let origin_url = "https://github.com/tailrocks/terminal-components-claude.git";
        json!({
            "schema_version": 2,
            "queue_authority": {
                "repository_id": "terminal-components-claude",
                "worktree_id": "queue-coordinator"
            },
            "repositories": {
                "terminal-components-claude": {"origin_url": origin_url}
            },
            "worktrees": {
                "queue-coordinator": {
                    "role": "queue-coordinator",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(self.base.candidate_root()),
                    "common_dir": common_dir,
                    "mode": "branch",
                    "branch": CANDIDATE_BRANCH,
                    "pinned_sha": null,
                    "writable": true,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                },
                "candidate-source": {
                    "role": "candidate-source-authority",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&self.candidate_source_root),
                    "common_dir": common_dir,
                    "mode": "source-pin",
                    "branch": null,
                    "pinned_sha": self.candidate_source_sha,
                    "writable": false,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                },
                "reference": {
                    "role": "reference-source",
                    "repository_id": "terminal-components-claude",
                    "root": path_text(&self.base.reference_root),
                    "common_dir": common_dir,
                    "mode": "branch",
                    "branch": "visual-baseline",
                    "pinned_sha": self.base.base_sha,
                    "writable": false,
                    "source_pin_id": null,
                    "allowed_paths": [],
                    "source_selection_review": null
                }
            },
            "visual_authority": {
                "tag": "visual-baseline",
                "tag_object_sha": FROZEN_VISUAL_TAG_OBJECT_SHA,
                "peeled_commit_sha": FROZEN_VISUAL_TAG_COMMIT_SHA
            }
        })
    }

    fn add_optional_velnor_pair(&self, registry: &mut Value) {
        let optional_root = self.base.control.root().join("not-created/velnor");
        let source_root = optional_root.join("source-pin");
        let scratch_root = optional_root.join("scratch");
        let common_dir = optional_root.join("velnor.git");
        let review_path = self
            .base
            .control
            .root()
            .join("not-created/source-review.json");
        let pinned_sha = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        registry["repositories"]["velnor-new"] = json!({
            "origin_url": "https://github.com/tailrocks/velnor-new.git"
        });
        registry["worktrees"]["velnor-source-pin"] = json!({
            "role": "immutable-source-pin",
            "repository_id": "velnor-new",
            "root": path_text(&source_root),
            "common_dir": path_text(&common_dir),
            "mode": "source-pin",
            "branch": null,
            "pinned_sha": pinned_sha,
            "writable": false,
            "source_pin_id": null,
            "allowed_paths": [],
            "source_selection_review": {
                "review_record_path": path_text(&review_path),
                "review_record_sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            }
        });
        registry["worktrees"]["velnor-scratch"] = json!({
            "role": "writable-isolated-scratch",
            "repository_id": "velnor-new",
            "root": path_text(&scratch_root),
            "common_dir": path_text(&common_dir),
            "mode": "scratch",
            "branch": null,
            "pinned_sha": pinned_sha,
            "writable": true,
            "source_pin_id": "velnor-source-pin",
            "allowed_paths": ["src/lib.rs"],
            "source_selection_review": null
        });
    }

    fn snapshot_roots(&self) -> Vec<SnapshotRoot> {
        let mut roots = vec![
            SnapshotRoot {
                label: "coordinator-root".to_owned(),
                path: self.base.candidate_root().to_path_buf(),
            },
            SnapshotRoot {
                label: "candidate-source-root".to_owned(),
                path: self.candidate_source_root.clone(),
            },
            SnapshotRoot {
                label: "reference-root".to_owned(),
                path: self.base.reference_root.clone(),
            },
            SnapshotRoot {
                label: "common-git".to_owned(),
                path: self.common_dir.clone(),
            },
            SnapshotRoot {
                label: "bare-origin".to_owned(),
                path: self.base.origin.root().to_path_buf(),
            },
            SnapshotRoot {
                label: "external-control".to_owned(),
                path: self.base.control.root().to_path_buf(),
            },
        ];
        for (role, path) in &self.git_dirs {
            roots.push(SnapshotRoot {
                label: format!("gitdir-{role}"),
                path: path.clone(),
            });
        }
        roots
    }

    fn snapshot(&self) -> Vec<SnapshotEntry> {
        let mut entries = Vec::new();
        for root in self.snapshot_roots() {
            collect_snapshot_tree(&root, &root.path, &mut entries)
                .expect("capture complete read-only registry fixture envelope");
        }
        entries.sort_by(|left, right| {
            (&left.root_label, &left.relative_path).cmp(&(&right.root_label, &right.relative_path))
        });
        entries
    }

    fn assert_read_only(&self, before: &[SnapshotEntry], after: &[SnapshotEntry]) {
        let compare_path = |label: &str, path: &str, description: &str| {
            let before_entry = snapshot_entry(before, label, path);
            let after_entry = snapshot_entry(after, label, path);
            assert!(
                before_entry == after_entry,
                "{description} changed: before {}; after {}",
                snapshot_entry_summary(before_entry),
                snapshot_entry_summary(after_entry)
            );
        };

        for (label, path) in [
            (
                "coordinator-root",
                "docs/implementation/visibility/tasks.json",
            ),
            ("coordinator-root", "WORK_QUEUE.md"),
            ("coordinator-root", ".git"),
            ("candidate-source-root", ".git"),
            ("reference-root", ".git"),
            ("common-git", "."),
        ] {
            compare_path(label, path, "queue authority or Git root metadata");
        }

        let registry_relative = self
            .registry_path
            .strip_prefix(self.base.control.root())
            .expect("registry path is inside the external control root")
            .to_string_lossy()
            .into_owned();
        compare_path(
            "external-control",
            &registry_relative,
            "external registry bytes",
        );

        for (role, index_path) in &self.index_paths {
            let relative = index_path
                .strip_prefix(&self.common_dir)
                .expect("linked-worktree index is inside the common Git directory")
                .to_string_lossy()
                .into_owned();
            compare_path("common-git", &relative, &format!("{role} index"));
        }

        for (role, _git_dir) in &self.git_dirs {
            let label = format!("gitdir-{role}");
            compare_path(&label, "HEAD", &format!("{role} worktree HEAD"));
            compare_path(&label, "gitdir", &format!("{role} membership record"));
        }

        fn ref_entries<'a>(snapshot: &'a [SnapshotEntry]) -> Vec<&'a SnapshotEntry> {
            snapshot
                .iter()
                .filter(|entry| {
                    entry.root_label == "common-git"
                        && (entry.relative_path == "packed-refs"
                            || entry.relative_path.starts_with("refs/"))
                })
                .collect::<Vec<_>>()
        }
        assert!(
            ref_entries(before) == ref_entries(after),
            "common Git ref inventory or packed-refs changed"
        );

        assert_snapshot_equal(before, after);
    }

    fn write_registry(&self, registry: &Value, path: &Path) -> String {
        fs::create_dir_all(path.parent().expect("registry parent"))
            .expect("create registry parent directory");
        let raw = serde_json::to_vec(registry).expect("serialize v2 live registry fixture");
        fs::write(path, &raw).expect("write v2 live registry fixture");
        sha256_hex(&raw)
    }

    fn run_check(
        &self,
        registry: &Value,
        worktree_id: &str,
        registry_path_override: Option<&Path>,
        registry_sha256_override: Option<&str>,
        root_override: Option<&Path>,
        environment: &[(&str, &str)],
    ) -> CliOutput {
        let registry_path = registry_path_override.unwrap_or(&self.registry_path);
        let registry_sha256 = self.write_registry(registry, registry_path);
        let root = root_override.unwrap_or(self.base.candidate_root());
        let arguments = vec![
            "--root".to_owned(),
            path_text(root),
            "registry-check-v2".to_owned(),
            "--registry".to_owned(),
            path_text(registry_path),
            "--registry-sha256".to_owned(),
            registry_sha256_override
                .unwrap_or(registry_sha256.as_str())
                .to_owned(),
            "--worktree".to_owned(),
            worktree_id.to_owned(),
        ];
        let before = self.snapshot();
        let mut child_environment = vec![("PYTHONDONTWRITEBYTECODE", "1")];
        child_environment.extend_from_slice(environment);
        let output = run_cli(
            &self.script,
            &arguments,
            self.base.candidate_root(),
            &child_environment,
            None,
        );
        let after = self.snapshot();
        self.assert_read_only(&before, &after);
        output.expect("run registry-check-v2 CLI")
    }

    fn run_validate(&self, registry: &Value) -> CliOutput {
        let registry_sha256 = self.write_registry(registry, &self.registry_path);
        let arguments = vec![
            "--root".to_owned(),
            path_text(self.base.candidate_root()),
            "registry-validate-v2".to_owned(),
            "--registry".to_owned(),
            path_text(&self.registry_path),
            "--registry-sha256".to_owned(),
            registry_sha256,
        ];
        let before = self.snapshot();
        let output = run_cli(
            &self.script,
            &arguments,
            self.base.candidate_root(),
            &[("PYTHONDONTWRITEBYTECODE", "1")],
            None,
        );
        let after = self.snapshot();
        self.assert_read_only(&before, &after);
        output.expect("run registry-validate-v2 CLI")
    }
}

fn assert_snapshot_equal(before: &[SnapshotEntry], after: &[SnapshotEntry]) {
    if before == after {
        return;
    }
    let difference = before
        .iter()
        .zip(after)
        .find(|(left, right)| left != right)
        .map(|(left, right)| {
            format!(
                "before {}; after {}",
                snapshot_entry_summary(Some(left)),
                snapshot_entry_summary(Some(right))
            )
        })
        .unwrap_or_else(|| {
            let index = before.len().min(after.len());
            format!(
                "entry count changed from {} to {}; first extra entry: {}",
                before.len(),
                after.len(),
                snapshot_entry_summary(before.get(index).or_else(|| after.get(index)))
            )
        });
    panic!("read-only registry command changed the fixture envelope: {difference}");
}

#[test]
fn registry_v2_validation_accepts_distinct_authority_roles() {
    let fixture = QueueV2Fixture::new();
    let mut registry = fixture.base_role_registry();
    fixture.add_velnor_pair(&mut registry);
    let output = fixture.validate_registry(&registry);
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).expect("parse v2 role report");
    assert_eq!(report["schema_version"], 2);
    assert_eq!(
        report["queue_authority"]["worktree_id"],
        "queue-coordinator"
    );
    assert_eq!(report["live_roots_qualified"], false);
    assert_eq!(
        report["worktree_ids"],
        json!([
            "candidate-source",
            "queue-coordinator",
            "reference",
            "velnor-scratch",
            "velnor-source-pin"
        ])
    );
    assert!(
        !fixture
            .repository
            .root()
            .join("worktrees/queue-coordinator")
            .exists(),
        "schema validation must not create a registered worktree"
    );

    registry["repositories"]["terminal-components-claude"]["origin_url"] =
        json!("ssh://git@github.com/tailrocks/terminal-components-claude.git");
    assert_success(&fixture.validate_registry(&registry));
}

#[test]
fn registry_v2_validation_rejects_role_path_origin_mode_and_pin_errors() {
    let fixture = QueueV2Fixture::new();
    let mut registry = fixture.base_role_registry();
    let duplicate_root = registry["worktrees"]["queue-coordinator"]["root"].clone();
    registry["worktrees"]["candidate-source"]["root"] = duplicate_root;
    assert_error(
        &fixture.validate_registry(&registry),
        "registered worktree roots must be disjoint",
    );

    let mut registry = fixture.base_role_registry();
    let coordinator_root = registry["worktrees"]["queue-coordinator"]["root"]
        .as_str()
        .expect("coordinator root")
        .to_owned();
    registry["worktrees"]["candidate-source"]["root"] =
        json!(format!("{coordinator_root}/nested-source"));
    assert_error(
        &fixture.validate_registry(&registry),
        "registered worktree roots must be disjoint",
    );

    let mut registry = fixture.base_role_registry();
    registry["worktrees"]["candidate-source"]["role"] = json!("queue-coordinator");
    assert_error(
        &fixture.validate_registry(&registry),
        "role does not match its registered worktree identity",
    );

    let mut registry = fixture.base_role_registry();
    registry["worktrees"]["candidate-source"]["mode"] = json!("branch");
    assert_error(
        &fixture.validate_registry(&registry),
        "candidate-source-authority must be a pinned read-only detached worktree",
    );

    let mut registry = fixture.base_role_registry();
    registry["worktrees"]["candidate-source"]["pinned_sha"] = json!("not-a-sha");
    assert_error(
        &fixture.validate_registry(&registry),
        "pinned_sha must be a full lowercase SHA-1 or null",
    );

    let mut registry = fixture.base_role_registry();
    registry["worktrees"]["candidate-source"]["root"] = json!("../candidate-source");
    assert_error(&fixture.validate_registry(&registry), "must be absolute");

    let mut registry = fixture.base_role_registry();
    registry["repositories"]["terminal-components-claude"]["origin_url"] =
        json!("https:///missing-host/repository.git");
    assert_error(
        &fixture.validate_registry(&registry),
        "must be an HTTP(S), SSH, or scp-style repository origin",
    );
}

#[test]
fn registry_v2_validation_rejects_velnor_aliases_and_unsafe_scratch_paths() {
    let fixture = QueueV2Fixture::new();
    let mut registry = fixture.base_role_registry();
    fixture.add_velnor_pair(&mut registry);
    let scratch_root = registry["worktrees"]["velnor-scratch"]["root"].clone();
    registry["worktrees"]["velnor-source-pin"]["root"] = scratch_root;
    assert_error(
        &fixture.validate_registry(&registry),
        "registered worktree roots must be disjoint",
    );

    let mut registry = fixture.base_role_registry();
    fixture.add_velnor_pair(&mut registry);
    registry["worktrees"]["velnor-scratch"]["pinned_sha"] =
        json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
    assert_error(
        &fixture.validate_registry(&registry),
        "matching immutable source pin",
    );

    let mut registry = fixture.base_role_registry();
    fixture.add_velnor_pair(&mut registry);
    let candidate_origin =
        registry["repositories"]["terminal-components-claude"]["origin_url"].clone();
    registry["repositories"]["velnor-new"]["origin_url"] = candidate_origin;
    assert_error(
        &fixture.validate_registry(&registry),
        "repository identity and origin distinct from candidate",
    );

    let mut registry = fixture.base_role_registry();
    fixture.add_velnor_pair(&mut registry);
    registry["worktrees"]["velnor-scratch"]["allowed_paths"] = json!(["../src/lib.rs"]);
    assert_error(
        &fixture.validate_registry(&registry),
        "allowed path traverses its root",
    );
}

#[test]
fn registry_v2_validation_requires_caller_pinned_raw_bytes() {
    let fixture = QueueV2Fixture::new();
    let registry = fixture.base_role_registry();
    let path = fixture.registry_path();
    fs::create_dir_all(path.parent().expect("registry parent"))
        .expect("create registry control directory");
    let raw = serde_json::to_vec(&registry).expect("serialize role registry fixture");
    fs::write(&path, raw).expect("write role registry fixture");
    let output = fixture.run(&[
        "registry-validate-v2".to_owned(),
        "--registry".to_owned(),
        path_text(&path),
        "--registry-sha256".to_owned(),
        "0".repeat(64),
    ]);
    assert_error(
        &output,
        "root registry raw SHA-256 does not match the caller's pin",
    );
}

#[test]
fn current_accepted_records_preview_to_v2_without_writing_and_preserve_history() {
    let fixture = QueueV2Fixture::new();
    let (tasks_path, view_path, source_tasks, source_view) = fixture.install_current_v1_queue();
    assert_success(&fixture.run(&["check".to_owned()]));
    let v1_render = fixture.run(&["render".to_owned()]);
    assert_success(&v1_render);
    assert_eq!(
        v1_render.stdout, source_view,
        "schema-v1 render behavior must remain unchanged"
    );

    let source_records: Value =
        serde_json::from_slice(&source_tasks).expect("parse copied checked-in records");
    let output = fixture.run(&fixture.identity_map_args());
    assert_success(&output);
    let preview: Value = serde_json::from_slice(&output.stdout).expect("parse migration preview");
    assert_eq!(
        preview["preview_schema"],
        "termrock-visibility-migration-preview/v1"
    );
    assert_eq!(
        preview["source"]["queue_revision"],
        source_records["queue_revision"]
    );
    assert_eq!(preview["source"]["tasks_sha256"], sha256_hex(&source_tasks));
    assert_eq!(
        preview["source"]["work_queue_sha256"],
        sha256_hex(&source_view)
    );

    let mut invalid_mapping = fixture.identity_map_args();
    invalid_mapping[2] = "Invalid-Repository".to_owned();
    assert_error(
        &fixture.run(&invalid_mapping),
        "must be a valid registry ID",
    );

    let mut expected = source_records.clone();
    expected["schema_version"] = json!(2);
    for task in expected["tasks"].as_array_mut().expect("task array") {
        task["repository_id"] = json!("terminal-components-claude");
        task["worktree_id"] = json!("queue-coordinator");
        if let Some(history) = task.get_mut("claim_history").and_then(Value::as_array_mut) {
            for entry in history {
                entry["claim"]["repository_id"] = json!("terminal-components-claude");
                entry["claim"]["worktree_id"] = json!("queue-coordinator");
            }
        }
    }
    assert_eq!(
        preview["records"], expected,
        "preview must preserve all old values"
    );
    assert_eq!(
        fs::read(&tasks_path).expect("read task source after preview"),
        source_tasks
    );
    assert_eq!(
        fs::read(&view_path).expect("read view source after preview"),
        source_view
    );

    let mut missing_task_identity = expected.clone();
    missing_task_identity["tasks"][0]
        .as_object_mut()
        .expect("task object")
        .remove("repository_id");
    let invalid_bytes = serde_json::to_vec_pretty(&missing_task_identity)
        .expect("serialize v2 record without task identity");
    fs::write(&tasks_path, invalid_bytes).expect("write invalid fixture v2 records");
    assert_error(
        &fixture.run(&["render-v2".to_owned()]),
        "missing fields: repository_id",
    );

    let history_task = expected["tasks"]
        .as_array()
        .expect("task array")
        .iter()
        .position(|task| {
            task.get("claim_history")
                .and_then(Value::as_array)
                .is_some_and(|history| !history.is_empty())
        })
        .expect("accepted queue contains a claim history snapshot");
    let mut missing_snapshot_identity = expected.clone();
    missing_snapshot_identity["tasks"][history_task]["claim_history"][0]["claim"]
        .as_object_mut()
        .expect("claim snapshot object")
        .remove("worktree_id");
    let invalid_bytes = serde_json::to_vec_pretty(&missing_snapshot_identity)
        .expect("serialize v2 record without snapshot identity");
    fs::write(&tasks_path, invalid_bytes).expect("write invalid fixture history snapshot");
    assert_error(
        &fixture.run(&["render-v2".to_owned()]),
        "claim snapshot has missing or unknown fields",
    );

    let rendered = preview["rendered_queue"]
        .as_str()
        .expect("preview contains deterministic v2 view")
        .as_bytes()
        .to_vec();
    let mut v2_bytes = serde_json::to_vec_pretty(&expected).expect("serialize expected v2 records");
    v2_bytes.push(b'\n');
    fs::write(&tasks_path, v2_bytes).expect("install v2 records in disposable fixture");
    fs::write(&view_path, &rendered).expect("install v2 view in disposable fixture");
    assert_success(&fixture.run(&["check-v2".to_owned()]));
    let render = fixture.run(&["render-v2".to_owned()]);
    assert_success(&render);
    assert_eq!(
        render.stdout, rendered,
        "v2 rendering must be deterministic"
    );
}

#[test]
fn schema_v2_active_path_conflicts_are_repository_scoped() {
    let fixture = QueueV2Fixture::new();
    let (_tasks_path, _view_path, source_tasks, _source_view) = fixture.install_current_v1_queue();
    let preview = fixture.run(&fixture.identity_map_args());
    assert_success(&preview);
    let preview: Value = serde_json::from_slice(&preview.stdout).expect("parse migration preview");
    let mut records = preview["records"].clone();
    let active_ids = {
        let tasks = records["tasks"].as_array_mut().expect("task array");
        let active_ids: Vec<usize> = tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                matches!(
                    task["state"].as_str(),
                    Some("claimed" | "in_progress" | "review" | "blocked")
                )
            })
            .map(|(index, _)| index)
            .take(2)
            .collect();
        assert_eq!(
            active_ids.len(),
            2,
            "fixture needs two active accepted tasks"
        );
        for index in &active_ids {
            let task = &mut tasks[*index];
            task["allowed_paths"] = json!(["shared/fixture.rs"]);
            if let Some(history) = task.get_mut("claim_history").and_then(Value::as_array_mut) {
                for entry in history {
                    entry["claim"]["allowed_paths"] = json!(["shared/fixture.rs"]);
                }
            }
        }
        let second = active_ids[1];
        tasks[second]["worktree_id"] = json!("reference");
        if let Some(history) = tasks[second]
            .get_mut("claim_history")
            .and_then(Value::as_array_mut)
        {
            for entry in history {
                entry["claim"]["worktree_id"] = json!("reference");
            }
        }
        active_ids
    };

    let records_path = fixture
        .repository
        .root()
        .join("docs/implementation/visibility/tasks.json");
    let view_path = fixture.repository.root().join("WORK_QUEUE.md");
    let same_repository =
        serde_json::to_vec_pretty(&records).expect("serialize overlapping records");
    fs::write(&records_path, &same_repository).expect("write same-repository fixture records");
    assert_error(
        &fixture.run(&["render-v2".to_owned()]),
        "active path conflict",
    );

    let second = active_ids[1];
    {
        let tasks = records["tasks"].as_array_mut().expect("task array");
        tasks[second]["repository_id"] = json!("separate-repository");
        tasks[second]["worktree_id"] = json!("velnor-scratch");
        if let Some(history) = tasks[second]
            .get_mut("claim_history")
            .and_then(Value::as_array_mut)
        {
            for entry in history {
                entry["claim"]["repository_id"] = json!("separate-repository");
                entry["claim"]["worktree_id"] = json!("velnor-scratch");
            }
        }
    }
    let distinct_repository =
        serde_json::to_vec_pretty(&records).expect("serialize independent repository records");
    fs::write(&records_path, distinct_repository).expect("write independent fixture records");
    let rendered = fixture.run(&["render-v2".to_owned()]);
    assert_success(&rendered);
    fs::write(&view_path, &rendered.stdout).expect("write generated v2 view");
    assert_success(&fixture.run(&["check-v2".to_owned()]));

    let source_root = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .expect("resolve source repository root");
    assert_eq!(
        fs::read(source_root.join("docs/implementation/visibility/tasks.json"))
            .expect("read source records after fixture checks"),
        source_tasks,
        "Rust fixture checks must not alter checked-in accepted records"
    );
}

#[test]
fn registry_check_v2_binds_raw_registry_bytes_and_is_read_only() {
    let fixture = RegistryCheckV2Fixture::new();
    let registry = fixture.base_registry();
    let inherited_git_dir = path_text(fixture.base.origin.root());
    let environment = [
        ("GIT_DIR", inherited_git_dir.as_str()),
        ("GIT_OPTIONAL_LOCKS", "1"),
        ("GIT_NO_LAZY_FETCH", "0"),
        ("GIT_NO_REPLACE_OBJECTS", "0"),
    ];
    let output = fixture.run_check(
        &registry,
        "candidate-source",
        None,
        None,
        None,
        &environment,
    );
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).expect("parse registry observation");
    assert_eq!(
        report["schema"],
        "termrock-visibility-registry-observation/v1"
    );
    assert_eq!(report["registry_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(
        report["queue_authority"]["worktree_id"],
        "queue-coordinator"
    );
    assert_eq!(
        report["queue_authority"]["root"],
        path_text(fixture.base.candidate_root())
    );
    assert_eq!(
        report["selected_worktree"]["worktree_id"],
        "candidate-source"
    );
    assert_eq!(
        report["selected_worktree"]["role"],
        "candidate-source-authority"
    );
    assert_eq!(report["selected_worktree"]["identity_matches"], true);
    assert_eq!(report["selected_worktree"]["identity_qualified"], true);
    assert_eq!(report["selected_worktree"]["clean"], true);
    assert_eq!(report["coordinator_observation"]["clean"], true);
    assert_eq!(
        report["coordinator_observation"]["identity_qualified"],
        true
    );
    assert_eq!(
        report["visual_authority"]["tag_object_sha"],
        FROZEN_VISUAL_TAG_OBJECT_SHA
    );
    assert_eq!(
        report["visual_authority"]["peeled_commit_sha"],
        FROZEN_VISUAL_TAG_COMMIT_SHA
    );
    assert_eq!(report["optional_roles"]["velnor-source-pin"], "not_probed");
    assert_eq!(report["optional_roles"]["velnor-scratch"], "not_probed");
    assert_eq!(report["limits"]["read_only_observation"], true);
    assert_eq!(report["limits"]["unselected_roles_qualified"], false);
    assert_eq!(report["limits"]["writer_capability"], false);
    assert_eq!(report["limits"]["actor_authenticated"], false);
    assert_eq!(report["limits"]["source_selection_decision"], false);
    assert_eq!(report["limits"]["build_or_product_qualification"], false);
    assert!(report.get("live_roots_qualified").is_none());

    let stale = fixture.run_check(
        &registry,
        "candidate-source",
        None,
        Some(&"0".repeat(64)),
        None,
        &[],
    );
    assert_error(
        &stale,
        "root registry raw SHA-256 does not match the caller's pin",
    );
}

#[test]
fn registry_check_v2_reports_dirty_coordinator_separately_from_clean_source() {
    let fixture = RegistryCheckV2Fixture::new();
    fixture
        .base
        .candidate
        .write_file(
            Path::new("untracked-coordinator.txt"),
            b"dirty coordinator\n",
        )
        .expect("make coordinator dirty without changing source worktree");
    let registry = fixture.base_registry();

    let source_output = fixture.run_check(&registry, "candidate-source", None, None, None, &[]);
    assert_success(&source_output);
    let source_report: Value =
        serde_json::from_slice(&source_output.stdout).expect("parse source observation");
    assert_eq!(source_report["selected_worktree"]["clean"], true);
    assert_eq!(
        source_report["selected_worktree"]["identity_qualified"],
        true
    );
    assert_eq!(
        source_report["coordinator_observation"]["identity_matches"],
        true
    );
    assert_eq!(source_report["coordinator_observation"]["clean"], false);
    assert_eq!(
        source_report["coordinator_observation"]["identity_qualified"],
        false
    );

    let coordinator_output =
        fixture.run_check(&registry, "queue-coordinator", None, None, None, &[]);
    assert_success(&coordinator_output);
    let coordinator_report: Value =
        serde_json::from_slice(&coordinator_output.stdout).expect("parse coordinator observation");
    assert_eq!(
        coordinator_report["selected_worktree"]["identity_matches"],
        true
    );
    assert_eq!(coordinator_report["selected_worktree"]["clean"], false);
    assert_eq!(
        coordinator_report["selected_worktree"]["identity_qualified"],
        false
    );
    assert!(coordinator_report.get("coordinator_observation").is_none());
}

#[test]
fn registry_check_v2_checks_candidate_pin_reference_ancestry_and_tracking_sha() {
    let fixture = RegistryCheckV2Fixture::new();
    let registry = fixture.base_registry();
    let source_output = fixture.run_check(&registry, "candidate-source", None, None, None, &[]);
    assert_success(&source_output);
    let source_report: Value =
        serde_json::from_slice(&source_output.stdout).expect("parse candidate-source observation");
    assert_eq!(
        source_report["selected_worktree"]["head_sha"],
        fixture.candidate_source_sha
    );
    assert_eq!(
        source_report["selected_worktree"]["pinned_sha"],
        fixture.candidate_source_sha
    );
    assert_eq!(source_report["selected_worktree"]["branch"], Value::Null);

    let reference_output = fixture.run_check(&registry, "reference", None, None, None, &[]);
    assert_success(&reference_output);
    let reference_report: Value =
        serde_json::from_slice(&reference_output.stdout).expect("parse reference observation");
    assert_eq!(
        reference_report["selected_worktree"]["branch"],
        "visual-baseline"
    );
    assert_eq!(
        reference_report["selected_worktree"]["head_sha"],
        fixture.base.base_sha
    );
    assert_eq!(
        reference_report["selected_worktree"]["tracking_ref_sha"],
        fixture.base.base_sha
    );
    assert_eq!(
        reference_report["selected_worktree"]["pinned_sha_is_ancestor"],
        true
    );
}

#[test]
fn registry_check_v2_rejects_wrong_remote_branch_head_and_common_dir() {
    let fixture = RegistryCheckV2Fixture::new();
    let mut wrong_origin = fixture.base_registry();
    wrong_origin["repositories"]["terminal-components-claude"]["origin_url"] =
        json!("https://example.invalid/wrong/repository.git");
    assert_error(
        &fixture.run_check(&wrong_origin, "candidate-source", None, None, None, &[]),
        "origin does not match its canonical repository URL",
    );

    let fixture = RegistryCheckV2Fixture::new();
    git(
        &fixture.candidate_source_root,
        &["switch", "--quiet", "-c", "wrong-source-branch"],
    );
    assert_error(
        &fixture.run_check(
            &fixture.base_registry(),
            "candidate-source",
            None,
            None,
            None,
            &[],
        ),
        "must be detached",
    );

    let fixture = RegistryCheckV2Fixture::new();
    let mut wrong_head = fixture.base_registry();
    wrong_head["worktrees"]["candidate-source"]["pinned_sha"] =
        json!("ffffffffffffffffffffffffffffffffffffffff");
    assert_error(
        &fixture.run_check(&wrong_head, "candidate-source", None, None, None, &[]),
        "HEAD does not match its pinned SHA",
    );

    let fixture = RegistryCheckV2Fixture::new();
    let wrong_common = fixture.base.control.root().join("wrong-common-dir");
    fs::create_dir_all(&wrong_common).expect("create wrong common-dir fixture");
    let mut wrong_common_registry = fixture.base_registry();
    for worktree_id in ["queue-coordinator", "candidate-source", "reference"] {
        wrong_common_registry["worktrees"][worktree_id]["common_dir"] =
            json!(path_text(&wrong_common));
    }
    assert_error(
        &fixture.run_check(
            &wrong_common_registry,
            "candidate-source",
            None,
            None,
            None,
            &[],
        ),
        "Git common-dir does not match the registry",
    );
}

#[test]
fn registry_check_v2_reports_dirty_source_roots_as_unqualified() {
    let fixture = RegistryCheckV2Fixture::new();
    fs::write(
        fixture.candidate_source_root.join("untracked-source.txt"),
        b"dirty source\n",
    )
    .expect("make candidate source dirty");
    let registry = fixture.base_registry();
    let source_output = fixture.run_check(&registry, "candidate-source", None, None, None, &[]);
    assert_success(&source_output);
    let source_report: Value =
        serde_json::from_slice(&source_output.stdout).expect("parse dirty source observation");
    assert_eq!(source_report["selected_worktree"]["identity_matches"], true);
    assert_eq!(source_report["selected_worktree"]["clean"], false);
    assert_eq!(
        source_report["selected_worktree"]["identity_qualified"],
        false
    );

    fs::remove_file(fixture.candidate_source_root.join("untracked-source.txt"))
        .expect("restore candidate source fixture");
    fs::write(
        fixture.base.reference_root.join("untracked-reference.txt"),
        b"dirty reference\n",
    )
    .expect("make reference worktree dirty");
    let reference_output = fixture.run_check(&registry, "reference", None, None, None, &[]);
    assert_success(&reference_output);
    let reference_report: Value = serde_json::from_slice(&reference_output.stdout)
        .expect("parse dirty reference observation");
    assert_eq!(
        reference_report["selected_worktree"]["identity_matches"],
        true
    );
    assert_eq!(reference_report["selected_worktree"]["clean"], false);
    assert_eq!(
        reference_report["selected_worktree"]["identity_qualified"],
        false
    );
}

#[cfg(unix)]
#[test]
fn registry_check_v2_rejects_symlinked_overlapping_and_registry_inside_roots() {
    use std::os::unix::fs::symlink;

    let fixture = RegistryCheckV2Fixture::new();
    let alias = fixture.base.control.root().join("candidate-source-alias");
    symlink(&fixture.candidate_source_root, &alias).expect("create candidate-source symlink");
    let mut symlinked = fixture.base_registry();
    symlinked["worktrees"]["candidate-source"]["root"] = json!(path_text(&alias));
    assert_error(
        &fixture.run_check(&symlinked, "candidate-source", None, None, None, &[]),
        "registered worktree candidate-source root/common-dir contains a symlink",
    );

    let fixture = RegistryCheckV2Fixture::new();
    let mut overlapping = fixture.base_registry();
    overlapping["worktrees"]["candidate-source"]["root"] = json!(path_text(
        &fixture.base.candidate_root().join("nested-source")
    ));
    assert_error(
        &fixture.run_check(&overlapping, "candidate-source", None, None, None, &[]),
        "registered worktree roots must be disjoint",
    );

    let fixture = RegistryCheckV2Fixture::new();
    let inside_root = fixture.candidate_source_root.join("registry-inside.json");
    assert_error(
        &fixture.run_check(
            &fixture.base_registry(),
            "candidate-source",
            Some(&inside_root),
            None,
            None,
            &[],
        ),
        "root registry file must be outside every registered worktree root",
    );
}

#[test]
fn registry_check_v2_keeps_static_shape_validation_distinct_from_live_observation() {
    let fixture = RegistryCheckV2Fixture::new();
    let registry = fixture.base_registry();
    let shape_output = fixture.run_validate(&registry);
    assert_success(&shape_output);
    let shape_report: Value =
        serde_json::from_slice(&shape_output.stdout).expect("parse static registry shape report");
    assert_eq!(shape_report["live_roots_qualified"], false);

    let live_output = fixture.run_check(&registry, "reference", None, None, None, &[]);
    assert_success(&live_output);
    let live_report: Value =
        serde_json::from_slice(&live_output.stdout).expect("parse selected-role observation");
    assert_eq!(live_report["selected_worktree"]["worktree_id"], "reference");
    assert_eq!(live_report["limits"]["unselected_roles_qualified"], false);
    assert!(live_report.get("worktrees").is_none());
    assert!(live_report.get("live_roots_qualified").is_none());
}

#[test]
fn registry_check_v2_reports_optional_roles_unprobed_and_rejects_them() {
    let fixture = RegistryCheckV2Fixture::new();
    let mut registry = fixture.base_registry();
    fixture.add_optional_velnor_pair(&mut registry);
    let shape_output = fixture.run_validate(&registry);
    assert_success(&shape_output);
    let shape_report: Value =
        serde_json::from_slice(&shape_output.stdout).expect("parse optional-role shape report");
    assert_eq!(shape_report["live_roots_qualified"], false);

    let live_output = fixture.run_check(&registry, "candidate-source", None, None, None, &[]);
    assert_success(&live_output);
    let live_report: Value =
        serde_json::from_slice(&live_output.stdout).expect("parse optional-role observation");
    assert_eq!(
        live_report["optional_roles"]["velnor-source-pin"],
        "not_probed"
    );
    assert_eq!(
        live_report["optional_roles"]["velnor-scratch"],
        "not_probed"
    );
    assert_eq!(live_report["limits"]["unselected_roles_qualified"], false);

    assert_error(
        &fixture.run_check(&registry, "velnor-source-pin", None, None, None, &[]),
        "supports only queue-coordinator, candidate-source, and reference",
    );
    assert_error(
        &fixture.run_check(&registry, "velnor-scratch", None, None, None, &[]),
        "supports only queue-coordinator, candidate-source, and reference",
    );
    assert!(
        !fixture
            .base
            .control
            .root()
            .join("not-created/velnor")
            .exists(),
        "unselected optional roots must not be created or probed"
    );
}

#[test]
fn registry_check_v2_requires_the_registered_coordinator_root() {
    let fixture = RegistryCheckV2Fixture::new();
    let registry = fixture.base_registry();
    assert_error(
        &fixture.run_check(
            &registry,
            "candidate-source",
            None,
            None,
            Some(&fixture.candidate_source_root),
            &[],
        ),
        "queue-coordinator root does not exactly match --root",
    );
}

#[test]
fn registry_check_v2_rejects_a_reference_pin_outside_tracking_ancestry() {
    let fixture = RegistryCheckV2Fixture::new();
    let unrelated = git_output(
        fixture.base.candidate_root(),
        &["rev-parse", "--verify", "HEAD"],
    );
    git(
        fixture.base.candidate_root(),
        &[
            "update-ref",
            "refs/remotes/origin/visual-baseline",
            unrelated.as_str(),
        ],
    );
    assert_error(
        &fixture.run_check(&fixture.base_registry(), "reference", None, None, None, &[]),
        "is not reachable from refs/remotes/origin/visual-baseline",
    );
}
