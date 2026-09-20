# Visual validation during refactoring

The only visual oracle is the grouped store frozen by the immutable
`visual-baseline` tag. This branch already contains the tag-derived suite at
[`tests/visual_baseline/`](../tests/visual_baseline/) and the matching
[`.config/nextest.toml`](../.config/nextest.toml). The snapshot store itself
remains an external protected-oracle input; it is not present on the candidate
branch.

A verifier subagent must first import the required oracle store read-only into
an external run directory from the peeled tag commit recorded in
[`docs/refactoring-plan/execution-readiness-report.md`](../docs/refactoring-plan/execution-readiness-report.md).
The import must be exact and disposable. Never modify the tag, its release, or
the oracle namespace, and never bless candidate output.

No visual acceptance run is valid until that import covers the complete frozen
matrix: exactly 7,550 keys and 30,200 artifacts (ANSI, plain text, PNG, and
HTML for every key), five terminal sizes (72x20, 80x24, 100x30, 120x40, and
160x50), and five color modes (truecolor, 256-color, 16-color, `none`, and
`nocolor`). A smaller “supported,” “relevant,” or “applicable” subset is not
an acceptance gate.

After the oracle store is bound, use only the affected `cargo nextest` filters
during an edit loop and the unfiltered full gate at acceptance boundaries.
Visual equality must include the real settled component state and behavioral
transitions; static frames alone are insufficient. The gate must fail closed
on missing, changed, or unapproved artifacts.

The historical grouped-store taxonomy is:

```text
snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.
{ansi,txt,png,html}
```

The suite and nextest configuration already exist in this checkout. The
commands below remain unrunnable as an acceptance gate until the verifier
binds the imported grouped store:

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
