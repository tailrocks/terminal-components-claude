# Common live stream record

- Task: create the canonical four-flow live runner and exercise the frozen raw PTY capture API.
- Initial task start: exact time was not recorded. Do not infer it from a commit timestamp.
- Current work observation: 2026-10-26 23:27:36 UTC; suite workspace setup started.
- Current stage: canonical suite committed next; guarded PR26 reference capture A is next. Engine fork `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff` is frozen for independent review.
- Owned writes: this new suite workspace; `/Users/donbeave/Downloads/termrock-live-results-20261027/streams/common` and the suite result directories. No application source, engine source, or upstream source is owned now.
- Blocked periods: none recorded. Earlier exact blocked start and end times were not captured.
- First consumer build attempt: guarded `cargo build`, exit 101 before compilation. Cargo could not fetch the unpushed derived commit from GitHub. This is preserved as a setup-only failure; the suite patch now uses the local engine checkout and verifies its exact commit and clean status before build and run.
- Second consumer build: guarded `cargo build`, session 99277, exit 101 at `engine.rs:2073` with E0283. The explicit `TuiTestError::from` conversion was added in commit `9ff17bb7a7b2dc446ef1064cc8e4c0a8307934ff`. No branch binaries ran during either failed build.
- Corrected consumer build: guarded `cargo build`, started at 2026-10-26 23:38:15 UTC; exit 0. Cargo reported target build time 10.27 seconds. Resolved `tui-test-rs v0.1.1` path: `/Users/donbeave/Downloads/tui-test-raw-capture-20261027/crates/tui-test`. Tuiscotti resolved at its exact pinned revision. One dead-code warning on existing `Session::spawn_with` remains.
- Suite lock SHA-256: `1e4d4b06a1cf31756925e42156fd71d3674a8acc2e8a9fa3d8cf563f60470062`.
- Final full base-to-head engine diff artifact: `/Users/donbeave/Downloads/termrock-live-results-20261027/streams/common/tui-test-raw-capture-9ff17bb.patch`, SHA-256 `35bf7fee6faf6e2b42c564c27d230bacc3e7f0eb068ebbc6b60bc4a71d9b3c21`.
- Live-process time: no child process has run yet. Build time and live-process time remain separate.
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
