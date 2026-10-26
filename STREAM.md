# Common live stream record

- Task: create the canonical four-flow live runner and exercise the frozen raw PTY capture API.
- Initial task start: exact time was not recorded. Do not infer it from a commit timestamp.
- Current work observation: 2026-10-26 23:53:51 UTC; correcting source-backed help transition handling after PR26 A1.
- Current stage: rerun PR26 A with a corrected Showcase help title and without the extra initial-versus-restored cursor equality gate. PR26 A1 proved the fixed Jackin `?` action cannot leave the first-use Intro route. Engine `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff` remains frozen for independent review.
- Owned writes: this new suite workspace; `/Users/donbeave/Downloads/termrock-live-results-20261027/streams/common` and the suite result directories. No application source, engine source, or upstream source is owned now.
- Blocked periods: none recorded. Earlier exact blocked start and end times were not captured.
- First consumer build attempt: guarded `cargo build`, exit 101 before compilation. Cargo could not fetch the unpushed derived commit from GitHub. This is preserved as a setup-only failure; the suite patch now uses the local engine checkout and verifies its exact commit and clean status before build and run.
- Second consumer build: guarded `cargo build`, session 99277, exit 101 at `engine.rs:2073` with E0283. The explicit `TuiTestError::from` conversion was added in commit `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff`. No branch binaries ran during either failed build.
- Corrected consumer build: guarded `cargo build`, started at 2026-10-26 23:38:15 UTC; exit 0. Cargo reported target build time 10.27 seconds. Resolved `tui-test-rs v0.1.1` path: `/Users/donbeave/Downloads/tui-test-raw-capture-20261027/crates/tui-test`. Tuiscotti resolved at its exact pinned revision. The warning diagnostic is `associated function `spawn_with` is never used` at `session.rs:576`; the symbol exists in the pinned base, but warning provenance is unknown because the base was not built. Independent source review will classify it.
- Corrected suite build after the A0 harness fixes: guarded `cargo build`, session 27685; exit 0. Cargo reported target build time 3.02 seconds. The exact start time was not recorded. This build ran no child process. The compiler emitted the same warning listed above.
- Suite lock SHA-256: `1e4d4b06a1cf31756925e42156fd71d3674a8acc2e8a9fa3d8cf563f60470062`.
- Final full base-to-head engine diff artifact: `/Users/donbeave/Downloads/termrock-live-results-20261027/streams/common/tui-test-raw-capture-9ff17bb.patch`, SHA-256 `35bf7fee6faf6e2b42c564c27d230bacc3e7f0eb068ebbc6b60bc4a71d9b3c21`.
- A0 candidate attempt: exact command was `TERMROCK_LIVE_MODE=capture MISE_ENABLE_TOOLS=rust,mr-boxington MISE_AUTO_INSTALL=false MISE_LOCKFILE=false /Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo run --locked -- --bin-dir /Users/donbeave/Downloads/termrock-live-bins-20261027/pr26 --out /Users/donbeave/Downloads/termrock-live-results-20261027/suite/pr26-a --group first`, from the suite cwd. Session 61265; exit 1; run result `failed`, 0/4 flows and 0/12 checkpoints. This is a harness setup failure, not application behavior. The harness observed visual quiet before waiting for an initial visible marker, and the pixel sample ID lacked Tuiscotti's required `/pixels-<platform>` suffix. The preserved A0 output is not overwritten.
- PR26 A1 candidate: suite `81b2cec2dfb0afad867701d0980d85810f20c73c`, session 99846, exit 1, elapsed 62.186 seconds. Command was the A0 command with output `/Users/donbeave/Downloads/termrock-live-results-20261027/suite/pr26-a1`. Result: `failed`/`incomplete`, 1/4 process-complete and 7/12 checkpoints; 0 flows/checkpoints passed. Holla completed all three checkpoints, exited 0 with no signal, and drained/flushed 40,752 bytes. Showcase captured only initial before the wrong help marker (`Keyboard shortcuts`) timed out; its raw stream contains the actual visible dialog title `Keyboard & mouse`. Jackin captured only initial; PR26 `Route::Intro` consumed `?` and only Enter/Escape skip that route. TablePro captured initial and help, then an extra self-comparison rejected a cursor column difference; this unrequested gate is removed for the next run. Showcase, Jackin, and TablePro raw statuses after `Session::close` were incomplete, undrained, and unflushed; their child exits were not proven. Full result: `/Users/donbeave/Downloads/termrock-live-results-20261027/suite/pr26-a1/run-result.json`. No frames were approved.
- Marker checks against the pinned source commits: PR26 `src/bin/showcase/app.rs:160`, `src/bin/holla/app.rs:58`, `src/bin/jackin_preview/app.rs:46`, and `src/bin/tablepro/app.rs:1951`; PR38 `crates/showcase-ui/src/app.rs:153`, `crates/holla-ui/src/app.rs:333`, `crates/jackin-preview-app/src/app.rs:8658`, and `crates/tablepro-ui/src/app.rs:7762`. Each initial screen contains the marker used before visual quiet.
- Live-process time: A0 child time was 2.835 seconds. A1 aggregate process time was 62.186 seconds; its Holla child completed in 9.316 seconds. A1 did not produce a complete capture run. Later child and build times remain separate.
- Disk observation before suite workspace creation: `disk-check` reported 2,390,041,300,992 free bytes (`status=ok`, above the 2 TB cleanup gate).
- Build time and live-process time will be recorded separately after execution.

## Frozen inputs

- Reference source: PR26 `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.
- Candidate source: PR38 `573a595e97451646cdd910ec392274c6a425f23b`.
- Tuiscotti: `9de60de9d2b9e883e5901dad07ecee508ac6dd76`.
- `tui-test-rs`: registry version `0.1.1`, features `recording-raster` and `recording-font-jetbrains-mono-styles`, patched by this workspace to the frozen derived engine commit above.
- Engine base: `81da26fa9dc29664a897dc802be816a4432c0618`.
- Final engine commit: `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff` (`Fix snapshot conversion inference`), after the raw capture commit `04fddc63ba46acac4c32a894c1c502851af4b9bf`.
- Local source override: `/Users/donbeave/Downloads/tui-test-raw-capture-20261027/crates/tui-test`; `build.rs` and the runner reject any engine HEAD other than `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff` or a dirty worktree.

## First group declaration

- Fixed flow denominator: 4.
- Fixed checkpoint denominator: 12, three per flow (`initial`, `help`, `restored`).
- Cases: `first/showcase`, `first/holla`, `first/jackin`, `first/tablepro`.
- Full parity inventory and the existing 302-screen claim: not run and not fresh evidence.
