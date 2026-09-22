# Subagent proof and verification contract

**Authority:** [`subagent-only-policy.md`](subagent-only-policy.md) and
[`path-contract.md`](path-contract.md).

This contract defines evidence for the refactoring tasks. It does not create a
host daemon, container workflow, receipt service, lifecycle CLI, or integration
authority. The coordinator delegates implementation, verification, and review
to subagents; the coordinator integrates only reviewed task commits.

## Fixed inputs

- Product/source oracle: peeled `visual-baseline` commit
  `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Architectural parent: `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Latest taskfmt source:
  `/Users/donbeave/Projects/taskfmt/task-format`.
- Latest taskfmt revision:
  `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`.
- Latest taskfmt version: `0.2.0`.
- Frozen `visual-baseline` tag/release/store: policy-protected and never
  mutated; provider-level immutability is not assumed.

Taskfmt is used only for one task at a time:

```sh
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify \
  --root "$WORKTREE" \
  --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" \
  --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

No `taskfmt init`, `taskfmt status`, `taskfmt-host`, `taskfmt-runtime`,
`taskfmt run`, `taskfmt monitor`, `taskfmt promote`, or other lifecycle command
is permitted. Taskfmt never starts or supervises a container.

## Check workers

`verify.toml` check commands are untrusted task workers. They run on the host
inside the verifier subagent's isolated worktree and external run directory.
They may execute real production code and tests, but they must not write:

- task packages or trusted fixtures;
- the immutable visual oracle or baseline tag;
- another task's worktree or run directory;
- coordinator state, refs, or unrelated files.

Allowed proof operations are task check workers such as:

```text
tools/refactor-proof/bin/tc-proof preflight --context RUN_DIR/contexts/CHK-001.json
tools/refactor-proof/bin/tc-proof required --context RUN_DIR/contexts/CHK-002.json
tools/refactor-proof/bin/tc-proof oracle --context RUN_DIR/contexts/CHK-003.json --namespace APP
tools/refactor-proof/bin/tc-proof capture --context RUN_DIR/contexts/CHK-003.json --lane direct
tools/refactor-proof/bin/tc-proof compare --context RUN_DIR/contexts/CHK-004.json
tools/refactor-proof/bin/tc-proof account-tests --context RUN_DIR/contexts/CHK-005.json
tools/refactor-proof/bin/tc-proof architecture --context RUN_DIR/contexts/CHK-006.json
tools/refactor-proof/bin/tc-proof close --context RUN_DIR/contexts/CHK-007.json
```

These are check workers, not coordinator commands. Context files are
task-scoped, immutable for one verification run, and stored under `RUN_DIR`.
Each context binds task ID, worktree commit, scope base, operation, required
outputs, and expected provenance. Missing, extra, substituted, cross-run, or
mutated contexts fail verification.

Before invoking taskfmt, the verifier subagent must perform native proof
preparation in the isolated worktree: run
`scripts/campaign-build-proof.sh`, materialize the exact context set and
context index under the external `RUN_DIR`, and bind the per-check
preparation results, source/oracle, comparator, and observer capability. The
Rust `tc-proof` launcher then supervises each proof worker. The dispatcher
invokes only standalone taskfmt `lint`/`verify`, passes the explicit
`RUN_DIR/taskfmt-logs` path, invokes `tc-proof validate --run-dir` afterward,
and preserves a nonzero taskfmt status. No observer socket is created inside
the run directory or selected through a worker-controlled default.

The preparation ABI is intentionally shared with
`scripts/campaign_ledger.py`:

The ledger and the external proof document are two different schemas. The
tracked `ledger.preparation` value is the
`campaign-preparation-qualification/v1` wrapper. Its
`proof_preparation` member uses the `preparationProof` definition and contains
only immutable path/hash references. It must not be replaced with the strong
document. The external `$RUN_DIR/proof-preparation.json` file is the
`campaign-proof-preparation/v1` document and is schema-addressable as
`docs/refactoring-plan/campaign-ledger.schema.json#proofPreparation` (the
`proofPreparation` anchor). The wrapper path/hash is checked first; the strong
document and every referenced artifact are then checked by
`validate_proof_preparation` / `validate_preparation_qualification`.

The strong document is sealed after native materialization and never contains
its own future hash. This keeps evidence non-circular: the external file binds
the run inputs, while the tracked wrapper records that file's immutable hash.

Native proof layout is verifier-owned and exact:

```text
$RUN_DIR/
  target/debug/tc-proof
  target/debug/tc-proof.build.json
  contexts/CHK-NNN.json
  results/CHK-NNN.json
  outputs/
  taskfmt-logs/
  observer.json
  context-index.json
  proof-preparation.json
```

`target` is a direct child of `$RUN_DIR`, and the run root contains exactly
these members after preparation. The target is never the candidate worktree's
`target` path or a shared-cache symlink. Every trust file and output reference
must resolve through real path components to a regular single-link file;
symlink and hard-link substitution is rejected. The JSON schema describes the
ABI shape; the native validator additionally checks bytes, hashes, source
commit/tree, oracle identity, worker identity, comparator identity, taskfmt
provenance, observer nonce/capability, trust-manifest digest, and exact run
membership.

- `context-index.json`: `tc-proof-context-index/v1`, with `task_id`,
  `run_id`, `worktree_commit`, `scope_base`, `contexts`, `results`, and
  `observer`.
- `contexts/CHK-NNN.json`: `tc-proof-context/v1`.
- `results/CHK-NNN.json`: `tc-proof-preparation-result/v1`, status `ready`.
- `observer.json`: `tc-proof-observer-capability/v1`, transport
  `inherited-pipe/v1`.

Worker results are separate host-selected artifacts under
`RUN_DIR/outputs/CHK-NNN.result.json` using
`tc-proof-runner-result/v1`; they are not substituted for preparation
results. Runtime closure is split by the bound operation:

- Native members require their exact result, any declared exact
  `CHK-NNN.compare.json` comparator report, and the native `close` result when
  present. Existing result identity, passed status, observer-event, comparator,
  and close checks remain mandatory.
- Members classified as `operation=external` are taskfmt-driven. Their bound
  context and preparation result remain mandatory, and taskfmt's exit status
  plus a final `DONE` log line remain authoritative. An external driver may
  emit no runtime artifact, or may emit its exact bound result/report paths;
  present artifacts still pass the existing schema, identity, and trust-path
  checks. Missing external artifacts are allowed. Any unbound output remains an
  extra-file failure.

A task-specific comparator retains `tc-proof-compare-context/v1` as the nested
`qualification.comparator.context`, while the immutable runner context binds
its identity and exact `CHK-NNN.compare.json` report path under the allowed
runtime output directory. This does not relax native observer binding or allow
external artifacts outside their prepared names and paths.

The observer request/response transport is one inherited-pipe protocol. Each
message binds run, task, check, nonce, request order, operation, source
commit, and source tree. An accepted response must use the exact schema,
report successful execution, and contain non-empty payload and records;
missing, replayed, truncated, empty, forged, alternate-transport, or
incomplete-close evidence fails closed. The native Rust launcher never
fabricates an observation. If an independent observer response provider is
not attached, the launch is rejected rather than accepting a worker-written
passed result. The protocol is an integrity and execution-observation control
for the native same-user threat model; it does not claim arbitrary filesystem isolation from a hostile process running as the
same user. Seatbelt or container isolation is not part of this contract.

## Evidence ownership

The implementer subagent produces a scoped commit and advisory test output.
The verifier subagent produces raw `taskfmt lint`/`verify` output plus check
logs. The reviewer subagent independently checks:

1. task scope and writable paths;
2. exact worktree commit and scope base;
3. taskfmt executable revision/version/SHA;
4. check exit codes, final `DONE`, and required outputs;
5. no oracle, trust-root, ref, or unrelated-file mutation.

The reviewer returns `VERIFIED`, `REJECTED`, or `BLOCKED`. A passing taskfmt
result alone never authorizes integration or task completion.

## Visual and behavioral evidence

Visual comparisons must read the frozen oracle; candidates never write or bless
it. Candidate capture output is untrusted and lives only in `RUN_DIR` until
the reviewer accepts the task evidence. Behavioral evidence must cover real
production update/draw/event paths, not detached fixtures or copied oracle
frames. Any missing oracle asset is a blocker, not a reason to weaken a gate.

## Integration

After verifier and reviewer approval, the coordinator integrates the exact
subagent commit against the expected parent with ordinary Git compare-and-swap
checks. Integration is not performed by taskfmt or a proof worker. Use the
repository commit policy: `git commit -s` and
`Co-authored-by: Codex <codex@openai.com>`.
