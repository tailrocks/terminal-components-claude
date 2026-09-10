# HP07 — Native project-task adapters

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Represent package.json, Just, Make, Taskfile and mise task sources using fixture manifests or fixture tool output; show runner, provenance, limits, args, errors and child cwd. Pure parsing of fixture input is allowed.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live manifest discovery, runner probes, external task listing and actual project-task execution.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve package.json, Just, Make, Taskfile and mise tasks. Current pnpm-looking tasks are mise fixtures, not native adapters.

**Source evidence and mandatory scope:** [matrix HP07](../holla-parity-matrix.md#hp07--native-project-task-adapters) — `OP20`, `OP21`, `OP22`, `OP23`, `OP24`, `OP25`, `OP26`, `OP27`, `OP28`, `OP29`, `OP30`, `OP58`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Each task resource states defining file, runner, effective cwd, description, provenance and exact argv. Preserve package lock precedence pnpm/yarn/bun/npm, sorted script names and string-only values; Just summary discovery; conservative Make declaration order; Taskfile JSON discovery; uncapped ordered mise descriptions. Preserve visible first-30 limits where legacy has them, or provide an explicit more-results control. Missing runners/discovery errors must be truthful rather than invented task success.

**Architecture / reusable components:** Create bounded typed source adapters with no implicit shell expansion. Task execution still delegates recipe interpretation to its tool; trusted custom config does not automatically trust a project task file. Review newly introduced trust separately while preserving intentional task execution. Reuse existing task picker, argument form, trust page and activities.

**Required deterministic fixture:** `parity-task-sources` — each filename variant/parser, invalid/missing manifests, runner absent, duplicate tasks, 31 entries, package lock conflicts, whitespace/Unicode/metacharacters, description-only mise output, failed discovery, child versus ancestor cwd.

**Acceptance / automated verification:** Assert exact IDs, order/cap/source diagnostics and argv for every OP20–OP30 variant. Assert Make never executes during discovery and rejects unsupported target syntax without claiming a full Make parser. Assert failed mise discovery cannot consume misleading stdout as authoritative. Test original cwd and explicit trust cancellation; launch-time args are not runtime stdin. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture one native adapter per source, capped results, parse/unavailable state, provenance review and running task in child cwd.

**Dependencies:** HP01/HP14/HP15/HP17; F01/F08/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
