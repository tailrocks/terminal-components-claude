# Spinner

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W38 · Group: Feedback · Phase: P5.

## Purpose and exclusions

Spinner paints one deterministic phase supplied by the caller/runtime. It does not read a wall clock, create a task or decide cadence.

Exclusions:

- Owning time, timers or an async job.
- An implicit universal cadence for every application.
- Focus, hover, press or click behavior.
- Replacing an application status or error policy.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
Spinner::new(id: Id, sample: AnimationSample) -> Spinner

Spinner::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
Spinner::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

Spinner::status(SpinnerStatus)
    .patch(StylePatch);
```

There is no update method, durable SpinnerState or typed action. The sample determines the phase and explicit status determines stopped, paused or reduced-motion rendering.

## Ordinary use

Consumer recipe: [EX-13 — Motion uses time, not another screen](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include Showcase progress and the Jackin cockpit; their baseline output remains protected. The caller supplies the sample and status; draw is a pure projection of the sample. Rendering twice with the same sample produces the same cell. measure uses the same glyph width as draw.

## Ownership

| Concern | Owner |
| --- | --- |
| Epoch, tick, cadence and motion policy | Runtime/caller |
| Status and semantic tone | Caller |
| Phase glyph selection | Spinner |
| Focus, hit and capture | Parent/runtime; Spinner owns none |
| Surrounding padding and separators | Parent component |

Shared dependencies: [`runtime`](../foundations/runtime.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`layout`](../foundations/layout.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

The baseline has exactly ten frames, in order:

    ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏

Frame selection is modulo ten. Preserve the glyphs, width, tone and clipping. The phase schedule comes from the owner; it is not hard-coded as one wall-clock interval. When embedded in StatusBar or Meter, the frame occupies its declared cells and cannot change layout width.

Unicode width and terminal capability fallback follow [text](../foundations/text.md) and [theme](../foundations/theme.md).

Ordinary example: `Spinner::new(id, sample).status(SpinnerStatus::Active)`.

## Behavior

- Spinner is display-only. There is no independent focus, hover, pressed, active/clicked state.
- draw is a pure projection of the sample. Equal samples produce equal output. Timing is an input to proof, never inferred from draw count.
- Frames cannot be skipped or reordered.
- A spinner cannot register focus/hit/capture.
- A narrow area cannot panic or paint outside bounds.
- Embedding cannot change measured width across phases.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, narrow then wide |
| Focus/hover | Not applicable; no independent focus stop |
| Activation | Not applicable |
| Motion | all ten phases, wrap 9→0, before/at/after owner boundary, paused, reduced, stopped, same-time repaint |
| Composition | embedded in StatusBar and Meter with exact gaps and stable width |
| Color/capability | status/theme fallback in every capture capability |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen widget source is frozen-at-4a79c0a2 ([progress.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/progress.rs)). Use [`../reference/capture-plans/spinner.json`](../reference/capture-plans/spinner.json), the [runtime/time contract](../foundations/runtime.md) and [visual parity proof](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W38-01 | All ten phases plus wrap from 9 to 0 |
| W38-02 | Immediately before, at and after each owner cadence boundary |
| W38-03 | Same time/sample rendered twice gives identical cells |
| W38-04 | Spinner inside StatusBar and Meter preserves separator gaps |
| W38-05 | Paused, reduced and stopped semantics |

Record exact cells/styles, sample/time input, cursor (none), focus/capture owners (none), dimensions and capability.

Required negative tests:

- draw cannot call wall-clock APIs or mutate the sample;
- equal samples produce equal output;
- frames cannot be skipped or reordered;
- a spinner cannot register focus/hit/capture;
- a narrow area cannot panic or paint outside bounds;
- embedding cannot change measured width across phases.

## Rejected use

Forbidden: painting a hardcoded spinner glyph from wall-clock time inside preview drawing instead of using Spinner with a supplied sample.

```rust
// Forbidden: preview reads the clock and paints its own frame.
let frame = now_ms() / 80 % 10;
ui.paint_str(row, 0, FRAMES[frame]);
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls". Use `Spinner::new` with a caller-supplied `AnimationSample`.

## Known gaps

- Capture plan W38 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
