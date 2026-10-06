# F10 · Public component-author surface

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C52  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/ui/ctx.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/ctx.rs)
- [src/bin/jackin_preview/rain.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/rain.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
author::register(ui: &mut Ui<'_>, region: RegionSpec)
author::paint(ui: &mut Ui<'_>, area: Rect, text: StyledText<'_>, part: Part)
author::request_cursor(ui: &mut Ui<'_>, owner: Id, cursor: CursorSpec)
author::blit_terminal(ui: &mut Ui<'_>, area: Rect, source: &dyn TerminalSource)
Ui::part<R>(owner: Id, part: Part, area: Rect, painter: impl FnOnce(&mut PartUi<'_>) -> R) -> R
```

## Contract

1. Application-author facade exposes curated standard widgets; author facade exposes only the constrained vocabulary needed to implement a new generic component or product artwork.
2. Public component authors get borrowed styles, clipped part/row/cell paint, region registration and normalized intents. They do not get raw access to mutable runtime focus/hit/layer storage.
3. Never duplicate Button/List/Grid/Menu painting in a custom component that visually presents one of those controls. Missing styling or behavior belongs in the standard component API.
4. Product rain, warp phrases, logos and instance-entry/exit policies remain outside core. Test-only baseline artwork fixtures may exercise clipping/time APIs; do not grow a particle engine or JackinBrand widget.
5. TerminalView is the narrowly specified raw-cell exception. It preserves child cell data while participating in runtime clip/cursor/input ownership. Standard UI surfaces must not use raw blitting to conceal broken standard widgets.
6. Draw purity includes custom slots: review callbacks for hidden mutable captures/IO and test before/after model snapshots. Fn is an API constraint, not a complete side-effect prohibition.

## Required proof

- external consumer crate defines keyed custom component without private imports
- custom part cannot paint outside reserved rect
- custom author reorder/remove behavior shares runtime hit semantics
- mutation test on standard Button changes every composition that contains it
- terminal raw-cell path cannot overwrite outer modal

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
