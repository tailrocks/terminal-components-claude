---
schema: task/v5
id: TASK-078
title: "Project bound dependency receipts into production accounting claims"
kind: bugfix
---

# TASK-078 — Project bound dependency receipts into production accounting claims

## Goal

`tc-proof prepare` honors `requires_inventory_receipt` and
`requires_disposition_receipt` in production-mode accounting contexts by
projecting the bound accepted+integrated dependency stubs into
`accepted_inventory` and `accepted_disposition`, so production
account-tests checks clear the worker `TEST_ACCOUNTING` gate while
preparation mode stays byte-identical in behavior.

## Context

Every catalog production account-tests check (tasks 009-069 plus 073
CHK-005) declares `mode=production` with both requires flags true, but
`bind_accounting_preparation` (`tools/refactor-proof/src/verifier.rs`)
never inserts the accepted claims: its only receipt handling strips
both keys in non-production mode. The worker
(`tools/refactor-proof/accounting/extension.py`) rejects any
production contract without truthy `accepted_inventory` and
`accepted_disposition`, so all of those checks fail with
`TEST_ACCOUNTING`.

The repair input already exists at bind time: bound, accepted,
integrated, ancestry-verified dependency receipts sit in
`qualification.common.dependencies` (built by `dependency_receipts`,
cloned into every qualification by `build_context` before
`bind_accounting_preparation` runs). Nothing projects them into the
accepted claims. The intended repair, in prepare only and in production
mode only: when a requires flag is true, set the matching accepted
claim to a provenance object derived solely from those bound stubs
(`{task_id, sha256, integration_commit}` per receipt); fail closed when
a required receipt is unbound.

TASK-071 owns `accounting/` and TASK-072 owns `architecture/` plus the
shared `bin/tc-proof` dispatcher bundle; TASK-074 owns the accepted
binding family and runner glue; TASK-075 owns the accepted five-key
inventory shape in the same function this task extends. This task
touches none of those files — only `verifier.rs`.

Visual acceptance stays governed by the frozen oracle: a verifier subagent
must import the grouped store and PTY suite read-only from the immutable
`visual-baseline` tag into the external run before using
[`visual-validation.md`](../../../visual-validation.md). Never write
`snapshots/`, bless output, or mutate the tag.

This is a repair-task package. Task dependencies are `TASK-071`,
`TASK-072`, `TASK-074`, and `TASK-075`; their accepted behavior is
prerequisite evidence this task consumes but never re-implements:
071/072 own the demand-side worker/bundle behavior, 074 owns the
accepted binding family, and 075 owns the accepted inventory shape in
the function under repair. TASK-001 and TASK-070 stay retired
fail-closed. UI authority is
`02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from
`7b27732a8c3c131760ec3438f641cb3c11343a42`. Follow the subagent-only
host-local workflow in proof-contract.md; use standalone taskfmt only for
this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md): prepare's binding obligations and the native run layout.
- `trusted/obligations.md`: every demand site and its required binding under its requirement/acceptance/check IDs.
- The failing demand sites: `tools/refactor-proof/src/verifier.rs` `bind_accounting_preparation`/`build_context`/`dependency_receipts` and `tools/refactor-proof/accounting/extension.py` production branch.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Accepted TASK-071, TASK-072, TASK-074, and TASK-075 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The oracle/commit pins, the task contract, and the trusted driver/templates are immutable. Derived bindings come from the task manifest, the oracle, and the candidate worktree — never from candidate-written expectations.
- **Host-local verifier inputs:** The verifier subagent builds the native proof tool with `scripts/campaign-build-proof.sh`, runs `tc-proof prepare` for this package, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths and `RUN_DIR` exported. Per-check contexts resolve under `$RUN_DIR/contexts/`. No mounts are involved.

## Scope

In scope:

- `tools/refactor-proof/src/verifier.rs` for the named bind-function repair only (`bind_accounting_preparation` and its direct receipt-projection helpers) and narrow directly necessary colocated tests, including the R-002 fail-closed unit test.
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Any `refactoring-tasks/**` change — in particular no template, driver, or manifest edits inside this package or any other task package. The fix lives in prepare, not in trusted inputs.
- `tools/refactor-proof/accounting/**` (TASK-071), `tools/refactor-proof/architecture/**` (TASK-072), and `tools/refactor-proof/bin/tc-proof` (shared 071/072 bundle): consume their accepted behavior, never edit them.
- TASK-074's accepted runner glue (`runner/*.py` in full): consume it, never edit it.
- Demand-side weakening: `tools/refactor-proof/src/compare.rs`, `tools/refactor-proof/src/bin/`, and runner/accounting/architecture validators keep their exact requirements. No allowlist thinning, no truthiness relaxation, no threshold change.
- Comparator/host-core redesign, application/component repairs, oracle baseline capture/approval, new scheduler/monitor/services, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001). In production mode only, when a template requires flag is true, bind the matching accepted claim as a provenance object derived solely from the bound `common.dependencies` stubs, with the receipt set equal to the bound stub set.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-002). Fail closed when a required receipt is unbound: prepare must refuse the context rather than fabricate or default a claim. This negative path is evidenced by a colocated Rust `#[test]` in `verifier.rs` plus a reviewer-run `cargo nextest run -p refactor-proof --lib <filter>` recording; no verify.toml shell runs cargo.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-003). Change no other binding: preparation mode stays byte-identical in behavior (accepted claims still stripped, `original`/`required`/`preparation_register` intact) and every preserved binding is re-asserted on the current tree.
- **R-004 (MUST NOT):** Violate any prohibition mapped to R-004 in `trusted/obligations.md` (O-004). Never edit task packages, 071/072-owned files, 074's accepted glue, or demand-side validators; never hardcode digests, receipts, commits, frames, states, or membership to satisfy the judge; never derive an accepted claim from anything but the bound stubs; never substitute candidate-written expectations for manifest/oracle/worktree derivation.

## Acceptance criteria

### AC-001 — Production accepted claims project bound stubs

```gherkin
Given the declared production template with both requires flags true and the bound accepted integrated dependency stubs
When prepare binds the production account-tests worker-path context
Then accepted inventory and accepted disposition each carry exactly the bound stub receipts and a required but unbound receipt fails closed
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`, `R-002`
- **Check:** `CHK-002`

### AC-002 — Preparation mode unchanged

```gherkin
Given the declared preparation template and the pinned source registers
When prepare binds the preparation account-tests worker-path context
Then original required future and the preparation register stay intact and both accepted claims stay stripped
```

**Verification**

- **Type:** scenario
- **Covers:** `R-003`
- **Check:** `CHK-003`

### AC-003 — Keep authority immutable

```gherkin
Given the campaign-pinned task contract and protected source and verification inputs
When source identity ownership scope and receipt authority are validated
Then every R-004 prohibition holds and no candidate-selected artifact defines success
```

**Verification**

- **Type:** invariant
- **Covers:** `R-004`
- **Check:** `CHK-001`

### AC-004 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-004`

## Fixed decisions

- **D-001:** The independent `receipt-bind` driver is the judge for this task. It reads prepared contexts for the binding profiles only; it never executes workers, the observer provider, or the dispatcher bundle. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No task orchestration call is allowed; no ref update targets main.
- **D-004:** Checks assert genuine runnable bindings, not end-to-end worker verdicts. Full worker acceptance on catalog production checks remains those tasks' own gates once this repair unblocks them; the fail-closed negative path is evidenced by the reviewer-run colocated unit test because a refused context leaves no artifact for the driver to judge.
- **D-005:** All commands and result schemas are exactly the fixed proof contract. Unsupported required behavior fails; adding permissive flags, weakening demand sites, or editing other tasks' trusted inputs cannot unblock it.
- **D-006:** TASK-071 and TASK-072 stay the sole owners of `tools/refactor-proof/accounting`, `tools/refactor-proof/architecture`, and `tools/refactor-proof/bin/tc-proof`; TASK-074 stays the owner of its accepted glue; TASK-075 stays the owner of the accepted inventory shape. This package inherits their accepted behavior through `task.toml.dependencies` and validates binding output only; no 071/072/074/075 file or receipt is a work product here. The catalog production re-runs are explicit follow-ups.
- **D-007:** The accounting worker is immutable. Claims must satisfy its exact production gate (truthy accepted objects); any implementation that relaxes the worker, fabricates a receipt, or defaults an unbound requirement fails review even if the judge passes.

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
    - [ ] **1.1** Verify protected tools, source pins, receipts and scope. (`R-004`, `AC-003`, `CHK-001`)
- [ ] **2** Repair only the assigned binding.
    - [ ] **2.1** Project bound stubs into production accepted claims with fail-closed refusal. (`R-001`, `R-002`, `AC-001`, `CHK-002`)
    - [ ] **2.2** Keep preparation mode byte-identical with accepted claims stripped. (`R-003`, `AC-002`, `CHK-003`)
- [ ] **3** Qualify the fixed executable.
    - [ ] **3.1** Run every independent case and the complete gate with actual logs. (`R-001`, `R-002`, `R-003`, `R-004`, `AC-004`, `CHK-004`)
<!-- checklist:end -->
