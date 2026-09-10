# F10 — Segment logical text before applying styles

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the retained shared TextViewport contract. Prove split-style grapheme geometry and exact copies in Holla Activity, Plan and file preview, with relevant Diff primitive regressions.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice; unrelated application redesign is excluded.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed text-loss defect.** `TextViewport::ensure_layout`,
`src/widgets/viewport.rs:305`, segments each span separately and drops zero-width
pieces. One span `👩‍💻X` puts X at column 2; split styled spans put X at 4.
Separate spans `a`, combining acute, `X` copy `aX`, losing the stored accent.
Text and interaction agents independently rendered and copied both cases.

Root: style boundaries incorrectly define grapheme boundaries. Segment a whole
logical line, map source byte ranges to complete graphemes, then resolve styles
with a documented deterministic precedence for a cluster crossing runs. Use the
same map for rendering, width, selection, caret and copy. Reuse current Line/Span
concepts; the previous Diff-specific span fix is not the generic root fix.

Risk: medium style/API compatibility. Acceptance: split/unsplit combining,
ZWJ, variation-selector and flag sequences retain identical text, widths and
copied ranges; only declared style precedence may differ. Test narrow clipping,
nonzero origin, mixed tones and existing Diff emphasis/geometry. Cache logical
graphemes with document revisions so correctness does not add idle reparsing.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
