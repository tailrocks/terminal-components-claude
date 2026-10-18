# Independent comparator qualification protocol

This is the planner-owned test contract for the comparison part of B-HARNESS. The executable under test is a future deliverable. These files do not implement the production comparator:

- `proof-comparator-bootstrap.py`: independent Python standard-library black-box driver.
- `proof-comparator-vectors.json`: 72 explicit positive, mutation, aggregation and tamper cases.

Copy both files unchanged into B-HARNESS's external `trusted/proof-bootstrap/` directory. Freeze their SHA-256 values in the initial host authority before dispatch. The task may implement the comparator and its own unit tests, but cannot edit this driver, vectors or expected outcomes.

Existing preparation self-test:

```sh
python3 docs/refactoring-plan/evidence/proof-comparator-bootstrap.py --self-test
```

Optional tool-backed fixture validation, using the verification owner's actual qualified executable:

```sh
python3 docs/refactoring-plan/evidence/proof-comparator-bootstrap.py --self-test --tuisnap /absolute/qualified/tuisnap
```

The self-test constructs every fixture, validates the report contract for every positive/negative outcome, rejects 888 report-schema and aggregation mutations, rejects nine deliberately lying/incomplete runners, and optionally roundtrips the positive frame through tuisnap's existing `render --format json` command. The aggregation probes reject later omissions, duplicate identity substitutions and unrequested checkpoints even when reported counters match the incomplete result array. Explicit guards remain active with `python3 -O`; no qualification decision uses Python `assert`. This is not a passing production-harness qualification.

Future B-HARNESS acceptance command:

```sh
python3 /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py --runner /absolute/candidate/tc-proof
```

The driver invokes that regular executable as `tc-proof compare --context ABSOLUTE_CONTEXT_JSON`. All 72 cases must pass their declared outcome, followed by a fresh valid case after each of the 69 negative cases: 141 successful invocations of the qualification process. Each recovery retains the negative case's required-set cardinality. A successful negative case means the comparator correctly rejected its input, not that input matched. Run IDs and fixture paths are opaque random values, case order is shuffled, and inert binary/tool/adapter fixture contents receive fresh salt. Mutation names and expected verdicts are not supplied to the comparator. No shell command, candidate import, candidate test report or stdout marker supplies the driver verdict.

## Input context

`tc-proof-compare-context/v1` is the normal comparator input schema. The trusted host creates it; an application process never chooses it. Other project operations may use their own exact context schemas. The comparison fixture uses:

| Field | Meaning |
| --- | --- |
| `schema` | Exactly `tc-proof-compare-context/v1`. |
| `run_id`, `task_id` | Host-selected identities; outputs must echo them. |
| `oracle_commit` | Exactly `02f5294bfdbf38004cc49130d0aff1d01f31434c`. |
| `candidate_source_tree` | Host-selected frozen candidate tree. Different from oracle is valid. |
| `oracle_root`, `candidate_root` | Absolute input directories selected by the host. |
| `oracle_manifest_sha256`, `candidate_manifest_sha256` | SHA-256 over each root's exact `manifest.json` bytes. |
| `required_sha256`, `actions_sha256` | Protected oracle membership and ordered action payload hashes. |
| `required_ids`, `required_count` | Protected complete ordered membership and its length; available even if `required.json` is corrupt. |
| `tool_sha256` | Accepted capture tool identity. |
| `oracle_adapter_sha256`, `candidate_adapter_sha256` | Separately accepted adapter identities. |
| `report_path` | Host-selected output file outside both input directories. |

All hashes have lowercase hexadecimal encoding. SHA-256 uses 64 characters; Git tree/commit fixture IDs use 40. JSON canonicalization is UTF-8, sorted object keys, no optional spaces, no ASCII escaping and no trailing newline. Artifact manifest hashes are byte hashes, not hashes over reparsed pretty-printed JSON. Context SHA-256 binds its exact bytes, including absolute host paths. Context is run-specific evidence, not a portable canonical oracle payload.

The positive fixture intentionally uses a synthetic candidate tree and inert `binary.bin` contents. The comparator verifies their bindings without executing them. Actual Git ancestry, build provenance, executable rebuilding and adapter source review belong to host/capture qualification, not this isolated comparator test. The fixture font is an integrity payload, not a raster font. No PNG is declared, so this fixture does not request rendering or qualify PNG equality.

## Manifests and observations

Each root's `manifest.json` uses `tc-proof-artifacts/v1` with `scenario_ids` and sorted `files`. Each file entry has `path`, `size` and `sha256`. The manifest excludes itself. Validate safe relative paths and reject symlinks before reading content. Validate exact file membership, sizes and hashes. Reject duplicate file paths, duplicate JSON keys and duplicate scenario IDs. Extra candidate checkpoint artifacts cannot enlarge the required set. A candidate `approved/` artifact is always forbidden, even if included in its manifest.

Oracle `required.json` uses `tc-proof-required/v1`. Its `scenarios` array contains exact `id`, `frame`, `state`, `provenance`, `state_keys`, `lane` and `checkpoint`. IDs identify a complete scenario/lane/axes/checkpoint combination. Single-member fixtures isolate individual fields. Three-member fixtures contain pressed, released and settled checkpoints from one scenario; four-member fixtures add a separate editor scenario. The full project adds these identities through accepted required-set expansion, not candidate file discovery. Each entry's state keys and nested field types come from its accepted semantic observation schema. Missing/unknown semantic keys fail; expected and candidate semantic values must match exactly.

The multi-member corpus includes all-matching sets, first/middle/final mismatches, two failures with a passing checkpoint between them, a malformed final frame, a missing final artifact and a dropped final required identity. A comparison must process later entries after a scenario mismatch. Whole-bundle structural validation runs before entry comparisons, so integrity/membership rejection produces a global failure without partial comparison results.

Frames use real tuisnap schema 3. Both must validate before comparing. Exact cell/cursor equality excludes frame provenance, which is checked separately. Frame `provenance.source` must equal that lane's host-selected source identity. Legitimate source identity and `created_unix` differences do not make equal UX fail.

Each sidecar uses `tc-proof-provenance/v1`, with `scenario_id`, `source_tree`, `binary_path`, `binary_sha256`, `actions_sha256`, `tool_sha256`, `adapter_sha256`, `lane` and `checkpoint`. Candidate sidecars additionally require `run_id` and `task_id` equal to the current host comparison context; copied evidence from another run/task fails. Oracle sidecars retain their original baseline provenance, whose receipt is separately verified by the host. Verify all shared fields against context, recipe membership and actual binary payload hash. The oracle's source identity must be the oracle commit; the candidate's must be its frozen tree. This test protocol uses `source_tree` for both source identity forms; the full capture receipt distinguishes the oracle commit and resolved Git tree separately.

The driver rehashes candidate manifests after visual/semantic mutations. Therefore a comparator that only checks artifact hashes cannot pass those cases. It keeps protected oracle hashes fixed for corruption cases. The wrong-oracle-source case is resealed deliberately so source authority must also be checked beyond byte integrity.

## Results and failure classes

Every invocation, including input rejection, writes one regular JSON file at `report_path` using `tc-proof-comparison/v1`. Its exact top-level fields are `schema`, `run_id`, `task_id`, `context_sha256`, `required_count`, `checked_count`, `passed_count`, `results` and `failures`. Unknown or missing fields fail. Identity/hash fields are strings; counts are nonnegative integers, never booleans; `results` and `failures` are arrays. Counts cannot exceed the protected context's `required_count`.

Let `N` be the protected context's `required_count`, which must equal the length of its nonempty, duplicate-free ordered `required_ids`. A valid comparison requires exit 0, `required_count = checked_count = passed_count = N`, no failures and one passing result for every required identity in that exact order. A reported count cannot substitute for those exact identities.

Global validation failures (`UNSAFE_PATH`, `UNEXPECTED_APPROVAL`, `INTEGRITY`, `REQUIRED_SET`) require nonzero exit, `required_count = N`, `checked_count = passed_count = 0`, `results = []`, and exactly the declared global failure `{"id": null, "code": EXPECTED_CODE}`. Scenario failures (`PROVENANCE`, `FRAME_INVALID`, `FRAME_MISMATCH`, `STATE_INVALID`, `STATE_MISMATCH`) require nonzero exit and complete execution: `checked_count = N`, `passed_count = N - F`, and exactly one result for every required identity, where `F` is the number of failed required identities. Passing members retain `status: "passed"`; each failing member has `status: "failed"`. The failure array must contain exactly the independently expected identity/category pairs in required identity order. Neither a missing final checkpoint nor a duplicate first checkpoint can satisfy membership. Failure records may additionally contain a string `detail`; no other fields are allowed. Result records contain exactly `id` and `status`.

For example, a three-checkpoint set with first and final failures requires `required_count = checked_count = 3`, `passed_count = 1`, statuses `[failed, passed, failed]`, and both exact failure identities/categories. A first-checkpoint failure does not authorize early termination before the remaining checkpoints.

The input context includes the protected IDs and count independently of `required.json`, so malformed membership cannot erase the required count from the failure report. The comparator verifies `required.json` matches that authority once artifact integrity succeeds. An unreadable artifact is never an empty successful set.

Failure codes required by the fixture are:

- `UNSAFE_PATH`: traversal or symlink before artifact reads.
- `UNEXPECTED_APPROVAL`: candidate supplies an approval artifact.
- `INTEGRITY`: missing payload or byte/hash/size mismatch.
- `REQUIRED_SET`: extra/duplicate scenario/checkpoint membership.
- `PROVENANCE`: source/binary/action/tool binding mismatch despite valid artifact hashes.
- `FRAME_INVALID`: invalid schema, dimensions, cell count, position or wide continuation.
- `FRAME_MISMATCH`: valid frame with different cells/cursor.
- `STATE_INVALID`: missing/unknown semantic schema fields.
- `STATE_MISMATCH`: valid semantic state with different values.

Path safety and unexpected approvals must be diagnosed before a generic membership error. The fixture's mutations are otherwise isolated so the expected category is unambiguous. Preserve details and artifacts for diagnosis; neither a stdout `DONE` line nor missing JSON counts as a result.

The qualification report records executable, driver and vector hashes, case/invocation counts and every failed expectation. The driver checks oracle/context and executable bytes before/after each invocation. Host isolation qualification separately proves that a candidate cannot temporarily modify and restore trusted bytes; this driver does not claim to observe all filesystem writes. These public finite tests are not a proof against an arbitrary adversarial program specialized to their contents. Opaque challenges remove trivial case-label/hash-table verdict lookup; independent source review and host-owned qualification remain mandatory.

## Coverage boundary

This suite qualifies exact comparator behavior and artifact/provenance rejection. It does not qualify production input decoding, PTY fidelity, source-only capture, driver equivalence, complete application inventory, source freeze, ref updates or host isolation. Those retain independent named preparation owners in [proof-contract.md](../proof-contract.md). Passing this suite cannot authorize oracle sealing or component implementation by itself.
