# Visual validation during refactoring

Live visual oracle is the committed grouped store. Use it to prove a refactor
did not change user-visible output. Do not use the deleted `shots/` corpus.

```text
snapshots/<app>/<family>/[<surface>/][<state>/]<cols>x<rows>/<color>.{ansi,txt,png,html}
```

Each terminal size is its own folder. Color is the leaf. Taxonomy:
`docs/baseline/snapshots-v2.md`. Suite: `tests/visual_baseline/`.

Tiered gates, filters, and timing: [Campaign iteration guide](../docs/refactoring-plan/campaign-iteration-guide.md).

## Gate

Fail-closed. Tests never bless. Candidates never write `snapshots/` and never
run `tuisnap accept`.

### Tier summary

| Tier | Command | Scope | When |
| --- | --- | --- | --- |
| **Edit loop** | `TUISNAP_FAST=1` + targeted `-E` filter | Affected app/scenario only | Per production edit during a task |
| **Ordinary regression** | `cargo nextest run` | ~544 non-ignored tests (no PTY captures) | After production changes |
| **PR / CI smoke** | `--profile ci --run-ignored only -E 'binary(visual_baseline)'` | ~302 captures (120×40 truecolor) | Pull-request automation |
| **Nightly speed feedback** | `TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` | Full 7,550 combos (tiered gate) | Optional; may drift vs fidelity-blessed snapshots on timing-sensitive flows |
| **Acceptance / closure** | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` | Full matrix; PNG/HTML every combo | Task acceptance, integration boundaries, chain closures, TASK-069 |

Smoke and targeted filters do **not** substitute for the mandatory full gate at acceptance boundaries.

### Commands

```sh
# unit/lib tests (~544 tests, ~1–3 min; excludes PTY captures)
cargo nextest run

# store inventory (non-PTY, always cheap)
cargo nextest run -E 'test(store_integrity)'

# edit loop — targeted filter + fast mode (seconds–few minutes)
TUISNAP_FAST=1 cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(<filter>)'

# PR / CI smoke (~302 captures, ~1–2 min)
cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline)'

# acceptance / closure — full matrix at fidelity timing (~45–60 min)
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'

# optional nightly speed feedback — full matrix, tiered gate (~30–45 min)
TUISNAP_FAST=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
```

Example filters (see iteration guide §5):

```sh
# whole app
-E 'binary(visual_baseline) & test(showcase_)'

# one family
-E 'binary(visual_baseline) & test(holla_flows_)'

# one capture root (25 combos)
-E 'binary(visual_baseline) & test(holla_concept_first_use)'

# single combo
-E 'binary(visual_baseline) & test(holla_concept_first_use_default_120x40_truecolor::c120x40_truecolor)'

# smoke slice of one app (CI profile)
cargo nextest run --profile ci --run-ignored only -E 'binary(visual_baseline) & test(holla_)'
```

`matched` = no UI/UX drift. `cells-differ` / `pixels-differ` = regression.
Inspect `target/tuisnap/diff/<name>.png` and the first-difference notes.

`verify.toml` `forbidden_paths` includes `snapshots` and `shots` on every task.
`CHK-004` compare still uses the sealed oracle bundle when that product exists;
this store is the additional live regression gate for product edits on holla.

## Bless (host only)

Only the host blesses an *intended* visual change:

```sh
ln -sfn target/tuisnap/actual snapshots.actual
cargo run --manifest-path ~/Projects/tui-snap/Cargo.toml --release -- \
  accept --grouped --store snapshots --name <group/…/size/color>
```
