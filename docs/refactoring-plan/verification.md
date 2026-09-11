# Deterministic parity verification contract

## Authority and measured tool state

The user-visible oracle is exactly `02f5294bfdbf38004cc49130d0aff1d01f31434c`, the peeled `holla-fable-2026-09-10` tag. `git rev-parse 'holla-fable-2026-09-10^{}'` reproduced that SHA on 2026-09-11. The architectural candidate inspected here is `7b27732a`; the coordinator's topology evidence supplies its complete SHA. The working `holla` checkout is not the oracle.

`git ls-remote https://github.com/donbeave/tui-snap.git refs/heads/main` returned `5036cf87e621e6beb66deffe3224abdbefc955cb`. README, USAGE, CI, MIGRATION, Cargo manifest, frame adapters, PTY wrapper, store implementation and tests were inspected at that revision. The crate name is `tuisnap`, version 0.2.0. Its Ratatui dependency is 0.30, matching the oracle's Ratatui generation. Main uses `ratatui-core` 0.1.2; the external verification harness may depend on Ratatui 0.30 but must not reintroduce that facade into the backend-free production crate.

Current upstream has real reusable capabilities; it also has concrete fidelity defects. The existing main-side `tools/qualified-capture/pins.json` already identifies three repaired tool commits ending at `e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2`, tree `e87f3fe3ae49d8066afe8ab7b319f319c2a1dd0f`. They are stored in a verified Git bundle, not merged upstream. Their authorship, DCO signoffs and Codex trailers are preserved. A current GitHub query initially found no PRs and only upstream `main`; publication status and final rerun evidence are recorded in the companion tool evidence document.

Required tool dependency is now [tui-snap PR #1](https://github.com/donbeave/tui-snap/pull/1), pinned head `883d03f19d890bbbf27468798db78b04e85297ac`, tree `dadbaa70facc317cfabb52f0374c1f3cdceb46a1`. It includes the existing repairs plus independently reviewed corrections for downstream dependency packaging, DECAWM serialization, concealed SVG spacing and reproducible reference migration. All 51 all-feature tests, 41 no-default tests including doctest, explicit doctest, formatting, clippy, consumer and migration checks pass. The independent 384-cell PTY proof passes. PR remains unmerged; pinning its reviewed head is permitted, while using moving upstream main is not equivalent. Detailed measured evidence is in [tuisnap-assessment.md](evidence/tuisnap-assessment.md).

## Existing capability assessment

| Requirement | Existing API and proof boundary |
| --- | --- |
| Production component/view frames | `ratatui::widget_frame`, `draw_frame`, `capture`, `from_buffer`; render real widgets/app view into `TestBackend` or convert the real published buffer. Never render an imitation. |
| Exact visible state | `Frame::validate`, `diff_cells`, `digest`; schema, dimensions, row-major cells, graphemes, display widths, continuation cells, colors, modifiers and cursor. Gate exact cell comparison, not digest alone. |
| Full executable | `pty::Session::spawn(&argv, &PtyOptions)` owns a real PTY and kills/reaps the process on drop. |
| Keyboard | `send_key` includes arrows, Tab/backtab, Home/End, PageUp/PageDown, Insert/Delete, F1–F12, Ctrl and Alt; `type_text` sends arbitrary bytes representable as UTF-8, including raw terminal sequences. |
| Editing and paste | `type_text`, `paste`; repaired `paste_literal` preserves LF and rejects bracket delimiters, requiring bracketed-paste mode. Multiline editing must use literal paste, not the older newline-converting helper. |
| Mouse | `click` and `drag` use zero-based coordinates. Raw SGR via `type_text` supplies move, separate press/release, right click and wheel; no new mouse library primitive is required. |
| Resize | `resize(cols, rows)` changes PTY geometry. Assert application reflow after processing resize, not merely `snapshot` geometry. Use canonical dimensions at most 512; the PTY's larger engine limit is not the frame import limit. |
| Readiness and settlement | `wait_for_text`, `wait_stable(Duration)`, `wait_idle`, `wait_exit`; bounded timeouts fail with screen evidence. `wait_stable` observes styles and cursor as well as text. |
| Fail-closed artifacts | `Store::check(..., 1.0)` writes actual JSON/PNG first; `ensure_matched` fails missing approval, corrupt approval, dimension change, cell change or strict pixel mismatch. `check` never writes under `approved`. |
| Review aids | Canonical frame JSON regenerates PNG/HTML expected/actual/diff reports using pinned profile and vendored font. Rendering approximations and missing-glyph tofu do not replace authoritative cell equality. |

The repaired tool uses schema 3. Schema 2 is rejected, not silently upgraded. It preserves hidden/blink flags, continuation styles, combined bold+DIM, strike in raw replay, autowrap clipping and physical cursor coordinates. DIM arithmetic uses full precision before narrowing. Raw ANSI replay still lacks cursor appearance; use direct PTY cursor capture for style/blink and direct production cursor intent for component fixtures. Blink phase is not a frame field: animation scheduling and rate belong to deterministic transition assertions.

Protocol examples for a zero-based `(x, y)` are `ESC[<35;x+1;y+1M` for hover, `ESC[<64;x+1;y+1M`/`65` for vertical wheel, `ESC[<0;x+1;y+1M` for press and the same ending in `m` for release. Record decoded logical action and exact bytes. Keep a press checkpoint before release, otherwise pressed-state regressions disappear. Assert effects after pointer movement; checking transmitted bytes alone proves only transport.

## Historical infrastructure: reuse and limits

The oracle has real app `TestBackend` drivers in `src/bin/{showcase,holla,jackin_preview,tablepro}/app_tests*.rs`, direct input dispatch, deterministic tick stepping and app fixtures. Holla and Jackin expose `--scenario`, `--motion paused`, `--frame` and palette flags. Preserve their deterministic domain worlds; do not substitute real providers. Showcase and TablePro use wall `Instant` for some transient status/flash behavior. Their test drivers already provide a useful narrow seam for production rendering and input.

Oracle `tools/capture.sh` handles tmux keys, SGR mouse, resize and captures ANSI/text/cursor/PNG/HTML with source/binary/font manifests. Fixed sleeps, system rasterizer dependence and tmux normalization make those captures supporting evidence, not complete deterministic acceptance. Oracle `tests/holla_pty.rs` proves fresh-process color policy and input-flood fairness; `tests/terminal_suspend.rs` proves terminal ownership restoration. Keep those semantic gates.

Main has `crates/tui-testing::{Harness, Scene, Conformance}`, runtime `deliver`/`deliver_buffer`, `xtask` parity/build/confinement/wiring checks, replay recipes, source/binary provenance binding, and qualified tooling. Reuse lifecycle publication semantics: settle, draw, commit presented, then deliver input. Never bypass `PendingInput`, geometry publication or focus settlement just to make a fixture pass.

Main's 499 historical recipes belong to the older frozen archive at `c12cad8728755cd2d03eefdd8e02891143fca86d`, using source `d5e7075f436f0e437c7d12cf3d1e638e763b26f6`. `tools/historical-render/README.md` explicitly says those regenerated images are not pinned-Holla snapshots. Preserve the archive and its provenance; do not relabel it as the new oracle. It supplies scenario/history coverage only until each recipe is reconstructed against the new tag. The current `xtask/src/parity.rs` recipe switch covers Showcase, TablePro and Jackin, not Holla. Holla requires its own complete catalog.

## Two capture paths, one scenario contract

Every scenario has a stable ID from the application/component matrices, exact fixture, viewport, palette/theme, initial state, ordered actions and checkpoint IDs. For each checkpoint store the complete canonical frame plus semantic observations needed to distinguish visually identical states: focus target, active edit/caret/selection state, scroll offsets and boundaries, selected identity, overlay stack/capture owner, routing outcome, model mutation and exit/navigation outcome. Normalize architectural identity to oracle-derived semantic identity; do not compare internal pointer addresses or invent new UX semantics.

Direct tests drive actual production event handlers and rendering. App adapters may translate the old versus new API, but must not duplicate rendering, business logic or hit testing. Reference adapters are separately reviewed and trusted. Compare initial frame and every observable transition, including unchanged-frame assertions for ignored actions. PTY tests then prove real input decoding, transport, runtime ordering, terminal setup/cleanup and executable routing for representative paths in every interaction family. A TestBackend result cannot stand in for PTY input correctness; a PTY screenshot cannot stand in for hidden state correctness.

Suggested direct capture sequence using existing APIs:

```rust
let frame = tuisnap::ratatui::capture(&mut actual_production_terminal, provenance);
frame.validate()?;
let expected = tuisnap::Frame::from_json(&trusted_reference_json)?;
assert!(frame.diff_cells(&expected)?.is_empty());
store.check(scenario_checkpoint_id, &frame, &profile, pinned_font, 1.0)?
    .ensure_matched()?;
assert_eq!(actual_semantic_observation, trusted_semantic_observation);
```

Use separate stores for reference/view, candidate/view, reference/PTY and candidate/PTY. Compare like capture paths. Their shared schema does not guarantee that buffer reset colors and terminal defaults have identical provenance or that a TestBackend cursor contains hardware cursor appearance. A cross-path equivalence probe must establish any deliberate adapter correspondence; never silently weaken fields to force agreement.

## Clock, environment and deterministic data

The trusted scenario runner owns logical time, fixture tick counts, event ordering and random seeds. Holla/Jackin `--motion paused --frame N` define frozen rendering; action sequences needing progress use exact production tick calls and then freeze a checkpoint. Keep full/reduced/paused motion as distinct scenario axes. Do not freeze away animation requirements: sample every distinct animation phase and assert phase transition boundaries.

For `Instant` consumers, a reviewed reference-only adapter changes only the time-source boundary in a disposable checkout of the immutable tree. Preserve original conditions and durations. Candidate production code supplies the same logical instants through its accepted clock seam. Assert immediately before, exactly at and after expiry; include Showcase busy 2200 ms, status `> 4 s`, flash `>= 140 ms` and each application-specific deadline discovered in the inventory. A fixed sleep is not an exact timing oracle. Never mask transient regions, erase status text, replace state with a hand-crafted expected frame or let candidate time select the expected reference.

The concrete adapter and boundary proof are retained in [oracle-clock-adapter.patch](evidence/oracle-clock-adapter.patch) and [oracle-clock.rs](evidence/oracle-clock.rs). Against exactly the oracle tree, add `oracle_clock.rs` beside Showcase's `main.rs`; TablePro's sibling clock module contains `include!("../showcase/oracle_clock.rs");`, then apply the patch. The only substitutions in pre-existing code are the `Instant` imports in `src/bin/showcase/app.rs` and `src/bin/tablepro/app.rs`, three fully qualified Instant references in Showcase `pages/buttons.rs`, and two in `pages/forms.rs`; root modules include the adapter. Duration literals, comparison operators, `Input::Tick`, rendering and event dispatch bodies remain byte-for-byte unchanged. The injected test modules are clearly named `planner_clock_probe`.

The clock is thread-local `Duration` state with `Instant::now`, saturating `elapsed`, checked `Add<Duration>` and derived ordering/equality. Every independent scenario resets time before constructing state. No wall sleep changes logical time. This matches all Instant operations actually used by those four production files; the library event loop's wall clock and performance measurements are not substituted. Rerun `cargo test --locked --bin showcase --bin tablepro planner_clock_probe`: all four production-handler tests pass at 139/140 ms flash, 4000/4001 ms Showcase status, 5000/5001 ms TablePro status, 2199/2200 ms busy button and 1799/1800/1801 ms form submission. Running both entire binary suites on the adapted tree passed all 80 tests (76 existing plus four probes). Reverse the allowlisted substitutions and remove only the named injected modules/root includes to require original blob equality; source-hash drift fails adapter qualification.

The regular CLI lane builds the untouched oracle source and runs real PTY keyboard/mouse/resize and stable terminal outcomes. Exact deadline frames use the proven in-process production-handler lane; PTY elapsed wall time never claims exact 139/140 ms parity. Any future controlled PTY executable is a separate test driver sharing the same production application, not a replacement for the untouched CLI lane. Its command protocol, if introduced for a scenario, must map `time(n)` only to the demonstrated clock setter and use the existing terminal decoder for user input. This is application harness work using existing tuisnap APIs, not a missing general tuisnap primitive.

PTY stable surfaces run with the existing paused fixture modes. Transient PTY cases require a test driver that selects logical production ticks before presentation and exposes a readiness acknowledgement; its only added behavior is clock/input orchestration. Gate driver equivalence against unmodified executable decoding and rendering. Both reference and candidate use the same trusted action script. If a surface cannot yet support this seam, its scenario remains required and failed/unimplemented, not skipped.

Record Rust toolchain and lock hash, target triple/OS image, capture tool commit/tree, terminal engine lock, profile JSON and font SHA-256, source tree/adapter hashes, binary hashes, fixture hashes, scenario manifest hash, argv, initial dimensions and every resize. Use isolated temporary working/data directories; scrub inherited `NO_COLOR`, `CLICOLOR_FORCE`, `FORCE_COLOR`, app motion overrides and build overrides, then set explicit `TERM=xterm-256color`, `COLORTERM=truecolor`, locale, timezone and scenario-specific environment. Pin `LINES`/`COLUMNS` initially; observe actual PTY size on resize. Run color-policy tests in fresh processes for explicit truecolor/256/16/none and NO_COLOR absent/empty/nonempty; do not let a global scrub erase those dedicated cases.

All domain data stays deterministic and isolated from network, user files, Git status, Docker, databases, providers and clipboard side effects. Existing simulated application worlds are the source, including their failures and empty states. Clipboard/terminal lifecycle requirements use fake providers or owned PTYs plus explicit side-effect assertions.

## Coverage closure

The scenario manifest is a required set, not a best-effort glob. Its IDs join bidirectionally to component/application matrices, historical obligations and owning task IDs. Required checkpoint dimensions are 60x18, 72x20, 80x24, 100x30, 120x40 and 160x50 where that viewport exercises a distinct breakpoint; each family has standard and narrow proof, and screen layout branches add their exact threshold minus one/at/plus one. Retain existing historical viewport cases. Cover every supported palette and theme at least once per reusable visual family, with additional combinations where a branch depends on theme/palette.

Each interactive family covers default/focus/hover/pressed/selected/active/inactive/disabled/empty/populated/error as applicable. An explicit evidence-backed not-applicable row replaces impossible combinations; absence does not. Every scrolling family covers top/middle/bottom, attempted overscroll, nested routing, thumb drag, wheel over child, resize anchoring and scroll-edge fades. Fade proofs include heights 3/4/11/12, 55%/80% rounded RGB blends, non-RGB DIM behavior, alternate backgrounds, reverse, cursor and protected current-row exemptions. Editing covers focus without edit, first pointer click at grapheme, same-cell repeated click, read-only caret movement, selection direction, Unicode/wide/combining content, paste, deletion and horizontal viewport/cursor movement. Overlay tests assert capture, click-through prevention, Escape precedence, nested stack order and restored focus. Do not collapse separately observable states into one screenshot.

## Trusted evidence and tamper resistance

The first execution gate assembles reference fixtures using the immutable source and reviewed adapters before component implementation begins. That gate cannot approve any candidate output. Its acceptance must include repeat capture equality, independent adapter review, required-set completeness, canonical validation, source/binary bindings and negative probes. Publish the oracle bundle and manifest from the planner/reviewer trust domain. Each downstream task receives it through task-format's current trusted verification mechanism; the task-format owner supplies the exact schema fields.

Expected frames, semantic observations, action recipes, required IDs, comparator source/binary, profile/font pins and oracle source SHA belong outside executor writable scope. Filesystem read-only flags alone are insufficient when executor owns the files. A trusted runner materializes them from the pinned bundle, verifies cryptographic hashes before and after execution, and compares only candidate artifacts copied into a separate output directory. The implementation checkout contains references to that trust root, not an editable definition of success. Reject symlinks, substituted paths, duplicate IDs, missing artifacts, unexpected approvals and stale candidate binary/source hashes.

Do not invoke `tuisnap accept` in implementation tasks or CI. Do not permit environment blessing, rewriting expected JSON, altering `pixel_threshold`, regenerating from candidate code, changing reference SHA, deleting scenarios, filtering failures or replacing the trusted verifier. `Frame::digest` is a diagnostic identifier, not a cryptographic integrity check. Use SHA-256 for bundle/source/artifact integrity and exact parsed frame equality for UX.

Required negative qualification: mutate glyph; foreground/background; DIM/reverse; wide continuation; cursor position/visibility/style/blink; dimensions; semantic focus; hitbox edge; scroll boundary; overlay capture; action order; timer boundary. Every applicable mutation must fail. Also fail missing expected frame, corrupt frame, removed/duplicate scenario, changed manifest/profile/font/oracle commit, candidate-generated expected file, replaced comparator, symlink escape, stale source/binary and attempted approval writes. Record which requirement each mutation proves. A green verifier without these probes is insufficient evidence against baseline drift.

## Execution gates and merge closure

1. Tool qualification and external PR dependency: pin a reviewed tool revision and rerun its frame, PTY, renderer and fail-closed store tests. Complete before golden capture.
2. Reference capture gate: all required reference states/actions produce deterministic canonical and semantic evidence; hashes bind the immutable source, reviewed adapters and environment. Existing main outputs cannot populate expected files.
3. Per-component/app task gate: typed semantic/architecture checks plus exact scoped golden comparisons and PTY scenarios selected through the matrices. Every mismatch fails and produces artifacts.
4. Final integration gate: run the entire required-set manifest, all four real app binaries, direct and PTY suites, and all historically authoritative build/MSRV/lint/docs/backend-free/API/performance gates. No skipped, unknown, deferred or unowned scenario can pass.
5. Merge readiness: verify trusted bundle unchanged, candidate source matches tested integration head, branch strategy/preconditions still hold, all architectural obligations closed, no prohibited duplicate implementation or app-local paint-over survives, and independent review findings are resolved. Do not merge as part of this planning goal.

Strengthen architectural checks with mutation probes: live app-local styling can bypass a regex that detects only `fn render`, `Style::new`, `Block::default` or `buf.set_string`. Verify reusable ownership and call paths while preserving accepted custom-component drawing. A global ban on painters would reject the intended architecture; a weak keyword gate admits known duplicate implementations.

## Explicit limitations

This document designs the execution contract; it does not claim that the future complete reference bundle or candidate parity has already passed. Application scenario TSVs and component inventory define the required set. Tool tests prove capture capabilities, not application UX parity. PNGs use a deterministic renderer with documented faux bold/italic and tofu behavior; exact canonical cells and semantic state remain the gate. Unsupported terminal image/hyperlink payloads would require separate observable assertions only if the application inventory proves their use. No such required capability has been identified here.
