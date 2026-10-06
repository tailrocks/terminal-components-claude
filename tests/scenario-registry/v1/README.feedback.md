# FEEDBACK Scenario Registry

This file describes the FEEDBACK group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.feedback.json` holds 13 rows.
  It covers 6 components: ProgressBar, Spinner, Meter,
  StatusBar, HintBar, KeyHint.
  It uses the same stable IDs as `registry.json`.
- `validate-feedback.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-feedback.py`.
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
- **ProgressBar**: a determinate bar with a label, a track, a percent, and a marker.
  Done ends ` ✓`, Error ends ` !`, Paused ends ` ‖`.
  Green is reserved for Done.
  A running bar is white.
- **Spinner**: the shared tick clock made visible.
  Ten braille frames cycle by modulo.
  The compact row pairs one frame with a label.
  The indeterminate bar sweeps a short accent segment.
- **Meter**: a capacity readout that is never green.
  Low is white, medium is warning, high is error.
  Domain tones add warning, exhausted, stale, refreshing, error, and unknown.
  Line draws a run, Block fills the used share.
- **StatusBar**: one full-width row in three groups.
  Groups split by spacing, never by glyphs.
  Narrow rows drop center first, then right, then left.
  The strongest left item always survives, truncated.
- **HintBar**: the one key-hint surface a shell owns.
  Layers stack by context and the topmost present layer wins.
  A modal never paints its own hint row.
  The footer never moves.
- **KeyHint**: the pairs and the status the HintBar draws.
  Narrow rows drop hints from the right and mark the cut with `…`.
  Error and warning statuses carry `!` and `▲` so mono keeps the weight.

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
- ProgressBar rows cite `render_bar`.
  Spinner rows cite `spinner_frame`, `render_spinner`, and `render_indeterminate`.
- Meter rows cite the `Meter` widget and its tone table.
  StatusBar, HintBar, and KeyHint rows cite the like-named widget.
