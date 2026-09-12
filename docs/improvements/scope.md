# Current implementation scope

The current goal is a usable, keyboard-complete Holla design prototype with
meaningful simulated outcomes, reusable component improvements and retained
interaction/visual proof. It is not a production Holla integration project.

This scope is the user's current execution boundary. It takes precedence over
the broad delivery order and full-project closure language retained in task
contracts and [shared contracts](shared-contracts.md). No requirement is deleted:
work outside this phase stays unchecked under Later in the tracker.

## Product and code boundaries

- Work in `src/bin/holla/`: Here/context, resource/action previews, scope,
  activities, plans, files, disk/cleanup and the named provider fixtures.
- Use [CONCEPT.md](../product/CONCEPT.md) for product intent,
  [DESIGN.md](../design/DESIGN.md) for Junie visual/interaction grammar and the
  [Holla design note](../product/notes/04-design-note.md) for the
  existing prototype model. Extend this model; do not restart the design.
- Reuse and extend existing `src/widgets`, `src/core`, `src/ui` and theme
  semantics where a selected Holla flow needs them. Inspect existing widget
  and showcase code first. Keep Holla domain/policy in Holla.
- Shared API/invariant fixes include necessary caller migrations, affected
  consumer regressions, docs and targeted showcase examples. That does not
  authorize unrelated TablePro, Jackin or showcase redesign.
- Extend existing Holla tests, deterministic fixtures and capture tools. Preserve
  existing new-Holla flows through H00; broad unimplemented concept ideas are
  not additional feature requirements for this phase.

## Simulation and operational boundary

All product providers remain deterministic in-memory simulations on a virtual
clock. Exact action specifications, target identity, errors, cancellation,
trust, eligibility and final world effects are real prototype behavior.
A label or generic success animation is not sufficient.

Model filesystem, cache/history, restart, concurrency, subprocess, prompt,
clipboard, Trash and platform outcomes with fixtures or in-memory adapters.
Pure parsers/codecs and round trips on fixture bytes are allowed. Never silently
replace a required failure/identity contract with a fixture label.

Do not wire real Git/Docker/Cargo/Gradle/Brew/mise/SSH/PG commands, external
services, home scans, user config/history I/O, durable trust/logs, OS clipboard,
open/reveal or filesystem cleanup into the product. Live providers, persistence,
production execution and all HP16 CLI parity remain Later work. Existing preview
scenario/color/motion/frame options continue to work.

Normal repository editing, build/test commands, deterministic test temporary
files, capture outputs and isolated PTYs that run the preview/test helpers are
allowed verification work. This does not authorize production provider effects.
F21 repairs the preview's own terminal lifecycle; HP15's real task processes
remain deferred. Unavailable real-platform evidence cannot pass via fixtures.

## Current task selection

The [tracker](PLAN.md) is authoritative for Current versus
Later checkboxes. Each task states its current slice, deferred remainder and
full retained acceptance contract. A current checkbox only closes that slice.
Full task completion requires every retained acceptance clause, including any
later owner/platform/production gate.

F01/F03/F04/F06/F07/F08a/F08c/F09/F16/F17/F18 are deferred as standalone work.
F02/F08b/F08d/F20 and selected F23 proofs have both current and later slices.
HP01–HP15 and HP17–HP23 have current simulated UI slices and later operational
or non-UI slices. HP16 is entirely deferred. O04/O05/O07 remain conditional.
Retired generic widget/theme/collection/container/worker frameworks remain
outside the plan; required concrete task ownership stays current.

If inspection proves a deferred task is a necessary dependency of a selected
Holla flow or a changed shared invariant, document the exact consumer and failing
contract before changing its classification. Promote only that required slice;
keep unrelated acceptance unchecked. Do not pick an unnecessary widget merely
to activate its backlog. Do not leave a known-wrong current flow or introduced
regression behind under the deferred label.

## Delivery order

1. Start H00's current-source inventory and baseline. Record which required
   scenarios already work and which fail; historical test totals are not proof.
2. Establish Holla action/target/input ownership with HP01/HP14/HP15/HP17/HP21/HP22
   and F02/F08b/F08d. Define small shared contracts first so circular legacy
   dependencies do not require finishing entire HP items before starting peers.
3. Repair shared text ownership F10–F14, then F19/F20/F15. Complete F05 before
   lazy Holla trees rely on it. Run F21/F22 and relevant F23 proof work throughout.
4. Complete HP03/HP04/HP18/HP19/HP20 file and disk flows, then HP05–HP13 provider
   UI variants using the same simulated action/outcome/safety contracts.
5. Finish HP02 usage/search and HP17 trust/config UI coverage. HP23 and H00
   integrate platform fixtures, existing concept flows and cross-surface design.
6. Close current checkboxes only after their semantic tests and inspected visual
   evidence pass. Finish with shared repository gates and an explicit list of
   remaining Later work.

This is dependency order, not permission to postpone an independently fixable
current P1. HP14/HP15, HP18–HP22 and HP01/HP17 share contracts; their reciprocal
references are integration requirements, not a cyclic wait schedule. References
to deferred F01/F03/F09/HP16 in original HP dependencies require equivalent Holla
ownership behavior now, not unrelated application or production implementation.

## Progress and completion

- The tracker owns phase completion. Task checklists record implementation,
  validation and visual evidence; update both in the same change.
- Keep unchecked rows with `In progress`, `Blocked: <evidence>` or
  `Deferred: <reason>` notes. Never use a checked box for work merely planned,
  scaffolded, captured without inspection, skipped or unavailable.
- Each task's Evidence section records test/scenario names, inspected captures,
  source/binary provenance, platform scope and residual limitations. For HP
  work, map every referenced capability row to current representation evidence
  or its explicitly deferred non-UI/operational clause. Do not silently omit rows.
- A current slice may finish while its Later checkbox remains open. Do not mark
  a matrix row Covered until the complete stated representation is demonstrated;
  keep Partial/Missing where deferred representation clauses remain.
- Finish the current goal when all Current rows pass their defined slices,
  relevant [acceptance gates](shared-contracts.md#per-change-acceptance-gate)
  pass, current visuals are inspected and no introduced regression remains.
  Later rows do not block this scoped completion and must remain visible.
- A proven tool/platform limit names the missing evidence and next external
  check. It is a limitation, not a passing result or full production-parity claim.
