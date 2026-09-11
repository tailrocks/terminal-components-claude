---
schema: task/v5
id: TASK-001
title: "Implement independently qualified comparator and host core"
kind: feature
---

# TASK-001 — Implement independently qualified comparator and host core

## Goal

A separately installed tc-proof comparator and tc-proof-host core pass independent comparator, host isolation, exact-tree verification and integration qualification.

## Context

This is a later execution task. The current planning goal only creates this immutable package. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. The accepted producer product is `qualified-harness`. Task dependencies are none; independently qualified external tooling and planner bootstrap are host preconditions. Dependency status alone never proves integrated source ancestry or trusted product acceptance. Follow the standalone host workflow in proof-contract.md; do not execute taskfmt's main-only dispatcher or promotion.

Read before editing:

- [docs/refactoring-plan/proof-contract.md](/work/docs/refactoring-plan/proof-contract.md).
- [docs/refactoring-plan/evidence/proof-comparator-protocol.md](/work/docs/refactoring-plan/evidence/proof-comparator-protocol.md).
- [docs/refactoring-plan/evidence/host-bootstrap-protocol.md](/work/docs/refactoring-plan/evidence/host-bootstrap-protocol.md).
- `/task/trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `/task/trusted/obligations.md` and `/work/docs/refactoring-plan/proof-contract.md`.

## Preconditions

- **P-001:** The host has pinned the immutable task/catalog/bootstrap, taskfmt revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, accepted tui-snap PR/revision and toolchain/image/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected host receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.

## Scope

In scope:

- `tools/refactor-proof` for this task's stated product and directly necessary tests.
- Independently inspectable qualified-harness evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `/task/trusted/source-obligations.tsv` together with the bounded outcome below. Implement only tc-proof compare and the thin host interfaces in proof-contract.md under tools/refactor-proof/. The comparator must pass all 72 independently authored vectors and 69 fresh positive recoveries. The host must pass every independently authored install/prepare/freeze/verify/seal-rejection/integrate case, including hostile workers and actual standalone taskfmt. A missing or unsupported isolation capability is a failing task, not an accepted limitation.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Keep project orchestration outside junie-tui and tui-snap. Use existing tuisnap schema-3 capture and exact comparison capabilities, pinned standalone taskfmt, protected host authority, immutable file receipts and local integration compare-and-swap. Do not build a scheduler, monitor, receipt service or taskfmt schema extension. Candidate build/test code runs in separate workers without expected artifacts, credentials, network or trust-root access.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Qualify nonzero exit with printed DONE, omitted logs/checks, forged receipt, wrong producer, unintegrated prerequisite, mutable source/index/symlink/hard-link attacks, changed parent, refs/heads/main, missing expected files, incomplete test execution and temporary trust writes restored afterward. Every applicable rejection preserves independently inspected refs and trusted bytes and is followed by a fresh passing case.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never qualify either executable solely through its own tests or output claims. Never create a real terminal-components integration commit, change production code, seal a real oracle bundle, invoke taskfmt run/monitor/promote, alter the independent driver/vectors, or claim synthetic fixtures are product UX evidence.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by the host.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the qualified-harness producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed qualified-harness implementation
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

## Fixed decisions

- **D-001:** The UI oracle and architecture source have separate authority; never substitute main output for an expected frame.
- **D-002:** This task produces `qualified-harness`. Only the protected host can accept/seal its output after independent checks; the executor cannot edit receipts or task metadata.
- **D-003:** Use the existing proof-contract operation interface and canonical taskfmt standalone gate. Source/scenario runner operations are owned by TASK-070, accounting by TASK-071 and architecture verification by TASK-072. Those are distinct independently qualified prerequisites; do not claim this core task establishes them.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Independent bootstrap drivers judge the submitted executables; the submitted proof tool cannot be its own sole completion gate.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare the exact authority inputs.
    - [ ] **1.1** Validate source pins, prerequisites and forbidden scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Produce the bounded evidence product.
    - [ ] **2.1** Implement strict comparison contexts, manifest validation and exact frame/semantic/provenance comparison. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Implement isolated workers, protected receipt resolution, complete source freeze and external standalone taskfmt verification. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Implement accepted binary installation and local compare-and-swap integration with DCO/coauthor commits in synthetic fixtures. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Run independent comparator and host qualification, including recovery positives and hostile write/read attempts. (`R-001`, `AC-001`, `CHK-004`)
- [ ] **3** Verify the frozen product.
    - [ ] **3.1** Run the complete host-controlled gate and preserve evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
