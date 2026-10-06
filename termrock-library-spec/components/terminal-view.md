# W44 · TerminalView

**Group:** Adapter boundary · **Phase:** P6 · **Classification:** adapter-extension  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow a terminal-cell snapshot, supported mode/selection metadata and stable history coordinates. A future Jackin adapter supplies termpane data; no termpane dependency is required in the core library.

Legacy family mapping: New adapter boundary; not a historical widget-family claim.  
Proposed implementation home: `crates/termrock/src/components/terminal_view.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/bin/jackin_preview/screens/capsule.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/capsule.rs)
- [src/bin/jackin_preview/sim/pty.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/sim/pty.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- No corresponding standalone refactored widget is claimed; this is an explicit new generic adapter boundary.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin/capsule](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/capsule)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
TerminalView::new(id: Id, source: &'a dyn TerminalSource) -> TerminalView<'a>

update(&self, cx: &mut Cx<'_>, state: &mut TerminalViewState) -> Response<TerminalAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TerminalViewState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
interaction(TerminalInteraction)
selection(Option<TerminalSelection>)
dimmed(bool)
patch(StylePatch) // chrome only; child cell colors preserved
```

**Durable state:** TerminalViewState: view offset and selection anchor in stable line/cell coordinates. The committed selection is caller-controlled. No terminal parser, PTY process, mode authority or session data.

Safe access: source-keyed selection/caret and reading offset. Mutations are explicit commands or dispatched input; no global Buffer or source collection is exposed.

**Typed actions:** TerminalAction::{Forward { token: InputToken }, CopyRequested(TerminalSelection), OpenLink { key: LinkKey }}

## Visual parts and composition

Advertised styled parts: `container`, `terminal-cell`, `cursor`, `selection`, `link`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md)

## Behavioral contract

1. This is an explicit new generic adapter API extracted from app-local baseline rendering. It is not an existing standalone baseline widget; baseline region parity and extension conformance are separate lanes.
2. Blit original cell styles and wide-cell continuation metadata; do not recolor child terminal output to the Termrock theme. Dim only when the baseline overlay policy requires it.
3. Outer modal/prefix ownership wins before forwarding input. Return InputToken so the application can retrieve its original event representation. Byte-for-byte forwarding requires host-retained raw input; the widget does not reconstruct discarded bytes.
4. Draw reports inner viewport and cursor facts; a subsequent update/host adapter handles resize requests. Never resize PTYs as a paint side effect.
5. Clipboard/export, hyperlink safety policy, terminal modes, daemon reconnect and process lifetime stay outside Termrock.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| output | `source_replaced`, `source_appended`, `selection`, `source_copy`, `cursor_clipped` | borrowed content projection; eviction only for retained-history sources, cursor only when owned |
| retention | `history_evicted`, `evicted_selection`, `retained_reading_anchor` | sources with retained line history |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W44-01 | borrowed cell buffer with wide cells, combining symbols and cursor visibility |
| W44-02 | focused/inactive/dimmed terminal regions in generic pane fixture |
| W44-03 | modal intercepts input; normal mode emits forwarding token |
| W44-04 | selection mode copies by source coordinates and rejects stale history |
| W44-05 | viewport resize reports facts but performs no IO |
| W44-06 | new protocol attributes explicitly classified outside baseline proof |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/terminal-view.json](../capture-plans/terminal-view.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
