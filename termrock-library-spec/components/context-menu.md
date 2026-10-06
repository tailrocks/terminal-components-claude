# W24 · ContextMenu

**Group:** Overlays · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller supplies the stable target and actions. Captured target is not recomputed from current visual row index on invocation.

Legacy family mapping: `C32`  
Proposed implementation home: `crates/termrock/src/components/menu.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/menu.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs) — observed Git blob `87934aaabeefb7dd37aa8ea2d093f005f643abf4`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: menu.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/menu.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/chrome](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/chrome)
- [snapshots/jackin/capsule](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/capsule)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
ContextMenu::new(id: Id, target: ItemKey, items: &'a [MenuItem<'a>], revision: Revision) -> ContextMenu<'a>

update(&self, cx: &mut Cx<'_>, state: &mut MenuState) -> Response<MenuAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
anchor(Anchor)
patch(StylePatch)
```

**Durable state:** MenuState reused; no separate item interaction engine.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** MenuAction::{Invoke { action: ActionKey, origin }, Dismissed}

## Visual parts and composition

Advertised styled parts: `container`, `row`, `marker`, `label`, `shortcut`, `submenu`, `separator`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [menu](./menu.md)

## Behavioral contract

1. Configure Menu with a pointer/keyboard anchor on a runtime layer. Support right click and the caller-bound keyboard context action.
2. Opening another menu replaces the correct layer, not a global shared popup ID.
3. If target is removed or no longer eligible, dismiss or disable its commands; never target the item that inherited its old index.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |
| layer | `closed`, `open`, `nested`, `escape`, `outside`, `owner_removed`, `resize_reanchor`, `focus_restore`, `modal_first_paste` | runtime-owned overlay |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W24-01 | open on a nonselected row without retargeting command to selected row |
| W24-02 | remove/reorder target while menu open |
| W24-03 | keyboard anchor and pointer anchor near terminal borders |
| W24-04 | context menu beneath a modal cannot activate |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/context-menu.json](../capture-plans/context-menu.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
