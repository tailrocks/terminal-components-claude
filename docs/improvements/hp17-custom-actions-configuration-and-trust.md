# HP17 — Custom actions, configuration and trust

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Use fixture configurations and in-memory versioned trust to show exact argv/cwd/source/digest, diagnostics, risk/confirm rules and review invalidation. Simulate edits, relocation, restart and save failures without live config/history I/O.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live global/project config reads, persistent approvals, filesystem migration and trust CLI operations.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve global/project custom actions, declared risk/confirm, exact argv, scoped execution and durable content trust. Session path trust is insufficient.

**Source evidence and mandatory scope:** [matrix HP17](../holla-parity-matrix.md#hp17--custom-actions-configuration-and-trust) — `E11`, `E12`, `E13`, `E15`, `E16`, `E17`, `E18`, `E19`, `E14`, `E20`, `E21`, `E22`, `E23`, `E24`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Read XDG_CONFIG_HOME/holla/actions.toml or ~/.config/holla/actions.toml plus cwd .holla.toml; retain every required/optional [[action]] field and default. Present configuration resources, diagnostics with source/index, valid siblings, grouping and keywords. Reserve IDs against actual registry rather than stale hardcoded names. Show program/each argv/cwd/source/trust/risk before execution; explicit sh -c remains possible and clearly identified.

**Architecture / reusable components:** One validated action specification drives preview and execution; never interpolate argument strings. Bind trust to reviewed whole-file digest, origin/path and effective cwd/argv; re-read on execution so edits revoke review. Legacy digest-only trusted.json may be read as migration evidence, but broader path binding requires renewed review. Persist sorted/versioned approvals atomically; corruption untrusted, save failure no launch. Global config remains user-owned trusted input; inherited environment has no invented TOML overrides.

**Required deterministic fixture:** `parity-custom-actions` — minimal/full schema, invalid ID/type/blank field, duplicate global/project/builtin ID, malformed sibling/file, XDG/fallback, spaces/newlines/metacharacters, comment edit, identical bytes moved, config edited during review, corrupt store/write failure.

**Acceptance / automated verification:** Assert accepted argv preserved exactly, safe/mutating/destructive plus confirm combinations, precise cwd and no implicit shell. Review displays whole trust scope; cancellation changes nothing; unchanged authorized content survives restart, changed content/path re-prompts. --yes authorizes one run only; explicit durable approval supports automation and revocation without alias bypass. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom provenance, exact argv facts, source-index diagnostics, Cancel-default trust review and changed-definition rejection.

**Dependencies:** HP01/HP14/HP16/HP21; F01/F08/F09/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
