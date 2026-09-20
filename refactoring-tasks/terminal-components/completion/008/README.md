---
schema: task/v5
id: TASK-008
title: "Replace oracle-conflicting test authority with protected staged assertions"
kind: test
---

# TASK-008 — Replace oracle-conflicting test authority with protected staged assertions

## Goal

Conflicting product tests use trusted oracle authority while every intermediate task has explicit complete-execution failure accounting and preserved compatible assertions.

## Context

Visual acceptance is currently blocked: this branch lacks the grouped store and PTY suite. A verifier subagent must import them read-only from the immutable `visual-baseline` tag into the external run before using [`visual-validation.md`](../../../visual-validation.md). Never write `snapshots/`, bless output, or mutate the tag.

This is a later execution task. The current planning goal only creates this immutable package. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. The accepted producer product is `test-disposition`. Task dependencies are `TASK-007`, `TASK-006`, `TASK-071`, `TASK-072`. TASK-001 and TASK-070 stay retired fail-closed and cannot supply accepted producer receipts. Former host/runner/lifecycle obligations now live in `scripts/campaign-build-proof.sh`, [`path-contract.md`](../../../../docs/refactoring-plan/path-contract.md), [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md), and `tools/refactor-proof/{src,runner,bin}`. Dependency status alone never proves integrated source ancestry or trusted product acceptance. Follow the subagent-only host-local workflow in proof-contract.md; use standalone taskfmt only for this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [docs/refactoring-plan/proof-contract.md](../../../../docs/refactoring-plan/proof-contract.md).
- [docs/refactoring-plan/task-graph.md](../../../../docs/refactoring-plan/task-graph.md).
- [docs/refactoring-plan/holla.md](../../../../docs/refactoring-plan/holla.md).
- [docs/refactoring-plan/showcase.md](../../../../docs/refactoring-plan/showcase.md).
- [docs/refactoring-plan/tablepro.md](../../../../docs/refactoring-plan/tablepro.md).
- `trusted/source-obligations.tsv`: every mapped clause, remaining-work obligation and named test is binding under its requirement/acceptance/check IDs.
- `trusted/obligations.md`, `trusted/inline-test-source-scope.md` and `../../../../docs/refactoring-plan/proof-contract.md`.
- `trusted/branch-test-disposition-bindings.tsv` and `trusted/external-test-source-scope.md`: components-B disposition authority and external test span targets.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Every declared predecessor's accepted product and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The source requirements and finite scenario expansion contract are immutable. Expected numeric traces and capture hashes are outputs of the authorized baseline producer, never fabricated preconditions.
- **Host-local verifier inputs:** The verifier subagent resolves the comparator at `$WORKTREE/tools/refactor-proof/bin/tc-proof` and per-check contexts under `$RUN_DIR/contexts/`; taskfmt receives explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths. No mounts are involved.

## Scope

In scope:

- `tools/test-disposition` for this task's stated product and directly necessary tests.
- `crates/tui/tests` for this task's stated product and directly necessary tests.
- `apps/showcase/tests` for this task's stated product and directly necessary tests.
- `apps/holla/tests` for this task's stated product and directly necessary tests.
- `apps/jackin-preview/tests` for this task's stated product and directly necessary tests.
- `apps/tablepro/tests` for this task's stated product and directly necessary tests.
- `archives/test-authority` for this task's stated product and directly necessary tests.
- Independently inspectable test-disposition evidence and exact provenance; write generated run outputs only to host-assigned directories.

Out of scope:

- Production component or application repairs, product redesign, real provider operations, branch merging, publication, and unrelated tooling changes.
- Changes to task packages, independent bootstrap tests, the oracle commit, protected expected artifacts or accepted products owned by earlier tasks.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/source-obligations.tsv` together with the bounded outcome below. For every oracle-conflicting current test, record exact original blob/expectation/source SHA, decisive oracle scenario, retained architecture assertions, replacement identity and canonical correction owner. Replace only conflicting product assertions using independently accepted oracle data; preserve original bytes in the immutable historical archive. Produce a fixed task-stage map of still-unfinished future-owned identities and exact closing task.
- **R-002 (MUST):** Satisfy every exact source clause mapped to R-002 in the protected source obligations. Keep compatible ownership/safety/backend/API tests active and passing; preserve primary/MSRV assignments. Reconcile Holla Ctrl+A/scope-first/frame-ms conflicts, old22-pageShowcase authority, TablePro Surface/confirmation conflicts, Meter changedpins, and any additional evidenced conflict from TASK-007. No individual component/app coordinator can edit dispositions, required memberships, expected frames or stage policy.
- **R-003 (MUST):** Preserve every source-qualified assertion and test mapped to R-003 in the protected source obligations. Execute the complete required inventory with no-fail-fast and record every actual failure. Qualify unknown failure, missing/filtered/ignored test, renamed identity without relocation, changed classification, previously closed failure and candidate-edited stage-map rejection. A future-owner failure remains visibly failed and must close monotonically by its owner; app closure and final stages have empty relevant unresolved sets.
- **R-004 (MUST NOT):** Violate any rejection, deferral, non-goal or trust rule mapped to R-004 in the protected source obligations. Never skip or weaken a failing test to obtain green output, replace architecture assertions merely because product text changes, let candidate frames generate expected output, add broad tolerances, treat diagnostic failures as passing parity, or allow any unresolved identity at TASK-069.
- **R-005 (MUST):** The complete authoritative gate succeeds with all actual check logs, protected-input integrity and the exact accepted source tree recorded by verifier and reviewer subagents.

R-001/R-002 additionally bind `trusted/inline-test-source-scope.md`. R-001/R-002 additionally bind `trusted/branch-test-disposition-bindings.tsv` and `trusted/external-test-source-scope.md` for components-B conflicting starting tests (W-021-07, W-025-07/08, W-026-05). The exact listed source files are writable only for reviewer-approved test assertion spans, never production behavior. Before candidate edits, independently review proposed replacements against accepted TASK-007 identity evidence and TASK-006 oracle data and record the patch manifest. CHK-006 checks unchanged bytes outside approved spans, preserved module/cfg/ignore structure and compatible assertions, plus exact original archive bytes. Merely being in verify.toml's file scope grants no other mutation. Holla app hint/Escape assertions and scenario world-count assertions must be migrated through these exact rules.

R-001/R-003 require the coordinator, before coordinator dispatch, to apply that exact reviewed span-patch manifest independently to a disposable recorded-parent checkout, verify the unchanged production/outside-span projection, and execute the complete inventory under pinned profiles without fail-fast omissions. Preserve pre-patch results separately; record the post-approved-test-migration register with original parent/tree, patch digest, independently derived patched tree and full execution evidence. Every newly exposed oracle-conflict failure needs exact identity/outcome/classification, source/oracle evidence and canonical correction/closing owner. CHK-004/005 accept those precise failed diagnostics while rejecting unapproved new failures, altered classification, missing execution, parent/patch swaps or compatible-assertion regressions. Neither candidate output nor this task's future receipt can authorize the register. R-003 requires the independently qualified approved-new-oracle-failure positive and unapproved-failure/patch-swap negatives.

## Acceptance criteria

### AC-001 — Complete owned product
```gherkin
Given the pinned source inputs and accepted prerequisite receipts
When the test-disposition producer executes the exact work in R-001
Then every required identity and observation is present and its independent verification passes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve the architectural boundary
```gherkin
Given the completed test-disposition implementation
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
When the complete required test inventory is executed and accounted for
Then every owned product artifact binds the correct source and complete actual execution
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** The UI oracle and architecture source have separate authority; never substitute main output for an expected frame.
- **D-002:** This task produces `test-disposition`. Only verifier and reviewer subagents can approve evidence after independent checks; the coordinator cannot edit receipts or task metadata.
- **D-003:** Use the existing proof-contract operation interface and canonical taskfmt standalone gate. Native comparator/runner/close live in `tools/refactor-proof/{src,runner,bin}` and `scripts/campaign-build-proof.sh`; accounting qualification is TASK-071 and architecture verification is TASK-072. TASK-001 and TASK-070 remain retired fail-closed packages. Do not invent private bypass scripts.
- **D-004:** Preserve original source/fixture/test identities and exact finite membership. Artifact absence, invalid data and unimplemented required cases fail closed.
- **D-005:** Trusted candidate capture output and trusted expected oracle output occupy separate authority domains. This preparation task cannot approve changed candidate UX.
- **D-006:** Components-B branch disposition rows in `trusted/branch-test-disposition-bindings.tsv` are immutable TASK-008 authority. Inline spans use `inline-test-source-scope.md`; external modules use `external-test-source-scope.md`.

## Preparation accounting authority

All `account-tests` checks in this task use TASK-071's independently qualified preparation mode, fixed by the campaign-pinned per-check context. They do not require this task's own future accepted product or disposition receipt. The verifier subagent independently discovers the exact pinned-source inventory using protected source/module/profile inputs and actual compiler listings/execution; a candidate inventory or disposition is a proposal to validate, never membership or allowed-failure authority. Before dispatch the coordinator records the post-approved-test-migration expectation register from the independently patched recorded-parent checkout described above, preserving pre-patch results separately. Do not demand the obsolete main outcome vector after correct oracle assertions replace it. Require complete execution and unchanged compatible assertions; precisely evidenced newly exposed oracle failures remain failed diagnostics with fixed closing owners. Missing/unknown/unapproved new failures and self-approved omissions fail. TASK-008 consumes accepted TASK-007/TASK-006 inputs, never its own proposal as a receipt. Only later production consumers require accepted inventory and disposition receipts together.

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
    - [ ] **2.1** Resolve accepted complete oracle and exact test-inventory receipts. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Review each conflicting expectation and create protected source-to-replacement records. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Install oracle-derived assertions and strict stage accounting without production repair. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Execute complete inventory and qualify rejection of every unauthorized failure or policy mutation. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.5** Validate exact source membership. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.6** Execute and record the owned preparation operation. (`R-001`, `AC-007`, `CHK-003`)
- [ ] **3** Verify the fixed product.
    - [ ] **3.1** Run the complete subagent-verified gate and preserve evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
