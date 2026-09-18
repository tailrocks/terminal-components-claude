---
schema: task/v5
id: TASK-069
title: "Prove complete integration tree and merge readiness"
kind: test
---

# TASK-069 — Prove complete integration tree and merge readiness

## Goal

Produce an independently verified merge-readiness receipt for one exact integration tree: complete architecture/API, every required oracle state and interaction, every authoritative build/test/lint/doc/performance gate, immutable evidence and safe ancestry.

## Context

Visual acceptance is currently blocked: this branch lacks the grouped store and PTY suite. A verifier subagent must import them read-only from the immutable `visual-baseline` tag into the external run before using [`visual-validation.md`](../../../visual-validation.md). Never write `snapshots/`, bless output, or mutate the tag.

Acceptance requires a tested integration head to pass the whole required oracle direct/PTY set, authoritative stable/MSRV/build/lint/docs/API/performance gates, trust immutability and ancestry/merge-readiness checks. This task is a bounded closure of already restored production work, not permission to reopen unrelated design choices. HIST:A130; HIST:RG49; HIST:RG51; HIST:RG52; HIST:RG56; ARCH:A28–A32; every HIST/ARCH/COMP/APP/DEC source in traceability.tsv; [historical PLANNING_GOAL.md at the immutable visual-baseline commit](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2/docs/sources/PLANNING_GOAL.md) §§16–18; proof-contract.md.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- `trusted/coordinator-review-contract.md`: coordinator adversarial review (not satisfied by CHK-007 alone).
- `trusted/host-context/`: fixed preflight, fidelity visual, and machine-close templates.
- `trusted/obligations.md` and the exact historical clauses in `trusted/source-obligations.tsv`.
- The immutable host-mounted proof contract, architecture assessment/adjudication, test dispositions, scenario expansions and task-stage command inventory.
- The accepted parent versions of every writable file and the commit-qualified source references above.

## Preconditions

- **P-001:** The host has accepted and integrated every hard dependency: `terminal-components/completion/068`, `terminal-components/completion/031`. Their tested source identities must be ancestors of the actual integration parent.
- **P-002:** The accepted TASK-001/TASK-070/TASK-071/TASK-072 proof products, sealed complete oracle bundle, required-test inventory and dispositions are installed read-only. The verifier has recorded this package and generated `$RUN_DIR/context-index.json` and its `$RUN_DIR/context-index.json` and `$RUN_DIR/contexts/CHK-NNN.json` (immutable per-check contexts) from those receipts.
- **P-003:** Start from the host's isolated integration worktree, never main or the oracle. All four application closures pass and the closure-stage unresolved migration set is empty. Missing or changed trust products stop execution.
- **Host-local verifier inputs:** The verifier subagent resolves the comparator at `$WORKTREE/tools/refactor-proof/bin/tc-proof` and per-check contexts under `$RUN_DIR/contexts/`; taskfmt receives explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths. No mounts are involved.

## Scope

In scope:

- Produce an independently verified merge-readiness receipt for one exact integration tree: complete architecture/API, every required oracle state and interaction, every authoritative build/test/lint/doc/performance gate, immutable evidence and safe ancestry.
- Only the explicit files in verify.toml; the detailed clauses and independent checks in trusted/obligations.md are binding.

Out of scope:

- New product features, redesign, baseline changes, oracle adaptation, provider integration, library/application repairs outside the declared closure scope, dependency upgrades and release publication.
- Editing the immutable catalog, trusted verifier, oracle source, expected evidence, scenario membership or accepted test dispositions; pushing or merging into main.

## Requirements

- **R-001 (MUST):** Produce an independently verified merge-readiness receipt for one exact integration tree: complete architecture/API, every required oracle state and interaction, every authoritative build/test/lint/doc/performance gate, immutable evidence and safe ancestry. Every numbered closure clause in `trusted/obligations.md` and every mapped exact historical clause in `trusted/source-obligations.tsv` must have its designated passing independent proof. Preserve complete direct and PTY oracle equality for all completed product scenarios.
- **R-002 (MUST):** Verification is read-only with respect to production source and accepted proof authority. The verification records source, independently checks every artifact, joins all prerequisite receipts and binds final evidence to the exact tested tree; no self-authored completion marker establishes success.
- **R-003 (MUST):** Execute the full required source-qualified test/profile/scenario inventory without fail-fast suppression. At this closure stage the unresolved set is empty: all owned, prerequisite and previously closed cases pass. Whole-workspace compilation, MSRV, formatting, lint, documentation and compatible architecture gates remain green. Missing execution and renamed or removed assertions without approved relocation fail.
- **R-004 (MUST NOT):** Alter the oracle, expected artifacts, required membership, action coordinates, normalization, trust roots, test-disposition authority, tool pins or host context; bless candidate output; suppress failure; fabricate review/worker evidence; reintroduce rejected architecture; or push or merge main.
- **R-005 (MUST):** The completion gate succeeds on the exact fixed tree with genuine accepted dependency ancestry and unchanged protected evidence.

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
- **Check:** `CHK-004`

### AC-002 — Accepted architecture and proof mechanisms remain intact

```gherkin
Given the fixed candidate and protected architecture mutation witnesses
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
Given the campaign-pinned catalog oracle tool pins and accepted dependency receipts
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
When the independent runner builds the fixed production tree and replays the direct actions
Then all required frame and semantic checkpoints are captured with authentic source bindings
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-007 — Executable PTY captures are complete

```gherkin
Given all applicable sealed executable scenarios and literal input transcripts
When the independent runner executes the fixed binaries in owned PTYs
Then all terminal checkpoints exits and lifecycle observations are captured without retargeting
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-007`

## Fixed decisions

- **D-001:** This task is validation-only. Failure cannot be repaired inside its report directory or waived in prose; reopen the responsible task, re-integrate and rerun affected closure including this task. No write to main or automatic tool-PR merge is authorized.
- **D-002:** The immutable oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. Neither old main snapshots nor current moving branches can redefine UX.
- **D-003:** All 361 source scenario rows and their accepted finite expansions remain mandatory at closure. Direct-only states retain their independently approved lane applicability; coordinators cannot waive an unavailable PTY scenario.
- **D-004:** The host owns machine verdicts, immutable context and integration refs. Human adversarial review is coordinator-owned evidence per `coordinator-review-contract.md`; TASK-069 CHK-007 establishes machine merge-readiness closure only. Candidate-written reports are untrusted inputs.
- **D-005:** Taskfmt's run/promote lifecycle hardcodes main and is prohibited. Use supported standalone verification and the qualified host's exact-tree expected-parent integration protocol.
- **D-006:** Shared xtask, CI, facade and test-inventory files have one integration writer. Parallel individually green siblings require a fresh combined-tree verification before downstream use.

## Subagent execution

The implementer, verifier, and reviewer subagents own this task. The coordinator assigns isolated worktrees, reviews evidence, and integrates only reviewed commits; it does not edit task-owned files.

All execution is host-local. Use `$TASK_DIR` for this package, `$WORKTREE` for the isolated repository, `$RUN_DIR` for evidence and logs, and `$SCOPE_BASE` for the recorded parent. Run the latest standalone taskfmt only for this package:

```text
taskfmt lint "$TASK_DIR"
taskfmt verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

Taskfmt is validation only. No containers, images, mounts, or task orchestration commands are used. The verifier owns the final taskfmt evidence; the reviewer checks it against every `R-*`, `AC-*`, and `CHK-*` obligation before the coordinator integrates. Keep generated evidence under `$RUN_DIR` and do not modify task metadata or protected oracle inputs.


## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Validate source, trust, scope and integrated prerequisites. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Complete the bounded closure.
    - [ ] **2.1** Deliver every exact closure clause and its independent proof. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.2** Capture all required direct production checkpoints. (`R-001`, `AC-006`, `CHK-002`)
    - [ ] **2.3** Capture all required executable PTY checkpoints. (`R-001`, `AC-007`, `CHK-003`)
    - [ ] **2.4** Compare complete cells cursor semantics and provenance. (`R-001`, `AC-001`, `CHK-004`)
    - [ ] **2.5** Retain accepted architecture and nonvacuous proof. (`R-002`, `AC-002`, `CHK-006`)
    - [ ] **2.6** Execute the complete inventory with no unresolved result. (`R-003`, `AC-003`, `CHK-005`)
- [ ] **3** Verify.
    - [ ] **3.1** Pass the independent exact-tree completion gate. (`R-005`, `AC-005`, `CHK-007`)
<!-- checklist:end -->
