# HP23 — Platform and terminal capability contract

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Cover every mapped macOS/Linux UI availability/fallback state using explicit fixture capabilities; prove actual preview color/terminal behavior on available test hosts. Make simulated platform behavior distinct from verified native execution.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Production OS adapters, native filesystem/Trash/open/process probes on both operating systems and full CLI parity.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve implemented macOS/Linux and terminal behavior while keeping explicit exclusions separate. Current platform fixtures do not cover legacy fallbacks.

**Source evidence and mandatory scope:** [matrix HP23](../parity/holla-parity-matrix.md#hp23--platform-and-terminal-capability-contract) — `X01`, `X02`, `X03`, `X04`, `X05`, `X06`, `X07`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide host-specific fixtures for every platform-sensitive matrix row: macOS casks/insights/dataless scanning/Spotlight/native Trash/open/reveal; Linux portable tools, hidden mac-only actions, FreeDesktop Trash, xdg-open fallback and process subreaper. Do not add apt/dnf/systemd-user as an old requirement. Do not call Windows, plugins, release infrastructure or nonexistent CLI strings parity gaps.

**Architecture / reusable components:** Platform capability adapter supplies explicit available/unavailable/error and native-path/store resolution. Preserve distinct XDG paths for frecency/sizes/logs versus dirs-based service/trust caches. Keep all effects simulated until production integration; no generic OS framework extraction without reuse. Apply Junie glyph/focus/no-color grammar to every added surface.

**Required deterministic fixture:** `parity-platforms` — macOS personal/development, Linux without desktop/trash mount helper, Linuxbrew, missing opener/OSC52, dataless policy failure, protected platform aliases, unavailable Spotlight and TERM/backend error.

**Acceptance / automated verification:** Assert all platform gates/fallbacks, no macOS command spawned on Linux, no destructive fallback, honest clipboard result and correct store location. At operational gate run controlled filesystem/PTY/Trash/open probes on both OSes; unavailable OS evidence remains explicitly pending, never passed by a macOS fixture. Run all relevant F21/F22/F23 terminal and provenance gates. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture platform availability/unsupported explanations in TrueColor/Mono/NO_COLOR; retain CLI/PTY/platform transcripts. No capture requirement for excluded CI or documentation-only features.

**Dependencies:** All HP01–HP22 platform cases; F21/F22/F23. This is a cross-cutting acceptance gate, not a new operating-system integration project.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-platforms` and `parity-platforms-linux` (`platforms_mac` and `platforms_linux` in src/bin/holla/domain/parity.rs) · `hp23_platform_capabilities_are_stated_and_never_faked` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: validation_denies_every_documented_rule` (`/tmp/junk` on `Os::Debian` refused, `/private/tmp/junk` on macOS), `src/bin/holla/domain/cleanup.rs: detection_respects_platform_tools_and_roots` (mac-only categories hidden on `Os::Debian`), `src/bin/holla/sim/fs.rs: trash_keeps_used_bytes_and_permanent_frees_them` ("Trash unavailable" leaves the file).

**What the journey asserts:**
- macOS: `Top files on this Mac` shows "did not finish within 5 s" (fixture `Spotlight::Timeout`).
- macOS: `upgrade.brew-casks` is an item; `platform.dataless_failure` contains "EPERM" (fixture "setiopolicy_np failed: EPERM").
- macOS: `validate("/private/tmp/x")` and `validate("/tmp/x")` both return `/private/tmp/x`.
- Linux: `validate("/tmp/x")` returns `/tmp/x` unchanged; no `upgrade.brew-casks` item; no item id starts with `cleanup.xcode`.
- Linux: `Review cleanup candidates` shows "Cargo registry cache" or "Project artifacts"; the gate reads "no Trash backend · every item will fail, never fall back".
- Linux gate 2 phrase "TRASH {n} UNDER /home/alex ON devbox"; report `count(Outcome::Failed) == r.items.len()`, every error contains "Trash unavailable", and `/home/alex/Projects/app/node_modules/x` still exists.
- Linux: `Copy this folder's path` shows "does not accept OSC 52" and `world.clipboard` is `None`.
- Linux: opening `package.json` from the Files actions menu shows "No opener on devbox" (fixture removes `xdg-open`, `opener: None`).

**Captures:** `shots/h_hp23_mac_top_files` ("Disk › Top files" with "did not finish within 5 s"), `shots/h_hp23_linux_gate` ("no Trash backend" fact on gate 1), `shots/h_hp23_linux_gate2` ("gate 2 of 2" with "TRASH 1 UNDER /home/alex ON devbox"), `shots/h_hp23_linux_report` ("Cleanup report", "failed", "Trash unavailable"); base matrix `shots/h_parity_platforms_{80x24,100x30,120x40,160x50,mono}` and `shots/h_parity_platforms_linux_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| X01 | not applicable (boundary row); the journey proves the Linux host states its own capabilities instead: no Trash backend ("Trash unavailable" on every item, `node_modules/x` survives), `/tmp` stays `/tmp`. | none |
| X02 | not applicable (boundary row); the journey proves platform statements only: "No opener on devbox", "does not accept OSC 52" with `clipboard == None`. | none |
| X03 | not applicable (boundary row); provenance is carried by the capture manifests, not by the journey. | none |
| X04 | not applicable (boundary row); the journey uses catalog labels (`Top files on this Mac`, `Review cleanup candidates`, `Copy this folder's path`), not CLI subcommands. | none |
| X05 | not applicable (boundary row); no browser filter or scanner option is claimed. | none |
| X06 | not applicable (boundary row); dry run is proven per plan in `hp21_*`, not as a global CLI flag. | none |
| X07 | not applicable (boundary row); the macOS world proves honest limits instead: Spotlight "did not finish within 5 s", dataless "EPERM", `/tmp/x` resolved to `/private/tmp/x`, and the HP21 gate names the remaining pathname race. | none |

**Deferred remainder:**
- Production OS adapters, native filesystem/Trash/open/process probes on both operating systems and full CLI parity (Later section).
- Acceptance clauses not proven by the tests: FreeDesktop Trash on a Linux host that has one, `xdg-open` fallback when present, Linuxbrew paths beyond `brew` in tools with `linux: true`, process subreaper behaviour (`subreaper: true` is fixture data only), `process_probe` `Err("pgrep: command not found")` is not asserted by `hp23_*`, store location (`xdg_config_home`, `xdg_cache_home`) is fixture data and not asserted, TERM/backend error state, TrueColor/Mono/NO_COLOR explanations are captured (`_mono`, base matrix) but not asserted, no macOS command spawned on Linux is proven only by item absence (`upgrade.brew-casks`, `cleanup.xcode*`).
- Controlled filesystem/PTY/Trash/open probes on a real Linux host remain pending; a macOS fixture does not pass them.

**Limits:**
- Both worlds run in the same test process on macOS; `Os::Debian` is a fixture value, so nothing about a real Linux kernel, desktop or Trash mount is measured.
- Timeouts, EPERM and missing tools are fixture strings, not observed syscalls.
