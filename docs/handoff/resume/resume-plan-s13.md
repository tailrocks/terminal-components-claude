> **Status 2026-10-18: historical, landed.** S13 is done on `design/interaction-polish` (`70bf91575`, `09adee5ca` Edit menu, `d67f78dce` docs). Kept for context only; nothing to resume.

I've paused S13 as the coordinator asked. The ListCore and ContentState part is pushed to `design/interaction-polish`; the helper's Edit-menu work (M3, M22) is committed only in its own worktree, not pushed.

**Pushed (full gate inside the push lock)**
- `70bf91575` feat(list_core): ListCore and ContentState; List, Tree, Props and Steps on it (S13, K37)
  - **New shared pieces:** `ui::list_core::ListCore` holds cursor, wrap, skip_disabled, nav, scroll, selection, click, reveal, type-ahead, content, empty and `skip_when_empty`. `ui::content::ContentState` is `Ready`, `Loading(label)` or `Error(EmptyState)`; an error's Retry action is a real, clickable button.
  - **Widgets moved onto it:** ListBox, TreeView, PropsList and StepRail. They reach the shared fields through `Deref`, so app code reading `list.cursor` or `tree.scroll` needed no changes. Two small borrow fixes were needed: `src/bin/showcase/app.rs` and `pages/workspaces.rs`.
  - **List empty APIs:** they are now one field. `empty_text` is shorthand for a bare title and still draws exactly as before.
  - **New builders:** `content` on List, Tree, Props and Steps; `empty` on Tree and Props; `skip_when_empty` on List, Tree and Props.
  - **K37:** a list-like widget that is empty, loading or failed is no longer a Tab stop. A widget that already has focus keeps it while it empties, so typing a sidebar filter that matches nothing does not move focus. The existing `slash_filters_the_sidebar_and_enter_opens_the_match` test caught that case before I added the rule.
  - **Tests:** `tab_passes_the_empty_list` fails on the parent commit. There are widget tests for List, Tree and Props, plus ListCore unit tests.
  - **Showcase:** the Lists, Trees and Text utilities States grids gain Loading and Error rows, and the Lists page gains a `lists.content` option. Only those pages' digest rows changed, plus Read-only report · States: its PropsList column now draws the Loading and Error rows it used to leave blank.
- `74bb268ba` docs: "S13 outcome", deviations and notes in the plan; the S13 row in `docs/remaining-work.md` set to partial; K37 marked Fixed in `keyboard.md`.

**Deviations from the plan** (recorded in "S13 outcome")
- `ListCore::cursor` stays a `usize` and `cursor_model()` builds the `Cursor` from it. A `Cursor` field would have meant rewriting about 115 app sites during concurrent page work. As a result `Navigable` gains no `cursor_mut`.
- Tables and grids are not on ListCore yet, so empty tables keep their Tab stop for now (S15/S16).
- The planned `ContentState::Empty` is `ListCore::empty` instead, and `Error` carries a full empty state (title, hint, actions) rather than a string.
- M6 and M30 were already fixed by earlier steps.

**Gate:** nextest passed by default (2113/2113) and with `JUNIE_TUI_CLIENT_PLATFORM=linux` (2113/2113). clippy with `-D warnings` and `fmt --check` are clean. The visual check-touched run had one failure, `holla::flows::task::input_cancelled::160x50::none`. All 25 sizes of that scenario passed on a rerun, so I treated it as a timing flake under load. The helper also saw that scenario fail on the older base without my changes.

**Not pushed: the helper's work**
It is in worktree `/Users/donbeave/Projects/terminal-components-claude/.claude/worktrees/agent-aa7672b7991216a0f`, branch `plan-s13-edit`, based on 706b4a68c and not rebased.
- `bfa653935`: the standard Edit context menu on every text widget (M3), in `src/ui/edit_menu.rs`.
  - It also changes behaviour: a right click now focuses the control under it, with an off switch (`PointerConfig::secondary_focuses`).
  - "Fails on the parent" is only shown as a compile failure, because the API is new.
- `e894a152e`: default context menus built from the key maps for DataGrid and ChipBar (M22).
- `ce603b854`: WIP update to the mouse audit rows #3 and #22. Row #22 names a TreeView commit and test that don't exist yet.

The helper's visual pre-check had 15 failures. Some are clearly visible changes, so they may be caused by its own work rather than flakes:
- `jackin/capsule/new_tab` at all sizes: the expected frame is dimmed, the actual is not.
- `holla/flows/brew/services_stop_failed`: a status text now appears in the footer.
- `holla/fade/cleanup_list_wheel` and `holla/flows/task/input_cancelled`: not inspected yet.

**Where to resume**
1. In the helper's worktree, run those 15 scenarios on origin to see whether they fail there too, and fix any that come from its changes.
2. Rebase `plan-s13-edit` onto origin. Expect conflicts in `src/widgets/input.rs` (`TextInput::readout` was added upstream) and `src/catalog.rs`. Then push under the lock with the full gate.
3. Add TreeView's Expand all / Collapse all context items with the new shared builder (`ui::context::command_items`). `tree.rs` is now on ListCore.
4. Write the "S13 (Edit menu part) outcome" paragraph after my S13 notes in the plan, fix mouse.md row #22, and mark the S13 row in `docs/remaining-work.md` done.
5. Recorded for later steps: page options for Tree and Props `content`, and the `skip_when_empty` knobs (allowlisted against SC34). The helper also lists text panes in tablepro and holla that don't get the Edit menu yet, and tablepro's hard-coded grid menu, both for S22.

The push lock is released. Nothing new was started.