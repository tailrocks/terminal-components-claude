---
schema: task/v5
id: TASK-070
title: "Implement qualified source capture and scenario runner"
kind: feature
---

# TASK-070 — Implement qualified source capture and scenario runner

## Goal

The source/scenario runner executes preflight, required, oracle, capture and close through independently qualified production-source fixtures and isolated workers.

## Context

TASK-001 qualifies comparison and the thin host core only. This package owns the distinct `source-runner` operation implementation. Its independently frozen group70 fixture contract establishes acceptance; the implementation cannot judge itself. This package is a later execution contract, not authorization to implement terminal-components during the planning goal.

Read before editing:

- `/work/docs/refactoring-plan/proof-contract.md` and `/work/docs/refactoring-plan/architecture-adjudication.md`.
- `/task/trusted/runner-bootstrap/runner-bootstrap-protocol.md` and the exact group70 fixture/driver source.
- `/task/trusted/source-obligations.tsv`: all mapped clauses, exact historical source revisions, remaining-work and named-test obligations bind their requirement/acceptance/check IDs.
- `/task/trusted/proof-bootstrap/proof-comparator-protocol.md` and `host-bootstrap-protocol.md`.

## Preconditions

- **P-001:** The protected host validates `TASK-001` receipts and actual integrated source ancestry, the recorded candidate parent/scope base, current canonical taskfmt revision and fingerprint, and the reviewed tui-snap tool pin.
- **P-002:** The driver, fixture application, worker program, isolation observer and source requirement inputs are immutable outside the executor checkout. The submitted executable cannot alter the independent judge or its expected data.
- **P-003:** Candidate workers have the pinned offline toolchain and owned PTYs where required, with no host authority, expected artifacts, network, credentials or host socket access.

## Scope

In scope:

- `tools/refactor-proof/runner` for the stated operation and narrow directly necessary tests.
- `tools/refactor-proof/bin/tc-proof` for the stated operation and narrow directly necessary tests.
- Independent qualification outputs in host-assigned run directories; no production application changes.

Out of scope:

- Comparator/host-core redesign; application/component repairs; oracle baseline capture/approval for the real application inventory; new scheduler/monitor/services.
- Independent driver/vectors/protocol modifications, catalog changes, shared expected data, real provider side effects, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy all exact R-001 clauses in the protected source obligations. Implement exactly preflight, required, oracle, capture and close from proof-contract.md and runner-bootstrap-protocol.md. Source-only expansion freezes finite numeric events/checkpoints before candidate replay. Oracle capture rebuilds the pinned immutable source and reviewed adapter, captures twice in fresh workers, validates complete membership and exact repeated output, and never uses candidate data to choose expected output. Candidate direct/PTY capture builds the frozen source and replays fixed numeric actions through actual production handlers/rendering/executable input. Close joins actual typed result identities and complete integrity evidence; it cannot manufacture missing results.
- **R-002 (MUST):** Satisfy all exact R-002 clauses in the protected source obligations. Reuse TASK-001's independently qualified comparator and thin host without weakening their contexts, immutable tree/receipt authority or worker isolation. Capture workers receive no expected frames or host ledger. Preserve direct private-semantic versus PTY terminal-observable lanes and independently qualify logical time/observation adapters against original source blobs. A project runner is thin orchestration over existing tuisnap, not a test-only application or new scheduler.
- **R-003 (MUST):** Preserve every source-qualified assertion/test mapped to R-003. Run every group070 fixture and recovery positive against the submitted executable, covering changed oracle/source/adapter/action/profile pins, omitted/extra/duplicate membership, fake captures, reordered/missing checkpoints, stale binaries, candidate-driven selectors, mutated output/trust roots, incomplete close records and wrong tree/run/task binding. Rerun the full independently qualified comparator corpus after changing its CLI dispatcher. Existing comparator/host accepted behaviors must remain unchanged.
- **R-004 (MUST NOT):** Violate any mapped rejection, deferral or trust rule. Never add baseline blessing, relaxation, skipped-state, mutable expected source or candidate-selected identity options. Never copy rendering/business/hit-test logic into adapters, replace original fixed conditions, call the runner as its own sole qualification gate, or implement production terminal-components repairs.
- **R-005 (MUST):** The complete independently controlled gate passes on the exact frozen candidate tree with all required outputs and unchanged trusted inputs.

R-001/R-002 require candidate capture to compile extraction-only untrusted seams with the exact frozen candidate and invoke real production handlers/rendering. Baseline TASK-002–006 own the protected state schema/identity mapping; production task owners implement seams only in their declared source/completion-test scope. No future accepted candidate-adapter receipt is required and no candidate seam receives verdict authority. R-003 includes independent wrong-state, constant-state, omitted-field, wrong-source/binary and test-only-path substitution cases; actual variable actions/state and frame provenance must reject each mutant.

R-002 also preserves the shared observer transport for raw style-timing evidence: canonical JSON types and exact private observation bytes remain bound independently of a submitted digest list, and the candidate cannot choose source, timing scope, frame denominator or observer argv. TASK-072 CHK-012 qualifies the later architecture operation and actual timing semantics against protected pinned-main source mutations; TASK-070 does not require that not-yet-implemented operation or a future TASK-073 receipt. TASK-073 produces the real measurement implementation only after those prerequisites, and TASK-067 consumes its independently observed results.

## Acceptance criteria

### AC-001 — Qualified source-runner behavior
```gherkin
Given independently authored group70 positive negative and recovery fixtures
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
When the group70 authority and ownership probes execute against the submitted implementation
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

### AC-006 — Preserve exact comparison
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-008`

## Fixed decisions

- **D-001:** The independently reviewed runner-bootstrap group70 driver and its fixtures judge this product. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No taskfmt dispatcher/monitor/promote call is allowed; no ref update targets main.
- **D-004:** Host accepts `source-runner` only after independent qualification and source rebuild. A later task resolves exactly that accepted producer product; no mutable latest path or candidate-written receipt is valid.
- **D-005:** All commands and result schemas are exactly the frozen proof contract. Unsupported required behavior fails; adding permissive flags or alternative expected data cannot unblock it.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Validate the accepted foundation.
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Implement only the assigned operation group.
    - [ ] **2.1** Implement the complete group70 observable behavior. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve isolation, identity and ownership boundaries. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Retain the accepted comparator and every required regression. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Preserve exact comparison with its independent corpus. (`R-003`, `AC-006`, `CHK-008`)
- [ ] **3** Qualify the frozen executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
