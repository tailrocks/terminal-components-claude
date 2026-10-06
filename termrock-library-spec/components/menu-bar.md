# W25 · MenuBar

**Group:** Chrome · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Application supplies top-level menu labels and right-hand context. No fixed File/Go/Session vocabulary in the library.

Legacy family mapping: `C32`  
Proposed implementation home: `crates/termrock/src/components/menu.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/menu.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs) — observed Git blob `87934aaabeefb7dd37aa8ea2d093f005f643abf4`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: menu.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/menu.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/chrome](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/chrome)
- [snapshots/jackin](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
MenuBar::new(id: Id, menus: &'a [TopMenu<'a>], revision: Revision) -> MenuBar<'a>

update(&self, cx: &mut Cx<'_>, state: &mut MenuBarState) -> Response<MenuAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuBarState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
leading(Option<Brand<'a>>)
trailing(StyledText<'a>)
patch(StylePatch)
```

**Durable state:** MenuBarState: selected top menu and a MenuState for the open menu chain.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** MenuAction::{Invoke { action: ActionKey, origin }, Dismissed}

## Visual parts and composition

Advertised styled parts: `container`, `brand`, `title`, `menu-label`, `dropdown`, `metadata`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [menu](./menu.md), [brand](./brand.md)

## Behavioral contract

1. Preserve a one-row bar, label padding and dropdown anchors. F10/registered accelerator enters menu navigation; click/hover switching follows reference behavior.
2. MenuBar delegates dropdown selection to Menu; it does not implement another command dispatcher.
3. Right-hand context truncates according to the measured layout without making left-hand menu hitboxes disagree with paint.

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
| W25-01 | F10 open; left/right switch titles; Escape returns owner focus |
| W25-02 | pointer open then switch to another top menu |
| W25-03 | narrow bar with trailing metadata and clickable Brand |
| W25-04 | dynamic menu action eligibility after open |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/menu-bar.json](../capture-plans/menu-bar.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
