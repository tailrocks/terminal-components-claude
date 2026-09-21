---
schema: task/v5
id: TASK-076
title: "Regenerate stale tc-proof bundle from current sources"
kind: bugfix
---

# TASK-076 — Regenerate stale tc-proof bundle from current sources

## Goal

The tracked `tools/refactor-proof/bin/tc-proof` dispatcher bundle is
regenerated from the current sources with the qualified generator, so
its sections byte-match (including the accepted TASK-075 supervision
fix), its compare path emits the runner result, and the TASK-002 verify
failures caused by the stale bundle clear.

## Context

Every task check invokes the tracked `bin/tc-proof` bundle, but that
bundle is stale: its `runner/__main__.py` section still `os.execv`s the
native comparator (bundle line 3977) while the accepted TASK-075 source
`runner/__main__.py` supervises it as a subprocess via `run_compare`
and emits the bound result via `finish()`. So the CHK-004 compare
branch writes `compare.json` but never emits
`outputs/<CHK>.result.json`, and CHK-007 close fails closed (TASK-002
verify r3: 10/11). TASK-075 was forbidden from touching the bundle.
This task is its explicit rebundle follow-up, named by TASK-075 D-006.

TASK-071 owns `accounting/` and TASK-072 owns `architecture/` plus the
shared `bin/tc-proof` dispatcher bundle; TASK-074 owns its accepted
glue and TASK-075 owns the fixed source branch. This task consumes
their accepted behavior and touches none of their sources: the sole
work product is the regenerated bundle.

This task has no rendering impact and neither reads nor writes oracle
data. Visual acceptance stays governed by the frozen oracle: any later
visual claim still needs a verifier-subagent import of the grouped
store and PTY suite read-only from the immutable `visual-baseline` tag
before using
[`visual-validation.md`](../../../visual-validation.md). Never write
`snapshots/`, bless output, or mutate the tag.

This is a repair-task package. Task dependencies are `TASK-071`,
`TASK-072`, `TASK-074`, and `TASK-075`; their accepted behavior is
prerequisite evidence this task consumes but never re-implements.
TASK-001 and TASK-070 stay retired fail-closed. UI authority is
`02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture starts from
`7b27732a8c3c131760ec3438f641cb3c11343a42`. Follow the subagent-only
host-local workflow in proof-contract.md; use standalone taskfmt only for
this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md): prepare's binding obligations and the native run layout.
- `trusted/obligations.md`: every demand site and its required rebundle result under its requirement/acceptance/check IDs.
- The qualified generator `tools/refactor-proof/architecture/rebundle.py`, the authoritative freshness check `tools/refactor-proof/tests/rebundle_check.py`, and the two trusted probes under `trusted/rebundle/`.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Accepted TASK-071, TASK-072, TASK-074, and TASK-075 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The oracle/commit pins, the task contract, the qualified generator, the freshness check, and the trusted probes are immutable. The regenerated bundle comes from the generator run against the candidate worktree — never from hand edits or candidate-written bytes.
- **Host-local verifier inputs:** The verifier subagent runs the three check commands from the worktree root, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths. No `tc-proof prepare`, no oracle import, and no mounts are involved.

## Scope

In scope:

- `tools/refactor-proof/bin/tc-proof`, regenerated only by running the qualified `tools/refactor-proof/architecture/rebundle.py` generator against the candidate tree.
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Any `refactoring-tasks/**` change — in particular no probe, obligation, or manifest edits inside this or any other task package. The fix is the generator product, not trusted-input edits.
- Every proof source: `tools/refactor-proof/runner/**` (including the TASK-075 fixed branch), `tools/refactor-proof/src/**`, `tools/refactor-proof/accounting/**` (TASK-071), `tools/refactor-proof/architecture/**` (TASK-072), `tools/refactor-proof/tests/**`, `tools/refactor-proof/scripts/**`, and `tools/refactor-proof/bin/tc-proof-host`. Consume their accepted behavior, never edit them.
- Hand edits to the bundle, hardcoded bundle bytes, oracle contact of any kind, comparator/host-core redesign, application/component repairs, new scheduler/monitor/services, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001). Every tracked bundle section byte-matches its stripped current source, and the authoritative freshness check exits 0.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-002). The tracked bundle compare path carries the accepted supervision fix: no `os.execv` anywhere, and the runner section defines `run_compare` and emits via `finish(`.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-003). The tracked bundle is executable and valid python, and the qualified generator reproduces it byte-identical into a temp file.
- **R-004 (MUST NOT):** Violate any prohibition mapped to R-004 in `trusted/obligations.md` (O-004). Regenerate only via the qualified generator — no hand edits, no source changes, no task-package changes, no oracle contact, no hardcoded bundle bytes.

## Acceptance criteria

### AC-001 — Bundle sections match current sources
```gherkin
Given the current proof sources and the qualified rebundle generator
When the tracked bundle sections are compared against the stripped sources
Then every section byte-matches and the authoritative freshness check exits 0
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Bundle compare path supervises and emits
```gherkin
Given the tracked dispatcher bundle
When its compare path is inspected for the accepted supervision fix
Then no os.execv call remains and the runner section carries run_compare with finish emission
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-002`

### AC-003 — Regeneration gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-003`

## Fixed decisions

- **D-001:** The authoritative freshness check plus the two trusted probes are the judges for this task. The freshness check compares bundle sections against current sources; the supervision probe asserts the compare-path fix; the regeneration probe asserts executability, validity, and byte-identical generator reproduction. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No task orchestration call is allowed; no ref update targets main.
- **D-004:** The qualified `architecture/rebundle.py` generator is the only writer of the bundle. Its output is deterministic: a correct regeneration is byte-identical on re-run and needs no follow-up touch-up. Any hand edit or hardcoded byte fails review even if the judges pass.
- **D-005:** All commands and result schemas are exactly the fixed proof contract. This task needs no `tc-proof prepare` run and no oracle contact; adding permissive flags, weakening demand sites, or editing trusted inputs cannot unblock it.
- **D-006:** TASK-071 and TASK-072 stay the sole owners of `tools/refactor-proof/accounting`, `tools/refactor-proof/architecture`, and the `tools/refactor-proof/bin/tc-proof` design; TASK-074 and TASK-075 stay the owners of their accepted glue and branch fix. This package inherits their accepted behavior through `task.toml.dependencies` and validates the regenerated bundle only; no 071/072/074/075 source or receipt is a work product here. The TASK-002 re-run is an explicit follow-up.
- **D-007:** The stale-bundle failure is a harness defect, not a product defect. This task changes no product rendering or behavior; full worker acceptance on TASK-002..007 remains those tasks' own gates once this rebundle unblocks them.

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
- [ ] **1** Confirm the stale bundle.
    - [ ] **1.1** Run the authoritative freshness check and record the mismatch. (`R-001`, `AC-001`, `CHK-001`)
- [ ] **2** Regenerate the bundle with the qualified generator only.
    - [ ] **2.1** Regenerate bin/tc-proof and re-run the freshness check to green. (`R-001`, `AC-001`, `CHK-001`)
    - [ ] **2.2** Prove the compare path supervises and emits. (`R-002`, `AC-002`, `CHK-002`)
- [ ] **3** Qualify the regenerated bundle.
    - [ ] **3.1** Run the regeneration gate with actual logs. (`R-003`, `R-004`, `AC-003`, `CHK-003`)
<!-- checklist:end -->
