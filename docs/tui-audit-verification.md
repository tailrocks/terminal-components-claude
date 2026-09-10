# Verification evidence

Scope: current `holla-fable` worktree only. No other branch or history was
inspected. Initial and final gate results are recorded separately below.

## Initial gates

| Command | Observed result |
| --- | --- |
| `rtk cargo fmt --check` | Passed |
| `rtk cargo clippy --all-targets -- -D warnings` | Passed |
| `rtk cargo test` | 252 tests passed across six suites; 8.97 s |

These results describe the initial worktree, not proof that subsequent edits
passed the same gates.

## Confirmed defects and regression proof

| Classification | Priority | Evidence and impact | Fix risk |
| --- | --- | --- | --- |
| Confirmed defect | P1 | Runtime startup can return after enabling raw mode without a restoration owner; the calling shell loses canonical input and echo | Low: private ownership boundary; public API unchanged |
| Confirmed defect | P2 | Overflowing viewport invalidates its cache twice per unchanged render; large retained logs waste work on every redraw | Low: retain existing layout algorithm; cache complete constraints and compare output after mutations |
| Confirmed defect | P2 | Capture defaults reference a nonexistent binary; PNG failures were suppressed, and actual NO_COLOR could not be exercised | Low: preserve command interface; expose environment overrides and report failures |
| Confirmed defect | P1 | Crossterm 0.29 suppresses colors as empty SGR commands after emitting modifiers; real NO_COLOR loses selection reversal, bold, and underline | Low: normalize only final backend colors under Crossterm's actual suppression policy |
| Confirmed defect | P2 | Queued page-change and activation keys use the previous render's focus ring; fast navigation loses editing/selection actions | Low: drain unchanged events, yield for rendering after each state change |
| Architecture/API weakness | P2 | Visual baseline excludes the sidebar and most interaction/color states; current PNG renderer lacks rigorous grapheme/font handling | Documented limitation: supplement visual review with cell-level regression tests; no baseline claims beyond actual coverage |

### Startup rollback ownership

`TerminalSession::enter` enabled raw mode before constructing its restoration
owner. Both terminal command output and `Terminal::new` could return an error
between those operations, leaving the calling terminal in raw mode.

The restoration owner now exists before either fallible operation. Partial
initialization, successful ownership transfer, explicit leave, and drop share
one idempotent restoration path. Public signatures are unchanged.

`rtk cargo test --lib runtime::tests`: three tests passed. They cover a failing
terminal writer, initialization panic, and successful ownership transfer with
repeated explicit leave followed by drop. The injected failure preserves its
original `BrokenPipe` error.

A rebuilt showcase was also exercised in isolated controlling PTYs:

| Path | Process status | Terminal state after exit |
| --- | --- | --- |
| Startup stdout pipe without a reader | Exit 1 | Full termios equals original; ICANON and ECHO enabled |
| Normal keyboard `q` | Exit 0 | Full termios equals original; ICANON and ECHO enabled |

This PTY check covers actual terminal startup and normal exit. Panic rollback
is tested through the initialization boundary. It does not claim recovery
from SIGKILL or an unavailable output device.

### Actual NO_COLOR preserves state modifiers

Local pinned sources establish the complete failure chain:
`crossterm-0.29.0/src/style/types/colored.rs` returns an empty formatting result
when its memoized suppression policy is enabled; `SetColors::write_ansi` in
`style.rs` still surrounds those empty values with `ESC[;m`. In
`ratatui-crossterm-0.1.2/src/lib.rs`, modifier differences are emitted before
`SetColors`. The empty SGR therefore clears the state attributes before the
text reaches the terminal.

The shared runtime now uses Crossterm's own memoized policy and clears cell
foreground, background, and underline color to `Reset` immediately before
backend rendering. Theme colors remain intact while widgets calculate states
and planes. Normal and final-tick draws use the same boundary. This avoids the
invalid color commands while preserving bold, underline, and reversal.
`--color none` with no actual `NO_COLOR` retains its grayscale output.

`rtk cargo test --lib runtime::tests` now passes four tests. The fourth starts
an isolated subprocess with `NO_COLOR=1`, renders selected, bold, underlined
text through the actual `CrosstermBackend`, and verifies the attribute SGRs
remain active before the text. It rejects both empty and explicit reset
commands between those attributes and the text. No process-wide environment
mutation races are introduced into the test suite.

### Queued navigation respects render-time focus

An isolated terminal reproduced another runtime ordering defect. Starting the
showcase on Inputs and delivering `]`, `Tab`, `Enter`, `Ctrl+L` as one queued
burst switched to Text areas but lost editing and selection. Delivering the
same keys with gaps entered the text area and selected its content. The
unchanged-page `Tab`, `Enter`, `Ctrl+L` burst did not reproduce the problem;
the confirmed trigger requires a transition that changes the focus ring.

The runtime previously drained all pending events before rendering. The page
changed immediately, but its controls, hit regions, and valid focus were only
established in the next render. Later events in the same burst therefore used
the previous page's registry.

The pending-input pump now stops on `Outcome::Changed` and the runtime renders
before consuming another queued input. Ignored and consumed events still
drain together. Animation ticks remain checked between batches. Public APIs
are unchanged; documentation no longer claims unrestricted input coalescing.

The regression uses the real `Focus`/`FocusRing` types with a queued page
change and activation. It proves that unchanged events are batched, activation
waits for render, and activation reaches the new page's control. Runtime tests
now total five.

The actual burst replay now matches delayed replay byte for byte in text,
ANSI, cursor metadata, and PNG. Before/after artifacts are in
`/tmp/junie-burst-before` and `/tmp/junie-burst-after` on the audit host. The
deterministic reproduction briefly paused only the isolated showcase process,
queued the complete burst through tmux, and resumed it; it did not rely on
scheduler timing or alter another session.

### Idle scrollback reflow

`TextViewport::render` and `set_area` laid out full width, then width minus the
scrollbar. The width cache retained only the second width. Every unchanged
overflowing redraw therefore rebuilt every grapheme twice; callers that also
called `set_area` repeated that work twice more.

The cache now keys the complete width, height, and wrapping decision. Content
changes invalidate it through existing mutation methods. Rendering and input
layout share `set_area`. A second regression compares cached output against a
fresh viewport after resizing, toggling wrapping, replacing content, appending,
replacing the final line, and clearing.

Reproduction and measurement:

```sh
rtk proxy cargo test --release --lib unchanged_overflow_redraw_reuses_layout -- --nocapture
rtk cargo test --lib widgets::viewport::tests
```

For 4,000 lines at 100×30, ten unchanged redraws with matching `set_area` calls:

| Implementation | Measured elapsed time | Additional full reflows |
| --- | --- | --- |
| Original | 93.290084 ms | 40 |
| Fixed | 386.667 µs | 0 |

Both measurements use optimized builds on the same host. This is one targeted
measurement, not a general application benchmark. The regression asserts
identical complete buffers and zero added reflows; it does not assert a flaky
wall-clock threshold. The viewport suite passed five tests.

The release build emitted a toolchain warning: `rust-objcopy` could not find
`libLLVM.dylib` while stripping debug information. Compilation and the executed
tests still completed. No toolchain files were changed.

## Existing verification coverage and limits

- Showcase has behavioral tests for focus, disabled controls, keyboard/mouse
  activation, scrolling, sorting, editing, dialogs, validation, and resize.
  Its initial visual baseline hashed every cell's symbol, foreground,
  background, and modifiers at 120×40 and 80×24, with the first control focused.
  During this audit, coverage expanded to all five review sizes and four
  palettes: 460 page/size/palette cases. The navigation sidebar is intentionally
  excluded. It is not a baseline of every interaction state or application,
  and palette-only TestBackend snapshots do not exercise actual backend
  NO_COLOR suppression; the runtime subprocess test covers that boundary.
- TablePro tests exercise connections, explorer, grids, execution, safety
  gates, completion, history, pending changes, responsive drawers, and keyboard
  and mouse acceptance journeys.
- Jackin tests exercise deterministic frames, intro/outro transitions, launch
  success/failure, accounts, masking, configuration, capsule chrome, tab menus,
  and narrow recovery.
- Holla tests exercise every scenario at 72×20, 80×24, 100×30, 120×40, and
  160×50, deterministic paused frames, scope, gates, drift revalidation,
  activities, plans, cleanup, and keyboard/mouse flows. Assertions generally
  inspect behavior and selected strings. They do not compare application
  frames against every `shots/*.txt` file.
- Lists and tables render their visible rows. No broad large-dataset benchmark
  existed. The new viewport measurement covers idle retained scrollback;
  changed content still rebuilds layout. Public direct mutation of viewport
  storage remains a cache-invalidation API concern for later review.
- Runtime event coalescing and dirty rendering exist. Sustained input floods,
  shell job-control signals, and every terminal-emulator capability are not
  covered by the current tests.

## Capture commands

`tools/capture.sh` now defaults to `target/debug/showcase`, isolates its tmux
server on a named socket, accepts `CAPTURE_SESSION` and `SHOT_DIR`, and fails
when PNG rendering fails. It preserves tmux cell padding with `-N`.
`PRESERVE_NO_COLOR=1` passes the caller's actual `NO_COLOR` into the application;
ordinary captures clear that variable and use the selected color flag.

The preexisting Pillow interpreter on the audit host is
`/tmp/holla-venv/bin/python`. No dependency installation was required. Set
`PY` to another existing interpreter with Pillow when reproducing elsewhere.

```sh
rtk cargo build --bins
PY=/tmp/holla-venv/bin/python tools/audit_shots.sh
```

The default matrix is seven representative cases across 72×20, 80×24, 100×30,
120×40, and 160×50, in `truecolor`, `256`, `16`, `none`, and actual `NO_COLOR=1`.
It writes 175 frames to `shots/audit`, each with ANSI, text, cursor, HTML, and
PNG artifacts. Jackin and Holla use `--motion paused --frame 40`; showcase and
TablePro use their initial fixture states.

Narrow a rerun without touching baselines:

```sh
PY=/tmp/holla-venv/bin/python \
  CASES="showcase-forms tablepro-production jackin-capsule holla-upgrade" \
  SIZES="80x24 120x40 160x50" COLORS="truecolor no_color" \
  tools/audit_shots.sh
```

Additional selectable cases are `showcase-inputs`, `showcase-textareas`,
`showcase-terminal`, `showcase-editor`, `showcase-datagrid`, `showcase-diff`,
`jackin-hard-cases`, `holla-activities`, and `holla-hard-cases`. These support focused changed-surface review alongside
the default buttons, forms, workbench, capsule, accounts, finder, and plan.

For one explicit capture:

```sh
export PY=/tmp/holla-venv/bin/python
export CAPTURE_SOCKET=junie-review CAPTURE_SESSION=review SHOT_DIR=shots/review
BIN=target/debug/jackin-preview \
  ARGS="--scenario capsule-multi --motion paused --frame 40 --color 16" \
  tools/capture.sh start 100 30
tools/capture.sh shot capsule_100x30_16
tools/capture.sh stop
```

Use `NO_COLOR=1 PRESERVE_NO_COLOR=1` on `start` for the backend suppression
case. `--color none` alone selects the monochrome palette but does not exercise
the same backend behavior.

Capture-harness checks passed:

- Two independently launched Holla `activities-multi` and Jackin
  `capsule-multi` runs at 100×30, `--motion paused --frame 40 --color none`,
  produced byte-identical ANSI, text, cursor metadata, and PNG files. Files
  were compared with `cmp`; artifacts are in `/tmp/junie-determinism` on the
  audit host.
- The actual `NO_COLOR=1` forms capture contains only modifier/reset SGR
  parameters. The `--color none` capture contains indexed grayscale
  foreground/background parameters. This verifies that the two matrix modes
  exercise distinct backend paths.
- Shell syntax checks pass with
  `rtk proxy bash -n tools/capture.sh tools/audit_shots.sh`.

Captures are review evidence, not baseline approval. Inspect rendered PNGs and
ANSI/text before deciding whether a baseline change is intended. The PNG
rasterizer uses a small code-point width heuristic, so complex emoji and
combining sequences require direct terminal inspection or cell-level tests;
the PNG alone cannot prove grapheme correctness.

### Executed matrix

The audit ran the full five-size, five-mode matrix for ten cases:

```sh
PY=/tmp/holla-venv/bin/python \
  CASES="showcase-buttons showcase-forms showcase-inputs showcase-textareas showcase-diff tablepro-production jackin-capsule jackin-accounts holla-rust holla-upgrade" \
  tools/audit_shots.sh
```

All 250 captures completed. Each has ANSI, text, cursor metadata, HTML, and PNG
artifacts in `shots/audit`; artifact counts are 250 for each extension. PNG
dimensions match all five terminal sizes, with 50 images per size. After the
NO_COLOR runtime and disabled-state fixes, all 100 `none` and `no_color` frames
were regenerated with the same command and `COLORS="none no_color"`.
The complete 250-frame matrix was then regenerated successfully after the
queued-input runtime correction. The final focused hint change affects only
in-page focus; this matrix captures initial navigation focus, so those frames
remain current. Focused interaction captures were reviewed separately.

Representative review links:

- [Forms at minimum size, actual NO_COLOR](../shots/audit/showcase-forms_72x20_no_color.png)
- [TablePro minimum-size drawer, actual NO_COLOR](../shots/audit/tablepro-production_72x20_no_color.png)
- [Jackin capsule at normal size](../shots/audit/jackin-capsule_120x40_truecolor.png)
- [Holla upgrade plan at wide size, monochrome palette](../shots/audit/holla-upgrade_160x50_none.png)
- [Diff at normal size](../shots/audit/showcase-diff_120x40_truecolor.png), subject to the Unicode rasterizer limitation above.

The monochrome owner additionally verified live input selection after
`Tab`, `Enter`, `Ctrl+L` on the Inputs page. The actual NO_COLOR capture retains
reverse and underline for selected text and DIM for disabled input text.
The captured ANSI contains SGR `4;7` and `2`; the image is
`/tmp/junie-monochrome.EkDSix/input_selection_120x40.png` on the audit host.

## Consolidated completion gates

Final gates passed after the queued-input correction and contextual-hint
deduplication, with the reviewed 460-case showcase baseline:

| Command | Final result |
| --- | --- |
| `rtk cargo fmt --check` | Passed |
| `rtk cargo clippy --all-targets -- -D warnings` | Passed |
| `rtk cargo test` | 328 passed across eight suites; 9.32 s |
| `rtk git diff --check` | Passed |

`cargo doc --no-deps` also passed before the final hint-text-only change.
The showcase baseline was updated deliberately after rendered-output review;
it was not regenerated to bypass an unexplained failure.
