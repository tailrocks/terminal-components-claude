# Qualification and DAG dispatch authorized

This is the generated next implementation `/goal`. It does not arm the
ledger. Current readiness is **GO** for qualification and DAG dispatch per
[`execution-readiness-report.md`](execution-readiness-report.md). Dispatch
`TASK-071` then `TASK-072`, then the remaining valid DAG. Do not dispatch
retired `TASK-001`/`TASK-070`. Ledger stays `armed = false` until a later
explicit arming step. Do not wait for full 7,550-key tag recapture exit 0.

This handoff is generated from the assessed pre-documentation source payload
`4118c4f550637f8cb764b9eabb6d31dc3dd649cd` / tree
`2f939761d675cedda5a04ed0a54c41eac0ee4dd8` / parent
`a22abb2f000c96cabeaac6ec395bcba32b6b137e`. The subsequent documentation
repair changes the source tree; this file cannot self-attest that future tree.
The next run must bind its own clean commit/tree and invalidate every affected
receipt, result, context, review, and calibration run after any relevant edit.

## Objective

On `refactor/holla-parity`, complete all valid reconciled refactoring work,
preserve original behavior, and reproduce the immutable product oracle exactly:

```text
complete intended refactoring + correct original behavior + exact original visual output
```

Stop at merge readiness. Do not merge into `main` without separate permission.

## Mandatory startup gate

Before any implementation agent is spawned, the coordinator must independently:

1. Read `AGENTS.md`, the current readiness report, all campaign contracts,
   every task package and schema, the architecture documents, and the visual
   validation contract.
2. Re-read Git refs. Bind the actual branch, local/remote `main`, merge-base,
   scope base, clean worktree, final source commit/tree/parent, and dependency
   ancestry. The preparation payload before this generated prompt was
   `4118c4f550637f8cb764b9eabb6d31dc3dd649cd` / tree
   `2f939761d675cedda5a04ed0a54c41eac0ee4dd8`, parent
   `a22abb2f000c96cabeaac6ec395bcba32b6b137e`; local and remote `main` are
   `7b27732a8c3c131760ec3438f641cb3c11343a42`, and the assessed remote
   campaign tip equals `4118c4f5`. The documentation rebinding commit and any
   later edit invalidate that identity. Never trust this embedded value
   without fresh Git reads.
3. Verify the protected oracle exactly:

   ```text
   refs/tags/visual-baseline^{commit}: 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
   tag tree:                         0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
   snapshots tree:                   3f0261c32849e26feda24d87697de4a7ce6b8375
   matrix:                           7,550 keys / 30,200 artifacts
   ```

   Import only the read-only oracle at
   `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`.
   Its manifest is
   `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`
   with SHA-256
   `95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
4. Verify the qualified taskfmt, requalifying on any change:

   ```text
   source: /Users/donbeave/Projects/taskfmt/task-format
   revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
   source tree: b7d90bd8adbe6c341a08fc485099ee8cf1584431
   version: 0.2.0
   binary: /Users/donbeave/.cargo/bin/taskfmt
   SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
   ```

   Use only standalone `taskfmt lint` and `taskfmt verify`; never taskfmt
   lifecycle/orchestration commands. Never use Docker, Podman, containers,
   images, mounts, firmlinks, namespaces, or `cargo test`.
5. Verify the graph exactly: 73 direct packages, 77 recursive contracts, 506
   direct/526 recursive checks, 276 edges, depth 35, 193 conflict pairs, 0
   serialization pairs, no cycles/dangling IDs. `TASK-001`/`TASK-070` remain
   retired fail-closed. `TASK-071`/`TASK-072` are the first dispatchable
   qualification tasks. Valid implementation work is `TASK-002`–`TASK-069`
   and `TASK-073` after accepted 071/072 receipts unless the current
   reviewed graph says otherwise.
6. Materialize contexts/results/logs/census outputs only in external
   verifier-owned native run directories. Validate every
   source/tree/tool/oracle binding, observer nonce/FD, result schema/hash,
   dependency receipt, and read-only trust input. The worker cannot
   authorize its own success. Do not invent ledger receipts. Per-task
   verifier/reviewer evidence remains mandatory; a campaign-wide final-tree
   seal is not a startup prerequisite.
7. Independent verifier and reviewer `VERIFIED` evidence is required for
   each dispatched task and again for the exact final tree. Predetermined
   final-seal paths remain:

   ```text
   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-verifier-report.md
   /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/final-seal-2026-09-19/final-reviewer-report.md
   ```

8. Continue the full 7,550-key tag recapture as a **census**, not a
   dispatch timer. Do not wait for tag exit 0. `form_advanced` Class A/B
   are candidate/product obligations; snapshots remain the oracle. Do not
   normalize, bless, mask, skip, or replace oracle artifacts. Halt only on
   an unexplained Class C mismatch.
9. Native macOS evidence is required. Linux remains unavailable /
   unverifiable as a separate platform item; it does not hold this GO.
10. This prompt never arms the ledger. Keep `armed = false` until a later
    explicit arming step. Qualification and DAG dispatch are already
    authorized by the readiness report.

If oracle, taskfmt, graph, or Git identity checks fail, exit nonzero and
do not spawn implementers. Do not stop on the circular tag-exit-0 demand.

## Execution plan after authorization

Use bounded subagents, each explicitly configured as `gpt-5.6-luna` with `max`
reasoning. Assign disjoint file ownership and require each agent to return
exact source identity, commands, exit status, raw evidence path, and unresolved
risks. The coordinator schedules only the validated DAG and integrates reviewed
commits serially with compare-and-swap parent checks.

Waves:

1. Revalidate preparation, then qualify/bootstrap prerequisites (`TASK-071`,
   `TASK-072`) without accepting retired lifecycle tasks.
2. Implement foundational API/state/ownership obligations in graph order.
3. Implement reusable components and runtime routing/focus/layer/pointer
   boundaries; independently verify each affected visual/behavioral surface.
4. Migrate Holla, Showcase, Jackin, and TablePro consumers and remove every
   duplicate renderer, compatibility painter, fixed-grid workaround, copied
   oracle frame, and application-specific substitute for reusable components.
   Complete the migration with breaking changes where required: remove every
   legacy path and implementation, duplicate renderer, compatibility shim,
   alias, and deprecation period. No compatibility bridge may remain hidden
   behind old names, forwarding APIs, fallback painters, or feature flags.
   Fix the architectural condition that permits each legacy path; a symptom
   patch requires an explicit root-cause deferral and independent review.
5. Execute affected parity replay after every integration checkpoint; rerun the
   full 7,550-key matrix at major milestones.
6. Run the complete final product gates and obtain independent final
   verifier/reviewer evidence. Publish no baseline updates.

Every task must satisfy its own contract only after prerequisites are accepted;
no task may require its future result to start, and no final-product result may
be required before the first valid implementation task.

## Required final acceptance

The implementation is not complete until all valid tasks are independently
verified and integrated, taskfmt lint/verify passes, the full locked workspace
uses `cargo nextest`, APIs/static/docs/actionlint/Lychee checks pass, platform
evidence is genuine, performance/allocation budgets pass, and all application
and PTY/lifecycle behavior is covered. The exact gate must compare every
7,550 key and all 30,200 ANSI/plain/PNG/HTML artifacts against the protected
oracle with no unexplained mismatch. Focus, hover, keyboard, shortcuts,
selection, scrolling, mouse, resize, loading/empty/error/disabled,
overlays/modals, tables/forms/lists/panels/navigation, persistence, async
updates, startup, terminal restoration, and shutdown must follow real component
and application paths.

Architecture acceptance additionally requires caller-owned state, borrowed
props, read-only drawing, runtime-owned routing/focus/layers/pointer/cursor,
correct public API migration, component ownership, reuse, and no hidden
compatibility implementation. It must also prove that no legacy path,
duplicate renderer, compatibility painter, shim, alias, forwarding facade, or
deprecation window remains anywhere in the migrated scope. Breaking API changes
are acceptable and preferred over retaining compatibility. Product output must
be 1:1 with the frozen oracle, not merely visually similar.

## Integration, invalidation, and handoff

Each receipt binds the exact tested commit/tree/parent, scope base, task and
graph hashes, tool/comparator/oracle identity, environment, run/check IDs,
dependency ancestry, outputs, observer evidence, and independent decisions.
Any relevant source, docs, task contract, schema, script, generated workflow,
tool, comparator, oracle, environment, or generated-output change invalidates
affected evidence and requires requalification. A parallel-agent result never
survives integration automatically. A tool revision, executable hash, oracle
identity, task-contract hash, or final source/tree change invalidates the
bound receipt; no ancestor or sibling receipt may authorize the changed tree.

At the end, produce merge-readiness evidence for `main`: clean final tree,
correct ancestry, complete parity/behavior/architecture/performance/static/
platform evidence, protected baseline unchanged, and independent final
acceptance. Do not merge or push without the separately authorized workflow.
