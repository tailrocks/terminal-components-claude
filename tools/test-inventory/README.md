# Exact test inventory

`inventory.py` uses Cargo and stable libtest/rustdoc interfaces. It does not scan
Rust function names or accept a nonzero test count as coverage.

Requirements: Python 3.11+, Git, Cargo/Rust (tested with 1.88 and stable).

```sh
python3 tools/test-inventory/inventory.py capture \
  --root "$PWD" --profiles tools/test-inventory/profiles.json \
  --toolchain 1.88.0 --execute --output /external/evidence/test-capture.json
python3 tools/test-inventory/inventory.py verify \
  --root "$PWD" --capture /external/evidence/test-capture.json \
  --required tools/test-inventory/required.json
```

Capture writes evidence, even when compilation or target coverage is blocked.
It never writes requirements, mappings, snapshots, or source. Omit `--execute`
for listing only; listing cannot pass verification. Existing output is refused.
The wrapper deliberately records hashes of command output instead of retaining
fixture payloads or failure diagnostics. Commands, test identities/statuses,
compiler versions, source/lock/environment fingerprints and executable hashes
remain reviewable. Only a small environment allowlist reaches subprocesses;
baseline blessing and arbitrary Rust/test flags are not inherited. Each command
has a 20-minute timeout. Baseline/environment-sensitive tests needing additional
variables need a separately reviewed wrapper change, not an implicit passthrough.

Each explicit package/feature profile gets a new Cargo target directory. Only
that invocation's `compiler-artifact` messages with `profile.test=true` and an
executable qualify. Package/kind/target keys must be unique. This prevents reuse
of binaries produced by another profile or earlier capture. Cargo may still
unify dependency features within one package's dev-dependency graph: recorded
artifact features expose that result; this tool does not replace the isolated
backend-free consumer gate.

Cargo-owned testable library, binary, integration and example targets must
produce artifacts. `test=false` targets are explicitly classified. Custom
`harness=false` and benchmark targets block coverage pending an explicit adapter;
their existence is never treated as zero tests. Inactive `required-features`
targets are explicitly classified with required/active feature sets. Activation
comes from same-package artifacts in that exact isolated Cargo invocation;
conflicting artifact feature sets fail closed. A package producing no artifacts
uses its declared local feature closure (default, aliases and transitive local
features); unsupported qualified selectors fail. An enabled target without its
expected artifact remains a blocker. Workspace-wide metadata feature unions and
manual target skip lists never decide activation.
Library rustdoc targets are listed and run separately, including compile-fail
and should-panic modes. Documentation identities retain their actual source
line: moves require a reviewed mapping, not fuzzy matching.

Each compiled target is listed and executed through Cargo with its exact
`--lib`, `--bin NAME`, `--test NAME`, or `--example NAME` selector and the same
isolated target directory/feature flags. This preserves Cargo's package runtime
environment as well as package working directory; both command and target record
the canonical package cwd. Artifact bytes are checked again after execution.
Listing uses `--list --format pretty` (terse omits the count summary on Rust1.88).
Execution uses `--format pretty --test-threads=1` with no filter, plus a fresh
external `--logfile` for exact libtest statuses. This stable-but-deprecated option
is verified on MSRV/stable; if unavailable, capture fails closed. Its separate
status channel prevents subprocess stdout from splitting/impersonating pretty
result lines. Only the status-file hash and parsed identities/statuses survive
capture; ignored reasons or fixture output are not copied into the evidence.
Rustdoc uses its normal Cargo-owned listing/execution interface. Names and
statuses must agree with all libtest summaries. Missing,
duplicate, measured, filtered or unexecuted names fail. Ignored tests require an
explicit per-identity reason in the reviewed requirements. Empty targets also
require an explicit reviewed reason. A compile failure is coverage failure,
not an empty successful inventory.

## Historical obligations and approval

`historical.json` preserves 3,211 source-qualified obligations from the pinned
initial repaired-main inventory (2,470), Holla reference inventory (269), and
publication parent identity map (472). Original artifact SHA256s and exact
source revisions are recorded. These sets overlap intentionally: they represent
distinct source obligations, not a claim of 3,211 distinct current tests.
Historical executable basenames retain build hashes because otherwise identical
`perf` target/test names from different packages would collapse. The initial
repaired-main inventory is explicitly identified as such, not mislabeled as an
unchanged MAIN_BASE execution.

`required.json` is intentionally **pending**, with every historical obligation
unresolved and no approved target matrix. Current missing coverage is not
blessed. Integration must review target-qualified identities and exact
relocations, then populate:

- `targets`: `profile`, `package`, `kind`, `target`, exact `identities`, optional
  `ignored` identity-to-reason map, and `empty_reason` where appropriate.
- `obligations`: every catalog `source_id`, a review reference, and one or more
  destination tuples `[profile, package, kind, target, test_identity]`.
- `approval: "reviewed"` only after those requirements are accepted.

Verification compares the full target/profile/name matrix, verifies catalog
fingerprint and exact obligation coverage, and requires every destination to
exist in executed coverage. Removing an obligation or target cannot make a
missing test disappear. Capture source and lock fingerprints must match the
current checkout. Schema/fixture tests use a separate synthetic reviewed map;
they do not approve this project's pending requirements. CI/xtask integration
is owned separately; this slice does not alter their current behavior.

```sh
cd tools/test-inventory
python3 -m unittest -v test_inventory.py
INVENTORY_TEST_TOOLCHAIN=stable python3 -m unittest -v test_inventory.py
```

The tests create a dependency-free Cargo fixture with library, binary-local,
integration, feature, ignored, normal-doc and compile-fail-doc tests. Mutation
checks cover missing names/targets/features/docs, duplicate identities/targets,
zero-filter results, listing without execution, unresolved/deleted historical
mappings, unreviewed ignored tests, duplicate profile commands, custom harnesses
and stale output publication.
The feature matrix regression covers a real gated integration target both
disabled and enabled, actual-artifact feature authority, transitive default
activation without artifacts, duplicate artifacts, and missing enabled targets.

The execution-context regression uses an actual nested workspace package with
relative input data and runtime Cargo package variables, compares genuine Cargo
execution, and reproduces child stdout interleaving. Missing/duplicate/unknown
status-file identities remain failures; using a dedicated status file does not
permit filtered or empty execution to satisfy required names.
