# W23 · Menu

**Group:** Overlays · **Phase:** P4 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow typed command/submenu/separator entries and live action eligibility. Never execute effects.

Legacy family mapping: `C32`  
Proposed implementation home: `crates/termrock/src/components/menu.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/menu.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs) — observed Git blob `87934aaabeefb7dd37aa8ea2d093f005f643abf4`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: menu.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/menu.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Menu::new(id: Id, items: &'a [MenuItem<'a>], revision: Revision) -> Menu<'a>

update(&self, cx: &mut Cx<'_>, state: &mut MenuState) -> Response<MenuAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &MenuState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
patch(StylePatch)
patch_part(Part, StylePatch)
```

**Durable state:** MenuState: highlighted keyed item, open submenu path and scroll. Shared by ContextMenu and MenuBar surfaces.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** MenuAction::{Invoke { action: ActionKey, origin }, Dismissed}

## Visual parts and composition

Advertised styled parts: `container`, `row`, `marker`, `label`, `shortcut`, `submenu`, `separator`, `status`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md), [key-hint](./key-hint.md)

## Behavioral contract

1. Arrow keys traverse actionable rows, open/close submenus and preserve the active path. Disabled rows are barriers, not click-through gaps.
2. Shortcut display derives from shared action metadata. A submenu may flip at the right edge without losing hover ownership between parent and child.
3. Dismiss outer menu only according to layer policy; menu key handling cannot consume text intended for a modal editor above it.

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
| W23-01 | disabled and separator entries between actionable rows |
| W23-02 | three-level submenu at right/bottom edge |
| W23-03 | move pointer from parent to submenu and back without accidental close |
| W23-04 | invoke remapped shortcut and compare ActionKey |
| W23-05 | outside click consumed once; nested Escape closes one level |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/menu.json](../capture-plans/menu.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
