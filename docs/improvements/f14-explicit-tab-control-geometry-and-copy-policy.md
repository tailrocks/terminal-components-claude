# F14 — Explicit tab/control geometry and copy policy

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Define source-preserving tab/control geometry and apply it at the shared text boundary used by Holla input, output and preview. Complete the retained cross-renderer fixture and required compatibility migrations so shared policy stays consistent.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated editor features; the shared policy and its cross-renderer correctness checks remain current work.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · design inconsistency / architecture weakness.** Literal `A\tB` gives
different geometry in TextInput/TextArea/CodeEditor versus the viewport's existing
four-space expansion. `TextBuffer` normalizes CR/LF but stores other controls;
separate renderers have no shared declared display/copy rule. Text probes and
interaction peer verification agree. Ratatui filters control graphemes here;
these observations do **not** prove terminal escape injection.

Define source storage separately from display: tab expansion, visible treatment
of controls, source-to-cell mapping and copy-as-source versus copy-as-displayed.
Reuse the logical grapheme mapping from F10 and existing four-space viewport
policy as the default candidate. Literal document tabs are not the same as the
Tab key's configurable code indentation. Preserve data; no silent control-byte
deletion. Risk: medium/high public cursor-column semantics. Acceptance: the same
tab/ESC/BEL/CR/LF/CRLF/combining fixture across input, textarea, code, cell editors,
viewport and both Diff modes proves declared storage, cursor, click, clipping
and copy behavior. Use additive policy/mapping APIs during compatibility migration.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
