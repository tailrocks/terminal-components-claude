# HP18 — Disk measurement, progress and cache

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build truthful fixture observations for allocated/apparent bytes, hardlinks, symlinks, partial/error states, bounded updates, cancellation and cache freshness. Use virtual storage/clock for schema/TTL/merge/restart cases.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live filesystem scanner, OS file metadata/dataless behavior, physical measurements and durable cache I/O.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve trustworthy disk bytes, progressive scans, cancellation, error states and durable cached hints. Timed fixture GB counts lack scanner semantics.

**Source evidence and mandatory scope:** [matrix HP18](../holla-parity-matrix.md#hp18--disk-measurement-progress-and-cache) — `LD004`, `LD005`, `LD006`, `LD010`, `LD011`, `LD007`, `LD008`, `LD009`, `LD017`, `LD018`, `LD019`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model file/directory roots, allocated versus apparent bytes, hardlink identity, symlink leaves, hidden entries, skip subtrees and inaccessible/dataless outcomes. Stream bounded updates after first paint without moving active targets. Cache age is a hint: root/depth-two v3 sizes, seven-day/mtime validation, pre-scan snapshot, changed-path omission, merge/atomic save; partial and deep-unverified data must remain labeled.

**Architecture / reusable components:** Introduce a filesystem observation adapter and immutable identity-bearing events used by simulation and later live scanner. Bound/coalesce producer queues and cancel/join workers on owner removal/rescan. Separate measured bytes, apparent size, estimate, observation freshness and physical free space. Reuse TreeView/progress/Props/StatusBar.

**Required deterministic fixture:** `parity-disk-scan` — sparse files, hardlinks, symlink dirs, hidden/skip roots, permission/missing/I/O/dataless errors, rapidly changing trees, bounded burst, cancel/rescan, old/new/missing cache nodes, stale TTL/mtime, two writers and corrupt cache.

**Acceptance / automated verification:** Assert recursive totals/counts and dedup saturating arithmetic, no symlink traversal, live partial/error counts, bounded queue, cancellation acknowledgment, no stale generation replacement. Assert exact cache schema/TTL behavior, no disk/history I/O before first paint, changed-path exclusion and valid concurrent merge. Do not label incomplete or depth-limited freshness exact. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture cached-first/live/partial/error/cancelled states, allocated/apparent differences and cache replacement preserving selection.

**Dependencies:** HP19/HP20/HP22/HP23; F05/F11–F15/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
