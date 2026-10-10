# Verify ownership, appearance, and behavior

This document owns the ownership and parity proof contract.
It defines how implementation work proves that reusable components produce
the reference appearance and behavior.
Read it with the [component-only contract](../architecture/component-composition.md),
[visual parity](visual-parity.md), [interaction parity](interaction-parity.md),
[oracle and provenance](oracle-and-provenance.md), and [conformance](conformance.md).

## VER-001 — Separate gate results

Record each gate independently:

| Gate | Question |
| --- | --- |
| Reference | Which immutable source and visual inputs define expected behavior? |
| Integrity | Are expected files complete, unchanged, and bound to their admitted generation? |
| API | Can an ordinary external consumer use the public interface? |
| Ownership | Do reusable components own the output and interaction? |
| Visual | Do current cells, cursor, and pixels match the reference? |
| Interaction | Do actual events produce the expected state and actions? |
| Generalization | Does the same implementation work with changed data and geometry? |
| Review | Did an independent reviewer approve this exact work and commit? |

A pass in one row cannot replace another row.
Store integrity is not live execution.
A compiled example is not a visual comparison.
A visual comparison is not proof of component ownership.

## VER-002 — Reference capture

Use the frozen `visual-baseline` tag for appearance and observable interaction.
Keep reference source separate from candidate source.
Record the source, toolchain, fixture, input sequence, time samples, and renderer identity.

ANSI replay can convert historical captures to another format.
It cannot recover original focus ownership, actions, or cursor state that the capture omitted.
Label converted evidence as converted evidence.
Use real reference execution to obtain missing observations.

Do not use `main` images or candidate output as expected output.
A new approved capture must identify the unchanged reference path that produced it.

## VER-003 — Capture bundle

Retain the established structure:

```text
baselines/<generation>/<app>/<screen>/<substep>/<geometry>/<color>.<ext>
baselines/<generation>/components/<component>/<substep>/<geometry>/<color>.<ext>
```

Generate ANSI, HTML, PNG, ASCII, TXT, and canonical frame JSON from one observation.
Include ASCII loss, PNG fidelity, semantic observations, and complete manifests.
ASCII is a diagnostic projection. It does not prove Unicode equality.

Bind every artifact to the same frame and renderer profile.
Use the stored expected PNG for comparison.
Do not regenerate expected pixels with candidate code.
Compare supported cells, styles, continuation cells, cursor, and decoded pixels exactly.
Do not trim meaningful whitespace or use a perceptual threshold as the acceptance condition.

Verify the complete artifact and admission chain.
Hash equality proves integrity, not independent approval.
A candidate-editable `approved` field is not sufficient trust.

## VER-004 — Component ownership proof

For each preview surface, record the components that own:

```text
visible content
borders and backgrounds
selection and focus marks
editing and cursor placement
scrolling and fades
layer placement and dismissal
action dispatch and disabled behavior
```

Trace real calls and their final output.
Reject a component call whose pixels are later replaced by a screen painter.
Reject state changes that bypass the component's normal update path.

Use controlled mutations in disposable builds.
Change a component's visible part and check the affected preview checkpoint.
Suppress one component action and check the actual preview journey.
These tests must use the production capture and interaction paths.
A synthetic cloned buffer only qualifies a comparator.

## VER-005 — Generalization checks

Use independently reviewed tests beyond the writer's initial capture set.
Keep seeds and fixtures reproducible after the reviewer freezes them.
Do not generate expected data from the candidate.

Test changed labels, row counts, row order, values, and disabled targets.
Test noncanonical widths and nonzero layout origins.
Test widths around real responsive breakpoints.
Keep the established five-size and five-color matrix as regression coverage.
Do not use that matrix as the entire input space.

Test paused mode with real keyboard, mouse, paste, and layer actions.
Paused mode must not disable menus, palettes, focus, or capture.
Test down, held press, release, cancellation, removal, and disablement.
Test nested overlays and recovery after resize.

Use controlled time for finite animation phases and timing boundaries.
A stable final frame does not cover transient press or loading states.
Test all distinct finite frames and the transition into and out of each relevant state.

## VER-006 — Forbidden-pattern detection

Specify source checks for preview crates and their generated sources.
Resolve aliases and wrapper calls where practical.
Do not rely only on names containing `historical`.
A renamed painter remains prohibited.

Flag raw buffers, paint methods, padded control strings, frame assets, and fixture-gated rendering.
Flag paused or size-specific bypasses of normal component update or layers.
Flag library APIs that consume preview-specific worlds or stored expected frames.

Treat static findings as evidence for review, not automatic proof in every context.
A component renderer may legitimately draw cells.
A content string may legitimately contain a box character.
Classify its role and owner instead of adding a broad exemption.

## VER-007 — Mutation matrix

The future verifier must reject:

| Mutation | Required failure |
| --- | --- |
| One cell glyph, color, or modifier changes | Visual mismatch. |
| One wide-cell continuation changes | Structural mismatch. |
| One cursor position or visibility changes | Cursor mismatch. |
| One decoded pixel changes | Exact pixel mismatch. |
| One component action is lost or duplicated | Interaction mismatch. |
| A target changes after reorder | Stable-identity mismatch. |
| A required artifact is missing or corrupt | Integrity failure. |
| A manifest, profile, or source hash is false | Provenance failure. |
| A case or shard is absent | Completeness failure. |
| A screenshot replaces live drawing | Ownership/generalization failure. |
| Paused mode bypasses a menu | Interaction/ownership failure. |
| Expected output is written during verification | Approval-boundary failure. |

Run mutations through the verifier used for real acceptance.
Do not substitute a separate assertion that the altered bytes differ.

## VER-008 — API examples

Compile current examples against their declared source revision when possible.
Use only ordinary public consumer exports.
Do not include private files with `#[path]` or copy implementation internals.

Keep compile status and runtime status separate.
A proposed API example remains pending until implementation and verification exist.
Do not silently mark it ignored and count it as passed.

## VER-009 — Complete execution evidence

Use stable requirement, case, checkpoint, run, and shard IDs.
The final aggregator must compare exact required and successful ID sets.
A total count alone can hide missing or duplicated cases.

Record not-run, failed, cancelled, timed-out, skipped, and passed outcomes separately.
Required skipped or missing cases fail implementation acceptance.
Do not let a cached receipt conceal changed inputs.

Rust test execution uses `cargo nextest`.
Explicitly select the visual tests that the normal run ignores.
Do not include capture/admission writers in that selection.

## VER-010 — Documentation task boundary

This document specifies future implementation gates.
It does not add their code or install their workflows.
The documentation task must report which checks already exist and which remain pending.

For this task, verify document scope, links, schema consistency, claims, examples, and reviewer records.
Known application defects remain implementation blockers, not reasons to expand the current write scope.
