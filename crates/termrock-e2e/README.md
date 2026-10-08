# termrock-e2e

This is the shared, product-independent PTY suite package. It depends on the
pinned Tuiscotti toolkit and does not import a Termrock, Ratatui, or preview
application crate.

The first executable case is `HELP-HOLLA-004`. The registry also keeps all 23
known deferred component obligations with `NOT_RUN` status. One pilot case
does not mean that the full old assertion inventory has migrated.

## Run the contract checks

```sh
cargo nextest run --manifest-path crates/termrock-e2e/Cargo.toml --locked --jobs 2
```

The Holla PTY test is ignored during ordinary runs. A launcher must build and
pin both real subjects, write a `termrock-spec/parity-subject-manifest-v1`
JSON file, and pass its path in `TERMROCK_E2E_SUBJECT_MANIFEST`. The manifest
contains a `subjects` array with one reference and one candidate record. Each
subject requires build inputs and a bound
`termrock-spec/parity-subject-build-receipt-v1` payload. The suite checks that
the receipt binds source, package, target, feature/default-feature policy,
toolchain, build inputs, argv, and executable hash.

The caller must also provide an independent trust record through absolute
`TERMROCK_E2E_TRUST_RECORD` and `TERMROCK_E2E_TRUST_RECORD_SHA256` values. The
record must be stored outside the suite checkout, both subject source and
artifact roots, oracle roots, the expected-generation root, and both actual
output roots. The suite verifies its bytes and bindings to the compiled/current
suite digest, case/profile/lock hashes, current test binary, both subject
receipts, and any admitted expected generation before it launches either
subject. The trust record also binds the SHA-256 of the resolved write policy.
Its canonical path
and raw SHA-256 are copied into the run receipt. The record and digest must come
from outside candidate-controlled files and flags; parsing a record does not
authenticate its author. Without this record, the test fails before PTY launch.

The caller must define the write boundary before launch:

- `TERMROCK_E2E_WRITE_ROOT` names an existing absolute scratch directory outside
  the suite checkout, both subject source and artifact roots, the oracle,
  expected-generation data, the external trust record, and the test binary.
- `TERMROCK_E2E_RECEIPT_PATH` names a currently absent file strictly beneath the
  write root.
- `TERMROCK_E2E_PROTECTED_ROOTS` is a JSON array with `subject_source` and
  `subject_artifact` directory entries for each role, plus at least one
  `oracle` directory entry. Each entry has `kind`, `role`, and absolute `path`
  fields. Subject entries use `role: "reference"` or `role: "candidate"`;
  oracle entries use `role: null`.

The runner adds the suite checkout, expected-generation root, trust record and
its parent, test-binary parent, and each executable parent to the protected
set. It resolves and records this policy, then requires the write root,
receipt, and both actual-output roots to be pairwise safe against the protected
roots. Actual outputs must be strictly beneath the write root. The externally
accepted trust record must bind the resolved policy digest, including the
canonical write root, receipt destination, protected-root set, and actual
output roots. The run receipt records that same policy and digest. Receipt
publication uses an exclusive staged file and a no-replacement link; it never
creates parent directories or overwrites an existing receipt. The digest is
SHA-256 over compact UTF-8 JSON for the policy payload; object keys are
recursively sorted, protected roots are sorted by kind, role, and canonical
path, and actual roots are sorted by role. A preflight failure returns
diagnostics without emitting a receipt, so the collector must classify a
missing receipt as incomplete evidence.

The caller should build each subject into an isolated read-only output root.
The suite checks each executable's hash and file metadata at pair preflight,
again immediately before its PTY spawn, and after its journey. Tuiscotti accepts
an executable path, so a same-user replacement in the interval between the
last check and process open remains a filesystem race; this runner does not
claim to pin an executable file descriptor or provide an OS sandbox.

Set `TERMROCK_E2E_RECEIPT_PATH` to write the machine-readable run receipt.
Actual captures go below each subject's `actual_output_root`. They are written
to sibling staging directories; the manifest is written last and the completed
directory is published atomically. Capture and receipt writes share a
component-wise directory check rooted at `TERMROCK_E2E_WRITE_ROOT`. Capture
parents are created one component at a time; existing components are inspected
without following symlinks. A symlink below the write root fails the write.
Failed captures are retained in quarantine. Actual output roots must be
disjoint from each other and every protected root. These path-based checks do
not prevent a concurrent same-user replacement between inspection and
filesystem operations; this runner does not claim race-free directory-handle
confinement.
A matching expected tree hash reports `HASH_MATCH`; it reports `ADMITTED` only
when the external trust record binds an admission receipt. Exact visual
comparison remains blocked in this pilot, and the runner never writes expected
data.

## Current stop condition

Before the write-boundary update, a focused package check used Cargo 1.98.1 and
cargo-nextest 0.9.146 with `--locked --jobs 2`: 21 tests passed and the Holla
PTY journey remained ignored because it needs pinned real binaries and an
admitted expected generation. That earlier check does not verify the current
write-boundary changes and does not run either product binary or qualify visual
parity. The pilot is not a complete suite or visibility acceptance.
`cases/deferred-obligations.json` records 23 known deferred obligations; it is
not a complete inventory of the old assertion set. Additional legacy checks
still need inventory and mapping to common black-box cases or separate
mandatory API/ownership assertions.
