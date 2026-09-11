---
schema: task/v5
id: TASK-006
title: "Capture reusable component states and seal complete baseline"
kind: test
---

# TASK-006 — Capture reusable component states and seal complete baseline

## Goal

Every reusable component family and all accepted application bundles form one complete, independently accepted oracle contract.

## Context

This is a later execution task. The current planning goal only creates this immutable package. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. The accepted producer product is `oracle-complete`. Task dependencies are `TASK-002`, `TASK-003`, `TASK-004`, `TASK-005`, `TASK-070`, `TASK-071`, `TASK-072`. Dependency status alone never proves integrated source ancestry or trusted product acceptance. Follow the standalone host workflow in proof-contract.md; do not execute taskfmt's main-only dispatcher or promotion.

Read before editing:

- [docs/refactoring-plan/components.md](/work/docs/refactoring-plan/components.md).
- [docs/refactoring-plan/component-parity.tsv](/work/docs/refactoring-plan/component-parity.tsv).
- [docs/refactoring-plan/parity-synthesis.md](/work/docs/refactoring-plan/parity-synthesis.md).
- [docs/refactoring-plan/verification.md](/work/docs/refactoring-plan/verification.md).
- `/task/trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `/task/trusted/obligations.md` and `/work/docs/refactoring-plan/proof-contract.md`.

## Preconditions

- **P-001:** The host has pinned the immutable task/catalog/bootstrap, taskfmt revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, accepted tui-snap PR/revision and toolchain/image/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected host receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.

## Scope

In scope:

- `tools/refactor-proof-adapters/components` for this task's stated product and directly necessary tests.
- Independently inspectable oracle-complete evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `/task/trusted/source-obligations.tsv` together with the bounded outcome below. Classify all 54 component rows. Expand every oracle-renderable row into actual production widget/view fixtures, preserving the common states and CP-specific corpus. The `testing-registry` row is architecture-only: CHK-002 must retain its explicit non-frame disposition and bind its attribution/conformance obligations to TASK-073 and TASK-031. It has no invented oracle screenshot, direct capture, or PTY identity. Cover default/focus/hover/pressed/selected/disabled/readiness, all source-supported capability levels, tiny/nonzero origins, exact resize thresholds, scroll/fades, Unicode/selection/cursor, overlays and semantic identities. Explicit evidence-backed non-applicability replaces impossible states; absence never does.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Use original widget code and separately reviewed oracle observation adapters; keep architecture-only Paper/API tests distinct from oracle-Junie's product frames. Components introduced only by the new architecture are proven through matching oracle compositions, with an explicit old-to-new semantic mapping and architecture fixture; do not invent an old component screenshot.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Resolve accepted oracle-showcase/holla/jackin/tablepro receipts, verify all source/tool/adapter/required-set pins, and join every namespace without duplicate/missing/checkpoint IDs. Require two independent component captures and repeat equality plus exact union coverage of every APP and COMP obligation. Include fade heights3/4/11/12,55%/80% rounding, non-RGB DIM, protected rows/cursor, styled-wide continuation and all retained-output mutation cases.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never seal a complete product while an app bundle or required component state is missing. Never infer parity from2560main digests, the499historical archive, hand-painted expected fixtures or candidate-specific semantics. Do not modify accepted app bundles or shared comparator/expansion rules.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by the host.

R-001/R-002 also require the protected observation schema and logical identity mapping consumed by candidate capture. Reference adapters remain independently accepted baseline products. Candidate extraction seams are separately owned untrusted implementation code under the proof contract; this baseline never requires a future candidate-adapter receipt or permits a candidate-defined schema.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the oracle-complete producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed oracle-complete implementation
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
When the original production oracle is captured and independently replayed
Then every owned product artifact binds the correct source and complete actual execution
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** The UI oracle and architecture source have separate authority; never substitute main output for an expected frame.
- **D-002:** This task produces `oracle-complete`. Only the protected host can accept/seal its output after independent checks; the executor cannot edit receipts or task metadata.
- **D-003:** Use the existing proof-contract operation interface and canonical taskfmt standalone gate. Runner operations belong to TASK-070, accounting to TASK-071 and architecture verification to TASK-072; do not invent private bypass scripts.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Trusted candidate capture output and trusted expected oracle output occupy separate authority domains. This preparation task cannot approve changed candidate UX.
- **D-006:** Apply the finite source-witness binding in `trusted/obligations.md`: seal all twenty-four component witness files and their fixed contracts, retain each explicit lane and stage, and reject missing or altered membership. Architecture-only cases bind future owner predicates without inventing oracle frames or requiring successor receipts.

## Preparation accounting authority

All `account-tests` checks in this task use TASK-071's independently qualified preparation mode, fixed by the host-owned per-check context. They do not require this task's own future accepted product or TASK-008's future disposition receipt. The host independently discovers the exact pinned-source inventory using protected source/module/profile inputs and actual compiler listings/execution; a candidate inventory or disposition is a proposal to validate, never the membership or allowed-failure authority. The host freezes the preparation expectation register before dispatch, recording exact source/test/outcome/classification and any source-evidenced pre-existing product failure. Require complete execution and unchanged compatible assertions; failures remain failed diagnostics. Missing/unknown/new failures and self-approved omissions fail. TASK-008 may use accepted TASK-007/TASK-006 products but cannot use its own proposal as a receipt. Only later production consumers require accepted inventory and disposition receipts together.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare the exact authority inputs.
    - [ ] **1.1** Validate source pins, prerequisites and forbidden scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Produce the bounded evidence product.
    - [ ] **2.1** Resolve four accepted application bundle receipts and validated source requirements. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Materialize source-derived component states and honest old/new semantic mappings. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Capture/replay all component actions and validate family/state coverage. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Join exact accepted bundle hashes and submit complete oracle receipt for independent sealing. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.5** Validate exact source membership. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.6** Execute and record the owned preparation operation. (`R-001`, `AC-007`, `CHK-003`)
- [ ] **3** Verify the frozen product.
    - [ ] **3.1** Run the complete host-controlled gate and preserve evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
