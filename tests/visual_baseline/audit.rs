//! Audit matrix (250 captures): the 10 audit fixtures × 5 sizes
//! {72x20,80x24,100x30,120x40,160x50} × 5 colours {truecolor,256,16,none,nocolor},
//! generated data-drivenly so every combo exists exactly once
//! (docs/baseline/snapshots-v2.md). These absorb the 57 static audit captures
//! the pre-grouped suite kept in the app modules; static defaults of audit
//! surfaces live ONLY here (the dedupe rule).

use crate::support::{
    self, CANONICAL_COLORS, CANONICAL_SIZES, Case, HOLLA, JACKIN, SHOWCASE, TABLEPRO,
};

const SHOWCASE_BOOT: &str = "Junie Design system";
const HOLLA_BOOT: &str = "holla❯";
const JACKIN_BOOT: &str = "jackin❯";

/// One fixture's full 5×5 sweep as `<group>/<sub_group>/<leaf>/<cols>x<rows>/<color>`.
fn audit_matrix(
    prefix: &str,
    bin: &'static str,
    args: &'static [&'static str],
    needle: &'static str,
) {
    let mut failures = Vec::new();
    for (cols, rows) in CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let name = support::audit_default_name(prefix, cols, rows, color);
            let case = Case::dynamic(name.clone(), bin, args, cols, rows, color, needle);
            if !support::collect_matrix(&name, || support::run_and_assert(&case)) {
                failures.push(name);
            }
        }
    }
    support::finish_matrix(&failures);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_audit_rust_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_HOLLA_RUST,
        HOLLA,
        &[
            "--scenario",
            "rust-dirty",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        HOLLA_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_audit_upgrade_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_HOLLA_UPGRADE,
        HOLLA,
        &[
            "--scenario",
            "upgrade-plan",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        HOLLA_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn jackin_audit_accounts_matrix() {
    // Determinism contract: accounts-mixed never finishes its boot refresh
    // under paused motion (`refreshing 1` → `busy_rows` non-empty), so the
    // Manager's on_tick returns Changed and a repaint lands every 500 ms.
    // The Accounts scroll badge (`· 1–11 of 23`) needs the list viewport,
    // learned on render one, so it first appears in the t≈500 ms repaint —
    // while wait_stable(400 ms) can fire at t≈400, before it. At sizes where
    // the list overflows (verified: 72x20/80x24/100x30 badge, 120x40/160x50
    // none) wait for the badge text itself; the capture then always lands
    // after the badge repaint, whose damage end is the same every run.
    let mut failures = Vec::new();
    for (cols, rows) in CANONICAL_SIZES {
        for color in CANONICAL_COLORS {
            let case = Case::dynamic(
                support::audit_default_name(
                    support::AUDIT_PREFIX_JACKIN_ACCOUNTS,
                    cols,
                    rows,
                    color,
                ),
                JACKIN,
                &[
                    "--scenario",
                    "accounts-mixed",
                    "--motion",
                    "paused",
                    "--frame",
                    "40",
                ],
                cols,
                rows,
                color,
                JACKIN_BOOT,
            );
            let name = case.name.to_string();
            if !support::collect_matrix(&name, || {
                let mut s = support::spawn(&case);
                support::boot(&mut s, JACKIN_BOOT);
                if rows <= 30 {
                    s.wait_for_text("of 23")
                        .unwrap_or_else(|e| panic!("accounts badge never rendered: {e:#}"));
                }
                support::settle_and_gate(&mut s, &case.name);
            }) {
                failures.push(name);
            }
        }
    }
    support::finish_matrix(&failures);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn jackin_audit_capsule_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_JACKIN_CAPSULE,
        JACKIN,
        &[
            "--scenario",
            "capsule-multi",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        JACKIN_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_audit_buttons_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_SHOWCASE_BUTTONS,
        SHOWCASE,
        &["--page", "buttons"],
        SHOWCASE_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_audit_diff_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_SHOWCASE_DIFF,
        SHOWCASE,
        &["--page", "diff"],
        SHOWCASE_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_audit_forms_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_SHOWCASE_FORMS,
        SHOWCASE,
        &["--page", "forms"],
        SHOWCASE_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_audit_inputs_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_SHOWCASE_INPUTS,
        SHOWCASE,
        &["--page", "inputs"],
        SHOWCASE_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_audit_textareas_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_SHOWCASE_TEXTAREAS,
        SHOWCASE,
        &["--page", "textareas"],
        SHOWCASE_BOOT,
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_audit_production_matrix() {
    audit_matrix(
        support::AUDIT_PREFIX_TABLEPRO_PRODUCTION,
        TABLEPRO,
        &["--connect", "Production"],
        "Query 1",
    );
}
