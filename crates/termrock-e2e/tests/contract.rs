use std::fs;
use termrock_e2e::{
    BuildFacts, BuildInputs, BuilderReceipt, EvidenceReference, Executable, MiseConfigInput,
    ProtectedRoot, SUBJECT_SCHEMA, Subject, SubjectBuildEvidence, SubjectManifest, TrustRecord,
    builder_receipt_digest, deferred_row_count, registry, suite_digest, validate_subject_manifest,
};

#[test]
fn registry_contains_four_seed_cases_and_holla_checkpointed_preconditions() {
    let registry = registry().expect("registry parses");
    assert_eq!(registry.schema, "termrock-e2e/case-registry-v1");
    assert_eq!(registry.suite_revision, "termrock-e2e-2026-10-09.1");

    let expected_ids = std::collections::BTreeSet::from([
        "HELP-HOLLA-004",
        "JACKIN-EDITOR-SAVE-CANCEL-120X40-TRUECOLOR",
        "SHOWCASE-DIALOG-001",
        "TABLEPRO-TABLE-001-120X40-TRUECOLOR",
    ]);
    let actual_ids = registry
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(registry.cases.len(), expected_ids.len());
    assert_eq!(actual_ids, expected_ids);
    for case in &registry.cases {
        termrock_e2e::validate_case_contract(case)
            .unwrap_or_else(|error| panic!("case {} is invalid: {error}", case.id));
    }

    let case = registry
        .cases
        .iter()
        .find(|case| case.id == "HELP-HOLLA-004")
        .expect("Holla pilot remains registered");
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
        expected_generation: None,
        build_evidence: test_build_evidence(),
        subjects: vec![subject("reference", 'a'), subject("candidate", 'b')],
    };
    #[cfg(unix)]
    assert!(validate_subject_manifest(&manifest).is_ok());
    #[cfg(not(unix))]
    assert!(
        validate_subject_manifest(&manifest)
            .unwrap_err()
            .contains("physical directory identity is unsupported")
    );
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/subject-manifest-v2.schema.json"))
            .expect("subject manifest schema parses");
    let validator = jsonschema::draft202012::new(&schema).expect("subject schema compiles");
    let manifest_json = serde_json::to_value(&manifest).expect("serialize manifest fixture");
    assert!(
        validator.is_valid(&manifest_json),
        "v2 manifest rejected: {manifest_json}"
    );
    assert!(serde_json::from_value::<SubjectManifest>(manifest_json.clone()).is_ok());
    let mut manifest_without_generation = manifest_json.clone();
    manifest_without_generation
        .as_object_mut()
        .expect("manifest object")
        .remove("expected_generation");
    assert!(!validator.is_valid(&manifest_without_generation));
    assert!(serde_json::from_value::<SubjectManifest>(manifest_without_generation).is_err());
    let evidence_keys = manifest_json["build_evidence"]
        .as_object()
        .expect("manifest build evidence object")
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        evidence_keys,
        std::collections::BTreeSet::from([
            "build_environment".to_string(),
            "source_inputs".to_string()
        ])
    );
    assert!(manifest_json["build_evidence"].get("builder_run").is_none());

    let mut legacy_manifest = manifest_json.clone();
    legacy_manifest["schema"] = serde_json::json!("termrock-spec/parity-subject-manifest-v1");
    legacy_manifest["expected_generation"] = serde_json::json!({
        "id": "legacy-fixture",
        "sha256": "2".repeat(64)
    });
    legacy_manifest
        .as_object_mut()
        .expect("legacy subject manifest object")
        .remove("build_evidence");
    let legacy_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/subject-manifest-v1.schema.json"))
            .expect("historical v1 subject schema parses");
    let legacy_validator =
        jsonschema::draft202012::new(&legacy_schema).expect("historical v1 schema compiles");
    assert!(legacy_validator.is_valid(&legacy_manifest));
    assert!(!validator.is_valid(&legacy_manifest));
    assert!(serde_json::from_value::<SubjectManifest>(legacy_manifest).is_err());

    let mut unsafe_sidecar = manifest.clone();
    unsafe_sidecar.build_evidence.source_inputs.path = "../oracle/source-inputs.json".to_string();
    assert!(validate_subject_manifest(&unsafe_sidecar).is_err());

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
        expected_generation: None,
        build_evidence: test_build_evidence(),
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
        expected_generation: None,
        build_evidence: test_build_evidence(),
        subjects: vec![subject("reference", 'a'), subject("candidate", 'b')],
    };
    mismatched_builder_receipt.subjects[0].builder_receipt_sha256 = "0".repeat(64);
    assert!(validate_subject_manifest(&mismatched_builder_receipt).is_err());
}

#[test]
fn schema_required_nullable_fields_reject_omission_but_accept_explicit_null() {
    let trust_record = serde_json::json!({
        "schema": "termrock-spec/parity-trust-record-v2",
        "suite": {
            "digest": "4".repeat(64),
            "case_set_digest": "5".repeat(64),
            "profile_digest": "6".repeat(64),
            "dependency_lock_sha256": "7".repeat(64),
            "review_receipt_sha256": "8".repeat(64),
            "test_binary_sha256": ["c".repeat(64)]
        },
        "subjects": [
            {
                "role": "reference",
                "source_commit": "d".repeat(40),
                "builder_receipt_sha256": "e".repeat(64)
            },
            {
                "role": "candidate",
                "source_commit": "f".repeat(40),
                "builder_receipt_sha256": "0".repeat(64)
            }
        ],
        "expected_generation": null,
        "build_evidence": {
            "subject_manifest": {"path": "/run/subject-manifest.json", "sha256": "9".repeat(64)},
            "builder_run": {"path": "/run/run.json", "sha256": "a".repeat(64)}
        },
        "write_policy_sha256": "b".repeat(64)
    });
    let trust_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/trust-record-v2.schema.json"))
            .expect("trust schema parses");
    let trust_validator =
        jsonschema::draft202012::new(&trust_schema).expect("trust schema compiles");
    assert!(trust_validator.is_valid(&trust_record));
    assert!(serde_json::from_value::<TrustRecord>(trust_record.clone()).is_ok());
    let mut trust_without_generation = trust_record;
    trust_without_generation
        .as_object_mut()
        .expect("trust record object")
        .remove("expected_generation");
    assert!(!trust_validator.is_valid(&trust_without_generation));
    assert!(serde_json::from_value::<TrustRecord>(trust_without_generation).is_err());

    let mise_nulls = serde_json::json!({"present": false, "path": null, "sha256": null});
    let manifest_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/subject-manifest-v2.schema.json"))
            .expect("manifest schema parses");
    let mise_schema = manifest_schema["$defs"]["mise_config"].clone();
    let mise_validator =
        jsonschema::draft202012::new(&mise_schema).expect("mise config schema compiles");
    assert!(mise_validator.is_valid(&mise_nulls));
    assert!(serde_json::from_value::<MiseConfigInput>(mise_nulls.clone()).is_ok());
    for field in ["path", "sha256"] {
        let mut missing = mise_nulls.clone();
        missing
            .as_object_mut()
            .expect("mise config object")
            .remove(field);
        assert!(!mise_validator.is_valid(&missing));
        assert!(
            serde_json::from_value::<MiseConfigInput>(missing).is_err(),
            "missing mise_config.{field} must be rejected"
        );
    }

    let oracle_root = serde_json::json!({"kind": "oracle", "role": null, "path": "/oracle"});
    let receipt_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/receipt-v2.schema.json"))
            .expect("receipt schema parses");
    let protected_root_schema = receipt_schema["$defs"]["protected_root"].clone();
    let protected_root_validator = jsonschema::draft202012::new(&protected_root_schema)
        .expect("protected-root schema compiles");
    assert!(protected_root_validator.is_valid(&oracle_root));
    assert!(serde_json::from_value::<ProtectedRoot>(oracle_root.clone()).is_ok());
    let mut oracle_without_role = oracle_root;
    oracle_without_role
        .as_object_mut()
        .expect("oracle root object")
        .remove("role");
    assert!(!protected_root_validator.is_valid(&oracle_without_role));
    assert!(serde_json::from_value::<ProtectedRoot>(oracle_without_role).is_err());
}

#[test]
fn receipt_schema_validates_complete_record_and_rejects_malformed_nested_data() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/receipt-v2.schema.json"))
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
fn receipt_v3_requires_admitted_exact_evidence_for_visual_verdicts() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/receipt-v3.schema.json"))
            .expect("v3 receipt schema parses");
    assert!(jsonschema::draft202012::meta::is_valid(&schema));
    let validator = jsonschema::draft202012::new(&schema).expect("v3 schema compiles");

    let mut blocked = valid_receipt();
    blocked["schema"] = serde_json::json!("termrock-spec/parity-run-receipt-v3");
    blocked["checks"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "reference:HELP-HOLLA-004:00-boot:visual",
            "subject_role": "reference",
            "case_id": "HELP-HOLLA-004",
            "checkpoint_id": "00-boot",
            "dimension": "visual",
            "status": "BLOCKED",
            "reason": "expected content is unadmitted",
            "evidence": []
        }));
    assert!(
        validator.is_valid(&blocked),
        "blocked v3 receipt rejected: {blocked}"
    );

    let mut blocked_with_evidence = blocked.clone();
    blocked_with_evidence["checks"][1]["visual_comparison"] = visual_comparison_fixture(true);
    assert!(!validator.is_valid(&blocked_with_evidence));

    let mut pass = blocked.clone();
    pass["checks"][1]["status"] = serde_json::json!("PASS");
    pass["checks"][1]["visual_comparison"] = visual_comparison_fixture(true);
    assert!(
        validator.is_valid(&pass),
        "valid exact visual verdict rejected: {pass}"
    );

    let mut pass_without_evidence = pass.clone();
    pass_without_evidence["checks"][1]
        .as_object_mut()
        .unwrap()
        .remove("visual_comparison");
    assert!(!validator.is_valid(&pass_without_evidence));

    let mut unverified_pass = pass.clone();
    unverified_pass["checks"][1]["visual_comparison"]["admission"]["verified"] =
        serde_json::json!(false);
    assert!(!validator.is_valid(&unverified_pass));

    let mut inexact_pass = pass.clone();
    inexact_pass["checks"][1]["visual_comparison"]["png"]["pixels_equal"] =
        serde_json::json!(false);
    assert!(!validator.is_valid(&inexact_pass));

    let mut differing_cells_pass = pass.clone();
    differing_cells_pass["checks"][1]["visual_comparison"]["frame"]["cells_equal"] =
        serde_json::json!(false);
    assert!(!validator.is_valid(&differing_cells_pass));

    let mut approximate_pass = pass.clone();
    approximate_pass["checks"][1]["visual_comparison"]["fidelity"]["actual"]["approximate"] =
        serde_json::json!(true);
    assert!(!validator.is_valid(&approximate_pass));

    let mut fail = blocked.clone();
    fail["checks"][1]["status"] = serde_json::json!("FAIL");
    fail["checks"][1]["visual_comparison"] = visual_comparison_fixture(false);
    assert!(
        validator.is_valid(&fail),
        "exact mismatch receipt rejected: {fail}"
    );

    let mut false_fail = fail.clone();
    false_fail["checks"][1]["visual_comparison"] = visual_comparison_fixture(true);
    assert!(!validator.is_valid(&false_fail));

    let mut approximate_fail = fail.clone();
    approximate_fail["checks"][1]["visual_comparison"]["fidelity"]["actual"]["approximate"] =
        serde_json::json!(true);
    assert!(!validator.is_valid(&approximate_fail));

    let v2_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/receipt-v2.schema.json"))
            .expect("historical v2 schema parses");
    let v2_validator =
        jsonschema::draft202012::new(&v2_schema).expect("historical v2 schema compiles");
    assert!(
        !v2_validator.is_valid(&pass),
        "v2 must not accept v3 receipt data"
    );
}

#[test]
fn expected_generation_v1_schema_is_closed_and_pins_checkpoint_files() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../schemas/expected-generation-v1.schema.json"
    ))
    .expect("expected-generation schema parses");
    assert!(jsonschema::draft202012::meta::is_valid(&schema));
    let validator =
        jsonschema::draft202012::new(&schema).expect("expected-generation schema compiles");
    let manifest = valid_expected_generation_manifest();
    assert!(
        validator.is_valid(&manifest),
        "valid expected-generation fixture rejected: {manifest}"
    );

    let mut missing = manifest.clone();
    missing.as_object_mut().unwrap().remove("oracle");
    assert!(!validator.is_valid(&missing));

    let mut unexpected = manifest.clone();
    unexpected["unreviewed"] = serde_json::json!(true);
    assert!(!validator.is_valid(&unexpected));

    let mut duplicate_checkpoint = manifest.clone();
    duplicate_checkpoint["checkpoints"][1]["id"] = serde_json::json!("00-boot");
    assert!(!validator.is_valid(&duplicate_checkpoint));

    let mut moved_checkpoint_file = manifest;
    moved_checkpoint_file["checkpoints"][0]["png"]["path"] =
        serde_json::json!("checkpoints/00-boot/other.png");
    assert!(!validator.is_valid(&moved_checkpoint_file));
}

#[test]
fn trust_record_schema_requires_the_bound_write_policy_digest() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/trust-record-v2.schema.json"))
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
        "schema": "termrock-spec/parity-run-receipt-v2",
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
        "expected_generation": null,
        "build_evidence": {
            "subject_manifest": {"path": "/tmp/run/subject-manifest.json", "sha256": hash('d')},
            "source_inputs": {"path": "/tmp/run/source-inputs.json", "sha256": hash('e')},
            "build_environment": {"path": "/tmp/run/build-environment.json", "sha256": hash('f')},
            "builder_run": {"path": "/tmp/run/run.json", "sha256": hash('a')}
        },
        "environment": {
            "cleared": true,
            "cwd": "/repo",
            "common": {"TERM": "xterm-256color", "COLORTERM": "truecolor", "LC_ALL": "C.UTF-8", "SHELL": "/bin/sh"},
            "case_allowlist": ["HOLLA_NO_HISTORY"],
            "case_values": {"HOLLA_NO_HISTORY": "1"}
        },
        "renderer": {
            "schema": "termrock-spec/tuiscotti-renderer-identity-v1",
            "hash": hash('d'),
            "name": "tuiscotti-default",
            "renderer_version": 1,
            "font_px": 16.0,
            "cell_w": 10,
            "cell_h": 21,
            "pad": 12,
            "scale": 2,
            "default_fg": [208, 208, 208],
            "default_bg": [0, 0, 0],
            "indexed_palette": "Xterm",
            "face_hashes": [hash('1'), hash('2'), hash('3'), hash('4')],
            "fallback_order": [
                {"description": "Symbols2", "sha256": hash('5')},
                {"description": "Symbols", "sha256": hash('6')},
                {"description": "CJK", "sha256": hash('7')}
            ],
            "cursor_policy": "Show",
            "blink_phase": "On",
            "missing_glyph_policy": "Strict"
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
            "schema": "termrock-spec/parity-write-policy-v2",
            "write_root": "/tmp/parity-run",
            "receipt_path": "/tmp/parity-run/receipt.json",
            "subject_cwd": "/repo",
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

fn visual_comparison_fixture(equal: bool) -> serde_json::Value {
    let hash = |nibble: char| nibble.to_string().repeat(64);
    let artifact = |path: &str, nibble: char| serde_json::json!({"path": path, "sha256": hash(nibble), "bytes": 16});
    let fidelity = |frame_nibble: char, approximate: bool| {
        serde_json::json!({
            "sha256": hash('7'),
            "bytes": 20,
            "source_frame_sha256": hash(frame_nibble),
            "renderer_sha256": hash('8'),
            "rerender_png_sha256": hash('9'),
            "approximate": approximate,
            "rerender_matches_bound_png": !approximate
        })
    };
    serde_json::json!({
        "schema": "termrock-spec/parity-visual-comparison-v1",
        "method": "frame-v3-diff-cells+opaque-rgb-decoded-exact-v1",
        "admission": {
            "verified": true,
            "trust_record_sha256": hash('a'),
            "expected_generation_id": "expected-1",
            "expected_tree_sha256": hash('b'),
            "expected_manifest_sha256": hash('c'),
            "admission_receipt_sha256": hash('d')
        },
        "renderer": {"expected_sha256": hash('8'), "actual_sha256": hash('8'), "equal": true},
        "frame": {
            "expected": artifact("expected/frame.json", 'e'),
            "actual": artifact("actual/frame.json", 'f'),
            "version": 3,
            "expected_geometry": {"cols": 120, "rows": 40},
            "actual_geometry": {"cols": 120, "rows": 40},
            "dimensions_equal": true,
            "cells_equal": equal,
            "cursor_equal": true,
            "differing_positions": if equal { vec![] } else { vec![serde_json::json!({"x": 1, "y": 2})] },
            "equal": equal
        },
        "png": {
            "expected": artifact("expected/screen.png", '1'),
            "actual": artifact("actual/screen.png", '2'),
            "expected_info": {"width": 1448, "height": 888, "color_type": 2, "bit_depth": 8},
            "actual_info": {"width": 1448, "height": 888, "color_type": 2, "bit_depth": 8},
            "alpha_policy": "opaque",
            "dimensions_equal": true,
            "pixels_equal": equal,
            "equal": equal
        },
        "fidelity": {"expected": fidelity('e', false), "actual": fidelity('f', false)}
    })
}

fn valid_expected_generation_manifest() -> serde_json::Value {
    let hash = |nibble: char| nibble.to_string().repeat(64);
    let checkpoint = |id: &str| {
        serde_json::json!({
            "id": id,
            "observation_id": format!("observation-{id}"),
            "geometry": {"cols": 120, "rows": 40},
            "color_path": "truecolor",
            "frame": {
                "path": format!("checkpoints/{id}/frame.json"),
                "sha256": hash('1'),
                "bytes": 4096
            },
            "png": {
                "path": format!("checkpoints/{id}/screen.png"),
                "sha256": hash('2'),
                "bytes": 512,
                "width": 2448,
                "height": 1728,
                "color_type": 2,
                "bit_depth": 8
            }
        })
    };
    let renderer = valid_receipt()["renderer"].clone();
    serde_json::json!({
        "schema": "termrock-spec/parity-expected-generation-v1",
        "id": "expected-holla-help-1",
        "case_id": "HELP-HOLLA-004",
        "case_set_digest": hash('3'),
        "profile_digest": hash('4'),
        "dependency_lock_sha256": hash('5'),
        "case_input_sha256": hash('6'),
        "tuiscotti_revision": "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274",
        "tree_hash_algorithm": "termrock-e2e/expected-tree-v1",
        "oracle": {
            "tag_ref": "refs/tags/visual-baseline",
            "tag_object": "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5",
            "tag_commit": "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b",
            "capture_run_sha256": hash('7'),
            "source_inputs_sha256": hash('8'),
            "build_environment_sha256": hash('9'),
            "builder_receipt_sha256": hash('a'),
            "oracle_executable_sha256": hash('b')
        },
        "renderer": renderer,
        "checkpoints": [
            checkpoint("00-boot"),
            checkpoint("01-help"),
            checkpoint("02-finder"),
            checkpoint("03-help-again")
        ]
    })
}

fn valid_trust_record() -> serde_json::Value {
    let hash = |nibble: char| nibble.to_string().repeat(64);
    let commit = |nibble: char| nibble.to_string().repeat(40);
    serde_json::json!({
        "schema": "termrock-spec/parity-trust-record-v2",
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
        "build_evidence": {
            "subject_manifest": {"path": "/tmp/run/subject-manifest.json", "sha256": hash('a')},
            "builder_run": {"path": "/tmp/run/run.json", "sha256": hash('b')}
        },
        "write_policy_sha256": hash('9')
    })
}

fn test_build_evidence() -> SubjectBuildEvidence {
    SubjectBuildEvidence {
        source_inputs: EvidenceReference {
            path: "source-inputs.json".to_string(),
            sha256: "a".repeat(64),
        },
        build_environment: EvidenceReference {
            path: "build-environment.json".to_string(),
            sha256: "b".repeat(64),
        },
    }
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
    let output_base = fs::canonicalize(std::env::temp_dir())
        .expect("canonicalize contract test temporary directory")
        .join(format!(
            "termrock-e2e-contract-outputs-{}",
            std::process::id()
        ));
    let actual_output_root = output_base.join(role);
    fs::create_dir_all(&actual_output_root).expect("create contract test actual output root");
    let actual_output_root = fs::canonicalize(actual_output_root)
        .expect("canonicalize contract test actual output root");
    Subject {
        role: role.to_string(),
        source_commit,
        build,
        executable,
        actual_output_root,
        build_inputs,
        builder_receipt_sha256: builder_receipt.sha256.clone(),
        builder_receipt,
    }
}

#[test]
fn immutable_tag_capture_receipt_schema_is_separate_and_closed() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../schemas/oracle-capture-receipt-v1.schema.json"
    ))
    .expect("immutable-tag capture schema parses");
    assert!(jsonschema::draft202012::meta::is_valid(&schema));
    let validator =
        jsonschema::draft202012::new(&schema).expect("immutable-tag capture schema compiles");
    assert_eq!(
        schema["$id"],
        serde_json::json!("termrock-spec/parity-oracle-capture-receipt-v1")
    );
    assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    for forbidden in ["source_pair", "expected_generation", "trust"] {
        assert!(schema["properties"].get(forbidden).is_none());
        assert!(!schema["required"]
            .as_array()
            .expect("required fields")
            .iter()
            .any(|field| field.as_str() == Some(forbidden)));
    }
    let validator_execution = &schema["$defs"]["validator_execution"];
    let required = validator_execution["required"]
        .as_array()
        .expect("validator execution required fields");
    for field in ["git_dir", "git_common_dir"] {
        assert!(required.iter().any(|value| value.as_str() == Some(field)));
        assert!(validator_execution["properties"].get(field).is_some());
    }
    assert!(validator.is_valid(&serde_json::json!({})) == false);
}

#[test]
fn generation_admission_and_independent_review_schemas_are_closed() {
    for (path, source, expected_id) in [
        (
            "generation-admission-v1.schema.json",
            include_str!("../schemas/generation-admission-v1.schema.json"),
            "termrock-spec/visual-generation-admission-v1",
        ),
        (
            "qualification-review-v1.schema.json",
            include_str!("../schemas/qualification-review-v1.schema.json"),
            "termrock-spec/visual-oracle-qualification-review-v1",
        ),
    ] {
        let schema: serde_json::Value =
            serde_json::from_str(source).unwrap_or_else(|error| panic!("{path} parses: {error}"));
        assert!(jsonschema::draft202012::meta::is_valid(&schema), "{path}");
        let validator = jsonschema::draft202012::new(&schema)
            .unwrap_or_else(|error| panic!("{path} compiles: {error}"));
        assert_eq!(schema["$id"], serde_json::json!(expected_id));
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
        assert!(!validator.is_valid(&serde_json::json!({})), "{path}");
    }
}
