# F02 — Picker action eligibility and destructive target identity

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Fix shared Picker eligibility for primary, secondary and pointer actions. Exercise Holla activity/resource pickers with empty, disabled, loading, filtered and stale results; preserve deliberate query-reset semantics.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

TablePro tab-ID mapping, destructive fallback removal and its full application regression remain deferred. Shared-widget completion alone does not close the full F02 finding.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed defect.** `Picker::on_key`, `src/widgets/picker.rs:153/195`,
guards Enter but emits `Secondary(cursor)` for Delete on empty, disabled and
loading results. `src/bin/tablepro/app.rs:1502` uses display detail as a tab index
and falls back with `unwrap_or(i)`. Actual flow: connected TablePro → Ctrl+G →
unmatched query → Delete closes Query1 despite “No matches”. API probes and an
independent real-terminal design replay both confirm it.

Root: action eligibility differs by input path, and absent identity becomes an
unrelated valid target. Share an eligible-item resolver across primary,
secondary and pointer actions. Keep tab identity in an explicit owner mapping,
not display text. Missing/stale mappings must reject action. Fix both boundaries;
a widget-only guard leaves destructive owner fallback unsafe.

Risk: medium because tab close can discard state. Acceptance: empty, loading,
error and disabled results emit no item action; valid filtered Delete closes
exactly the selected tab. Insertion/removal while the picker remains open must
not retarget a result. Query reset still deliberately selects the first eligible
result; do not impose blanket cursor preservation on filtering.

## Evidence

**Slice status:** current shared/Holla slice complete · TablePro tab-ID mapping and its application regression stay Later.

**Shared widget:** `src/widgets/picker.rs` resolves one eligible row for Enter, Alt+Enter, Delete and pointer activation; an empty, loading, disabled or filtered-out result emits no item action, and `PickerEvent::Submit` reports Enter with a query and no eligible row so an owner can accept free text (Holla's path jump) instead of a stale index. Query reset still selects the first eligible row deliberately. Tests: `picker.rs: actions_only_target_eligible_rows_on_every_path`, `refresh_keeps_identity_and_query_reset_selects_first`, `keyboard_navigation_pulls_the_cursor_back_into_view`, `wheel_at_the_boundary_is_consumed_not_changed`.

**Holla target identity:** the second gate binds the phrase to the resolved target and host (`GateTarget::Plan`, `Item`, `Cleanup(DeletePlan)`; `TRASH N UNDER <root> ON <host>`, `REMOVE ALL CONTAINERS ON mbp`) and revalidates the plan fingerprint before any effect. Unavailable items are refused with `<label> is unavailable · <why>` (`app.rs`), never retargeted. Tests: `app_tests_flows.rs: docker_cleanup_takes_two_gates_and_revalidates_before_executing`, `app_tests_parity.rs: hp04_browser_lists_previews_and_jumps_safely` (Enter on a jump query with no suggestion is a Submit: `~/nowhere` reports "! nowhere"), `hp06_sibling_batches_carry_exact_members_and_report_each_failure` (activities picker over live rows), `hp09_…` (daemon-down rows unavailable with the daemon's reason), `hp21_deletion_is_authorized_validated_at_commit_and_never_falls_back`.

**Caller migrations:** showcase pickers, TablePro and Jackin match the new `Submit` variant explicitly (a query that matches nothing keeps the picker open); no behaviour change for them.

**Captures inspected:** `shots/h_hp04_jump_error` (the Submit path), `shots/h_hp06_batch_picker` (activities picker), `shots/h_hp09_remove_gate2` and `shots/h_hp21_gate2_typed` (target-bound phrases).

**Deferred remainder:** TablePro tab-ID owner mapping and the destructive fallback removal (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
