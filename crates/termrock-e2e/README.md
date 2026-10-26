# `termrock-e2e`

This package retains a narrow generation-admission API and command-line
interface. It validates a prepared expected generation against a pinned
capture and qualification review, then writes one admission receipt. It does
not run a PTY journey, acquire a capture, qualify an oracle, or establish
product behavior. No behavior lane is verified by this document.

The source identities below refer to the fixed product input
[`1d6ee92`](https://github.com/tailrocks/terminal-components-claude/tree/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd).
The [previous README at that input](https://github.com/tailrocks/terminal-components-claude/blob/1d6ee92adf6ccca38eeee64b1371cba1497e8fcd/crates/termrock-e2e/README.md)
is preserved as historical evidence for the retired PTY, tag-capture,
preflight, deferred-inventory, and test procedures. Those procedures are not
current operating guidance.

## Admission interface

The public function is
[`admit_expected_generation`](src/lib.rs). The retained binary is
[`admit_expected_generation`](src/bin/admit_expected_generation.rs); it accepts
these nine required flags, each followed by one value:

- `--generation-id`
- `--generation-root`
- `--generation-sha256`
- `--capture-receipt`
- `--capture-sha256`
- `--qualification-review`
- `--qualification-review-sha256`
- `--write-root`
- `--output`

The function checks the compiled suite digest, the `HELP-HOLLA-004` registry
entry, the profile and renderer identity, the expected-generation tree, the
immutable-tag capture, and a hash-pinned qualification review. The review must
approve the same generation, capture, suite, case, renderer, and execution
evidence. The inputs are loaded and bound again before publication; a changed
input stops admission.

The command writes one compact JSON result to standard output on success. The
result identifies the admission receipt path and digest, generation ID and
tree digest, and the `admitted` decision. On error, the command writes an
`admit_expected_generation:` diagnostic to standard error and exits with status
2.

The output path must name a new file beneath the existing explicit write root.
The write root must not use symlink components. The output must remain
disjoint from the suite source, generation, capture, validator, and evidence
inputs. Existing output is an error; publication does not replace a file.
These checks are path-based protections, not a guarantee against concurrent
same-user filesystem changes. The function writes only the admission receipt;
it does not create or alter a generation, capture, review, trust record, or
expected visual data. See the [admission implementation](src/lib.rs), the
[binding and review checks](src/admission.rs), and the
[receipt schema](schemas/generation-admission-v1.schema.json).

Admission does not run the qualification review, authenticate its author, or
prove that its evidence is independent. The caller must provide the review
and its raw digest from an independent source. A successful admission receipt
records the approved binding; it is not a product-behavior result.

## Case and profile inputs

The retained admission path reads the suite case registry and profile to bind
`HELP-HOLLA-004`, its renderer, and the suite digest. They remain input data for
admission. The former deferred-obligation inventory and PTY journey registry
are not current behavior inventories. The old README linked above preserves
the source-pinned history.

## Schema status

All twelve schema files remain byte-identical in [`schemas/`](schemas/). Seven
records marked **Active** have retained consumers: the admission path uses
five, and the retained Python subject builder/validator and diagnostic receipt
reader use two. `tools/visibility/subjects.py` builds and validates subject
manifests. `tools/visibility/status.py` reads receipt-v3 records for
diagnostics. The Rust PTY subject-loader lane is retired. “Active” records a
current consumer; it does not mean that a runner still produces that record.
Consumers may hash or embed a schema file, validate a typed record, or check a
version string. The other five schemas are **Historical-only** formats for
retired PTY-runner inputs.

| Status | Schema | Use |
|---|---|---|
| Active | [`expected-generation-v1.schema.json`](schemas/expected-generation-v1.schema.json) | Prepared generation input |
| Active | [`generation-admission-v1.schema.json`](schemas/generation-admission-v1.schema.json) | Admission receipt |
| Active | [`oracle-capture-receipt-v1.schema.json`](schemas/oracle-capture-receipt-v1.schema.json) | Pinned capture input |
| Active | [`qualification-review-v1.schema.json`](schemas/qualification-review-v1.schema.json) | Independent approval input |
| Active | [`tag-validator-toolchain-v1.schema.json`](schemas/tag-validator-toolchain-v1.schema.json) | Validator identity bound by the capture |
| Active | [`receipt-v3.schema.json`](schemas/receipt-v3.schema.json) | Python diagnostic reader for historical receipts; no current runner or producer |
| Active | [`subject-manifest-v2.schema.json`](schemas/subject-manifest-v2.schema.json) | Python subject builder and validator pin the schema bytes and validate its fields |
| Historical-only | [`receipt-v1.schema.json`](schemas/receipt-v1.schema.json) | Retired PTY-run receipt format |
| Historical-only | [`receipt-v2.schema.json`](schemas/receipt-v2.schema.json) | Retired PTY-run receipt format |
| Historical-only | [`subject-manifest-v1.schema.json`](schemas/subject-manifest-v1.schema.json) | Retired subject-manifest format |
| Historical-only | [`trust-record-v1.schema.json`](schemas/trust-record-v1.schema.json) | Retired PTY-run trust input |
| Historical-only | [`trust-record-v2.schema.json`](schemas/trust-record-v2.schema.json) | Retired PTY-run trust input |

No PTY journey, test suite, capture, qualification, visual comparison, or
product-behavior result was run or verified for this documentation update.
