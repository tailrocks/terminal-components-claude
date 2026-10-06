# OVERLAYS Scenario Registry

This file describes the OVERLAYS group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.overlays.json` holds 21 rows.
  It covers 10 components: Select, Picker, CommandPalette,
  PickerChain, Completion, Dialog, Menu, ContextMenu,
  MenuBar, HelpOverlay.
  It uses the same stable IDs as `registry.json`.
- `validate-overlays.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-overlays.py`.
  It must print `PASS`.

## Words

- **Reference**: the code before the refactor.
  It lives on branch `visual-baseline`.
  Its frozen tag is `visual-baseline`.
  All source symbols come from the Reference.
- **Candidate**: the code after the refactor.
  It lives on branch `termrock-implementation`.
  The Candidate must show the same output as the Reference.
- **Coverage row**: one test case in the registry.
  It has a stable ID.
  It names the application, the component, and the part.
  It lists input steps and checkpoints.
  It lists visual, state, action, and negative checks.
  It names the source symbols and the snapshot files.
- **Checkpoint**: one named point in a Coverage row.
  The test stops at the Checkpoint.
  The test captures the screen at the Checkpoint.
  The test compares the capture with the snapshot.
- **PTY**: a pseudo-terminal.
  It runs the application as a real terminal runs it.
  It sends keys and reads screen output.
  Tests that need a PTY use the `pty-capture` adapter.
  Tests that draw to a buffer use the `headless-tick` adapter.
- **Select**: a dropdown field.
  Closed it shows the value with `▾`.
  Open it shows an anchored popup list.
  Choosing the same value emits no event.
- **Picker**: a centered searchable modal list.
  The owner ranks and re-supplies the rows.
  Typing filters, Tab scopes, Delete secondaries.
  Backspace on an empty query rewinds the owner.
- **CommandPalette**: a Picker that runs commands.
  Jackin capsule owns one with 20 fixed commands.
  Unavailable rows disable with a reason.
- **PickerChain**: pickers chained into stages.
  The jackin 1Password flow chains Account, Vault, Item, Field.
  Each stage reloads with its own title and crumb.
  Back walks one stage back.
- **Completion**: an anchored non-modal suggestion popup.
  The owner keeps keyboard focus and forwards keys.
  Tab or Enter accepts, Esc dismisses.
  A render never moves the viewport to the cursor.
- **Dialog**: a modal with its own focus scope.
  Confirm, destructive, prompt, and facts shapes exist.
  The page dims behind it.
  `y` and `n` answer text bodies only.
- **Menu**: one open dropdown list.
  The cursor row is a solid highlight fill.
  Shortcuts right-align, danger rows warn, disabled rows skip.
- **ContextMenu**: a free anchored popover menu.
  It opens under a rectangle or at a pointer position.
  It may carry a title row.
  The tab and brand menus are ContextMenus.
- **MenuBar**: a strip of labels that open menus.
  The open label shares the popover plane.
  The brand lockup fires Brand.
  F10 opens the first menu.
- **HelpOverlay**: the shortcut reference.
  Showcase and tablepro use a single-action dialog.
  Jackin uses a scrolled multi-column overlay.
  Esc always closes it.

## Rules

- Do not change a stable ID.
- Do not change `ref_source` without reading the source.
- Candidate symbols stay empty until Stage B knows them.
- Mapped legacy roots keep their old path.
  They point to the new row ID.
- Pending roots have no row yet.
  Keep them in the list.
- Some roots are shared with other groups.
  Each group pins a different widget on the same page.
  The entry note names the overlap.
- CommandPalette and PickerChain rows cite `picker.rs`.
  That is the widget that owns the behavior.
- Menu rows cite the open dropdown.
  MenuBar rows cite the strip and the open wiring.
- Help dialog rows cite `dialog.rs`.
  Only the jackin row cites the `HelpOverlay` struct.

## Candidate port

- This file is the `termrock-implementation` port of the Reference slice.
- Source: `visual-baseline` commit `6007e9fdcbf353fef59177ed8b35d5801f1b0fe2`.
- The 21 rows and the `registry.json` entries are byte-identical to the source.
- The validator resolves Reference paths through git at `ref_commit`.
  The Candidate worktree holds no `src/` or `snapshots/` copies.
