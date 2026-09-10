# HP13 — All legacy upgrade managers

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Complete every mapped manager alone and in aggregate with exact stages, platform availability, failure barriers and simulated version changes. Fix the standalone mise loop.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real manager discovery, installation/upgrade commands and host mutation.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Brew packages/casks, mise, Amp, Oh My Zsh and upgrade-all. Current apt+mise graph misses macOS workflows; standalone mise upgrade loops.

**Source evidence and mandatory scope:** [matrix HP13](../holla-parity-matrix.md#hp13--all-legacy-upgrade-managers) — `OP51`, `OP52`, `OP55`, `OP53`, `OP54`, `OP56`, `OP57`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Build host-specific plan from detected managers, and keep each manager directly searchable. Preserve Brew update, greedy --yes upgrade, cleanup, autoremove, doctor; cask variant only macOS. Preserve amp update, mise upgrade and sh <resolved ZSH>/tools/upgrade.sh. Show $ZSH override exactly. Aggregate independently runnable managers; review before apply and keep per-stage output.

**Architecture / reusable components:** Use shared typed stage specifications for standalone and aggregate variants; remove shell-chain duplication. Explicitly block dependent Brew stages after prerequisite failure while retaining independent managers. Refresh availability before execution and invalidate changed reviewed plans. Reuse existing upgrade Plan/Activity.

**Required deterministic fixture:** `parity-upgrade-managers` — every manager alone, all managers, none, Linuxbrew versus macOS casks, ZSH override/fallback, disappeared executable, failed update/doctor, independent successful manager, standalone mise follow-up.

**Acceptance / automated verification:** Assert every command/flag/order/cwd and exact preview, correct availability, parallel manager branches and blocked dependent stages. Standalone mise Upgrade must reach execution and change versions. Report partial upgrade and verification failure without generic success. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS aggregate plan, cask-only route, custom ZSH path, failed Brew prerequisite, independent completion and mise upgrade result.

**Dependencies:** HP01/HP14/HP15/HP17/HP23; F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
