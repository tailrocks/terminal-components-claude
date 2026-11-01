# Functional testing through the real terminal

Use the ordinary `functional` integration target as the reference for new
application tests. It launches the shipped executables in a pseudo-terminal,
sends the same keyboard and mouse reports that a terminal sends, and asserts
the visible consequences. It never constructs application state, calls widget
handlers, injects hit-region IDs, or renders into a test backend.

The reference source is [tests/functional](../tests/functional/main.rs). The
initial application review was against PR #26, commit
`5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`, on
`design/interaction-polish`. This is the monolithic `junie-tui` package with
four binaries and an `xtask` workspace member. Architecture reports describing
`crates/*` on other branches are not evidence about these binaries.

## Run the acceptance examples

Install the repository's pinned tools, then run:

```sh
mise install
cargo nextest run --locked --test functional --no-tests fail
```

The named profile is convenient for CI and retains a JUnit report:

```sh
cargo nextest run --locked --workspace --profile functional --no-tests fail
```

Filter one journey during development, without hiding missing tests:

```sh
cargo nextest run --locked --test functional --no-tests fail \
  -E 'test(showcase_button_hover_hold_release_cancel_and_disabled_truecolor)'
```

These tests are ordinary, non-ignored integration tests. They also run in the
existing default Nextest gate. Do not add `#[ignore]`, a missing-baseline early
return, or an environment flag that silently turns a requested case into
success. No baseline approval is needed to run this target. Linux and macOS
are the supported test hosts; Windows requires separate application/runtime
qualification, not just a compatible test library.

Each test gets its own process, terminal, home, working directory, and evidence
directory. Nextest limits PTY concurrency; a functional test consumes two
thread-budget slots. Each observation has a ten-second deadline, and Nextest
terminates a test after three thirty-second slow periods. Repository agent
sessions should additionally bound individual commands with `timeout 230` and
split larger existing gates as necessary.

## Which harness to use

The similar names describe different layers and versions:

| Tool | Verified identity | Appropriate role |
| --- | --- | --- |
| Existing tuiscotti dependency | Git revision `a1ea27f757d75d07b05499276cd932e9f8e067ad` | Preserve the existing PTY visual harness and frozen oracle while its old APIs are still in use. |
| Current tuiscotti | `0.4.0`, main `d8d54155947f6303a41dfd6079b53c0abc16f29a` at review | Offline samples, explicit baseline approval, comparison, reports, and rendering. It does not launch a process or open a PTY. |
| Tailrocks live fork | Registry `tuiscotti-tui-test = "=0.2.1"`; release branch commit `7abeac8478aa9c05ec7bef93afb10209cc348bec` | Recommended live driver for the new suite: checked mouse input, native launch options, raw byte input, owned terminal snapshots, and bounded waits. |
| Microsoft released tui-test | Rust `tui-test-rs 0.1.1` / JS `@microsoft/tui-test 0.1.1` | Real PTY interaction is supported, but this release lacks the fork's checked-input and owned-snapshot additions. It is not a drop-in replacement for this suite. |

The dependency alias is intentional:

```toml
[dev-dependencies]
tui-test = { package = "tuiscotti-tui-test", version = "=0.2.1" }
```

The Rust import remains `tui_test`. Pin the published package and commit
`Cargo.lock`. The fork's default branch is not its release branch. Do not
replace the exact package with an unreviewed Git `main` dependency.

Current tuiscotti explicitly delegates live application testing to
`tui_test::Session`. See its [README at the reviewed revision][tuiscotti-readme]
and the fork's [package][fork-package], [Session API][fork-session], and
[checked-input validation][fork-checked]. The branch's older rule about
"latest tuiscotti for PTY" predates this architectural change. This addition
uses the requested fork for live functional tests and retains the old frozen
harness deliberately; [dependency compatibility notes](dependencies.md)
record the migration blocker.

No raster feature is needed to assert terminal cells. A terminal-cell snapshot
records graphemes, foreground/background, attributes, underline, hyperlinks,
dimensions, and cursor state. Pixel images are useful review evidence, but a
plain text screenshot cannot establish hover or pressed styling.

## Why the existing visual suite is insufficient by itself

The existing visual harness remains valuable. However,
[main.rs](../tests/visual_baseline/main.rs) registers its generated scenarios
as ignored. Its [capture guard](../tests/visual_baseline/support.rs) can also
return without launching the application when a scenario is pending and
`VISUAL_FULL` is absent. A normal green test run therefore does not mean those
mouse states or journeys were exercised.

Many scenarios drive several inputs and compare one final frame. That can
miss a button firing on press instead of release, a canceled operation that
still changed data, or a wheel event routed into the wrong pane. A screen
revision changing only establishes that output occurred. A hover change,
spinner, or footer update can produce that revision without the intended
action working.

The new target complements frozen visual comparison with assertions between
steps. Keep both kinds of evidence. Do not replace the frozen oracle with a
candidate's own screenshots, and do not count in-process application or
component tests as executable PTY coverage.

## The test boundary

The path under test is:

1. Real executable startup and terminal mode negotiation.
2. Terminal keyboard, paste, pointer, wheel, and resize input.
3. Production input decoding, focus routing, overlays, and event loop.
4. Application operations and shared components.
5. Production rendering and emitted terminal control sequences.
6. The resulting visible cells and clean process shutdown.

Use `env!("CARGO_BIN_EXE_...")`, exposed as `SHOWCASE`, `TABLEPRO`, `HOLLA`,
and `JACKIN` by the helper. Avoid a shell command string. `App::new` passes
arguments as native process arguments and clears inherited environment. It
sets a fixed locale, timezone, terminal type, client platform, and color mode;
isolates HOME and XDG directories; and disables Holla's optional history.

These applications are deterministic demonstrations. TablePro executes its
local SQL/catalog model; Holla simulates task execution; Jackin simulates
workspace/account operations. A saved row or launched task is a real
application interaction and an in-memory simulated domain effect. It does
not prove an external PostgreSQL connection, Docker command, real Jackin
service, or credential provider works. When real adapters are added, give
them a separate functional tier with disposable services, observable external
effects, and teardown; preserve the same terminal interaction layer.

Use `--motion paused --frame <n>` for held-state and static view tests.
Holla and TablePro asynchronous operations need normal/reduced motion.
Jackin additionally supports opt-in `JUNIE_TUI_TEST_CLOCK=1`; the helper's
`advance_clock(ms)` sends its documented bracketed-paste clock request. Do
not assume every application implements that clock. Never send clock input
as a substitute for a domain interaction.

## Write one journey around observable contracts

Start from a user task, such as "cancel a write, retry it, and execute it only
after entering the required table name." Write down the initial condition,
input, expected visible result, and forbidden side effect for each step.

An effective assertion answers a specific question:

| Input | Strong visible proof | Insufficient proof |
| --- | --- | --- |
| Hover a button | That button's cell styling changes; its action has not run. | Any cell on the screen changes. |
| Hold a button | Target styling differs from rest and hover while the action remains pending. | A screenshot after a complete click. |
| Release a button | Its expected status, selection, dialog, or domain result appears. | The terminal emitted more bytes. |
| Cancel a save | The preview closes and pending edits remain; reopening shows them. | The word "Cancel" disappears. |
| Save | Reopen the affected object and observe the exact changed field. | A generic saved toast alone. |
| Scroll a pane | Its visible item/line identities advance and the neighboring pane is unchanged. | The terminal viewport moved or the scrollbar changed color. |
| Delete an item | A search/list scoped to the item's identity has no matching row. | A removed toast followed by an unrelated screen. |
| Resize | The app's rendered size/layout changes and the open view remains usable. | The emulator reports new dimensions before the app repaints. |

### Locate targets from rendered output

`find(screen, label)` requires exactly one physical-row match and returns
zero-based terminal cells. It maps UTF-8 offsets through actual cells, rather
than treating byte offsets or Unicode code points as columns. For repeated
labels, use `find_all`, then scope the matches to a visible panel or dialog
action row and assert that the scoped result is unique.

Do not blindly select the first match. A dimmed editor's Save action can
remain visible behind a modal's Save action. A footer can advertise
`g Go to path` while the path dialog is closed. Prefer a dialog-specific
title or prompt as readiness evidence, and locate the action within that
dialog. The reference account and file-browser journeys demonstrate this.

Polling predicates must return `false` while a transient frame lacks their
required content. Do not put an unconditional `find(...).unwrap()` inside a
wait predicate. Resolve strict locators after readiness, or use total
`find_all`/`Option` logic inside the predicate.

### Test rest, hover, held press, release, and cancellation separately

This example uses the same real-process helper as the suite:

```rust
let app = App::new(
    "run-button-example",
    SHOWCASE,
    &["--page", "buttons", "--motion", "paused"],
    160,
    50,
    "truecolor",
);
app.wait_text("Disabled primary");
let initial = app.settle();
let run = find(&initial, "Run task");
let rest = crop(&initial, run, 8, 1);

app.move_to(run);
let hovered = app.wait_screen("Run task hover", |s| crop(s, run, 8, 1) != rest);
let hover = crop(&hovered, run, 8, 1);

app.down(run);
let held = app.wait_screen("Run task held", |s| {
    let cells = crop(s, run, 8, 1);
    cells != rest && cells != hover
});
assert!(!text(&held).contains("Run task ✓"));

app.up(run);
app.wait_text("Run task ✓");
app.finish();
```

Use a new session or a distinct prior action state when asserting that the
button has not fired: a status already present before the input is not new
evidence. For an exactly-once action, prefer a visible count or a stateful
toggle whose resulting on/off state exposes an extra activation. The
Showcase's internal activation counter is not painted at the current
Playground height; do not assert invisible text or read its private field.

A combined `click` is appropriate when only the completed action matters.
It cannot prove the intermediate held appearance. For drag cancellation,
call `down`, then `drag_to` outside the hit region, then `up` outside. Plain
`move_to` encodes movement with no button held and is not the correct
held-drag event.

Checked mouse operations also verify that the application enabled the
required tracking and SGR coordinate modes. The released upstream raw mouse
API can inject events even when a real terminal would not send them. This
can conceal a missing `EnableMouseCapture` regression, which is one reason
the fork's checked API is preferable here.

Do not confuse persistent selected/active-page state with transient held
press. A selected tab remains selected after release; a pressed button
should not retain its held appearance indefinitely. Some controls show a
brief release flash. Before comparing the next hover/press sequence, wait
for that specific target to return to its expected non-flash state.

### Send wheel input to the application component

The inspected fork and upstream convenience `MouseAction::Scroll` encode
wheel events at `(0, 0)`, irrespective of the last mouse move. The separate
`Operation::Scroll` scrolls the terminal emulator's history viewport and
does not exercise application scrolling. Neither is suitable for a pane
routing assertion.

`App::wheel(point, down, amount)` first uses checked pointer motion, validating
the live coordinate and negotiated mode. It then calls the fork's public
SGR wheel encoder at that exact point and writes the bytes through the same
PTY. Down uses code 65 and up code 64; the wire coordinate is one-based.
There is no separate PTY implementation or synthetic application event.
The helper is deliberately small and should be replaced by a checked,
coordinate-aware upstream wheel API when one is released.

After scrolling, assert the actual first visible row or line changed. Also
assert a neighboring pane's content stayed fixed, the original selection
did not unexpectedly move, and keyboard navigation follows the clicked
focus. Test reverse scrolling and boundary clamping for long content. For
scrollbar dragging, split press/drag/release and inspect the thumb while held.

### Synchronize on effects

`wait_text` and `wait_screen` have bounded deadlines and check that the process
is still alive. `settle` is a supplemental quiet wait. It is not an event
acknowledgment or application frame boundary; the fork itself documents this
limitation. A negative assertion evaluated immediately after an input can
pass before the application handles that input.

Pair absence/unchanged assertions with positive progress: observe a modal
close, wait for a distinct result, or perform a subsequent visible action
that makes an unwanted earlier effect observable. Check disabled styling
with the target held, then exercise a working control in the same session.
Do not increase arbitrary sleeps until a flaky test appears green.

For an asynchronous operation, wait for a durable result rather than a
very short spinner frame. Inspect an intermediate progress state only when
the application provides a controlled clock or stable observable barrier.

## Coverage to retain and extend

The checked-in target provides executable examples for the following
contracts. It is a starting acceptance suite, not an exhaustive census of
every component and option.

| Application | Reference journeys |
| --- | --- |
| Showcase | Button rest/hover/held/release/cancel/disabled/toggle behavior in truecolor, 256 colors, 16 colors, monochrome, and `NO_COLOR`; wheel routing between adjacent panes and keyboard navigation after click; open help surviving shrink/grow and returning to page navigation. |
| TablePro | Authentication failure, retry, and recovery by choosing another connection; write-confirmation outside-click policy, cancel/retry, wrong token, disabled Execute, correct token, held Execute, and completed operation. |
| Holla | Child-directory task search, trust cancel/confirm, arguments and simulated task output; hidden-file toggling, long-preview wheel movement, bad-path recovery and reopening. |
| Jackin | Exact workspace mount changes, unsaved-change cancel, Save hover/held/release, controlled completion, and reopened values; required account fields, masked credentials, save, canceled removal, confirmed removal and filtered-list absence. |

Add shared component contracts through Showcase and at least one real
consumer when a change affects that consumer. Use this matrix to choose
follow-up journeys rather than multiplying screenshots without a behavioral
question:

| Component family | Required functional contracts |
| --- | --- |
| Buttons, icon actions, links | Rest/hover/press, release inside, drag-release outside, disabled no-op, keyboard activation, exactly-once observable effect. |
| Checkboxes, radios, toggles | Pointer and keyboard changes, group exclusivity, disabled values, persistent chosen state after focus moves, mixed-state transitions where supported. |
| Tabs, segmented controls, sidebars | Selection distinct from focus, click and arrow navigation, active page content, closable/dirty tabs, selection neutralized beneath an overlay, focus restoration on close. |
| Lists, trees, menus | Selection retained after blur, disabled-item skipping, type-ahead, expanded/collapsed children, menu hover switching, outside dismissal, long-content scrolling, no viewport snap after hover. |
| Tables and grids | Row/column selection, sort and filter effects on actual IDs, horizontal/vertical scrolling, inline edit cancel/commit, pending-change gate, history, context-menu action and column resize. |
| Inputs, editors, forms | Typing, bracketed paste, selection, Unicode and wide text, required/invalid input, secret masking, submit/cancel, focus traversal and short-form scrolling, completion acceptance and dismissal. |
| Selects, pickers, dialogs, drawers | Correct top layer receives pointer/keys, no click-through, outside-click policy, Escape cancel, no lower-layer focus gutter, reopen state and return focus, nested overlays. |
| Scrollbars, splitters, pane layouts | Thumb/seam rest/hover/held appearance, drag position/content effect, track clicks, independent neighboring pane, bounds, resize while open and small-terminal recovery. |
| Runtime/shell | TTY startup, input/paste/mouse negotiation, keyboard fallback, resize, supported color modes, process exit code, restored alternate-screen/paste/mouse state. |

Prioritize TablePro row edit → canceled save → confirmed save → history;
query error → correction → results; Jackin launch failure and recovery;
Holla activity stop/quit guards; then wider catalog states. Cover different
data shapes: empty, one item, long lists, disabled rows, repeated labels,
long labels, and combining/wide Unicode. Avoid fixtures that only work at one
magic coordinate or exact viewport.

For pull requests, keep a small representative matrix fast and mandatory.
Run broader adjacent/breakpoint sizes and all color modes in a scheduled
tier. Include 80×24 and a larger size for responsive behavior; add widths
immediately below and above actual layout breakpoints. A complete matrix of
sizes and colors does not replace distinct domain journeys.

## CI and failure evidence

The source configuration is [.velnor/config.toml](../.velnor/config.toml).
The dedicated functional suite runs on PRs and branch pushes, and can be
selected manually with `tier=functional`. Its generated command is:

```sh
cargo nextest run --workspace --no-tests fail --profile functional
```

The profile selects only the non-ignored functional target. Failures retain
`target/functional` and `target/nextest/functional` for fourteen days. The
harness writes an ordered action trace, text and full-cell JSON checkpoints,
and final screen evidence. Action traces are persisted during the test so
a hard timeout does not erase all progress. Argument or key payload lengths
are logged for typed/pasted content; tests use synthetic credentials.

`finish()` asserts a live child before Ctrl+Q, exit code zero without a
termination signal, and restored terminal modes. `Drop` still closes the
session and removes the child home after assertion failure. A cleanup kill
does not count as a successful application exit.

Velnor generates the workflows; change its configuration and run
`velnor-actions generate`, never hand-edit generated YAML. The generated
aggregate `required` job does not explicitly depend on `suite-functional`.
The existing Rust crate job also runs the unignored target, but administrators
should require the dedicated functional check if they want that particular
unconditional suite and its evidence to be a separate merge gate. This change
does not alter branch protection.

On a failure, read the action trace and the last frame first. Classify the
failure as an incorrect locator/precondition, input-mode/routing failure,
product regression, stale visual reference, or infrastructure timeout. Fix
the specific cause. Preserve first-failure evidence, do not silently retry
until green, and do not update expected snapshots from a failed candidate.

For important regression tests, demonstrate sensitivity: run a controlled
mutation in a disposable checkout, such as suppressing pointer motion or
redirecting wheel coordinates, and confirm the targeted test fails for the
intended reason. Restore the original source and rerun the same test. Keep
mutation controls out of the application and out of normal acceptance runs.

## Migrating the old visual harness later

Updating only the old tuiscotti Git revision will break its removed live
runtime and grouped-store APIs. Migrate deliberately:

1. Keep the frozen `visual-baseline` tag and its artifacts unchanged.
2. Move live sessions to the released fork's `tui_test::Session` API.
3. Feed captured `TerminalSnapshot` values into current tuiscotti samples;
   do not replace executable journeys with `capture_view(TestBackend)`.
4. Adopt `FrozenStore` checks and explicit candidate/digest approval.
5. Verify semantic frame/cursor preservation and review any renderer/schema
   differences before approving new reference data.
6. Switch to upstream `tui-test-rs` only after a released version contains
   the required APIs and this application suite passes on it.

The published fork has useful checked-input child-process tests, but its
release commit had no reported CI runs at review and its inherited workflow
still referenced the old package name. Do not infer portability from those
workflow files. Pin the fork and test the actual application on each supported
OS. The current upstream main also has later fixes absent from this release;
future dependency changes need explicit qualification.

## Primary references

- [Current tuiscotti architecture and installation][tuiscotti-readme].
- [Released fork package and features][fork-package].
- [Public Session launch, input, capture, and quiet-wait API][fork-session].
- [Checked input, mouse mode validation, and held-drag semantics][fork-checked].
- [SGR mouse encoder][fork-mouse] and [legacy wheel routing][fork-engine].
- [Owned terminal snapshot schema and equality][fork-snapshot].
- [Quiet wait sampling and frame-boundary limitations][fork-wait].
- [Microsoft tui-test 0.1.1 release][upstream-release].
- [Upstream checked-input contribution #282][upstream-checked].

[tuiscotti-readme]: https://github.com/tailrocks/tuiscotti/blob/d8d54155947f6303a41dfd6079b53c0abc16f29a/README.md
[fork-package]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/Cargo.toml
[fork-session]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/runtime.rs
[fork-checked]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/session.rs
[fork-mouse]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/input/mouse.rs
[fork-engine]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/engine.rs
[fork-snapshot]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/snapshot.rs
[fork-wait]: https://github.com/tailrocks/tui-test/blob/7abeac8478aa9c05ec7bef93afb10209cc348bec/crates/tui-test/src/wait.rs
[upstream-release]: https://github.com/microsoft/tui-test/releases/tag/0.1.1
[upstream-checked]: https://github.com/microsoft/tui-test/pull/282

## The standard for every functional test

Every test in this target follows one standardized pattern, with no
exceptions and no legacy-API tests:

1. Launch the real executable through the shared `support::App` harness
   (`tui_test` / `tuiscotti-tui-test =0.2.1`, isolated environment, bounded
   semantic waits, clean-exit and terminal-restoration assertions).
2. Drive it with terminal input and assert the observable outcome between
   steps (hover/held/release, cancellation, focus, neighbouring panes).
3. End with `app.visual("<test-id>")`: the live screen is frozen as a
   Tuiscotti state sample and compared with `FrozenStore` from latest
   Tuiscotti (`tuiscotti-latest`, main `d8d5415`). The test asserts
   `outcome.is_pass()`; a missing or changed baseline fails. Baselines for
   all 25 tests live in `tests/functional/baselines/` and were approved
   once with the Tuiscotti CLI `accept` command — tests never approve.
   Checkpoints sit on deterministic screens (after `settle()`, not on
   cycling status words or mid-flash frames).

The legacy Tuiscotti 0.2 dependency remains only for the separate
pre-existing `visual_baseline` harness; nothing in `tests/functional`
uses it.

## Union suite (best of both reviews)

The suite combines both reviewed deliveries: the `tuiscotti-tui-test` journeys
(color matrix, nested scrolling, resize/help, TablePro connection/write gates,
Holla trust/browser, Jackin workspace/account) plus journeys ported from the
companion review in `tests/functional/ported.rs` (literal Showcase colours
`#48e054`/`#3ab343`/`#2e9a38` and one focus gutter, datagrid wheel with an
unchanged sidebar, TablePro dirty-tab close guard, Holla finder/second-gate,
Jackin launch/editor/help) and a strict visual check in
`tests/functional/visual.rs`.

`visual.rs` uses latest Tuiscotti main `d8d5415` (0.4.0, renamed dev-dependency
`tuiscotti-latest`): a live `tui_test` snapshot is checked with
`FrozenStore::check` and the test asserts `outcome.is_pass()` — an `Ok` result
alone is not a pass, and a missing baseline fails. The approved baseline lives
in `tests/functional/baselines/`; approval was done once with the Tuiscotti
CLI `accept` command, never by the test itself.

TablePro fix: the tab picker's Delete route now shares the Ctrl+W dirty-tab
guard (`request_close_tab` in `src/bin/tablepro/app.rs`). The regression test
`ported::tablepro_dirty_picker_close_matches_ctrl_w_and_cancel_preserves_sql`
fails on the original source and passes with the fix.

Verified on macOS (in addition to the Linux evidence in the audit):
`cargo nextest run --locked --test functional --profile functional` — 25 passed.
