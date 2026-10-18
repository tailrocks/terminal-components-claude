# Layout and measurement

**Canonical owner:** geometry, measurement, clipping and responsive allocation for
Termrock components. This is the target contract for the in-place refactor on
`termrock-implementation`; it is not a claim about the current Rust implementation.

**Specification record:** F05, legacy family C05. The visual evidence is the
immutable `visual-baseline` commit
[`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).

Read this with the [architecture overview](../architecture/overview.md),
[runtime contract](runtime.md), [layer contract](layers.md),
[public API](../api/public-api.md), [shared types](../api/types.md), and the
[conformance contract](../verification/conformance.md). Those documents own
runtime dispatch, semantic identity, layers, and the public naming rules. This
document owns geometry and measurement only.

## Baseline evidence and future names

The frozen implementation exposes the relevant behavior through
[`src/ui/layout.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/layout.rs),
[`src/widgets/splitter.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/splitter.rs),
and [`src/widgets/panel.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/panel.rs).
The current `Split`, `Rect`, `RenderCtx`, and ratatui types are implementation
evidence. They are not future compatibility requirements. The refactor may
change their spelling while preserving the frozen applications' cells,
geometry, focus behavior, and pointer behavior.

## Responsibility

This foundation owns:

- terminal-cell `Rect` and `Size` arithmetic, min/max `Constraints`, track
  allocation, gaps, clipping rectangles, and split areas;
- pure `measure` and reusable layout helpers used by both drawing and hit
  registration;
- allocation facts for responsive layouts, including whether a region is
  side-by-side or requires an overlay/drawer;
- design metrics read from `MeasureCx` and inherited surface metadata passed
  down a component tree;
- deterministic handling of empty, narrow, resized, and non-zero-origin
  rectangles.

The runtime owns focus, hover, pointer capture, layer order, event dispatch,
and publication of hit regions. The theme foundation owns semantic colors,
style resolution, and surface tokens. The collections foundation owns stable
row/column identities and source reconciliation. Components own the choice of
which layout helper to use and what their measured content means.

## Non-goals

- no focus ring, hover state, press state, pointer capture, layer stack, clock,
  or event routing;
- no color or raw terminal-background argument in a component's public
  contract; semantic surfaces and style patches belong to
  [theming](theme.md);
- no mutation of durable component state during `measure` or `draw`;
- no product-specific `WorkspaceShell`, workbench, provider layout, or
  application navigation API;
- no universal `72x20` minimum. That size is a baseline composed-fixture
  condition; every helper and component must still define behavior for local
  `0x0` and `1x1` allocations;
- no second layout engine hidden inside a component. Panel, split, grid,
  fields, overlays, and text views consume this contract.

## Proposed data and API contract

The following Rustdoc-style declarations describe the target public surface.
They are design notation, not compiled code or an implementation promise.

```rust
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Constraints {
    pub min: Size,
    pub max: Size,
}

pub enum Track {
    Fixed(u16),
    Flex(u16),
    Auto,
}

pub trait Measure {
    fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
}

pub fn rows(area: Rect, tracks: &[Track], gap: u16) -> Vec<Rect>;
pub fn columns(area: Rect, tracks: &[Track], gap: u16) -> Vec<Rect>;
pub fn action_row(
    area: Rect,
    sizes: &[Size],
    align: Alignment,
    gap: u16,
) -> Vec<Rect>;
pub fn responsive_columns(area: Rect, spec: ResponsiveSpec) -> ResponsiveAreas;

impl Ui<'_> {
    pub fn with_surface<R>(
        &mut self,
        surface: Surface,
        body: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R;

    pub fn clip<R>(&mut self, area: Rect, body: impl FnOnce(&mut Ui<'_>) -> R) -> R;
}
```

`MeasureCx` is immutable. It can expose the resolved theme's design metrics,
capability and the immutable caller model needed to measure dynamic content.
It does not expose mutable state, event queues, a global buffer, a runtime
registry, or an executor. Components use `update` for semantic state changes,
`measure` for size, and `draw` for immutable paint plus geometry publication;
see the [API lifecycle](../api/public-api.md#ownership-and-phases).

### Allocation invariants

1. `Constraints.min <= Constraints.max` component-wise. Invalid constraints are
   rejected or normalized at the boundary; arithmetic never wraps.
2. Every returned rectangle is inside the input area, retains its non-zero
   origin, and has a checked or saturating extent. A zero-width or zero-height
   area yields empty child rectangles and never panics.
3. A track allocation accounts for every gap exactly once. Fixed tracks are
   honored when possible; finite flex weights divide remaining cells with a
   deterministic remainder policy; `Auto` uses measured content bounded by
   constraints. If minima cannot fit, the helper follows its documented
   collapse policy rather than producing overlapping rectangles.
4. `rows`, `columns`, `action_row`, split layout, and hit geometry use the same
   calculated rectangles. A component must not recompute a different rectangle
   while registering a click target.
5. Measurement is deterministic for equal props, immutable state/model,
   constraints, and theme metrics. It does not advance scrolling, select a
   row, enter editing, mutate a text buffer, or resolve focus.
6. Responsive helpers return allocation facts and breakpoint metadata. The
   caller chooses whether content is moved into a drawer, collapsed, or kept
   side-by-side; the helper does not move focus or invent a product workflow.

### Surfaces and clipping

Containers pass an inherited semantic `Surface` into child paint. A child does
not accept a trailing raw background color and no widget infers a surface by
comparing RGB values. `Surface` resolution and `StylePatch` behavior are owned
by [the theme contract](theme.md); this page only defines how the surface
travels with a measured/painted region.

Clipping is hierarchical. A child painter receives its reserved rectangle and
cannot write outside it. Wide-glyph continuation handling belongs to the text
projection contract, but layout must provide the exact clip edge so a glyph at
the right boundary cannot leak into a sibling or scrollbar.

## Integration rules

- `measure` runs before a component's area is chosen when the caller needs
  intrinsic or constrained sizing. The resulting area is passed to `draw`.
- `draw` publishes the geometry that the runtime will use for pointer hit
  testing. Input is dispatched against the last committed valid publication;
  resize or topology changes invalidate it. The runtime contract owns that
  transaction and stale-geometry rejection.
- `Panel` plus `TextViewport` supplies the canonical scrollable text-pane
  composition. A separate `ScrollPanel` layout mechanism is not introduced.
- `SplitPane` uses one split allocation engine for horizontal and vertical
  directions, minimums, seam width, resize, and maximize. Keyboard resize and
  pointer drag update caller-owned split state through typed actions; geometry
  itself is not durable state.
- `Grid`, `Form`, `MenuBar`, `Props`, `Dialog`, `Wizard`, and fields use these
  helpers for exact-fit and narrow-width behavior. Their component pages own
  visual parts and state matrices; this page owns only the shared geometry
  invariant.
- Width-dependent caches include all relevant dimensions, wrapping policy,
  source revision, and theme metrics. A resize invalidates stale layout facts
  before the next coordinate-dependent interaction.

## Conformance and negative tests

The verification harness records dimensions, published rectangles, clipped
cells, cursor/capture ownership, and typed actions alongside visual output.
Layout has no independent hover or pressed screenshot; those effects are
proved by the component and composed application cases in the
[visual](../verification/visual-parity.md) and
[interaction](../verification/interaction-parity.md) contracts. Cases are
classified as `ExistingOracle`, `ExtractedOracle`, or `Extension` by the
conformance registry.

Required layout proof:

- all helpers at `0x0`, `1x1`, and non-zero origins;
- exact-fit and one-cell-short action rows, including deterministic alignment;
- horizontal/vertical split symmetry, minima, seams, collapse, maximize, drag,
  and resize;
- nested clipping with a wide grapheme at the right edge and no neighboring
  cell corruption;
- responsive breakpoint round-trip that retains caller selection, focus, and
  durable state;
- draw and hit registration consume identical rectangles after resize;
- dynamic panel metadata uses the current layout facts on the first frame.

Negative tests must fail when arithmetic wraps, rectangles overlap, hitboxes
disagree with paint, measure mutates state, a child escapes its clip, a
non-finite ratio is accepted, a hard-coded global minimum rejects a valid
small component, or a product-specific shell API is smuggled into this
foundation. A candidate implementation cannot create or approve its own
expected visual baseline.

## Source-pack coverage

This page migrates F05's measurement/surface/responsive contract in full:
`Size`, `Constraints`, `Track`, `Measure`, row/column/action-row helpers,
responsive allocation, inherited surfaces, zero-size and non-zero-origin
arithmetic, the baseline fixture-size qualification, and all five required
proof groups. The F05 mapping to legacy family C05 is retained for the
repository's family-disposition ledger. No F01 identity, F03 runtime, F04
layer, or F06 theme rule is redefined here; those contracts are linked above.
