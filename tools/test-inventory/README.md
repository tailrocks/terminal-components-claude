# Exact cargo-nextest inventory

`inventory.py` uses Cargo metadata plus the standalone `cargo nextest` listing
and structured result interfaces. It does not scan Rust function names or
accept a nonzero test count as coverage. It never invokes `cargo test`.

Requirements: Python 3.11+, Git, Cargo/Rust, and cargo-nextest 0.9.143 or a
newer reviewed version (tested with rust-toolchain.toml 1.98.1).

```sh
python3 tools/test-inventory/inventory.py discover \
  --root "$PWD" --seed docs/refactoring-plan/inline-test-source-scope.md \
  --output /external/evidence/source-discovery.json
python3 tools/test-inventory/inventory.py reconcile \
  --root "$PWD" --discovery /external/evidence/source-discovery.json \
  --canonical docs/refactoring-plan/historical-obligations-canonical.tsv \
  --output /external/evidence/obligation-proposals.json
python3 tools/test-inventory/inventory.py capture \
  --root "$PWD" --profiles tools/test-inventory/profiles.json \
  --toolchain 1.98.1 --output /external/evidence/test-capture.json
python3 tools/test-inventory/inventory.py bind-listing \
  --root "$PWD" --capture /external/evidence/test-capture.json \
  --required tools/test-inventory/required.json \
  --output tools/test-inventory/listing.json
python3 tools/test-inventory/inventory.py capture \
  --root "$PWD" --profiles tools/test-inventory/profiles.json \
  --toolchain 1.98.1 --execute --output /external/evidence/test-execute.json
python3 tools/test-inventory/inventory.py verify \
  --root "$PWD" --capture /external/evidence/test-execute.json \
  --required tools/test-inventory/required.json
```

`discover` walks every workspace crate/app/xtask/example/test crate root, follows
`mod` declarations including `#[path]` and cfg(test) modules, expands known
test-generating macros (`conformance_suite`, `matrix`, `baseline_case` and the
combo/audit wrappers), records rustdoc fences (including `include_str!`
markdown) and trybuild `compile_fail` globs, and classifies each assertion
`preserve` or `oracle-conflict`. Parsing is not execution. The 146-path
`inline-test-source-scope.md` list is a seed: every seed path must exist and be
reached; extra external/generated identities are required completeness, not a
license to skip the seed.

`reconcile` maps the 3,211 historical catalog identities onto discovered
package/kind/target/identity tuples and retains the 620-row canonical
historical union. Exact current names map; basename-only hits are
`unapproved-relocation` and stay unresolved. Duplicate-equivalent source rows
remain independent. `required.json` stays `approval: pending` until executed nextest coverage has
no doctest/harness blockers and the listing/execution matrix is reviewed.
`bind-listing` copies nextest-listed identities into `listing.json` and into
`required.json` targets without setting `approval: reviewed` or filling
obligation destinations. Listing is not execution. Capture subprocesses receive
`MISE_NO_CONFIG=1` and `CARGO_HOME/bin` first on `PATH` so mise/mbx wrappers
cannot substitute `cargo test`.

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
of binaries produced by another profile or earlier capture. Nextest's
structured listing supplies exact test identities and runtime binaries; the
compiler-artifact feature set remains authoritative for feature activation.

Cargo-owned testable library, binary, integration and example targets must
produce artifacts. `test=false` targets without test artifacts are explicitly
classified. Cargo's `--all-targets` can emit runnable example test artifacts even
when metadata defaults `test=false`; these actual artifacts are listed/executed,
so that metadata default cannot discard example-local tests. Custom
`harness=false` and benchmark targets block coverage pending an explicit adapter;
their existence is never treated as zero tests. Inactive `required-features`
targets are explicitly classified with required/active feature sets. Activation
comes from same-package artifacts in that exact isolated Cargo invocation;
conflicting artifact feature sets fail closed. A package producing no artifacts
uses its declared local feature closure (default, aliases and transitive local
features); unsupported qualified selectors fail. An enabled target without its
expected artifact remains a blocker. Workspace-wide metadata feature unions and
manual target skip lists never decide activation.
Library rustdoc targets are recorded as explicit blockers because
cargo-nextest 0.9.143 has no doctest runner. Do not restore `cargo test --doc`;
an approved nextest-compatible adapter is required. Documentation identities
retain their actual source line: moves require a reviewed mapping, not fuzzy
matching.

Each compiled target is listed and executed through cargo-nextest with its exact
`--lib`, `--bin NAME`, `--test NAME`, or `--example NAME` selector and the same
isolated target directory/feature flags. This preserves Cargo's package runtime
environment as well as package working directory. Commands and targets record
the workspace invocation cwd; targets separately record the package runtime cwd. Artifact bytes are checked again after execution.
Listing uses nextest JSON. Execution uses `libtest-json-plus`,
`--test-threads=1`, and no filter, with ignored tests included and executed. The
structured execution stream is hashed and parsed, so child stdout cannot
impersonate a test result. Names and statuses must agree with
the nextest events. Missing,
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
python3 -m unittest -v test_source_discovery.py test_reconcile.py test_inventory.py
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
relative input data and runtime Cargo package variables, compares genuine
nextest execution, and verifies that structured child output cannot impersonate
a result. Missing/duplicate/unknown nextest identities remain failures; a
filtered or empty execution cannot satisfy required names.

Cargo metadata, build, listing, and execution all use the same workspace invocation directory, so nested package Cargo configuration cannot substitute another executable. Each command and target records this `cwd`; target `runtime_cwd` records the package directory supplied by Cargo to the test process. Legacy captures without this distinction are rejected.
