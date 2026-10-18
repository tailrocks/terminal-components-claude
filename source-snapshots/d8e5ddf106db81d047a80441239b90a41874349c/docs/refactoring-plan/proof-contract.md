# Trusted proof bootstrap and execution contract

This document specifies the missing project harness and its authority boundaries. It does not claim that the harness or complete oracle bundle already exists. The canonical allocation is [task-index.tsv](task-index.tsv): TASK-001 owns comparator/host qualification; TASK-070 owns execution/expansion/oracle/capture/closure; TASK-071 owns test accounting; TASK-072 owns architecture verification. Earlier B/C/S/H/J/T/X labels refer to their corresponding indexed outcomes, not an umbrella implementation authority.

## Verified foundations

The UI source is `02f5294bfdbf38004cc49130d0aff1d01f31434c`. The architectural starting commit is `7b27732a8c3c131760ec3438f641cb3c11343a42`. They serve different purposes. Never capture an expected application frame from the latter.

The inspected task-format revision is `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, at `/Users/donbeave/Projects/donbeave/task-format`. These existing commands are supported:

```sh
taskfmt --version
taskfmt fingerprint
taskfmt --config /proof/bootstrap/experiment.toml project lint terminal-components --projects-root /absolute/catalog
taskfmt --config /proof/bootstrap/experiment.toml project show terminal-components --projects-root /absolute/catalog --json
taskfmt --config /proof/bootstrap/experiment.toml lint /absolute/catalog/terminal-components/completion/NNN
taskfmt --config /proof/bootstrap/experiment.toml progress-init /absolute/catalog/terminal-components/completion/NNN --out /absolute/run/progress.md
taskfmt --config /proof/bootstrap/experiment.toml verify --root /absolute/verification-checkout --task-dir /absolute/catalog/terminal-components/completion/NNN --base RECORDED_SCOPE_BASE_COMMIT --progress /absolute/run/progress.md --log-dir /absolute/run/taskfmt-logs
```

Paths, `NNN`, and the base above are invocation parameters, not literal fabricated artifact identities. The host supplies their resolved values. It checks the executable hash and compiled fingerprint in addition to the displayed version.

`harness/src/cmds/verify.rs` enables full task-contract enforcement. `gate.rs` checks package lint, scope, forbidden paths/patterns, declared checks and progress. Success requires process exit 0 **and** the final stdout line exactly `DONE`. Run without `--fail-fast`, `--no-progress`, or empty progress. Pass `--base` explicitly: source implementation refuses an absent immutable base even though help text still mentions a default `baseline`.

Standalone verification does not overlay `trusted/`, freeze source, isolate candidate code, establish dependency ancestry, or promote a branch. The host must provide those operations. `cmds/run.rs` installs its own trusted overlay, but its lifecycle clones `main`; `cmds/promote.rs`/`ops/git.rs` push `refs/heads/main`. Do not use taskfmt run, monitor dispatch, or promote for this campaign. The supported standalone gate imposes no branch name. Preserve canonical `task/v5`, `verify/v2`, and `task-meta/v1`; put host evidence outside those schemas.

The inspected repaired tuisnap source is `/tmp/tui-snap-audit.656lHG/repaired`; its final accepted revision is recorded by the verification owner in [tuisnap-review.md](tuisnap-review.md). Existing tool qualification commands are:

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --no-default-features
```

Run each from the qualified tuisnap checkout. PTY tests require a real owned PTY on the selected host. Record target, compiler, dependency lock and source revision; a temporary directory name is never a dependency pin. The final external PR and reviewed revision must be resolved before B-HARNESS accepts that tool.

## Ownership and the bootstrap root

Future hashes are outputs. A declaration that a bundle is trusted cannot make its producer trusted. Use two separate acceptance levels:

1. The planner freezes a bootstrap package containing this protocol, exact qualification fixtures, an independent black-box test driver, tool/source pins and the required-set inputs. An independent reviewer examines this package before B-HARNESS dispatch. Its hash is an initial host configuration input, outside the implementation checkout.
2. B-HARNESS implements the project harness. The already frozen bootstrap driver tests its built executable as an untrusted program. The host reviews the implementation and passing evidence, rebuilds it from the accepted source tree, hashes its executable/dependencies, and records an accepted harness receipt. Later tasks resolve that receipt, never a mutable executable path or a receipt authored by the candidate.

The independent driver must not import the candidate comparator or delegate verdicts to candidate-authored tests. It supplies its own positive and negative inputs and checks process status, full required output and filesystem effects. Candidate unit tests are additional evidence. Never use the harness being implemented as the only gate for B-HARNESS itself. The concrete comparator preparation is [proof-comparator-bootstrap.py](evidence/proof-comparator-bootstrap.py), [72 explicit vectors](evidence/proof-comparator-vectors.json) and its [machine protocol](evidence/proof-comparator-protocol.md). Host qualification has a separate [machine protocol](evidence/host-bootstrap-protocol.md); these two black-box suites are independent of the future harness implementation.

The B-HARNESS owner implements only `tools/refactor-proof/`: a thin host adapter for Git/worktrees/isolation and pinned standalone taskfmt, bundle validation, comparator, test accounting and runner protocol. It may not repair production components or choose expected UX. Do not build another scheduler, monitor or receipt service: host evidence consists of immutable files and an operator-controlled local index. The planner owns `trusted/proof-bootstrap/` in B-HARNESS's external task package. That directory is excluded from B-HARNESS writable scope. Copy the independently reviewed preparation driver, vectors and protocols there before dispatch; their creation is planning infrastructure, not production refactoring.

Application baseline owners implement only their own reference adapters and concrete action expansions. B-COMPONENTS owns reusable component adapters. They consume the accepted common harness and may not change its comparator, bundle parser or scenario-membership rules. B-INVENTORY owns source-qualified test discovery and identity mappings. B-TEST-DISPOSITION owns reviewed replacement assertions and stage membership. Component/application repair tasks consume all these accepted products; they have no authority to modify them.

This is project-specific orchestration over existing tuisnap primitives. Do not add terminal-components recipes or taskfmt lifecycle policy to tuisnap.

## Artifact layout and identities

The trusted host keeps this layout outside every executor checkout:

```text
campaign/
  bootstrap/<sha256>/                  # planner-frozen driver, fixtures and pins
  catalog/<sha256>/                    # complete immutable canonical packages
  harness/<sha256>/                    # accepted executable, source map and lock
  bundles/<sha256>/                    # independently accepted capture products
    manifest.json
    required.json
    actions.json
    environment.json
    adapters/
    oracle-source.bundle
    profiles/
    fonts/
    expected/<lane>/<scenario>/<checkpoint>.frame.json
    expected/<lane>/<scenario>/<checkpoint>.state.json
    provenance/<lane>/<scenario>.json
    repeat-evidence.json
  receipts/<sha256>.json               # host-only acceptance records
  runs/<run-id>/
    context-index.json                 # immutable host binding of all check contexts
    contexts/CHK-NNN.json               # one immutable operation-specific context per check
    progress.md
    freeze.json
    taskfmt-logs/
    candidate/<lane>/<scenario>/
    comparison.json
    tests.json
    verdict.json
  ledger/                             # host-only accepted dependency history
```

Use SHA-256 over canonical manifest bytes. The manifest contains every payload's relative path, size, SHA-256 and media/schema type, sorted by bytewise path. Its own digest is external; avoid a self-referential hash. Exclude mutable timestamps and local absolute paths from canonical payloads. Keep wall-time diagnostics in separate evidence. Reject duplicate keys/IDs, extra required-output files, missing files, unsupported versions, path traversal, symlinks, hard-link escape and archive extraction outside the destination. Validate files before consuming them and revalidate trust roots after execution.

Before package generation, freeze source-level required IDs, finite axes, source-qualified expansion algorithms and independent validation fixtures. These define which oracle states the baseline owners must enumerate; they do not pretend their future captures already exist. During accepted baseline preparation, those algorithms produce `required.json` with every finite scenario/lane/viewport/palette/motion/action/checkpoint identity and `actions.json` with numeric coordinates, exact input bytes and clock steps. Validate and seal that materialized expansion before production repair. A glob, open-ended selector or candidate-provided list cannot define membership. Record explicit, evidence-backed non-applicability separately. A compound description needs an exact source-only expansion contract before its baseline task can be generated, and a flat reviewed numeric expansion before implementation can consume it.

Each receipt binds product digest, producer task and accepted source tree, bootstrap/harness/catalog digests, oracle SHA, tool pins, dependency receipt digests, full gate evidence and reviewer decision. The host owns receipt creation and verification. An expected producer task ID and product name resolve through the host ledger to exactly one accepted digest. An absent, ambiguous, unaccepted or superseded product fails. This permits unknown future output hashes without accepting a candidate-selected hash. No `latest` file, environment override or working-tree manifest participates in resolution.

Oracle bundles may be sealed per application and component namespace. The complete baseline receipt names their exact digests plus the combined required-set digest. B-COMPONENTS cannot seal the complete product until every required namespace is present and duplicate-free. Downstream context binds that complete receipt before any production repair starts.

## Required project command interface

The following `tc-proof` commands are **new deliverables of B-HARNESS**, not commands available today. They are specified here so packages can depend on one bounded implementation. Only the host installation after B-HARNESS acceptance creates `/proof/bin/tc-proof`. Do not invoke them during planning or claim their exit status before implementation.

```sh
/proof/bin/tc-proof preflight --context /run/tc-proof/contexts/CHK-001.json
/proof/bin/tc-proof required --context /run/tc-proof/contexts/CHK-002.json
/proof/bin/tc-proof oracle --context /run/tc-proof/contexts/CHK-003.json --namespace showcase
/proof/bin/tc-proof oracle --context /run/tc-proof/contexts/CHK-003.json --namespace holla
/proof/bin/tc-proof oracle --context /run/tc-proof/contexts/CHK-003.json --namespace jackin
/proof/bin/tc-proof oracle --context /run/tc-proof/contexts/CHK-003.json --namespace tablepro
/proof/bin/tc-proof oracle --context /run/tc-proof/contexts/CHK-003.json --namespace components
/proof/bin/tc-proof capture --context /run/tc-proof/contexts/CHK-002.json --lane direct
/proof/bin/tc-proof capture --context /run/tc-proof/contexts/CHK-003.json --lane pty
/proof/bin/tc-proof compare --context /run/tc-proof/contexts/CHK-004.json
/proof/bin/tc-proof account-tests --context /run/tc-proof/contexts/CHK-005.json
/proof/bin/tc-proof architecture --context /run/tc-proof/contexts/CHK-006.json
/proof/bin/tc-proof close --context /run/tc-proof/contexts/CHK-007.json
```

These are package-specific alternatives, not commands sharing one context: for example baseline CHK-003 is oracle while repair CHK-003 is PTY capture. The immutable package determines the actual operation. The host freezes `/run/tc-proof/context-index.json` and one `/run/tc-proof/contexts/CHK-NNN.json` for each declared operation check before verification. The index binds run, task, frozen tree, catalog, trust receipts and each child's exact bytes/hash, strict schema, operation, lane or namespace, required-set identity and predetermined output identity. Existing comparator and runner context schemas remain distinct. No context is rewritten between commands, and no operation borrows another check's context. Missing, extra, substituted, cross-run or mutated members fail independently. Index integrity and member isolation are independently qualified by TASK-001/TASK-070 before consumers dispatch.

All commands reject unknown flags, missing context fields and context hashes inconsistent with the host invocation. They accept no approval, alternate oracle, relaxed threshold, candidate-defined normalization, skipped scenario or writable comparator option. Exit 0 means the named operation completed and passed its contract; nonzero means failure. Each writes one schema-versioned result to its host-assigned output namespace. The index binds expected output identities, never fabricated hashes of future results. After each operation the host records independently verified actual status, observed process evidence and output hashes against that same immutable index. `close` reruns integrity and joins these host-authentic completed operation records; missing/failed/replayed records or a candidate-written success marker cannot pass.

`preflight` resolves accepted prerequisite receipts, verifies their source commits are ancestors of the actual integrated parent, checks toolchain/environment/source pins, and validates task/check selection. `required` expands and checks bidirectional matrix/traceability membership. `oracle` rebuilds only the pinned oracle and accepted adapter, captures twice in fresh isolated directories, and checks exact repeat equality. `capture` builds the frozen candidate with accepted observation adapters and executes the required action subset; it cannot read expected artifacts. `compare` runs outside the candidate process boundary and consumes candidate outputs as untrusted data. `account-tests` executes the full required inventory and enforces the stage policy below. `architecture` combines existing architecture commands with independently qualified ownership probes. `close` cannot manufacture absent outputs or rerun a failed capture with altered inputs.

The host-only interface is also owned by B-HARNESS:

```sh
tc-proof-host install --receipt /absolute/accepted-harness-receipt.json --destination /absolute/install
tc-proof-host prepare --campaign /absolute/campaign --task terminal-components/completion/NNN --parent INTEGRATION_PARENT_COMMIT --run /absolute/run
tc-proof-host freeze --run /absolute/run --candidate /absolute/executor-checkout
tc-proof-host verify --run /absolute/run
tc-proof-host seal --run /absolute/run --product oracle-showcase
tc-proof-host integrate --run /absolute/run --ref refs/heads/refactor/holla-parity --expected-parent INTEGRATION_PARENT_COMMIT
```

These commands are a required host implementation, not taskfmt subcommands. `install` initially runs from the independently accepted harness build; its receipt is validated by the bootstrap driver/host, not by trusting its own output. `prepare` derives the operation context set from immutable catalog and accepted receipts; `freeze` finalizes its tree binding and seals the index and child bytes before verification. `seal` is permitted only for the baseline producer named by the catalog and only after independent source rebuild/repeat/review evidence passes. Implementation-task contexts have no seal authority. `integrate` updates only the named local integration ref. It never pushes or merges into main. Final publication/merge remains a separately authorized action.

The generated `verify.toml` files invoke these fixed absolute commands through `argv`, with requirement/acceptance references and expected exit 0. The task gate check invokes `close`, not recursive `taskfmt verify`. Each package's context authorizes a fixed set of check IDs and products. If a proposed task needs a new operation, add an explicit prerequisite harness extension with independent qualification before writing that task's checks; do not invent an unowned command.

## Preparation accounting and inline assertion authority

TASK-071 implements independently qualified host-selected `preparation` and `production` accounting modes without a permissive caller flag. The context fixes mode and legal task IDs. TASK-002–008 use preparation mode, never require their own future accepted product, and do not depend on a future TASK-008 receipt. Before dispatch the host independently discovers the pinned main/oracle inventory from immutable source/module/profile inputs and actual compiler listings and executes it completely. TASK-002–007 use the protected source-derived preparation expectation register. TASK-008 instead uses the independently measured post-approved-test-migration register defined below: its correct replacement assertions may expose product failures absent from the old main expectations. These are host observations, not accepted candidate inventory. Candidate product proposals are checked against this independent authority; they cannot define membership, excuse omissions or relabel failures. A preparation success certifies complete honest production of the assigned artifact, not passing application parity. Production tasks require both accepted TASK-007 inventory and TASK-008 disposition receipts plus the monotonic closed-set ledger. Missing receipts cannot silently select preparation mode.

TASK-007 inventories every inline test across all crate/app source trees as well as external tests and generated/macro cases, reconciling source spans with compiler identities. [inline-test-source-scope.md](inline-test-source-scope.md) freezes the initial finite source locations. TASK-008 has explicit file permissions only for independently approved assertion-span changes: before its candidate edits, the host reviews each proposed patch against accepted inventory and oracle evidence and freezes source blob, exact old/new span bytes/hashes, test identity, retained assertions and decision. This patch review is narrow input authority, not a premature product acceptance receipt. Its later full gate independently checks the implementation against that manifest and all actual execution. Outside-span source bytes, cfg/ignore/module structure and compatible assertions remain unchanged; original whole-file bytes remain archived. Proposals cannot enlarge scope or approve their own patch. TASK-071/TASK-072 qualification rejects missing/renamed execution, self-approved register/disposition, broadened spans, outside-span edits, suppressed tests and archive changes.

Before TASK-008 executor dispatch, the operator independently applies that exact reviewed assertion-span patch manifest to a fresh disposable checkout of the recorded integration parent, verifies the unchanged-byte projection outside approved test spans, and executes the complete required inventory with the pinned profiles/toolchain and no fail-fast omissions. No production correction is applied to this preparation checkout. Preserve the pre-patch inventory/results and original blobs separately. Freeze the resulting post-approved-test-migration expectation register, binding original parent/tree, patch-manifest digest, independently derived patched tree, complete execution evidence and each test's exact identity/outcome/classification. Every newly exposed oracle-conflict failure additionally binds decisive source/oracle evidence and its canonical correction/closing owner; it remains visibly failed until that owner repairs production. This narrow input-preparation step uses accepted TASK-007/TASK-006 authority and independent operator review, never TASK-008's own future disposition receipt or candidate results.

TASK-008 accounting compares against this post-migration register, not a requirement to preserve the obsolete pre-patch pass/fail vector. It permits only the exact independently observed and source-justified newly exposed oracle failures, plus recorded retained failures; unknown or unapproved new failures, changed classifications, absent executions and compatible-assertion regressions fail. Parent, approved patch or tool/profile changes invalidate the register and require independent remeasurement before dispatch; candidate-controlled substitutions cannot refresh it. TASK-071/TASK-072 qualification includes a passing approved migration from an obsolete passing assertion to an honestly reported future-owner oracle failure, followed by rejection of an unapproved new failure, patch swap, wrong parent, hidden pre-patch evidence or broadened assertion span. No such failure becomes parity success or an ignored test.

## Candidate observation ownership

TASK-002–006 produce the immutable observation schema, logical identity mapping and observation requirements alongside reference adapters. TASK-070 owns the independently qualified capture machinery; it does not author application behavior. Each component/application implementation task explicitly owns extraction-only untrusted seams inside its already writable source or completion-test files. These seams compile with the frozen candidate, serialize actual production state, and carry source/binary/action provenance. They need no independent accepted-adapter receipt before their own candidate capture: they remain untrusted inputs to the accepted runner and comparator throughout. Protected reference adapters, schemas, identity mappings, action sets and judges remain forbidden to the repair executor.

The observation path must invoke real production update/draw and capture the actual frame. It may read actual edit/focus/selection/target/overlay/domain state; it may not synthesize oracle state, copy production logic, alter behavior under tests or return a detached fixture frame. TASK-070's independently frozen cases require variable state/action responses and reject wrong state, constant state, omitted fields, wrong source/binary and test-only-path substitution. TASK-072 enforces extraction-only ownership and real production call paths. New cross-owner source permissions or schema changes require a separately reviewed prerequisite assignment; a task cannot infer them from this rule.

## Isolation, freeze, verification and integration

The mandatory [campaign executor adaptation](campaign-executor-protocol.md) explicitly supersedes the canonical template's executor-authoritative and empty-progress sequence while preserving the canonical AGENTS bytes and task schemas. Executor checks are advisory. An untrusted request/progress handoff causes an operator to invoke existing host freeze/verify commands; only the protected host's exact-tree verdict, complete progress and actual independent check records authorize integration. This is an operator workflow, not a new CLI or scheduler capability.

The host uses an isolated executor and separate verification workers. Candidate builds, build scripts, tests and application processes are arbitrary code. Read-only flags on files owned by the same executor account do not establish a trust boundary. Qualification must prove that candidate processes cannot write the host ledger, catalog, accepted binaries, expected frames, signing material, parent repository refs or another run's outputs. Do not mount a host Docker socket or credentials. Pin worker image/toolchain digests; deny network and use preinstalled locked dependencies. Separate owned PTYs and temporary application data are required.

The candidate capture worker receives frozen source, accepted adapter/action inputs and an empty output directory. It receives no expected frames and no host ledger. The comparator worker receives expected artifacts read-only and validated copied candidate artifacts; it never executes candidate code. Taskfmt and the trusted dispatcher run under the host verification authority. Candidate tests run only in subordinate workers. Prevent worker environment variables, PATH changes or repository-local tools from replacing the trusted dispatcher/compiler. Hash checks before and after running are necessary but do not replace isolation against temporary mutation.

The host prepares the accepted integration parent, overlays approved trusted inputs if needed, and records both the original parent and resulting scope-base commit. Standalone taskfmt never performs this overlay automatically. The overlay may add verification assets; it must not conceal production changes. An overlay tree is not an already passing candidate. Every task's forbidden paths protect oracle artifacts, comparator/loader/dispatcher, fixture membership, test dispositions, profiles/fonts and tool pins. Approved trust inputs mounted externally are protected independently of Git scope checks.

At freeze, stop executor access to the candidate copy. In a fresh host-owned object database, reconstruct the candidate tree from the recorded parent plus allowed changed files. Include relevant untracked source files; reject unexpected ignored source, hidden index flags, changed Git configuration, unsafe links, submodule substitutions and paths outside scope. Do not trust the executor's index or Git aliases/hooks. Record parent, scope base, complete candidate tree, source file map and task/trust digests. Verify from a new checkout of that tree with isolated target/output directories. Freeze also snapshots the external progress file; progress is evidence of claimed completion, not verification authority.

The host `verify` operation invokes the existing pinned standalone taskfmt command shown above with explicit paths and base. It collects every check's actual status and logs and rejects missing logs/results. Independent postchecks require unchanged source and trust hashes and unchanged candidate tree after all checks. A passing taskfmt result cannot override a failed independent integrity check. Store the final verdict only in the host ledger, bound to the tested tree and dependency receipts; do not synthesize monitor `done` state or taskfmt lifecycle manifests.

Integration constructs a commit with exactly the verified tree and recorded integration parent. Use `git commit -s` and `Co-authored-by: Codex <codex@openai.com>` when creating that commit, then assert its tree equals the verified tree. Advance the local integration ref through `git update-ref REF NEW_COMMIT EXPECTED_PARENT`. A changed parent rejects the update. Re-integrate and reverify against the new parent; never attach a tested tree to an untested parent. Parallel siblings require a fresh join tree and rerunning the union of affected task gates, full test accounting and whole-workspace compile/lint gates before exposing the join as a downstream base.

## Oracle derivation and exact comparison

The reference runner imports the oracle Git bundle into an empty repository and checks the resolved commit/tree against the fixed oracle SHA. It cannot accept a source directory supplied by a component/app executor. The source map records every original file and any reference adapter delta. Only reviewed observation, time and input-orchestration seams are allowed. Reversing the allowlisted patch must reproduce original blob hashes. Baseline adapters may call private production handlers; they may not copy rendering, business logic, hit testing or substitute expected state. The existing clock patch and boundary probes in [verification.md](verification.md) are a concrete initial qualification case.

For each scenario, capture the real production frame and separately serialize semantic observations. Validate both frames with `Frame::validate`/`Frame::from_json`, require schema 3, equal dimensions and no `diff_cells` results. Validation is mandatory before `diff_cells`: its implementation zips the cell vectors and does not itself reject malformed vector length or schema. It compares cursor as well as cell content. Provenance is intentionally excluded from visual equality and must pass the independent source/binary/action/environment binding checks. A frame digest is not cryptographic integrity proof.

Require exact semantic object equality under the fixed schema: focus/edit owner, caret/selection, selected identity, scroll bounds/offsets, overlay/capture owner, model effect and navigation/exit outcome as applicable. Unknown fields, missing required observations and candidate-selected identifiers fail. Logical identity normalization belongs to the reviewed adapter and is fixed before capture. No masks, tolerance, region exclusion, time erasure or image-only matching.

Where PNGs are required, call `Store::check(..., 1.0)?.ensure_matched()` using the pinned profile/font. The tool may regenerate a missing approved PNG in memory; the project artifact validator must still reject a missing PNG when the manifest declares one required. Store's lack of an approval write is useful but does not establish full provenance or required-set completeness. Direct and PTY lanes compare against their own oracle lanes. Candidate outputs never populate `approved/` and no task may invoke `tuisnap accept`.

Capture repeat evidence uses fresh builds/runs and canonical payload equality. It separately records different binary/source provenance where expected; wall timestamps are diagnostic, not UX. Component and app selectors, exact event bytes, time steps and checkpoint order are frozen before reference capture. An adapter that cannot observe a required state leaves an explicit failed preparation obligation; it cannot mark the state skipped or invent a frame.

## Independent qualification cases

The planner-owned bootstrap fixture pack must contain at least one independently authored valid schema-3 frame pair with styled narrow and wide cells, styled continuation, combining grapheme and non-default cursor; a matching semantic pair; a finite ordered recipe; and source/provenance fixtures. Include valid cases where visually identical frames have different legitimate source provenance so provenance checks do not accidentally require oracle and candidate commits to be equal.

For each single mutation below, the driver requires nonzero status and a specific failure category, verifies no receipt/ref/expected artifact was changed, then reruns the untouched positive case successfully:

- Glyph, foreground, background, DIM, reverse, hidden/blink, wide lead/continuation, cursor position/visibility/style/blink, dimensions, schema, malformed row-major cells.
- Focus/edit owner, selection direction, hitbox boundary effect, scroll boundary, overlay capture, action order, missing press-before-release checkpoint, timer boundary.
- Missing/corrupt expected file, missing/extra/duplicate scenario or checkpoint, altered required set, source SHA, adapter, comparator, profile, font, fixture, lock or tool binary.
- Candidate-written approval, environment blessing, stale candidate binary/source pair, reused output from another task/tree, unsupported normalization, incomplete comparison result.
- Symlink/path traversal/hard-link escape, attempted trust write followed by byte restoration, candidate process access to host receipt/ref directories, premature sealing, forged predecessor receipt, unintegrated predecessor and changed-parent ref update.
- Partial full-test execution, dropped or renamed required test without approved relocation, unregistered failure, previously closed scenario failing, candidate-edited allowed-failure list and missing subordinate worker result.

Use tiny independent Git repositories for freeze/ref tests; never the real repository. Assert exact tested-tree identity and no ref changes on rejection. Test the standalone taskfmt wrapper against a valid canonical fixture package with an out-of-scope edit, failed check, incomplete progress, and a process that prints `DONE` but exits nonzero. Neither stdout alone nor a copied successful report may pass.

The comparator fixture pack and black-box driver now exist under `evidence/`. Its self-test constructs 72 cases, rejects nine lying/incomplete runners including first-result-only, case-label lookup and missing negative fields, rejects 888 report-schema/aggregation mutations, and successfully validates the positive schema-3 frame through the repaired tuisnap executable. Explicit guards remain active under Python optimization. Opaque run/path identities, shuffled cases and fresh inert payload salts prevent trivial public label/hash-table verdict lookup. Protected context IDs/counts remain available when required artifacts are corrupt. Exact complete-set results cover multiple checkpoints and separate scenarios, including first/middle/final failures, mixed passing/failing members, missing final artifacts and duplicate/substituted result identities. Adapter identity, duplicate JSON/manifest paths and cross-task/run replay have explicit rejection cases. The future comparator must pass all 72 cases plus 69 fresh positive recoveries preserving required-set cardinality. Host freeze/isolation/ref-update qualification remains a separate fixture suite. Neither self-test claims a production harness has passed.

## Existing tests, oracle conflicts and intermediate gates

Preserve historical archives under their original provenance. Main's older render baselines and recipes are not the new oracle. A test asserting obsolete product behavior must not force a repair task to preserve that behavior, and a broad baseline rewrite must not erase its useful architecture assertions.

B-INVENTORY produces exact identities `(source SHA, package, target, test name, feature/toolchain profile)` and required execution commands. B-TEST-DISPOSITION records each oracle conflict with original blob/expectation, specific oracle evidence, retained architecture assertions, replacement identity, primary correction owner and relocation map. It replaces only conflicting product assertions with protected oracle assertions. An assertion that is merely inconvenient is not an oracle conflict. Any relocation or expectation change requires its immutable disposition record and replacement proof; archive bytes remain unchanged.

At every production task stage:

1. Whole-workspace build, MSRV compile, formatting, lint, backend-free, examples and compatible architecture checks must pass. If existing failures prevent this, add a bounded preparation correction owner before production dispatch. Do not describe a red compile/lint gate as a buildable green task.
2. All task-owned and previously closed oracle scenarios must pass. Dependency ancestry does not make every future-owned app scenario a component's parity gate: the immutable stage map fixes ownership. TASK-009–031 compare complete direct component fixtures; their full application PTY captures remain diagnostic until the fixed app owner closes them. Capture completeness is not comparison success. TASK-073 proves attribution/registry architecture and accounting without inventing oracle frames or a zero-case PTY success. A genuinely inapplicable lane is recorded as non-applicable, never matched parity. Shell contributions use the frozen semantic and complete-frame contribution tables: only explicitly owned, nonempty, complete-frame checkpoints may close; no crop or parent-row pass is inferred. All previously closed scenarios must continue passing. The host ledger monotonically records newly passing scenarios; an executor cannot reopen them.
3. Execute the complete required test/scenario inventory as a diagnostic sweep, including future-owned oracle failures. Record every actual result, including failures. Use no-fail-fast test execution so an early failing binary does not suppress later targets.
4. The accounting check accepts only the exact still-unfinished future-owner identities admitted by the immutable stage map minus closed identities. It fails on missing execution, unknown failures, unexpected errors or changed failure classification. A diagnostic failure remains a failure in the report; it never becomes a parity pass or an ignored test.

This allows scoped completion during a migration whose full oracle suite starts red. It does not weaken any finished obligation. Every allowed unresolved entry has a fixed correction owner, source evidence and closing stage. At each app closure all that application's unresolved entries are empty; at X-FINAL the global set is empty. The final unfiltered primary workspace suite must pass. Do not claim unconditional `cargo test -p holla` success for H-SHELL if later Holla tasks still own known oracle failures.

Existing main CI commands at `7b27732a:.github/workflows/ci.yml` include these real gates:

```sh
cargo metadata --locked --format-version 1
cargo +1.88.0 check --locked --workspace --all-targets --all-features
cargo check --locked -p junie-tui --no-default-features
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --locked --workspace --all-targets --all-features
cargo build --locked -p junie-tui --examples
cargo test --locked --workspace --all-targets --all-features --no-fail-fast
cargo test --locked -p junie-tui --test render --test render_components
cargo test --locked --workspace --doc --all-features --no-fail-fast
cargo doc --locked --workspace --all-features --no-deps
cargo run --locked -p xtask -- app-inventory --json
cargo run --locked -p xtask -- boundary
cargo run --locked -p xtask -- bless-guard
cargo run --locked -p xtask -- doc-check
```

Set `RUSTFLAGS=-D warnings` and `RUSTDOCFLAGS=-D warnings` through the trusted worker environment. Supply `BLESS_GUARD_BASE` as the recorded real parent/base; never compare a checkout to itself. Also preserve exact render identity reachability, empty allowlist checks, accepted performance gates and the independent ownership probes. A listing-only test command proves inventory, not execution. B-INVENTORY must record all test targets and verify every required result; cargo's aggregate exit code alone does not prove complete execution.

Main CI intentionally assigns compiler-sensitive `architecture::compile_fail_cases_hold` snapshots to the primary toolchain and skips that name only on MSRV. Preserve the assertion on primary and preserve MSRV compilation/behavioral coverage. This existing profile assignment is not permission to skip a new failure or exclude the assertion globally. Pin actual primary/nightly versions in the host environment receipt while retaining the stable-update policy as an explicit separately qualified refresh.

Known main boundary/parity portability failures and obsolete self-baseline assertions require exact disposition/correction owners. B-TEST-DISPOSITION owns test authority changes; it cannot silently waive a production or architecture defect. The final task graph must place necessary correction tasks before any dependent gate demanding their success.

## Readiness blockers

Before canonical package generation, finish independent review and freeze the bootstrap drivers/fixtures, resolve final tuisnap PR/revision, freeze source-level required IDs/finite axes/exact source-only expansion algorithms and map every protocol operation to its bounded producer. Do not require future captured hashes or flat numeric traces at this stage. Before B-HARNESS acceptance, implement and independently qualify both executables, isolation, installation, freeze, evidence index and ref-update behavior. Before any component/app implementation, materialize and independently validate every flat numeric trace and oracle namespace, accept exact test dispositions/stage maps, then seal their complete baseline receipt.

No complete oracle bundle digest can be recorded today. No `tc-proof` command above is implemented by this document. Those are explicit prerequisite deliverables with independently testable contracts. An execution-ready catalog must not replace them with shell placeholders, editable success files, fabricated hashes or broad permission to bless.
