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

The source checkout was clean when inspected. The binary was built from
`crates/taskfmt` into an external install root and its `--version` output
matched the source commit. No old taskfmt checkout, fingerprint command, or
`experiment.toml` is an authority input.

## Current command split

The latest source has separate binaries:

- `taskfmt`: in-container `init`, `status`, `lint`, and `verify`.
- `taskfmt-host`: host catalog lint, self-tests, dispatch, gate, and promotion.
- `taskfmt-runtime`: container boot and agent launch.

The removed `taskfmt project`, `taskfmt group`, `taskfmt fingerprint`, and
`taskfmt progress-init` commands must not appear in live instructions.

Use these commands:

```sh
taskfmt lint refactoring-tasks/terminal-components/completion/[0-9][0-9][0-9]
taskfmt init --task-dir /absolute/catalog/terminal-components/completion/NNN --out /absolute/run/progress.md
taskfmt verify --root /absolute/work --task-dir /absolute/catalog/terminal-components/completion/NNN --base RECORDED_SCOPE_BASE_COMMIT --progress /absolute/run/progress.md --log-dir /absolute/run/taskfmt-logs
taskfmt-host lint --json
```

Do not pass `--config` to the latest `taskfmt` binary. Host lifecycle
commands are not used by this campaign unless the current campaign contract
explicitly adopts them; in particular, do not use a lifecycle command that
creates or promotes `main`.

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
/tmp/taskfmt-latest-install/bin/taskfmt --version
/tmp/taskfmt-latest-install/bin/taskfmt lint refactoring-tasks/terminal-components/completion/[0-9][0-9][0-9]
```

Rust validation for the task-format source uses `cargo nextest` only:

```sh
CARGO_TARGET_DIR=/tmp/taskfmt-latest-target cargo nextest run --manifest-path "$TASKFMT_SRC/Cargo.toml" --locked --workspace --all-targets
```
