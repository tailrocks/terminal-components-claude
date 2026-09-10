# HP21 — Deletion authorization, dry run and recovery mode

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Enforce immutable simulated target/mode/revision authorization, all mapped path/policy rules, Trash default, explicit permanent mode and truthful dry-run results. Model drift, aliases, protected descendants and backend failure before any simulated effect.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Actual canonical filesystem validation, Trash/permanent mutation, pathname-race probes and CLI bypass tests.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve deletion review, Trash default, explicit permanent mode, dry-run and path protections. Current gates protect fixture labels, not filesystem identity.

**Source evidence and mandatory scope:** [matrix HP21](../holla-parity-matrix.md#hp21--deletion-authorization-dry-run-and-recovery-mode) — `L20`, `L21`, `LD048`, `LD049`, `LD051`, `LD055`, `LD057`, `LD058`, `LD050`, `LD052`, `LD053`, `LD054`, `LD056`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Resolve paths, inherited policies, recovery mode and estimated effects before review. Keep Cancel default; permanent selection is explicit and resets authorization; broad operation adds typed target-bound phrase. Dry-run uses identical discovery/validation and emits Would remove, with no daemon stop, cleanup-target mutation or false Removed result; its audit-log write is explicit. Application-owned filesystem mutation has one shared boundary; external Cargo/Gradle/Docker effects disclose tool-native behavior.

**Architecture / reusable components:** Create immutable authorized cleanup specification carrying canonical parent/leaf identity, host/root/selection/mode/revision, category/process/age policy and typed dry-run outcome. Validate absolute lexical components and every LD053/LD054 deny rule, including ancestor containment of protected descendants. Reject symlink ancestors except exact macOS aliases; preserve selected-link-only semantics. Revalidate after sizing, never silently fallback from Trash to permanent. State the unprivileged-user threat model and remaining pathname race limit.

**Required deterministic fixture:** `parity-delete-safety` — every allowed/denied root, home/container/browser/cloud data, ancestor containing protected child, duplicate parent/child, Unicode/newline names, replaced symlink ancestor/leaf, missing/unreadable target, drift during sizing, Trash collision/exhaustion, mode change and dry-run.

**Acceptance / automated verification:** Assert no denied target or protected descendant mutates through any entry, canonical checks rerun at commit, leaf symlink target survives, retry only transient AlreadyExists (max three), no permanent fallback. Assert dry-run validates/logs exact would-results but triggers no process or cleanup-target effects; explicit audit logging is allowed. Gate invalidates on any material target/mode/policy change; alias/history/CLI cannot bypass authorization. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture selected path/mode facts, permanent toggle, typed gate, invalid/changed target, Trash backend failure and truthful dry-run result.

**Dependencies:** HP17/HP18/HP19/HP20/HP22/HP23; F01/F02/F05/F23. Safety boundary precedes wiring any new destructive action.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
