# F20 — Find in read-only output through existing composition

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Compose query input and match navigation in Holla output and file preview. Apply every retained search/follow/focus/Unicode acceptance case to those surfaces; keep implementation local until shared reuse is demonstrated.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

Jackin panes/Inspect integration and extraction of a cross-application controller remain deferred. Holla completion does not close the full two-application contract.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · coverage gap: confirmed capability absence; proposed interaction/API design.**
TextViewport/Diff expose no query or next/previous match action. Holla service
filters and Jackin command filtering do not search displayed output. CodeEditor's
FindState and shared `ui::text::find_ranges` already provide a source-grapheme
mapping foundation. Ecosystem and API agents independently verified both absence
and reuse. This is not a defect in a promised current search feature.

After F19, compose a query input/match navigation into two present consumers
(Holla output and Jackin panes/Inspect), then extract only the demonstrated common
controller. Reuse TextInput/HintBar and source mapping, not a new search widget.
Risk: medium focus/follow/content invalidation. Acceptance: pasted query never
edits output, empty/no-match/next/previous/wraparound states, Unicode ranges,
content revision/eviction, match reveal, resize, Escape and explicit follow
restoration. Defer regex, replacement, file search and multiple cursors; current
evidence supports text finding, not those broader semantics.

Design evidence and peer dispositions:
[design verification](../plan-design-verification.md).

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
