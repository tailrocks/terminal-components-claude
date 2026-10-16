# Visual testing

The real binaries (`showcase`, `tablepro`, `jackin-preview`, `holla`) run in
a PTY under [tuiscotti](https://github.com/tailrocks/tuiscotti) (exact `rev`
in `Cargo.toml`). Scripted keys, mouse and resize drive them, and each
captured screen is compared against one committed `.ansi` file (cells,
styles and a cursor trailer) in `snapshots/`, and in `baselines/oracle/`
once oracle v0 is imported (see "Oracle store" below).

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
| Pre-push guard for UI changes | `cargo xtask visual check-touched` (below) |
| Capture twice, fail on any byte difference | `cargo xtask visual determinism '<filterset>'` |
| Skip review PNG/HTML on mismatches (fast reruns) | prefix `VISUAL_REVIEW=0` |

Without `VISUAL_FULL=1`, a scenario runs only when its approval is already a
tuiscotti `.ansi`; every other one passes as a skip with a note.

The default run also holds the harness checks: `oracle_integrity`,
`isolation::env_isolation` (no parent variable reaches a capture; each one
gets an emptied `target/tuiscotti/home/<name>` as `HOME` and cwd),
`determinism::sources_are_deterministic` (no wall-clock or random source
in `src/`, every `Instant::now` reviewed) and `overlay_sites` (every
overlay call site in `src/bin` maps to a scenario row or a listed gap).

## Gate for UI changes (part of the push protocol)

Approvals go stale silently when a change moves an approved screen and
nobody re-runs it. Before every push of a change under `src/`, after the
rebase and next to the repo gate, run:

```sh
cargo xtask visual check-touched            # vs origin/design/interaction-polish
cargo xtask visual check-touched --dry-run  # print the mapping and filterset only
```

It maps every file changed since the merge base (committed, staged,
unstaged, untracked) to apps: `src/bin/<app>/` to that app; any other
`src/` file, `tests/visual_baseline/`, `Cargo.toml` or `Cargo.lock` to all
four; `snapshots/<app>/` or `baselines/oracle/<app>/` to that app. Then it
runs every re-approved group of those apps (`--base <ref>` picks another
base). Green: push. Red: the change moved an approved screen. Review
`target/tuiscotti/{actual,expected,diff}`; re-approve only what the change
intends, in a snapshot-only commit that names the change and its commit
(`cargo xtask visual accept`), or fix the regression.

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
accepts: `accept`, `import`, `seal`, `ratchet` and `sync-oracle` refuse to
run when `CI` is set.

1. Run the affected scenarios (add `VISUAL_FULL=1` for scenarios not yet on
   tuiscotti) and look at `target/tuiscotti/{actual,expected,diff}`.
2. `cargo xtask visual accept '<nextest filterset>'` copies the actual
   `.ansi` of every matching scenario into `snapshots/` and drops tuisnap's
   `.txt/.png/.html` for it. It copies only the current run: every
   capture writes a `<name>.run` stamp (nextest run id and a hash of the
   `.ansi`), and `accept` refuses, writing nothing, when a selected actual
   is unstamped, changed since, or older than the newest run in the
   selection (for example a pending row captured by an earlier
   `VISUAL_FULL=1` run). Each scenario test also deletes its previous
   actual before running, so a skip or an early failure leaves none.
3. `cargo nextest run --all-targets` (`store_integrity` must stay green).
4. Commit the `snapshots/` change on its own, with a message that names the
   visual change, why, and the scenarios touched. AGENTS.md requires
   deliberate visual changes to be called out.

There is no environment variable that blesses snapshots, and there never
will be.

To import a whole capture (oracle v0/v1: a CI or local `actual/` tree):
`cargo xtask visual import <dir> --dry-run` prints new / changed /
cursor-only / unchanged counts per app/group; without `--dry-run` it writes
the current store, and with `--oracle [--source <commit>]` it writes every
scenario into `baselines/oracle/` and seals the manifest.

## Oracle store

`baselines/oracle/` (plan §B4) holds one `.ansi` per scenario and
`ORACLE.toml`: `version`, `pending`, `source_commit`, `tuiscotti_rev`,
`harness_sha256`, `approved_by`, `date` and a count per app. Until oracle
v0 it is a skeleton with `pending = true`: the gate keeps reading
`snapshots/`, and `oracle_integrity` only checks that the manifest parses
and the store holds nothing else. `cargo xtask visual import <capture>
--oracle` writes it and seals `pending = false`; from then on the gate,
`accept` and `list` use the oracle, `accept` re-seals it, and
`oracle_integrity` checks every file, the counts, the tuiscotti pin and
the harness hash (`cargo xtask visual seal` after a reviewed harness
change).

| Command | Use |
|---|---|
| `cargo xtask visual seal [--source <commit>]` | rewrite `ORACLE.toml` for the store as it is (local only) |
| `cargo xtask visual ratchet [--check] ['<filterset>']` | refactor branches (plan §F1): listed passing scenarios must still pass; newly passing ones are added to `tests/visual_baseline/progress/main-passing.txt` (not part of the harness hash). `--check` writes nothing and may run in CI |
| `cargo xtask visual sync-oracle --from <commit> [--repo <path>] [--harness-dest <dir>] [--allow-interim]` | plan §E3: install `baselines/oracle/` and the harness from an approved commit (v1+, or `--allow-interim` for v0), refusing a harness whose hash differs from the manifest; the destination keeps its `progress/` |

## Rules

- Do not touch the `visual-baseline` tag or its release.
- A flaky scenario is a bug. Fix the wait or the determinism hook (`--motion
  paused --frame N`, now on all four binaries); do not add retries.
- New coverage goes into the tables as `.pending()` rows; a pending row
  gates once its approval is committed.
- Whoever changes a re-approved surface re-runs its group and approves it
  with the change: `cargo xtask visual check-touched` before the push.
- A new popup, menu, dialog or picker gets an `OVERLAYS` row (open + both
  dismissals) and an entry in `overlay_sites::SITES`.
