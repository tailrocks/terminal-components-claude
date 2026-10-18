# Source-first planning re-audit

## Status
Historical audit plan; its dated execution gate is superseded. The split-crate implementation remains incomplete.

## Authority
The current architecture direction is main’s reusable crate split. The design/interaction-polish branch owns UI/UX, colors, and interaction behavior. This report is evidence from the recorded pins below, not a competing authority.

## Current evidence
The findings and task coverage below are historical, commit-scoped audit records. Their verification applies only to the named artifacts and recorded revisions.

## Remaining work
Preserve unresolved findings and planned repairs in [remaining work](../remaining-work.md); use the [documentation map](../README.md) for current authority and readiness context.

## Verification
No current verification is implied by this historical report. Re-run relevant checks against the current branch before closing a task.

## Scope and current disposition

This is a historical source-first audit plan. Its old authorization boundary and dated task checkpoints do not govern current work. Current direction keeps the main branch’s reusable crate architecture while treating design/interaction-polish as authoritative for UI/UX, colors, and behavior. The architecture refactor remains incomplete; use the documentation map and remaining-work ledger for present readiness.

The expanded source-first audit remains incomplete. Earlier catalogs and test receipts do not settle its current scope or source/task synchronization. No generated planning-tree hash is maintained as a readiness signal; executable validators and trusted task inputs remain the authoritative checks.

## Independent ownership

| Audit | Exact task coverage | Additional source coverage |
| --- | --- | --- |
| [Foundation](reaudit-foundation.md) | 001–008, 070–072 | Baseline provenance, host/runner/accounting/architecture qualification and production-provider handoff |
| [Components A](reaudit-components-a.md) | 009–020 | Runtime, theme, layout, text, fields and source-level repair feasibility |
| [Components B](reaudit-components-b.md) | 021–031, 073 | Editors, collections, component composition, registry and attribution |
| [Showcase](reaudit-showcase.md) | 032–039 | Source-driven screen/state/input/flow completeness |
| [Holla](reaudit-holla.md) | 040–050 | Source-driven routes, native/resized assertions and staged full-flow closure |
| [Jackin and TablePro](reaudit-jackin-tablepro.md) | 051–064 | Source-driven screen/state/input/flow completeness and staged contributions |
| [Closure](reaudit-closure.md) | 065–069 | Test relocation, performance, API/docs, canonical task-format, DAG and integration authority |
| [History](reaudit-history.md) | Cross-cutting source obligations | Historical revisions, authority/supersession, public component/API inventory and missing clauses |
| Coordinator | Shared contracts and complete acceptance | Evidence triage, structural repairs, bidirectional coverage, exact current source/tool identities, independent repair review and final integrity |

The seven task partitions cover each of the 73 task packages exactly once. Reviewers must read all assigned task files, record path/line count/SHA-256 and semantic coverage, and compare claims with actual source and required outcomes. Repeated protected payloads still require exact identity and relevant semantic checks. Binary archives require source membership/byte identity and use-path verification rather than a fictional line count.

## Finding and repair standard

Each finding must identify the exact contract line, authoritative source, violated requirement, concrete counterexample or reproducer, enabling architectural condition and proposed structural repair. An isolated weak test is not automatically a proven whole-task false pass; the applicable acceptance chain must be inspected. Missing or indirect evidence is unverified, not accepted.

Repair scope includes task contracts, writable-path ownership, dependencies, source matrices, protected payloads and verification preparation. Historical evidence is preserved rather than silently rewritten. Canonical task-format schemas and AGENTS bytes remain unchanged unless the actual current canonical source requires a new version. Material corrections receive an independent recheck; authors cannot close their own repairs solely with self-tests.

## Completion conditions

1. Derive and audit every explicit PLANNING_GOAL requirement and all 24 required top-level sections against current evidence.
2. Complete every assigned task-file read and semantic audit, with no uncovered task/package file.
3. Independently compare source histories, component APIs and four-app behavior with inventories so self-consistent omissions cannot pass by joins alone.
4. Resolve all material findings with source-backed repairs or evidence-backed rejection; independently recheck changed contracts and their original counterexamples.
5. Prove command feasibility, scope ownership, hard dependency sufficiency, nonvacuous exact verification, protected oracle provenance and safe integration for every task.
6. Re-run applicable canonical lint, graph/source/payload joins, preparation qualification and negative probes on repaired bytes. Do not substitute generic PASS totals for semantic coverage.
7. Reconcile current completion claims, record remaining future prerequisites honestly, regenerate and independently check the final artifact inventory, and only then close the goal.

Reports and repairs are in progress. No current completion claim is made.

## Added whole-branch comparison

The user additionally requested a complete reread of the difference between `main` and `visual-baseline` (then `holla`), including completed and half-completed work on main, broken behavior, and changes that made the product worse. The historical comparison records source-qualified findings for preservation. Current implementation follows the split-crate architecture while matching the design branch’s product contract.

Compare exact recorded tips `main=7b27732a8c3c131760ec3438f641cb3c11343a42` and `2e2401393c47360741ebd321679de08982dca50a` (then `holla`; live checkout is `visual-baseline`). Keep immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` separate: moving-branch additions are contextual evidence, not automatic product acceptance. Enumerate every changed path without relying on incomplete rename detection; inspect relocation counterparts explicitly. Text changes require complete assigned diff reading, binary artifacts require exact identity, decoded content where applicable, and provenance/use disposition. Record completed main work to preserve, unfinished main work to continue, regressions requiring source-compatible repair, and obsolete material not to port. Every proposed change must join a task owner and proof obligation. The whole-branch census, source reading, decisions and independent recheck are additional completion conditions; previous task-file audit alone does not satisfy them.

The [cross-partition finding register](reaudit-findings.tsv) preserves original findings, repair locations, historical review status, and each remaining synchronization or qualification action. Detailed product requirements stay in their task-owned plans. A repaired row with pending synchronization or qualification is not a closed goal condition.
