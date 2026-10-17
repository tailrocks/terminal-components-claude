> **Status 2026-10-18: historical, landed.** S14 is done (`932b234e8`..`2f014008a`, plan "S14 outcome"). Follow-ups are in [resume-s14-followups.md](resume-s14-followups.md). Nothing to resume here.

I stopped as you asked. Nothing was pushed: I have no fully-gated commit, and the push lock was held by plan-s17 (since 18:04) when I tried at 18:10. Two commits are on local branch `plan-s14` in `/Users/donbeave/Projects/terminal-components-claude/.claude/worktrees/agent-a332246b2abdf8bed`, on top of `08af0b70f`. The working tree is clean, and I removed the temporary gate worktree I had set up.

**`e90d45cbb` – S14 library (complete, not yet through the full gate).** Its own tests, clippy, fmt, the showcase coverage/catalog tests and `docs_components_fresh` pass.
- **`shell::Router` and `shell::Nav`** (`src/shell/nav.rs`, tests in `src/shell/nav_tests.rs`):
  - one page stack per tab, with push, back, pop-to-level, replace, reset, and open, select and close tab;
  - each shown page gets its own `PageKey`;
  - `Router::request` asks the `LeaveGuard` only when a move would drop a page (K4/K6);
  - `breadcrumb` and `crumb_click` make the trail clickable (M5).
- **`shell::Chrome`** (`src/shell/chrome.rs`): computes the header, body and footer areas once (A17). `fit` keeps a dialog clear of the header and footer rows, which is the replacement for tablepro's `outer_margin` (CO14).
- **`shell::Menus<C>`** (also `src/shell/chrome.rs`): menus whose items carry a typed command, so a choice resolves to the command and never to its label (M23).
- **`shell::Clipboard`** (`src/shell/clipboard.rs`): writes OSC 52 within the terminal's size limit and reports the true result. The runtime writes the escape sequence after the frame through a new `Application::take_output`.
- **Shell and App additions:**
  - `Shell::copy_at` and `Shell::quit_through`, and `App::stakes` with a default of clean.
  - `Request::Copy` now carries an optional name, and `Cx::copy_as` sets it, so the status can say "Copied Host". The three other apps' match arms are updated to the new shape.

**`fc7e40f2d` – WIP showcase migration (not gated, regression tests missing).** The 526 existing showcase tests pass.
- `goto` is gone. The showcase now runs on `Router<Shown>`: the sidebar, `--page` and Ctrl+PgDn start the stack over; a page link pushes onto it; going back returns focus to the control it came from.
- **P0-G6 is closed:** Read-only report's "Disk cleanup →" and "Activity →" now open those pages instead of posting a status.
- Esc at the top of a pushed page goes back. On a pushed page the header trail is clickable and pops back to the level clicked.
- Copies go through the clipboard, and the pages that used to set their own "Copied …" status now name the copy with `copy_as`.
- **Your decision:** on a pushed page the header trail drops the sidebar group name and shows only the stack ("Read-only report › Disk cleanup"). A group isn't a page, so it shouldn't look like a link. A root page shows "group › page" exactly as before. This needs recording in docs/audits.
- **Possible visual changes:** the header on a pushed page, and statuses after a copy now that the shell reports them. The visual check hasn't been run on any of this.

**Where to resume**
1. Add regression tests through the real app, each shown failing on `e90d45cbb`:
   - a click on "Disk cleanup →" shows that page;
   - Esc then returns to Read-only report with focus back on the button;
   - a click on the header trail pops back;
   - an over-limit copy shows the "Not copied" error.
2. Run the gate. The script is `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/plan-s14/gate.sh <checkout dir>`. Run it inside the push lock, attribute any visual changes and re-approve them in snapshot-only commits, then push the library commit first and the showcase second.
3. Migrate tablepro: replace the `Screen` enum with `Router`, use `Chrome::fit` instead of `outer_margin` at `app.rs:100` and `:1272` (CO14), give its regions to `Chrome` (A17), and quit through `quit_through`.
4. Migrate holla:
   - its `go` stack moves to a `Router` with tabs, the crumb and trail go through `Router::breadcrumb`, and quit through `quit_through`;
   - the domain cases of `go` stay in the app;
   - `run_menu`'s matching on labels becomes `Menus<HollaCmd>`;
   - `copy_osc52` becomes `Clipboard` with `osc52` and `limit` taken from `world.platform` and `write(false)`, keeping holla's wording.
5. Migrate jackin: `Screens`, `ROUTES` and `go` move to `Router`, `run_host_menu`'s label matching becomes `Menus<Cmd>`, its copy goes through `Clipboard`, and its quit dialog goes through `quit_through`.
6. Docs: update the plan's S14 status row and outcome, and mark S14, CO14, A17 and P0-G6 in `docs/remaining-work.md`. The SI17 Ctrl+C rule is already in `KeyRouter`, so it only needs a check per app.

**For other steps:** the library tree `src/shell` is not in the coverage scan's roots (`ROOTS` in `src/bin/showcase/catalog_scan.rs`). The new shell API is therefore not gated by coverage, and that should be recorded as a gap.