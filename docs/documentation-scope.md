# Documentation scope and retained task inputs

## Status

Documentation reconciliation covers active guidance, specifications, plans, reviews, and task-package prose. Product refactoring remains incomplete and not ready.

## Authority

`main` defines the intended split-crate architecture. `design/interaction-polish` defines the exact UI/UX, colors, and behavior. Historical task pins identify their original proof scope; they do not supersede those project references.

## Current evidence

The [per-file task disposition](refactoring-task-disposition.tsv) inventories every task Markdown path, its original and current SHA-256, and its disposition. An exact-line comparison against all 737 original design Markdown paths found all 963 original unchecked checklist lines still represented in the current documentation; none was lost. Detailed design specifications, task contracts, and API plans retain their concrete acceptance criteria. All 73 typed task READMEs pass `taskfmt lint --json`; requirements, typed acceptance blocks, checklist IDs, commit pins, and check commands remain preserved. All 73 outer task execution protocols remain intact beneath their updated context. CLAUDE symlinks remain aliases.

Task contracts bind `/task/trusted` as read-only inputs. For example, TASK-003's `trusted/obligations.md` describes source documents and scenarios bound into an immutable campaign manifest; TASK-072's `trusted/obligations.md` identifies independent driver, fixture and observer inputs as immutable. Their trusted Markdown bytes and synthetic qualification-fixture documents remain preserved as task inputs. They are historical evidence artifacts, not live product authority or a declaration that their old baseline is today's target. Their surrounding indexes and task contexts carry the current authority and reconciliation requirement.

Generated `.github` guidance and generated component-reference pages remain generator-owned outputs. Capture bundles, terminal transcripts, and fixture payloads remain data. Preserve their bytes and generation contracts; do not present them as current readiness evidence without a current matching verification run.

## Remaining work

Retain every planned requirement. Reconcile pinned task contracts and their judge inputs with current architecture and the accepted product reference before executing those tasks. Changing trusted inputs requires an intentional task/proof migration, rather than silently changing a historical judge while rewriting prose.

## Verification

Use the disposition table to compare preserved input hashes. Lint all 73 task packages. Compare typed requirements, acceptance, checklist blocks and pins with the original revision; a prose rewrite must not remove planned work or weaken gates. Active prose uses Status, Authority, Current evidence, Remaining work, and Verification sections after its title and optional frontmatter. Instruction aliases, generated outputs, and frozen fixture inputs are explicitly classified instead of being reformatted as active prose.

### Historical source-relative links

The frozen runner-bootstrap inputs for TASK-070, TASK-071, and TASK-072 retain six source-relative links from their original context. Their repository-level references are the [bootstrap review](refactoring-plan/bootstrap-review.md) and [proof contract](refactoring-plan/proof-contract.md). The copied task inputs retain their original bytes and historical scope; use these canonical links for project navigation rather than treating the copied relative paths as active links.
