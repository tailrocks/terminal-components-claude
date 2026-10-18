## Scope and method

Before-refactor evidence set for `showcase`, `tablepro` and `jackin-preview` at commit `d5e7075`
(binaries from `cargo build --bins`, debug profile). Driven by `tools/baseline_capture.sh`
(tmux pane, real binaries, real key/mouse input; deterministic scenarios, `--motion paused`
with fixed `--frame` for Jackin). Every capture below was reached and inspected; the state
markers listed in the "Steps" column are the exact tmux key names / SGR mouse events sent.

Coverage:

* showcase: all 22 pages at 80×24, 100×30, 120×40, 160×50 (truecolor); overview/buttons/inputs/tables
  at 120×40 in `--color 256`, `--color 16`, `--color none`; representative states at 120×40
  (focused, hovered, pressed, activated, disabled hover/click, selected row, sorted, error field,
  busy/progress, editing input/cell/textarea, validation error, overflow + wheel scrolling,
  empty state (Tables "No checks have run yet", Chips "Empty state" card, Lists empty filter text),
  confirm/prompt/destructive dialogs, menu bar + context menu (keyboard `m`, F10, right-click, mouse),
  quick/tab/level pickers, select popup, code-editor completion, help overlay, state inspector,
  navigation cursor/hover/click, 72×20 minimum, 60×18 too-small and resize recovery).
* tablepro: connections (default, selection, focus walk, hover, failed connect + Reconnect, help),
  connecting spinner, workbench default at 120×40 / 100×30 / 160×50 / 80×24 (drawer) / 72×20,
  explorer focus/hover, editor insert/nav/editing, completion (auto after FROM/WHERE and Ctrl+Space),
  running + results grid (focus, movement, horizontal scroll), error diagnostics, EXPLAIN tree/raw,
  cancel, table grid (sort by header click, cell editing, pending bar, preview SQL, save dialog with
  token, saved), filter editor, structure view, tabs (Ctrl+T ×3, overflow, Ctrl+G list picker, `+`,
  click), Open Quickly, history (search), Safe Mode picker (+ `safe+` strip), safety dialogs
  (DELETE without WHERE, wrong token, cancelled, DROP, UPDATE with token → executed), `z` explorer
  toggle, help, 60×18 too-small and resize recovery.
* jackin-preview: every scenario paused at stable frames (first-use 0/45/282, returning 0,
  accounts-mixed 0, launch-running 0/60/150/300 + build log, launch-failure 0/200/600 + Esc back,
  capsule-multi 0, outro-last 0/50/88/105/140, hard-cases 0 + manager/accounts/settings/editor
  hard cases); manager (selection, expand, detail focus, hover/click, launch picker, help, File/Go/Help
  menus by keyboard and mouse, lockup menu); prelude steps 1–5 → new-workspace editor → create dialog;
  editor tabs (general/mounts/roles/env/accounts, editing, masked secret, add-env form, leave/save
  dialogs, File menu); settings tabs 1–5, trust toggle, save dialog; accounts (selection, detail focus,
  remove dialog, new-account form, API-key masking while typing and after commit, 1Password picker,
  help, refreshing, File menu); usage overview/limits/help → accounts; Capsule (File/Edit/View/Session/
  Help + lockup menus by keyboard and mouse, tab hover/click, right-click and `Ctrl+B m` tab menus,
  rename dialog, close-tab dialog, prefix hint, new-tab + account pickers, usage overlay, command
  palette via Help menu (+ filtered), zoom, split picker + split, quit dialog, typed echo, scrollback,
  inspect-changes dialog/diff/compact, detach → manager → reconnect); responsive manager + capsule at
  80×24, 100×30, 160×50, 72×20, 60×18 and menus/dialogs/forms at 80×24 and 160×50; too-small + resize
  recovery.

## Pre-existing failures found while capturing (not regressions)

1. **PANIC — jackin-preview `View → Container info` crashes the app.**
   `src/bin/jackin_preview/screens/capsule.rs:1183:65` slices `&i.run_id.replace('-', "")[..8]`;
   the capsule-multi instance's dash-stripped run id is 7 bytes → `end byte index 8 is out of
   bounds for string of length 7`. Reproduced by keyboard (F10 → View → Container info → Enter) and
   by mouse; the pane goes blank (`jackin_capsule_container_info_120x40` is the post-crash frame).
   Evidence: `stderr/602_jackin-preview.log`, `stderr/panic_container_info_backtrace_keyboard.log`,
   `stderr/panic_container_info_backtrace_mouse.log`. Root cause class: fixed-width byte slicing of
   a domain string without a length guard.
2. **`Ctrl+B i` is advertised but not wired.** The View menu lists "Container info    Ctrl+B i", but
   the prefix handler answers "Not a prefix command: i" (`jackin_capsule_ctrl_b_i_120x40`, hint row).
3. **`Ctrl+\` (command palette) cannot be delivered through a legacy-encoding terminal.** tmux sends
   0x1C, which crossterm decodes as `Ctrl+4`; nothing opens (`jackin_capsule_ctrl_backslash_120x40`
   equals the default frame). Reachable via Help → Command palette (captured as
   `jackin_capsule_palette_120x40`). Classified as an environment limitation with product impact:
   the binding is unreachable unless the kitty keyboard protocol is negotiated.
4. **Jackin `F10` reopens the last-opened menu, not File.** After opening Session by mouse and closing
   it, F10 shows Session again (observed while capturing; the plan therefore opens View by mouse).
5. **TablePro has no menu bar.** F10 is a no-op (`tablepro_f10_noop_120x40` equals the previous frame).
   Consistent with README, but the three apps disagree on chrome.
6. `showcase` hover/pressed/disabled-hover states differ from the default frame only in colour —
   compare the `.ansi`/`.png`, the `.txt` is identical except for the hint bar.

## States that could not be captured

* Jackin "Container info" dialog — unreachable (panic, item 1).
* Jackin command palette via its own `Ctrl+\` binding — unreachable in tmux (item 3); captured via menu.
* TablePro application menu — does not exist (item 5).
* Jackin account refresh completion and TablePro/Jackin jobs that need ticks under `--motion paused`
  were captured in their in-progress state only ("Refreshing…", cockpit frames); completed states are
  covered by `--frame` selection where the scenario supports it.

## Environment notes

* PNG rendering uses the Pillow interpreter from `tools/env.sh` (`PY`); it points at a scratch venv
  outside the repo. If absent the harness still writes `.ansi/.txt/.cursor/.html`.
* `find`/`mouse_on` compute columns as character offsets of the plain `.txt` row; all anchors used
  are on rows without wide glyphs, so the click cells match the rendered cells.
