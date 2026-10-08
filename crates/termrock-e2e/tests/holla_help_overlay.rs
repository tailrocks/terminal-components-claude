use termrock_e2e::{prepare_holla_help_overlay_preflight, receipt_has_blocking_result, run_case};

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
