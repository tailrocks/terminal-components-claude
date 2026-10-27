# Termrock functional suite

This package uses one standard Rust integration target at `tests/functional.rs`. It has no Suite CLI. The target runs the existing application journeys through real `Session` children and the fixed `Case` table in `tests/common/`.

The target has six real journeys: `first_showcase`, `first_holla`, `first_jackin`, `first_tablepro`, `showcase_help_paging`, and `holla_first_use_action`. The first four are the selected lane: four journeys and sixteen checkpoints. The complete retained lane has six journeys and thirty-one checkpoints. `required_inventory_ready` is a separate zero-flow readiness test. It fails with `NOT VERIFIED` while the full application inventory is absent. An unfiltered run therefore stays incomplete.

## Fixed journey inputs

The first four journeys use a 120 by 40 PTY. Showcase help uses 80 by 24. All journeys start with a cleared child environment containing `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`, `TZ=UTC`, `TERM=xterm-256color`, and `COLORTERM=truecolor`; `NO_COLOR` is absent. They use their existing paused-motion, frame-zero, TrueColor launch arguments and fixture directories. The first Showcase journey also passes `--page overview`. Jackin uses the returning scenario, Holla uses `first-use`, and TablePro does not connect to a server.

The first lane records active checkpoint labels `opened`, `help`, `returned`, and `exited`. The frozen Sample IDs keep their historical suffixes:

| Active label | Historical Sample ID suffix |
| --- | --- |
| `opened` | `initial` |
| `help` | `help` |
| `returned` | `restored` |
| `exited` | `exit` |

Showcase help and Holla first-use keep their other existing checkpoint labels and semantic assertions. Their `initial`, `restored`, and `exit` labels use the same mapping. No approved Sample ID or sample bytes are renamed.

Each test reads only absolute `TERMROCK_BIN_DIR` and `TERMROCK_OUT_DIR` paths at runtime. It verifies both paths against the sealed export boundaries. Give each invocation a fresh, pre-created OUT directory below a sealed run-output root. Each test creates its own case directory there and copies the case fixture into a private working directory. A reused case directory fails. Frozen checks always run. The tests do not select a mode, group, source revision, or seal at runtime. The compiled test executable uses the export SHA supplied at build time.

Each case waits for the visible initial screen and visual quiet before it captures. It uses the owned active-screen snapshot and atomic raw-output offset, creates a Tuiscotti Sample with pixels, serializes it, and checks it against the frozen store. It saves the raw stream, reached prefixes, snapshots, Samples, PNGs, actions, semantic assertions, process state, checks, and case result before returning a case error where evidence writing succeeds.

At exit, the case sends its existing public quit action, waits for `WaitExit`, and records State. It requires exit code zero, no signal, complete raw capture, successful drain and flush, no raw-capture errors, and matching captured, processed, file-length, and final snapshot offsets. It captures the final active screen before cleanup. Cleanup cannot count as an exit capture.

Text blink is not retained by the Alacritty backend. Link IDs with the `_alacritty` suffix are not retained. The comparator does not resolve Default, Indexed, and RGB color references to compare their resulting colors. These fields remain failed or `NOT VERIFIED` where applicable; this suite does not claim full exact-state fidelity.

## Source and binary inputs

The binary directory must contain `showcase`, `holla`, `jackin`, and `tablepro`, plus a valid `source-manifest.json`. The staged `jackin` binary must have the same SHA-256 as the built `jackin-preview` binary. The manifest binds the clean source commit and tree, build log, build hash file, and staged binary hashes. The test checks these values before it launches an application.

The suite pins `tui-test-rs` 0.1.1 to maintained engine base `3887a75dee4463ce0122c37eb278ab7ebb71aeaf` (tree `4840743cfb7371d2628f2b77afd619e18cee29c9`). The complete capture patch is rebased from historical patch input `ba409cfb2a7b2481a98068a36ed906a07059222c7c3ec1eccc18c19c6c5fef9e`; its SHA-256 is `fad39f69c647ac2218efdf263a76ff569b8e7c715c0d13451e8b6a963362971d` and its result tree is `43571813ce7ef7481c7032dfa110c62df7beb062`. The recovery bundle contains the sole ref `common-raw-388` at `4e51249f5fb6baa33495972b8b8ee924a9122ded` (tree `2dae292afaa4a47f84f234faecc69b680dce0325`). The suite pins Tuiscotti `0.3.0` at `ac5f70d14d1967f551261d599a4cd857ec9fa2d7`.

The old in-package source-manifest writer and raw-probe commands are retired. `STREAM.md` records the earlier CLI and results as history. Its commands do not describe this standard target. Historical capture records remain in Git history and the preserved stream evidence.

## Export and nextest

Create one authenticated export from the reviewed source commit. Keep the export, application worktrees, staged binary directories, and run-output roots disjoint. Record the helper SHA `V` and the export SHA `H` from the authenticated export procedure. Keep the same `V`, `H`, lockfile, feature set, engine checkout, and nextest version for both application branches. Only the two runtime BIN and OUT path values may differ.

Before each Cargo command, verify the sealed export and use the guarded mise command with a private command-scoped target directory outside the source and export. The workspace pins Rust 1.99.1, Mr. Boxington 1.28.0, mise 2026.10.21 or later, and cargo-nextest 0.9.150. The project Mise configuration places the selected Nextest install before ambient Cargo binaries. Keep `MISE_ENABLE_TOOLS` set to include the canonical ID `aqua:nextest-rs/nextest/cargo-nextest`.

First list the selected tests and validate the names returned by nextest JSON. Use the default profile and two build jobs. Stop before the run if the selected set is not exactly these four names: `first_showcase`, `first_holla`, `first_jackin`, `first_tablepro`.

```sh
(cd "$EXPORT" && \
  TERMROCK_SUITE_EXPORT_SHA256="$H" CARGO_TARGET_DIR="$PRIVATE_TARGET" \
  MISE_ENABLE_TOOLS=rust,mr-boxington,aqua:nextest-rs/nextest/cargo-nextest \
  MISE_AUTO_INSTALL=false MISE_LOCKFILE=false \
  /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo \
  nextest list --locked --test functional --profile default --build-jobs 2 \
  -E 'test(/^first_/)' --message-format json) || exit $?
```

Run the first lane from the same export for each branch. Use a fresh OUT root for every invocation. Set `TERMROCK_SUITE_EXPORT_SHA256` to the one fixed parent H used to build the target.

```sh
(cd "$EXPORT" && \
  TERMROCK_SUITE_EXPORT_SHA256="$H" TERMROCK_BIN_DIR="$BIN" TERMROCK_OUT_DIR="$RUN_OUT" \
  CARGO_TARGET_DIR="$PRIVATE_TARGET" \
  MISE_ENABLE_TOOLS=rust,mr-boxington,aqua:nextest-rs/nextest/cargo-nextest \
  MISE_AUTO_INSTALL=false MISE_LOCKFILE=false \
  /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo \
  nextest run --locked --test functional --profile default --build-jobs 2 \
  --test-threads 2 --no-tests=fail -E 'test(/^first_/)' \
  --no-fail-fast --retries 0) || exit $?
```

The default Nextest profile uses `default-filter = "all()"`, zero retries, no fail-fast behavior, and a terminating timeout. On Unix, Nextest sends SIGTERM to the test process group after five periods of 60 seconds, then sends SIGKILL after a 10-second grace period. A timeout is a failure. This outer limit does not set a ten-second deadline for each cleanup call or prove that the PTY child exited, capture drained, or final evidence was written. Forced termination can prevent later evidence from being written. Record missing evidence as unreached and unverified. Do not use `cargo test`, retries, ignored tests, xfails, capture mode, or baseline approval in CI. Keep the unfiltered target incomplete until the required inventories are implemented and run.

The stored first samples have independent approval for the finite first-lane profile only. The other 13 retained samples stay unchanged; the two retained journeys also capture their final exit screen, for which no approved frozen sample exists. Preserve those ordinary test failures and their evidence. The 16 first references and the remaining 13 files do not prove 31 completed checkpoints.
