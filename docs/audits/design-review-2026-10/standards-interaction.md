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
