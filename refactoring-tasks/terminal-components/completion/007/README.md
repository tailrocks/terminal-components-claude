---
schema: task/v5
id: TASK-007
title: "Reconcile exact historical and oracle test identities"
kind: test
---

# TASK-007 — Reconcile exact historical and oracle test identities

## Goal

Every accepted historical and immutable-oracle test obligation resolves to an exact executable target/profile/test identity and reviewed preservation disposition.

## Context

This is a later execution task. The current planning goal only creates this immutable package. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. The accepted producer product is `test-inventory`. Task dependencies are `TASK-001`, `TASK-070`, `TASK-071`, `TASK-072`. Dependency status alone never proves integrated source ancestry or trusted product acceptance. Follow the standalone host workflow in proof-contract.md; do not execute taskfmt's main-only dispatcher or promotion.

Read before editing:

- [docs/refactoring-plan/history.md](/work/docs/refactoring-plan/history.md).
- [docs/refactoring-plan/historical-obligations-canonical.tsv](/work/docs/refactoring-plan/historical-obligations-canonical.tsv).
- [docs/refactoring-plan/architecture.md](/work/docs/refactoring-plan/architecture.md).
- [tools/test-inventory/README.md](/work/tools/test-inventory/README.md).
- [Pinned inline-test source scope](/work/docs/refactoring-plan/inline-test-source-scope.md), including complete-discovery and no-production-mutation limits.
- `/task/trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `/task/trusted/obligations.md` and `/work/docs/refactoring-plan/proof-contract.md`.

## Preconditions

- **P-001:** The host has pinned the immutable task/catalog/bootstrap, taskfmt revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, accepted tui-snap PR/revision and toolchain/image/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected host receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.

## Scope

In scope:

- `tools/test-inventory` for this task's stated product and directly necessary tests.
- Independently inspectable test-inventory evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `/task/trusted/source-obligations.tsv` together with the bounded outcome below. Reconcile the 3,211 previously inventoried historical test identities with all newly pinned oracle tests and the complete 620-row historical union. Record source commit, package, binary/test target, fully qualified test name, feature/toolchain profile and actual execution command. Preserve independently meaningful requirements even when source rows are duplicate-equivalent. Generate exact required inventory and fail absent targets, zero matches, missing results or unapproved name relocation.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Reuse tools/test-inventory and the qualified tc-proof account-tests protocol; source parsing discovers identities but never substitutes for execution. Separate accepted primary trybuild stderr owner from MSRV compilation/behavior. Keep historical archive provenance and test relocation identities explicit; counts alone are insufficient.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Exercise strict inventory parsing, missing/duplicate/renamed target rejection, filtered/ignored/no-fail-fast omissions, stale profile and empty allowlist failures. Preserve exact named allocation/workload tests and original assertions; document main-versus-oracle product conflicts for TASK-008 rather than changing their expectations here.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never approve required.json by counting names alone, drop supplemental historical clauses, treat old PASS counts as new execution, restore obsolete clone implementation to satisfy a name, or change production code or oracle expected artifacts.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by the host.

R-001 additionally requires complete inline-test discovery across every crate/app source tree, including external cfg(test) modules and macro/generated tests, reconciled against actual compiled listings under every required profile. Seed locations are the protected `inline-test-source-scope.md`; every discovered assertion is classified preserve or exact oracle conflict, with source blob/test/span identity for TASK-008. Discovery never grants edit or expected-outcome authority.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the test-inventory producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed test-inventory implementation
When its production call paths and exact ownership invariants in R-002 are checked
Then every declared boundary holds without a substituted renderer or self-approved verdict
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Reject omissions and regressions
```gherkin
Given the required source-qualified regression inventory and negative qualification inputs
When the complete R-003 execution runs with all outcomes recorded
Then every required case passes its specified outcome and no execution or result is silently omitted
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Keep authority immutable
```gherkin
Given the host-owned task contract and protected source and verification inputs
When source identity ownership scope and receipt authority are validated
Then every R-004 prohibition holds and no candidate-selected artifact defines success
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Validate exact required membership
```gherkin
Given the protected source-level scenario and test requirement inputs
When every finite namespace and expansion obligation is checked
Then no required identity is missing duplicated or selected by candidate output
```

**Verification**

- **Type:** invariant
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — Execute the owned preparation operation
```gherkin
Given the exact pinned source and accepted producer prerequisites
When the complete required test inventory is executed and accounted for
Then every owned product artifact binds the correct source and complete actual execution
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** The UI oracle and architecture source have separate authority; never substitute main output for an expected frame.
- **D-002:** This task produces `test-inventory`. Only the protected host can accept/seal its output after independent checks; the executor cannot edit receipts or task metadata.
- **D-003:** Use the existing proof-contract operation interface and canonical taskfmt standalone gate. Runner operations belong to TASK-070, accounting to TASK-071 and architecture verification to TASK-072; do not invent private bypass scripts.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Trusted candidate capture output and trusted expected oracle output occupy separate authority domains. This preparation task cannot approve changed candidate UX.

## Preparation accounting authority

All `account-tests` checks in this task use TASK-071's independently qualified preparation mode, fixed by the host-owned per-check context. They do not require this task's own future accepted product or TASK-008's future disposition receipt. The host independently discovers the exact pinned-source inventory using protected source/module/profile inputs and actual compiler listings/execution; a candidate inventory or disposition is a proposal to validate, never the membership or allowed-failure authority. The host freezes the preparation expectation register before dispatch, recording exact source/test/outcome/classification and any source-evidenced pre-existing product failure. Require complete execution and unchanged compatible assertions; failures remain failed diagnostics. Missing/unknown/new failures and self-approved omissions fail. TASK-008 may use accepted TASK-007/TASK-006 products but cannot use its own proposal as a receipt. Only later production consumers require accepted inventory and disposition receipts together.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare the exact authority inputs.
    - [ ] **1.1** Validate source pins, prerequisites and forbidden scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Produce the bounded evidence product.
    - [ ] **2.1** Discover exact historical/main/oracle test identities and execution profiles. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Reconcile accepted/superseded/duplicate obligations without orphaning source rows. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Qualify strict complete-execution accounting against independent failure probes. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Submit immutable test-inventory product and explicit conflict inputs for TASK-008. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.5** Validate exact source membership. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.6** Execute and record the owned preparation operation. (`R-001`, `AC-007`, `CHK-003`)
- [ ] **3** Verify the frozen product.
    - [ ] **3.1** Run the complete host-controlled gate and preserve evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
