# Documentation maintenance

## Product contracts

Keep product behavior and design authority in the active design documentation and linked feature specifications. Keep architectural decisions and package boundaries in [architecture](architecture.md). The roadmap should link to each active owner instead of repeating its implementation status.

When a planned behavior changes, update the owning specification and its acceptance checks. Preserve task IDs, dependency edges, prior scoped closures, and explicit open work. A completed slice closes only its recorded scope and revision; compile success alone does not establish parity or readiness.

## Generated and executable documentation

`docs/components/` is generated from Rust API documentation by `src/docs.rs`; update its source or generator and verify freshness through [development](development.md). `docs/visual-testing/README.md` and `docs/visual-testing/plan.md` are consumed by `xtask`; preserve their command and approval contracts.

## Task contracts and fixtures

Refactoring task packages contain versioned contracts, and `trusted/` trees contain revision-pinned proof inputs. Treat those bytes as immutable. A necessary change requires an intentional task/proof migration that updates its source pins, task ownership, verification inputs, and dependent tasks. Do not rewrite a historical fixture to make it match current implementation.

Generated guidance, legal notices, captures, transcripts, and structured fixtures have their own owners and formats. Preserve them unless their generator or data contract has an explicit migration.

The [bootstrap asset manifest](refactoring-plan/bootstrap-assets.tsv) pins both source files under `docs/refactoring-plan/` and their copied task destinations. Preserve both sides exactly, including Markdown protocols and standalone Rust proof fixtures; they are executable verification inputs rather than editable prose.

## Historical evidence

Historical audit records are useful only for their stated source revision and scope. Keep unresolved requirements in the active task or roadmap; do not treat old PASS results, branch names, or capture hashes as current verification.

## Frozen task-context links

Six source-relative Markdown links in the frozen TASK-070, TASK-071, and TASK-072 runner-bootstrap protocol copies point to their original task-container context (`../bootstrap-review.md` and `../proof-contract.md`). Preserve those bytes as pinned evidence. For repository navigation, use the canonical [bootstrap review](refactoring-plan/bootstrap-review.md) and [proof contract](refactoring-plan/proof-contract.md); the relative strings in the copies are historical context paths, not live repository links. The `/work/...` and `/task/...` paths in task contracts identify the task runner's mounted layout, not repository-relative links.
