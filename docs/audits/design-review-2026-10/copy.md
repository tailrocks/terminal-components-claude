1. **High** | src/bin/jackin_preview/app.rs:2556-2565, src/bin/tablepro/app.rs:2316-2331, src/bin/holla/app.rs:3260-3290, src/bin/showcase/app.rs:1224 | all apps / every confirm dialog | The footer always says `Enter Confirm` and `y / n Quick answer`. `Dialog::destructive` (src/widgets/dialog.rs:85) puts focus on **Cancel**, so Enter actually cancels. The snapshots show this in jackin/cockpit/cancel_confirm and tablepro/connections/delete_dialog (`▎Cancel   Delete` with the footer reading `Enter Confirm`). The same logic is also copied into four apps. | Library fix: add `Dialog::hints() -> Vec<Hint>` that builds the row from the dialog's state: `← → Choose · Enter <focused button label> · Esc Cancel · y <primary label> · n <cancel label>`, for example `Enter Cancel · y Delete · n Cancel`. Delete the four copies and call this instead. `Hint.action` would need to become `Cow<'static, str>`.

2. **High** | src/bin/jackin_preview/screens/capsule.rs:258 vs 1258-1325 | jackin capsule / View menu | The menu lists `Container info  Ctrl+B i`, but the prefix handler has no `'i'` case. Pressing it gives the status `Not a prefix command: i`. | Add `KeyCode::Char('i') => self.open_info(..)` to `handle_prefix_cmd`. Otherwise remove the shortcut from the menu item.

3. **High** | src/bin/tablepro/connections.rs:940-941 | tablepro / Connections list | The hints say `E Edit` and `D Delete`. The handler uses `key.is_char('e')` and `is_char('d')` (lines 444, 488, 492), so Shift+E and Shift+D do nothing. | Change the copy to `hint("e", "Edit…")` and `hint("d", "Delete…")`. Delete opens a dialog, so it gets the ellipsis.

4. **High** | src/bin/jackin_preview/screens/usage.rs:912 | jackin / Usage, empty state | The text says "Press R to refresh. · c registers an account". Uppercase R is not handled (only `'r'` at line 785). The rest of the screen uses `m Manage in Accounts`, and the punctuation is mixed. | Replace with `"r refreshes · m opens Accounts to add one"`.

5. **High** | src/bin/jackin_preview/screens/capsule.rs:2516 | jackin capsule / scrollback | In scrollback the hint says `Ctrl+\ Menu`. In normal mode the same key is labelled `Ctrl+\ Palette`, and the handler opens the palette (line 1926). F10 is the key that opens the menu. | Change to `hint("Ctrl+\\", "Palette")`. Separate functional risk worth checking: crossterm's legacy encoding reports Ctrl+\ as Ctrl+4, the same way the code already handles Ctrl+] as Ctrl+5 in holla/screens/activity.rs:326. Only `Char('\\')` is matched here.

6. **High** | src/bin/jackin_preview/screens/cockpit.rs:627-631 and src/widgets/dialog.rs:86 | jackin / Cancel the launch? | The buttons read `Cancel` and `Cancel launch`, so it is unclear which one keeps the launch running. | Library fix: give `Dialog::destructive` a dismiss-label parameter, e.g. `.dismiss_label("Keep launching")`, so the pair reads `Keep launching   Cancel launch`. Use the same pattern for Delete (`Keep`), Quit (`Stay`) and so on.

7. **High** | src/bin/tablepro/app.rs:2316-2325 (snapshot tablepro/workbench/commit_dialog) | tablepro / Save changes? (typed confirmation) | Focus sits in the "Type orders to confirm" field, but the footer shows `← → Choose  Enter Confirm`. Typing is required and is never mentioned. Holla's version of the same dialog says `EDIT Type Phrase Enter Next`. | Covered by fix #1: when the dialog has a token, `Dialog::hints()` returns `Type "orders" to confirm · Enter Next · Esc Cancel`.

8. **High** | tablepro/tabs.rs:720, 1302; connections.rs:919; holla/screens/review.rs:1221-1223; jackin app.rs:2585, editor.rs:2146; showcase grid/editable/settings/inputs/forms | all apps / inline field and cell edit | `Enter Commit` is jargon. In TablePro it collides with the real DB commit that `Ctrl+S Save` starts (the "Save changes?" transaction dialog). Holla's cancel label is `Esc Revert`, while every other app says `Esc Cancel`. | Use one vocabulary for all field edits: `Enter Apply · Esc Cancel · Tab Next field`. Add a shared `keyhint::EDIT_FIELD` constant in the library.

9. **Med** | src/widgets/keyhint.rs (Hint) plus ~60 call sites | all apps | Key labels are written inconsistently:
   - Arrows: `↑ ↓` (tablepro, showcase) vs `↑↓` (holla, jackin); `← →` vs `←→` (inspect.rs:774); `↑ ↓ ← →` vs `↑↓←→`.
   - Ranges: `1-6` (tabs.rs:736) vs `1–5` / `1–9` vs `1 2 3`.
   - Delete: `Del` (tabs.rs:732) vs `Delete` (files.rs:1762).
   - Slash-joined labels: `Enter / ↓` and `y / n`.

   | Add `junie_tui::keys` constants (`UP_DOWN="↑↓"`, `LEFT_RIGHT="←→"`, `ARROWS="↑↓←→"`, `DELETE="Delete"`, `range(1,6)` producing `1–6`). Also make `keyhint::render` normalise spaced arrows, so mistakes cannot render.

10. **Med** | holla/app.rs:662, screens/finder.rs:1542, screens/files.rs:1754 | holla | One key has three names: `Alt+Enter` is "Alternatives…" in the menu, "More" in the finder hint and "Actions" in the files hint. In files Find mode, plain `Enter` is also labelled "Actions". | Use `Alt+Enter Alternatives…` everywhere. Files Find mode: `Enter Open`.

11. **Med** | holla/screens/files.rs:1729-1770 | holla / Files | `/ Find` enters a mode whose hint says `Type Search`. In that mode `Delete Clear` and `Esc Clear` do the same thing. | `/` opens "Find", so the mode hint should be `Type Find`. Drop `Delete Clear`. Make Esc show `Esc Clear` when the query is non-empty and `Esc Back` otherwise (this logic already exists).

12. **Med** | jackin manager.rs:2461-2475, accounts.rs:2564-2566, config.rs:1145/1161, capsule.rs:2491/2498, inspect.rs:761; tablepro tabs.rs:727/755, connections.rs:942; holla review.rs:1229 | cross-screen | Within one app, a key means different things on different screens:
   - jackin `d`: Delete…, Disable, Remove…, Detach, Reset to sync, Unified / review.
   - jackin `x`: Remove…, Close, Shell.
   - jackin `c`: Accounts, Manage accounts, Cancel, New tab.
   - tablepro `Ctrl+D`: Toggle, Structure, Duplicate.
   - `Ctrl+S`: Save, Run (holla review), Submit (showcase).

   | Write a key-vocabulary table in the library docs (d means delete/remove, x means close, Ctrl+S means save, c means Accounts). In jackin, rename `x Shell` to `s Shell` where `s` is free, or label it `x Open shell`. In tablepro, label `Ctrl+D` as `Data / Structure` in both places. In holla review, `Ctrl+S Run task`.

13. **Med** | jackin manager.rs:2448, accounts.rs:2551, editor.rs:2140, settings.rs:1224 | jackin / button rows | When a button has focus, the hint is always `Enter Run`, including when Cancel is focused. | Library fix: `Button::hint()` returns `Enter <label>` for the focused button. Until then, use `Enter Activate`, which holla already uses (plan.rs:952).

14. **Med** | jackin cockpit.rs:988-991 | jackin / launch cockpit | `c Cancel` and `Ctrl+C Abort` look like synonyms. In fact c opens a confirmation dialog and Ctrl+C stops the launch immediately and skips cleanup. | `hint("c", "Cancel launch…")` and `hint("Ctrl+C", "Abort now")`.

15. **Med** | jackin app.rs:239; cockpit.rs:646; tablepro workbench.rs:806; showcase diff.rs:122; capsule.rs:1325 | status messages | Internal jargon reaches users:
   - "entered without the ritual"
   - "Blocked stage is a modeled state"
   - "the demo engine caps results at 500 rows"
   - "Selection copied in demo"
   - "Not a prefix command: {c}"

   | Proposed replacements:
   - `"Could not confirm running instances: {e} · list may be incomplete"`
   - `"This stage is blocked · c cancels the launch, Ctrl+Q quits"`
   - `"Showing the first 500 rows · add a LIMIT or filter to narrow"`
   - `"Selection copied"`
   - `"Ctrl+B {c} does nothing · Ctrl+B ? lists commands"`

16. **Med** | jackin capsule.rs:2488-2523; holla activity.rs:832-840, 456 | jackin capsule, holla activity | Terminal jargon in hints: `Ctrl+B Prefix`, `Ctrl+D EOF`, `Esc Keyboard back`, `h j k l Nav`. The input-mode status line also repeats the whole hint bar. | Use `Ctrl+B Pane commands…`, `Ctrl+D End input`, `Esc Release keyboard`, `h j k l Move between panes`. Shorten the status to `"Typing goes to the program"`.

17. **Med** | holla cleanup.rs:1196-1203, files.rs:1753; tablepro tabs.rs:727 | toggles | Toggle hints name the state you will switch to, with no verb. `m Permanent`, `n Dry run` / `n Live` and `Ctrl+H Hidden` all read like the current state. | Library convention: toggle hints start with a verb, e.g. `m Use permanent delete` / `m Use Trash`, `n Switch to dry run` / `n Switch to live run`, `Ctrl+H Show hidden` / `Hide hidden`. Optionally add a `toggle_hint(key, on, off, state)` helper.

18. **Med** | holla plan.rs:956-963, 1563-1569 | holla / plan review and run | `c Confirm` opens a confirmation dialog, so it needs an ellipsis. `x` discards the plan with no hint. `p Facts` is unclear (also in disk.rs:1565/1572 and cleanup.rs:1205). `Esc Discard` vs `Esc Discard plan`. `Enter Maximise output` / `Esc Un-maximise` use British spelling and an awkward verb. | Use `c Review & run…`, `p Details`, `Esc Discard plan`, `Enter Expand output`, `Esc Restore layout`.

19. **Med** | holla app.rs:728, 2832; jackin app.rs:808; capsule.rs:470, 1015; holla activity.rs:316 | menus | Menu items look enabled, then fail with "{label}: not available in the preview". | Build them with `MenuItem::disabled(true)`. Add a library `MenuItem::reason(&str)` and show it dimmed, e.g. "Not in this preview".

20. **Med** | holla `Esc Here` (activity.rs:877, plan.rs:1569, finder.rs), plus `Esc Outline` / `Steps` / `List` / `Tree` / `Results`, showcase `Esc Finish` / `Done` | Esc vocabulary | Esc labels mix verbs (Back, Cancel, Close) with destination nouns, and "Here" reads as a location word rather than the tab name. | Rule: `Esc Cancel` leaves an edit or modal without saving; `Esc Close` closes an overlay or menu; `Esc Back` goes back a page. Holla's `Esc Here` becomes `Esc Back to Here`. `Finish` and `Done` become `Close`.

21. **Low** | jackin editor.rs:2137, settings.rs:1221 | jackin / editor and settings tab strip | `Enter Body` is jargon. | `Enter Edit fields`.

22. **Low** | jackin capsule.rs:240/275/316 vs 2489-2496 | jackin capsule | Prefix hints and menu labels disagree: `x Close` vs "Close pane", `& Kill tab` vs "Close tab", `Ctrl+Q Quit` vs menu "Exit". | Hints: `x Close pane`, `& Close tab`. Menu: "Quit".

23. **Low** | tablepro tabs.rs:404; holla files.rs:691-693; tablepro connections.rs:1012, tabs.rs:2453 | empty states | Some empty states give no next step or repeat the title: "No rows / The table is empty", "Empty folder / nothing here", "Select a connection", "Select a query". | Proposed copy:
   - `"No rows · Alt+D adds one by duplicating, f changes the filter"`
   - `"Empty folder · ← goes to the parent"`
   - `"Select a connection · ↑↓ moves, Ctrl+N creates one"`
   - `"Select a query · ↑↓ moves through history"`

24. **Low** | jackin usage.rs:950 vs accounts.rs:2559 | jackin | The same action is `Enter Detail` on one screen and `Enter Details` on another. | `Enter Details`.

25. **Low** | tablepro app.rs:942-948 | tablepro / Save changes? facts | The row reads "Safe Mode    Safe Mode · deliberate confirmation required": the label repeats the value, and "deliberate" is jargon. | Value: `"{level} · type the table name to confirm"` / `"{level} · confirm before running"`. Separately, the "Cannot execute write queries…" status should end with `· Ctrl+L changes Safe Mode`.

Scratch notes (hint inventory and per-screen footers) are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/hints.txt`, `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/footers.txt` and `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/status.txt`. No repo files were changed.

## Status

Each finding above, with the commit on `design/interaction-polish` that
fixed it. Where a proposal was not taken in full, the reason is given.

| # | Status | Commit(s) | Notes |
|---|--------|-----------|-------|
| 1 | Fixed | f37b1da76 | `Dialog::hints(focused)` builds the footer from the dialog. The four app copies now call it. |
| 2 | Fixed | 8049ca930 | `Ctrl+B i` opens Container info. |
| 3 | Fixed | 8049ca930 | Lowercase `e Edit…` / `d Delete…`. |
| 4 | Fixed | 8049ca930 | The empty Usage screen names `r` and `m`. |
| 5 | Copy fixed; risk open | 8049ca930 | The scrollback hint reads `Ctrl+\ Palette`. The functional risk is not fixed: a terminal using the legacy encoding reports Ctrl+\ as Ctrl+4. That is a key-decoding change, not copy, and it belongs in a shared key-normalisation fix. Today holla handles Ctrl+] as Ctrl+5 in app code. |
| 6 | Fixed | 934985f8d | `Dialog::cancel_label`. The launch dialog reads `Keep launching   Cancel launch`. |
| 7 | Fixed | f37b1da76 | Typed acknowledgements read `Type "<token>" to confirm · Enter Next`. |
| 8 | Fixed | 88fd384a5 | `keyhint::EDIT_FIELD`: `Enter Apply · Esc Cancel · Tab Next field`. |
| 9 | Fixed | e97f56b7a | Key-label constants, and `normalize_key` applied at render. |
| 10 | Fixed; Enter label adapted | 133434765 | `Alt+Enter Alternatives…` in the finder and the Files pages. In Files Find mode, Enter reads `Enter Alternatives…` instead of the audit's `Enter Open`. Enter there opens the same menu as Alt+Enter, as the HP03 parity contract requires (I-F08), so the label follows the behaviour. |
| 11 | Fixed | 133434765 | `Type Find`; `Delete Clear` removed. Esc shows Clear when there is a query and Back when there is none. |
| 12 | Fixed | 9f2711d7a, c3515f652, 3d54fb7c6 | The key vocabulary table is in the `keyhint` module docs. jackin: d removes, x closes or stops, t toggles, c opens Accounts, v switches the view, ! opens a shell. The capsule's `Ctrl+B` chords keep tmux's letters as their own namespace. TablePro: Ctrl+D is Data / Structure and Alt+D duplicates, both for connections and for grid rows; `DataGrid::duplicate_keys` makes the grid's key configurable. holla review: `Ctrl+S Run task`. |
| 13 | Fixed | 66e5268fc | `Button::hint()` returns `Enter <label>`. |
| 14 | Fixed | 04647eb25 | `Cancel launch…` (now on x, see #12) and `Ctrl+C Abort now`. |
| 15 | Fixed; one line adapted | 7d3bb70e0 | Uses the audit's replacements except for the capsule line, which reads `Ctrl+B {c} does nothing · Ctrl+B Space lists commands`. `Ctrl+B ?` is not bound to anything, while Space opens the command palette. The launch activity line "modeled state with no runtime producer" now reads "this stage cannot continue". |
| 16 | Fixed | 04647eb25 | Plain-language capsule and activity hints. The help sheet was updated to match in c3515f652. |
| 17 | Fixed | 9f2711d7a | `keyhint::toggle_hint`. |
| 18 | Fixed | 9f2711d7a | holla plan copy. |
| 19 | Fixed | ca7d3b02d | `MenuItem::reason` disables the item and shows the reason dimmed. First landed in 00643499a, silently reverted by the docs commit 32a576251, restored in ca7d3b02d. |
| 20 | Fixed; Done kept | 9f2711d7a, f434e41a8 | Destination nouns became verbs: `Back to outline`, `Back to list`, `Back to results`, `Back to tree`, `Back to Here`. `tests/esc_vocabulary.rs` rejects any Esc label that does not start with a verb. Not taken: changing `Finish`/`Done` to `Close` in multiline editors. Esc there leaves the editor and keeps the text, so Cancel (which discards) and Close (which is for overlays) would both be wrong. They read `Done` / `Done editing`. |
| 21 | Fixed | 66e5268fc | `Enter Edit fields`. |
| 22 | Fixed | 04647eb25, d6797b63c | Hints match the menu (`x Close pane`, `& Close tab`, `Quit`). Every jackin quit confirmation is titled "Quit jackin❯?" and has a Quit button. |
| 23 | Fixed; one hint dropped | 676936c8a | The audit's `Alt+D adds one by duplicating` was left out because an empty table has no row to duplicate. |
| 24 | Fixed | 04647eb25 | `Enter Details`. |
| 25 | Fixed | 676936c8a | Safe Mode facts, and the status shown when a write query is refused under Safe Mode. |