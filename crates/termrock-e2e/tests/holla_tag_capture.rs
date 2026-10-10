use termrock_e2e::{ORACLE_CAPTURE_RECEIPT_SCHEMA_V2, run_tag_capture};

/// This explicit ignored lane launches only the immutable-tag Holla binary.
/// It records actual captures but cannot admit an expected generation.
#[test]
#[ignore = "requires an independently built immutable-tag Holla binary and isolated capture roots"]
fn captures_holla_from_pinned_immutable_tag() {
    let receipt =
        run_tag_capture("HELP-HOLLA-004").expect("capture the real immutable-tag Holla case");
    assert_eq!(receipt.schema, ORACLE_CAPTURE_RECEIPT_SCHEMA_V2);
    assert_eq!(
        receipt.oracle_identity.tag_commit,
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    );
    assert_eq!(receipt.capture_status, "COMPLETE");
    assert!(receipt.builder_evidence.is_none());
    assert!(receipt.builder_run.is_none());
    assert!(receipt.oracle_identity.builder_receipt_sha256.is_none());
    let receipt_json = serde_json::to_value(&receipt).expect("serialize the capture receipt");
    let origin = receipt_json
        .get("oracle_identity")
        .and_then(|identity| identity.get("normalized_existing_build_origin"))
        .expect("R6 capture retains its normalized existing-build origin");
    assert_eq!(
        origin.get("schema").and_then(serde_json::Value::as_str),
        Some("termrock-spec/visual-tag-holla-normalized-existing-build-origin-v2")
    );
    let addendum_path = std::env::var("TERMROCK_E2E_ORACLE_NORMALIZED_BUILD_ADDENDUM")
        .expect("capture invocation pins the R6 source addendum");
    let addendum_sha256 = std::env::var("TERMROCK_E2E_ORACLE_NORMALIZED_BUILD_ADDENDUM_SHA256")
        .expect("capture invocation pins the R6 source addendum digest");
    let origin_addendum = origin
        .get("normalization_addendum")
        .expect("normalized origin binds its source addendum");
    assert_eq!(
        origin_addendum.get("path").and_then(serde_json::Value::as_str),
        Some(addendum_path.as_str())
    );
    assert_eq!(
        origin_addendum
            .get("sha256")
            .and_then(serde_json::Value::as_str),
        Some(addendum_sha256.as_str())
    );
    let addendum_bytes = std::fs::read(&addendum_path)
        .expect("read the exact R6 normalization addendum");
    let addendum: serde_json::Value =
        serde_json::from_slice(&addendum_bytes).expect("parse the R6 normalization addendum");
    let build_result = addendum
        .get("bindings")
        .and_then(|bindings| bindings.get("build_result"))
        .expect("R6 addendum binds its actual build result");
    let expected_result_path = build_result
        .get("path")
        .and_then(serde_json::Value::as_str)
        .expect("R6 build result path is a string");
    let expected_result_sha256 = build_result
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .expect("R6 build result digest is a string");
    let origin_build_result = origin
        .get("build_result")
        .expect("normalized origin binds the actual build result");
    assert_eq!(
        origin_build_result
            .get("path")
            .and_then(serde_json::Value::as_str),
        Some(expected_result_path)
    );
    assert_eq!(
        origin_build_result
            .get("sha256")
            .and_then(serde_json::Value::as_str),
        Some(expected_result_sha256)
    );
    assert_eq!(receipt.artifacts.len(), 40);
    assert_eq!(receipt.case.checkpoints.len(), 4);
    assert_eq!(
        receipt
            .checks
            .iter()
            .filter(|check| {
                check.subject_role == "oracle"
                    && check.case_id == "HELP-HOLLA-004"
                    && check.dimension == "interaction"
                    && check.id.split(':').count() == 3
            })
            .count(),
        12
    );
    assert_eq!(receipt.execution_anchor_status, "unverified");
    assert_eq!(receipt.qualification.status, "blocked");
    assert_eq!(receipt.admission_status, "NOT_RUN");
    assert!(
        receipt
            .oracle_identity
            .validator_execution
            .git_dir
            .is_absolute()
    );
    assert!(
        receipt
            .oracle_identity
            .validator_execution
            .git_common_dir
            .is_absolute()
    );
    assert!(
        receipt
            .checks
            .iter()
            .all(|check| { check.dimension != "visual" || check.status != "PASS" })
    );
}

#[test]
#[ignore = "revalidates pinned R6 evidence without launching Holla; requires the reviewed R6 environment"]
fn validates_r6_normalized_preflight_and_rejects_mutated_origin_inputs_without_launch() {
    use sha2::{Digest, Sha256};
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    const ADDENDUM_PATH_ENV: &str = "TERMROCK_E2E_ORACLE_NORMALIZED_BUILD_ADDENDUM";
    const ADDENDUM_SHA_ENV: &str = "TERMROCK_E2E_ORACLE_NORMALIZED_BUILD_ADDENDUM_SHA256";
    const EXPECTED_ADDENDUM_PATH: &str = "/private/tmp/termrock-vis06-tag-four-binary-r6-20261010/source-qualification-addendum-r1.json";
    const EXPECTED_ADDENDUM_SHA: &str = "a91634352ec97829a2a2e4731fc4258a91c32c9d294fa8c943d11db701562b57";

    struct EnvRestore {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvRestore {
        fn set(key: &'static str, value: OsString) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: this ignored integration check is run alone with one test thread;
            // no other test reads process environment while the validator is invoked.
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            // SAFETY: restored by the same isolated single-test process described above.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    struct ScratchDir(PathBuf);

    impl ScratchDir {
        fn new() -> Self {
            let parent = fs::canonicalize(std::env::temp_dir())
                .expect("canonicalize the caller-provided external test temp directory");
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock is after Unix epoch")
                .as_nanos();
            for attempt in 0..32u8 {
                let path = parent.join(format!(
                    "termrock-vis06-r7-preflight-{}-{nonce}-{attempt}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create external test scratch directory: {error}"),
                }
            }
            panic!("could not allocate a unique external test scratch directory");
        }

        fn write_json(&self, name: &str, value: &serde_json::Value) -> (PathBuf, String) {
            let path = self.0.join(name);
            let bytes = serde_json::to_vec(value).expect("serialize test evidence copy");
            fs::write(&path, &bytes).expect("write test evidence copy");
            let digest = format!("{:x}", Sha256::digest(&bytes));
            (fs::canonicalize(path).expect("canonicalize test evidence copy"), digest)
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn call_with_addendum(path: &Path, digest: &str) -> Result<(), String> {
        let _path = EnvRestore::set(ADDENDUM_PATH_ENV, path.as_os_str().to_os_string());
        let _digest = EnvRestore::set(ADDENDUM_SHA_ENV, OsString::from(digest));
        termrock_e2e::prepare_frozen_tag_holla_preflight("HELP-HOLLA-004").map(|_| ())
    }

    let addendum_path = PathBuf::from(
        std::env::var_os(ADDENDUM_PATH_ENV).expect("caller pins the R6 addendum path"),
    );
    let addendum_sha = std::env::var(ADDENDUM_SHA_ENV)
        .expect("caller pins the R6 addendum digest");
    assert_eq!(addendum_path, Path::new(EXPECTED_ADDENDUM_PATH));
    assert_eq!(addendum_sha, EXPECTED_ADDENDUM_SHA);

    let preflight = termrock_e2e::prepare_frozen_tag_holla_preflight("HELP-HOLLA-004")
        .expect("the production helper and Rust loader accept the exact reviewed R6 origin");
    let preflight = serde_json::to_value(preflight).expect("serialize read-only preflight");
    assert_eq!(preflight["state"], "blocked");
    assert_eq!(preflight["run_id"], "tag-four-binary-r6-20261010");
    assert_eq!(preflight["execution_anchor_status"], "unverified");
    assert_eq!(preflight["qualification"]["status"], "blocked");
    assert_eq!(preflight["capture_status"], "NOT_RUN");
    assert_eq!(preflight["admission_status"], "NOT_RUN");
    let identity = &preflight["tag_identity"];
    assert_eq!(identity["product_build"], "PASS");
    assert_eq!(identity["source_payload"], "VERIFIED");
    assert_eq!(identity["locked_cache_payload"], "VERIFIED");
    assert_eq!(identity["environment_integrity"], "INCOMPLETE");
    assert_eq!(identity["execution_anchor_status"], "unverified");
    assert_eq!(identity["qualification"]["status"], "blocked");
    assert_eq!(identity["capture_status"], "NOT_RUN");
    assert_eq!(identity["admission_status"], "NOT_RUN");
    assert_eq!(identity["source_snapshot"]["included_file_count"], 1415);
    assert_eq!(
        identity["source_snapshot"]["included_path_blob_map_sha256"],
        "4e2cf9d17700833023c1c70b33f4b11e710ccb60fbc8c9c9d727c81745436bb4"
    );
    assert_eq!(identity["normalization_addendum"]["path"], EXPECTED_ADDENDUM_PATH);
    assert_eq!(identity["normalization_addendum"]["sha256"], EXPECTED_ADDENDUM_SHA);

    let scratch = ScratchDir::new();
    let scratch_path = fs::canonicalize(&scratch.0).expect("canonicalize scratch root");
    let source_root = PathBuf::from(
        identity["source_snapshot"]["materialized_root"]
            .as_str()
            .expect("preflight source root is bound"),
    );
    let artifact_root = PathBuf::from(
        identity["build_artifact_root"]
            .as_str()
            .expect("preflight artifact root is bound"),
    );
    let actual_root = PathBuf::from(
        std::env::var_os("TERMROCK_E2E_ORACLE_ACTUAL_ROOT")
            .expect("caller pins an existing actual-output root"),
    );
    let suite_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("suite checkout root is an ancestor of the package");
    for protected in [
        suite_root.to_path_buf(),
        source_root,
        artifact_root,
        fs::canonicalize(actual_root).expect("canonicalize actual-output root"),
    ] {
        assert!(
            !scratch_path.starts_with(&protected) && !protected.starts_with(&scratch_path),
            "test evidence scratch path overlaps protected input/output {}",
            protected.display()
        );
    }

    let original_addendum_bytes = fs::read(&addendum_path).expect("read exact R6 addendum");
    let original_addendum: serde_json::Value = serde_json::from_slice(&original_addendum_bytes)
        .expect("parse exact R6 addendum");
    let original_result_path = PathBuf::from(
        original_addendum["bindings"]["original_build_result"]["path"]
            .as_str()
            .expect("R6 result path is bound"),
    );
    let original_result_bytes = fs::read(&original_result_path).expect("read exact R6 result");
    let original_result: serde_json::Value = serde_json::from_slice(&original_result_bytes)
        .expect("parse exact R6 result");

    let mut wrong_origin_digest = original_addendum.clone();
    wrong_origin_digest["bindings"]["original_build_result"]["sha256"] =
        serde_json::Value::String("0".repeat(64));
    let (wrong_digest_path, wrong_digest_sha) =
        scratch.write_json("wrong-origin-digest.json", &wrong_origin_digest);
    let wrong_digest_error = call_with_addendum(&wrong_digest_path, &wrong_digest_sha)
        .expect_err("the production helper rejects a tampered R6 build-result digest");
    assert!(
        wrong_digest_error.contains("original_build_result bytes differ"),
        "unexpected origin-digest rejection: {wrong_digest_error}"
    );

    let mut changed_map = original_addendum.clone();
    changed_map["tag_source_payload"]["path_blob_map_sha256"] =
        serde_json::Value::String("0".repeat(64));
    let (changed_map_path, changed_map_sha) =
        scratch.write_json("changed-source-map.json", &changed_map);
    let changed_map_error = call_with_addendum(&changed_map_path, &changed_map_sha)
        .expect_err("the production helper rejects a changed immutable-tag source map");
    assert!(
        changed_map_error.contains("R6 source snapshot does not match the immutable visual-baseline tag"),
        "unexpected source-map rejection: {changed_map_error}"
    );

    let mut promoted_result = original_result;
    promoted_result["protected_inputs"]["environment_integrity"] =
        serde_json::Value::String("VERIFIED".to_string());
    let (promoted_result_path, promoted_result_sha) =
        scratch.write_json("promoted-environment-result.json", &promoted_result);
    let mut promoted_environment = original_addendum;
    promoted_environment["bindings"]["original_build_result"]["path"] =
        serde_json::Value::String(promoted_result_path.display().to_string());
    promoted_environment["bindings"]["original_build_result"]["sha256"] =
        serde_json::Value::String(promoted_result_sha);
    let (promoted_addendum_path, promoted_addendum_sha) =
        scratch.write_json("promoted-environment-addendum.json", &promoted_environment);
    let promoted_error = call_with_addendum(&promoted_addendum_path, &promoted_addendum_sha)
        .expect_err("the production helper rejects promotion of R6 environment integrity");
    assert!(
        promoted_error.contains("R6 protected-input states differ from the independently reviewed outcome"),
        "unexpected environment-state rejection: {promoted_error}"
    );

    assert_eq!(
        format!("{:x}", Sha256::digest(&fs::read(&addendum_path).expect("re-read original addendum"))),
        EXPECTED_ADDENDUM_SHA,
        "preflight tests never mutate the original R6 addendum"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&fs::read(&original_result_path).expect("re-read original result"))),
        "3cce15319b9abea255d34a8984d7a4f900e6a9a029b9f2bc61055e7429852b82",
        "preflight tests never mutate the original R6 result"
    );
}
