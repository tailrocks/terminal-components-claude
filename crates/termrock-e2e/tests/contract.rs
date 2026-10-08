use termrock_e2e::{
    BuildFacts, BuildInputs, BuilderReceipt, Executable, ExpectedGenerationInput, MiseConfigInput,
    SUBJECT_SCHEMA, Subject, SubjectManifest, builder_receipt_digest, deferred_row_count, registry,
    suite_digest, validate_subject_manifest,
};

#[test]
fn registry_contains_one_holla_pilot_with_checkpointed_preconditions() {
    let registry = registry().expect("registry parses");
    assert_eq!(registry.schema, "termrock-e2e/case-registry-v1");
    assert_eq!(registry.cases.len(), 1);
    let case = &registry.cases[0];
    assert_eq!(case.id, "HELP-HOLLA-004");
    assert_eq!(case.app, "holla");
    assert_eq!(case.geometry.cols, 120);
    assert_eq!(case.geometry.rows, 40);
    assert_eq!(case.color_path, "truecolor");
    assert_eq!(case.legacy_snapshot_root, "holla/flows/help/overlay");
    assert!(termrock_e2e::validate_case_contract(case).is_ok());

    let mut assertion_ids = std::collections::BTreeSet::new();
    let mut positive_ids = std::collections::BTreeSet::new();
    for step in &case.steps {
        if let termrock_e2e::Step::Checkpoint { assertions, .. } = step {
            assert!(!assertions.is_empty(), "checkpoints need assertions");
            for assertion in assertions {
                assert!(
                    !assertion_ids.contains(assertion.id.as_str()),
                    "duplicate assertion ID {}",
                    assertion.id
                );
                if let Some(required) = &assertion.requires {
                    assert!(
                        assertion.kind == "absent" && positive_ids.contains(required.as_str()),
                        "negative assertion must require an earlier positive precondition {required}"
                    );
                }
                if matches!(assertion.kind.as_str(), "contains" | "same_line") {
                    positive_ids.insert(assertion.id.as_str());
                }
                assertion_ids.insert(assertion.id.as_str());
            }
        }
    }
    assert!(assertion_ids.contains("boot.finder_footer"));
    assert!(assertion_ids.contains("help.quit_row"));

    let mut empty_checkpoint = case.clone();
    let termrock_e2e::Step::Checkpoint { assertions, .. } = &mut empty_checkpoint.steps[0] else {
        panic!("pilot begins with a checkpoint");
    };
    assertions.clear();
    assert!(termrock_e2e::validate_case_contract(&empty_checkpoint).is_err());

    let mut non_positive_prerequisite = case.clone();
    for step in &mut non_positive_prerequisite.steps {
        if let termrock_e2e::Step::Checkpoint { assertions, .. } = step {
            if let Some(assertion) = assertions
                .iter_mut()
                .find(|assertion| assertion.id == "finder.quit_row_absent")
            {
                assertion.requires = Some("help.finder_footer_absent".to_string());
            }
        }
    }
    assert!(termrock_e2e::validate_case_contract(&non_positive_prerequisite).is_err());
}

#[test]
fn all_known_deferred_rows_remain_not_run_and_bd19_stays_absent() {
    assert_eq!(
        deferred_row_count().expect("deferred inventory parses"),
        (23, 25, "BD-19".to_string())
    );
}

#[test]
fn suite_digest_covers_the_independent_package() {
    let digest = suite_digest().expect("suite digest is available");
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn subject_manifest_requires_one_pinned_pair() {
    let case_revision = registry().expect("registry parses").suite_revision;
    let manifest = SubjectManifest {
        schema: SUBJECT_SCHEMA.to_string(),
        run_id: "test-run".to_string(),
        suite_revision: case_revision,
        suite_sha256: "1".repeat(64),
        expected_generation: ExpectedGenerationInput {
            id: "shared-parity-v1".to_string(),
            root: None,
            sha256: "0".repeat(64),
        },
        subjects: vec![subject("reference", 'a'), subject("candidate", 'b')],
    };
    assert!(validate_subject_manifest(&manifest).is_ok());

    let mut duplicate = manifest.clone();
    duplicate.subjects[1].role = "reference".to_string();
    assert!(validate_subject_manifest(&duplicate).is_err());

    let mut relative_executable = manifest.clone();
    relative_executable.subjects[0].executable.path = std::path::PathBuf::from("holla");
    assert!(
        validate_subject_manifest(&relative_executable)
            .unwrap_err()
            .contains("must be absolute")
    );

    let mut unpinned = manifest;
    unpinned.subjects[0].source_commit = "abc".to_string();
    assert!(validate_subject_manifest(&unpinned).is_err());

    let mut malformed_build_input = SubjectManifest {
        schema: SUBJECT_SCHEMA.to_string(),
        run_id: "test-run".to_string(),
        suite_revision: registry().expect("registry parses").suite_revision,
        suite_sha256: "1".repeat(64),
        expected_generation: ExpectedGenerationInput {
            id: "shared-parity-v1".to_string(),
            root: None,
            sha256: "0".repeat(64),
        },
        subjects: vec![subject("reference", 'a'), subject("candidate", 'b')],
    };
    malformed_build_input.subjects[0]
        .build_inputs
        .manifest_sha256 = "bad".to_string();
    assert!(validate_subject_manifest(&malformed_build_input).is_err());

    let mut mismatched_builder_receipt = SubjectManifest {
        schema: SUBJECT_SCHEMA.to_string(),
        run_id: "test-run".to_string(),
        suite_revision: registry().expect("registry parses").suite_revision,
        suite_sha256: "1".repeat(64),
        expected_generation: ExpectedGenerationInput {
            id: "shared-parity-v1".to_string(),
            root: None,
            sha256: "0".repeat(64),
        },
        subjects: vec![subject("reference", 'a'), subject("candidate", 'b')],
    };
    mismatched_builder_receipt.subjects[0].builder_receipt_sha256 = "0".repeat(64);
    assert!(validate_subject_manifest(&mismatched_builder_receipt).is_err());
}

#[test]
fn receipt_schema_validates_complete_record_and_rejects_malformed_nested_data() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/receipt-v1.schema.json"))
            .expect("receipt schema parses");
    assert!(jsonschema::draft202012::meta::is_valid(&schema));
    let validator = jsonschema::draft202012::new(&schema).expect("draft 2020-12 schema compiles");
    let receipt = valid_receipt();
    assert!(
        validator.is_valid(&receipt),
        "valid fixture rejected: {receipt}"
    );

    let mut bad_build_type = receipt.clone();
    *bad_build_type
        .pointer_mut("/source_pair/0/build/default_features")
        .expect("nested build field") = serde_json::json!("true");
    assert!(!validator.is_valid(&bad_build_type));

    let mut missing_builder_identity = receipt.clone();
    missing_builder_identity
        .pointer_mut("/source_pair/0/builder_receipt")
        .and_then(serde_json::Value::as_object_mut)
        .expect("builder receipt object")
        .remove("executable_sha256");
    assert!(!validator.is_valid(&missing_builder_identity));

    let mut bad_suite_digest_type = receipt.clone();
    *bad_suite_digest_type
        .pointer_mut("/suite/digest")
        .expect("suite digest") = serde_json::json!(12);
    assert!(!validator.is_valid(&bad_suite_digest_type));

    let mut bad_check_identity_type = receipt.clone();
    *bad_check_identity_type
        .pointer_mut("/checks/0/case_id")
        .expect("check case ID") = serde_json::json!(false);
    assert!(!validator.is_valid(&bad_check_identity_type));

    let mut missing_write_policy = receipt.clone();
    missing_write_policy
        .as_object_mut()
        .expect("receipt object")
        .remove("write_policy");
    assert!(!validator.is_valid(&missing_write_policy));

    let mut missing_trust_policy_digest = receipt.clone();
    missing_trust_policy_digest["trust"]
        .as_object_mut()
        .expect("trust object")
        .remove("write_policy_sha256");
    assert!(!validator.is_valid(&missing_trust_policy_digest));

    let mut bad_protected_root_kind = receipt.clone();
    bad_protected_root_kind["write_policy"]["protected_roots"][0]["kind"] =
        serde_json::json!(false);
    assert!(!validator.is_valid(&bad_protected_root_kind));

    let mut repeated_output_role = receipt.clone();
    repeated_output_role["write_policy"]["actual_output_roots"][1]["role"] =
        serde_json::json!("reference");
    assert!(!validator.is_valid(&repeated_output_role));

    let mut unknown_nested_key = receipt.clone();
    unknown_nested_key["source_pair"][0]["build"]["surprise"] = serde_json::json!(true);
    assert!(!validator.is_valid(&unknown_nested_key));

    let mut wrong_pair_roles = receipt.clone();
    wrong_pair_roles["source_pair"][1]["role"] = serde_json::json!("reference");
    assert!(!validator.is_valid(&wrong_pair_roles));

    let mut missing_artifact_size = receipt;
    missing_artifact_size
        .pointer_mut("/artifacts/0")
        .and_then(serde_json::Value::as_object_mut)
        .expect("artifact object")
        .remove("bytes");
    assert!(!validator.is_valid(&missing_artifact_size));
}

#[test]
fn trust_record_schema_requires_the_bound_write_policy_digest() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/trust-record-v1.schema.json"))
            .expect("trust-record schema parses");
    assert!(jsonschema::draft202012::meta::is_valid(&schema));
    let validator =
        jsonschema::draft202012::new(&schema).expect("draft 2020-12 trust-record schema compiles");
    let record = valid_trust_record();
    assert!(
        validator.is_valid(&record),
        "valid fixture rejected: {record}"
    );

    let mut missing_policy = record.clone();
    missing_policy
        .as_object_mut()
        .expect("record object")
        .remove("write_policy_sha256");
    assert!(!validator.is_valid(&missing_policy));

    let mut malformed_policy = record.clone();
    malformed_policy["write_policy_sha256"] = serde_json::json!(false);
    assert!(!validator.is_valid(&malformed_policy));

    let mut malformed_subject = record;
    malformed_subject["subjects"][0]["builder_receipt_sha256"] = serde_json::json!(12);
    assert!(!validator.is_valid(&malformed_subject));
}

#[test]
fn schema_validator_is_exact_and_has_no_default_reference_resolution() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("jsonschema = { version = \"=0.58.6\", default-features = false }"));
    let lock = include_str!("../Cargo.lock");
    assert!(lock.contains("name = \"jsonschema\"\nversion = \"0.58.6\""));
}

fn valid_receipt() -> serde_json::Value {
    let hash = |nibble: char| nibble.to_string().repeat(64);
    let mise_config = serde_json::json!({
        "present": false,
        "path": null,
        "sha256": null
    });
    let subject = |role: &str, commit_nibble: char, hash_nibble: char| {
        let commit = commit_nibble.to_string().repeat(40);
        let build_hash = hash(hash_nibble);
        serde_json::json!({
            "role": role,
            "source_commit": commit,
            "build": {
                "package_id": "holla 0.1.0",
                "target_name": "holla",
                "features": [],
                "default_features": true,
                "target_triple": "aarch64-apple-darwin",
                "toolchain": "1.98.1",
                "profile": "release"
            },
            "build_inputs": {
                "manifest_sha256": build_hash,
                "lock_sha256": hash('b'),
                "mise_config": mise_config,
                "cargo_version": "cargo 1.98.1",
                "rustc_version": "rustc 1.98.1",
                "host_triple": "aarch64-apple-darwin",
                "executed_argv": ["cargo", "build", "--release"]
            },
            "builder_receipt": {
                "schema": "termrock-spec/parity-subject-build-receipt-v1",
                "source_commit": commit,
                "package_id": "holla 0.1.0",
                "target_name": "holla",
                "requested_features": [],
                "default_features": true,
                "target_triple": "aarch64-apple-darwin",
                "toolchain": "1.98.1",
                "profile": "release",
                "manifest_sha256": build_hash,
                "lock_sha256": hash('b'),
                "mise_config": mise_config,
                "cargo_version": "cargo 1.98.1",
                "rustc_version": "rustc 1.98.1",
                "host_triple": "aarch64-apple-darwin",
                "executed_argv": ["cargo", "build", "--release"],
                "executable_path": "/tmp/holla",
                "executable_sha256": hash('e'),
                "sha256": hash('f')
            },
            "builder_receipt_sha256": hash('f'),
            "executable": {
                "path": "/tmp/holla",
                "expected_sha256": hash('e'),
                "actual_sha256": null
            },
            "actual_output_root": format!("/tmp/{role}")
        })
    };

    serde_json::json!({
        "schema": "termrock-spec/parity-run-receipt-v1",
        "run_id": "contract-test",
        "source_pair": [subject("reference", '1', 'a'), subject("candidate", '2', 'b')],
        "suite": {
            "revision": "termrock-e2e-2026-10-08.1",
            "digest": hash('1'),
            "compiled_digest": hash('2'),
            "case_set_digest": hash('3'),
            "profile_digest": hash('4'),
            "test_binary_digest": hash('5'),
            "dependency_lock_sha256": hash('6'),
            "platform": "macos"
        },
        "expected_generation": {
            "id": "shared-parity-v1",
            "expected_sha256": hash('7'),
            "actual_sha256": hash('7'),
            "state": "HASH_MATCH",
            "admission_receipt_sha256": null,
            "root": null
        },
        "trust": {
            "status": "unverified",
            "record_path": "/tmp/trust-record.json",
            "record_sha256": hash('8'),
            "receipt_sha256": hash('9'),
            "write_policy_sha256": hash('c'),
            "reason": "fixture only"
        },
        "write_policy": {
            "schema": "termrock-spec/parity-write-policy-v1",
            "write_root": "/tmp/parity-run",
            "receipt_path": "/tmp/parity-run/receipt.json",
            "protected_roots": [
                {"kind": "oracle", "role": null, "path": "/tmp/oracle"}
            ],
            "actual_output_roots": [
                {"role": "reference", "path": "/tmp/reference"},
                {"role": "candidate", "path": "/tmp/candidate"}
            ],
            "sha256": hash('c')
        },
        "checks": [{
            "id": "reference:HELP-HOLLA-004:00-boot:build",
            "subject_role": "reference",
            "case_id": "HELP-HOLLA-004",
            "checkpoint_id": "00-boot",
            "dimension": "build",
            "status": "PASS",
            "reason": "fixture only",
            "evidence": []
        }],
        "artifacts": [{
            "subject_role": "reference",
            "case_id": "HELP-HOLLA-004",
            "checkpoint_id": "01-help",
            "format": "manifest_json",
            "path": "/tmp/reference/manifest.json",
            "sha256": hash('a'),
            "bytes": 64
        }]
    })
}

fn valid_trust_record() -> serde_json::Value {
    let hash = |nibble: char| nibble.to_string().repeat(64);
    let commit = |nibble: char| nibble.to_string().repeat(40);
    serde_json::json!({
        "schema": "termrock-spec/parity-trust-record-v1",
        "suite": {
            "digest": hash('1'),
            "case_set_digest": hash('2'),
            "profile_digest": hash('3'),
            "dependency_lock_sha256": hash('4'),
            "review_receipt_sha256": hash('5'),
            "test_binary_sha256": [hash('6')]
        },
        "subjects": [
            {"role": "reference", "source_commit": commit('a'), "builder_receipt_sha256": hash('7')},
            {"role": "candidate", "source_commit": commit('b'), "builder_receipt_sha256": hash('8')}
        ],
        "expected_generation": null,
        "write_policy_sha256": hash('9')
    })
}

fn subject(role: &str, source_nibble: char) -> Subject {
    let source_commit = source_nibble.to_string().repeat(40);
    let build = BuildFacts {
        package_id: "holla 0.1.0".to_string(),
        target_name: "holla".to_string(),
        features: Vec::new(),
        default_features: true,
        target_triple: "aarch64-apple-darwin".to_string(),
        toolchain: "1.98.1".to_string(),
        profile: "release".to_string(),
    };
    let build_inputs = BuildInputs {
        manifest_sha256: "d".repeat(64),
        lock_sha256: "e".repeat(64),
        mise_config: MiseConfigInput {
            present: false,
            path: None,
            sha256: None,
        },
        cargo_version: "cargo 1.98.1".to_string(),
        rustc_version: "rustc 1.98.1".to_string(),
        host_triple: "aarch64-apple-darwin".to_string(),
        executed_argv: vec![
            "cargo".to_string(),
            "build".to_string(),
            "--release".to_string(),
        ],
    };
    let executable = Executable {
        path: std::path::PathBuf::from("/tmp/holla"),
        sha256: "c".repeat(64),
    };
    let mut builder_receipt = BuilderReceipt {
        schema: "termrock-spec/parity-subject-build-receipt-v1".to_string(),
        source_commit: source_commit.clone(),
        package_id: build.package_id.clone(),
        target_name: build.target_name.clone(),
        requested_features: build.features.clone(),
        default_features: build.default_features,
        target_triple: build.target_triple.clone(),
        toolchain: build.toolchain.clone(),
        profile: build.profile.clone(),
        manifest_sha256: build_inputs.manifest_sha256.clone(),
        lock_sha256: build_inputs.lock_sha256.clone(),
        mise_config: build_inputs.mise_config.clone(),
        cargo_version: build_inputs.cargo_version.clone(),
        rustc_version: build_inputs.rustc_version.clone(),
        host_triple: build_inputs.host_triple.clone(),
        executed_argv: build_inputs.executed_argv.clone(),
        executable_path: executable.path.clone(),
        executable_sha256: executable.sha256.clone(),
        sha256: String::new(),
    };
    builder_receipt.sha256 =
        builder_receipt_digest(&builder_receipt).expect("builder receipt hashes");
    Subject {
        role: role.to_string(),
        source_commit,
        build,
        executable,
        actual_output_root: std::path::PathBuf::from(format!("/tmp/{role}")),
        build_inputs,
        builder_receipt_sha256: builder_receipt.sha256.clone(),
        builder_receipt,
    }
}
