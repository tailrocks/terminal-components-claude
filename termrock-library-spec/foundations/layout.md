# F05 · Measurement, surfaces and responsive layout

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C05  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/ui/layout.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/layout.rs)
- [src/widgets/splitter.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/splitter.rs)
- [src/widgets/panel.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/panel.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
struct Size { width: u16, height: u16 }
struct Constraints { min: Size, max: Size }
enum Track { Fixed(u16), Flex(u16), Auto }
trait Measure { fn measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size; }
layout::rows(area: Rect, tracks: &[Track], gap: u16) -> Vec<Rect>
layout::columns(area: Rect, tracks: &[Track], gap: u16) -> Vec<Rect>
layout::action_row(area: Rect, sizes: &[Size], align: Alignment, gap: u16) -> Vec<Rect>
layout::responsive_columns(area: Rect, spec: ResponsiveSpec) -> ResponsiveAreas
Ui::with_surface<R>(surface: Surface, body: impl FnOnce(&mut Ui<'_>) -> R) -> R
Ui::clip<R>(area: Rect, body: impl FnOnce(&mut Ui<'_>) -> R) -> R
```

## Contract

1. One geometry calculation serves draw and hit registration. All arithmetic handles zero size, nonzero origins and available-space exhaustion without panics or wrapping.
2. Pure measure reads props, theme design metrics and constraints; state-dependent models have an explicit immutable model/state parameter. Never mutate text, focus or active choices while measuring.
3. Containers establish an inherited surface. Children do not accept a trailing raw background color, and widgets do not infer surface identity by comparing RGB values.
4. Responsive layout yields allocation facts (side-by-side versus overlay/drawer) and leaves content/focus decisions to the caller. Do not add a domain-specific WorkspaceShell API.
5. The 72x20 limit belongs to a baseline composed fixture, not all widgets. Library tests include local 0x0/1x1 allocations; finite proportions and minima are validated before layout.

## Required proof

- all helpers on0/1 extents and nonzero origins
- exact-fit/one-cell-short action rows
- horizontal/vertical split symmetry
- nested clipping with wide glyph at right edge
- responsive breakpoint roundtrip retains caller state

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
