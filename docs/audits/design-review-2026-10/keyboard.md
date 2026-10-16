Audited `origin/design/interaction-polish` at 9776abef7. I walked Tab across every showcase page and every holla and jackin scenario, plus the tablepro welcome and connected workbench, using throwaway TestBackend harnesses that record the focus stop, its area, whether the caret is visible and the footer. Four read-only subagents then went through the code screen by screen. The worktree has been removed and nothing was committed.

**The main result:** across all 24 showcase pages, every holla scenario and the jackin host screens, Tab never produced a visible caret. In TablePro, a newly focused text field also shows no caret; the caret only appears in an editor or grid cell after Enter. The only stops that showed a caret on arrival were holla's finder search and the jackin terminal pane. Text fields, prompts, typed confirmations, the SQL editor and filters all need Enter or F2 before typing does anything. Until then, stray letters fall through to page shortcuts, including `q` quit, `e` reload the form and `z` zoom. Second, hints are hand-written per page, so they often don't match the focused control.

## 1. Keyboard interaction model to adopt library-wide

**Focus order**
- One ring, built in render order, which is visual order. Disabled controls are skipped. The ring wraps; add `FocusRing::wrap` (default true).
- Modals fence the ring. ModalStack restores focus on close. When the focused control vanishes, focus goes to `FocusFallback::Nearest`. That should be the runtime and app default; today it is `First`, which in showcase means the sidebar.
- Popups (select, menu, picker) also save and restore their owner's focus.
- Containers can return focus to the child that was last focused (`remember_child`, default on), so Esc to the sidebar and back resumes where the user was.
- Anything that can't act doesn't take a Tab stop: an empty list, a menu bar that isn't wired, a 1×1 placeholder. An empty control may opt in to staying focusable, with an "Empty" hint.

**Caret rules, by control type**

| Control | Rule |
|---|---|
| TextInput | Focus shows the caret at the end of the value. Typing edits immediately; Enter applies. Esc reverts while dirty, otherwise it bubbles. The old navigation mode stays available as `EditEntry::Explicit`. |
| TextArea | Same as TextInput, with Enter inserting a newline. Leave with Tab or Esc. |
| CodeEditor | Same as TextArea. Tab indents only while `tab_leaves=false`, and that state is hinted ("Tab Indent · Esc then Tab Leave"). Add F2. |
| Grid / table cell | Arrows move the cell. Enter or F2 edits, with the caret in the cell. Typing a printable character replaces the value, spreadsheet style. Opt in with `type_to_edit_cell`. |
| Search / filter / picker query | Always editing; the caret is always shown. |
| Prompt / typed-confirm dialog | The field starts in editing mode with the caret. Enter fires the primary action when valid; it never moves focus to Cancel. |

The policy is `EditEntry::{Immediate, TypeToEdit, Explicit}`, set on `TextInput`, `TextArea`, `CodeEditor` and `Dialog::prompt`, plus a theme or app-wide default:
- Library default: `Immediate` (macOS).
- `TypeToEdit`: caret drawn as a steady block only while editing, with a placeholder underline on focus.
- `Explicit`: today's behaviour.

Single-letter page shortcuts must yield to a focused editable control. Global quit becomes `Ctrl+Q`, or `q` only when no field is focused.

**Activation keys**
- Button: Enter and Space.
- Checkbox and toggle: Space; Enter also works, but is configurable so Enter can submit a form instead.
- Radio group: ↑↓←→, Home and End, and the selection follows the cursor.
- Select (closed): Enter, Space or Alt+↓ opens it; ↑↓ steps the value; typeahead on by default.
- Select (open): Tab chooses and moves on.
- List and tree: arrows, Enter opens, Space marks.
- Tabs: ←→ activate.
- Dialogs: Enter is the primary action; Esc dismisses per the dialog's `Dismiss` policy.
- Destructive one-key answers (`y`) only work when the danger button is focused. Knob: `quick_answer: Safe` (default) or `Always`.

**Esc ladder** (one rule)
1. Close the topmost popup or reveal.
2. Cancel an in-progress edit, reverting it. TextArea: add `esc_reverts`; it currently commits, and is hinted "Done".
3. Clear a non-empty filter.
4. Leave the pane, going back to its parent.
5. Never quit the app without confirmation.

Ctrl+C never bypasses an unsaved-changes guard or a page's own Ctrl+C handler.

**Hints**
- Each widget gets `fn hints(&self, focused, state) -> Vec<Hint>`. The hint bar composes them as: focused widget → page extras → globals (`Tab Next`, `? Help`, `Shift+F10 Menu`).
- Read-only and empty states change the hints (no "Enter Edit" on a cell that can't be edited).
- `Tab <label>` is generated from the ring's next stop, never typed by hand.
- Text editing uses only `EDIT_FIELD` and `EDIT_CELL`. Esc wording is limited to Cancel, Close and Back, plus "Done" for documents.

**Help**
- `?` (when no field is editing) and F1 everywhere, including inside menus and selects, using the one `HelpOverlay`.
- Sections are generated from the registered keymaps and the focused widget's hints, then the page's, then the globals.
- `?` and F1 also close it.

**Mouse-only affordances need a keyboard path**
- Splitter: becomes a focus stop, or gets a pane chord (Alt+Shift+arrows, as Capsule already does), and is hinted.
- Column resize: does not exist by mouse or keyboard; if added, give it a key too (for example Ctrl+Alt+←→).
- Hover reveal: → reveals on every widget that truncates (Menu, Picker, Select, StatusBar, read-only Grid).
- Status-bar chips: reachable through a status-bar focus stop or F6 region cycling.
- Text selection: Shift+arrows exists in viewports and should be hinted.
- Chord timeouts are configurable (`PREFIX_TIMEOUT_MS`).

## 2. Ranked findings

`severity | app/screen or widget | file:line root cause | what a keyboard-only user experiences | library-level fix (+ knob/default)`

1. **Critical** | all apps, every TextInput | `src/widgets/input.rs:117` (`type_to_edit: false`), `:313-331`, caret only while editing `:653-655` | Tab to a field: a bar and no caret. Typed letters vanish or fire page shortcuts. My walks recorded `caret=false` at every stop on all 24 showcase pages. | `EditEntry::Immediate` by default: focus starts the edit and places the caret at the end. Keep `Explicit` as the opt-out. App key routing gives an editing field priority over letter shortcuts.
2. **Critical** | tablepro: delete and drop typed confirm | `src/widgets/dialog.rs:769-775` (Enter in the ack field moves focus to the next control, Cancel) | Type the name, press Enter twice, and the action is cancelled. The field also needs Enter before typing works. | The ack field starts editing. Enter fires the primary action once the text matches, and otherwise shows an inline error. Knob: `ack_enter: Submit` (default) or `NextField`.
3. **Critical** | holla Plan tab | `src/bin/holla/app.rs:667-670`; global Ctrl+C runs before `plan.rs:1356` | The footer says "Ctrl+C Cancel remaining", but Ctrl+C quits the app and kills running activities. | The runtime gives the page the first chance at Ctrl+C. Quit always goes through an unsaved or running guard.
4. **High** | jackin Editor and Settings | `jackin_preview/app.rs:639-647` (Ctrl+C), `:752-757` (`q` goes to Manager, skipping `leave()`) | Ctrl+C or a stray `q` throws away unsaved edits without a prompt. | A library `LeaveGuard` that every route change consults. `q` is not global while a form is dirty.
5. **High** | tablepro query editor | `src/widgets/code.rs:452` (navigation mode); `tablepro/app.rs:709,847` | The placeholder says "Type SQL…", but `q` quits, `z` zooms and `[`/`]` switch tabs. No caret. | CodeEditor follows `EditEntry`. Default `Immediate` for an editor that is a tab's main control.
6. **High** | tablepro connection form | `tablepro/connections.rs:471-482` (`e` reloads), `app.rs:385-399` (`q` ignores a dirty form), `app.rs:920` (no Esc) | `e` wipes typed values, `q` quits, and Esc does nothing. | A dirty-form guard in FormDialog or page. Esc runs Cancel with a confirm-if-dirty step.
7. **High** | prompts in all apps (Rename, Add variable, alias, filters, capsule rename/export) | `src/widgets/dialog.rs:285` (`prompt` focuses the field without starting edit), `:321-327` (facts ack) | The dialog opens with no caret, and typing does nothing until Enter. jackin's prelude does start editing, so behaviour is inconsistent. | `Dialog::prompt` and ack start editing by default (`prompt_autoedit=true`).
8. **High** | showcase: everywhere a control vanishes or becomes disabled | `src/bin/showcase/app.rs:954-959` uses `ensure_valid` (First); runtime `src/runtime.rs:1149` | After Save, Apply, Run or Remove, focus jumps to the sidebar. | Default `FocusFallback::Nearest` in `runtime.rs` and in the apps' `ensure_valid`.
9. **High** | showcase Pickers (and any popup without a ring stop) | `pages/pickers.rs:371`: the picker isn't pushed through ModalStack | Closing the picker drops focus to the sidebar, not the button that opened it. | Picker, Select and Menu record their owner and restore it on close (`popup::restore_focus`, default on).
10. **High** | footer hints, all apps | Hints are hand-built per page (`grep '"Enter","Edit"'`: about 16 sites); only `Dialog::hints` (`dialog.rs:382`) and `Button::hint` are derived from state | The footer lies: tablepro filter says "Type Filter" but typing does nothing; "Esc Clear" actually reverts or moves focus; the holla and jackin menu bar shows the page's "Type Search…Enter Open"; jackin manager "Tab Actions" goes to the menu bar; "Enter Edit" shows on read-only cells. | A `Hints` trait on every widget. The hint bar composes widget → page → global hints. `Tab` labels come from `ring.next()`.
11. **High** | jackin host menu bar | `src/widgets/menu.rs:998` registers focus; jackin routes keys only while the menu is open (`app.rs:668`) | Tab lands on "File Go Help"; Enter and the arrows do nothing. | MenuBar handles its own closed-but-focused keys (Enter or ↓ opens, ←→ moves), or opts out of the ring with `in_ring` (default true, but self-sufficient).
12. **High** | read-only grids and results | `src/ui/reveal.rs:504-536`; Grid has no keyboard reveal (`grid.rs:2170`); Enter is a no-op (`grid.rs:762`) | Truncated JSON or long text can only be seen with mouse hover. | Grid, Table, Menu, Picker, Select and StatusBar all call `RevealState::cell` (→ reveals, ←/Esc closes). Read-only Enter opens a viewer.
13. **Med** | Select (open) | `src/widgets/select.rs:307` (`_ => Consumed`) | Tab, `?` and `q` are swallowed while the list is open. | Tab chooses and moves on (macOS). `?` passes through. Knob: `tab_in_open: Choose` (default), `Close` or `Swallow`.
14. **Med** | ContextMenu and MenuBar | `src/widgets/menu.rs:376` | Shortcut letters shown in menus ("c", "z", "?", "r", "x") do nothing; no mnemonics or typeahead. | Menus honour item accelerators and typeahead; any other key closes the menu and passes through. Knob: `unknown_key: CloseAndPass` (default).
15. **Med** | TextInput and TextArea Tab handling | `CommittedTab` is left for the caller (`input.rs:338`); jackin editor ignores it (`editor.rs:1549-1562`) | In jackin Name, Tab commits but focus stays; it needs two Tabs. | The widget asks the ring to move itself (`ctx.request_focus_next(backward)`), so apps can't forget.
16. **Med** | Esc semantics while editing | `textarea.rs:191`, `code.rs:456` commit; `input.rs:343` and grid cancel; `field_common.rs:224` doc vs code | Esc saves in one control and discards in the next, with hints of "Done", "Done editing" or "Cancel". | One rule: Esc reverts, Ctrl+Enter or Tab keeps. Documents opt into `esc_commits=true` with an "Esc Done" hint.
17. **Med** | CodeEditor Tab | `code.rs:186` (`tab_leaves=false`), `:460-475` | While editing, Tab indents; the hint doesn't say so, and the find bar swallows Tab. | Hint "Tab Indent · Esc Leave". Add `Ctrl+Tab`/`Ctrl+Shift+Tab` as leave keys that always work. Default `tab_leaves=false` stays.
18. **Med** | Splitter (showcase Terminal, jackin Accounts) | `src/widgets/splitter.rs:1-2,45` (mouse-only by design) | Panes can't be resized; the footer even advertises "drag ┃ Resize". | Splitter gets keyboard resize on the owning pane (Alt+Shift+arrows, as in `capsule.rs:2017`) through a shared `SplitKeys`, auto-hinted.
19. **Med** | Tables `s` Sort | `src/widgets/table.rs:852`: no column cursor | Only the first column can be sorted. | Table gets a column cursor (←→ when `cell_nav`) or a "Sort by…" picker.
20. **Med** | tablepro find bar | `tabs.rs:1033` `is_editing` ignores find | Typing in Find triggers `z`, `[`, `]`. | `CodeEditor::is_editing()` includes find. Widgets expose `wants_text_input()` for app key routing.
21. **Med** | quitting | showcase `app.rs:634` `q`; jackin manager Esc and `q`; holla Esc on empty Here | Leaves immediately, even with unsaved Forms, Settings or grid changes. | Quit goes through one guarded `request_quit`. Esc never quits (`esc_quits=false`).
22. **Med** | `y` quick answer | `dialog.rs:144-149,802-812` | One keystroke runs Delete or Discard even while Cancel is focused. | `quick_answer: Safe` (default): `y` only works on non-destructive dialogs.
23. **Med** | Help | `HelpOverlay` is used only in jackin (`app.rs:1299`); others build their own; holla `?` is typed into search (`finder.rs:1649`); the help lists miss page keys | Help is generic, doesn't open in many places, and is missing grid, editor and sidebar keys. | `HelpOverlay` built from the composed keymaps and hints. F1 always opens it. `?` opens it whenever no text field is editing.
24. **Med** | tablepro editable query results | `tabs.rs:1426-1432` drops save events | Edits are accepted but can never be saved, and quit doesn't warn. | Grid `editable` should follow the owner's save capability (`editable(false)` unless a save handler is wired).
25. **Med** | Grid Enter on a read-only or PK cell | `grid.rs:763-770` returns Consumed silently | "Enter Edit" shown; nothing happens and there is no feedback. | Return `GridEvent::ReadOnly{reason}` and show a status message. Hints drop Edit.
26. **Med** | jackin Capsule prefix | `capsule.rs:56` `PREFIX_TIMEOUT_MS=2000` | Ctrl+B commands time out for slower typists. | Configurable, defaulting to sticky (no timeout) with Esc to cancel.
27. **Med** | narrow tablepro Connections | `connections.rs:1107-1111` | The form covers the list, but the filter and tree stay in the Tab ring. | Content covered by an overlay is `inert` (register a barrier when a full-cover panel draws).
28. **Med** | jackin launch cockpit | Single 1×1 focus stop at (48,8) with nothing drawn | Tab shows no focus at all. | StepRail is a stop only when `selectable`; otherwise it is not in the ring.
29. **Med** | TextBuffer editing keys | `src/widgets/field_common.rs:226-314`; no undo in `core/text.rs`; Ctrl+A is Home, Ctrl+L is select all | No Ctrl+Z, no copy or cut of a selection, and select-all is non-standard. | Add an undo stack and Ctrl+Z / Ctrl+Shift+Z. Add a keymap preset `FieldKeys::{Emacs, Desktop}` (Desktop: Ctrl+A select all).
30. **Low** | jackin Manager order | `manager.rs:1586-1596` | Bottom buttons come before the header detail. | Containers can sort their children by (y, x) (`order: Visual`) independently of draw order.
31. **Low** | Picker | `picker.rs:524`: Tab is NextScope, BackTab unhandled | Can't go back a scope. | BackTab goes to the previous scope.
32. **Low** | FormDialog vs Dialog buttons | `form_dialog.rs:456-471` wraps without skipping disabled; `dialog.rs:861-879` clamps and skips | ←→ behaves differently between dialogs. | One `ButtonRow` navigation with `wrap` (default false) that skips disabled buttons.
33. **Low** | NavKeys vim letters on by default | `core/keymap.rs:100-130` | `g`/`G`/`j`/`k` collide with typeahead and letter maps (Grid, Tabs `n`/`x`, List typeahead eats `j`/`k`, `list.rs:329`). | Default to `NO_VIM`; apps opt in.
34. **Low** | Typeahead | List and Select only, off by default (`list.rs:144`, `select.rs:127`) | No jump-by-letter in trees, menus or long selects. | On by default for Select, Menu and Tree when vim keys are off.
35. **Low** | Busy Button | `button.rs:86-96` stays in the ring and consumes | Enter does nothing, with no hint. | Hint "Working…" from `Button::hints`.
36. **Low** | holla search caret | `finder.rs:807-809` | Caret pinned at the end; ←→ can't move it. | Finder query uses TextInput (always editing).
37. **Low** | Empty focus stops (Tables "Checks", Lists "Search results", Overview) | Empty widgets still call `ctx.control` | Tab lands on nothing, with Move/Select hints. | `skip_when_empty` (default true).
38. **Low** | Status-bar items and the Chips "match all ▾" lead | `chrome.rs:95,103`, `chips.rs:218` | Click-only. | Status bar gets a `focusable` option; ChipBar keymap gets a key for the lead toggle.
39. **Low** | Undiscoverable keys | showcase sidebar →/l, Home/End; F10; Shift+arrow selection; tablepro Ctrl+↑↓ split resize | They work, but no hint or help mentions them. | Comes for free once hints are derived from widgets (#10, #23).

## 3. Per-widget keyboard behaviour (proposed; current behaviour where it differs)

| Widget | Focus | Keys | Caret | Hints |
|---|---|---|---|---|
| Button | `▎` gutter; skipped when disabled; busy shows "Working…" | Enter, Space | none | `Enter <Label>` |
| Checkbox / Toggle | `▎` | Space toggles; Enter toggles (configurable to submit) | none | `Space Toggle` |
| RadioGroup | one stop; `▎` on the current option | ↑↓←→, Home and End select directly | none | `↑↓ Choose` |
| Select (closed) | `▎`, value with ▾ | Enter, Space or Alt+↓ open; ↑↓ step (`SelectArrows::Step`); typeahead | none | `Enter Open · ↑↓ Change` |
| Select (open) | list cursor fill, field keeps `▎` | ↑↓, PgUp/PgDn, Home/End, typeahead; Enter/Space choose; Tab chooses and moves on (today swallowed); Esc reverts | none | `↑↓ Choose · Enter Pick · Esc Close` |
| TextInput | `▎` plus caret | Immediate (today Enter/F2 first); edit keys per `field_common`; Enter applies; Tab applies and moves on; Esc reverts | at the insertion point whenever focused (today only while editing) | `EDIT_FIELD` |
| TextArea | `▎` plus caret | Immediate (today Enter/F2); Enter newline; Tab leaves; Esc reverts (today commits) | whenever focused | `Enter Newline · Tab Next · Esc Cancel` |
| CodeEditor | `▎` plus caret | Enter/F2/`i` edit; Tab indents (`tab_leaves`); Ctrl+Tab leaves; Esc ends the edit; `/` find | while editing (default Immediate when it is the main control) | `Tab Indent · Esc Done · Ctrl+R Run` |
| ListBox | `▎` on the cursor row | ↑↓, PgUp/PgDn, Home/End; Enter/Space activate; Shift extends; → reveal; typeahead opt-in; Shift+F10 menu | none | `↑↓ Move · Enter Open · Shift+F10 Menu` |
| TreeView | `▎` on the row | ↑↓, ←→ collapse/expand or parent/child, `*` and `-`, Enter/Space, Shift+F10 | none | `←→ Fold / open` |
| Tabs | `▎`, active tab neutral | ←→ activate, Home/End, digits, `x` close, `n` new | none | `←→ Switch` |
| DataTable / Grid | `▎` on the row, cell fill | Arrows move the cell; Enter/F2 edit (or a viewer when read-only); typing replaces when the cell option is on; Space marks the row; Tab moves to the next writable cell, then leaves; Esc cancels or clears | in the cell while editing | `EDIT_CELL`; "Read-only" on such cells |
| ChipBar | `▎` on a chip | ←→, Home/End; Enter activates; Space toggles; Delete/Backspace removes; `+` adds | none | `←→ Move · Delete Remove` |
| Panel / Viewport / Diff | title restyle (Panel) or `▎` | ↑↓, PgUp/PgDn, ←→ scroll; Shift+arrows select; `f` follow; `y` copy; End follows; Esc clears selection | browse caret when focused | `↑↓ Scroll · Shift+↑↓ Select · y Copy` |
| PropsList | `▎` on the row | ↑↓, Enter, `y` copy, → reveal | none | `→ Show full` |
| MenuBar | `▎` on the title | F10 or Alt focuses; ←→; Enter or ↓ opens | none | `←→ Menu · Enter Open` |
| ContextMenu | cursor fill | ↑↓, Home/End, Enter/Space, accelerators and typeahead; Esc closes; other keys close and pass through | none | `↑↓ Move · Enter Choose · Esc Close` |
| Picker | `▎` on the query, row fill | type to search; ↑↓, PgUp/PgDn; Enter; Alt+Enter; Tab/BackTab change scope; Esc clears, then closes | always in the query | given, plus `Tab Scope` |
| Completion | row fill | ↑↓, Ctrl+N/P; Enter or Tab accept; Esc dismiss | editor caret | `Tab Accept` |
| Dialog / FormDialog / ChoiceDialog | modal barrier; primary or safe default focused | Tab/BackTab cycle; ←→ buttons (no wrap, skip disabled); Enter primary; Esc per `Dismiss`; `y`/`n` only when safe | the prompt field starts editing | `Dialog::hints` |
| HelpOverlay | modal | ↑↓, PgUp/PgDn; Esc, `?`, F1, `q` close | none | `↑↓ Scroll · Esc Close` |
| Splitter | not a stop (today mouse-only) | owning pane gets Alt+Shift+arrows | none | `Alt+Shift+←→ Resize` |
| Scrollbar | not a stop | owner's PgUp/PgDn, Home/End | none | owner's |
| Reveal (tooltip) | n/a | → opens, ←/Esc close, PgUp/PgDn scroll; every truncating widget (today List and Props only) | none | `→ Show full` |

The walk transcripts were saved under `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/kb-audit/`: `kb_showcase.txt`, `kb_tablepro.txt`, `kb_tp2.txt`, `kb_holla.txt`, `kb_holla_menubar.txt`, `kb_jackin-preview.txt` and the `dump_*.txt` frame captures.

The snapshot text captures are stale (dated 14 Sep, code 16 Oct), so their footers weren't used as evidence for hint mismatches.

## Status (caret rules and edit entry)

Section 1 "Caret rules", corrected by the standards review (keyboard focus
selects a field's value; grid cells stay in navigation; multi-line Esc keeps
the text; Ctrl+Tab is not advertised; readline keys stay the default).

| # | Status | Commit(s) | Notes |
|---|---|---|---|
| 1 | Fixed | d5ad6ffb6, 1da20cc01 | `EditEntry { Immediate, TypeToEdit, Explicit }` on `TextInput`, `TextArea`, `CodeEditor` and `Dialog::edit_entry`, app-wide `Theme::edit_entry` (default `Immediate`). Keyboard focus starts the edit at render with `FocusSelect::SelectAll` (single-line) or `CaretEnd` (TextArea); a click keeps the caret at the pointer; typing replaces the selection; the placeholder stays, with the caret on it, until the first character. Proof: Tab walks with a caret at every text stop in showcase Inputs, tablepro (connection form, query editor), holla arguments and jackin Name. |
| 5 | Fixed | 1da20cc01 | The SQL editor edits on focus; typed `q`, `z`, `[`, `]` go into the SQL. Tab indents (`tab_leaves=false`), Esc is "Done", then Tab leaves; Ctrl+Tab / Ctrl+Shift+Tab leave where the terminal delivers them (never hinted). Footer: `Tab Indent · Esc Done`. |
| 7 | Fixed | 1da20cc01 | `Dialog::prompt` and the typed acknowledgement open with the caret in the field; Esc in an immediate prompt cancels the dialog at once; a submit blocked by validation hands the caret back (`TextInput::resume`). |
| 15 | Fixed | d5ad6ffb6 | `core::focus::request_focus_next`: Tab out of a text control asks the ring to move on; `Focus::settle` applies it after each input in all four apps; an owner that moves focus itself cancels it. jackin's Name field now leaves on one Tab. |
| 16 | Fixed | d5ad6ffb6, 1da20cc01 | One rule: a single-line field's Esc reverts while dirty, otherwise it passes on (page Esc ladder); multi-line documents (TextArea, CodeEditor) end the edit keeping the text ("Done"), `esc_reverts` opts into reverting. Every revert is undoable. Hints use `EDIT_FIELD`, `EDIT_TEXT`, `EDIT_CODE`. |
| 17 | Fixed | 1da20cc01 | See #5; `CodeEditor::hints` derives the hint from `tab_leaves`/`esc_reverts`. Leave = Esc then Tab; F6 pane cycling is not added here (no pane-cycle mechanism yet). |
| 20 | Fixed | 1da20cc01 | `CodeEditor::wants_text_input` counts the find bar; tablepro `QueryTab::is_editing` uses it, so Find takes `z`, `[`, `]`. `TextInput`/`TextArea::wants_text_input` give apps the same question. |
| 29 | Fixed | d5ad6ffb6 | `UndoStack` (typing folds by word): Ctrl+Z, Ctrl+Shift+Z. `FieldKeys::Emacs` (default; Cocoa/readline: Ctrl+A line start, Ctrl+L select all, Alt+W copy, Ctrl+W cut or word, Ctrl+Y yank, Ctrl+H delete left) and `FieldKeys::Desktop` (Ctrl+A select all, Ctrl+C/X/V, Ctrl+Y redo); `Theme::field_keys`. Copy/cut go to `core::edit::clipboard` with a `set_hook` for the system clipboard (OSC 52 or a platform crate). A multi-line paste into a single-line field flattens line breaks to spaces. Desktop's Ctrl+C reaches a field only where the app hands Ctrl+C to the page first. |
| 36 | Deferred | — | holla's finder query keeps its own string. Its keys are pinned by parity spec HP02 (Ctrl+A select all, Ctrl+W belongs to the shell, its own undo), which differ from `FieldKeys::Emacs`, and `TextInput` draws a label and help row the finder's one-row query has no room for. Needs a one-row `TextInput` presentation and a finder key preset; the query already always shows its caret. |

Also: app key routing gives a focused editable control printable keys
before page and app letter shortcuts in all four apps (the immediate field
is editing, so the existing `editing` gates hold; typed keys never reach
`q`, `e`, `[`, `]`, `z`, `y`, `n`); keys a field has no use for (↑↓, PgUp,
unbound chords) pass on from an immediate edit, so a list under a search
field still moves. Grid and table cells are unchanged (Enter or F2 edits,
APG grid). Dialog Enter in a form activates the default button (the typed
confirm change, 3ec122eb2, owns that).

Visual: carets and selected values now show on focused fields (intended).
The in-repo showcase baseline was regenerated (Inputs, Forms, Text areas,
Code editor). The approved tuiscotti smoke tier (120x40 truecolor and 72x20
nocolor, 910 scenarios) still passes at 1da20cc01. Scenarios still pending
re-approval (`VISUAL_FULL=1`) that stop on a text control will re-approve
with the caret: holla `flows::args`, `flows::finder::query*`,
`flows::query::*`, `flows::*::gate2_typed`, `flows::child::query`,
`flows::remote::query`, `flows::task::input_*`; jackin
`accounts::add_form*`, `accounts::filter`; showcase pages `inputs`, `forms`,
`textareas`, `codeeditor`, `dialogs`; tablepro scenarios on the query
editor, filter editor and connection form. They were not re-approved here.
