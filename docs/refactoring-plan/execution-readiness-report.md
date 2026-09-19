# Execution readiness report

**Verdict: NO-GO.**

This is the sole current readiness authority for `refactor/holla-parity`.
It is a preparation gate, not an implementation prompt. The campaign ledger
must remain `armed: false`; no production task may be dispatched while this
verdict is NO-GO.

## 1. Bound source and Git truth

The documentation repair started from the freshly inspected checkout below.
The resulting documentation commit changes the tree; therefore the identities
below are the exact source payload against which the repair was authored, not a
self-attested post-commit receipt.

| field | value |
| --- | --- |
| branch | `refactor/holla-parity` |
| HEAD at repair start | `bb574d84bf25ff9179b42e951fca52068f2ab623` |
| tree at repair start | `6e464830897f7c39014b12a79219d6cde8b56549` |
| parent | `8e783592afd0a2c2f08076858a386a091a35e712` |
| local `main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| `origin/main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| remote campaign ref | `f5013f609aed1ba32ce60352b38fd0b1b11b063c` |
| merge-base with `main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| merge-base with baseline | `cc14dd6beae526884aabdf897e309be837b4f504` |
| `main..HEAD` at repair start | 78 commits |
| `origin/refactor/holla-parity..HEAD` at repair start | 34 commits |

Local and remote `main` agree. The campaign ref has not been pushed in this
preparation wave. The directory `.worktrees/main` is not evidence of `main`;
its branch identity was checked from Git refs. No protected ref was moved,
rewritten, pruned, or force-pushed.

The semantic three-way comparison is:

- **Baseline ↔ main:** the frozen baseline is the older single-package
  `junie-tui` product; `main` contains the physical workspace/library/test
  refactor and four applications. This is a large architectural change, not
  proof of behavioral equivalence.
- **Baseline ↔ campaign:** campaign inherits the current refactor branch and
  adds preparation/proof infrastructure. The preparation commits do not
  authorize product output changes. The frozen baseline remains the product
  oracle.
- **Main ↔ campaign:** the preparation commits modify proof code, scripts,
  ledger/preflight/dispatch authority, and documentation. The known product
  ownership/rendering gaps remain; they were not hidden or accepted by this
  preparation lane.

Relevant semantic findings are recorded in `architecture.md`,
`branch-diff-foundation.md`, `branch-diff-holla.md`,
`branch-diff-showcase.md`, and `branch-diff-jackin-tablepro.md`. File-count or
diff similarity is not treated as parity proof.

The independent Bernoulli review (`gpt-5.6-luna`, max) rejected the preceding
tree `21f875318f61c30f55bf0a47b8daa4326d48540a7` for accepting taskfmt final /
parent aliases and Rust verifier symlinked parent trust paths. The taskfmt
defect is repaired by `8e783592afd0a2c2f08076858a386a091a35e712`; the Rust
defect is repaired by `bb574d84bf25ff9179b42e951fca52068f2ab623`. Raw review
evidence is `/tmp/campaign-review-evidence-21f87531.7EkYOO/`. These repairs
close the reported defects, not the final independent readiness review.

## 2. Immutable visual authority

The only protected visual authority is the peeled tag:

| identity | value |
| --- | --- |
| tag peel | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| tag commit tree | `0b1f13431fdfd6060cf9f45a114afa5a99cc6c26` |
| `snapshots/` tree | `3f0261c32849e26feda24d87697de4a7ce6b8375` |
| keys | 7,550 |
| artifacts | 30,200: ANSI, plain text, PNG, HTML, 7,550 each |

The exact read-only import and independent manifest are:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19
```

The import records the tag-derived tree and SHA-256 manifest; its key lists
agree and its symlink inventory is empty. The protected branch, tag, release,
snapshots, grouped store, fixtures, and expected artifacts were not changed.

The source used to identify the complete snapshot-producing corpus is
`89218626011f2f82c4e87c4dfd5868a4c5f3e284`, tree
`6fccf997cd742071ebcff0e0a00e89404ef95ca8`, with snapshot tree
`3f0261c32849e26feda24d87697de4a7ce6b8375`. Its external control source is
under:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19
```

This identity is separate from the artifact tag and from architecture/source
oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` (tree
`efa2b409b77077caf5c639f7f4c6154cbadbce5`). The previously cited
`3570a2ed23444dddf1eddcdcc49b654b169038fe` source is not authoritative: its
tree `77d6a559536d6a1d733b52d3b78b5b49112315cb` contains only 28,580 artifacts
and 7,145 keys.

## 3. Catalog, architecture, and remaining scope

The machine-checked catalog contains:

| measure | result |
| --- | --- |
| task packages | 73 (`001`–`073`) |
| checks | 506: 27 `argv`, 479 `shell` |
| dependency edges | 276 |
| maximum dependency depth | 35 |
| source obligations | 1,174 |
| traceability rows | 3,256 |
| plan/DAG structural errors | 0 |
| taskfmt standalone lints | 73/73 passed in the source-payload run |
| accepted production tasks | 0 |

Disposition is explicit. `TASK-001` and `TASK-070` are retired/non-qualifying
fail-closed lifecycle tasks, not dispatchable work. `TASK-071` and `TASK-072`
remain blocked qualification prerequisites. `TASK-002` through `TASK-069` and
`TASK-073` retain the valid component, application, ownership, behavior,
visual, performance, API, and closure work. The generated graph is structural;
it does not mark any node complete or authorize dispatch.

The target architecture remains caller-owned state, borrowed props, read-only
draw, runtime-owned routing/focus/layers/pointer/cursor, and one reusable
implementation per component family. Current source still exhibits the known
obligations: Showcase compatibility paint-over and fixed grid text, Jackin
historical projections, TablePro legacy painters, reduced Holla route/scenario
coverage, and component-level behavioral risks documented in the branch-diff
reports. No compatibility painter may be retained as a final solution.

## 4. Preparation infrastructure and checks

The preparation commits are:

- `e8c4950928b0ab6cc1268777dbed6f96cb0309ba` — strict proof-code Clippy and
  rustdoc repair; tree `ea5edaaeddb3c5e5d5b5906a00c998d25375d3d4`.
- `f6f94dc995f5b6451800d739174d7f23802a40c3` — shell/preparation guard
  repair; tree `61563f7b48d0fe7b8e5bdddae57072e96f6a6f12`.
- `8e783592afd0a2c2f08076858a386a091a35e712` — taskfmt final/parent
  symlink-alias repair and adversarial shell coverage; tree
  `e2867345caf658d339deaecd7f8fa13d04e7c008`.
- `bb574d84bf25ff9179b42e951fca52068f2ab623` — Rust verifier symlinked-parent
  trust-path repair; tree `6e464830897f7c39014b12a79219d6cde8b56549`.

The native proof changes are preparation infrastructure only. The observed
source-payload checks reported:

- `cargo fmt --all --check`: pass;
- `cargo nextest run --locked -p refactor-proof`: 27/27 pass;
- strict proof-code Clippy and rustdoc: pass in the sealed repair evidence;
- shell syntax, ShellCheck, shfmt, preparation guards, proof-path guards, and
  dispatch-authorization guards: pass after the taskfmt path repair;
- plan/DAG validator: pass; all 73 standalone taskfmt lints: pass.

Current evidence paths:

```text
taskfmt lints (73/73):
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-lints-final-retry-2026-09-19
proof build (exit 0; SHA-256 f85400a16dc131fe0f59bfc90b5ec22cadf97de02833a216ed81385bf70a1b44):
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-preparation-final-2026-09-19
Rust trust-path repair and sealed focused nextest/Clippy/rustdoc:
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/symlink-parent-repair-2026-09-19
taskfmt trust-path repair:
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-path-repair-2026-09-19
```

The TASK-071/TASK-072 preparation attempt fails closed because prerequisite
dependency receipts are absent. It is not an acceptance result. The prior
proof-static result is stale relative to the trust repairs. The corrected
static run has fmt 0, Clippy 0, and rustdoc-correct 0 at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/static-final-2026-09-19`.
Shell evidence before the trust repair is at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/shell-final-current-2026-09-19`;
the repair-root contains the latest shell guard evidence. Documentation checks
are at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/docs-final-current-retry-2026-09-19`
(actionlint 0, Lychee 0).

The current boundary run is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/boundary-final-current-2026-09-19`;
exit 1 is solely the missing `parity/evidence.tsv` parity-contract check and
all other boundary checks pass. The current preflight run is
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/preflight-final-current-2026-09-19`;
exit 1 at the exact NO-GO report gate. The refreshed ignored ledger remains
`armed=false`.

These observations are not an accepted receipt for the post-documentation
tree. They must be re-run by a fresh verifier after the final source payload is
sealed. Native proof is integrity/control evidence, not same-user hostile
process isolation; the supported threat model and limitations remain in
`proof-contract.md`.

The prior full-workspace nextest observation was not a clean gate: `3,359
passed, 1 failed, 6 skipped`; an isolated rerun of the affected architecture
test passed. This is historical evidence, not the current final result. The
sealed native macOS final-tree command bound by its identity file to source
commit `4a95fcdeedb8f7a3a132162e536ca28c2404b823` and tree
`f98f4908506f55026a35ea4e2701b78c240d147c` completed with `3,342 passed` and
exit 0. Durable log:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-sealed-final-2026-09-19/nextest.log`;
identity file: `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-sealed-final-2026-09-19/identity.txt`.
This is raw evidence only; no preparation receipt exists. `xtask boundary`
still fails the parity contract because `parity/evidence.tsv` is absent. The
required full behavioral and visual runs are not complete.

Taskfmt source integration qualification is unavailable: the Docker-only test
path was not run, and its guard fails closed when the explicit integration
opt-in is absent. No Docker, Podman, container, image, mount, or namespace was
used. Native Linux is unavailable in this environment; macOS evidence alone
cannot satisfy a required Linux gate.

## 5. Calibration and parity status

The complete gate must execute the independently materialized `892` source,
not copy expected files and not use `3570` as a substitute. The full 892 run
did execute all 302 cases and produced the complete artifact counts, but it is
not a clean calibration:

```text
source commit: 89218626011f2f82c4e87c4dfd5868a4c5f3e284
source tree:   6fccf997cd742071ebcff0e0a00e89404ef95ca8
snapshot tree: 3f0261c32849e26feda24d87697de4a7ce6b8375
test cases: 302; passed 298; failed 4; skipped 2; leaky 1; exit 100
ANSI/plain/PNG/HTML: 7,550 each; diff files: 17
```

Raw summary:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19/logs/final-direct-matrix-summary-v2-2026-09-19.txt`
(SHA-256 `33034862cfb3d7ae2676bbe01269db5ac0c1af6d3cf6e61896f408c36eb14d3c`).
The failures are:

- Holla `task_input_cancelled`: `nocolor` timing is `0 s` where frozen output
  records `1 s` at `80x24` and `120x40`;
- pointer `tablepro_resize_workbench_grown`;
- TablePro `ack_gate`;
- TablePro `connections/form_advanced`.

No normalization or blessing was applied. This is evidence of complete matrix
execution coverage, not a calibration pass. The available control evidence is
therefore insufficient:

- `baseline-control-892-2026-09-19/logs/source-identity.txt` records the
  correct 892 commit/tree and 30,200-source-artifact inventory. The complete
  run above is retained as failed raw evidence, not an accepted replay.
- `baseline-control-3570-2026-09-19/logs/full-rtk.log` stops at 151/302 with
  baseline-approval failures and is invalid for the complete corpus. Its
  source corpus is only 28,580 artifacts.
- The older mixed/dirty calibration
  `calibration-frozen-connections/full-run-1` is `302 selected, 301 passed,
  1 failed, 2 skipped`, exit `100`; it used an invalid source/target mix and
  is not acceptance evidence.
- The exact-tag control under `calibration-baseline-4a79c0a2` also did not
  establish a clean complete pass. No candidate output has been blessed and
  no mismatch has been normalized away.

The final parity gate must prove exact equality for ANSI, plain text, PNG, and
HTML across every key, terminal size, color mode, application, fixture, route,
interaction checkpoint, PTY setup/resize/settled transition, exit, and cleanup.
It must also prove negative-control rejection of altered visual and behavioral
outputs. Static frames, broad image tolerances, masks, copied snapshots, or
arbitrary sleeps are not substitutes.

## 6. Blocker register

| blocker | classification | closure condition |
| --- | --- | --- |
| final-tree binding | preparation | commit payload, then external verifier/reviewer manifests bind exact post-commit HEAD/tree/parent; rerun affected checks |
| native proof/receipt qualification | preparation | positive and adversarial matrix passes with independent observed termination, outputs, contexts, nonce, hashes, and receipts; the two repaired trust-path defects must still be requalified on the final tree |
| complete 892 control | preparation | clean external run covers all 7,550 keys and 30,200 artifacts, deterministic rerun, and altered-output rejection; current run covers the matrix but has four failures and one leak |
| current task verification | preparation | standalone taskfmt `verify` plus native evidence for every accepted task; lint alone is insufficient |
| `parity/evidence.tsv` / boundary contract | preparation | restore a truthful executable parity evidence path and pass `xtask boundary` |
| full workspace reliability/static/doc gates | preparation | qualified nextest/static/API/documentation/CI checks pass on the final tree |
| Linux | external mandatory evidence | run required native Linux lane; no container substitute exists |
| current product regressions | implementation | reconcile valid tasks, remove duplicate painters, restore behavior and prove parity; do not mark preparation accepted |
| independent final verifier/reviewer | preparation | separate read-only verifier and reviewer each return `VERIFIED` for the same final tree |

Preparation defects are fixed only in preparation code/contracts/docs. Product
obligations remain assigned to the implementation DAG. No condition is closed
by a stale report, an ancestor receipt, a copied artifact, or a documentation
claim.

## 7. Evidence sealing and decision

The non-circular protocol is:

1. Freeze and commit the preparation payload.
2. Build the proof tool in a verifier-owned external target and bind source,
   tree, build inputs, binary hash, taskfmt, comparator, and oracle.
3. Materialize exact contexts/results/observer capabilities outside the
   worktree; execute only standalone taskfmt `lint`/`verify` plus declared
   native checks.
4. Collect independent observations and immutable raw evidence.
5. Have a separate verifier and reviewer inspect the exact same final tree.
6. Record a receipt only after both return `VERIFIED`.

Any relevant source, task contract, schema, command, tool, comparator, oracle,
environment, or documentation change invalidates affected evidence. A receipt
cannot attest to a future commit containing itself. The ignored runtime ledger
must remain schema-valid, current, and `armed=false`; it cannot replace the
external seal.

Rejected historical roots are preserved in the evidence index, including:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/reviewer-final-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-211c29ad-independent
```

They are not receipts for this source. Bernoulli's prior rejection is recorded
above and its two findings are repaired, but no current independent final
verifier or reviewer has returned `VERIFIED`.

**Final decision: NO-GO.** The report must remain NO-GO until every blocker
above has fresh, exact-tree, independently reviewed evidence. This decision
does not declare the product refactoring complete and does not authorize the
next goal.
