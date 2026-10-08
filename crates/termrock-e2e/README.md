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
pin both real subjects, write a `termrock-spec/parity-subject-manifest-v2`
JSON file, and pass its path in `TERMROCK_E2E_SUBJECT_MANIFEST`. The manifest
contains a `subjects` array with one reference and one candidate record. Each
subject requires build inputs and a bound
`termrock-spec/parity-subject-build-receipt-v1` payload. The suite checks that
the receipt binds source, package, target, feature/default-feature policy,
toolchain, build inputs, argv, and executable hash. The v1 subject, trust, and
run-receipt schemas remain historical; the v2 reader rejects v1 manifests
instead of converting them implicitly.

The v2 manifest may set `expected_generation` to `null` while a shared oracle
is unavailable. This allows a real paired pilot to run the declared inputs and
assertions and export captures. A null generation does not authorize visual
comparison: every visual checkpoint remains `BLOCKED`, so the ignored Holla
acceptance gate exits unsuccessfully after recording the run.

The manifest's `build_evidence` contains only references to the fixed relative
basenames `source-inputs.json` and `build-environment.json`, each with a raw
file SHA-256. The final `run.json` binds the raw manifest path and digest, plus
the sidecar paths and digests. The v2 loader checks typed sidecar fields and
cross-file source, toolchain, target, and run identities. The external trust
record binds the canonical manifest and final-run paths and raw digests. This
keeps the manifest and final-run evidence links acyclic.

The v2 reader requires oracle lineage to identify `refs/tags/visual-baseline`,
tag object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, and peeled commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. It also requires
`refs/remotes/origin/visual-baseline` with the producer's `verified-reachable`
membership label. The selected reference commit must equal the declared branch
tip and remain cross-bound to the manifest, final run, and external trust record.
The suite does not hardcode a branch tip; the independently sourced trust record
pins the accepted subject pair. Both qualification status fields must remain
`blocked`, with their nonempty reasons preserved independently. The frozen v1
builder can select an older reachable ancestor, which the v2 reader rejects;
the separately reviewed v2 builder must fail before building when the selection
does not equal the branch tip.

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

The ignored `holla_write_policy_preflight` test resolves the same policy before
the trust record exists. It requires an absent trust-record leaf under an
existing parent with no symlink components, validates the subject manifest and
build evidence, verifies both executable hashes, and prints a `prepared_only`
record to stdout. It does not load trust, launch products, capture output, or
write files. The prepared record can supply `write_policy_sha256` to an
independent trust-record author, but it is not itself a trust record or the
independent review receipt named by `suite.review_receipt_sha256`. Run only
this exact ignored test for preparation; the separate `holla_help_overlay`
test is the paired acceptance gate.

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

The caller must create each role's actual-output root before preflight. Each
root must be an existing normalized directory with no symlink components, and
the two roots must be physically distinct and non-nested. Roots may contain
completed captures from other cases; only the current case's checkpoint leaves
must be absent. On Unix, the resolver compares canonical paths and device/inode
identity, including ancestor identity, so case-only aliases on a
case-insensitive filesystem fail before launch. Targets without the reviewed
filesystem identity implementation fail closed. The native case-alias probe
reports `UNSUPPORTED` on a case-sensitive volume; that result does not count as
coverage of case-insensitive lookup.

The trust-record file itself is checked against caller-provided protected
roots. Its parent is protected from overlap with the write root, receipt, and
actual-output roots. Against caller-provided source, artifact, and oracle
roots, the resolver uses no-follow physical directory ancestry: a parent equal
to or beneath one of those roots is rejected, while a strict ancestor is
allowed only when the trust-record file itself remains outside the protected
root. A shared ancestor does not prove independent trust governance; callers
needing stronger separation should place the trust record under a distinct
parent.

The caller should build each subject into an isolated read-only artifact root.
The suite checks each executable's hash and file metadata at pair preflight,
again immediately before its PTY spawn, and after its journey. Tuiscotti accepts
an executable path, so a same-user replacement in the interval between the
last check and process open remains a filesystem race; this runner does not
claim to pin an executable file descriptor or provide an OS sandbox.

Each product process starts with an empty inherited environment and the same
profile-owned common variables (`TERM`, `COLORTERM`, `LC_ALL`, and
`SHELL=/bin/sh`) for both roles. Case-specific variables must appear in the
profile allowlist and case record. The process CWD is the canonical suite
checkout for both subjects. The run receipt records these inputs and the
resolved vendored Tuiscotti renderer profile with ordered primary and fallback
font hashes. This identifies the renderer used for capture; it does not provide
the deferred exact cell, cursor, or decoded-pixel comparison.

Set `TERMROCK_E2E_RECEIPT_PATH` to write the machine-readable run receipt.
Actual captures go below each subject's `actual_output_root`. They are written
to sibling staging directories; the manifest is written last and the completed
directory is published atomically. Capture and receipt writes share a
component-wise directory check rooted at `TERMROCK_E2E_WRITE_ROOT`. Capture
parents are created one component at a time; before launch, every existing
checkpoint-path component is inspected without following symlinks and the exact
checkpoint leaf must be absent. Publication repeats the component checks. A
symlink below the write root fails the write.
Failed captures are retained in quarantine. Actual output roots must be
disjoint from each other and every protected root. These path-based checks do
not prevent a concurrent same-user replacement between inspection and
filesystem operations; this runner does not claim race-free directory-handle
confinement.
A matching expected tree hash reports `HASH_MATCH`; it reports `ADMITTED` only
when the external trust record binds an admission receipt. With
`expected_generation: null`, the receipt records that no shared generation was
provided. Exact visual comparison remains blocked in this pilot, and the runner
never writes expected data.

## Current stop condition

An earlier v1 package check used Cargo 1.98.1 and cargo-nextest 0.9.146 with
`--locked --jobs 2`: 21 tests passed and the Holla PTY journey remained ignored.
That historical check does not verify the v2 schema, evidence chain, process
environment, or renderer profile. The current v2 increment still needs its
focused package check and independent review. The pilot is not a complete suite
or visibility acceptance.
`cases/deferred-obligations.json` records 23 known deferred obligations; it is
not a complete inventory of the old assertion set. Additional legacy checks
still need inventory and mapping to common black-box cases or separate
mandatory API/ownership assertions. Candidate API and ownership obligations
remain separate lanes; this PTY receipt does not infer their status from screen
output. They stay in the migration denominator until their own evidence is
registered and reviewed.
