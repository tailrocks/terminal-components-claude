# W20 · PickerChain

**Group:** Overlays · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller owns stage availability, result data and async generations. Generic navigation remembers view state; it does not know vaults, providers or directories.

Legacy family mapping: `C29`  
Proposed implementation home: `crates/termrock/src/components/picker_chain.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/bin/jackin_preview/screens/modals.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: picker_chain.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/picker_chain.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
PickerChain::new(id: Id, stages: &'a [PickerStage<'a>], revision: Revision) -> PickerChain<'a>

update(&self, cx: &mut Cx<'_>, state: &mut PickerChainState) -> Response<PickerChainAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerChainState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
current(ItemKey)
readiness(Readiness<'a>)
patch(StylePatch)
```

**Durable state:** PickerChainState: stable stage path plus saved query/cursor/scroll per stage. No backend results or credentials.

Safe access: current stable result/stage key, `query() -> &str` and scroll observations. No mutable access to caller-owned result data.

**Typed actions:** PickerChainAction::{Accept { stage: ItemKey, key: ItemKey }, Back, Retry { stage: ItemKey }, Cancel}

## Visual parts and composition

Advertised styled parts: `container`, `breadcrumb`, `title`, `query`, `row`, `footer`, `empty`, `status`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [picker](./picker.md)

## Behavioral contract

1. Back restores the prior stage query, highlighted key and scroll. Cancel exits the chain without resetting an underlying form.
2. Retry emits a stable stage key. Out-of-date source results are ignored by caller generation checks; the component reconciles only the accepted revision.
3. A changed earlier choice invalidates incompatible later stage view state without rebinding an old key to a new result.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| readiness | `loading`, `partial`, `error`, `retry_or_refresh_when_offered` | explicit readiness or item lifecycle supplied by caller, not synthetic whole-widget states |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| layer | `closed`, `open`, `nested`, `escape`, `outside`, `owner_removed`, `resize_reanchor`, `focus_restore`, `modal_first_paste` | runtime-owned overlay |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W20-01 | stage1 -> stage2 -> Back restores query and selection |
| W20-02 | loading/error/retry at each stage |
| W20-03 | stale completion for an abandoned stage produces no selection change |
| W20-04 | nested form retains all drafts after chain Cancel |
| W20-05 | breadcrumb collapse at narrow widths |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/picker-chain.json](../capture-plans/picker-chain.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
