# `termrock-e2e`

`termrock-e2e` is a product-independent PTY harness. It depends on the pinned
Tuiscotti PTY toolkit and has no Termrock, Ratatui, or application-crate
dependency. Its manifest declares a separate Cargo workspace, so this package
has its own resolver and lockfile. The source and schema links below describe
the implementation reviewed at PR38 commit
[`7b2eb51c925a105f210c853e81b8ad440dad210f`](https://github.com/tailrocks/terminal-components-claude/tree/7b2eb51c925a105f210c853e81b8ad440dad210f);
those E2E implementation files are unchanged in the documentation-only commits
on the checkout used to draft this page. [The manifest](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/Cargo.toml#L1-L29)
records edition 2024, Rust 1.99, the standalone workspace, and Tuiscotti Git
revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

This is a source guide, not a run receipt. No command, binary, test, preflight,
capture, trust record, or admission flow was executed while drafting it. The
package's current repository policy requires the guarded Cargo/Nextest path;
this page intentionally omits stale raw-Cargo and `--manifest-path` examples.
Use the package's own workspace and the governing repository instructions for
any separately authorized execution. The Cargo Git dependency revision, the
profile's `tool_revision`, and the renderer identity are distinct recorded
values, not interchangeable version pins; see the
[manifest](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/Cargo.toml#L15-L20),
[profile](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/profile.json#L1-L18), and
[renderer identity](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7999-L8025).

## Current evidence state

The registry contract test declares four seed cases, but that source assertion
is not evidence that any journey ran. All four registered real-binary journey
tests are ignored, as are the Holla paired-acceptance, write-policy preflight,
immutable-tag capture, and tag-preflight tests. The profile keeps visual
acceptance blocked pending an independently qualified oracle and exact
comparison. The deferred inventory contains 23 `NOT_RUN` rows, 25 case
references, and the missing historical ID `BD-19`. These are source-level
statuses, not results from this README review: [registry contract](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L17-L39),
[ignored paired journeys](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/shared_journeys.rs#L28-L47),
[Holla preflight and acceptance declarations](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/holla_help_overlay.rs#L6-L54),
[tag-capture declaration](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/holla_tag_capture.rs#L3-L52),
[profile gate](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/profile.json#L19-L30), and
[deferred obligations](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/cases/deferred-obligations.json#L1-L33).

## Contract map

The numbered entries retain the operational distinctions from the earlier
guide while linking to the current implementation and schema. The linked tests
describe contract coverage; their presence does not mean they have been run.

1. **Closed typed input programs.** A case is a closed record. Its step enum
   accepts `press`, `key_event` (`down`, `repeat`, `up`), `text`, `paste`,
   `mouse`, `resize`, `expect_exit`, and `checkpoint`. Mouse input separately
   models click, down, generic release, move, drag, and wheel; buttons are left,
   middle, and right, wheel directions are up/down/left/right, and modifiers
   are shift/alt/ctrl. `text` sends literal UTF-8 bytes; `paste` is a distinct
   negotiated call that uses bracketed-paste markers when enabled and rejects
   delimiter-containing content. Generic mouse release carries no button
   identity, even though the toolkit method accepts a button argument.
   Unknown fields and variants are rejected. See
   [step types](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L160-L300),
   [Tuiscotti text and paste calls](https://github.com/tailrocks/tuiscotti/blob/a1ea27f757d75d07b05499276cd932e9f8e067ad/crates/tuiscotti-runtime/src/tui/session_input.rs#L18-L49),
   [buttonless release](https://github.com/tailrocks/tuiscotti/blob/a1ea27f757d75d07b05499276cd932e9f8e067ad/crates/tuiscotti-runtime/src/tui/session_input.rs#L220-L231), and the
   [deserialize/bounds contract](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L103-L136).

2. **Geometry and step order.** Geometry is checked against Tuiscotti's limits;
   mouse coordinates are zero-based and must fit the effective geometry at the
   moment of the event. A resize changes the dimensions used by subsequent
   events and checkpoints. Checkpoint geometry is reconstructed in sequence.
   The pinned Tuiscotti revision defines supported columns and rows as
   `1..=1000`; these are revision-specific limits, not a timeless toolkit API
   promise. At most one `expect_exit` is allowed; it must be last and follow a
   checkpoint, and records the direct child status. See
   [case validation and geometry](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3623-L3895) and
   [pinned Tuiscotti geometry constants](https://github.com/tailrocks/tuiscotti/blob/a1ea27f757d75d07b05499276cd932e9f8e067ad/crates/tuiscotti-runtime/src/tui/limits.rs#L16-L23) and
   [input order tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L139-L335).

3. **Input success is not application effect.** A successful Tuiscotti call
   records that the input API accepted the call; it does not establish that the
   application acted on it. For example, key-up may be ineffective when Kitty
   keyboard mode is inactive, and mouse behavior depends on compatible DEC
   mouse reporting. The runner does not inspect or restore the caller's parent
   terminal state. See [input result handling and event dispatch](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6774-L7257)
   and [checkpoint exit/restoration rows](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7030-L7156).

4. **Subject and build provenance.** The current subject manifest is v2 and
   requires one reference and one candidate subject. Each binds a source
   commit, package ID, target name, requested/default features, target triple,
   toolchain/profile, manifest and lock digests, mise configuration,
   Cargo/rustc/host identities, executed argv, and executable path/hash. Its
   manifest path is supplied to the paired reader by
   `TERMROCK_E2E_SUBJECT_MANIFEST`.
   The build-evidence object references typed `source-inputs.json` and
   `build-environment.json` sidecars by fixed relative basename and raw
   SHA-256; the builder-run receipt binds the resulting build. See
   [manifest/build types](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L324-L437),
   [manifest input and evidence loading](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L1380-L1497),
   [chain validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L1846-L2518),
   [v2 schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/subject-manifest-v2.schema.json), and
   [manifest contract tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L596-L711).

5. **Schema versions are per record.** The active run receipt is
   `parity-run-receipt-v3`; the v1/v2 receipt schemas and v2 identifier are
   retained historical artifacts, not a supported current validation route.
   The active run receipt and current visual-receipt verification require v3.
   The subject manifest is v2, builder receipt v1,
   trust record v2, write policy v2, expected generation v1, immutable-tag
   capture v1, and generation admission and qualification review each have
   their own v1 schema. Do not describe these independent records as one
   “v2” or “v3” format. The current reader rejects the historical v1 subject
   manifest rather than converting it. See
   [schema identifiers](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L21-L55),
   [historical receipt v1 schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/receipt-v1.schema.json),
   [receipt schemas](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/receipt-v2.schema.json) and
   [v3 receipt schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/receipt-v3.schema.json),
   [historical subject-manifest v1 schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/subject-manifest-v1.schema.json),
   [subject schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/subject-manifest-v2.schema.json),
   [trust schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/trust-record-v2.schema.json), and
   [v3 visual-receipt requirement](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7392-L7394), and
   [version-separation tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L596-L711).

6. **A null generation does not admit comparison.** `expected_generation` is a
   required field which may explicitly be `null`. Null allows the bounded
   paired pilot to run its declared interaction inputs and export actual
   captures, but there is no shared expected tree: visual checks remain
   `BLOCKED`. A hash-matching tree is still only `HASH_MATCH` until a separate
   admission is verified. See
   [required nullable manifest field](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L324-L335),
   [comparison gate](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7297-L7350),
   [v3 visual verdict contract](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L880-L973), and
   [profile status](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/profile.json#L19-L30).

7. **Tag and reference identity are separate pins.** The immutable oracle
   identity names `refs/tags/visual-baseline`, tag object
   `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, and peeled commit
   `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Its accepted branch reference
   is `refs/remotes/origin/visual-baseline` with membership label
   `verified-reachable`. Source evidence must declare the selected reference
   commit equal to the accepted branch OID; the membership label alone is not
   sufficient. The suite binds those identities and the selected subject pair;
   it does not make a moving branch tip a permanent suite constant. Builder
   and source qualification statuses must both remain `blocked`, and each
   must have a nonempty reason until external admission.
   See [identity constants](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L21-L30),
   [tag identity/binding validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L4463-L4666),
   [branch-tip equality and blocked qualification](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2001-L2038),
   [tag capture receipt validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L5291-L5400),
   [tag contract assertions](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/holla_help_overlay.rs#L39-L65), and
   [tag identity schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/oracle-capture-receipt-v1.schema.json).

8. **Trust is an independent external input.** A caller supplies an absolute
   trust-record path and its raw SHA-256 through
   `TERMROCK_E2E_TRUST_RECORD` and `TERMROCK_E2E_TRUST_RECORD_SHA256`. The file
   must be outside the suite, both subject source/artifact roots, oracle and
   expected-generation roots, and both actual-output roots. The record binds
   suite/case/profile/lock/review/test-binary digests, each subject's
   source and builder receipt, optional generation/admission, and the resolved
   write-policy digest. The runner verifies the bytes and bindings before
   launching either paired subject. Parsing a record and checking its digest
   do not authenticate who authored it; the caller must obtain both from an
   independent source. The trust-record parent is rejected when it is equal
   to or below a protected source, artifact, or oracle root. A strict common
   ancestor may be allowed only while the record itself remains outside
   protected roots; this path relation does not prove independent governance.
   See
   [trust input and validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2519-L2571),
   [trust-parent relation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L1756-L1784),
   [record/protected-root overlap checks](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2904-L2947),
   [record checks](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3266-L3520),
   [v2 trust schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/trust-record-v2.schema.json), and
   [trust contract](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L1046-L1075).

9. **Preparation is not trust or acceptance.** The ignored
   `holla_write_policy_preflight` resolves a prospective policy before the
   trust record exists. It requires an absent trust-record leaf beneath an
   existing no-symlink parent, validates subject/build evidence, and checks
   executable hashes. It returns a `prepared_only` record; the test prints it
   to stdout. It does not write a trust record, launch a product, capture
   output, or produce an independent review receipt. That digest can inform an
   independent trust author, but the separate ignored `holla_help_overlay`
   test is the paired-acceptance route. See
   [preflight function](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3897-L4055) and
   [test declarations](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/holla_help_overlay.rs#L6-L37).

10. **The write boundary is explicit and hashed.** The caller provides an
    existing absolute `TERMROCK_E2E_WRITE_ROOT`, an absent
    `TERMROCK_E2E_RECEIPT_PATH` strictly beneath it, and
    `TERMROCK_E2E_PROTECTED_ROOTS` as JSON. Each caller protected-root entry
    has `kind`, `role`, and `path`: every subject role requires
    `subject_source` and `subject_artifact`, while an `oracle` entry has
    no subject role. The runner adds the suite, expected-generation root, trust
    record and parent, test-binary parent, and executable parents. The write
    root must not overlap protected roots. The receipt must be new, strictly
    beneath it, and disjoint from protected roots and actual-output roots.
    Each actual-output root must already exist, be strictly beneath the write
    root, and be disjoint from protected roots; output roots must also be
    mutually distinct and non-nested. The policy SHA-256 covers the
    implementation's compact JSON payload with its defined fields
    (`schema`, `write_root`, `receipt_path`, `subject_cwd`,
    `protected_roots`, `actual_output_roots`); field ordering and
    normalization belong to that implementation, not arbitrary presentation
    JSON. The external trust record must bind that digest. See
    [policy records](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L797-L866),
    [policy input/resolution](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2584-L2640) and
    [required caller root roles](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3060-L3117),
    [resolved root and receipt/output disjointness](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2897-L3024),
    [policy digest construction](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3252-L3264), and
    [resolved policy validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3040-L3265), plus
    [write-policy schema test](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L800-L879).

11. **Filesystem identity is checked conservatively.** Paths must be absolute
    and normalized; checked input and output paths reject symlink components.
    Output roots must be existing directories, physically distinct, and
    non-nested. On Unix, the resolver uses canonical paths plus device/inode
    identity for directories and ancestors; targets without the required
    physical-identity implementation fail closed. Case-only alias probes may
    report `UNSUPPORTED` on a case-sensitive volume; that is not evidence that
    case-insensitive lookup was tested. See
    [path resolution and overlap](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L1498-L1834),
    [platform-specific identity](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L1650-L1671),
    [output-root validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L3521-L3621), and
    [native alias probes](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L9340-L9507).

12. **Receipt and capture publication are staged, but races remain.** The
    receipt destination must be new; receipt publication uses an exclusive
    staged file and a no-replacement link. Captures are written in sibling
    staging directories with the manifest last, then published as a completed
    directory. Every capture leaf for the current case's checkpoints must be
    absent before launch; completed captures for other cases may coexist.
    Existing checkpoint leaves and symlink components are checked again
    during publication. Failed captures are retained in quarantine; a
    preflight failure may leave no receipt, which is incomplete evidence.
    These path-based checks do not prevent a concurrent
    same-user filesystem replacement between inspection and use. See
    [capture construction](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8270-L8394),
    [leaf checks](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8424-L8505),
    [atomic capture publication](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8507-L8760),
    [receipt publication](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8794-L8840), and
    [quarantine handling](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8734-L8759).

13. **Executable checks do not pin process-open identity.** Subject executable
    bytes and file metadata are checked during pair preflight, immediately
    before spawn, and after the journey. This detects persistent digest or
    identity changes, but Tuiscotti launches by path: a same-user replacement
    between the last check and process open is still a race. The PTY runner is
    not an operating-system sandbox. Read-only isolated subject artifact roots
    are a caller-side practice, not a suite guarantee. See
    [executable verification](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7718-L7819),
    [launch](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7851-L7896), and
    [build validation tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L596-L711).

14. **Child environment and renderer identity are recorded.** A subject starts
    with cleared inherited environment and profile-owned `TERM`, `COLORTERM`,
    `LC_ALL`, and `SHELL=/bin/sh`; per-case variables must be allowlisted. Its
    working directory is the canonical suite checkout. The renderer receipt
    includes the resolved renderer/profile identity and ordered font/fallback
    hashes. That binds the capture renderer; it is not proof of exact cell,
    cursor, or decoded-pixel equality with an oracle. See
    [profile](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/profile.json#L1-L30),
    [environment construction and validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7851-L7997),
    [renderer identity](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7999-L8025), and
    [capture observations](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L8308-L8325).

15. **`HASH_MATCH` is not `ADMITTED`.** The expected-tree digest reports
    `HASH_MATCH` when bytes match; only a readable, independently admitted
    receipt bound through trust can produce `ADMITTED`. Visual comparison also
    checks the exact frame/cursor data, renderer binding, strict fidelity, and
    decoded opaque RGB pixels. A trust digest alone cannot enable comparison
    if its admission receipt or bound inputs are absent or stale. See
    [state distinction](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7841-L7849),
    [comparison gate](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L7297-L7350),
    [expected-tree and frame verification](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/visual.rs#L369-L710),
    [admission binding](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/admission.rs#L192-L271), and
    [visual verdict tests](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L880-L973).

16. **Immutable-tag capture is actual-only evidence.** The ignored
    `captures_holla_from_pinned_immutable_tag` route uses the Holla case driver
    and emits its own capture receipt. If complete, the current source expects
    ten artifact formats at each of four checkpoints and twelve interaction
    assertions. The receipt remains `execution_anchor_status: unverified`,
    `qualification: blocked`, and `admission_status: NOT_RUN`; it does not
    write an expected generation, admission receipt, or paired trust record.
    These are declared assertions in an ignored test, not a captured result.
    See [tag capture implementation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L5108-L5290),
    [receipt schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/oracle-capture-receipt-v1.schema.json), and
    [ignored capture assertions](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/holla_tag_capture.rs#L3-L52).

17. **The tag validator has its own pinned toolchain boundary.** A caller
    supplies an absolute toolchain-record path and raw SHA-256 through
    `TERMROCK_VIS06_VALIDATOR_TOOLCHAIN` and
    `TERMROCK_VIS06_VALIDATOR_TOOLCHAIN_SHA256`. That record pins absolute
    Python and Git paths, their raw hashes, and exact version output. The
    validator helper source is
    hashed and transported to the pinned Python process over stdin; its Git
    query is dispatched to the absolute pinned Git path without inheriting
    caller `PATH`. Each tool identity binds the exact `--version` output as
    well as its path and raw SHA-256; Python/Git runtime identities and Git
    metadata paths/identities are checked before and after validator
    execution. Capture reload revalidates the source's Git paths and path/blob
    map. Admission
    protects the captured Git directory and common-directory identities, the
    worktree `.git` metadata entry when present, and the identified
    validator/toolchain/runtime files and their parents, rather than excluding
    the entire validator checkout. Persistent
    executable replacement is detected, but a transient same-user replacement
    race remains. See
    [toolchain schema](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/tag-validator-toolchain-v1.schema.json),
    [version and executable checks](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L4111-L4202),
    [pre/post-process identity checks](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L4220-L4281),
    [captured Git metadata paths and validator files](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L4823-L4919),
    [source/Git-path validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L4463-L4990), and
    [source revalidation on capture reload](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L5997-L6057).

18. **Generation admission is a separate, non-generating write.** The expected
    generation is a prebuilt input. Its tree is exactly one manifest plus a
    frame JSON and screen PNG for each case checkpoint; the current Holla case
    has four checkpoints, hence nine files. The admission flow validates that
    tree and its renderer/provenance, then requires the corresponding frame
    and PNG inputs to match the immutable-tag capture. It writes only a new
    admission receipt. Admission requires a raw-hash-pinned independent
    qualification review with `decision: approve`,
    `execution_anchor_status: qualified`, and at least one hash-pinned
    execution-anchor evidence file. The review binds the tag, builder, capture,
    generation, suite, profile, lock, case, checkpoint sequence, and renderer;
    it does not edit the trust record. Ordinary verification accepts
    `TERMROCK_E2E_GENERATION_ADMISSION` only together with the absolute path's
    raw SHA-256 in `TERMROCK_E2E_GENERATION_ADMISSION_SHA256`; a lone or
    malformed counterpart is rejected, and that digest must match the
    admission receipt hash bound by external trust. The
    `admit_expected_generation` binary requires exactly these nine flags:
    `--generation-id`, `--generation-root`, `--generation-sha256`,
    `--capture-receipt`, `--capture-sha256`, `--qualification-review`,
    `--qualification-review-sha256`, `--write-root`, and `--output`.
    These are interface names, not an execution recipe or authorization.
    Canonical compact JSON and duplicate-key rejection apply. On ordinary
    verification the admission, review, tag capture, all 40 capture artifact
    bytes, build/source evidence, and current expected tree are reloaded and
    rechecked. The new `--output` receipt must be beneath its explicit
    `--write-root`; both paths must stay disjoint from the suite source,
    generation, capture roots, captured
    Git metadata, and the validator/evidence files and relevant parents. This
    targeted protection does not reserve the whole validator checkout.
    See
    [generation tree rules](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/visual.rs#L369-L655),
    [admission bindings](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/admission.rs#L12-L84),
    [review/admission validators](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/admission.rs#L100-L271),
    [admission entry point](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6212-L6379),
    [admission binding construction](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6070-L6211),
    [ordinary capture reload](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L5797-L6070),
    [admission environment variable names](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L43-L44),
    [paired path/hash validation](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L2544-L2571),
    [ordinary run trust/admission binding](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6451-L6498),
    [admission CLI flag parser](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/bin/admit_expected_generation.rs#L13-L57),
    [admission protected inputs](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6291-L6341),
    [admission output disjointness](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L6191-L6209),
    [schemas](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/expected-generation-v1.schema.json),
    [generation admission](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/generation-admission-v1.schema.json), and
    [qualification review](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/qualification-review-v1.schema.json).
    The expected-generation contract test checks the checkpoint files and
    dimensions at the current case geometry
    ([contract test](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L975-L1044)).

19. **Deferred behavior remains separate from these PTY contracts.** The
    deferred inventory explicitly keeps every listed obligation at `NOT_RUN`,
    retains 23 rows/25 case references, and names `BD-19` as a missing
    historical identifier. A registry entry, assertion name, source test, or
    successful input API call does not establish component/API/design
    acceptance. Keep the deferred file and profile as the authority for their
    current source-level status; do not infer readiness or fill the missing ID
    by guesswork. See [deferred rows](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/cases/deferred-obligations.json#L1-L33),
    [inventory contract](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/tests/contract.rs#L581-L593), and
    [visual status](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/profile.json#L19-L30).

## Related schemas and tests

The full shapes remain in the versioned JSON Schemas under
[`schemas/`](https://github.com/tailrocks/terminal-components-claude/tree/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/schemas/).
The contract tests pin selected schema behavior, required-null semantics, and
trust/write-policy binding. Source validators also reject duplicate keys and
require canonical compact JSON for trust, capture, admission, and qualification
records ([JSON reader](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/lib.rs#L58-L150),
[admission loader](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/crates/termrock-e2e/src/admission.rs#L100-L245)).
Real-binary journey tests have separate ignored status and prerequisites
described above. None of these links should be read as a test result, an
authorization to author trust, or permission to publish or write expected
visual data.
