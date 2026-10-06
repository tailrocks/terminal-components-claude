# F06 · Semantic themes, recipes, parts and color capability

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C06  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/theme.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/theme.rs)
- [src/ui/fade.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/fade.rs)
- [src/ui/text.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/text.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
Theme::termrock() -> Theme
Theme::paper() -> Theme
Theme::with_palette(self, level: ColorLevel, tokens: ColorTokens) -> Theme
Theme::patch(self, family: Family, variant: Variant, part: Part, patch: StylePatch) -> Theme
enum ColorLevel { TrueColor, Ansi256, Ansi16, Mono }
enum Surface { Canvas, Surface, Elevated, Overlay, Popover, Field, FieldHover }
enum PatchSlot<T> { Inherit, Set(T), Clear }
struct StylePatch { foreground: PatchSlot<Role>, background: PatchSlot<Role>, modifiers: ModifierPatch }
Ui::with_patch<R>(patch: ScopedPatch, body: impl FnOnce(&mut Ui<'_>) -> R) -> R
```

## Contract

1. Theme::termrock preserves the pinned default visual tokens. Initial anchors include canvas #000000, surface #111111, elevated #18181b, overlay #27272a, field #1e1e22, field-hover #232328, popover #3f3f46 and accent #48e054. Complete values/recipes are extracted from the pinned source; these anchors alone are not a full palette.
2. Keep colors, spacing, borders, glyphs, density and timing in concrete theme data, not widget literals or a theme trait. A small Paper palette demonstrates separation; its new visuals are extension conformance, not claimed baseline parity.
3. Override order is explicit: base recipe -> variant -> state rules -> monochrome fallback -> global override -> subtree override -> instance patch -> more-specific instance part patch. Inherit, Set and Clear are distinct. Bind semantic roles to capability/surface colors last.
4. Part slots are borrowed, immediate paint callbacks within reserved rectangles. A slot replaces only its declared part; it cannot erase the whole component, alter hitboxes or install its own focus router.
5. Preserve truecolor, 256,16, explicit none and environment NO_COLOR capture lanes. None and NO_COLOR can produce equivalent palettes while testing different input policy paths. User-requested capability cannot silently exceed the terminal ceiling.
6. Baseline focus uses a gutter; hover lifts one plane; selection/check markers remain distinct; errors and warning tones do not remove focus. Green budget and progress-versus-capacity roles remain separate.
7. Raw terminal cell blitting is a qualified adapter exception: child terminal colors remain terminal data, not semantic Termrock roles.

## Required proof

- same component across all five capability lanes
- focus/hover/disabled precedence painted-cell tests
- Clear versus Inherit part patch
- custom row/cell slots use owner patch
- Paper sentinel theme exposes hardcoded baseline colors
- no Color literals in standard component painters outside documented adapter exceptions

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
