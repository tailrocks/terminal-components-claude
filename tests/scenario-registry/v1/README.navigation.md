# NAVIGATION Scenario Registry

This file describes the NAVIGATION group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.navigation.json` holds 19 rows.
  It covers 8 components: ChipBar, List, FilterList,
  NavList, Tree, Steps, Tabs, PropsList.
  It uses the same stable IDs as `registry.json`.
- `validate-navigation.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-navigation.py`.
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
- **ChipBar**: a row of removable, toggleable chips.
  It is one focus stop.
  Left and Right move the cursor.
  Enter activates, Space toggles, Delete removes.
- **List**: a scrollable single or multi select box.
  Single mode marks the choice with `›`.
  Multi mode marks each checked row with `✓`.
- **FilterList**: a searchable modal list.
  It is the `Picker` widget used as a list.
  Typing filters the rows.
  The owner ranks and re-supplies them.
- **NavList**: a sectioned navigation list.
  `›` is the current item.
  `▎` is the keyboard cursor.
  The showcase shell and the sidebars page each own one.
- **Tree**: a folding hierarchy of rows.
  `▸` is folded, `▾` is open.
  The cursor follows path identity, not row index.
- **Steps**: a stage rail with a lifecycle per row.
  No app makes it selectable.
  It shows progress, it takes no input.
- **Tabs**: a horizontal tab strip.
  The active tab carries the only accent rule.
  The strip scrolls instead of shrinking labels.
- **PropsList**: interactive label and value facts.
  It has a row cursor.
  `y` copies a copyable row.
  Enter activates it.

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
- FilterList rows cite `picker.rs`.
  That is the widget that owns the behavior.
