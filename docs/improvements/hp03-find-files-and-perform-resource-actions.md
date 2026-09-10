# HP03 — Find files and perform resource actions

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Use an in-memory file index with progressive results, exclusions and exact native-path identities. Model Open/Reveal/Copy/Analyze intents and success/failure, including target/cwd/argv and clipboard destination.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real home indexing, OS open/reveal, OSC52/system clipboard transport and worker joins against live providers.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve home file/folder discovery and Open, Reveal, Copy path, Analyze actions. Current file rows are fixture resources; Reveal incorrectly copies cwd.

**Source evidence and mandatory scope:** [matrix HP03](../holla-parity-matrix.md#hp03--find-files-and-perform-resource-actions) — `I-F01`, `I-F02`, `I-F03`, `I-F04`, `I-F05`, `I-F06`, `I-F07`, `I-F08`, `I-F09`, `I-F10`, `I-F11`, `I-F12`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide a Files scope in Here with mixed files/directories, indexed/partial counts and exact resource identity. Rank exact filename/stem, filename substring and fuzzy matches with deterministic path ties; expose bounded results (legacy 100) honestly. Keep an explicit blank-query state. Alternatives must operate on the selected path, not current cwd. Analyze directories directly and files via parent; preview resulting scope.

**Architecture / reusable components:** Use a cancellable file-index adapter; retain ignore/hidden policy, no symlink following and cloud/root exclusions. OS handoff and clipboard adapters receive validated path identity and typed intent. Reuse Picker, menu, Props and existing disk page routing.

**Required deterministic fixture:** `parity-files` — mixed home results; indexing in progress; .ignore/hidden/iCloud exclusions; same basename; Unicode byte-to-grapheme match; open failure; reveal on macOS/Linux; clipboard size/write failure; file versus folder Analyze.

**Acceptance / automated verification:** Assert bounded ranking, exact selected path through refresh/alternatives and cancellation joins pending index work. Assert macOS open/open -R and Linux xdg-open/parent fallback argv, failed handoff visibility, OSC52 payload/cap/flush behavior, and simulated Analyze root. Clipboard result must name actual destination without unsupported success claims. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture progressive results, resource alternatives, failed opener/clipboard and Analyze handoff at narrow width.

**Dependencies:** HP01/HP02/HP18/HP19/HP23; F09/F10/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
