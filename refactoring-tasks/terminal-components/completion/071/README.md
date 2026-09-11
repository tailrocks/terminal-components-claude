---
schema: task/v5
id: TASK-071
title: "Implement qualified exact test execution accounting"
kind: feature
---

# TASK-071 — Implement qualified exact test execution accounting

## Goal

The accounting operation executes every required source-qualified test and enforces exact immutable stage ownership without hiding a failure.

## Context

TASK-001 qualifies comparison and the thin host core only. This package owns the distinct `test-accounting` operation implementation. Its independently frozen group71 fixture contract establishes acceptance; the implementation cannot judge itself. This package is a later execution contract, not authorization to implement terminal-components during the planning goal.

Read before editing:

- `/work/docs/refactoring-plan/proof-contract.md` and `/work/docs/refactoring-plan/architecture-adjudication.md`.
- `/task/trusted/runner-bootstrap/runner-bootstrap-protocol.md` and the exact group71 fixture/driver source.
- `/task/trusted/source-obligations.tsv`: all mapped clauses, exact historical source revisions, remaining-work and named-test obligations bind their requirement/acceptance/check IDs.
- `/task/trusted/proof-bootstrap/proof-comparator-protocol.md` and `host-bootstrap-protocol.md`.

## Preconditions

- **P-001:** The protected host validates `TASK-070` receipts and actual integrated source ancestry, the recorded candidate parent/scope base, current canonical taskfmt revision and fingerprint, and the reviewed tui-snap tool pin.
- **P-002:** The driver, fixture application, worker program, isolation observer and source requirement inputs are immutable outside the executor checkout. The submitted executable cannot alter the independent judge or its expected data.
- **P-003:** Candidate workers have the pinned offline toolchain and owned PTYs where required, with no host authority, expected artifacts, network, credentials or host socket access.

## Scope

In scope:

- `tools/refactor-proof/accounting` for the stated operation and narrow directly necessary tests.
- `tools/refactor-proof/bin/tc-proof` for the stated operation and narrow directly necessary tests.
- Independent qualification outputs in host-assigned run directories; no production application changes.

Out of scope:

- Comparator/host-core redesign; application/component repairs; oracle baseline capture/approval for the real application inventory; new scheduler/monitor/services.
- Independent driver/vectors/protocol modifications, catalog changes, shared expected data, real provider side effects, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy all exact R-001 clauses in the protected source obligations. Implement account-tests using exact source SHA/package/target/test/profile/feature identities, no-fail-fast execution and complete subordinate result accounting. Run every required identity, report its actual pass/fail/error outcome, reject missing/duplicate/filtered/ignored/unmapped identities, and preserve primary/MSRV assignments. Admit only the frozen stage's still-unfinished future-owner failures minus all already closed identities; every failure remains reported as failed. At app/final closure the relevant unresolved set must be empty.
- **R-002 (MUST):** Satisfy all exact R-002 clauses in the protected source obligations. In production mode, consume trusted inventory and disposition producer receipts rather than caller-selected test filters or an editable allowed-failure file. Test listing is discovery evidence only; actual executed test results are required. Preserve exact relocation maps and all compatible architecture/safety assertions. The accounting operation cannot change source tests, historical archives, public components or accepted oracle artifacts.
- **R-003 (MUST):** Preserve every source-qualified assertion/test mapped to R-003. Pass every group071 independent fixture and recovery positive: missing target/results, duplicate identity, renamed test without relocation, early runner exit, aggregate success with absent subordinate execution, unknown failure, changed failure class, closed scenario regression, forged stage policy, wrong source/profile/run and legitimate future-owner failures reported honestly. Preserve all prior runner and comparator accepted behavior after dispatcher extension.
- **R-004 (MUST NOT):** Violate any mapped rejection, deferral or trust rule. Never interpret skipped/ignored/listed-only tests as executed passes, silence failure output, weaken expected assertions, reopen a closed obligation, invent a replacement profile, or permit unresolved entries at final integration.
- **R-005 (MUST):** The complete independently controlled gate passes on the exact frozen candidate tree with all required outputs and unchanged trusted inputs.

## Preparation and production modes

R-001/R-002 require two independently qualified, host-selected accounting modes with the same command ABI. `preparation` is restricted to TASK-002–008: use independently discovered pinned-source/compiler inventory and a protected pre-dispatch preparation expectation register, not future accepted TASK-007/008 receipts. Proposed inventory/dispositions are untrusted comparison subjects. Exact known source failures remain failed observations; no proposal can omit an execution, broaden an admitted failure or approve its own source patch. TASK-008's independently reviewed test-only patch manifest binds original blobs, exact assertion spans, oracle evidence and retained assertions before candidate editing; all outside-span bytes remain unchanged. `production` requires both accepted inventory and disposition receipts and the monotonic closed-set ledger; preparation authority cannot be selected by a production task.

R-003 additionally requires producer-before-receipt positives and self-approved inventory/disposition, omitted/renamed execution, forged preparation-mode selection, widened test span, removed compatible assertion, cfg/ignore suppression, changed archive and outside-span edit negatives. These are independent group071 qualification cases, not candidate-authored tests.

## Acceptance criteria

### AC-001 — Qualified test-accounting behavior
```gherkin
Given independently authored group71 positive negative and recovery fixtures
When the submitted executable runs every operation case required by R-001
Then every actual result and protected filesystem effect matches the independently specified outcome
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-002 — Preserve operation authority and boundaries
```gherkin
Given isolated candidate workers and the protected source and receipt inputs
When the group71 authority and ownership probes execute against the submitted implementation
Then the R-002 boundaries hold and no candidate claim substitutes for independent evidence
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Preserve accepted comparison and exact regression behavior
```gherkin
Given the accepted comparator corpus and this task's source-qualified regression obligations
When the dispatcher extension is checked against independent positive and negative inputs
Then accepted comparison behavior remains exact and no required regression is omitted or relabeled
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Immutable prerequisites and forbidden paths
```gherkin
Given the immutable task bootstrap receipts and recorded source base
When the protected preflight validates actual tools inputs scope and prerequisite authority
Then no R-004 prohibited action or candidate-controlled definition of success is admitted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Preserve accepted operation group70
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-008`

### AC-007 — Preserve exact comparison
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-009`

## Fixed decisions

- **D-001:** The independently reviewed runner-bootstrap group71 driver and its fixtures judge this product. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No taskfmt dispatcher/monitor/promote call is allowed; no ref update targets main.
- **D-004:** Host accepts `test-accounting` only after independent qualification and source rebuild. A later task resolves exactly that accepted producer product; no mutable latest path or candidate-written receipt is valid.
- **D-005:** All commands and result schemas are exactly the frozen proof contract. Unsupported required behavior fails; adding permissive flags or alternative expected data cannot unblock it.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Validate the accepted foundation.
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Implement only the assigned operation group.
    - [ ] **2.1** Implement the complete group71 observable behavior. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve isolation, identity and ownership boundaries. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Retain the accepted comparator and every required regression. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Preserve accepted group70 with its independent corpus. (`R-003`, `AC-006`, `CHK-008`)
    - [ ] **2.5** Preserve exact comparison with its independent corpus. (`R-003`, `AC-007`, `CHK-009`)
- [ ] **3** Qualify the frozen executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
