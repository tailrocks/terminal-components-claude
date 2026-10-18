# Meter

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W39 · Group: Feedback · Phase: P5.

## Purpose and exclusions

Meter presents caller-supplied capacity or consumption information in a compact line or filled-block visual. The caller decides whether a value means used or remaining, its freshness and domain status.

Exclusions:

- Inferring missing capacity as zero.
- Treating capacity as operation completion.
- Fetching quota, currency, reset times or provider data.
- Synthesizing success/green from a full value.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
Meter::new(id: Id, value: Option<Percent>) -> Meter<'a>

Meter::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
Meter::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

Meter::readout(&'a str)
    .visual(MeterVisual)       // Line or Block
    .tone(MeterTone)
    .animation(AnimationSample)
    .patch(StylePatch);
```

Value and tone are controlled props; Meter has no durable state or typed action. There is no update method. Refreshing may compose Spinner using the explicit animation sample.

## Ordinary use

Consumer recipe: [EX-13 — Motion uses time, not another screen](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include Jackin usage and accounts; their baseline output remains protected. The caller supplies the optional value, readout, visual, tone and animation sample; draw does not infer, fetch or mutate status.

## Ownership

| Concern | Owner |
| --- | --- |
| Optional value, readout and domain meaning | Caller |
| Threshold or explicit warning/error classification | Caller via MeterTone |
| Refresh phase and motion policy | Runtime/caller AnimationSample |
| Line/block geometry and painting | Meter |
| Quota/network/provider lifetime | Application |

draw does not infer, fetch or mutate status. measure and draw share rounding, readout placement, suffix width and clipping.

Shared dependencies: [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Two modes are required: line and filled block. Preserve embedded readout placement, clipping, fill rounding and fixed suffix width. Default consumption thresholds are <=59 low, <=84 medium and >84 high. Domain-explicit Warning, Exhausted, Stale, Refreshing, Error and Unknown override generic thresholds.

The compact line recipe is a `━` filled run followed by the remaining `─` track, then a two-cell status/readout suffix (for example, `━━━━────  38%`). The block recipe fills the used share with its level tone, places the value inside in dark bold text, and leaves the remainder one plane up. Both are variants of this one component. Low is 0–59%, medium 60–84%, and high 85–100%; the caller's explicit domain status takes priority over that threshold table. Unknown has no run/bar and shows faint `—`; refreshing uses a spinner in place of the value. Capacity never uses completion green.

Parts are container, fill, rest, value and suffix. Unicode readouts, grapheme width, narrow geometry, semantic tones and capability fallback follow [text](../foundations/text.md), [layout](../foundations/layout.md) and [theme](../foundations/theme.md).

Ordinary example: `Meter::new(id, Some(pct)).visual(MeterVisual::Line)`.

## Behavior

- Meter is display-only. There is no focus, hover or activation behavior.
- Capacity is not completion: no green success fill. Unknown draws no invented track; it uses an explicit unavailable presentation. Error and refresh remain distinct. Readout text is already safe caller data.
- draw cannot advance refresh motion or call services.
- Used versus remaining must be explicitly supplied; the caller readout cannot be reinterpreted or formatted as a domain value.
- Zero/tiny geometry cannot underflow or write outside area.
- Phase changes cannot alter declared layout width.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, readout boundary, narrow then wide |
| Focus/hover | Not applicable; display-only |
| Activation | Not applicable |
| Value | 0, 59, 60, 84, 85, 100 and unknown |
| Tone | normal, warning, exhausted, stale, refreshing, error, unknown |
| Motion | full refresh spinner phases, paused/reduced motion and same-sample repaint |
| Meaning | used versus remaining must be explicitly supplied |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen sources are frozen-at-4a79c0a2 ([progress.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/progress.rs) and [usage.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/usage.rs)). Use [`../reference/capture-plans/meter.json`](../reference/capture-plans/meter.json); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W39-01 | 0, 59, 60, 84, 85, 100 and unknown |
| W39-02 | Line and Block layouts at rounding/readout boundaries |
| W39-03 | Warning/exhausted/stale/refreshing/error/unknown for one value |
| W39-04 | Refresh spinner full cycle; unknown has no bar |
| W39-05 | Remaining versus used meaning is explicit caller data |

Record exact cells/styles, value meaning, tone, sample, cursor/focus/capture (none), dimensions and capability. Capture each semantic tone independently; do not accept threshold inference as a substitute for explicit domain states.

Required negative tests:

- None cannot render as zero or invent a track;
- full capacity cannot become completion green;
- caller readout cannot be reinterpreted or formatted as a domain value;
- draw cannot advance refresh motion or call services;
- line/block measurement and clipping cannot disagree;
- phase changes cannot alter declared layout width;
- zero/tiny geometry cannot underflow or write outside area.

## Rejected use

Forbidden: painting a padded consumption bar from an inferred value instead of using Meter with an explicit optional value and tone.

```rust
// Forbidden: preview paints its own bar and treats missing as zero.
let v = quota.unwrap_or(0);
ui.paint_str(row, 0, "━━━━────  38%");
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls" and "padded strings that simulate columns, selection, fields, menus, or buttons". Use `Meter::new` with the caller value and explicit `MeterTone`.

## Known gaps

- Capture plan W39 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
