use termrock_e2e::{
    prepare_frozen_tag_holla_preflight, prepare_holla_help_overlay_preflight,
    receipt_has_blocking_result, run_case,
};

/// Resolve the policy for an external trust record that has not been written.
/// This test does not load trust, launch a product, create a capture, or admit
/// expected data. Run this exact ignored test separately from acceptance.
#[test]
#[ignore = "read-only preparation for an independently created trust record"]
fn holla_write_policy_preflight() {
    let prepared = prepare_holla_help_overlay_preflight().expect("prepare the paired Holla policy");
    let json = serde_json::to_string(&prepared).expect("serialize preflight record");
    println!("TERMROCK_E2E_WRITE_POLICY_PREFLIGHT={json}");
}

/// Run with the same subject manifest and executable case on both pinned
/// Holla binaries. This remains ignored until the paired launcher is ready.
#[test]
#[ignore = "requires source-pinned reference and candidate binaries plus a shared expected generation"]
fn holla_help_overlay() {
    let receipt = run_case("HELP-HOLLA-004").expect("write the paired Holla receipt");
    assert!(
        !receipt.checks.is_empty(),
        "a run without check receipts cannot pass"
    );
    if receipt_has_blocking_result(&receipt) {
        let blocked = receipt
            .checks
            .iter()
            .filter(|check| {
                matches!(
                    check.status.as_str(),
                    "FAIL" | "ERROR" | "BLOCKED" | "NOT_RUN" | "STALE"
                )
            })
            .map(|check| format!("{}={} ({})", check.id, check.status, check.reason))
            .collect::<Vec<_>>()
            .join("\n");
        panic!("HELP-HOLLA-004 is not acceptance-ready:\n{blocked}");
    }
}

/// Validate immutable-tag source/build evidence and output policy without launching
/// the tag binary. The helper's source closure is not an independent execution
/// anchor, so this preflight must remain blocked and capture/admission stay NOT_RUN.
#[test]
#[ignore = "requires the pinned VIS-06 validator, immutable-tag builder receipt, and isolated output roots"]
fn holla_help_overlay_frozen_tag_capture_preflight() {
    let prepared = prepare_frozen_tag_holla_preflight("HELP-HOLLA-004")
        .expect("validate immutable-tag source evidence and prepare its output policy");
    let json = serde_json::to_value(&prepared).expect("serialize read-only tag preflight");
    assert_eq!(
        json["schema"],
        "termrock-spec/parity-oracle-capture-preflight-v1"
    );
    assert_eq!(json["state"], "blocked");
    assert_eq!(json["execution_anchor_status"], "unverified");
    assert_eq!(json["qualification"]["status"], "blocked");
    assert_eq!(json["capture_status"], "NOT_RUN");
    assert_eq!(json["admission_status"], "NOT_RUN");
    assert_eq!(
        json["tag_identity"]["schema"],
        "termrock-spec/visual-tag-holla-validated-identity-v1"
    );
    assert_eq!(
        json["tag_identity"]["oracle_lineage"]["tag_object"],
        "1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5"
    );
    assert_eq!(
        json["tag_identity"]["oracle_lineage"]["tag_commit"],
        "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"
    );
    assert!(
        json["validator_execution"]["git_dir"]
            .as_str()
            .is_some_and(|path| path.starts_with('/'))
    );
    assert!(
        json["validator_execution"]["git_common_dir"]
            .as_str()
            .is_some_and(|path| path.starts_with('/'))
    );
    assert_eq!(json["case"]["checkpoints"].as_array().unwrap().len(), 4);
    assert_eq!(
        json["case"]["checkpoints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|checkpoint| checkpoint["assertions"].as_array().unwrap().len())
            .sum::<usize>(),
        12
    );
    println!("TERMROCK_E2E_TAG_PREFLIGHT={}", json);
}
