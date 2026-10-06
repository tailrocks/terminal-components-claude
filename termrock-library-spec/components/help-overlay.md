# W26 · HelpOverlay

**Group:** Overlays · **Phase:** P5 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow explanatory sections and effective bindings; no second static keybinding catalog.

Legacy family mapping: `C33`  
Proposed implementation home: `crates/termrock/src/components/help.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/bin/jackin_preview/screens/modals.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: help.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/help.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin)
- [snapshots/showcase/pages/dialogs](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/dialogs)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
HelpOverlay::new(id: Id, sections: &'a [HelpSection<'a>]) -> HelpOverlay<'a>

update(&self, cx: &mut Cx<'_>, state: &mut HelpOverlayState) -> Response<HelpAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &HelpOverlayState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
bindings(&'a BindingView)
patch(StylePatch)
```

**Durable state:** HelpOverlayState: viewport scroll only.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** HelpAction::Dismissed

## Visual parts and composition

Advertised styled parts: `container`, `title`, `section`, `chord`, `description`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [dialog](./dialog.md), [text-viewport](./text-viewport.md), [key-hint](./key-hint.md)

## Behavioral contract

1. Use Dialog, KeyHint and TextViewport; preserve baseline grouping and width-sensitive columns.
2. Help descriptions may be explanatory rather than runnable actions, but shown bindings must agree with actual enabled remaps.
3. Escape/outside closes through the runtime layer and restores the prior focus and draft.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| layer | `closed`, `open`, `nested`, `escape`, `outside`, `owner_removed`, `resize_reanchor`, `focus_restore`, `modal_first_paste` | runtime-owned overlay |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W26-01 | wide columns collapse at narrow width |
| W26-02 | remapped/disabled action representation matches HintBar and Menu |
| W26-03 | scroll help while underlying editor is active |
| W26-04 | close restores exact owner focus |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/help-overlay.json](../capture-plans/help-overlay.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
