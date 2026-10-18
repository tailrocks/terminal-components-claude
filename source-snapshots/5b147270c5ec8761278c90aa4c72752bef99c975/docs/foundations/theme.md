# Semantic theme and style resolution

**Canonical owner:** semantic surfaces, roles, recipe lookup, part patches,
and terminal color-capability conversion.
This is the target contract for the in-place refactor on `termrock-implementation`;
it does not describe a completed source rename.

**Specification record:** F06, legacy family C06. The visual authority is the
immutable [`visual-baseline` commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).

Read this with the [visual contract](../design/visual-contract.md),
[layout and measurement](layout.md), [public API](../api/public-api.md), [shared
types](../api/types.md), and [conformance rules](../verification/conformance.md).
This page owns style data and resolution. Components own which declared parts
they expose; runtime owns focus, capture, layers, time, and action dispatch.

## Baseline evidence and future names

The pinned implementation is the evidence in
[`src/theme.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/theme.rs),
[`src/ui/fade.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/fade.rs),
and [`src/ui/text.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/text.rs).
The current source still exposes a legacy `Theme::junie()` constructor and
literal ratatui `Color` fields. Those are migration evidence, not future
branding or compatibility requirements. The target default is
`Theme::termrock()` and must reproduce the same rendered cells in the four
frozen applications.

## Responsibility

This foundation owns:

- semantic `Surface`, `Role`, `Tone`, variant, and state-style resolution;
- `ColorLevel` capability conversion for truecolor, 256-color, 16-color, and
  monochrome output;
- deterministic recipe, fallback, global/subtree, instance, and declared-part
  patch precedence;
- inherited surface context and constrained part/row/cell slot painting;
- style-side inputs to backdrop and fade painting while preserving
  written-cell ownership.

The [visual contract](../design/visual-contract.md) owns exact baseline token
values, spacing, glyphs, density, and rendered state recipes. The
[interaction contract](../design/interaction-contract.md) owns when
focus/hover/press/activation states exist and their observable timing. This
foundation resolves the semantic roles supplied by those contracts. Layout
owns rectangles and clipping; authoring owns the restricted
`PartUi`/`RowUi`/`CellUi` vocabulary. No component may create a second style
cascade.

## Non-goals

- no product-specific theme, logo, brand widget, or application redesign;
- no CSS-like trait hierarchy or per-component RGB literals;
- no focus router, hit registry, pointer capture, layer stack, clock, or event
  dispatch;
- no geometry mutation from a style patch or part slot;
- no raw terminal-cell recoloring through semantic roles. `TerminalView` may
  use the qualified raw-cell adapter defined below;
- no promise that a new Paper palette is baseline parity. It is an extension
  lane used to expose hardcoded colors and verify theme separation.

## Proposed data and API contract

The following Rustdoc-style declarations are target notation. They are not
compiled code and do not require the current legacy names to remain.

```rust
pub fn Theme::termrock() -> Theme;
pub fn Theme::paper() -> Theme;
pub fn Theme::with_palette(self, level: ColorLevel, tokens: ColorTokens) -> Theme;
pub fn Theme::patch(
    self,
    family: Family,
    variant: Variant,
    part: Part,
    patch: StylePatch,
) -> Theme;

pub enum ColorLevel { TrueColor, Ansi256, Ansi16, Mono }

pub enum Surface {
    Canvas,
    Surface,
    Elevated,
    Overlay,
    Popover,
    Field,
    FieldHover,
}

pub enum PatchSlot<T> { Inherit, Set(T), Clear }

pub struct StylePatch {
    pub foreground: PatchSlot<Role>,
    pub background: PatchSlot<Role>,
    pub modifiers: ModifierPatch,
}

impl Ui<'_> {
    pub fn with_patch<R>(
        &mut self,
        patch: ScopedPatch,
        body: impl FnOnce(&mut Ui<'_>) -> R,
    ) -> R;
}
```

`Role`, `Tone`, `Family`, `Variant`, `Part`, `ColorTokens`, and
`ModifierPatch` are shared semantic types. `PatchSlot::Inherit`, `Set`, and
`Clear` are intentionally different: omission inherits, `Set` replaces a
role, and `Clear` removes the declared value so the next resolution layer can
apply its documented fallback. Unsupported part names are diagnosed instead
of silently ignored.

## Default theme entry point

`Theme::termrock()` selects the baseline semantic token set. Its exact values,
surface hierarchy, spacing, glyphs, and component-visible state treatments
have one owner: the [visual contract](../design/visual-contract.md). The full
palette must be carried through the capability conversion; a partial palette
is a parity failure. New palette behavior remains a separately reviewed
extension.

## Resolution rules

Resolve every painted part in this order:

1. base family recipe;
2. component variant;
3. state rules supplied by props and runtime;
4. monochrome fallback for the selected capability;
5. global theme override;
6. inherited subtree override;
7. instance patch;
8. more-specific instance part patch.

Bind the resulting semantic roles to capability-specific colors and the
inherited surface last. A child receives a surface from its container; it does
not compare RGB values to infer one. Standard painters must ask the theme for
roles or family resolvers instead of constructing `Color` values.

State resolution applies the recipe selected by the component and runtime.
The exact visible rule for each state is owned by the visual and interaction
contracts and the component's own contract; this foundation does not add a
second state table.

## Capability and motion policy

Capability conversion is deterministic and cannot exceed the known terminal
ceiling. Explicit `NO_COLOR` remains distinguishable from a caller-selected
monochrome level as an input lane. The exact resulting palette and behavior
are in the visual contract. The runtime owns monotonic time and motion samples;
their observable phases and timing are in the interaction contract. Drawing
never advances a clock.

## Parts, slots, and raw-cell boundary

Each component advertises its replaceable parts. `patch_part` changes semantic
style for one declared part. `slot` invokes an immediate borrowed callback in
the part's reserved rectangle using constrained author APIs. A slot may not:

- erase the component or paint outside its clip;
- change measured geometry, hitboxes, focus ownership, layers, or capture;
- restyle an unrelated sibling;
- mutate a domain model, access a global buffer, or install a router.

Custom row/cell painters receive the owner patch and the same semantic state
as the stock painter. Terminal raw-cell blitting is the one qualified
exception: `TerminalView` preserves caller-provided terminal foreground,
background, continuation, and supported modifiers for child cells while its
chrome still uses Termrock roles and its rectangle remains clipped.

## Integration rules

- `update`, `draw`, and `measure` use one resolved theme context. Draw takes
  immutable props/state and does not mutate theme or component state.
- Layout owns rectangles and inherited surface transport; theme owns the
  meaning of a surface and style. Runtime owns which semantic state flags are
  active. This separation prevents a component from inferring focus or hover
  from color.
- Text segmentation and Unicode width use the [text foundation](text.md);
  style spans carry semantic tones and modifiers rather than pre-resolved
  RGB values.
- Grid/list/tree/custom presenters use the same owner patch for row/cell
  slots. Component pages define their advertised parts and visual state
  matrices; this page defines the shared resolution algorithm.
- `StatusBar` absorbs old segments-style presentation and `DerivedHintBar` is
  metadata adaptation. Neither introduces a second theme or rendering engine.

## Conformance and negative tests

Theme effects are checked through exact painted-cell comparisons and the
component/composed fixture cases in the [visual parity](../verification/visual-parity.md)
contract. Record dimensions, symbols, foreground, background, modifiers,
cursor, focus/capture, and typed action trace. Classify cases as
`ExistingOracle`, `ExtractedOracle`, or `Extension`; Paper and newly approved
capability robustness cases are extensions unless a pinned baseline case
exists.

Required proof:

- the same component in truecolor, 256-color, 16-color, explicit mono, and
  `NO_COLOR` lanes;
- focus/hover/disabled/pressed/selection precedence in painted cells;
- `Clear` versus `Inherit` on a part patch;
- custom row and cell slots receive the owner's patch and cannot escape their
  reserved rectangle;
- Paper sentinel theme exposes every standard painter that hardcodes a
  baseline color;
- no raw color literal in a standard component painter except the documented
  terminal-cell adapter.

Negative tests must fail when a patch silently changes geometry, an unsupported
part is ignored, an instance override misses a custom presenter, capability
mapping exceeds the ceiling, mono loses focus/selection distinction, a
backdrop paints through a layer, or a mutation in a shared resolver leaves a
component's expected cells unchanged.

## Source-pack coverage

This page migrates F06's complete semantic theme contract: proposed theme and
patch API, seven surfaces, capability lanes, explicit `NO_COLOR`, resolution
precedence, inherited surfaces, baseline token anchors, focus/hover/selection/
disabled rules, 140 ms activation styling, fades, constrained part slots,
custom presenter ownership, and the qualified terminal-cell exception. The
F06 mapping to legacy family C06 remains available to the repository's
family-disposition ledger. F03 owns time and runtime state; F05 owns geometry;
F10 owns authoring permissions; they are linked rather than redefined here.
