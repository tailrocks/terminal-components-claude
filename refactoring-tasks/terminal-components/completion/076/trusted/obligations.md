# TASK-076 rebundle obligations

Every task check invokes the tracked dispatcher bundle
`tools/refactor-proof/bin/tc-proof`, but that bundle is stale: its
`runner/__main__.py` section still `os.execv`s the native comparator
while the accepted TASK-075 source supervises it as a subprocess and
emits the runner result via `finish()`. So the compare branch writes
`compare.json` but never emits `outputs/<CHK>.result.json`, and close
fails closed (TASK-002 verify r3: 10/11 with only `CHK-007` failing on
the missing result). TASK-075 was forbidden from touching the bundle;
this task is its explicit rebundle follow-up.

The sole work product is the regenerated tracked bundle. All section
details live in the qualified generator
`tools/refactor-proof/architecture/rebundle.py` (authoritative; 24
sections including `architecture/*`, the bundle guard, the
`runner/__main__.py` entrypoint, and the qualification overlay) and the
authoritative freshness check
`tools/refactor-proof/tests/rebundle_check.py`.

## O-001 — Bundle sections match current sources (CHK-001, R-001, AC-001)

Demand: `tools/refactor-proof/tests/rebundle_check.py`. Today the
tracked bundle's runner entrypoint differs from source, so the check
fails with `bundle runner entrypoint differs from source`.

Obligation: `python3 tools/refactor-proof/tests/rebundle_check.py` exits
0 on the candidate tree: the `runner/__main__.py` marker exists, the
stripped source entrypoint is contained in the bundle, the header
carries `stat`, every generated section (runner, accounting,
architecture, entrypoint) byte-matches its stripped current source, and
the bundle parses, compiles, and keeps the runner
`_collect_observations` with its `observer_sequence` keyword.

## O-002 — Bundle compare path supervises and emits (CHK-002, R-002, AC-002)

Demand: the TASK-002 `CHK-004` compare branch and `CHK-007` close
validation. Today the bundle's runner section calls `os.execv`, which
replaces the process with the native comparator: the comparison report
is written but `outputs/<CHK>.result.json` is never emitted, so close
fails closed. The accepted source instead defines `run_compare`, which
runs the comparator as a child process and emits the bound result via
`finish()` before returning to the caller.

Obligation: the trusted `compare-supervision-probe.py` exits 0 on the
candidate tree: no `os.execv` anywhere in the tracked bundle, the
`runner/__main__.py` section carries `run_compare` and emits via
`finish(`, and the bundle AST defines `run_compare`.

## O-003 — Regeneration gate (CHK-003, R-003, AC-003)

Demand: the regenerated bundle must be the genuine deterministic
product of the qualified generator, runnable in place.

Obligation: the trusted `regeneration-probe.py` exits 0 on the
candidate tree: the tracked bundle is a regular file with user, group,
and other exec bits set, it parses (`ast.parse`) and compiles, and
running the qualified `architecture/rebundle.py` generator with its
output redirected to a temp file produces byte-identical output to the
tracked bundle.

## O-004 — Generator-only prohibitions (all checks, R-004)

Regenerate ONLY by running the qualified generator
`tools/refactor-proof/architecture/rebundle.py` (for example
`python3 tools/refactor-proof/architecture/rebundle.py` from the
worktree root). Forbidden: any hand edit to the tracked bundle; any
change to any proof source (`runner/`, `src/`, `accounting/`,
`architecture/`, `tests/`, `scripts/`, `bin/tc-proof-host`) or to any
other repository file; any `refactoring-tasks/**` change (in particular
no probe, obligation, or manifest edits inside this or any other task
package); any oracle contact (no baseline import, capture, approval, or
blessing — this task neither reads nor writes oracle data); and any
hardcoding of bundle bytes to satisfy the probes. The generator output
is deterministic: a correct regeneration needs no follow-up touch-up.
Taskfmt scope + forbidden-path gates enforce this; the reviewer
confirms the bundle diff equals the generator product.
