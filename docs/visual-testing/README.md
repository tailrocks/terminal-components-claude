# Visual testing

The real binaries (`showcase`, `tablepro`, `jackin-preview`, `holla`) run in
a PTY under [tuiscotti](https://github.com/tailrocks/tuiscotti) (exact `rev`
in `Cargo.toml`). Scripted keys, mouse and resize drive them, and each
captured screen is compared against one committed `.ansi` file (cells,
styles and a cursor trailer) in `baselines/oracle/`.

- Plan and decisions: [`plan.md`](plan.md)
- Evidence: [`research/`](research/)

Some commands below arrive with plan tasks T1–T5 (§H). Until those land, the
`tuisnap`-era instructions in `docs/baseline/` apply.

## Run

| What | Command |
|---|---|
| Default gate (unit tests, `store_integrity`, boot tests, journeys) | `cargo nextest run --all-targets` |
| Full visual suite | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` |
| Smoke tier (the PR job) | `cargo nextest run --run-ignored only -E 'binary(visual_baseline) & (test(/::120x40::truecolor$/) \| test(/::72x20::nocolor$/))'` |
| One app | add `& test(/^tablepro::/)` |
| One scenario | `-E 'test(=tablepro::connections::default::120x40::truecolor)'` |
| One CI shard locally | add `--partition hash:3/12` |

Never use `cargo test` (AGENTS.md).

## Read a failure

Each failing scenario writes:

| File | Meaning |
|---|---|
| `target/tuiscotti/report.html` | Every scenario with its status. Open this first. |
| `target/tuiscotti/review/<name>/expected.png`, `actual.png` | Approved and captured screens, rendered from `.ansi`. |
| `target/tuiscotti/review/<name>/diff.png` | Changed cells in red. |
| `target/tuiscotti/review/<name>/*.html` | The same screens with the frame JSON. |
| `target/tuiscotti/actual/<name>.ansi` | The captured file. `cat` it in a terminal, or `diff` it against `baselines/oracle/<name>.ansi`. |

In CI, failed shards upload these as `visual-<tier>-<k>` artifacts (14 days).

To view any approved scenario: `cargo xtask visual render <name>`.

## Approve an intended change

Only on your machine, only after looking at the report. CI never accepts.

1. Run the affected scenarios and open `target/tuiscotti/report.html`.
2. `cargo xtask visual accept '<nextest filterset>'` copies the actual
   `.ansi` files into `baselines/oracle/`.
3. `cargo nextest run --all-targets` (`store_integrity` must stay green).
4. Commit with a message that names the visual change and why. AGENTS.md
   requires deliberate visual changes to be called out.

There is no environment variable that blesses snapshots, and there never
will be.

## Rules

- Do not touch the `visual-baseline` tag or its release.
- `baselines/oracle/` on `main` changes only through `cargo xtask visual
  sync-oracle` from an approved commit of `design/interaction-polish`.
- A flaky scenario is a bug. Fix the wait or the determinism hook; do not
  add retries.
