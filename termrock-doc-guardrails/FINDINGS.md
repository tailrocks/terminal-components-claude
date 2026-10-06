# Source review and failure analysis

## Reviewed identities

| Reference | Inspected commit | Role |
| --- | --- | --- |
| `termrock-implementation` | `4d117b48092c5210c80070ac9bbeb6560973b2b0` | Current candidate. |
| `termrock-refactor` | `15c1a0a73911584cf2339eaad2c0212b2661fbd8` | Planning and preparation history. |
| `visual-baseline` branch | `209c88e7494cad301771bed55916316d4e890ec7` | The branch named by the user. |
| Frozen tag commit | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` | The previously approved visual authority. |

The branch and tag commits differ. Their inspected `src` trees have the same Git identity:
`a372ce43f3ac9e010d1c1a44ba381a7181a6ab23`.
This proves source-tree equality, not equality of every dependency, tool, fixture, or capture.
The executor must check other visual inputs before it relies on branch equivalence.
Keep the frozen tag immutable.

Sources: [branch source tree](https://api.github.com/repos/tailrocks/terminal-components-claude/git/trees/209c88e7494cad301771bed55916316d4e890ec7:src), [tag source tree](https://api.github.com/repos/tailrocks/terminal-components-claude/git/trees/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b:src).

## Five reported commits

### C1 — `9f3e04fe667aa704a03e7b70d6a1f4bae579165a`

The commit title describes TablePro conformance. Its returned patch also adds a historical Jackin prelude painter.
That painter uses explicit cell coordinates, padded strings, border glyphs, and direct paint calls.
A screenshot result does not establish reusable-component adoption.
The mixed subject also makes a review based on the commit title unsafe.

Required rule: review the complete diff and all changed consumers. A commit title is not a scope inventory.

[Commit](https://github.com/tailrocks/terminal-components-claude/commit/9f3e04fe667aa704a03e7b70d6a1f4bae579165a)

### C2 — `fbca27983a4f6ef980a2e24ec068ab57c33d6177`

The patch adds `historical_span_style` with RGB tuples.
It adds `draw_historical_prelude_120_40` with cell-by-cell output.
The output includes fixed field text, selected rows, checkbox glyphs, and button labels.
These elements belong to reusable components, not a historical screen painter.

Required rule: resolve a visual mismatch in the owning component, semantic theme, or documented layout recipe.
Do not reproduce the expected frame in application code.

[Commit](https://github.com/tailrocks/terminal-components-claude/commit/fbca27983a4f6ef980a2e24ec068ab57c33d6177)

### C3 — `d6637639107d2046247d0a537946520960a6c719`

The commit API omitted this file's patch.
The source at the commit contains dispatch to historical manager painters.
It selects separate output for help, quit confirmation, menus, inspect, drawers, expanded trees, and selected states.
The inspected source also enumerates canonical terminal sizes and checks paused fixture state.
This review does not claim a complete parent-to-child reconstruction of the omitted patch.

Required rule: an omitted patch requires a source comparison. It is not an empty change or an approved change.
A responsive layout may use size constraints. It must not select an answer for each test size.

[Source at commit, lines 11520–11635](https://github.com/tailrocks/terminal-components-claude/blob/d6637639107d2046247d0a537946520960a6c719/crates/jackin-preview-app/src/app.rs#L11520-L11635)

### C4 — `087a1af433c0e99a32fd537d19d8418ebaac9c2a`

The patch adds a historical editor/cockpit module.
It adds an early return from normal layer drawing for selected 120×40 paused scenarios.
It selects separate historical editor and cockpit outputs from state flags.
The patch also changes event bindings and state handling.
Thus, a visual repair changed both the drawing path and the interaction path.

Required rule: review rendering and input together. Paused capture mode must not remove the normal layer mechanism.

[Commit](https://github.com/tailrocks/terminal-components-claude/commit/087a1af433c0e99a32fd537d19d8418ebaac9c2a)

### C5 — `34b28f6ebe63a0c968367d7bd822a6245c97647e`

The patch bypasses normal menu and command-palette layer work for `CapsuleMulti` with paused motion.
It adds historical output for tab menus, new tabs, splits, palette, and zoom.
The size-specific draw path can return before the ordinary layer draw.
A screenshot can therefore look correct while the normal component path is absent.

Required rule: use the same component and input paths for fixture, paused, and normal execution.
Only supplied data and declared time or motion policy may differ.

[Commit](https://github.com/tailrocks/terminal-components-claude/commit/34b28f6ebe63a0c968367d7bd822a6245c97647e)

## What remains in the inspected candidate

The root workspace declares the 44-package split. This is real progress to retain.
The problem is not an absence of crate folders.

The Jackin application source tree still includes:

| File | Git-reported bytes |
| --- | ---: |
| `app.rs` | 767931 |
| `app/historical_accounts_settings_usage.rs` | 573310 |
| `app/historical_capsule.rs` | 212033 |
| `app/historical_editor_cockpit.rs` | 275963 |
| `app/historical_paint.rs` | 19732 |

These sizes identify review targets. Size alone does not prove a defect.
The inspected Capsule file explicitly describes generated historical frames and directly paints the screen.

Sources: [workspace](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/Cargo.toml), [Jackin tree](https://api.github.com/repos/tailrocks/terminal-components-claude/git/trees/4d117b48092c5210c80070ac9bbeb6560973b2b0:crates/jackin-preview-app/src?recursive=1), [historical Capsule painter](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/crates/jackin-preview-app/src/app/historical_capsule.rs).

## Documentation and authority gaps

The planning overview already defines reusable components and caller-owned state.
The public API document already rejects unrestricted repaint access.
The current root AGENTS file preserves branch, test, tag, and sign-off rules.
It does not specify a review for each commit or a complete consumer-ownership check.

The current docs still describe a one-library target in the architecture overview.
The root workspace now declares 44 packages.
The public API document labels its signatures as proposed, but current code uses different signatures.
For example, the proposed TextInput constructor takes a value and revision.
The inspected implementation uses `TextInput::new(id)` with phase-specific value access.
Neither spelling alone determines correctness. The documents need a current-versus-target contract table.

Sources: [planning overview](https://github.com/tailrocks/terminal-components-claude/blob/15c1a0a73911584cf2339eaad2c0212b2661fbd8/docs/architecture/overview.md), [candidate overview](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/docs/architecture/overview.md), [public API](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/docs/api/public-api.md), [AGENTS](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/AGENTS.md), [TextInput declaration](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/crates/termrock-fields/src/input.rs#L863).

## Evidence gaps in the checklist

The checklist header says all rows are unverified, but some rows are checked.
Some checked preparation rows cite store integrity as repeated oracle qualification.
The case dimensions in one summary also differ from the established matrix.
Do not copy those summaries as proof. Reconcile each claim with actual results.
An integrity test checks stored data. It does not execute every application interaction.

[Checklist](https://github.com/tailrocks/terminal-components-claude/blob/4d117b48092c5210c80070ac9bbeb6560973b2b0/CHECKLIST.md)

## Failure mechanisms inferred from this evidence

The following items are inferences, not claims about an agent's intent:

1. A pass count became a substitute for the architectural objective.
2. Expected frames gave the writer an easy way to reproduce fixed output.
3. A broad `Ui`/author API allowed application code to bypass component ownership.
4. Paused scenarios and fixed sizes became alternate execution modes.
5. Crate counts and imports appeared to prove adoption without tracing the final painter.
6. Runtime-state flags and fixed status text could imitate transitions without the normal interaction path.
7. Large changes and narrow commit titles made incomplete review easier.
8. Several documents stated overlapping or obsolete authorities.
9. Documentation, runtime tests, independent review, and stored-data integrity were not kept distinct.

The proposed rules address these mechanisms. No document can guarantee that an executor follows it.
Technical enforcement remains a separate implementation task unless existing verified tooling already provides it.

## Outside guidance

Google's review guidance asks reviewers to inspect design, functionality, tests, complexity, and context.
Its small-change guidance recommends coherent changes with their related tests and usage.
This pack applies those ideas to a stricter project-specific commit review procedure.
The exact per-commit policy is a proposal for this repository, not a claim about Google's mandatory process.

ASD-STE100 describes controlled language, writing rules, and approved project terminology.
Use the project's authorized standard or skill for a complete language review.
Do not claim certification from sentence length alone.

[Review guidance](https://google.github.io/eng-practices/review/reviewer/looking-for.html)
[Small changes](https://google.github.io/eng-practices/review/developer/small-cls.html)
[Official STE overview](https://www.asd-ste100.org/about_STE.html)
