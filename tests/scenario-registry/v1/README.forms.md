# FORMS Scenario Registry

This file describes the FORMS group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.forms.json` holds 7 rows.
  It covers 2 components: Form, Wizard.
  It uses the same stable IDs as `registry.json`.
- `validate-forms.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-forms.py`.
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
- **Form**: validated fields plus actions that submit them.
  Showcase owns a page, jackin owns a modal dialog, tablepro owns a connection form.
  Invalid input blocks submit and focuses the first invalid field.
  Valid input submits exactly once.
- **Wizard**: the jackin five-step prelude chain.
  No widget of that name exists.
  Source, Destination, Edit, Working dir, and Name modals chain in order.
  Titles count the steps, the stepper marks the past, and rewinds keep values.

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
- Form rows cite the page or dialog that owns the submit.
  Child input chrome stays owned by the FIELDS rows.
- Wizard rows cite `prelude.rs`.
  That is the chain that owns the step behavior.
