# H00 — Existing Holla design and journey coverage

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Preserve and improve the existing Holla concept flows while adding the HP
representations. This task owns design consistency and existing-flow coverage;
it does not add all unimplemented ideas from CONCEPT.md to the backlog.

Inspect current code and [preview inventory](../holla-preview-inventory.md),
[Holla README](../../src/bin/holla/README.md),
[design note](../../holla-project/notes/04-design-note.md),
[CONCEPT.md](../../holla-project/CONCEPT.md) and [DESIGN.md](../../DESIGN.md).
The historical inventory is a starting point, not current completion evidence.

## Acceptance checklist

- [ ] Inventory current Holla pages, modals, actions, input owners and scenario routes;
  record missing current-phase behaviors and baseline failures before editing.
- [ ] Preserve Here as the decision/navigation surface and activities/plans as
  running work; keep context, host, cwd, provenance, risk and alternatives clear.
- [ ] Prove aliases/pins/hide/reset/why, scope navigation and child/parent context
  do not bypass query matching, availability, trust or destructive eligibility.
- [ ] Cover existing PostgreSQL, SSH/remote identity, GitHub clone and specialist
  handoff flows with truthful simulated results, failure and cancellation.
  Fix observed gaps in those existing flows; do not add new provider families.
- [ ] Cover dependency-aware editable plans, exclusion/undo/retry/skip/stop,
  in-session activity tabs and return-to-Here continuity without losing reports.
- [ ] Make keys, pointer targets, paste, drag/wheel and modal ownership agree.
  Holla modal tests must cover before/after first render, hidden background input,
  cancellation, stale opener, queued events and target drift even while F01 is deferred.
- [ ] Inspect current Junie geometry/focus, restrained planes, semantic color,
  contextual hints and readable narrow layouts. Preserve drafts/selection on resize.
- [ ] Keep all product effects in simulation; assert exact effects and secret
  redaction rather than accepting generic success or label-only fixtures.
- [ ] Retain the scenario journeys below plus every new current HP fixture;
  record tests and inspected captures. Run the shared final repository gates.

## Existing scenario coverage

These names come from the current Holla README; verify against Scenario::ALL.
Add newly implemented current-phase scenarios as they are introduced.

| Scenario | Required journey | Evidence |
| --- | --- | --- |
| first-use | Progressive discovery, suggestions, explore and alternatives | Pending |
| rust-dirty | Git state, review, activity and resulting resource state | Pending |
| monorepo-root | Multi-project plan with correct repository identities | Pending |
| monorepo-child | Child cwd, parent scope, trust and arguments | Pending |
| docker-cleanup | Selection, both gates, drift, execution and outcome | Pending |
| disk-cleanup | Eligibility, exact targets, cleanup gates and honest results | Pending |
| upgrade-plan | Exclusion/dependencies, failure, retry and verification | Pending |
| activities-multi | Tab ownership, retained/merged logs, attach/detach and picker | Pending |
| remote-host | Host provenance and host-bound review/cancellation | Pending |
| launch-failure | Truthful failure, retained output and usable next action | Pending |
| hard-cases | Partial/unavailable sources and exceptional resource states | Pending |

## Dependencies and evidence

Start inventory before implementation. Finish after the Current tracker slices,
HP23 platform fixtures and F23 Holla proof slices integrate. Use 80×24/120×40
TrueColor/Mono journeys and the additional size/color/input cases required by
changed geometry/state grammar. Capture files alone do not pass review.

Evidence: pending. Record source/binary provenance, tests, inspected capture
paths, platform scope and any unresolved limits. This task can finish without
Later work only when every current acceptance above passes.
