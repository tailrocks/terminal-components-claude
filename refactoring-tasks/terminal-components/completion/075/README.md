---
schema: task/v5
id: TASK-075
title: "Repair compare roots, accounting inventory, and compare supervision"
kind: bugfix
---

# TASK-075 — Repair compare roots, accounting inventory, and compare supervision

## Goal

`tc-proof prepare` binds genuine runnable compare roots and a five-key
source-derived accounting inventory, and the source `runner/__main__.py`
compare branch supervises the native comparator and emits its runner
result, so the three TASK-002 verify failures clear.

## Context

TASK-002 verify fails on candidate `fc955f56` (scope base `eacea930`,
with TASK-074 accepted+integrated): `CHK-004` compare fails with
`UNSAFE_PATH`, `CHK-005` account-tests fails with `TEST_ACCOUNTING`, and
`CHK-007` close fails because no compare runner result exists. The three
gaps all live in TASK-074's scope family — `verifier.rs` bind functions
plus the `runner/__main__.py` compare branch:

- `bind_compare_qualification` binds `oracle_root` to the git object store
  (the `.git` dir) and `candidate_root` to the whole worktree, with
  domain-derived manifest digests: no `manifest.json`, symlinks present.
- `bind_accounting_preparation` binds `required` as one-key `[{id: ...}]`
  rows while the worker accounts `{package, target, profile,
  source_commit, name}` seen keys.
- the compare branch `os.execv`s the native comparator, which writes only
  the comparison report; `outputs/CHK-004.result.json` is never emitted.

TASK-071 owns `accounting/` and TASK-072 owns `architecture/` plus the
shared `bin/tc-proof` dispatcher bundle; TASK-074's accepted binding logic
and runner glue stay untouched except the named bind functions. This task
touches none of those files.

Visual acceptance stays governed by the frozen oracle: a verifier subagent
must import the grouped store and PTY suite read-only from the immutable
`visual-baseline` tag into the external run before using
[`visual-validation.md`](../../../visual-validation.md). Never write
`snapshots/`, bless output, or mutate the tag.

This is a repair-task package. Task dependencies are `TASK-071`,
`TASK-072`, and `TASK-074`; their accepted behavior is prerequisite
evidence this task consumes but never re-implements. TASK-001 and TASK-070
stay retired fail-closed. UI authority is
`02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from
`7b27732a8c3c131760ec3438f641cb3c11343a42`. Follow the subagent-only
host-local workflow in proof-contract.md; use standalone taskfmt only for
this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md): prepare's binding obligations and the native run layout.
- `trusted/obligations.md`: every demand site and its required binding under its requirement/acceptance/check IDs.
- `trusted/bind-repair/bind-repair-protocol.md`: the exact independent judge, profiles, and verdicts.
- The failing demand sites: `tools/refactor-proof/src/verifier.rs` `bind_compare_qualification`/`bind_accounting_preparation`, `tools/refactor-proof/src/compare.rs` root/manifest/required-set/provenance checks, `tools/refactor-proof/accounting/extension.py` seen-keys accounting, `tools/refactor-proof/runner/__main__.py` compare branch, and `tools/refactor-proof/runner/validate.py` close validation.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Accepted TASK-071, TASK-072, and TASK-074 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The oracle/commit pins, the task contract, and the trusted driver/templates/loader are immutable. Derived bindings come from the task manifest, the oracle, and the candidate worktree — never from candidate-written expectations.
- **Host-local verifier inputs:** The verifier subagent builds the native proof tool with `scripts/campaign-build-proof.sh`, runs `tc-proof prepare` for this package, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths and `RUN_DIR` exported. Per-check contexts resolve under `$RUN_DIR/contexts/`. No mounts are involved.

## Scope

In scope:

- `tools/refactor-proof/src/verifier.rs` for the named bind-function repair only (`bind_compare_qualification`, `bind_accounting_preparation`, and their direct root/inventory helpers) and narrow directly necessary inline tests.
- `tools/refactor-proof/runner/__main__.py` for the compare-branch supervision repair only (subprocess + `finish()` instead of `execv`).
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Any `refactoring-tasks/**` change — in particular no template, driver, loader, or manifest edits inside TASK-002..007 or any other task package. The fix lives in prepare and the source branch, not in other tasks' trusted dirs.
- `tools/refactor-proof/accounting/**` (TASK-071), `tools/refactor-proof/architecture/**` (TASK-072), and `tools/refactor-proof/bin/tc-proof` (shared 071/072 bundle): consume their accepted behavior, never edit them. The 072-owned rebundle of the frozen dispatcher from the fixed source is an explicit follow-up, not this task's work product.
- TASK-074's accepted runner glue (`runner/context.py`, `runner/operations.py`, `runner/validate.py`, and every other `runner/*.py` file): consume it, never edit it.
- Demand-side weakening: `tools/refactor-proof/src/compare.rs`, `tools/refactor-proof/src/bin/`, and runner/accounting/architecture validators keep their exact requirements. No allowlist thinning, no symlink tolerance, no threshold change.
- Comparator/host-core redesign, application/component repairs, oracle baseline capture/approval, new scheduler/monitor/services, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001). Bind real symlink-free artifact trees as compare roots — never the worktree root or the git dir — with `manifest.json`, scenario files, and recomputed digests, derived from the task manifest, the oracle, and the candidate worktree.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-002). Bind the required inventory and preparation register as exact five-key source-derived identities matching the worker's seen keys.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-003). Supervise the native comparator as a subprocess from the source compare branch and emit the bound runner result via `finish()`, returning to the caller.
- **R-004 (MUST):** Satisfy every exact clause mapped to R-004 in `trusted/obligations.md` (O-004). Preserve every binding prepare already emits: identity, trust, tool, dependency (now including TASK-074), index, result, template-flow, manifest-agreement, nested comparator agreement, and task-derived membership bindings.
- **R-005 (MUST NOT):** Violate any prohibition mapped to R-005 in `trusted/obligations.md` (O-005). Never edit task packages, 071/072-owned files, 074's accepted glue, or demand-side validators; never hardcode digests, manifests, frames, states, or membership to satisfy the judge; never substitute candidate-written expectations for manifest/oracle/worktree derivation.
- **R-006 (MUST):** The complete independently judged gate passes on the exact fixed candidate tree with all required outputs and unchanged trusted inputs.

## Acceptance criteria

### AC-001 — Real compare roots
```gherkin
Given the pinned oracle and the required comparison set
When prepare binds the compare worker-path context
Then both roots are genuine symlink-free artifact trees with manifests scenario files and recomputed digests and neither is the worktree root or the git dir
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-002`

### AC-002 — Five-key accounting inventory
```gherkin
Given the declared preparation template and the pinned source registers
When prepare binds the account-tests worker-path context
Then required and the preparation register carry exact five-key source-derived identities matching the worker seen keys
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-003`

### AC-003 — Compare supervision emits a runner result
```gherkin
Given the bound compare context and the native comparator
When the source compare branch runs supervised without preset result bindings
Then it returns after the comparator exits and emits the bound runner result with coherent status and the comparison report
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-004`

### AC-004 — Keep authority immutable
```gherkin
Given the campaign-pinned task contract and protected source and verification inputs
When source identity ownership scope and receipt authority are validated
Then every R-005 prohibition holds and no candidate-selected artifact defines success
```

**Verification**

- **Type:** invariant
- **Covers:** `R-005`
- **Check:** `CHK-001`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-005`

## Fixed decisions

- **D-001:** The independent `bind-repair` driver is the judge for this task. It reads prepared contexts for the binding profiles and executes the fixed source compare branch exactly once, as a subprocess, for the supervision profile; it never executes 071/072-owned workers, the observer provider, or the frozen dispatcher bundle. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No task orchestration call is allowed; no ref update targets main.
- **D-004:** Checks assert genuine runnable bindings plus supervised emission, not end-to-end worker verdicts. Full worker acceptance on TASK-002..007 remains those tasks' own gates once this repair unblocks them; frame/state byte equality stays the comparator's verdict, and the supervision profile requires status/exit coherence, not a forced pass.
- **D-005:** All commands and result schemas are exactly the fixed proof contract. Unsupported required behavior fails; adding permissive flags, weakening demand sites, or editing other tasks' trusted inputs cannot unblock it.
- **D-006:** TASK-071 and TASK-072 stay the sole owners of `tools/refactor-proof/accounting`, `tools/refactor-proof/architecture`, and `tools/refactor-proof/bin/tc-proof`; TASK-074 stays the owner of its accepted glue. This package inherits their accepted behavior through `task.toml.dependencies` and validates binding output plus the named branch behavior only; no 071/072/074 file or receipt is a work product here. The 072-owned dispatcher rebundle and the TASK-002 re-run are explicit follow-ups.
- **D-007:** The comparator is immutable. Roots must satisfy its exact checks (symlink scan, manifest digests, required-set equality, provenance agreement); any implementation that thins its allowlist, tolerates symlinks, or weakens a threshold fails review even if the judge passes.

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
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-005`, `AC-004`, `CHK-001`)
- [ ] **2** Repair only the assigned bindings and branch.
    - [ ] **2.1** Bind real symlink-free compare roots with manifests and recomputed digests. (`R-001`, `AC-001`, `CHK-002`)
    - [ ] **2.2** Bind the five-key source-derived accounting inventory and register. (`R-002`, `AC-002`, `CHK-003`)
    - [ ] **2.3** Supervise the comparator and emit the runner result via finish. (`R-003`, `AC-003`, `CHK-004`)
- [ ] **3** Qualify the fixed executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-006`, `AC-005`, `CHK-005`)
<!-- checklist:end -->
