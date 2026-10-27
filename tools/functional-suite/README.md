# Termrock functional suite

The `termrock-functional-suite` package uses one Rust integration target: `tests/functional.rs`. It has no suite command-line program. The tests start the real application binaries through `Session`.

The machine-readable case inventory is in `cases.toml`. The terminal profiles are in `terminal.toml`. The test target checks both files against its compiled case registry before it launches an application. First30 sample IDs use `{case_id}/{checkpoint_label}/pixels-{platform_key}` and keep the new checkpoint labels unchanged. The inventory is a bounded protocol; it is not the full product denominator.

## Selected first30-v1 lane

The selected lane has four cases, thirty live checkpoints, and four separate exit receipts.

| Case ID | Live checkpoints | Exit action |
| --- | ---: | --- |
| `first30-v1/showcase.buttons` | 7 | `q` |
| `first30-v1/holla.first_use` | 7 | `Ctrl+q` |
| `first30-v1/jackin.settings` | 10 | `q` |
| `first30-v1/tablepro.connections` | 6 | `Ctrl+q` |

The Holla frame-one variant is a separate case. It has seven live checkpoints and one exit receipt. It changes only `--frame 0` to `--frame 1`. It is outside the four-case, thirty-frame lane.

Keep these requirements in the target:

- `showcase_help_paging` tests the Showcase help journey.
- `holla_first_use_action` tests the Holla action journey.
- `holla_first_use_frame_one_variant` tests the separate Holla frame-one case.
- `required_inventory_ready` fails with `NOT VERIFIED` while the full application inventory is absent.

The selected first30-v1 filter does not run the Showcase help journey, the Holla action journey, or the frame-one variant. Case results list these as separate not-run requirements. The unfiltered target also runs the retained journeys and the Holla frame-one variant. The full inventory check still fails.

The case inventory records why first30-v1 replaces the historical four-case, sixteen-frame subset. It preserves the older executions and all 29 frozen samples. It does not mark the full application inventory ready.

The previous four-case, sixteen-checkpoint lane remains historical evidence in `STREAM.md`. Keep all 29 frozen sample files byte for byte. The first30-v1 sample IDs use their own namespace. They have no approval until independent reference captures and approvals exist. Keep the existing Showcase help and Holla action requirements and their expected samples.

## Input and capture rules

At runtime, the target reads only these path selectors:

- `TERMROCK_PARITY_BIN_DIR`
- `TERMROCK_PARITY_OUT_DIR`

Both values must be canonical absolute directories. The runner has no fallback to the former environment names. `TERMROCK_SUITE_EXPORT_SHA256` is a build-time export authentication input. It is not a runtime selector.

The runner clears the child environment. It sets `TERM=xterm-256color`, `COLORTERM=truecolor`, `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`, and `TZ=UTC`. It leaves `NO_COLOR` unset and uses the default theme.

The first30-v1 cases start at 120 by 40 cells. The named `small` checkpoint resizes to 80 by 24. The `restored` checkpoint resizes back to 120 by 40. The retained Showcase help journey starts at 80 by 24; the retained Holla action journey starts at 120 by 40. Each terminal profile records the launch arguments, dimensions, environment, readiness bound, and quiet interval. Showcase also uses `showcase --page buttons`; Holla uses `holla --scenario first-use`; Jackin uses `jackin-preview --scenario returning`; TablePro uses `tablepro`.

Each live checkpoint waits up to eight seconds for its visible marker. It then waits for 400 ms of visual quiet and captures the visible screen with its raw-output offset. It checks the full semantic predicate against that captured screen. It checks the child state at each live checkpoint. It records the matched and missing text, forbidden text, dimensions, tree row styles where required, and raw offset.

A semantic readiness or input postcondition failure stops later dependent inputs and checkpoints. The runner keeps the failed sample, PNG, snapshot, semantic record, action log, and raw prefix. A frozen state or pixel mismatch does not stop an otherwise valid journey. The runner completes the remaining steps and records all reachable checkpoint outcomes.

The runner compares every reached sample after the journey stops. It does this even when a later input, semantic check, exit, or raw-drain check fails. Each reached sample has its own comparison record. The record reports `pass`, `failure`, `setup_failed`, or `unavailable`. A missing baseline is `setup_failed`; it is separate from an observed state or pixel difference. The result keeps live-frame counts separate from exit-receipt comparison counts. An unavailable comparison never counts as a pass. The overall result is `setup_failed` only when missing or unavailable comparisons are the only failures. An application, semantic, exit, or frozen-comparison failure keeps the overall result `failed`.

The final screen and its raw offset are stored as an exit receipt. The receipt is not part of the live-frame denominator. The runner records the complete final `State` and the captured exit snapshot. It checks that the alternate screen is off, the cursor is visible, bracketed paste is off, mouse tracking is `none`, and the terminal has the case's expected dimensions. The target also requires exit code zero, no signal, complete raw capture, a complete drain, a successful flush, no capture errors, and equal captured, processed, file-length, and final snapshot offsets.

Each case uses a private working directory under its fresh output directory. The suite export binds the application roots, binary roots, and output roots so this directory stays outside the product trees. The result records the working directory, fixture files, and copied fixture-data hash for each case. Both branches use the same exported fixtures.

## Binary and source evidence

The staged binary directory must contain exactly these application names:

- `showcase`
- `holla`
- `jackin-preview`
- `tablepro`

The runner launches `jackin-preview` directly. It does not require a `jackin` alias. The source manifest and build hash evidence list the same four names. The source manifest binds the clean source commit and tree, build log, build hash file, and staged binary hashes.

The suite uses the committed engine base at `3887a75dee4463ce0122c37eb278ab7ebb71aeaf` with the maintained capture patch. Read the current Tuiscotti version and Git revision from `engine/pin.json` and `Cargo.lock`. Keep the package version and Git revision paired with the committed lockfile.

The engine pixel-after-state correction remains separate. When state differs, the current comparator can stop before it computes pixels. Zero pixel counts do not prove pixel equality. Report pixel results as unavailable when the comparator does not compute them. Do not claim pixel parity for those cases.

## Export and selected run

Create one authenticated export from the reviewed source commit. Keep the export, application worktrees, staged binary directories, output roots, and private build target separate. Use the same export seal, source revision, lockfile, feature set, engine, executable, and comparison policy for both branches. Only the two runtime path values may differ.

Before each Cargo command, check the sealed export and use the disk-guarded mise command with a private command-scoped target directory outside the source and export. Use the committed mise and Rust pins. Use two build jobs and the default Nextest profile.

List the selected tests and verify that Nextest returns exactly these four names:

- `first30_showcase_buttons`
- `first30_holla_first_use`
- `first30_jackin_settings`
- `first30_tablepro_connections`

Stop if the selected set differs.

```sh
(cd "$EXPORT" &&   TERMROCK_SUITE_EXPORT_SHA256="$H" CARGO_TARGET_DIR="$PRIVATE_TARGET"   MISE_ENABLE_TOOLS=rust,mr-boxington,aqua:nextest-rs/nextest/cargo-nextest   MISE_AUTO_INSTALL=false MISE_LOCKFILE=false   /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo   nextest list --locked --test functional --profile default --build-jobs 2   -E 'test(/^first30_/)' --message-format json) || exit $?
```

Run the selected lane from the same export for each branch. Use a new output directory for each run. Set the same build-time export seal for both test executables.

```sh
(cd "$EXPORT" &&   TERMROCK_SUITE_EXPORT_SHA256="$H"   TERMROCK_PARITY_BIN_DIR="$BIN" TERMROCK_PARITY_OUT_DIR="$RUN_OUT"   CARGO_TARGET_DIR="$PRIVATE_TARGET"   MISE_ENABLE_TOOLS=rust,mr-boxington,aqua:nextest-rs/nextest/cargo-nextest   MISE_AUTO_INSTALL=false MISE_LOCKFILE=false   /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo   nextest run --locked --test functional --profile default --build-jobs 2   --test-threads 2 --no-tests=fail -E 'test(/^first30_/)'   --no-fail-fast --retries 0) || exit $?
```

The default Nextest profile has zero retries and does not stop after the first test failure. It has a 60-second timeout with five attempts and a 10-second termination grace period. A timeout is a failure. A forced stop can prevent later evidence from being written. Record missing evidence as not run.

Do not use `cargo test`, retries, ignored tests, expected failures, capture mode, or reference approval in CI. Reference approval uses the reviewed independent process. A build or a passing bounded lane does not prove all 54 routes, all components, four applications, or full acceptance.
