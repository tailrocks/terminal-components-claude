# TablePro new-connection draft

Status: replacement for the candidate clone-default behavior; focused regression verified on an isolated copy of PR #38 source `b8f2ae519a122f98addb6832e65557c9e3e43208`.

The frozen `ConnForm::new(None, None)` starts a new connection with blank name, database, and user; PostgreSQL engine, `localhost`, and port `5432` remain the driver defaults. The candidate's `TableProApp::begin_connection_form` instead copied the active “Local PostgreSQL” name, `acme_dev` database, and `postgres` user. Its prior helper named `empty_new_connection_draft` encoded that copied state, so the existing checks accepted the regression.

`ConnectionDraft::new` now owns new-connection defaults. `ConnectionDraft::from_connection` remains the separate existing-connection prefill path. `begin_connection_form` uses the former; close/cancel behavior and active connection ownership remain unchanged.

Regression evidence on the isolated source copy:

- Before production change, `cargo nextest run -p tablepro-ui --lib new_connection_starts_with_blank_identity_fields_and_driver_defaults --no-fail-fast`: 1 run, 0 passed, 1 failed. First failure observed `Local PostgreSQL` where blank name was required.
- After production change, the focused public `Harness` regressions for blank identity fields and driver defaults, BackTab focus preservation, and typing replacement: 3 run, 3 passed, 0 failed, 23 filtered.
- `cargo fmt --check`: passed.
- Scoped strict Clippy (`cargo clippy -p tablepro-domain -p tablepro-ui --all-targets -- -D warnings`): blocked by existing `clippy::too_many_arguments` in `crates/termrock-feedback/src/settings_page.rs:509` (`paint_field_parts`, 8 arguments). Clippy did not reach the changed packages.
- Logs: `/private/tmp/tablepro-blank-before-red-20261021.log`, `/private/tmp/tablepro-blank-after-green-final-20261021.log`, `/private/tmp/tablepro-blank-clippy-20261021.log`.

This proves only the domain initialization contract. It does not prove New connection layout, full application behavior, or frozen CLI parity. No expected output or artifact changed.
