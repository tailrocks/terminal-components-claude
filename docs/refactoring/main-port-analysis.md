# Split-crate migration and porting rules

This page governs how implementation work moves from the monolithic WIP into the split workspace. It is a current migration contract, not a branch comparison report or a record of old test results.

## Current source shape

The WIP root manifest defines one `junie-tui` package and `junie_tui` library, with Showcase, TablePro, Jackin preview, and Holla binaries under `src/bin/`. Shared modules and app-family code remain in this package. Verify the exact state from the current root manifest and source tree.

## Structural direction

`main` provides the split-crate direction: reusable `termrock-*` foundation and component packages, separate app-family packages, and explicit test/proof/tooling packages. The current split contains 46 workspace members and 50 Cargo manifests including the root and non-member manifests. This directory structure does not prove implementation completeness, parity, or readiness; `main` remains incomplete, broken, and not ready.

`design/interaction-polish` is the exact UI/UX and behavior target. Do not treat a main-branch API or rendering difference as permission to change that target.

## Porting rules

1. Read the owning requirements in [component architecture](../architecture/component-architecture.md), [shared component plan](../architecture/shared-components-plan.md), [remaining work](../remaining-work.md), and the relevant task package before changing code.
2. Assign shared APIs to their actual target package, based on current manifests and imports. Keep app-domain vocabulary, workflow state, and simulations in app-family packages.
3. Reuse an API idea only if it fits the target boundary and satisfies the design contract. Do not copy an entire source tree or preserve a second implementation just to match an old branch layout.
4. Migrate one scoped behavior at a time. Preserve stable identity, controlled state, focus and input policy, terminal lifecycle, and semantic theme behavior through the reusable owner.
5. Verify the integrated result with task-specific unit and integration tests, architecture checks, and matched visual scenarios. Record the exact source revision and evidence scope; compilation alone establishes neither parity nor migration readiness.

## Active work ownership

Component and state contracts live in the architecture and shared-component plans. Application and cross-component work lives in the branch-specific remaining-work ledger and its linked task packages. Historical branch-diff ledgers remain source evidence for their listed clauses; they do not define current package paths or status.
