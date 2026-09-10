# HP23 — Platform and terminal capability contract

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Cover every mapped macOS/Linux UI availability/fallback state using explicit fixture capabilities; prove actual preview color/terminal behavior on available test hosts. Make simulated platform behavior distinct from verified native execution.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Production OS adapters, native filesystem/Trash/open/process probes on both operating systems and full CLI parity.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve implemented macOS/Linux and terminal behavior while keeping explicit exclusions separate. Current platform fixtures do not cover legacy fallbacks.

**Source evidence and mandatory scope:** [matrix HP23](../holla-parity-matrix.md#hp23--platform-and-terminal-capability-contract) — `X01`, `X02`, `X03`, `X04`, `X05`, `X06`, `X07`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide host-specific fixtures for every platform-sensitive matrix row: macOS casks/insights/dataless scanning/Spotlight/native Trash/open/reveal; Linux portable tools, hidden mac-only actions, FreeDesktop Trash, xdg-open fallback and process subreaper. Do not add apt/dnf/systemd-user as an old requirement. Do not call Windows, plugins, release infrastructure or nonexistent CLI strings parity gaps.

**Architecture / reusable components:** Platform capability adapter supplies explicit available/unavailable/error and native-path/store resolution. Preserve distinct XDG paths for frecency/sizes/logs versus dirs-based service/trust caches. Keep all effects simulated until production integration; no generic OS framework extraction without reuse. Apply Junie glyph/focus/no-color grammar to every added surface.

**Required deterministic fixture:** `parity-platforms` — macOS personal/development, Linux without desktop/trash mount helper, Linuxbrew, missing opener/OSC52, dataless policy failure, protected platform aliases, unavailable Spotlight and TERM/backend error.

**Acceptance / automated verification:** Assert all platform gates/fallbacks, no macOS command spawned on Linux, no destructive fallback, honest clipboard result and correct store location. At operational gate run controlled filesystem/PTY/Trash/open probes on both OSes; unavailable OS evidence remains explicitly pending, never passed by a macOS fixture. Run all relevant F21/F22/F23 terminal and provenance gates. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture platform availability/unsupported explanations in TrueColor/Mono/NO_COLOR; retain CLI/PTY/platform transcripts. No capture requirement for excluded CI or documentation-only features.

**Dependencies:** All HP01–HP22 platform cases; F21/F22/F23. This is a cross-cutting acceptance gate, not a new operating-system integration project.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
