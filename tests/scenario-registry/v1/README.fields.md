# FIELDS Scenario Registry

This file describes the FIELDS group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.fields.json` holds 16 rows.
  It covers 3 components: Field, TextInput, TextArea.
  It uses the same stable IDs as `registry.json`.
- `validate-fields.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-fields.py`.
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
- **Field**: the chrome shared by every text control.
  It is the label row, the gutter bar, the body, and the help/error row.
  TextInput and TextArea both paint it.
  Neither owns it alone.
- **Navigation**: a focused field that is not editing.
  The gutter bar shows focus.
  Enter or F2 starts editing.
  Typing does nothing.
  A paste starts editing.
- **Editing**: the field owns the hardware cursor.
  Enter commits, Esc reverts (TextInput) or commits (TextArea).
  Tab commits and moves focus.

## Rules

- Do not change a stable ID.
- Do not change `ref_source` without reading the source.
- Candidate symbols stay empty until Stage B knows them.
- Mapped legacy roots keep their old path.
  They point to the new row ID.
- Pending roots have no row yet.
  Keep them in the list.
- Disabled is read-only.
  There is no separate read-only flag.
- There is no undo stack.
  Esc reverts to the snapshot taken at begin_edit.
  That snapshot is the only rewind.
