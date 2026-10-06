# W02 · Button

**Group:** Controls · **Phase:** P2 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller owns label, checked value and eligibility. Runtime owns focus, pointer capture and the 140 ms activation flash.

Legacy family mapping: `C11`  
Proposed implementation home: `crates/termrock/src/components/button.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/button.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/button.rs) — observed Git blob `d2c52e75084ff783a9b3bf5d95f736870f480051`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: button.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/button.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/audit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/audit)
- [snapshots/showcase/pages/chrome](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/chrome)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Button::new(id: Id, label: &'a str) -> Button<'a>

update(&self, cx: &mut Cx<'_>) -> Response<Activated>
draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
variant(ButtonVariant) // Secondary default; Primary, Subtle, Danger, Toggle, Quiet, Ghost
disabled(bool)
status(ControlStatus)
checked(Option<bool>)
icon(Option<Glyph>)
autofocus(bool)
patch(StylePatch)
patch_part(Part, StylePatch)
slot(Part, &SlotPainter)
```

**Durable state:** None. Do not add ButtonState just to duplicate focus/hover/press.



**Typed actions:** Activated { origin: ActivationOrigin }

## Visual parts and composition

Advertised styled parts: `container`, `gutter`, `icon`, `marker`, `label`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: Shared foundations only.

## Behavioral contract

1. One-row control; measure gutter, optional icon/marker, label and padding from the baseline, not a generic boxed-button recipe.
2. Enter, Space and completed pointer click emit one activation. Pointer down alone and release outside emit none. Disabled or blocked/busy activation emits none.
3. Checked, focused, hovered, held-pressed and activation-feedback are independent. Focus+hover must show both the gutter and lift.
4. Autofocus is one-shot per mounted identity, requested during update. Rendering, reenabling, or repainting a covered button must not steal modal focus.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W02-01 | all variants at normal, focus, hover, focus+hover and disabled |
| W02-02 | pointer down inside; release outside; repeat with target removed or disabled |
| W02-03 | successful keyboard and pointer activation at 0, 139, 140 and 141 ms after feedback start |
| W02-04 | checked but unfocused; unchecked but focused; busy activation rejection |
| W02-05 | label containing combining characters, CJK and an empty label |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/button.json](../capture-plans/button.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
