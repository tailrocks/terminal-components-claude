# Visual validation during refactoring

The only visual oracle is the grouped store and PTY suite frozen by the
immutable `visual-baseline` tag. The current refactor branch does not contain
that store, `tests/visual_baseline/`, or the matching nextest configuration.
Therefore no visual acceptance gate is runnable or passable in this tree.

A verifier subagent must first import the required oracle inputs read-only into
an external run directory from the peeled tag commit recorded in
[`docs/refactoring-plan/execution-readiness-report.md`](../docs/refactoring-plan/execution-readiness-report.md).
The import must be exact and disposable. Never modify the tag, its release, or
the oracle namespace, and never bless candidate output.

After the oracle suite is ported to the current architecture, use only the
affected `cargo nextest` filters during an edit loop and the unfiltered full
gate at acceptance boundaries. Visual equality must include the real settled
component state and behavioral transitions; static frames alone are
insufficient. The gate must fail closed on missing, changed, or unapproved
artifacts.

The historical grouped-store taxonomy is:

```text
snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.
{ansi,txt,png,html}
```

The historical gate commands below are future-only until the store and suite
exist in the candidate checkout:

```sh
# focused edit loop
TUISNAP_FAST=1 cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(<filter>)'

# acceptance/closure
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Candidates never write `snapshots/`, run `tuisnap accept`, or use the deleted
`shots/` corpus as an oracle. See the readiness report and
[`docs/refactoring-plan/proof-contract.md`](../docs/refactoring-plan/proof-contract.md)
for current evidence ownership.
