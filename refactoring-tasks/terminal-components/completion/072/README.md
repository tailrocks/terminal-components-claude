---
schema: task/v5
id: TASK-072
title: "Implement qualified architecture evidence verification"
kind: feature
---

# TASK-072 — Implement qualified architecture evidence verification

## Goal

A protected architecture operation executes source-qualified invariants and rejects independently planted ownership/scanner mutants before production repair begins.

## Context

TASK-001 qualifies comparison and the thin host core only. This package owns the distinct `architecture-verifier` operation implementation. Its independently frozen group72 fixture contract establishes acceptance; the implementation cannot judge itself. This package is a later execution contract, not authorization to implement terminal-components during the planning goal.

Read before editing:

- `/work/docs/refactoring-plan/proof-contract.md` and `/work/docs/refactoring-plan/architecture-adjudication.md`.
- `/task/trusted/runner-bootstrap/runner-bootstrap-protocol.md` and the exact group72 fixture/driver source.
- `/task/trusted/source-obligations.tsv`: all mapped clauses, exact historical source revisions, remaining-work and named-test obligations bind their requirement/acceptance/check IDs.
- `/task/trusted/proof-bootstrap/proof-comparator-protocol.md` and `host-bootstrap-protocol.md`.

## Preconditions

- **P-001:** The protected host validates `TASK-071` receipts and actual integrated source ancestry, the recorded candidate parent/scope base, current canonical taskfmt revision and fingerprint, and the reviewed tui-snap tool pin.
- **P-002:** The driver, fixture application, worker program, isolation observer and source requirement inputs are immutable outside the executor checkout. The submitted executable cannot alter the independent judge or its expected data.
- **P-003:** Candidate workers have the pinned offline toolchain and owned PTYs where required, with no host authority, expected artifacts, network, credentials or host socket access.

## Scope

In scope:

- `tools/refactor-proof/architecture` for the stated operation and narrow directly necessary tests.
- `tools/refactor-proof/bin/tc-proof` for the stated operation and narrow directly necessary tests.
- Independent qualification outputs in host-assigned run directories; no production application changes.

Out of scope:

- Comparator/host-core redesign; application/component repairs; oracle baseline capture/approval for the real application inventory; new scheduler/monitor/services.
- Independent driver/vectors/protocol modifications, catalog changes, shared expected data, real provider side effects, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy all exact R-001 clauses in the protected source obligations. Implement architecture from proof-contract.md using immutable per-task source obligations and qualified test/source-check profiles. Qualify whole-source parsing, missing/empty roots, forbidden dependency/public API paths, receiver-bearing Props helpers, copied/renamed painters, inert/dead component calls and paint-over witnesses against independent fixtures. Preserve legitimate external author components and application-owned art. Distinguish attributed PARTS resolution, documented painted-slot equality and source registry completeness; TASK-073 supplies production observation infrastructure and TASK-031 closes the full real registry.
- **R-002 (MUST):** Satisfy all exact R-002 clauses in the protected source obligations. Separate the independent verifier implementation/fixtures from later production scanner and component repairs. Stage-specific known unfinished architecture obligations retain exact owners and truthful failure records; they are never silently passed. In-scope already completed invariants and the task's own fixtures must pass. A narrow syntax scan cannot claim whole-program reachability or API semantics; pair explicit scan scope with executed production phase/action tests.
- **R-003 (MUST):** Preserve every source-qualified assertion/test mapped to R-003. Pass every group072 independent fixture and recovery positive, including malformed syntax, empty claimed root, omitted registry member, subset-only PARTS acceptance, false component-versus-row attribution, ignored slot, self/borrowed/typed receiver forms, unreferenced Props helper, duplicate same-ID construction and live paint-over. Reject fixture/allowlist/profile substitution and source/task/tree mismatch. Preserve existing runner/accounting/comparator behavior when adding dispatch.
- **R-004 (MUST NOT):** Violate any mapped rejection, deferral or trust rule. Never use broad keyword bans that reject legitimate custom author painting, grant blanket generic drawing waivers, claim dynamic-disabled capture is unimplemented despite its publication tests, invent public Ord/FieldControl/patch_part APIs, or edit application/component production source in this verifier task.
- **R-005 (MUST):** The complete independently controlled gate passes on the exact frozen candidate tree with all required outputs and unchanged trusted inputs.

R-001 also requires the protected `architecture-bootstrap/architecture-bootstrap-protocol.md` corpus through CHK-011. Qualify both the finite Rust contract model and actual pinned terminal-components production probes, including real compiler/runtime evidence, focused positive controls, mutation contrasts, complete registry/state membership and protected external observer transport. Neither an integer-cell model nor an already-failing whole-main subject substitutes for actual positive/mutant discrimination. These fixtures use existing pinned source and private test observations; they do not require the future TASK-073 product.

R-002 additionally requires CHK-006's protected `architecture-bootstrap-source-driver.py` corpus: compile actual public-TC external consumers, reject private/missing/wrong-arity APIs and invalid Rust references under both CHECKS heading forms in sections 18, 19, 20, 29 and beyond the pinned latest section, and traverse real Cargo metadata from every library root to reject direct/transitive application dependencies. Every negative is followed by a fresh positive; compiler and metadata processes run under the external observer, never a submitted verdict model. The source-qualification protocol identifies the bounded covered controls and the remaining mandatory source-review controls explicitly.

R-001 also requires CHK-012's protected `style-timing-bootstrap/style-timing-bootstrap-protocol.md` corpus before accepting the future TASK-073 timing seam. Qualify actual pinned style-resolution entries and an actual whole Showcase frame denominator, disabled/calibration/measured modes, complete invocation census, unchanged cells/state/cache/allocation observations, conservative uncertainty arithmetic and independently planted source/runtime/timing-forgery controls. This proves the measurement instrument, not TASK-067's final five-percent result; it consumes no future TASK-073 receipt.

R-002 additionally requires CHK-013's protected `broker-bootstrap/broker-bootstrap-protocol.md` corpus against the submitted architecture dispatcher. Qualify ADJ-13's sole private Unix/crossterm signal broker, parsed whole-core source inventory, effective cfg/module guards, qualified and aliased type identities, bounded fields and forbidden additional mutable globals. Every rejection requires protected syntax observation and a fresh positive recovery. The existing 193-case source corpus does not cover this new exception. `--self-test` records instrument premises only and cannot satisfy CHK-013. Backend-free metadata/compile and actual singleton initialization/lease/retry/signal behavior remain separate TASK-009 obligations; no future TASK-009 receipt is prerequisite here.

## Acceptance criteria

### AC-001 — Qualified architecture-verifier behavior
```gherkin
Given independently authored group72 positive negative and recovery fixtures
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
When the protected actual public API compiler documentation and Cargo dependency probes execute against the submitted implementation
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

### AC-006 — Preserve accepted operation group71
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-008`

### AC-007 — Preserve accepted operation group70
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-009`

### AC-008 — Preserve exact comparison
```gherkin
Given the protected independently accepted prerequisite corpus
When all original positive negative and recovery cases execute against the extended dispatcher
Then the complete prerequisite behavior remains exact without omitted results
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-010`

### AC-009 — Qualify actual Rust and production architecture probes
```gherkin
Given the protected Rust model and actual pinned terminal components fixture corpus
When the submitted architecture dispatcher executes every positive negative and recovery case through the independent observer
Then exact source compiler runtime and result evidence passes without fixture substitution or omitted production probes
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-011`

### AC-010 — Qualify actual style timing measurement authority
```gherkin
Given frozen actual source timing subjects and an independent host clock and observer
When the submitted architecture dispatcher handles every protected timing positive negative and recovery case
Then raw calibration corrected and complete frame observations bind to actual production paths without forged durations omitted calls or a future receipt cycle
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-012`

### AC-011 — Qualify the sole private backend broker exception
```gherkin
Given the immutable ADJ-13 source corpus and protected real Rust syntax observer
When the submitted architecture dispatcher handles every exact positive negative and recovery case
Then the sole private guarded broker and immutable data are accepted while every forbidden global state cfg type alias path visibility and parse mutation rejects
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-013`

## Fixed decisions

- **D-001:** Both the independently reviewed runner-bootstrap group072 driver and the separate Rust/actual-production architecture corpus judge this product. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No taskfmt dispatcher/monitor/promote call is allowed; no ref update targets main.
- **D-004:** Host accepts `architecture-verifier` only after independent qualification and source rebuild. A later task resolves exactly that accepted producer product; no mutable latest path or candidate-written receipt is valid.
- **D-005:** All commands and result schemas are exactly the frozen proof contract. Unsupported required behavior fails; adding permissive flags or alternative expected data cannot unblock it.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Validate the accepted foundation.
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Implement only the assigned operation group.
    - [ ] **2.1** Implement the complete group72 observable behavior. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Preserve isolation, identity and ownership boundaries. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Retain the accepted comparator and every required regression. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Preserve accepted group71 with its independent corpus. (`R-003`, `AC-006`, `CHK-008`)
    - [ ] **2.5** Preserve accepted group70 with its independent corpus. (`R-003`, `AC-007`, `CHK-009`)
    - [ ] **2.6** Preserve exact comparison with its independent corpus. (`R-003`, `AC-008`, `CHK-010`)
    - [ ] **2.7** Qualify the Rust model and actual production architecture corpus through protected execution. (`R-001`, `AC-009`, `CHK-011`)
    - [ ] **2.8** Qualify real style timing paths, raw measurements and forgery controls without a future producer receipt. (`R-001`, `AC-010`, `CHK-012`)
    - [ ] **2.9** Qualify the exact ADJ-13 broker AST exception through the independent submitted-checker corpus. (`R-002`, `AC-011`, `CHK-013`)
- [ ] **3** Qualify the frozen executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
