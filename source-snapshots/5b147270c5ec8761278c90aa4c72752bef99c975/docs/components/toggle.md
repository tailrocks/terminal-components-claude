# Toggle

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W04 · Group: Controls · Phase: P3.

## Purpose and exclusions

`Toggle` is a controlled on/off choice with the baseline switch glyph and label treatment. It shares the choice reducer and runtime interaction rules with Checkbox, but keeps its own visual recipe. The caller owns wording and boolean value; Termrock reports the requested next value.

Exclusions:

- No graphical switch redesign, persistence, animation loop, or product settings model.
- No second event router or duplicate focus/capture implementation.
- No stateful toggle value inside the component.

## Public API

Proposed target. No part of this API is source-checked against current code.

```rust
Toggle::new(id: Id, label: &'a str, on: bool) -> Toggle<'a>

toggle.update(&mut cx) -> Response<ValueChanged<bool>>
toggle.draw(&mut ui, area) -> Rect
toggle.measure(&measure_cx, constraints) -> Size
```

Builders: `disabled(bool)`, `patch(StylePatch)`, and `patch_part(Part, StylePatch)`.

`on` is controlled borrowed input. Durable state is none. Runtime owns focus, hover, capture, and shared timing. The action is `ValueChanged<bool> { value, origin: ActivationOrigin }`.

Update, draw, and measure:

- `update` uses the Checkbox one-action-per-gesture reducer: Enter/Space or completed pointer click requests a single next value when enabled.
- `draw` paints the baseline switch glyph and label, preserving independent focus gutter and on/off marker. It does not mutate `on` or start animation.
- `measure` accounts for switch marker, label, spacing, clipping, and Unicode display width.

## Ordinary use

Consumer recipe: [EX-03 — Controlled choices](../api/consumer-recipes.md) (`proposed_target`).

Showcase settings uses the baseline on/off visuals and interactions.

Ordinary use passes a stable `Id`, borrowed `label`, and controlled `on`. The caller applies the requested inverse value from the typed `ValueChanged<bool>` action. A rejected action cannot cause visual drift.

## Ownership

Stable `Id` owns runtime records. Adjacent toggles have disjoint clipped hit areas. Reordering preserves identity; removal or disablement cancels pointer capture and hover. The action is applied only by the caller, so a rejected action cannot cause visual drift.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md) — stable IDs and typed value actions.
- [`runtime`](../foundations/runtime.md) — shared focus, hit, capture, hover, and time.
- [`layout`](../foundations/layout.md) — one-row measurement and safe clipping.
- [`theme`](../foundations/theme.md) — semantic switch styles and capability fallback.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — constrained customization and oracle proof.

## Customization

Parts are `container`, `gutter`, `marker`, and `label`. Preserve the baseline `○──`/`──●` treatment (and compact marker at narrow widths), spacing, label and optional state text, accent/on color, muted/off color, disabled style, hover lift, and clipping. This is not a generic graphical switch. [`theme`](../foundations/theme.md) owns semantic style resolution; part patches cannot replace whole-surface or runtime ownership.

Ordinary example: `Toggle::new(id, "Wi-Fi", true).disabled(false)`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Enter/Space request one inverse value. Focus alone never changes on/off. |
| Pointer | Only the clipped control/label rectangle is eligible. Down plus inside release requests one change; outside/removed release does not. |
| Focus/hover | Runtime-owned; keyboard suppresses stale hover and pointer motion restores it. Focus and on/off state are independent. |
| Disabled/read-only | Disabled on/off states ignore keyboard and pointer activation and clear pressed/hover feedback. Read-only is caller eligibility, not hidden state. |
| Editing | Not applicable. |
| Resize/Unicode | Preserve marker and label clipping at minimum width; use display-cell width for combining/wide glyphs. |
| Capability/color | Resolve accent, muted, disabled, and surface styles for truecolor, 256, 16, none, and nocolor. |
| Motion | No private animation or persistence; shared activation timing only where baseline requires. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held press, release inside, release outside, removed target, disabled target, keyboard activation, feedback active/expired where applicable |
| Value | on/off crossed with normal, focused, hovered, disabled |

Capture each applicable case at 72×20, 80×24, 100×30, 120×40, and 160×50 under truecolor, 256, 16, none, and nocolor. Record action count, target, cursor, focus/capture owner, and values.

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current legacy implementation is `Toggle` in [`src/widgets/choice.rs`](../../src/widgets/choice.rs), source blob `9405acf87c152c0766ae9bbcd71c01c9d5db4e80`.

Use [`capture-plans/toggle.json`](../reference/capture-plans/toggle.json), W04. It is planned and uncaptured; bind ExistingOracle/ExtractedOracle before candidate comparisons.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W04-01 | On/off crossed with normal, focused, hovered, and disabled. |
| W04-02 | Enter/Space and mouse produce the same next controlled value. |
| W04-03 | Adjacent toggles do not overlap hit regions. |
| W04-04 | Minimum width and clipped marker/label behavior. |

Negative tests:

- `draw` cannot mutate controlled `on`; caller rejection leaves it unchanged.
- Disabled toggles emit no action, capture, or pressed state.
- Adjacent controls cannot cross-target.
- Minimum-width and Unicode writes stay within the allocation.
- No separate toggle event loop or animation owner is introduced.

Accept only after API, state, exact visual, pointer, Unicode, and negative tests plus independent review pass.

## Rejected use

Forbidden: painting a switch glyph and flipping an app boolean on click instead of using Toggle.

```rust
// Forbidden: preview owns switch rendering and the flip.
ui.paint_str(row, 0, "──● Wi-Fi");
if clicked { wifi = !wifi; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls in preview drawing paths. Use `Toggle::new` and apply the typed `ValueChanged<bool>` action.

## Known gaps

- Capture plan W04 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
