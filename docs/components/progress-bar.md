# ProgressBar

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W37 · Group: Feedback · Phase: P5.

## Purpose and exclusions

ProgressBar paints a deterministic progress sample. The caller owns operation progress, lifecycle, cancellation and timing. The component never starts a job or timer.

Exclusions:

- Starting, polling, pausing or cancelling an operation.
- Emitting cancel/retry actions; those are separate Buttons.
- Advancing animation from draw count or wall-clock access.
- Replacing the baseline bar with a new visual style.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
ProgressBar::new(id: Id, value: ProgressValue) -> ProgressBar<'a>

ProgressBar::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
ProgressBar::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

ProgressBar::label(&'a str)
    .status(ProgressStatus)       // Active, Done, Error, Paused
    .animation(AnimationSample)
    .patch(StylePatch);
```

ProgressValue accepts a finite determinate fraction or an explicit indeterminate sample. Nonfinite input is rejected before rendering. No durable component state or typed action is required. There is no update method.

The frozen source's render_bar helper is evidence for baseline geometry and tones. Its current Rust names are not future compatibility requirements.

## Ordinary use

Consumer recipe: [EX-13 — Motion uses time, not another screen](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include Showcase progress and the Jackin cockpit; their baseline output remains protected. The caller supplies the value, status, label and animation sample; draw consumes the supplied sample without advancing it.

## Ownership

| Concern | Owner |
| --- | --- |
| Value, status, label and lifetime | Caller, borrowed/value props |
| Animation phase and motion policy | Runtime/caller AnimationSample |
| Geometry | Shared measure and draw calculation |
| Theme tones and part patches | Termrock theme |
| Cancel/retry/pause | Separate Button/application action |

draw consumes the supplied sample without advancing it. measure and draw share label, track, suffix and clipping rules.

Shared dependencies: [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`runtime`](../foundations/runtime.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, label, fill, rest, percentage and suffix. Determinate fill uses round(track width × ratio), a rounded integer percentage and a fixed two-cell suffix column. Preserve suffixes: blank for active, check for done, exclamation for error and double bar for paused.

The baseline label is omitted unless available width exceeds label width plus eight cells. A remaining track shorter than six cells renders percentage only. Active bars use secondary text tone; done is success, error is error and pause is muted. Green is reserved for completion.

Unicode track glyphs, grapheme-safe labels, clipping, color fallback and no-color behavior follow [text](../foundations/text.md) and [theme](../foundations/theme.md).

Ordinary example: `ProgressBar::new(id, ProgressValue::ratio(0.5)).status(ProgressStatus::Active)`.

## Behavior

- ProgressBar is display-only. There is no focus, hover, press or activation behavior; adjacent controls own actions.
- draw consumes the supplied sample without advancing it and without mutating caller state.
- Indeterminate rendering is characterized over the complete source phase cycle for each tested width. It must not be reduced to one arbitrary still.
- Pause, resume and reduced motion do not advance rendering from draw count.
- Narrow areas cannot underflow or write outside area.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, label boundary, track boundary, narrow then wide |
| Focus/hover | Not applicable; display-only |
| Activation | Not applicable; adjacent controls own actions |
| Value/status | 0, fractional rounding boundaries, 50, 100, active, done, error, paused, indeterminate |
| Motion | all unique phases, phase wrap, before/at/after boundary, paused and reduced motion, same-time repaint |
| Color/capability | semantic tone fallback across truecolor, 256, 16, none and nocolor |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen widget source is frozen-at-4a79c0a2 ([progress.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/progress.rs)). Use [`../reference/capture-plans/progress-bar.json`](../reference/capture-plans/progress-bar.json), [motion rules](../foundations/runtime.md), [theme](../foundations/theme.md) and [visual parity proof](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W37-01 | 0%, fractional rounding boundaries, 50%, 100% and completed value |
| W37-02 | Active/done/error/paused with aligned suffix column |
| W37-03 | Label width+8 and width+9; track length 5 versus 6 |
| W37-04 | Indeterminate full phase cycle at narrow, normal and wide sizes |
| W37-05 | Pause/resume/reduced motion do not advance from draw count |

Record exact cells/styles, dimensions, capability, cursor (none), focus and capture (none), animation sample and status. A missing or invalid oracle capture is blocked evidence, never a passing result.

Required negative tests:

- NaN and infinity are rejected before any cell is painted;
- draw cannot advance animation or mutate caller state;
- measured width/height and painted track boundaries agree;
- narrow areas cannot underflow or write outside area;
- status tones cannot recolor unrelated surfaces;
- a progress bar cannot emit operation controls or domain actions.

## Rejected use

Forbidden: painting a padded fill string from draw count instead of using ProgressBar with a caller sample.

```rust
// Forbidden: preview paints its own bar and advances from repaints.
ui.paint_str(row, 0, "█████░░░░░  50%");
frame_count += 1;
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls" and "padded strings that simulate columns, selection, fields, menus, or buttons". Use `ProgressBar::new` with a caller-supplied value and animation sample.

## Known gaps

- Capture plan W37 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
