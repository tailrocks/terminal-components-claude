# Task-format authority

This is the current taskfmt qualification record for the terminal-components
catalog. It is an operational reference, not proof that the campaign is ready
to execute. Current readiness is [`execution-readiness-report.md`](execution-readiness-report.md).

## Exact source

| Field | Value |
| --- | --- |
| Source checkout | `/Users/donbeave/Projects/taskfmt/task-format` |
| Branch | `main` = `origin/main` |
| Commit | `afd3b575dbcc7044620bec4b9493a74eca3e5ef2` |
| Version | `0.2.0` |
| Rust requirement | `1.98.1` toolchain / source `rust-version = 1.98` |
| Binary identity | `taskfmt 0.2.0 (git afd3b575dbcc7044620bec4b9493a74eca3e5ef2)` |
| Executable SHA-256 | `f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de` |

The source checkout was clean when inspected. The binary was built from
`crates/taskfmt` into an external install root and its `--version` output
matched the source commit. No old taskfmt checkout, fingerprint command, or
`experiment.toml` is an authority input.

The authoritative installer is [`scripts/campaign-install-taskfmt.sh`](../../scripts/campaign-install-taskfmt.sh).
It builds the qualified binary, verifies its source revision, exact
`--version` string, and SHA-256, then materializes one regular-file inode with
`nlink=1`. Preflight and dispatch consume the emitted `TC_TASKFMT` path and
repeat those identity and single-link checks.

## Current command split

The refactoring permits only the standalone `taskfmt` `lint` and `verify`
commands for per-task validation and verification.

Use these commands with `TASKFMT` bound to the qualified standalone binary:

```sh
TASKFMT=/absolute/path/to/qualified/taskfmt
"$TASKFMT" lint /absolute/catalog/terminal-components/completion/NNN
"$TASKFMT" verify --root /absolute/subagent-worktree \
  --task-dir /absolute/catalog/terminal-components/completion/NNN \
  --base RECORDED_SCOPE_BASE_COMMIT --progress "" \
  --log-dir <RUN_DIR>/taskfmt-logs
```

Do not pass `--config` or any container path mapping to the latest `taskfmt`
binary. Do not use `init`, `status`, host lifecycle, dispatch, promotion, or
container-runtime commands.

## Catalog result

The latest standalone binary linted all 73 numbered packages with zero errors
and zero warnings. Do not lint `completion/*` because that also passes the
group `README.md`, which is not a task package. The package schemas remain
`task/v5`, `verify/v2`, and `task-meta/v1`; task status remains `pending`.

Latest taskfmt validates package shape only. It does not prove command
availability, host isolation, dependency ancestry, visual-oracle integrity, or
product parity. Those remain separate readiness gates.

## Requalification commands

Build outside both repositories, then bind the exact source commit and binary
identity:

```sh
TASKFMT_SRC=/Users/donbeave/Projects/taskfmt/task-format
CARGO_TARGET_DIR=/tmp/taskfmt-latest-target cargo install --locked --path "$TASKFMT_SRC/crates/taskfmt" --root /tmp/taskfmt-latest-install --bin taskfmt
git -C "$TASKFMT_SRC" status --short --branch
git -C "$TASKFMT_SRC" rev-parse HEAD
TASKFMT=/tmp/taskfmt-latest-install/bin/taskfmt
"$TASKFMT" --version
"$TASKFMT" lint refactoring-tasks/terminal-components/completion/NNN
```

Rust validation for the task-format source uses `cargo nextest` only:

```sh
CARGO_TARGET_DIR=/tmp/taskfmt-latest-target cargo nextest run --manifest-path "$TASKFMT_SRC/Cargo.toml" --locked --workspace --all-targets
```
