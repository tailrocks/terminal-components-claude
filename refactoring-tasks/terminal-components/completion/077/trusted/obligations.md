# TASK-077 native oracle namespace obligations

Every native oracle check invokes the tracked dispatcher bundle
`tools/refactor-proof/bin/tc-proof`, but the gate rejects the catalog's
`components` namespace. TASK-006's contract requires `bin/tc-proof
oracle --namespace components` (verify.toml CHK-003, in catalog since
`51dd9559`); prepare correctly stamps family=native, but the runner
rejects `PROTOCOL` at `tools/refactor-proof/runner/context.py:40`,
where `NATIVE_ORACLE_NAMESPACES` holds only `{showcase, holla, jackin,
tablepro}` (introduced by `b96a62e4` for app namespaces only). The
CHK-007 close cascades (TASK-006 verify r2: FAIL 9/11).
Coordinator-confirmed: `validate_native_extension` plus all downstream
oracle flow is namespace-agnostic, and Rust-side namespace hits are
`#[test]`-only. The ONLY production gap is the Python allowlist. Since
checks invoke the generated bundle, the fix regenerates
`tools/refactor-proof/bin/tc-proof` via the qualified generator
`tools/refactor-proof/architecture/rebundle.py`.

The work product is exactly two files: one added set member in
`tools/refactor-proof/runner/context.py`, and the tracked bundle
regenerated from it. All section details live in the qualified generator
(authoritative) and the authoritative freshness check
`tools/refactor-proof/tests/rebundle_check.py`.

## O-001 — Allowlist carries components in source and bundle (CHK-001, R-001, AC-001)

Demand: the TASK-006 `CHK-003` oracle branch. Today
`NATIVE_ORACLE_NAMESPACES` in both the runner source and the tracked
bundle holds only the four app namespaces, so `components` with a native
family is rejected.

Obligation: the trusted `allowlist-probe.py` exits 0 on the candidate
tree: `NATIVE_ORACLE_NAMESPACES` extracted by AST from
`tools/refactor-proof/runner/context.py` equals exactly `{showcase,
holla, jackin, tablepro, components}` (no fewer, no more), the same
exact set is extracted by AST from the tracked bundle, and the
authoritative freshness check
`tools/refactor-proof/tests/rebundle_check.py` exits 0 (every bundle
section byte-matches its stripped current source).

## O-002 — Bundle gate accepts components, still rejects the rest (CHK-002, R-002, AC-002)

Demand: the runtime gate `validate_oracle_namespace` reached by the
TASK-006 `CHK-003` oracle branch through the bundle. Today the bundled
gate raises `Reject("PROTOCOL")` for `components` with a native family.

Obligation: the trusted `gate-behavior-probe.py` exits 0 on the
candidate tree. The probe loads the tracked bundle as a module in its
own disposable process (never the repo sources) and asserts exactly:
`validate_oracle_namespace('components', 'native')` returns without
raising; each of `showcase`, `holla`, `jackin`, `tablepro` with
`'native'` returns without raising; `'bogus'`, `''`, and `None` with
`'native'` each raise `Reject` with category exactly `PROTOCOL`;
`('synthetic', 'synthetic')` returns without raising; and `'components'`
and `'bogus'` with `'synthetic'` each raise `Reject` with category
exactly `PROTOCOL` (the synthetic family still requires exactly
`'synthetic'`). The bundled `NATIVE_ORACLE_NAMESPACES` attribute equals
the exact five-member set.

## O-003 — Regeneration gate (CHK-003, R-003, AC-003)

Demand: the regenerated bundle must be the genuine deterministic
product of the qualified generator run against the fixed source,
runnable in place. A fresh-but-unfixed bundle must not pass this gate.

Obligation: the trusted `regeneration-probe.py` exits 0 on the
candidate tree: the tracked bundle is a regular file with user, group,
and other exec bits set, it parses (`ast.parse`) and compiles, running
the qualified `architecture/rebundle.py` generator with its output
redirected to a temp file produces byte-identical output to the tracked
bundle, and the staged generator product's `NATIVE_ORACLE_NAMESPACES`
(extracted by AST) equals exactly `{showcase, holla, jackin, tablepro,
components}`.

## O-004 — Single-member generator-only prohibitions (all checks, R-004)

Add ONLY the `'components'` member to `NATIVE_ORACLE_NAMESPACES`: no
other allowlist change, no validation weakening (no broadened family
rule, no swallowed rejection, no permissive flag), and no source change
beyond that one set member in
`tools/refactor-proof/runner/context.py`. Regenerate the bundle ONLY by
running the qualified generator
`tools/refactor-proof/architecture/rebundle.py` (for example `python3
tools/refactor-proof/architecture/rebundle.py` from the worktree root).
Forbidden: any hand edit to the tracked bundle; any change to any other
proof source (every other `runner/` file, `src/` including
`src/verifier.rs`, `accounting/`, `architecture/`, `tests/`,
`scripts/`, `bin/tc-proof-host`) or to the proof adapters
(`tools/refactor-proof-adapters/**` — the fix is the harness gate, not
adapter edits) or to any other repository file; any
`refactoring-tasks/**` change (in particular no probe, obligation, or
manifest edits inside this or any other task package); any oracle
contact (no baseline import, capture, approval, or blessing — this task
neither reads nor writes oracle data); and any hardcoding of bundle or
probe bytes to satisfy the judges. The generator output is
deterministic: a correct regeneration needs no follow-up touch-up.
Taskfmt scope + forbidden-path gates enforce this; the reviewer
confirms the source diff is the one set member and the bundle diff
equals the generator product.
