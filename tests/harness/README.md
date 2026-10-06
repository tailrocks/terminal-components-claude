# Visual-baseline harness

Isolated runner for the tuiscotti visual-baseline suite. It exists so the
root `Cargo.toml`/`Cargo.lock` carry no capture-engine dependency: this
crate has its own manifest and lockfile, and the suite moved here verbatim
from `tests/visual_baseline/`.

## Layout

- `tests/visual_baseline/` — the suite (moved, not copied).
- `Cargo.toml` — own deps: `junie-tui` (path), the four preview `[[bin]]`
  targets compiled from `../../src/bin/*/main.rs` (so
  `env!("CARGO_BIN_EXE_*")` keeps working), `tuiscotti` + `ratatui029`.
- `.config/nextest.toml` — copy of the root PTY nextest profile.
- `target/` — harness-local build + capture scratch (ignored).

The approved store stays at the repo-root `snapshots/`; the suite resolves
it as `../../snapshots` from `CARGO_MANIFEST_DIR`. Scratch (actuals, diffs,
report) lands under this crate's `target/tuiscotti/`.

## Run

From this directory:

```sh
cargo nextest run -E 'binary(visual_baseline)'                        # default (non-PTY) suite
cargo nextest run --run-ignored only -E 'binary(visual_baseline)'     # full PTY baseline
```

Bin unit tests are `test = false` here on purpose: they still run in the
root package (`cargo nextest run` at the repo root). Bless captures from
this directory with
`tuiscotti accept --grouped --store ../../snapshots --all`.
