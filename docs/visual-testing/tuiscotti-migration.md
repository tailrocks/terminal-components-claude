# tuiscotti migration status

The visual baseline suite (`tests/visual_baseline/`) runs on
[tuiscotti](https://github.com/tailrocks/tuiscotti) `main`, pinned in
`Cargo.toml` to `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`. Its predecessor
`tuisnap` (`donbeave/tui-snap` at `2d43458`) is gone from the repository.

## What changed

- **Driver.** Every capture is tuiscotti Workflow 2:
  `tui::Tui::new([CARGO_BIN_EXE_<bin>, args…])` with an explicit
  environment (`env_clear(true)`, `TERM=xterm-256color`,
  `COLORTERM=truecolor`, `LANG`/`LC_ALL=C.UTF-8`, `TZ=UTC`, `HOME` under
  `target/tuiscotti/home`, a minimal `PATH`, `HOLLA_NO_HISTORY=1`,
  `NO_COLOR=1` only for `nocolor`, crate root as cwd). Readiness is a
  content wait (a text `Locator` on physical rows), then
  `wait_stable_quiet(400 ms)`, then `render::frame_from_screen`. Argv,
  needles, send steps and timeouts are unchanged.
- **Committed format.** One file per scenario:
  `snapshots/<app>/<sub_group>/<cols>x<rows>/<palette>.ansi`. It is
  tuiscotti's `render::ansi_dump` plus one cursor trailer
  (`ESC[?25l` when hidden, else `ESC[?25h ESC[y;xH ESC[Ps q`), because
  `ansi_dump` alone drops the caret. `.txt`, `.png` and `.html` are no
  longer committed. The set is the single constant `COMMITTED` in
  `tests/visual_baseline/support.rs`.
- **Gate.** A local gate replaces `GroupedStore::check_with`, which cannot
  gate a subset. Every check writes `target/tuiscotti/actual/<name>.ansi`
  and `.frame.json`. A mismatch also writes actual PNG/HTML, expected
  PNG/HTML rebuilt from the approved `.ansi`
  (`target/tuiscotti/expected/`) and a diff PNG
  (`target/tuiscotti/diff/`).
- **Approval.** `accept_actuals` (ignored test, excluded by the nextest
  default filter) copies reviewed actuals for one `VISUAL_ACCEPT` prefix
  into `snapshots/` and removes that scenario's legacy files. It is local
  only, never CI.
- **Terminal modes.** No snapshot format records them, so
  `modes::*_boot_sets_terminal_modes` asserts 1049, 1003, 1006 and 2004 on
  each app's live emulator state.

## Re-approval status

A scenario is re-approved when its approved files are exactly the
committed set (`support::is_reapproved`). Only re-approved scenarios run
in the gate (`--run-ignored only -E 'binary(visual_baseline)'`). Every
other scenario still holds tuisnap's four files, is skipped, and runs only
with `VISUAL_FULL=1`, where it fails until it is re-approved.
`store_integrity` accepts either set per scenario and nothing else.

| Group | Scenarios | Status |
|---|---|---|
| `showcase/pages` (18 legacy pages + `selectable`) | 475 | re-approved (pilot) |
| `showcase/flows/selectable` (new) | 50 | approved on tuiscotti |
| every other group | 7,100 | pending re-approval |

The visual-test plan (`docs/visual-testing/plan.md`) decides the final
matrix and the OS that generates baselines (CI runs ubuntu-24.04) before
the rest is re-approved. Re-approve one group at a time:

```sh
VISUAL_FULL=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(/<group>_/)'
# review target/tuiscotti/{actual,expected,diff}
VISUAL_ACCEPT=<app>/<group>/ cargo nextest run --run-ignored only --ignore-default-filter -E 'test(=accept_actuals)'
```

Commit the `snapshots/` change on its own, apart from code.

## Pilot measurements (`showcase/pages`, 18 roots × 25 captures)

Local run on macOS arm64, debug test profile, nextest `pty` group (8
threads).

| Run | Wall time | Notes |
|---|---|---|
| tuiscotti, gate passing | 262 s | about 13.5 s per root (0.54 s per capture: boot 15–400 ms, 400 ms quiet window, gate about 20 ms); `terminal` alone 234 s (its boot streams until `7 of 7`) |
| tuiscotti, every capture mismatching | 1,129 s | the mismatch path renders actual and expected PNG/HTML plus a diff, 2–9 s per capture; optimizing the render crates did not change it |
| tuisnap `2d43458`, same source, no approvals | 375 s | about 47 s per root; renders PNG and HTML for every capture |

Churn against the committed tuisnap approvals: all 450 `.ansi` differed.
On the same source, the tuisnap and tuiscotti engines produce
byte-identical `.ansi` for all 450 captures (cursor trailer excluded). So
none of the churn comes from the engine; it is product work on the branch
since the approval (the `↑↓` footer hint in 445, scrollbar gap and
horizontal bar, list counters, sidebar indicator, theme colours). The
cursor is hidden in all 450 pilot captures. A second gate run against the
new approvals matched all 450.

Approvals go stale fast while the UI is moving: after one rebase (54 new
commits on the branch) all 450 pilot scenarios differed again, again from
product changes only, and were re-approved on that HEAD (a later gate run
matched all 525 pilot and selectable scenarios in 278 s). Whoever changes
a re-approved surface re-runs the group and approves it with the change.

| Size | Before (4 files) | After (`.ansi`) |
|---|---|---|
| `showcase/pages` on disk | 801 MB | 5.1 MB |
| per scenario, average | `.ansi` 9.6 KB, `.txt` 2.0 KB, `.png` 390 KB, `.html` 1.46 MB | 9.8 KB |
| scratch per mismatch (not committed) | | `.frame.json` 913 KB, `.html` 1.46 MB, `.png` 390 KB |

## Known tuiscotti main issues (at `a47c9aa`)

- A lone `f`/`F` key fails as a malformed function key:
  `crates/tuiscotti-runtime/src/tui/input_types.rs:165` strips the `F`
  prefix before the single-character case. The suite sends bare
  characters as `Key::Char`.
- Exact dependency pins (`Cargo.toml:22,31,32`: `crossterm =0.28.1`,
  `ratatui =0.29.0`, `unicode-width =0.2.0`) force this crate's
  `unicode-width` down from 0.2.2 to 0.2.0 and add a second
  ratatui/crossterm to the dev graph.
- `GroupedStore` always reads, renders and accepts all four artifacts
  (`grouped/check.rs:84-104,132-135`, `grouped/mutate.rs:22-27`), and
  `render::ansi_dump` (`render/text.rs:46`) writes no cursor. An upstream
  `ArtifactSet::AnsiCursor` mode would let the local gate go.
- `BoundLocator::expect_visible` requires a unique match
  (`bound_locator.rs:162-181`); boot needles that repeat on screen need
  `Locator::resolve` inside `wait_predicate` instead.
- `WaitError`'s `Display` (`tui/error.rs:107`) names only the revision;
  the suite prints the evidence screen itself.
- Rendering review PNG/HTML costs seconds per large frame, so a run where
  many captures mismatch is slow.
