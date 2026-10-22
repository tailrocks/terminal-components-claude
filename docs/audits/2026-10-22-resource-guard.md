# Rust build free-space guard

This increment adds an opt-in launcher for Rust work while the host is under
the 3 TB free-space policy:

```sh
python3 tools/rust_build_guard.py -- mise exec -- cargo nextest run --all-targets
```

The launcher checks the hardcoded threshold of 3,000,000,000,000 available
bytes on every distinct filesystem it can discover before it starts the
command. It checks the working directory, home and Cargo home, temporary
directories, Cargo output roots from environment and applicable Cargo config,
and MBX's cache and managed-target roots. Symlinks are resolved. Missing leaves
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
are inspected for `build.target-dir` and `build.build-dir`. A templated Cargo
build-dir is denied because its complete destination cannot be resolved without
Cargo's workspace context. `--target-dir` paths are added directly; all
`--config` options are rejected because the guard does not evaluate Cargo's
configuration merge rules. Configured or environment-provided
`rustc-wrapper`/`rustc-workspace-wrapper` is rejected because arbitrary
wrappers can change output placement. `CARGO_TARGET_DIR`,
`CARGO_BUILD_TARGET_DIR`, and non-templated `CARGO_BUILD_BUILD_DIR` are checked
as environment-provided paths. The launcher accepts the literal command
`mise exec -- cargo`; an absolute Cargo path is rejected to prevent bypassing
the MBX shim selected by `mise`.

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
