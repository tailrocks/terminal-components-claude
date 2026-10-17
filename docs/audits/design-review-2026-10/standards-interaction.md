## Verdict table

| # | Item | Verdict | One-line correction |
|---|---|---|---|
| 1 | Popup outside-click (PO) / consume by default, completion pass-through (C) | Agree | Keep it. If there is no room to flip or shift the popup, show it full-width as a sheet. Never clip it. |
| 2 | Backdrop light-dismiss for dialogs (PO/C) | Agree with adjustment | Light-dismiss only for dialogs with nothing to lose: info, confirm, destructive (it means Cancel). Turn it off for a prompt once its input is dirty. Ignore the click that focuses the terminal window. |
| 3 | "Every behaviour configurable" (PO) | **Contradicts** | Configurability does not replace good defaults. Expose app-level policies and end-user key remapping, not per-widget knobs for everything. |
| 4 | Menu bar hover-switch (PO/C) | Agree with adjustment | Needs any-motion mouse tracking (mode 1003) and must degrade without it. Add press-drag-release item selection. |
| 5 | Ctrl-click = secondary on macOS; Option-click toggles rows | Agree with adjustment | Detect the client terminal, not `cfg!(target_os)`. Option-click is a fallback only. Clicking a checkbox cell must toggle with no modifier. |
| 6 | `EditEntry::Immediate` | Agree with adjustment | Keyboard focus selects all (desktop). Pointer focus puts the caret at the click. Grid cells default to `Explicit` (APG grid). |
| 7 | Keyboard-only + mouse-first + on-screen hints (PO) | Agree with adjustment | Hints must be contextual and list only keys the terminal can actually deliver. Add a `?` overlay, and generate docs from the keymaps. |
| 8 | Truncation: hover tooltip, → expands row | Agree with adjustment | → already means expand children in trees and next column in tables with horizontal scroll. Show the full value on focus (status/peek line) or on a dedicated peek key. |
| 9 | H-scroll: Auto bar, Shift+wheel, track click = Jump | **Contradicts** (track click only) | Default track click to `Page` (macOS and Windows default). `Jump` becomes an option. Also accept native horizontal wheel buttons. |
| 10 | Context menus; F10 = menu bar | Agree with adjustment | F10 is usually unreachable: it is a media key on Mac keyboards and GNOME Terminal grabs it. Add a second documented binding and Alt+mnemonics. |
| 11 | Dialog Enter/Esc/y/n/focus Cancel | Agree with adjustment | On a destructive dialog, make Cancel both the default and the focused button so Enter is safe either way. Use label mnemonics instead of y/n when labels are verbs. |
| 12 | Vim keys ON, type-ahead OFF | **Contradicts** | Flip both: `NavKeys::NO_VIM` default with vim as an opt-in profile; type-ahead ON in lists and selects. |
| 13 | Closed Select ↑↓ = Step; Tab in open select chooses and moves on | Agree with adjustment | Default `Open` (APG/macOS), `Step` as an option. The Tab behaviour is correct per APG. |
| 14 | List single click = Activate | **Contradicts** for data views | Data lists, tables and trees default to `Select` (double-click or Enter activates). Keep `Activate` for nav lists, sidebars and menus. |
| 15 | Focus ring always; KeyboardOnly optional | Agree | Text inputs show focus always, even in KeyboardOnly mode. |
| 16 | Input coalescing | Agree with adjustment | Only coalesce wheel, motion, resize and auto-repeat navigation. Never drop or merge typed characters or pastes. Drain all pending keys before drawing. |
| 17 | Esc ladder, Ctrl+C guards, quit keys | Agree with adjustment | Ctrl+C copies when there is a selection. Confirm quit only when data would be lost. Esc never quits. |
| 18 | Blur commits; dead-space click ends edit | Agree | An invalid value stays marked; focus is never trapped. |
| 19 | TextArea Esc reverts; CodeEditor Ctrl+Tab leaves | **Contradicts** | Esc in multiline should leave editing and keep the text (revert via undo). Ctrl+Tab does not reach the app in most terminals, so use Esc then Tab, plus F6. |
| 20 | `FieldKeys::Desktop` (Ctrl+A select all) "per macOS" | **Contradicts** | On macOS, Ctrl+A is line start; Cmd+A selects all and Cmd is not delivered. Keep readline/Emacs as default; Desktop is a Windows/Linux option. |
| 21 | Disabled items visible with reason | Agree with adjustment | Put the reason in the status line or tooltip, not in the menu's width. Keep disabled items focusable but not activatable. |
| 22 | Focus restoration, `FocusFallback::Nearest` | Agree | Also keep Tab cycling inside modals. |
| 23 | Tabs: digits, x/Delete, n, middle-click, Home/End | Agree with adjustment | Letter and digit keys only while the tab strip has focus. Add global Ctrl+PgUp/PgDn. Closing a dirty tab goes through the unsaved guard. |
| 24 | Selectable table keys; "Discard unsaved changes?" on selection change | Agree with adjustment | Keys are correct. Commit on selection change (matches #18) and ask only if validation fails, as Save / Don't Save / Cancel. Shift+click range is mostly not delivered (gap 2). |

## Detailed notes

**2. Backdrop dismiss.** Apple alerts and sheets do not dismiss on an outside click (https://developer.apple.com/design/human-interface-guidelines/alerts). WinUI ContentDialog is not light-dismissable; flyouts are (https://learn.microsoft.com/en-us/windows/apps/design/controls/dialogs-and-flyouts/). APG dialog requires only Esc (https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/). Light-dismissing a confirm or destructive dialog is safe because the result is Cancel. Typed input is not safe to throw away.
- Default ON for confirm, info and destructive.
- Default OFF for a prompt once it is dirty.
- Require press and release both on the backdrop (not the end of a drag).
- Terminal constraint: the click that activates the terminal window is often forwarded to the app. Enable focus reporting (`CSI ?1004h`) and ignore clicks for about 300 ms after FocusIn.

**3. Configure everything.** This contradicts consistency (Nielsen heuristic #4, https://www.nngroup.com/articles/ten-usability-heuristics/) and multiplies the test matrix. The configuration that standards actually require is end-user remapping or disabling of single-character shortcuts (WCAG 2.1.4, https://www.w3.org/WAI/WCAG22/Understanding/character-key-shortcuts.html). Correct shape:
- A few app-level policy profiles (e.g. `Desktop`, `Vim`, `Mc`) plus a keymap file.
- Per-widget knobs only where real apps legitimately differ: dismiss, click action, edit entry.
- "Hardcoded" is fine for behaviour that is a standard, such as Esc closing popups.

**4. Hover switch.** Correct macOS behaviour (https://developer.apple.com/design/human-interface-guidelines/the-menu-bar). It only works under any-motion tracking (`?1003`). Some terminals (and tmux without passthrough) send motion only while a button is held. Without it, fall back to click-to-switch and arrow keys (APG menubar ←/→, https://www.w3.org/WAI/ARIA/apg/patterns/menubar/). Also add press, drag to item, release to activate.

**5. Ctrl-click / Option-click.**
- `cfg!(target_os = "macos")` (`src/core/event.rs:166`) checks where the binary runs. Over SSH from a Mac to a Linux server it is off. On a Linux desktop terminal it would wrongly be on, and Ctrl+click there means "toggle".
- Detect the client instead: `TERM_PROGRAM` (Apple_Terminal, iTerm.app, WezTerm, ghostty) or `LC_TERMINAL`, which iTerm forwards over SSH by default.
- Option-click is unreliable: iTerm2 and Terminal.app use Option-click to move the cursor and Option-drag for block selection, and may not report the modifier. Keep it, but toggling must always also work by clicking the checkbox cell and with Space.
- The Mac-standard toggle is Cmd-click. It is only available through the kitty keyboard protocol, which reports super (https://sw.kovidgoyal.net/kitty/keyboard-protocol/).

**6. EditEntry.**
- Tabbing into a text field selects its whole contents on both Windows and macOS (Windows UX guide, text boxes: https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-text-boxes). Caret-at-end is a deviation. Default `SelectAll` on keyboard focus, `CaretAtClick` on pointer focus, `CaretEnd` as an option.
- Grid and table cells must keep navigation mode, with Enter or F2 to edit (APG grid, https://www.w3.org/WAI/ARIA/apg/patterns/grid/#keyboardinteraction-settingfocusandnavigatinginsidecells).
- Enter inside a form field submits the default button, not just "apply".
- "Printable keys go to the field first" is exactly what WCAG 2.1.4 requires. Keep it.

**7. Hints.**
- Keep a contextual key-hint bar (mc/lazygit style) of 3–6 items, plus a `?` overlay generated from the live keymaps so it cannot drift from the code.
- Hide hints for chords the terminal cannot deliver (Ctrl+Tab, Cmd, Ctrl+Shift+letter without the kitty protocol).
- Mouse affordances (`[x]`, `▾`, scrollbar thumb) are the mouse-side hints.

**8. Truncation.**
- The macOS expansion tooltip is right. Under WCAG 1.4.13 (https://www.w3.org/WAI/WCAG22/Understanding/content-on-hover-or-focus.html) it must be dismissible with Esc, hoverable, and persistent.
- The Finder analogy fails: → in Finder expands folders, not text. In trees, APG reserves → for children (https://www.w3.org/WAI/ARIA/apg/patterns/treeview/); in tables with horizontal scroll, → scrolls.
- Keyboard equivalent: on focus, show the full value in the status/peek line automatically (no keypress needed), and add an explicit peek key (e.g. `Space` in read-only lists, or `i`) to expand inline.
- Inline-wrap and `y` copy: keep.

**9. Track click.** macOS System Settings defaults "Click in the scroll bar to: Jump to the next page" (https://support.apple.com/guide/mac-help/change-appearance-settings-mchlp1225/mac). Windows pages too. Default `Page`; `Jump` as an option or on Shift-click (GTK convention).
- Shift+wheel: many terminals send it as plain wheel or scroll their own scrollback. Also handle SGR buttons 66/67 (horizontal wheel), which iTerm2, kitty and WezTerm send for trackpad horizontal swipes.

**10. F10.** F10 for the menu bar is a Windows/GNOME convention (https://learn.microsoft.com/en-us/windows/win32/uxguide/inter-keyboard). In terminals:
- GNOME Terminal grabs F10 by default.
- Mac keyboards need Fn.
- mc and htop users expect F10 to quit.

Add Alt+mnemonic to open a specific menu, and show the binding in the hint bar. Right-click while a popup is open (close and reopen at the new spot) matches both macOS and Windows. Shift+F10 and the Menu key: correct.

**11. Dialogs.**
- Apple: never make a destructive button the default, and prefer verb labels over Yes/No (alerts HIG above).
- macOS: Return activates the default button, Space the focused one. Windows: Enter activates the focused button. With Cancel both default and focused on destructive dialogs, the platforms agree.
- Replace y/n with mnemonics taken from the labels (`[D]elete` / `[K]eep`), shown on screen and active only when the dialog has no text field.
- `quick_answer` "y when danger focused" is redundant. Drop it.

**12. Vim/type-ahead.** APG listbox: "Type-ahead is recommended for all listboxes, especially those with more than seven options" (https://www.w3.org/WAI/ARIA/apg/patterns/listbox/#keyboardinteraction). Type-ahead is native in Windows, macOS and GNOME lists. hjkl as a default fails WCAG 2.1.4 unless remappable and blocks type-ahead. Default `NavKeys::NO_VIM` and type-ahead ON, with a `Vim` profile opt-in. g/G go with it.

**13. Select.** APG select-only combobox: Down Arrow opens the list; Alt+Down opens it; Tab selects the current option, closes and moves on (https://www.w3.org/WAI/ARIA/apg/patterns/combobox/examples/combobox-select-only/). macOS pop-up buttons open on arrow or Space. `Step` (Windows/HTML-on-Windows) fires a change event on every keypress, which is bad when changes have side effects. Default `Open`; `Step` as an option.

**14. Click action.** Windows UX guide: single click selects, double-click performs the default action (https://learn.microsoft.com/en-us/windows/win32/uxguide/inter-mouse). macOS Finder/Mail and GNOME are the same. Single-click activate is correct only for navigation lists, menus, links and launchers. Make the default depend on the widget's role: data views `Select`, nav/menu `Activate`. Double-click must be synthesized with a timing threshold (~400 ms, configurable), because terminals report single clicks only.

**16. Coalescing.** Merging non-repeat keys reorders or loses text. Coalesce only wheel, motion, resize and auto-repeat of navigation keys. Process every pending key before rendering, so a frame never delays input. Use bracketed paste (`?2004`) so a paste is never interpreted as shortcuts.

**17. Esc / quit.**
- Ctrl+C: when text is selected, copy (Windows Terminal convention). Otherwise quit, through the guard.
- Apple: don't ask to confirm quitting unless data would be lost. Confirming every quit is nagging.
- Esc latency: use the kitty protocol where available to disambiguate Esc; otherwise use a short timeout (≤ 50 ms; vim `ttimeoutlen`, ncurses `ESCDELAY`).
- "Leave pane" belongs last on the ladder, after "clear selection" (#24).

**19. TextArea/CodeEditor.**
- No desktop toolkit throws away a whole multiline edit on one Esc. Default: Esc ends editing and keeps the text ("Done"); revert via undo, or via the dialog's Cancel.
- In legacy encoding, Ctrl+Tab arrives as plain Tab, so a CodeEditor with "Tab indents" becomes a keyboard trap (WCAG 2.1.2, https://www.w3.org/WAI/WCAG22/Understanding/no-keyboard-trap.html).
- Use Esc then Tab or Shift+Tab (helix/vim-like), plus F6 for pane cycling. Allow Ctrl+Tab only when kitty flags confirm it is delivered. The VS Code-style toggle (Ctrl+M) is impossible because Ctrl+M is Enter.

**20. Field keys.** macOS Cocoa text fields natively use Emacs bindings: Control-A goes to the start of the line (https://support.apple.com/en-us/102650). Select all is Cmd+A, which terminals don't deliver. Readline users expect the same.
- Default `Emacs` on macOS and in terminals generally. `Desktop` (Ctrl+A select all) as an option, perhaps the default on Windows.
- Ctrl+Backspace is often indistinguishable from Backspace or Ctrl+H, so offer Alt+Backspace or Ctrl+W.

**21. Disabled items.** Keeping disabled items visible matches Apple (https://developer.apple.com/design/human-interface-guidelines/menus). APG allows disabled items to be focusable. Show the reason on focus or hover in the status line rather than widening every menu. A fixed suffix truncates in narrow menus.

**23. Tabs.**
- APG tabs allows Delete to close (https://www.w3.org/WAI/ARIA/apg/patterns/tabs/), but only while the tab strip has focus.
- Use Alt+digit for global jumps (bare digits clash with fields).
- Ctrl+Tab is not delivered, so use Ctrl+PgUp/PgDn. They are delivered by most terminals and are the browser/VS Code standard.
- Middle-click: Linux terminals paste instead when mouse reporting is off. Fine as an extra, never the only way.

**24. Table selection.**
- Keys match APG listbox/grid multi-select.
- A modal "Discard?" on every selection change punishes routine navigation. Commit on selection change (consistent with #18). Only if validation fails, show Save / Don't Save / Cancel (the Apple pattern).
- Esc clearing a selection is non-standard on Windows. Keep it, but as a ladder step before "leave pane".

## Gaps not yet raised

1. **First click after window focus.** Covered in #2: enable `?1004`, ignore pointer input briefly after FocusIn. Otherwise a click on the terminal window dismisses or activates things.
2. **Shift+click is usually not reported.** xterm, GNOME Terminal, iTerm2 and kitty reserve Shift(-drag) for native text selection while mouse capture is on. Shift-click range selection needs a fallback (click the anchor, then Shift+arrows).
3. **Copy story under mouse capture.** Mouse capture disables native terminal selection. Provide in-app selection plus OSC 52 clipboard writes, and document the terminal's bypass modifier.
4. **Kitty keyboard protocol.** Enable it as progressive enhancement (Cmd/super, Ctrl+Tab, unambiguous Esc, key-release). Feed what it detects into the hints so they only show keys that actually work.
5. **Bracketed paste.** Multi-line paste into a single-line field should flatten or warn. Pasted text must never trigger shortcuts.
6. **Never rely on colour alone (WCAG 1.4.1).** Focus and selection need a non-colour cue (marker glyph, reverse video, bold) so they survive `NO_COLOR`, monochrome and colour-blind themes.
7. **Undo/redo in fields and tables.** Ctrl+Z is safe in raw mode (no SIGTSTP). It is also the right replacement for "Esc reverts" (#19).
8. **Resize while open.** Popups and tooltips reposition on resize. Focused content stays visible (WCAG 2.4.11 Focus Not Obscured).
9. **Validation timing (WCAG 3.3.1/3.3.3).** Show errors on commit/blur, not per keystroke. The error message lives near the field and in the status line.
10. **Wide characters and type-ahead timing.** CJK and emoji width must be handled in truncation, ellipsis and column snapping. The type-ahead buffer needs a ~500–1000 ms reset (APG) and wrap-around.

## Product owner decision

Defaults must follow the standards cited in this review. Every behaviour
stays configurable, so users who prefer different behaviour can override
the defaults (app-wide profile, per-widget setting, or key map; see
`AGENTS.md`, "Component architecture").

## Status (plan S4)

- Verdict 12 (vim keys on, type-ahead off): fixed. `NavProfile::Desktop` is the default (desktop keys, type-ahead on in lists, trees, selects and menus); `NavProfile::Vim` is the opt-in profile. Disabled rows stay skipped by the arrows (`Cursor::skip_disabled`, per widget), as before.
- Gap 10 (type-ahead timing): the reset is `TypeAhead::DEFAULT_TIMEOUT` (1 s), configurable per widget (`type_ahead_timeout`); the search wraps. Wide-character truncation is not part of S4.

## Status (T19 sync, 2026-10-17)

Each row was checked against the code at `942b869d7`, the plan's §4 table, its "Step status" and outcome sections, and `docs/remaining-work.md`; verdict 12 and the type-ahead half of gap 10 are covered by "Status (plan S4)" above.

| # | Status | Commit(s) / owner | Notes |
|---|---|---|---|
| 1 | Open → A11 | — | Consume-by-default and completion pass-through hold (`popup::Dismiss`). Remaining: a full-width sheet when the popup can neither flip nor shift, never clipped. |
| 2 | Fixed | 55d88297c, 3ec122eb2, b34ead943 | `Dismiss` policy per dialog; `guard_dirty_prompt` (default on) refuses light dismiss of a dirty prompt; press and release must both land on the backdrop (`core::pointer`); SIg1 guard. |
| 3 | Open → A11 | — | Standard defaults plus per-widget overrides are in place (AGENTS.md); app-level profiles beyond `NavProfile` and a user keymap file are not. |
| 4 | Fixed | 55d88297c, 1cee36e98 | `MenuBar::hover_switch`; hover switching stops when the terminal reports no buttonless motion (`Interaction::motion_unreported`); `MenuBar::press_drag` press-drag-release (M12). |
| 5 | Fixed (S9) | 85cec55af; S9 "RowSelection, ClickPolicy and desktop click modifiers" | Client detection (`ClientPlatform::from_env`: `LC_TERMINAL`, `TERM_PROGRAM`) and Option-click toggle landed; the toggle sets live in `SelectGesture` (`toggle_mods_for`). A click on a check box (table checkbox column, list `✓` column, a tree node's mark) toggles with no modifier (`CheckClick::BoxOnly`; `a_click_on_the_mark_column_toggles_through_the_hit_regions`). |
| 6 | Fixed (S11 recorded) | d5ad6ffb6, 1da20cc01; S11 | Text fields: `EditEntry::Immediate` default, keyboard focus `FocusSelect::SelectAll`, click places the caret (keyboard.md #1). Grid cells stay `Explicit` (Enter/F2 or a double click); an Enter/F2 cell edit starts with the caret at the end (the spreadsheet F2 convention), typing never selects-all a cell. Every one is an `EditField` since S11. |
| 7 | Fixed (S8b) | "HasHints, HintCx and Bindings", "Footer composes hints", "HelpOverlay::from_keymaps" | `shell::Footer` drops a Cmd chord until the kitty protocol confirms it and never names Shift+click/drag; `Bindings` and `ShellKeys::help_section` leave undeliverable chords out; `?`/F1 open `HelpOverlay::from_keymaps` in every app. Navigation hints come from each widget's `NavKeys` (CO3). |
| 8 | Partial (S10) | 0e99f5329 (delay), "the peek line shows the focused cut value in full" (peek line) | M25 done. The peek line shows the focused cut value in the footer's status slot on focus (List, Props, Table, Grid; `Footer::peek`). → stays the keyboard reveal only in List and Props, where it means nothing else; a dedicated peek key is open (Space activates, letters type-select). Reveal for cut labels in A12. |
| 9 | Fixed | 9315b44f1 | `TrackClick::Page` is the default with hold-repeat (M35); `Jump` stays an option. Native horizontal wheel (66/67) maps to `WheelLeft`/`WheelRight` in `core::event`. |
| 10 | Partly fixed (S8a); rest A11 | "KeyRouter, ShellKeys and ShellAction" | `ShellKeys` binds the menu bar to F10 and Ctrl+F2 (macOS "move focus to the menu bar"), both overridable. Alt+mnemonics per menu with F10 shown in the hints stay A11. |
| 11 | Fixed | 4b857a079, 3ec122eb2 | `Dialog::destructive` opens on Cancel (`initial_focus: cancel_id`; test `a_destructive_dialog_names_the_focused_cancel_button_not_confirm`); label mnemonics replace y/n. Plan S5 deviation 1. |
| 13 | Fixed (S12) | "↓ on a closed select opens its list (SI13)" | `Select::arrows` defaults to `SelectArrows::Open` (↑/↓ open the list on the current value, ←/→ pass on); `Step` stays the option (`closed_arrows_step_or_open_per_config`). Tab-in-open-select behaviour already matches APG. |
| 14 | Fixed (S9) | S9 table, list, tree and props commits | `ClickPolicy::DATA` (double click activates) is the default of every data view; `ClickPolicy::NAVIGATION` (one click) for navigation lists and sidebars (the showcase sidebar uses it); menus keep their own one-click choice. Double click is the router's timed count (`PointerConfig::double_click_ms`, 500 ms). |
| 15 | Fixed | 63b9ac943 | `FocusVisible::{Always, KeyboardOnly}` (default `Always`) exists on buttons only; text inputs and areas have no such option, so they always show focus. |
| 16 | Fixed (already) | — | Verified in S8a: the runtime `Coalesce` policy folds only wheel notches, pointer moves, resizes and repeats of one navigation key; typed characters, Enter, Tab and pastes never fold, and `drain_pending_input` (S2) drains queued keys. No router change needed. |
| 17 | Open → S14 | — | Ctrl+C copies with a selection, else `Page::on_interrupt`, else quit through `LeaveGuard` (`QuitKeys`); quit confirmation only when data would be lost (`App::stakes`). |
| 18 | Fixed | e184db2b0, 0d785adad | Blur commits (`BlurAction::Commit` default) and a dead-space press ends the edit; see interaction.md #15. Invalid values stay marked; validation timing is SIg9 (S11). |
| 19 | Fixed (S11) | d5ad6ffb6; S11 F6 | `esc_reverts` defaults to false: Esc ends the multiline edit and keeps the text. A CodeEditor is left with Esc then Tab, or F6 / Shift+F6 (S11); the F6 hint waits for S8b's derived hints (keyboard.md #17). |
| 20 | Fixed | d5ad6ffb6, 170ce0e90 | `FieldKeys::for_platform`: Emacs bindings on macOS clients, `Desktop` elsewhere; Alt+Backspace and Ctrl+W delete a word in both presets. |
| 21 | Fixed (S12) | S12 "a disabled row keeps its dimmed shortcut and says why in the status row" | A disabled menu row keeps its shortcut in its disabled ink; its reason (`Item` reason via `MenuItem::reason`) never widens the menu and shows dimmed in the menu's status row (the bottom border, cut to fit, clear of the ▼ marker) while the cursor or the pointer is on the row (`OptionList::status`). The keyboard cursor reaches disabled rows (`ContextMenu::skip_disabled`, default off); Enter, a click, a secondary release and the row's shortcut never activate them. Every menu follows (MenuBar, Tabs `+N`, ChipBar `+N`, Toolbar `»`). Test `an_unavailable_item_keeps_its_dimmed_shortcut_and_says_why_in_the_status_row`. |
| 22 | Partial; Open → A11 | 7bf62651d, 777adce9a, 3825da948 | `FocusFallback::Nearest` is the runtime fallback (K8, S7). Tab cycling inside modals is A11. |
| 23 | Partly fixed (S8a) | "KeyRouter, ShellKeys and ShellAction" | Ctrl+PgDn/PgUp (`ShellAction::NextPage`/`PrevPage`) step pages or tabs in the showcase, tablepro and holla. tablepro's `[`/`]` and holla's Alt+digit stay app bindings (overridable `ShellKeys`); letters and digits on a focused strip are `Tabs::keymap`. Dirty-tab close through the guard is S14. |
| 24 | Fixed (S9 verified; page guard → S18) | — | Table and grid clicks commit a valid edit on a selection change and keep an invalid one with its error (`clicking_another_cell_preserves_invalid_edit_and_cursor`); the showcase Selectable editor asks Save / Don't Save / Cancel only when its edits are invalid (`changing_the_selection_commits_valid_edits`, `changing_the_selection_with_invalid_edits_asks_save_dont_save_cancel`), its right click included since S9. That editor guard is page code: it moves into the library with `FormPanel` (S18). |
| Gap 1 | Fixed | b34ead943 | The runtime enables `?1004`; `runtime::FocusClickGuard` drops the click within 300 ms of FocusIn (`set_focus_click_guard`). |
| Gap 2 | Fixed (S9) | S9 list commit | A click sets the anchor (`SelectGesture::Only`) and Shift+arrows extend from it, growing and shrinking (`SelectGesture::Extend`; `shift_arrows_extend_and_shrink_from_the_clicked_anchor`), in List and Table. |
| Gap 3 | Open → S14, A11 | — | S14 owns the `Clipboard` and OSC 52 writes; documenting the terminal's bypass modifier and in-app selection under capture is A11. |
| Gap 4 | Fixed (S8b) | "Footer composes hints" | `shell::deliverable` and `KeyBinding::deliverable` feed `super_delivered` into the footer, help and component docs: a Cmd chord shows only once the terminal delivers it. |
| Gap 5 | Fixed (S11) | S11 | A bracketed paste reaches the focused control as `Input::Paste`, never as keys, in every app; `EditField::paste` is one undoable edit and flattens line breaks in every one-line field (TextInput, cells, Picker, Combo, find, finder). Table and grid cells took the raw newline before (`the_cell_editor_selects_by_pointer_undoes_and_flattens_pastes`). |
| Gap 6 | Fixed (S10) | "a selected row is bold" | Selected rows are bold at every colour level (`StateModifiers::selected`). |
| Gap 7 | Partial; Open → A21 | d5ad6ffb6 | Fields: Ctrl+Z undo and Ctrl+Shift+Z redo (`UndoStack`, K15). DataGrid undoes pending changes with `u` only; DataTable has no undo. No step owns table undo. |
| Gap 8 | Fixed | 3d0f10bd8 | Popups re-place against the current screen every frame; a showcase test locks it (S5). Focus kept visible is SVg9 in `standards-visual.md`. |
| Gap 9 | Fixed (S11) | S11 | `ValidateWhen { Commit, AfterError (default), Live }` on `EditField` (`TextInput::validate_when`): an ended edit validates; while an error shows, typing re-checks. The grid cell drift (it cleared on any typed character) is fixed (`a_cell_edit_rechecks_after_an_error_undoes_and_flattens_pastes`). The error text stays under the field; the form-level timing is D3. |
| Gap 10 | Partial; Open → A11 | f46630729 | Type-ahead reset and wrap: see "Status (plan S4)". Wide-character truncation, ellipsis and column snapping remain A11. |

## File browser keys (S22, 2026-10-17)

Decisions for `widgets::file_browser::FileBrowser` (reference: the macOS
Open panel, then GNOME's file chooser):

- Enter (and →) opens the folder under the cursor; Space chooses. In the
  macOS panel Return presses the default button and ⌘↓ goes into a
  folder; a terminal has no Cmd, and every terminal file manager uses
  Enter to go in, so Enter keeps that meaning and Space is the choose key
  (jackin's shipped behaviour). Both are `FileCmd` bindings an app remaps
  (`FileBrowser::keys`); `FileCmd::vim_keymap` adds `h`/`l`/`s`/`g`.
- `.` shows or hides hidden files: Finder's ⇧⌘. without the Cmd the
  terminal does not deliver. Punctuation is not type-ahead, so letters
  keep jumping. A "Show hidden files" check box offers the same to the
  pointer (`hidden_toggle`, default on).
- `/` or `~` types a location (Finder's Go to Folder, which also opens on
  those keys); Ctrl+F moves to the optional name filter.
- The path bar (`path_bar`) is off by default: the editable path field
  already names the folder, and the panel stays one row shorter.
- Esc, the close button and an outside click resolve to `Cancelled`
  exactly as the `Dismiss` policy allows (default `ESCAPE`); the footer
  shows Esc only when it closes.

