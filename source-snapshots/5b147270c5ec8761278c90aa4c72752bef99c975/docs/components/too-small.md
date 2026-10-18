# TooSmall

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W43 · Group: Chrome · Phase: P2.

## Purpose and exclusions

TooSmall is a generic minimum-size notice. The caller supplies the threshold and exact message. The library does not impose one product's minimum on every component.

Exclusions:

- Redesigning the application when the terminal is small.
- Owning quit/exit handling.
- Reconstructing child scenes or discarding their durable state.
- Assuming the Jackin threshold applies to Showcase, TablePro or Holla.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
TooSmall::new(minimum: Size, message: &'a str) -> TooSmall<'a>

TooSmall::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
TooSmall::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

TooSmall::patch(StylePatch);
```

TooSmall has no durable state or typed action. The caller/runtime retains previous component state while the notice is shown; exit handling stays in the host binding scope.

The baseline-like Jackin fixture uses 72 by 20. That is an application composition case, not a global Termrock default.

## Ordinary use

Existing consumers include frozen Jackin views; their baseline output remains protected.

The caller supplies the minimum size and exact message; the host/application composition decides whether to show the notice. Growing back above the threshold reveals the existing scene and restores prior focus/draft state; it does not reconstruct the scene.

## Ownership

| Concern | Owner |
| --- | --- |
| Minimum size and exact message | Caller |
| Whether to show the notice | Host/application composition |
| Existing focus, capture, draft and modal state | Runtime/application |
| Painting and clipping | TooSmall |

Shared dependencies: [`layout`](../foundations/layout.md), [`runtime`](../foundations/runtime.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, message and dimensions. Preserve baseline notice text, alignment, dimensions and surface treatment supplied by the caller.

Unicode message width and clipping follow [text](../foundations/text.md); semantic styling and no-color behavior follow [theme](../foundations/theme.md). Resize behavior is an explicit geometry state, with no motion.

Ordinary example: `TooSmall::new(Size::new(72, 20), "Terminal too small")`.

## Behavior

- The component is display-only and does not consume quit or other host bindings. The notice is not a focus stop; the host remains responsible for quit.
- draw is bounded by area and safe for zero/tiny rectangles. Unsigned geometry arithmetic cannot underflow; draw cannot write outside area.
- Showing the notice cannot discard focus, pointer capture or drafts.
- TooSmall cannot consume host quit or invent an action.
- A product-specific threshold cannot become a library default.
- Grow-after-shrink cannot create a fresh state object.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, 71/72/73 columns × 19/20/21 rows for the baseline-like fixture, 0×0, 1×1, nonzero origin |
| Focus/hover | Not applicable; notice is not a focus stop |
| Activation | Not applicable; host remains responsible for quit |
| Continuity | shrink with modal open, then grow; preserve focus and draft |
| Color/capability | semantic fallback across all capture capabilities |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen Jackin composition is frozen-at-4a79c0a2 ([app.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/app.rs)). Use [`../reference/capture-plans/too-small.json`](../reference/capture-plans/too-small.json), the [layout contract](../foundations/layout.md), the [runtime contract](../foundations/runtime.md) and [visual parity proof](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W43-01 | 71/72/73 columns crossed with 19/20/21 rows |
| W43-02 | 0×0, 1×1 and nonzero-origin notices |
| W43-03 | Shrink while modal open then grow restores focus/draft |
| W43-04 | Quit remains available while too small |

Record exact cells/styles, threshold, actual size, host focus/capture and draft state. The last case is an application/runtime trace, not a TooSmall action.

Required negative tests:

- unsigned geometry arithmetic cannot underflow;
- draw cannot write outside area;
- showing the notice cannot discard focus, pointer capture or drafts;
- TooSmall cannot consume host quit or invent an action;
- a product-specific threshold cannot become a library default;
- grow-after-shrink cannot create a fresh state object.

## Rejected use

Forbidden: hardcoding one product's size answer in preview drawing instead of using TooSmall with a caller threshold.

```rust
// Forbidden: preview hardcodes the small-screen answer.
if width <= 72 && height <= 20 {
    ui.paint_str(0, 0, "Too small, enlarge!");
}
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50". Use `TooSmall::new` with a caller-supplied minimum and message.

## Known gaps

- Capture plan W43 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
