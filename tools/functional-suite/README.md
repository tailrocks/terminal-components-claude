# Common functional suite

This standalone Cargo workspace runs real-process journeys for Showcase, Holla, Jackin Preview, and TablePro. It owns one case manifest, lockfile, `functional` Nextest target, and pinned patched engine sources. It does not build the application binaries or update expected samples.

`cases/cases.toml` registers 89 cases and 276 checkpoint IDs: 84 retained legacy cases, four first-delivery journeys with 30 checkpoints, and a separate seven-checkpoint Holla frame-1 journey. The first-delivery subset is only part of the listed inventory and does not establish full project acceptance.

## Export and prepare

Run the exporter from the application checkout. Give it a new absolute destination outside both application worktrees; it excludes the engine forks from the source export. The engine helper then creates and verifies the pinned full source trees and applies their recorded patches.

```sh
APP_ROOT="$PWD"
EXPORT="/absolute/path/to/new-functional-suite-export"
OTHER_APP_ROOT="/absolute/path/to/other-application-worktree"
python3 tools/functional-suite/scripts/export-source.py "$EXPORT" \
  --forbid-root "$APP_ROOT" \
  --forbid-root "$OTHER_APP_ROOT"
cd "$EXPORT"
python3 scripts/prepare-engines.py
```

Build `showcase`, `holla`, `jackin-preview`, and `tablepro` from the same application source and profile. Keep the binaries and each run's output directory outside the application worktrees.

## Run the first-delivery subset

From the exported suite root, use a new output directory for each run. Keep the source export, lockfile, prepared engines, binaries, and profile fixed across paired runs.

```sh
BIN_DIR="/absolute/path/to/the-four-production-binaries"
OUT_DIR="/absolute/path/to/new-output-A"
CARGO_BUILD_JOBS=2 TERMROCK_PARITY_BIN_DIR="$BIN_DIR" \
TERMROCK_PARITY_OUT_DIR="$OUT_DIR" \
"$HOME/.local/bin/rust-build-guard" -- mise exec -- cargo nextest run \
  --locked --offline --test functional --no-tests fail --retries 0 \
  --no-fail-fast --test-threads 2 -E 'test(/^first_delivery_/)'
```

Use a different fresh `OUT_DIR` for the paired run. This exact filter selects four tests and leaves 85 registered cases unrun. A nonzero exit keeps the run failed; review its saved case and frame receipts before drawing a result. Do not copy target output into the expected store as approval.

## Results at 2026-10-28

The independently accepted reference run passed 30/30 exact state-and-pixel comparisons. All four selected application processes exited cleanly, drained output, and restored the terminal. The checked-in approval record is `expected/BASELINE-APPROVAL.json` (SHA-256 `f0b1d89d44829c88b54cf0b6686835b241e7cb5a7a97a7b18bafadad56e2b3a9`).

The target run reached 16 of those 30 checkpoints: two matched, 14 had visual differences, and 14 were not reached. It exited 100 and saved its evidence. The per-application results, exact receipt hashes, current CI probe, and scope limits are in the [migration record](../../docs/architecture/termrock-migration-record.md#common-functional-suite-current-results-2026-10-28). The registered legacy cases and Holla frame-1 case remain outside this result.
