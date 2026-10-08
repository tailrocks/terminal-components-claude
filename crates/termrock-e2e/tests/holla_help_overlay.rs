use termrock_e2e::{receipt_has_blocking_result, run_case};

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
