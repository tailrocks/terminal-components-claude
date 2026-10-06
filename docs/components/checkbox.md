# Checkbox

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W03 · Group: Controls · Phase: P3.

## Purpose and exclusions

`Checkbox` presents one controlled boolean choice. The caller owns the label and checked value. Termrock reports the requested next value and never mutates a domain model or retains an uncontrolled copy.

Exclusions:

- No form model, persistence, validation schema, or application-specific side effects.
- No focus state stored in the component.
- No generic button or switch recipe; marker and focus gutter preserve the baseline choice-control geometry.

## Public API

Proposed target. No part of this API is source-checked against current code.

```rust
Checkbox::new(id: Id, label: &'a str, checked: bool) -> Checkbox<'a>

checkbox.update(&mut cx) -> Response<ValueChanged<bool>>
checkbox.draw(&mut ui, area) -> Rect
checkbox.measure(&measure_cx, constraints) -> Size
```

Builders: `disabled(bool)`, `status(ControlStatus)`, `patch(StylePatch)`, and `patch_part(Part, StylePatch)`.

The checked value is borrowed controlled input. Durable component state is none. Runtime owns focus, hover, capture, and press timing. The typed action is `ValueChanged<bool> { value, origin: ActivationOrigin }`.

Update, draw, and measure:

- `update` handles Enter, Space, and completed pointer clicks. It requests the inverse controlled value once per eligible gesture. If the caller declines to apply the action, the next draw still shows the caller's value.
- `draw` paints the focus gutter, marker, and label inside the clipped row. Focus never changes `checked`. No semantic state changes occur during draw.
- `measure` accounts for the gutter, compact/full marker, label, and clipping by display cells.

## Ordinary use

Consumer recipe: [EX-03 — Controlled choices](../api/consumer-recipes.md) (`proposed_target`).

Showcase settings and choice pages provide baseline consumers.

Ordinary use passes a stable `Id`, borrowed `label`, and controlled `checked`. The caller applies the requested inverse value from the typed `ValueChanged<bool>` action. If the caller declines, the next draw still shows the caller's value.

## Ownership

The stable `Id` owns focus/hit/capture records. Adjacent controls receive disjoint clipped rectangles. Removing or disabling a checkbox cancels capture and hover. Reordering controls preserves identity and cannot transfer an action. No index-based state is retained.

Shared dependencies:

- [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md) — stable identity and typed value-change origin.
- [`runtime`](../foundations/runtime.md) — focus, hover, hit testing, pointer capture, and timing.
- [`layout`](../foundations/layout.md) — clipped row and display-cell measurement.
- [`theme`](../foundations/theme.md) — choice surfaces, markers, and capability mapping.
- [`author`](../foundations/author.md) — part patches without parallel renderers.
- [`conformance`](../foundations/conformance.md) — source/output proof and negative tests.

## Customization

Parts are `container`, `gutter`, `marker`, and `label`. Preserve baseline marker glyphs and spacing: full rows use `[✓]`/`[ ]`; narrow rows use compact `✓`/`□` as applicable. The focus gutter occupies a different cell from the checked marker. Disabled colors, hover lift, semantic surfaces, and clipping come from [`theme`](../foundations/theme.md). Part patches cannot replace geometry, focus/capture, or the whole surface.

Ordinary example: `Checkbox::new(id, "Notify", true).disabled(false)`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Enter/Space requests the inverse value once. Focus alone never changes it. |
| Pointer | Marker, label, and last valid clipped cell are eligible. Down plus release inside requests one change; release outside, removed, or adjacent cells do not. |
| Focus/hover | Runtime owns focus and hover. Keyboard suppresses stale hover; pointer motion restores it. Focus and checked marker are independent. |
| Disabled/read-only | Disabled checked and unchecked controls ignore activation, clear hover/press state, and emit no value change. Read-only callers pass disabled/eligibility policy; no hidden mutation occurs. |
| Editing | Not applicable. |
| Resize/Unicode | Clip label and marker to allocation; measure combining marks and wide glyphs by cells. |
| Capability/color | Semantic accent, muted, disabled, and surface styles must resolve in all supported color modes. |
| Motion | Use shared activation feedback only where the baseline applies; no private animation. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | pointer down, held press, release inside, release outside, removed target, disabled target, keyboard activation, feedback active/expired when applicable |
| Value | checked/unchecked, controlled caller accepts action, controlled caller rejects action |

Run applicable cases at all five capture dimensions and truecolor, 256, 16, none, and nocolor capabilities.

## Verification

The oracle is frozen at [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current source is [`src/widgets/choice.rs`](../../src/widgets/choice.rs), legacy `Checkbox`, source blob `9405acf87c152c0766ae9bbcd71c01c9d5db4e80`.

The W03 capture contract is [`capture-plans/checkbox.json`](../reference/capture-plans/checkbox.json). It is planned and has no approved artifacts. Bind source output before candidate testing.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W03-01 | Checked and unchecked crossed with independent focus and hover. |
| W03-02 | Disabled checked and disabled unchecked ignore all activation. |
| W03-03 | Click marker, label, last valid cell, and immediately outside. |
| W03-04 | Caller rejects the change; checkbox does not drift into an uncontrolled value. |

Record exact cells, cursor, focus/capture/layer owner, action count/target, and controlled value before/after.

Negative tests:

- Focus, hover, or draw cannot mutate the checked value.
- A caller rejection leaves the next render equal to the caller input.
- Disabled controls cannot capture, press, or emit an action.
- Adjacent rows never receive a neighboring checkbox's click.
- Unicode clipping cannot write outside the rectangle.
- Candidate code cannot accept its own output as expected output.

Accept after the external API example, source-state tests, exact applicable snapshots, boundary hit tests, and independent review pass.

## Rejected use

Forbidden: painting `[x]` text and flipping an app boolean on click instead of using Checkbox.

```rust
// Forbidden: preview owns choice rendering and the toggle.
ui.paint_str(row, 0, "[x] Notify");
if clicked { notify = !notify; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate fields. Use `Checkbox::new` and apply the typed `ValueChanged<bool>` action.

## Known gaps

- Capture plan W03 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
