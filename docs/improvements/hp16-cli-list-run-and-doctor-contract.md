# HP16 — CLI list, run and doctor contract

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Full list/run/doctor/JSON/exit-code and headless CLI parity, including simulated CLI expansion, is outside this design/prototype phase. Existing preview scenario/color/motion/frame options must continue to work.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve bare launcher, browse, list, list --json, exact-ID run, doctor, help/version and exits. Preview parser currently ignores unknown arguments.

**Source evidence and mandatory scope:** [matrix HP16](../holla-parity-matrix.md#hp16--cli-list-run-and-doctor-contract) — `E01`, `E03`, `E04`, `E06`, `E07`, `E08`, `E09`, `E10`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Specify one CLI over the shared registry, with deterministic simulated adapters while in preview. Preserve JSON v1 fields id/label/group/danger and text listing, warnings on stderr, exits 0/1/2/3/4, global warning refusal for run, exact-ID lookup and confirmation requirements. Doctor exposes detected groups/counts, scan timing and config paths/health. Bare invocation remains Here; browse retains file/directory start semantics.

**Architecture / reusable components:** Use a typed parser and execution kind so GUI-required actions are explicitly identified; do not claim all registry actions are noninteractive. Preserve actual headless task routes and stdin. Separate explicit durable trust approval from one-run --yes, document changed behavior, and provide an explicit trust operation for unattended setups without changing existing list v1 fields.

**Required deterministic fixture:** `parity-cli` — help/version/invalid args, empty/mixed registry, exact/unknown ID, all risk-confirm-trust combinations, valid+invalid config, warning-before-lookup, known task output, stdin prompt/EOF and GUI-required action.

**Acceptance / automated verification:** Assert valid JSON-only stdout, exact schema/danger strings, diagnostics stderr and correct exit for every branch. No execution on refusal or parse error. Verify list and run resolve same ID/cwd/argv. Retain old commands; new explicit trust command and simulation selectors must have their own strict parser tests. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** CLI transcript captures: JSON/text listing, all exits, doctor healthy/warnings and stdin prompt. UI captures only for browse/Here handoff; no invented CLI screen.

**Dependencies:** HP01/HP03/HP04/HP14/HP15/HP17/HP23; F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
