# Bounded fresh-clone status-render case

Status: `PASS_NO_CARGO` preflight; **Rust case NOT_RUN**. No Cargo, Nextest, or Rust test was launched for this packet.

## Exact case and checkout

- Clone: `/private/tmp/termrock-vis01-status-fresh-clone-1d66f444-20261009/repo`
- Branch/commit/tree: `termrock-implementation` / `1d66f444f5962414b7c33734fa37376471e5f853` / `0ae9e285e8f771b8e250c63bf80f902d112dcd95`
- The clone is shallow and clean.
- Selected Rust test: `checked_in_records_render_from_exact_copies`, defined once in `crates/termrock-visibility-tests/tests/status.rs`.
- Exact filterset: `test(=checked_in_records_render_from_exact_copies)`.
- The crate declares `rust-version = "1.98.1"`; the runner uses that installed toolchain.

The test copies the production status and queue scripts, source facts, task records, `WORK_QUEUE.md`, every source archive manifest member, and each product23 checksum member into a disposable repository. It invokes the copied status CLI twice and compares the outputs. The test helper resolves the checkout root from `CARGO_MANIFEST_DIR/../..`; the fresh clone makes that root unambiguous. The test asserts the published report's status facts while preserving `Refactor / Ready | NOT_READY` and product23's 22 pass / 1 fail result. It does not run paired product visuals.

## Pinned child environment

- Supervisor: Node `v24.20.0`, `/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node`, SHA-256 `9d050fd455b56426e25d4d603c7c501cbb2630348e836cf221dcce748e90588a`.
- Cargo and rustc: Rust toolchain `1.98.1-aarch64-apple-darwin`; Cargo SHA-256 `6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d`; rustc SHA-256 `766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941`. Direct binaries report Cargo `1.98.1 (797e8a9bc 2026-08-05)` and rustc `1.98.1 (48a229cea 2026-09-01)`.
- `cargo-nextest` `0.9.146`, `/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest`, SHA-256 `7a558b157d164ab4fb6cb1a48cbac5a57b7b8ad99f5d3492eb6eda64faf91df0`.
- Status CLI Python: `/usr/bin/python3`, Python `3.9.6`, SHA-256 `34129c71a01a74f7f3b2443521519b2e5447553fa187f5fcafaaf8c42cc192e2`.
- `CARGO_HOME`: `/private/tmp/termrock-vis04-deferred-controls-b106-cache-repair-20261009-v7.Qs1c13/cargo-home`. Its pinned cache manifest SHA-256 is `64ca31c14e9d9cbbc8c6f6ce8ad69dc1014e6955669c3eae079a9a9a64190b71`; the standalone crate lock SHA-256 is `d6372475262b723ebdfe3bec9134ba6c675ff1d3becb3fa4625eb9e38c40b2c2`; preflight rehashed all 32 crate archives and 32 index entries.
- `CARGO_TARGET_DIR`: `/private/tmp/termrock-vis01-status-fresh-clone-1d66f444-20261009/target-r10`, initially absent and outside the clone.
- Private `HOME` and `TMPDIR` are under this packet's `home/` and `tmp/` directories. `PATH` starts with `/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin` and then fixed system directories.
- Other child settings: `CARGO_BUILD_JOBS=2`, `CARGO_NET_OFFLINE=true`, `RUSTUP_HOME=/Users/donbeave/.rustup`, `RUSTUP_TOOLCHAIN=1.98.1-aarch64-apple-darwin`, `GIT_OPTIONAL_LOCKS=0`, `CARGO_TERM_COLOR=never`, `NO_COLOR=1`, `TERM=dumb`, `TZ=UTC`.

The preflight Git commands set `GIT_OPTIONAL_LOCKS=0` so status checks do not refresh the clone index. The checked-in `.config/nextest.toml` has SHA-256 `688eecbcf066e5fefd6ae71cd711390cf245c04ce11a03928c187660b28b9b7e`. The runner pins it and passes `--ignore-default-filter`, then supplies the exact one-case filterset. A private temporary `HOME` prevents an ambient user Nextest config from changing selection.

## Bounded execution

Runner SHA-256: `21b1c22023c24cf364af0edebb76d1e966377d418f2208b898bb8296f9bb3c2e`.

The runner starts exactly one `cargo-nextest` child in its own process group and captures unmodified child stdout and stderr to separate external files. It uses one selected test, `--no-tests fail`, `--retries 0`, and `--test-threads 1`; Cargo offline and locked modes with two build jobs; a 600-second overall wall-clock limit, SIGTERM, SIGKILL after a 5-second grace period, and a second 5-second close deadline. If the child and its inherited output pipes still have not closed by that deadline, the runner closes its read streams, marks `close_deadline_exceeded`, and records infrastructure failure. Each child stream has an 8 MiB cap. The selected test's CLI helper has a 120-second per-command timeout and an 8 MiB per-stream cap. The supervisor does not retry or interpret a test result; it records the child exit code and stream hashes. If the outer limit or output cap is hit, it records that as an infrastructure stop and preserves partial raw streams.

## No-Cargo preflight

`preflight.json` SHA-256: `25466196c8b8a62c2a15d16da7e3034994b3cac1c25f84c19b9232d67584fb9b`. It reports `PASS_NO_CARGO`, the exact clean clone and source pins, one selected test, Nextest config and binary pins, Rust 1.98.1 binary hashes, the cache closure, and absent target/output directories. The preflight ran with the pinned Node supervisor and did not invoke Cargo, Nextest, or tests.

Before execution, rerun preflight and review its output:

```sh
/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node \
  /private/tmp/termrock-vis01-status-fresh-clone-1d66f444-20261009/run-r10/runner.cjs preflight
```

Only after Root grants the Cargo slot, execute the one-case runner from its versioned directory:

```sh
/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node \
  /private/tmp/termrock-vis01-status-fresh-clone-1d66f444-20261009/run-r10/runner.cjs execute
```

The runner creates one-shot output paths with exclusive-create semantics. It writes child stdout/stderr and `run-result.json` under `result/`; the invoking command does not use shell redirection that could truncate an existing file. The execute command has not been run. Preserve its first result and review raw Nextest output before making any report claim.
