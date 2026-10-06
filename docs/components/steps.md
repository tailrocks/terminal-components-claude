# Steps

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-navigation.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W16 · Group: Feedback · Phase: P5.

## Purpose and exclusions

Canonical Termrock contract for W16. `Steps` is a lifecycle/status rail with an optional inspection cursor. It never runs work.

The caller supplies ordered stable step keys, lifecycle status, labels, detail/meta and the current status snapshot. The component presents queued/running/skipped/blocked/done/failed states and may provide navigable inspection. It has no dependency graph, scheduler, retry policy, launch engine or lifecycle authority.

## Public API

```rust
Steps::new(id: Id, steps: &'a [StepItem<'a>], revision: Revision) -> Steps<'a>

Steps::mode(StepsMode) // Display or Navigable
    .animation(AnimationSample)
    .row(&StepRowPainter)
    .patch(StylePatch);

Steps::update(&self, cx: &mut Cx<'_>, state: &mut StepsState)
    -> Response<StepsAction>;
Steps::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &StepsState) -> Rect;
Steps::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`StepsState` exists only in `Navigable` mode and owns cursor key plus shared scroll. Display mode has no focus stop or interaction state. The only typed action is `StepsAction::Activate { key: ItemKey, origin: ActivationOrigin }`; it asks the caller to inspect a step without changing its lifecycle.

## Ordinary use

Jackin Preview cockpit and Showcase taskrunner are reference consumers. Holla has its own product plan model; any shared visual use remains a protected application fixture, not a product roadmap for this component.

The caller supplies borrowed steps, a source revision, and the status snapshot, then applies the typed `StepsAction` inspection requests.

## Ownership

- Caller owns status, labels, details, ordering and revision. The component may derive read-only frontier/count observations but cannot write lifecycle state.
- Stable step keys survive source replacement/reorder where present. A removed current key is stale and cannot activate a new row at its old index.
- Display mode is purely presentational. Navigable mode owns only cursor/scroll and uses the shared runtime focus/hit/capture system.
- `update` changes only navigable state and returns inspection actions. `draw` never advances status, spinner phase, cursor or scroll. `measure` is pure.
- `AnimationSample` is an explicit caller-provided phase/time sample. Repaint count and polling never advance animation.

Shared dependencies: [`scroll-region`](./scroll-region.md#scroll-and-capture-behavior), [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `row`, `gutter`, `marker`, `label`, `detail`, `status`, `scrollbar` and `fade`. Theme/patch precedence follows [`theme`](../foundations/theme.md); parts cannot replace geometry or runtime ownership.

Preserve the baseline numbered rail, indentation, status glyphs and tone:
queued uses muted `·`; running uses the sampled accent spinner; done uses a
secondary `✓`; skipped uses faint `–`; failed uses error `✗` and a bold
label; blocked uses faint `·` and a faint label. The frontier is the running
or failed step; a failure blocks later steps. Details/durations are
right-aligned and clipped by display-cell width. The running spinner is the
only live-activity use of green in this rail. Display-only progress has no
focus gutter; navigable mode adds the existing focus treatment without
changing status colors.

## Behavior

- Display mode accepts no focus, hover, pointer activation or keyboard action. Its rows remain readable and do not create focus stops.
- Navigable mode uses the baseline Up/Down, `j`/`k`, Home/`g` and End/`G` navigation plus pointer row selection when offered. Enter/click emits `Activate` for the stable key and does not alter the caller status.
- Wheel and scrollbar thumb drag scroll the rail under the pointer with shared capture and boundary behavior. They do not change lifecycle or transfer focus in display mode.
- Hover/focus are meaningful only in navigable mode. Keyboard input suppresses stale hover until pointer movement. Removed/disabled rows cannot activate.
- Resize, long labels/details and narrow widths preserve numbering, status glyph placement and the protected cursor row. Motion supports every supplied phase, pause and reduced-motion sample.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode labels/details, narrow-to-wide resize |
| Focus/hover | display: not applicable; navigable: unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | navigable only: pointer down/held/release inside/outside, removed/disabled target, keyboard activation, applicable feedback; display has none |
| Source/status | empty, ready, cursor separate from caller status, insert/remove/reorder, stale target, each queued/running/skipped/blocked/done/failed lifecycle |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade and protected row |
| Motion | every unique spinner/animation phase, wrap, before/at/after boundary, paused, reduced motion and same-time repaint |

Only status variants supplied by the caller are captured. Shared dimensions/capabilities are in [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen source is [`src/widgets/steps.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/steps.rs) (legacy family C26), currently exposing `Step`, `StepRail`, `StepState`, `WidgetId` and `Outcome`.

Use [`../reference/capture-plans/steps.json`](../reference/capture-plans/steps.json). Expected output is bound before candidate testing through [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md). Compare exact cells, continuation cells, colors/modifiers, cursor visibility/position, focus/capture owner, key, status, animation phase and action count/target. Invalid oracle setup blocks the case.

| Case | Required observation |
|---|---|
| W16-01 | Every lifecycle state, including skipped and blocked, uses its baseline glyph/tone. |
| W16-02 | Running spinner renders all unique sampled phases. |
| W16-03 | Navigating to a failed step emits inspection without changing caller lifecycle. |
| W16-04 | Partial details, long labels and overflow retain baseline clipping. |
| W16-05 | Live source replacement preserves the current stable key when it survives. |

Negative tests must prove: display mode never registers a focus stop; lifecycle status cannot change during update/draw; spinner phase cannot advance from repaint count; removed/disabled keys cannot activate; duplicate keys are rejected; draw is semantically pure; wheel/scrollbar cannot change status; long Unicode text cannot corrupt cells; and part patches cannot overwrite required status/gutter cells. Steps does not embed a scheduler or retry operation.

## Rejected use

Forbidden: painting status glyph rows and tracking the lifecycle in preview instead of using Steps.

```rust
// Forbidden: preview paints its own step rail.
ui.paint_str(row, 0, "✓ fetch  ◌ build");
if done { status = "done"; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate status. Use `Steps::new` with stable step keys and handle the typed `StepsAction`.

## Known gaps

- Capture plan W16 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
