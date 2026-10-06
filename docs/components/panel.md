# Panel

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W32 · Phase: P2 · Legacy families: C41, C40.

## Purpose and exclusions

Panel is Termrock's chrome-only container. The contract covers card and framed recipes and their open clipped body slot. It does not retain a child tree or become an interactive widget.

Panel owns borrowed chrome props: kind, title, metadata, badge, focus-within state, and semantic style patch. The caller supplies child composition and child state.

Panel must not retain children or domain state; create a decorative focus stop, hit target, or activation action; own scrolling, focus, pointer capture, or child selection; invent a layout when the body allocation is empty; or promise a universal widget trait or compatibility with current `WidgetId`, `Outcome`, or `RenderCtx` internals.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked.

```rust
Panel::new(id: Id) -> Panel<'a>

draw<R>(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> R

measure(
    &self,
    cx: &MeasureCx<'_>,
    child: Size,
    constraints: Constraints,
) -> Size
```

Configuration:

```rust
kind(PanelKind::Card | PanelKind::Framed)
title(&'a str)
meta(StyledText<'a>)
badge(Option<Badge<'a>>)
focus_within(bool)
patch(StylePatch)
```

Panel has no durable state and no typed action family. Child responses pass through unchanged. The callback body receives a clipped allocation; border and title are not child hit targets.

## Ordinary use

Panel appears in [EX-06 — Tree and detail layout](../api/consumer-recipes.md) and [EX-11 — A log viewport, not a repainted text block](../api/consumer-recipes.md) (both `proposed_target`).

The real preview consumer is the Showcase panels page; child focus and scroll composition are also exercised by the four preserved applications. An independent synthetic fixture supplies a title, metadata, and a small child body at exact-fit width.

The caller composes a child inside the body slot and reads the child response unchanged. Panel plus TextViewport is the target replacement for the old ScrollPanel composition; no separate ScrollPanel engine is retained.

## Ownership

- The caller owns child composition and child state.
- Panel owns only borrowed chrome: kind, title, metadata, badge, focus-within state, and semantic style patch.
- There is no Panel `update`: it is display-only. Runtime focus-within facts are read from the child composition or supplied as borrowed props. Panel must not claim focus or capture.
- `draw` paints a filled card or rounded framed surface, title row, metadata, badge, and clipped body. It invokes the child exactly once in the returned inner area. A zero/tiny intersection emits no invalid geometry. If metadata depends on child layout (for example a scroll position), the composition may repaint the title row after fresh measurement; this is a deterministic chrome pass, not a state mutation.
- `measure` reserves the card/frame insets and title row according to kind and constraints, then returns the body allocation facts used by the caller. It must not inspect or mutate child state.
- `draw` paints chrome and invokes a body slot inside the exact inner clip; `measure` accounts for chrome around the child size.

Shared dependencies: [`identity`](../foundations/identity.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `border`, `title`, `meta`, `badge`, and `body`.

Ordinary customization patches declared title, meta, or badge tones. Card is a filled surface with the baseline title row and padding. Framed is the rounded border recipe with its exact inset and title gap. Focus-within changes the frame/title treatment only; the child remains the focus owner and keeps its focus gutter. No color or motion reinterpretation is introduced. Capability/motion resolution uses the shared theme foundation.

Advanced customization keeps the title/metadata width budget: the title yields first to preserve metadata, then metadata truncates while the title keeps its minimum naming cells. Badge claims space only when it fits. Body paint inherits the selected surface and clip. Nested panels and nonzero origins retain exact border, padding, clipping, and background behavior.

Part patches cannot replace geometry, child focus/capture, or whole-surface ownership. The panel border never becomes a substitute hit target for children.

## Behavior

Panel has no independent focus, hover, press, pointer, keyboard, edit, scroll, or activation behavior. A containing focus indication is a visual reflection of child focus only. Child interactions continue to use their own canonical component contracts. Child focus must not change child selection semantics; focus-within is visual only.

## Visual matrix

| Axis | Required states | Disposition |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Component |
| Focus/hover | none on Panel; child focused/hovered may change `focus_within` | Child owner |
| Activation | none | Not applicable |
| Editing/scroll | none | Child owner |
| Readiness | none unless a child body renders it | Child owner |
| Motion | only shared border/theme transition if a recipe advertises it | Theme policy |

## Verification

Parity is pinned to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source evidence is `src/widgets/panel.rs` (blob `14a9edef27f16e463a635f314fa37d5825f57fd9`). Inline tests cover title/meta width budgets, late metadata repaint after child measurement, card/framed geometry, nested clipping, and scroll-panel tail or prose behavior now split between Panel and TextViewport. The visual consumer is Showcase panels, with protected snapshots under `snapshots/showcase/pages/panels`; child focus and scroll composition are also exercised by the four preserved applications and `tests/visual_baseline`. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/panel.json`](../reference/capture-plans/panel.json). It is planned with no expected artifacts. Bind baseline output as `ExistingOracle` or `ExtractedOracle`; any new API/safety guarantee is an `Extension`.

Run applicable cases at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`, with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities.

| Case | Required observation |
|---|---|
| W32-01 | Card/framed title, metadata, and badge at exact-fit widths. |
| W32-02 | Child focused versus Panel remaining decorative. |
| W32-03 | Nested panels, nonzero origins, and clipping. |
| W32-04 | Empty/zero/tiny body and late scroll-count metadata. |

Record exact area, inner clip, title/meta/badge cells, child focus owner, capability, and any late metadata repaint. Compare symbols, wide continuations, foreground/background, modifiers, and dimensions exactly.

Required negative tests:

- Panel cannot register an independent focus stop or generic activation action.
- Panel cannot mutate child state, scroll state, or metadata during `draw`.
- Body output cannot paint outside the inner clip or over the border/title.
- Title/meta/badge truncation cannot overwrite corners, padding, or each other.
- Zero/tiny/nonzero-origin areas cannot panic or underflow dimensions.
- Child focus must not change child selection semantics; focus-within is visual only.
- A custom part patch cannot replace child layout, hit geometry, or capture.
- No ScrollPanel implementation may be introduced beside Panel and TextViewport.

Acceptance requires an external body-slot consumer, exact applicable snapshots, and independent review. A decorative placeholder or a panel that swallows child responses fails acceptance.

## Rejected use

Forbidden: drawing a manual border and title with padded strings instead of using Panel.

```rust
// Forbidden: preview owns chrome rendering by hand.
ui.paint_str(0, 0, "╭─ Details ─────────╮");
ui.paint_str(1, 0, "│");
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must not recreate component rendering or generic interaction. The following patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `Panel::new` with a body-slot callback and let the child own its interaction.

## Known gaps

- Capture plan W32 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
