---
schema: task/v5
id: TASK-004
title: "Capture all immutable Jackin oracle scenarios"
kind: test
---

# TASK-004 — Capture all immutable Jackin oracle scenarios

## Goal

Every Jackin oracle world, route, overlay, motion phase and JA action sequence has complete repeatable capture evidence.

## Context

Visual acceptance is currently blocked: this branch lacks the grouped store and PTY suite. A verifier subagent must import them read-only from the immutable `visual-baseline` tag into the external run before using [`visual-validation.md`](../../../visual-validation.md). Never write `snapshots/`, bless output, or mutate the tag.

This is a later execution task. The current planning goal only creates this immutable package. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. The accepted producer product is `oracle-jackin`. Task dependencies are `TASK-001`, `TASK-070`, `TASK-071`, `TASK-072`. Dependency status alone never proves integrated source ancestry or trusted product acceptance. Follow the subagent-only host-local workflow in proof-contract.md; use standalone taskfmt only for this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- `trusted/app-flow-contribution-contract.md`, `trusted/app-flow-contributions.tsv`, `trusted/app-flow-frame-contributions.tsv`, and `trusted/app-flow-stage-audit.tsv`; consume only this application's rows, without dropping any parent scenario.

- [docs/refactoring-plan/jackin.md](../../../../docs/refactoring-plan/jackin.md).
- [docs/refactoring-plan/jackin-scenarios.tsv](../../../../docs/refactoring-plan/jackin-scenarios.tsv).
- [refactoring-tasks/visual-validation.md](../../../../refactoring-tasks/visual-validation.md).
- `trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `trusted/obligations.md` and `../../../../docs/refactoring-plan/proof-contract.md`.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.
- **Host-local verifier inputs:** The verifier subagent resolves the comparator at `$WORKTREE/tools/refactor-proof/bin/tc-proof` and per-check contexts under `$RUN_DIR/contexts/`; taskfmt receives explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths. No mounts are involved.

## Scope

In scope:

- `tools/refactor-proof-adapters/jackin` for this task's stated product and directly necessary tests.
- Independently inspectable oracle-jackin evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Expand and independently qualify every app-specific flow contribution and source-derived direct seed under `trusted/app-flow-contribution-contract.md`, preserving every intact parent event/assertion/frame and the separate seeded-versus-PTY lane classification. Satisfy every exact clause mapped to R-001 in `trusted/source-obligations.tsv` together with the bounded outcome below. Expand all JA-001–JA-070 rows and their finite world/motion/color/viewport/error/dirty-exit variants. Capture Rituals, Manager/Prelude, Editor/Settings shared config, Accounts/Usage/1Password, Cockpit, Capsule and compact/advanced Inspect. Record numeric event traces, source-enum variant order, exact source test fixture mutations, bounded waits and checkpoint identities before candidate consumption.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Use App::for_scenario and production handle/render with the original world job queue, seed 0x4A41434B494E5E5E, epoch1788401640 and source route clock cadence. Keep Full/Reduced/Paused distinct; derive actual reachable phase ticks and flash boundaries. Preserve legitimate rain renderer, modeled-only BlockedSidecar fixture classification and direct-only private state observations.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Run the original 63 application tests and their complete assertions, plus inline/domain clock/launch/inspect/topology tests required by inventory. Preserve original helper drawing schedules and prove additional observation cannot mutate state. Repeat flattened traces from fresh worlds, including prefix/selection/split/overlay restoration and horizontal/vertical wheel ±3 equivalence.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never dynamically find candidate labels, invoke real daemon/Docker/provider/1Password/clipboard operations, represent a modeled-only case as CLI reachable, normalize the oracle's config guard differences, or record away required animation phases.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by verifier and reviewer subagents.

R-001/R-002 also require the protected observation schema and logical identity mapping consumed by candidate capture. Reference adapters remain independently accepted baseline products. Candidate extraction seams are separately owned untrusted implementation code under the proof contract; this baseline never requires a future candidate-adapter receipt or permits a candidate-defined schema.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the oracle-jackin producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed oracle-jackin implementation
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
Given the campaign-pinned task contract and protected source and verification inputs
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
- **D-002:** This task produces `oracle-jackin`. Only verifier and reviewer subagents can approve evidence after independent checks; the coordinator cannot edit receipts or task metadata.
- **D-003:** Use the existing proof-contract operation interface and canonical taskfmt standalone gate. Runner operations belong to TASK-070, accounting to TASK-071 and architecture verification to TASK-072; do not invent private bypass scripts.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Trusted candidate capture output and trusted expected oracle output occupy separate authority domains. This preparation task cannot approve changed candidate UX.

## Preparation accounting authority

All `account-tests` checks in this task use TASK-071's independently qualified preparation mode, fixed by the campaign-pinned per-check context. They do not require this task's own future accepted product or TASK-008's future disposition receipt. The verifier subagent independently discovers the exact pinned-source inventory using protected source/module/profile inputs and actual compiler listings/execution; a candidate inventory or disposition is a proposal to validate, never the membership or allowed-failure authority. The verification records the preparation expectation register before dispatch, recording exact source/test/outcome/classification and any source-evidenced pre-existing product failure. Require complete execution and unchanged compatible assertions; failures remain failed diagnostics. Missing/unknown/new failures and self-approved omissions fail. TASK-008 may use accepted TASK-007/TASK-006 products but cannot use its own proposal as a receipt. Only later production consumers require accepted inventory and disposition receipts together.

## Subagent execution

The implementer, verifier, and reviewer subagents own this task. The coordinator assigns isolated worktrees, reviews evidence, and integrates only reviewed commits; it does not edit task-owned files.

All execution is host-local. Use `$TASK_DIR` for this package, `$WORKTREE` for the isolated repository, `$RUN_DIR` for evidence and logs, and `$SCOPE_BASE` for the recorded parent. Run the latest standalone taskfmt only for this package:

```text
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

Taskfmt is validation only. No containers, images, mounts, or task orchestration commands are used. The verifier owns the final taskfmt evidence; the reviewer checks it against every `R-*`, `AC-*`, and `CHK-*` obligation before the coordinator integrates. Keep generated evidence under `$RUN_DIR` and do not modify task metadata or protected oracle inputs.


## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare the exact authority inputs.
    - [ ] **1.1** Validate source pins, prerequisites and forbidden scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Produce the bounded evidence product.
    - [ ] **2.1** Expand every JA finite axis and pinned test-helper sequence. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Qualify direct state observation and actual executable PTY lanes. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Capture all declared motion/checkpoint variants twice. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Validate exact membership and submit oracle-jackin for independent sealing. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.5** Validate exact source membership. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.6** Execute and record the owned preparation operation. (`R-001`, `AC-007`, `CHK-003`)
- [ ] **3** Verify the fixed product.
    - [ ] **3.1** Run the complete subagent-verified gate and preserve evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
