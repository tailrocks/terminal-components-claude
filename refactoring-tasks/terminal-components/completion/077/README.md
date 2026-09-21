---
schema: task/v5
id: TASK-077
title: "Accept components native oracle namespace in harness gate"
kind: bugfix
---

# TASK-077 — Accept components native oracle namespace in harness gate

## Goal

The harness oracle gate accepts the `components` native namespace, so
TASK-006 `CHK-003` (`bin/tc-proof oracle --namespace components`)
passes validation instead of failing `PROTOCOL`, the tracked bundle
carries the same fixed allowlist, and the TASK-006 verify failures
caused by the rejection clear.

## Context

Every native oracle check invokes the tracked `bin/tc-proof` bundle,
but the gate rejects the catalog's `components` namespace: TASK-006's
contract requires `bin/tc-proof oracle --namespace components`
(verify.toml CHK-003, in catalog since `51dd9559`) and prepare
correctly stamps family=native, yet the runner rejects `PROTOCOL` at
`tools/refactor-proof/runner/context.py:40`, where
`NATIVE_ORACLE_NAMESPACES` holds only `{showcase, holla, jackin,
tablepro}` (introduced by `b96a62e4` for app namespaces only). The
CHK-007 close cascades (TASK-006 verify r2: FAIL 9/11).
Coordinator-confirmed: `validate_native_extension` plus all downstream
oracle flow is namespace-agnostic, and Rust-side namespace hits are
`#[test]`-only. The ONLY production gap is the Python allowlist. Since
checks invoke the generated bundle, the fix also regenerates
`tools/refactor-proof/bin/tc-proof` via the qualified
`tools/refactor-proof/architecture/rebundle.py` generator.

TASK-071 owns `accounting/` and TASK-072 owns `architecture/` plus the
shared `bin/tc-proof` dispatcher design; TASK-074 owns its accepted
glue, TASK-075 owns the fixed source branch, and TASK-076 owns the
regenerated bundle. This task consumes their accepted behavior and
changes exactly one set member in one runner source plus its regenerated
bundle.

This task has no rendering impact and neither reads nor writes oracle
data. Visual acceptance stays governed by the frozen oracle: any later
visual claim still needs a verifier-subagent import of the grouped
store and PTY suite read-only from the immutable `visual-baseline` tag
before using
[`visual-validation.md`](../../../visual-validation.md). Never write
`snapshots/`, bless output, or mutate the tag.

This is a repair-task package. Task dependencies are `TASK-071`,
`TASK-072`, `TASK-074`, `TASK-075`, and `TASK-076`; their accepted
behavior is prerequisite evidence this task consumes but never
re-implements. TASK-001 and TASK-070 stay retired fail-closed. UI
authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architecture
starts from `7b27732a8c3c131760ec3438f641cb3c11343a42`. Follow the
subagent-only host-local workflow in proof-contract.md; use standalone
taskfmt only for this package's lint and verify evidence.

Read before editing:

- `CAMPAIGN_AGENTS.md`: repository scope and integration constraints; this task-local AGENTS.md defines subagent execution and verification.
- [`proof-contract.md`](../../../../docs/refactoring-plan/proof-contract.md): prepare's binding obligations and the native run layout.
- `trusted/obligations.md`: every demand site and its required namespace result under its requirement/acceptance/check IDs.
- `validate_oracle_namespace` in `tools/refactor-proof/runner/context.py`, the qualified generator `tools/refactor-proof/architecture/rebundle.py`, the authoritative freshness check `tools/refactor-proof/tests/rebundle_check.py`, and the three trusted probes under `trusted/namespace/`.

## Preconditions

- **P-001:** The coordinator has pinned the immutable task/catalog/bootstrap, taskfmt revision `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, accepted tui-snap PR/revision and toolchain/lock fingerprints.
- **P-002:** Accepted TASK-071, TASK-072, TASK-074, TASK-075, and TASK-076 evidence and actual integrated source ancestry resolve from protected receipts; the candidate starts at the recorded parent and scope base.
- **P-003:** The oracle/commit pins, the task contract, the qualified generator, the freshness check, and the trusted probes are immutable. The source change is exactly the one added set member; the regenerated bundle comes from the generator run against the candidate worktree — never from hand edits or candidate-written bytes.
- **Host-local verifier inputs:** The verifier subagent runs the three check commands from the worktree root, then standalone taskfmt verify with explicit `--task-dir`, `--root`, `--base`, and `--log-dir` paths. No `tc-proof prepare`, no oracle import, and no mounts are involved.

## Scope

In scope:

- `tools/refactor-proof/runner/context.py`, changed only by adding the `'components'` member to `NATIVE_ORACLE_NAMESPACES`.
- `tools/refactor-proof/bin/tc-proof`, regenerated only by running the qualified `tools/refactor-proof/architecture/rebundle.py` generator against the candidate tree.
- Independent qualification outputs in verifier-subagent-owned external run directories; no production application changes.

Out of scope:

- Any `refactoring-tasks/**` change — in particular no probe, obligation, or manifest edits inside this or any other task package. The fix is the source member plus the generator product, not trusted-input edits.
- Every other proof source: all other `tools/refactor-proof/runner/**` files, `tools/refactor-proof/src/**` (including `src/verifier.rs`), `tools/refactor-proof/accounting/**` (TASK-071), `tools/refactor-proof/architecture/**` (TASK-072), `tools/refactor-proof/tests/**`, `tools/refactor-proof/scripts/**`, and `tools/refactor-proof/bin/tc-proof-host`. Consume their accepted behavior, never edit them.
- The proof adapters (`tools/refactor-proof-adapters/**`): the fix is the harness gate, not adapter edits.
- Hand edits to the bundle, hardcoded bundle or probe bytes, oracle contact of any kind, comparator/host-core redesign, application/component repairs, new scheduler/monitor/services, publication or merges.

## Requirements

- **R-001 (MUST):** Satisfy every exact clause mapped to R-001 in `trusted/obligations.md` (O-001). `NATIVE_ORACLE_NAMESPACES` is exactly `{showcase, holla, jackin, tablepro, components}` in both the runner source and the tracked bundle, and the authoritative freshness check exits 0.
- **R-002 (MUST):** Satisfy every exact clause mapped to R-002 in `trusted/obligations.md` (O-002). The bundled gate accepts `components` and the four app namespaces with a native family, still rejects unknown/empty/missing namespaces with `PROTOCOL`, and still requires exactly `'synthetic'` for the synthetic family.
- **R-003 (MUST):** Satisfy every exact clause mapped to R-003 in `trusted/obligations.md` (O-003). The tracked bundle is executable and valid python, the qualified generator reproduces it byte-identical into a temp file, and the staged generator product carries the fixed allowlist.
- **R-004 (MUST NOT):** Violate any prohibition mapped to R-004 in `trusted/obligations.md` (O-004). Add only the `'components'` member — no other allowlist or validation change and no source change beyond the one set member; regenerate only via the qualified generator — no hand edits, no other source changes, no task-package changes, no oracle contact, no hardcoded bytes.

## Acceptance criteria

### AC-001 — Allowlist carries components in source and bundle
```gherkin
Given the runner source and the tracked dispatcher bundle
When the native oracle allowlist is extracted from each
Then both hold exactly the five-member set with components and the freshness check exits 0
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Bundle gate accepts components, still rejects the rest
```gherkin
Given the tracked dispatcher bundle loaded as a module
When the oracle namespace gate is exercised over native and synthetic families
Then components and the app namespaces pass while unknown and mismatched names fail closed
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

- **D-001:** The three trusted probes are the judges for this task. The allowlist probe asserts the exact five-member set in source and bundle plus freshness; the gate-behavior probe asserts the runtime accept/reject contract against the loaded bundle; the regeneration probe asserts executability, validity, byte-identical generator reproduction, and that the staged product carries the fix. No self-test or printed success marker is acceptance.
- **D-002:** UI oracle remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural starting point remains `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- **D-003:** Use existing canonical taskfmt standalone verification and current tui-snap primitives. No task orchestration call is allowed; no ref update targets main.
- **D-004:** The source change is exactly the one added allowlist member, and the qualified `architecture/rebundle.py` generator is the only writer of the bundle. Its output is deterministic: a correct regeneration is byte-identical on re-run and needs no follow-up touch-up. Any hand edit, hardcoded byte, or validation weakening fails review even if the judges pass.
- **D-005:** All commands and result schemas are exactly the fixed proof contract. This task needs no `tc-proof prepare` run and no oracle contact; adding permissive flags, weakening demand sites, or editing trusted inputs cannot unblock it.
- **D-006:** TASK-071 and TASK-072 stay the sole owners of `tools/refactor-proof/accounting`, `tools/refactor-proof/architecture`, and the `tools/refactor-proof/bin/tc-proof` design; TASK-074, TASK-075, and TASK-076 stay the owners of their accepted glue, branch fix, and regenerated bundle. This package inherits their accepted behavior through `task.toml.dependencies` and validates the one-member fix plus its regeneration only; no 071/072/074/075/076 source or receipt is a work product here. The TASK-006 re-run is an explicit follow-up.
- **D-007:** The rejected namespace is a harness defect, not a product defect. This task changes no product rendering or behavior; full worker acceptance on TASK-002..007 remains those tasks' own gates once this gate fix unblocks them.

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
- [ ] **1** Confirm the rejected namespace.
    - [ ] **1.1** Run the allowlist probe and record the missing member. (`R-001`, `AC-001`, `CHK-001`)
    - [ ] **1.2** Run the gate-behavior probe and record the PROTOCOL rejection. (`R-002`, `AC-002`, `CHK-002`)
- [ ] **2** Add the member and regenerate with the qualified generator only.
    - [ ] **2.1** Add only 'components' to the source allowlist. (`R-001`, `R-004`, `AC-001`, `CHK-001`)
    - [ ] **2.2** Regenerate bin/tc-proof and prove the gate accepts. (`R-001`, `R-002`, `R-004`, `AC-001`, `AC-002`, `CHK-001`, `CHK-002`)
- [ ] **3** Qualify the regenerated bundle.
    - [ ] **3.1** Run the regeneration gate with actual logs. (`R-003`, `R-004`, `AC-003`, `CHK-003`)
<!-- checklist:end -->
