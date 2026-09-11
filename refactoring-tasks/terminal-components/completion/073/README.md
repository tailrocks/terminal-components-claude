---
schema: task/v5
id: TASK-073
title: "Establish truthful resolution attribution and generated registry"
kind: test
---

# TASK-073 — Establish truthful resolution attribution and generated registry

## Goal

Truthful resolution provenance and one generated registry support later conformance repair.

## Context

This package describes future implementation, not work authorized by the current planning goal. UI authority is immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Existing source/tests establish preservation obligations, not oracle parity. Required predecessors: `TASK-008`, `TASK-072`. Receipt acceptance and integrated source ancestry are both mandatory.

Read before editing:

- `/task/trusted/source-witnesses.md` in full; its finite source-state, actual-paint and real-mutant cases are normative R-001/R-002/R-003 proof obligations, not optional examples.
- `/task/trusted/style-timing-contract.md` in full; its single production timing seam and independent qualification are normative R-001/R-002/R-003 obligations, consumed by TASK-067.
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

- `crates/tui/src/ui/mod.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/src/ui/paint.rs` solely for complete source-bound style-call coverage and fail-closed observation of color-binding paint branches; never charge whole painting operations as style resolution or alter paint behavior.
- `crates/tui/src/lib.rs` solely for testing-feature export of the caller-owned style timing probe if the public testing consumer requires it; no ordinary application API changes.
- `crates/tui/src/collection/rowui.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/src/components/status.rs` and `crates/tui/src/components/grid.rs` solely to attach the same testing-only caller-owned probe to their existing direct binding leaves and qualify empty/nonempty branches; no component behavior or public API repair.
- `crates/tui-testing/src` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/conformance.rs` solely for the behavior and direct proof stated in this package.
- `crates/tui/tests/completion_073.rs` solely for the named task-specific regression and mutation witnesses.

Out of scope:

- Application domain repairs, unrelated component families, alternative runtime/editor/collection engines, baseline blessing, provider execution, branch promotion and publication.
- Changing accepted proof tools, protected adapters, historical archives, task packages, expected artifacts, fixture membership, stage dispositions or another task's source scope.

## Requirements

- **R-001 (MUST):** Extend existing testing observation with component-owned versus caller-row-owned provenance retaining real Id/Family/Variant/Part/Resolved. Propagate through nested rows/columns and restore on callback exit; measurement-only resolution remains distinct. Generate conformance subject enumeration once, preserving all current invocations and detecting absent registry subjects. All source-qualified historical clauses assigned to this task and listed in trusted/obligations.md are binding with their recorded accepted/superseded/deferred dispositions. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-002 (MUST):** This is foundation, not all-component closure. Qualify observation and registry with independently controlled small fixtures and mutated production paths accepted by TASK-072. No fabricated IDs, second unchecked part list, public patch_part hook or weakening promised customization. TASK-031 alone later requires complete repaired-family equality. Preserve the architectural and CP-COMMON clauses in trusted/obligations.md, using the same production owner for pixels and interaction. Apply every corresponding row in `trusted/source-obligations.tsv` according to its exact source-qualified disposition and assigned acceptance/check mapping; accepted clauses require implementation or preservation proof, and rejected/superseded/deferred alternatives require their stated non-goal or replacement proof.
- **R-003 (MUST):** Positive fixture uses same-ID child component plus legitimate custom row part and nested callback restoration. Negatives omit provenance, misattribute row, suppress an observed paint, count measurement as paint, drop a registry invocation or install ignored slot; exact reason must fail. Existing public StyledQuery consumers retain identity/fields semantics, and same production frame before/after instrumentation is byte equal. Execute every required historical/oracle test identity under the immutable stage map; compatible and previously closed cases stay green, all future-owned failures remain recorded and no case is silently omitted.
- **R-004 (MUST NOT):** Do not edit expected frames, oracle or comparison rules, accepted harness/adapter/test-disposition inputs, task protocols, required-set membership, capability/slot allowances or receipts. Do not duplicate generic widget engines, substitute test-only/inert painters, fabricate semantics, weaken checks, bless candidate output, or perform domain/provider operations.
- **R-005 (MUST):** The full host-controlled canonical gate passes on the frozen exact tree, with protected-input integrity, actual prerequisite ancestry and complete check evidence.

## Acceptance criteria

### AC-001 — Real resolution provenance and registry are qualified
```gherkin
Given independently accepted TASK-072 Rust probes for real same-ID composition caller rows and generated registry subjects
When actual production observation paths and their provenance and registry mutants execute
Then truthful owner fields callback restoration and subject coverage pass while every specified violating mutant fails
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

## Fixed decisions

- **D-TIMING-OWNER:** This task establishes the single testing-only caller-owned style timing seam consumed by TASK-067 and preserved by TASK-011/028. The exact production-boundary census, three-mode calibration, uncertainty rule and mutation controls in `trusted/style-timing-contract.md` are part of R-001/R-002/R-003 and CHK-006/CHK-005. Foundation qualification uses real small production fixtures; it does not claim the final Showcase Lists frame meets the five-percent threshold. TASK-067 alone measures that restored production workload. No second timer, global probe or detached query-cost extrapolation is allowed.

- **D-TIMING-LEAVES:** The census includes actual binding outside Ui entry methods: `CellUi::drop` final-cell binding, Status item deltas and Grid style deltas. Pin the exact bind expressions and their source branches; do not time RowUi alignment/painting or double-count a caller and its leaf. At main's Lists120×40 the94-column page reaches12 RowUi final binds; closed-shell Status deltas and Grid deltas are zero-reached but exercised by separate actual branch witnesses. Color-binding `dim_layer`/private paint helpers must be censused if reached, otherwise protected qualification fails uncovered-path rather than assigning implicit zero. These paths are assigned here before dispatch; later component owners preserve the accepted seam.

- **D-001:** This is foundation, not all-component closure. Qualify observation and registry with independently controlled small fixtures and mutated production paths accepted by TASK-072. No fabricated IDs, second unchecked part list, public patch_part hook or weakening promised customization. TASK-031 alone later requires complete repaired-family equality.
- **D-002:** Apply ADJ-04 exactly as recorded in architecture-adjudication.md.
- **D-003:** TASK-073 establishes observation/registry infrastructure; TASK-031 proves complete repaired-family equality. This foundation must pass its independent small positive/negative fixtures without requiring unrepaired families to satisfy final equality.
- **D-004:** Missing or impossible required behavior is a failed obligation, not a skip. Any necessary change outside this narrow scope returns to its owning prerequisite and requires renewed acceptance.
- **D-005:** Future `/proof/bin/tc-proof` operations are installed only from accepted producer receipts: TASK-001 host/comparator core, TASK-070 preflight/required/oracle/capture/close, TASK-071 account-tests and TASK-072 architecture. No candidate-local substitute or recursive taskfmt gate is permitted.
- **D-006:** Registry attribution is an architectural observation contract, not an oracle widget or frame namespace. TASK-072 independently accepted Rust fixtures execute the actual production observation path and compare its before/after pixels while separately checking provenance and registry behavior. Those fixture expectations prove instrumentation preservation, never historical application parity. No direct or PTY oracle capture/compare check is required by this foundation; full application execution and honest unresolved outcome accounting remain mandatory through CHK-005.

## Candidate observation ownership

R-001 and R-002 explicitly permit extraction-only observation seams in this task's already writable production source or completion-test files. They are untrusted candidate code, built with the frozen candidate and real production handlers/renderers; they are not accepted oracle adapters, judges, schema authors or receipt producers. TASK-002–006 own the protected observation schema and logical identity mapping; TASK-070 owns independently qualified source/binary/action binding and wrong-state, constant-state, omitted-field, wrong-source and test-only-substitution rejection. Candidate seams may serialize actual focus/edit/selection/target/overlay/domain state, but may not replace input dispatch/rendering, synthesize expected state, branch on a test-only product path or alter protected mappings. The existing prohibition on changing observation adapters means protected reference adapters and runner policy; it does not forbid these explicitly scoped untrusted extraction seams. A seam needing another task's source path remains outside scope and must be assigned before dispatch.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Bind exact authority and scope.
    - [ ] **1.1** Validate protected receipts, parent ancestry and task inputs. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Deliver the owned component outcome.
    - [ ] **2.1** Qualify actual resolution provenance and generated registry with independent Rust probes. (`R-001`, `AC-001`, `CHK-006`)
    - [ ] **2.2** Preserve borrowed ownership, attributed paint and the task-specific architecture contract. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.3** Execute exact regression identities and every stated negative witness. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify the frozen candidate.
    - [ ] **3.1** Pass the complete host gate and preserve exact-tree evidence. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
