# TablePro blank database validation

Status: bounded domain correction prepared on WIP branch `wip/tablepro-blank-database-validation-20261021`. Source base `4f40bb294da89bbfcb2a6716c88a3158427aa0b2`; prerequisite `f0760cc204afc533e202604d0437f913b72bfd10` cherry-picked separately as `31a7049d4`.

Frozen `src/bin/tablepro/connections.rs` validates only Name and Port in `ConnForm::validate` (baseline `4169783ae883788186d161af09ba9c481bd84e5c`, lines 301–305). Candidate `ConnectionDraft::validate_all` had an additional PostgreSQL-only required Database check. Removed that rule; retained required nonblank Name and existing Port validation. Added domain regression cases for valid Name/Port with blank PostgreSQL Database and for invalid Name and Port.

The reconnect-guard test now types `Unsaved guard draft` into the visible Name field before Save & Connect. Database remains blank. Existing reconnect, cancel, active-tab, pending-edit, and diagnostics assertions remain intact. `footer_unicode.rs` remains unchanged.

Checks on this source:

- `cargo nextest run --locked -p tablepro-domain --all-targets --no-fail-fast --status-level fail --final-status-level fail`: 14 passed, 0 failed, 0 skipped.
- `cargo nextest run --locked -p tablepro-ui --test replacement_contract --test footer_unicode --no-fail-fast --status-level fail --final-status-level fail`: 7 passed, 0 failed, 0 skipped.
- Independent external consumer probe copied from review packet `tablepro-blank-domain-validation-e404-20261021/review.json` (SHA-256 `897c7015eb34303bd6054cdccce32a4ee8aca3266a885837d682d108a4b07d6a`), paths redirected to this worktree: 4 passed, 0 failed, 0 skipped.
- Combined `cargo nextest run --locked -p tablepro-domain -p tablepro-ui --all-targets --no-fail-fast --status-level fail --final-status-level fail`: 209 passed, 1 failed, 0 skipped. Remaining `tablepro-ui::form_radio::form_radio_enter_submits` fixture still assumes Production/Safe values in a blank New form. Separate radio fixture/product contract work remains open; this correction does not bless that behavior.
- `cargo fmt --check` is blocked by pre-existing formatting in incoming `crates/jackin-preview-app/src/app.rs:261`; the two owned Rust files were formatted directly with `rustfmt --edition 2024`.

Separate source-established discrepancies remain open: candidate Port accepts `0`, while frozen TablePro accepts only empty or `1..=65535`; candidate `to_connection` does not trim Name like the frozen implementation. Neither was changed here. This source correction and its widget tests do not establish real-CLI or `tuiscotti` parity.

## Strict Clippy test cleanup

At parent integration checkpoint `0354d2d0640b228b5ec9c4613fad13a8a7535a72`, strict Clippy for `tablepro-domain` and `tablepro-ui` exposed only test-code findings in this increment: an emptiness assertion, two `unwrap_err` calls, and one `expect` for the active draft. Replaced these with value equality, structural error-field matching, and a fallible test result. No production behavior or expected output changed.

`rustfmt --check --edition 2024` passes for both edited files. Focused Nextest cases pass: `name_and_port_remain_required_validation_fields` (1/1) and `new_connection_starts_with_blank_identity_fields_and_driver_defaults` (1/1). Strict `cargo clippy -p tablepro-domain -p tablepro-ui --all-targets --no-deps -- -Dwarnings` passes. The full package Nextest run reports 209 passed, 1 failed, 0 skipped; the remaining `tablepro-ui::form_radio::form_radio_enter_submits` fixture is the separately tracked radio mismatch and is not cleared by this lint-only change. Workspace formatting remains blocked by incoming `crates/jackin-preview-app/src/app.rs:261`.
