> **Status 2026-10-18: open.** Task 1 landed (`95ac0ca05`). Task 2 is on `origin/wip/s14-followups` at `95f62ca33`. Tasks 3 (K6) and 4 (P0-G6 `PageHeader`) not started. The scratchpad `ct8.sh` path below is gone: use `docs/handoff/scripts/gate.sh` (4 visual parts). Next steps: [../README.md](../README.md).

Stopped early because the budget ran low. Task 1 is pushed. Task 2 is committed but not pushed and is missing its capture re-approval. Tasks 3 and 4 were not started. I don't hold the push lock (I released it after the task 1 push).

**Task 1 – SI17 copy-first: pushed (`95ac0ca05`)**
- **How it works:** each widget marks its selection on the frame as it draws (new `RenderCtx::selection`, which writes `FocusRing::mark_selection`). The default `KeyHost::selection()` reads that mark for the focused control, so no app writes its own check.
- **Router:** over a selection, Ctrl+C skips the interrupt and the quit and goes to the focused control.
- **Widgets that now copy on Ctrl+C:**
  - TextInput, TextArea and CodeEditor copy under every key preset. The Emacs preset (the macOS default) had left Ctrl+C unbound; the shared fix is in `EditField::action`/`text_cmd`.
  - TextViewport, ScrollPanel and DataGrid copy a cell range or marked rows.
  - A masked (password) field never copies; Ctrl+C there still quits. Before, its Ctrl+C copied the secret under the Desktop preset.
- **App changes:**
  - The showcase's own `interrupt`, which replayed Ctrl+C to the page as a per-app workaround, is removed.
  - In jackin, Ctrl+C over a selection in the build log now copies it. Before, it aborted the launch, and the log's own Copy result was dropped.
- **Behaviour change:** a focused text field with a selection (including a value selected on Tab) now copies on Ctrl+C instead of quitting.
- **Tests, each confirmed failing on the code before the fix:**
  - showcase `ctrl_c_over_a_panel_selection_copies_instead_of_quitting`
  - tablepro `ctrl_c_over_a_grid_range_copies_instead_of_quitting`
  - holla `ctrl_c_over_the_activity_selection_copies_instead_of_quitting`
  - jackin `ctrl_c_over_the_build_log_selection_copies_instead_of_aborting`
  - two new cases in `router_tests`, and `ctrl_c_copies_a_selection_under_every_preset_but_never_a_secret` in `input.rs`
- **Gate:** passed on both platforms, with clippy and fmt clean.
  - One tablepro unit test, `the_write_gate_refuses_an_outside_click_and_cancels_on_esc`, failed once in the full macOS nextest run and passed on re-run.
  - In the parallel visual run, `jackin capsule/new_tab`, `holla task/input_cancelled` and `holla brew/services_stop_failed` failed only under load and all 79 passed when re-run alone. `services_stop_failed` is new to the known timing-flake list.

**Task 2 – real clipboard everywhere: committed, not pushed (`95f62ca33`)**
- **Shared change:** `shell::run<A: App>` now calls `app.shell().clipboard.install()`, and tablepro's own install call in `main.rs` is removed.
- **holla:** plain copies go through `Shell::copy_at`, using a clipboard that follows `world.platform`'s OSC 52 support and limit. The status now reads "Copied N characters · …" or "Not copied · …" instead of always "Copied …". `copy_osc52` keeps its wording but now writes to the terminal for real.
- **jackin:** the `write(false)` setting is removed, so copies are written to the terminal (OSC 52) and no longer only to the preview's simulated clipboard.
- **Tests:** holla `a_plain_copy_reports_what_the_terminal_did` and the jackin cockpit test's OSC 52 assertion, both confirmed failing before. There is no unit test for the install inside `shell::run`, because it needs a real terminal.
- **Gate status:** nextest (macOS), clippy and fmt pass. The `JUNIE_TUI_CLIENT_PLATFORM=linux` nextest run and the visual check were not run.

**Where to resume (branch `s14-followups`, worktree `/Users/donbeave/Projects/terminal-components-claude/.claude/worktrees/agent-a3909a1b1ab8561ed`):**
1. Run the visual check with `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/s14-followups/ct8.sh <worktree>`. The holla captures that show a copy status will change wording. Re-approve them in a holla snapshot-only commit after a script attributes every changed cell to `95f62ca33`; jackin's wording should not change.
2. Rebase onto `origin/design/interaction-polish`, take the lock, run the full gate including the linux nextest run, then push.
3. Docs commit:
   - Mark SI17 done in `docs/remaining-work.md` and in the plan's "S14 deviations" 1–3.
   - Add a status row in `docs/audits/design-review-2026-10/showcase-pages-navigation.md` item 4.
   - Record the findings below in docs/audits.
4. Task 3 (K6: tablepro asks before quitting or disconnecting while a query runs, through the shared leave guard) and task 4 (P0-G6: Read-only report adopts `PageHeader`/`Heading`/`ActionRow`) are still open.

**Findings to record:**
- Table has no Copy command, so its row selection does not take part in copy-first.
- jackin's Capsule still handles its own Ctrl+C copy rather than going through the shared path.
- A field's Copy writes only to the in-app clipboard and sets no status.