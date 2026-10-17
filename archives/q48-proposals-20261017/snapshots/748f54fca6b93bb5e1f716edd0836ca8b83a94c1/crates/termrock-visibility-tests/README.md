# Termrock visibility CLI tests

This is a standalone Rust test package for the visibility command-line tools.
Its local workspace and lock file keep its dependency closure separate from the
Termrock product workspace and the PTY suite in `termrock-e2e`.

The tests are authored and run in Rust. A Rust test may start `python3` to run
one byte-verified production tool as a command-line program. It does not import
the tool as a Python module or run a Python test driver. Tool copies, fixture
inputs, queue changes, and generated receipts stay below a unique temporary
repository root that is removed when the test finishes.

The package has no UI, product-crate, PTY, or Tuiscotti dependency. Direct
dependencies are pinned to versions already present in the common suite lock:
`libc` 0.2.190, `serde_json` 1.0.151, `sha2` 0.10.9, and `tempfile` 3.27.0.

## Run the Rust checks

Use the pinned Rust toolchain and Nextest. Do not use `cargo test`.

```sh
rustup run 1.98.1 cargo nextest run \
  --manifest-path crates/termrock-visibility-tests/Cargo.toml \
  --locked --jobs 2
```

The package is intentionally not a member of the root Cargo workspace.

## Test support

`TempRepo::new()` creates a temporary repository root with automatic cleanup.
It returns the canonical path, so copied files and CLI working directories do
not retain macOS `/var` aliases.
`copy_tool_exact` copies a production visibility script to its normal relative
path and verifies both its bytes and SHA-256. `copy_project_file_exact` does
the same for fixed inputs such as schemas. `run_cli` invokes
`python3 <script> <args>` through `std::process::Command`, with captured raw
stdout, stderr, exit code, and signal status. Stdin, stdout, and stderr move
concurrently to avoid pipe deadlocks. Each output capture retains at most 8 MiB
and reports truncation; the command has a 120-second deadline, kills its Unix
process group on timeout, and reaps the direct child after group cleanup. The
runner checks cancellation between every pipe read, so a descendant that leaves
the process group cannot hold capture threads open indefinitely. Such escaped
processes are outside the helper's process-containment guarantee. Bounded CLI
capture is supported only on Unix; other platforms return `Unsupported` before
spawning. Tests can call `run_cli_with_timeout` for a different bounded deadline
when measured work needs more time. The runner does not use a shell.

`hold_queue_lock` matches the queue CLI lock key and `fcntl.flock` primitive.
Its temporary root selection checks `TMPDIR`, `TEMP`, and `TMP`, then the Unix
fallbacks `/tmp`, `/var/tmp`, and `/usr/tmp`, with the current directory last.
It canonicalizes the selected directory, matching Python's
`tempfile.gettempdir()` behavior when those variables are absent. It opens each
directory component with `openat` and `O_NOFOLLOW`, creates the private lock
directory relative to the verified temp-root descriptor, then opens the lock
file relative to that descriptor with `O_NOFOLLOW`.

Integration tests pass `Path::new(env!("CARGO_BIN_EXE_fixture"))` to
`install_fixture_aliases`. This places the compiled Rust fixture under selected
command names such as `rustup`, `mise`, `cargo`, `rustc`, and `cargo-nextest` in a
temporary `bin` directory and returns a PATH value with that directory first.

The fixture supports these protocols:

- `rustup which --toolchain 1.98.1 cargo` and `mise which cargo-nextest` return
  the fixture aliases. `rustup show home` and pinned Cargo/rustc version queries
  return deterministic fixture values. `rustup which` also supports `rustc`.
- `rustup run 1.98.1 cargo nextest list` copies the exact bytes from
  `VISIBILITY_FIXTURE_NEXTTEST_LIST`; `VISIBILITY_FIXTURE_NEXTTEST_LIST_EXIT`
  selects its exit status.
- `rustup run 1.98.1 cargo nextest run` copies the exact JSONL bytes from
  `VISIBILITY_FIXTURE_NEXTTEST_RUN`. `VISIBILITY_FIXTURE_NEXTTEST_EXIT` can
  return statuses such as 100 or 101. On Unix,
  `VISIBILITY_FIXTURE_NEXTTEST_SIGNAL=9` terminates the fixture with SIGKILL;
  unsupported values are rejected so the fixture cannot stop or ignore itself.
- Fake Cargo metadata reads `VISIBILITY_FIXTURE_CARGO_METADATA` and replaces
  `__WORKSPACE_ROOT__` and `__MANIFEST_PATH__` in JSON string values. A raw
  output file, exit status, and stderr file can be selected with the related
  constants in `fixture_env`.
- Fake Cargo build emits one machine-readable, non-test artifact event and a
  small synthetic executable under the requested target directory. The
  `VISIBILITY_FIXTURE_ARTIFACT_MODE` setting selects malformed, missing,
  duplicate, stale, wrong-target, and out-of-directory artifact probes.
- `VISIBILITY_FIXTURE_TRACE` appends one JSON object per invocation with
  `program`, `argv`, and `cwd` fields.

Fake version text and exit codes can be changed with the corresponding
`CARGO_VERSION`, `RUSTC_VERSION`, `RUSTC_VERBOSE`, `NEXTEST_VERSION`, and
`NEXTEST_VERSION_EXIT` constants in `fixture_env`. Metadata and build commands
also accept explicit stderr files and nonzero exit codes for failure-path
checks.

These fixture responses verify CLI orchestration and protocol handling only.
They are synthetic tool outputs. They do not build, run, measure, or qualify a
Termrock product binary and do not prove paired visual or interaction parity.
