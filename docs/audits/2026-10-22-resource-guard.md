# Rust build free-space guard

This audit records the reviewed free-space guard installed for agent Rust
commands while the host is under the 3 TB free-space policy. The installed
entry point is `~/.local/bin/rust-build-guard`:

```sh
rust-build-guard -- mise exec -- cargo nextest run --all-targets
```

The launcher checks the hardcoded threshold of 3,000,000,000,000 available
bytes on every distinct filesystem it can discover before it starts the
command. It checks the working directory, home and Cargo home, temporary
directories, Cargo output roots from environment and applicable Cargo config,
and MBX's cache and managed-target roots. The default Cargo output directory
`<workspace>/target` is checked even when it is missing or redirected by a
symlink. Symlinks are resolved. Missing leaves
are checked against their closest existing ancestor. A filesystem is counted
once using its device identity, and free bytes are computed as
`f_bavail * f_frsize`. Unknown paths, invalid filesystem values, unparseable
configuration, or stat failures deny execution. The threshold cannot be
lowered through an environment variable or function argument.

The guard resolves the project environment with `mise env --json` in memory;
it does not print or persist that JSON. It then executes the original command
with the original process environment and unchanged argument vector. MBX roots
follow [MBX configuration precedence](https://mr-boxington.jdx.dev/configuration):
`MBX_CACHE_DIR` and `MBX_TARGET_ROOT`, the platform global configuration, and
documented platform defaults. On macOS the global config is
`~/Library/Application Support/mbx/config.toml`; on Linux it is
`$XDG_CONFIG_HOME/mbx/config.toml` (or `~/.config/mbx/config.toml`); on Windows
it is `%APPDATA%\mbx\config.toml`. The target root defaults to
`<cache_dir>/targets`. Relative cache roots are rejected as ambiguous; relative
managed target roots are resolved under the cache root as documented. See also
the [MBX repository](https://github.com/jdx/mr-boxington).

Cargo config files in the working directory's ancestor chain and `$CARGO_HOME`
are inspected for `build.target-dir` and `build.build-dir`. Config `include` and
`env` tables are rejected because their indirect effects cannot be evaluated
safely. A templated Cargo build-dir is denied because its complete destination
cannot be resolved without Cargo's workspace context. `--target-dir` paths are
added directly; `--config` and `--manifest-path` options are rejected because
they can change path resolution beyond what this small guard evaluates.
Configured or environment-provided `rustc-wrapper` and
`rustc-workspace-wrapper` (including Cargo's `CARGO_BUILD_*` aliases) are
rejected because arbitrary wrappers can change output placement.
`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR`, and non-templated
`CARGO_BUILD_BUILD_DIR` are checked as environment-provided paths. The launcher
accepts the literal command `mise exec -- cargo`; an absolute Cargo path is
rejected to prevent bypassing the MBX shim selected by `mise`. The accepted
Cargo subcommands are `build`, `check`, `clippy`, `fmt`, `doc`, and `run`, plus
`nextest run` and `nextest list`. Only the first listed subcommand may appear
immediately after `cargo`; global options before it are denied. Unknown
commands (including aliases such as `low = "build --target-dir …"`) are denied,
and any accepted command configured as a Cargo alias (in `[alias]` or
`CARGO_ALIAS_<COMMAND>`) is denied. This includes the native nextest
subcommand. This deliberately avoids interpreting Cargo's full alias and
install command grammar.

## Limits and verification

This is an invocation guard, not an OS-wide quota. It cannot prevent a process
from changing storage roots after the check, launching Cargo outside the guard,
or writing arbitrary files from build scripts. It does not model container
quotas or reserved blocks beyond the OS-reported `f_bavail`. Users must route
Rust commands through this launcher and the pinned `mise exec -- cargo` path.

`PYTHONDONTWRITEBYTECODE=1 python3 tools/test_rust_build_guard.py` covers the
threshold boundary, multiple devices, symlinks and nonexistent leaves,
filesystem errors and invalid values, ignored threshold overrides, preserved
argv/environment, Cargo config roots, and a real deny probe that stops before
`cargo --version`. These Python checks do not build Rust or attest to consumer
tests, CI, or completion of the 3 TB free-space gate.

## Installation and scope receipts (2026-10-22)

The earlier opt-in/design-only description was accurate before global
installation; this receipt section supersedes that historical status. Git
history retains the prior wording.

The reviewed source commit was `09a182b0bf865e1b241bcdf7742ab2de45ae718b`;
the source script SHA-256 is
`48b606262b4e8a5b485770532fef9175a1318118e9f789dc5bf8a12b34f9dd9b`. The
installed executable at `/Users/donbeave/.local/bin/rust-build-guard` has the
same SHA-256, UID 501, and mode 0755. Four global AGENTS/CODEX_POLICY files
received the guard requirement; the installation receipt records their before
and after hashes, verifies each original prefix, and records one marker per
file. The `mise exec -- command -v cargo` check resolved Cargo to
`/Users/donbeave/Library/Application Support/mbx/bin/cargo`, the MBX shim.

The installed launcher denied its harmless child-execution probe with status
125 at a recorded 2,818,665,132,032 available bytes; the receipt records zero
child exec calls. This demonstrates the denial path only. It does not establish
that the required 3,000,000,000,000-byte free-space threshold has been met.
No Rust build ran, and Rust Analyzer has not been configured or verified. The
guard does not control Rust Analyzer, direct shell/GUI Cargo invocations,
foreign Rust processes already running, or invocations that bypass the
launcher. It is process-level routing guidance and is not OS-wide enforcement.

Receipts are preserved outside the repository:

- `resource-guard-global-install-20261022.json` — SHA-256
  `832f1d4c7c86b15e65df95dde9c7fdccbbac0622bc97dcf6b5b716ba229e29b1`
- `resource-guard-global-verification-20261022.json` — SHA-256
  `c018253e6609e54d5b629c5096115291fd1c8b9814efdffd93dcb7a007ecee95`
