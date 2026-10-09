//! Black-box contract tests for the pinned subject-build launcher.
//!
//! The operational success case uses synthetic Cargo metadata/build messages,
//! but resolves and archives the exact current branch tips from a synthetic
//! caller-pinned facts record, plus the immutable oracle tag. The fake Cargo
//! fixture never compiles or launches a product executable.

#[cfg(unix)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::env;
use std::fs;
#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(unix)]
use std::process::Output;
use std::sync::{Mutex, MutexGuard};
#[cfg(unix)]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use termrock_visibility_tests::run_cli_with_timeout;
use termrock_visibility_tests::{CliOutput, TempRepo, Tool, install_fixture_aliases, run_cli};

const ORACLE_COMMIT_SHA: &str = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b";
const REFERENCE_TAG_OBJECT: &str = "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5";
const REFERENCE_BRANCH_REF: &str = "refs/remotes/origin/visual-baseline";
const CANDIDATE_BRANCH_REF: &str = "refs/remotes/origin/termrock-implementation";
const SUBJECT_SCHEMA_SHA256: &str =
    "e2d294145e6e2a9b5b682e0dfcb7793fe0e20e4cf5492b30d458d85ad8bde8f0";
const TREE_OID_FIX_SUBJECTS_SHA256: &str =
    "d990b77fbec0d01f242dbd191698939cbf5790d8dcd203337166c26688d85853";
const SUBJECTS_TOOL_OVERRIDE_ENV: &str = "TERMROCK_VIS06_SUBJECTS_PY";
const SUITE_SHA256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const GENERATION_SHA256: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const APPS: [&str; 4] = ["showcase", "jackin-preview", "holla", "tablepro"];
const FIXTURE_SETTINGS_FILE: &str = "termrock-visibility-fixture-settings.json";
const FIXTURE_SETTINGS_SCHEMA: &str = "termrock-visibility-fixture-settings/v1";
const FIXTURE_SETTING_KEYS: &[&str] = &[
    "VISIBILITY_FIXTURE_TRACE",
    "VISIBILITY_FIXTURE_NEXTTEST_LIST",
    "VISIBILITY_FIXTURE_NEXTTEST_LIST_EXIT",
    "VISIBILITY_FIXTURE_NEXTTEST_RUN",
    "VISIBILITY_FIXTURE_NEXTTEST_EXIT",
    "VISIBILITY_FIXTURE_NEXTTEST_SIGNAL",
    "VISIBILITY_FIXTURE_NEXTEST_VERSION",
    "VISIBILITY_FIXTURE_NEXTEST_VERSION_EXIT",
    "VISIBILITY_FIXTURE_CARGO_METADATA",
    "VISIBILITY_FIXTURE_CARGO_METADATA_RAW",
    "VISIBILITY_FIXTURE_CARGO_METADATA_EXIT",
    "VISIBILITY_FIXTURE_CARGO_METADATA_STDERR",
    "VISIBILITY_FIXTURE_CARGO_BUILD_EXIT",
    "VISIBILITY_FIXTURE_CARGO_BUILD_STDOUT",
    "VISIBILITY_FIXTURE_CARGO_BUILD_STDERR",
    "VISIBILITY_FIXTURE_ARTIFACT_MODE",
    "VISIBILITY_FIXTURE_CARGO_VERSION",
    "VISIBILITY_FIXTURE_CARGO_VERSION_EXIT",
    "VISIBILITY_FIXTURE_RUSTC_VERSION",
    "VISIBILITY_FIXTURE_RUSTC_VERSION_EXIT",
    "VISIBILITY_FIXTURE_RUSTC_VERBOSE",
    "VISIBILITY_FIXTURE_RUSTC_VERBOSE_EXIT",
    "VISIBILITY_FIXTURE_RUSTUP_HOME",
];
static SUBJECTS_FIXTURE_LOCK: Mutex<()> = Mutex::new(());

struct SubjectsFixture {
    repo: TempRepo,
    script: PathBuf,
    repository: PathBuf,
    fixture_path: String,
    metadata_path: PathBuf,
    trace_path: PathBuf,
    rustup_home: PathBuf,
    cargo_home: PathBuf,
    home: PathBuf,
    expected_root: PathBuf,
    reference_output_root: PathBuf,
    candidate_output_root: PathBuf,
    source_facts_path: PathBuf,
    source_facts_sha256: String,
    reference_sha: String,
    candidate_sha: String,
    source_trees: RefCell<Vec<PathBuf>>,
    #[cfg(unix)]
    rustup_adapter_sequence: Cell<u64>,
    #[cfg(unix)]
    last_rustup_adapter: RefCell<Option<RustupFixtureAdapter>>,
}

impl SubjectsFixture {
    fn new() -> Self {
        let repo = TempRepo::new().expect("create disposable subjects fixture");
        let copied_tool = repo
            .copy_tool_exact(Tool::Subjects)
            .expect("copy exact subjects CLI");
        let source_tool =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/visibility/subjects.py");
        if let Some(override_tool) = std::env::var_os(SUBJECTS_TOOL_OVERRIDE_ENV) {
            let override_tool = PathBuf::from(override_tool);
            assert_eq!(
                sha256_file(&override_tool),
                TREE_OID_FIX_SUBJECTS_SHA256,
                "the external tree OID validator must match its frozen source hash"
            );
            fs::copy(&override_tool, &copied_tool.path)
                .expect("copy the frozen external subjects CLI into the fixture");
            assert_eq!(
                sha256_file(&copied_tool.path),
                TREE_OID_FIX_SUBJECTS_SHA256,
                "the CLI subprocess must run the frozen external subjects source"
            );
        } else {
            assert_eq!(copied_tool.sha256, sha256_file(&source_tool));
        }
        let copied_schema = repo
            .copy_project_file_exact(Path::new(
                "crates/termrock-e2e/schemas/subject-manifest-v2.schema.json",
            ))
            .expect("copy exact frozen subject schema");
        assert_eq!(
            copied_schema.sha256, SUBJECT_SCHEMA_SHA256,
            "the shared subject schema must remain frozen"
        );

        let fixture_binary = Path::new(env!("CARGO_BIN_EXE_fixture"));
        let fixture_bin_dir = repo.root().join("fixture-bin");
        let aliases = install_fixture_aliases(
            fixture_binary,
            &fixture_bin_dir,
            &["rustup", "mise", "cargo", "rustc", "cargo-nextest"],
        )
        .expect("install fixture command aliases")
        .to_string_lossy()
        .into_owned();

        let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("resolve product repository from test package");
        let reference_sha = git_ref(&repository, REFERENCE_BRANCH_REF);
        let candidate_sha = git_ref(&repository, CANDIDATE_BRANCH_REF);
        let (source_facts_path, source_facts_sha256) =
            write_source_facts(&repo, "source-facts.json", &reference_sha, &candidate_sha);
        let metadata_path = repo
            .write_file(
                Path::new("fixtures/cargo-metadata.json"),
                &serde_json::to_vec(&cargo_metadata()).expect("serialize synthetic metadata"),
            )
            .expect("write synthetic Cargo metadata");
        let trace_path = repo.root().join("fixtures/trace.jsonl");
        let rustup_home = repo.root().join("fixture-rustup-home");
        let cargo_home = repo.root().join("fixture-cargo-home");
        let home = repo.root().join("fixture-home");
        fs::create_dir_all(&rustup_home).expect("create fixture rustup home");
        fs::create_dir_all(&cargo_home).expect("create empty fixture Cargo home");
        fs::create_dir_all(&home).expect("create fixture HOME");

        Self {
            expected_root: repo.root().join("expected-generation"),
            reference_output_root: repo.root().join("actual/reference"),
            candidate_output_root: repo.root().join("actual/candidate"),
            source_facts_path,
            source_facts_sha256,
            reference_sha,
            candidate_sha,
            source_trees: RefCell::new(Vec::new()),
            #[cfg(unix)]
            rustup_adapter_sequence: Cell::new(0),
            #[cfg(unix)]
            last_rustup_adapter: RefCell::new(None),
            repo,
            script: copied_tool.path,
            repository,
            fixture_path: aliases,
            metadata_path,
            trace_path,
            rustup_home,
            cargo_home,
            home,
        }
    }

    fn base_args(&self, run_id: &str) -> Vec<String> {
        vec![
            "--repository".to_owned(),
            self.repository.display().to_string(),
            "--candidate-commit".to_owned(),
            self.candidate_sha.clone(),
            "--reference-commit".to_owned(),
            self.reference_sha.clone(),
            "--source-facts-path".to_owned(),
            self.source_facts_path.display().to_string(),
            "--source-facts-sha256".to_owned(),
            self.source_facts_sha256.clone(),
            "--evidence-root".to_owned(),
            self.repo.root().join("evidence").display().to_string(),
            "--run-id".to_owned(),
            run_id.to_owned(),
            "--app".to_owned(),
            "tablepro".to_owned(),
            "--suite-revision".to_owned(),
            "synthetic-suite-revision".to_owned(),
            "--suite-sha256".to_owned(),
            SUITE_SHA256.to_owned(),
            "--expected-generation-id".to_owned(),
            "synthetic-generation".to_owned(),
            "--expected-generation-sha256".to_owned(),
            GENERATION_SHA256.to_owned(),
            "--expected-generation-root".to_owned(),
            self.expected_root.display().to_string(),
            "--target-triple".to_owned(),
            native_target().to_owned(),
            "--reference-output-root".to_owned(),
            self.reference_output_root.display().to_string(),
            "--candidate-output-root".to_owned(),
            self.candidate_output_root.display().to_string(),
        ]
    }

    fn tag_args(&self, run_id: &str, repository: &Path) -> Vec<String> {
        vec![
            "build-frozen-tag-holla".to_owned(),
            "--repository".to_owned(),
            repository.display().to_string(),
            "--evidence-root".to_owned(),
            self.repo.root().join("tag-evidence").display().to_string(),
            "--run-id".to_owned(),
            run_id.to_owned(),
            "--target-triple".to_owned(),
            native_target().to_owned(),
        ]
    }

    fn tag_run_root(&self, run_id: &str) -> PathBuf {
        self.repo.root().join("tag-evidence").join(run_id)
    }

    fn run(&self, args: &[String], overrides: &[(&str, String)]) -> CliOutput {
        let mut fixture_settings = BTreeMap::from([
            (
                "VISIBILITY_FIXTURE_RUSTUP_HOME".to_owned(),
                self.rustup_home.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_CARGO_METADATA".to_owned(),
                self.metadata_path.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_TRACE".to_owned(),
                self.trace_path.display().to_string(),
            ),
        ]);
        for (name, value) in overrides {
            if FIXTURE_SETTING_KEYS.contains(name) {
                fixture_settings.insert((*name).to_owned(), value.clone());
            } else {
                assert!(
                    matches!(*name, "GIT_NO_REPLACE_OBJECTS" | "GIT_NO_LAZY_FETCH"),
                    "test environment override `{name}` is outside the narrow allowlist"
                );
            }
        }
        let settings_bytes = serde_json::to_vec(&json!({
            "schema": FIXTURE_SETTINGS_SCHEMA,
            "settings": &fixture_settings,
        }))
        .expect("serialize fixed fixture settings envelope");
        let settings_relative_path = Path::new("fixture-bin").join(FIXTURE_SETTINGS_FILE);
        self.repo
            .write_file(&settings_relative_path, &settings_bytes)
            .expect("write fixed fixture settings file beside aliases");

        #[cfg(unix)]
        let rustup_adapter = {
            let rustup_home = fixture_settings
                .get("VISIBILITY_FIXTURE_RUSTUP_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| self.rustup_home.clone());
            install_rustup_fixture_test_adapter(self, &rustup_home, &fixture_settings)
        };

        if let Some(run_id) = arg_value(args, "--run-id")
            && is_safe_fixture_run_id(run_id)
        {
            let evidence_name = if args
                .first()
                .is_some_and(|arg| arg == "build-frozen-tag-holla")
            {
                "tag-evidence"
            } else {
                "evidence"
            };
            let source_tree = self
                .repo
                .root()
                .join(evidence_name)
                .join(run_id)
                .join("source");
            let mut registered = self.source_trees.borrow_mut();
            if !registered.contains(&source_tree) {
                registered.push(source_tree);
            }
        }

        #[cfg(unix)]
        let fixture_path = format!(
            "{}:{}",
            rustup_adapter
                .alias_path
                .parent()
                .expect("test adapter has a parent directory")
                .display(),
            self.fixture_path
        );
        #[cfg(not(unix))]
        let fixture_path = self.fixture_path.clone();

        let mut environment = vec![
            ("PATH".to_owned(), fixture_path),
            ("HOME".to_owned(), self.home.display().to_string()),
            (
                "CARGO_HOME".to_owned(),
                self.cargo_home.display().to_string(),
            ),
            (
                "RUSTUP_HOME".to_owned(),
                self.rustup_home.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_RUSTUP_HOME".to_owned(),
                self.rustup_home.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_CARGO_METADATA".to_owned(),
                self.metadata_path.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_TRACE".to_owned(),
                self.trace_path.display().to_string(),
            ),
        ];
        environment.extend(
            overrides
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone())),
        );
        let borrowed: Vec<(&str, &str)> = environment
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let output = run_cli(&self.script, args, self.repo.root(), &borrowed, None)
            .expect("launch subjects CLI");
        #[cfg(unix)]
        self.last_rustup_adapter.replace(Some(rustup_adapter));
        output
    }

    fn cleanup_sources_for_run(&self, run_id: &str) {
        if !is_safe_fixture_run_id(run_id) {
            return;
        }
        let source_tree = self
            .repo
            .root()
            .join("evidence")
            .join(run_id)
            .join("source");
        remove_owned_readonly_tree(self.repo.root(), &source_tree)
            .unwrap_or_else(|error| panic!("clean fixture source tree: {error}"));
        match fs::symlink_metadata(&source_tree) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => panic!("owned fixture source tree remains after cleanup"),
            Err(error) => panic!("inspect owned fixture source tree after cleanup: {error}"),
        }
    }

    fn cleanup_tag_source_for_run(&self, run_id: &str) {
        let source_tree = self.tag_run_root(run_id).join("source");
        remove_owned_readonly_tree(self.repo.root(), &source_tree)
            .unwrap_or_else(|error| panic!("clean tag fixture source tree: {error}"));
        match fs::symlink_metadata(&source_tree) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => panic!("owned tag fixture source tree remains after cleanup"),
            Err(error) => panic!("inspect tag source tree after cleanup: {error}"),
        }
    }

    fn manifest_path(&self, run_id: &str) -> PathBuf {
        self.repo
            .root()
            .join("evidence")
            .join(run_id)
            .join("subject-manifest.json")
    }

    fn run_record_path(&self, run_id: &str) -> PathBuf {
        self.repo
            .root()
            .join("evidence")
            .join(run_id)
            .join("run.json")
    }

    fn set_arg(args: &mut [String], name: &str, value: String) {
        let index = args
            .iter()
            .position(|arg| arg == name)
            .unwrap_or_else(|| panic!("missing argument {name}"));
        args[index + 1] = value;
    }

    fn remove_arg(args: &mut Vec<String>, name: &str) {
        let index = args
            .iter()
            .position(|arg| arg == name)
            .unwrap_or_else(|| panic!("missing argument {name}"));
        args.drain(index..=index + 1);
    }

    fn set_output_root(args: &mut [String], name: &str, path: &Path) {
        Self::set_arg(args, name, path.display().to_string());
    }

    fn set_source_facts(
        &self,
        args: &mut [String],
        name: &str,
        reference_sha: &str,
        candidate_sha: &str,
    ) {
        let path_name = format!("source-facts-{name}.json");
        let (path, sha256) =
            write_source_facts(&self.repo, &path_name, reference_sha, candidate_sha);
        Self::set_arg(args, "--source-facts-path", path.display().to_string());
        Self::set_arg(args, "--source-facts-sha256", sha256);
    }
}

impl Drop for SubjectsFixture {
    fn drop(&mut self) {
        for source_tree in self.source_trees.get_mut() {
            let _ = remove_owned_readonly_tree(self.repo.root(), source_tree);
        }
    }
}

#[cfg(unix)]
#[derive(Clone)]
struct RustupFixtureAdapter {
    alias_path: PathBuf,
    alias_sha256: String,
    interpreter_entry_path: PathBuf,
    interpreter_path: PathBuf,
    interpreter_sha256: String,
}

#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn shell_quote_path(path: &Path) -> String {
    shell_quote(
        path.to_str()
            .expect("fixture adapter paths must be valid UTF-8"),
    )
}

#[cfg(unix)]
fn install_rustup_fixture_test_adapter(
    fixture: &SubjectsFixture,
    rustup_home: &Path,
    fixture_settings: &BTreeMap<String, String>,
) -> RustupFixtureAdapter {
    assert_eq!(
        fixture_settings
            .get("VISIBILITY_FIXTURE_RUSTUP_HOME")
            .map(String::as_str),
        rustup_home.to_str(),
        "the adapter Rustup home must match the fixed fixture settings"
    );
    let fixture_alias_path = fixture.repo.root().join("fixture-bin/rustup");
    let alias_metadata = fs::symlink_metadata(&fixture_alias_path)
        .expect("inspect the original disposable rustup fixture alias");
    assert!(
        alias_metadata.file_type().is_symlink(),
        "the adapter must leave the fixture's original rustup symlink intact"
    );
    let fixture_binary = Path::new(env!("CARGO_BIN_EXE_fixture"))
        .canonicalize()
        .expect("resolve the compiled synthetic fixture binary");
    assert_eq!(
        fixture_alias_path
            .canonicalize()
            .expect("resolve the original rustup fixture alias"),
        fixture_binary,
        "the original rustup alias must target the compiled fixture"
    );

    let sequence = fixture.rustup_adapter_sequence.get();
    fixture
        .rustup_adapter_sequence
        .set(sequence.checked_add(1).expect("adapter sequence overflow"));
    let adapter_dir = fixture
        .repo
        .root()
        .join("fixture-rustup-adapters")
        .join(format!("run-{sequence:04}"));
    fs::create_dir_all(&adapter_dir).expect("create isolated rustup adapter directory");
    for tool in ["cargo", "rustc"] {
        let tool_alias = adapter_dir.join(tool);
        assert!(
            fs::symlink_metadata(&tool_alias).is_err(),
            "a new adapter must not replace an existing {tool} alias"
        );
        symlink(&fixture_binary, &tool_alias)
            .unwrap_or_else(|error| panic!("install synthetic {tool} alias: {error}"));
        assert_eq!(
            tool_alias
                .canonicalize()
                .unwrap_or_else(|error| panic!("resolve synthetic {tool} alias: {error}")),
            fixture_binary,
            "the synthetic {tool} path must resolve to the pinned fixture binary"
        );
        let tool_metadata = fs::metadata(&tool_alias)
            .unwrap_or_else(|error| panic!("inspect synthetic {tool} alias: {error}"));
        assert!(tool_metadata.is_file());
        assert_ne!(
            tool_metadata.permissions().mode() & 0o111,
            0,
            "the synthetic {tool} alias must resolve to an executable"
        );
    }
    let alias_path = adapter_dir.join("rustup");
    assert!(
        fs::symlink_metadata(&alias_path).is_err(),
        "a new test invocation must have a fresh rustup adapter path"
    );

    // Bash's exec -a keeps argv[0] equal to the rustup alias while this
    // test-only adapter supplies synthetic fixture settings to the child.
    // The production CLI still receives and applies its unchanged clean-env
    // policy. This adapter is verified on the pinned macOS runner; other Unix
    // targets need their own native portability check.
    let interpreter_entry_path = PathBuf::from("/bin/bash");
    let interpreter_path = interpreter_entry_path
        .canonicalize()
        .expect("resolve the Bash interpreter used by the fixture adapter");
    let interpreter_metadata = fs::metadata(&interpreter_path)
        .expect("inspect the Bash interpreter used by the fixture adapter");
    assert!(interpreter_metadata.is_file());
    assert_ne!(
        interpreter_metadata.permissions().mode() & 0o111,
        0,
        "the fixture adapter's Bash interpreter must be executable"
    );
    let interpreter_sha256 = sha256_file(&interpreter_path);

    let mut script = format!(
        "#!{}\n\
         # test-only rustup adapter\n\
         # interpreter_entry={}\n\
         # interpreter_resolved={}\n\
         # interpreter_sha256={interpreter_sha256}\n",
        interpreter_entry_path.display(),
        interpreter_entry_path.display(),
        interpreter_path.display(),
    );
    for (name, value) in fixture_settings {
        assert!(
            FIXTURE_SETTING_KEYS.contains(&name.as_str()),
            "the rustup adapter may export only named synthetic fixture settings"
        );
        script.push_str("export ");
        script.push_str(name);
        script.push('=');
        script.push_str(&shell_quote(value));
        script.push('\n');
    }
    let requested_build_exit = fixture_settings
        .get("VISIBILITY_FIXTURE_CARGO_BUILD_EXIT")
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|exit| *exit != 0);
    let has_build_stdout = fixture_settings
        .get("VISIBILITY_FIXTURE_CARGO_BUILD_STDOUT")
        .is_some_and(|path| !path.is_empty());
    if let Some(exit) = requested_build_exit.filter(|_| has_build_stdout) {
        // The committed fixture returns before emitting configured stdout for
        // a failed build. For this synthetic protocol case only, let it emit
        // the requested streams on the success branch, then return the
        // requested failure status from the adapter so the production CLI
        // observes the complete command result.
        script.push_str(&format!(
            "if [ \"$#\" -ge 4 ] && [ \"$1\" = run ] && [ \"$2\" = 1.98.1 ] && [ \"$3\" = cargo ] && [ \"$4\" = build ]; then\n\\
             VISIBILITY_FIXTURE_CARGO_BUILD_EXIT=0 /bin/bash -c 'exec -a \"$0\" \"$1\" \"${{@:2}}\"' \"$0\" {} \"$@\"\n\\
             fixture_status=$?\n\\
             if [ \"$fixture_status\" -ne 0 ]; then exit \"$fixture_status\"; fi\n\\
             exit {exit}\n\\
         fi\n",
            shell_quote_path(&fixture_binary),
        ));
    }
    script.push_str(&format!(
        "exec -a \"$0\" {} \"$@\"\n",
        shell_quote_path(&fixture_binary),
    ));
    fs::write(&alias_path, script.as_bytes()).expect("write the disposable rustup adapter");
    let mut adapter_permissions = fs::metadata(&alias_path)
        .expect("inspect the disposable rustup adapter")
        .permissions();
    adapter_permissions.set_mode(0o755);
    fs::set_permissions(&alias_path, adapter_permissions)
        .expect("make the disposable rustup adapter executable");
    assert!(
        !fs::symlink_metadata(&alias_path)
            .expect("reinspect the disposable rustup adapter")
            .file_type()
            .is_symlink(),
        "the rustup adapter must be a regular file, not a redirected alias"
    );

    RustupFixtureAdapter {
        alias_sha256: sha256_file(&alias_path),
        alias_path,
        interpreter_entry_path,
        interpreter_path,
        interpreter_sha256,
    }
}

fn arg_value<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == option)
        .map(|pair| pair[1].as_str())
}

fn git_ref(repository: &Path, reference: &str) -> String {
    let output = Command::new("git")
        .current_dir(repository)
        .args(["--no-replace-objects", "rev-parse", "--verify", reference])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("resolve exact fetched branch ref for synthetic source facts");
    assert!(
        output.status.success(),
        "git rev-parse {reference} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git branch oid is UTF-8")
        .trim()
        .to_owned()
}

fn write_source_facts(
    repo: &TempRepo,
    filename: &str,
    reference_sha: &str,
    candidate_sha: &str,
) -> (PathBuf, String) {
    let payload = json!({
        "schema_version": 2,
        "repository": "tailrocks/terminal-components-claude",
        "observed_at": "synthetic-test-input",
        "reference": {
            "branch": "visual-baseline",
            "head_sha": "0000000000000000000000000000000000000000"
        },
        "candidate": {
            "branch": "termrock-implementation",
            "head_sha": "1111111111111111111111111111111111111111"
        },
        "latest_source_observation": {
            "observed_at": "synthetic-test-input",
            "method": "synthetic Rust black-box fixture using real fetched Git refs",
            "reference_remote": {
                "branch": "visual-baseline",
                "head_sha": reference_sha,
                "updated_at": "synthetic-test-input"
            },
            "candidate_remote": {
                "branch": "termrock-implementation",
                "head_sha": candidate_sha,
                "updated_at": "synthetic-test-input"
            }
        }
    });
    let bytes = serde_json::to_vec(&payload).expect("serialize synthetic source facts");
    let relative = Path::new("fixtures").join(filename);
    let path = repo
        .write_file(&relative, &bytes)
        .expect("write synthetic source facts fixture");
    let sha256 = sha256_bytes(&bytes);
    (path, sha256)
}

fn is_safe_fixture_run_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn serialize_subjects_fixture() -> MutexGuard<'static, ()> {
    SUBJECTS_FIXTURE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(unix)]
fn remove_owned_readonly_tree(repository_root: &Path, tree: &Path) -> std::io::Result<()> {
    let relative = match tree.strip_prefix(repository_root) {
        Ok(relative) => relative,
        Err(_) => return Ok(()),
    };
    let mut current = repository_root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Ok(());
        };
        current.push(name);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Ok(());
        }
    }

    fn restore_directory_permissions(directory: &Path) -> std::io::Result<()> {
        let metadata = fs::symlink_metadata(directory)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Ok(());
        }
        let mode = metadata.permissions().mode();
        if mode & 0o700 != 0o700 {
            fs::set_permissions(directory, fs::Permissions::from_mode(mode | 0o700))?;
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let child = entry.path();
            let child_metadata = fs::symlink_metadata(&child)?;
            if child_metadata.file_type().is_dir() {
                restore_directory_permissions(&child)?;
            }
        }
        Ok(())
    }

    restore_directory_permissions(tree)?;
    match fs::symlink_metadata(tree) {
        Ok(metadata) if !metadata.file_type().is_symlink() && metadata.file_type().is_dir() => {
            fs::remove_dir_all(tree)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(not(unix))]
fn remove_owned_readonly_tree(_repository_root: &Path, _tree: &Path) -> std::io::Result<()> {
    // The shared CLI runner is Unix-only and returns before any snapshot can
    // be materialized on other hosts.
    Ok(())
}

fn cargo_metadata() -> Value {
    let packages: Vec<Value> = APPS
        .iter()
        .map(|app| {
            json!({
                "id": format!("path+file://__WORKSPACE_ROOT__#{app}@0.1.0"),
                "name": app,
                "version": "0.1.0",
                "manifest_path": "__MANIFEST_PATH__",
                "targets": [{
                    "name": app,
                    "kind": ["bin"],
                    "src_path": "__MANIFEST_PATH__"
                }]
            })
        })
        .collect();
    json!({
        "packages": packages,
        "workspace_members": [],
        "workspace_default_members": [],
        "resolve": null,
        "target_directory": "__WORKSPACE_ROOT__/target",
        "build_directory": "__WORKSPACE_ROOT__/target",
        "version": 1,
        "workspace_root": "__WORKSPACE_ROOT__",
        "metadata": null
    })
}

fn native_target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        other => panic!("unsupported test host {other:?}"),
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_file(path: &Path) -> String {
    sha256_bytes(&fs::read(path).expect("read file for SHA-256"))
}

#[cfg(unix)]
fn git_object_store_digest(root: &Path) -> String {
    fn collect(root: &Path, directory: &Path, rows: &mut Vec<String>) {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .expect("read disposable Git object-store directory")
            .map(|entry| entry.expect("read Git object-store entry"))
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("object-store entry is under its root")
                .to_string_lossy()
                .into_owned();
            let metadata = fs::symlink_metadata(&path).expect("inspect Git object-store entry");
            let mode = metadata.permissions().mode() & 0o7777;
            let file_type = metadata.file_type();
            if file_type.is_dir() {
                rows.push(format!("{relative}\0directory\0{mode:o}\0\n"));
                collect(root, &path, rows);
            } else if file_type.is_file() {
                rows.push(format!(
                    "{relative}\0file\0{mode:o}\0{}\n",
                    sha256_file(&path)
                ));
            } else if file_type.is_symlink() {
                let target = fs::read_link(&path).expect("read object-store symlink");
                rows.push(format!(
                    "{relative}\0symlink\0{mode:o}\0{}\n",
                    target.to_string_lossy()
                ));
            } else {
                panic!("unsupported node in disposable Git object store: {relative}");
            }
        }
    }

    let mut rows = Vec::new();
    collect(root, root, &mut rows);
    rows.sort();
    sha256_bytes(rows.concat().as_bytes())
}

fn object_keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("JSON object")
        .keys()
        .cloned()
        .collect()
}

fn path_blob_map_sha256(mut paths: Vec<(Vec<u8>, Vec<u8>)>) -> String {
    paths.sort_by(|left, right| left.0.cmp(&right.0));
    let mut digest = Sha256::new();
    for (path, object_id) in paths {
        digest.update(path);
        digest.update([0]);
        digest.update(object_id);
        digest.update([b'\n']);
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn split_tree_blob_maps(tree_listing: &[u8]) -> (Vec<(Vec<u8>, Vec<u8>)>, Vec<(Vec<u8>, Vec<u8>)>) {
    let mut included = Vec::new();
    let mut excluded = Vec::new();
    for record in tree_listing
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let separator = record
            .iter()
            .position(|byte| *byte == b'\t')
            .expect("Git tree record has a tab separator");
        let metadata = &record[..separator];
        let path = &record[separator + 1..];
        let metadata = std::str::from_utf8(metadata).expect("Git tree metadata is ASCII");
        let fields: Vec<_> = metadata.split_ascii_whitespace().collect();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[1], "blob");
        let row = (path.to_vec(), fields[2].as_bytes().to_vec());
        if path.starts_with(b"snapshots/") || path.starts_with(b"baselines/") {
            excluded.push(row);
        } else {
            included.push(row);
        }
    }
    (included, excluded)
}

fn output_text(output: &CliOutput) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn error_text(output: &CliOutput) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_failed(output: &CliOutput, expected_message: &str) {
    assert!(output.signal.is_none(), "CLI was terminated by a signal");
    assert_ne!(output.exit_code, Some(0), "CLI unexpectedly succeeded");
    let combined = format!("{}\n{}", error_text(output), output_text(output));
    assert!(
        combined.contains(expected_message),
        "expected failure containing `{expected_message}`, got:\n{combined}"
    );
}

fn assert_no_archive_metadata_or_build(fixture: &SubjectsFixture, run_id: &str) {
    let run_root = fixture.repo.root().join("evidence").join(run_id);
    assert!(
        !run_root.join("snapshot-logs").exists(),
        "rejected pins must fail before archive operations"
    );
    assert!(
        !run_root.join("metadata").exists(),
        "rejected pins must fail before Cargo metadata"
    );
    assert!(
        !run_root.join("build").exists(),
        "rejected pins must fail before Cargo build"
    );
}

#[cfg(unix)]
fn git_output(cwd: &Path, args: &[&str], disable_replacements: bool) -> Output {
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    if disable_replacements {
        command.env("GIT_NO_REPLACE_OBJECTS", "1");
    } else {
        command.env_remove("GIT_NO_REPLACE_OBJECTS");
    }
    command.output().expect("run real Git command")
}

#[cfg(unix)]
fn git_success(cwd: &Path, args: &[&str]) -> String {
    let output = git_output(cwd, args, true);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git emitted UTF-8 output")
        .trim()
        .to_owned()
}

#[cfg(unix)]
fn create_annotated_visual_tag(cwd: &Path, commit: &str) {
    git_success(cwd, &["update-ref", "-d", "refs/tags/visual-baseline"]);
    let output = Command::new("git")
        .current_dir(cwd)
        .args([
            "-c",
            "user.name=Visibility Test",
            "-c",
            "user.email=visibility-test@example.invalid",
            "tag",
            "-a",
            "visual-baseline",
            commit,
            "-m",
            "alternate visual-baseline test tag",
        ])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("create disposable annotated visual-baseline tag");
    assert!(
        output.status.success(),
        "git tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
fn assert_tag_preflight_blocked(
    fixture: &SubjectsFixture,
    run_id: &str,
    repository: &Path,
    expected_message: &str,
) {
    let args = fixture.tag_args(run_id, repository);
    let output = fixture.run(&args, &[]);
    assert_failed(&output, expected_message);
    let run_root = fixture.tag_run_root(run_id);
    let run_record: Value = serde_json::from_slice(
        &fs::read(run_root.join("run.json")).expect("read tag preflight run record"),
    )
    .expect("parse tag preflight run record");
    assert_eq!(run_record["build_result"], "BLOCKED");
    assert_eq!(run_record["build_attempted"], false);
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert_eq!(run_record["capture_status"], "NOT_RUN");
    assert_eq!(run_record["admission_status"], "NOT_RUN");
    assert!(!run_root.join("tag-builder-receipt.json").exists());
    assert!(!run_root.join("toolchain").exists());
    assert!(!run_root.join("source").exists());
    assert!(!run_root.join("snapshot-logs").exists());
    assert!(!run_root.join("metadata").exists());
    assert!(!run_root.join("build").exists());
    assert!(!run_root.join("subject-manifest.json").exists());
    assert!(
        !fixture.trace_path.exists(),
        "tag identity rejection must precede fake toolchain/Cargo invocations"
    );
}

#[cfg(unix)]
fn git_commit_parent(cwd: &Path, commit: &str) -> String {
    let revision = format!("{commit}^");
    git_success(cwd, &["rev-parse", "--verify", revision.as_str()])
}

#[cfg(unix)]
fn adversarial_candidate(
    fixture: &SubjectsFixture,
    repository: &Path,
    case_name: &str,
    entry_name: &str,
    symlink_target: Option<&str>,
) -> String {
    let init_path = repository.display().to_string();
    git_success(
        fixture.repo.root(),
        &["init", "--quiet", init_path.as_str()],
    );
    assert!(repository.join(".git").is_dir());

    let object_dir = git_success(
        &fixture.repository,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "objects",
        ],
    );
    let alternates = repository.join(".git/objects/info/alternates");
    fs::create_dir_all(alternates.parent().expect("alternates parent"))
        .expect("create Git alternates directory");
    fs::write(&alternates, format!("{object_dir}\n")).expect("write actual Git object alternate");
    git_success(
        repository,
        &[
            "update-ref",
            "refs/tags/visual-baseline",
            REFERENCE_TAG_OBJECT,
        ],
    );
    git_success(
        repository,
        &[
            "update-ref",
            REFERENCE_BRANCH_REF,
            fixture.reference_sha.as_str(),
        ],
    );
    git_success(
        repository,
        &[
            "update-ref",
            CANDIDATE_BRANCH_REF,
            fixture.candidate_sha.as_str(),
        ],
    );

    let entry_path = repository.join(entry_name);
    if let Some(target) = symlink_target {
        std::os::unix::fs::symlink(target, &entry_path).expect("create adversarial symlink");
    } else {
        fs::write(&entry_path, format!("unsafe {case_name}\n"))
            .expect("create adversarial tracked path");
    }
    git_success(repository, &["add", "--all"]);
    let tree = git_success(repository, &["write-tree"]);
    let mut commit = Command::new("git");
    let commit_message =
        format!("{case_name}\n\nSigned-off-by: Alexey Zhokhov <alexey@zhokhov.com>");
    let output = commit
        .current_dir(repository)
        .args(["commit-tree", tree.as_str(), "-p"])
        .arg(fixture.candidate_sha.as_str())
        .arg("-m")
        .arg(commit_message)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Visibility Test")
        .env("GIT_AUTHOR_EMAIL", "visibility-test@example.invalid")
        .env("GIT_COMMITTER_NAME", "Visibility Test")
        .env("GIT_COMMITTER_EMAIL", "visibility-test@example.invalid")
        .output()
        .expect("create adversarial commit from actual Git objects");
    assert!(
        output.status.success(),
        "git commit-tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let commit = String::from_utf8(output.stdout)
        .expect("Git commit id is UTF-8")
        .trim()
        .to_owned();
    git_success(
        repository,
        &["update-ref", CANDIDATE_BRANCH_REF, commit.as_str()],
    );
    git_success(
        repository,
        &["replace", commit.as_str(), fixture.candidate_sha.as_str()],
    );

    let unreplaced = git_output(repository, &["cat-file", "commit", commit.as_str()], true);
    let replaced = git_output(repository, &["cat-file", "commit", commit.as_str()], false);
    assert!(unreplaced.status.success() && replaced.status.success());
    assert_ne!(
        unreplaced.stdout, replaced.stdout,
        "the disposable repository must exercise a real refs/replace mapping"
    );
    commit
}

#[test]
fn rejects_a_nonimmutable_tag_before_toolchain_or_cargo_work() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-wrong-oracle-tag");
    let _candidate = adversarial_candidate(
        &fixture,
        &repository,
        "wrong-oracle-tag",
        "tag-probe.txt",
        None,
    );
    create_annotated_visual_tag(&repository, ORACLE_COMMIT_SHA);

    let run_id = "tag-negative-wrong-object";
    let args = fixture.tag_args(run_id, &repository);
    let output = fixture.run(&args, &[]);
    assert_failed(&output, "immutable visual-baseline tag object changed");
    let run_root = fixture.tag_run_root(run_id);
    let run_record: Value = serde_json::from_slice(
        &fs::read(run_root.join("run.json")).expect("read tag preflight run record"),
    )
    .expect("parse tag preflight run record");
    assert_eq!(run_record["build_result"], "BLOCKED");
    assert_eq!(run_record["build_attempted"], false);
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert_eq!(run_record["capture_status"], "NOT_RUN");
    assert_eq!(run_record["admission_status"], "NOT_RUN");
    assert!(!run_root.join("tag-builder-receipt.json").exists());
    assert!(!run_root.join("toolchain").exists());
    assert!(!run_root.join("metadata").exists());
    assert!(!run_root.join("build").exists());
    assert!(!run_root.join("subject-manifest.json").exists());
    let tag_argv: Vec<String> = serde_json::from_slice(
        &fs::read(run_root.join("git/oracle-tag/tag-object.argv.json"))
            .expect("read tag resolution argv"),
    )
    .expect("parse tag resolution argv");
    assert_eq!(tag_argv[1], "--no-replace-objects");
}

#[test]
fn rejects_a_missing_immutable_tag_before_toolchain_or_cargo_work() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-missing-oracle-tag");
    let _candidate = adversarial_candidate(
        &fixture,
        &repository,
        "missing-oracle-tag",
        "tag-probe.txt",
        None,
    );
    git_success(
        &repository,
        &["update-ref", "-d", "refs/tags/visual-baseline"],
    );

    assert_tag_preflight_blocked(
        &fixture,
        "tag-negative-missing",
        &repository,
        "cannot resolve immutable visual-baseline tag",
    );
}

#[test]
fn rejects_a_lightweight_tag_before_toolchain_or_cargo_work() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-lightweight-oracle-tag");
    let candidate = adversarial_candidate(
        &fixture,
        &repository,
        "lightweight-oracle-tag",
        "tag-probe.txt",
        None,
    );
    git_success(
        &repository,
        &[
            "update-ref",
            "refs/tags/visual-baseline",
            candidate.as_str(),
        ],
    );

    assert_tag_preflight_blocked(
        &fixture,
        "tag-negative-lightweight",
        &repository,
        "visual-baseline must remain an annotated tag object",
    );
}

#[test]
fn rejects_an_annotated_tag_with_the_wrong_peeled_commit_before_tools() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-wrong-peeled-oracle-tag");
    let candidate = adversarial_candidate(
        &fixture,
        &repository,
        "wrong-peeled-oracle-tag",
        "tag-probe.txt",
        None,
    );
    create_annotated_visual_tag(&repository, &candidate);

    assert_tag_preflight_blocked(
        &fixture,
        "tag-negative-wrong-peel",
        &repository,
        "immutable visual-baseline oracle commit changed",
    );
}

#[test]
fn frozen_tag_build_does_not_require_branch_refs_or_emit_a_pair_manifest() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-tag-only");
    let _candidate = adversarial_candidate(
        &fixture,
        &repository,
        "tag-only-no-branches",
        "tag-probe.txt",
        None,
    );
    git_success(
        &repository,
        &["replace", ORACLE_COMMIT_SHA, fixture.candidate_sha.as_str()],
    );
    git_success(&repository, &["update-ref", "-d", REFERENCE_BRANCH_REF]);
    git_success(&repository, &["update-ref", "-d", CANDIDATE_BRANCH_REF]);

    let run_id = "tag-fail-real-branchless";
    let args = fixture.tag_args(run_id, &repository);
    let build_stdout_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/tag-build-stdout.txt"),
            b"synthetic build stdout\n",
        )
        .expect("write fake Cargo stdout fixture");
    let build_stderr_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/tag-build-stderr.txt"),
            b"synthetic build stderr\n",
        )
        .expect("write fake Cargo stderr fixture");
    let output = fixture.run(
        &args,
        &[
            ("VISIBILITY_FIXTURE_CARGO_BUILD_EXIT", "23".to_owned()),
            (
                "VISIBILITY_FIXTURE_CARGO_BUILD_STDOUT",
                build_stdout_path.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_CARGO_BUILD_STDERR",
                build_stderr_path.display().to_string(),
            ),
            ("GIT_NO_REPLACE_OBJECTS", "0".to_owned()),
            ("GIT_NO_LAZY_FETCH", "0".to_owned()),
        ],
    );
    assert_eq!(output.exit_code, Some(3), "{}", error_text(&output));
    assert!(output.signal.is_none());
    let run_root = fixture.tag_run_root(run_id);
    let run_record: Value = serde_json::from_slice(
        &fs::read(run_root.join("run.json")).expect("read tag build run record"),
    )
    .expect("parse tag build run record");
    assert_eq!(run_record["build_result"], "FAIL");
    assert_eq!(run_record["build_attempted"], true);
    assert_eq!(run_record["capture_status"], "NOT_RUN");
    assert_eq!(run_record["admission_status"], "NOT_RUN");
    assert_eq!(
        run_record["oracle_lineage"]["tag_object"],
        REFERENCE_TAG_OBJECT
    );
    assert_eq!(
        run_record["oracle_lineage"]["tag_commit"],
        ORACLE_COMMIT_SHA
    );
    assert_eq!(
        run_record["source_snapshot"]["source_commit"],
        ORACLE_COMMIT_SHA
    );
    assert_eq!(
        run_record["git_lazy_fetch_policy"],
        "disabled-by-environment"
    );
    assert_eq!(
        run_record["git_object_replacement_policy"],
        "disabled-by-option-and-environment"
    );
    assert!(!run_root.join("tag-builder-receipt.json").exists());
    assert!(!run_root.join("subject-manifest.json").exists());
    let build_dir = run_root.join("build/oracle/holla");
    assert!(build_dir.join("build.argv.json").is_file());
    assert_eq!(
        fs::read(build_dir.join("build.stdout")).expect("preserve failed build stdout"),
        b"synthetic build stdout\n"
    );
    assert_eq!(
        fs::read(build_dir.join("build.stderr")).expect("preserve failed build stderr"),
        b"synthetic build stderr\n"
    );
    assert_eq!(
        fs::read(build_dir.join("build.exit")).expect("preserve failed build exit"),
        b"23\n"
    );
    assert_eq!(
        fs::read(build_dir.join("build.stdout")).expect("read recorded Cargo stdout"),
        b"synthetic build stdout\n"
    );
    assert_eq!(
        fs::read(build_dir.join("build.stderr")).expect("read recorded Cargo stderr"),
        b"synthetic build stderr\n"
    );
    assert_eq!(
        fs::read_to_string(build_dir.join("build.exit")).expect("read fake Cargo exit"),
        "23\n"
    );
    fixture.cleanup_tag_source_for_run(run_id);
}

#[cfg(unix)]
#[test]
fn validates_tag_bundle_source_closure_but_keeps_execution_unverified() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let repository = fixture.repo.root().join("git-tag-validator");
    let _candidate = adversarial_candidate(
        &fixture,
        &repository,
        "tag-validator",
        "tag-probe.txt",
        None,
    );
    let run_id = "tag-validator-positive";
    let rustup_home_alias = fixture.repo.root().join("rustup-home-alias");
    symlink(&fixture.rustup_home, &rustup_home_alias)
        .expect("create a path alias for the pinned Rustup home");
    let build_output = fixture.run(
        &fixture.tag_args(run_id, &repository),
        &[(
            "VISIBILITY_FIXTURE_RUSTUP_HOME",
            rustup_home_alias.display().to_string(),
        )],
    );
    let rustup_adapter = fixture
        .last_rustup_adapter
        .borrow()
        .as_ref()
        .expect("the run must retain its rustup adapter evidence")
        .clone();
    assert_eq!(
        build_output.exit_code,
        Some(0),
        "{}",
        error_text(&build_output)
    );
    assert!(build_output.signal.is_none());
    assert_eq!(
        rustup_adapter
            .interpreter_entry_path
            .canonicalize()
            .expect("resolve the Bash adapter entry after execution"),
        rustup_adapter.interpreter_path,
        "the adapter entry must keep resolving to the pinned Bash path"
    );
    assert_eq!(
        sha256_file(&rustup_adapter.interpreter_path),
        rustup_adapter.interpreter_sha256,
        "the pinned Bash interpreter bytes must remain stable during the fixture build"
    );
    assert_eq!(
        sha256_file(&rustup_adapter.alias_path),
        rustup_adapter.alias_sha256,
        "the disposable rustup adapter bytes must remain stable during the fixture build"
    );
    let run_root = fixture.tag_run_root(run_id);
    let receipt_path = run_root.join("tag-builder-receipt.json");
    let receipt_bytes = fs::read(&receipt_path).expect("read synthetic tag receipt");
    let receipt_sha = sha256_bytes(&receipt_bytes);
    let receipt: Value =
        serde_json::from_slice(&receipt_bytes).expect("parse synthetic tag receipt");
    let source_root = PathBuf::from(
        receipt["source_snapshot"]["materialized_root"]
            .as_str()
            .expect("tag materialized source path"),
    );
    let cargo_toml = source_root.join("Cargo.toml");
    let original_cargo_mode = fs::symlink_metadata(&cargo_toml)
        .expect("inspect read-only Cargo.toml")
        .permissions()
        .mode();
    let original_cargo_bytes = fs::read(&cargo_toml).expect("read pinned Cargo.toml bytes");
    let run_path = run_root.join("run.json");
    let original_run_bytes = fs::read(&run_path).expect("read original tag run record");

    let validate = |path: &Path, sha: &str| {
        let args = vec![
            "validate-frozen-tag-holla".to_owned(),
            "--repository".to_owned(),
            repository.display().to_string(),
            "--receipt".to_owned(),
            path.display().to_string(),
            "--receipt-sha256".to_owned(),
            sha.to_owned(),
        ];
        fixture.run(&args, &[])
    };

    let accepted = validate(&receipt_path, &receipt_sha);
    assert_eq!(accepted.exit_code, Some(0), "{}", error_text(&accepted));
    assert!(accepted.signal.is_none());
    let identity: Value =
        serde_json::from_slice(&accepted.stdout).expect("parse validated tag identity");
    assert_eq!(
        object_keys(&identity),
        BTreeSet::from(
            [
                "schema",
                "receipt",
                "run",
                "oracle_lineage",
                "source_snapshot",
                "metadata_target",
                "build",
                "executable",
                "builder_receipt_sha256",
                "build_environment",
                "execution_anchor_status",
                "qualification",
                "capture_status",
                "admission_status",
            ]
            .map(str::to_owned)
        )
    );
    assert_eq!(
        identity["schema"],
        "termrock-spec/visual-tag-holla-validated-identity-v1"
    );
    assert_eq!(identity["receipt"]["sha256"], receipt_sha);
    assert_eq!(
        identity["run"]["sha256"],
        sha256_bytes(&fs::read(run_root.join("run.json")).expect("read tag run record"))
    );
    assert_eq!(
        identity["source_snapshot"]["source_commit"],
        ORACLE_COMMIT_SHA
    );
    assert_eq!(
        identity["source_snapshot"]["tree_oid"],
        receipt["source_snapshot"]["tree_oid"]
    );
    let valid_tree_oid = receipt["source_snapshot"]["tree_oid"]
        .as_str()
        .expect("valid tag tree object ID");
    assert_eq!(valid_tree_oid.len(), 40);
    assert!(
        valid_tree_oid
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    let tree_revision = format!("{ORACLE_COMMIT_SHA}^{{tree}}");
    let actual_tree_oid = git_success(
        &repository,
        &["rev-parse", "--verify", tree_revision.as_str()],
    );
    assert_eq!(
        valid_tree_oid,
        actual_tree_oid.as_str(),
        "the accepted tree OID must be the pinned Git tree object"
    );
    assert_eq!(identity["execution_anchor_status"], "unverified");
    assert_eq!(identity["qualification"]["status"], "blocked");
    assert_eq!(identity["capture_status"], "NOT_RUN");
    assert_eq!(identity["admission_status"], "NOT_RUN");

    let run_record: Value =
        serde_json::from_slice(&original_run_bytes).expect("parse original tag run record");
    let expected_rustup_path = rustup_adapter.alias_path.to_string_lossy().into_owned();
    assert_eq!(
        run_record["rustup_path"].as_str(),
        Some(expected_rustup_path.as_str()),
        "the run record must identify the test-owned rustup adapter"
    );
    assert_eq!(
        run_record["rustup_sha256"].as_str(),
        Some(rustup_adapter.alias_sha256.as_str()),
        "the run record must bind the test-owned rustup adapter bytes"
    );
    let recorded_home_output = fs::read(run_root.join("toolchain/rustup-home.stdout"))
        .expect("read recorded rustup home output");
    let recorded_home_path = PathBuf::from(
        String::from_utf8(recorded_home_output.clone())
            .expect("rustup home output UTF-8")
            .trim(),
    );
    let canonical_rustup_home = PathBuf::from(
        run_record["rustup_home"]
            .as_str()
            .expect("recorded canonical rustup home"),
    );
    assert_ne!(
        recorded_home_path, canonical_rustup_home,
        "the positive fixture must record a noncanonical Rustup home alias"
    );
    assert_eq!(
        recorded_home_path
            .canonicalize()
            .expect("resolve recorded Rustup home alias"),
        canonical_rustup_home,
        "the recorded Rustup home alias must resolve to its pinned canonical path"
    );
    for binary_name in ["cargo", "rustc"] {
        let recorded_path = fs::read(
            run_root
                .join("toolchain")
                .join(format!("{binary_name}-path.stdout")),
        )
        .expect("read recorded rustup binary path output");
        let recorded_path = PathBuf::from(
            String::from_utf8(recorded_path)
                .expect("rustup binary path output UTF-8")
                .trim(),
        );
        let canonical_path = PathBuf::from(
            run_record["toolchain_binaries"][binary_name]["path"]
                .as_str()
                .expect("recorded canonical toolchain binary path"),
        );
        assert_ne!(
            recorded_path, canonical_path,
            "the positive {binary_name} fixture must record the command-name alias"
        );
        assert_eq!(
            recorded_path
                .canonicalize()
                .expect("resolve recorded toolchain binary alias"),
            canonical_path,
            "the recorded {binary_name} alias must resolve to its pinned canonical path"
        );
    }

    let rustup_home_output_path = run_root.join("toolchain/rustup-home.stdout");
    let alternate_rustup_home = fixture.repo.root().join("same-content-rustup-home");
    fs::create_dir(&alternate_rustup_home).expect("create a different empty Rustup home");
    assert_ne!(
        alternate_rustup_home
            .canonicalize()
            .expect("resolve alternate home"),
        canonical_rustup_home,
        "the alternate Rustup home must have a different canonical path"
    );
    fs::write(
        &rustup_home_output_path,
        format!("{}\n", alternate_rustup_home.display()),
    )
    .expect("replace recorded Rustup home output with a different directory");
    let alternate_home_validation = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        alternate_home_validation.exit_code,
        Some(2),
        "{}",
        error_text(&alternate_home_validation)
    );
    assert!(alternate_home_validation.signal.is_none());
    assert!(alternate_home_validation.stdout.is_empty());
    assert!(
        error_text(&alternate_home_validation)
            .contains("recorded rustup home path does not resolve to the pinned canonical path")
    );
    fs::write(&rustup_home_output_path, &recorded_home_output)
        .expect("restore recorded Rustup home output");

    for binary_name in ["cargo", "rustc"] {
        let path_output = run_root
            .join("toolchain")
            .join(format!("{binary_name}-path.stdout"));
        let original_path_output =
            fs::read(&path_output).expect("read original binary path output");
        let canonical_binary = PathBuf::from(
            run_record["toolchain_binaries"][binary_name]["path"]
                .as_str()
                .expect("recorded canonical binary path"),
        );
        let copied_binary = fixture
            .repo
            .root()
            .join(format!("same-byte-{binary_name}-copy"));
        fs::copy(&canonical_binary, &copied_binary)
            .expect("copy the pinned binary to a distinct path");
        assert_eq!(
            sha256_file(&copied_binary),
            run_record["toolchain_binaries"][binary_name]["sha256"],
            "the negative {binary_name} path probe must preserve the pinned bytes"
        );
        assert_ne!(
            copied_binary.canonicalize().expect("resolve copied binary"),
            canonical_binary
                .canonicalize()
                .expect("resolve canonical binary"),
            "the same-byte {binary_name} probe must use a different canonical path"
        );
        fs::write(&path_output, format!("{}\n", copied_binary.display()))
            .expect("replace recorded path with the same-byte copy");
        let copied_path_validation = validate(&receipt_path, &receipt_sha);
        assert_eq!(
            copied_path_validation.exit_code,
            Some(2),
            "{}",
            error_text(&copied_path_validation)
        );
        assert!(copied_path_validation.signal.is_none());
        assert!(copied_path_validation.stdout.is_empty());
        assert!(error_text(&copied_path_validation).contains(&format!(
            "recorded rustup {binary_name} path does not resolve to the pinned canonical path"
        )));
        fs::write(&path_output, &original_path_output)
            .expect("restore recorded binary path output");
    }

    let rustup_argv_path = run_root.join("toolchain/rustup-home.argv.json");
    let original_rustup_argv = fs::read(&rustup_argv_path).expect("read recorded rustup-home argv");
    let pinned_rustup_path = PathBuf::from(
        run_record["rustup_path"]
            .as_str()
            .expect("recorded pinned rustup path"),
    );
    let pinned_rustup_canonical = pinned_rustup_path
        .canonicalize()
        .expect("resolve recorded pinned rustup path");
    assert_eq!(
        pinned_rustup_path, pinned_rustup_canonical,
        "the recorded pinned rustup path must already be canonical"
    );
    let pinned_rustup_sha256 = run_record["rustup_sha256"]
        .as_str()
        .expect("recorded pinned rustup SHA-256");
    assert_eq!(
        sha256_file(&pinned_rustup_canonical),
        pinned_rustup_sha256,
        "the recorded pinned rustup digest must bind its actual bytes"
    );
    let wrong_executable = fixture.repo.root().join("wrong-rustup-executable");
    fs::copy(&pinned_rustup_canonical, &wrong_executable)
        .expect("copy the pinned rustup executable to a distinct path");
    assert_eq!(
        sha256_file(&wrong_executable),
        pinned_rustup_sha256,
        "the wrong-path probe must retain the pinned rustup executable bytes"
    );
    assert_ne!(
        wrong_executable
            .canonicalize()
            .expect("resolve copied executable"),
        pinned_rustup_canonical,
        "the same-byte probe must use a different resolved executable path"
    );
    let wrong_alias = fixture.repo.root().join("wrong-rustup-alias");
    symlink(&wrong_executable, &wrong_alias).expect("create a symlink resolving to the wrong path");
    let mut wrong_alias_argv: Value =
        serde_json::from_slice(&original_rustup_argv).expect("parse recorded rustup-home argv");
    wrong_alias_argv[0] = json!(wrong_alias.display().to_string());
    fs::write(
        &rustup_argv_path,
        serde_json::to_vec(&wrong_alias_argv).expect("serialize wrong-path rustup-home argv"),
    )
    .expect("write wrong-path rustup-home argv probe");
    let wrong_alias_validation = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        wrong_alias_validation.exit_code,
        Some(2),
        "{}",
        error_text(&wrong_alias_validation)
    );
    assert!(wrong_alias_validation.signal.is_none());
    assert!(wrong_alias_validation.stdout.is_empty());
    assert!(
        error_text(&wrong_alias_validation)
            .contains("recorded rustup-home executable path does not resolve to the pinned path")
    );
    fs::write(&rustup_argv_path, &original_rustup_argv)
        .expect("restore recorded rustup-home argv after wrong-path probe");

    let cargo_path_argv_path = run_root.join("toolchain/cargo-path.argv.json");
    let original_cargo_path_argv =
        fs::read(&cargo_path_argv_path).expect("read recorded cargo-path argv");
    let mut wrong_program_argv: Value =
        serde_json::from_slice(&original_cargo_path_argv).expect("parse recorded cargo-path argv");
    assert_eq!(
        wrong_program_argv.as_array().map(Vec::len),
        Some(5),
        "the recorded rustup which query must keep its full argument list"
    );
    wrong_program_argv[4] = json!("rustc");
    fs::write(
        &cargo_path_argv_path,
        serde_json::to_vec(&wrong_program_argv).expect("serialize wrong rustup program argv"),
    )
    .expect("write wrong rustup program argv probe");
    let wrong_program_validation = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        wrong_program_validation.exit_code,
        Some(2),
        "{}",
        error_text(&wrong_program_validation)
    );
    assert!(wrong_program_validation.signal.is_none());
    assert!(wrong_program_validation.stdout.is_empty());
    assert!(
        error_text(&wrong_program_validation)
            .contains("recorded cargo-path argv tail differs from the pinned query")
    );
    fs::write(&cargo_path_argv_path, &original_cargo_path_argv)
        .expect("restore recorded cargo-path argv after wrong-program probe");

    for (case, invalid_tree_oid) in [
        ("malformed-40", "g".repeat(40)),
        ("sha256-width", "0".repeat(64)),
        ("uppercase-40", "A".repeat(40)),
    ] {
        let mut invalid_receipt = receipt.clone();
        invalid_receipt["source_snapshot"]["tree_oid"] = json!(invalid_tree_oid);
        let invalid_receipt_bytes =
            serde_json::to_vec(&invalid_receipt).expect("serialize invalid tree OID receipt");
        fs::write(&receipt_path, &invalid_receipt_bytes)
            .expect("write disposable invalid tree OID receipt");

        let mut invalid_run: Value = serde_json::from_slice(&original_run_bytes)
            .expect("parse original run for invalid tree OID probe");
        invalid_run["source_snapshot"]["tree_oid"] = json!(invalid_tree_oid);
        invalid_run["tag_builder_receipt_sha256"] = json!(sha256_bytes(&invalid_receipt_bytes));
        fs::write(
            &run_path,
            serde_json::to_vec(&invalid_run).expect("serialize invalid tree OID run"),
        )
        .expect("write disposable invalid tree OID run");

        let invalid_tree = validate(&receipt_path, &sha256_bytes(&invalid_receipt_bytes));
        assert_eq!(
            invalid_tree.exit_code,
            Some(2),
            "{case}: {}",
            error_text(&invalid_tree)
        );
        assert!(invalid_tree.stdout.is_empty(), "{case} emitted an identity");
        assert!(
            error_text(&invalid_tree)
                .contains("snapshot tree_oid must be a lowercase 40-character Git object ID"),
            "{case} did not fail at Git tree OID shape validation: {}",
            error_text(&invalid_tree)
        );
    }
    fs::write(&receipt_path, &receipt_bytes).expect("restore original disposable tag receipt");
    fs::write(&run_path, &original_run_bytes).expect("restore original disposable run record");

    let mut mismatched_environment_receipt = receipt.clone();
    let original_tmpdir = receipt["build_environment"]["environment"]["oracle"]["tmpdir"]
        .as_str()
        .expect("recorded oracle temp directory");
    assert!(Path::new(original_tmpdir).is_absolute());
    let mismatched_tmpdir = PathBuf::from(original_tmpdir).join("receipt-only-tamper");
    assert!(mismatched_tmpdir.is_absolute());
    mismatched_environment_receipt["build_environment"]["environment"]["oracle"]["tmpdir"] =
        json!(mismatched_tmpdir.display().to_string());
    let mismatched_environment_bytes = serde_json::to_vec(&mismatched_environment_receipt)
        .expect("serialize receipt with one altered environment fact");
    fs::write(&receipt_path, &mismatched_environment_bytes)
        .expect("write disposable environment-mismatch receipt");
    let original_run: Value =
        serde_json::from_slice(&original_run_bytes).expect("parse original run record");
    let mut mismatched_environment_run = original_run.clone();
    mismatched_environment_run["tag_builder_receipt_sha256"] =
        json!(sha256_bytes(&mismatched_environment_bytes));
    assert_eq!(
        mismatched_environment_run["build_environment"], original_run["build_environment"],
        "the negative probe must leave the run environment unchanged"
    );
    fs::write(
        &run_path,
        serde_json::to_vec(&mismatched_environment_run)
            .expect("serialize run with updated receipt digest only"),
    )
    .expect("write environment-mismatch run record");
    let mismatched_environment =
        validate(&receipt_path, &sha256_bytes(&mismatched_environment_bytes));
    assert_eq!(
        mismatched_environment.exit_code,
        Some(2),
        "{}",
        error_text(&mismatched_environment)
    );
    assert!(mismatched_environment.signal.is_none());
    assert!(mismatched_environment.stdout.is_empty());
    assert!(
        error_text(&mismatched_environment)
            .contains("tag isolated environment facts differ from the run record")
    );
    fs::write(&receipt_path, &receipt_bytes).expect("restore original disposable tag receipt");
    fs::write(&run_path, &original_run_bytes).expect("restore original disposable run record");

    let wrong_hash = validate(&receipt_path, &"0".repeat(64));
    assert_eq!(wrong_hash.exit_code, Some(2), "{}", error_text(&wrong_hash));
    assert!(
        wrong_hash.stdout.is_empty(),
        "rejected tag identity must not be emitted"
    );
    assert!(String::from_utf8_lossy(&wrong_hash.stderr).contains("tag receipt raw SHA-256"));

    let receipt_alias = run_root
        .parent()
        .expect("tag evidence parent")
        .join("receipt-alias");
    symlink(&run_root, &receipt_alias).expect("create symlinked receipt ancestor probe");
    let symlinked_receipt = validate(
        &receipt_alias.join("tag-builder-receipt.json"),
        &receipt_sha,
    );
    assert_eq!(
        symlinked_receipt.exit_code,
        Some(2),
        "{}",
        error_text(&symlinked_receipt)
    );
    assert!(symlinked_receipt.stdout.is_empty());
    assert!(String::from_utf8_lossy(&symlinked_receipt.stderr).contains("symlink component"));
    fs::remove_file(&receipt_alias).expect("remove test-owned receipt ancestor symlink");

    let alternates_path = repository.join(".git/objects/info/alternates");
    let alternates_bytes = fs::read(&alternates_path).expect("read test-owned object alternate");
    let promisor_remote = fixture.repo.root().join("empty-promisor-remote.git");
    let promisor_remote_arg = promisor_remote.display().to_string();
    git_success(
        fixture.repo.root(),
        &["init", "--bare", "--quiet", promisor_remote_arg.as_str()],
    );
    let upload_pack_marker = fixture.repo.root().join("promisor-upload-pack-invoked");
    let upload_pack_probe = fixture.repo.root().join("promisor-upload-pack-probe");
    let marker_path = upload_pack_marker.display().to_string();
    assert!(
        !marker_path.contains('\''),
        "fixture path must be shell-quote safe"
    );
    fs::write(
        &upload_pack_probe,
        format!("#!/bin/sh\nprintf invoked > '{marker_path}'\nexit 73\n"),
    )
    .expect("write local no-network promisor upload-pack probe");
    fs::set_permissions(&upload_pack_probe, fs::Permissions::from_mode(0o755))
        .expect("make local promisor upload-pack probe executable");
    let upload_pack_arg = upload_pack_probe.display().to_string();
    git_success(
        &repository,
        &["config", "extensions.partialClone", "origin"],
    );
    git_success(&repository, &["config", "remote.origin.promisor", "true"]);
    git_success(
        &repository,
        &["config", "remote.origin.url", promisor_remote_arg.as_str()],
    );
    git_success(
        &repository,
        &[
            "config",
            "remote.origin.uploadpack",
            upload_pack_arg.as_str(),
        ],
    );
    fs::remove_file(&alternates_path).expect("hide the fixed tag object from this disposable repo");
    let object_store = repository.join(".git/objects");

    // Prove this is a promisor-missing object case: with lazy fetch enabled,
    // Git reaches only the local probe helper, which records the attempt and
    // exits before any network or object transfer can occur.
    let lazy_fetch_control = Command::new("git")
        .current_dir(&repository)
        .args([
            "--no-replace-objects",
            "cat-file",
            "-t",
            REFERENCE_TAG_OBJECT,
        ])
        .env_clear()
        .env("PATH", env::var_os("PATH").unwrap_or_default())
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env_remove("GIT_NO_LAZY_FETCH")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("run local promisor fetch control");
    assert!(!lazy_fetch_control.status.success());
    assert!(
        upload_pack_marker.is_file(),
        "promisor control did not reach its local helper"
    );
    fs::remove_file(&upload_pack_marker).expect("reset local promisor fetch marker");
    let object_store_before = git_object_store_digest(&object_store);
    let fixture_trace_before =
        fs::read(&fixture.trace_path).expect("read completed fake-tool trace");

    let missing_promisor_object = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        missing_promisor_object.exit_code,
        Some(2),
        "{}",
        error_text(&missing_promisor_object)
    );
    assert!(missing_promisor_object.signal.is_none());
    assert!(missing_promisor_object.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&missing_promisor_object.stderr)
            .contains("pinned Git object query failed")
    );
    assert!(
        !upload_pack_marker.exists(),
        "validator attempted a promisor fetch despite its no-lazy-fetch policy"
    );
    assert_eq!(
        fs::read(&fixture.trace_path).expect("read fake-tool trace after validation"),
        fixture_trace_before,
        "read-only validator invoked a fake toolchain or Cargo command"
    );
    assert_eq!(
        git_object_store_digest(&object_store),
        object_store_before,
        "missing-object validation changed the Git object store"
    );
    assert_eq!(
        sha256_file(&receipt_path),
        receipt_sha,
        "missing-object validation changed the builder receipt"
    );
    assert_eq!(
        sha256_file(&run_path),
        sha256_bytes(&original_run_bytes),
        "missing-object validation changed the builder run record"
    );

    for key in [
        "extensions.partialClone",
        "remote.origin.promisor",
        "remote.origin.url",
        "remote.origin.uploadpack",
    ] {
        git_success(&repository, &["config", "--unset-all", key]);
    }
    fs::write(&alternates_path, alternates_bytes)
        .expect("restore the test-owned alternate for later source probes");

    fs::set_permissions(
        &cargo_toml,
        fs::Permissions::from_mode(original_cargo_mode | 0o100),
    )
    .expect("introduce executable-bit drift");
    let mode_drift = validate(&receipt_path, &receipt_sha);
    assert_eq!(mode_drift.exit_code, Some(2), "{}", error_text(&mode_drift));
    assert!(mode_drift.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&mode_drift.stderr).contains("snapshot executable mode differs")
    );
    fs::set_permissions(&cargo_toml, fs::Permissions::from_mode(original_cargo_mode))
        .expect("restore Cargo.toml mode");

    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root for disposable probe");
    let extra_directory = source_root.join("unexpected-empty");
    fs::create_dir(&extra_directory).expect("create extra empty snapshot directory");
    fs::set_permissions(&extra_directory, fs::Permissions::from_mode(0o555))
        .expect("make extra empty directory read-only");
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("restore snapshot root mode");
    let extra_directory_result = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        extra_directory_result.exit_code,
        Some(2),
        "{}",
        error_text(&extra_directory_result)
    );
    assert!(extra_directory_result.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&extra_directory_result.stderr).contains("directory set differs")
    );
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root for cleanup");
    fs::remove_dir(&extra_directory).expect("remove test-owned empty directory");
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("restore snapshot root mode after cleanup");

    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root for type probe");
    fs::remove_file(&cargo_toml).expect("remove test-owned Cargo.toml copy");
    symlink("Cargo.lock", &cargo_toml)
        .expect("substitute same-name symlink for regular source file");
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("restore snapshot root mode");
    let type_drift = validate(&receipt_path, &receipt_sha);
    assert_eq!(type_drift.exit_code, Some(2), "{}", error_text(&type_drift));
    assert!(type_drift.stdout.is_empty());
    assert!(String::from_utf8_lossy(&type_drift.stderr).contains("snapshot symlink type differs"));
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root for cleanup");
    fs::remove_file(&cargo_toml).expect("remove test-owned symlink");
    fs::write(&cargo_toml, &original_cargo_bytes).expect("restore test-owned Cargo.toml bytes");
    fs::set_permissions(&cargo_toml, fs::Permissions::from_mode(original_cargo_mode))
        .expect("restore Cargo.toml read-only mode");
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("restore snapshot root mode after cleanup");

    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root for unsupported-node probe");
    fs::remove_file(&cargo_toml).expect("remove tracked Cargo.toml for FIFO mode probe");
    let mkfifo = Command::new("mkfifo")
        .arg(&cargo_toml)
        .output()
        .expect("replace tracked Cargo.toml with a test-owned FIFO");
    assert!(
        mkfifo.status.success(),
        "mkfifo failed: {}",
        String::from_utf8_lossy(&mkfifo.stderr)
    );
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("preserve the tracked parent directory read-only before FIFO validation");
    let unsupported_mode = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        unsupported_mode.exit_code,
        Some(2),
        "{}",
        error_text(&unsupported_mode)
    );
    assert!(unsupported_mode.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&unsupported_mode.stderr)
            .contains("snapshot contains an unsupported filesystem entry")
    );
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o755))
        .expect("temporarily open snapshot root to restore tracked Cargo.toml");
    fs::remove_file(&cargo_toml).expect("remove test-owned FIFO");
    fs::write(&cargo_toml, &original_cargo_bytes).expect("restore tracked Cargo.toml bytes");
    fs::set_permissions(&cargo_toml, fs::Permissions::from_mode(original_cargo_mode))
        .expect("restore tracked Cargo.toml mode");
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o555))
        .expect("restore snapshot root mode after unsupported-node probe");

    let mut wrong_tree_receipt = receipt.clone();
    wrong_tree_receipt["source_snapshot"]["tree_oid"] = json!("0".repeat(40));
    let wrong_tree_receipt_bytes =
        serde_json::to_vec(&wrong_tree_receipt).expect("serialize wrong-tree receipt probe");
    fs::write(&receipt_path, &wrong_tree_receipt_bytes).expect("write wrong-tree receipt probe");
    let mut wrong_tree_run: Value = serde_json::from_slice(&original_run_bytes)
        .expect("parse original run for wrong-tree probe");
    wrong_tree_run["source_snapshot"]["tree_oid"] = json!("0".repeat(40));
    wrong_tree_run["tag_builder_receipt_sha256"] = json!(sha256_bytes(&wrong_tree_receipt_bytes));
    fs::write(
        &run_path,
        serde_json::to_vec(&wrong_tree_run).expect("serialize wrong-tree run probe"),
    )
    .expect("write wrong-tree run probe");
    let wrong_tree = validate(&receipt_path, &sha256_bytes(&wrong_tree_receipt_bytes));
    assert_eq!(wrong_tree.exit_code, Some(2), "{}", error_text(&wrong_tree));
    assert!(wrong_tree.stdout.is_empty());
    assert!(String::from_utf8_lossy(&wrong_tree.stderr).contains("snapshot tree oid differs"));

    let mut malformed_env_receipt = receipt.clone();
    malformed_env_receipt["build_environment"]["external_cargo_config_policy"] =
        json!("allow-ambient-global-config");
    let malformed_env_bytes =
        serde_json::to_vec(&malformed_env_receipt).expect("serialize malformed environment probe");
    fs::write(&receipt_path, &malformed_env_bytes).expect("write malformed environment probe");
    let mut malformed_env_run: Value = serde_json::from_slice(&original_run_bytes)
        .expect("parse original run for environment probe");
    malformed_env_run["tag_builder_receipt_sha256"] = json!(sha256_bytes(&malformed_env_bytes));
    fs::write(
        &run_path,
        serde_json::to_vec(&malformed_env_run).expect("serialize environment-probe run"),
    )
    .expect("write environment-probe run");
    let malformed_environment = validate(&receipt_path, &sha256_bytes(&malformed_env_bytes));
    assert_eq!(
        malformed_environment.exit_code,
        Some(2),
        "{}",
        error_text(&malformed_environment)
    );
    assert!(malformed_environment.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&malformed_environment.stderr)
            .contains("Cargo configuration policy")
    );

    fs::write(&receipt_path, &receipt_bytes).expect("restore original tag receipt bytes");
    fs::write(&run_path, &original_run_bytes).expect("restore original tag run bytes");
    let executable_path = PathBuf::from(
        receipt["executable"]["path"]
            .as_str()
            .expect("tag executable path"),
    );
    let executable_mode = fs::metadata(&executable_path)
        .expect("inspect synthetic tag executable")
        .permissions()
        .mode();
    fs::set_permissions(
        &executable_path,
        fs::Permissions::from_mode(executable_mode | 0o200),
    )
    .expect("make synthetic executable writable for digest probe");
    OpenOptions::new()
        .append(true)
        .open(&executable_path)
        .expect("open synthetic executable for digest probe")
        .write_all(b"changed")
        .expect("change synthetic executable bytes");
    fs::set_permissions(
        &executable_path,
        fs::Permissions::from_mode(executable_mode),
    )
    .expect("restore synthetic executable mode");
    let executable_drift = validate(&receipt_path, &receipt_sha);
    assert_eq!(
        executable_drift.exit_code,
        Some(2),
        "{}",
        error_text(&executable_drift)
    );
    assert!(executable_drift.stdout.is_empty());
    assert!(String::from_utf8_lossy(&executable_drift.stderr).contains("executable bytes"));

    fixture.cleanup_tag_source_for_run(run_id);
}

#[test]
fn builds_a_synthetic_tablepro_subject_pair_from_exact_git_objects() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let run_id = "tablepro-synthetic-positive";
    let args = fixture.base_args(run_id);
    let output = fixture.run(&args, &[]);
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    assert!(output.signal.is_none());

    let manifest_path = fixture.manifest_path(run_id);
    assert_eq!(PathBuf::from(output_text(&output).trim()), manifest_path);
    let manifest_bytes = fs::read(&manifest_path).expect("read generated manifest");
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).expect("parse generated manifest");
    assert_eq!(
        manifest["schema"],
        "termrock-spec/parity-subject-manifest-v2"
    );
    assert_eq!(manifest["run_id"], run_id);
    assert_eq!(manifest["suite_revision"], "synthetic-suite-revision");
    assert_eq!(manifest["suite_sha256"], SUITE_SHA256);
    assert_eq!(
        manifest["expected_generation"]["id"],
        "synthetic-generation"
    );
    assert_eq!(manifest["expected_generation"]["sha256"], GENERATION_SHA256);
    assert_eq!(
        manifest["expected_generation"]["root"],
        fixture.expected_root.display().to_string()
    );

    let top_level: BTreeSet<&str> = manifest
        .as_object()
        .expect("manifest object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        top_level,
        BTreeSet::from([
            "build_evidence",
            "expected_generation",
            "run_id",
            "schema",
            "subjects",
            "suite_revision",
            "suite_sha256",
        ])
    );
    let evidence_root = fixture.repo.root().join("evidence").join(run_id);
    let source_input_path = evidence_root.join("source-inputs.json");
    let source_input_bytes = fs::read(&source_input_path).expect("read source-inputs sidecar");
    let build_environment_path = evidence_root.join("build-environment.json");
    let build_environment_bytes =
        fs::read(&build_environment_path).expect("read build-environment sidecar");
    assert_eq!(
        object_keys(&manifest["build_evidence"]),
        BTreeSet::from(["source_inputs".to_owned(), "build_environment".to_owned()])
    );
    assert_eq!(
        manifest["build_evidence"]["source_inputs"]["path"],
        "source-inputs.json"
    );
    assert_eq!(
        manifest["build_evidence"]["source_inputs"]["sha256"],
        sha256_bytes(&source_input_bytes)
    );
    assert_eq!(
        manifest["build_evidence"]["build_environment"]["path"],
        "build-environment.json"
    );
    assert_eq!(
        manifest["build_evidence"]["build_environment"]["sha256"],
        sha256_bytes(&build_environment_bytes)
    );
    assert!(
        !String::from_utf8_lossy(&manifest_bytes).contains("run.json"),
        "manifest must not point back to the final builder run"
    );

    let subjects = manifest["subjects"].as_array().expect("subjects array");
    assert_eq!(subjects.len(), 2);
    for (subject, role, commit, output_root) in [
        (
            &subjects[0],
            "reference",
            fixture.reference_sha.as_str(),
            &fixture.reference_output_root,
        ),
        (
            &subjects[1],
            "candidate",
            fixture.candidate_sha.as_str(),
            &fixture.candidate_output_root,
        ),
    ] {
        assert_eq!(subject["role"], role);
        assert_eq!(subject["source_commit"], commit);
        assert_eq!(subject["build"]["target_name"], "tablepro");
        assert_eq!(subject["build"]["toolchain"], "1.98.1");
        assert_eq!(subject["build"]["profile"], "release");
        assert_eq!(subject["build"]["target_triple"], native_target());
        assert_eq!(subject["build"]["default_features"], true);
        assert_eq!(subject["build"]["features"], json!([]));
        assert_eq!(
            subject["actual_output_root"],
            output_root.display().to_string()
        );

        let executable_path = PathBuf::from(
            subject["executable"]["path"]
                .as_str()
                .expect("executable path string"),
        );
        assert!(executable_path.is_absolute());
        assert_eq!(
            subject["executable"]["sha256"],
            sha256_file(&executable_path)
        );
        assert_eq!(
            subject["builder_receipt"]["executable_sha256"],
            subject["executable"]["sha256"]
        );
        assert_eq!(
            subject["builder_receipt"]["source_commit"],
            subject["source_commit"]
        );
        assert_eq!(
            subject["build_inputs"]["manifest_sha256"],
            subject["builder_receipt"]["manifest_sha256"]
        );
        assert_eq!(
            subject["build_inputs"]["lock_sha256"],
            subject["builder_receipt"]["lock_sha256"]
        );
        for key in [
            "cargo_version",
            "rustc_version",
            "host_triple",
            "executed_argv",
        ] {
            assert_eq!(
                subject["build_inputs"][key],
                subject["builder_receipt"][key]
            );
        }

        let receipt = subject["builder_receipt"]
            .as_object()
            .expect("builder receipt object");
        let mut receipt_payload = receipt.clone();
        let receipt_sha = receipt_payload
            .remove("sha256")
            .and_then(|value| value.as_str().map(str::to_owned))
            .expect("receipt digest");
        assert_eq!(
            receipt_sha,
            sha256_bytes(&serde_json::to_vec(&receipt_payload).expect("serialize receipt payload"))
        );
        assert_eq!(subject["builder_receipt_sha256"], receipt_sha);
    }

    let trace: Vec<Value> = fs::read_to_string(&fixture.trace_path)
        .expect("read fake rustup/cargo trace")
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse trace line"))
        .collect();
    let metadata_calls: Vec<_> = trace
        .iter()
        .filter(|call| call["argv"][3] == "metadata")
        .collect();
    let build_calls: Vec<_> = trace
        .iter()
        .filter(|call| call["argv"][3] == "build")
        .collect();
    assert_eq!(metadata_calls.len(), 2);
    assert_eq!(build_calls.len(), 2);
    assert!(trace.iter().all(|call| call["program"] == "rustup"));
    for call in build_calls {
        let argv = call["argv"].as_array().expect("build argv");
        let argv: Vec<&str> = argv.iter().map(|arg| arg.as_str().unwrap()).collect();
        for required in [
            "--release",
            "--locked",
            "--offline",
            "--jobs",
            "2",
            "--message-format=json-render-diagnostics",
            "--target",
            native_target(),
            "--target-dir",
        ] {
            assert!(argv.contains(&required), "missing `{required}` in {argv:?}");
        }
        assert!(argv.windows(2).any(|pair| pair == ["--bin", "tablepro"]));
    }

    let run_record: Value = serde_json::from_slice(
        &fs::read(fixture.run_record_path(run_id)).expect("read run record"),
    )
    .expect("parse run record");
    assert_eq!(run_record["state"], "complete");
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert_eq!(run_record["subject_schema_sha256"], SUBJECT_SCHEMA_SHA256);
    assert_eq!(
        run_record["subject_manifest"],
        manifest_path.display().to_string()
    );
    assert_eq!(
        run_record["subject_manifest_sha256"],
        sha256_bytes(&manifest_bytes)
    );
    assert_eq!(
        object_keys(&run_record),
        BTreeSet::from(
            [
                "schema",
                "state",
                "qualification",
                "run_id",
                "app",
                "subject_schema_sha256",
                "reference_tag_object",
                "reference_tag_commit",
                "reference_branch_ref",
                "reference_commit",
                "candidate_commit",
                "target_triple",
                "toolchain",
                "rustup_path",
                "rustup_sha256",
                "rustup_home",
                "toolchain_binaries",
                "git_identity",
                "reference_branch_oid",
                "git_object_replacement_policy",
                "cargo_version",
                "rustc_version",
                "host_triple",
                "snapshots",
                "snapshot_materialization",
                "cargo_config_sha256",
                "source_input_evidence",
                "build_environment",
                "build_environment_evidence",
                "metadata_targets",
                "subject_manifest",
                "subject_manifest_sha256",
            ]
            .map(str::to_owned)
        )
    );
    assert_eq!(
        run_record["git_object_replacement_policy"],
        "disabled-by-option-and-environment"
    );
    for label in [
        "reference-tag-object",
        "reference-tag-commit",
        "reference-branch-object",
        "reference-branch-object-type",
        "candidate-branch-object",
        "candidate-branch-object-type",
        "reference-commit",
        "reference-branch-membership",
        "candidate-commit",
    ] {
        let argv: Value = serde_json::from_slice(
            &fs::read(
                fixture
                    .repo
                    .root()
                    .join("evidence")
                    .join(run_id)
                    .join("git")
                    .join(format!("{label}.argv.json")),
            )
            .expect("read Git identity argv"),
        )
        .expect("parse Git identity argv");
        assert_eq!(argv[1], "--no-replace-objects");
    }
    assert_eq!(run_record["reference_tag_object"], REFERENCE_TAG_OBJECT);
    assert_eq!(run_record["reference_tag_commit"], ORACLE_COMMIT_SHA);
    assert_eq!(run_record["reference_branch_ref"], REFERENCE_BRANCH_REF);
    assert_eq!(run_record["reference_commit"], fixture.reference_sha);
    assert_eq!(run_record["candidate_commit"], fixture.candidate_sha);
    let reference_branch_oid = run_record["reference_branch_oid"]
        .as_str()
        .expect("reference branch oid");
    assert_eq!(reference_branch_oid.len(), 40);
    assert!(
        reference_branch_oid
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    );
    assert_eq!(
        fs::read_to_string(
            fixture
                .repo
                .root()
                .join("evidence")
                .join(run_id)
                .join("git/reference-branch-object.stdout"),
        )
        .expect("read selected branch ref oid")
        .trim(),
        reference_branch_oid
    );
    assert_eq!(
        fs::read_to_string(
            fixture
                .repo
                .root()
                .join("evidence")
                .join(run_id)
                .join("git/candidate-branch-object.stdout"),
        )
        .expect("read selected candidate branch ref oid")
        .trim(),
        fixture.candidate_sha
    );
    let membership_argv: Value = serde_json::from_slice(
        &fs::read(
            fixture
                .repo
                .root()
                .join("evidence")
                .join(run_id)
                .join("git/reference-branch-membership.argv.json"),
        )
        .expect("read branch membership argv"),
    )
    .expect("parse branch membership argv");
    assert_eq!(membership_argv[1], "--no-replace-objects");
    assert_eq!(membership_argv[2], "merge-base");
    assert_eq!(membership_argv[3], "--is-ancestor");
    assert_eq!(membership_argv[4], fixture.reference_sha);
    assert_eq!(membership_argv[5], reference_branch_oid);
    assert_eq!(
        run_record["subject_manifest_sha256"],
        sha256_bytes(&manifest_bytes)
    );
    let environment_evidence_path = PathBuf::from(
        run_record["build_environment_evidence"]["path"]
            .as_str()
            .expect("environment evidence path"),
    );
    let environment_bytes =
        fs::read(&environment_evidence_path).expect("read environment evidence");
    assert_eq!(
        run_record["build_environment_evidence"]["sha256"],
        sha256_bytes(&environment_bytes)
    );
    let environment: Value =
        serde_json::from_slice(&environment_bytes).expect("parse environment evidence");
    assert_eq!(
        object_keys(&environment),
        BTreeSet::from(
            [
                "schema",
                "run_id",
                "source_input_evidence",
                "source_commits",
                "reference_branch",
                "oracle_lineage",
                "cargo_config_sha256",
                "external_cargo_config_policy",
                "rustup",
                "toolchain_binaries",
                "cargo_version",
                "rustc_version",
                "host_triple",
                "target_triple",
                "environment",
            ]
            .map(str::to_owned)
        )
    );
    let source_input_evidence_path = PathBuf::from(
        run_record["source_input_evidence"]["path"]
            .as_str()
            .expect("source input evidence path"),
    );
    let source_input_bytes =
        fs::read(&source_input_evidence_path).expect("read source input evidence");
    assert_eq!(
        run_record["source_input_evidence"]["sha256"],
        sha256_bytes(&source_input_bytes)
    );
    assert_eq!(
        environment["source_input_evidence"]["sha256"],
        run_record["source_input_evidence"]["sha256"]
    );
    let source_input: Value =
        serde_json::from_slice(&source_input_bytes).expect("parse source input evidence");
    assert_eq!(
        object_keys(&source_input),
        BTreeSet::from(
            [
                "schema",
                "run_id",
                "qualification_status",
                "qualification_reason",
                "git_object_replacement_policy",
                "oracle_lineage",
                "reference_branch",
                "recipe",
                "excluded_roots",
                "path_blob_digest_algorithm",
                "sources",
            ]
            .map(str::to_owned)
        )
    );
    assert_eq!(
        source_input["schema"],
        "termrock-spec/parity-subject-source-inputs-v1"
    );
    assert_eq!(source_input["qualification_status"], "blocked");
    let source_facts_context = format!(
        "source-facts-sha256={}; reference-ref={}/{}; candidate-ref={}/{}",
        fixture.source_facts_sha256,
        REFERENCE_BRANCH_REF,
        fixture.reference_sha,
        CANDIDATE_BRANCH_REF,
        fixture.candidate_sha,
    );
    let source_reason = source_input["qualification_reason"]
        .as_str()
        .expect("blocked source qualification reason");
    let run_reason = run_record["qualification"]["reason"]
        .as_str()
        .expect("blocked run qualification reason");
    assert_ne!(source_reason, run_reason);
    assert!(source_reason.contains(&source_facts_context));
    assert!(run_reason.contains(&source_facts_context));
    assert!(source_reason.to_lowercase().contains("blocked"));
    assert!(run_reason.to_lowercase().contains("blocked"));
    assert_eq!(
        source_input["git_object_replacement_policy"],
        "disabled-by-option-and-environment"
    );
    assert_eq!(
        source_input["recipe"],
        "git-archive-excluding-parity-oracle-roots-v1"
    );
    assert_eq!(
        source_input["excluded_roots"],
        json!(["snapshots/**", "baselines/**"])
    );
    assert_eq!(
        source_input["path_blob_digest_algorithm"],
        "sha256(sorted-raw-path-nul-git-blob-oid-lf-v1)"
    );
    assert_eq!(
        source_input["oracle_lineage"]["tag_ref"],
        "refs/tags/visual-baseline"
    );
    assert_eq!(
        source_input["oracle_lineage"]["tag_object"],
        REFERENCE_TAG_OBJECT
    );
    assert_eq!(
        source_input["oracle_lineage"]["tag_commit"],
        ORACLE_COMMIT_SHA
    );
    assert_eq!(
        source_input["reference_branch"]["ref"],
        REFERENCE_BRANCH_REF
    );
    assert_eq!(
        source_input["reference_branch"]["oid"],
        reference_branch_oid
    );
    assert_eq!(
        source_input["reference_branch"]["selected_commit"],
        fixture.reference_sha
    );
    assert_eq!(
        source_input["reference_branch"]["membership"],
        "verified-reachable"
    );
    assert_eq!(
        object_keys(&source_input["sources"]),
        BTreeSet::from(["reference".to_owned(), "candidate".to_owned()])
    );
    assert_eq!(
        run_record["snapshot_materialization"],
        source_input["sources"]
    );
    assert_eq!(run_record["build_environment"], environment["environment"]);
    assert_eq!(
        run_record["cargo_config_sha256"],
        environment["cargo_config_sha256"]
    );
    assert_eq!(
        environment["source_input_evidence"]["path"],
        run_record["source_input_evidence"]["path"]
    );
    assert_eq!(
        environment["source_input_evidence"]["sha256"],
        run_record["source_input_evidence"]["sha256"]
    );
    assert_eq!(
        run_record["build_environment_evidence"]["path"],
        environment_evidence_path.display().to_string()
    );
    assert_eq!(
        run_record["build_environment_evidence"]["sha256"],
        sha256_bytes(&environment_bytes)
    );
    for (role, commit) in [
        ("reference", fixture.reference_sha.as_str()),
        ("candidate", fixture.candidate_sha.as_str()),
    ] {
        let source = &source_input["sources"][role];
        assert_eq!(
            object_keys(source),
            BTreeSet::from(
                [
                    "role",
                    "path",
                    "source_commit",
                    "tree_oid",
                    "git_object_replacement_policy",
                    "recipe",
                    "archive_argv",
                    "git_ls_tree_stdout_sha256",
                    "excluded_roots",
                    "path_blob_digest_algorithm",
                    "tracked_path_blob_map_sha256",
                    "tracked_file_count",
                    "included_path_blob_map_sha256",
                    "included_file_count",
                    "excluded_path_blob_map_sha256",
                    "excluded_file_count",
                    "archive_sha256",
                    "archive_member_count",
                    "read_only",
                ]
                .map(str::to_owned)
            )
        );
        assert_eq!(source["source_commit"], commit);
        assert_eq!(
            source["git_object_replacement_policy"],
            "disabled-by-option-and-environment"
        );
        assert_eq!(source["recipe"], source_input["recipe"]);
        assert_eq!(source["excluded_roots"], source_input["excluded_roots"]);
        assert_eq!(
            source["path_blob_digest_algorithm"],
            source_input["path_blob_digest_algorithm"]
        );
        assert_eq!(source["read_only"], true);
        assert_eq!(
            source["tracked_file_count"].as_u64().unwrap(),
            source["included_file_count"].as_u64().unwrap()
                + source["excluded_file_count"].as_u64().unwrap()
        );
        assert!(source["excluded_file_count"].as_u64().unwrap() > 0);
        assert_eq!(source["tree_oid"].as_str().unwrap().len(), 40);
        for key in [
            "tracked_path_blob_map_sha256",
            "included_path_blob_map_sha256",
            "excluded_path_blob_map_sha256",
            "archive_sha256",
            "git_ls_tree_stdout_sha256",
        ] {
            assert_eq!(source[key].as_str().unwrap().len(), 64);
        }
        assert!(source["archive_member_count"].as_u64().unwrap() > 0);
        let archive_argv = source["archive_argv"].as_array().expect("archive argv");
        assert_eq!(archive_argv[1], "--no-replace-objects");
        assert!(archive_argv.iter().any(|arg| arg == "."));
        assert!(
            archive_argv
                .iter()
                .any(|arg| arg == ":(top,exclude)snapshots/**")
        );
        assert!(
            archive_argv
                .iter()
                .any(|arg| arg == ":(top,exclude)baselines/**")
        );
        let snapshot_path = PathBuf::from(
            run_record["snapshots"][role]["path"]
                .as_str()
                .expect("snapshot path"),
        );
        assert!(!snapshot_path.join("snapshots").exists());
        assert!(!snapshot_path.join("baselines").exists());

        let snapshot_log = fixture
            .repo
            .root()
            .join("evidence")
            .join(run_id)
            .join("snapshot-logs")
            .join(role);
        for label in ["tree", "tree-oid"] {
            let argv: Value = serde_json::from_slice(
                &fs::read(snapshot_log.join(format!("{label}.argv.json")))
                    .expect("read Git snapshot argv"),
            )
            .expect("parse Git snapshot argv");
            assert_eq!(argv[1], "--no-replace-objects");
        }
        let tree_listing =
            fs::read(snapshot_log.join("tree.stdout")).expect("read raw Git tree listing");
        assert_eq!(
            source["git_ls_tree_stdout_sha256"],
            sha256_bytes(&tree_listing)
        );
        assert_eq!(
            fs::read_to_string(snapshot_log.join("tree-oid.stdout"))
                .expect("read tree object identity")
                .trim(),
            source["tree_oid"].as_str().unwrap()
        );
        let (included, excluded) = split_tree_blob_maps(&tree_listing);
        assert_eq!(
            source["tracked_file_count"].as_u64().unwrap(),
            (included.len() + excluded.len()) as u64
        );
        assert_eq!(
            source["included_file_count"].as_u64().unwrap(),
            included.len() as u64
        );
        assert_eq!(
            source["excluded_file_count"].as_u64().unwrap(),
            excluded.len() as u64
        );
        assert_eq!(
            source["tracked_path_blob_map_sha256"].as_str().unwrap(),
            path_blob_map_sha256([included.clone(), excluded.clone()].concat())
        );
        assert_eq!(
            source["included_path_blob_map_sha256"].as_str().unwrap(),
            path_blob_map_sha256(included)
        );
        assert_eq!(
            source["excluded_path_blob_map_sha256"].as_str().unwrap(),
            path_blob_map_sha256(excluded)
        );
    }
    assert_eq!(
        environment["external_cargo_config_policy"],
        "reject-any-config-from-snapshot-parent-through-filesystem-root"
    );
    let environments = &environment["environment"];
    assert_eq!(
        object_keys(environments),
        BTreeSet::from(["reference".to_owned(), "candidate".to_owned()])
    );
    for role in ["reference", "candidate"] {
        assert_eq!(
            object_keys(&environments[role]),
            BTreeSet::from(
                [
                    "home",
                    "cargo_home",
                    "cargo_home_config",
                    "cache_home",
                    "cache_links",
                    "rustup_home",
                    "tmpdir",
                    "path",
                    "platform_inputs",
                ]
                .map(str::to_owned)
            )
        );
        assert_eq!(
            run_record["build_environment"][role], environments[role],
            "run environment facts must duplicate the typed sidecar"
        );
        for path_key in ["home", "cargo_home", "cache_home", "rustup_home", "tmpdir"] {
            assert!(Path::new(environments[role][path_key].as_str().unwrap()).is_absolute());
        }
        assert!(!environments[role]["path"].as_str().unwrap().is_empty());
        for cache_path in environments[role]["cache_links"]
            .as_object()
            .unwrap()
            .values()
        {
            assert!(Path::new(cache_path.as_str().unwrap()).is_absolute());
        }
        let target = &run_record["metadata_targets"][role]["tablepro"];
        assert_eq!(
            object_keys(target),
            BTreeSet::from(
                [
                    "package_id",
                    "package_name",
                    "target_name",
                    "manifest_path",
                    "source_path",
                ]
                .map(str::to_owned)
            )
        );
        assert_eq!(target["target_name"], "tablepro");
    }
    assert_eq!(environments["reference"]["cargo_home_config"], "absent");
    assert_eq!(environments["candidate"]["cargo_home_config"], "absent");
    assert_ne!(
        environments["reference"]["cargo_home"], environments["candidate"]["cargo_home"],
        "the two source builds must not share writable Cargo config homes"
    );
    for program in ["cargo", "rustc"] {
        let tool = &environment["toolchain_binaries"][program];
        assert!(Path::new(tool["path"].as_str().unwrap()).is_absolute());
        assert_eq!(tool["sha256"].as_str().unwrap().len(), 64);
    }
    assert_eq!(
        object_keys(&environment["toolchain_binaries"]),
        BTreeSet::from(["cargo".to_owned(), "rustc".to_owned()])
    );
    assert_eq!(
        run_record["toolchain_binaries"],
        environment["toolchain_binaries"]
    );
    fixture.cleanup_sources_for_run(run_id);
}

#[test]
fn emits_nullable_expected_generation_without_inventing_a_corpus() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let run_id = "tablepro-null-generation";
    let mut args = fixture.base_args(run_id);
    for option in [
        "--expected-generation-id",
        "--expected-generation-sha256",
        "--expected-generation-root",
    ] {
        SubjectsFixture::remove_arg(&mut args, option);
    }

    let output = fixture.run(&args, &[]);
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    let manifest_bytes = fs::read(fixture.manifest_path(run_id)).expect("read v2 manifest");
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("parse v2 manifest");
    assert_eq!(
        manifest["schema"],
        "termrock-spec/parity-subject-manifest-v2"
    );
    assert!(manifest["expected_generation"].is_null());
    assert_eq!(manifest["suite_sha256"], SUITE_SHA256);
    assert_eq!(
        object_keys(&manifest),
        BTreeSet::from(
            [
                "schema",
                "run_id",
                "suite_revision",
                "suite_sha256",
                "expected_generation",
                "build_evidence",
                "subjects",
            ]
            .map(str::to_owned)
        )
    );

    let run_record: Value = serde_json::from_slice(
        &fs::read(fixture.run_record_path(run_id)).expect("read complete builder run"),
    )
    .expect("parse complete builder run");
    assert_eq!(run_record["state"], "complete");
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert!(
        !run_record["qualification"]["reason"]
            .as_str()
            .unwrap()
            .trim()
            .is_empty()
    );
    fixture.cleanup_sources_for_run(run_id);
}

#[cfg(unix)]
#[test]
fn rejects_nonportable_git_archives_and_symlink_escapes_with_replacement_refs_present() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let cases = [
        (
            "backslash-member",
            "component\\part",
            None,
            "source archive contains a non-portable path",
        ),
        (
            "drive-member",
            "C:drive-relative",
            None,
            "source archive contains a non-portable path",
        ),
        (
            "unc-member",
            "\\\\server\\share",
            None,
            "source archive contains a non-portable path",
        ),
        (
            "windows-symlink-escape",
            "portable-link",
            Some("..\\..\\outside"),
            "source archive has a non-portable symlink target",
        ),
        (
            "posix-symlink-escape",
            "portable-link",
            Some("../../outside"),
            "source archive symlink escapes the snapshot",
        ),
    ];

    for (case_name, entry_name, symlink_target, expected_message) in cases {
        let adversarial_repo = fixture.repo.root().join(format!("git-{case_name}"));
        let candidate = adversarial_candidate(
            &fixture,
            &adversarial_repo,
            case_name,
            entry_name,
            symlink_target,
        );
        let run_id = format!("archive-negative-{case_name}");
        let mut args = fixture.base_args(&run_id);
        SubjectsFixture::set_arg(
            &mut args,
            "--repository",
            adversarial_repo.display().to_string(),
        );
        SubjectsFixture::set_arg(&mut args, "--candidate-commit", candidate.clone());
        fixture.set_source_facts(&mut args, case_name, &fixture.reference_sha, &candidate);

        let output = fixture.run(&args, &[]);
        assert_failed(&output, expected_message);
        assert!(!fixture.manifest_path(&run_id).exists());
        let run_root = fixture.repo.root().join("evidence").join(&run_id);
        assert_eq!(
            fs::read_to_string(run_root.join("git/candidate-commit.stdout"))
                .expect("read resolved candidate commit")
                .trim(),
            candidate
        );
        let archive_argv: Value = serde_json::from_slice(
            &fs::read(run_root.join("snapshot-logs/candidate/archive.argv.json"))
                .expect("read archive argv"),
        )
        .expect("parse archive argv");
        assert_eq!(archive_argv[1], "--no-replace-objects");
        assert_eq!(
            fs::read_to_string(run_root.join("snapshot-logs/candidate/archive.exit"))
                .expect("read archive status")
                .trim(),
            "0"
        );
        let tree_listing = fs::read(run_root.join("snapshot-logs/candidate/tree.stdout"))
            .expect("read candidate Git tree listing");
        assert!(
            tree_listing
                .windows(entry_name.len())
                .any(|window| window == entry_name.as_bytes())
        );
        fixture.cleanup_sources_for_run(&run_id);
    }
}

#[test]
fn rejects_all_late_cargo_artifact_failures_after_pinned_source_materialization() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let cases = [
        ("missing", "found 0"),
        (
            "malformed",
            "cargo build stdout line 1 is not machine-readable JSON",
        ),
        ("duplicate", "found 2"),
        ("wrong-package", "found 0"),
        ("wrong-target", "found 0"),
        (
            "fresh",
            "Cargo reported a fresh artifact from a non-fresh target directory",
        ),
        (
            "test-profile",
            "matching compiler-artifact is not a non-test product build",
        ),
        (
            "outside-target",
            "compiler-artifact executable escaped its isolated target directory",
        ),
        (
            "non-executable",
            "compiler-artifact executable is not runnable",
        ),
    ];
    let mut completed = 0usize;

    for (mode, expected_message) in cases {
        if mode == "non-executable" && !cfg!(unix) {
            continue;
        }
        let run_id = format!("artifact-negative-{mode}");
        let args = fixture.base_args(&run_id);
        let output = fixture.run(
            &args,
            &[("VISIBILITY_FIXTURE_ARTIFACT_MODE", mode.to_owned())],
        );
        assert_failed(&output, expected_message);
        assert!(
            !fixture.manifest_path(&run_id).exists(),
            "a rejected compiler artifact must not produce a subject manifest"
        );

        let run_root = fixture.repo.root().join("evidence").join(&run_id);
        let run_record: Value = serde_json::from_slice(
            &fs::read(fixture.run_record_path(&run_id)).expect("read failed run record"),
        )
        .expect("parse failed run record");
        assert_eq!(run_record["state"], "failed");
        assert_eq!(run_record["qualification"]["status"], "blocked");
        assert!(
            run_record["error"]
                .as_str()
                .unwrap()
                .contains(expected_message)
        );

        let source_evidence_path = PathBuf::from(
            run_record["source_input_evidence"]["path"]
                .as_str()
                .expect("source input evidence path"),
        );
        let source_evidence_bytes =
            fs::read(&source_evidence_path).expect("read source input evidence");
        assert_eq!(
            run_record["source_input_evidence"]["sha256"],
            sha256_bytes(&source_evidence_bytes)
        );

        let build_dir = run_root.join("build/reference/tablepro");
        assert!(build_dir.join("build.argv.json").is_file());
        assert!(build_dir.join("build.stdout").is_file());
        assert!(build_dir.join("build.stderr").is_file());
        assert_eq!(
            fs::read_to_string(build_dir.join("build.exit")).expect("read Cargo exit code"),
            "0\n",
            "the fixture Cargo command succeeds; the launcher rejects its artifact"
        );
        assert!(
            !run_root
                .join("build/candidate/tablepro/build.argv.json")
                .exists()
        );
        fixture.cleanup_sources_for_run(&run_id);
        completed += 1;
    }

    assert_eq!(completed, if cfg!(unix) { 9 } else { 8 });
}

#[test]
fn rejects_unaccepted_source_facts_and_branch_pin_mismatches_before_source_work() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();

    let checked_in_facts_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/visibility/source-facts.json")
        .canonicalize()
        .expect("resolve checked-in source facts");
    let checked_in_facts_sha256 = sha256_file(&checked_in_facts_path);
    let copied_facts = fixture
        .repo
        .copy_project_file_exact(Path::new("tools/visibility/source-facts.json"))
        .expect("copy exact checked-in v2 source facts");
    assert_eq!(copied_facts.sha256, checked_in_facts_sha256);
    assert_eq!(sha256_file(&copied_facts.path), checked_in_facts_sha256);
    let copied_facts_json: Value = serde_json::from_slice(
        &fs::read(&copied_facts.path).expect("read copied checked-in source facts"),
    )
    .expect("parse copied checked-in v2 source facts");
    assert_eq!(copied_facts_json["schema_version"], 2);
    assert_ne!(
        fixture.candidate_sha,
        copied_facts_json["latest_source_observation"]["reference_remote"]["head_sha"]
            .as_str()
            .expect("checked-in reference pin"),
        "candidate pin must be a valid but distinct requested reference commit"
    );
    let checked_in_run_id = "source-facts-checked-in-reference-pin-mismatch";
    let mut args = fixture.base_args(checked_in_run_id);
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        copied_facts.path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        copied_facts.sha256.clone(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.candidate_sha.clone(),
    );
    let output = fixture.run(&args, &[]);
    assert_eq!(output.signal, None);
    assert_eq!(output.exit_code, Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"subjects: reference_commit does not match the latest accepted source-facts reference tip\n"
    );
    assert_no_archive_metadata_or_build(&fixture, checked_in_run_id);
    assert!(
        !fixture.run_record_path(checked_in_run_id).exists(),
        "caller-pin rejection must precede run evidence creation"
    );
    assert_eq!(sha256_file(&checked_in_facts_path), checked_in_facts_sha256);
    assert_eq!(sha256_file(&copied_facts.path), copied_facts.sha256);

    let mut args = fixture.base_args("source-facts-wrong-digest");
    SubjectsFixture::set_arg(&mut args, "--source-facts-sha256", "f".repeat(64));
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "source_facts_sha256 does not match the exact source-facts bytes",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-wrong-digest");

    let duplicate_keys_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/source-facts-duplicate-keys.json"),
            br#"{"schema_version":2,"repository":"tailrocks/terminal-components-claude","observed_at":"synthetic-test-input","latest_source_observation":{"observed_at":"synthetic-test-input","method":"synthetic Rust black-box fixture","reference_remote":{"branch":"visual-baseline","head_sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","head_sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","updated_at":"synthetic-test-input"},"candidate_remote":{"branch":"termrock-implementation","head_sha":"cccccccccccccccccccccccccccccccccccccccc","updated_at":"synthetic-test-input"}}}"#,
        )
        .expect("write duplicate-key source facts");
    let mut args = fixture.base_args("source-facts-duplicate-keys");
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        duplicate_keys_path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        sha256_file(&duplicate_keys_path),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "source-facts JSON contains a duplicate key: head_sha",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-duplicate-keys");

    let schema_v1_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/source-facts-schema-v1.json"),
            &serde_json::to_vec(&json!({
                "schema_version": 1,
                "repository": "tailrocks/terminal-components-claude",
                "observed_at": "synthetic-test-input",
                "latest_source_observation": {
                    "observed_at": "synthetic-test-input",
                    "method": "synthetic Rust black-box fixture with otherwise valid v1 facts",
                    "reference_remote": {
                        "branch": "visual-baseline",
                        "head_sha": fixture.reference_sha.as_str(),
                        "updated_at": "synthetic-test-input"
                    },
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": fixture.candidate_sha.as_str(),
                        "updated_at": "synthetic-test-input"
                    }
                }
            }))
            .expect("serialize otherwise-valid schema-v1 source facts"),
        )
        .expect("write otherwise-valid schema-v1 source facts");
    let schema_v1_run_id = "source-facts-schema-v1";
    let mut args = fixture.base_args(schema_v1_run_id);
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        schema_v1_path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        sha256_file(&schema_v1_path),
    );
    let output = fixture.run(&args, &[]);
    assert_eq!(output.signal, None);
    assert_eq!(output.exit_code, Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"subjects: source-facts schema_version must be 2\n"
    );
    assert_no_archive_metadata_or_build(&fixture, schema_v1_run_id);

    let legacy_only_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/source-facts-legacy-only.json"),
            &serde_json::to_vec(&json!({
                "schema_version": 2,
                "repository": "tailrocks/terminal-components-claude",
                "observed_at": "synthetic-test-input",
                "reference": {
                    "branch": "visual-baseline",
                    "head_sha": fixture.reference_sha.as_str(),
                },
                "candidate": {
                    "branch": "termrock-implementation",
                    "head_sha": fixture.candidate_sha.as_str(),
                }
            }))
            .expect("serialize legacy-only source facts"),
        )
        .expect("write legacy-only source facts");
    let mut args = fixture.base_args("source-facts-legacy-only");
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        legacy_only_path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        sha256_file(&legacy_only_path),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "source-facts latest_source_observation is required",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-legacy-only");

    let wrong_branch_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/source-facts-wrong-branch.json"),
            &serde_json::to_vec(&json!({
                "schema_version": 2,
                "repository": "tailrocks/terminal-components-claude",
                "observed_at": "synthetic-test-input",
                "latest_source_observation": {
                    "observed_at": "synthetic-test-input",
                    "method": "synthetic Rust black-box fixture",
                    "reference_remote": {
                        "branch": "stale-visual-baseline",
                        "head_sha": fixture.reference_sha.as_str(),
                        "updated_at": "synthetic-test-input"
                    },
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": fixture.candidate_sha.as_str(),
                        "updated_at": "synthetic-test-input"
                    }
                }
            }))
            .expect("serialize wrong-branch source facts"),
        )
        .expect("write wrong-branch source facts");
    let mut args = fixture.base_args("source-facts-wrong-branch");
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        wrong_branch_path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        sha256_file(&wrong_branch_path),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "source-facts reference branch must be visual-baseline",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-wrong-branch");

    let malformed_pin_path = fixture
        .repo
        .write_file(
            Path::new("fixtures/source-facts-malformed-pin.json"),
            &serde_json::to_vec(&json!({
                "schema_version": 2,
                "repository": "tailrocks/terminal-components-claude",
                "observed_at": "synthetic-test-input",
                "latest_source_observation": {
                    "observed_at": "synthetic-test-input",
                    "method": "synthetic Rust black-box fixture",
                    "reference_remote": {
                        "branch": "visual-baseline",
                        "head_sha": 7,
                        "updated_at": "synthetic-test-input"
                    },
                    "candidate_remote": {
                        "branch": "termrock-implementation",
                        "head_sha": fixture.candidate_sha.as_str(),
                        "updated_at": "synthetic-test-input"
                    }
                }
            }))
            .expect("serialize malformed source facts"),
        )
        .expect("write malformed source facts");
    let mut args = fixture.base_args("source-facts-malformed-pin");
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-path",
        malformed_pin_path.display().to_string(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--source-facts-sha256",
        sha256_file(&malformed_pin_path),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "source-facts reference head_sha must be a lowercase full commit SHA",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-malformed-pin");

    let mut args = fixture.base_args("source-facts-reference-pin-mismatch");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.candidate_sha.clone(),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "reference_commit does not match the latest accepted source-facts reference tip",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-reference-pin-mismatch");

    let mut args = fixture.base_args("source-facts-candidate-pin-mismatch");
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        fixture.reference_sha.clone(),
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "candidate_commit does not match the latest accepted source-facts candidate tip",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-candidate-pin-mismatch");

    let mut args = fixture.base_args("source-facts-local-reference-mismatch");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.candidate_sha.clone(),
    );
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        fixture.candidate_sha.clone(),
    );
    fixture.set_source_facts(
        &mut args,
        "local-reference-mismatch",
        &fixture.candidate_sha,
        &fixture.candidate_sha,
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "fetched visual-baseline ref does not match the accepted source-facts reference tip",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-local-reference-mismatch");

    let mut args = fixture.base_args("source-facts-local-candidate-mismatch");
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        fixture.reference_sha.clone(),
    );
    fixture.set_source_facts(
        &mut args,
        "local-candidate-mismatch",
        &fixture.reference_sha,
        &fixture.reference_sha,
    );
    let output = fixture.run(&args, &[]);
    assert_failed(
        &output,
        "fetched termrock-implementation ref does not match the accepted source-facts candidate tip",
    );
    assert_no_archive_metadata_or_build(&fixture, "source-facts-local-candidate-mismatch");

    #[cfg(unix)]
    {
        let ancestor = git_commit_parent(&fixture.repository, &fixture.reference_sha);
        let mut args = fixture.base_args("source-facts-reference-ancestor-not-tip");
        SubjectsFixture::set_arg(&mut args, "--reference-commit", ancestor.clone());
        fixture.set_source_facts(
            &mut args,
            "reference-ancestor-not-tip",
            &ancestor,
            &fixture.candidate_sha,
        );
        let output = fixture.run(&args, &[]);
        assert_failed(
            &output,
            "fetched visual-baseline ref does not match the accepted source-facts reference tip",
        );
        assert_no_archive_metadata_or_build(&fixture, "source-facts-reference-ancestor-not-tip");
    }

    #[cfg(unix)]
    {
        let link = fixture
            .repo
            .root()
            .join("fixtures/source-facts-symlink.json");
        symlink(&fixture.source_facts_path, &link).expect("create source-facts symlink probe");
        let mut args = fixture.base_args("source-facts-symlink");
        SubjectsFixture::set_arg(&mut args, "--source-facts-path", link.display().to_string());
        let output = fixture.run(&args, &[]);
        assert_failed(
            &output,
            "cannot open source-facts file without following symlinks",
        );
        assert_no_archive_metadata_or_build(&fixture, "source-facts-symlink");
    }
}

#[test]
fn records_exact_cargo_metadata_failure_output_and_failed_run_receipt() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let run_id = "cargo-metadata-nonzero";
    let raw_stdout = b"\x00synthetic metadata failure stdout\n\xff";
    let raw_stderr = b"synthetic metadata failure stderr\nsecond line\n";
    let stdout_path = fixture
        .repo
        .write_file(Path::new("fixtures/metadata-failure.stdout"), raw_stdout)
        .expect("write raw Cargo metadata stdout");
    let stderr_path = fixture
        .repo
        .write_file(Path::new("fixtures/metadata-failure.stderr"), raw_stderr)
        .expect("write raw Cargo metadata stderr");
    let args = fixture.base_args(run_id);
    let output = fixture.run(
        &args,
        &[
            (
                "VISIBILITY_FIXTURE_CARGO_METADATA_RAW",
                stdout_path.display().to_string(),
            ),
            (
                "VISIBILITY_FIXTURE_CARGO_METADATA_STDERR",
                stderr_path.display().to_string(),
            ),
            ("VISIBILITY_FIXTURE_CARGO_METADATA_EXIT", "17".to_owned()),
        ],
    );
    assert_failed(&output, "cargo metadata failed for reference source");
    assert!(!fixture.manifest_path(run_id).exists());

    let run_root = fixture.repo.root().join("evidence").join(run_id);
    let metadata_dir = run_root.join("metadata/reference");
    assert_eq!(
        fs::read(metadata_dir.join("metadata.stdout")).expect("read raw metadata stdout"),
        raw_stdout
    );
    assert_eq!(
        fs::read(metadata_dir.join("metadata.stderr")).expect("read raw metadata stderr"),
        raw_stderr
    );
    assert_eq!(
        fs::read_to_string(metadata_dir.join("metadata.exit")).expect("read metadata exit"),
        "17\n"
    );
    assert!(!run_root.join("metadata/candidate").exists());
    assert!(!run_root.join("build").exists());

    let run_record: Value = serde_json::from_slice(
        &fs::read(fixture.run_record_path(run_id)).expect("read failed metadata run receipt"),
    )
    .expect("parse failed metadata run receipt");
    assert_eq!(run_record["state"], "failed");
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert!(
        run_record["error"]
            .as_str()
            .unwrap()
            .contains("cargo metadata failed for reference source")
    );
    assert!(
        !run_record
            .as_object()
            .unwrap()
            .contains_key("subject_manifest")
    );
    let source_evidence = PathBuf::from(
        run_record["source_input_evidence"]["path"]
            .as_str()
            .expect("source evidence path in failed receipt"),
    );
    let source_bytes = fs::read(&source_evidence).expect("read failure source sidecar");
    assert_eq!(
        run_record["source_input_evidence"]["sha256"],
        sha256_bytes(&source_bytes)
    );
    fixture.cleanup_sources_for_run(run_id);
}

#[test]
fn records_exact_cargo_build_failure_output_and_failed_run_receipt() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let run_id = "cargo-build-nonzero";
    let raw_stderr = b"synthetic release build failure stderr\nlinker stopped\n";
    let stderr_path = fixture
        .repo
        .write_file(Path::new("fixtures/build-failure.stderr"), raw_stderr)
        .expect("write raw Cargo build stderr");
    let args = fixture.base_args(run_id);
    let output = fixture.run(
        &args,
        &[
            ("VISIBILITY_FIXTURE_CARGO_BUILD_EXIT", "23".to_owned()),
            (
                "VISIBILITY_FIXTURE_CARGO_BUILD_STDERR",
                stderr_path.display().to_string(),
            ),
        ],
    );
    assert_failed(
        &output,
        "release build failed for reference tablepro source",
    );
    assert!(!fixture.manifest_path(run_id).exists());

    let run_root = fixture.repo.root().join("evidence").join(run_id);
    let build_dir = run_root.join("build/reference/tablepro");
    assert_eq!(
        fs::read(build_dir.join("build.stdout")).expect("read raw build stdout"),
        b""
    );
    assert_eq!(
        fs::read(build_dir.join("build.stderr")).expect("read raw build stderr"),
        raw_stderr
    );
    assert_eq!(
        fs::read_to_string(build_dir.join("build.exit")).expect("read build exit"),
        "23\n"
    );
    assert!(
        !run_root
            .join("build/candidate/tablepro/build.argv.json")
            .exists()
    );

    let run_record: Value = serde_json::from_slice(
        &fs::read(fixture.run_record_path(run_id)).expect("read failed build run receipt"),
    )
    .expect("parse failed build run receipt");
    assert_eq!(run_record["state"], "failed");
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert!(
        run_record["error"]
            .as_str()
            .unwrap()
            .contains("release build failed for reference tablepro source")
    );
    assert!(
        !run_record
            .as_object()
            .unwrap()
            .contains_key("subject_manifest")
    );
    let source_evidence = PathBuf::from(
        run_record["source_input_evidence"]["path"]
            .as_str()
            .expect("source evidence path in failed receipt"),
    );
    let source_bytes = fs::read(&source_evidence).expect("read failed build source sidecar");
    assert_eq!(
        run_record["source_input_evidence"]["sha256"],
        sha256_bytes(&source_bytes)
    );
    fixture.cleanup_sources_for_run(run_id);
}

#[test]
fn rejects_partial_expected_generation_tuple_before_source_work() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    for (index, option) in [
        "--expected-generation-id",
        "--expected-generation-sha256",
        "--expected-generation-root",
    ]
    .into_iter()
    .enumerate()
    {
        let run_id = format!("partial-generation-{index}");
        let mut args = fixture.base_args(&run_id);
        SubjectsFixture::remove_arg(&mut args, option);
        let output = fixture.run(&args, &[]);
        assert_failed(
            &output,
            "expected generation id, sha256, and root must be supplied together",
        );
        assert_no_archive_metadata_or_build(&fixture, &run_id);
    }
}

#[test]
fn rejects_invalid_cli_toolchain_and_ambient_config_before_building() {
    let _fixture_guard = serialize_subjects_fixture();
    let fixture = SubjectsFixture::new();
    let mut count = 0usize;
    let mut run_case =
        |name: &str, mut args: Vec<String>, overrides: Vec<(&str, String)>, expected: &str| {
            count += 1;
            let run_id_index = args
                .iter()
                .position(|arg| arg == "--run-id")
                .expect("run-id option");
            let run_id = if args[run_id_index + 1] == "placeholder" {
                let generated = format!("negative-{count:02}-{name}");
                args[run_id_index + 1] = generated.clone();
                generated
            } else {
                args[run_id_index + 1].clone()
            };
            let output = fixture.run(&args, &overrides);
            assert_failed(&output, expected);
            assert!(
                !fixture.manifest_path(&run_id).exists(),
                "a rejected input must not produce a subject manifest"
            );
        };

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--run-id", "unsafe/path".to_owned());
    run_case("run-id-separator", args, vec![], "safe path segment");

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--run-id", "..".to_owned());
    run_case("run-id-parent", args, vec![], "safe path segment");

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--run-id", "x".repeat(129));
    run_case("run-id-too-long", args, vec![], "safe path segment");

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        fixture.candidate_sha[..39].to_owned(),
    );
    run_case(
        "candidate-short-sha",
        args,
        vec![],
        "lowercase full commit SHA",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        fixture.candidate_sha.to_uppercase(),
    );
    run_case(
        "candidate-uppercase-sha",
        args,
        vec![],
        "lowercase full commit SHA",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--candidate-commit",
        "ffffffffffffffffffffffffffffffffffffffff".to_owned(),
    );
    fixture.set_source_facts(
        &mut args,
        "candidate-unresolved",
        &fixture.reference_sha,
        "ffffffffffffffffffffffffffffffffffffffff",
    );
    run_case(
        "candidate-unresolved-sha",
        args,
        vec![],
        "cannot resolve pinned candidate commit",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.reference_sha[..39].to_owned(),
    );
    run_case(
        "reference-short-sha",
        args,
        vec![],
        "reference_commit must be a lowercase full commit SHA",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.reference_sha.to_uppercase(),
    );
    run_case(
        "reference-uppercase-sha",
        args,
        vec![],
        "reference_commit must be a lowercase full commit SHA",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        "ffffffffffffffffffffffffffffffffffffffff".to_owned(),
    );
    fixture.set_source_facts(
        &mut args,
        "reference-unresolved",
        "ffffffffffffffffffffffffffffffffffffffff",
        &fixture.candidate_sha,
    );
    run_case(
        "reference-unresolved-sha",
        args,
        vec![],
        "cannot resolve pinned requested reference commit",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--reference-commit",
        fixture.candidate_sha.clone(),
    );
    run_case(
        "reference-not-on-visual-baseline",
        args,
        vec![],
        "reference_commit does not match the latest accepted source-facts reference tip",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--suite-sha256", "a".repeat(63));
    run_case(
        "suite-short-sha",
        args,
        vec![],
        "suite_sha256 must be a lowercase SHA-256",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--suite-sha256", SUITE_SHA256.to_uppercase());
    run_case(
        "suite-uppercase-sha",
        args,
        vec![],
        "suite_sha256 must be a lowercase SHA-256",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--expected-generation-sha256", "b".repeat(65));
    run_case(
        "generation-long-sha",
        args,
        vec![],
        "expected_generation_sha256 must be a lowercase SHA-256",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--expected-generation-sha256",
        GENERATION_SHA256.to_uppercase(),
    );
    run_case(
        "generation-uppercase-sha",
        args,
        vec![],
        "expected_generation_sha256 must be a lowercase SHA-256",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--suite-revision", "  ".to_owned());
    run_case(
        "blank-suite-revision",
        args,
        vec![],
        "suite_revision is required",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--expected-generation-id", " ".to_owned());
    run_case(
        "blank-generation-id",
        args,
        vec![],
        "expected_generation_id is required when expected generation is supplied",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--target-triple", "  ".to_owned());
    run_case(
        "blank-target-triple",
        args,
        vec![],
        "target_triple is required",
    );

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--app", "unknown-app".to_owned());
    run_case("unknown-app", args, vec![], "invalid choice");

    let mut args = fixture.base_args("placeholder");
    let reference = fixture.reference_output_root.clone();
    SubjectsFixture::set_output_root(&mut args, "--candidate-output-root", &reference);
    run_case("same-output-root", args, vec![], "output roots must differ");

    let mut args = fixture.base_args("placeholder");
    let candidate = fixture.candidate_output_root.clone();
    SubjectsFixture::set_output_root(
        &mut args,
        "--reference-output-root",
        &candidate.join("nested"),
    );
    run_case(
        "reference-under-candidate",
        args,
        vec![],
        "output roots must be disjoint",
    );

    let mut args = fixture.base_args("placeholder");
    let reference = fixture.reference_output_root.clone();
    SubjectsFixture::set_output_root(
        &mut args,
        "--candidate-output-root",
        &reference.join("nested"),
    );
    run_case(
        "candidate-under-reference",
        args,
        vec![],
        "output roots must be disjoint",
    );

    let mut args = fixture.base_args("placeholder");
    let expected = fixture.expected_root.clone();
    SubjectsFixture::set_output_root(
        &mut args,
        "--reference-output-root",
        &expected.join("reference"),
    );
    run_case(
        "reference-overlaps-generation",
        args,
        vec![],
        "reference output root overlaps expected generation",
    );

    let mut args = fixture.base_args("placeholder");
    let expected = fixture.expected_root.clone();
    SubjectsFixture::set_output_root(
        &mut args,
        "--candidate-output-root",
        &expected.join("candidate"),
    );
    run_case(
        "candidate-overlaps-generation",
        args,
        vec![],
        "candidate output root overlaps expected generation",
    );

    let schema_path = fixture
        .repo
        .root()
        .join("crates/termrock-e2e/schemas/subject-manifest-v2.schema.json");
    let original_schema = fs::read(&schema_path).expect("read copied schema");
    fs::write(&schema_path, b"{}\n").expect("tamper copied schema");
    let args = fixture.base_args("placeholder");
    run_case(
        "schema-changed",
        args,
        vec![],
        "subject schema digest changed",
    );
    fs::write(&schema_path, original_schema).expect("restore exact schema");

    let stale_run = "negative-21-existing-evidence";
    fs::create_dir_all(fixture.repo.root().join("evidence").join(stale_run))
        .expect("create stale run directory");
    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(&mut args, "--run-id", stale_run.to_owned());
    run_case("existing-run", args, vec![], "refusing stale evidence");

    let mut args = fixture.base_args("placeholder");
    SubjectsFixture::set_arg(
        &mut args,
        "--target-triple",
        "unsupported-target".to_owned(),
    );
    run_case(
        "target-not-host",
        args,
        vec![],
        "differs from pinned toolchain host",
    );

    let mut args = fixture.base_args("placeholder");
    let mut missing_repo = fixture.repo.root().join("missing-product-repository");
    missing_repo.push("repo");
    SubjectsFixture::set_arg(
        &mut args,
        "--repository",
        missing_repo.display().to_string(),
    );
    run_case(
        "repository-missing",
        args,
        vec![],
        "git repository path is not a directory",
    );

    let mut args = fixture.base_args("placeholder");
    args.retain(|arg| arg != "--candidate-commit");
    let candidate_sha = fixture.candidate_sha.clone();
    let candidate_index = args.iter().position(|arg| arg == &candidate_sha).unwrap();
    args.remove(candidate_index);
    run_case(
        "candidate-argument-missing",
        args,
        vec![],
        "--candidate-commit",
    );

    let mut args = fixture.base_args("placeholder");
    args.retain(|arg| arg != "--reference-commit");
    let reference_sha = fixture.reference_sha.clone();
    let reference_index = args.iter().position(|arg| arg == &reference_sha).unwrap();
    args.remove(reference_index);
    run_case(
        "reference-argument-missing",
        args,
        vec![],
        "--reference-commit",
    );

    let args = fixture.base_args("placeholder");
    run_case(
        "cargo-version-failure",
        args,
        vec![("VISIBILITY_FIXTURE_CARGO_VERSION_EXIT", "7".to_owned())],
        "cannot query pinned Cargo version",
    );

    let args = fixture.base_args("placeholder");
    run_case(
        "rustc-version-failure",
        args,
        vec![("VISIBILITY_FIXTURE_RUSTC_VERSION_EXIT", "7".to_owned())],
        "cannot query pinned rustc version",
    );

    let args = fixture.base_args("placeholder");
    run_case(
        "rustc-verbose-failure",
        args,
        vec![("VISIBILITY_FIXTURE_RUSTC_VERBOSE_EXIT", "7".to_owned())],
        "cannot query pinned rustc verbose version",
    );

    let missing_rustup_home = fixture.repo.root().join("missing-rustup-home");
    let args = fixture.base_args("placeholder");
    run_case(
        "rustup-home-missing",
        args,
        vec![(
            "VISIBILITY_FIXTURE_RUSTUP_HOME",
            missing_rustup_home.display().to_string(),
        )],
        "resolved RUSTUP_HOME does not exist",
    );

    fixture
        .repo
        .write_file(
            Path::new(".cargo/config.toml"),
            b"[build]\nrustflags = ['--cfg', 'ambient_config_must_be_rejected']\n",
        )
        .expect("write ambient Cargo config above the snapshot root");
    let args = fixture.base_args("placeholder");
    run_case(
        "ancestor-cargo-config",
        args,
        vec![],
        "Cargo config outside pinned source snapshot can affect the build",
    );
    let run_root = fixture
        .repo
        .root()
        .join("evidence/negative-29-ancestor-cargo-config");
    assert!(
        !run_root
            .join("snapshot-logs/reference/archive.exit")
            .exists()
    );
    assert!(
        !run_root
            .join("snapshot-logs/candidate/archive.exit")
            .exists()
    );

    assert_eq!(count, 34, "keep each rejection case explicit and countable");
}

#[cfg(unix)]
fn real_build_required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("missing caller-pinned {name} input"))
}

#[cfg(unix)]
fn real_build_write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    output.write_all(bytes)?;
    output.sync_all()
}

#[cfg(unix)]
fn real_build_write_json_new(path: &Path, value: &Value) -> std::io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec_pretty(value)
        .expect("serialize external real-build supervisor evidence");
    real_build_write_new(path, &bytes)?;
    Ok(bytes)
}

#[cfg(unix)]
fn real_build_resolve_executable(name: &str, path: &std::ffi::OsStr) -> (PathBuf, String) {
    for directory in env::split_paths(path) {
        let candidate = directory.join(name);
        let Ok(metadata) = fs::symlink_metadata(&candidate) else {
            continue;
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            continue;
        }
        let resolved = candidate
            .canonicalize()
            .unwrap_or_else(|error| panic!("resolve {name} on controlled PATH: {error}"));
        return (resolved.clone(), sha256_file(&resolved));
    }
    panic!("{name} is not executable on the controlled PATH");
}

#[cfg(unix)]
fn real_build_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Runs one caller-authorized production Holla pair build, without launching either product.
///
/// The invocation driver must provide a fresh, absent Nextest target root and verify that
/// absence before calling Nextest. It also supplies a unique per-run disposable CARGO_HOME
/// populated for locked offline builds. This test confirms the paths and rejects symlinked
/// registry/git roots, but does not copy or clean the caller's cache.
#[cfg(unix)]
#[test]
#[ignore = "real Holla pair build; requires accepted source facts and an assigned build slot"]
fn supervises_real_holla_pair_build() {
    const REFERENCE_COMMIT: &str = "b682cb26d68b353aeeccf9e51653eddf097b39f5";
    const CANDIDATE_COMMIT: &str = "1ea1c17707f0a8f1639af506be179013d5e2d52a";
    const SUBJECTS_TOOL_SHA256: &str =
        "cdcd4b3ad560a3aaa2dda1bbc19e06f9b3b52838609dad4771669e3739f9d32d";
    const CAPTURE_LIMIT: usize = 8 * 1024 * 1024;

    let run_id = real_build_required_env("TERMROCK_VIS06_REAL_RUN_ID");
    assert!(
        !run_id.is_empty()
            && run_id.len() <= 128
            && run_id.as_bytes()[0].is_ascii_alphanumeric()
            && run_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)),
        "caller run id must use the production run-id character set"
    );

    let run_root_input = PathBuf::from(real_build_required_env("TERMROCK_VIS06_REAL_RUN_ROOT"));
    assert!(run_root_input.is_absolute(), "run root must be absolute");
    let expected_run_root_name = format!("termrock-vis06-real-holla-{run_id}");
    assert_eq!(
        run_root_input.file_name().and_then(|name| name.to_str()),
        Some(expected_run_root_name.as_str()),
        "run root must be unique to this caller-pinned run id"
    );
    let run_root_parent = run_root_input
        .parent()
        .expect("run root has a parent")
        .canonicalize()
        .expect("resolve existing external run-root parent");
    let run_root = run_root_parent.join(
        run_root_input
            .file_name()
            .expect("run root has a final component"),
    );
    assert!(
        !run_root.exists(),
        "refuse a pre-existing real-build run root"
    );

    let evidence_root = run_root.join("evidence");
    let reference_output_root = run_root.join("outputs/reference");
    let candidate_output_root = run_root.join("outputs/candidate");
    for path in [
        &evidence_root,
        &reference_output_root,
        &candidate_output_root,
    ] {
        assert!(
            !path.exists(),
            "refuse stale real-build output path {}",
            path.display()
        );
    }

    let manifest_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository = manifest_directory
        .join("../..")
        .canonicalize()
        .expect("resolve real product repository");
    let subjects_script = repository.join("tools/visibility/subjects.py");
    assert!(subjects_script.is_file());
    let subjects_script_sha256 = sha256_file(&subjects_script);
    assert_eq!(
        subjects_script_sha256, SUBJECTS_TOOL_SHA256,
        "real build must use the frozen reviewed launcher"
    );

    let facts_path = PathBuf::from(real_build_required_env(
        "TERMROCK_VIS06_REAL_SOURCE_FACTS_PATH",
    ));
    assert!(
        facts_path.is_absolute(),
        "accepted source-facts path must be absolute"
    );
    let facts_metadata = fs::symlink_metadata(&facts_path)
        .expect("inspect caller-pinned source-facts file without following it");
    assert!(
        facts_metadata.is_file() && !facts_metadata.file_type().is_symlink(),
        "accepted source facts must be a regular non-symlink file"
    );
    assert_eq!(
        facts_metadata.permissions().mode() & 0o222,
        0,
        "accepted source facts must be read-only"
    );
    let source_facts_sha256 = real_build_required_env("TERMROCK_VIS06_REAL_SOURCE_FACTS_SHA256");
    assert_eq!(
        source_facts_sha256.len(),
        64,
        "source-facts SHA-256 must be full length"
    );
    assert!(
        source_facts_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "source-facts SHA-256 must be lowercase hexadecimal"
    );
    let source_facts_bytes = fs::read(&facts_path).expect("read accepted source-facts bytes");
    assert_eq!(sha256_bytes(&source_facts_bytes), source_facts_sha256);
    let source_facts: Value =
        serde_json::from_slice(&source_facts_bytes).expect("parse accepted source-facts v2");
    assert_eq!(source_facts["schema_version"], 2);
    assert_eq!(
        source_facts["latest_source_observation"]["reference_remote"]["branch"],
        "visual-baseline"
    );
    assert_eq!(
        source_facts["latest_source_observation"]["reference_remote"]["head_sha"],
        REFERENCE_COMMIT
    );
    assert_eq!(
        source_facts["latest_source_observation"]["candidate_remote"]["branch"],
        "termrock-implementation"
    );
    assert_eq!(
        source_facts["latest_source_observation"]["candidate_remote"]["head_sha"],
        CANDIDATE_COMMIT
    );

    let suite_revision = real_build_required_env("TERMROCK_VIS06_REAL_SUITE_REVISION");
    let suite_sha256 = real_build_required_env("TERMROCK_VIS06_REAL_SUITE_SHA256");
    assert!(!suite_revision.trim().is_empty());
    assert_eq!(
        suite_sha256.len(),
        64,
        "accepted suite digest must be SHA-256"
    );
    assert!(
        suite_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "accepted suite digest must be lowercase hexadecimal"
    );
    let target_triple = real_build_required_env("TERMROCK_VIS06_REAL_TARGET_TRIPLE");
    assert_eq!(
        target_triple, "aarch64-apple-darwin",
        "the fixed build target is native Apple ARM64"
    );

    let path_value = env::var_os("PATH").expect("controlled PATH is required");
    let path_text = path_value
        .to_str()
        .expect("controlled PATH must be valid UTF-8");
    assert!(
        !path_text
            .to_ascii_lowercase()
            .replace('\\', "/")
            .contains("/application support/mbx/bin")
            && !path_text
                .to_ascii_lowercase()
                .replace('\\', "/")
                .contains("/command-wrappers/bin"),
        "controlled PATH must exclude wrapper and MBX command directories"
    );
    let tool_paths = ["python3", "git", "rustup"]
        .into_iter()
        .map(|name| {
            let (path, sha256) = real_build_resolve_executable(name, &path_value);
            (
                name.to_owned(),
                json!({"path": path.display().to_string(), "sha256": sha256}),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    let sensitive_fragments = [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "CREDENTIAL",
        "PROXY",
        "ASKPASS",
    ];
    for (name, _) in env::vars_os() {
        let name = name.to_string_lossy().to_ascii_uppercase();
        assert!(
            !sensitive_fragments
                .iter()
                .any(|fragment| name.contains(fragment)),
            "run under a cleared environment; sensitive variable name {name} must be absent"
        );
    }

    let home = PathBuf::from(real_build_required_env("HOME"))
        .canonicalize()
        .expect("resolve clean outer HOME");
    assert!(home.is_dir());
    for git_config in [home.join(".gitconfig"), home.join(".config/git/config")] {
        assert!(
            !git_config.exists() && !git_config.is_symlink(),
            "clean outer HOME must not contain global Git config"
        );
    }

    let cargo_home_input = PathBuf::from(real_build_required_env("CARGO_HOME"));
    assert!(
        cargo_home_input.is_absolute(),
        "disposable Cargo home must be absolute"
    );
    let expected_cargo_home_name = format!("termrock-vis06-cargo-home-{run_id}");
    assert_eq!(
        cargo_home_input.file_name().and_then(|name| name.to_str()),
        Some(expected_cargo_home_name.as_str()),
        "Cargo cache path must be unique and disposable for this run id"
    );
    let cargo_home_metadata = fs::symlink_metadata(&cargo_home_input)
        .expect("inspect caller-pinned disposable Cargo home");
    assert!(
        cargo_home_metadata.is_dir() && !cargo_home_metadata.file_type().is_symlink(),
        "disposable Cargo home must be a real directory"
    );
    let cargo_home = cargo_home_input
        .canonicalize()
        .expect("canonicalize disposable Cargo home");
    let default_cargo_home = home.join(".cargo");
    assert_ne!(cargo_home, default_cargo_home);
    assert!(
        !cargo_home.starts_with(&default_cargo_home)
            && !default_cargo_home.starts_with(&cargo_home),
        "disposable Cargo cache must not overlap the default shared cache"
    );
    for cache_name in ["registry", "git"] {
        let cache_root = cargo_home.join(cache_name);
        let metadata = fs::symlink_metadata(&cache_root)
            .unwrap_or_else(|_| panic!("disposable offline cache is missing {cache_name}"));
        assert!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "disposable Cargo {cache_name} cache must be a real directory, not a shared alias"
        );
        assert!(cache_root.canonicalize().unwrap().starts_with(&cargo_home));
    }
    for config_name in ["config", "config.toml", "credentials", "credentials.toml"] {
        let config_path = cargo_home.join(config_name);
        assert!(
            !config_path.exists() && !config_path.is_symlink(),
            "disposable Cargo home must not contain {config_name}"
        );
    }

    let rustup_home = PathBuf::from(real_build_required_env("RUSTUP_HOME"))
        .canonicalize()
        .expect("resolve pinned Rustup home");
    assert!(rustup_home.is_dir());
    let tmpdir = PathBuf::from(real_build_required_env("TMPDIR"))
        .canonicalize()
        .expect("resolve unique outer TMPDIR");
    assert!(tmpdir.is_dir());
    let expected_tmpdir_name = format!("termrock-vis06-tmp-{run_id}");
    assert_eq!(
        tmpdir.file_name().and_then(|name| name.to_str()),
        Some(expected_tmpdir_name.as_str()),
        "outer TMPDIR must be unique to this run id"
    );
    let nextest_target_root_input = PathBuf::from(real_build_required_env(
        "TERMROCK_VIS06_REAL_NEXTTEST_TARGET_ROOT",
    ));
    assert!(nextest_target_root_input.is_absolute());
    let expected_nextest_target_name = format!("termrock-vis06-nextest-target-{run_id}");
    assert_eq!(
        nextest_target_root_input
            .file_name()
            .and_then(|name| name.to_str()),
        Some(expected_nextest_target_name.as_str()),
        "Nextest target root must be fresh and unique to this run id"
    );
    let nextest_target_metadata = fs::symlink_metadata(&nextest_target_root_input)
        .expect("inspect already-created Nextest target root");
    assert!(
        nextest_target_metadata.is_dir() && !nextest_target_metadata.file_type().is_symlink(),
        "Nextest target root must be a real directory"
    );
    let nextest_target_root = nextest_target_root_input
        .canonicalize()
        .expect("resolve the already-created outer Nextest target root");
    let test_executable = env::current_exe()
        .expect("resolve current Nextest test executable")
        .canonicalize()
        .expect("canonicalize current Nextest test executable");
    assert!(
        test_executable.starts_with(&nextest_target_root),
        "current test executable must be under the caller-selected Nextest target root"
    );
    for (name, path) in [
        ("run root", &run_root),
        ("Cargo home", &cargo_home),
        ("Nextest target", &nextest_target_root),
    ] {
        assert!(
            !path.starts_with(&repository) && !repository.starts_with(path),
            "{name} must be external to the product repository"
        );
    }
    assert!(
        !run_root.starts_with(&cargo_home)
            && !cargo_home.starts_with(&run_root)
            && !run_root.starts_with(&nextest_target_root)
            && !nextest_target_root.starts_with(&run_root)
            && !cargo_home.starts_with(&nextest_target_root)
            && !nextest_target_root.starts_with(&cargo_home),
        "evidence, disposable cache, and Nextest target roots must be disjoint"
    );

    let reference_output_text = reference_output_root.display().to_string();
    let candidate_output_text = candidate_output_root.display().to_string();
    let evidence_root_text = evidence_root.display().to_string();
    let repository_text = repository.display().to_string();
    let facts_path_text = facts_path.display().to_string();
    let args = vec![
        "--repository".to_owned(),
        repository_text.clone(),
        "--reference-commit".to_owned(),
        REFERENCE_COMMIT.to_owned(),
        "--candidate-commit".to_owned(),
        CANDIDATE_COMMIT.to_owned(),
        "--source-facts-path".to_owned(),
        facts_path_text.clone(),
        "--source-facts-sha256".to_owned(),
        source_facts_sha256.clone(),
        "--evidence-root".to_owned(),
        evidence_root_text.clone(),
        "--run-id".to_owned(),
        run_id.clone(),
        "--app".to_owned(),
        "holla".to_owned(),
        "--suite-revision".to_owned(),
        suite_revision.clone(),
        "--suite-sha256".to_owned(),
        suite_sha256.clone(),
        "--target-triple".to_owned(),
        target_triple.clone(),
        "--reference-output-root".to_owned(),
        reference_output_text.clone(),
        "--candidate-output-root".to_owned(),
        candidate_output_text.clone(),
    ];
    let mut actual_argv = vec!["python3".to_owned(), subjects_script.display().to_string()];
    actual_argv.extend(args.iter().cloned());
    let argv_bytes = serde_json::to_vec(&actual_argv).expect("serialize exact CLI argv");
    let argv_sha256 = sha256_bytes(&argv_bytes);
    let started_unix_ms = real_build_unix_millis();

    fs::create_dir(&run_root).expect("create fresh external real-build run root");
    fs::set_permissions(&run_root, fs::Permissions::from_mode(0o700))
        .expect("restrict external real-build run root");
    let supervisor_root = run_root.join("supervisor");
    fs::create_dir(&supervisor_root).expect("create external supervisor evidence directory");
    fs::set_permissions(&supervisor_root, fs::Permissions::from_mode(0o700))
        .expect("restrict supervisor evidence directory");
    let actual_argv_json = json!({
        "schema": "termrock-visibility-real-cli-argv-v1",
        "run_id": run_id.as_str(),
        "argv": actual_argv,
        "argv_sha256": argv_sha256,
        "cwd": repository_text,
        "script_sha256": subjects_script_sha256.as_str(),
        "source_facts_path": facts_path_text,
        "source_facts_sha256": source_facts_sha256.as_str(),
        "reference_commit": REFERENCE_COMMIT,
        "candidate_commit": CANDIDATE_COMMIT,
        "suite_revision": suite_revision.as_str(),
        "suite_sha256": suite_sha256.as_str(),
        "target_triple": target_triple.as_str(),
        "evidence_root": evidence_root_text.as_str(),
        "reference_output_root": reference_output_text.as_str(),
        "candidate_output_root": candidate_output_text.as_str(),
        "nextest_target_root": nextest_target_root.display().to_string(),
        "cargo_home": cargo_home.display().to_string(),
        "timeout_seconds": 1200,
        "outer_capture_limit_bytes_per_stream": CAPTURE_LIMIT,
        "tool_binaries": tool_paths,
        "environment_names": [
            "PATH",
            "HOME",
            "CARGO_HOME",
            "RUSTUP_HOME",
            "TMPDIR",
            "DEVELOPER_DIR",
            "SDKROOT",
            "TERMROCK_VIS06_REAL_* caller pins",
        ],
        "nested_cargo_log_limitation": "Only the Python CLI outer streams are captured here; subjects.py buffers nested Cargo stdout/stderr until each command returns.",
        "environment_values_serialized": false,
        "qualification": "blocked",
    });
    let argv_json_bytes =
        real_build_write_json_new(&supervisor_root.join("argv.json"), &actual_argv_json)
            .expect("persist structured argv evidence before CLI spawn");
    let raw_argv_file_sha256 = sha256_bytes(&argv_json_bytes);

    let mut child_environment = vec![
        ("PATH", path_text),
        ("HOME", home.to_str().expect("HOME path is UTF-8")),
        (
            "CARGO_HOME",
            cargo_home.to_str().expect("Cargo home path is UTF-8"),
        ),
        (
            "RUSTUP_HOME",
            rustup_home.to_str().expect("Rustup home path is UTF-8"),
        ),
        ("TMPDIR", tmpdir.to_str().expect("TMPDIR path is UTF-8")),
        ("CARGO_NET_OFFLINE", "true"),
        ("CARGO_TERM_COLOR", "never"),
    ];
    let developer_dir = env::var("DEVELOPER_DIR").ok();
    let sdkroot = env::var("SDKROOT").ok();
    if let Some(value) = developer_dir.as_deref() {
        child_environment.push(("DEVELOPER_DIR", value));
    }
    if let Some(value) = sdkroot.as_deref() {
        child_environment.push(("SDKROOT", value));
    }

    let started = SystemTime::now();
    let cli_result = run_cli_with_timeout(
        &subjects_script,
        &args,
        &repository,
        &child_environment,
        None,
        Duration::from_secs(1200),
    );
    let completed_unix_ms = real_build_unix_millis();
    let elapsed_ms = started.elapsed().unwrap_or_default().as_millis();
    let run_json_path = evidence_root.join(&run_id).join("run.json");

    let output = match cli_result {
        Ok(output) => {
            let stdout_path = supervisor_root.join("stdout.bin");
            let stderr_path = supervisor_root.join("stderr.bin");
            real_build_write_new(&stdout_path, &output.stdout)
                .expect("persist raw bounded CLI stdout before assertions");
            real_build_write_new(&stderr_path, &output.stderr)
                .expect("persist raw bounded CLI stderr before assertions");
            let run_json = match fs::symlink_metadata(&run_json_path) {
                Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                    match fs::read(&run_json_path) {
                        Ok(bytes) => {
                            let sha256 = sha256_bytes(&bytes);
                            match serde_json::from_slice::<Value>(&bytes) {
                                Ok(record) => json!({
                                    "path": run_json_path.display().to_string(),
                                    "present": true,
                                    "raw_sha256": sha256,
                                    "state": record.get("state").cloned().unwrap_or(Value::Null),
                                }),
                                Err(error) => json!({
                                    "path": run_json_path.display().to_string(),
                                    "present": true,
                                    "raw_sha256": sha256,
                                    "parse_error": error.to_string(),
                                }),
                            }
                        }
                        Err(error) => json!({
                            "path": run_json_path.display().to_string(),
                            "present": false,
                            "read_error": error.to_string(),
                        }),
                    }
                }
                Ok(_) => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                    "read_error": "run.json is not a regular non-symlink file",
                }),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                }),
                Err(error) => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                    "read_error": error.to_string(),
                }),
            };
            let result = json!({
                "schema": "termrock-visibility-real-cli-result-v1",
                "outcome": if output.timed_out { "timed_out" } else if output.success() { "success" } else { "child_failure" },
                "run_id": run_id.as_str(),
                "started_unix_ms": started_unix_ms,
                "completed_unix_ms": completed_unix_ms,
                "elapsed_ms": elapsed_ms,
                "argv_file_sha256": raw_argv_file_sha256.as_str(),
                "argv_sha256": argv_sha256.as_str(),
                "run_json": run_json,
                "child": {
                    "exit_code": output.exit_code,
                    "signal": output.signal,
                    "timed_out": output.timed_out,
                    "stdout": {
                        "path": "stdout.bin",
                        "retained_bytes": output.stdout.len(),
                        "sha256": sha256_bytes(&output.stdout),
                        "truncated": output.stdout_truncated,
                    },
                    "stderr": {
                        "path": "stderr.bin",
                        "retained_bytes": output.stderr.len(),
                        "sha256": sha256_bytes(&output.stderr),
                        "truncated": output.stderr_truncated,
                    },
                },
                "outer_capture_limit_bytes_per_stream": CAPTURE_LIMIT,
                "nested_cargo_log_limitation": "If the supervisor kills the CLI during an in-flight Cargo child, Python may not flush that command's partial stdout/stderr. Preserve this outer capture and all completed command evidence; mark the build incomplete.",
                "qualification": "blocked",
            });
            real_build_write_json_new(&supervisor_root.join("result.json"), &result)
                .expect("persist structured CLI result before assertions");
            output
        }
        Err(error) => {
            let run_json = match fs::symlink_metadata(&run_json_path) {
                Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                    match fs::read(&run_json_path) {
                        Ok(bytes) => {
                            let sha256 = sha256_bytes(&bytes);
                            match serde_json::from_slice::<Value>(&bytes) {
                                Ok(record) => json!({
                                    "path": run_json_path.display().to_string(),
                                    "present": true,
                                    "raw_sha256": sha256,
                                    "state": record.get("state").cloned().unwrap_or(Value::Null),
                                }),
                                Err(parse_error) => json!({
                                    "path": run_json_path.display().to_string(),
                                    "present": true,
                                    "raw_sha256": sha256,
                                    "parse_error": parse_error.to_string(),
                                }),
                            }
                        }
                        Err(read_error) => json!({
                            "path": run_json_path.display().to_string(),
                            "present": false,
                            "read_error": read_error.to_string(),
                        }),
                    }
                }
                Ok(_) => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                    "read_error": "run.json is not a regular non-symlink file",
                }),
                Err(run_error) if run_error.kind() == std::io::ErrorKind::NotFound => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                }),
                Err(run_error) => json!({
                    "path": run_json_path.display().to_string(),
                    "present": false,
                    "read_error": run_error.to_string(),
                }),
            };
            let result = json!({
                "schema": "termrock-visibility-real-cli-result-v1",
                "outcome": "supervisor_io_error",
                "run_id": run_id.as_str(),
                "started_unix_ms": started_unix_ms,
                "completed_unix_ms": completed_unix_ms,
                "elapsed_ms": elapsed_ms,
                "argv_file_sha256": raw_argv_file_sha256.as_str(),
                "argv_sha256": argv_sha256.as_str(),
                "run_json": run_json,
                "supervisor_error": {
                    "kind": format!("{:?}", error.kind()),
                    "message": error.to_string(),
                },
                "outer_capture_limit_bytes_per_stream": CAPTURE_LIMIT,
                "nested_cargo_log_limitation": "subjects.py buffers nested Cargo stdout/stderr until each command returns; if the supervisor encounters an I/O error during a Cargo child, that command's partial output may not be flushed into run.json. This result records no child exit or stream fields because run_cli_with_timeout returned io::Error.",
                "qualification": "blocked",
            });
            real_build_write_json_new(&supervisor_root.join("result.json"), &result)
                .expect("persist structured supervisor error before assertion");
            panic!("real subject CLI supervisor failed: {error}");
        }
    };

    assert!(
        output.success(),
        "real subjects CLI failed; inspect preserved supervisor/result.json and raw streams"
    );
    assert!(
        !output.stdout_truncated && !output.stderr_truncated,
        "outer 8 MiB CLI capture must not truncate"
    );

    let run_json_bytes = fs::read(&run_json_path).expect("read final builder run JSON");
    let run_json_sha256 = sha256_bytes(&run_json_bytes);
    let run_record: Value =
        serde_json::from_slice(&run_json_bytes).expect("parse final builder run JSON");
    let manifest_path = PathBuf::from(
        run_record["subject_manifest"]
            .as_str()
            .expect("completed run records its manifest path"),
    );
    let manifest_bytes = fs::read(&manifest_path).expect("read real subject manifest");
    let manifest_sha256 = sha256_bytes(&manifest_bytes);
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).expect("parse real subject manifest");
    let manifest_reference = run_record["subject_manifest_sha256"]
        .as_str()
        .expect("run records manifest digest");
    let result_path = supervisor_root.join("result.json");
    let mut result: Value =
        serde_json::from_slice(&fs::read(&result_path).expect("read persisted supervisor result"))
            .expect("parse persisted supervisor result");
    result["run_json"]["raw_sha256"] = json!(run_json_sha256.as_str());
    result["subject_manifest"] = json!({
        "path": manifest_path.display().to_string(),
        "run_json_sha256": run_json_sha256.as_str(),
        "raw_sha256": manifest_sha256.as_str(),
        "run_json_digest_matches": manifest_reference == manifest_sha256,
    });
    let result_bytes =
        serde_json::to_vec_pretty(&result).expect("serialize bound supervisor result");
    let mut result_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&result_path)
        .expect("open owned supervisor result for hash binding");
    result_file
        .write_all(&result_bytes)
        .expect("bind raw run and manifest hashes in supervisor result");
    result_file
        .sync_all()
        .expect("sync bound supervisor result before assertions");

    assert_eq!(run_record["state"], "complete");
    assert_eq!(run_record["app"], "holla");
    assert_eq!(run_record["reference_commit"], REFERENCE_COMMIT);
    assert_eq!(run_record["candidate_commit"], CANDIDATE_COMMIT);
    assert_eq!(run_record["qualification"]["status"], "blocked");
    assert!(
        !run_record["qualification"]["reason"]
            .as_str()
            .unwrap()
            .trim()
            .is_empty()
    );
    assert_eq!(
        run_record["subject_manifest_sha256"], manifest_sha256,
        "run.json must bind the exact raw manifest bytes"
    );
    assert_eq!(
        manifest["schema"],
        "termrock-spec/parity-subject-manifest-v2"
    );
    assert_eq!(manifest["run_id"], run_id);
    assert_eq!(manifest["suite_revision"], suite_revision);
    assert_eq!(manifest["suite_sha256"], suite_sha256);
    assert!(manifest["expected_generation"].is_null());
    assert_eq!(manifest["subjects"].as_array().unwrap().len(), 2);
    assert!(
        run_record["source_input_evidence"]["sha256"]
            .as_str()
            .is_some()
    );
    assert!(
        run_record["build_environment_evidence"]["sha256"]
            .as_str()
            .is_some()
    );

    let mut seen_roles = BTreeSet::new();
    for subject in manifest["subjects"].as_array().unwrap() {
        let role = subject["role"].as_str().expect("subject role");
        assert!(
            seen_roles.insert(role.to_owned()),
            "subject role must be unique"
        );
        let expected_commit = match role {
            "reference" => REFERENCE_COMMIT,
            "candidate" => CANDIDATE_COMMIT,
            other => panic!("unexpected subject role {other}"),
        };
        assert_eq!(subject["source_commit"], expected_commit);
        assert_eq!(subject["build"]["target_name"], "holla");
        assert_eq!(subject["build"]["target_triple"], target_triple);
        assert_eq!(subject["build"]["toolchain"], "1.98.1");
        assert_eq!(subject["build"]["profile"], "release");
        assert_eq!(subject["build"]["default_features"], true);
        assert_eq!(subject["build"]["features"], json!([]));
        let executable_path = PathBuf::from(
            subject["executable"]["path"]
                .as_str()
                .expect("built executable path"),
        );
        assert!(executable_path.is_absolute());
        assert!(executable_path.is_file());
        assert_eq!(
            subject["executable"]["sha256"],
            sha256_file(&executable_path),
            "built executable bytes must match the manifest digest"
        );
        assert_eq!(
            subject["builder_receipt"]["executable_sha256"],
            subject["executable"]["sha256"]
        );
        let build_directory = evidence_root
            .join(&run_id)
            .join("build")
            .join(role)
            .join("holla");
        assert_eq!(
            fs::read_to_string(build_directory.join("build.exit"))
                .expect("read recorded Cargo build status"),
            "0\n"
        );
        let build_argv: Vec<String> = serde_json::from_slice(
            &fs::read(build_directory.join("build.argv.json"))
                .expect("read recorded Cargo build argv"),
        )
        .expect("parse recorded Cargo build argv");
        for required in [
            "build",
            "--release",
            "--locked",
            "--offline",
            "--jobs",
            "2",
            "--bin",
            "holla",
            "--target",
            target_triple.as_str(),
        ] {
            assert!(
                build_argv.iter().any(|argument| argument == required),
                "missing real build argument {required}"
            );
        }
    }
    assert_eq!(
        seen_roles,
        BTreeSet::from(["reference".to_owned(), "candidate".to_owned()])
    );

    let source_evidence_path = evidence_root.join(&run_id).join("source-inputs.json");
    let environment_evidence_path = evidence_root.join(&run_id).join("build-environment.json");
    let source_evidence_bytes = fs::read(&source_evidence_path).expect("read source-input sidecar");
    let environment_evidence_bytes =
        fs::read(&environment_evidence_path).expect("read build-environment sidecar");
    assert_eq!(
        run_record["source_input_evidence"]["sha256"],
        sha256_bytes(&source_evidence_bytes)
    );
    assert_eq!(
        run_record["build_environment_evidence"]["sha256"],
        sha256_bytes(&environment_evidence_bytes)
    );
    assert_eq!(
        manifest["build_evidence"]["source_inputs"]["path"],
        "source-inputs.json"
    );
    assert_eq!(
        manifest["build_evidence"]["source_inputs"]["sha256"],
        sha256_bytes(&source_evidence_bytes)
    );
    assert_eq!(
        manifest["build_evidence"]["build_environment"]["path"],
        "build-environment.json"
    );
    assert_eq!(
        manifest["build_evidence"]["build_environment"]["sha256"],
        sha256_bytes(&environment_evidence_bytes)
    );
    let source_evidence: Value =
        serde_json::from_slice(&source_evidence_bytes).expect("parse source-input sidecar");
    let environment_evidence: Value = serde_json::from_slice(&environment_evidence_bytes)
        .expect("parse build-environment sidecar");
    assert_eq!(source_evidence["qualification_status"], "blocked");
    assert_eq!(environment_evidence["run_id"], run_id);
    assert_eq!(
        environment_evidence["source_input_evidence"]["sha256"],
        sha256_bytes(&source_evidence_bytes)
    );
    assert_eq!(
        environment_evidence["source_commits"]["reference"],
        REFERENCE_COMMIT
    );
    assert_eq!(
        environment_evidence["source_commits"]["candidate"],
        CANDIDATE_COMMIT
    );
    let argv_file = fs::read(supervisor_root.join("argv.json")).expect("read raw argv evidence");
    assert_eq!(sha256_bytes(&argv_file), raw_argv_file_sha256);
    let argv_record: Value = serde_json::from_slice(&argv_file).expect("parse raw argv evidence");
    assert_eq!(argv_record["argv_sha256"], argv_sha256);
    assert_eq!(
        result["run_json"]["raw_sha256"], run_json_sha256,
        "supervisor result binds the exact raw run.json bytes"
    );
    assert_eq!(sha256_bytes(&argv_bytes), argv_sha256);
    assert_eq!(
        result["subject_manifest"]["run_json_digest_matches"], true,
        "supervisor receipt binds raw run.json and manifest bytes"
    );
    assert!(
        result["nested_cargo_log_limitation"]
            .as_str()
            .unwrap()
            .contains("partial stdout/stderr"),
        "timeout limitation must be explicit in the durable result"
    );
}

/// The only positive tag-receipt producer uses the real pinned tag and real Cargo.
/// An outer caller supplies a unique external run root, private cache/HOME, and exact
/// reviewed producer digest. Run with process-group timeout and an external Cargo slot.
#[cfg(unix)]
#[test]
#[ignore = "actual immutable-tag Holla build; requires a root-granted build slot"]
fn supervises_real_frozen_tag_holla_build() {
    const OUTER_TIMEOUT: Duration = Duration::from_secs(1200);
    const TAG_BUILDER_COMMAND: &str = "build-frozen-tag-holla";

    let run_id = real_build_required_env("TERMROCK_VIS06_TAG_RUN_ID");
    assert!(is_safe_fixture_run_id(&run_id), "caller run id is unsafe");
    let run_root_input = PathBuf::from(real_build_required_env("TERMROCK_VIS06_TAG_RUN_ROOT"));
    assert!(
        run_root_input.is_absolute(),
        "tag run root must be absolute"
    );
    let expected_run_root_name = format!("termrock-vis06-tag-holla-{run_id}");
    assert_eq!(
        run_root_input.file_name().and_then(|name| name.to_str()),
        Some(expected_run_root_name.as_str()),
        "tag run root must be unique to this run id"
    );
    let parent = run_root_input
        .parent()
        .expect("tag run root parent")
        .canonicalize()
        .expect("resolve external tag run-root parent");
    let run_root = parent.join(run_root_input.file_name().unwrap());
    assert!(
        !run_root.exists() && !run_root.is_symlink(),
        "refuse stale tag run root"
    );

    let cargo_home = PathBuf::from(real_build_required_env("TERMROCK_VIS06_TAG_CARGO_HOME"));
    let cargo_home_meta = fs::symlink_metadata(&cargo_home).expect("inspect private Cargo cache");
    assert!(cargo_home_meta.is_dir() && !cargo_home_meta.file_type().is_symlink());
    for name in ["config", "config.toml"] {
        assert!(!cargo_home.join(name).exists() && !cargo_home.join(name).is_symlink());
    }
    for name in ["registry", "git"] {
        let cache_entry = cargo_home.join(name);
        if cache_entry.exists() {
            let metadata = fs::symlink_metadata(&cache_entry).expect("inspect copied Cargo cache");
            assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
        }
    }
    let home = PathBuf::from(real_build_required_env("TERMROCK_VIS06_TAG_HOME"));
    let home_meta = fs::symlink_metadata(&home).expect("inspect private tag HOME");
    assert!(home_meta.is_dir() && !home_meta.file_type().is_symlink());
    assert_eq!(
        home_meta.permissions().mode() & 0o077,
        0,
        "private tag HOME must not be group/world accessible"
    );
    assert_eq!(
        cargo_home_meta.permissions().mode() & 0o077,
        0,
        "private Cargo cache must not be group/world accessible"
    );
    let rustup_home = PathBuf::from(real_build_required_env("TERMROCK_VIS06_TAG_RUSTUP_HOME"));
    assert!(rustup_home.is_dir(), "pinned Rustup home must exist");

    let path_value = env::var_os("PATH").expect("controlled PATH is required");
    let path_text = path_value.to_str().expect("controlled PATH must be UTF-8");
    assert!(
        !path_text
            .to_ascii_lowercase()
            .replace('\\', "/")
            .contains("/application support/mbx/bin")
            && !path_text
                .to_ascii_lowercase()
                .replace('\\', "/")
                .contains("/command-wrappers/bin"),
        "controlled PATH must exclude wrapper and MBX directories"
    );
    let sensitive_fragments = [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "CREDENTIAL",
        "PROXY",
        "ASKPASS",
    ];
    for (name, _) in env::vars_os() {
        let name = name.to_string_lossy().to_ascii_uppercase();
        assert!(
            !sensitive_fragments
                .iter()
                .any(|fragment| name.contains(fragment)),
            "run under a cleared environment; sensitive variable name {name} must be absent"
        );
    }
    let tool_paths = ["python3", "git", "rustup"]
        .into_iter()
        .map(|name| {
            let (path, sha256) = real_build_resolve_executable(name, &path_value);
            (name.to_owned(), json!({"path": path, "sha256": sha256}))
        })
        .collect::<serde_json::Map<_, _>>();
    let subjects_script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/visibility/subjects.py")
        .canonicalize()
        .expect("resolve frozen subjects producer");
    let expected_subjects_sha256 = real_build_required_env("TERMROCK_VIS06_TAG_SUBJECTS_SHA256");
    assert_eq!(expected_subjects_sha256.len(), 64);
    assert!(
        expected_subjects_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    );
    assert_eq!(
        sha256_file(&subjects_script),
        expected_subjects_sha256,
        "tag builder must match the caller-pinned reviewed producer"
    );
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve product repository");

    let evidence_root = run_root.join("evidence");
    assert!(!evidence_root.exists() && !evidence_root.is_symlink());
    let supervisor_root = run_root.join("supervisor");
    fs::create_dir_all(&supervisor_root).expect("create owned supervisor log directory");
    fs::set_permissions(&run_root, fs::Permissions::from_mode(0o700))
        .expect("restrict owned tag run root");
    fs::set_permissions(&supervisor_root, fs::Permissions::from_mode(0o700))
        .expect("restrict tag supervisor logs");
    let args = vec![
        TAG_BUILDER_COMMAND.to_owned(),
        "--repository".to_owned(),
        repository.display().to_string(),
        "--evidence-root".to_owned(),
        evidence_root.display().to_string(),
        "--run-id".to_owned(),
        run_id.clone(),
        "--target-triple".to_owned(),
        "aarch64-apple-darwin".to_owned(),
    ];
    let raw_argv = json!({
        "program": "python3",
        "script": subjects_script.display().to_string(),
        "arguments": args.clone(),
    });
    let argv_bytes = serde_json::to_vec(&raw_argv).expect("serialize raw tag builder argv");
    fs::write(supervisor_root.join("argv.json"), &argv_bytes).expect("preserve raw argv");
    let mut environment_owned = vec![
        ("PATH".to_owned(), path_text.to_owned()),
        (
            "HOME".to_owned(),
            home.to_str().expect("HOME path UTF-8").to_owned(),
        ),
        (
            "CARGO_HOME".to_owned(),
            cargo_home.to_str().expect("Cargo cache UTF-8").to_owned(),
        ),
        (
            "RUSTUP_HOME".to_owned(),
            rustup_home.to_str().expect("Rustup home UTF-8").to_owned(),
        ),
    ];
    for name in ["DEVELOPER_DIR", "SDKROOT"] {
        if let Ok(value) = env::var(name) {
            environment_owned.push((name.to_owned(), value));
        }
    }
    let environment: Vec<(&str, &str)> = environment_owned
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    let output = run_cli_with_timeout(
        &subjects_script,
        &args,
        &repository,
        &environment,
        None,
        OUTER_TIMEOUT,
    )
    .expect("launch bounded real tag build CLI");
    fs::write(supervisor_root.join("stdout.bin"), &output.stdout)
        .expect("preserve bounded tag CLI stdout");
    fs::write(supervisor_root.join("stderr.bin"), &output.stderr)
        .expect("preserve bounded tag CLI stderr");
    let result_record = json!({
        "argv_sha256": sha256_bytes(&argv_bytes),
        "subjects_script_sha256": expected_subjects_sha256,
        "tool_paths": tool_paths,
        "exit_code": output.exit_code,
        "signal": output.signal,
        "timed_out": output.timed_out,
        "stdout_sha256": sha256_bytes(&output.stdout),
        "stderr_sha256": sha256_bytes(&output.stderr),
        "stdout_truncated": output.stdout_truncated,
        "stderr_truncated": output.stderr_truncated,
    });
    fs::write(
        supervisor_root.join("result.json"),
        serde_json::to_vec(&result_record).expect("serialize tag supervisor result"),
    )
    .expect("preserve tag supervisor result");
    assert!(
        !output.timed_out,
        "real tag builder exceeded its bounded deadline"
    );
    assert!(
        !output.stdout_truncated && !output.stderr_truncated,
        "real tag builder output exceeded the retained log bound"
    );
    assert_eq!(output.exit_code, Some(0), "{}", error_text(&output));
    assert!(output.signal.is_none());

    let receipt_path = PathBuf::from(output_text(&output).trim());
    assert_eq!(
        receipt_path,
        evidence_root.join(&run_id).join("tag-builder-receipt.json")
    );
    let receipt_bytes = fs::read(&receipt_path).expect("read actual immutable-tag receipt");
    let receipt: Value = serde_json::from_slice(&receipt_bytes).expect("parse tag receipt");
    let receipt_fields: BTreeSet<String> = [
        "schema",
        "run_id",
        "purpose",
        "build_result",
        "qualification",
        "oracle_lineage",
        "source_snapshot",
        "build_artifact_root",
        "build",
        "executable",
        "builder_receipt",
        "builder_receipt_sha256",
        "build_environment",
        "capture_status",
        "admission_status",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(object_keys(&receipt), receipt_fields);
    assert_eq!(
        receipt["schema"],
        "termrock-spec/visual-tag-holla-build-evidence-v1"
    );
    assert_eq!(receipt["build_result"], "PASS");
    assert_eq!(receipt["qualification"]["status"], "blocked");
    assert!(
        receipt["qualification"]["reason"]
            .as_str()
            .unwrap()
            .trim()
            .len()
            > 0
    );
    assert_eq!(receipt["capture_status"], "NOT_RUN");
    assert_eq!(receipt["admission_status"], "NOT_RUN");
    assert_eq!(
        object_keys(&receipt["qualification"]),
        BTreeSet::from(["reason".to_owned(), "status".to_owned()])
    );
    assert_eq!(
        object_keys(&receipt["oracle_lineage"]),
        BTreeSet::from([
            "tag_ref".to_owned(),
            "tag_object".to_owned(),
            "tag_commit".to_owned(),
            "git_object_replacement_policy".to_owned(),
            "git_lazy_fetch_policy".to_owned(),
        ])
    );
    assert_eq!(
        object_keys(&receipt["source_snapshot"]),
        BTreeSet::from([
            "source_commit".to_owned(),
            "tree_oid".to_owned(),
            "git_object_replacement_policy".to_owned(),
            "recipe".to_owned(),
            "archive_argv".to_owned(),
            "git_ls_tree_stdout_sha256".to_owned(),
            "excluded_roots".to_owned(),
            "path_blob_digest_algorithm".to_owned(),
            "tracked_path_blob_map_sha256".to_owned(),
            "tracked_file_count".to_owned(),
            "included_path_blob_map_sha256".to_owned(),
            "included_file_count".to_owned(),
            "excluded_path_blob_map_sha256".to_owned(),
            "excluded_file_count".to_owned(),
            "archive_sha256".to_owned(),
            "archive_member_count".to_owned(),
            "read_only".to_owned(),
            "materialized_root".to_owned(),
        ])
    );
    assert_eq!(
        object_keys(&receipt["build"]),
        BTreeSet::from([
            "package_id".to_owned(),
            "target_name".to_owned(),
            "features".to_owned(),
            "default_features".to_owned(),
            "target_triple".to_owned(),
            "toolchain".to_owned(),
            "profile".to_owned(),
        ])
    );
    assert_eq!(
        object_keys(&receipt["executable"]),
        BTreeSet::from(["path".to_owned(), "sha256".to_owned()])
    );
    assert_eq!(
        object_keys(&receipt["build_environment"]),
        BTreeSet::from([
            "cargo_config_sha256".to_owned(),
            "external_cargo_config_policy".to_owned(),
            "rustup".to_owned(),
            "toolchain_binaries".to_owned(),
            "cargo_version".to_owned(),
            "rustc_version".to_owned(),
            "host_triple".to_owned(),
            "target_triple".to_owned(),
            "environment".to_owned(),
        ])
    );
    assert_eq!(
        receipt["oracle_lineage"]["tag_object"],
        REFERENCE_TAG_OBJECT
    );
    assert_eq!(receipt["oracle_lineage"]["tag_commit"], ORACLE_COMMIT_SHA);
    assert_eq!(receipt["build"]["target_name"], "holla");
    assert_eq!(receipt["build"]["target_triple"], "aarch64-apple-darwin");
    assert_eq!(receipt["executable"]["sha256"].as_str().unwrap().len(), 64);
    let executable = PathBuf::from(receipt["executable"]["path"].as_str().unwrap());
    let artifact_root = PathBuf::from(receipt["build_artifact_root"].as_str().unwrap());
    assert!(artifact_root.is_absolute());
    assert!(executable.starts_with(&artifact_root));
    let executable_sha256 = sha256_file(&executable);
    assert_eq!(
        Some(executable_sha256.as_str()),
        receipt["executable"]["sha256"].as_str()
    );
    let source_root = PathBuf::from(
        receipt["source_snapshot"]["materialized_root"]
            .as_str()
            .unwrap(),
    );
    let source_meta = fs::symlink_metadata(&source_root).expect("inspect read-only tag snapshot");
    assert!(source_meta.is_dir() && !source_meta.file_type().is_symlink());
    assert_eq!(source_meta.permissions().mode() & 0o222, 0);

    let builder_receipt = receipt["builder_receipt"]
        .as_object()
        .expect("nested builder receipt");
    assert_eq!(
        builder_receipt.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "schema".to_owned(),
            "source_commit".to_owned(),
            "package_id".to_owned(),
            "target_name".to_owned(),
            "requested_features".to_owned(),
            "default_features".to_owned(),
            "target_triple".to_owned(),
            "toolchain".to_owned(),
            "profile".to_owned(),
            "manifest_sha256".to_owned(),
            "lock_sha256".to_owned(),
            "mise_config".to_owned(),
            "cargo_version".to_owned(),
            "rustc_version".to_owned(),
            "host_triple".to_owned(),
            "executed_argv".to_owned(),
            "executable_path".to_owned(),
            "executable_sha256".to_owned(),
            "sha256".to_owned(),
        ])
    );
    let mut builder_payload = builder_receipt.clone();
    let nested_sha = builder_payload
        .remove("sha256")
        .and_then(|value| value.as_str().map(str::to_owned))
        .expect("nested canonical builder receipt hash");
    assert_eq!(
        Some(nested_sha.as_str()),
        receipt["builder_receipt_sha256"].as_str()
    );
    assert_eq!(
        nested_sha,
        sha256_bytes(&serde_json::to_vec(&builder_payload).unwrap())
    );
    assert_eq!(
        builder_receipt["source_commit"].as_str(),
        Some(ORACLE_COMMIT_SHA)
    );

    let run_record_path = evidence_root.join(&run_id).join("run.json");
    let run_record_bytes = fs::read(&run_record_path).expect("read final tag run record");
    let run_record: Value =
        serde_json::from_slice(&run_record_bytes).expect("parse final tag run record");
    assert_eq!(run_record["build_result"], "PASS");
    let receipt_sha256 = sha256_bytes(&receipt_bytes);
    assert_eq!(
        run_record["tag_builder_receipt_sha256"].as_str(),
        Some(receipt_sha256.as_str())
    );
    assert_eq!(run_record["capture_status"], "NOT_RUN");
    assert_eq!(run_record["admission_status"], "NOT_RUN");
    let sealed_supervisor = json!({
        "argv_sha256": sha256_bytes(&argv_bytes),
        "run_json_sha256": sha256_bytes(&run_record_bytes),
        "tag_builder_receipt_sha256": sha256_bytes(&receipt_bytes),
        "subjects_script_sha256": expected_subjects_sha256,
        "tool_paths": tool_paths,
        "exit_code": output.exit_code,
        "signal": output.signal,
        "timed_out": output.timed_out,
        "stdout_sha256": sha256_bytes(&output.stdout),
        "stderr_sha256": sha256_bytes(&output.stderr),
        "stdout_truncated": output.stdout_truncated,
        "stderr_truncated": output.stderr_truncated,
    });
    fs::write(
        supervisor_root.join("result.json"),
        serde_json::to_vec(&sealed_supervisor).expect("serialize sealed tag supervisor result"),
    )
    .expect("bind raw run and receipt hashes into supervisor result");
}
