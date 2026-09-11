# Source-first planning re-audit

## Scope and current disposition

The user reopened the planning goal on 2026-09-11 to verify every plan item and every line of every task, identify omissions, and repair them. The original planning/preparation boundary remains: no terminal-components production refactor, task execution, integration branch, commit, merge or publication is authorized by this re-audit. The explicitly permitted external tui-snap dependency remains separate.

Completion is unproven. The previous turn made progress by producing the catalog and executed evidence, but those results do not settle the expanded source-first audit. At audit start, the previous complete artifact inventory matched 864 files and 143,008,176 bytes; its manifest SHA-256 was `e31a2cb70a4a987c5a5e5abc46fcc3d8c1f3188cd4c6ac5988dc0e4de6fce82f`. That inventory is a pre-audit snapshot, not a current seal after edits.

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

The user additionally requested a complete reread of the difference between `main` and `holla`, including completed and half-completed work on main, broken behavior, and changes that made the product worse. The working interpretation is to continue main's accepted architecture and restore source-qualified Holla behavior, not restart the refactoring from Holla or mutate either branch. Clarification about a separate Holla-branch plan is pending.

Compare exact local branch tips `main=7b27732a8c3c131760ec3438f641cb3c11343a42` and `holla=2e2401393c47360741ebd321679de08982dca50a`. Keep immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` separate: moving-branch additions are contextual evidence, not automatic product acceptance. Enumerate every changed path without relying on incomplete rename detection; inspect relocation counterparts explicitly. Text changes require complete assigned diff reading, binary artifacts require exact identity, decoded content where applicable, and provenance/use disposition. Record completed main work to preserve, unfinished main work to continue, regressions requiring source-compatible repair, and obsolete material not to port. Every proposed change must join a task owner and proof obligation. The whole-branch census, source reading, decisions and independent recheck are additional completion conditions; previous task-file audit alone does not satisfy them.

The current cross-partition [finding register](reaudit-findings.tsv) links original findings, repair evidence, independent rereviews and remaining integration checks. Grouped application findings retain their individual source-backed detail in the linked reports. A repaired row with pending synchronization or qualification is not a closed goal condition.
