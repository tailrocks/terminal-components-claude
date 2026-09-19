# Repository agent policy

This file is the first-read operating contract for the entire terminal-components
refactoring campaign. Follow it together with the current readiness report and
the current task contracts. Do not infer campaign authority from stale goal
files, handoffs, continuation prompts, historical reports, or comments.

## Non-negotiable repository rules

- Preserve correctness, consistency, product behavior, and architectural intent.
- All Rust test and Rust test-validation commands use `cargo nextest`; never
  use `cargo test`. Use the repository's qualified native build/doc/static
  commands where a test runner is not applicable.
- Refactoring execution is native macOS plus isolated subagents only. Never use
  Docker, Podman, containers, images, mounts, root firmlinks, or a container
  runtime. Never recreate the old `/task`, `/work`, `/proof`, or `/run`
  container namespaces.
- Grok Build delegates the campaign to isolated host-local subagents. Only
  those subagents implement, test, verify, and review; the coordinator
  schedules the DAG and integrates reviewed commits serially. The coordinator
  does not replace task-owned implementation with coordinator edits.
- `CLAUDE.md` must remain a symlink to this directory's `AGENTS.md`. Never
  write a separate `CLAUDE.md` body.
- Never move, delete, retarget, recreate, or force-push the `visual-baseline`
  tag or its GitHub release. Never write to the baseline branch, tag, release,
  snapshots, oracle store, or expected artifacts.
- Always use the latest versions of toolchains, crates, actions, and CLIs
  **except** frozen oracle pins (`visual-baseline` tag/store, tuisnap rev
  `2d43458…` for the visual suite, and other campaign-frozen hashes). Latest
  means regenerate pins; never float `@latest` in YAML.
- GitHub Actions workflows are generated only by `velnor-workflow`
  (https://github.com/tailrocks/velnor). Never hand-edit `.github/workflows/**`.
  Change `.github-gen/velnor-workflow.toml` (or the generator, after
  multi-agent vision alignment) then regenerate. `.github/workflows/AGENTS.md`
  is generated: regenerate; do not hand-edit.
- Keep `cargo nextest`; never `cargo test`.
- Prefer `rtk` for shell commands. Every commit uses `git commit -s` and
  includes `Co-authored-by: Codex <codex@openai.com>`.

## Current campaign authority

The sole current readiness authority is
[`docs/refactoring-plan/execution-readiness-report.md`](docs/refactoring-plan/execution-readiness-report.md).
It is a preparation gate, not an execution prompt. Its current verdict is
**NO-GO** until every listed blocker is resolved with current evidence.
While it is **NO-GO**, do not arm `/goal` or dispatch production refactoring
tasks. Non-authorizing preflight, catalog, taskfmt, or proof checks may gather
evidence for resolving blockers, but they never authorize execution or make a
task accepted.

The current campaign contracts are:

- [`docs/refactoring-plan/README.md`](docs/refactoring-plan/README.md) for
  navigation and authority;
- [`docs/refactoring-plan/campaign-policy.md`](docs/refactoring-plan/campaign-policy.md)
  for branch, scope, and protection rules;
- [`docs/refactoring-plan/subagent-only-policy.md`](docs/refactoring-plan/subagent-only-policy.md)
  and [`docs/refactoring-plan/campaign-executor-protocol.md`](docs/refactoring-plan/campaign-executor-protocol.md)
  for execution and isolation;
- [`docs/refactoring-plan/path-contract.md`](docs/refactoring-plan/path-contract.md)
  and [`docs/refactoring-plan/proof-contract.md`](docs/refactoring-plan/proof-contract.md)
  for native paths, proof inputs, and receipts;
- [`docs/refactoring-plan/campaign-ledger.schema.json`](docs/refactoring-plan/campaign-ledger.schema.json)
  for the campaign ledger shape;
- [`docs/refactoring-plan/task-format.md`](docs/refactoring-plan/task-format.md)
  for the current taskfmt identity and command surface;
- [`docs/refactoring-plan/task-graph.json`](docs/refactoring-plan/task-graph.json)
  is generated structural dependency data only. It contains no task status,
  acceptance result, or execution authorization and cannot override a NO-GO.
- [`refactoring-tasks/visual-validation.md`](refactoring-tasks/visual-validation.md)
  for the visual comparison contract.

Root [`GOAL.md`](GOAL.md), [`COMPONENT_ARCHITECTURE.md`](COMPONENT_ARCHITECTURE.md),
and [`DESIGN.md`](DESIGN.md) are active product/architecture references only;
the current report and contracts above govern campaign execution. Retired
goals, coordination files, handoffs, state dumps, continuation prompts,
superseded plans, and old reports are removed from this branch; Git history is
their recovery source. Remaining historical evidence is retained only when a
current contract or machine ledger explicitly binds it. A historical record
cannot authorize execution or override current source, tests, task contracts,
deterministic proof, or the frozen visual oracle.

The current NO-GO facts are binding: the frozen suite/store is absent from the
active branch gate; no trusted native per-check context/result/observer
materializer or accepted verifier-subagent evidence exists; the ledger is
disarmed; consumer migration and ownership cleanup are incomplete; and full
behavioral, visual, and performance parity is unproven. GitHub CI/performance
runs are supplementary and cannot substitute for the required native macOS
verification.

## Frozen visual baseline: final product oracle

The canonical product oracle is the frozen
[`visual-baseline/snapshots`](https://github.com/donbeave/terminal-components-claude/tree/visual-baseline/snapshots)
corpus. Treat the `visual-baseline` branch, tag, release, snapshot store, and
expected artifacts as read-only policy-protected inputs. Provider immutability
is not assumed; repository policy is the guard. The URL is a navigation
reference. The proof oracle identity is the peeled `visual-baseline` tag commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; verify the tag-derived commit
before importing any oracle input, and fail closed on a mismatch.

The refactoring is considered visually correct only when the final
implementation reproduces the frozen `visual-baseline` snapshots **1:1** for
every supported fixture, application, viewport/terminal size, color mode, and
other captured variant. This is exact parity with the existing product, not an
approximate, aesthetically equivalent, or “improved” result.

The frozen corpus currently contains exactly 7,550 matrix keys and 30,200
artifacts: four artifacts per key (ANSI, plain text, PNG, and HTML), five
terminal sizes (72x20, 80x24, 100x30, 120x40, and 160x50), and five color modes
(truecolor, 256-color, 16-color, `none`, and `nocolor`). The tag-derived suite,
including `tests/visual_baseline/` and its matching `.config/nextest.toml`, must
enumerate every key and compare every artifact. A smaller “supported,”
“relevant,” or “applicable” subset is not an acceptance gate.

The architecture may change substantially. The observable product represented
by the baseline may not. The target is the same rendered and interactive result
with the improved internal architecture.

### Mandatory snapshot verification

After any implementation change that can affect rendering or behavior, compare
the refactored implementation with the affected frozen baseline surface as
early as practical. At larger milestones, expand from the affected keys to the
full 7,550-key matrix. The final gate must cover all 30,200 frozen artifacts.

For every baseline-covered case:

```text
refactored implementation output == visual-baseline snapshot output
```

Use deterministic repository tooling and proof contracts; do not reduce the
gate to manually looking at screenshots. Verification must cover every
representation and input dimension in the complete frozen corpus:

- ANSI and plain-text terminal output;
- canonical cells, graphemes, continuation cells, styles, cursor, and
  dimensions;
- PNG and HTML render artifacts through the qualified deterministic compare;
- all five terminal sizes and all five recorded color modes;
- every application, fixture, route, state, interaction checkpoint, and
  captured variant in the 7,550-key store;
- the full PTY input/output, terminal setup/cleanup, resize, and settled-frame
  transition suite represented by the frozen capture configuration.

Missing oracle data, missing artifacts, an incomplete matrix, an un-replayed
settled transition, or an unknown comparison result is a failed gate, not
permission to skip the case.
No active visual acceptance run is valid until a verifier imports the exact
read-only suite, configuration, and grouped oracle store from the pinned tag
commit. Candidate-generated digests, self-baselines, or static headless frames
are not comparisons against the product oracle and do not prove behavior.

### Baseline immutability

Never change the expected baseline merely to make a refactoring pass. Forbidden
approaches include:

- modifying baseline snapshots or expected artifacts to match new output;
- automatically blessing new output or running an update/acceptance command
  because snapshots differ;
- deleting failing cases or reducing the size/color/state matrix;
- weakening comparison rules, thresholds, fields, or provenance checks;
- changing fixtures, timing, inputs, or environment to hide a regression;
- replacing missing expected output with candidate-generated output.

A mismatch means either the refactored implementation is wrong or the
verification harness is wrong. Investigate both possibilities. The baseline
remains the reference unless repository evidence proves a particular
historical snapshot invalid and that conclusion is independently verified and
explicitly documented. Never silently redefine the oracle.

## 1:1 parity includes behavior and architecture

Snapshot parity is the final visual gate, not a loophole. The implementation
must preserve actual component and application behavior. It must not hardcode
historical pixels, bypass reusable components, or repaint a reference frame
after an incorrect component has rendered.

If a reusable component owns a visible region, that component must own its
rendering, state, hit testing, and behavior. A compatibility layer that paints
over output solely to reproduce a snapshot is not an acceptable final
architecture. No hidden duplicate renderer, app-local painter, copied oracle
frame, or compatibility hack may mask an incorrect refactor.

Final acceptance therefore requires both:

1. visual parity: all required output matches the frozen snapshots 1:1;
2. behavioral and architectural correctness: component ownership, public APIs,
   boundaries, keyboard input, focus, hover, scrolling, selection, layout,
   state transitions, application flows, PTY behavior, and lifecycle semantics
   remain correct and are independently evidenced.

## Authoritative execution workflow

Use this loop for every task and integration milestone:

```text
understand task
→ inspect current source, frozen baseline, task contract, and relevant history
→ implement in an isolated subagent worktree
→ run targeted cargo nextest and static checks
→ run latest standalone taskfmt validation/verification
→ run native behavioral and visual proof
→ compare every affected result with the frozen baseline
→ obtain independent verifier/reviewer evidence
→ integrate the reviewed commit serially
```

Do not postpone parity checking until the end. Every rendering or behavior
change gets the narrowest affected baseline replay first, followed by broader
matrices at integration milestones. Before the campaign is declared complete,
run the complete frozen matrix and full settled-transition replay.

Before dispatch, the coordinator must pass the readiness/dependency preflight,
confirm accepted dependency receipts and a clean candidate worktree, and bind
the expected parent commit with compare-and-swap integration semantics. A
verifier receives the implementer's committed candidate in a frozen read-only
view plus a separate external `RUN_DIR`; it must not mutate the candidate.

Each task uses isolated host-local roles:

- implementer subagent: scoped code change and focused tests;
- verifier subagent: clean committed candidate worktree frozen read-only to the
  verifier, separate external run directory, native proof preparation, taskfmt
  lint/verify, and raw evidence;
- independent reviewer subagent: scope, provenance, behavior, visual results,
  architecture ownership, and forbidden-mutation review;
- serial coordinator: DAG scheduling, compare-and-swap integration, and final
  campaign gates.

Task-owned implementation, focused tests, and per-task verification are
subagent responsibilities. The coordinator may run only branch-level
integration and final gates after accepted subagent evidence; coordinator
checks never substitute for task-owned verifier or reviewer evidence.

No two roles share writable worktrees, build directories, snapshot stores, or
run directories. Candidate outputs remain untrusted in the external run
directory until the verifier records complete evidence and the reviewer
returns `VERIFIED`; `REJECTED` or `BLOCKED` evidence cannot be integrated.
Integration is compare-and-swap: the coordinator must confirm the candidate's
expected parent is still the branch tip and must reject stale ancestry or
missing dependency receipts.

## taskfmt role and native verification

Grok Build delegates the actual work to subagents. **taskfmt is used only for
deterministic per-task validation and verification.** It must not orchestrate
agents, create/reset campaign workspaces, manage refs, promote tasks, start
Docker, start containers, or provide integration authority.

The campaign execution path is:

```text
Grok Build → subagents → implementation → taskfmt deterministic verification
→ native visual/behavioral gates → serial integration
```

Use only the latest standalone taskfmt qualified from:

```text
/Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
version: 0.2.0
binary SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

If that checkout revision, version, or executable hash changes, stop and
requalify the new latest taskfmt before using it. Older pins are invalid.

The only permitted taskfmt commands are the standalone per-task gates:

```sh
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify \
  --root "$WORKTREE" \
  --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" \
  --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

Never use taskfmt `init`, `status`, `run`, `start`, `monitor`, `host`,
`runtime`, `dispatch`, `promote`, or any other lifecycle/orchestration command.
Taskfmt does not create containers and is not a substitute for native proof,
visual comparison, behavioral verification, review, or integration.
The current 73/73 package `lint` result is only catalog-shape evidence; it does
not claim task `verify` success or task acceptance. The readiness report records
that no accepted current verifier-subagent evidence exists.

Verification must work natively on macOS. Before taskfmt, the verifier owns the
native proof setup: build the qualified comparator with the repository helper,
materialize an exact immutable per-check context set and context index in the
external run directory, bind task/check/run/source/oracle identities, provide
observer and result capabilities, and record commit/path/hash receipts.
Taskfmt then validates the individual task contract; it does not invent or
bind proof provenance that the native launcher failed to provide.

## Source-of-truth hierarchy

Resolve disagreements in this order:

1. repository safety rules in this file and current user instructions;
2. frozen `visual-baseline/snapshots` for expected visual product output;
3. current source, tests, task manifests, deterministic proof, verified
   architecture requirements, and current Git history for implementation
   correctness;
4. the current readiness report and current campaign contracts for execution
   status and process;
5. historical goals, handoffs, continuation prompts, reports, comments, and
   old tool pins as provenance only.

When documentation conflicts with deterministic evidence, investigate the
discrepancy and update the documentation. Never weaken verification to preserve
a stale claim. A task's local acceptance criteria cannot override the
repository-wide final parity gate.

## Documentation and link integrity

Every tracked Markdown file is part of the documentation surface, including
root instructions, `CLAUDE.md`, `docs/**`, task READMEs/contracts, examples,
architecture records, reports, and historical files that remain referenced.
The repository-wide link gate is [`lychee.toml`](lychee.toml), qualified against
the latest stable Lychee release. CI enumerates the tracked inputs with Git and
runs the pinned `lycheeverse/lychee-action` job in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml). The current qualification
is Lychee `0.24.2` and action `v2.9.0`; re-qualify both together when either
version changes.
All external GitHub Actions in the CI and performance workflows are pinned to
full commit SHAs. An action update requires verifying the upstream release or
ref, updating its version comment, and re-running workflow syntax validation.

The gate checks local relative/root-relative paths, Markdown files and
directories, fragments, verbatim references, repository GitHub URLs, and
external HTTP/HTTPS links. `root-dir` is the checkout root, fragment checking
is `full`, hidden and ignored tracked inputs are included, and no broad URL or
path allowlist is permitted. The cache only reduces repeated external requests;
it never turns a local or fragment mismatch into a pass. Retries and per-host
throttling bound transient network pressure without accepting error statuses.
Mail-looking values in task fixtures are data, not links; there are no active
Markdown `mailto:` destinations to validate.

Run the same native local check from the repository root:

```sh
INPUTS="$(mktemp)"
git ls-files -z | tr '\0' '\n' | awk 'tolower($0) ~ /\.(md|mkd|mdx|mdown|mdwn|mkdn|mkdown|markdown|mdc)$/ { print }' > "$INPUTS"
mise exec lychee@0.24.2 -- lychee --config lychee.toml --root-dir "$PWD" --files-from "$INPUTS"
rm "$INPUTS"
```

A broken internal reference requires semantic investigation, not mechanical
replacement. Check current source, Git history, renames, deletions,
predecessor/successor documents, and the authority hierarchy. Update the
surrounding claim and link to the authoritative current destination; restore a
deleted document only when evidence shows it is still required, then modernize
it fully. Never create placeholders, delete useful links, convert links to
plain text, bless new output, or add a broad exclusion to make Lychee green.
Lychee does not detect every bare path claim, so review explicit repository
paths in prose and code blocks as well. Historical paths may remain only when
their historical scope is explicit and they do not present themselves as live
destinations.

## Definition of refactoring complete

Do not declare the refactoring complete, ready to merge, or product-correct
until all of these are true:

- every valid refactoring task is implemented and independently reviewed;
- latest taskfmt lint/verify passes for every required task;
- full locked, unfiltered workspace `cargo nextest` and required
  static/API/documentation checks pass;
- behavioral verification passes through real component and application paths;
- all application integrations and the full PTY/lifecycle settled-transition
  suite pass;
- every declared performance requirement and existing performance baseline for
  the campaign passes;
- no compatibility hack, duplicate renderer, or painter remains solely to fake
  visual parity;
- explicit duplicate-painter/ownership scans pass and no generated baseline or
  performance files changed unexpectedly;
- the complete frozen PTY visual suite runs and grouped-store integrity passes;
- all 7,550 frozen-baseline cases and all 30,200 artifacts have been compared;
- there are no unexplained snapshot, semantic, behavioral, or provenance
  mismatches;
- final rendered output is **1:1 with `visual-baseline/snapshots`**;
- the final integration and proof trees are clean, ancestry is correct, and
  the final tree is bound to accepted dependency and verification receipts;
- an independent verifier/reviewer confirms the complete results and the final
  integration tree;
- the frozen baseline and all protected refs/stores remain unchanged.

The final principle is:

> **Refactor the internals, not the product output.**
>
> The architecture may change substantially, but everything represented by the
> frozen `visual-baseline` must continue to render and behave exactly as before.
>
> **No unexplained visual drift is acceptable.**
