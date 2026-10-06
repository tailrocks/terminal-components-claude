# W30 · DiffView

**Group:** Data and text · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow already computed diff rows and original source ranges. Computing Git diffs, accepting patches and mutating files are not component responsibilities.

Legacy family mapping: `C38`  
Proposed implementation home: `crates/termrock/src/components/diff.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/diff.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/diff.rs) — observed Git blob `e0b5bdefa3b7fa514295a245de80ebf0f3bcf438`
- [src/bin/jackin_preview/screens/inspect.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/inspect.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: diff.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/diff.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/audit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/audit)
- [snapshots/jackin](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
DiffView::new(id: Id, source: &'a dyn DiffSource) -> DiffView<'a>

update(&self, cx: &mut Cx<'_>, state: &mut DiffViewState) -> Response<DiffAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &DiffViewState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
mode(DiffMode) // Unified or Review
wrap(WrapMode)
patch(StylePatch)
```

**Durable state:** DiffViewState: requested DiffMode, shared ViewportState and keyed hunk/line navigation. Effective narrow mode is derived, never overwrites requested mode.

Safe access: source-keyed selection/caret and reading offset. Mutations are explicit commands or dispatched input; no global Buffer or source collection is exposed.

**Typed actions:** DiffAction::{SelectionChanged, CopyRequested(TextSelection), HunkActivated { key: ItemKey }}

## Visual parts and composition

Advertised styled parts: `container`, `header`, `old-gutter`, `new-gutter`, `context`, `addition`, `deletion`, `emphasis`, `selection`, `scrollbar`, `fade`, `empty`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [text-viewport](./text-viewport.md)

## Behavioral contract

1. Preserve requested review mode across a too-narrow terminal: draw the baseline unified fallback and restore review on widening.
2. Scrollbar width participates in the breakpoint calculation. Tab expansion, intra-line emphasis and selection use shared display mapping.
3. Copy returns original selected content according to the reference side/mode semantics, not decorated gutter text.
4. An empty diff is a shared Empty/readiness presentation, not an invented success screen.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| output | `source_replaced`, `source_appended`, `selection`, `source_copy`, `cursor_clipped` | borrowed content projection; eviction only for retained-history sources, cursor only when owned |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W30-01 | unified/review with additions, deletions, context and intraline emphasis |
| W30-02 | wide -> narrow -> wide restores requested Review |
| W30-03 | exact width around split/scrollbar threshold |
| W30-04 | selection crosses tabs, wide graphemes and hunk boundaries |
| W30-05 | empty/no-change and source revision replacement |
| W30-06 | click/keyboard selection equivalence and raw-source copy |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/diff-view.json](../capture-plans/diff-view.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
