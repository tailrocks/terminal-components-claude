use termrock_e2e::{ORACLE_CAPTURE_RECEIPT_SCHEMA, run_tag_capture};

/// This explicit ignored lane launches only the immutable-tag Holla binary.
/// It records actual captures but cannot admit an expected generation.
#[test]
#[ignore = "requires an independently built immutable-tag Holla binary and isolated capture roots"]
fn captures_holla_from_pinned_immutable_tag() {
    let receipt =
        run_tag_capture("HELP-HOLLA-004").expect("capture the real immutable-tag Holla case");
    assert_eq!(receipt.schema, ORACLE_CAPTURE_RECEIPT_SCHEMA);
    assert_eq!(
        receipt.oracle_identity.tag_commit,
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    );
    assert_eq!(receipt.capture_status, "COMPLETE");
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
