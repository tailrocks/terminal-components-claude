# Termrock common live suite

This workspace stores the canonical live suite. Acceptance runs use one external export outside both application worktrees.

The runner captures exact PTY bytes before decoding. It pins `tui-test-rs` `0.1.1`, engine `438563a86a3c5d78e6da6e12511fa089c6f15c61`, and Tuiscotti `f40513120727f83c0c9713ae8d04eac70dfc955e`.

## First group

The frozen group contains four flows and twelve checkpoints. Every case uses a 120 by 40 PTY, a cleared child environment with `LANG=LC_ALL=C.UTF-8`, `TERM=xterm-256color`, and `COLORTERM=truecolor`, and a fixed working directory under `data/first`. `NO_COLOR` is absent. The app arguments and key actions are in `src/main.rs` and do not depend on a branch.

The runner requires all four staged binaries and a `source-manifest.json` in each `--bin-dir`. Generate the manifest after staging with `write-source-manifest`; it records a full source commit and tree, a clean source worktree, the successful build log, the build hash file, and the SHA-256 for each staged binary. The runner rechecks these records and hashes before it starts a child. The manifest binds staged bytes to the recorded source and build evidence. Keep the source worktree, log, and hash file. The manifest alone does not prove how a binary was built. Jackin is staged as `jackin`, copied from `jackin-preview` with the same SHA-256. The runner does not build application binaries.

Before quiet capture, the runner waits for a visible startup marker: `Overview` for Showcase, `holla❯` for Holla, `Current directory` for Jackin, and `TablePro` for TablePro. Jackin uses the returning scenario so the `?` key opens help from Manager. Escape restores Manager, and `q` exits. After `?`, Showcase waits for its source title `Keyboard & mouse`. After Escape, each flow waits for its startup marker again before the restored screen capture.

Create a source manifest with the verified commit, tree, source worktree, build log, and build hash file. The command creates metadata only and refuses to replace an existing manifest.

## Canonical export and run

Use one clean source commit and one external export for both application branches. Keep export, binary, application, and run-output roots disjoint.

Record `V` from the reviewed `export.py` Git blob. Record the printed `H` independently before either branch runs. Keep both values fixed.

Set `CANONICAL` to `$SOURCE_ROOT/tools/functional-suite`. Set `EXPORT` to a fresh path outside both application worktrees.
Obtain `V` without executing the blob:

```sh
git -C "$SOURCE_ROOT" cat-file blob "$FIXED_HEAD:tools/functional-suite/export.py" | shasum -a 256
```

```sh
trusted_export() {
  python3 -I -c '
import hashlib, pathlib, sys
p = pathlib.Path(sys.argv[2])
if not p.is_absolute() or ".." in p.parts or any(q.is_symlink() for q in (p, *p.parents)):
    raise SystemExit("untrusted helper path")
b = p.read_bytes()
if hashlib.sha256(b).hexdigest() != sys.argv[1]:
    raise SystemExit("canonical helper digest mismatch")
sys.argv = [str(p), *sys.argv[3:]]
exec(compile(b, str(p), "exec"), {"__name__": "__main__", "__file__": str(p)})
' "$V" "$CANONICAL/export.py" "$@"
}

trusted_export create --source-root "$SOURCE_ROOT" --source-commit "$FIXED_HEAD" --out "$EXPORT" --application-root "$REFERENCE" --application-root "$CANDIDATE" --bin-root "$BIN26" --bin-root "$BIN38" --run-output-root "$RUN_ROOT"
```

Copy `H` from the create output into the task record. Do not read a replacement seal from the export.
Keep `RUN_ROOT` existing and outside the export, binary roots, and application worktrees.

Before each guarded Cargo command, run authenticated preflight. Run Cargo from `EXPORT` with the locked offline command.

```sh
trusted_export verify --root "$EXPORT" --sha256 "$H"
(cd "$EXPORT" && TERMROCK_SUITE_EXPORT_SHA256="$H" MISE_ENABLE_TOOLS=rust,mr-boxington MISE_AUTO_INSTALL=false MISE_LOCKFILE=false /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo build --locked --offline)

trusted_export verify --root "$EXPORT" --sha256 "$H" --bin-dir "$BIN"
(cd "$EXPORT" && TERMROCK_SUITE_EXPORT_SHA256="$H" MISE_ENABLE_TOOLS=rust,mr-boxington MISE_AUTO_INSTALL=false MISE_LOCKFILE=false /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo run --locked --offline -- write-source-manifest --bin-dir "$BIN" --source-dir "$SOURCE" --source-commit "$COMMIT" --source-tree "$TREE" --build-log "$BUILD_LOG" --binary-sha256-file "$HASH_FILE")

trusted_export verify --root "$EXPORT" --sha256 "$H" --bin-dir "$BIN" --run-out "$RUN_OUT"
(cd "$EXPORT" && TERMROCK_SUITE_EXPORT_SHA256="$H" TERMROCK_LIVE_MODE=check MISE_ENABLE_TOOLS=rust,mr-boxington MISE_AUTO_INSTALL=false MISE_LOCKFILE=false /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo run --locked --offline -- --bin-dir "$BIN" --out "$RUN_OUT" --group first-plus-help)
trusted_export verify --root "$EXPORT" --sha256 "$H"
```

Repeat the final two commands for each branch and group. Change only `BIN` and `RUN_OUT` for a branch.
Keep `TERMROCK_LIVE_MODE` equal for both branches. Check mode is the only passing mode.
Capture and stability results remain unapproved. Missing or different references fail.
Never regenerate `Cargo.lock` or copy run output into `expected/`.
Reference approval stays independent and outside CI. Check mode never approves or writes a baseline.
Candidate A/B comparison reports `stable-unapproved` and exits nonzero when visible results match.

The runner keeps the existing `--bin-dir BIN --out OUT --group GROUP` interface. The first group remains four flows and twelve checkpoints.
The `first-plus-help` group remains five flows and eighteen checkpoints. `all` remains incomplete and fails.

The decoder uses the pinned engine with `TERM=xterm-256color`, `COLORTERM=truecolor`, the default xterm 256-color profile, and the app's `--color truecolor` option where specified. Snapshots retain each cell's exact grapheme, terminal width, foreground and background reference, attributes, underline, hyperlink, the resolved 256-entry palette, default colors, title, and cursor. `state-and-pixels` samples use the pinned Tuiscotti `tuiscotti-pinned-v1` render profile. Their PNG captures use the vendored JetBrains Mono styles and fixed 10 by 20 cell geometry at scale 1.

The selected group is process-complete only when every child exits with code zero, raw capture drains and flushes without errors, and all selected checkpoints are captured. Capture output remains unapproved and the command exits nonzero. A failed child or incomplete raw capture makes the run ineligible for reference approval. The runner records the cell-level difference, including the first differing cell or cursor field, when frozen comparison is available. ANSI byte streams may differ while decoded visible state matches.

## First plus Showcase Help 80x24

The `first-plus-help` group runs the same first four cases first. Their case IDs, sample IDs, expected paths, output paths, working directories, arguments, actions, and twelve approved references stay the same. It then runs `showcase-help/80x24-v1`. This selected group has five flows and eighteen checkpoints. It does not replace the first group or the full inventory.

The added Showcase case uses an 80 by 24 PTY and the same cleared environment, locale, terminal colors, paused motion, and frame zero as the other cases. It uses the same Showcase binary arguments and the fixed working directory `data/showcase-help/80x24-v1` on both branches. Its sequence is:

1. Wait for `Overview`, wait for visual quiet, and capture `initial`.
2. Press `?`, wait for `Keyboard & mouse` and quiet, then capture `help-start`.
3. Require the `Mouse` section to be hidden at `help-start`. Press `PageDown`, wait for the `Mouse` section and open help marker, then wait for quiet. Capture `help-scrolled` only if visible help text changed from `help-start`.
4. Press Escape, wait for help to hide and `Overview` to return, then capture `restored`.
5. Press `?` again and capture `help-reopened`. Require the same cells, attributes, colors, viewport, cursor, render contract, and pixels as `help-start`.
6. Press `q` in help. Require help and the quit dialog title to be hidden, `Overview` to remain visible, and the child to remain active. Wait for quiet and capture `restored-after-modal-q`.
7. Press `q` from the restored Overview. Require WaitExit to report code zero with no signal and require complete raw drain and flush.

The runner supports `--group first-plus-help` for this selected set. The `first` denominator remains four flows and twelve checkpoints. `all` still fails until the full inventory is implemented. Missing references and incomplete flows still fail check mode. Capture and stability results remain unapproved and return nonzero.

`--group all` is reserved for the full inventory and fails while that inventory is not implemented. The existing 302-screen claim is not a result of this suite. The `first-plus-help` group is not a replacement for the full inventory.

## Raw capture probes

Use the external export and its fixed seal for raw capture probes.

```sh
trusted_export verify --root "$EXPORT" --sha256 "$H" --run-out "$OUT"
(cd "$EXPORT" && TERMROCK_SUITE_EXPORT_SHA256="$H" MISE_ENABLE_TOOLS=rust,mr-boxington MISE_AUTO_INSTALL=false MISE_LOCKFILE=false /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo run --locked --offline -- probe-raw-capture --out "$OUT")
trusted_export verify --root "$EXPORT" --sha256 "$H"
```

These probes use real child processes. They check binary PTY output, processed byte offsets, forced incomplete drain, setup rejection, live-session capture rejection, captured restart rejection, and a regular-file write failure under `RLIMIT_FSIZE`. The probes do not count as application flows and do not change the four-flow or twelve-checkpoint denominators. The write-failure probe runs on Unix. It ignores and restores `SIGXFSZ` only in the probe process. PTY output newline processing is accounted for by using no carriage return or line feed in the exact-byte stream.
