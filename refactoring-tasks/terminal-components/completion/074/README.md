---
schema: task/v5
id: TASK-074
title: "Bind complete per-check contexts in tc-proof prepare"
kind: bugfix
---

# TASK-074 — Bind complete per-check contexts in tc-proof prepare

## Goal

`tc-proof prepare` emits complete per-check contexts for the adapter-task
worker paths, so TASK-002..007 checks can run against the drivers they
already declare.

## Context

TASK-002..007 adapter-task checks cannot run: `prepare` binds incomplete
per-check contexts, missing the native oracle contract
(`original{source_sha256}`-class fields), the nested comparator roots
(`oracle_root`/`candidate_root`), and the required/test-register bindings
(`required_*`, preparation register) that the oracle/compare/account-tests
workers require. The bind code —
`tools/refactor-proof/src/verifier.rs::build_context`/`bind_*` and the
`tools/refactor-proof/runner` glue — was owned by retired TASK-001/TASK-070
and no live task owns it. TASK-071 owns `accounting/` and TASK-072 owns
`architecture/` plus the shared `bin/tc-proof` dispatcher; this task
touches none of those.

Visual acceptance stays governed by the frozen oracle: a verifier subagent
must import the grouped store and PTY suite read-only from the immutable
`visual-baseline` tag into the external run before using
[`visual-validation.md`](../../../visual-validation.md). Never write
`snapshots/`, bless output, or mutate the tag.

This is a repair-task package. Task dependencies are `TASK-071` and
`TASK-072`; their accepted worker behavior is prerequisite evidence this
task consumes but never re-implements. TASK-001 and TASK-070 stay retired
fail-closed. UI authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`;
architecture starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. Follow
the subagent-only host-local workflow in proof-contract.md; use standalone
taskfmt only for this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md): prepare's binding obligations and the native run layout.
- `trusted/obligations.md`: every demand site and its required binding under its requirement/acceptance/check IDs.
- `trusted/context-bind/context-bind-protocol.md`: the exact independent judge, profiles, and verdicts.
- The failing demand sites: `tools/refactor-proof/src/verifier.rs` `build_context`/`bind_compare_qualification`, `tools/refactor-proof/src/compare.rs`, `tools/refactor-proof/runner/operations.py` `run_oracle`, `tools/refactor-proof/runner/validate.py`, `tools/refactor-proof/accounting/extension.py`, and `tools/refactor-proof/accounting/dispatch.py`.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Accepted TASK-071 and TASK-072 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The oracle/commit pins, the task contract, and the trusted driver/templates are immutable. Derived bindings come from the task manifest, the oracle, and the candidate worktree — never from candidate-written expectations.
- **Host-local verifier inputs:** The verifier subagent builds the native proof tool with `scripts/campaign-build-proof.sh`, runs `tc-proof prepare` for this package, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths and `RUN_DIR` exported. Per-check contexts resolve under `$RUN_DIR/contexts/`. No mounts are involved.

## Scope

In scope:

- `tools/refactor-proof/src/verifier.rs` for the stated binding repair and narrow directly necessary inline tests.
- `tools/refactor-proof/runner/__main__.py`, `tools/refactor-proof/runner/context.py`, `tools/refactor-proof/runner/operations.py`, and `tools/refactor-proof/runner/validate.py` for directly necessary binding-glue repair only.
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Any `refactoring-tasks/**` change — in particular no template, driver, or manifest edits inside TASK-002..007 or any other task package. The fix lives in prepare, not in other tasks' trusted dirs.
- `tools/refactor-proof/accounting/**` (TASK-071), `tools/refactor-proof/architecture/**` (TASK-072), and `tools/refactor-proof/bin/tc-proof` (shared 071/072 bundle): consume their accepted behavior, never edit them.
- Demand-side weakening: `tools/refactor-proof/src/compare.rs`, `tools/refactor-proof/src/bin/`, and runner/accounting/architecture validators keep their exact requirements.
- Comparator/host-core redesign, application/component repairs, oracle baseline capture/approval, new scheduler/monitor/services, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001, O-002). Bind the complete native oracle contract for native-namespace oracle contexts and the complete nested comparator context for compare contexts, derived from the task manifest, the oracle, and the candidate worktree.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-003, O-004). Lift the declared account-tests template values into the qualification and derive the complete preparation register; bind well-formed architecture configuration for architecture contexts.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-005, O-006). Bind task-derived axes/members for every worker-path context instead of the hardcoded tiny fixture default, and preserve every binding prepare already emits: identity, trust, tool, dependency, index, result, template-flow, and manifest-agreement bindings.
- **R-004 (MUST NOT):** Violate any prohibition mapped to R-004 in `trusted/obligations.md` (O-007). Never edit task packages, 071/072-owned files, or demand-side validators; never hardcode oracle frames, digests, or membership to satisfy the judge; never substitute candidate-written expectations for manifest/oracle/worktree derivation.
- **R-005 (MUST):** The complete independently judged gate passes on the exact fixed candidate tree with all required outputs and unchanged trusted inputs.

## Acceptance criteria

### AC-001 — Complete native oracle context
```gherkin
Given the pinned oracle and the native showcase namespace declaration
When prepare binds the oracle worker-path context
Then every native contract field the oracle worker requires is present well-formed and cross-consistent
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-002 — Complete architecture context
```gherkin
Given the pinned source and the architecture worker requirements
When prepare binds the architecture worker-path context
Then configuration membership and preserved bindings satisfy the independent judge
```

**Verification**

- **Type:** invariant
- **Covers:** `R-002`
- **Check:** `CHK-005`

### AC-003 — Complete accounting preparation context
```gherkin
Given the declared preparation template and the pinned source registers
When prepare binds the account-tests worker-path context
Then lifted declarations and the derived preparation register satisfy the independent judge
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-004`

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
- **Check:** `CHK-006`

### AC-006 — Complete nested comparator context
```gherkin
Given the pinned oracle and candidate roots and the required comparison set
When prepare binds the compare worker-path context
Then every nested comparator field the native comparator requires is present well-formed and cross-consistent
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** The independent `context-bind` driver is the judge for this task. It reads prepared contexts only and never executes 071/072-owned workers, the observer provider, or candidate code. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No task orchestration call is allowed; no ref update targets main.
- **D-004:** Checks assert prepared-context completeness, not end-to-end worker verdicts. Full worker acceptance on TASK-002..007 remains those tasks' own gates once this repair unblocks them; this task proves the contexts are complete enough to run.
- **D-005:** All commands and result schemas are exactly the fixed proof contract. Unsupported required behavior fails; adding permissive flags, weakening demand sites, or editing other tasks' trusted inputs cannot unblock it.
- **D-006:** TASK-071 and TASK-072 stay the sole owners of `tools/refactor-proof/accounting`, `tools/refactor-proof/architecture`, and `tools/refactor-proof/bin/tc-proof`. This package inherits their accepted worker behavior through `task.toml.dependencies` and validates binding output only; no 071/072 file or receipt is a work product here.

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
- [ ] **1** Validate the accepted foundation.
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-004`, `AC-004`, `CHK-001`)
- [ ] **2** Repair only the assigned bindings.
    - [ ] **2.1** Bind the complete native oracle contract. (`R-001`, `AC-001`, `CHK-002`)
    - [ ] **2.2** Bind the complete nested comparator context. (`R-001`, `AC-006`, `CHK-003`)
    - [ ] **2.3** Bind the complete accounting preparation register. (`R-002`, `AC-003`, `CHK-004`)
    - [ ] **2.4** Bind well-formed architecture configuration and task-derived membership. (`R-002`, `AC-002`, `CHK-005`)
- [ ] **3** Qualify the fixed executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-005`, `AC-005`, `CHK-006`)
<!-- checklist:end -->
