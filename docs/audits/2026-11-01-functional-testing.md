# Executable functional testing review

Reviewed PR #26, `design/interaction-polish`, at
`5b3dbd7ae799f325efbb4f4cc11b2f5518dec948` on 2026-11-01. Three parallel
application/framework reviews used GPT-6 Astra with ultra reasoning. The
implementation and verification below use the actual checked-out monolithic
workspace, not architecture reports describing a different branch.

## Decision and delivered scope

Use the exact published `tuiscotti-tui-test = "=0.2.1"` package, imported as
`tui_test`, for live executable functional tests. Current tuiscotti 0.4.0 is
an offline sample/store/report library; it does not spawn applications or
open PTYs. Retain the legacy tuiscotti revision while the frozen visual
harness still depends on its removed APIs. The compatibility blocker is in
[dependencies](../dependencies.md); exact upstream references and the testing
method are in [Functional testing](../functional-testing.md).

The new [functional target](../../tests/functional/main.rs) contains thirteen
non-ignored tests. Seven cover Showcase (five color variants of the button
lifecycle, nested scrolling/focus, and help/resize), two cover TablePro, two
cover Holla, and two cover Jackin. Each launches a shipped executable on a
real PTY, uses terminal input, and asserts rendered output and clean exit.
No new unit tests or production behavior changes are included.

The harness clears inherited environment, isolates cwd/HOME/XDG directories,
and disables optional Holla history. It records text, full terminal cells,
and ordered input/checkpoint traces. It validates negotiated mouse input,
separates press from release, and sends wheel reports at the target's
coordinates. The fork's legacy wheel operation otherwise targets `(0, 0)`;
its terminal `Scroll` operation moves emulator history rather than app
content. The reference helper documents this narrow workaround.

These are real UI/process integrations with simulated domain operations.
Reopening a saved Jackin workspace proves the application's saved in-memory
preview state; it does not prove persistence across process restart. Holla
does not execute the displayed shell task, and TablePro does not contact an
external database. No external service integration or exhaustive component
coverage is claimed.

## Verification on Linux x86_64

Toolchain: Rust 1.99.1, cargo-nextest 0.9.148, velnor-actions 0.1.29. Commands
used the repository pins and a task-local tool installation. Offline Cargo
commands ran after dependency fetch. All existing package identities in
Cargo.lock were preserved; the live driver adds 37 package identities.

| Check | Result |
| --- | --- |
| `cargo build --locked --offline --bins` | Passed for all four application binaries. |
| `cargo nextest run --locked --offline --test functional --profile functional --test-threads 4 --no-tests fail` | **13 passed, 0 skipped**, 14.478 seconds; run `23004bed-ff3d-4b55-a781-66fb0e227eb5`. This is the final strengthened test source. |
| Real hover mutation | The exact truecolor button case failed at the intended hover assertion after 10.250 seconds; details below. |
| Approved visual smoke | `showcase::pages::overview::72x20::nocolor`: **1 passed**, 0.576 seconds; run `9018b0ee-d399-4f9b-99f8-959fa72cd9a4`. Other scenarios were filtered out. |
| Isolated existing scrolling-reach case | **1 passed** in 206.689 seconds; run `82a7107c-abea-4453-bf56-f011a21e2bb6`. This resolves the combined run's time-limited case below. |
| `cargo clippy --locked --offline --workspace --all-targets` | Passed with existing warnings in untouched application/library files. |
| `cargo clippy --locked --offline --test functional` after final assertion strengthening | Passed; no new functional-target warnings. |
| `rustfmt --check tests/functional/*.rs` and `git diff --check` | Passed. |
| Pinned Velnor generation | Passed; generated workflows come from `.velnor/config.toml`. |

The visual smoke used:

```sh
cargo nextest run --locked --offline --test visual_baseline \
  --run-ignored only \
  -E 'test(=showcase::pages::overview::72x20::nocolor)' \
  --no-tests fail
```

This stored, reapproved case actually opens a PTY without `VISUAL_FULL`.
Its snapshot SHA-256 remained
`60927b0cd4d03ae036c5ebc38c620f4c9a5602028c8db5c295f590e03315f986` before and
after. No approval operation was performed. The full visual matrix was not
run and is not claimed green.

### Negative control and recovery

A disposable worktree at the reviewed base changed only the shared
`PointerRouter::dispatch` motion branch in `src/core/pointer.rs`: the
`MouseKind::Move` arm returned `Outcome::Ignored` instead of calling
`state.on_move`. The separately built Showcase binary had SHA-256:

```text
23c36acc888ca99f0caaa0c5992162e63a66de1be659eec11b9b7964f983b327
```

The existing functional test executable was run against that binary using
Nextest's saved Cargo/binary metadata, preventing a rebuild from silently
replacing the mutant. Run `d9605e04-60c7-4902-a6c3-0b8bc8c06a3b` exited 100
with `timed out: Run task visibly responds to hover`. This establishes
sensitivity to a real shared input-routing regression, rather than to a
changed expectation or fake terminal fixture.

The original binary was restored through an EXIT trap and its SHA-256 was
verified as
`fde11376ad553c801f12b7df0738772f5de674fcede7bc4b1cb9930522476ad2`. The final
thirteen-case run then passed, including the identical hover test. Mutation
source/binaries are not part of the change.

## Existing gate failures and unresolved findings

The full required workspace command was attempted with a 230-second command
budget before the last two Holla assertion improvements:

```sh
cargo nextest run --locked --offline --workspace --all-targets \
  --no-tests fail --status-level fail --final-status-level fail
```

Run `36094953-f24a-4637-8ddf-fde3cd03fc2e` reported **3294 passed, 2 failed,
28899 skipped**; the enclosing timeout exited 124. The two failed entries
have different meanings:

- `docs::tests::docs_components_fresh` failed because the existing generated
  component documentation is stale: `list_box.md`, `tree_view.md`,
  `select.md`, `tabs.md`, `data_table.md`, `data_grid.md`, and their README.
  Those source/documentation files are unchanged by this addition. Refresh
  and review them in a dedicated documentation change.
- `app_tests_reach::every_section_reaches_its_whole_content_by_scrolling_3`
  did not finish within the overall command budget. After timeout, that
  specific owned process required SIGKILL; Nextest recorded 181.677 seconds
  for it. A subsequent bounded, isolated run of this exact case passed in
  206.689 seconds (run `82a7107c-abea-4453-bf56-f011a21e2bb6`). The combined
  result is a command-budget limitation, not a demonstrated product
  regression. Split this expensive existing case from shorter work when
  enforcing the repository's four-minute command limit. The stale-docs
  failure still prevents a fully green workspace gate.

Other findings are recorded rather than silently fixed outside this scope:

| Finding | Evidence and disposition |
| --- | --- |
| Repository-wide formatting is already red | `cargo fmt --all -- --check` reports differences in 163 untouched files. The new functional files pass their focused formatting check. A bulk reformat would obscure this testing change. |
| Strict Clippy is already red | `cargo clippy --locked --offline --all-targets -- -D warnings` stops on existing `collapsible_if` diagnostics in `src/widgets/field_common.rs` and `src/widgets/menu.rs`. Standard Clippy completes; additional existing warnings are in Holla/Jackin source. No lint suppression was added. |
| Existing visual cases do not imply live coverage in the default gate | Generated visual trials are ignored, and pending/unapproved cases can return without capture. New functional cases have no such bypass. Preserve the legacy suite and migrate separately. |
| Legacy Holla PTY isolation | `tests/holla_pty.rs::spawn` inherits host cwd/environment and does not generally disable history. New tests isolate these inputs; old helper migration is a follow-up. The broad existing run is not claimed isolated. |
| Showcase activation counter is not rendered in the observed Playground | `src/bin/showcase/pages/buttons.rs` allocates a fixed-height section that clips its `last: ... activations` line. Reference tests use the visible action footer and actual toggle mark instead. Layout correction is a separate component/demo review. |
| Dedicated functional CI check is not in the aggregate `required` job's explicit dependencies | The existing Rust job also discovers the unignored target, but require the dedicated `Suite (functional)` check in branch protection for an unconditional independent gate. This addition does not change branch protection. |
| Fork portability is not established by inherited CI | At review, the fork release commit had no reported CI runs and its inherited workflow referenced the old package name. Linux execution is verified here; macOS remains to be qualified. |
| Existing generator/tooling warnings | Raw actionlint reports the pre-existing `background`/`wait-all` compatibility diagnostics already handled by the generator's existing ignore patterns. The pinned generator's validation succeeds; manual suite actionlint/shellcheck and offline zizmor report no new finding. The existing Rust mise lock checksum warning remains. No new ignore or validation bypass was added. |

## Next acceptance work

The full component contract matrix is in the reference guide. Prioritize
TablePro row editing and save/history, query error recovery, Holla stop and
quit guards/concurrent activities, and Jackin launch failure/recovery and
capsule navigation. Add scrollbar-thumb and splitter-seam drag assertions,
table sort/filter/column-resize behavior, nested overlays with no
click-through, and independent chosen/focused/held states.

The Holla and Jackin examples currently use 120x40 with representative
truecolor and NO_COLOR cases. The five-color lifecycle matrix applies to
Showcase buttons only. Expand responsive sizes around actual breakpoints,
keyboard alternatives, long/empty/disabled data, and wide/combining Unicode
as separate behavioral questions. A large screenshot count is not a
substitute for these transitions and effects.
