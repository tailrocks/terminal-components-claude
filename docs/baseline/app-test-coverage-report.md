# Application test coverage parity

Status: implemented. Audit scope: `showcase`, `holla`, `tablepro`, and
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
- Current app test counts: Showcase `53`, Holla `76`, TablePro `32`, Jackin
  Preview `44`.

Verification:

- `rtk cargo fmt --all` — passed.
- `rtk cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `rtk cargo nextest run --workspace --no-fail-fast` — `521 passed`, `303
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
- [~] New interactive PTY visual captures: capture tests remain ignored until a
  human reviews actual/diff artifacts and explicitly accepts them.
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
- [~] TaskRunner all-success branch: the fixture intentionally makes the
  `integration` task fail, so the success-only branch is unreachable without a
  new fixture. The failure completion branch is covered.
- [~] New PTY interactive captures: existing tuisnap pointer/static roots remain
  the visual substitute; no new snapshot approval was performed.

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
- [~] Populated-history PTY visuals: the PTY hygiene path sets
  `HOLLA_NO_HISTORY=1`; enabled and disabled history are covered in-process.
- [~] Live spinner/streaming/percentage PTY frames: these remain excluded from
  approved deterministic snapshots; paused/reduced and state assertions cover
  their contracts.
- [~] New PTY pointer visual captures: TestBackend pointer assertions cover the
  event contract; capture approval remains a separate explicit operation.

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
- [~] New PTY click/drag/scrollbar visual captures: behavior is covered by
  TestBackend/widget tests and existing visual roots; no new approved captures
  were created.

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
- [~] Jackin-specific new PTY pointer visual matrix: TestBackend and existing
  Chrome/flow tests cover behavior; no new snapshot approval was performed.

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
