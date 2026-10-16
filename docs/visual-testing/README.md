# Visual testing

The real binaries (`showcase`, `tablepro`, `jackin-preview`, `holla`) run in
a PTY under [tuiscotti](https://github.com/tailrocks/tuiscotti) (exact `rev`
in `Cargo.toml`). Scripted keys, mouse and resize drive them, and each
captured screen is compared against one committed `.ansi` file (cells,
styles and a cursor trailer) in `snapshots/` (`baselines/oracle/` from plan
task T5 on).

- Plan and decisions: [`plan.md`](plan.md)
- Migration and re-approval status: [`tuiscotti-migration.md`](tuiscotti-migration.md)
- Evidence: [`research/`](research/)

## Scenarios

Scenarios are typed table rows in `tests/visual_baseline/scenarios/<app>.rs`:
`SURFACES`, `ELEMENTS` (hover, held press, disabled, selected+hover),
`OVERLAYS` (open, plus `-dismissed-esc` / `-dismissed-outside`),
`JOURNEYS` and `RESIZES`. `MATRIX` in `scenarios/mod.rs` expands each row
into one test per size and colour, named

```
<app>::<group>::<leaf>…::<cols>x<rows>::<color>      e.g. tablepro::connections::default::120x40::truecolor
```

The store name is the same with `/` (`tablepro/connections/default/120x40/truecolor`).
Rows marked `.pending()` and the whole narrow tier are new coverage without
an approval: they are listed and runnable, skip by default, and do not gate
until re-approved.

## Run

| What | Command |
|---|---|
| Default gate (unit tests, `store_integrity`, `scenario_tables`, boot modes) | `cargo nextest run --all-targets` |
| Smoke tier (`120x40/truecolor` + `72x20/nocolor`, the PR tier) | `cargo nextest run --profile smoke --run-ignored only` |
| Full tier (5 sizes × 5 colours, nightly) | `cargo nextest run --profile full --run-ignored only` |
| Narrow tier (`40x12`, `60x18` × truecolor, nocolor; pending) | `cargo nextest run --profile narrow --run-ignored only` |
| Everything | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` |
| One app / group / size / palette | add `-E 'test(/^tablepro::/)'`, `'test(/::connections::/)'`, `'test(/::80x24::/)'`, `'test(/::nocolor$/)'` |
| One scenario | `-E 'test(=tablepro::connections::default::120x40::truecolor)'` |
| One CI shard locally | add `--partition hash:1/2` |
| Scenarios still on tuisnap approvals, or pending | prefix `VISUAL_FULL=1` (they fail until approved) |
| Inventory with approval state | `cargo xtask visual list ['<filterset>']` |

Without `VISUAL_FULL=1`, a scenario runs only when its approval is already a
tuiscotti `.ansi`; every other one passes as a skip with a note.

Never use `cargo test` (AGENTS.md).

## Read a failure

Each failing scenario writes:

| File | Meaning |
|---|---|
| `target/tuiscotti/actual/<name>.ansi` | The captured file. `cat` it in a terminal, or `diff` it against `snapshots/<name>.ansi`. |
| `target/tuiscotti/actual/<name>.{png,html}` | The captured screen, rendered. |
| `target/tuiscotti/expected/<name>.{png,html}` | The approved screen, rendered from its `.ansi`. |
| `target/tuiscotti/diff/<name>.png` | Changed cells in red. |
| `target/tuiscotti/actual/<name>.frame.json` | The full frame (cells, styles, cursor). |

To view any approved scenario: `cargo xtask visual render <name>` writes
`target/tuiscotti/render/<name>.{png,html}`.

## Approve an intended change

Only on your machine, only after looking at the review files. CI never
accepts: `accept` and `import` refuse to run when `CI` is set.

1. Run the affected scenarios (add `VISUAL_FULL=1` for scenarios not yet on
   tuiscotti) and look at `target/tuiscotti/{actual,expected,diff}`.
2. `cargo xtask visual accept '<nextest filterset>'` copies the actual
   `.ansi` of every matching scenario into `snapshots/` and drops tuisnap's
   `.txt/.png/.html` for it.
3. `cargo nextest run --all-targets` (`store_integrity` must stay green).
4. Commit the `snapshots/` change on its own, with a message that names the
   visual change, why, and the scenarios touched. AGENTS.md requires
   deliberate visual changes to be called out.

There is no environment variable that blesses snapshots, and there never
will be.

To import a whole capture (oracle v0/v1: a CI or local `actual/` tree):
`cargo xtask visual import <dir> --dry-run` prints new / changed /
cursor-only / unchanged counts per app/group; without `--dry-run` it writes
`snapshots/`.

## Rules

- Do not touch the `visual-baseline` tag or its release.
- A flaky scenario is a bug. Fix the wait or the determinism hook (`--motion
  paused --frame N`, now on all four binaries); do not add retries.
- New coverage goes into the tables as `.pending()` rows; a pending row
  gates once its approval is committed.
- Whoever changes a re-approved surface re-runs its group and approves it
  with the change.
