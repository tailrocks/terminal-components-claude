use termrock_e2e::{receipt_has_blocking_result, run_case};

fn run_registered_case(case_id: &str) {
    let receipt = run_case(case_id).unwrap_or_else(|error| {
        panic!("run paired real-binary case {case_id}: {error}");
    });
    assert!(
        !receipt.checks.is_empty(),
        "case {case_id} returned no checks"
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
        panic!("case {case_id} is not acceptance-ready:\n{blocked}");
    }
}

macro_rules! shared_case_test {
    ($test_name:ident, $case_id:literal) => {
        #[test]
        #[ignore = "requires a pinned subject pair; visual acceptance stays blocked while expected_generation is null"]
        fn $test_name() {
            run_registered_case($case_id);
        }
    };
}

shared_case_test!(holla_help_overlay_004, "HELP-HOLLA-004");
shared_case_test!(
    jackin_editor_save_cancel_120x40_truecolor,
    "JACKIN-EDITOR-SAVE-CANCEL-120X40-TRUECOLOR"
);
shared_case_test!(showcase_dialog_001, "SHOWCASE-DIALOG-001");
shared_case_test!(
    tablepro_table_001_120x40_truecolor,
    "TABLEPRO-TABLE-001-120X40-TRUECOLOR"
);
