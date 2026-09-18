# Next implementation goal — NOT AUTHORIZED FOR EXECUTION

This file is the complete future `/goal` prompt. It is a plan only. The
current readiness verdict is **NO-GO**. Do not arm `.campaign/ledger.json`,
dispatch a task, create task worktrees, or start product refactoring until a
fresh readiness check returns **GO**.

## Current preparation binding

The source state reviewed by this preparation update is branch
`refactor/holla-parity`, HEAD
`19f6d2ebd7ecc839e932b92ed76a592c2bee483c`, tree
`c83a213b8e8b0eb8d262578f9a2adb2d9bd989ba`. A subsequent documentation commit
invalidates source-bound receipts; rebind them to the final clean tree before
any readiness decision.

The immutable oracle is the peeled tag commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, snapshot tree
`3f0261c32849e26feda24d87697de4a7ce6b8375`, with `7,550` keys and `30,200`
ANSI/plain/PNG/HTML artifacts. Qualified taskfmt is source revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, version `0.2.0`, executable
SHA-256
`f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`.
Proof repairs are in `8c9e050c`, `f4ce758e`, `4737da3c`, `7639e7ae`, and
`19f6d2eb`. The HEAD repair's positive native-launch check remains
**UNVERIFIED/PENDING**; no independent verifier has run or accepted it.
Historical fresh verifier evidence at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-4737da3c`
is **REJECTED** for candidate `4737da3c`; no current independent reviewer
acceptance exists for the `19f6d2eb` proof chain.
Calibration evidence at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-frozen-connections`
has targeted `3/3` passing, but full `302` selected with `301` passed,
`1` failed, and `2` skipped. The failure is
`tablepro_connections_form_advanced_120x40_truecolor`, with `76/100`
artifacts mismatching. The ledger remains `armed: false`.

## Objective

Prepare and then complete the `refactor/holla-parity` campaign without changing
the frozen product oracle. Refactor internals and ownership; preserve all
rendered, interactive, PTY, lifecycle, performance, and API behavior. Never
modify `main`, the `visual-baseline` tag/branch/release/store, expected
artifacts, or the frozen snapshots.

## Mandatory startup gate

1. Read `AGENTS.md`, this goal, the current readiness report, and all current
   task contracts. Run the repository preflight from a clean macOS worktree.
2. Stop immediately unless the report is **GO**, the ledger is explicitly
   armed by the authorized workflow, the branch and parent are correct, and
   all dependency receipts are current. **NO-GO never authorizes execution.**
3. Rebind every receipt to the live clean HEAD and tree. Reject stale, missing,
   self-attested, candidate-generated, or unknown evidence.
4. Verify the peeled oracle tag is exactly
   `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
5. Verify standalone taskfmt is exactly version `0.2.0`, revision
   `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, executable SHA-256
   `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`.
6. Use native macOS only. No Docker, Podman, containers, mounts, firmlinks,
   old `/task`, `/work`, `/proof`, or `/run` namespaces. Rust tests use
   `cargo nextest`; never use `cargo test`.

## Preparation blockers to resolve first

Repair and independently qualify the rejected native proof path before any
production task:

- materialize one immutable context and exact context index per check;
- bind task, check, run, source HEAD/tree, oracle tag/manifest, report, and
  taskfmt identities end-to-end;
- make the compare context/result ABI identical across preparation, launcher,
  comparator, and task checks;
- supervise the observer, bind its socket/FD and nonce/request sequence, and
  reject alternate or stale transports;
- make result paths verifier-owned capabilities with symlink/hardlink,
  outside-root, stale, missing, duplicate, and side-effect rejection;
- wire native preparation and post-run validation into the dispatcher so
  taskfmt cannot bypass proof; and
- record independently reproducible positive and negative evidence, including
  rejection of every malformed, stale, mismatched, or mutated input.

The three proof-fix commits do not close these blockers: current verifier
evidence records a failed format check and a positive bundled-worker result
rejected for `CONTEXT_INDEX` with no observer event. The full calibration also
remains failed. Do not reinterpret either result as product work or as a
reason to weaken the gate.

## Exact commands still required before authorization

Run from the final clean campaign worktree, with external verifier-owned
`RUN_DIR` and target paths:

```sh
scripts/campaign-preflight.sh
python3 docs/refactoring-plan/evidence/validate-plan.py --summary
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
scripts/campaign-build-proof.sh
cargo nextest run --locked --workspace --no-fail-fast
```

The verifier must also run the complete native proof positive/adversarial
matrix, then the tag-derived calibration and final visual gate. A failed,
unknown, stale, or unreviewed result stops authorization. Do not run a
taskfmt lifecycle command, `cargo test`, a baseline acceptance command, or a
reduced final matrix.

Import the exact frozen suite, checked-in configuration, and grouped store from
the read-only tag. If the installed nextest rejects the tag's binary override,
use only a separately hashed external compatibility config; never edit the
tag. Re-run all 7,550 keys and 30,200 artifacts: ANSI, plain text, PNG, and
HTML across five sizes and five color modes, including PTY setup, cleanup,
resize, input, and settled-frame transitions. The latest targeted Holla case
passed; the remaining calibration failure is structural TablePro drift. The
historical split is explicit: `3570a2ed23444dddf1eddcdcc49b654b169038fe`
supplies responsive form ownership, while
`89218626011f2f82c4e87c4dfd5868a4c5f3e284` supplies the later responsive
rendering source; frozen expected output requires list+form for `form_new` and
full-pane form ownership for `form_advanced`. Resolve that source question
without changing the oracle. Never bless, rewrite, delete, filter, or weaken
an expected artifact.

## Roles and execution loop

Use isolated host-local subagents, all `gpt-5.6-luna` with `max` reasoning:

- coordinator: readiness, immutable refs, DAG, compare-and-swap integration,
  and final branch gates only;
- implementer: scoped code and focused tests in one isolated worktree;
- verifier: frozen committed candidate, separate external run directory, native
  proof materialization, latest taskfmt `lint`/`verify`, behavior, and affected
  oracle replay;
- independent reviewer: scope, ancestry, provenance, ownership, forbidden
  mutations, behavior, visual evidence, and architecture.

Do not share writable worktrees, build directories, run directories, stores, or
snapshots. For every task: inspect → implement → commit with DCO signoff and
`Co-authored-by: Codex <codex@openai.com>` → verify → review → integrate
serially. Use only taskfmt `lint` and `verify`; taskfmt never orchestrates,
creates workspaces, promotes refs, or replaces native proof.

The current catalog has 73 numbered packages, 506 checks, 276 dependency edges,
and maximum depth 35. Respect the generated DAG: proof foundation
001→070→071→072, oracle/identity 002–008→073, shared layers 009–031,
Showcase 032–039, Holla 040–050, Jackin 051–057, TablePro 058–064, then
closure 065–069. Parallel work is allowed only for independent DAG nodes in
disjoint worktrees; integration is always serial and compare-and-swap.

## Acceptance gates

Do not call any task, wave, or campaign complete until its verifier and
reviewer return accepted evidence bound to the exact integrated tree. Final
acceptance requires all taskfmt gates, locked full-workspace `cargo nextest`,
static/API/documentation checks, real component/application behavior, PTY and
lifecycle transitions, strict performance baselines, ownership and
duplicate-painter scans, and clean ancestry/provenance receipts.

The final visual gate must compare every frozen key and all 30,200 artifacts
1:1. No unexplained semantic, behavioral, visual, performance, or provenance
mismatch is acceptable. Compatibility painters, hidden duplicate renderers,
hardcoded oracle frames, and app-local repaint overlays are failures even if a
snapshot matches.

## Stop conditions

Stop, keep the ledger disarmed, and report **NO-GO** on any failed or unknown
check; stale parent; dirty candidate; missing receipt; rejected reviewer;
observer/result side effect; ABI or nonce mismatch; oracle/config/store drift;
taskfmt identity change; protected-ref mutation; baseline mismatch; container
use; taskfmt lifecycle invocation; or unsupported attempt to reduce coverage.
Never manufacture a receipt, bless candidate output, weaken a comparator, or
continue past a rejected gate.

Only after every preparation blocker, task, dependency, independent review,
behavioral gate, performance gate, and complete frozen visual comparison is
accepted may the readiness report change to **GO** and the campaign become
merge-ready.
