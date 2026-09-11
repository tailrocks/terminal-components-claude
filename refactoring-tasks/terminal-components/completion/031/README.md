---
schema: task/v5
id: TASK-031
title: "Prove complete attributed PARTS and painted-slot closure"
kind: test
---

# TASK-031 — Prove complete attributed PARTS and painted-slot closure

## Goal

The complete component registry proves exact owned PARTS and advertised painted slots. Its retained diagnostic baseline helper also provides truthful cross-process transaction safety without granting oracle-blessing authority.

## Context

This package describes future implementation, not work authorized by the current planning goal. UI authority is immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Existing source/tests establish preservation obligations, not oracle parity. Required predecessors: `TASK-009`, `TASK-010`, `TASK-011`, `TASK-012`, `TASK-013`, `TASK-014`, `TASK-015`, `TASK-016`, `TASK-017`, `TASK-018`, `TASK-019`, `TASK-020`, `TASK-021`, `TASK-022`, `TASK-023`, `TASK-024`, `TASK-025`, `TASK-026`, `TASK-027`, `TASK-028`, `TASK-029`, `TASK-030`, `TASK-008`, `TASK-073`. Receipt acceptance and integrated source ancestry are both mandatory.

Read before editing:

- `/task/trusted/source-witnesses.md` in full; its finite source-state, actual-paint and real-mutant cases are normative R-001/R-002/R-003 proof obligations, not optional examples.
- `/task/trusted/obligations.md` in full, including every assigned historical clause.
- `/task/trusted/source-obligations.tsv` in full; its exact source-qualified clauses, dispositions and task/check mappings are normative, not optional background.
- `/work/docs/refactoring-plan/components.md`, `architecture.md`, `architecture-adjudication.md` and `proof-contract.md`.
- Every pinned oracle/main source and existing test named in the trusted family/historical rows.

## Preconditions

- **P-001:** The protected host has accepted TASK-001/070/071/072 products, sealed complete baseline TASK-006 and exact inventory/dispositions TASK-007/008; every listed predecessor is accepted and integrated into the recorded parent.
- **P-002:** Source pins, taskfmt `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, reviewed tui-snap revision, image/toolchain/lock and immutable catalog/context identities resolve from protected receipts.
- **P-003:** The host froze exact owned scenario membership, source-qualified non-applicability, numeric traces and new-test obligations; the executor cannot choose expected output or expand write scope.

## Scope

In scope:

- `crates/tui-testing/src` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/conformance.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/completion_031.rs` solely for the named task-specific regression and mutation witnesses.

Out of scope:

- Application domain repairs, unrelated component families, alternative runtime/editor/collection engines, baseline blessing, provider execution, branch promotion and publication.
- Changing accepted proof tools, protected adapters, historical archives, task packages, expected artifacts, fixture membership, stage dispositions or another task's source scope.

## Requirements

- **R-001 (MUST):** Using TASK-073 attribution, compute each registered component owned resolution union across every applicable real state and require equality with PARTS. Same-ID child composition counts; caller row parts do not. Include Probe/Dialog/Props/PropsList and every source-discovered family; generate degenerate-area and PARTS coverage from one registry. All source-qualified historical clauses assigned to this task and listed in trusted/obligations.md are binding with their recorded accepted/superseded/deferred dispositions. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-002 (MUST):** Parse actual public Overrides named slot promises and require exact equality to slots changing intended painted cells across fixture sweep. Measuring-only style queries do not count as paint. No extra set, manual coverage list, fake owner IDs, new public patch_part hook or declaration-only slot proof. Preserve the architectural and CP-COMMON clauses in trusted/obligations.md, using the same production owner for pixels and interaction. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-003 (MUST):** Mutation controls introduce undeclared owned part, declared unreachable part, removed registry invocation, ignored slot, mistaken custom row ownership and empty readiness coverage; all fail. Legitimate same-ID composition and custom row part pass with truthful attribution. Geometry/hits/focus/unrelated neighbors remain unchanged under slots; draw invariance and reference suppression hold. Execute every required historical/oracle test identity under the immutable stage map; compatible and previously closed cases stay green, all future-owned failures remain recorded and no case is silently omitted.
- **R-004 (MUST NOT):** Do not edit expected frames, oracle or comparison rules, accepted harness/adapter/test-disposition inputs, task protocols, required-set membership, capability/slot allowances or receipts. Do not duplicate generic widget engines, substitute test-only/inert painters, fabricate semantics, weaken checks, bless candidate output, or perform domain/provider operations.
- **R-005 (MUST):** The full host-controlled canonical gate passes on the frozen exact tree, with protected-input integrity, actual prerequisite ancestry and complete check evidence.
- **R-006 (MUST):** Repair the retained `junie_tui_testing::digest::Baseline` implementation in `crates/tui-testing/src/digest.rs` without changing its public API: a genuine shared interprocess lock covers the entire fresh-disk read, merge, update and atomic publication transaction. Preserve every committed unrelated entry, including another process's newer same-key value when this process writes a different key; cached entries cannot override fresh committed disk state. Execute W-031-10/11 through CHK-005 against the actual submitted helper. A process-local mutex, rename alone, sequential subprocesses or a rewritten concurrency claim is insufficient. This diagnostic-helper repair never permits tracked baseline or immutable-oracle writes during acceptance.

## Acceptance criteria

### AC-001 — Exact registry PARTS and painted-slot contracts hold
```gherkin
Given the accepted real Rust registry attribution and painted-slot fixtures
When the independent architecture verifier executes every R-001 positive and violating witness
Then the exact owned PARTS and documented painted-slot sets match with truthful provenance and complete registry coverage
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-006`

### AC-002 — Architectural ownership holds
```gherkin
Given the live testing-registry implementation and independent ownership witnesses
When the R-002 borrowed state routing styling and production-path boundaries are exercised
Then each boundary holds and every specified violating witness is rejected
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — Complete regressions and negative cases hold
```gherkin
Given the frozen required test identities and the task-specific negative cases in R-003
When every required target and profile executes with complete outcome accounting
Then each positive and negative case has its required outcome and no closed case regresses
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Verification authority stays immutable
```gherkin
Given the host-owned catalog source pins expected artifacts and accepted prerequisite receipts
When exact identity ancestry scope and trust boundaries are checked
Then no forbidden write or candidate-controlled acceptance input is admitted
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

### AC-006 — Retained diagnostic publication is cross-process safe
```gherkin
Given two actual helper processes released from a common readiness barrier and an isolated temporary diagnostic baseline
When protected transaction-phase evidence proves that the second writer waits outside the first writer's held critical section and both then publish disjoint entries and ordered same-key updates
Then all 48 entries survive, fresh committed values are preserved, and lock or publication failures fail closed without unsafe recovery
```

**Verification**

- **Type:** scenario
- **Covers:** `R-006`
- **Check:** `CHK-005`

## Fixed decisions

- **D-001:** Parse actual public Overrides named slot promises and require exact equality to slots changing intended painted cells across fixture sweep. Measuring-only style queries do not count as paint. No extra set, manual coverage list, fake owner IDs, new public patch_part hook or declaration-only slot proof.
- **D-002:** Apply ADJ-04 exactly as recorded in architecture-adjudication.md.
- **D-003:** TASK-073 establishes observation/registry infrastructure; TASK-031 proves complete repaired-family equality. Do not weaken inherited attribution or registry checks to accommodate an incomplete family.
- **D-004:** Missing or impossible required behavior is a failed obligation, not a skip. Any necessary change outside this narrow scope returns to its owning prerequisite and requires renewed acceptance.
- **D-005:** Future `/proof/bin/tc-proof` operations are installed only from accepted producer receipts: TASK-001 host/comparator core, TASK-070 preflight/required/oracle/capture/close, TASK-071 account-tests and TASK-072 architecture. No candidate-local substitute or recursive taskfmt gate is permitted.
- **D-006:** A testing registry is not an oracle widget. This task has no invented historical registry frame or PTY parity case. CHK-006 proves exact production resolution, registration, slot paint, degenerate geometry and public conformance through independently qualified Rust witnesses. CHK-005 executes the complete required inventory, preserving every already closed component case while reporting still-future-owned app failures without masks or omissions.
- **D-007:** BT03 fixes transaction ownership at the existing helper, not every caller. Retain the public API and use an MSRV-1.88-compatible, std-only exclusive sibling-directory lock acquired by atomic `create_dir` on the supported local filesystem, keyed by the canonical existing parent plus destination filename. Reject symlink destinations/lock paths and ambiguous aliases rather than allowing different locks for one file. Keep the existing in-process mutex for cache access, acquire locks in one fixed order, and replace cached state with the freshly read committed file inside the interprocess critical section before applying the requested update. Use bounded contention timeout and a uniquely owned temporary publication file; release only the acquired lock via ownership-scoped cleanup. Crash-left/stale/unreadable locks fail closed with an actionable error: no age-based, PID-based or automatic lock stealing. Recovery is a separate explicit operator action after all writers are stopped, never an acceptance side effect. Failure before successful rename preserves the previously committed file and cleans up only this transaction's owned temporary resources. Existing callers need no API migration; update the helper's documentation and actual tests to this precise supported-filesystem/failure contract. No extra dependency or MSRV increase is authorized.

BT03 supported-path and scheduling boundary: before authorizing diagnostic publication, the protected host establishes a local filesystem supporting atomic exclusive directory creation and same-directory rename, canonical existing parent identity, a regular nonsymlink destination and a unique link identity. Reject existing hardlinked destinations (`nlink > 1` on supported Unix metadata, or the equivalent independently established host filesystem identity); reject ambiguous parent aliases and unsupported/unprovable identity/atomicity rather than promising portable cross-process safety by assumption. This adds no global filesystem or elevated-privilege authority. The actual transaction phase instrumentation is test-only and independently frozen: hold writer A after acquiring the interprocess lock; observe writer B's attempted acquisition and demonstrated waiting without admission to fresh-read/mutate/commit; release A, observe its commit/unlock, then B's acquisition/read/commit. No barrier waits for both writers to enter a correctly exclusive critical section. The unlocked mutant fails the observed admission/order invariant even if the final 48-entry union survives by chance; scheduler luck is not negative-control proof.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Bind exact authority and scope.
    - [ ] **1.1** Validate protected receipts, parent ancestry and task inputs. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Deliver the owned component outcome.
    - [ ] **2.1** Prove the exact attributed registry and painted-slot contracts with completion_031 witnesses. (`R-001`, `AC-001`, `CHK-006`)
    - [ ] **2.2** Preserve borrowed ownership, attributed paint and the task-specific architecture contract. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Execute exact regression identities and every stated negative witness. (`R-003`, `AC-003`, `CHK-005`)
    - [ ] **2.4** Prove retained diagnostic baseline transaction safety with barrier-overlapping processes, fresh-state and fail-closed controls. (`R-006`, `AC-006`, `CHK-005`)
- [ ] **3** Verify the frozen candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
