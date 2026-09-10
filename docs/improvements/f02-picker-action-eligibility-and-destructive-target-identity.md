# F02 — Picker action eligibility and destructive target identity

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Fix shared Picker eligibility for primary, secondary and pointer actions. Exercise Holla activity/resource pickers with empty, disabled, loading, filtered and stale results; preserve deliberate query-reset semantics.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

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

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
