# Termrock common live suite

This external workspace runs the same four live terminal flows against staged PR26 and PR38 binaries. It records exact PTY output bytes before decoding and associates each captured screen with the processed byte offset. Its own `[patch.crates-io]` uses a local path to the derived engine checkout. `build.rs` and the runner require the full engine commit `5994dbf57047fe41f7401aaaa9197d49b8dfc0ae` and a clean engine worktree. There is no registry fallback.

## First group

The frozen group contains four flows and twelve checkpoints. Every case uses a 120 by 40 PTY, a cleared child environment with `LANG=LC_ALL=C.UTF-8`, `TERM=xterm-256color`, and `COLORTERM=truecolor`, and a fixed working directory under `data/first`. `NO_COLOR` is absent. The app arguments and key actions are in `src/main.rs` and do not depend on a branch.

The runner requires all four staged binaries. Jackin must be staged as `jackin`, copied from `jackin-preview` with the same SHA-256. The runner does not build application binaries.

Before quiet capture, the runner waits for a visible startup marker: `Overview` for Showcase, `holla❯` for Holla, `jackin❯` for Jackin, and `TablePro` for TablePro. These markers exist in both pinned source commits. After `?`, Showcase waits for its source title `Keyboard & mouse`. After Escape, each flow waits for its startup marker again before the restored screen capture.

## Run modes

Set `TERMROCK_LIVE_MODE` explicitly for every run. Capture mode writes unapproved candidate artifacts and exits nonzero, even when all four flows complete:

```sh
TERMROCK_LIVE_MODE=capture cargo run --locked --manifest-path /Users/donbeave/Downloads/termrock-live-suite-20261027/Cargo.toml -- --bin-dir BIN --out OUT --group first
```

After independent reference approval places all twelve platform-qualified samples under `expected/first/...`, frozen checking uses the same command shape:

```sh
TERMROCK_LIVE_MODE=check cargo run --locked --manifest-path /Users/donbeave/Downloads/termrock-live-suite-20261027/Cargo.toml -- --bin-dir BIN --out OUT --group first
```

The mode is shared by both branches for a workflow. The only branch-specific command values are `--bin-dir` and `--out`. Candidate A/B comparison reports `stable-unapproved` and exits nonzero when all twelve visible samples match. Check mode is the only passing mode. It never approves or writes a baseline and returns failure for missing or different samples. Reference approval is a separate, explicit action outside CI; do not copy target samples into `expected/`.

Before the first `--locked` run, generate `Cargo.lock` with the guarded Cargo command without `--locked`, then commit the lock file. All Cargo builds and runs use the Tailrocks guard from this workspace directory. The public command can use `--manifest-path` in environments where the guard accepts it; local guard execution uses this workspace as cwd and equivalent Cargo arguments.

The guard rejects Cargo's `--manifest-path` argument. For guarded local execution, use the suite directory as cwd and omit only that argument; this selects the same manifest and lock file. The recorded local run command follows that cwd form.

The decoder uses the pinned engine with `TERM=xterm-256color`, `COLORTERM=truecolor`, the default xterm 256-color profile, and the app's `--color truecolor` option where specified. Snapshots retain each cell's exact grapheme, terminal width, foreground and background reference, attributes, underline, hyperlink, the resolved 256-entry palette, default colors, title, and cursor. `state-and-pixels` samples use Tuiscotti's pinned `tuiscotti-pinned-v1` render profile. Their PNG captures use the vendored JetBrains Mono styles and fixed 10 by 20 cell geometry at scale 1.

The first group is process-complete only when all four child processes exit with code zero, raw capture drains and flushes without errors, and all twelve checkpoints are captured. Capture output remains unapproved and the command exits nonzero. A failed child or incomplete raw capture makes the run ineligible for reference approval. The runner records the cell-level difference, including the first differing cell or cursor field, when frozen comparison is available. ANSI byte streams may differ while decoded visible state matches.

`--group all` is reserved for the full inventory and fails while that inventory is not implemented. The existing 302-screen claim is not a result of this suite. This first delivery does not run the full inventory.

## Raw capture probes

Run the raw capture probes with the same suite consumer and engine pin:

```sh
cargo run --locked -- probe-raw-capture --out OUT
```

These probes use real child processes. They check binary PTY output, processed byte offsets, forced incomplete drain, setup rejection, live-session capture rejection, captured restart rejection, and a regular-file write failure under `RLIMIT_FSIZE`. The probes do not count as application flows and do not change the four-flow or twelve-checkpoint denominators. The write-failure probe runs on Unix. It ignores and restores `SIGXFSZ` only in the probe process. PTY output newline processing is accounted for by using no carriage return or line feed in the exact-byte stream.
