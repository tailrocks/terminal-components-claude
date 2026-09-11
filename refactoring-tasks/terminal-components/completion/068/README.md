---
schema: task/v5
id: TASK-068
title: "Close public API documentation and obsolete deferrals"
kind: refactor
---

# TASK-068 — Close public API documentation and obsolete deferrals

## Goal

Close the public application/author facade, executable documentation and obsolete deferrals against the completed implementation; every accepted API requirement has a current compiling owner or explicit superseded disposition.

## Context

Public facade/author keyed example/docs/registry/examples/backend-free boundaries complete; doc-check covers §§18–20 and real signatures; obsolete deferrals and clone name reconciled; future semver baseline prepared. This is a bounded closure of already restored production work, not permission to reopen unrelated design choices. ARCH:A01; ARCH:A02; ARCH:A25; ARCH:A29; ARCH:A30; ARCH:A32; HIST:A108; HIST:A109; accepted architecture §§16,18–20,37,42,48,73,74; architecture.md and architecture-adjudication.md.

Read before editing:

- `/task/trusted/obligations.md` and the exact historical clauses in `/task/trusted/source-obligations.tsv`.
- The immutable host-mounted proof contract, architecture assessment/adjudication, test dispositions, scenario expansions and task-stage command inventory.
- The accepted parent versions of every writable file and the commit-qualified source references above.

## Preconditions

- **P-001:** The host has accepted and integrated every hard dependency: `terminal-components/completion/065`, `terminal-components/completion/066`, `terminal-components/completion/067`. Their tested source identities must be ancestors of the actual integration parent.
- **P-002:** The accepted TASK-001/TASK-070/TASK-071/TASK-072 proof products, sealed complete oracle bundle, required-test inventory and dispositions are installed read-only. The host has frozen this package and generated `/run/tc-proof/context-index.json` and its immutable per-check contexts from those receipts.
- **P-003:** Start from the host's isolated integration worktree, never main or the oracle. All four application closures pass and the closure-stage unresolved migration set is empty. Missing or changed trust products stop execution.

## Scope

In scope:

- Close the public application/author facade, executable documentation and obsolete deferrals against the completed implementation; every accepted API requirement has a current compiling owner or explicit superseded disposition.
- Only the explicit files in verify.toml; the detailed clauses and independent checks in trusted/obligations.md are binding.

Out of scope:

- New product features, redesign, baseline changes, oracle adaptation, provider integration, library/application repairs outside the declared closure scope, dependency upgrades and release publication.
- Editing the immutable catalog, trusted verifier, oracle source, expected evidence, scenario membership or accepted test dispositions; pushing or merging into main.

## Requirements

- **R-001 (MUST):** Close the public application/author facade, executable documentation and obsolete deferrals against the completed implementation; every accepted API requirement has a current compiling owner or explicit superseded disposition. Every numbered closure clause in `trusted/obligations.md` and every mapped exact historical clause in `trusted/source-obligations.tsv` must have its designated passing independent proof. Preserve complete direct and PTY oracle equality for all completed product scenarios.
- **R-002 (MUST):** Keep the virtual workspace, one junie_tui facade, unpublished application libraries, backend-free consumers and no library-to-app dependency. Preserve receiver-free borrowing, typed identity/actions and all accepted API boundaries; no legacy facade or compatibility module returns.
- **R-003 (MUST):** Execute the full required source-qualified test/profile/scenario inventory without fail-fast suppression. At this closure stage the unresolved set is empty: all owned, prerequisite and previously closed cases pass. Whole-workspace compilation, MSRV, formatting, lint, documentation and compatible architecture gates remain green. Missing execution and renamed or removed assertions without approved relocation fail.
- **R-004 (MUST NOT):** Alter the oracle, expected artifacts, required membership, action coordinates, normalization, trust roots, test-disposition authority, tool pins or host context; bless candidate output; suppress failure; fabricate review/worker evidence; reintroduce rejected architecture; or push or merge main.
- **R-005 (MUST):** The completion gate succeeds on the exact frozen tree with genuine accepted dependency ancestry and unchanged protected evidence.

## Acceptance criteria

### AC-001 — The bounded closure outcome holds

```gherkin
Given the accepted predecessor tree and the exact closure clauses in trusted/obligations.md
When the independently qualified check executes every named closure proof
Then each closure clause has passing source-bound evidence and no unresolved owned obligation
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-006`

### AC-002 — Accepted architecture and proof mechanisms remain intact

```gherkin
Given the frozen candidate and protected architecture mutation witnesses
When the trusted architecture verifier executes the applicable production and negative fixtures
Then every named ownership API and measurement invariant holds without weakening proof
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-006`

### AC-003 — All required executions and closed scenarios remain green

```gherkin
Given the complete required test inventory and empty closure-stage unresolved set
When the trusted accounting verifier executes all profiles without fail-fast suppression
Then all required results are present and pass with no skipped or silently relocated assertion
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-005`

### AC-004 — Trust and integration authority remain unchanged

```gherkin
Given the host-owned catalog oracle tool pins and accepted dependency receipts
When preflight verifies provenance scope isolation and actual ancestor identities
Then the candidate cannot redefine proof authority and every prerequisite is integrated
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-006 — Direct production captures are complete

```gherkin
Given all completed application and component direct scenarios in the sealed required set
When the independent runner builds the frozen production tree and replays the direct actions
Then all required frame and semantic checkpoints are captured with authentic source bindings
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — Executable PTY captures are complete

```gherkin
Given all applicable sealed executable scenarios and literal input transcripts
When the independent runner executes the frozen binaries in owned PTYs
Then all terminal checkpoints exits and lifecycle observations are captured without retargeting
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

### AC-008 — Exact completed product parity is preserved

```gherkin
Given the complete sealed oracle and independently captured candidate outputs
When the comparator validates schemas artifacts provenance cells cursor and semantic state
Then every required checkpoint is exactly equal and no expectation or membership changed
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-004`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

## Fixed decisions

- **D-001:** Documentation describes implemented accepted contracts; it cannot amend the product oracle or invent new public APIs to satisfy obsolete prose. A valid cross-crate API must be resolved by the gate, not removed merely because the old scanner cannot see it.
- **D-002:** The immutable oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. Neither old main snapshots nor current moving branches can redefine UX.
- **D-003:** All 361 source scenario rows and their accepted finite expansions remain mandatory at closure. Direct-only states retain their independently approved lane applicability; executors cannot waive an unavailable PTY scenario.
- **D-004:** The host owns verdicts, review attestations, immutable context and integration refs. Candidate-written reports are untrusted inputs. Exact comparison plus source/semantic proof is required; no masks, tolerances or screenshots alone.
- **D-005:** Taskfmt's run/promote lifecycle hardcodes main and is prohibited. Use supported standalone verification and the qualified host's exact-tree expected-parent integration protocol.
- **D-006:** Shared xtask, CI, facade and test-inventory files have one integration writer. Parallel individually green siblings require a fresh combined-tree verification before downstream use.

## Campaign execution binding

The explicit [campaign executor adaptation](/work/docs/refactoring-plan/campaign-executor-protocol.md) is mandatory. It preserves canonical AGENTS template provenance but supersedes its executor-authoritative checks and empty-progress invocation. Local checks are advisory; the executor submits candidate/progress to the operator, who invokes the existing host freeze and verify commands with immutable per-check contexts, pinned configuration, explicit base and complete progress. Only the host-authentic exact-tree verdict authorizes completion/integration. No new scheduler or taskfmt lifecycle API is implied.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Validate source, trust, scope and integrated prerequisites. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Complete the bounded closure.
    - [ ] **2.1** Deliver every exact closure clause and its independent proof. (`R-001`, `AC-001`, `CHK-006`)
    - [ ] **2.2** Capture all required direct production checkpoints. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.3** Capture all required executable PTY checkpoints. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.4** Compare complete cells cursor semantics and provenance. (`R-001`, `AC-008`, `CHK-004`)
    - [ ] **2.5** Retain accepted architecture and nonvacuous proof. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.6** Execute the complete inventory with no unresolved result. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify.
    - [ ] **3.1** Pass the independent exact-tree completion gate. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
