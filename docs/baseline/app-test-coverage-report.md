# Application test coverage parity

Status: verified. Scope: `showcase`, `holla`, `tablepro`, and
`jackin-preview`.

The four applications now use the same behavioral coverage contract. Test
counts are intentionally different because the applications expose different
state machines; parity means the same categories are checked, with explicit
exceptions for unstable or unreachable states.

## Result

- Added deterministic TestBackend coverage modules:
  - [`showcase/app_tests_coverage.rs`](../../src/bin/showcase/app_tests_coverage.rs)
  - [`holla/app_tests_coverage.rs`](../../src/bin/holla/app_tests_coverage.rs)
  - [`tablepro/app_tests_coverage.rs`](../../src/bin/tablepro/app_tests_coverage.rs)
  - [`jackin_preview/app_tests_coverage.rs`](../../src/bin/jackin_preview/app_tests_coverage.rs)
- Added CLI contract tests to all four `app_tests.rs` files.
- Replaced all four hand-written CLI parsers with typed `clap::Parser` and
  `clap::ValueEnum` definitions. Invalid values, missing values, `--help`, and
  unknown arguments now use Clap's error model.
- TablePro semantic `--connect` failures also go through a Clap-formatted
  `ValueValidation` error.
- Current app test counts: Showcase `62`, Holla `77`, TablePro `39`, Jackin
  Preview `51`.

Verification:

- `rtk cargo fmt --all` — passed.
- `rtk cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `rtk cargo nextest run --workspace --no-fail-fast` — `545 passed`, `303
  skipped`.
- The skipped tests are intentionally ignored PTY capture tests. The normal
  run includes the non-PTY snapshot-store integrity check.
- No snapshots were generated, accepted, or modified.

## Shared parity contract

Legend: `[x]` is covered by deterministic app tests or existing verified
coverage. `[~]` is an explicit boundary, not an accidental gap; the reason
and substitute are recorded below.

- [x] CLI valid values, aliases, defaults, short options, and typed conversion.
- [x] CLI invalid values, missing values, `--help`, and unknown-argument
  rejection.
- [x] Startup state: default route plus registered scenario/page inventories.
- [x] Minimum size, resize recovery, narrow layout, and wide layout.
- [x] Keyboard navigation, reverse navigation, focus ownership, and focus
  restoration.
- [x] Mouse hit testing, click activation, hover, secondary-click menus, and
  wheel behavior where the app exposes them.
- [x] Modal/chrome open, focus trap, accept, cancel, dismiss, and outside-click
  behavior.
- [x] Primary success, error/failure, cancellation, discard, and quit flows.
- [x] Loading/running/completed transitions and deterministic motion controls.
- [x] Typing, paste, modifier keys, wheel, click, drag, and scrollbar behavior
  where supported by the application.
- [x] Model/state integrity: rendered confirmations agree with stored state.
- [x] Secret masking and destructive-action confirmation checks.
- [x] Static visual baseline matrix and snapshot-store integrity.
- [~] New interactive PTY visual captures: the ignored capture functions in
  `tests/visual_baseline/{audit,showcase,holla,tablepro,jackin,pointer}.rs`
  remain manual-gated until a human reviews actual/diff artifacts and accepts
  them; deterministic substitutes are covered below.
- [x] Each remaining unstable, fixture-unreachable, or production-follow-up
  branch is documented below.

## Initial research findings and disposition

### Showcase

Initial weakness: `app_tests.rs` had broad page smoke coverage and a digest
baseline, but many component-specific event branches were only represented by
static rendering.

- [x] CLI parsing and failure behavior. The test uses `Cli::try_parse_from` for
  every color/motion value, aliases, invalid values, missing values, help, and
  unknown arguments.
- [x] Header Help and Inspector mouse actions.
- [x] Buttons: actions, toggles, disabled controls, running state, cancellation,
  and completed failure reporting.
- [x] Chips: selection, enable/disable/remove/clear, add, match mode, sort,
  page-size, and disabled engine behavior.
- [x] Chrome: status chips, menu actions, zoom state, context menu, and
  dismissal.
- [x] Dialogs: choice, destructive, prompt validation, cancel, outside click,
  and accepted actions.
- [x] Forms and Inputs: reset, toggles/radios, paste, field validation, and
  disabled-field behavior.
- [x] Text areas, lists, trees, panels, scrollbar click/drag, wheel, and focus
  transitions.
- [x] Pickers: filtering, scope changes, alternate accept, cancel, close, and
  tab actions.
- [x] Grid: edit validation, insert/delete, save, discard, fetch, refresh,
  viewer, copy, and interaction state.
- [x] Settings: edit, save confirmation, cancel, environment removal, and
  member removal.
- [x] Progress pause/resume/restart and TaskRunner running/cancelled/finished
  failure state.
- [x] Terminal selection/copy/follow/pause and Editor completion/paste/
  diagnostic paths.
- [~] Grid `FollowReference` is fixture-unreachable because Showcase's grid
  columns have no reference metadata (`src/bin/showcase/pages/grid.rs:112`);
  reachable grid interaction is covered by the `grid_*` tests in
  `app_tests_coverage.rs`.
- [~] TaskRunner all-success branch: `TaskRunnerPage::new` hard-codes the
  `integration` task to fail (`src/bin/showcase/pages/taskrunner.rs`,
  `TaskRunnerPage::new`/`Page::tick`), so the success-only branch is
  unreachable without a new fixture. The failure completion branch is covered
  by `app_tests_coverage::taskrunner_reaches_failure_completion_and_reports_it`.
- [~] New PTY interactive captures: `tests/visual_baseline/pointer.rs` capture
  functions (`showcase_hover_*`, `showcase_flows_*`, `showcase_fade_*`, and
  `showcase_resize_*`) plus the static/flow cases in
  `tests/visual_baseline/showcase.rs` remain the visual substitute; no new
  snapshot approval was performed.

Evidence: [`app_tests.rs`](../../src/bin/showcase/app_tests.rs),
[`app_tests_coverage.rs`](../../src/bin/showcase/app_tests_coverage.rs),
[`tests/showcase_baseline.txt`](../../tests/showcase_baseline.txt), and the
Showcase entries in [`tests/visual_baseline`](../../tests/visual_baseline).

### Holla

Initial weakness: base tests were only four tests; broader behavior existed in
separate flow/parity files, but CLI, modal inventory, history variants, and
pointer behavior were not explicit in one parity layer.

- [x] CLI values, aliases, defaults, motion precedence, invalid/missing values,
  help, and unknown arguments.
- [x] All 34 scenario fixtures retain representative startup and size
  coverage; the added inventory checks marker, focus owner, hit area, and Esc
  restoration for every registered page surface.
- [x] Modal inventory: Help, About, Why, Activities, file jump, alternatives,
  and both plan gates.
- [x] Mouse hover/click/secondary-click/double-click, file preview wheel/drag,
  outside-click dismissal, and focus restoration.
- [x] Resize ladder through too-small, minimum, wide, and recovery states.
- [x] Plan review, second gate, cleanup report, and model phase preservation.
- [x] History enabled/disabled behavior has a deterministic TestBackend
  substitute.
- [x] Paused/reduced motion tests cover stable substitutes for live progress.
- [~] Populated-history PTY visuals: `tests/visual_baseline/support.rs::opts_for`
  sets `HOLLA_NO_HISTORY=1`; enabled and disabled history are covered by
  `app_tests_coverage::history_enabled_and_disabled_states_are_deterministic`.
- [~] Live spinner/streaming/percentage PTY frames: these remain excluded from
  approved deterministic snapshots; `app_tests::paused_frames_are_deterministic`
  and `app_tests_flows::every_scenario_is_deterministic_at_a_frame`, plus the
  paused/reduced state assertions, cover their contracts.
- [~] New PTY pointer visual captures: `tests/visual_baseline/pointer.rs::holla_fade_browser_wheel_matrix`,
  `holla_resize_rust_dirty_shrunk_80x24_truecolor`, and
  `holla_resize_rust_dirty_grown_120x40_truecolor` remain ignored/manual-gated;
  TestBackend pointer assertions cover the event contract.
- [~] Plan/trust/snapshot gate scrollbar paths are routed in
  `screens/{review,plan,snapshot}.rs`, but current minimum-size fixtures do not
  overflow those panes; the raw scrollbar contract is covered by
  `app_tests_coverage::raw_scrollbars_route_pointer_press_and_drag`.

Evidence: [`app_tests.rs`](../../src/bin/holla/app_tests.rs),
[`app_tests_flows.rs`](../../src/bin/holla/app_tests_flows.rs),
[`app_tests_parity.rs`](../../src/bin/holla/app_tests_parity.rs),
[`app_tests_proofs.rs`](../../src/bin/holla/app_tests_proofs.rs),
[`app_tests_rows.rs`](../../src/bin/holla/app_tests_rows.rs), and
[`app_tests_coverage.rs`](../../src/bin/holla/app_tests_coverage.rs).

### TablePro

Initial weakness: the original 25 tests covered the main happy path and some
safety flows, but omitted connection outcomes, most structure sections,
multi-statement result variants, several grid events, and accepted destructive
routes.

- [x] Connection `Authentication failed`, retry, successful `Tested`, and
  timeout `Tested` outcomes.
- [x] New connection validation, save, edit, and delete.
- [x] Structure sections: Columns, Indexes, Foreign keys, Constraints,
  Triggers, and DDL.
- [x] Grid FetchMore, Refresh, Copy, FollowReference, OpenViewer, ClearFilters,
  edit/save, discard, and preview SQL.
- [x] Query syntax error, SELECT rows, INSERT/ALTER/DELETE/TRUNCATE affected
  results, safe-mode confirmation, cancellation, and history recording.
- [x] Result tab pin/unpin/close, manual completion, paste, and result routing.
- [x] History scope/filter/rerun/copy, picker scopes, empty filter validation,
  and safe-mode picker.
- [x] Accepted discard, close-tab, quit, viewer, filter-error, and destructive
  dialog routes.
- [x] Keyboard, mouse, horizontal scrolling, narrow drawer, resize, and static
  visual coverage remain covered by the original suite and visual inventory.
- [~] New PTY click/drag/scrollbar visual captures:
  `tests/visual_baseline/pointer.rs::tablepro_fade_table_wheel_matrix`,
  `tablepro_resize_workbench_shrunk_80x24_truecolor`, and
  `tablepro_resize_workbench_grown_120x40_truecolor` remain ignored/manual-gated;
  behavior is covered by TestBackend/widget tests and existing visual roots.
  No new approved captures were created.
- [~] TablePro has no distinct double-click event in the shared `MouseKind`,
  and its secondary-click path intentionally has no context menu
  (`src/bin/tablepro/app.rs:2029`). Real keychain, database, and PTY behavior
  remains outside deterministic TestBackend coverage.

Evidence: [`app_tests.rs`](../../src/bin/tablepro/app_tests.rs),
[`app_tests_coverage.rs`](../../src/bin/tablepro/app_tests_coverage.rs), and
the TablePro entries in [`tests/visual_baseline`](../../tests/visual_baseline).

### Jackin Preview

Initial weakness: the original journey and Chrome tests did not fully exercise
manager lifecycle outcomes, credential/sidecar failures, capsule commands,
editor validation, account/usage/settings branches, or Prelude errors.

- [x] Manager reconnect outcomes: running, crashed, failed setup, clean exit,
  stop, purge, workspace delete, refresh failure, prewarm, GitHub, and shell
  session.
- [x] Cockpit credential locked/retry/plain/cancel, blocked sidecar, abort,
  log, debug/info, quit, and launch failure acknowledgment.
- [x] Capsule palette, prefix commands, usage, redraw, zoom, new tab, clear,
  close, and dirty-exit keep/inspect/discard choices.
- [x] Editor duplicate-name validation, save failure, environment-key
  validation, secret masking, pending-change preservation, and account scope.
- [x] Accounts edit/enable/disable/validate/refresh and Usage detail/refresh/
  scroll read-only behavior.
- [x] Settings cancel/discard, Prelude invalid destination/backtracking, and
  screen-level launch failure detail.
- [x] Attach-with-pane restores the daemon's requested focused pane.
- [x] Existing Chrome tests plus added interaction tests cover menu, context
  menu, inspector, palette scrolling, mouse, wheel, and resize contracts.
- [~] Locked-credential spinner and asynchronous post-save PTY frames remain
  unstable/manual-gated (`tests/visual_baseline/jackin.rs:255`); deterministic
  manager/editor state transitions cover the corresponding contracts.
- [~] Jackin-specific new PTY pointer visual matrix: the static/flow cases in
  `tests/visual_baseline/jackin.rs` remain ignored/manual-gated; TestBackend
  coverage is in `app_tests_chrome::{menu_bar_opens_switches_and_runs_an_action,
  tab_context_menu_renames_and_closes_by_mouse_and_keyboard,
  command_palette_scrolls_with_the_wheel_and_keeps_the_selection}` and
  `app_tests_coverage::{usage_refresh_detail_and_scroll_are_read_only,
  attach_restores_the_requested_pane_focus}`. No new snapshot approval was
  performed.

Evidence: [`app_tests.rs`](../../src/bin/jackin_preview/app_tests.rs),
[`app_tests_chrome.rs`](../../src/bin/jackin_preview/app_tests_chrome.rs),
[`app_tests_coverage.rs`](../../src/bin/jackin_preview/app_tests_coverage.rs),
and the Jackin entries in [`tests/visual_baseline`](../../tests/visual_baseline).

## Harness and snapshot rules

- Behavioral coverage uses Ratatui `TestBackend` and the real app event router.
- PTY visual coverage uses tuisnap through
  [`tests/visual_baseline/support.rs`](../../tests/visual_baseline/support.rs).
- PTY capture tests are ignored by default. Run them only with an explicit
  `cargo nextest` ignored-test command, review actual/diff artifacts, then
  accept deliberately.
- `tests/showcase_baseline.txt` is a deterministic TestBackend digest, not a
  tuisnap snapshot store. It remains a useful broad rendering regression test.
- Live clocks, spinners, external history, and fixture-unreachable branches are
  excluded from the approved deterministic matrix only when the substitute or
  reason is recorded above.

## Final acceptance checklist

- [x] Four apps expose the same parity categories.
- [x] Every original missing/weak behavior is covered or explicitly classified
  as unstable, fixture-unreachable, or a production follow-up.
- [x] New tests assert model/state effects as well as rendered evidence.
- [x] All four CLIs use Clap typed parsing and Clap-formatted errors.
- [x] `cargo nextest` workspace gate passes.
- [x] Snapshot-store integrity passes in the normal nextest run.
- [~] Fresh PTY visual capture approval remains intentionally manual and was not
  performed in this change.
