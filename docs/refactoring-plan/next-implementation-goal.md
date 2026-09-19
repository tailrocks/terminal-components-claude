# NOT AUTHORIZED FOR EXECUTION — next implementation `/goal`

This is the complete future implementation prompt generated from the current
preparation package. It is **not executable**. The current readiness report is
NO-GO, the current ledger must remain `armed=false`, and no production task may
be dispatched. The prompt becomes eligible only after a separate readiness
revalidation and explicit authorization.

## Objective

On `refactor/holla-parity`, complete every valid reconciled refactoring task,
preserve the frozen product contract, and produce merge-readiness evidence for
`main`. The desired result is all three at once:

> intended refactoring complete + original behavior correct + exact original
> visual representation.

Do not merge into `main` without separate permission. Stop at merge readiness.

## Immutable inputs and exact identities

Before any dispatch, read `AGENTS.md`, the current readiness report, all
campaign contracts, the task graph, task packages, product architecture, and
the visual validation contract. Then require these identities to be freshly
revalidated; the values below are preparation references, not permission:

```text
campaign preparation payload at documentation start:
  branch refactor/holla-parity
  HEAD f6f94dc995f5b6451800d739174d7f23802a40c3
  tree 61563f7b48d0fe7b8e5bdddae57072e96f6a6f12
  parent e8c4950928b0ab6cc1268777dbed6f96cb0309ba

protected visual tag:
  refs/tags/visual-baseline^{commit}
  4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
  tag tree 0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
  snapshot tree 3f0261c32849e26feda24d87697de4a7ce6b8375
  7,550 keys; 30,200 artifacts; 7,550 each ANSI/plain/PNG/HTML

complete visual snapshot source:
  commit 89218626011f2f82c4e87c4dfd5868a4c5f3e284
  tree 6fccf997cd742071ebcff0e0a00e89404ef95ca8
  snapshot tree 3f0261c32849e26feda24d87697de4a7ce6b8375

architecture/source oracle:
  commit 02f5294bfdbf38004cc49130d0aff1d01f31434c
  tree efa2b409b77077caf5c639f7f4c6154cbadbce5

qualified taskfmt:
  source /Users/donbeave/Projects/taskfmt/task-format
  revision afd3b575dbcc7044620bec4b9493a74eca3e5ef2
  version 0.2.0
  binary /tmp/taskfmt-latest-install/bin/taskfmt
  SHA-256 f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

The exact read-only oracle import is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`
with manifest
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19`.
Do not use `3570a2ed23444dddf1eddcdcc49b654b169038fe` as a complete oracle
source: it has only 28,580 artifacts / 7,145 keys.

## Startup gate — reject dispatch now

The implementation coordinator must exit nonzero before spawning any
implementer if any condition below is false:

1. The readiness report says `**GO.**`, not NO-GO, and its exact current
   commit/tree/parent is independently sealed. The current report says
   NO-GO: therefore this prompt is **NOT AUTHORIZED FOR EXECUTION**.
2. The protected tag peel, tag tree, snapshot tree, grouped oracle store,
   fixtures, manifests, and expected artifacts match the exact read-only
   import. Any mismatch stops the run.
3. The candidate branch, local/remote `main`, merge-bases, scope base, and
   clean worktree are freshly read from Git. Never infer branch identity from a
   directory name. Never reset, force-push, prune, or modify another worktree.
4. The ledger is schema-valid, current, and `armed=false`; no production row
   is accepted from inspection, lint, stale evidence, or a future result. A
   separate explicit authorization step must occur after readiness recheck and
   before arming dispatch.
5. The 73-package catalog and generated DAG validate: 506 checks, 276 edges,
   maximum depth 35, no cycles/dangling IDs/conflicts, and all original tasks
   are mapped. `TASK-001`/`TASK-070` remain retired fail-closed tasks;
   `TASK-071`/`TASK-072` require fresh accepted qualification before use;
   valid implementation work is `TASK-002`–`TASK-069` and `TASK-073` unless a
   freshly reviewed contract reconciliation changes that disposition.
6. Taskfmt is exactly the qualified source/version/binary above. If source,
   version, or binary hash changes, stop and requalify. Use only:

   ```sh
   taskfmt lint "$TASK_DIR"
   taskfmt verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
     --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
   ```

   No taskfmt lifecycle, host, runtime, start, monitor, dispatch, promotion,
   workspace, ref, or container command is allowed.
7. Native proof preparation is independently built and bound to the final
   source/tree, external regular target, binary hash, exact contexts/index,
   observer transport, nonce, comparator, oracle, task contract, scope base,
   environment, and dependency receipts. Symlink/hard-link substitutions,
   stale/cross-run inputs, extra/duplicate contexts, forged results, wrong
   hashes, missing observer evidence, and mutated trust inputs must fail closed.
8. A fresh independent verifier and a separate independent reviewer both
   return `VERIFIED` for this exact tree. The preserved rejected roots are not
   receipts:

   ```text
   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-proof-requal-b20ca5c6
   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/reviewer-final-proof-requal-b20ca5c6
   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-211c29ad-independent
   ```
9. Native macOS and required native Linux evidence are available. Containers,
   Docker, Podman, mounts, firmlinks, and namespaces are forbidden. If Linux
   is unavailable, reject startup.

No item may be bypassed by changing the report, accepting snapshots, adding a
skip/allow-failure, or treating an unavailable check as pass.

## Delegation and integration protocol

When the gate eventually passes, use only bounded host-local subagents. Every
subagent must be configured as `gpt-5.6-luna` with `max` reasoning effort.
Record actual model/effort, source identity, assigned task IDs, writable paths,
commands, exit codes, raw evidence, and unresolved findings. Never claim a
spawn, model, review, or acceptance that did not occur.

For each task use separate roles:

- implementer: isolated worktree, scoped change, focused nextest/static tests;
- verifier: frozen committed candidate, verifier-owned external `RUN_DIR` and
  target, native proof materialization, taskfmt lint/verify, behavior/visual
  evidence, and raw results;
- independent reviewer: separate worktree/read-only candidate, provenance,
  architecture ownership, scope, forbidden mutation, and result review;
- coordinator: DAG scheduling and serial compare-and-swap integration only.

No roles share writable worktrees, targets, run directories, oracle stores, or
generated outputs. Integration checks the expected parent, dependency receipt
ancestry, exact candidate tree, and clean state. A stale candidate is rejected.
Every accepted result is invalidated by relevant source, task contract, schema,
tool, comparator, oracle, environment, or documentation changes.

Schedule the generated DAG, not a hand-edited list. Respect its bounded waves,
shared interfaces, file ownership, migration boundaries, chokepoints, and
integration checkpoints. Parallelize only genuinely disjoint tasks; serialize
application/shared-surface work. Never require a task's own future receipt to
start it, and never use final-product success as a prerequisite for the first
implementation task.

## Product implementation obligations

Complete all valid reconciled tasks. Preserve public API intent or record an
explicit reviewed migration. Component owners must own state, draw, hit
testing, focus, hover, keyboard, selection, scrolling, resizing, disabled,
empty/loading/error, overlays, tables, forms, lists, panels, and navigation.
Restore application/showcase integration, Holla routes and scenarios, Jackin
and TablePro behavior, persistence/async transitions, startup, input,
terminal restoration, and shutdown. Remove duplicate renderers, compatibility
painters, fixed historical frame text, app-local repaint overlays, copied
oracle frames, and any parallel legacy implementation used to hide incomplete
migration. The final source must be architecturally complete, not merely
visually patched.

## Verification gates

For each task, after prerequisites are accepted:

1. inspect source/history/contracts and define the affected baseline surface;
2. implement in the isolated task worktree;
3. run qualified `cargo nextest` and required static/API checks; never run
   `cargo test`;
4. build native proof in an external target and materialize exact contexts;
5. run only standalone taskfmt lint/verify and declared native checks;
6. replay affected visual/behavioral/PTY cases against the immutable oracle;
7. obtain independent verifier and reviewer evidence;
8. integrate serially with compare-and-swap and re-run affected evidence.

Final acceptance requires all of the following:

- every valid reconciled task independently verified and reviewed;
- taskfmt lint/verify and native receipts for every required task;
- locked full-workspace `cargo nextest`, API/static/documentation checks,
  actionlint/Lychee/path checks, and qualified performance/allocation budgets;
- native macOS and required native Linux checks;
- real application/component behavior through PTY and lifecycle settled
  transitions, including resize, input, exit, and cleanup;
- exact equality for all 7,550 keys and 30,200 artifacts across ANSI/plain/
  PNG/HTML, five sizes, five color modes, fixtures, applications, routes,
  interactions, and deterministic reruns;
- negative controls reject altered visual and behavioral outputs;
- no duplicate renderer, compatibility painter, snapshot copier, unexplained
  output drift, or protected-ref mutation;
- final integration tree is clean, ancestry is correct, and all receipts bind
  the exact final tree and dependencies;
- independent final verifier and reviewer return `VERIFIED`.

## Completion handoff

The goal ends with a merge-readiness report for `main`, not a merge. Report the
exact branch/commit/tree, ancestry, task dispositions, tool/oracle hashes,
complete matrix counts, behavioral/PTY results, performance/API/static/doc
results, independent verdicts, and protected-ref checks. If any required item
is missing or unknown, return NO-GO and keep the ledger disarmed.

Do not treat this prompt as authorization. Its startup gate must reject the
current state until the readiness report changes through fresh evidence,
explicit independent verification/review, and a separate authorization step.
